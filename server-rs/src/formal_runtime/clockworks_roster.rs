//! One bounded authored-roster upgrade for Clockworks ordinary enemies.
//! Version zero accepts only the exact previous Guard roster (or historical
//! absent actor records). Version one never recreates omitted saved enemies in any canonical CW scene.
use super::*;
use crate::world_persistent_v1::ClockworksPersistentState;

pub(crate) const ACTOR_ROSTER_VERSION: u32 = 1;
pub(crate) const DRONE: &str = "enemy.clockworks.pressure_drone";
pub(crate) const HOUND: &str = "enemy.clockworks.furnace_hound";

fn additions(scene_id: &str) -> Option<(&'static str, usize)> {
    match scene_id {
        "cw_pressure_hall" | "cw_conveyor_bridge" => Some((DRONE, 2)),
        "cw_gear_shaft" => Some((DRONE, 3)),
        "cw_boiler_chamber" | "cw_furnace_heart" => Some((HOUND, 3)),
        _ => None,
    }
}

fn old_guard_positions(scene_id: &str) -> &'static [[f32; 3]] {
    match scene_id {
        "cw_pressure_hall" | "cw_gear_shaft" | "cw_furnace_heart" => &[[6.5, 0.0, 8.0], [9.5, 0.0, 8.0]],
        "cw_conveyor_bridge" => &[[5.5, 0.0, 8.0], [8.0, 0.0, 8.0], [10.5, 0.0, 8.0]],
        _ => &[],
    }
}

fn same_identity(left: &crate::world_v3::ActorRuntime, right: &crate::world_v3::ActorRuntime) -> bool {
    left.entity_id == right.entity_id && left.entity_type == right.entity_type && left.home_m == right.home_m
}

fn validate_authored(scene: &crate::scene_runtime::SceneDefinition) -> Result<Vec<crate::world_v3::ActorRuntime>, String> {
    let Some((new_type, count)) = additions(&scene.scene_id) else { return Err("E_CW_ACTOR_ROSTER_SCENE".into()); };
    if scene.world_id != "clockworks" { return Err("E_CW_ACTOR_ROSTER_SCENE".into()); }
    let actors = spawn_generic_actors(scene)?;
    let old = old_guard_positions(&scene.scene_id);
    if actors.len() != old.len() + count { return Err("E_CW_ACTOR_ROSTER_CONTENT".into()); }
    for (index, actor) in actors.iter().take(old.len()).enumerate() {
        if actor.entity_id != format!("{}_forged_guard_{:02}", scene.scene_id, index + 1)
            || actor.entity_type != "enemy.clockworks.forged_guard"
            || actor.home_m != vec3_from_array(old[index])
        { return Err("E_CW_ACTOR_ROSTER_CONTENT".into()); }
    }
    let role = if new_type == DRONE { "pressure_drone" } else { "furnace_hound" };
    for (index, actor) in actors.iter().skip(old.len()).enumerate() {
        if actor.entity_id != format!("{}_{role}_{:02}", scene.scene_id, index + 1)
            || actor.entity_type != new_type || !actor.validate()
        { return Err("E_CW_ACTOR_ROSTER_CONTENT".into()); }
    }
    Ok(actors)
}

pub(super) fn mark_installed(
    persistent: &mut ClockworksPersistentState,
    scene: &crate::scene_runtime::SceneDefinition,
    strict_native: bool,
) -> Result<(), String> {
    if persistent.actor_roster_version > ACTOR_ROSTER_VERSION {
        return Err("E_SAVE_CW_ACTOR_ROSTER_VERSION".into());
    }
    if strict_native && scene.world_id == "clockworks" {
        current_definition(&scene.scene_id)?;
        if additions(&scene.scene_id).is_some() { validate_authored(scene)?; }
        persistent.actor_roster_version = ACTOR_ROSTER_VERSION;
    }
    Ok(())
}

/// Called on restored candidates only. No save file is written by this upgrade.
pub(super) fn restore(
    world: &mut WorldStateV3,
    scene: &crate::scene_runtime::SceneDefinition,
    persistent: &mut ClockworksPersistentState,
    strict_native: bool,
) -> Result<bool, String> {
    if !strict_native || scene.world_id != "clockworks" {
        return Ok(false);
    }
    if persistent.actor_roster_version > ACTOR_ROSTER_VERSION {
        return Err("E_SAVE_CW_ACTOR_ROSTER_VERSION".into());
    }
    current_definition(&scene.scene_id)?;
    let expanded = additions(&scene.scene_id).is_some();
    let authored = if expanded { validate_authored(scene)? } else { spawn_generic_actors(scene)? };
    let exact = world.generic_actors.len() == authored.len()
        && world.generic_actors.iter().zip(&authored).all(|(saved, expected)| same_identity(saved, expected) && saved.validate());
    if exact {
        persistent.actor_roster_version = ACTOR_ROSTER_VERSION;
        return Ok(true);
    }
    if persistent.actor_roster_version == ACTOR_ROSTER_VERSION {
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    if !expanded {
        // Preserve the historical empty-record fallback only for version zero.
        if world.generic_actors.is_empty() {
            world.generic_actors = authored;
            persistent.actor_roster_version = ACTOR_ROSTER_VERSION;
            return Ok(true);
        }
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    let old_count = old_guard_positions(&scene.scene_id).len();
    let exact_old = world.generic_actors.len() == old_count
        && world.generic_actors.iter().zip(authored.iter().take(old_count))
            .all(|(saved, expected)| same_identity(saved, expected) && saved.validate());
    if !world.generic_actors.is_empty() && !exact_old {
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    let mut upgraded = authored;
    // Preserve every historical actor byte/state; only the new authored rows
    // are initialized. Empty records retain the existing pre-actor fallback.
    for (index, saved) in world.generic_actors.iter().enumerate() {
        upgraded[index] = saved.clone();
    }
    world.generic_actors = upgraded;
    persistent.actor_roster_version = ACTOR_ROSTER_VERSION;
    Ok(true)
}

fn current_definition(scene_id: &str) -> Result<crate::scene_runtime::SceneDefinition, String> {
    let raw = match scene_id {
        "cw_entry_foundry" => include_str!("../../../content/scenes/compiled/cw_entry_foundry.json"),
        "cw_forged_guard_arena" => include_str!("../../../content/scenes/compiled/cw_forged_guard_arena.json"),
        "cw_regulator_core" => include_str!("../../../content/scenes/compiled/cw_regulator_core.json"),
        "cw_shutdown_exit" => include_str!("../../../content/scenes/compiled/cw_shutdown_exit.json"),
        "cw_pressure_hall" => include_str!("../../../content/scenes/compiled/cw_pressure_hall.json"),
        "cw_conveyor_bridge" => include_str!("../../../content/scenes/compiled/cw_conveyor_bridge.json"),
        "cw_gear_shaft" => include_str!("../../../content/scenes/compiled/cw_gear_shaft.json"),
        "cw_boiler_chamber" => include_str!("../../../content/scenes/compiled/cw_boiler_chamber.json"),
        "cw_furnace_heart" => include_str!("../../../content/scenes/compiled/cw_furnace_heart.json"),
        _ => return Err("E_CW_ACTOR_ROSTER_SCENE".into()),
    };
    serde_json::from_str(raw).map_err(|_| "E_CW_ACTOR_ROSTER_CONTENT".into())
}

pub(crate) fn validate_saved(
    save: &crate::save_v5::SaveV5,
    persistent: &ClockworksPersistentState,
) -> Result<(), String> {
    if persistent.actor_roster_version > ACTOR_ROSTER_VERSION {
        return Err("E_SAVE_CW_ACTOR_ROSTER_VERSION".into());
    }
    if save.world_id != "clockworks" { return Ok(()); }
    // Generic fixtures keep their old contract; strict native Continue performs
    // its own canonical-scene preflight before any state or file mutation.
    let Ok(scene) = current_definition(&save.scene_id) else { return Ok(()); };
    let expanded = additions(&save.scene_id).is_some();
    let authored = if expanded { validate_authored(&scene)? } else { spawn_generic_actors(&scene)? };
    let exact = save.generic_actors.len() == authored.len()
        && save.generic_actors.iter().zip(&authored)
            .all(|(saved, source)| same_identity(saved, source) && saved.validate());
    if exact { return Ok(()); }
    if persistent.actor_roster_version == ACTOR_ROSTER_VERSION {
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    if !expanded {
        return if save.generic_actors.is_empty() { Ok(()) }
            else { Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into()) };
    }
    let old_count = old_guard_positions(&save.scene_id).len();
    let exact_old = save.generic_actors.len() == old_count
        && save.generic_actors.iter().zip(authored.iter().take(old_count))
            .all(|(saved, source)| same_identity(saved, source) && saved.validate());
    if save.generic_actors.is_empty() || exact_old { Ok(()) }
    else { Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into()) }
}

#[cfg(test)]
pub(crate) mod tests;
