#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    continuous_input::InputSample,
    formal_runtime::{ActionCommandRequest, ActionKind, FormalRuntime},
    save_v6::{self, SAVE_V6_FILE_NAME},
};

const RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH_REMAINDER: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");
const ENTRY: &str = include_str!("../../content/scenes/compiled/gh_entry_maintenance.json");
const POWER: &str = include_str!("../../content/scenes/compiled/gh_power_room.json");
const GATE: &str = include_str!("../../content/scenes/compiled/gh_gate_a.json");
const SHAFT: &str = include_str!("../../content/scenes/compiled/gh_central_shaft.json");
const LOCKDOWN: &str = include_str!("../../content/scenes/compiled/gh_lockdown.json");
const BIO_ISOLATION: &str = include_str!("../../content/scenes/compiled/gh_bio_isolation.json");
const GATE_B: &str = include_str!("../../content/scenes/compiled/gh_gate_b.json");
const DEEP_DECON: &str = include_str!("../../content/scenes/compiled/gh_deep_decon.json");
const SENTINEL_ARENA: &str = include_str!("../../content/scenes/compiled/gh_sentinel_arena.json");
const BEACON: &str = include_str!("../../content/scenes/compiled/gh_beacon.json");
const EXIT: &str = include_str!("../../content/scenes/compiled/gh_exit.json");
const RUNTIME_MANIFEST: &str = include_str!("../../governance/assets/RUNTIME_ASSET_MANIFEST.json");
const ENTITY_CATALOG: &str = include_str!("../../content/enemies/entity-types.json");

static NEXT_TIME: AtomicU64 = AtomicU64::new(100_000);
static NEXT_ACTION: AtomicU64 = AtomicU64::new(1);
static LANDED_HITS: AtomicU64 = AtomicU64::new(0);

fn save_root() -> PathBuf {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("gh-first-three-{}-{token}", std::process::id()))
}

fn load_eleven_scene_registry(runtime: &FormalRuntime, start_scene: &str) {
    let manifest: serde_json::Value = serde_json::from_str(RUNTIME_MANIFEST).unwrap();
    let assets: BTreeSet<String> = manifest["assets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|asset| asset["admission"] == "release_approved")
        .map(|asset| asset["assetId"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(assets.len(), 43);
    let entities: serde_json::Value = serde_json::from_str(ENTITY_CATALOG).unwrap();
    let entities: BTreeSet<String> = entities["entityTypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entity| entity.as_str().unwrap().to_owned())
        .collect();
    runtime
        .load_scene_registry(
            [
                RS,
                GH_REMAINDER,
                MH,
                CW,
                ENTRY,
                POWER,
                GATE,
                SHAFT,
                LOCKDOWN,
                BIO_ISOLATION,
                GATE_B,
                DEEP_DECON,
                SENTINEL_ARENA,
                BEACON,
                EXIT,
            ],
            start_scene,
            &assets,
            &entities,
        )
        .unwrap();
}

fn walk_x(
    runtime: &FormalRuntime,
    mut view: wuxian_horror_ch1::world_v3::WorldView,
    target_x: f32,
) -> wuxian_horror_ch1::world_v3::WorldView {
    let direction = if target_x >= view.player.transform.position_m.x_m {
        1.0
    } else {
        -1.0
    };
    let distance = (target_x - view.player.transform.position_m.x_m).abs();
    let ticks = ((distance / 4.0 * 60.0).ceil() as u64).saturating_add(30);
    let after_event_id = runtime
        .presentation_events_since(view.world_epoch, 0)
        .unwrap()
        .last()
        .map_or(0, |event| event.event_id);
    // Keep the original bounded route and collision probes, but defend against
    // nearby authored enemies through real aimed Action-v2 attacks. Walking
    // through every attack used to hide deaths while the runtime accepted input
    // at zero HP. Nothing here changes health, enemies, collision, or progression.
    for step in 0..ticks {
        assert!(
            view.player.current_hp > 0,
            "route died in {} before walking to {target_x}",
            view.scene_id
        );
        let player = view.player.transform.position_m;
        let target = view
            .actors
            .iter()
            .filter(|actor| actor.active)
            .min_by(|left, right| {
                let distance = |actor: &wuxian_horror_ch1::world_v3::ActorView| {
                    let position = actor.transform.position_m;
                    (position.x_m - player.x_m).hypot(position.z_m - player.z_m)
                };
                distance(left).total_cmp(&distance(right))
            })
            .map(|actor| actor.transform.position_m);
        let (aim_x, aim_z, nearby) = target
            .map(|target| {
                let dx = target.x_m - player.x_m;
                let dz = target.z_m - player.z_m;
                let distance = dx.hypot(dz).max(f32::EPSILON);
                (dx / distance, dz / distance, distance <= 1.8)
            })
            .unwrap_or((direction, 0.0, false));
        view = runtime
            .submit_input(
                InputSample::new(
                    view.world_epoch,
                    view.ack_seq + 1,
                    NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                    direction,
                    0.0,
                )
                .unwrap()
                .with_aim(aim_x, aim_z)
                .unwrap(),
                vec![],
            )
            .unwrap();
        assert!(view.player.current_hp > 0, "route died in {}", view.scene_id);
        if nearby && step % 12 == 0 {
            view = runtime
                .submit_action(ActionCommandRequest {
                    protocol_version: 2,
                    world_epoch: view.world_epoch,
                    request_id: NEXT_ACTION.fetch_add(1, Ordering::Relaxed),
                    client_time_ms: NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                    kind: ActionKind::PrimaryAttack,
                })
                .unwrap();
        }
    }
    view = runtime
        .submit_input(
            InputSample::new(
                view.world_epoch,
                view.ack_seq + 1,
                NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                0.0,
                0.0,
            )
            .unwrap(),
            vec![],
        )
        .unwrap();
    assert!(view.player.current_hp > 0, "route died in {}", view.scene_id);
    let landed_hits = runtime
        .presentation_events_since(view.world_epoch, after_event_id)
        .unwrap()
        .iter()
        .filter(|event| event.kind == "Hit")
        .count() as u64;
    LANDED_HITS.fetch_add(landed_hits, Ordering::Relaxed);
    eprintln!(
        "walk {} to {target_x}: {} HP, {landed_hits} real attack hits, position {:?}",
        view.scene_id, view.player.current_hp, view.player.transform.position_m
    );
    view
}

fn run_create(root: &std::ffi::OsStr) {
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    load_eleven_scene_registry(&runtime, "gh_entry_maintenance");
    let mut view = runtime.snapshot().unwrap();
    assert_eq!(view.scene_id, "gh_entry_maintenance");

    view = walk_x(&runtime, view, 18.0);
    view = runtime
        .transition_scene(
            "gh_entry_to_power_room",
            "enter-power-room",
            view.world_epoch,
        )
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    assert_eq!(view.scene_id, "gh_power_room");

    // Approach Gate A before touching the console: it must remain a blocking,
    // locked scene door while the persisted route lacks hive_power.
    view = walk_x(&runtime, view, 18.0);
    view = runtime
        .transition_scene(
            "gh_power_to_gate_a",
            "enter-gate-unpowered",
            view.world_epoch,
        )
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    assert_eq!(view.scene_id, "gh_gate_a");
    assert!(view
        .doors
        .iter()
        .any(|door| door.door_id == "gh_gate_a" && !door.open && door.locked));
    view = walk_x(&runtime, view, 12.0);
    assert!(view.player.transform.position_m.x_m < 9.5);
    let unpowered_exit = runtime.transition_scene(
        "gh_gate_a_to_shaft",
        "reject-unpowered-gate-exit",
        view.world_epoch,
    );
    assert!(
        unpowered_exit
            .as_ref()
            .is_err_and(|error| error.contains("ProgressionLocked")),
        "unpowered Gate A exit must be authoritatively rejected: {unpowered_exit:?}"
    );
    assert_eq!(runtime.snapshot().unwrap().scene_id, "gh_gate_a");

    // Return to the Power Room. F activates the actual authored interaction;
    // its event is applied to Rust route progression before Gate A is revisited.
    view = walk_x(&runtime, view, 2.0);
    view = runtime
        .transition_scene(
            "gh_gate_a_return_to_power_room",
            "return-to-power-room",
            view.world_epoch,
        )
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap_or_else(|error| {
            panic!(
                "return transition rejected at {:?}: {error}",
                view.player.transform.position_m
            )
        });
    assert_eq!(view.scene_id, "gh_power_room");
    let powered = runtime
        .activate_scene_interaction("gh_power_console", "f-restore-main-power", view.world_epoch)
        .unwrap();
    assert!(powered.applied, "{:?}", powered.error_code);
    assert!(powered
        .view
        .objectives
        .iter()
        .any(
            |objective| objective.objective_id == "gh_restore_main_power"
                && objective.state == "complete"
        ));
    view = walk_x(&runtime, powered.view, 18.0);
    view = runtime
        .transition_scene("gh_power_to_gate_a", "enter-gate-powered", view.world_epoch)
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    assert_eq!(view.scene_id, "gh_gate_a");
    assert!(view
        .doors
        .iter()
        .any(|door| door.door_id == "gh_gate_a" && door.open && !door.locked));
    view = runtime
        .activate_scene_checkpoint(
            "gh_gate_a_checkpoint",
            "checkpoint-gate-a",
            view.world_epoch,
        )
        .unwrap();
    assert_eq!(view.checkpoint_id.as_deref(), Some("gh_gate_a_checkpoint"));
    assert!(LANDED_HITS.load(Ordering::Relaxed) > 0, "route must land real Action-v2 attacks");
    runtime.save().unwrap();
    let saved = save_v6::read_save(PathBuf::from(root).as_path()).unwrap().save;
    assert_eq!(saved.player.current_hp, view.player.current_hp);
    assert!(saved.player.current_hp > 0, "checkpoint must save a surviving player");
    assert_eq!(saved.scene_id, "gh_gate_a");
    assert_eq!(saved.checkpoint_id.as_deref(), Some("gh_gate_a_checkpoint"));
    assert!(saved
        .progression
        .progress
        .iter()
        .find(|world| world.world_id == "grey_hive")
        .unwrap()
        .completed_events
        .contains(&"hive_power".to_string()));
    assert!(PathBuf::from(root).join(SAVE_V6_FILE_NAME).is_file());
}

fn run_continue(root: &std::ffi::OsStr) {
    let saved = save_v6::read_save(PathBuf::from(root).as_path()).unwrap().save;
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    load_eleven_scene_registry(&runtime, "gh_entry_maintenance");
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.player.current_hp, saved.player.current_hp, "Continue preserves earned health");
    assert!(view.player.current_hp > 0);
    assert_eq!(view.scene_id, "gh_gate_a");
    assert_eq!(view.checkpoint_id.as_deref(), Some("gh_gate_a_checkpoint"));
    assert!(view
        .doors
        .iter()
        .any(|door| door.door_id == "gh_gate_a" && door.open && !door.locked));
    view = walk_x(&runtime, view, 12.0);
    assert!(view.player.transform.position_m.x_m > 10.5);
}

fn run_five_scene_create(root: &std::ffi::OsStr) {
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    load_eleven_scene_registry(&runtime, "gh_entry_maintenance");
    let mut view = runtime.snapshot().unwrap();
    assert_eq!(view.scene_id, "gh_entry_maintenance");

    view = walk_x(&runtime, view, 18.0);
    view = runtime
        .transition_scene(
            "gh_entry_to_power_room",
            "five-enter-power-room",
            view.world_epoch,
        )
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    let powered = runtime
        .activate_scene_interaction("gh_power_console", "five-restore-power", view.world_epoch)
        .unwrap();
    assert!(powered.applied, "{:?}", powered.error_code);
    view = walk_x(&runtime, powered.view, 18.0);
    view = runtime
        .transition_scene("gh_power_to_gate_a", "five-enter-gate-a", view.world_epoch)
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    assert!(view
        .doors
        .iter()
        .any(|door| door.door_id == "gh_gate_a" && door.open && !door.locked));
    view = walk_x(&runtime, view, 18.0);
    view = runtime
        .transition_scene("gh_gate_a_to_shaft", "five-enter-shaft", view.world_epoch)
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    assert_eq!(view.scene_id, "gh_central_shaft");

    let shaft: serde_json::Value = serde_json::from_str(SHAFT).unwrap();
    let occluders = shaft["occluders"].as_array().unwrap();
    assert!(occluders
        .iter()
        .any(|item| { item["id"] == "gh_shaft_occlusion_bridge_beam" && item["fadeTo"] == 0.2 }));
    assert!(occluders
        .iter()
        .any(|item| item["id"] == "gh_shaft_occlusion_lift_beam"));
    assert!(view.interactables.iter().any(|item| {
        item.entity_id == "gh_log_shaft_01" && item.kind == "facility_log" && item.active
    }));
    let log = runtime
        .activate_scene_interaction("gh_log_shaft_01", "read-shaft-evac-log", view.world_epoch)
        .unwrap();
    assert!(log.applied, "{:?}", log.error_code);
    assert!(log
        .view
        .interactables
        .iter()
        .any(|item| item.entity_id == "gh_log_shaft_01" && item.active));
    assert!(log.view.capabilities.items.iter().any(|item|
        item.capability_id == "information.enemy_vitals_basic" && item.granted));
    let online = runtime.activate_scene_interaction("gh_log_shaft_01", "scanner-online", log.view.world_epoch).unwrap();
    assert!(!online.applied && online.already_applied);
    assert!(online.events.is_empty());
    view = walk_x(&runtime, log.view, 22.5);
    view = runtime
        .transition_scene(
            "gh_shaft_to_lockdown",
            "five-enter-lockdown",
            view.world_epoch,
        )
        .map(|prepared| acknowledge_ready(&runtime, prepared))
        .unwrap();
    assert_eq!(view.scene_id, "gh_lockdown");

    // The unsealed bio corridor is physically blocked until the real terminal interaction.
    view = walk_x(&runtime, view, 20.0);
    assert!(
        view.player.transform.position_m.x_m < 10.0,
        "unsealed bio barrier should stop before x=10: {}",
        view.player.transform.position_m.x_m
    );
    view = walk_x(&runtime, view, 4.0);
    let lockdown = runtime
        .activate_scene_interaction(
            "gh_lockdown_terminal",
            "activate-lockdown-terminal",
            view.world_epoch,
        )
        .unwrap();
    assert!(lockdown.applied, "{:?}", lockdown.error_code);
    assert!(lockdown
        .view
        .objectives
        .iter()
        .any(|item| { item.objective_id == "gh_lockdown_objective" && item.state == "complete" }));
    view = walk_x(&runtime, lockdown.view, 14.0);
    assert!(view.player.transform.position_m.x_m > 10.5);
    view = walk_x(&runtime, view, 6.0);
    view = runtime
        .activate_scene_checkpoint(
            "gh_lockdown_checkpoint",
            "checkpoint-lockdown",
            view.world_epoch,
        )
        .unwrap();
    assert_eq!(
        view.checkpoint_id.as_deref(),
        Some("gh_lockdown_checkpoint")
    );
    assert!(LANDED_HITS.load(Ordering::Relaxed) > 0, "route must land real Action-v2 attacks");
    runtime.save().unwrap();
    let saved = save_v6::read_save(PathBuf::from(root).as_path()).unwrap().save;
    assert_eq!(saved.player.current_hp, view.player.current_hp);
    assert!(saved.player.current_hp > 0, "checkpoint must save a surviving player");
    assert_eq!(saved.scene_id, "gh_lockdown");
    assert_eq!(
        saved.checkpoint_id.as_deref(),
        Some("gh_lockdown_checkpoint")
    );
    let completed = &saved
        .progression
        .progress
        .iter()
        .find(|world| world.world_id == "grey_hive")
        .unwrap()
        .completed_events;
    assert!(completed.contains(&"hive_power".to_string()));
    assert!(completed.contains(&"hive_lockdown".to_string()));
}

fn run_five_scene_continue(root: &std::ffi::OsStr) {
    let saved = save_v6::read_save(PathBuf::from(root).as_path()).unwrap().save;
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    load_eleven_scene_registry(&runtime, "gh_entry_maintenance");
    let view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.player.current_hp, saved.player.current_hp, "Continue preserves earned health");
    assert!(view.player.current_hp > 0);
    assert_eq!(view.scene_id, "gh_lockdown");
    assert_eq!(
        view.checkpoint_id.as_deref(),
        Some("gh_lockdown_checkpoint")
    );
    assert!(view
        .objectives
        .iter()
        .any(|item| { item.objective_id == "gh_lockdown_objective" && item.state == "complete" }));
    let view = walk_x(&runtime, view, 14.0);
    assert!(
        view.player.transform.position_m.x_m > 10.5,
        "Continue must rebuild the lockdown barrier from hive_lockdown"
    );
}

#[test]
fn gh_first_three_scene_process_worker() {
    let (Some(root), Some(role)) = (
        std::env::var_os("GH_FIRST_THREE_SAVE_ROOT"),
        std::env::var("GH_FIRST_THREE_ROLE").ok(),
    ) else {
        return;
    };
    match role.as_str() {
        "create" => run_create(&root),
        "continue" => run_continue(&root),
        "create_five" => run_five_scene_create(&root),
        "continue_five" => run_five_scene_continue(&root),
        other => panic!("unknown GH first-three worker role: {other}"),
    }
}

#[test]
fn new_journey_gate_shaft_lockdown_survives_cross_process_save_v5_continue() {
    let root = save_root();
    for role in ["create_five", "continue_five"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "gh_first_three_scene_process_worker",
                "--nocapture",
            ])
            .env("GH_FIRST_THREE_SAVE_ROOT", &root)
            .env("GH_FIRST_THREE_ROLE", role)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "worker {role} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!("worker {role} verified:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn new_journey_power_gate_a_checkpoint_survives_separate_process_continue() {
    let root = save_root();
    for role in ["create", "continue"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "gh_first_three_scene_process_worker",
                "--nocapture",
            ])
            .env("GH_FIRST_THREE_SAVE_ROOT", &root)
            .env("GH_FIRST_THREE_ROLE", role)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "worker {role} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!("worker {role} verified:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    let _ = fs::remove_dir_all(root);
}
