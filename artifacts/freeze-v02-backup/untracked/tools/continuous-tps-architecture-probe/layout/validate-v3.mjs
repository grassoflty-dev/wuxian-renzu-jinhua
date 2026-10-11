#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const layoutPath = path.join(scriptDir, "layout-v3.json");
const reportPath = path.join(scriptDir, "..", "..", "..", "docs", "GREY_HIVE_LAYOUT_V3_REVIEW.md");
const outputPath = path.join(scriptDir, "..", "results", "layout-v3.json");
const data = JSON.parse(fs.readFileSync(layoutPath, "utf8"));
const markdown = fs.readFileSync(reportPath, "utf8");
const errors = [];
const round = (value) => Math.round(value * 1000) / 1000;
const distance = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const point2 = (point) => [point[0], point.length === 2 ? point[1] : point[2]];
const pathLength = (points) => points.slice(1).reduce((sum, point, index) => sum + distance(point2(points[index]), point2(point)), 0);
const rooms = new Map(data.rooms.map((room) => [room.id, room]));
const edges = new Map(data.connectors.map((edge) => [edge.id, edge]));
const entities = new Map(data.entities.map((entity) => [entity.id, entity]));

function roomBounds(room) {
  const [width, depth] = room.size;
  return { minX: room.center[0] - width / 2, maxX: room.center[0] + width / 2, minZ: room.center[2] - depth / 2, maxZ: room.center[2] + depth / 2 };
}

function contains(room, position, radius = 0) {
  const bounds = roomBounds(room);
  return position[0] >= bounds.minX + radius && position[0] <= bounds.maxX - radius && position[2] >= bounds.minZ + radius && position[2] <= bounds.maxZ - radius && Math.abs(position[1] - room.floor_y_m) < 0.01;
}

function adjacency(stateName) {
  const state = data.gate_states[stateName];
  const adjacencyMap = new Map(data.rooms.map((room) => [room.id, []]));
  for (const edge of data.connectors) {
    const doorState = state[edge.door.id] ?? edge.door.state;
    if (doorState === "Open") {
      adjacencyMap.get(edge.from).push(edge.to);
      adjacencyMap.get(edge.to).push(edge.from);
    }
  }
  return adjacencyMap;
}

function reachable(stateName, start = "GH-A01") {
  const graph = adjacency(stateName);
  const seen = new Set([start]);
  const queue = [start];
  while (queue.length) {
    const room = queue.shift();
    for (const next of graph.get(room) ?? []) if (!seen.has(next)) { seen.add(next); queue.push(next); }
  }
  return [...seen].sort();
}

function documentConsistency() {
  const missing = [];
  if (!markdown.includes("AG-09")) missing.push("AG-09 marker");
  if (!markdown.includes("250–350m") || !markdown.includes("500–800m")) missing.push("route target bands");
  if (!markdown.includes("100×100m") || !markdown.includes("60×80m")) missing.push("world size bands");
  for (const room of data.rooms) {
    if (!markdown.includes(`| ${room.id} |`)) missing.push(`${room.id} table row`);
    if (!markdown.includes(room.name)) missing.push(`${room.id} name`);
  }
  return missing;
}

if (data.schema !== "grey_hive_layout_ag09_v3") errors.push("schema is not grey_hive_layout_ag09_v3");
if (data.world_id !== "grey_hive") errors.push("world_id must remain grey_hive");
if (!Array.isArray(data.rooms) || data.rooms.length < 18) errors.push("v3 room graph must include canonical and added side/hidden rooms");
if (new Set(data.rooms.map((room) => room.id)).size !== data.rooms.length) errors.push("room IDs are not unique");
if (new Set(data.connectors.map((edge) => edge.id)).size !== data.connectors.length) errors.push("connector IDs are not unique");

const bounds = data.rooms.map(roomBounds);
const worldBounds = {
  min_x: round(Math.min(...bounds.map((bound) => bound.minX))),
  max_x: round(Math.max(...bounds.map((bound) => bound.maxX))),
  min_z: round(Math.min(...bounds.map((bound) => bound.minZ))),
  max_z: round(Math.max(...bounds.map((bound) => bound.maxZ))),
};
worldBounds.size_x = round(worldBounds.max_x - worldBounds.min_x);
worldBounds.size_z = round(worldBounds.max_z - worldBounds.min_z);
const envelopeTarget = data.target.building_envelope_m;
const boundsInTarget = worldBounds.size_x >= envelopeTarget.x[0] && worldBounds.size_x <= envelopeTarget.x[1] && worldBounds.size_z >= envelopeTarget.z[0] && worldBounds.size_z <= envelopeTarget.z[1];
if (!boundsInTarget) errors.push(`building envelope ${worldBounds.size_x}x${worldBounds.size_z}m is outside approximately 100x100m target`);

const primary = data.primary_playable_envelope;
const primarySizePass = primary.size_m[0] === 60 && primary.size_m[1] === 80 && primary.max[0] - primary.min[0] === 60 && primary.max[1] - primary.min[1] === 80;
if (!primarySizePass) errors.push("primary playable envelope is not 60x80m");

const layerChecks = data.rooms.map((room) => {
  const [width, depth, height] = room.size;
  const layer = data.vertical_layers[room.level];
  const floorInLayer = Boolean(layer && room.floor_y_m >= layer.floor_y_m[0] && room.floor_y_m <= layer.floor_y_m[1]);
  const heightInLayer = Boolean(layer && height >= layer.height_m[0] && height <= layer.height_m[1]);
  if (!floorInLayer || !heightInLayer) errors.push(`${room.id} violates ${room.level} vertical layer bounds`);
  if (!room.boundary || Object.keys(room.boundary).length < 4) errors.push(`${room.id} boundary contract missing`);
  return { id: room.id, level: room.level, floor_y_m: room.floor_y_m, size_m: [width, depth, height], floor_in_layer: floorInLayer, height_in_layer: heightInLayer };
});

const connectorChecks = data.connectors.map((edge) => {
  const from = rooms.get(edge.from);
  const to = rooms.get(edge.to);
  const first = edge.path[0];
  const last = edge.path.at(-1);
  const routeLength = pathLength(edge.path);
  const startInside = Boolean(from && contains(from, [first[0], from.floor_y_m, first.length === 2 ? first[1] : first[2]], 0.05));
  const endInside = Boolean(to && contains(to, [last[0], to.floor_y_m, last.length === 2 ? last[1] : last[2]], 0.05));
  const finite = edge.path.every((point) => point2(point).every(Number.isFinite));
  const blockerCovers = edge.door.blocker_width_m >= edge.width_m && edge.door.blocker_depth_m > 0 && edge.door.height_m >= edge.height_m;
  const dimensions = edge.width_m >= data.target.ordinary_corridor_min_m && edge.height_m >= data.target.door_min_height_m;
  if (!startInside || !endInside) errors.push(`${edge.id} endpoint is outside its declared room`);
  if (!finite) errors.push(`${edge.id} has non-finite centerline point`);
  if (!blockerCovers) errors.push(`${edge.id} door blocker does not cover the corridor section`);
  if (!dimensions) errors.push(`${edge.id} is below corridor/door minimum`);
  return { id: edge.id, from: edge.from, to: edge.to, width_m: edge.width_m, height_m: edge.height_m, route_length_m: round(routeLength), start_in_from: startInside, end_in_to: endInside, path_finite: finite, door_id: edge.door.id, blocker_covers_full_section: blockerCovers, dimensions_pass: dimensions };
});

const mainlineEdges = data.mainline.map((id) => edges.get(id));
if (mainlineEdges.some((edge) => !edge)) errors.push("mainline references a missing connector");
const mainlineDistance = mainlineEdges.reduce((sum, edge) => sum + pathLength(edge.path), 0);
const coreTarget = data.target.core_path_m;
const corePathPass = mainlineDistance >= coreTarget[0] && mainlineDistance <= coreTarget[1];
if (!corePathPass) errors.push(`core path ${round(mainlineDistance)}m is outside 250-350m target`);

function routeAudit(route) {
  let currentRoom = "GH-A01";
  let total = 0;
  const segments = [];
  for (const item of route) {
    const edge = edges.get(item.edge);
    if (!edge) {
      errors.push(`exploration route references missing ${item.edge}`);
      continue;
    }
    const forward = item.direction !== "reverse";
    const start = forward ? edge.from : edge.to;
    const end = forward ? edge.to : edge.from;
    const length = pathLength(edge.path);
    if (start !== currentRoom) errors.push(`exploration route discontinuity before ${item.edge}: expected ${currentRoom}, got ${start}`);
    currentRoom = end;
    total += length;
    segments.push({ edge: item.edge, direction: item.direction, from: start, to: end, purpose: item.purpose, length_m: round(length) });
  }
  return { start_room: "GH-A01", end_room: currentRoom, total_m: round(total), segments };
}
const exploration = routeAudit(data.exploration_route);
const explorationTarget = data.target.first_exploration_m;
const explorationPass = exploration.total_m >= explorationTarget[0] && exploration.total_m <= explorationTarget[1] && exploration.end_room === "GH-C04";
if (!explorationPass) errors.push(`first exploration route ${exploration.total_m}m or endpoint ${exploration.end_room} is outside 500-800m route contract`);

const entityChecks = data.entities.map((entity) => {
  const room = rooms.get(entity.room);
  const inside = Boolean(room && contains(room, entity.position, entity.radius_m ?? 0));
  if (!inside) errors.push(`${entity.id} is outside ${entity.room} with declared radius/floor`);
  return { id: entity.id, type: entity.type, room: entity.room, position: entity.position, contained_with_radius: inside };
});
for (const checkpointId of data.checkpoint_order) {
  const entity = entities.get(checkpointId);
  if (!entity || entity.type !== "checkpoint") errors.push(`checkpoint order references missing/non-checkpoint ${checkpointId}`);
}
const postBio = entities.get("gh_cp_post_bio");
if (!postBio || postBio.room !== "GH-B05" || data.rooms.find((room) => room.id === "GH-B05")?.role !== "gate_b_safety") errors.push("gh_cp_post_bio is not in the new Gate B pre-entry safety room");
if (postBio?.room === "GH-B03" || postBio?.room === "GH-B04") errors.push("gh_cp_post_bio still uses a deprecated bio-room location");

const gateConnectivity = Object.fromEntries(Object.keys(data.gate_states).map((state) => {
  const reachableRooms = reachable(state);
  return [state, { reachable_rooms: reachableRooms, b01: reachableRooms.includes("GH-B01"), b05: reachableRooms.includes("GH-B05"), c01: reachableRooms.includes("GH-C01"), c04: reachableRooms.includes("GH-C04") }];
}));
const gateChecks = [
  { state: "INITIAL", expected: { b01: false, b05: false, c01: false }, pass: !gateConnectivity.INITIAL.b01 && !gateConnectivity.INITIAL.b05 && !gateConnectivity.INITIAL.c01 },
  { state: "POWERED", expected: { b01: true, b05: true, c01: false }, pass: gateConnectivity.POWERED.b01 && gateConnectivity.POWERED.b05 && !gateConnectivity.POWERED.c01 },
  { state: "BAIZHI_COMPLETE", expected: { b01: true, b05: true, c01: false }, pass: gateConnectivity.BAIZHI_COMPLETE.b01 && gateConnectivity.BAIZHI_COMPLETE.b05 && !gateConnectivity.BAIZHI_COMPLETE.c01 },
  { state: "SENTINEL_READY", expected: { b01: true, b05: true, c01: true }, pass: gateConnectivity.SENTINEL_READY.b01 && gateConnectivity.SENTINEL_READY.b05 && gateConnectivity.SENTINEL_READY.c01 },
];
if (gateChecks.some((check) => !check.pass)) errors.push("gate state reachability does not match Gate A/B progression");

const gateGeometry = ["gh_gate_a", "gh_gate_b"].map((doorId) => {
  const edge = data.connectors.find((candidate) => candidate.door.id === doorId);
  const fullSection = Boolean(edge && edge.door.blocker_width_m >= edge.width_m && edge.door.blocker_depth_m > 0);
  if (!fullSection) errors.push(`${doorId} does not cover full corridor section`);
  return { door_id: doorId, edge: edge?.id ?? null, corridor_width_m: edge?.width_m ?? null, blocker_width_m: edge?.door.blocker_width_m ?? null, closed_states: ["Locked", "Unlocking"], full_section_covered: fullSection, alternate_graph_edge: false };
});

const rhythmChecks = {
  gate_a: data.gate_rhythm.gate_a.pre_corridor_width_m_max <= 3.5 && data.gate_rhythm.gate_a.post_size_m[0] >= 28 && data.gate_rhythm.gate_a.post_size_m[1] >= 24,
  gate_b: data.gate_rhythm.gate_b.deep_floor_y_delta_m < 0 && data.gate_rhythm.gate_b.deep_corridor_width_m_max <= 3.2 && data.gate_rhythm.gate_b.audio_relative_db < 0 && data.gate_rhythm.gate_b.light_relative < 1 && data.gate_rhythm.gate_b.biotrace_density_per_10m > 0,
};
if (!rhythmChecks.gate_a || !rhythmChecks.gate_b) errors.push("gate rhythm measurable contract is incomplete");

const documentMissing = documentConsistency();
if (documentMissing.length) errors.push(`document consistency missing: ${documentMissing.join(", ")}`);

const result = {
  schema: data.schema,
  status: errors.length ? "failed" : "passed_with_gate_evidence_limits",
  errors,
  document_consistent: documentMissing.length === 0,
  room_count: data.rooms.length,
  connector_count: data.connectors.length,
  bounds_m: worldBounds,
  bounds_target_m: data.target.building_envelope_m,
  bounds_in_target: boundsInTarget,
  primary_playable_envelope: { declared: data.primary_playable_envelope, size_pass: primarySizePass },
  vertical_layers: layerChecks,
  core_path: { edge_count: data.mainline.length, distance_m: round(mainlineDistance), target_m: coreTarget, in_target: corePathPass, edges: connectorChecks.filter((edge) => data.mainline.includes(edge.id)) },
  first_exploration_route: { distance_m: exploration.total_m, target_m: explorationTarget, in_target: explorationPass, ...exploration },
  connectors: connectorChecks,
  entities: entityChecks,
  entity_count: entityChecks.length,
  checkpoint_order: data.checkpoint_order,
  gate_connectivity: gateConnectivity,
  gate_checks: gateChecks,
  gate_geometry: gateGeometry,
  gate_rhythm: { declared: data.gate_rhythm, checks: rhythmChecks },
  checkpoint_semantic_migration: data.checkpoint_semantic_migration,
  room_id_traceability: data.room_id_traceability,
  transform_status: "candidate coordinates only; final transforms must be exported from formal TP-09 Blockout",
  formal_blockout_pending: data.formal_blockout_pending,
};
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(`AG-09 layout validation: ${result.status}; errors=${errors.length}; rooms=${data.rooms.length}; bounds=${worldBounds.size_x}x${worldBounds.size_z}m; core=${round(mainlineDistance)}m; exploration=${exploration.total_m}m`);
if (errors.length) process.exitCode = 1;
