//! Earned Beacon route extension. Movement and actions use public projections;
//! persisted fields are read only to assert actual receipts and exact restoration.
use super::*;
use std::path::Path;
use wuxian_horror_ch1::{
    world_persistent_v1::{GreyHiveBeaconAction as Action, GreyHiveBeaconStage as Stage},
    world_v3::WorldView,
};
const COLLECT: &str = "gh_beacon_deploy_marker";
const MOUNT: &str = "gh_beacon_storage_mount_marker";

fn assert_stage(view: &WorldView, expected: Stage) {
    let beacon = view.grey_hive_beacon.as_ref().expect("native public Beacon state");
    assert_eq!(beacon.state, expected);
    assert!(!beacon.legacy_completed_without_receipt, "this is a genuinely new journey");
    assert!(view.player.current_hp > 0);
}

fn assert_unchanged_view(before: &WorldView, after: &WorldView) {
    // The live owner may run between two commands. Only its three clock fields
    // may advance, by exactly the same number of fixed 17 ms owner steps.
    let ticks = after.server_tick.checked_sub(before.server_tick)
        .expect("a repeated/rejected action cannot rewind the owner clock");
    assert_eq!(after.authority_revision.checked_sub(before.authority_revision), Some(ticks));
    let elapsed = after.server_time_ms.checked_sub(before.server_time_ms)
        .expect("a repeated/rejected action cannot rewind server time");
    assert_eq!(elapsed, ticks.checked_mul(17).expect("owner time delta must be representable"));
    let mut current = after.clone();
    current.server_tick = before.server_tick;
    current.authority_revision = before.authority_revision;
    current.server_time_ms = before.server_time_ms;
    assert_eq!(&current, before,
        "rejected or repeated Beacon actions cannot alter any non-clock public field");
}

fn approach(runtime: &FormalRuntime, mut view: WorldView, id: &str) -> WorldView {
    let target = view.interactables.iter().find(|item| item.entity_id == id)
        .unwrap_or_else(|| panic!("missing public target {id}")).transform.position_m;
    let arrival = if view.interactables.iter().any(|item| item.entity_id == id && item.kind == "scene_transition") { 0.15 } else { 1.8 };
    for _ in 0..1200 {
        let player = view.player.transform.position_m;
        let (dx, dz) = (target.x_m - player.x_m, target.z_m - player.z_m);
        let distance = dx.hypot(dz);
        if distance <= arrival {
            return runtime.submit_input(InputSample::new(view.world_epoch, view.ack_seq + 1,
                NEXT_TIME.fetch_add(17, Ordering::Relaxed), 0.0, 0.0).unwrap(), vec![]).unwrap();
        }
        view = runtime.submit_input(InputSample::new(view.world_epoch, view.ack_seq + 1,
            NEXT_TIME.fetch_add(17, Ordering::Relaxed), dx / distance, dz / distance).unwrap(), vec![]).unwrap();
        assert!(view.player.current_hp > 0, "real Beacon approach must survive");
    }
    panic!("public Beacon route could not reach {id}");
}

fn preserve(runtime: &FormalRuntime, dir: &Path, stage: Stage, actions: &[Action]) -> WorldView {
    // Save/Continue compares an exact checkpoint, so hold the owner through the
    // second save. Live cross-tick rejection checks above remain live.
    runtime.pause().unwrap();
    let before = runtime.save().unwrap();
    let path = save_v6::save_path(dir);
    let bytes = fs::read(&path).unwrap();
    let saved = save_v6::read_save(dir).unwrap();
    let record = saved.world_persistent_v1.grey_hive.beacon.as_ref().unwrap();
    assert_eq!(record.state, stage);
    assert_eq!(record.receipts.iter().map(|receipt| receipt.action).collect::<Vec<_>>(), actions);
    assert_eq!(record.receipts.len(), actions.len(), "only real actions grant receipts");
    let prepared = runtime.continue_saved().unwrap();
    assert_eq!(fs::read(&path).unwrap(), bytes, "Continue cannot rewrite the original Beacon save");
    assert_eq!(runtime.resume().unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
    assert_eq!(runtime.snapshot().unwrap(), prepared);
    let continued = runtime.scene_ready(prepared.entry_token.as_ref().unwrap(), true).unwrap();
    assert_stage(&continued, stage);
    assert_eq!(continued.player.transform.position_m, before.player.transform.position_m);
    assert_eq!(continued.player.current_hp, before.player.current_hp);
    assert_eq!(continued.player.current_energy, before.player.current_energy);
    assert_eq!(continued.server_tick, before.server_tick);
    assert_eq!(continued.progression, before.progression);
    assert_eq!(continued.capabilities, before.capabilities);
    assert!(runtime.presentation_events_since(continued.world_epoch, 0).unwrap().iter()
        .all(|event| !matches!(event.kind.as_str(), "GreyHiveBeaconCollected" | "GreyHiveBeaconMounted")),
        "Continue cannot re-emit physical Beacon actions");
    runtime.save().unwrap();
    let after = save_v6::read_save(dir).unwrap();
    assert_eq!(after.world_persistent_v1, saved.world_persistent_v1);
    assert_eq!(after.save.server_time_ms, saved.save.server_time_ms);
    assert_eq!(after.save.generic_actors, saved.save.generic_actors);
    runtime.resume().unwrap()
}

pub(super) fn collect_and_mount(runtime: &FormalRuntime, mut view: WorldView, dir: &Path) -> WorldView {
    assert_eq!(view.scene_id, "gh_beacon");
    assert_stage(&view, Stage::Uncollected);
    // A new journey cannot skip the device. Test the real exit rejection, then
    // walk the newly authored recovery connection instead of editing a save.
    view = approach(runtime, view, "gh_beacon_to_exit");
    view = apply_transition(runtime, "gh_beacon_to_exit", "beacon-missing-to-exit", view.world_epoch);
    view = approach(runtime, view, "gh_exit_extraction_console");
    let rejected = runtime.activate_scene_interaction("gh_exit_extraction_console", "beacon-missing-extraction", view.world_epoch).unwrap();
    assert!(!rejected.applied && !rejected.already_applied);
    assert_eq!(rejected.error_code.as_deref(), Some("E_BEACON_REQUIRED"));
    assert_unchanged_view(&view, &rejected.view);
    assert_stage(&rejected.view, Stage::Uncollected);
    view = approach(runtime, rejected.view, "gh_exit_to_beacon");
    view = apply_transition(runtime, "gh_exit_to_beacon", "beacon-recovery-return", view.world_epoch);
    view = approach(runtime, view, COLLECT);
    let collect_epoch = view.world_epoch;
    let collected = runtime.activate_scene_interaction(COLLECT, "genuine-beacon-collect", collect_epoch).unwrap();
    assert!(collected.applied && !collected.already_applied, "{:?}", collected.error_code);
    assert_v3_receipt(&collected.receipt, "genuine-beacon-collect", true);
    assert_eq!(collected.receipt.snapshot.grey_hive_beacon, collected.view.grey_hive_beacon);
    assert!(collected.events.is_empty(), "Beacon collection adds no route reward/event");
    assert_eq!(collected.view.progression, view.progression);
    assert_eq!(collected.view.capabilities, view.capabilities);
    assert_stage(&collected.view, Stage::Carried);
    let duplicate = runtime.activate_scene_interaction(COLLECT, "genuine-beacon-collect", collect_epoch).unwrap();
    assert!(!duplicate.applied && duplicate.already_applied);
    assert_unchanged_view(&collected.view, &duplicate.view);
    assert_eq!(runtime.presentation_events_since(collect_epoch, 0).unwrap().iter()
        .filter(|event| event.kind == "GreyHiveBeaconCollected").count(), 1);
    view = preserve(runtime, dir, Stage::Carried, &[Action::Collect]);
    let saved = save_v6::read_save(dir).unwrap();
    let receipt = &saved.world_persistent_v1.grey_hive.beacon.as_ref().unwrap().receipts[0];
    assert_eq!(receipt.request_id, "genuine-beacon-collect");
    assert_eq!(receipt.world_epoch, collect_epoch);
    let replay = runtime.activate_scene_interaction(COLLECT, "genuine-beacon-collect", view.world_epoch).unwrap();
    assert!(!replay.applied && replay.already_applied);
    assert_unchanged_view(&view, &replay.view);
    // Carried ownership alone exposes the genuine extraction control. Mounting
    // remains an optional action; this driver exercises it after that check.
    view = approach(runtime, replay.view, "gh_beacon_to_exit");
    view = apply_transition(runtime, "gh_beacon_to_exit", "beacon-carried-to-exit", view.world_epoch);
    assert!(view.interactables.iter().any(|item| item.entity_id == "gh_exit_extraction_console" && item.active));
    view = approach(runtime, view, "gh_exit_to_beacon");
    view = apply_transition(runtime, "gh_exit_to_beacon", "beacon-carried-return", view.world_epoch);
    view = approach(runtime, view, MOUNT);
    let mount_epoch = view.world_epoch;
    let mounted = runtime.activate_scene_interaction(MOUNT, "genuine-beacon-mount", mount_epoch).unwrap();
    assert!(mounted.applied && !mounted.already_applied, "{:?}", mounted.error_code);
    assert_v3_receipt(&mounted.receipt, "genuine-beacon-mount", true);
    assert_eq!(mounted.receipt.snapshot.grey_hive_beacon, mounted.view.grey_hive_beacon);
    assert!(mounted.events.is_empty());
    assert_eq!(mounted.view.progression, view.progression);
    assert_eq!(mounted.view.capabilities, view.capabilities);
    assert_stage(&mounted.view, Stage::Mounted);
    let duplicate = runtime.activate_scene_interaction(MOUNT, "genuine-beacon-mount", mount_epoch).unwrap();
    assert!(!duplicate.applied && duplicate.already_applied);
    assert_unchanged_view(&mounted.view, &duplicate.view);
    assert_eq!(runtime.presentation_events_since(mount_epoch, 0).unwrap().iter()
        .filter(|event| event.kind == "GreyHiveBeaconMounted").count(), 1);
    view = preserve(runtime, dir, Stage::Mounted, &[Action::Collect, Action::Mount]);
    let saved = save_v6::read_save(dir).unwrap();
    let receipts = &saved.world_persistent_v1.grey_hive.beacon.as_ref().unwrap().receipts;
    assert_eq!(receipts[0].world_epoch, collect_epoch);
    assert_eq!(receipts[1].world_epoch, mount_epoch);
    assert_eq!(receipts[1].request_id, "genuine-beacon-mount");
    let replay = runtime.activate_scene_interaction(MOUNT, "genuine-beacon-mount", view.world_epoch).unwrap();
    assert!(!replay.applied && replay.already_applied);
    assert_unchanged_view(&view, &replay.view);
    runtime.save().unwrap();
    assert_eq!(save_v6::read_save(dir).unwrap().world_persistent_v1, saved.world_persistent_v1);
    println!("Genuine Beacon: uncollected extraction rejected; real return, collect and optional mount; two exact durable receipts; both Save/Continue and replay guards passed; {} HP", view.player.current_hp);
    replay.view
}

#[cfg(test)]
mod clock_regressions {
    use super::*;
    use std::{thread, time::{Duration, Instant}};
    use wuxian_horror_ch1::world_v3::{ActorRuntime, GreyHiveBeaconProjection};

    #[test]
    fn live_rejected_beacon_request_allows_only_real_owner_clock_progress() {
        let runtime = FormalRuntime::new_with_save_dir(save_root()).unwrap();
        let before = runtime.snapshot().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.snapshot().unwrap().server_tick == before.server_tick {
            assert!(Instant::now() < deadline, "live owner must advance");
            thread::sleep(Duration::from_millis(5));
        }
        let rejected = runtime.activate_scene_interaction(COLLECT, "live-clock-rejection", before.world_epoch).unwrap();
        assert!(!rejected.applied && !rejected.already_applied);
        assert_eq!(rejected.error_code.as_deref(), Some("E_SCENE_REGISTRY_NOT_LOADED"));
        assert!(rejected.view.server_tick > before.server_tick);
        assert_unchanged_view(&before, &rejected.view);
        let held = runtime.pause().unwrap();
        let repeated = runtime.activate_scene_interaction(COLLECT, "held-clock-rejection", held.world_epoch).unwrap();
        assert_eq!(repeated.view, held, "a held owner still requires exact full-view equality");
    }

    #[test]
    fn clock_normalization_cannot_hide_business_mutations_or_invalid_time() {
        let runtime = FormalRuntime::new_with_save_dir(save_root()).unwrap();
        let mut before = runtime.pause().unwrap();
        before.server_tick = 10;
        before.authority_revision = 20;
        before.server_time_ms = 170;
        before.grey_hive_beacon = Some(GreyHiveBeaconProjection { state: Stage::Carried, legacy_completed_without_receipt: false });
        before.actors = vec![ActorRuntime::spawn("ordinary", "enemy.mist_harbor.drowned", wuxian_horror_ch1::world_v3::Vec3::zero()).unwrap().view()];
        let mut advanced = before.clone();
        advanced.server_tick += 2;
        advanced.authority_revision += 2;
        advanced.server_time_ms += 34;
        assert_unchanged_view(&before, &advanced);
        for fault in ["rewind", "revision", "time", "hp", "energy", "position", "epoch", "input", "capability", "progression", "beacon", "actor"] {
            let mut after = advanced.clone();
            match fault {
                "rewind" => after.server_tick = 9,
                "revision" => after.authority_revision += 1,
                "time" => after.server_time_ms += 1,
                "hp" => after.player.current_hp -= 1,
                "energy" => after.player.current_energy -= 1,
                "position" => after.player.transform.position_m.x_m += 1.,
                "epoch" => after.world_epoch += 1,
                "input" => after.ack_seq += 1,
                "capability" => after.capabilities.acoustic_mapping_authorized = Some(true),
                "progression" => after.progression.event_seq += 1,
                "beacon" => after.grey_hive_beacon.as_mut().unwrap().state = Stage::Mounted,
                "actor" => after.actors[0].active = false,
                _ => unreachable!(),
            }
            assert!(std::panic::catch_unwind(|| assert_unchanged_view(&before, &after)).is_err(), "{fault} must remain a hard failure");
        }
    }
}
