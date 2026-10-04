use super::{ActorView, Transform, Vec3};
#[path = "warden_controller.rs"]
mod warden_controller;
#[path = "ordinary_controller.rs"]
mod ordinary_controller;
#[path = "group_members.rs"]
mod group_members;
pub use group_members::{GroupProfile, GroupState};
pub use ordinary_controller::{OrdinaryActiveAttack, OrdinaryCombatProfile, OrdinaryState, OrdinaryTerrainVariant};
use crate::continuous_kcc::StaticKccWorld;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};
pub use warden_controller::{WardenAttack, WardenConfig, WardenCueKind, WardenStage, WardenState};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorProfile {
    pub entity_type: String,
    pub render_type: String,
    pub max_hp: u32,
    pub perception_m: f32,
    pub leash_m: f32,
    pub speed_mps: f32,
    pub attack_range_m: f32,
    pub attack_damage: u32,
    pub windup_ms: u64,
    pub cooldown_ms: u64,
    #[serde(default)]
    pub pressure_wave: Option<PressureWaveProfile>,
    #[serde(default)]
    pub warden: Option<WardenConfig>,
    #[serde(default)]
    pub ordinary: Option<OrdinaryCombatProfile>,
    #[serde(default)]
    pub group: Option<GroupProfile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PressureWaveProfile {
    pub radius_m: f32,
    pub damage: u32,
    pub windup_ms: u64,
    pub cooldown_ms: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProfileCatalog {
    schema_version: u32,
    profiles: Vec<ActorProfile>,
    #[serde(default)]
    controller_variants: Vec<ControllerVariantProfile>,
}

/// Opt-in controller data. Native factories remain on variant zero until the
/// paired roster migration and public warning presentation are admitted.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ControllerVariantProfile {
    entity_type: String,
    variant: u8,
    perception_m: f32,
    speed_mps: f32,
    attack_range_m: f32,
    attack_damage: u32,
    windup_ms: u64,
    cooldown_ms: u64,
    ordinary: OrdinaryCombatProfile,
}
struct Profiles {
    base: BTreeMap<String, ActorProfile>,
    variants: BTreeMap<String, BTreeMap<u8, ActorProfile>>,
}

fn profiles() -> &'static Profiles {
    static PROFILES: OnceLock<Profiles> = OnceLock::new();
    PROFILES.get_or_init(|| {
        let catalog: ProfileCatalog =
            serde_json::from_str(include_str!("../../data/actor_profiles_v1.json"))
                .expect("actor profile catalog parses");
        assert_eq!(catalog.schema_version, 1, "actor profile schema");
        let mut profiles = BTreeMap::new();
        for profile in catalog.profiles {
            assert!(
                profile.warden.as_ref().is_none_or(WardenConfig::validate),
                "warden profile schema"
            );
            assert!(
                !profile.entity_type.is_empty()
                    && !profile.render_type.is_empty()
                    && profile.max_hp > 0
                    && profile.attack_damage > 0
                    && profile.windup_ms > 0
                    && profile.cooldown_ms > 0
                    && [
                        profile.perception_m,
                        profile.leash_m,
                        profile.speed_mps,
                        profile.attack_range_m
                    ]
                    .iter()
                    .all(|value| value.is_finite() && *value > 0.0)
                    && profile.leash_m >= profile.perception_m
                    && profile.perception_m >= profile.attack_range_m,
                "invalid actor profile"
            );
            assert!(profile.ordinary.as_ref().is_none_or(|config| config.validate(&profile))
                && !(profile.ordinary.is_some() && profile.warden.is_some()), "invalid ordinary combat profile");
            assert!(profile.group.as_ref().is_none_or(|group| group.validate(&profile)), "invalid group profile");
            if let Some(wave) = &profile.pressure_wave {
                assert!(
                    wave.radius_m.is_finite()
                        && wave.radius_m > 0.0
                        && wave.damage > 0
                        && wave.windup_ms > 0
                        && wave.cooldown_ms > 0,
                    "invalid actor pressure wave profile"
                );
            }
            assert!(
                profiles
                    .insert(profile.entity_type.clone(), profile)
                    .is_none(),
                "duplicate actor profile"
            );
        }
        let mut variants: BTreeMap<String, BTreeMap<u8, ActorProfile>> = BTreeMap::new();
        for variant in catalog.controller_variants {
            let mut profile = profiles.get(&variant.entity_type).expect("variant base profile").clone();
            assert!(variant.variant > 0 && profile.warden.is_none() && profile.group.is_none()
                && profile.pressure_wave.is_none(), "invalid controller variant identity");
            profile.perception_m = variant.perception_m;
            profile.speed_mps = variant.speed_mps;
            profile.attack_range_m = variant.attack_range_m;
            profile.attack_damage = variant.attack_damage;
            profile.windup_ms = variant.windup_ms;
            profile.cooldown_ms = variant.cooldown_ms;
            profile.ordinary = Some(variant.ordinary);
            assert!([profile.perception_m, profile.speed_mps, profile.attack_range_m].into_iter()
                .all(|v| v.is_finite() && v > 0.0)
                && profile.perception_m <= profile.leash_m && profile.attack_range_m <= profile.perception_m
                && profile.attack_damage > 0 && profile.windup_ms > 0 && profile.cooldown_ms > 0
                && profile.ordinary.as_ref().unwrap().validate(&profile), "invalid controller variant profile");
            assert!(variants.entry(profile.entity_type.clone()).or_default()
                .insert(variant.variant, profile).is_none(), "duplicate controller variant");
        }
        Profiles { base: profiles, variants }
    })
}

pub fn actor_profile(entity_type: &str) -> Option<&'static ActorProfile> {
    profiles().base.get(entity_type)
}
fn actor_profile_variant(entity_type: &str, variant: u8) -> Option<&'static ActorProfile> {
    if variant == 0 { actor_profile(entity_type) }
    else { profiles().variants.get(entity_type)?.get(&variant) }
}
fn is_zero(value: &u8) -> bool { *value == 0 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorAiState {
    Idle,
    Chasing,
    Investigate,
    Windup,
    Active,
    Stagger,
    Cooldown,
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorAttackKind {
    HeavyMelee,
    PressureWave,
    PressureShot,
    SignalShot,
    Bite,
    Leap,
    Charge,
    Slam,
    Lunge,
    Swing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorCueKind { Alert, Stagger, Death }

/// Committed public attack geometry retained even after the actor recovers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorAttackGeometry {
    pub direction_rad: f32,
    pub range_m: f32,
    pub half_angle_rad: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActorRuntimeEvent {
    Relocated { actor_id: String, from_m: Vec3, to_m: Vec3 },
    MemberHit {
        actor_id: String, member_id: String, position_m: Vec3, destroyed: bool,
    },
    EnemyCue {
        actor_id: String,
        kind: ActorCueKind,
        origin_m: Vec3,
        duration_ms: u64,
    },
    AttackMotion {
        actor_id: String,
        kind: ActorAttackKind,
        from_m: Vec3,
        to_m: Vec3,
        radius_m: f32,
        remaining_ms: u64,
    },
    WardenCue {
        actor_id: String,
        kind: WardenCueKind,
        origin_m: Vec3,
        direction_rad: f32,
        radius_m: f32,
        duration_ms: u64,
        attack_serial: u64,
    },
    AttackWindup {
        actor_id: String,
        kind: ActorAttackKind,
        origin_m: Vec3,
        radius_m: f32,
        windup_ms: u64,
    },
    AttackImpact {
        actor_id: String,
        kind: ActorAttackKind,
        origin_m: Vec3,
        radius_m: f32,
        damage: u32,
        hit_player: bool,
        geometry: Option<ActorAttackGeometry>,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActorTickOutput {
    pub events: Vec<ActorRuntimeEvent>,
    pub damage_to_player: Option<u32>,
    pub damage_tag: Option<crate::effects::HazardTag>,
    pub ranged_damage: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorRuntime {
    pub entity_id: String,
    pub entity_type: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub controller_variant: u8,
    pub position_m: Vec3,
    pub home_m: Vec3,
    pub hp: u32,
    pub state: ActorAiState,
    pub state_remaining_ms: u64,
    pub attack_serial: u64,
    #[serde(default)]
    pub current_attack: Option<ActorAttackKind>,
    #[serde(default)]
    pub windup_origin_m: Option<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warden: Option<WardenState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordinary: Option<OrdinaryState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<GroupState>,
}

impl ActorRuntime {
    pub fn spawn(entity_id: &str, entity_type: &str, position_m: Vec3) -> Result<Self, String> {
        Self::spawn_with_controller_variant(entity_id, entity_type, position_m, 0)
    }

    pub fn spawn_with_controller_variant(entity_id: &str, entity_type: &str, position_m: Vec3, controller_variant: u8) -> Result<Self, String> {
        let profile = actor_profile_variant(entity_type, controller_variant).ok_or("E_ACTOR_PROFILE_UNKNOWN")?;
        if entity_id.trim().is_empty() || !finite(position_m) {
            return Err("E_ACTOR_SPAWN_INVALID".into());
        }
        Ok(Self {
            entity_id: entity_id.into(),
            entity_type: entity_type.into(),
            controller_variant,
            position_m,
            home_m: position_m,
            hp: profile.max_hp,
            state: ActorAiState::Idle,
            state_remaining_ms: 0,
            attack_serial: 0,
            current_attack: None,
            windup_origin_m: None,
            warden: profile.warden.as_ref().map(|_| WardenState::default()),
            ordinary: profile.ordinary.as_ref().map(|_| OrdinaryState::default()),
            group: profile.group.as_ref().map(GroupState::new),
        })
    }

    fn selected_profile(&self) -> Option<&'static ActorProfile> {
        actor_profile_variant(&self.entity_type, self.controller_variant)
    }

    /// Source-owned interference only. The caller applies Acoustic Mapping to
    /// presentation; this query never changes physics, inputs or capability rules.
    pub fn signal_interferes_at(&self, observer: Vec3, kcc: &StaticKccWorld) -> bool {
        self.selected_profile().is_some_and(|profile| ordinary_controller::signal_interferes_at(self, profile, observer, kcc))
    }

    pub fn body_radius_m(&self) -> f32 {
        self.selected_profile().and_then(|profile|profile.ordinary.as_ref())
            .map_or(0.3,|profile|profile.body_radius_m)
    }

    pub fn validate(&self) -> bool {
        let Some(profile) = self.selected_profile() else {
            return false;
        };
        if !group_members::validate(self, profile) { return false; }
        if profile.ordinary.is_some() {
            let resolved = ordinary_controller::resolved_profile(profile, self.ordinary.as_ref().and_then(|state| state.terrain_variant));
            return self.warden.is_none() && ordinary_controller::validate(self, &resolved, resolved.ordinary.as_ref().unwrap());
        }
        if self.ordinary.is_some() || matches!(self.state, ActorAiState::Active | ActorAiState::Investigate | ActorAiState::Stagger)
            || matches!(self.current_attack, Some(ActorAttackKind::PressureShot | ActorAttackKind::SignalShot | ActorAttackKind::Bite | ActorAttackKind::Leap | ActorAttackKind::Charge | ActorAttackKind::Slam | ActorAttackKind::Lunge | ActorAttackKind::Swing)) {
            return false;
        }
        (match (&profile.warden, &self.warden) {
            (None, None) => true,
            (Some(config), Some(state)) => {
                state.validate(config, self.hp, profile.max_hp, self.attack_serial)
                    && state.actor_state() == self.state
                    && state.remaining_ms == self.state_remaining_ms
                    && state.committed_origin_m.is_none_or(|origin| {
                        origin == self.position_m && horizontal_distance(origin, self.home_m) <= profile.leash_m
                    })
                    && self.current_attack.is_none()
                    && self.windup_origin_m.is_none()
            }
            _ => false,
        }) && !self.entity_id.trim().is_empty()
            && finite(self.position_m)
            && finite(self.home_m)
            && self.hp <= profile.max_hp
            && (self.hp == 0) == (self.state == ActorAiState::Dead)
            && (self.current_attack.is_none() == self.windup_origin_m.is_none())
            && (self.state == ActorAiState::Windup || self.current_attack.is_none())
            && match self.state {
                ActorAiState::Windup | ActorAiState::Cooldown => self.state_remaining_ms > 0,
                _ => self.state_remaining_ms == 0,
            }
    }

    pub(crate) fn ordinary_profile_for_current_phase(&self) -> Option<std::borrow::Cow<'static, ActorProfile>> {
        let profile = self.selected_profile()?;
        profile.ordinary.as_ref()?;
        Some(ordinary_controller::resolved_profile(profile, self.ordinary.as_ref().and_then(|state| state.terrain_variant)))
    }

    pub fn max_hp(&self) -> Option<u32> {
        self.selected_profile().map(|profile| profile.max_hp)
    }

    pub fn take_damage_with_stagger(&mut self, damage: u32, stagger: u32) -> Vec<ActorRuntimeEvent> {
        if !self.validate() { return vec![]; }
        if self.group.is_some() { return group_members::take_budget(self, damage, stagger); }
        if let Some(profile) = self.selected_profile() {
            if profile.ordinary.is_some() {
                let resolved = ordinary_controller::resolved_profile(profile, self.ordinary.as_ref().and_then(|state| state.terrain_variant));
                return ordinary_controller::take_damage(self, &resolved, resolved.ordinary.as_ref().unwrap(), damage, stagger);
            }
        }
        self.take_damage(damage);
        vec![]
    }

    pub fn take_damage(&mut self, damage: u32) {
        if !self.validate() { return; }
        if self.group.is_some() { let _ = group_members::take_budget(self, damage, 0); return; }
        if let Some(profile) = self.selected_profile() {
            if profile.ordinary.is_some() {
                let _ = self.take_damage_with_stagger(damage, 0);
                return;
            }
        }
        if self.hp == 0 {
            return;
        }
        self.hp = self.hp.saturating_sub(damage);
        if let Some(profile) = self.selected_profile() {
            if profile.warden.is_some() {
                warden_controller::take_damage(self, profile, damage);
                return;
            }
        }
        if self.hp == 0 {
            self.state = ActorAiState::Dead;
            self.state_remaining_ms = 0;
            self.current_attack = None;
            self.windup_origin_m = None;
        }
    }

    pub fn view(&self) -> ActorView {
        let profile = self.selected_profile().expect("validated actor type");
        ActorView {
            entity_id: self.entity_id.clone(),
            entity_type: profile.render_type.clone(),
            actor_kind: "enemy".into(),
            transform: Transform {
                position_m: self.position_m,
                yaw_rad: 0.0,
            },
            active: self.hp > 0,
            members: group_members::view_members(self),
            signal_perception: None,
        }
    }

    /// A fixed owner tick. All attack phases and impact checks are Rust-authoritative.
    pub fn tick(
        &mut self,
        player: Vec3,
        kcc: &StaticKccWorld,
        dt_s: f32,
        allow_authored_special_attacks: bool,
    ) -> ActorTickOutput {
        if self.hp == 0 || !self.validate() || !dt_s.is_finite() || dt_s <= 0.0
            || !kcc.stable_actor_footprint(self.position_m,self.body_radius_m()) {
            return ActorTickOutput::default();
        }
        let Some(profile) = self.selected_profile() else {
            return ActorTickOutput::default();
        };
        if profile.ordinary.is_some() {
            let selected = ordinary_controller::terrain_variant_for_tick(self, profile, kcc);
            let resolved = ordinary_controller::resolved_profile(profile, selected);
            return ordinary_controller::tick(self, &resolved, player, kcc, dt_s, selected);
        }
        if profile.warden.is_some() {
            if !allow_authored_special_attacks {
                return ActorTickOutput::default();
            }
            return warden_controller::tick(self, profile, player, kcc, dt_s);
        }
        let dt_ms = (dt_s * 1000.0).round() as u64;
        let distance = horizontal_distance(self.position_m, player);
        let home_distance = horizontal_distance(self.home_m, player);
        let sensed = crate::continuous_combat::combat_vertical_overlap(self.position_m, player)
            && distance <= profile.perception_m
            && home_distance <= profile.leash_m
            && line_clear(self.position_m, player, kcc);
        let mut output = ActorTickOutput::default();
        match self.state {
            ActorAiState::Idle => {
                if sensed {
                    self.state = ActorAiState::Chasing;
                }
            }
            ActorAiState::Chasing => {
                if !sensed {
                    self.state = ActorAiState::Idle;
                } else if distance
                    <= if allow_authored_special_attacks
                        && profile.pressure_wave.is_some()
                        && self.attack_serial % 2 == 0
                    {
                        profile
                            .pressure_wave
                            .as_ref()
                            .map_or(profile.attack_range_m, |wave| wave.radius_m)
                    } else {
                        profile.attack_range_m
                    }
                {
                    self.state = ActorAiState::Windup;
                    let kind = if allow_authored_special_attacks
                        && profile.pressure_wave.is_some()
                        && self.attack_serial % 2 == 0
                    {
                        ActorAttackKind::PressureWave
                    } else {
                        ActorAttackKind::HeavyMelee
                    };
                    let wave = (kind == ActorAttackKind::PressureWave)
                        .then_some(profile.pressure_wave.as_ref())
                        .flatten();
                    self.current_attack = Some(kind);
                    self.windup_origin_m = Some(self.position_m);
                    self.state_remaining_ms = wave.map_or(profile.windup_ms, |wave| wave.windup_ms);
                    output.events.push(ActorRuntimeEvent::AttackWindup {
                        actor_id: self.entity_id.clone(),
                        kind,
                        origin_m: self.position_m,
                        radius_m: wave.map_or(profile.attack_range_m, |wave| wave.radius_m),
                        windup_ms: self.state_remaining_ms,
                    });
                } else {
                    let step = (profile.speed_mps * dt_s).min(distance);
                    let candidate = Vec3 {
                        x_m: self.position_m.x_m
                            + (player.x_m - self.position_m.x_m) / distance * step,
                        y_m: self.position_m.y_m,
                        z_m: self.position_m.z_m
                            + (player.z_m - self.position_m.z_m) / distance * step,
                    };
                    if kcc.can_occupy(candidate, 0.3)
                        && kcc.stable_actor_footprint(candidate,0.3)
                        && horizontal_distance(candidate, self.home_m) <= profile.leash_m
                    {
                        self.position_m = candidate;
                    }
                }
            }
            ActorAiState::Windup => {
                // Old saves may have a melee windup without these optional fields.
                if self.current_attack.is_none() {
                    self.current_attack = Some(ActorAttackKind::HeavyMelee);
                    self.windup_origin_m = Some(self.position_m);
                }
                self.state_remaining_ms = self.state_remaining_ms.saturating_sub(dt_ms);
                if self.state_remaining_ms == 0 {
                    let kind = self.current_attack.unwrap_or(ActorAttackKind::HeavyMelee);
                    let origin = self.windup_origin_m.unwrap_or(self.position_m);
                    let wave = (kind == ActorAttackKind::PressureWave)
                        .then_some(profile.pressure_wave.as_ref())
                        .flatten();
                    let radius = wave.map_or(profile.attack_range_m, |wave| wave.radius_m);
                    let damage = wave.map_or(profile.attack_damage, |wave| wave.damage);
                    let hit_player = if kind == ActorAttackKind::PressureWave {
                        crate::continuous_combat::combat_vertical_overlap(origin, player)
                            && horizontal_distance(origin, player) <= radius
                            && line_clear(origin, player, kcc)
                    } else {
                        crate::continuous_combat::combat_vertical_overlap(self.position_m, player)
                            && distance <= profile.attack_range_m
                            && line_clear(self.position_m, player, kcc)
                    };
                    self.state = ActorAiState::Cooldown;
                    self.state_remaining_ms =
                        wave.map_or(profile.cooldown_ms, |wave| wave.cooldown_ms);
                    self.attack_serial = self.attack_serial.saturating_add(1);
                    output.events.push(ActorRuntimeEvent::AttackImpact {
                        actor_id: self.entity_id.clone(),
                        kind,
                        origin_m: origin,
                        radius_m: radius,
                        damage,
                        hit_player,
                        geometry: None,
                    });
                    if hit_player {
                        output.damage_to_player = Some(damage);
                    }
                    self.current_attack = None;
                    self.windup_origin_m = None;
                }
            }
            ActorAiState::Cooldown => {
                self.state_remaining_ms = self.state_remaining_ms.saturating_sub(dt_ms);
                if self.state_remaining_ms == 0 {
                    self.state = ActorAiState::Idle;
                }
            }
            ActorAiState::Dead | ActorAiState::Investigate | ActorAiState::Active | ActorAiState::Stagger => {}
        }
        output
    }
}

fn finite(position: Vec3) -> bool {
    [position.x_m, position.y_m, position.z_m]
        .iter()
        .all(|value| value.is_finite())
}
fn horizontal_distance(a: Vec3, b: Vec3) -> f32 {
    ((a.x_m - b.x_m).powi(2) + (a.z_m - b.z_m).powi(2)).sqrt()
}
fn line_clear(from: Vec3, to: Vec3, kcc: &StaticKccWorld) -> bool {
    let distance = horizontal_distance(from, to);
    let samples = ((distance / 0.25).ceil() as usize).max(1);
    (0..=samples).all(|index| {
        let t = index as f32 / samples as f32;
        kcc.can_occupy(
            Vec3 {
                x_m: from.x_m + (to.x_m - from.x_m) * t,
                y_m: from.y_m + (to.y_m - from.y_m) * t,
                z_m: from.z_m + (to.z_m - from.z_m) * t,
            },
            0.05,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::continuous_kcc::Aabb;
    use crate::{
        continuous_combat::{ActiveCombatAction, CombatActionKind, CombatEvent, CombatIntent},
        continuous_input::InputSample,
        world_v3::{step_world, CwStepInput, WorldStateV3},
    };

    #[test]
    fn profiles_are_data_driven_and_unknown_types_fail_closed() {
        let worker = ActorRuntime::spawn(
            "worker",
            "grey_hive.infected_maintenance_worker",
            Vec3::zero(),
        )
        .unwrap();
        assert_eq!(worker.max_hp(), Some(50));
        assert!(actor_profile("grey_hive.infected_maintenance_worker").is_some());
        assert!(actor_profile("grey_hive.infected_security").is_some());
        assert!(actor_profile("enemy.mist_harbor.drowned").is_some());
        assert!(actor_profile("enemy.mist_harbor.signal_wraith").is_some());
        assert_eq!(
            actor_profile("enemy.clockworks.forged_guard")
                .unwrap()
                .render_type,
            "runtime2d.enemy.clockworks.forged_guard.v2"
        );
        let elite = ActorRuntime::spawn(
            "cw_forged_guard_elite",
            "enemy.clockworks.forged_guard_elite",
            Vec3::zero(),
        )
        .unwrap();
        assert_eq!(elite.max_hp(), Some(120));
        assert_eq!(
            elite.view().entity_type,
            "runtime2d.enemy.clockworks.forged_guard.v2"
        );
        assert!(ActorRuntime::spawn("unknown", "enemy.not_authorized", Vec3::zero()).is_err());
        assert_eq!(
            ActorRuntime {
                entity_type: "enemy.not_authorized".into(),
                ..worker
            }
            .max_hp(),
            None
        );
    }

    #[test]
    fn deterministic_perception_chase_attack_damage_and_death() {
        let kcc = StaticKccWorld::new(Aabb::new(0.0, 20.0, 0.0, 20.0).unwrap(), vec![]);
        let spawn = Vec3::new(5.0, 0.0, 5.0).unwrap();
        let player = Vec3::new(7.0, 0.0, 5.0).unwrap();
        let mut left =
            ActorRuntime::spawn("enemy-1", "grey_hive.infected_maintenance_worker", spawn).unwrap();
        let mut right = left.clone();
        let mut attacks = 0;
        for _ in 0..180 {
            let a = left.tick(player, &kcc, 1.0 / 60.0, false);
            assert_eq!(a, right.tick(player, &kcc, 1.0 / 60.0, false));
            if a.damage_to_player.is_some() {
                attacks += 1;
            }
        }
        assert_eq!(left, right);
        assert!(left.position_m.x_m > spawn.x_m);
        assert!(attacks > 0);
        left.take_damage(999);
        assert_eq!(left.state, ActorAiState::Dead);
        assert!(!left.view().active);
        assert_eq!(
            left.tick(player, &kcc, 1.0 / 60.0, false).damage_to_player,
            None
        );
    }

    #[test]
    fn forged_guard_profile_has_deterministic_melee_behavior() {
        let kcc = StaticKccWorld::new(Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(), vec![]);
        let spawn = Vec3::new(6.5, 0.0, 8.0).unwrap();
        let player = Vec3::new(9.0, 0.0, 8.0).unwrap();
        let mut guard = ActorRuntime::spawn(
            "cw-entry-forged-guard-01",
            "enemy.clockworks.forged_guard",
            spawn,
        )
        .unwrap();
        assert_eq!(
            guard.view().entity_type,
            "runtime2d.enemy.clockworks.forged_guard.v2"
        );
        let mut attacks = 0;
        for _ in 0..240 {
            if guard
                .tick(player, &kcc, 1.0 / 60.0, true)
                .damage_to_player
                .is_some()
            {
                attacks += 1;
            }
        }
        assert!(guard.position_m.x_m > spawn.x_m);
        assert!(attacks > 0);
        guard.take_damage(999);
        assert_eq!(guard.state, ActorAiState::Dead);
        assert!(!guard.view().active);
        assert_eq!(
            guard.tick(player, &kcc, 1.0 / 60.0, true).damage_to_player,
            None
        );
    }

    #[test]
    fn elite_pressure_wave_is_arena_gated_and_alternates_with_heavy_melee() {
        let profile = actor_profile("enemy.clockworks.forged_guard_elite").unwrap();
        let wave = profile.pressure_wave.as_ref().unwrap();
        assert_eq!(
            (wave.radius_m, wave.damage, wave.windup_ms, wave.cooldown_ms),
            (3.0, 16, 900, 1800)
        );
        assert_eq!(
            (
                profile.attack_damage,
                profile.windup_ms,
                profile.cooldown_ms
            ),
            (14, 700, 1200)
        );
        let kcc = StaticKccWorld::new(Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(), vec![]);
        let mut world =
            WorldStateV3::new("clockworks", 5, Vec3::new(14.5, 0.0, 8.0).unwrap()).unwrap();
        world.scene_id = "cw_forged_guard_arena".into();
        world.generic_actors.push(
            ActorRuntime::spawn(
                "cw_forged_guard_elite",
                "enemy.clockworks.forged_guard_elite",
                Vec3::new(12.0, 0.0, 8.0).unwrap(),
            )
            .unwrap(),
        );
        let mut kinds = vec![];
        for seq in 1..=290 {
            let sample = InputSample::new(5, seq, seq, 0.0, 0.0).unwrap();
            let output = step_world(
                &mut world,
                &kcc,
                CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[],
                },
            )
            .unwrap();
            for event in output.actor_runtime {
                kinds.push(match event {
                    ActorRuntimeEvent::AttackWindup { kind, .. } => ("windup", kind),
                    ActorRuntimeEvent::AttackImpact { kind, .. } => ("impact", kind),
                    ActorRuntimeEvent::EnemyCue { .. } | ActorRuntimeEvent::AttackMotion { .. } | ActorRuntimeEvent::MemberHit { .. } | ActorRuntimeEvent::Relocated { .. } => {
                        panic!("legacy elite must not emit ordinary-controller cues")
                    },
                    ActorRuntimeEvent::WardenCue { .. } => {
                        panic!("ordinary elite must not emit Warden cues")
                    }
                });
            }
        }
        assert_eq!(
            world.player_hp, 70,
            "wave 16 and melee 14 resolve on the owner tick"
        );
        assert!(kinds.windows(4).any(|events| events
            == [
                ("windup", ActorAttackKind::PressureWave),
                ("impact", ActorAttackKind::PressureWave),
                ("windup", ActorAttackKind::HeavyMelee),
                ("impact", ActorAttackKind::HeavyMelee),
            ]));

        let mut outside_arena = ActorRuntime::spawn(
            "elite",
            "enemy.clockworks.forged_guard_elite",
            Vec3::new(12.0, 0.0, 8.0).unwrap(),
        )
        .unwrap();
        let mut outside_first_attack = None;
        for _ in 0..100 {
            let output =
                outside_arena.tick(Vec3::new(14.5, 0.0, 8.0).unwrap(), &kcc, 1.0 / 60.0, false);
            if let Some(ActorRuntimeEvent::AttackWindup { kind, .. }) = output.events.first() {
                outside_first_attack = Some(*kind);
                break;
            }
        }
        assert_eq!(outside_first_attack, Some(ActorAttackKind::HeavyMelee));
    }

    #[test]
    fn actor_runtime_old_windup_save_defaults_to_melee() {
        let mut actor = ActorRuntime::spawn(
            "elite",
            "enemy.clockworks.forged_guard_elite",
            Vec3::new(12.0, 0.0, 8.0).unwrap(),
        )
        .unwrap();
        actor.state = ActorAiState::Windup;
        actor.state_remaining_ms = 1;
        let mut saved = serde_json::to_value(actor).unwrap();
        let object = saved.as_object_mut().unwrap();
        object.remove("currentAttack");
        object.remove("windupOriginM");
        let mut restored: ActorRuntime = serde_json::from_value(saved).unwrap();
        assert!(restored.validate());
        let kcc = StaticKccWorld::new(Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(), vec![]);
        let output = restored.tick(Vec3::new(12.5, 0.0, 8.0).unwrap(), &kcc, 1.0 / 60.0, true);
        assert!(output.events.iter().any(|event| matches!(
            event,
            ActorRuntimeEvent::AttackImpact {
                kind: ActorAttackKind::HeavyMelee,
                ..
            }
        )));
    }

    #[test]
    fn pressure_wave_save_continue_resolves_once_then_resumes_with_heavy_melee() {
        let kcc = StaticKccWorld::new(Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(), vec![]);
        let origin = Vec3::new(12.0, 0.0, 8.0).unwrap();
        let player = Vec3::new(14.5, 0.0, 8.0).unwrap();
        let mut actor = ActorRuntime::spawn(
            "cw_forged_guard_elite",
            "enemy.clockworks.forged_guard_elite",
            origin,
        )
        .unwrap();
        actor.state = ActorAiState::Windup;
        actor.current_attack = Some(ActorAttackKind::PressureWave);
        actor.windup_origin_m = Some(origin);
        actor.state_remaining_ms = 450;
        let saved = serde_json::to_vec(&actor).unwrap();
        let mut continued: ActorRuntime = serde_json::from_slice(&saved).unwrap();
        assert!(continued.validate());

        let mut impacts = 0;
        let mut resolved_damage = 0;
        for _ in 0..30 {
            let output = continued.tick(player, &kcc, 1.0 / 60.0, true);
            impacts += output
                .events
                .iter()
                .filter(|event| {
                    matches!(
                        event,
                        ActorRuntimeEvent::AttackImpact {
                            kind: ActorAttackKind::PressureWave,
                            ..
                        }
                    )
                })
                .count();
            resolved_damage += output.damage_to_player.unwrap_or(0);
        }
        assert_eq!(impacts, 1);
        assert_eq!(resolved_damage, 16);
        assert_eq!(continued.attack_serial, 1);

        let saved_after_impact = serde_json::to_vec(&continued).unwrap();
        let mut continued_again: ActorRuntime =
            serde_json::from_slice(&saved_after_impact).unwrap();
        let mut next_windup = None;
        for _ in 0..180 {
            let output = continued_again.tick(player, &kcc, 1.0 / 60.0, true);
            if let Some(ActorRuntimeEvent::AttackWindup { kind, .. }) = output.events.first() {
                next_windup = Some(*kind);
                break;
            }
        }
        assert_eq!(next_windup, Some(ActorAttackKind::HeavyMelee));
        assert_eq!(continued_again.attack_serial, 1);
    }

    #[test]
    fn pressure_wave_uses_impact_position_occlusion_and_active_guard_only() {
        fn windup_world(player: Vec3) -> WorldStateV3 {
            let mut world = WorldStateV3::new("clockworks", 8, player).unwrap();
            world.scene_id = "cw_forged_guard_arena".into();
            let mut actor = ActorRuntime::spawn(
                "cw_forged_guard_elite",
                "enemy.clockworks.forged_guard_elite",
                Vec3::new(12.0, 0.0, 8.0).unwrap(),
            )
            .unwrap();
            actor.state = ActorAiState::Windup;
            actor.current_attack = Some(ActorAttackKind::PressureWave);
            actor.windup_origin_m = Some(actor.position_m);
            actor.state_remaining_ms = 17;
            world.generic_actors.push(actor);
            world
        }
        fn step(world: &mut WorldStateV3, kcc: &StaticKccWorld) -> Vec<CombatEvent> {
            let sample = InputSample::new(8, 1, 17, 0.0, 0.0).unwrap();
            step_world(
                world,
                kcc,
                CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[],
                },
            )
            .unwrap()
            .combat
        }

        let open = StaticKccWorld::new(Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(), vec![]);
        let mut active_guard = windup_world(Vec3::new(14.5, 0.0, 8.0).unwrap());
        active_guard.combat_state.active_action = Some(ActiveCombatAction {
            kind: CombatActionKind::Guard,
            request_id: 1,
            elapsed_ms: 200,
            impact_resolved: false,
            end_requested: false,
        });
        let guarded = step(&mut active_guard, &open);
        assert_eq!(active_guard.player_hp, 100);
        assert!(guarded.iter().any(
            |event| matches!(event, CombatEvent::GuardImpact { source_id, .. }
            if source_id == "cw_forged_guard_elite")
        ));
        assert!(!guarded
            .iter()
            .any(|event| matches!(event, CombatEvent::PlayerDamaged { .. })));

        let mut inactive_guard = windup_world(Vec3::new(14.5, 0.0, 8.0).unwrap());
        let unguarded = step(&mut inactive_guard, &open);
        assert_eq!(inactive_guard.player_hp, 84);
        assert!(!unguarded
            .iter()
            .any(|event| matches!(event, CombatEvent::GuardImpact { .. })));

        let mut dashed_clear = windup_world(Vec3::new(15.1, 0.0, 8.0).unwrap());
        step(&mut dashed_clear, &open);
        assert_eq!(
            dashed_clear.player_hp, 100,
            "leaving the 3 m radius avoids the impact"
        );

        let wall = StaticKccWorld::new(
            Aabb::new(0.0, 24.0, 0.0, 16.0).unwrap(),
            vec![Aabb::new(13.0, 13.3, 7.5, 8.5).unwrap()],
        );
        let mut occluded = windup_world(Vec3::new(14.5, 0.0, 8.0).unwrap());
        step(&mut occluded, &wall);
        assert_eq!(
            occluded.player_hp, 100,
            "radial damage respects authored occlusion"
        );
    }

    #[test]
    fn owner_combat_hits_generic_actor_and_dead_actor_stops_attacking() {
        let kcc = StaticKccWorld::new(Aabb::new(0.0, 20.0, 0.0, 20.0).unwrap(), vec![]);
        let mut world =
            WorldStateV3::new("grey_hive", 1, Vec3::new(5.0, 0.0, 5.0).unwrap()).unwrap();
        world.generic_actors.push(
            ActorRuntime::spawn(
                "gh-worker",
                "grey_hive.infected_maintenance_worker",
                Vec3::new(5.8, 0.0, 5.0).unwrap(),
            )
            .unwrap(),
        );
        for seq in 1..=2 {
            let sample = InputSample::new(1, seq, seq, 0.0, 0.0).unwrap();
            let output = step_world(
                &mut world,
                &kcc,
                CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[CombatIntent::Attack { request_id: seq }],
                },
            )
            .unwrap();
            assert!(output.combat.iter().any(|event| matches!(event, CombatEvent::AttackHit { target_id, damage: 25, .. } if target_id == "gh-worker")));
        }
        assert_eq!(world.generic_actors[0].hp, 0);
        assert!(!world.generic_actors[0].view().active);
        let hp = world.player_hp;
        for seq in 3..=90 {
            let sample = InputSample::new(1, seq, seq, 0.0, 0.0).unwrap();
            step_world(
                &mut world,
                &kcc,
                CwStepInput {
                    sample: &sample,
                    dt_s: 1.0 / 60.0,
                    combat: &[],
                },
            )
            .unwrap();
        }
        assert_eq!(world.player_hp, hp);
    }
}

#[cfg(test)]
#[path = "ordinary_controller_tests.rs"]
mod ordinary_controller_tests;

#[cfg(test)]
#[path = "brute_controller_tests.rs"]
mod brute_controller_tests;

#[cfg(test)]
#[path = "swarm_controller_tests.rs"]
mod swarm_controller_tests;

#[cfg(test)]
#[path = "tidebound_controller_tests.rs"]
mod tidebound_controller_tests;

#[cfg(test)]
#[path = "signal_controller_tests.rs"]
mod signal_controller_tests;
