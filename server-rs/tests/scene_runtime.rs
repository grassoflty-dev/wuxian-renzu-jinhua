#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;

use serde_json::Value;
use std::collections::BTreeSet;
use wuxian_horror_ch1::{
    formal_runtime::FormalRuntime,
    scene_registry::WorldRegistry,
    scene_runtime::{SceneRuntime, SceneRuntimeError},
    world_v3::Vec3,
};

const RS: &str = include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH: &str = include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH: &str = include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW: &str = include_str!("fixtures/scene-runtime-v1/clockworks.json");
const CW_PRESSURE_HALL: &str = include_str!("../../content/scenes/compiled/cw_pressure_hall.json");
const CW_ENTRY_FOUNDRY: &str = include_str!("../../content/scenes/compiled/cw_entry_foundry.json");
const CW_CONVEYOR_BRIDGE: &str =
    include_str!("../../content/scenes/compiled/cw_conveyor_bridge.json");
const CW_BOILER_CHAMBER: &str =
    include_str!("../../content/scenes/compiled/cw_boiler_chamber.json");
const CW_GEAR_SHAFT: &str = include_str!("../../content/scenes/compiled/cw_gear_shaft.json");
const CW_FURNACE_HEART: &str = include_str!("../../content/scenes/compiled/cw_furnace_heart.json");
const CW_FORGED_GUARD_ARENA: &str =
    include_str!("../../content/scenes/compiled/cw_forged_guard_arena.json");
const CW_REGULATOR_CORE: &str =
    include_str!("../../content/scenes/compiled/cw_regulator_core.json");
const CW_SHUTDOWN_EXIT: &str = include_str!("../../content/scenes/compiled/cw_shutdown_exit.json");
const COMPILED_CLOCKWORKS_SCENES: [&str; 9] = [
    CW_ENTRY_FOUNDRY,
    CW_PRESSURE_HALL,
    CW_CONVEYOR_BRIDGE,
    CW_BOILER_CHAMBER,
    CW_GEAR_SHAFT,
    CW_FURNACE_HEART,
    CW_FORGED_GUARD_ARENA,
    CW_REGULATOR_CORE,
    CW_SHUTDOWN_EXIT,
];

fn registry() -> WorldRegistry {
    WorldRegistry::load([RS, GH, MH, CW], &BTreeSet::new(), &BTreeSet::new()).unwrap()
}

fn pressure_hall_registry() -> (WorldRegistry, Value) {
    let mut hall: Value = serde_json::from_str(CW_PRESSURE_HALL).unwrap();
    hall["presentation"]["sprites"] = serde_json::json!([]);
    hall["transitions"] = serde_json::json!([]);
    hall["spawns"] = serde_json::Value::Array(
        hall["spawns"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|spawn| spawn["kind"] == "player")
            .cloned()
            .collect(),
    );
    let encoded = serde_json::to_string(&hall).unwrap();
    let registry = WorldRegistry::load(
        [RS, GH, MH, CW, encoded.as_str()],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap();
    (registry, hall)
}

fn staged_clockworks_registry() -> WorldRegistry {
    let scenes = COMPILED_CLOCKWORKS_SCENES
        .iter()
        .map(|raw| {
            let mut scene: Value = serde_json::from_str(raw).unwrap();
            scene["presentation"] = serde_json::json!({
                "cameraProfile": "oblique_default",
                "layers": [],
                "sprites": [],
            });
            scene["spawns"] = serde_json::Value::Array(
                scene["spawns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|spawn| spawn["kind"] == "player")
                    .cloned()
                    .collect(),
            );
            scene["collision"] = serde_json::Value::Array(
                scene["collision"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|collision| collision["requiresActorFirstKill"].is_null())
                    .cloned()
                    .collect(),
            );
            scene["doors"] = serde_json::json!([]);
            scene["vfxMarkers"] = serde_json::json!([]);
            serde_json::to_string(&scene).unwrap()
        })
        .collect::<Vec<_>>();
    let mut documents = vec![
        RS.to_string(),
        GH.to_string(),
        MH.to_string(),
        CW.to_string(),
    ];
    documents.extend(scenes);
    WorldRegistry::load(documents, &BTreeSet::new(), &BTreeSet::new()).unwrap()
}

#[test]
fn forged_guard_arena_authors_one_attackable_elite_and_preserves_ordinary_guard_types() {
    let arena: Value = serde_json::from_str(CW_FORGED_GUARD_ARENA).unwrap();
    let elite: Vec<_> = arena["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spawn| spawn["entityType"] == "enemy.clockworks.forged_guard_elite")
        .collect();
    assert_eq!(elite.len(), 1);
    assert_eq!(elite[0]["id"], "cw_forged_guard_elite");
    assert_eq!(elite[0]["kind"], "enemy");
    assert_eq!(elite[0]["position"], serde_json::json!([12.0, 0.0, 7.0]));
    let shutter_blockers: Vec<_> = arena["collision"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|collision| collision["requiresActorFirstKill"].is_string())
        .map(|collision| {
            (
                collision["id"].as_str().unwrap(),
                collision["requiresActorFirstKill"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        shutter_blockers,
        [
            ("cw_arena_shutter_west_blocker", "cw_forged_guard_elite"),
            ("cw_arena_shutter_east_blocker", "cw_forged_guard_elite"),
        ]
    );
    let scene_documents = |arena_scene: &Value| {
        let mut documents = vec![
            RS.to_string(),
            GH.to_string(),
            MH.to_string(),
            CW.to_string(),
        ];
        documents.extend(COMPILED_CLOCKWORKS_SCENES.iter().map(|raw| {
            let mut scene: Value = if *raw == CW_FORGED_GUARD_ARENA {
                arena_scene.clone()
            } else {
                serde_json::from_str(raw).unwrap()
            };
            scene["presentation"] = serde_json::json!({
                "cameraProfile": "oblique_default",
                "layers": [],
                "sprites": [],
            });
            scene["spawns"] = serde_json::Value::Array(
                scene["spawns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|spawn| {
                        spawn["kind"] == "player"
                            || (scene["sceneId"] == "cw_forged_guard_arena"
                                && spawn["id"] == "cw_forged_guard_elite")
                    })
                    .cloned()
                    .collect(),
            );
            scene["doors"] = serde_json::json!([]);
            scene["vfxMarkers"] = serde_json::json!([]);
            serde_json::to_string(&scene).unwrap()
        }));
        documents
    };
    let elite_entity_catalog = BTreeSet::from(["enemy.clockworks.forged_guard_elite".to_owned()]);
    let valid_documents = scene_documents(&arena);
    assert!(
        WorldRegistry::load(
            valid_documents.iter().map(String::as_str),
            &BTreeSet::new(),
            &elite_entity_catalog
        )
        .is_ok(),
        "the scene fixture with its single canonical elite and valid shutter references loads"
    );

    let mut malformed_arena = arena.clone();
    malformed_arena["collision"][6]["requiresActorFirstKill"] =
        Value::String("unknown_actor".into());
    let malformed_documents = scene_documents(&malformed_arena);
    assert_eq!(
        WorldRegistry::load(
            malformed_documents.iter().map(String::as_str),
            &BTreeSet::new(),
            &elite_entity_catalog
        )
        .unwrap_err(),
        SceneRuntimeError::MalformedSceneDefinition,
        "compiled scenes reject unknown actor references"
    );
    assert!(!arena["interactions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "cw_forged_guard_elite_staged"));

    let ordinary: Vec<_> = [
        CW_ENTRY_FOUNDRY,
        CW_PRESSURE_HALL,
        CW_CONVEYOR_BRIDGE,
        CW_BOILER_CHAMBER,
        CW_GEAR_SHAFT,
        CW_FURNACE_HEART,
    ]
    .into_iter()
    .flat_map(|raw| {
        serde_json::from_str::<Value>(raw).unwrap()["spawns"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|spawn| spawn["kind"] == "enemy")
            .map(|spawn| spawn["entityType"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
    let mut counts = std::collections::BTreeMap::new();
    for entity_type in ordinary { *counts.entry(entity_type).or_insert(0usize) += 1; }
    assert_eq!(counts, std::collections::BTreeMap::from([
        ("enemy.clockworks.forged_guard".to_owned(), 11),
        ("enemy.clockworks.pressure_drone".to_owned(), 7),
        ("enemy.clockworks.furnace_hound".to_owned(), 6),
    ]));
}

fn interaction_position(scene: &Value, id: &str) -> Vec3 {
    let position = scene["interactions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()["position"]
        .as_array()
        .unwrap();
    Vec3 {
        x_m: position[0].as_f64().unwrap() as f32,
        y_m: position[1].as_f64().unwrap() as f32,
        z_m: position[2].as_f64().unwrap() as f32,
    }
}

fn transition_position(scene: &Value, id: &str) -> Vec3 {
    let polygon = scene["transitions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap()["polygon"]
        .as_array()
        .unwrap();
    let (x_sum, z_sum) = polygon.iter().fold((0.0_f64, 0.0_f64), |(x, z), point| {
        (
            x + point[0].as_f64().unwrap(),
            z + point[1].as_f64().unwrap(),
        )
    });
    Vec3 {
        x_m: (x_sum / polygon.len() as f64) as f32,
        y_m: 0.0,
        z_m: (z_sum / polygon.len() as f64) as f32,
    }
}

#[test]
fn pressure_hall_aggregate_emits_once_after_every_valve_in_any_order() {
    const VALVES: [&str; 3] = [
        "cw_pressure_valve_01_staged",
        "cw_pressure_valve_02_staged",
        "cw_pressure_valve_03_staged",
    ];
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    for (case, order) in ORDERS.iter().enumerate() {
        let (registry, hall) = pressure_hall_registry();
        let mut runtime = SceneRuntime::new(registry, "cw_pressure_hall", 12).unwrap();
        assert!(!runtime.event_complete("clockworks_valves"));
        assert_eq!(
            runtime.interact(
                "cw_clockworks_valves_staged",
                &format!("aggregate-marker-{case}"),
                12,
                interaction_position(&hall, "cw_clockworks_valves_staged"),
            ),
            Err(SceneRuntimeError::StaticMarker),
            "the aggregate marker cannot emit the event directly",
        );
        assert_eq!(
            runtime.interact(
                VALVES[order[0]],
                &format!("stale-{case}"),
                11,
                interaction_position(&hall, VALVES[order[0]]),
            ),
            Err(SceneRuntimeError::StaleEpoch),
        );
        assert_eq!(
            runtime.interact(
                VALVES[order[0]],
                &format!("range-{case}"),
                12,
                Vec3 {
                    x_m: 0.0,
                    y_m: 0.0,
                    z_m: 0.0
                },
            ),
            Err(SceneRuntimeError::OutOfRange),
        );
        assert!(!runtime.object_activated(VALVES[order[0]]));
        assert!(!runtime.event_complete("clockworks_valves"));

        let mut emitted = Vec::new();
        for (count, valve_index) in order.iter().enumerate() {
            let request_id = format!("valve-{case}-{count}");
            let events = runtime
                .interact(
                    VALVES[*valve_index],
                    &request_id,
                    12,
                    interaction_position(&hall, VALVES[*valve_index]),
                )
                .unwrap();
            let event_id = match &events[0] {
                wuxian_horror_ch1::scene_runtime::SceneEvent::Interaction { event_id, .. } => {
                    event_id.clone()
                }
                event => panic!("unexpected valve event: {event:?}"),
            };
            if count < 2 {
                assert_eq!(event_id, None);
                assert!(!runtime.event_complete("clockworks_valves"));
            } else {
                assert_eq!(event_id.as_deref(), Some("clockworks_valves"));
                assert!(runtime.event_complete("clockworks_valves"));
                emitted.push(event_id);
            }
        }
        assert_eq!(emitted.len(), 1);
        assert_eq!(
            runtime.interact(
                VALVES[order[2]],
                &format!("valve-duplicate-{case}"),
                12,
                interaction_position(&hall, VALVES[order[2]]),
            ),
            Err(SceneRuntimeError::AlreadyApplied),
        );
        assert_eq!(
            runtime.interact(
                VALVES[order[2]],
                &format!("valve-{case}-2"),
                12,
                interaction_position(&hall, VALVES[order[2]]),
            ),
            Err(SceneRuntimeError::DuplicateRequest),
        );
        assert_eq!(emitted.len(), 1);
        assert!(runtime.event_complete("clockworks_valves"));
    }
}

#[test]
fn pressure_hall_forward_route_requires_all_valves_and_return_routes_remain_open() {
    const VALVES: [&str; 3] = [
        "cw_pressure_valve_01_staged",
        "cw_pressure_valve_02_staged",
        "cw_pressure_valve_03_staged",
    ];
    let mut runtime =
        SceneRuntime::new(staged_clockworks_registry(), "cw_pressure_hall", 37).unwrap();
    let hall: Value = serde_json::from_str(CW_PRESSURE_HALL).unwrap();
    let mut completed_events = BTreeSet::new();

    assert_eq!(
        runtime.interact(
            "cw_clockworks_valves_staged",
            "route-gate-aggregate-marker",
            37,
            interaction_position(&hall, "cw_clockworks_valves_staged"),
        ),
        Err(SceneRuntimeError::StaticMarker),
        "a staged aggregate marker cannot emit the route event",
    );
    assert!(!runtime.event_complete("clockworks_valves"));
    assert_eq!(
        runtime.transition_with_progression(
            "cw_to_conveyor_bridge",
            "route-gate-before-valves",
            37,
            transition_position(&hall, "cw_to_conveyor_bridge"),
            &completed_events,
        ),
        Err(SceneRuntimeError::ProgressionLocked),
    );

    for (count, valve_id) in VALVES.iter().take(2).enumerate() {
        runtime
            .interact(
                valve_id,
                &format!("route-gate-valve-{count}"),
                37,
                interaction_position(&hall, valve_id),
            )
            .unwrap();
        assert!(!runtime.event_complete("clockworks_valves"));
        assert_eq!(
            runtime.transition_with_progression(
                "cw_to_conveyor_bridge",
                &format!("route-gate-after-{count}-valves"),
                37,
                transition_position(&hall, "cw_to_conveyor_bridge"),
                &completed_events,
            ),
            Err(SceneRuntimeError::ProgressionLocked),
            "the route remains locked after {} valves",
            count + 1,
        );
    }

    runtime
        .interact(
            VALVES[2],
            "route-gate-valve-2",
            37,
            interaction_position(&hall, VALVES[2]),
        )
        .unwrap();
    assert!(runtime.event_complete("clockworks_valves"));
    completed_events.insert("clockworks_valves".into());
    let forward = runtime
        .transition_with_progression(
            "cw_to_conveyor_bridge",
            "route-gate-after-three-valves",
            37,
            transition_position(&hall, "cw_to_conveyor_bridge"),
            &completed_events,
        )
        .unwrap();
    assert_eq!(forward.target_scene_id, "cw_conveyor_bridge");

    let bridge: Value = serde_json::from_str(CW_CONVEYOR_BRIDGE).unwrap();
    let back_to_hall = runtime
        .transition_with_progression(
            "cw_bridge_to_pressure_hall",
            "route-gate-return-to-hall",
            38,
            transition_position(&bridge, "cw_bridge_to_pressure_hall"),
            &BTreeSet::new(),
        )
        .unwrap();
    assert_eq!(back_to_hall.target_scene_id, "cw_pressure_hall");

    let back_to_entry = runtime
        .transition_with_progression(
            "cw_return_to_entry_foundry",
            "route-gate-return-to-entry",
            39,
            transition_position(&hall, "cw_return_to_entry_foundry"),
            &BTreeSet::new(),
        )
        .unwrap();
    assert_eq!(back_to_entry.target_scene_id, "cw_entry_foundry");
}

#[test]
fn regulator_core_shutdown_exit_requires_clockworks_core_but_arena_return_is_open() {
    const CORE_EVENT: &str = "clockworks_core";
    const SHUTDOWN_TRANSITION: &str = "cw_regulator_core_to_shutdown_exit";
    const ARENA_TRANSITION: &str = "cw_regulator_core_return_to_arena";

    let core: Value = serde_json::from_str(CW_REGULATOR_CORE).unwrap();
    let regulator_spawns: Vec<_> = core["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spawn| {
            spawn["id"] == "cw_prime_regulator"
                || spawn["entityType"] == "enemy.clockworks.prime_regulator"
        })
        .collect();
    assert_eq!(regulator_spawns.len(), 1);
    assert_eq!(regulator_spawns[0]["kind"], "enemy");
    assert_eq!(
        regulator_spawns[0]["entityType"],
        "enemy.clockworks.prime_regulator"
    );
    assert_eq!(
        regulator_spawns[0]["position"],
        serde_json::json!([14.0, 0.0, 8.0])
    );
    let console = core["interactions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "cw_regulator_core_console_staged")
        .unwrap();
    assert_eq!(console["kind"], "terminal");
    assert_eq!(console["event"], CORE_EVENT);
    assert_eq!(console["rangeM"], 2.5);
    assert!(core["interactions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["id"] != "cw_regulator_core_console_staged")
        .all(|item| item["event"].is_null()));

    let registry = staged_clockworks_registry();
    let mut runtime = SceneRuntime::new(registry, "cw_regulator_core", 51).unwrap();
    assert!(!runtime.event_complete(CORE_EVENT));
    // SceneRuntime resolves authored data only. FormalRuntime owns the
    // legitimate defeat/roster guard before it invokes this interaction;
    // its combat-authority regression tests cover rejection while alive.
    let console_events = runtime
        .interact(
            "cw_regulator_core_console_staged",
            "core-authored-console-event",
            51,
            interaction_position(&core, "cw_regulator_core_console_staged"),
        )
        .unwrap();
    assert!(matches!(
        console_events.as_slice(),
        [wuxian_horror_ch1::scene_runtime::SceneEvent::Interaction { event_id: Some(event), .. }]
            if event == CORE_EVENT
    ));
    assert!(runtime.event_complete(CORE_EVENT));
    // The explicit progression API still uses the caller's authoritative
    // completed events rather than trusting this local authored emission.
    assert_eq!(
        runtime.transition_with_progression(
            SHUTDOWN_TRANSITION,
            "core-to-shutdown-locked",
            51,
            transition_position(&core, SHUTDOWN_TRANSITION),
            &BTreeSet::new(),
        ),
        Err(SceneRuntimeError::ProgressionLocked),
    );

    let completed_events = BTreeSet::from([CORE_EVENT.to_owned()]);
    let forward = runtime
        .transition_with_progression(
            SHUTDOWN_TRANSITION,
            "core-to-shutdown-after-core-event",
            51,
            transition_position(&core, SHUTDOWN_TRANSITION),
            &completed_events,
        )
        .unwrap();
    assert_eq!(forward.target_scene_id, "cw_shutdown_exit");

    let mut return_runtime =
        SceneRuntime::new(staged_clockworks_registry(), "cw_regulator_core", 52).unwrap();
    let back = return_runtime
        .transition_with_progression(
            ARENA_TRANSITION,
            "core-return-to-forged-guard-arena",
            52,
            transition_position(&core, ARENA_TRANSITION),
            &BTreeSet::new(),
        )
        .unwrap();
    assert_eq!(back.target_scene_id, "cw_forged_guard_arena");
}

#[test]
fn shutdown_exit_terminal_emits_the_registered_clockworks_shutdown_event() {
    let shutdown: Value = serde_json::from_str(CW_SHUTDOWN_EXIT).unwrap();
    let interaction = shutdown["interactions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "cw_master_shutdown_staged")
        .unwrap();
    assert_eq!(interaction["kind"], "terminal");
    assert_eq!(interaction["event"], "clockworks_shutdown");

    let mut runtime =
        SceneRuntime::new(staged_clockworks_registry(), "cw_shutdown_exit", 71).unwrap();
    let events = runtime
        .interact(
            "cw_master_shutdown_staged",
            "shutdown-terminal-live",
            71,
            interaction_position(&shutdown, "cw_master_shutdown_staged"),
        )
        .unwrap();
    assert!(matches!(
        events.as_slice(),
        [wuxian_horror_ch1::scene_runtime::SceneEvent::Interaction {
            id,
            event_id: Some(event_id),
        }] if id == "cw_master_shutdown_staged" && event_id == "clockworks_shutdown"
    ));
    assert!(runtime.event_complete("clockworks_shutdown"));
    assert!(runtime.object_activated("cw_master_shutdown_staged"));
}

#[test]
fn test_registry_rejects_unknown_world_duplicate_ids_bad_refs_and_missing_events() {
    let unknown_world = GH.replace("grey_hive", "unknown_world");
    let err = WorldRegistry::load(
        [RS, unknown_world.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::UnknownWorld(_)));

    let duplicate_id = GH.replace("gh_lockdown_test", "gh_power_console_test");
    let err = WorldRegistry::load(
        [RS, duplicate_id.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert_eq!(err, SceneRuntimeError::DuplicateObjectId);

    let missing_spawn = RS.replace("gh_spawn", "missing_spawn");
    let err = WorldRegistry::load(
        [missing_spawn.as_str(), GH, MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::UnknownTransitionSpawn(_)));

    let missing_scene = RS.replace(
        "\"toSceneId\": \"gh_test_power\"",
        "\"toSceneId\": \"unknown_scene\"",
    );
    let err = WorldRegistry::load(
        [missing_scene.as_str(), GH, MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::UnknownTransitionTarget(_)));

    let out_of_bounds_spawn = GH.replace(
        "\"position\": [1, 0, 1], \"navNode\": \"gh_nav\"",
        "\"position\": [30, 0, 1], \"navNode\": \"gh_nav\"",
    );
    let err = WorldRegistry::load(
        [RS, out_of_bounds_spawn.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert_eq!(err, SceneRuntimeError::InvalidPosition);

    let missing_event = GH.replace("hive_extraction", "not_a_required_event");
    let err = WorldRegistry::load(
        [RS, missing_event.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::MissingRequiredEvent(_)));
}

#[test]
fn test_registry_checks_asset_entity_nav_bounds_and_canonical_event_ownership() {
    let with_asset = GH.replace(
        "\"sprites\": []",
        "\"sprites\": [{\"id\":\"gh_art\",\"assetId\":\"asset.known\",\"position\":[1,0,1]}]",
    );
    let err = WorldRegistry::load(
        [RS, with_asset.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::UnknownAsset(_)));
    let assets = BTreeSet::from(["asset.known".to_string()]);
    assert!(
        WorldRegistry::load([RS, with_asset.as_str(), MH, CW], &assets, &BTreeSet::new()).is_ok()
    );

    let with_enemy = GH.replace(
        "\"navNode\": \"gh_nav\" }],",
        "\"navNode\": \"gh_nav\" }, {\"id\":\"gh_enemy\",\"kind\":\"enemy\",\"entityType\":\"enemy.fixture\",\"position\":[2,0,2],\"navNode\":\"gh_nav\" }],",
    );
    let err = WorldRegistry::load(
        [RS, with_enemy.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::UnknownEntityType(_)));
    let entities = BTreeSet::from(["enemy.fixture".to_string()]);
    assert!(WorldRegistry::load(
        [RS, with_enemy.as_str(), MH, CW],
        &BTreeSet::new(),
        &entities
    )
    .is_ok());

    let wrong_world_event = GH.replace("hive_power", "mist_signal");
    let err = WorldRegistry::load(
        [RS, wrong_world_event.as_str(), MH, CW],
        &BTreeSet::new(),
        &BTreeSet::new(),
    )
    .unwrap_err();
    assert!(matches!(err, SceneRuntimeError::UnknownEvent(_)));
}

#[test]
fn test_scene_runtime_transitions_events_objectives_checkpoint_and_stale_duplicate_guards() {
    let mut runtime = SceneRuntime::new(registry(), "rs_test_hub", 7).unwrap();
    let pos = Vec3 {
        x_m: 1.0,
        y_m: 0.0,
        z_m: 1.0,
    };
    let checkpoint = runtime.checkpoint("rs_checkpoint", "cp-1", 7, pos).unwrap();
    assert!(format!("{checkpoint:?}").contains("rs_checkpoint"));
    assert_eq!(runtime.checkpoint_id.as_deref(), Some("rs_checkpoint"));
    assert_eq!(
        runtime.checkpoint("rs_checkpoint", "cp-2", 7, pos),
        Err(SceneRuntimeError::AlreadyApplied)
    );
    assert_eq!(
        runtime.transition(
            "rs_to_gh",
            "out-of-range-transition",
            7,
            Vec3 {
                x_m: 10.0,
                y_m: 0.0,
                z_m: 10.0
            }
        ),
        Err(SceneRuntimeError::OutOfRange)
    );

    let entered = runtime.transition("rs_to_gh", "enter-gh", 7, pos).unwrap();
    assert_eq!(entered.target_scene_id, "gh_test_power");
    assert_eq!(entered.next_epoch, 8);
    assert_eq!(
        runtime.transition("gh_to_hub", "old-epoch", 7, entered.position),
        Err(SceneRuntimeError::StaleEpoch)
    );

    runtime
        .interact("gh_power_console_test", "power-1", 8, entered.position)
        .unwrap();
    assert!(runtime.objective_complete("gh_power_objective"));
    assert_eq!(
        runtime.interact("gh_power_console_test", "power-1", 8, entered.position),
        Err(SceneRuntimeError::DuplicateRequest)
    );
    runtime
        .interact("gh_lockdown_test", "lockdown-1", 8, entered.position)
        .unwrap();
    runtime
        .interact("gh_extract_test", "extract-1", 8, entered.position)
        .unwrap();

    let hub = runtime
        .transition("gh_to_hub", "return-hub", 8, entered.position)
        .unwrap();
    assert_eq!(hub.target_scene_id, "rs_test_hub");
    assert_eq!(
        runtime.transition("rs_to_mh", "enter-gh", 9, hub.position),
        Err(SceneRuntimeError::DuplicateRequest)
    );
    let mh = runtime
        .transition("rs_to_mh", "enter-mh", 9, hub.position)
        .unwrap();
    runtime
        .interact("mh_west_test", "west", 10, mh.position)
        .unwrap();
    runtime
        .interact("mh_east_test", "east", 10, mh.position)
        .unwrap();
    runtime
        .interact("mh_signal_test", "signal", 10, mh.position)
        .unwrap();
    let hub = runtime
        .transition("mh_to_hub", "mh-hub", 10, mh.position)
        .unwrap();
    let cw = runtime
        .transition("rs_to_cw", "enter-cw", 11, hub.position)
        .unwrap();
    runtime
        .interact("cw_valves_test", "valves", 12, cw.position)
        .unwrap();
    runtime
        .interact("cw_core_test", "core", 12, cw.position)
        .unwrap();
    runtime
        .trigger("cw_shutdown_trigger_test", "shutdown", 12, cw.position)
        .unwrap();
    assert_eq!(runtime.traversals().len(), 1);
    assert_eq!(runtime.traversals()[0].id, "cw_air_step_test");
    assert_eq!(
        runtime.transition("unknown_transition", "bad-transition", 12, cw.position),
        Err(SceneRuntimeError::UnsafeTransition)
    );
}

#[test]
fn formal_runtime_uses_loaded_scene_content_and_keeps_snapshot_receipt_and_events_in_one_epoch() {
    let runtime = FormalRuntime::new().unwrap();
    let view = runtime
        .load_scene_registry(
            [RS, GH, MH, CW],
            "rs_test_hub",
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .unwrap();
    assert_eq!(view.scene_id, "rs_test_hub");
    let old_epoch = view.world_epoch;
    let checkpoint = runtime
        .activate_scene_checkpoint("rs_checkpoint", "formal-cp", old_epoch)
        .unwrap();
    assert_eq!(checkpoint.checkpoint_id.as_deref(), Some("rs_checkpoint"));
    let entered = acknowledge_ready(
        &runtime,
        runtime.transition_scene("rs_to_gh", "formal-enter-gh", old_epoch).unwrap(),
    );
    assert_eq!(entered.scene_id, "gh_test_power");
    assert_eq!(entered.world_epoch, old_epoch + 1);
    assert!(runtime
        .transition_scene("rs_to_gh", "duplicate-old-transition", old_epoch)
        .is_err());
    assert!(
        !runtime
            .activate_scene_interaction("gh_power_console_test", "stale-interaction", old_epoch)
            .unwrap()
            .applied
    );

    let response = runtime
        .interact("gh_power_console_test", "formal-power")
        .unwrap();
    assert!(response.applied);
    assert_eq!(response.view.scene_id, "gh_test_power");
    assert_eq!(response.receipt.snapshot.scene_id, response.view.scene_id);
    assert_eq!(response.receipt.world_epoch, response.view.world_epoch);
    assert_eq!(
        response
            .view
            .objectives
            .iter()
            .find(|o| o.objective_id == "gh_power_objective")
            .unwrap()
            .state,
        "complete"
    );
    let events = runtime
        .presentation_events_since(response.view.world_epoch, 0)
        .unwrap();
    assert!(events
        .iter()
        .all(|e| e.world_epoch == response.view.world_epoch));
    let reset = acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    assert_eq!(reset.world_id, "grey_hive");
    assert_eq!(reset.scene_id, "grey_hive");
    assert!(reset.objectives.is_empty());
}
