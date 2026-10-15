use std::{collections::BTreeSet, fs};

use wuxian_horror_ch1::{
    formal_runtime::FormalRuntime,
    production_scene_bootstrap as production,
    save_v6::{self, SaveV6},
    scene_route_commands,
    world_persistent_v1::{GreyHiveBeaconAction, GreyHiveBeaconStage, PumpStatus},
    world_progression::{HiveChoice, WORLD_CLOCKWORKS, WORLD_GREY_HIVE, WORLD_MIST_HARBOR},
};

const EARNED_SAVE_JSON: &str =
    include_str!("../fixtures/earned-three-world-return-station-v6.json");
const OLD_EARNED_CAPABILITIES: [&str; 4] = [
    "information.enemy_vitals_basic",
    "mobility.air_step_i",
    "perception.acoustic_mapping_i",
    "perception.rear_view_i",
];
const FRESH_SCENE_IDS: [&str; 2] = ["rs_core_room", "gh_entry_maintenance"];
const GREY_HIVE_EVENTS: [&str; 3] = ["hive_power", "hive_lockdown", "hive_extraction"];
const MIST_HARBOR_EVENTS: [&str; 3] = ["mist_beacon_west", "mist_beacon_east", "mist_signal"];
const CLOCKWORKS_EVENTS: [&str; 3] = ["clockworks_valves", "clockworks_core", "clockworks_shutdown"];

#[test]
fn completed_earned_run_starts_fresh_without_overwriting_until_save() {
    let save_dir = super::save_root();
    assert!(save_dir.starts_with(std::env::temp_dir()));
    fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_v6::save_path(&save_dir);
    assert!(!save_path.exists(), "the task-owned save directory is fresh");
    fs::write(&save_path, EARNED_SAVE_JSON.as_bytes()).unwrap();
    let original_bytes = fs::read(&save_path).unwrap();
    assert_eq!(original_bytes.as_slice(), EARNED_SAVE_JSON.as_bytes());

    let earned_save: SaveV6 = serde_json::from_slice(&original_bytes).unwrap();
    earned_save.validate().unwrap();
    assert_completed_earned_run(&earned_save);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    let installed = production::install(&runtime).unwrap();
    assert_eq!(installed.scene_id, "rs_core_room");
    let continued_ready = super::acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    let continued = runtime.pause().unwrap();
    assert_eq!(continued_ready.world_epoch, continued.world_epoch);
    assert_eq!(continued.scene_id, "rs_core_room");
    assert_eq!(continued.world_id, "return_station");
    assert_completed_earned_progress(&continued);
    assert_eq!(fs::read(&save_path).unwrap(), original_bytes, "setup Continue preserves the admitted completed input");

    let earned_epoch = continued.world_epoch;
    let new_prepared = runtime.reset_new().unwrap();
    assert!(new_prepared.world_epoch > earned_epoch);
    assert_eq!(new_prepared.world_id, "return_station");
    assert_eq!(new_prepared.scene_id, "rs_core_room");
    let token = new_prepared.entry_token.as_ref().expect("New prepares an ordinary Return Station entry");
    assert_eq!(token.world_id, "return_station");
    assert_eq!(token.scene_id, "rs_core_room");
    assert_eq!(token.world_epoch, new_prepared.world_epoch);

    let _new_ready = super::acknowledge_ready(&runtime, new_prepared);
    let new_paused = runtime.pause().unwrap();
    assert_eq!(new_paused.scene_id, "rs_core_room");
    assert_eq!(new_paused.world_id, "return_station");
    assert!(new_paused.world_epoch > earned_epoch);
    assert!(new_paused.entry_token.is_none(), "the exact New entry token was acknowledged");
    assert_fresh_progression_view(&new_paused);
    assert_eq!(new_paused.player.current_hp, 100, "New uses WorldStateV3::new starting HP");
    assert_eq!(new_paused.player.max_hp, 100);
    assert_eq!(new_paused.player.current_energy, 100, "New uses WorldStateV3::new starting energy");
    assert_eq!(new_paused.player.max_energy, 100);
    assert_eq!(fs::read(&save_path).unwrap(), original_bytes, "New and readiness do not implicitly overwrite the earned default");

    runtime.resume().unwrap();
    let at_gate = super::walk_x(&runtime, new_paused, 18.0);
    let gate_id = "new-run-rs-to-gh";
    let receipt = scene_route_commands::world_gate(
        &runtime,
        "rs_world_gate_to_gh",
        gate_id,
        at_gate.world_epoch,
    )
    .unwrap();
    super::assert_v3_receipt(&receipt, gate_id, true);
    assert_eq!(receipt.snapshot.world_id, "grey_hive");
    assert_eq!(receipt.snapshot.scene_id, "gh_entry_maintenance");
    assert_eq!(receipt.world_epoch, at_gate.world_epoch + 1);
    let _gh_ready = super::acknowledge_ready(&runtime, runtime.snapshot().unwrap());
    let gh_paused = runtime.pause().unwrap();
    assert_eq!(gh_paused.world_id, "grey_hive");
    assert_eq!(gh_paused.scene_id, "gh_entry_maintenance");
    assert_eq!(gh_paused.world_epoch, receipt.world_epoch);
    assert!(gh_paused.entry_token.is_none(), "the exact first-GH entry token was acknowledged");
    assert_fresh_progression_view(&gh_paused);
    assert_eq!(gh_paused.progression.current_world_id, WORLD_GREY_HIVE);
    assert_eq!(gh_paused.player.current_hp, 100);
    assert_eq!(gh_paused.player.current_energy, 100);
    assert_fresh_entry_roster(&gh_paused);
    assert_eq!(fs::read(&save_path).unwrap(), original_bytes, "normal first-world entry also leaves the earned default untouched until explicit Save");

    let saved_view = runtime.save().unwrap();
    assert_eq!(saved_view.world_epoch, gh_paused.world_epoch);
    assert_eq!(saved_view.server_time_ms, gh_paused.server_time_ms);
    let fresh_save = save_v6::read_save(&save_dir).unwrap();
    assert_fresh_save(&fresh_save, &gh_paused);
    let final_save_bytes = fs::read(&save_path).unwrap();
    assert_ne!(final_save_bytes, original_bytes, "only the explicit fresh Save replaces the earned default");

    println!(
        "Fresh New Journey: completed input preserved through reset; first GH visit {}/{}/revisit {}; two authored entry workers active; explicit save {} ({} bytes)",
        gh_paused.progression.worlds.iter().find(|world| world.world_id == WORLD_GREY_HIVE).unwrap().visit_id,
        gh_paused.progression.worlds.iter().find(|world| world.world_id == WORLD_GREY_HIVE).unwrap().cycle_id,
        gh_paused.progression.worlds.iter().find(|world| world.world_id == WORLD_GREY_HIVE).unwrap().revisit_count,
        save_path.display(),
        fs::metadata(&save_path).unwrap().len(),
    );
}

fn assert_completed_earned_run(save: &SaveV6) {
    assert_eq!(save.save.world_id, "return_station");
    assert_eq!(save.save.scene_id, "rs_core_room");
    assert_eq!(save.save.checkpoint_id, None);
    assert_eq!(save.save.player.position_m.x_m, 4.0);
    assert_eq!(save.save.player.position_m.y_m, 0.0);
    assert_eq!(save.save.player.position_m.z_m, 8.0);
    assert_eq!(save.save.player.current_hp, 46);
    assert_eq!(save.save.player.current_energy, 28);
    assert_eq!(save.save.server_time_ms, 747_048);
    assert_eq!(save.progression.capabilities.len(), 4);
    assert_eq!(save.save.capabilities.grants.len(), 4);
    assert!(save.progression.capabilities.contains(&OLD_EARNED_CAPABILITIES[0].to_owned()));
    assert!(save.progression.capabilities.contains(&OLD_EARNED_CAPABILITIES[1].to_owned()));
    assert!(save.progression.capabilities.contains(&OLD_EARNED_CAPABILITIES[2].to_owned()));
    assert!(save.progression.capabilities.contains(&OLD_EARNED_CAPABILITIES[3].to_owned()));
    for capability in ["mobility.air_step_i", "perception.rear_view_i"] {
        assert!(save.save.capabilities.selected.iter().any(|selected| selected == capability));
    }
    for capability in [
        "information.enemy_vitals_basic",
        "mobility.air_step_i",
        "perception.rear_view_i",
    ] {
        assert!(save.effect_sources.iter().any(|source| source.source_id == capability));
    }

    let worlds = &save.save.progression.progress;
    assert_eq!(worlds.len(), 3);
    for (world_id, expected_events) in [
        (WORLD_GREY_HIVE, &GREY_HIVE_EVENTS),
        (WORLD_MIST_HARBOR, &MIST_HARBOR_EVENTS),
        (WORLD_CLOCKWORKS, &CLOCKWORKS_EVENTS),
    ] {
        let world = worlds.iter().find(|world| world.world_id == world_id).unwrap();
        assert!(world.completed && world.first_completion, "{world_id} is earned in the admitted input");
        assert_eq!(world.visit_id, 1);
        assert_eq!(world.cycle_id, 1);
        assert_eq!(world.revisit_count, 0);
        assert_eq!(
            world.completed_events.iter().map(String::as_str).collect::<BTreeSet<_>>(),
            expected_events.iter().copied().collect::<BTreeSet<_>>(),
        );
    }

    let persistent = &save.world_persistent_v1;
    assert_eq!(persistent.clockworks.pressure_valve_ids.len(), 3);
    assert!(persistent.clockworks.core_console_confirmed);
    assert!(persistent.clockworks.forged_guard_elite_first_kill);
    assert!(persistent.clockworks.regulator_defeated);
    let beacon = persistent.grey_hive.beacon.as_ref().unwrap();
    assert_eq!(beacon.state, GreyHiveBeaconStage::Carried);
    assert_eq!(beacon.receipts.len(), 1);
    assert_eq!(beacon.receipts[0].action, GreyHiveBeaconAction::Collect);
    assert_eq!(persistent.mist_harbor.pump.state, PumpStatus::Drained);
    assert!(persistent.mist_harbor.warden_defeated);
    assert!(!persistent.mist_harbor.explored_region_ids.is_empty());

    let history = persistent.scene_history_v1.as_ref().expect("completed input carries earned scene history");
    let regulator = history.scene_states.iter().find(|row| row.scene_id == "cw_regulator_core").unwrap();
    assert!(regulator.activated_ids.iter().any(|id| id == "cw_regulator_core_console_staged"));
    assert!(regulator.emitted_event_ids.iter().any(|id| id == "clockworks_core"));
}

fn assert_completed_earned_progress(view: &wuxian_horror_ch1::world_v3::WorldView) {
    assert_eq!(view.progression.worlds.len(), 3);
    assert!(view.progression.worlds.iter().all(|world| world.completed && world.first_completion));
    for capability in ["mobility.air_step_i", "perception.rear_view_i"] {
        let item = view.capabilities.items.iter().find(|item| item.capability_id == capability).unwrap();
        assert!(item.granted && item.selected);
    }
}

fn assert_fresh_progression_view(view: &wuxian_horror_ch1::world_v3::WorldView) {
    assert_eq!(view.progression.worlds.len(), 3);
    assert_eq!(view.progression.current_world_id, WORLD_GREY_HIVE);
    for world in &view.progression.worlds {
        assert!(!world.completed);
        assert!(!world.first_completion);
        assert_eq!(world.revisit_count, 0);
        assert!(world.completed_events.is_empty());
    }
    let grey_hive = view.progression.worlds.iter().find(|world| world.world_id == WORLD_GREY_HIVE).unwrap();
    let mist_harbor = view.progression.worlds.iter().find(|world| world.world_id == WORLD_MIST_HARBOR).unwrap();
    let clockworks = view.progression.worlds.iter().find(|world| world.world_id == WORLD_CLOCKWORKS).unwrap();
    assert_eq!((grey_hive.visit_id, grey_hive.cycle_id), (1, 1));
    assert_eq!((mist_harbor.visit_id, mist_harbor.cycle_id), (0, 1));
    assert_eq!((clockworks.visit_id, clockworks.cycle_id), (0, 1));
    assert_eq!(grey_hive.hive_choice, Some(HiveChoice::Unresolved));
    assert!(view.capabilities.first_enhancement_choice.is_none());
    assert!(view.capabilities.items.iter().filter(|item| OLD_EARNED_CAPABILITIES.contains(&item.capability_id.as_str()))
        .all(|item| !item.granted && !item.selected));
    assert!(!view.capabilities.rear_view.granted);
    assert_ne!(view.capabilities.acoustic_mapping_authorized, Some(true));
}

fn assert_fresh_entry_roster(view: &wuxian_horror_ch1::world_v3::WorldView) {
    let authored_worker_ids = ["gh_entry_worker_01", "gh_entry_worker_02"];
    let workers = view.actors.iter()
        .filter(|actor| authored_worker_ids.contains(&actor.entity_id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        workers.iter().map(|actor| actor.entity_id.as_str()).collect::<BTreeSet<_>>(),
        BTreeSet::from(authored_worker_ids),
    );
    assert!(workers.iter().all(|actor| {
        actor.entity_type == "runtime2d.enemy.infected_maintenance_worker.v1" && actor.active
    }));
}

fn assert_fresh_save(save: &SaveV6, paused_view: &wuxian_horror_ch1::world_v3::WorldView) {
    assert_eq!(save.save.world_id, "grey_hive");
    assert_eq!(save.save.scene_id, "gh_entry_maintenance");
    assert_eq!(save.save.checkpoint_id, paused_view.checkpoint_id);
    assert_eq!(save.save.player.position_m, paused_view.player.transform.position_m);
    assert_eq!(save.save.player.current_hp, paused_view.player.current_hp);
    assert_eq!(save.save.player.max_hp, paused_view.player.max_hp);
    assert_eq!(save.save.player.current_energy, paused_view.player.current_energy);
    assert_eq!(save.save.player.max_energy, paused_view.player.max_energy);
    assert_eq!(save.save.server_time_ms, paused_view.server_time_ms);
    assert_eq!(save.save.progression.current_world_id, WORLD_GREY_HIVE);
    assert_eq!(save.save.progression.reward_ledger.len(), 0);
    assert!(save.save.progression.progress.iter().all(|world| {
        !world.completed && !world.first_completion && world.revisit_count == 0
            && world.completed_events.is_empty() && world.first_mainline_result.is_none()
    }));
    for (world_id, visit_id) in [(WORLD_GREY_HIVE, 1), (WORLD_MIST_HARBOR, 0), (WORLD_CLOCKWORKS, 0)] {
        let world = save.save.progression.progress.iter().find(|world| world.world_id == world_id).unwrap();
        assert_eq!(world.visit_id, visit_id);
        assert_eq!(world.cycle_id, 1);
    }
    assert_eq!(
        save.save.progression.progress.iter().find(|world| world.world_id == WORLD_GREY_HIVE).unwrap().hive_choice,
        HiveChoice::Unresolved,
    );
    assert!(save.save.capabilities.selected.is_empty());
    assert!(save.save.capabilities.grants.iter().all(|grant| !OLD_EARNED_CAPABILITIES.contains(&grant.capability_id.as_str())));
    assert!(save.progression.capabilities.iter().all(|id| !OLD_EARNED_CAPABILITIES.contains(&id.as_str())));
    assert!(save.effect_sources.iter().all(|source| !OLD_EARNED_CAPABILITIES.contains(&source.source_id.as_str())));

    let persistent = &save.world_persistent_v1;
    assert!(persistent.clockworks.pressure_valve_ids.is_empty());
    assert!(!persistent.clockworks.core_console_confirmed);
    assert!(!persistent.clockworks.forged_guard_elite_first_kill);
    assert!(!persistent.clockworks.regulator_defeated);
    assert!(!persistent.grey_hive.sentinel_first_kill);
    let beacon = persistent.grey_hive.beacon.as_ref().expect("fresh production New initializes its physical beacon as uncollected");
    assert_eq!(beacon.state, GreyHiveBeaconStage::Uncollected);
    assert!(beacon.receipts.is_empty());
    assert!(!persistent.mist_harbor.warden_defeated);
    assert_eq!(persistent.mist_harbor.pump.state, PumpStatus::Ready);
    assert!(persistent.mist_harbor.explored_region_ids.is_empty());

    assert_eq!(save.save.scene_states.len(), 1);
    assert_eq!(save.save.scene_states[0].scene_id, "gh_entry_maintenance");
    if let Some(history) = persistent.scene_history_v1.as_ref() {
        assert!(history.scene_states.iter().all(|row| FRESH_SCENE_IDS.contains(&row.scene_id.as_str())));
        for row in &history.scene_states {
            assert!(row.emitted_event_ids.is_empty(), "New carries no old scene events");
            assert!(row.checkpoint_id.is_none(), "fresh current scenes have no old checkpoint");
            if row.scene_id == "gh_entry_maintenance" {
                assert!(row.activated_ids.is_empty(), "no GH interaction was used on the fresh entry");
            } else {
                assert!(row.activated_ids.iter().all(|id| id == "rs_world_gate_to_gh"),
                    "the only possible fresh RS activation is the actual gate used for this entry");
            }
        }
    }
}
