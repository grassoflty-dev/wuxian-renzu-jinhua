//! Admission/transaction boundary tests with explicit arranged authority.
//! The separate genuine campaign scenario must earn every objective/combat win.
use super::*;
use super::tests::fixture;
use crate::formal_runtime::entry_test_support::acknowledge_ready;

fn install_scene(runtime: &FormalRuntime, id: &str) {
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    runtime.install_scene_registry(registry, id).unwrap();
    runtime.state.lock().unwrap().paused = false;
}
fn at(runtime: &FormalRuntime, x: f32, z: f32) {
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(x, 0.0, z).unwrap();
}
fn extract_mh(runtime: &FormalRuntime) {
    let mut state = runtime.state.lock().unwrap();
    let mh = state.route.progress.iter_mut().find(|p| p.world_id == "mist_harbor").unwrap();
    mh.completed = true; mh.first_completion = true; mh.visit_id = 1;
    mh.completed_events = ["mist_beacon_west", "mist_beacon_east", "mist_signal"].into_iter().map(str::to_owned).collect();
    state.world_persistent_v1.mist_harbor.warden_defeated = true;
}
fn gate(runtime: &FormalRuntime, id: &str, request: &str) -> WorldView {
    let view = runtime.snapshot().unwrap();
    let prepared = runtime.use_world_gate(id, request, view.world_epoch).unwrap();
    assert!(prepared.entry_token.is_some());
    assert!(runtime.state.lock().unwrap().paused);
    acknowledge_ready(runtime, prepared)
}
fn finish_and_return(runtime: &FormalRuntime) {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    assert!(runtime.activate_scene_interaction(SHUTDOWN_ID, "first-settlement", epoch).unwrap().applied);
    at(runtime, 20.0, 8.0);
    gate(runtime, "cw_shutdown_return_to_rs", "first-return");
}
fn traverse_completed_route(runtime: &FormalRuntime, visit: u32) {
    let route = ["cw_entry_foundry", "cw_pressure_hall", "cw_conveyor_bridge", "cw_boiler_chamber",
        "cw_gear_shaft", "cw_furnace_heart", "cw_forged_guard_arena", "cw_regulator_core", "cw_shutdown_exit"];
    for (index, pair) in route.windows(2).enumerate() {
        let definition = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene().clone();
        assert_eq!(definition.scene_id, pair[0]);
        let transition = definition.transitions.iter().find(|item| item.to_scene_id == pair[1]).unwrap();
        let (x,z) = polygon_interior(&transition.polygon).unwrap();
        at(runtime, x, z);
        // This test arranges transaction preconditions, not a traversal proof.
        // The authored upper exit must be approached on its actual deck plane.
        if let Some(height) = transition.height_range_m {
            let mut state=runtime.state.lock().unwrap();
            state.world.player.position_m.y_m=height.midpoint();
            assert!(state.kcc.valid_saved_support_contact(&state.world.player,state.world.server_time_ms));
        }
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let prepared = runtime.transition_scene(&transition.id, &format!("visit-{visit}-step-{index}"), epoch).unwrap();
        acknowledge_ready(runtime, prepared);
    }
}

#[test]
fn native_startup_uses_strict_campaign_but_hides_clockworks_before_actual_extraction() {
    let runtime = fixture();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    assert!(runtime.scene_registry.lock().unwrap().as_ref().unwrap().is_complete_clockworks_production());
    at(&runtime, 20.0, 4.0);
    let before = runtime.snapshot().unwrap();
    assert!(!before.interactables.iter().any(|i| i.entity_id == "rs_cw_world_gate_marker"));
    assert_eq!(runtime.use_world_gate("rs_world_gate_to_cw", "too-early", before.world_epoch).unwrap_err(), "E_WORLD_GATE_MH_EXTRACTION_REQUIRED");
    {
        let mut state = runtime.state.lock().unwrap();
        let mh = state.route.progress.iter_mut().find(|p| p.world_id == "mist_harbor").unwrap();
        mh.completed_events = ["mist_beacon_west", "mist_beacon_east", "mist_signal"].into_iter().map(str::to_owned).collect();
        state.world_persistent_v1.mist_harbor.warden_defeated = true;
    }
    assert_eq!(runtime.use_world_gate("rs_world_gate_to_cw", "too-early", before.world_epoch).unwrap_err(), "E_WORLD_GATE_MH_EXTRACTION_REQUIRED");
    extract_mh(&runtime);
    assert!(runtime.snapshot().unwrap().interactables.iter().any(|i| i.entity_id == "rs_cw_world_gate_marker" && i.kind == "world_gate" && i.active));
    let entered = gate(&runtime, "rs_world_gate_to_cw", "too-early");
    assert_eq!(entered.scene_id, "cw_entry_foundry");
    assert_eq!(entered.world_epoch, before.world_epoch + 1);
}

#[test]
fn shutdown_return_is_completion_gated_and_save_continue_preserves_one_settlement() {
    let runtime = fixture();
    extract_mh(&runtime);
    at(&runtime, 20.0, 8.0);
    let before = runtime.snapshot().unwrap();
    assert!(!before.interactables.iter().any(|i| i.entity_id == "cw_shutdown_return_to_rs"));
    assert_eq!(runtime.use_world_gate("cw_shutdown_return_to_rs", "unsettled", before.world_epoch).unwrap_err(), "E_CLOCKWORKS_SETTLEMENT_REQUIRED");
    at(&runtime, 8.0, 6.0);
    let response = runtime.activate_scene_interaction(SHUTDOWN_ID, "shutdown", before.world_epoch).unwrap();
    assert!(response.applied);
    assert!(response.view.interactables.iter().any(|i| i.entity_id == "cw_shutdown_return_to_rs" && i.active));
    let seq = response.view.progression.event_seq;
    at(&runtime, 20.0, 8.0);
    let returned = gate(&runtime, "cw_shutdown_return_to_rs", "unsettled");
    assert_eq!(returned.scene_id, "rs_core_room");
    assert_eq!(returned.progression.event_seq, seq);
    runtime.save().unwrap();
    let bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
    let restored = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(restored.scene_id, "rs_core_room");
    assert!(completed(&runtime.state.lock().unwrap()));
    assert!(has_one_air_step_source(&runtime.state.lock().unwrap()));
    assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), bytes);
}

#[test]
fn four_real_revisit_gate_installs_and_duplicate_request_never_installs_a_fifth() {
    let runtime = fixture(); extract_mh(&runtime);
    runtime.state.lock().unwrap().world_persistent_v1.clockworks.forged_guard_elite_first_kill = true;
    finish_and_return(&runtime);
    for count in 1..=4 {
        at(&runtime, 20.0, 4.0);
        let entered = gate(&runtime, "rs_world_gate_to_cw", &format!("cw-revisit-{count}"));
        let cw = entered.progression.worlds.iter().find(|p| p.world_id == "clockworks").unwrap();
        assert_eq!(cw.visit_id, u64::from(count) + 1); assert_eq!(cw.revisit_count, count);
        traverse_completed_route(&runtime, count);
        assert!(completed(&runtime.state.lock().unwrap()));
        assert!(has_one_air_step_source(&runtime.state.lock().unwrap()));
        at(&runtime, 20.0, 8.0);
        gate(&runtime, "cw_shutdown_return_to_rs", &format!("return-{count}"));
        at(&runtime, 20.0, 4.0);
        let before = runtime.snapshot().unwrap();
        assert!(runtime.use_world_gate("rs_world_gate_to_cw", &format!("cw-revisit-{count}"), before.world_epoch).is_err());
        assert_eq!(runtime.snapshot().unwrap(), before, "duplicate at fresh epoch cannot install or reset controls");
    }
    let before = runtime.snapshot().unwrap();
    assert!(!before.interactables.iter().any(|i| i.entity_id == "rs_cw_world_gate_marker"));
    assert_eq!(runtime.use_world_gate("rs_world_gate_to_cw", "fifth", before.world_epoch).unwrap_err(), "E_WORLD_GATE_UNAVAILABLE");
    assert_eq!(runtime.snapshot().unwrap(), before);
}

#[test]
fn epoch_bound_interaction_rejects_late_same_scene_continue_return_and_new() {
    let runtime = fixture();
    let first = runtime.snapshot().unwrap();
    assert_eq!(runtime.interact(SHUTDOWN_ID, "missing-epoch").unwrap_err(), "E_INTERACTION_SOURCE_EPOCH_REQUIRED");
    runtime.save().unwrap();
    let restored = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(restored.scene_id, first.scene_id); assert!(restored.world_epoch > first.world_epoch);
    let before = runtime.snapshot().unwrap();
    let stale = runtime.activate_scene_interaction(SHUTDOWN_ID, "delayed-same-scene", first.world_epoch).unwrap();
    assert!(!stale.applied); assert_eq!(stale.error_code.as_deref(), Some("E_SCENE_RUNTIME_StaleEpoch"));
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert!(runtime.activate_scene_interaction(SHUTDOWN_ID, "delayed-same-scene", before.world_epoch).unwrap().applied);
    at(&runtime, 20.0, 8.0);
    gate(&runtime, "cw_shutdown_return_to_rs", "epoch-return");
    let at_rs = runtime.snapshot().unwrap();
    assert_eq!(runtime.activate_scene_interaction(SHUTDOWN_ID, "delayed-return", before.world_epoch).unwrap().error_code.as_deref(), Some("E_SCENE_RUNTIME_StaleEpoch"));
    assert_eq!(runtime.snapshot().unwrap(), at_rs);
    acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    let fresh = runtime.snapshot().unwrap();
    assert_eq!(runtime.activate_scene_interaction(SHUTDOWN_ID, "delayed-new", first.world_epoch).unwrap().error_code.as_deref(), Some("E_SCENE_RUNTIME_StaleEpoch"));
    assert_eq!(runtime.snapshot().unwrap(), fresh);
}

#[test]
fn rejected_cw_gate_range_epoch_and_late_prepare_leave_the_request_reusable() {
    for invalid in ["range", "non-finite", "stale", "generation"] {
        let runtime = fixture(); extract_mh(&runtime); install_scene(&runtime, "rs_core_room");
        at(&runtime, if invalid == "range" { 2.0 } else { 20.0 }, 4.0);
        if invalid == "non-finite" { runtime.state.lock().unwrap().world.player.position_m.x_m = f32::NAN; }
        if invalid == "generation" { runtime.state.lock().unwrap().entry_generation = build_ui::MAX_SAFE_REVISION; }
        let before = runtime.snapshot().unwrap();
        let epoch = before.world_epoch + u64::from(invalid == "stale");
        assert!(runtime.use_world_gate("rs_world_gate_to_cw", "retry-gate", epoch).is_err());
        assert_eq!(serde_json::to_value(runtime.snapshot().unwrap()).unwrap(), serde_json::to_value(before).unwrap(), "{invalid}");
        runtime.state.lock().unwrap().entry_generation = 0;
        at(&runtime, 20.0, 4.0);
        assert_eq!(gate(&runtime, "rs_world_gate_to_cw", "retry-gate").scene_id, "cw_entry_foundry");
    }
}

#[test]
fn interaction_transport_receipt_echoes_request_with_one_atomic_snapshot() {
    let runtime = fixture(); let epoch = runtime.snapshot().unwrap().world_epoch;
    let rejected = crate::scene_route_commands::interaction(&runtime, SHUTDOWN_ID, "transport-stale", Some(epoch + 1)).unwrap();
    assert!(!rejected.applied);
    assert_eq!(rejected.receipt.command_id, "transport-stale");
    assert_eq!(rejected.receipt.snapshot, crate::world_v3::WorldSnapshot::from_view(rejected.view));
    assert_eq!(rejected.receipt.error_code, rejected.error_code);
    let applied = crate::scene_route_commands::interaction(&runtime, SHUTDOWN_ID, "transport-valid", Some(epoch)).unwrap();
    assert!(applied.applied);
    assert_eq!(applied.receipt.command_id, "transport-valid");
    assert_eq!(applied.receipt.snapshot, crate::world_v3::WorldSnapshot::from_view(applied.view));
    assert!(applied.receipt.snapshot.progression.worlds.iter().any(|p| p.world_id == CW && p.completed));
}

#[test]
fn native_mh_route_commands_cannot_forge_extraction_for_clockworks_admission() {
    let runtime = fixture(); install_scene(&runtime, "mh_extraction");
    {
        let mut state = runtime.state.lock().unwrap();
        state.route.current_world_id = "mist_harbor".into();
        state.world_persistent_v1.mist_harbor.warden_defeated = true;
    }
    for request in [
        RouteCommandRequest::Progress { event_id: "mist_beacon_west".into(), request_id: "forge-beacon".into() },
        RouteCommandRequest::Progress { event_id: "mist_signal".into(), request_id: "forge-signal".into() },
        RouteCommandRequest::Complete { world_id: "mist_harbor".into(), request_id: "forge-extraction".into() },
        RouteCommandRequest::Enter { world_id: "mist_harbor".into(), request_id: "bypass-visit".into() },
        RouteCommandRequest::Revisit { world_id: "mist_harbor".into(), request_id: "bypass-revisit".into() },
    ] {
        let before = runtime.snapshot().unwrap();
        assert_eq!(runtime.apply_route_command(request).unwrap_err(), "E_MIST_HARBOR_AUTHORED_COMMAND_REQUIRED");
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert!(!mist_harbor_extracted(&runtime.state.lock().unwrap()));
    }
}

#[test]
fn mh_extraction_request_collision_cannot_install_a_return_without_completion() {
    let runtime = fixture(); install_scene(&runtime, "mh_extraction"); extract_mh(&runtime);
    {
        let mut state = runtime.state.lock().unwrap();
        state.route.current_world_id = "mist_harbor".into();
        let mh = state.route.progress.iter_mut().find(|p| p.world_id == "mist_harbor").unwrap();
        mh.completed = false; mh.first_completion = false;
        let revision = state.world.revision;
        apply_route_command(&mut state.route, RouteCommand::Progress {
            event_id: "mist_signal".into(), request_id: "world-gate-complete-collision".into(),
        }, revision);
    }
    let position = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene()
        .interactions.iter().find(|i| i.id == "mh_extraction_exit").unwrap().position;
    at(&runtime, position[0], position[2]);
    let before = runtime.snapshot().unwrap();
    assert_eq!(runtime.use_world_gate("mh_extraction_return_to_rs", "collision", before.world_epoch).unwrap_err(),
        "E_WORLD_GATE_COMPLETE_REJECTED");
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert!(!mist_harbor_extracted(&runtime.state.lock().unwrap()));
    let returned = gate(&runtime, "mh_extraction_return_to_rs", "actual-extraction");
    assert_eq!(returned.world_id, "return_station");
    assert!(mist_harbor_extracted(&runtime.state.lock().unwrap()));
}

#[test]
fn native_legacy_clockworks_slot_keeps_incomplete_authority_until_physical_reconfirmation() {
    let runtime = super::tests::legacy_fixture(true);
    runtime.save_slot("legacy-cw", "Legacy Clockworks", true).unwrap();
    let path = runtime.save_root.join("slots/legacy-cw/slot-v6.json");
    let bytes = std::fs::read(&path).unwrap();
    let old = runtime.snapshot().unwrap();
    acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    let prepared = runtime.continue_slot("legacy-cw").unwrap();
    assert!(prepared.entry_token.is_some()); assert_eq!(prepared.scene_id, "cw_shutdown_exit");
    assert!(prepared.world_epoch > old.world_epoch);
    assert!(!completed(&runtime.state.lock().unwrap()));
    let restored = acknowledge_ready(&runtime, prepared);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(runtime.state.lock().unwrap().world_persistent_v1.clockworks.pressure_valve_ids.is_empty());
    assert!(!runtime.state.lock().unwrap().world_persistent_v1.clockworks.core_console_confirmed);
    assert!(runtime.activate_scene_interaction(SHUTDOWN_ID, "slot-reconfirm", restored.world_epoch).unwrap().applied);
    assert!(completed(&runtime.state.lock().unwrap())); assert!(has_one_air_step_source(&runtime.state.lock().unwrap()));
    assert_eq!(std::fs::read(path).unwrap(), bytes, "confirmation never rewrites a selected old slot");
}
