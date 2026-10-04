use super::*;
use std::sync::atomic::AtomicU64;
static SERIAL: AtomicU64 = AtomicU64::new(1);

fn runtime() -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("first-evolution-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::SeqCst)));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    entry_test_support::acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    {
        let mut state = runtime.state.lock().unwrap();
        state.world.player.position_m = Vec3 { x_m: 15.0, y_m: 0.0, z_m: 4.0 };
        let gh = state.route.progress.iter_mut().find(|p| p.world_id == "grey_hive").unwrap();
        gh.completed = true; gh.first_completion = true;
        gh.completed_events = vec!["hive_power".into(), "hive_lockdown".into(), "hive_extraction".into()];
        // Isolated historical completed fixture: no invented physical Beacon receipt.
        state.world_persistent_v1.grey_hive.beacon = None;
        state.route.progress.iter_mut().find(|p| p.world_id == "mist_harbor").unwrap().completed_events.push("mist_signal".into());
        state.route.progress.iter_mut().find(|p| p.world_id == "clockworks").unwrap().completed_events.push("clockworks_shutdown".into());
    }
    runtime
}
fn grant(runtime: &FormalRuntime, id: &str) {
    let mut state = runtime.state.lock().unwrap();
    let mut capabilities = state.capabilities.clone();
    let effects = apply_command_at_revision(&mut capabilities, CapabilityCommand::Grant { capability_id: id.into() }, state.world.revision).unwrap();
    let mut world = state.world.clone();
    if !effects.is_empty() { apply_effects_atomically(&mut world, &effects).unwrap(); }
    state.install_capability_state(world, capabilities).unwrap();
}
fn session(runtime: &FormalRuntime) -> SessionContext {
    let view = runtime.snapshot().unwrap();
    SessionContext { world_id: view.world_id, scene_id: view.scene_id, world_epoch: view.world_epoch }
}
fn open(runtime: &FormalRuntime, seq: u64) -> EnhancementTerminalContext {
    let ctx = EnhancementTerminalContext { id: "rs_capability_terminal_marker".into(), request_id: format!("evolution-{seq}"),
        world_epoch: runtime.snapshot().unwrap().world_epoch, pause_command_sequence: seq };
    runtime.capability_terminal_status(&ctx.id, &ctx.request_id, ctx.world_epoch).unwrap();
    runtime.pause_context_ordered(&session(runtime), seq).unwrap();
    ctx
}
fn assert_unchanged(runtime: &FormalRuntime, before: &WorldView, file: &[u8]) {
    assert_eq!(&runtime.snapshot().unwrap(), before);
    assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), file);
}

#[test]
fn first_evolution_terminal_and_confirmation_fail_closed_without_state_or_file_mutation() {
    let runtime = runtime();
    let ctx = open(&runtime, 1);
    runtime.save().unwrap();
    let before = runtime.snapshot().unwrap();
    let bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
    for bad in [EnhancementTerminalContext { id: "other".into(), ..ctx.clone() },
        EnhancementTerminalContext { request_id: "forged".into(), ..ctx.clone() },
        EnhancementTerminalContext { world_epoch: ctx.world_epoch + 1, ..ctx.clone() },
        EnhancementTerminalContext { pause_command_sequence: 0, ..ctx.clone() },
        EnhancementTerminalContext { pause_command_sequence: 2, ..ctx.clone() }] {
        assert!(runtime.choose_first_enhancement(CAP_LOCAL_MAP, &bad).is_err());
        assert_unchanged(&runtime, &before, &bytes);
    }
    assert!(runtime.choose_first_enhancement(CAP_ENEMY_VITALS, &ctx).unwrap_err().contains("IllegalFirstEnhancement"));
    assert_unchanged(&runtime, &before, &bytes);
    let base = runtime.state.lock().unwrap().clone();
    for case in 0..7 {
        {
            let mut state = runtime.state.lock().unwrap();
            *state = base.clone();
            match case {
                0 => state.world.player.position_m.x_m = 17.501,
                1 => state.world.player_hp = 0,
                2 => state.paused = false,
                3 => state.world.scene_id = "other".into(),
                4 => state.world.world_id = "grey_hive".into(),
                5 => state.pending_entry = Some(crate::world_v3::SceneEntryToken { generation: 1,
                    world_id: "return_station".into(), scene_id: "rs_core_room".into(), world_epoch: ctx.world_epoch }),
                _ => state.route.progress.iter_mut().find(|p| p.world_id == "grey_hive").unwrap().completed = false,
            }
        }
        let rejected = runtime.snapshot().unwrap();
        assert!(runtime.choose_first_enhancement(CAP_LOCAL_MAP, &ctx).is_err(), "case {case}");
        assert_unchanged(&runtime, &rejected, &bytes);
    }
    *runtime.state.lock().unwrap() = base;
    runtime.resume_context_ordered(&session(&runtime), 2).unwrap();
    runtime.pause_context_ordered(&session(&runtime), 3).unwrap();
    let stale = runtime.snapshot().unwrap();
    assert_eq!(runtime.choose_first_enhancement(CAP_LOCAL_MAP, &ctx).unwrap_err(), "E_ENHANCEMENT_TERMINAL_CONTEXT");
    assert_unchanged(&runtime, &stale, &bytes);
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}

#[test]
fn first_evolution_late_claim_preserves_selection_order_idempotent_grants_and_old_choice() {
    for already_granted in [false, true] {
        let runtime = runtime();
        for id in [CAP_ACOUSTIC_MAPPING, CAP_AIR_STEP] {
            grant(&runtime, id);
        }
        let mut selected = vec![CAP_AIR_STEP.into(), CAP_ACOUSTIC_MAPPING.into()];
        if already_granted {
            grant(&runtime, CAP_LOCAL_MAP);
            selected.insert(1, CAP_LOCAL_MAP.into());
        }
        runtime.apply_capability_command(CapabilityCommandRequest::Select { capability_ids: selected.clone() }).unwrap();
        let ctx = open(&runtime, 1);
        let chosen = runtime.choose_first_enhancement(CAP_LOCAL_MAP, &ctx).unwrap();
        if !already_granted { selected.push(CAP_LOCAL_MAP.into()); }
        let state = runtime.state.lock().unwrap();
        assert_eq!(state.capabilities.selected, selected);
        assert_eq!(state.capabilities.grants.iter().filter(|g| g.capability_id == CAP_LOCAL_MAP).count(), 1);
        drop(state);
        assert_eq!(chosen.capabilities.first_enhancement_choice.as_deref(), Some(CAP_LOCAL_MAP));
        let raw = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
        let saved = crate::save_v6::read_save(&runtime.save_root).unwrap();
        assert_eq!(saved.save.capabilities.selected, selected);
        assert!(!String::from_utf8(raw.clone()).unwrap().contains("enhancementTerminal"));
        assert_eq!(runtime.choose_first_enhancement(CAP_REAR_VIEW, &ctx).unwrap_err(), "E_CAPABILITY_REJECTED: FirstEnhancementAlreadyChosen");
        assert_unchanged(&runtime, &chosen, &raw);
        for _ in 0..3 { assert!(runtime.has_save()); assert!(runtime.has_default_save()); }
        assert_unchanged(&runtime, &chosen, &raw);
        let continued = runtime.continue_saved().unwrap();
        assert_eq!(continued.capabilities.first_enhancement_choice.as_deref(), Some(CAP_LOCAL_MAP));
        assert_eq!(std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap(), raw);
        std::fs::remove_dir_all(&runtime.save_root).unwrap();
    }
}

#[test]
fn first_evolution_cancel_scene_change_and_terminal_readiness_never_grant() {
    let runtime = runtime();
    let ctx = open(&runtime, 1);
    runtime.resume_context_ordered(&session(&runtime), 2).unwrap();
    assert!(!runtime.has_default_save());
    assert!(runtime.snapshot().unwrap().capabilities.first_enhancement_choice.is_none());
    let pending = runtime.reset_new().unwrap();
    assert_eq!(runtime.capability_terminal_status(&ctx.id, "loading-terminal", pending.world_epoch).unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
    assert_eq!(runtime.choose_first_enhancement(CAP_LOCAL_MAP, &ctx).unwrap_err(), "E_SCENE_ENTRY_NOT_READY");
    entry_test_support::acknowledge_ready(&runtime, pending);
    runtime.pause_context_ordered(&session(&runtime), 3).unwrap();
    assert!(runtime.choose_first_enhancement(CAP_LOCAL_MAP, &ctx).is_err());
    assert!(!runtime.has_default_save());
}

#[cfg(unix)]
#[test]
fn first_evolution_save_failure_is_atomic_and_same_ticket_can_retry() {
    use std::os::unix::fs::PermissionsExt;
    let runtime = runtime();
    let ctx = open(&runtime, 1);
    runtime.save().unwrap();
    let before = runtime.snapshot().unwrap();
    let bytes = std::fs::read(crate::save_v6::save_path(&runtime.save_root)).unwrap();
    std::fs::set_permissions(&runtime.save_root, std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = runtime.choose_first_enhancement(CAP_REGENERATION, &ctx);
    std::fs::set_permissions(&runtime.save_root, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.unwrap_err().starts_with("E_SAVE_"));
    assert_unchanged(&runtime, &before, &bytes);
    let retry = runtime.choose_first_enhancement(CAP_REGENERATION, &ctx).unwrap();
    assert_eq!(retry.capabilities.first_enhancement_choice.as_deref(), Some(CAP_REGENERATION));
    assert_eq!(crate::save_v6::read_save(&runtime.save_root).unwrap().save.capabilities.first_enhancement_choice.as_deref(), Some(CAP_REGENERATION));
    std::fs::remove_dir_all(&runtime.save_root).unwrap();
}
