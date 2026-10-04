use super::*;
use crate::formal_runtime::clockworks_roster::tests::fixture;
use crate::formal_runtime::entry_test_support::acknowledge_ready;

fn tick_actor(runtime: &FormalRuntime, id: &str, player: Vec3) -> Vec<PresentationEvent> {
    let mut state = runtime.state.lock().unwrap();
    let kcc = state.kcc.clone();
    let actor = state.world.generic_actors.iter_mut().find(|actor| actor.entity_id == id).unwrap();
    let output = actor.tick(player, &kcc, 1.0 / 60.0, true);
    let scene = runtime.scene_runtime.lock().unwrap();
    let offset = state.presentation_events.len();
    present_events(&mut state, scene.as_ref(), &output.events);
    state.presentation_events[offset..].to_vec()
}
fn drone() -> (FormalRuntime, String, Vec3) {
    let runtime = fixture("cw_pressure_hall");
    let actor = runtime.state.lock().unwrap().world.generic_actors.iter().find(|a| a.entity_type == clockworks_roster::DRONE).unwrap().clone();
    (runtime, actor.entity_id, Vec3 { z_m: actor.position_m.z_m + 3.0, ..actor.position_m })
}
fn windup(runtime: &FormalRuntime, id: &str, player: Vec3) -> PresentationEvent {
    for _ in 0..120 {
        for event in tick_actor(runtime, id, player) {
            if event.kind == "EnemyAttackTelegraph" { return event; }
        }
    }
    panic!("no authored enemy telegraph");
}

#[test]
fn warning_uses_committed_profile_geometry_lifetime_and_no_hp_fields() {
    let (runtime, id, player) = drone();
    let event = windup(&runtime, &id, player);
    let profile = crate::world_v3::actor_profile(clockworks_roster::DRONE).unwrap();
    let projectile = profile.ordinary.as_ref().unwrap().projectile.as_ref().unwrap();
    assert_eq!(event.actor_id.as_deref(), Some(id.as_str()));
    assert_eq!(event.attack_kind.as_deref(), Some("pressure_shot"));
    assert_eq!(event.radius_m, projectile.radius_m);
    assert_eq!(event.range_m, Some(projectile.travel_m));
    assert_eq!(event.direction_rad, 0.0);
    assert_eq!(event.duration_ms, Some((profile.windup_ms.div_ceil(17) * 1000).div_ceil(60)));
    tick_actor(&runtime, &id, Vec3 { x_m: player.x_m + 4.0, ..player });
    assert_eq!(runtime.state.lock().unwrap().presentation_events.iter().find(|e| e.event_id == event.event_id).unwrap(), &event);
    let json = serde_json::to_value(&event).unwrap();
    assert!(!json.as_object().unwrap().keys().any(|key| key.to_lowercase().contains("hp") || key.contains("damage")));
    assert!(runtime.snapshot().unwrap().capabilities.enemy_vitals.is_empty());
    assert!(runtime.state.lock().unwrap().presentation_events.iter().any(|e| e.kind == "EnemyAlert" && e.duration_ms.unwrap() > 0));
}

#[test]
fn hound_leap_warning_has_committed_travel_width_and_direction() {
    let runtime = fixture("cw_boiler_chamber");
    let actor = runtime.state.lock().unwrap().world.generic_actors[0].clone();
    let target = Vec3 { x_m: actor.position_m.x_m + 4.0, ..actor.position_m };
    let event = windup(&runtime, &actor.entity_id, target);
    let leap = crate::world_v3::actor_profile(clockworks_roster::HOUND).unwrap().ordinary.as_ref().unwrap().leap.as_ref().unwrap();
    assert_eq!(event.attack_kind.as_deref(), Some("leap"));
    assert_eq!(event.direction_rad, std::f32::consts::FRAC_PI_2);
    assert_eq!(event.radius_m, leap.hit_radius_m);
    assert_eq!(event.range_m, Some(leap.speed_mps * leap.active_ms as f32 / 1000.0));
}

#[test]
fn contact_blocked_and_expired_shots_never_publish_trailing_terminal_motion() {
    for mode in ["contact", "blocked", "expired"] {
        let (runtime, id, player) = drone(); windup(&runtime, &id, player);
        let target = if mode == "contact" { player } else { Vec3 { x_m: player.x_m + 6.0, ..player } };
        if mode == "blocked" {
            // A real authoritative obstacle added after aim commit is a unit fixture,
            // not a campaign geometry or route modification.
            let mut state = runtime.state.lock().unwrap();
            state.kcc.walls.push(crate::continuous_kcc::Aabb::new(13.0, 15.0, 7.0, 7.2).unwrap());
        }
        let mut found = false;
        for _ in 0..240 {
            let batch = tick_actor(&runtime, &id, target);
            if batch.iter().any(|event| event.kind == "EnemyAttackImpact") {
                assert!(!batch.iter().any(|event| event.kind == "PressureShotMotion"), "{mode}");
                let state = runtime.state.lock().unwrap();
                assert_eq!(state.world.generic_actors.iter().find(|a| a.entity_id == id).unwrap().state, ActorAiState::Cooldown);
                found = true; break;
            }
        }
        assert!(found, "{mode}");
    }
}

#[test]
fn restored_windup_and_active_motion_are_republished_once_in_new_epoch_without_timer_mutation() {
    for phase in ["windup", "active"] {
        let (runtime, id, player) = drone(); windup(&runtime, &id, player);
        if phase == "active" {
            for _ in 0..100 {
                tick_actor(&runtime, &id, Vec3 { x_m: player.x_m + 6.0, ..player });
                if runtime.state.lock().unwrap().world.generic_actors.iter().find(|a| a.entity_id == id).unwrap().state == ActorAiState::Active { break; }
            }
            tick_actor(&runtime, &id, Vec3 { x_m: player.x_m + 6.0, ..player });
        }
        let before = runtime.state.lock().unwrap().world.generic_actors.clone();
        runtime.save().unwrap();
        let bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        let first_epoch = runtime.snapshot().unwrap().world_epoch;
        let view = runtime.continue_saved().unwrap();
        assert!(view.entry_token.is_some()); assert!(view.world_epoch > first_epoch);
        assert_eq!(runtime.state.lock().unwrap().world.generic_actors, before);
        let events = runtime.state.lock().unwrap().presentation_events.clone();
        let restored: Vec<_> = events.iter().filter(|event| event.actor_id.as_deref() == Some(id.as_str())).collect();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].kind, if phase == "windup" { "EnemyAttackTelegraph" } else { "PressureShotMotion" });
        assert_eq!(restored[0].world_epoch, view.world_epoch);
        assert!(restored[0].duration_ms.unwrap() > 0);
        acknowledge_ready(&runtime, view);
        assert_eq!(runtime.state.lock().unwrap().presentation_events, events);
        assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), bytes);
    }
}

#[test]
fn malformed_actor_epoch_and_death_ownership_cannot_publish_live_cues() {
    let (runtime, id, player) = drone(); let event = windup(&runtime, &id, player);
    let mut state = runtime.state.lock().unwrap();
    let mut scene = runtime.scene_runtime.lock().unwrap().clone().unwrap();
    let forged = ActorRuntimeEvent::EnemyCue { actor_id: "unknown".into(), kind: ActorCueKind::Alert, origin_m: player, duration_ms: 10 };
    let before = state.presentation_events.clone(); present_events(&mut state, Some(&scene), &[forged]); assert_eq!(state.presentation_events, before);
    scene.world_epoch += 1;
    let valid = ActorRuntimeEvent::EnemyCue { actor_id: id.clone(), kind: ActorCueKind::Alert, origin_m: player, duration_ms: 10 };
    present_events(&mut state, Some(&scene), &[valid.clone()]); assert_eq!(state.presentation_events, before);
    scene.world_epoch -= 1;
    let actor = state.world.generic_actors.iter_mut().find(|a| a.entity_id == id).unwrap();
    let death = actor.take_damage_with_stagger(u32::MAX, 0);
    present_events(&mut state, Some(&scene), &death);
    assert_eq!(state.presentation_events.last().unwrap().kind, "EnemyDeath");
    let count = state.presentation_events.len(); present_events(&mut state, Some(&scene), &[valid]); assert_eq!(state.presentation_events.len(), count);
    assert_eq!(state.presentation_events.last().unwrap().actor_id, event.actor_id);
}

#[test]
fn paused_phase_resume_republishes_once_and_rejects_stale_context_without_mutation() {
    for phase in ["windup", "active"] {
        let (runtime, id, player) = drone(); windup(&runtime, &id, player);
        if phase == "active" {
            for _ in 0..100 {
                tick_actor(&runtime, &id, Vec3 { x_m: player.x_m + 6.0, ..player });
                if runtime.state.lock().unwrap().world.generic_actors.iter().find(|a| a.entity_id == id).unwrap().state == ActorAiState::Active { break; }
            }
            tick_actor(&runtime, &id, Vec3 { x_m: player.x_m + 6.0, ..player });
        }
        runtime.resume().unwrap();
        let view = runtime.snapshot().unwrap();
        let context = SessionContext { world_id: view.world_id.clone(), scene_id: view.scene_id.clone(), world_epoch: view.world_epoch };
        runtime.pause_context_ordered(&context, 1).unwrap();
        let actors = runtime.state.lock().unwrap().world.generic_actors.clone();
        let last = runtime.state.lock().unwrap().next_presentation_event_id - 1;
        let mut stale = context.clone(); stale.world_epoch += 1;
        assert_eq!(runtime.resume_context_ordered(&stale, 2).unwrap_err(), "E_LIFECYCLE_STALE_CONTEXT");
        assert!(runtime.presentation_events_since(view.world_epoch, last).unwrap().is_empty());
        runtime.resume_context_ordered(&context, 2).unwrap();
        let fresh = runtime.presentation_events_since(view.world_epoch, last).unwrap();
        assert_eq!(fresh.len(), 1);
        assert_eq!(fresh[0].kind, if phase == "windup" { "EnemyAttackTelegraph" } else { "PressureShotMotion" });
        assert!(fresh[0].duration_ms.unwrap() > 0);
        assert_eq!(runtime.state.lock().unwrap().world.generic_actors, actors);
        runtime.resume_context_ordered(&context, 3).unwrap();
        assert_eq!(runtime.presentation_events_since(view.world_epoch, last).unwrap(), fresh);
        assert_eq!(runtime.state.lock().unwrap().world.generic_actors, actors);
    }
}

#[test]
fn ordered_pause_refresh_holds_one_tick_phase_and_fences_old_resume() {
    for phase in ["windup", "active"] {
        let (runtime, id, player) = drone(); windup(&runtime, &id, player);
        if phase == "active" {
            for _ in 0..100 {
                tick_actor(&runtime, &id, Vec3 { x_m: player.x_m + 6.0, ..player });
                if runtime.state.lock().unwrap().world.generic_actors.iter().find(|a| a.entity_id == id).unwrap().state == ActorAiState::Active { break; }
            }
        }
        {
            let mut state = runtime.state.lock().unwrap();
            let actor = state.world.generic_actors.iter_mut().find(|a| a.entity_id == id).unwrap();
            if let Some(active) = actor.ordinary.as_mut().unwrap().active.as_mut() {
                active.elapsed_ms += actor.state_remaining_ms - 1;
            }
            actor.state_remaining_ms = 1;
            assert!(actor.validate());
        }
        let view = runtime.snapshot().unwrap();
        let context = SessionContext { world_id: view.world_id.clone(), scene_id: view.scene_id.clone(), world_epoch: view.world_epoch };
        let before_actors = runtime.state.lock().unwrap().world.generic_actors.clone();
        let last = runtime.state.lock().unwrap().next_presentation_event_id - 1;
        runtime.pause_context_ordered(&context, 2).unwrap();
        assert!(runtime.state.lock().unwrap().paused);
        let cues = runtime.presentation_events_since(view.world_epoch, last).unwrap();
        assert_eq!(cues.len(), 1); assert_eq!(cues[0].duration_ms, Some(17));
        assert_eq!(runtime.state.lock().unwrap().world.generic_actors, before_actors);
        let held = runtime.snapshot().unwrap();
        assert_eq!(runtime.resume_context_ordered(&context, 1).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert_eq!(runtime.pause_context_ordered(&context, 2).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert_eq!(runtime.snapshot().unwrap(), held);
        assert_eq!(runtime.presentation_events_since(view.world_epoch, last).unwrap(), cues);
        runtime.resume_context_ordered(&context, 3).unwrap();
        assert_eq!(runtime.state.lock().unwrap().world.generic_actors, before_actors);
        assert_eq!(runtime.snapshot().unwrap().player, held.player);
    }
}

#[test]
fn zero_elapsed_active_restore_and_hold_keep_nonzero_committed_yaw() {
    let (runtime, id, base_player) = drone();
    let player = Vec3 { x_m: base_player.x_m + 2.0, ..base_player };
    windup(&runtime, &id, player);
    for _ in 0..100 {
        tick_actor(&runtime, &id, player);
        if runtime.state.lock().unwrap().world.generic_actors.iter().find(|a| a.entity_id == id).unwrap().state == ActorAiState::Active { break; }
    }
    let before = runtime.state.lock().unwrap().world.generic_actors.clone();
    let active = before.iter().find(|a| a.entity_id == id).unwrap().ordinary.as_ref().unwrap().active.as_ref().unwrap();
    assert_eq!(active.elapsed_ms, 0);
    let yaw = active.direction[0].atan2(active.direction[1]); assert!(yaw > 0.1);
    runtime.save().unwrap();
    let view = runtime.continue_saved().unwrap();
    let cues = runtime.presentation_events_since(view.world_epoch, 0).unwrap();
    assert_eq!(cues.len(), 1); assert_eq!(cues[0].direction_rad, yaw);
    assert_eq!(cues[0].position_m, active.origin_m);
    acknowledge_ready(&runtime, view.clone());
    let context = SessionContext { world_id: view.world_id, scene_id: view.scene_id, world_epoch: view.world_epoch };
    runtime.pause_context_ordered(&context, 1).unwrap();
    let refreshed = runtime.presentation_events_since(context.world_epoch, cues[0].event_id).unwrap();
    assert_eq!(refreshed.len(), 1); assert_eq!(refreshed[0].direction_rad, yaw);
    assert_eq!(runtime.state.lock().unwrap().world.generic_actors, before);
}
