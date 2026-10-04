//! Bounded unit fixtures for the real actor and world-input paths. These arrange
//! a small arena/initial roster; they are not a campaign playthrough proof.
use super::*;
use crate::{continuous_combat::{CombatEvent, CombatIntent}, continuous_input::InputSample,
    continuous_kcc::{Aabb, StaticKccWorld}, world_v3::{step_world, CwStepInput, WorldStateV3}};

const DRONE: &str = "enemy.clockworks.pressure_drone";
const HOUND: &str = "enemy.clockworks.furnace_hound";
const DT: f32 = 1.0 / 60.0;
fn p(x: f32, z: f32) -> Vec3 { Vec3::new(x, 0.0, z).unwrap() }
fn arena() -> StaticKccWorld { StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![]) }
fn spawn(kind: &str) -> ActorRuntime { ActorRuntime::spawn("fixture", kind, p(5.0, 8.0)).unwrap() }
fn reach(actor: &mut ActorRuntime, target: Vec3, wanted: ActorAiState, kcc: &StaticKccWorld) {
    for _ in 0..500 {
        if actor.state == wanted { return; }
        actor.tick(target, kcc, DT, false);
        assert!(actor.validate());
    }
    panic!("did not reach {wanted:?}");
}
fn impacts(events: &[ActorRuntimeEvent]) -> Vec<(ActorAttackKind, bool)> {
    events.iter().filter_map(|e| if let ActorRuntimeEvent::AttackImpact{kind,hit_player,..}=e {Some((*kind,*hit_player))} else {None}).collect()
}

#[test]
fn ordinary_profiles_are_semantic_generic_and_data_tuned() {
    let drone = actor_profile(DRONE).unwrap();
    let hound = actor_profile(HOUND).unwrap();
    assert_eq!(drone.render_type, DRONE);
    assert_eq!(hound.render_type, HOUND);
    assert!(drone.ordinary.as_ref().unwrap().projectile.is_some());
    assert!(drone.pressure_wave.is_none());
    assert!(hound.ordinary.as_ref().unwrap().leap.is_some());
    assert!(hound.speed_mps > actor_profile("enemy.clockworks.forged_guard").unwrap().speed_mps);
    assert!(spawn(DRONE).validate() && spawn(HOUND).validate());
}

#[test]
fn pressure_shot_has_real_flight_pressure_tag_and_committed_aim() {
    let kcc = arena();
    let player = p(9.0, 8.0);
    let mut drone = spawn(DRONE);
    reach(&mut drone, player, ActorAiState::Active, &kcc);
    assert_eq!(drone.current_attack, Some(ActorAttackKind::PressureShot));
    let before_flight = drone.clone();
    let first = drone.tick(player, &kcc, DT, false);
    assert_eq!(first.damage_to_player, None, "a shot is not a ranged instant melee hit");
    assert!(first.events.iter().any(|e| matches!(e, ActorRuntimeEvent::AttackMotion { kind: ActorAttackKind::PressureShot, from_m, to_m, .. } if to_m.x_m > from_m.x_m)));
    let mut damage = 0;
    for _ in 0..100 {
        let out = drone.tick(player, &kcc, DT, false);
        if let Some(hit) = out.damage_to_player {
            assert_eq!(out.damage_tag, Some(crate::effects::HazardTag::Pressure));
            damage += hit;
        }
        if drone.state == ActorAiState::Cooldown { break; }
    }
    assert_eq!(damage, actor_profile(DRONE).unwrap().attack_damage);
    let mut dodged = before_flight;
    let mut events = vec![];
    while dodged.state == ActorAiState::Active {
        let out = dodged.tick(p(9.0, 11.0), &kcc, DT, false);
        assert_eq!(out.damage_to_player, None);
        events.extend(out.events);
    }
    assert_eq!(impacts(&events), vec![(ActorAttackKind::PressureShot, false)]);
    assert_eq!(dodged.position_m, p(5.0, 8.0), "shot travel does not teleport the actor");
}

#[test]
fn pressure_shot_sweeps_cover_and_cannot_hit_through_a_thin_wall() {
    let mut drone = spawn(DRONE);
    let player = p(9.0, 8.0);
    reach(&mut drone, player, ActorAiState::Active, &arena());
    let covered = StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![Aabb::new(7.03, 7.04, 7.0, 9.0).unwrap()]);
    let mut seen = vec![];
    while drone.state == ActorAiState::Active {
        let out = drone.tick(player, &covered, 0.25, false);
        assert_eq!(out.damage_to_player, None);
        seen.extend(out.events);
    }
    assert_eq!(impacts(&seen), vec![(ActorAttackKind::PressureShot, false)]);
}

#[test]
fn hound_chases_fast_and_uses_bite_and_committed_leap_without_heat_damage() {
    let kcc = arena();
    let mut hound = spawn(HOUND);
    let far = p(11.0, 8.0);
    hound.tick(far, &kcc, DT, false);
    hound.tick(far, &kcc, DT, false);
    assert!((hound.position_m.x_m - 5.0 - actor_profile(HOUND).unwrap().speed_mps * 0.017).abs() < 0.0001);
    let leap_target = p(8.0, 8.0);
    reach(&mut hound, leap_target, ActorAiState::Active, &kcc);
    assert_eq!(hound.current_attack, Some(ActorAttackKind::Leap));
    let mut count = 0;
    while hound.state == ActorAiState::Active {
        let out = hound.tick(leap_target, &kcc, DT, false);
        if out.damage_to_player.is_some() { count += 1; assert_eq!(out.damage_tag, None); }
        assert!(hound.validate());
    }
    assert_eq!(count, 1);
    let mut bite = spawn(HOUND);
    let mut found = false;
    for _ in 0..80 {
        let out = bite.tick(p(5.5, 8.0), &kcc, DT, false);
        if let Some(damage) = out.damage_to_player {
            assert_eq!(damage, actor_profile(HOUND).unwrap().attack_damage);
            assert_eq!(impacts(&out.events), vec![(ActorAttackKind::Bite, true)]);
            found = true; break;
        }
    }
    assert!(found);
}

#[test]
fn hound_leap_can_miss_or_be_blocked_and_investigate_expires() {
    let kcc = arena();
    let target = p(8.0, 8.0);
    let mut hound = spawn(HOUND);
    reach(&mut hound, target, ActorAiState::Active, &kcc);
    while hound.state == ActorAiState::Active {
        assert_eq!(hound.tick(p(8.0, 12.0), &kcc, DT, false).damage_to_player, None);
    }
    let mut blocked = spawn(HOUND);
    reach(&mut blocked, target, ActorAiState::Active, &kcc);
    let wall = StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![Aabb::new(6.0, 6.05, 7.0, 9.0).unwrap()]);
    while blocked.state == ActorAiState::Active {
        assert_eq!(blocked.tick(target, &wall, 0.25, false).damage_to_player, None);
    }
    assert!(blocked.position_m.x_m < 6.0 && blocked.validate());
    let mut investigating = spawn(HOUND);
    investigating.tick(p(11.0, 8.0), &kcc, DT, false);
    investigating.tick(p(11.0, 8.0), &wall, DT, false);
    assert_eq!(investigating.state, ActorAiState::Investigate);
    for _ in 0..100 { investigating.tick(p(11.0, 8.0), &wall, DT, false); }
    assert_eq!(investigating.state, ActorAiState::Idle);
}

#[test]
fn active_and_resolved_leap_saves_resume_exactly_without_replayed_hits() {
    for kind in [DRONE, HOUND] {
        let kcc = arena();
        let target = p(8.0, 8.0);
        let mut original = spawn(kind);
        reach(&mut original, target, ActorAiState::Windup, &kcc);
        // Every tick compares a serialized Continue with uninterrupted simulation,
        // including windup, flight/leap, post-contact and recovery boundaries.
        let mut hits = 0;
        for _ in 0..110 {
            let mut restored: ActorRuntime = serde_json::from_slice(&serde_json::to_vec(&original).unwrap()).unwrap();
            assert!(restored.validate());
            let expected = original.tick(target, &kcc, DT, false);
            let actual = restored.tick(target, &kcc, DT, false);
            assert_eq!(actual, expected);
            assert_eq!(restored, original);
            if actual.damage_to_player.is_some() { hits += 1; }
        }
        assert_eq!(hits, 1);
    }
}

#[test]
fn malformed_controller_states_reject_before_they_can_attack() {
    let mut valid = spawn(DRONE);
    reach(&mut valid, p(9.0, 8.0), ActorAiState::Active, &arena());
    let mut variants = vec![];
    let mut v=valid.clone(); v.state_remaining_ms=u64::MAX; variants.push(v);
    let mut v=valid.clone(); v.ordinary.as_mut().unwrap().active.as_mut().unwrap().elapsed_ms=u64::MAX; variants.push(v);
    let mut v=valid.clone(); v.ordinary.as_mut().unwrap().active.as_mut().unwrap().direction=[2.0,0.0]; variants.push(v);
    let mut v=valid.clone(); v.ordinary.as_mut().unwrap().committed_direction=Some([f32::NAN,0.0]); variants.push(v);
    let mut v=valid.clone(); v.ordinary.as_mut().unwrap().active.as_mut().unwrap().origin_m.x_m=f32::INFINITY; variants.push(v);
    let mut v=valid.clone(); v.ordinary.as_mut().unwrap().active.as_mut().unwrap().hit_resolved=true; variants.push(v);
    let mut v=valid.clone(); v.current_attack=Some(ActorAttackKind::Leap); variants.push(v);
    let mut v=valid.clone(); v.position_m.x_m+=1.0; variants.push(v);
    let mut v=valid.clone(); v.ordinary=None; variants.push(v);
    let mut v=valid.clone(); v.state=ActorAiState::Idle; variants.push(v);
    let mut v=valid.clone(); v.hp=0; variants.push(v);
    for mut bad in variants {
        assert!(!bad.validate(), "malformed fixture accepted: {bad:?}");
        assert_eq!(bad.tick(p(9.0,8.0), &arena(), DT, false), ActorTickOutput::default());
    }
    let mut old = ActorRuntime::spawn("legacy","enemy.clockworks.forged_guard",p(5.0,8.0)).unwrap();
    let mut value=serde_json::to_value(&old).unwrap(); value.as_object_mut().unwrap().remove("ordinary");
    assert_eq!(serde_json::from_value::<ActorRuntime>(value).unwrap(),old);
    old.ordinary=valid.ordinary.clone(); assert!(!old.validate());
    let mut value=serde_json::to_value(valid).unwrap(); value["ordinary"]["unexpected"]=true.into();
    assert!(serde_json::from_value::<ActorRuntime>(value).is_err());
}

#[test]
fn stagger_and_death_cancel_inflight_attacks_and_emit_named_cues() {
    for kind in [DRONE,HOUND] {
        let mut actor=spawn(kind);
        reach(&mut actor,p(8.0,8.0),ActorAiState::Active,&arena());
        let hp=actor.hp;
        let threshold=actor_profile(kind).unwrap().ordinary.as_ref().unwrap().stagger_threshold;
        let events=actor.take_damage_with_stagger(0,threshold);
        assert_eq!(actor.hp,hp,"Guard counter-stagger causes no invented damage");
        assert_eq!(actor.state,ActorAiState::Stagger);
        assert!(events.iter().any(|e| matches!(e,ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Stagger,..})));
        assert!(actor.validate());
        assert_eq!(actor.tick(p(8.0,8.0),&arena(),DT,false).damage_to_player,None);
        let events=actor.take_damage_with_stagger(hp,threshold);
        assert_eq!(actor.state,ActorAiState::Dead);
        assert!(events.iter().any(|e| matches!(e,ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Death,..})));
        assert!(actor.validate());
        for _ in 0..120 { assert_eq!(actor.tick(p(8.0,8.0),&arena(),DT,false),ActorTickOutput::default()); }
    }
}

fn world_step(world: &mut WorldStateV3, seq: u64, axes: (f32,f32), intents: &[CombatIntent]) -> crate::world_v3::CwStepOutput {
    let sample=InputSample::new(8,seq,seq*17,axes.0,axes.1).unwrap();
    step_world(world,&arena(),CwStepInput{sample:&sample,dt_s:DT,combat:intents}).unwrap()
}
#[test]
fn ranged_hits_guard_and_dash_use_real_world_inputs_in_mixed_roster_order() {
    for reverse in [false,true] {
        for mode in ["hit","guard","dash"] {
            let mut world=WorldStateV3::new("clockworks",8,p(9.0,8.0)).unwrap();
            world.scene_id="cw_pressure_hall".into();
            world.combat_state.qer_v1_active=true;
            let drone=ActorRuntime::spawn("drone",DRONE,p(5.0,8.0)).unwrap();
            let other=ActorRuntime::spawn("guard","enemy.clockworks.forged_guard",p(24.0,15.0)).unwrap();
            world.generic_actors=if reverse {vec![other,drone]}else{vec![drone,other]};
            let mut action_sent=false;
            let mut guards=0;
            for seq in 1..=100 {
                let flying=world.generic_actors.iter().find(|a|a.entity_id=="drone").unwrap().state==ActorAiState::Active;
                let act=flying&&!action_sent;
                let intents=if act&&mode=="guard" {vec![CombatIntent::GuardStart{request_id:seq}]}
                    else if act&&mode=="dash" {vec![CombatIntent::ActionDash{request_id:seq}]} else {vec![]};
                if act {action_sent=true;}
                let axes=if act&&mode=="dash" {(0.0,1.0)} else {(0.0,0.0)};
                let out=world_step(&mut world,seq,axes,&intents);
                assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::IntentRejected{..})),"{mode}");
                guards+=out.combat.iter().filter(|e|matches!(e,CombatEvent::GuardImpact{source_id,..}if source_id=="drone")).count();
            }
            if mode=="hit" { assert_eq!(world.player_hp,100-actor_profile(DRONE).unwrap().attack_damage); }
            else { assert_eq!(world.player_hp,100,"{mode}"); }
            if mode=="guard" {assert_eq!(guards,1);}
            if mode=="dash" {assert!(world.player.position_m.z_m>8.0);}
        }
    }
}

#[test]
fn simultaneous_pressure_shots_share_owner_invulnerability_without_index_special_cases() {
    let mut world = WorldStateV3::new("clockworks", 8, p(9.0, 8.0)).unwrap();
    world.scene_id = "cw_pressure_hall".into();
    world.combat_state.qer_v1_active = true;
    world.generic_actors = vec![
        ActorRuntime::spawn("left-shot", DRONE, p(5.0, 8.0)).unwrap(),
        ActorRuntime::spawn("right-shot", DRONE, p(13.0, 8.0)).unwrap(),
    ];
    let mut damage = 0;
    let mut absorbed = 0;
    for seq in 1..=100 {
        let out = world_step(&mut world, seq, (0.0, 0.0), &[]);
        damage += out.combat.iter().filter(|e| matches!(e, CombatEvent::PlayerDamaged { .. })).count();
        absorbed += out.combat.iter().filter(|e| matches!(e, CombatEvent::DamageAbsorbed { .. })).count();
    }
    assert_eq!(world.player_hp, 100 - actor_profile(DRONE).unwrap().attack_damage);
    assert_eq!((damage, absorbed), (1, 1));
}

#[test]
fn paused_actor_time_and_wrong_world_input_cannot_advance_an_active_shot() {
    let mut actor = spawn(DRONE);
    reach(&mut actor, p(9.0, 8.0), ActorAiState::Active, &arena());
    let before = actor.clone();
    assert_eq!(actor.tick(p(9.0, 8.0), &arena(), 0.0, false), ActorTickOutput::default());
    assert_eq!(actor, before);
    let mut world = WorldStateV3::new("clockworks", 8, p(9.0, 8.0)).unwrap();
    world.generic_actors.push(actor);
    let sample = InputSample::new(9, 1, 17, 0.0, 0.0).unwrap();
    assert!(step_world(&mut world, &arena(), CwStepInput { sample: &sample, dt_s: DT, combat: &[] }).is_err());
    assert_eq!(world.generic_actors, vec![before]);
    assert_eq!(world.player_hp, 100);
    let mut replacement = WorldStateV3::new("clockworks", 9, p(9.0, 8.0)).unwrap();
    let out = step_world(&mut replacement, &arena(), CwStepInput { sample: &sample, dt_s: DT, combat: &[] }).unwrap();
    assert!(out.actor_runtime.is_empty(), "actor-owned flight has no global late-hit queue");
    assert_eq!(replacement.player_hp, 100);
}

fn pressure_arena(immune: bool) -> StaticKccWorld {
    use crate::effects::{EffectLifetime, EffectResolver, EffectSource, EffectSourceKind, EffectSpec,
        HazardImmunityEffect, HazardResistanceEffect, HazardTag, StackRule};
    let effect = if immune {
        EffectSpec::HazardImmunity(HazardImmunityEffect { tag: HazardTag::Pressure, stack: StackRule::Any })
    } else {
        EffectSpec::HazardResistance(HazardResistanceEffect { tag: HazardTag::Pressure, reduction_bps: 5_000, stack: StackRule::MultiplyRemaining })
    };
    let rules = EffectResolver::resolve(&[EffectSource {
        source_kind: EffectSourceKind::TemporaryBuff,
        source_id: "bounded_test_pressure_rule".into(), instance_id: "fixture-rule".into(),
        lifetime: EffectLifetime::Equipped, ui_category: None, effects: vec![effect],
    }]).unwrap();
    arena().with_movement_rules(rules, crate::player_rules::MovementMode::Ground)
}

#[test]
fn pressure_resistance_immunity_guard_and_iframes_use_the_real_shared_owner_path() {
    for mode in ["resistance", "immunity", "guard", "iframes"] {
        let kcc = pressure_arena(mode == "immunity");
        let mut world = WorldStateV3::new("clockworks", 8, p(9.0, 8.0)).unwrap();
        world.scene_id = "cw_pressure_hall".into();
        world.combat_state.qer_v1_active = true;
        world.generic_actors.push(ActorRuntime::spawn("drone", DRONE, p(5.0, 8.0)).unwrap());
        if mode == "iframes" {
            world.generic_actors.push(ActorRuntime::spawn("second-drone", DRONE, p(13.0, 8.0)).unwrap());
        }
        let mut action_sent = false;
        let mut damage_events = vec![];
        let mut absorbed = 0;
        let mut guard_impacts = 0;
        for seq in 1..=100 {
            let flying = world.generic_actors[0].state == ActorAiState::Active;
            let intents = if mode == "guard" && flying && !action_sent {
                action_sent = true;
                vec![CombatIntent::GuardStart { request_id: seq }]
            } else { vec![] };
            let sample = InputSample::new(8, seq, seq * 17, 0.0, 0.0).unwrap();
            let out = step_world(&mut world, &kcc, CwStepInput { sample: &sample, dt_s: DT, combat: &intents }).unwrap();
            for event in out.combat {
                match event {
                    CombatEvent::PlayerDamaged { damage, .. } => damage_events.push(damage),
                    CombatEvent::DamageAbsorbed { .. } => absorbed += 1,
                    CombatEvent::GuardImpact { .. } => guard_impacts += 1,
                    CombatEvent::IntentRejected { reason, .. } => panic!("real Guard input rejected: {reason}"),
                    _ => {}
                }
            }
            assert!(!out.actor_runtime.iter().any(|e| matches!(e, ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Stagger, .. })),
                "projectile Guard must not remotely stagger the shooter");
        }
        let reduced = ((u64::from(actor_profile(DRONE).unwrap().attack_damage) * 5_000 + 5_000) / 10_000) as u32;
        if mode == "resistance" || mode == "iframes" {
            assert_eq!(damage_events, vec![reduced]);
            assert_eq!(world.player_hp, 100 - reduced);
        } else {
            assert!(damage_events.is_empty(), "fully mitigated hits do not emit PlayerDamaged");
            assert_eq!(world.player_hp, 100);
        }
        if mode == "guard" { assert_eq!(guard_impacts, 1); }
        if mode == "immunity" || mode == "iframes" { assert_eq!(absorbed, 1); }
    }
}

#[test]
fn hp_zero_world_freezes_actor_flight_and_windup_without_late_hits() {
    for kind in [DRONE, HOUND] {
        for phase in [ActorAiState::Windup, ActorAiState::Active] {
            let mut actor = spawn(kind);
            reach(&mut actor, p(8.0, 8.0), phase, &arena());
            let mut world = WorldStateV3::new("clockworks", 8, p(8.0, 8.0)).unwrap();
            world.generic_actors.push(actor.clone());
            // Bounded death-state fixture; no campaign HP mutation or proof claim.
            world.player_hp = 0;
            for seq in 1..=100 {
                let out = world_step(&mut world, seq, (0.0, 0.0), &[]);
                assert!(out.actor_runtime.is_empty());
                assert!(!out.combat.iter().any(|e| matches!(e, CombatEvent::PlayerDamaged { .. })));
            }
            assert_eq!(world.generic_actors, vec![actor]);
            assert_eq!(world.player_hp, 0);
        }
    }
}

#[test]
fn action_v2_hits_and_melee_guard_collect_authoritative_stagger_and_death_cues() {
    let mut world = WorldStateV3::new("clockworks", 8, p(5.0, 8.0)).unwrap();
    world.scene_id = "cw_boiler_chamber".into();
    world.combat_state.qer_v1_active = true;
    world.generic_actors.push(ActorRuntime::spawn("hound", HOUND, p(5.0, 8.8)).unwrap());
    let mut staggers = 0;
    let mut deaths = 0;
    for seq in 1..=140 {
        let intents = match seq {
            1 => vec![CombatIntent::Pulse { request_id: seq }],
            50 => vec![CombatIntent::Pierce { request_id: seq }],
            100 => vec![CombatIntent::ActionAttack { request_id: seq }],
            _ => vec![],
        };
        let out = world_step(&mut world, seq, (0.0, 0.0), &intents);
        assert!(!out.combat.iter().any(|e| matches!(e, CombatEvent::IntentRejected { .. })));
        for event in out.actor_runtime {
            match event {
                ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Stagger, .. } => staggers += 1,
                ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Death, .. } => deaths += 1,
                _ => {}
            }
        }
    }
    assert!(staggers >= 2);
    assert_eq!(deaths, 1);
    assert_eq!(world.generic_actors[0].state, ActorAiState::Dead);

    let mut guarded = WorldStateV3::new("clockworks", 8, p(5.0, 8.0)).unwrap();
    guarded.combat_state.qer_v1_active = true;
    guarded.generic_actors.push(ActorRuntime::spawn("hound", HOUND, p(5.0, 8.8)).unwrap());
    let mut guard_cues = 0;
    let hp = guarded.generic_actors[0].hp;
    for seq in 1..=40 {
        let intents = if seq == 1 { vec![CombatIntent::GuardStart { request_id: seq }] } else { vec![] };
        let out = world_step(&mut guarded, seq, (0.0, 0.0), &intents);
        guard_cues += out.actor_runtime.iter().filter(|e| matches!(e, ActorRuntimeEvent::EnemyCue { kind: ActorCueKind::Stagger, .. })).count();
    }
    assert_eq!(guard_cues, 1);
    assert_eq!(guarded.player_hp, 100);
    assert_eq!(guarded.generic_actors[0].hp, hp, "counter-stagger is not counter-damage");
}
