//! Atomic public support/rider projection over real authoritative fixture movement.
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


fn atomic(view:&WorldView)->&wuxian_horror_ch1::moving_support::SupportSceneView {
    let projection=view.support_scene.as_ref().expect("support source must project");
    assert_eq!(projection.schema_version,1);assert_eq!(projection.world_id,view.world_id);assert_eq!(projection.scene_id,view.scene_id);assert_eq!(projection.world_epoch,view.world_epoch);assert_eq!(projection.server_tick,view.server_tick);assert_eq!(projection.authority_revision,view.authority_revision);assert_eq!(projection.server_time_ms,view.server_time_ms);assert_eq!(projection.rider.position_m,view.player.transform.position_m);assert_eq!(projection.scene_source_sha256.len(),64);near(projection.rider.radius_m,0.35);
    if let Some(id)=&projection.rider.support_id {near(projection.poses.iter().find(|pose|&pose.support_id==id).unwrap().height_m,view.player.transform.position_m.y_m);}
    let snapshot=wuxian_horror_ch1::world_v3::WorldSnapshot::from_view(view.clone());assert_eq!(snapshot.support_scene.as_ref(),Some(projection));projection
}
#[test]
#[cfg(feature="deterministic-replay")]
fn floor_board_ride_deck_airborne_and_continue_project_one_exact_authority_frame() {
    use wuxian_horror_ch1::moving_support::SupportRiderMode as Mode;
    let (runtime,root)=fixture(false,scene(2.,true));let mut view=runtime.resume().unwrap();assert_eq!(atomic(&view).rider.mode,Mode::Floor);
    for _ in 0..80 {if view.player.transform.position_m.x_m<=2.25{break;}view=input(&runtime,&view,-1.);atomic(&view);}
    let mut saw_surface=false;for _ in 0..650 {let projection=atomic(&view);saw_surface|=projection.rider.mode==Mode::Surface;if (view.player.transform.position_m.y_m-2.).abs()<0.001{break;}view=idle(&runtime,&view);}assert!(saw_surface);near(view.player.transform.position_m.y_m,2.);
    for _ in 0..80 {if view.player.transform.position_m.x_m>=3.6{break;}view=input(&runtime,&view,1.);atomic(&view);}assert_eq!(atomic(&view).rider.support_id.as_deref(),Some("fixture_deck"));
    view=runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,next_time(),0.,0.).unwrap(),vec![CombatIntentRequest::Jump{request_id:1}]).unwrap();assert_eq!(atomic(&view).rider.mode,Mode::Airborne);
    runtime.save().unwrap();let bytes=std::fs::read(root.join("formal-save-v6.json")).unwrap();let prepared=runtime.continue_saved().unwrap();let held=atomic(&prepared).clone();stable(&runtime);let ready=acknowledge_ready(&runtime,prepared);assert_eq!(atomic(&ready).poses,held.poses);assert_eq!(atomic(&ready).rider,held.rider);assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),bytes);
}
#[test]
#[cfg(feature="deterministic-replay")]
fn source_hash_is_registry_owned_and_visual_only_changes_do_not_invalidate_saved_supports() {
    use sha2::{Digest,Sha256};
    let definition=scene(2.,true);let (runtime,root)=fixture(false,definition.clone());let initial=runtime.resume().unwrap();let view=upper(&runtime,ride(&runtime,initial));assert_eq!(atomic(&view).scene_source_sha256,format!("{:x}",Sha256::digest(definition.to_string().as_bytes())));
    runtime.save().unwrap();let saved=save_v6::read_save(&root).unwrap();let bytes=std::fs::read(root.join("formal-save-v6.json")).unwrap();
    let mut forged=definition.clone();forged["verifiedSourceSha256"]=serde_json::json!("0".repeat(64));assert!(load(&runtime,&forged).is_err());
    let mut changed=definition;changed["presentation"]["cameraProfile"]=serde_json::json!("oblique_default");load(&runtime,&changed).unwrap();let prepared=runtime.continue_saved().unwrap();let projection=atomic(&prepared);assert_eq!(projection.scene_source_sha256,format!("{:x}",Sha256::digest(changed.to_string().as_bytes())));assert_eq!(projection.poses,saved.save.moving_support_frame.as_ref().unwrap().poses);assert_eq!(prepared.player.transform.position_m,saved.save.player.position_m);assert_eq!(std::fs::read(root.join("formal-save-v6.json")).unwrap(),bytes);
}
#[test]
#[cfg(feature="deterministic-replay")]
fn transition_to_flat_scene_removes_old_support_projection_before_readiness() {
    let (runtime,_)=fixture(false,scene(2.,true));let initial=runtime.resume().unwrap();let view=upper(&runtime,ride(&runtime,initial));atomic(&view);assert!(scene_route_commands::transition(&runtime,"gh_to_hub","flat-entry",view.world_epoch).unwrap().applied);let prepared=runtime.snapshot().unwrap();assert_eq!(prepared.scene_id,"rs_test_hub");assert!(prepared.support_scene.is_none());assert!(!serde_json::to_value(wuxian_horror_ch1::world_v3::WorldSnapshot::from_view(prepared.clone())).unwrap().as_object().unwrap().contains_key("supportScene"));acknowledge_ready(&runtime,prepared);
}
