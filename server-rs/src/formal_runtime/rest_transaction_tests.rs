use super::*;
use std::{fs, sync::atomic::AtomicU64, time::{SystemTime, UNIX_EPOCH}};

static SERIAL: AtomicU64 = AtomicU64::new(1);
const TERMINAL: &str = "rs_save_rest_terminal_marker";

fn runtime() -> FormalRuntime {
    let root = std::env::temp_dir().join(format!(
        "rest-transaction-{}-{}-{}", std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        SERIAL.fetch_add(1, Ordering::SeqCst)
    ));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    entry_test_support::acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    {
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = Vec3 { x_m: 8.0, y_m: 0.0, z_m: 12.0 };
        state.world.player_hp = 37;
        state.world.player_energy = 21;
    }
    runtime.save().unwrap();
    runtime
}

fn scene_state(runtime: &FormalRuntime) -> String {
    format!("{:?}", runtime.scene_runtime.lock().unwrap().as_ref().unwrap())
}

fn assert_no_live_change(runtime: &FormalRuntime, before: &WorldView, scene_before: &str, active_before: &Option<String>) {
    assert_eq!(runtime.snapshot().unwrap(), *before);
    assert_eq!(scene_state(runtime), scene_before);
    assert_eq!(*runtime.active_slot.lock().unwrap(), *active_before);
}

fn assert_retry_then_duplicate(runtime: &FormalRuntime, request: &str, epoch: u64) {
    let rested = runtime.save_rest_terminal(TERMINAL, request, epoch).unwrap();
    assert_eq!(rested.player.current_hp, rested.player.max_hp);
    assert_eq!(rested.player.current_energy, rested.player.max_energy);
    let path = crate::save_v6::save_path(&runtime.save_root);
    let saved = crate::save_v6::read_save(&runtime.save_root).unwrap();
    assert_eq!(saved.save.player.current_hp, rested.player.max_hp);
    assert_eq!(saved.save.player.current_energy, rested.player.max_energy);
    let file = fs::read(&path).unwrap();
    let scene_before = scene_state(runtime);
    // Success consumes the ID: it must never heal or persist a second time.
    runtime.state.lock().unwrap().world.player_hp = 13;
    let before = runtime.snapshot().unwrap();
    assert_eq!(runtime.save_rest_terminal(TERMINAL, request, epoch).unwrap_err(), "E_REST_TERMINAL_DuplicateRequest");
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert_eq!(scene_state(runtime), scene_before);
    assert_eq!(fs::read(path).unwrap(), file);
}

#[cfg(unix)]
#[test]
fn rest_temp_write_failure_rolls_back_scene_claim_and_same_id_retries() {
    use std::os::unix::fs::PermissionsExt;
    let runtime = runtime();
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    let active_before = runtime.active_slot.lock().unwrap().clone();
    let path = crate::save_v6::save_path(&runtime.save_root);
    let bytes = fs::read(&path).unwrap();
    fs::set_permissions(&runtime.save_root, fs::Permissions::from_mode(0o555)).unwrap();
    let result = runtime.save_rest_terminal(TERMINAL, "same-rest-write", before.world_epoch);
    fs::set_permissions(&runtime.save_root, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.unwrap_err().starts_with("E_SAVE_TEMP"));
    assert_no_live_change(&runtime, &before, &scene_before, &active_before);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::read_dir(&runtime.save_root).unwrap().count(), 1);
    assert_retry_then_duplicate(&runtime, "same-rest-write", before.world_epoch);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn rest_existing_target_read_failure_releases_request_without_changing_memory() {
    let runtime = runtime();
    let path = crate::save_v6::save_path(&runtime.save_root);
    let original = fs::read(&path).unwrap();
    let fixture_backup = path.with_extension("fixture-backup");
    fs::rename(&path, &fixture_backup).unwrap();
    fs::create_dir(&path).unwrap();
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    let active_before = runtime.active_slot.lock().unwrap().clone();
    let result = runtime.save_rest_terminal(TERMINAL, "same-rest-read", before.world_epoch);
    assert!(result.unwrap_err().starts_with("E_SAVE_READ"));
    assert_no_live_change(&runtime, &before, &scene_before, &active_before);
    assert!(path.is_dir());
    assert_eq!(fs::read(&fixture_backup).unwrap(), original);
    fs::remove_dir(&path).unwrap();
    fs::rename(&fixture_backup, &path).unwrap();
    assert_retry_then_duplicate(&runtime, "same-rest-read", before.world_epoch);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn rest_capture_failure_releases_request_without_healing_or_overwriting() {
    let runtime = runtime();
    let path = crate::save_v6::save_path(&runtime.save_root);
    let original = fs::read(&path).unwrap();
    let max_hp = runtime.state.lock().unwrap().world.player_max_hp;
    runtime.state.lock().unwrap().world.player_max_hp = 0;
    let before = runtime.snapshot().unwrap();
    let scene_before = scene_state(&runtime);
    let active_before = runtime.active_slot.lock().unwrap().clone();
    assert!(runtime.save_rest_terminal(TERMINAL, "same-rest-capture", before.world_epoch).unwrap_err().starts_with("E_SAVE_"));
    assert_no_live_change(&runtime, &before, &scene_before, &active_before);
    assert_eq!(fs::read(&path).unwrap(), original);
    runtime.state.lock().unwrap().world.player_max_hp = max_hp;
    assert_retry_then_duplicate(&runtime, "same-rest-capture", before.world_epoch);
    fs::remove_dir_all(&runtime.save_root).unwrap();
}
