#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const layoutPath = path.join(scriptDir, "layout-v4.json");
const reportPath = path.join(scriptDir, "..", "..", "..", "docs", "GREY_HIVE_ROUTE_SEMANTICS_REVIEW.md");
const outputPath = path.join(scriptDir, "..", "results", "layout-v4.json");
const data = JSON.parse(fs.readFileSync(layoutPath, "utf8"));
const markdown = fs.existsSync(reportPath) ? fs.readFileSync(reportPath, "utf8") : "";
const errors = [];
const round = (value) => Math.round(value * 1000) / 1000;
const point2 = (point) => [point[0], point.length === 2 ? point[1] : point[2]];
const distance = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const pathLength = (points) => points.slice(1).reduce((sum, point, index) => sum + distance(point2(points[index]), point2(point)), 0);
const rooms = new Map(data.rooms.map((room) => [room.id, room]));
const edges = new Map(data.connectors.map((edge) => [edge.id, edge]));
const entities = new Map(data.entities.map((entity) => [entity.id, entity]));

function roomBounds(room) {
  return { minX: room.center[0] - room.size[0] / 2, maxX: room.center[0] + room.size[0] / 2, minZ: room.center[2] - room.size[1] / 2, maxZ: room.center[2] + room.size[1] / 2 };
}

function contains(room, position, radius = 0) {
  const bounds = roomBounds(room);
  return position[0] >= bounds.minX + radius && position[0] <= bounds.maxX - radius && position[2] >= bounds.minZ + radius && position[2] <= bounds.maxZ - radius && Math.abs(position[1] - room.floor_y_m) <= 0.01;
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

function documentConsistency() {
  const missing = [];
  for (const token of ["AG-10", "250–350m", "500–800m", "100×100m", "60×80m", "609.702m", "hive_power", "hive_lockdown", "hive_rescue", "hive_help_baizhi", "hive_leave_sample"]) if (!markdown.includes(token)) missing.push(token);
  for (const room of data.rooms) {
    if (!markdown.includes(`| ${room.id} |`)) missing.push(`${room.id} row`);
    if (!markdown.includes(room.name)) missing.push(`${room.id} name`);
  }
  return missing;
}

const bounds = data.rooms.map(roomBounds);
const worldBounds = { min_x: round(Math.min(...bounds.map((bound) => bound.minX))), max_x: round(Math.max(...bounds.map((bound) => bound.maxX))), min_z: round(Math.min(...bounds.map((bound) => bound.minZ))), max_z: round(Math.max(...bounds.map((bound) => bound.maxZ))) };
worldBounds.size_x = round(worldBounds.max_x - worldBounds.min_x);
worldBounds.size_z = round(worldBounds.max_z - worldBounds.min_z);
const envelopeTarget = data.target.building_envelope_m;
const boundsPass = worldBounds.size_x >= envelopeTarget.x[0] && worldBounds.size_x <= envelopeTarget.x[1] && worldBounds.size_z >= envelopeTarget.z[0] && worldBounds.size_z <= envelopeTarget.z[1];
if (!boundsPass) errors.push("v4 building envelope is outside approximately 100x100m target");
const primary = data.primary_playable_envelope;
const primaryPass = primary.size_m[0] === 60 && primary.size_m[1] === 80 && primary.max[0] - primary.min[0] === 60 && primary.max[1] - primary.min[1] === 80;
if (!primaryPass) errors.push("v4 primary playable envelope is not 60x80m");

const layerChecks = data.rooms.map((room) => {
  const layer = data.vertical_layers[room.level];
  const floorPass = Boolean(layer && room.floor_y_m >= layer.floor_y_m[0] && room.floor_y_m <= layer.floor_y_m[1]);
  const heightPass = Boolean(layer && room.size[2] >= layer.height_m[0] && room.size[2] <= layer.height_m[1]);
  if (!floorPass || !heightPass) errors.push(`${room.id} violates vertical layer contract`);
  return { id: room.id, level: room.level, floor_y_m: room.floor_y_m, size_m: room.size, floor_pass: floorPass, height_pass: heightPass };
});

const connectorChecks = data.connectors.map((edge) => {
  const from = rooms.get(edge.from);
  const to = rooms.get(edge.to);
  const first = edge.path[0];
  const last = edge.path.at(-1);
  const startInside = Boolean(from && contains(from, [first[0], from.floor_y_m, first.length === 2 ? first[1] : first[2]], 0.05));
  const endInside = Boolean(to && contains(to, [last[0], to.floor_y_m, last.length === 2 ? last[1] : last[2]], 0.05));
  const finite = edge.path.every((point) => point2(point).every(Number.isFinite));
  const blocker = edge.door.blocker_width_m >= edge.width_m && edge.door.blocker_depth_m > 0 && edge.door.height_m >= edge.height_m;
  const dimensions = edge.width_m >= data.target.ordinary_corridor_min_m && edge.height_m >= data.target.door_min_height_m;
  if (!startInside || !endInside) errors.push(`${edge.id} endpoint outside room`);
  if (!finite) errors.push(`${edge.id} centerline contains non-finite point`);
  if (!blocker) errors.push(`${edge.id} blocker does not cover corridor`);
  if (!dimensions) errors.push(`${edge.id} corridor/door below minimum`);
  return { id: edge.id, from: edge.from, to: edge.to, width_m: edge.width_m, height_m: edge.height_m, length_m: round(pathLength(edge.path)), start_in_from: startInside, end_in_to: endInside, blocker_covers: blocker, dimensions_pass: dimensions };
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
    const forward = step.direction !== "reverse";
    const start = forward ? edge.from : edge.to;
    const end = forward ? edge.to : edge.from;
    const length = pathLength(edge.path);
    if (start !== current) errors.push(`route discontinuity at ${step.edge}: expected ${current}, got ${start}`);
    current = end;
    total += length;
    segments.push({ edge: step.edge, direction: step.direction, from: start, to: end, event: step.event, length_m: round(length) });
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
      const forward = step.direction !== "reverse";
      const start = forward ? edge.from : edge.to;
      const end = forward ? edge.to : edge.from;
      if (start !== room) issues.push(`edge ${edge.id} starts at ${start}, current room is ${room}`);
      if (edge.door.id === "gh_gate_a" && !flags.has("hive_power")) issues.push("Gate A crossed before hive_power");
      if (edge.door.id === "gh_gate_b" && (!flags.has("hive_lockdown") || !flags.has("baizhi_choice_resolved") || !flags.has("post_bio_checkpoint_active"))) issues.push("Gate B crossed before lockdown, Baizhi choice and post-bio checkpoint");
      room = end;
    }
    const event = step.event;
    if (!event) continue;
    if (events.includes(event) && ["complete_hive_power", "complete_hive_lockdown", "encounter_baizhi", "resolve_baizhi_choice", "activate_post_bio_checkpoint", "pass_gate_b", "defeat_sentinel_and_take_beacon", "reach_exit"].includes(event)) issues.push(`required event repeated: ${event}`);
    switch (event) {
      case "complete_hive_power":
        if (room !== "GH-A03") issues.push("hive_power completed outside GH-A03");
        flags.add("hive_power"); flags.add("gate_a_open"); break;
      case "complete_hive_lockdown":
        if (room !== "GH-B02" || !flags.has("hive_power")) issues.push("hive_lockdown is out of order or wrong room");
        flags.add("hive_lockdown"); break;
      case "encounter_baizhi":
        if (room !== "GH-B04" || !flags.has("hive_lockdown")) issues.push("Baizhi encounter is out of order or wrong room");
        flags.add("baizhi_encountered"); break;
      case "resolve_baizhi_choice":
        if (room !== "GH-B04" || !flags.has("baizhi_encountered") || !["hive_help_baizhi", "hive_leave_sample"].includes(choice)) issues.push("Baizhi choice is invalid or out of order");
        flags.add(choice); flags.add("baizhi_choice_resolved"); break;
      case "activate_post_bio_checkpoint":
        if (room !== "GH-B05" || !flags.has("baizhi_choice_resolved")) issues.push("post-bio checkpoint is not after Baizhi choice in GH-B05");
        flags.add("post_bio_checkpoint_active"); break;
      case "pass_gate_b":
        if (room !== "GH-C01" || !flags.has("post_bio_checkpoint_active")) issues.push("Gate B pass is out of order or wrong room");
        flags.add("gate_b_open"); break;
      case "defeat_sentinel_and_take_beacon":
        if (room !== "GH-C03" || !flags.has("gate_b_open")) issues.push("Sentinel/beacon event is out of order or wrong room");
        flags.add("sentinel_defeated"); flags.add("hive_rescue"); flags.add("beacon_taken"); break;
      case "reach_exit":
        if (room !== "GH-C04" || !flags.has("hive_rescue") || !flags.has("beacon_taken")) issues.push("Exit reached before rescue/beacon");
        flags.add("extracted"); break;
      default: break;
    }
    events.push(event);
  }
  const missing = requiredEvents.filter((event) => !events.includes(event));
  const repeatedRequired = requiredEvents.filter((event) => events.filter((seen) => seen === event).length !== 1);
  return { choice, end_room: room, events, flags: [...flags].sort(), missing_required_events: missing, repeated_required_events: repeatedRequired, issues, pass: room === "GH-C04" && missing.length === 0 && repeatedRequired.length === 0 && issues.length === 0 };
}

const coreRoute = routeAudit(data.core_story_route);
const explorationRoute = routeAudit(data.first_exploration_route);
const coreTarget = data.target.core_story_route_m;
const explorationTarget = data.target.first_exploration_route_m;
const coreLengthPass = coreRoute.distance_m >= coreTarget[0] && coreRoute.distance_m <= coreTarget[1] && coreRoute.end_room === "GH-C04";
const explorationLengthPass = explorationRoute.distance_m >= explorationTarget[0] && explorationRoute.distance_m <= explorationTarget[1] && explorationRoute.end_room === "GH-C04";
if (!coreLengthPass) errors.push(`core route ${coreRoute.distance_m}m is outside 250-350m`);
if (!explorationLengthPass) errors.push(`exploration route ${explorationRoute.distance_m}m is outside 500-800m`);
const coreSimulations = ["hive_help_baizhi", "hive_leave_sample"].map((choice) => simulate(data.core_story_route, choice));
const explorationSimulations = ["hive_help_baizhi", "hive_leave_sample"].map((choice) => simulate(data.first_exploration_route, choice));
if (coreSimulations.some((simulation) => !simulation.pass)) errors.push("core route objective/flag simulation failed for a Baizhi choice branch");
if (explorationSimulations.some((simulation) => !simulation.pass)) errors.push("exploration route objective/flag simulation failed for a Baizhi choice branch");
if (data.core_story_route.some((step) => step.edge && step.edge.includes("b01-c01"))) errors.push("core route contains a forbidden B01 direct deep shortcut");

const entityChecks = data.entities.map((entity) => {
  const room = rooms.get(entity.room);
  const inside = Boolean(room && contains(room, entity.position, entity.radius_m ?? 0));
  if (!inside) errors.push(`${entity.id} is outside ${entity.room} with clearance/floor`);
  return { id: entity.id, type: entity.type, room: entity.room, contained_with_radius: inside };
});
const postBio = entities.get("gh_cp_post_bio");
if (!postBio || postBio.room !== "GH-B05") errors.push("gh_cp_post_bio is not in GH-B05 Gate B safety room");
for (const checkpoint of data.checkpoint_order) if (!entities.has(checkpoint) || entities.get(checkpoint).type !== "checkpoint") errors.push(`checkpoint missing or wrong type: ${checkpoint}`);

const gateStates = Object.fromEntries(Object.keys(data.gate_states).map((state) => [state, { reachable: reachable(state), b01: reachable(state).includes("GH-B01"), b05: reachable(state).includes("GH-B05"), c01: reachable(state).includes("GH-C01"), c04: reachable(state).includes("GH-C04") }]));
const gateChecks = [
  { state: "INITIAL", pass: !gateStates.INITIAL.b01 && !gateStates.INITIAL.b05 && !gateStates.INITIAL.c01 },
  { state: "POWERED", pass: gateStates.POWERED.b01 && gateStates.POWERED.b05 && !gateStates.POWERED.c01 },
  { state: "LOCKDOWN_CLEARED", pass: gateStates.LOCKDOWN_CLEARED.b01 && gateStates.LOCKDOWN_CLEARED.b05 && !gateStates.LOCKDOWN_CLEARED.c01 },
  { state: "BAIZHI_RESOLVED", pass: gateStates.BAIZHI_RESOLVED.b01 && gateStates.BAIZHI_RESOLVED.b05 && gateStates.BAIZHI_RESOLVED.c01 },
];
if (gateChecks.some((check) => !check.pass)) errors.push("gate state reachability does not match v4 objective progression");

const documentMissing = documentConsistency();
if (documentMissing.length) errors.push(`document consistency missing: ${documentMissing.join(", ")}`);

const result = {
  schema: data.schema,
  status: errors.length ? "failed" : "passed_with_gate_evidence_limits",
  errors,
  document_consistent: documentMissing.length === 0,
  room_count: data.rooms.length,
  connector_count: data.connectors.length,
  entity_count: data.entities.length,
  bounds_m: worldBounds,
  bounds_in_target: boundsPass,
  primary_playable_envelope: { declared: primary, pass: primaryPass },
  vertical_layers: layerChecks,
  connectors: connectorChecks,
  core_story_route: { ...coreRoute, target_m: coreTarget, in_target: coreLengthPass, branch_simulations: coreSimulations },
  first_exploration_route: { ...explorationRoute, target_m: explorationTarget, in_target: explorationLengthPass, branch_simulations: explorationSimulations },
  gate_connectivity: gateStates,
  gate_checks: gateChecks,
  entities: entityChecks,
  checkpoint_order: data.checkpoint_order,
  objective_contract: data.objective_contract,
  room_id_traceability: data.room_id_traceability,
  v3_root_cause: { direct_graph_route_m: 305.569, omitted_mandatory_branch_proxy_m: 304.133, v3_mandatory_route_proxy_m: 609.702, note: "historical v3 evidence is preserved; v4 does not overwrite it" },
  transform_status: "candidate only; formal TP-09 Blockout must export final transforms",
  formal_blockout_pending: data.formal_blockout_pending,
};
fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(`AG-10 route validation: ${result.status}; errors=${errors.length}; core=${coreRoute.distance_m}m; exploration=${explorationRoute.distance_m}m; core_branches=${coreSimulations.length}; exploration_branches=${explorationSimulations.length}`);
if (errors.length) process.exitCode = 1;
