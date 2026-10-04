use super::*;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);

fn fixture_registry(
    mutate: impl FnOnce(&mut Value),
) -> Result<WorldRegistry, crate::scene_runtime::SceneRuntimeError> {
    // Synthetic engine fixture uses the real scene validation/runtime. This is not
    // authored campaign acceptance; production content receives separate tests.
    let mut values: Vec<Value> = crate::production_scene_bootstrap::embedded_scene_json()
        .unwrap()
        .iter()
        .map(|raw| serde_json::from_str(raw).unwrap())
        .collect();
    let scene = values
        .iter_mut()
        .find(|v| v["sceneId"] == "gh_deep_decon")
        .unwrap();
    scene["hazards"][0]["kind"] = json!("heat_zone");
    scene["hazards"][0]["environment"] = json!({"tag":"heat","warningMs":600,"activeMs":2000,"recoveryMs":1000,"damage":8,"damageIntervalMs":500});
    scene["hazards"][1]["environment"] = json!({"tag":"toxin","warningMs":600,"activeMs":2000,"recoveryMs":1000,"damage":6,"damageIntervalMs":500});
    scene["interactions"][0]["kind"] = json!("environment_control");
    scene["interactions"][0]["rangeM"] = json!(2.5);
    scene["interactions"][0]["cooldownMs"] = json!(8000);
    scene["interactions"][0]["environmentControl"] = json!({"targetHazardIds":["gh_decon_steam_jet_zone"],"suppressionMs":6000,"exposureReductionUnits":0});
    mutate(scene);
    fn collect(value: &Value, property: &str, out: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    if k == property {
                        if let Some(s) = v.as_str() {
                            out.insert(s.into());
                        }
                    }
                    collect(v, property, out);
                }
            }
            Value::Array(array) => {
                for v in array {
                    collect(v, property, out);
                }
            }
            _ => {}
        }
    }
    let mut assets = BTreeSet::new();
    let mut entities = BTreeSet::new();
    for v in &values {
        collect(v, "assetId", &mut assets);
        collect(v, "entityType", &mut entities);
    }
    let raws: Vec<_> = values.iter().map(Value::to_string).collect();
    WorldRegistry::load_staged_clockworks_production(raws, &assets, &entities)
}

fn fixture() -> (FormalRuntime, std::path::PathBuf) {
    let registry = fixture_registry(|_| {}).unwrap();
    let root = std::env::temp_dir().join(format!(
        "env-hazard-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let runtime = FormalRuntime::new_with_save_dir(root.clone()).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime
        .owner_handle
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .join()
        .unwrap();
    runtime.state.lock().unwrap().world.server_time_ms = 0;
    runtime
        .install_scene_registry(registry, "gh_deep_decon")
        .unwrap();
    (runtime, root)
}
fn step(runtime: &FormalRuntime, now: u64, position: [f32; 3]) {
    let mut state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    state.world.player.position_m = vec3_from_array(position);
    state.world.server_time_ms = now;
    advance(&mut state, scene.as_ref().unwrap()).unwrap();
}
#[test]
fn runtime_damage_uses_regions_resolved_equipment_and_removal() {
    let (runtime, _) = fixture();
    step(&runtime, 599, [8.0, 0.0, 6.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 100);
    runtime
        .grant_trusted_build_item("heat_resistance_lining")
        .unwrap();
    runtime.equip_build_item("heat_resistance_lining").unwrap();
    step(&runtime, 600, [7.0, 0.0, 4.0]); // Authored polygon edge belongs to the zone.
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 96);
    step(&runtime, 1100, [6.99, 0.0, 4.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 96);
    runtime.unequip_build_slot("lining").unwrap();
    step(&runtime, 1101, [8.0, 0.0, 6.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 88);
    // Heat resistance does not grant toxin resistance to the other region.
    runtime.equip_build_item("heat_resistance_lining").unwrap();
    step(&runtime, 1601, [16.0, 0.0, 11.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 82);
}
#[test]
fn pause_epoch_and_revisit_do_not_tick_or_apply_offscreen_damage() {
    let (runtime, _) = fixture();
    step(&runtime, 600, [8.0, 0.0, 6.0]);
    let before = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    runtime.pause().unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        // The owner loop already guards the world clock; the new consumer must
        // independently remain inert when called while paused.
        advance(&mut state, scene.as_ref().unwrap()).unwrap();
        assert_eq!(state.world.server_time_ms, 600);
        assert_eq!(state.world_persistent_v1.environment, before);
        let mut stale = scene.as_ref().unwrap().clone();
        stale.world_epoch += 1;
        state.paused = false;
        advance(&mut state, &stale).unwrap();
        assert_eq!(state.world_persistent_v1.environment, before);
        state.world.server_time_ms = 80_000;
        let hp = state.world.player_hp;
        prepare_scene(&mut state, scene.as_ref().unwrap().current_scene()).unwrap();
        assert_eq!(state.world.player_hp, hp);
    }
}
#[test]
fn controls_are_transactional_ranged_replay_safe_and_do_not_complete_progression() {
    let (runtime, _) = fixture();
    let id = "gh_decon_valve_01_static_marker";
    let epoch = runtime.snapshot().unwrap().world_epoch;
    let before = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    assert!(
        !runtime
            .activate_environment_control(id, "far", epoch)
            .unwrap()
            .applied
    );
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .environment,
        before
    );
    step(&runtime, 600, [7.0, 0.0, 3.0]);
    let route = runtime.state.lock().unwrap().route.clone();
    let receipt = runtime
        .activate_environment_control(id, "cool", epoch)
        .unwrap();
    assert!(receipt.applied);
    assert_eq!(
        receipt.view.hazards[0].environment.as_ref().unwrap().phase,
        EnvironmentPhase::Suppressed
    );
    let saved = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    for (request, e) in [
        ("cool", epoch),
        ("early", epoch),
        ("wrong-epoch", epoch + 1),
    ] {
        assert!(
            !runtime
                .activate_environment_control(id, request, e)
                .unwrap()
                .applied
        );
        assert_eq!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .environment,
            saved
        );
    }
    assert_eq!(runtime.state.lock().unwrap().route, route);
    assert_eq!(
        runtime.snapshot().unwrap().hazards[1]
            .environment
            .as_ref()
            .unwrap()
            .phase,
        EnvironmentPhase::Active
    );
    step(&runtime, 8599, [7.0, 0.0, 3.0]);
    assert!(
        !runtime
            .activate_environment_control(id, "one-ms-early", epoch)
            .unwrap()
            .applied
    );
    step(&runtime, 8600, [7.0, 0.0, 3.0]);
    assert!(
        runtime
            .activate_environment_control(id, "reuse", epoch)
            .unwrap()
            .applied
    );
}
#[test]
fn saved_hazard_ids_must_come_from_canonical_authored_content() {
    let state = model::EnvironmentPersistentState {
        hazards: BTreeMap::from([("forged/world/zone".into(), EnvironmentHazardState::new(0))]),
        ..Default::default()
    };
    assert_eq!(
        validate_saved(&state, 0),
        Err("E_ENV_SAVED_HAZARD_UNKNOWN".into())
    );
    validate_saved(&Default::default(), 0).unwrap();
}

#[test]
fn unknown_environment_identity_cannot_be_written_as_a_canonical_save() {
    let (runtime, root) = fixture();
    runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .hazards
        .insert("forged/world/zone".into(), EnvironmentHazardState::new(0));
    runtime.pause().unwrap();
    let error = runtime.save().unwrap_err();
    assert_eq!(error, "E_ENV_SAVED_HAZARD_UNKNOWN");
    assert!(!root.join(crate::save_v6::SAVE_V6_FILE_NAME).exists());
}

#[test]
fn failed_prepare_does_not_publish_partial_environment_state() {
    let (runtime, _) = fixture();
    let mut state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    let definition = scene.as_ref().unwrap().current_scene();
    let invalid_key = key(definition, "gh_decon_steam_jet_zone");
    state
        .world_persistent_v1
        .environment
        .hazards
        .get_mut(&invalid_key)
        .unwrap()
        .last_advanced_at_ms = 1;
    let before = state.world_persistent_v1.environment.clone();
    assert_eq!(
        prepare_scene(&mut state, definition),
        Err("E_ENV_HAZARD_STATE".into())
    );
    assert_eq!(state.world_persistent_v1.environment, before);
}

#[test]
fn scene_schema_rejects_ambiguous_damage_unknown_targets_and_progression_controls() {
    let bad: Vec<Box<dyn Fn(&mut Value)>> = vec![
        Box::new(|s| s["hazards"][0]["damage"] = json!(1)),
        Box::new(|s| s["hazards"][0]["periodMs"] = json!(1000)),
        Box::new(|s| s["hazards"][0]["environment"]["tag"] = json!("toxin")),
        Box::new(|s| s["hazards"][0]["environment"]["activeMs"] = json!(0)),
        Box::new(|s| s["hazards"][0]["environment"]["untrusted"] = json!(true)),
        Box::new(|s| {
            s["interactions"][0]["environmentControl"]["targetHazardIds"] = json!(["invented_zone"])
        }),
        Box::new(|s| {
            s["interactions"][0]["environmentControl"]["targetHazardIds"] =
                json!(["gh_decon_steam_jet_zone", "gh_decon_steam_jet_zone"])
        }),
        Box::new(|s| {
            s["interactions"][0]["environmentControl"]["grant"] = json!("hive_extraction")
        }),
        Box::new(|s| s["interactions"][0]["event"] = json!("hive_extraction")),
        Box::new(|s| s["interactions"][0]["cooldownMs"] = json!(5999)),
        Box::new(|s| s["interactions"][0]["kind"] = json!("valve")),
        Box::new(|s| {
            s["interactions"][0]
                .as_object_mut()
                .unwrap()
                .remove("environmentControl");
        }),
    ];
    fixture_registry(|_| {}).unwrap();
    for (index, mutate) in bad.into_iter().enumerate() {
        assert!(fixture_registry(mutate).is_err(), "mutation {index}");
    }
}

fn stopped_authored(root: std::path::PathBuf, scene_id: &str) -> FormalRuntime {
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime
        .owner_handle
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .join()
        .unwrap();
    runtime.state.lock().unwrap().world.server_time_ms = 0;
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    let registry = runtime
        .scene_registry
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    // Test-only selection of an embedded staged scene is not a route unlock.
    runtime.install_scene_registry(registry, scene_id).unwrap();
    if scene_id.starts_with("cw_") {
        // Match this explicitly staged unit fixture's world label without adding
        // any completion event, opening a native gate or bypassing Continue guards.
        runtime.state.lock().unwrap().route.current_world_id = "clockworks".into();
    }
    runtime
}
fn authored(scene_id: &str) -> (FormalRuntime, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "authored-environment-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    (stopped_authored(root.clone(), scene_id), root)
}
#[test]
fn authored_heat_resistance_uses_live_equipment_bloodline_stack_and_removal() {
    for (scene_id, threshold_time, position, full, half, quarter) in [
        ("cw_boiler_chamber", 3000, [14.0, 0.0, 4.0], 8, 4, 2),
        ("cw_furnace_heart", 3200, [11.0, 0.0, 4.0], 10, 5, 3),
    ] {
        let (runtime, _) = authored(scene_id);
        let route = runtime.state.lock().unwrap().route.clone();
        let pulse = |now, damage| {
            let hp = runtime.snapshot().unwrap().player.current_hp;
            step(&runtime, now, position);
            assert_eq!(
                hp - runtime.snapshot().unwrap().player.current_hp,
                damage,
                "{scene_id} at {now}"
            );
        };
        pulse(threshold_time - 1, 0);
        pulse(threshold_time, full);
        runtime
            .grant_trusted_build_item("heat_resistance_lining")
            .unwrap();
        runtime.equip_build_item("heat_resistance_lining").unwrap();
        pulse(threshold_time + 1000, half);
        runtime
            .grant_trusted_build_bloodline("internal_thermal_adaptation")
            .unwrap();
        pulse(threshold_time + 2000, quarter);
        runtime.unequip_build_slot("lining").unwrap();
        pulse(threshold_time + 3000, half);
        runtime
            .remove_trusted_build_bloodline("internal_thermal_adaptation")
            .unwrap();
        pulse(threshold_time + 4000, full);
        assert_eq!(runtime.state.lock().unwrap().route, route);
    }
}
#[test]
fn authored_steam_mist_cycles_regions_and_safe_ground_lane_are_real() {
    let (runtime, _) = authored("gh_deep_decon");
    let route = runtime.state.lock().unwrap().route.clone();
    step(&runtime, 899, [8.0, 0.0, 5.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 100);
    step(&runtime, 900, [8.0, 0.0, 5.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 92);
    step(&runtime, 1599, [8.0, 0.0, 5.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 92);
    step(&runtime, 1600, [8.0, 0.0, 5.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 92);
    assert_eq!(
        runtime.snapshot().unwrap().hazards[0]
            .environment
            .as_ref()
            .unwrap()
            .phase,
        EnvironmentPhase::Recovery
    );
    step(&runtime, 2000, [16.0, 0.0, 11.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 86);
    step(&runtime, 3000, [16.0, 0.0, 8.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 86);
    step(&runtime, 4000, [8.0, 0.0, 5.0]);
    assert_eq!(
        runtime.snapshot().unwrap().hazards[0]
            .environment
            .as_ref()
            .unwrap()
            .phase,
        EnvironmentPhase::Warning
    );
    step(&runtime, 4900, [8.0, 0.0, 8.0]);
    assert_eq!(runtime.snapshot().unwrap().player.current_hp, 86);
    assert_eq!(runtime.state.lock().unwrap().route, route);
}
#[test]
fn authored_coolant_disk_restore_preserves_state_through_native_continue() {
    for (scene_id, control_id, position, hot, time) in [
        (
            "cw_boiler_chamber",
            "cw_coolant_valve_staged",
            [18.0, 0.0, 8.0],
            [14.0, 0.0, 4.0],
            4000,
        ),
        (
            "cw_furnace_heart",
            "cw_cooling_switch_staged",
            [16.0, 0.0, 7.0],
            [11.0, 0.0, 4.0],
            4200,
        ),
        (
            "gh_deep_decon",
            "gh_decon_valve_02_static_marker",
            [17.0, 0.0, 13.0],
            [16.0, 0.0, 11.0],
            2000,
        ),
    ] {
        let (runtime, root) = authored(scene_id);
        let route = runtime.state.lock().unwrap().route.clone();
        step(&runtime, time, hot);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime.state.lock().unwrap().world.player.position_m = vec3_from_array(position);
        assert!(
            runtime
                .activate_environment_control(control_id, "cool-authored", epoch)
                .unwrap()
                .applied
        );
        let expected = runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .environment
            .clone();
        assert!(expected
            .hazards
            .values()
            .any(|h| h.suppressed_until_ms > time));
        runtime.pause().unwrap();
        runtime.save().unwrap();
        let bytes = std::fs::read(root.join(crate::save_v6::SAVE_V6_FILE_NAME)).unwrap();
        let saved = crate::save_v6::read_save(&root).unwrap();
        assert_eq!(saved.effect_sources_version, 2);
        assert_eq!(saved.world_persistent_v1.environment, expected);
        assert_eq!(
            std::fs::read(root.join(crate::save_v6::SAVE_V6_FILE_NAME)).unwrap(),
            bytes
        );
        assert_eq!(runtime.state.lock().unwrap().route, route);
        drop(runtime);
        let restored = stopped_authored(root.clone(), "rs_core_room");
        let from_disk = crate::save_v6::read_save(&root).unwrap().restore_state().unwrap();
        assert_eq!(from_disk.world_persistent_v1.environment, expected);
        assert_eq!(from_disk.route, route);
        let view = restored.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&restored, view)).unwrap();
        assert_eq!(view.scene_id, scene_id);
        assert_eq!(
            restored
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .environment,
            expected
        );
        assert_eq!(restored.state.lock().unwrap().route, route);
        let before = restored.snapshot().unwrap().player.current_hp;
        step(&restored, time + 1, hot);
        assert_eq!(restored.snapshot().unwrap().player.current_hp, before);
        drop(restored);
        let _ = std::fs::remove_dir_all(root);
    }
}
#[test]
fn legacy_read_keeps_exact_bytes_and_current_profile_two_rejects_forged_environment() {
    let root = std::env::temp_dir().join(format!(
        "env-save-profile-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(crate::save_v6::SAVE_V6_FILE_NAME);
    let legacy =
        include_bytes!("../../../tests/fixtures/capability-save-profiles/legacy-empty.json");
    std::fs::write(&path, legacy).unwrap();
    let normalized = crate::save_v6::read_save(&root).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), legacy);
    assert_eq!(normalized.effect_sources_version, 2);
    assert!(normalized.world_persistent_v1.environment.is_default());
    let mut current = normalized.clone();
    current.world_persistent_v1.environment.hazards.insert(
        "clockworks/cw_boiler_chamber/cw_heat_accumulation_staged".into(),
        EnvironmentHazardState::new(current.save.server_time_ms),
    );
    current.validate().unwrap();
    let valid = serde_json::to_vec(&current).unwrap();
    std::fs::write(&path, &valid).unwrap();
    assert_eq!(crate::save_v6::read_save(&root).unwrap(), current);
    assert_eq!(std::fs::read(&path).unwrap(), valid);
    for mutation in 0..4 {
        let mut bad = serde_json::to_value(&current).unwrap();
        let env = &mut bad["worldPersistentV1"]["environment"];
        match mutation {
            0 => {
                env["hazards"]["forged/world/zone"] =
                    serde_json::to_value(EnvironmentHazardState::new(current.save.server_time_ms))
                        .unwrap();
            }
            1 => {
                env["controlCooldownUntilMs"]["clockworks/cw_boiler_chamber/invented_valve"] =
                    json!(0);
            }
            2 => {
                env["hazards"]["clockworks/cw_boiler_chamber/cw_heat_accumulation_staged"]
                    ["exposureMilliunits"] = json!(100001);
            }
            _ => {
                env["hazards"]["clockworks/cw_boiler_chamber/cw_heat_accumulation_staged"]
                    ["immune"] = json!(true);
            }
        }
        let bytes = serde_json::to_vec(&bad).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(crate::save_v6::read_save(&root).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn authored_furnace_second_control_preserves_the_longer_safe_window() {
    let (runtime, _) = authored("cw_furnace_heart");
    step(&runtime, 5000, [11.0, 0.0, 4.0]);
    let epoch = runtime.snapshot().unwrap().world_epoch;
    runtime.state.lock().unwrap().world.player.position_m = vec3_from_array([16.0, 0.0, 7.0]);
    assert!(
        runtime
            .activate_environment_control("cw_cooling_switch_staged", "long", epoch)
            .unwrap()
            .applied
    );
    let before = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    step(&runtime, 5100, [18.0, 0.0, 10.0]);
    assert!(
        runtime
            .activate_environment_control("cw_coolant_valve_staged", "short", epoch)
            .unwrap()
            .applied
    );
    let after = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    let key = "clockworks/cw_furnace_heart/cw_heat_accumulation_staged";
    assert_eq!(before.hazards[key].suppressed_until_ms, 14000);
    assert_eq!(after.hazards[key].suppressed_until_ms, 14000);
    assert!(after.hazards[key].exposure_milliunits < before.hazards[key].exposure_milliunits);
    assert_eq!(after.control_cooldown_until_ms.len(), 2);
}

#[test]
fn real_owner_step_drives_authored_heat_clock_and_damage() {
    let (runtime, _) = authored("cw_boiler_chamber");
    let mut state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    // Isolate the owner's authored heat path from the now-present Hound combat.
    // This is a hazard unit fixture; the genuine campaign keeps every actor.
    state.world.generic_actors.clear();
    state.world.player.position_m = vec3_from_array([14.0, 0.0, 4.0]);
    state.latest_input_received_at = Instant::now();
    let before_route = state.route.clone();
    let tick_ms = (state.stepper.config.dt_s() * 1000.0).round() as u64;
    while state.world.server_time_ms + tick_ms < 3000 {
        advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
        assert_eq!(state.world.player_hp, 100);
    }
    advance_owner_step_with_scene(&mut state, scene.as_ref()).unwrap();
    assert!(state.world.server_time_ms >= 3000);
    assert_eq!(state.world.player_hp, 92);
    let hazard = &state.world_persistent_v1.environment.hazards
        ["clockworks/cw_boiler_chamber/cw_heat_accumulation_staged"];
    assert_eq!(hazard.last_advanced_at_ms, state.world.server_time_ms);
    assert!(hazard.exposure_milliunits >= 50000);
    assert_eq!(state.route, before_route);
}

#[test]
fn actual_paused_owner_loop_keeps_environment_clock_damage_and_exposure_frozen() {
    let (runtime, _) = authored("cw_boiler_chamber");
    step(&runtime, 3000, [14.0, 0.0, 4.0]);
    runtime.pause().unwrap();
    let before = runtime.snapshot().unwrap();
    let persistent = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    let stop = Arc::new(AtomicBool::new(false));
    let owner_state = Arc::clone(&runtime.state);
    let owner_scene = Arc::clone(&runtime.scene_runtime);
    let owner_stop = Arc::clone(&stop);
    let (ready, started) = std::sync::mpsc::sync_channel(0);
    let handle = std::thread::spawn(move || {
        ready.send(()).unwrap();
        simulation_owner_loop(owner_state, owner_scene, owner_stop);
    });
    started.recv().unwrap();
    // Exercise the real scheduler's paused branch across several wake opportunities.
    // This is an inertness check, not a wall-time/FPS acceptance threshold.
    std::thread::sleep(Duration::from_millis(60));
    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();
    let after = runtime.snapshot().unwrap();
    assert_eq!(after.server_time_ms, before.server_time_ms);
    assert_eq!(after.server_tick, before.server_tick);
    assert_eq!(after.player.current_hp, before.player.current_hp);
    assert_eq!(
        serde_json::to_value(&after.hazards).unwrap(),
        serde_json::to_value(&before.hazards).unwrap()
    );
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .environment,
        persistent
    );
}

#[test]
fn environment_transport_rejects_stale_rebound_legacy_and_payload_replay_with_atomic_receipts() {
    let (runtime, _) = authored("cw_boiler_chamber");
    let original_epoch = runtime.snapshot().unwrap().world_epoch;
    runtime.state.lock().unwrap().world.player.position_m = vec3_from_array([23.0, 0.0, 8.0]);
    runtime
        .transition_scene("cw_boiler_to_gear_shaft", "next-one", original_epoch).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
        .unwrap();
    let middle = runtime.snapshot().unwrap().world_epoch;
    // Transport fixture only: the current Gear Shaft exit is on the y=2 deck.
    { let mut state=runtime.state.lock().unwrap();
      state.world.player.position_m = vec3_from_array([23.0, 2.0, 8.0]);
      assert!(state.kcc.valid_saved_support_contact(&state.world.player,state.world.server_time_ms)); }
    runtime
        .transition_scene("cw_gear_to_furnace_heart", "next-two", middle).map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view))
        .unwrap();
    let epoch = runtime.snapshot().unwrap().world_epoch;
    assert!(epoch > original_epoch);
    runtime.state.lock().unwrap().world.player.position_m = vec3_from_array([18.0, 0.0, 10.0]);
    let before = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    let stale = crate::scene_route_commands::environment_control(
        &runtime,
        "cw_coolant_valve_staged",
        "delayed-boiler",
        original_epoch,
    )
    .unwrap();
    assert!(!stale.applied);
    assert_eq!(stale.command_id, "delayed-boiler");
    assert_eq!(stale.world_epoch, epoch);
    assert_eq!(
        stale.error_code.as_deref(),
        Some("E_SCENE_RUNTIME_StaleEpoch")
    );
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .environment,
        before
    );
    let legacy = runtime
        .interact("cw_coolant_valve_staged", "legacy-no-epoch")
        .unwrap_err();
    assert_eq!(legacy, "E_INTERACTION_SOURCE_EPOCH_REQUIRED");
    let wrong = crate::scene_route_commands::environment_control(
        &runtime,
        "cw_cy_heat_01_static_dialogue_marker",
        "wrong-kind",
        epoch,
    )
    .unwrap();
    assert!(!wrong.applied);
    assert_eq!(wrong.command_id, "wrong-kind");
    assert_eq!(
        wrong.error_code.as_deref(),
        Some("E_ENV_CONTROL_KIND_REQUIRED")
    );
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .environment,
        before
    );
    let applied = crate::scene_route_commands::environment_control(
        &runtime,
        "cw_coolant_valve_staged",
        "apply-once",
        epoch,
    )
    .unwrap();
    assert!(applied.applied);
    assert_eq!(applied.command_id, "apply-once");
    let wire = serde_json::to_value(&applied).unwrap();
    assert_eq!(wire["worldEpoch"], wire["snapshot"]["worldEpoch"]);
    assert_eq!(
        wire["authorityRevision"],
        wire["snapshot"]["authorityRevision"]
    );
    let saved = runtime
        .state
        .lock()
        .unwrap()
        .world_persistent_v1
        .environment
        .clone();
    for target in ["cw_coolant_valve_staged", "cw_cooling_switch_staged"] {
        runtime.state.lock().unwrap().world.player.position_m =
            vec3_from_array(if target == "cw_cooling_switch_staged" {
                [16.0, 0.0, 7.0]
            } else {
                [18.0, 0.0, 10.0]
            });
        let rejected =
            crate::scene_route_commands::environment_control(&runtime, target, "apply-once", epoch)
                .unwrap();
        assert!(!rejected.applied);
        assert_eq!(rejected.command_id, "apply-once");
        assert_eq!(
            runtime
                .state
                .lock()
                .unwrap()
                .world_persistent_v1
                .environment,
            saved
        );
    }
}
