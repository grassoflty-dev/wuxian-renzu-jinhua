//! Public vertical combat/hazard fixture. All lift/deck/band values are development tuning.
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
    runtime.load_scene_registry(scenes,"gh_test_power",&BTreeSet::from(["fixture.door".to_owned()]),&BTreeSet::from(["grey_hive.infected_maintenance_worker".into(),"enemy.grey_hive.brute".into()])).unwrap();(runtime,root)
}
fn load(runtime:&FormalRuntime,definition:&serde_json::Value)->Result<(),String> {
    runtime.load_scene_registry([RS.to_owned(),definition.to_string(),MH.to_owned(),CW.to_owned()],"gh_test_power",&BTreeSet::from(["fixture.door".to_owned()]),&BTreeSet::from(["grey_hive.infected_maintenance_worker".into(),"enemy.grey_hive.brute".into()])).map(|_|())
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


fn actor_scene()->serde_json::Value {let mut value=scene(2.,true);value["spawns"].as_array_mut().unwrap().push(serde_json::json!({"id":"floor_worker","kind":"enemy","entityType":"grey_hive.infected_maintenance_worker","position":[5.,0.,2.]}));value}
fn pulse(runtime:&FormalRuntime,view:&WorldView,id:u64,kind:ActionKind)->WorldView {runtime.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:view.world_epoch,request_id:id,client_time_ms:next_time(),kind}).unwrap()}
#[test]
fn real_boarding_separates_both_attack_directions_then_floor_counterplay_still_hits() {
    let (runtime,root)=fixture(true,actor_scene());let initial=runtime.resume().unwrap();let mut view=upper(&runtime,ride(&runtime,initial));let upper_hp=view.player.current_hp;assert!(upper_hp>0);
    runtime.save().unwrap();let before=save_v6::read_save(&root).unwrap();let actor_hp=before.save.generic_actors[0].hp;
    let target=view.actors.iter().find(|actor|actor.entity_id=="floor_worker").unwrap().transform.position_m;let position=view.player.transform.position_m;assert!((target.x_m-position.x_m).hypot(target.z_m-position.z_m)<3.);
    view=pulse(&runtime,&view,1,ActionKind::Pulse);for _ in 0..140 {view=idle(&runtime,&view);near(view.player.transform.position_m.y_m,2.);}
    assert_eq!(view.player.current_hp,upper_hp);runtime.save().unwrap();assert_eq!(save_v6::read_save(&root).unwrap().save.generic_actors[0].hp,actor_hp);
    for _ in 0..100 {if view.player.transform.position_m.x_m>6.4{break;}view=input(&runtime,&view,1.);}
    for _ in 0..90 {view=idle(&runtime,&view);if view.player.transform.position_m.y_m==0.{break;}}near(view.player.transform.position_m.y_m,0.);
    for _ in 0..350 {let target=view.actors.iter().find(|a|a.entity_id=="floor_worker").unwrap().transform.position_m;let here=view.player.transform.position_m;if (target.x_m-here.x_m).hypot(target.z_m-here.z_m)<1.5{break;}view=idle(&runtime,&view);}
    let target=view.actors.iter().find(|a|a.entity_id=="floor_worker").unwrap().transform.position_m;let here=view.player.transform.position_m;let dx=target.x_m-here.x_m;let dz=target.z_m-here.z_m;let length=dx.hypot(dz);assert!(length<1.5);
    view=runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,next_time(),0.,0.).unwrap().with_aim(dx/length,dz/length).unwrap(),vec![]).unwrap();view=pulse(&runtime,&view,2,ActionKind::PrimaryAttack);
    for _ in 0..120 {view=idle(&runtime,&view);assert!(view.player.current_hp>0);}
    runtime.save().unwrap();let after=save_v6::read_save(&root).unwrap();assert!(after.save.generic_actors[0].hp<actor_hp);assert!(view.player.current_hp<upper_hp);assert!(runtime.presentation_events_since(view.world_epoch,0).unwrap().iter().any(|event|event.kind=="Damaged"));
}

#[test]
#[cfg(feature="deterministic-replay")]
fn explicit_environment_band_excludes_lower_floor_and_projection_matches_height() {
    let config:serde_json::Value=serde_json::from_str(include_str!("../../content/scenes/compiled/gh_deep_decon.json")).unwrap();let environment=config["hazards"].as_array().unwrap().iter().find(|item|item["id"]=="gh_decon_steam_jet_zone").unwrap()["environment"].clone();
    for elevated in [false,true] {let mut definition=scene(2.,true);definition["hazards"]=serde_json::json!([{"id":"fixture_pressure","kind":"steam_jet","polygon":[[1.,1.],[6.5,1.],[6.5,4.],[1.,4.]],"environment":environment,"heightRangeM":if elevated{vec![1.9,2.1]}else{vec![0.,1.]}}]);let (runtime,_)=fixture(false,definition);let mut start=runtime.resume().unwrap();let floor_hp=start.player.current_hp;for _ in 0..120 {start=idle(&runtime,&start);near(start.player.transform.position_m.y_m,0.);}assert_eq!(start.player.current_hp<floor_hp,!elevated);let mut view=upper(&runtime,ride(&runtime,start));let hp=view.player.current_hp;
        let hazard=view.hazards.iter().find(|item|item.entity_id=="fixture_pressure").unwrap();assert_eq!(hazard.height_range_m.unwrap().0,if elevated{[1.9,2.1]}else{[0.,1.]});near(hazard.transform.position_m.y_m,if elevated{2.}else{0.5});
        for _ in 0..300 {view=idle(&runtime,&view);}assert_eq!(view.player.current_hp<hp,elevated);assert!(view.player.current_hp>0);
        runtime.pause().unwrap();stable(&runtime);
    }
}

#[test]
#[cfg(feature="deterministic-replay")]
fn raised_actor_initial_admission_and_both_continue_routes_fail_closed_atomically() {
    let mut definition=actor_scene();definition["spawns"][1]["position"]=serde_json::json!([4.,2.,2.]);let (runtime,root)=fixture(false,definition.clone());let before=runtime.snapshot().unwrap();runtime.save().unwrap();runtime.save_slot("actor","Actor",true).unwrap();
    let path=root.join("formal-save-v6.json");let slot_path=root.join("slots/actor/slot-v6.json");let bytes=std::fs::read(&path).unwrap();let slot_bytes=std::fs::read(&slot_path).unwrap();let original:serde_json::Value=serde_json::from_slice(&bytes).unwrap();let slot_original:serde_json::Value=serde_json::from_slice(&slot_bytes).unwrap();
    for position in [[9.,2.,2.],[4.,1.,2.],[4.,0.,2.],[4.,99.,2.]] {
        let mut bad=original.clone();bad["save"]["genericActors"][0]["positionM"]=serde_json::json!({"xM":position[0],"yM":position[1],"zM":position[2]});let bad_bytes=serde_json::to_vec(&bad).unwrap();std::fs::write(&path,&bad_bytes).unwrap();assert_eq!(runtime.continue_saved().unwrap_err(),"E_SAVE_ACTOR_SUPPORT_INVALID","{position:?}");assert_eq!(runtime.snapshot().unwrap(),before);assert_eq!(std::fs::read(&path).unwrap(),bad_bytes);
        let mut bad_slot=slot_original.clone();bad_slot["save"]["save"]=bad["save"].clone();let bad_slot_bytes=serde_json::to_vec(&bad_slot).unwrap();std::fs::write(&slot_path,&bad_slot_bytes).unwrap();assert_eq!(runtime.continue_slot("actor").unwrap_err(),"E_SAVE_ACTOR_SUPPORT_INVALID","{position:?}");assert_eq!(runtime.snapshot().unwrap(),before);assert_eq!(std::fs::read(&slot_path).unwrap(),bad_slot_bytes);
    }
    std::fs::write(&path,&bytes).unwrap();std::fs::write(&slot_path,&slot_bytes).unwrap();let ready=acknowledge_ready(&runtime,runtime.continue_saved().unwrap());assert_eq!(ready.actors.iter().find(|actor|actor.entity_id=="floor_worker").unwrap().transform.position_m,p_vec(4.,2.,2.));runtime.pause().unwrap();
    let mut invalid=definition;invalid["standingDecks"].as_array_mut().unwrap().push(serde_json::json!({"id":"too_narrow_for_brute","polygon":[[10.1,10.1],[10.9,10.1],[10.9,10.9],[10.1,10.9]],"heightM":2.}));invalid["spawns"].as_array_mut().unwrap().push(serde_json::json!({"id":"unsupported_brute","kind":"enemy","entityType":"enemy.grey_hive.brute","position":[10.5,2.,10.5]}));let before=runtime.snapshot().unwrap();assert!(load(&runtime,&invalid).unwrap_err().contains("E_ACTOR_SUPPORT_INVALID"));assert_eq!(runtime.snapshot().unwrap(),before);
}
fn p_vec(x:f32,y:f32,z:f32)->wuxian_horror_ch1::world_v3::Vec3 {wuxian_horror_ch1::world_v3::Vec3::new(x,y,z).unwrap()}

#[test]
#[cfg(feature="deterministic-replay")]
fn malformed_or_missing_hazard_height_band_cannot_replace_registered_scene() {
    let (runtime,_)=fixture(false,scene(2.,true));let before=runtime.snapshot().unwrap();
    for band in [serde_json::Value::Null,serde_json::json!([2.1,1.9]),serde_json::json!([-0.1,1.]),serde_json::json!([2.,3.1]),serde_json::json!([2.])] {
        let mut definition=scene(2.,true);definition["hazards"]=serde_json::json!([{"id":"vertical_marker","kind":"steam_jet","polygon":[[1.,1.],[2.,1.],[2.,2.],[1.,2.]],"heightRangeM":band}]);
        assert!(load(&runtime,&definition).is_err());assert_eq!(runtime.snapshot().unwrap(),before);
    }
}
