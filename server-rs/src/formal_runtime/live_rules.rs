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

// This is a map-output frame only. Saved observations remain in their original
// provider frames and unsupported or legacy scenes retain their existing view.
fn grey_hive_map_origin(scene: &SceneDefinition) -> Option<(f64, f64)> {
    if scene.world_id == "grey_hive" {
        if let Some(origin)=grey_hive_unmapped_origin(&scene.scene_id) { return Some(origin); }
    }
    if super::dense_service_wing::admitted(scene) {
        return Some((if scene.scene_id == "gh_gate_a" { 20.0 } else { 0.0 }, 0.0));
    }
    if super::bio_info::admitted(scene) { return Some((88.0, -1.0)); }
    if super::decon_info::admitted(scene) { return Some((136.0, -1.0)); }
    if super::beacon_exit_info::admitted(scene) {
        return Some((if scene.scene_id == "gh_exit" { 208.0 } else { 184.0 }, -1.0));
    }
    None
}

fn grey_hive_unmapped_origin(id: &str) -> Option<(f64, f64)> {
    match id {
        "gh_entry_maintenance" => Some((-20.0, 0.0)),
        "gh_central_shaft" => Some((40.0, -1.0)),
        "gh_lockdown" => Some((64.0, -1.0)),
        "gh_gate_b" => Some((112.0, -1.0)),
        "gh_sentinel_arena" => Some((160.0, -1.0)),
        _ => None,
    }
}

fn grey_hive_unmapped_scene(scene: &SceneDefinition) -> bool {
    scene.world_id == "grey_hive" && grey_hive_unmapped_origin(&scene.scene_id).is_some()
}

fn upper_history_layer_allowed(world:&str,scene:&str,floor_key:Option<&str>)->bool {
    world!="grey_hive" || scene!="gh_central_shaft" || floor_key.is_some_and(|key|key.starts_with("shaft_"))
}

fn append_grey_hive_history(canonical: &mut CanonicalMap, exploration: &mut MapExplorationState, rooms: &[ExploredRoom], open: bool) {
    let mut add = |room: &ExploredRoom, x: f64, z: f64| {
        if canonical.regions.iter().any(|region| region.id == room.room_id) { return; }
        canonical.regions.push(topology_region(room.room_id.clone(), MapPolygon {
            vertices: room.outline_m.iter().map(|p| MapPoint {
                x: f64::from(p.x_m) + x, y: f64::from(p.z_m) + z,
            }).collect(),
        }));
        exploration.explored_region_ids.insert(room.room_id.clone());
    };
    for room in rooms {
        if super::dense_service_wing::valid_stored_room(room)
            && rooms.iter().filter(|r|r.room_id==room.room_id).count()==1 {
            add(room,0.0,0.0);
        }
    }
    for id in super::bio_info::stored_regions(rooms) {
        if let Some(parts)=super::bio_info::rows(&id) {
            for part in &parts { add(part,88.0,-1.0); }
        }
    }
    for id in super::decon_info::stored_regions(rooms) {
        if let Some(parts)=super::decon_info::rows(&id) {
            for part in &parts { add(part,136.0,-1.0); }
        }
    }
    for scene_id in ["gh_beacon", "gh_exit"] {
        for id in super::beacon_exit_info::stored_regions(scene_id,open,rooms) {
            if let Some(parts)=super::beacon_exit_info::rows(scene_id,&id) {
                for part in &parts { add(part,184.0,-1.0); }
            }
        }
    }
}

fn append_floor_route_history(canonical:&mut CanonicalMap, exploration:&mut MapExplorationState,
    rooms:&[ExploredRoom], current_scene:&str, current_key:Option<&str>, lockdown_open:bool,
    shaft_observed:Option<&crate::world_v3::ObservedMapVariant>, lockdown_observed:Option<&crate::world_v3::ObservedMapVariant>) {
    let mut keys=vec![("entry",-20.0,0.0),(if lockdown_open {"gate_b_open"} else {"gate_b_closed"},112.0,-1.0),("sentinel",160.0,-1.0)];
    if current_scene=="gh_central_shaft" {
        if let Some(key)=current_key.filter(|key|key.starts_with("shaft_")) {keys.push((key,40.0,-1.0));}
    } else if shaft_observed.and_then(|tag|tag.trusted_family())==Some("shaft_active") {
        // Landing phase is unknown off-scene; never infer an open threshold.
        keys.push(("shaft_active_closed",40.0,-1.0));
    }
    if current_scene=="gh_lockdown" {
        if let Some(key)=current_key.filter(|key|key.starts_with("lockdown_")) {keys.push((key,64.0,-1.0));}
    } else {
        match lockdown_observed.and_then(|tag|tag.trusted_family()) {
            Some("lockdown")=>keys.push((if lockdown_open {"lockdown_open"} else {"lockdown_closed"},64.0,-1.0)),
            Some("lockdown_legacy")=>keys.push((if lockdown_open {"lockdown_legacy_open"} else {"lockdown_legacy_closed"},64.0,-1.0)),
            _=>{}
        }
    }
    for (key,x,z) in keys {
        for id in super::floor_route_info::stored_regions(key,rooms) {
            if let Some(parts)=super::floor_route_info::parts(key,&id) {
                for (i,part) in parts.iter().enumerate() {
                    let part_id=format!("{id}:part{i}");
                    if canonical.regions.iter().any(|region|region.id==part_id){continue;}
                    canonical.regions.push(topology_region(part_id.clone(),MapPolygon{vertices:part.iter().map(|p|
                        MapPoint{x:f64::from(p[0])+x,y:f64::from(p[1])+z}).collect()}));
                    exploration.explored_region_ids.insert(part_id);
                }
            }
        }
    }
}

// Output-only shared frame. Stored source outlines remain scene-local.
fn mhcw_canonical_map(state:&RuntimeState,scene:&SceneDefinition)->(CanonicalMap,MapExplorationState) {
    let mut canonical=CanonicalMap::default();let mut exploration=MapExplorationState::default();
    let Some(current)=super::mhcw_history_group(state,scene) else{return (canonical,exploration);};
    let (Some(frame),Some(layer))=(current["frame_id"].as_str(),current["layer_id"].as_str()) else{return (canonical,exploration);};
    let origin=|g:&serde_json::Value|->Option<(f64,f64)>{Some((g["origin_xz"][0].as_f64()?,g["origin_xz"][1].as_f64()?))};
    let Some((cx,cz))=origin(current) else{return (canonical,exploration);};
    exploration.player_position=Some(MapPoint{x:f64::from(state.world.player.position_m.x_m)+cx,y:f64::from(state.world.player.position_m.z_m)+cz});
    if state.world.explored.world_id!=state.world.world_id {return (canonical,exploration);}
    if let Some(groups)=super::mhcw_map_region::groups(){for g in groups {
        if g["world_id"].as_str()!=Some(state.world.world_id.as_str()) || g["frame_id"].as_str()!=Some(frame) || g["layer_id"].as_str()!=Some(layer){continue;}
        let Some((ox,oz))=origin(g) else{continue;};
        let history=super::mhcw_map_region::history_rows(state,g);
        for id in super::mhcw_map_region::stored(g,&history) {
            let Some(rows)=super::mhcw_map_region::rows(g,&id) else{continue;};
            for row in rows {
                let vertices=row.outline_m.iter().map(|p|MapPoint{x:f64::from(p.x_m)+ox,y:f64::from(p.z_m)+oz}).collect();
                exploration.explored_region_ids.insert(row.room_id.clone());
                canonical.regions.push(topology_region(row.room_id,MapPolygon{vertices}));
            }
        }
    }}
    for terrain in &scene.terrain_regions {
        let mut shape=polygon(&terrain.polygon);for p in &mut shape.vertices {p.x+=cx;p.y+=cz;}
        if canonical.regions.iter().any(|r|r.channel==KnowledgeChannel::Topology && exploration.explored_region_ids.contains(&r.id) && convex_contains(&r.polygon,&shape)) {
            let id=format!("terrain:{}",terrain.id);exploration.explored_region_ids.insert(id.clone());
            canonical.regions.push(CanonicalMapRegion{id,channel:KnowledgeChannel::Terrain,polygon:shape,
                terrain:Some(state.world_persistent_v1.resolve_terrain_tag(&scene.world_id,&terrain.id,terrain.tag)),boundary:None});
        }
    }
    for known in &state.world.explored.objectives {
        let Some(authored)=scene.objectives.iter().find(|o|o.id==known.objective_id) else{continue;};
        let position=MapPoint{x:f64::from(authored.position[0])+cx,y:f64::from(authored.position[2])+cz};
        let id=format!("objective:{}",known.objective_id);canonical.features.push(feature(id.clone(),KnowledgeChannel::Objectives,position));exploration.known_feature_ids.insert(id);
    }
    (canonical,exploration)
}
fn canonical_map(
    state: &RuntimeState,
    scene: Option<&SceneDefinition>,
    dense_visible: Option<&str>,
    floor_route_key: Option<&str>,
) -> (CanonicalMap, MapExplorationState) {
    if let Some(definition)=scene {
        if super::mhcw_map_region::group(definition,"ground_y0").is_some() {return mhcw_canonical_map(state,definition);}
    }
    let mut canonical = CanonicalMap::default();
    let mut exploration = MapExplorationState {
        player_position: Some(point(state.world.player.position_m)),
        ..Default::default()
    };
    let matching_history = state.world.explored.world_id == state.world.world_id;
    let gh_history=state.world_persistent_v1.map_knowledge_archive_v2.is_some()
        && scene.is_some_and(|d|super::gh_map_archive::committed_frame(state,d,floor_route_key));
    let history_rooms=if gh_history {super::gh_map_archive::history_rows(state)} else {state.world.explored.rooms.clone()};
    if scene.is_none() && matching_history {
        for room in &history_rooms {
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
        let dense_scope = scene.world_id == "grey_hive" && ["gh_power_room", "gh_gate_a"].contains(&scene.scene_id.as_str());
        if let Some(key)=floor_route_key {
            let mut ids=if matching_history {super::floor_route_info::stored_regions(key,&history_rooms)} else {Vec::new()};
            if let Some(id)=dense_visible {if !ids.iter().any(|known|known==id){ids.push(id.to_owned());}}
            for id in ids {if let Some(parts)=super::floor_route_info::parts(key,&id) {
                for (i,part) in parts.iter().enumerate() {
                    let part_id=format!("{id}:part{i}");
                    canonical.regions.push(topology_region(part_id.clone(),polygon(part)));
                    exploration.explored_region_ids.insert(part_id);
                }
            }}
            if super::shaft_lower_service::history_ready(state,scene,Some(key)) {
                for row in super::shaft_lower_service::history_rows(state) {
                    let shape=MapPolygon{vertices:row.outline_m.iter().map(|p|point(*p)).collect()};
                    exploration.explored_region_ids.insert(row.room_id.clone());
                    canonical.regions.push(topology_region(row.room_id,shape));
                }
            }
        } else if super::bio_info::admitted(scene) {
            let mut ids=if matching_history {super::bio_info::stored_regions(&history_rooms)} else {Vec::new()};
            if let Some(id)=dense_visible {
                if !ids.iter().any(|known|known==id) {ids.push(id.to_owned());}
            }
            for id in ids {
                if let Some(parts)=super::bio_info::rows(&id) {
                    for part in parts {
                        let shape=MapPolygon{vertices:part.outline_m.iter().map(|p|point(*p)).collect()};
                        exploration.explored_region_ids.insert(part.room_id.clone());
                        canonical.regions.push(topology_region(part.room_id,shape));
                    }
                }
            }
        } else if super::beacon_exit_info::admitted(scene) {
            let open=super::completed_world_events(state,"grey_hive").contains("hive_extraction");
            let mut ids=if matching_history {super::beacon_exit_info::stored_regions(&scene.scene_id,open,&history_rooms)} else {Vec::new()};
            if let Some(id)=dense_visible {
                if !ids.iter().any(|known|known==id) {ids.push(id.to_owned());}
            }
            let offset=if scene.scene_id=="gh_exit" {24.0} else {0.0};
            for id in ids {
                // The current phase decides availability; a closed passage is retained
                // in storage but is never projected as open floor.
                if super::beacon_exit_info::parts(&scene.scene_id,&id,open,false).is_none(){continue;}
                if let Some(parts)=super::beacon_exit_info::rows(&scene.scene_id,&id) {
                    for part in parts {
                        let shape=MapPolygon{vertices:part.outline_m.iter().map(|p|MapPoint{x:f64::from(p.x_m-offset),y:f64::from(p.z_m)}).collect()};
                        exploration.explored_region_ids.insert(part.room_id.clone());
                        canonical.regions.push(topology_region(part.room_id,shape));
                    }
                }
            }
        } else if super::decon_info::admitted(scene) {
            let mut ids=if matching_history {super::decon_info::stored_regions(&history_rooms)} else {Vec::new()};
            if let Some(id)=dense_visible {
                if !ids.iter().any(|known|known==id) {ids.push(id.to_owned());}
            }
            for id in ids {
                if let Some(parts)=super::decon_info::rows(&id) {
                    for part in parts {
                        let shape=MapPolygon{vertices:part.outline_m.iter().map(|p|point(*p)).collect()};
                        exploration.explored_region_ids.insert(part.room_id.clone());
                        canonical.regions.push(topology_region(part.room_id,shape));
                    }
                }
            }
        } else if dense_scope {
            // The observation is transient and uses the same authoritative occupied result.
            if let Some((id, shape)) = dense_visible.and_then(|id|
                super::dense_service_wing::region_polygon(scene, id).map(|shape| (id, shape))) {
                canonical.regions.push(topology_region(id.to_owned(), polygon(&shape)));
                exploration.explored_region_ids.insert(id.to_owned());
            }
            // Saved dense polygons use the explicit shared Power/Gate frame. Only
            // exact known IDs and geometry are admitted as historical observations.
            if matching_history {
                let offset = if scene.scene_id == "gh_gate_a" { 20.0 } else { 0.0 };
                for room in &history_rooms {
                    if !super::dense_service_wing::valid_stored_room(room)
                        || history_rooms.iter().filter(|r|r.room_id==room.room_id).count()!=1
                        || canonical.regions.iter().any(|region| region.id == room.room_id) { continue; }
                    let vertices = room.outline_m.iter().map(|p| MapPoint {
                        x:f64::from(p.x_m-offset), y:f64::from(p.z_m),
                    }).collect();
                    canonical.regions.push(topology_region(room.room_id.clone(), MapPolygon { vertices }));
                    exploration.explored_region_ids.insert(room.room_id.clone());
                }
            }
        } else if !grey_hive_unmapped_scene(scene) {
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
        }
        if !grey_hive_unmapped_scene(scene) {
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
    }
    if matching_history {
        for objective in &state.world.explored.objectives {
            let position = match scene {
                Some(scene) => {
                    if grey_hive_unmapped_scene(scene) { continue; }
                    let Some(authored) = scene
                        .objectives
                        .iter()
                        .find(|entry| entry.id == objective.objective_id)
                    else {
                        continue;
                    };
                    if floor_route_key.is_some_and(|key|key.starts_with("shaft_lower_")) && authored.position[1]!=-4.0 {continue;}
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
    // Map projection uses one shared coordinate frame; gameplay and saves keep
    // their original scene-local player position.
    // The lower Shaft has its own scene-local layer and is never offset into
    // the horizontal upper-floor atlas.
    let origin = if floor_route_key.is_some_and(|key|key.starts_with("shaft_lower_")) {Some((0.0,0.0))}
        else {scene.and_then(grey_hive_map_origin)};
    let (map_x,map_z)=origin.unwrap_or_else(|| (scene.map_or(0.0,|s|
        if s.world_id=="grey_hive" && s.scene_id=="gh_gate_a" {20.0} else {0.0}),0.0));
    if map_x!=0.0 || map_z!=0.0 {
        for region in &mut canonical.regions {
            for p in &mut region.polygon.vertices { p.x += map_x; p.y += map_z; }
        }
        for feature in &mut canonical.features { feature.position.x += map_x; feature.position.y += map_z; }
        if let Some(position) = &mut exploration.player_position { position.x += map_x; position.y += map_z; }
    }
    if origin.is_some() && matching_history && !floor_route_key.is_some_and(|key|key.starts_with("shaft_lower_")) {
        // Only complete, exact provider-owned records are projected. They are
        // never written back in this output frame, and no neighbor is visited here.
        let open=super::completed_world_events(state,"grey_hive").contains("hive_extraction");
        append_grey_hive_history(&mut canonical,&mut exploration,&history_rooms,open);
        let lockdown_open=super::completed_world_events(state,"grey_hive").contains("hive_lockdown");
        let archive=if gh_history {state.world_persistent_v1.map_knowledge_archive_v2.as_ref()} else {None};
        append_floor_route_history(&mut canonical,&mut exploration,&history_rooms,
            &state.world.scene_id,floor_route_key,lockdown_open,
            state.world.shaft_upper_map_observation.as_ref().or_else(||archive.and_then(|a|a.shaft_upper_observation.as_ref())),
            state.world.lockdown_map_observation.as_ref().or_else(||archive.and_then(|a|a.lockdown_observation.as_ref())));
    }
    (canonical, exploration)
}

pub(super) fn project_information(
    state: &RuntimeState,
    scene: Option<&SceneDefinition>,
    dense_visible: Option<&str>,
    floor_route_key: Option<&str>,
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
    let (canonical, exploration) = canonical_map(state, scene, dense_visible, floor_route_key);
    // Invalid geometry is all-or-nothing, never a partial information disclosure.
    let dense_without_scene = scene.is_none() && state.world.world_id == "grey_hive"
        && ["gh_entry_maintenance", "gh_power_room", "gh_gate_a", "gh_central_shaft", "gh_lockdown", "gh_bio_isolation", "gh_gate_b", "gh_deep_decon", "gh_sentinel_arena", "gh_beacon", "gh_exit"].contains(&state.world.scene_id.as_str());
    let matching_scene = !dense_without_scene && scene.is_none_or(|scene| {
        let dense_scope = scene.world_id == "grey_hive"
            && ["gh_power_room", "gh_gate_a"].contains(&scene.scene_id.as_str());
        let decon = super::decon_info::admitted(scene);
        let bio = super::bio_info::admitted(scene);
        let pair = super::beacon_exit_info::admitted(scene);
        let open=super::completed_world_events(state,"grey_hive").contains("hive_extraction");
        let gh_history=state.world_persistent_v1.map_knowledge_archive_v2.is_some()
            && super::gh_map_archive::committed_frame(state,scene,floor_route_key);
        scene.world_id == state.world.world_id && scene.scene_id == state.world.scene_id
            // The durable history is an upper-floor frame. An unqualified
            // Shaft height/support mode cannot project that frame as current.
            && upper_history_layer_allowed(&scene.world_id,&scene.scene_id,floor_route_key)
            && (super::mhcw_map_region::group(scene,"ground_y0").is_none() || super::mhcw_history_group(state,scene).is_some())
            && (!dense_scope || gh_history || dense_visible.is_some_and(|id|
                super::dense_service_wing::region_polygon(scene, id).is_some()))
            && (!decon || gh_history || dense_visible.is_some_and(|id|super::decon_info::parts(id).is_some()))
            && (!bio || gh_history || dense_visible.is_some_and(|id|super::bio_info::parts(id).is_some()))
            && (!pair || gh_history || dense_visible.is_some_and(|id|super::beacon_exit_info::parts(&scene.scene_id,id,open,false).is_some()))
            && (floor_route_key.is_none() || gh_history || super::shaft_lower_service::history_ready(state,scene,floor_route_key) || dense_visible.is_some_and(|id|super::floor_route_info::parts(floor_route_key.unwrap(),id).is_some()))
    });
    let knowledge = if matching_scene {
        project_map_knowledge(&canonical, &exploration, rules).unwrap_or_default()
    } else {
        MapKnowledgeProjection::default()
    };
    let map_y=if scene.and_then(|d|super::mhcw_history_contact(state,d)).is_some_and(|c|c.layer=="gear_static_decks_y2") {2.0} else if floor_route_key.is_some_and(|key|key.starts_with("shaft_lower_")) {-4.0} else {0.0};
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
                    y_m: map_y,
                    z_m: p.y as f32,
                })
                .collect(),
        })
        .collect();
    let authorized=matching_scene && rules.map.level(KnowledgeChannel::Topology) != KnowledgeLevel::None;
    projection.map_current_room_ids=None;projection.map_explored_room_ids=None;
    if authorized {
        if let Some(d)=scene {
            let mhcw=super::mhcw_for_definition(state,d);
            let mhcw_history=super::mhcw_history_group(state,d);
            let supported=mhcw_history.is_some() || floor_route_key.is_some() || super::dense_service_wing::admitted(d)
                || super::bio_info::admitted(d) || super::decon_info::admitted(d) || super::beacon_exit_info::admitted(d)
                || !d.exploration_regions.is_empty();
            if supported {
                let current:Vec<String>=if let Some(c)=mhcw {
                    super::mhcw_map_region::group(d,c.layer).and_then(|g|super::mhcw_map_region::rows(g,&c.region)).unwrap_or_default().into_iter().map(|r|r.room_id).collect()
                } else if let Some(id)=dense_visible {
                    if super::shaft_lower_service::history_ready(state,d,floor_route_key) && id.starts_with("grey_hive.gh_central_shaft.lower_service_v2.") {
                        super::shaft_lower_service::candidate_rows(id).unwrap_or_default().into_iter().map(|r|r.room_id).collect()
                    } else if let Some(key)=floor_route_key {
                        super::floor_route_info::rows(key,id).unwrap_or_default().into_iter().map(|r|r.room_id).collect()
                    } else if super::bio_info::admitted(d) {super::bio_info::rows(id).unwrap_or_default().into_iter().map(|r|r.room_id).collect()}
                    else if super::decon_info::admitted(d) {super::decon_info::rows(id).unwrap_or_default().into_iter().map(|r|r.room_id).collect()}
                    else if super::beacon_exit_info::admitted(d) {super::beacon_exit_info::rows(&d.scene_id,id).unwrap_or_default().into_iter().map(|r|r.room_id).collect()}
                    else if super::dense_service_wing::region_polygon(d,id).is_some() {vec![id.to_owned()]} else {vec![]}
                } else if let Some(region) = d.exploration_regions.iter().find(|region| super::point_in_or_on_polygon(state.world.player.position_m.x_m, state.world.player.position_m.z_m, &region.polygon)) { vec![region.id.clone()] } else {vec![]};
                let displayed:std::collections::BTreeSet<_>=rooms.iter().map(|r|r.room_id.as_str()).collect();
                projection.map_current_room_ids=Some(current.into_iter().filter(|id|displayed.contains(id.as_str())).collect());
                // Reproject with no transient occupied parent: only provider-validated
                // durable groups can contribute history classification.
                let (history,observed)=canonical_map(state,Some(d),None,floor_route_key);
                let gh_rows=if state.world_persistent_v1.map_knowledge_archive_v2.is_some()
                    && super::gh_map_archive::committed_frame(state,d,floor_route_key){Some(super::gh_map_archive::history_rows(state))}else{None};
                let service_rows=if super::shaft_lower_service::history_ready(state,d,floor_route_key) {super::shaft_lower_service::history_rows(state)} else {Vec::new()};
                let mut past:Vec<String>=history.regions.into_iter().filter(|r|r.channel==KnowledgeChannel::Topology
                    && observed.explored_region_ids.contains(&r.id) && displayed.contains(r.id.as_str())
                    && (mhcw_history.is_some() || gh_rows.as_ref().is_some_and(|rows|rows.iter().filter(|stored|stored.room_id==r.id).count()==1)
                        || service_rows.iter().filter(|stored|stored.room_id==r.id).count()==1 || state.world.explored.rooms.iter().filter(|stored|stored.room_id==r.id).count()==1)).map(|r|r.id).collect();
                past.sort();past.dedup();projection.map_explored_room_ids=Some(past);
            }
        }
    }
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
                    y_m: map_y,
                    z_m: item.position.y as f32,
                },
            })
        })
        .collect();
    let mut map_position = state.world.player.position_m;
    if let Some(scene)=scene {
        let mhcw_origin=super::mhcw_history_group(state,scene).and_then(|g|Some((g["origin_xz"][0].as_f64()?,g["origin_xz"][1].as_f64()?)));
        let (x,z)=if let Some(origin)=mhcw_origin {origin} else if floor_route_key.is_some_and(|key|key.starts_with("shaft_lower_")) {(0.0,0.0)} else {
            grey_hive_map_origin(scene).unwrap_or_else(||
                (if scene.world_id=="grey_hive" && scene.scene_id=="gh_gate_a" {20.0} else {0.0},0.0))};
        map_position.x_m+=x as f32; map_position.z_m+=z as f32;
    }
    projection.explored_map = ExploredMap::new(
        state.world.world_id.clone(),
        map_position,
        rooms,
        connections,
        objectives,
    )
    .expect("validated world id");
    projection.map_topology_authorized =
        Some(matching_scene && rules.map.level(KnowledgeChannel::Topology) != KnowledgeLevel::None);
    projection.map_knowledge = Some(knowledge);
}

/// How far two region outlines may be apart and still count as touching.
const OPEN_BOUNDARY_GAP_M: f32 = 0.05;
/// A touching span shorter than this is a corner kiss, not an opening a player
/// can see through, so it must not make the neighbouring region visible.
const OPEN_BOUNDARY_MIN_M: f32 = 0.5;
/// Sampling step used to measure the touching span between two outlines.
const OPEN_BOUNDARY_SAMPLE_M: f32 = 0.25;

/// Worlds whose scene visibility is authored purely by exploration-region
/// geometry. The hive scenes publish it through their dense/service providers
/// and the harbour/clockworks scenes through the MH/CW contact provider, so
/// only the return station qualifies.
fn region_geometry_visibility_world(world_id: &str) -> bool {
    world_id == crate::world_persistent_v1::RETURN_STATION_ID
}

/// Cheap rejection: outlines whose bounding boxes are further apart than `gap`
/// cannot share a boundary.
fn polygon_bounds_gap_within(a: &[[f32; 2]], b: &[[f32; 2]], gap: f32) -> bool {
    fn bounds(points: &[[f32; 2]]) -> Option<(f32, f32, f32, f32)> {
        let mut bounds = (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for [x, z] in points {
            if !x.is_finite() || !z.is_finite() {
                return None;
            }
            bounds = (bounds.0.min(*x), bounds.1.min(*z), bounds.2.max(*x), bounds.3.max(*z));
        }
        Some(bounds)
    }
    let (Some((amin_x, amin_z, amax_x, amax_z)), Some((bmin_x, bmin_z, bmax_x, bmax_z))) =
        (bounds(a), bounds(b))
    else {
        return false;
    };
    let dx = (amin_x - bmax_x).max(bmin_x - amax_x).max(0.0);
    let dz = (amin_z - bmax_z).max(bmin_z - amax_z).max(0.0);
    (dx * dx + dz * dz).sqrt() <= gap
}

/// True when two region outlines share at least [`OPEN_BOUNDARY_MIN_M`] of
/// boundary within [`OPEN_BOUNDARY_GAP_M`], i.e. there is an opening a player
/// can see through. A wall between them leaves a gap wider than the tolerance.
fn regions_share_open_boundary(a: &[[f32; 2]], b: &[[f32; 2]]) -> bool {
    if a.len() < 3 || b.len() < 3 || !polygon_bounds_gap_within(a, b, OPEN_BOUNDARY_GAP_M) {
        return false;
    }
    let mut open = 0.0_f32;
    for index in 0..b.len() {
        let [ax, az] = b[index];
        let [bx, bz] = b[(index + 1) % b.len()];
        let length = ((bx - ax) * (bx - ax) + (bz - az) * (bz - az)).sqrt();
        if !length.is_finite() || length <= 0.0 {
            continue;
        }
        let steps = ((length / OPEN_BOUNDARY_SAMPLE_M).ceil() as usize).max(1);
        let step = length / steps as f32;
        for sample in 0..steps {
            let t = (sample as f32 + 0.5) / steps as f32;
            if super::polygon_edge_distance(ax + (bx - ax) * t, az + (bz - az) * t, a)
                <= OPEN_BOUNDARY_GAP_M
            {
                open += step;
                if open >= OPEN_BOUNDARY_MIN_M {
                    return true;
                }
            }
        }
    }
    false
}

/// Region ids visible from `(x, z)` under the authored-geometry rule: the region
/// the player occupies plus every region that shares an open boundary with it.
/// Standing on the wall band between regions falls back to the nearest authored
/// region instead of claiming that nothing is visible.
fn visible_region_ids(
    regions: &[crate::scene_runtime::ScenePolygon],
    x: f32,
    z: f32,
) -> Option<Vec<String>> {
    if regions.len() < 2 || !x.is_finite() || !z.is_finite() {
        return None;
    }
    let mut occupied: Vec<usize> = (0..regions.len())
        .filter(|index| super::point_in_or_on_polygon(x, z, &regions[*index].polygon))
        .collect();
    if occupied.is_empty() {
        let nearest = regions
            .iter()
            .enumerate()
            .map(|(index, region)| (index, super::polygon_edge_distance(x, z, &region.polygon)))
            .min_by(|a, b| a.1.total_cmp(&b.1))?;
        occupied.push(nearest.0);
    }
    let mut visible: Vec<String> = occupied
        .iter()
        .map(|index| regions[*index].id.clone())
        .collect();
    for index in 0..regions.len() {
        if occupied.contains(&index) {
            continue;
        }
        if occupied.iter().any(|here| {
            regions_share_open_boundary(&regions[*here].polygon, &regions[index].polygon)
        }) {
            visible.push(regions[index].id.clone());
        }
    }
    visible.sort();
    visible.dedup();
    Some(visible)
}

/// Authored-region visibility for the live scene of a world that has no
/// per-region visibility provider. `None` means this scene publishes no
/// region-geometry visibility, so every other world keeps its previous
/// projection byte for byte.
pub(super) fn project_authored_region_visibility(
    state: &RuntimeState,
    scene: &SceneDefinition,
) -> Option<Vec<String>> {
    if !region_geometry_visibility_world(&scene.world_id)
        || scene.world_id != state.world.world_id
        || scene.scene_id != state.world.scene_id
        || scene.exploration_regions.is_empty()
    {
        return None;
    }
    let position = state.world.player.position_m;
    visible_region_ids(&scene.exploration_regions, position.x_m, position.z_m)
}

#[cfg(test)]
mod region_visibility_tests {
    use super::*;

    fn rect(x0: f32, x1: f32, z0: f32, z1: f32) -> Vec<[f32; 2]> {
        vec![[x0, z0], [x1, z0], [x1, z1], [x0, z1]]
    }

    fn hub() -> crate::scene_runtime::SceneDefinition {
        serde_json::from_str(include_str!("../../../content/scenes/compiled/rs_core_room.json"))
            .unwrap()
    }

    #[test]
    fn open_boundary_counts_and_a_wall_between_regions_does_not() {
        assert!(regions_share_open_boundary(
            &rect(0.5, 8.0, 0.5, 15.5),
            &rect(8.0, 16.0, 6.0, 10.0)
        ));
        // The hub's 0.5 m wall band keeps two regions out of sight of each other.
        assert!(!regions_share_open_boundary(
            &rect(0.5, 8.0, 0.5, 15.5),
            &rect(8.5, 16.0, 0.5, 15.5)
        ));
        // Touching at one corner is not an opening.
        assert!(!regions_share_open_boundary(
            &rect(0.0, 8.0, 0.0, 8.0),
            &rect(8.0, 16.0, 8.0, 16.0)
        ));
        assert!(!regions_share_open_boundary(
            &rect(0.0, 8.0, 0.0, 8.0),
            &rect(20.0, 28.0, 0.0, 8.0)
        ));
    }

    #[test]
    fn hub_visibility_is_current_plus_open_neighbours_never_all_or_nothing() {
        let scene = hub();
        assert_eq!(scene.scene_id, "rs_core_room");
        assert_eq!(scene.exploration_regions.len(), 5);
        // Standing in the west hall: its three open neighbours follow, the gate
        // behind the centre block does not, so the set is neither all nor empty.
        let west = visible_region_ids(&scene.exploration_regions, 4.0, 8.0).unwrap();
        assert_eq!(
            west,
            vec![
                "rs_core_center",
                "rs_core_north",
                "rs_core_south",
                "rs_core_west"
            ]
        );
        // Standing at the gate: the west hall drops out, the gate and its three
        // neighbours stay.
        let gate = visible_region_ids(&scene.exploration_regions, 20.0, 8.0).unwrap();
        assert_eq!(
            gate,
            vec![
                "rs_core_center",
                "rs_core_gate",
                "rs_core_north",
                "rs_core_south"
            ]
        );
        assert_ne!(west, gate, "visibility must move with the player");
        // A position on the wall band is still answered, never empty.
        let band = visible_region_ids(&scene.exploration_regions, 8.0, 8.0).unwrap();
        assert!(!band.is_empty());
    }

    #[test]
    fn only_the_authored_region_world_publishes_geometry_visibility() {
        assert!(region_geometry_visibility_world("return_station"));
        assert!(!region_geometry_visibility_world("grey_hive"));
        assert!(!region_geometry_visibility_world("mist_harbor"));
        assert!(!region_geometry_visibility_world("clockworks"));
    }
}

#[cfg(test)]
mod whole_floor_map_tests {
    use super::*;

    #[test]
    fn gear_archive_never_guesses_a_layer_without_contact() {
        let mut state=super::super::FormalRuntime::initial_state(1).unwrap();
        state.world=crate::world_v3::WorldStateV3::new("clockworks",1,Vec3::new(16.,2.,8.).unwrap()).unwrap();
        state.world.scene_id="cw_gear_shaft".into();state.paused=false;state.pending_entry=None;state.pending_exit=None;
        let group=super::super::mhcw_map_region::groups().unwrap().iter().find(|g|g["scene_id"]=="cw_gear_shaft" && g["layer_id"]=="gear_static_decks_y2").unwrap();
        let id=group["parents"][0]["parent_id"].as_str().unwrap();super::super::mhcw_map_region::archive_parent(&mut state.world_persistent_v1,group,id).unwrap();
        let mut scene:SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/cw_gear_shaft.json")).unwrap();scene.verified_source_sha256=group["selected_sha_declared"].as_str().unwrap().into();
        assert!(super::super::mhcw_history_group(&state,&scene).is_none());
        let (map,_)=mhcw_canonical_map(&state,&scene);assert!(map.regions.is_empty());
        assert_eq!(state.world_persistent_v1.map_knowledge_archive_v1.as_ref().unwrap().records.len(),1);
    }

    #[test]
    fn mhcw_history_projects_complete_local_rows_once_without_mutating_save() {
        let mut state=super::super::FormalRuntime::initial_state(1).unwrap();
        state.world=crate::world_v3::WorldStateV3::new("mist_harbor",1,Vec3::new(4.0,0.0,8.0).unwrap()).unwrap();
        state.world.scene_id="mh_tidal_warehouse".into();state.paused=false;state.pending_entry=None;state.pending_exit=None;
        let groups=super::super::mhcw_map_region::groups().unwrap();
        let prior=groups.iter().find(|g|g["scene_id"]=="mh_fog_pier").unwrap();
        let current=groups.iter().find(|g|g["scene_id"]=="mh_tidal_warehouse").unwrap();
        let prior_id=prior["parents"][0]["parent_id"].as_str().unwrap();
        let current_id=current["parents"][0]["parent_id"].as_str().unwrap();
        state.world.explored.rooms.extend(super::super::mhcw_map_region::rows(prior,prior_id).unwrap());
        state.world.explored.rooms.extend(super::super::mhcw_map_region::rows(current,current_id).unwrap());
        let saved=state.world.explored.rooms.clone();
        let mut scene:SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/mh_tidal_warehouse.json")).unwrap();
        scene.verified_source_sha256=current["selected_sha_declared"].as_str().unwrap().into();
        state.mhcw_contact=Some(super::super::MhcwContact{world_id:scene.world_id.clone(),scene_id:scene.scene_id.clone(),scene_sha:scene.verified_source_sha256.clone(),revision:state.world.revision,server_time_ms:state.world.server_time_ms,body:state.world.player.clone(),layer:"ground_y0",region:current_id.into()});
        let (map,explored)=mhcw_canonical_map(&state,&scene);
        assert_eq!(map.regions.len(),2);assert_eq!(explored.player_position.unwrap().x,28.0);
        let warehouse=map.regions.iter().find(|r|r.id.starts_with(current_id)).unwrap();
        assert_eq!(warehouse.polygon.vertices[0].x,f64::from(saved[1].outline_m[0].x_m)+24.0);
        assert_eq!(state.world.explored.rooms,saved);
        let rev=state.world.revision;
        crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Grant{capability_id:crate::capability_v1::CAP_LOCAL_MAP.into()},rev).unwrap();
        crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Select{capability_ids:vec![crate::capability_v1::CAP_LOCAL_MAP.into()]},rev).unwrap();
        let build=state.progression_v6.clone();state.install_build(build).unwrap();
        let mut output=CapabilityProjection::new(vec![],state.world.explored.clone(),vec![],RearViewAuthorization::denied());
        project_information(&state,Some(&scene),Some(current_id),None,&mut output);
        assert_eq!(output.map_current_room_ids,Some(vec![saved[1].room_id.clone()]));
        assert_eq!(output.map_explored_room_ids.as_ref().unwrap().len(),2);
        assert!(output.map_explored_room_ids.as_ref().unwrap().contains(&saved[0].room_id));
        assert_eq!(state.world.explored.rooms,saved);
        super::super::mhcw_map_region::archive_parent(&mut state.world_persistent_v1,prior,prior_id).unwrap();
        super::super::mhcw_map_region::archive_parent(&mut state.world_persistent_v1,current,current_id).unwrap();
        state.world.explored.rooms.clear();state.mhcw_contact=None;
        project_information(&state,Some(&scene),None,None,&mut output);
        assert_eq!(output.map_topology_authorized,Some(true));assert_eq!(output.explored_map.rooms.len(),2);
        assert_eq!(output.map_current_room_ids,Some(vec![]));assert_eq!(output.map_explored_room_ids.as_ref().unwrap().len(),2);
        assert!(state.world.explored.rooms.is_empty());
        state.paused=true;project_information(&state,Some(&scene),None,None,&mut output);assert_eq!(output.map_topology_authorized,Some(false));
        state.paused=false;state.world.explored.rooms=saved.clone();
        state.mhcw_contact=Some(super::super::MhcwContact{world_id:scene.world_id.clone(),scene_id:scene.scene_id.clone(),scene_sha:scene.verified_source_sha256.clone(),revision:state.world.revision,server_time_ms:state.world.server_time_ms,body:state.world.player.clone(),layer:"ground_y0",region:current_id.into()});
        state.world.explored.rooms.push(saved[1].clone());
        let (bad,_)=mhcw_canonical_map(&state,&scene);assert_eq!(bad.regions.len(),1);
        project_information(&state,Some(&scene),Some(current_id),None,&mut output);
        assert!(!output.map_explored_room_ids.as_ref().unwrap().contains(&saved[1].room_id));
        let rev=state.world.revision;
        crate::capability_v1::apply_command_at_revision(&mut state.capabilities,crate::capability_v1::CapabilityCommand::Select{capability_ids:vec![]},rev).unwrap();
        let build=state.progression_v6.clone();state.install_build(build).unwrap();
        let retained=state.world_persistent_v1.map_knowledge_archive_v1.clone();
        project_information(&state,Some(&scene),Some(current_id),None,&mut output);
        assert_eq!(output.map_topology_authorized,Some(false));
        assert!(output.map_current_room_ids.is_none() && output.map_explored_room_ids.is_none());
        assert_eq!(state.world_persistent_v1.map_knowledge_archive_v1,retained);
    }

    #[test]
    fn lower_shaft_map_is_local_and_never_mixes_upper_history() {
        let mut state=super::super::FormalRuntime::initial_state(1).unwrap();
        state.world.scene_id="gh_central_shaft".into();
        state.world.player.position_m=Vec3{x_m:10.5,y_m:-4.0,z_m:10.0};
        let lower_id="grey_hive.gh_central_shaft.lower_info_v1.central_return_corridor";
        let upper_id="grey_hive.gh_central_shaft.upper_active_info_v1.cross_transit";
        let lower=super::super::floor_route_info::rows("shaft_lower_open",lower_id).unwrap();
        let upper=super::super::floor_route_info::rows("shaft_active_open",upper_id).unwrap();
        state.world.explored.rooms.extend(lower.clone());state.world.explored.rooms.extend(upper.clone());
        let mut scene:SceneDefinition=serde_json::from_str(include_str!("../../../content/scenes/compiled/gh_central_shaft.json")).unwrap();
        scene.verified_source_sha256="6880bdf8dbab71a8bf74e38b8c614ad751e564cf87e8a2936ee801e52b72a832".into();
        let (map,explored)=canonical_map(&state,Some(&scene),Some(lower_id),Some("shaft_lower_open"));
        assert!(!map.regions.is_empty());assert!(map.regions.iter().all(|r|r.id.starts_with(lower_id)));
        assert_eq!(map.regions[0].polygon.vertices[0].x,f64::from(lower[0].outline_m[0].x_m));
        assert_eq!(map.regions[0].polygon.vertices[0].y,f64::from(lower[0].outline_m[0].z_m));
        assert_eq!(explored.player_position.unwrap().x,10.5);
        state.world.player.position_m.y_m=0.0;
        let (upper_map,_)=canonical_map(&state,Some(&scene),Some(upper_id),Some("shaft_active_open"));
        assert!(upper_map.regions.iter().all(|r|!r.id.starts_with("grey_hive.gh_central_shaft.lower_info_v1")));
    }

    #[test]
    fn shaft_upper_history_requires_selected_upper_provider() {
        assert!(!upper_history_layer_allowed("grey_hive","gh_central_shaft",None));
        assert!(!upper_history_layer_allowed("grey_hive","gh_central_shaft",Some("entry")));
        assert!(upper_history_layer_allowed("grey_hive","gh_central_shaft",Some("shaft_active_open")));
        assert!(upper_history_layer_allowed("grey_hive","gh_gate_b",None));
    }

    #[test]
    fn observed_variant_history_is_validated_and_current_variant_wins() {
        use crate::world_v3::ObservedMapVariant;
        let shaft_id="grey_hive.gh_central_shaft.upper_active_info_v1.cross_transit";
        let dense_id="grey_hive.gh_lockdown.dense_info_v1.west_transit";
        let legacy_id="grey_hive.gh_lockdown.legacy_info_v1.west_approach";
        let mut rooms=super::super::floor_route_info::rows("shaft_active_open",shaft_id).unwrap();
        rooms.extend(super::super::floor_route_info::rows("lockdown_open",dense_id).unwrap());
        rooms.extend(super::super::floor_route_info::rows("lockdown_legacy_open",legacy_id).unwrap());
        let shaft=ObservedMapVariant::from_floor_route_key("shaft_active_open").unwrap();
        let dense=ObservedMapVariant::from_floor_route_key("lockdown_open").unwrap();
        let mut map=CanonicalMap::default();let mut exploration=MapExplorationState::default();
        append_floor_route_history(&mut map,&mut exploration,&rooms,"gh_gate_b",Some("gate_b_closed"),false,Some(&shaft),Some(&dense));
        assert!(map.regions.iter().any(|r|r.id.starts_with(dense_id)));
        assert!(!map.regions.iter().any(|r|r.id.starts_with(legacy_id)));
        let closed=super::super::floor_route_info::parts("shaft_active_closed",shaft_id).unwrap();
        let actual:Vec<_>=map.regions.iter().filter(|r|r.id.starts_with(shaft_id)).collect();
        assert_eq!(actual.len(),closed.len());
        for (region,part) in actual.iter().zip(closed.iter()) {
            assert_eq!(region.polygon.vertices[0].x,f64::from(part[0][0])+40.0);
            assert_eq!(region.polygon.vertices[0].y,f64::from(part[0][1])-1.0);
        }
        let mut current=CanonicalMap::default();
        append_floor_route_history(&mut current,&mut MapExplorationState::default(),&rooms,"gh_lockdown",Some("lockdown_legacy_closed"),false,Some(&shaft),Some(&dense));
        assert!(current.regions.iter().any(|r|r.id.starts_with(legacy_id)));
        assert!(!current.regions.iter().any(|r|r.id.starts_with(dense_id)));
        let mut unknown=shaft.clone();unknown.provider_family="future.family".into();
        let mut denied=CanonicalMap::default();
        append_floor_route_history(&mut denied,&mut MapExplorationState::default(),&rooms,"gh_gate_b",None,false,Some(&unknown),None);
        assert!(!denied.regions.iter().any(|r|r.id.starts_with(shaft_id)));
    }

    #[test]
    fn complete_provider_history_uses_one_offset_and_never_mutates_storage() {
        let bio_id = super::super::bio_info::regions().unwrap().iter()
            .filter_map(|r| r["id"].as_str())
            .find(|id| super::super::bio_info::rows(id).is_some_and(|p|p.len()>1)).unwrap();
        let bio=super::super::bio_info::rows(bio_id).unwrap();
        let decon_id=super::super::decon_info::regions().unwrap()[0]["id"].as_str().unwrap();
        let decon=super::super::decon_info::rows(decon_id).unwrap();
        let power=ExploredRoom {room_id:"power_nw".into(),outline_m:super::super::dense_service_wing::shared_region_polygon("power_nw").unwrap().iter()
            .map(|p|Vec3{x_m:p[0],y_m:0.0,z_m:p[1]}).collect()};
        let beacon=super::super::beacon_exit_info::rows("gh_beacon","grey_hive.beacon_exit.info_v1.gh_beacon.north_service").unwrap();
        let passage=super::super::beacon_exit_info::rows("gh_exit","grey_hive.beacon_exit.info_v1.gh_exit.seal_passage").unwrap();
        let mut rooms=bio.clone();
        rooms.push(power.clone());
        rooms.extend(decon.clone());
        rooms.extend(beacon.clone());
        rooms.extend(passage.clone());
        let original=rooms.clone();
        let mut canonical=CanonicalMap::default();
        let mut explored=MapExplorationState::default();
        append_grey_hive_history(&mut canonical,&mut explored,&rooms,false);
        for (part,x,z) in [(&power,0.0,0.0),(&bio[0],88.0,-1.0),(&decon[0],136.0,-1.0),(&beacon[0],184.0,-1.0)] {
            let region=canonical.regions.iter().find(|r|r.id==part.room_id).unwrap();
            assert_eq!(region.polygon.vertices[0].x,f64::from(part.outline_m[0].x_m)+x);
            assert_eq!(region.polygon.vertices[0].y,f64::from(part.outline_m[0].z_m)+z);
        }
        assert!(!canonical.regions.iter().any(|r|r.id==passage[0].room_id));
        assert_eq!(rooms,original);
        let mut opened=CanonicalMap::default();
        append_grey_hive_history(&mut opened,&mut MapExplorationState::default(),&rooms,true);
        let region=opened.regions.iter().find(|r|r.id==passage[0].room_id).unwrap();
        assert_eq!(region.polygon.vertices[0].x,f64::from(passage[0].outline_m[0].x_m)+184.0);
        let mut incomplete=bio.clone();
        incomplete.pop();
        let mut result=CanonicalMap::default();
        append_grey_hive_history(&mut result,&mut MapExplorationState::default(),&incomplete,false);
        assert!(result.regions.is_empty());
        for (scene,origin) in [("gh_entry_maintenance",(-20.0,0.0)),("gh_central_shaft",(40.0,-1.0)),("gh_lockdown",(64.0,-1.0)),("gh_gate_b",(112.0,-1.0)),("gh_sentinel_arena",(160.0,-1.0))] {
            assert_eq!(grey_hive_unmapped_origin(scene),Some(origin));
            let mut past=CanonicalMap::default();
            append_grey_hive_history(&mut past,&mut MapExplorationState::default(),&rooms,false);
            assert!(past.regions.iter().any(|r|r.id==power.room_id));
            assert!(!past.regions.iter().any(|r|r.id==format!("scene:{scene}")));
        }
    }
}

#[cfg(test)]
mod tests;
