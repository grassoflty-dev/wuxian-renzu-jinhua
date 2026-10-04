//! Public generic fixture proof only. A 2m deck, 1m dock overlap, and the
//! explicit bands below are development-test tuning, not canonical Gear Shaft.
#[path="support/entry_readiness.rs"] mod entry_readiness;
use entry_readiness::acknowledge_ready;
use std::{collections::BTreeSet,path::PathBuf,time::{Duration,SystemTime,UNIX_EPOCH}};
use wuxian_horror_ch1::{continuous_input::InputSample,formal_runtime::{FormalRuntime,ActionCommandRequest,ActionKind,CombatIntentRequest},save_v6,scene_route_commands,world_v3::WorldView};
static NEXT_CLIENT_TIME:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(1000);
fn next_time()->u64 {NEXT_CLIENT_TIME.fetch_add(17,std::sync::atomic::Ordering::Relaxed)}
const RS:&str=include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH:&str=include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH:&str=include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW:&str=include_str!("fixtures/scene-runtime-v1/clockworks.json");
fn scene(height:f32,deck:bool)->serde_json::Value {
    let mut value:serde_json::Value=serde_json::from_str(GH).unwrap();let y=if deck {height}else{0.};let band=if deck {serde_json::json!([height-0.1,height+0.1])}else{serde_json::json!([0.,0.1])};
    value["spawns"][0]["position"]=serde_json::json!([4.,0.,2.]);value["navigation"]["nodes"][0]["position"]=serde_json::json!([4.,0.,2.]);value["navigation"]["nodes"][1]["position"]=serde_json::json!([4.,y,2.]);
    value["movingSupports"]=serde_json::json!([{"id":"fixture_lift","polygon":[[1.,1.],[4.,1.],[4.,4.],[1.,4.]],"lowerM":0.,"upperM":2.,"travelMs":1000,"endpointHoldMs":750}]);
    if deck {value["standingDecks"]=serde_json::json!([{"id":"fixture_deck","polygon":[[3.,1.],[6.,1.],[6.,4.],[3.,4.]],"heightM":height}]);}
    for item in value["interactions"].as_array_mut().unwrap() {item["heightRangeM"]=serde_json::json!([0.,0.1]);}
    value["interactions"][0]["position"]=serde_json::json!([4.,y,2.]);value["interactions"][0]["heightRangeM"]=band.clone();
    value["checkpoints"][0]["position"]=serde_json::json!([4.,y,2.]);value["checkpoints"][0]["heightRangeM"]=band.clone();
    let polygon=serde_json::json!([[3.5,1.5],[4.5,1.5],[4.5,2.5],[3.5,2.5]]);
    value["triggers"]=serde_json::json!([{"id":"upper_trigger","event":"hive_lockdown","polygon":polygon,"heightRangeM":band}]);
    value["transitions"][0]["polygon"]=polygon.clone();value["transitions"][0]["heightRangeM"]=band.clone();
    value["doors"]=serde_json::json!([{"id":"upper_door","toSceneId":"rs_test_hub","spawnId":"rs_spawn","position":[4.,y,2.],"assetId":"fixture.door","heightRangeM":band}]);
    value["logic"]["traversal"]=serde_json::json!([{"id":"deck_vault","from":[4.,y,2.],"to":[5.,y,2.],"rangeM":1.2,"cooldownMs":350,"requiredCapabilities":[],"fromHeightRangeM":band}]);value
}
fn fixture(live:bool,definition:serde_json::Value)->(FormalRuntime,PathBuf) {fixture_extra(live,definition,vec![])}
fn fixture_extra(live:bool,definition:serde_json::Value,extras:Vec<serde_json::Value>)->(FormalRuntime,PathBuf) {
    let root=std::env::temp_dir().join(format!("standing-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let runtime=if live {FormalRuntime::new_with_save_dir(root.clone()).unwrap()}else {
        #[cfg(feature="deterministic-replay")] {FormalRuntime::new_input_replay_with_save_dir(root.clone()).unwrap()}
        #[cfg(not(feature="deterministic-replay"))] {panic!("exact fixtures require deterministic-replay")}
    };runtime.pause().unwrap();
    let mut scenes=vec![RS.to_owned(),definition.to_string(),MH.to_owned(),CW.to_owned()];scenes.extend(extras.into_iter().map(|v|v.to_string()));
    runtime.load_scene_registry(scenes,"gh_test_power",&BTreeSet::from(["fixture.door".to_owned()]),&BTreeSet::new()).unwrap();(runtime,root)
}
fn load(runtime:&FormalRuntime,definition:&serde_json::Value)->Result<(),String> {
    runtime.load_scene_registry([RS.to_owned(),definition.to_string(),MH.to_owned(),CW.to_owned()],"gh_test_power",&BTreeSet::from(["fixture.door".to_owned()]),&BTreeSet::new()).map(|_|())
}
fn input(runtime:&FormalRuntime,view:&WorldView,x:f32)->WorldView {runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,next_time(),x,0.).unwrap(),vec![]).unwrap()}
fn idle(runtime:&FormalRuntime,view:&WorldView)->WorldView {input(runtime,view,0.)}
fn near(a:f32,b:f32) {assert!((a-b).abs()<0.001,"{a} != {b}");}
fn stable(runtime:&FormalRuntime)->WorldView {let before=runtime.snapshot().unwrap();std::thread::sleep(Duration::from_millis(40));assert_eq!(runtime.snapshot().unwrap(),before);before}
fn ride(runtime:&FormalRuntime,mut view:WorldView)->WorldView {
    for _ in 0..80 {if view.player.transform.position_m.x_m<=2.25 {break;}view=input(runtime,&view,-1.);}
    assert!(view.player.transform.position_m.x_m<=2.25);
    for _ in 0..650 {if (view.player.transform.position_m.y_m-2.).abs()<0.001 {break;}view=idle(runtime,&view);}
    near(view.player.transform.position_m.y_m,2.);
    for _ in 0..80 {if view.player.transform.position_m.x_m>=3.6 {break;}view=input(runtime,&view,1.);}
    assert!(view.player.transform.position_m.x_m>=3.6);near(view.player.transform.position_m.y_m,2.);
    for _ in 0..110 {view=idle(runtime,&view);near(view.player.transform.position_m.y_m,2.);}
    view
}
fn upper(runtime:&FormalRuntime,mut view:WorldView)->WorldView {for _ in 0..20 {if view.player.transform.position_m.x_m>=3.95 {break;}view=input(runtime,&view,1.);}view}

#[test]
#[cfg(feature="deterministic-replay")]
fn raised_commands_and_affordances_reject_same_xz_below_then_real_lift_reaches_deck() {
    let (runtime,_)=fixture(false,scene(2.,true));let before=runtime.resume().unwrap();near(before.player.transform.position_m.y_m,0.);
    for id in ["gh_power_console_test","gh_checkpoint_test","upper_trigger","gh_to_hub"] {assert!(!before.interactables.iter().find(|item|item.entity_id==id).unwrap().active,"{id}");}
    assert!(!scene_route_commands::interaction(&runtime,"gh_power_console_test","below-console",Some(before.world_epoch)).unwrap().applied);
    assert!(!scene_route_commands::checkpoint(&runtime,"gh_checkpoint_test","below-checkpoint",before.world_epoch).unwrap().applied);
    assert!(!scene_route_commands::trigger(&runtime,"upper_trigger","below-trigger",before.world_epoch).unwrap().applied);
    for id in ["gh_to_hub","upper_door"] {assert!(!scene_route_commands::transition(&runtime,id,&format!("below-{id}"),before.world_epoch).unwrap().applied);}
    assert_eq!(runtime.snapshot().unwrap(),before);
    let view=upper(&runtime,ride(&runtime,before));near(view.player.transform.position_m.y_m,2.);
    for id in ["gh_power_console_test","gh_checkpoint_test","upper_trigger","gh_to_hub"] {assert!(view.interactables.iter().find(|item|item.entity_id==id).unwrap().active,"{id}");}
    assert!(scene_route_commands::interaction(&runtime,"gh_power_console_test","above-console",Some(view.world_epoch)).unwrap().applied);
    assert!(scene_route_commands::checkpoint(&runtime,"gh_checkpoint_test","above-checkpoint",view.world_epoch).unwrap().applied);
    assert!(scene_route_commands::trigger(&runtime,"upper_trigger","above-trigger",view.world_epoch).unwrap().applied);
    assert!(scene_route_commands::transition(&runtime,"gh_to_hub","above-exit",view.world_epoch).unwrap().applied);
    let prepared=runtime.snapshot().unwrap();stable(&runtime);let ready=acknowledge_ready(&runtime,prepared);assert_eq!(ready.scene_id,"rs_test_hub");near(ready.player.transform.position_m.y_m,0.);let next=idle(&runtime,&ready);assert_eq!(next.player.transform.position_m,ready.player.transform.position_m);
}

#[test]
#[cfg(feature="deterministic-replay")]
fn normal_jump_and_dash_from_lower_floor_do_not_open_raised_exit() {
    let (runtime,_)=fixture(false,scene(2.,true));let view=runtime.resume().unwrap();
    let mut view=runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,next_time(),0.,0.).unwrap(),vec![CombatIntentRequest::Jump{request_id:1}]).unwrap();
    let mut max_y=0.0_f32;
    for _ in 0..75 {max_y=max_y.max(view.player.transform.position_m.y_m);assert!(!view.interactables.iter().find(|item|item.entity_id=="gh_to_hub").unwrap().active);view=idle(&runtime,&view);}
    assert!(max_y<1.0);near(view.player.transform.position_m.y_m,0.);
    view=runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,next_time(),1.,0.).unwrap(),vec![CombatIntentRequest::Dash{request_id:2}]).unwrap();near(view.player.transform.position_m.y_m,0.);assert!(!scene_route_commands::transition(&runtime,"gh_to_hub","dash-cannot-exit",view.world_epoch).unwrap().applied);
}

#[test]
fn live_context_traversal_rejects_below_then_finishes_on_actual_supported_deck() {
    let (runtime,_)=fixture(true,scene(2.,true));let view=runtime.resume().unwrap();
    assert_eq!(runtime.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:view.world_epoch,request_id:1,client_time_ms:next_time(),kind:ActionKind::ContextTraversal}).unwrap_err(),"E_TRAVERSAL_OUT_OF_RANGE");
    let view=upper(&runtime,ride(&runtime,runtime.snapshot().unwrap()));
    let mut view=runtime.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:view.world_epoch,request_id:2,client_time_ms:next_time(),kind:ActionKind::ContextTraversal}).unwrap();
    let mut completed=false;
    for _ in 0..80 {if let Some(event)=runtime.presentation_events_since(view.world_epoch,0).unwrap().iter().find(|event|event.kind=="ContextTraversalCompleted") {near(event.position_m.x_m,5.);near(event.position_m.y_m,2.);completed=true;break;}view=idle(&runtime,&view);}
    assert!(completed);for _ in 0..40 {view=idle(&runtime,&view);near(view.player.transform.position_m.y_m,2.);}
    runtime.pause().unwrap();stable(&runtime);
}

#[test]
#[cfg(feature="deterministic-replay")]
fn standing_checkpoint_trigger_and_both_save_routes_restore_without_phase_drift() {
    let (runtime,root)=fixture(false,scene(2.,true));let initial=runtime.resume().unwrap();let view=upper(&runtime,ride(&runtime,initial));
    assert!(scene_route_commands::checkpoint(&runtime,"gh_checkpoint_test","save-checkpoint",view.world_epoch).unwrap().applied);assert!(scene_route_commands::trigger(&runtime,"upper_trigger","save-trigger",view.world_epoch).unwrap().applied);
    runtime.save().unwrap();runtime.save_slot("upper","Upper deck",true).unwrap();let saved=save_v6::read_save(&root).unwrap();assert!(saved.save.player.grounded);near(saved.save.player.position_m.y_m,2.);let bytes=std::fs::read(root.join("formal-save-v6.json")).unwrap();
    for slot in [false,true] {let prepared=if slot {runtime.continue_slot("upper").unwrap()}else{runtime.continue_saved().unwrap()};assert_eq!(prepared.server_time_ms,saved.save.server_time_ms);assert_eq!(prepared.player.transform.position_m,saved.save.player.position_m);stable(&runtime);let ready=acknowledge_ready(&runtime,prepared);assert!(!ready.interactables.iter().find(|item|item.entity_id=="upper_trigger").unwrap().active);assert!(!ready.interactables.iter().find(|item|item.entity_id=="gh_checkpoint_test").unwrap().active);let next=idle(&runtime,&ready);near(next.player.transform.position_m.y_m,2.);}
    assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),bytes);
}

#[test]
#[cfg(feature="deterministic-replay")]
fn changed_or_removed_static_catalog_rejects_continues_atomically() {
    let (runtime,root)=fixture(false,scene(2.,true));let initial=runtime.resume().unwrap();ride(&runtime,initial);runtime.save().unwrap();runtime.save_slot("deck","Deck",true).unwrap();let bytes=std::fs::read(root.join("formal-save-v6.json")).unwrap();let slot_bytes=std::fs::read(root.join("slots/deck/slot-v6.json")).unwrap();
    for definition in [scene(1.8,true),scene(0.,false)] {
        let (other,other_root)=fixture(false,definition);std::fs::create_dir_all(other_root.join("slots/deck")).unwrap();std::fs::write(other_root.join("formal-save-v6.json"),&bytes).unwrap();std::fs::write(other_root.join("slots/deck/slot-v6.json"),&slot_bytes).unwrap();let before=other.snapshot().unwrap();
        assert_eq!(other.continue_saved().unwrap_err(),"E_SAVE_SUPPORT_FRAME_INVALID");assert_eq!(other.snapshot().unwrap(),before);assert_eq!(std::fs::read(other_root.join("formal-save-v6.json")).unwrap(),bytes);
        assert_eq!(other.continue_slot("deck").unwrap_err(),"E_SAVE_SUPPORT_FRAME_INVALID");assert_eq!(other.snapshot().unwrap(),before);assert_eq!(std::fs::read(other_root.join("slots/deck/slot-v6.json")).unwrap(),slot_bytes);
    }
}

#[test]
#[cfg(feature="deterministic-replay")]
fn malformed_height_or_support_scene_cannot_replace_current_runtime() {
    let (runtime,_)=fixture(false,scene(2.,true));let before=runtime.snapshot().unwrap();
    for mutation in ["missing","reversed","typo","unbacked_endpoint","duplicate"] {let mut value=scene(2.,true);match mutation {"missing"=>{value["transitions"][0].as_object_mut().unwrap().remove("heightRangeM");},"reversed"=>value["interactions"][0]["heightRangeM"]=serde_json::json!([2.1,1.9]),"typo"=>{let range=value["checkpoints"][0].as_object_mut().unwrap().remove("heightRangeM").unwrap();value["checkpoints"][0]["heightRange"]=range;},"unbacked_endpoint"=>value["logic"]["traversal"][0]["to"]=serde_json::json!([5.,1.,2.]),"duplicate"=>value["standingDecks"][0]["id"]=serde_json::json!("fixture_lift"),_=>unreachable!()};assert!(load(&runtime,&value).is_err(),"{mutation}");assert_eq!(runtime.snapshot().unwrap(),before);}
}

#[test]
#[cfg(feature="deterministic-replay")]
fn static_only_scene_preserves_airborne_velocity_in_file_and_slot_continue() {
    for descending in [false,true] {
        let mut source=scene(2.,true);source["transitions"].as_array_mut().unwrap().push(serde_json::json!({"id":"to_static_only","toSceneId":"gh_static_upper","spawnId":"upper_spawn","polygon":[[3.5,1.5],[4.5,1.5],[4.5,2.5],[3.5,2.5]],"heightRangeM":[1.9,2.1]}));
        let upper=serde_json::json!({"schemaVersion":1,"worldId":"grey_hive","sceneId":"gh_static_upper","boundsM":{"x":0,"z":0,"width":12,"depth":12},"navigation":{"nodes":[{"id":"upper_nav","position":[4,2,2]}],"links":[]},"spawns":[{"id":"upper_spawn","kind":"player","position":[4,2,2],"navNode":"upper_nav"}],"standingDecks":[{"id":"static_only_deck","polygon":[[3,1],[6,1],[6,4],[3,4]],"heightM":2} ]});
        let (runtime,root)=fixture_extra(false,source,vec![upper]);let initial=runtime.resume().unwrap();let view=upper_position(&runtime,ride(&runtime,initial));
        assert!(scene_route_commands::transition(&runtime,"to_static_only","static-entry",view.world_epoch).unwrap().applied);let ready=acknowledge_ready(&runtime,runtime.snapshot().unwrap());assert_eq!(ready.scene_id,"gh_static_upper");
        let mut airborne=runtime.submit_input(InputSample::new(ready.world_epoch,ready.ack_seq+1,next_time(),0.,0.).unwrap(),vec![CombatIntentRequest::Jump{request_id:1}]).unwrap();
        for _ in 0..40 {if !descending || airborne.player.velocity_mps.y_m<0. {break;}airborne=idle(&runtime,&airborne);}
        assert!(airborne.player.transform.position_m.y_m>2.);assert_eq!(airborne.player.velocity_mps.y_m<0.,descending);
        runtime.save().unwrap();runtime.save_slot("air","Air",true).unwrap();let saved=save_v6::read_save(&root).unwrap();assert!(!saved.save.player.grounded);assert_eq!(saved.save.moving_support_frame.as_ref().unwrap().poses.len(),1);
        let control=idle(&runtime,&airborne);
        for slot in [false,true] {let prepared=if slot {runtime.continue_slot("air").unwrap()}else{runtime.continue_saved().unwrap()};near(prepared.player.velocity_mps.y_m,saved.save.player.velocity_mps.y_m);let ready=acknowledge_ready(&runtime,prepared);let next=idle(&runtime,&ready);assert_eq!(next.server_time_ms,control.server_time_ms);near(next.player.transform.position_m.y_m,control.player.transform.position_m.y_m);near(next.player.velocity_mps.y_m,control.player.velocity_mps.y_m);}
    }
}
fn upper_position(runtime:&FormalRuntime,view:WorldView)->WorldView {upper(runtime,view)}

#[test]
fn exact_compiler_output_loads_with_unflattened_height_authority() {
    let raw=include_str!("fixtures/standing-deck-compiled.json");
    let registry=wuxian_horror_ch1::scene_registry::WorldRegistry::load(
        [RS,GH,MH,CW,raw],&BTreeSet::from(["runtime2d.grey_hive.floor_tiles.v1".to_owned()]),&BTreeSet::new()).unwrap();
    let scene=registry.scene("gh_test").unwrap();
    assert_eq!(scene.standing_decks[0].height_m,2.);
    assert_eq!(scene.logic.traversal[0].from,[4.,2.,4.]);
    assert_eq!(scene.logic.traversal[0].to,[6.5,2.,4.]);
    assert!(!scene.height_allows("upper_exit",0.));assert!(scene.height_allows("upper_exit",2.));
}
