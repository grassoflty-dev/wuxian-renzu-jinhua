use super::FormalRuntime;
use crate::{save_slots, save_v6::SaveV6, world_v3::{Vec3, WorldView}};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const TARGET_SLOT_ID: &str = "named-v6-target";
const TARGET_SLOT_NAME: &str = "Named V6 Target";
const OTHER_SLOT_ID: &str = "other-v6-slot";
const OTHER_SLOT_NAME: &str = "Other V6 Slot";
const RETURN_STATION: &str = "return_station";
const RS_CORE_ROOM: &str = "rs_core_room";
const RS_CORE_CHECKPOINT: &str = "rs_core_checkpoint";
static NEXT_SERIAL: AtomicU64 = AtomicU64::new(1);

fn unique_owned_save_root() -> PathBuf {
    let serial = NEXT_SERIAL.fetch_add(1, Ordering::SeqCst);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "named-v6-overwrite-roundtrip-{}-{now}-{serial}",
        std::process::id(),
    ));
    assert!(!root.exists(), "owned save root must be new: {}", root.display());
    fs::create_dir(&root).expect("create owned save root");
    root
}

fn stopped_runtime(save_root: &Path) -> FormalRuntime {
    let runtime = FormalRuntime::new_with_save_dir(save_root.to_path_buf())
        .expect("start an owned FormalRuntime");
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime
        .owner_handle
        .lock()
        .expect("lock this runtime's owner handle")
        .take()
        .expect("each runtime owns one simulation thread")
        .join()
        .expect("join this runtime's simulation owner");
    crate::production_scene_bootstrap::install(&runtime)
        .expect("install full production scene registry");
    runtime
}

fn ready(runtime: &FormalRuntime, prepared: WorldView) -> WorldView {
    let mut view = crate::formal_runtime::entry_test_support::acknowledge_ready(runtime, prepared);
    if runtime.state.lock().expect("lock runtime state").paused {
        view = runtime
            .resume()
            .expect("resume through the native API after readiness");
    }
    assert!(
        !runtime.state.lock().expect("lock runtime state").paused,
        "readiness leaves the scene usable through native APIs"
    );
    view
}

fn assert_return_station_view(view: &WorldView, expected_checkpoint: Option<&str>) {
    assert_eq!(view.world_id, RETURN_STATION);
    assert_eq!(view.scene_id, RS_CORE_ROOM);
    assert_eq!(view.checkpoint_id.as_deref(), expected_checkpoint);
    assert!(view.player.current_hp > 0, "player remains alive");
    assert!(view.player.current_hp <= view.player.max_hp);
    assert!(view.player.current_energy <= view.player.max_energy);
}

fn assert_checkpoint_owners(
    runtime: &FormalRuntime,
    view: &WorldView,
    expected_checkpoint: Option<&str>,
) {
    assert_return_station_view(view, expected_checkpoint);
    {
        let state = runtime.state.lock().expect("lock runtime state");
        assert_eq!(state.world.world_id, RETURN_STATION);
        assert_eq!(state.world.scene_id, RS_CORE_ROOM);
        assert_eq!(state.world.checkpoint_id.as_deref(), expected_checkpoint);
        assert!(state.pending_entry.is_none());
    }
    let scene_guard = runtime.scene_runtime.lock().expect("lock scene runtime");
    let scene = scene_guard.as_ref().expect("active production scene runtime");
    assert_eq!(scene.scene_id, RS_CORE_ROOM);
    assert_eq!(scene.checkpoint_id.as_deref(), expected_checkpoint);
    assert_eq!(scene.world_epoch, view.world_epoch);
    assert_eq!(scene.current_scene().world_id, RETURN_STATION);
    assert_eq!(scene.current_scene().scene_id, RS_CORE_ROOM);
}

fn capture(runtime: &FormalRuntime) -> SaveV6 {
    let state = runtime.state.lock().expect("lock runtime state for capture");
    let scene = runtime
        .scene_runtime
        .lock()
        .expect("lock scene runtime for capture");
    SaveV6::capture(&state, scene.as_ref()).expect("capture current authoritative SaveV6")
}

fn target_path(root: &Path) -> PathBuf {
    root.join("slots")
        .join(TARGET_SLOT_ID)
        .join("slot-v6.json")
}

fn other_path(root: &Path) -> PathBuf {
    root.join("slots")
        .join(OTHER_SLOT_ID)
        .join("slot-v6.json")
}

fn default_path(root: &Path) -> PathBuf {
    crate::save_v6::save_path(root)
}

fn vitals(view: &WorldView) -> (u32, u32) {
    (view.player.current_hp, view.player.current_energy)
}

fn assert_checkpoint_free_sentinels(root: &Path) {
    let (_, other) = save_slots::read_slot_v6(root, OTHER_SLOT_ID)
        .expect("read untouched other named V6 slot");
    let default = crate::save_v6::read_save(root).expect("read untouched default V6 save");
    assert_eq!(other.save.world_id, RETURN_STATION);
    assert_eq!(other.save.scene_id, RS_CORE_ROOM);
    assert_eq!(other.save.checkpoint_id, None);
    assert_eq!(default.save.world_id, RETURN_STATION);
    assert_eq!(default.save.scene_id, RS_CORE_ROOM);
    assert_eq!(default.save.checkpoint_id, None);
}

#[test]
fn continued_named_v6_overwrite_restores_checkpoint_in_a_new_runtime() {
    let save_root = unique_owned_save_root();
    let target_file = target_path(&save_root);
    let other_file = other_path(&save_root);
    let default_file = default_path(&save_root);

    let (
        initial_target_bytes,
        initial_other_bytes,
        initial_default_bytes,
        initial_target_save,
        original_vitals,
    ) = {
        let runtime1 = stopped_runtime(&save_root);
        let initial_view = ready(&runtime1, runtime1.reset_new().expect("start new journey"));
        assert_checkpoint_owners(&runtime1, &initial_view, None);
        let original_vitals = vitals(&initial_view);
        assert!(original_vitals.0 > 0);

        runtime1
            .save_slot(TARGET_SLOT_ID, TARGET_SLOT_NAME, true)
            .expect("create named target through FormalRuntime");
        runtime1
            .save_slot(OTHER_SLOT_ID, OTHER_SLOT_NAME, true)
            .expect("create independent named sentinel through FormalRuntime");
        runtime1.save().expect("create default sentinel through FormalRuntime");

        let (target_name, initial_target_save) =
            save_slots::read_slot_v6(&save_root, TARGET_SLOT_ID).expect("read initial target");
        assert_eq!(target_name, TARGET_SLOT_NAME);
        assert_eq!(initial_target_save.save.world_id, RETURN_STATION);
        assert_eq!(initial_target_save.save.scene_id, RS_CORE_ROOM);
        assert_eq!(initial_target_save.save.checkpoint_id, None);
        assert_checkpoint_free_sentinels(&save_root);
        assert_eq!(
            *runtime1.active_slot.lock().expect("lock selected slot"),
            Some(OTHER_SLOT_ID.into())
        );

        let initial_target_bytes = fs::read(&target_file).expect("read target sentinel bytes");
        let initial_other_bytes = fs::read(&other_file).expect("read other sentinel bytes");
        let initial_default_bytes = fs::read(&default_file).expect("read default sentinel bytes");
        drop(runtime1);
        (
            initial_target_bytes,
            initial_other_bytes,
            initial_default_bytes,
            initial_target_save,
            original_vitals,
        )
    };

    let (saved_target_bytes, expected_saved_v6, checkpoint_vitals) = {
        let runtime2 = stopped_runtime(&save_root);
        let continued = ready(
            &runtime2,
            runtime2
                .continue_slot(TARGET_SLOT_ID)
                .expect("continue the named V6 target in runtime2"),
        );
        assert_checkpoint_owners(&runtime2, &continued, None);
        assert_eq!(vitals(&continued), original_vitals);
        assert_eq!(
            *runtime2.active_slot.lock().expect("lock selected slot"),
            Some(TARGET_SLOT_ID.into())
        );

        // Synthetic fixture setup only: place this test-owned player at the authored RS checkpoint.
        {
            let mut state = runtime2.state.lock().expect("lock runtime2 fixture state");
            state.world.player.position_m = Vec3 {
                x_m: 4.0,
                y_m: 0.0,
                z_m: 8.0,
            };
        }
        let checkpoint_request = format!(
            "named-v6-checkpoint-{}-{}",
            std::process::id(),
            NEXT_SERIAL.fetch_add(1, Ordering::SeqCst)
        );
        let checkpoint_epoch = runtime2.snapshot().expect("read checkpoint epoch").world_epoch;
        let checkpoint_view = runtime2
            .activate_scene_checkpoint(
                RS_CORE_CHECKPOINT,
                &checkpoint_request,
                checkpoint_epoch,
            )
            .expect("activate authored Return Station checkpoint");
        assert_checkpoint_owners(&runtime2, &checkpoint_view, Some(RS_CORE_CHECKPOINT));
        let checkpoint_vitals = vitals(&checkpoint_view);
        assert_eq!(checkpoint_vitals, original_vitals);
        assert_eq!(
            fs::read(&target_file).expect("verify no implicit target rewrite"),
            initial_target_bytes,
            "checkpoint activation must precede the explicit overwrite"
        );
        assert_eq!(fs::read(&other_file).unwrap(), initial_other_bytes);
        assert_eq!(fs::read(&default_file).unwrap(), initial_default_bytes);

        let expected_saved_v6 = capture(&runtime2);
        let overwritten_view = runtime2
            .save_slot(TARGET_SLOT_ID, TARGET_SLOT_NAME, false)
            .expect("explicitly overwrite the selected named V6 slot");
        assert_checkpoint_owners(&runtime2, &overwritten_view, Some(RS_CORE_CHECKPOINT));
        assert_eq!(vitals(&overwritten_view), checkpoint_vitals);
        assert_eq!(overwritten_view.world_epoch, checkpoint_view.world_epoch);
        assert_eq!(
            *runtime2.active_slot.lock().expect("lock selected slot"),
            Some(TARGET_SLOT_ID.into())
        );

        let (saved_name, saved_v6) =
            save_slots::read_slot_v6(&save_root, TARGET_SLOT_ID).expect("read overwritten target");
        assert_eq!(saved_name, TARGET_SLOT_NAME);
        assert_eq!(saved_v6, expected_saved_v6);
        assert_eq!(saved_v6.save.world_id, RETURN_STATION);
        assert_eq!(saved_v6.save.scene_id, RS_CORE_ROOM);
        assert_eq!(saved_v6.save.checkpoint_id.as_deref(), Some(RS_CORE_CHECKPOINT));
        assert_eq!(saved_v6.save.scene_states.len(), 1);
        assert_eq!(saved_v6.save.scene_states[0].scene_id, RS_CORE_ROOM);
        assert_eq!(
            saved_v6.save.scene_states[0].checkpoint_id.as_deref(),
            Some(RS_CORE_CHECKPOINT)
        );
        assert_eq!(initial_target_save.save.checkpoint_id, None);
        assert_ne!(
            saved_v6.save.checkpoint_id,
            initial_target_save.save.checkpoint_id,
            "the overwrite changes checkpoint semantics"
        );
        let saved_target_bytes = fs::read(&target_file).expect("read overwritten target bytes");
        assert_ne!(saved_target_bytes, initial_target_bytes);
        assert_eq!(fs::read(&other_file).unwrap(), initial_other_bytes);
        assert_eq!(fs::read(&default_file).unwrap(), initial_default_bytes);
        assert_checkpoint_free_sentinels(&save_root);
        drop(runtime2);
        (saved_target_bytes, expected_saved_v6, checkpoint_vitals)
    };

    {
        let runtime3 = stopped_runtime(&save_root);
        let restored = ready(
            &runtime3,
            runtime3
                .continue_slot(TARGET_SLOT_ID)
                .expect("continue the overwritten target in runtime3"),
        );
        assert_checkpoint_owners(&runtime3, &restored, Some(RS_CORE_CHECKPOINT));
        assert_eq!(vitals(&restored), checkpoint_vitals);
        assert_eq!(
            *runtime3.active_slot.lock().expect("lock selected slot"),
            Some(TARGET_SLOT_ID.into())
        );
        let (restored_name, restored_v6) =
            save_slots::read_slot_v6(&save_root, TARGET_SLOT_ID).expect("read restored target");
        assert_eq!(restored_name, TARGET_SLOT_NAME);
        assert_eq!(restored_v6, expected_saved_v6);
        assert_eq!(fs::read(&target_file).unwrap(), saved_target_bytes);
        assert_eq!(fs::read(&other_file).unwrap(), initial_other_bytes);
        assert_eq!(fs::read(&default_file).unwrap(), initial_default_bytes);
        assert_checkpoint_free_sentinels(&save_root);
        drop(runtime3);
    }

    // Preserve this unique test-owned directory and all three V6 files as evidence.
    assert!(save_root.exists());
    assert!(target_file.exists());
    assert!(other_file.exists());
    assert!(default_file.exists());
}
