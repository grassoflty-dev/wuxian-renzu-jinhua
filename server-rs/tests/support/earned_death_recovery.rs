use super::*;

const EARNED_CW_REGULATOR_ENTRY_V6: &str =
    include_str!("../fixtures/earned-cw-regulator-entry-v6.json");

fn assert_earned_regulator_route_save(save: &save_v6::SaveV6) {
    assert_eq!(save.save.world_id, "clockworks");
    assert_eq!(save.save.scene_id, "cw_regulator_core");
    let clockworks = save.save.progression.progress.iter()
        .find(|row| row.world_id == "clockworks")
        .expect("Clockworks progression is present in the typed save");
    assert_eq!(clockworks.visit_id, 1);
    assert_eq!(clockworks.cycle_id, 1);
    assert_eq!(clockworks.revisit_count, 0);
    assert!(!clockworks.completed && !clockworks.first_completion);
    assert_eq!(clockworks.completed_events, vec!["clockworks_valves"]);
    let grey_hive = save.save.progression.progress.iter()
        .find(|row| row.world_id == "grey_hive").unwrap();
    assert!(grey_hive.completed && grey_hive.first_completion);
    assert_eq!(grey_hive.completed_events,
        vec!["hive_power", "hive_lockdown", "hive_extraction"]);
    let mist_harbor = save.save.progression.progress.iter()
        .find(|row| row.world_id == "mist_harbor").unwrap();
    assert!(mist_harbor.completed && mist_harbor.first_completion);
    assert_eq!(mist_harbor.completed_events,
        vec!["mist_beacon_west", "mist_beacon_east", "mist_signal"]);
    assert_eq!(save.world_persistent_v1.clockworks.pressure_valve_ids, vec![
        "cw_pressure_valve_01_staged".to_owned(),
        "cw_pressure_valve_02_staged".to_owned(),
        "cw_pressure_valve_03_staged".to_owned(),
    ]);
    assert!(save.world_persistent_v1.clockworks.forged_guard_elite_first_kill);
    assert!(!save.world_persistent_v1.clockworks.regulator_defeated);
    assert!(!save.world_persistent_v1.clockworks.core_console_confirmed);
    assert!(!save.save.capabilities.grants.iter()
        .any(|grant| grant.capability_id == wuxian_horror_ch1::capability_v1::CAP_AIR_STEP));
}

fn assert_public_regulator_vitals_healthy(view: &wuxian_horror_ch1::world_v3::WorldView) {
    let vitals = view.capabilities.enemy_vitals.iter()
        .find(|vitals| vitals.entity_id == "cw_prime_regulator")
        .expect("earned EnemyVitals capability projects the active regulator");
    assert_eq!(vitals.tier, wuxian_horror_ch1::world_v3::VitalTier::Healthy);
}
fn assert_earned_regulator_route_view(view: &wuxian_horror_ch1::world_v3::WorldView) {
    assert_eq!(view.world_id, "clockworks");
    assert_eq!(view.scene_id, "cw_regulator_core");
    assert_eq!(view.progression.current_world_id, "clockworks");
    let clockworks = view.progression.worlds.iter()
        .find(|row| row.world_id == "clockworks").unwrap();
    assert_eq!(clockworks.visit_id, 1);
    assert_eq!(clockworks.cycle_id, 1);
    assert_eq!(clockworks.revisit_count, 0);
    assert!(!clockworks.completed && !clockworks.first_completion);
    assert_eq!(clockworks.completed_events, vec!["clockworks_valves"]);
    let actor = view.actors.iter()
        .find(|actor| actor.entity_id == "cw_prime_regulator")
        .expect("the public view contains the authored regulator");
    assert_eq!(actor.actor_kind, "enemy");
    assert!(actor.active);
}

#[test]
fn genuine_earned_regulator_death_continues_last_living_save() {
    const MAX_WALL: std::time::Duration = std::time::Duration::from_secs(120);
    const MAX_OWNER_TIME_MS: u64 = 120_000;
    const MAX_INPUTS: usize = 7_200;
    const INTER_INPUT_DELAY: std::time::Duration = std::time::Duration::from_millis(16);

    let save_dir = save_root();
    assert!(!save_dir.exists(), "task-owned save directory must be fresh");
    fs::create_dir_all(&save_dir).unwrap();
    let save_path = save_dir.join("formal-save-v6.json");
    fs::write(&save_path, EARNED_CW_REGULATOR_ENTRY_V6.as_bytes()).unwrap();
    let living_bytes = fs::read(&save_path).unwrap();
    assert_eq!(living_bytes, EARNED_CW_REGULATOR_ENTRY_V6.as_bytes());

    let living_save = save_v6::read_save(&save_dir).unwrap();
    assert_earned_regulator_route_save(&living_save);
    assert_eq!(living_save.save.checkpoint_id, None);
    assert_eq!(living_save.save.player.current_hp, 46);
    assert_eq!(living_save.save.player.current_energy, 28);
    assert_eq!(living_save.save.player.position_m.x_m, 3.0);
    assert_eq!(living_save.save.player.position_m.y_m, 0.0);
    assert_eq!(living_save.save.player.position_m.z_m, 8.0);
    assert_eq!(living_save.save.server_time_ms, 658_274);
    assert_eq!(living_save.save.last_client_time_ms, 0);
    assert_eq!(living_save.save.generic_actors.len(), 1);
    assert_eq!(living_save.save.generic_actors[0].entity_id, "cw_prime_regulator");
    assert_eq!(living_save.save.generic_actors[0].entity_type,
        "enemy.clockworks.prime_regulator");
    assert_eq!(living_save.save.generic_actors[0].hp, 600);
    assert!(living_save.effect_sources.iter()
        .any(|source| source.source_id == "information.enemy_vitals_basic"));
    assert!(living_save.effect_sources.iter()
        .any(|source| source.source_id == "perception.rear_view_i"));
    let baseline_persistent = living_save.world_persistent_v1.clone();
    let baseline_progression = living_save.save.progression.clone();
    let baseline_effect_sources = living_save.effect_sources.clone();
    let baseline_grants = living_save.save.capabilities.grants.clone();

    let runtime = FormalRuntime::new_with_save_dir(save_dir.clone()).unwrap();
    production::install(&runtime).unwrap();
    let prepared = runtime.continue_saved().unwrap();
    let entry_epoch = prepared.world_epoch;
    let entry_token = prepared.entry_token.as_ref().expect("normal Continue entry token");
    assert_eq!(entry_token.world_id, "clockworks");
    assert_eq!(entry_token.scene_id, "cw_regulator_core");
    assert_eq!(entry_token.world_epoch, entry_epoch);
    assert_earned_regulator_route_view(&prepared);
    assert_eq!(prepared.player.current_hp, 46);
    assert_eq!(prepared.player.current_energy, 28);
    assert_eq!(prepared.player.transform.position_m.x_m, 3.0);
    assert_eq!(prepared.player.transform.position_m.y_m, 0.0);
    assert_eq!(prepared.player.transform.position_m.z_m, 8.0);
    assert_eq!(prepared.server_time_ms, 658_274);
    assert_eq!(prepared.checkpoint_id, None);
    assert_eq!(fs::read(&save_path).unwrap(), living_bytes,
        "Continue preparation preserves the exact living SaveV6 bytes");
    assert!(runtime.presentation_events_since(entry_epoch, 0).unwrap().iter().all(|event|
        !matches!(event.kind.as_str(), "PlayerDeath" | "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")));
    let mut view = acknowledge_ready(&runtime, prepared);
    assert_earned_regulator_route_view(&view);
    assert_public_regulator_vitals_healthy(&view);
    assert_eq!(view.player.current_hp, 46);
    assert_eq!(view.player.current_energy, 28);
    assert_eq!(view.server_time_ms, 658_274);

    let mut event_cursor = runtime.presentation_events_since(entry_epoch, 0).unwrap().iter()
        .map(|event| event.event_id).max().unwrap_or(0);
    let mut event_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut hp_trace = vec![view.player.current_hp];
    let mut reached_attack_range = false;
    let mut death_confirmed = false;
    let started = std::time::Instant::now();
    let mut inputs = 0usize;

    for index in 0..MAX_INPUTS {
        assert!(started.elapsed() < MAX_WALL, "authored-combat wall-time bound reached");
        assert!(view.server_time_ms.saturating_sub(658_274) <= MAX_OWNER_TIME_MS,
            "authored-combat owner-time bound reached");
        assert!(view.player.current_hp > 0, "unexpected dead snapshot before public input {index}");

        let target = view.actors.iter()
            .find(|actor| actor.entity_id == "cw_prime_regulator" && actor.active)
            .expect("the authored regulator remains present during the death scenario");
        let player = view.player.transform.position_m;
        let target_position = target.transform.position_m;
        let dx = target_position.x_m - player.x_m;
        let dz = target_position.z_m - player.z_m;
        let distance = dx.hypot(dz);
        let (move_x, move_z) = if reached_attack_range {
            (0.0, 0.0)
        } else if distance <= 2.0 {
            reached_attack_range = true;
            (0.0, 0.0)
        } else {
            let distance = distance.max(f32::EPSILON);
            (dx / distance, dz / distance)
        };

        let floor = living_save.save.last_client_time_ms.saturating_add(1);
        let _ = NEXT_TIME.fetch_max(floor, Ordering::Relaxed);
        let client_time_ms = NEXT_TIME.fetch_add(17, Ordering::Relaxed).max(floor);
        let sample = InputSample::new(
            view.world_epoch,
            view.ack_seq.saturating_add(1),
            client_time_ms,
            move_x,
            move_z,
        ).unwrap();
        inputs = index + 1;
        view = match runtime.submit_input(sample, vec![]) {
            Ok(next) => next,
            Err(error) if error == "E_RUNTIME_DEAD" => {
                let dead = runtime.snapshot().expect("public dead snapshot after owner death");
                assert_eq!(dead.player.current_hp, 0,
                    "E_RUNTIME_DEAD is accepted only with an actual zero-HP snapshot");
                dead
            }
            Err(error) => panic!("public movement input failed: {error}"),
        };
        hp_trace.push(view.player.current_hp);

        let fresh_events = runtime.presentation_events_since(entry_epoch, event_cursor)
            .expect("fresh-cursor public presentation events");
        for event in &fresh_events {
            if event.world_epoch == entry_epoch {
                *event_counts.entry(event.kind.clone()).or_default() += 1;
            }
        }
        if let Some(last) = fresh_events.iter().map(|event| event.event_id).max() {
            event_cursor = last;
        }
        if view.player.current_hp == 0 {
            death_confirmed = true;
            break;
        }
        std::thread::sleep(INTER_INPUT_DELAY);
    }

    assert!(reached_attack_range, "the player must reach the authored regulator within 2m");
    assert!(death_confirmed, "the player's real HP must reach zero within the bounds");
    assert!(view.player.current_hp == 0 && hp_trace.last() == Some(&0));
    assert!(hp_trace.iter().any(|hp| *hp < 46), "authored AI must actually damage the player");
    assert!(event_counts.get("PrimeRegulatorPressureWindup").copied().unwrap_or(0) > 0,
        "the fresh public cursor observes the regulator's authored warning");
    assert!(event_counts.get("PrimeRegulatorPressureImpact").copied().unwrap_or(0) > 0,
        "the fresh public cursor observes the regulator's authored impact");
    assert_eq!(event_counts.get("PlayerDeath").copied().unwrap_or(0), 1,
        "exactly one same-epoch public PlayerDeath follows the fresh cursor");
    assert!(view.server_time_ms.saturating_sub(658_274) <= MAX_OWNER_TIME_MS);
    let death_view = runtime.snapshot().unwrap();
    assert_eq!(death_view.world_epoch, entry_epoch);
    assert_eq!(death_view.player.current_hp, 0);
    assert_eq!(fs::read(&save_path).unwrap(), living_bytes,
        "real-time death does not alter the last living SaveV6");
    assert_eq!(runtime.save().unwrap_err(), "E_RUNTIME_DEAD");
    assert_eq!(fs::read(&save_path).unwrap(), living_bytes,
        "rejected dead save preserves the last living SaveV6 bytes");
    std::thread::sleep(std::time::Duration::from_millis(100));
    let quiescent = runtime.snapshot().unwrap();
    assert_eq!(quiescent.world_epoch, entry_epoch);
    assert_eq!(quiescent.player.current_hp, 0);
    assert_eq!(quiescent.server_tick, death_view.server_tick,
        "the dead owner remains quiescent without replaying more combat");
    assert_eq!(runtime.presentation_events_since(entry_epoch, event_cursor).unwrap().iter()
        .filter(|event| event.kind == "PlayerDeath").count(), 0,
        "the quiescent owner does not replay PlayerDeath");
    assert_eq!(fs::read(&save_path).unwrap(), living_bytes);

    let continued = runtime.continue_saved().unwrap();
    assert_ne!(continued.world_epoch, entry_epoch,
        "the same dead owner restores the last living file into a fresh epoch");
    let continued_epoch = continued.world_epoch;
    let token = continued.entry_token.as_ref().expect("continued living entry token");
    assert_eq!(token.world_id, "clockworks");
    assert_eq!(token.scene_id, "cw_regulator_core");
    assert_eq!(token.world_epoch, continued.world_epoch);
    assert_eq!(continued.scene_id, "cw_regulator_core");
    assert_earned_regulator_route_view(&continued);
    assert_eq!(continued.player.transform.position_m.x_m, 3.0);
    assert_eq!(continued.player.transform.position_m.y_m, 0.0);
    assert_eq!(continued.player.transform.position_m.z_m, 8.0);
    assert_eq!(continued.player.current_hp, 46);
    assert_eq!(continued.player.current_energy, 28);
    assert_eq!(continued.server_time_ms, 658_274);
    assert_eq!(continued.checkpoint_id, None);
    assert_eq!(fs::read(&save_path).unwrap(), living_bytes,
        "same-owner Continue leaves the last living SaveV6 bytes intact");
    let _ready = acknowledge_ready(&runtime, continued);
    let ready = runtime.pause().unwrap();
    assert_public_regulator_vitals_healthy(&ready);
    assert_eq!(ready.world_epoch, continued_epoch);
    assert_eq!(ready.scene_id, "cw_regulator_core");
    assert_eq!(ready.player.current_hp, 46);
    assert_eq!(ready.player.current_energy, 28);
    assert_eq!(ready.server_time_ms, 658_274);
    assert_eq!(ready.checkpoint_id, None);
    assert_earned_regulator_route_view(&ready);
    let continued_events = runtime.presentation_events_since(ready.world_epoch, 0).unwrap();
    assert!(continued_events.iter().all(|event|
        !matches!(event.kind.as_str(), "PlayerDeath" | "EnemyDeath" | "ForgedGuardEliteDeath" | "PrimeRegulatorDeath")),
        "new-epoch Continue/readiness does not replay the prior death or a defeat");

    let final_save = save_v6::read_save(&save_dir).unwrap();
    assert_eq!(final_save.world_persistent_v1, baseline_persistent);
    assert_eq!(final_save.save.progression, baseline_progression);
    assert_eq!(final_save.effect_sources, baseline_effect_sources);
    assert_eq!(final_save.save.capabilities.grants, baseline_grants);
    assert_eq!(final_save, living_save);
    assert_eq!(fs::read(&save_path).unwrap(), living_bytes,
        "the task ends on the exact unchanged last living SaveV6");
    println!(
        "Earned regulator death and same-owner last-living Continue: {inputs} public inputs; HP trace {hp_trace:?}; warning/impact/death events {:?}; reached range {reached_attack_range}; restored {} HP/{} energy at {} ms; retained save {}",
        event_counts,
        ready.player.current_hp,
        ready.player.current_energy,
        ready.server_time_ms,
        save_path.display(),
    );
    drop(runtime);
}