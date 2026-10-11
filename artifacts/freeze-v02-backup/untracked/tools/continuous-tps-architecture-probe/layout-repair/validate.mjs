#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const layoutPath = path.join(scriptDir, "layout-v5.json");
const resultPath = path.join(scriptDir, "..", "results", "layout-repair-ag18.json");
const data = JSON.parse(fs.readFileSync(layoutPath, "utf8"));
const errors = [];
const warnings = [];
const round = (n) => Math.round(n * 1000) / 1000;
const rooms = new Map(data.rooms.map((room) => [room.id, room]));
const edges = new Map(data.connectors.map((edge) => [edge.id, edge]));
const entities = new Map(data.entities.map((entity) => [entity.id, entity]));
const dist3 = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
const pathLength = (points) => points.slice(1).reduce((sum, point, i) => sum + dist3(points[i], point), 0);

function bounds(room) {
  return {
    minX: room.center[0] - room.size[0] / 2,
    maxX: room.center[0] + room.size[0] / 2,
    minZ: room.center[2] - room.size[1] / 2,
    maxZ: room.center[2] + room.size[1] / 2,
  };
}

function contains(room, point, margin = 0) {
  const b = bounds(room);
  return point[0] >= b.minX + margin && point[0] <= b.maxX - margin
    && point[2] >= b.minZ + margin && point[2] <= b.maxZ - margin
    && Math.abs(point[1] - room.floor_y_m) <= 0.01;
}

function pointAt(a, b, t) {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}

function samplePath(points, step = data.target.sampled_nav_max_step_m) {
  const samples = [points[0]];
  for (let i = 1; i < points.length; i += 1) {
    const a = points[i - 1];
    const b = points[i];
    const length = dist3(a, b);
    const count = Math.max(1, Math.ceil(length / step));
    for (let j = 1; j <= count; j += 1) samples.push(pointAt(a, b, j / count));
  }
  return samples;
}

function roomHits(point) {
  return data.rooms.filter((room) => contains(room, point, 0.001)).map((room) => room.id);
}

function checkVertical(edge, from, to) {
  const deltaY = to.floor_y_m - from.floor_y_m;
  if (Math.abs(deltaY) < 0.001) {
    if (edge.vertical) errors.push(`${edge.id} declares vertical link on same-floor rooms`);
    return { required: false, pass: !edge.vertical };
  }
  const v = edge.vertical;
  const pass = Boolean(v && v.kind && ["stair", "ramp", "elevator"].includes(v.kind)
    && v.enter_exit_landings === true
    && Array.isArray(v.path ?? edge.path)
    && (v.kind !== "stair" || (Number.isFinite(v.slope_degrees) && v.slope_degrees <= 35 && v.run_m >= Math.abs(deltaY))));
  if (!pass) errors.push(`${edge.id} lacks a valid explicit vertical connector`);
  if (v?.kind === "elevator" && (!Array.isArray(v.stops_y_m) || !v.stops_y_m.includes(from.floor_y_m) || !v.stops_y_m.includes(to.floor_y_m) || !Array.isArray(v.door_states))) {
    errors.push(`${edge.id} elevator lacks explicit stops or door states`);
  }
  if (v?.kind === "stair" && (!Array.isArray(v.landings_y_m) || !v.landings_y_m.includes(from.floor_y_m) || !v.landings_y_m.includes(to.floor_y_m))) {
    errors.push(`${edge.id} stair lacks both floor landings`);
  }
  return { required: true, pass, kind: v?.kind ?? null, delta_y_m: deltaY, slope_degrees: v?.slope_degrees ?? null };
}

const roomChecks = data.rooms.map((room) => {
  const layer = data.vertical_layers[room.level];
  const floorPass = Boolean(layer && room.floor_y_m >= layer.floor_y_m[0] && room.floor_y_m <= layer.floor_y_m[1]);
  const heightPass = Boolean(layer && room.size[2] >= layer.height_m[0] && room.size[2] <= layer.height_m[1]);
  const b = bounds(room);
  const envelopePass = b.minX >= data.site_envelope.min[0] && b.maxX <= data.site_envelope.max[0] && b.minZ >= data.site_envelope.min[1] && b.maxZ <= data.site_envelope.max[1];
  if (!floorPass || !heightPass || !envelopePass) errors.push(`${room.id} violates vertical layer or envelope contract`);
  return { id: room.id, level: room.level, bounds: b, floor_pass: floorPass, height_pass: heightPass, envelope_pass: envelopePass };
});

const connectorChecks = data.connectors.map((edge) => {
  const from = rooms.get(edge.from);
  const to = rooms.get(edge.to);
  const first = edge.path[0];
  const last = edge.path.at(-1);
  const endpointPass = Boolean(from && to && contains(from, first, 0.05) && contains(to, last, 0.05));
  const dimensionsPass = edge.width_m >= data.target.ordinary_corridor_min_m && edge.height_m >= data.target.door_min_height_m && edge.door.height_m >= data.target.door_min_height_m && edge.door.blocker_width_m >= edge.width_m && edge.door.blocker_depth_m > 0;
  const finitePass = edge.path.every((point) => point.length === 3 && point.every(Number.isFinite));
  const vertical = from && to ? checkVertical(edge, from, to) : { required: false, pass: false };
  const samples = samplePath(edge.path);
  const intermediateRoomCrossings = [];
  for (const point of samples) {
    const hits = roomHits(point).filter((id) => id !== edge.from && id !== edge.to);
    for (const room of hits) if (!intermediateRoomCrossings.some((hit) => hit.room === room)) intermediateRoomCrossings.push({ room, point: point.map(round) });
  }
  if (!endpointPass) errors.push(`${edge.id} endpoint is outside its declared room`);
  if (!dimensionsPass) errors.push(`${edge.id} corridor, door, or blocker is below minimum`);
  if (!finitePass) errors.push(`${edge.id} contains non-finite or non-3D path data`);
  if (intermediateRoomCrossings.length) errors.push(`${edge.id} crosses a non-door room: ${intermediateRoomCrossings.map((hit) => hit.room).join(", ")}`);
  if (!vertical.pass) errors.push(`${edge.id} vertical contract failed`);
  return {
    id: edge.id,
    from: edge.from,
    to: edge.to,
    length_m: round(pathLength(edge.path)),
    sample_count: samples.length,
    max_step_m: data.target.sampled_nav_max_step_m,
    endpoint_pass: endpointPass,
    dimensions_pass: dimensionsPass,
    capsule_width_pass: edge.width_m >= data.target.capsule_radius_m * 2,
    vertical,
    intermediate_non_door_room_crossings: intermediateRoomCrossings,
    nav_pass: endpointPass && dimensionsPass && finitePass && intermediateRoomCrossings.length === 0 && vertical.pass,
  };
});

function routeAudit(route) {
  let current = "GH-A01";
  let total = 0;
  const segments = [];
  for (const step of route) {
    if (step.edge === null) {
      segments.push({ edge: null, direction: "stay", room: current, event: step.event, length_m: 0 });
      continue;
    }
    const edge = edges.get(step.edge);
    if (!edge) {
      errors.push(`route references missing edge ${step.edge}`);
      continue;
    }
    const reverse = step.direction === "reverse";
    const start = reverse ? edge.to : edge.from;
    const end = reverse ? edge.from : edge.to;
    if (start !== current) errors.push(`route discontinuity at ${step.edge}: expected ${current}, got ${start}`);
    current = end;
    const length = pathLength(edge.path);
    total += length;
    segments.push({ edge: edge.id, direction: step.direction, from: start, to: end, event: step.event, length_m: round(length), sampled_nav: true });
  }
  return { start_room: "GH-A01", end_room: current, distance_m: round(total), segments };
}

const requiredEvents = data.objective_contract.required_sequence.map((step) => step.event);
function simulate(route, choice) {
  const flags = new Set();
  let room = "GH-A01";
  const events = [];
  const issues = [];
  for (const step of route) {
    const edge = step.edge === null ? null : edges.get(step.edge);
    if (step.edge !== null && !edge) continue;
    if (edge) {
      const reverse = step.direction === "reverse";
      const start = reverse ? edge.to : edge.from;
      const end = reverse ? edge.from : edge.to;
      if (start !== room) issues.push(`edge ${edge.id} starts at ${start}, current room is ${room}`);
      if (edge.door.id === "gh_gate_a" && !flags.has("hive_power")) issues.push("Gate A crossed before hive_power");
      if (edge.door.id === "gh_gate_b" && (!flags.has("hive_lockdown") || !flags.has("baizhi_choice_resolved") || !flags.has("post_bio_checkpoint_active"))) issues.push("Gate B crossed before required progression");
      room = end;
    }
    const event = step.event;
    switch (event) {
      case "complete_hive_power": if (room !== "GH-A03") issues.push("hive_power wrong room"); flags.add("hive_power"); break;
      case "complete_hive_lockdown": if (room !== "GH-B02" || !flags.has("hive_power")) issues.push("hive_lockdown out of order"); flags.add("hive_lockdown"); break;
      case "encounter_baizhi": if (room !== "GH-B04" || !flags.has("hive_lockdown")) issues.push("Baizhi encounter out of order"); flags.add("baizhi_encountered"); break;
      case "resolve_baizhi_choice": if (room !== "GH-B04" || !flags.has("baizhi_encountered") || !["hive_help_baizhi", "hive_leave_sample"].includes(choice)) issues.push("Baizhi choice invalid"); flags.add("baizhi_choice_resolved"); flags.add(choice); break;
      case "activate_post_bio_checkpoint": if (room !== "GH-B05" || !flags.has("baizhi_choice_resolved")) issues.push("post-bio checkpoint out of order"); flags.add("post_bio_checkpoint_active"); break;
      case "pass_gate_b": if (room !== "GH-C01" || !flags.has("post_bio_checkpoint_active")) issues.push("Gate B pass out of order"); flags.add("gate_b_open"); break;
      case "defeat_sentinel_and_take_beacon": if (room !== "GH-C03" || !flags.has("gate_b_open")) issues.push("Sentinel event out of order"); flags.add("sentinel_defeated"); flags.add("hive_rescue"); flags.add("beacon_taken"); break;
      case "reach_exit": if (room !== "GH-C04" || !flags.has("hive_rescue") || !flags.has("beacon_taken")) issues.push("exit reached before rescue/beacon"); flags.add("extracted"); break;
      default: break;
    }
    if (event) events.push(event);
  }
  const missing = requiredEvents.filter((event) => !events.includes(event));
  const repeated = requiredEvents.filter((event) => events.filter((seen) => seen === event).length !== 1);
  return { choice, end_room: room, events, missing_required_events: missing, repeated_required_events: repeated, issues, pass: room === "GH-C04" && missing.length === 0 && repeated.length === 0 && issues.length === 0 };
}

function reachable(stateName, start = "GH-A01") {
  const state = data.gate_states[stateName];
  const graph = new Map(data.rooms.map((room) => [room.id, []]));
  for (const edge of data.connectors) {
    if ((state[edge.door.id] ?? edge.door.state) === "Open") {
      graph.get(edge.from).push(edge.to);
      graph.get(edge.to).push(edge.from);
    }
  }
  const seen = new Set([start]);
  const queue = [start];
  while (queue.length) {
    const room = queue.shift();
    for (const next of graph.get(room) ?? []) if (!seen.has(next)) { seen.add(next); queue.push(next); }
  }
  return [...seen].sort();
}

const core = routeAudit(data.core_story_route);
const exploration = routeAudit(data.first_exploration_route);
const coreBranches = ["hive_help_baizhi", "hive_leave_sample"].map((choice) => simulate(data.core_story_route, choice));
const explorationBranches = ["hive_help_baizhi", "hive_leave_sample"].map((choice) => simulate(data.first_exploration_route, choice));
const coreLengthPass = core.distance_m >= data.target.core_story_route_m[0] && core.distance_m <= data.target.core_story_route_m[1];
const explorationLengthPass = exploration.distance_m >= data.target.first_exploration_route_m[0] && exploration.distance_m <= data.target.first_exploration_route_m[1];
if (!coreLengthPass) errors.push(`core route ${core.distance_m}m is outside target`);
if (!explorationLengthPass) errors.push(`exploration route ${exploration.distance_m}m is outside target`);
if (core.end_room !== "GH-C04" || exploration.end_room !== "GH-C04") errors.push("one or more routes do not end at GH-C04");
if (coreBranches.some((branch) => !branch.pass) || explorationBranches.some((branch) => !branch.pass)) errors.push("objective/flag simulation failed for a Baizhi branch");
if (connectorChecks.some((check) => !check.nav_pass)) errors.push("one or more sampled navigation connectors failed");

const arena = rooms.get("GH-C03");
const arenaMinDimension = Math.min(arena.size[0], arena.size[1]);
const arenaPass = arenaMinDimension >= data.target.sentinel_arena_min_dimension_m;
if (!arenaPass) errors.push("Sentinel Arena minimum dimension is below 18m");

const entityChecks = data.entities.map((entity) => {
  const room = rooms.get(entity.room);
  const pass = Boolean(room && contains(room, entity.position, entity.radius_m ?? 0));
  if (!pass) errors.push(`${entity.id} is outside ${entity.room} with capsule margin`);
  return { id: entity.id, room: entity.room, contained_with_capsule_margin: pass };
});
for (const checkpoint of data.checkpoint_order) if (!entities.has(checkpoint) || entities.get(checkpoint).type !== "checkpoint") errors.push(`checkpoint missing or wrong type: ${checkpoint}`);

const gateConnectivity = Object.fromEntries(Object.keys(data.gate_states).map((state) => {
  const nodes = reachable(state);
  return [state, { reachable: nodes, b01: nodes.includes("GH-B01"), b05: nodes.includes("GH-B05"), c01: nodes.includes("GH-C01"), c04: nodes.includes("GH-C04") }];
}));
const gateChecks = [
  { state: "INITIAL", pass: !gateConnectivity.INITIAL.b01 && !gateConnectivity.INITIAL.b05 && !gateConnectivity.INITIAL.c01 },
  { state: "POWERED", pass: gateConnectivity.POWERED.b01 && gateConnectivity.POWERED.b05 && !gateConnectivity.POWERED.c01 },
  { state: "LOCKDOWN_CLEARED", pass: gateConnectivity.LOCKDOWN_CLEARED.b01 && gateConnectivity.LOCKDOWN_CLEARED.b05 && !gateConnectivity.LOCKDOWN_CLEARED.c01 },
  { state: "BAIZHI_RESOLVED", pass: gateConnectivity.BAIZHI_RESOLVED.b01 && gateConnectivity.BAIZHI_RESOLVED.b05 && gateConnectivity.BAIZHI_RESOLVED.c01 },
];
if (gateChecks.some((check) => !check.pass)) errors.push("gate state reachability does not match progression");

const result = {
  schema: "grey_hive_layout_repair_ag18_v1",
  status: errors.length ? "blocked_geometry_candidate" : "passed_isolated_v5_candidate",
  scope: "isolated v5 polygon/vertical-link/door-frame/capsule sampled-nav evidence; not formal Babylon Blockout or Rapier collider validation",
  source: "layout-v5.json",
  repair_map: data.repair_map,
  thresholds: data.target,
  room_count: data.rooms.length,
  connector_count: data.connectors.length,
  entity_count: data.entities.length,
  room_checks: roomChecks,
  connectors: connectorChecks,
  sentinel_arena: { room: "GH-C03", size_m: [arena.size[0], arena.size[1], arena.size[2]], minimum_horizontal_dimension_m: arenaMinDimension, pass: arenaPass },
  core_story_route: { ...core, target_m: data.target.core_story_route_m, in_target: coreLengthPass, branch_simulations: coreBranches },
  first_exploration_route: { ...exploration, target_m: data.target.first_exploration_route_m, in_target: explorationLengthPass, branch_simulations: explorationBranches },
  gate_connectivity: gateConnectivity,
  gate_checks: gateChecks,
  entities: entityChecks,
  checkpoint_order: data.checkpoint_order,
  remaining_tp09_gaps: data.formal_blockout_pending,
  errors,
  warnings,
  formal_gate_note: "A passed isolated v5 candidate does not imply Gate I or TP09 acceptance",
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(`AG-18 layout repair: ${result.status}; errors=${errors.length}; core=${core.distance_m}m; exploration=${exploration.distance_m}m; vertical=${connectorChecks.filter((check) => check.vertical.required).length}; arena_min=${arenaMinDimension}m`);
if (errors.length) process.exitCode = 1;
