use super::*;
use crate::continuous_combat::{combat_v1, ActiveCombatAction, CombatIntent};
use crate::continuous_input::InputSample;
use crate::continuous_kcc::{Aabb, StaticKccWorld};
use crate::formal_runtime::{project_view, ActionCommandRequest, ActionKind, FormalRuntime};
use crate::sentinel_ai::{Sentinel, SentinelState};
use crate::world_v3::{step_world, ActionPhase, ActorRuntime, CwStepInput, PresentationEvent, Vec3, WorldStateV3};

fn p(x: f32, z: f32) -> Vec3 { Vec3::new(x, 0.0, z).unwrap() }
fn fixture() -> RuntimeState {
    let runtime = FormalRuntime::new().unwrap();
    runtime.pause().unwrap();
    let mut state = runtime.state.lock().unwrap().clone();
    state.world = WorldStateV3::new("grey_hive", 1, p(5.0, 5.0)).unwrap();
    state.world.scene_id = "feedback_fixture".into();
    state.kcc = StaticKccWorld::new(Aabb::new(0.0, 30.0, 0.0, 20.0).unwrap(), vec![]);
    state.presentation_events.clear();
    state.next_presentation_event_id = 1;
    state
}
fn step(state: &mut RuntimeState, intents: &[CombatIntent]) -> Vec<CombatEvent> {
    let seq = state.world.last_input_seq + 1;
    let sample = InputSample::new(state.world.revision.world_epoch, seq, seq * 17, 0.0, 0.0).unwrap().with_aim(0.0, 1.0).unwrap();
    step_world(&mut state.world, &state.kcc, CwStepInput { sample: &sample, dt_s: 1.0 / 60.0, combat: intents }).unwrap().combat
}
fn tick(state: &mut RuntimeState, intents: &[CombatIntent]) {
    let events = step(state, intents);
    present_events(state, &events);
}
fn sentinel(state: &mut RuntimeState, hp: u32, position: Vec3) {
    let mut actor = Sentinel::new("feedback_enemy", position, hp);
    actor.state = SentinelState::Recover;
    actor.state_remaining_ms = 10_000;
    state.world.sentinels = vec![actor];
}
fn result(state: &RuntimeState, outcome: CombatOutcome) -> Vec<&PresentationEvent> {
    state.presentation_events.iter().filter(|event| event.combat_feedback.as_ref().is_some_and(|feedback| feedback.outcome == outcome)).collect()
}

#[test]
fn combat_phase_windows_are_owner_timed_bounded_and_idle_or_dead_is_absent() {
    let mut state = fixture();
    assert!(project_view(&state).player.action_presentation.is_none());
    for kind in [CombatActionKind::PrimaryAttack, CombatActionKind::Dash, CombatActionKind::Pulse, CombatActionKind::Guard, CombatActionKind::Pierce] {
        let spec = &combat_v1().actions[kind.data_key()];
        let total = spec.windup_ms + spec.active_ms + spec.recovery_ms;
        for elapsed in [0, spec.windup_ms.saturating_sub(1), spec.windup_ms, spec.windup_ms + spec.active_ms - 1, spec.windup_ms + spec.active_ms, total - 1] {
            state.world.combat_state.active_action = Some(ActiveCombatAction { kind, request_id: 71, elapsed_ms: elapsed, impact_resolved: false, end_requested: false });
            let phase = project_view(&state).player.action_presentation.unwrap();
            assert_eq!(phase.phase, if elapsed < spec.windup_ms { ActionPhase::Windup } else if elapsed < spec.windup_ms + spec.active_ms { ActionPhase::Active } else { ActionPhase::Recovery });
            assert_eq!((phase.elapsed_ms, phase.duration_ms, phase.request_id), (elapsed, total, 71));
            assert_eq!((phase.range_m, phase.line_half_width_m), (spec.range_m, spec.line_half_width_m.unwrap_or(0.0)));
        }
        state.world.combat_state.active_action.as_mut().unwrap().elapsed_ms = total;
        assert!(project_view(&state).player.action_presentation.is_none());
    }
    state.world.combat_state.active_action.as_mut().unwrap().elapsed_ms = 1;
    state.world.player_hp = 0;
    assert_eq!(project_view(&state).player.action_state, "death");
    assert!(project_view(&state).player.action_presentation.is_none());
}

#[test]
fn early_guard_release_and_expiry_preserve_the_original_balance_and_total_clock() {
    for early in [true, false] {
        let mut state = fixture();
        tick(&mut state, &[CombatIntent::GuardStart { request_id: 1 }]);
        let started = project_view(&state).player.action_presentation.unwrap();
        assert_eq!(started.phase, ActionPhase::Windup);
        if early { tick(&mut state, &[CombatIntent::GuardEnd { request_id: 2 }]); }
        let spec = &combat_v1().actions["guard"];
        while state.world.combat_state.active_action.is_some() {
            let player = project_view(&state).player;
            let phase = player.action_presentation.unwrap();
            assert_eq!(player.action_state, "guard");
            assert_eq!(phase.duration_ms, started.duration_ms);
            if early || phase.elapsed_ms >= spec.windup_ms + spec.active_ms { assert_eq!(phase.phase, ActionPhase::Recovery); }
            tick(&mut state, &[]);
        }
        assert!(state.world.server_time_ms >= started.duration_ms);
        assert!(project_view(&state).player.action_presentation.is_none());
        assert_eq!(state.world.player_energy, 100 - spec.energy);
    }
}

#[test]
fn actual_primary_pulse_pierce_and_legacy_contacts_survive_death_and_later_motion_without_duplicates() {
    for (intent, expected_kind, start_kind) in [
        (CombatIntent::ActionAttack { request_id: 1 }, "Hit", "AttackStarted"),
        (CombatIntent::Pulse { request_id: 1 }, "PulseHit", "PulseCast"),
        (CombatIntent::Pierce { request_id: 1 }, "PierceHit", "PierceStarted"),
        (CombatIntent::Attack { request_id: 1 }, "Hit", "AttackStarted"),
    ] {
        let mut state = fixture();
        let contact_position = p(5.0, 6.0);
        sentinel(&mut state, 1, contact_position);
        for i in 0..50 {
            let events = step(&mut state, if i == 0 { std::slice::from_ref(&intent) } else { &[] });
            if !state.world.sentinels[0].active { state.world.sentinels[0].position_m = p(17.0, 12.0); }
            present_events(&mut state, &events);
        }
        let hits = result(&state, CombatOutcome::EnemyHit);
        assert_eq!(hits.len(), 1, "{intent:?}");
        assert_eq!(hits[0].kind, expected_kind);
        assert_eq!(hits[0].position_m, contact_position);
        assert_eq!(hits[0].direction_rad, 0.0);
        assert_eq!(hits[0].duration_ms, Some(220));
        let feedback = hits[0].combat_feedback.as_ref().unwrap();
        assert_eq!(feedback.world_id, "grey_hive");
        assert_eq!(feedback.scene_id, "feedback_fixture");
        assert_eq!(feedback.target_id.as_deref(), Some("feedback_enemy"));
        assert_eq!(feedback.source_id.as_deref(), Some("player"));
        assert_eq!(feedback.request_id, Some(1));
        assert_eq!(state.presentation_events.iter().filter(|event| event.kind == start_kind).count(), 1);
        let wire = serde_json::to_string(hits[0]).unwrap();
        assert!(!wire.contains("damage") && !wire.contains("Hp") && !wire.contains("health"));
    }
}

#[test]
fn actual_misses_and_rejections_never_become_hits_or_second_modern_starts() {
    for intent in [CombatIntent::ActionAttack { request_id: 1 }, CombatIntent::Pulse { request_id: 1 }, CombatIntent::Pierce { request_id: 1 }, CombatIntent::Attack { request_id: 1 }] {
        let mut state = fixture();
        sentinel(&mut state, 100, p(20.0, 15.0));
        tick(&mut state, &[intent]);
        for _ in 0..50 { tick(&mut state, &[]); }
        assert!(result(&state, CombatOutcome::EnemyHit).is_empty());
        let misses = result(&state, CombatOutcome::Rejected);
        assert_eq!(misses.len(), 1, "{intent:?}");
        assert_eq!(misses[0].combat_feedback.as_ref().unwrap().reason.as_deref(), Some("no_target"));
        assert_eq!(misses[0].duration_ms, Some(1000));
        assert!(state.presentation_events.iter().filter(|e| e.kind == "AttackStarted").count() <= 1);
    }
    let mut state = fixture(); state.world.player_energy = 0;
    tick(&mut state, &[CombatIntent::Pulse { request_id: 1 }]);
    assert_eq!(state.presentation_events.len(), 1);
    assert_eq!(result(&state, CombatOutcome::Rejected)[0].combat_feedback.as_ref().unwrap().reason.as_deref(), Some("insufficient_energy"));
    assert!(state.world.combat_state.active_action.is_none());
}

#[test]
fn real_player_hurt_guard_and_iframes_have_distinct_contact_metadata() {
    for mode in ["hurt", "guard", "iframe"] {
        let mut state = fixture();
        sentinel(&mut state, 100, p(5.0, 4.0)); // Behind the player's +Z facing.
        if mode == "guard" {
            tick(&mut state, &[CombatIntent::GuardStart { request_id: 1 }]);
            for _ in 0..5 { tick(&mut state, &[]); }
        }
        if mode == "iframe" { state.world.combat_state.qer_v1_active = true; state.world.combat_state.invulnerability_remaining_ms = 500; }
        state.world.sentinels[0].state = SentinelState::Attack;
        state.world.sentinels[0].state_remaining_ms = 1;
        tick(&mut state, &[]);
        let expected = match mode { "guard" => CombatOutcome::Blocked, "iframe" => CombatOutcome::Absorbed, _ => CombatOutcome::PlayerHurt };
        let events = result(&state, expected); assert_eq!(events.len(), 1);
        let event = events[0]; let metadata = event.combat_feedback.as_ref().unwrap();
        assert_eq!(event.position_m, state.world.player.position_m);
        assert_eq!(event.direction_rad, 0.0);
        assert_eq!(metadata.source_id.as_deref(), Some("feedback_enemy"));
        assert_eq!(metadata.target_id.as_deref(), Some("player"));
        assert_eq!(state.world.player_hp, if mode == "hurt" { 92 } else { 100 });
        if mode == "guard" { assert_eq!(metadata.request_id, Some(1)); }
        if mode == "iframe" { assert_eq!(metadata.reason.as_deref(), Some("invulnerable")); }
        if mode != "hurt" { assert!(result(&state, CombatOutcome::PlayerHurt).is_empty()); }
    }
}

#[test]
fn full_pressure_immunity_emits_absorbed_from_real_projectile_contact() {
    use crate::effects::{EffectLifetime, EffectResolver, EffectSource, EffectSourceKind, EffectSpec, HazardImmunityEffect, HazardTag, StackRule};
    let mut state = fixture();
    state.world.world_id = "clockworks".into(); state.world.scene_id = "cw_pressure_hall".into();
    state.world.player.position_m = p(9.0, 8.0);
    let rules = EffectResolver::resolve(&[EffectSource { source_kind: EffectSourceKind::TemporaryBuff, source_id: "test_immunity".into(), instance_id: "test_immunity".into(), lifetime: EffectLifetime::Equipped, ui_category: None, effects: vec![EffectSpec::HazardImmunity(HazardImmunityEffect { tag: HazardTag::Pressure, stack: StackRule::Any })] }]).unwrap();
    state.kcc = state.kcc.clone().with_movement_rules(rules, crate::player_rules::MovementMode::Ground);
    state.world.generic_actors.push(ActorRuntime::spawn("drone", "enemy.clockworks.pressure_drone", p(5.0, 8.0)).unwrap());
    for _ in 0..100 { tick(&mut state, &[]); }
    assert_eq!(state.world.player_hp, 100);
    assert!(result(&state, CombatOutcome::PlayerHurt).is_empty());
    let absorbed = result(&state, CombatOutcome::Absorbed); assert_eq!(absorbed.len(), 1);
    assert_eq!(absorbed[0].combat_feedback.as_ref().unwrap().reason.as_deref(), Some("damage_reduced_to_zero"));
    assert_eq!(absorbed[0].position_m, p(9.0, 8.0));
    assert_eq!(absorbed[0].direction_rad, std::f32::consts::FRAC_PI_2);
}

#[test]
fn q_and_r_keep_owner_facing_when_mouse_aim_changes_during_windup_and_active() {
    for intent in [CombatIntent::Pulse { request_id: 1 }, CombatIntent::Pierce { request_id: 1 }] {
        let mut state = fixture();
        tick(&mut state, &[intent]);
        let original = project_view(&state).player;
        state.latest_sample = InputSample::new(1, 100, 1700, 0.0, 0.0).unwrap().with_aim(1.0, 0.0).unwrap();
        for _ in 0..20 {
            let seq = state.world.last_input_seq + 1;
            let sample = InputSample::new(1, seq, seq * 17, 0.0, 0.0).unwrap().with_aim(1.0, 0.0).unwrap();
            step_world(&mut state.world, &state.kcc, CwStepInput { sample: &sample, dt_s: 1.0 / 60.0, combat: &[] }).unwrap();
            let view = project_view(&state).player;
            assert_eq!((view.aim_x, view.aim_z), (1.0, 0.0));
            assert_eq!((view.facing_x, view.facing_z), (original.facing_x, original.facing_z));
            assert_eq!(view.transform.yaw_rad, original.transform.yaw_rad);
            assert!(view.action_presentation.is_some());
        }
    }
}

#[test]
fn ambiguous_signal_target_is_not_revealed_but_authorized_stagger_and_lethal_hit_are_kept() {
    let mut state = fixture();
    let target = p(5.0, 6.0);
    let actor = ActorRuntime::spawn_with_controller_variant("wraith", "enemy.mist_harbor.signal_wraith", target, 1).unwrap();
    state.world.generic_actors.push(actor);
    let hit = CombatEvent::ActionImpact { action: CombatActionKind::Pulse, target_id: "wraith".into(), damage: 10, stagger: 35, contact: Some(CombatContact::new(state.world.player.position_m, target, Some(1))) };
    assert!(!super::super::signal_wraith_presentation::projection(&state, &state.world.generic_actors[0]).unwrap().precise);
    present_events(&mut state, &[hit.clone()]);
    assert!(state.presentation_events.is_empty(), "unmapped alternatives cannot gain a true-position marker");
    state.world.generic_actors[0].take_damage_with_stagger(10, 35);
    present_events(&mut state, &[hit.clone()]);
    assert_eq!(result(&state, CombatOutcome::EnemyHit).len(), 1);
    state.world.generic_actors[0].take_damage(1000);
    present_events(&mut state, &[hit]);
    assert_eq!(result(&state, CombatOutcome::EnemyHit).len(), 2, "death is a publicly precise signal phase");
}

#[test]
fn optional_schema_is_backward_readable_and_invalid_internal_metadata_fails_closed() {
    let mut state = fixture();
    let mut json = serde_json::to_value(project_view(&state).player).unwrap();
    assert!(json.get("actionPresentation").is_none());
    let _: crate::world_v3::PlayerView = serde_json::from_value(json.clone()).unwrap();
    tick(&mut state, &[CombatIntent::Pierce { request_id: 1 }]);
    json = serde_json::to_value(project_view(&state).player).unwrap();
    assert_eq!(json["actionPresentation"]["phase"], "windup");
    assert!((json["actionPresentation"]["lineHalfWidthM"].as_f64().unwrap() - 0.45).abs() < 0.000001);
    super::super::emit_presentation(&mut state, "Fixture", p(5.0, 5.0), 1.0);
    let generic = state.presentation_events.last().unwrap();
    let wire = serde_json::to_value(generic).unwrap();
    assert!(wire.get("combatFeedback").is_none());
    let _: PresentationEvent = serde_json::from_value(wire).unwrap();
    state.presentation_events.clear();
    sentinel(&mut state, 100, p(5.0, 6.0));
    for (id, request_id, position) in [("feedback_enemy", super::super::build_ui::MAX_SAFE_REVISION + 1, p(5.0, 6.0)), ("unknown", 1, p(5.0, 6.0)), ("feedback_enemy", 1, Vec3 { x_m: f32::NAN, ..p(5.0, 6.0) })] {
        present_events(&mut state, &[CombatEvent::ActionImpact { action: CombatActionKind::Pierce, target_id: id.into(), damage: 32, stagger: 25, contact: Some(CombatContact { position_m: position, direction_rad: 0.0, request_id: Some(request_id) }) }]);
    }
    present_events(&mut state, &[CombatEvent::PlayerDamaged { source_id: "feedback_enemy".into(), damage: 0, contact: Some(CombatContact::new(p(5.0, 6.0), p(5.0, 5.0), None)) }]);
    assert!(state.presentation_events.is_empty());
}

#[test]
fn pause_freezes_pulse_time_releases_guard_and_continue_cannot_replay_action_feedback() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let root = std::env::temp_dir().join(format!("combat-presentation-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    let epoch = runtime.snapshot().unwrap().world_epoch;
    let start = runtime.submit_action(ActionCommandRequest { protocol_version: 2, world_epoch: epoch, request_id: 1, client_time_ms: 0, kind: ActionKind::Pulse }).unwrap();
    assert!(start.player.action_presentation.is_some());
    let paused = runtime.pause().unwrap();
    let phase = paused.player.action_presentation.clone().unwrap();
    std::thread::sleep(Duration::from_millis(45));
    assert_eq!(runtime.snapshot().unwrap().player.action_presentation, Some(phase.clone()));
    runtime.resume().unwrap();
    let resumed = runtime.submit_input(InputSample::new(epoch, 1, 17, 0.0, 0.0).unwrap(), vec![]).unwrap();
    assert!(resumed.player.action_presentation.unwrap().elapsed_ms > phase.elapsed_ms);
    runtime.pause().unwrap();
    runtime.save_slot("alive", "Alive", true).unwrap();
    let restored = runtime.continue_slot("alive").unwrap();
    assert!(restored.world_epoch > epoch);
    assert!(restored.player.action_presentation.is_none());
    assert!(runtime.presentation_events_since(restored.world_epoch, 0).unwrap().is_empty());
    runtime.scene_ready(restored.entry_token.as_ref().unwrap(), false).unwrap();
    let guard = runtime.submit_action(ActionCommandRequest { protocol_version: 2, world_epoch: restored.world_epoch, request_id: 2, client_time_ms: 18, kind: ActionKind::GuardStart }).unwrap();
    assert_eq!(guard.player.action_state, "guard");
    assert!(runtime.pause().unwrap().player.action_presentation.is_none());
    assert!(runtime.resume().unwrap().player.action_presentation.is_none());
    assert_eq!(runtime.submit_action(ActionCommandRequest { protocol_version: 2, world_epoch: restored.world_epoch, request_id: super::super::build_ui::MAX_SAFE_REVISION + 1, client_time_ms: 19, kind: ActionKind::Pulse }).unwrap_err(), "E_ACTION_REQUEST_INVALID");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn new_pulse_and_pierce_miss_signals_do_not_restart_regeneration_delay() {
    use crate::capability_v1::{tick_regeneration_authorized, CapabilityState, CapabilityTickInput, RegenerationConfig};
    for intent in [CombatIntent::Pulse { request_id: 1 }, CombatIntent::Pierce { request_id: 1 }] {
        let mut state = fixture();
        let mut misses = vec![];
        for i in 0..40 {
            let events = step(&mut state, if i == 0 { std::slice::from_ref(&intent) } else { &[] });
            misses.extend(events.into_iter().filter(|event| matches!(event, CombatEvent::ActionMiss { .. })));
        }
        assert_eq!(misses.len(), 1);
        assert!(!misses[0].counts_as_combat_activity());
        let mut empty = CapabilityState::new("grey_hive", RegenerationConfig::new(1000, 1000, 10.0, 1.0).unwrap()).unwrap();
        let mut with_miss = empty.clone();
        let evaluate = |capabilities: &mut CapabilityState, events: &[CombatEvent]| tick_regeneration_authorized(capabilities, CapabilityTickInput {
            revision: crate::world_v3::WorldRevision::new(1, 1, 1).unwrap(), now_ms: 2000, dt_s: 0.2,
            current_hp: 50, max_hp: 100, last_damaged_at_ms: None, last_combat_at_ms: Some(0), combat_events: events,
        }, true).unwrap().effects.into_iter().map(|effect| match effect {
            crate::world_v3::WorldEffect::HealPlayer { amount } => amount,
            _ => panic!("regeneration produced unrelated effect"),
        }).collect::<Vec<_>>();
        assert_eq!(evaluate(&mut empty, &[]), evaluate(&mut with_miss, &misses));
        assert_eq!(empty, with_miss);
    }
}

// Append to formal_runtime/combat_presentation/tests.rs after the active build finishes.
// Arranged fixture with a fresh charge-enabled actor; all attacks and Guard resolution
// below advance through the shared owner, with no fabricated contact or HP changes.
#[test]
fn real_sentinel_charge_hurt_and_guard_reach_truthful_contact_presentation_once() {
    for guarded in [false, true] {
        let mut state = fixture();
        let mut enemy = Sentinel::new("feedback_enemy", p(2.0, 5.0), 100);
        enemy.enable_charge();
        state.world.sentinels = vec![enemy];
        let mut guard_sent = false;
        let mut resolved = false;
        for _ in 0..150 {
            let actor = &state.world.sentinels[0];
            let from = actor.position_m;
            let guard_now = guarded && !guard_sent
                && actor.state == SentinelState::ChargeWindup
                && actor.state_remaining_ms <= 100;
            let intents = if guard_now {
                guard_sent = true;
                vec![CombatIntent::GuardStart { request_id: 777 }]
            } else { vec![] };
            let events = step(&mut state, &intents);
            let contact = events.iter().find_map(|event| match event {
                CombatEvent::GuardImpact { contact, .. } if guarded => *contact,
                CombatEvent::PlayerDamaged { contact, .. } if !guarded => *contact,
                _ => None,
            });
            present_events(&mut state, &events);
            if let Some(contact) = contact {
                let expected = CombatContact::new(from, state.world.player.position_m,
                    guarded.then_some(777));
                assert_eq!(contact, expected);
                assert_eq!(state.world.sentinels[0].attack_serial, 0, "charge must not alter old melee cadence");
                assert_eq!(state.world.sentinels[0].charge_controller.as_ref().unwrap().charge_serial, 1);
                let outcome = if guarded { CombatOutcome::Blocked } else { CombatOutcome::PlayerHurt };
                let rows = result(&state, outcome);
                assert_eq!(rows.len(), 1);
                let row = rows[0];
                assert_eq!(row.kind, if guarded { "GuardImpact" } else { "Damaged" });
                assert_eq!(row.position_m, contact.position_m);
                assert_eq!(row.direction_rad, contact.direction_rad);
                assert_eq!(row.duration_ms, Some(260));
                let metadata = row.combat_feedback.as_ref().unwrap();
                assert_eq!(metadata.world_id, "grey_hive");
                assert_eq!(metadata.scene_id, "feedback_fixture");
                assert_eq!(metadata.source_id.as_deref(), Some("feedback_enemy"));
                assert_eq!(metadata.target_id.as_deref(), Some("player"));
                assert_eq!(metadata.request_id, guarded.then_some(777));
                assert_eq!(state.world.player_hp, if guarded { 100 } else { 84 });
                assert_eq!(state.world.player_energy, if guarded { 85 } else { 100 });
                if guarded {
                    assert_eq!(state.world.sentinels[0].state, SentinelState::Stagger);
                    assert!(state.world.sentinels[0].charge_controller.as_ref().unwrap().attack.is_none());
                    assert!(state.world.sentinels[0].encounter_view(60).unwrap().warning.is_none());
                    assert!(result(&state, CombatOutcome::PlayerHurt).is_empty());
                } else {
                    assert!(state.world.sentinels[0].charge_controller.as_ref().unwrap().attack.as_ref().unwrap().hit_resolved);
                    assert!(result(&state, CombatOutcome::Blocked).is_empty());
                }
                // Cover the rest of this committed sweep / initial counter-stagger;
                // a new melee attack is outside this test's single-charge boundary.
                for _ in 0..12 { tick(&mut state, &[]); }
                assert_eq!(result(&state, outcome).len(), 1);
                assert_eq!(state.world.player_hp, if guarded { 100 } else { 84 });
                resolved = true;
                break;
            }
        }
        assert!(resolved, "shared owner never resolved charge contact; guarded={guarded}");
        assert_eq!(guard_sent, guarded);
    }
}
