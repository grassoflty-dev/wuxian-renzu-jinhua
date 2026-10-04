//! Arranged historical-save and phase fixtures; not campaign progression proof.
use super::*;
use crate::formal_runtime::entry_test_support::acknowledge_ready;
use crate::save_v6::SaveV6;
use crate::world_v3::{ActorAiState, ActorAttackKind};

fn fixture(scene: &str) -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("gate-b-roster-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap(); runtime.pause().unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    let registry = runtime.scene_registry.lock().unwrap().clone().unwrap();
    runtime.install_scene_registry(registry, scene).unwrap(); runtime
}
fn capture(runtime: &FormalRuntime) -> SaveV6 {
    SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap()
}
fn actors(runtime: &FormalRuntime) -> Vec<ActorRuntime> { runtime.state.lock().unwrap().world.generic_actors.clone() }

#[test]
fn fresh_gate_b_marks_only_its_exact_security_and_brute_roster() {
    let runtime = fixture(SCENE); let save = capture(&runtime);
    assert_eq!(save.world_persistent_v1.grey_hive.gate_b_actor_roster_version, 1);
    assert_eq!(save.save.generic_actors, authored(&canonical().unwrap()).unwrap());
    assert_eq!(save.save.generic_actors[2].entity_type, BRUTE);
    let state = runtime.state.lock().unwrap();
    assert!(state.kcc.can_occupy(state.world.generic_actors[2].home_m, 0.55));
    drop(state);
    let other = fixture("gh_entry_maintenance");
    assert_eq!(capture(&other).world_persistent_v1.grey_hive.gate_b_actor_roster_version, 0);
}

#[test]
fn historical_security_state_upgrades_on_clone_for_file_slot_and_repeated_continue() {
    let runtime = fixture(SCENE); let mut old = capture(&runtime);
    old.world_persistent_v1.grey_hive.gate_b_actor_roster_version = 0;
    old.world_persistent_v1.grey_hive.sentinel_first_kill = true;
    old.save.generic_actors.truncate(2);
    for actor in &mut old.save.generic_actors {
        actor.hp -= 7; actor.state = ActorAiState::Chasing; actor.position_m.x_m += 0.1;
    }
    crate::save_v6::write_save(&runtime.save_root, &old).unwrap();
    crate::save_slots::create_slot_v6(&runtime.save_root, "historical", "Historical", &old).unwrap();
    let path = crate::save_v6::save_path(&runtime.save_root); let slot_path = runtime.save_root.join("slots/historical/slot-v6.json");
    let bytes = std::fs::read(&path).unwrap(); let slot_bytes = std::fs::read(&slot_path).unwrap();
    let mut epoch = runtime.snapshot().unwrap().world_epoch;
    for slot in [false, false, true] {
        let view = if slot { runtime.continue_slot("historical").unwrap() } else { runtime.continue_saved().unwrap() };
        assert!(runtime.state.lock().unwrap().paused && view.entry_token.is_some());
        assert!(view.world_epoch > epoch); epoch = view.world_epoch;
        let loaded = capture(&runtime);
        assert_eq!(&loaded.save.generic_actors[..2], &old.save.generic_actors);
        assert_eq!(loaded.save.generic_actors[2], authored(&canonical().unwrap()).unwrap()[2]);
        assert!(loaded.world_persistent_v1.grey_hive.sentinel_first_kill);
        assert_eq!(loaded.world_persistent_v1.grey_hive.gate_b_actor_roster_version, 1);
        assert_eq!(loaded.save.progression, old.save.progression);
        acknowledge_ready(&runtime, view);
        assert_eq!(std::fs::read(&path).unwrap(), bytes); assert_eq!(std::fs::read(&slot_path).unwrap(), slot_bytes);
    }
}

#[test]
fn version_zero_empty_and_already_exact_current_rosters_remain_compatible() {
    for empty in [true, false] {
        let runtime = fixture(SCENE); let mut old = capture(&runtime);
        old.world_persistent_v1.grey_hive.gate_b_actor_roster_version = 0;
        if empty { old.save.generic_actors.clear(); }
        crate::save_v6::write_save(&runtime.save_root, &old).unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert_eq!(actors(&runtime), authored(&canonical().unwrap()).unwrap());
        assert_eq!(capture(&runtime).world_persistent_v1.grey_hive.gate_b_actor_roster_version, 1);
    }
}

#[test]
fn current_gate_b_omissions_duplicates_identity_shifts_and_version_reject_atomically() {
    for fault in ["missing", "empty", "extra", "duplicate", "id", "type", "old-home", "new-home", "version"] {
        let runtime = fixture(SCENE); runtime.save().unwrap(); runtime.save_slot("current", "Current", true).unwrap();
        let path = crate::save_v6::save_path(&runtime.save_root); let slot_path = runtime.save_root.join("slots/current/slot-v6.json");
        let bytes = std::fs::read(&path).unwrap(); let slot_bytes = std::fs::read(&slot_path).unwrap();
        let mut bad = capture(&runtime);
        match fault {
            "missing" => { bad.save.generic_actors.pop(); },
            "empty" => bad.save.generic_actors.clear(),
            "extra" => { let mut actor = bad.save.generic_actors[2].clone(); actor.entity_id.push_str("_extra"); bad.save.generic_actors.push(actor); },
            "duplicate" => bad.save.generic_actors[1] = bad.save.generic_actors[0].clone(),
            "id" => bad.save.generic_actors[2].entity_id.push_str("_shift"),
            "type" => { let old = bad.save.generic_actors[2].clone(); bad.save.generic_actors[2] = ActorRuntime::spawn(&old.entity_id, "enemy.clockworks.furnace_hound", old.home_m).unwrap(); },
            "old-home" => bad.save.generic_actors[0].home_m.x_m += 0.1,
            "new-home" => bad.save.generic_actors[2].home_m.x_m += 0.1,
            "version" => bad.world_persistent_v1.grey_hive.gate_b_actor_roster_version = 2,
            _ => unreachable!(),
        }
        assert!(crate::save_v6::write_save(&runtime.save_root, &bad).is_err(), "{fault}");
        assert!(crate::save_slots::overwrite_slot_v6(&runtime.save_root, "current", "Bad", &bad).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes); assert_eq!(std::fs::read(&slot_path).unwrap(), slot_bytes);
        let raw = serde_json::to_vec(&bad).unwrap(); std::fs::write(&path, &raw).unwrap();
        let mut slot: serde_json::Value = serde_json::from_slice(&slot_bytes).unwrap(); slot["save"] = serde_json::to_value(&bad).unwrap();
        let bad_slot = serde_json::to_vec(&slot).unwrap(); std::fs::write(&slot_path, &bad_slot).unwrap();
        let before = runtime.snapshot().unwrap(); let before_actors = actors(&runtime);
        assert!(runtime.continue_saved().is_err(), "{fault}"); assert!(runtime.continue_slot("current").is_err(), "{fault}");
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(actors(&runtime), before_actors);
        assert_eq!(std::fs::read(&path).unwrap(), raw); assert_eq!(std::fs::read(&slot_path).unwrap(), bad_slot);
    }
}

#[test]
fn current_damaged_or_dead_brute_is_not_reset_and_other_gh_sentinel_state_is_unchanged() {
    for dead in [false, true] {
        let runtime = fixture(SCENE);
        runtime.state.lock().unwrap().world.generic_actors[2].take_damage_with_stagger(if dead { u32::MAX } else { 9 }, 20);
        let before = actors(&runtime); runtime.save().unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap()); assert_eq!(actors(&runtime), before);
        assert!(!runtime.state.lock().unwrap().presentation_events.iter().any(|event| event.kind == "EnemyDeath"));
    }
    for scene in ["gh_entry_maintenance", "gh_sentinel_arena"] {
        let runtime = fixture(scene);
        runtime.state.lock().unwrap().world_persistent_v1.grey_hive.gate_b_actor_roster_version = 1;
        runtime.save().unwrap(); let before = capture(&runtime);
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap()); let after = capture(&runtime);
        assert_eq!(after.save.generic_actors, before.save.generic_actors);
        assert_eq!(after.save.actors, before.save.actors);
        assert_eq!(after.world_persistent_v1.grey_hive, before.world_persistent_v1.grey_hive);
    }
}

fn committed(runtime: &FormalRuntime, kind: ActorAttackKind, active: bool) -> ActorRuntime {
    let mut state = runtime.state.lock().unwrap(); let kcc = state.kcc.clone();
    let target = Vec3::new(if kind == ActorAttackKind::Charge { 13.5 } else { 16.0 }, 0.0, 8.0).unwrap();
    state.world.player.position_m = target;
    let actor = &mut state.world.generic_actors[2];
    for _ in 0..100 {
        actor.tick(target, &kcc, 1.0 / 60.0, true);
        if actor.state == if active { ActorAiState::Active } else { ActorAiState::Windup } { break; }
    }
    assert_eq!(actor.current_attack, Some(kind)); actor.clone()
}

#[test]
fn native_brute_restore_and_hold_publish_exact_radial_or_committed_phase_before_readiness() {
    for (kind, active) in [(ActorAttackKind::Charge, false), (ActorAttackKind::Charge, true), (ActorAttackKind::Slam, false)] {
        let runtime = fixture(SCENE); let actor = committed(&runtime, kind, active);
        runtime.save().unwrap(); let view = runtime.continue_saved().unwrap();
        assert!(runtime.state.lock().unwrap().paused && view.entry_token.is_some());
        let cues = runtime.presentation_events_since(view.world_epoch, 0).unwrap();
        assert_eq!(cues.len(), 1); assert_eq!(cues[0].actor_id.as_deref(), Some("gh_gate_b_brute_01"));
        assert_eq!(cues[0].kind, if active { "EnemyChargeMotion" } else { "EnemyAttackTelegraph" });
        assert_eq!(cues[0].attack_kind.as_deref(), Some(if kind == ActorAttackKind::Charge { "charge" } else { "slam" }));
        assert!(cues[0].duration_ms.unwrap() > 0);
        if kind == ActorAttackKind::Slam { assert_eq!(cues[0].radius_m, 2.0); assert_eq!(cues[0].range_m, Some(2.0)); }
        else { assert_eq!(cues[0].radius_m, 0.55); assert_eq!(cues[0].range_m, Some(4.4)); assert_eq!(cues[0].direction_rad, -std::f32::consts::FRAC_PI_2); }
        assert_eq!(actors(&runtime)[2], actor);
        acknowledge_ready(&runtime, view.clone());
        let context = SessionContext { world_id: view.world_id, scene_id: view.scene_id, world_epoch: view.world_epoch };
        runtime.pause_context_ordered(&context, 2).unwrap();
        let fresh = runtime.presentation_events_since(context.world_epoch, cues[0].event_id).unwrap();
        assert_eq!(fresh.len(), 1); assert_eq!(fresh[0].direction_rad, cues[0].direction_rad);
        assert_eq!(actors(&runtime)[2], actor);
        assert_eq!(runtime.resume_context_ordered(&context, 1).unwrap_err(), "E_LIFECYCLE_STALE_COMMAND");
        assert!(runtime.state.lock().unwrap().paused);
    }
}

#[test]
fn historical_partial_security_and_late_gate_b_install_failure_do_not_mutate_authority() {
    let runtime = fixture(SCENE); let mut partial = capture(&runtime);
    partial.world_persistent_v1.grey_hive.gate_b_actor_roster_version = 0;
    partial.save.generic_actors.truncate(1);
    assert_eq!(partial.validate().unwrap_err(), "E_SAVE_ACTOR_ROSTER_MISMATCH");
    let runtime = fixture("gh_bio_isolation"); runtime.resume().unwrap();
    let transition = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene().transitions.iter()
        .find(|transition| transition.to_scene_id == SCENE).unwrap().clone();
    let (x, z) = polygon_interior(&transition.polygon).unwrap();
    {
        let mut state = runtime.state.lock().unwrap(); state.world.player.position_m = Vec3::new(x, 0.0, z).unwrap();
        state.entry_generation = build_ui::MAX_SAFE_REVISION;
    }
    let locked = runtime.snapshot().unwrap();
    assert_eq!(runtime.transition_scene(&transition.id, "late-brute-install", locked.world_epoch).unwrap_err(), "E_SCENE_RUNTIME_ProgressionLocked");
    assert_eq!(runtime.snapshot().unwrap(), locked);
    // Arrange the authored prerequisite so this boundary fixture reaches the
    // later readiness failure; it is not a claim of earned campaign progress.
    runtime.state.lock().unwrap().route.progress.iter_mut().find(|p| p.world_id == "grey_hive").unwrap()
        .completed_events.push(transition.requires_event.clone().unwrap());
    let before = runtime.snapshot().unwrap(); let persistent = capture(&runtime).world_persistent_v1;
    assert_eq!(runtime.transition_scene(&transition.id, "late-brute-install", before.world_epoch).unwrap_err(), "E_SCENE_ENTRY_GENERATION_EXHAUSTED");
    assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(capture(&runtime).world_persistent_v1, persistent);
    runtime.state.lock().unwrap().entry_generation = 0;
    let view = runtime.transition_scene(&transition.id, "late-brute-install", before.world_epoch).unwrap();
    assert!(view.entry_token.is_some() && runtime.state.lock().unwrap().paused);
    assert_eq!(capture(&runtime).world_persistent_v1.grey_hive.gate_b_actor_roster_version, 1);
    assert!(view.actors.iter().any(|actor| actor.entity_id == "gh_gate_b_brute_01"));
    acknowledge_ready(&runtime, view);
}
