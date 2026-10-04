//! Shared data-driven hazard geometry and effective-rule damage. No scene IDs.
use super::*;
use crate::effects::HazardTag;
use crate::scene_runtime::HazardDefinition;

pub(super) struct HazardFrame {
    pub polygon: Vec<[f32; 2]>,
    pub warning_remaining_ms: Option<u64>,
}

/// Reciprocating motion has an initial warning and a safe hold before each reversal.
/// The authority's simulation clock drives this, so pause and Continue cannot drift.
pub(super) fn motion_frame(
    hazard: &HazardDefinition,
    started_at_ms: u64,
    now: u64,
) -> Option<HazardFrame> {
    let [dx, dz] = hazard.translation_m?;
    let speed = hazard.speed_mps?;
    let hold = hazard.endpoint_hold_ms?;
    let warning = hazard.warning_ms?;
    if !dx.is_finite() || !dz.is_finite() || !speed.is_finite() || speed <= 0.0 {
        return None;
    }
    let travel = ((f64::from(dx).hypot(f64::from(dz)) / f64::from(speed)) * 1000.0).ceil() as u64;
    if travel == 0 {
        return None;
    }
    let half_cycle = travel.checked_add(hold)?;
    let cycle = half_cycle.checked_mul(2)?;
    let elapsed = now.saturating_sub(started_at_ms);
    let (fraction, remaining) = if elapsed < warning {
        (0.0, Some(warning - elapsed))
    } else {
        let t = (elapsed - warning) % cycle;
        if t < travel {
            (t as f32 / travel as f32, None)
        } else if t < half_cycle {
            (1.0, Some(half_cycle - t))
        } else if t < half_cycle + travel {
            (1.0 - (t - half_cycle) as f32 / travel as f32, None)
        } else {
            (0.0, Some(cycle - t))
        }
    };
    Some(HazardFrame {
        polygon: hazard
            .polygon
            .iter()
            .map(|[x, z]| [x + dx * fraction, z + dz * fraction])
            .collect(),
        warning_remaining_ms: remaining,
    })
}

pub(super) fn apply_hazard_damage(
    state: &mut RuntimeState,
    damage: u32,
    tag: HazardTag,
    feedback: &str,
) {
    let remaining = state.effective_rules_v6.hazards.remaining_damage_bps(tag);
    let applied = ((u64::from(damage) * u64::from(remaining) + 5_000) / 10_000) as u32;
    if applied > 0 && state.world.player_hp > 0 {
        state.world.player_hp = state.world.player_hp.saturating_sub(applied);
        state.last_damaged_at_ms = Some(state.world.server_time_ms);
        emit_presentation(state, feedback, state.world.player.position_m, 1.0);
    }
}

pub(super) fn advance_machinery(
    state: &mut RuntimeState,
    hazards: &[HazardDefinition],
    started_at_ms: u64,
) -> Result<(), String> {
    let now = state.world.server_time_ms;
    for hazard in hazards
        .iter()
        .filter(|h| h.kind == "moving_machinery" && h.phase.as_deref() == Some("phase3"))
    {
        let frame = motion_frame(hazard, started_at_ms, now).ok_or("E_HAZARD_MOTION_INVALID")?;
        let damage = hazard.damage.ok_or("E_HAZARD_DAMAGE_MISSING")?;
        let period = hazard.period_ms.ok_or("E_HAZARD_PERIOD_MISSING")?;
        let next = state
            .world_persistent_v1
            .clockworks
            .regulator_hazard_next_damage_at_ms
            .get(&hazard.id)
            .copied()
            .unwrap_or(0);
        if frame.warning_remaining_ms.is_none()
            && hazard.height_allows(state.world.player.position_m.y_m)
            && now >= next
            && point_in_or_on_polygon(
                state.world.player.position_m.x_m,
                state.world.player.position_m.z_m,
                &frame.polygon,
            )
        {
            apply_hazard_damage(state, damage, HazardTag::Machinery, "MachineryDamage");
            state
                .world_persistent_v1
                .clockworks
                .regulator_hazard_next_damage_at_ms
                .insert(hazard.id.clone(), now.saturating_add(period));
        }
    }
    Ok(())
}

pub(super) fn project_hazard(
    hazard: &HazardDefinition,
    encounter: &crate::world_persistent_v1::ClockworksPersistentState,
    now: u64,
    live: bool,
) -> Option<crate::world_v3::HazardView> {
    let (phase, active, polygon, warning) = match (hazard.kind.as_str(), hazard.phase.as_deref()) {
        ("heat_zone", Some("phase2")) => {
            let phase = live && encounter.regulator_phase2_active;
            let active = phase && now >= encounter.regulator_heat_cooled_until_ms;
            let warning = (active && encounter.regulator_heat_warning_until_ms > now)
                .then(|| encounter.regulator_heat_warning_until_ms - now);
            (phase, active, hazard.polygon.clone(), warning)
        }
        ("moving_machinery", Some("phase3")) => {
            let phase = live && encounter.regulator_phase3_active;
            let frame = motion_frame(hazard, encounter.regulator_phase3_started_at_ms, now)?;
            (
                phase,
                phase,
                if phase {
                    frame.polygon
                } else {
                    hazard.polygon.clone()
                },
                if phase {
                    frame.warning_remaining_ms
                } else {
                    None
                },
            )
        }
        _ => return None,
    };
    Some(crate::world_v3::HazardView {
        environment: None,
        height_range_m: hazard.height_range_m,
        entity_id: hazard.id.clone(),
        kind: hazard.kind.clone(),
        transform: Transform {
            position_m: {let mut p=polygon_center(&polygon);p.y_m=hazard.presentation_height();p},
            yaw_rad: 0.0,
        },
        active,
        polygon_m: Some(polygon),
        warning_remaining_ms: warning,
        phase_active: Some(phase),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reciprocation_has_exact_warning_travel_hold_return_and_cycle() {
        let hazard: HazardDefinition = serde_json::from_value(serde_json::json!({
            "id":"press", "kind":"moving_machinery", "polygon":[[8,11.5],[10,11.5],[10,13],[8,13]],
            "damage":16,"periodMs":750,"warningMs":900,"phase":"phase3",
            "translationM":[6,0],"speedMps":2,"endpointHoldMs":600
        }))
        .unwrap();
        for (time, x, warning) in [
            (100, 8.0, Some(900)),
            (999, 8.0, Some(1)),
            (1000, 8.0, None),
            (2500, 11.0, None),
            (4000, 14.0, Some(600)),
            (4599, 14.0, Some(1)),
            (4600, 14.0, None),
            (6100, 11.0, None),
            (7600, 8.0, Some(600)),
            (8200, 8.0, None),
        ] {
            let frame = motion_frame(&hazard, 100, time).unwrap();
            assert_eq!(frame.polygon[0], [x, 11.5], "time={time}");
            assert_eq!(frame.warning_remaining_ms, warning, "time={time}");
        }
    }
}

#[cfg(test)]
mod vertical_tests {
    use super::*;
    #[test]
    fn machinery_applies_its_authored_vertical_band_before_damage_or_cooldown() {
        for (player_y,range,hit) in [(0.,None,true),(2.,None,false),(0.,Some([1.9,2.1]),false),(2.,Some([1.9,2.1]),true)] {
            let mut state=FormalRuntime::initial_state(7).unwrap();state.world.player.position_m=Vec3{x_m:2.,y_m:player_y,z_m:2.};state.world.server_time_ms=1000;
            let mut hazard:HazardDefinition=serde_json::from_value(serde_json::json!({"id":"vertical_press","kind":"moving_machinery","polygon":[[1.,1.],[3.,1.],[3.,3.],[1.,3.]],"damage":16,"periodMs":750,"warningMs":900,"phase":"phase3","translationM":[1.,0.],"speedMps":1.,"endpointHoldMs":600})).unwrap();hazard.height_range_m=range.map(crate::moving_support::HeightRange);
            let hp=state.world.player_hp;advance_machinery(&mut state,&[hazard],0).unwrap();assert_eq!(state.world.player_hp<hp,hit);assert_eq!(!state.world_persistent_v1.clockworks.regulator_hazard_next_damage_at_ms.is_empty(),hit);
        }
    }
}
