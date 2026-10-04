//! Bounded shared-controller/world-owner fixtures, not an earned campaign route.
//! The Brute profile is intentionally not admitted to native scene content yet.
use super::*;
use crate::{continuous_combat::{CombatEvent, CombatIntent}, continuous_input::InputSample,
    continuous_kcc::{Aabb, StaticKccWorld}, world_v3::{step_world, CwStepInput, WorldStateV3}};
const BRUTE: &str = "enemy.grey_hive.brute";
const DT: f32 = 1.0 / 60.0;
fn p(x: f32, z: f32) -> Vec3 { Vec3::new(x, 0.0, z).unwrap() }
fn arena() -> StaticKccWorld { StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![]) }
fn brute() -> ActorRuntime { ActorRuntime::spawn("brute-fixture", BRUTE, p(5.0, 8.0)).unwrap() }
fn reach(actor: &mut ActorRuntime, target: Vec3, state: ActorAiState) {
    for _ in 0..250 {
        if actor.state == state { return; }
        actor.tick(target, &arena(), DT, false); assert!(actor.validate());
    }
    panic!("did not reach {state:?}");
}
fn world(actor_position: Vec3) -> WorldStateV3 {
    let mut world = WorldStateV3::new("grey_hive", 8, p(5.0, 8.0)).unwrap();
    world.combat_state.qer_v1_active = true;
    world.generic_actors.push(ActorRuntime::spawn("brute-fixture", BRUTE, actor_position).unwrap()); world
}
fn owner_step(world: &mut WorldStateV3, seq: u64, intents: &[CombatIntent]) -> crate::world_v3::CwStepOutput {
    let sample = InputSample::new(8, seq, seq * 17, 0.0, 0.0).unwrap().with_aim(1.0, 0.0).unwrap();
    step_world(world, &arena(), CwStepInput { sample: &sample, dt_s: DT, combat: intents }).unwrap()
}
fn impact_count(events: &[ActorRuntimeEvent]) -> usize { events.iter().filter(|e| matches!(e, ActorRuntimeEvent::AttackImpact { .. })).count() }

#[test]
fn brute_is_a_heavy_shared_profile_with_distinct_charge_and_slam() {
    let profile = actor_profile(BRUTE).unwrap(); let config = profile.ordinary.as_ref().unwrap();
    assert_eq!(profile.render_type, BRUTE);
    assert!(config.charge.is_some() && config.leap.is_none() && config.projectile.is_none());
    assert!(profile.max_hp > actor_profile("grey_hive.infected_security").unwrap().max_hp);
    assert!(profile.speed_mps < actor_profile("grey_hive.infected_security").unwrap().speed_mps);
    assert!(config.body_radius_m > actor_profile("enemy.clockworks.furnace_hound").unwrap().ordinary.as_ref().unwrap().body_radius_m);
    let mut charge = brute(); reach(&mut charge, p(9.0, 8.0), ActorAiState::Windup);
    assert_eq!(charge.current_attack, Some(ActorAttackKind::Charge));
    let mut slam = brute(); reach(&mut slam, p(6.0, 8.0), ActorAiState::Windup);
    assert_eq!(slam.current_attack, Some(ActorAttackKind::Slam));
    assert_ne!(charge.state_remaining_ms, slam.state_remaining_ms);
}

#[test]
fn charge_commits_grounded_aim_sweeps_body_and_hits_once() {
    let target = p(9.0, 8.0); let mut actor = brute(); reach(&mut actor, target, ActorAiState::Active);
    let committed = actor.clone(); let direction = actor.ordinary.as_ref().unwrap().active.as_ref().unwrap().direction;
    assert_eq!(direction, [1.0, 0.0]);
    let mut hits = 0; let mut impacts = 0;
    while actor.state == ActorAiState::Active {
        let out = actor.tick(target, &arena(), DT, false);
        if let Some(damage) = out.damage_to_player { hits += 1; assert_eq!(damage, 22); assert_eq!(out.damage_tag, None); }
        impacts += impact_count(&out.events); assert_eq!(actor.position_m.y_m, 0.0); assert!(actor.validate());
    }
    assert_eq!((hits, impacts), (1, 1)); assert!(actor.position_m.x_m > 9.0);
    let mut dodged = committed.clone();
    while dodged.state == ActorAiState::Active {
        assert!(dodged.tick(p(9.0, 11.0), &arena(), DT, false).damage_to_player.is_none());
    }
    assert_eq!(dodged.position_m.z_m, 8.0, "committed charge cannot retarget a strafing player");
    let covered = StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![Aabb::new(7.03, 7.04, 7.0, 9.0).unwrap()]);
    let mut blocked = committed; let mut count = 0;
    while blocked.state == ActorAiState::Active {
        let out = blocked.tick(target, &covered, 0.25, false); assert!(out.damage_to_player.is_none()); count += impact_count(&out.events);
    }
    assert_eq!(count, 1); assert!(blocked.position_m.x_m < 7.03 - 0.55);
}

#[test]
fn slam_is_radial_including_rear_and_sides_but_respects_radius_and_occlusion() {
    for (target, expected) in [(p(4.0, 8.0), true), (p(5.0, 9.5), true), (p(7.1, 8.0), false)] {
        let mut actor = brute(); reach(&mut actor, p(6.0, 8.0), ActorAiState::Windup);
        let mut observed = false;
        while actor.state == ActorAiState::Windup {
            let out = actor.tick(target, &arena(), DT, false);
            if impact_count(&out.events) > 0 {
                assert_eq!(out.damage_to_player.is_some(), expected);
                assert!(out.events.iter().any(|event| matches!(event, ActorRuntimeEvent::AttackImpact {
                    kind: ActorAttackKind::Slam, radius_m, hit_player, ..
                } if *radius_m == 2.0 && *hit_player == expected))); observed = true;
            }
        }
        assert!(observed); assert_eq!(actor.position_m, p(5.0, 8.0));
    }
    let mut actor = brute(); reach(&mut actor, p(6.0, 8.0), ActorAiState::Windup);
    let wall = StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![Aabb::new(4.0, 6.0, 8.7, 8.71).unwrap()]);
    while actor.state == ActorAiState::Windup { assert!(actor.tick(p(5.0, 9.5), &wall, DT, false).damage_to_player.is_none()); }
}

#[test]
fn real_primary_pierce_and_guard_have_distinct_counterplay_without_counter_damage() {
    for kind in ["primary", "pierce"] {
        let mut world = world(p(6.4, 8.0)); let mut stagger = 0;
        for seq in 1..=25 {
            let intents = if seq == 1 { vec![if kind == "primary" { CombatIntent::ActionAttack { request_id: seq } } else { CombatIntent::Pierce { request_id: seq } }] } else { vec![] };
            let out = owner_step(&mut world, seq, &intents);
            assert!(!out.combat.iter().any(|e| matches!(e, CombatEvent::IntentRejected { .. })));
            stagger += out.actor_runtime.iter().filter(|e| matches!(e, ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Stagger, .. })).count();
        }
        assert_eq!(world.generic_actors[0].hp, 96 - if kind == "primary" { 16 } else { 32 });
        assert_eq!(stagger, usize::from(kind == "pierce"));
        assert_eq!(world.player_hp, 100);
    }
    let mut guarded = world(p(6.4, 8.0)); let mut sent = false; let mut blocks = 0; let mut staggers = 0;
    for seq in 1..=80 {
        let actor = &guarded.generic_actors[0];
        let guard_now = !sent && actor.state == ActorAiState::Windup && actor.state_remaining_ms <= 220;
        let intents = if guard_now { sent = true; vec![CombatIntent::GuardStart { request_id: seq }] } else { vec![] };
        let out = owner_step(&mut guarded, seq, &intents);
        assert!(!out.combat.iter().any(|e| matches!(e, CombatEvent::IntentRejected { .. })));
        blocks += out.combat.iter().filter(|e| matches!(e, CombatEvent::GuardImpact { source_id, .. } if source_id == "brute-fixture")).count();
        staggers += out.actor_runtime.iter().filter(|e| matches!(e, ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Stagger, .. })).count();
    }
    assert_eq!((blocks, staggers), (1, 1)); assert_eq!(guarded.player_hp, 100); assert_eq!(guarded.generic_actors[0].hp, 96);
}

#[test]
fn public_pierce_defeat_emits_one_death_and_dead_actor_never_attacks_again() {
    let mut world = world(p(6.4, 8.0)); let mut deaths = 0;
    for seq in 1..=450 {
        let intents = if [1, 165, 330].contains(&seq) { vec![CombatIntent::Pierce { request_id: seq }] } else { vec![] };
        let out = owner_step(&mut world, seq, &intents);
        assert!(!out.combat.iter().any(|e| matches!(e, CombatEvent::IntentRejected { .. })));
        deaths += out.actor_runtime.iter().filter(|e| matches!(e, ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Death, .. })).count();
    }
    assert_eq!(deaths, 1); assert_eq!(world.generic_actors[0].hp, 0); assert!(world.player_hp > 0);
    let hp = world.player_hp;
    for seq in 451..=600 { assert!(owner_step(&mut world, seq, &[]).actor_runtime.is_empty()); }
    assert_eq!(world.player_hp, hp);
}

#[test]
fn saved_charge_slam_stagger_and_death_roundtrip_without_reset_or_duplicate_hit() {
    for phase in ["charge-windup", "charge-active", "slam", "stagger", "dead"] {
        let mut actor = brute();
        reach(&mut actor, if phase == "slam" { p(6.0, 8.0) } else { p(9.0, 8.0) },
            if phase == "charge-active" { ActorAiState::Active } else { ActorAiState::Windup });
        if phase == "charge-active" { actor.tick(p(9.0, 8.0), &arena(), DT, false); }
        if phase == "stagger" { actor.take_damage_with_stagger(1, 20); }
        if phase == "dead" { actor.take_damage_with_stagger(u32::MAX, 0); }
        let mut save: crate::save_v5::SaveV5 = serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();
        save.generic_actors = vec![actor.clone()]; save.validate().unwrap();
        let root = std::env::temp_dir().join(format!("brute-save-{phase}-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        crate::save_v5::write_save(&root, &save).unwrap();
        let bytes = std::fs::read(crate::save_v5::save_path(&root)).unwrap();
        let loaded = crate::save_v5::read_save(&root).unwrap();
        let restored = loaded.restore_state().unwrap();
        assert_eq!(restored.world.generic_actors, vec![actor.clone()]);
        assert_eq!(std::fs::read(crate::save_v5::save_path(&root)).unwrap(), bytes);
        let mut continued = restored.world.generic_actors[0].clone();
        for _ in 0..150 {
            let target = if phase == "slam" { p(6.0, 8.0) } else { p(9.0, 8.0) };
            assert_eq!(actor.tick(target, &arena(), DT, false), continued.tick(target, &arena(), DT, false));
            assert_eq!(actor, continued);
        }
    }
}

#[test]
fn malformed_or_foreign_profile_attack_state_is_rejected_before_save_write() {
    let mut actor = brute(); reach(&mut actor, p(9.0, 8.0), ActorAiState::Active);
    let mut save: crate::save_v5::SaveV5 = serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();
    save.generic_actors = vec![actor.clone()];
    let root = std::env::temp_dir().join(format!("brute-invalid-save-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    crate::save_v5::write_save(&root, &save).unwrap();
    let bytes = std::fs::read(crate::save_v5::save_path(&root)).unwrap();
    for fault in ["missing", "direction", "timer", "elapsed", "origin", "position", "kind"] {
        let mut bad = actor.clone();
        match fault {
            "missing" => bad.ordinary = None,
            "direction" => bad.ordinary.as_mut().unwrap().committed_direction = Some([f32::NAN, 0.0]),
            "timer" => bad.state_remaining_ms = u64::MAX,
            "elapsed" => bad.ordinary.as_mut().unwrap().active.as_mut().unwrap().elapsed_ms = 800,
            "origin" => bad.windup_origin_m = Some(p(30.0, 8.0)),
            "position" => bad.position_m.x_m += 1.0,
            "kind" => bad.current_attack = Some(ActorAttackKind::Leap),
            _ => unreachable!(),
        }
        assert!(!bad.validate(), "{fault}");
        let mut invalid = save.clone(); invalid.generic_actors = vec![bad]; assert!(invalid.validate().is_err(), "{fault}");
        assert!(crate::save_v5::write_save(&root, &invalid).is_err(), "{fault}");
        assert_eq!(std::fs::read(crate::save_v5::save_path(&root)).unwrap(), bytes);
    }
    for kind in [ActorAttackKind::Charge, ActorAttackKind::Slam] {
        for ty in ["enemy.clockworks.forged_guard", "enemy.clockworks.pressure_drone", "enemy.clockworks.furnace_hound"] {
            let mut foreign = ActorRuntime::spawn("foreign", ty, p(5.0, 8.0)).unwrap();
            foreign.state = ActorAiState::Windup; foreign.current_attack = Some(kind); foreign.windup_origin_m = Some(foreign.position_m); foreign.state_remaining_ms = 1;
            assert!(!foreign.validate());
        }
    }
}

#[test]
fn paused_fatal_or_stale_owner_cannot_advance_charge_or_slam() {
    for target in [p(6.0, 8.0), p(9.0, 8.0)] {
        let mut actor = brute(); reach(&mut actor, target, ActorAiState::Windup);
        let before = actor.clone(); assert_eq!(actor.tick(target, &arena(), 0.0, false), ActorTickOutput::default()); assert_eq!(actor, before);
        let mut world = WorldStateV3::new("grey_hive", 8, target).unwrap(); world.generic_actors = vec![actor];
        let wrong = InputSample::new(9, 1, 17, 0.0, 0.0).unwrap();
        assert!(step_world(&mut world, &arena(), CwStepInput { sample: &wrong, dt_s: DT, combat: &[] }).is_err());
        assert_eq!(world.generic_actors, vec![before.clone()]);
        // Explicit fatal unit fixture; no campaign HP injection or victory claim.
        world.player_hp = 0;
        for seq in 1..=100 { assert!(owner_step(&mut world, seq, &[]).actor_runtime.is_empty()); }
        assert_eq!(world.generic_actors, vec![before]); assert_eq!(world.player_hp, 0);
    }
}
