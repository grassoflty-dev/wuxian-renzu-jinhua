//! Native Clockworks campaign transactions. Loading never settles an old save:
//! the canonical shutdown terminal is the only completion authority.
use super::*;
use crate::world_progression::RouteEvent;

pub(super) const SHUTDOWN_ID: &str = "cw_master_shutdown_staged";
const SHUTDOWN_EVENT: &str = "clockworks_shutdown";
const CW: &str = "clockworks";
const EVENTS: [&str; 3] = ["clockworks_valves", "clockworks_core", SHUTDOWN_EVENT];

pub(super) fn mist_harbor_extracted(state: &RuntimeState) -> bool {
    state.world_persistent_v1.mist_harbor.warden_defeated
        && state.route.progress.iter().any(|progress| {
            progress.world_id == "mist_harbor"
                && progress.completed
                && progress.first_completion
                && ["mist_beacon_west", "mist_beacon_east", "mist_signal"].iter()
                    .all(|event| progress.completed_events.iter().any(|actual| actual == event))
        })
}

pub(super) fn entry_eligible(state: &RuntimeState) -> bool {
    mist_harbor_extracted(state)
        && state.route.progress.iter().find(|progress| progress.world_id == CW)
            .is_some_and(|progress| !progress.completed
                || crate::world_progression::can_revisit_world(&state.route, CW))
}

/// Prepare a route change without installing a destination. A repeated request
/// is a no-op in the route ledger and must never become a fresh world visit.
pub(super) fn prepare_entry_route(state: &RuntimeState, request_id: &str) -> Result<RouteState, String> {
    if !valid_id(request_id) { return Err("E_WORLD_GATE_REQUEST_INVALID".into()); }
    if !mist_harbor_extracted(state) { return Err("E_WORLD_GATE_MH_EXTRACTION_REQUIRED".into()); }
    if !entry_eligible(state) { return Err("E_WORLD_GATE_UNAVAILABLE".into()); }
    let revisiting = state.route.progress.iter().find(|progress| progress.world_id == CW)
        .ok_or("E_CLOCKWORKS_PROGRESS_MISSING")?.completed;
    let command = if revisiting {
        RouteCommand::Revisit { world_id: CW.into(), request_id: format!("world-gate-revisit-{request_id}") }
    } else {
        RouteCommand::Enter { world_id: CW.into(), request_id: format!("world-gate-enter-{request_id}") }
    };
    let mut candidate = state.route.clone();
    let result = apply_route_command(&mut candidate, command, state.world.revision);
    let expected = match result {
        RouteResult::Applied { ref events, .. } if revisiting => matches!(events.as_slice(),
            [RouteEvent::Revisited { world_id, .. }] if world_id == CW),
        RouteResult::Applied { ref events, .. } => matches!(events.as_slice(),
            [RouteEvent::Entered { world_id, .. }] if world_id == CW),
        RouteResult::Rejected { .. } => false,
    };
    if !expected { return Err("E_CLOCKWORKS_ENTRY_ROUTE_REJECTED".into()); }
    Ok(candidate)
}

/// The persisted acquisition and its resolved source must agree. Selection may
/// differ in a legacy save and is never changed merely to reconfirm settlement.
fn has_one_air_step_source(state: &RuntimeState) -> bool {
    if state.capabilities.grants.iter().filter(|grant| grant.capability_id == CAP_AIR_STEP).count() != 1 {
        return false;
    }
    let Ok((expected, _)) = state.progression_v6.resolve_rules_with_capabilities(
        &state.capabilities, &state.world.rear_view,
    ) else { return false; };
    expected == state.effect_sources_v6
        && expected.iter().filter(|source| {
            source.source_id == CAP_AIR_STEP
                && source.instance_id == format!("capability:{CAP_AIR_STEP}")
                && source.source_kind == crate::effects::EffectSourceKind::InnateCapability
        }).count() == 1
}

pub(super) fn completed(state: &RuntimeState) -> bool {
    state.world_persistent_v1.clockworks.regulator_defeated
        && has_one_air_step_source(state)
        && state.route.progress.iter().any(|progress| {
            progress.world_id == CW && progress.completed && progress.first_completion
                && EVENTS.iter().all(|event| progress.completed_events.iter().any(|actual| actual == event))
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShutdownKind { FirstSettlement, LegacyReconfirmation }

fn shutdown_kind(state: &RuntimeState, scene: &SceneRuntime) -> Result<ShutdownKind, String> {
    let definition = scene.current_scene();
    if !scene.is_complete_clockworks_production()
        || definition.world_id != CW || definition.scene_id != "cw_shutdown_exit"
        || state.world.world_id != CW || state.world.scene_id != definition.scene_id
        || state.route.current_world_id != CW || scene.world_epoch != state.world.revision.world_epoch
        || !definition.interactions.iter().any(|item| {
            item.id == SHUTDOWN_ID && item.kind == "terminal"
                && item.event.as_deref() == Some(SHUTDOWN_EVENT)
        })
    {
        return Err("E_CLOCKWORKS_SHUTDOWN_SOURCE_INVALID".into());
    }
    let progress = state.route.progress.iter().find(|progress| progress.world_id == CW)
        .ok_or("E_CLOCKWORKS_PROGRESS_MISSING")?;
    if progress.completed || progress.first_completion {
        return Err("E_CLOCKWORKS_SHUTDOWN_ALREADY_SETTLED".into());
    }
    if !["clockworks_valves", "clockworks_core"].iter()
        .all(|event| progress.completed_events.iter().any(|actual| actual == event))
    {
        return Err("E_CLOCKWORKS_SHUTDOWN_PREREQUISITES_MISSING".into());
    }
    if !state.world_persistent_v1.clockworks.regulator_defeated {
        return Err("E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED".into());
    }
    let already_shutdown = progress.completed_events.iter().any(|event| event == SHUTDOWN_EVENT);
    if already_shutdown {
        if !has_one_air_step_source(state) {
            return Err("E_CLOCKWORKS_LEGACY_SOURCE_INVALID".into());
        }
        // Old valid objects predate per-valve durable records. Reconfirm the
        // actual terminal; never manufacture the missing historical records.
        return Ok(ShutdownKind::LegacyReconfirmation);
    }
    if state.capabilities.grants.iter().any(|grant| grant.capability_id == CAP_AIR_STEP) {
        return Err("E_CLOCKWORKS_AIR_STEP_DUPLICATE_GRANT".into());
    }
    let persistent = &state.world_persistent_v1.clockworks;
    if !persistent.core_console_confirmed
        || persistent.pressure_valve_ids.len() != 3
        || !crate::world_persistent_v1::CLOCKWORKS_PRESSURE_VALVE_IDS.iter()
            .all(|id| persistent.pressure_valve_ids.iter().any(|recorded| recorded.as_str() == *id))
    {
        return Err("E_CLOCKWORKS_OBJECTIVE_SOURCE_REQUIRED".into());
    }
    Ok(ShutdownKind::FirstSettlement)
}

pub(super) fn shutdown_available(state: &RuntimeState, scene: &SceneRuntime) -> bool {
    !state.paused && state.world.player_hp > 0 && shutdown_kind(state, scene).is_ok()
}

/// A previously activated legacy terminal can be physically confirmed again.
/// Its prior activation/event ledgers remain intact, and the new request is
/// consumed only on a candidate that the outer transaction commits in full.
pub(super) fn reconfirm_shutdown(
    state: &RuntimeState, scene: &mut SceneRuntime, interaction_id: &str,
    request_id: &str, world_epoch: u64,
) -> Result<Option<Vec<SceneEvent>>, String> {
    if interaction_id != SHUTDOWN_ID || !scene.is_complete_clockworks_production() {
        return Ok(None);
    }
    let kind = shutdown_kind(state, scene)?;
    if world_epoch != state.world.revision.world_epoch {
        return Err("E_SCENE_RUNTIME_StaleEpoch".into());
    }
    if kind != ShutdownKind::LegacyReconfirmation || !scene.object_activated(SHUTDOWN_ID) {
        return Ok(None);
    }
    let item = scene.current_scene().interactions.iter().find(|item| item.id == SHUTDOWN_ID)
        .ok_or("E_CLOCKWORKS_SHUTDOWN_SOURCE_INVALID")?;
    let p = state.world.player.position_m;
    let distance2 = (p.x_m - item.position[0]).powi(2)
        + (p.y_m - item.position[1]).powi(2) + (p.z_m - item.position[2]).powi(2);
    if !distance2.is_finite() || distance2 > item.range_m.unwrap_or(2.5).powi(2) {
        return Err("E_SCENE_RUNTIME_OutOfRange".into());
    }
    if !scene.event_complete(SHUTDOWN_EVENT) {
        return Err("E_CLOCKWORKS_LEGACY_SOURCE_INVALID".into());
    }
    scene.claim_command_request(request_id, world_epoch).map_err(|error| error.to_string())?;
    Ok(Some(vec![SceneEvent::Interaction {
        id: SHUTDOWN_ID.into(), event_id: Some(SHUTDOWN_EVENT.into()),
    }]))
}

pub(super) fn apply_shutdown(
    state: &mut RuntimeState, scene: &SceneRuntime, events: &[SceneEvent], request_id: &str,
) -> Result<bool, String> {
    if !scene.is_complete_clockworks_production() { return Ok(false); }
    let related = events.iter().any(|event| match event {
        SceneEvent::Interaction { id, event_id } => id == SHUTDOWN_ID || event_id.as_deref() == Some(SHUTDOWN_EVENT),
        SceneEvent::Trigger { event_id, .. } => event_id == SHUTDOWN_EVENT,
        _ => false,
    });
    if !related { return Ok(false); }
    let kind = shutdown_kind(state, scene)?;
    if !matches!(events, [SceneEvent::Interaction { id, event_id: Some(event) }]
        if id == SHUTDOWN_ID && event == SHUTDOWN_EVENT)
        || !scene.object_activated(SHUTDOWN_ID) || !scene.event_complete(SHUTDOWN_EVENT)
    {
        return Err("E_CLOCKWORKS_SHUTDOWN_SOURCE_INVALID".into());
    }
    let mut candidate = state.clone();
    if kind == ShutdownKind::FirstSettlement {
        let result = apply_route_command(&mut candidate.route, RouteCommand::Progress {
            event_id: SHUTDOWN_EVENT.into(), request_id: format!("scene-event-{request_id}"),
        }, state.world.revision);
        if !matches!(result, RouteResult::Applied { ref events, .. }
            if matches!(events.as_slice(), [RouteEvent::Progressed { world_id, event_id }]
                if world_id == CW && event_id == SHUTDOWN_EVENT))
        {
            return Err("E_CLOCKWORKS_SHUTDOWN_PROGRESS_REJECTED".into());
        }
    }
    let result = apply_route_command(&mut candidate.route, RouteCommand::Complete {
        world_id: CW.into(), request_id: format!("scene-complete-{request_id}"),
    }, state.world.revision);
    if !matches!(result, RouteResult::Applied { ref events, .. }
        if matches!(events.as_slice(), [RouteEvent::Completed { world_id, first_completion: true }]
            if world_id == CW))
    {
        return Err("E_CLOCKWORKS_SHUTDOWN_COMPLETE_REJECTED".into());
    }
    if kind == ShutdownKind::FirstSettlement {
        let mut capabilities = candidate.capabilities.clone();
        apply_command_at_revision(&mut capabilities, CapabilityCommand::Grant {
            capability_id: CAP_AIR_STEP.into(),
        }, candidate.world.revision).map_err(|error| format!("E_CLOCKWORKS_AIR_STEP_GRANT: {error:?}"))?;
        let mut selected = capabilities.selected.clone();
        selected.push(CAP_AIR_STEP.into());
        apply_command_at_revision(&mut capabilities, CapabilityCommand::Select {
            capability_ids: selected,
        }, candidate.world.revision).map_err(|error| format!("E_CLOCKWORKS_AIR_STEP_SELECT: {error:?}"))?;
        candidate.install_capability_state(candidate.world.clone(), capabilities)?;
    }
    if !completed(&candidate) { return Err("E_CLOCKWORKS_SETTLEMENT_INCONSISTENT".into()); }
    *state = candidate;
    Ok(true)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod admission_tests;
