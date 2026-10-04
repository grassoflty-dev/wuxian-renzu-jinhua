#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use wuxian_horror_ch1::{
    continuous_input::InputSample,
    formal_runtime::{CombatIntentRequest, FormalRuntime, RouteCommandRequest},
    save_slots,
    save_v3::SAVE_FILE_NAME,
    save_v5::{self, SaveV5, SAVE_V5_FILE_NAME},
    save_v6::{self, SaveV6, SAVE_V6_FILE_NAME},
    scene_route_commands,
    world_v3::WorldView,
};

const RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");

fn root(label: &str) -> PathBuf {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("save-v5-{label}-{}-{token}", std::process::id()))
}

fn load_gh(runtime: &FormalRuntime) {
    runtime
        .load_scene_registry(
            [RS, GH, MH, CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
}

fn aggregate_grey_hive_fixture() -> String {
    let mut scene: serde_json::Value = serde_json::from_str(GH).unwrap();
    scene["sceneId"] = serde_json::json!("gh_test_aggregate");
    scene["transitions"] = serde_json::json!([]);
    scene["interactions"] = serde_json::json!([
        { "id": "aggregate_valve_a", "kind": "valve_control", "event": null, "position": [1, 0, 1] },
        { "id": "aggregate_valve_b", "kind": "valve_control", "event": null, "position": [1.5, 0, 1] },
        { "id": "aggregate_valve_c", "kind": "valve_control", "event": null, "position": [2, 0, 1] },
        { "id": "aggregate_complete_marker", "kind": "all_valves_complete_marker", "event": null, "position": [1.5, 0, 2] }
    ]);
    scene["interactionAggregates"] = serde_json::json!([{
        "markerId": "aggregate_complete_marker",
        "memberIds": ["aggregate_valve_a", "aggregate_valve_b", "aggregate_valve_c"],
        "event": "hive_power"
    }]);
    serde_json::to_string(&scene).unwrap()
}

fn load_aggregate(runtime: &FormalRuntime) {
    let scene = aggregate_grey_hive_fixture();
    runtime
        .load_scene_registry(
            [RS, GH, MH, CW, scene.as_str()],
            "gh_test_aggregate",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
}

#[test]
fn scene_state_continue_restores_two_and_three_member_aggregate_progress() {
    let path = root("interaction-aggregate-roundtrip");
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_aggregate(&runtime);
        let initial = runtime.snapshot().unwrap();
        for (id, request) in [
            ("aggregate_valve_c", "aggregate-c-first"),
            ("aggregate_valve_a", "aggregate-a-second"),
        ] {
            let receipt = runtime
                .activate_scene_interaction(id, request, initial.world_epoch)
                .unwrap();
            assert!(receipt.applied, "{request}: {:?}", receipt.error_code);
            assert!(!receipt.receipt.snapshot.progression.worlds[0]
                .completed_events
                .contains(&"hive_power".to_string()));
        }
        runtime.save().unwrap();
        let saved = save_v6::read_save(&path).unwrap().save;
        assert_eq!(saved.scene_id, "gh_test_aggregate");
        assert_eq!(saved.scene_states[0].activated_ids,
            ["aggregate_valve_a", "aggregate_valve_c"]);
        assert!(saved.scene_states[0].emitted_event_ids.is_empty());
    }
    {
        let continued = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_aggregate(&continued);
        let restored = acknowledge_ready(&continued, continued.continue_saved().unwrap());
        assert_eq!(restored.scene_id, "gh_test_aggregate");
        for id in ["aggregate_valve_a", "aggregate_valve_c"] {
            assert!(restored.interactables.iter().any(|item| item.entity_id == id && !item.active));
        }
        assert!(restored.interactables.iter().any(|item| item.entity_id == "aggregate_valve_b" && item.active));
        assert!(!restored.progression.worlds[0]
            .completed_events
            .contains(&"hive_power".to_string()));

        let completed = continued
            .activate_scene_interaction("aggregate_valve_b", "aggregate-b-third", restored.world_epoch)
            .unwrap();
        assert!(completed.applied, "{:?}", completed.error_code);
        assert!(completed.receipt.snapshot.progression.worlds[0]
            .completed_events
            .contains(&"hive_power".to_string()));
        continued.save().unwrap();
        let saved = save_v6::read_save(&path).unwrap().save;
        assert_eq!(saved.scene_states[0].activated_ids,
            ["aggregate_valve_a", "aggregate_valve_b", "aggregate_valve_c"]);
        assert_eq!(saved.scene_states[0].emitted_event_ids, ["hive_power"]);
    }
    {
        let continued = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_aggregate(&continued);
        let restored = acknowledge_ready(&continued, continued.continue_saved().unwrap());
        assert_eq!(restored.scene_id, "gh_test_aggregate");
        assert!(restored.progression.worlds[0]
            .completed_events
            .contains(&"hive_power".to_string()));
        assert!(restored.interactables.iter().filter(|item| item.entity_id.starts_with("aggregate_valve_")).all(|item| !item.active));
        let duplicate = continued
            .activate_scene_interaction("aggregate_valve_b", "aggregate-b-repeated", restored.world_epoch)
            .unwrap();
        assert!(!duplicate.applied);
        assert!(duplicate.error_code.unwrap().contains("AlreadyApplied"));
        assert!(continued.snapshot().unwrap().progression.worlds[0]
            .completed_events
            .contains(&"hive_power".to_string()));
    }
    let _ = fs::remove_dir_all(path);
}

#[test]
fn v6_disk_continue_restores_world_scene_checkpoint_objective_progression_and_capability() {
    let path = root("scene-roundtrip");
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_gh(&runtime);
        let initial = runtime.snapshot().unwrap();
        runtime
            .activate_scene_checkpoint("gh_checkpoint_test", "checkpoint-save", initial.world_epoch)
            .unwrap();
        let before = runtime.snapshot().unwrap();
        runtime
            .activate_scene_interaction("gh_power_console_test", "power-save", before.world_epoch)
            .unwrap();
        runtime.save().unwrap();
        assert!(path.join(SAVE_V6_FILE_NAME).is_file());
        let saved = save_v6::read_save(&path).unwrap().save;
        assert_eq!(saved.world_id, "grey_hive");
        assert_eq!(saved.scene_id, "gh_test_power");
        assert_eq!(saved.checkpoint_id.as_deref(), Some("gh_checkpoint_test"));
        assert_eq!(
            saved.scene_states[0].activated_ids,
            ["gh_power_console_test"]
        );
        assert!(saved.scene_states[0]
            .emitted_event_ids
            .contains(&"hive_power".to_string()));
        assert!(saved
            .progression
            .progress
            .iter()
            .find(|p| p.world_id == "grey_hive")
            .unwrap()
            .completed_events
            .contains(&"hive_power".to_string()));
    }
    {
        let restarted = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_gh(&restarted);
        let restored = acknowledge_ready(&restarted, restarted.continue_saved().unwrap());
        assert_eq!(restored.scene_id, "gh_test_power");
        assert_eq!(
            restored.checkpoint_id.as_deref(),
            Some("gh_checkpoint_test")
        );
        assert!(restored
            .objectives
            .iter()
            .any(|o| o.objective_id == "gh_power_objective" && o.state == "complete"));
        restarted.save().unwrap();
        assert!(save_v6::read_save(&path)
            .unwrap()
            .save
            .progression
            .progress
            .iter()
            .find(|p| p.world_id == "grey_hive")
            .unwrap()
            .completed_events
            .contains(&"hive_power".to_string()));
        assert_eq!(restored.world_epoch, 2);
    }
    let _ = fs::remove_dir_all(path);
}

#[test]
fn v5_accepts_native_return_station_scene_without_changing_campaign_progression() {
    let path = root("return-station-scene");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_gh(&runtime);
    runtime.save().unwrap();

    let mut saved = save_v6::read_save(&path).unwrap().save;
    saved.world_id = "return_station".into();
    saved.scene_id = "rs_core_room".into();
    saved.capabilities.world_id = "return_station".into();
    saved.explored.world_id = "return_station".into();
    saved.scene_states[0].scene_id = "rs_core_room".into();
    assert_eq!(saved.progression.current_world_id, "grey_hive");
    assert!(saved.validate().is_ok());

    let _ = fs::remove_dir_all(path);
}

#[test]
fn v4_migration_validates_backs_up_and_retains_source_while_writing_v5() {
    let path = root("v4-migration");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    runtime.save().unwrap();
    let legacy = v4_json_from_v5(&save_v6::read_save(&path).unwrap().save);
    let bytes = serde_json::to_vec_pretty(&legacy).unwrap();
    let source = path.join(SAVE_FILE_NAME);
    fs::write(&source, &bytes).unwrap();
    fs::remove_file(path.join(SAVE_V6_FILE_NAME)).unwrap();
    let migrated = save_v5::read_or_migrate(&path).unwrap();
    assert_eq!(migrated.schema_version, 5);
    assert_eq!(
        migrated
            .migration_provenance
            .as_ref()
            .unwrap()
            .source_format,
        "formal-save-v3.json"
    );
    assert_eq!(migrated.world_id, "grey_hive");
    assert_eq!(migrated.scene_id, "grey_hive");
    assert_eq!(migrated.checkpoint_id.as_deref(), Some("gh_cp_airlock"));
    assert!(migrated.migration_provenance.is_some());
    assert_eq!(fs::read(source).unwrap(), bytes);
    assert_eq!(
        fs::read(path.join(format!("{SAVE_FILE_NAME}.bak"))).unwrap(),
        bytes
    );
    assert_eq!(save_v5::read_save(&path).unwrap(), migrated);
    let _ = fs::remove_dir_all(path);
}

#[test]
fn v4_migration_rejects_corruption_before_creating_backup_or_v5() {
    let path = root("v4-corrupt");
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join(SAVE_FILE_NAME), b"{not valid").unwrap();
    let error = save_v5::read_or_migrate(&path).unwrap_err();
    assert_eq!(error, "E_SAVE_CORRUPT");
    assert!(!path.join(format!("{SAVE_FILE_NAME}.bak")).exists());
    assert!(!path.join(SAVE_V5_FILE_NAME).exists());
    let _ = fs::remove_dir_all(path);
}

#[test]
fn three_world_fixture_with_authored_core_survives_disk_continue_in_clockworks() {
    // CW-SHUTDOWN-AIR-STEP-GRANT-01 makes the official terminal the only
    // valid shutdown source. Keep the generic trigger to verify rejection.
    let mut clockworks: serde_json::Value = serde_json::from_str(CW).unwrap();
    clockworks["sceneId"] = serde_json::json!("cw_shutdown_exit");
    clockworks["interactions"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "cw_master_shutdown_staged",
            "kind": "terminal",
            "event": "clockworks_shutdown",
            "position": [1, 0, 1]
        }));
    // Keep the ordinary fixture route, but obtain Core only from the authored
    // Regulator encounter. Only test bridge transitions are adapted; Boss,
    // console, geometry, phase tuning and hazards remain the compiled content.
    clockworks["transitions"] = serde_json::json!([{
        "id": "cw_test_to_regulator_core",
        "toSceneId": "cw_regulator_core",
        "spawnId": "cw_regulator_core_spawn",
        "requiresEvent": "clockworks_valves",
        "polygon": [[0, 0], [2, 0], [2, 2], [0, 2]]
    }]);
    let clockworks = serde_json::to_string(&clockworks).unwrap();
    let mut core: serde_json::Value = serde_json::from_str(include_str!(
        "../../content/scenes/compiled/cw_regulator_core.json"
    )).unwrap();
    core["transitions"].as_array_mut().unwrap()
        .retain(|exit| exit["id"] == "cw_regulator_core_to_shutdown_exit");
    core["transitions"][0]["spawnId"] = serde_json::json!("cw_spawn");
    let core = serde_json::to_string(&core).unwrap();
    let station = RS.replace("cw_test_exit", "cw_shutdown_exit");
    let scenes = [station.as_str(), GH, MH, clockworks.as_str(), core.as_str()];
    let (assets, entities) = authored_scene_catalogs();
    let path = root("three-world-continue");
    let saved_progression = {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        runtime
            .load_scene_registry(scenes, "gh_test_power", &assets, &entities)
            .unwrap();
        for (id, request) in [
            ("gh_power_console_test", "save-gh-power"),
            ("gh_lockdown_test", "save-gh-lockdown"),
            ("gh_extract_test", "save-gh-extract"),
        ] {
            interact(&runtime, id, request);
        }
        complete_world(&runtime, "grey_hive", "save-gh-complete");
        transition(&runtime, "gh_to_hub", "save-gh-hub");
        transition(&runtime, "rs_to_mh", "save-mh-enter");
        for (id, request) in [
            ("mh_west_test", "save-mh-west"),
            ("mh_east_test", "save-mh-east"),
            ("mh_signal_test", "save-mh-signal"),
        ] {
            interact(&runtime, id, request);
        }
        complete_world(&runtime, "mist_harbor", "save-mh-complete");
        transition(&runtime, "mh_to_hub", "save-mh-hub");
        transition(&runtime, "rs_to_cw", "save-cw-enter");
        interact(&runtime, "cw_valves_test", "save-cw-valves");
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let forged_core = runtime
            .activate_scene_interaction("cw_core_test", "save-cw-forged-core", epoch)
            .unwrap();
        assert!(!forged_core.applied);
        assert_eq!(forged_core.error_code.as_deref(), Some("E_CLOCKWORKS_CORE_SOURCE_INVALID"));
        transition(&runtime, "cw_test_to_regulator_core", "save-cw-real-core-enter");
        defeat_and_activate_authored_core(&runtime);
        move_core_player_to(&runtime, 23.0, 8.0);
        transition(&runtime, "cw_regulator_core_to_shutdown_exit", "save-cw-real-core-exit");
        let before = runtime.snapshot().unwrap();
        let affordance = before
            .interactables
            .iter()
            .find(|item| item.entity_id == "cw_shutdown_trigger_test")
            .expect("generic fixture trigger is projected");
        assert_eq!(affordance.kind, "scene_trigger");
        assert!(affordance.active);

        runtime.pause().unwrap();
        let paused = scene_route_commands::trigger(
            &runtime,
            "cw_shutdown_trigger_test",
            "save-cw-paused-trigger",
            before.world_epoch,
        )
        .unwrap();
        assert_trigger_receipt(&paused, "save-cw-paused-trigger", false);
        assert_eq!(paused.error_code.as_deref(), Some("E_RUNTIME_PAUSED"));
        assert!(runtime
            .snapshot()
            .unwrap()
            .interactables
            .iter()
            .find(|item| item.entity_id == "cw_shutdown_trigger_test")
            .is_some_and(|item| !item.active));
        runtime.resume().unwrap();

        let stale = scene_route_commands::trigger(
            &runtime,
            "cw_shutdown_trigger_test",
            "save-cw-stale-trigger",
            before.world_epoch + 1,
        )
        .unwrap();
        assert_trigger_receipt(&stale, "save-cw-stale-trigger", false);
        assert!(stale
            .error_code
            .as_deref()
            .is_some_and(|error| error.contains("StaleEpoch")));
        let unknown = scene_route_commands::trigger(
            &runtime,
            "browser_forged_trigger",
            "save-cw-unknown-trigger",
            before.world_epoch,
        )
        .unwrap();
        assert_trigger_receipt(&unknown, "save-cw-unknown-trigger", false);
        assert!(unknown
            .error_code
            .as_deref()
            .is_some_and(|error| error.contains("UnknownTrigger")));

        let forged = scene_route_commands::trigger(
            &runtime,
            "cw_shutdown_trigger_test",
            "save-cw-forged-shutdown",
            before.world_epoch,
        )
        .unwrap();
        assert_trigger_receipt(&forged, "save-cw-forged-shutdown", false);
        assert_eq!(
            forged.error_code.as_deref(),
            Some("E_CLOCKWORKS_SHUTDOWN_SOURCE_INVALID")
        );
        let after_forged = runtime.snapshot().unwrap();
        assert_eq!(after_forged.progression, before.progression);
        assert_eq!(after_forged.capabilities, before.capabilities);
        assert_eq!(after_forged.interactables, before.interactables);

        // Force a collision with the internal route-event request namespace.
        // The terminal must not become consumed unless its required route event is
        // recorded; scene, route and capability updates commit as one action.
        let collision_request = "scene-event-atomic-shutdown";
        let claimed = runtime
            .apply_route_command(RouteCommandRequest::Progress {
                event_id: "clockworks_valves".into(),
                request_id: collision_request.into(),
            })
            .unwrap();
        assert!(claimed.applied, "{:?}", claimed.error_code);
        let collision_base = runtime.snapshot().unwrap();
        let collision = runtime
            .activate_scene_interaction(
                "cw_master_shutdown_staged",
                "atomic-shutdown",
                collision_base.world_epoch,
            )
            .unwrap();
        assert!(!collision.applied);
        assert_eq!(
            collision.error_code.as_deref(),
            Some("E_SCENE_PROGRESS_DUPLICATE")
        );
        let after_collision = runtime.snapshot().unwrap();
        assert_eq!(
            after_collision.progression.event_seq,
            collision_base.progression.event_seq
        );
        assert_eq!(
            after_collision.capabilities.items,
            collision_base.capabilities.items
        );
        assert!(after_collision
            .interactables
            .iter()
            .any(|item| item.entity_id == "cw_master_shutdown_staged" && item.active));

        let view = runtime.snapshot().unwrap();
        let applied = runtime
            .activate_scene_interaction(
                "cw_master_shutdown_staged",
                "save-cw-shutdown",
                view.world_epoch,
            )
            .unwrap();
        assert_trigger_receipt(&applied.receipt, "save-cw-shutdown", true);
        assert_eq!(applied.receipt.snapshot.scene_id, "cw_shutdown_exit");
        assert_eq!(applied.receipt.snapshot.world_id, "clockworks");
        assert_eq!(applied.receipt.snapshot.world_epoch, view.world_epoch);
        assert_eq!(
            applied.receipt.snapshot.progression.event_seq,
            view.progression.event_seq + 1
        );
        assert!(!view.capabilities.items.iter().any(|item| {
            item.capability_id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP && item.granted
        }));
        assert!(applied
            .receipt
            .snapshot
            .capabilities
            .items
            .iter()
            .any(|item| {
                item.capability_id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP
                    && item.granted
                    && item.selected
            }));
        let duplicate = runtime
            .activate_scene_interaction(
                "cw_master_shutdown_staged",
                "save-cw-shutdown",
                view.world_epoch,
            )
            .unwrap();
        assert!(!duplicate.applied);
        assert_eq!(
            runtime.snapshot().unwrap().progression,
            applied.receipt.snapshot.progression
        );
        assert_eq!(
            runtime.snapshot().unwrap().capabilities,
            applied.receipt.snapshot.capabilities
        );
        assert!(duplicate
            .error_code
            .as_deref()
            .is_some_and(|error| error.contains("DuplicateRequest")));
        complete_world(&runtime, "clockworks", "save-cw-complete");
        runtime.save().unwrap();
        let save_v6 = save_v6::read_save(&path).unwrap();
        assert_eq!(save_v6.save.world_id, "clockworks");
        assert_eq!(save_v6.save.scene_id, "cw_shutdown_exit");
        let clockworks_progress = save_v6
            .save
            .progression
            .progress
            .iter()
            .find(|progress| progress.world_id == "clockworks")
            .unwrap();
        assert!(clockworks_progress
            .completed_events
            .contains(&"clockworks_shutdown".to_string()));
        let clockworks_scene = save_v6
            .save
            .scene_states
            .iter()
            .find(|scene| scene.scene_id == "cw_shutdown_exit")
            .unwrap();
        assert!(clockworks_scene
            .activated_ids
            .contains(&"cw_master_shutdown_staged".to_string()));
        assert!(clockworks_scene
            .emitted_event_ids
            .contains(&"clockworks_shutdown".to_string()));
        assert_eq!(
            save_v6
                .save
                .capabilities
                .grants
                .iter()
                .filter(|grant| {
                    grant.capability_id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP
                })
                .count(),
            1
        );
        assert!(save_v6
            .save
            .capabilities
            .selected
            .iter()
            .any(|id| { id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP }));
        assert!(!clockworks_scene
            .activated_ids
            .contains(&"cw_shutdown_trigger_test".to_string()));
        assert!(save_v6.world_persistent_v1.clockworks.regulator_defeated);
        save_v6.validate().unwrap();
        for world in ["grey_hive", "mist_harbor", "clockworks"] {
            assert!(
                save_v6
                    .save
                    .progression
                    .progress
                    .iter()
                    .find(|p| p.world_id == world)
                    .unwrap()
                    .completed
            );
        }
        save_v6.save.progression.clone()
    };
    {
        let restarted = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        restarted
            .load_scene_registry(
                scenes,
                "cw_shutdown_exit",
                &assets,
                &entities,
            )
            .unwrap();
        let restored = acknowledge_ready(&restarted, restarted.continue_saved().unwrap());
        assert_eq!(restored.world_id, "clockworks");
        assert_eq!(restored.scene_id, "cw_shutdown_exit");
        assert_eq!(restored.objectives.len(), 0);
        assert!(restored.capabilities.items.iter().any(|item| {
            item.capability_id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP
                && item.granted
                && item.selected
        }));
        assert!(restored.interactables.iter().any(|item| {
            item.entity_id == "cw_master_shutdown_staged" && item.kind == "terminal" && !item.active
        }));
        restarted.save().unwrap();
        let again = save_v6::read_save(&path).unwrap();
        assert!(again.world_persistent_v1.clockworks.regulator_defeated);
        let again = again.save;
        assert_eq!(again.progression, saved_progression);
        // The new shutdown visit emitted only Shutdown. Earlier Valves/Core
        // progression survives in the world route, not this scene instance ledger.
        assert_eq!(again.scene_states[0].emitted_event_ids, ["clockworks_shutdown"]);
        let restored_events = &again.progression.progress.iter()
            .find(|world| world.world_id == "clockworks").unwrap().completed_events;
        for event in ["clockworks_valves", "clockworks_core", "clockworks_shutdown"] {
            assert!(restored_events.contains(&event.to_owned()), "missing {event}");
        }
        assert!(again.capabilities.grants.iter().any(|grant| {
            grant.capability_id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP
        }));
        assert!(again
            .capabilities
            .selected
            .iter()
            .any(|id| { id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP }));
    }
    let _ = fs::remove_dir_all(path);
}

#[test]
fn air_step_save_without_clockworks_shutdown_event_is_rejected() {
    let path = root("air-step-without-shutdown");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_gh(&runtime);
    runtime.save().unwrap();
    let mut save = save_v6::read_save(&path).unwrap();
    save.save.capabilities.grants.push(
        wuxian_horror_ch1::capability_v1::CapabilityGrant {
            capability_id: wuxian_horror_ch1::capability_v1::CAP_AIR_STEP.into(),
            granted_at_revision: save.save.revision,
        },
    );
    save.save
        .capabilities
        .selected
        .push(wuxian_horror_ch1::capability_v1::CAP_AIR_STEP.into());
    save.progression
        .sync_capabilities(&save.save.capabilities);
    save.effect_sources = save.progression.resolve_rules().unwrap().0;

    assert_eq!(save.validate().unwrap_err(), "E_SAVE_CAPABILITY_ROUTE_INVALID");
    assert_eq!(
        save_v6::write_save(&path, &save).unwrap_err(),
        "E_SAVE_CAPABILITY_ROUTE_INVALID"
    );
    assert!(save_v6::read_save(&path).unwrap().save.capabilities.grants.is_empty());
    drop(runtime);
    let _ = fs::remove_dir_all(path);
}

fn assert_trigger_receipt(
    receipt: &wuxian_horror_ch1::world_v3::CommandReceipt,
    request_id: &str,
    applied: bool,
) {
    assert_eq!(receipt.command_id, request_id);
    assert_eq!(receipt.applied, applied);
    assert!(!receipt.already_applied);
    assert_eq!(receipt.snapshot.protocol_version, 3);
    assert_eq!(
        receipt.snapshot.schema_version,
        wuxian_horror_ch1::world_v3::WORLD_SCHEMA_VERSION
    );
    assert_eq!(receipt.world_epoch, receipt.snapshot.world_epoch);
    assert_eq!(receipt.server_tick, receipt.snapshot.server_tick);
    assert_eq!(
        receipt.authority_revision,
        receipt.snapshot.authority_revision
    );
    let wire = serde_json::to_value(receipt).unwrap();
    assert!(wire.get("commandId").is_some());
    assert!(wire.get("snapshot").is_some());
}

#[test]
fn formal_scene_trigger_rejects_outside_authored_polygon_without_projection_mutation() {
    let path = root("trigger-outside");
    let outside_trigger = CW.replace(
        "\"polygon\": [[0, 0], [2, 0], [2, 2], [0, 2]]",
        "\"polygon\": [[8, 8], [10, 8], [10, 10], [8, 10]]",
    );
    assert_ne!(
        outside_trigger, CW,
        "generic trigger fixture replacement matched"
    );
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    let before = runtime
        .load_scene_registry(
            [RS, GH, MH, outside_trigger.as_str()],
            "cw_test_exit",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
    let affordance = before
        .interactables
        .iter()
        .find(|item| item.entity_id == "cw_shutdown_trigger_test")
        .expect("out-of-polygon trigger remains discoverable");
    assert_eq!(affordance.kind, "scene_trigger");
    assert!(!affordance.active);

    let rejected = scene_route_commands::trigger(
        &runtime,
        "cw_shutdown_trigger_test",
        "outside-trigger",
        before.world_epoch,
    )
    .unwrap();
    assert_trigger_receipt(&rejected, "outside-trigger", false);
    assert!(rejected
        .error_code
        .as_deref()
        .is_some_and(|error| error.contains("OutOfRange")));
    assert_eq!(rejected.snapshot.scene_id, before.scene_id);
    assert_eq!(rejected.snapshot.world_epoch, before.world_epoch);
    assert_eq!(
        rejected.snapshot.progression.event_seq,
        before.progression.event_seq
    );
    assert_eq!(
        rejected.snapshot.capabilities.items,
        before.capabilities.items
    );
    let _ = fs::remove_dir_all(path);
}

fn authored_scene_catalogs() -> (BTreeSet<String>, BTreeSet<String>) {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../governance/assets/RUNTIME_ASSET_MANIFEST.json"
    )).unwrap();
    let assets = manifest["assets"].as_array().unwrap().iter()
        .filter(|asset| asset["admission"] == "release_approved")
        .map(|asset| asset["assetId"].as_str().unwrap().to_owned()).collect();
    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../content/enemies/entity-types.json"
    )).unwrap();
    let entities = catalog["entityTypes"].as_array().unwrap().iter()
        .map(|entity| entity.as_str().unwrap().to_owned()).collect();
    (assets, entities)
}

fn core_input(runtime: &FormalRuntime, view: &WorldView, x: f32, z: f32,
              combat: Vec<CombatIntentRequest>) -> WorldView {
    runtime.submit_input(
        InputSample::new(view.world_epoch, view.ack_seq + 1,
            view.server_time_ms + 1_000_000, x, z).unwrap(), combat
    ).unwrap()
}

fn move_core_player_to(runtime: &FormalRuntime, x: f32, z: f32) {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut view = runtime.snapshot().unwrap();
    for _ in 0..600 {
        assert!(Instant::now() < deadline, "timed out reaching ({x}, {z})");
        let position = view.player.transform.position_m;
        let (dx, dz) = (x - position.x_m, z - position.z_m);
        let distance = dx.hypot(dz);
        if distance <= 0.15 {
            core_input(runtime, &view, 0.0, 0.0, vec![]);
            return;
        }
        view = core_input(runtime, &view, dx / distance, dz / distance, vec![]);
        assert!(view.player.current_hp > 0, "player died on the authored Core route");
    }
    panic!("authored Core route did not reach ({x}, {z}) in 600 input acknowledgements");
}

fn defeat_and_activate_authored_core(runtime: &FormalRuntime) {
    const CONSOLE: &str = "cw_regulator_core_console_staged";
    let mut view = runtime.snapshot().unwrap();
    assert_eq!(view.scene_id, "cw_regulator_core");
    let early = runtime.activate_scene_interaction(CONSOLE, "save-core-before-defeat", view.world_epoch).unwrap();
    assert!(!early.applied);
    assert_eq!(early.error_code.as_deref(), Some("E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED"));
    let deadline = Instant::now() + Duration::from_secs(30);
    for _ in 0..600 {
        assert!(Instant::now() < deadline, "timed out defeating authored Regulator");
        let boss = view.actors.iter().find(|actor| actor.entity_id == "cw_prime_regulator")
            .expect("authored Regulator actor is present");
        if !boss.active { break; }
        let player = view.player.transform.position_m;
        let (dx, dz) = (boss.transform.position_m.x_m - player.x_m,
                        boss.transform.position_m.z_m - player.z_m);
        let distance = dx.hypot(dz);
        let (x, z, attacks) = if distance <= 1.5 {
            // One public combat request per acknowledged owner step executes
            // real damage; no HP, persistence flag or route event is injected.
            (0.0, 0.0, vec![CombatIntentRequest::Attack {
                request_id: view.ack_seq + 1,
            }])
        } else { (dx / distance, dz / distance, vec![]) };
        view = core_input(runtime, &view, x, z, attacks);
        assert!(view.player.current_hp > 0, "player died before authored Regulator defeat");
    }
    assert!(view.actors.iter().any(|actor| actor.entity_id == "cw_prime_regulator" && !actor.active));
    assert!(!view.progression.worlds.iter().find(|world| world.world_id == "clockworks").unwrap()
        .completed_events.contains(&"clockworks_core".to_owned()));
    move_core_player_to(runtime, 12.0, 7.0);
    interact(runtime, CONSOLE, "save-cw-authored-core");
    assert!(runtime.snapshot().unwrap().progression.worlds.iter()
        .find(|world| world.world_id == "clockworks").unwrap()
        .completed_events.contains(&"clockworks_core".to_owned()));
}

fn interact(runtime: &FormalRuntime, id: &str, request_id: &str) {
    let view = runtime.snapshot().unwrap();
    let response = runtime
        .activate_scene_interaction(id, request_id, view.world_epoch)
        .unwrap();
    assert!(response.applied, "{id}/{request_id}: {:?}", response.error_code);
}

fn complete_world(runtime: &FormalRuntime, world_id: &str, request_id: &str) {
    let response = runtime
        .apply_route_command(RouteCommandRequest::Complete {
            world_id: world_id.into(),
            request_id: request_id.into(),
        })
        .unwrap();
    assert!(response.applied, "{world_id}/{request_id}: {:?}", response.error_code);
    assert!(response.view.progression.worlds.iter()
        .any(|world| world.world_id == world_id && world.completed));
}

fn transition(runtime: &FormalRuntime, id: &str, request_id: &str) {
    let view = runtime.snapshot().unwrap();
    acknowledge_ready(
        &runtime,
        runtime.transition_scene(id, request_id, view.world_epoch).unwrap(),
    );
}

#[test]
fn strict_build_content_schema_and_slot_v6_are_checked_on_disk() {
    let path = root("strict-slot");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    runtime.save_slot("a", "A", true).unwrap();
    assert!(path.join("slots/a/slot-v6.json").is_file());
    let (_, save) = save_slots::read_slot_v6(&path, "a").unwrap();
    assert_eq!(save.schema_version, 6);
    let mut bad_build = save.clone();
    bad_build.save.build_version = "other-build".into();
    assert_eq!(
        bad_build.validate().unwrap_err(),
        "E_SAVE_BUILD_UNSUPPORTED"
    );
    let mut bad_content = save.clone();
    bad_content.save.content_version = "other-content".into();
    assert_eq!(
        bad_content.validate().unwrap_err(),
        "E_SAVE_CONTENT_UNSUPPORTED"
    );
    let mut raw = serde_json::to_value(&save).unwrap();
    raw["futureField"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SaveV6>(raw).is_err());
    let restarted = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    let continued = acknowledge_ready(&restarted, restarted.continue_slot("a").unwrap());
    assert_eq!(continued.world_id, "grey_hive");
    let _ = fs::remove_dir_all(path);
}

#[test]
fn legacy_v4_slot_is_backed_up_migrated_and_kept_readable() {
    let path = root("slot-v4-migration");
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    runtime.save().unwrap();
    let legacy_save = v4_json_from_v5(&save_v6::read_save(&path).unwrap().save);
    let document = serde_json::json!({
        "schemaVersion": 1,
        "slotId": "legacy",
        "displayName": "Legacy",
        "updatedAtMs": 123,
        "save": legacy_save,
    });
    let dir = path.join("slots/legacy");
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("slot-v4.json");
    let bytes = serde_json::to_vec_pretty(&document).unwrap();
    fs::write(&source, &bytes).unwrap();
    let (name, migrated) = save_slots::read_slot(&path, "legacy").unwrap();
    assert_eq!(name, "Legacy");
    assert_eq!(migrated.schema_version, 5);
    assert_eq!(
        migrated
            .migration_provenance
            .as_ref()
            .unwrap()
            .source_format,
        "slot-v4.json"
    );
    assert!(dir.join("slot-v5.json").is_file());
    assert_eq!(fs::read(&source).unwrap(), bytes);
    assert_eq!(fs::read(dir.join("slot-v4.json.bak")).unwrap(), bytes);
    let legacy = save_slots::list_slots(&path)
        .into_iter()
        .find(|slot| slot.slot_id == "legacy")
        .unwrap();
    assert!(!legacy.read_only);
    let _ = fs::remove_dir_all(path);
}

fn v4_json_from_v5(save: &SaveV5) -> serde_json::Value {
    let mut value = serde_json::to_value(save).unwrap();
    let object = value.as_object_mut().unwrap();
    object.insert("schemaVersion".into(), serde_json::json!(4));
    object.insert(
        "contentVersion".into(),
        serde_json::json!("grey-hive-first-flow/1"),
    );
    object.insert("checkpointId".into(), serde_json::json!("gh_cp_airlock"));
    for key in [
        "buildVersion",
        "sceneId",
        "inventory",
        "dialogFlags",
        "sceneStates",
    ] {
        object.remove(key);
    }
    let actors = object
        .remove("actors")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|actor| {
            let mut actor = actor.as_object().unwrap().clone();
            actor.remove("actorType");
            let actor_id = actor.remove("actorId").unwrap();
            actor.insert("entityId".into(), actor_id);
            serde_json::Value::Object(actor)
        })
        .collect::<Vec<_>>();
    object.insert("sentinels".into(), serde_json::Value::Array(actors));
    let progression = object.remove("progression").unwrap();
    object.insert("route".into(), progression);
    object.insert("combatHandledRequestIds".into(), serde_json::json!([]));
    object.insert("lastReceivedSeq".into(), serde_json::json!(0));
    object.insert("lastReceivedClientTimeMs".into(), serde_json::json!(0));
    value
}
