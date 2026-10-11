import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const load = (relative) => JSON.parse(fs.readFileSync(path.resolve(root, relative), "utf8"));
const baseLayout = load("layout-repair/layout-v5.json");
const v7 = load("layout-core-length/layout-v7.json");
const ag18 = load("results/layout-repair-ag18.json");
const ag22 = load("results/layout-optional-ag22.json");
const ag23 = load("results/layout-core-length-ag23.json");
const tp04b2 = load("results/physics-gate-b-tp04b2.json");

const errors = [];
const warnings = [];
const check = (condition, code, detail) => {
  if (!condition) errors.push({ code, detail });
};
const warn = (code, connectorId, detail) => warnings.push({ code, connector_id: connectorId, detail });
const finite = (value) => Number.isFinite(value);
const distance3 = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
const pathLength = (points) => points.slice(1).reduce((sum, point, index) => sum + distance3(points[index], point), 0);

const rooms = new Map(baseLayout.rooms.map((room) => [room.id, room]));
const replacementId = v7.core_length_repair.replacement_id;
const baseCore = baseLayout.connectors.find((connector) => connector.id === "edge-b01-b02");
const v7Core = {
  ...baseCore,
  id: replacementId,
  width_m: v7.core_length_repair.width_m,
  height_m: v7.core_length_repair.height_m,
  path: v7.core_length_repair.path,
  door_ids: v7.core_length_repair.door_blockers.map((door) => door.id),
};
const activeSources = baseLayout.connectors.map((connector) => connector.id === "edge-b01-b02" ? v7Core : connector);
const evidence = new Map(ag18.connectors.map((connector) => [connector.id, connector]));
evidence.set(replacementId, {
  id: replacementId,
  length_m: v7.core_length_repair.v7_length_m,
  sample_count: v7.core_length_repair.sampled_nav.sample_count,
  max_step_m: v7.core_length_repair.sampled_nav.max_step_m,
  nav_pass: true,
  intermediate_non_door_room_crossings: v7.core_length_repair.intermediate_non_door_room_crossings,
});
const routeEntries = [...baseLayout.core_story_route, ...baseLayout.first_exploration_route].filter((entry) => entry.edge);
const excludedGateB = new Set(["edge-b05-c01"]);
const inventorySources = activeSources.filter((connector) => !excludedGateB.has(connector.id));

const graph = new Map();
for (const connector of activeSources) {
  if (!graph.has(connector.from)) graph.set(connector.from, new Set());
  if (!graph.has(connector.to)) graph.set(connector.to, new Set());
  graph.get(connector.from).add(connector.to);
  graph.get(connector.to).add(connector.from);
}

const hasAlternateGraphPath = (connectorId, from, to) => {
  const queue = [from];
  const visited = new Set([from]);
  while (queue.length) {
    const current = queue.shift();
    for (const connector of activeSources) {
      if (connector.id === connectorId) continue;
      let next = null;
      if (connector.from === current) next = connector.to;
      if (connector.to === current) next = connector.from;
      if (next && !visited.has(next)) {
        if (next === to) return true;
        visited.add(next);
        queue.push(next);
      }
    }
  }
  return false;
};

const classify = (connector, floorDelta) => {
  if (connector.vertical) {
    if (connector.vertical.kind === "stair") return "cross_level_stair";
    return "dynamic_platform_elevator";
  }
  if (Math.abs(floorDelta) > 0.001) return "cross_level_unspecified";
  return "same_level_static_door_channel";
};

const prerequisiteFor = (doorId) => {
  if (doorId === "gh_gate_a") return ["hive_power"];
  if (doorId === "gh_gate_b") return ["hive_lockdown"];
  return [];
};

const inventory = inventorySources.map((connector) => {
  const from = rooms.get(connector.from);
  const to = rooms.get(connector.to);
  const measured = evidence.get(connector.id);
  const floorDelta = to && from ? to.floor_y_m - from.floor_y_m : null;
  const centerlineLength = pathLength(connector.path);
  const routeDirections = connector.id === replacementId
    ? ["forward", "reverse"]
    : [...new Set(routeEntries.filter((entry) => entry.edge === connector.id).map((entry) => entry.direction))];
  const directionality = routeDirections.includes("forward") && routeDirections.includes("reverse") ? "bidirectional_candidate" : "unknown";
  const door = connector.door ?? null;
  const vertical = connector.vertical ?? null;
  const nominalWidthClearance = connector.width_m - 2 * baseLayout.target.capsule_radius_m;
  const nominalHeightClearance = connector.height_m - baseLayout.target.capsule_height_m;
  const alternatePath = hasAlternateGraphPath(connector.id, connector.from, connector.to);
  const intermediateCrossings = measured?.intermediate_non_door_room_crossings ?? [];

  check(Boolean(from) && Boolean(to), "E_ENDPOINT_MISSING", `${connector.id}: ${connector.from}->${connector.to}`);
  check(Boolean(measured), "E_EVIDENCE_MISSING", connector.id);
  check(!inventorySources.some((other) => other !== connector && other.id === connector.id), "E_DUPLICATE_ID", connector.id);
  check(Math.abs(floorDelta ?? 0) <= 0.001 ? !connector.vertical : Boolean(connector.vertical), "E_VERTICAL_EXPLICITNESS", `${connector.id}: floor_delta=${floorDelta}, vertical=${Boolean(connector.vertical)}`);
  if (intermediateCrossings.length) warn("W_NON_DOOR_ROOM_CROSSING", connector.id, JSON.stringify(intermediateCrossings));
  if (!connector.directionality) warn("W_DIRECTION_UNDECLARED", connector.id, `route evidence inferred ${directionality}; source has no explicit one-way/bidirectional declaration`);
  warn("W_FORMAL_COLLIDER_UNKNOWN", connector.id, "v5 formal_blockout_pending still includes final Rapier floor/wall/door/elevator/stair colliders");
  warn("W_NET_CLEARANCE_UNKNOWN", connector.id, "source exposes candidate width/height but not a formal net clearance or collider bake");
  if (door && alternatePath) warn("W_CLOSED_DOOR_GRAPH_BYPASS_CANDIDATE", connector.id, "removing this door still leaves an alternate candidate graph path");
  if (door && !alternatePath) warn("W_CLOSED_DOOR_GEOMETRY_UNPROVEN", connector.id, "candidate graph has no alternate path, but closed-door geometric bypass is not Rapier-proven");
  if (Math.abs(centerlineLength - (measured?.length_m ?? centerlineLength)) > 0.001) warn("W_LENGTH_SOURCE_DISAGREEMENT", connector.id, `layout path=${centerlineLength.toFixed(3)}m, AG-18 measured=${measured?.length_m}m`);

  return {
    id: connector.id,
    category: classify(connector, floorDelta),
    endpoints: {
      from_room: connector.from,
      to_room: connector.to,
      from_floor_y_m: from?.floor_y_m ?? null,
      to_floor_y_m: to?.floor_y_m ?? null,
      floor_delta_m: floorDelta,
      start_position_m: connector.path[0],
      end_position_m: connector.path.at(-1),
    },
    access: {
      door_id: door?.id ?? null,
      door_ids: connector.door_ids ?? (door ? [door.id] : []),
      door_state: door?.state ?? null,
      door_transition: door?.transition ?? null,
      door_open_states: vertical?.door_states ?? (door ? [door.state] : []),
      prerequisites: prerequisiteFor(door?.id),
      directionality_declared: connector.directionality ?? null,
      directionality_inferred: directionality,
      route_directions_seen: routeDirections,
    },
    candidate_geometry: {
      centerline_length_m: Number(centerlineLength.toFixed(3)),
      evidence_length_m: measured?.length_m ?? null,
      width_m: connector.width_m,
      height_m: connector.height_m,
      nominal_capsule_lateral_clearance_m: Number(nominalWidthClearance.toFixed(3)),
      nominal_capsule_vertical_clearance_m: Number(nominalHeightClearance.toFixed(3)),
      path_sample_count: measured?.sample_count ?? null,
      path_max_step_m: measured?.max_step_m ?? null,
      intermediate_non_door_room_crossings: intermediateCrossings,
      formal_collision_status: "unknown",
    },
    vertical: vertical ? {
      required: true,
      kind: vertical.kind,
      delta_y_m: vertical.rise_m,
      run_m: vertical.run_m ?? null,
      slope_degrees: vertical.slope_degrees ?? null,
      stops_y_m: vertical.stops_y_m ?? vertical.landings_y_m ?? null,
      cabin_width_m: vertical.cabin_width_m ?? null,
      cabin_height_m: vertical.cabin_height_m ?? null,
      landings: vertical.enter_exit_landings ?? false,
      handrail: vertical.handrail ?? false,
      motion_parameters: "unknown",
    } : { required: false },
    evidence: {
      source_layout: "layout-repair/layout-v5.json",
      source_result: "results/layout-repair-ag18.json",
      ag18_nav_pass: measured?.nav_pass === true,
      ag22_policy_read_only: ag22.status,
      ag23_v7_read_only: ag23.status,
      evidence_level: "isolated_layout_candidate",
    },
    static_checks: {
      endpoint_exists: Boolean(from) && Boolean(to),
      id_unique: true,
      vertical_explicit: Math.abs(floorDelta ?? 0) <= 0.001 ? !vertical : Boolean(vertical),
      obvious_non_door_wall_crossing: intermediateCrossings.length === 0 ? "not_found_in_source_evidence" : "present",
      closed_door_bypass_candidate: alternatePath,
      closed_door_geometry: "unknown",
    },
  };
});

if (tp04b2.source.replacement_connector !== "edge-b05-c01") {
  warnings.push({
    code: "W_TP04B2_SCOPE_MISMATCH",
    connector_id: "edge-b05-c01",
    detail: `TP-04B2 read-only result names ${tp04b2.source.replacement_connector}; v7 gh_gate_b is edge-b05-c01. B2 was not rerun or modified.`,
  });
}

const counts = {
  ag18_v5_logical_connectors: baseLayout.connectors.length,
  active_v7_logical_connectors: activeSources.length,
  gate_b_logical_connector_excluded: [...excludedGateB].length,
  remaining_connector_count: inventory.length,
  historical_remaining_16_check: inventory.length === 16,
  v7_gate_b_replacement_reference: v7.core_length_repair.replacement_id,
  v7_gate_b_b2_result_reference: tp04b2.schema,
};

const categories = {
  same_level_static_door_channel: inventory.filter((item) => item.category === "same_level_static_door_channel").map((item) => item.id),
  same_level_slope_or_ramp: inventory.filter((item) => item.category === "same_level_slope_or_ramp").map((item) => item.id),
  cross_level_stair: inventory.filter((item) => item.category === "cross_level_stair").map((item) => item.id),
  cross_level_ladder: inventory.filter((item) => item.category === "cross_level_ladder").map((item) => item.id),
  dynamic_platform_elevator: inventory.filter((item) => item.category === "dynamic_platform_elevator").map((item) => item.id),
  special_one_way: inventory.filter((item) => item.access.directionality_inferred === "one_way_candidate").map((item) => item.id),
};

const slices = [
  {
    priority: "P0",
    slice_id: "AG36_STATIC_MAINLINE_DOORS",
    connector_ids: ["edge-a01-a02", "edge-a02-a03", "edge-a03-a04", "edge-a04-b01", "edge-b01-b02-core-v7", "edge-b01-b03", "edge-b03-b04", "edge-b01-b05", "edge-c01-c02", "edge-c02-c03", "edge-c03-c04"],
    kind: "static_door_channel",
    proposed_new_directory: "tools/continuous-tps-architecture-probe/physics-static-mainline-doors/",
    proposed_result: "tools/continuous-tps-architecture-probe/results/physics-static-mainline-doors-ag36.json",
    proposed_report: "docs/CONTINUOUS_PHYSICS_STATIC_MAINLINE_DOORS_REVIEW.md",
    dependencies: ["v7 room/door endpoints", "Gate A state for edge-a04-b01", "Gate B/B2 only as a downstream reference"],
    required_evidence: ["Rapier static floor/wall/door blockers", "KCC capsule clearance", "closed-door corner and bypass attempts", "camera/nav blockout export"],
    reason: "Global route spine; static-door failures affect shortest path and all downstream route claims.",
  },
  {
    priority: "P1",
    slice_id: "AG37_STAIR_B01_S03",
    connector_ids: ["edge-b01-s03"],
    kind: "stair",
    proposed_new_directory: "tools/continuous-tps-architecture-probe/physics-stair-b01-s03/",
    proposed_result: "tools/continuous-tps-architecture-probe/results/physics-stair-b01-s03-ag37.json",
    proposed_report: "docs/CONTINUOUS_PHYSICS_STAIR_B01_S03_REVIEW.md",
    dependencies: ["AG-18 repaired non-door crossing evidence", "12m rise / 20m run / 30.96deg candidate"],
    required_evidence: ["Rapier stair solids and landings", "KCC autostep/slope threshold", "handrail and wall containment", "camera/nav/Blockout path"],
    reason: "Cross-level optional branch with explicit historical wall-crossing repair; high geometric risk.",
  },
  {
    priority: "P1",
    slice_id: "AG38_ELEVATORS_OPTIONAL_VERTICAL",
    connector_ids: ["edge-a03-s01", "edge-b05-s04", "edge-b03-h01"],
    kind: "dynamic_platform_elevator",
    proposed_new_directory: "tools/continuous-tps-architecture-probe/physics-optional-elevators/",
    proposed_result: "tools/continuous-tps-architecture-probe/results/physics-optional-elevators-ag38.json",
    proposed_report: "docs/CONTINUOUS_PHYSICS_OPTIONAL_ELEVATORS_REVIEW.md",
    dependencies: ["explicit stops and landing data from v5", "motion speed/acceleration/door timing still unknown"],
    required_evidence: ["Rapier kinematic platform/cabin support", "entry/exit and door collision synchronization", "fall/recovery cases", "camera/nav/Blockout export"],
    reason: "Three cross-level dynamic candidates; unresolved motion parameters make them formal blockers even where stops are listed.",
  },
  {
    priority: "P2",
    slice_id: "AG39_STATIC_OPTIONAL_OFFICE",
    connector_ids: ["edge-b01-s02"],
    kind: "static_door_channel",
    proposed_new_directory: "tools/continuous-tps-architecture-probe/physics-static-optional-office/",
    proposed_result: "tools/continuous-tps-architecture-probe/results/physics-static-optional-office-ag39.json",
    proposed_report: "docs/CONTINUOUS_PHYSICS_STATIC_OPTIONAL_OFFICE_REVIEW.md",
    dependencies: ["optional branch semantics", "v7 route policy remains read-only"],
    required_evidence: ["Rapier static door/wall blockers", "KCC clearance", "optional-branch nav/camera blockout"],
    reason: "Same-level optional branch with no stated vertical risk; can run independently after the mainline static slice.",
  },
];

const result = {
  schema: "grey_hive_connector_inventory_ag35_v1",
  status: errors.length === 0 ? "passed_readonly_inventory_with_unknowns" : "blocked_readonly_inventory",
  formal_gate_status: "blocked",
  scope: "Gate B excluded from the inventory; v7, AG-18/22/23 and TP-04A/B1/B2 evidence read-only",
  source: {
    v7: "layout-core-length/layout-v7.json",
    ag18_layout: "layout-repair/layout-v5.json",
    ag18_result: "results/layout-repair-ag18.json",
    ag22_result: "results/layout-optional-ag22.json",
    ag23_result: "results/layout-core-length-ag23.json",
    tp04b2_result: "results/physics-gate-b-tp04b2.json",
  },
  count_audit: counts,
  gate_b_reference: {
    excluded_id: "edge-b05-c01",
    v7_reference_id: "edge-b05-c01",
    b2_status: tp04b2.status,
    b2_machine_pass: tp04b2.machine_pass,
    b2_blockers_preserved: tp04b2.formal_blockers,
    not_rerun: true,
    scope_anomaly: tp04b2.source.replacement_connector === "edge-b01-b02-core-v7"
      ? "TP-04B2 result is read-only evidence for edge-b01-b02-core-v7, while v7 gh_gate_b is edge-b05-c01; no B2 file was changed or rerun."
      : null,
  },
  categories,
  connectors: inventory,
  next_parallel_physics_slices: slices,
  warnings,
  errors,
  formal_gate_notes: [
    "Inventory is not a formal nav proof and does not bake Rapier colliders.",
    "All connector formal collision statuses remain unknown until isolated physics slices and final Blockout export exist.",
    "B2's v7 1.8m net-height and missing motion-parameter blockers are preserved; Gate C and Gate I remain blocked.",
  ],
};

const output = path.resolve(root, "results/connector-inventory-ag35.json");
fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`);
console.log(`AG-35 connector inventory: ${result.status}; total=${counts.ag18_v5_logical_connectors}; excluded_gate_b=${counts.gate_b_logical_connector_excluded}; remaining=${counts.remaining_connector_count}; warnings=${warnings.length}; errors=${errors.length}`);
process.exitCode = errors.length === 0 ? 0 : 1;
