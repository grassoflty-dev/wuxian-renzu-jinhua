//! Explicit historical-save/phase fixtures; genuine campaign evidence is separate.
use super::*;
use crate::formal_runtime::entry_test_support::acknowledge_ready;
use crate::save_v6::SaveV6;
use crate::world_v3::{ActorAiState,ActorAttackKind,ActorRuntimeEvent};
fn fixture(scene:&str)->FormalRuntime {
    let root=std::env::temp_dir().join(format!("swarm-roster-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let r=FormalRuntime::new_with_save_dir(root).unwrap();r.pause().unwrap();r.stop_owner.store(true,Ordering::SeqCst);r.owner_handle.lock().unwrap().take().unwrap().join().unwrap();
    crate::production_scene_bootstrap::install(&r).unwrap();let registry=r.scene_registry.lock().unwrap().clone().unwrap();r.install_scene_registry(registry,scene).unwrap();r
}
fn capture(r:&FormalRuntime)->SaveV6{SaveV6::capture(&r.state.lock().unwrap(),r.scene_runtime.lock().unwrap().as_ref()).unwrap()}
fn actors(r:&FormalRuntime)->Vec<ActorRuntime>{r.state.lock().unwrap().world.generic_actors.clone()}
#[test]
fn four_authored_groups_have_exact_identities_clear_cluster_space_and_hp_free_member_views(){
    let mut count=0;
    for scene in SCENES{
        let r=fixture(scene);let save=capture(&r);assert_eq!(save.world_persistent_v1.grey_hive.swarm_actor_roster_version,1);
        assert_eq!(save.save.generic_actors,authored(&canonical(scene).unwrap()).unwrap());
        let view=r.snapshot().unwrap();let state=r.state.lock().unwrap();
        for actor in state.world.generic_actors.iter().filter(|a|a.entity_type==SWARM){
            count+=1;assert!(state.kcc.can_occupy(actor.home_m,0.95));let public=view.actors.iter().find(|a|a.entity_id==actor.entity_id).unwrap();let members=public.members.as_ref().unwrap();assert_eq!(members.len(),4);
            for(i,m)in members.iter().enumerate(){assert_eq!(m.member_id,format!("{}/member/{}",actor.entity_id,i+1));assert!(m.active);assert_eq!(m.radius_m,0.15);assert!(state.kcc.can_occupy(m.position_m,m.radius_m));}
            let raw=serde_json::to_value(public).unwrap();assert!(!raw.to_string().contains("Hp"));assert!(!raw.to_string().contains("ordinary"));
        }
    }assert_eq!(count,4);let other=fixture("gh_gate_b");assert_eq!(capture(&other).world_persistent_v1.grey_hive.swarm_actor_roster_version,0);
}
#[test]
fn old_exact_rows_and_empty_or_current_rosters_upgrade_only_on_paused_candidate(){
    for scene in SCENES{for mode in ["old","empty","current"]{
        let r=fixture(scene);let mut old=capture(&r);old.world_persistent_v1.grey_hive.swarm_actor_roster_version=0;old.world_persistent_v1.grey_hive.sentinel_first_kill=true;
        if mode=="old"{old.save.generic_actors.truncate(old_count(scene));for a in &mut old.save.generic_actors{a.hp-=7;a.state=ActorAiState::Chasing;a.position_m.x_m+=0.1;}}
        if mode=="empty"{old.save.generic_actors.clear();}
        crate::save_v6::write_save(&r.save_root,&old).unwrap();crate::save_slots::create_slot_v6(&r.save_root,"old","Old",&old).unwrap();
        let path=crate::save_v6::save_path(&r.save_root);let slot=r.save_root.join("slots/old/slot-v6.json");let bytes=std::fs::read(&path).unwrap();let slot_bytes=std::fs::read(&slot).unwrap();let mut epoch=r.snapshot().unwrap().world_epoch;
        for use_slot in [false,false,true]{let view=if use_slot{r.continue_slot("old").unwrap()}else{r.continue_saved().unwrap()};assert!(r.state.lock().unwrap().paused&&view.entry_token.is_some());assert!(view.world_epoch>epoch);epoch=view.world_epoch;
            let loaded=capture(&r);assert_eq!(loaded.world_persistent_v1.grey_hive.swarm_actor_roster_version,1);assert!(loaded.world_persistent_v1.grey_hive.sentinel_first_kill);
            assert_eq!(&loaded.save.generic_actors[..old.save.generic_actors.len()],&old.save.generic_actors);assert_eq!(loaded.save.progression,old.save.progression);
            acknowledge_ready(&r,view);assert_eq!(std::fs::read(&path).unwrap(),bytes);assert_eq!(std::fs::read(&slot).unwrap(),slot_bytes);
        }
    }}
}
#[test]
fn malformed_current_roster_or_member_state_rejects_write_and_continue_atomically(){
    for scene in SCENES{for fault in ["missing","empty","extra","duplicate","type","old-home","new-home","members","hp-sum","version"]{
        let r=fixture(scene);r.save().unwrap();r.save_slot("current","Current",true).unwrap();let path=crate::save_v6::save_path(&r.save_root);let slot_path=r.save_root.join("slots/current/slot-v6.json");let bytes=std::fs::read(&path).unwrap();let slot_bytes=std::fs::read(&slot_path).unwrap();let mut bad=capture(&r);let i=old_count(scene);
        match fault{"missing"=>{bad.save.generic_actors.pop();},"empty"=>bad.save.generic_actors.clear(),"extra"=>{let mut a=bad.save.generic_actors[i].clone();a.entity_id.push_str("_extra");bad.save.generic_actors.push(a);},"duplicate"=>bad.save.generic_actors[i+1]=bad.save.generic_actors[i].clone(),"type"=>{let a=&bad.save.generic_actors[i];bad.save.generic_actors[i]=ActorRuntime::spawn(&a.entity_id,"enemy.grey_hive.brute",a.home_m).unwrap();},"old-home"=>bad.save.generic_actors[0].home_m.x_m+=0.1,"new-home"=>bad.save.generic_actors[i].home_m.x_m+=0.1,"members"=>{bad.save.generic_actors[i].group.as_mut().unwrap().member_hp.pop();},"hp-sum"=>bad.save.generic_actors[i].hp-=1,"version"=>bad.world_persistent_v1.grey_hive.swarm_actor_roster_version=2,_=>unreachable!()}
        assert!(crate::save_v6::write_save(&r.save_root,&bad).is_err(),"{scene}:{fault}");assert!(crate::save_slots::overwrite_slot_v6(&r.save_root,"current","Bad",&bad).is_err());assert_eq!(std::fs::read(&path).unwrap(),bytes);assert_eq!(std::fs::read(&slot_path).unwrap(),slot_bytes);
        let raw=serde_json::to_vec(&bad).unwrap();std::fs::write(&path,&raw).unwrap();let mut slot:serde_json::Value=serde_json::from_slice(&slot_bytes).unwrap();slot["save"]=serde_json::to_value(&bad).unwrap();let slot_raw=serde_json::to_vec(&slot).unwrap();std::fs::write(&slot_path,&slot_raw).unwrap();let before=r.snapshot().unwrap();let before_actors=actors(&r);
        assert!(r.continue_saved().is_err());assert!(r.continue_slot("current").is_err());assert_eq!(r.snapshot().unwrap(),before);assert_eq!(actors(&r),before_actors);assert_eq!(std::fs::read(&path).unwrap(),raw);assert_eq!(std::fs::read(&slot_path).unwrap(),slot_raw);
    }}
}
#[test]
fn damaged_and_dead_members_survive_continue_without_resurrection_or_death_replay(){
    for scene in SCENES{for dead in [false,true]{let r=fixture(scene);let i=old_count(scene);r.state.lock().unwrap().world.generic_actors[i].take_member_hits(if dead{&[0,1,2,3]}else{&[0,2]},12,35).unwrap();let before=actors(&r);r.save().unwrap();let view=r.continue_saved().unwrap();
        assert!(view.entry_token.is_some());assert_eq!(actors(&r),before);let public=view.actors.iter().find(|a|a.entity_id==before[i].entity_id).unwrap();assert_eq!(public.members.as_ref().unwrap().iter().filter(|m|m.active).count(),if dead{0}else{2});
        assert!(!r.state.lock().unwrap().presentation_events.iter().any(|e|matches!(e.kind.as_str(),"EnemyDeath"|"EnemyMemberDisperse")));acknowledge_ready(&r,view);
    }}
    for scene in ["gh_entry_maintenance","gh_gate_b","gh_sentinel_arena"]{let r=fixture(scene);r.state.lock().unwrap().world_persistent_v1.grey_hive.swarm_actor_roster_version=1;r.save().unwrap();let old=capture(&r);acknowledge_ready(&r,r.continue_saved().unwrap());let new=capture(&r);assert_eq!(new.save.generic_actors,old.save.generic_actors);assert_eq!(new.save.actors,old.save.actors);assert_eq!(new.world_persistent_v1.grey_hive,old.world_persistent_v1.grey_hive);}
}
#[test]
fn saved_one_tick_lunge_and_zero_elapsed_motion_republish_exact_geometry_while_held(){
    for scene in SCENES{for active in [false,true]{let r=fixture(scene);let i=old_count(scene);let expected;
        {let mut state=r.state.lock().unwrap();let kcc=state.kcc.clone();let a=&mut state.world.generic_actors[i];let target=Vec3::new(a.position_m.x_m+2.0,a.position_m.y_m,a.position_m.z_m).unwrap();for _ in 0..100{if a.state==if active{ActorAiState::Active}else{ActorAiState::Windup}{break;}a.tick(target,&kcc,1.0/60.0,false);}assert_eq!(a.current_attack,Some(ActorAttackKind::Lunge));if !active{a.state_remaining_ms=1;}assert!(a.validate());expected=a.clone();}
        r.save().unwrap();let view=r.continue_saved().unwrap();assert!(r.state.lock().unwrap().paused);assert_eq!(actors(&r)[i],expected);
        let rows=r.presentation_events_since(view.world_epoch,0).unwrap();let cue=rows.iter().find(|e|e.actor_id.as_deref()==Some(&expected.entity_id)&&e.attack_kind.as_deref()==Some("lunge")).unwrap();assert_eq!(cue.kind,if active{"EnemyLungeMotion"}else{"EnemyAttackTelegraph"});assert_eq!(cue.direction_rad,std::f32::consts::FRAC_PI_2);assert_eq!(cue.radius_m,0.95);assert_eq!(cue.range_m,Some(2.0));assert!(cue.duration_ms.unwrap()>0);
        if active{assert_eq!(expected.ordinary.as_ref().unwrap().active.as_ref().unwrap().elapsed_ms,0);assert_eq!(cue.position_m,expected.position_m);}
        acknowledge_ready(&r,view);assert_eq!(actors(&r)[i],expected);
    }}
}
#[test]
fn member_cues_require_current_authored_actor_and_member_liveness(){
    let r=fixture("gh_lockdown");let scene=r.scene_runtime.lock().unwrap().clone().unwrap();let mut state=r.state.lock().unwrap();let i=old_count("gh_lockdown");let events=state.world.generic_actors[i].take_member_hits(&[0,2],12,35).unwrap();
    ordinary_enemy_presentation::present_events(&mut state,Some(&scene),&events);let cues:Vec<_>=state.presentation_events.iter().filter(|e|e.kind=="EnemyMemberDisperse").collect();assert_eq!(cues.len(),2);assert_ne!(cues[0].member_id,cues[1].member_id);assert!(cues.iter().all(|e|e.radius_m==0.15&&e.duration_ms.unwrap()>0));
    let before=state.next_presentation_event_id;let a=&state.world.generic_actors[i];let bad=ActorRuntimeEvent::MemberHit{actor_id:a.entity_id.clone(),member_id:format!("{}/member/99",a.entity_id),position_m:a.position_m,destroyed:true};ordinary_enemy_presentation::present_events(&mut state,Some(&scene),&[bad]);assert_eq!(state.next_presentation_event_id,before);
}
#[test]
fn late_ready_failure_rolls_back_roster_upgrade_and_preserves_source_save(){
    for scene in SCENES{
        let source=fixture(scene);let mut old=capture(&source);old.world_persistent_v1.grey_hive.swarm_actor_roster_version=0;old.save.generic_actors.truncate(old_count(scene));old.save.generic_actors[0].hp-=4;
        let r=fixture("gh_gate_b");crate::save_v6::write_save(&r.save_root,&old).unwrap();let path=crate::save_v6::save_path(&r.save_root);let bytes=std::fs::read(&path).unwrap();r.state.lock().unwrap().entry_generation=build_ui::MAX_SAFE_REVISION;
        let before=r.snapshot().unwrap();let before_save=capture(&r);assert_eq!(r.continue_saved().unwrap_err(),"E_SCENE_ENTRY_GENERATION_EXHAUSTED");assert_eq!(r.snapshot().unwrap(),before);assert_eq!(capture(&r),before_save);assert_eq!(std::fs::read(&path).unwrap(),bytes);
        r.state.lock().unwrap().entry_generation=0;let view=r.continue_saved().unwrap();assert!(view.entry_token.is_some()&&r.state.lock().unwrap().paused);assert_eq!(capture(&r).world_persistent_v1.grey_hive.swarm_actor_roster_version,1);assert_eq!(std::fs::read(&path).unwrap(),bytes);acknowledge_ready(&r,view);
    }
}
