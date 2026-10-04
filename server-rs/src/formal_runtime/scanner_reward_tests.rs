use super::*;
use std::sync::atomic::AtomicU64;
static SERIAL: AtomicU64 = AtomicU64::new(1);
const SCANNER_ID: &str = "gh_log_shaft_01";

fn runtime(powered: bool) -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("scanner-reward-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::SeqCst)));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    let registry = runtime.scene_registry.lock().unwrap().as_ref().unwrap().clone();
    runtime.install_scene_registry(registry, "gh_central_shaft").unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = Vec3 { x_m: 3.0, y_m: 0.0, z_m: 8.0 };
        let revision = state.world.revision;
        assert!(matches!(apply_route_command(&mut state.route, RouteCommand::Enter {
            world_id: "grey_hive".into(), request_id: "scanner-enter".into(),
        }, revision), RouteResult::Applied { .. }));
        if powered { assert!(matches!(apply_route_command(&mut state.route, RouteCommand::Progress {
            event_id: "hive_power".into(), request_id: "scanner-power".into(),
        }, revision), RouteResult::Applied { .. })); }
    }
    runtime
}
fn has_scanner(view: &WorldView) -> bool {
    view.capabilities.items.iter().any(|item| item.capability_id == CAP_ENEMY_VITALS && item.granted)
}
fn interact(runtime: &FormalRuntime, request: &str) -> Result<FormalInteractionResponse, String> {
    runtime.activate_scene_interaction(SCANNER_ID, request, runtime.snapshot().unwrap().world_epoch)
}
fn old_activation(runtime: &FormalRuntime) {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    runtime.scene_runtime.lock().unwrap().as_mut().unwrap().interact(SCANNER_ID, "old-log", epoch,
        Vec3 { x_m: 3.0, y_m: 0.0, z_m: 8.0 }).unwrap();
}
fn bytes(runtime: &FormalRuntime) -> Vec<u8> {
    std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap()
}
fn assert_unchanged(runtime: &FormalRuntime, before: &WorldView, file: &[u8]) {
    assert_eq!(&runtime.snapshot().unwrap(), before);
    assert_eq!(bytes(runtime), file);
}

#[test]
fn scanner_formal_f_grants_once_preserves_facility_event_and_atomically_saves() {
    let runtime = runtime(true);
    let before = runtime.snapshot().unwrap();
    assert!(!has_scanner(&before));
    assert!(before.capabilities.enemy_vitals.is_empty());
    let first = interact(&runtime, "scanner-f").unwrap();
    assert!(first.applied && !first.already_applied);
    assert!(has_scanner(&first.view));
    assert_eq!(first.events, vec!["Interaction { id: \"gh_log_shaft_01\", event_id: None }"]);
    assert_eq!(first.view.progression, before.progression);
    let saved = crate::save_v6::read_save(&runtime.save_root).unwrap();
    assert_eq!(saved.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_ENEMY_VITALS).count(), 1);
    assert!(saved.save.capabilities.selected.is_empty());
    assert!(saved.save.scene_states[0].activated_ids.contains(&SCANNER_ID.to_string()));
    assert!(saved.resolve_rules().unwrap().capability_permissions.contains(&CapabilityPermission::EnemyVitalsBasic));
    assert!(first.view.interactables.iter().find(|i| i.entity_id == SCANNER_ID).unwrap().active);
    let file = bytes(&runtime);
    for request in ["scanner-f", "scanner-f-again"] {
        let repeat = interact(&runtime, request).unwrap();
        assert!(!repeat.applied && repeat.already_applied);
        assert!(repeat.events.is_empty());
        assert_unchanged(&runtime, &first.view, &file);
    }
    let raw = String::from_utf8(file).unwrap();
    assert!(!raw.contains("scannerUnlocked"));
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_without_power_rejects_even_when_standing_at_authored_log() {
    let runtime = runtime(false);
    runtime.save().unwrap();
    let before = runtime.snapshot().unwrap(); let file = bytes(&runtime);
    let rejected = interact(&runtime, "no-power").unwrap();
    assert!(!rejected.applied && !rejected.already_applied);
    assert_eq!(rejected.error_code.as_deref(), Some("E_SCANNER_POWER_REQUIRED"));
    assert_unchanged(&runtime, &before, &file);
    assert!(!runtime.scene_runtime.lock().unwrap().as_ref().unwrap().object_activated(SCANNER_ID));
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_explicit_continue_backfills_exact_powered_old_log_but_probes_are_read_only() {
    let runtime = runtime(true); old_activation(&runtime); runtime.save().unwrap();
    let before = runtime.snapshot().unwrap(); let file = bytes(&runtime);
    for _ in 0..3 { assert!(runtime.has_save()); assert!(runtime.has_default_save()); runtime.list_save_slots(); }
    assert_unchanged(&runtime, &before, &file);
    assert!(!has_scanner(&before));
    let continued = runtime.continue_saved().unwrap();
    assert!(has_scanner(&continued));
    let persisted = crate::save_v6::read_save(&runtime.save_root).unwrap();
    assert_eq!(persisted.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_ENEMY_VITALS).count(), 1);
    let migrated_file = bytes(&runtime);
    assert_ne!(file, migrated_file);
    for _ in 0..2 { assert!(has_scanner(&runtime.continue_saved().unwrap())); assert_eq!(bytes(&runtime), migrated_file); }
    entry_test_support::acknowledge_ready(&runtime, runtime.snapshot().unwrap());
    assert!(interact(&runtime, "online-after-continue").unwrap().already_applied);
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[cfg(unix)]
#[test]
fn scanner_save_failure_rolls_back_grant_activation_receipt_and_all_file_bytes() {
    use std::os::unix::fs::PermissionsExt;
    let runtime = runtime(true); runtime.save().unwrap();
    let before = runtime.snapshot().unwrap(); let file = bytes(&runtime);
    std::fs::set_permissions(&runtime.save_root, std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = interact(&runtime, "retry-same-ticket");
    std::fs::set_permissions(&runtime.save_root, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.unwrap_err().starts_with("E_SAVE_"));
    assert_unchanged(&runtime, &before, &file);
    assert!(!runtime.scene_runtime.lock().unwrap().as_ref().unwrap().object_activated(SCANNER_ID));
    assert!(has_scanner(&interact(&runtime, "retry-same-ticket").unwrap().view));
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_rejects_stale_far_dead_paused_loading_wrong_world_or_scene_without_mutation() {
    let runtime = runtime(true); runtime.save().unwrap();
    let base = runtime.state.lock().unwrap().clone(); let file = bytes(&runtime);
    for case in 0..9 {
        {
            let mut state = runtime.state.lock().unwrap(); *state = base.clone();
            match case {
                0 => state.world.player.position_m.x_m = 5.501,
                1 => state.world.player.position_m.y_m = 2.501,
                2 => state.world.player_hp = 0,
                3 => state.paused = true,
                4 => state.pending_entry = Some(crate::world_v3::SceneEntryToken { generation: 1,
                    world_id: "grey_hive".into(), scene_id: "gh_central_shaft".into(), world_epoch: 1 }),
                5 => state.world.world_id = "mist_harbor".into(),
                6 => state.world.scene_id = "gh_sentinel_arena".into(),
                7 => state.route.current_world_id = "return_station".into(),
                _ => (),
            }
        }
        let before = runtime.snapshot().unwrap();
        let epoch = before.world_epoch + if case == 8 { 1 } else { 0 };
        let result = runtime.activate_scene_interaction(SCANNER_ID, &format!("reject-{case}"), epoch);
        assert!(result.is_err() || result.as_ref().is_ok_and(|r| !r.applied && !r.already_applied), "case {case}");
        assert_unchanged(&runtime, &before, &file);
        assert!(!runtime.scene_runtime.lock().unwrap().as_ref().unwrap().object_activated(SCANNER_ID));
    }
    *runtime.state.lock().unwrap() = base;
    assert!(!runtime.activate_scene_interaction(SCANNER_ID, "", 1).unwrap().applied);
    assert!(!runtime.activate_scene_interaction("gh_sys_sentinel_01", "other-log", 1).unwrap().applied);
    assert!(!has_scanner(&runtime.snapshot().unwrap()));
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_exact_range_and_existing_grant_preserve_selection_and_log_semantics() {
    let runtime = runtime(true);
    runtime.state.lock().unwrap().world.player.position_m = Vec3 { x_m: 0.5, y_m: 0.0, z_m: 8.0 };
    assert!(interact(&runtime, "edge-2_5m").unwrap().applied);
    let registry = runtime.scene_registry.lock().unwrap().as_ref().unwrap().clone();
    runtime.install_scene_registry(registry, "gh_central_shaft").unwrap();
    runtime.state.lock().unwrap().world.player.position_m = Vec3 { x_m: 3.0, y_m: 0.0, z_m: 8.0 };
    let view = interact(&runtime, "new-visit-log").unwrap();
    assert!(view.applied);
    assert_eq!(view.events.len(), 1, "new visit retains its original facility event even if module is online");
    let saved = crate::save_v6::read_save(&runtime.save_root).unwrap();
    assert_eq!(saved.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_ENEMY_VITALS).count(), 1);
    assert!(saved.save.capabilities.selected.is_empty());
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_old_save_requires_both_exact_activation_and_power() {
    for (powered, activated) in [(true, false), (false, true), (false, false)] {
        let runtime = runtime(powered); if activated { old_activation(&runtime); }
        runtime.save().unwrap(); let file = bytes(&runtime);
        assert!(runtime.has_default_save());
        assert!(!has_scanner(&runtime.continue_saved().unwrap()));
        assert_eq!(bytes(&runtime), file);
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn scanner_other_facility_logs_and_other_scene_history_do_not_award() {
    let runtime = runtime(true);
    let registry = runtime.scene_registry.lock().unwrap().as_ref().unwrap().clone();
    runtime.install_scene_registry(registry, SENTINEL_SCENE_ID).unwrap();
    let point = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene().interactions.iter()
        .find(|i| i.id == "gh_sys_sentinel_01").unwrap().position;
    runtime.state.lock().unwrap().world.player.position_m = vec3_from_array(point);
    let response = runtime.activate_scene_interaction("gh_sys_sentinel_01", "other-facility", runtime.snapshot().unwrap().world_epoch).unwrap();
    assert!(response.applied); assert!(!has_scanner(&response.view));
    runtime.save().unwrap(); let file = bytes(&runtime);
    assert!(!has_scanner(&runtime.continue_saved().unwrap())); assert_eq!(bytes(&runtime), file);
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[cfg(unix)]
#[test]
fn scanner_continue_backfill_save_failure_never_installs_candidate() {
    use std::os::unix::fs::PermissionsExt;
    let runtime = runtime(true); old_activation(&runtime); runtime.save().unwrap();
    let before = runtime.snapshot().unwrap(); let file = bytes(&runtime);
    std::fs::set_permissions(&runtime.save_root, std::fs::Permissions::from_mode(0o555)).unwrap();
    assert!(runtime.has_default_save(), "read-only probe must not attempt the write");
    let result = runtime.continue_saved();
    std::fs::set_permissions(&runtime.save_root, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.unwrap_err().starts_with("E_SAVE_")); assert_unchanged(&runtime, &before, &file);
    assert!(has_scanner(&runtime.continue_saved().unwrap()));
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_v5_default_migration_and_slot_continue_commit_only_complete_reward() {
    for slot in [false, true] {
        let runtime = runtime(true); old_activation(&runtime);
        let saved = crate::save_v6::SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap();
        let old_path = if slot {
            crate::save_slots::create_slot(&runtime.save_root, "scanner", "Old Scanner", &saved.save).unwrap();
            runtime.save_root.join("slots/scanner/slot-v5.json")
        } else {
            crate::save_v5::write_save(&runtime.save_root, &saved.save).unwrap();
            crate::save_v5::save_path(&runtime.save_root)
        };
        let old_file = std::fs::read(&old_path).unwrap();
        let before = runtime.snapshot().unwrap();
        for _ in 0..2 { assert!(runtime.has_save()); runtime.list_save_slots(); }
        assert_eq!(runtime.snapshot().unwrap(), before);
        assert!(!crate::save_v6::save_path(&runtime.save_root).exists());
        assert!(!runtime.save_root.join("slots/scanner/slot-v6.json").exists());
        let view = if slot { runtime.continue_slot("scanner") } else { runtime.continue_saved() }.unwrap();
        assert!(has_scanner(&view)); assert_eq!(std::fs::read(old_path).unwrap(), old_file);
        let restored = if slot { crate::save_slots::read_slot_v6(&runtime.save_root, "scanner").unwrap().1 }
            else { crate::save_v6::read_save(&runtime.save_root).unwrap() };
        assert_eq!(restored.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_ENEMY_VITALS).count(), 1);
        assert!(restored.save.scene_states[0].activated_ids.contains(&SCANNER_ID.to_string()));
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn scanner_corrupt_preferred_default_or_slot_never_falls_back_or_grants() {
    for slot in [false, true] {
        let runtime = runtime(true); old_activation(&runtime);
        let saved = crate::save_v6::SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap();
        let target = if slot {
            crate::save_slots::create_slot(&runtime.save_root, "scanner", "Old Scanner", &saved.save).unwrap();
            runtime.save_root.join("slots/scanner/slot-v6.json")
        } else {
            crate::save_v5::write_save(&runtime.save_root, &saved.save).unwrap();
            crate::save_v6::save_path(&runtime.save_root)
        };
        std::fs::write(&target, b"broken newer save").unwrap();
        let before = runtime.snapshot().unwrap();
        if slot { assert!(!runtime.list_save_slots().iter().any(|s| s.valid)); assert!(runtime.continue_slot("scanner").is_err()); }
        else { assert!(!runtime.has_default_save()); assert!(runtime.continue_saved().is_err()); }
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(std::fs::read(target).unwrap(), b"broken newer save");
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn scanner_real_transition_revisit_and_continue_preserve_grant_and_tiers_without_hp() {
    let runtime = runtime(true); interact(&runtime, "scanner-first").unwrap();
    runtime.state.lock().unwrap().world.player.position_m = Vec3 { x_m: 1.0, y_m: 0.0, z_m: 8.0 };
    let epoch = runtime.snapshot().unwrap().world_epoch;
    let gate = entry_test_support::acknowledge_ready(&runtime, runtime.transition_scene("gh_shaft_return_to_gate_a", "scanner-to-gate", epoch).unwrap());
    assert!(has_scanner(&gate));
    runtime.state.lock().unwrap().world.player.position_m = Vec3 { x_m: 18.0, y_m: 0.0, z_m: 7.0 };
    let returned = entry_test_support::acknowledge_ready(&runtime, runtime.transition_scene("gh_gate_a_to_shaft", "scanner-return", gate.world_epoch).unwrap());
    assert!(has_scanner(&returned)); runtime.save().unwrap();
    let continued = entry_test_support::acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert!(has_scanner(&continued));
    for (hp, tier) in [(50, "healthy"), (49, "wounded"), (31, "wounded"), (30, "severelyWounded"), (13, "severelyWounded"), (12, "critical"), (1, "critical"), (0, "dead")] {
        let mut state = runtime.state.lock().unwrap();
        let actor = &mut state.world.generic_actors[0]; actor.hp = hp;
        let id = actor.entity_id.clone();
        let projected = serde_json::to_value(project_view(&state).capabilities).unwrap();
        let row = projected["enemyVitals"].as_array().unwrap().iter().find(|r| r["entityId"] == id);
        if hp == 0 { assert!(row.is_none()); } else {
            let row = row.unwrap(); assert_eq!(row["tier"], tier);
            assert_eq!(row.as_object().unwrap().keys().map(String::as_str).collect::<BTreeSet<_>>(), BTreeSet::from(["entityId", "tier"]));
        }
    }
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[cfg(unix)]
#[test]
fn scanner_named_slot_f_and_continue_backfill_are_atomic_and_preserve_other_saves() {
    use std::os::unix::fs::PermissionsExt;
    for legacy_activation in [false, true] {
        let runtime = runtime(true); if legacy_activation { old_activation(&runtime); }
        let saved = crate::save_v6::SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap();
        crate::save_slots::create_slot_v6(&runtime.save_root, "scanner", "Scanner slot", &saved).unwrap();
        runtime.save().unwrap(); let default_bytes = bytes(&runtime);
        let target = runtime.save_root.join("slots/scanner/slot-v6.json");
        let old_slot = std::fs::read(&target).unwrap();
        if !legacy_activation { entry_test_support::acknowledge_ready(&runtime, runtime.continue_slot("scanner").unwrap()); }
        let before = runtime.snapshot().unwrap();
        let parent = target.parent().unwrap();
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o555)).unwrap();
        let result = if legacy_activation { runtime.continue_slot("scanner").map(|view| has_scanner(&view)) }
            else { interact(&runtime, "slot-f").map(|result| has_scanner(&result.view)) };
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.unwrap_err().starts_with("E_SLOT_"));
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(std::fs::read(&target).unwrap(), old_slot);
        assert_eq!(bytes(&runtime), default_bytes);
        if legacy_activation { assert!(has_scanner(&runtime.continue_slot("scanner").unwrap())); }
        else { assert!(has_scanner(&interact(&runtime, "slot-f").unwrap().view)); }
        assert!(has_scanner(&runtime.continue_slot("scanner").unwrap()));
        assert_eq!(bytes(&runtime), default_bytes);
        // Explicit default Continue replaces the active destination; future F saves there only.
        let default = entry_test_support::acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        if !legacy_activation {
            assert!(!has_scanner(&default)); let committed_slot = std::fs::read(&target).unwrap();
            assert!(has_scanner(&interact(&runtime, "default-f").unwrap().view));
            assert_eq!(std::fs::read(&target).unwrap(), committed_slot);
        }
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn scanner_old_unknown_activation_or_invalid_geometry_fails_closed_without_repair() {
    for wrong_activation in [false, true] {
        let runtime = runtime(true); old_activation(&runtime); runtime.save().unwrap();
        let path = crate::save_v6::save_path(&runtime.save_root);
        let mut saved: serde_json::Value = serde_json::from_slice(&bytes(&runtime)).unwrap();
        if wrong_activation { saved["save"]["sceneStates"][0]["activatedIds"] = serde_json::json!(["gh_log_other"]); }
        else { saved["save"]["player"]["positionM"]["xM"] = serde_json::json!(999.0); }
        std::fs::write(&path, serde_json::to_vec_pretty(&saved).unwrap()).unwrap();
        let before = runtime.snapshot().unwrap(); let file = bytes(&runtime);
        assert!(!runtime.has_default_save()); assert!(runtime.continue_saved().is_err());
        assert_unchanged(&runtime, &before, &file);
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn scanner_metadata_never_falls_back_through_nonregular_newer_slot() {
    let runtime = runtime(true); old_activation(&runtime);
    let saved = crate::save_v6::SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap();
    crate::save_slots::create_slot(&runtime.save_root, "scanner", "Scanner slot", &saved.save).unwrap();
    let path = runtime.save_root.join("slots/scanner/slot-v6.json");
    std::fs::create_dir(&path).unwrap();
    let before = runtime.snapshot().unwrap();
    assert!(!runtime.has_save()); assert!(!runtime.list_save_slots().iter().any(|s| s.valid));
    assert!(runtime.continue_slot("scanner").is_err());
    assert_eq!(runtime.snapshot().unwrap(), before); assert!(path.is_dir());
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn scanner_acquired_by_f_survives_world_change_without_revealing_unmapped_wraith_position() {
    let runtime = runtime(true); interact(&runtime, "scanner-real-reward").unwrap();
    let registry = runtime.scene_registry.lock().unwrap().as_ref().unwrap().clone();
    runtime.install_scene_registry(registry, "mh_signal_yard").unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        state.route.current_world_id = "mist_harbor".into();
        state.route.progress.iter_mut().find(|p| p.world_id == "mist_harbor").unwrap().visit_id = 1;
        state.world.player.position_m = Vec3 { x_m: 11.0, y_m: 0.0, z_m: 6.0 };
    }
    for continued in [false, true] {
        let view = if continued {
            runtime.save().unwrap(); entry_test_support::acknowledge_ready(&runtime, runtime.continue_saved().unwrap())
        } else { runtime.snapshot().unwrap() };
        assert!(has_scanner(&view));
        let id = "mh_signal_yard_signal_wraith_01";
        let actual = runtime.state.lock().unwrap().world.generic_actors.iter().find(|a| a.entity_id == id).unwrap().position_m;
        let row = view.actors.iter().find(|a| a.entity_id == id).unwrap();
        let perception = row.signal_perception.as_ref().unwrap();
        assert!(!perception.precise); assert_eq!(perception.positions_m.len(), 2);
        assert_eq!(row.transform.position_m, signal_wraith_presentation::anchor(perception));
        assert_ne!(row.transform.position_m, actual);
        let vitals = serde_json::to_value(&view.capabilities.enemy_vitals).unwrap();
        let vital = vitals.as_array().unwrap().iter().find(|row| row["entityId"] == id).unwrap();
        assert_eq!(vital.as_object().unwrap().keys().map(String::as_str).collect::<BTreeSet<_>>(), BTreeSet::from(["entityId", "tier"]));
        let state = runtime.state.lock().unwrap();
        assert!(state.effective_rules_v6.capability_permissions.contains(&CapabilityPermission::EnemyVitalsBasic));
        assert!(!state.effective_rules_v6.capability_permissions.contains(&CapabilityPermission::AcousticMapping));
    }
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

fn recovery_evidence(root: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn visit(root: &std::path::Path, at: &std::path::Path, result: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() { let path=entry.unwrap().path(); if path.is_dir() {visit(root,&path,result);} else {result.insert(path.strip_prefix(root).unwrap().to_string_lossy().into_owned(),std::fs::read(path).unwrap());} }
    }
    let mut result=std::collections::BTreeMap::new();visit(root,root,&mut result);result
}
fn recovery_gap(runtime: &FormalRuntime, named: bool) -> std::path::PathBuf {
    let path=if named {runtime.save_root.join("slots/recover/slot-v6.json")}else{crate::save_v6::save_path(&runtime.save_root)};
    let name=path.file_name().unwrap().to_str().unwrap();
    std::fs::write(path.with_file_name(format!(".{name}.424242.1.tmp")),b"uncommitted tmp bytes").unwrap();
    std::fs::rename(&path,path.with_file_name(format!(".{name}.424242.1.bak"))).unwrap();path
}
#[test]
fn scanner_recovery_and_backfill_share_one_commit_and_repeat_without_new_grants() {
    for named in [false,true] { for activated in [false,true] {
        let runtime=runtime(true);if activated {old_activation(&runtime);}
        if named {runtime.save_slot("recover","Recovery",true).unwrap();}else{runtime.save().unwrap();}
        runtime.save_slot("previous","Previous",true).unwrap();
        let target=recovery_gap(&runtime,named);let before=recovery_evidence(&runtime.save_root);let world=runtime.snapshot().unwrap();
        for _ in 0..3 {assert!(runtime.has_save());runtime.list_save_slots();assert_eq!(runtime.snapshot().unwrap(),world);assert_eq!(*runtime.active_slot.lock().unwrap(),Some("previous".into()));assert_eq!(recovery_evidence(&runtime.save_root),before);}
        let commits=crate::save_v6::RECOVERY_TEST_COMMITS.with(|n|n.get());
        let view=if named {runtime.continue_slot("recover")}else{runtime.continue_saved()}.unwrap();
        assert_eq!(has_scanner(&view),activated);assert_eq!(crate::save_v6::RECOVERY_TEST_COMMITS.with(|n|n.get()),commits+1);
        assert_eq!(*runtime.active_slot.lock().unwrap(),if named {Some("recover".into())}else{None});
        let after=recovery_evidence(&runtime.save_root);assert_eq!(after.len(),before.len()+1);for(key,value)in before {assert_eq!(after.get(&key),Some(&value));}
        assert!(target.exists());let persisted=if named{crate::save_slots::read_slot_v6(&runtime.save_root,"recover").unwrap().1}else{crate::save_v6::read_save(&runtime.save_root).unwrap()};
        assert_eq!(persisted.save.capabilities.grants.iter().filter(|g|g.capability_id==CAP_ENEMY_VITALS).count(),usize::from(activated));
        for _ in 0..2 {if named {runtime.continue_slot("recover").unwrap();}else{runtime.continue_saved().unwrap();}assert_eq!(recovery_evidence(&runtime.save_root),after);}
        assert_eq!(crate::save_v6::RECOVERY_TEST_COMMITS.with(|n|n.get()),commits+1);std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }}
}
#[test]
fn scanner_recovery_faults_leave_canonical_absent_all_evidence_and_active_slot_unchanged() {
    for named in [false,true] {for stage in ["prepared","commit","postwrite","verified"] {
        let runtime=runtime(true);old_activation(&runtime);
        if named {runtime.save_slot("recover","Recovery",true).unwrap();}else{runtime.save().unwrap();}
        runtime.save_slot("previous","Previous",true).unwrap();let target=recovery_gap(&runtime,named);
        let before=recovery_evidence(&runtime.save_root);let world=runtime.snapshot().unwrap();let events=runtime.presentation_events_since(world.world_epoch,0).unwrap();
        crate::save_v6::RECOVERY_TEST_FAULT.with(|fault|fault.set(Some(stage)));
        let result=if named {runtime.continue_slot("recover")}else{runtime.continue_saved()};
        crate::save_v6::RECOVERY_TEST_FAULT.with(|fault|fault.set(None));
        assert!(result.unwrap_err().contains("INJECTED"));assert!(!target.exists());assert_eq!(recovery_evidence(&runtime.save_root),before);assert_eq!(runtime.snapshot().unwrap(),world);assert_eq!(runtime.presentation_events_since(world.world_epoch,0).unwrap(),events);assert_eq!(*runtime.active_slot.lock().unwrap(),Some("previous".into()));assert!(!has_scanner(&runtime.snapshot().unwrap()));
        assert!(has_scanner(&if named {runtime.continue_slot("recover")}else{runtime.continue_saved()}.unwrap()));
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }}
}

#[test]
fn recovery_rechecks_the_source_even_if_canonical_appears_before_continue_commit() {
    for route in ["default","alias","named"] {
        let runtime=runtime(true);let named=route=="named";
        if named {runtime.save_slot("recover","Recovery",true).unwrap();}else{runtime.save().unwrap();}
        runtime.save_slot("previous","Previous",true).unwrap();let target=recovery_gap(&runtime,named);
        let before=recovery_evidence(&runtime.save_root);let world=runtime.snapshot().unwrap();let publish=target.clone();
        crate::save_v6::BEFORE_RECOVERY_CONTINUE.with(|hook|*hook.borrow_mut()=Some(Box::new(move||{std::fs::write(publish,b"concurrent canonical must survive").unwrap();})));
        let result=match route{"default"=>runtime.continue_saved(),"alias"=>runtime.continue_slot("legacy-save-v3"),_=>runtime.continue_slot("recover")};
        assert!(result.unwrap_err().contains("RECOVERY_SOURCE_CHANGED"));assert_eq!(runtime.snapshot().unwrap(),world);assert_eq!(*runtime.active_slot.lock().unwrap(),Some("previous".into()));
        assert_eq!(std::fs::read(&target).unwrap(),b"concurrent canonical must survive");let after=recovery_evidence(&runtime.save_root);assert_eq!(after.len(),before.len()+1);for(key,value)in before{assert_eq!(after.get(&key),Some(&value));}
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}
