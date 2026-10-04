use super::*;
use crate::{save_slots, save_v6::SaveV6};
use std::{fs, sync::atomic::AtomicU64, time::{SystemTime, UNIX_EPOCH}};

static SERIAL: AtomicU64 = AtomicU64::new(1);
const REST: &str = "rs_save_rest_terminal_marker";
const SCANNER: &str = "gh_log_shaft_01";
#[derive(Clone, Copy, Debug)]
enum Operation { Rest, Evolution, Scanner }
const OPERATIONS: [Operation; 3] = [Operation::Rest, Operation::Evolution, Operation::Scanner];

fn runtime(operation: Operation) -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("reward-save-target-{}-{}-{}", std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(), SERIAL.fetch_add(1, Ordering::SeqCst)));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    entry_test_support::acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    if matches!(operation, Operation::Scanner) {
        let registry = runtime.scene_registry.lock().unwrap().as_ref().unwrap().clone();
        runtime.install_scene_registry(registry, "gh_central_shaft").unwrap();
    }
    {
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = match operation {
            Operation::Rest => Vec3 { x_m: 8.0, y_m: 0.0, z_m: 12.0 },
            Operation::Evolution => Vec3 { x_m: 15.0, y_m: 0.0, z_m: 4.0 },
            Operation::Scanner => Vec3 { x_m: 3.0, y_m: 0.0, z_m: 8.0 },
        };
        state.world.player_hp = 37;
        state.world.player_energy = 21;
        match operation {
            Operation::Evolution => {
                let gh = state.route.progress.iter_mut().find(|p| p.world_id == "grey_hive").unwrap();
                gh.completed = true; gh.first_completion = true;
                gh.completed_events = vec!["hive_power".into(), "hive_lockdown".into(), "hive_extraction".into()];
                state.world_persistent_v1.grey_hive.beacon = None;
                // Existing Acoustic Mapping must retain its historical route authority.
                state.route.progress.iter_mut().find(|p| p.world_id == "mist_harbor").unwrap()
                    .completed_events.push("mist_signal".into());
                let mut capabilities = state.capabilities.clone();
                let effects = apply_command_at_revision(&mut capabilities,
                    CapabilityCommand::Grant { capability_id: CAP_ACOUSTIC_MAPPING.into() }, state.world.revision).unwrap();
                let mut world = state.world.clone();
                if !effects.is_empty() { apply_effects_atomically(&mut world, &effects).unwrap(); }
                state.install_capability_state(world, capabilities).unwrap();
            }
            Operation::Scanner => {
                let revision = state.world.revision;
                assert!(matches!(apply_route_command(&mut state.route, RouteCommand::Enter {
                    world_id: "grey_hive".into(), request_id: "target-enter".into() }, revision), RouteResult::Applied { .. }));
                assert!(matches!(apply_route_command(&mut state.route, RouteCommand::Progress {
                    event_id: "hive_power".into(), request_id: "target-power".into() }, revision), RouteResult::Applied { .. }));
            }
            Operation::Rest => (),
        }
    }
    if matches!(operation, Operation::Evolution) {
        runtime.apply_capability_command(CapabilityCommandRequest::Select {
            capability_ids: vec![CAP_ACOUSTIC_MAPPING.into()] }).unwrap();
    }
    runtime
}

fn capture(runtime: &FormalRuntime) -> SaveV6 {
    SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap()
}
fn named_path(runtime: &FormalRuntime) -> PathBuf { runtime.save_root.join("slots/manual/slot-v6.json") }
fn default_path(runtime: &FormalRuntime) -> PathBuf { crate::save_v6::save_path(&runtime.save_root) }
fn scene_state(runtime: &FormalRuntime) -> String { format!("{:?}", runtime.scene_runtime.lock().unwrap().as_ref().unwrap()) }

fn prepare(runtime: &FormalRuntime, operation: Operation) -> Option<EnhancementTerminalContext> {
    if !matches!(operation, Operation::Evolution) { return None; }
    let view = runtime.snapshot().unwrap();
    let context = EnhancementTerminalContext { id: "rs_capability_terminal_marker".into(), request_id: "target-evolution".into(),
        world_epoch: view.world_epoch, pause_command_sequence: 1 };
    runtime.capability_terminal_status(&context.id, &context.request_id, context.world_epoch).unwrap();
    runtime.pause_context_ordered(&SessionContext { world_id: view.world_id, scene_id: view.scene_id, world_epoch: view.world_epoch }, 1).unwrap();
    Some(context)
}

fn apply(runtime: &FormalRuntime, operation: Operation, context: Option<&EnhancementTerminalContext>) -> Result<WorldView, String> {
    let epoch = runtime.snapshot().unwrap().world_epoch;
    match operation {
        Operation::Rest => runtime.save_rest_terminal(REST, "target-rest", epoch),
        Operation::Evolution => runtime.choose_first_enhancement(CAP_LOCAL_MAP, context.unwrap()),
        Operation::Scanner => runtime.activate_scene_interaction(SCANNER, "target-scanner", epoch).map(|r| {
            assert!(r.applied || r.already_applied, "{:?}", r.error_code); r.view
        }),
    }
}

fn assert_effect(operation: Operation, view: &WorldView, save: &SaveV6) {
    match operation {
        Operation::Rest => {
            assert_eq!(view.player.current_hp, view.player.max_hp);
            assert_eq!(view.player.current_energy, view.player.max_energy);
            assert_eq!(save.save.player.current_hp, view.player.max_hp);
            assert_eq!(save.save.player.current_energy, view.player.max_energy);
        }
        Operation::Evolution => {
            assert_eq!(view.capabilities.first_enhancement_choice.as_deref(), Some(CAP_LOCAL_MAP));
            assert_eq!(save.save.capabilities.first_enhancement_choice.as_deref(), Some(CAP_LOCAL_MAP));
            assert_eq!(save.save.capabilities.selected, [CAP_ACOUSTIC_MAPPING, CAP_LOCAL_MAP]);
            assert_eq!(save.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_LOCAL_MAP).count(), 1);
        }
        Operation::Scanner => {
            assert!(view.capabilities.items.iter().any(|item| item.capability_id == CAP_ENEMY_VITALS && item.granted));
            assert_eq!(save.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_ENEMY_VITALS).count(), 1);
            assert!(save.save.scene_states[0].activated_ids.contains(&SCANNER.into()));
        }
    }
}

#[test]
fn rewards_write_only_active_named_slot_and_ignore_newer_default() {
    for operation in OPERATIONS {
        let runtime = runtime(operation);
        runtime.save_slot("manual", "Chosen manual slot", true).unwrap();
        runtime.save().unwrap(); // A later default never overrides the explicit destination.
        let original_default = fs::read(default_path(&runtime)).unwrap();
        let original_slot = fs::read(named_path(&runtime)).unwrap();
        let context = prepare(&runtime, operation);
        let view = apply(&runtime, operation, context.as_ref()).unwrap();
        let (name, saved) = save_slots::read_slot_v6(&runtime.save_root, "manual").unwrap();
        assert_eq!(name, "Chosen manual slot");
        assert_effect(operation, &view, &saved);
        assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
        assert_ne!(fs::read(named_path(&runtime)).unwrap(), original_slot);
        assert_eq!(*runtime.active_slot.lock().unwrap(), Some("manual".into()));
        let committed = fs::read(named_path(&runtime)).unwrap();
        match operation {
            Operation::Rest => assert_eq!(apply(&runtime, operation, context.as_ref()).unwrap_err(), "E_REST_TERMINAL_DuplicateRequest"),
            Operation::Evolution => assert!(apply(&runtime, operation, context.as_ref()).unwrap_err().contains("FirstEnhancementAlreadyChosen")),
            Operation::Scanner => { assert_eq!(apply(&runtime, operation, context.as_ref()).unwrap(), view); }
        }
        assert_eq!(fs::read(named_path(&runtime)).unwrap(), committed);
        assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
        let restored = entry_test_support::acknowledge_ready(&runtime, runtime.continue_slot("manual").unwrap());
        assert_effect(operation, &restored, &saved);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn rewards_with_no_active_slot_write_default_and_leave_other_named_slots_untouched() {
    for operation in OPERATIONS {
        let runtime = runtime(operation);
        save_slots::create_slot_v6(&runtime.save_root, "manual", "Unselected slot", &capture(&runtime)).unwrap();
        runtime.save().unwrap();
        assert_eq!(*runtime.active_slot.lock().unwrap(), None);
        let original_slot = fs::read(named_path(&runtime)).unwrap();
        let context = prepare(&runtime, operation);
        let view = apply(&runtime, operation, context.as_ref()).unwrap();
        assert_effect(operation, &view, &crate::save_v6::read_save(&runtime.save_root).unwrap());
        assert_eq!(fs::read(named_path(&runtime)).unwrap(), original_slot);
        assert_eq!(*runtime.active_slot.lock().unwrap(), None);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn invalid_active_targets_fail_without_fallback_and_preserve_same_request_or_modal_ticket() {
    for operation in OPERATIONS {
        for fault in ["missing", "corrupt", "future", "v4-read-only", "directory"] {
            let runtime = runtime(operation);
            runtime.save_slot("manual", "Current slot", true).unwrap();
            runtime.save().unwrap();
            let target = named_path(&runtime);
            let original = fs::read(&target).unwrap();
            let original_default = fs::read(default_path(&runtime)).unwrap();
            fs::remove_file(&target).unwrap();
            let v4 = target.with_file_name("slot-v4.json");
            let fault_bytes = match fault {
                "corrupt" => b"broken-current-slot".to_vec(),
                "future" => {
                    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
                    value["schemaVersion"] = 999.into(); serde_json::to_vec(&value).unwrap()
                }
                "v4-read-only" => {
                    let save: serde_json::Value = serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v4.json")).unwrap();
                    serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"slotId":"manual","displayName":"Read only V4","updatedAtMs":1,"save":save})).unwrap()
                }
                _ => vec![],
            };
            match fault {
                "missing" => (),
                "directory" => fs::create_dir(&target).unwrap(),
                "v4-read-only" => fs::write(&v4, &fault_bytes).unwrap(),
                _ => fs::write(&target, &fault_bytes).unwrap(),
            }
            let context = prepare(&runtime, operation);
            let before = runtime.snapshot().unwrap();
            let scene_before = scene_state(&runtime);
            let ticket = runtime.state.lock().unwrap().enhancement_terminal.clone();
            assert!(apply(&runtime, operation, context.as_ref()).is_err(), "{operation:?}/{fault}");
            assert_eq!(runtime.snapshot().unwrap(), before, "{operation:?}/{fault}");
            assert_eq!(scene_state(&runtime), scene_before);
            assert_eq!(runtime.state.lock().unwrap().enhancement_terminal, ticket);
            assert_eq!(*runtime.active_slot.lock().unwrap(), Some("manual".into()));
            assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
            match fault {
                "missing" => assert!(!target.exists()),
                "directory" => { assert!(target.is_dir()); fs::remove_dir(&target).unwrap(); },
                "v4-read-only" => {
                    assert!(!target.exists()); assert_eq!(fs::read(&v4).unwrap(), fault_bytes);
                    assert_eq!(fs::read_dir(target.parent().unwrap()).unwrap().count(), 1);
                    fs::remove_file(&v4).unwrap();
                }
                _ => assert_eq!(fs::read(&target).unwrap(), fault_bytes),
            }
            fs::write(&target, &original).unwrap();
            let view = apply(&runtime, operation, context.as_ref()).unwrap();
            assert_effect(operation, &view, &save_slots::read_slot_v6(&runtime.save_root, "manual").unwrap().1);
            assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
            fs::remove_dir_all(&runtime.save_root).unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn active_slot_write_failure_rolls_back_each_reward_and_retries_without_default_fallback() {
    use std::os::unix::fs::PermissionsExt;
    for operation in OPERATIONS {
        let runtime = runtime(operation);
        runtime.save_slot("manual", "Current slot", true).unwrap(); runtime.save().unwrap();
        let target = named_path(&runtime); let original = fs::read(&target).unwrap();
        let original_default = fs::read(default_path(&runtime)).unwrap();
        let context = prepare(&runtime, operation);
        let before = runtime.snapshot().unwrap(); let scene_before = scene_state(&runtime);
        let ticket = runtime.state.lock().unwrap().enhancement_terminal.clone();
        fs::set_permissions(target.parent().unwrap(), fs::Permissions::from_mode(0o555)).unwrap();
        let result = apply(&runtime, operation, context.as_ref());
        fs::set_permissions(target.parent().unwrap(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.unwrap_err().starts_with("E_SLOT_TEMP"), "{operation:?}");
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(scene_state(&runtime), scene_before);
        assert_eq!(runtime.state.lock().unwrap().enhancement_terminal, ticket);
        assert_eq!(*runtime.active_slot.lock().unwrap(), Some("manual".into()));
        assert_eq!(fs::read(&target).unwrap(), original);
        assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
        let view = apply(&runtime, operation, context.as_ref()).unwrap();
        assert_effect(operation, &view, &save_slots::read_slot_v6(&runtime.save_root, "manual").unwrap().1);
        assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn active_valid_v5_only_slot_remains_writable_for_each_reward_without_intermediate_migration() {
    for operation in OPERATIONS {
        let runtime = runtime(operation);
        runtime.save_slot("manual", "Current slot", true).unwrap(); runtime.save().unwrap();
        let original_default = fs::read(default_path(&runtime)).unwrap();
        let target = named_path(&runtime);
        let save = capture(&runtime).save;
        let source = target.with_file_name("slot-v5.json");
        let original = serde_json::to_vec(&serde_json::json!({"schemaVersion":2,"slotId":"manual","displayName":"Current slot","updatedAtMs":1,"save":save})).unwrap();
        fs::write(&source, &original).unwrap(); fs::remove_file(&target).unwrap();
        let context = prepare(&runtime, operation);
        let view = apply(&runtime, operation, context.as_ref()).unwrap();
        assert_effect(operation, &view, &save_slots::read_slot_v6(&runtime.save_root, "manual").unwrap().1);
        assert_eq!(fs::read(&source).unwrap(), original);
        assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
        assert_eq!(fs::read_dir(target.parent().unwrap()).unwrap().count(), 2);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn explicit_default_continue_resets_destination_before_future_rest_or_evolution() {
    for operation in [Operation::Rest, Operation::Evolution] {
        let runtime = runtime(operation);
        runtime.save_slot("manual", "Old target", true).unwrap(); runtime.save().unwrap();
        entry_test_support::acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(*runtime.active_slot.lock().unwrap(), None);
        let original_slot = fs::read(named_path(&runtime)).unwrap();
        let context = prepare(&runtime, operation);
        let view = apply(&runtime, operation, context.as_ref()).unwrap();
        assert_effect(operation, &view, &crate::save_v6::read_save(&runtime.save_root).unwrap());
        assert_eq!(fs::read(named_path(&runtime)).unwrap(), original_slot);
        fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn read_only_winning_v5_or_v6_file_is_not_replaced_despite_writable_directory() {
    use std::os::unix::fs::PermissionsExt;
    for operation in OPERATIONS {
        for v5 in [false, true] {
            let runtime = runtime(operation);
            runtime.save_slot("manual", "Read-only intent", true).unwrap(); runtime.save().unwrap();
            let target = named_path(&runtime);
            let source = if v5 {
                let source = target.with_file_name("slot-v5.json");
                let save = capture(&runtime).save;
                let bytes = serde_json::to_vec(&serde_json::json!({"schemaVersion":2,"slotId":"manual","displayName":"Read-only intent","updatedAtMs":1,"save":save})).unwrap();
                fs::write(&source, bytes).unwrap(); fs::remove_file(&target).unwrap(); source
            } else { target.clone() };
            let original = fs::read(&source).unwrap();
            let original_default = fs::read(default_path(&runtime)).unwrap();
            let context = prepare(&runtime, operation);
            let before = runtime.snapshot().unwrap(); let scene_before = scene_state(&runtime);
            let ticket = runtime.state.lock().unwrap().enhancement_terminal.clone();
            fs::set_permissions(&source, fs::Permissions::from_mode(0o444)).unwrap();
            let result = apply(&runtime, operation, context.as_ref());
            assert!(fs::metadata(&source).unwrap().permissions().readonly());
            fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(result.unwrap_err(), "E_SLOT_READ_ONLY", "{operation:?}/V5={v5}");
            assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(scene_state(&runtime), scene_before);
            assert_eq!(runtime.state.lock().unwrap().enhancement_terminal, ticket);
            assert_eq!(*runtime.active_slot.lock().unwrap(), Some("manual".into()));
            assert_eq!(fs::read(&source).unwrap(), original);
            assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
            if v5 { assert!(!target.exists()); }
            let view = apply(&runtime, operation, context.as_ref()).unwrap();
            assert_effect(operation, &view, &save_slots::read_slot_v6(&runtime.save_root, "manual").unwrap().1);
            assert_eq!(fs::read(default_path(&runtime)).unwrap(), original_default);
            if v5 { assert_eq!(fs::read(&source).unwrap(), original); }
            fs::remove_dir_all(&runtime.save_root).unwrap();
        }
    }
}
