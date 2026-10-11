#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import {
  CAPSULE,
  INPUTS,
  GEOMETRY_DIR,
  boundaryCrossings,
  connectorLevel,
  corridorContains,
  corridorPolygon,
  distance,
  edgeState,
  firstExitCrossing,
  floorY,
  pathLength,
  pathSample,
  point2,
  polygonArea,
  polygonSelfIntersections,
  polylineDistance,
  polylineSelfIntersections,
  readJson,
  roomBounds,
  roomContains,
  roomHeight,
  roomPolygon,
  round,
  shortestPath,
  lastEntryCrossing,
  vadd,
  vmul,
  vsub,
  normalize,
  leftNormal,
} from "./geometry.mjs";

const OUTPUT = `${GEOMETRY_DIR}/../results/layout-geometry-ag17.json`;
const REPORT = `${GEOMETRY_DIR}/../../../../docs/GREY_HIVE_GEOMETRY_CLEARANCE_REVIEW.md`;

function assertFile(file) {
  if (!fs.existsSync(file)) throw new Error(`required input missing: ${file}`);
}
for (const file of Object.values(INPUTS)) assertFile(file);

const data = readJson(INPUTS.layout);
const layoutResult = readJson(INPUTS.layoutResult);
const coordinates = readJson(INPUTS.coordinates);
const physics = readJson(INPUTS.physics);
const rooms = new Map(data.rooms.map((room) => [room.id, room]));
const edges = new Map(data.connectors.map((edge) => [edge.id, edge]));
const requiredEdgeIds = new Set([
  ...data.core_story_route.filter((step) => step.edge).map((step) => step.edge),
]);

const errors = [];
const warnings = [];
const checks = [];
function check(id, input, threshold, actual, pass, detail = "") {
  const result = { id, input, threshold, actual, pass: Boolean(pass), detail };
  checks.push(result);
  if (!pass) errors.push({ id, detail, input, threshold, actual });
  return result;
}
function warn(id, detail, actual = null) { warnings.push({ id, detail, actual }); }
function sameFloor(a, b) { return Math.abs(a.floor_y_m - b.floor_y_m) <= 1e-7; }
function centerlineInside(room, pathPoints) { return roomContains(room, point2(pathPoints[0]), 0) && roomContains(room, point2(pathPoints.at(-1)), 0); }
function angleDegrees(a, b, c) {
  const ab = normalize(vsub(a, b));
  const cb = normalize(vsub(c, b));
  const cosine = Math.max(-1, Math.min(1, ab[0] * cb[0] + ab[1] * cb[1]));
  return Math.acos(cosine) * 180 / Math.PI;
}
function doorFramePolygon(portal, tangent, width, depth) {
  const normal = leftNormal(normalize(tangent));
  const halfWidth = width / 2;
  const halfDepth = depth / 2;
  return [
    vadd(vadd(portal, vmul(normal, halfWidth)), vmul(tangent, halfDepth)),
    vadd(vadd(portal, vmul(normal, -halfWidth)), vmul(tangent, halfDepth)),
    vadd(vadd(portal, vmul(normal, -halfWidth)), vmul(tangent, -halfDepth)),
    vadd(vadd(portal, vmul(normal, halfWidth)), vmul(tangent, -halfDepth)),
  ];
}
function portalInfo(edge, from, to) {
  const pathPoints = edge.path.map(point2);
  const fromCrossing = firstExitCrossing(pathPoints, from);
  const toCrossing = lastEntryCrossing(pathPoints, to);
  const fromTangent = fromCrossing ? normalize(vsub(pathPoints[Math.min(pathPoints.length - 1, fromCrossing.segment + 1)], pathPoints[fromCrossing.segment])) : [0, 1];
  const toTangent = toCrossing ? normalize(vsub(pathPoints[toCrossing.segment + 1], pathPoints[toCrossing.segment])) : [0, 1];
  return {
    from: fromCrossing ? { point_xz: fromCrossing.point.map((value) => round(value, 3)), segment: fromCrossing.segment, frame_polygon_xz: doorFramePolygon(fromCrossing.point, fromTangent, edge.door.blocker_width_m, edge.door.blocker_depth_m).map((point) => point.map((value) => round(value, 3))) } : null,
    to: toCrossing ? { point_xz: toCrossing.point.map((value) => round(value, 3)), segment: toCrossing.segment, frame_polygon_xz: doorFramePolygon(toCrossing.point, toTangent, edge.door.blocker_width_m, edge.door.blocker_depth_m).map((point) => point.map((value) => round(value, 3))) } : null,
  };
}

const roomGeometry = data.rooms.map((room) => {
  const polygon = roomPolygon(room);
  const selfIntersections = polygonSelfIntersections(polygon);
  const area = Math.abs(polygonArea(polygon));
  const floorCheck = Number.isFinite(room.floor_y_m) && Number.isFinite(roomHeight(room));
  check(`room.${room.id}.polygon`, { center: room.center, size_m: room.size, floor_y_m: room.floor_y_m }, { finite: true, area_m2: "> 0", self_intersections: 0 }, { area_m2: round(area), self_intersections: selfIntersections.length, floor_y_m: room.floor_y_m, ceiling_y_m: round(room.floor_y_m + roomHeight(room)) }, floorCheck && area > 0 && selfIntersections.length === 0, "candidate rectangular ground polygon; wall extrusion is pending formal Blockout");
  return { id: room.id, level: room.level, floor_y_m: room.floor_y_m, ceiling_y_m: room.floor_y_m + roomHeight(room), polygon_xz: polygon.map((point) => point.map((value) => round(value))), area_m2: round(area), size_m: room.size };
});

const connectorGeometry = data.connectors.map((edge) => {
  const from = rooms.get(edge.from);
  const to = rooms.get(edge.to);
  const centerline = edge.path.map(point2);
  const level = connectorLevel(data, edge);
  const pathSelfIntersections = polylineSelfIntersections(centerline);
  const corridor = corridorPolygon(centerline, edge.width_m);
  const corridorSelfIntersections = polygonSelfIntersections(corridor);
  const length = pathLength(centerline);
  const fromEndpoint = roomContains(from, centerline[0], 0);
  const toEndpoint = roomContains(to, centerline.at(-1), 0);
  const portal = portalInfo(edge, from, to);
  const otherRoomCrossings = [];
  for (const candidate of data.rooms) {
    if (candidate.id === edge.from || candidate.id === edge.to || !sameFloor(from, candidate)) continue;
    const samples = pathSample(centerline, 0.25).filter((point) => roomContains(candidate, point, 0));
    if (samples.length) {
      const crossings = boundaryCrossings(centerline, candidate);
      otherRoomCrossings.push({ room: candidate.id, sample_count: samples.length, crossings: crossings.map((crossing) => ({ point_xz: crossing.point.map((value) => round(value)), from_inside: crossing.from_inside, to_inside: crossing.to_inside })) });
    }
  }
  const turns = [];
  for (let i = 1; i < centerline.length - 1; i += 1) {
    const angle = angleDegrees(centerline[i - 1], centerline[i], centerline[i + 1]);
    turns.push({ vertex_index: i, point_xz: centerline[i].map((value) => round(value)), angle_deg: round(angle), available_turn_clearance_m: round(edge.width_m / 2 - CAPSULE.radius_m), pass: edge.width_m / 2 - CAPSULE.radius_m >= 0 });
  }
  const horizontalClearance = edge.width_m - 2 * CAPSULE.radius_m;
  const verticalClearance = edge.height_m - CAPSULE.height_m;
  const dimensionsPass = edge.width_m >= data.target.ordinary_corridor_min_m && edge.height_m >= data.target.door_min_height_m;
  const capsulePass = horizontalClearance >= 0 && verticalClearance >= 0;
  const portalPass = level.same_floor ? Boolean(portal.from && portal.to) : false;
  const intermediateWallPass = otherRoomCrossings.length === 0;
  const geometryPass = fromEndpoint && toEndpoint && pathSelfIntersections.length === 0 && corridorSelfIntersections.length === 0 && dimensionsPass && capsulePass && portalPass && intermediateWallPass;
  const samples = pathSample(centerline, 0.5);
  const samplePass = level.same_floor && edge.height_m >= CAPSULE.height_m && samples.every((point) => roomContains(from, point, CAPSULE.radius_m) || roomContains(to, point, CAPSULE.radius_m) || corridorContains(point, { centerline, width_m: edge.width_m, same_floor: level.same_floor, from_floor_y_m: from.floor_y_m, height_m: edge.height_m }, CAPSULE.radius_m));
  if (!fromEndpoint || !toEndpoint) errors.push({ id: `connector.${edge.id}.endpoints`, detail: "connector end is outside its declared room", actual: { fromEndpoint, toEndpoint } });
  if (pathSelfIntersections.length) errors.push({ id: `connector.${edge.id}.centerline_self_intersection`, detail: "centerline self-intersection", actual: pathSelfIntersections });
  if (corridorSelfIntersections.length) errors.push({ id: `connector.${edge.id}.corridor_polygon_self_intersection`, detail: "buffered corridor polygon self-intersection", actual: corridorSelfIntersections });
  if (!dimensionsPass) errors.push({ id: `connector.${edge.id}.dimensions`, detail: "corridor or door below v4 target", actual: { width_m: edge.width_m, height_m: edge.height_m } });
  if (!capsulePass) errors.push({ id: `connector.${edge.id}.capsule_clearance`, detail: "AG-06 capsule cannot fit", actual: { horizontalClearance, verticalClearance } });
  if (!portalPass) errors.push({ id: `connector.${edge.id}.floor_transition`, detail: "different floor Y needs explicit stair/ramp/elevator; X/Z path is not a traversable connector", actual: level });
  if (!intermediateWallPass) errors.push({ id: `connector.${edge.id}.non_door_wall_crossing`, detail: "centerline/corridor crosses another room polygon without a declared door", actual: otherRoomCrossings });
  if (!samplePass && level.same_floor) errors.push({ id: `connector.${edge.id}.sample_clearance`, detail: "sampled capsule center leaves the candidate corridor/room clearance", actual: { sample_count: samples.length } });
  for (const turn of turns) if (!turn.pass) errors.push({ id: `connector.${edge.id}.turn.${turn.vertex_index}`, detail: "turning clearance is narrower than capsule radius", actual: turn });
  check(`connector.${edge.id}.polygon`, { path_xz: centerline, width_m: edge.width_m, height_m: edge.height_m }, { finite: true, centerline_self_intersections: 0, corridor_polygon_self_intersections: 0 }, { length_m: round(length), area_m2: round(Math.abs(polygonArea(corridor))), centerline_self_intersections: pathSelfIntersections.length, corridor_polygon_self_intersections: corridorSelfIntersections.length }, pathSelfIntersections.length === 0 && corridorSelfIntersections.length === 0, "buffered corridor candidate");
  check(`connector.${edge.id}.level`, level, { same_floor: true, explicit_vertical_link_if_false: true }, { same_floor: level.same_floor, delta_y_m: level.delta_y_m, from_level: level.from_level, to_level: level.to_level }, level.same_floor, level.same_floor ? "same-floor corridor" : "BLOCKER: no ramp/stair/elevator candidate in v4");
  check(`connector.${edge.id}.capsule`, { radius_m: CAPSULE.radius_m, height_m: CAPSULE.height_m, corridor_width_m: edge.width_m, connector_height_m: edge.height_m }, { width_minus_2_radius_m: ">= 0", height_minus_capsule_m: ">= 0" }, { width_minus_2_radius_m: round(horizontalClearance), height_minus_capsule_m: round(verticalClearance), turn_count: turns.length, turn_min_clearance_m: round(turns.length ? Math.min(...turns.map((turn) => turn.available_turn_clearance_m)) : edge.width_m / 2 - CAPSULE.radius_m) }, capsulePass, "candidate clearance only; not a Rapier collider");
  check(`connector.${edge.id}.door_frame`, { door_id: edge.door.id, blocker_width_m: edge.door.blocker_width_m, blocker_depth_m: edge.door.blocker_depth_m, door_height_m: edge.door.height_m }, { blocker_depth_m: "> 0", blocker_covers_width: true, door_height_m: `>= ${data.target.door_min_height_m}` }, { blocker_covers_width: edge.door.blocker_width_m >= edge.width_m, blocker_depth_m: edge.door.blocker_depth_m, door_height_m: edge.door.height_m, from_portal: portal.from, to_portal: portal.to }, edge.door.blocker_width_m >= edge.width_m && edge.door.blocker_depth_m > 0 && edge.door.height_m >= data.target.door_min_height_m, "candidate frame only; dynamic state/collider synchronization remains TP09");
  return {
    id: edge.id,
    from: edge.from,
    to: edge.to,
    door_id: edge.door.id,
    door_state: edge.door.state,
    required_core_route: requiredEdgeIds.has(edge.id),
    centerline: centerline.map((point) => point.map((value) => round(value))),
    corridor_polygon_xz: corridor.map((point) => point.map((value) => round(value))),
    corridor_polygon_area_m2: round(Math.abs(polygonArea(corridor))),
    length_m: round(length),
    level,
    from_endpoint_inside: fromEndpoint,
    to_endpoint_inside: toEndpoint,
    portals: { from: portal.from, to: portal.to },
    door_frame: { blocker_width_m: edge.door.blocker_width_m, blocker_depth_m: edge.door.blocker_depth_m, height_m: edge.door.height_m },
    horizontal_clearance_m: round(horizontalClearance),
    vertical_clearance_m: round(verticalClearance),
    turns,
    intermediate_non_door_room_crossings: otherRoomCrossings,
    sampled_capsule_clearance: { sample_count: samples.length, max_step_m: 0.5, pass: samplePass },
    checks: { dimensions_pass: dimensionsPass, capsule_pass: capsulePass, portal_pass: portalPass, intermediate_wall_pass: intermediateWallPass, centerline_self_intersections: pathSelfIntersections.length === 0, corridor_polygon_self_intersections: corridorSelfIntersections.length === 0, geometry_pass: geometryPass },
    nav_pass: geometryPass && samplePass,
  };
});
const connectorById = new Map(connectorGeometry.map((connector) => [connector.id, connector]));

const endpointPairs = [];
const sharedPortals = [];
for (let i = 0; i < connectorGeometry.length; i += 1) {
  for (let j = i + 1; j < connectorGeometry.length; j += 1) {
    const a = connectorGeometry[i];
    const b = connectorGeometry[j];
    if (!a.level.same_floor || !b.level.same_floor || Math.abs(a.level.from_floor_y_m - b.level.from_floor_y_m) > 1e-7) continue;
    for (const sideA of ["from", "to"]) for (const sideB of ["from", "to"]) {
      const portalA = a.portals[sideA];
      const portalB = b.portals[sideB];
      if (!portalA || !portalB) continue;
      const separation = distance(portalA.point_xz, portalB.point_xz);
      const overlap = separation < (edges.get(a.id).width_m + edges.get(b.id).width_m) / 2;
      if (overlap) {
        const expectedContinuation = a.to === b.from && sideA === "to" && sideB === "from";
        const record = { connector_a: a.id, side_a: sideA, connector_b: b.id, side_b: sideB, separation_m: round(separation), threshold_m: round((edges.get(a.id).width_m + edges.get(b.id).width_m) / 2) };
        if (expectedContinuation) sharedPortals.push(record);
        else endpointPairs.push(record);
      }
    }
  }
}
check("portal.overlap", { portal_pairs: "same-floor connector door frames; expected chain continuations excluded" }, { overlapping_pairs: 0 }, { overlapping_pairs: endpointPairs.length, pairs: endpointPairs, expected_chain_shared_portals: sharedPortals }, endpointPairs.length === 0, "overlap is a geometric risk even when the graph is connected; shared chain ports are retained separately");

function makeAdjacency(stateName, excludedDoorId = null) {
  const adjacency = new Map(data.rooms.map((room) => [room.id, []]));
  for (const connector of connectorGeometry) {
    const edge = edges.get(connector.id);
    if (!connector.nav_pass || !connector.level.same_floor || edge.door.id === excludedDoorId || edgeState(data, edge, stateName) !== "Open") continue;
    adjacency.get(edge.from).push({ to: edge.to, weight: connector.length_m, edge: edge.id });
    adjacency.get(edge.to).push({ to: edge.from, weight: connector.length_m, edge: edge.id });
  }
  return adjacency;
}
function navRoute(route, stateName) {
  let total = 0;
  let room = "GH-A01";
  const segments = [];
  let failure = null;
  for (const step of route) {
    if (!step.edge) { segments.push({ edge: null, event: step.event, room, length_m: 0 }); continue; }
    const edge = edges.get(step.edge);
    const connector = connectorById.get(step.edge);
    const forward = step.direction !== "reverse";
    const start = forward ? edge.from : edge.to;
    const end = forward ? edge.to : edge.from;
    const state = edgeState(data, edge, stateName);
    const pass = connector.nav_pass && connector.level.same_floor && state === "Open" && start === room;
    if (!pass && !failure) {
      failure = { edge: edge.id, from: start, to: end, state, room_before: room, connector_nav_pass: connector.nav_pass, same_floor: connector.level.same_floor, reason: start !== room ? "route discontinuity" : !connector.level.same_floor ? "missing explicit vertical connector" : state !== "Open" ? "door is closed in state" : "geometry/capsule clearance failed" };
      segments.push({ edge: edge.id, direction: step.direction, from: start, to: end, length_m: null, sampled_nav_pass: false, state });
      break;
    }
    if (pass) {
      total += connector.length_m;
      room = end;
      segments.push({ edge: edge.id, direction: step.direction, from: start, to: end, length_m: connector.length_m, sampled_nav_pass: true });
    }
  }
  return { start_room: "GH-A01", end_room: room, distance_m: failure ? null : round(total), partial_distance_m: round(total), complete: !failure && room === "GH-C04", failure, segments };
}

const navRoutes = {
  core_story: navRoute(data.core_story_route, "BAIZHI_RESOLVED"),
  first_exploration: navRoute(data.first_exploration_route, "BAIZHI_RESOLVED"),
};
const centerlineRoutes = {
  core_story_m: layoutResult.core_story_route?.distance_m ?? 318.946,
  first_exploration_m: layoutResult.first_exploration_route?.distance_m ?? 671.267,
};
for (const [name, route] of Object.entries(navRoutes)) {
  const target = name === "core_story" ? [250, 350] : [500, 800];
  check(`route.${name}.sampled_nav_graph`, { route: name, state: "BAIZHI_RESOLVED", sampled_step_m: 0.5 }, { complete: true, distance_m: `${target[0]}-${target[1]}` }, { complete: route.complete, distance_m: route.distance_m, partial_distance_m: route.partial_distance_m, failure: route.failure }, route.complete && route.distance_m >= target[0] && route.distance_m <= target[1], route.complete ? "sampled capsule-clearance nav proxy" : "BLOCKER: no complete traversable nav path; centerline is retained only as comparison evidence");
}

const gateIds = ["gh_gate_a", "gh_gate_b"];
const gateChecks = gateIds.map((doorId) => {
  const edge = data.connectors.find((candidate) => candidate.door.id === doorId);
  const connector = connectorById.get(edge.id);
  const bypassAdjacency = makeAdjacency("BAIZHI_RESOLVED", doorId);
  const bypass = shortestPath(bypassAdjacency, edge.from, edge.to);
  const openAdjacency = makeAdjacency("BAIZHI_RESOLVED");
  const openPath = shortestPath(openAdjacency, edge.from, edge.to);
  const closedAdjacency = makeAdjacency("INITIAL");
  const closedPath = shortestPath(closedAdjacency, edge.from, edge.to);
  const landingOffset = Math.max(CAPSULE.radius_m + 0.15, 0.5);
  const centerline = connector.centerline;
  const fromSegment = connector.portals.from?.segment ?? 0;
  const toSegment = connector.portals.to?.segment ?? Math.max(0, centerline.length - 2);
  const fromTangent = normalize(vsub(centerline[Math.min(fromSegment + 1, centerline.length - 1)], centerline[fromSegment]));
  const toTangent = normalize(vsub(centerline[Math.min(toSegment + 1, centerline.length - 1)], centerline[toSegment]));
  const fromLanding = vadd(connector.portals.from?.point_xz ?? centerline[0], vmul(fromTangent, -landingOffset));
  const toLanding = vadd(connector.portals.to?.point_xz ?? centerline.at(-1), vmul(toTangent, landingOffset));
  const fromRoom = rooms.get(edge.from);
  const toRoom = rooms.get(edge.to);
  const fromSafe = roomContains(fromRoom, fromLanding, CAPSULE.radius_m) && roomHeight(fromRoom) >= CAPSULE.height_m;
  const toSafe = roomContains(toRoom, toLanding, CAPSULE.radius_m) && roomHeight(toRoom) >= CAPSULE.height_m;
  check(`gate.${doorId}.closed_no_bypass`, { excluded_door: doorId, graph_state: "INITIAL" }, { bypass_reachable: false }, { bypass_reachable: bypass.reachable, bypass_distance_m: bypass.distance_m, path_nodes: bypass.nodes }, !bypass.reachable, "graph-only anti-bypass check; final physical walls/colliders remain TP09");
  check(`gate.${doorId}.open_connection`, { door: doorId, graph_state: "BAIZHI_RESOLVED" }, { reachable_when_open: true }, { reachable_when_open: openPath.reachable, distance_m: openPath.distance_m, connector_nav_pass: connector.nav_pass, same_floor: connector.level.same_floor }, openPath.reachable, connector.level.same_floor ? "open-state geometry reachability" : "BLOCKER: open state has no vertical transition candidate");
  check(`gate.${doorId}.landing`, { radius_m: CAPSULE.radius_m, height_m: CAPSULE.height_m, landing_offset_m: landingOffset }, { from_side: true, to_side: true }, { from_side: fromSafe, to_side: toSafe, from_landing_xz: fromLanding.map((value) => round(value)), to_landing_xz: toLanding.map((value) => round(value)) }, fromSafe && toSafe, "capsule-safe candidate landing only; not a real KCC spawn/collider");
  return {
    door_id: doorId,
    edge_id: edge.id,
    from: edge.from,
    to: edge.to,
    state_open_test: { state: "BAIZHI_RESOLVED", reachable: openPath.reachable, distance_m: openPath.distance_m },
    state_closed_test: { state: "INITIAL", reachable: closedPath.reachable },
    bypass_test: { excluding_gate_edge: true, reachable: bypass.reachable, distance_m: bypass.distance_m, nodes: bypass.nodes },
    landing_test: { offset_m: landingOffset, from_side: fromSafe, to_side: toSafe, from_landing_xz: fromLanding.map((value) => round(value)), to_landing_xz: toLanding.map((value) => round(value)) },
    connector: { level: connector.level, horizontal_clearance_m: connector.horizontal_clearance_m, vertical_clearance_m: connector.vertical_clearance_m, nav_pass: connector.nav_pass },
  };
});

function reachableRooms(stateName) {
  const adjacency = makeAdjacency(stateName);
  const seen = new Set(["GH-A01"]);
  const queue = ["GH-A01"];
  while (queue.length) for (const edge of adjacency.get(queue.shift()) ?? []) if (!seen.has(edge.to)) { seen.add(edge.to); queue.push(edge.to); }
  return [...seen].sort();
}
const gateStateReachability = Object.fromEntries(Object.keys(data.gate_states).map((state) => [state, { reachable_rooms: reachableRooms(state), b01: reachableRooms(state).includes("GH-B01"), b05: reachableRooms(state).includes("GH-B05"), c01: reachableRooms(state).includes("GH-C01"), c03: reachableRooms(state).includes("GH-C03"), c04: reachableRooms(state).includes("GH-C04") }]));

const checkpointIds = ["gh_cp_airlock", "gh_cp_power", "gh_cp_central", "gh_cp_post_bio", "gh_cp_pre_sentinel", "gh_cp_exit"];
const checkpointChecks = checkpointIds.map((id) => {
  const entity = data.entities.find((candidate) => candidate.id === id);
  const room = entity ? rooms.get(entity.room) : null;
  const point = entity?.position ? point2(entity.position) : null;
  const safe = Boolean(entity && room && roomContains(room, point, CAPSULE.radius_m) && roomHeight(room) >= CAPSULE.height_m);
  check(`checkpoint.${id}.capsule_landing`, { entity_position_m: entity?.position ?? null, room: entity?.room ?? null, capsule_radius_m: CAPSULE.radius_m, capsule_height_m: CAPSULE.height_m }, { contained_with_capsule_margin: true }, { contained_with_capsule_margin: safe, room_bounds: room ? roomBounds(room) : null }, safe, "candidate room margin; checkpoint trigger/collider not authored");
  return { id, room: entity?.room ?? null, floor_y_m: room?.floor_y_m ?? null, position_m: entity?.position ?? null, capsule_safe: safe };
});
const sentinel = rooms.get("GH-C03");
const sentinelMinDimension = Math.min(sentinel.size[0], sentinel.size[1]);
check("arena.GH-C03.minimum_dimension", { room: "GH-C03", size_m: sentinel.size }, { min_dimension_m: data.target.sentinel_arena_min_dimension_m }, { min_dimension_m: sentinelMinDimension, size_m: sentinel.size }, sentinelMinDimension >= data.target.sentinel_arena_min_dimension_m, "v4 target is 18m; current candidate is 24x16m");

const explicitVerticalMissing = connectorGeometry.filter((connector) => !connector.level.same_floor).map((connector) => ({ id: connector.id, from: connector.from, to: connector.to, from_floor_y_m: connector.level.from_floor_y_m, to_floor_y_m: connector.level.to_floor_y_m, delta_y_m: connector.level.delta_y_m, required_for_core: connector.required_core_route, required_action: "add stair/ramp/elevator candidate and validate slope/landings; do not rely on X/Z overlap" }));
for (const missing of explicitVerticalMissing) warn(`vertical.${missing.id}`, "cross-floor connector lacks explicit vertical traversal geometry", missing);

const sourceChecks = {
  ag05_coordinate_basis: {
    status: coordinates.status,
    position_formula: coordinates.candidate_contract?.position_formula,
    handedness: coordinates.candidate_contract?.handedness,
    axes: coordinates.candidate_contract?.axes,
    pass: coordinates.status === "candidate_adapter_only" && coordinates.candidate_contract?.position_formula === "B.x=R.x; B.y=R.y; B.z=R.z",
  },
  ag06_capsule: {
    status: physics.status,
    radius_m: physics.platform_contract?.capsule_radius_m,
    height_m: physics.platform_contract?.capsule_total_height_m,
    foot_anchor: physics.platform_contract?.foot_anchor,
    pass: physics.status === "isolated_poc_passed" && physics.platform_contract?.foot_anchor === "capsule bottom at y=0",
  },
};
check("source.AG05.coordinate_basis", { status: coordinates.status, contract: coordinates.candidate_contract }, { candidate_only: true, identity_position_mapping: true }, sourceChecks.ag05_coordinate_basis, sourceChecks.ag05_coordinate_basis.pass, "read-only coordinate evidence");
check("source.AG06.capsule", { status: physics.status, platform_contract: physics.platform_contract }, { radius_m: CAPSULE.radius_m, height_m: CAPSULE.height_m, foot_anchor: "capsule bottom at y=0" }, sourceChecks.ag06_capsule, sourceChecks.ag06_capsule.pass, "read-only isolated Rapier evidence");

const routeTargets = {
  core_story: { centerline_m: centerlineRoutes.core_story_m, target_m: data.target.core_story_route_m, nav: navRoutes.core_story },
  first_exploration: { centerline_m: centerlineRoutes.first_exploration_m, target_m: data.target.first_exploration_route_m, nav: navRoutes.first_exploration },
};
for (const route of Object.values(routeTargets)) {
  route.centerline_delta_to_target_m = { low: round(route.centerline_m - route.target_m[0]), high: round(route.centerline_m - route.target_m[1]) };
  route.nav_delta_to_target_m = route.nav.complete ? { low: round(route.nav.distance_m - route.target_m[0]), high: round(route.nav.distance_m - route.target_m[1]) } : null;
}

const actualStatus = errors.length ? "blocked_geometry_candidate" : "isolated_geometry_passed_with_tp09_limits";
const result = {
  schema: "grey_hive_layout_geometry_ag17_v1",
  status: actualStatus,
  generated_at_utc: new Date().toISOString(),
  scope: "isolated polygon/door-frame/capsule-clearance evidence; not Babylon Blockout and not Rust/Rapier collider validation",
  inputs: {
    layout_v4: INPUTS.layout,
    layout_v4_result: INPUTS.layoutResult,
    coordinates_ag05: INPUTS.coordinates,
    physics_ag06: INPUTS.physics,
    layout_v4_status: data.status,
    layout_v4_result_status: layoutResult.status,
  },
  coordinate_system: data.axes,
  capsule_contract: CAPSULE,
  thresholds: {
    ordinary_corridor_min_m: data.target.ordinary_corridor_min_m,
    door_min_height_m: data.target.door_min_height_m,
    sentinel_arena_min_dimension_m: data.target.sentinel_arena_min_dimension_m,
    sampled_nav_max_step_m: 0.5,
    non_door_wall_crossing: 0,
    required_floor_transition: "explicit ramp/stair/elevator candidate",
  },
  source_checks: sourceChecks,
  room_polygons: roomGeometry,
  connectors: connectorGeometry,
  portal_overlap: { overlapping_pairs: endpointPairs.length, pairs: endpointPairs, expected_chain_shared_portals: sharedPortals },
  gate_checks: gateChecks,
  gate_state_reachability: gateStateReachability,
  checkpoints: checkpointChecks,
  sentinel_arena: { room: "GH-C03", size_m: sentinel.size, floor_y_m: sentinel.floor_y_m, min_dimension_m: sentinelMinDimension, target_min_dimension_m: data.target.sentinel_arena_min_dimension_m, pass: sentinelMinDimension >= data.target.sentinel_arena_min_dimension_m },
  vertical_transition_gaps: explicitVerticalMissing,
  route_recalculation: routeTargets,
  checks,
  errors,
  warnings,
  tp09_gaps: [
    "final Babylon scene export must replace room/corridor candidates with authored wall thickness and final transforms",
    "real Rapier wall, floor, ceiling, door-frame and dynamic-gate colliders are not instantiated here",
    "KCC/camera clearance through every authored polygon and door timeline is not proven by AG-06 isolated scenarios",
    "dynamic gate animation and collider synchronization for gh_gate_a/gh_gate_b remain unverified",
    "cross-floor Gate B needs an explicit stair/ramp/elevator candidate; open flag alone is insufficient",
    "GH-C03 is 24x16m against the v4 18m minimum dimension target",
    "checkpoint trigger volumes, Sentinel arena combat clearance and real spawn safety are not authored",
  ],
  formal_gate_note: "isolated geometry pass, if achieved, does not imply Gate I or TP09 acceptance",
};
fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
fs.writeFileSync(OUTPUT, `${JSON.stringify(result, null, 2)}\n`, "utf8");

console.log(`AG-17 geometry validation: ${result.status}; errors=${errors.length}; warnings=${warnings.length}; connectors=${connectorGeometry.length}; core_nav=${navRoutes.core_story.distance_m ?? "BLOCKED"}; exploration_nav=${navRoutes.first_exploration.distance_m ?? "BLOCKED"}`);
if (errors.length) process.exitCode = 1;
