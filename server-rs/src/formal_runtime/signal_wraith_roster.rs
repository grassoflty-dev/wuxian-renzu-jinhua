//! Exact native Wraith controller admission; no additional actor or reward.
use super::*;
use crate::world_persistent_v1::MistHarborPersistentState;
use crate::world_v3::{ActorAiState,ActorRuntime};
pub(crate) const WRAITH:&str="enemy.mist_harbor.signal_wraith";
pub(crate) const SCENES:[&str;4]=["mh_tidal_warehouse","mh_signal_yard","mh_breakwater","mh_resonance_tower"];
fn identity(a:&ActorRuntime,b:&ActorRuntime)->bool{a.entity_id==b.entity_id&&a.entity_type==b.entity_type&&a.home_m==b.home_m}
fn canonical(scene:&str)->Result<crate::scene_runtime::SceneDefinition,String>{
    let raw=match scene{
        "mh_tidal_warehouse"=>include_str!("../../../content/scenes/compiled/mh_tidal_warehouse.json"),
        "mh_signal_yard"=>include_str!("../../../content/scenes/compiled/mh_signal_yard.json"),
        "mh_breakwater"=>include_str!("../../../content/scenes/compiled/mh_breakwater.json"),
        "mh_resonance_tower"=>include_str!("../../../content/scenes/compiled/mh_resonance_tower.json"),
        _=>return Err("E_SIGNAL_ROSTER_SCENE".into()),
    };serde_json::from_str(raw).map_err(|_|"E_SIGNAL_ROSTER_CONTENT".into())
}
fn authored(scene:&crate::scene_runtime::SceneDefinition)->Result<Vec<ActorRuntime>,String>{
    if scene.world_id!="mist_harbor"||!SCENES.contains(&scene.scene_id.as_str()){return Err("E_SIGNAL_ROSTER_SCENE".into());}
    let expected=spawn_generic_actors(&canonical(&scene.scene_id)?)?;
    let actual=spawn_generic_actors(scene)?;
    if !exact(&actual,&expected){return Err("E_SIGNAL_ROSTER_CONTENT".into());}Ok(expected)
}
fn exact(saved:&[ActorRuntime],expected:&[ActorRuntime])->bool{saved.len()==expected.len()&&saved.iter().zip(expected).all(|(a,b)|identity(a,b)&&a.validate())}
fn version(p:&MistHarborPersistentState)->Result<(),String>{if p.signal_wraith_controller_version>1{Err("E_SAVE_SIGNAL_CONTROLLER_VERSION".into())}else{Ok(())}}
fn all_variant(rows:&[ActorRuntime],v:u8)->bool{rows.iter().filter(|a|a.entity_type==WRAITH).all(|a|a.controller_variant==v)}
fn permitted_versions(rows:&[ActorRuntime],p:&MistHarborPersistentState)->bool{all_variant(rows,1)||(p.signal_wraith_controller_version==0&&all_variant(rows,0))}
/// Conversion is clone-only inside the caller's entry transaction. A historical
/// melee phase is cancelled without damage. Its replacement starts Idle (or
/// remains Dead), so any next cast receives the full new warning. Current phase
/// state is never cancelled, re-aimed or reset.
fn upgrade(rows:&[ActorRuntime])->Result<Vec<ActorRuntime>,String>{rows.iter().map(|a|{
    if a.entity_type!=WRAITH||a.controller_variant==1{return Ok(a.clone());}
    if a.controller_variant!=0||!a.validate(){return Err("E_SAVE_SIGNAL_CONTROLLER_STATE".into());}
    let mut next=ActorRuntime::spawn_with_controller_variant(&a.entity_id,WRAITH,a.home_m,1)?;
    next.position_m=a.position_m;next.hp=a.hp;next.attack_serial=a.attack_serial;
    if a.hp==0{next.state=ActorAiState::Dead;}
    if !next.validate(){return Err("E_SAVE_SIGNAL_CONTROLLER_STATE".into());}Ok(next)
}).collect()}
pub(super) fn install(state:&mut RuntimeState,scene:&crate::scene_runtime::SceneDefinition,strict:bool)->Result<(),String>{
    let world=&mut state.world;let p=&mut state.world_persistent_v1.mist_harbor;
    version(p)?;if !strict||scene.world_id!="mist_harbor"||!SCENES.contains(&scene.scene_id.as_str()){return Ok(());}
    let expected=authored(scene)?;if !exact(&world.generic_actors,&expected)||!all_variant(&world.generic_actors,0){return Err("E_SIGNAL_ROSTER_CONTENT".into());}
    world.generic_actors=upgrade(&world.generic_actors)?;p.signal_wraith_controller_version=1;Ok(())
}
pub(super) fn restore(world:&mut WorldStateV3,scene:&crate::scene_runtime::SceneDefinition,p:&mut MistHarborPersistentState,strict:bool)->Result<bool,String>{
    if !strict||scene.world_id!="mist_harbor"||!SCENES.contains(&scene.scene_id.as_str()){return Ok(false);}version(p)?;
    let expected=authored(scene)?;
    let rows=if world.generic_actors.is_empty()&&p.signal_wraith_controller_version==0{expected.clone()}else{world.generic_actors.clone()};
    if !exact(&rows,&expected)||!permitted_versions(&rows,p){return Err("E_SAVE_SIGNAL_CONTROLLER_STATE".into());}
    world.generic_actors=upgrade(&rows)?;p.signal_wraith_controller_version=1;Ok(true)
}
pub(crate) fn validate_saved(save:&crate::save_v5::SaveV5,p:&MistHarborPersistentState)->Result<(),String>{
    version(p)?;if save.world_id!="mist_harbor"||!SCENES.contains(&save.scene_id.as_str()){return Ok(());}
    let expected=authored(&canonical(&save.scene_id)?)?;
    if p.signal_wraith_controller_version==0&&save.generic_actors.is_empty(){return Ok(());}
    // Tidebound's existing validation runs first. Historical pre-Tidebound rows
    // can omit only those additions; restoration adds them before this upgrade.
    let old_tide:Vec<_>=expected.iter().filter(|a|a.entity_type!=tidebound_roster::TIDEBOUND).cloned().collect();
    if (exact(&save.generic_actors,&expected)||(p.tidebound_actor_roster_version==0&&exact(&save.generic_actors,&old_tide)))
        && permitted_versions(&save.generic_actors,p){return Ok(());}
    Err("E_SAVE_SIGNAL_CONTROLLER_STATE".into())
}
#[cfg(test)]mod tests;
