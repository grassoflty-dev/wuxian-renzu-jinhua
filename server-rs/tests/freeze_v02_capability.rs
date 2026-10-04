#![allow(dead_code)]

use wuxian_horror_ch1::capability_v1::{
    apply_command, apply_command_at_revision, choose_first_enhancement, project_capabilities,
    tick_regeneration, CapabilityCommand, CapabilityError, CapabilityState, CapabilityTickInput,
    RegenerationConfig, CAP_ACOUSTIC_MAPPING, CAP_AIR_STEP, CAP_ENEMY_VITALS, CAP_LOCAL_MAP,
    CAP_REAR_VIEW, CAP_REGENERATION,
};
use wuxian_horror_ch1::continuous_combat::CombatEvent;
use wuxian_horror_ch1::world_v3::{
    self, EnemyVitals, ExploredMap, KnownConnection, KnownObjective, RearViewAuthorization, Vec3,
    VitalTier, WorldEffect, WorldRevision,
};

fn config(rate: f32, cap: f32, damage_delay: u64, combat_delay: u64) -> RegenerationConfig {
    RegenerationConfig::new(damage_delay, combat_delay, rate, cap).unwrap()
}
fn state(config: RegenerationConfig) -> CapabilityState {
    CapabilityState::new("grey_hive", config).unwrap()
}
fn revision(tick: u64) -> WorldRevision {
    WorldRevision::new(1, tick, tick).unwrap()
}
fn assert_error<T>(result: Result<T, CapabilityError>, expected: CapabilityError) {
    match result {
        Err(actual) => assert_eq!(actual, expected),
        Ok(_) => panic!("expected capability error: {expected:?}"),
    }
}
fn tick<'a>(
    revision: WorldRevision,
    now_ms: u64,
    dt_s: f32,
    hp: u32,
    max_hp: u32,
    damaged: Option<u64>,
    combat: Option<u64>,
    events: &'a [CombatEvent],
) -> CapabilityTickInput<'a> {
    CapabilityTickInput {
        revision,
        now_ms,
        dt_s,
        current_hp: hp,
        max_hp,
        last_damaged_at_ms: damaged,
        last_combat_at_ms: combat,
        combat_events: events,
    }
}

#[test]
fn four_canonical_grants_selects_serde_and_rejections() {
    let mut s = state(config(2.0, 0.75, 0, 0));
    for id in [
        CAP_LOCAL_MAP,
        CAP_REAR_VIEW,
        CAP_ENEMY_VITALS,
        CAP_REGENERATION,
    ] {
        apply_command_at_revision(
            &mut s,
            CapabilityCommand::Grant {
                capability_id: id.into(),
            },
            revision(1),
        )
        .unwrap();
    }
    let before = s.clone();
    assert_error(
        apply_command(
            &mut s,
            CapabilityCommand::Grant {
                capability_id: CAP_LOCAL_MAP.into(),
            },
        ),
        CapabilityError::DuplicateGrant,
    );
    assert_eq!(s.grants, before.grants);
    assert_error(
        apply_command(
            &mut s,
            CapabilityCommand::Grant {
                capability_id: "debug.all".into(),
            },
        ),
        CapabilityError::UnknownCapability,
    );
    assert_error(
        apply_command(
            &mut s,
            CapabilityCommand::Select {
                capability_ids: vec![CAP_LOCAL_MAP.into(), CAP_LOCAL_MAP.into()],
            },
        ),
        CapabilityError::DuplicateSelection,
    );
    assert_error(
        apply_command(
            &mut s,
            CapabilityCommand::Select {
                capability_ids: vec!["body.noclip".into()],
            },
        ),
        CapabilityError::UnknownCapability,
    );
    assert_error(
        apply_command_at_revision(
            &mut s,
            CapabilityCommand::Select {
                capability_ids: vec![CAP_LOCAL_MAP.into()],
            },
            revision(0),
        ),
        CapabilityError::StaleRevision,
    );
    apply_command(
        &mut s,
        CapabilityCommand::Select {
            capability_ids: vec![CAP_LOCAL_MAP.into(), CAP_REGENERATION.into()],
        },
    )
    .unwrap();
    let encoded = serde_json::to_string(&s).unwrap();
    let decoded: CapabilityState = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.grants, s.grants);
    assert_eq!(decoded.selected, s.selected);
    assert_eq!(decoded.regeneration, s.regeneration);
    assert_eq!(decoded.first_enhancement_choice, None);
}

#[test]
fn first_grey_hive_enhancement_is_one_atomic_choice() {
    for chosen in [CAP_LOCAL_MAP, CAP_REAR_VIEW, CAP_REGENERATION] {
        let mut s = state(config(2.0, 0.75, 0, 0));
        s.world_id = "return_station".into();
        let effects = choose_first_enhancement(&mut s, chosen, revision(1)).unwrap();
        assert_eq!(s.first_enhancement_choice.as_deref(), Some(chosen));
        assert_eq!(s.grants.len(), 1);
        assert_eq!(s.selected, vec![chosen]);
        if chosen == CAP_REAR_VIEW {
            assert!(matches!(
                effects.as_slice(),
                [WorldEffect::SetRearViewAuthorization { .. }]
            ));
        } else {
            assert!(effects.is_empty());
        }
        let before = s.clone();
        assert_error(
            choose_first_enhancement(&mut s, CAP_LOCAL_MAP, revision(2)),
            CapabilityError::FirstEnhancementAlreadyChosen,
        );
        assert_eq!(s.grants, before.grants);
        assert_eq!(s.first_enhancement_choice, before.first_enhancement_choice);
    }
}

#[test]
fn map_projection_contains_only_authorized_known_metre_data() {
    let mut s = state(config(2.0, 0.75, 0, 0));
    let known_map = ExploredMap::new(
        "grey_hive",
        Vec3::new(2.25, 0.0, 7.5).unwrap(),
        vec![world_v3::ExploredRoom {
            room_id: "room-known".into(),
            outline_m: vec![
                Vec3::zero(),
                Vec3::new(1.0, 0.0, 0.0).unwrap(),
                Vec3::new(1.0, 0.0, 1.0).unwrap(),
            ],
        }],
        vec![KnownConnection {
            connection_id: "door-known".into(),
            from_room_id: "room-known".into(),
            to_room_id: "room-known".into(),
        }],
        vec![KnownObjective {
            objective_id: "objective-known".into(),
            position_m: Vec3::new(3.0, 0.0, 8.0).unwrap(),
        }],
    )
    .unwrap();
    let hidden_enemy = EnemyVitals::new("hidden-enemy", 10, 20).unwrap();
    let denied = project_capabilities(
        &s,
        known_map.clone(),
        vec![hidden_enemy.clone()],
        RearViewAuthorization::denied(),
    );
    assert!(denied.explored_map.rooms.is_empty());
    assert!(denied.explored_map.connections.is_empty());
    assert!(denied.explored_map.objectives.is_empty());
    assert!(denied.enemy_vitals.is_empty());
    apply_command(
        &mut s,
        CapabilityCommand::Grant {
            capability_id: CAP_LOCAL_MAP.into(),
        },
    )
    .unwrap();
    let granted = project_capabilities(&s, known_map, vec![], RearViewAuthorization::denied());
    assert_eq!(granted.explored_map.rooms[0].room_id, "room-known");
    assert_eq!(granted.explored_map.player_position_m.x_m, 2.25);
    assert!(granted
        .explored_map
        .rooms
        .iter()
        .all(|r| r.room_id != "hidden-room"));
}

#[test]
fn enemy_vitals_are_four_tiers_redacted_and_dead_filtered() {
    let mut s = state(config(2.0, 0.75, 0, 0));
    let vitals = vec![
        EnemyVitals::new("healthy", 100, 100).unwrap(),
        EnemyVitals::new("wounded", 61, 100).unwrap(),
        EnemyVitals::new("severely", 60, 100).unwrap(),
        EnemyVitals::new("critical", 25, 100).unwrap(),
        EnemyVitals::new("critical-low", 1, 100).unwrap(),
        EnemyVitals::new("dead", 0, 100).unwrap(),
    ];
    let hidden = project_capabilities(
        &s,
        s.explored_map.clone(),
        vitals.clone(),
        RearViewAuthorization::denied(),
    );
    assert!(hidden.enemy_vitals.is_empty());
    apply_command(
        &mut s,
        CapabilityCommand::Grant {
            capability_id: CAP_ENEMY_VITALS.into(),
        },
    )
    .unwrap();
    let shown = project_capabilities(
        &s,
        s.explored_map.clone(),
        vitals,
        RearViewAuthorization::denied(),
    );
    assert_eq!(shown.enemy_vitals.len(), 5);
    assert_eq!(shown.enemy_vitals[0].tier, VitalTier::Healthy);
    assert_eq!(shown.enemy_vitals[1].tier, VitalTier::Wounded);
    assert_eq!(shown.enemy_vitals[2].tier, VitalTier::SeverelyWounded);
    assert_eq!(shown.enemy_vitals[3].tier, VitalTier::Critical);
    assert_eq!(shown.enemy_vitals[4].tier, VitalTier::Critical);
    let json = serde_json::to_value(shown).unwrap();
    let text = json.to_string();
    assert!(!text.contains("currentHp"));
    assert!(!text.contains("maxHp"));
    assert!(!text.contains("dead"));
}

#[test]
fn rear_view_authorization_requires_rust_grant() {
    let mut s = state(config(2.0, 0.75, 0, 0));
    let forged = RearViewAuthorization::granted("browser-self-grant", revision(1)).unwrap();
    let denied = project_capabilities(&s, s.explored_map.clone(), vec![], forged.clone());
    assert!(!denied.rear_view.granted);
    let effects = apply_command_at_revision(
        &mut s,
        CapabilityCommand::Grant {
            capability_id: CAP_REAR_VIEW.into(),
        },
        revision(2),
    )
    .unwrap();
    assert!(
        matches!(effects.as_slice(), [WorldEffect::SetRearViewAuthorization { authorization } ] if authorization.grant_id.as_deref()==Some(CAP_REAR_VIEW))
    );
    let granted = project_capabilities(
        &s,
        s.explored_map.clone(),
        vec![],
        s.rear_view_authorization.clone(),
    );
    assert!(granted.rear_view.granted);
    let spoofed = project_capabilities(&s, s.explored_map.clone(), vec![], forged);
    assert!(!spoofed.rear_view.granted);
}

#[test]
fn regeneration_respects_delays_damage_reset_cap_and_fraction() {
    let mut s = state(config(2.0, 0.75, 1000, 2000));
    apply_command(
        &mut s,
        CapabilityCommand::Grant {
            capability_id: CAP_REGENERATION.into(),
        },
    )
    .unwrap();
    let empty = [];
    assert!(tick_regeneration(
        &mut s,
        tick(
            revision(1),
            2500,
            0.5,
            50,
            100,
            Some(2000),
            Some(1000),
            &empty
        )
    )
    .unwrap()
    .effects
    .is_empty());
    assert!(tick_regeneration(
        &mut s,
        tick(
            revision(2),
            3000,
            0.5,
            50,
            100,
            Some(2000),
            Some(1500),
            &empty
        )
    )
    .unwrap()
    .effects
    .is_empty());
    let first = tick_regeneration(
        &mut s,
        tick(
            revision(3),
            3500,
            0.25,
            50,
            100,
            Some(2000),
            Some(1500),
            &empty,
        ),
    )
    .unwrap();
    assert!(first.effects.is_empty());
    let second = tick_regeneration(
        &mut s,
        tick(
            revision(4),
            3750,
            0.25,
            50,
            100,
            Some(2000),
            Some(1500),
            &empty,
        ),
    )
    .unwrap();
    assert!(matches!(
        second.effects.as_slice(),
        [WorldEffect::HealPlayer { amount: 1 }]
    ));
    let damage = [CombatEvent::PlayerDamaged {
        source_id: "sentinel".into(),
        damage: 1,
        contact: None,
    }];
    assert!(tick_regeneration(
        &mut s,
        tick(
            revision(5),
            3800,
            0.5,
            51,
            100,
            Some(3800),
            Some(3800),
            &damage
        )
    )
    .unwrap()
    .effects
    .is_empty());
    let capped = tick_regeneration(
        &mut s,
        tick(
            revision(6),
            6000,
            3.0,
            70,
            100,
            Some(3800),
            Some(3800),
            &empty,
        ),
    )
    .unwrap();
    assert!(matches!(
        capped.effects.as_slice(),
        [WorldEffect::HealPlayer { amount: 5 }]
    ));
    let at_cap = tick_regeneration(
        &mut s,
        tick(
            revision(7),
            6500,
            2.0,
            75,
            100,
            Some(3800),
            Some(3800),
            &empty,
        ),
    )
    .unwrap();
    assert!(at_cap.effects.is_empty());
}

#[test]
fn regeneration_rejects_duplicate_revision_and_invalid_config() {
    let mut s = state(config(1.0, 0.8, 0, 0));
    apply_command(
        &mut s,
        CapabilityCommand::Grant {
            capability_id: CAP_REGENERATION.into(),
        },
    )
    .unwrap();
    let empty = [];
    tick_regeneration(
        &mut s,
        tick(revision(1), 1000, 1.0, 10, 100, None, None, &empty),
    )
    .unwrap();
    assert_error(
        tick_regeneration(
            &mut s,
            tick(revision(1), 1100, 1.0, 10, 100, None, None, &empty),
        ),
        CapabilityError::StaleRevision,
    );
    assert!(RegenerationConfig::new(0, 0, f32::NAN, 0.5).is_err());
    assert!(RegenerationConfig::new(0, 0, 1.0, 0.0).is_err());
    assert!(RegenerationConfig::new(0, 0, 1.0, 1.01).is_err());
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigFile {
    schema_version: u32,
    canonical_capability_ids: Vec<String>,
    regeneration: RegenerationConfig,
}

#[test]
fn data_config_has_schema_and_canonical_ids() {
    let config: ConfigFile =
        serde_json::from_str(include_str!("../data/capability_v1.json")).unwrap();
    assert_eq!(config.schema_version, 1);
    assert_eq!(
        config.canonical_capability_ids,
        [
            CAP_LOCAL_MAP,
            CAP_REAR_VIEW,
            CAP_ENEMY_VITALS,
            CAP_REGENERATION,
            CAP_ACOUSTIC_MAPPING,
            CAP_AIR_STEP,
        ]
    );
    config.regeneration.validate().unwrap();
}
