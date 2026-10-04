#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use wuxian_horror_ch1::{
    effects::{
        CapabilityCategory, EffectLifetime, EffectResolver, EffectSourceKind, EffectSpec,
        HazardResistanceEffect, HazardTag, StackRule, TerrainTag,
    },
    formal_runtime::{build_v6::PlayerProgressionV6, FormalRuntime},
    player_rules::{
        BoundaryTag, EffectivePlayerRules, KnowledgeLevel, MovementContext, MovementMode,
        ObstacleTag, FULL_DAMAGE_BPS,
    },
    save_slots,
    save_v5::{self, InventoryItemV5},
    save_v6,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

fn test_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "wuxian-build-v6-test-{}-{}",
        std::process::id(),
        NEXT_DIR.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&path);
    path
}

#[test]
fn trusted_equipment_effect_survives_save_continue_and_unequips_immediately() {
    let root = test_dir();
    let expected_rules;
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        assert!(!runtime.build_snapshot().unwrap().2.perception.rear_view);
        runtime.grant_trusted_build_item("rear_view_charm").unwrap();
        runtime.grant_trusted_build_item("rear_view_lens").unwrap();
        runtime.equip_build_item("rear_view_charm").unwrap();
        runtime.equip_build_item("rear_view_lens").unwrap();
        let (build, sources, rules) = runtime.build_snapshot().unwrap();
        assert_eq!(build.inventory.get("rear_view_charm"), Some(&1));
        assert_eq!(
            build.equipment.get("charm").map(String::as_str),
            Some("rear_view_charm")
        );
        assert_eq!(sources.len(), 2);
        assert!(rules.perception.rear_view);
        expected_rules = rules;
        runtime.save().unwrap();
        let saved = save_v6::read_save(&root).unwrap();
        assert_eq!(saved.effect_sources_version, 2);
        assert!(!serde_json::to_value(&saved)
            .unwrap()
            .to_string()
            .contains("effectiveRules"));
    }
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        let (_, sources, rules) = runtime.build_snapshot().unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(rules, expected_rules);
        runtime.unequip_build_slot("charm").unwrap();
        let (_, remaining_sources, after_first_unequip) = runtime.build_snapshot().unwrap();
        assert_eq!(remaining_sources.len(), 1);
        assert!(after_first_unequip.perception.rear_view);
        runtime.unequip_build_slot("lens").unwrap();
        let (_, remaining_sources, after) = runtime.build_snapshot().unwrap();
        assert!(remaining_sources.is_empty());
        assert!(!after.perception.rear_view);
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn arbitrary_client_effect_data_is_not_an_equipment_command() {
    let root = test_dir();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.grant_trusted_build_item("rear_view_charm").unwrap();
    let before = runtime.build_snapshot().unwrap();
    assert_eq!(
        runtime
            .equip_build_item("client_supplied_effect_source")
            .unwrap_err(),
        "E_BUILD_CONTENT_NOT_FOUND"
    );
    assert_eq!(runtime.build_snapshot().unwrap(), before);
    drop(runtime);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn v5_migration_preserves_inventory_and_original_file_without_guessing_build() {
    let root = test_dir();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.save().unwrap();
    let mut legacy = save_v6::read_save(&root).unwrap().save;
    legacy.inventory.items.push(InventoryItemV5 {
        item_id: "legacy_scrap".into(),
        quantity: 3,
    });
    save_v5::write_save(&root, &legacy).unwrap();
    std::fs::remove_file(save_v6::save_path(&root)).unwrap();
    drop(runtime);

    let original = std::fs::read(save_v5::save_path(&root)).unwrap();
    let migrated = save_v6::read_or_migrate(&root).unwrap();
    assert_eq!(migrated.schema_version, 6);
    assert_eq!(migrated.effect_sources_version, 2);
    assert_eq!(migrated.progression.inventory.get("legacy_scrap"), Some(&3));
    assert!(migrated.progression.equipment.is_empty());
    assert!(migrated.progression.owned_skills.is_empty());
    assert!(migrated.progression.bloodline.is_none());
    assert_eq!(
        (
            migrated.effect_sources.clone(),
            migrated.resolve_rules().unwrap()
        ),
        migrated
            .progression
            .resolve_rules_with_capabilities(&legacy.capabilities, &legacy.rear_view)
            .unwrap()
    );
    assert_eq!(migrated.save, legacy);
    assert_eq!(std::fs::read(save_v5::save_path(&root)).unwrap(), original);
    assert!(save_v6::save_path(&root).is_file());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn corrupt_v5_migration_fails_without_writing_v6_or_changing_source() {
    let root = test_dir();
    std::fs::create_dir_all(&root).unwrap();
    let source = save_v5::save_path(&root);
    let original = b"{broken legacy save";
    std::fs::write(&source, original).unwrap();
    assert_eq!(
        save_v6::read_or_migrate(&root).unwrap_err(),
        "E_SAVE_CORRUPT"
    );
    assert_eq!(std::fs::read(source).unwrap(), original);
    assert!(!save_v6::save_path(&root).exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn v6_rejects_tampered_effect_sources_and_capability_build() {
    let root = test_dir();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.grant_trusted_build_item("rear_view_charm").unwrap();
    runtime.equip_build_item("rear_view_charm").unwrap();
    runtime.save().unwrap();
    let saved = save_v6::read_save(&root).unwrap();

    let mut missing_source = saved.clone();
    missing_source.effect_sources.clear();
    assert_eq!(
        missing_source.validate().unwrap_err(),
        "E_SAVE_EFFECT_SOURCES_INVALID"
    );

    let mut forged_capability = saved;
    forged_capability
        .progression
        .capabilities
        .push("perception.rear_view_i".into());
    assert_eq!(
        forged_capability.validate().unwrap_err(),
        "E_SAVE_CAPABILITY_BUILD_MISMATCH"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn v5_slot_migrates_to_v6_without_replacing_the_original_slot() {
    let root = test_dir();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.save().unwrap();
    let mut legacy = save_v6::read_save(&root).unwrap().save;
    legacy.inventory.items.push(InventoryItemV5 {
        item_id: "legacy_token".into(),
        quantity: 2,
    });
    save_slots::create_slot(&root, "legacy", "Legacy", &legacy).unwrap();
    let source = root.join("slots/legacy/slot-v5.json");
    let original = std::fs::read(&source).unwrap();
    let (name, migrated) = save_slots::read_slot_v6(&root, "legacy").unwrap();
    assert_eq!(name, "Legacy");
    assert_eq!(migrated.effect_sources_version, 2);
    assert_eq!(
        (
            migrated.effect_sources.clone(),
            migrated.resolve_rules().unwrap()
        ),
        migrated
            .progression
            .resolve_rules_with_capabilities(&legacy.capabilities, &legacy.rear_view)
            .unwrap()
    );
    assert_eq!(migrated.progression.inventory.get("legacy_token"), Some(&2));
    assert!(root.join("slots/legacy/slot-v6.json").is_file());
    assert_eq!(std::fs::read(source).unwrap(), original);
    let _ = std::fs::remove_dir_all(root);
}

fn equipped_build(items: &[&str]) -> PlayerProgressionV6 {
    let mut build = PlayerProgressionV6::default();
    for item in items {
        build.grant_trusted_item(item).unwrap();
        build.equip(item).unwrap();
    }
    build
}

#[test]
fn native_catalog_preserves_existing_rear_view_source_identity() {
    let build = equipped_build(&["rear_view_charm", "rear_view_lens"]);
    let (sources, _) = build.resolve_rules().unwrap();
    let expected = serde_json::json!([
        {
            "source_kind": "equipment", "source_id": "rear_view_charm",
            "instance_id": "equipment:charm:rear_view_charm", "lifetime": "equipped",
            "ui_category": "perception", "effects": [{"perception": {"tag": "rear_view", "stack": "any"}}]
        },
        {
            "source_kind": "equipment", "source_id": "rear_view_lens",
            "instance_id": "equipment:lens:rear_view_lens", "lifetime": "equipped",
            "ui_category": "perception", "effects": [{"perception": {"tag": "rear_view", "stack": "any"}}]
        }
    ]);
    assert_eq!(serde_json::to_value(sources).unwrap(), expected);
}

#[test]
fn owned_internal_samples_do_not_grant_rules_before_equipping() {
    let mut build = PlayerProgressionV6::default();
    for item in [
        "map_fog_bypass_array",
        "terrain_penalty_ignore_boots",
        "heat_resistance_lining",
        "water_movement_module",
    ] {
        build.grant_trusted_item(item).unwrap();
    }
    let (sources, rules) = build.resolve_rules().unwrap();
    assert!(sources.is_empty());
    assert_eq!(rules, EffectivePlayerRules::default());
    // Unknown legacy inventory is preserved for migration, but cannot become a source.
    build.inventory.insert("legacy_unknown_item".into(), 1);
    assert_eq!(build.resolve_rules().unwrap().1, rules);
    let before = build.clone();
    assert_eq!(
        build.equip("legacy_unknown_item").unwrap_err(),
        "E_BUILD_CONTENT_NOT_FOUND"
    );
    assert_eq!(build, before);
}

#[test]
fn map_samples_merge_only_authored_channels_and_remove_independently() {
    let mut build = equipped_build(&["map_fog_bypass_array", "map_topology_chip"]);
    let (_, rules) = build.resolve_rules().unwrap();
    assert_eq!(rules.map.topology, KnowledgeLevel::Full);
    assert_eq!(rules.map.terrain, KnowledgeLevel::Full);
    assert_eq!(rules.map.connections, KnowledgeLevel::Full);
    for level in [
        rules.map.objectives,
        rules.map.enemies,
        rules.map.hazards,
        rules.map.loot,
        rules.map.npcs,
        rules.map.secrets,
    ] {
        assert_eq!(level, KnowledgeLevel::None);
    }
    assert_eq!(rules.map.reveal_radius_mm, 0);
    build.unequip("map_array").unwrap();
    let (_, remaining) = build.resolve_rules().unwrap();
    assert_eq!(remaining.map.topology, KnowledgeLevel::Full);
    assert_eq!(remaining.map.terrain, KnowledgeLevel::None);
    assert_eq!(remaining.map.connections, KnowledgeLevel::None);
    build.unequip("map_chip").unwrap();
    assert_eq!(
        build.resolve_rules().unwrap().1,
        EffectivePlayerRules::default()
    );
}

#[test]
fn terrain_samples_merge_penalty_permissions_without_opening_walls_or_deep_water() {
    let mut build = equipped_build(&[
        "terrain_penalty_ignore_boots",
        "water_penalty_ignore_module",
    ]);
    let (_, rules) = build.resolve_rules().unwrap();
    let shallow = MovementContext {
        terrain: Some(TerrainTag::WaterShallow),
        ..MovementContext::default()
    };
    assert_eq!(
        rules
            .resolve_movement(MovementMode::Ground, shallow)
            .movement_multiplier_bps,
        FULL_DAMAGE_BPS
    );
    for context in [
        MovementContext {
            obstacle: Some(ObstacleTag::Wall),
            ..shallow
        },
        MovementContext {
            boundary: Some(BoundaryTag::World),
            ..shallow
        },
        MovementContext {
            boundary: Some(BoundaryTag::Quest),
            ..shallow
        },
        MovementContext {
            terrain: Some(TerrainTag::WaterDeep),
            ..MovementContext::default()
        },
    ] {
        assert!(
            !rules
                .resolve_movement(MovementMode::Ground, context)
                .allowed
        );
    }
    assert!(rules.terrain.passable.is_empty());
    build.unequip("boots").unwrap();
    let (_, remaining) = build.resolve_rules().unwrap();
    assert!(remaining
        .terrain
        .ignored_move_penalties
        .contains(&TerrainTag::WaterShallow));
    assert!(!remaining
        .terrain
        .ignored_move_penalties
        .contains(&TerrainTag::Mud));
    build.unequip("water_module").unwrap();
    let (_, empty) = build.resolve_rules().unwrap();
    assert_eq!(
        empty
            .resolve_movement(MovementMode::Ground, shallow)
            .movement_multiplier_bps,
        6_000
    );
    assert_eq!(empty, EffectivePlayerRules::default());
}

#[test]
fn heat_samples_multiply_remaining_damage_and_unequip_one_source_at_a_time() {
    let mut build = equipped_build(&["heat_resistance_lining", "heat_resistance_charm"]);
    let (_, rules) = build.resolve_rules().unwrap();
    assert_eq!(rules.hazards.remaining_damage_bps(HazardTag::Heat), 3_750);
    assert_eq!(
        rules.hazards.remaining_damage_bps(HazardTag::Machinery),
        FULL_DAMAGE_BPS
    );
    assert_eq!(
        rules.hazards.remaining_damage_bps(HazardTag::Water),
        FULL_DAMAGE_BPS
    );
    assert!(rules.hazards.immune.is_empty());
    build.unequip("lining").unwrap();
    assert_eq!(
        build
            .resolve_rules()
            .unwrap()
            .1
            .hazards
            .remaining_damage_bps(HazardTag::Heat),
        7_500
    );
    build.unequip("charm").unwrap();
    assert_eq!(
        build.resolve_rules().unwrap().1,
        EffectivePlayerRules::default()
    );
}

#[test]
fn water_samples_stack_by_max_and_penalty_ignore_takes_precedence() {
    let mut build = equipped_build(&["water_movement_module", "water_movement_charm"]);
    let shallow = MovementContext {
        terrain: Some(TerrainTag::WaterShallow),
        ..MovementContext::default()
    };
    let (_, rules) = build.resolve_rules().unwrap();
    assert_eq!(
        rules
            .resolve_movement(MovementMode::Ground, shallow)
            .movement_multiplier_bps,
        9_000
    );
    assert!(rules.terrain.passable.is_empty());
    for context in [
        MovementContext {
            obstacle: Some(ObstacleTag::Wall),
            ..shallow
        },
        MovementContext {
            boundary: Some(BoundaryTag::World),
            ..shallow
        },
        MovementContext {
            boundary: Some(BoundaryTag::Quest),
            ..shallow
        },
        MovementContext {
            terrain: Some(TerrainTag::WaterDeep),
            ..MovementContext::default()
        },
    ] {
        assert!(
            !rules
                .resolve_movement(MovementMode::Ground, context)
                .allowed
        );
    }
    build
        .grant_trusted_item("terrain_penalty_ignore_boots")
        .unwrap();
    build.equip("terrain_penalty_ignore_boots").unwrap();
    assert_eq!(
        build
            .resolve_rules()
            .unwrap()
            .1
            .resolve_movement(MovementMode::Ground, shallow)
            .movement_multiplier_bps,
        FULL_DAMAGE_BPS
    );
    build.unequip("boots").unwrap();
    assert_eq!(
        build
            .resolve_rules()
            .unwrap()
            .1
            .resolve_movement(MovementMode::Ground, shallow)
            .movement_multiplier_bps,
        9_000
    );
    build.unequip("charm").unwrap();
    assert_eq!(
        build
            .resolve_rules()
            .unwrap()
            .1
            .resolve_movement(MovementMode::Ground, shallow)
            .movement_multiplier_bps,
        8_000
    );
    build.unequip("water_module").unwrap();
    assert_eq!(
        build.resolve_rules().unwrap().1,
        EffectivePlayerRules::default()
    );
}

#[test]
fn catalog_rules_are_byte_equivalent_under_equip_source_and_effect_order_changes() {
    let items = [
        "rear_view_lens",
        "map_fog_bypass_array",
        "map_topology_chip",
        "terrain_penalty_ignore_boots",
        "heat_resistance_lining",
        "water_movement_module",
        "water_movement_charm",
    ];
    let forward = equipped_build(&items);
    let reversed = equipped_build(&items.into_iter().rev().collect::<Vec<_>>());
    let (sources, rules) = forward.resolve_rules().unwrap();
    let expected = serde_json::to_vec(&rules).unwrap();
    assert_eq!(
        forward.resolve_rules().unwrap(),
        reversed.resolve_rules().unwrap()
    );
    let mut shuffled = sources;
    shuffled.reverse();
    for source in &mut shuffled {
        source.effects.reverse();
    }
    assert_eq!(
        serde_json::to_vec(&EffectResolver::resolve(&shuffled).unwrap()).unwrap(),
        expected
    );
}

#[test]
fn catalog_samples_reresolve_identical_rules_after_save_close_continue() {
    let root = test_dir();
    let expected;
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        for item in [
            "map_fog_bypass_array",
            "terrain_penalty_ignore_boots",
            "heat_resistance_lining",
            "water_movement_module",
        ] {
            runtime.grant_trusted_build_item(item).unwrap();
            runtime.equip_build_item(item).unwrap();
        }
        expected = runtime.build_snapshot().unwrap();
        runtime.save().unwrap();
        let saved = save_v6::read_save(&root).unwrap();
        assert_eq!(saved.resolve_rules().unwrap(), expected.2);
        let encoded = serde_json::to_value(&saved).unwrap();
        assert_eq!(encoded["effectSourcesVersion"], 2);
        assert!(encoded.get("effectiveRules").is_none());
        assert!(encoded.get("effective_rules_v6").is_none());
        assert_eq!(saved.effect_sources, expected.1);
    }
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(runtime.build_snapshot().unwrap(), expected);
        for slot in ["map_array", "boots", "lining", "water_module"] {
            runtime.unequip_build_slot(slot).unwrap();
        }
        let (_, sources, rules) = runtime.build_snapshot().unwrap();
        assert!(sources.is_empty());
        assert_eq!(rules, EffectivePlayerRules::default());
        runtime.save().unwrap();
    }
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        let (build, sources, rules) = runtime.build_snapshot().unwrap();
        assert_eq!(build.inventory.len(), 4);
        assert!(build.equipment.is_empty());
        assert!(sources.is_empty());
        assert_eq!(rules, EffectivePlayerRules::default());
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn v6_rejects_forged_catalog_source_identity_category_lifetime_and_effects() {
    let root = test_dir();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime
        .grant_trusted_build_item("water_movement_module")
        .unwrap();
    runtime.equip_build_item("water_movement_module").unwrap();
    runtime.save().unwrap();
    let saved = save_v6::read_save(&root).unwrap();
    for mutation in 0..8 {
        let mut forged = saved.clone();
        let source = &mut forged.effect_sources[0];
        match mutation {
            0 => source.source_id = "unknown_source".into(),
            1 => source.instance_id = "equipment:other:water_movement_module".into(),
            2 => source.source_kind = EffectSourceKind::Bloodline,
            3 => source.lifetime = EffectLifetime::Permanent,
            4 => source.ui_category = Some(CapabilityCategory::Body),
            5 => source
                .effects
                .push(EffectSpec::HazardResistance(HazardResistanceEffect {
                    tag: HazardTag::Heat,
                    reduction_bps: 5_000,
                    stack: StackRule::MultiplyRemaining,
                })),
            6 => source.effects.clear(),
            7 => {
                let EffectSpec::TerrainMovementModifier(effect) = &mut source.effects[0] else {
                    panic!("water sample must retain its typed movement effect");
                };
                effect.multiplier_bps = FULL_DAMAGE_BPS;
            }
            _ => unreachable!(),
        }
        assert_eq!(
            forged.validate().unwrap_err(),
            "E_SAVE_EFFECT_SOURCES_INVALID"
        );
        assert!(forged.resolve_rules().is_err());
    }
    let mut unknown_equipment = saved.clone();
    unknown_equipment
        .progression
        .inventory
        .insert("unknown_source".into(), 1);
    unknown_equipment
        .progression
        .equipment
        .insert("water_module".into(), "unknown_source".into());
    assert_eq!(
        unknown_equipment.validate().unwrap_err(),
        "E_BUILD_EQUIPMENT_INVALID"
    );
    let mut wrong_slot = saved.clone();
    wrong_slot.progression.equipment.clear();
    wrong_slot
        .progression
        .equipment
        .insert("boots".into(), "water_movement_module".into());
    assert_eq!(
        wrong_slot.validate().unwrap_err(),
        "E_BUILD_EQUIPMENT_INVALID"
    );
    let mut unowned = saved.clone();
    unowned.progression.inventory.clear();
    assert_eq!(unowned.validate().unwrap_err(), "E_BUILD_EQUIPMENT_INVALID");
    let mut injected_rules = serde_json::to_value(&saved).unwrap();
    injected_rules["effectiveRules"] = serde_json::json!({"map": {"topology": "full"}});
    assert!(serde_json::from_value::<save_v6::SaveV6>(injected_rules).is_err());
    assert_eq!(save_v6::read_save(&root).unwrap(), saved);
    drop(runtime);
    let _ = std::fs::remove_dir_all(root);
}
