//! Narrow Gate B roster upgrade. Other GH scenes and Sentinel state are untouched.
use super::*;
use crate::world_persistent_v1::GreyHivePersistentState;
use crate::world_v3::ActorRuntime;
const VERSION: u32 = 1;
const SCENE: &str = "gh_gate_b";
pub(crate) const BRUTE: &str = "enemy.grey_hive.brute";

fn identity(a: &ActorRuntime, b: &ActorRuntime) -> bool {
    a.entity_id == b.entity_id && a.entity_type == b.entity_type && a.home_m == b.home_m
}
fn authored(scene: &crate::scene_runtime::SceneDefinition) -> Result<Vec<ActorRuntime>, String> {
    if scene.world_id != "grey_hive" || scene.scene_id != SCENE { return Err("E_GATE_B_ROSTER_SCENE".into()); }
    let actors = spawn_generic_actors(scene)?;
    if actors.len() != 3 { return Err("E_GATE_B_ROSTER_CONTENT".into()); }
    for (index, home) in [[7.0, 0.0, 4.5], [17.0, 0.0, 11.5]].into_iter().enumerate() {
        let actor = &actors[index];
        if actor.entity_id != format!("gh_gate_b_security_{:02}", index + 1)
            || actor.entity_type != "grey_hive.infected_security" || actor.home_m != vec3_from_array(home)
        { return Err("E_GATE_B_ROSTER_CONTENT".into()); }
    }
    if actors[2].entity_id != "gh_gate_b_brute_01" || actors[2].entity_type != BRUTE || !actors[2].validate() {
        return Err("E_GATE_B_ROSTER_CONTENT".into());
    }
    Ok(actors)
}
fn canonical() -> Result<crate::scene_runtime::SceneDefinition, String> {
    serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_gate_b.json"))
        .map_err(|_| "E_GATE_B_ROSTER_CONTENT".into())
}
fn check_version(persistent: &GreyHivePersistentState) -> Result<(), String> {
    if persistent.gate_b_actor_roster_version > VERSION { Err("E_SAVE_GATE_B_ROSTER_VERSION".into()) } else { Ok(()) }
}
fn exact(saved: &[ActorRuntime], expected: &[ActorRuntime]) -> bool {
    saved.len() == expected.len() && saved.iter().zip(expected).all(|(a, b)| identity(a, b) && a.validate())
}

pub(super) fn mark_installed(persistent: &mut GreyHivePersistentState, scene: &crate::scene_runtime::SceneDefinition, strict: bool) -> Result<(), String> {
    check_version(persistent)?;
    if strict && scene.world_id == "grey_hive" && scene.scene_id == SCENE {
        authored(scene)?; persistent.gate_b_actor_roster_version = VERSION;
    }
    Ok(())
}

pub(super) fn restore(world: &mut WorldStateV3, scene: &crate::scene_runtime::SceneDefinition, persistent: &mut GreyHivePersistentState, strict: bool) -> Result<bool, String> {
    if !strict || scene.world_id != "grey_hive" || scene.scene_id != SCENE { return Ok(false); }
    check_version(persistent)?;
    let actors = authored(scene)?;
    if exact(&world.generic_actors, &actors) {
        persistent.gate_b_actor_roster_version = VERSION; return Ok(true);
    }
    if persistent.gate_b_actor_roster_version == VERSION
        || (!world.generic_actors.is_empty() && !exact(&world.generic_actors, &actors[..2])) {
        return Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into());
    }
    let mut upgraded = actors;
    for (i, saved) in world.generic_actors.iter().enumerate() { upgraded[i] = saved.clone(); }
    world.generic_actors = upgraded;
    persistent.gate_b_actor_roster_version = VERSION;
    Ok(true)
}

pub(crate) fn validate_saved(save: &crate::save_v5::SaveV5, persistent: &GreyHivePersistentState) -> Result<(), String> {
    check_version(persistent)?;
    if save.world_id != "grey_hive" || save.scene_id != SCENE { return Ok(()); }
    let actors = authored(&canonical()?)?;
    if exact(&save.generic_actors, &actors) { return Ok(()); }
    if persistent.gate_b_actor_roster_version == 0
        && (save.generic_actors.is_empty() || exact(&save.generic_actors, &actors[..2])) { return Ok(()); }
    Err("E_SAVE_ACTOR_ROSTER_MISMATCH".into())
}

#[cfg(test)]
mod tests;
