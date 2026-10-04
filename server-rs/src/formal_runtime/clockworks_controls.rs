//! Durable source receipts for the three pressure valves and the Core console.
//! Route event names never manufacture interaction receipts on load/revisit.
use super::*;
use crate::world_persistent_v1::{ClockworksPersistentState, CLOCKWORKS_PRESSURE_VALVE_IDS};

const PRESSURE_SCENE_ID: &str = "cw_pressure_hall";
const VALVES_EVENT_ID: &str = "clockworks_valves";
const AGGREGATE_ID: &str = "cw_clockworks_valves_staged";
const CORE_EVENT_ID: &str = "clockworks_core";

fn validate_records(persistent: &ClockworksPersistentState) -> Result<(), String> {
    let ids = &persistent.pressure_valve_ids;
    if ids.len() > CLOCKWORKS_PRESSURE_VALVE_IDS.len()
        || ids.iter().any(|id| !CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id.as_str()))
        || ids.windows(2).any(|pair| pair[0] >= pair[1])
        || (persistent.core_console_confirmed && !persistent.regulator_defeated)
    {
        return Err("E_SAVE_CLOCKWORKS_CONTROL_RECORD_INVALID".into());
    }
    Ok(())
}

pub(crate) fn validate_saved(
    save: &crate::save_v5::SaveV5,
    persistent: &ClockworksPersistentState,
) -> Result<(), String> {
    validate_records(persistent)?;
    let events = route_completed_events(&save.progression, "clockworks");
    if (persistent.pressure_valve_ids.len() == CLOCKWORKS_PRESSURE_VALVE_IDS.len()
        && !events.contains(VALVES_EVENT_ID))
        || (persistent.core_console_confirmed && !events.contains(CORE_EVENT_ID))
    {
        return Err("E_SAVE_CLOCKWORKS_CONTROL_PROGRESS_INVALID".into());
    }
    // Missing new receipts remain missing, even when old event strings exist.
    Ok(())
}

fn validate_pressure_source(scene: &SceneRuntime) -> Result<(), String> {
    let definition = scene.current_scene();
    let aggregates: Vec<_> = definition.interaction_aggregates.iter().filter(|a| {
        a.event == VALVES_EVENT_ID || a.marker_id == AGGREGATE_ID
            || a.member_ids.iter().any(|id| CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id.as_str()))
    }).collect();
    let canonical: BTreeSet<_> = CLOCKWORKS_PRESSURE_VALVE_IDS.into_iter().collect();
    if definition.world_id != "clockworks" || definition.scene_id != PRESSURE_SCENE_ID
        || aggregates.len() != 1
        || aggregates[0].marker_id != AGGREGATE_ID
        || aggregates[0].event != VALVES_EVENT_ID
        || aggregates[0].member_ids.len() != canonical.len()
        || aggregates[0].member_ids.iter().map(String::as_str).collect::<BTreeSet<_>>() != canonical
        || !definition.interactions.iter().any(|i| i.id == AGGREGATE_ID
            && i.kind == "three_valve_sequence_staged_marker" && i.event.is_none())
        || CLOCKWORKS_PRESSURE_VALVE_IDS.iter().any(|id| !definition.interactions.iter()
            .any(|i| i.id == *id && i.kind == "valve_control" && i.event.is_none()))
        || definition.interactions.iter().any(|i| i.event.as_deref() == Some(VALVES_EVENT_ID))
        || definition.triggers.iter().any(|t| t.event == VALVES_EVENT_ID)
    {
        return Err("E_CLOCKWORKS_VALVES_SOURCE_INVALID".into());
    }
    Ok(())
}

/// Validate all valve-bearing events, not just the first generic route event.
pub(super) fn validate_pressure_event(
    state: &RuntimeState, scene: &SceneRuntime, events: &[SceneEvent],
) -> Result<(), String> {
    // Generic registry fixtures retain their historical route-event behavior,
    // but can never earn canonical source receipts from an event name alone.
    let strict_event_source = scene.is_complete_clockworks_production()
        || scene.current_scene().scene_id == PRESSURE_SCENE_ID;
    let related = events.iter().any(|event| match event {
        SceneEvent::Interaction { id, event_id } => CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id.as_str())
            || id == AGGREGATE_ID || (strict_event_source && event_id.as_deref() == Some(VALVES_EVENT_ID)),
        SceneEvent::Trigger { event_id, .. } => strict_event_source && event_id == VALVES_EVENT_ID,
        _ => false,
    });
    if !related { return Ok(()); }
    validate_records(&state.world_persistent_v1.clockworks)?;
    validate_pressure_source(scene)?;
    let [SceneEvent::Interaction { id, event_id }] = events else {
        return Err("E_CLOCKWORKS_VALVES_SOURCE_INVALID".into());
    };
    let persistent = &state.world_persistent_v1.clockworks;
    let activated_ids: BTreeSet<_> = CLOCKWORKS_PRESSURE_VALVE_IDS.into_iter()
        .filter(|id| scene.object_activated(id)).collect();
    let complete = activated_ids.len() == CLOCKWORKS_PRESSURE_VALVE_IDS.len();
    if state.world.world_id != "clockworks" || state.world.scene_id != PRESSURE_SCENE_ID
        || state.route.current_world_id != "clockworks"
        || scene.world_epoch != state.world.revision.world_epoch
        || !CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id.as_str())
        || persistent.pressure_valve_ids.contains(id)
        || !activated_ids.contains(id.as_str())
        || persistent.pressure_valve_ids.iter().any(|id| !activated_ids.contains(id.as_str()))
        || (event_id.as_deref() != complete.then_some(VALVES_EVENT_ID)
            && !(complete && event_id.is_none() && completed_world_events(state, "clockworks").contains(VALVES_EVENT_ID)))
        || scene.event_complete(VALVES_EVENT_ID) != complete
    {
        return Err("E_CLOCKWORKS_VALVES_SOURCE_INVALID".into());
    }
    Ok(())
}

/// Expose the same narrowly scoped physical reconfirmation through the normal
/// interaction projection. No ledger, request claim, or progress is changed.
pub(super) fn reconfirmation_available(state: &RuntimeState, scene: &SceneRuntime, id: &str) -> bool {
    let persistent = &state.world_persistent_v1.clockworks;
    if state.paused || state.world.player_hp == 0 || !scene.object_activated(id)
        || scene.world_epoch != state.world.revision.world_epoch
        || scene.current_scene().world_id != state.world.world_id
        || scene.scene_id != state.world.scene_id || state.route.current_world_id != "clockworks"
        || validate_records(persistent).is_err()
    {
        return false;
    }
    let eligible = if CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id) {
        !persistent.pressure_valve_ids.iter().any(|saved| saved == id)
            && validate_pressure_source(scene).is_ok()
    } else if id == clockworks_core::CORE_CONSOLE_ID {
        !persistent.core_console_confirmed && clockworks_core::validate_core_source(state, scene).is_ok()
    } else { false };
    if !eligible { return false; }
    let Some(item) = scene.current_scene().interactions.iter().find(|i| i.id == id) else { return false; };
    let position = state.world.player.position_m;
    let distance = ((position.x_m - item.position[0]).powi(2)
        + (position.y_m - item.position[1]).powi(2)
        + (position.z_m - item.position[2]).powi(2)).sqrt();
    distance.is_finite() && distance <= item.range_m.unwrap_or(2.5)
}

/// Old scene-local activations remain untouched. A missing durable receipt can
/// be reconfirmed only by a real, in-range, epoch-bound owner command.
pub(super) fn reconfirm_interaction(
    state: &RuntimeState, scene: &mut SceneRuntime, id: &str, request_id: &str, epoch: u64,
) -> Result<Option<Vec<SceneEvent>>, String> {
    let persistent = &state.world_persistent_v1.clockworks;
    let pressure = CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id);
    let core = id == clockworks_core::CORE_CONSOLE_ID;
    if !scene.object_activated(id) || (!pressure && !core)
        || (pressure && persistent.pressure_valve_ids.iter().any(|saved| saved == id))
        || (core && persistent.core_console_confirmed)
    {
        return Ok(None);
    }
    validate_records(persistent)?;
    if epoch != scene.world_epoch || epoch != state.world.revision.world_epoch {
        return Err("E_SCENE_RUNTIME_StaleEpoch".into());
    }
    if scene.current_scene().world_id != state.world.world_id
        || scene.scene_id != state.world.scene_id || state.route.current_world_id != "clockworks"
    {
        return Err("E_CLOCKWORKS_CONTROL_SCOPE_INVALID".into());
    }
    if pressure { validate_pressure_source(scene)?; }
    if core { clockworks_core::validate_core_source(state, scene)?; }
    let item = scene.current_scene().interactions.iter().find(|i| i.id == id)
        .ok_or("E_CLOCKWORKS_CONTROL_SOURCE_INVALID")?;
    let position = state.world.player.position_m;
    let distance = ((position.x_m - item.position[0]).powi(2)
        + (position.y_m - item.position[1]).powi(2)
        + (position.z_m - item.position[2]).powi(2)).sqrt();
    if !distance.is_finite() || distance > item.range_m.unwrap_or(2.5) {
        return Err("E_SCENE_RUNTIME_OutOfRange".into());
    }
    let event_id = if core { Some(CORE_EVENT_ID.to_owned()) }
        else { scene.event_complete(VALVES_EVENT_ID).then(|| VALVES_EVENT_ID.to_owned()) };
    scene.claim_command_request(request_id, epoch).map_err(|error| error.to_string())?;
    Ok(Some(vec![SceneEvent::Interaction { id: id.to_owned(), event_id }]))
}

pub(super) enum ControlReceipt { PressureValve(String), CoreConsole }

/// Only a fresh before/after interaction, already spatially/epoch checked by
/// SceneRuntime, can produce a receipt. The caller commits it with all effects.
pub(super) fn validated_receipt(
    state: &RuntimeState, before: &SceneRuntime, after: &SceneRuntime, events: &[SceneEvent],
    reconfirmed: bool,
) -> Result<Option<ControlReceipt>, String> {
    validate_pressure_event(state, after, events)?;
    clockworks_core::validate_core_event(state, after, events)?;
    let [SceneEvent::Interaction { id, .. }] = events else { return Ok(None); };
    let receipt = if CLOCKWORKS_PRESSURE_VALVE_IDS.contains(&id.as_str()) {
        Some(ControlReceipt::PressureValve(id.clone()))
    } else if id == clockworks_core::CORE_CONSOLE_ID {
        Some(ControlReceipt::CoreConsole)
    } else { None };
    if receipt.is_some() && (before.world_epoch != after.world_epoch
        || before.scene_id != after.scene_id || before.object_activated(id) != reconfirmed
        || !after.object_activated(id))
    {
        return Err("E_CLOCKWORKS_CONTROL_RECEIPT_INVALID".into());
    }
    Ok(receipt)
}

pub(super) fn record_receipt(state: &mut RuntimeState, receipt: Option<ControlReceipt>) {
    let persistent = &mut state.world_persistent_v1.clockworks;
    match receipt {
        Some(ControlReceipt::PressureValve(id)) => {
            persistent.pressure_valve_ids.push(id);
            persistent.pressure_valve_ids.sort();
        }
        Some(ControlReceipt::CoreConsole) => persistent.core_console_confirmed = true,
        None => {}
    }
}

/// Called only after source validators, before any duplicate route Progress.
pub(super) fn reconfirming_completed_event(state: &RuntimeState, event_id: &str) -> bool {
    let source_scene = match event_id {
        VALVES_EVENT_ID => PRESSURE_SCENE_ID,
        CORE_EVENT_ID => CLOCKWORKS_REGULATOR_SCENE_ID,
        _ => return false,
    };
    state.world.world_id == "clockworks" && state.world.scene_id == source_scene
        && completed_world_events(state, "clockworks").contains(event_id)
}

/// Restore local affordances silently from source records, never from route
/// event strings. Old local activations are preserved for physical reconfirmation.
pub(super) fn restore_scene(
    scene: &mut SceneRuntime, persistent: &ClockworksPersistentState,
) -> Result<(), String> {
    validate_records(persistent)?;
    if scene.current_scene().world_id != "clockworks" { return Ok(()); }
    if scene.scene_id == PRESSURE_SCENE_ID {
        validate_pressure_source(scene)?;
        let ids: Vec<_> = persistent.pressure_valve_ids.iter().map(String::as_str).collect();
        scene.restore_durable_interactions(&ids)
            .map_err(|_| "E_CLOCKWORKS_VALVES_SOURCE_INVALID")?;
    } else if scene.scene_id == CLOCKWORKS_REGULATOR_SCENE_ID {
        // Generic legacy fixtures without the official console remain unmodified.
        if !scene.current_scene().interactions.iter().any(|i| i.id == clockworks_core::CORE_CONSOLE_ID) {
            if persistent.core_console_confirmed { return Err("E_CLOCKWORKS_CORE_SOURCE_INVALID".into()); }
            return Ok(());
        }
        let definition = scene.current_scene();
        let consoles: Vec<_> = definition.interactions.iter().filter(|i|
            i.id == clockworks_core::CORE_CONSOLE_ID || i.event.as_deref() == Some(CORE_EVENT_ID)).collect();
        if consoles.len() != 1 || consoles[0].kind != "terminal"
            || consoles[0].event.as_deref() != Some(CORE_EVENT_ID)
            || definition.triggers.iter().any(|t| t.event == CORE_EVENT_ID)
            || definition.interaction_aggregates.iter().any(|a| a.event == CORE_EVENT_ID)
        {
            return Err("E_CLOCKWORKS_CORE_SOURCE_INVALID".into());
        }
        let ids = [clockworks_core::CORE_CONSOLE_ID];
        scene.restore_durable_interactions(if persistent.core_console_confirmed { &ids } else { &[] })
            .map_err(|_| "E_CLOCKWORKS_CORE_SOURCE_INVALID")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal_runtime::entry_test_support::acknowledge_ready;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn runtime() -> FormalRuntime {
        let mut runtime = super::super::tests::runtime_for_clockworks_shutdown(PRESSURE_SCENE_ID);
        runtime.save_root = std::env::temp_dir().join(format!("cw-controls-{}-{}",
            std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        runtime
    }
    fn activate(runtime: &FormalRuntime, index: usize, request: &str) -> FormalInteractionResponse {
        let id = CLOCKWORKS_PRESSURE_VALVE_IDS[index];
        let position = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene()
            .interactions.iter().find(|i| i.id == id).unwrap().position;
        runtime.state.lock().unwrap().world.player.position_m = vec3_from_array(position);
        let epoch = runtime.snapshot().unwrap().world_epoch;
        runtime.activate_scene_interaction(id, request, epoch).unwrap()
    }
    fn transition(runtime: &FormalRuntime, id: &str, x: f32, request: &str) {
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(x, 0.0, 8.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        acknowledge_ready(runtime, runtime.transition_scene(id, request, epoch).unwrap());
    }
    fn saved(runtime: &FormalRuntime) -> crate::save_v6::SaveV6 {
        let state = runtime.state.lock().unwrap();
        let scene = runtime.scene_runtime.lock().unwrap();
        crate::save_v6::SaveV6::capture(&state, scene.as_ref()).unwrap()
    }

    #[test]
    fn partial_valves_survive_transition_file_slot_continue_and_new_journey_resets() {
        let runtime = runtime();
        assert!(activate(&runtime, 1, "valve-two").applied);
        let first = saved(&runtime);
        assert_eq!(first.world_persistent_v1.clockworks.pressure_valve_ids,
            vec![CLOCKWORKS_PRESSURE_VALVE_IDS[1]]);
        assert!(!route_completed_events(&first.save.progression, "clockworks").contains(VALVES_EVENT_ID));
        transition(&runtime, "cw_return_to_entry_foundry", 1.0, "leave-partial");
        runtime.save().unwrap();
        let continued = runtime.continue_saved().unwrap();
        acknowledge_ready(&runtime, continued);
        assert_eq!(saved(&runtime).world_persistent_v1.clockworks, first.world_persistent_v1.clockworks);
        transition(&runtime, "cw_to_pressure_hall", 23.0, "revisit-partial");
        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let scene = scene.as_ref().unwrap();
            assert!(scene.object_activated(CLOCKWORKS_PRESSURE_VALVE_IDS[1]));
            assert!(!scene.object_activated(CLOCKWORKS_PRESSURE_VALVE_IDS[0]));
            assert!(!scene.event_complete(VALVES_EVENT_ID));
        }
        let before = saved(&runtime);
        assert!(!activate(&runtime, 1, "valve-two-again").applied);
        assert_eq!(saved(&runtime).world_persistent_v1, before.world_persistent_v1);
        assert_eq!(saved(&runtime).save.revision, before.save.revision);
        assert!(activate(&runtime, 0, "valve-one").applied);
        assert!(!route_completed_events(&saved(&runtime).save.progression, "clockworks").contains(VALVES_EVENT_ID));
        assert!(activate(&runtime, 2, "valve-three").applied);
        let complete = saved(&runtime);
        assert_eq!(complete.world_persistent_v1.clockworks.pressure_valve_ids,
            CLOCKWORKS_PRESSURE_VALVE_IDS);
        assert!(route_completed_events(&complete.save.progression, "clockworks").contains(VALVES_EVENT_ID));
        runtime.save_slot("valves", "Valves", true).unwrap();
        acknowledge_ready(&runtime, runtime.continue_slot("valves").unwrap());
        let restored = saved(&runtime);
        assert_eq!(restored.save.progression, complete.save.progression);
        assert_eq!(restored.save.capabilities.grants, complete.save.capabilities.grants);
        assert_eq!(restored.progression, complete.progression);
        assert_eq!(restored.world_persistent_v1.clockworks, complete.world_persistent_v1.clockworks);
        assert!(runtime.state.lock().unwrap().presentation_events.is_empty());
        acknowledge_ready(&runtime, runtime.reset_new().unwrap());
        assert!(runtime.state.lock().unwrap().world_persistent_v1.clockworks.is_default());
        let _ = std::fs::remove_dir_all(&runtime.save_root);
    }

    #[test]
    fn old_valve_events_and_scene_ledgers_require_physical_reconfirmation_without_progress_replay() {
        let runtime = runtime();
        for index in 0..3 { assert!(activate(&runtime, index, &format!("old-valve-{index}")).applied); }
        runtime.state.lock().unwrap().world_persistent_v1.clockworks.pressure_valve_ids.clear();
        let old = saved(&runtime);
        runtime.save().unwrap();
        acknowledge_ready(&runtime, runtime.continue_saved().unwrap());
        assert!(saved(&runtime).world_persistent_v1.clockworks.pressure_valve_ids.is_empty());
        {
            let scene = runtime.scene_runtime.lock().unwrap();
            let scene = scene.as_ref().unwrap();
            assert!(scene.event_complete(VALVES_EVENT_ID));
            assert!(CLOCKWORKS_PRESSURE_VALVE_IDS.iter().all(|id| scene.object_activated(id)));
        }
        for index in [2, 0, 1] {
            let id = CLOCKWORKS_PRESSURE_VALVE_IDS[index];
            let position = runtime.scene_runtime.lock().unwrap().as_ref().unwrap().current_scene()
                .interactions.iter().find(|i| i.id == id).unwrap().position;
            runtime.state.lock().unwrap().world.player.position_m = vec3_from_array(position);
            assert!(runtime.snapshot().unwrap().interactables.iter().find(|i| i.entity_id == id).unwrap().active);
            runtime.state.lock().unwrap().paused = true;
            assert!(!runtime.snapshot().unwrap().interactables.iter().find(|i| i.entity_id == id).unwrap().active);
            runtime.state.lock().unwrap().paused = false;
            assert!(activate(&runtime, index, &format!("reconfirm-valve-{index}")).applied);
            assert!(!runtime.snapshot().unwrap().interactables.iter().find(|i| i.entity_id == id).unwrap().active);
            assert_eq!(saved(&runtime).save.progression, old.save.progression);
            assert_eq!(saved(&runtime).save.capabilities.grants, old.save.capabilities.grants);
        }
        assert_eq!(saved(&runtime).world_persistent_v1.clockworks.pressure_valve_ids,
            CLOCKWORKS_PRESSURE_VALVE_IDS);
        assert!(!activate(&runtime, 0, "already-reconfirmed").applied);
        let _ = std::fs::remove_dir_all(&runtime.save_root);
    }

    #[test]
    fn invalid_valve_commands_leave_receipts_scene_and_revision_unchanged() {
        for invalid in ["stale", "far", "route", "scene-epoch"] {
            let runtime = runtime();
            let epoch = runtime.snapshot().unwrap().world_epoch;
            {
                let mut state = runtime.state.lock().unwrap();
                state.world.player.position_m = Vec3::new(8.0, 0.0, 8.0).unwrap();
                if invalid == "far" { state.world.player.position_m = Vec3::new(2.0, 0.0, 8.0).unwrap(); }
                if invalid == "route" { state.route.current_world_id = "grey_hive".into(); }
            }
            if invalid == "scene-epoch" { runtime.scene_runtime.lock().unwrap().as_mut().unwrap().world_epoch += 1; }
            let before_scene = format!("{:?}", runtime.scene_runtime.lock().unwrap());
            let revision = runtime.state.lock().unwrap().world.revision;
            let persistent = runtime.state.lock().unwrap().world_persistent_v1.clone();
            let route = runtime.state.lock().unwrap().route.clone();
            let response = runtime.activate_scene_interaction(CLOCKWORKS_PRESSURE_VALVE_IDS[0],
                "invalid-command", epoch + u64::from(invalid == "stale")).unwrap();
            assert!(!response.applied, "{invalid}");
            let state = runtime.state.lock().unwrap();
            assert_eq!(state.world.revision, revision, "{invalid}");
            assert_eq!(state.world_persistent_v1, persistent, "{invalid}");
            assert_eq!(state.route, route, "{invalid}");
            drop(state);
            assert_eq!(format!("{:?}", runtime.scene_runtime.lock().unwrap()), before_scene, "{invalid}");
        }
    }

    #[test]
    fn generic_valves_console_keeps_legacy_event_without_canonical_source_receipts() {
        let runtime = super::super::tests::runtime_for_clockworks_shutdown("cw_test_exit");
        runtime.state.lock().unwrap().world.player.position_m = Vec3::new(1.0, 0.0, 1.0).unwrap();
        let epoch = runtime.snapshot().unwrap().world_epoch;
        let response = runtime.activate_scene_interaction("cw_valves_test", "generic-valves", epoch).unwrap();
        assert!(response.applied, "{:?}", response.error_code);
        let state = runtime.state.lock().unwrap();
        assert!(completed_world_events(&state, "clockworks").contains(VALVES_EVENT_ID));
        assert!(state.world_persistent_v1.clockworks.pressure_valve_ids.is_empty());
        assert!(!reconfirming_completed_event(&state, VALVES_EVENT_ID));
    }

    #[test]
    fn malformed_control_save_records_fail_without_changing_file_or_live_state() {
        let runtime = runtime();
        assert!(activate(&runtime, 0, "one-valid-valve").applied);
        runtime.save().unwrap();
        let before = saved(&runtime);
        let file = crate::save_v6::save_path(&runtime.save_root);
        let bytes = std::fs::read(&file).unwrap();
        for ids in [vec!["unknown"], vec![CLOCKWORKS_PRESSURE_VALVE_IDS[0]; 2],
            vec![CLOCKWORKS_PRESSURE_VALVE_IDS[1], CLOCKWORKS_PRESSURE_VALVE_IDS[0]],
            vec![CLOCKWORKS_PRESSURE_VALVE_IDS[0]; 4], CLOCKWORKS_PRESSURE_VALVE_IDS.to_vec()] {
            let mut bad = before.clone();
            bad.world_persistent_v1.clockworks.pressure_valve_ids = ids.into_iter().map(str::to_owned).collect();
            assert!(bad.restore_state().is_err());
            assert!(crate::save_v6::write_save(&runtime.save_root, &bad).is_err());
            assert_eq!(std::fs::read(&file).unwrap(), bytes);
            assert_eq!(saved(&runtime), before);
            let invalid_bytes = serde_json::to_vec(&bad).unwrap();
            std::fs::write(&file, &invalid_bytes).unwrap();
            assert!(runtime.continue_saved().is_err());
            assert_eq!(std::fs::read(&file).unwrap(), invalid_bytes);
            assert_eq!(saved(&runtime), before);
            std::fs::write(&file, &bytes).unwrap();
        }
        for defeated in [false, true] {
            let mut bad = before.clone();
            bad.world_persistent_v1.clockworks.core_console_confirmed = true;
            bad.world_persistent_v1.clockworks.regulator_defeated = defeated;
            assert!(bad.restore_state().is_err());
            assert!(crate::save_v6::write_save(&runtime.save_root, &bad).is_err());
            assert_eq!(std::fs::read(&file).unwrap(), bytes);
            assert_eq!(saved(&runtime), before);
        }
        let _ = std::fs::remove_dir_all(&runtime.save_root);
    }
}
