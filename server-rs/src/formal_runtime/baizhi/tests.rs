use super::*;
use crate::{save_slots, save_v6::{self, SaveV6}, world_progression::HiveChoice::*};
use std::{fs, sync::atomic::AtomicU64};
static SERIAL: AtomicU64 = AtomicU64::new(1);
fn runtime() -> FormalRuntime {
    let root = std::env::temp_dir().join(format!("baizhi-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::SeqCst)));
    let runtime = FormalRuntime::new_with_save_dir(root).unwrap();
    runtime.stop_owner.store(true, Ordering::SeqCst);
    runtime.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&runtime).unwrap();
    entry_test_support::acknowledge_ready(&runtime, runtime.reset_new().unwrap());
    let registry = runtime.scene_registry.lock().unwrap().as_ref().unwrap().clone();
    runtime.install_scene_registry(registry, "gh_bio_isolation").unwrap();
    {
        let mut state = runtime.state.lock().unwrap();
        let revision = state.world.revision;
        for command in [RouteCommand::Enter { world_id: "grey_hive".into(), request_id: "bz-enter".into() },
            RouteCommand::Progress { event_id: "hive_power".into(), request_id: "bz-power".into() },
            RouteCommand::Progress { event_id: "hive_lockdown".into(), request_id: "bz-lockdown".into() }] {
            assert!(matches!(apply_route_command(&mut state.route, command, revision), RouteResult::Applied { .. }));
        }
        state.world.player.position_m = Vec3 { x_m: 8.0, y_m: 0.0, z_m: 9.3 };
    }
    runtime
}
fn request(runtime: &FormalRuntime, seq: u64) -> BaizhiBeginRequest {
    let state = runtime.state.lock().unwrap();
    BaizhiBeginRequest { context: SessionContext { world_id: state.world.world_id.clone(), scene_id: state.world.scene_id.clone(),
        world_epoch: state.world.revision.world_epoch }, entity_id: ENTITY_ID.into(), interaction_id: INTERACTION_ID.into(),
        owner_id: "baizhi-ui-1".into(), generation: seq, command_sequence: seq }
}
fn open(runtime: &FormalRuntime, seq: u64) -> BaizhiDialogueTicket { runtime.baizhi_begin(request(runtime, seq)).unwrap().ticket.unwrap() }
fn capture(runtime: &FormalRuntime) -> SaveV6 { SaveV6::capture(&runtime.state.lock().unwrap(), runtime.scene_runtime.lock().unwrap().as_ref()).unwrap() }
fn assert_outcome(runtime: &FormalRuntime, outcome: HiveChoice) {
    let view = runtime.snapshot().unwrap();
    assert_eq!(view.baizhi.as_ref().unwrap().choice, outcome);
    assert_eq!(view.npcs.as_ref().unwrap().len(), 1);
    assert!(view.actors.iter().all(|actor| actor.entity_id != ENTITY_ID));
    assert!(view.interactables.iter().all(|item| item.entity_id != INTERACTION_ID));
}
fn cleanup(runtime: &FormalRuntime) { if runtime.save_root.exists() { fs::remove_dir_all(&runtime.save_root).unwrap(); } }

#[test]
fn three_outcomes_keep_npc_and_gate_b_with_real_kcc_entry_detour_and_return() {
    for outcome in [Unresolved, Taken, Left] {
        let runtime = runtime();
        // Use the actual compiled scene KCC and production player body, including
        // its radius, for the entire entry -> NPC -> main path -> Gate B walk.
        let (mut body, kcc) = { let state = runtime.state.lock().unwrap(); (state.world.player.clone(), state.kcc.clone()) };
        body.position_m = Vec3 { x_m: 2.0, y_m: 0.0, z_m: 8.0 };
        for target in [[8.0, 8.0], [8.0, 9.3]] { walk(&mut body, &kcc, target); }
        runtime.state.lock().unwrap().world.player = body.clone();
        let seq = runtime.state.lock().unwrap().route.event_seq;
        let ticket = open(&runtime, 1);
        if outcome != Unresolved { runtime.baizhi_commit(&ticket, "choice-1", outcome.clone()).unwrap(); }
        else { assert!(!save_v6::save_path(&runtime.save_root).exists()); }
        assert_eq!(runtime.state.lock().unwrap().route.event_seq, seq);
        assert_outcome(&runtime, outcome.clone());
        assert!(runtime.baizhi_close(&ticket).unwrap().resumed);
        for target in [[8.0, 8.0], [20.0, 8.0], [23.0, 8.0]] { walk(&mut body, &kcc, target); }
        runtime.state.lock().unwrap().world.player = body;
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let gate = runtime.transition_scene("gh_bio_to_gate_b", "bz-gate", epoch).unwrap();
        assert_eq!(gate.scene_id, "gh_gate_b");
        assert!(gate.baizhi.is_none() && gate.npcs.is_none());
        assert_eq!(choice(&runtime.state.lock().unwrap()), Some(outcome));
        cleanup(&runtime);
    }
}
fn walk(body: &mut crate::continuous_kcc::KccBody, world: &StaticKccWorld, target: [f32;2]) {
    for _ in 0..1000 {
        let dx = target[0] - body.position_m.x_m; let dz = target[1] - body.position_m.z_m;
        let distance = (dx*dx + dz*dz).sqrt();
        if distance < 0.06 { return; }
        crate::continuous_kcc::step_kcc(body, world, (dx/distance, dz/distance), 1.0/60.0).unwrap();
        assert!(world.can_occupy(body.position_m, body.radius_m));
    }
    panic!("KCC failed to reach {target:?}: {:?}", body.position_m);
}
#[test]
fn unresolved_close_is_no_save_and_reopen_can_choose_readonly_terminal() {
    let runtime = runtime();
    let ticket = open(&runtime, 1);
    assert_eq!(runtime.baizhi_commit(&ticket, "bad-unresolved", Unresolved).unwrap_err(), "E_BAIZHI_CHOICE_NOT_COMMITTABLE");
    runtime.baizhi_close(&ticket).unwrap();
    assert!(!save_v6::save_path(&runtime.save_root).exists());
    let ticket = open(&runtime, 2);
    runtime.baizhi_commit(&ticket, "taken", Taken).unwrap();
    let bytes = fs::read(save_v6::save_path(&runtime.save_root)).unwrap();
    assert!(runtime.state.lock().unwrap().paused);
    assert_eq!(runtime.state.lock().unwrap().baizhi.ticket.as_ref(), Some(&ticket));
    assert_eq!(runtime.baizhi_commit(&ticket, "taken", Taken).unwrap_err(), "E_BAIZHI_DUPLICATE_REQUEST");
    assert_eq!(runtime.baizhi_commit(&ticket, "taken", Left).unwrap_err(), "E_BAIZHI_REQUEST_PAYLOAD_MISMATCH");
    runtime.baizhi_close(&ticket).unwrap();
    let ticket = open(&runtime, 3);
    assert_eq!(runtime.baizhi_commit(&ticket, "new-terminal", Taken).unwrap_err(), "E_BAIZHI_TERMINAL_CHOICE");
    assert_eq!(runtime.baizhi_commit(&ticket, "new-terminal", Left).unwrap_err(), "E_BAIZHI_REQUEST_PAYLOAD_MISMATCH");
    assert_eq!(runtime.baizhi_commit(&ticket, "new-conflict", Left).unwrap_err(), "E_BAIZHI_TERMINAL_CHOICE");
    assert_eq!(fs::read(save_v6::save_path(&runtime.save_root)).unwrap(), bytes);
    runtime.baizhi_close(&ticket).unwrap(); cleanup(&runtime);
}
#[test]
fn begin_and_commit_reject_bad_context_target_range_dead_loading_without_mutation() {
    let runtime = runtime(); runtime.save().unwrap();
    for case in 0..9 {
        let base = runtime.state.lock().unwrap().clone();
        let mut req = request(&runtime, 1);
        { let mut s = runtime.state.lock().unwrap(); match case {
            0 => req.context.world_epoch += 1, 1 => req.context.scene_id = "wrong".into(),
            2 => req.entity_id = "other".into(), 3 => req.interaction_id = "other".into(),
            4 => s.world.player.position_m.x_m = 30.0, 5 => s.world.player_hp = 0,
            6 => s.paused = true, 7 => s.pending_entry = Some(crate::world_v3::SceneEntryToken { generation:1,
                world_id:"grey_hive".into(), scene_id:"gh_bio_isolation".into(), world_epoch:s.world.revision.world_epoch }),
            _ => req.command_sequence = 0,
        }}
        let before = runtime.snapshot().unwrap();
        let bytes = fs::read(save_v6::save_path(&runtime.save_root)).unwrap();
        assert!(runtime.baizhi_begin(req).is_err(), "begin case {case}");
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(fs::read(save_v6::save_path(&runtime.save_root)).unwrap(), bytes);
        *runtime.state.lock().unwrap() = base;
    }
    let ticket = open(&runtime, 1);
    let base = runtime.state.lock().unwrap().clone();
    for case in 0..8 {
        *runtime.state.lock().unwrap() = base.clone(); let mut forged = ticket.clone();
        { let mut s = runtime.state.lock().unwrap(); match case {
            0 => forged.owner_id = "other".into(), 1 => forged.generation += 1, 2 => forged.pause_command_sequence += 1,
            3 => s.world.player.position_m.y_m = 3.0, 4 => s.world.player_hp = 0,
            5 => s.paused = false, 6 => s.world.revision.world_epoch += 1,
            _ => s.pending_entry = Some(crate::world_v3::SceneEntryToken { generation:1, world_id:"grey_hive".into(), scene_id:"gh_bio_isolation".into(), world_epoch:s.world.revision.world_epoch }),
        }}
        let before = runtime.snapshot().unwrap(); assert!(runtime.baizhi_commit(&forged, "bad-confirm", Taken).is_err(), "confirm case {case}");
        assert_eq!(runtime.snapshot().unwrap(), before);
    }
    *runtime.state.lock().unwrap() = base; cleanup(&runtime);
}
#[test]
fn later_pause_takes_ownership_and_close_failure_keeps_retry_ticket() {
    let runtime = runtime(); let ticket = open(&runtime, 1);
    let ctx = request(&runtime, 2).context;
    assert_eq!(runtime.resume_context_ordered(&ctx, 2).unwrap_err(), "E_BAIZHI_CLOSE_REQUIRED");
    runtime.state.lock().unwrap().world.player_hp = 0;
    assert_eq!(runtime.baizhi_close(&ticket).unwrap_err(), "E_RUNTIME_DEAD");
    assert_eq!(runtime.state.lock().unwrap().baizhi.ticket.as_ref(), Some(&ticket));
    assert!(runtime.state.lock().unwrap().paused);
    runtime.state.lock().unwrap().world.player_hp = 100;
    runtime.pause_context_ordered(&ctx, 2).unwrap();
    assert!(!runtime.baizhi_close(&ticket).unwrap().resumed);
    assert!(runtime.state.lock().unwrap().paused);
    runtime.resume_context_ordered(&ctx, 3).unwrap();
    let ticket = open(&runtime, 4);
    assert!(runtime.baizhi_close(&ticket).unwrap().resumed);
    assert!(runtime.baizhi_close(&ticket).unwrap().resumed); // lost close acknowledgement safely retried
    let ticket = open(&runtime, 5);
    runtime.reset_new().unwrap(); let before = runtime.snapshot().unwrap();
    assert_eq!(runtime.baizhi_close(&ticket).unwrap_err(), "E_BAIZHI_STALE_CONTEXT");
    assert_eq!(runtime.baizhi_commit(&ticket, "stale", Taken).unwrap_err(), "E_BAIZHI_STALE_CONTEXT");
    assert_eq!(runtime.snapshot().unwrap(), before); cleanup(&runtime);
}
#[test]
fn active_slot_only_atomic_failures_bind_payload_and_retry_exact_request() {
    for named in [false, true] {
        for stage in ["prepared", "commit", "postwrite", "verified"] {
            let runtime = runtime();
            runtime.save().unwrap();
            save_slots::create_slot_v6(&runtime.save_root, "manual", "Chosen slot", &capture(&runtime)).unwrap();
            if named { *runtime.active_slot.lock().unwrap() = Some("manual".into()); }
            let default_path = save_v6::save_path(&runtime.save_root);
            let slot_path = runtime.save_root.join("slots/manual/slot-v6.json");
            let default_bytes = fs::read(&default_path).unwrap(); let slot_bytes = fs::read(&slot_path).unwrap();
            let ticket = open(&runtime, 1); let before = runtime.snapshot().unwrap();
            save_v6::RECOVERY_TEST_FAULT.with(|f| f.set(Some(stage)));
            let result = runtime.baizhi_commit(&ticket, "choice", Left);
            save_v6::RECOVERY_TEST_FAULT.with(|f| f.set(None));
            assert!(result.is_err(), "{named}/{stage}");
            assert_eq!(runtime.snapshot().unwrap(), before);
            assert_eq!(fs::read(&default_path).unwrap(), default_bytes); assert_eq!(fs::read(&slot_path).unwrap(), slot_bytes);
            assert_eq!(runtime.baizhi_commit(&ticket, "choice", Taken).unwrap_err(), "E_BAIZHI_REQUEST_PAYLOAD_MISMATCH");
            runtime.baizhi_commit(&ticket, "choice", Left).unwrap();
            assert_eq!(*runtime.active_slot.lock().unwrap(), if named { Some("manual".into()) } else { None });
            if named { assert_eq!(fs::read(&default_path).unwrap(), default_bytes); assert_ne!(fs::read(&slot_path).unwrap(), slot_bytes); }
            else { assert_ne!(fs::read(&default_path).unwrap(), default_bytes); assert_eq!(fs::read(&slot_path).unwrap(), slot_bytes); }
            let view = if named { runtime.continue_slot("manual").unwrap() } else { runtime.continue_saved().unwrap() };
            entry_test_support::acknowledge_ready(&runtime, view); assert_outcome(&runtime, Left);
            cleanup(&runtime);
        }
    }
}
#[test]
fn missing_corrupt_readonly_named_target_never_falls_back() {
    for fault in ["missing", "corrupt", "future", "directory", "v4-readonly"] {
        let runtime = runtime(); runtime.save_slot("manual", "Chosen", true).unwrap(); runtime.save().unwrap();
        let path = runtime.save_root.join("slots/manual/slot-v6.json"); let original = fs::read(&path).unwrap();
        let default = fs::read(save_v6::save_path(&runtime.save_root)).unwrap(); fs::remove_file(&path).unwrap();
        match fault {
            "missing" => (), "corrupt" => fs::write(&path, b"bad").unwrap(),
            "future" => { let mut v:serde_json::Value=serde_json::from_slice(&original).unwrap(); v["schemaVersion"]=999.into();fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap(); },
            "directory" => fs::create_dir(&path).unwrap(),
            _ => { let old:serde_json::Value=serde_json::from_str(include_str!("../../../tests/fixtures/capability-save-profiles/legacy-v4.json")).unwrap();fs::write(path.with_file_name("slot-v4.json"),serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"slotId":"manual","displayName":"Read only","updatedAtMs":1,"save":old})).unwrap()).unwrap(); },
        }
        let ticket=open(&runtime,1);let before=runtime.snapshot().unwrap();
        assert!(runtime.baizhi_commit(&ticket,"choice",Taken).is_err(),"{fault}");assert_eq!(runtime.snapshot().unwrap(),before);
        assert_eq!(fs::read(save_v6::save_path(&runtime.save_root)).unwrap(),default);assert_eq!(*runtime.active_slot.lock().unwrap(),Some("manual".into()));
        if fault=="directory" {fs::remove_dir(&path).unwrap();} if fault=="v4-readonly" {fs::remove_file(path.with_file_name("slot-v4.json")).unwrap();}
        fs::write(&path,original).unwrap();runtime.baizhi_commit(&ticket,"choice",Taken).unwrap();cleanup(&runtime);
    }
}
#[test]
fn old_terminal_and_non_gh_choices_roundtrip_without_second_authority_or_schema_change() {
    for old in [Taken, Left] {
        let runtime=runtime(); {let mut state=runtime.state.lock().unwrap();for row in &mut state.route.progress {row.hive_choice=old.clone();}}
        runtime.save().unwrap();let bytes=fs::read(save_v6::save_path(&runtime.save_root)).unwrap();
        let json:serde_json::Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(json["schemaVersion"],6);assert_eq!(json["save"]["dialogFlags"],serde_json::json!([]));
        let view=runtime.continue_saved().unwrap();entry_test_support::acknowledge_ready(&runtime,view);assert_outcome(&runtime,old.clone());
        assert!(runtime.state.lock().unwrap().route.progress.iter().all(|r|r.hive_choice==old));
        for bad in [serde_json::Value::Null,serde_json::json!("unknown")] {let mut json=json.clone();json["save"]["progression"]["progress"][0]["hiveChoice"]=bad;assert!(serde_json::from_value::<SaveV6>(json).is_err());}
        let mut missing=json;missing["save"]["progression"]["progress"][0].as_object_mut().unwrap().remove("hiveChoice");assert!(serde_json::from_value::<SaveV6>(missing).is_err());
        cleanup(&runtime);
    }
}
#[test]
fn normal_far_and_paused_projection_is_narrow_and_no_generic_interact() {
    let runtime=runtime();assert!(runtime.snapshot().unwrap().baizhi.unwrap().can_interact);
    runtime.state.lock().unwrap().world.player.position_m.x_m=2.0;let view=runtime.snapshot().unwrap();
    assert!(view.baizhi.as_ref().unwrap().available);assert!(!view.baizhi.unwrap().can_interact);assert!(!view.npcs.unwrap()[0].interactable);
    runtime.state.lock().unwrap().world.player.position_m.x_m=8.0;let ticket=open(&runtime,1);
    let view=runtime.snapshot().unwrap();assert!(view.baizhi.as_ref().unwrap().available);assert!(!view.baizhi.unwrap().can_interact);assert!(!view.npcs.unwrap()[0].interactable);
    assert_eq!(runtime.activate_scene_interaction(INTERACTION_ID,"ordinary",ticket.world_epoch).unwrap_err(),"E_BAIZHI_DEDICATED_COMMAND_REQUIRED");
    let value=serde_json::to_value(runtime.snapshot().unwrap()).unwrap();assert_eq!(value["npcs"][0]["position"],serde_json::json!([8.0,0.0,10.5]));
    runtime.baizhi_close(&ticket).unwrap();runtime.reset_new().unwrap();let json=serde_json::to_value(runtime.snapshot().unwrap()).unwrap();assert!(json.get("baizhi").is_none()&&json.get("npcs").is_none());cleanup(&runtime);
}
#[test]
fn live_owner_freezes_enemy_ai_time_hp_and_clears_input_on_begin() {
    let runtime=runtime();
    {let mut state=runtime.state.lock().unwrap();state.latest_sample.move_x=1.0;state.world.player.velocity_mps.x_m=1.0;state.pending_combat.push(CombatIntent::Attack { request_id: 811 });
        // Alive security in attack range: combat is deliberately not a begin gate.
        state.world.generic_actors[0].position_m=Vec3{x_m:8.0,y_m:0.0,z_m:10.0};}
    runtime.stop_owner.store(false,Ordering::SeqCst);
    let (s,c,stop)=(Arc::clone(&runtime.state),Arc::clone(&runtime.scene_runtime),Arc::clone(&runtime.stop_owner));
    *runtime.owner_handle.lock().unwrap()=Some(std::thread::spawn(move||simulation_owner_loop(s,c,stop)));
    let ticket=open(&runtime,1);let before=runtime.snapshot().unwrap();
    {let state=runtime.state.lock().unwrap();assert_eq!(state.latest_sample.move_x,0.0);assert_eq!(state.world.player.velocity_mps,Vec3::zero());assert!(state.pending_combat.is_empty());}
    std::thread::sleep(Duration::from_millis(90));let after=runtime.snapshot().unwrap();assert_eq!(after.server_tick,before.server_tick);assert_eq!(after.server_time_ms,before.server_time_ms);assert_eq!(after.player.current_hp,before.player.current_hp);assert_eq!(after.actors,before.actors);
    runtime.baizhi_close(&ticket).unwrap();std::thread::sleep(Duration::from_millis(45));assert!(runtime.snapshot().unwrap().server_tick>before.server_tick);cleanup(&runtime);
}

#[test]
fn never_talking_still_walks_production_kcc_straight_to_gate_b() {
    let runtime = runtime();
    let (mut body, kcc) = { let s = runtime.state.lock().unwrap(); (s.world.player.clone(), s.kcc.clone()) };
    body.position_m = Vec3 { x_m: 2.0, y_m: 0.0, z_m: 8.0 };
    for target in [[8.0, 8.0], [20.0, 8.0], [23.0, 8.0]] { walk(&mut body, &kcc, target); }
    runtime.state.lock().unwrap().world.player = body;
    let epoch = runtime.snapshot().unwrap().world_epoch;
    assert_eq!(runtime.transition_scene("gh_bio_to_gate_b", "never-bz-gate", epoch).unwrap().scene_id, "gh_gate_b");
    assert_eq!(choice(&runtime.state.lock().unwrap()), Some(Unresolved));
    assert!(runtime.state.lock().unwrap().baizhi.ticket.is_none());
    assert!(!save_v6::save_path(&runtime.save_root).exists()); cleanup(&runtime);
}

#[cfg(unix)]
#[test]
fn filesystem_readonly_save_directory_preserves_target_and_allows_same_payload_retry() {
    use std::os::unix::fs::PermissionsExt;
    for named in [false, true] {
        let runtime = runtime(); runtime.save().unwrap();
        runtime.save_slot("manual", "Chosen", true).unwrap();
        if !named { *runtime.active_slot.lock().unwrap() = None; }
        let target = if named { runtime.save_root.join("slots/manual/slot-v6.json") } else { save_v6::save_path(&runtime.save_root) };
        let bytes = fs::read(&target).unwrap(); let ticket = open(&runtime, 1); let before = runtime.snapshot().unwrap();
        fs::set_permissions(target.parent().unwrap(), fs::Permissions::from_mode(0o555)).unwrap();
        let result = runtime.baizhi_commit(&ticket, "readonly-choice", Taken);
        fs::set_permissions(target.parent().unwrap(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(result.is_err(), "readonly write unexpectedly succeeded");
        assert_eq!(runtime.snapshot().unwrap(), before); assert_eq!(fs::read(&target).unwrap(), bytes);
        assert_eq!(runtime.state.lock().unwrap().baizhi.ticket.as_ref(), Some(&ticket));
        runtime.baizhi_commit(&ticket, "readonly-choice", Taken).unwrap();
        assert_outcome(&runtime, Taken); cleanup(&runtime);
    }
}

#[test]
fn actual_compiled_tauri_client_wire_fixture_obeys_strict_native_request_types() {
    // These bytes were captured from the compiled production TauriClient using
    // a complete V3 snapshot and deliberately polluted ticket inputs. The Web
    // regression compares its real invoke arguments to this identical fixture.
    let calls: Vec<serde_json::Value> = serde_json::from_str(include_str!("../../../tests/fixtures/baizhi-wire-v1.json")).unwrap();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0]["command"], "formal_baizhi_begin");
    assert_eq!(calls[1]["command"], "formal_baizhi_commit");
    assert_eq!(calls[2]["command"], "formal_baizhi_close");
    let begin: BaizhiBeginRequest = serde_json::from_value(calls[0]["args"]["request"].clone()).unwrap();
    let commit_ticket: BaizhiDialogueTicket = serde_json::from_value(calls[1]["args"]["ticket"].clone()).unwrap();
    let close_ticket: BaizhiDialogueTicket = serde_json::from_value(calls[2]["args"]["ticket"].clone()).unwrap();
    let selected: HiveChoice = serde_json::from_value(calls[1]["args"]["choice"].clone()).unwrap();
    assert_eq!(selected, Taken);
    assert!(valid_id(calls[1]["args"]["requestId"].as_str().unwrap()));
    assert_eq!(begin.owner_id, commit_ticket.owner_id);
    assert_eq!(begin.context.world_epoch, commit_ticket.world_epoch);
    assert_eq!(begin.command_sequence, commit_ticket.pause_command_sequence);
    assert_eq!(begin.generation, commit_ticket.generation);
    assert_eq!(commit_ticket, close_ticket);
    assert_eq!(serde_json::to_value(&begin).unwrap(), calls[0]["args"]["request"]);
    assert_eq!(serde_json::to_value(&commit_ticket).unwrap(), calls[1]["args"]["ticket"]);
    let mut bad = calls[0]["args"]["request"].clone();
    bad["context"]["protocolVersion"] = 3.into();
    assert!(serde_json::from_value::<BaizhiBeginRequest>(bad).is_err());
    for extra in ["protocolVersion", "player", "accidentalSaveField"] {
        let mut bad = calls[1]["args"]["ticket"].clone(); bad[extra] = serde_json::json!({});
        assert!(serde_json::from_value::<BaizhiDialogueTicket>(bad).is_err());
    }
    let mut wrong_case = calls[0]["args"]["request"].clone();
    let context = wrong_case["context"].as_object_mut().unwrap();
    let epoch = context.remove("worldEpoch").unwrap(); context.insert("world_epoch".into(), epoch);
    assert!(serde_json::from_value::<BaizhiBeginRequest>(wrong_case).is_err());
}
