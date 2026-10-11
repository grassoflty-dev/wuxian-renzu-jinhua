use std::{
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    continuous_input::InputSample,
    continuous_kcc::{step_kcc, KccBody, StaticKccWorld},
    formal_runtime::{FormalRuntime, POWER_CONSOLE_ID},
    world_v3::Vec3,
};

fn isolated_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "grey-hive-first-flow-{}-{nonce}",
        std::process::id()
    ))
}

fn move_player(
    runtime: &FormalRuntime,
    mut view: wuxian_horror_ch1::world_v3::WorldView,
    dz: f32,
    ticks: u64,
) -> wuxian_horror_ch1::world_v3::WorldView {
    for seq in (view.ack_seq + 1)..=(view.ack_seq + ticks) {
        view = runtime
            .submit_input(
                InputSample::new(view.world_epoch, seq, seq * 50, 0.0, dz).unwrap(),
                vec![],
            )
            .unwrap();
    }
    view
}

#[test]
fn collision_shell_and_closed_gate_block_the_contracted_route() {
    let mut body = KccBody::new(Vec3::new(0.0, 0.0, 7.4).unwrap());
    let closed = StaticKccWorld::grey_hive_first_flow(false);
    let collisions = step_kcc(&mut body, &closed, (0.0, 1.0), 0.05).unwrap();
    assert!(!collisions.is_empty());
    assert!(body.position_m.z_m <= 7.4);

    let mut room_edge = KccBody::new(Vec3::zero());
    let mut boundary_collision = false;
    for _ in 0..20 {
        let collisions = step_kcc(&mut room_edge, &closed, (-1.0, 0.0), 0.05).unwrap();
        boundary_collision |= !collisions.is_empty();
    }
    assert!(boundary_collision);
    assert!(room_edge.position_m.x_m > -3.7 && room_edge.position_m.x_m < -3.3);
}

#[test]
fn interaction_is_rust_validated_and_idempotent_and_gate_passes_when_powered() {
    let root = isolated_dir();
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    let initial = runtime.snapshot().unwrap();
    assert_eq!(
        initial
            .actors
            .iter()
            .find(|actor| actor.entity_id == POWER_CONSOLE_ID)
            .unwrap()
            .transform
            .position_m,
        Vec3::new(2.0, 0.0, 3.0).unwrap()
    );
    let gate = initial
        .doors
        .iter()
        .find(|door| door.door_id == "gh_gate_a")
        .unwrap();
    assert_eq!(gate.transform.position_m, Vec3::new(0.0, 0.0, 8.0).unwrap());
    assert!(!gate.open && gate.locked);
    assert_eq!(
        runtime
            .interact(POWER_CONSOLE_ID, "too-far")
            .unwrap()
            .error_code
            .as_deref(),
        Some("E_INTERACTION_OUT_OF_RANGE")
    );

    let nearby = move_player(&runtime, initial, 1.0, 8);
    let first = runtime.interact(POWER_CONSOLE_ID, "power-once-a").unwrap();
    assert!(first.applied && !first.already_applied);
    assert!(first
        .view
        .doors
        .iter()
        .any(|door| door.door_id == "gh_gate_a" && door.open && !door.locked));
    assert_eq!(
        nearby.player.transform.position_m,
        first.view.player.transform.position_m
    );
    let seq = first.view.progression.event_seq;
    let repeated = runtime.interact(POWER_CONSOLE_ID, "power-once-b").unwrap();
    assert!(!repeated.applied && repeated.already_applied);
    assert_eq!(repeated.view.progression.event_seq, seq);

    let passed = move_player(&runtime, repeated.view, 1.0, 48);
    assert!(passed.player.transform.position_m.z_m > 10.0);
}

#[test]
fn corrupt_or_non_walkable_saved_state_fails_closed() {
    let root = isolated_dir();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.save().unwrap();
    let path = root.join("formal-save-v3.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    document["player"]["positionM"]["zM"] = serde_json::json!(8.0);
    std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    assert_eq!(
        runtime.continue_saved().unwrap_err(),
        "E_SAVE_NUMERIC_INVALID"
    );
    assert_eq!(
        runtime.snapshot().unwrap().player.transform.position_m,
        Vec3::zero()
    );
}

#[test]
fn power_gate_cross_process_worker() {
    let (Some(root), Some(role)) = (
        std::env::var_os("GREY_HIVE_FLOW_SAVE_ROOT"),
        std::env::var("GREY_HIVE_FLOW_SAVE_ROLE").ok(),
    ) else {
        return;
    };
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    match role.as_str() {
        "write" => {
            let start = runtime.snapshot().unwrap();
            let near = move_player(&runtime, start, 1.0, 8);
            let response = runtime
                .interact(POWER_CONSOLE_ID, "cross-process-power")
                .unwrap();
            assert!(response.applied);
            assert_eq!(
                response.view.player.transform.position_m,
                near.player.transform.position_m
            );
            runtime.save().unwrap();
        }
        "load" => {
            let restored = runtime.continue_saved().unwrap();
            assert!(restored
                .doors
                .iter()
                .any(|door| door.door_id == "gh_gate_a" && door.open));
            assert!(restored.progression.event_seq >= 2);
            assert!((restored.player.transform.position_m.z_m - 1.6).abs() < 0.02);
            let passed = move_player(&runtime, restored, 1.0, 48);
            assert!(passed.player.transform.position_m.z_m > 10.0);
        }
        other => panic!("unexpected subprocess role {other}"),
    }
}

#[test]
fn power_and_open_gate_survive_process_restart_with_collision_rebuilt() {
    let root = isolated_dir();
    let exe = std::env::current_exe().unwrap();
    for role in ["write", "load"] {
        let output = Command::new(&exe)
            .args(["--exact", "power_gate_cross_process_worker", "--nocapture"])
            .env("GREY_HIVE_FLOW_SAVE_ROOT", &root)
            .env("GREY_HIVE_FLOW_SAVE_ROLE", role)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{role} worker failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
