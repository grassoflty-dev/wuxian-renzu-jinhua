//! Pure map-information projection. Invalid input yields no partial projection.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::effects::{HazardTag, TerrainTag};
use crate::player_rules::{
    BoundaryTag, EffectivePlayerRules, KnowledgeChannel, KnowledgeLevel, ObstacleTag,
    RuleValidationError,
};

const MAX_COORDINATE_M: f64 = 1_000_000.0;
const MAX_MAP_ENTRIES: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapPolygon {
    pub vertices: Vec<MapPoint>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalMapRegion {
    pub id: String,
    pub channel: KnowledgeChannel,
    pub polygon: MapPolygon,
    pub terrain: Option<TerrainTag>,
    pub boundary: Option<BoundaryTag>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalMapFeature {
    pub id: String,
    pub channel: KnowledgeChannel,
    pub position: MapPoint,
    pub region_id: Option<String>,
    pub terrain: Option<TerrainTag>,
    pub hazard: Option<HazardTag>,
    pub obstacle: Option<ObstacleTag>,
    pub boundary: Option<BoundaryTag>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalMap {
    pub regions: Vec<CanonicalMapRegion>,
    pub features: Vec<CanonicalMapFeature>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapExplorationState {
    pub player_position: Option<MapPoint>,
    pub explored_region_ids: BTreeSet<String>,
    pub known_region_ids: BTreeSet<String>,
    pub detected_feature_ids: BTreeSet<String>,
    pub known_feature_ids: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapProjectionError {
    InvalidRules,
    InvalidId,
    DuplicateId,
    TooManyEntries,
    InvalidCoordinate,
    InvalidPolygon,
    InvalidExplorationReference,
    InvalidChannelData,
}

impl From<RuleValidationError> for MapProjectionError {
    fn from(_: RuleValidationError) -> Self {
        Self::InvalidRules
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectedMapRegion {
    pub id: String,
    pub channel: KnowledgeChannel,
    pub polygon: MapPolygon,
    pub terrain: Option<TerrainTag>,
    pub boundary: Option<BoundaryTag>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectedMapFeature {
    /// Missing for proximity detections so a hidden canonical ID cannot leak.
    pub id: Option<String>,
    pub channel: KnowledgeChannel,
    pub position: MapPoint,
    pub knowledge: KnowledgeLevel,
    pub terrain: Option<TerrainTag>,
    pub hazard: Option<HazardTag>,
    pub obstacle: Option<ObstacleTag>,
    pub boundary: Option<BoundaryTag>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapKnowledgeProjection {
    pub regions: Vec<ProjectedMapRegion>,
    pub features: Vec<ProjectedMapFeature>,
}

/// Projects only channel-authorized knowledge; invalid source geometry fails the whole call.
pub fn project_map_knowledge(
    canonical: &CanonicalMap,
    exploration: &MapExplorationState,
    rules: &EffectivePlayerRules,
) -> Result<MapKnowledgeProjection, MapProjectionError> {
    rules.map.validate()?;
    if canonical.regions.len() + canonical.features.len() > MAX_MAP_ENTRIES {
        return Err(MapProjectionError::TooManyEntries);
    }
    if let Some(position) = exploration.player_position {
        validate_point(position)?;
    }

    let mut region_ids = BTreeSet::new();
    for region in &canonical.regions {
        validate_id(&region.id)?;
        if !region_ids.insert(region.id.as_str()) {
            return Err(MapProjectionError::DuplicateId);
        }
        validate_polygon(&region.polygon)?;
        if region.terrain.is_some() && region.channel != KnowledgeChannel::Terrain {
            return Err(MapProjectionError::InvalidChannelData);
        }
        if region.boundary.is_some() && region.channel != KnowledgeChannel::Topology {
            return Err(MapProjectionError::InvalidChannelData);
        }
    }
    let region_by_id: BTreeMap<_, _> = canonical
        .regions
        .iter()
        .map(|region| (region.id.as_str(), region))
        .collect();
    let mut feature_ids = BTreeSet::new();
    for feature in &canonical.features {
        validate_id(&feature.id)?;
        if !feature_ids.insert(feature.id.as_str()) {
            return Err(MapProjectionError::DuplicateId);
        }
        validate_point(feature.position)?;
        if feature
            .region_id
            .as_deref()
            .is_some_and(|id| !region_ids.contains(id))
        {
            return Err(MapProjectionError::InvalidExplorationReference);
        }
        if (feature.terrain.is_some() && feature.channel != KnowledgeChannel::Terrain)
            || (feature.hazard.is_some() && feature.channel != KnowledgeChannel::Hazards)
            || (feature.obstacle.is_some() && feature.channel != KnowledgeChannel::Topology)
            || (feature.boundary.is_some()
                && !matches!(
                    feature.channel,
                    KnowledgeChannel::Topology | KnowledgeChannel::Connections
                ))
        {
            return Err(MapProjectionError::InvalidChannelData);
        }
        if let Some(region_id) = &feature.region_id {
            let region = region_by_id
                .get(region_id.as_str())
                .expect("validated region reference");
            if !point_in_polygon(feature.position, &region.polygon) {
                return Err(MapProjectionError::InvalidExplorationReference);
            }
        }
    }
    if exploration
        .explored_region_ids
        .iter()
        .chain(exploration.known_region_ids.iter())
        .any(|id| !region_ids.contains(id.as_str()))
        || exploration
            .detected_feature_ids
            .iter()
            .chain(exploration.known_feature_ids.iter())
            .any(|id| !feature_ids.contains(id.as_str()))
    {
        return Err(MapProjectionError::InvalidExplorationReference);
    }

    let mut projection = MapKnowledgeProjection::default();
    for region in &canonical.regions {
        let level = rules.map.level(region.channel);
        let authorized = match level {
            KnowledgeLevel::None | KnowledgeLevel::DetectedOnly => false,
            KnowledgeLevel::ExploredOnly => exploration.explored_region_ids.contains(&region.id),
            KnowledgeLevel::Known => exploration.known_region_ids.contains(&region.id),
            KnowledgeLevel::Full => true,
        };
        if authorized {
            projection.regions.push(ProjectedMapRegion {
                id: region.id.clone(),
                channel: region.channel,
                polygon: region.polygon.clone(),
                terrain: (region.channel == KnowledgeChannel::Terrain)
                    .then_some(region.terrain)
                    .flatten(),
                boundary: region.boundary,
            });
        }
    }

    let radius_m = f64::from(rules.map.reveal_radius_mm) / 1_000.0;
    let radius_squared = radius_m * radius_m;
    for feature in &canonical.features {
        let level = rules.map.level(feature.channel);
        let is_detected = exploration.detected_feature_ids.contains(&feature.id)
            || (radius_m > 0.0
                && exploration.player_position.is_some_and(|origin| {
                    distance_squared(origin, feature.position) <= radius_squared
                }));
        let (include, effective_level) = match level {
            KnowledgeLevel::None => (false, KnowledgeLevel::None),
            KnowledgeLevel::ExploredOnly => (
                feature
                    .region_id
                    .as_ref()
                    .is_some_and(|id| exploration.explored_region_ids.contains(id)),
                KnowledgeLevel::ExploredOnly,
            ),
            KnowledgeLevel::DetectedOnly => (is_detected, KnowledgeLevel::DetectedOnly),
            KnowledgeLevel::Known => (
                exploration.known_feature_ids.contains(&feature.id),
                KnowledgeLevel::Known,
            ),
            KnowledgeLevel::Full => (true, KnowledgeLevel::Full),
        };
        if include {
            let detailed = effective_level != KnowledgeLevel::DetectedOnly;
            projection.features.push(ProjectedMapFeature {
                id: detailed.then(|| feature.id.clone()),
                channel: feature.channel,
                position: feature.position,
                knowledge: effective_level,
                terrain: (detailed && feature.channel == KnowledgeChannel::Terrain)
                    .then_some(feature.terrain)
                    .flatten(),
                hazard: (detailed && feature.channel == KnowledgeChannel::Hazards)
                    .then_some(feature.hazard)
                    .flatten(),
                obstacle: (detailed && feature.channel == KnowledgeChannel::Topology)
                    .then_some(feature.obstacle)
                    .flatten(),
                boundary: (detailed
                    && matches!(
                        feature.channel,
                        KnowledgeChannel::Topology | KnowledgeChannel::Connections
                    ))
                .then_some(feature.boundary)
                .flatten(),
            });
        }
    }
    projection
        .regions
        .sort_by(|left, right| left.id.cmp(&right.id));
    projection.features.sort_by(|left, right| {
        left.channel
            .cmp(&right.channel)
            .then_with(|| left.id.cmp(&right.id))
            .then_with(|| left.position.x.total_cmp(&right.position.x))
            .then_with(|| left.position.y.total_cmp(&right.position.y))
    });
    Ok(projection)
}

fn validate_id(id: &str) -> Result<(), MapProjectionError> {
    if id.is_empty()
        || id.len() > 128
        || !id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._:-".contains(&byte)
        })
    {
        return Err(MapProjectionError::InvalidId);
    }
    Ok(())
}

fn validate_point(point: MapPoint) -> Result<(), MapProjectionError> {
    if !point.x.is_finite()
        || !point.y.is_finite()
        || point.x.abs() > MAX_COORDINATE_M
        || point.y.abs() > MAX_COORDINATE_M
    {
        return Err(MapProjectionError::InvalidCoordinate);
    }
    Ok(())
}

fn validate_polygon(polygon: &MapPolygon) -> Result<(), MapProjectionError> {
    if polygon.vertices.len() < 3 || polygon.vertices.len() > 4_096 {
        return Err(MapProjectionError::InvalidPolygon);
    }
    for point in &polygon.vertices {
        validate_point(*point)?;
    }
    if polygon.vertices.windows(2).any(|pair| pair[0] == pair[1])
        || polygon.vertices.first() == polygon.vertices.last()
    {
        return Err(MapProjectionError::InvalidPolygon);
    }
    let area_twice: f64 = polygon
        .vertices
        .iter()
        .zip(polygon.vertices.iter().cycle().skip(1))
        .take(polygon.vertices.len())
        .map(|(a, b)| a.x * b.y - b.x * a.y)
        .sum();
    if !area_twice.is_finite() || area_twice.abs() < f64::EPSILON {
        return Err(MapProjectionError::InvalidPolygon);
    }
    let len = polygon.vertices.len();
    for i in 0..len {
        let a = polygon.vertices[i];
        let b = polygon.vertices[(i + 1) % len];
        for j in (i + 1)..len {
            if j == i || j == (i + 1) % len || (j + 1) % len == i {
                continue;
            }
            let c = polygon.vertices[j];
            let d = polygon.vertices[(j + 1) % len];
            if segments_intersect(a, b, c, d) {
                return Err(MapProjectionError::InvalidPolygon);
            }
        }
    }
    Ok(())
}

fn orientation(a: MapPoint, b: MapPoint, c: MapPoint) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn segments_intersect(a: MapPoint, b: MapPoint, c: MapPoint, d: MapPoint) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);
    (ab_c.signum() != ab_d.signum() && cd_a.signum() != cd_b.signum())
        || (ab_c == 0.0 && on_segment(a, b, c))
        || (ab_d == 0.0 && on_segment(a, b, d))
        || (cd_a == 0.0 && on_segment(c, d, a))
        || (cd_b == 0.0 && on_segment(c, d, b))
}

fn on_segment(a: MapPoint, b: MapPoint, point: MapPoint) -> bool {
    point.x >= a.x.min(b.x)
        && point.x <= a.x.max(b.x)
        && point.y >= a.y.min(b.y)
        && point.y <= a.y.max(b.y)
}

fn point_in_polygon(point: MapPoint, polygon: &MapPolygon) -> bool {
    let mut inside = false;
    let mut previous = *polygon.vertices.last().expect("validated polygon");
    for current in &polygon.vertices {
        if orientation(previous, *current, point) == 0.0 && on_segment(previous, *current, point) {
            return true;
        }
        if (current.y > point.y) != (previous.y > point.y)
            && point.x
                < (previous.x - current.x) * (point.y - current.y) / (previous.y - current.y)
                    + current.x
        {
            inside = !inside;
        }
        previous = *current;
    }
    inside
}

fn distance_squared(a: MapPoint, b: MapPoint) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}
