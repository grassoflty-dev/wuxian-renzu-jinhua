//! Generic, profile-driven ordinary actor attacks. All numeric profile values are
//! IMPLEMENTATION TUNING, not frozen source requirements. Projectile, bite and
//! leap reuse ActorRuntime, authoritative collision, and the owner's damage path.
use super::*;
use crate::continuous_kcc::KccBody;
use crate::effects::{HazardTag, TerrainTag};
use std::borrow::Cow;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrdinaryCombatProfile {
    pub body_radius_m: f32,
    pub investigate_ms: u64,
    pub stagger_ms: u64,
    pub stagger_threshold: u32,
    #[serde(default)]
    pub projectile: Option<ProjectileProfile>,
    #[serde(default)]
    pub leap: Option<MotionAttackProfile>,
    #[serde(default)]
    pub charge: Option<MotionAttackProfile>,
    #[serde(default)]
    pub lunge: Option<MotionAttackProfile>,
    #[serde(default)]
    pub directional_half_angle_rad: Option<f32>,
    #[serde(default)]
    pub terrain_override: Option<TerrainCombatOverride>,
    #[serde(default)]
    pub signal: Option<SignalCombatProfile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectileProfile {
    #[serde(default)]
    pub kind: ProjectileKind,
    pub speed_mps: f32,
    pub radius_m: f32,
    pub travel_m: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileKind { #[default] Pressure, Signal }
impl ProjectileKind {
    fn attack(self) -> ActorAttackKind { match self { Self::Pressure => ActorAttackKind::PressureShot, Self::Signal => ActorAttackKind::SignalShot } }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignalCombatProfile {
    pub interference_range_m: f32,
    pub relocate_trigger_m: f32,
    pub relocate_distance_m: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionAttackProfile {
    pub min_range_m: f32,
    pub range_m: f32,
    pub speed_mps: f32,
    pub hit_radius_m: f32,
    pub active_ms: u64,
    pub windup_ms: u64,
    pub cooldown_ms: u64,
    pub damage: u32,
}

/// Optional shared terrain variant. No new terrain, immunity or tide clock is created.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerrainCombatOverride {
    pub tags: Vec<TerrainTag>,
    pub speed_mps: f32,
    pub attack_range_m: f32,
    pub attack_damage: u32,
    pub windup_ms: u64,
    pub cooldown_ms: u64,
    pub charge: MotionAttackProfile,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrdinaryTerrainVariant { Normal, Terrain }

pub(super) fn resolved_profile(profile: &ActorProfile, variant: Option<OrdinaryTerrainVariant>) -> Cow<'_, ActorProfile> {
    let Some(terrain) = profile.ordinary.as_ref().and_then(|p| p.terrain_override.as_ref())
        .filter(|_| variant == Some(OrdinaryTerrainVariant::Terrain)) else { return Cow::Borrowed(profile); };
    let mut resolved = profile.clone();
    resolved.speed_mps = terrain.speed_mps; resolved.attack_range_m = terrain.attack_range_m;
    resolved.attack_damage = terrain.attack_damage; resolved.windup_ms = terrain.windup_ms;
    resolved.cooldown_ms = terrain.cooldown_ms;
    resolved.ordinary.as_mut().unwrap().charge = Some(terrain.charge.clone());
    Cow::Owned(resolved)
}
pub(super) fn terrain_variant_for_tick(actor: &ActorRuntime, profile: &ActorProfile, kcc: &StaticKccWorld) -> Option<OrdinaryTerrainVariant> {
    let config = profile.ordinary.as_ref()?;
    let terrain = config.terrain_override.as_ref()?;
    if matches!(actor.state, ActorAiState::Windup | ActorAiState::Active) {
        return actor.ordinary.as_ref().and_then(|state| state.terrain_variant);
    }
    Some(if kcc.intersects_terrain_at(actor.position_m, config.body_radius_m, &terrain.tags) {
        OrdinaryTerrainVariant::Terrain
    } else { OrdinaryTerrainVariant::Normal })
}
impl TerrainCombatOverride {
    fn validate(&self, profile: &ActorProfile, config: &OrdinaryCombatProfile) -> bool {
        if config.charge.is_none() || config.directional_half_angle_rad.is_none() || profile.group.is_some()
            || self.tags.is_empty() || self.tags.len() > 8
            || self.tags.iter().collect::<std::collections::BTreeSet<_>>().len() != self.tags.len()
            || !positive(self.speed_mps) || !positive(self.attack_range_m)
            || self.attack_range_m > profile.perception_m || self.attack_damage == 0
            || self.windup_ms == 0 || self.cooldown_ms < profile.cooldown_ms
            || self.charge.cooldown_ms < config.charge.as_ref().unwrap().cooldown_ms { return false; }
        let mut resolved = resolved_profile(profile, Some(OrdinaryTerrainVariant::Terrain)).into_owned();
        resolved.ordinary.as_mut().unwrap().terrain_override = None;
        resolved.ordinary.as_ref().unwrap().validate(&resolved)
    }
}

impl OrdinaryCombatProfile {
    pub fn validate(&self, profile: &ActorProfile) -> bool {
        positive(self.body_radius_m) && self.body_radius_m <= profile.attack_range_m
            && self.investigate_ms > 0 && self.stagger_ms > 0 && self.stagger_threshold > 0
            && self.directional_half_angle_rad.is_none_or(|angle| self.charge.is_some() && positive(angle) && angle <= std::f32::consts::PI)
            && self.terrain_override.as_ref().is_none_or(|terrain| terrain.validate(profile, self))
            && self.signal.as_ref().is_none_or(|signal| {
                self.projectile.as_ref().is_some_and(|p| p.kind == ProjectileKind::Signal)
                    && self.terrain_override.is_none() && profile.group.is_none()
                    && [signal.interference_range_m, signal.relocate_trigger_m, signal.relocate_distance_m].into_iter().all(positive)
                    && signal.interference_range_m <= profile.perception_m
                    && signal.relocate_trigger_m < profile.attack_range_m
                    && signal.relocate_distance_m <= profile.leash_m
            })
            && self.projectile.as_ref().is_none_or(|p| (p.kind == ProjectileKind::Signal) == self.signal.is_some())
            && [self.projectile.is_some(), self.leap.is_some(), self.charge.is_some(), self.lunge.is_some()].into_iter().filter(|present| *present).count() == 1
            && self.projectile.as_ref().is_none_or(|p| {
                positive(p.speed_mps) && positive(p.radius_m) && positive(p.travel_m)
                    && p.radius_m < profile.attack_range_m && p.travel_m >= profile.attack_range_m
                    && p.travel_m <= profile.leash_m && (p.travel_m / p.speed_mps).is_finite() && projectile_ms(p) > 0
            })
            && [(self.leap.as_ref(), false), (self.charge.as_ref(), false), (self.lunge.as_ref(), true)]
                .into_iter().filter_map(|(p, close)| p.map(|p| (p, close))).all(|(p, close)| {
                [p.range_m, p.speed_mps, p.hit_radius_m].into_iter().all(positive)
                    && p.min_range_m.is_finite() && p.min_range_m >= if close { 0.0 } else { profile.attack_range_m }
                    && p.min_range_m < p.range_m
                    && p.range_m <= profile.perception_m && p.speed_mps > profile.speed_mps
                    && p.hit_radius_m <= profile.attack_range_m && p.active_ms > 0
                    && p.speed_mps * p.active_ms as f32 / 1000.0 <= profile.leash_m
                    && p.windup_ms > 0 && p.cooldown_ms > 0 && p.damage > 0
            })
    }

    fn motion(&self, kind: ActorAttackKind) -> Option<&MotionAttackProfile> {
        match kind { ActorAttackKind::Leap => self.leap.as_ref(), ActorAttackKind::Charge => self.charge.as_ref(), ActorAttackKind::Lunge => self.lunge.as_ref(), _ => None }
    }
    fn windup_ms(&self, kind: ActorAttackKind, profile: &ActorProfile) -> u64 {
        self.motion(kind).map_or(profile.windup_ms, |motion| motion.windup_ms)
    }
    fn cooldown_ms(&self, kind: ActorAttackKind, profile: &ActorProfile) -> u64 {
        self.motion(kind).map_or(profile.cooldown_ms, |motion| motion.cooldown_ms)
    }
    fn active_ms(&self, kind: ActorAttackKind) -> Option<u64> {
        match kind {
            ActorAttackKind::PressureShot | ActorAttackKind::SignalShot => self.projectile.as_ref().filter(|p| p.kind.attack() == kind).map(projectile_ms),
            ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge => self.motion(kind).map(|p| p.active_ms),
            _ => None,
        }
    }
    fn supported(&self, kind: ActorAttackKind) -> bool {
        match kind {
            ActorAttackKind::PressureShot | ActorAttackKind::SignalShot => self.projectile.as_ref().is_some_and(|p| p.kind.attack() == kind),
            ActorAttackKind::Bite | ActorAttackKind::Leap => self.leap.is_some(),
            ActorAttackKind::Charge => self.charge.is_some(),
            ActorAttackKind::Slam => self.charge.is_some() && self.directional_half_angle_rad.is_none(),
            ActorAttackKind::Swing => self.charge.is_some() && self.directional_half_angle_rad.is_some(),
            ActorAttackKind::Lunge => self.lunge.is_some(),
            _ => false,
        }
    }
    fn radius(&self, kind: ActorAttackKind, profile: &ActorProfile) -> f32 {
        match kind {
            ActorAttackKind::PressureShot | ActorAttackKind::SignalShot => self.projectile.as_ref().unwrap().radius_m,
            ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge => self.motion(kind).unwrap().hit_radius_m,
            _ => profile.attack_range_m,
        }
    }
    fn damage(&self, kind: ActorAttackKind, profile: &ActorProfile) -> u32 {
        self.motion(kind).map_or(profile.attack_damage, |motion| motion.damage)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrdinaryState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain_variant: Option<OrdinaryTerrainVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_m: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_direction: Option<[f32; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<OrdinaryActiveAttack>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrdinaryActiveAttack {
    pub kind: ActorAttackKind,
    pub origin_m: Vec3,
    pub direction: [f32; 2],
    pub elapsed_ms: u64,
    pub hit_resolved: bool,
}

fn positive(v: f32) -> bool { v.is_finite() && v > 0.0 }
fn projectile_ms(p: &ProjectileProfile) -> u64 { (p.travel_m / p.speed_mps * 1000.0).ceil() as u64 }
fn unit(d: [f32; 2]) -> bool { d.into_iter().all(f32::is_finite) && (d[0].hypot(d[1]) - 1.0).abs() <= 0.0001 }
fn near(a: Vec3, b: Vec3) -> bool { horizontal_distance(a, b) <= 0.0001 && (a.y_m - b.y_m).abs() <= 0.0001 }
fn valid_point(p: Vec3, actor: &ActorRuntime, profile: &ActorProfile) -> bool {
    finite(p) && (p.y_m - actor.home_m.y_m).abs() <= 0.0001
        && horizontal_distance(p, actor.home_m) <= profile.leash_m + 0.0001
}
fn active_point(active: &OrdinaryActiveAttack, config: &OrdinaryCombatProfile, elapsed_ms: u64) -> Vec3 {
    let distance = match active.kind {
        ActorAttackKind::PressureShot | ActorAttackKind::SignalShot => {
            let p = config.projectile.as_ref().unwrap();
            (p.speed_mps * elapsed_ms as f32 / 1000.0).min(p.travel_m)
        }
        ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge => config.motion(active.kind).unwrap().speed_mps * elapsed_ms as f32 / 1000.0,
        _ => 0.0,
    };
    offset(active.origin_m, active.direction, distance)
}
fn offset(from: Vec3, direction: [f32; 2], distance: f32) -> Vec3 {
    Vec3 { x_m: from.x_m + direction[0] * distance, y_m: from.y_m, z_m: from.z_m + direction[1] * distance }
}

pub(super) fn validate(actor: &ActorRuntime, profile: &ActorProfile, config: &OrdinaryCombatProfile) -> bool {
    let Some(state) = &actor.ordinary else { return false; };
    if !group_members::validate(actor, profile) { return false; }
    if actor.entity_id.trim().is_empty() || !finite(actor.home_m) || !valid_point(actor.position_m, actor, profile)
        || actor.hp > profile.max_hp || (actor.hp == 0) != (actor.state == ActorAiState::Dead)
        || state.last_seen_m.is_some_and(|p| !valid_point(p, actor, profile))
        || state.committed_direction.is_some_and(|d| !unit(d))
        || actor.attack_serial == u64::MAX {
        return false;
    }
    let committed = matches!(actor.state, ActorAiState::Windup | ActorAiState::Active);
    if (config.terrain_override.is_some() && committed) != state.terrain_variant.is_some()
        || committed != actor.current_attack.is_some() || committed != actor.windup_origin_m.is_some()
        || committed != state.committed_direction.is_some()
        || (actor.state == ActorAiState::Active) != state.active.is_some()
        || actor.current_attack.is_some_and(|kind| !config.supported(kind))
        || actor.windup_origin_m.is_some_and(|p| !valid_point(p, actor, profile)) {
        return false;
    }
    match actor.state {
        ActorAiState::Idle | ActorAiState::Dead => actor.state_remaining_ms == 0 && state.last_seen_m.is_none(),
        ActorAiState::Chasing => actor.state_remaining_ms == 0 && state.last_seen_m.is_some(),
        ActorAiState::Investigate => state.last_seen_m.is_some()
            && (1..=config.investigate_ms).contains(&actor.state_remaining_ms),
        ActorAiState::Stagger => (1..=config.stagger_ms).contains(&actor.state_remaining_ms),
        ActorAiState::Cooldown => {
            let mut max = [&config.leap, &config.charge, &config.lunge].into_iter().flatten().fold(profile.cooldown_ms, |max, p| max.max(p.cooldown_ms));
            if let Some(terrain) = &config.terrain_override { max = max.max(terrain.cooldown_ms).max(terrain.charge.cooldown_ms); }
            (1..=max).contains(&actor.state_remaining_ms)
        }
        ActorAiState::Windup => {
            state.last_seen_m.is_some() && near(actor.windup_origin_m.unwrap(), actor.position_m)
                && (1..=config.windup_ms(actor.current_attack.unwrap(), profile)).contains(&actor.state_remaining_ms)
        }
        ActorAiState::Active => {
            let active = state.active.as_ref().unwrap();
            let Some(total) = config.active_ms(active.kind) else { return false; };
            let expected_actor = if matches!(active.kind, ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge) { active_point(active, config, active.elapsed_ms) } else { active.origin_m };
            state.last_seen_m.is_some() && actor.current_attack == Some(active.kind)
                && actor.windup_origin_m == Some(active.origin_m)
                && state.committed_direction == Some(active.direction) && unit(active.direction)
                && active.elapsed_ms < total && actor.state_remaining_ms == total - active.elapsed_ms
                && (!active.hit_resolved || matches!(active.kind, ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge))
                && near(actor.position_m, expected_actor)
        }
    }
}

fn cue(actor: &ActorRuntime, kind: ActorCueKind, duration_ms: u64) -> ActorRuntimeEvent {
    ActorRuntimeEvent::EnemyCue { actor_id: actor.entity_id.clone(), kind, origin_m: actor.position_m, duration_ms }
}
fn clear_attack(actor: &mut ActorRuntime, state: &mut OrdinaryState) {
    actor.current_attack = None;
    actor.windup_origin_m = None;
    state.active = None;
    state.committed_direction = None;
    state.terrain_variant = None;
}
fn idle(actor: &mut ActorRuntime, state: &mut OrdinaryState) {
    clear_attack(actor, state);
    actor.state = ActorAiState::Idle;
    actor.state_remaining_ms = 0;
    state.last_seen_m = None;
}
fn recover(actor: &mut ActorRuntime, state: &mut OrdinaryState, config: &OrdinaryCombatProfile, profile: &ActorProfile, kind: ActorAttackKind) {
    clear_attack(actor, state);
    actor.state = ActorAiState::Cooldown;
    actor.state_remaining_ms = config.cooldown_ms(kind, profile);
    actor.attack_serial = actor.attack_serial.saturating_add(1).min(u64::MAX - 1);
}

pub(super) fn take_damage(actor: &mut ActorRuntime, profile: &ActorProfile, config: &OrdinaryCombatProfile, damage: u32, stagger: u32) -> Vec<ActorRuntimeEvent> {
    if actor.hp == 0 || (damage == 0 && stagger == 0) || !validate(actor, profile, config) { return vec![]; }
    let mut state = actor.ordinary.take().unwrap();
    actor.hp = actor.hp.saturating_sub(damage);
    let mut events = vec![];
    if actor.hp == 0 {
        idle(actor, &mut state);
        actor.state = ActorAiState::Dead;
        events.push(cue(actor, ActorCueKind::Death, 0));
    } else if stagger >= config.stagger_threshold {
        clear_attack(actor, &mut state);
        actor.state = ActorAiState::Stagger;
        actor.state_remaining_ms = config.stagger_ms;
        events.push(cue(actor, ActorCueKind::Stagger, config.stagger_ms));
    }
    actor.ordinary = Some(state);
    events
}

fn move_toward(actor: &mut ActorRuntime, destination: Vec3, speed: f32, config: &OrdinaryCombatProfile, profile: &ActorProfile, kcc: &StaticKccWorld, dt_ms: u64) {
    let distance = horizontal_distance(actor.position_m, destination);
    if distance <= 0.0001 { return; }
    let d = [(destination.x_m - actor.position_m.x_m) / distance, (destination.z_m - actor.position_m.z_m) / distance];
    let candidate = offset(actor.position_m, d, distance.min(speed * dt_ms as f32 / 1000.0));
    if valid_point(candidate, actor, profile) && actor_sweep_clear(actor.position_m, candidate, config.body_radius_m, kcc) {
        actor.position_m = candidate;
    }
}
fn actor_sweep_clear(from: Vec3, to: Vec3, radius: f32, kcc: &StaticKccWorld) -> bool {
    let steps=((horizontal_distance(from,to)/(radius*0.5).min(0.05)).ceil() as usize).max(1);
    (0..=steps).all(|i| {let p=lerp(from,to,i as f32/steps as f32);
        kcc.can_occupy(p,radius) && kcc.stable_actor_footprint(p,radius)})
}
fn sweep_clear(from: Vec3, to: Vec3, radius: f32, kcc: &StaticKccWorld) -> bool {
    let steps = ((horizontal_distance(from, to) / (radius * 0.5).min(0.05)).ceil() as usize).max(1);
    (0..=steps).all(|i| kcc.can_occupy(lerp(from, to, i as f32 / steps as f32), radius))
}
fn lerp(from: Vec3, to: Vec3, t: f32) -> Vec3 {
    Vec3 { x_m: from.x_m + (to.x_m - from.x_m) * t, y_m: from.y_m, z_m: from.z_m + (to.z_m - from.z_m) * t }
}
fn impact(out: &mut ActorTickOutput, actor: &ActorRuntime, config: &OrdinaryCombatProfile, profile: &ActorProfile, kind: ActorAttackKind, origin: Vec3, direction: [f32; 2], hit: bool) {
    let damage = group_members::scale_attack_damage(actor, config.damage(kind, profile));
    let range_m = if matches!(kind, ActorAttackKind::PressureShot | ActorAttackKind::SignalShot) { config.projectile.as_ref().unwrap().travel_m }
        else { config.motion(kind).map_or(profile.attack_range_m, |motion| motion.speed_mps * motion.active_ms as f32 / 1000.0) };
    let geometry = Some(ActorAttackGeometry { direction_rad: direction[0].atan2(direction[1]), range_m,
        half_angle_rad: (kind == ActorAttackKind::Swing).then_some(config.directional_half_angle_rad).flatten() });
    out.events.push(ActorRuntimeEvent::AttackImpact { actor_id: actor.entity_id.clone(), kind, origin_m: origin, radius_m: config.radius(kind, profile), damage, hit_player: hit, geometry });
    if hit {
        out.damage_to_player = Some(damage);
        out.ranged_damage = matches!(kind, ActorAttackKind::PressureShot | ActorAttackKind::SignalShot);
        out.damage_tag = (kind == ActorAttackKind::PressureShot).then_some(HazardTag::Pressure);
    }
}

pub(super) fn signal_interferes_at(actor: &ActorRuntime, profile: &ActorProfile, observer: Vec3, kcc: &StaticKccWorld) -> bool {
    let Some(config) = profile.ordinary.as_ref() else { return false; };
    let Some(signal) = &config.signal else { return false; };
    actor.hp > 0 && validate(actor, profile, config) && finite(observer)
        && !matches!(actor.state, ActorAiState::Windup | ActorAiState::Active | ActorAiState::Stagger)
        && kcc.stable_actor_footprint(actor.position_m, config.body_radius_m)
        && crate::continuous_combat::combat_vertical_overlap(actor.position_m, observer)
        && horizontal_distance(actor.position_m, observer) <= signal.interference_range_m
        && sweep_clear(actor.position_m, observer, 0.05, kcc)
}

/// A completed recovery can relocate away from a nearby observer, but only along
/// supported, unobstructed space on the same home plane. No invulnerability,
/// damage, extra actor, or navigation permission is created.
fn relocate_after_recovery(actor: &mut ActorRuntime, player: Vec3, sensed: bool, profile: &ActorProfile, config: &OrdinaryCombatProfile, kcc: &StaticKccWorld) -> Option<ActorRuntimeEvent> {
    let signal = config.signal.as_ref()?;
    let distance = horizontal_distance(actor.position_m, player);
    if !sensed || distance > signal.relocate_trigger_m { return None; }
    let away = if distance > 0.0001 { [(actor.position_m.x_m-player.x_m)/distance, (actor.position_m.z_m-player.z_m)/distance] } else { [1.0,0.0] };
    for direction in [away, [-away[1],away[0]], [away[1],-away[0]]] {
        let to = offset(actor.position_m, direction, signal.relocate_distance_m);
        if valid_point(to,actor,profile) && actor_sweep_clear(actor.position_m,to,config.body_radius_m,kcc) {
            let from = actor.position_m; actor.position_m = to;
            return Some(ActorRuntimeEvent::Relocated { actor_id: actor.entity_id.clone(), from_m: from, to_m: to });
        }
    }
    None
}

pub(super) fn tick(actor: &mut ActorRuntime, profile: &ActorProfile, player: Vec3, kcc: &StaticKccWorld, dt_s: f32, selected_terrain: Option<OrdinaryTerrainVariant>) -> ActorTickOutput {
    let config = profile.ordinary.as_ref().unwrap();
    let mut out = ActorTickOutput::default();
    if !validate(actor, profile, config) || !finite(player) || !dt_s.is_finite() || dt_s <= 0.0 || dt_s > 0.25 || actor.hp == 0 { return out; }
    let dt_ms = (dt_s * 1000.0).round() as u64;
    if dt_ms == 0 { return out; }
    let mut state = actor.ordinary.take().unwrap();
    let distance = horizontal_distance(actor.position_m, player);
    let sensed = crate::continuous_combat::combat_vertical_overlap(actor.position_m, player)
        && distance <= profile.perception_m && horizontal_distance(actor.home_m, player) <= profile.leash_m
        && sweep_clear(actor.position_m, player, 0.05, kcc);
    if sensed { state.last_seen_m = Some(Vec3 { y_m: actor.home_m.y_m, ..player }); }
    match actor.state {
        ActorAiState::Idle => {
            if sensed {
                actor.state = ActorAiState::Chasing;
                out.events.push(cue(actor, ActorCueKind::Alert, 0));
            }
        }
        ActorAiState::Chasing => {
            if !sensed {
                actor.state = ActorAiState::Investigate;
                actor.state_remaining_ms = config.investigate_ms;
            } else {
                let kind = if config.projectile.is_some() && distance <= profile.attack_range_m {
                    Some(config.projectile.as_ref().unwrap().kind.attack())
                } else if let Some(leap) = &config.leap {
                    if distance <= profile.attack_range_m { Some(ActorAttackKind::Bite) }
                    else if distance >= leap.min_range_m && distance <= leap.range_m { Some(ActorAttackKind::Leap) }
                    else { None }
                } else if let Some(charge) = &config.charge {
                    if distance <= profile.attack_range_m { Some(if config.directional_half_angle_rad.is_some() { ActorAttackKind::Swing } else { ActorAttackKind::Slam }) }
                    else if distance >= charge.min_range_m && distance <= charge.range_m { Some(ActorAttackKind::Charge) }
                    else { None }
                } else if let Some(lunge) = &config.lunge {
                    (distance >= lunge.min_range_m && distance <= lunge.range_m).then_some(ActorAttackKind::Lunge)
                } else { None };
                if let Some(kind) = kind {
                    actor.state = ActorAiState::Windup;
                    state.terrain_variant = selected_terrain;
                    actor.current_attack = Some(kind);
                    actor.windup_origin_m = Some(actor.position_m);
                    actor.state_remaining_ms = config.windup_ms(kind, profile);
                    state.committed_direction = Some(if distance > 0.0001 {
                        [(player.x_m - actor.position_m.x_m) / distance, (player.z_m - actor.position_m.z_m) / distance]
                    } else { [1.0, 0.0] });
                    out.events.push(ActorRuntimeEvent::AttackWindup { actor_id: actor.entity_id.clone(), kind, origin_m: actor.position_m, radius_m: config.motion(kind).map_or(profile.attack_range_m, |motion| motion.range_m), windup_ms: actor.state_remaining_ms });
                } else {
                    move_toward(actor, player, profile.speed_mps, config, profile, kcc, dt_ms);
                }
            }
        }
        ActorAiState::Investigate => {
            if sensed { actor.state = ActorAiState::Chasing; actor.state_remaining_ms = 0; }
            else {
                actor.state_remaining_ms = actor.state_remaining_ms.saturating_sub(dt_ms);
                if actor.state_remaining_ms == 0 { idle(actor, &mut state); }
                else if let Some(last) = state.last_seen_m { move_toward(actor, last, profile.speed_mps, config, profile, kcc, dt_ms); }
            }
        }
        ActorAiState::Windup => {
            actor.state_remaining_ms = actor.state_remaining_ms.saturating_sub(dt_ms);
            if actor.state_remaining_ms == 0 {
                let kind = actor.current_attack.unwrap();
                if matches!(kind, ActorAttackKind::Bite | ActorAttackKind::Slam | ActorAttackKind::Swing) {
                    let in_sector = kind != ActorAttackKind::Swing || distance <= 0.0001 || {
                        let direction = state.committed_direction.unwrap();
                        ((player.x_m - actor.position_m.x_m) * direction[0] + (player.z_m - actor.position_m.z_m) * direction[1]) / distance
                            >= config.directional_half_angle_rad.unwrap().cos()
                    };
                    let hit = crate::continuous_combat::combat_vertical_overlap(actor.position_m, player)
                        && distance <= profile.attack_range_m && in_sector && sweep_clear(actor.position_m, player, 0.05, kcc);
                    impact(&mut out, actor, config, profile, kind, actor.position_m, state.committed_direction.unwrap(), hit);
                    recover(actor, &mut state, config, profile, kind);
                } else {
                    actor.state = ActorAiState::Active;
                    actor.state_remaining_ms = config.active_ms(kind).unwrap();
                    state.active = Some(OrdinaryActiveAttack { kind, origin_m: actor.windup_origin_m.unwrap(), direction: state.committed_direction.unwrap(), elapsed_ms: 0, hit_resolved: false });
                }
            }
        }
        ActorAiState::Active => {
            let active = state.active.as_mut().unwrap();
            let total = config.active_ms(active.kind).unwrap();
            let elapsed = active.elapsed_ms.saturating_add(dt_ms).min(total);
            let from = active_point(active, config, active.elapsed_ms);
            let intended = active_point(active, config, elapsed);
            let radius = config.radius(active.kind, profile);
            let collision_radius = if matches!(active.kind, ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge) { config.body_radius_m } else { radius };
            let steps = ((horizontal_distance(from, intended) / (collision_radius * 0.5).min(0.05)).ceil() as usize).max(1);
            let mut endpoint = from;
            let mut blocked = false;
            let mut contact = false;
            for i in 0..=steps {
                let p = lerp(from, intended, i as f32 / steps as f32);
                if !kcc.can_occupy(p, collision_radius) || (matches!(active.kind, ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge)
                    && (!valid_point(p, actor, profile) || !kcc.stable_actor_footprint(p,collision_radius))) { blocked = true; break; }
                endpoint = p;
                if !active.hit_resolved && crate::continuous_combat::combat_vertical_overlap(p, player)
                    && horizontal_distance(p, player) <= radius + KccBody::new(player).radius_m
                    && sweep_clear(p, player, 0.05, kcc) {
                    contact = true;
                    active.hit_resolved = true;
                    impact(&mut out, actor, config, profile, active.kind, p, active.direction, true);
                    if matches!(active.kind, ActorAttackKind::PressureShot | ActorAttackKind::SignalShot) { break; }
                }
            }
            active.elapsed_ms = elapsed;
            actor.state_remaining_ms = total - elapsed;
            if matches!(active.kind, ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Lunge) { actor.position_m = endpoint; }
            out.events.push(ActorRuntimeEvent::AttackMotion { actor_id: actor.entity_id.clone(), kind: active.kind, from_m: from, to_m: endpoint, radius_m: radius, remaining_ms: actor.state_remaining_ms });
            if blocked || elapsed == total || (contact && matches!(active.kind, ActorAttackKind::PressureShot | ActorAttackKind::SignalShot)) {
                let kind = active.kind;
                if !active.hit_resolved { impact(&mut out, actor, config, profile, kind, endpoint, active.direction, false); }
                recover(actor, &mut state, config, profile, kind);
            }
        }
        ActorAiState::Cooldown | ActorAiState::Stagger => {
            let recovered_attack = actor.state == ActorAiState::Cooldown;
            actor.state_remaining_ms = actor.state_remaining_ms.saturating_sub(dt_ms);
            if actor.state_remaining_ms == 0 {
                if recovered_attack {
                    if let Some(event) = relocate_after_recovery(actor,player,sensed,profile,config,kcc) { out.events.push(event); }
                }
                idle(actor, &mut state);
            }
        }
        ActorAiState::Dead => {}
    }
    actor.ordinary = Some(state);
    debug_assert!(validate(actor, profile, config));
    out
}
