//! The complete admission API is deliberately not selected by native startup
//! yet. These tests prove its content boundary before campaign promotion.
use std::collections::BTreeSet;
use serde_json::{json, Value};
use wuxian_horror_ch1::{
    production_scene_bootstrap as production,
    scene_registry::WorldRegistry,
    scene_runtime::SceneRuntimeError,
};

fn catalogs() -> (BTreeSet<String>, BTreeSet<String>) {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../governance/assets/RUNTIME_ASSET_MANIFEST.json"
    )).unwrap();
    let entities: Value = serde_json::from_str(include_str!(
        "../../content/enemies/entity-types.json"
    )).unwrap();
    (
        manifest["assets"].as_array().unwrap().iter()
            .filter(|row| row["admission"] == "release_approved")
            .map(|row| row["assetId"].as_str().unwrap().to_owned()).collect(),
        entities["entityTypes"].as_array().unwrap().iter()
            .map(|row| row.as_str().unwrap().to_owned()).collect(),
    )
}

fn bundle() -> Vec<Value> {
    production::embedded_scene_json().unwrap().iter()
        .map(|raw| serde_json::from_str(raw).unwrap()).collect()
}

fn scene<'a>(bundle: &'a mut [Value], id: &str) -> &'a mut Value {
    bundle.iter_mut().find(|row| row["sceneId"] == id).unwrap()
}

fn load(bundle: &[Value]) -> Result<WorldRegistry, SceneRuntimeError> {
    let (assets, entities) = catalogs();
    WorldRegistry::load_complete_clockworks_production(
        bundle.iter().map(Value::to_string), &assets, &entities,
    )
}

#[test]
fn complete_admission_accepts_exact_locked_thirty_scene_bundle() {
    let registry = load(&bundle()).unwrap();
    assert!(registry.is_complete_clockworks_production());
    assert!(!registry.is_staged_clockworks_production());
    assert_eq!(registry.scenes().count(), 30);
    assert_eq!(registry.required_events("clockworks").unwrap(), &BTreeSet::from([
        "clockworks_valves".into(), "clockworks_core".into(), "clockworks_shutdown".into(),
    ]));
}

#[test]
fn complete_admission_rejects_missing_extra_or_reassigned_scene() {
    for invalid in ["missing", "extra", "wrong-world", "renamed"] {
        let mut documents = bundle();
        match invalid {
            "missing" => { documents.retain(|row| row["sceneId"] != "cw_shutdown_exit"); }
            "extra" => {
                let mut extra = scene(&mut documents, "cw_entry_foundry").clone();
                extra["sceneId"] = json!("cw_extra_room");
                documents.push(extra);
            }
            "wrong-world" => scene(&mut documents, "cw_entry_foundry")["worldId"] = json!("grey_hive"),
            "renamed" => scene(&mut documents, "cw_entry_foundry")["sceneId"] = json!("cw_substitute_room"),
            _ => unreachable!(),
        }
        assert!(load(&documents).is_err(), "{invalid}");
    }
}

#[test]
fn complete_admission_requires_all_three_exact_event_emitters() {
    for (scene_id, event) in [
        ("cw_pressure_hall", "clockworks_valves"),
        ("cw_regulator_core", "clockworks_core"),
        ("cw_shutdown_exit", "clockworks_shutdown"),
    ] {
        let mut documents = bundle();
        let target = scene(&mut documents, scene_id);
        if event == "clockworks_valves" {
            target["interactionAggregates"] = json!([]);
        } else {
            for item in target["interactions"].as_array_mut().unwrap() {
                if item["event"] == event { item["event"] = Value::Null; }
            }
        }
        assert!(load(&documents).is_err(), "missing {event}");
    }
}

#[test]
fn complete_admission_rejects_duplicate_moved_or_wrong_kind_terminals() {
    for invalid in ["duplicate", "moved", "renamed", "wrong-kind", "trigger"] {
        let mut documents = bundle();
        let source = scene(&mut documents, "cw_shutdown_exit");
        let index = source["interactions"].as_array().unwrap().iter()
            .position(|row| row["event"] == "clockworks_shutdown").unwrap();
        let original = source["interactions"][index].clone();
        match invalid {
            "duplicate" => {
                let mut duplicate = original;
                duplicate["id"] = json!("cw_second_shutdown_terminal");
                source["interactions"].as_array_mut().unwrap().push(duplicate);
            }
            "moved" => {
                source["interactions"][index]["event"] = Value::Null;
                scene(&mut documents, "cw_furnace_heart")["interactions"].as_array_mut().unwrap().push(original);
            }
            "renamed" => source["interactions"][index]["id"] = json!("cw_fake_shutdown"),
            "wrong-kind" => source["interactions"][index]["kind"] = json!("valve_control"),
            "trigger" => {
                source["interactions"][index]["event"] = Value::Null;
                source["triggers"] = json!([{
                    "id": "cw_fake_shutdown_trigger", "event": "clockworks_shutdown",
                    "polygon": [[7.0,5.0],[9.0,5.0],[9.0,7.0],[7.0,7.0]]
                }]);
            }
            _ => unreachable!(),
        }
        assert!(load(&documents).is_err(), "{invalid}");
    }
}

#[test]
fn complete_admission_rejects_terminal_substituted_for_three_valve_aggregate() {
    let mut documents = bundle();
    let pressure = scene(&mut documents, "cw_pressure_hall");
    pressure["interactionAggregates"] = json!([]);
    let marker = pressure["interactions"].as_array_mut().unwrap().iter_mut()
        .find(|row| row["id"] == "cw_clockworks_valves_staged").unwrap();
    marker["kind"] = json!("terminal");
    marker["event"] = json!("clockworks_valves");
    // This candidate passes the old generic campaign validator. The strict
    // validator must reject its emitter type, not merely generic scene shape.
    let (assets, entities) = catalogs();
    WorldRegistry::load(documents.iter().map(Value::to_string), &assets, &entities).unwrap();
    assert_eq!(load(&documents).unwrap_err(), SceneRuntimeError::InvalidNativeSceneSet);
}

#[test]
fn complete_admission_requires_three_canonical_valve_controls() {
    for invalid in ["two-members", "substitute-member", "wrong-kind", "wrong-marker-kind", "direct-event", "duplicate-aggregate"] {
        let mut documents = bundle();
        let source = scene(&mut documents, "cw_pressure_hall");
        match invalid {
            "two-members" => { source["interactionAggregates"][0]["memberIds"].as_array_mut().unwrap().pop(); }
            "substitute-member" => {
                source["interactionAggregates"][0]["memberIds"][2] = json!("cw_substitute_valve");
                source["interactions"].as_array_mut().unwrap().iter_mut()
                    .find(|row| row["id"] == "cw_pressure_valve_03_staged").unwrap()["id"] = json!("cw_substitute_valve");
            }
            "wrong-kind" => source["interactions"][0]["kind"] = json!("terminal"),
            "wrong-marker-kind" => source["interactions"].as_array_mut().unwrap().iter_mut()
                .find(|row| row["id"] == "cw_clockworks_valves_staged").unwrap()["kind"] = json!("substitute_marker"),
            "direct-event" => source["interactions"][0]["event"] = json!("clockworks_valves"),
            "duplicate-aggregate" => {
                let mut extra = source["interactionAggregates"][0].clone();
                extra["markerId"] = json!("cw_extra_valve_aggregate");
                source["interactionAggregates"].as_array_mut().unwrap().push(extra);
            }
            _ => unreachable!(),
        }
        assert!(load(&documents).is_err(), "{invalid}");
    }
}

#[test]
fn complete_admission_preserves_return_station_and_world_route_boundaries() {
    for invalid in ["cross-world", "station-route", "station-event"] {
        let mut documents = bundle();
        let (target_scene, target_spawn) = {
            let target = scene(&mut documents, "gh_entry_maintenance");
            (target["sceneId"].clone(), target["spawns"].as_array().unwrap().iter()
                .find(|row| row["kind"] == "player").unwrap()["id"].clone())
        };
        match invalid {
            "cross-world" => {
                let transition = &mut scene(&mut documents, "cw_entry_foundry")["transitions"][0];
                transition["toSceneId"] = target_scene;
                transition["spawnId"] = target_spawn;
            }
            "station-route" => {
                let mut transition = scene(&mut documents, "cw_entry_foundry")["transitions"][0].clone();
                transition["id"] = json!("rs_unauthorized_transition");
                transition["toSceneId"] = target_scene;
                transition["spawnId"] = target_spawn;
                scene(&mut documents, "rs_core_room")["transitions"] = json!([transition]);
            }
            "station-event" => scene(&mut documents, "rs_core_room")["interactions"][0]["event"] = json!("clockworks_shutdown"),
            _ => unreachable!(),
        }
        assert!(load(&documents).is_err(), "{invalid}");
    }
}

#[test]
fn staged_loader_remains_separate() {
    let (assets, entities) = catalogs();
    let registry = WorldRegistry::load_staged_clockworks_production(
        production::embedded_scene_json().unwrap().iter().copied(), &assets, &entities,
    ).unwrap();
    assert!(registry.is_staged_clockworks_production());
    assert!(!registry.is_complete_clockworks_production());
}
