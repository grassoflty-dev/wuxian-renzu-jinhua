//! Optional, genuine Swarm encounters along the fresh production campaign.
//! Decisions use public member positions/liveness, player resources, and cues only.
use super::*;
use std::{collections::BTreeMap, path::Path};
use wuxian_horror_ch1::{
    formal_runtime::{ActionCommandRequest, ActionKind},
    world_v3::{ActorView, PresentationEvent, Vec3, WorldView},
};

const SWARM: &str = "enemy.grey_hive.swarm";
const REACTION_TICKS: u64 = 9; // Same fixed 150 ms response as the accepted CW driver.

fn input(runtime: &FormalRuntime, view: &WorldView, movement: (f32, f32), aim: (f32, f32)) -> WorldView {
    runtime.submit_input(InputSample::new(view.world_epoch, view.ack_seq + 1,
        NEXT_TIME.fetch_add(17, Ordering::Relaxed), movement.0, movement.1).unwrap()
        .with_aim(aim.0, aim.1).unwrap(), vec![]).unwrap()
}

fn target_point(actor: &ActorView, player: Vec3) -> Vec3 {
    actor.members.as_ref().map(|members| members.iter().filter(|member| member.active)
        .min_by(|a,b| distance(a.position_m,player).total_cmp(&distance(b.position_m,player)))
        .expect("active group exposes a living public member").position_m)
        .unwrap_or(actor.transform.position_m)
}
fn distance(a: Vec3,b: Vec3)->f32 {(a.x_m-b.x_m).hypot(a.z_m-b.z_m)}

#[derive(Default)]
struct GroupProof { primary: bool, pulse: bool, restored: bool, deaths: usize, dispersed: BTreeSet<String> }
struct Pending { kind: ActionKind, until: u64, started: bool, impact: bool, members: BTreeMap<String,BTreeSet<String>> }

/// Public Save/Continue, preserving the earned partial/dead group and player HP.
/// Saved private fields are assertions only, never combat decision inputs.
fn save_continue(runtime: &FormalRuntime, save_dir: &Path, partial: bool) -> WorldView {
    let before=runtime.pause().unwrap();
    runtime.save().unwrap();
    let saved=save_v6::read_save(save_dir).unwrap();
    let prepared=runtime.continue_saved().unwrap();
    assert_eq!(prepared.scene_id,before.scene_id);
    assert_eq!(prepared.player.current_hp,before.player.current_hp);
    for actor in before.actors.iter().filter(|a|a.entity_type==SWARM) {
        let after=prepared.actors.iter().find(|a|a.entity_id==actor.entity_id).unwrap();
        assert_eq!(after,actor,"Continue must preserve public members and liveness exactly");
    }
    assert!(!runtime.presentation_events_since(prepared.world_epoch,0).unwrap().iter()
        .any(|e|matches!(e.kind.as_str(),"EnemyDeath"|"EnemyMemberDisperse")),
        "Continue cannot replay member or group deaths");
    let ready=runtime.scene_ready(prepared.entry_token.as_ref().unwrap(),true).unwrap();
    assert_eq!(ready.player.current_hp,before.player.current_hp);
    runtime.save().unwrap();
    let restored=save_v6::read_save(save_dir).unwrap();
    assert_eq!(restored.save.generic_actors,saved.save.generic_actors,
        "Save/Continue retains exact earned actor/member state while entry is held");
    assert_eq!(restored.save.progression,saved.save.progression);
    assert_eq!(restored.world_persistent_v1,saved.world_persistent_v1);
    println!("Genuine Swarm {} Save/Continue: {} at {} HP, save {}",if partial{"partial"}else{"defeated"},before.scene_id,before.player.current_hp,save_dir.display());
    runtime.resume().unwrap()
}

pub(super) fn clear_scene(runtime: &FormalRuntime, mut view: WorldView, save_dir: &Path) -> WorldView {
    let scene=view.scene_id.clone();
    assert!(matches!(scene.as_str(),"gh_lockdown"|"gh_deep_decon"));
    let mut proofs=BTreeMap::<String,GroupProof>::new();
    for actor in view.actors.iter().filter(|a|a.entity_type==SWARM) {
        assert!(actor.active);
        let members=actor.members.as_ref().unwrap();assert_eq!(members.len(),4);
        for (index,member) in members.iter().enumerate() {
            assert!(member.active);assert!(member.radius_m>0.0);
            assert_eq!(member.member_id,format!("{}/member/{}",actor.entity_id,index+1));
        }
        let json=serde_json::to_value(actor).unwrap();
        assert!(!json.to_string().contains("Hp"),"public group projection cannot leak HP");
        proofs.insert(actor.entity_id.clone(),GroupProof::default());
    }
    assert_eq!(proofs.len(),2,"both authored optional groups must be exercised");
    let mut last_event=0;let mut deferred=Vec::new();
    let mut warnings=BTreeMap::<String,(PresentationEvent,u64)>::new();
    let mut pending:Option<Pending>=None;
    let mut attack_after=0;let mut hold_until=0;let mut pulse_after=0;
    let mut attacks=0;let mut lunges=0;let mut evades=0;
    let mut enemy_damage_cues=0;let mut hazard_damage_cues=0;
    let mut previous_hp=view.player.current_hp;
    for hazard in view.hazards.iter().filter(|h|h.environment.is_some()) {
        println!("Swarm public hazard {}: polygon {:?}, environment {:?}",hazard.entity_id,hazard.polygon_m,hazard.environment);
    }
    for step in 0..9000 {
        assert!(view.player.current_hp>0,"genuine Swarm campaign died in {scene} at input {step}; earned save {}",save_dir.display());
        if view.player.current_hp!=previous_hp {
            println!("Swarm public health {scene}: tick {}, {} -> {} HP at {:?}",view.server_tick,previous_hp,view.player.current_hp,view.player.transform.position_m);
            previous_hp=view.player.current_hp;
        }
        for event in runtime.presentation_events_since(view.world_epoch,last_event).unwrap(){last_event=last_event.max(event.event_id);deferred.push(event);}
        for event in std::mem::take(&mut deferred) {
            if event.server_tick>view.server_tick {deferred.push(event);continue;}
            // Attribution is public presentation evidence, not an enemy-state input.
            if matches!(event.kind.as_str(),"Damaged"|"EnvironmentHazardDamage") {
                if event.kind=="Damaged" {enemy_damage_cues+=1;} else {hazard_damage_cues+=1;}
                println!("Swarm public damage {scene}: {} at tick {}, position {:?}, visible environments {:?}",event.kind,event.server_tick,event.position_m,
                    view.hazards.iter().filter_map(|h|h.environment.as_ref().map(|e|(&h.entity_id,e))).collect::<Vec<_>>());
            }
            if event.kind=="EnemyAttackTelegraph" {
                assert_eq!(event.attack_kind.as_deref(),Some("lunge"));
                assert!(event.duration_ms.is_some_and(|n|n>0));assert!(event.range_m.is_some_and(|n|n>0.0));
                warnings.insert(event.actor_id.clone().unwrap(),(event.clone(),view.server_tick));lunges+=1;
            }else if event.kind=="EnemyLungeMotion" {
                warnings.entry(event.actor_id.clone().unwrap()).or_insert((event.clone(),view.server_tick));
            }else if matches!(event.kind.as_str(),"EnemyAttackImpact"|"EnemyStagger"|"EnemyDeath") {
                if let Some(id)=&event.actor_id {warnings.remove(id);}
            }
            if let Some(action)=pending.as_mut() {
                let(start,hit)=match action.kind {ActionKind::PrimaryAttack=>("AttackStarted","Hit"),ActionKind::Pulse=>("PulseCast","PulseHit"),_=>unreachable!()};
                if event.kind==start {action.started=true;}
                if event.kind==hit {action.impact=true;}
                if matches!(event.kind.as_str(),"EnemyMemberHit"|"EnemyMemberDisperse") {
                    action.members.entry(event.actor_id.clone().unwrap()).or_default().insert(event.member_id.clone().unwrap());
                }
            }
            if let Some(proof)=event.actor_id.as_ref().and_then(|id|proofs.get_mut(id)) {
                if event.kind=="EnemyDeath" {proof.deaths+=1;assert_eq!(proof.deaths,1,"no duplicate group death");}
                if event.kind=="EnemyMemberDisperse" {assert!(proof.dispersed.insert(event.member_id.unwrap()),"no duplicate member death");}
            }
        }
        if view.player.action_state=="idle" && pending.as_ref().is_some_and(|action|view.server_tick>=action.until) {
            let action=pending.take().unwrap();assert!(action.started,"submitted action must have a real public acceptance cue");
            if !action.members.is_empty(){assert!(action.impact,"member changes require an accepted real action impact");}
            if matches!(action.kind,ActionKind::PrimaryAttack) {
                assert!(action.members.values().map(BTreeSet::len).sum::<usize>()<=1,"Primary can hit only one living member");
                for(id,members)in action.members {assert_eq!(members.len(),1);proofs.get_mut(&id).unwrap().primary=true;}
            }else{
                for(id,members)in action.members {if members.len()>=2 {proofs.get_mut(&id).unwrap().pulse=true;println!("Genuine Swarm grouped Pulse: {id}, {} distinct living members",members.len());}}
            }
            let partial=proofs.iter().find(|(id,p)|p.primary&&!p.restored&&view.actors.iter().any(|a|&a.entity_id==*id&&a.members.as_ref().unwrap().iter().filter(|m|m.active).count()==3)).map(|(id,_)|id.clone());
            if let Some(id)=partial {
                proofs.get_mut(&id).unwrap().restored=true;
                view=save_continue(runtime,save_dir,true);
                last_event=0;deferred.clear();warnings.clear();
                attack_after=view.server_tick+72;hold_until=0;pulse_after=attack_after;
                continue;
            }
        }
        warnings.retain(|id,_|view.actors.iter().any(|a|a.entity_id==*id&&a.active));
        let player=view.player.transform.position_m;
        // Approach the requested groups, but do not ignore a closer visible threat.
        // Legacy enemies outside immediate defensive range are not extra objectives.
        let groups_alive=view.actors.iter().any(|a|a.active&&a.entity_type==SWARM);
        let target=view.actors.iter().filter(|a|groups_alive&&a.active&&a.actor_kind=="enemy"&&(a.entity_type==SWARM||distance(a.transform.position_m,player)<3.0))
            .min_by(|a,b|distance(target_point(a,player),player).total_cmp(&distance(target_point(b,player),player)));
        let Some(target)=target else {
            if pending.is_some(){view=input(runtime,&view,(0.0,0.0),(1.0,0.0));continue;}
            for(id,proof)in &proofs {
                assert!(proof.primary&&proof.pulse&&proof.restored,"{id} requires single-member Primary, multi-member Pulse, and partial Save/Continue");
                assert_eq!(proof.deaths,1,"{id} complete real defeat");assert_eq!(proof.dispersed.len(),4);
                assert!(view.actors.iter().find(|a|&a.entity_id==id).unwrap().members.as_ref().unwrap().iter().all(|m|!m.active));
            }
            assert!(lunges>0&&evades>0,"real Swarm encounters must exercise public lunge counterplay");
            println!("Campaign genuine Swarm {scene}: {step} inputs, {attacks} accepted actions, {lunges} lunge warnings, {evades} cue-aware movements (150 ms reaction), {enemy_damage_cues} enemy-damage cues, {hazard_damage_cues} hazard-damage cues, {} HP",view.player.current_hp);
            return save_continue(runtime,save_dir,false);
        };
        if step%600==0 {println!("Swarm combat progress {scene}: input {step}, target {}, player ({:.2},{:.2}), {} HP, {attacks} actions",target.entity_id,player.x_m,player.z_m,view.player.current_hp);}
        let target_pos=target_point(target,player);let dx=target_pos.x_m-player.x_m;let dz=target_pos.z_m-player.z_m;
        let range=dx.hypot(dz).max(f32::EPSILON);let aim=(dx/range,dz/range);
        let wants_pulse=proofs.get(&target.entity_id).is_some_and(|p|p.primary&&!p.pulse);
        let pulse_ready=view.server_tick>=pulse_after&&view.player.current_energy>=12;
        let mut movement=if view.server_tick<hold_until {(0.0,0.0)}else if view.server_tick<attack_after {
            if range<2.7 {(-aim.0,-aim.1)}else{(0.0,0.0)}
        }else if range>1.6 {aim}else{(0.0,0.0)};
        let mut avoiding=false;
        for(warning,observed)in warnings.values(){
            if view.server_tick<observed+REACTION_TICKS{continue;}
            let d=(warning.direction_rad.sin(),warning.direction_rad.cos());
            let x=player.x_m-warning.position_m.x_m;let z=player.z_m-warning.position_m.z_m;
            let along=x*d.0+z*d.1;let lateral=-d.1*x+d.0*z;let clearance=warning.radius_m+0.65;
            let next_lateral=lateral+(-d.1*movement.0+d.0*movement.1)*0.8;
            // The public lunge is a swept body: include both rounded end caps.
            if along>=-clearance&&along<=warning.range_m.unwrap()+clearance&&(lateral.abs()<clearance||next_lateral.abs()<clearance){
                if lateral.abs()<clearance+0.3 {let sign=if lateral>=0.0{1.0}else{-1.0};movement=(-d.1*sign,d.0*sign);
                    if player.x_m+movement.0<1.1||player.x_m+movement.0>22.9||player.z_m+movement.1<1.1||player.z_m+movement.1>14.9{movement=(-movement.0,-movement.1);}
                }else{movement=(0.0,0.0);}
                avoiding=true;break;
            }
        }
        if avoiding{evades+=1;}
        if(player.x_m<1.1&&movement.0<0.0)||(player.x_m>22.9&&movement.0>0.0){movement=(0.0,if player.z_m<8.0{1.0}else{-1.0});}
        if(player.z_m<1.1&&movement.1<0.0)||(player.z_m>14.9&&movement.1>0.0){movement=(if player.x_m<12.0{1.0}else{-1.0},0.0);}
        let pulse_geometry=target.members.as_ref().is_some_and(|members|members.iter().filter(|m|m.active&&distance(m.position_m,player)<2.3).count()>=2);
        let attack=!avoiding&&view.player.action_state=="idle"&&pending.is_none()&&range<=1.68&&view.server_tick>=attack_after&&(!wants_pulse||(pulse_ready&&pulse_geometry));
        view=input(runtime,&view,movement,aim);
        if attack {
            let kind=if wants_pulse{ActionKind::Pulse}else{ActionKind::PrimaryAttack};
            view=runtime.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:view.world_epoch,
                request_id:NEXT_COMBAT_REQUEST.fetch_add(1,Ordering::Relaxed),client_time_ms:NEXT_TIME.fetch_add(17,Ordering::Relaxed),kind:kind.clone()}).unwrap();
            // A fixed input pacing policy, gated above by the public idle state.
            // The accepted-start and impact cues below must prove every request.
            attacks+=1;hold_until=view.server_tick+if wants_pulse{16}else{8};attack_after=view.server_tick+if wants_pulse{72}else{36};
            if wants_pulse{pulse_after=view.server_tick+144;}
            pending=Some(Pending{kind,until:view.server_tick+if wants_pulse{48}else{26},started:false,impact:false,members:BTreeMap::new()});
        }
    }
    panic!("genuine Swarm combat exceeded 9000 public inputs in {scene}; earned save {}",save_dir.display());
}
