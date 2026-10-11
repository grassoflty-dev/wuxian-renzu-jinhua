use std::{
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    capability_v1::CAP_REAR_VIEW,
    continuous_input::InputSample,
    formal_runtime::{CombatIntentRequest, FormalRuntime, RouteCommandRequest},
};

const REAL_LEGACY_V2: &str = include_str!("fixtures/freeze_v02_save_v2.json");

fn isolated_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "wuxian-save-v3-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn complete_owner_roundtrip_preserves_state_and_advances_epoch_safely() {
    let root = isolated_dir("roundtrip");
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.choose_first_enhancement(CAP_REAR_VIEW).unwrap();
    runtime
        .apply_route_command(RouteCommandRequest::Progress {
            event_id: "hive_power".into(),
            request_id: "save-test-power-once".into(),
        })
        .unwrap();
    let before = runtime
        .submit_input(
            InputSample::new(1, 1, 100, 0.0, 1.0).unwrap(),
            vec![CombatIntentRequest::Jump { request_id: 77 }],
        )
        .unwrap();
    runtime.save().unwrap();
    assert!(runtime.has_save());
    let persisted = wuxian_horror_ch1::save_v3::read_save(&root).unwrap();
    assert!(persisted.combat_handled_request_ids.contains(&77));

    let restarted = FormalRuntime::new_with_save_dir(root).unwrap();
    let after = restarted.continue_saved().unwrap();
    assert_eq!(after.world_id, before.world_id);
    assert_eq!(after.server_tick, before.server_tick);
    assert_eq!(after.authority_revision, before.authority_revision);
    assert_eq!(after.ack_seq, before.ack_seq);
    assert_eq!(after.server_time_ms, before.server_time_ms);
    assert_eq!(after.player, before.player);
    assert_eq!(after.capabilities, before.capabilities);
    assert_eq!(after.progression, before.progression);
    assert_eq!(after.world_epoch, before.world_epoch + 1);
    assert_eq!(after.capabilities.rear_view.granted, true);

    let duplicate_progress = restarted
        .apply_route_command(RouteCommandRequest::Progress {
            event_id: "hive_power".into(),
            request_id: "save-test-power-once".into(),
        })
        .unwrap();
    assert_eq!(
        duplicate_progress.view.progression.event_seq,
        after.progression.event_seq
    );

    assert_eq!(
        restarted
            .submit_input(
                InputSample::new(before.world_epoch, 2, 101, 0.0, 0.0).unwrap(),
                vec![]
            )
            .unwrap_err(),
        "E_INPUT_STALE_EPOCH"
    );
    let continued = restarted
        .submit_input(
            InputSample::new(after.world_epoch, after.ack_seq + 1, 101, 0.0, 1.0).unwrap(),
            vec![],
        )
        .unwrap();
    assert_eq!(continued.ack_seq, after.ack_seq + 1);
    assert!(continued.server_tick > after.server_tick);
    assert_eq!(continued.progression.event_seq, after.progression.event_seq);
}

#[test]
fn cross_process_worker() {
    let (Some(root), Some(role)) = (
        std::env::var_os("WUXIAN_SAVE_TEST_ROOT"),
        std::env::var("WUXIAN_SAVE_TEST_ROLE").ok(),
    ) else {
        return;
    };
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    match role.as_str() {
        "write" => {
            runtime
                .submit_input(InputSample::new(1, 1, 100, 1.0, 0.0).unwrap(), vec![])
                .unwrap();
            runtime.save().unwrap();
        }
        "load" => {
            let restored = runtime.continue_saved().unwrap();
            assert_eq!(restored.server_tick, 1);
            assert_eq!(restored.ack_seq, 1);
            assert_eq!(restored.world_epoch, 2);
            assert!(restored.player.transform.position_m.x_m > 0.0);
        }
        other => panic!("unexpected worker role {other}"),
    }
}

#[test]
fn separate_processes_can_save_exit_and_continue() {
    let root = isolated_dir("process");
    let exe = std::env::current_exe().unwrap();
    for role in ["write", "load"] {
        let result = Command::new(&exe)
            .args(["--exact", "cross_process_worker", "--nocapture"])
            .env("WUXIAN_SAVE_TEST_ROOT", &root)
            .env("WUXIAN_SAVE_TEST_ROLE", role)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{role} child failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn corrupt_save_fails_closed_and_does_not_mutate_owner() {
    let root = isolated_dir("corrupt");
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.save().unwrap();
    runtime.save().unwrap();
    std::fs::write(root.join("formal-save-v3.json"), "{ broken").unwrap();
    let before = runtime.snapshot().unwrap();
    assert_eq!(runtime.continue_saved().unwrap_err(), "E_SAVE_CORRUPT");
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert!(!runtime.has_save());
    assert!(std::fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with("formal-save-v3.json.bak.")));
}

#[test]
fn unknown_save_version_is_rejected_without_changing_live_state() {
    let root = isolated_dir("version");
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.save().unwrap();
    let path = root.join("formal-save-v3.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    document["schemaVersion"] = serde_json::json!(99);
    std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let before = runtime.snapshot().unwrap();
    assert_eq!(
        runtime.continue_saved().unwrap_err(),
        "E_SAVE_VERSION_UNSUPPORTED"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
    assert!(!runtime.has_save());
}

#[test]
fn orphaned_interrupted_temp_file_does_not_replace_a_valid_save() {
    let root = isolated_dir("interrupted");
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.save().unwrap();
    let target = root.join("formal-save-v3.json");
    let before = std::fs::read(&target).unwrap();
    std::fs::write(root.join(".formal-save-v3.999999.1.tmp"), "{ incomplete").unwrap();
    assert!(runtime.has_save());
    let restored = FormalRuntime::new_with_save_dir(root.clone())
        .unwrap()
        .continue_saved()
        .unwrap();
    assert_eq!(restored.server_tick, 0);
    assert_eq!(std::fs::read(target).unwrap(), before);
}

#[test]
fn legacy_grid_save_slot_is_never_used_as_formal_save() {
    let root = isolated_dir("legacy-isolation");
    std::fs::create_dir_all(&root).unwrap();
    let old = br#"{"saveVersion":2,"worldId":"biohazard_ch1","px":14,"py":9,"sceneId":"s_office"}"#;
    std::fs::write(root.join("save.json"), old).unwrap();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    assert!(!runtime.has_save());
    assert_eq!(runtime.continue_saved().unwrap_err(), "E_NO_SAVE");
    assert_eq!(std::fs::read(root.join("save.json")).unwrap(), old);
}

#[test]
fn authentic_legacy_game_state_v2_is_rejected_without_grid_coordinate_conversion() {
    let parsed: wuxian_horror_ch1::state::GameState = serde_json::from_str(REAL_LEGACY_V2).unwrap();
    assert_eq!(parsed.save_version, 2);
    assert_eq!(parsed.world_id, "biohazard_ch1");
    assert_eq!((parsed.px, parsed.py), (1, 1));
    let error = wuxian_horror_ch1::save_v3::inspect_legacy_grid_v2(REAL_LEGACY_V2).unwrap_err();
    assert!(error.starts_with("E_V2_GRID_NO_CHECKPOINT_MAPPING"));
    assert!(!error.contains("CELL_SIZE"));
}
