use super::*;
use crate::player_rules::{
    BoundaryTag, KnowledgeLevel, MovementContext, MovementMode, ObstacleTag, FULL_DAMAGE_BPS,
};

fn source(
    kind: EffectSourceKind,
    id: &str,
    instance: &str,
    effects: Vec<EffectSpec>,
) -> EffectSource {
    EffectSource {
        source_kind: kind,
        source_id: id.into(),
        instance_id: instance.into(),
        lifetime: EffectLifetime::Equipped,
        ui_category: None,
        effects,
    }
}

fn rear_view() -> EffectSpec {
    EffectSpec::Perception(PerceptionEffect {
        tag: PerceptionTag::RearView,
        stack: StackRule::Any,
    })
}

#[test]
fn rear_view_remains_enabled_until_both_independent_sources_are_removed() {
    let a = source(
        EffectSourceKind::InnateCapability,
        "perception.rear_view_i",
        "grant-a",
        vec![rear_view()],
    );
    let b = source(
        EffectSourceKind::Equipment,
        "all_direction_mirror",
        "mirror-b",
        vec![rear_view()],
    );
    assert!(
        EffectResolver::resolve(&[a.clone(), b.clone()])
            .unwrap()
            .perception
            .rear_view
    );
    assert!(EffectResolver::resolve(&[b]).unwrap().perception.rear_view);
    assert!(!EffectResolver::resolve(&[]).unwrap().perception.rear_view);
}

#[test]
fn full_map_grants_only_explicit_topology_terrain_and_connections() {
    let source = source(
        EffectSourceKind::Item,
        "test_full_map",
        "test-map-1",
        [
            MapKnowledgeTag::Topology,
            MapKnowledgeTag::Terrain,
            MapKnowledgeTag::Connections,
        ]
        .into_iter()
        .map(|tag| {
            EffectSpec::MapKnowledge(MapKnowledgeEffect {
                tag,
                stack: StackRule::Any,
            })
        })
        .collect(),
    );
    let rules = EffectResolver::resolve(&[source]).unwrap();
    assert_eq!(rules.map.granted.len(), 3);
    assert_eq!(rules.map.topology, KnowledgeLevel::Full);
    assert_eq!(rules.map.terrain, KnowledgeLevel::Full);
    assert_eq!(rules.map.connections, KnowledgeLevel::Full);
    assert_eq!(rules.map.objectives, KnowledgeLevel::None);
    assert_eq!(rules.map.enemies, KnowledgeLevel::None);
    assert_eq!(rules.map.hazards, KnowledgeLevel::None);
    assert_eq!(rules.map.loot, KnowledgeLevel::None);
    assert_eq!(rules.map.npcs, KnowledgeLevel::None);
    assert_eq!(rules.map.secrets, KnowledgeLevel::None);
}

#[test]
fn hover_ignores_only_selected_terrain_penalties_and_never_route_boundaries() {
    let hover = source(
        EffectSourceKind::Equipment,
        "test_hover",
        "hover-1",
        [TerrainTag::Water, TerrainTag::Mud]
            .into_iter()
            .map(|tag| {
                EffectSpec::TerrainPenaltyIgnore(TerrainPenaltyEffect {
                    tag,
                    stack: StackRule::Any,
                })
            })
            .collect(),
    );
    let rules = EffectResolver::resolve(&[hover]).unwrap();
    assert!(rules
        .terrain
        .ignored_move_penalties
        .contains(&TerrainTag::Water));
    assert!(rules
        .terrain
        .ignored_move_penalties
        .contains(&TerrainTag::Mud));
    assert!(!rules
        .terrain
        .ignored_move_penalties
        .contains(&TerrainTag::Concrete));
    assert_eq!(
        rules.boundaries.blocked,
        BTreeSet::from([BoundaryTag::World, BoundaryTag::Quest])
    );
}

#[test]
fn heat_resistance_multiplies_remaining_damage_and_immunity_is_explicit() {
    let resistance = |instance| {
        source(
            EffectSourceKind::TemporaryBuff,
            "test_heat_guard",
            instance,
            vec![EffectSpec::HazardResistance(HazardResistanceEffect {
                tag: HazardTag::Heat,
                reduction_bps: 5_000,
                stack: StackRule::MultiplyRemaining,
            })],
        )
    };
    let a = resistance("heat-a");
    let b = resistance("heat-b");
    let partial = EffectResolver::resolve(&[a.clone(), b.clone()]).unwrap();
    assert_eq!(partial.hazards.remaining_damage_bps(HazardTag::Heat), 2_500);
    assert_eq!(
        partial.hazards.remaining_damage_bps(HazardTag::Water),
        FULL_DAMAGE_BPS
    );
    assert!(!partial.hazards.immune.contains(&HazardTag::Heat));
    assert_eq!(
        EffectResolver::resolve(&[])
            .unwrap()
            .hazards
            .remaining_damage_bps(HazardTag::Heat),
        FULL_DAMAGE_BPS
    );

    let immunity = source(
        EffectSourceKind::Item,
        "test_heat_immunity",
        "immune-1",
        vec![EffectSpec::HazardImmunity(HazardImmunityEffect {
            tag: HazardTag::Heat,
            stack: StackRule::Any,
        })],
    );
    let immune = EffectResolver::resolve(&[a, b, immunity]).unwrap();
    assert_eq!(immune.hazards.remaining_damage_bps(HazardTag::Heat), 0);
    assert!(immune.hazards.immune.contains(&HazardTag::Heat));
}

#[test]
fn source_and_effect_order_do_not_change_serialized_rules() {
    let mut map = source(
        EffectSourceKind::Item,
        "test_map",
        "map-1",
        vec![
            EffectSpec::MapKnowledge(MapKnowledgeEffect {
                tag: MapKnowledgeTag::Connections,
                stack: StackRule::Any,
            }),
            EffectSpec::MapKnowledge(MapKnowledgeEffect {
                tag: MapKnowledgeTag::Topology,
                stack: StackRule::Any,
            }),
        ],
    );
    map.ui_category = Some(CapabilityCategory::Information);
    let rear = source(
        EffectSourceKind::Bloodline,
        "test_rear",
        "rear-1",
        vec![rear_view()],
    );
    let heat = source(
        EffectSourceKind::TemporaryBuff,
        "test_heat",
        "heat-1",
        vec![EffectSpec::HazardResistance(HazardResistanceEffect {
            tag: HazardTag::Heat,
            reduction_bps: 5_000,
            stack: StackRule::MultiplyRemaining,
        })],
    );
    let original = serde_json::to_vec(
        &EffectResolver::resolve(&[map.clone(), rear.clone(), heat.clone()]).unwrap(),
    )
    .unwrap();
    map.effects.reverse();
    map.ui_category = Some(CapabilityCategory::Perception);
    let shuffled =
        serde_json::to_vec(&EffectResolver::resolve(&[heat, rear, map]).unwrap()).unwrap();
    assert_eq!(original, shuffled);
}

#[test]
fn invalid_inputs_fail_closed_and_unknown_tags_cannot_deserialize() {
    let valid = source(
        EffectSourceKind::Item,
        "test_item",
        "item-1",
        vec![rear_view()],
    );
    let mut invalid_cases = Vec::new();
    let mut empty = valid.clone();
    empty.effects.clear();
    invalid_cases.push((empty, EffectError::EmptyEffects));
    let mut bad_id = valid.clone();
    bad_id.source_id = "NOT-A-TAG".into();
    invalid_cases.push((bad_id, EffectError::InvalidId));
    let mut expired = valid.clone();
    expired.lifetime = EffectLifetime::Timed { remaining_ms: 0 };
    invalid_cases.push((expired, EffectError::InvalidLifetime));
    let mut wrong_stack = valid.clone();
    wrong_stack.effects = vec![EffectSpec::Perception(PerceptionEffect {
        tag: PerceptionTag::RearView,
        stack: StackRule::Add,
    })];
    invalid_cases.push((wrong_stack, EffectError::InvalidStackRule));
    let mut full_resistance = valid.clone();
    full_resistance.effects = vec![EffectSpec::HazardResistance(HazardResistanceEffect {
        tag: HazardTag::Heat,
        reduction_bps: 10_000,
        stack: StackRule::MultiplyRemaining,
    })];
    invalid_cases.push((full_resistance, EffectError::InvalidResistance));
    for (candidate, expected) in invalid_cases {
        assert_eq!(EffectResolver::resolve(&[candidate]), Err(expected));
    }
    assert_eq!(
        EffectResolver::resolve(&[valid.clone(), valid]),
        Err(EffectError::DuplicateSource)
    );

    for json in [
        r#"{"terrain_penalty_ignore":{"tag":"unknown","stack":"any"}}"#,
        r#"{"map_knowledge":{"tag":"enemy_positions","stack":"any"}}"#,
        r#"{"hazard_resistance":{"tag":"unknown","reduction_bps":5000,"stack":"multiply_remaining"}}"#,
        r#"{"world_boundary_bypass":{"stack":"any"}}"#,
        r#"{"perception":{"tag":"rear_view","stack":"any","extra":true}}"#,
    ] {
        assert!(
            serde_json::from_str::<EffectSpec>(json).is_err(),
            "accepted {json}"
        );
    }
}

#[test]
fn map_channels_resolve_to_independent_full_levels_and_radius_is_bounded() {
    let knowledge = source(
        EffectSourceKind::Item,
        "test_map_channels",
        "map-channels-1",
        [MapKnowledgeTag::Topology, MapKnowledgeTag::Objectives]
            .into_iter()
            .map(|tag| {
                EffectSpec::MapKnowledge(MapKnowledgeEffect {
                    tag,
                    stack: StackRule::Any,
                })
            })
            .collect(),
    );
    let radius = source(
        EffectSourceKind::Equipment,
        "test_map_scan",
        "map-scan-1",
        vec![EffectSpec::MapRevealRadius(MapRevealRadiusEffect {
            radius_mm: 25_000,
            stack: StackRule::Max,
        })],
    );
    let enemies = source(
        EffectSourceKind::InnateCapability,
        "test_enemy_detection",
        "enemy-detection-1",
        vec![EffectSpec::MapKnowledgeLevel(MapKnowledgeLevelEffect {
            tag: MapKnowledgeTag::Enemies,
            level: KnowledgeLevel::DetectedOnly,
            stack: StackRule::Max,
        })],
    );
    let rules = EffectResolver::resolve(&[knowledge, radius, enemies]).unwrap();
    assert_eq!(rules.map.topology, KnowledgeLevel::Full);
    assert_eq!(rules.map.objectives, KnowledgeLevel::Full);
    assert_eq!(rules.map.enemies, KnowledgeLevel::DetectedOnly);
    assert_eq!(rules.map.reveal_radius_mm, 25_000);

    for invalid in [0, crate::player_rules::MAX_REVEAL_RADIUS_MM + 1] {
        let candidate = source(
            EffectSourceKind::Item,
            "test_invalid_scan",
            "bad-scan-1",
            vec![EffectSpec::MapRevealRadius(MapRevealRadiusEffect {
                radius_mm: invalid,
                stack: StackRule::Max,
            })],
        );
        assert_eq!(
            EffectResolver::resolve(&[candidate]),
            Err(EffectError::InvalidRevealRadius)
        );
    }
}

#[test]
fn movement_modes_keep_boundaries_and_hard_walls_closed() {
    let hover = source(
        EffectSourceKind::Equipment,
        "test_water_hover",
        "water-hover-1",
        vec![EffectSpec::TerrainPenaltyIgnore(TerrainPenaltyEffect {
            tag: TerrainTag::Water,
            stack: StackRule::Any,
        })],
    );
    let rules = EffectResolver::resolve(&[hover]).unwrap();
    let water = rules.resolve_movement(
        MovementMode::Hover,
        MovementContext {
            terrain: Some(TerrainTag::Water),
            ..MovementContext::default()
        },
    );
    let conveyor = rules.resolve_movement(
        MovementMode::Ground,
        MovementContext {
            terrain: Some(TerrainTag::Conveyor),
            ..MovementContext::default()
        },
    );
    assert_eq!(water.terrain_penalty_bps, 0);
    assert_eq!(conveyor.terrain_penalty_bps, 1_000);
    assert!(
        !rules
            .resolve_movement(
                MovementMode::Flight,
                MovementContext {
                    obstacle: Some(ObstacleTag::Wall),
                    ..MovementContext::default()
                }
            )
            .allowed
    );
    assert!(
        !rules
            .resolve_movement(
                MovementMode::Flight,
                MovementContext {
                    boundary: Some(BoundaryTag::World),
                    ..MovementContext::default()
                }
            )
            .allowed
    );
    let mut rules_without_configured_gate = rules.clone();
    rules_without_configured_gate.boundaries.blocked.clear();
    assert!(
        !rules_without_configured_gate
            .resolve_movement(
                MovementMode::Ground,
                MovementContext {
                    boundary: Some(BoundaryTag::Quest),
                    ..MovementContext::default()
                }
            )
            .allowed
    );
    assert!(
        rules
            .resolve_movement(
                MovementMode::AirStep,
                MovementContext {
                    obstacle: Some(ObstacleTag::TraversalMarker),
                    ..MovementContext::default()
                }
            )
            .allowed
    );
}

#[test]
fn old_map_rules_json_deserializes_with_new_channels_disabled() {
    let old = r#"{
        "granted":["topology"]
    }"#;
    let map: crate::player_rules::MapKnowledgeRules = serde_json::from_str(old).unwrap();
    assert!(map.granted.contains(&MapKnowledgeTag::Topology));
    assert_eq!(map.topology, KnowledgeLevel::None);
    assert_eq!(
        map.level_for_tag(MapKnowledgeTag::Topology),
        KnowledgeLevel::Full
    );
    assert_eq!(map.secrets, KnowledgeLevel::None);
    assert_eq!(map.reveal_radius_mm, 0);
}

#[test]
fn terrain_movement_modifiers_use_max_explicit_value_and_restore_default_on_removal() {
    let modifier = |instance, multiplier_bps| {
        source(
            EffectSourceKind::Equipment,
            "test_water_movement",
            instance,
            vec![EffectSpec::TerrainMovementModifier(
                TerrainMovementModifierEffect {
                    tag: TerrainTag::WaterShallow,
                    multiplier_bps,
                    stack: StackRule::Max,
                },
            )],
        )
    };
    let a = modifier("water-a", 8_000);
    let b = modifier("water-b", 9_000);
    let both = EffectResolver::resolve(&[a.clone(), b.clone()]).unwrap();
    let reversed = EffectResolver::resolve(&[b.clone(), a.clone()]).unwrap();
    assert_eq!(
        serde_json::to_vec(&both).unwrap(),
        serde_json::to_vec(&reversed).unwrap()
    );
    assert_eq!(
        both.terrain
            .movement_multiplier_bps(TerrainTag::WaterShallow),
        9_000
    );
    assert_eq!(
        EffectResolver::resolve(&[a])
            .unwrap()
            .terrain
            .movement_multiplier_bps(TerrainTag::WaterShallow),
        8_000
    );
    assert_eq!(
        EffectResolver::resolve(&[b])
            .unwrap()
            .terrain
            .movement_multiplier_bps(TerrainTag::WaterShallow),
        9_000
    );
    let empty = EffectResolver::resolve(&[]).unwrap();
    assert_eq!(
        empty
            .terrain
            .movement_multiplier_bps(TerrainTag::WaterShallow),
        6_000
    );
    assert!(empty.terrain.movement_multiplier_bps.is_empty());
    // Values are absolute fractions of normal movement, so a slow explicit modifier
    // must not be accidentally raised to the default when the first source is applied.
    for multiplier in [1, 5_000, FULL_DAMAGE_BPS] {
        let rules = EffectResolver::resolve(&[modifier("water-edge", multiplier)]).unwrap();
        assert_eq!(
            rules
                .terrain
                .movement_multiplier_bps(TerrainTag::WaterShallow),
            multiplier
        );
    }
}

#[test]
fn terrain_movement_modifier_validation_fails_closed() {
    for (multiplier_bps, stack, error) in [
        (0, StackRule::Max, EffectError::InvalidMovementMultiplier),
        (
            10_001,
            StackRule::Max,
            EffectError::InvalidMovementMultiplier,
        ),
        (
            u16::MAX,
            StackRule::Max,
            EffectError::InvalidMovementMultiplier,
        ),
        (8_000, StackRule::Add, EffectError::InvalidStackRule),
        (8_000, StackRule::Multiply, EffectError::InvalidStackRule),
        (8_000, StackRule::Any, EffectError::InvalidStackRule),
    ] {
        let candidate = source(
            EffectSourceKind::Item,
            "test_water",
            "water-invalid",
            vec![EffectSpec::TerrainMovementModifier(
                TerrainMovementModifierEffect {
                    tag: TerrainTag::WaterShallow,
                    multiplier_bps,
                    stack,
                },
            )],
        );
        assert_eq!(EffectResolver::resolve(&[candidate]), Err(error));
    }
    for json in [
        r#"{"terrain_movement_modifier":{"tag":"unknown","multiplier_bps":8000,"stack":"max"}}"#,
        r#"{"terrain_movement_modifier":{"tag":"terrain.water_shallow","multiplier_bps":-1,"stack":"max"}}"#,
        r#"{"terrain_movement_modifier":{"tag":"terrain.water_shallow","multiplier_bps":8000.5,"stack":"max"}}"#,
        r#"{"terrain_movement_modifier":{"tag":"terrain.water_shallow","multiplier_bps":8000,"stack":"max","passable":true}}"#,
    ] {
        assert!(
            serde_json::from_str::<EffectSpec>(json).is_err(),
            "accepted {json}"
        );
    }
}

#[test]
fn typed_capability_permissions_union_across_sources_without_creating_other_rules() {
    let first = EffectSource {
        source_kind: EffectSourceKind::InnateCapability,
        source_id: "trusted_capability".into(),
        instance_id: "capability:trusted".into(),
        lifetime: EffectLifetime::Permanent,
        ui_category: Some(CapabilityCategory::Mobility),
        effects: vec![EffectSpec::GrantCapability(CapabilityPermissionEffect {
            tag: CapabilityPermission::AuthoredAirStep,
            stack: StackRule::Any,
        })],
    };
    let mut second = first.clone();
    second.source_kind = EffectSourceKind::Equipment;
    second.instance_id = "equipment:trusted".into();
    let rules = EffectResolver::resolve(&[first.clone(), second.clone()]).unwrap();
    assert_eq!(
        rules.capability_permissions,
        BTreeSet::from([CapabilityPermission::AuthoredAirStep])
    );
    assert_eq!(EffectResolver::resolve(&[first]).unwrap(), rules);
    assert_eq!(EffectResolver::resolve(&[second]).unwrap(), rules);
    assert!(EffectResolver::resolve(&[])
        .unwrap()
        .capability_permissions
        .is_empty());
    assert_eq!(rules.terrain, EffectivePlayerRules::default().terrain);
    assert_eq!(rules.boundaries, EffectivePlayerRules::default().boundaries);
}
