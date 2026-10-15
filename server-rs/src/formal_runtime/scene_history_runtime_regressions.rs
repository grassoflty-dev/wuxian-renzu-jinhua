use super::{entry_test_support, FormalRuntime};
use crate::{
    production_scene_bootstrap,
    save_v6::SaveV6,
    save_v5::SceneStateSave,
    world_v3::{Vec3, WorldView},
};
use std::{
    path::PathBuf,
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

const NAMED_RETURN_SLOT: &str = "scene-history-return";

fn save_root(label: &str) -> PathBuf {
    let base = match std::env::var_os("WUXIAN_SCENE_HISTORY_RECONSTRUCTION_SAVES") {
        Some(base) => {
            let path = PathBuf::from(base);
            assert!(
                !path.as_os_str().is_empty(),
                "save override must not be empty"
            );
            path
        }
        None => std::env::temp_dir().join("scene-history-regression"),
    };
    base.join(format!(
        "{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after UNIX epoch")
            .as_nanos()
    ))
}

fn start_runtime(label: &str) -> (FormalRuntime, PathBuf, WorldView) {
    let root = save_root(label);
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.stop_owner.store(true, Ordering::Release);
    runtime
        .owner_handle
        .lock()
        .unwrap()
        .take()
        .expect("test runtime owns its simulation thread")
        .join()
        .expect("test runtime simulation thread exits");
    production_scene_bootstrap::install(&runtime).unwrap();
    let fresh = runtime
        .reset_new()
        .map(|view| entry_test_support::acknowledge_ready(&runtime, view))
        .unwrap();
    assert_eq!(fresh.world_id, "return_station");
    assert_eq!(fresh.scene_id, "rs_core_room");
    (runtime, root, fresh)
}

fn place_player(runtime: &FormalRuntime, x: f32, y: f32, z: f32) {
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(x, y, z).unwrap();
}

fn gh_progress(runtime: &FormalRuntime) -> crate::world_progression::WorldProgress {
    runtime
        .state
        .lock()
        .unwrap()
        .route
        .progress
        .iter()
        .find(|progress| progress.world_id == "grey_hive")
        .unwrap()
        .clone()
}

fn history_row<'a>(save: &'a SaveV6, scene_id: &str) -> &'a SceneStateSave {
    save.world_persistent_v1
        .scene_history_v1
        .as_ref()
        .expect("return save carries scene history")
        .scene_states
        .iter()
        .find(|row| row.scene_id == scene_id)
        .unwrap_or_else(|| panic!("missing history row for {scene_id}"))
}

fn assert_departed_rows(save: &SaveV6) {
    assert_eq!(save.save.scene_id, "rs_core_room");
    assert_eq!(save.save.scene_states.len(), 1, "SaveV5 remains active-scene-only");
    assert_eq!(save.save.scene_states[0].scene_id, "rs_core_room");

    let entry = history_row(save, "gh_entry_maintenance");
    assert_eq!(entry.activated_ids, vec!["gh_entry_tutorial_terminal"]);
    assert!(entry.emitted_event_ids.is_empty());
    assert_eq!(entry.checkpoint_id.as_deref(), Some("gh_entry_checkpoint"));

    let power = history_row(save, "gh_power_room");
    assert_eq!(power.activated_ids, vec!["gh_power_console"]);
    assert_eq!(power.emitted_event_ids, vec!["hive_power"]);
    assert_eq!(power.checkpoint_id.as_deref(), Some("gh_power_checkpoint"));
}

fn exercise_departure(runtime: &FormalRuntime, start: &WorldView, prefix: &str) {
    place_player(runtime, 20.0, 0.0, 8.0);
    let entered = runtime
        .use_world_gate(
            "rs_world_gate_to_gh",
            &format!("{prefix}-enter-gh"),
            start.world_epoch,
        )
        .map(|view| entry_test_support::acknowledge_ready(runtime, view))
        .unwrap();
    assert_eq!(entered.scene_id, "gh_entry_maintenance");

    place_player(runtime, 4.0, 0.0, 7.0);
    let terminal = runtime
        .activate_scene_interaction(
            "gh_entry_tutorial_terminal",
            &format!("{prefix}-entry-terminal"),
            entered.world_epoch,
        )
        .unwrap();
    assert!(terminal.applied, "{:?}", terminal.error_code);

    place_player(runtime, 3.0, 0.0, 7.0);
    runtime
        .activate_scene_checkpoint(
            "gh_entry_checkpoint",
            &format!("{prefix}-entry-checkpoint"),
            entered.world_epoch,
        )
        .unwrap();

    place_player(runtime, 18.0, 0.0, 7.0);
    let power = runtime
        .transition_scene(
            "gh_entry_to_power_room",
            &format!("{prefix}-entry-to-power"),
            entered.world_epoch,
        )
        .map(|view| entry_test_support::acknowledge_ready(runtime, view))
        .unwrap();
    assert_eq!(power.scene_id, "gh_power_room");

    place_player(runtime, 4.0, 0.0, 7.0);
    let console = runtime
        .activate_scene_interaction(
            "gh_power_console",
            &format!("{prefix}-power-console"),
            power.world_epoch,
        )
        .unwrap();
    assert!(console.applied, "{:?}", console.error_code);

    place_player(runtime, 3.0, 0.0, 7.0);
    runtime
        .activate_scene_checkpoint(
            "gh_power_checkpoint",
            &format!("{prefix}-power-checkpoint"),
            power.world_epoch,
        )
        .unwrap();

    let scene = runtime.scene_runtime.lock().unwrap();
    let scene = scene.as_ref().unwrap();
    assert!(scene.object_activated("gh_power_console"));
    assert!(scene.event_complete("hive_power"));
    assert_eq!(scene.checkpoint_id.as_deref(), Some("gh_power_checkpoint"));
}

fn continue_runtime(save_root: &std::path::Path) -> FormalRuntime {
    let continued = FormalRuntime::new_with_save_dir(save_root.to_path_buf()).unwrap();
    continued.stop_owner.store(true, Ordering::Release);
    continued
        .owner_handle
        .lock()
        .unwrap()
        .take()
        .expect("continued runtime owns its simulation thread")
        .join()
        .expect("continued runtime simulation thread exits");
    production_scene_bootstrap::install(&continued).unwrap();
    continued
}

#[test]
fn default_return_save_reconstructs_departed_grey_hive_scene_history() {
    let (runtime, root, fresh) = start_runtime("default-return");
    exercise_departure(&runtime, &fresh, "default-return");

    let returned = runtime.return_to_hub().unwrap();
    assert_eq!(returned.world_id, "return_station");
    assert_eq!(returned.scene_id, "rs_core_room");
    let persisted = crate::save_v6::read_save(&root).unwrap();
    assert_departed_rows(&persisted);
    drop(runtime);

    let continued = continue_runtime(&root);
    let resumed = continued
        .continue_saved()
        .map(|view| entry_test_support::acknowledge_ready(&continued, view))
        .unwrap();
    assert_eq!(resumed.scene_id, "rs_core_room");
    assert_reconstruction_after_continue(&continued, &persisted, &resumed);
}

#[test]
fn named_return_slot_reconstructs_departed_history_without_replacing_default_save() {
    let (runtime, root, fresh) = start_runtime("named-return");
    runtime.save().unwrap();
    let sentinel = std::fs::read(crate::save_v6::save_path(&root)).unwrap();

    exercise_departure(&runtime, &fresh, "named-return");
    let returned = runtime
        .return_to_hub_slot(NAMED_RETURN_SLOT, "Scene History Return", true)
        .unwrap();
    assert_eq!(returned.scene_id, "rs_core_room");
    assert_eq!(std::fs::read(crate::save_v6::save_path(&root)).unwrap(), sentinel);
    let (_, persisted) = crate::save_slots::read_slot_v6_candidate(&root, NAMED_RETURN_SLOT).unwrap();
    assert_departed_rows(&persisted);
    drop(runtime);

    let continued = continue_runtime(&root);
    let resumed = continued
        .continue_slot(NAMED_RETURN_SLOT)
        .map(|view| entry_test_support::acknowledge_ready(&continued, view))
        .unwrap();
    assert_eq!(resumed.scene_id, "rs_core_room");
    assert_reconstruction_after_continue(&continued, &persisted, &resumed);
    assert_eq!(std::fs::read(crate::save_v6::save_path(&root)).unwrap(), sentinel);
}

fn assert_reconstruction_after_continue(
    continued: &FormalRuntime,
    persisted: &SaveV6,
    resumed: &WorldView,
) {
    {
        let state = continued.state.lock().unwrap();
        assert_eq!(
            state.route.reward_ledger,
            persisted.save.progression.reward_ledger,
            "Continue restores the persisted route reward ledger before re-entry"
        );
        assert_eq!(
            state.capabilities.grants,
            persisted.save.capabilities.grants,
            "Continue restores persisted capability grants before re-entry"
        );
        assert_eq!(
            state.route.progress,
            persisted.save.progression.progress,
            "Continue restores persisted route progress before re-entry"
        );
        assert_eq!(
            state.route.event_seq,
            persisted.save.progression.event_seq,
            "Continue restores the persisted route event sequence before re-entry"
        );
    }

    let rewards_before = serde_json::to_value(
        &continued.state.lock().unwrap().route.reward_ledger,
    )
    .unwrap();
    let grants_before = serde_json::to_value(
        &continued.state.lock().unwrap().capabilities.grants,
    )
    .unwrap();
    let gh_before = gh_progress(continued);
    let route_event_seq_before = continued.state.lock().unwrap().route.event_seq;
    assert!(!gh_before.completed, "controlled setup has not earned completion");

    place_player(continued, 20.0, 0.0, 8.0);
    let entered = continued
        .use_world_gate(
            "rs_world_gate_to_gh",
            "scene-history-reconstructed-enter-gh",
            resumed.world_epoch,
        )
        .map(|view| entry_test_support::acknowledge_ready(continued, view))
        .unwrap();
    assert_eq!(entered.scene_id, "gh_entry_maintenance");
    {
        let state = continued.state.lock().unwrap();
        assert_eq!(state.route.event_seq, route_event_seq_before + 1,
            "only the authored GH gate entry is recorded during history hydration");
        let gh = state.route.progress.iter().find(|progress| progress.world_id == "grey_hive").unwrap();
        assert_eq!(gh.completed_events, gh_before.completed_events,
            "scene history does not replay route progress");
    }
    {
        let scene = continued.scene_runtime.lock().unwrap();
        let scene = scene.as_ref().unwrap();
        assert!(scene.object_activated("gh_entry_tutorial_terminal"));
        assert_eq!(scene.checkpoint_id.as_deref(), Some("gh_entry_checkpoint"));
        assert!(!scene.event_complete("hive_power"));
    }

    place_player(continued, 18.0, 0.0, 7.0);
    let power = continued
        .transition_scene(
            "gh_entry_to_power_room",
            "scene-history-reconstructed-enter-power",
            entered.world_epoch,
        )
        .map(|view| entry_test_support::acknowledge_ready(continued, view))
        .unwrap();
    assert_eq!(power.scene_id, "gh_power_room");
    {
        let scene = continued.scene_runtime.lock().unwrap();
        let scene = scene.as_ref().unwrap();
        assert!(scene.object_activated("gh_power_console"));
        assert!(scene.event_complete("hive_power"));
        assert_eq!(scene.checkpoint_id.as_deref(), Some("gh_power_checkpoint"));
    }

    let duplicate = continued
        .activate_scene_interaction(
            "gh_power_console",
            "scene-history-reconstructed-console-duplicate",
            power.world_epoch,
        )
        .unwrap();
    assert!(!duplicate.applied);
    assert_eq!(
        duplicate.error_code.as_deref(),
        Some("E_SCENE_RUNTIME_AlreadyApplied")
    );

    let state = continued.state.lock().unwrap();
    let gh_after = state
        .route
        .progress
        .iter()
        .find(|progress| progress.world_id == "grey_hive")
        .unwrap();
    assert!(!gh_after.completed, "hydration does not complete a world");
    assert_eq!(
        serde_json::to_value(&state.route.reward_ledger).unwrap(),
        rewards_before,
        "hydration and duplicate interaction grant no route rewards"
    );
    assert_eq!(
        serde_json::to_value(&state.capabilities.grants).unwrap(),
        grants_before,
        "hydration grants no capabilities"
    );
    assert_eq!(state.route.event_seq, route_event_seq_before + 1,
        "scene re-entry and duplicate activation add no route events");
    drop(state);

    let reset = continued
        .reset_new()
        .map(|view| entry_test_support::acknowledge_ready(continued, view))
        .unwrap();
    assert_eq!(reset.scene_id, "rs_core_room");
    assert!(continued
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .scene_history_v1
        .is_none());
}
