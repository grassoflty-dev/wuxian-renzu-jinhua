//! Closed, validated gameplay effects and a deterministic resolver.
//! The caller selects currently applicable sources; runtime ownership is not wired here.

mod map_projection;
pub use map_projection::*;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::player_rules::{
    EffectivePlayerRules, KnowledgeLevel, FULL_DAMAGE_BPS, MAX_REVEAL_RADIUS_MM,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectSourceKind {
    InnateCapability,
    Item,
    Equipment,
    Bloodline,
    Skill,
    Treasure,
    Gene,
    Cultivation,
    TemporaryBuff,
    WorldEffect,
    QuestEffect,
}

/// Presentation classification is independent of the gameplay effect variant.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    Perception,
    Information,
    Body,
    Mobility,
    Cognition,
    Combat,
    Space,
    Soul,
    Rule,
    Utility,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectLifetime {
    Permanent,
    Owned,
    Equipped,
    Selected,
    Activated,
    Timed { remaining_ms: u64 },
    Scene,
    World,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackRule {
    Any,
    Add,
    Multiply,
    Max,
    Min,
    HighestPriority,
    Replace,
    MultiplyRemaining,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerceptionTag {
    RearView,
}

/// Closed gameplay permissions for the existing V1 capability consumers.
/// Acquisition IDs and selection policy belong to the trusted source adapter.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityPermission {
    EnemyVitalsBasic,
    DelayedRegeneration,
    AcousticMapping,
    AuthoredAirStep,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityPermissionEffect {
    pub tag: CapabilityPermission,
    pub stack: StackRule,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapKnowledgeTag {
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainTag {
    Water,
    Mud,
    Concrete,
    Conveyor,
    #[serde(rename = "terrain.water_deep")]
    WaterDeep,
    #[serde(rename = "terrain.water_shallow")]
    WaterShallow,
    #[serde(rename = "terrain.wet_floor")]
    WetFloor,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardTag {
    Heat,
    Pressure,
    Toxin,
    Water,
    Machinery,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerceptionEffect {
    pub tag: PerceptionTag,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapKnowledgeEffect {
    pub tag: MapKnowledgeTag,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapKnowledgeLevelEffect {
    pub tag: MapKnowledgeTag,
    pub level: KnowledgeLevel,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapRevealRadiusEffect {
    /// Millimeters; zero and values above the finite core limit are rejected.
    pub radius_mm: u32,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainPenaltyEffect {
    pub tag: TerrainTag,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainMovementModifierEffect {
    pub tag: TerrainTag,
    /// 1..=10_000 basis points of normal movement speed, never a passage grant.
    /// Max selects the strongest explicit modifier; penalty ignore takes precedence.
    pub multiplier_bps: u16,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HazardResistanceEffect {
    pub tag: HazardTag,
    /// 1..=9_999 basis points. Full immunity requires HazardImmunity.
    pub reduction_bps: u16,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HazardImmunityEffect {
    pub tag: HazardTag,
    pub stack: StackRule,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectSpec {
    GrantCapability(CapabilityPermissionEffect),
    Perception(PerceptionEffect),
    MapKnowledge(MapKnowledgeEffect),
    MapKnowledgeLevel(MapKnowledgeLevelEffect),
    MapRevealRadius(MapRevealRadiusEffect),
    TerrainPenaltyIgnore(TerrainPenaltyEffect),
    TerrainMovementModifier(TerrainMovementModifierEffect),
    HazardResistance(HazardResistanceEffect),
    HazardImmunity(HazardImmunityEffect),
}

impl EffectSpec {
    fn validate(&self) -> Result<(), EffectError> {
        let (stack, expected) = match self {
            Self::GrantCapability(value) => (value.stack, StackRule::Any),
            Self::Perception(value) => (value.stack, StackRule::Any),
            Self::MapKnowledge(value) => (value.stack, StackRule::Any),
            Self::MapKnowledgeLevel(value) => {
                if value.level == KnowledgeLevel::None {
                    return Err(EffectError::InvalidKnowledgeLevel);
                }
                (value.stack, StackRule::Max)
            }
            Self::MapRevealRadius(value) => {
                if value.radius_mm == 0 || value.radius_mm > MAX_REVEAL_RADIUS_MM {
                    return Err(EffectError::InvalidRevealRadius);
                }
                (value.stack, StackRule::Max)
            }
            Self::TerrainPenaltyIgnore(value) => (value.stack, StackRule::Any),
            Self::TerrainMovementModifier(value) => {
                if !(1..=FULL_DAMAGE_BPS).contains(&value.multiplier_bps) {
                    return Err(EffectError::InvalidMovementMultiplier);
                }
                (value.stack, StackRule::Max)
            }
            Self::HazardResistance(value) => {
                if !(1..FULL_DAMAGE_BPS).contains(&value.reduction_bps) {
                    return Err(EffectError::InvalidResistance);
                }
                (value.stack, StackRule::MultiplyRemaining)
            }
            Self::HazardImmunity(value) => (value.stack, StackRule::Any),
        };
        if stack != expected {
            return Err(EffectError::InvalidStackRule);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectSource {
    pub source_kind: EffectSourceKind,
    pub source_id: String,
    pub instance_id: String,
    pub lifetime: EffectLifetime,
    #[serde(default)]
    pub ui_category: Option<CapabilityCategory>,
    pub effects: Vec<EffectSpec>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectError {
    TooManySources,
    TooManyEffects,
    InvalidId,
    EmptyEffects,
    InvalidLifetime,
    InvalidStackRule,
    InvalidResistance,
    InvalidMovementMultiplier,
    InvalidRevealRadius,
    InvalidKnowledgeLevel,
    DuplicateSource,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._:-".contains(&byte)
        })
}

pub struct EffectResolver;

impl EffectResolver {
    pub fn resolve(sources: &[EffectSource]) -> Result<EffectivePlayerRules, EffectError> {
        if sources.len() > 1_024 {
            return Err(EffectError::TooManySources);
        }
        let mut identities = BTreeSet::new();
        for source in sources {
            if !valid_id(&source.source_id) || !valid_id(&source.instance_id) {
                return Err(EffectError::InvalidId);
            }
            if matches!(source.lifetime, EffectLifetime::Timed { remaining_ms: 0 }) {
                return Err(EffectError::InvalidLifetime);
            }
            if source.effects.is_empty() {
                return Err(EffectError::EmptyEffects);
            }
            if source.effects.len() > 128 {
                return Err(EffectError::TooManyEffects);
            }
            if !identities.insert((source.source_kind, source.instance_id.as_str())) {
                return Err(EffectError::DuplicateSource);
            }
            for effect in &source.effects {
                effect.validate()?;
            }
        }

        let mut ordered: Vec<_> = sources.iter().collect();
        ordered.sort_by_key(|source| {
            (
                source.source_kind,
                source.instance_id.as_str(),
                source.source_id.as_str(),
            )
        });
        let mut rules = EffectivePlayerRules::default();
        for source in ordered {
            let mut effects = source.effects.clone();
            effects.sort();
            for effect in effects {
                match effect {
                    EffectSpec::GrantCapability(value) => {
                        rules.capability_permissions.insert(value.tag);
                    }
                    EffectSpec::Perception(value) => match value.tag {
                        PerceptionTag::RearView => rules.perception.rear_view = true,
                    },
                    EffectSpec::MapKnowledge(value) => {
                        rules.map.granted.insert(value.tag);
                        rules.map.set_level(value.tag, KnowledgeLevel::Full);
                    }
                    EffectSpec::MapKnowledgeLevel(value) => {
                        let current = rules.map.level_for_tag(value.tag);
                        rules.map.set_level(value.tag, current.max(value.level));
                    }
                    EffectSpec::MapRevealRadius(value) => {
                        rules.map.reveal_radius_mm =
                            rules.map.reveal_radius_mm.max(value.radius_mm);
                    }
                    EffectSpec::TerrainPenaltyIgnore(value) => {
                        rules.terrain.ignored_move_penalties.insert(value.tag);
                    }
                    EffectSpec::TerrainMovementModifier(value) => {
                        rules
                            .terrain
                            .movement_multiplier_bps
                            .entry(value.tag)
                            .and_modify(|current| *current = (*current).max(value.multiplier_bps))
                            .or_insert(value.multiplier_bps);
                    }
                    EffectSpec::HazardResistance(value) => {
                        let remaining = rules
                            .hazards
                            .remaining_damage_bps
                            .entry(value.tag)
                            .or_insert(FULL_DAMAGE_BPS);
                        let factor = u32::from(FULL_DAMAGE_BPS - value.reduction_bps);
                        *remaining =
                            ((u32::from(*remaining) * factor + u32::from(FULL_DAMAGE_BPS / 2))
                                / u32::from(FULL_DAMAGE_BPS)) as u16;
                    }
                    EffectSpec::HazardImmunity(value) => {
                        rules.hazards.immune.insert(value.tag);
                    }
                }
            }
        }
        Ok(rules)
    }
}

#[cfg(test)]
mod tests;
