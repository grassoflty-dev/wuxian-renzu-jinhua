//! Narrow additive upgrade for the three authored MH Tidebound scenes only.
use super::*;
use crate::world_persistent_v1::MistHarborPersistentState;
use crate::world_v3::ActorRuntime;
const VERSION:u32=1;
pub(crate) const TIDEBOUND:&str="enemy.mist_harbor.tidebound";
const SCENES:[&str;3]=["mh_drowned_quay","mh_breakwater","mh_resonance_tower"];
fn identity(a:&ActorRuntime,b:&ActorRuntime)->bool { a.entity_id==b.entity_id&&a.entity_type==b.entity_type&&a.home_m==b.home_m }
fn old_count(scene:&str)->usize { match scene { "mh_breakwater"=>2, _=>4 } }
fn canonical(scene:&str)->Result<crate::scene_runtime::SceneDefinition,String> {
    let raw=match scene{
        "mh_drowned_quay"=>include_str!("../../../content/scenes/compiled/mh_drowned_quay.json"),
        "mh_breakwater"=>include_str!("../../../content/scenes/compiled/mh_breakwater.json"),
        "mh_resonance_tower"=>include_str!("../../../content/scenes/compiled/mh_resonance_tower.json"),
        _=>return Err("E_TIDEBOUND_ROSTER_SCENE".into()),
    };serde_json::from_str(raw).map_err(|_|"E_TIDEBOUND_ROSTER_CONTENT".into())
}
fn authored(scene:&crate::scene_runtime::SceneDefinition)->Result<Vec<ActorRuntime>,String> {
    if scene.world_id!="mist_harbor"||!SCENES.contains(&scene.scene_id.as_str()){return Err("E_TIDEBOUND_ROSTER_SCENE".into());}
    let actors=spawn_generic_actors(scene)?;
    let rows:Vec<(&str,&str,[f32;3])>=match scene.scene_id.as_str(){
        "mh_drowned_quay"=>vec![
            ("mh_drowned_quay_drowned_01","enemy.mist_harbor.drowned",[8.0, 0.0, 10.0]),
            ("mh_drowned_quay_drowned_02","enemy.mist_harbor.drowned",[12.0, 0.0, 7.0]),
            ("mh_drowned_quay_drowned_03","enemy.mist_harbor.drowned",[16.0, 0.0, 11.0]),
            ("mh_drowned_quay_drowned_04","enemy.mist_harbor.drowned",[20.0, 0.0, 6.0]),
            ("mh_drowned_quay_tidebound_01","enemy.mist_harbor.tidebound",[16.0, 0.0, 8.0]),
        ],
        "mh_breakwater"=>vec![
            ("mh_breakwater_signal_wraith_01","enemy.mist_harbor.signal_wraith",[13.0, 0.0, 6.0]),
            ("mh_breakwater_signal_wraith_02","enemy.mist_harbor.signal_wraith",[19.0, 0.0, 11.0]),
            ("mh_breakwater_tidebound_01","enemy.mist_harbor.tidebound",[10.0, 0.0, 5.0]),
            ("mh_breakwater_tidebound_02","enemy.mist_harbor.tidebound",[17.0, 0.0, 11.0]),
        ],
        "mh_resonance_tower"=>vec![
            ("mh_resonance_tower_signal_wraith_01","enemy.mist_harbor.signal_wraith",[9.0, 0.0, 6.0]),
            ("mh_resonance_tower_signal_wraith_02","enemy.mist_harbor.signal_wraith",[12.0, 0.0, 5.0]),
            ("mh_resonance_tower_signal_wraith_03","enemy.mist_harbor.signal_wraith",[15.0, 0.0, 9.0]),
            ("mh_resonance_tower_signal_wraith_04","enemy.mist_harbor.signal_wraith",[19.0, 0.0, 6.0]),
            ("mh_resonance_tower_tidebound_01","enemy.mist_harbor.tidebound",[12.0, 0.0, 10.5]),
        ],
        _=>return Err("E_TIDEBOUND_ROSTER_SCENE".into()),
    };
    if actors.len()!=rows.len()||actors.iter().zip(rows).any(|(a,(id,ty,home))|a.entity_id!=id||a.entity_type!=ty||a.home_m!=vec3_from_array(home)||!a.validate()){
        return Err("E_TIDEBOUND_ROSTER_CONTENT".into());
    }Ok(actors)
}
fn version(p:&MistHarborPersistentState)->Result<(),String>{if p.tidebound_actor_roster_version>VERSION{Err("E_SAVE_TIDEBOUND_ROSTER_VERSION".into())}else{Ok(())}}
fn exact(saved:&[ActorRuntime],expected:&[ActorRuntime])->bool{saved.len()==expected.len()&&saved.iter().zip(expected).all(|(a,b)|identity(a,b)&&a.validate())}
pub(super) fn mark_installed(p:&mut MistHarborPersistentState,scene:&crate::scene_runtime::SceneDefinition,strict:bool)->Result<(),String>{
    version(p)?;if strict&&scene.world_id=="mist_harbor"&&SCENES.contains(&scene.scene_id.as_str()){authored(scene)?;p.tidebound_actor_roster_version=VERSION;}Ok(())
}
pub(super) fn restore(world:&mut WorldStateV3,scene:&crate::scene_runtime::SceneDefinition,p:&mut MistHarborPersistentState,strict:bool)->Result<bool,String>{
    if !strict||scene.world_id!="mist_harbor"||!SCENES.contains(&scene.scene_id.as_str()){return Ok(false);}version(p)?;
    let expected=authored(scene)?;
    if exact(&world.generic_actors,&expected){p.tidebound_actor_roster_version=VERSION;return Ok(true);}
    if p.tidebound_actor_roster_version==VERSION||(!world.generic_actors.is_empty()&&!exact(&world.generic_actors,&expected[..old_count(&scene.scene_id)])){
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    let mut upgraded=expected;for(i,a)in world.generic_actors.iter().enumerate(){upgraded[i]=a.clone();}
    world.generic_actors=upgraded;p.tidebound_actor_roster_version=VERSION;Ok(true)
}
pub(crate) fn validate_saved(save:&crate::save_v5::SaveV5,p:&MistHarborPersistentState)->Result<(),String>{
    version(p)?;if save.world_id!="mist_harbor"||!SCENES.contains(&save.scene_id.as_str()){return Ok(());}
    let expected=authored(&canonical(&save.scene_id)?)?;
    if exact(&save.generic_actors,&expected)||(p.tidebound_actor_roster_version==0&&(save.generic_actors.is_empty()||exact(&save.generic_actors,&expected[..old_count(&save.scene_id)]))){return Ok(());}
    Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into())
}
#[cfg(test)]mod tests;
