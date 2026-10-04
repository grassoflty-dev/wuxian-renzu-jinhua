use super::*;
use crate::{save_slots, save_v5::SaveV5, save_v6::SaveV6};
use std::{fs, path::Path, sync::atomic::AtomicU64, time::{SystemTime, UNIX_EPOCH}};

static SERIAL: AtomicU64 = AtomicU64::new(1);

fn runtime() -> FormalRuntime {
    let root = std::env::temp_dir().join(format!(
        "v5-slot-overwrite-{}-{}-{}", std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        SERIAL.fetch_add(1, Ordering::SeqCst)
    ));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    entry_test_support::acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    runtime
}

fn legacy_slot(runtime: &FormalRuntime) -> (PathBuf, Vec<u8>) {
    let state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    let legacy = SaveV5::capture(&state, scene.as_ref()).unwrap();
    save_slots::create_slot(&runtime.save_root, "legacy", "Legacy V5", &legacy).unwrap();
    let path = runtime.save_root.join("slots/legacy/slot-v5.json");
    let bytes = fs::read(&path).unwrap();
    (path, bytes)
}

fn scene_state(runtime: &FormalRuntime) -> String {
    format!("{:?}", runtime.scene_runtime.lock().unwrap().as_ref().unwrap())
}

fn assert_source_only(path: &Path, original: &[u8]) {
    assert_eq!(fs::read(path).unwrap(), original);
    let files: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap()
        .map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(files, ["slot-v5.json"], "No pre-migration or temporary file may survive failure");
}

#[test]
fn v5_only_writable_slot_overwrite_saves_current_journey_without_pre_migration() {
    let runtime = runtime();
    let (source, original) = legacy_slot(&runtime);
    let listed = save_slots::list_slots(&runtime.save_root);
    assert_eq!(listed.len(), 1);
    assert!(listed[0].valid && !listed[0].read_only);
    assert_source_only(&source, &original);
    // A different journey, not Continue on the target slot.
    entry_test_support::acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    runtime.state.lock().unwrap().world.player_hp = 37;
    runtime.pause().unwrap();
    runtime.save_slot("anchor", "Other active slot", true).unwrap();
    let anchor = runtime.save_root.join("slots/anchor/slot-v6.json");
    let anchor_bytes = fs::read(&anchor).unwrap();
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    let expected = {
        let state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        SaveV6::capture(&state, scene.as_ref()).unwrap()
    };
    runtime.save_slot("legacy", "Updated current journey", false).unwrap();
    let (name, saved) = save_slots::read_slot_v6(&runtime.save_root, "legacy").unwrap();
    assert_eq!(name, "Updated current journey");
    assert_eq!(saved, expected);
    assert_eq!(saved.save.player.current_hp, 37);
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(scene_state(&runtime), scene_before);
    assert_eq!(*runtime.active_slot.lock().unwrap(), Some("legacy".into()));
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(fs::read(&anchor).unwrap(), anchor_bytes);
    assert!(!crate::save_v6::save_path(&runtime.save_root).exists());
    let continued = entry_test_support::acknowledge_ready(&runtime, runtime.continue_slot("legacy").unwrap());
    assert_eq!(continued.player.current_hp, 37);
    runtime.pause().unwrap();
    runtime.save_slot("legacy", "Existing V6 overwrite", false).unwrap();
    assert_eq!(fs::read(&source).unwrap(), original);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn v5_overwrite_rejects_corrupt_future_or_mismatched_source_without_writing() {
    for fault in ["corrupt", "envelope-version", "save-version", "identity", "unknown-field"] {
        let runtime = runtime();
        let (source, original) = legacy_slot(&runtime);
        let mut raw: serde_json::Value = serde_json::from_slice(&original).unwrap();
        match fault {
            "envelope-version" => raw["schemaVersion"] = 999.into(),
            "save-version" => raw["save"]["schemaVersion"] = 999.into(),
            "identity" => raw["slotId"] = "different".into(),
            "unknown-field" => raw["futureField"] = true.into(),
            _ => (),
        }
        let bytes = if fault == "corrupt" { b"not-json".to_vec() } else { serde_json::to_vec(&raw).unwrap() };
        fs::write(&source, &bytes).unwrap();
        let before = runtime.snapshot().unwrap();
        let scene_before = scene_state(&runtime);
        let listed = save_slots::list_slots(&runtime.save_root);
        assert!(!listed[0].valid && listed[0].read_only, "{fault}");
        assert!(runtime.save_slot("legacy", "Must reject", false).is_err(), "{fault}");
        assert_source_only(&source, &bytes);
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(scene_state(&runtime), scene_before);
        assert_eq!(*runtime.active_slot.lock().unwrap(), None);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn present_invalid_v6_never_falls_back_to_valid_v5_for_overwrite() {
    for fault in ["corrupt", "future-envelope", "future-save", "directory"] {
        let runtime = runtime();
        let (source, original) = legacy_slot(&runtime);
        let target = source.with_file_name("slot-v6.json");
        let bytes = if fault.starts_with("future") {
            runtime.save_slot("current", "Current", true).unwrap();
            let mut raw: serde_json::Value = serde_json::from_slice(&fs::read(runtime.save_root.join("slots/current/slot-v6.json")).unwrap()).unwrap();
            raw["slotId"] = "legacy".into();
            if fault == "future-envelope" { raw["schemaVersion"] = 999.into(); }
            else { raw["save"]["schemaVersion"] = 999.into(); }
            serde_json::to_vec(&raw).unwrap()
        } else { b"corrupt-newer".to_vec() };
        if fault == "directory" { fs::create_dir(&target).unwrap(); }
        else { fs::write(&target, &bytes).unwrap(); }
        let before = runtime.snapshot().unwrap();
        let active_before = runtime.active_slot.lock().unwrap().clone();
        let scene_before = scene_state(&runtime);
        assert!(runtime.save_slot("legacy", "Must not fall back", false).is_err(), "{fault}");
        assert_eq!(fs::read(&source).unwrap(), original);
        if fault == "directory" { assert!(target.is_dir()); }
        else { assert_eq!(fs::read(&target).unwrap(), bytes); }
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(scene_state(&runtime), scene_before);
        assert_eq!(*runtime.active_slot.lock().unwrap(), active_before);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn dangling_newer_v6_path_blocks_v5_overwrite() {
    let runtime = runtime();
    let (source, original) = legacy_slot(&runtime);
    let target = source.with_file_name("slot-v6.json");
    std::os::unix::fs::symlink("missing-newer", &target).unwrap();
    assert!(runtime.save_slot("legacy", "Must not fall back", false).is_err());
    assert_eq!(fs::read_link(&target).unwrap(), Path::new("missing-newer"));
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(*runtime.active_slot.lock().unwrap(), None);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[cfg(unix)]
#[test]
fn unreadable_newer_v6_path_blocks_v5_overwrite() {
    use std::os::unix::fs::PermissionsExt;
    let runtime = runtime();
    let (source, original) = legacy_slot(&runtime);
    let target = source.with_file_name("slot-v6.json");
    fs::write(&target, b"newer-source-must-win").unwrap();
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    fs::set_permissions(&target, fs::Permissions::from_mode(0o000)).unwrap();
    let result = runtime.save_slot("legacy", "Must not fall back", false);
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(result.unwrap_err().starts_with("E_SLOT_READ"));
    assert_eq!(fs::read(&target).unwrap(), b"newer-source-must-win");
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(scene_state(&runtime), scene_before);
    assert_eq!(*runtime.active_slot.lock().unwrap(), None);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn v4_only_slot_stays_read_only_and_missing_slot_cannot_be_overwritten() {
    let runtime = runtime();
    let dir = runtime.save_root.join("slots/old");
    fs::create_dir_all(&dir).unwrap();
    let save: serde_json::Value = serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v4.json")).unwrap();
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schemaVersion": 1, "slotId": "old", "displayName": "V4", "updatedAtMs": 1, "save": save
    })).unwrap();
    let source = dir.join("slot-v4.json");
    fs::write(&source, &bytes).unwrap();
    let listed = save_slots::list_slots(&runtime.save_root);
    assert!(listed[0].valid && listed[0].read_only);
    assert_eq!(runtime.save_slot("old", "No implicit V4 migration", false).unwrap_err(), "E_SLOT_NOT_FOUND");
    assert_eq!(runtime.save_slot("missing", "Missing", false).unwrap_err(), "E_SLOT_NOT_FOUND");
    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    assert!(!runtime.save_root.join("slots/missing").exists());
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn v5_overwrite_capture_failure_preserves_files_scene_and_active_slot() {
    let runtime = runtime();
    let (source, original) = legacy_slot(&runtime);
    runtime.save_slot("anchor", "Anchor", true).unwrap();
    { let mut state = runtime.state.lock().unwrap(); state.world.player_hp = state.world.player_max_hp + 1; }
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    assert!(runtime.save_slot("legacy", "Capture must fail", false).is_err());
    assert_source_only(&source, &original);
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(scene_state(&runtime), scene_before);
    assert_eq!(*runtime.active_slot.lock().unwrap(), Some("anchor".into()));
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[cfg(unix)]
#[test]
fn v5_overwrite_write_failure_preserves_source_and_destination_then_retries() {
    use std::os::unix::fs::PermissionsExt;
    let runtime = runtime();
    let (source, original) = legacy_slot(&runtime);
    runtime.state.lock().unwrap().world.player_hp = 37;
    runtime.pause().unwrap();
    runtime.save_slot("anchor", "Anchor", true).unwrap();
    runtime.save().unwrap();
    let default_path = crate::save_v6::save_path(&runtime.save_root);
    let default_bytes = fs::read(&default_path).unwrap();
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    fs::set_permissions(source.parent().unwrap(), fs::Permissions::from_mode(0o555)).unwrap();
    let result = runtime.save_slot("legacy", "Retry", false);
    fs::set_permissions(source.parent().unwrap(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.unwrap_err().starts_with("E_SLOT_TEMP"));
    assert_source_only(&source, &original);
    assert_eq!(fs::read(&default_path).unwrap(), default_bytes);
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(scene_state(&runtime), scene_before);
    assert_eq!(*runtime.active_slot.lock().unwrap(), Some("anchor".into()));
    runtime.save_slot("legacy", "Retry", false).unwrap();
    assert_eq!(save_slots::read_slot_v6(&runtime.save_root, "legacy").unwrap().1.save.player.current_hp, 37);
    assert_eq!(*runtime.active_slot.lock().unwrap(), Some("legacy".into()));
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(fs::read(&default_path).unwrap(), default_bytes);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}
