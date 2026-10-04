use wuxian_horror_ch1::continuous_combat::{CombatEvent, CombatIntent};
use wuxian_horror_ch1::continuous_input::InputSample;
use wuxian_horror_ch1::continuous_kcc::{Aabb, StaticKccWorld};
use wuxian_horror_ch1::fixed_step;
use wuxian_horror_ch1::sentinel_ai::{Sentinel, SentinelState};
use wuxian_horror_ch1::world_v3::{step_world, CwStepInput, Vec3, WorldStateV3};

fn arena() -> StaticKccWorld {
    StaticKccWorld::new(
        Aabb::new(-20.0, 20.0, -20.0, 20.0).unwrap(),
        vec![Aabb::new(1.0, 2.0, -2.0, 2.0).unwrap()],
    )
}
fn sample(seq: u64, x: f32, z: f32) -> InputSample {
    InputSample::new(7, seq, seq * 16, x, z).unwrap()
}

#[test]
fn diagonal_speed_is_normalized_and_motion_is_continuous() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 1.0, 1.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    assert!(out.collisions.is_empty());
    let speed = (s.player.velocity_mps.x_m.powi(2) + s.player.velocity_mps.z_m.powi(2)).sqrt();
    assert!((speed - 4.0).abs() < 1e-5);
    assert!(s.player.position_m.x_m > 0.0 && s.player.position_m.x_m < 0.21);
}

#[test]
fn wall_blocks_without_grid_or_snap() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::new(0.6, 0.0, 0.0).unwrap()).unwrap();
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 1.0, 0.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    assert_eq!(s.player.position_m.x_m, 0.6);
    assert_eq!(out.collisions.len(), 1);
}

#[test]
fn airborne_jump_is_rejected() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 0.0, 0.0),
            dt_s: 0.016,
            combat: &[CombatIntent::Jump { request_id: 1 }],
        },
    )
    .unwrap();
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(2, 0.0, 0.0),
            dt_s: 0.016,
            combat: &[CombatIntent::Jump { request_id: 2 }],
        },
    )
    .unwrap();
    assert!(out
        .combat
        .iter()
        .any(|e| matches!(e, CombatEvent::IntentRejected { request_id: 2, .. })));
}

#[test]
fn dash_has_duration_and_cooldown() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, -1.0, 0.0),
            dt_s: 0.05,
            combat: &[CombatIntent::Dash { request_id: 10 }],
        },
    )
    .unwrap();
    assert_eq!(s.player.velocity_mps.x_m, -10.0);
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(2, -1.0, 0.0),
            dt_s: 0.05,
            combat: &[CombatIntent::Dash { request_id: 11 }],
        },
    )
    .unwrap();
    assert!(out
        .combat
        .iter()
        .any(|e| matches!(e, CombatEvent::IntentRejected { request_id: 11, .. })));
    for i in 3..18 {
        step_world(
            &mut s,
            &arena(),
            CwStepInput {
                sample: &sample(i, -1.0, 0.0),
                dt_s: 0.05,
                combat: &[],
            },
        )
        .unwrap();
    }
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(18, -1.0, 0.0),
            dt_s: 0.05,
            combat: &[CombatIntent::Dash { request_id: 12 }],
        },
    )
    .unwrap();
    assert!(!out
        .combat
        .iter()
        .any(|e| matches!(e, CombatEvent::IntentRejected { .. })));
}

#[test]
fn spatial_attack_hits_misses_and_deduplicates() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    s.sentinels
        .push(Sentinel::new("near", Vec3::new(1.0, 0.0, 0.0).unwrap(), 50));
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 0.0, 0.0),
            dt_s: 0.016,
            combat: &[
                CombatIntent::Attack { request_id: 20 },
                CombatIntent::Attack { request_id: 20 },
            ],
        },
    )
    .unwrap();
    assert_eq!(s.sentinels[0].hp, 25);
    assert_eq!(
        out.combat
            .iter()
            .filter(|e| matches!(e, CombatEvent::AttackHit { .. }))
            .count(),
        1
    );
    s.sentinels[0].position_m = Vec3::new(10.0, 0.0, 0.0).unwrap();
    let out = step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(2, 0.0, 0.0),
            dt_s: 0.016,
            combat: &[CombatIntent::Attack { request_id: 21 }],
        },
    )
    .unwrap();
    assert!(out
        .combat
        .iter()
        .any(|e| matches!(e, CombatEvent::AttackMiss { .. })));
}

#[test]
fn sentinel_moves_transitions_damages_and_death_stops_attacks() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    s.sentinels.push(Sentinel::new(
        "sentinel-1",
        Vec3::new(4.0, 0.0, 0.0).unwrap(),
        100,
    ));
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 0.0, 0.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    assert!(s.sentinels[0].position_m.x_m < 4.0);
    s.sentinels[0].position_m = Vec3::new(1.0, 0.0, 0.0).unwrap();
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(2, 0.0, 0.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    assert_eq!(s.sentinels[0].state, SentinelState::Attack);
    let hp = s.player_hp;
    for i in 3..9 {
        step_world(
            &mut s,
            &arena(),
            CwStepInput {
                sample: &sample(i, 0.0, 0.0),
                dt_s: 0.05,
                combat: &[],
            },
        )
        .unwrap();
    }
    assert!(s.player_hp < hp);
    assert!(matches!(
        s.sentinels[0].state,
        SentinelState::Recover | SentinelState::HeavyAttack
    ));
    s.sentinels[0].take_damage(1);
    assert_eq!(s.sentinels[0].state, SentinelState::Hit);
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(9, 0.0, 0.0),
            dt_s: 0.1,
            combat: &[],
        },
    )
    .unwrap();
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(10, 0.0, 0.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    assert_eq!(s.sentinels[0].state, SentinelState::Chase);
    s.sentinels[0].take_damage(999);
    assert_eq!(s.sentinels[0].state, SentinelState::Death);
    let hp = s.player_hp;
    for i in 11..20 {
        step_world(
            &mut s,
            &arena(),
            CwStepInput {
                sample: &sample(i, 0.0, 0.0),
                dt_s: 0.05,
                combat: &[],
            },
        )
        .unwrap();
    }
    assert_eq!(s.player_hp, hp);
}

#[test]
fn sentinel_heavy_attack_has_real_damage_timing() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    let mut enemy = Sentinel::new("heavy", Vec3::new(1.0, 0.0, 0.0).unwrap(), 100);
    enemy.state = SentinelState::Attack;
    enemy.attack_serial = 2;
    s.sentinels.push(enemy);
    let hp = s.player_hp;
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 0.0, 0.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    assert_eq!(s.sentinels[0].state, SentinelState::HeavyAttack);
    assert_eq!(s.player_hp, hp - 8);
    for i in 2..10 {
        step_world(
            &mut s,
            &arena(),
            CwStepInput {
                sample: &sample(i, 0.0, 0.0),
                dt_s: 0.05,
                combat: &[],
            },
        )
        .unwrap();
    }
    assert_eq!(s.sentinels[0].state, SentinelState::Recover);
    assert_eq!(s.player_hp, hp - 24);
}

#[test]
fn fixed_step_is_bounded() {
    let config = fixed_step::FixedStepConfig::new(60, 4).unwrap();
    let mut clock = fixed_step::FixedStepClock::new(config);
    assert_eq!(clock.push_frame(1.0).unwrap(), 4);
}

#[test]
fn stale_input_fails_without_partial_mutation() {
    let mut s = WorldStateV3::new("grey_hive", 7, Vec3::zero()).unwrap();
    step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, 1.0, 0.0),
            dt_s: 0.05,
            combat: &[],
        },
    )
    .unwrap();
    let position = s.player.position_m;
    assert!(step_world(
        &mut s,
        &arena(),
        CwStepInput {
            sample: &sample(1, -1.0, 0.0),
            dt_s: 0.05,
            combat: &[]
        },
    )
    .is_err());
    assert_eq!(s.player.position_m, position);
}
