//! Live consumers of resolved Build sources. Acquisition and exploration stay authoritative.

use super::{build_v6::PlayerProgressionV6, RuntimeState};
use crate::{
    capability_v1::CapabilityState,
    effects::{
        project_map_knowledge, CanonicalMap, CanonicalMapFeature, CanonicalMapRegion,
        MapExplorationState, MapKnowledgeProjection, MapPoint, MapPolygon,
    },
    player_rules::{KnowledgeChannel, KnowledgeLevel, MovementMode},
    scene_runtime::SceneDefinition,
    world_v3::{
        CapabilityProjection, ExploredMap, ExploredRoom, RearViewAuthorization, Vec3, WorldStateV3,
    },
};

impl RuntimeState {
    /// Resolve before committing; an invalid command cannot partially update a live consumer.
    pub(crate) fn install_build(&mut self, mut build: PlayerProgressionV6) -> Result<(), String> {
        build.sync_capabilities(&self.capabilities);
        let (sources, rules) =
            build.resolve_rules_with_capabilities(&self.capabilities, &self.world.rear_view)?;
        self.progression_v6 = build;
        self.effect_sources_v6 = sources;
        self.effective_rules_v6 = rules;
        self.refresh_build_movement_rules();
        Ok(())
    }

    /// Resolve candidate acquisition/selection and world authorization before committing either.
    pub(super) fn install_capability_state(
        &mut self,
        world: WorldStateV3,
        capabilities: CapabilityState,
    ) -> Result<(), String> {
        let mut build = self.progression_v6.clone();
        build.sync_capabilities(&capabilities);
        let (sources, rules) =
            build.resolve_rules_with_capabilities(&capabilities, &world.rear_view)?;
        self.world = world;
        self.capabilities = capabilities;
        self.progression_v6 = build;
        self.effect_sources_v6 = sources;
        self.effective_rules_v6 = rules;
        self.refresh_build_movement_rules();
        Ok(())
    }

    pub(super) fn install_build_command(
        &mut self,
        build: PlayerProgressionV6,
    ) -> Result<(), String> {
        let revision = self
            .world
            .revision
            .authority_revision
            .checked_add(1)
            .ok_or("E_WORLD_REVISION: RevisionExhausted")?;
        let build_revision = self
            .build_commands
            .revision
            .checked_add(1)
            .filter(|value| *value <= super::build_ui::MAX_SAFE_REVISION)
            .ok_or("E_BUILD_REVISION_EXHAUSTED")?;
        self.install_build(build)?;
        self.world.revision.authority_revision = revision;
        self.build_commands.revision = build_revision;
        Ok(())
    }

    pub(super) fn refresh_build_movement_rules(&mut self) {
        // Updating only rules preserves collision, closed gates and pump-resolved terrain tags.
        self.kcc
            .set_movement_rules(self.effective_rules_v6.clone(), MovementMode::Ground);
    }
}

fn point(position: Vec3) -> MapPoint {
    MapPoint {
        x: f64::from(position.x_m),
        y: f64::from(position.z_m),
    }
}

fn polygon(points: &[[f32; 2]]) -> MapPolygon {
    MapPolygon {
        vertices: points
            .iter()
            .map(|[x, z]| MapPoint {
                x: f64::from(*x),
                y: f64::from(*z),
            })
            .collect(),
    }
}

fn topology_region(id: String, polygon: MapPolygon) -> CanonicalMapRegion {
    CanonicalMapRegion {
        id,
        channel: KnowledgeChannel::Topology,
        polygon,
        terrain: None,
        boundary: None,
    }
}

fn feature(id: String, channel: KnowledgeChannel, position: MapPoint) -> CanonicalMapFeature {
    CanonicalMapFeature {
        id,
        channel,
        position,
        region_id: None,
        terrain: None,
        hazard: None,
        obstacle: None,
        boundary: None,
    }
}

/// Conservative containment: a convex explored region contains the whole terrain polygon.
/// Concave explored outlines fail closed rather than exposing an unexplored indentation.
fn convex_contains(outer: &MapPolygon, inner: &MapPolygon) -> bool {
    let vertices = &outer.vertices;
    if vertices.len() < 3 || inner.vertices.len() < 3 {
        return false;
    }
    let cross = |a: MapPoint, b: MapPoint, p: MapPoint| {
        (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
    };
    let mut direction = 0.0_f64;
    for i in 0..vertices.len() {
        let turn = cross(
            vertices[i],
            vertices[(i + 1) % vertices.len()],
            vertices[(i + 2) % vertices.len()],
        );
        if turn != 0.0 {
            if direction != 0.0 && turn.signum() != direction {
                return false;
            }
            direction = turn.signum();
        }
    }
    direction != 0.0
        && inner.vertices.iter().all(|p| {
            (0..vertices.len()).all(|i| {
                cross(vertices[i], vertices[(i + 1) % vertices.len()], *p) * direction >= 0.0
            })
        })
}

fn canonical_map(
    state: &RuntimeState,
    scene: Option<&SceneDefinition>,
) -> (CanonicalMap, MapExplorationState) {
    let mut canonical = CanonicalMap::default();
    let mut exploration = MapExplorationState {
        player_position: Some(point(state.world.player.position_m)),
        ..Default::default()
    };
    let matching_history = state.world.explored.world_id == state.world.world_id;
    if scene.is_none() && matching_history {
        for room in &state.world.explored.rooms {
            canonical.regions.push(topology_region(
                room.room_id.clone(),
                MapPolygon {
                    vertices: room
                        .outline_m
                        .iter()
                        .map(|position| point(*position))
                        .collect(),
                },
            ));
            exploration.explored_region_ids.insert(room.room_id.clone());
        }
    }
    if let Some(scene) = scene {
        // Current-scene canonical geometry only; never expose another world's scene registry.
        let regions = if scene.exploration_regions.is_empty() {
            &scene.walkable_polygons
        } else {
            &scene.exploration_regions
        };
        for region in regions {
            canonical
                .regions
                .push(topology_region(region.id.clone(), polygon(&region.polygon)));
            if matching_history
                && state
                    .world
                    .explored
                    .rooms
                    .iter()
                    .any(|room| room.room_id == region.id)
            {
                exploration.explored_region_ids.insert(region.id.clone());
            }
        }
        if canonical.regions.is_empty() {
            let b = &scene.bounds_m;
            canonical.regions.push(topology_region(
                format!("scene:{}", scene.scene_id),
                polygon(&[
                    [b.x, b.z],
                    [b.x + b.width, b.z],
                    [b.x + b.width, b.z + b.depth],
                    [b.x, b.z + b.depth],
                ]),
            ));
        }
        for region in &scene.terrain_regions {
            let shape = polygon(&region.polygon);
            let id = format!("terrain:{}", region.id);
            if canonical.regions.iter().any(|known| {
                known.channel == KnowledgeChannel::Topology
                    && exploration.explored_region_ids.contains(&known.id)
                    && convex_contains(&known.polygon, &shape)
            }) {
                exploration.explored_region_ids.insert(id.clone());
            }
            canonical.regions.push(CanonicalMapRegion {
                id,
                channel: KnowledgeChannel::Terrain,
                polygon: shape,
                terrain: Some(state.world_persistent_v1.resolve_terrain_tag(
                    &scene.world_id,
                    &region.id,
                    region.tag,
                )),
                boundary: None,
            });
        }
        for door in &scene.doors {
            canonical.features.push(feature(
                format!("door:{}", door.id),
                KnowledgeChannel::Connections,
                point(super::vec3_from_array(door.position)),
            ));
        }
        for transition in &scene.transitions {
            // Only connection geometry is disclosed, never destination, event or unlock data.
            let mut position = MapPoint { x: 0.0, y: 0.0 };
            for [x, z] in &transition.polygon {
                position.x += f64::from(*x);
                position.y += f64::from(*z);
            }
            let len = transition.polygon.len() as f64;
            position.x /= len;
            position.y /= len;
            canonical.features.push(feature(
                format!("transition:{}", transition.id),
                KnowledgeChannel::Connections,
                position,
            ));
        }
    }
    if matching_history {
        for objective in &state.world.explored.objectives {
            let position = match scene {
                Some(scene) => {
                    let Some(authored) = scene
                        .objectives
                        .iter()
                        .find(|entry| entry.id == objective.objective_id)
                    else {
                        continue;
                    };
                    point(super::vec3_from_array(authored.position))
                }
                None => point(objective.position_m),
            };
            let id = format!("objective:{}", objective.objective_id);
            canonical
                .features
                .push(feature(id.clone(), KnowledgeChannel::Objectives, position));
            exploration.known_feature_ids.insert(id);
        }
    }
    // Assign the existing, actual exploration region to point features. No remote detection
    // or equipment projection mutates WorldPersistentState or the durable ExploredMap.
    for item in &mut canonical.features {
        item.region_id = canonical
            .regions
            .iter()
            .find(|region| {
                region.channel == KnowledgeChannel::Topology
                    && super::point_in_or_on_polygon(
                        item.position.x as f32,
                        item.position.y as f32,
                        &region
                            .polygon
                            .vertices
                            .iter()
                            .map(|p| [p.x as f32, p.y as f32])
                            .collect::<Vec<_>>(),
                    )
            })
            .map(|region| region.id.clone());
    }
    (canonical, exploration)
}

pub(super) fn project_information(
    state: &RuntimeState,
    scene: Option<&SceneDefinition>,
    projection: &mut CapabilityProjection,
) {
    if !state.effective_rules_v6.perception.rear_view {
        projection.rear_view = RearViewAuthorization::denied();
    } else if !projection.rear_view.granted || projection.rear_view.granted_at_revision.is_none() {
        projection.rear_view =
            RearViewAuthorization::granted("effective.rear_view", state.world.revision)
                .expect("constant effective authorization id");
    }
    let rules = &state.effective_rules_v6;
    let (canonical, exploration) = canonical_map(state, scene);
    // Invalid geometry is all-or-nothing, never a partial information disclosure.
    let matching_scene = scene.is_none_or(|scene| {
        scene.world_id == state.world.world_id && scene.scene_id == state.world.scene_id
    });
    let knowledge = if matching_scene {
        project_map_knowledge(&canonical, &exploration, rules).unwrap_or_default()
    } else {
        MapKnowledgeProjection::default()
    };
    let rooms: Vec<_> = knowledge
        .regions
        .iter()
        .filter(|region| region.channel == KnowledgeChannel::Topology)
        .map(|region| ExploredRoom {
            room_id: region.id.clone(),
            outline_m: region
                .polygon
                .vertices
                .iter()
                .map(|p| Vec3 {
                    x_m: p.x as f32,
                    y_m: 0.0,
                    z_m: p.y as f32,
                })
                .collect(),
        })
        .collect();
    let connections = if state.world.explored.world_id == state.world.world_id
        && matches!(
            rules.map.level(KnowledgeChannel::Connections),
            KnowledgeLevel::ExploredOnly | KnowledgeLevel::Known | KnowledgeLevel::Full
        ) {
        state
            .world
            .explored
            .connections
            .iter()
            .filter(|link| {
                rooms.iter().any(|room| room.room_id == link.from_room_id)
                    && rooms.iter().any(|room| room.room_id == link.to_room_id)
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let objectives = knowledge
        .features
        .iter()
        .filter(|item| item.channel == KnowledgeChannel::Objectives)
        .filter_map(|item| {
            Some(crate::world_v3::KnownObjective {
                objective_id: item.id.as_deref()?.strip_prefix("objective:")?.into(),
                position_m: Vec3 {
                    x_m: item.position.x as f32,
                    y_m: 0.0,
                    z_m: item.position.y as f32,
                },
            })
        })
        .collect();
    projection.explored_map = ExploredMap::new(
        state.world.world_id.clone(),
        state.world.player.position_m,
        rooms,
        connections,
        objectives,
    )
    .expect("validated world id");
    projection.map_topology_authorized =
        Some(matching_scene && rules.map.level(KnowledgeChannel::Topology) != KnowledgeLevel::None);
    projection.map_knowledge = Some(knowledge);
}

#[cfg(test)]
mod tests;
