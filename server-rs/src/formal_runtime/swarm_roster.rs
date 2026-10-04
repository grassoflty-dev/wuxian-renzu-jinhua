//! Narrow additive upgrade for the two authored GH Swarm scenes only.
use super::*;
use crate::world_persistent_v1::GreyHivePersistentState;
use crate::world_v3::ActorRuntime;
const VERSION:u32=1;
pub(crate) const SWARM:&str="enemy.grey_hive.swarm";
const SCENES:[&str;2]=["gh_lockdown","gh_deep_decon"];
fn identity(a:&ActorRuntime,b:&ActorRuntime)->bool { a.entity_id==b.entity_id&&a.entity_type==b.entity_type&&a.home_m==b.home_m }
fn old_count(scene:&str)->usize { if scene==SCENES[0]{6}else{2} }
fn canonical(scene:&str)->Result<crate::scene_runtime::SceneDefinition,String> {
    let raw=match scene{
        "gh_lockdown"=>include_str!("../../../content/scenes/compiled/gh_lockdown.json"),
        "gh_deep_decon"=>include_str!("../../../content/scenes/compiled/gh_deep_decon.json"),
        _=>return Err("E_SWARM_ROSTER_SCENE".into()),
    };serde_json::from_str(raw).map_err(|_|"E_SWARM_ROSTER_CONTENT".into())
}
fn authored(scene:&crate::scene_runtime::SceneDefinition)->Result<Vec<ActorRuntime>,String> {
    if scene.world_id!="grey_hive"||!SCENES.contains(&scene.scene_id.as_str()){return Err("E_SWARM_ROSTER_SCENE".into());}
    let actors=spawn_generic_actors(scene)?;
    let rows:Vec<(&str,&str,[f32;3])>=if scene.scene_id==SCENES[0]{vec![
        ("gh_lockdown_worker_01","grey_hive.infected_maintenance_worker",[6.,0.,4.5]),
        ("gh_lockdown_worker_02","grey_hive.infected_maintenance_worker",[12.,0.,4.]),
        ("gh_lockdown_worker_03","grey_hive.infected_maintenance_worker",[17.,0.,12.]),
        ("gh_lockdown_security_01","grey_hive.infected_security",[9.,0.,11.5]),
        ("gh_lockdown_security_02","grey_hive.infected_security",[14.,0.,4.5]),
        ("gh_lockdown_security_03","grey_hive.infected_security",[20.,0.,5.5]),
        ("gh_lockdown_swarm_01",SWARM,[7.,0.,8.]),("gh_lockdown_swarm_02",SWARM,[17.,0.,8.]),
    ]}else{vec![
        ("gh_decon_worker_01","grey_hive.infected_maintenance_worker",[9.,0.,5.]),
        ("gh_decon_worker_02","grey_hive.infected_maintenance_worker",[17.,0.,10.5]),
        ("gh_deep_decon_swarm_01",SWARM,[11.5,0.,8.]),("gh_deep_decon_swarm_02",SWARM,[20.,0.,5.]),
    ]};
    if actors.len()!=rows.len()||actors.iter().zip(rows).any(|(a,(id,ty,home))|a.entity_id!=id||a.entity_type!=ty||a.home_m!=vec3_from_array(home)||!a.validate()){
        return Err("E_SWARM_ROSTER_CONTENT".into());
    }Ok(actors)
}
fn version(p:&GreyHivePersistentState)->Result<(),String>{if p.swarm_actor_roster_version>VERSION{Err("E_SAVE_SWARM_ROSTER_VERSION".into())}else{Ok(())}}
fn exact(saved:&[ActorRuntime],expected:&[ActorRuntime])->bool{saved.len()==expected.len()&&saved.iter().zip(expected).all(|(a,b)|identity(a,b)&&a.validate())}
pub(super) fn mark_installed(p:&mut GreyHivePersistentState,scene:&crate::scene_runtime::SceneDefinition,strict:bool)->Result<(),String>{
    version(p)?;if strict&&scene.world_id=="grey_hive"&&SCENES.contains(&scene.scene_id.as_str()){authored(scene)?;p.swarm_actor_roster_version=VERSION;}Ok(())
}
pub(super) fn restore(world:&mut WorldStateV3,scene:&crate::scene_runtime::SceneDefinition,p:&mut GreyHivePersistentState,strict:bool)->Result<bool,String>{
    if !strict||scene.world_id!="grey_hive"||!SCENES.contains(&scene.scene_id.as_str()){return Ok(false);}version(p)?;
    let expected=authored(scene)?;
    if exact(&world.generic_actors,&expected){p.swarm_actor_roster_version=VERSION;return Ok(true);}
    if p.swarm_actor_roster_version==VERSION||(!world.generic_actors.is_empty()&&!exact(&world.generic_actors,&expected[..old_count(&scene.scene_id)])){
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    let mut upgraded=expected;for(i,a)in world.generic_actors.iter().enumerate(){upgraded[i]=a.clone();}
    world.generic_actors=upgraded;p.swarm_actor_roster_version=VERSION;Ok(true)
}
pub(crate) fn validate_saved(save:&crate::save_v5::SaveV5,p:&GreyHivePersistentState)->Result<(),String>{
    version(p)?;if save.world_id!="grey_hive"||!SCENES.contains(&save.scene_id.as_str()){return Ok(());}
    let expected=authored(&canonical(&save.scene_id)?)?;
    if exact(&save.generic_actors,&expected)||(p.swarm_actor_roster_version==0&&(save.generic_actors.is_empty()||exact(&save.generic_actors,&expected[..old_count(&save.scene_id)]))){return Ok(());}
    Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into())
}
#[cfg(test)]mod tests;
