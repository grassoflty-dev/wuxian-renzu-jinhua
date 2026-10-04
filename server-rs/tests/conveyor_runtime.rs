//! Generic fixture integration, distinct from the genuine canonical campaign.
//! Every movement, action, pause, save and Continue below uses the public runtime.
#[path = "support/entry_readiness.rs"]
mod entry_readiness;
use entry_readiness::acknowledge_ready;
use std::{collections::BTreeSet, path::PathBuf, time::{Duration,SystemTime,UNIX_EPOCH}};
use wuxian_horror_ch1::{continuous_input::InputSample, formal_runtime::{FormalRuntime,ActionCommandRequest,ActionKind},
    scene_route_commands, save_v6, world_v3::WorldView};
const RS:&str=include_str!("fixtures/scene-runtime-v1/return_station.json");
const GH:&str=include_str!("fixtures/scene-runtime-v1/grey_hive.json");
const MH:&str=include_str!("fixtures/scene-runtime-v1/mist_harbor.json");
const CW:&str=include_str!("fixtures/scene-runtime-v1/clockworks.json");

fn fixture(enemy:bool) -> (FormalRuntime,PathBuf) {
    let root=std::env::temp_dir().join(format!("conveyor-{}-{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let runtime=FormalRuntime::new_with_save_dir(root.clone()).unwrap();runtime.pause().unwrap();
    let mut gh:serde_json::Value=serde_json::from_str(GH).unwrap();
    gh["terrainRegions"]=serde_json::json!([{"id":"generic_belt","tag":"conveyor",
        "polygon":[[0.5,0.5],[5.5,0.5],[5.5,5.5],[0.5,5.5]],"surfaceVelocityMps":[1.0,0.0]}]);
    gh["logic"]["traversal"]=serde_json::json!([{"id":"belt_vault","from":[1.0,0.0,1.0],"to":[3.0,0.0,1.0],
        "rangeM":1.2,"cooldownMs":350,"requiredCapabilities":[]}]);
    let mut entities=BTreeSet::new();
    if enemy {
        let id="grey_hive.infected_maintenance_worker";
        entities.insert(id.to_owned());
        gh["spawns"].as_array_mut().unwrap().push(serde_json::json!({"id":"test_worker","kind":"enemy","entityType":id,"position":[3.0,0.0,1.0]}));
    }
    runtime.load_scene_registry([RS.to_owned(),gh.to_string(),MH.to_owned(),CW.to_owned()],"gh_test_power",&BTreeSet::new(),&entities).unwrap();
    (runtime,root)
}

fn idle(runtime:&FormalRuntime,view:&WorldView)->WorldView {
    runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,1000+view.ack_seq*17,0.0,0.0).unwrap(),vec![]).unwrap()
}
fn assert_stable(runtime:&FormalRuntime)->WorldView {
    let before=runtime.snapshot().unwrap();std::thread::sleep(Duration::from_millis(50));
    assert_eq!(runtime.snapshot().unwrap(),before);before
}

#[test]
fn pause_and_save_continue_preserve_position_until_exact_entry_acknowledgment() {
    let (runtime,root)=fixture(false);
    assert_stable(&runtime);
    let initial=runtime.resume().unwrap();let mut view=initial.clone();
    for _ in 0..15 {view=idle(&runtime,&view);}
    assert!(view.player.transform.position_m.x_m>initial.player.transform.position_m.x_m);
    let paused=runtime.pause().unwrap();assert_stable(&runtime);
    assert_eq!(paused.player.velocity_mps.x_m,0.0);
    runtime.save().unwrap();let before=save_v6::read_save(&root).unwrap();
    let prepared=runtime.continue_saved().unwrap();
    assert!(prepared.entry_token.is_some());assert_stable(&runtime);
    assert_eq!(prepared.player.transform.position_m,paused.player.transform.position_m);
    assert_eq!(prepared.player.current_hp,paused.player.current_hp);
    let ready=acknowledge_ready(&runtime,prepared);
    let moved=idle(&runtime,&ready);
    assert!(moved.player.transform.position_m.x_m>ready.player.transform.position_m.x_m);
    assert_eq!(save_v6::read_save(&root).unwrap(),before,"resuming movement must not rewrite the saved checkpoint");
}

#[test]
fn context_vault_reaches_exact_authored_endpoint_before_support_motion_resumes() {
    let (runtime,_)=fixture(false);let view=runtime.resume().unwrap();
    let mut view=runtime.submit_action(ActionCommandRequest {protocol_version:2,world_epoch:view.world_epoch,
        request_id:1,client_time_ms:0,kind:ActionKind::ContextTraversal}).unwrap();
    let mut completion=None;
    for _ in 0..50 {
        if let Some(event)=runtime.presentation_events_since(view.world_epoch,0).unwrap().into_iter().find(|e|e.kind=="ContextTraversalCompleted") {
            completion=Some(event);break;
        }
        view=idle(&runtime,&view);
    }
    let complete=completion.expect("public context traversal must complete");
    assert_eq!(complete.position_m.x_m,3.0);assert_eq!(complete.position_m.z_m,1.0);
    let resumed=idle(&runtime,&view);assert!(resumed.player.transform.position_m.x_m>=3.0);
}

#[test]
fn scene_transition_and_new_do_not_retain_prior_surface_velocity() {
    let (runtime,_)=fixture(false);let view=runtime.resume().unwrap();
    let receipt=scene_route_commands::transition(&runtime,"gh_to_hub","conveyor-leave",view.world_epoch).unwrap();
    assert!(receipt.applied);let view=acknowledge_ready(&runtime,runtime.snapshot().unwrap());
    let next=idle(&runtime,&view);assert_eq!(next.player.transform.position_m,view.player.transform.position_m);
    let prepared=runtime.reset_new().unwrap();assert_stable(&runtime);
    let fresh=acknowledge_ready(&runtime,prepared);let next=idle(&runtime,&fresh);
    assert_eq!(next.player.transform.position_m,fresh.player.transform.position_m);
}

#[test]
fn genuine_enemy_damage_to_zero_hp_freezes_belt_carry_and_rejects_save() {
    let (runtime,_)=fixture(true);let mut view=runtime.resume().unwrap();
    let start_hp=view.player.current_hp;
    for _ in 0..1800 {
        if view.player.current_hp==0 {break;}
        match runtime.submit_input(InputSample::new(view.world_epoch,view.ack_seq+1,1000+view.ack_seq*17,0.0,0.0).unwrap(),vec![]) {
            Ok(next)=>view=next,
            Err(error) if error=="E_RUNTIME_DEAD"=>{view=runtime.snapshot().unwrap();break;},
            Err(error)=>panic!("unexpected runtime failure: {error}"),
        }
    }
    assert!(start_hp>0);assert_eq!(view.player.current_hp,0,"ordinary real attacks must cause death");
    assert!(runtime.presentation_events_since(view.world_epoch,0).unwrap().iter().any(|e|e.kind=="Damaged"));
    assert_stable(&runtime);assert_eq!(runtime.save().unwrap_err(),"E_RUNTIME_DEAD");
}

#[test]
fn malformed_surface_scene_is_rejected_without_replacing_the_active_runtime() {
    let (runtime,_)=fixture(false);let before=runtime.snapshot().unwrap();
    for (tag,velocity,polygon) in [
        ("conveyor",serde_json::json!([0.0,0.0]),serde_json::json!([[1,1],[4,1],[4,4],[1,4]])),
        ("conveyor",serde_json::json!([4.1,0.0]),serde_json::json!([[1,1],[4,1],[4,4],[1,4]])),
        ("conveyor",serde_json::Value::Null,serde_json::json!([[1,1],[4,1],[4,4],[1,4]])),
        ("conveyor",serde_json::json!([1.0,0.0]),serde_json::json!([[1,1],[14,1],[14,4],[1,4]])),
        ("terrain.water_shallow",serde_json::json!([1.0,0.0]),serde_json::json!([[1,1],[4,1],[4,4],[1,4]])),
    ] {
        let mut gh:serde_json::Value=serde_json::from_str(GH).unwrap();
        gh["terrainRegions"]=serde_json::json!([{"id":"bad_surface","tag":tag,"polygon":polygon,"surfaceVelocityMps":velocity}]);
        assert!(runtime.load_scene_registry([RS.to_owned(),gh.to_string(),MH.to_owned(),CW.to_owned()],
            "gh_test_power",&BTreeSet::new(),&BTreeSet::new()).is_err());
        assert_eq!(runtime.snapshot().unwrap(),before);
    }
}
