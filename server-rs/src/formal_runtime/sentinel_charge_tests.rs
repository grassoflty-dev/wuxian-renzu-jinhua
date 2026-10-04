//! Arranged native-owner/save fixtures, not evidence of a complete player journey.
use super::*;
use crate::{sentinel_ai::{Sentinel,SentinelState},save_v6::SaveV6,world_v3::Vec3};
use super::entry_test_support::acknowledge_ready;
fn p(x:f32,z:f32)->Vec3{Vec3::new(x,0.,z).unwrap()}
fn fixture()->FormalRuntime {
 let root=std::env::temp_dir().join(format!("sentinel-charge-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
 let r=FormalRuntime::new_with_save_dir(root).unwrap();r.pause().unwrap();r.stop_owner.store(true,Ordering::SeqCst);r.owner_handle.lock().unwrap().take().unwrap().join().unwrap();crate::production_scene_bootstrap::install(&r).unwrap();let registry=r.scene_registry.lock().unwrap().clone().unwrap();r.install_scene_registry(registry,SENTINEL_SCENE_ID).unwrap();
 {let mut s=r.state.lock().unwrap();s.paused=false;s.world.sentinels[0].position_m=p(8.,8.);s.world.player.position_m=p(11.,8.);}r
}
fn actor(r:&FormalRuntime)->Sentinel{r.state.lock().unwrap().world.sentinels[0].clone()}
fn capture(r:&FormalRuntime)->SaveV6{SaveV6::capture(&r.state.lock().unwrap(),r.scene_runtime.lock().unwrap().as_ref()).unwrap()}
fn phase(r:&FormalRuntime,active:bool,consumed:bool)->Sentinel {
 let mut s=r.state.lock().unwrap();let k=s.kcc.clone();let player=s.world.player.position_m;let a=&mut s.world.sentinels[0];crate::sentinel_ai::tick_sentinel(a,player,1./60.,&k);assert_eq!(a.state,SentinelState::ChargeWindup);
 if active {while a.state!=SentinelState::Charge{crate::sentinel_ai::tick_sentinel(a,player,1./60.,&k);}if consumed {while !a.charge_controller.as_ref().unwrap().attack.as_ref().unwrap().hit_resolved{crate::sentinel_ai::tick_sentinel(a,player,1./60.,&k);}}}else{a.state_remaining_ms=1;}
 assert!(a.validate());a.clone()
}
#[test] fn sentinel_charge_one_tick_windup_zero_elapsed_and_consumed_continue_hold_exact_state(){
 for (active,consumed) in [(false,false),(true,false),(true,true)] {let r=fixture();let expected=phase(&r,active,consumed);let projection=r.snapshot().unwrap().sentinel_encounter.unwrap();r.save().unwrap();r.save_slot("phase","Phase",true).unwrap();let path=crate::save_v6::save_path(&r.save_root);let slot=r.save_root.join("slots/phase/slot-v6.json");let raw=std::fs::read(&path).unwrap();let raw_slot=std::fs::read(&slot).unwrap();
 for slot_load in [false,true] {let prepared=if slot_load{r.continue_slot("phase").unwrap()}else{r.continue_saved().unwrap()};assert!(prepared.entry_token.is_some()&&r.state.lock().unwrap().paused);assert_eq!(prepared.sentinel_encounter,Some(projection.clone()));assert_eq!(actor(&r),expected);acknowledge_ready(&r,prepared);assert_eq!(actor(&r),expected);
 let v=r.snapshot().unwrap();let context=SessionContext{world_id:v.world_id,scene_id:v.scene_id,world_epoch:v.world_epoch};r.pause_context_ordered(&context,1).unwrap();let held=r.pause_context_ordered(&context,2).unwrap();assert_eq!(held.sentinel_encounter,Some(projection.clone()));assert_eq!(actor(&r),expected);r.resume_context_ordered(&context,3).unwrap();assert_eq!(actor(&r),expected);assert_eq!(std::fs::read(&path).unwrap(),raw);assert_eq!(std::fs::read(&slot).unwrap(),raw_slot);}
 }
}
#[test] fn sentinel_charge_legacy_file_slot_upgrade_is_clone_only_and_preserves_old_phases(){
 for old_phase in [SentinelState::Attack,SentinelState::HeavyAttack,SentinelState::Hit,SentinelState::Death]{let r=fixture();{let mut s=r.state.lock().unwrap();let a=&mut s.world.sentinels[0];a.charge_controller=None;a.state=old_phase;a.hp=if old_phase==SentinelState::Death{0}else{75};a.active=a.hp>0;a.state_remaining_ms=if a.hp>0{1}else{0};a.attack_serial=11;}
 let old=capture(&r);r.save().unwrap();r.save_slot("old","Old",true).unwrap();let path=crate::save_v6::save_path(&r.save_root);let raw=std::fs::read(&path).unwrap();let slot=r.save_root.join("slots/old/slot-v6.json");let raw_slot=std::fs::read(&slot).unwrap();assert!(!String::from_utf8_lossy(&raw).contains("chargeController"));
 for slot_load in [false,true]{let prepared=if slot_load{r.continue_slot("old").unwrap()}else{r.continue_saved().unwrap()};assert!(prepared.entry_token.is_some());let a=actor(&r);assert_eq!((a.hp,a.active,a.state,a.state_remaining_ms,a.attack_serial),(old.save.actors[0].hp,old.save.actors[0].active,old_phase,old.save.actors[0].state_remaining_ms,11));assert_eq!(a.charge_controller,Some(Default::default()));assert!(!r.state.lock().unwrap().world_persistent_v1.grey_hive.sentinel_first_kill);acknowledge_ready(&r,prepared);assert_eq!(std::fs::read(&path).unwrap(),raw);assert_eq!(std::fs::read(&slot).unwrap(),raw_slot);}
 }
}
#[test] fn sentinel_charge_malformed_current_files_slots_and_writes_preserve_authority(){
 let r=fixture();phase(&r,false,false);let good=capture(&r);r.save().unwrap();r.save_slot("bad","Bad",true).unwrap();let path=crate::save_v6::save_path(&r.save_root);let slot=r.save_root.join("slots/bad/slot-v6.json");let raw=std::fs::read(&path).unwrap();let raw_slot=std::fs::read(&slot).unwrap();
 for fault in ["null","version","missing-attack","bad-dir","elapsed","consumed","remaining","missing-controller","long","shift","light-timer","recover-timer","chase-timer"] {let mut value=serde_json::to_value(&good).unwrap();let row=&mut value["save"]["actors"][0];match fault {
 "null"=>row["chargeController"]=serde_json::Value::Null,"version"=>row["chargeController"]["schemaVersion"]=serde_json::json!(2),"missing-attack"=>{row["chargeController"].as_object_mut().unwrap().remove("attack");},"bad-dir"=>row["chargeController"]["attack"]["directionM"]["xM"]=serde_json::json!(2.),"elapsed"=>row["chargeController"]["attack"]["elapsedMs"]=serde_json::json!(1),"consumed"=>row["chargeController"]["attack"]["hitResolved"]=serde_json::json!(true),"remaining"=>row["stateRemainingMs"]=serde_json::json!(901),"missing-controller"=>{row.as_object_mut().unwrap().remove("chargeController");},"long"=>row["chargeController"]["attack"]["travelM"]=serde_json::json!(3.7),"light-timer"|"recover-timer"|"chase-timer"=>{row["state"]=serde_json::json!(if fault=="light-timer"{"attack"}else if fault=="recover-timer"{"recover"}else{"chase"});row["stateRemainingMs"]=serde_json::json!(6000);row["chargeController"]["attack"]=serde_json::Value::Null;},_=>row["positionM"]["xM"]=serde_json::json!(9.)}
 if let Ok(bad)=serde_json::from_value::<SaveV6>(value.clone()){assert!(crate::save_v6::write_save(&r.save_root,&bad).is_err(),"{fault}");assert!(crate::save_slots::overwrite_slot_v6(&r.save_root,"bad","Bad",&bad).is_err());assert_eq!(std::fs::read(&path).unwrap(),raw);assert_eq!(std::fs::read(&slot).unwrap(),raw_slot);}
 let bad=serde_json::to_vec(&value).unwrap();std::fs::write(&path,&bad).unwrap();let mut wrapper:serde_json::Value=serde_json::from_slice(&raw_slot).unwrap();wrapper["save"]=value;let bad_slot=serde_json::to_vec(&wrapper).unwrap();std::fs::write(&slot,&bad_slot).unwrap();let before=capture(&r);assert!(r.continue_saved().is_err(),"{fault}");assert!(r.continue_slot("bad").is_err(),"{fault}");assert_eq!(capture(&r),before);assert_eq!(std::fs::read(&path).unwrap(),bad);assert_eq!(std::fs::read(&slot).unwrap(),bad_slot);std::fs::write(&path,&raw).unwrap();std::fs::write(&slot,&raw_slot).unwrap();}
}
#[test] fn sentinel_charge_late_entry_failure_and_stale_context_do_not_change_phase(){
 let r=fixture();phase(&r,false,false);r.save().unwrap();let path=crate::save_v6::save_path(&r.save_root);let raw=std::fs::read(&path).unwrap();let before=capture(&r);r.state.lock().unwrap().entry_generation=build_ui::MAX_SAFE_REVISION;assert_eq!(r.continue_saved().unwrap_err(),"E_SCENE_ENTRY_GENERATION_EXHAUSTED");assert_eq!(capture(&r),before);assert_eq!(std::fs::read(&path).unwrap(),raw);r.state.lock().unwrap().entry_generation=0;let old=r.snapshot().unwrap();let prepared=r.continue_saved().unwrap();let expected=actor(&r);let bad=SessionContext{world_id:old.world_id,scene_id:old.scene_id,world_epoch:old.world_epoch};assert!(r.resume_context_ordered(&bad,1).is_err());assert_eq!(actor(&r),expected);assert!(r.state.lock().unwrap().paused);acknowledge_ready(&r,prepared);
}
#[test] fn sentinel_charge_projection_metadata_match_all_native_windup_kinds_and_source(){
 for (state,kind)in [(SentinelState::Chase,"SentinelChargeWindup"),(SentinelState::Chase,"SentinelAttackWindup"),(SentinelState::Attack,"SentinelHeavyWindup")] {let r=fixture();{let mut s=r.state.lock().unwrap();if kind!="SentinelChargeWindup" {s.world.player.position_m=p(9.,8.);}s.world.sentinels[0].state=state;s.world.sentinels[0].attack_serial=2;let scene=r.scene_runtime.lock().unwrap();advance_owner_step_with_scene(&mut s,scene.as_ref()).unwrap();}
 let v=r.snapshot().unwrap();let projected=v.sentinel_encounter.unwrap();let w=projected.warning.unwrap();let events=r.presentation_events_since(v.world_epoch,0).unwrap();let e=events.iter().find(|e|e.kind==kind).unwrap();assert_eq!(e.actor_id.as_deref(),Some(SENTINEL_SPAWN_ID));assert_eq!(e.attack_id,Some(projected.attack_serial));assert_eq!(e.duration_ms,Some(projected.remaining_ms));assert_eq!((e.position_m,e.direction_rad,e.radius_m,e.range_m),(w.origin_m,w.direction_rad,w.radius_m,Some(w.range_m)));
 r.scene_runtime.lock().unwrap().as_mut().unwrap().world_epoch+=1;assert!(r.snapshot().unwrap().sentinel_encounter.is_none());}
}

#[test] fn sentinel_legacy_due_melee_and_long_hit_keep_authority_time_through_held_continue(){
 for phase in [SentinelState::Attack,SentinelState::HeavyAttack,SentinelState::Hit] {let r=fixture();{let mut s=r.state.lock().unwrap();let a=&mut s.world.sentinels[0];a.charge_controller=None;a.state=phase;a.state_remaining_ms=if phase==SentinelState::Hit{250}else{0};}r.save().unwrap();let before=capture(&r);let view=r.continue_saved().unwrap();assert!(r.state.lock().unwrap().paused);let a=actor(&r);assert_eq!(a.state_remaining_ms,before.save.actors[0].state_remaining_ms);let projection=view.sentinel_encounter.as_ref().unwrap();if phase==SentinelState::Hit{assert_eq!(a.state,SentinelState::Stagger);assert_eq!(projection.phase,"stagger");assert!(projection.warning.is_none());}else{assert_eq!(a.state,phase);assert_eq!(projection.remaining_ms,17);assert!(projection.warning.is_some());}acknowledge_ready(&r,view);assert_eq!(actor(&r),a);}
}

#[test] fn sentinel_native_dynamic_obstacle_preserves_required_projection_until_real_stop(){
 let r=fixture();phase(&r,true,false);let hp=r.state.lock().unwrap().world.player_hp;
 r.state.lock().unwrap().kcc.walls.push(crate::continuous_kcc::Aabb::new(9.,9.001,7.,9.).unwrap());
 for _ in 0..20 {let scene=r.scene_runtime.lock().unwrap().clone().unwrap();{let mut s=r.state.lock().unwrap();advance_owner_step_with_scene(&mut s,Some(&scene)).unwrap();}let view=r.snapshot().unwrap();assert!(view.sentinel_encounter.is_some());assert_eq!(view.player.current_hp,hp);if actor(&r).state==SentinelState::Recover {break;}}
 assert_eq!(actor(&r).state,SentinelState::Recover);assert!(actor(&r).position_m.x_m<9.);assert!(r.snapshot().unwrap().sentinel_encounter.unwrap().warning.is_none());
}

#[test] fn old_inactive_living_sentinel_file_and_slot_keep_raw_phase_but_project_inactive(){
    let mut wrong=Vec::new();
    for phase in [SentinelState::Chase,SentinelState::Attack,SentinelState::HeavyAttack,SentinelState::Recover,SentinelState::Hit] {
        let r=fixture();
        {let mut s=r.state.lock().unwrap();let a=&mut s.world.sentinels[0];a.charge_controller=None;a.state=phase;a.state_remaining_ms=if phase==SentinelState::Chase{0}else{1};a.hp=75;
            crate::world_v3::apply_sentinel(&mut s.world,crate::sentinel_ai::SentinelCommand::Deactivate{sentinel_id:SENTINEL_SPAWN_ID.into()}).unwrap();}
        let old=capture(&r);r.save().unwrap();r.save_slot("inactive","Inactive",true).unwrap();let path=crate::save_v6::save_path(&r.save_root);let slot=r.save_root.join("slots/inactive/slot-v6.json");let raw=std::fs::read(&path).unwrap();let raw_slot=std::fs::read(&slot).unwrap();
        assert!(!String::from_utf8_lossy(&raw).contains("chargeController"));
        for slot_load in [false,true] {let prepared=if slot_load{r.continue_slot("inactive").unwrap()}else{r.continue_saved().unwrap()};assert!(prepared.entry_token.is_some()&&r.state.lock().unwrap().paused);let a=actor(&r);assert_eq!((a.hp,a.active,a.state,a.state_remaining_ms,a.attack_serial),(75,false,phase,old.save.actors[0].state_remaining_ms,old.save.actors[0].attack_serial));assert!(!r.state.lock().unwrap().world_persistent_v1.grey_hive.sentinel_first_kill);
            let public=prepared.sentinel_encounter.as_ref().unwrap();if public.phase!="inactive" {wrong.push(format!("phase={phase:?}, slot={slot_load}, projected={}",public.phase));} else {assert_eq!(public.remaining_ms,0);assert!(public.warning.is_none());}acknowledge_ready(&r,prepared);assert_eq!(actor(&r),a);assert_eq!(std::fs::read(&path).unwrap(),raw);assert_eq!(std::fs::read(&slot).unwrap(),raw_slot);}
    }
    assert!(wrong.is_empty(),"incompatible living-inactive projections: {wrong:?}");
}

#[test] fn inactive_current_charge_retains_commitment_and_reactivation_restores_same_warning(){
    let r=fixture();let committed=phase(&r,true,false);let warning=r.snapshot().unwrap().sentinel_encounter.unwrap();
    {let mut s=r.state.lock().unwrap();crate::world_v3::apply_sentinel(&mut s.world,crate::sentinel_ai::SentinelCommand::Deactivate{sentinel_id:SENTINEL_SPAWN_ID.into()}).unwrap();}
    let inactive=actor(&r);r.save().unwrap();let view=r.continue_saved().unwrap();let projection=view.sentinel_encounter.as_ref().unwrap();assert_eq!(projection.phase,"inactive");assert!(projection.warning.is_none());assert_eq!(projection.remaining_ms,0);assert_eq!(actor(&r),inactive);acknowledge_ready(&r,view);
    {let mut s=r.state.lock().unwrap();let k=s.kcc.clone();let player=s.world.player.position_m;let out=crate::sentinel_ai::tick_sentinel(&mut s.world.sentinels[0],player,1./60.,&k);assert!(out.0.is_empty()&&out.1.is_empty());assert_eq!(s.world.sentinels[0],inactive);
        crate::world_v3::apply_sentinel(&mut s.world,crate::sentinel_ai::SentinelCommand::Activate{sentinel_id:SENTINEL_SPAWN_ID.into()}).unwrap();}
    assert_eq!(actor(&r),committed);assert_eq!(r.snapshot().unwrap().sentinel_encounter,Some(warning));assert!(!r.state.lock().unwrap().world_persistent_v1.grey_hive.sentinel_first_kill);
}
