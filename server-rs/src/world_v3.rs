use crate::{
    continuous_combat::{
        combat_v1, horizontal_distance, ActiveCombatAction, CombatActionData, CombatActionKind,
        CombatContact, CombatEvent, CombatIntent, CombatState,
    },
    continuous_input::InputSample,
    continuous_kcc::{CollisionEvent, KccBody, StaticKccWorld},
    sentinel_ai::{tick_sentinel, Sentinel, SentinelCommand, SentinelEvent, SentinelState},
};
use serde::{Deserialize, Serialize};
#[path = "world_v3/actor_runtime.rs"]
mod actor_runtime;
pub use actor_runtime::{ActorAiState, ActorAttackKind, ActorCueKind, ActorRuntime, ActorRuntimeEvent, ActorAttackGeometry, OrdinaryState, OrdinaryActiveAttack, OrdinaryTerrainVariant, WardenAttack, WardenConfig, WardenCueKind, WardenStage, WardenState};

pub(crate) fn actor_profile(entity_type: &str) -> Option<&'static actor_runtime::ActorProfile> {
    actor_runtime::actor_profile(entity_type)
}

pub(crate) fn actor_profile_for_warden() -> Option<&'static actor_runtime::ActorProfile> {
    actor_runtime::actor_profile("enemy.mist_harbor.resonance_warden")
}

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
    pub current_energy: u32,
    pub max_energy: u32,
    pub facing_x: f32,
    pub facing_z: f32,
    pub aim_x: f32,
    pub aim_z: f32,
    pub action_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_presentation: Option<ActionPresentation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPhase { Windup, Active, Recovery }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPresentation {
    pub request_id: u64,
    pub phase: ActionPhase,
    /// Elapsed and total duration of the entire action, including early Guard recovery.
    pub elapsed_ms: u64,
    pub duration_ms: u64,
    pub range_m: f32,
    pub line_half_width_m: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorMemberView {
    pub member_id: String,
    pub position_m: Vec3,
    pub radius_m: f32,
    pub active: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal_perception: Option<crate::signal_perception::SignalPositionProjection>,
    pub entity_id: String,
    /// Stable content identity; renderers resolve it through their own registry.
    pub entity_type: String,
    pub actor_kind: String,
    pub transform: Transform,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<ActorMemberView>>,
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
    #[serde(default)]
    pub first_enhancement_choice: Option<String>,
    pub items: Vec<CapabilityItemProjection>,
    pub explored_map: ExploredMap,
    /// Resolved authorization, independent of permanent capability acquisition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_topology_authorized: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acoustic_mapping_authorized: Option<bool>,
    /// Rust-filtered current-scene knowledge. Never contains exploration history writes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_knowledge: Option<wuxian_horror_ch1::effects::MapKnowledgeProjection>,
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
            first_enhancement_choice: None,
            items,
            explored_map,
            map_topology_authorized: None,
            acoustic_mapping_authorized: None,
            map_knowledge: None,
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
    #[serde(default)]
    pub completed_events: Vec<String>,
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
/// Ephemeral owner-issued identity for one prepared destination. Never persisted in saves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SceneEntryToken {
    pub generation: u64,
    pub world_id: String,
    pub scene_id: String,
    pub world_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldView {
    pub protocol: String,
    pub version: u32,
    pub schema_version: String,
    pub world_id: String,
    #[serde(default)]
    pub scene_id: String,
    #[serde(default)]
    pub checkpoint_id: Option<String>,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub authority_revision: u64,
    pub ack_seq: u64,
    pub server_time_ms: u64,
    pub player: PlayerView,
    pub actors: Vec<ActorView>,
    pub doors: Vec<DoorView>,
    #[serde(default)]
    pub interactables: Vec<InteractableView>,
    #[serde(default)]
    pub hazards: Vec<HazardView>,
    #[serde(default)]
    pub objectives: Vec<ObjectiveView>,
    pub capabilities: CapabilityProjection,
    pub progression: RouteProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<wuxian_horror_ch1::build_projection::BuildProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_encounter: Option<wuxian_horror_ch1::boss_view_v1::BossEncounterView>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub sentinel_encounter: Option<crate::sentinel_ai::SentinelEncounterView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mist_harbor_pump: Option<MistHarborPumpProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grey_hive_beacon: Option<GreyHiveBeaconProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baizhi: Option<crate::formal_runtime::baizhi::BaizhiProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npcs: Option<Vec<crate::formal_runtime::baizhi::BaizhiNpcProjection>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_token: Option<SceneEntryToken>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub support_scene: Option<crate::moving_support::SupportSceneView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GreyHiveBeaconProjection {
    pub state: crate::world_persistent_v1::GreyHiveBeaconStage,
    pub legacy_completed_without_receipt: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MistHarborPumpProjection {
    pub state: MistHarborPumpState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MistHarborPumpState {
    Ready,
    Draining,
    Drained,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSnapshotV2 {
    pub kind: String,
    pub protocol_version: u32,
    pub schema_version: String,
    pub world_id: String,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub authority_revision: u64,
    pub ack_seq: u64,
    pub view: WorldView,
}

/// Canonical flattened Snapshot v3 envelope. The nested v2 shape remains
/// available as `WorldSnapshotV2` for migration fixtures and old consumers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSnapshot {
    pub kind: String,
    pub protocol_version: u32,
    pub schema_version: String,
    pub world_id: String,
    pub scene_id: String,
    pub checkpoint_id: Option<String>,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub authority_revision: u64,
    pub ack_seq: u64,
    pub player: PlayerView,
    pub actors: Vec<ActorView>,
    pub doors: Vec<DoorView>,
    pub interactables: Vec<InteractableView>,
    pub hazards: Vec<HazardView>,
    pub objectives: Vec<ObjectiveView>,
    pub capabilities: CapabilityProjection,
    pub progression: RouteProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<wuxian_horror_ch1::build_projection::BuildProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_encounter: Option<wuxian_horror_ch1::boss_view_v1::BossEncounterView>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub sentinel_encounter: Option<crate::sentinel_ai::SentinelEncounterView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mist_harbor_pump: Option<MistHarborPumpProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grey_hive_beacon: Option<GreyHiveBeaconProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baizhi: Option<crate::formal_runtime::baizhi::BaizhiProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npcs: Option<Vec<crate::formal_runtime::baizhi::BaizhiNpcProjection>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_token: Option<SceneEntryToken>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub support_scene: Option<crate::moving_support::SupportSceneView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractableView {
    pub entity_id: String,
    pub kind: String,
    pub transform: Transform,
    pub active: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentHazardProjection {
    pub phase: wuxian_horror_ch1::environment_hazards::EnvironmentPhase,
    pub remaining_ms: u64,
    pub exposure_bps: u32,
    pub tag: wuxian_horror_ch1::effects::HazardTag,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HazardView {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<EnvironmentHazardProjection>,
    pub entity_id: String,
    pub kind: String,
    pub transform: Transform,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polygon_m: Option<Vec<[f32; 2]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning_remaining_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_active: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveView {
    pub objective_id: String,
    pub state: String,
    pub position_m: Vec3,
}

impl WorldSnapshot {
    pub fn from_view(view: WorldView) -> Self {
        Self {
            kind: "full".into(),
            protocol_version: 3,
            schema_version: WORLD_SCHEMA_VERSION.into(),
            world_id: view.world_id.clone(),
            scene_id: view.scene_id.clone(),
            checkpoint_id: view.checkpoint_id.clone(),
            world_epoch: view.world_epoch,
            server_tick: view.server_tick,
            authority_revision: view.authority_revision,
            ack_seq: view.ack_seq,
            player: view.player,
            actors: view.actors,
            doors: view.doors,
            interactables: view.interactables,
            hazards: view.hazards,
            objectives: view.objectives,
            capabilities: view.capabilities,
            progression: view.progression,
            build: view.build,
            boss_encounter: view.boss_encounter,
            sentinel_encounter: view.sentinel_encounter,
            mist_harbor_pump: view.mist_harbor_pump,
            grey_hive_beacon: view.grey_hive_beacon,
            baizhi: view.baizhi,
            npcs: view.npcs,
            entry_token: view.entry_token,
            support_scene: view.support_scene,
        }
    }
}

/// Ephemeral presentation signal. The renderer must not infer these from
/// successive snapshots; event identity is scoped to one world epoch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationEvent {
    pub protocol_version: u32,
    pub event_id: u64,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub kind: String,
    pub position_m: Vec3,
    pub direction_rad: f32,
    pub radius_m: f32,
    pub intensity: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub half_angle_rad: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_kind: Option<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub attack_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range_m: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_feedback: Option<CombatFeedback>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatOutcome { EnemyHit, PlayerHurt, Blocked, Absorbed, Rejected }

/// Public combat result: no HP, damage amount, or hidden enemy state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatFeedback {
    pub world_id: String,
    pub scene_id: String,
    pub outcome: CombatOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Ephemeral, epoch-scoped acoustic event. Source coordinates are never sent;
/// bearing and distance are captured only when Acoustic Mapping is active.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundCueEvent {
    pub protocol_version: u32,
    pub event_id: u64,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub world_id: String,
    pub scene_id: String,
    pub kind: String,
    pub direction_rad: Option<f32>,
    pub distance_m: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandReceipt {
    pub command_id: String,
    pub applied: bool,
    pub already_applied: bool,
    pub error_code: Option<String>,
    pub world_epoch: u64,
    pub server_tick: u64,
    pub authority_revision: u64,
    pub snapshot: WorldSnapshot,
}

impl CommandReceipt {
    pub fn from_view(command_id: impl Into<String>, view: WorldView) -> Self {
        Self::outcome(command_id, true, false, None, view)
    }

    pub fn outcome(
        command_id: impl Into<String>,
        applied: bool,
        already_applied: bool,
        error_code: Option<String>,
        view: WorldView,
    ) -> Self {
        let snapshot = WorldSnapshot::from_view(view);
        Self {
            command_id: command_id.into(),
            applied,
            already_applied,
            error_code,
            world_epoch: snapshot.world_epoch,
            server_tick: snapshot.server_tick,
            authority_revision: snapshot.authority_revision,
            snapshot,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorldStateV3 {
    pub world_id: String,
    pub scene_id: String,
    pub checkpoint_id: Option<String>,
    pub revision: WorldRevision,
    pub server_time_ms: u64,
    pub player: KccBody,
    pub player_hp: u32,
    pub player_max_hp: u32,
    pub player_energy: u32,
    pub player_max_energy: u32,
    pub sentinels: Vec<Sentinel>,
    pub generic_actors: Vec<ActorRuntime>,
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
            scene_id: id.clone(),
            checkpoint_id: None,
            revision: WorldRevision::new(world_epoch, 0, 0)?,
            server_time_ms: 0,
            player: KccBody::new(player_position_m),
            player_hp: 100,
            player_max_hp: 100,
            // Provisional owner-side meter; no current skill command spends it.
            player_energy: 100,
            player_max_energy: 100,
            sentinels: vec![],
            generic_actors: vec![],
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
        self.revision.server_tick = self
            .revision
            .server_tick
            .checked_add(1)
            .ok_or(ContinuousWorldError::RevisionExhausted)?;
        self.bump_authority_revision()?;
        Ok(self.revision)
    }
    pub fn bump_authority_revision(&mut self) -> Result<WorldRevision, ContinuousWorldError> {
        self.revision.authority_revision = self
            .revision
            .authority_revision
            .checked_add(1)
            .ok_or(ContinuousWorldError::RevisionExhausted)?;
        Ok(self.revision)
    }
    pub fn actors_have_stable_support(&self,kcc:&StaticKccWorld)->bool {
        self.generic_actors.iter().all(|actor|(actor.position_m.y_m-actor.home_m.y_m).abs()<=0.001
            && kcc.stable_actor_footprint(actor.position_m,actor.body_radius_m()))
            && self.sentinels.iter().all(|actor|actor.charge_geometry_valid(kcc) && kcc.stable_actor_footprint(actor.position_m,0.3))
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
            scene_id: self.scene_id.clone(),
            checkpoint_id: self.checkpoint_id.clone(),
            world_epoch: self.revision.world_epoch,
            server_tick: self.revision.server_tick,
            authority_revision: self.revision.authority_revision,
            ack_seq,
            server_time_ms: self.server_time_ms,
            player: PlayerView {
                entity_id: "player".into(),
                transform: Transform {
                    position_m: self.player.position_m,
                    yaw_rad: self.combat_state.facing_x.atan2(self.combat_state.facing_z),
                },
                velocity_mps: self.player.velocity_mps,
                current_hp: self.player_hp,
                max_hp: self.player_max_hp,
                current_energy: self.player_energy,
                max_energy: self.player_max_energy,
                facing_x: self.combat_state.facing_x,
                facing_z: self.combat_state.facing_z,
                aim_x: self.combat_state.facing_x,
                aim_z: self.combat_state.facing_z,
                action_state: if self.player_hp == 0 { "death".into() } else { self.combat_state.action_state().into() },
                action_presentation: if self.player_hp == 0 { None } else { self.combat_state.action_presentation() },
            },
            actors: self
                .sentinels
                .iter()
                .map(|s| ActorView {
                    entity_id: s.entity_id.clone(),
                    entity_type: "enemy.grey_hive.sentinel".into(),
                    actor_kind: "sentinel".into(),
                    transform: Transform {
                        position_m: s.position_m,
                        yaw_rad: 0.0,
                    },
                    active: s.active, members: None, signal_perception: None,
                })
                .chain(
                    self.generic_actors
                        .iter()
                        .filter(|actor| actor.validate())
                        .map(ActorRuntime::view),
                )
                .collect(),
            doors: vec![],
            interactables: vec![],
            hazards: vec![],
            objectives: vec![],
            capabilities,
            progression,
            build: None,
            boss_encounter: None,
            sentinel_encounter: None,
            mist_harbor_pump: None,
            grey_hive_beacon: None,
            baizhi: None,
            npcs: None,
            entry_token: None,
            support_scene: None,
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
    pub actor_runtime: Vec<ActorRuntimeEvent>,
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
    step_world_internal(state, kcc, input, false)
}

/// A simulation owner may keep the last client input over several fixed ticks.
/// Sequence validation remains at the mailbox boundary; this variant only
/// permits equality with the most recently applied client sequence.
pub fn step_world_held_input(
    state: &mut WorldStateV3,
    kcc: &StaticKccWorld,
    input: CwStepInput<'_>,
) -> Result<CwStepOutput, ContinuousWorldError> {
    step_world_internal(state, kcc, input, true)
}

fn step_world_internal(
    state: &mut WorldStateV3,
    kcc: &StaticKccWorld,
    input: CwStepInput<'_>,
    held_input: bool,
) -> Result<CwStepOutput, ContinuousWorldError> {
    if input.sample.protocol != "continuous-input"
        || input.sample.protocol_version != crate::continuous_input::INPUT_PROTOCOL_VERSION
        || input.sample.world_epoch != state.revision.world_epoch
    {
        return Err(ContinuousWorldError::WrongWorld);
    }
    match input.sample.validate_axes() {
        Ok(()) => {}
        Err(crate::continuous_input::InputError::NonFinite) => {
            return Err(ContinuousWorldError::InputNonFinite)
        }
        Err(crate::continuous_input::InputError::AxisOutOfRange) => {
            return Err(ContinuousWorldError::InputAxisOutOfRange)
        }
    }
    if input.sample.seq < state.last_input_seq
        || (!held_input && input.sample.seq == state.last_input_seq)
        || input.sample.client_time_ms < state.last_client_time_ms
    {
        return Err(ContinuousWorldError::StaleInput);
    }
    if state.sentinels.iter().any(|s|!s.validate()) {return Err(ContinuousWorldError::InvalidDelta);}
    let backup = state.clone();
    let result = (|| {
        let mut axes = input.sample.normalized_axes();
        let dt_ms = (input.dt_s * 1000.0).round() as u64;
        state.combat_state.tick_timers(dt_ms);
        if !state.combat_state.action_locks_facing {
            let aim_length_sq =
                input.sample.aim_x * input.sample.aim_x + input.sample.aim_z * input.sample.aim_z;
            if aim_length_sq >= 0.04 {
                (state.combat_state.facing_x, state.combat_state.facing_z) =
                    normalize_horizontal(input.sample.aim_x, input.sample.aim_z);
            }
        }
        let mut combat_events = vec![];
        let mut actor_runtime_events = vec![];
        for intent in input.combat {
            begin_combat_intent(state, kcc, *intent, axes, &mut combat_events, &mut actor_runtime_events);
        }
        let traversing_this_step = state.combat_state.active_traversal.is_some();
        advance_combat_action(state, kcc, dt_ms, &mut combat_events, &mut actor_runtime_events);
        advance_context_traversal(state, kcc, dt_ms, &mut combat_events);
        if traversing_this_step {
            axes = (0.0, 0.0);
        } else if state.player.dash_remaining_ms > 0 {
            axes = (state.combat_state.dash_x, state.combat_state.dash_z);
        }
        let collisions = crate::continuous_kcc::step_kcc_at_world_time(
            &mut state.player, kcc, axes, input.dt_s, !traversing_this_step, state.server_time_ms)
            .map_err(|_| ContinuousWorldError::InvalidDelta)?;
        let mut sentinel_events = vec![];
        let combat_data = combat_v1();
        let guard_spec = &combat_data.actions["guard"];
        let guarding = state
            .combat_state
            .guard_active(guard_spec.windup_ms, guard_spec.active_ms);
        let guard_request_id = state.combat_state.active_action.as_ref()
            .filter(|action| action.kind == CombatActionKind::Guard)
            .map(|action| action.request_id);
        for s in &mut state.sentinels {
            if state.player_hp == 0 { break; }
            let (e, c) = tick_sentinel(s, state.player.position_m, input.dt_s, kcc);
            sentinel_events.extend(e);
            for event in c {
                if let CombatEvent::PlayerDamaged { source_id, damage, contact } = event {
                    if guarding {
                        let stagger_ms = u64::from(guard_spec.stagger)
                            .saturating_mul(combat_data.stagger_ms_per_point);
                        s.apply_stagger(stagger_ms);
                        let absorbed = ((damage as f32)
                            * combat_data.guard_damage_reduction.clamp(0.0, 1.0))
                        .round() as u32;
                        let applied_damage = damage.saturating_sub(absorbed);
                        state.player_hp = state.player_hp.saturating_sub(applied_damage);
                        combat_events.push(CombatEvent::GuardImpact {
                            source_id: source_id.clone(),
                            stagger: guard_spec.stagger,
                            contact: contact.map(|contact| CombatContact { request_id: guard_request_id, ..contact }),
                        });
                        if applied_damage > 0 {
                            combat_events.push(CombatEvent::PlayerDamaged {
                                source_id: source_id.clone(),
                                damage: applied_damage,
                                contact,
                            });
                        }
                    } else if state.combat_state.qer_v1_active
                        && state.combat_state.invulnerability_remaining_ms > 0
                    {
                        combat_events.push(CombatEvent::DamageAbsorbed {
                            source_id, reason: "invulnerable".into(), contact,
                        });
                    } else {
                        let applied_damage = damage;
                        state.player_hp = state.player_hp.saturating_sub(applied_damage);
                        if state.combat_state.qer_v1_active {
                            state.combat_state.invulnerability_remaining_ms =
                                combat_data.invulnerability_ms;
                        }
                        if applied_damage > 0 {
                            combat_events.push(CombatEvent::PlayerDamaged {
                                source_id,
                                damage: applied_damage,
                                contact,
                            });
                        }
                    }
                }
            }
        }
        let allow_authored_special_attacks = state.world_id == "clockworks"
            && ((state.scene_id == "cw_forged_guard_arena"
                && state.generic_actors.len() == 1
                && state.generic_actors[0].entity_id == "cw_forged_guard_elite"
                && state.generic_actors[0].entity_type == "enemy.clockworks.forged_guard_elite"
                && state.generic_actors[0].validate())
                || (state.scene_id == "cw_regulator_core"
                    && state.generic_actors.len() == 1
                    && state.generic_actors[0].entity_id == "cw_prime_regulator"
                    && state.generic_actors[0].entity_type == "enemy.clockworks.prime_regulator"
                    && state.generic_actors[0].validate()));
        let allow_authored_special_attacks = allow_authored_special_attacks || (
            state.player_hp > 0 && state.world_id == "mist_harbor" && state.scene_id == "mh_warden_arena"
            && state.generic_actors.len() == 1
            && state.generic_actors[0].entity_id == "mh_resonance_warden_staged_marker"
            && state.generic_actors[0].entity_type == "enemy.mist_harbor.resonance_warden"
            && state.generic_actors[0].validate());
        for actor in &mut state.generic_actors {
            if state.player_hp == 0 { break; }
            let actor_output = actor.tick(
                state.player.position_m,
                kcc,
                input.dt_s,
                allow_authored_special_attacks,
            );
            let mut contact = CombatContact::new(actor.position_m, state.player.position_m, None);
            if let Some(direction_rad) = actor_output.events.iter().find_map(|event| match event {
                ActorRuntimeEvent::AttackImpact { hit_player: true, geometry: Some(geometry), .. } => Some(geometry.direction_rad),
                _ => None,
            }) {
                contact.direction_rad = direction_rad;
            }
            let contact = Some(contact);
            actor_runtime_events.extend(actor_output.events);
            if let Some(raw_damage) = actor_output.damage_to_player {
                let source_id = actor.entity_id.clone();
                // Same basis-point rounding as the shared encounter hazard path.
                // Untagged ordinary melee keeps its existing physical damage.
                let damage = actor_output.damage_tag.map_or(raw_damage, |tag| {
                    ((u64::from(raw_damage) * u64::from(kcc.remaining_hazard_damage_bps(tag))
                        + 5_000) / 10_000) as u32
                });
                if damage == 0 {
                    combat_events.push(CombatEvent::DamageAbsorbed {
                        source_id, reason: "damage_reduced_to_zero".into(), contact,
                    });
                    continue;
                }
                if guarding {
                    // A projectile block does not invent a remote counter-stagger.
                    if !actor_output.ranged_damage {
                        actor_runtime_events.extend(actor.take_damage_with_stagger(0, guard_spec.stagger));
                    }
                    let absorbed = ((damage as f32)
                        * combat_data.guard_damage_reduction.clamp(0.0, 1.0))
                    .round() as u32;
                    let applied = damage.saturating_sub(absorbed);
                    state.player_hp = state.player_hp.saturating_sub(applied);
                    combat_events.push(CombatEvent::GuardImpact {
                        source_id: source_id.clone(),
                        stagger: guard_spec.stagger,
                        contact: contact.map(|contact| CombatContact { request_id: guard_request_id, ..contact }),
                    });
                    if applied > 0 {
                        combat_events.push(CombatEvent::PlayerDamaged {
                            source_id: source_id.clone(),
                            damage: applied,
                            contact,
                        });
                    }
                } else if state.combat_state.qer_v1_active
                    && state.combat_state.invulnerability_remaining_ms > 0
                {
                    combat_events.push(CombatEvent::DamageAbsorbed {
                        source_id, reason: "invulnerable".into(), contact,
                    });
                } else {
                    state.player_hp = state.player_hp.saturating_sub(damage);
                    if state.combat_state.qer_v1_active {
                        state.combat_state.invulnerability_remaining_ms =
                            combat_data.invulnerability_ms;
                    }
                    combat_events.push(CombatEvent::PlayerDamaged { source_id, damage, contact });
                }
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
            actor_runtime: actor_runtime_events,
        })
    })();
    if result.is_err() {
        *state = backup
    }
    result
}

fn normalize_horizontal(x: f32, z: f32) -> (f32, f32) {
    let length = (x * x + z * z).sqrt();
    if length <= f32::EPSILON {
        (0.0, 0.0)
    } else {
        (x / length, z / length)
    }
}

fn begin_combat_intent(
    state: &mut WorldStateV3,
    kcc: &StaticKccWorld,
    intent: CombatIntent,
    movement: (f32, f32),
    events: &mut Vec<CombatEvent>,
    actor_events: &mut Vec<ActorRuntimeEvent>,
) {
    let request_id = match intent {
        CombatIntent::Attack { request_id }
        | CombatIntent::Jump { request_id }
        | CombatIntent::Dash { request_id }
        | CombatIntent::Pulse { request_id }
        | CombatIntent::GuardStart { request_id }
        | CombatIntent::GuardEnd { request_id }
        | CombatIntent::Pierce { request_id }
        | CombatIntent::ActionAttack { request_id }
        | CombatIntent::ActionDash { request_id }
        | CombatIntent::ContextTraversal { request_id, .. } => request_id,
    };
    let intent_contact = Some(CombatContact::new(
        state.player.position_m, state.player.position_m, Some(request_id),
    ));
    if !state.combat_state.claim(request_id) {
        events.push(CombatEvent::IntentRejected {
            request_id,
            reason: "duplicate_or_stale_request".into(),
            contact: intent_contact,
        });
        return;
    }

    if let CombatIntent::Jump { request_id } = intent {
        if state.combat_state.active_action.is_some()
            || state.combat_state.active_traversal.is_some()
            || !state.player.try_jump()
        {
            events.push(CombatEvent::IntentRejected {
                request_id,
                reason: "jump_rejected".into(),
                contact: intent_contact,
            });
        }
        return;
    }
    if let CombatIntent::Attack { request_id } = intent {
        let sentinel_target = state
            .sentinels
            .iter()
            .enumerate()
            .filter(|(_, sentinel)| {
                sentinel.active
                    && crate::continuous_combat::combat_vertical_overlap(state.player.position_m, sentinel.position_m)
                    && horizontal_distance(state.player.position_m, sentinel.position_m) <= 1.7
            })
            .min_by(|(_, left), (_, right)| {
                horizontal_distance(state.player.position_m, left.position_m).total_cmp(
                    &horizontal_distance(state.player.position_m, right.position_m),
                )
            })
            .map(|(index, sentinel)| {
                (
                    horizontal_distance(state.player.position_m, sentinel.position_m),
                    index,
                )
            });
        let actor_target = state.generic_actors.iter().enumerate()
            .flat_map(|(index, actor)| actor.combat_points(state.player.position_m, kcc).into_iter()
                .map(move |(member, point)| (index, member, point)))
            .filter_map(|(index, member, point)| {
                let distance = horizontal_distance(state.player.position_m, point);
                (distance <= 1.7).then_some((distance, index, member, point))
            }).min_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)));
        if actor_target
            .is_some_and(|actor| sentinel_target.is_none_or(|sentinel| actor.0 < sentinel.0))
        {
            let index = actor_target.unwrap().1;
            let target_id = state.generic_actors[index].entity_id.clone();
            let contact = Some(CombatContact::new(state.player.position_m, actor_target.unwrap().3, Some(request_id)));
            if let Some(member) = actor_target.unwrap().2 {
                if let Ok(hit_events) = state.generic_actors[index].take_member_hits(&[member], 25, 0) {
                    actor_events.extend(hit_events);
                }
            } else { actor_events.extend(state.generic_actors[index].take_damage_with_stagger(25, 0)); }
            events.push(CombatEvent::AttackHit {
                request_id,
                target_id,
                damage: 25,
                modern: false,
                contact,
            });
        } else if let Some((_, index)) = sentinel_target {
            let target_id = state.sentinels[index].entity_id.clone();
            let contact = Some(CombatContact::new(state.player.position_m, state.sentinels[index].position_m, Some(request_id)));
            state.sentinels[index].take_damage(25);
            events.push(CombatEvent::AttackHit {
                request_id,
                target_id,
                damage: 25,
                modern: false,
                contact,
            });
        } else {
            events.push(CombatEvent::AttackMiss { request_id, modern: false, contact: intent_contact });
        }
        return;
    }
    if let CombatIntent::Dash { .. } = intent {
        if !state.player.try_dash() {
            events.push(CombatEvent::IntentRejected {
                request_id,
                reason: "dash_rejected".into(),
                contact: intent_contact,
            });
            return;
        }
        let direction = if movement.0 * movement.0 + movement.1 * movement.1 >= 0.04 {
            normalize_horizontal(movement.0, movement.1)
        } else {
            (state.combat_state.facing_x, state.combat_state.facing_z)
        };
        state.combat_state.dash_x = direction.0;
        state.combat_state.dash_z = direction.1;
        return;
    }
    if let CombatIntent::GuardEnd { request_id } = intent {
        if let Some(action) = state.combat_state.active_action.as_mut() {
            if action.kind == CombatActionKind::Guard && !action.end_requested {
                action.end_requested = true;
                events.push(CombatEvent::GuardEnded { request_id });
                return;
            }
        }
        events.push(CombatEvent::IntentRejected {
            request_id,
            reason: "guard_not_active".into(),
            contact: intent_contact,
        });
        return;
    }

    if let CombatIntent::ContextTraversal {
        request_id,
        marker_index,
    } = intent
    {
        if state.combat_state.active_action.is_some()
            || state.combat_state.active_traversal.is_some()
            || state.player.dash_remaining_ms > 0
        {
            events.push(CombatEvent::IntentRejected {
                request_id,
                reason: "action_busy".into(),
                contact: intent_contact,
            });
            return;
        }
        let Some(marker) = state
            .combat_state
            .traversal_markers
            .get(marker_index)
            .cloned()
        else {
            events.push(CombatEvent::IntentRejected {
                request_id,
                reason: "traversal_marker_missing".into(),
                contact: intent_contact,
            });
            return;
        };
        if horizontal_distance(state.player.position_m, marker.from) > marker.range_m
            || marker.from_height_range_m.is_some_and(|range|!range.contains(state.player.position_m.y_m)) {
            events.push(CombatEvent::IntentRejected {
                request_id,
                reason: "traversal_out_of_range".into(),
                contact: intent_contact,
            });
            return;
        }
        if state
            .combat_state
            .traversal_cooldowns_ms
            .get(&marker.id)
            .copied()
            .unwrap_or(0)
            > 0
        {
            events.push(CombatEvent::IntentRejected {
                request_id,
                reason: "traversal_cooldown".into(),
                contact: intent_contact,
            });
            return;
        }
        state
            .combat_state
            .traversal_cooldowns_ms
            .insert(marker.id.clone(), marker.cooldown_ms);
        state.combat_state.active_traversal = Some(crate::continuous_combat::ActiveTraversal {
            marker_index,
            request_id,
            start_position: state.player.position_m,
            elapsed_ms: 0,
        });
        events.push(CombatEvent::TraversalStarted {
            request_id,
            marker_id: marker.id,
        });
        return;
    }

    let (kind, dash) = match intent {
        CombatIntent::Attack { .. } => (CombatActionKind::PrimaryAttack, false),
        CombatIntent::Dash { .. } => (CombatActionKind::Dash, true),
        CombatIntent::ActionAttack { .. } => (CombatActionKind::PrimaryAttack, false),
        CombatIntent::ActionDash { .. } => (CombatActionKind::Dash, true),
        CombatIntent::Pulse { .. } => (CombatActionKind::Pulse, false),
        CombatIntent::GuardStart { .. } => (CombatActionKind::Guard, false),
        CombatIntent::Pierce { .. } => (CombatActionKind::Pierce, false),
        CombatIntent::Jump { .. }
        | CombatIntent::GuardEnd { .. }
        | CombatIntent::ContextTraversal { .. } => return,
    };
    if state.combat_state.active_action.is_some() || state.combat_state.active_traversal.is_some() {
        events.push(CombatEvent::IntentRejected {
            request_id,
            reason: "action_busy".into(),
            contact: intent_contact,
        });
        return;
    }
    let tuning = combat_v1();
    let spec = &tuning.actions[kind.data_key()];
    if state
        .combat_state
        .cooldowns_ms
        .get(&kind)
        .copied()
        .unwrap_or(0)
        > 0
    {
        events.push(CombatEvent::IntentRejected {
            request_id,
            reason: "cooldown".into(),
            contact: intent_contact,
        });
        return;
    }
    if state.player_energy < spec.energy {
        events.push(CombatEvent::IntentRejected {
            request_id,
            reason: "insufficient_energy".into(),
            contact: intent_contact,
        });
        return;
    }
    if dash
        && !state
            .player
            .try_dash_for_distance(spec.active_ms, spec.cooldown_ms, spec.range_m)
    {
        events.push(CombatEvent::IntentRejected {
            request_id,
            reason: "dash_rejected".into(),
            contact: intent_contact,
        });
        return;
    }
    state.player_energy -= spec.energy;
    state
        .combat_state
        .cooldowns_ms
        .insert(kind, spec.cooldown_ms);
    if dash {
        let facing = if movement.0 * movement.0 + movement.1 * movement.1 >= 0.04 {
            normalize_horizontal(movement.0, movement.1)
        } else {
            (state.combat_state.facing_x, state.combat_state.facing_z)
        };
        state.combat_state.dash_x = facing.0;
        state.combat_state.dash_z = facing.1;
        state.combat_state.facing_x = facing.0;
        state.combat_state.facing_z = facing.1;
    }
    state.combat_state.action_locks_facing = true;
    state.combat_state.qer_v1_active = true;
    state.combat_state.active_action = Some(ActiveCombatAction {
        kind,
        request_id,
        elapsed_ms: 0,
        impact_resolved: false,
        end_requested: false,
    });
    events.push(CombatEvent::ActionStarted {
        action: kind,
        request_id,
    });
}

fn advance_context_traversal(
    state: &mut WorldStateV3,
    kcc: &StaticKccWorld,
    dt_ms: u64,
    events: &mut Vec<CombatEvent>,
) {
    const DURATION_MS: u64 = 350;
    let Some(active) = state.combat_state.active_traversal.as_mut() else {
        return;
    };
    let Some(marker) = state
        .combat_state
        .traversal_markers
        .get(active.marker_index)
        .cloned()
    else {
        state.combat_state.active_traversal = None;
        return;
    };
    active.elapsed_ms = active.elapsed_ms.saturating_add(dt_ms).min(DURATION_MS);
    let progress = active.elapsed_ms as f32 / DURATION_MS as f32;
    let candidate = Vec3 {
        x_m: active.start_position.x_m + (marker.to.x_m - active.start_position.x_m) * progress,
        y_m: active.start_position.y_m + (marker.to.y_m - active.start_position.y_m) * progress,
        z_m: active.start_position.z_m + (marker.to.z_m - active.start_position.z_m) * progress,
    };
    let request_id = active.request_id;
    let complete = active.elapsed_ms >= DURATION_MS;
    let previous = state.player.position_m;
    let sweep_distance = horizontal_distance(previous, candidate);
    let sweep_steps = (sweep_distance / 0.05).ceil().max(1.0) as usize;
    let mut last_safe = previous;
    let mut blocked = false;
    for step in 1..=sweep_steps {
        let fraction = step as f32 / sweep_steps as f32;
        let sample = Vec3 {
            x_m: previous.x_m + (candidate.x_m - previous.x_m) * fraction,
            y_m: previous.y_m + (candidate.y_m - previous.y_m) * fraction,
            z_m: previous.z_m + (candidate.z_m - previous.z_m) * fraction,
        };
        if !kcc.can_occupy(sample, state.player.radius_m) {
            blocked = true;
            break;
        }
        last_safe = sample;
    }
    if blocked {
        state.player.position_m = last_safe;
        state.combat_state.active_traversal = None;
        events.push(CombatEvent::TraversalBlocked {
            request_id,
            marker_id: marker.id,
        });
        return;
    }
    state.player.position_m = candidate;
    if complete {
        state.combat_state.active_traversal = None;
        events.push(CombatEvent::TraversalCompleted {
            request_id,
            marker_id: marker.id,
        });
    }
}

fn advance_combat_action(
    state: &mut WorldStateV3, kcc: &StaticKccWorld, dt_ms: u64, events: &mut Vec<CombatEvent>,
    actor_events: &mut Vec<ActorRuntimeEvent>,
) {
    let Some(active) = state.combat_state.active_action.as_mut() else {
        return;
    };
    let spec = &combat_v1().actions[active.kind.data_key()];
    let previous = active.elapsed_ms;
    active.elapsed_ms = active.elapsed_ms.saturating_add(dt_ms);
    if active.kind == CombatActionKind::Guard
        && !active.end_requested
        && previous < spec.windup_ms
        && active.elapsed_ms >= spec.windup_ms
    {
        events.push(CombatEvent::GuardStarted {
            request_id: active.request_id,
        });
    }
    let impact_kind = matches!(
        active.kind,
        CombatActionKind::PrimaryAttack | CombatActionKind::Pulse | CombatActionKind::Pierce
    );
    let impact = impact_kind && !active.impact_resolved && active.elapsed_ms >= spec.windup_ms;
    if impact {
        active.impact_resolved = true;
    }
    if active.kind == CombatActionKind::Guard
        && !active.end_requested
        && previous < spec.windup_ms.saturating_add(spec.active_ms)
        && active.elapsed_ms >= spec.windup_ms.saturating_add(spec.active_ms)
    {
        active.end_requested = true;
        events.push(CombatEvent::GuardEnded {
            request_id: active.request_id,
        });
    }
    let total_ms = spec
        .windup_ms
        .saturating_add(spec.active_ms)
        .saturating_add(spec.recovery_ms);
    let finish = active.elapsed_ms >= total_ms;
    let kind = active.kind;
    let request_id = active.request_id;
    if finish {
        state.combat_state.active_action = None;
        state.combat_state.action_locks_facing = false;
    }
    if impact {
        resolve_action_impact(state, kcc, kind, request_id, events, actor_events);
    }
}

fn resolve_action_impact(
    state: &mut WorldStateV3,
    kcc: &StaticKccWorld,
    kind: CombatActionKind,
    request_id: u64,
    events: &mut Vec<CombatEvent>,
    actor_events: &mut Vec<ActorRuntimeEvent>,
) {
    let tuning = combat_v1();
    let spec: &CombatActionData = &tuning.actions[kind.data_key()];
    let (fx, fz) = (state.combat_state.facing_x, state.combat_state.facing_z);
    let origin = state.player.position_m;
    let mut targets = state
        .sentinels
        .iter()
        .filter(|sentinel| sentinel.active && crate::continuous_combat::combat_vertical_overlap(origin, sentinel.position_m))
        .filter_map(|sentinel| {
            let dx = sentinel.position_m.x_m - origin.x_m;
            let dz = sentinel.position_m.z_m - origin.z_m;
            let distance = (dx * dx + dz * dz).sqrt();
            let along = dx * fx + dz * fz;
            let lateral = (-fz * dx + fx * dz).abs();
            let hit = match spec.shape.as_str() {
                "cone" => {
                    distance <= spec.range_m
                        && distance > 0.0
                        && along / distance >= spec.min_facing_dot.unwrap_or(0.5)
                }
                "radial" => distance <= spec.range_m,
                "line" => {
                    along >= 0.0
                        && along <= spec.range_m
                        && lateral <= spec.line_half_width_m.unwrap_or(0.45)
                }
                _ => false,
            };
            hit.then(|| {
                (
                    if spec.shape == "line" {
                        along
                    } else {
                        distance
                    },
                    sentinel.entity_id.clone(), None, sentinel.position_m,
                )
            })
        })
        .collect::<Vec<_>>();
    targets.extend(
        state
            .generic_actors
            .iter()
            .filter(|actor| actor.hp > 0)
            .flat_map(|actor| actor.combat_points(origin, kcc).into_iter().filter_map(move |(member, point)| {
                let dx = point.x_m - origin.x_m;
                let dz = point.z_m - origin.z_m;
                let distance = (dx * dx + dz * dz).sqrt();
                let along = dx * fx + dz * fz;
                let lateral = (-fz * dx + fx * dz).abs();
                let hit = match spec.shape.as_str() {
                    "cone" => {
                        distance <= spec.range_m
                            && distance > 0.0
                            && along / distance >= spec.min_facing_dot.unwrap_or(0.5)
                    }
                    "radial" => distance <= spec.range_m,
                    "line" => {
                        along >= 0.0
                            && along <= spec.range_m
                            && lateral <= spec.line_half_width_m.unwrap_or(0.45)
                    }
                    _ => false,
                };
                hit.then(|| {
                    (
                        if spec.shape == "line" {
                            along
                        } else {
                            distance
                        },
                        actor.entity_id.clone(), member, point,
                    )
                })
            })),
    );
    targets.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1)).then_with(|| left.2.cmp(&right.2))
    });
    if targets.is_empty() {
        let contact = CombatContact::new(origin, origin, Some(request_id));
        if kind == CombatActionKind::PrimaryAttack {
            events.push(CombatEvent::AttackMiss { request_id, modern: true, contact: Some(contact) });
        } else {
            events.push(CombatEvent::ActionMiss { action: kind, request_id, contact });
        }
        return;
    }
    if kind == CombatActionKind::PrimaryAttack {
        if let Some((_, target_id, _, position)) = targets.first() {
            events.push(CombatEvent::AttackHit {
                request_id,
                target_id: target_id.clone(),
                damage: spec.damage,
                modern: true,
                contact: Some(CombatContact::new(origin, *position, Some(request_id))),
            });
        }
    }
    let targets = if kind == CombatActionKind::PrimaryAttack {
        targets.into_iter().take(1).collect::<Vec<_>>()
    } else {
        targets
    };
    let mut visited = std::collections::BTreeSet::new();
    for (_, target_id, _, position) in &targets {
        if !visited.insert(target_id.clone()) { continue; }
        let target_id = target_id.clone();
        if let Some(target) = state
            .sentinels
            .iter_mut()
            .find(|sentinel| sentinel.entity_id == target_id && sentinel.active)
        {
            target.take_damage_with_stagger(
                spec.damage,
                u64::from(spec.stagger).saturating_mul(tuning.stagger_ms_per_point),
            );
            events.push(CombatEvent::ActionImpact {
                action: kind,
                target_id,
                damage: spec.damage,
                stagger: spec.stagger,
                contact: Some(CombatContact::new(origin, *position, Some(request_id))),
            });
        } else if let Some(target) = state
            .generic_actors
            .iter_mut()
            .find(|actor| actor.entity_id == target_id && actor.hp > 0)
        {
            let damage = if target.group.is_some() {
                let members = targets.iter().filter(|(_, id, _, _)| id == &target_id)
                    .filter_map(|(_, _, member, _)| *member).collect::<Vec<_>>();
                let before = target.hp;
                match target.take_member_hits(&members, spec.damage, spec.stagger) {
                    Ok(hit_events) => actor_events.extend(hit_events), Err(_) => continue,
                }
                before - target.hp
            } else {
                actor_events.extend(target.take_damage_with_stagger(spec.damage, spec.stagger));
                spec.damage
            };
            events.push(CombatEvent::ActionImpact {
                action: kind,
                target_id,
                damage,
                stagger: spec.stagger,
                contact: Some(CombatContact::new(origin, *position, Some(request_id))),
            });
        }
    }
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
    InputNonFinite,
    InputAxisOutOfRange,
    InvalidHp,
    InvalidDelta,
    WrongWorld,
    InvalidEffect,
    UnknownSentinel,
    DeadSentinel,
    StaleInput,
    RevisionExhausted,
}
