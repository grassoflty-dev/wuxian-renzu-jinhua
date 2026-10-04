//! Server-side SceneDefinition validation and in-memory scene progression.

use crate::{
    effects::TerrainTag,
    scene_registry::{valid_id, WorldRegistry, RETURN_STATION_ID, WORLD_IDS},
    world_persistent_v1::{
        DROWNED_QUAY_WATER_ID, PUMP_CONTROL_ID, PUMP_CONTROL_POSITION_M, PUMP_EAST_WATER_ID,
    },
    world_v3::{InteractableView, Transform, Vec3},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct SceneDefinition {
    /// Established by WorldRegistry from the exact source bytes; never admitted
    /// from JSON or stored in a save.
    #[serde(skip)]
    pub(crate) verified_source_sha256:String,
    pub schema_version: u32,
    pub world_id: String,
    pub scene_id: String,
    pub bounds_m: BoundsM,
    #[serde(default)]
    pub presentation: PresentationDefinition,
    #[serde(default)]
    pub collision: Vec<CollisionDefinition>,
    #[serde(default)]
    pub navigation: NavigationDefinition,
    #[serde(default)]
    pub spawns: Vec<SpawnDefinition>,
    #[serde(default)]
    pub interactions: Vec<InteractionDefinition>,
    #[serde(default)]
    pub interaction_aggregates: Vec<InteractionAggregateDefinition>,
    #[serde(default)]
    pub doors: Vec<DoorDefinition>,
    #[serde(default)]
    pub triggers: Vec<TriggerDefinition>,
    #[serde(default)]
    pub hazards: Vec<HazardDefinition>,
    #[serde(default)]
    pub walkable_polygons: Vec<ScenePolygon>,
    #[serde(default)]
    pub exploration_regions: Vec<ScenePolygon>,
    #[serde(default)]
    pub terrain_regions: Vec<TerrainRegionDefinition>,
    #[serde(default)]
    pub moving_supports: Vec<crate::moving_support::MovingSupportDefinition>,
    #[serde(default)]
    pub standing_decks: Vec<crate::moving_support::StandingDeckDefinition>,
    #[serde(default)]
    pub checkpoints: Vec<CheckpointDefinition>,
    #[serde(default)]
    pub objectives: Vec<ObjectiveDefinition>,
    #[serde(default)]
    pub transitions: Vec<TransitionDefinition>,
    #[serde(default)]
    pub camera_zones: Vec<CameraZoneDefinition>,
    #[serde(default)]
    pub logic: LogicDefinition,
    #[serde(default)]
    pub occluders: Vec<ScenePolygon>,
    #[serde(default)]
    pub vfx_markers: Vec<AssetPosition>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundsM {
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub depth: f32,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationDefinition {
    #[serde(default)]
    pub background_asset: Option<String>,
    #[serde(default)]
    pub layers: Vec<PresentationLayer>,
    #[serde(default)]
    pub sprites: Vec<AssetPosition>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationLayer {
    pub id: String,
    pub z_group: i32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
    pub id: String,
    pub asset_id: String,
    pub position: [f32; 3],
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePolygon {
    pub id: String,
    pub polygon: Vec<[f32; 2]>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollisionDefinition {
    pub id: String,
    pub polygon: Vec<[f32; 2]>,
    #[serde(default)]
    pub requires_event: Option<String>,
    #[serde(default)]
    pub requires_actor_first_kill: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationDefinition {
    #[serde(default)]
    pub nodes: Vec<NavigationNode>,
    #[serde(default)]
    pub links: Vec<NavigationLink>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationNode {
    pub id: String,
    pub position: [f32; 3],
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationLink {
    pub from: String,
    pub to: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnDefinition {
    pub id: String,
    pub kind: String,
    pub entity_type: Option<String>,
    pub position: [f32; 3],
    pub nav_node: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionDefinition {
    #[serde(default)]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    #[serde(default)]
    pub environment_control: Option<crate::environment_hazards::EnvironmentControlConfig>,
    pub id: String,
    pub kind: String,
    pub event: Option<String>,
    pub position: [f32; 3],
    #[serde(default)]
    pub range_m: Option<f32>,
    #[serde(default)]
    pub cooldown_ms: Option<u64>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionAggregateDefinition {
    pub marker_id: String,
    pub member_ids: Vec<String>,
    pub event: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoorDefinition {
    #[serde(default)]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    pub id: String,
    #[serde(default)]
    pub to_scene_id: Option<String>,
    #[serde(default)]
    pub spawn_id: Option<String>,
    pub position: [f32; 3],
    pub asset_id: String,
    #[serde(default)]
    pub requires_event: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerDefinition {
    #[serde(default)]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    pub id: String,
    pub event: String,
    pub polygon: Vec<[f32; 2]>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HazardDefinition {
    #[serde(default)]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    #[serde(default)]
    pub environment: Option<crate::environment_hazards::EnvironmentHazardConfig>,
    pub id: String,
    pub kind: String,
    pub polygon: Vec<[f32; 2]>,
    #[serde(default)]
    pub damage: Option<u32>,
    #[serde(default)]
    pub period_ms: Option<u64>,
    #[serde(default)]
    pub warning_ms: Option<u64>,
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub translation_m: Option<[f32; 2]>,
    #[serde(default)]
    pub speed_mps: Option<f32>,
    #[serde(default)]
    pub endpoint_hold_ms: Option<u64>,
}
impl HazardDefinition {
    pub fn height_allows(&self,height:f32)->bool {
        self.height_range_m.map_or_else(||crate::continuous_combat::floor_height_overlap(height),|range|range.contains(height))
    }
    pub fn presentation_height(&self)->f32 {self.height_range_m.map_or(0.0,|range|range.midpoint())}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerrainRegionDefinition {
    pub id: String,
    pub tag: TerrainTag,
    pub polygon: Vec<[f32; 2]>,
    /// Authored horizontal support velocity; not player-owned save authority.
    #[serde(default)]
    pub surface_velocity_mps: Option<[f32; 2]>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointDefinition {
    #[serde(default)]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    pub id: String,
    pub position: [f32; 3],
    pub nav_node: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveDefinition {
    pub id: String,
    pub completing_event: String,
    pub position: [f32; 3],
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDefinition {
    #[serde(default)]
    pub height_range_m: Option<crate::moving_support::HeightRange>,
    pub id: String,
    pub to_scene_id: String,
    pub spawn_id: String,
    pub polygon: Vec<[f32; 2]>,
    #[serde(default)]
    pub requires_event: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraZoneDefinition {
    pub id: String,
    pub polygon: Vec<[f32; 2]>,
    pub profile: String,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogicDefinition {
    #[serde(default)]
    pub traversal: Vec<TraversalDefinition>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraversalDefinition {
    #[serde(default)]
    pub from_height_range_m: Option<crate::moving_support::HeightRange>,
    pub id: String,
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub range_m: f32,
    pub cooldown_ms: u64,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneRuntimeError {
    MalformedSceneDefinition,
    InvalidProgressionCatalog,
    InvalidGreyHiveSlice,
    InvalidNativeSceneSet,
    StaticMarker,
    EmptyRegistry,
    MissingReturnStation,
    MissingWorld(String),
    MissingRequiredEvent(String),
    DuplicateSceneId,
    DuplicateObjectId,
    UnknownWorld(String),
    InvalidBounds,
    InvalidPosition,
    InvalidHeightRange,
    MissingHeightRange,
    UnsupportedStandingPosition,
    InvalidNavigation,
    UnknownNavNode,
    InvalidPolygon,
    UnknownAsset(String),
    UnknownEntityType(String),
    InvalidSpawn,
    UnknownTransitionTarget(String),
    UnknownTransitionSpawn(String),
    UnknownEvent(String),
    UnknownScene(String),
    UnknownInteraction(String),
    UnknownTrigger(String),
    UnknownCheckpoint(String),
    RequestInvalid,
    DuplicateRequest,
    StaleEpoch,
    OutOfRange,
    AlreadyApplied,
    UnsafeTransition,
    ProgressionLocked,
}
impl std::fmt::Display for SceneRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E_SCENE_RUNTIME_{self:?}")
    }
}
impl std::error::Error for SceneRuntimeError {}

impl SceneDefinition {
    pub(crate) fn validate(
        &self,
        assets: &BTreeSet<String>,
        entities: &BTreeSet<String>,
        required: &BTreeMap<String, BTreeSet<String>>,
    ) -> Result<(), SceneRuntimeError> {
        if self.schema_version != 1 || !valid_id(&self.scene_id) || !valid_id(&self.world_id) {
            return Err(SceneRuntimeError::MalformedSceneDefinition);
        }
        if !WORLD_IDS.contains(&self.world_id.as_str()) && self.world_id != RETURN_STATION_ID {
            return Err(SceneRuntimeError::UnknownWorld(self.world_id.clone()));
        }
        let b = &self.bounds_m;
        if ![b.x, b.z, b.width, b.depth, b.x + b.width, b.z + b.depth]
            .iter()
            .all(|v| v.is_finite())
            || b.width <= 0.0
            || b.depth <= 0.0
            || b.x < 0.0
            || b.z < 0.0
        {
            return Err(SceneRuntimeError::InvalidBounds);
        }
        for collision in &self.collision {
            check_polygon(&collision.polygon, b)?;
            if let Some(event) = &collision.requires_event {
                check_required_event(event, &self.world_id, required)?;
            }
        }
        let mut ids = BTreeSet::new();
        macro_rules! unique {
            ($iter:expr) => {
                for id in $iter {
                    if !valid_id(id) || !ids.insert(id.to_string()) {
                        return Err(SceneRuntimeError::DuplicateObjectId);
                    }
                }
            };
        }
        unique!(self.collision.iter().map(|v| v.id.as_str()));
        unique!(self.navigation.nodes.iter().map(|v| v.id.as_str()));
        unique!(self.spawns.iter().map(|v| v.id.as_str()));
        unique!(self.interactions.iter().map(|v| v.id.as_str()));
        unique!(self.doors.iter().map(|v| v.id.as_str()));
        unique!(self.triggers.iter().map(|v| v.id.as_str()));
        let mut hazard_ids = BTreeSet::new();
        if self
            .hazards
            .iter()
            .any(|hazard| !valid_id(&hazard.id) || !hazard_ids.insert(hazard.id.as_str()))
        {
            return Err(SceneRuntimeError::DuplicateObjectId);
        }
        unique!(self.walkable_polygons.iter().map(|v| v.id.as_str()));
        unique!(self.exploration_regions.iter().map(|v| v.id.as_str()));
        unique!(self.terrain_regions.iter().map(|v| v.id.as_str()));
        unique!(self.moving_supports.iter().map(|v| v.id.as_str()));
        unique!(self.standing_decks.iter().map(|v| v.id.as_str()));
        unique!(self.checkpoints.iter().map(|v| v.id.as_str()));
        unique!(self.objectives.iter().map(|v| v.id.as_str()));
        unique!(self.transitions.iter().map(|v| v.id.as_str()));
        unique!(self.camera_zones.iter().map(|v| v.id.as_str()));
        unique!(self.logic.traversal.iter().map(|v| v.id.as_str()));
        unique!(self.presentation.sprites.iter().map(|v| v.id.as_str()));
        unique!(self.occluders.iter().map(|v| v.id.as_str()));
        unique!(self.vfx_markers.iter().map(|v| v.id.as_str()));
        unique!(self.presentation.layers.iter().map(|v| v.id.as_str()));
        for collision in &self.collision {
            if let Some(actor_id) = &collision.requires_actor_first_kill {
                if collision.requires_event.is_some()
                    || !self
                        .spawns
                        .iter()
                        .any(|spawn| spawn.id == *actor_id && spawn.kind == "enemy")
                {
                    return Err(SceneRuntimeError::MalformedSceneDefinition);
                }
            }
        }
        if self.world_id == "clockworks"
            && self.scene_id == "cw_forged_guard_arena"
            && self
                .collision
                .iter()
                .any(|collision| collision.requires_actor_first_kill.is_some())
        {
            let conditioned: Vec<_> = self
                .collision
                .iter()
                .filter(|collision| collision.requires_actor_first_kill.is_some())
                .collect();
            let mut ids: Vec<_> = conditioned
                .iter()
                .map(|collision| collision.id.as_str())
                .collect();
            ids.sort_unstable();
            let elite_spawns: Vec<_> = self
                .spawns
                .iter()
                .filter(|spawn| {
                    spawn.id == "cw_forged_guard_elite"
                        && spawn.kind == "enemy"
                        && spawn.entity_type.as_deref()
                            == Some("enemy.clockworks.forged_guard_elite")
                })
                .collect();
            if ids
                != [
                    "cw_arena_shutter_east_blocker",
                    "cw_arena_shutter_west_blocker",
                ]
                || conditioned.iter().any(|collision| {
                    collision.requires_actor_first_kill.as_deref() != Some("cw_forged_guard_elite")
                })
                || elite_spawns.len() != 1
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
        }
        let nodes: BTreeSet<_> = self
            .navigation
            .nodes
            .iter()
            .map(|n| n.id.as_str())
            .collect();
        if nodes.is_empty()
            || self
                .navigation
                .links
                .iter()
                .any(|l| !nodes.contains(l.from.as_str()) || !nodes.contains(l.to.as_str()))
        {
            return Err(SceneRuntimeError::InvalidNavigation);
        }
        for node in &self.navigation.nodes {
            check_position(node.position, b)?;
            if !scene_position_clear(node.position, 0.0, b, &self.collision) {
                return Err(SceneRuntimeError::InvalidNavigation);
            }
        }
        let player_spawns: Vec<_> = self.spawns.iter().filter(|s| s.kind == "player").collect();
        if player_spawns.len() != 1 {
            return Err(SceneRuntimeError::InvalidSpawn);
        }
        for spawn in &self.spawns {
            check_position(spawn.position, b)?;
            if !matches!(spawn.kind.as_str(), "player" | "enemy" | "npc") {
                return Err(SceneRuntimeError::InvalidSpawn);
            }
            if spawn.kind == "player"
                && !scene_position_clear(spawn.position, 0.35, b, &self.collision)
            {
                return Err(SceneRuntimeError::InvalidSpawn);
            }
            if spawn
                .entity_type
                .as_ref()
                .is_some_and(|entity| !entities.contains(entity))
            {
                return Err(SceneRuntimeError::UnknownEntityType(
                    spawn.entity_type.clone().unwrap_or_default(),
                ));
            }
            if matches!(spawn.kind.as_str(), "enemy" | "npc") {
                let entity = spawn
                    .entity_type
                    .as_ref()
                    .ok_or(SceneRuntimeError::InvalidSpawn)?;
                if !entities.contains(entity) {
                    return Err(SceneRuntimeError::UnknownEntityType(entity.clone()));
                }
            }
            if let Some(node) = &spawn.nav_node {
                if !nodes.contains(node.as_str()) {
                    return Err(SceneRuntimeError::UnknownNavNode);
                }
            }
        }
        let start = player_spawns[0]
            .nav_node
            .as_deref()
            .ok_or(SceneRuntimeError::InvalidSpawn)?;
        if !nodes.contains(start) {
            return Err(SceneRuntimeError::UnknownNavNode);
        }
        let mut reachable = BTreeSet::from([start]);
        loop {
            let prior = reachable.len();
            for link in &self.navigation.links {
                if reachable.contains(link.from.as_str()) {
                    reachable.insert(link.to.as_str());
                }
                if reachable.contains(link.to.as_str()) {
                    reachable.insert(link.from.as_str());
                }
            }
            if reachable.len() == prior {
                break;
            }
        }
        for checkpoint in &self.checkpoints {
            check_position(checkpoint.position, b)?;
            if !scene_position_clear(checkpoint.position, 0.35, b, &self.collision) {
                return Err(SceneRuntimeError::InvalidPosition);
            }
            if !reachable.contains(checkpoint.nav_node.as_str()) {
                return Err(SceneRuntimeError::UnknownNavNode);
            }
        }
        for i in &self.interactions {
            check_position(i.position, b)?;
            if i.range_m
                .is_some_and(|range| !range.is_finite() || range <= 0.0 || range > 2.5)
                || i.cooldown_ms.is_some_and(|cooldown| cooldown > 3_600_000)
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
            if let Some(event) = &i.event {
                check_event(event, &self.world_id, required)?;
            }
            if let Some(control) = &i.environment_control {
                control.validate().map_err(|_| SceneRuntimeError::MalformedSceneDefinition)?;
                if i.kind != "environment_control" || i.event.is_some()
                    || i.range_m.is_none()
                    || i.cooldown_ms.is_none_or(|cooldown| cooldown < control.suppression_ms || cooldown > 60_000)
                    || self.interaction_aggregates.iter().any(|aggregate| aggregate.marker_id == i.id || aggregate.member_ids.contains(&i.id))
                    || control.target_hazard_ids.iter().any(|id| !self.hazards.iter().any(|hazard| hazard.id == *id && hazard.environment.is_some())) {
                    return Err(SceneRuntimeError::MalformedSceneDefinition);
                }
            } else if i.kind == "environment_control" {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
            if i.id == "cw_regulator_valve_furnace_link_staged"
                && (self.world_id != "clockworks"
                    || self.scene_id != "cw_regulator_core"
                    || i.kind != "coolant_valve"
                    || i.position != [18.0, 0.0, 11.0]
                    || i.range_m != Some(2.5)
                    || i.cooldown_ms.is_none_or(|cooldown| cooldown < 8_000))
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
            if i.kind == "pump_control"
                && (self.world_id != "mist_harbor"
                    || self.scene_id != "mh_pump_station"
                    || i.id != PUMP_CONTROL_ID
                    || i.event.is_some()
                    || i.position != PUMP_CONTROL_POSITION_M)
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
        }
        let mut aggregate_markers = BTreeSet::new();
        let mut aggregate_members = BTreeSet::new();
        for aggregate in &self.interaction_aggregates {
            let Some(marker) = self
                .interactions
                .iter()
                .find(|item| item.id == aggregate.marker_id)
            else {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            };
            if !marker.kind.ends_with("_marker")
                || marker.event.is_some()
                || !aggregate_markers.insert(aggregate.marker_id.as_str())
                || aggregate.member_ids.is_empty()
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
            let mut members = BTreeSet::new();
            for member_id in &aggregate.member_ids {
                let Some(member) = self.interactions.iter().find(|item| item.id == *member_id)
                else {
                    return Err(SceneRuntimeError::UnknownInteraction(member_id.clone()));
                };
                if member_id == &aggregate.marker_id
                    || member.event.is_some()
                    || !members.insert(member_id.as_str())
                    || !aggregate_members.insert(member_id.as_str())
                {
                    return Err(SceneRuntimeError::MalformedSceneDefinition);
                }
            }
            check_required_event(&aggregate.event, &self.world_id, required)?;
        }
        for d in &self.doors {
            check_position(d.position, b)?;
            check_asset(&d.asset_id, assets)?;
            if d.to_scene_id.is_some() != d.spawn_id.is_some()
                || (d.to_scene_id.is_none() && d.requires_event.is_none())
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
            if let Some(event) = &d.requires_event {
                check_required_event(event, &self.world_id, required)?;
            }
        }
        for c in &self.collision {
            check_polygon(&c.polygon, b)?;
        }
        for t in &self.triggers {
            check_polygon(&t.polygon, b)?;
            check_event(&t.event, &self.world_id, required)?;
        }
        for t in &self.transitions {
            check_polygon(&t.polygon, b)?;
            if let Some(event) = &t.requires_event {
                check_required_event(event, &self.world_id, required)?;
            }
        }
        for camera in &self.camera_zones {
            check_polygon(&camera.polygon, b)?;
            if camera.profile.trim().is_empty() {
                return Err(SceneRuntimeError::InvalidPolygon);
            }
        }
        for h in &self.hazards {
            check_valid_region_polygon(&h.polygon, b)?;
            if let Some(environment) = &h.environment {
                environment.validate().map_err(|_| SceneRuntimeError::MalformedSceneDefinition)?;
                let tag_matches = matches!((h.kind.as_str(), environment.tag),
                    ("heat_zone", crate::effects::HazardTag::Heat)
                    | ("steam_jet", crate::effects::HazardTag::Pressure)
                    | ("decon_mist", crate::effects::HazardTag::Toxin));
                if !tag_matches || h.damage.is_some() || h.period_ms.is_some()
                    || h.warning_ms.is_some() || h.phase.is_some() || h.translation_m.is_some()
                    || h.speed_mps.is_some() || h.endpoint_hold_ms.is_some() {
                    return Err(SceneRuntimeError::MalformedSceneDefinition);
                }
                continue;
            }
            let moving = h.kind == "moving_machinery";
            let has_motion =
                h.translation_m.is_some() || h.speed_mps.is_some() || h.endpoint_hold_ms.is_some();
            let tuned = h.damage.is_some()
                || h.period_ms.is_some()
                || h.warning_ms.is_some()
                || h.phase.is_some()
                || has_motion
                || moving;
            if tuned
                && ((!moving && (h.kind != "heat_zone" || has_motion))
                    || h.damage.is_none_or(|damage| damage == 0 || damage > 10_000)
                    || h.period_ms
                        .is_none_or(|period| !(100..=60_000).contains(&period))
                    || h.warning_ms
                        .is_none_or(|warning| !(1..=10_000).contains(&warning))
                    || h.phase.as_deref() != Some(if moving { "phase3" } else { "phase2" }))
            {
                return Err(SceneRuntimeError::MalformedSceneDefinition);
            }
            if moving {
                let translation = h
                    .translation_m
                    .ok_or(SceneRuntimeError::MalformedSceneDefinition)?;
                let distance = translation[0].hypot(translation[1]);
                if !translation.iter().all(|value| value.is_finite())
                    || !distance.is_finite()
                    || distance < 0.01
                    || h.speed_mps
                        .is_none_or(|speed| !speed.is_finite() || !(0.1..=20.0).contains(&speed))
                    || h.endpoint_hold_ms
                        .is_none_or(|hold| !(100..=10_000).contains(&hold))
                {
                    return Err(SceneRuntimeError::MalformedSceneDefinition);
                }
                // Linear translations of an in-bounds polygon remain in the
                // rectangular scene iff the translated endpoint is in bounds.
                let endpoint: Vec<_> = h
                    .polygon
                    .iter()
                    .map(|point| [point[0] + translation[0], point[1] + translation[1]])
                    .collect();
                check_valid_region_polygon(&endpoint, b)?;
            }
            if self.world_id == "clockworks"
                && self.scene_id == "cw_regulator_core"
                && h.id == "cw_regulator_furnace_heat_zone"
            {
                if h.kind != "heat_zone"
                    || h.damage != Some(12)
                    || h.period_ms != Some(1_000)
                    || h.warning_ms != Some(600)
                    || h.phase.as_deref() != Some("phase2")
                {
                    return Err(SceneRuntimeError::MalformedSceneDefinition);
                }
                let spawn = self
                    .spawns
                    .iter()
                    .find(|spawn| spawn.kind == "player")
                    .ok_or(SceneRuntimeError::InvalidSpawn)?;
                if point_in_polygon(spawn.position[0], spawn.position[2], &h.polygon)
                    || h.polygon.iter().any(|point| point[1] >= 7.0)
                {
                    return Err(SceneRuntimeError::InvalidPolygon);
                }
            }
        }
        for region in &self.walkable_polygons {
            check_valid_region_polygon(&region.polygon, b)?;
        }
        if self.world_id == "mist_harbor" && !self.walkable_polygons.is_empty() {
            if self.walkable_polygons.len() != 1
                || b.width != 24.0
                || b.depth != 16.0
                || self.walkable_polygons[0].polygon
                    != [[0.5, 0.5], [23.5, 0.5], [23.5, 15.5], [0.5, 15.5]]
            {
                return Err(SceneRuntimeError::InvalidPolygon);
            }
        }
        for region in &self.exploration_regions {
            check_valid_region_polygon(&region.polygon, b)?;
        }
        crate::moving_support::validate_support_catalog(&self.moving_supports,&self.standing_decks,
            crate::continuous_kcc::Aabb::new(b.x,b.x+b.width,b.z,b.z+b.depth)
                .map_err(|_|SceneRuntimeError::InvalidBounds)?)
            .map_err(|_|SceneRuntimeError::InvalidPolygon)?;
        // New raised-deck scenes must author every actionable height interval.
        // Existing flat and moving-only content keeps its prior default contract.
        let validate_height=|range:Option<crate::moving_support::HeightRange>,anchor:Option<[f32;3]>|->Result<(),SceneRuntimeError> {
            if let Some(range)=range {
                range.validate().map_err(|_|SceneRuntimeError::InvalidHeightRange)?;
                if anchor.is_some_and(|point|!range.contains(point[1])) {return Err(SceneRuntimeError::InvalidHeightRange);}
            } else if !self.standing_decks.is_empty() {return Err(SceneRuntimeError::MissingHeightRange);}
            if let Some(point)=anchor {
                if point[1]>0.001 && !self.standing_position_valid(point,0.35) {return Err(SceneRuntimeError::UnsupportedStandingPosition);}
            }
            Ok(())
        };
        for item in &self.interactions {validate_height(item.height_range_m,Some(item.position))?;}
        for item in &self.doors {validate_height(item.height_range_m,Some(item.position))?;}
        for item in &self.checkpoints {validate_height(item.height_range_m,Some(item.position))?;}
        for item in &self.transitions {validate_height(item.height_range_m,None)?;}
        for item in &self.triggers {validate_height(item.height_range_m,None)?;}
        for item in &self.hazards {validate_height(item.height_range_m,None)?;}
        for item in &self.logic.traversal {validate_height(item.from_height_range_m,Some(item.from))?;}
        for point in self.spawns.iter().map(|v|v.position).chain(self.navigation.nodes.iter().map(|v|v.position)) {
            if point[1]>0.001 && !self.standing_position_valid(point,0.35) {return Err(SceneRuntimeError::UnsupportedStandingPosition);}
        }

        for region in &self.terrain_regions {
            check_valid_region_polygon(&region.polygon, b)?;
            if region.tag == TerrainTag::Conveyor {
                let velocity = region.surface_velocity_mps
                    .ok_or(SceneRuntimeError::InvalidPolygon)?;
                let speed = velocity[0].hypot(velocity[1]);
                if !velocity.iter().all(|value| value.is_finite()) || !(0.1..=4.0).contains(&speed) {
                    return Err(SceneRuntimeError::InvalidPolygon);
                }
                continue;
            }
            if region.surface_velocity_mps.is_some() {
                return Err(SceneRuntimeError::InvalidPolygon);
            }
            let expected = match region.tag {
                TerrainTag::WaterDeep => (self.world_id == "mist_harbor"
                    && self.scene_id == "mh_pump_station"
                    && region.id == PUMP_EAST_WATER_ID)
                    .then_some(vec![[18.0, 5.5], [23.5, 5.5], [23.5, 10.5], [18.0, 10.5]]),
                TerrainTag::WaterShallow => (self.world_id == "mist_harbor"
                    && self.scene_id == "mh_drowned_quay"
                    && region.id == DROWNED_QUAY_WATER_ID)
                    .then_some(vec![[13.0, 5.0], [19.0, 5.0], [19.0, 11.0], [13.0, 11.0]]),
                _ => None,
            };
            if expected.as_deref() != Some(region.polygon.as_slice()) {
                return Err(SceneRuntimeError::InvalidPolygon);
            }
        }
        for o in &self.objectives {
            check_position(o.position, b)?;
            check_event(&o.completing_event, &self.world_id, required)?;
        }
        for a in &self.presentation.sprites {
            check_position(a.position, b)?;
            check_asset(&a.asset_id, assets)?;
        }
        if let Some(asset_id) = &self.presentation.background_asset {
            check_asset(asset_id, assets)?;
        }
        for a in &self.vfx_markers {
            check_position(a.position, b)?;
            check_asset(&a.asset_id, assets)?;
        }
        for a in &self.occluders {
            check_polygon(&a.polygon, b)?;
        }
        let known_capabilities = [
            "information.local_map_i",
            "perception.rear_view_i",
            "information.enemy_vitals_basic",
            "body.regeneration_i",
            "mobility.air_step_i",
        ];
        for marker in &self.logic.traversal {
            check_position(marker.from, b)?;
            check_position(marker.to, b)?;
            if !self.standing_position_valid(marker.from,0.35) || !self.standing_position_valid(marker.to,0.35) {
                return Err(SceneRuntimeError::UnsupportedStandingPosition);
            }
            if !marker.range_m.is_finite()
                || (marker.range_m - 1.2).abs() > f32::EPSILON
                || marker.cooldown_ms != 350
            {
                return Err(SceneRuntimeError::InvalidPosition);
            }
            let mut caps = BTreeSet::new();
            for capability in &marker.required_capabilities {
                if !known_capabilities.contains(&capability.as_str()) || !caps.insert(capability) {
                    return Err(SceneRuntimeError::UnknownEvent(capability.clone()));
                }
            }
            let distance = ((marker.from[0] - marker.to[0]).powi(2)
                + (marker.from[2] - marker.to[2]).powi(2))
            .sqrt();
            let steps = (distance / 0.1).ceil().max(1.0) as usize;
            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let x = marker.from[0] + (marker.to[0] - marker.from[0]) * t;
                let z = marker.from[2] + (marker.to[2] - marker.from[2]) * t;
                let y=marker.from[1]+(marker.to[1]-marker.from[1])*t;
                if !scene_position_clear([x, y, z], 0.35, b, &self.collision) {
                    return Err(SceneRuntimeError::InvalidPosition);
                }
            }
        }
        Ok(())
    }

    pub fn standing_position_valid(&self,position:[f32;3],radius:f32)->bool {
        scene_position_clear(position,radius,&self.bounds_m,&self.collision)
            && (position[1].abs()<=0.001 || self.standing_decks.iter().any(|deck|
                (deck.height_m-position[1]).abs()<=0.001 && deck.supports(vec3(position),radius)))
    }
    pub fn height_allows(&self,id:&str,height:f32)->bool {
        let range=self.interactions.iter().find(|v|v.id==id).and_then(|v|v.height_range_m)
            .or_else(||self.doors.iter().find(|v|v.id==id).and_then(|v|v.height_range_m))
            .or_else(||self.checkpoints.iter().find(|v|v.id==id).and_then(|v|v.height_range_m))
            .or_else(||self.transitions.iter().find(|v|v.id==id).and_then(|v|v.height_range_m))
            .or_else(||self.triggers.iter().find(|v|v.id==id).and_then(|v|v.height_range_m))
            .or_else(||self.logic.traversal.iter().find(|v|v.id==id).and_then(|v|v.from_height_range_m));
        range.is_none_or(|range|range.contains(height))
    }

    pub(crate) fn emitted_event_ids(&self) -> BTreeSet<String> {
        self.interactions
            .iter()
            .filter_map(|i| i.event.clone())
            .chain(
                self.interaction_aggregates
                    .iter()
                    .map(|aggregate| aggregate.event.clone()),
            )
            .chain(self.triggers.iter().map(|t| t.event.clone()))
            .collect()
    }
    pub(crate) fn transition_targets(&self) -> Vec<TransitionTarget> {
        self.transitions
            .iter()
            .map(|v| TransitionTarget {
                scene_id: v.to_scene_id.clone(),
                spawn_id: v.spawn_id.clone(),
            })
            .chain(self.doors.iter().filter_map(|v| {
                Some(TransitionTarget {
                    scene_id: v.to_scene_id.clone()?,
                    spawn_id: v.spawn_id.clone()?,
                })
            }))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TransitionTarget {
    pub scene_id: String,
    pub spawn_id: String,
}

fn check_event(
    event: &str,
    world_id: &str,
    required: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), SceneRuntimeError> {
    if !valid_id(event) {
        return Err(SceneRuntimeError::UnknownEvent(event.into()));
    }
    if let Some(owner) = required
        .iter()
        .find_map(|(world, ids)| ids.contains(event).then_some(world.as_str()))
    {
        if owner != world_id {
            return Err(SceneRuntimeError::UnknownEvent(event.into()));
        }
    }
    Ok(())
}
fn check_required_event(
    event: &str,
    world_id: &str,
    required: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), SceneRuntimeError> {
    check_event(event, world_id, required)?;
    if required
        .get(world_id)
        .is_some_and(|events| events.contains(event))
    {
        Ok(())
    } else {
        Err(SceneRuntimeError::UnknownEvent(event.into()))
    }
}
fn check_asset(id: &str, assets: &BTreeSet<String>) -> Result<(), SceneRuntimeError> {
    if !assets.contains(id) {
        Err(SceneRuntimeError::UnknownAsset(id.into()))
    } else {
        Ok(())
    }
}
fn check_position(p: [f32; 3], b: &BoundsM) -> Result<(), SceneRuntimeError> {
    if p.iter().all(|v| v.is_finite())
        && p[0] >= b.x
        && p[0] <= b.x + b.width
        && p[2] >= b.z
        && p[2] <= b.z + b.depth
        && p[1] >= 0.0
        && p[1] <= 3.0
    {
        Ok(())
    } else {
        Err(SceneRuntimeError::InvalidPosition)
    }
}
fn check_polygon(p: &[[f32; 2]], b: &BoundsM) -> Result<(), SceneRuntimeError> {
    if p.len() < 3
        || polygon_area(p) <= 1e-9
        || p.iter().any(|v| {
            !v.iter().all(|x| x.is_finite())
                || v[0] < b.x
                || v[0] > b.x + b.width
                || v[1] < b.z
                || v[1] > b.z + b.depth
        })
    {
        Err(SceneRuntimeError::InvalidPolygon)
    } else {
        Ok(())
    }
}
fn check_valid_region_polygon(
    polygon: &[[f32; 2]],
    bounds: &BoundsM,
) -> Result<(), SceneRuntimeError> {
    check_polygon(polygon, bounds)?;
    if polygon.len() < 3
        || polygon.windows(2).any(|pair| pair[0] == pair[1])
        || (0..polygon.len()).any(|i| {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            (i + 1..polygon.len()).any(|j| {
                if j == i || j == (i + 1) % polygon.len() || (j + 1) % polygon.len() == i {
                    return false;
                }
                segments_intersect(a, b, polygon[j], polygon[(j + 1) % polygon.len()])
            })
        })
    {
        Err(SceneRuntimeError::InvalidPolygon)
    } else {
        Ok(())
    }
}
fn segments_intersect(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    fn orient(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    }
    fn on_segment(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> bool {
        p[0] >= a[0].min(b[0])
            && p[0] <= a[0].max(b[0])
            && p[1] >= a[1].min(b[1])
            && p[1] <= a[1].max(b[1])
    }
    let ab_c = orient(a, b, c);
    let ab_d = orient(a, b, d);
    let cd_a = orient(c, d, a);
    let cd_b = orient(c, d, b);
    (ab_c.signum() != ab_d.signum() && cd_a.signum() != cd_b.signum())
        || (ab_c == 0.0 && on_segment(a, b, c))
        || (ab_d == 0.0 && on_segment(a, b, d))
        || (cd_a == 0.0 && on_segment(c, d, a))
        || (cd_b == 0.0 && on_segment(c, d, b))
}
fn polygon_area(p: &[[f32; 2]]) -> f32 {
    p.iter()
        .enumerate()
        .map(|(i, a)| {
            let b = p[(i + 1) % p.len()];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f32>()
        .abs()
        * 0.5
}
fn scene_position_clear(
    position: [f32; 3],
    radius: f32,
    b: &BoundsM,
    collisions: &[CollisionDefinition],
) -> bool {
    position.iter().all(|n|n.is_finite()) && (0.0..=3.0).contains(&position[1])
        && position[0] >= b.x + radius
        && position[0] <= b.x + b.width - radius
        && position[2] >= b.z + radius
        && position[2] <= b.z + b.depth - radius
        && !collisions
            .iter()
            .any(|wall| point_hits_polygon(position[0], position[2], radius, &wall.polygon))
}
fn point_hits_polygon(x: f32, z: f32, radius: f32, p: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for i in 0..p.len() {
        let a = p[i];
        let b = p[(i + 1) % p.len()];
        if (a[1] > z) != (b[1] > z) && x < (b[0] - a[0]) * (z - a[1]) / (b[1] - a[1]) + a[0] {
            inside = !inside;
        }
        let dx = b[0] - a[0];
        let dz = b[1] - a[1];
        let denom = dx * dx + dz * dz;
        let t = if denom == 0.0 {
            0.0
        } else {
            (((x - a[0]) * dx + (z - a[1]) * dz) / denom).clamp(0.0, 1.0)
        };
        if (x - (a[0] + t * dx)).powi(2) + (z - (a[1] + t * dz)).powi(2) <= radius * radius {
            return true;
        }
    }
    inside
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SceneEvent {
    Interaction {
        id: String,
        event_id: Option<String>,
    },
    Trigger {
        id: String,
        event_id: String,
    },
    Checkpoint {
        checkpoint_id: String,
    },
    Transition {
        from_scene_id: String,
        to_scene_id: String,
        spawn_id: String,
        world_epoch: u64,
    },
}

#[derive(Clone, Debug)]
pub struct SceneRuntime {
    registry: WorldRegistry,
    pub scene_id: String,
    pub checkpoint_id: Option<String>,
    pub world_epoch: u64,
    emitted: BTreeSet<String>,
    completed_objectives: BTreeSet<String>,
    activated: BTreeSet<String>,
    processed: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TransitionResult {
    pub target_world_id: String,
    pub target_scene_id: String,
    pub spawn_id: String,
    pub position: Vec3,
    pub next_epoch: u64,
    pub event: SceneEvent,
}
impl SceneRuntime {
    pub fn new(
        registry: WorldRegistry,
        scene_id: &str,
        world_epoch: u64,
    ) -> Result<Self, SceneRuntimeError> {
        if registry.scene(scene_id).is_none() {
            return Err(SceneRuntimeError::UnknownScene(scene_id.into()));
        }
        Ok(Self {
            registry,
            scene_id: scene_id.into(),
            checkpoint_id: None,
            world_epoch,
            emitted: BTreeSet::new(),
            completed_objectives: BTreeSet::new(),
            activated: BTreeSet::new(),
            processed: BTreeSet::new(),
        })
    }
    pub fn current_scene(&self) -> &SceneDefinition {
        self.registry
            .scene(&self.scene_id)
            .expect("scene remains registered")
    }
    pub(crate) fn is_complete_clockworks_production(&self) -> bool {
        self.registry.is_complete_clockworks_production()
    }
    pub fn is_required_event(&self, world_id: &str, event_id: &str) -> bool {
        self.registry.is_required_event(world_id, event_id)
    }
    fn begin_request(&mut self, request_id: &str, epoch: u64) -> Result<(), SceneRuntimeError> {
        if epoch != self.world_epoch {
            return Err(SceneRuntimeError::StaleEpoch);
        }
        if !valid_id(request_id) {
            return Err(SceneRuntimeError::RequestInvalid);
        }
        if !self.processed.insert(request_id.into()) {
            return Err(SceneRuntimeError::DuplicateRequest);
        }
        Ok(())
    }

    /// Claim a v3 scene command request without activating a content interaction.
    /// This shares the scene epoch and replay protection with authored commands.
    pub fn claim_command_request(
        &mut self,
        request_id: &str,
        epoch: u64,
    ) -> Result<(), SceneRuntimeError> {
        self.begin_request(request_id, epoch)
    }

    pub fn interact(
        &mut self,
        id: &str,
        request_id: &str,
        epoch: u64,
        position: Vec3,
    ) -> Result<Vec<SceneEvent>, SceneRuntimeError> {
        self.begin_request(request_id, epoch)?;
        let item = self
            .current_scene()
            .interactions
            .iter()
            .find(|v| v.id == id)
            .cloned()
            .ok_or(SceneRuntimeError::UnknownInteraction(id.into()))?;
        if self.scene_id == "rs_core_room" && item.kind.ends_with("_marker") {
            return Err(SceneRuntimeError::StaticMarker);
        }
        if self
            .current_scene()
            .interaction_aggregates
            .iter()
            .any(|aggregate| aggregate.marker_id == id)
        {
            return Err(SceneRuntimeError::StaticMarker);
        }
        if !self.current_scene().height_allows(id,position.y_m) || distance(position, item.position) > item.range_m.unwrap_or(2.5) {
            return Err(SceneRuntimeError::OutOfRange);
        }
        // World-persistent controls remain queryable after their scene-local
        // activation. Their authoritative state is held by the world runtime.
        if !matches!(item.kind.as_str(), "pump_control" | "coolant_valve" | "environment_control")
            && !self.activated.insert(id.into())
        {
            return Err(SceneRuntimeError::AlreadyApplied);
        }
        let aggregate_event = self
            .current_scene()
            .interaction_aggregates
            .iter()
            .find(|aggregate| {
                aggregate.member_ids.iter().any(|member_id| member_id == id)
                    && aggregate
                        .member_ids
                        .iter()
                        .all(|member_id| self.activated.contains(member_id))
                    && !self.emitted.contains(&aggregate.event)
            })
            .map(|aggregate| aggregate.event.clone());
        let event_id = item.event.clone().or(aggregate_event);
        let event = SceneEvent::Interaction {
            id: id.into(),
            event_id: event_id.clone(),
        };
        if let Some(event_id) = event_id {
            self.record_event(&event_id);
            self.refresh_objectives();
        }
        Ok(vec![event])
    }
    pub fn trigger(
        &mut self,
        id: &str,
        request_id: &str,
        epoch: u64,
        position: Vec3,
    ) -> Result<Vec<SceneEvent>, SceneRuntimeError> {
        self.begin_request(request_id, epoch)?;
        let item = self
            .current_scene()
            .triggers
            .iter()
            .find(|v| v.id == id)
            .cloned()
            .ok_or(SceneRuntimeError::UnknownTrigger(id.into()))?;
        if !self.current_scene().height_allows(id,position.y_m) || !point_in_polygon(position.x_m, position.z_m, &item.polygon) {
            return Err(SceneRuntimeError::OutOfRange);
        }
        if !self.activated.insert(id.into()) {
            return Err(SceneRuntimeError::AlreadyApplied);
        }
        self.record_event(&item.event);
        self.refresh_objectives();
        Ok(vec![SceneEvent::Trigger {
            id: id.into(),
            event_id: item.event,
        }])
    }
    pub fn checkpoint(
        &mut self,
        id: &str,
        request_id: &str,
        epoch: u64,
        position: Vec3,
    ) -> Result<SceneEvent, SceneRuntimeError> {
        self.begin_request(request_id, epoch)?;
        let item = self
            .current_scene()
            .checkpoints
            .iter()
            .find(|v| v.id == id)
            .ok_or(SceneRuntimeError::UnknownCheckpoint(id.into()))?;
        if !self.current_scene().height_allows(id,position.y_m) || distance(position, item.position) > 2.5 {
            return Err(SceneRuntimeError::OutOfRange);
        }
        if self.checkpoint_id.as_deref() == Some(id) {
            return Err(SceneRuntimeError::AlreadyApplied);
        }
        self.checkpoint_id = Some(id.into());
        Ok(SceneEvent::Checkpoint {
            checkpoint_id: id.into(),
        })
    }
    pub fn transition(
        &mut self,
        id: &str,
        request_id: &str,
        epoch: u64,
        position: Vec3,
    ) -> Result<TransitionResult, SceneRuntimeError> {
        let completed_events = self.emitted.clone();
        self.transition_with_progression(id, request_id, epoch, position, &completed_events)
    }

    pub fn transition_with_progression(
        &mut self,
        id: &str,
        request_id: &str,
        epoch: u64,
        position: Vec3,
        completed_events: &BTreeSet<String>,
    ) -> Result<TransitionResult, SceneRuntimeError> {
        self.begin_request(request_id, epoch)?;
        let scene = self.current_scene();
        if !scene.height_allows(id,position.y_m) {return Err(SceneRuntimeError::OutOfRange);}
        let target = scene
            .transitions
            .iter()
            .find(|v| v.id == id)
            .map(|v| {
                (
                    v.to_scene_id.clone(),
                    v.spawn_id.clone(),
                    Some(v.polygon.clone()),
                    None,
                    v.requires_event.clone(),
                )
            })
            .or_else(|| {
                scene.doors.iter().find(|v| v.id == id).and_then(|v| {
                    Some((
                        v.to_scene_id.clone()?,
                        v.spawn_id.clone()?,
                        None,
                        Some(v.position),
                        v.requires_event.clone(),
                    ))
                })
            })
            .ok_or(SceneRuntimeError::UnsafeTransition)?;
        if let Some(required_event) = target.4.as_deref() {
            if !completed_events.contains(required_event) {
                return Err(SceneRuntimeError::ProgressionLocked);
            }
        }
        if let Some(polygon) = &target.2 {
            if !point_in_polygon(position.x_m, position.z_m, polygon) {
                return Err(SceneRuntimeError::OutOfRange);
            }
        }
        if let Some(door_position) = target.3 {
            if distance(position, door_position) > 2.5 {
                return Err(SceneRuntimeError::OutOfRange);
            }
        }
        if self.activated.contains(id) {
            return Err(SceneRuntimeError::AlreadyApplied);
        }
        let target_scene = self
            .registry
            .scene(&target.0)
            .ok_or(SceneRuntimeError::UnknownTransitionTarget(target.0.clone()))?;
        let spawn = target_scene
            .spawns
            .iter()
            .find(|s| s.id == target.1)
            .ok_or(SceneRuntimeError::UnknownTransitionSpawn(target.1.clone()))?;
        let next_epoch = self
            .world_epoch
            .checked_add(1)
            .ok_or(SceneRuntimeError::StaleEpoch)?;
        let from = self.scene_id.clone();
        self.scene_id = target.0.clone();
        self.world_epoch = next_epoch;
        self.checkpoint_id = None;
        self.emitted.clear();
        self.completed_objectives.clear();
        self.activated.clear();
        Ok(TransitionResult {
            target_world_id: target_scene.world_id.clone(),
            target_scene_id: target.0,
            spawn_id: target.1,
            position: vec3(spawn.position),
            next_epoch,
            event: SceneEvent::Transition {
                from_scene_id: from,
                to_scene_id: self.scene_id.clone(),
                spawn_id: spawn.id.clone(),
                world_epoch: next_epoch,
            },
        })
    }
    /// Hydrate authority-owned source activations without interacting, claiming
    /// request IDs, or emitting gameplay events. Preserve all existing ledgers.
    pub(crate) fn restore_durable_interactions(
        &mut self,
        activated_ids: &[&str],
    ) -> Result<(), SceneRuntimeError> {
        if activated_ids.iter().any(|id| !self.current_scene().interactions.iter().any(|i| i.id == *id)) {
            return Err(SceneRuntimeError::MalformedSceneDefinition);
        }
        self.activated.extend(activated_ids.iter().map(|id| (*id).to_owned()));
        let definition = self.current_scene().clone();
        for item in &definition.interactions {
            if activated_ids.contains(&item.id.as_str()) {
                if let Some(event) = &item.event { self.emitted.insert(event.clone()); }
            }
        }
        for aggregate in &definition.interaction_aggregates {
            if aggregate.member_ids.iter().any(|id| activated_ids.contains(&id.as_str()))
                && aggregate.member_ids.iter().all(|id| self.activated.contains(id))
            {
                self.emitted.insert(aggregate.event.clone());
            }
        }
        self.refresh_objectives();
        Ok(())
    }

    pub fn objective_complete(&self, id: &str) -> bool {
        self.completed_objectives.contains(id)
    }
    pub fn object_activated(&self, id: &str) -> bool {
        self.activated.contains(id)
    }
    pub fn event_complete(&self, id: &str) -> bool {
        self.emitted.contains(id)
    }
    pub fn door_is_open(&self, id: &str, completed_events: &BTreeSet<String>) -> Option<bool> {
        self.current_scene()
            .doors
            .iter()
            .find(|door| door.id == id)
            .and_then(|door| door.requires_event.as_ref())
            .map(|event| completed_events.contains(event))
    }
    /// Project authored route affordances into the existing v3 interactable collection.
    /// `active` is true only where the same server-side spatial/progression checks used by
    /// the corresponding command would currently accept the request.
    pub fn route_interactables(
        &self,
        position: Vec3,
        completed_events: &BTreeSet<String>,
        commands_allowed: bool,
    ) -> Vec<InteractableView> {
        let scene = self.current_scene();
        let mut result = Vec::new();
        for transition in &scene.transitions {
            let active = commands_allowed
                && scene.height_allows(&transition.id,position.y_m)
                && !self.activated.contains(&transition.id)
                && transition
                    .requires_event
                    .as_ref()
                    .is_none_or(|event| completed_events.contains(event))
                && point_in_polygon(position.x_m, position.z_m, &transition.polygon);
            result.push(InteractableView {
                entity_id: transition.id.clone(),
                kind: "scene_transition".into(),
                transform: Transform {
                    position_m: {let mut p=polygon_center(&transition.polygon);p.y_m=transition.height_range_m.map_or(0.0,|r|r.midpoint());p},
                    yaw_rad: 0.0,
                },
                active,
            });
        }
        for checkpoint in &scene.checkpoints {
            result.push(InteractableView {
                entity_id: checkpoint.id.clone(),
                kind: "scene_checkpoint".into(),
                transform: Transform {
                    position_m: vec3(checkpoint.position),
                    yaw_rad: 0.0,
                },
                active: commands_allowed
                    && scene.height_allows(&checkpoint.id,position.y_m)
                    && self.checkpoint_id.as_deref() != Some(checkpoint.id.as_str())
                    && distance(position, checkpoint.position) <= 2.5,
            });
        }
        for trigger in &scene.triggers {
            result.push(InteractableView {
                entity_id: trigger.id.clone(),
                kind: "scene_trigger".into(),
                transform: Transform {
                    position_m: {let mut p=polygon_center(&trigger.polygon);p.y_m=trigger.height_range_m.map_or(0.0,|r|r.midpoint());p},
                    yaw_rad: 0.0,
                },
                active: commands_allowed
                    && scene.height_allows(&trigger.id,position.y_m)
                    && !self.activated.contains(&trigger.id)
                    && point_in_polygon(position.x_m, position.z_m, &trigger.polygon),
            });
        }
        result
    }
    pub fn objectives(&self) -> Vec<(String, String, [f32; 3])> {
        self.current_scene()
            .objectives
            .iter()
            .map(|o| {
                (
                    o.id.clone(),
                    if self.completed_objectives.contains(&o.id) {
                        "complete".into()
                    } else {
                        "active".into()
                    },
                    o.position,
                )
            })
            .collect()
    }
    pub fn traversals(&self) -> Vec<crate::continuous_combat::TraversalMarker> {
        self.current_scene()
            .logic
            .traversal
            .iter()
            .map(|t| crate::continuous_combat::TraversalMarker {
                id: t.id.clone(),
                from: vec3(t.from),
                to: vec3(t.to),
                range_m: t.range_m,
                cooldown_ms: t.cooldown_ms,
                required_capabilities: t.required_capabilities.clone(),
                from_height_range_m: t.from_height_range_m,
            })
            .collect()
    }
    fn record_event(&mut self, event: &str) {
        self.emitted.insert(event.into());
    }
    fn refresh_objectives(&mut self) {
        let completed: Vec<String> = self
            .current_scene()
            .objectives
            .iter()
            .filter(|objective| self.emitted.contains(&objective.completing_event))
            .map(|objective| objective.id.clone())
            .collect();
        self.completed_objectives.extend(completed);
    }
}
fn vec3(v: [f32; 3]) -> Vec3 {
    Vec3 {
        x_m: v[0],
        y_m: v[1],
        z_m: v[2],
    }
}
fn distance(p: Vec3, q: [f32; 3]) -> f32 {
    ((p.x_m - q[0]).powi(2) + (p.y_m - q[1]).powi(2) + (p.z_m - q[2]).powi(2)).sqrt()
}
fn polygon_center(polygon: &[[f32; 2]]) -> Vec3 {
    let n = polygon.len().max(1) as f32;
    Vec3 {
        x_m: polygon.iter().map(|p| p[0]).sum::<f32>() / n,
        y_m: 0.0,
        z_m: polygon.iter().map(|p| p[1]).sum::<f32>() / n,
    }
}
fn point_in_polygon(x: f32, z: f32, p: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for i in 0..p.len() {
        let a = p[i];
        let b = p[(i + 1) % p.len()];
        if (a[1] > z) != (b[1] > z) && x < (b[0] - a[0]) * (z - a[1]) / (b[1] - a[1]) + a[0] {
            inside = !inside;
        }
    }
    inside
}

#[cfg(test)]
mod machinery_schema_tests {
    use super::*;

    fn scene() -> SceneDefinition {
        serde_json::from_value(serde_json::json!({
            "schemaVersion": 1, "worldId": "clockworks", "sceneId": "cw_motion_schema_test",
            "boundsM": {"x": 0, "z": 0, "width": 12, "depth": 12},
            "navigation": {"nodes": [{"id": "entry_nav", "position": [1, 0, 1]}], "links": []},
            "spawns": [{"id": "entry", "kind": "player", "position": [1, 0, 1], "navNode": "entry_nav"}],
            "hazards": [{
                "id": "press", "kind": "moving_machinery",
                "polygon": [[2, 2], [3, 2], [3, 3], [2, 3]],
                "damage": 16, "periodMs": 750, "warningMs": 900, "phase": "phase3",
                "translationM": [6, 0], "speedMps": 2, "endpointHoldMs": 600
            }]
        }))
        .unwrap()
    }

    fn validate(scene: &SceneDefinition) -> Result<(), SceneRuntimeError> {
        scene.validate(&BTreeSet::new(), &BTreeSet::new(), &BTreeMap::new())
    }

    #[test]
    fn moving_machinery_accepts_generic_positive_negative_and_diagonal_motion() {
        let mut scene = scene();
        for translation in [[6.0, 0.0], [-2.0, -2.0], [0.0, 9.0], [1.0, 1.0]] {
            scene.hazards[0].translation_m = Some(translation);
            assert!(validate(&scene).is_ok(), "{translation:?}");
        }
    }

    #[test]
    fn moving_machinery_requires_complete_timing_motion_and_phase() {
        let original = scene();
        for missing in 0..8 {
            let mut scene = original.clone();
            let h = &mut scene.hazards[0];
            match missing {
                0 => h.damage = None,
                1 => h.period_ms = None,
                2 => h.warning_ms = None,
                3 => h.phase = None,
                4 => h.translation_m = None,
                5 => h.speed_mps = None,
                6 => h.endpoint_hold_ms = None,
                _ => h.phase = Some("phase2".into()),
            }
            assert!(matches!(
                validate(&scene),
                Err(SceneRuntimeError::MalformedSceneDefinition)
            ));
        }
    }

    #[test]
    fn moving_machinery_rejects_nonfinite_zero_or_out_of_bounds_motion() {
        for translation in [
            [f32::NAN, 0.0],
            [0.0, f32::INFINITY],
            [f32::NEG_INFINITY, 0.0],
            [0.0, 0.0],
            [0.001, 0.001],
        ] {
            let mut scene = scene();
            scene.hazards[0].translation_m = Some(translation);
            assert!(matches!(
                validate(&scene),
                Err(SceneRuntimeError::MalformedSceneDefinition)
            ));
        }
        for translation in [[9.01, 0.0], [-2.01, 0.0], [0.0, 9.01], [0.0, -2.01]] {
            let mut scene = scene();
            scene.hazards[0].translation_m = Some(translation);
            assert!(matches!(
                validate(&scene),
                Err(SceneRuntimeError::InvalidPolygon)
            ));
        }
        for speed in [f32::NAN, f32::INFINITY, 0.0, -1.0, 20.01] {
            let mut scene = scene();
            scene.hazards[0].speed_mps = Some(speed);
            assert!(matches!(
                validate(&scene),
                Err(SceneRuntimeError::MalformedSceneDefinition)
            ));
        }
        for hold in [0, 99, 10_001, u64::MAX] {
            let mut scene = scene();
            scene.hazards[0].endpoint_hold_ms = Some(hold);
            assert!(matches!(
                validate(&scene),
                Err(SceneRuntimeError::MalformedSceneDefinition)
            ));
        }
    }

    #[test]
    fn machinery_motion_is_rejected_on_other_hazard_kinds() {
        for kind in ["heat_zone", "unknown"] {
            let mut scene = scene();
            scene.hazards[0].kind = kind.into();
            scene.hazards[0].phase = Some("phase2".into());
            assert!(matches!(
                validate(&scene),
                Err(SceneRuntimeError::MalformedSceneDefinition)
            ));
        }
    }
}
