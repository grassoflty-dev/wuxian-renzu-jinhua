//! Read-only native Signal Wraith projection. One actor remains one damage target.
use super::*;
use crate::signal_perception::{project_signal_positions,SignalPerceptionConfig,SignalPositionProjection,SignalProjectionMode};
use crate::world_v3::ActorRuntime;
pub(super) fn projection(state:&RuntimeState,actor:&ActorRuntime)->Option<SignalPositionProjection>{
    if actor.entity_type!=signal_wraith_roster::WRAITH||actor.controller_variant!=1||!actor.validate(){return None;}
    let mapping=state.effective_rules_v6.capability_permissions.contains(&CapabilityPermission::AcousticMapping);
    let mode=if mapping||!actor.signal_interferes_at(state.world.player.position_m,&state.kcc){SignalProjectionMode::AcousticMapping}else{SignalProjectionMode::Unmapped};
    let actual=actor.position_m;let radius=actor.body_radius_m();
    project_signal_positions(&actor.entity_id,actual,actor.state,actor.attack_serial,mode,SignalPerceptionConfig::default(),|to|{
        let steps=(((to.x_m-actual.x_m).hypot(to.z_m-actual.z_m)/0.025).ceil() as usize).max(1);
        (0..=steps).all(|i|{let t=i as f32/steps as f32;let p=Vec3{x_m:actual.x_m+(to.x_m-actual.x_m)*t,y_m:actual.y_m,z_m:actual.z_m+(to.z_m-actual.z_m)*t};
            state.kcc.can_occupy(p,radius)&&state.kcc.stable_actor_footprint(p,radius)})
    }).ok().flatten()
}
pub(super) fn anchor(projection:&SignalPositionProjection)->Vec3{
    let count=projection.positions_m.len() as f32;
    projection.positions_m.iter().fold(Vec3::zero(),|sum,p|Vec3{x_m:sum.x_m+p.x_m/count,y_m:sum.y_m+p.y_m/count,z_m:sum.z_m+p.z_m/count})
}
pub(super) fn project(state:&RuntimeState,scene:&SceneRuntime,view:&mut WorldView){
    let definition=scene.current_scene();
    if !scene.is_complete_clockworks_production()||definition.world_id!="mist_harbor"
        ||!signal_wraith_roster::SCENES.contains(&definition.scene_id.as_str())
        ||state.world.scene_id!=definition.scene_id||state.world.world_id!=definition.world_id
        ||state.world.revision.world_epoch!=scene.world_epoch{return;}
    for actor in &state.world.generic_actors{
        if actor.entity_type!=signal_wraith_roster::WRAITH||actor.controller_variant!=1{continue;}
        if definition.spawns.iter().filter(|s|s.id==actor.entity_id&&s.kind=="enemy"&&s.entity_type.as_deref()==Some(signal_wraith_roster::WRAITH)
            &&vec3_from_array(s.position)==actor.home_m).count()!=1{continue;}
        let matches:Vec<_>=view.actors.iter_mut().filter(|a|a.entity_id==actor.entity_id).collect();
        if matches.len()!=1{continue;}let row=matches.into_iter().next().unwrap();
        // A distinct semantic render type avoids altering the Warden's reused asset alias.
        row.entity_type=signal_wraith_roster::WRAITH.into();
        row.signal_perception=projection(state,actor);
        if let Some(projection)=&row.signal_perception{row.transform.position_m=anchor(projection);}
        // Missing projection on a semantic Wraith is an invalid visible frame,
        // not permission for the frontend to invent an actual position.
    }
}
