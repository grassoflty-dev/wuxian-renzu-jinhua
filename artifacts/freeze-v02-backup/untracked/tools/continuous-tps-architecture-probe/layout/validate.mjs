#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const scriptDir = path.dirname(new URL(import.meta.url).pathname.replace(/^\/[A-Za-z]:/, (match) => match.slice(1)));

function args(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 1) if (argv[i].startsWith("--")) out[argv[i].slice(2)] = argv[i + 1] ?? true;
  return out;
}

function round(value) { return Math.round(value * 1000) / 1000; }
function distance(a, b) { return Math.hypot(a[0] - b[0], a[1] - b[1]); }
function pathLength(points) { let total = 0; for (let i = 1; i < points.length; i += 1) total += distance(points[i - 1], points[i]); return total; }
function path2d(points) { return points.map((point) => point.length === 2 ? point : [point[0], point[2]]); }
function roomBounds(room) {
  const [x, , z] = room.center; const [width, depth] = room.size;
  return { minX: x - width / 2, maxX: x + width / 2, minZ: z - depth / 2, maxZ: z + depth / 2 };
}
function contains(room, position, radius = 0) {
  const b = roomBounds(room); const [x, , z] = position;
  return x >= b.minX + radius && x <= b.maxX - radius && z >= b.minZ + radius && z <= b.maxZ - radius;
}
function adjacency(data, stateName) {
  const state = data.gate_states[stateName];
  const adj = new Map(data.rooms.map((room) => [room.id, []]));
  for (const edge of data.connectors) {
    const doorState = state[edge.door.id] ?? edge.door.state;
    if (doorState === "Open") {
      adj.get(edge.from).push(edge.to); adj.get(edge.to).push(edge.from);
    }
  }
  return adj;
}
function reachable(data, stateName, start) {
  const adj = adjacency(data, stateName); const seen = new Set([start]); const queue = [start];
  while (queue.length) for (const next of adj.get(queue.shift())) if (!seen.has(next)) { seen.add(next); queue.push(next); }
  return [...seen].sort();
}
function markdownConsistency(markdown, data) {
  const errors = [];
  for (const room of data.rooms) {
    const [width, depth, height] = room.size;
    const escaped = room.id.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const row = new RegExp(`\\|\\s*${escaped}\\s*\\|[^\\n]*\\|[^\\n]*\\|\\s*${width}×${depth}×${height}\\s*\\|`);
    if (!row.test(markdown)) errors.push(`${room.id} size row missing or inconsistent`);
  }
  if (!markdown.includes("GH-A04") || !markdown.includes("GH-C04")) errors.push("canonical A04/C04 IDs absent from document");
  if (!markdown.includes("GH_GA") || !markdown.includes("GH_GB")) errors.push("legacy alias mapping absent from document");
  return errors;
}
function graphPath(data, edgeIds) { return edgeIds.reduce((sum, id) => sum + pathLength(path2d(data.connectors.find((edge) => edge.id === id).path)), 0); }

const options = args(process.argv.slice(2));
if (!options.layout || !options.out) {
  console.error("Usage: node validate.mjs --layout <markdown> --out <json>"); process.exit(2);
}
const layoutPath = path.resolve(String(options.layout));
const outputPath = path.resolve(String(options.out));
const markdown = fs.readFileSync(layoutPath, "utf8");
const data = JSON.parse(fs.readFileSync(path.join(scriptDir, "layout-v2.json"), "utf8"));
const rooms = new Map(data.rooms.map((room) => [room.id, room]));
const errors = markdownConsistency(markdown, data);
const connectorChecks = data.connectors.map((edge) => {
  const from = rooms.get(edge.from); const to = rooms.get(edge.to); const first = edge.path[0]; const last = edge.path[edge.path.length - 1];
  const routeLength = pathLength(path2d(edge.path));
  const startInFrom = contains(from, [first[0], 0, first[1]], 0.01); const endInTo = contains(to, [last[0], 0, last[1]], 0.01);
  const blockerCovers = edge.door.blocker_width_m >= edge.width_m && edge.door.blocker_depth_m > 0 && edge.door.height_m >= edge.height_m;
  const pathInsideBounds = path2d(edge.path).every(([x, z]) => Number.isFinite(x) && Number.isFinite(z));
  if (!startInFrom || !endInTo) errors.push(`${edge.id} endpoint is not inside its room`);
  if (edge.width_m < data.target.ordinary_corridor_min_m) errors.push(`${edge.id} corridor narrower than ordinary target`);
  if (!blockerCovers) errors.push(`${edge.id} door blocker does not cover full corridor section`);
  return { id: edge.id, from: edge.from, to: edge.to, width_m: edge.width_m, height_m: edge.height_m, route_length_m: round(routeLength), start_in_from: startInFrom, end_in_to: endInTo, door: { id: edge.door.id, state: edge.door.state, blocker_covers_full_section: blockerCovers }, path_finite: pathInsideBounds };
});
const bounds = data.rooms.map(roomBounds);
const worldBounds = { min_x: Math.min(...bounds.map((b) => b.minX)), max_x: Math.max(...bounds.map((b) => b.maxX)), min_z: Math.min(...bounds.map((b) => b.minZ)), max_z: Math.max(...bounds.map((b) => b.maxZ)) };
worldBounds.size_x = round(worldBounds.max_x - worldBounds.min_x); worldBounds.size_z = round(worldBounds.max_z - worldBounds.min_z);
const boundsInTarget = worldBounds.size_x >= data.target.bounds_x_m[0] && worldBounds.size_x <= data.target.bounds_x_m[1] && worldBounds.size_z >= data.target.bounds_z_m[0] && worldBounds.size_z <= data.target.bounds_z_m[1];
if (!boundsInTarget) errors.push("candidate bounds outside target");
const sentinel = rooms.get("GH-C03");
const sentinelMinDimension = Math.min(sentinel.size[0], sentinel.size[1]);
const sentinelArenaInTarget = sentinelMinDimension >= data.target.sentinel_arena_min_m && sentinelMinDimension <= data.target.sentinel_arena_max_m;
if (!sentinelArenaInTarget) errors.push("GH-C03 Sentinel Arena dimension outside target");
const entityChecks = data.entities.map((entity) => {
  const room = rooms.get(entity.room); const inside = contains(room, entity.position, entity.radius_m ?? 0);
  if (!inside) errors.push(`${entity.id} is outside ${entity.room} with declared radius`);
  return { id: entity.id, room: entity.room, type: entity.type, position: entity.position, contained_with_radius: inside };
});
const mainlineDistance = graphPath(data, data.mainline);
const branchDistances = data.exploration_branches.map((id) => ({ id, distance_m: round(graphPath(data, [id])) }));
const explorationReturnDistance = branchDistances.reduce((sum, edge) => sum + edge.distance_m * 2, 0);
const gateConnectivity = Object.fromEntries(Object.keys(data.gate_states).map((state) => [state, { from_airlock: reachable(data, state, "GH-A01"), b01_reachable: reachable(data, state, "GH-A01").includes("GH-B01"), c01_reachable: reachable(data, state, "GH-A01").includes("GH-C01"), c04_reachable: reachable(data, state, "GH-A01").includes("GH-C04") }]));
const gateChecks = [
  { state: "INITIAL", expect_b01: false, expect_c01: false, pass: !gateConnectivity.INITIAL.b01_reachable && !gateConnectivity.INITIAL.c01_reachable },
  { state: "POWERED", expect_b01: true, expect_c01: false, pass: gateConnectivity.POWERED.b01_reachable && !gateConnectivity.POWERED.c01_reachable },
  { state: "LOCKDOWN_CLEARED", expect_b01: true, expect_c01: true, pass: gateConnectivity.LOCKDOWN_CLEARED.b01_reachable && gateConnectivity.LOCKDOWN_CLEARED.c01_reachable },
];
const result = {
  schema: data.schema,
  input_layout: layoutPath,
  canonical_data: path.join(scriptDir, "layout-v2.json"),
  status: errors.length ? "failed" : "passed_with_gate_evidence_limits",
  errors,
  document_consistent: errors.filter((error) => error.includes("row missing") || error.includes("IDs absent") || error.includes("alias mapping")).length === 0,
  room_count: data.rooms.length,
  bounds_m: worldBounds,
  bounds_in_target: boundsInTarget,
  bounds_target_m: data.target,
  sentinel_arena: { size_m: sentinel.size, min_dimension_m: sentinelMinDimension, in_target: sentinelArenaInTarget },
  mainline: { edge_count: data.mainline.length, shortest_candidate_m: round(mainlineDistance), target_m: data.target.mainline_m, in_target: mainlineDistance >= data.target.mainline_m[0] && mainlineDistance <= data.target.mainline_m[1] },
  exploration: { branches: branchDistances, return_distance_m: round(explorationReturnDistance), mainline_plus_exploration_m: round(mainlineDistance + explorationReturnDistance), not_counted_in_mainline_target: true },
  connectors: connectorChecks,
  entities: entityChecks,
  gate_connectivity: gateConnectivity,
  gate_checks: gateChecks,
  gate_geometry_evidence: data.connectors.filter((edge) => edge.door.id === "gh_gate_a" || edge.door.id === "gh_gate_b").map((edge) => ({ edge: edge.id, door: edge.door.id, closed_states: ["Locked", "Unlocking"], blocker_width_m: edge.door.blocker_width_m, corridor_width_m: edge.width_m, full_section_covered: edge.door.blocker_width_m >= edge.width_m, alternate_graph_edge: false })),
  transform_status: "candidate coordinates for isolated validation only; final transforms must be exported from formal Blockout",
};
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(`AG-02 layout validation: ${result.status}; errors=${errors.length}; mainline=${round(mainlineDistance)}m; exploration_return=${round(explorationReturnDistance)}m; bounds=${worldBounds.size_x}x${worldBounds.size_z}m`);
if (errors.length) process.exitCode = 1;
