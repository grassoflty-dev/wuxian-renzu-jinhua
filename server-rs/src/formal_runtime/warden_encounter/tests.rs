use super::*;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn fixture() -> (FormalRuntime, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "warden-{}-{}",
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
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        let mut route = RouteState::new();
        let revision = state.world.revision;
        for (index, command) in [
            RouteCommand::Enter {
                world_id: "grey_hive".into(),
                request_id: "gh".into(),
            },
            RouteCommand::Progress {
                event_id: "hive_power".into(),
                request_id: "power".into(),
            },
            RouteCommand::Progress {
                event_id: "hive_lockdown".into(),
                request_id: "lockdown".into(),
            },
            RouteCommand::Progress {
                event_id: "hive_extraction".into(),
                request_id: "extract".into(),
            },
            RouteCommand::Complete {
                world_id: "grey_hive".into(),
                request_id: "gh-end".into(),
            },
            RouteCommand::Enter {
                world_id: WORLD_ID.into(),
                request_id: "mh".into(),
            },
            RouteCommand::Progress {
                event_id: "mist_beacon_west".into(),
                request_id: "west".into(),
            },
            RouteCommand::Progress {
                event_id: "mist_beacon_east".into(),
                request_id: "east".into(),
            },
            RouteCommand::Progress {
                event_id: "mist_signal".into(),
                request_id: "signal".into(),
            },
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                matches!(
                    apply_route_command(&mut route, command, revision),
                    RouteResult::Applied { .. }
                ),
                "command {index}"
            );
        }
        state.route = route;
    }
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    runtime.install_scene_registry(registry, SCENE_ID).unwrap();
    (runtime, root)
}
fn step(runtime: &FormalRuntime) {
    let mut state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    advance_owner_step_with_input_age(&mut state, scene.as_ref(), Some(Duration::ZERO)).unwrap();
}
fn capture(runtime: &FormalRuntime) -> crate::save_v6::SaveV6 {
    let state = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    crate::save_v6::SaveV6::capture(&state, scene.as_ref()).unwrap()
}
#[test]
fn canonical_actor_and_boss_bar_do_not_require_mapping_or_enemy_vitals() {
    let (runtime, _) = fixture();
    let view = runtime.snapshot().unwrap();
    assert_eq!(view.actors.len(), 1);
    let boss = view.boss_encounter.unwrap();
    assert_eq!(boss.entity_id, ACTOR_ID);
    assert_eq!(boss.current_hp, 360);
    assert!(boss.temporary_visual);
    assert!(!boss.public_release_eligible);
    assert!(!boss.mapped_true_source);
    assert!(boss.warning.is_none());
    assert!(!runtime
        .state
        .lock()
        .unwrap()
        .effective_rules_v6
        .capability_permissions
        .contains(&CapabilityPermission::AcousticMapping));
    assert!(
        !view
            .interactables
            .iter()
            .find(|x| x.entity_id == "mh_warden_arena_to_extraction")
            .unwrap()
            .active
    );
}
#[test]
fn real_owner_combat_sets_first_kill_once_without_invented_event_and_unlocks_exit() {
    let (runtime, _) = fixture();
    {
        let mut s = runtime.state.lock().unwrap();
        s.world.player.position_m = Vec3::new(17.0, 0.0, 8.0).unwrap();
        s.world.generic_actors[0].hp = 1;
        s.latest_sample.aim_x = 1.0;
        s.latest_sample.aim_z = 0.0;
        s.pending_combat
            .push(crate::continuous_combat::CombatIntent::ActionAttack { request_id: 1 });
    }
    let events_before = runtime.snapshot().unwrap().progression;
    for _ in 0..60 {
        step(&runtime);
    }
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(22.0, 0.0, 8.0).unwrap();
    let view = runtime.snapshot().unwrap();
    assert!(view.boss_encounter.is_none());
    assert!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .warden_defeated
    );
    assert_eq!(view.progression, events_before);
    assert!(
        view.interactables
            .iter()
            .find(|x| x.entity_id == "mh_warden_arena_to_extraction")
            .unwrap()
            .active
    );
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .presentation_events
            .iter()
            .filter(|e| e.kind == "ResonanceWardenDeath")
            .count(),
        1
    );
    for _ in 0..60 {
        step(&runtime);
    }
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .presentation_events
            .iter()
            .filter(|e| e.kind == "ResonanceWardenDeath")
            .count(),
        1
    );
    assert!(capture(&runtime).validate().is_ok());
}
#[test]
fn externally_dead_actor_and_stale_proof_never_authorize_defeat() {
    let (runtime, _) = fixture();
    let mut s = runtime.state.lock().unwrap();
    let scene = runtime.scene_runtime.lock().unwrap();
    let proof = capture_combat(&s, scene.as_ref());
    s.world.generic_actors[0].take_damage(u32::MAX);
    assert!(!record_defeat(&mut s, scene.as_ref(), proof, &[]));
    assert!(!s.world_persistent_v1.mist_harbor.warden_defeated);
    assert!(crate::save_v6::SaveV6::capture(&s, scene.as_ref()).is_err());
}
#[test]
fn arena_exit_and_extraction_require_real_defeat_after_all_three_events() {
    let (runtime, _) = fixture();
    {
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(22.0, 0.0, 8.0).unwrap();
    }
    let before = runtime.snapshot().unwrap();
    assert_eq!(
        runtime
            .transition_scene(
                "mh_warden_arena_to_extraction",
                "blocked",
                before.world_epoch
            )
            .unwrap_err(),
        "E_MH_WARDEN_DEFEAT_REQUIRED"
    );
    assert_eq!(runtime.snapshot().unwrap(), before);
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    let view = runtime
        .install_scene_registry(registry, "mh_extraction")
        .unwrap();
    assert!(!view
        .interactables
        .iter()
        .any(|x| x.entity_id == "mh_extraction_return_to_rs"));
    let result = crate::scene_route_commands::world_gate(
        &runtime,
        "mh_extraction_return_to_rs",
        "blocked-extract",
        view.world_epoch,
    )
    .unwrap();
    assert!(!result.applied);
    assert_eq!(
        result.error_code.as_deref(),
        Some("E_MH_WARDEN_DEFEAT_REQUIRED")
    );
}
#[test]
fn live_windup_save_restores_exact_geometry_serial_and_dead_revisit_stays_dead() {
    let (runtime, root) = fixture();
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(16.0, 0.0, 8.0).unwrap();
    for _ in 0..8 {
        step(&runtime);
    }
    let saved = capture(&runtime);
    let actor = saved.save.generic_actors[0].clone();
    assert_eq!(actor.warden.as_ref().unwrap().stage, WardenStage::Windup);
    runtime.save().unwrap();
    let bytes = std::fs::read(root.join(crate::save_v6::SAVE_V6_FILE_NAME)).unwrap();
    runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    assert_eq!(runtime.state.lock().unwrap().world.generic_actors[0], actor);
    assert_eq!(
        std::fs::read(root.join(crate::save_v6::SAVE_V6_FILE_NAME)).unwrap(),
        bytes
    );
    // Low-level current-profile restore proves serialization; loading safety is
    // separately covered by the pending entry-readiness handshake, not this test.
    {
        let mut s = runtime.state.lock().unwrap();
        s.world.generic_actors[0].take_damage(u32::MAX);
        s.world_persistent_v1.mist_harbor.warden_defeated = true;
    }
    runtime.save().unwrap();
    runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    runtime
        .install_scene_registry(registry.clone(), "mh_resonance_tower")
        .unwrap();
    let view = runtime.install_scene_registry(registry, SCENE_ID).unwrap();
    assert!(view.boss_encounter.is_none());
    assert_eq!(runtime.state.lock().unwrap().world.generic_actors[0].hp, 0);
}
#[test]
fn malformed_current_saved_warden_states_fail_closed() {
    let (runtime, _) = fixture();
    let save = capture(&runtime);
    for mutate in [
        |v: &mut serde_json::Value| {
            v["save"]["genericActors"][0]["warden"]["remainingMs"] = json!(999999);
        },
        |v: &mut serde_json::Value| {
            v["save"]["genericActors"][0]["entityId"] = json!("forged");
        },
        |v: &mut serde_json::Value| {
            v["worldPersistentV1"]["mistHarbor"]["wardenDefeated"] = json!(true);
        },
        |v: &mut serde_json::Value| {
            v["save"]["sceneId"] = json!("mh_fog_pier");
        },
    ] {
        let mut value = serde_json::to_value(&save).unwrap();
        mutate(&mut value);
        let forged: crate::save_v6::SaveV6 = serde_json::from_value(value).unwrap();
        assert!(forged.validate().is_err());
    }
}
#[test]
fn legacy_profile_rejects_new_warden_keys_and_retains_original_disk_bytes() {
    let (runtime, root) = fixture();
    let mut raw = serde_json::to_value(capture(&runtime)).unwrap();
    raw.as_object_mut().unwrap().remove("effectSourcesVersion");
    std::fs::create_dir_all(&root).unwrap();
    let bytes = serde_json::to_vec(&raw).unwrap();
    let path = root.join(crate::save_v6::SAVE_V6_FILE_NAME);
    std::fs::write(&path, &bytes).unwrap();
    assert!(crate::save_v6::read_or_migrate(&root).is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[test]
fn true_cues_are_ordinary_but_mapping_only_adds_early_source_projection() {
    let (runtime, _) = fixture();
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(16.0, 0.0, 8.0).unwrap();
    for _ in 0..3 {
        step(&runtime);
    }
    let view = runtime.snapshot().unwrap();
    assert!(view.boss_encounter.unwrap().warning.is_none());
    {
        let s = runtime.state.lock().unwrap();
        assert!(s
            .presentation_events
            .iter()
            .any(|e| e.kind == "ResonanceWardenStrikeWindup"));
        let cue = s
            .sound_cues
            .iter()
            .find(|e| e.kind == "warden_true_call")
            .unwrap();
        assert!(cue.direction_rad.is_none() && cue.distance_m.is_none());
    }
    // This unit fixture installs a resolved permission directly to isolate the
    // consumer. Acquisition/source authority is tested by capability migration.
    runtime
        .state
        .lock()
        .unwrap()
        .effective_rules_v6
        .capability_permissions
        .insert(CapabilityPermission::AcousticMapping);
    let boss = runtime.snapshot().unwrap().boss_encounter.unwrap();
    assert!(boss.mapped_true_source);
    assert!(boss.warning.unwrap().remaining_ms > 600);
    runtime
        .state
        .lock()
        .unwrap()
        .effective_rules_v6
        .capability_permissions
        .remove(&CapabilityPermission::AcousticMapping);
    for _ in 0..35 {
        step(&runtime);
    }
    let boss = runtime.snapshot().unwrap().boss_encounter.unwrap();
    assert!(!boss.mapped_true_source);
    assert!(boss.warning.is_some());
}
#[test]
fn paused_real_owner_freezes_pending_attack_geometry_clock_and_hp() {
    let (runtime, _) = fixture();
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(16.0, 0.0, 8.0).unwrap();
    for _ in 0..8 {
        step(&runtime);
    }
    runtime.pause().unwrap();
    let before = runtime.state.lock().unwrap().world.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let owner_stop = Arc::clone(&stop);
    let owner_state = Arc::clone(&runtime.state);
    let owner_scene = Arc::clone(&runtime.scene_runtime);
    let (tx, rx) = std::sync::mpsc::sync_channel(0);
    let handle = std::thread::spawn(move || {
        tx.send(()).unwrap();
        simulation_owner_loop(owner_state, owner_scene, owner_stop);
    });
    rx.recv().unwrap();
    std::thread::sleep(Duration::from_millis(60));
    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();
    let after = &runtime.state.lock().unwrap().world;
    assert_eq!(after.server_time_ms, before.server_time_ms);
    assert_eq!(after.revision, before.revision);
    assert_eq!(after.generic_actors, before.generic_actors);
    assert_eq!(after.player_hp, before.player_hp);
}
#[test]
fn frozen_legacy_files_and_slots_reject_even_empty_warden_presence_without_rewrite() {
    let root = std::env::temp_dir().join(format!(
        "warden-legacy-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(root.join("slots/frozen")).unwrap();
    let old: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/capability-save-profiles/legacy-empty.json"
    ))
    .unwrap();
    let slot: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/capability-save-profiles/legacy-slot.json"
    ))
    .unwrap();
    let original = serde_json::to_vec(&old).unwrap();
    let path = root.join(crate::save_v6::SAVE_V6_FILE_NAME);
    std::fs::write(&path, &original).unwrap();
    crate::save_v6::read_save(&root).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original);
    for value in [json!(false), json!(true), serde_json::Value::Null] {
        let mut raw = old.clone();
        raw["worldPersistentV1"]["mistHarbor"]["wardenDefeated"] = value;
        let bytes = serde_json::to_vec(&raw).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(crate::save_v6::read_save(&root).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let mut raw_slot = slot.clone();
        raw_slot["save"] = raw;
        let bytes = serde_json::to_vec(&raw_slot).unwrap();
        let path = root.join("slots/frozen/slot-v6.json");
        std::fs::write(&path, &bytes).unwrap();
        assert!(crate::save_slots::read_slot_v6(&root, "frozen").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}
#[test]
fn pre_warden_empty_arena_current_save_spawns_live_boss_only_in_memory() {
    let (runtime, root) = fixture();
    let mut old = capture(&runtime);
    old.save.generic_actors.clear();
    old.validate().unwrap();
    crate::save_v6::write_save(&root, &old).unwrap();
    let path = root.join(crate::save_v6::SAVE_V6_FILE_NAME);
    let bytes = std::fs::read(&path).unwrap();
    let view = runtime.continue_saved().map(|view| crate::formal_runtime::entry_test_support::acknowledge_ready(&runtime, view)).unwrap();
    assert_eq!(view.boss_encounter.unwrap().current_hp, 360);
    assert!(
        !runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .warden_defeated
    );
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
#[cfg(feature = "deterministic-replay")]
#[test]
fn ordinary_sound_driven_movement_and_primary_combat_can_finish_both_phases_without_mapping() {
    let (mut runtime, _) = fixture();
    runtime.input_replay = true;
    runtime.state.lock().unwrap().world.player.position_m = Vec3::new(14.0, 0.0, 8.0).unwrap();
    let mut cursor = 0;
    let mut escape: Option<Vec3> = None;
    let mut decoys = 0;
    let mut strikes = 0;
    let mut pulses = 0;
    let mut finished = false;
    for seq in 1..=12_000_u64 {
        let (epoch, position, boss_position, dead) = {
            let s = runtime.state.lock().unwrap();
            for e in s.presentation_events.iter().filter(|e| e.event_id > cursor) {
                match e.kind.as_str() {
                    "ResonanceWardenStrikeWindup" => {
                        strikes += 1;
                        let a = e.direction_rad + std::f32::consts::FRAC_PI_2;
                        escape = Some(
                            Vec3::new(
                                (e.position_m.x_m + a.sin() * 1.3).clamp(1.0, 23.0),
                                0.0,
                                (e.position_m.z_m + a.cos() * 1.3).clamp(1.0, 15.0),
                            )
                            .unwrap(),
                        );
                    }
                    "ResonanceWardenPulseWindup" => {
                        pulses += 1;
                        let dx = s.world.player.position_m.x_m - e.position_m.x_m;
                        let dz = s.world.player.position_m.z_m - e.position_m.z_m;
                        let d = (dx * dx + dz * dz).sqrt().max(0.01);
                        escape = Some(
                            Vec3::new(
                                (e.position_m.x_m + dx / d * 4.6).clamp(1.0, 23.0),
                                0.0,
                                (e.position_m.z_m + dz / d * 4.6).clamp(1.0, 15.0),
                            )
                            .unwrap(),
                        );
                    }
                    "ResonanceWardenStrikeImpact" | "ResonanceWardenPulseImpact" => escape = None,
                    "ResonanceWardenDecoy" => decoys += 1,
                    _ => {}
                }
            }
            cursor = s
                .presentation_events
                .last()
                .map(|e| e.event_id)
                .unwrap_or(cursor);
            assert!(!s
                .effective_rules_v6
                .capability_permissions
                .contains(&CapabilityPermission::AcousticMapping));
            assert!(
                s.world.player_hp > 0,
                "ordinary strategy died after {seq} ticks with Warden HP {}",
                s.world.generic_actors[0].hp
            );
            (
                s.world.revision.world_epoch,
                s.world.player.position_m,
                s.world.generic_actors[0].position_m,
                s.world_persistent_v1.mist_harbor.warden_defeated,
            )
        };
        if dead {
            finished = true;
            eprintln!(
                "ordinary Warden fight completed in {} scheduled ticks, player HP {}",
                seq,
                runtime.snapshot().unwrap().player.current_hp
            );
            break;
        }
        let dx = boss_position.x_m - position.x_m;
        let dz = boss_position.z_m - position.z_m;
        let d = (dx * dx + dz * dz).sqrt().max(0.01);
        let target = escape.unwrap_or(
            Vec3::new(
                boss_position.x_m - dx / d * 1.0,
                0.0,
                boss_position.z_m - dz / d * 1.0,
            )
            .unwrap(),
        );
        let mx = target.x_m - position.x_m;
        let mz = target.z_m - position.z_m;
        let md = (mx * mx + mz * mz).sqrt();
        let moving = md > 0.1;
        let mut input = InputSample::new(
            epoch,
            seq,
            seq * 17,
            if moving { mx / md } else { 0.0 },
            if moving { mz / md } else { 0.0 },
        )
        .unwrap();
        input.aim_x = dx / d;
        input.aim_z = dz / d;
        // The input-only replay seam schedules the real owner. The test queues
        // the exact Action-v2 combat intent at that same trusted mailbox boundary;
        // it makes no claim about native IPC action transport or wall-clock FPS.
        if seq % 30 == 1 && d < 2.0 {
            runtime
                .state
                .lock()
                .unwrap()
                .pending_combat
                .push(crate::continuous_combat::CombatIntent::ActionAttack { request_id: seq });
        }
        runtime.submit_input(input, vec![]).unwrap();
    }
    assert!(finished, "sound-driven ordinary combat never completed");
    assert!(decoys > 0 && strikes > 0 && pulses > 0);
    assert!(runtime.snapshot().unwrap().player.current_hp > 0);
    assert_eq!(
        runtime
            .state
            .lock()
            .unwrap()
            .world_persistent_v1
            .mist_harbor
            .warden_defeated,
        true
    );
}
#[test]
fn actual_combat_guard_and_dash_remain_effective_against_the_committed_attack() {
    fn impact(defense: &str) -> u32 {
        let (runtime, _) = fixture();
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(16.0, 0.0, 8.0).unwrap();
        for _ in 0..3 {
            step(&runtime);
        }
        let mut used = false;
        for _ in 0..90 {
            {
                let mut s = runtime.state.lock().unwrap();
                let remaining = s.world.generic_actors[0]
                    .warden
                    .as_ref()
                    .unwrap()
                    .remaining_ms;
                if !used
                    && ((defense == "guard" && remaining <= 180)
                        || (defense == "dash" && remaining <= 180))
                {
                    // Dash avoids this committed cone by displacement; the
                    // existing action has no newly invented invincibility grant.
                    s.latest_sample.aim_x = 0.0;
                    s.latest_sample.aim_z = 1.0;
                    s.pending_combat.push(if defense == "guard" {
                        crate::continuous_combat::CombatIntent::GuardStart { request_id: 1 }
                    } else {
                        crate::continuous_combat::CombatIntent::ActionDash { request_id: 1 }
                    });
                    used = true;
                }
            }
            step(&runtime);
            let s = runtime.state.lock().unwrap();
            if s.presentation_events
                .iter()
                .any(|e| e.kind == "ResonanceWardenStrikeImpact")
            {
                return s.world.player_hp;
            }
        }
        panic!("no impact");
    }
    let normal = impact("none");
    let guard = impact("guard");
    let dash = impact("dash");
    assert_eq!(normal, 86);
    assert!(guard > normal && guard <= 100);
    assert_eq!(dash, 100);
}
