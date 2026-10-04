//! Engine-only grouped combat fixtures; native placement is a separate admission.
use super::*;
use crate::{continuous_combat::{CombatEvent, CombatIntent}, continuous_input::InputSample,
    continuous_kcc::{Aabb, StaticKccWorld}, world_v3::{step_world, CwStepInput, WorldStateV3}};
const SWARM: &str = "enemy.grey_hive.swarm";
const DT: f32 = 1.0 / 60.0;
fn p(x:f32,z:f32)->Vec3 { Vec3::new(x,0.0,z).unwrap() }
fn arena()->StaticKccWorld { StaticKccWorld::new(Aabb::new(0.0,30.0,0.0,20.0).unwrap(),vec![]) }
fn swarm()->ActorRuntime { ActorRuntime::spawn("group-fixture",SWARM,p(6.0,8.0)).unwrap() }
fn world()->WorldStateV3 {
    let mut w=WorldStateV3::new("grey_hive",8,p(5.0,8.0)).unwrap();
    w.combat_state.qer_v1_active=true;w.generic_actors.push(swarm());w
}
fn step(w:&mut WorldStateV3,seq:u64,intents:&[CombatIntent])->crate::world_v3::CwStepOutput {
    let sample=InputSample::new(8,seq,seq*17,0.0,0.0).unwrap().with_aim(1.0,0.0).unwrap();
    step_world(w,&arena(),CwStepInput{sample:&sample,dt_s:DT,combat:intents}).unwrap()
}
#[test]
fn grouped_profile_has_four_independently_damaged_members() {
    let mut a=swarm();assert!(a.validate());assert_eq!(a.group.as_ref().unwrap().member_hp,[12;4]);
    assert_eq!(a.combat_points(p(5.0,8.0),&arena()).len(),4);
    let events=a.take_member_hits(&[0,0,0,1],10,35).unwrap();
    assert_eq!(a.group.as_ref().unwrap().member_hp,[2,2,12,12]);assert_eq!(a.hp,28);
    assert_eq!(events.iter().filter(|e|matches!(e,ActorRuntimeEvent::MemberHit{..})).count(),2);
    assert_eq!(events.iter().filter(|e|matches!(e,ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Stagger,..})).count(),1);
    assert!(a.validate());let old=a.clone();assert!(a.take_member_hits(&[0,8],10,35).is_err());assert_eq!(a,old);
}
#[test]
fn actual_primary_hits_one_member_and_pulse_hits_each_member_once() {
    for pulse in [false,true] {
        let mut w=world();let mut impacts=0;
        for seq in 1..=20 {
            let intents=if seq==1{vec![if pulse{CombatIntent::Pulse{request_id:1}}else{CombatIntent::ActionAttack{request_id:1}}]}else{vec![]};
            let out=step(&mut w,seq,&intents);assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::IntentRejected{..})));
            impacts+=out.combat.iter().filter(|e|matches!(e,CombatEvent::ActionImpact{..})).count();
        }
        assert_eq!(w.generic_actors[0].hp,if pulse{8}else{36});assert_eq!(impacts,1);assert_eq!(w.player_hp,100);
        let a=&w.generic_actors[0];assert!(a.validate());
        if pulse{assert_eq!(a.group.as_ref().unwrap().member_hp,[2;4]);}
        else{assert_eq!(a.group.as_ref().unwrap().member_hp.iter().filter(|hp|**hp==0).count(),1);}
    }
}
fn reach(a:&mut ActorRuntime,target:Vec3,state:ActorAiState) {
    for _ in 0..150 { if a.state==state{return;} a.tick(target,&arena(),DT,false);assert!(a.validate()); }
    panic!("did not reach {state:?}");
}
#[test]
fn primary_chooses_nearest_living_point_globally_including_legacy_attack() {
    for legacy in [false,true] { for close in [false,true] {
        let mut w=world();w.generic_actors.push(ActorRuntime::spawn("other","grey_hive.infected_maintenance_worker",p(if close{5.3}else{5.9},8.0)).unwrap());
        for seq in 1..=8 {
            let intents=if seq==1{vec![if legacy{CombatIntent::Attack{request_id:1}}else{CombatIntent::ActionAttack{request_id:1}}]}else{vec![]};step(&mut w,seq,&intents);
        }
        assert_eq!(w.generic_actors[0].hp,if close{48}else{36});
        assert_eq!(w.generic_actors[1].hp,if close{50-if legacy{25}else{16}}else{50});
    }}
    let mut w=world();w.sentinels.push(crate::sentinel_ai::Sentinel::new("nearest-sentinel",p(5.2,8.0),100));
    for seq in 1..=8 {let intents=if seq==1{vec![CombatIntent::ActionAttack{request_id:1}]}else{vec![]};step(&mut w,seq,&intents);}
    assert_eq!(w.generic_actors[0].hp,48);assert_eq!(w.sentinels[0].hp,84);
}
#[test]
fn action_geometry_hits_only_intersected_members_and_wall_blocks_member_hits() {
    // A narrow line through the lower lane intersects exactly its two members.
    let mut w=world();w.player.position_m.z_m=7.45;
    for seq in 1..=22 {let intents=if seq==1{vec![CombatIntent::Pierce{request_id:1}]}else{vec![]};step(&mut w,seq,&intents);}
    assert_eq!(w.generic_actors[0].group.as_ref().unwrap().member_hp,[0,0,12,12]);
    // Pulse at the outer radius reaches the left pair, not the whole group by center.
    let mut w=world();w.generic_actors[0]=ActorRuntime::spawn("group-fixture",SWARM,p(8.0,8.0)).unwrap();
    for seq in 1..=20 {let intents=if seq==1{vec![CombatIntent::Pulse{request_id:1}]}else{vec![]};step(&mut w,seq,&intents);}
    assert_eq!(w.generic_actors[0].group.as_ref().unwrap().member_hp,[2,12,2,12]);
    let wall=StaticKccWorld::new(Aabb::new(0.0,30.0,0.0,20.0).unwrap(),vec![Aabb::new(6.0,6.01,4.0,12.0).unwrap()]);
    let mut w=world();w.generic_actors[0]=ActorRuntime::spawn("group-fixture",SWARM,p(7.0,8.0)).unwrap();
    assert!(wall.can_occupy(w.generic_actors[0].position_m,0.95));assert!(w.generic_actors[0].combat_points(w.player.position_m,&wall).is_empty());
    for seq in 1..=20 {let sample=InputSample::new(8,seq,seq*17,0.0,0.0).unwrap();let intents=if seq==1{vec![CombatIntent::Pulse{request_id:1}]}else{vec![]};
        let out=step_world(&mut w,&wall,CwStepInput{sample:&sample,dt_s:DT,combat:&intents}).unwrap();
        assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::ActionImpact{..})));
    }
    assert_eq!(w.generic_actors[0].hp,48);
}
#[test]
fn duplicate_requests_never_repeat_member_damage_and_two_real_pulses_disperse_once() {
    let mut w=world();let mut death=0;let mut member_deaths=std::collections::BTreeSet::new();
    for seq in 1..=230 {
        let intents=if [1,2,100,161].contains(&seq){vec![CombatIntent::Pulse{request_id:if seq==161{161}else{1}}]}else{vec![]};
        let out=step(&mut w,seq,&intents);
        if [2,100].contains(&seq){assert!(out.combat.iter().any(|e|matches!(e,CombatEvent::IntentRejected{reason,..}if reason=="duplicate_or_stale_request")));}
        for e in out.actor_runtime {match e {
            ActorRuntimeEvent::EnemyCue{kind:ActorCueKind::Death,..}=>death+=1,
            ActorRuntimeEvent::MemberHit{actor_id,member_id,destroyed:true,..}=>{assert_eq!(actor_id,"group-fixture");assert!(member_deaths.insert(member_id));},_=>{}
        }}
        if seq==100{assert_eq!(w.generic_actors[0].hp,8);}
    }
    assert_eq!(death,1);assert_eq!(member_deaths.len(),4);assert_eq!(w.generic_actors[0].hp,0);assert_eq!(w.generic_actors[0].state,ActorAiState::Dead);assert!(w.player_hp>0);
    let dead=w.generic_actors[0].clone();let hp=w.player_hp;
    for seq in 231..=310{assert!(step(&mut w,seq,&[]).actor_runtime.is_empty());}
    assert_eq!(w.generic_actors[0],dead);assert_eq!(w.player_hp,hp);
}
#[test]
fn lunge_commits_direction_encloses_cluster_and_damage_scales_with_living_members() {
    for living in [4,2] {
        let mut a=swarm();if living==2{a.take_member_hits(&[0,1],12,0).unwrap();}
        reach(&mut a,p(8.0,8.0),ActorAiState::Active);assert_eq!(a.current_attack,Some(ActorAttackKind::Lunge));
        let saved=a.clone();let mut damage=vec![];
        while a.state==ActorAiState::Active{let out=a.tick(p(8.0,8.0),&arena(),DT,false);if let Some(d)=out.damage_to_player{damage.push(d);}assert!(a.validate());}
        assert_eq!(damage,vec![if living==4{12}else{6}]);
        let mut dodged=saved.clone();while dodged.state==ActorAiState::Active{assert!(dodged.tick(p(8.0,12.0),&arena(),DT,false).damage_to_player.is_none());}
        assert_eq!(dodged.position_m.z_m,8.0);
        let wall=StaticKccWorld::new(Aabb::new(0.0,30.0,0.0,20.0).unwrap(),vec![Aabb::new(7.1,7.11,5.0,11.0).unwrap()]);
        let mut blocked=saved;while blocked.state==ActorAiState::Active{assert!(blocked.tick(p(9.0,8.0),&wall,0.25,false).damage_to_player.is_none());}
        assert!(blocked.position_m.x_m+0.95<=7.1, "blocked position {:?}", blocked.position_m); assert!(wall.can_occupy(blocked.position_m,0.95));assert!(blocked.validate());
    }
}
#[test]
fn guard_pause_stale_input_and_fatal_owner_preserve_group_authority() {
    let mut w=world();let mut sent=false;let mut blocks=0;
    for seq in 1..=80 {
        let a=&w.generic_actors[0];let guard=!sent&&a.state==ActorAiState::Windup&&a.state_remaining_ms<=220;
        let intents=if guard{sent=true;vec![CombatIntent::GuardStart{request_id:seq}]}else{vec![]};
        let out=step(&mut w,seq,&intents);blocks+=out.combat.iter().filter(|e|matches!(e,CombatEvent::GuardImpact{..})).count();
    }
    assert_eq!(blocks,1);assert_eq!(w.player_hp,100);assert_eq!(w.generic_actors[0].hp,48);
    let mut a=swarm();reach(&mut a,p(8.0,8.0),ActorAiState::Active);let before=a.clone();
    assert_eq!(a.tick(p(8.0,8.0),&arena(),0.0,false),ActorTickOutput::default());assert_eq!(a,before);
    let mut w=world();w.generic_actors=vec![a];let wrong=InputSample::new(9,1,17,0.0,0.0).unwrap();
    assert!(step_world(&mut w,&arena(),CwStepInput{sample:&wrong,dt_s:DT,combat:&[]}).is_err());assert_eq!(w.generic_actors[0],before);
    w.player_hp=0;for seq in 1..=40{assert!(step(&mut w,seq,&[]).actor_runtime.is_empty());}assert_eq!(w.generic_actors[0],before);
}
#[test]
fn saved_partial_lunge_and_dead_members_roundtrip_without_reset_or_duplicate_hits() {
    for phase in ["partial","windup","active","resolved","dead"] {
        let mut a=swarm();a.take_member_hits(&[0],12,0).unwrap();
        if phase!="partial"&&phase!="dead"{reach(&mut a,p(8.0,8.0),if phase=="windup"{ActorAiState::Windup}else{ActorAiState::Active});}
        if phase=="resolved"{while !a.ordinary.as_ref().unwrap().active.as_ref().unwrap().hit_resolved{a.tick(p(8.0,8.0),&arena(),DT,false);}}
        if phase=="dead"{a.take_member_hits(&[1,2,3],12,0).unwrap();}
        let mut save:crate::save_v5::SaveV5=serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();save.generic_actors=vec![a.clone()];save.validate().unwrap();
        let dir=std::env::temp_dir().join(format!("swarm-{phase}-{}",std::process::id()));crate::save_v5::write_save(&dir,&save).unwrap();let bytes=std::fs::read(crate::save_v5::save_path(&dir)).unwrap();
        let loaded=crate::save_v5::read_save(&dir).unwrap().restore_state().unwrap();assert_eq!(loaded.world.generic_actors,vec![a.clone()]);let mut b=loaded.world.generic_actors[0].clone();
        for _ in 0..140{assert_eq!(a.tick(p(8.0,8.0),&arena(),DT,false),b.tick(p(8.0,8.0),&arena(),DT,false));assert_eq!(a,b);}
        assert_eq!(std::fs::read(crate::save_v5::save_path(&dir)).unwrap(),bytes);
    }
}
#[test]
fn malformed_group_save_and_failed_damage_are_atomic_and_old_profiles_reject_group_state() {
    let mut good=swarm();good.take_member_hits(&[0,2],5,0).unwrap();
    let mut save:crate::save_v5::SaveV5=serde_json::from_str(include_str!("../../tests/fixtures/capability-save-profiles/legacy-v5.json")).unwrap();save.generic_actors=vec![good.clone()];
    let dir=std::env::temp_dir().join(format!("swarm-invalid-{}",std::process::id()));crate::save_v5::write_save(&dir,&save).unwrap();let bytes=std::fs::read(crate::save_v5::save_path(&dir)).unwrap();
    for fault in ["absent","missing","extra","overflow","sum","dead","foreign"] {
        let mut a=good.clone();match fault{
            "absent"=>a.group=None,"missing"=>{a.group.as_mut().unwrap().member_hp.pop();},"extra"=>a.group.as_mut().unwrap().member_hp.push(0),
            "overflow"=>a.group.as_mut().unwrap().member_hp[0]=13,"sum"=>a.hp+=1,"dead"=>a.state=ActorAiState::Dead,"foreign"=>a.entity_type="enemy.clockworks.furnace_hound".into(),_=>unreachable!()}
        assert!(!a.validate(),"{fault}");let original=a.clone();assert!(a.take_member_hits(&[0],10,35).is_err());assert_eq!(a,original);
        let mut bad=save.clone();bad.generic_actors=vec![a];assert!(crate::save_v5::write_save(&dir,&bad).is_err());assert_eq!(std::fs::read(crate::save_v5::save_path(&dir)).unwrap(),bytes);
    }
    let mut legacy=ActorRuntime::spawn("old","grey_hive.infected_security",p(5.0,8.0)).unwrap();legacy.group=good.group.clone();assert!(!legacy.validate());
    let mut foreign=ActorRuntime::spawn("old","enemy.clockworks.furnace_hound",p(5.0,8.0)).unwrap();foreign.state=ActorAiState::Windup;foreign.current_attack=Some(ActorAttackKind::Lunge);foreign.windup_origin_m=Some(foreign.position_m);foreign.state_remaining_ms=1;assert!(!foreign.validate());
}
#[test]
fn malformed_group_controller_and_owner_boundaries_never_mutate_or_panic() {
    let mut valid=swarm();reach(&mut valid,p(8.0,8.0),ActorAiState::Active);
    for fault in ["absent","empty","missing","extra","sum","dead","all-dead"] {
        let mut a=valid.clone();match fault{
            "absent"=>a.group=None,"empty"=>a.group.as_mut().unwrap().member_hp.clear(),
            "missing"=>{a.group.as_mut().unwrap().member_hp.pop();},"extra"=>a.group.as_mut().unwrap().member_hp.push(0),
            "sum"=>a.hp-=1,"dead"=>a.state=ActorAiState::Dead,"all-dead"=>{a.group.as_mut().unwrap().member_hp.fill(0);a.hp=0;},_=>unreachable!()}
        let before=a.clone();assert!(!a.validate());
        assert_eq!(a.tick(p(8.0,8.0),&arena(),0.25,false),ActorTickOutput::default());assert_eq!(a,before);
        assert!(a.take_damage_with_stagger(10,35).is_empty());assert_eq!(a,before);a.take_damage(10);assert_eq!(a,before);
        let mut w=world();w.generic_actors=vec![a];for seq in 1..=3{assert!(step(&mut w,seq,&[]).actor_runtime.is_empty());}
        assert_eq!(w.generic_actors,vec![before]);assert_eq!(w.player_hp,100);
    }
}
