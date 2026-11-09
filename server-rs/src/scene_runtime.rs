//! Server-side SceneDefinition validation and in-memory scene progression.

use crate::{
    effects::TerrainTag,
    scene_registry::{valid_id, WorldRegistry, RETURN_STATION_ID, WORLD_IDS},
    save_v5::SceneStateSave,
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
        if !crate::formal_runtime::shaft_lift::source_support_valid(self) {
            crate::moving_support::validate_support_catalog(&self.moving_supports,&self.standing_decks,
                crate::continuous_kcc::Aabb::new(b.x,b.x+b.width,b.z,b.z+b.depth)
                    .map_err(|_|SceneRuntimeError::InvalidBounds)?)
                .map_err(|_|SceneRuntimeError::InvalidPolygon)?;
        }
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
    // A whole-entry legacy variant, never a second canonical registry entry.
    lockdown_legacy: Option<SceneDefinition>,
    utility_legacy: Option<SceneDefinition>,
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
            lockdown_legacy: None,
            utility_legacy: None,
            scene_id: scene_id.into(),
            checkpoint_id: None,
            world_epoch,
            emitted: BTreeSet::new(),
            completed_objectives: BTreeSet::new(),
            activated: BTreeSet::new(),
            processed: BTreeSet::new(),
        })
    }
    /// Read-only definitions for qualified map geometry; this does not prepare a transition.
    pub(crate) fn gh_map_scene(&self, id: &str) -> Option<&SceneDefinition> {
        let scene = if id == self.scene_id { self.current_scene() } else { self.registry.scene(id)? };
        (scene.world_id == "grey_hive").then_some(scene)
    }
    /// Immutable definitions for the two maintained Clockworks map endpoints.
    pub(crate) fn cw_height_map_scene(&self,id:&str)->Option<&SceneDefinition> {
        if !["cw_gear_shaft","cw_furnace_heart"].contains(&id) {return None;}
        let scene=if id==self.scene_id {self.current_scene()} else {self.registry.scene(id)?};
        (scene.world_id=="clockworks").then_some(scene)
    }
    pub fn current_scene(&self) -> &SceneDefinition {
        if let Some(scene) = &self.utility_legacy {
            if scene.scene_id == self.scene_id { return scene; }
        }
        if self.scene_id == "gh_lockdown" {
            if let Some(scene) = &self.lockdown_legacy { return scene; }
        }
        self.registry
            .scene(&self.scene_id)
            .expect("scene remains registered")
    }
    pub(crate) fn select_lockdown_legacy(&mut self) -> Result<(), SceneRuntimeError> {
        if self.scene_id != "gh_lockdown" || self.registry.scene("gh_lockdown")
            .map_or(true, |s| s.world_id != "grey_hive" || s.verified_source_sha256 != "23e5d5d9f37f6f44b1f8b9105aab656424778ef5edf0e775bbc364143fe1b30c") {
            return Err(SceneRuntimeError::MalformedSceneDefinition);
        }
        let raw = include_str!("../../support/legacy/gh_lockdown.json");
        let mut scene: SceneDefinition = serde_json::from_str(raw)
            .map_err(|_| SceneRuntimeError::MalformedSceneDefinition)?;
        if scene.scene_id != "gh_lockdown" || scene.world_id != "grey_hive" {
            return Err(SceneRuntimeError::MalformedSceneDefinition);
        }
        use sha2::Digest;
        scene.verified_source_sha256 = format!("{:x}", sha2::Sha256::digest(raw.as_bytes()));
        if scene.verified_source_sha256 != "718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850" {
            return Err(SceneRuntimeError::MalformedSceneDefinition);
        }
        self.lockdown_legacy = Some(scene);
        Ok(())
    }
    pub(crate) fn lockdown_legacy_definition(&self) -> Result<SceneDefinition, SceneRuntimeError> {
        let mut copy = self.clone();
        copy.select_lockdown_legacy()?;
        copy.lockdown_legacy.ok_or(SceneRuntimeError::MalformedSceneDefinition)
    }
    pub(crate) fn select_utility_legacy(&mut self) -> Result<(), SceneRuntimeError> {
        let (canonical,old,raw)=match self.scene_id.as_str() {
            "gh_bio_isolation" => ("bf531fd9d49ee875653a8048130dd0c6e1d9b2170c1848ad48c49f2cb35ce0db","a3d629f04dc955cf0f8e644b54a4a64d054411a7d59a9fb15da5c9d9ea953aff",include_str!("../../support/legacy/gh_bio_isolation.json")),
            "gh_deep_decon" => ("a53fd5c62a4540a375b51f7286798c7bab1a5c73aa37e98d3f4d6bad7b24f1c0","65e0cbf11183fe114259d680b0cd84a23a7f8619a25e628f4b30a7a84ce870a8",include_str!("../../support/legacy/gh_deep_decon.json")),
            "gh_beacon" => ("e6434c598904d7fe36f4f48ebc67559e6d484eb535393a7b447bc99cbf86c766","3cab424771d10799ee82a453a72117affb0ebca09dd814f25308be70310da9ee",include_str!("../../support/legacy/gh_beacon.json")),
            "gh_exit" => ("b32ad8d1204e9eaf45512b0e2d5ea22e08f5bd09ec0b5e31980e41336c45ee09","6ab262467e1992feac94f27d8bf571dcf77ddbc8c1693a79c57b3aeebff0c54c",include_str!("../../support/legacy/gh_exit.json")),
            _ => return Err(SceneRuntimeError::MalformedSceneDefinition),
        };
        if self.registry.scene(&self.scene_id).map_or(true,|s|s.world_id!="grey_hive" || s.verified_source_sha256!=canonical) {return Err(SceneRuntimeError::MalformedSceneDefinition);}
        let mut scene:SceneDefinition=serde_json::from_str(raw).map_err(|_|SceneRuntimeError::MalformedSceneDefinition)?;
        use sha2::Digest;
        scene.verified_source_sha256=format!("{:x}",sha2::Sha256::digest(raw.as_bytes()));
        if scene.scene_id!=self.scene_id || scene.world_id!="grey_hive" || scene.verified_source_sha256!=old {return Err(SceneRuntimeError::MalformedSceneDefinition);}
        self.utility_legacy=Some(scene);
        Ok(())
    }
    pub(crate) fn utility_legacy_definition(&self)->Result<SceneDefinition,SceneRuntimeError> {
        let mut copy=self.clone();copy.select_utility_legacy()?;
        copy.utility_legacy.ok_or(SceneRuntimeError::MalformedSceneDefinition)
    }
    pub(crate) fn same_floor_neighbor(&self) -> Option<&SceneDefinition> {
        if self.current_scene().world_id != "grey_hive" { return None; }
        let id = match self.scene_id.as_str() {
            "gh_power_room" => "gh_gate_a",
            "gh_gate_a" => "gh_power_room",
            _ => return None,
        };
        self.registry.scene(id)
    }
    pub(crate) fn shaft_definition(&self) -> Option<&SceneDefinition> {
        let shaft=self.registry.scene("gh_central_shaft")?;
        (shaft.world_id=="grey_hive").then_some(shaft)
    }
    /// The east Shaft portal is separate from Gate A's existing west neighbor.
    pub(crate) fn gate_shaft_upper_neighbor(&self) -> Option<&SceneDefinition> {
        if self.current_scene().world_id!="grey_hive" {return None;}
        let id=match self.scene_id.as_str() {
            "gh_gate_a"=>"gh_central_shaft",
            "gh_central_shaft"=>"gh_gate_a",
            _=>return None,
        };
        let neighbor=self.registry.scene(id)?;
        (neighbor.world_id=="grey_hive").then_some(neighbor)
    }
    pub(crate) fn gate_decon_neighbor(&self)->Option<&SceneDefinition> {
        if self.current_scene().world_id!="grey_hive" {return None;}
        let id=match self.scene_id.as_str(){"gh_gate_b"=>"gh_deep_decon","gh_deep_decon"=>"gh_gate_b",_=>return None};
        let neighbor=self.registry.scene(id)?;
        (neighbor.world_id=="grey_hive").then_some(neighbor)
    }
    /// Pair-local Beacon/Exit adjacency; the authored ordinary portals remain
    /// unchanged and continue to own their legacy spawn behavior.
    pub(crate) fn beacon_exit_neighbor(&self)->Option<&SceneDefinition> {
        if self.current_scene().world_id!="grey_hive" {return None;}
        let id=match self.scene_id.as_str(){"gh_beacon"=>"gh_exit","gh_exit"=>"gh_beacon",_=>return None};
        let neighbor=self.registry.scene(id)?;
        (neighbor.world_id=="grey_hive").then_some(neighbor)
    }
    /// Transaction for a swept, already-authorized physical seam crossing.
    /// The ordinary portal's interior trigger remains unchanged for fallback.
    pub(crate) fn beacon_exit_walk(&mut self,id:&str,request_id:&str,epoch:u64,position:Vec3)
        ->Result<TransitionResult,SceneRuntimeError> {
        self.begin_request(request_id,epoch)?;
        let (source,target_id,reverse,spawn_id)=match (self.scene_id.as_str(),id) {
            ("gh_beacon","gh_beacon_to_exit")=>("gh_beacon","gh_exit","gh_exit_to_beacon","gh_exit_spawn"),
            ("gh_exit","gh_exit_to_beacon")=>("gh_exit","gh_beacon","gh_beacon_to_exit","gh_beacon_spawn"),
            _=>return Err(SceneRuntimeError::UnsafeTransition),
        };
        if self.current_scene().world_id!="grey_hive" || !position.x_m.is_finite() || position.y_m!=0.0 || !position.z_m.is_finite() {return Err(SceneRuntimeError::OutOfRange);}
        let source_scene=self.current_scene();let target=self.registry.scene(target_id).ok_or_else(||SceneRuntimeError::UnknownTransitionTarget(target_id.into()))?;
        if target.world_id!="grey_hive" || source_scene.transitions.iter().filter(|t|t.id==id && t.to_scene_id==target_id && t.requires_event.is_none()).count()!=1
            || target.transitions.iter().filter(|t|t.id==reverse && t.to_scene_id==source && t.requires_event.is_none()).count()!=1 {return Err(SceneRuntimeError::UnsafeTransition);}
        let spawn=target.spawns.iter().find(|s|s.id==spawn_id).ok_or_else(||SceneRuntimeError::UnknownTransitionSpawn(spawn_id.into()))?;
        let spawn_position=vec3(spawn.position);let spawn_id=spawn.id.clone();
        let next_epoch=self.world_epoch.checked_add(1).ok_or(SceneRuntimeError::StaleEpoch)?;
        self.scene_id=target_id.into();self.world_epoch=next_epoch;self.checkpoint_id=None;
        self.emitted.clear();self.completed_objectives.clear();self.activated.clear();
        Ok(TransitionResult{target_world_id:"grey_hive".into(),target_scene_id:self.scene_id.clone(),spawn_id:spawn_id.clone(),position:spawn_position,next_epoch,
            event:SceneEvent::Transition{from_scene_id:source.into(),to_scene_id:self.scene_id.clone(),spawn_id,world_epoch:next_epoch}})
    }

    /// The first three same-floor Clockworks pairs only (not the Gear lift).
    pub(crate) fn cw_early_neighbor(&self,east:bool)->Option<&SceneDefinition> {
        if self.current_scene().world_id!="clockworks" {return None;}
        let target=match (self.scene_id.as_str(),east) {
            ("cw_entry_foundry",true)=>"cw_pressure_hall",("cw_pressure_hall",false)=>"cw_entry_foundry",
            ("cw_pressure_hall",true)=>"cw_conveyor_bridge",("cw_conveyor_bridge",false)=>"cw_pressure_hall",
            ("cw_conveyor_bridge",true)=>"cw_boiler_chamber",("cw_boiler_chamber",false)=>"cw_conveyor_bridge",
            ("cw_boiler_chamber",true)=>"cw_gear_shaft",("cw_gear_shaft",false)=>"cw_boiler_chamber",
            ("cw_furnace_heart",true)=>"cw_forged_guard_arena",("cw_forged_guard_arena",false)=>"cw_furnace_heart",
            ("cw_forged_guard_arena",true)=>"cw_regulator_core",("cw_regulator_core",false)=>"cw_forged_guard_arena",
            ("cw_regulator_core",true)=>"cw_shutdown_exit",("cw_shutdown_exit",false)=>"cw_regulator_core",
            _=>return None,
        };
        self.registry.scene(target).filter(|scene|scene.world_id=="clockworks")
    }
    pub(crate) fn cw_early_candidate(&self,target:&str)->Option<Self> {
        if ![false,true].into_iter().any(|east|self.cw_early_neighbor(east).is_some_and(|s|s.scene_id==target)) {return None;}
        let mut candidate=self.clone();candidate.scene_id=target.into();Some(candidate)
    }
    pub(crate) fn cw_early_connection_verified(&self,target_id:&str,events:&BTreeSet<String>,elite_first_kill:bool)->bool {
        let source=self.scene_id.as_str();
        let (east,forward,reverse,fg,rg)=match (source,target_id) {
            ("cw_entry_foundry","cw_pressure_hall")=>(true,"cw_to_pressure_hall","cw_return_to_entry_foundry",None,None),
            ("cw_pressure_hall","cw_entry_foundry")=>(false,"cw_return_to_entry_foundry","cw_to_pressure_hall",None,None),
            ("cw_pressure_hall","cw_conveyor_bridge")=>(true,"cw_to_conveyor_bridge","cw_bridge_to_pressure_hall",Some("clockworks_valves"),None),
            ("cw_conveyor_bridge","cw_pressure_hall")=>(false,"cw_bridge_to_pressure_hall","cw_to_conveyor_bridge",None,Some("clockworks_valves")),
            ("cw_conveyor_bridge","cw_boiler_chamber")=>(true,"cw_bridge_to_boiler_chamber","cw_boiler_return_to_conveyor_bridge",None,None),
            ("cw_boiler_chamber","cw_conveyor_bridge")=>(false,"cw_boiler_return_to_conveyor_bridge","cw_bridge_to_boiler_chamber",None,None),
            ("cw_boiler_chamber","cw_gear_shaft")=>(true,"cw_boiler_to_gear_shaft","cw_gear_return_to_boiler",None,None),
            ("cw_gear_shaft","cw_boiler_chamber")=>(false,"cw_gear_return_to_boiler","cw_boiler_to_gear_shaft",None,None),
            ("cw_furnace_heart","cw_forged_guard_arena")=>(true,"cw_furnace_heart_to_arena","cw_arena_return_to_furnace_heart",None,None),
            ("cw_forged_guard_arena","cw_furnace_heart")=>(false,"cw_arena_return_to_furnace_heart","cw_furnace_heart_to_arena",None,None),
            ("cw_forged_guard_arena","cw_regulator_core")=>(true,"cw_arena_to_regulator_core","cw_regulator_core_return_to_arena",None,None),
            ("cw_regulator_core","cw_forged_guard_arena")=>(false,"cw_regulator_core_return_to_arena","cw_arena_to_regulator_core",None,None),
            ("cw_regulator_core","cw_shutdown_exit")=>(true,"cw_regulator_core_to_shutdown_exit","cw_shutdown_exit_return_to_regulator_core",Some("clockworks_core"),None),
            ("cw_shutdown_exit","cw_regulator_core")=>(false,"cw_shutdown_exit_return_to_regulator_core","cw_regulator_core_to_shutdown_exit",None,Some("clockworks_core")),
            _=>return false,
        };
        if fg.is_some_and(|event|!events.contains(event)) || source=="cw_forged_guard_arena" && !elite_first_kill {return false;}
        let Some(target)=self.cw_early_neighbor(east).filter(|v|v.scene_id==target_id) else {return false;};
        let west=[[0.5,6.5],[2.5,6.5],[2.5,9.5],[0.5,9.5]];
        let east_rect=[[22.0,6.5],[24.0,6.5],[24.0,9.5],[22.0,9.5]];
        let fheight=if source=="cw_gear_shaft" {Some(crate::moving_support::HeightRange([0.0,0.1]))} else {None};
        let rheight=if target_id=="cw_gear_shaft" {Some(crate::moving_support::HeightRange([0.0,0.1]))} else {None};
        self.current_scene().transitions.iter().filter(|t|t.id==forward && t.to_scene_id==target_id && t.requires_event.as_deref()==fg && t.height_range_m==fheight && t.polygon==if east {east_rect} else {west}).count()==1
            && target.transitions.iter().filter(|t|t.id==reverse && t.to_scene_id==source && t.requires_event.as_deref()==rg && t.height_range_m==rheight && t.polygon==if east {west} else {east_rect}).count()==1
    }
    pub(crate) fn cw_early_walk(&mut self,target_id:&str,request_id:&str,epoch:u64,position:Vec3,events:&BTreeSet<String>,elite_first_kill:bool)->Result<TransitionResult,SceneRuntimeError> {
        self.begin_request(request_id,epoch)?;
        let source=self.scene_id.clone();
        if !self.cw_early_connection_verified(target_id,events,elite_first_kill) || !position.x_m.is_finite() || position.y_m!=0.0 || !position.z_m.is_finite() || !(6.0..=9.5).contains(&position.z_m) {return Err(SceneRuntimeError::UnsafeTransition);}
        let target=self.registry.scene(target_id).ok_or_else(||SceneRuntimeError::UnknownTransitionTarget(target_id.into()))?;
        let spawn_id=match target_id {"cw_entry_foundry"=>"cw_entry_spawn","cw_pressure_hall"=>"cw_pressure_hall_spawn","cw_conveyor_bridge"=>"cw_conveyor_bridge_spawn","cw_boiler_chamber"=>"cw_boiler_chamber_spawn","cw_gear_shaft"=>"cw_gear_shaft_spawn","cw_furnace_heart"=>"cw_furnace_heart_spawn","cw_forged_guard_arena"=>"cw_forged_guard_arena_spawn","cw_regulator_core"=>"cw_regulator_core_spawn","cw_shutdown_exit"=>"cw_shutdown_exit_spawn",_=>return Err(SceneRuntimeError::UnsafeTransition)};
        let spawn=target.spawns.iter().find(|s|s.id==spawn_id).ok_or_else(||SceneRuntimeError::UnknownTransitionSpawn(spawn_id.into()))?;
        let spawn_position=vec3(spawn.position);let spawn_id=spawn.id.clone();
        let next_epoch=self.world_epoch.checked_add(1).ok_or(SceneRuntimeError::StaleEpoch)?;
        self.scene_id=target_id.into();self.world_epoch=next_epoch;self.checkpoint_id=None;
        self.emitted.clear();self.completed_objectives.clear();self.activated.clear();
        Ok(TransitionResult{target_world_id:"clockworks".into(),target_scene_id:self.scene_id.clone(),spawn_id:spawn_id.clone(),position:spawn_position,next_epoch,event:SceneEvent::Transition{from_scene_id:source,to_scene_id:self.scene_id.clone(),spawn_id,world_epoch:next_epoch}})
    }

    /// Three paired Mist Harbor shore doors, without granting preview rights.
    pub(crate) fn mh_shore_neighbor(&self,east:bool)->Option<&SceneDefinition> {
        if self.current_scene().world_id!="mist_harbor" {return None;}
        let target=match (self.scene_id.as_str(),east) {
            ("mh_fog_pier",true)=>"mh_tidal_warehouse",
            ("mh_tidal_warehouse",false)=>"mh_fog_pier",
            ("mh_tidal_warehouse",true)=>"mh_signal_yard",
            ("mh_signal_yard",false)=>"mh_tidal_warehouse",
            ("mh_signal_yard",true)=>"mh_drowned_quay",
            ("mh_drowned_quay",false)=>"mh_signal_yard",
            ("mh_drowned_quay",true)=>"mh_breakwater",
            ("mh_breakwater",false)=>"mh_drowned_quay",
            ("mh_breakwater",true)=>"mh_pump_station",
            ("mh_pump_station",false)=>"mh_breakwater",
            ("mh_pump_station",true)=>"mh_resonance_tower",
            ("mh_resonance_tower",false)=>"mh_pump_station",
            ("mh_resonance_tower",true)=>"mh_warden_arena",
            ("mh_warden_arena",false)=>"mh_resonance_tower",
            ("mh_warden_arena",true)=>"mh_extraction",
            ("mh_extraction",false)=>"mh_warden_arena",
            _=>return None,
        };
        self.registry.scene(target).filter(|scene|scene.world_id=="mist_harbor")
    }
    pub(crate) fn mh_shore_candidate(&self,target:&str)->Option<Self> {
        if ![false,true].into_iter().any(|east|self.mh_shore_neighbor(east).is_some_and(|s|s.scene_id==target)) {return None;}
        let mut candidate=self.clone();candidate.scene_id=target.into();Some(candidate)
    }
    pub(crate) fn mh_shore_connection_verified(&self,target_id:&str,events:&BTreeSet<String>,warden_defeated:bool)->bool {
        let source=self.scene_id.as_str();
        let (east,forward,reverse,fg,rg)=match (source,target_id) {
            ("mh_fog_pier","mh_tidal_warehouse")=>(true,"mh_to_tidal_warehouse","mh_warehouse_to_fog_pier",None,None),
            ("mh_tidal_warehouse","mh_fog_pier")=>(false,"mh_warehouse_to_fog_pier","mh_to_tidal_warehouse",None,None),
            ("mh_tidal_warehouse","mh_signal_yard")=>(true,"mh_warehouse_to_signal_yard","mh_signal_yard_to_warehouse",None,None),
            ("mh_signal_yard","mh_tidal_warehouse")=>(false,"mh_signal_yard_to_warehouse","mh_warehouse_to_signal_yard",None,None),
            ("mh_signal_yard","mh_drowned_quay")=>(true,"mh_signal_yard_to_quay","mh_drowned_quay_to_signal_yard",None,None),
            ("mh_drowned_quay","mh_signal_yard")=>(false,"mh_drowned_quay_to_signal_yard","mh_signal_yard_to_quay",None,None),
            ("mh_drowned_quay","mh_breakwater")=>(true,"mh_drowned_quay_to_breakwater","mh_breakwater_to_drowned_quay",None,None),
            ("mh_breakwater","mh_drowned_quay")=>(false,"mh_breakwater_to_drowned_quay","mh_drowned_quay_to_breakwater",None,None),
            ("mh_breakwater","mh_pump_station")=>(true,"mh_breakwater_to_pump_station","mh_pump_station_to_breakwater",Some("mist_beacon_east"),None),
            ("mh_pump_station","mh_breakwater")=>(false,"mh_pump_station_to_breakwater","mh_breakwater_to_pump_station",None,Some("mist_beacon_east")),
            ("mh_pump_station","mh_resonance_tower")=>(true,"mh_pump_station_to_tower","mh_tower_to_pump_station",None,None),
            ("mh_resonance_tower","mh_pump_station")=>(false,"mh_tower_to_pump_station","mh_pump_station_to_tower",None,None),
            ("mh_resonance_tower","mh_warden_arena")=>(true,"mh_tower_to_warden_arena","mh_warden_arena_to_tower",Some("mist_signal"),None),
            ("mh_warden_arena","mh_resonance_tower")=>(false,"mh_warden_arena_to_tower","mh_tower_to_warden_arena",None,Some("mist_signal")),
            ("mh_warden_arena","mh_extraction")=>(true,"mh_warden_arena_to_extraction","mh_extraction_to_warden_arena",None,None),
            ("mh_extraction","mh_warden_arena")=>(false,"mh_extraction_to_warden_arena","mh_warden_arena_to_extraction",None,None),
            _=>return false,
        };
        if fg.is_some_and(|event|!events.contains(event)) || source=="mh_warden_arena" && target_id=="mh_extraction" && !warden_defeated {return false;}
        let Some(target)=self.mh_shore_neighbor(east).filter(|v|v.scene_id==target_id) else {return false;};
        let west=[[0.0,6.5],[2.0,6.5],[2.0,9.5],[0.0,9.5]];
        let east_rect=[[22.0,6.5],[24.0,6.5],[24.0,9.5],[22.0,9.5]];
        self.current_scene().transitions.iter().filter(|t|t.id==forward && t.to_scene_id==target_id && t.requires_event.as_deref()==fg && t.polygon==if east {east_rect} else {west}).count()==1
            && target.transitions.iter().filter(|t|t.id==reverse && t.to_scene_id==source && t.requires_event.as_deref()==rg && t.polygon==if east {west} else {east_rect}).count()==1
    }
    pub(crate) fn mh_shore_walk(&mut self,target_id:&str,request_id:&str,epoch:u64,position:Vec3,events:&BTreeSet<String>,warden_defeated:bool)
        ->Result<TransitionResult,SceneRuntimeError> {
        self.begin_request(request_id,epoch)?;
        let source=self.scene_id.clone();
        if !self.mh_shore_connection_verified(target_id,events,warden_defeated) || !position.x_m.is_finite() || position.y_m!=0.0 || !position.z_m.is_finite() || !(6.0..=9.5).contains(&position.z_m) {return Err(SceneRuntimeError::UnsafeTransition);}
        let target=self.registry.scene(target_id).ok_or_else(||SceneRuntimeError::UnknownTransitionTarget(target_id.into()))?;
        let spawn_id=match target_id {
            "mh_fog_pier"=>"mh_fog_pier_spawn", "mh_tidal_warehouse"=>"mh_warehouse_spawn", "mh_signal_yard"=>"mh_signal_yard_spawn", "mh_drowned_quay"=>"mh_drowned_quay_spawn", "mh_breakwater"=>"mh_breakwater_spawn", "mh_pump_station"=>"mh_pump_station_spawn", "mh_resonance_tower"=>"mh_resonance_tower_spawn", "mh_warden_arena"=>"mh_warden_arena_spawn", "mh_extraction"=>"mh_extraction_spawn", _=>return Err(SceneRuntimeError::UnsafeTransition),
        };
        let spawn=target.spawns.iter().find(|s|s.id==spawn_id).ok_or_else(||SceneRuntimeError::UnknownTransitionSpawn(spawn_id.into()))?;
        let spawn_position=vec3(spawn.position);let spawn_id=spawn.id.clone();
        let next_epoch=self.world_epoch.checked_add(1).ok_or(SceneRuntimeError::StaleEpoch)?;
        self.scene_id=target_id.into();self.world_epoch=next_epoch;self.checkpoint_id=None;
        self.emitted.clear();self.completed_objectives.clear();self.activated.clear();
        Ok(TransitionResult{target_world_id:"mist_harbor".into(),target_scene_id:self.scene_id.clone(),spawn_id:spawn_id.clone(),position:spawn_position,next_epoch,event:SceneEvent::Transition{from_scene_id:source,to_scene_id:self.scene_id.clone(),spawn_id,world_epoch:next_epoch}})
    }
    pub(crate) fn hive_route_neighbor(&self,east:bool)->Option<&SceneDefinition> {
        if self.current_scene().world_id!="grey_hive" {return None;}
        let target=match (self.scene_id.as_str(),east) {
            ("gh_entry_maintenance",true)=>"gh_power_room",
            ("gh_power_room",false)=>"gh_entry_maintenance",
            ("gh_lockdown",true)=>"gh_bio_isolation",
            ("gh_bio_isolation",false)=>"gh_lockdown",
            ("gh_bio_isolation",true)=>"gh_gate_b",
            ("gh_gate_b",false)=>"gh_bio_isolation",
            ("gh_deep_decon",true)=>"gh_sentinel_arena",
            ("gh_sentinel_arena",false)=>"gh_deep_decon",
            ("gh_sentinel_arena",true)=>"gh_beacon",
            ("gh_beacon",false)=>"gh_sentinel_arena",
            _=>return None,
        };
        self.registry.scene(target).filter(|scene|scene.world_id=="grey_hive")
    }
    pub(crate) fn hive_route_candidate(&self,target:&str)->Option<Self> {
        if ![false,true].into_iter().any(|east|self.hive_route_neighbor(east).is_some_and(|s|s.scene_id==target)) {return None;}
        let mut candidate=self.clone();
        candidate.scene_id=target.into();
        candidate.lockdown_legacy=None;
        candidate.utility_legacy=None;
        Some(candidate)
    }

    /// Commit a pre-authorized swept crossing on the selected first-floor route.
    /// Callers own the collision union, real-radius crossing and visual proof.
    pub(crate) fn hive_route_walk(&mut self,target_id:&str,request_id:&str,epoch:u64,
        position:Vec3,events:&BTreeSet<String>,sentinel_first_kill:bool)
        ->Result<TransitionResult,SceneRuntimeError> {
        self.begin_request(request_id,epoch)?;
        let source=self.scene_id.clone();
        let (forward_id, forward_source, spawn_id, required, lo, hi)=match (source.as_str(),target_id) {
            ("gh_entry_maintenance","gh_power_room")=>("gh_entry_to_power_room",true,"gh_power_spawn",None,5.5,8.5),
            ("gh_power_room","gh_entry_maintenance")=>("gh_power_return_to_entry",true,"gh_entry_spawn",None,5.5,8.5),
            ("gh_lockdown","gh_bio_isolation")=>("gh_lockdown_to_bio_isolation",true,"gh_bio_isolation_spawn",Some("hive_lockdown"),6.5,9.5),
            ("gh_bio_isolation","gh_lockdown")=>("gh_lockdown_to_bio_isolation",false,"gh_lockdown_spawn",None,6.5,9.5),
            ("gh_bio_isolation","gh_gate_b")=>("gh_bio_to_gate_b",true,"gh_gate_b_spawn",Some("hive_lockdown"),6.5,9.5),
            ("gh_gate_b","gh_bio_isolation")=>("gh_bio_to_gate_b",false,"gh_bio_isolation_spawn",None,6.5,9.5),
            ("gh_deep_decon","gh_sentinel_arena")=>("gh_deep_decon_to_sentinel_arena",true,"gh_sentinel_arena_spawn",None,6.5,9.5),
            ("gh_sentinel_arena","gh_deep_decon")=>("gh_sentinel_arena_return_to_deep_decon",true,"gh_deep_decon_spawn",None,6.5,9.5),
            ("gh_sentinel_arena","gh_beacon")=>("gh_sentinel_arena_to_beacon",true,"gh_beacon_spawn",None,6.5,9.5),
            ("gh_beacon","gh_sentinel_arena")=>("gh_sentinel_arena_to_beacon",false,"gh_sentinel_arena_spawn",None,6.5,9.5),
            _=>return Err(SceneRuntimeError::UnsafeTransition),
        };
        if self.current_scene().world_id!="grey_hive" || !position.x_m.is_finite()
            || position.y_m!=0.0 || !position.z_m.is_finite() || !(lo..=hi).contains(&position.z_m)
            || required.is_some_and(|e|!events.contains(e))
            || (source=="gh_sentinel_arena" && target_id=="gh_beacon" && !sentinel_first_kill) {
            return Err(SceneRuntimeError::UnsafeTransition);
        }
        let target=self.registry.scene(target_id).ok_or_else(||SceneRuntimeError::UnknownTransitionTarget(target_id.into()))?;
        let owner=if forward_source {self.current_scene()} else {target};
        let expected_target=if forward_source {target_id} else {source.as_str()};
        if target.world_id!="grey_hive" || owner.transitions.iter().filter(|t|t.id==forward_id && t.to_scene_id==expected_target).count()!=1 {
            return Err(SceneRuntimeError::UnsafeTransition);
        }
        let spawn=target.spawns.iter().find(|s|s.id==spawn_id).ok_or_else(||SceneRuntimeError::UnknownTransitionSpawn(spawn_id.into()))?;
        let spawn_position=vec3(spawn.position);let spawn_id=spawn.id.clone();
        let next_epoch=self.world_epoch.checked_add(1).ok_or(SceneRuntimeError::StaleEpoch)?;
        self.scene_id=target_id.into();self.world_epoch=next_epoch;self.checkpoint_id=None;
        self.lockdown_legacy=None;self.utility_legacy=None;
        self.emitted.clear();self.completed_objectives.clear();self.activated.clear();
        Ok(TransitionResult{target_world_id:"grey_hive".into(),target_scene_id:self.scene_id.clone(),spawn_id:spawn_id.clone(),position:spawn_position,next_epoch,
            event:SceneEvent::Transition{from_scene_id:source,to_scene_id:self.scene_id.clone(),spawn_id,world_epoch:next_epoch}})
    }
    /// The Shaft east edge is distinct from its Gate A west edge.
    pub(crate) fn shaft_lockdown_neighbor(&self)->Option<&SceneDefinition> {
        if self.current_scene().world_id!="grey_hive" {return None;}
        let id=match self.scene_id.as_str(){"gh_central_shaft"=>"gh_lockdown","gh_lockdown"=>"gh_central_shaft",_=>return None};
        let neighbor=self.registry.scene(id)?;
        (neighbor.world_id=="grey_hive").then_some(neighbor)
    }

    /// Explicit new walking edge only. Ordinary scene transitions continue to
    /// use the authored list and its spawn rules.
    pub(crate) fn decon_reverse_walk(&mut self,request_id:&str,epoch:u64,position:Vec3,
        events:&BTreeSet<String>)->Result<TransitionResult,SceneRuntimeError> {
        self.begin_request(request_id,epoch)?;
        if self.current_scene().world_id!="grey_hive" || self.scene_id!="gh_deep_decon"
            || !position.x_m.is_finite() || !position.y_m.is_finite() || !position.z_m.is_finite()
            || position.y_m!=0.0 || !(0.0..=2.0).contains(&position.x_m)
            || !(7.0..=9.0).contains(&position.z_m) {return Err(SceneRuntimeError::OutOfRange);}
        if !events.contains("hive_lockdown") {return Err(SceneRuntimeError::ProgressionLocked);}
        let target=self.registry.scene("gh_gate_b").ok_or_else(||SceneRuntimeError::UnknownTransitionTarget("gh_gate_b".into()))?;
        if target.world_id!="grey_hive" || target.transitions.iter().filter(|t|t.id=="gh_gate_b_to_deep_decon"
            && t.to_scene_id=="gh_deep_decon" && t.requires_event.as_deref()==Some("hive_lockdown")
            && t.polygon==[[22.0,7.0],[24.0,7.0],[24.0,9.0],[22.0,9.0]]).count()!=1 {
            return Err(SceneRuntimeError::UnsafeTransition);
        }
        let spawn=target.spawns.iter().find(|s|s.id=="gh_gate_b_spawn").ok_or_else(||SceneRuntimeError::UnknownTransitionSpawn("gh_gate_b_spawn".into()))?;
        let spawn_id=spawn.id.clone();let spawn_position=vec3(spawn.position);
        let next_epoch=self.world_epoch.checked_add(1).ok_or(SceneRuntimeError::StaleEpoch)?;
        self.scene_id="gh_gate_b".into();self.world_epoch=next_epoch;self.checkpoint_id=None;
        self.emitted.clear();self.completed_objectives.clear();self.activated.clear();
        Ok(TransitionResult{target_world_id:"grey_hive".into(),target_scene_id:self.scene_id.clone(),spawn_id:spawn_id.clone(),position:spawn_position,next_epoch,
            event:SceneEvent::Transition{from_scene_id:"gh_deep_decon".into(),to_scene_id:self.scene_id.clone(),spawn_id,world_epoch:next_epoch}})
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
        self.lockdown_legacy = None;
        self.utility_legacy = None;
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

    pub(crate) fn validate_scene_history(
        &self,
        persistent: &crate::world_persistent_v1::WorldPersistentState,
    ) -> Result<(), String> {
        Self::validate_scene_history_in_registry(&self.registry, persistent)
    }

    pub(crate) fn validate_scene_history_in_registry(
        registry: &WorldRegistry,
        persistent: &crate::world_persistent_v1::WorldPersistentState,
    ) -> Result<(), String> {
        persistent
            .validate()
            .map_err(|_| "E_SAVE_SCENE_HISTORY_INVALID".to_string())?;
        if !registry.is_complete_clockworks_production() {
            return Err("E_SAVE_SCENE_HISTORY_REGISTRY_REQUIRED".into());
        }
        let Some(history) = &persistent.scene_history_v1 else {
            return Ok(());
        };
        if history.scene_states.len() > registry.scenes().count() {
            return Err("E_SAVE_SCENE_HISTORY_INVALID".into());
        }
        for row in &history.scene_states {
            let definition = registry
                .scene(&row.scene_id)
                .ok_or("E_SAVE_SCENE_HISTORY_INVALID")?;
            validate_scene_history_row(definition, row)?;
        }
        Ok(())
    }

    /// Restore only authored generic scene-local state. This does not issue
    /// commands or alter the target's request ledger or world progression.
    pub(crate) fn hydrate_scene_history(
        &mut self,
        persistent: &crate::world_persistent_v1::WorldPersistentState,
    ) -> Result<(), String> {
        self.validate_scene_history(persistent)?;
        // Requests belong to the departed epoch; the installed target begins fresh.
        self.processed.clear();
        if let Some(row) = persistent.scene_history_row(&self.scene_id) {
            self.activated = row.activated_ids.iter().cloned().collect();
            self.emitted = row.emitted_event_ids.iter().cloned().collect();
            self.checkpoint_id = row.checkpoint_id.clone();
        } else {
            self.activated.clear();
            self.emitted.clear();
            self.checkpoint_id = None;
        }
        self.completed_objectives.clear();
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

fn validate_scene_history_row(
    definition: &SceneDefinition,
    row: &SceneStateSave,
) -> Result<(), String> {
    if definition.scene_id.as_str() != row.scene_id.as_str() {
        return Err("E_SAVE_SCENE_HISTORY_INVALID".into());
    }
    let activated: BTreeSet<&str> = row.activated_ids.iter().map(String::as_str).collect();
    let mut capturable_ids = BTreeSet::new();
    for interaction in &definition.interactions {
        let static_return_marker = definition.scene_id == "rs_core_room"
            && interaction.kind.ends_with("_marker");
        let aggregate_marker = definition
            .interaction_aggregates
            .iter()
            .any(|aggregate| aggregate.marker_id.as_str() == interaction.id.as_str());
        let command_only_control = matches!(
            interaction.kind.as_str(),
            "pump_control" | "coolant_valve" | "environment_control"
        );
        if !static_return_marker && !aggregate_marker && !command_only_control {
            capturable_ids.insert(interaction.id.as_str());
        }
    }
    capturable_ids.extend(definition.triggers.iter().map(|trigger| trigger.id.as_str()));
    if row.activated_ids.iter().any(|id| !capturable_ids.contains(id.as_str())) {
        return Err("E_SAVE_SCENE_HISTORY_INVALID".into());
    }

    // Rebuild precisely the event set produced by the existing interaction,
    // trigger, aggregate, and durable-interaction restoration semantics.
    let mut expected_events = BTreeSet::new();
    for interaction in &definition.interactions {
        if activated.contains(interaction.id.as_str()) {
            if let Some(event) = &interaction.event {
                expected_events.insert(event.as_str());
            }
        }
    }
    for trigger in &definition.triggers {
        if activated.contains(trigger.id.as_str()) {
            expected_events.insert(trigger.event.as_str());
        }
    }
    for aggregate in &definition.interaction_aggregates {
        if aggregate.member_ids.iter().all(|id| activated.contains(id.as_str())) {
            expected_events.insert(aggregate.event.as_str());
        }
    }
    let saved_events: BTreeSet<_> = row.emitted_event_ids.iter().map(String::as_str).collect();
    if saved_events != expected_events
        || row.checkpoint_id.as_ref().is_some_and(|checkpoint| {
            !definition.checkpoints.iter().any(|item| item.id.as_str() == checkpoint.as_str())
        })
    {
        return Err("E_SAVE_SCENE_HISTORY_INVALID".into());
    }
    Ok(())
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
