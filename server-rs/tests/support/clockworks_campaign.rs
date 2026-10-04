//! Genuine continuation of the existing Grey Hive / Mist Harbor campaign.
//! Only public movement, Action-v2, authored interactions, gates and saves.
use super::*;
use std::{path::Path, collections::BTreeMap};
use wuxian_horror_ch1::{
    capability_v1::CAP_AIR_STEP,
    formal_runtime::{ActionCommandRequest, ActionKind},
    world_v3::WorldView,
};

const ROUTE: [(&str, &str); 8] = [
    ("cw_entry_foundry", "cw_to_pressure_hall"),
    ("cw_pressure_hall", "cw_to_conveyor_bridge"),
    ("cw_conveyor_bridge", "cw_bridge_to_boiler_chamber"),
    ("cw_boiler_chamber", "cw_boiler_to_gear_shaft"),
    ("cw_gear_shaft", "cw_gear_to_furnace_heart"),
    ("cw_furnace_heart", "cw_furnace_heart_to_arena"),
    ("cw_forged_guard_arena", "cw_arena_to_regulator_core"),
    ("cw_regulator_core", "cw_regulator_core_to_shutdown_exit"),
];

fn input(runtime: &FormalRuntime, view: &WorldView, movement: (f32, f32), aim: (f32, f32)) -> WorldView {
    runtime.submit_input(InputSample::new(view.world_epoch, view.ack_seq + 1,
        NEXT_TIME.fetch_add(17, Ordering::Relaxed), movement.0, movement.1).unwrap()
        .with_aim(aim.0, aim.1).unwrap(), vec![]).unwrap()
}

fn clear_live_enemies(runtime: &FormalRuntime, mut view: WorldView) -> WorldView {
    let scene_id = view.scene_id.clone();
    let mut last_event = 0;
    let mut wave: Option<(f32, f32, f32)> = None;
    let mut hold_until = 0;
    let mut attack_after = 0;
    let mut attacks = 0;
    let mut impacts = 0;
    let mut pressure_warnings = 0;
    let mut ordinary_warnings = BTreeMap::<String, (wuxian_horror_ch1::world_v3::PresentationEvent,u64)>::new();
    let mut deferred_events = Vec::new();
    let mut ordinary_seen = 0;
    let mut evade_inputs = 0;
    let mut brute_attack_kinds = BTreeSet::new();
    // Fixed before testing: allow 150 ms after a warning first becomes visible.
    // No private enemy HP, cooldown, AI state, or future event is a decision input.
    const REACTION_TICKS: u64 = 9;
    for step in 0..6000 {
        assert!(view.player.current_hp > 0, "genuine CW route died in {scene_id} at input {step}");
        for event in runtime.presentation_events_since(view.world_epoch, last_event).unwrap() {
            last_event = last_event.max(event.event_id);
            deferred_events.push(event);
        }
        let events = std::mem::take(&mut deferred_events);
        for event in events {
            if event.server_tick > view.server_tick {deferred_events.push(event);continue;}
            if event.kind == "EnemyAttackTelegraph" {
                let id = event.actor_id.clone().expect("ordinary warning identifies its public actor");
                assert!(event.duration_ms.is_some_and(|duration| duration>0));
                assert!(event.range_m.is_some_and(|range| range>0.0));
                assert!(matches!(event.attack_kind.as_deref(),Some("pressure_shot"|"bite"|"leap"|"charge"|"slam")));
                if scene_id=="gh_gate_b" && id=="gh_gate_b_brute_01" {
                    brute_attack_kinds.insert(event.attack_kind.clone().unwrap());
                }
                ordinary_warnings.insert(id,(event.clone(),view.server_tick));
                ordinary_seen += 1;
            } else if matches!(event.kind.as_str(),"EnemyAttackImpact"|"EnemyStagger"|"EnemyDeath") {
                if let Some(id)=&event.actor_id {ordinary_warnings.remove(id);}
            }
            if matches!(event.kind.as_str(), "ForgedGuardPressureWindup" | "PrimeRegulatorPressureWindup") {
                wave = Some((event.position_m.x_m, event.position_m.z_m, event.radius_m));
                pressure_warnings += 1;
            } else if matches!(event.kind.as_str(), "ForgedGuardPressureImpact" | "PrimeRegulatorPressureImpact") {
                wave = None;
            } else if event.kind == "Hit" { impacts += 1; }
        }
        ordinary_warnings.retain(|id,_|view.actors.iter().any(|actor|actor.entity_id==*id && actor.active));
        let player = view.player.transform.position_m;
        let target = view.actors.iter().filter(|a| a.active && a.actor_kind == "enemy")
            .min_by(|a, b| {
                let distance = |a: &wuxian_horror_ch1::world_v3::ActorView| {
                    let p = a.transform.position_m;
                    (p.x_m - player.x_m).hypot(p.z_m - player.z_m)
                };
                distance(a).total_cmp(&distance(b))
            });
        let Some(target) = target else {
            println!("Campaign genuine combat {scene_id}: {step} inputs, {attacks} attacks, {impacts} hit events, {pressure_warnings} pressure warnings, {ordinary_seen} ordinary warnings, {evade_inputs} cue-aware movement inputs (150 ms reaction), {} HP", view.player.current_hp);
            if attacks > 0 { assert!(impacts > 0, "real accepted hits must accompany a defeat"); }
            if scene_id=="gh_gate_b" {
                assert_eq!(brute_attack_kinds,["charge".to_owned(),"slam".to_owned()].into_iter().collect(),
                    "real Gate B combat must observe both public Brute attack warnings");
                assert!(evade_inputs>0,"Brute counterplay must use public warning geometry");
                assert_eq!(runtime.presentation_events_since(view.world_epoch,0).unwrap().iter()
                    .filter(|event|event.kind=="EnemyDeath" && event.actor_id.as_deref()==Some("gh_gate_b_brute_01")).count(),1);
            }

            if matches!(scene_id.as_str(), "cw_pressure_hall" | "cw_conveyor_bridge" | "cw_gear_shaft"
                | "cw_boiler_chamber" | "cw_furnace_heart") {
                assert!(ordinary_seen > 0, "admitted ordinary enemies must emit public warnings");
                assert!(evade_inputs > 0, "the genuine route must exercise warning-driven counterplay");
            }
            if matches!(scene_id.as_str(), "cw_forged_guard_arena" | "cw_regulator_core") {
                assert!(pressure_warnings > 0, "real Boss combat must observe ordinary pressure telegraphs");
            }
            return view;
        };
        let target = target.transform.position_m;
        let dx = target.x_m - player.x_m;
        let dz = target.z_m - player.z_m;
        let distance = dx.hypot(dz).max(f32::EPSILON);
        let aim = (dx / distance, dz / distance);
        // Ordinary hit-and-retreat movement, using no hidden actor HP or AI state.
        // Remain inside melee reach through the visible player attack's hit window.
        let mut movement = if let Some((x, z, radius)) = wave {
            let dx = player.x_m - x;
            let dz = player.z_m - z;
            let length = dx.hypot(dz).max(f32::EPSILON);
            if length < radius + 0.4 { (dx / length, dz / length) } else { (0.0, 0.0) }
        } else if view.server_tick < hold_until {
            (0.0, 0.0)
        } else if view.server_tick < attack_after {
            if distance < 2.7 { (-aim.0, -aim.1) } else { (0.0, 0.0) }
        } else if distance > 1.45 { aim } else { (0.0, 0.0) };
        // React to the same committed ground geometry shown by the renderer.
        // Preserve a shot/leap corridor until its public impact/stagger/death;
        // do not predict a projectile from hidden velocity or AI timers.
        let mut avoiding_warning = false;
        for (warning, observed_tick) in ordinary_warnings.values() {
            if view.server_tick < observed_tick + REACTION_TICKS {continue;}
            let origin=warning.position_m;
            let dx=player.x_m-origin.x_m;
            let dz=player.z_m-origin.z_m;
            let range=warning.range_m.unwrap();
            if matches!(warning.attack_kind.as_deref(),Some("bite"|"slam")) {
                let radial=dx.hypot(dz).max(f32::EPSILON);
                if radial<range+0.35 {
                    movement=(dx/radial,dz/radial);avoiding_warning=true;break;
                }
            } else {
                let direction=(warning.direction_rad.sin(),warning.direction_rad.cos());
                let along=dx*direction.0+dz*direction.1;
                let lateral=-direction.1*dx+direction.0*dz;
                let clearance=warning.radius_m+0.65; // own visible body footprint + margin
                let next_lateral=lateral+(-direction.1*movement.0+direction.0*movement.1)*0.8;
                if along>=-0.5 && along<=range+0.5 && (lateral.abs()<clearance || next_lateral.abs()<clearance) {
                    if lateral.abs()<clearance+0.3 {
                        let sign=if lateral>=0.0 {1.0}else{-1.0};
                        movement=(-direction.1*sign,direction.0*sign);
                        if player.x_m+movement.0<1.1 || player.x_m+movement.0>22.9
                            || player.z_m+movement.1<1.1 || player.z_m+movement.1>14.9 {
                            movement=(-movement.0,-movement.1);
                        }
                    } else {movement=(0.0,0.0);}
                    avoiding_warning=true;break;
                }
            }
        }
        if avoiding_warning {evade_inputs+=1;}
        // Follow the visible arena boundary instead of trying to leave its walls.
        if (player.x_m < 1.1 && movement.0 < 0.0) || (player.x_m > 22.9 && movement.0 > 0.0) {
            movement = (0.0, if player.z_m < 8.0 { 1.0 } else { -1.0 });
        }
        if (player.z_m < 1.1 && movement.1 < 0.0) || (player.z_m > 14.9 && movement.1 > 0.0) {
            movement = (if player.x_m < 12.0 { 1.0 } else { -1.0 }, 0.0);
        }
        let attack = !avoiding_warning && wave.is_none() && distance <= 1.55 && view.server_tick >= attack_after;
        view = input(runtime, &view, movement, aim);
        if attack {
            view = runtime.submit_action(ActionCommandRequest {
                protocol_version: 2, world_epoch: view.world_epoch,
                request_id: NEXT_COMBAT_REQUEST.fetch_add(1, Ordering::Relaxed),
                client_time_ms: NEXT_TIME.fetch_add(17, Ordering::Relaxed), kind: ActionKind::PrimaryAttack,
            }).unwrap();
            attacks += 1;
            hold_until = view.server_tick + 8;
            attack_after = view.server_tick + 72;
        }
    }
    panic!("genuine CW combat exceeded 6000 public inputs in {scene_id}");
}

/// Optional encounter exercised deliberately by the test, without changing a gameplay gate.
pub(super) fn clear_gate_b_enemies(runtime:&FormalRuntime, view:WorldView)->WorldView {
    assert_eq!(view.scene_id,"gh_gate_b");
    assert_eq!(view.actors.iter().filter(|actor|actor.entity_id=="gh_gate_b_brute_01" && actor.active).count(),1);
    clear_live_enemies(runtime,view)
}

fn interact(runtime: &FormalRuntime, view: WorldView, id: &str, request: &str) -> WorldView {
    let scene = view.scene_id.clone();
    let at = walk_to_compiled_target(runtime, view, &scene, "interactions", id);
    let result = scene_route_commands::interaction(runtime, id, request, Some(at.world_epoch)).unwrap();
    assert!(result.applied, "{scene}.{id}: {:?}", result.error_code);
    assert_eq!(result.receipt.command_id, request);
    assert!(result.view.player.current_hp > 0);
    result.view
}

fn rest_and_enter(runtime: &FormalRuntime, request: &str) -> WorldView {
    let at = walk_to_compiled_target(runtime, runtime.snapshot().unwrap(), "rs_core_room",
        "interactions", "rs_save_rest_terminal_marker");
    let rested = runtime.save_rest_terminal("rs_save_rest_terminal_marker", &format!("{request}-rest"), at.world_epoch).unwrap();
    assert_eq!(rested.player.current_hp, rested.player.max_hp);
    let at = walk_to_compiled_target(runtime, rested, "rs_core_room", "interactions", "rs_cw_world_gate_marker");
    let entered = scene_route_commands::world_gate(runtime, "rs_world_gate_to_cw", request, at.world_epoch).unwrap();
    assert_v3_receipt(&entered, request, true);
    let view = acknowledge_ready(runtime, runtime.snapshot().unwrap());
    assert_eq!(view.scene_id, "cw_entry_foundry");
    view
}

fn return_to_station(runtime: &FormalRuntime, view: WorldView, request: &str) -> WorldView {
    let at = walk_to_compiled_target(runtime, view, "cw_shutdown_exit", "interactions", "cw_shutdown_exit_portal_staged");
    let receipt = scene_route_commands::world_gate(runtime, "cw_shutdown_return_to_rs", request, at.world_epoch).unwrap();
    assert_v3_receipt(&receipt, request, true);
    let view = acknowledge_ready(runtime, runtime.snapshot().unwrap());
    assert_eq!(view.world_id, "return_station");
    view
}

pub(super) fn run(runtime: &FormalRuntime, save_dir: &Path) {
    let before = runtime.snapshot().unwrap();
    assert_eq!(before.world_id, "return_station");
    assert!(before.progression.worlds.iter().any(|w| w.world_id == "mist_harbor" && w.completed));
    assert!(!before.capabilities.items.iter().any(|i| i.capability_id == CAP_AIR_STEP && i.granted));
    let mut view = rest_and_enter(runtime, "genuine-cw-first-entry");
    for (scene, transition) in ROUTE {
        assert_eq!(view.scene_id, scene);
        view = clear_live_enemies(runtime, view);
        if scene == "cw_pressure_hall" {
            for number in 1..=3 {
                view = interact(runtime, view, &format!("cw_pressure_valve_0{number}_staged"), &format!("genuine-cw-valve-{number}"));
                if number == 2 {
                    runtime.save().unwrap();
                    let saved = save_v6::read_save(save_dir).unwrap();
                    assert_eq!(saved.world_persistent_v1.clockworks.pressure_valve_ids.len(), 2);
                    assert!(!saved.save.progression.progress.iter().find(|w| w.world_id == "clockworks").unwrap()
                        .completed_events.iter().any(|e| e == "clockworks_valves"));
                    let old_hp = view.player.current_hp;
                    view = acknowledge_ready(runtime, runtime.continue_saved().unwrap());
                    assert_eq!(view.player.current_hp, old_hp);
                }
            }
        }
        if scene == "cw_forged_guard_arena" {
            runtime.save().unwrap();
            assert!(save_v6::read_save(save_dir).unwrap().world_persistent_v1.clockworks.forged_guard_elite_first_kill);
            assert_eq!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter()
                .filter(|e| e.kind == "ForgedGuardEliteDeath").count(), 1);
        }
        if scene == "cw_regulator_core" {
            runtime.save().unwrap();
            let saved = save_v6::read_save(save_dir).unwrap();
            assert!(saved.world_persistent_v1.clockworks.regulator_defeated);
            assert!(!saved.world_persistent_v1.clockworks.core_console_confirmed);
            assert_eq!(saved.save.generic_actors[0].hp, 0);
            assert_eq!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter()
                .filter(|e| e.kind == "PrimeRegulatorDeath").count(), 1);
            view = acknowledge_ready(runtime, runtime.continue_saved().unwrap());
            assert!(view.actors.iter().all(|a| !a.active));
            assert!(!runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter().any(|e| e.kind == "PrimeRegulatorDeath"));
            view = interact(runtime, view, "cw_regulator_core_console_staged", "genuine-cw-core");
        }
        if scene=="cw_gear_shaft" {view=gear_shaft_ascent(runtime,view,save_dir,false);}
        view = walk_to_compiled_target(runtime, view, scene, "transitions", transition);
        view = apply_transition(runtime, transition, &format!("genuine-first-{transition}"), view.world_epoch);
    }
    assert_eq!(view.scene_id, "cw_shutdown_exit");
    assert!(!view.capabilities.items.iter().any(|i| i.capability_id == CAP_AIR_STEP && i.granted));
    view = interact(runtime, view, "cw_master_shutdown_staged", "genuine-cw-shutdown");
    let completed = view.progression.worlds.iter().find(|w| w.world_id == "clockworks").unwrap();
    assert!(completed.completed && completed.first_completion);
    assert_eq!(completed.completed_events.iter().cloned().collect::<BTreeSet<_>>(),
        ["clockworks_valves", "clockworks_core", "clockworks_shutdown"].into_iter().map(str::to_owned).collect());
    runtime.save().unwrap();
    let settled = save_v6::read_save(save_dir).unwrap();
    assert_eq!(settled.save.capabilities.grants.iter().filter(|g| g.capability_id == CAP_AIR_STEP).count(), 1);
    assert_eq!(settled.effect_sources.iter().filter(|s| s.source_id == CAP_AIR_STEP).count(), 1);
    assert!(settled.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(settled.world_persistent_v1.clockworks.pressure_valve_ids.len(), 3);
    let progress = view.progression.clone();
    let repeated = scene_route_commands::interaction(runtime, "cw_master_shutdown_staged", "genuine-cw-shutdown-again", Some(view.world_epoch)).unwrap();
    assert!(!repeated.applied);
    assert_eq!(repeated.view.progression, progress);
    view = acknowledge_ready(runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.progression, progress);
    return_to_station(runtime, view, "genuine-cw-return");

    view = rest_and_enter(runtime, "genuine-cw-revisit");
    assert_eq!(view.progression.worlds.iter().find(|w| w.world_id == "clockworks").unwrap().revisit_count, 1);
    for (scene, transition) in ROUTE {
        assert_eq!(view.scene_id, scene);
        if matches!(scene, "cw_forged_guard_arena" | "cw_regulator_core") {
            assert!(view.actors.iter().all(|a| !a.active), "defeated Boss must remain dead on revisit");
            assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter()
                .all(|e| !matches!(e.kind.as_str(), "PrimeRegulatorDeath" | "ForgedGuardEliteDeath")));
        } else { view = clear_live_enemies(runtime, view); }
        if scene=="cw_gear_shaft" {view=gear_shaft_ascent(runtime,view,save_dir,true);}
        view = walk_to_compiled_target(runtime, view, scene, "transitions", transition);
        view = apply_transition(runtime, transition, &format!("genuine-revisit-{transition}"), view.world_epoch);
    }
    runtime.save().unwrap();
    let revisited = save_v6::read_save(save_dir).unwrap();
    assert_eq!(revisited.effect_sources, settled.effect_sources, "revisit cannot duplicate or change acquired sources");
    assert_eq!(revisited.save.capabilities.grants.len(), settled.save.capabilities.grants.len());
    assert!(revisited.world_persistent_v1.clockworks.regulator_defeated);
    let returned = return_to_station(runtime, view, "genuine-cw-revisit-return");
    assert!(returned.player.current_hp > 0);
    println!("Genuine GH→MH→CW campaign, shutdown, save/Continue and dead-Boss revisit completed alive");
}

/// Actual authored main route, then an earned-only shortcut on the real revisit.
fn gear_shaft_ascent(runtime:&FormalRuntime,mut view:WorldView,save_dir:&Path,earned:bool)->WorldView {
    assert_eq!(view.scene_id,"cw_gear_shaft");assert!(view.actors.iter().all(|a|!a.active),"the genuine driver must earn every prior defeat");
    assert_eq!(view.capabilities.items.iter().any(|c|c.capability_id==CAP_AIR_STEP&&c.granted),earned);
    // The outer south lane keeps the approach on the visible lower floor.
    view=gear_walk(runtime,view,12.,11.);
    if earned {
        runtime.save().unwrap();let before=save_v6::read_save(save_dir).unwrap();
        assert!(before.save.progression.progress.iter().find(|p|p.world_id=="clockworks").unwrap().completed_events.iter().any(|e|e=="clockworks_shutdown"));
        assert_eq!(before.effect_sources.iter().filter(|s|s.source_id==CAP_AIR_STEP).count(),1);
        assert_eq!(before.save.capabilities.grants.iter().filter(|g|g.capability_id==CAP_AIR_STEP).count(),1);
        view=gear_walk(runtime,view,10.1,8.);assert_eq!(view.player.transform.position_m.y_m,0.);
        view=runtime.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:view.world_epoch,
            request_id:NEXT_COMBAT_REQUEST.fetch_add(1,Ordering::Relaxed),client_time_ms:NEXT_TIME.fetch_add(17,Ordering::Relaxed),kind:ActionKind::ContextTraversal}).unwrap();
        for _ in 0..90{if view.support_scene.as_ref().unwrap().rider.support_id.as_deref()==Some("cw_gear_upper_dock"){break;}view=input(runtime,&view,(0.,0.),(1.,0.));}
        assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_upper_dock"));
        runtime.save().unwrap();assert_eq!(save_v6::read_save(save_dir).unwrap().effect_sources,before.effect_sources,"AirStep traversal cannot create another source");
    }else{
        runtime.save().unwrap();let before=save_v6::read_save(save_dir).unwrap();
        assert!(!before.save.progression.progress.iter().find(|p|p.world_id=="clockworks").unwrap().completed_events.iter().any(|e|e=="clockworks_shutdown"));
        assert!(!before.effect_sources.iter().any(|s|s.source_id==CAP_AIR_STEP));
        assert!(!before.save.capabilities.grants.iter().any(|g|g.capability_id==CAP_AIR_STEP));
        for _ in 0..850{let lift=view.support_scene.as_ref().unwrap().poses.iter().find(|p|p.support_id=="cw_gear_main_lift").unwrap();
            if lift.phase==wuxian_horror_ch1::moving_support::SupportPhase::LowerHold&&lift.phase_elapsed_ms<600{break;}view=input(runtime,&view,(0.,0.),(1.,0.));}
        view=gear_walk(runtime,view,12.,8.);assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_main_lift"));
        let mut saved=false;
        for _ in 0..550{
            let y=view.player.transform.position_m.y_m;
            if !saved&&y>0.7&&y<1.5{let held=runtime.pause().unwrap();runtime.save().unwrap();let bytes=std::fs::read(save_dir.join("formal-save-v6.json")).unwrap();
                let prepared=runtime.continue_saved().unwrap();assert_eq!(prepared.player.transform.position_m,held.player.transform.position_m);assert_eq!(prepared.player.current_hp,held.player.current_hp);
                assert_eq!(prepared.server_time_ms,held.server_time_ms);assert_eq!(prepared.support_scene.as_ref().unwrap().poses,held.support_scene.as_ref().unwrap().poses);
                assert_eq!(std::fs::read(save_dir.join("formal-save-v6.json")).unwrap(),bytes);view=acknowledge_ready(runtime,prepared);saved=true;}
            if (view.player.transform.position_m.y_m-2.).abs()<0.001{break;}view=input(runtime,&view,(0.,0.),(1.,0.));
        }
        assert!(saved);assert!((view.player.transform.position_m.y_m-2.).abs()<0.001);view=gear_walk(runtime,view,16.,8.);
        assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_upper_dock"));
    }
    let checkpoint=scene_route_commands::checkpoint(runtime,"cw_gear_upper_dock_checkpoint",if earned{"genuine-gear-upper-revisit"}else{"genuine-gear-upper-first"},view.world_epoch).unwrap();assert!(checkpoint.applied);
    view=gear_walk(runtime,runtime.snapshot().unwrap(),17.8,8.);
    view=runtime.submit_action(ActionCommandRequest{protocol_version:2,world_epoch:view.world_epoch,
        request_id:NEXT_COMBAT_REQUEST.fetch_add(1,Ordering::Relaxed),client_time_ms:NEXT_TIME.fetch_add(17,Ordering::Relaxed),kind:ActionKind::ContextTraversal}).unwrap();
    for _ in 0..90{if view.support_scene.as_ref().unwrap().rider.support_id.as_deref()==Some("cw_gear_furnace_landing"){break;}view=input(runtime,&view,(0.,0.),(1.,0.));}
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(),Some("cw_gear_furnace_landing"));assert!(view.player.current_hp>0);
    println!("Genuine Gear Shaft {} ascent, actual upper gap, {} HP",if earned{"earned AirStep"}else{"pre-reward lift and mid-ride Continue"},view.player.current_hp);view
}

fn gear_walk(runtime:&FormalRuntime,mut view:WorldView,x:f32,z:f32)->WorldView{
    for _ in 0..600{assert!(view.player.current_hp>0);let p=view.player.transform.position_m;let dx=x-p.x_m;let dz=z-p.z_m;let d=dx.hypot(dz);
        if d<0.12{return input(runtime,&view,(0.,0.),(1.,0.));}view=input(runtime,&view,(dx/d,dz/d),(dx/d,dz/d));}
    panic!("genuine Gear Shaft walk failed to reach ({x},{z}) from {:?}",view.player.transform.position_m)
}
