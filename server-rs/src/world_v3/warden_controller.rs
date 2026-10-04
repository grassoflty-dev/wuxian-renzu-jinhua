//! Original Resonance Warden controller on the shared Actor runtime.
//! All numbers come from the authored profile; this module never grants a capability or route event.
use super::super::Vec3;
use super::{ActorAiState, ActorProfile, ActorRuntime, ActorRuntimeEvent, ActorTickOutput};
use crate::continuous_kcc::StaticKccWorld;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WardenConfig {
    pub phase_two_hp_bps: u32,
    pub strike_radius_m: f32,
    pub strike_half_angle_rad: f32,
    pub strike_windup_ms: u64,
    pub strike_recovery_ms: u64,
    pub strike_damage: u32,
    pub pulse_radius_m: f32,
    pub pulse_windup_ms: u64,
    pub pulse_recovery_ms: u64,
    pub pulse_damage: u32,
    pub decoy_ms: u64,
    pub ordinary_warning_ms: u64,
    pub stagger_damage: u32,
    pub stagger_ms: u64,
    pub stagger_cooldown_ms: u64,
}
impl WardenConfig {
    pub fn validate(&self) -> bool {
        (1..10_000).contains(&self.phase_two_hp_bps)
            && [self.strike_radius_m, self.pulse_radius_m]
                .iter()
                .all(|n| n.is_finite() && *n > 0.0 && *n <= 12.0)
            && self.strike_half_angle_rad.is_finite()
            && self.strike_half_angle_rad > 0.0
            && self.strike_half_angle_rad <= std::f32::consts::PI
            && (100..=self.strike_windup_ms.min(self.pulse_windup_ms))
                .contains(&self.ordinary_warning_ms)
            && [
                self.strike_windup_ms,
                self.strike_recovery_ms,
                self.pulse_windup_ms,
                self.pulse_recovery_ms,
                self.decoy_ms,
                self.stagger_ms,
                self.stagger_cooldown_ms,
            ]
            .iter()
            .all(|n| (100..=60_000).contains(n))
            && [self.strike_damage, self.pulse_damage, self.stagger_damage]
                .iter()
                .all(|n| (1..=10_000).contains(n))
    }
    pub fn phase(&self, hp: u32, max_hp: u32) -> u8 {
        if u64::from(hp) * 10_000 <= u64::from(max_hp) * u64::from(self.phase_two_hp_bps) {
            2
        } else {
            1
        }
    }
    pub fn windup(&self, attack: WardenAttack) -> u64 {
        match attack {
            WardenAttack::Strike => self.strike_windup_ms,
            WardenAttack::Pulse => self.pulse_windup_ms,
        }
    }
    pub fn recovery(&self, attack: WardenAttack) -> u64 {
        match attack {
            WardenAttack::Strike => self.strike_recovery_ms,
            WardenAttack::Pulse => self.pulse_recovery_ms,
        }
    }
    pub fn radius(&self, attack: WardenAttack) -> f32 {
        match attack {
            WardenAttack::Strike => self.strike_radius_m,
            WardenAttack::Pulse => self.pulse_radius_m,
        }
    }
    pub fn damage(&self, attack: WardenAttack) -> u32 {
        match attack {
            WardenAttack::Strike => self.strike_damage,
            WardenAttack::Pulse => self.pulse_damage,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WardenStage {
    Idle,
    Chase,
    Decoy,
    Windup,
    Recovery,
    Stagger,
    Dead,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WardenAttack {
    Strike,
    Pulse,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WardenCueKind {
    Decoy,
    StrikeWindup,
    PulseWindup,
    StrikeImpact,
    PulseImpact,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WardenState {
    pub stage: WardenStage,
    pub remaining_ms: u64,
    pub attack: Option<WardenAttack>,
    pub committed_origin_m: Option<Vec3>,
    pub committed_direction_rad: Option<f32>,
    pub resolved_attack_serial: u64,
    pub stagger_cooldown_ms: u64,
}
impl Default for WardenState {
    fn default() -> Self {
        Self {
            stage: WardenStage::Idle,
            remaining_ms: 0,
            attack: None,
            committed_origin_m: None,
            committed_direction_rad: None,
            resolved_attack_serial: 0,
            stagger_cooldown_ms: 0,
        }
    }
}
impl WardenState {
    pub fn actor_state(&self) -> ActorAiState {
        match self.stage {
            WardenStage::Idle => ActorAiState::Idle,
            WardenStage::Chase => ActorAiState::Chasing,
            WardenStage::Decoy | WardenStage::Windup => ActorAiState::Windup,
            WardenStage::Recovery | WardenStage::Stagger => ActorAiState::Cooldown,
            WardenStage::Dead => ActorAiState::Dead,
        }
    }
    pub fn validate(&self, config: &WardenConfig, hp: u32, max_hp: u32, serial: u64) -> bool {
        if !config.validate()
            || serial == u64::MAX
            || self.stagger_cooldown_ms > config.stagger_cooldown_ms
            || (hp == 0) != (self.stage == WardenStage::Dead)
        {
            return false;
        }
        let pending = matches!(self.stage, WardenStage::Decoy | WardenStage::Windup);
        if pending {
            let (Some(attack), Some(origin), Some(direction)) = (
                self.attack,
                self.committed_origin_m,
                self.committed_direction_rad,
            ) else {
                return false;
            };
            let maximum = if self.stage == WardenStage::Decoy {
                config.decoy_ms
            } else {
                config.windup(attack)
            };
            (self.stage != WardenStage::Decoy || config.phase(hp, max_hp) == 2)
                && attack
                    == if serial % 2 == 1 {
                        WardenAttack::Strike
                    } else {
                        WardenAttack::Pulse
                    }
                && self.remaining_ms > 0
                && self.remaining_ms <= maximum
                && super::finite(origin)
                && direction.is_finite()
                && direction.abs() <= std::f32::consts::PI
                && self.resolved_attack_serial.checked_add(1) == Some(serial)
        } else {
            self.attack.is_none()
                && self.committed_origin_m.is_none()
                && self.committed_direction_rad.is_none()
                && self.resolved_attack_serial == serial
                && match self.stage {
                    WardenStage::Recovery => {
                        self.remaining_ms > 0
                            && self.remaining_ms
                                <= config.recovery(if serial % 2 == 1 {
                                    WardenAttack::Strike
                                } else {
                                    WardenAttack::Pulse
                                })
                    }
                    WardenStage::Stagger => {
                        self.remaining_ms > 0 && self.remaining_ms <= config.stagger_ms
                    }
                    _ => self.remaining_ms == 0,
                }
        }
    }
    fn clear_attack(&mut self, serial: u64) {
        self.attack = None;
        self.committed_origin_m = None;
        self.committed_direction_rad = None;
        self.resolved_attack_serial = serial;
    }
}
fn emit(
    output: &mut ActorTickOutput,
    actor: &ActorRuntime,
    state: &WardenState,
    config: &WardenConfig,
    kind: WardenCueKind,
    origin: Vec3,
) {
    let attack = state.attack.unwrap_or(WardenAttack::Strike);
    output.events.push(ActorRuntimeEvent::WardenCue {
        actor_id: actor.entity_id.clone(),
        kind,
        origin_m: origin,
        direction_rad: state.committed_direction_rad.unwrap_or(0.0),
        radius_m: config.radius(attack),
        duration_ms: state.remaining_ms,
        attack_serial: actor.attack_serial,
    });
}
pub(super) fn take_damage(actor: &mut ActorRuntime, profile: &ActorProfile, damage: u32) {
    let (Some(config), Some(state)) = (profile.warden.as_ref(), actor.warden.as_mut()) else {
        return;
    };
    if actor.hp == 0 {
        state.stage = WardenStage::Dead;
        state.remaining_ms = 0;
        state.stagger_cooldown_ms = 0;
        state.clear_attack(actor.attack_serial);
    } else if damage >= config.stagger_damage
        && state.stagger_cooldown_ms == 0
        && matches!(state.stage, WardenStage::Decoy | WardenStage::Windup)
    {
        state.stage = WardenStage::Stagger;
        state.remaining_ms = config.stagger_ms;
        state.stagger_cooldown_ms = config.stagger_cooldown_ms;
        state.clear_attack(actor.attack_serial);
    }
    actor.state = state.actor_state();
    actor.state_remaining_ms = state.remaining_ms;
}

pub(super) fn tick(
    actor: &mut ActorRuntime,
    profile: &ActorProfile,
    player: Vec3,
    kcc: &StaticKccWorld,
    dt_s: f32,
) -> ActorTickOutput {
    let Some(config) = profile.warden.as_ref() else {
        return ActorTickOutput::default();
    };
    let Some(mut state) = actor.warden.take() else {
        return ActorTickOutput::default();
    };
    let mut out = ActorTickOutput::default();
    if !state.validate(config, actor.hp, profile.max_hp, actor.attack_serial)
        || !super::finite(player)
        || !dt_s.is_finite()
        || dt_s <= 0.0
        || dt_s > 0.25
    {
        actor.warden = Some(state);
        return out;
    }
    let dt_ms = (dt_s * 1000.0).round() as u64;
    if dt_ms == 0 {
        actor.warden = Some(state);
        return out;
    }
    state.stagger_cooldown_ms = state.stagger_cooldown_ms.saturating_sub(dt_ms);
    let distance = super::horizontal_distance(actor.position_m, player);
    let sensed = crate::continuous_combat::combat_vertical_overlap(actor.position_m, player)
        && distance <= profile.perception_m
        && super::horizontal_distance(actor.home_m, player) <= profile.leash_m
        && super::line_clear(actor.position_m, player, kcc);
    match state.stage {
        WardenStage::Idle => {
            if sensed {
                state.stage = WardenStage::Chase;
            }
        }
        WardenStage::Chase => {
            let attack = if actor.attack_serial % 2 == 0 {
                WardenAttack::Strike
            } else {
                WardenAttack::Pulse
            };
            if !sensed {
                state.stage = WardenStage::Idle;
            } else if distance <= config.radius(attack) {
                let Some(serial) = actor.attack_serial.checked_add(1).filter(|n| *n < u64::MAX)
                else {
                    actor.warden = Some(state);
                    return out;
                };
                actor.attack_serial = serial;
                state.attack = Some(attack);
                state.committed_origin_m = Some(actor.position_m);
                state.committed_direction_rad = Some(
                    (player.x_m - actor.position_m.x_m).atan2(player.z_m - actor.position_m.z_m),
                );
                if config.phase(actor.hp, profile.max_hp) == 2 {
                    state.stage = WardenStage::Decoy;
                    state.remaining_ms = config.decoy_ms;
                    let a = state.committed_direction_rad.unwrap() + std::f32::consts::FRAC_PI_2;
                    let fake = Vec3 {
                        x_m: actor.position_m.x_m + a.sin() * 2.0,
                        y_m: actor.position_m.y_m,
                        z_m: actor.position_m.z_m + a.cos() * 2.0,
                    };
                    emit(
                        &mut out,
                        actor,
                        &state,
                        config,
                        WardenCueKind::Decoy,
                        if kcc.can_occupy(fake, 0.05) {
                            fake
                        } else {
                            actor.position_m
                        },
                    );
                } else {
                    state.stage = WardenStage::Windup;
                    state.remaining_ms = config.windup(attack);
                    emit(
                        &mut out,
                        actor,
                        &state,
                        config,
                        if attack == WardenAttack::Strike {
                            WardenCueKind::StrikeWindup
                        } else {
                            WardenCueKind::PulseWindup
                        },
                        actor.position_m,
                    );
                }
            } else if distance > 0.0 {
                let step = (profile.speed_mps * dt_s).min(distance);
                let next = Vec3 {
                    x_m: actor.position_m.x_m
                        + (player.x_m - actor.position_m.x_m) / distance * step,
                    y_m: actor.position_m.y_m,
                    z_m: actor.position_m.z_m
                        + (player.z_m - actor.position_m.z_m) / distance * step,
                };
                if kcc.can_occupy(next, 0.3)
                    && kcc.stable_actor_footprint(next,0.3)
                    && super::horizontal_distance(next, actor.home_m) <= profile.leash_m
                {
                    actor.position_m = next;
                }
            }
        }
        WardenStage::Decoy => {
            state.remaining_ms = state.remaining_ms.saturating_sub(dt_ms);
            if state.remaining_ms == 0 {
                let attack = state.attack.unwrap();
                state.stage = WardenStage::Windup;
                state.remaining_ms = config.windup(attack);
                emit(
                    &mut out,
                    actor,
                    &state,
                    config,
                    if attack == WardenAttack::Strike {
                        WardenCueKind::StrikeWindup
                    } else {
                        WardenCueKind::PulseWindup
                    },
                    state.committed_origin_m.unwrap(),
                );
            }
        }
        WardenStage::Windup => {
            state.remaining_ms = state.remaining_ms.saturating_sub(dt_ms);
            if state.remaining_ms == 0 {
                let attack = state.attack.unwrap();
                let origin = state.committed_origin_m.unwrap();
                let radius = config.radius(attack);
                let dx = player.x_m - origin.x_m;
                let dz = player.z_m - origin.z_m;
                let distance = (dx * dx + dz * dz).sqrt();
                let direction = state.committed_direction_rad.unwrap();
                let in_cone = distance <= f32::EPSILON
                    || (dx * direction.sin() + dz * direction.cos()) / distance
                        >= config.strike_half_angle_rad.cos();
                let hit = crate::continuous_combat::combat_vertical_overlap(origin, player)
                    && distance <= radius
                    && (attack == WardenAttack::Pulse || in_cone)
                    && super::line_clear(origin, player, kcc);
                emit(
                    &mut out,
                    actor,
                    &state,
                    config,
                    if attack == WardenAttack::Strike {
                        WardenCueKind::StrikeImpact
                    } else {
                        WardenCueKind::PulseImpact
                    },
                    origin,
                );
                if hit {
                    out.damage_to_player = Some(config.damage(attack));
                }
                state.clear_attack(actor.attack_serial);
                state.stage = WardenStage::Recovery;
                state.remaining_ms = config.recovery(attack);
            }
        }
        WardenStage::Recovery | WardenStage::Stagger => {
            state.remaining_ms = state.remaining_ms.saturating_sub(dt_ms);
            if state.remaining_ms == 0 {
                state.stage = WardenStage::Idle;
            }
        }
        WardenStage::Dead => {}
    }
    actor.state = state.actor_state();
    actor.state_remaining_ms = state.remaining_ms;
    actor.warden = Some(state);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::continuous_kcc::Aabb;
    fn fixture() -> (ActorRuntime, StaticKccWorld, Vec3) {
        (
            ActorRuntime::spawn(
                "mh_resonance_warden_staged_marker",
                "enemy.mist_harbor.resonance_warden",
                Vec3::new(10.0, 0.0, 8.0).unwrap(),
            )
            .unwrap(),
            StaticKccWorld::new(Aabb::new(0.5, 23.5, 0.5, 15.5).unwrap(), vec![]),
            Vec3::new(12.0, 0.0, 8.0).unwrap(),
        )
    }
    #[test]
    fn profile_state_is_typed_and_only_authored_special_context_ticks() {
        let (mut actor, kcc, player) = fixture();
        let before = actor.clone();
        actor.tick(player, &kcc, 1.0 / 60.0, false);
        assert_eq!(actor, before);
        assert!(actor.validate());
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        assert_eq!(actor.warden.as_ref().unwrap().stage, WardenStage::Chase);
        assert!(actor.validate());
        let mut forged = actor.clone();
        forged.entity_type = "enemy.mist_harbor.signal_wraith".into();
        assert!(!forged.validate());
        let mut forged = actor.clone();
        forged.warden = None;
        assert!(!forged.validate());
    }
    #[test]
    fn deterministic_strike_and_pulse_are_distinct_and_round_trip_once() {
        let (mut actor, kcc, player) = fixture();
        let mut twin = actor.clone();
        let mut hits = Vec::new();
        for _ in 0..450 {
            let a = actor.tick(player, &kcc, 1.0 / 60.0, true);
            let b = twin.tick(player, &kcc, 1.0 / 60.0, true);
            assert_eq!(a, b);
            assert_eq!(actor, twin);
            assert!(actor.validate());
            if let Some(damage) = a.damage_to_player {
                hits.push(damage);
            }
            twin = serde_json::from_str(&serde_json::to_string(&twin).unwrap()).unwrap();
        }
        assert!(hits.starts_with(&[14, 18]));
        let serial = actor.attack_serial;
        let resolved = actor.warden.as_ref().unwrap().resolved_attack_serial;
        assert!(resolved == serial || resolved + 1 == serial);
    }
    #[test]
    fn committed_direction_allows_normal_movement_dodge_without_any_capability() {
        let (mut actor, kcc, player) = fixture();
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        let windup = actor.tick(player, &kcc, 1.0 / 60.0, true);
        assert!(matches!(
            windup.events[0],
            ActorRuntimeEvent::WardenCue {
                kind: WardenCueKind::StrikeWindup,
                ..
            }
        ));
        let direction = actor.warden.as_ref().unwrap().committed_direction_rad;
        let sideways = Vec3::new(10.0, 0.0, 10.0).unwrap();
        let mut impacts = 0;
        for _ in 0..80 {
            let out = actor.tick(sideways, &kcc, 1.0 / 60.0, true);
            assert!(out.damage_to_player.is_none());
            impacts += out
                .events
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        ActorRuntimeEvent::WardenCue {
                            kind: WardenCueKind::StrikeImpact,
                            ..
                        }
                    )
                })
                .count();
        }
        assert_eq!(impacts, 1);
        assert_eq!(direction, Some(std::f32::consts::FRAC_PI_2));
    }
    #[test]
    fn lower_hp_decoy_never_hits_and_true_warning_follows_with_full_window() {
        let (mut actor, kcc, player) = fixture();
        actor.take_damage(180);
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        let out = actor.tick(player, &kcc, 1.0 / 60.0, true);
        assert!(matches!(
            out.events[0],
            ActorRuntimeEvent::WardenCue {
                kind: WardenCueKind::Decoy,
                ..
            }
        ));
        assert!(out.damage_to_player.is_none());
        while actor.warden.as_ref().unwrap().stage == WardenStage::Decoy {
            assert!(actor
                .tick(player, &kcc, 1.0 / 60.0, true)
                .damage_to_player
                .is_none());
        }
        assert_eq!(actor.warden.as_ref().unwrap().remaining_ms, 1100);
        assert_eq!(actor.warden.as_ref().unwrap().stage, WardenStage::Windup);
    }
    #[test]
    fn strong_hit_cancels_one_pending_attack_and_death_never_impacts() {
        let (mut actor, kcc, player) = fixture();
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        let serial = actor.attack_serial;
        actor.take_damage(22);
        assert_eq!(actor.warden.as_ref().unwrap().stage, WardenStage::Stagger);
        assert_eq!(
            actor.warden.as_ref().unwrap().resolved_attack_serial,
            serial
        );
        assert!(actor.validate());
        for _ in 0..20 {
            assert!(actor
                .tick(player, &kcc, 1.0 / 60.0, true)
                .damage_to_player
                .is_none());
        }
        actor.take_damage(1000);
        assert!(actor.validate());
        let before = actor.clone();
        assert_eq!(
            actor.tick(player, &kcc, 1.0 / 60.0, true),
            ActorTickOutput::default()
        );
        assert_eq!(actor, before);
    }
    #[test]
    fn malformed_saved_commit_and_timers_fail_closed() {
        let (mut actor, kcc, player) = fixture();
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        actor.tick(player, &kcc, 1.0 / 60.0, true);
        let mut bad = actor.clone();
        bad.warden.as_mut().unwrap().committed_direction_rad = Some(f32::NAN);
        assert!(!bad.validate());
        let mut bad = actor.clone();
        bad.warden.as_mut().unwrap().resolved_attack_serial = bad.attack_serial;
        assert!(!bad.validate());
        let mut bad = actor.clone();
        bad.warden.as_mut().unwrap().remaining_ms = 1101;
        bad.state_remaining_ms = 1101;
        assert!(!bad.validate());
        let mut raw = serde_json::to_value(&actor).unwrap();
        raw["warden"]["pretendKilled"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ActorRuntime>(raw).is_err());
    }
}
