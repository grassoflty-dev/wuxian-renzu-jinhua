#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use wuxian_horror_ch1::{
    continuous_kcc::{step_kcc, Aabb, KccBody, KccTerrainRegion, StaticKccWorld},
    effects::TerrainTag,
    formal_runtime::FormalRuntime,
    player_rules::{EffectivePlayerRules, MovementMode},
    save_v5, save_v6,
    world_persistent_v1::{
        PumpStartResult, PumpStateError, PumpStatus, WorldPersistentState, DROWNED_QUAY_WATER_ID,
        MIST_HARBOR_ID, PUMP_CONTROL_ID, PUMP_DRAIN_DURATION_MS, PUMP_EAST_WATER_ID,
    },
    world_v3::Vec3,
};

const RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");
static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

fn root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "mh-pump-v1-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn pump_scene() -> serde_json::Value {
    let mut scene: serde_json::Value = serde_json::from_str(MH).unwrap();
    scene["sceneId"] = "mh_pump_station".into();
    scene["boundsM"] = serde_json::json!({"x":0,"z":0,"width":24,"depth":16});
    scene["navigation"]["nodes"][0]["position"] = serde_json::json!([17, 0, 7]);
    scene["spawns"][0]["position"] = serde_json::json!([17, 0, 7]);
    for interaction in scene["interactions"].as_array_mut().unwrap() {
        interaction["position"] = serde_json::json!([17, 0, 7]);
    }
    scene["interactions"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": PUMP_CONTROL_ID, "kind": "pump_control", "event": null,
            "position": [17, 0, 7]
        }));
    scene["walkablePolygons"] = serde_json::json!([{
        "id":"mh_pump_station_walkable_area",
        "polygon":[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]
    }]);
    scene["explorationRegions"] = serde_json::json!([{
        "id":"mh_ps_control_hall",
        "polygon":[[10.5,0.5],[18.0,0.5],[18.0,15.5],[10.5,15.5]]
    }]);
    scene["terrainRegions"] = serde_json::json!([{
        "id":PUMP_EAST_WATER_ID, "tag":"terrain.water_deep",
        "polygon":[[18.0,5.5],[23.5,5.5],[23.5,10.5],[18.0,10.5]]
    }]);
    scene["transitions"][0]["polygon"] = serde_json::json!([[16, 6], [18, 6], [18, 8], [16, 8]]);
    scene
}

fn station_scene() -> serde_json::Value {
    let mut station: serde_json::Value = serde_json::from_str(RS).unwrap();
    let to_mh = station["transitions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["id"] == "rs_to_mh")
        .unwrap();
    to_mh["toSceneId"] = "mh_pump_station".into();
    station
}

fn load_fixture(runtime: &FormalRuntime) {
    let station = station_scene().to_string();
    let pump = pump_scene().to_string();
    runtime
        .load_scene_registry(
            [station.as_str(), GH, pump.as_str(), CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
}

fn interact(runtime: &FormalRuntime, id: &str, request: &str) {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    let result = runtime
        .activate_scene_interaction(id, request, epoch)
        .unwrap();
    assert!(result.applied, "{id}: {:?}", result.error_code);
}

fn transition(runtime: &FormalRuntime, id: &str, request: &str) {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    acknowledge_ready(&runtime, runtime.transition_scene(id, request, epoch).unwrap());
}

fn enter_pump_station(runtime: &FormalRuntime) {
    for (id, request) in [
        ("gh_power_console_test", "pump-gh-power"),
        ("gh_lockdown_test", "pump-gh-lockdown"),
        ("gh_extract_test", "pump-gh-extraction"),
    ] {
        interact(runtime, id, request);
    }
    transition(runtime, "gh_to_hub", "pump-gh-hub");
    transition(runtime, "rs_to_mh", "pump-rs-mh");
    assert_eq!(runtime.snapshot().unwrap().scene_id, "mh_pump_station");
}

fn point(x_m: f32, z_m: f32) -> Vec3 {
    Vec3 { x_m, y_m: 0.0, z_m }
}

fn water_world(persistent: &WorldPersistentState, mode: MovementMode) -> StaticKccWorld {
    let mut world = StaticKccWorld::new(Aabb::new(0.5, 23.5, 0.5, 15.5).unwrap(), vec![])
        .with_walkable_polygons(vec![vec![
            [0.5, 0.5],
            [23.5, 0.5],
            [23.5, 15.5],
            [0.5, 15.5],
        ]])
        .with_terrain_regions(vec![
            KccTerrainRegion {
                id: PUMP_EAST_WATER_ID.into(),
                tag: TerrainTag::WaterDeep,
                polygon: vec![[18.0, 5.5], [23.5, 5.5], [23.5, 10.5], [18.0, 10.5]],
                surface_velocity_mps: None,
            },
            KccTerrainRegion {
                id: DROWNED_QUAY_WATER_ID.into(),
                tag: TerrainTag::WaterShallow,
                polygon: vec![[13.0, 5.0], [19.0, 5.0], [19.0, 11.0], [13.0, 11.0]],
                surface_velocity_mps: None,
            },
        ])
        .with_movement_rules(EffectivePlayerRules::default(), mode);
    world.map_terrain_tags(|id, tag| persistent.resolve_terrain_tag(MIST_HARBOR_ID, id, tag));
    world
}

#[test]
fn pump_state_has_exact_world_time_deadline_and_is_idempotent() {
    let mut persistent = WorldPersistentState::default();
    let pump = &mut persistent.mist_harbor.pump;
    assert_eq!(
        pump.start("grey_hive", true, 100),
        Err(PumpStateError::WrongWorld)
    );
    assert_eq!(
        pump.start(MIST_HARBOR_ID, false, 100),
        Err(PumpStateError::EastBeaconIncomplete)
    );
    assert_eq!(pump.state, PumpStatus::Ready);
    assert_eq!(
        pump.start(MIST_HARBOR_ID, true, 100),
        Ok(PumpStartResult::Started)
    );
    assert_eq!(
        pump.drain_complete_at_world_time_ms,
        Some(100 + PUMP_DRAIN_DURATION_MS)
    );
    assert_eq!(
        pump.start(MIST_HARBOR_ID, true, 101),
        Ok(PumpStartResult::AlreadyDraining)
    );
    assert!(!pump.resolve_at(100 + 5_999).unwrap());
    assert_eq!(pump.state, PumpStatus::Draining);
    assert!(pump.resolve_at(100 + 6_000).unwrap());
    assert_eq!(pump.state, PumpStatus::Drained);
    assert!(!pump.resolve_at(100 + 6_001).unwrap());
    assert_eq!(
        pump.start(MIST_HARBOR_ID, true, 100 + 6_001),
        Ok(PumpStartResult::AlreadyDrained)
    );
    persistent.reset_new_journey();
    assert_eq!(persistent.mist_harbor.pump.state, PumpStatus::Ready);
}

#[test]
fn terrain_rules_block_ground_allow_hover_flight_and_change_only_two_water_regions() {
    let mut persistent = WorldPersistentState::default();
    let ready = water_world(&persistent, MovementMode::Ground);
    assert_eq!(
        ready.terrain_tag(PUMP_EAST_WATER_ID),
        Some(TerrainTag::WaterDeep)
    );
    assert_eq!(
        ready.terrain_tag(DROWNED_QUAY_WATER_ID),
        Some(TerrainTag::WaterShallow)
    );
    assert!(!ready.can_occupy(point(18.5, 7.0), 0.35));
    assert!(ready.can_occupy(point(17.0, 7.0), 0.35));
    let hover = water_world(&persistent, MovementMode::Hover);
    let flight = water_world(&persistent, MovementMode::Flight);
    assert!(hover.can_occupy(point(18.5, 7.0), 0.35));
    assert!(flight.can_occupy(point(18.5, 7.0), 0.35));
    assert_eq!(persistent.mist_harbor.pump.state, PumpStatus::Ready);

    let mut before = KccBody::new(point(15.0, 7.0));
    step_kcc(&mut before, &ready, (1.0, 0.0), 0.1).unwrap();
    assert!((before.velocity_mps.x_m - 2.4).abs() < 1e-4);
    persistent
        .mist_harbor
        .pump
        .start(MIST_HARBOR_ID, true, 0)
        .unwrap();
    assert!(!water_world(&persistent, MovementMode::Ground).can_occupy(point(18.5, 7.0), 0.35));
    persistent.resolve_at(6_000).unwrap();
    let drained = water_world(&persistent, MovementMode::Ground);
    assert_eq!(
        drained.terrain_tag(PUMP_EAST_WATER_ID),
        Some(TerrainTag::WetFloor)
    );
    assert_eq!(
        drained.terrain_tag(DROWNED_QUAY_WATER_ID),
        Some(TerrainTag::WetFloor)
    );
    assert!(drained.can_occupy(point(18.5, 7.0), 0.35));
    let mut after = KccBody::new(point(15.0, 7.0));
    step_kcc(&mut after, &drained, (1.0, 0.0), 0.1).unwrap();
    assert!((after.velocity_mps.x_m - 4.0).abs() < 1e-4);
    assert_eq!(
        persistent.resolve_terrain_tag("grey_hive", PUMP_EAST_WATER_ID, TerrainTag::WaterDeep),
        TerrainTag::WaterDeep
    );
}

#[test]
fn authored_pump_rechecks_beacon_and_does_not_reveal_map_or_repeat_activation() {
    let path = root();
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    load_fixture(&runtime);
    enter_pump_station(&runtime);
    let before = runtime.snapshot().unwrap();
    let control = before
        .interactables
        .iter()
        .find(|item| item.entity_id == PUMP_CONTROL_ID)
        .unwrap();
    assert!(!control.active);
    let rejected = runtime
        .activate_scene_interaction(PUMP_CONTROL_ID, "pump-too-early", before.world_epoch)
        .unwrap();
    assert!(!rejected.applied);
    assert_eq!(
        rejected.error_code.as_deref(),
        Some("E_PUMP_EastBeaconIncomplete")
    );
    assert_eq!(
        runtime.snapshot().unwrap().capabilities.explored_map,
        before.capabilities.explored_map
    );

    interact(&runtime, "mh_east_test", "pump-east-beacon");
    let before_start = runtime.snapshot().unwrap();
    assert!(
        before_start
            .interactables
            .iter()
            .find(|item| item.entity_id == PUMP_CONTROL_ID)
            .unwrap()
            .active
    );
    let started = runtime
        .activate_scene_interaction(PUMP_CONTROL_ID, "pump-start", before_start.world_epoch)
        .unwrap();
    assert!(started.applied);
    assert!(!started.already_applied);
    assert_eq!(
        started.view.capabilities.explored_map,
        before_start.capabilities.explored_map
    );
    let second = runtime
        .activate_scene_interaction(PUMP_CONTROL_ID, "pump-second-f", started.view.world_epoch)
        .unwrap();
    assert!(!second.applied);
    assert!(second.already_applied);
    assert!(second.events.is_empty());
    runtime.save().unwrap();
    let saved = save_v6::read_save(&path).unwrap();
    assert_eq!(
        saved.world_persistent_v1.mist_harbor.pump.state,
        PumpStatus::Draining
    );
    assert!(saved
        .save
        .progression
        .progress
        .iter()
        .find(|progress| progress.world_id == MIST_HARBOR_ID)
        .unwrap()
        .completed_events
        .contains(&"mist_beacon_east".to_string()));
    assert!(!saved
        .save
        .progression
        .progress
        .iter()
        .find(|progress| progress.world_id == MIST_HARBOR_ID)
        .unwrap()
        .completed_events
        .contains(&"mist_pump".to_string()));

    transition(&runtime, "mh_to_hub", "pump-return-hub");
    assert_eq!(runtime.snapshot().unwrap().world_id, "return_station");
    transition(&runtime, "rs_to_mh", "pump-revisit-mh");
    assert_eq!(runtime.snapshot().unwrap().scene_id, "mh_pump_station");
    runtime.save().unwrap();
    assert_ne!(
        save_v6::read_save(&path)
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .pump
            .state,
        PumpStatus::Ready
    );
    acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    runtime.save().unwrap();
    assert_eq!(
        save_v6::read_save(&path)
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .pump
            .state,
        PumpStatus::Ready
    );
    drop(runtime);
    let _ = fs::remove_dir_all(path);
}

#[test]
fn draining_save_continues_and_expired_deadline_resolves_on_load() {
    let path = root();
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_fixture(&runtime);
        enter_pump_station(&runtime);
        interact(&runtime, "mh_east_test", "save-east-beacon");
        interact(&runtime, PUMP_CONTROL_ID, "save-pump-start");
        runtime.save().unwrap();
    }
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_fixture(&runtime);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        runtime.save().unwrap();
        assert_eq!(
            save_v6::read_save(&path)
                .unwrap()
                .world_persistent_v1
                .mist_harbor
                .pump
                .state,
            PumpStatus::Draining
        );
    }
    let mut expired = save_v6::read_save(&path).unwrap();
    let deadline = expired
        .world_persistent_v1
        .mist_harbor
        .pump
        .drain_complete_at_world_time_ms
        .unwrap();
    expired.save.server_time_ms = deadline;
    save_v6::write_save(&path, &expired).unwrap();
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_fixture(&runtime);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        runtime.save().unwrap();
        assert_eq!(
            save_v6::read_save(&path)
                .unwrap()
                .world_persistent_v1
                .mist_harbor
                .pump
                .state,
            PumpStatus::Drained
        );
        let view = runtime.snapshot().unwrap();
        assert!(
            !view
                .interactables
                .iter()
                .find(|item| item.entity_id == PUMP_CONTROL_ID)
                .unwrap()
                .active
        );
        transition(&runtime, "mh_to_hub", "drained-return-hub");
        transition(&runtime, "rs_to_mh", "drained-revisit-mh");
        runtime.save().unwrap();
        assert_eq!(
            save_v6::read_save(&path)
                .unwrap()
                .world_persistent_v1
                .mist_harbor
                .pump
                .state,
            PumpStatus::Drained
        );
    }
    {
        let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
        load_fixture(&runtime);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        runtime.save().unwrap();
        assert_eq!(
            save_v6::read_save(&path)
                .unwrap()
                .world_persistent_v1
                .mist_harbor
                .pump
                .state,
            PumpStatus::Drained
        );
    }
    let _ = fs::remove_dir_all(path);
}

#[test]
fn old_v6_and_v5_saves_default_to_ready_without_changing_legacy_file() {
    let path = root();
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    runtime.save().unwrap();
    let v6 = save_v6::read_save(&path).unwrap();
    let mut corrupt_pump = v6.clone();
    corrupt_pump.world_persistent_v1.mist_harbor.pump.state = PumpStatus::Draining;
    assert!(corrupt_pump.validate().is_err());
    let mut value = serde_json::to_value(&v6).unwrap();
    value.as_object_mut().unwrap().remove("worldPersistentV1");
    fs::write(
        save_v6::save_path(&path),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    assert_eq!(
        save_v6::read_save(&path)
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .pump
            .state,
        PumpStatus::Ready
    );
    save_v5::write_save(&path, &v6.save).unwrap();
    let original = fs::read(save_v5::save_path(&path)).unwrap();
    fs::remove_file(save_v6::save_path(&path)).unwrap();
    drop(runtime);
    let migrated = save_v6::read_or_migrate(&path).unwrap();
    assert_eq!(
        migrated.world_persistent_v1.mist_harbor.pump.state,
        PumpStatus::Ready
    );
    assert_eq!(fs::read(save_v5::save_path(&path)).unwrap(), original);
    let _ = fs::remove_dir_all(path);
}

#[test]
fn bad_terrain_geometry_and_unknown_tags_fail_closed() {
    let station = station_scene().to_string();
    let mut wrong_shape = pump_scene();
    wrong_shape["terrainRegions"][0]["polygon"][0] = serde_json::json!([17.5, 5.5]);
    let bad_shape = wrong_shape.to_string();
    let path = root();
    let runtime = FormalRuntime::new_with_save_dir(path.clone()).unwrap();
    assert!(runtime
        .load_scene_registry(
            [station.as_str(), GH, bad_shape.as_str(), CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new()
        )
        .is_err());

    let mut unknown_tag = pump_scene();
    unknown_tag["terrainRegions"][0]["tag"] = "terrain.water_unknown".into();
    let bad_tag = unknown_tag.to_string();
    assert!(runtime
        .load_scene_registry(
            [station.as_str(), GH, bad_tag.as_str(), CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new()
        )
        .is_err());

    let mut bad_walkable = pump_scene();
    bad_walkable["walkablePolygons"][0]["polygon"][0] = serde_json::json!([1.0, 0.5]);
    let bad_walkable = bad_walkable.to_string();
    assert!(runtime
        .load_scene_registry(
            [station.as_str(), GH, bad_walkable.as_str(), CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new()
        )
        .is_err());

    let mut duplicate_region = pump_scene();
    duplicate_region["explorationRegions"][0]["id"] = "mh_pump_station_walkable_area".into();
    let duplicate_region = duplicate_region.to_string();
    assert!(runtime
        .load_scene_registry(
            [station.as_str(), GH, duplicate_region.as_str(), CW],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new()
        )
        .is_err());
    drop(runtime);
    let _ = fs::remove_dir_all(path);
}
