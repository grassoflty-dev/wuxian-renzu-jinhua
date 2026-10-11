use crate::{
    continuous_combat::{horizontal_distance, CombatEvent, CombatIntent, CombatState},
    continuous_input::InputSample,
    continuous_kcc::{step_kcc, CollisionEvent, KccBody, StaticKccWorld},
    sentinel_ai::{tick_sentinel, Sentinel, SentinelCommand, SentinelEvent, SentinelState},
};
use serde::{Deserialize, Serialize};

pub const WORLD_STATE_SCHEMA_VERSION: u32 = 3;
pub const WORLD_VIEW_PROTOCOL: &str = "continuous-ipc";
pub const WORLD_VIEW_VERSION: u32 = 1;
pub const WORLD_SCHEMA_VERSION: &str = "freeze-v02-interfaces/1.2";

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec3 {
    pub x_m: f32,
    pub y_m: f32,
    pub z_m: f32,
}
impl Vec3 {
    pub fn new(x_m: f32, y_m: f32, z_m: f32) -> Result<Self, ContinuousWorldError> {
        if [x_m, y_m, z_m].iter().all(|v| v.is_finite()) {
            Ok(Self { x_m, y_m, z_m })
        } else {
            Err(ContinuousWorldError::NonFinite)
        }
    }
    pub fn zero() -> Self {
        Self {
            x_m: 0.0,
            y_m: 0.0,
            z_m: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub position_m: Vec3,
    pub yaw_rad: f32,
}
impl Transform {
    pub fn new(position_m: Vec3, yaw_rad: f32) -> Result<Self, ContinuousWorldError> {
        if yaw_rad.is_finite() {
            Ok(Self {
                position_m,
                yaw_rad,
            })
        } else {
            Err(ContinuousWorldError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldRevision {
    pub world_epoch: u64,
    pub server_tick: u64,
    pub authority_revision: u64,
}
impl WorldRevision {
    pub fn new(
        world_epoch: u64,
        server_tick: u64,
        authority_revision: u64,
    ) -> Result<Self, ContinuousWorldError> {
        Ok(Self {
            world_epoch,
            server_tick,
            authority_revision,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub entity_id: String,
    pub transform: Transform,
    pub velocity_mps: Vec3,
    pub current_hp: u32,
    pub max_hp: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorView {
    pub entity_id: String,
    pub actor_kind: String,
    pub transform: Transform,
    pub active: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorView {
    pub door_id: String,
    pub transform: Transform,
    pub open: bool,
    pub locked: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExploredRoom {
    pub room_id: String,
    pub outline_m: Vec<Vec3>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownConnection {
    pub connection_id: String,
    pub from_room_id: String,
    pub to_room_id: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownObjective {
    pub objective_id: String,
    pub position_m: Vec3,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExploredMap {
    pub world_id: String,
    pub player_position_m: Vec3,
    pub rooms: Vec<ExploredRoom>,
    pub connections: Vec<KnownConnection>,
    pub objectives: Vec<KnownObjective>,
}
impl ExploredMap {
    pub fn new(
        world_id: impl Into<String>,
        player_position_m: Vec3,
        rooms: Vec<ExploredRoom>,
        connections: Vec<KnownConnection>,
        objectives: Vec<KnownObjective>,
    ) -> Result<Self, ContinuousWorldError> {
        let world_id = world_id.into();
        if world_id.trim().is_empty() {
            return Err(ContinuousWorldError::EmptyId);
        };
        Ok(Self {
            world_id,
            player_position_m,
            rooms,
            connections,
            objectives,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VitalTier {
    Healthy,
    Wounded,
    SeverelyWounded,
    Critical,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EnemyVitals {
    pub entity_id: String,
    pub current_hp: u32,
    pub max_hp: u32,
}
impl EnemyVitals {
    pub fn new(
        entity_id: impl Into<String>,
        current_hp: u32,
        max_hp: u32,
    ) -> Result<Self, ContinuousWorldError> {
        let entity_id = entity_id.into();
        if entity_id.trim().is_empty() {
            return Err(ContinuousWorldError::EmptyId);
        }
        if max_hp == 0 || current_hp > max_hp {
            return Err(ContinuousWorldError::InvalidHp);
        }
        Ok(Self {
            entity_id,
            current_hp,
            max_hp,
        })
    }
    pub fn tier(&self) -> VitalTier {
        let ratio = self.current_hp as f32 / self.max_hp as f32;
        if ratio <= 0.25 {
            VitalTier::Critical
        } else if ratio <= 0.60 {
            VitalTier::SeverelyWounded
        } else if ratio < 1.0 {
            VitalTier::Wounded
        } else {
            VitalTier::Healthy
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyVitalProjection {
    pub entity_id: String,
    pub tier: VitalTier,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RearViewAuthorization {
    pub granted: bool,
    pub grant_id: Option<String>,
    pub granted_at_revision: Option<WorldRevision>,
}
impl RearViewAuthorization {
    pub fn denied() -> Self {
        Self {
            granted: false,
            grant_id: None,
            granted_at_revision: None,
        }
    }
    pub fn granted(
        grant_id: impl Into<String>,
        revision: WorldRevision,
    ) -> Result<Self, ContinuousWorldError> {
        let id = grant_id.into();
        if id.trim().is_empty() {
            return Err(ContinuousWorldError::EmptyId);
        }
        Ok(Self {
            granted: true,
            grant_id: Some(id),
            granted_at_revision: Some(revision),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityItemProjection {
    pub capability_id: String,
    pub granted: bool,
    pub selected: bool,
    pub cooldown_remaining_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityProjection {
    pub schema_version: u32,
    pub items: Vec<CapabilityItemProjection>,
    pub explored_map: ExploredMap,
    pub enemy_vitals: Vec<EnemyVitalProjection>,
    pub rear_view: RearViewAuthorization,
}
impl CapabilityProjection {
    pub fn new(
        items: Vec<CapabilityItemProjection>,
        explored_map: ExploredMap,
        enemy_vitals: Vec<EnemyVitalProjection>,
        rear_view: RearViewAuthorization,
    ) -> Self {
        Self {
            schema_version: 1,
            items,
            explored_map,
            enemy_vitals,
            rear_view,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldProgressProjection {
    pub world_id: String,
    pub completed: bool,
    pub first_completion: bool,
    pub visit_id: u64,
    pub cycle_id: u64,
    pub revisit_count: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteProjection {
    pub schema_version: u32,
    pub current_world_id: String,
    pub event_seq: u64,
    pub worlds: Vec<WorldProgressProjection>,
}
impl RouteProjection {
    pub fn new(
        current_world_id: impl Into<String>,
        event_seq: u64,
        worlds: Vec<WorldProgressProjection>,
    ) -> Result<Self, ContinuousWorldError> {
        let id = current_world_id.into();
        if id.trim().is_empty() {
            return Err(ContinuousWorldError::EmptyId);
        }
        Ok(Self {
            schema_version: 1,
            current_world_id: id,
            event_seq,
            worlds,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldView {
    pub protocol: String,
    pub version: u32,
    pub schema_version: String,
    pub world_id: String,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub authority_revision: u64,
    pub ack_seq: u64,
    pub server_time_ms: u64,
    pub player: PlayerView,
    pub actors: Vec<ActorView>,
    pub doors: Vec<DoorView>,
    pub capabilities: CapabilityProjection,
    pub progression: RouteProjection,
}

#[derive(Clone, Debug)]
pub struct WorldStateV3 {
    pub world_id: String,
    pub revision: WorldRevision,
    pub server_time_ms: u64,
    pub player: KccBody,
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub sentinels: Vec<Sentinel>,
    pub combat_state: CombatState,
    pub explored: ExploredMap,
    pub rear_view: RearViewAuthorization,
    pub last_input_seq: u64,
    pub last_client_time_ms: u64,
}
impl WorldStateV3 {
    pub fn new(
        world_id: impl Into<String>,
        world_epoch: u64,
        player_position_m: Vec3,
    ) -> Result<Self, ContinuousWorldError> {
        let id = world_id.into();
        if id.trim().is_empty() {
            return Err(ContinuousWorldError::EmptyId);
        }
        Ok(Self {
            world_id: id.clone(),
            revision: WorldRevision::new(world_epoch, 0, 0)?,
            server_time_ms: 0,
            player: KccBody::new(player_position_m),
            player_hp: 100,
            player_max_hp: 100,
            sentinels: vec![],
            combat_state: CombatState::default(),
            explored: ExploredMap::new(id, player_position_m, vec![], vec![], vec![])?,
            rear_view: RearViewAuthorization::denied(),
            last_input_seq: 0,
            last_client_time_ms: 0,
        })
    }
    pub fn advance_clock(&mut self, dt_s: f32) -> Result<WorldRevision, ContinuousWorldError> {
        if !dt_s.is_finite() || dt_s <= 0.0 {
            return Err(ContinuousWorldError::InvalidDelta);
        }
        self.server_time_ms = self
            .server_time_ms
            .saturating_add((dt_s * 1000.0).round() as u64);
        self.revision.server_tick += 1;
        self.revision.authority_revision += 1;
        Ok(self.revision)
    }
    pub fn set_player_transform(&mut self, t: Transform) {
        self.player.position_m = t.position_m
    }
    pub fn set_player_velocity(&mut self, v: Vec3) {
        self.player.velocity_mps = v
    }
    pub fn view(
        &self,
        ack_seq: u64,
        capabilities: CapabilityProjection,
        progression: RouteProjection,
    ) -> WorldView {
        WorldView {
            protocol: WORLD_VIEW_PROTOCOL.into(),
            version: WORLD_VIEW_VERSION,
            schema_version: WORLD_SCHEMA_VERSION.into(),
            world_id: self.world_id.clone(),
            world_epoch: self.revision.world_epoch,
            server_tick: self.revision.server_tick,
            authority_revision: self.revision.authority_revision,
            ack_seq,
            server_time_ms: self.server_time_ms,
            player: PlayerView {
                entity_id: "player".into(),
                transform: Transform {
                    position_m: self.player.position_m,
                    yaw_rad: 0.0,
                },
                velocity_mps: self.player.velocity_mps,
                current_hp: self.player_hp,
                max_hp: self.player_max_hp,
            },
            actors: self
                .sentinels
                .iter()
                .map(|s| ActorView {
                    entity_id: s.entity_id.clone(),
                    actor_kind: "sentinel".into(),
                    transform: Transform {
                        position_m: s.position_m,
                        yaw_rad: 0.0,
                    },
                    active: s.active,
                })
                .collect(),
            doors: vec![],
            capabilities,
            progression,
        }
    }
}

pub struct CwStepInput<'a> {
    pub sample: &'a InputSample,
    pub dt_s: f32,
    pub combat: &'a [CombatIntent],
}
pub struct CwStepOutput {
    pub revision: WorldRevision,
    pub collisions: Vec<CollisionEvent>,
    pub combat: Vec<CombatEvent>,
    pub sentinel: Vec<SentinelEvent>,
}
pub enum WorldEffect {
    HealPlayer {
        amount: u32,
    },
    RevealRooms {
        rooms: Vec<ExploredRoom>,
        connections: Vec<KnownConnection>,
    },
    SetRearViewAuthorization {
        authorization: RearViewAuthorization,
    },
}

pub fn step_world(
    state: &mut WorldStateV3,
    kcc: &StaticKccWorld,
    input: CwStepInput<'_>,
) -> Result<CwStepOutput, ContinuousWorldError> {
    if input.sample.protocol != "continuous-input"
        || input.sample.world_epoch != state.revision.world_epoch
    {
        return Err(ContinuousWorldError::WrongWorld);
    }
    if input.sample.seq <= state.last_input_seq
        || input.sample.client_time_ms < state.last_client_time_ms
    {
        return Err(ContinuousWorldError::StaleInput);
    }
    let backup = state.clone();
    let result = (|| {
        let axes = input.sample.normalized_axes();
        let mut combat_events = vec![];
        for intent in input.combat {
            match *intent {
                CombatIntent::Jump { request_id } => {
                    if !state.combat_state.claim(request_id) || !state.player.try_jump() {
                        combat_events.push(CombatEvent::IntentRejected {
                            request_id,
                            reason: "jump_rejected".into(),
                        })
                    }
                }
                CombatIntent::Dash { request_id } => {
                    if !state.combat_state.claim(request_id) || !state.player.try_dash() {
                        combat_events.push(CombatEvent::IntentRejected {
                            request_id,
                            reason: "dash_rejected".into(),
                        })
                    }
                }
                CombatIntent::Attack { request_id } => {
                    if !state.combat_state.claim(request_id) {
                        combat_events.push(CombatEvent::IntentRejected {
                            request_id,
                            reason: "duplicate_request".into(),
                        });
                        continue;
                    }
                    let target = state
                        .sentinels
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| {
                            s.active
                                && horizontal_distance(state.player.position_m, s.position_m) <= 1.7
                        })
                        .min_by(|(_, a), (_, b)| {
                            horizontal_distance(state.player.position_m, a.position_m).total_cmp(
                                &horizontal_distance(state.player.position_m, b.position_m),
                            )
                        })
                        .map(|(i, _)| i);
                    if let Some(i) = target {
                        let id = state.sentinels[i].entity_id.clone();
                        state.sentinels[i].take_damage(25);
                        combat_events.push(CombatEvent::AttackHit {
                            request_id,
                            target_id: id,
                            damage: 25,
                        })
                    } else {
                        combat_events.push(CombatEvent::AttackMiss { request_id })
                    }
                }
            }
        }
        let collisions = step_kcc(&mut state.player, kcc, axes, input.dt_s)
            .map_err(|_| ContinuousWorldError::InvalidDelta)?;
        let mut sentinel_events = vec![];
        for s in &mut state.sentinels {
            let (e, c) = tick_sentinel(s, state.player.position_m, input.dt_s);
            sentinel_events.extend(e);
            for event in c {
                if let CombatEvent::PlayerDamaged { damage, .. } = event {
                    state.player_hp = state.player_hp.saturating_sub(damage)
                }
                combat_events.push(event)
            }
        }
        state.explored.player_position_m = state.player.position_m;
        state.last_input_seq = input.sample.seq;
        state.last_client_time_ms = input.sample.client_time_ms;
        let revision = state.advance_clock(input.dt_s)?;
        Ok(CwStepOutput {
            revision,
            collisions,
            combat: combat_events,
            sentinel: sentinel_events,
        })
    })();
    if result.is_err() {
        *state = backup
    }
    result
}

pub fn apply_effects_atomically(
    state: &mut WorldStateV3,
    effects: &[WorldEffect],
) -> Result<WorldRevision, ContinuousWorldError> {
    for e in effects {
        match e {
            WorldEffect::HealPlayer { amount } if *amount == 0 => {
                return Err(ContinuousWorldError::InvalidEffect)
            }
            WorldEffect::RevealRooms { rooms, .. }
                if rooms
                    .iter()
                    .any(|r| r.room_id.trim().is_empty() || r.outline_m.len() < 3) =>
            {
                return Err(ContinuousWorldError::InvalidEffect)
            }
            WorldEffect::SetRearViewAuthorization { authorization }
                if authorization.granted
                    && (authorization.grant_id.is_none()
                        || authorization.granted_at_revision.is_none()) =>
            {
                return Err(ContinuousWorldError::InvalidEffect)
            }
            _ => {}
        }
    }
    for e in effects {
        match e {
            WorldEffect::HealPlayer { amount } => {
                state.player_hp = state
                    .player_hp
                    .saturating_add(*amount)
                    .min(state.player_max_hp)
            }
            WorldEffect::RevealRooms { rooms, connections } => {
                state.explored.rooms.extend(rooms.clone());
                state.explored.connections.extend(connections.clone())
            }
            WorldEffect::SetRearViewAuthorization { authorization } => {
                state.rear_view = authorization.clone()
            }
        }
    }
    state.revision.authority_revision += 1;
    Ok(state.revision)
}
pub fn apply_sentinel(
    state: &mut WorldStateV3,
    command: SentinelCommand,
) -> Result<SentinelEvent, ContinuousWorldError> {
    match command {
        SentinelCommand::Query => state
            .sentinels
            .first()
            .map(|s| SentinelEvent::Queried {
                sentinel_id: s.entity_id.clone(),
                state: s.state,
            })
            .ok_or(ContinuousWorldError::UnknownSentinel),
        SentinelCommand::Activate { sentinel_id } => {
            let s = state
                .sentinels
                .iter_mut()
                .find(|s| s.entity_id == sentinel_id)
                .ok_or(ContinuousWorldError::UnknownSentinel)?;
            if s.state == SentinelState::Death {
                return Err(ContinuousWorldError::DeadSentinel);
            }
            s.active = true;
            Ok(SentinelEvent::Activated { sentinel_id })
        }
        SentinelCommand::Deactivate { sentinel_id } => {
            let s = state
                .sentinels
                .iter_mut()
                .find(|s| s.entity_id == sentinel_id)
                .ok_or(ContinuousWorldError::UnknownSentinel)?;
            s.active = false;
            Ok(SentinelEvent::Deactivated { sentinel_id })
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContinuousWorldError {
    EmptyId,
    NonFinite,
    InvalidHp,
    InvalidDelta,
    WrongWorld,
    InvalidEffect,
    UnknownSentinel,
    DeadSentinel,
    StaleInput,
}
