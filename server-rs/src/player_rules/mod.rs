//! Resolved permissions only. No item, bloodline, save, or scene identity is stored here.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::effects::{CapabilityPermission, HazardTag, MapKnowledgeTag, TerrainTag};

pub const FULL_DAMAGE_BPS: u16 = 10_000;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerceptionRules {
    pub rear_view: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeLevel {
    #[default]
    None,
    ExploredOnly,
    DetectedOnly,
    Known,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeChannel {
    Topology,
    Terrain,
    Connections,
    Objectives,
    Enemies,
    Hazards,
    Loot,
    Npcs,
    Secrets,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapKnowledgeRules {
    /// Legacy grants remain readable and writable for existing resolver callers.
    pub granted: BTreeSet<MapKnowledgeTag>,
    #[serde(default)]
    pub topology: KnowledgeLevel,
    #[serde(default)]
    pub terrain: KnowledgeLevel,
    #[serde(default)]
    pub connections: KnowledgeLevel,
    #[serde(default)]
    pub objectives: KnowledgeLevel,
    #[serde(default)]
    pub enemies: KnowledgeLevel,
    #[serde(default)]
    pub hazards: KnowledgeLevel,
    #[serde(default)]
    pub loot: KnowledgeLevel,
    #[serde(default)]
    pub npcs: KnowledgeLevel,
    #[serde(default)]
    pub secrets: KnowledgeLevel,
    /// Exact millimeters avoids NaN/infinity and remains serde/Eq compatible.
    #[serde(default)]
    pub reveal_radius_mm: u32,
}

impl Default for MapKnowledgeRules {
    fn default() -> Self {
        Self {
            granted: BTreeSet::new(),
            topology: KnowledgeLevel::None,
            terrain: KnowledgeLevel::None,
            connections: KnowledgeLevel::None,
            objectives: KnowledgeLevel::None,
            enemies: KnowledgeLevel::None,
            hazards: KnowledgeLevel::None,
            loot: KnowledgeLevel::None,
            npcs: KnowledgeLevel::None,
            secrets: KnowledgeLevel::None,
            reveal_radius_mm: 0,
        }
    }
}

impl MapKnowledgeRules {
    pub fn level_for_tag(&self, tag: MapKnowledgeTag) -> KnowledgeLevel {
        let level = match tag {
            MapKnowledgeTag::Topology => self.topology,
            MapKnowledgeTag::Terrain => self.terrain,
            MapKnowledgeTag::Connections => self.connections,
            MapKnowledgeTag::Objectives => self.objectives,
            MapKnowledgeTag::Enemies => self.enemies,
            MapKnowledgeTag::Hazards => self.hazards,
            MapKnowledgeTag::Loot => self.loot,
            MapKnowledgeTag::Npcs => self.npcs,
            MapKnowledgeTag::Secrets => self.secrets,
        };
        if level == KnowledgeLevel::None && self.granted.contains(&tag) {
            KnowledgeLevel::Full
        } else {
            level
        }
    }

    pub fn set_level(&mut self, tag: MapKnowledgeTag, level: KnowledgeLevel) {
        let target = match tag {
            MapKnowledgeTag::Topology => &mut self.topology,
            MapKnowledgeTag::Terrain => &mut self.terrain,
            MapKnowledgeTag::Connections => &mut self.connections,
            MapKnowledgeTag::Objectives => &mut self.objectives,
            MapKnowledgeTag::Enemies => &mut self.enemies,
            MapKnowledgeTag::Hazards => &mut self.hazards,
            MapKnowledgeTag::Loot => &mut self.loot,
            MapKnowledgeTag::Npcs => &mut self.npcs,
            MapKnowledgeTag::Secrets => &mut self.secrets,
        };
        *target = level;
    }

    pub fn level(&self, channel: KnowledgeChannel) -> KnowledgeLevel {
        match channel {
            KnowledgeChannel::Topology => self.level_for_tag(MapKnowledgeTag::Topology),
            KnowledgeChannel::Terrain => self.level_for_tag(MapKnowledgeTag::Terrain),
            KnowledgeChannel::Connections => self.level_for_tag(MapKnowledgeTag::Connections),
            KnowledgeChannel::Objectives => self.level_for_tag(MapKnowledgeTag::Objectives),
            KnowledgeChannel::Enemies => self.level_for_tag(MapKnowledgeTag::Enemies),
            KnowledgeChannel::Hazards => self.level_for_tag(MapKnowledgeTag::Hazards),
            KnowledgeChannel::Loot => self.level_for_tag(MapKnowledgeTag::Loot),
            KnowledgeChannel::Npcs => self.level_for_tag(MapKnowledgeTag::Npcs),
            KnowledgeChannel::Secrets => self.level_for_tag(MapKnowledgeTag::Secrets),
        }
    }

    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if self.reveal_radius_mm > MAX_REVEAL_RADIUS_MM {
            return Err(RuleValidationError::InvalidRevealRadius);
        }
        Ok(())
    }
}

pub const MAX_REVEAL_RADIUS_MM: u32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleValidationError {
    InvalidRevealRadius,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainRules {
    /// Ignores movement penalties on these terrain tags, not collision or route gates.
    pub ignored_move_penalties: BTreeSet<TerrainTag>,
    /// Explicit terrain passage granted by world state or validated player rules.
    #[serde(default)]
    pub passable: BTreeSet<TerrainTag>,
    /// Fixed-point movement multipliers, 0..=10_000.
    #[serde(default)]
    pub movement_multiplier_bps: BTreeMap<TerrainTag, u16>,
}

impl TerrainRules {
    pub fn movement_multiplier_bps(&self, tag: TerrainTag) -> u16 {
        self.movement_multiplier_bps
            .get(&tag)
            .copied()
            .unwrap_or(match tag {
                TerrainTag::WaterShallow => 6_000,
                TerrainTag::WetFloor => FULL_DAMAGE_BPS,
                _ => FULL_DAMAGE_BPS,
            })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HazardRules {
    /// Deterministic fixed-point remaining-damage factors, 0..=10_000.
    pub remaining_damage_bps: BTreeMap<HazardTag, u16>,
    /// Explicit immunity is distinct from partial resistance.
    pub immune: BTreeSet<HazardTag>,
}

impl HazardRules {
    pub fn remaining_damage_bps(&self, tag: HazardTag) -> u16 {
        if self.immune.contains(&tag) {
            0
        } else {
            self.remaining_damage_bps
                .get(&tag)
                .copied()
                .unwrap_or(FULL_DAMAGE_BPS)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryTag {
    World,
    Quest,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObstacleTag {
    Wall,
    SoftObstacle,
    TraversalMarker,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementMode {
    Ground,
    Hover,
    AirStep,
    Flight,
    Phase,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MovementContext {
    pub terrain: Option<TerrainTag>,
    pub hazard: Option<HazardTag>,
    pub obstacle: Option<ObstacleTag>,
    pub boundary: Option<BoundaryTag>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MovementResolution {
    pub allowed: bool,
    pub terrain_penalty_ignored: bool,
    pub terrain_penalty_bps: u16,
    pub movement_multiplier_bps: u16,
    pub remaining_hazard_damage_bps: u16,
}

impl EffectivePlayerRules {
    /// Resolves movement constraints without letting penalty immunity erase collision or gates.
    pub fn resolve_movement(
        &self,
        mode: MovementMode,
        context: MovementContext,
    ) -> MovementResolution {
        let blocked_boundary = context.boundary.is_some_and(|tag| {
            matches!(tag, BoundaryTag::World | BoundaryTag::Quest)
                || self.boundaries.blocked.contains(&tag)
        });
        let blocked_terrain = context.terrain.is_some_and(|tag| {
            tag == TerrainTag::WaterDeep
                && !self.terrain.passable.contains(&tag)
                && !matches!(mode, MovementMode::Hover | MovementMode::Flight)
        });
        let blocked_obstacle = match context.obstacle {
            None => false,
            Some(ObstacleTag::TraversalMarker) => mode != MovementMode::AirStep,
            Some(ObstacleTag::SoftObstacle) => mode != MovementMode::Phase,
            Some(ObstacleTag::Wall) => true,
        };
        let terrain_penalty_ignored = context
            .terrain
            .is_some_and(|tag| self.terrain.ignored_move_penalties.contains(&tag));
        let remaining_hazard_damage_bps = context
            .hazard
            .map(|tag| self.hazards.remaining_damage_bps(tag))
            .unwrap_or(FULL_DAMAGE_BPS);
        let terrain_penalty_bps = if terrain_penalty_ignored {
            0
        } else {
            match context.terrain {
                Some(TerrainTag::Water) => 3_000,
                Some(TerrainTag::WaterShallow) => 4_000,
                Some(TerrainTag::WaterDeep) | Some(TerrainTag::WetFloor) => 0,
                Some(TerrainTag::Mud) => 2_000,
                Some(TerrainTag::Conveyor) => 1_000,
                Some(TerrainTag::Concrete) | None => 0,
            }
        };
        let movement_multiplier_bps = if terrain_penalty_ignored {
            FULL_DAMAGE_BPS
        } else {
            context
                .terrain
                .map(|tag| self.terrain.movement_multiplier_bps(tag))
                .unwrap_or(FULL_DAMAGE_BPS)
        };
        MovementResolution {
            allowed: !blocked_boundary && !blocked_obstacle && !blocked_terrain,
            terrain_penalty_ignored,
            terrain_penalty_bps,
            movement_multiplier_bps,
            remaining_hazard_damage_bps,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryRules {
    /// These gates stay blocked; this first core has no effect capable of removing them.
    pub blocked: BTreeSet<BoundaryTag>,
}

impl Default for BoundaryRules {
    fn default() -> Self {
        Self {
            blocked: BTreeSet::from([BoundaryTag::World, BoundaryTag::Quest]),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivePlayerRules {
    #[serde(default)]
    pub capability_permissions: BTreeSet<CapabilityPermission>,
    pub perception: PerceptionRules,
    pub map: MapKnowledgeRules,
    pub terrain: TerrainRules,
    pub hazards: HazardRules,
    pub boundaries: BoundaryRules,
}
