//! One authored facility log restores the permanent Scanner capability.
//! No world event, alternate grant endpoint, selection, or duplicate save flag is added.
use super::*;
pub(super) const OBJECT_ID: &str = "gh_log_shaft_01";
const WORLD_ID: &str = "grey_hive";
const SCENE_ID: &str = "gh_central_shaft";

pub(super) fn is_source(scene: &SceneRuntime, id: &str) -> bool {
    let definition = scene.current_scene();
    id == OBJECT_ID && definition.world_id == WORLD_ID && definition.scene_id == SCENE_ID
        && definition.interactions.iter().any(|item| item.id == OBJECT_ID
            && item.kind == "facility_log" && item.position == [3.0, 0.0, 8.0]
            && item.range_m.unwrap_or(2.5) == 2.5 && item.event.is_none())
        && !definition.interaction_aggregates.iter().any(|aggregate|
            aggregate.member_ids.iter().any(|id| id == OBJECT_ID))
}
fn granted(state: &RuntimeState) -> bool {
    state.capabilities.grants.iter().any(|grant| grant.capability_id == CAP_ENEMY_VITALS)
}
fn powered(state: &RuntimeState) -> bool {
    completed_world_events(state, WORLD_ID).contains(POWER_EVENT_ID)
}
pub(super) fn available(state: &RuntimeState, scene: &SceneRuntime, id: &str) -> bool {
    is_source(scene, id) && state.world.world_id == WORLD_ID && state.world.scene_id == SCENE_ID
        && state.route.current_world_id == WORLD_ID && powered(state)
}

/// Validate even repeated requests; a duplicate never bypasses range or lifecycle guards.
pub(super) fn validate(state: &RuntimeState, scene: &SceneRuntime, request: &str, epoch: u64) -> Result<(), String> {
    if !is_source(scene, OBJECT_ID) || state.world.world_id != WORLD_ID || state.world.scene_id != SCENE_ID
        || state.route.current_world_id != WORLD_ID { return Err("E_SCANNER_SOURCE_MISMATCH".into()); }
    if state.pending_entry.is_some() { return Err("E_SCENE_ENTRY_NOT_READY".into()); }
    if epoch != state.world.revision.world_epoch || epoch != scene.world_epoch {
        return Err("E_SCENE_RUNTIME_StaleEpoch".into());
    }
    if !valid_id(request) { return Err("E_SCENE_RUNTIME_RequestInvalid".into()); }
    let p = state.world.player.position_m;
    let distance = ((p.x_m - 3.0).powi(2) + p.y_m.powi(2) + (p.z_m - 8.0).powi(2)).sqrt();
    if !distance.is_finite() || distance > 2.5 || !scene.current_scene().height_allows(OBJECT_ID, p.y_m) {
        return Err("E_SCENE_RUNTIME_OutOfRange".into());
    }
    if !powered(state) { return Err("E_SCANNER_POWER_REQUIRED".into()); }
    Ok(())
}
pub(super) fn online(state: &RuntimeState, scene: &SceneRuntime) -> bool {
    scene.object_activated(OBJECT_ID) && granted(state)
}

/// Operates exclusively on the caller's unpublished transaction candidate.
pub(super) fn grant_candidate(state: &mut RuntimeState) -> Result<bool, String> {
    if granted(state) { return Ok(false); }
    let mut capabilities = state.capabilities.clone();
    let effects = apply_command_at_revision(&mut capabilities, CapabilityCommand::Grant {
        capability_id: CAP_ENEMY_VITALS.into(),
    }, state.world.revision).map_err(|error| format!("E_CAPABILITY_REJECTED: {error:?}"))?;
    let mut world = state.world.clone();
    if !effects.is_empty() {
        apply_effects_atomically(&mut world, &effects).map_err(|error| format!("E_WORLD_EFFECT: {error:?}"))?;
    }
    state.install_capability_state(world, capabilities)?;
    Ok(true)
}

/// Only explicit Continue calls this, after all source/geometry/save validation.
/// Saves retain one current scene, so absent historical evidence never implies a grant.
pub(super) fn restore_candidate(state: &mut RuntimeState, scene: Option<&SceneRuntime>, saved: &crate::save_v5::SaveV5) -> Result<bool, String> {
    if granted(state) || saved.world_id != WORLD_ID || saved.scene_id != SCENE_ID || !powered(state)
        || !saved.scene_states.iter().any(|history| history.scene_id == SCENE_ID
            && history.activated_ids.iter().any(|id| id == OBJECT_ID))
        || !scene.is_some_and(|scene| is_source(scene, OBJECT_ID) && scene.object_activated(OBJECT_ID)) {
        return Ok(false);
    }
    state.world.bump_authority_revision().map_err(|error| format!("E_WORLD_REVISION: {error:?}"))?;
    grant_candidate(state)
}
