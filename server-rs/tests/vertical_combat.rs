//! Engine geometry fixtures. Numerical reach and the 2 m deck are development
//! tuning; genuine public-runtime boarding/entry/save proofs live separately.
use wuxian_horror_ch1::{continuous_combat::{combat_vertical_overlap,CombatIntent,CombatEvent},continuous_input::InputSample,continuous_kcc::{Aabb,StaticKccWorld,KccTerrainRegion},moving_support::StandingDeckDefinition,effects::TerrainTag,world_v3::{ActorRuntime,ActorAiState,ActorAttackKind,ActorRuntimeEvent,OrdinaryTerrainVariant,WorldStateV3,Vec3,step_world,CwStepInput},sentinel_ai::{Sentinel,SentinelState,tick_sentinel}};
const DT:f32=1.0/60.0;
fn p(x:f32,y:f32,z:f32)->Vec3 {Vec3::new(x,y,z).unwrap()}
fn arena()->StaticKccWorld {StaticKccWorld::new(Aabb::new(0.,30.,0.,20.).unwrap(),vec![]).with_standing_decks(vec![StandingDeckDefinition{id:"upper".into(),polygon:vec![[1.,1.],[28.,1.],[28.,18.],[1.,18.]],height_m:2.}]).unwrap()}
fn actor(kind:&str,y:f32)->ActorRuntime {ActorRuntime::spawn("fixture",kind,p(6.,y,5.)).unwrap()}
fn reach_windup(a:&mut ActorRuntime,kcc:&StaticKccWorld,target:Vec3,special:bool) {for _ in 0..300 {if a.state==ActorAiState::Windup || a.warden.as_ref().is_some_and(|w|matches!(w.stage,wuxian_horror_ch1::world_v3::WardenStage::Windup)){return;}a.tick(target,kcc,DT,special);}panic!("no windup {} {:?}",a.entity_type,a.state);}
#[test]
fn shared_height_reach_is_symmetric_finite_and_boundary_exact() {
    for (delta,expected) in [(0.,true),(0.945,true),(1.,true),(1.0001,false),(2.,false)] {assert_eq!(combat_vertical_overlap(p(2.,0.,2.),p(2.,delta,2.)),expected);assert_eq!(combat_vertical_overlap(p(2.,delta,2.),p(2.,0.,2.)),expected);}
    for bad in [f32::NAN,f32::INFINITY,f32::NEG_INFINITY] {assert!(!combat_vertical_overlap(p(2.,0.,2.),Vec3{y_m:bad,..p(2.,0.,2.)}));}
}
#[test]
fn player_primary_pulse_pierce_and_legacy_attacks_are_symmetric_across_decks() {
    for intent in [CombatIntent::ActionAttack{request_id:1},CombatIntent::Pulse{request_id:1},CombatIntent::Pierce{request_id:1},CombatIntent::Attack{request_id:1}] {
        for kind in ["grey_hive.infected_maintenance_worker","enemy.grey_hive.swarm"] {for (py,ay,hit) in [(0.,0.,true),(2.,2.,true),(2.,0.,false),(0.,2.,false)] {
            let kcc=arena();let mut w=WorldStateV3::new("grey_hive",7,p(5.,py,5.)).unwrap();w.combat_state.qer_v1_active=true;w.generic_actors=vec![actor(kind,ay)];let hp=w.generic_actors[0].hp;
            let public=w.generic_actors[0].view();let point=public.members.as_ref().and_then(|members|members.iter().find(|member|member.active)).map_or(public.transform.position_m,|member|member.position_m);
            let dx=point.x_m-5.;let dz=point.z_m-5.;let length=dx.hypot(dz);
            for seq in 1..=80 {let sample=InputSample::new(7,seq,seq*17,0.,0.).unwrap().with_aim(dx/length,dz/length).unwrap();let intents=if seq==1{vec![intent.clone()]}else{vec![]};step_world(&mut w,&kcc,CwStepInput{sample:&sample,dt_s:DT,combat:&intents}).unwrap();}
            assert_eq!(w.generic_actors[0].hp<hp,hit,"{kind} {py} {ay} {intent:?}");
        }}
    }
}
#[test]
fn every_ordinary_attack_rechecks_height_after_its_committed_warning() {
    for (kind,range,expected_kind) in [("enemy.clockworks.pressure_drone",3.,ActorAttackKind::PressureShot),("enemy.clockworks.furnace_hound",1.,ActorAttackKind::Bite),("enemy.clockworks.furnace_hound",3.,ActorAttackKind::Leap),("enemy.grey_hive.brute",1.,ActorAttackKind::Slam),("enemy.grey_hive.brute",3.,ActorAttackKind::Charge),("enemy.grey_hive.swarm",1.,ActorAttackKind::Lunge),("enemy.mist_harbor.tidebound",1.,ActorAttackKind::Swing),("enemy.mist_harbor.tidebound",3.,ActorAttackKind::Charge)] {
        for (ay,ty,hit) in [(0.,0.,true),(2.,2.,true),(0.,2.,false),(2.,0.,false)] {let kcc=arena();let mut a=actor(kind,ay);reach_windup(&mut a,&kcc,p(6.+range,ay,5.),false);assert_eq!(a.current_attack,Some(expected_kind));let mut damage=0;let mut impact=false;
            for _ in 0..400 {let out=a.tick(p(6.+range,ty,5.),&kcc,DT,false);damage+=out.damage_to_player.unwrap_or(0);impact|=out.events.iter().any(|event|matches!(event,ActorRuntimeEvent::AttackImpact{..}));if a.state==ActorAiState::Cooldown{break;}}
            assert!(impact,"{kind} {expected_kind:?}");assert_eq!(damage>0,hit,"{kind} {expected_kind:?} {ay} {ty}");
        }
    }
}
#[test]
fn separate_floor_does_not_consume_a_pressure_shot_before_same_floor_contact() {
    let kcc=arena();let mut a=actor("enemy.clockworks.pressure_drone",0.);reach_windup(&mut a,&kcc,p(9.,0.,5.),false);
    while a.state==ActorAiState::Windup {a.tick(p(9.,2.,5.),&kcc,DT,false);}
    assert_eq!(a.state,ActorAiState::Active);
    for _ in 0..4 {let out=a.tick(p(6.2,2.,5.),&kcc,DT,false);assert!(out.damage_to_player.is_none());assert!(!a.ordinary.as_ref().unwrap().active.as_ref().unwrap().hit_resolved);}
    let mut damage=0;for _ in 0..100 {damage+=a.tick(p(9.,0.,5.),&kcc,DT,false).damage_to_player.unwrap_or(0);if a.state==ActorAiState::Cooldown{break;}}
    assert!(damage>0);
}
#[test]
fn legacy_melee_pressure_wave_and_sentinel_do_not_hit_another_floor() {
    for (kind,special) in [("grey_hive.infected_maintenance_worker",false),("enemy.clockworks.forged_guard_elite",true),("enemy.clockworks.prime_regulator",true)] {for ay in [0.,2.] {let kcc=arena();let mut a=actor(kind,ay);reach_windup(&mut a,&kcc,p(7.,ay,5.),special);let mut seen=false;for _ in 0..400 {let out=a.tick(p(7.,2.-ay,5.),&kcc,DT,special);assert!(out.damage_to_player.is_none());seen|=out.events.iter().any(|e|matches!(e,ActorRuntimeEvent::AttackImpact{hit_player:false,..}));if a.state==ActorAiState::Cooldown{break;}}assert!(seen);}}
    for state in [SentinelState::Attack,SentinelState::HeavyAttack] {let kcc=arena();let mut s=Sentinel::new("fixture",p(6.,0.,5.),100);s.state=state;s.state_remaining_ms=1;let (_,events)=tick_sentinel(&mut s,p(7.,2.,5.),DT,&kcc);assert!(!events.iter().any(|e|matches!(e,CombatEvent::PlayerDamaged{..})));}
}
#[test]
fn higher_floor_is_not_sensed_and_raised_actor_cannot_walk_off_static_support() {
    for kind in ["grey_hive.infected_maintenance_worker","enemy.clockworks.pressure_drone","enemy.clockworks.furnace_hound","enemy.grey_hive.brute","enemy.grey_hive.swarm","enemy.mist_harbor.tidebound"] {
        let kcc=arena();let mut a=actor(kind,0.);for _ in 0..100 {assert!(a.tick(p(7.,2.,5.),&kcc,DT,false).events.is_empty());}assert_eq!(a.state,ActorAiState::Idle);
        let deck=StandingDeckDefinition{id:"small".into(),polygon:vec![[4.,4.],[8.,4.],[8.,8.],[4.,8.]],height_m:2.};let kcc=StaticKccWorld::new(Aabb::new(0.,30.,0.,20.).unwrap(),vec![]).with_standing_decks(vec![deck]).unwrap();let mut a=actor(kind,2.);for _ in 0..300 {a.tick(p(20.,2.,5.),&kcc,DT,false);assert!(kcc.stable_actor_footprint(a.position_m,a.body_radius_m()),"{kind}");}
    }
}
#[test]
fn dry_raised_tidebound_uses_normal_profile_above_floor_water() {
    let kcc=arena().with_terrain_regions(vec![KccTerrainRegion{id:"water".into(),tag:TerrainTag::WaterShallow,polygon:vec![[1.,1.],[28.,1.],[28.,18.],[1.,18.]],surface_velocity_mps:None}]);
    for (height,variant,windup) in [(0.,OrdinaryTerrainVariant::Terrain,900),(2.,OrdinaryTerrainVariant::Normal,700)] {let mut a=actor("enemy.mist_harbor.tidebound",height);reach_windup(&mut a,&kcc,p(7.,height,5.),false);assert_eq!(a.ordinary.as_ref().unwrap().terrain_variant,Some(variant));assert_eq!(a.state_remaining_ms,windup);}
}

#[test]
fn both_committed_warden_attacks_miss_across_floors() {
    for ay in [0.,2.] {
        let kcc=arena();let mut a=actor("enemy.mist_harbor.resonance_warden",ay);
        for expected in [wuxian_horror_ch1::world_v3::WardenAttack::Strike,wuxian_horror_ch1::world_v3::WardenAttack::Pulse] {
            reach_windup(&mut a,&kcc,p(7.,ay,5.),true);assert_eq!(a.warden.as_ref().unwrap().attack,Some(expected));
            let mut resolved=false;for _ in 0..300 {let out=a.tick(p(7.,2.-ay,5.),&kcc,DT,true);assert!(out.damage_to_player.is_none());if a.warden.as_ref().unwrap().attack.is_none(){resolved=true;break;}}
            assert!(resolved);assert!(a.validate());
        }
    }
}

#[test]
fn hazard_bands_are_finite_explicit_and_default_floor_only() {
    use wuxian_horror_ch1::{scene_runtime::HazardDefinition,moving_support::HeightRange};
    let mut hazard:HazardDefinition=serde_json::from_value(serde_json::json!({"id":"height","kind":"steam_jet","polygon":[[1,1],[2,1],[2,2],[1,2]]})).unwrap();
    assert!(hazard.height_allows(0.));assert!(hazard.height_allows(1.));assert!(!hazard.height_allows(1.0001));assert!(!hazard.height_allows(2.));
    hazard.height_range_m=Some(HeightRange([1.9,2.1]));assert!(!hazard.height_allows(0.));assert!(hazard.height_allows(2.));
    for range in [[2.1,1.9],[0.,f32::NAN],[-0.1,1.],[2.,3.1]] {hazard.height_range_m=Some(HeightRange(range));assert!(!hazard.height_allows(2.));}
}
