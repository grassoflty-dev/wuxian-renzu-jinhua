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

// Exact qualified Continue save at the earned Gear upper dock, before the free gap traversal.
const EARNED_CW_UPPER_DOCK_ENTRY_V6: &str = include_str!("../fixtures/earned-cw-upper-dock-v6.json");

#[test]
fn genuine_earned_upper_gap_reaches_durable_furnace_entry() {
    let save_dir = save_root();
    assert!(!save_dir.exists(), "task save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    std::fs::write(save_dir.join("formal-save-v6.json"), EARNED_CW_UPPER_DOCK_ENTRY_V6.as_bytes()).unwrap();

    let gear_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&gear_save, "cw_gear_shaft");
    assert_eq!(gear_save.save.checkpoint_id.as_deref(), Some("cw_gear_upper_dock_checkpoint"));
    assert_eq!(gear_save.save.player.current_hp, 56);
    assert_eq!(gear_save.save.player.current_energy, 28);
    assert_eq!(gear_save.save.scene_states.len(), 1, "SaveV5 retains its active-scene row contract");
    assert_eq!(gear_save.save.generic_actors.len(), 5);
    assert!(gear_save.save.generic_actors.iter().all(|actor| actor.hp == 0),
        "the earned upper-dock save retains all five defeated Gear actors");
    let gear_actor_ids = BTreeSet::from([
        "cw_gear_shaft_forged_guard_01".to_owned(),
        "cw_gear_shaft_forged_guard_02".to_owned(),
        "cw_gear_shaft_pressure_drone_01".to_owned(),
        "cw_gear_shaft_pressure_drone_02".to_owned(),
        "cw_gear_shaft_pressure_drone_03".to_owned(),
    ]);
    assert_eq!(gear_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), gear_actor_ids);
    let gear_frame = gear_save.save.moving_support_frame.as_ref().expect("earned Gear save retains its actual lift frame");
    assert_eq!(gear_frame.server_time_ms, gear_save.save.server_time_ms);
    assert!(gear_frame.poses.iter().any(|pose| pose.support_id == "cw_gear_upper_dock"
        && pose.height_m == 2.0), "the saved upper dock remains at its authored height");
    NEXT_TIME.fetch_max(gear_save.save.last_client_time_ms.saturating_add(1), Ordering::Relaxed);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let gear_save_bytes = std::fs::read(save_v6::save_path(&save_dir)).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_eq!(prepared.world_id, "clockworks");
    assert_eq!(prepared.scene_id, "cw_gear_shaft");
    assert_eq!(prepared.entry_token.as_ref().map(|token| token.scene_id.as_str()), Some("cw_gear_shaft"));
    assert_eq!(prepared.player.transform.position_m, gear_save.save.player.position_m);
    assert_eq!(prepared.server_time_ms, gear_save.save.server_time_ms);
    assert_eq!(prepared.player.current_hp, 56);
    assert_eq!(prepared.player.current_energy, 28);
    assert_eq!(prepared.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    assert!(active_enemy_ids(&prepared).is_empty());
    assert_earned_valve_route_view(&prepared, "cw_gear_shaft");
    assert_eq!(std::fs::read(save_v6::save_path(&save_dir)).unwrap(), gear_save_bytes,
        "Continue preparation must not rewrite the pinned earned input");
    let mut view = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&view, "cw_gear_shaft");
    assert!(active_enemy_ids(&view).is_empty());

    let authored_gear = compiled_route_scene("cw_gear_shaft");
    let marker = authored_gear["logic"]["traversal"].as_array().unwrap().iter()
        .find(|marker| marker["id"] == "cw_gear_upper_gap")
        .expect("current authored upper-gap marker exists");
    assert!(marker["requiredCapabilities"].as_array().unwrap().is_empty(),
        "the authored first-clear upper gap is not gated by AirStep");
    assert_eq!(marker["from"], serde_json::json!([17.8, 2.0, 8.0]));
    assert_eq!(marker["to"], serde_json::json!([20.2, 2.0, 8.0]));
    assert!(!view.capabilities.items.iter().any(|item| item.capability_id == CAP_AIR_STEP && item.granted),
        "the earned valve-only route does not grant AirStep");

    let from = marker["from"].as_array().unwrap();
    view = gear_walk(&runtime, view, from[0].as_f64().unwrap() as f32, from[2].as_f64().unwrap() as f32);
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    assert!((view.player.transform.position_m.y_m - 2.0).abs() < 0.1,
        "the genuine action begins on the earned upper dock");
    view = input(&runtime, &view, (0.0, 0.0), (1.0, 0.0));
    let before_action = runtime.snapshot().unwrap();
    assert_eq!(before_action.scene_id, "cw_gear_shaft");
    assert!(before_action.player.current_hp > 0);
    assert_eq!(before_action.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    let event_cursor = runtime.presentation_events_since(before_action.world_epoch, 0).unwrap()
        .iter().map(|event| event.event_id).max().unwrap_or(0);
    let start_position = before_action.player.transform.position_m;
    let start_time_ms = before_action.server_time_ms;
    let epoch = before_action.world_epoch;
    let wall_start = std::time::Instant::now();
    let action_view = runtime.submit_action(ActionCommandRequest {
        protocol_version: 2,
        world_epoch: epoch,
        request_id: NEXT_COMBAT_REQUEST.fetch_add(1, Ordering::Relaxed),
        client_time_ms: NEXT_TIME.fetch_add(17, Ordering::Relaxed),
        kind: ActionKind::ContextTraversal,
    }).unwrap();
    assert_eq!(action_view.world_epoch, epoch);
    let mut observed = action_view;
    let mut started_event = None;
    let mut completed_event = None;
    let mut blocked_event = None;
    for _ in 0..300 {
        observed = runtime.snapshot().unwrap();
        assert_eq!(observed.world_epoch, epoch, "the authored gap traversal stays in Gear");
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
        let elapsed_owner_ms = observed.server_time_ms.saturating_sub(start_time_ms);
        if elapsed_owner_ms >= 3_000 || wall_start.elapsed() >= std::time::Duration::from_secs(3) { break; }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(started_event.is_some(), "one new ContextTraversalStarted cue must follow the captured cursor");
    assert!(completed_event.is_some(), "the single authored gap traversal must complete within three seconds");
    assert!(blocked_event.is_none(), "the authored capability-free gap must not be blocked");
    assert!(started_event.unwrap() < completed_event.unwrap(), "traversal start precedes completion");
    assert!(observed.server_time_ms.saturating_sub(start_time_ms) <= 3_000,
        "the traversal must complete within three seconds of owner time");
    assert_eq!(observed.scene_id, "cw_gear_shaft");
    assert!(observed.player.current_hp > 0, "the living player must reach the Furnace landing");
    let end_position = observed.player.transform.position_m;
    assert!(end_position.x_m - start_position.x_m >= 1.0,
        "completed gap traversal must demonstrate at least 1m forward displacement: {start_position:?} -> {end_position:?}");
    assert!((1.9..=2.1).contains(&end_position.y_m), "the traversal must land in the authored 2m height band");
    let landing = observed.support_scene.as_ref().expect("landing support view");
    assert_eq!(landing.rider.support_id.as_deref(), Some("cw_gear_furnace_landing"),
        "the player must be physically supported by the authored Furnace landing");
    assert_eq!(landing.rider.mode, wuxian_horror_ch1::moving_support::SupportRiderMode::Surface);
    assert!(landing.poses.iter().any(|pose| pose.support_id == "cw_gear_furnace_landing"
        && pose.height_m == 2.0), "the landing support remains at the authored height");
    println!("Upper gap: observed Started/Completed cues, {:.2}m forward displacement, {:.2}m landing height, actual landing support, {} HP",
        end_position.x_m - start_position.x_m, end_position.y_m, observed.player.current_hp);

    view = walk_to_compiled_target(&runtime, observed, "cw_gear_shaft", "transitions", "cw_gear_to_furnace_heart");
    let transition_scene = compiled_route_scene("cw_gear_shaft");
    let transition = transition_scene["transitions"].as_array().unwrap().iter()
        .find(|transition| transition["id"] == "cw_gear_to_furnace_heart").unwrap();
    assert_eq!(transition["toSceneId"], "cw_furnace_heart");
    assert_eq!(transition["spawnId"], "cw_furnace_heart_spawn");
    assert!(transition["heightRangeM"].as_array().unwrap().iter()
        .all(|height| (1.9..=2.1).contains(&(height.as_f64().unwrap() as f32))));
    assert_eq!(view.support_scene.as_ref().unwrap().rider.support_id.as_deref(), Some("cw_gear_furnace_landing"));
    assert!((1.9..=2.1).contains(&view.player.transform.position_m.y_m),
        "the authored height-gated Furnace route is approached from its landing height");
    view = apply_transition(&runtime, "cw_gear_to_furnace_heart", "genuine-upper-gap-to-furnace-entry", view.world_epoch);
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_furnace_heart");
    assert!(view.player.current_hp > 0);
    assert!((0.0..=0.1).contains(&view.player.transform.position_m.y_m), "Furnace entry uses its authored lower floor");
    assert!((view.player.transform.position_m.x_m - 3.0).abs() <= 0.1);
    assert!((view.player.transform.position_m.z_m - 8.0).abs() <= 0.1);
    assert_earned_valve_route_view(&view, "cw_furnace_heart");
    let expected_furnace_enemies = BTreeSet::from([
        "cw_furnace_heart_forged_guard_01".to_owned(),
        "cw_furnace_heart_forged_guard_02".to_owned(),
        "cw_furnace_heart_furnace_hound_01".to_owned(),
        "cw_furnace_heart_furnace_hound_02".to_owned(),
        "cw_furnace_heart_furnace_hound_03".to_owned(),
    ]);
    assert_eq!(active_enemy_ids(&view), expected_furnace_enemies,
        "actual Furnace entry loads its five authored live enemies");

    // Pause and save the genuine entry immediately; no Furnace combat or checkpoint action follows.
    let entry_view = runtime.pause().unwrap();
    assert_earned_valve_route_view(&entry_view, "cw_furnace_heart");
    assert_eq!(active_enemy_ids(&entry_view), expected_furnace_enemies);
    runtime.save().unwrap();
    let furnace_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&furnace_save, "cw_furnace_heart");
    assert_eq!(furnace_save.save.server_time_ms, entry_view.server_time_ms);
    assert_eq!(furnace_save.save.player.position_m, entry_view.player.transform.position_m);
    assert_eq!(furnace_save.save.player.current_hp, entry_view.player.current_hp);
    assert_eq!(furnace_save.save.player.current_energy, entry_view.player.current_energy);
    assert_eq!(furnace_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), expected_furnace_enemies);
    assert!(furnace_save.save.generic_actors.iter().all(|actor| actor.hp > 0),
        "entry save preserves all five live Furnace actor rows");
    let entry_checkpoint = furnace_save.save.checkpoint_id.clone();
    let entry_save_bytes = std::fs::read(save_v6::save_path(&save_dir)).unwrap();
    let entry_position = entry_view.player.transform.position_m;
    let entry_hp = entry_view.player.current_hp;
    let entry_energy = entry_view.player.current_energy;
    let entry_time_ms = entry_view.server_time_ms;
    let entry_epoch = entry_view.world_epoch;
    println!("Paused Furnace entry: {} HP/{} energy, {} live actors, destination checkpoint {:?}",
        entry_hp, entry_energy, expected_furnace_enemies.len(), entry_checkpoint);
    drop(runtime);

    // A fresh production owner must prepare the same saved entry before the normal ready handshake.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, entry_epoch, "Continue creates a coherent new owner epoch");
    let token = prepared.entry_token.as_ref().expect("fresh Furnace owner remains prepared before readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_furnace_heart");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.world_id, "clockworks");
    assert_eq!(prepared.scene_id, "cw_furnace_heart");
    assert_eq!(prepared.player.transform.position_m, entry_position);
    assert_eq!(prepared.player.current_hp, entry_hp);
    assert_eq!(prepared.player.current_energy, entry_energy);
    assert_eq!(prepared.server_time_ms, entry_time_ms);
    assert_eq!(active_enemy_ids(&prepared), expected_furnace_enemies);
    assert_eq!(std::fs::read(save_v6::save_path(&save_dir)).unwrap(), entry_save_bytes,
        "Continue preparation cannot rewrite the saved Furnace entry");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "fresh-owner preparation cannot replay prior Gear deaths");
    let continued = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&continued, "cw_furnace_heart");
    assert!(continued.player.current_hp > 0);
    assert_eq!(continued.player.transform.position_m, entry_position);
    assert_eq!(continued.player.current_hp, entry_hp);
    assert_eq!(continued.player.current_energy, entry_energy);
    assert_eq!(continued.server_time_ms, entry_time_ms);
    assert_eq!(active_enemy_ids(&continued), expected_furnace_enemies);
    assert!(runtime.presentation_events_since(continued.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "readiness cannot replay prior Gear deaths");
    let final_entry_view = runtime.pause().unwrap();
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&final_save, "cw_furnace_heart");
    assert_eq!(final_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(), expected_furnace_enemies);
    assert!(final_save.save.generic_actors.iter().all(|actor| actor.hp > 0));
    assert_eq!(final_save.save.player.current_hp, final_entry_view.player.current_hp);
    assert_eq!(final_save.save.player.current_energy, final_entry_view.player.current_energy);
    println!("Durable Furnace entry survived new-owner Continue at {} HP/{} energy; stopped before combat",
        final_entry_view.player.current_hp, final_entry_view.player.current_energy);
    drop(runtime);
}

// Exact qualified SaveV6 for the earned Furnace entry: retained from the upper-gap successor.
const EARNED_CW_FURNACE_ENCOUNTER_ENTRY_V6: &str =
    include_str!("../fixtures/earned-cw-furnace-entry-v6.json");

#[test]
fn genuine_earned_furnace_encounter_reaches_durable_arena_entry() {
    let save_dir = save_root();
    assert!(!save_dir.exists(), "task save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_dir.join("formal-save-v6.json");
    std::fs::write(&save_path, EARNED_CW_FURNACE_ENCOUNTER_ENTRY_V6.as_bytes()).unwrap();

    let furnace_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&furnace_save, "cw_furnace_heart");
    assert_eq!(furnace_save.save.checkpoint_id.as_deref(), None);
    assert_eq!(furnace_save.save.player.current_hp, 56);
    assert_eq!(furnace_save.save.player.current_energy, 28);
    assert_eq!(furnace_save.save.player.position_m.x_m, 3.0);
    assert_eq!(furnace_save.save.player.position_m.y_m, 0.0);
    assert_eq!(furnace_save.save.player.position_m.z_m, 8.0);
    assert_eq!(furnace_save.save.server_time_ms, 608_549);
    assert_eq!(furnace_save.save.last_client_time_ms, 0);

    let expected_furnace_actors = BTreeMap::from([
        ("cw_furnace_heart_forged_guard_01".to_owned(), "enemy.clockworks.forged_guard".to_owned()),
        ("cw_furnace_heart_forged_guard_02".to_owned(), "enemy.clockworks.forged_guard".to_owned()),
        ("cw_furnace_heart_furnace_hound_01".to_owned(), "enemy.clockworks.furnace_hound".to_owned()),
        ("cw_furnace_heart_furnace_hound_02".to_owned(), "enemy.clockworks.furnace_hound".to_owned()),
        ("cw_furnace_heart_furnace_hound_03".to_owned(), "enemy.clockworks.furnace_hound".to_owned()),
    ]);
    assert_eq!(furnace_save.save.generic_actors.len(), expected_furnace_actors.len());
    assert_eq!(
        furnace_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(),
        expected_furnace_actors.keys().cloned().collect::<BTreeSet<_>>(),
    );
    assert!(furnace_save.save.generic_actors.iter().all(|actor| {
        actor.hp > 0 && expected_furnace_actors.get(&actor.entity_id) == Some(&actor.entity_type)
    }), "the exact five typed Furnace enemies are alive in the qualified input");
    NEXT_TIME.fetch_max(furnace_save.save.last_client_time_ms.saturating_add(1), Ordering::Relaxed);

    let input_bytes = std::fs::read(&save_path).unwrap();
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_eq!(prepared.world_id, "clockworks");
    assert_eq!(prepared.scene_id, "cw_furnace_heart");
    let token = prepared.entry_token.as_ref().expect("fresh Furnace owner remains prepared before readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_furnace_heart");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.player.transform.position_m, furnace_save.save.player.position_m);
    assert_eq!(prepared.server_time_ms, furnace_save.save.server_time_ms);
    assert_eq!(prepared.player.current_hp, 56);
    assert_eq!(prepared.player.current_energy, 28);
    assert_eq!(active_enemy_ids(&prepared), expected_furnace_actors.keys().cloned().collect());
    assert_earned_valve_route_view(&prepared, "cw_furnace_heart");
    assert_eq!(std::fs::read(&save_path).unwrap(), input_bytes,
        "Continue preparation must not rewrite the qualified Furnace entry");
    let prepared_epoch = prepared.world_epoch;
    assert!(runtime.presentation_events_since(prepared_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "Continue cannot replay prior enemy deaths");

    let ready = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&ready, "cw_furnace_heart");
    assert!(ready.player.current_hp > 0);
    assert_eq!(active_enemy_ids(&ready), expected_furnace_actors.keys().cloned().collect());
    assert!(runtime.presentation_events_since(ready.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "readiness cannot replay prior enemy deaths");

    let cleared = clear_live_enemies(&runtime, ready);
    assert_earned_valve_route_view(&cleared, "cw_furnace_heart");
    assert!(cleared.player.current_hp > 0);
    assert!(active_enemy_ids(&cleared).is_empty(), "all five Furnace enemies must be defeated through public combat");
    let cleared_entry = runtime.pause().unwrap();
    assert_earned_valve_route_view(&cleared_entry, "cw_furnace_heart");
    assert!(active_enemy_ids(&cleared_entry).is_empty());
    runtime.save().unwrap();
    let cleared_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&cleared_save, "cw_furnace_heart");
    assert_eq!(cleared_save.save.checkpoint_id.as_deref(), None,
        "the encounter does not create a checkpoint");
    assert_eq!(cleared_save.save.server_time_ms, cleared_entry.server_time_ms);
    assert_eq!(cleared_save.save.player.position_m, cleared_entry.player.transform.position_m);
    assert_eq!(cleared_save.save.player.current_hp, cleared_entry.player.current_hp);
    assert_eq!(cleared_save.save.player.current_energy, cleared_entry.player.current_energy);
    assert_eq!(
        cleared_save.save.generic_actors.iter().map(|actor| actor.entity_id.clone()).collect::<BTreeSet<_>>(),
        expected_furnace_actors.keys().cloned().collect::<BTreeSet<_>>(),
    );
    assert!(cleared_save.save.generic_actors.iter().all(|actor| {
        actor.hp == 0 && expected_furnace_actors.get(&actor.entity_id) == Some(&actor.entity_type)
    }), "the durable cleared entry retains exactly the five defeated typed Furnace actors");
    let cleared_bytes = std::fs::read(&save_path).unwrap();
    let cleared_position = cleared_entry.player.transform.position_m;
    let cleared_hp = cleared_entry.player.current_hp;
    let cleared_energy = cleared_entry.player.current_energy;
    let cleared_time = cleared_entry.server_time_ms;
    let cleared_epoch = cleared_entry.world_epoch;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, cleared_epoch, "fresh Furnace Continue creates a new owner epoch");
    let token = prepared.entry_token.as_ref().expect("fresh owner waits for its exact readiness token");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_furnace_heart");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.scene_id, "cw_furnace_heart");
    assert_eq!(prepared.player.transform.position_m, cleared_position);
    assert_eq!(prepared.player.current_hp, cleared_hp);
    assert_eq!(prepared.player.current_energy, cleared_energy);
    assert_eq!(prepared.server_time_ms, cleared_time);
    assert!(active_enemy_ids(&prepared).is_empty(), "fresh Continue keeps the Furnace encounter cleared");
    assert_earned_valve_route_view(&prepared, "cw_furnace_heart");
    assert_eq!(std::fs::read(&save_path).unwrap(), cleared_bytes,
        "fresh Continue preparation cannot rewrite the cleared Furnace save");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "fresh owner preparation cannot replay Furnace deaths");
    let ready = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&ready, "cw_furnace_heart");
    assert_eq!(ready.player.transform.position_m, cleared_position);
    assert_eq!(ready.player.current_hp, cleared_hp);
    assert_eq!(ready.player.current_energy, cleared_energy);
    assert_eq!(ready.server_time_ms, cleared_time);
    assert!(active_enemy_ids(&ready).is_empty());
    assert!(runtime.presentation_events_since(ready.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "readiness cannot replay Furnace deaths");

    let furnace_scene = compiled_route_scene("cw_furnace_heart");
    let furnace_transition = furnace_scene["transitions"].as_array().unwrap().iter()
        .find(|transition| transition["id"] == "cw_furnace_heart_to_arena")
        .expect("the authored Furnace-to-arena transition exists");
    assert_eq!(furnace_transition["toSceneId"], "cw_forged_guard_arena");
    assert_eq!(furnace_transition["spawnId"], "cw_forged_guard_arena_spawn");
    let arena_scene = compiled_route_scene("cw_forged_guard_arena");
    let arena_player_spawn = arena_scene["spawns"].as_array().unwrap().iter()
        .find(|spawn| spawn["id"] == furnace_transition["spawnId"])
        .expect("the transition destination has its registered player spawn");
    assert_eq!(arena_player_spawn["kind"], "player");
    assert_eq!(arena_player_spawn["position"], serde_json::json!([3.0, 0.0, 8.0]));
    let elite_spawn = arena_scene["spawns"].as_array().unwrap().iter()
        .find(|spawn| spawn["id"] == "cw_forged_guard_elite")
        .expect("the authored arena has its typed elite actor spawn");
    assert_eq!(elite_spawn["kind"], "enemy");
    assert_eq!(elite_spawn["entityType"], "enemy.clockworks.forged_guard_elite");

    let approach = walk_to_compiled_target(
        &runtime,
        ready,
        "cw_furnace_heart",
        "transitions",
        "cw_furnace_heart_to_arena",
    );
    let affordance = approach.interactables.iter()
        .find(|item| item.entity_id == "cw_furnace_heart_to_arena")
        .expect("the authored Furnace-to-arena transition is projected at its polygon");
    assert_eq!(affordance.kind, "scene_transition");
    assert!(affordance.active, "the earned Furnace encounter route reaches the arena transition");
    let arena_entry = apply_transition(
        &runtime,
        "cw_furnace_heart_to_arena",
        "genuine-furnace-to-arena-entry",
        approach.world_epoch,
    );
    assert_eq!(arena_entry.world_id, "clockworks");
    assert_eq!(arena_entry.scene_id, "cw_forged_guard_arena");
    assert_earned_valve_route_view(&arena_entry, "cw_forged_guard_arena");
    assert!(arena_entry.player.current_hp > 0);
    let player_spawn_position = arena_player_spawn["position"].as_array().unwrap();
    assert!((arena_entry.player.transform.position_m.x_m - player_spawn_position[0].as_f64().unwrap() as f32).abs() <= 0.1);
    assert!((arena_entry.player.transform.position_m.y_m - player_spawn_position[1].as_f64().unwrap() as f32).abs() <= 0.1);
    assert!((arena_entry.player.transform.position_m.z_m - player_spawn_position[2].as_f64().unwrap() as f32).abs() <= 0.1);
    let expected_elite_ids = BTreeSet::from(["cw_forged_guard_elite".to_owned()]);
    assert_eq!(active_enemy_ids(&arena_entry), expected_elite_ids,
        "arena entry must retain the living elite without starting its encounter");
    let elite_view = arena_entry.actors.iter().find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("the canonical elite actor is projected at arena entry");
    assert_eq!(elite_view.actor_kind, "enemy");
    assert!(elite_view.active, "the canonical elite actor is alive at arena entry");
    assert!(runtime.presentation_events_since(arena_entry.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "arena entry records no enemy defeat");

    let arena_entry = runtime.pause().unwrap();
    assert_earned_valve_route_view(&arena_entry, "cw_forged_guard_arena");
    assert_eq!(active_enemy_ids(&arena_entry), expected_elite_ids);
    runtime.save().unwrap();
    let arena_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&arena_save, "cw_forged_guard_arena");
    assert_eq!(arena_save.save.checkpoint_id.as_deref(), None,
        "the arena entry does not invent a checkpoint");
    assert_eq!(arena_save.save.server_time_ms, arena_entry.server_time_ms);
    assert_eq!(arena_save.save.player.position_m, arena_entry.player.transform.position_m);
    assert_eq!(arena_save.save.player.current_hp, arena_entry.player.current_hp);
    assert_eq!(arena_save.save.player.current_energy, arena_entry.player.current_energy);
    let elite_row = arena_save.save.generic_actors.iter()
        .find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("the arena SaveV6 retains the canonical elite row");
    assert_eq!(elite_row.entity_type, "enemy.clockworks.forged_guard_elite");
    assert!(elite_row.hp > 0, "the elite remains alive in the durable arena-entry save");
    let arena_bytes = std::fs::read(&save_path).unwrap();
    let arena_position = arena_entry.player.transform.position_m;
    let arena_hp = arena_entry.player.current_hp;
    let arena_energy = arena_entry.player.current_energy;
    let arena_time = arena_entry.server_time_ms;
    let arena_epoch = arena_entry.world_epoch;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, arena_epoch, "fresh arena Continue creates a new owner epoch");
    let token = prepared.entry_token.as_ref().expect("fresh arena owner waits for its exact readiness token");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_forged_guard_arena");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.scene_id, "cw_forged_guard_arena");
    assert_eq!(prepared.player.transform.position_m, arena_position);
    assert_eq!(prepared.player.current_hp, arena_hp);
    assert_eq!(prepared.player.current_energy, arena_energy);
    assert_eq!(prepared.server_time_ms, arena_time);
    assert_eq!(active_enemy_ids(&prepared), expected_elite_ids);
    let elite_view = prepared.actors.iter().find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("fresh Continue retains the canonical elite actor");
    assert!(elite_view.active);
    assert_earned_valve_route_view(&prepared, "cw_forged_guard_arena");
    assert_eq!(std::fs::read(&save_path).unwrap(), arena_bytes,
        "Continue preparation must not rewrite the durable arena entry");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "arena Continue cannot replay an enemy defeat");
    let continued = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&continued, "cw_forged_guard_arena");
    assert!(continued.player.current_hp > 0);
    assert_eq!(continued.player.transform.position_m, arena_position);
    assert_eq!(continued.player.current_hp, arena_hp);
    assert_eq!(continued.player.current_energy, arena_energy);
    assert_eq!(continued.server_time_ms, arena_time);
    assert_eq!(active_enemy_ids(&continued), expected_elite_ids);
    let elite_view = continued.actors.iter().find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("readiness retains the canonical elite actor");
    assert!(elite_view.active);
    assert!(runtime.presentation_events_since(continued.world_epoch, 0).unwrap().iter()
        .all(|event| event.kind != "EnemyDeath"), "readiness cannot replay an enemy defeat");

    let final_entry = runtime.pause().unwrap();
    assert_earned_valve_route_view(&final_entry, "cw_forged_guard_arena");
    assert_eq!(active_enemy_ids(&final_entry), expected_elite_ids);
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&final_save, "cw_forged_guard_arena");
    assert_eq!(final_save.save.checkpoint_id.as_deref(), None);
    let final_elite = final_save.save.generic_actors.iter()
        .find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("the final owned SaveV6 retains the canonical elite row");
    assert_eq!(final_elite.entity_type, "enemy.clockworks.forged_guard_elite");
    assert!(final_elite.hp > 0, "the elite remains alive after the fresh-owner Continue");
    let final_bytes = std::fs::read(&save_path).unwrap();
    println!(
        "Earned Furnace encounter cleared and saved; fresh Continue reached durable arena entry with elite {} ({}) at {} HP; player {} HP/{} energy, time {}, checkpoint {:?}; final save {} bytes, path {}",
        final_elite.entity_id,
        final_elite.entity_type,
        final_elite.hp,
        final_save.save.player.current_hp,
        final_save.save.player.current_energy,
        final_save.save.server_time_ms,
        final_save.save.checkpoint_id,
        final_bytes.len(),
        save_path.display(),
    );
    drop(runtime);
}

// Exact qualified SaveV6 for the living elite arena entry; preserve its actual resources and world history.
const EARNED_CW_ARENA_ENTRY_V6: &str =
    include_str!("../fixtures/earned-cw-arena-entry-v6.json");

#[test]
fn genuine_earned_elite_defeat_reaches_durable_regulator_entry() {
    let save_dir = save_root();
    assert!(!save_dir.exists(), "task save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_dir.join("formal-save-v6.json");
    std::fs::write(&save_path, EARNED_CW_ARENA_ENTRY_V6.as_bytes()).unwrap();

    let arena_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&arena_save, "cw_forged_guard_arena");
    assert_eq!(arena_save.save.checkpoint_id.as_deref(), None);
    assert_eq!(arena_save.save.player.current_hp, 46);
    assert_eq!(arena_save.save.player.current_energy, 28);
    assert_eq!(arena_save.save.player.position_m.x_m, 3.0);
    assert_eq!(arena_save.save.player.position_m.y_m, 0.0);
    assert_eq!(arena_save.save.player.position_m.z_m, 8.0);
    assert_eq!(arena_save.save.server_time_ms, 640_611);
    assert_eq!(arena_save.save.last_client_time_ms, 0);
    assert_eq!(arena_save.save.generic_actors.len(), 1);
    let elite_row = &arena_save.save.generic_actors[0];
    assert_eq!(elite_row.entity_id, "cw_forged_guard_elite");
    assert_eq!(elite_row.entity_type, "enemy.clockworks.forged_guard_elite");
    assert_eq!(elite_row.hp, 120);
    assert!(!arena_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(!arena_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!arena_save.world_persistent_v1.clockworks.core_console_confirmed);
    let earned_effect_sources = arena_save.effect_sources.clone();
    let earned_grants = arena_save.save.capabilities.grants.clone();
    let arena_input_bytes = std::fs::read(&save_path).unwrap();
    NEXT_TIME.fetch_max(arena_save.save.last_client_time_ms.saturating_add(1), Ordering::Relaxed);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_eq!(prepared.world_id, "clockworks");
    assert_eq!(prepared.scene_id, "cw_forged_guard_arena");
    let token = prepared.entry_token.as_ref().expect("arena Continue waits for its normal readiness token");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_forged_guard_arena");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.player.transform.position_m, arena_save.save.player.position_m);
    assert_eq!(prepared.server_time_ms, arena_save.save.server_time_ms);
    assert_eq!(prepared.player.current_hp, 46);
    assert_eq!(prepared.player.current_energy, 28);
    let expected_elite_ids = BTreeSet::from(["cw_forged_guard_elite".to_owned()]);
    assert_eq!(active_enemy_ids(&prepared), expected_elite_ids);
    assert_earned_valve_route_view(&prepared, "cw_forged_guard_arena");
    let live_elite = prepared.actors.iter()
        .find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("the admitted canonical elite is projected at arena entry");
    assert_eq!(live_elite.actor_kind, "enemy");
    assert!(live_elite.active);
    let arena_epoch = prepared.world_epoch;
    assert_eq!(std::fs::read(&save_path).unwrap(), arena_input_bytes,
        "Continue preparation cannot rewrite the qualified arena input");
    assert!(runtime.presentation_events_since(arena_epoch, 0).unwrap().iter()
        .all(|event| !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath")),
        "arena Continue cannot replay a prior elite defeat");

    let ready = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&ready, "cw_forged_guard_arena");
    assert_eq!(ready.player.transform.position_m, arena_save.save.player.position_m);
    assert_eq!(ready.player.current_hp, 46);
    assert_eq!(ready.player.current_energy, 28);
    assert_eq!(ready.server_time_ms, arena_save.save.server_time_ms);
    assert_eq!(active_enemy_ids(&ready), expected_elite_ids);
    let event_cursor = runtime.presentation_events_since(ready.world_epoch, 0).unwrap().iter()
        .map(|event| event.event_id).max().unwrap_or(0);

    // Earn the one canonical elite defeat using the unchanged public combat driver.
    let defeated = clear_live_enemies(&runtime, ready);
    assert_eq!(defeated.world_id, "clockworks");
    assert_eq!(defeated.scene_id, "cw_forged_guard_arena");
    assert!(defeated.player.current_hp > 0, "the elite defeat must be earned alive");
    assert!(active_enemy_ids(&defeated).is_empty());
    assert_earned_valve_route_view(&defeated, "cw_forged_guard_arena");
    let defeat_events = runtime.presentation_events_since(defeated.world_epoch, event_cursor).unwrap();
    let elite_deaths = defeat_events.iter().filter(|event| {
        event.world_epoch == defeated.world_epoch
            && event.kind == "ForgedGuardEliteDeath"
    }).collect::<Vec<_>>();
    assert_eq!(elite_deaths.len(), 1, "lethal public combat emits one post-cursor elite first-kill event");
    println!(
        "Elite defeated by public combat: {} hit points remaining; observed one new ForgedGuardEliteDeath",
        defeated.player.current_hp
    );

    let defeated_view = runtime.pause().unwrap();
    assert_eq!(defeated_view.scene_id, "cw_forged_guard_arena");
    assert!(defeated_view.player.current_hp > 0);
    assert!(active_enemy_ids(&defeated_view).is_empty());
    runtime.save().unwrap();
    let defeated_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&defeated_save, "cw_forged_guard_arena");
    assert_eq!(defeated_save.save.checkpoint_id, arena_save.save.checkpoint_id);
    assert_eq!(defeated_save.save.server_time_ms, defeated_view.server_time_ms);
    assert_eq!(defeated_save.save.player.position_m, defeated_view.player.transform.position_m);
    assert_eq!(defeated_save.save.player.current_hp, defeated_view.player.current_hp);
    assert_eq!(defeated_save.save.player.current_energy, defeated_view.player.current_energy);
    assert!(defeated_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(!defeated_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!defeated_save.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(defeated_save.effect_sources, earned_effect_sources);
    assert_eq!(defeated_save.save.capabilities.grants, earned_grants);
    assert_eq!(defeated_save.save.generic_actors.len(), 1);
    let defeated_elite = defeated_save.save.generic_actors.iter()
        .find(|actor| actor.entity_id == "cw_forged_guard_elite")
        .expect("the first-kill SaveV6 retains the canonical elite row");
    assert_eq!(defeated_elite.entity_type, "enemy.clockworks.forged_guard_elite");
    assert_eq!(defeated_elite.hp, 0);
    assert!(!defeated_save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == CAP_AIR_STEP));
    let defeated_bytes = std::fs::read(&save_path).unwrap();
    let defeated_position = defeated_view.player.transform.position_m;
    let defeated_hp = defeated_view.player.current_hp;
    let defeated_energy = defeated_view.player.current_energy;
    let defeated_time = defeated_view.server_time_ms;
    let defeated_epoch = defeated_view.world_epoch;
    drop(runtime);

    // A distinct full production owner must restore the first-kill route before readiness.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, defeated_epoch, "first-kill Continue creates a new owner epoch");
    let token = prepared.entry_token.as_ref().expect("fresh owner waits for its arena readiness token");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_forged_guard_arena");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.world_id, "clockworks");
    assert_eq!(prepared.scene_id, "cw_forged_guard_arena");
    assert_eq!(prepared.player.transform.position_m, defeated_position);
    assert_eq!(prepared.player.current_hp, defeated_hp);
    assert_eq!(prepared.player.current_energy, defeated_energy);
    assert_eq!(prepared.server_time_ms, defeated_time);
    assert!(prepared.player.current_hp > 0);
    assert!(active_enemy_ids(&prepared).is_empty(), "Continue keeps the saved elite dead");
    assert!(prepared.actors.iter().all(|actor| {
        actor.entity_id != "cw_forged_guard_elite" || !actor.active
    }), "the stable elite ID remains inactive in the restored arena");
    assert_earned_valve_route_view(&prepared, "cw_forged_guard_arena");
    assert_eq!(std::fs::read(&save_path).unwrap(), defeated_bytes,
        "first-kill Continue preparation cannot rewrite its SaveV6");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath")),
        "fresh-owner preparation cannot replay the elite defeat");

    let continued = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&continued, "cw_forged_guard_arena");
    assert_eq!(continued.player.transform.position_m, defeated_position);
    assert_eq!(continued.player.current_hp, defeated_hp);
    assert_eq!(continued.player.current_energy, defeated_energy);
    assert_eq!(continued.server_time_ms, defeated_time);
    assert!(continued.player.current_hp > 0);
    assert!(active_enemy_ids(&continued).is_empty());
    assert!(runtime.presentation_events_since(continued.world_epoch, 0).unwrap().iter()
        .all(|event| !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath")),
        "arena readiness cannot replay the elite defeat");

    let arena_scene = compiled_route_scene("cw_forged_guard_arena");
    let arena_transition = arena_scene["transitions"].as_array().unwrap().iter()
        .find(|transition| transition["id"] == "cw_arena_to_regulator_core")
        .expect("the authored arena-to-regulator transition exists");
    assert_eq!(arena_transition["toSceneId"], "cw_regulator_core");
    assert_eq!(arena_transition["spawnId"], "cw_regulator_core_spawn");
    let regulator_scene = compiled_route_scene("cw_regulator_core");
    let regulator_spawn = regulator_scene["spawns"].as_array().unwrap().iter()
        .find(|spawn| spawn["id"] == "cw_regulator_core_spawn")
        .expect("the registered regulator player spawn exists");
    assert_eq!(regulator_spawn["kind"], "player");
    assert_eq!(regulator_spawn["position"], serde_json::json!([3.0, 0.0, 8.0]));
    let prime_spawn = regulator_scene["spawns"].as_array().unwrap().iter()
        .find(|spawn| spawn["id"] == "cw_prime_regulator")
        .expect("the authored regulator actor spawn exists");
    assert_eq!(prime_spawn["kind"], "enemy");
    assert_eq!(prime_spawn["entityType"], "enemy.clockworks.prime_regulator");

    let route_event_cursor = runtime.presentation_events_since(continued.world_epoch, 0).unwrap().iter()
        .map(|event| event.event_id).max().unwrap_or(0);
    let approach = walk_to_compiled_target(
        &runtime, continued, "cw_forged_guard_arena", "transitions", "cw_arena_to_regulator_core");
    assert_eq!(approach.scene_id, "cw_forged_guard_arena");
    assert!(approach.player.current_hp > 0);
    assert!(active_enemy_ids(&approach).is_empty(), "the cleared arena stays clear during route traversal");
    let affordance = approach.interactables.iter()
        .find(|item| item.entity_id == "cw_arena_to_regulator_core")
        .expect("the authored arena route is projected after first-kill Continue");
    assert_eq!(affordance.kind, "scene_transition");
    assert!(affordance.active, "the earned first-kill state opens the real arena route");
    let arena_events = runtime.presentation_events_since(approach.world_epoch, route_event_cursor).unwrap();
    assert!(arena_events.iter().all(|event| event.kind != "ForgedGuardEliteDeath"),
        "walking through the unlocked route does not replay the elite defeat");
    assert!(arena_events.iter().all(|event| event.kind != "PrimeRegulatorDeath"));
    let before_transition_cursor = runtime.presentation_events_since(approach.world_epoch, 0).unwrap().iter()
        .map(|event| event.event_id).max().unwrap_or(0);
    let entry = apply_transition(
        &runtime,
        "cw_arena_to_regulator_core",
        "genuine-elite-first-kill-to-regulator-entry",
        approach.world_epoch,
    );
    assert_eq!(entry.world_id, "clockworks");
    assert_eq!(entry.scene_id, "cw_regulator_core");
    assert!(entry.player.current_hp > 0);
    assert!((entry.player.transform.position_m.x_m - 3.0).abs() <= 0.1);
    assert!((entry.player.transform.position_m.y_m - 0.0).abs() <= 0.1);
    assert!((entry.player.transform.position_m.z_m - 8.0).abs() <= 0.1);
    assert_earned_valve_route_view(&entry, "cw_regulator_core");
    let expected_regulator_ids = BTreeSet::from(["cw_prime_regulator".to_owned()]);
    assert_eq!(active_enemy_ids(&entry), expected_regulator_ids,
        "the real transition loads only the living authored Prime Regulator");
    let regulator_actor = entry.actors.iter()
        .find(|actor| actor.entity_id == "cw_prime_regulator")
        .expect("the canonical regulator ID is projected at entry");
    assert_eq!(regulator_actor.actor_kind, "enemy");
    assert!(regulator_actor.active);
    let owner_events = runtime.presentation_events_since(entry.world_epoch, before_transition_cursor).unwrap();
    assert!(owner_events.iter().filter(|event| event.world_epoch == entry.world_epoch)
        .all(|event| event.kind != "ForgedGuardEliteDeath"),
        "entering the regulator scene cannot replay the elite defeat");
    assert!(owner_events.iter().all(|event| event.kind != "PrimeRegulatorDeath"),
        "regulator combat has not started");
    // Save and Continue the live regulator entry, then stop before attacking it or the core console.
    let entry_view = runtime.pause().unwrap();
    assert_earned_valve_route_view(&entry_view, "cw_regulator_core");
    assert_eq!(active_enemy_ids(&entry_view), expected_regulator_ids);
    runtime.save().unwrap();
    let entry_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&entry_save, "cw_regulator_core");
    assert_eq!(entry_save.save.server_time_ms, entry_view.server_time_ms);
    assert_eq!(entry_save.save.player.position_m, entry_view.player.transform.position_m);
    assert_eq!(entry_save.save.player.current_hp, entry_view.player.current_hp);
    assert_eq!(entry_save.save.player.current_energy, entry_view.player.current_energy);
    assert!(entry_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(!entry_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!entry_save.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(entry_save.effect_sources, earned_effect_sources);
    assert_eq!(entry_save.save.capabilities.grants, earned_grants);
    assert_eq!(entry_save.save.generic_actors.len(), 1);
    let saved_regulator = entry_save.save.generic_actors.iter()
        .find(|actor| actor.entity_id == "cw_prime_regulator")
        .expect("durable regulator entry retains its canonical actor row");
    assert_eq!(saved_regulator.entity_type, "enemy.clockworks.prime_regulator");
    assert!(saved_regulator.hp > 0, "the regulator remains alive in the entry save");
    let entry_checkpoint = entry_save.save.checkpoint_id.clone();
    let entry_bytes = std::fs::read(&save_path).unwrap();
    let entry_position = entry_view.player.transform.position_m;
    let entry_hp = entry_view.player.current_hp;
    let entry_energy = entry_view.player.current_energy;
    let entry_time = entry_view.server_time_ms;
    let entry_epoch = entry_view.world_epoch;
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, entry_epoch, "regulator Continue creates a new owner epoch");
    let token = prepared.entry_token.as_ref().expect("fresh regulator owner awaits normal readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_regulator_core");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.scene_id, "cw_regulator_core");
    assert_eq!(prepared.player.transform.position_m, entry_position);
    assert_eq!(prepared.player.current_hp, entry_hp);
    assert_eq!(prepared.player.current_energy, entry_energy);
    assert_eq!(prepared.server_time_ms, entry_time);
    assert!(prepared.player.current_hp > 0);
    assert_eq!(active_enemy_ids(&prepared), expected_regulator_ids);
    assert!(prepared.actors.iter().any(|actor| actor.entity_id == "cw_prime_regulator" && actor.active));
    assert_earned_valve_route_view(&prepared, "cw_regulator_core");
    assert_eq!(std::fs::read(&save_path).unwrap(), entry_bytes,
        "fresh regulator Continue preparation cannot rewrite the durable entry");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter()
        .all(|event| !matches!(event.kind.as_str(),
            "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "fresh owner preparation cannot replay the elite or begin the regulator encounter");

    let continued = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&continued, "cw_regulator_core");
    assert_eq!(continued.player.transform.position_m, entry_position);
    assert_eq!(continued.player.current_hp, entry_hp);
    assert_eq!(continued.player.current_energy, entry_energy);
    assert_eq!(continued.server_time_ms, entry_time);
    assert!(continued.player.current_hp > 0);
    assert_eq!(active_enemy_ids(&continued), expected_regulator_ids);
    assert!(runtime.presentation_events_since(continued.world_epoch, 0).unwrap().iter()
        .all(|event| !matches!(event.kind.as_str(),
            "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "readiness cannot replay the elite or start regulator combat");

    let final_entry_view = runtime.pause().unwrap();
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&final_save, "cw_regulator_core");
    assert_eq!(final_save.save.checkpoint_id, entry_checkpoint,
        "fresh owner preserves the actual saved destination checkpoint semantics");
    assert_eq!(final_save.save.server_time_ms, final_entry_view.server_time_ms);
    assert_eq!(final_save.save.player.position_m, final_entry_view.player.transform.position_m);
    assert_eq!(final_save.save.player.current_hp, final_entry_view.player.current_hp);
    assert_eq!(final_save.save.player.current_energy, final_entry_view.player.current_energy);
    assert!(final_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(!final_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!final_save.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(final_save.effect_sources, earned_effect_sources);
    assert_eq!(final_save.save.capabilities.grants, earned_grants);
    assert_eq!(final_save.save.generic_actors.len(), 1);
    let final_regulator = final_save.save.generic_actors.iter()
        .find(|actor| actor.entity_id == "cw_prime_regulator")
        .expect("the final regulator-entry SaveV6 retains its canonical actor row");
    assert_eq!(final_regulator.entity_type, "enemy.clockworks.prime_regulator");
    assert!(final_regulator.hp > 0, "final Continue keeps the regulator alive");
    let final_bytes = std::fs::read(&save_path).unwrap();
    println!(
        "Durable regulator entry survived fresh-owner Continue: {} HP/{} energy, actor {} ({} HP), time {}, checkpoint {:?}, final SaveV6 {} bytes at {}",
        final_save.save.player.current_hp,
        final_save.save.player.current_energy,
        final_regulator.entity_id,
        final_regulator.hp,
        final_save.save.server_time_ms,
        final_save.save.checkpoint_id,
        final_bytes.len(),
        save_path.display(),
    );
    drop(runtime);
}

// Exact qualified living regulator-entry SaveV6 from the earned elite first-kill route.
const EARNED_CW_REGULATOR_ENTRY_V6: &str =
    include_str!("../fixtures/earned-cw-regulator-entry-v6.json");

#[test]
fn genuine_earned_regulator_core_reaches_durable_shutdown_entry() {
    let save_dir = save_root();
    assert!(!save_dir.exists(), "task save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_dir.join("formal-save-v6.json");
    std::fs::write(&save_path, EARNED_CW_REGULATOR_ENTRY_V6.as_bytes()).unwrap();

    let entry_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_valve_route_save(&entry_save, "cw_regulator_core");
    assert_eq!(entry_save.save.checkpoint_id.as_deref(), None);
    assert_eq!(entry_save.save.player.current_hp, 46);
    assert_eq!(entry_save.save.player.current_energy, 28);
    assert_eq!(entry_save.save.player.position_m.x_m, 3.0);
    assert_eq!(entry_save.save.player.position_m.y_m, 0.0);
    assert_eq!(entry_save.save.player.position_m.z_m, 8.0);
    assert_eq!(entry_save.save.server_time_ms, 658_274);
    assert_eq!(entry_save.save.generic_actors.len(), 1);
    let living_regulator = &entry_save.save.generic_actors[0];
    assert_eq!(living_regulator.entity_id, "cw_prime_regulator");
    assert_eq!(living_regulator.entity_type, "enemy.clockworks.prime_regulator");
    assert_eq!(living_regulator.hp, 600);
    assert!(entry_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(!entry_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!entry_save.world_persistent_v1.clockworks.core_console_confirmed);
    let earned_effect_sources = entry_save.effect_sources.clone();
    let earned_grants = entry_save.save.capabilities.grants.clone();

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    let entry_epoch = prepared.world_epoch;
    let token = prepared.entry_token.as_ref().expect("fresh regulator owner awaits normal readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_regulator_core");
    assert_eq!(token.world_epoch, entry_epoch);
    assert_earned_valve_route_view(&prepared, "cw_regulator_core");
    assert_eq!(prepared.player.current_hp, 46);
    assert_eq!(prepared.player.current_energy, 28);
    assert_eq!(prepared.player.transform.position_m.x_m, 3.0);
    assert_eq!(prepared.player.transform.position_m.z_m, 8.0);
    assert_eq!(prepared.server_time_ms, 658_274);
    assert_eq!(active_enemy_ids(&prepared), BTreeSet::from(["cw_prime_regulator".to_owned()]));
    assert!(prepared.actors.iter().any(|actor|
        actor.entity_id == "cw_prime_regulator" && actor.actor_kind == "enemy" && actor.active));
    assert!(runtime.presentation_events_since(entry_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "regulator-entry Continue preparation cannot replay an earlier defeat");
    let continued_entry = acknowledge_ready(&runtime, prepared);
    assert_earned_valve_route_view(&continued_entry, "cw_regulator_core");
    assert_eq!(continued_entry.player.current_hp, 46);
    assert_eq!(continued_entry.player.current_energy, 28);
    assert_eq!(active_enemy_ids(&continued_entry), BTreeSet::from(["cw_prime_regulator".to_owned()]));
    assert!(runtime.presentation_events_since(entry_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "regulator-entry readiness cannot replay an earlier defeat");

    let combat_cursor = runtime.presentation_events_since(entry_epoch, 0).unwrap().iter()
        .map(|event| event.event_id).max().unwrap_or(0);
    let defeated_view = clear_live_enemies(&runtime, continued_entry);
    assert_eq!(defeated_view.world_epoch, entry_epoch, "the real boss defeat remains in the entered epoch");
    assert_eq!(defeated_view.scene_id, "cw_regulator_core");
    assert!(defeated_view.player.current_hp > 0, "earned regulator combat must finish alive");
    assert!(active_enemy_ids(&defeated_view).is_empty(), "public combat must actually defeat the regulator");
    let combat_events = runtime.presentation_events_since(entry_epoch, combat_cursor).unwrap();
    let regulator_deaths = combat_events.iter().filter(|event|
        event.kind == "PrimeRegulatorDeath" && event.world_epoch == entry_epoch).collect::<Vec<_>>();
    assert_eq!(regulator_deaths.len(), 1,
        "one same-epoch public PrimeRegulatorDeath must follow the observed combat cursor");

    runtime.save().unwrap();
    let defeated_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(defeated_save.save.world_id, "clockworks");
    assert_eq!(defeated_save.save.scene_id, "cw_regulator_core");
    let defeated_progress = defeated_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").expect("Clockworks progress is saved");
    assert_eq!(defeated_progress.visit_id, 1);
    assert_eq!(defeated_progress.cycle_id, 1);
    assert_eq!(defeated_progress.revisit_count, 0);
    assert!(!defeated_progress.completed && !defeated_progress.first_completion);
    assert_eq!(defeated_progress.completed_events, vec!["clockworks_valves"]);
    assert!(defeated_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(defeated_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!defeated_save.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(defeated_save.effect_sources, earned_effect_sources);
    assert_eq!(defeated_save.save.capabilities.grants, earned_grants);
    assert!(!defeated_save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == CAP_AIR_STEP));
    assert_eq!(defeated_save.save.generic_actors.len(), 1);
    let defeated_regulator = &defeated_save.save.generic_actors[0];
    assert_eq!(defeated_regulator.entity_id, "cw_prime_regulator");
    assert_eq!(defeated_regulator.entity_type, "enemy.clockworks.prime_regulator");
    assert_eq!(defeated_regulator.hp, 0);
    assert_eq!(defeated_save.save.player.current_hp, defeated_view.player.current_hp);
    assert!(defeated_save.save.player.current_hp > 0);
    assert_eq!(defeated_save.save.server_time_ms, defeated_view.server_time_ms);
    let defeated_position = defeated_view.player.transform.position_m;
    let defeated_hp = defeated_view.player.current_hp;
    let defeated_energy = defeated_view.player.current_energy;
    let defeated_time = defeated_view.server_time_ms;
    let defeated_bytes = std::fs::read(&save_path).unwrap();
    drop(runtime);

    // A distinct full owner must continue the dead regulator before and after readiness.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, entry_epoch, "defeat Continue creates a fresh owner epoch");
    let token = prepared.entry_token.as_ref().expect("new regulator owner awaits readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_regulator_core");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.scene_id, "cw_regulator_core");
    assert_eq!(prepared.player.transform.position_m, defeated_position);
    assert_eq!(prepared.player.current_hp, defeated_hp);
    assert_eq!(prepared.player.current_energy, defeated_energy);
    assert_eq!(prepared.server_time_ms, defeated_time);
    assert!(prepared.player.current_hp > 0);
    assert!(active_enemy_ids(&prepared).is_empty(), "Continue retains the dead regulator");
    assert!(prepared.actors.iter().any(|actor|
        actor.entity_id == "cw_prime_regulator" && actor.actor_kind == "enemy" && !actor.active));
    assert_earned_valve_route_view(&prepared, "cw_regulator_core");
    assert_eq!(std::fs::read(&save_path).unwrap(), defeated_bytes,
        "Continue preparation cannot rewrite the defeated SaveV6");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "new-owner preparation cannot replay the elite or regulator defeat");
    let mut view = acknowledge_ready(&runtime, prepared);
    assert_eq!(view.player.transform.position_m, defeated_position);
    assert_eq!(view.player.current_hp, defeated_hp);
    assert_eq!(view.player.current_energy, defeated_energy);
    assert_eq!(view.server_time_ms, defeated_time);
    assert!(active_enemy_ids(&view).is_empty());
    assert_earned_valve_route_view(&view, "cw_regulator_core");
    assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "new-owner readiness cannot replay the elite or regulator defeat");
    let console_epoch = view.world_epoch;
    view = interact(&runtime, view, "cw_regulator_core_console_staged", "genuine-cw-core");
    assert_eq!(view.world_epoch, console_epoch, "the core receipt belongs to the current owner epoch");
    assert_eq!(view.scene_id, "cw_regulator_core");
    assert!(view.player.current_hp > 0);
    assert!(active_enemy_ids(&view).is_empty());
    assert_earned_clockworks_prefix(&view);
    let core_progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks")
        .expect("Clockworks core-stage progress remains projected");
    assert_eq!(core_progress.visit_id, 1);
    assert_eq!(core_progress.cycle_id, 1);
    assert_eq!(core_progress.revisit_count, 0);
    assert!(!core_progress.completed && !core_progress.first_completion);
    assert_eq!(core_progress.completed_events,
        vec!["clockworks_valves".to_owned(), "clockworks_core".to_owned()]);
    assert!(runtime.presentation_events_since(console_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "PrimeRegulatorDeath")),
        "core confirmation cannot replay or duplicate the regulator defeat");
    assert!(!view.capabilities.items.iter()
        .any(|item| item.capability_id == CAP_AIR_STEP && item.granted));

    let core_epoch = view.world_epoch;
    let at_exit = walk_to_compiled_target(&runtime, view, "cw_regulator_core", "transitions",
        "cw_regulator_core_to_shutdown_exit");
    let exit_affordance = at_exit.interactables.iter()
        .find(|item| item.entity_id == "cw_regulator_core_to_shutdown_exit")
        .expect("authored shutdown transition is projected at the reached gate");
    assert_eq!(exit_affordance.kind, "scene_transition");
    assert!(exit_affordance.active,
        "the actual core confirmation enables the authored transition when the player reaches it");
    let mut shutdown_view = apply_transition(&runtime, "cw_regulator_core_to_shutdown_exit",
        "genuine-cw-core-to-shutdown-entry", at_exit.world_epoch);
    assert_ne!(shutdown_view.world_epoch, core_epoch, "the gated authored transition installs a fresh scene epoch");
    assert_eq!(shutdown_view.world_id, "clockworks");
    assert_eq!(shutdown_view.scene_id, "cw_shutdown_exit");
    assert!(shutdown_view.player.current_hp > 0);
    assert!((shutdown_view.player.transform.position_m.x_m - 3.0).abs() <= 0.1);
    assert!((shutdown_view.player.transform.position_m.y_m - 0.0).abs() <= 0.1);
    assert!((shutdown_view.player.transform.position_m.z_m - 8.0).abs() <= 0.1);
    assert!(active_enemy_ids(&shutdown_view).is_empty());
    assert_earned_clockworks_prefix(&shutdown_view);
    let shutdown_progress = shutdown_view.progression.worlds.iter()
        .find(|row| row.world_id == "clockworks").expect("Clockworks shutdown-entry progress remains projected");
    assert_eq!(shutdown_progress.visit_id, 1);
    assert_eq!(shutdown_progress.cycle_id, 1);
    assert_eq!(shutdown_progress.revisit_count, 0);
    assert!(!shutdown_progress.completed && !shutdown_progress.first_completion);
    assert_eq!(shutdown_progress.completed_events,
        vec!["clockworks_valves".to_owned(), "clockworks_core".to_owned()]);
    assert!(shutdown_view.capabilities.items.iter()
        .all(|item| item.capability_id != CAP_AIR_STEP || !item.granted));
    assert!(runtime.presentation_events_since(shutdown_view.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "shutdown-entry installation cannot replay either earned Boss defeat");
    assert_eq!(shutdown_view.checkpoint_id.as_deref(), None,
        "the authored transition enters at the lower spawn without activating the later checkpoint");

    runtime.save().unwrap();
    let shutdown_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(shutdown_save.save.world_id, "clockworks");
    assert_eq!(shutdown_save.save.scene_id, "cw_shutdown_exit");
    assert_eq!(shutdown_save.save.checkpoint_id.as_deref(), None);
    assert_eq!(shutdown_save.save.player.position_m, shutdown_view.player.transform.position_m);
    assert!(shutdown_save.save.player.current_hp > 0);
    assert_eq!(shutdown_save.save.server_time_ms, shutdown_view.server_time_ms);
    assert!(shutdown_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(shutdown_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(shutdown_save.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(shutdown_save.world_persistent_v1.clockworks.pressure_valve_ids, vec![
        "cw_pressure_valve_01_staged".to_owned(),
        "cw_pressure_valve_02_staged".to_owned(),
        "cw_pressure_valve_03_staged".to_owned(),
    ]);
    let shutdown_saved_progress = shutdown_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").expect("shutdown-entry progress is durable");
    assert!(!shutdown_saved_progress.completed && !shutdown_saved_progress.first_completion);
    assert_eq!(shutdown_saved_progress.completed_events,
        vec!["clockworks_valves".to_owned(), "clockworks_core".to_owned()]);
    assert_eq!(shutdown_save.effect_sources, earned_effect_sources);
    assert_eq!(shutdown_save.save.capabilities.grants, earned_grants);
    assert!(!shutdown_save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == CAP_AIR_STEP));
    let shutdown_checkpoint = shutdown_save.save.checkpoint_id.clone();
    let shutdown_bytes = std::fs::read(&save_path).unwrap();
    let shutdown_position = shutdown_view.player.transform.position_m;
    let shutdown_hp = shutdown_view.player.current_hp;
    let shutdown_energy = shutdown_view.player.current_energy;
    let shutdown_time = shutdown_view.server_time_ms;
    let shutdown_epoch = shutdown_view.world_epoch;
    drop(runtime);

    // A final fresh owner proves the durable two-event shutdown entry is Continue-ready.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, shutdown_epoch, "shutdown-entry Continue creates a fresh owner epoch");
    let token = prepared.entry_token.as_ref().expect("shutdown-entry owner waits for normal readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_shutdown_exit");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.scene_id, "cw_shutdown_exit");
    assert_eq!(prepared.player.transform.position_m, shutdown_position);
    assert_eq!(prepared.player.current_hp, shutdown_hp);
    assert_eq!(prepared.player.current_energy, shutdown_energy);
    assert_eq!(prepared.server_time_ms, shutdown_time);
    assert!(prepared.player.current_hp > 0);
    assert!(active_enemy_ids(&prepared).is_empty());
    assert_earned_clockworks_prefix(&prepared);
    let prepared_progress = prepared.progression.worlds.iter()
        .find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(prepared_progress.completed_events,
        vec!["clockworks_valves".to_owned(), "clockworks_core".to_owned()]);
    assert!(!prepared_progress.completed && !prepared_progress.first_completion);
    assert_eq!(std::fs::read(&save_path).unwrap(), shutdown_bytes,
        "shutdown-entry Continue preparation cannot rewrite its durable save");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "shutdown-entry preparation cannot replay an earned defeat");
    shutdown_view = acknowledge_ready(&runtime, prepared);
    assert_eq!(shutdown_view.scene_id, "cw_shutdown_exit");
    assert_eq!(shutdown_view.player.transform.position_m, shutdown_position);
    assert_eq!(shutdown_view.player.current_hp, shutdown_hp);
    assert_eq!(shutdown_view.player.current_energy, shutdown_energy);
    assert_eq!(shutdown_view.server_time_ms, shutdown_time);
    assert_eq!(shutdown_view.checkpoint_id, shutdown_checkpoint);
    assert!(shutdown_view.player.current_hp > 0);
    assert!(active_enemy_ids(&shutdown_view).is_empty());
    assert_earned_clockworks_prefix(&shutdown_view);
    assert!(runtime.presentation_events_since(shutdown_view.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "shutdown-entry readiness cannot replay an earned defeat");
    assert!(!shutdown_view.capabilities.items.iter()
        .any(|item| item.capability_id == CAP_AIR_STEP && item.granted));
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(final_save.save.scene_id, "cw_shutdown_exit");
    assert_eq!(final_save.save.checkpoint_id, shutdown_checkpoint,
        "final Continue save preserves the exact authored destination checkpoint state");
    assert!(final_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(final_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(final_save.world_persistent_v1.clockworks.core_console_confirmed);
    let final_progress = final_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(final_progress.completed_events,
        vec!["clockworks_valves".to_owned(), "clockworks_core".to_owned()]);
    assert!(!final_progress.completed && !final_progress.first_completion);
    assert!(!final_save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == CAP_AIR_STEP));
    let final_bytes = std::fs::read(&save_path).unwrap();
    println!(
        "Earned Prime Regulator defeat, actual core confirmation, gated shutdown-entry transition, and fresh-owner Continue completed alive at {} HP/{} energy, time {}, checkpoint {:?}; final SaveV6 {} bytes at {}",
        final_save.save.player.current_hp,
        final_save.save.player.current_energy,
        final_save.save.server_time_ms,
        final_save.save.checkpoint_id,
        final_bytes.len(),
        save_path.display(),
    );
    drop(runtime);
}

// Exact qualified two-event shutdown-entry SaveV6 from the earned Regulator/core route.
const EARNED_CW_SHUTDOWN_ENTRY_V6: &str =
    include_str!("../fixtures/earned-cw-shutdown-entry-v6.json");

#[test]
fn genuine_earned_shutdown_returns_durable_three_world_completion() {
    let save_dir = save_root();
    assert!(!save_dir.exists(), "task save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_dir.join("formal-save-v6.json");
    std::fs::write(&save_path, EARNED_CW_SHUTDOWN_ENTRY_V6.as_bytes()).unwrap();
    let fixture_bytes = EARNED_CW_SHUTDOWN_ENTRY_V6.as_bytes().to_vec();
    assert_eq!(std::fs::read(&save_path).unwrap(), fixture_bytes,
        "the scenario starts from the exact embedded qualified SaveV6");

    let entry_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(entry_save.save.world_id, "clockworks");
    assert_eq!(entry_save.save.scene_id, "cw_shutdown_exit");
    assert_eq!(entry_save.save.checkpoint_id, None);
    assert_eq!(entry_save.save.player.position_m.x_m, 3.0);
    assert_eq!(entry_save.save.player.position_m.y_m, 0.0);
    assert_eq!(entry_save.save.player.position_m.z_m, 8.0);
    assert_eq!(entry_save.save.player.current_hp, 46);
    assert_eq!(entry_save.save.player.current_energy, 28);
    assert_eq!(entry_save.save.server_time_ms, 743_223);
    assert!(entry_save.save.generic_actors.is_empty(),
        "the qualified shutdown entry has no active or latent actors");
    let clockworks_entry = entry_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").expect("Clockworks progress is saved");
    assert_eq!(clockworks_entry.visit_id, 1);
    assert_eq!(clockworks_entry.cycle_id, 1);
    assert_eq!(clockworks_entry.revisit_count, 0);
    assert!(!clockworks_entry.completed && !clockworks_entry.first_completion);
    assert_eq!(clockworks_entry.completed_events,
        vec!["clockworks_valves".to_owned(), "clockworks_core".to_owned()]);
    assert!(entry_save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(entry_save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(entry_save.world_persistent_v1.clockworks.core_console_confirmed);
    let initial_effect_sources = entry_save.effect_sources.clone();
    let initial_grants = entry_save.save.capabilities.grants.clone();
    let initial_selection = entry_save.save.capabilities.selected.clone();
    assert!(!initial_effect_sources.iter().any(|source| source.source_id == CAP_AIR_STEP));
    assert!(!initial_grants.iter().any(|grant| grant.capability_id == CAP_AIR_STEP));
    assert!(!initial_selection.iter().any(|capability| capability == CAP_AIR_STEP));
    let prior_worlds = [
        ("grey_hive", vec!["hive_power", "hive_lockdown", "hive_extraction"]),
        ("mist_harbor", vec!["mist_beacon_west", "mist_beacon_east", "mist_signal"]),
    ];
    for (world_id, events) in &prior_worlds {
        let row = entry_save.save.progression.progress.iter()
            .find(|row| row.world_id == *world_id).expect("prior earned world is saved");
        assert!(row.completed && row.first_completion,
            "prior {world_id} completion remains earned at shutdown entry");
        assert_eq!(row.completed_events.iter().map(String::as_str).collect::<Vec<_>>(),
            events.clone(), "prior {world_id} event set is preserved");
    }

    // Keep client request timestamps strictly beyond the admitted last-client value.
    // The native saved server clock and body are never synthesized or reset.
    NEXT_TIME.fetch_max(entry_save.save.last_client_time_ms.saturating_add(1), Ordering::Relaxed);
    assert!(NEXT_TIME.load(Ordering::Relaxed) > entry_save.save.last_client_time_ms);

    let assert_campaign_view = |view: &WorldView, expected_world: &str, expected_scene: &str,
                                expected_events: &[&str], completed: bool, air_step: bool| {
        assert_eq!(view.world_id, expected_world);
        assert_eq!(view.scene_id, expected_scene);
        assert!(view.player.current_hp > 0, "the earned shutdown route remains alive");
        let progress = view.progression.worlds.iter().find(|row| row.world_id == "clockworks")
            .expect("Clockworks progress remains projected");
        assert_eq!(progress.visit_id, 1);
        assert_eq!(progress.cycle_id, 1);
        assert_eq!(progress.revisit_count, 0);
        assert_eq!(progress.completed, completed);
        assert_eq!(progress.first_completion, completed);
        assert_eq!(progress.completed_events,
            expected_events.iter().map(|event| (*event).to_owned()).collect::<Vec<_>>());
        for (world_id, expected_prior_events) in &prior_worlds {
            let prior = view.progression.worlds.iter().find(|row| row.world_id == *world_id)
                .unwrap_or_else(|| panic!("missing retained {world_id} progress"));
            assert!(prior.completed && prior.first_completion);
            assert_eq!(prior.visit_id, 1);
            assert_eq!(prior.cycle_id, 1);
            assert_eq!(prior.revisit_count, 0);
            assert_eq!(prior.completed_events,
                expected_prior_events.iter().map(|event| (*event).to_owned()).collect::<Vec<_>>());
        }
        let projected_air_step = view.capabilities.items.iter()
            .find(|item| item.capability_id == CAP_AIR_STEP);
        assert_eq!(projected_air_step.is_some_and(|item| item.granted), air_step,
            "AirStep grant follows only the actual shutdown transaction");
        assert_eq!(projected_air_step.is_some_and(|item| item.selected), air_step,
            "AirStep selection follows only the actual shutdown transaction");
    };
    let assert_campaign_save = |saved: &save_v6::SaveV6, expected_world: &str, expected_scene: &str,
                                expected_events: &[&str], completed: bool, air_step: bool| {
        assert_eq!(saved.save.world_id, expected_world);
        assert_eq!(saved.save.scene_id, expected_scene);
        let progress = saved.save.progression.progress.iter()
            .find(|row| row.world_id == "clockworks").expect("Clockworks SaveV6 row is present");
        assert_eq!(progress.visit_id, 1);
        assert_eq!(progress.cycle_id, 1);
        assert_eq!(progress.revisit_count, 0);
        assert_eq!(progress.completed, completed);
        assert_eq!(progress.first_completion, completed);
        assert_eq!(progress.completed_events,
            expected_events.iter().map(|event| (*event).to_owned()).collect::<Vec<_>>());
        for (world_id, expected_prior_events) in &prior_worlds {
            let prior = saved.save.progression.progress.iter()
                .find(|row| row.world_id == *world_id)
                .unwrap_or_else(|| panic!("missing retained {world_id} SaveV6 progress"));
            assert!(prior.completed && prior.first_completion);
            assert_eq!(prior.visit_id, 1);
            assert_eq!(prior.cycle_id, 1);
            assert_eq!(prior.revisit_count, 0);
            assert_eq!(prior.completed_events,
                expected_prior_events.iter().map(|event| (*event).to_owned()).collect::<Vec<_>>());
        }
        let clockworks = &saved.world_persistent_v1.clockworks;
        assert!(clockworks.forged_guard_elite_first_kill);
        assert!(clockworks.regulator_defeated);
        assert!(clockworks.core_console_confirmed);
        assert_eq!(clockworks.pressure_valve_ids, vec![
            "cw_pressure_valve_01_staged".to_owned(),
            "cw_pressure_valve_02_staged".to_owned(),
            "cw_pressure_valve_03_staged".to_owned(),
        ]);
        let expected_award_count = if air_step { 1 } else { 0 };
        assert_eq!(saved.effect_sources.iter()
            .filter(|source| source.source_id == CAP_AIR_STEP).count(), expected_award_count,
            "the real shutdown awards exactly one AirStep effect source");
        assert_eq!(saved.save.capabilities.grants.iter()
            .filter(|grant| grant.capability_id == CAP_AIR_STEP).count(), expected_award_count,
            "the real shutdown awards exactly one AirStep capability grant");
        assert_eq!(saved.save.capabilities.selected.iter()
            .any(|capability| capability == CAP_AIR_STEP), air_step,
            "AirStep selection matches the earned completion stage");
    };

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    let entry_epoch = prepared.world_epoch;
    let token = prepared.entry_token.as_ref()
        .expect("full production Continue waits for normal shutdown-entry readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_shutdown_exit");
    assert_eq!(token.world_epoch, entry_epoch);
    assert_campaign_view(&prepared, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core"], false, false);
    assert_eq!(prepared.player.transform.position_m, entry_save.save.player.position_m);
    assert_eq!(prepared.player.current_hp, entry_save.save.player.current_hp);
    assert_eq!(prepared.player.current_energy, entry_save.save.player.current_energy);
    assert_eq!(prepared.server_time_ms, entry_save.save.server_time_ms);
    assert_eq!(prepared.checkpoint_id, entry_save.save.checkpoint_id);
    assert_eq!(std::fs::read(&save_path).unwrap(), fixture_bytes,
        "shutdown-entry Continue preparation cannot rewrite the admitted save");
    assert!(runtime.presentation_events_since(entry_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "shutdown-entry preparation cannot replay an earned Boss defeat");
    let mut view = acknowledge_ready(&runtime, prepared);
    assert_campaign_view(&view, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core"], false, false);
    assert_eq!(view.player.transform.position_m, entry_save.save.player.position_m);
    assert_eq!(view.player.current_hp, entry_save.save.player.current_hp);
    assert_eq!(view.player.current_energy, entry_save.save.player.current_energy);
    assert_eq!(view.server_time_ms, entry_save.save.server_time_ms);

    // The authored master terminal is the only completion action in this scenario.
    view = interact(&runtime, view, "cw_master_shutdown_staged", "genuine-cw-shutdown-once");
    let shutdown_epoch = view.world_epoch;
    assert_eq!(shutdown_epoch, entry_epoch, "the real terminal receipt is in the entered epoch");
    assert_campaign_view(&view, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert!(runtime.presentation_events_since(shutdown_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "shutdown completion does not replay either earned Boss defeat");
    runtime.save().unwrap();
    let awarded_save = save_v6::read_save(&save_dir).unwrap();
    assert_campaign_save(&awarded_save, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert!(initial_effect_sources.iter().all(|source| awarded_save.effect_sources.contains(source)),
        "all prior earned effect sources survive first completion");
    assert!(initial_grants.iter().all(|grant| awarded_save.save.capabilities.grants.contains(grant)),
        "all prior earned capability grants survive first completion");
    assert_eq!(awarded_save.effect_sources.iter()
        .filter(|source| source.source_id != CAP_AIR_STEP).cloned().collect::<Vec<_>>(),
        initial_effect_sources);
    assert_eq!(awarded_save.save.capabilities.grants.iter()
        .filter(|grant| grant.capability_id != CAP_AIR_STEP).cloned().collect::<Vec<_>>(),
        initial_grants);
    let awarded_effect_sources = awarded_save.effect_sources.clone();
    let awarded_grants = awarded_save.save.capabilities.grants.clone();
    let awarded_selection = awarded_save.save.capabilities.selected.clone();
    let completed_progression = view.progression.clone();

    // One bounded, fresh-ID duplicate at the same terminal and in the same owner.
    let duplicate = scene_route_commands::interaction(&runtime, "cw_master_shutdown_staged",
        "genuine-cw-shutdown-duplicate", Some(view.world_epoch)).unwrap();
    assert!(!duplicate.applied, "the one repeated master-terminal interaction is not applied");
    assert_eq!(duplicate.view.world_epoch, shutdown_epoch);
    assert_eq!(duplicate.view.progression, completed_progression,
        "duplicate shutdown cannot change completed progression");
    assert_campaign_view(&duplicate.view, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    let paused_shutdown = runtime.pause().unwrap();
    assert_eq!(paused_shutdown.progression, completed_progression);
    runtime.save().unwrap();
    let duplicate_save = save_v6::read_save(&save_dir).unwrap();
    assert_campaign_save(&duplicate_save, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(duplicate_save.effect_sources, awarded_effect_sources,
        "duplicate shutdown preserves source identity and cannot add a second source");
    assert_eq!(duplicate_save.save.capabilities.grants, awarded_grants,
        "duplicate shutdown preserves grant identity and cannot add a second grant");
    assert_eq!(duplicate_save.save.capabilities.selected, awarded_selection,
        "duplicate shutdown preserves the original selection");
    assert_eq!(duplicate_save.effect_sources.iter()
        .filter(|source| source.source_id == CAP_AIR_STEP).count(), 1);
    assert_eq!(duplicate_save.save.capabilities.grants.iter()
        .filter(|grant| grant.capability_id == CAP_AIR_STEP).count(), 1);
    let shutdown_view = runtime.pause().unwrap();
    runtime.save().unwrap();
    let shutdown_save = save_v6::read_save(&save_dir).unwrap();
    assert_campaign_save(&shutdown_save, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(shutdown_save.effect_sources, awarded_effect_sources);
    assert_eq!(shutdown_save.save.capabilities.grants, awarded_grants);
    assert_eq!(shutdown_save.save.capabilities.selected, awarded_selection);
    assert!(shutdown_view.player.current_hp > 0);
    let shutdown_position = shutdown_view.player.transform.position_m;
    let shutdown_hp = shutdown_view.player.current_hp;
    let shutdown_energy = shutdown_view.player.current_energy;
    let shutdown_time = shutdown_view.server_time_ms;
    let shutdown_checkpoint = shutdown_view.checkpoint_id.clone();
    let shutdown_bytes = std::fs::read(&save_path).unwrap();
    drop(runtime);

    // A full new production owner must continue the completed shutdown stage without replay.
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    assert_ne!(prepared.world_epoch, shutdown_epoch,
        "shutdown completion Continue installs a fresh owner epoch");
    let token = prepared.entry_token.as_ref().expect("new shutdown owner awaits readiness");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_shutdown_exit");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_campaign_view(&prepared, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(prepared.player.transform.position_m, shutdown_position);
    assert_eq!(prepared.player.current_hp, shutdown_hp);
    assert_eq!(prepared.player.current_energy, shutdown_energy);
    assert_eq!(prepared.server_time_ms, shutdown_time);
    assert_eq!(prepared.checkpoint_id, shutdown_checkpoint);
    assert_eq!(std::fs::read(&save_path).unwrap(), shutdown_bytes,
        "new-owner preparation cannot rewrite the completed shutdown save");
    assert!(runtime.presentation_events_since(prepared.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "new-owner preparation cannot replay an earned Boss defeat");
    view = acknowledge_ready(&runtime, prepared);
    assert_campaign_view(&view, "clockworks", "cw_shutdown_exit",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(view.progression, completed_progression);
    assert_eq!(view.player.transform.position_m, shutdown_position);
    assert_eq!(view.player.current_hp, shutdown_hp);
    assert_eq!(view.player.current_energy, shutdown_energy);
    assert_eq!(view.server_time_ms, shutdown_time);
    assert_eq!(view.checkpoint_id, shutdown_checkpoint);
    assert_eq!(std::fs::read(&save_path).unwrap(), shutdown_bytes,
        "readiness cannot rewrite the completed shutdown save");
    assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "new-owner readiness cannot replay an earned Boss defeat");
    assert_eq!(save_v6::read_save(&save_dir).unwrap().effect_sources, awarded_effect_sources);
    assert_eq!(save_v6::read_save(&save_dir).unwrap().save.capabilities.grants, awarded_grants);
    assert_eq!(save_v6::read_save(&save_dir).unwrap().save.capabilities.selected, awarded_selection);

    // Return only through the authored shutdown portal, then Continue in Return Station.
    let completed_epoch = view.world_epoch;
    let returned = return_to_station(&runtime, view, "genuine-cw-shutdown-return-to-rs");
    assert_ne!(returned.world_epoch, completed_epoch,
        "the actual shutdown world-gate receipt installs the Return Station scene");
    assert_campaign_view(&returned, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert!(active_enemy_ids(&returned).is_empty());
    assert!(runtime.presentation_events_since(returned.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "Return Station entry cannot replay an earned Boss defeat");
    let returned_view = runtime.pause().unwrap();
    assert_campaign_view(&returned_view, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    let returned_position = returned_view.player.transform.position_m;
    let returned_hp = returned_view.player.current_hp;
    let returned_energy = returned_view.player.current_energy;
    let returned_time = returned_view.server_time_ms;
    let returned_checkpoint = returned_view.checkpoint_id.clone();
    runtime.save().unwrap();
    let returned_save = save_v6::read_save(&save_dir).unwrap();
    assert_campaign_save(&returned_save, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(returned_save.effect_sources, awarded_effect_sources);
    assert_eq!(returned_save.save.capabilities.grants, awarded_grants);
    assert_eq!(returned_save.save.capabilities.selected, awarded_selection);
    let returned_bytes = std::fs::read(&save_path).unwrap();
    drop(runtime);

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    let rs_epoch = prepared.world_epoch;
    assert!(prepared.entry_token.as_ref().is_some_and(|token|
        token.world_id == "return_station" && token.scene_id == "rs_core_room"
            && token.world_epoch == rs_epoch));
    assert_campaign_view(&prepared, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(prepared.player.transform.position_m, returned_position);
    assert_eq!(prepared.player.current_hp, returned_hp);
    assert_eq!(prepared.player.current_energy, returned_energy);
    assert_eq!(prepared.server_time_ms, returned_time);
    assert_eq!(prepared.checkpoint_id, returned_checkpoint);
    assert_eq!(std::fs::read(&save_path).unwrap(), returned_bytes,
        "final Return Station Continue preparation cannot rewrite its save");
    assert!(runtime.presentation_events_since(rs_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "final Return Station Continue preparation cannot replay an earned Boss defeat");
    view = acknowledge_ready(&runtime, prepared);
    assert_campaign_view(&view, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(view.player.transform.position_m, returned_position);
    assert_eq!(view.player.current_hp, returned_hp);
    assert_eq!(view.player.current_energy, returned_energy);
    assert_eq!(view.server_time_ms, returned_time);
    assert_eq!(view.checkpoint_id, returned_checkpoint);
    assert_eq!(std::fs::read(&save_path).unwrap(), returned_bytes,
        "normal Return Station readiness cannot rewrite the held save");
    assert!(runtime.presentation_events_since(view.world_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "final Return Station readiness cannot replay an earned Boss defeat");

    let final_rs_view = runtime.pause().unwrap();
    assert_campaign_view(&final_rs_view, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    runtime.save().unwrap();
    let final_rs_save = save_v6::read_save(&save_dir).unwrap();
    assert_campaign_save(&final_rs_save, "return_station", "rs_core_room",
        &["clockworks_valves", "clockworks_core", "clockworks_shutdown"], true, true);
    assert_eq!(final_rs_save.effect_sources, awarded_effect_sources);
    assert_eq!(final_rs_save.save.capabilities.grants, awarded_grants);
    assert_eq!(final_rs_save.save.capabilities.selected, awarded_selection);
    assert_eq!(final_rs_save.save.player.position_m, final_rs_view.player.transform.position_m);
    assert_eq!(final_rs_save.save.player.current_hp, final_rs_view.player.current_hp);
    assert_eq!(final_rs_save.save.player.current_energy, final_rs_view.player.current_energy);
    assert_eq!(final_rs_save.save.server_time_ms, final_rs_view.server_time_ms);
    assert_eq!(final_rs_save.save.checkpoint_id, final_rs_view.checkpoint_id);
    assert!(final_rs_save.save.player.current_hp > 0);
    assert!(active_enemy_ids(&final_rs_view).is_empty());
    let final_bytes = std::fs::read(&save_path).unwrap();
    println!(
        "Earned master shutdown, one AirStep award, fresh-owner Continue, authored Return Station gate, and final Return Station Continue completed alive at {} HP/{} energy, time {}, checkpoint {:?}; final SaveV6 {} bytes at {}",
        final_rs_save.save.player.current_hp,
        final_rs_save.save.player.current_energy,
        final_rs_save.save.server_time_ms,
        final_rs_save.save.checkpoint_id,
        final_bytes.len(),
        save_path.display(),
    );
    drop(runtime);
}
const EARNED_THREE_WORLD_RETURN_STATION_V6: &str = include_str!("../fixtures/earned-three-world-return-station-v6.json");

fn submit_and_confirm_public_context_traversal(
    runtime: &FormalRuntime,
    view: &WorldView,
    aim: (f32, f32),
) -> (WorldView, (f32, f32, f32), u64) {
    const MAX_OWNER_TIME_MS: u64 = 3_000;
    let epoch = view.world_epoch;
    let start_position = view.player.transform.position_m;
    let start = (start_position.x_m, start_position.y_m, start_position.z_m);
    let owner_start_ms = view.server_time_ms;
    let event_cursor = runtime
        .presentation_events_since(epoch, 0)
        .unwrap()
        .iter()
        .map(|event| event.event_id)
        .max()
        .unwrap_or(0);
    let wall_start = std::time::Instant::now();
    let mut observed = runtime
        .submit_action(ActionCommandRequest {
            protocol_version: 2,
            world_epoch: epoch,
            request_id: NEXT_COMBAT_REQUEST.fetch_add(1, Ordering::Relaxed),
            client_time_ms: NEXT_TIME.fetch_add(17, Ordering::Relaxed),
            kind: ActionKind::ContextTraversal,
        })
        .unwrap();
    assert_eq!(observed.world_epoch, epoch);
    let mut started = BTreeSet::new();
    let mut completed = BTreeSet::new();
    let mut blocked = BTreeSet::new();

    for _ in 0..300 {
        observed = runtime.snapshot().unwrap();
        assert_eq!(observed.world_epoch, epoch, "ContextTraversal stays in its originating epoch");
        for event in runtime.presentation_events_since(epoch, event_cursor).unwrap() {
            if event.event_id <= event_cursor || event.world_epoch != epoch {
                continue;
            }
            match event.kind.as_str() {
                "ContextTraversalStarted" => { started.insert(event.event_id); }
                "ContextTraversalCompleted" => { completed.insert(event.event_id); }
                "ContextTraversalBlocked" => { blocked.insert(event.event_id); }
                _ => {}
            }
        }
        if !completed.is_empty() {
            break;
        }
        if observed.server_time_ms.saturating_sub(owner_start_ms) >= MAX_OWNER_TIME_MS
            || wall_start.elapsed() >= std::time::Duration::from_secs(3)
        {
            break;
        }
        observed = input(runtime, &observed, (0.0, 0.0), aim);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let elapsed_owner_ms = observed.server_time_ms.saturating_sub(owner_start_ms);
    assert_eq!(started.len(), 1, "one new authoritative ContextTraversalStarted must follow the cursor");
    assert_eq!(completed.len(), 1, "one new authoritative ContextTraversalCompleted must follow the cursor");
    assert!(blocked.is_empty(), "the selected authored traversal cannot be blocked");
    assert!(started.iter().next().unwrap() < completed.iter().next().unwrap(),
        "ContextTraversalStarted precedes ContextTraversalCompleted");
    assert!(elapsed_owner_ms <= MAX_OWNER_TIME_MS,
        "ContextTraversal must finish within three seconds of owner time: {elapsed_owner_ms}ms");
    assert!(wall_start.elapsed() <= std::time::Duration::from_secs(3),
        "ContextTraversal must finish within three seconds of wall time");
    (observed, start, elapsed_owner_ms)
}

#[test]
fn genuine_first_revisit_uses_earned_air_step_shortcut() {
    let save_dir = save_root();
    assert!(!save_dir.exists(), "the task-owned revisit save directory must be fresh");
    std::fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_dir.join("formal-save-v6.json");
    std::fs::write(&save_path, EARNED_THREE_WORLD_RETURN_STATION_V6.as_bytes()).unwrap();
    let fixture_bytes = EARNED_THREE_WORLD_RETURN_STATION_V6.as_bytes().to_vec();
    assert_eq!(std::fs::read(&save_path).unwrap(), fixture_bytes,
        "the scenario begins with the exact admitted Return Station SaveV6");

    let entry_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(entry_save.save.world_id, "return_station");
    assert_eq!(entry_save.save.scene_id, "rs_core_room");
    assert_eq!(entry_save.save.player.position_m.x_m, 4.0);
    assert_eq!(entry_save.save.player.position_m.y_m, 0.0);
    assert_eq!(entry_save.save.player.position_m.z_m, 8.0);
    assert_eq!(entry_save.save.player.current_hp, 46);
    assert_eq!(entry_save.save.player.current_energy, 28);
    assert_eq!(entry_save.save.server_time_ms, 747_048);
    assert_eq!(entry_save.save.checkpoint_id, None);
    let completed_events = BTreeSet::from([
        "clockworks_valves".to_owned(),
        "clockworks_core".to_owned(),
        "clockworks_shutdown".to_owned(),
    ]);
    let initial_clockworks = entry_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").expect("completed Clockworks progression is saved");
    assert!(initial_clockworks.completed && initial_clockworks.first_completion);
    assert_eq!(initial_clockworks.visit_id, 1);
    assert_eq!(initial_clockworks.cycle_id, 1);
    assert_eq!(initial_clockworks.revisit_count, 0);
    assert_eq!(initial_clockworks.completed_events.iter().cloned().collect::<BTreeSet<_>>(), completed_events);
    let initial_gh = entry_save.save.progression.progress.iter()
        .find(|row| row.world_id == "grey_hive").unwrap().clone();
    let initial_mh = entry_save.save.progression.progress.iter()
        .find(|row| row.world_id == "mist_harbor").unwrap().clone();
    assert!(initial_gh.completed && initial_gh.first_completion);
    assert!(initial_mh.completed && initial_mh.first_completion);
    let initial_effect_sources = entry_save.effect_sources.clone();
    let initial_grants = entry_save.save.capabilities.grants.clone();
    let initial_selection = entry_save.save.capabilities.selected.clone();
    assert_eq!(initial_effect_sources.iter().filter(|source| source.source_id == CAP_AIR_STEP).count(), 1);
    assert_eq!(initial_grants.iter().filter(|grant| grant.capability_id == CAP_AIR_STEP).count(), 1);
    assert!(initial_selection.iter().any(|capability| capability == CAP_AIR_STEP));
    assert!(entry_save.world_persistent_v1.clockworks.core_console_confirmed);
    assert_eq!(entry_save.world_persistent_v1.clockworks.pressure_valve_ids.len(), 3);

    // Requests advance beyond the seed's client timestamp without changing its saved clock.
    NEXT_TIME.fetch_max(entry_save.save.last_client_time_ms.saturating_add(1), Ordering::Relaxed);
    assert!(NEXT_TIME.load(Ordering::Relaxed) > entry_save.save.last_client_time_ms);
    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    let token = prepared.entry_token.as_ref().expect("normal Return Station readiness is required");
    assert_eq!(token.world_id, "return_station");
    assert_eq!(token.scene_id, "rs_core_room");
    assert_eq!(token.world_epoch, prepared.world_epoch);
    assert_eq!(prepared.player.transform.position_m.x_m, entry_save.save.player.position_m.x_m);
    assert_eq!(prepared.player.transform.position_m.y_m, entry_save.save.player.position_m.y_m);
    assert_eq!(prepared.player.transform.position_m.z_m, entry_save.save.player.position_m.z_m);
    assert_eq!(prepared.player.current_hp, entry_save.save.player.current_hp);
    assert_eq!(prepared.player.current_energy, entry_save.save.player.current_energy);
    assert_eq!(prepared.server_time_ms, entry_save.save.server_time_ms);
    assert_eq!(prepared.checkpoint_id, entry_save.save.checkpoint_id);
    assert_eq!(std::fs::read(&save_path).unwrap(), fixture_bytes,
        "Return Station Continue preparation cannot rewrite the admitted input");
    let ready = acknowledge_ready(&runtime, prepared);
    assert_eq!(ready.world_id, "return_station");
    assert_eq!(ready.scene_id, "rs_core_room");
    assert_eq!(ready.player.current_hp, 46);
    assert_eq!(ready.player.current_energy, 28);
    assert_eq!(ready.server_time_ms, 747_048);
    assert_eq!(std::fs::read(&save_path).unwrap(), fixture_bytes,
        "Return Station readiness cannot rewrite the admitted input");

    let pre_rest_hp = ready.player.current_hp;
    let pre_rest_energy = ready.player.current_energy;
    let mut view = rest_and_enter(&runtime, "earned-first-cw-revisit");
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_entry_foundry");
    assert!(view.player.current_hp > 0);
    println!("Native Return Station rest and genuine gate entry: HP {pre_rest_hp}->{} and energy {pre_rest_energy}->{}; {}ms",
        view.player.current_hp, view.player.current_energy, view.server_time_ms.saturating_sub(ready.server_time_ms));
    let clockworks = view.progression.worlds.iter()
        .find(|row| row.world_id == "clockworks").expect("revisited Clockworks progression is projected");
    assert!(clockworks.completed && clockworks.first_completion);
    assert_eq!(clockworks.visit_id, 2);
    assert_eq!(clockworks.cycle_id, 1);
    assert_eq!(clockworks.revisit_count, 1);
    assert_eq!(clockworks.completed_events.iter().cloned().collect::<BTreeSet<_>>(), completed_events);
    assert!(view.capabilities.items.iter().any(|item|
        item.capability_id == CAP_AIR_STEP && item.granted && item.selected),
        "the original earned AirStep remains both granted and selected on revisit");
    for (world_id, expected) in [("grey_hive", &initial_gh), ("mist_harbor", &initial_mh)] {
        let prior = view.progression.worlds.iter().find(|row| row.world_id == world_id)
            .unwrap_or_else(|| panic!("missing retained {world_id} progress"));
        assert!(prior.completed && prior.first_completion);
        assert_eq!(prior.visit_id, expected.visit_id);
        assert_eq!(prior.cycle_id, expected.cycle_id);
        assert_eq!(prior.revisit_count, expected.revisit_count);
        assert_eq!(prior.completed_events, expected.completed_events);
    }

    // The gate entry itself is typed and durable; no route award is repeated.
    runtime.save().unwrap();
    let gate_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(gate_save.save.scene_id, "cw_entry_foundry");
    let gate_bytes = std::fs::read(&save_path).unwrap();
    assert_ne!(gate_bytes, fixture_bytes, "the genuine gate-entry save replaces the admitted Return Station seed");
    let gate_progress = gate_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").unwrap();
    assert!(gate_progress.completed && gate_progress.first_completion);
    assert_eq!(gate_progress.visit_id, 2);
    assert_eq!(gate_progress.cycle_id, 1);
    assert_eq!(gate_progress.revisit_count, 1);
    assert_eq!(gate_progress.completed_events.iter().cloned().collect::<BTreeSet<_>>(), completed_events);
    assert_eq!(gate_save.effect_sources, initial_effect_sources);
    assert_eq!(gate_save.save.capabilities.grants, initial_grants);
    assert_eq!(gate_save.save.capabilities.selected, initial_selection);
    assert_eq!(gate_save.effect_sources.iter().filter(|source| source.source_id == CAP_AIR_STEP).count(), 1);
    assert_eq!(gate_save.save.capabilities.grants.iter().filter(|grant| grant.capability_id == CAP_AIR_STEP).count(), 1);

    // Reach Gear on the public, registered route. Each encountered roster is recorded and
    // cleared only through the existing live ordinary-combat helper.
    for &(scene_id, transition_id) in ROUTE.iter().take(4) {
        assert_eq!(view.scene_id, scene_id);
        let enemies_before = active_enemy_ids(&view);
        println!("Revisit route {scene_id}: {} active enemies at {} HP/{} energy",
            enemies_before.len(), view.player.current_hp, view.player.current_energy);
        if scene_id != "cw_entry_foundry" {
            assert!(!enemies_before.is_empty(), "the revisit must encounter the authored {scene_id} roster");
        }
        view = clear_live_enemies(&runtime, view);
        assert!(view.player.current_hp > 0, "the ordinary {scene_id} encounter ends alive");
        assert!(active_enemy_ids(&view).is_empty(), "the actual {scene_id} ordinary roster is cleared");
        println!("Revisit route {scene_id} cleared: {} HP/{} energy",
            view.player.current_hp, view.player.current_energy);

        if scene_id == "cw_conveyor_bridge" {
            let conveyor_scene = compiled_route_scene(scene_id);
            let vault = conveyor_scene["logic"]["traversal"].as_array().unwrap().iter()
                .find(|marker| marker["id"] == "cw_conveyor_context_vault")
                .expect("the registered Conveyor vault is authored");
            assert!(vault["requiredCapabilities"].as_array().unwrap().is_empty(),
                "only the existing capability-free Conveyor vault supplies this route step");
            let at_vault = walk_to_authored_traversal_start(
                &runtime, view, scene_id, "cw_conveyor_context_vault");
            let floor_start = at_vault.player.transform.position_m;
            assert!((0.0..=0.1).contains(&floor_start.y_m));
            let (vault_view, start, elapsed) = submit_and_confirm_public_context_traversal(
                &runtime, &at_vault, (1.0, 0.0));
            assert_eq!(vault_view.scene_id, "cw_conveyor_bridge");
            assert!(vault_view.player.current_hp > 0);
            let floor_end = vault_view.player.transform.position_m;
            assert!((0.0..=0.1).contains(&floor_end.y_m), "the vault lands on its lower floor");
            assert!(floor_end.x_m - start.0 >= 1.0,
                "the completed vault has real forward displacement: {start:?} -> {:?}",
                (floor_end.x_m, floor_end.y_m, floor_end.z_m));
            println!("Capability-free Conveyor vault completed in {elapsed} owner ms with {:.2}m forward movement",
                floor_end.x_m - start.0);
            view = vault_view;
        }

        view = walk_to_compiled_target(&runtime, view, scene_id, "transitions", transition_id);
        view = apply_transition(&runtime, transition_id,
            &format!("earned-revisit-{transition_id}"), view.world_epoch);
    }

    assert_eq!(view.scene_id, "cw_gear_shaft");
    let gear_enemies = active_enemy_ids(&view);
    assert!(!gear_enemies.is_empty(), "the actual Gear ordinary roster spawns on entry");
    println!("Gear revisit entry: {} authored enemies at {} HP/{} energy",
        gear_enemies.len(), view.player.current_hp, view.player.current_energy);
    view = clear_live_enemies(&runtime, view);
    assert!(view.player.current_hp > 0);
    assert!(active_enemy_ids(&view).is_empty());
    println!("Gear ordinary encounter cleared alive at {} HP/{} energy",
        view.player.current_hp, view.player.current_energy);

    // Walk around the south edge and approach from the lower west side; never ride the lift.
    view = gear_walk(&runtime, view, 12.0, 11.0);
    assert!((0.0..=0.1).contains(&view.player.transform.position_m.y_m));
    view = gear_walk(&runtime, view, 9.9, 11.0);
    view = gear_walk(&runtime, view, 9.9, 8.0);
    let lower_position = view.player.transform.position_m;
    assert!((0.0..=0.1).contains(&lower_position.y_m), "AirStep must start on the lower floor");
    assert!(!view.support_scene.as_ref().is_some_and(|support|
        support.rider.support_id.as_deref() == Some("cw_gear_main_lift")),
        "the approach cannot ride the main Gear lift");
    let gear_scene = compiled_route_scene("cw_gear_shaft");
    let shortcut = gear_scene["logic"]["traversal"].as_array().unwrap().iter()
        .find(|marker| marker["id"] == "cw_gear_earned_air_step")
        .expect("the earned ground-to-upper-dock AirStep marker is authored");
    let from = shortcut["from"].as_array().unwrap();
    let from_x = from[0].as_f64().unwrap() as f32;
    let from_z = from[2].as_f64().unwrap() as f32;
    let from_height = shortcut["fromHeightRangeM"].as_array().unwrap();
    let min_height = from_height[0].as_f64().unwrap() as f32;
    let max_height = from_height[1].as_f64().unwrap() as f32;
    let range = shortcut["rangeM"].as_f64().unwrap() as f32;
    let distance = (from_x - lower_position.x_m).hypot(from_z - lower_position.z_m);
    assert!((min_height..=max_height).contains(&lower_position.y_m),
        "the actual living body begins within the authored ground height range");
    assert!(distance <= range,
        "the actual ground position is in range of the compiled AirStep marker: {distance}m/{range}m");
    let required = shortcut["requiredCapabilities"].as_array().unwrap().iter()
        .filter_map(serde_json::Value::as_str).collect::<BTreeSet<_>>();
    assert_eq!(required, BTreeSet::from([CAP_AIR_STEP]));
    assert!(view.capabilities.items.iter().any(|item|
        item.capability_id == CAP_AIR_STEP && item.granted && item.selected),
        "the actual revisited owner still has the earned and selected AirStep");

    let air_step_hp = view.player.current_hp;
    let air_step_energy = view.player.current_energy;
    let (upper_view, air_step_start, elapsed) =
        submit_and_confirm_public_context_traversal(&runtime, &view, (1.0, 0.0));
    assert_eq!(upper_view.scene_id, "cw_gear_shaft");
    assert!(upper_view.player.current_hp > 0);
    let upper_position = upper_view.player.transform.position_m;
    assert!((1.9..=2.1).contains(&upper_position.y_m),
        "the one real earned AirStep moves the body from ground to the two-meter dock: {air_step_start:?} -> ({}, {}, {})",
        upper_position.x_m, upper_position.y_m, upper_position.z_m);
    assert!(upper_position.x_m - air_step_start.0 >= 1.0,
        "the real AirStep has forward displacement: {air_step_start:?} -> ({}, {}, {})",
        upper_position.x_m, upper_position.y_m, upper_position.z_m);
    let dock = gear_scene["standingDecks"].as_array().unwrap().iter()
        .find(|surface| surface["id"] == "cw_gear_upper_dock")
        .expect("the two-meter upper dock is an authored standing surface");
    assert_eq!(dock["heightM"].as_f64().unwrap(), 2.0);
    let support = upper_view.support_scene.as_ref().expect("Gear support contact is projected");
    assert_eq!(support.rider.mode,
        wuxian_horror_ch1::moving_support::SupportRiderMode::Surface);
    assert_eq!(support.rider.support_id.as_deref(), Some("cw_gear_upper_dock"));
    assert!((support.rider.position_m.y_m - upper_position.y_m).abs() <= 0.001);
    let upper_pose = support.poses.iter().find(|pose| pose.support_id == "cw_gear_upper_dock")
        .expect("the completed AirStep is supported by the authored upper dock");
    assert!((upper_pose.height_m - 2.0).abs() <= 0.001);
    println!("Earned AirStep revisit: ground ({:.2}, {:.2}, {:.2}) to upper dock ({:.2}, {:.2}, {:.2}) in {elapsed} owner ms; HP {air_step_hp}->{}, energy {air_step_energy}->{}",
        air_step_start.0, air_step_start.1, air_step_start.2,
        upper_position.x_m, upper_position.y_m, upper_position.z_m,
        upper_view.player.current_hp, upper_view.player.current_energy);

    // Pause and save the actual living AirStep result. Do not continue into the Furnace route.
    let final_view = runtime.pause().unwrap();
    assert_eq!(final_view.scene_id, "cw_gear_shaft");
    assert!(final_view.player.current_hp > 0);
    assert!((1.9..=2.1).contains(&final_view.player.transform.position_m.y_m));
    let final_position = final_view.player.transform.position_m;
    let final_checkpoint = final_view.checkpoint_id.clone();
    runtime.save().unwrap();
    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(final_save.save.scene_id, "cw_gear_shaft");
    assert_eq!(final_save.save.player.position_m.x_m, final_position.x_m);
    assert_eq!(final_save.save.player.position_m.y_m, final_position.y_m);
    assert_eq!(final_save.save.player.position_m.z_m, final_position.z_m);
    assert_eq!(final_save.save.player.current_hp, final_view.player.current_hp);
    assert_eq!(final_save.save.player.current_energy, final_view.player.current_energy);
    assert_eq!(final_save.save.server_time_ms, final_view.server_time_ms);
    assert_eq!(final_save.save.checkpoint_id, final_checkpoint,
        "the actual checkpoint outcome is retained without earning one in the test");
    assert!(final_save.save.moving_support_frame.as_ref().is_some_and(|frame|
        frame.poses.iter().any(|pose| pose.support_id == "cw_gear_upper_dock"
            && (pose.height_m - 2.0).abs() <= 0.001)),
        "the paused saved body retains its actual upper-dock support frame");
    let final_clockworks = final_save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks").expect("final Clockworks state is saved");
    assert!(final_clockworks.completed && final_clockworks.first_completion);
    assert_eq!(final_clockworks.visit_id, 2);
    assert_eq!(final_clockworks.cycle_id, 1);
    assert_eq!(final_clockworks.revisit_count, 1);
    assert_eq!(final_clockworks.completed_events.iter().cloned().collect::<BTreeSet<_>>(), completed_events);
    assert_eq!(final_save.save.progression.progress.iter()
        .find(|row| row.world_id == "grey_hive").unwrap(), &initial_gh,
        "Grey Hive progression remains byte-semantic-equivalent to the admitted save");
    assert_eq!(final_save.save.progression.progress.iter()
        .find(|row| row.world_id == "mist_harbor").unwrap(), &initial_mh,
        "Mist Harbor progression remains byte-semantic-equivalent to the admitted save");
    assert_eq!(final_save.effect_sources, initial_effect_sources,
        "AirStep use on revisit cannot duplicate or change earned effect sources");
    assert_eq!(final_save.save.capabilities.grants, initial_grants,
        "AirStep use on revisit cannot duplicate or change capability grants");
    assert_eq!(final_save.save.capabilities.selected, initial_selection,
        "AirStep remains selected as in the admitted earned save");
    assert_eq!(final_save.effect_sources.iter().filter(|source| source.source_id == CAP_AIR_STEP).count(), 1);
    assert_eq!(final_save.save.capabilities.grants.iter().filter(|grant| grant.capability_id == CAP_AIR_STEP).count(), 1);
    assert!(final_save.save.capabilities.selected.iter().any(|capability| capability == CAP_AIR_STEP));
    println!("Paused earned AirStep revisit saved at {}: {} HP/{} energy, {}ms, checkpoint {:?}",
        save_path.display(), final_view.player.current_hp, final_view.player.current_energy,
        final_view.server_time_ms, final_checkpoint);
    drop(runtime);
}