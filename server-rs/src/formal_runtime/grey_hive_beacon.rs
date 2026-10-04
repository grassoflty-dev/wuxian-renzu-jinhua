//! GH-BEACON-1. Physical collect/mount receipts, never inferred from old markers
//! or route completion. All changes are committed with the cloned scene owner.
use super::*;
use crate::world_persistent_v1::{GreyHivePersistentState,GreyHiveBeaconState,GreyHiveBeaconStage as Stage,GreyHiveBeaconAction as Action,GreyHiveBeaconReceipt};
pub(super) const COLLECT_ID:&str="gh_beacon_deploy_marker";
pub(super) const MOUNT_ID:&str="gh_beacon_storage_mount_marker";
const EXTRACT_ID:&str="gh_exit_extraction_console";
const EXTRACT_EVENT:&str="hive_extraction";
pub(super) fn is_beacon_id(id:&str)->bool { matches!(id,COLLECT_ID|MOUNT_ID) }
fn action(id:&str)->Option<Action>{match id{COLLECT_ID=>Some(Action::Collect),MOUNT_ID=>Some(Action::Mount),_=>None}}
fn completed(route:&RouteState)->bool{route.progress.iter().find(|p|p.world_id=="grey_hive").is_some_and(|p|p.completed)}
fn valid(p:&GreyHivePersistentState)->Result<(),String>{if p.beacon.as_ref().is_some_and(|b|!b.validate()){Err("E_BEACON_STATE_INVALID".into())}else{Ok(())}}
pub(super) fn owned(state:&RuntimeState)->bool{state.world_persistent_v1.grey_hive.beacon.as_ref().is_some_and(GreyHiveBeaconState::owned)}
pub(crate) fn validate_saved(save:&crate::save_v5::SaveV5,p:&GreyHivePersistentState)->Result<(),String>{
    valid(p)?;
    if let Some(beacon)=&p.beacon{
        if beacon.receipts.iter().any(|r|r.world_epoch>save.revision.world_epoch)
            ||(beacon.state==Stage::Uncollected&&(completed(&save.progression)||route_completed_events(&save.progression,"grey_hive").contains(EXTRACT_EVENT))){return Err("E_SAVE_BEACON_PROGRESS_INVALID".into());}
    }
    // Whole-object omission is also the supported historical form. It cannot
    // identify whether a user deleted a newer object; never invent a receipt.
    Ok(())
}
fn source(scene:&SceneRuntime)->Result<(),String>{
    let d=scene.current_scene();
    if !scene.is_complete_clockworks_production()||d.world_id!="grey_hive"||d.scene_id!="gh_beacon"{return Err("E_BEACON_SOURCE_INVALID".into());}
    for(id,kind,position)in[(COLLECT_ID,"beacon_collect",[8.,0.,8.]),(MOUNT_ID,"beacon_mount",[17.,0.,8.])]{
        let found:Vec<_>=d.interactions.iter().filter(|i|i.id==id||i.kind==kind).collect();
        if found.len()!=1||found[0].id!=id||found[0].kind!=kind||found[0].position!=position||found[0].event.is_some()
            ||found[0].range_m.unwrap_or(2.5)!=2.5||found[0].environment_control.is_some(){return Err("E_BEACON_SOURCE_INVALID".into());}
    }Ok(())
}
fn in_range(state:&RuntimeState,scene:&SceneRuntime,id:&str)->bool{
    let Some(item)=scene.current_scene().interactions.iter().find(|i|i.id==id)else{return false};
    let p=state.world.player.position_m;
    let distance2=(p.x_m-item.position[0]).powi(2)+(p.y_m-item.position[1]).powi(2)+(p.z_m-item.position[2]).powi(2);
    distance2.is_finite()&&distance2<=item.range_m.unwrap_or(2.5).powi(2)&&scene.current_scene().height_allows(id,p.y_m)
}
pub(super) fn available(state:&RuntimeState,scene:&SceneRuntime,id:&str)->bool{
    if state.paused||state.world.player_hp==0||source(scene).is_err()||valid(&state.world_persistent_v1.grey_hive).is_err()
        ||state.world.world_id!="grey_hive"||state.world.scene_id!="gh_beacon"||state.route.current_world_id!="grey_hive"
        ||scene.world_epoch!=state.world.revision.world_epoch{return false;}
    match action(id){Some(Action::Collect)=>!owned(state),Some(Action::Mount)=>state.world_persistent_v1.grey_hive.beacon.as_ref().is_some_and(|b|b.state==Stage::Carried),_=>false}
}
pub(super) fn project(state:&RuntimeState,scene:&SceneRuntime)->Option<crate::world_v3::GreyHiveBeaconProjection>{
    if !scene.is_complete_clockworks_production()||scene.current_scene().world_id!="grey_hive"||state.world.world_id!="grey_hive"
        ||state.route.current_world_id!="grey_hive"||scene.scene_id!=state.world.scene_id||scene.world_epoch!=state.world.revision.world_epoch||valid(&state.world_persistent_v1.grey_hive).is_err(){return None;}
    let record=state.world_persistent_v1.grey_hive.beacon.as_ref();
    Some(crate::world_v3::GreyHiveBeaconProjection{state:record.map_or(Stage::Uncollected,|r|r.state),legacy_completed_without_receipt:record.is_none()&&completed(&state.route)})
}
fn receipt_response(state:&RuntimeState,scene:&SceneRuntime,request:&str,applied:bool)->FormalInteractionResponse{
    let view=project_scene_view(state,Some(scene));
    FormalInteractionResponse{applied,already_applied:!applied,error_code:None,events:vec![],receipt:CommandReceipt::outcome(request,applied,!applied,None,view.clone()),view}
}
pub(super) fn interact(state:&mut RuntimeState,scene:&mut SceneRuntime,id:&str,request:&str,epoch:u64)->Result<FormalInteractionResponse,String>{
    if state.world.player_hp==0{return Err("E_RUNTIME_DEAD".into());}if state.paused{return Err("E_RUNTIME_PAUSED".into());}
    source(scene)?;valid(&state.world_persistent_v1.grey_hive)?;
    if epoch!=scene.world_epoch||epoch!=state.world.revision.world_epoch{return Err("E_SCENE_RUNTIME_StaleEpoch".into());}
    if state.world.world_id!="grey_hive"||state.world.scene_id!="gh_beacon"||state.route.current_world_id!="grey_hive"{return Err("E_BEACON_SOURCE_INVALID".into());}
    if !crate::scene_registry::valid_id(request){return Err("E_SCENE_RUNTIME_RequestInvalid".into());}
    if !in_range(state,scene,id){return Err("E_SCENE_RUNTIME_OutOfRange".into());}
    let action=action(id).ok_or("E_BEACON_SOURCE_INVALID")?;
    if let Some(prior)=state.world_persistent_v1.grey_hive.beacon.as_ref().and_then(|b|b.receipts.iter().find(|r|r.request_id==request)){
        if prior.action!=action{return Err("E_BEACON_REQUEST_COLLISION".into());}
        return Ok(receipt_response(state,scene,request,false));
    }
    let mut next=state.clone();let mut candidate=scene.clone();
    candidate.claim_command_request(request,epoch).map_err(|e|e.to_string())?;
    let beacon=next.world_persistent_v1.grey_hive.beacon.get_or_insert_with(GreyHiveBeaconState::new_uncollected);
    let applied=match(action,beacon.state){
        (Action::Collect,Stage::Uncollected)=>{beacon.state=Stage::Carried;true},
        (Action::Mount,Stage::Carried)=>{beacon.state=Stage::Mounted;true},
        (Action::Mount,Stage::Uncollected)=>return Err("E_BEACON_NOT_CARRIED".into()),
        _=>false,
    };
    if applied{
        beacon.receipts.push(GreyHiveBeaconReceipt{action,request_id:request.into(),world_epoch:epoch});
        if !beacon.validate(){return Err("E_BEACON_STATE_INVALID".into());}
        // Preserves historical activation/request ledgers, while adding only the
        // real current action. Old marker activation alone never reaches here.
        candidate.restore_durable_interactions(&[id]).map_err(|e|e.to_string())?;
        next.world.bump_authority_revision().map_err(|e|format!("E_WORLD_REVISION: {e:?}"))?;
        emit_presentation(&mut next,if action==Action::Collect{"GreyHiveBeaconCollected"}else{"GreyHiveBeaconMounted"},state.world.player.position_m,1.0);
    }
    *state=next;*scene=candidate;
    Ok(receipt_response(state,scene,request,applied))
}
pub(super) fn reconfirm_extraction(state:&RuntimeState,scene:&mut SceneRuntime,id:&str,request:&str,epoch:u64)->Result<Option<Vec<SceneEvent>>,String>{
    if !scene.is_complete_clockworks_production()||id!=EXTRACT_ID||!scene.object_activated(id){return Ok(None);}
    if epoch!=scene.world_epoch||epoch!=state.world.revision.world_epoch{return Err("E_SCENE_RUNTIME_StaleEpoch".into());}
    let events=vec![SceneEvent::Interaction{id:EXTRACT_ID.into(),event_id:Some(EXTRACT_EVENT.into())}];
    validate_extraction(state,scene,&events)?;
    scene.claim_command_request(request,epoch).map_err(|e|e.to_string())?;Ok(Some(events))
}
pub(super) fn extraction_reconfirmation_available(state:&RuntimeState,scene:&SceneRuntime,id:&str)->bool{
    id==EXTRACT_ID&&!state.paused&&state.world.player_hp>0&&scene.object_activated(id)
        &&validate_extraction(state,scene,&[SceneEvent::Interaction{id:EXTRACT_ID.into(),event_id:Some(EXTRACT_EVENT.into())}]).is_ok()
}
/// Strict native extraction requires the one authored console receipt and real
/// current ownership. No other trigger, event string, or generic route IPC can
/// create it. Generic registry fixtures retain their historical semantics.
pub(super) fn validate_extraction(state:&RuntimeState,scene:&SceneRuntime,events:&[SceneEvent])->Result<(),String>{
    if !scene.is_complete_clockworks_production(){return Ok(());}
    let related=events.iter().any(|e|match e{SceneEvent::Interaction{event_id,..}=>event_id.as_deref()==Some(EXTRACT_EVENT),SceneEvent::Trigger{event_id,..}=>event_id==EXTRACT_EVENT,_=>false});
    if !related{return Ok(());}
    if state.world.player_hp==0{return Err("E_RUNTIME_DEAD".into());}if state.paused{return Err("E_RUNTIME_PAUSED".into());}
    let d=scene.current_scene();let found:Vec<_>=d.interactions.iter().filter(|i|i.id==EXTRACT_ID||i.event.as_deref()==Some(EXTRACT_EVENT)).collect();
    if d.world_id!="grey_hive"||d.scene_id!="gh_exit"||state.world.world_id!="grey_hive"||state.world.scene_id!="gh_exit"
        ||state.route.current_world_id!="grey_hive"||scene.world_epoch!=state.world.revision.world_epoch
        ||found.len()!=1||found[0].id!=EXTRACT_ID||found[0].kind!="extraction_console"||found[0].event.as_deref()!=Some(EXTRACT_EVENT)
        ||d.triggers.iter().any(|t|t.event==EXTRACT_EVENT)||d.interaction_aggregates.iter().any(|a|a.event==EXTRACT_EVENT)
        ||!matches!(events,[SceneEvent::Interaction{id,event_id:Some(event)}] if id==EXTRACT_ID&&event==EXTRACT_EVENT)
        ||!scene.object_activated(EXTRACT_ID)||!scene.event_complete(EXTRACT_EVENT)||!in_range(state,scene,EXTRACT_ID){return Err("E_GREY_HIVE_EXTRACTION_SOURCE_INVALID".into());}
    if completed(&state.route){return Err("E_GREY_HIVE_EXTRACTION_ALREADY_COMPLETE".into());}
    valid(&state.world_persistent_v1.grey_hive)?;
    if !owned(state){return Err("E_BEACON_REQUIRED".into());}Ok(())
}
#[cfg(test)]mod tests;
