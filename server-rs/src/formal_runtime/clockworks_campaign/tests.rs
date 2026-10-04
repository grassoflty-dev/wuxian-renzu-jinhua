//! Bounded transaction tests intentionally arrange prerequisite authority.
//! They are not evidence of a genuine campaign route or combat victory.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn fixture() -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("cw-settlement-{}-{}", std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.pause().unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    let manifest: serde_json::Value = serde_json::from_str(include_str!("../../../../governance/assets/RUNTIME_ASSET_MANIFEST.json")).unwrap();
    let entity_catalog: serde_json::Value = serde_json::from_str(include_str!("../../../../content/enemies/entity-types.json")).unwrap();
    let assets = manifest["assets"].as_array().unwrap().iter().filter(|row| row["admission"] == "release_approved")
        .map(|row| row["assetId"].as_str().unwrap().to_owned()).collect();
    let entities = entity_catalog["entityTypes"].as_array().unwrap().iter()
        .map(|row| row.as_str().unwrap().to_owned()).collect();
    let registry = WorldRegistry::load_complete_clockworks_production(
        crate::production_scene_bootstrap::embedded_scene_json().unwrap().iter().copied(), &assets, &entities,
    ).unwrap();
    runtime.install_scene_registry(registry, "cw_shutdown_exit").unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        state.paused = false;
        state.route.current_world_id = CW.into();
        let progress = state.route.progress.iter_mut().find(|p| p.world_id == CW).unwrap();
        progress.visit_id = 1;
        progress.completed_events = vec!["clockworks_valves".into(), "clockworks_core".into()];
        state.world.player.position_m = Vec3::new(8.0, 0.0, 6.0).unwrap();
        state.world_persistent_v1.clockworks.regulator_defeated = true;
        state.world_persistent_v1.clockworks.core_console_confirmed = true;
        state.world_persistent_v1.clockworks.pressure_valve_ids =
            crate::world_persistent_v1::CLOCKWORKS_PRESSURE_VALVE_IDS.iter().map(|id| (*id).to_owned()).collect();
    }
    runtime
}

pub(super) fn legacy_fixture(activated: bool) -> FormalRuntime {
    let runtime = fixture();
    {
        let mut state = runtime.state.lock().unwrap();
        state.route.progress.iter_mut().find(|p| p.world_id == CW).unwrap().completed_events.push(SHUTDOWN_EVENT.into());
        state.world_persistent_v1.clockworks.core_console_confirmed = false;
        state.world_persistent_v1.clockworks.pressure_valve_ids.clear();
        let mut capabilities = state.capabilities.clone();
        apply_command_at_revision(&mut capabilities, CapabilityCommand::Grant { capability_id: CAP_AIR_STEP.into() }, state.world.revision).unwrap();
        let world = state.world.clone();
        state.install_capability_state(world, capabilities).unwrap();
    }
    if activated {
        let mut guard = runtime.scene_runtime.lock().unwrap();
        let scene = guard.as_mut().unwrap();
        scene.interact(SHUTDOWN_ID, "old-shutdown", scene.world_epoch, Vec3::new(8.0, 0.0, 6.0).unwrap()).unwrap();
    }
    runtime
}

fn durable(runtime: &FormalRuntime) -> serde_json::Value {
    let state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    serde_json::to_value(crate::save_v6::SaveV6::capture(&state, scene.as_ref()).unwrap()).unwrap()
}

fn shutdown(runtime: &FormalRuntime, request: &str) -> FormalInteractionResponse {
    runtime.activate_scene_interaction(SHUTDOWN_ID, request, runtime.snapshot().unwrap().world_epoch).unwrap()
}

#[test]
fn first_shutdown_commits_completion_and_exactly_one_air_step_source() {
    let runtime = fixture();
    assert!(!completed(&runtime.state.lock().unwrap()));
    let before_seq = runtime.state.lock().unwrap().route.event_seq;
    let receipt = shutdown(&runtime, "first-shutdown");
    assert!(receipt.applied, "{:?}", receipt.error_code);
    let state = runtime.state.lock().unwrap();
    assert!(completed(&state));
    assert!(has_one_air_step_source(&state));
    assert_eq!(state.route.event_seq, before_seq + 2);
    assert!(state.capabilities.selected.iter().any(|id| id == CAP_AIR_STEP));
    drop(state);
    let after = durable(&runtime);
    for request in ["first-shutdown", "another-shutdown"] {
        assert!(!shutdown(&runtime, request).applied);
        assert_eq!(durable(&runtime), after);
    }
    runtime.save().unwrap();
    let view = runtime.continue_saved().unwrap();
    assert!(view.entry_token.is_some());
    crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view);
    assert!(completed(&runtime.state.lock().unwrap()));
}

#[test]
fn legacy_save_requires_physical_confirmation_without_regrant_or_backfilled_proof() {
    for activated in [false, true] {
        let runtime = legacy_fixture(activated);
        runtime.save().unwrap();
        let before_bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        let before_grants = runtime.state.lock().unwrap().capabilities.grants.clone();
        let before_sources = runtime.state.lock().unwrap().effect_sources_v6.clone();
        let before_view = runtime.continue_saved().unwrap();
        assert!(!completed(&runtime.state.lock().unwrap()));
        assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), before_bytes);
        crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, before_view);
        // Continue rebases revision ownership but retains the exact acquired rows.
        let rebased_grants = runtime.state.lock().unwrap().capabilities.grants.clone();
        assert_eq!(rebased_grants.len(), before_grants.len());
        assert!(runtime.snapshot().unwrap().interactables.iter().any(|item| item.entity_id == SHUTDOWN_ID && item.active));
        let before_seq = runtime.state.lock().unwrap().route.event_seq;
        let response = shutdown(&runtime, "confirm-old-shutdown");
        assert!(response.applied, "{:?}", response.error_code);
        let state = runtime.state.lock().unwrap();
        assert!(completed(&state));
        assert_eq!(state.capabilities.grants, rebased_grants);
        assert_eq!(state.effect_sources_v6, before_sources);
        assert!(state.world_persistent_v1.clockworks.pressure_valve_ids.is_empty());
        assert!(!state.world_persistent_v1.clockworks.core_console_confirmed);
        assert_eq!(state.route.event_seq, before_seq + 1);
    }
}

#[test]
fn missing_defeat_objective_proof_or_bound_source_rejects_without_mutation() {
    for invalid in ["defeat", "valve", "core", "legacy-source"] {
        let runtime = if invalid == "legacy-source" { legacy_fixture(true) } else { fixture() };
        {
            let mut state = runtime.state.lock().unwrap();
            match invalid {
                "defeat" => state.world_persistent_v1.clockworks.regulator_defeated = false,
                "valve" => { state.world_persistent_v1.clockworks.pressure_valve_ids.pop(); }
                "core" => state.world_persistent_v1.clockworks.core_console_confirmed = false,
                "legacy-source" => state.effect_sources_v6.retain(|source| source.source_id != CAP_AIR_STEP),
                _ => unreachable!(),
            }
        }
        let before = runtime.snapshot().unwrap();
        let route = runtime.state.lock().unwrap().route.clone();
        let response = shutdown(&runtime, "rejected-shutdown");
        assert!(!response.applied, "{invalid}");
        assert_eq!(serde_json::to_value(runtime.snapshot().unwrap()).unwrap(), serde_json::to_value(before).unwrap());
        assert_eq!(runtime.state.lock().unwrap().route, route);
    }
}

#[test]
fn stale_out_of_range_paused_and_dead_confirmation_cannot_consume_request() {
    for invalid in ["stale", "range", "non-finite", "paused", "dead"] {
        let runtime = legacy_fixture(true);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        {
            let mut state = runtime.state.lock().unwrap();
            match invalid {
                "range" => state.world.player.position_m = Vec3::new(20.0, 0.0, 8.0).unwrap(),
                "non-finite" => state.world.player.position_m.x_m = f32::NAN,
                "paused" => state.paused = true,
                "dead" => state.world.player_hp = 0,
                _ => {},
            }
        }
        let before = runtime.snapshot().unwrap();
        let result = runtime.activate_scene_interaction(SHUTDOWN_ID, "physical-confirm", if invalid == "stale" { epoch + 1 } else { epoch });
        assert!(!result.as_ref().is_ok_and(|response| response.applied), "{invalid}");
        if invalid == "non-finite" {
            assert_eq!(result.unwrap().error_code.as_deref(), Some("E_SCENE_RUNTIME_OutOfRange"));
        }
        assert_eq!(serde_json::to_value(runtime.snapshot().unwrap()).unwrap(), serde_json::to_value(before).unwrap());
        {
            let mut state = runtime.state.lock().unwrap();
            state.world.player.position_m = Vec3::new(8.0, 0.0, 6.0).unwrap();
            state.paused = false;
            state.world.player_hp = 100;
        }
        assert!(shutdown(&runtime, "physical-confirm").applied, "{invalid}");
    }
}

#[test]
fn late_revision_failure_rolls_back_completion_grant_and_scene_request() {
    let runtime = fixture();
    let revision = runtime.state.lock().unwrap().world.revision;
    runtime.state.lock().unwrap().world.revision.authority_revision = u64::MAX;
    let before = runtime.snapshot().unwrap();
    assert!(runtime.activate_scene_interaction(SHUTDOWN_ID, "retry-after-rollback", before.world_epoch).is_err());
    assert_eq!(serde_json::to_value(runtime.snapshot().unwrap()).unwrap(), serde_json::to_value(before).unwrap());
    assert!(!runtime.scene_runtime.lock().unwrap().as_ref().unwrap().object_activated(SHUTDOWN_ID));
    runtime.state.lock().unwrap().world.revision = revision;
    assert!(shutdown(&runtime, "retry-after-rollback").applied);
}

#[test]
fn eligibility_requires_extraction_and_revisit_limit_counts_only_real_acceptance() {
    let runtime = fixture();
    let mut state = runtime.state.lock().unwrap();
    assert!(!entry_eligible(&state));
    let mh = state.route.progress.iter_mut().find(|progress| progress.world_id == "mist_harbor").unwrap();
    mh.completed_events = vec!["mist_beacon_west".into(), "mist_beacon_east".into(), "mist_signal".into()];
    assert!(!entry_eligible(&state), "events alone are not extraction");
    state.world_persistent_v1.mist_harbor.warden_defeated = true;
    assert!(!entry_eligible(&state), "defeat is not extraction");
    let mh = state.route.progress.iter_mut().find(|progress| progress.world_id == "mist_harbor").unwrap();
    mh.completed = true; mh.first_completion = true;
    assert!(entry_eligible(&state));
    state.route = prepare_entry_route(&state, "cw-enter").unwrap();
    let before = state.route.clone();
    state.world.revision.world_epoch += 1;
    assert_eq!(prepare_entry_route(&state, "cw-enter").unwrap_err(), "E_CLOCKWORKS_ENTRY_ROUTE_REJECTED");
    assert_eq!(state.route, before);
    let cw = state.route.progress.iter_mut().find(|progress| progress.world_id == CW).unwrap();
    cw.completed = true; cw.first_completion = true;
    for count in 1..=4 {
        state.route = prepare_entry_route(&state, &format!("revisit-{count}")).unwrap();
        let cw = state.route.progress.iter().find(|progress| progress.world_id == CW).unwrap();
        assert_eq!(cw.revisit_count, count); assert_eq!(cw.visit_id, u64::from(count) + 1);
        let before = state.route.clone();
        state.world.revision.world_epoch += 1;
        assert!(prepare_entry_route(&state, &format!("revisit-{count}")).is_err());
        assert_eq!(state.route, before);
    }
    assert!(!entry_eligible(&state));
    assert_eq!(prepare_entry_route(&state, "fifth-revisit").unwrap_err(), "E_WORLD_GATE_UNAVAILABLE");
}

#[test]
fn generic_native_commands_cannot_bypass_clockworks_transactions() {
    let runtime = fixture();
    for request in [
        RouteCommandRequest::Progress { event_id: "clockworks_valves".into(), request_id: "generic-valves".into() },
        RouteCommandRequest::Progress { event_id: SHUTDOWN_EVENT.into(), request_id: "generic-shutdown".into() },
        RouteCommandRequest::Complete { world_id: CW.into(), request_id: "generic-complete".into() },
        RouteCommandRequest::Enter { world_id: CW.into(), request_id: "generic-enter".into() },
        RouteCommandRequest::Revisit { world_id: CW.into(), request_id: "generic-revisit".into() },
    ] {
        let before = durable(&runtime);
        assert_eq!(runtime.apply_route_command(request).unwrap_err(), "E_CLOCKWORKS_AUTHORED_COMMAND_REQUIRED");
        assert_eq!(durable(&runtime), before);
    }
}

#[test]
fn shutdown_event_forgery_or_mixed_event_cannot_settle() {
    let runtime = fixture();
    let mut guard = runtime.scene_runtime.lock().unwrap();
    let mut candidate = guard.as_mut().unwrap().clone();
    let epoch = candidate.world_epoch;
    let events = candidate.interact(SHUTDOWN_ID, "real-candidate", epoch, Vec3::new(8.0, 0.0, 6.0).unwrap()).unwrap();
    let mut state = runtime.state.lock().unwrap();
    let before = state.route.clone();
    for forged in [
        vec![SceneEvent::Trigger { id: "forged-trigger".into(), event_id: SHUTDOWN_EVENT.into() }],
        vec![SceneEvent::Interaction { id: "forged-console".into(), event_id: Some(SHUTDOWN_EVENT.into()) }],
        vec![SceneEvent::Interaction { id: "benign-first".into(), event_id: None }, events[0].clone()],
    ] {
        assert!(apply_shutdown(&mut state, &candidate, &forged, "forged").is_err());
        assert_eq!(state.route, before);
        assert!(!state.capabilities.grants.iter().any(|grant| grant.capability_id == CAP_AIR_STEP));
    }
    assert!(apply_shutdown(&mut state, &candidate, &events, "real-candidate").unwrap());
    assert!(completed(&state));
}

#[test]
fn completion_request_collision_does_not_partially_commit_shutdown() {
    let runtime = fixture();
    {
        let mut state = runtime.state.lock().unwrap();
        let revision = state.world.revision;
        assert!(matches!(apply_route_command(&mut state.route, RouteCommand::Progress {
            event_id: "clockworks_core".into(), request_id: "scene-complete-collision".into(),
        }, revision), RouteResult::Applied { .. }));
    }
    let before = durable(&runtime);
    let response = shutdown(&runtime, "collision");
    assert!(!response.applied);
    assert_eq!(response.error_code.as_deref(), Some("E_CLOCKWORKS_SHUTDOWN_COMPLETE_REJECTED"));
    assert_eq!(durable(&runtime), before);
    assert!(!runtime.scene_runtime.lock().unwrap().as_ref().unwrap().object_activated(SHUTDOWN_ID));
    assert!(shutdown(&runtime, "distinct-request").applied);
}
