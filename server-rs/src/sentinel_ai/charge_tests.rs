use super::*;
use crate::continuous_kcc::{Aabb,StaticKccWorld};
fn p(x:f32,z:f32)->Vec3 {Vec3::new(x,0.,z).unwrap()}
fn world()->StaticKccWorld {StaticKccWorld::new(Aabb::new(-20.,20.,-20.,20.).unwrap(),vec![])}
fn sentinel()->Sentinel {let mut s=Sentinel::new("test",p(0.,0.),100);s.enable_charge();s}
fn start(s:&mut Sentinel,k:&StaticKccWorld)->Vec3 {let target=p(3.,0.);let(e,c)=tick_sentinel(s,target,0.017,k);assert!(c.is_empty());assert_eq!(e.len(),1);assert_eq!(s.state,SentinelState::ChargeWindup);target}
fn active(s:&mut Sentinel,k:&StaticKccWorld,target:Vec3) {for _ in 0..60 {if s.state==SentinelState::Charge {break;}tick_sentinel(s,target,0.017,k);}assert_eq!(s.state,SentinelState::Charge);assert_eq!(s.state_remaining_ms,600);assert!(s.validate());}
#[test] fn charge_commits_truthful_geometry_and_does_not_retarget_or_change_melee_cadence(){
 let k=world();let mut s=sentinel();s.attack_serial=2;let target=start(&mut s,&k);let view=s.encounter_view(60).unwrap();let w=view.warning.unwrap();assert_eq!((w.radius_m,w.range_m),(0.6,3.6));assert!((w.direction_rad-std::f32::consts::FRAC_PI_2).abs()<0.0001);assert_eq!(view.attack_serial,1);assert_eq!(s.attack_serial,2);
 active(&mut s,&k,p(-3.,0.));let mut hits=0;for _ in 0..40 {let before=s.position_m;let events=tick_sentinel(&mut s,target,0.017,&k).1;hits+=events.len();for event in events {assert_eq!(event,CombatEvent::PlayerDamaged{source_id:"test".into(),damage:16,contact:Some(crate::continuous_combat::CombatContact::new(before,target,None))});}assert!(s.validate());}assert_eq!(hits,1);assert_eq!(s.attack_serial,2);assert!(s.position_m.x_m>3.5);assert_eq!(s.charge_controller.as_ref().unwrap().charge_serial,1);
}
#[test] fn charge_public_corridor_has_exact_total_center_radius_and_fair_sidestep(){
 for(z,expected)in [(0.59,1),(0.61,0)] {let k=world();let mut s=sentinel();start(&mut s,&k);active(&mut s,&k,p(3.,z));let mut hits=0;for _ in 0..40{hits+=tick_sentinel(&mut s,p(3.,z),0.017,&k).1.len();}assert_eq!(hits,expected);}
 let k=world();let mut s=sentinel();start(&mut s,&k);let mut player=p(3.,0.);for n in 0..100{if n>=9{player.z_m=(player.z_m+4.0*0.017).min(1.);}let(_,events)=tick_sentinel(&mut s,player,0.017,&k);assert!(events.is_empty());}
}
#[test] fn charge_stops_at_thin_wall_and_cannot_contact_through_side_wall(){
 let mut k=world();k.walls.push(Aabb::new(1.,1.001,-1.,1.).unwrap());let mut s=sentinel();tick_sentinel(&mut s,p(3.,0.),0.017,&k);assert_ne!(s.state,SentinelState::ChargeWindup); // LOS denial
 let k=world();let mut s=sentinel();start(&mut s,&k);active(&mut s,&k,p(3.,0.));let mut blocked=k.clone();blocked.walls.push(Aabb::new(1.,1.001,-1.,1.).unwrap());let mut hits=0;for _ in 0..40 {hits+=tick_sentinel(&mut s,p(3.,0.),0.017,&blocked).1.len();}assert_eq!(hits,0);assert!(s.position_m.x_m<1.);
}
#[test] fn stagger_and_damage_interrupt_charge_for_the_entire_actual_window(){
 let k=world();for active_phase in [false,true] {let mut s=sentinel();start(&mut s,&k);if active_phase{active(&mut s,&k,p(3.,0.));}
 let before=s.state_remaining_ms;s.apply_stagger(200);assert_eq!(s.state,SentinelState::Stagger);assert_eq!(s.state_remaining_ms,before.max(200));assert!(s.charge_controller.as_ref().unwrap().attack.is_none());assert!(s.encounter_view(60).unwrap().warning.is_none());s.state_remaining_ms=119;tick_sentinel(&mut s,p(3.,0.),0.017,&k);assert_eq!(s.state,SentinelState::Stagger);
 let mut s=sentinel();start(&mut s,&k);s.take_damage_with_stagger(32,250);assert_eq!((s.hp,s.state,s.state_remaining_ms),(68,SentinelState::Stagger,250));assert!(s.charge_controller.as_ref().unwrap().attack.is_none());
 }
}
#[test] fn charge_zero_elapsed_and_consumed_phase_round_trip_keep_geometry(){
 let k=world();let mut s=sentinel();start(&mut s,&k);active(&mut s,&k,p(3.,0.));let c=s.charge_controller.clone().unwrap();let decoded:ChargeController=serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();assert_eq!(decoded,c);assert!((s.encounter_view(60).unwrap().warning.unwrap().direction_rad-std::f32::consts::FRAC_PI_2).abs()<0.0001);
 for _ in 0..30{tick_sentinel(&mut s,p(3.,0.),0.017,&k);}assert!(s.charge_controller.as_ref().unwrap().attack.as_ref().unwrap().hit_resolved);let mut restored=s.clone();for _ in 0..10{assert!(tick_sentinel(&mut restored,p(3.,0.),0.017,&k).1.is_empty());}
}
#[test] fn malformed_charge_cannot_tick_take_damage_or_stagger(){
 let k=world();let mut original=sentinel();start(&mut original,&k);for fault in 0..8 {let mut s=original.clone();match fault {0=>s.charge_controller=None,1=>s.charge_controller.as_mut().unwrap().schema_version=2,2=>s.charge_controller.as_mut().unwrap().attack=None,3=>s.charge_controller.as_mut().unwrap().attack.as_mut().unwrap().direction_m.x_m=2.,4=>s.charge_controller.as_mut().unwrap().attack.as_mut().unwrap().travel_m=4.,5=>s.charge_controller.as_mut().unwrap().attack.as_mut().unwrap().elapsed_ms=1,6=>s.charge_controller.as_mut().unwrap().attack.as_mut().unwrap().hit_resolved=true,_=>s.position_m.x_m+=1.}let before=s.clone();assert!(!s.validate());assert!(tick_sentinel(&mut s,p(3.,0.),0.017,&k).0.is_empty());s.take_damage(5);s.apply_stagger(200);assert_eq!(s,before);}
}
#[test] fn legacy_controller_preserves_original_light_heavy_and_stagger(){
 let mut s=Sentinel::new("legacy",p(0.,0.),100);let k=world();s.state=SentinelState::Attack;s.attack_serial=2;let(_,c)=tick_sentinel(&mut s,p(1.,0.),0.017,&k);assert_eq!(c,vec![CombatEvent::PlayerDamaged{source_id:"legacy".into(),damage:8,contact:Some(crate::continuous_combat::CombatContact::new(p(0.,0.),p(1.,0.),None))}]);assert_eq!((s.state,s.state_remaining_ms,s.attack_serial),(SentinelState::HeavyAttack,400,3));s.state_remaining_ms=0;let(_,c)=tick_sentinel(&mut s,p(1.,0.),0.017,&k);assert_eq!(c,vec![CombatEvent::PlayerDamaged{source_id:"legacy".into(),damage:16,contact:Some(crate::continuous_combat::CombatContact::new(p(0.,0.),p(1.,0.),None))}]);assert_eq!((s.state,s.state_remaining_ms,s.attack_serial),(SentinelState::Recover,500,4));s.apply_stagger(200);assert_eq!((s.state,s.state_remaining_ms),(SentinelState::Hit,500));
}
#[test] fn charge_height_death_and_nonfinite_inputs_are_safe(){
 let k=world();let mut s=sentinel();let old=s.clone();assert!(tick_sentinel(&mut s,Vec3{x_m:f32::NAN,..p(3.,0.)},0.017,&k).0.is_empty());assert_eq!(s,old);tick_sentinel(&mut s,Vec3{y_m:2.,..p(3.,0.)},0.017,&k);assert_eq!(s,old);start(&mut s,&k);s.take_damage(100);assert_eq!((s.hp,s.state,s.active),(0,SentinelState::Death,false));assert!(s.charge_controller.as_ref().unwrap().attack.is_none());let old=s.clone();tick_sentinel(&mut s,p(3.,0.),0.017,&k);assert_eq!(s,old);
}

fn owner_world()->crate::world_v3::WorldStateV3 {let mut w=crate::world_v3::WorldStateV3::new("grey_hive",1,p(3.,0.)).unwrap();w.sentinels.push(sentinel());w}
fn owner_step(w:&mut crate::world_v3::WorldStateV3,k:&StaticKccWorld,seq:u64,intents:&[crate::continuous_combat::CombatIntent])->crate::world_v3::CwStepOutput{
 let input=crate::continuous_input::InputSample::new(1,seq,seq*17,0.,0.).unwrap().with_aim(-1.,0.).unwrap();crate::world_v3::step_world(w,k,crate::world_v3::CwStepInput{sample:&input,dt_s:1./60.,combat:intents}).unwrap()
}
#[test] fn charge_shared_owner_guard_and_pierce_have_real_damage_and_interrupt_value(){
 use crate::continuous_combat::{CombatIntent,CombatEvent};
 for guard in [true,false] {let k=world();let mut w=owner_world();let hp=w.player_hp;let mut sent=false;let mut resolved=false;
 for seq in 1..150 {let trigger=!sent&&w.sentinels[0].state==SentinelState::ChargeWindup&&w.sentinels[0].state_remaining_ms<=if guard{100}else{700};
 let commands=if trigger{sent=true;vec![if guard{CombatIntent::GuardStart{request_id:91}}else{CombatIntent::Pierce{request_id:92}}]}else{vec![]};let out=owner_step(&mut w,&k,seq,&commands);
 if out.combat.iter().any(|e|if guard{matches!(e,CombatEvent::GuardImpact{..})}else{matches!(e,CombatEvent::ActionImpact{..})}) {resolved=true;break;}}
 assert!(sent&&resolved,"guard={guard}");assert_eq!(w.player_hp,hp);assert_eq!(w.sentinels[0].state,SentinelState::Stagger);assert!(w.sentinels[0].charge_controller.as_ref().unwrap().attack.is_none());assert_eq!(w.sentinels[0].hp,if guard{100}else{68});assert_eq!(w.player_energy,if guard{85}else{82});
 for seq in 160..165{let out=owner_step(&mut w,&k,seq,&[]);assert!(!out.combat.iter().any(|e|matches!(e,CombatEvent::PlayerDamaged{..})));}assert_eq!(w.player_hp,hp);
 }
}
#[test] fn invalid_charge_owner_rolls_back_player_clock_actions_and_combat(){
 let k=world();let mut w=owner_world();owner_step(&mut w,&k,1,&[]);w.sentinels[0].charge_controller.as_mut().unwrap().attack.as_mut().unwrap().travel_m=100.;let before=w.clone();let sample=crate::continuous_input::InputSample::new(1,2,34,1.,0.).unwrap();let result=crate::world_v3::step_world(&mut w,&k,crate::world_v3::CwStepInput{sample:&sample,dt_s:1./60.,combat:&[crate::continuous_combat::CombatIntent::Pierce{request_id:1}]});assert!(result.is_err());assert_eq!(format!("{w:?}"),format!("{before:?}"));
}
#[test] fn current_phase_limits_and_zero_elapsed_consumed_flags_fail_closed(){
 let k=world();for state in [SentinelState::Chase,SentinelState::Attack,SentinelState::HeavyAttack,SentinelState::Recover,SentinelState::Hit,SentinelState::Stagger,SentinelState::Death]{let mut s=sentinel();s.state=state;if state==SentinelState::Death{s.hp=0;s.active=false;}s.state_remaining_ms=6000;assert!(!s.validate());}
 let mut s=sentinel();start(&mut s,&k);active(&mut s,&k,p(3.,0.));s.charge_controller.as_mut().unwrap().attack.as_mut().unwrap().hit_resolved=true;assert!(!s.validate());
}
#[test] fn newly_closed_wall_stops_charge_without_freezing_the_shared_owner(){
 let mut k=world();let mut w=owner_world();owner_step(&mut w,&k,1,&[]);let mut seq=2;while w.sentinels[0].state!=SentinelState::Charge {owner_step(&mut w,&k,seq,&[]);seq+=1;}
 k.walls.push(Aabb::new(1.,1.001,-1.,1.).unwrap());let hp=w.player_hp;for _ in 0..20 {owner_step(&mut w,&k,seq,&[]);seq+=1;if w.sentinels[0].state==SentinelState::Recover{break;}}
 assert_eq!(w.sentinels[0].state,SentinelState::Recover);assert!(w.sentinels[0].position_m.x_m<1.);assert_eq!(w.player_hp,hp);assert!(w.sentinels[0].charge_controller.as_ref().unwrap().attack.is_none());
}

#[test] fn sentinel_serial_exhaustion_never_commits_unsafe_metadata_or_repeat_damage(){
 const MAX:u64=9_007_199_254_740_991;let k=world();
 for phase in [SentinelState::Attack,SentinelState::HeavyAttack]{let mut s=sentinel();s.state=phase;s.state_remaining_ms=1;s.attack_serial=MAX;let before=s.clone();assert_eq!(tick_sentinel(&mut s,p(1.,0.),0.017,&k),(vec![],vec![]));assert_eq!(s,before);assert!(s.encounter_view(60).is_some());
 s.attack_serial=MAX-1;let (_,hit)=tick_sentinel(&mut s,p(1.,0.),0.017,&k);assert_eq!(hit.len(),1);assert_eq!(s.attack_serial,MAX);assert!(s.validate());}
 let mut s=sentinel();s.charge_controller.as_mut().unwrap().charge_serial=MAX;let before=s.clone();assert_eq!(tick_sentinel(&mut s,p(3.,0.),0.017,&k),(vec![],vec![]));assert_eq!(s,before);
 let mut s=sentinel();s.charge_controller.as_mut().unwrap().charge_serial=MAX-1;let target=start(&mut s,&k);assert_eq!(s.charge_controller.as_ref().unwrap().charge_serial,MAX);active(&mut s,&k,target);let mut hits=0;for _ in 0..40{hits+=tick_sentinel(&mut s,target,0.017,&k).1.len();}assert_eq!(hits,1);assert_eq!(s.state,SentinelState::Recover);assert!(s.validate());
 let mut s=sentinel();s.take_damage(1);assert_eq!(s.encounter_view(60).unwrap().remaining_ms,134);assert_eq!(s.state_remaining_ms,120);
}
