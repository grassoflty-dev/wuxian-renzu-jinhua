//! Trusted, server-only bloodline lifecycle and durable source compatibility.
//! These samples are not public content or a player-facing acquisition path.

#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use wuxian_horror_ch1::{
    capability_v1::{apply_command_at_revision, CapabilityCommand, CAP_LOCAL_MAP},
    effects::{
        CapabilityCategory, EffectLifetime, EffectResolver, EffectSource, EffectSourceKind,
        EffectSpec, HazardTag,
    },
    formal_runtime::FormalRuntime,
    player_rules::{EffectivePlayerRules, KnowledgeLevel, FULL_DAMAGE_BPS},
    save_slots,
    save_v5::{self, InventoryItemV5},
    save_v6::{self, SaveV6},
};

const STRONG: &str = "internal_thermal_adaptation";
const LIGHT: &str = "internal_thermal_adaptation_light";
const LEGACY: &str = "legacy_unknown_lineage";
static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "wuxian-bloodline-v6-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn runtime(root: &TestRoot) -> FormalRuntime {
    let runtime = FormalRuntime::new_with_save_dir(root.0.clone()).unwrap();
    runtime.pause().unwrap();
    runtime
}

fn empty_save(root: &TestRoot) -> SaveV6 {
    {
        let runtime = runtime(root);
        runtime.save().unwrap();
    }
    save_v6::read_save(root.path()).unwrap()
}

fn equip(runtime: &FormalRuntime, id: &str) {
    runtime.grant_trusted_build_item(id).unwrap();
    runtime.equip_build_item(id).unwrap();
}

fn heat_remaining(runtime: &FormalRuntime) -> u16 {
    runtime
        .build_snapshot()
        .unwrap()
        .2
        .hazards
        .remaining_damage_bps(HazardTag::Heat)
}

// Frozen old-wire inputs remain independent of the current SaveV6 serializer/resolver.
fn legacy_save_value(with_map: bool) -> serde_json::Value {
    if with_map {
        let rows: Vec<serde_json::Value> = serde_json::from_str(include_str!(
            "fixtures/capability-save-profiles/legacy-matrix.json"
        ))
        .unwrap();
        rows.into_iter()
            .find(|row| row["name"] == CAP_LOCAL_MAP)
            .unwrap()["save"]
            .clone()
    } else {
        serde_json::from_str(include_str!(
            "fixtures/capability-save-profiles/legacy-empty.json"
        ))
        .unwrap()
    }
}

fn put_legacy_save(root: &TestRoot, raw: &serde_json::Value) -> SaveV6 {
    assert!(raw.get("effectSourcesVersion").is_none());
    std::fs::create_dir_all(root.path()).unwrap();
    let original = serde_json::to_vec_pretty(raw).unwrap();
    std::fs::write(save_v6::save_path(root.path()), &original).unwrap();
    let saved = save_v6::read_save(root.path()).unwrap();
    assert_eq!(
        std::fs::read(save_v6::save_path(root.path())).unwrap(),
        original
    );
    saved
}

fn expected_local_map_source() -> EffectSource {
    serde_json::from_value(serde_json::json!({
        "source_kind": "innate_capability", "source_id": CAP_LOCAL_MAP,
        "instance_id": format!("capability:{CAP_LOCAL_MAP}"), "lifetime": "selected",
        "ui_category": "information", "effects": [
            {"map_knowledge_level": {"tag": "topology", "level": "explored_only", "stack": "max"}},
            {"map_knowledge_level": {"tag": "terrain", "level": "explored_only", "stack": "max"}},
            {"map_knowledge_level": {"tag": "connections", "level": "explored_only", "stack": "max"}},
            {"map_knowledge_level": {"tag": "objectives", "level": "known", "stack": "max"}}
        ]
    }))
    .unwrap()
}

fn expected_local_map_rules() -> EffectivePlayerRules {
    let mut rules = EffectivePlayerRules::default();
    rules.map.topology = KnowledgeLevel::ExploredOnly;
    rules.map.terrain = KnowledgeLevel::ExploredOnly;
    rules.map.connections = KnowledgeLevel::ExploredOnly;
    rules.map.objectives = KnowledgeLevel::Known;
    rules
}

fn add_legacy_map_grant(save: &mut SaveV6) {
    apply_command_at_revision(
        &mut save.save.capabilities,
        CapabilityCommand::Grant {
            capability_id: CAP_LOCAL_MAP.into(),
        },
        save.save.revision,
    )
    .unwrap();
    apply_command_at_revision(
        &mut save.save.capabilities,
        CapabilityCommand::Select {
            capability_ids: vec![CAP_LOCAL_MAP.into()],
        },
        save.save.revision,
    )
    .unwrap();
    save.progression.sync_capabilities(&save.save.capabilities);
}

#[test]
fn trusted_grant_has_canonical_owned_identity_and_only_authored_heat_rules() {
    let root = TestRoot::new();
    let runtime = runtime(&root);
    assert_eq!(
        runtime.build_snapshot().unwrap().2,
        EffectivePlayerRules::default()
    );
    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    let (build, sources, rules) = runtime.build_snapshot().unwrap();
    assert_eq!(build.bloodline.as_deref(), Some(STRONG));
    assert_eq!(build.bloodline_tier, 1);
    assert_eq!(build.bloodline_proficiency, 0);
    assert!(build.inventory.is_empty());
    assert!(build.equipment.is_empty());
    assert!(build.capabilities.is_empty());
    assert_eq!(
        serde_json::to_value(&sources).unwrap(),
        serde_json::json!([{
            "source_kind": "bloodline",
            "source_id": STRONG,
            "instance_id": format!("bloodline:{STRONG}"),
            "lifetime": "owned",
            "ui_category": "body",
            "effects": [{"hazard_resistance": {
                "tag": "heat", "reduction_bps": 5000, "stack": "multiply_remaining"
            }}]
        }])
    );
    assert_eq!(rules.hazards.remaining_damage_bps(HazardTag::Heat), 5_000);
    assert_eq!(
        rules.hazards.remaining_damage_bps(HazardTag::Water),
        FULL_DAMAGE_BPS
    );
    assert_eq!(
        rules.hazards.remaining_damage_bps(HazardTag::Machinery),
        FULL_DAMAGE_BPS
    );
    assert!(rules.hazards.immune.is_empty());
    let mut without_heat = rules;
    without_heat.hazards = Default::default();
    assert_eq!(without_heat, EffectivePlayerRules::default());
}

#[test]
fn commands_reject_unknown_occupied_stale_same_and_empty_targets_without_build_mutation() {
    let root = TestRoot::new();
    let runtime = runtime(&root);
    let empty = runtime.build_snapshot().unwrap();
    for id in [LEGACY, "client_supplied_effect_source", "INVALID ID"] {
        assert!(runtime.grant_trusted_build_bloodline(id).is_err());
        assert_eq!(runtime.build_snapshot().unwrap(), empty);
    }
    assert!(runtime.remove_trusted_build_bloodline(STRONG).is_err());
    assert!(runtime
        .replace_trusted_build_bloodline(STRONG, LIGHT)
        .is_err());
    assert_eq!(runtime.build_snapshot().unwrap(), empty);

    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    let acquired = runtime.build_snapshot().unwrap();
    assert!(runtime.grant_trusted_build_bloodline(STRONG).is_err());
    assert!(runtime.grant_trusted_build_bloodline(LIGHT).is_err());
    assert!(runtime
        .replace_trusted_build_bloodline(LIGHT, STRONG)
        .is_err());
    assert!(runtime
        .replace_trusted_build_bloodline(STRONG, STRONG)
        .is_err());
    assert!(runtime
        .replace_trusted_build_bloodline(STRONG, LEGACY)
        .is_err());
    assert!(runtime.remove_trusted_build_bloodline(LIGHT).is_err());
    assert_eq!(runtime.build_snapshot().unwrap(), acquired);

    runtime.remove_trusted_build_bloodline(STRONG).unwrap();
    assert_eq!(runtime.build_snapshot().unwrap(), empty);
    assert!(runtime.remove_trusted_build_bloodline(STRONG).is_err());
    assert_eq!(runtime.build_snapshot().unwrap(), empty);
}

#[test]
fn equipment_and_bloodline_resistance_stack_and_remove_independently_in_both_orders() {
    let root = TestRoot::new();
    let runtime = runtime(&root);
    equip(&runtime, "heat_resistance_lining");
    let equipment_sources = runtime.build_snapshot().unwrap().1;
    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    let (_, sources, _) = runtime.build_snapshot().unwrap();
    assert_eq!(
        &sources[..equipment_sources.len()],
        equipment_sources.as_slice()
    );
    assert_eq!(
        sources.last().unwrap().source_kind,
        EffectSourceKind::Bloodline
    );
    assert_eq!(heat_remaining(&runtime), 2_500);

    runtime.remove_trusted_build_bloodline(STRONG).unwrap();
    assert_eq!(runtime.build_snapshot().unwrap().1, equipment_sources);
    assert_eq!(heat_remaining(&runtime), 5_000);
    runtime.unequip_build_slot("lining").unwrap();
    assert_eq!(heat_remaining(&runtime), FULL_DAMAGE_BPS);

    runtime.grant_trusted_build_bloodline(STRONG).unwrap();
    runtime.equip_build_item("heat_resistance_lining").unwrap();
    assert_eq!(heat_remaining(&runtime), 2_500);
    runtime.unequip_build_slot("lining").unwrap();
    assert_eq!(heat_remaining(&runtime), 5_000);
    runtime.remove_trusted_build_bloodline(STRONG).unwrap();
    assert_eq!(heat_remaining(&runtime), FULL_DAMAGE_BPS);
}

#[test]
fn bloodline_composition_is_deterministic_across_command_and_source_order() {
    let first_root = TestRoot::new();
    let second_root = TestRoot::new();
    let first = runtime(&first_root);
    let second = runtime(&second_root);
    first.grant_trusted_build_bloodline(STRONG).unwrap();
    equip(&first, "heat_resistance_lining");
    equip(&first, "rear_view_lens");
    equip(&second, "rear_view_lens");
    equip(&second, "heat_resistance_lining");
    second.grant_trusted_build_bloodline(STRONG).unwrap();
    let expected = first.build_snapshot().unwrap();
    assert_eq!(second.build_snapshot().unwrap(), expected);
    let mut reordered = expected.1;
    reordered.reverse();
    for source in &mut reordered {
        source.effects.reverse();
    }
    assert_eq!(
        serde_json::to_vec(&EffectResolver::resolve(&reordered).unwrap()).unwrap(),
        serde_json::to_vec(&expected.2).unwrap()
    );
}

#[test]
fn explicit_replace_resets_proficiency_and_changes_only_the_bloodline_source() {
    let root = TestRoot::new();
    {
        let runtime = runtime(&root);
        equip(&runtime, "heat_resistance_lining");
        runtime.grant_trusted_build_bloodline(STRONG).unwrap();
        runtime.save().unwrap();
    }
    let mut saved = save_v6::read_save(root.path()).unwrap();
    saved.progression.bloodline_proficiency = 734;
    save_v6::write_save(root.path(), &saved).unwrap();
    let expected_equipment = saved.effect_sources[0].clone();
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(
            runtime.build_snapshot().unwrap().0.bloodline_proficiency,
            734
        );
        runtime
            .replace_trusted_build_bloodline(STRONG, LIGHT)
            .unwrap();
        let (build, sources, _) = runtime.build_snapshot().unwrap();
        assert_eq!(build.bloodline.as_deref(), Some(LIGHT));
        assert_eq!((build.bloodline_tier, build.bloodline_proficiency), (1, 0));
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0], expected_equipment);
        assert_eq!(sources[1].source_id, LIGHT);
        assert_eq!(sources[1].instance_id, format!("bloodline:{LIGHT}"));
        assert_eq!(sources[1].lifetime, EffectLifetime::Owned);
        assert_eq!(heat_remaining(&runtime), 3_750);
        assert!(!sources.iter().any(|source| source.source_id == STRONG));
        runtime.save().unwrap();
    }
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(
            runtime.build_snapshot().unwrap().0.bloodline.as_deref(),
            Some(LIGHT)
        );
        assert_eq!(heat_remaining(&runtime), 3_750);
        assert!(runtime.remove_trusted_build_bloodline(STRONG).is_err());
        runtime.remove_trusted_build_bloodline(LIGHT).unwrap();
        assert_eq!(heat_remaining(&runtime), 5_000);
    }
}

#[test]
fn bloodline_source_survives_save_drop_fresh_continue_and_stays_removed_after_resaving() {
    let root = TestRoot::new();
    let expected;
    {
        let runtime = runtime(&root);
        equip(&runtime, "rear_view_lens");
        runtime.grant_trusted_build_bloodline(STRONG).unwrap();
        expected = runtime.build_snapshot().unwrap();
        runtime.save().unwrap();
        let saved = save_v6::read_save(root.path()).unwrap();
        assert_eq!(saved.effect_sources, expected.1);
        assert_eq!(saved.resolve_rules().unwrap(), expected.2);
        let encoded = serde_json::to_value(saved).unwrap();
        assert!(encoded.get("effectiveRules").is_none());
        assert!(encoded.get("effective_rules_v6").is_none());
    }
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(runtime.build_snapshot().unwrap(), expected);
        runtime.remove_trusted_build_bloodline(STRONG).unwrap();
        let (build, sources, rules) = runtime.build_snapshot().unwrap();
        assert!(build.bloodline.is_none());
        assert_eq!((build.bloodline_tier, build.bloodline_proficiency), (0, 0));
        assert_eq!(sources.len(), 1);
        assert!(rules.perception.rear_view);
        assert_eq!(heat_remaining(&runtime), FULL_DAMAGE_BPS);
        runtime.save().unwrap();
    }
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        let (build, sources, rules) = runtime.build_snapshot().unwrap();
        assert!(build.bloodline.is_none());
        assert_eq!((build.bloodline_tier, build.bloodline_proficiency), (0, 0));
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_kind, EffectSourceKind::Equipment);
        assert!(rules.perception.rear_view);
        assert_eq!(heat_remaining(&runtime), FULL_DAMAGE_BPS);
    }
}

#[test]
fn bloodline_lifecycle_round_trips_through_save_slots() {
    let root = TestRoot::new();
    let expected;
    {
        let runtime = runtime(&root);
        equip(&runtime, "heat_resistance_lining");
        runtime.grant_trusted_build_bloodline(LIGHT).unwrap();
        expected = runtime.build_snapshot().unwrap();
        runtime.save_slot("lineage", "Lineage", true).unwrap();
    }
    let (name, saved) = save_slots::read_slot_v6(root.path(), "lineage").unwrap();
    assert_eq!(name, "Lineage");
    assert_eq!(saved.effect_sources, expected.1);
    assert_eq!(saved.resolve_rules().unwrap(), expected.2);
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_slot("lineage").unwrap());
        assert_eq!(runtime.build_snapshot().unwrap(), expected);
        runtime
            .replace_trusted_build_bloodline(LIGHT, STRONG)
            .unwrap();
        runtime.save_slot("lineage", "Lineage", false).unwrap();
    }
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_slot("lineage").unwrap());
        assert_eq!(heat_remaining(&runtime), 2_500);
        runtime.remove_trusted_build_bloodline(STRONG).unwrap();
        runtime.save_slot("lineage", "Lineage", false).unwrap();
    }
    {
        let runtime = runtime(&root);
        acknowledge_ready(&runtime, runtime.continue_slot("lineage").unwrap());
        assert!(runtime.build_snapshot().unwrap().0.bloodline.is_none());
        assert_eq!(heat_remaining(&runtime), 5_000);
    }
}

#[test]
fn old_v6_equipment_and_capabilities_normalize_sources_without_rewriting_original_bytes() {
    for with_capability in [false, true] {
        let root = TestRoot::new();
        let mut raw = legacy_save_value(with_capability);
        raw["progression"]["inventory"] = serde_json::json!({"rear_view_lens": 1});
        raw["progression"]["equipment"] = serde_json::json!({"lens": "rear_view_lens"});
        // Literal frozen equipment shape, never synthesized by a current source constructor.
        raw["effectSources"] = serde_json::json!([{
            "source_kind": "equipment", "source_id": "rear_view_lens",
            "instance_id": "equipment:lens:rear_view_lens", "lifetime": "equipped",
            "ui_category": "perception",
            "effects": [{"perception": {"tag": "rear_view", "stack": "any"}}]
        }]);
        let saved = put_legacy_save(&root, &raw);
        assert_eq!(saved.effect_sources_version, 2);
        assert_eq!(
            saved.save,
            serde_json::from_value::<save_v5::SaveV5>(raw["save"].clone()).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&saved.progression).unwrap(),
            raw["progression"]
        );
        let original = std::fs::read(save_v6::save_path(root.path())).unwrap();
        let backup = root
            .path()
            .join("formal-save-v6.json.legacy-effect-sources-v1.bak");
        let mut expected_sources: Vec<EffectSource> =
            serde_json::from_value(raw["effectSources"].clone()).unwrap();
        let mut expected_rules = if with_capability {
            expected_sources.push(expected_local_map_source());
            expected_local_map_rules()
        } else {
            EffectivePlayerRules::default()
        };
        expected_rules.perception.rear_view = true;
        let expected = (expected_sources, expected_rules);
        assert_eq!(
            saved
                .progression
                .resolve_rules_with_capabilities(&saved.save.capabilities, &saved.save.rear_view)
                .unwrap(),
            expected
        );
        {
            let runtime = runtime(&root);
            let view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
            let (build, sources, rules) = runtime.build_snapshot().unwrap();
            assert_eq!(build, saved.progression);
            assert_eq!((sources, rules), expected);
            assert_eq!(
                view.capabilities.items.iter().any(|item| {
                    item.capability_id == CAP_LOCAL_MAP && item.granted && item.selected
                }),
                with_capability
            );
            assert_eq!(
                view.capabilities.map_topology_authorized,
                Some(with_capability)
            );
            assert_eq!(
                std::fs::read(save_v6::save_path(root.path())).unwrap(),
                original
            );
            assert!(
                !backup.exists(),
                "read-only Continue must not create a migration backup"
            );
            runtime.pause().unwrap();
            runtime.save().unwrap();
            assert_eq!(std::fs::read(&backup).unwrap(), original);
            let current = save_v6::read_save(root.path()).unwrap();
            assert_eq!(current.effect_sources_version, 2);
            assert_eq!(current.effect_sources, expected.0);
            assert_eq!(current.resolve_rules().unwrap(), expected.1);
            let encoded: serde_json::Value =
                serde_json::from_slice(&std::fs::read(save_v6::save_path(root.path())).unwrap())
                    .unwrap();
            assert_eq!(encoded["effectSourcesVersion"], 2);
        }
        assert_eq!(std::fs::read(&backup).unwrap(), original);
    }
}

#[test]
fn v5_file_and_slot_migration_preserve_legacy_data_and_default_to_no_bloodline() {
    let root = TestRoot::new();
    let mut seed = empty_save(&root);
    add_legacy_map_grant(&mut seed);
    let mut legacy = seed.save;
    legacy.inventory.items.push(InventoryItemV5 {
        item_id: "legacy_scrap".into(),
        quantity: 3,
    });
    save_v5::write_save(root.path(), &legacy).unwrap();
    save_slots::create_slot(root.path(), "legacy", "Legacy", &legacy).unwrap();
    std::fs::remove_file(save_v6::save_path(root.path())).unwrap();
    let file_path = save_v5::save_path(root.path());
    let slot_path = root.path().join("slots/legacy/slot-v5.json");
    let original_file = std::fs::read(&file_path).unwrap();
    let original_slot = std::fs::read(&slot_path).unwrap();
    let migrated = save_v6::read_or_migrate(root.path()).unwrap();
    let (_, migrated_slot) = save_slots::read_slot_v6(root.path(), "legacy").unwrap();
    for value in [migrated, migrated_slot] {
        assert_eq!(value.save, legacy);
        assert_eq!(value.progression.inventory.get("legacy_scrap"), Some(&3));
        assert_eq!(
            value.progression.capabilities,
            vec![CAP_LOCAL_MAP.to_owned()]
        );
        assert!(value.progression.bloodline.is_none());
        assert_eq!(
            (
                value.progression.bloodline_tier,
                value.progression.bloodline_proficiency
            ),
            (0, 0)
        );
        assert_eq!(value.effect_sources_version, 2);
        assert_eq!(value.effect_sources, vec![expected_local_map_source()]);
        assert_eq!(value.resolve_rules().unwrap(), expected_local_map_rules());
    }
    assert_eq!(std::fs::read(file_path).unwrap(), original_file);
    assert_eq!(std::fs::read(slot_path).unwrap(), original_slot);
}

#[test]
fn legacy_none_and_unknown_metadata_remain_inert_and_round_trip_without_inferred_grants() {
    for id in [None, Some(LEGACY)] {
        let root = TestRoot::new();
        let mut raw = legacy_save_value(false);
        raw["progression"]["bloodline"] = serde_json::json!(id);
        raw["progression"]["bloodlineTier"] = serde_json::json!(73);
        raw["progression"]["bloodlineProficiency"] = serde_json::json!(456_789);
        let saved = put_legacy_save(&root, &raw);
        {
            let runtime = runtime(&root);
            acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
            let (build, sources, rules) = runtime.build_snapshot().unwrap();
            assert_eq!(build, saved.progression);
            assert!(sources.is_empty());
            assert_eq!(rules, EffectivePlayerRules::default());
            runtime.save().unwrap();
        }
        assert_eq!(
            save_v6::read_save(root.path()).unwrap().progression,
            saved.progression
        );
        {
            let runtime = runtime(&root);
            acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
            if let Some(id) = id {
                assert!(runtime.grant_trusted_build_bloodline(STRONG).is_err());
                assert_eq!(runtime.build_snapshot().unwrap().0, saved.progression);
                runtime.replace_trusted_build_bloodline(id, STRONG).unwrap();
            } else {
                runtime.grant_trusted_build_bloodline(STRONG).unwrap();
            }
            let build = runtime.build_snapshot().unwrap().0;
            assert_eq!(build.bloodline.as_deref(), Some(STRONG));
            assert_eq!((build.bloodline_tier, build.bloodline_proficiency), (1, 0));
        }
    }
}

#[test]
fn legacy_unknown_lineage_can_be_explicitly_removed_without_granting_rules() {
    let root = TestRoot::new();
    let mut raw = legacy_save_value(false);
    raw["progression"]["bloodline"] = serde_json::json!(LEGACY);
    raw["progression"]["bloodlineTier"] = serde_json::json!(100);
    raw["progression"]["bloodlineProficiency"] = serde_json::json!(1_000_000);
    put_legacy_save(&root, &raw);
    let runtime = runtime(&root);
    acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert!(runtime.remove_trusted_build_bloodline(STRONG).is_err());
    runtime.remove_trusted_build_bloodline(LEGACY).unwrap();
    let (build, sources, rules) = runtime.build_snapshot().unwrap();
    assert!(build.bloodline.is_none());
    assert_eq!((build.bloodline_tier, build.bloodline_proficiency), (0, 0));
    assert!(sources.is_empty());
    assert_eq!(rules, EffectivePlayerRules::default());
}

#[test]
fn v6_rejects_missing_extra_duplicate_reordered_and_forged_bloodline_sources() {
    let root = TestRoot::new();
    {
        let runtime = runtime(&root);
        equip(&runtime, "rear_view_lens");
        runtime.grant_trusted_build_bloodline(STRONG).unwrap();
        runtime.save().unwrap();
    }
    let saved = save_v6::read_save(root.path()).unwrap();
    let original = std::fs::read(save_v6::save_path(root.path())).unwrap();
    assert_eq!(saved.effect_sources.len(), 2);
    assert_eq!(
        saved.effect_sources[1].source_kind,
        EffectSourceKind::Bloodline
    );
    for mutation in 0..17 {
        let mut forged = saved.clone();
        match mutation {
            0 => {
                forged.effect_sources.pop();
            }
            1 => forged.effect_sources.push(forged.effect_sources[1].clone()),
            2 => forged.effect_sources.swap(0, 1),
            3 => forged.effect_sources[1].source_id = LEGACY.into(),
            4 => forged.effect_sources[1].instance_id = "bloodline:other".into(),
            5 => forged.effect_sources[1].source_kind = EffectSourceKind::Equipment,
            6 => forged.effect_sources[1].lifetime = EffectLifetime::Permanent,
            7 => forged.effect_sources[1].ui_category = Some(CapabilityCategory::Information),
            8 => forged.effect_sources[1].ui_category = None,
            9 => forged.effect_sources[1].effects.clear(),
            10 => {
                let EffectSpec::HazardResistance(effect) = &mut forged.effect_sources[1].effects[0]
                else {
                    panic!("thermal lineage must retain its typed heat-resistance definition");
                };
                effect.reduction_bps = 9_999;
            }
            11 => {
                let extra = forged.effect_sources[1].effects[0].clone();
                forged.effect_sources[1].effects.push(extra);
            }
            12 => forged.progression.bloodline = Some(LIGHT.into()),
            13 => forged.progression.bloodline = Some(LEGACY.into()),
            14 => forged.progression.bloodline = None,
            15 => {
                forged.progression.bloodline = Some(LEGACY.into());
                forged.effect_sources[1].source_id = LEGACY.into();
                forged.effect_sources[1].instance_id = format!("bloodline:{LEGACY}");
            }
            16 => {
                forged.effect_sources[1].lifetime = EffectLifetime::Timed {
                    remaining_ms: 1_000,
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(
            forged.validate().unwrap_err(),
            "E_SAVE_EFFECT_SOURCES_INVALID",
            "mutation {mutation}"
        );
        assert!(forged.resolve_rules().is_err(), "mutation {mutation}");
        assert!(
            save_v6::write_save(root.path(), &forged).is_err(),
            "mutation {mutation}"
        );
        assert_eq!(
            std::fs::read(save_v6::save_path(root.path())).unwrap(),
            original
        );
    }
    for tier in [0, 2, 100, 101] {
        let mut forged = saved.clone();
        forged.progression.bloodline_tier = tier;
        assert!(
            forged.validate().is_err(),
            "accepted unsupported tier {tier}"
        );
    }
    let mut invalid_proficiency = saved;
    invalid_proficiency.progression.bloodline_proficiency = 1_000_001;
    assert!(invalid_proficiency.validate().is_err());
}

#[test]
fn recognized_id_without_canonical_source_is_not_a_legacy_migration_or_inferred_grant() {
    let root = TestRoot::new();
    let mut forged = empty_save(&root);
    forged.progression.bloodline = Some(STRONG.into());
    forged.progression.bloodline_tier = 1;
    assert!(forged.effect_sources.is_empty());
    assert_eq!(
        forged.validate().unwrap_err(),
        "E_SAVE_EFFECT_SOURCES_INVALID"
    );
    let original = serde_json::to_vec_pretty(&forged).unwrap();
    std::fs::write(save_v6::save_path(root.path()), &original).unwrap();
    assert_eq!(
        save_v6::read_or_migrate(root.path()).unwrap_err(),
        "E_SAVE_EFFECT_SOURCES_INVALID"
    );
    let runtime = runtime(&root);
    let before = runtime.build_snapshot().unwrap();
    assert!(runtime.continue_saved().is_err());
    assert_eq!(runtime.build_snapshot().unwrap(), before);
    assert_eq!(
        std::fs::read(save_v6::save_path(root.path())).unwrap(),
        original
    );
}
