#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;
#[path = "support/clockworks_campaign.rs"]
mod clockworks_campaign;
#[path = "support/swarm_campaign.rs"]
mod swarm_campaign;
#[path = "support/mist_harbor_campaign.rs"]
mod mist_harbor_campaign;
#[path = "support/grey_hive_beacon_campaign.rs"]
mod grey_hive_beacon_campaign;

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    continuous_input::InputSample, formal_runtime::FormalRuntime,
    production_scene_bootstrap as production, save_v6, scene_registry::WorldRegistry,
    scene_route_commands, scene_runtime::SceneRuntimeError,
};

const RUNTIME_MANIFEST: &str = include_str!("../../governance/assets/RUNTIME_ASSET_MANIFEST.json");
const ENTITY_CATALOG: &str = include_str!("../../content/enemies/entity-types.json");
const OLD_A5_GREY_HIVE_FIXTURE: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const FULL_TEST_RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const FULL_TEST_GH: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const FULL_TEST_MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const FULL_TEST_CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");
static NEXT_TIME: AtomicU64 = AtomicU64::new(500_000);
static NEXT_COMBAT_REQUEST: AtomicU64 = AtomicU64::new(1);

const SENTINEL_ENTITY_ID: &str = "gh_sentinel_arena_sentinel_01";
const SENTINEL_ENTITY_TYPE: &str = "enemy.grey_hive.sentinel";

fn catalogs() -> (BTreeSet<String>, BTreeSet<String>) {
    let runtime: serde_json::Value = serde_json::from_str(RUNTIME_MANIFEST).unwrap();
    let assets = runtime["assets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|asset| asset["admission"] == "release_approved")
        .map(|asset| asset["assetId"].as_str().unwrap().to_owned())
        .collect();
    let entities: serde_json::Value = serde_json::from_str(ENTITY_CATALOG).unwrap();
    let entities = entities["entityTypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entity| entity.as_str().unwrap().to_owned())
        .collect();
    (assets, entities)
}

fn compiled_route_scene(scene_id: &str) -> serde_json::Value {
    let raw = match scene_id {
        "gh_lockdown" => include_str!("../../content/scenes/compiled/gh_lockdown.json"),
        "gh_deep_decon" => include_str!("../../content/scenes/compiled/gh_deep_decon.json"),
        "rs_core_room" => include_str!("../../content/scenes/compiled/rs_core_room.json"),
        "mh_breakwater" => include_str!("../../content/scenes/compiled/mh_breakwater.json"),
        "mh_drowned_quay" => include_str!("../../content/scenes/compiled/mh_drowned_quay.json"),
        "mh_extraction" => include_str!("../../content/scenes/compiled/mh_extraction.json"),
        "mh_fog_pier" => include_str!("../../content/scenes/compiled/mh_fog_pier.json"),
        "mh_pump_station" => include_str!("../../content/scenes/compiled/mh_pump_station.json"),
        "mh_resonance_tower" => {
            include_str!("../../content/scenes/compiled/mh_resonance_tower.json")
        }
        "mh_signal_yard" => include_str!("../../content/scenes/compiled/mh_signal_yard.json"),
        "mh_tidal_warehouse" => {
            include_str!("../../content/scenes/compiled/mh_tidal_warehouse.json")
        }
        "mh_warden_arena" => include_str!("../../content/scenes/compiled/mh_warden_arena.json"),
        "cw_entry_foundry" => include_str!("../../content/scenes/compiled/cw_entry_foundry.json"),
        "cw_pressure_hall" => include_str!("../../content/scenes/compiled/cw_pressure_hall.json"),
        "cw_conveyor_bridge" => include_str!("../../content/scenes/compiled/cw_conveyor_bridge.json"),
        "cw_boiler_chamber" => include_str!("../../content/scenes/compiled/cw_boiler_chamber.json"),
        "cw_gear_shaft" => include_str!("../../content/scenes/compiled/cw_gear_shaft.json"),
        "cw_furnace_heart" => include_str!("../../content/scenes/compiled/cw_furnace_heart.json"),
        "cw_forged_guard_arena" => include_str!("../../content/scenes/compiled/cw_forged_guard_arena.json"),
        "cw_regulator_core" => include_str!("../../content/scenes/compiled/cw_regulator_core.json"),
        "cw_shutdown_exit" => include_str!("../../content/scenes/compiled/cw_shutdown_exit.json"),
        _ => panic!("unknown compiled route scene {scene_id}"),
    };
    serde_json::from_str(raw).expect("compiled route scene JSON is valid")
}

fn walk_to_compiled_target(
    runtime: &FormalRuntime,
    mut view: wuxian_horror_ch1::world_v3::WorldView,
    scene_id: &str,
    collection: &str,
    target_id: &str,
) -> wuxian_horror_ch1::world_v3::WorldView {
    const INTERACTION_RANGE_M: f32 = 2.5;
    const ARRIVAL_MARGIN_M: f32 = 0.2;
    const MAX_INPUT_STEPS: usize = 600;

    let scene = compiled_route_scene(scene_id);
    let target = scene[collection]
        .as_array()
        .unwrap_or_else(|| panic!("{scene_id}.{collection} is absent"))
        .iter()
        .find(|entry| entry["id"] == target_id)
        .unwrap_or_else(|| panic!("{scene_id}.{collection} lacks {target_id}"));
    let arrival_distance = if collection == "transitions" {
        ARRIVAL_MARGIN_M
    } else {
        INTERACTION_RANGE_M - ARRIVAL_MARGIN_M
    };
    let (target_x, target_z) = if collection == "transitions" {
        let polygon = target["polygon"].as_array().expect("transition polygon");
        let (x, z) = polygon.iter().fold((0.0_f32, 0.0_f32), |(x, z), point| {
            (
                x + point[0].as_f64().unwrap() as f32,
                z + point[1].as_f64().unwrap() as f32,
            )
        });
        (x / polygon.len() as f32, z / polygon.len() as f32)
    } else {
        let position = target["position"].as_array().expect("interaction position");
        (
            position[0].as_f64().unwrap() as f32,
            position[2].as_f64().unwrap() as f32,
        )
    };

    for _ in 0..MAX_INPUT_STEPS {
        let player = view.player.transform.position_m;
        let dx = target_x - player.x_m;
        let dz = target_z - player.z_m;
        let distance = dx.hypot(dz);
        if distance <= arrival_distance {
            return view;
        }
        let length = distance.max(f32::EPSILON);
        view = runtime
            .submit_input(
                InputSample::new(
                    view.world_epoch,
                    view.ack_seq + 1,
                    NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                    dx / length,
                    dz / length,
                )
                .unwrap(),
                vec![],
            )
            .unwrap();
    }

    let player = view.player.transform.position_m;
    let distance = (target_x - player.x_m).hypot(target_z - player.z_m);
    assert!(
        distance <= arrival_distance,
        "could not reach {scene_id}.{collection}.{target_id} at target range {arrival_distance}m (interaction max {INTERACTION_RANGE_M}m): player=({}, {}), target=({}, {}), distance={distance}m",
        player.x_m,
        player.z_m,
        target_x,
        target_z
    );
    view
}

fn save_root() -> PathBuf {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "production-scene-bootstrap-{}-{token}",
        std::process::id()
    ))
}

fn clockworks_shaped_save(save: save_v6::SaveV6, scene_id: &str) -> save_v6::SaveV6 {
    let mut value = serde_json::to_value(save).unwrap();
    value["save"]["worldId"] = serde_json::json!("clockworks");
    value["save"]["sceneId"] = serde_json::json!(scene_id);
    value["save"]["checkpointId"] = serde_json::Value::Null;
    value["save"]["progression"]["currentWorldId"] = serde_json::json!("clockworks");
    value["save"]["capabilities"]["worldId"] = serde_json::json!("clockworks");
    value["save"]["explored"]["worldId"] = serde_json::json!("clockworks");
    value["save"]["player"]["positionM"] = serde_json::json!({"xM": 1.0, "yM": 0.0, "zM": 1.0});
    value["save"]["sceneStates"] = if scene_id == "clockworks" {
        serde_json::json!([])
    } else {
        serde_json::json!([{
            "sceneId": scene_id,
            "activatedIds": [],
            "emittedEventIds": [],
            "checkpointId": null,
        }])
    };
    let progress = value["save"]["progression"]["progress"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["worldId"] == "clockworks")
        .unwrap();
    progress["completedEvents"] = serde_json::json!([
        "clockworks_valves",
        "clockworks_core",
        "clockworks_shutdown",
    ]);
    progress["completed"] = serde_json::json!(true);
    progress["firstCompletion"] = serde_json::json!(true);
    let save: save_v6::SaveV6 = serde_json::from_value(value).unwrap();
    save.validate().unwrap();
    save
}

fn assert_same_runtime_snapshot(
    before: &wuxian_horror_ch1::world_v3::WorldView,
    after: &wuxian_horror_ch1::world_v3::WorldView,
) {
    assert_eq!(after.world_id, before.world_id);
    assert_eq!(after.scene_id, before.scene_id);
    assert_eq!(after.world_epoch, before.world_epoch);
    assert_eq!(after.authority_revision, before.authority_revision);
    assert_eq!(after.progression, before.progression);
    assert_eq!(after.capabilities, before.capabilities);
    assert_eq!(after.interactables, before.interactables);
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
    for _ in 0..ticks {
        view = runtime
            .submit_input(
                InputSample::new(
                    view.world_epoch,
                    view.ack_seq + 1,
                    NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                    direction,
                    0.0,
                )
                .unwrap(),
                vec![],
            )
            .unwrap();
    }
    runtime
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
        .unwrap()
}

fn walk_z(
    runtime: &FormalRuntime,
    mut view: wuxian_horror_ch1::world_v3::WorldView,
    target_z: f32,
) -> wuxian_horror_ch1::world_v3::WorldView {
    let direction = if target_z >= view.player.transform.position_m.z_m {
        1.0
    } else {
        -1.0
    };
    let distance = (target_z - view.player.transform.position_m.z_m).abs();
    let ticks = ((distance / 4.0 * 60.0).ceil() as u64).saturating_add(30);
    for _ in 0..ticks {
        view = runtime
            .submit_input(
                InputSample::new(
                    view.world_epoch,
                    view.ack_seq + 1,
                    NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                    0.0,
                    direction,
                )
                .unwrap(),
                vec![],
            )
            .unwrap();
    }
    runtime
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
        .unwrap()
}

fn walk_to(
    runtime: &FormalRuntime,
    view: wuxian_horror_ch1::world_v3::WorldView,
    x: f32,
    z: f32,
) -> wuxian_horror_ch1::world_v3::WorldView {
    let view = walk_x(runtime, view, x);
    walk_z(runtime, view, z)
}

fn assert_registered_sentinel(view: &wuxian_horror_ch1::world_v3::WorldView, active: bool) {
    let sentinels: Vec<_> = view
        .actors
        .iter()
        .filter(|actor| actor.entity_type == SENTINEL_ENTITY_TYPE)
        .collect();
    assert_eq!(sentinels.len(), 1, "expected one registered Sentinel actor");
    assert_eq!(sentinels[0].entity_id, SENTINEL_ENTITY_ID);
    assert_eq!(sentinels[0].actor_kind, "sentinel");
    assert_eq!(sentinels[0].active, active);
}

fn kill_registered_sentinel_with_input(
    runtime: &FormalRuntime,
    mut view: wuxian_horror_ch1::world_v3::WorldView,
) -> wuxian_horror_ch1::world_v3::WorldView {
    const ATTACK_RANGE_M: f32 = 1.5;
    const MAX_INPUT_STEPS: usize = 600;

    assert_eq!(view.scene_id, "gh_sentinel_arena");
    assert_registered_sentinel(&view, true);
    for _ in 0..MAX_INPUT_STEPS {
        let sentinel = view
            .actors
            .iter()
            .find(|actor| actor.entity_type == SENTINEL_ENTITY_TYPE)
            .expect("registered Sentinel actor remains in the arena snapshot");
        if !sentinel.active {
            assert_registered_sentinel(&view, false);
            return view;
        }

        let player = view.player.transform.position_m;
        let target = sentinel.transform.position_m;
        let dx = target.x_m - player.x_m;
        let dz = target.z_m - player.z_m;
        let distance = dx.hypot(dz);
        let (move_x, move_z) = if distance > ATTACK_RANGE_M {
            let length = distance.max(f32::EPSILON);
            (dx / length, dz / length)
        } else {
            (0.0, 0.0)
        };
        let aim = if distance > f32::EPSILON {
            (dx / distance, dz / distance)
        } else {
            (0.0, 0.0)
        };
        let sample = InputSample::new(
            view.world_epoch,
            view.ack_seq + 1,
            NEXT_TIME.fetch_add(17, Ordering::Relaxed),
            move_x,
            move_z,
        )
        .unwrap()
        .with_aim(aim.0, aim.1)
        .unwrap();
        let combat = if distance <= ATTACK_RANGE_M {
            vec![
                wuxian_horror_ch1::formal_runtime::CombatIntentRequest::Attack {
                    request_id: NEXT_COMBAT_REQUEST.fetch_add(1, Ordering::Relaxed),
                },
            ]
        } else {
            vec![]
        };
        view = runtime.submit_input(sample, combat).unwrap();
    }

    panic!("public input/combat requests did not kill the registered Sentinel in the arena");
}

fn assert_registered_warden(view: &wuxian_horror_ch1::world_v3::WorldView, active: bool) {
    assert_eq!(view.scene_id, "mh_warden_arena");
    let wardens: Vec<_> = view.actors.iter().filter(|actor|
        actor.entity_id == "mh_resonance_warden_staged_marker").collect();
    assert_eq!(wardens.len(), 1);
    assert_eq!(wardens[0].entity_id, "mh_resonance_warden_staged_marker");
    assert_eq!(wardens[0].entity_type, "runtime2d.enemy.mistharbor.signal_wraith.v2");
    if let Some(boss) = &view.boss_encounter {
        assert_eq!(boss.entity_type, "enemy.mist_harbor.resonance_warden");
    }
    assert_eq!(wardens[0].active, active);
    assert_eq!(view.boss_encounter.is_some(), active);
}

fn kill_registered_warden_with_public_actions(
    runtime: &FormalRuntime,
    mut view: wuxian_horror_ch1::world_v3::WorldView,
) -> wuxian_horror_ch1::world_v3::WorldView {
    use wuxian_horror_ch1::formal_runtime::{ActionCommandRequest, ActionKind};
    assert_registered_warden(&view, true);
    // The authored tower signal legitimately grants Acoustic Mapping. Deselect
    // it through the public command so this fight proves it is not required.
    view = runtime.apply_capability_command(
        wuxian_horror_ch1::formal_runtime::CapabilityCommandRequest::Select {
            capability_ids: vec![wuxian_horror_ch1::capability_v1::CAP_REAR_VIEW.into()],
        }).unwrap();
    assert!(view.capabilities.rear_view.granted);
    assert_eq!(view.capabilities.acoustic_mapping_authorized, Some(false));
    assert_eq!(view.capabilities.map_topology_authorized, Some(false));
    assert_eq!(view.capabilities.items.iter().filter(|item| item.selected)
        .map(|item| item.capability_id.as_str()).collect::<Vec<_>>(),
        vec![wuxian_horror_ch1::capability_v1::CAP_REAR_VIEW],
        "the fight selects only the genuinely earned RearView; no map or acoustic assistance");
    let progression_before = view.progression.clone();
    let mut last_dodge_serial = None;
    let mut visible_warnings = 0;
    let mut attacks = 0;
    let mut dodges = 0;
    let mut observed_real_dash = false;
    for step in 0..3000 {
        observed_real_dash |= view.player.action_state == "dash";
        assert!(view.player.current_hp > 0, "Warden route player died at input {step}");
        let Some(boss) = view.boss_encounter.clone() else {
            assert_registered_warden(&view, false);
            assert_eq!(view.progression, progression_before, "a local Boss defeat cannot invent progression events");
            let cues = runtime.sound_cues_since(view.world_epoch, 0).unwrap();
            assert!(cues.iter().any(|cue| cue.kind == "warden_true_call"));
            assert!(cues.iter().filter(|cue| cue.kind == "warden_true_call")
                .all(|cue| cue.direction_rad.is_none() && cue.distance_m.is_none()),
                "ordinary audio must not leak mapping-only source coordinates");
            let deaths = runtime.presentation_events_since(view.world_epoch, 0).unwrap()
                .iter().filter(|event| event.kind == "ResonanceWardenDeath").count();
            assert_eq!(deaths, 1);
            assert!(visible_warnings > 0 && attacks > 0 && dodges > 0 && observed_real_dash);
            println!("Warden real combat: {step} inputs, {attacks} attack requests, {dodges} dash requests, {} HP remaining", view.player.current_hp);
            return view;
        };
        assert!(!boss.mapped_true_source);
        let actor = view.actors.iter().find(|actor|
            actor.entity_id == "mh_resonance_warden_staged_marker").unwrap();
        let player = view.player.transform.position_m;
        let target = actor.transform.position_m;
        let dx = target.x_m - player.x_m;
        let dz = target.z_m - player.z_m;
        let distance = dx.hypot(dz).max(f32::EPSILON);
        let aim = (dx / distance, dz / distance);
        let mut movement = if distance > 1.45 { aim } else { (0.0, 0.0) };
        let mut action = None;
        if let Some(warning) = boss.warning.as_ref().filter(|warning| warning.ordinary_visible && warning.kind != "decoy") {
            visible_warnings += 1;
            // React only to the ordinary current warning. No private controller,
            // pre-warning bearings, injected HP, capabilities, or defeat flags.
            movement = if warning.kind == "pulse" {
                (-aim.0, -aim.1)
            } else {
                (aim.1, -aim.0)
            };
            if warning.remaining_ms <= 400 && last_dodge_serial != Some(warning.attack_serial) {
                last_dodge_serial = Some(warning.attack_serial);
                action = Some(ActionKind::Dash);
                dodges += 1;
            }
        } else if distance <= 1.65 && step % 12 == 0 {
            action = Some(ActionKind::PrimaryAttack);
            attacks += 1;
        }
        view = runtime.submit_input(
            InputSample::new(view.world_epoch, view.ack_seq + 1,
                NEXT_TIME.fetch_add(17, Ordering::Relaxed), movement.0, movement.1)
                .unwrap().with_aim(aim.0, aim.1).unwrap(), vec![]).unwrap();
        if let Some(kind) = action {
            view = runtime.submit_action(ActionCommandRequest {
                protocol_version: 2, world_epoch: view.world_epoch,
                request_id: NEXT_COMBAT_REQUEST.fetch_add(1, Ordering::Relaxed),
                client_time_ms: NEXT_TIME.fetch_add(17, Ordering::Relaxed), kind,
            }).unwrap();
        }
    }
    panic!("real Warden combat exceeded 3000 public input steps");
}

fn enter_grey_hive(
    runtime: &FormalRuntime,
    view: wuxian_horror_ch1::world_v3::WorldView,
    request_prefix: &str,
) -> wuxian_horror_ch1::world_v3::WorldView {
    assert_eq!(view.scene_id, "rs_core_room");
    let view = walk_x(runtime, view, 18.0);
    let receipt = scene_route_commands::world_gate(
        runtime,
        "rs_world_gate_to_gh",
        &format!("{request_prefix}-rs-to-gh"),
        view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&receipt, &format!("{request_prefix}-rs-to-gh"), true);
    assert_eq!(receipt.snapshot.scene_id, "gh_entry_maintenance");
    acknowledge_ready(runtime, runtime.snapshot().unwrap())
}

#[test]
fn production_startup_and_new_begin_in_return_station_then_enter_grey_hive() {
    let runtime = FormalRuntime::new_with_save_dir(save_root()).unwrap();
    let startup = production::install(&runtime).unwrap();
    assert_eq!(startup.scene_id, "rs_core_room");
    assert_eq!(startup.world_id, "return_station");
    let gate = startup
        .interactables
        .iter()
        .find(|item| item.entity_id == "rs_world_gate_marker")
        .unwrap();
    assert_eq!(gate.kind, "world_gate");
    assert!(!startup
        .interactables
        .iter()
        .any(|item| item.entity_id == "rs_mh_world_gate_marker"));
    let locked_mh = scene_route_commands::world_gate(
        &runtime,
        "rs_world_gate_to_mh",
        "mh-gate-before-grey-clear",
        startup.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&locked_mh, "mh-gate-before-grey-clear", false);
    assert_eq!(
        locked_mh.error_code.as_deref(),
        Some("E_WORLD_GATE_GH_EXTRACTION_REQUIRED")
    );
    assert_eq!(locked_mh.snapshot.scene_id, "rs_core_room");
    assert_eq!(locked_mh.snapshot.world_epoch, startup.world_epoch);
    let mission_terminal = startup
        .interactables
        .iter()
        .find(|item| item.entity_id == "rs_mission_terminal_marker")
        .unwrap();
    assert_eq!(mission_terminal.kind, "mission_terminal");
    assert!(mission_terminal.active);
    let capability_terminal = startup
        .interactables
        .iter()
        .find(|item| item.entity_id == "rs_capability_terminal_marker")
        .unwrap();
    assert_eq!(capability_terminal.kind, "capability_terminal");
    assert!(capability_terminal.active);

    let new_journey = acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    assert_eq!(new_journey.scene_id, "rs_core_room");
    assert_eq!(new_journey.world_id, "return_station");
    assert!(new_journey
        .interactables
        .iter()
        .any(|item| item.entity_id == "rs_mission_terminal_marker"
            && item.kind == "mission_terminal"));
    assert!(!new_journey
        .interactables
        .iter()
        .any(|item| item.kind == "world_gate" && item.entity_id != "rs_world_gate_marker"));
    let static_marker = runtime
        .activate_scene_interaction(
            "rs_mission_terminal_marker",
            "static-rs-terminal-marker",
            new_journey.world_epoch,
        )
        .unwrap();
    assert!(!static_marker.applied);
    assert!(static_marker
        .error_code
        .as_deref()
        .unwrap()
        .contains("StaticMarker"));
    assert!(runtime
        .snapshot()
        .unwrap()
        .interactables
        .iter()
        .any(|item| { item.entity_id == "rs_mission_terminal_marker" && item.active }));

    let unavailable_return = scene_route_commands::world_gate(
        &runtime,
        "gh_extraction_return_to_rs",
        "gate-return-too-early",
        new_journey.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&unavailable_return, "gate-return-too-early", false);
    assert_eq!(
        unavailable_return.error_code.as_deref(),
        Some("E_WORLD_GATE_UNAVAILABLE")
    );

    let out_of_range = scene_route_commands::world_gate(
        &runtime,
        "rs_world_gate_to_gh",
        "gate-out-of-range",
        new_journey.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&out_of_range, "gate-out-of-range", false);
    assert_eq!(
        out_of_range.error_code.as_deref(),
        Some("E_WORLD_GATE_OUT_OF_RANGE")
    );

    let at_gate = walk_x(&runtime, new_journey, 18.0);
    let entered = scene_route_commands::world_gate(
        &runtime,
        "rs_world_gate_to_gh",
        "gate-enter-grey-hive",
        at_gate.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&entered, "gate-enter-grey-hive", true);
    acknowledge_ready(&runtime, runtime.snapshot().unwrap());
    assert_eq!(entered.snapshot.world_id, "grey_hive");
    assert_eq!(entered.snapshot.scene_id, "gh_entry_maintenance");
    assert_eq!(entered.snapshot.world_epoch, at_gate.world_epoch + 1);

    let replay = scene_route_commands::world_gate(
        &runtime,
        "rs_world_gate_to_gh",
        "gate-enter-grey-hive",
        at_gate.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&replay, "gate-enter-grey-hive", false);
    assert_eq!(replay.snapshot.scene_id, "gh_entry_maintenance");
    assert!(replay
        .error_code
        .as_deref()
        .unwrap()
        .contains("STALE_EPOCH"));
}

#[test]
fn production_scene_route_commands_are_v3_receipted_and_fail_closed() {
    let runtime = FormalRuntime::new_with_save_dir(save_root()).unwrap();
    production::install(&runtime).unwrap();
    let initial = enter_grey_hive(
        &runtime,
        acknowledge_ready(&runtime, runtime.reset_new().unwrap()),
        "route-command-start",
    );
    let entry_route = initial
        .interactables
        .iter()
        .find(|item| item.entity_id == "gh_entry_to_power_room")
        .expect("entry route is projected before activation");
    assert_eq!(entry_route.kind, "scene_transition");
    assert!(!entry_route.active);
    let checkpoint = scene_route_commands::checkpoint(
        &runtime,
        "gh_entry_checkpoint",
        "route-entry-checkpoint",
        initial.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&checkpoint, "route-entry-checkpoint", true);
    assert_eq!(
        checkpoint.snapshot.checkpoint_id.as_deref(),
        Some("gh_entry_checkpoint")
    );
    let duplicate = scene_route_commands::checkpoint(
        &runtime,
        "gh_entry_checkpoint",
        "route-entry-checkpoint",
        initial.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&duplicate, "route-entry-checkpoint", false);
    assert!(duplicate
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("DuplicateRequest")));

    runtime.pause().unwrap();
    let paused = scene_route_commands::transition(
        &runtime,
        "gh_entry_to_power_room",
        "route-paused-transition",
        initial.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&paused, "route-paused-transition", false);
    assert_eq!(paused.error_code.as_deref(), Some("E_RUNTIME_PAUSED"));
    assert_eq!(paused.snapshot.scene_id, "gh_entry_maintenance");
    let paused_checkpoint = scene_route_commands::checkpoint(
        &runtime,
        "gh_entry_checkpoint",
        "route-paused-checkpoint",
        initial.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&paused_checkpoint, "route-paused-checkpoint", false);
    assert_eq!(
        paused_checkpoint.error_code.as_deref(),
        Some("E_RUNTIME_PAUSED")
    );
    runtime.resume().unwrap();

    let stale = scene_route_commands::transition(
        &runtime,
        "gh_entry_to_power_room",
        "route-stale-epoch",
        initial.world_epoch + 1,
    )
    .unwrap();
    assert_v3_receipt(&stale, "route-stale-epoch", false);
    assert!(stale
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("StaleEpoch")));
    let unknown = scene_route_commands::transition(
        &runtime,
        "browser_forged_scene_route",
        "route-unknown-id",
        initial.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&unknown, "route-unknown-id", false);
    assert!(unknown
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("UnsafeTransition")));
    let far = walk_x(&runtime, runtime.snapshot().unwrap(), 18.0);
    let out_of_range_checkpoint = scene_route_commands::checkpoint(
        &runtime,
        "gh_entry_checkpoint",
        "route-checkpoint-out-of-range",
        far.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(
        &out_of_range_checkpoint,
        "route-checkpoint-out-of-range",
        false,
    );
    assert!(out_of_range_checkpoint
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("OutOfRange")));

    let journey = run_route_journey(&runtime, "route-journey");
    assert_eq!(journey.scene_id, "gh_lockdown");
    assert_eq!(
        journey.checkpoint_id.as_deref(),
        Some("gh_lockdown_checkpoint")
    );
}

fn assert_v3_receipt(
    receipt: &wuxian_horror_ch1::world_v3::CommandReceipt,
    command_id: &str,
    applied: bool,
) {
    assert_eq!(receipt.command_id, command_id);
    assert_eq!(
        receipt.applied, applied,
        "command {command_id} receipt error: {:?}",
        receipt.error_code
    );
    assert!(!receipt.already_applied);
    assert_eq!(receipt.snapshot.protocol_version, 3);
    assert_eq!(receipt.snapshot.schema_version, "freeze-v02-interfaces/1.2");
    assert_eq!(receipt.world_epoch, receipt.snapshot.world_epoch);
    assert_eq!(receipt.server_tick, receipt.snapshot.server_tick);
    assert_eq!(
        receipt.authority_revision,
        receipt.snapshot.authority_revision
    );
    let serialized = serde_json::to_value(receipt).unwrap();
    assert!(serialized.get("commandId").is_some());
    assert!(serialized.get("worldEpoch").is_some());
    assert!(serialized.get("snapshot").is_some());
}

fn run_route_journey(
    runtime: &FormalRuntime,
    request_prefix: &str,
) -> wuxian_horror_ch1::world_v3::WorldView {
    let mut view = enter_grey_hive(
        runtime,
        acknowledge_ready(&runtime, runtime.reset_new().unwrap()),
        &format!("{request_prefix}-station-entry"),
    );
    view = walk_x(runtime, view, 18.0);
    let entry_route = view
        .interactables
        .iter()
        .find(|item| item.entity_id == "gh_entry_to_power_room")
        .expect("entry transition projected");
    assert_eq!(entry_route.kind, "scene_transition");
    assert!(entry_route.active, "route is active only inside its gate");
    runtime.pause().unwrap();
    let paused_entry_route = runtime
        .snapshot()
        .unwrap()
        .interactables
        .into_iter()
        .find(|item| item.entity_id == "gh_entry_to_power_room")
        .unwrap();
    assert!(!paused_entry_route.active);
    runtime.resume().unwrap();
    view = apply_transition(
        runtime,
        "gh_entry_to_power_room",
        &format!("{request_prefix}-entry-to-power"),
        view.world_epoch,
    );
    assert_eq!(view.scene_id, "gh_power_room");

    // Gate A remains a valid authored scene, but its exit cannot bypass the
    // power event. Returning to Power and using the existing F interaction is
    // still the only way to unlock the forward route.
    view = walk_x(runtime, view, 18.0);
    view = apply_transition(
        runtime,
        "gh_power_to_gate_a",
        &format!("{request_prefix}-power-to-gate-first"),
        view.world_epoch,
    );
    assert_eq!(view.scene_id, "gh_gate_a");
    let locked = scene_route_commands::transition(
        runtime,
        "gh_gate_a_to_shaft",
        &format!("{request_prefix}-gate-unpowered"),
        view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&locked, &format!("{request_prefix}-gate-unpowered"), false);
    assert!(locked
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("ProgressionLocked")));
    view = apply_transition(
        runtime,
        "gh_gate_a_return_to_power_room",
        &format!("{request_prefix}-gate-back-to-power"),
        view.world_epoch,
    );
    let powered = runtime
        .activate_scene_interaction(
            "gh_power_console",
            &format!("{request_prefix}-restore-power"),
            view.world_epoch,
        )
        .unwrap();
    assert!(powered.applied, "{:?}", powered.error_code);
    view = walk_x(runtime, powered.view, 18.0);
    view = apply_transition(
        runtime,
        "gh_power_to_gate_a",
        &format!("{request_prefix}-power-to-gate-powered"),
        view.world_epoch,
    );
    let gate_checkpoint = scene_route_commands::checkpoint(
        runtime,
        "gh_gate_a_checkpoint",
        &format!("{request_prefix}-gate-checkpoint"),
        view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(
        &gate_checkpoint,
        &format!("{request_prefix}-gate-checkpoint"),
        true,
    );
    view = walk_x(runtime, runtime.snapshot().unwrap(), 18.0);
    view = apply_transition(
        runtime,
        "gh_gate_a_to_shaft",
        &format!("{request_prefix}-gate-to-shaft"),
        view.world_epoch,
    );
    assert_eq!(view.scene_id, "gh_central_shaft");
    view = walk_x(runtime, view, 22.5);
    view = apply_transition(
        runtime,
        "gh_shaft_to_lockdown",
        &format!("{request_prefix}-shaft-to-lockdown"),
        view.world_epoch,
    );
    assert_eq!(view.scene_id, "gh_lockdown");
    view = walk_x(runtime, view, 6.0);
    let checkpoint = scene_route_commands::checkpoint(
        runtime,
        "gh_lockdown_checkpoint",
        &format!("{request_prefix}-lockdown-checkpoint"),
        view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(
        &checkpoint,
        &format!("{request_prefix}-lockdown-checkpoint"),
        true,
    );
    runtime.snapshot().unwrap()
}

fn apply_transition(
    runtime: &FormalRuntime,
    id: &str,
    request_id: &str,
    world_epoch: u64,
) -> wuxian_horror_ch1::world_v3::WorldView {
    let receipt = scene_route_commands::transition(runtime, id, request_id, world_epoch).unwrap();
    assert_v3_receipt(&receipt, request_id, true);
    acknowledge_ready(runtime, runtime.snapshot().unwrap())
}

#[test]
fn production_route_reaches_exit_and_records_extraction_after_lockdown() {
    let save_dir = save_root();
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = run_route_journey(&runtime, "seven-scene-route");
    assert_eq!(view.scene_id, "gh_lockdown");

    let locked = scene_route_commands::transition(
        &runtime,
        "gh_lockdown_to_bio_isolation",
        "seven-scene-locked",
        view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&locked, "seven-scene-locked", false);

    view = walk_x(&runtime, view, 4.0);
    let activated = runtime
        .activate_scene_interaction(
            "gh_lockdown_terminal",
            "seven-scene-clear-lockdown",
            view.world_epoch,
        )
        .unwrap();
    assert!(activated.applied, "{:?}", activated.error_code);
    // Optional fights follow the existing terminal, which opens the central seal.
    // Defeating either Swarm group is not a new gameplay progression gate.
    view = swarm_campaign::clear_scene(&runtime, activated.view, &save_dir);
    view = walk_to_compiled_target(&runtime, view, "gh_lockdown", "transitions", "gh_lockdown_to_bio_isolation");
    view = apply_transition(
        &runtime,
        "gh_lockdown_to_bio_isolation",
        "seven-scene-to-bio",
        view.world_epoch,
    );
    assert_eq!(view.scene_id, "gh_bio_isolation");
    view = walk_x(&runtime, view, 22.5);
    view = apply_transition(
        &runtime,
        "gh_bio_to_gate_b",
        "seven-scene-to-gate-b",
        view.world_epoch,
    );
    assert_eq!(view.scene_id, "gh_gate_b");
    assert!(view
        .doors
        .iter()
        .any(|door| door.door_id == "gh_gate_b" && door.open));

    view = clockworks_campaign::clear_gate_b_enemies(&runtime, view);
    assert!(view.player.current_hp>0);
    runtime.save().unwrap();
    let brute_save=save_v6::read_save(&save_dir).unwrap();
    let brute=brute_save.save.generic_actors.iter().find(|actor|actor.entity_id=="gh_gate_b_brute_01").unwrap();
    assert_eq!(brute.hp,0);assert_eq!(brute.state,wuxian_horror_ch1::world_v3::ActorAiState::Dead);
    let continued_runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&continued_runtime).unwrap();
    let mut continued = acknowledge_ready(&continued_runtime, continued_runtime.continue_saved().unwrap());
    assert_eq!(continued.scene_id, "gh_gate_b");
    assert!(continued.actors.iter().any(|actor|actor.entity_id=="gh_gate_b_brute_01" && !actor.active));
    assert!(!continued_runtime.presentation_events_since(continued.world_epoch,0).unwrap().iter()
        .any(|event|event.kind=="EnemyDeath" && event.actor_id.as_deref()==Some("gh_gate_b_brute_01")),
        "Continue cannot re-emit the genuine Brute defeat");

    continued = walk_x(&continued_runtime, continued, 22.5);
    continued = apply_transition(
        &continued_runtime,
        "gh_gate_b_to_deep_decon",
        "eleven-scene-to-decon",
        continued.world_epoch,
    );
    assert_eq!(continued.scene_id, "gh_deep_decon");
    continued = swarm_campaign::clear_scene(&continued_runtime, continued, &save_dir);
    continued = walk_to_compiled_target(&continued_runtime, continued, "gh_deep_decon", "transitions", "gh_deep_decon_to_sentinel_arena");
    continued = apply_transition(
        &continued_runtime,
        "gh_deep_decon_to_sentinel_arena",
        "eleven-scene-to-arena",
        continued.world_epoch,
    );
    assert_eq!(continued.scene_id, "gh_sentinel_arena");
    assert_registered_sentinel(&continued, true);
    continued = walk_x(&continued_runtime, continued, 22.5);
    let arena_before_locked_beacon = continued_runtime.snapshot().unwrap();
    let locked_beacon = scene_route_commands::transition(
        &continued_runtime,
        "gh_sentinel_arena_to_beacon",
        "eleven-scene-beacon-before-sentinel-kill",
        continued.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(
        &locked_beacon,
        "eleven-scene-beacon-before-sentinel-kill",
        false,
    );
    assert_eq!(
        locked_beacon.error_code.as_deref(),
        Some("E_SCENE_SENTINEL_FIRST_KILL_REQUIRED")
    );
    assert_eq!(locked_beacon.snapshot.scene_id, "gh_sentinel_arena");
    assert_eq!(
        locked_beacon.snapshot.world_epoch,
        arena_before_locked_beacon.world_epoch
    );
    assert_eq!(
        locked_beacon.snapshot.player.transform.position_m,
        arena_before_locked_beacon.player.transform.position_m
    );
    assert_eq!(
        locked_beacon.snapshot.progression,
        arena_before_locked_beacon.progression
    );
    assert_registered_sentinel(&arena_before_locked_beacon, true);
    assert_registered_sentinel(&continued_runtime.snapshot().unwrap(), true);

    continued = kill_registered_sentinel_with_input(&continued_runtime, continued);
    assert_registered_sentinel(&continued, false);
    continued_runtime.save().unwrap();
    drop(continued_runtime);

    let continued_runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&continued_runtime).unwrap();
    let mut continued = acknowledge_ready(&continued_runtime, continued_runtime.continue_saved().unwrap());
    assert_eq!(continued.scene_id, "gh_sentinel_arena");
    assert_registered_sentinel(&continued, false);
    continued = walk_x(&continued_runtime, continued, 22.5);
    continued = apply_transition(
        &continued_runtime,
        "gh_sentinel_arena_to_beacon",
        "eleven-scene-to-beacon",
        continued.world_epoch,
    );
    assert_eq!(continued.scene_id, "gh_beacon");
    continued = grey_hive_beacon_campaign::collect_and_mount(&continued_runtime, continued, &save_dir);
    continued = walk_x(&continued_runtime, continued, 22.5);
    continued = apply_transition(
        &continued_runtime,
        "gh_beacon_to_exit",
        "eleven-scene-to-exit",
        continued.world_epoch,
    );
    assert_eq!(continued.scene_id, "gh_exit");
    continued = walk_x(&continued_runtime, continued, 6.0);
    let extraction = continued_runtime
        .activate_scene_interaction(
            "gh_exit_extraction_console",
            "eleven-scene-extraction",
            continued.world_epoch,
        )
        .unwrap();
    assert!(extraction.applied, "{:?}", extraction.error_code);
    let gh_progress = extraction
        .view
        .progression
        .worlds
        .iter()
        .find(|world| world.world_id == "grey_hive")
        .unwrap();
    assert!(gh_progress.completed);
    let spent_extraction_console = extraction
        .view
        .interactables
        .iter()
        .find(|item| item.entity_id == "gh_exit_extraction_console")
        .unwrap();
    assert!(!spent_extraction_console.active);
    let return_gate = extraction
        .view
        .interactables
        .iter()
        .find(|item| item.entity_id == "gh_extraction_return_to_rs")
        .unwrap();
    assert_eq!(return_gate.kind, "world_gate");
    assert!(return_gate.active);
    continued_runtime.pause().unwrap();
    let chosen = continued_runtime.choose_first_enhancement(wuxian_horror_ch1::capability_v1::CAP_REAR_VIEW).unwrap();
    assert!(chosen.capabilities.rear_view.granted);
    continued_runtime.resume().unwrap();
    let returned = scene_route_commands::world_gate(
        &continued_runtime,
        "gh_extraction_return_to_rs",
        "eleven-scene-return-to-station",
        extraction.view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&returned, "eleven-scene-return-to-station", true);
    acknowledge_ready(&continued_runtime, continued_runtime.snapshot().unwrap());
    assert_eq!(returned.snapshot.world_id, "return_station");
    assert_eq!(returned.snapshot.scene_id, "rs_core_room");
    continued_runtime.save().unwrap();
    let saved = save_v6::read_save(&save_dir).unwrap().save;
    assert_eq!(saved.world_id, "return_station");
    assert_eq!(saved.scene_id, "rs_core_room");
    assert!(saved.progression.progress.iter().any(|world| {
        world.world_id == "grey_hive"
            && world.completed
            && world.first_completion
            && world
                .completed_events
                .iter()
                .any(|event| event == "hive_extraction")
    }));
    let restarted = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&restarted).unwrap();
    let resumed = acknowledge_ready(&restarted, restarted.continue_saved().unwrap());
    assert_eq!(resumed.world_id, "return_station");
    assert_eq!(resumed.scene_id, "rs_core_room");

    let mh_gate = resumed
        .interactables
        .iter()
        .find(|item| item.entity_id == "rs_mh_world_gate_marker")
        .unwrap();
    assert_eq!(mh_gate.kind, "world_gate");
    assert!(mh_gate.active);
    let far_mh_gate = scene_route_commands::world_gate(
        &restarted,
        "rs_world_gate_to_mh",
        "mh-gate-range-check",
        resumed.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&far_mh_gate, "mh-gate-range-check", false);
    assert_eq!(
        far_mh_gate.error_code.as_deref(),
        Some("E_WORLD_GATE_OUT_OF_RANGE")
    );
    let stale_mh_gate = scene_route_commands::world_gate(
        &restarted,
        "rs_world_gate_to_mh",
        "mh-gate-stale-check",
        resumed.world_epoch.saturating_sub(1),
    )
    .unwrap();
    assert_v3_receipt(&stale_mh_gate, "mh-gate-stale-check", false);
    assert_eq!(
        stale_mh_gate.error_code.as_deref(),
        Some("E_WORLD_GATE_STALE_EPOCH")
    );
    let forged_mh_exit = scene_route_commands::world_gate(
        &restarted,
        "mh_extraction_return_to_rs",
        "mh-exit-from-return-station",
        resumed.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&forged_mh_exit, "mh-exit-from-return-station", false);
    assert_eq!(
        forged_mh_exit.error_code.as_deref(),
        Some("E_WORLD_GATE_UNAVAILABLE")
    );

    let at_rest = walk_to_compiled_target(&restarted, restarted.snapshot().unwrap(),
        "rs_core_room", "interactions", "rs_save_rest_terminal_marker");
    let rested = restarted.save_rest_terminal("rs_save_rest_terminal_marker", "campaign-rest-before-mh", at_rest.world_epoch).unwrap();
    assert_eq!(rested.player.current_hp, rested.player.max_hp);
    assert_eq!(rested.player.current_energy, rested.player.max_energy);
    let at_mh_gate = walk_to(&restarted, rested, 18.5, 12.0);
    let mh_entry = scene_route_commands::world_gate(
        &restarted,
        "rs_world_gate_to_mh",
        "first-mist-harbor-entry",
        at_mh_gate.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&mh_entry, "first-mist-harbor-entry", true);
    acknowledge_ready(&restarted, restarted.snapshot().unwrap());
    assert_eq!(mh_entry.snapshot.world_id, "mist_harbor");
    assert_eq!(mh_entry.snapshot.scene_id, "mh_fog_pier");

    let mut mh_view = mist_harbor_campaign::clear_scene(&restarted,restarted.snapshot().unwrap(),&save_dir);
    mh_view = walk_to_compiled_target(
        &restarted,
        mh_view,
        "mh_fog_pier",
        "transitions",
        "mh_to_tidal_warehouse",
    );
    mh_view = apply_transition(
        &restarted,
        "mh_to_tidal_warehouse",
        "mh-first-to-warehouse",
        mh_view.world_epoch,
    );
    mh_view = mist_harbor_campaign::clear_scene(&restarted,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &restarted,
        mh_view,
        "mh_tidal_warehouse",
        "interactions",
        "mh_west_beacon",
    );
    let west = restarted
        .activate_scene_interaction(
            "mh_west_beacon",
            "mh-first-west-beacon",
            mh_view.world_epoch,
        )
        .unwrap();
    assert!(
        west.applied,
        "{:?} at {:?}",
        west.error_code, mh_view.player.transform.position_m
    );
    mh_view = walk_to_compiled_target(
        &restarted,
        west.view,
        "mh_tidal_warehouse",
        "transitions",
        "mh_warehouse_to_signal_yard",
    );
    mh_view = apply_transition(
        &restarted,
        "mh_warehouse_to_signal_yard",
        "mh-first-to-signal-yard",
        mh_view.world_epoch,
    );
    mh_view = mist_harbor_campaign::clear_scene(&restarted,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &restarted,
        mh_view,
        "mh_signal_yard",
        "transitions",
        "mh_signal_yard_to_quay",
    );
    mh_view = apply_transition(
        &restarted,
        "mh_signal_yard_to_quay",
        "mh-first-to-quay",
        mh_view.world_epoch,
    );
    mh_view = mist_harbor_campaign::clear_scene(&restarted,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &restarted,
        mh_view,
        "mh_drowned_quay",
        "transitions",
        "mh_drowned_quay_to_breakwater",
    );
    mh_view = apply_transition(
        &restarted,
        "mh_drowned_quay_to_breakwater",
        "mh-first-to-breakwater",
        mh_view.world_epoch,
    );
    mh_view = mist_harbor_campaign::clear_scene(&restarted,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &restarted,
        mh_view,
        "mh_breakwater",
        "interactions",
        "mh_east_beacon",
    );
    let east = restarted
        .activate_scene_interaction(
            "mh_east_beacon",
            "mh-first-east-beacon",
            mh_view.world_epoch,
        )
        .unwrap();
    assert!(east.applied, "{:?}", east.error_code);
    mh_view = walk_to_compiled_target(
        &restarted,
        east.view,
        "mh_breakwater",
        "transitions",
        "mh_breakwater_to_pump_station",
    );
    mh_view = apply_transition(
        &restarted,
        "mh_breakwater_to_pump_station",
        "mh-first-to-pump",
        mh_view.world_epoch,
    );
    mh_view = mist_harbor_campaign::clear_scene(&restarted,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &restarted,
        mh_view,
        "mh_pump_station",
        "interactions",
        "mh_pump_control_primary",
    );
    let pump = restarted
        .activate_scene_interaction(
            "mh_pump_control_primary",
            "mh-first-pump-start",
            mh_view.world_epoch,
        )
        .unwrap();
    assert!(pump.applied, "{:?}", pump.error_code);
    assert_eq!(
        pump.view.mist_harbor_pump.as_ref().unwrap().state,
        wuxian_horror_ch1::world_v3::MistHarborPumpState::Draining
    );
    restarted.save().unwrap();
    let saved_mh = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(saved_mh.save.world_id, "mist_harbor");
    assert_eq!(saved_mh.save.scene_id, "mh_pump_station");
    assert_eq!(saved_mh.save.progression.current_world_id, "mist_harbor");
    assert_eq!(
        saved_mh.world_persistent_v1.mist_harbor.pump.state,
        wuxian_horror_ch1::world_persistent_v1::PumpStatus::Draining
    );
    assert!(saved_mh
        .world_persistent_v1
        .mist_harbor
        .explored_region_ids
        .iter()
        .any(|id| id == "mh_fp_entry_berth"));
    assert!(saved_mh
        .world_persistent_v1
        .mist_harbor
        .explored_region_ids
        .iter()
        .any(|id| id == "mh_ps_intake"));

    let continued_mh_runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&continued_mh_runtime).unwrap();
    let continued_mh = acknowledge_ready(&continued_mh_runtime, continued_mh_runtime.continue_saved().unwrap());
    assert_eq!(continued_mh.world_id, "mist_harbor");
    assert_eq!(continued_mh.scene_id, "mh_pump_station");
    assert_eq!(continued_mh.progression.current_world_id, "mist_harbor");
    assert_eq!(
        continued_mh.mist_harbor_pump.unwrap().state,
        wuxian_horror_ch1::world_v3::MistHarborPumpState::Draining
    );
    continued_mh_runtime.save().unwrap();
    let continued_save = save_v6::read_save(&save_dir).unwrap();
    assert!(continued_save
        .world_persistent_v1
        .mist_harbor
        .explored_region_ids
        .iter()
        .any(|id| id == "mh_fp_entry_berth"));
    assert!(continued_save
        .world_persistent_v1
        .mist_harbor
        .explored_region_ids
        .iter()
        .any(|id| id == "mh_ps_intake"));

    let mut mh_view = continued_mh_runtime.snapshot().unwrap();
    for _ in 0..420 {
        if mh_view.mist_harbor_pump.as_ref().unwrap().state
            == wuxian_horror_ch1::world_v3::MistHarborPumpState::Drained
        {
            break;
        }
        mh_view = continued_mh_runtime
            .submit_input(
                InputSample::new(
                    mh_view.world_epoch,
                    mh_view.ack_seq + 1,
                    NEXT_TIME.fetch_add(17, Ordering::Relaxed),
                    0.0,
                    0.0,
                )
                .unwrap(),
                vec![],
            )
            .unwrap();
    }
    assert_eq!(
        mh_view.mist_harbor_pump.as_ref().unwrap().state,
        wuxian_horror_ch1::world_v3::MistHarborPumpState::Drained
    );
    mh_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &continued_mh_runtime,
        mh_view,
        "mh_pump_station",
        "transitions",
        "mh_pump_station_to_tower",
    );
    mh_view = apply_transition(
        &continued_mh_runtime,
        "mh_pump_station_to_tower",
        "mh-first-to-tower",
        mh_view.world_epoch,
    );
    mh_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &continued_mh_runtime,
        mh_view,
        "mh_resonance_tower",
        "interactions",
        "mh_signal_console_staged",
    );
    let signal = continued_mh_runtime
        .activate_scene_interaction(
            "mh_signal_console_staged",
            "mh-first-signal",
            mh_view.world_epoch,
        )
        .unwrap();
    assert!(signal.applied, "{:?}", signal.error_code);
    mh_view = walk_to_compiled_target(
        &continued_mh_runtime,
        signal.view,
        "mh_resonance_tower",
        "transitions",
        "mh_tower_to_warden_arena",
    );
    mh_view = apply_transition(
        &continued_mh_runtime,
        "mh_tower_to_warden_arena",
        "mh-first-to-warden",
        mh_view.world_epoch,
    );
    assert_registered_warden(&mh_view, true);
    assert!(!mh_view.interactables.iter().find(|item|
        item.entity_id == "mh_warden_arena_to_extraction").unwrap().active,
        "the authored exit stays inactive until a real Warden defeat");
    continued_mh_runtime.save().unwrap();
    println!("Real campaign pre-Warden save: {}", save_dir.display());
    mh_view = kill_registered_warden_with_public_actions(&continued_mh_runtime, mh_view);
    continued_mh_runtime.save().unwrap();
    let defeated_save = save_v6::read_save(&save_dir).unwrap();
    assert!(defeated_save.world_persistent_v1.mist_harbor.warden_defeated);
    assert_eq!(defeated_save.save.generic_actors.len(), 1);
    assert_eq!(defeated_save.save.generic_actors[0].hp, 0);
    let after_defeat = acknowledge_ready(&continued_mh_runtime, continued_mh_runtime.continue_saved().unwrap());
    assert_registered_warden(&after_defeat, false);
    assert!(after_defeat.player.current_hp > 0);
    assert_eq!(after_defeat.progression, mh_view.progression);
    assert!(continued_mh_runtime.presentation_events_since(after_defeat.world_epoch, 0).unwrap()
        .iter().all(|event| event.kind != "ResonanceWardenDeath"));
    mh_view = after_defeat;
    mh_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,mh_view,&save_dir);
    mh_view = walk_to_compiled_target(
        &continued_mh_runtime,
        mh_view,
        "mh_warden_arena",
        "transitions",
        "mh_warden_arena_to_extraction",
    );
    mh_view = apply_transition(
        &continued_mh_runtime,
        "mh_warden_arena_to_extraction",
        "mh-first-to-extraction",
        mh_view.world_epoch,
    );
    assert_eq!(mh_view.scene_id, "mh_extraction");
    assert!(!mh_view
        .interactables
        .iter()
        .any(|item| item.entity_id.contains("staged")));
    let mh_exit = mh_view
        .interactables
        .iter()
        .find(|item| item.entity_id == "mh_extraction_return_to_rs")
        .unwrap();
    assert_eq!(mh_exit.kind, "world_gate");
    assert!(mh_exit.active);
    let exit_range = scene_route_commands::world_gate(
        &continued_mh_runtime,
        "mh_extraction_return_to_rs",
        "mh-exit-range-check",
        mh_view.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&exit_range, "mh-exit-range-check", false);
    assert_eq!(
        exit_range.error_code.as_deref(),
        Some("E_WORLD_GATE_OUT_OF_RANGE")
    );
    let at_mh_exit = walk_to_compiled_target(
        &continued_mh_runtime,
        mh_view,
        "mh_extraction",
        "interactions",
        "mh_extraction_exit",
    );
    let mh_return = scene_route_commands::world_gate(
        &continued_mh_runtime,
        "mh_extraction_return_to_rs",
        "mh-first-return-to-station",
        at_mh_exit.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&mh_return, "mh-first-return-to-station", true);
    acknowledge_ready(&continued_mh_runtime, continued_mh_runtime.snapshot().unwrap());
    assert_eq!(mh_return.snapshot.world_id, "return_station");
    let mh_first_clear = mh_return
        .snapshot
        .progression
        .worlds
        .iter()
        .find(|world| world.world_id == "mist_harbor")
        .unwrap();
    assert!(mh_first_clear.completed && mh_first_clear.first_completion);
    assert_eq!(mh_first_clear.completed_events.len(), 3);

    let at_revisit_rest = walk_to_compiled_target(&continued_mh_runtime,
        continued_mh_runtime.snapshot().unwrap(), "rs_core_room", "interactions",
        "rs_save_rest_terminal_marker");
    let rested_for_revisit = continued_mh_runtime.save_rest_terminal(
        "rs_save_rest_terminal_marker", "campaign-rest-before-mh-revisit",
        at_revisit_rest.world_epoch).unwrap();
    assert_eq!(rested_for_revisit.player.current_hp, rested_for_revisit.player.max_hp);
    let at_mh_gate = walk_to(
        &continued_mh_runtime,
        rested_for_revisit,
        18.5,
        12.0,
    );
    let mh_revisit = scene_route_commands::world_gate(
        &continued_mh_runtime,
        "rs_world_gate_to_mh",
        "mist-harbor-revisit",
        at_mh_gate.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&mh_revisit, "mist-harbor-revisit", true);
    acknowledge_ready(&continued_mh_runtime, continued_mh_runtime.snapshot().unwrap());
    assert_eq!(
        mh_revisit
            .snapshot
            .progression
            .worlds
            .iter()
            .find(|world| world.world_id == "mist_harbor")
            .unwrap()
            .revisit_count,
        1
    );
    let mut revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,continued_mh_runtime.snapshot().unwrap(),&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_fog_pier",
        "transitions",
        "mh_to_tidal_warehouse",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_to_tidal_warehouse",
        "mh-revisit-to-warehouse",
        revisit_view.world_epoch,
    );
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_tidal_warehouse",
        "transitions",
        "mh_warehouse_to_signal_yard",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_warehouse_to_signal_yard",
        "mh-revisit-to-yard",
        revisit_view.world_epoch,
    );
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_signal_yard",
        "transitions",
        "mh_signal_yard_to_quay",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_signal_yard_to_quay",
        "mh-revisit-to-quay",
        revisit_view.world_epoch,
    );
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_drowned_quay",
        "transitions",
        "mh_drowned_quay_to_breakwater",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_drowned_quay_to_breakwater",
        "mh-revisit-to-breakwater",
        revisit_view.world_epoch,
    );
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_breakwater",
        "transitions",
        "mh_breakwater_to_pump_station",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_breakwater_to_pump_station",
        "mh-revisit-to-pump",
        revisit_view.world_epoch,
    );
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_pump_station",
        "transitions",
        "mh_pump_station_to_tower",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_pump_station_to_tower",
        "mh-revisit-to-tower",
        revisit_view.world_epoch,
    );
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_resonance_tower",
        "transitions",
        "mh_tower_to_warden_arena",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_tower_to_warden_arena",
        "mh-revisit-to-warden",
        revisit_view.world_epoch,
    );
    assert_registered_warden(&revisit_view, false);
    assert!(revisit_view.player.current_hp > 0);
    assert!(continued_mh_runtime.presentation_events_since(revisit_view.world_epoch, 0).unwrap()
        .iter().all(|event| event.kind != "ResonanceWardenDeath"));
    revisit_view = mist_harbor_campaign::clear_scene(&continued_mh_runtime,revisit_view,&save_dir);
    revisit_view = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_warden_arena",
        "transitions",
        "mh_warden_arena_to_extraction",
    );
    revisit_view = apply_transition(
        &continued_mh_runtime,
        "mh_warden_arena_to_extraction",
        "mh-revisit-to-extraction",
        revisit_view.world_epoch,
    );
    let at_revisit_exit = walk_to_compiled_target(
        &continued_mh_runtime,
        revisit_view,
        "mh_extraction",
        "interactions",
        "mh_extraction_exit",
    );
    let mh_revisit_return = scene_route_commands::world_gate(
        &continued_mh_runtime,
        "mh_extraction_return_to_rs",
        "mh-revisit-return-to-station",
        at_revisit_exit.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&mh_revisit_return, "mh-revisit-return-to-station", true);
    acknowledge_ready(&continued_mh_runtime, continued_mh_runtime.snapshot().unwrap());
    let mh_after_revisit = mh_revisit_return
        .snapshot
        .progression
        .worlds
        .iter()
        .find(|world| world.world_id == "mist_harbor")
        .unwrap();
    assert!(mh_after_revisit.completed && mh_after_revisit.first_completion);
    assert_eq!(mh_after_revisit.revisit_count, 1);
    assert_eq!(mh_after_revisit.completed_events.len(), 3);

    clockworks_campaign::run(&continued_mh_runtime, &save_dir);

    let at_gate = walk_to(
        &continued_mh_runtime,
        continued_mh_runtime.snapshot().unwrap(),
        18.5,
        8.0,
    );
    let revisit = scene_route_commands::world_gate(
        &continued_mh_runtime,
        "rs_world_gate_to_gh",
        "post-clear-gh-revisit",
        at_gate.world_epoch,
    )
    .unwrap();
    assert_v3_receipt(&revisit, "post-clear-gh-revisit", true);
    acknowledge_ready(&continued_mh_runtime, continued_mh_runtime.snapshot().unwrap());
    assert_eq!(revisit.snapshot.world_id, "grey_hive");
    assert_eq!(
        revisit
            .snapshot
            .progression
            .worlds
            .iter()
            .find(|world| world.world_id == "grey_hive")
            .unwrap()
            .revisit_count,
        1
    );
}

#[test]
fn bundle_fails_closed_for_missing_tampered_and_old_fixture_content() {
    let embedded = production::embedded_scene_json().unwrap();
    assert_eq!(embedded.len(), 30);
    assert!(production::validate_scene_bundle(&embedded[..29]).is_err());

    let mut tampered: Vec<String> = embedded.iter().map(|scene| (*scene).to_owned()).collect();
    tampered[0] = tampered[0].replace("gh_beacon", "gh_forged_beacon");
    let tampered_refs: Vec<&str> = tampered.iter().map(String::as_str).collect();
    assert!(production::validate_scene_bundle(&tampered_refs).is_err());

    let mut tampered_mist: Vec<String> = embedded.iter().map(|scene| (*scene).to_owned()).collect();
    tampered_mist[12] = tampered_mist[12].replace("mh_fog_pier", "mh_forged_pier");
    let tampered_mist_refs: Vec<&str> = tampered_mist.iter().map(String::as_str).collect();
    assert!(production::validate_scene_bundle(&tampered_mist_refs).is_err());

    let mut tampered_clockworks: Vec<String> =
        embedded.iter().map(|scene| (*scene).to_owned()).collect();
    tampered_clockworks[21] =
        tampered_clockworks[21].replace("cw_entry_foundry", "cw_forged_foundry");
    let tampered_clockworks_refs: Vec<&str> =
        tampered_clockworks.iter().map(String::as_str).collect();
    assert!(production::validate_scene_bundle(&tampered_clockworks_refs).is_err());

    let mut fixture: Vec<String> = embedded.iter().map(|scene| (*scene).to_owned()).collect();
    fixture[0] = OLD_A5_GREY_HIVE_FIXTURE.to_owned();
    let fixture_refs: Vec<&str> = fixture.iter().map(String::as_str).collect();
    assert!(production::validate_scene_bundle(&fixture_refs).is_err());
}

#[test]
fn slice_registry_rejects_missing_scenes_old_fixtures_and_unauthored_exits() {
    let embedded = production::embedded_scene_json().unwrap();
    let (assets, entities) = catalogs();
    assert_eq!(assets.len(), 43);
    let expected_entities = BTreeSet::from([
        "enemy.clockworks.forged_guard".to_owned(),
        "enemy.clockworks.forged_guard_elite".to_owned(),
        "enemy.clockworks.prime_regulator".to_owned(),
        "enemy.clockworks.pressure_drone".to_owned(),
        "enemy.clockworks.furnace_hound".to_owned(),
        "enemy.grey_hive.brute".to_owned(),
        "enemy.grey_hive.swarm".to_owned(),
        "enemy.grey_hive.sentinel".to_owned(),
        "enemy.mist_harbor.drowned".to_owned(),
        "enemy.mist_harbor.resonance_warden".to_owned(),
        "enemy.mist_harbor.signal_wraith".to_owned(),
        "enemy.mist_harbor.tidebound".to_owned(),
        "grey_hive.infected_maintenance_worker".to_owned(),
        "grey_hive.infected_security".to_owned(),
    ]);
    assert_eq!(entities, expected_entities);

    let partial =
        WorldRegistry::load_grey_hive_slice(embedded[..11].iter().copied(), &assets, &entities)
            .unwrap();
    assert_eq!(partial.scenes().count(), 11);

    let native = WorldRegistry::load_grey_hive_with_return_station(
        embedded[..12].iter().copied(),
        &assets,
        &entities,
    )
    .unwrap();
    assert_eq!(native.scenes().count(), 12);
    let return_station = native.scene("rs_core_room").unwrap();
    assert_eq!(return_station.world_id, "return_station");
    assert!(return_station.transitions.is_empty());
    assert!(return_station
        .interactions
        .iter()
        .all(|interaction| interaction.event.is_none()));

    let native_with_mist = WorldRegistry::load_grey_hive_mist_harbor_with_return_station(
        embedded[..21].iter().copied(),
        &assets,
        &entities,
    )
    .unwrap();
    assert_eq!(native_with_mist.scenes().count(), 21);
    assert_eq!(
        native_with_mist
            .required_events("mist_harbor")
            .unwrap()
            .len(),
        3
    );
    assert!(native_with_mist.scene("mh_fog_pier").is_some());
    assert_eq!(
        WorldRegistry::load_grey_hive_mist_harbor_with_return_station(
            embedded[..20].iter().copied(),
            &assets,
            &entities,
        )
        .unwrap_err(),
        SceneRuntimeError::InvalidNativeSceneSet
    );

    // All authored event sources exist; the production installer
    // selects the separate strict native mode with canonical Clockworks sources.
    let complete_content =
        WorldRegistry::load(embedded.iter().copied(), &assets, &entities).unwrap();
    assert_eq!(complete_content.scenes().count(), 30);
    let mut missing_core: Vec<serde_json::Value> = embedded
        .iter()
        .map(|raw| serde_json::from_str(raw).unwrap())
        .collect();
    for scene in &mut missing_core {
        for item in scene["interactions"].as_array_mut().unwrap() {
            if item["event"] == "clockworks_core" {
                item["event"] = serde_json::Value::Null;
            }
        }
    }
    let missing_core: Vec<String> = missing_core
        .iter()
        .map(serde_json::Value::to_string)
        .collect();
    assert_eq!(
        WorldRegistry::load(missing_core.iter(), &assets, &entities).unwrap_err(),
        SceneRuntimeError::MissingRequiredEvent("clockworks".into())
    );
    assert_eq!(
        WorldRegistry::load_grey_hive_slice(embedded[..4].iter().copied(), &assets, &entities)
            .unwrap_err(),
        SceneRuntimeError::InvalidGreyHiveSlice
    );

    let mut fixture: Vec<String> = embedded.iter().map(|scene| (*scene).to_owned()).collect();
    fixture[0] = OLD_A5_GREY_HIVE_FIXTURE.to_owned();
    assert_eq!(
        WorldRegistry::load_grey_hive_slice(
            fixture.iter().take(11).map(String::as_str),
            &assets,
            &entities
        )
        .unwrap_err(),
        SceneRuntimeError::InvalidGreyHiveSlice
    );

    let mut invalid_exit: Vec<String> = embedded.iter().map(|scene| (*scene).to_owned()).collect();
    let gate = invalid_exit
        .iter_mut()
        .find(|scene| scene.contains("gh_gate_a_to_shaft"))
        .unwrap();
    *gate = gate.replace("gh_central_shaft", "gh_scene_not_authored");
    assert!(matches!(
        WorldRegistry::load_grey_hive_slice(
            invalid_exit.iter().take(11).map(String::as_str),
            &assets,
            &entities
        ),
        Err(SceneRuntimeError::UnknownTransitionTarget(target)) if target == "gh_scene_not_authored"
    ));
}

fn staged_scene_values() -> Vec<serde_json::Value> {
    production::embedded_scene_json()
        .unwrap()
        .iter()
        .map(|raw| serde_json::from_str(raw).unwrap())
        .collect()
}

fn staged_scene_strings(scenes: &[serde_json::Value]) -> Vec<String> {
    scenes.iter().map(serde_json::Value::to_string).collect()
}

#[test]
fn staged_production_registry_admits_exact_thirty_scene_set_without_clockworks_route() {
    let embedded = production::embedded_scene_json().unwrap();
    let (assets, entities) = catalogs();
    let registry = WorldRegistry::load_staged_clockworks_production(
        embedded.iter().copied(),
        &assets,
        &entities,
    )
    .unwrap();
    assert_eq!(registry.scenes().count(), 30);
    assert!(registry.is_staged_clockworks_production());
    assert_eq!(
        registry
            .scenes()
            .filter(|scene| scene.world_id == "grey_hive")
            .count(),
        11
    );
    assert_eq!(
        registry
            .scenes()
            .filter(|scene| scene.world_id == "return_station")
            .count(),
        1
    );
    assert_eq!(
        registry
            .scenes()
            .filter(|scene| scene.world_id == "mist_harbor")
            .count(),
        9
    );
    assert_eq!(
        registry
            .scenes()
            .filter(|scene| scene.world_id == "clockworks")
            .count(),
        9
    );
    let arena = registry.scene("cw_forged_guard_arena").unwrap();
    let elite_spawns: Vec<_> = arena
        .spawns
        .iter()
        .filter(|spawn| spawn.entity_type.as_deref() == Some("enemy.clockworks.forged_guard_elite"))
        .collect();
    assert_eq!(elite_spawns.len(), 1);
    assert_eq!(elite_spawns[0].id, "cw_forged_guard_elite");
    assert_eq!(elite_spawns[0].kind, "enemy");
    let regulator_core = registry.scene("cw_regulator_core").unwrap();
    let regulator_spawns: Vec<_> = regulator_core
        .spawns
        .iter()
        .filter(|spawn| {
            spawn.id == "cw_prime_regulator"
                || spawn.entity_type.as_deref() == Some("enemy.clockworks.prime_regulator")
        })
        .collect();
    assert_eq!(regulator_spawns.len(), 1);
    assert_eq!(regulator_spawns[0].id, "cw_prime_regulator");
    assert_eq!(
        regulator_spawns[0].entity_type.as_deref(),
        Some("enemy.clockworks.prime_regulator")
    );
    assert_eq!(regulator_spawns[0].position, [14.0, 0.0, 8.0]);
    let shutter_blockers: Vec<_> = arena
        .collision
        .iter()
        .filter(|collision| collision.requires_actor_first_kill.is_some())
        .collect();
    assert_eq!(shutter_blockers.len(), 2);
    assert!(shutter_blockers.iter().all(|collision| {
        collision.requires_actor_first_kill.as_deref() == Some("cw_forged_guard_elite")
    }));
    let expected: BTreeSet<_> = production::EMBEDDED_SCENE_IDS.iter().copied().collect();
    let actual: BTreeSet<_> = registry
        .scenes()
        .map(|scene| scene.scene_id.as_str())
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(
        registry.required_events("clockworks").unwrap(),
        &BTreeSet::from([
            "clockworks_core".to_owned(),
            "clockworks_shutdown".to_owned(),
            "clockworks_valves".to_owned(),
        ])
    );
    assert_eq!(registry.event_owner("clockworks_core"), Some("clockworks"));
    let clockworks_emitted: BTreeSet<_> = registry
        .scenes()
        .filter(|scene| scene.world_id == "clockworks")
        .flat_map(|scene| {
            scene
                .interactions
                .iter()
                .filter_map(|item| item.event.clone())
                .chain(
                    scene
                        .interaction_aggregates
                        .iter()
                        .map(|aggregate| aggregate.event.clone()),
                )
                .chain(scene.triggers.iter().map(|trigger| trigger.event.clone()))
        })
        .collect();
    assert!(clockworks_emitted.contains("clockworks_valves"));
    assert!(clockworks_emitted.contains("clockworks_shutdown"));
    assert!(clockworks_emitted.contains("clockworks_core"));
    let core_console = regulator_core
        .interactions
        .iter()
        .find(|item| item.id == "cw_regulator_core_console_staged")
        .unwrap();
    assert_eq!(core_console.kind, "terminal");
    assert_eq!(core_console.event.as_deref(), Some("clockworks_core"));
    assert_eq!(core_console.range_m, Some(2.5));
    let return_station = registry.scene("rs_core_room").unwrap();
    assert!(return_station.transitions.is_empty());
    assert!(return_station
        .doors
        .iter()
        .all(|door| door.to_scene_id.is_none()));
    assert!(return_station
        .interactions
        .iter()
        .all(|interaction| interaction.event.is_none()));

    let root = save_root();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    let startup = production::install(&runtime).unwrap();
    assert_eq!(startup.scene_id, "rs_core_room");
    drop(runtime);
    if root.exists() {
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn staged_production_registry_rejects_wrong_set_resources_entities_and_unexpected_gates() {
    let (assets, entities) = catalogs();
    let source = production::embedded_scene_json().unwrap();
    assert_eq!(
        WorldRegistry::load_staged_clockworks_production(
            source.iter().take(29).copied(),
            &assets,
            &entities,
        )
        .unwrap_err(),
        SceneRuntimeError::InvalidNativeSceneSet
    );

    let mut wrong_identity = staged_scene_values();
    wrong_identity[21]["sceneId"] = serde_json::Value::String("cw_wrong_scene".into());
    let wrong_identity = staged_scene_strings(&wrong_identity);
    assert_eq!(
        WorldRegistry::load_staged_clockworks_production(
            wrong_identity.iter().map(String::as_str),
            &assets,
            &entities,
        )
        .unwrap_err(),
        SceneRuntimeError::InvalidNativeSceneSet
    );

    let mut wrong_world = staged_scene_values();
    wrong_world[21]["worldId"] = serde_json::Value::String("mist_harbor".into());
    let wrong_world = staged_scene_strings(&wrong_world);
    assert_eq!(
        WorldRegistry::load_staged_clockworks_production(
            wrong_world.iter().map(String::as_str),
            &assets,
            &entities,
        )
        .unwrap_err(),
        SceneRuntimeError::InvalidNativeSceneSet
    );

    let mut swapped_worlds = staged_scene_values();
    swapped_worlds[12]["worldId"] = serde_json::Value::String("clockworks".into());
    swapped_worlds[21]["worldId"] = serde_json::Value::String("mist_harbor".into());
    let swapped_worlds = staged_scene_strings(&swapped_worlds);
    assert_eq!(
        WorldRegistry::load_staged_clockworks_production(
            swapped_worlds.iter().map(String::as_str),
            &assets,
            &entities,
        )
        .unwrap_err(),
        SceneRuntimeError::InvalidNativeSceneSet
    );

    let mut missing_assets = assets.clone();
    missing_assets.remove("runtime2d.world.clockworks.floor_steel.v1");
    assert!(WorldRegistry::load_staged_clockworks_production(
        source.iter().copied(),
        &missing_assets,
        &entities,
    )
    .is_err());
    let mut missing_entities = entities.clone();
    missing_entities.remove("enemy.clockworks.forged_guard");
    assert!(WorldRegistry::load_staged_clockworks_production(
        source.iter().copied(),
        &assets,
        &missing_entities,
    )
    .is_err());

    let mut cross_world = staged_scene_values();
    cross_world[21]["transitions"] = serde_json::json!([{
        "id": "cw_to_gh_unexpected",
        "toSceneId": "gh_entry_maintenance",
        "spawnId": "gh_entry_spawn",
        "polygon": [[20.0, 7.0], [21.0, 7.0], [21.0, 8.0], [20.0, 8.0]]
    }]);
    let cross_world = staged_scene_strings(&cross_world);
    assert!(matches!(
        WorldRegistry::load_staged_clockworks_production(
            cross_world.iter().map(String::as_str),
            &assets,
            &entities,
        ),
        Err(SceneRuntimeError::InvalidNativeSceneSet)
    ));

    let mut return_gate = staged_scene_values();
    return_gate[11]["transitions"] = serde_json::json!([{
        "id": "rs_to_cw_unexpected",
        "toSceneId": "cw_entry_foundry",
        "spawnId": "cw_entry_spawn",
        "polygon": [[8.0, 5.0], [9.0, 5.0], [9.0, 6.0], [8.0, 6.0]]
    }]);
    let return_gate = staged_scene_strings(&return_gate);
    assert!(matches!(
        WorldRegistry::load_staged_clockworks_production(
            return_gate.iter().map(String::as_str),
            &assets,
            &entities,
        ),
        Err(SceneRuntimeError::InvalidNativeSceneSet)
    ));
}

fn run_create(root: &std::ffi::OsStr) {
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    let startup = production::install(&runtime).unwrap();
    assert_eq!(startup.scene_id, "rs_core_room");
    let new_journey = enter_grey_hive(&runtime, acknowledge_ready(&runtime, runtime.reset_new().unwrap()), "cross-process-new");
    let mut view = walk_x(&runtime, new_journey, 18.0);
    view = runtime
        .transition_scene(
            "gh_entry_to_power_room",
            "production-enter-power-room",
            view.world_epoch,
        ).map(|prepared| acknowledge_ready(&runtime, prepared)).unwrap();
    assert_eq!(view.scene_id, "gh_power_room");
    runtime.save().unwrap();
    let saved = save_v6::read_save(PathBuf::from(root).as_path())
        .unwrap()
        .save;
    assert_eq!(saved.scene_id, "gh_power_room");
}

fn run_continue(root: &std::ffi::OsStr) {
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    let startup = production::install(&runtime).unwrap();
    assert_eq!(startup.scene_id, "rs_core_room");
    let continued = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(continued.scene_id, "gh_power_room");
    assert!(continued
        .interactables
        .iter()
        .any(|item| item.entity_id == "gh_power_console" && item.active));
}

#[test]
fn production_bundle_new_save_and_continue_work_across_processes_without_fixtures() {
    let root = save_root();
    for role in ["create", "continue"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "production_scene_process_worker", "--nocapture"])
            .env("PRODUCTION_SCENE_SAVE_ROOT", &root)
            .env("PRODUCTION_SCENE_ROLE", role)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "worker {role} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn production_scene_process_worker() {
    let (Some(root), Some(role)) = (
        std::env::var_os("PRODUCTION_SCENE_SAVE_ROOT"),
        std::env::var("PRODUCTION_SCENE_ROLE").ok(),
    ) else {
        return;
    };
    match role.as_str() {
        "create" => run_create(&root),
        "continue" => run_continue(&root),
        "route-create" => run_route_create(&root),
        "route-continue" => run_route_continue(&root),
        other => panic!("unknown production scene worker role: {other}"),
    }
}

#[test]
fn native_production_continues_canonical_clockworks_without_inventing_settlement() {
    for scene_id in ["cw_entry_foundry", "cw_pressure_hall", "cw_shutdown_exit"] {
        let root = save_root();
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        production::install(&runtime).unwrap();
        runtime.save().unwrap();
        let mut crafted = clockworks_shaped_save(save_v6::read_save(&root).unwrap(), scene_id);
        let cw = crafted.save.progression.progress.iter_mut().find(|p| p.world_id == "clockworks").unwrap();
        cw.completed = false; cw.first_completion = false; cw.completed_events.clear();
        crafted.validate().unwrap();
        save_v6::write_save(&root, &crafted).unwrap();
        let path = save_v6::save_path(&root);
        let bytes_before = fs::read(&path).unwrap();
        let prepared = runtime.continue_saved().unwrap();
        assert!(prepared.entry_token.is_some());
        assert_eq!(prepared.scene_id, scene_id);
        let continued = acknowledge_ready(&runtime, prepared);
        assert!(continued.progression.worlds.iter().any(|p| p.world_id == "clockworks" && !p.completed && p.completed_events.is_empty()));
        assert!(!continued.interactables.iter().any(|i| i.entity_id == "cw_shutdown_return_to_rs"));
        assert_eq!(fs::read(&path).unwrap(), bytes_before);
        let unchanged = save_v6::read_save(&root).unwrap();
        assert!(unchanged.world_persistent_v1.clockworks.pressure_valve_ids.is_empty());
        assert!(!unchanged.world_persistent_v1.clockworks.core_console_confirmed);
        assert!(!unchanged.world_persistent_v1.clockworks.regulator_defeated);
    }
}

#[test]
fn native_production_rejects_noncanonical_world_only_clockworks_without_mutation() {
    let root = save_root();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    production::install(&runtime).unwrap(); runtime.save().unwrap();
    let crafted = clockworks_shaped_save(save_v6::read_save(&root).unwrap(), "clockworks");
    save_v6::write_save(&root, &crafted).unwrap();
    let path = save_v6::save_path(&root); let bytes = fs::read(&path).unwrap();
    runtime.pause().unwrap(); let before = runtime.snapshot().unwrap();
    assert_eq!(runtime.continue_saved().unwrap_err(), "E_SAVE_SCENE_UNKNOWN");
    assert_same_runtime_snapshot(&before, &runtime.snapshot().unwrap());
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn native_production_continues_clockworks_slot_without_rewriting_or_completing_it() {
    let root = save_root();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    production::install(&runtime).unwrap(); runtime.save().unwrap();
    let mut crafted = clockworks_shaped_save(save_v6::read_save(&root).unwrap(), "cw_shutdown_exit");
    let cw = crafted.save.progression.progress.iter_mut().find(|p| p.world_id == "clockworks").unwrap();
    cw.completed = false; cw.first_completion = false; cw.completed_events.clear();
    crafted.validate().unwrap();
    wuxian_horror_ch1::save_slots::create_slot_v6(&root, "old-cw", "Clockworks", &crafted).unwrap();
    let path = root.join("slots/old-cw/slot-v6.json"); let bytes = fs::read(&path).unwrap();
    let prepared = runtime.continue_slot("old-cw").unwrap();
    assert!(prepared.entry_token.is_some());
    let continued = acknowledge_ready(&runtime, prepared);
    assert_eq!(continued.scene_id, "cw_shutdown_exit");
    assert!(continued.progression.worlds.iter().any(|p| p.world_id == "clockworks" && !p.completed && p.completed_events.is_empty()));
    assert!(!continued.interactables.iter().any(|i| i.entity_id == "cw_shutdown_return_to_rs"));
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn native_production_preserves_prime_regulator_core_actor_state() {
    let root = save_root();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    production::install(&runtime).unwrap();
    runtime.save().unwrap();
    let mut actor = wuxian_horror_ch1::world_v3::ActorRuntime::spawn(
        "cw_prime_regulator",
        "enemy.clockworks.prime_regulator",
        wuxian_horror_ch1::world_v3::Vec3::new(14.0, 0.0, 8.0).unwrap(),
    )
    .unwrap();
    actor.take_damage(37);
    actor.state = wuxian_horror_ch1::world_v3::ActorAiState::Windup;
    actor.state_remaining_ms = 700;
    actor.current_attack = Some(wuxian_horror_ch1::world_v3::ActorAttackKind::PressureWave);
    actor.windup_origin_m = Some(wuxian_horror_ch1::world_v3::Vec3::new(14.0, 0.0, 8.0).unwrap());
    let crafted = clockworks_shaped_save(save_v6::read_save(&root).unwrap(), "cw_regulator_core");
    let mut value = serde_json::to_value(crafted).unwrap();
    value["save"]["genericActors"] = serde_json::json!([actor]);
    save_v6::write_save(&root, &serde_json::from_value(value).unwrap()).unwrap();

    let continued = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(continued.scene_id, "cw_regulator_core");
    assert!(continued
        .actors
        .iter()
        .any(|actor| { actor.entity_id == "cw_prime_regulator" && actor.active }));
    runtime.save().unwrap();
    let restored = save_v6::read_save(&root).unwrap().save.generic_actors;
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].entity_id, "cw_prime_regulator");
    assert_eq!(restored[0].hp, 563);
    assert_eq!(
        restored[0].state,
        wuxian_horror_ch1::world_v3::ActorAiState::Windup
    );
    assert!((650..=700).contains(&restored[0].state_remaining_ms));
    assert_eq!(
        restored[0].current_attack,
        Some(wuxian_horror_ch1::world_v3::ActorAttackKind::PressureWave)
    );
    assert!(!continued
        .interactables
        .iter()
        .any(|item| item.entity_id == "cw_regulator_core_to_shutdown_exit" && item.active));
}

#[test]
fn full_campaign_registry_keeps_legal_clockworks_save_continue_available() {
    let root = save_root();
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime
        .load_scene_registry(
            [FULL_TEST_RS, FULL_TEST_GH, FULL_TEST_MH, FULL_TEST_CW],
            "cw_test_exit",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();

    let seed_root = save_root();
    let seed_runtime = FormalRuntime::new_with_save_dir(seed_root.clone()).unwrap();
    production::install(&seed_runtime).unwrap();
    seed_runtime.save().unwrap();
    let crafted = clockworks_shaped_save(save_v6::read_save(&seed_root).unwrap(), "cw_test_exit");
    save_v6::write_save(&root, &crafted).unwrap();

    let continued = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(continued.world_id, "clockworks");
    assert_eq!(continued.scene_id, "cw_test_exit");
}

fn run_route_create(root: &std::ffi::OsStr) {
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    production::install(&runtime).unwrap();
    let view = run_route_journey(&runtime, "route-cross-process");
    assert_eq!(view.scene_id, "gh_lockdown");
    runtime.save().unwrap();
    let saved = save_v6::read_save(PathBuf::from(root).as_path())
        .unwrap()
        .save;
    assert_eq!(saved.scene_id, "gh_lockdown");
    assert_eq!(
        saved.checkpoint_id.as_deref(),
        Some("gh_lockdown_checkpoint")
    );
}

fn run_route_continue(root: &std::ffi::OsStr) {
    let runtime = FormalRuntime::new_with_save_dir(PathBuf::from(root)).unwrap();
    production::install(&runtime).unwrap();
    let continued = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(continued.scene_id, "gh_lockdown");
    assert_eq!(
        continued.checkpoint_id.as_deref(),
        Some("gh_lockdown_checkpoint")
    );
    assert!(continued.interactables.iter().any(|item| {
        item.entity_id == "gh_lockdown_checkpoint"
            && item.kind == "scene_checkpoint"
            && !item.active
    }));
}

#[test]
fn production_scene_routes_save_and_continue_across_processes() {
    let root = save_root();
    for role in ["route-create", "route-continue"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "production_scene_process_worker", "--nocapture"])
            .env("PRODUCTION_SCENE_SAVE_ROOT", &root)
            .env("PRODUCTION_SCENE_ROLE", role)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "worker {role} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let _ = fs::remove_dir_all(root);
}
