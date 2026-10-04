#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use std::{
    collections::BTreeSet,
    fs,
    sync::atomic::{AtomicU64, Ordering},
};
use wuxian_horror_ch1::world_persistent_v1::{
    WorldPersistentState, MIST_HARBOR_EXPLORATION_REGION_IDS,
};
use wuxian_horror_ch1::{formal_runtime::FormalRuntime, save_v6, world_v3::Vec3};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn all_nine_compiled_scenes_match_the_persistent_region_catalog() {
    let sources = [
        include_str!("../../content/scenes/compiled/mh_fog_pier.json"),
        include_str!("../../content/scenes/compiled/mh_tidal_warehouse.json"),
        include_str!("../../content/scenes/compiled/mh_signal_yard.json"),
        include_str!("../../content/scenes/compiled/mh_drowned_quay.json"),
        include_str!("../../content/scenes/compiled/mh_breakwater.json"),
        include_str!("../../content/scenes/compiled/mh_pump_station.json"),
        include_str!("../../content/scenes/compiled/mh_resonance_tower.json"),
        include_str!("../../content/scenes/compiled/mh_warden_arena.json"),
        include_str!("../../content/scenes/compiled/mh_extraction.json"),
    ];
    let mut actual = BTreeSet::new();
    for source in sources {
        let scene: serde_json::Value = serde_json::from_str(source).unwrap();
        assert_eq!(scene["worldId"], "mist_harbor");
        let regions = scene["explorationRegions"].as_array().unwrap();
        assert_eq!(regions.len(), 3);
        for region in regions {
            let id = region["id"].as_str().unwrap();
            assert!(actual.insert(id.to_owned()), "duplicate region {id}");
            assert_eq!(region["polygon"].as_array().unwrap().len(), 4);
        }
    }
    assert_eq!(
        actual,
        MIST_HARBOR_EXPLORATION_REGION_IDS
            .iter()
            .map(|id| (*id).to_owned())
            .collect()
    );
}

fn fixture_mist_harbor() -> String {
    let mut scene: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/scene-runtime-v1/mist_harbor.json")).unwrap();
    scene["explorationRegions"] = serde_json::json!([
        {"id":"mh_fp_entry_berth","polygon":[[0,0],[5,0],[5,12],[0,12]]},
        {"id":"mh_fp_foghorn_pier","polygon":[[5,0],[12,0],[12,12],[5,12]]}
    ]);
    scene.to_string()
}

fn load_fixture(runtime: &FormalRuntime, mist: &str) {
    runtime
        .load_scene_registry(
            [
                include_str!("fixtures/scene-runtime-v1/return_station.json"),
                include_str!("fixtures/scene-runtime-v1/grey_hive.json"),
                mist,
                include_str!("fixtures/scene-runtime-v1/clockworks.json"),
            ],
            "gh_test_power",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
}

fn interact(runtime: &FormalRuntime, id: &str, request: &str) {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    assert!(
        runtime
            .activate_scene_interaction(id, request, epoch)
            .unwrap()
            .applied
    );
}

fn transition(runtime: &FormalRuntime, id: &str, request: &str) {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    acknowledge_ready(&runtime, runtime.transition_scene(id, request, epoch).unwrap());
}

#[test]
fn actual_spawn_region_survives_scene_return_save_load_and_new_journey_resets() {
    let root = std::env::temp_dir().join(format!(
        "mh-exploration-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let mist = fixture_mist_harbor();
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        load_fixture(&runtime, &mist);
        for (id, request) in [
            ("gh_power_console_test", "explore-gh-power"),
            ("gh_lockdown_test", "explore-gh-lockdown"),
            ("gh_extract_test", "explore-gh-extraction"),
        ] {
            interact(&runtime, id, request);
        }
        transition(&runtime, "gh_to_hub", "explore-gh-hub");
        transition(&runtime, "rs_to_mh", "explore-rs-mh");
        let first = runtime.snapshot().unwrap();
        assert_eq!(first.scene_id, "mh_test_beacons");
        assert_eq!(
            first.mist_harbor_pump.unwrap().state,
            wuxian_horror_ch1::world_v3::MistHarborPumpState::Ready
        );
        assert!(first.capabilities.explored_map.rooms.is_empty());
        runtime.save().unwrap();
        let saved = save_v6::read_save(&root).unwrap();
        assert_eq!(
            saved.world_persistent_v1.mist_harbor.explored_region_ids,
            ["mh_fp_entry_berth"]
        );
        transition(&runtime, "mh_to_hub", "explore-mh-hub");
        transition(&runtime, "rs_to_mh", "explore-rs-mh-revisit");
        runtime.save().unwrap();
        assert_eq!(
            save_v6::read_save(&root)
                .unwrap()
                .world_persistent_v1
                .mist_harbor
                .explored_region_ids,
            ["mh_fp_entry_berth"]
        );
    }
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        load_fixture(&runtime, &mist);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(runtime.snapshot().unwrap().scene_id, "mh_test_beacons");
        runtime.save().unwrap();
        assert_eq!(
            save_v6::read_save(&root)
                .unwrap()
                .world_persistent_v1
                .mist_harbor
                .explored_region_ids,
            ["mh_fp_entry_berth"]
        );
        acknowledge_ready(&runtime, runtime.reset_new().unwrap());
        runtime.save().unwrap();
        assert!(save_v6::read_save(&root)
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .explored_region_ids
            .is_empty());
    }
    let resolved_root = root.canonicalize().unwrap();
    let resolved_temp = std::env::temp_dir().canonicalize().unwrap();
    assert!(resolved_root.starts_with(&resolved_temp));
    assert!(resolved_root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("mh-exploration-"));
    fs::remove_dir_all(resolved_root).unwrap();
}

#[test]
fn legacy_continue_near_a_region_border_does_not_reveal_the_authored_spawn() {
    let root = std::env::temp_dir().join(format!(
        "mh-exploration-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let mist = fixture_mist_harbor();
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        load_fixture(&runtime, &mist);
        for (id, request) in [
            ("gh_power_console_test", "border-gh-power"),
            ("gh_lockdown_test", "border-gh-lockdown"),
            ("gh_extract_test", "border-gh-extraction"),
        ] {
            interact(&runtime, id, request);
        }
        transition(&runtime, "gh_to_hub", "border-gh-hub");
        transition(&runtime, "rs_to_mh", "border-rs-mh");
        runtime.save().unwrap();
    }
    let mut legacy = save_v6::read_save(&root).unwrap();
    legacy
        .world_persistent_v1
        .mist_harbor
        .explored_region_ids
        .clear();
    let position = Vec3::new(5.1, 0.0, 6.0).unwrap();
    legacy.save.player.position_m = position;
    legacy.save.explored.player_position_m = position;
    legacy.save.explored.rooms.clear();
    legacy.save.capabilities.explored_map = legacy.save.explored.clone();
    save_v6::write_save(&root, &legacy).unwrap();
    {
        let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
        load_fixture(&runtime, &mist);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        runtime.save().unwrap();
        assert!(save_v6::read_save(&root)
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .explored_region_ids
            .is_empty());
        assert!(runtime
            .snapshot()
            .unwrap()
            .capabilities
            .explored_map
            .rooms
            .is_empty());
    }
    let resolved_root = root.canonicalize().unwrap();
    let resolved_temp = std::env::temp_dir().canonicalize().unwrap();
    assert!(resolved_root.starts_with(&resolved_temp));
    assert!(resolved_root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("mh-exploration-"));
    fs::remove_dir_all(resolved_root).unwrap();
}

#[test]
fn save_exploration_field_is_backward_compatible_and_strict() {
    let old_v6_world = serde_json::json!({ "mistHarbor": { "pump": { "state": "ready", "drainCompleteAtWorldTimeMs": null } } });
    let mut persistent: WorldPersistentState = serde_json::from_value(old_v6_world).unwrap();
    assert!(persistent.mist_harbor.explored_region_ids.is_empty());
    assert!(persistent.validate().is_ok());

    for id in MIST_HARBOR_EXPLORATION_REGION_IDS {
        assert!(persistent.mark_explored(id).unwrap());
    }
    assert_eq!(persistent.mist_harbor.explored_region_ids.len(), 27);
    assert!(persistent.validate().is_ok());
    let round_trip: WorldPersistentState =
        serde_json::from_value(serde_json::to_value(&persistent).unwrap()).unwrap();
    assert_eq!(round_trip, persistent);

    let mut duplicate = persistent.clone();
    duplicate
        .mist_harbor
        .explored_region_ids
        .push("mh_fp_entry_berth".into());
    assert!(duplicate.validate().is_err());
    let mut unknown = persistent;
    unknown.mist_harbor.explored_region_ids[0] = "mh_not_a_compiled_region".into();
    assert!(unknown.validate().is_err());
}
