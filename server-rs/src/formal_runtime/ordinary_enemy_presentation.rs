//! Snapshot-owned ordinary enemy presentation. Geometry and lifetime come from
//! the authoritative controller; this channel never exposes HP or applies damage.
use super::*;
use crate::world_v3::{ActorAiState, ActorAttackKind, ActorCueKind, ActorRuntimeEvent};

fn attack_name(entity_type: &str, kind: ActorAttackKind) -> Option<&'static str> {
    if entity_type == signal_wraith_roster::WRAITH {
        return (kind == ActorAttackKind::SignalShot).then_some("signal_shot");
    }
    if entity_type == tidebound_roster::TIDEBOUND {
        return match kind { ActorAttackKind::Swing => Some("tide_swing"), ActorAttackKind::Charge => Some("tide_charge"), _ => None };
    }
    match kind {
        ActorAttackKind::PressureShot => Some("pressure_shot"),
        ActorAttackKind::Bite => Some("bite"),
        ActorAttackKind::Leap => Some("leap"),
        ActorAttackKind::Charge => Some("charge"),
        ActorAttackKind::Slam => Some("slam"),
        ActorAttackKind::Lunge => Some("lunge"),
        _ => None,
    }
}

fn geometry(actor: &crate::world_v3::ActorRuntime, kind: ActorAttackKind) -> Option<(f32, f32, Option<f32>)> {
    let profile = actor.ordinary_profile_for_current_phase()?;
    let ordinary = profile.ordinary.as_ref()?;
    match kind {
        ActorAttackKind::PressureShot | ActorAttackKind::SignalShot => ordinary.projectile.as_ref().map(|p| (p.radius_m, p.travel_m, None)),
        ActorAttackKind::Leap => ordinary.leap.as_ref().map(|p| (p.hit_radius_m, p.speed_mps * p.active_ms as f32 / 1000.0, None)),
        ActorAttackKind::Bite if ordinary.leap.is_some() => Some((profile.attack_range_m, profile.attack_range_m, None)),
        ActorAttackKind::Charge => ordinary.charge.as_ref().map(|p| (p.hit_radius_m, p.speed_mps * p.active_ms as f32 / 1000.0, None)),
        ActorAttackKind::Lunge => ordinary.lunge.as_ref().map(|p| (p.hit_radius_m, p.speed_mps * p.active_ms as f32 / 1000.0, None)),
        ActorAttackKind::Slam if ordinary.charge.is_some() => Some((profile.attack_range_m, profile.attack_range_m, None)),
        ActorAttackKind::Swing => ordinary.directional_half_angle_rad.map(|angle| (profile.attack_range_m, profile.attack_range_m, Some(angle))),
        _ => None,
    }
}

// Controller timers round one 60 Hz owner step to 17 ms. Presentation timelines
// use the server tick, so convert remaining owner steps rather than drifting by
// treating the controller's rounded milliseconds as wall time.
fn duration_on_tick_timeline(state: &RuntimeState, duration_ms: u64) -> u64 {
    if duration_ms == 0 { return 0; }
    let owner_step_ms = (state.stepper.config.dt_s() * 1000.0).round() as u64;
    let ticks = duration_ms.div_ceil(owner_step_ms.max(1));
    (ticks * 1000).div_ceil(u64::from(state.stepper.config.hz))
}

pub(super) fn present_events(
    state: &mut RuntimeState,
    scene: Option<&SceneRuntime>,
    events: &[ActorRuntimeEvent],
) {
    let Some(scene) = scene else { return };
    let definition = scene.current_scene();
    if state.world.world_id != definition.world_id
        || state.world.scene_id != definition.scene_id || scene.world_epoch != state.world.revision.world_epoch
    { return; }
    for event in events {
        let member_id = match event { ActorRuntimeEvent::MemberHit { member_id, .. } => Some(member_id.clone()), _ => None };
        let (actor_id, kind, position, direction, duration, attack) = match event {
            ActorRuntimeEvent::Relocated { actor_id, from_m, to_m } => (
                actor_id, "SignalBlink", *from_m, (to_m.x_m-from_m.x_m).atan2(to_m.z_m-from_m.z_m), 255, None,
            ),
            ActorRuntimeEvent::MemberHit { actor_id, position_m, destroyed, .. } => (
                actor_id, if *destroyed { "EnemyMemberDisperse" } else { "EnemyMemberHit" }, *position_m, 0.0, 170, None,
            ),
            ActorRuntimeEvent::EnemyCue { actor_id, kind, origin_m, duration_ms } => (
                actor_id,
                match kind { ActorCueKind::Alert => "EnemyAlert", ActorCueKind::Stagger => "EnemyStagger", ActorCueKind::Death => "EnemyDeath" },
                *origin_m, 0.0, *duration_ms, None,
            ),
            ActorRuntimeEvent::AttackWindup { actor_id, kind, origin_m, windup_ms, .. } => (
                actor_id, "EnemyAttackTelegraph", *origin_m, 0.0, *windup_ms, Some(*kind),
            ),
            ActorRuntimeEvent::AttackMotion { actor_id, kind, from_m, to_m, remaining_ms, .. } => (
                actor_id,
                if *kind == ActorAttackKind::SignalShot { "SignalShotMotion" } else if *kind == ActorAttackKind::PressureShot { "PressureShotMotion" } else if *kind == ActorAttackKind::Leap { "FurnaceHoundLeapMotion" }
                else if *kind == ActorAttackKind::Charge { "EnemyChargeMotion" }
                else if *kind == ActorAttackKind::Lunge { "EnemyLungeMotion" } else { continue },
                *to_m, (to_m.x_m - from_m.x_m).atan2(to_m.z_m - from_m.z_m), *remaining_ms, Some(*kind),
            ),
            ActorRuntimeEvent::AttackImpact { actor_id, kind, origin_m, .. } => (
                // Impact/death flash lengths are presentation-only development tuning.
                actor_id, "EnemyAttackImpact", *origin_m, 0.0, 170, Some(*kind),
            ),
            _ => continue,
        };
        let mut matches = state.world.generic_actors.iter().filter(|actor| &actor.entity_id == actor_id);
        let Some(actor) = matches.next() else { continue };
        let source_matches = match actor.entity_type.as_str() {
            clockworks_roster::DRONE | clockworks_roster::HOUND => definition.world_id == "clockworks",
            signal_wraith_roster::WRAITH => actor.controller_variant == 1 && definition.world_id == "mist_harbor" && signal_wraith_roster::SCENES.contains(&definition.scene_id.as_str()),
            tidebound_roster::TIDEBOUND => definition.world_id == "mist_harbor" && matches!(definition.scene_id.as_str(), "mh_drowned_quay" | "mh_breakwater" | "mh_resonance_tower"),
            swarm_roster::SWARM => definition.world_id == "grey_hive" && matches!(definition.scene_id.as_str(), "gh_lockdown" | "gh_deep_decon"),
            grey_hive_roster::BRUTE => definition.world_id == "grey_hive" && definition.scene_id == "gh_gate_b",
            _ => false,
        };
        if matches.next().is_some() || !actor.validate() || !source_matches
            || (actor.hp == 0 && !matches!(kind, "EnemyDeath" | "EnemyMemberDisperse"))
            || (kind == "EnemyDeath" && actor.hp != 0)
            || definition.spawns.iter().filter(|spawn| &spawn.id == actor_id && spawn.kind == "enemy"
                && spawn.entity_type.as_deref() == Some(actor.entity_type.as_str())
                && vec3_from_array(spawn.position) == actor.home_m).count() != 1
        { continue; }
        let member = if let Some(id) = &member_id {
            let Some(member) = actor.view().members.and_then(|members| members.into_iter().find(|member| &member.member_id == id)) else { continue };
            if member.active != (kind == "EnemyMemberHit") { continue; }
            Some(member)
        } else { None };
        if matches!(event, ActorRuntimeEvent::AttackMotion { .. })
            && (actor.state != ActorAiState::Active || actor.ordinary.as_ref()
                .and_then(|state| state.active.as_ref()).map(|active| active.kind) != attack)
        { continue; }
        let Some(profile) = actor.ordinary_profile_for_current_phase() else { continue };
        let Some(ordinary) = profile.ordinary.as_ref() else { continue };
        if let ActorRuntimeEvent::Relocated { from_m, to_m, .. } = event {
            let Some(signal) = ordinary.signal.as_ref() else { continue };
            if actor.entity_type != signal_wraith_roster::WRAITH || *to_m != actor.position_m
                || from_m.y_m != to_m.y_m
                || ((to_m.x_m-from_m.x_m).hypot(to_m.z_m-from_m.z_m)-signal.relocate_distance_m).abs()>0.0001 { continue; }
        }
        // A recovered terrain attack has already cleared its saved variant. The
        // impact receipt carries the geometry it actually used before recovery.
        let captured = if matches!(actor.entity_type.as_str(), tidebound_roster::TIDEBOUND | signal_wraith_roster::WRAITH) {
            match event {
                ActorRuntimeEvent::AttackImpact { radius_m, geometry: Some(geometry), .. } => Some((*radius_m, *geometry)),
                ActorRuntimeEvent::AttackImpact { geometry: None, .. } => continue,
                _ => None,
            }
        } else { None };
        let (radius, range, half_angle, attack_kind) = if let Some(attack) = attack {
            let Some(name) = attack_name(&actor.entity_type, attack) else { continue };
            let (radius, range, half_angle) = if let Some((radius, geometry)) = captured {
                (radius, geometry.range_m, geometry.half_angle_rad)
            } else {
                let Some(geometry) = geometry(actor, attack) else { continue }; geometry
            };
            (radius, Some(range), half_angle, Some(name.to_owned()))
        } else { (member.as_ref().map_or(ordinary.body_radius_m, |member| member.radius_m),
            match event { ActorRuntimeEvent::Relocated { from_m, to_m, .. } => Some((to_m.x_m-from_m.x_m).hypot(to_m.z_m-from_m.z_m)), _ => None }, None, None) };
        let direction = if kind == "EnemyAttackTelegraph" {
            let Some(direction) = actor.ordinary.as_ref().and_then(|ordinary| ordinary.committed_direction) else { continue };
            direction[0].atan2(direction[1])
        } else if matches!(event, ActorRuntimeEvent::AttackMotion { .. }) {
            // A restored attack can be Active at elapsed_ms == 0. Its segment
            // has zero displacement but its committed direction is still real.
            let active = actor.ordinary.as_ref().and_then(|state| state.active.as_ref()).expect("active motion checked above");
            active.direction[0].atan2(active.direction[1])
        } else { captured.map_or(direction, |(_, geometry)| geometry.direction_rad) };
        let position = if actor.entity_type == signal_wraith_roster::WRAITH && kind == "EnemyAlert" {
            let Some(projection) = signal_wraith_presentation::projection(state, actor) else { continue };
            signal_wraith_presentation::anchor(&projection)
        } else { position };
        if actor.entity_type == tidebound_roster::TIDEBOUND
            && ((attack == Some(ActorAttackKind::Swing)) != half_angle.is_some()) { continue; }
        if actor.entity_type == signal_wraith_roster::WRAITH && half_angle.is_some() { continue; }
        if range.is_some_and(|range| !range.is_finite() || range <= 0.0)
            || half_angle.is_some_and(|angle| !angle.is_finite() || angle <= 0.0 || angle > std::f32::consts::PI) { continue; }
        if ![position.x_m, position.y_m, position.z_m, direction, radius].iter().all(|value| value.is_finite()) { continue; }
        let duration = if kind == "EnemyDeath" { 340 } else if kind == "EnemyAlert" { 255 } else { duration };
        let duration = duration_on_tick_timeline(state, duration);
        emit_presentation_sized(state, kind, position, radius, 1.0);
        if let Some(event) = state.presentation_events.last_mut() {
            event.direction_rad = direction;
            event.actor_id = Some(actor_id.clone());
            event.member_id = member_id;
            event.half_angle_rad = half_angle;
            event.duration_ms = Some(duration);
            event.attack_kind = attack_kind;
            event.range_m = range;
        }
    }
}

/// Restore current telegraph/motion once in the fresh entry epoch. These are
/// transient cues, never reward/combat events, and never modify saved AI timers.
pub(super) fn prepare_restored(state: &mut RuntimeState, scene: &SceneRuntime) {
    let mut events = Vec::new();
    for actor in &state.world.generic_actors {
        let Some(ordinary) = actor.ordinary.as_ref() else { continue };
        if actor.state == ActorAiState::Windup {
            if let (Some(kind), Some(origin)) = (actor.current_attack, actor.windup_origin_m) {
                events.push(ActorRuntimeEvent::AttackWindup { actor_id: actor.entity_id.clone(), kind,
                    origin_m: origin, radius_m: 0.0, windup_ms: actor.state_remaining_ms });
            }
        } else if actor.state == ActorAiState::Active {
            if let Some(active) = &ordinary.active {
                let Some(resolved) = actor.ordinary_profile_for_current_phase() else { continue };
                let Some(profile) = resolved.ordinary.as_ref() else { continue };
                let speed = match active.kind {
                    ActorAttackKind::PressureShot | ActorAttackKind::SignalShot => profile.projectile.as_ref().map(|p| p.speed_mps),
                    ActorAttackKind::Leap => profile.leap.as_ref().map(|p| p.speed_mps),
                    ActorAttackKind::Charge => profile.charge.as_ref().map(|p| p.speed_mps),
                    ActorAttackKind::Lunge => profile.lunge.as_ref().map(|p| p.speed_mps),
                    _ => None,
                };
                if let Some(speed) = speed {
                    let mut distance = speed * active.elapsed_ms as f32 / 1000.0;
                    if matches!(active.kind, ActorAttackKind::PressureShot | ActorAttackKind::SignalShot) {
                        distance = distance.min(profile.projectile.as_ref().unwrap().travel_m);
                    }
                    let point = Vec3 { x_m: active.origin_m.x_m + active.direction[0] * distance,
                        y_m: active.origin_m.y_m, z_m: active.origin_m.z_m + active.direction[1] * distance };
                    events.push(ActorRuntimeEvent::AttackMotion { actor_id: actor.entity_id.clone(), kind: active.kind,
                        from_m: active.origin_m, to_m: point, radius_m: 0.0, remaining_ms: actor.state_remaining_ms });
                }
            }
        }
    }
    present_events(state, Some(scene), &events);
}

#[cfg(test)]
mod tests;
