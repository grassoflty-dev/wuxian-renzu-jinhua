//! Public-runtime generic support fixture, separate from canonical campaign proof.
#[path="support/entry_readiness.rs"] mod entry_readiness;
use entry_readiness::acknowledge_ready;
use std::{collections::BTreeSet,path::PathBuf,time::{Duration,SystemTime,UNIX_EPOCH}};
use wuxian_horror_ch1::{continuous_input::InputSample,formal_runtime::{FormalRuntime,CombatIntentRequest},
    moving_support::{MovingSupportDefinition,SupportPhase},save_v6,scene_route_commands,world_v3::WorldView};
const RS:&str=include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH:&str=include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH:&str=include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW:&str=include_str!("fixtures/scene-runtime-v1/clockworks.json");
fn support()->MovingSupportDefinition {MovingSupportDefinition{id:"fixture_lift".into(),polygon:vec![[0.5,0.5],[5.5,0.5],[5.5,5.5],[0.5,5.5]],lower_m:0.,upper_m:2.,travel_ms:1000,endpoint_hold_ms:250}}
fn fixture(definitions:Vec<MovingSupportDefinition>,enemy:bool,live_owner:bool)->(FormalRuntime,PathBuf) {
    let root=std::env::temp_dir().join(format!("support-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let runtime=if live_owner {FormalRuntime::new_with_save_dir(root.clone()).unwrap()} else {
        #[cfg(feature="deterministic-replay")]
        { FormalRuntime::new_input_replay_with_save_dir(root.clone()).unwrap() }
        #[cfg(not(feature="deterministic-replay"))]
        { panic!("exact-step support tests require the existing deterministic-replay feature") }
    };runtime.pause().unwrap();
    let mut scene:serde_json::Value=serde_json::from_str(GH).unwrap();scene["movingSupports"]=serde_json::to_value(definitions).unwrap();let mut entities=BTreeSet::new();
    if enemy {let id="grey_hive.infected_maintenance_worker";entities.insert(id.to_owned());scene["spawns"].as_array_mut().unwrap().push(serde_json::json!({"id":"test_worker","kind":"enemy","entityType":id,"position":[2.,0.,1.]}));}
    runtime.load_scene_registry([RS.to_owned(),scene.to_string(),MH.to_owned(),CW.to_owned()],"gh_test_power",&BTreeSet::new(),&entities).unwrap();(runtime,root)
}
fn input(runtime:&FormalRuntime,view:&WorldView,axis:(f32,f32))->WorldView {runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,1000+view.ack_seq*17,axis.0,axis.1).unwrap(),vec![]).unwrap()}
fn idle(runtime:&FormalRuntime,view:&WorldView)->WorldView {input(runtime,view,(0.,0.))}
fn stable(runtime:&FormalRuntime)->WorldView {let before=runtime.snapshot().unwrap();std::thread::sleep(Duration::from_millis(45));assert_eq!(runtime.snapshot().unwrap(),before);before}
fn near(a:f32,b:f32) {assert!((a-b).abs()<0.0001,"{a} != {b}");}

#[test]
#[cfg(feature="deterministic-replay")]
fn all_four_phases_save_continue_and_entry_hold_preserve_exact_clock_pose() {
    for target in [SupportPhase::LowerHold,SupportPhase::Rising,SupportPhase::UpperHold,SupportPhase::Falling] {
        let s=support();let (runtime,root)=fixture(vec![s.clone()],false,false);let mut view=runtime.resume().unwrap();
        for _ in 0..160 {if s.pose_at(view.server_time_ms).unwrap().phase==target {break;}view=idle(&runtime,&view);}
        assert_eq!(s.pose_at(view.server_time_ms).unwrap().phase,target);runtime.save().unwrap();let saved=save_v6::read_save(&root).unwrap();let bytes=std::fs::read(root.join("formal-save-v6.json")).unwrap();
        let frame=saved.save.moving_support_frame.as_ref().unwrap();assert_eq!(frame.poses[0],s.pose_at(saved.save.server_time_ms).unwrap());
        let control=idle(&runtime,&view);let prepared=runtime.continue_saved().unwrap();assert_eq!(prepared.server_time_ms,view.server_time_ms);assert_eq!(prepared.player.transform.position_m,view.player.transform.position_m);stable(&runtime);
        let old_token=prepared.entry_token.clone().unwrap();let prepared2=runtime.continue_saved().unwrap();assert_eq!(runtime.scene_ready(&old_token,false).unwrap_err(),"E_SCENE_ENTRY_STALE_TOKEN");stable(&runtime);
        let ready=acknowledge_ready(&runtime,prepared2);let next=idle(&runtime,&ready);assert_eq!(next.server_time_ms,control.server_time_ms);near(next.player.transform.position_m.y_m,control.player.transform.position_m.y_m);
        assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),bytes);
    }
}

#[test]
#[cfg(feature="deterministic-replay")]
fn airborne_save_preserves_vertical_trajectory_for_regular_and_slot_continue() {
    for slot in [false,true] {for descending in [false,true] {
        let (runtime,root)=fixture(vec![support()],false,false);let view=runtime.resume().unwrap();
        let mut view=runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,1000,0.,0.).unwrap(),
            vec![CombatIntentRequest::Jump{request_id:1}]).unwrap();
        for _ in 0..40 {if (descending && view.player.velocity_mps.y_m<0.) || (!descending && view.player.velocity_mps.y_m>0.) {break;}view=idle(&runtime,&view);}
        assert!(view.player.transform.position_m.y_m>0.);assert_eq!(view.player.velocity_mps.y_m<0.,descending);
        if slot {runtime.save_slot("airborne","Airborne",true).unwrap();}else{runtime.save().unwrap();}
        let control=idle(&runtime,&view);let prepared=if slot {runtime.continue_slot("airborne").unwrap()}else{runtime.continue_saved().unwrap()};
        near(prepared.player.velocity_mps.y_m,view.player.velocity_mps.y_m);near(prepared.player.velocity_mps.x_m,0.);near(prepared.player.velocity_mps.z_m,0.);stable(&runtime);
        let ready=acknowledge_ready(&runtime,prepared);let next=idle(&runtime,&ready);assert_eq!(next.server_time_ms,control.server_time_ms);near(next.player.transform.position_m.y_m,control.player.transform.position_m.y_m);near(next.player.velocity_mps.y_m,control.player.velocity_mps.y_m);let _=root;
    }}
}

#[test]
#[cfg(feature="deterministic-replay")]
fn saved_dash_does_not_invent_a_default_direction_after_continue() {
    let (runtime,root)=fixture(vec![support()],false,false);let view=runtime.resume().unwrap();let view=input(&runtime,&view,(1.,0.));
    let _view=runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,1100,1.,0.).unwrap(),vec![CombatIntentRequest::Dash{request_id:2}]).unwrap();runtime.save().unwrap();let saved=save_v6::read_save(&root).unwrap();assert!(saved.save.player.dash_remaining_ms>0);let before=saved.save.player.position_m;
    let prepared=runtime.continue_saved().unwrap();let ready=acknowledge_ready(&runtime,prepared);let next=idle(&runtime,&ready);near(next.player.transform.position_m.x_m,before.x_m);near(next.player.transform.position_m.z_m,before.z_m);runtime.save().unwrap();let after=save_v6::read_save(&root).unwrap();assert_eq!(after.save.player.dash_remaining_ms,0);assert!(after.save.player.dash_cooldown_remaining_ms>0);
}

#[test]
#[cfg(feature="deterministic-replay")]
fn removed_changed_missing_and_stale_support_records_reject_atomically() {
    let (runtime,root)=fixture(vec![support()],false,false);let mut view=runtime.resume().unwrap();for _ in 0..45 {view=idle(&runtime,&view);}runtime.save().unwrap();let original=std::fs::read(root.join("formal-save-v6.json")).unwrap();assert!(view.player.transform.position_m.y_m>0.);
    let mut changed=support();changed.travel_ms=2000;
    let mut geometry=support();geometry.polygon[0][0]=0.6;
    for definitions in [vec![],vec![changed],vec![geometry]] {let (other,other_root)=fixture(definitions,false,false);std::fs::create_dir_all(&other_root).unwrap();std::fs::write(other_root.join("formal-save-v6.json"),&original).unwrap();let before=other.snapshot().unwrap();assert_eq!(other.continue_saved().unwrap_err(),"E_SAVE_SUPPORT_FRAME_INVALID");assert_eq!(other.snapshot().unwrap(),before);assert_eq!(std::fs::read(other_root.join("formal-save-v6.json")).unwrap(),original);}
    for mutation in ["missing","epoch","time","duplicate","phase","height","contact"] {
        let mut value:serde_json::Value=serde_json::from_slice(&original).unwrap();let frame=&mut value["save"]["movingSupportFrame"];
        match mutation {"missing"=>{value["save"].as_object_mut().unwrap().remove("movingSupportFrame");},"epoch"=>frame["worldEpoch"]=serde_json::json!(99),"time"=>frame["serverTimeMs"]=serde_json::json!(999),"duplicate"=>{let p=frame["poses"][0].clone();frame["poses"].as_array_mut().unwrap().push(p);},"phase"=>frame["poses"][0]["phase"]=serde_json::json!("falling"),"height"=>frame["poses"][0]["heightM"]=serde_json::json!(2.9),"contact"=>value["save"]["player"]["positionM"]["xM"]=serde_json::json!(6.0),_=>unreachable!()}
        let bytes=serde_json::to_vec(&value).unwrap();std::fs::write(root.join("formal-save-v6.json"),&bytes).unwrap();let before=runtime.snapshot().unwrap();let error=runtime.continue_saved().unwrap_err();if mutation=="contact" {assert_eq!(error,"E_SAVE_SUPPORT_CONTACT_INVALID");}assert_eq!(runtime.snapshot().unwrap(),before);assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),bytes);
    }
}

#[test]
fn pause_entry_new_and_scene_transition_do_not_tick_or_retain_support_motion() {
    let (runtime,_)=fixture(vec![support()],false,true);stable(&runtime);let mut view=runtime.resume().unwrap();for _ in 0..45 {view=idle(&runtime,&view);}let paused=runtime.pause().unwrap();stable(&runtime);let resumed=runtime.resume().unwrap();assert_eq!(resumed.server_time_ms,paused.server_time_ms);let moved=idle(&runtime,&resumed);near(moved.player.transform.position_m.y_m,support().pose_at(moved.server_time_ms).unwrap().height_m);
    let receipt=scene_route_commands::transition(&runtime,"gh_to_hub","leave-support",moved.world_epoch).unwrap();assert!(receipt.applied);let view=acknowledge_ready(&runtime,runtime.snapshot().unwrap());let next=idle(&runtime,&view);assert_eq!(next.player.transform.position_m,view.player.transform.position_m);
    let prepared=runtime.reset_new().unwrap();stable(&runtime);let ready=acknowledge_ready(&runtime,prepared);let next=idle(&runtime,&ready);assert_eq!(next.player.transform.position_m,ready.player.transform.position_m);
}

#[test]
fn actual_enemy_death_freezes_support_phase_and_rejects_save() {
    let (runtime,_)=fixture(vec![support()],true,true);let mut view=runtime.resume().unwrap();
    for _ in 0..2400 {if view.player.current_hp==0 {break;}match runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,1000+view.ack_seq*17,0.,0.).unwrap(),vec![]) {Ok(next)=>view=next,Err(e) if e=="E_RUNTIME_DEAD"=>{view=runtime.snapshot().unwrap();break;},Err(e)=>panic!("{e}")}}
    assert_eq!(view.player.current_hp,0);assert!(runtime.presentation_events_since(view.world_epoch,0).unwrap().iter().any(|e|e.kind=="Damaged"));stable(&runtime);assert_eq!(runtime.save().unwrap_err(),"E_RUNTIME_DEAD");
}

#[test]
fn existing_no_support_save_omits_frame_and_legacy_profile_rejects_new_key_presence() {
    let (runtime,root)=fixture(vec![],false,true);runtime.save().unwrap();
    let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("formal-save-v6.json")).unwrap()).unwrap();
    assert!(!raw["save"].as_object().unwrap().contains_key("movingSupportFrame"));
    let prepared=runtime.continue_saved().unwrap();acknowledge_ready(&runtime,prepared);runtime.pause().unwrap();
    let frozen=include_str!("fixtures/capability-save-profiles/legacy-empty.json");
    std::fs::write(root.join("formal-save-v6.json"),frozen).unwrap();assert!(save_v6::read_save(&root).is_ok());
    for new_value in [serde_json::Value::Null,serde_json::json!({}),serde_json::json!({"poses":[]})] {
        let mut value:serde_json::Value=serde_json::from_str(frozen).unwrap();value["save"]["movingSupportFrame"]=new_value;
        let bytes=serde_json::to_vec(&value).unwrap();std::fs::write(root.join("formal-save-v6.json"),&bytes).unwrap();
        assert_eq!(save_v6::read_save(&root).unwrap_err(),"E_SAVE_SCHEMA_INVALID");assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),bytes);
    }
}

#[test]
#[cfg(feature="deterministic-replay")]
fn save_continue_straddles_each_endpoint_without_skipping_a_phase() {
    for boundary in [250,1250,1500,2500] {
        let s=support();let (runtime,root)=fixture(vec![s.clone()],false,false);let mut view=runtime.resume().unwrap();
        while view.server_time_ms+17<boundary {view=idle(&runtime,&view);}
        runtime.save().unwrap();let saved=save_v6::read_save(&root).unwrap();assert!(saved.save.server_time_ms<boundary);
        let control=idle(&runtime,&view);assert!(control.server_time_ms>=boundary);
        let prepared=runtime.continue_saved().unwrap();let ready=acknowledge_ready(&runtime,prepared);let next=idle(&runtime,&ready);
        assert_eq!(next.server_time_ms,control.server_time_ms);near(next.player.transform.position_m.y_m,control.player.transform.position_m.y_m);
        assert_eq!(s.pose_at(next.server_time_ms).unwrap().phase,s.pose_at(control.server_time_ms).unwrap().phase);
    }
}

#[test]
fn malformed_support_scene_cannot_replace_live_registry() {
    let (runtime,_)=fixture(vec![support()],false,true);let before=runtime.snapshot().unwrap();
    for mutation in ["duplicate","shared_id","timing","geometry"] {
        let mut scene:serde_json::Value=serde_json::from_str(GH).unwrap();let mut definitions=vec![support()];
        match mutation {"duplicate"=>definitions.push(support()),"shared_id"=>definitions[0].id="gh_spawn".into(),"timing"=>definitions[0].travel_ms=0,"geometry"=>definitions[0].polygon[0]=[13.,1.],_=>unreachable!()}
        scene["movingSupports"]=serde_json::to_value(definitions).unwrap();assert!(runtime.load_scene_registry([RS.to_owned(),scene.to_string(),MH.to_owned(),CW.to_owned()],"gh_test_power",&BTreeSet::new(),&BTreeSet::new()).is_err());assert_eq!(runtime.snapshot().unwrap(),before);
    }
}

#[test]
#[cfg(feature="deterministic-replay")]
fn airborne_out_of_envelope_write_and_file_slot_continue_are_atomic() {
    let (runtime,root)=fixture(vec![support()],false,false);runtime.save().unwrap();
    let saved=save_v6::read_save(&root).unwrap();let valid_bytes=std::fs::read(root.join("formal-save-v6.json")).unwrap();
    runtime.save_slot("envelope","Envelope",true).unwrap();
    let slot_path=root.join("slots/envelope/slot-v6.json");let valid_slot=std::fs::read(&slot_path).unwrap();
    for y in [-0.1,3.0001,99.] {
        let mut malformed=saved.clone();malformed.save.player.grounded=false;malformed.save.player.position_m.y_m=y;
        assert_eq!(save_v6::write_save(&root,&malformed).unwrap_err(),"E_SAVE_SUPPORT_CONTACT_INVALID");
        assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),valid_bytes);
        assert_eq!(wuxian_horror_ch1::save_slots::overwrite_slot_v6(&root,"envelope","Envelope",&malformed).unwrap_err(),"E_SAVE_SUPPORT_CONTACT_INVALID");
        assert_eq!(std::fs::read(&slot_path).unwrap(),valid_slot);
        // A corrupt external file bypasses the writer; loading must still fail
        // before changing the live owner or repairing any bytes on disk.
        let file_bytes=serde_json::to_vec(&malformed).unwrap();std::fs::write(root.join("formal-save-v6.json"),&file_bytes).unwrap();
        let before=runtime.snapshot().unwrap();assert_eq!(runtime.continue_saved().unwrap_err(),"E_SAVE_SUPPORT_CONTACT_INVALID");assert_eq!(runtime.snapshot().unwrap(),before);assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),file_bytes);
        std::fs::write(root.join("formal-save-v6.json"),&valid_bytes).unwrap();
        let mut wrapper:serde_json::Value=serde_json::from_slice(&valid_slot).unwrap();wrapper["save"]=serde_json::to_value(&malformed).unwrap();let slot_bytes=serde_json::to_vec(&wrapper).unwrap();std::fs::write(&slot_path,&slot_bytes).unwrap();
        assert_eq!(runtime.continue_slot("envelope").unwrap_err(),"E_SAVE_SUPPORT_CONTACT_INVALID");assert_eq!(runtime.snapshot().unwrap(),before);assert_eq!(std::fs::read(&slot_path).unwrap(),slot_bytes);std::fs::write(&slot_path,&valid_slot).unwrap();
    }
}
