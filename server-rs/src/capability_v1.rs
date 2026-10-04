use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::world_v3::{
    CapabilityItemProjection, CapabilityProjection, ContinuousWorldError, EnemyVitalProjection,
    EnemyVitals, ExploredMap, RearViewAuthorization, WorldEffect, WorldRevision,
};

pub const CAPABILITY_SCHEMA_VERSION: u32 = 1;
pub const CAP_LOCAL_MAP: &str = "information.local_map_i";
pub const CAP_REAR_VIEW: &str = "perception.rear_view_i";
pub const CAP_ENEMY_VITALS: &str = "information.enemy_vitals_basic";
pub const CAP_REGENERATION: &str = "body.regeneration_i";
pub const CAP_ACOUSTIC_MAPPING: &str = "perception.acoustic_mapping_i";
pub const CAP_AIR_STEP: &str = "mobility.air_step_i";
const FIRST_ENHANCEMENT_CHOICES: [&str; 3] = [CAP_LOCAL_MAP, CAP_REAR_VIEW, CAP_REGENERATION];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerationConfig {
    pub delay_after_damage_ms: u64,
    pub delay_after_combat_ms: u64,
    pub hp_per_second: f32,
    pub regen_cap_fraction: f32,
}

impl RegenerationConfig {
    pub fn new(
        delay_after_damage_ms: u64,
        delay_after_combat_ms: u64,
        hp_per_second: f32,
        regen_cap_fraction: f32,
    ) -> Result<Self, CapabilityError> {
        let config = Self {
            delay_after_damage_ms,
            delay_after_combat_ms,
            hp_per_second,
            regen_cap_fraction,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), CapabilityError> {
        if !self.hp_per_second.is_finite() || self.hp_per_second < 0.0 {
            return Err(CapabilityError::InvalidConfig);
        }
        if !self.regen_cap_fraction.is_finite()
            || self.regen_cap_fraction <= 0.0
            || self.regen_cap_fraction > 1.0
        {
            return Err(CapabilityError::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityGrant {
    pub capability_id: String,
    pub granted_at_revision: WorldRevision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityState {
    pub schema_version: u32,
    pub grants: Vec<CapabilityGrant>,
    pub selected: Vec<String>,
    pub regeneration: RegenerationConfig,
    pub first_enhancement_choice: Option<String>,
    pub world_id: String,
    pub explored_map: ExploredMap,
    pub rear_view_authorization: RearViewAuthorization,
    current_revision: WorldRevision,
    last_tick_revision: Option<WorldRevision>,
    last_now_ms: Option<u64>,
    last_damage_at_ms: Option<u64>,
    last_combat_at_ms: Option<u64>,
    fractional_heal: f64,
    handled_request_ids: Vec<u64>,
}

impl CapabilityState {
    pub fn new(
        world_id: impl Into<String>,
        regeneration: RegenerationConfig,
    ) -> Result<Self, CapabilityError> {
        regeneration.validate()?;
        let world_id = world_id.into();
        if world_id.trim().is_empty() {
            return Err(CapabilityError::EmptyId);
        }
        let explored_map = ExploredMap::new(
            world_id.clone(),
            crate::world_v3::Vec3::zero(),
            vec![],
            vec![],
            vec![],
        )?;
        Ok(Self {
            schema_version: CAPABILITY_SCHEMA_VERSION,
            grants: vec![],
            selected: vec![],
            regeneration,
            first_enhancement_choice: None,
            world_id,
            explored_map,
            rear_view_authorization: RearViewAuthorization::denied(),
            current_revision: WorldRevision::new(0, 0, 0)?,
            last_tick_revision: None,
            last_now_ms: None,
            last_damage_at_ms: None,
            last_combat_at_ms: None,
            fractional_heal: 0.0,
            handled_request_ids: vec![],
        })
    }

    pub fn set_explored_map(&mut self, explored: ExploredMap) -> Result<(), CapabilityError> {
        validate_map(&explored)?;
        if explored.world_id != self.world_id {
            return Err(CapabilityError::WrongWorld);
        }
        self.explored_map = explored;
        Ok(())
    }

    pub fn current_revision(&self) -> WorldRevision {
        self.current_revision
    }

    fn has_grant(&self, capability_id: &str) -> bool {
        self.grants.iter().any(|g| g.capability_id == capability_id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CapabilityCommand {
    Grant {
        capability_id: String,
    },
    Select {
        capability_ids: Vec<String>,
    },
    Activate {
        capability_id: String,
        request_id: u64,
    },
}

pub struct CapabilityTickInput<'a> {
    pub revision: WorldRevision,
    pub now_ms: u64,
    pub dt_s: f32,
    pub current_hp: u32,
    pub max_hp: u32,
    pub last_damaged_at_ms: Option<u64>,
    pub last_combat_at_ms: Option<u64>,
    pub combat_events: &'a [crate::continuous_combat::CombatEvent],
}

pub struct CapabilityTickOutput {
    pub effects: Vec<WorldEffect>,
    pub projection: CapabilityProjection,
}

pub fn apply_command(
    state: &mut CapabilityState,
    command: CapabilityCommand,
) -> Result<Vec<WorldEffect>, CapabilityError> {
    apply_command_at_revision(state, command, state.current_revision)
}

pub fn apply_command_at_revision(
    state: &mut CapabilityState,
    command: CapabilityCommand,
    revision: WorldRevision,
) -> Result<Vec<WorldEffect>, CapabilityError> {
    if !revision_at_least(revision, state.current_revision) {
        return Err(CapabilityError::StaleRevision);
    }
    let mut candidate = state.clone();
    candidate.current_revision = revision;
    let effects = match command {
        CapabilityCommand::Grant { capability_id } => {
            validate_id(&capability_id)?;
            if !is_canonical(&capability_id) {
                return Err(CapabilityError::UnknownCapability);
            }
            if candidate.has_grant(&capability_id) {
                return Err(CapabilityError::DuplicateGrant);
            }
            candidate.grants.push(CapabilityGrant {
                capability_id: capability_id.clone(),
                granted_at_revision: revision,
            });
            if capability_id == CAP_REAR_VIEW {
                candidate.rear_view_authorization =
                    RearViewAuthorization::granted(CAP_REAR_VIEW, revision)?;
                vec![WorldEffect::SetRearViewAuthorization {
                    authorization: candidate.rear_view_authorization.clone(),
                }]
            } else {
                vec![]
            }
        }
        CapabilityCommand::Select { capability_ids } => {
            if capability_ids.is_empty() {
                return Err(CapabilityError::EmptySelection);
            }
            let mut seen = BTreeSet::new();
            for id in &capability_ids {
                validate_id(id)?;
                if !is_canonical(id) {
                    return Err(CapabilityError::UnknownCapability);
                }
                if !candidate.has_grant(id) {
                    return Err(CapabilityError::NotGranted);
                }
                if !seen.insert(id) {
                    return Err(CapabilityError::DuplicateSelection);
                }
            }
            candidate.selected = capability_ids;
            vec![]
        }
        CapabilityCommand::Activate {
            capability_id,
            request_id,
        } => {
            validate_id(&capability_id)?;
            if !is_canonical(&capability_id) {
                return Err(CapabilityError::UnknownCapability);
            }
            if !candidate.has_grant(&capability_id) {
                return Err(CapabilityError::NotGranted);
            }
            if candidate.handled_request_ids.contains(&request_id) {
                return Err(CapabilityError::DuplicateRequest);
            }
            candidate.handled_request_ids.push(request_id);
            if capability_id == CAP_REAR_VIEW {
                candidate.rear_view_authorization =
                    RearViewAuthorization::granted(CAP_REAR_VIEW, revision)?;
                vec![WorldEffect::SetRearViewAuthorization {
                    authorization: candidate.rear_view_authorization.clone(),
                }]
            } else {
                vec![]
            }
        }
    };
    *state = candidate;
    Ok(effects)
}

pub fn choose_first_enhancement(
    state: &mut CapabilityState,
    capability_id: &str,
    revision: WorldRevision,
) -> Result<Vec<WorldEffect>, CapabilityError> {
    if state.world_id != "return_station" {
        return Err(CapabilityError::WrongWorld);
    }
    if state.first_enhancement_choice.is_some() {
        return Err(CapabilityError::FirstEnhancementAlreadyChosen);
    }
    if !FIRST_ENHANCEMENT_CHOICES.contains(&capability_id) {
        return Err(CapabilityError::IllegalFirstEnhancement);
    }
    if !revision_at_least(revision, state.current_revision) {
        return Err(CapabilityError::StaleRevision);
    }
    let mut candidate = state.clone();
    candidate.current_revision = revision;
    candidate.first_enhancement_choice = Some(capability_id.to_owned());
    if !candidate.has_grant(capability_id) {
        candidate.grants.push(CapabilityGrant {
            capability_id: capability_id.to_owned(),
            granted_at_revision: revision,
        });
    }
    if !candidate.selected.iter().any(|id| id == capability_id) {
        candidate.selected.push(capability_id.to_owned());
    }
    let mut effects = vec![];
    if capability_id == CAP_REAR_VIEW {
        candidate.rear_view_authorization =
            RearViewAuthorization::granted(CAP_REAR_VIEW, revision)?;
        effects.push(WorldEffect::SetRearViewAuthorization {
            authorization: candidate.rear_view_authorization.clone(),
        });
    }
    *state = candidate;
    Ok(effects)
}

pub fn tick_regeneration(
    state: &mut CapabilityState,
    input: CapabilityTickInput<'_>,
) -> Result<CapabilityTickOutput, CapabilityError> {
    let authorized = state.has_grant(CAP_REGENERATION);
    tick_regeneration_authorized(state, input, authorized)
}

/// Runtime permission comes from resolved sources; the existing saved tuning and clocks remain.
pub(crate) fn tick_regeneration_authorized(
    state: &mut CapabilityState,
    input: CapabilityTickInput<'_>,
    authorized: bool,
) -> Result<CapabilityTickOutput, CapabilityError> {
    state.regeneration.validate()?;
    if input.max_hp == 0 || input.current_hp > input.max_hp {
        return Err(CapabilityError::InvalidHp);
    }
    if !input.dt_s.is_finite() || input.dt_s <= 0.0 {
        return Err(CapabilityError::InvalidDelta);
    }
    if input
        .last_damaged_at_ms
        .into_iter()
        .chain(input.last_combat_at_ms)
        .any(|time| time > input.now_ms)
    {
        return Err(CapabilityError::FutureTimestamp);
    }
    if state.last_now_ms.is_some_and(|last| input.now_ms < last) {
        return Err(CapabilityError::NonMonotonicClock);
    }
    if state
        .last_tick_revision
        .is_some_and(|last| !revision_after(input.revision, last))
        || !revision_at_least(input.revision, state.current_revision)
    {
        return Err(CapabilityError::StaleRevision);
    }

    let event_activity = input.combat_events.iter().any(crate::continuous_combat::CombatEvent::counts_as_combat_activity);
    let event_damage = input.combat_events.iter().any(|event| {
        matches!(
            event,
            crate::continuous_combat::CombatEvent::PlayerDamaged { .. }
        )
    });
    let damage_at = if event_damage {
        Some(input.now_ms)
    } else {
        input.last_damaged_at_ms
    };
    let combat_at = if event_activity {
        Some(input.now_ms)
    } else {
        input.last_combat_at_ms
    };
    if timestamp_regressed(damage_at, state.last_damage_at_ms)
        || timestamp_regressed(combat_at, state.last_combat_at_ms)
    {
        return Err(CapabilityError::NonMonotonicClock);
    }

    let mut candidate = state.clone();
    if damage_at != candidate.last_damage_at_ms || combat_at != candidate.last_combat_at_ms {
        candidate.fractional_heal = 0.0;
    }
    candidate.last_damage_at_ms = damage_at;
    candidate.last_combat_at_ms = combat_at;
    candidate.last_now_ms = Some(input.now_ms);
    candidate.last_tick_revision = Some(input.revision);
    candidate.current_revision = input.revision;

    let mut effects = Vec::new();
    if authorized
        && delay_elapsed(
            input.now_ms,
            damage_at,
            candidate.regeneration.delay_after_damage_ms,
        )
        && delay_elapsed(
            input.now_ms,
            combat_at,
            candidate.regeneration.delay_after_combat_ms,
        )
    {
        let cap_hp = ((input.max_hp as f64 * candidate.regeneration.regen_cap_fraction as f64)
            .floor() as u32)
            .min(input.max_hp);
        let room = cap_hp.saturating_sub(input.current_hp);
        if room == 0 {
            candidate.fractional_heal = 0.0;
        } else {
            let earned = candidate.fractional_heal
                + candidate.regeneration.hp_per_second as f64 * input.dt_s as f64;
            let whole = earned.floor().min(u32::MAX as f64) as u32;
            let heal = whole.min(room);
            candidate.fractional_heal = if heal < room && whole == heal {
                earned - whole as f64
            } else {
                0.0
            };
            if heal > 0 {
                effects.push(WorldEffect::HealPlayer { amount: heal });
            }
        }
    }
    let projection = project_capabilities(
        &candidate,
        candidate.explored_map.clone(),
        vec![],
        candidate.rear_view_authorization.clone(),
    );
    *state = candidate;
    Ok(CapabilityTickOutput {
        effects,
        projection,
    })
}

pub fn project_capabilities(
    state: &CapabilityState,
    explored: ExploredMap,
    enemy_vitals: Vec<EnemyVitals>,
    rear_view: RearViewAuthorization,
) -> CapabilityProjection {
    let explored_map = if state.has_grant(CAP_LOCAL_MAP)
        && validate_map(&explored).is_ok()
        && explored.world_id == state.world_id
    {
        explored
    } else {
        ExploredMap {
            world_id: state.world_id.clone(),
            player_position_m: explored.player_position_m,
            rooms: vec![],
            connections: vec![],
            objectives: vec![],
        }
    };
    let enemy_projection = project_enemy_tiers(&enemy_vitals, state.has_grant(CAP_ENEMY_VITALS));
    let rear_valid = state.has_grant(CAP_REAR_VIEW)
        && state.rear_view_authorization.granted
        && rear_view == state.rear_view_authorization
        && rear_view.grant_id.as_deref() == Some(CAP_REAR_VIEW);
    let rear_view = if rear_valid {
        rear_view
    } else {
        RearViewAuthorization::denied()
    };
    let items = [
        CAP_LOCAL_MAP,
        CAP_REAR_VIEW,
        CAP_ENEMY_VITALS,
        CAP_REGENERATION,
        CAP_ACOUSTIC_MAPPING,
        CAP_AIR_STEP,
    ]
    .into_iter()
    .map(|id| CapabilityItemProjection {
        capability_id: id.to_owned(),
        granted: state.has_grant(id),
        selected: state.selected.iter().any(|selected| selected == id),
        cooldown_remaining_ms: 0,
    })
    .collect();
    let mut projection = CapabilityProjection::new(items, explored_map, enemy_projection, rear_view);
    projection.first_enhancement_choice = state.first_enhancement_choice.clone();
    projection
}

/// Preserve acquisition rows and legacy callers while the formal runtime authorizes information
/// through current resolved rules. Fully qualified rules also support source-inclusion fixtures.
pub(crate) fn project_capabilities_with_rules(
    state: &CapabilityState,
    explored: ExploredMap,
    enemy_vitals: Vec<EnemyVitals>,
    rear_view: RearViewAuthorization,
    rules: &wuxian_horror_ch1::player_rules::EffectivePlayerRules,
) -> CapabilityProjection {
    let mut projection = project_capabilities(state, explored, vec![], rear_view);
    projection.enemy_vitals = project_enemy_tiers(
        &enemy_vitals,
        rules
            .capability_permissions
            .contains(&wuxian_horror_ch1::effects::CapabilityPermission::EnemyVitalsBasic),
    );
    projection
}

fn project_enemy_tiers(
    enemy_vitals: &[EnemyVitals],
    authorized: bool,
) -> Vec<EnemyVitalProjection> {
    let mut seen = BTreeSet::new();
    if authorized {
        enemy_vitals
            .iter()
            .filter(|v| {
                !v.entity_id.trim().is_empty()
                    && v.current_hp > 0
                    && v.max_hp > 0
                    && v.current_hp <= v.max_hp
            })
            .filter(|v| seen.insert(v.entity_id.as_str()))
            .map(|v| EnemyVitalProjection {
                entity_id: v.entity_id.clone(),
                tier: v.tier(),
            })
            .collect()
    } else {
        vec![]
    }
}

fn validate_map(map: &ExploredMap) -> Result<(), CapabilityError> {
    if map.world_id.trim().is_empty() {
        return Err(CapabilityError::EmptyId);
    }
    if ![
        map.player_position_m.x_m,
        map.player_position_m.y_m,
        map.player_position_m.z_m,
    ]
    .iter()
    .all(|v| v.is_finite())
    {
        return Err(CapabilityError::InvalidMap);
    }
    let room_ids: BTreeSet<_> = map.rooms.iter().map(|room| room.room_id.as_str()).collect();
    if room_ids.len() != map.rooms.len()
        || map.rooms.iter().any(|room| {
            room.room_id.trim().is_empty()
                || room.outline_m.len() < 3
                || room
                    .outline_m
                    .iter()
                    .any(|p| ![p.x_m, p.y_m, p.z_m].iter().all(|v| v.is_finite()))
        })
        || map.connections.iter().any(|c| {
            c.connection_id.trim().is_empty()
                || !room_ids.contains(c.from_room_id.as_str())
                || !room_ids.contains(c.to_room_id.as_str())
        })
        || map.objectives.iter().any(|o| {
            o.objective_id.trim().is_empty()
                || ![o.position_m.x_m, o.position_m.y_m, o.position_m.z_m]
                    .iter()
                    .all(|v| v.is_finite())
        })
    {
        return Err(CapabilityError::InvalidMap);
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), CapabilityError> {
    if id.trim().is_empty() {
        Err(CapabilityError::EmptyId)
    } else {
        Ok(())
    }
}
fn is_canonical(id: &str) -> bool {
    [
        CAP_LOCAL_MAP,
        CAP_REAR_VIEW,
        CAP_ENEMY_VITALS,
        CAP_REGENERATION,
        CAP_ACOUSTIC_MAPPING,
        CAP_AIR_STEP,
    ]
    .contains(&id)
}
fn revision_tuple(r: WorldRevision) -> (u64, u64, u64) {
    (r.world_epoch, r.server_tick, r.authority_revision)
}
fn revision_at_least(a: WorldRevision, b: WorldRevision) -> bool {
    revision_tuple(a) >= revision_tuple(b)
}
fn revision_after(a: WorldRevision, b: WorldRevision) -> bool {
    revision_tuple(a) > revision_tuple(b)
}
fn timestamp_regressed(next: Option<u64>, previous: Option<u64>) -> bool {
    matches!((next,previous),(Some(n),Some(p)) if n<p)
}
fn delay_elapsed(now: u64, then: Option<u64>, delay: u64) -> bool {
    then.map_or(true, |time| now.saturating_sub(time) >= delay)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityError {
    EmptyId,
    UnknownCapability,
    DuplicateGrant,
    NotGranted,
    DuplicateSelection,
    EmptySelection,
    DuplicateRequest,
    StaleRevision,
    FirstEnhancementAlreadyChosen,
    IllegalFirstEnhancement,
    WrongWorld,
    InvalidConfig,
    InvalidHp,
    InvalidDelta,
    FutureTimestamp,
    NonMonotonicClock,
    InvalidMap,
}

impl From<ContinuousWorldError> for CapabilityError {
    fn from(value: ContinuousWorldError) -> Self {
        match value {
            ContinuousWorldError::EmptyId => Self::EmptyId,
            _ => Self::InvalidMap,
        }
    }
}
