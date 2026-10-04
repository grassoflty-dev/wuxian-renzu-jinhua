//! The original Warden's local combat proof and defeat gates. Never a new required event.
use super::*;
use crate::continuous_combat::CombatEvent;
use crate::world_v3::{ActorRuntime, WardenAttack, WardenCueKind, WardenStage};
pub(super) const SCENE_ID: &str = "mh_warden_arena";
pub(super) const ACTOR_ID: &str = "mh_resonance_warden_staged_marker";
pub(super) const ENTITY_TYPE: &str = "enemy.mist_harbor.resonance_warden";
const WORLD_ID: &str = "mist_harbor";

fn canonical_scene(scene: &crate::scene_runtime::SceneDefinition) -> bool {
    let actors: Vec<_> = scene.spawns.iter().filter(|s| s.kind == "enemy").collect();
    scene.world_id == WORLD_ID
        && scene.scene_id == SCENE_ID
        && actors.len() == 1
        && actors[0].id == ACTOR_ID
        && actors[0].entity_type.as_deref() == Some(ENTITY_TYPE)
        && actors[0].position == [18.0, 0.0, 8.0]
}
fn registered(state: &RuntimeState, scene: &SceneRuntime) -> bool {
    canonical_scene(scene.current_scene())
        && state.world.world_id == WORLD_ID
        && state.world.scene_id == SCENE_ID
        && state.world.revision.world_epoch == scene.world_epoch
        && state.route.current_world_id == WORLD_ID
        && state.world.generic_actors.len() == 1
        && state.world.generic_actors[0].entity_id == ACTOR_ID
        && state.world.generic_actors[0].entity_type == ENTITY_TYPE
        && state.world.generic_actors[0].home_m == Vec3::new(18.0, 0.0, 8.0).unwrap()
        && state.world.generic_actors[0].validate()
}
pub(super) struct CombatProof {
    epoch: u64,
    next_tick: u64,
}
pub(super) fn capture_combat(
    state: &RuntimeState,
    scene: Option<&SceneRuntime>,
) -> Option<CombatProof> {
    let scene = scene?;
    if !registered(state, scene)
        || state.world_persistent_v1.mist_harbor.warden_defeated
        || state.world.generic_actors[0].hp == 0
    {
        return None;
    }
    Some(CombatProof {
        epoch: state.world.revision.world_epoch,
        next_tick: state.world.revision.server_tick.checked_add(1)?,
    })
}
pub(super) fn record_defeat(
    state: &mut RuntimeState,
    scene: Option<&SceneRuntime>,
    proof: Option<CombatProof>,
    events: &[CombatEvent],
) -> bool {
    let (Some(scene), Some(proof)) = (scene, proof) else {
        return false;
    };
    if !registered(state,scene)||proof.epoch!=state.world.revision.world_epoch||proof.next_tick!=state.world.revision.server_tick
        ||state.world_persistent_v1.mist_harbor.warden_defeated||state.world.generic_actors[0].hp!=0
        ||!events.iter().any(|e|matches!(e,CombatEvent::AttackHit{target_id,damage,..}|CombatEvent::ActionImpact{target_id,damage,..} if target_id==ACTOR_ID&&*damage>0)) {return false;}
    state.world_persistent_v1.mist_harbor.warden_defeated = true;
    emit_presentation(
        state,
        "ResonanceWardenDeath",
        state.world.generic_actors[0].position_m,
        1.0,
    );
    true
}
pub(super) fn prepare_scene(
    state: &mut RuntimeState,
    scene: &crate::scene_runtime::SceneDefinition,
) -> Result<(), String> {
    prepare_world(
        &mut state.world,
        scene,
        state.world_persistent_v1.mist_harbor.warden_defeated,
    )
}
pub(super) fn prepare_world(
    world: &mut WorldStateV3,
    scene: &crate::scene_runtime::SceneDefinition,
    defeated: bool,
) -> Result<(), String> {
    if scene.world_id != WORLD_ID || scene.scene_id != SCENE_ID {
        return Ok(());
    }
    if !canonical_scene(scene) {
        return Err("E_WARDEN_AUTHORED_ROSTER".into());
    }
    // Pre-Warden valid saves can contain an empty arena. Installing the new
    // encounter restores a live Boss, never an inferred historical defeat.
    if world.generic_actors.is_empty() && !defeated {
        world.generic_actors.push(ActorRuntime::spawn(
            ACTOR_ID,
            ENTITY_TYPE,
            Vec3::new(18.0, 0.0, 8.0).unwrap(),
        )?);
    }
    if world.generic_actors.len() != 1
        || world.generic_actors[0].entity_id != ACTOR_ID
        || world.generic_actors[0].entity_type != ENTITY_TYPE
        || !world.generic_actors[0].validate()
    {
        return Err("E_WARDEN_SAVED_ROSTER".into());
    }
    if defeated {
        world.generic_actors[0].take_damage(u32::MAX);
    } else if world.generic_actors[0].hp == 0 {
        return Err("E_WARDEN_DEAD_WITHOUT_PROOF".into());
    }
    Ok(())
}
pub(crate) fn validate_saved(
    save: &crate::save_v5::SaveV5,
    persistent: &crate::world_persistent_v1::WorldPersistentState,
) -> Result<(), String> {
    let wardens: Vec<_> = save
        .generic_actors
        .iter()
        .filter(|a| a.entity_type == ENTITY_TYPE || a.warden.is_some())
        .collect();
    if !wardens.is_empty() {
        if save.world_id != WORLD_ID
            || save.scene_id != SCENE_ID
            || wardens.len() != 1
            || save.generic_actors.len() != 1
            || wardens[0].entity_id != ACTOR_ID
            || wardens[0].entity_type != ENTITY_TYPE
            || wardens[0].home_m != Vec3::new(18.0, 0.0, 8.0).unwrap()
            || (wardens[0].hp == 0) != persistent.mist_harbor.warden_defeated
        {
            return Err("E_SAVE_WARDEN_STATE_INVALID".into());
        }
    } else if save.world_id == WORLD_ID
        && save.scene_id == SCENE_ID
        && (!save.generic_actors.is_empty() || persistent.mist_harbor.warden_defeated)
    {
        return Err("E_SAVE_WARDEN_STATE_INVALID".into());
    }
    Ok(())
}
pub(super) fn project(
    state: &RuntimeState,
    scene: &SceneRuntime,
) -> Option<crate::boss_view_v1::BossEncounterView> {
    if !registered(state, scene) || state.world.generic_actors[0].hp == 0 {
        return None;
    }
    let actor = &state.world.generic_actors[0];
    let controller = actor.warden.as_ref()?;
    let profile = crate::world_v3::actor_profile_for_warden()?;
    let config = profile.warden.as_ref()?;
    let mapped_true_source = controller.stage == WardenStage::Windup
        && state
            .effective_rules_v6
            .capability_permissions
            .contains(&CapabilityPermission::AcousticMapping);
    let warning = if controller.stage == WardenStage::Windup
        && (mapped_true_source || controller.remaining_ms <= config.ordinary_warning_ms)
    {
        let kind = controller.attack?;
        let origin = controller.committed_origin_m?;
        Some(crate::boss_view_v1::BossWarningView {
            attack_serial: actor.attack_serial,
            kind: match kind {
                WardenAttack::Strike => "strike",
                WardenAttack::Pulse => "pulse",
            }
            .into(),
            origin_m: [origin.x_m, origin.y_m, origin.z_m],
            direction_rad: controller.committed_direction_rad?,
            radius_m: config.radius(kind),
            half_angle_rad: config.strike_half_angle_rad,
            remaining_ms: controller.remaining_ms,
            ordinary_visible: controller.remaining_ms <= config.ordinary_warning_ms,
        })
    } else if controller.stage == WardenStage::Decoy {
        let origin = controller.committed_origin_m?;
        let direction = controller.committed_direction_rad? + std::f32::consts::FRAC_PI_2;
        let offset = Vec3 {
            x_m: origin.x_m + direction.sin() * 2.0,
            y_m: origin.y_m,
            z_m: origin.z_m + direction.cos() * 2.0,
        };
        let point = if state.kcc.can_occupy(offset, 0.05) {
            offset
        } else {
            origin
        };
        Some(crate::boss_view_v1::BossWarningView {
            attack_serial: actor.attack_serial,
            kind: "decoy".into(),
            origin_m: [point.x_m, point.y_m, point.z_m],
            direction_rad: 0.0,
            radius_m: 1.0,
            half_angle_rad: std::f32::consts::PI,
            remaining_ms: controller.remaining_ms,
            ordinary_visible: true,
        })
    } else {
        None
    };
    Some(crate::boss_view_v1::BossEncounterView {
        entity_id: ACTOR_ID.into(),
        entity_type: ENTITY_TYPE.into(),
        display_name: "Resonance Warden".into(),
        current_hp: actor.hp,
        max_hp: profile.max_hp,
        phase: config.phase(actor.hp, profile.max_hp),
        state: format!("{:?}", controller.stage).to_lowercase(),
        temporary_visual: true,
        public_release_eligible: false,
        mapped_true_source,
        warning,
    })
}
pub(super) fn present_events(
    state: &mut RuntimeState,
    scene: Option<&SceneRuntime>,
    events: &[crate::world_v3::ActorRuntimeEvent],
) {
    let Some(scene) = scene else { return };
    if !registered(state, scene) {
        return;
    }
    for event in events {
        let crate::world_v3::ActorRuntimeEvent::WardenCue {
            actor_id,
            kind,
            origin_m,
            direction_rad,
            radius_m,
            duration_ms: _,
            attack_serial: _,
        } = event
        else {
            continue;
        };
        if actor_id != ACTOR_ID {
            continue;
        }
        let label = match kind {
            WardenCueKind::Decoy => "ResonanceWardenDecoy",
            WardenCueKind::StrikeWindup => "ResonanceWardenStrikeWindup",
            WardenCueKind::PulseWindup => "ResonanceWardenPulseWindup",
            WardenCueKind::StrikeImpact => "ResonanceWardenStrikeImpact",
            WardenCueKind::PulseImpact => "ResonanceWardenPulseImpact",
        };
        emit_presentation_sized(state, label, *origin_m, *radius_m, 1.0);
        if matches!(
            kind,
            WardenCueKind::StrikeWindup | WardenCueKind::PulseWindup
        ) {
            // Optional mapping only reveals a true, currently emitted call. Decoys
            // never enter this channel; later grants cannot recover old bearings.
            let mapped = state
                .effective_rules_v6
                .capability_permissions
                .contains(&CapabilityPermission::AcousticMapping);
            if let Some(next_id) = state.next_sound_cue_id.checked_add(1) {
                let dx = origin_m.x_m - state.world.player.position_m.x_m;
                let dz = origin_m.z_m - state.world.player.position_m.z_m;
                state.sound_cues.push(SoundCueEvent {
                    protocol_version: 1,
                    event_id: state.next_sound_cue_id,
                    world_epoch: state.world.revision.world_epoch,
                    server_tick: state.world.revision.server_tick,
                    world_id: WORLD_ID.into(),
                    scene_id: SCENE_ID.into(),
                    kind: "warden_true_call".into(),
                    direction_rad: mapped.then(|| dx.atan2(dz)),
                    distance_m: mapped.then(|| (dx * dx + dz * dz).sqrt()),
                });
                state.next_sound_cue_id = next_id;
                if state.sound_cues.len() > 256 {
                    state.sound_cues.remove(0);
                }
            }
        }
        // Preserve the committed direction in the event rather than retargeting its warning.
        if let Some(event) = state.presentation_events.last_mut() {
            event.direction_rad = *direction_rad;
        }
    }
}

#[cfg(test)]
mod tests;
