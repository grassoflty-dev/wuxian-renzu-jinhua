//! Compatibility arrangements are not earned campaign evidence. Public lift
//! traversal coverage below arranges already-defeated canonical actors, isolating
//! support behavior. It is not earned combat or a genuine journey proof.
use super::*;
use crate::save_v6::SaveV6;
use crate::formal_runtime::entry_test_support::acknowledge_ready;
fn fixture(live:bool)->FormalRuntime {
    let root=std::env::temp_dir().join(format!("gear-support-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let r=FormalRuntime::new_with_save_dir(root).unwrap();r.pause().unwrap();
    if !live {r.stop_owner.store(true,Ordering::SeqCst);r.owner_handle.lock().unwrap().take().unwrap().join().unwrap();}
    crate::production_scene_bootstrap::install(&r).unwrap();let registry=r.scene_registry.lock().unwrap().clone().unwrap();r.install_scene_registry(registry,SCENE).unwrap();
    {let mut s=r.state.lock().unwrap();s.route.current_world_id="clockworks".into();s.route.progress.iter_mut().find(|p|p.world_id=="clockworks").unwrap().visit_id=1;}
    r
}
fn capture(r:&FormalRuntime)->SaveV6{SaveV6::capture(&r.state.lock().unwrap(),r.scene_runtime.lock().unwrap().as_ref()).unwrap()}
fn old(r:&FormalRuntime)->SaveV6{let mut s=capture(r);s.world_persistent_v1.clockworks.gear_shaft_support_version=0;s.save.moving_support_frame=None;s}
fn paths(r:&FormalRuntime)->(PathBuf,PathBuf){(crate::save_v6::save_path(&r.save_root),r.save_root.join("slots/fixture/slot-v6.json"))}
fn write_both(r:&FormalRuntime,s:&SaveV6){crate::save_v6::write_save(&r.save_root,s).unwrap();crate::save_slots::create_slot_v6(&r.save_root,"fixture","Fixture",s).unwrap();}
#[test]
fn old_floor_file_and_slot_upgrade_preserve_authority_fields_and_original_bytes(){
    for (x,z) in [(3.,8.),(12.,8.),(22.5,8.)]{
        let r=fixture(false);let mut old=old(&r);old.save.player.position_m=Vec3{x_m:x,y_m:0.,z_m:z};old.save.player.current_hp=63;old.save.player.velocity_mps.x_m=0.25;
        old.save.server_time_ms=3217;old.save.revision.server_tick=193;old.save.revision.authority_revision=200;
        write_both(&r,&old);let (file,slot)=paths(&r);let bytes=std::fs::read(&file).unwrap();let sb=std::fs::read(&slot).unwrap();
        for in_slot in [false,true,false]{let ready=if in_slot{r.continue_slot("fixture").unwrap()}else{r.continue_saved().unwrap()};
            assert!(ready.entry_token.is_some());assert!(!ready.interactables.iter().find(|i|i.entity_id=="cw_gear_to_furnace_heart").unwrap().active);assert_eq!(ready.player.transform.position_m,old.save.player.position_m);assert_eq!(ready.player.current_hp,63);
            assert_eq!(ready.server_time_ms,3217);let current=capture(&r);
            assert_eq!(current.world_persistent_v1.clockworks.gear_shaft_support_version,1);
            let frame=current.save.moving_support_frame.as_ref().unwrap();assert_eq!(frame.server_time_ms,3217);
            let mut expected_player=old.save.player.clone();
            // The existing held Continue contract clears horizontal intent.
            expected_player.velocity_mps=Vec3::zero();assert_eq!(current.save.player,expected_player);assert_eq!(current.save.generic_actors,old.save.generic_actors);
            let mut expected_capabilities=old.save.capabilities.clone();rebase_capability_revision_epoch(&mut expected_capabilities,ready.world_epoch).unwrap();assert_eq!(current.save.capabilities,expected_capabilities);assert_eq!(current.save.progression,old.save.progression);
            assert_eq!(current.progression,old.progression);assert_eq!(current.effect_sources,old.effect_sources);
            assert_eq!(current.save.scene_states,old.save.scene_states);
            let mut p=current.world_persistent_v1.clone();p.clockworks.gear_shaft_support_version=0;assert_eq!(p,old.world_persistent_v1);
            let projection=ready.support_scene.as_ref().unwrap();assert_eq!(projection.rider.mode,crate::moving_support::SupportRiderMode::Floor);assert!(projection.rider.support_id.is_none());
            acknowledge_ready(&r,ready);assert_eq!(std::fs::read(&file).unwrap(),bytes);assert_eq!(std::fs::read(&slot).unwrap(),sb);
        }
    }
}
#[test]
fn malformed_legacy_and_current_frames_reject_write_file_and_slot_without_installing(){
    for kind in ["raised","airborne","vertical_velocity","dash","upper_checkpoint","future_version","legacy_frame","current_missing","current_phase","lower_hold_contact","wall_position"]{
        let r=fixture(false);write_both(&r,&capture(&r));let (original_file,original_slot)=paths(&r);let original_bytes=std::fs::read(&original_file).unwrap();let original_slot_bytes=std::fs::read(&original_slot).unwrap();let mut bad=old(&r);match kind{
            "wall_position"=>bad.save.player.position_m.x_m=0.,
            "lower_hold_contact"=>bad.save.player.position_m.x_m=12.,
            "raised"=>bad.save.player.position_m.y_m=2.,"airborne"=>bad.save.player.grounded=false,
            "vertical_velocity"=>bad.save.player.velocity_mps.y_m=-1.,"dash"=>bad.save.player.dash_remaining_ms=1,
            "upper_checkpoint"=>bad.save.checkpoint_id=Some("cw_gear_upper_dock_checkpoint".into()),
            "future_version"=>bad.world_persistent_v1.clockworks.gear_shaft_support_version=2,
            "legacy_frame"=>bad.save.moving_support_frame=capture(&r).save.moving_support_frame,
            "current_missing"=>bad.world_persistent_v1.clockworks.gear_shaft_support_version=1,
            "current_phase"=>{bad=capture(&r);bad.save.moving_support_frame.as_mut().unwrap().poses[0].height_m+=0.125;},_=>unreachable!()}
        assert!(bad.validate().is_err(),"{kind}");assert!(crate::save_v6::write_save(&r.save_root,&bad).is_err(),"{kind}");assert_eq!(std::fs::read(&original_file).unwrap(),original_bytes);assert_eq!(std::fs::read(&original_slot).unwrap(),original_slot_bytes);
        let bytes=serde_json::to_vec(&bad).unwrap();let (file,slot)=paths(&r);std::fs::create_dir_all(slot.parent().unwrap()).unwrap();std::fs::write(&file,&bytes).unwrap();std::fs::write(&slot,&bytes).unwrap();let before=r.snapshot().unwrap();
        assert!(r.continue_saved().is_err(),"file {kind}");assert_eq!(r.snapshot().unwrap(),before);assert_eq!(std::fs::read(&file).unwrap(),bytes);
        assert!(r.continue_slot("fixture").is_err(),"slot {kind}");assert_eq!(r.snapshot().unwrap(),before);assert_eq!(std::fs::read(&slot).unwrap(),bytes);
    }
}
#[test]
fn legacy_profile_rejects_new_version_key_even_zero_or_null(){
    const FROZEN:&str=include_str!("../../../tests/fixtures/capability-save-profiles/legacy-empty.json");
    let r=fixture(false);std::fs::create_dir_all(&r.save_root).unwrap();let file=crate::save_v6::save_path(&r.save_root);
    std::fs::write(&file,FROZEN).unwrap();assert!(crate::save_v6::read_save(&r.save_root).is_ok(),"positive exact old-wire control");
    for value in [serde_json::json!(0),serde_json::Value::Null,serde_json::json!(1)]{
        let mut raw:serde_json::Value=serde_json::from_str(FROZEN).unwrap();raw["worldPersistentV1"]=serde_json::json!({"clockworks":{"gearShaftSupportVersion":value}});
        let bytes=serde_json::to_vec(&raw).unwrap();std::fs::write(&file,&bytes).unwrap();
        assert_eq!(crate::save_v6::read_save(&r.save_root).unwrap_err(),"E_SAVE_SCHEMA_INVALID");assert_eq!(std::fs::read(&file).unwrap(),bytes);
    }
}
#[test]
fn migration_requires_strict_exact_registry_and_cannot_install_an_active_traversal(){
    let r=fixture(false);let old=old(&r);let mut state=old.restore_state().unwrap();let scene=r.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene().clone();state.kcc=scene_kcc(&scene,&BTreeSet::new()).unwrap();
    assert!(!restore_candidate(&mut state,&old.save,&scene,false).unwrap());assert_eq!(state.world_persistent_v1.clockworks.gear_shaft_support_version,0);
    let mut forged=scene.clone();forged.verified_source_sha256="0".repeat(64);assert_eq!(restore_candidate(&mut state,&old.save,&forged,true).unwrap_err(),"E_GEAR_SUPPORT_CONTENT");
    state.world.combat_state.active_traversal=Some(crate::continuous_combat::ActiveTraversal{marker_index:0,request_id:1,start_position:state.world.player.position_m,elapsed_ms:0});
    assert_eq!(restore_candidate(&mut state,&old.save,&scene,true).unwrap_err(),"E_SAVE_GEAR_SUPPORT_LEGACY");assert_eq!(state.world_persistent_v1.clockworks.gear_shaft_support_version,0);
}

#[test]
fn the_upgrade_itself_only_marks_layout_version_and_never_moves_or_boards(){
    let r=fixture(false);let mut old=old(&r);old.save.player.current_hp=71;old.save.player.velocity_mps.x_m=0.4;old.save.server_time_ms=3217;
    let mut state=old.restore_state().unwrap();let scene=r.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene().clone();state.kcc=scene_kcc(&scene,&BTreeSet::new()).unwrap();
    let before=serde_json::to_value(SaveV5::capture(&state, r.scene_runtime.lock().unwrap().as_ref()).unwrap()).unwrap();
    let world_before=format!("{:?}",state.world);let persistent_before=state.world_persistent_v1.clone();
    assert!(restore_candidate(&mut state,&old.save,&scene,true).unwrap());assert_eq!(format!("{:?}",state.world),world_before);
    assert_eq!(serde_json::to_value(SaveV5::capture(&state,r.scene_runtime.lock().unwrap().as_ref()).unwrap()).unwrap(),before);
    let mut persistent=state.world_persistent_v1.clone();persistent.clockworks.gear_shaft_support_version=0;assert_eq!(persistent,persistent_before);
}

static NEXT_TIME:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(1000);
fn input(r:&FormalRuntime,v:&WorldView,x:f32,z:f32)->WorldView{
    r.submit_input(InputSample::new(v.world_epoch,v.ack_seq+1,NEXT_TIME.fetch_add(17,Ordering::Relaxed),x,z).unwrap(),vec![]).unwrap()
}
fn walk(r:&FormalRuntime,mut v:WorldView,x:f32,z:f32)->WorldView{
    for _ in 0..400{let p=v.player.transform.position_m;let dx=x-p.x_m;let dz=z-p.z_m;let len=dx.hypot(dz);if len<0.12{return input(r,&v,0.,0.);}
        assert!(v.player.current_hp>0,"authored mechanics route died");v=input(r,&v,dx/len,dz/len);}
    panic!("walk did not reach {x},{z}: {:?}",v.player.transform.position_m)
}
fn ride_to_dock(r:&FormalRuntime,mut v:WorldView,save_midride:bool)->WorldView{
    v=walk(r,v,12.,11.);
    for _ in 0..850{let lift=v.support_scene.as_ref().unwrap().poses.iter().find(|p|p.support_id=="cw_gear_main_lift").unwrap();
        if lift.phase==crate::moving_support::SupportPhase::LowerHold&&lift.phase_elapsed_ms<700{break;}v=input(r,&v,0.,0.);}
    v=walk(r,v,12.,8.);
    assert_eq!(v.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_main_lift"));
    let mut saved=false;
    for _ in 0..550{
        let y=v.player.transform.position_m.y_m;
        if save_midride&&!saved&&y>0.7&&y<1.5{
            let held=r.pause().unwrap();std::thread::sleep(Duration::from_millis(60));assert_eq!(r.snapshot().unwrap(),held);
            r.save_slot("moving","Moving lift",true).unwrap();let path=r.save_root.join("slots/moving/slot-v6.json");let bytes=std::fs::read(&path).unwrap();
            let prepared=r.continue_slot("moving").unwrap();assert_eq!(prepared.player.transform.position_m,held.player.transform.position_m);
            assert_eq!(prepared.server_time_ms,held.server_time_ms);assert_eq!(prepared.support_scene.as_ref().unwrap().poses,held.support_scene.as_ref().unwrap().poses);
            assert_eq!(prepared.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_main_lift"));assert_eq!(std::fs::read(&path).unwrap(),bytes);
            v=acknowledge_ready(r,prepared);saved=true;
        }
        if (v.player.transform.position_m.y_m-2.).abs()<0.001{break;}v=input(r,&v,0.,0.);
    }
    assert_eq!(saved,save_midride);assert!((v.player.transform.position_m.y_m-2.).abs()<0.001);v=walk(r,v,16.,8.);
    assert_eq!(v.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_upper_dock"));v
}
fn cross_gap(r:&FormalRuntime,mut v:WorldView,id:u64)->WorldView{
    v=walk(r,v,17.8,8.);let time=NEXT_TIME.fetch_add(17,Ordering::Relaxed);
    v=r.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:v.world_epoch,request_id:id,client_time_ms:time,kind:ActionKind::ContextTraversal}).unwrap();
    for _ in 0..80{if v.support_scene.as_ref().unwrap().rider.support_id.as_deref()==Some("cw_gear_furnace_landing"){break;}v=input(r,&v,0.,0.);}
    assert!((v.player.transform.position_m.y_m-2.).abs()<0.001);assert!((v.player.transform.position_m.x_m-20.2).abs()<0.15);
    assert_eq!(v.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_furnace_landing"));v
}

#[test]
fn isolated_public_lift_gap_checkpoint_and_continue_work_before_air_step(){
    let r=fixture(true);
    // Mechanics component setup only. Genuine campaign must earn these defeats.
    {let mut state=r.state.lock().unwrap();for actor in &mut state.world.generic_actors{actor.take_damage(u32::MAX);}}
    let mut v=r.resume().unwrap();assert!(!v.capabilities.items.iter().any(|c|c.capability_id==CAP_AIR_STEP&&c.granted));
    // The upper exit is not offered from the lower floor. The same-XZ
    // command rejection is exercised after the deliberate drop below.
    assert!(!v.interactables.iter().find(|i|i.entity_id=="cw_gear_to_furnace_heart").unwrap().active);
    v=walk(&r,v,3.,11.);v=walk(&r,v,11.,8.);
    assert_eq!(r.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:v.world_epoch,request_id:9001,client_time_ms:NEXT_TIME.fetch_add(17,Ordering::Relaxed),kind:ActionKind::ContextTraversal}).unwrap_err(),"E_TRAVERSAL_CAPABILITY");
    v=walk(&r,v,12.,11.);
    v=ride_to_dock(&r,v,true);
    let checked=crate::scene_route_commands::checkpoint(&r,"cw_gear_upper_dock_checkpoint","gear-upper",v.world_epoch).unwrap();assert!(checked.applied);
    let held=r.pause().unwrap();r.save().unwrap();r.save_slot("upper","Upper deck",true).unwrap();let saved=capture(&r);
    let before_disk=std::fs::read(crate::save_v6::save_path(&r.save_root)).unwrap();let prepared=r.continue_slot("upper").unwrap();
    assert_eq!(prepared.player.transform.position_m,held.player.transform.position_m);assert_eq!(prepared.server_time_ms,saved.save.server_time_ms);
    assert_eq!(prepared.support_scene.as_ref().unwrap().poses,held.support_scene.as_ref().unwrap().poses);
    assert_eq!(std::fs::read(crate::save_v6::save_path(&r.save_root)).unwrap(),before_disk);v=acknowledge_ready(&r,prepared);
    v=cross_gap(&r,v,10001);
    let before_drop=capture(&r).save.progression;
    v=walk(&r,v,19.,8.);for _ in 0..120{if v.player.transform.position_m.y_m==0.{break;}v=input(&r,&v,0.,0.);}
    assert_eq!(v.player.transform.position_m.y_m,0.);assert_eq!(v.player.current_hp,100);
    assert_eq!(v.support_scene.as_ref().unwrap().rider.mode,crate::moving_support::SupportRiderMode::Floor);
    v=walk(&r,v,22.7,8.);let denied=crate::scene_route_commands::transition(&r,"cw_gear_to_furnace_heart","gear-below-exit",v.world_epoch).unwrap();assert!(!denied.applied);
    assert_eq!(capture(&r).save.progression,before_drop);assert!(!v.capabilities.items.iter().any(|c|c.capability_id==CAP_AIR_STEP&&c.granted));
    v=ride_to_dock(&r,v,false);v=cross_gap(&r,v,10002);
    v=walk(&r,v,22.7,8.);assert!(v.player.current_hp>0);
    assert!(!v.capabilities.items.iter().any(|c|c.capability_id==CAP_AIR_STEP&&c.granted));
    let response=crate::scene_route_commands::transition(&r,"cw_gear_to_furnace_heart","gear-upper-exit",v.world_epoch).unwrap();assert!(response.applied);
    let entered=r.snapshot().unwrap();assert_eq!(entered.scene_id,"cw_furnace_heart");assert!(entered.entry_token.is_some());
    println!("Isolated Gear Shaft mechanics fixture: {} HP, actual lift+upper checkpoint+slot Continue+no-capability gap+height-bound transition",entered.player.current_hp);
}
