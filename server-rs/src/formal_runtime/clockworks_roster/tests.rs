//! These tests arrange historical save records. They prove compatibility and
//! atomicity, not earned combat or a genuine campaign route.
use super::*;
use crate::formal_runtime::entry_test_support::acknowledge_ready;
use crate::save_v6::SaveV6;

pub(crate) fn fixture(scene_id: &str) -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("cw-roster-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.pause().unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    runtime.install_scene_registry(registry, scene_id).unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        state.route.current_world_id = "clockworks".into();
        state.route.progress.iter_mut().find(|p| p.world_id == "clockworks").unwrap().visit_id = 1;
    }
    runtime
}

fn capture(runtime: &FormalRuntime) -> SaveV6 {
    SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap()
}
fn actors(runtime: &FormalRuntime) -> Vec<crate::world_v3::ActorRuntime> {
    runtime.state.lock().unwrap().world.generic_actors.clone()
}
const EXPANDED: [&str; 5] = ["cw_pressure_hall", "cw_conveyor_bridge", "cw_boiler_chamber", "cw_gear_shaft", "cw_furnace_heart"];
const ALL: [&str; 9] = ["cw_entry_foundry", "cw_pressure_hall", "cw_conveyor_bridge", "cw_boiler_chamber", "cw_gear_shaft", "cw_furnace_heart", "cw_forged_guard_arena", "cw_regulator_core", "cw_shutdown_exit"];

#[test]
fn every_fresh_native_clockworks_scene_marks_current_exact_roster() {
    for scene in ALL {
        let runtime = fixture(scene);
        let save = capture(&runtime);
        assert_eq!(save.world_persistent_v1.clockworks.actor_roster_version, 1, "{scene}");
        assert_eq!(actors(&runtime), spawn_generic_actors(&current_definition(scene).unwrap()).unwrap());
        save.validate().unwrap();
    }
}

#[test]
fn historical_guard_states_upgrade_only_added_rows_without_rewriting_file_or_slot() {
    for scene in EXPANDED {
        let runtime = fixture(scene);
        let mut old = capture(&runtime);
        old.world_persistent_v1.clockworks.actor_roster_version = 0;
        old.save.generic_actors.truncate(old_guard_positions(scene).len());
        for guard in &mut old.save.generic_actors {
            guard.hp -= 5;
            guard.state = crate::world_v3::ActorAiState::Chasing;
            guard.position_m.x_m += 0.25;
            assert!(guard.validate());
        }
        crate::save_v6::write_save(&runtime.save_root, &old).unwrap();
        crate::save_slots::create_slot_v6(&runtime.save_root, "historical", "Historical", &old).unwrap();
        let path = crate::save_v6::save_path(&runtime.save_root);
        let slot_path = runtime.save_root.join("slots/historical/slot-v6.json");
        let bytes = std::fs::read(&path).unwrap();
        let slot_bytes = std::fs::read(&slot_path).unwrap();
        let mut previous_epoch = runtime.snapshot().unwrap().world_epoch;
        for slot in [false, false, true] {
            let view = if slot { runtime.continue_slot("historical").unwrap() } else { runtime.continue_saved().unwrap() };
            assert!(view.entry_token.is_some());
            assert!(runtime.state.lock().unwrap().paused);
            assert!(view.world_epoch > previous_epoch); previous_epoch = view.world_epoch;
            let migrated = actors(&runtime);
            assert_eq!(&migrated[..old.save.generic_actors.len()], old.save.generic_actors.as_slice());
            let authored = spawn_generic_actors(&current_definition(scene).unwrap()).unwrap();
            assert_eq!(&migrated[old.save.generic_actors.len()..], &authored[old.save.generic_actors.len()..]);
            assert_eq!(capture(&runtime).world_persistent_v1.clockworks.actor_roster_version, 1);
            acknowledge_ready(&runtime, view);
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
            assert_eq!(std::fs::read(&slot_path).unwrap(), slot_bytes);
        }
    }
}

#[test]
fn current_dead_and_damaged_added_actors_survive_continue_without_resurrection() {
    for scene in EXPANDED {
        let runtime = fixture(scene);
        {
            let mut state = runtime.state.lock().unwrap();
            let offset = old_guard_positions(scene).len();
            state.world.generic_actors[offset].take_damage(u32::MAX);
            state.world.generic_actors[offset + 1].take_damage_with_stagger(3, 100);
        }
        let before = actors(&runtime);
        runtime.save().unwrap();
        for _ in 0..2 {
            acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
            assert_eq!(actors(&runtime), before);
        }
    }
}

fn damage_roster(save: &mut SaveV6, fault: &str) {
    let rows = &mut save.save.generic_actors;
    match fault {
        "missing" => { rows.pop(); },
        "extra" => { let mut row = rows[0].clone(); row.entity_id.push_str("_extra"); rows.push(row); },
        "duplicate" => { rows[1] = rows[0].clone(); },
        "id" => rows[0].entity_id.push_str("_altered"),
        "type" => { let row = rows[0].clone(); rows[0] = crate::world_v3::ActorRuntime::spawn(&row.entity_id, if row.entity_type == HOUND { DRONE } else { HOUND }, row.home_m).unwrap(); },
        "old_home" => { rows[0].home_m.x_m += 0.2; },
        "new_home" => { rows.last_mut().unwrap().home_m.x_m += 0.2; },
        "version" => save.world_persistent_v1.clockworks.actor_roster_version = 2,
        _ => unreachable!(),
    }
}

#[test]
fn malformed_current_rosters_reject_save_writes_and_file_slot_continue_atomically() {
    for fault in ["missing", "extra", "duplicate", "id", "type", "old_home", "new_home", "version"] {
        let runtime = fixture("cw_pressure_hall");
        runtime.save().unwrap();
        runtime.save_slot("current", "Current", true).unwrap();
        let path = crate::save_v6::save_path(&runtime.save_root);
        let slot_path = runtime.save_root.join("slots/current/slot-v6.json");
        let good_bytes = std::fs::read(&path).unwrap();
        let good_slot = std::fs::read(&slot_path).unwrap();
        let mut malformed = capture(&runtime); damage_roster(&mut malformed, fault);
        assert!(crate::save_v6::write_save(&runtime.save_root, &malformed).is_err(), "{fault}");
        assert!(crate::save_slots::overwrite_slot_v6(&runtime.save_root, "current", "Bad", &malformed).is_err(), "{fault}");
        assert_eq!(std::fs::read(&path).unwrap(), good_bytes);
        assert_eq!(std::fs::read(&slot_path).unwrap(), good_slot);
        let raw = serde_json::to_vec(&malformed).unwrap(); std::fs::write(&path, &raw).unwrap();
        let mut slot: serde_json::Value = serde_json::from_slice(&good_slot).unwrap();
        slot["save"] = serde_json::to_value(&malformed).unwrap();
        let slot_raw = serde_json::to_vec(&slot).unwrap(); std::fs::write(&slot_path, &slot_raw).unwrap();
        let before = runtime.snapshot().unwrap(); let before_actors = actors(&runtime);
        assert!(runtime.continue_saved().is_err(), "{fault}");
        assert!(runtime.continue_slot("current").is_err(), "{fault}");
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(actors(&runtime), before_actors);
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        assert_eq!(std::fs::read(&slot_path).unwrap(), slot_raw);
    }
}

#[test]
fn version_zero_rejects_partial_mixed_or_shifted_rosters_but_keeps_empty_historical_fallback() {
    for scene in EXPANDED {
        let runtime = fixture(scene);
        let mut save = capture(&runtime); save.world_persistent_v1.clockworks.actor_roster_version = 0;
        for fault in ["missing", "extra", "duplicate", "id", "type", "new_home"] {
            let mut malformed = save.clone(); damage_roster(&mut malformed, fault);
            assert!(malformed.validate().is_err(), "{scene}: {fault}");
        }
        save.save.generic_actors.clear();
        crate::save_v6::write_save(&runtime.save_root, &save).unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(actors(&runtime), spawn_generic_actors(&current_definition(scene).unwrap()).unwrap());
    }
}

#[test]
fn unchanged_clockworks_scenes_current_omissions_reject_while_historical_empty_loads() {
    for scene in ["cw_entry_foundry", "cw_forged_guard_arena", "cw_regulator_core"] {
        let runtime = fixture(scene);
        let mut save = capture(&runtime); save.save.generic_actors.clear();
        assert_eq!(save.validate().unwrap_err(), "E_SAVE_ACTOR_ROSTER_MISMATCH", "{scene}");
        let path = crate::save_v6::save_path(&runtime.save_root);
        let raw = serde_json::to_vec(&save).unwrap(); std::fs::create_dir_all(&runtime.save_root).unwrap(); std::fs::write(&path, &raw).unwrap();
        let before = runtime.snapshot().unwrap();
        assert!(runtime.continue_saved().is_err()); assert_eq!(runtime.snapshot().unwrap(), before);
        assert_eq!(std::fs::read(&path).unwrap(), raw);
        save.world_persistent_v1.clockworks.actor_roster_version = 0;
        // Replace the intentionally corrupt current record for this historical fixture.
        std::fs::write(&path, serde_json::to_vec(&save).unwrap()).unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(capture(&runtime).world_persistent_v1.clockworks.actor_roster_version, 1);
        assert_eq!(actors(&runtime), spawn_generic_actors(&current_definition(scene).unwrap()).unwrap());
    }
}
