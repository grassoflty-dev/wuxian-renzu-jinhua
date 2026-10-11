//! Pure data contract for the future continuous Grey Hive world.
//!
//! This module deliberately contains no runtime, physics, renderer, IPC, or
//! filesystem code. `WorldStateV3` is the only mutable owner in this contract;
//! `WorldView` and the SaveV3 DTOs are serializable projections only.

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use std::fmt;

pub const WORLD_STATE_SCHEMA_VERSION: u32 = 3;
pub const WORLD_VIEW_PROTOCOL: &str = "continuous-ipc";
pub const WORLD_VIEW_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldModelError {
    NonFinite { field: &'static str },
    EmptyField { field: &'static str },
    ClockRegression { field: &'static str },
    InvalidSchemaVersion { expected: u32, actual: u32 },
}

impl fmt::Display for WorldModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { field } => write!(formatter, "{field} must be finite"),
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::ClockRegression { field } => write!(formatter, "{field} must be monotonic"),
            Self::InvalidSchemaVersion { expected, actual } => {
                write!(formatter, "schemaVersion must be {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for WorldModelError {}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), WorldModelError> {
    if value.is_empty() {
        Err(WorldModelError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn require_finite(value: f32, field: &'static str) -> Result<(), WorldModelError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(WorldModelError::NonFinite { field })
    }
}

fn normalize_yaw(yaw_rad: f32) -> Result<f32, WorldModelError> {
    require_finite(yaw_rad, "yawRad")?;
    let tau = std::f32::consts::TAU;
    let pi = std::f32::consts::PI;
    let mut normalized = (yaw_rad + pi).rem_euclid(tau) - pi;
    if normalized <= -pi {
        normalized = pi;
    }
    Ok(normalized)
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec3 {
    x_m: f32,
    y_m: f32,
    z_m: f32,
}

impl Vec3 {
    pub fn new(x_m: f32, y_m: f32, z_m: f32) -> Result<Self, WorldModelError> {
        require_finite(x_m, "xM")?;
        require_finite(y_m, "yM")?;
        require_finite(z_m, "zM")?;
        Ok(Self { x_m, y_m, z_m })
    }

    pub const fn zero() -> Self {
        Self {
            x_m: 0.0,
            y_m: 0.0,
            z_m: 0.0,
        }
    }

    pub const fn x_m(&self) -> f32 {
        self.x_m
    }

    pub const fn y_m(&self) -> f32 {
        self.y_m
    }

    pub const fn z_m(&self) -> f32 {
        self.z_m
    }
}

impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            x_m: f32,
            y_m: f32,
            z_m: f32,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.x_m, wire.y_m, wire.z_m).map_err(D::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    position_m: Vec3,
    yaw_rad: f32,
}

impl Transform {
    pub fn new(position_m: Vec3, yaw_rad: f32) -> Result<Self, WorldModelError> {
        Ok(Self {
            position_m,
            yaw_rad: normalize_yaw(yaw_rad)?,
        })
    }

    pub const fn position_m(&self) -> Vec3 {
        self.position_m
    }

    pub const fn yaw_rad(&self) -> f32 {
        self.yaw_rad
    }
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            position_m: Vec3,
            yaw_rad: f32,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.position_m, wire.yaw_rad).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorView {
    actor_id: String,
    kind: String,
    transform: Transform,
    velocity_mps: Vec3,
}

impl ActorView {
    pub fn new(
        actor_id: impl Into<String>,
        kind: impl Into<String>,
        transform: Transform,
        velocity_mps: Vec3,
    ) -> Result<Self, WorldModelError> {
        let actor_id = actor_id.into();
        let kind = kind.into();
        require_non_empty(&actor_id, "actorId")?;
        require_non_empty(&kind, "kind")?;
        Ok(Self {
            actor_id,
            kind,
            transform,
            velocity_mps,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorView {
    door_id: String,
    open: bool,
}

impl DoorView {
    pub fn new(door_id: impl Into<String>, open: bool) -> Result<Self, WorldModelError> {
        let door_id = door_id.into();
        require_non_empty(&door_id, "doorId")?;
        Ok(Self { door_id, open })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    transform: Transform,
    velocity_mps: Vec3,
}

impl PlayerView {
    fn new(transform: Transform, velocity_mps: Vec3) -> Self {
        Self {
            transform,
            velocity_mps,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldStateV3 {
    schema_version: u32,
    world_id: String,
    world_epoch: u64,
    server_tick: u64,
    server_time_ms: u64,
    player_transform: Transform,
    player_velocity_mps: Vec3,
    actors: Vec<ActorView>,
    doors: Vec<DoorView>,
}

impl<'de> Deserialize<'de> for WorldStateV3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Wire {
            schema_version: u32,
            world_id: String,
            world_epoch: u64,
            server_tick: u64,
            server_time_ms: u64,
            player_transform: Transform,
            player_velocity_mps: Vec3,
            actors: Vec<ActorView>,
            doors: Vec<DoorView>,
        }

        let wire = Wire::deserialize(deserializer)?;
        if wire.schema_version != WORLD_STATE_SCHEMA_VERSION {
            return Err(D::Error::custom(WorldModelError::InvalidSchemaVersion {
                expected: WORLD_STATE_SCHEMA_VERSION,
                actual: wire.schema_version,
            }));
        }
        Self::new(
            wire.world_id,
            wire.world_epoch,
            wire.server_tick,
            wire.server_time_ms,
            wire.player_transform,
            wire.player_velocity_mps,
            wire.actors,
            wire.doors,
        )
        .map_err(D::Error::custom)
    }
}

impl WorldStateV3 {
    pub fn new(
        world_id: impl Into<String>,
        world_epoch: u64,
        server_tick: u64,
        server_time_ms: u64,
        player_transform: Transform,
        player_velocity_mps: Vec3,
        actors: Vec<ActorView>,
        doors: Vec<DoorView>,
    ) -> Result<Self, WorldModelError> {
        let world_id = world_id.into();
        require_non_empty(&world_id, "worldId")?;
        Ok(Self {
            schema_version: WORLD_STATE_SCHEMA_VERSION,
            world_id,
            world_epoch,
            server_tick,
            server_time_ms,
            player_transform,
            player_velocity_mps,
            actors,
            doors,
        })
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn world_id(&self) -> &str {
        &self.world_id
    }

    pub const fn world_epoch(&self) -> u64 {
        self.world_epoch
    }

    pub const fn server_tick(&self) -> u64 {
        self.server_tick
    }

    pub const fn server_time_ms(&self) -> u64 {
        self.server_time_ms
    }

    pub fn set_player_transform(&mut self, transform: Transform) {
        self.player_transform = transform;
    }

    pub fn set_player_velocity(&mut self, velocity_mps: Vec3) {
        self.player_velocity_mps = velocity_mps;
    }

    pub fn advance_clock(
        &mut self,
        server_tick: u64,
        server_time_ms: u64,
    ) -> Result<(), WorldModelError> {
        if server_tick < self.server_tick {
            return Err(WorldModelError::ClockRegression {
                field: "serverTick",
            });
        }
        if server_time_ms < self.server_time_ms {
            return Err(WorldModelError::ClockRegression {
                field: "serverTimeMs",
            });
        }
        self.server_tick = server_tick;
        self.server_time_ms = server_time_ms;
        Ok(())
    }

    pub fn view(&self, ack_seq: u64) -> WorldView {
        WorldView {
            protocol: WORLD_VIEW_PROTOCOL.to_owned(),
            version: WORLD_VIEW_VERSION,
            world_epoch: self.world_epoch,
            server_tick: self.server_tick,
            ack_seq,
            server_time_ms: self.server_time_ms,
            world_id: self.world_id.clone(),
            player: PlayerView::new(self.player_transform, self.player_velocity_mps),
            actors: self.actors.clone(),
            doors: self.doors.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldView {
    protocol: String,
    version: u32,
    world_epoch: u64,
    server_tick: u64,
    ack_seq: u64,
    server_time_ms: u64,
    world_id: String,
    player: PlayerView,
    actors: Vec<ActorView>,
    doors: Vec<DoorView>,
}

impl WorldView {
    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn world_epoch(&self) -> u64 {
        self.world_epoch
    }

    pub const fn server_tick(&self) -> u64 {
        self.server_tick
    }

    pub const fn ack_seq(&self) -> u64 {
        self.ack_seq
    }

    pub const fn server_time_ms(&self) -> u64 {
        self.server_time_ms
    }

    pub fn world_id(&self) -> &str {
        &self.world_id
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointRef {
    world_id: String,
    checkpoint_id: String,
}

impl CheckpointRef {
    pub fn new(
        world_id: impl Into<String>,
        checkpoint_id: impl Into<String>,
    ) -> Result<Self, WorldModelError> {
        let world_id = world_id.into();
        let checkpoint_id = checkpoint_id.into();
        require_non_empty(&world_id, "worldId")?;
        require_non_empty(&checkpoint_id, "checkpointId")?;
        Ok(Self {
            world_id,
            checkpoint_id,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveV3 {
    schema_version: u32,
    save_id: String,
    checkpoint: CheckpointRef,
    transform: Option<Transform>,
}

impl SaveV3 {
    pub fn new(
        save_id: impl Into<String>,
        checkpoint: CheckpointRef,
        transform: Option<Transform>,
    ) -> Result<Self, WorldModelError> {
        let save_id = save_id.into();
        require_non_empty(&save_id, "saveId")?;
        Ok(Self {
            schema_version: WORLD_STATE_SCHEMA_VERSION,
            save_id,
            checkpoint,
            transform,
        })
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
}
