//! Genuine continuation of the existing Grey Hive / Mist Harbor campaign.
//! Only public movement, Action-v2, authored interactions, gates and saves.
use super::*;
use std::{path::Path, collections::BTreeMap};
use wuxian_horror_ch1::{
    capability_v1::CAP_AIR_STEP,
    formal_runtime::{ActionCommandRequest, ActionKind},
    world_v3::WorldView,
};

// Exact final SaveV6 from the successful Warden/extraction prefix, 14,242 bytes, SHA-256 233275e296937b66956159f6bd89f9e6d08130298fd3ebfef84e52363158a0fb.
const EARNED_CW_FOUNDRY_ENTRY_V6: &str = include_str!("../fixtures/earned-cw-foundry-entry-v6.json");

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

fn assert_earned_clockworks_prefix(view: &WorldView) {
    assert_eq!(view.progression.current_world_id, "clockworks");
    for world_id in ["grey_hive", "mist_harbor"] {
        let progress = view.progression.worlds.iter().find(|row| row.world_id == world_id)
            .unwrap_or_else(|| panic!("missing retained {world_id} progression"));
        assert!(progress.completed && progress.first_completion,
            "earned {world_id} completion must survive the Clockworks route");
    }
    let rear_view = view.capabilities.items.iter()
        .find(|item| item.capability_id == wuxian_horror_ch1::capability_v1::CAP_REAR_VIEW)
        .expect("earned Rear View remains projected");
    assert!(rear_view.granted && rear_view.selected);
    let acoustic = view.capabilities.items.iter()
        .find(|item| item.capability_id == wuxian_horror_ch1::capability_v1::CAP_ACOUSTIC_MAPPING)
        .expect("earned Acoustic Mapping remains projected");
    assert!(acoustic.granted);
    assert!(!acoustic.selected);
    assert!(!view.capabilities.items.iter()
        .any(|item| item.capability_id == CAP_AIR_STEP && item.granted),
        "pressure valves alone cannot grant AirStep");
}

fn assert_clockworks_pressure_hall_state(
    view: &WorldView,
    active: [bool; 3],
    receipts: &[&str],
    valves_event: bool,
) {
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_pressure_hall");
    assert!(view.player.current_hp > 0);
    let progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks")
        .expect("Clockworks progression is projected");
    assert_eq!(progress.visit_id, 1);
    assert_eq!(progress.cycle_id, 1);
    assert_eq!(progress.revisit_count, 0);
    assert!(!progress.completed && !progress.first_completion);
    let expected_receipts = receipts.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>();
    assert_eq!(progress.pressure_valve_receipts.as_ref(), Some(&expected_receipts));
    let actual_events = progress.completed_events.iter().cloned().collect::<BTreeSet<_>>();
    let expected_events = if valves_event {
        BTreeSet::from(["clockworks_valves".to_owned()])
    } else {
        BTreeSet::new()
    };
    assert_eq!(actual_events, expected_events);
    for (number, expected_active) in active.into_iter().enumerate() {
        let id = format!("cw_pressure_valve_0{}_staged", number + 1);
        let control = view.interactables.iter().find(|item| item.entity_id == id)
            .unwrap_or_else(|| panic!("missing authored Clockworks valve projection {id}"));
        assert_eq!(control.kind, "valve_control");
        assert_eq!(control.active, expected_active, "{id} availability");
    }
}

#[test]
fn genuine_earned_foundry_pressure_valves_and_conveyor_continue() {
    NEXT_TIME.fetch_max(1_000_000, Ordering::Relaxed);
    let save_dir = save_root();
    assert!(!save_dir.exists(), "test save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    std::fs::write(save_dir.join("formal-save-v6.json"), EARNED_CW_FOUNDRY_ENTRY_V6.as_bytes()).unwrap();

    let initial_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(initial_save.save.world_id, "clockworks");
    assert_eq!(initial_save.save.scene_id, "cw_entry_foundry");
    assert_eq!(initial_save.world_persistent_v1.clockworks.pressure_valve_ids, Vec::<String>::new());
    assert!(initial_save.save.progression.progress.iter().find(|row| row.world_id == "clockworks")
        .is_some_and(|row| row.visit_id == 1 && !row.completed && row.completed_events.is_empty()));

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_entry_foundry");
    assert_eq!(view.player.current_hp, 86, "use the earned prefix without healing");
    assert_eq!(view.player.current_energy, 28, "use the earned prefix without replenishing energy");
    assert!(view.player.current_hp > 0);
    assert_earned_clockworks_prefix(&view);
    let foundry_progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(foundry_progress.visit_id, 1);
    assert_eq!(foundry_progress.revisit_count, 0);
    assert!(!foundry_progress.completed && !foundry_progress.first_completion);
    assert!(foundry_progress.completed_events.is_empty());
    assert_eq!(view.actors.iter().filter(|actor| actor.active && actor.actor_kind == "enemy").count(), 2,
        "the saved Foundry entry must contain its actual two guards");
    view = clear_live_enemies(&runtime, view);
    assert!(view.player.current_hp > 0);
    assert!(view.actors.iter().all(|actor| !actor.active || actor.actor_kind != "enemy"));
    println!("Earned prefix: continued in Foundry and cleared its actual guards with {} HP/{} energy",
        view.player.current_hp, view.player.current_energy);

    view = walk_to_compiled_target(&runtime, view, "cw_entry_foundry", "transitions", "cw_to_pressure_hall");
    view = apply_transition(&runtime, "cw_to_pressure_hall", "earned-cw-foundry-to-hall", view.world_epoch);
    assert_earned_clockworks_prefix(&view);
    assert_eq!(view.scene_id, "cw_pressure_hall");
    assert_clockworks_pressure_hall_state(&view, [true; 3], &[], false);
    let active_enemies = view.actors.iter().filter(|actor| actor.active && actor.actor_kind == "enemy")
        .map(|actor| actor.entity_id.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(active_enemies, BTreeSet::from([
        "cw_pressure_hall_forged_guard_01",
        "cw_pressure_hall_forged_guard_02",
        "cw_pressure_hall_pressure_drone_01",
        "cw_pressure_hall_pressure_drone_02",
    ]), "Pressure Hall combat uses the four exact authored enemy actors");
    view = clear_live_enemies(&runtime, view);
    assert!(view.player.current_hp > 0);
    assert!(view.actors.iter().all(|actor| !actor.active || actor.actor_kind != "enemy"));
    assert_clockworks_pressure_hall_state(&view, [true; 3], &[], false);

    view = interact(&runtime, view, "cw_pressure_valve_01_staged", "earned-cw-valve-01");
    assert_clockworks_pressure_hall_state(&view, [false, true, true], &["cw_pressure_valve_01_staged"], false);
    view = interact(&runtime, view, "cw_pressure_valve_02_staged", "earned-cw-valve-02");
    let two_valves = ["cw_pressure_valve_01_staged", "cw_pressure_valve_02_staged"];
    assert_clockworks_pressure_hall_state(&view, [false, false, true], &two_valves, false);
    println!("Pressure Hall: earned the first two public valve receipts at {} HP/{} energy",
        view.player.current_hp, view.player.current_energy);

    runtime.pause().unwrap();
    runtime.save().unwrap();
    let two_valve_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(two_valve_save.world_persistent_v1.clockworks.pressure_valve_ids,
        two_valves.map(str::to_owned));
    assert!(two_valve_save.save.progression.progress.iter().find(|row| row.world_id == "clockworks")
        .is_some_and(|row| row.completed_events.is_empty() && !row.completed));
    let saved_hp = view.player.current_hp;
    let saved_energy = view.player.current_energy;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_earned_clockworks_prefix(&view);
    assert_eq!(view.player.current_hp, saved_hp);
    assert_eq!(view.player.current_energy, saved_energy);
    assert_clockworks_pressure_hall_state(&view, [false, false, true], &two_valves, false);
    assert!(view.actors.iter().all(|actor| !actor.active || actor.actor_kind != "enemy"));

    view = walk_to_compiled_target(&runtime, view, "cw_pressure_hall", "transitions", "cw_to_conveyor_bridge");
    let locked_affordance = view.interactables.iter().find(|item| item.entity_id == "cw_to_conveyor_bridge")
        .expect("authored Conveyor transition is projected at its real polygon");
    assert_eq!(locked_affordance.kind, "scene_transition");
    assert!(!locked_affordance.active, "two receipts leave the progression gate closed at its polygon");
    let locked = scene_route_commands::transition(&runtime, "cw_to_conveyor_bridge",
        "earned-cw-conveyor-rejected-two-valves", view.world_epoch).unwrap();
    assert_v3_receipt(&locked, "earned-cw-conveyor-rejected-two-valves", false);
    assert_eq!(locked.error_code.as_deref(), Some("E_SCENE_RUNTIME_ProgressionLocked"));
    assert_eq!(locked.snapshot.scene_id, "cw_pressure_hall");
    assert!(locked.snapshot.progression.worlds.iter().find(|row| row.world_id == "clockworks")
        .is_some_and(|row| row.completed_events.is_empty()));
    assert_eq!(runtime.snapshot().unwrap().scene_id, "cw_pressure_hall");
    println!("Pressure Hall: Conveyor transition returned E_SCENE_RUNTIME_ProgressionLocked at the authored polygon");

    view = interact(&runtime, view, "cw_pressure_valve_03_staged", "earned-cw-valve-03");
    let three_valves = [
        "cw_pressure_valve_01_staged",
        "cw_pressure_valve_02_staged",
        "cw_pressure_valve_03_staged",
    ];
    assert_clockworks_pressure_hall_state(&view, [false; 3], &three_valves, true);
    println!("Pressure Hall: earned all three receipts and the authored aggregate event");

    runtime.pause().unwrap();
    runtime.save().unwrap();
    let three_valve_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(three_valve_save.world_persistent_v1.clockworks.pressure_valve_ids,
        three_valves.map(str::to_owned));
    let saved_hall = three_valve_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(saved_hall.completed_events, vec!["clockworks_valves"]);
    assert!(!saved_hall.completed && !saved_hall.first_completion);
    let saved_hp = view.player.current_hp;
    let saved_energy = view.player.current_energy;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_earned_clockworks_prefix(&view);
    assert_eq!(view.player.current_hp, saved_hp);
    assert_eq!(view.player.current_energy, saved_energy);
    assert_clockworks_pressure_hall_state(&view, [false; 3], &three_valves, true);
    assert!(view.actors.iter().all(|actor| !actor.active || actor.actor_kind != "enemy"));

    view = walk_to_compiled_target(&runtime, view, "cw_pressure_hall", "transitions", "cw_to_conveyor_bridge");
    let open_affordance = view.interactables.iter().find(|item| item.entity_id == "cw_to_conveyor_bridge")
        .expect("authored Conveyor transition is projected at its real polygon");
    assert_eq!(open_affordance.kind, "scene_transition");
    assert!(open_affordance.active, "all three receipts unlock the route at its polygon");
    view = apply_transition(&runtime, "cw_to_conveyor_bridge", "earned-cw-hall-to-conveyor", view.world_epoch);
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_conveyor_bridge");
    assert!(view.player.current_hp > 0);
    assert_earned_clockworks_prefix(&view);
    let conveyor_progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(conveyor_progress.visit_id, 1);
    assert_eq!(conveyor_progress.revisit_count, 0);
    assert!(!conveyor_progress.completed && !conveyor_progress.first_completion);
    assert_eq!(conveyor_progress.completed_events, vec!["clockworks_valves"]);
    runtime.pause().unwrap();
    runtime.save().unwrap();
    let conveyor_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(conveyor_save.save.world_id, "clockworks");
    assert_eq!(conveyor_save.save.scene_id, "cw_conveyor_bridge");
    assert_eq!(conveyor_save.world_persistent_v1.clockworks.pressure_valve_ids,
        three_valves.map(str::to_owned));
    let saved_conveyor = conveyor_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(saved_conveyor.completed_events, vec!["clockworks_valves"]);
    assert_eq!(saved_conveyor.visit_id, 1);
    assert_eq!(saved_conveyor.revisit_count, 0);
    assert!(!saved_conveyor.completed && !saved_conveyor.first_completion);
    assert!(!conveyor_save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == CAP_AIR_STEP));
    println!("Conveyor entry saved without combat at {} HP/{} energy: {}",
        view.player.current_hp, view.player.current_energy, save_dir.display());
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_conveyor_bridge");
    assert!(view.player.current_hp > 0);
    assert_earned_clockworks_prefix(&view);
    let continued_progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(continued_progress.visit_id, 1);
    assert_eq!(continued_progress.revisit_count, 0);
    assert!(!continued_progress.completed && !continued_progress.first_completion);
    assert_eq!(continued_progress.completed_events, vec!["clockworks_valves"]);
    runtime.pause().unwrap();
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(final_save.save.scene_id, "cw_conveyor_bridge");
    assert_eq!(final_save.world_persistent_v1.clockworks.pressure_valve_ids,
        three_valves.map(str::to_owned));
    assert!(!final_save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == CAP_AIR_STEP));
    println!("Fresh Conveyor Continue retained the earned route at {} HP/{} energy; final save: {}",
        view.player.current_hp, view.player.current_energy, save_dir.display());
    drop(runtime);
}

// Exact SaveV6 retained by the successful Conveyor Continue scenario: 16,488 bytes,
// SHA-256 6a23121dd5c30063a33a922b594a284ee4077839a01aecb5db95713a8229ce47.
const EARNED_CW_CONVEYOR_ENTRY_V6: &str = include_str!("../fixtures/earned-cw-conveyor-entry-v6.json");

fn assert_earned_valve_route_view(view: &WorldView, scene_id: &str) {
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, scene_id);
    assert_earned_clockworks_prefix(view);
    let progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks")
        .expect("Clockworks route progress remains projected");
    assert_eq!(progress.visit_id, 1);
    assert_eq!(progress.cycle_id, 1);
    assert_eq!(progress.revisit_count, 0);
    assert!(!progress.completed && !progress.first_completion);
    assert_eq!(progress.completed_events, vec!["clockworks_valves"]);
}

fn assert_earned_valve_route_save(save: &save_v6::SaveV6, scene_id: &str) {
    assert_eq!(save.save.world_id, "clockworks");
    assert_eq!(save.save.scene_id, scene_id);
    let progress = save.save.progression.progress.iter().find(|row| row.world_id == "clockworks")
        .expect("Clockworks route progress is saved");
    assert_eq!(progress.visit_id, 1);
    assert_eq!(progress.cycle_id, 1);
    assert_eq!(progress.revisit_count, 0);
    assert!(!progress.completed && !progress.first_completion);
    assert_eq!(progress.completed_events, vec!["clockworks_valves"]);
    assert_eq!(save.world_persistent_v1.clockworks.pressure_valve_ids, vec![
        "cw_pressure_valve_01_staged".to_owned(),
        "cw_pressure_valve_02_staged".to_owned(),
        "cw_pressure_valve_03_staged".to_owned(),
    ]);
    assert!(!save.save.capabilities.grants.iter().any(|grant| grant.capability_id == CAP_AIR_STEP));
}

fn active_enemy_ids(view: &WorldView) -> BTreeSet<String> {
    view.actors.iter().filter(|actor| actor.active && actor.actor_kind == "enemy")
        .map(|actor| actor.entity_id.clone()).collect()
}

fn walk_to_authored_traversal_start(
    runtime: &FormalRuntime,
    mut view: WorldView,
    scene_id: &str,
    marker_id: &str,
) -> WorldView {
    const MAX_INPUT_STEPS: usize = 600;
    let scene = compiled_route_scene(scene_id);
    let marker = scene["logic"]["traversal"].as_array().expect("compiled traversal list")
        .iter().find(|marker| marker["id"] == marker_id)
        .unwrap_or_else(|| panic!("{scene_id} lacks authored traversal {marker_id}"));
    let from = marker["from"].as_array().expect("authored traversal start");
    let target_x = from[0].as_f64().expect("traversal start x") as f32;
    let target_y = from[1].as_f64().expect("traversal start y") as f32;
    let target_z = from[2].as_f64().expect("traversal start z") as f32;
    assert_eq!((target_x, target_y, target_z), (8.0, 0.0, 8.0), "authored Conveyor vault start");

    for _ in 0..MAX_INPUT_STEPS {
        assert!(view.player.current_hp > 0, "player must stay alive approaching {marker_id}");
        let position = view.player.transform.position_m;
        let dx = target_x - position.x_m;
        let dz = target_z - position.z_m;
        let distance = dx.hypot(dz);
        if distance <= 0.08 {
            let stopped = input(runtime, &view, (0.0, 0.0), (1.0, 0.0));
            assert_eq!(stopped.scene_id, scene_id);
            assert!(stopped.player.current_hp > 0);
            return stopped;
        }
        let length = distance.max(f32::EPSILON);
        view = input(runtime, &view, (dx / length, dz / length), (dx / length, dz / length));
    }
    panic!("could not reach authored {scene_id}.{marker_id} start from {:?}", view.player.transform.position_m);
}

#[test]
fn genuine_earned_conveyor_boiler_and_lower_gear_continue() {
    NEXT_TIME.fetch_max(600_000, Ordering::Relaxed);
    let save_dir = save_root();
    assert!(!save_dir.exists(), "test save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    std::fs::write(save_dir.join("formal-save-v6.json"), EARNED_CW_CONVEYOR_ENTRY_V6.as_bytes()).unwrap();

    let initial_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&initial_save, "cw_conveyor_bridge");
    assert_eq!(initial_save.save.player.current_hp, 86);
    assert_eq!(initial_save.save.player.current_energy, 28);
    assert_eq!(initial_save.save.generic_actors.len(), 5);
    assert_eq!(initial_save.save.generic_actors.iter().map(|actor| actor.entity_id.as_str()).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "cw_conveyor_bridge_forged_guard_01",
            "cw_conveyor_bridge_forged_guard_02",
            "cw_conveyor_bridge_forged_guard_03",
            "cw_conveyor_bridge_pressure_drone_01",
            "cw_conveyor_bridge_pressure_drone_02",
        ]));

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_conveyor_bridge");
    assert_eq!(view.player.current_hp, 86);
    assert_eq!(view.player.current_energy, 28);
    assert!(view.player.current_hp > 0);
    assert_earned_valve_route_view(&view, "cw_conveyor_bridge");
    let expected_conveyor_enemies = BTreeSet::from([
        "cw_conveyor_bridge_forged_guard_01".to_owned(),
        "cw_conveyor_bridge_forged_guard_02".to_owned(),
        "cw_conveyor_bridge_forged_guard_03".to_owned(),
        "cw_conveyor_bridge_pressure_drone_01".to_owned(),
        "cw_conveyor_bridge_pressure_drone_02".to_owned(),
    ]);
    assert_eq!(active_enemy_ids(&view), expected_conveyor_enemies,
        "Continue must restore the five actual Conveyor enemies");
    view = clear_live_enemies(&runtime, view);
    assert!(view.player.current_hp > 0);
    assert!(active_enemy_ids(&view).is_empty());
    assert_earned_valve_route_view(&view, "cw_conveyor_bridge");
    println!("Conveyor: cleared its actual five enemies alive at {} HP/{} energy",
        view.player.current_hp, view.player.current_energy);

    // Approach only the authored marker start. The production ActionV2 command remains the
    // authority that decides whether a loaded marker in range permits this free traversal.
    let _at_vault = walk_to_authored_traversal_start(&runtime, view, "cw_conveyor_bridge", "cw_conveyor_context_vault");
    let before_action = runtime.snapshot().unwrap();
    assert_eq!(before_action.world_id, "clockworks");
    assert_eq!(before_action.scene_id, "cw_conveyor_bridge");
    assert!(before_action.player.current_hp > 0);
    let start_position = before_action.player.transform.position_m;
    assert!(start_position.y_m.abs() <= 0.1, "vault starts on the actual lower Conveyor floor");
    let authored_scene = compiled_route_scene("cw_conveyor_bridge");
    let marker = authored_scene["logic"]["traversal"].as_array().unwrap().iter()
        .find(|marker| marker["id"] == "cw_conveyor_context_vault").unwrap();
    assert!(marker["requiredCapabilities"].as_array().unwrap().is_empty(),
        "the authored Conveyor vault is capability-free");
    let event_cursor = runtime.presentation_events_since(before_action.world_epoch, 0).unwrap()
        .iter().map(|event| event.event_id).max().unwrap_or(0);
    let epoch = before_action.world_epoch;
    let owner_start_ms = before_action.server_time_ms;
    let wall_start = std::time::Instant::now();
    let action_view = runtime.submit_action(ActionCommandRequest {
        protocol_version: 2,
        world_epoch: epoch,
        request_id: NEXT_COMBAT_REQUEST.fetch_add(1, Ordering::Relaxed),
        client_time_ms: NEXT_TIME.fetch_add(17, Ordering::Relaxed),
        kind: ActionKind::ContextTraversal,
    }).unwrap();
    assert_eq!(action_view.world_epoch, epoch);
    let mut observed = action_view.clone();
    let mut started_event = None;
    let mut completed_event = None;
    let mut blocked_event = None;
    for _ in 0..300 {
        observed = runtime.snapshot().unwrap();
        assert_eq!(observed.world_epoch, epoch, "vault remains in its originating epoch");
        for event in runtime.presentation_events_since(epoch, event_cursor).unwrap() {
            if event.event_id <= event_cursor || event.world_epoch != epoch { continue; }
            match event.kind.as_str() {
                "ContextTraversalStarted" => started_event = Some(event.event_id),
                "ContextTraversalCompleted" => completed_event = Some(event.event_id),
                "ContextTraversalBlocked" => blocked_event = Some(event.event_id),
                _ => {}
            }
        }
        if completed_event.is_some() { break; }
        let elapsed_owner_ms = observed.server_time_ms.saturating_sub(owner_start_ms);
        if elapsed_owner_ms >= 3_000 || wall_start.elapsed() >= std::time::Duration::from_secs(3) { break; }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(started_event.is_some(), "one new authoritative ContextTraversalStarted must follow the cursor");
    assert!(completed_event.is_some(), "one new authoritative ContextTraversalCompleted must follow the cursor within 3 seconds");
    assert!(blocked_event.is_none(), "the authored free vault must not emit a blocked outcome");
    assert!(started_event.unwrap() < completed_event.unwrap(), "vault start precedes its completion");
    assert!(observed.server_time_ms.saturating_sub(owner_start_ms) <= 3_000,
        "vault completion is bounded by three seconds of owner time");
    let end_position = observed.player.transform.position_m;
    assert_eq!(observed.scene_id, "cw_conveyor_bridge");
    assert!(observed.player.current_hp > 0);
    assert!(end_position.y_m.abs() <= 0.1, "vault completes on the authored lower floor");
    let forward_m = end_position.x_m - start_position.x_m;
    assert!(forward_m >= 1.0,
        "authoritative vault completion must accompany at least 1m of observed forward movement: {start_position:?} -> {end_position:?}");
    println!("Conveyor vault: new Started/Completed cues with {:.2}m forward displacement, {} HP",
        forward_m, observed.player.current_hp);

    view = walk_to_compiled_target(&runtime, observed, "cw_conveyor_bridge", "transitions", "cw_bridge_to_boiler_chamber");
    view = apply_transition(&runtime, "cw_bridge_to_boiler_chamber", "earned-cw-conveyor-to-boiler", view.world_epoch);
    assert_eq!(view.scene_id, "cw_boiler_chamber");
    assert!(view.player.current_hp > 0);
    assert_earned_valve_route_view(&view, "cw_boiler_chamber");
    let expected_hounds = BTreeSet::from([
        "cw_boiler_chamber_furnace_hound_01".to_owned(),
        "cw_boiler_chamber_furnace_hound_02".to_owned(),
        "cw_boiler_chamber_furnace_hound_03".to_owned(),
    ]);
    assert_eq!(active_enemy_ids(&view), expected_hounds, "Boiler entry loads its exact three Furnace Hounds");
    view = clear_live_enemies(&runtime, view);
    assert!(view.player.current_hp > 0, "the actual Boiler encounter must end alive");
    assert!(active_enemy_ids(&view).is_empty(), "all three actual Furnace Hounds must be defeated");
    println!("Boiler: defeated its actual three Furnace Hounds at {} HP/{} energy",
        view.player.current_hp, view.player.current_energy);

    let boiler_epoch = view.world_epoch;
    let saved_boiler_view = runtime.pause().unwrap();
    runtime.save().unwrap();
    let boiler_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&boiler_save, "cw_boiler_chamber");
    assert_eq!(boiler_save.save.player.current_hp, saved_boiler_view.player.current_hp);
    assert_eq!(boiler_save.save.player.current_energy, saved_boiler_view.player.current_energy);
    assert!(boiler_save.save.generic_actors.iter().all(|actor| actor.hp == 0),
        "typed Boiler actor saves retain all defeated enemies at 0 HP");
    assert_eq!(boiler_save.save.generic_actors.len(), 3);
    assert_eq!(boiler_save.save.generic_actors.iter().map(|actor| actor.entity_id.as_str()).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "cw_boiler_chamber_furnace_hound_01",
            "cw_boiler_chamber_furnace_hound_02",
            "cw_boiler_chamber_furnace_hound_03",
        ]));
    let saved_boiler_hp = saved_boiler_view.player.current_hp;
    let saved_boiler_energy = saved_boiler_view.player.current_energy;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_ne!(view.world_epoch, boiler_epoch, "Boiler Continue reconstructs a new owner epoch");
    assert_earned_valve_route_view(&view, "cw_boiler_chamber");
    assert_eq!(view.player.current_hp, saved_boiler_hp);
    assert_eq!(view.player.current_energy, saved_boiler_energy);
    assert!(active_enemy_ids(&view).is_empty(), "Boiler Continue must restore the defeated Hounds");
    assert_eq!(view.actors.iter().filter(|actor| actor.actor_kind == "enemy").count(), 3);
    assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "Boiler Continue cannot replay enemy deaths in its new epoch");
    let continued_boiler = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&continued_boiler, "cw_boiler_chamber");
    assert_eq!(continued_boiler.world_persistent_v1.environment, boiler_save.world_persistent_v1.environment,
        "Continue retains the actual saved environment state");
    assert_eq!(continued_boiler.save.capabilities.grants, boiler_save.save.capabilities.grants,
        "Continue retains the earned grants");

    view = walk_to_compiled_target(&runtime, view, "cw_boiler_chamber", "transitions", "cw_boiler_to_gear_shaft");
    view = apply_transition(&runtime, "cw_boiler_to_gear_shaft", "earned-cw-boiler-to-lower-gear", view.world_epoch);
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_gear_shaft");
    assert!(view.player.current_hp > 0);
    let lower_position = view.player.transform.position_m;
    assert!((0.0..=0.1).contains(&lower_position.y_m), "Gear entry is on the authored lower physical floor");
    assert_earned_valve_route_view(&view, "cw_gear_shaft");
    let expected_gear_enemies = BTreeSet::from([
        "cw_gear_shaft_forged_guard_01".to_owned(),
        "cw_gear_shaft_forged_guard_02".to_owned(),
        "cw_gear_shaft_pressure_drone_01".to_owned(),
        "cw_gear_shaft_pressure_drone_02".to_owned(),
        "cw_gear_shaft_pressure_drone_03".to_owned(),
    ]);
    assert_eq!(active_enemy_ids(&view), expected_gear_enemies,
        "Gear projects its own authored enemies, never Boiler Hounds");
    assert!(view.actors.iter().all(|actor| !actor.entity_id.starts_with("cw_boiler_chamber_")),
        "foreign Boiler actors cannot appear in the Gear scene");
    let saved_gear_view = runtime.pause().unwrap();
    runtime.save().unwrap();
    let gear_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&gear_save, "cw_gear_shaft");
    assert!((0.0..=0.1).contains(&gear_save.save.player.position_m.y_m));
    assert_ne!(gear_save.save.checkpoint_id.as_deref(), Some("cw_gear_upper_dock_checkpoint"));
    assert_eq!(gear_save.save.player.current_hp, saved_gear_view.player.current_hp);
    assert_eq!(gear_save.save.player.current_energy, saved_gear_view.player.current_energy);
    let saved_gear_hp = saved_gear_view.player.current_hp;
    let saved_gear_energy = saved_gear_view.player.current_energy;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_eq!(view.scene_id, "cw_gear_shaft");
    assert!(view.player.current_hp > 0);
    assert_eq!(view.player.current_hp, saved_gear_hp);
    assert_eq!(view.player.current_energy, saved_gear_energy);
    assert!((0.0..=0.1).contains(&view.player.transform.position_m.y_m));
    assert_earned_valve_route_view(&view, "cw_gear_shaft");
    assert_eq!(active_enemy_ids(&view), expected_gear_enemies);
    assert!(view.actors.iter().all(|actor| !actor.entity_id.starts_with("cw_boiler_chamber_")));
    let final_view = runtime.pause().unwrap();
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&final_save, "cw_gear_shaft");
    assert!((0.0..=0.1).contains(&final_save.save.player.position_m.y_m));
    assert_eq!(final_save.save.player.current_hp, saved_gear_hp);
    assert_eq!(final_save.save.player.current_energy, saved_gear_energy);
    println!("Lower Gear entry saved and restored at height {:.2}m, {} HP/{} energy; no Gear combat or ascent",
        final_view.player.transform.position_m.y_m, final_view.player.current_hp, final_view.player.current_energy);
    println!("Earned Conveyor → Boiler → lower Gear continuation retained: {}", save_dir.display());
    drop(runtime);
}

// Exact real saved lower Gear entry from the completed Conveyor/Boiler prefix.
const EARNED_CW_LOWER_GEAR_ENTRY_V6: &str = include_str!("../fixtures/earned-cw-lower-gear-entry-v6.json");

#[test]
fn genuine_earned_gear_lift_midride_continue_reaches_upper_checkpoint() {
    NEXT_TIME.fetch_max(600_000, Ordering::Relaxed);
    let save_dir = save_root();
    assert!(!save_dir.exists(), "test save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    std::fs::write(save_dir.join("formal-save-v6.json"), EARNED_CW_LOWER_GEAR_ENTRY_V6.as_bytes()).unwrap();

    let entry_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&entry_save, "cw_gear_shaft");
    assert_eq!(entry_save.save.player.current_hp, 56);
    assert_eq!(entry_save.save.player.current_energy, 28);
    assert_eq!(entry_save.save.checkpoint_id, None);
    assert_eq!(entry_save.save.generic_actors.len(), 5);
    assert!(entry_save.save.generic_actors.iter().all(|actor| actor.hp > 0));
    let entry_support = entry_save.save.moving_support_frame.as_ref().expect("qualified lower Gear save retains its real support frame");
    assert_eq!(entry_support.server_time_ms, entry_save.save.server_time_ms);
    assert!(entry_support.poses.iter().any(|pose| pose.support_id == "cw_gear_main_lift"
        && pose.phase != wuxian_horror_ch1::moving_support::SupportPhase::Stationary));
    let expected_enemies = BTreeSet::from([
        "cw_gear_shaft_forged_guard_01".to_owned(),
        "cw_gear_shaft_forged_guard_02".to_owned(),
        "cw_gear_shaft_pressure_drone_01".to_owned(),
        "cw_gear_shaft_pressure_drone_02".to_owned(),
        "cw_gear_shaft_pressure_drone_03".to_owned(),
    ]);
    assert_eq!(entry_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), expected_enemies);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let mut view = acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
    assert_earned_valve_route_view(&view, "cw_gear_shaft");
    assert_eq!(view.player.current_hp, 56);
    assert_eq!(view.player.current_energy, 28);
    assert_eq!(active_enemy_ids(&view), expected_enemies, "Continue restores the five authored live Gear enemies");

    let old_epoch = view.world_epoch;

    view = clear_live_enemies(&runtime, view);
    assert!(view.player.current_hp > 0, "all five Gear enemies must be defeated alive");
    assert!(active_enemy_ids(&view).is_empty());
    assert_eq!(view.actors.iter().filter(|actor| actor.actor_kind == "enemy").count(), 5);

    assert_earned_valve_route_view(&view, "cw_gear_shaft");

    // Wait for the authored lower hold with neutral inputs; do not change the saved clock or lift pose.
    view = gear_walk(&runtime, view, 12.0, 11.0);
    let lower_wait = std::time::Instant::now();
    let mut lower_hold = false;
    for _ in 0..1200 {
        let lift = view.support_scene.as_ref().unwrap().poses.iter()
            .find(|pose| pose.support_id == "cw_gear_main_lift").unwrap();
        if lift.phase == wuxian_horror_ch1::moving_support::SupportPhase::LowerHold
            && lift.phase_elapsed_ms < 600 {
            lower_hold = true;
            break;
        }
        assert!(lower_wait.elapsed() < std::time::Duration::from_secs(20), "natural lift lower-hold wait exceeded 20 seconds");
        view = input(&runtime, &view, (0.0, 0.0), (1.0, 0.0));
    }
    assert!(lower_hold, "the unchanged moving lift must naturally reach its lower hold");
    view = gear_walk(&runtime, view, 12.0, 8.0);
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_main_lift"),
        "boarding must come from real KCC contact with the authored lift");

    // Observe a real rising rider, then pause and save without advancing the owner clock.
    let rise_wait = std::time::Instant::now();
    let mut reached_midride = false;
    for _ in 0..600 {
        let support = view.support_scene.as_ref().unwrap();
        let lift = support.poses.iter().find(|pose| pose.support_id == "cw_gear_main_lift").unwrap();
        let y = view.player.transform.position_m.y_m;
        if lift.phase == wuxian_horror_ch1::moving_support::SupportPhase::Rising
            && y > 0.7 && y < 1.5
            && support.rider.support_id.as_deref() == Some("cw_gear_main_lift") {
            reached_midride = true;
            break;
        }
        assert!(rise_wait.elapsed() < std::time::Duration::from_secs(10), "rising lift observation exceeded 10 seconds");
        view = input(&runtime, &view, (0.0, 0.0), (1.0, 0.0));
    }
    assert!(reached_midride, "the actual lift must carry the living player through its rising midride band");
    let held = runtime.pause().unwrap();
    assert_eq!(held.player.transform.position_m, view.player.transform.position_m);
    assert_eq!(held.server_time_ms, view.server_time_ms);
    assert_eq!(held.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_main_lift"));
    runtime.save().unwrap();
    let midride_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&midride_save, "cw_gear_shaft");
    assert_eq!(midride_save.save.player.current_hp, held.player.current_hp);
    assert_eq!(midride_save.save.player.current_energy, held.player.current_energy);
    assert_eq!(midride_save.save.player.position_m, held.player.transform.position_m);
    assert_eq!(midride_save.save.server_time_ms, held.server_time_ms);
    assert_eq!(midride_save.save.checkpoint_id, None);
    assert_eq!(midride_save.save.scene_states.len(), 1, "SaveV5 keeps its existing active-scene row contract");
    assert_eq!(midride_save.save.generic_actors.len(), 5);
    assert!(midride_save.save.generic_actors.iter().all(|actor| actor.hp == 0), "all five actual Gear enemies must be saved defeated before riding continues");
    assert_eq!(midride_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), expected_enemies);
    let moving_frame = midride_save.save.moving_support_frame.as_ref().expect("real moving support frame is saved");
    assert_eq!(moving_frame.server_time_ms, held.server_time_ms);
    assert_eq!(moving_frame.poses, held.support_scene.as_ref().unwrap().poses);
    assert!(moving_frame.poses.iter().any(|pose| pose.support_id == "cw_gear_main_lift"
        && pose.phase == wuxian_horror_ch1::moving_support::SupportPhase::Rising));
    let midride_bytes = std::fs::read(save_v6::save_path(&save_dir)).unwrap();
    let midride_hp = held.player.current_hp;
    let midride_energy = held.player.current_energy;
    let midride_position = held.player.transform.position_m;
    let midride_time = held.server_time_ms;
    let midride_poses = held.support_scene.as_ref().unwrap().poses.clone();
    let midride_rider = held.support_scene.as_ref().unwrap().rider.clone();
    drop(runtime);

    // A distinct production owner must restore the exact moving time, player pose and rider before readiness.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, old_epoch, "Continue creates a fresh owner epoch");
    assert!(prepared.entry_token.is_some(), "new owner remains prepared until the normal ready acknowledgement");
    assert_eq!(prepared.player.transform.position_m, midride_position);
    assert_eq!(prepared.player.current_hp, midride_hp);
    assert_eq!(prepared.player.current_energy, midride_energy);
    assert_eq!(prepared.server_time_ms, midride_time);
    assert_eq!(prepared.support_scene.as_ref().unwrap().poses, midride_poses);
    assert_eq!(prepared.support_scene.as_ref().unwrap().rider, midride_rider);
    assert_eq!(std::fs::read(save_v6::save_path(&save_dir)).unwrap(), midride_bytes,
        "Continue preparation cannot rewrite the saved state");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "new owner preparation cannot replay Gear deaths");
    let mut view = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&view, "cw_gear_shaft");
    assert!(active_enemy_ids(&view).is_empty());
    assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "readiness cannot replay Gear deaths");

    let upper_wait = std::time::Instant::now();
    let mut reached_upper_hold = false;
    for _ in 0..600 {
        let lift = view.support_scene.as_ref().unwrap().poses.iter()
            .find(|pose| pose.support_id == "cw_gear_main_lift").unwrap();
        if lift.phase == wuxian_horror_ch1::moving_support::SupportPhase::UpperHold
            && (view.player.transform.position_m.y_m - 2.0).abs() < 0.001 {
            reached_upper_hold = true;
            break;
        }
        assert!(upper_wait.elapsed() < std::time::Duration::from_secs(10), "upper-hold wait exceeded 10 seconds");
        view = input(&runtime, &view, (0.0, 0.0), (1.0, 0.0));
    }
    assert!(reached_upper_hold, "Continue must preserve a naturally advancing rider until the upper hold");
    view = gear_walk(&runtime, view, 16.0, 8.0);
    assert!(view.player.current_hp > 0);
    assert!((view.player.transform.position_m.y_m - 2.0).abs() < 0.001);
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    assert!(view.actors.iter().filter(|actor| actor.actor_kind == "enemy").all(|actor| !actor.active));

    let checkpoint = scene_route_commands::checkpoint(&runtime, "cw_gear_upper_dock_checkpoint",
        "genuine-earned-gear-upper-dock", view.world_epoch).unwrap();
    assert!(checkpoint.applied, "the authored upper-dock checkpoint command must apply");
    view = runtime.snapshot().unwrap();
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    runtime.save().unwrap();
    let upper_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&upper_save, "cw_gear_shaft");
    assert_eq!(upper_save.save.checkpoint_id.as_deref(), Some("cw_gear_upper_dock_checkpoint"));
    assert_eq!(upper_save.save.player.position_m.y_m, 2.0);
    assert_eq!(upper_save.save.generic_actors.len(), 5);
    assert!(upper_save.save.generic_actors.iter().all(|actor| actor.hp == 0), "saved Gear enemies remain defeated");
    assert_eq!(upper_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), expected_enemies);
    let upper_bytes = std::fs::read(save_v6::save_path(&save_dir)).unwrap();
    let upper_hp = view.player.current_hp;
    let upper_energy = view.player.current_energy;
    let upper_position = view.player.transform.position_m;
    let upper_time = view.server_time_ms;
    let upper_epoch = view.world_epoch;
    drop(runtime);

    // A second fresh production owner must retain the checkpoint and dead enemies without replay.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, upper_epoch);
    assert!(prepared.entry_token.is_some());
    assert_eq!(prepared.player.transform.position_m, upper_position);
    assert_eq!(prepared.player.current_hp, upper_hp);
    assert_eq!(prepared.player.current_energy, upper_energy);
    assert_eq!(prepared.server_time_ms, upper_time);
    assert_eq!(prepared.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    assert_eq!(std::fs::read(save_v6::save_path(&save_dir)).unwrap(), upper_bytes);
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "checkpoint Continue cannot replay Gear deaths");
    let view = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&view, "cw_gear_shaft");
    assert_eq!(view.player.transform.position_m, upper_position);
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    assert!(active_enemy_ids(&view).is_empty());
    assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "checkpoint readiness cannot replay Gear deaths");
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(final_save.save.checkpoint_id.as_deref(), Some("cw_gear_upper_dock_checkpoint"));
    assert_eq!(final_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), expected_enemies);
    assert!(final_save.save.generic_actors.iter().all(|actor| actor.hp == 0));
    assert_earned_valve_route_save(&final_save, "cw_gear_shaft");
    println!("Earned Gear lift midride Continue reached and saved the real upper-dock checkpoint at {} HP/{} energy; stopped before the upper gap", view.player.current_hp, view.player.current_energy);
    drop(runtime);
}
