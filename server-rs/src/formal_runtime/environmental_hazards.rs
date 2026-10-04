//! Adapter for authored generic environmental hazards; mechanics do not branch on scene IDs.
use super::*;
use crate::environment_hazards::{self as model, EnvironmentHazardState, EnvironmentPhase};
use crate::scene_runtime::{HazardDefinition, InteractionDefinition, SceneDefinition};
use std::collections::BTreeMap;

fn key(scene: &SceneDefinition, id: &str) -> String {
    format!("{}/{}/{}", scene.world_id, scene.scene_id, id)
}

/// Revisit time outside a scene dissipates exposure but never inflicts off-screen damage.
/// Cycle/coolant deadlines survive; the authored safe spawn remains clear of every hazard.
pub(super) fn prepare_scene(
    state: &mut RuntimeState,
    scene: &SceneDefinition,
) -> Result<(), String> {
    state.world_persistent_v1.environment = prepared_scene(state, scene)?;
    Ok(())
}

pub(super) fn prepared_scene(
    state: &RuntimeState,
    scene: &SceneDefinition,
) -> Result<model::EnvironmentPersistentState, String> {
    let now = state.world.server_time_ms;
    let mut next = state.world_persistent_v1.environment.clone();
    for hazard in &scene.hazards {
        let Some(config) = &hazard.environment else {
            continue;
        };
        let saved = next
            .hazards
            .entry(key(scene, &hazard.id))
            .or_insert_with(|| EnvironmentHazardState::new(now));
        saved.advance(config, now, false)?;
    }
    Ok(next)
}

pub(super) fn advance(state: &mut RuntimeState, scene: &SceneRuntime) -> Result<(), String> {
    if state.paused
        || scene.world_epoch != state.world.revision.world_epoch
        || scene.current_scene().scene_id != state.world.scene_id
        || scene.current_scene().world_id != state.world.world_id
    {
        return Ok(());
    }
    let definition = scene.current_scene();
    let now = state.world.server_time_ms;
    let mut next = state.world_persistent_v1.environment.clone();
    let mut hits = Vec::new();
    for hazard in &definition.hazards {
        let Some(config) = &hazard.environment else {
            continue;
        };
        let saved = next
            .hazards
            .get_mut(&key(definition, &hazard.id))
            .ok_or("E_ENV_HAZARD_STATE_MISSING")?;
        let inside = hazard.height_allows(state.world.player.position_m.y_m) && point_in_or_on_polygon(
            state.world.player.position_m.x_m,
            state.world.player.position_m.z_m,
            &hazard.polygon,
        );
        if saved.advance(config, now, inside)? {
            hits.push((config.damage, config.tag));
        }
    }
    state.world_persistent_v1.environment = next;
    for (damage, tag) in hits {
        encounter_hazards::apply_hazard_damage(state, damage, tag, "EnvironmentHazardDamage");
    }
    Ok(())
}

pub(super) fn control_available(
    state: &RuntimeState,
    scene: &SceneDefinition,
    control: &InteractionDefinition,
) -> bool {
    control.environment_control.is_some()
        && !state.paused
        && state.world.server_time_ms
            >= state
                .world_persistent_v1
                .environment
                .control_cooldown_until_ms
                .get(&key(scene, &control.id))
                .copied()
                .unwrap_or(0)
}

/// Called only on the existing transactional candidate, after epoch/replay/range checks.
pub(super) fn apply_control(
    state: &mut RuntimeState,
    scene: &SceneDefinition,
    id: &str,
) -> Result<(), String> {
    let Some(item) = scene.interactions.iter().find(|item| item.id == id) else {
        return Ok(());
    };
    let Some(control) = &item.environment_control else {
        return Ok(());
    };
    if !control_available(state, scene, item) {
        return Err("E_ENV_CONTROL_COOLDOWN".into());
    }
    let now = state.world.server_time_ms;
    let cooldown = item.cooldown_ms.ok_or("E_ENV_CONTROL_CONFIG")?;
    let until = now.checked_add(cooldown).ok_or("E_ENV_TIME_OVERFLOW")?;
    let mut next = state.world_persistent_v1.environment.clone();
    for id in &control.target_hazard_ids {
        let hazard = scene
            .hazards
            .iter()
            .find(|h| h.id == *id)
            .ok_or("E_ENV_CONTROL_TARGET_UNKNOWN")?;
        let config = hazard
            .environment
            .as_ref()
            .ok_or("E_ENV_CONTROL_TARGET_UNKNOWN")?;
        let saved = next
            .hazards
            .get_mut(&key(scene, id))
            .ok_or("E_ENV_HAZARD_STATE_MISSING")?;
        saved.suppress(config, control, now)?;
    }
    next.control_cooldown_until_ms
        .insert(key(scene, &item.id), until);
    state.world_persistent_v1.environment = next;
    emit_presentation(
        state,
        "EnvironmentHazardSuppressed",
        vec3_from_array(item.position),
        1.0,
    );
    Ok(())
}

pub(super) fn project(
    state: &RuntimeState,
    scene: &SceneDefinition,
    hazard: &HazardDefinition,
) -> Option<crate::world_v3::HazardView> {
    let config = hazard.environment.as_ref()?;
    let saved = state
        .world_persistent_v1
        .environment
        .hazards
        .get(&key(scene, &hazard.id))?;
    let frame = saved.frame(config, state.world.server_time_ms).ok()?;
    Some(crate::world_v3::HazardView {
        height_range_m: hazard.height_range_m,
        entity_id: hazard.id.clone(),
        kind: hazard.kind.clone(),
        transform: Transform {
            position_m: {let mut p=polygon_center(&hazard.polygon);p.y_m=hazard.presentation_height();p},
            yaw_rad: 0.0,
        },
        active: frame.phase == EnvironmentPhase::Active,
        polygon_m: Some(hazard.polygon.clone()),
        warning_remaining_ms: (frame.phase == EnvironmentPhase::Warning)
            .then_some(frame.remaining_ms),
        phase_active: Some(true),
        environment: Some(crate::world_v3::EnvironmentHazardProjection {
            phase: frame.phase,
            remaining_ms: frame.remaining_ms,
            exposure_bps: frame.exposure_bps,
            tag: config.tag,
        }),
    })
}

/// Save data is matched to the statically embedded, SHA-pinned production definitions.
/// An unknown world/scene/hazard/control is rejected even when its values look plausible.
pub(crate) fn validate_saved(
    state: &model::EnvironmentPersistentState,
    now: u64,
) -> Result<(), String> {
    if state.is_default() {
        return Ok(());
    }
    type Catalog = (
        BTreeMap<String, model::EnvironmentHazardConfig>,
        BTreeMap<String, u64>,
    );
    static CATALOG: std::sync::OnceLock<Result<Catalog, String>> = std::sync::OnceLock::new();
    let (hazards, controls) = CATALOG
        .get_or_init(|| {
            let mut hazards = BTreeMap::new();
            let mut controls = BTreeMap::new();
            for raw in crate::production_scene_bootstrap::embedded_scene_json()? {
                let scene: SceneDefinition =
                    serde_json::from_str(raw).map_err(|_| "E_ENV_CANONICAL_SCENE")?;
                for hazard in &scene.hazards {
                    if let Some(config) = &hazard.environment {
                        config.validate()?;
                        hazards.insert(key(&scene, &hazard.id), config.clone());
                    }
                }
                for item in &scene.interactions {
                    if let Some(config) = &item.environment_control {
                        config.validate()?;
                        controls.insert(
                            key(&scene, &item.id),
                            item.cooldown_ms.ok_or("E_ENV_CONTROL_CONFIG")?,
                        );
                    }
                }
            }
            Ok((hazards, controls))
        })
        .as_ref()
        .map_err(Clone::clone)?;
    model::validate_persistence(state, hazards, controls, now).map_err(str::to_string)
}

#[cfg(test)]
mod tests;
