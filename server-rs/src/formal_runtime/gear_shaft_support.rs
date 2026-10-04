//! One exact canonical floor-save upgrade. This does not infer boarding or reset
//! the clock, move a player, grant progression, or relax generic support saves.
use super::*;
use crate::{save_v5::SaveV5,world_persistent_v1::ClockworksPersistentState};
use sha2::{Digest,Sha256};
const SCENE:&str="cw_gear_shaft";
const RAW:&str=include_str!("../../../content/scenes/compiled/cw_gear_shaft.json");
const VERSION:u32=1;
fn version(p:&ClockworksPersistentState)->Result<(),String>{
    if p.gear_shaft_support_version>VERSION {Err("E_SAVE_GEAR_SUPPORT_VERSION".into())}else{Ok(())}
}
fn canonical()->Result<crate::scene_runtime::SceneDefinition,String>{
    serde_json::from_str(RAW).map_err(|_|"E_GEAR_SUPPORT_CONTENT".into())
}
fn exact_scene(scene:&crate::scene_runtime::SceneDefinition)->Result<(),String>{
    if scene.world_id!="clockworks" || scene.scene_id!=SCENE
        || scene.verified_source_sha256!=format!("{:x}",Sha256::digest(RAW.as_bytes())) {
        return Err("E_GEAR_SUPPORT_CONTENT".into());
    }Ok(())
}
fn body(save:&SaveV5)->crate::continuous_kcc::KccBody{
    let p=&save.player;
    crate::continuous_kcc::KccBody{position_m:p.position_m,velocity_mps:p.velocity_mps,radius_m:p.radius_m,
        grounded:p.grounded,move_speed_mps:p.move_speed_mps,jump_speed_mps:p.jump_speed_mps,
        gravity_mps2:p.gravity_mps2,dash_speed_mps:p.dash_speed_mps,
        dash_remaining_ms:p.dash_remaining_ms,dash_cooldown_remaining_ms:p.dash_cooldown_remaining_ms}
}
fn old_floor(save:&SaveV5)->bool{
    // The old canonical scene had no traversal markers. Current records cannot
    // claim that profile while carrying a frame, elevated checkpoint or motion.
    save.moving_support_frame.is_none() && save.player.grounded
        && save.player.position_m.y_m==0.0 && save.player.velocity_mps.y_m==0.0
        && save.player.dash_remaining_ms==0
        && save.checkpoint_id.as_deref().is_none_or(|id|id=="cw_gear_shaft_checkpoint")
        && save.scene_states.iter().filter(|s|s.scene_id==SCENE).all(|s|
            s.checkpoint_id.as_deref().is_none_or(|id|id=="cw_gear_shaft_checkpoint")
            && s.activated_ids.iter().all(|id|id=="cw_gear_shaft_checkpoint")
            && s.emitted_event_ids.is_empty())
}
pub(crate) fn validate_saved(save:&SaveV5,p:&ClockworksPersistentState)->Result<(),String>{
    version(p)?;
    if save.world_id!="clockworks" || save.scene_id!=SCENE{return Ok(());}
    let kcc=scene_kcc(&canonical()?,&BTreeSet::new())?;let b=body(save);
    if !kcc.can_occupy(b.position_m,b.radius_m) || !kcc.valid_saved_support_contact(&b,save.server_time_ms){
        return Err("E_SAVE_GEAR_SUPPORT_CONTACT".into());
    }
    if p.gear_shaft_support_version==0 {
        if !old_floor(save) || kcc.support_rider_view(&b,save.server_time_ms,false)
            .map_or(true,|rider|rider.mode!=crate::moving_support::SupportRiderMode::Floor || rider.support_id.is_some()) {
            return Err("E_SAVE_GEAR_SUPPORT_LEGACY".into());
        }
    }else if !kcc.validate_saved_support_frame(save.moving_support_frame.as_ref(),save.revision.world_epoch,save.server_time_ms){
        return Err("E_SAVE_SUPPORT_FRAME_INVALID".into());
    }
    Ok(())
}
pub(super) fn mark_installed(p:&mut ClockworksPersistentState,scene:&crate::scene_runtime::SceneDefinition,strict:bool)->Result<(),String>{
    version(p)?;
    if strict && scene.world_id=="clockworks" && scene.scene_id==SCENE {
        exact_scene(scene)?;p.gear_shaft_support_version=VERSION;
    }Ok(())
}
/// Returns true only for the one missing-frame compatibility case. The caller
/// still performs normal contact/occupancy checks before installing the clone.
pub(super) fn restore_candidate(state:&mut RuntimeState,save:&SaveV5,scene:&crate::scene_runtime::SceneDefinition,strict:bool)->Result<bool,String>{
    version(&state.world_persistent_v1.clockworks)?;
    if !strict || scene.world_id!="clockworks" || scene.scene_id!=SCENE{return Ok(false);}
    exact_scene(scene)?;validate_saved(save,&state.world_persistent_v1.clockworks)?;
    if state.world_persistent_v1.clockworks.gear_shaft_support_version==VERSION{return Ok(false);}
    if state.world.combat_state.active_traversal.is_some()
        || !state.kcc.can_occupy(state.world.player.position_m,state.world.player.radius_m)
        || !state.kcc.valid_saved_support_contact(&state.world.player,state.world.server_time_ms){
        return Err("E_SAVE_GEAR_SUPPORT_LEGACY".into());
    }
    // The authority clock is retained. A current frame is derived on the next
    // capture/projection; no support contact/boarding record is fabricated.
    state.world_persistent_v1.clockworks.gear_shaft_support_version=VERSION;
    Ok(true)
}
#[cfg(test)] mod tests;
