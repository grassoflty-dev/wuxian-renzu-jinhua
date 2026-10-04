//! Narrow gate transaction fixtures arrange prerequisite route authority. These
//! are not a replacement for the genuine campaign's earned-progress driver.
use super::*;
use crate::formal_runtime::entry_test_support::acknowledge_ready;

fn fixture(world: &str, completed: bool) -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("gate-dedup-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.pause().unwrap(); runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        if world == "mist_harbor" || completed {
            let gh = state.route.progress.iter_mut().find(|p| p.world_id == "grey_hive").unwrap();
            gh.completed = true; gh.first_completion = true; gh.visit_id = 1;
            gh.completed_events = ["hive_power", "hive_lockdown", "hive_extraction"].into_iter().map(str::to_owned).collect();
        }
        if world == "mist_harbor" && completed {
            let mh = state.route.progress.iter_mut().find(|p| p.world_id == world).unwrap();
            mh.completed = true; mh.first_completion = true; mh.visit_id = 1;
            mh.completed_events = ["mist_beacon_west", "mist_beacon_east", "mist_signal"].into_iter().map(str::to_owned).collect();
            state.world_persistent_v1.mist_harbor.warden_defeated = true;
        }
    }
    runtime.resume().unwrap(); at_entry(&runtime, world); runtime
}
fn gate_id(world: &str) -> &'static str {
    if world == "grey_hive" { "rs_world_gate_to_gh" } else { "rs_world_gate_to_mh" }
}
fn at_marker(runtime: &FormalRuntime, marker: &str) {
    let position = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene().interactions.iter()
        .find(|item| item.id == marker).unwrap().position;
    runtime.state.lock().unwrap().world.player.position_m = vec3_from_array(position);
}
fn at_entry(runtime: &FormalRuntime, world: &str) {
    at_marker(runtime, if world == "grey_hive" { "rs_world_gate_marker" } else { "rs_mh_world_gate_marker" });
}
fn durable(runtime: &FormalRuntime) -> serde_json::Value {
    serde_json::to_value(crate::save_v6::SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap()).unwrap()
}
fn fingerprint(runtime: &FormalRuntime) -> String {
    let state = runtime.state.lock().unwrap();
    format!("{:?}", (&state.world, &state.route, &state.capabilities, &state.latest_sample,
        &state.pending_combat, &state.presentation_events, &state.sound_cues,
        (state.next_presentation_event_id, state.next_sound_cue_id, state.last_received_seq,
        state.last_client_time_ms, &state.pending_entry, state.entry_generation, state.paused)))
}
fn accepted_entry(runtime: &FormalRuntime, world: &str, request: &str) -> WorldView {
    let before = runtime.snapshot().unwrap();
    let view = runtime.use_world_gate(gate_id(world), request, before.world_epoch).unwrap();
    assert!(view.entry_token.is_some()); assert!(runtime.state.lock().unwrap().paused);
    assert_eq!(view.world_epoch, before.world_epoch + 1);
    acknowledge_ready(runtime, view)
}
fn returned(runtime: &FormalRuntime, world: &str, request: &str) -> WorldView {
    // Place this boundary fixture at the actual authored extraction; the gate
    // itself is public and requires the arranged completion/defeat authority.
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    runtime.install_scene_registry(registry, if world == "grey_hive" { "gh_exit" } else { "mh_extraction" }).unwrap();
    at_marker(runtime, if world == "grey_hive" { "gh_exit_extraction_console" } else { "mh_extraction_exit" });
    let before = runtime.snapshot().unwrap();
    let view = runtime.use_world_gate(if world == "grey_hive" { "gh_extraction_return_to_rs" } else { "mh_extraction_return_to_rs" }, request, before.world_epoch).unwrap();
    let view = acknowledge_ready(runtime, view); assert_eq!(view.world_id, "return_station"); view
}

#[test]
fn collided_enter_acknowledgement_never_installs_a_destination_or_consumes_request() {
    for world in ["grey_hive", "mist_harbor"] {
        let runtime = fixture(world, false);
        {
            let mut state = runtime.state.lock().unwrap(); let revision = state.world.revision;
            assert!(matches!(apply_route_command(&mut state.route, RouteCommand::Progress {
                event_id: "hive_power".into(), request_id: "world-gate-enter-collision".into(),
            }, revision), RouteResult::Applied { .. }));
        }
        runtime.save().unwrap();
        let bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        let before = runtime.snapshot().unwrap(); let authority = durable(&runtime); let exact = fingerprint(&runtime);
        assert_eq!(runtime.use_world_gate(gate_id(world), "collision", before.world_epoch).unwrap_err(), "E_WORLD_GATE_ROUTE_NOT_APPLIED");
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(durable(&runtime), authority); assert_eq!(fingerprint(&runtime), exact);
        assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), bytes);
        assert_eq!(accepted_entry(&runtime, world, "fresh-entry").world_id, world);
    }
}

#[test]
fn repeat_revisit_after_real_return_rejects_at_fresh_epoch_without_free_visit() {
    for world in ["grey_hive", "mist_harbor"] {
        let runtime = fixture(world, true); runtime.save().unwrap();
        let bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        let entered = accepted_entry(&runtime, world, "first-revisit");
        let returned_view = returned(&runtime, world, "return-after-first");
        assert!(returned_view.world_epoch > entered.world_epoch);
        at_entry(&runtime, world);
        let before = runtime.snapshot().unwrap(); let authority = durable(&runtime); let exact = fingerprint(&runtime);
        let progress = before.progression.worlds.iter().find(|p| p.world_id == world).unwrap();
        assert_eq!(progress.revisit_count, 1); assert_eq!(progress.visit_id, 2);
        assert_eq!(runtime.use_world_gate(gate_id(world), "first-revisit", before.world_epoch).unwrap_err(), "E_WORLD_GATE_ROUTE_NOT_APPLIED");
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(durable(&runtime), authority); assert_eq!(fingerprint(&runtime), exact);
        assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), bytes);
        assert_eq!(runtime.use_world_gate(gate_id(world), "fresh-second", entered.world_epoch).unwrap_err(), "E_WORLD_GATE_STALE_EPOCH");
        assert_eq!(runtime.snapshot().unwrap(), before);
        let second = accepted_entry(&runtime, world, "fresh-second");
        let progress = second.progression.worlds.iter().find(|p| p.world_id == world).unwrap();
        assert_eq!(progress.revisit_count, 2); assert_eq!(progress.visit_id, 3);
        returned(&runtime, world, "return-after-second"); at_entry(&runtime, world);
        let exhausted = runtime.snapshot().unwrap();
        assert!(runtime.use_world_gate(gate_id(world), "beyond-limit", exhausted.world_epoch).is_err());
        assert_eq!(runtime.snapshot().unwrap(), exhausted);
    }
}
