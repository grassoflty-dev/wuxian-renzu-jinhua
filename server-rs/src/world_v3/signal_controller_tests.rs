//! Bounded engine fixtures. Controller 1 is opt-in; no native ranged admission.
use super::*;
use crate::{continuous_combat::{CombatEvent,CombatIntent},continuous_input::InputSample,
    continuous_kcc::{Aabb,StaticKccWorld},world_v3::{step_world,CwStepInput,WorldStateV3}};
const WRAITH:&str="enemy.mist_harbor.signal_wraith";
const DT:f32=1.0/60.0;
fn p(x:f32,z:f32)->Vec3{Vec3::new(x,0.,z).unwrap()}
fn arena()->StaticKccWorld{StaticKccWorld::new(Aabb::new(0.,30.,0.,20.).unwrap(),vec![])}
fn wraith()->ActorRuntime{ActorRuntime::spawn_with_controller_variant("wraith-fixture",WRAITH,p(5.,8.),1).unwrap()}
fn reach(a:&mut ActorRuntime,target:Vec3,phase:ActorAiState,kcc:&StaticKccWorld){for _ in 0..400{if a.state==phase{return;}a.tick(target,kcc,DT,false);assert!(a.validate());}panic!("did not reach {phase:?}")}
fn owner(w:&mut WorldStateV3,kcc:&StaticKccWorld,seq:u64,axes:(f32,f32),intents:&[CombatIntent])->crate::world_v3::CwStepOutput{
    let sample=InputSample::new(8,seq,seq*17,axes.0,axes.1).unwrap().with_aim(1.,0.).unwrap();
    step_world(w,kcc,CwStepInput{sample:&sample,dt_s:DT,combat:intents}).unwrap()
}
#[test]
fn native_default_keeps_legacy_melee_profile_and_serialized_bytes(){
    let mut legacy=ActorRuntime::spawn("legacy",WRAITH,p(5.,8.)).unwrap();
    assert_eq!(legacy.controller_variant,0);assert!(legacy.ordinary.is_none());
    assert!(actor_profile(WRAITH).unwrap().ordinary.is_none());
    assert!(!serde_json::to_string(&legacy).unwrap().contains("controllerVariant"));
    let bytes=serde_json::to_vec(&legacy).unwrap();let old:ActorRuntime=serde_json::from_slice(&bytes).unwrap();assert_eq!(old,legacy);assert_eq!(serde_json::to_vec(&old).unwrap(),bytes);
    reach(&mut legacy,p(6.,8.),ActorAiState::Windup,&arena());assert_eq!(legacy.current_attack,Some(ActorAttackKind::HeavyMelee));assert_eq!(legacy.state_remaining_ms,400);
    assert!(!legacy.signal_interferes_at(p(6.,8.),&arena()));
    let current=wraith();assert_eq!(current.max_hp(),Some(45));assert_eq!(current.body_radius_m(),legacy.body_radius_m());
    assert_eq!(current.view().entity_type,legacy.view().entity_type);
    assert!(ActorRuntime::spawn_with_controller_variant("bad",WRAITH,p(5.,8.),2).is_err());
}
#[test]
fn signal_cast_has_committed_medium_range_warning_and_real_single_hit_flight(){
    let mut a=wraith();let kcc=arena();let target=p(10.,8.);
    reach(&mut a,target,ActorAiState::Windup,&kcc);assert_eq!(a.state_remaining_ms,1000);assert_eq!(a.current_attack,Some(ActorAttackKind::SignalShot));
    assert_eq!(a.ordinary.as_ref().unwrap().committed_direction,Some([1.,0.]));
    let origin=a.position_m;reach(&mut a,target,ActorAiState::Active,&kcc);assert_eq!(a.ordinary.as_ref().unwrap().active.as_ref().unwrap().elapsed_ms,0);
    assert_eq!(a.tick(target,&kcc,DT,false).damage_to_player,None);
    let mut hits=0;while a.state==ActorAiState::Active{let out=a.tick(target,&kcc,DT,false);if let Some(damage)=out.damage_to_player{hits+=1;assert_eq!(damage,10);assert_eq!(out.damage_tag,None);assert!(out.ranged_damage);assert!(out.events.iter().any(|e|matches!(e,ActorRuntimeEvent::AttackImpact{kind:ActorAttackKind::SignalShot,geometry:Some(ActorAttackGeometry{range_m,..}),hit_player:true,..}if *range_m==8.)));}}
    assert_eq!(hits,1);assert_eq!(a.position_m,origin);assert_eq!(a.attack_serial,1);assert!(a.validate());
}
#[test]
fn committed_shot_allows_ordinary_dodge_and_keeps_original_height(){
    let mut a=wraith();reach(&mut a,p(10.,8.),ActorAiState::Active,&arena());let original=a.clone();
    while a.state==ActorAiState::Active{let out=a.tick(p(10.,10.),&arena(),DT,false);assert_eq!(out.damage_to_player,None);if let Some(active)=a.ordinary.as_ref().unwrap().active.as_ref(){assert_eq!(active.direction,[1.,0.]);assert_eq!(active.origin_m.y_m,0.);}}
    let mut a=original;for _ in 0..55{assert_eq!(a.tick(Vec3{y_m:2.,..p(10.,8.)},&arena(),DT,false).damage_to_player,None);assert_eq!(a.state,ActorAiState::Active);assert!(!a.ordinary.as_ref().unwrap().active.as_ref().unwrap().hit_resolved);}
    let out=a.tick(p(10.,8.),&arena(),DT,false);assert_eq!(out.damage_to_player,Some(10));
}
#[test]
fn shot_walls_expiry_and_sensing_are_authoritative(){
    let mut a=wraith();reach(&mut a,p(10.,8.),ActorAiState::Active,&arena());
    let mut blocked=arena();blocked.walls.push(Aabb::new(7.,7.01,7.,9.).unwrap());let mut missed=0;
    while a.state==ActorAiState::Active{let out=a.tick(p(10.,8.),&blocked,0.25,false);assert_eq!(out.damage_to_player,None);missed+=out.events.iter().filter(|e|matches!(e,ActorRuntimeEvent::AttackImpact{hit_player:false,..})).count();}assert_eq!(missed,1);
    let mut a=wraith();let before=a.clone();for _ in 0..40{a.tick(p(10.,8.),&blocked,DT,false);}assert_eq!(a,before);
    let mut a=wraith();reach(&mut a,p(10.,8.),ActorAiState::Active,&arena());let mut missed=0;while a.state==ActorAiState::Active{let out=a.tick(p(20.,8.),&arena(),0.25,false);assert!(out.damage_to_player.is_none());missed+=out.events.iter().filter(|e|matches!(e,ActorRuntimeEvent::AttackImpact{hit_player:false,..})).count();}assert_eq!(missed,1);
}
#[test]
fn source_interference_is_bounded_read_only_and_cannot_hide_committed_threats(){
    let mut a=wraith();let before=a.clone();let kcc=arena();assert!(a.signal_interferes_at(p(10.,8.),&kcc));assert!(!a.signal_interferes_at(p(11.1,8.),&kcc));assert!(!a.signal_interferes_at(Vec3{y_m:2.,..p(10.,8.)},&kcc));assert!(!a.signal_interferes_at(Vec3{x_m:f32::NAN,..p(10.,8.)},&kcc));assert_eq!(a,before);
    let mut blocked=arena();blocked.walls.push(Aabb::new(7.,7.01,7.,9.).unwrap());assert!(!a.signal_interferes_at(p(10.,8.),&blocked));
    for phase in [ActorAiState::Windup,ActorAiState::Active]{reach(&mut a,p(10.,8.),phase,&kcc);assert!(!a.signal_interferes_at(p(10.,8.),&kcc));}
    a.take_damage_with_stagger(5,20);assert_eq!(a.state,ActorAiState::Stagger);assert!(!a.signal_interferes_at(p(10.,8.),&kcc));a.take_damage(40);assert_eq!(a.state,ActorAiState::Dead);assert!(!a.signal_interferes_at(p(10.,8.),&kcc));
}
#[test]
fn recovery_blink_uses_supported_clear_space_once_without_invulnerability(){
    let mut a=wraith();reach(&mut a,p(10.,8.),ActorAiState::Cooldown,&arena());let before=a.clone();let mut events=vec![];
    while a.state==ActorAiState::Cooldown{events.extend(a.tick(p(6.,8.),&arena(),DT,false).events);}assert_eq!(a.position_m,p(3.75,8.));assert_eq!(a.hp,before.hp);assert_eq!(a.attack_serial,before.attack_serial);assert_eq!(events.iter().filter(|e|matches!(e,ActorRuntimeEvent::Relocated{..})).count(),1);assert!(a.validate());
    a.take_damage_with_stagger(7,20);assert_eq!(a.hp,38);assert_eq!(a.state,ActorAiState::Stagger);
    let mut blocked=arena();blocked.walls=vec![Aabb::new(4.,4.01,6.,10.).unwrap(),Aabb::new(4.,6.,6.9,7.).unwrap(),Aabb::new(4.,6.,9.,9.1).unwrap()];
    let mut a=before;while a.state==ActorAiState::Cooldown{assert!(!a.tick(p(6.,8.),&blocked,DT,false).events.iter().any(|e|matches!(e,ActorRuntimeEvent::Relocated{..})));}assert_eq!(a.position_m,p(5.,8.));
}
#[test]
fn saved_variant_phases_roundtrip_and_replay_without_timer_or_file_rewrites(){
    for phase in ["idle","windup","active-zero","active","cooldown","stagger","dead"]{
        let mut a=wraith();if phase!="idle"{reach(&mut a,p(10.,8.),if phase=="windup"{ActorAiState::Windup}else{ActorAiState::Active},&arena());}
        if phase=="windup"{a.state_remaining_ms=1;}if phase=="active"{a.tick(p(10.,8.),&arena(),DT,false);}if phase=="cooldown"{reach(&mut a,p(10.,8.),ActorAiState::Cooldown,&arena());}if phase=="stagger"{a.take_damage_with_stagger(7,20);}if phase=="dead"{a.take_damage(45);}assert!(a.validate());
        let mut save:crate::save_v5::SaveV5=serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();save.generic_actors=vec![a.clone()];let root=std::env::temp_dir().join(format!("signal-save-{phase}-{}",std::process::id()));crate::save_v5::write_save(&root,&save).unwrap();let bytes=std::fs::read(crate::save_v5::save_path(&root)).unwrap();let loaded=crate::save_v5::read_save(&root).unwrap().restore_state().unwrap();let mut b=loaded.world.generic_actors[0].clone();assert_eq!(a,b);
        let held=a.clone();assert_eq!(a.tick(p(10.,8.),&arena(),0.,false),ActorTickOutput::default());assert_eq!(a,held);
        for _ in 0..140{assert_eq!(a.tick(p(6.,8.),&arena(),DT,false),b.tick(p(6.,8.),&arena(),DT,false));assert_eq!(a,b);}assert_eq!(std::fs::read(crate::save_v5::save_path(&root)).unwrap(),bytes);
    }
}
#[test]
fn malformed_variant_or_phase_rejects_before_tick_damage_and_save(){
    let mut active=wraith();reach(&mut active,p(10.,8.),ActorAiState::Active,&arena());
    for index in 0..8{let mut bad=active.clone();match index{0=>bad.controller_variant=2,1=>bad.controller_variant=0,2=>bad.ordinary=None,3=>bad.current_attack=Some(ActorAttackKind::PressureShot),4=>bad.state_remaining_ms+=1,5=>bad.ordinary.as_mut().unwrap().active.as_mut().unwrap().direction=[0.,0.],6=>bad.ordinary.as_mut().unwrap().active.as_mut().unwrap().hit_resolved=true,_=>bad.entity_type="enemy.mist_harbor.drowned".into()};assert!(!bad.validate());let before=bad.clone();assert_eq!(bad.tick(p(10.,8.),&arena(),DT,false),ActorTickOutput::default());assert!(bad.take_damage_with_stagger(20,20).is_empty());bad.take_damage(20);assert_eq!(bad,before);
        let mut save:crate::save_v5::SaveV5=serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();save.generic_actors=vec![bad];let root=std::env::temp_dir().join(format!("signal-invalid-{index}-{}",std::process::id()));assert!(crate::save_v5::write_save(&root,&save).is_err());assert!(!crate::save_v5::save_path(&root).exists());}
}
#[test]
fn real_guard_dash_iframes_and_pressure_immunity_keep_signal_damage_semantics(){
    use crate::effects::{EffectLifetime,EffectResolver,EffectSource,EffectSourceKind,EffectSpec,HazardImmunityEffect,HazardTag,StackRule};
    let rules=EffectResolver::resolve(&[EffectSource{source_kind:EffectSourceKind::TemporaryBuff,source_id:"test_pressure".into(),instance_id:"test".into(),lifetime:EffectLifetime::Equipped,ui_category:None,effects:vec![EffectSpec::HazardImmunity(HazardImmunityEffect{tag:HazardTag::Pressure,stack:StackRule::Any})]}]).unwrap();let kcc=arena().with_movement_rules(rules,crate::player_rules::MovementMode::Ground);
    for mode in ["hit","guard","dash","iframes"]{let mut w=WorldStateV3::new("mist_harbor",8,p(10.,8.)).unwrap();w.combat_state.qer_v1_active=true;w.generic_actors=vec![wraith()];if mode=="iframes"{w.generic_actors.push(ActorRuntime::spawn_with_controller_variant("second",WRAITH,p(15.,8.),1).unwrap());}let mut sent=false;let mut absorbed=0;let mut blocks=0;
        for seq in 1..=150{let a=&w.generic_actors[0];let act=!sent&&a.state==ActorAiState::Active&&a.state_remaining_ms<=850;let intents=if act&&mode=="guard"{vec![CombatIntent::GuardStart{request_id:seq}]}else if act&&mode=="dash"{vec![CombatIntent::ActionDash{request_id:seq}]}else{vec![]};if act{sent=true;}let out=owner(&mut w,&kcc,seq,if act&&mode=="dash"{(0.,1.)}else{(0.,0.)},&intents);assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::IntentRejected{..})));assert!(!out.actor_runtime.iter().any(|e|matches!(e,ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Stagger,..})));absorbed+=out.combat.iter().filter(|e|matches!(e,CombatEvent::DamageAbsorbed{..})).count();blocks+=out.combat.iter().filter(|e|matches!(e,CombatEvent::GuardImpact{..})).count();}
        assert_eq!(w.player_hp,if matches!(mode,"hit"|"iframes"){90}else{100},"{mode}");if mode=="guard"{assert_eq!(blocks,1);}if mode=="iframes"{assert_eq!(absorbed,1);}}
}
#[test]
fn genuine_owner_pierce_defeat_and_fatal_pause_have_no_late_shot(){
    let mut w=WorldStateV3::new("mist_harbor",8,p(3.8,8.)).unwrap();w.combat_state.qer_v1_active=true;w.generic_actors=vec![wraith()];let mut deaths=0;
    for seq in 1..=230{let intents=if [1,165].contains(&seq){vec![CombatIntent::Pierce{request_id:seq}]}else{vec![]};let out=owner(&mut w,&arena(),seq,(0.,0.),&intents);assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::IntentRejected{..})));deaths+=out.actor_runtime.iter().filter(|e|matches!(e,ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Death,..})).count();}assert_eq!(deaths,1);assert_eq!(w.generic_actors[0].hp,0);assert!(w.player_hp>0);
    for phase in [ActorAiState::Windup,ActorAiState::Active]{let mut a=wraith();reach(&mut a,p(10.,8.),phase,&arena());let mut w=WorldStateV3::new("mist_harbor",8,p(10.,8.)).unwrap();w.generic_actors=vec![a.clone()];w.player_hp=0;for seq in 1..30{assert!(owner(&mut w,&arena(),seq,(0.,0.),&[]).actor_runtime.is_empty());assert_eq!(w.generic_actors,vec![a.clone()]);}}
}
