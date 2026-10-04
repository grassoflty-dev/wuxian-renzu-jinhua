//! Genuine native MH ordinary encounters. Decisions use only public positions,
//! resources, ordinary cues and the same fixed 150ms warning response policy.
use super::*;
use std::collections::BTreeMap;
use std::path::Path;
use wuxian_horror_ch1::{formal_runtime::{ActionCommandRequest,ActionKind},world_v3::{ActorView,CombatOutcome,PresentationEvent,Vec3,WorldView}};
const WRAITH:&str="enemy.mist_harbor.signal_wraith";
const TIDE:&str="enemy.mist_harbor.tidebound";
fn distance(a:Vec3,b:Vec3)->f32{(a.x_m-b.x_m).hypot(a.z_m-b.z_m)}
fn input(r:&FormalRuntime,v:&WorldView,m:(f32,f32),aim:(f32,f32))->WorldView{
    r.submit_input(InputSample::new(v.world_epoch,v.ack_seq+1,NEXT_TIME.fetch_add(17,Ordering::Relaxed),m.0,m.1).unwrap().with_aim(aim.0,aim.1).unwrap(),vec![]).unwrap()
}
fn perceived(a:&ActorView)->Vec3{
    if a.entity_type==WRAITH{
        let p=a.signal_perception.as_ref().expect("native Wraith has a public perception record");
        assert_eq!(p.positions_m.len(),if p.precise{1}else{2});
        // Aim between the two unmarked visible alternatives. Never identify the
        // true point through a saved position, seed or hidden controller phase.
        p.positions_m.iter().fold(Vec3::zero(),|s,p|Vec3{x_m:s.x_m+p.x_m/p_count(a),y_m:s.y_m+p.y_m/p_count(a),z_m:s.z_m+p.z_m/p_count(a)})
    }else{a.transform.position_m}
}
fn p_count(a:&ActorView)->f32{a.signal_perception.as_ref().unwrap().positions_m.len() as f32}
#[derive(Default)]struct Proof{warnings:usize,ambiguous:usize,precise:usize,blink:usize,deaths:usize,wet:usize,dry:usize,kinds:BTreeSet<String>,flight:usize}
// A public authoritative contact identifies its recipient even if that actor
// moved/blinked after impact or overlaps another projected candidate. No saved
// position, health, controller phase or guessed geometry selects the checkpoint.
fn damaged_special(v:&WorldView,e:&PresentationEvent,request:Option<u64>,target:Option<&str>)->Option<String>{
    if e.world_epoch!=v.world_epoch||e.server_tick>v.server_tick
        ||!matches!(e.kind.as_str(),"Hit"|"PierceHit"|"PulseHit"){return None;}
    let f=e.combat_feedback.as_ref()?;
    if f.outcome!=CombatOutcome::EnemyHit||f.source_id.as_deref()!=Some("player")
        ||f.world_id!=v.world_id||f.scene_id!=v.scene_id||request.is_none()
        ||f.request_id!=request||f.target_id.as_deref()!=target{return None;}
    let id=f.target_id.as_deref()?;
    let mut matches=v.actors.iter().filter(|a|a.entity_id==id);
    let actor=matches.next()?;
    (matches.next().is_none()&&actor.active&&actor.actor_kind=="enemy"
        &&matches!(actor.entity_type.as_str(),WRAITH|TIDE)).then(||id.to_owned())
}
fn preserve(r:&FormalRuntime,dir:&Path,label:&str,damaged_target:Option<&str>)->WorldView{
    let held=r.pause().unwrap();r.save().unwrap();let saved=save_v6::read_save(dir).unwrap();
    if label=="partial" {
        let catalog:serde_json::Value=serde_json::from_str(include_str!("../../data/actor_profiles_v1.json")).unwrap();
        let id=damaged_target.expect("partial checkpoint requires a public damaged recipient");
        let actor=saved.save.generic_actors.iter().find(|a|a.entity_id==id).expect("public recipient is saved");
        let max_hp=catalog["profiles"].as_array().unwrap().iter().find(|p|p["entityType"]==actor.entity_type).unwrap()["maxHp"].as_u64().unwrap() as u32;
        assert!(matches!(actor.entity_type.as_str(),WRAITH|TIDE)&&actor.hp>0&&actor.hp<max_hp,
            "the exact public-hit recipient {id} must be a genuinely damaged living special enemy");
        println!("Genuine MH partial recipient: {id} at {}/{max_hp} HP",actor.hp);
    }
    let bytes=fs::read(dir.join("formal-save-v6.json")).unwrap();let prepared=r.continue_saved().unwrap();
    assert_eq!(fs::read(dir.join("formal-save-v6.json")).unwrap(),bytes,"Continue must leave the original bytes unchanged before any later save");
    assert_eq!(r.resume().unwrap_err(),"E_SCENE_ENTRY_NOT_READY");assert_eq!(r.snapshot().unwrap(),prepared);
    assert_eq!(prepared.player.current_hp,held.player.current_hp);assert_eq!(prepared.player.current_energy,held.player.current_energy);assert_eq!(prepared.player.transform.position_m,held.player.transform.position_m);
    assert_eq!(prepared.capabilities,held.capabilities);assert_eq!(prepared.progression,held.progression);assert_eq!(prepared.server_tick,held.server_tick);
    let ready=r.scene_ready(prepared.entry_token.as_ref().unwrap(),true).unwrap();r.save().unwrap();let restored=save_v6::read_save(dir).unwrap();
    assert_eq!(restored.save.generic_actors,saved.save.generic_actors,"{label} exact enemy state");assert_eq!(restored.world_persistent_v1,saved.world_persistent_v1);assert_eq!(restored.save.progression,saved.save.progression);
    let mut expected_capabilities=serde_json::to_value(&saved.save.capabilities).unwrap();
    for key in ["currentRevision","lastTickRevision"]{
        if let Some(revision)=expected_capabilities.get_mut(key).filter(|v|!v.is_null()){
            revision["worldEpoch"]=serde_json::json!(restored.save.revision.world_epoch);
        }
    }
    assert_eq!(serde_json::to_value(&restored.save.capabilities).unwrap(),expected_capabilities,
        "Continue may rebase capability epoch but no other capability state");
    assert_eq!(restored.save.inventory,saved.save.inventory);
    assert_eq!(serde_json::to_value(&restored.save.player).unwrap(),serde_json::to_value(&saved.save.player).unwrap());
    assert!(!r.presentation_events_since(ready.world_epoch,0).unwrap().iter().any(|e|e.kind=="EnemyDeath"),"Continue does not replay a defeat");
    println!("Genuine MH {label} Save/Continue: {} at {} HP",ready.scene_id,ready.player.current_hp);r.resume().unwrap()
}
pub(super) fn clear_scene(r:&FormalRuntime,mut v:WorldView,dir:&Path)->WorldView{
    if !matches!(v.scene_id.as_str(),"mh_fog_pier"|"mh_tidal_warehouse"|"mh_signal_yard"|"mh_drowned_quay"|"mh_breakwater"|"mh_pump_station"|"mh_resonance_tower")||!v.actors.iter().any(|a|a.active&&a.actor_kind=="enemy"){return v;}
    assert_eq!(v.capabilities.acoustic_mapping_authorized,Some(false),"ordinary counterplay must not require mapping");
    let drained=v.mist_harbor_pump.as_ref().is_some_and(|p|p.state==wuxian_horror_ch1::world_v3::MistHarborPumpState::Drained);
    let scene=v.scene_id.clone();let initial_hp=v.player.current_hp;let mut proofs:BTreeMap<String,Proof>=v.actors.iter().filter(|a|a.active&&matches!(a.entity_type.as_str(),WRAITH|TIDE)).map(|a|(a.entity_id.clone(),Proof::default())).collect();
    let wraith_ids:BTreeSet<_>=v.actors.iter().filter(|a|a.entity_type==WRAITH).map(|a|a.entity_id.clone()).collect();
    let tide_ids:BTreeSet<_>=v.actors.iter().filter(|a|a.entity_type==TIDE).map(|a|a.entity_id.clone()).collect();
    let mut cursor=0;let mut deferred=vec![];let mut warnings=BTreeMap::<String,(PresentationEvent,u64)>::new();let mut attacks=0;let mut evades=0;let mut enemy_damage=0;let mut hazard_damage=0;
    let mut last_action_target:Option<String>=None;let mut last_action_request=None;
    let mut next_attack=0;let mut hold=0;let mut pierce_after=0;let mut partial=false;let mut special_hit:Option<String>=None;let mut accepted_hits=0;let mut wet_seen=false;let mut dry_seen=false;
    for step in 0..18000{
        assert!(v.player.current_hp>0,"genuine MH death in {scene} at input {step}, entered with {initial_hp}HP");
        for a in v.actors.iter().filter(|a|a.active&&a.entity_type==WRAITH){let p=a.signal_perception.as_ref().unwrap();let proof=proofs.get_mut(&a.entity_id).unwrap();if p.precise{proof.precise+=1;}else{proof.ambiguous+=1;}}
        for e in r.presentation_events_since(v.world_epoch,cursor).unwrap(){cursor=cursor.max(e.event_id);deferred.push(e);}
        for e in std::mem::take(&mut deferred){if e.server_tick>v.server_tick{deferred.push(e);continue;}
            if e.kind=="Damaged"{enemy_damage+=1;}if e.kind=="EnvironmentHazardDamage"{hazard_damage+=1;}
            if matches!(e.kind.as_str(),"Hit"|"PierceHit"|"PulseHit"){
                accepted_hits+=1;
                if let Some(id)=damaged_special(&v,&e,last_action_request,last_action_target.as_deref()){special_hit=Some(id);}
            }
            if let Some(id)=&e.actor_id{
                if let Some(p)=proofs.get_mut(id){
                    if e.kind=="EnemyAttackTelegraph"{p.warnings+=1;let kind=e.attack_kind.as_deref().unwrap();assert!(matches!(kind,"signal_shot"|"tide_swing"|"tide_charge"));p.kinds.insert(kind.to_owned());
                        if id=="mh_drowned_quay_tidebound_01"{let wet=match kind{"tide_swing"=>e.range_m.unwrap()>2.,"tide_charge"=>e.range_m.unwrap()>3.,_=>false};if wet{p.wet+=1;wet_seen=true;}else{p.dry+=1;dry_seen=true;}}
                    }
                    if e.kind=="SignalShotMotion"{p.flight+=1;}
                    if e.kind=="SignalBlink"{p.blink+=1;}if e.kind=="EnemyDeath"{p.deaths+=1;assert_eq!(p.deaths,1,"duplicate public death {id}");}
                }
                if e.kind=="EnemyAttackTelegraph"{assert!(e.duration_ms.is_some_and(|n|n>0));assert!(e.range_m.is_some_and(|n|n>0.));warnings.insert(id.clone(),(e.clone(),v.server_tick));}
                else if matches!(e.kind.as_str(),"EnemyAttackImpact"|"EnemyStagger"|"EnemyDeath"){warnings.remove(id);}
            }
        }
        if !partial&&v.player.action_state=="idle"&&special_hit.as_ref().is_some_and(|id|v.actors.iter().any(|a|a.active&&a.entity_id==*id)){
            v=preserve(r,dir,"partial",special_hit.as_deref());cursor=0;deferred.clear();warnings.clear();partial=true;next_attack=v.server_tick+36;continue;
        }
        warnings.retain(|id,_|v.actors.iter().any(|a|a.entity_id==*id&&a.active));
        let player=v.player.transform.position_m;
        let target=v.actors.iter().filter(|a|a.active&&a.actor_kind=="enemy").min_by(|a,b|distance(perceived(a),player).total_cmp(&distance(perceived(b),player)));
        let Some(target)=target else{
            assert!(attacks>0&&accepted_hits>0);if !proofs.is_empty(){assert!(partial,"the genuine partial special-enemy state must be saved and restored");}for(id,p)in &proofs{assert_eq!(p.deaths,1,"{id} real defeat");assert!(p.warnings>0,"{id} real warning");}
            if scene=="mh_tidal_warehouse"{assert!(proofs.values().any(|p|p.blink>0),"real close-range recovery must show a public blink");}
            if !wraith_ids.is_empty(){assert!(proofs.iter().filter(|(id,_)|wraith_ids.contains(*id)).any(|(_,p)|p.ambiguous>0&&p.precise>0),"observe both public perception modes");}
            if !tide_ids.is_empty(){assert!(evades>0);
                let kinds:BTreeSet<_>=proofs.iter().filter(|(id,_)|tide_ids.contains(*id)).flat_map(|(_,p)|p.kinds.iter().cloned()).collect();
                assert_eq!(kinds,["tide_charge".to_owned(),"tide_swing".to_owned()].into_iter().collect(),"actual sector and corridor attacks must both occur");
            }
            if !wraith_ids.is_empty(){assert!(proofs.iter().filter(|(id,_)|wraith_ids.contains(*id)).any(|(_,p)|p.flight>0),"real SignalShot flight, not only cancelled casts");}
            if scene=="mh_drowned_quay"{
                if drained{assert!(dry_seen&&!wet_seen,"after genuine drainage new Quay attacks must use dry geometry");}
                else{assert!(wet_seen&&dry_seen,"real shoreline encounter must commit both wet and dry public geometry");}
            }
            println!("Genuine MH {scene}: {step} inputs,{attacks} actions,{accepted_hits} impacts,{evades} cue-aware movements(150ms),{enemy_damage} enemy/{hazard_damage} hazard damage cues, {}→{} HP; {} Wraith/{} Tidebound deaths, {} blinks",initial_hp,v.player.current_hp,wraith_ids.len(),tide_ids.len(),proofs.values().map(|p|p.blink).sum::<usize>());
            assert!(v.player.current_energy>=64,"ordinary campaign combat preserves the later boss reserve");
            return preserve(r,dir,"defeated",None);
        };
        let target_id=target.entity_id.clone();let point=perceived(target);let dx=point.x_m-player.x_m;let dz=point.z_m-player.z_m;let range=dx.hypot(dz).max(f32::EPSILON);let aim=(dx/range,dz/range);
        let wraith=target.entity_type==WRAITH;let exact=target.signal_perception.as_ref().is_none_or(|p|p.precise);
        let tide=target.entity_type==TIDE;
        let needs_swing=tide&&!proofs[&target_id].kinds.contains("tide_swing");
        let needs_charge=tide&&!proofs[&target_id].kinds.contains("tide_charge");
        // Deliberately exercise both ordinary attacks through visible range and
        // warnings, including the shorter dry Swing after actual drainage.
        let desired=if needs_swing{1.15}else if wraith&&!exact{1.0}else{1.5};
        let mut m=if v.server_tick<hold{(0.,0.)}else if v.server_tick<next_attack{if range<2.8{(-aim.0,-aim.1)}else{(0.,0.)}}else if range>desired{aim}else{(0.,0.)};
        let mut avoiding=false;
        for(w,seen)in warnings.values(){if v.server_tick<seen+9{continue;}
            let dx=player.x_m-w.position_m.x_m;let dz=player.z_m-w.position_m.z_m;let radius=dx.hypot(dz).max(f32::EPSILON);let direction=(w.direction_rad.sin(),w.direction_rad.cos());let along=dx*direction.0+dz*direction.1;let lateral=-direction.1*dx+direction.0*dz;
            if w.attack_kind.as_deref()==Some("tide_swing"){
                let half=w.half_angle_rad.expect("public swing sector");if radius<w.range_m.unwrap()+0.4&&along/radius>half.cos()-0.15{m=(dx/radius,dz/radius);avoiding=true;break;}
            }else{let clear=w.radius_m+0.65;let predicted=lateral+(-direction.1*m.0+direction.0*m.1)*0.8;
                if along>=-clear&&along<=w.range_m.unwrap()+clear&&(lateral.abs()<clear||predicted.abs()<clear){let sign=if lateral>=0.{1.}else{-1.};m=(-direction.1*sign,direction.0*sign);avoiding=true;break;}}
        }
        if avoiding{evades+=1;}
        if(player.x_m<1.1&&m.0<0.)||(player.x_m>22.9&&m.0>0.){m=(0.,if player.z_m<8.{1.}else{-1.});}
        if(player.z_m<1.1&&m.1<0.)||(player.z_m>14.9&&m.1>0.){m=(if player.x_m<12.{1.}else{-1.},0.);}
        // First Quay Tidebound is allowed to cross the shoreline visibly before
        // the final attacks. Geometry observations, never hidden terrain/HP, decide.
        let await_shore=target_id=="mh_drowned_quay_tidebound_01"&&!drained&&!(wet_seen&&dry_seen);
        if await_shore&&!avoiding&&wet_seen{m=if player.x_m>11.{(-1.,0.)}else{(0.,0.)};}
        // A non-staggering Primary can damage an ambiguous Wraith without a Hit
        // cue by design. The two Wraith-only scenes therefore earn their first
        // checkpoint with a real, nonlethal Pulse and its visible stagger/contact.
        // Reserve 64 for boss defense and 12 for the next Wraith-only checkpoint.
        let checkpoint_pulse=!partial&&wraith&&tide_ids.is_empty();
        let pulse_in_range=checkpoint_pulse&&target.signal_perception.as_ref().unwrap().positions_m.iter().all(|p|distance(*p,player)<2.1);
        let pierce=!checkpoint_pulse&&(wraith||tide)&&exact&&range>1.8&&range<4.2&&v.server_tick>=pierce_after&&v.player.current_energy>=94;
        let await_blink=scene=="mh_tidal_warehouse"&&wraith&&proofs[&target_id].blink==0;
        if tide&&!avoiding&&!await_shore&&!needs_swing&&needs_charge {
            m=if range<2.7{(-aim.0,-aim.1)}else if range>3.0{aim}else{(0.,0.)};
            if(player.x_m<1.1&&m.0<0.)||(player.x_m>22.9&&m.0>0.){m=(0.,if player.z_m<8.{1.}else{-1.});}
            if(player.z_m<1.1&&m.1<0.)||(player.z_m>14.9&&m.1>0.){m=(if player.x_m<12.{1.}else{-1.},0.);}
        }
        let attack=(partial||special_hit.is_none())&&!avoiding&&!await_shore&&!await_blink&&!needs_swing&&!needs_charge&&v.player.action_state=="idle"&&v.server_tick>=next_attack
            &&if checkpoint_pulse{pulse_in_range}else{range<desired+0.05||pierce};
        if step%900==0{println!("MH combat progress {scene}: input{step}, target{target_id}, range{range:.2}, {}HP",v.player.current_hp);}
        v=input(r,&v,m,aim);
        if attack{
            let kind=if checkpoint_pulse{assert!(v.player.current_energy>=76,"checkpoint Pulse must preserve the 64-energy boss reserve");ActionKind::Pulse}else if pierce{ActionKind::Pierce}else{ActionKind::PrimaryAttack};
            let energy_before=v.player.current_energy;
            let request_id=NEXT_COMBAT_REQUEST.fetch_add(1,Ordering::Relaxed);
            v=r.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:v.world_epoch,request_id,client_time_ms:NEXT_TIME.fetch_add(17,Ordering::Relaxed),kind}).unwrap();
            if checkpoint_pulse{
                assert_eq!(v.player.current_energy,energy_before-12,"real checkpoint Pulse spends exactly its authorized tuning cost");
                assert!(v.player.current_energy>=64,"real campaign keeps the boss reserve");
                println!("Genuine MH checkpoint Pulse {scene}: {energy_before}→{} energy",v.player.current_energy);
            }
            last_action_target=Some(target_id);last_action_request=Some(request_id);attacks+=1;
            hold=v.server_tick+if pierce||checkpoint_pulse{24}else{8};
            next_attack=v.server_tick+if checkpoint_pulse{150}else if pierce{60}else{36};
            if pierce{pierce_after=v.server_tick+170;}
        }
    }
    panic!("genuine MH {scene} exceeded18000 public inputs")
}

#[cfg(test)]
mod checkpoint_regressions {
    use super::*;
    use wuxian_horror_ch1::world_v3::ActorRuntime;

    fn fixture() -> (WorldView, PresentationEvent) {
        let runtime = FormalRuntime::new_with_save_dir(save_root()).unwrap();
        let mut view = runtime.pause().unwrap();
        view.world_id = "mist_harbor".into();
        view.scene_id = "mh_tidal_warehouse".into();
        view.server_tick = 120;
        view.actors = vec![ActorRuntime::spawn("special", WRAITH, Vec3::zero()).unwrap().view()];
        // The native Wraith projection replaces its legacy render-type alias.
        view.actors[0].entity_type = WRAITH.into();
        let event = serde_json::from_value(serde_json::json!({
            "protocolVersion":2,"eventId":7,"worldEpoch":view.world_epoch,"serverTick":100,
            "kind":"PulseHit","positionM":{"xM":0.,"yM":0.,"zM":0.},
            "directionRad":0.,"radiusM":1.,"intensity":1.,
            "combatFeedback":{"worldId":view.world_id,"sceneId":view.scene_id,"outcome":"enemy_hit",
                "targetId":"special","sourceId":"player","requestId":42}
        })).unwrap();
        (view, event)
    }

    #[test]
    fn authoritative_contact_selects_moved_recipient_despite_overlapping_candidates() {
        let (mut view, event) = fixture();
        view.actors[0].transform.position_m.x_m = 9.;
        let mut other = view.actors[0].clone();
        other.entity_id = "other".into();
        view.actors.push(other);
        assert_eq!(damaged_special(&view, &event, Some(42), Some("special")), Some("special".into()));
        view.actors[0].entity_type = TIDE.into();
        assert_eq!(damaged_special(&view, &event, Some(42), Some("special")), Some("special".into()));
    }

    #[test]
    fn checkpoint_rejects_stale_unrelated_missing_and_nondamaging_receipts() {
        let (view, event) = fixture();
        for fault in ["epoch", "future", "world", "scene", "request", "missing-request", "target", "source", "outcome", "missing-feedback", "stagger", "ordinary", "dead", "duplicate"] {
            let mut view = view.clone();
            let mut event = event.clone();
            match fault {
                "epoch" => event.world_epoch += 1,
                "future" => event.server_tick = view.server_tick + 1,
                "world" => event.combat_feedback.as_mut().unwrap().world_id = "other".into(),
                "scene" => event.combat_feedback.as_mut().unwrap().scene_id = "other".into(),
                "request" => event.combat_feedback.as_mut().unwrap().request_id = Some(41),
                "missing-request" => event.combat_feedback.as_mut().unwrap().request_id = None,
                "target" => event.combat_feedback.as_mut().unwrap().target_id = Some("other".into()),
                "source" => event.combat_feedback.as_mut().unwrap().source_id = Some("enemy".into()),
                "outcome" => event.combat_feedback.as_mut().unwrap().outcome = CombatOutcome::Rejected,
                "missing-feedback" => event.combat_feedback = None,
                "stagger" => event.kind = "EnemyStagger".into(),
                "ordinary" => view.actors[0].entity_type = "enemy.mist_harbor.drowned".into(),
                "dead" => { let mut untouched = view.actors[0].clone(); untouched.entity_id = "untouched".into(); view.actors.push(untouched); view.actors[0].active = false; }
                "duplicate" => view.actors.push(view.actors[0].clone()),
                _ => unreachable!(),
            }
            assert_eq!(damaged_special(&view, &event, Some(42), Some("special")), None, "{fault}");
        }
        assert_eq!(damaged_special(&view, &event, None, Some("special")), None);
        assert_eq!(damaged_special(&view, &event, Some(42), None), None);
    }
}
