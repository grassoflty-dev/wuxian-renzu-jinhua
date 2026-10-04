//! Engine-only terrain-coupled fixtures; no native Tidebound content admission.
use super::*;
use crate::{continuous_kcc::{Aabb,KccTerrainRegion,StaticKccWorld},effects::TerrainTag};
const TIDE:&str="enemy.mist_harbor.tidebound";
const DT:f32=1.0/60.0;
fn p(x:f32,z:f32)->Vec3{Vec3::new(x,0.0,z).unwrap()}
fn arena()->StaticKccWorld{StaticKccWorld::new(Aabb::new(0.,30.,0.,20.).unwrap(),vec![])}
fn water()->StaticKccWorld{arena().with_terrain_regions(vec![KccTerrainRegion{id:"mh_water_depth_region".into(),tag:TerrainTag::WaterShallow,polygon:vec![[13.,5.],[19.,5.],[19.,11.],[13.,11.]],surface_velocity_mps:None}])}
fn tide()->ActorRuntime{ActorRuntime::spawn("tide-fixture",TIDE,p(16.,8.)).unwrap()}
fn reach(a:&mut ActorRuntime,kcc:&StaticKccWorld,target:Vec3,state:ActorAiState){for _ in 0..180{if a.state==state{return;}a.tick(target,kcc,DT,false);assert!(a.validate());}panic!("did not reach {state:?}");}
#[test]
fn terrain_selects_distinct_committed_swing_and_charge_profiles(){
    for wet in [false,true]{let kcc=if wet{water()}else{arena()};let mut a=tide();reach(&mut a,&kcc,p(19.,8.),ActorAiState::Windup);assert_eq!(a.current_attack,Some(ActorAttackKind::Charge));assert_eq!(a.state_remaining_ms,if wet{1100}else{800});assert_eq!(a.ordinary.as_ref().unwrap().terrain_variant,Some(if wet{OrdinaryTerrainVariant::Terrain}else{OrdinaryTerrainVariant::Normal}));let profile=a.ordinary_profile_for_current_phase().unwrap();assert_eq!(profile.speed_mps,if wet{2.0}else{1.2});assert_eq!(profile.ordinary.as_ref().unwrap().charge.as_ref().unwrap().speed_mps,if wet{5.5}else{4.3});
        let mut close=tide();reach(&mut close,&kcc,p(17.,8.),ActorAiState::Windup);assert_eq!(close.current_attack,Some(ActorAttackKind::Swing));assert_eq!(close.state_remaining_ms,if wet{900}else{700});}
}
#[test]
fn terrain_query_is_finite_read_only_and_independent_of_player_movement_permissions(){
    use crate::player_rules::{EffectivePlayerRules,MovementMode};
    let mut kcc=water();let before=kcc.clone();let tags=[TerrainTag::WaterShallow,TerrainTag::WaterDeep];
    assert!(kcc.intersects_terrain_at(p(16.,8.),0.4,&tags));assert!(kcc.intersects_terrain_at(p(12.7,8.),0.4,&tags));assert!(!kcc.intersects_terrain_at(p(12.5,8.),0.4,&tags));
    assert!(!kcc.intersects_terrain_at(Vec3{x_m:f32::NAN,..p(16.,8.)},0.4,&tags));assert!(!kcc.intersects_terrain_at(p(16.,8.),f32::INFINITY,&tags));assert!(!kcc.intersects_terrain_at(p(16.,8.),0.0,&tags));assert_eq!(kcc,before);
    for mode in [MovementMode::Ground,MovementMode::Hover,MovementMode::AirStep,MovementMode::Flight,MovementMode::Phase]{kcc.set_movement_rules(EffectivePlayerRules::default(),mode);assert!(kcc.intersects_terrain_at(p(16.,8.),0.4,&tags));}
    assert_eq!(kcc.terrain_tag("mh_water_depth_region"),Some(TerrainTag::WaterShallow));
}
#[test]
fn directional_swing_commits_front_arc_and_respects_range_and_thin_wall(){
    for wet in [false,true]{let kcc=if wet{water()}else{arena()};
        for (target,hit) in [(p(17.3,8.),true),(p(15.,8.),false),(p(16.,9.),false),(p(17.8,8.),wet),(p(18.4,8.),false)]{
            let mut a=tide();reach(&mut a,&kcc,p(17.,8.),ActorAiState::Windup);assert_eq!(a.current_attack,Some(ActorAttackKind::Swing));let mut observed=false;
            while a.state==ActorAiState::Windup{let out=a.tick(target,&kcc,DT,false);if let Some(event)=out.events.iter().find(|e|matches!(e,ActorRuntimeEvent::AttackImpact{..})){observed=true;assert_eq!(out.damage_to_player,hit.then_some(if wet{16}else{12}));assert!(matches!(event,ActorRuntimeEvent::AttackImpact{kind:ActorAttackKind::Swing,hit_player,..}if *hit_player==hit));}}
            assert!(observed);assert_eq!(a.position_m,p(16.,8.));
        }
        let mut a=tide();reach(&mut a,&kcc,p(17.,8.),ActorAiState::Windup);let mut wall=kcc.clone();wall.walls.push(Aabb::new(16.7,16.71,7.,9.).unwrap());
        while a.state==ActorAiState::Windup{assert!(a.tick(p(17.3,8.),&wall,DT,false).damage_to_player.is_none());}
    }
}
#[test]
fn water_charge_crosses_dry_shore_without_retargeting_or_changing_committed_speed(){
    let mut a=ActorRuntime::spawn("tide-fixture",TIDE,p(18.,8.)).unwrap();let kcc=water();reach(&mut a,&kcc,p(22.,8.),ActorAiState::Active);let original=a.clone();let mut hits=vec![];let mut left_water=false;
    while a.state==ActorAiState::Active{let out=a.tick(p(22.,8.),&kcc,DT,false);if let Some(d)=out.damage_to_player{hits.push(d);}if a.state==ActorAiState::Active{assert_eq!(a.ordinary.as_ref().unwrap().terrain_variant,Some(OrdinaryTerrainVariant::Terrain));let active=a.ordinary.as_ref().unwrap().active.as_ref().unwrap();assert!((a.position_m.x_m-(18.+5.5*active.elapsed_ms as f32/1000.)).abs()<0.0001);left_water|=!kcc.intersects_terrain_at(a.position_m,0.4,&[TerrainTag::WaterShallow]);}assert!(a.validate());}
    assert!(left_water);assert_eq!(hits,vec![18]);assert_eq!(a.ordinary.as_ref().unwrap().terrain_variant,None);reach(&mut a,&kcc,p(22.,8.),ActorAiState::Windup);assert_eq!(a.ordinary.as_ref().unwrap().terrain_variant,Some(OrdinaryTerrainVariant::Normal));assert_eq!(a.current_attack,Some(ActorAttackKind::Swing));assert_eq!(a.state_remaining_ms,700);
    let mut dodge=original.clone();while dodge.state==ActorAiState::Active{assert!(dodge.tick(p(22.,12.),&kcc,DT,false).damage_to_player.is_none());}assert_eq!(dodge.position_m.z_m,8.);
    let mut blocked=original;let mut wall=kcc;wall.walls.push(Aabb::new(19.2,19.21,6.,10.).unwrap());while blocked.state==ActorAiState::Active{assert!(blocked.tick(p(22.,8.),&wall,0.25,false).damage_to_player.is_none());}assert!(wall.can_occupy(blocked.position_m,0.4));assert!(blocked.position_m.x_m+0.4<=19.2);
}
#[test]
fn exact_pump_drain_resolves_residual_floor_for_future_attacks_only(){
    use crate::world_persistent_v1::{WorldPersistentState,PumpStartResult,PumpStatus,PUMP_DRAIN_DURATION_MS,DROWNED_QUAY_WATER_ID};
    let mut persistent=WorldPersistentState::default();assert_eq!(PUMP_DRAIN_DURATION_MS,6000);assert_eq!(persistent.mist_harbor.pump.start("mist_harbor",true,100).unwrap(),PumpStartResult::Started);assert!(!persistent.resolve_at(6099).unwrap());assert_eq!(persistent.mist_harbor.pump.state,PumpStatus::Draining);
    let mut kcc=water();let mut committed=tide();reach(&mut committed,&kcc,p(17.8,8.),ActorAiState::Windup);assert_eq!(committed.current_attack,Some(ActorAttackKind::Swing));let phase=committed.clone();
    assert!(persistent.resolve_at(6100).unwrap());assert_eq!(persistent.mist_harbor.pump.state,PumpStatus::Drained);kcc.map_terrain_tags(|id,tag|persistent.resolve_terrain_tag("mist_harbor",id,tag));assert_eq!(kcc.terrain_tag(DROWNED_QUAY_WATER_ID),Some(TerrainTag::WetFloor));assert!(!kcc.intersects_terrain_at(p(16.,8.),0.4,&[TerrainTag::WaterShallow]));assert!(kcc.intersects_terrain_at(p(16.,8.),0.4,&[TerrainTag::WetFloor]));
    let mut unchanged=phase;while committed.state==ActorAiState::Windup{assert_eq!(committed.tick(p(17.8,8.),&kcc,DT,false),unchanged.tick(p(17.8,8.),&water(),DT,false));assert_eq!(committed,unchanged);}
    let mut next=tide();reach(&mut next,&kcc,p(17.,8.),ActorAiState::Windup);assert_eq!(next.ordinary.as_ref().unwrap().terrain_variant,Some(OrdinaryTerrainVariant::Normal));assert_eq!(next.state_remaining_ms,700);assert_eq!(next.ordinary_profile_for_current_phase().unwrap().attack_range_m,1.4);
    assert_eq!(persistent.resolve_terrain_tag("mist_harbor","unrelated-water",TerrainTag::WaterShallow),TerrainTag::WaterShallow);
}
fn owner(w:&mut crate::world_v3::WorldStateV3,kcc:&StaticKccWorld,seq:u64,intents:&[crate::continuous_combat::CombatIntent])->crate::world_v3::CwStepOutput{
    let sample=crate::continuous_input::InputSample::new(8,seq,seq*17,0.,0.).unwrap().with_aim(1.,0.).unwrap();crate::world_v3::step_world(w,kcc,crate::world_v3::CwStepInput{sample:&sample,dt_s:DT,combat:intents}).unwrap()
}
fn owner_world()->crate::world_v3::WorldStateV3{let mut w=crate::world_v3::WorldStateV3::new("mist_harbor",8,p(14.7,8.)).unwrap();w.combat_state.qer_v1_active=true;w.generic_actors=vec![tide()];w}
#[test]
fn real_public_pierce_defeats_tidebound_once_and_guard_blocks_without_counter_damage(){
    use crate::continuous_combat::{CombatEvent,CombatIntent};
    for wet in [false,true]{let kcc=if wet{water()}else{arena()};let mut w=owner_world();let mut deaths=0;
        for seq in 1..=460{let intents=if[1,165,330].contains(&seq){vec![CombatIntent::Pierce{request_id:seq}]}else{vec![]};let out=owner(&mut w,&kcc,seq,&intents);assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::IntentRejected{..})));deaths+=out.actor_runtime.iter().filter(|e|matches!(e,ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Death,..})).count();}
        assert_eq!(deaths,1);assert_eq!(w.generic_actors[0].hp,0);assert!(w.player_hp>0);let hp=w.player_hp;for seq in 461..=510{assert!(owner(&mut w,&kcc,seq,&[]).actor_runtime.is_empty());}assert_eq!(w.player_hp,hp);
        let mut w=owner_world();let mut sent=false;let mut blocks=0;for seq in 1..=110{let a=&w.generic_actors[0];let now=!sent&&a.state==ActorAiState::Windup&&a.state_remaining_ms<=180;let intents=if now{sent=true;vec![CombatIntent::GuardStart{request_id:seq}]}else{vec![]};let out=owner(&mut w,&kcc,seq,&intents);blocks+=out.combat.iter().filter(|e|matches!(e,CombatEvent::GuardImpact{..})).count();}assert_eq!(blocks,1);assert_eq!(w.player_hp,100);assert_eq!(w.generic_actors[0].hp,84);
    }
}
#[test]
fn normal_and_water_phases_roundtrip_without_rechoosing_saved_attack_variant(){
    for wet in [false,true]{for phase in ["swing","charge-windup","charge-active","stagger","dead"]{
        let kcc=if wet{water()}else{arena()};let target=if phase=="swing"{p(17.,8.)}else{p(19.,8.)};let mut a=tide();reach(&mut a,&kcc,target,if phase=="charge-active"{ActorAiState::Active}else{ActorAiState::Windup});if phase=="charge-active"{a.tick(target,&kcc,DT,false);}if phase=="stagger"{a.take_damage_with_stagger(7,25);}if phase=="dead"{a.take_damage(u32::MAX);}assert!(a.validate());
        let mut save:crate::save_v5::SaveV5=serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();save.generic_actors=vec![a.clone()];let root=std::env::temp_dir().join(format!("tide-save-{wet}-{phase}-{}",std::process::id()));crate::save_v5::write_save(&root,&save).unwrap();let bytes=std::fs::read(crate::save_v5::save_path(&root)).unwrap();let loaded=crate::save_v5::read_save(&root).unwrap().restore_state().unwrap();assert_eq!(loaded.world.generic_actors,vec![a.clone()]);let mut b=loaded.world.generic_actors[0].clone();
        // Continue may now have drained terrain; both copies retain the saved phase.
        let changed=arena();for _ in 0..150{assert_eq!(a.tick(target,&changed,DT,false),b.tick(target,&changed,DT,false));assert_eq!(a,b);}
        assert_eq!(std::fs::read(crate::save_v5::save_path(&root)).unwrap(),bytes);
    }}
}
fn assert_actor_unchanged_including_nan_bits(actual:&ActorRuntime,before:&ActorRuntime){
    let bits=|actor:&ActorRuntime|actor.ordinary.as_ref().and_then(|s|s.committed_direction).map(|d|d.map(f32::to_bits));
    assert_eq!(bits(actual),bits(before));
    let mut actual=actual.clone();let mut expected=before.clone();
    if let Some(state)=actual.ordinary.as_mut(){state.committed_direction=None;}
    if let Some(state)=expected.ordinary.as_mut(){state.committed_direction=None;}
    assert_eq!(actual,expected);
}
#[test]
fn malformed_or_foreign_terrain_variant_rejects_save_damage_and_tick_without_mutation(){
    let mut a=tide();reach(&mut a,&water(),p(19.,8.),ActorAiState::Active);let mut save:crate::save_v5::SaveV5=serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();save.generic_actors=vec![a.clone()];let root=std::env::temp_dir().join(format!("tide-invalid-{}",std::process::id()));crate::save_v5::write_save(&root,&save).unwrap();let bytes=std::fs::read(crate::save_v5::save_path(&root)).unwrap();
    for fault in ["missing","mixed-variant","timer","elapsed","direction","foreign","idle-variant"]{let mut bad=a.clone();match fault{
        "missing"=>bad.ordinary.as_mut().unwrap().terrain_variant=None,"mixed-variant"=>bad.ordinary.as_mut().unwrap().terrain_variant=Some(OrdinaryTerrainVariant::Normal),"timer"=>bad.state_remaining_ms=701,"elapsed"=>bad.ordinary.as_mut().unwrap().active.as_mut().unwrap().elapsed_ms=700,"direction"=>bad.ordinary.as_mut().unwrap().committed_direction=Some([f32::NAN,0.]),"foreign"=>bad.entity_type="enemy.grey_hive.brute".into(),"idle-variant"=>{bad=tide();bad.ordinary.as_mut().unwrap().terrain_variant=Some(OrdinaryTerrainVariant::Normal);},_=>unreachable!()}
        assert!(!bad.validate(),"{fault}");let before=bad.clone();assert_eq!(bad.tick(p(19.,8.),&water(),DT,false),ActorTickOutput::default());assert!(bad.take_damage_with_stagger(1,25).is_empty());assert_actor_unchanged_including_nan_bits(&bad,&before);let mut invalid=save.clone();invalid.generic_actors=vec![bad];assert!(crate::save_v5::write_save(&root,&invalid).is_err());assert_eq!(std::fs::read(crate::save_v5::save_path(&root)).unwrap(),bytes);
    }
    let raw=serde_json::to_value(&a).unwrap();for invalid in [serde_json::json!("unknown"),serde_json::json!(false),serde_json::json!(1)]{let mut bad=raw.clone();bad["ordinary"]["terrainVariant"]=invalid;assert!(serde_json::from_value::<ActorRuntime>(bad).is_err());}
}
#[test]
fn paused_fatal_or_stale_owner_preserves_committed_terrain_phase(){
    for wet in [false,true]{let kcc=if wet{water()}else{arena()};let mut a=tide();reach(&mut a,&kcc,p(19.,8.),ActorAiState::Active);let before=a.clone();assert_eq!(a.tick(p(19.,8.),&kcc,0.,false),ActorTickOutput::default());assert_eq!(a,before);let mut w=owner_world();w.generic_actors=vec![a];let wrong=crate::continuous_input::InputSample::new(9,1,17,0.,0.).unwrap();assert!(crate::world_v3::step_world(&mut w,&kcc,crate::world_v3::CwStepInput{sample:&wrong,dt_s:DT,combat:&[]}).is_err());assert_eq!(w.generic_actors[0],before);w.player_hp=0;for seq in 1..=100{assert!(owner(&mut w,&kcc,seq,&[]).actor_runtime.is_empty());}assert_eq!(w.generic_actors[0],before);}
}

#[test]
fn wet_impact_retains_committed_geometry_after_recovery_clears_the_variant(){
    for hit in [false,true]{let mut a=tide();reach(&mut a,&water(),p(17.,8.),ActorAiState::Windup);let target=if hit{p(17.8,8.)}else{p(15.,8.)};let mut impact=None;
        while a.state==ActorAiState::Windup{let out=a.tick(target,&arena(),DT,false);for event in out.events{if matches!(event,ActorRuntimeEvent::AttackImpact{..}){impact=Some(event);}}}
        assert_eq!(a.state,ActorAiState::Cooldown);assert_eq!(a.ordinary.as_ref().unwrap().terrain_variant,None);assert_eq!(a.ordinary_profile_for_current_phase().unwrap().attack_range_m,1.4);
        match impact.unwrap(){ActorRuntimeEvent::AttackImpact{radius_m,geometry:Some(g),hit_player,..}=>{assert_eq!(hit_player,hit);assert_eq!(radius_m,2.2);assert_eq!(g.range_m,2.2);assert_eq!(g.half_angle_rad,Some(0.8));assert_eq!(g.direction_rad,std::f32::consts::FRAC_PI_2);},_=>panic!("missing committed impact geometry")}
    }
}
