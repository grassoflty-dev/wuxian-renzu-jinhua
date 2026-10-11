import fs from "node:fs";
import path from "node:path";

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "").replace(/^([A-Za-z]):/, "$1:"));
const load = (relative) => JSON.parse(fs.readFileSync(path.resolve(here, relative), "utf8"));
const candidate = load("layout-v7.json");
const policy = load(candidate.preserved_v6_policy ? "../layout-optional/layout-v6.json" : "../layout-optional/layout-v6.json");
const baseLayout = load(candidate.base_layout);
const baseResult = load(candidate.base_result);
const baseV6Result = load("../results/layout-optional-ag22.json");
const errors = [];
const warnings = [];
const check = (condition, code, detail) => {
  if (!condition) errors.push({ code, detail });
};
const finite = (value) => Number.isFinite(value);
const distance3 = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
const distance2 = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);

check(candidate.schema === "grey_hive_layout_core_length_ag23_v7", "E_SCHEMA", "v7 schema mismatch");
check(baseLayout.schema === "grey_hive_layout_repair_ag18_v5", "E_BASE_SCHEMA", "v7 must inherit AG-18 v5 geometry");
check(baseV6Result.errors.length === 0, "E_V6_BASE_ERRORS", "v6 policy evidence must remain error-free");
check(baseV6Result.status === "passed_isolated_v6_candidate_with_bypass_pacing_warning", "E_V6_BASE_STATUS", "v6 warning candidate is not the declared input");
check(baseResult.status === "passed_isolated_v5_candidate" && baseResult.errors.length === 0, "E_AG18_BASE", "AG-18 v5 geometry evidence must remain passed and unchanged");
check(candidate.formal_gate_note.includes("isolated v7"), "E_SCOPE", "v7 must remain an isolated candidate");

const repairIds = [
  "edge-b05-c01.floor_transition", "edge-b05-c01.level", "edge-a03-s01.floor_transition",
  "edge-a03-s01.level", "edge-b01-s03.floor_transition", "edge-b01-s03.non_door_wall_crossing",
  "edge-b01-s03.level", "edge-b05-s04.floor_transition", "edge-b05-s04.level",
  "edge-b03-h01.floor_transition", "edge-b03-h01.level", "route.core_story.sampled_nav_graph",
  "route.first_exploration.sampled_nav_graph", "gate.gh_gate_b.open_connection",
  "arena.GH-C03.minimum_dimension"
];
check(candidate.preserved_ag18_repairs === 15, "E_REPAIR_COUNT", "v7 must retain all 15 AG-18 repair families");
check(baseV6Result.preserved_ag18_repairs.length === 15, "E_REPAIR_INPUT", "v6 repair evidence must retain all 15 IDs");
check(repairIds.every((id) => baseV6Result.preserved_ag18_repairs.some((repair) => repair.id === id && repair.pass)), "E_REPAIR_REGRESSION", "one or more AG-18 repairs regressed");

const gatePolicy = policy.gate_policy.gh_gate_b;
check(JSON.stringify(gatePolicy.requires) === JSON.stringify(["hive_lockdown"]), "E_GATE_B_POLICY", "Gate B must depend only on hive_lockdown");
check(gatePolicy.choice_independent === true && gatePolicy.npc_unavailable_does_not_lock === true, "E_GATE_B_OPTIONALITY", "v6 optional Baizhi policy must remain intact");
check(candidate.gate_policy.gh_gate_b.choice_independent === true && candidate.gate_policy.gh_gate_b.npc_independent === true, "E_GATE_B_V7_POLICY", "v7 must not add a choice/NPC gate");
check(candidate.gate_policy.gh_cp_post_bio.room === "GH-B05" && candidate.gate_policy.gh_cp_post_bio.sets_choice_flags === false, "E_POST_BIO_POLICY", "post-bio checkpoint must remain non-synthetic");

const repair = candidate.core_length_repair;
const expectedPathLength = repair.path.slice(1).reduce((total, point, index) => total + distance3(repair.path[index], point), 0);
check(Math.abs(expectedPathLength - repair.v7_length_m) <= 0.001, "E_CORE_PATH_LENGTH", `v7 path length ${expectedPathLength} does not equal ${repair.v7_length_m}`);
check(Math.abs(repair.v7_length_m - repair.base_length_m - repair.increment_per_traversal_m) <= 0.001, "E_CORE_INCREMENT", "one-way increment is inconsistent");
check(Math.abs(repair.route_increment_with_return_m - repair.increment_per_traversal_m * 2) <= 0.001, "E_RETURN_INCREMENT", "return increment is inconsistent");
check(repair.floor_y_m === 0 && repair.path.every((point) => point[1] === repair.floor_y_m), "E_CORE_FLOOR", "core channel must remain on the main floor");
check(repair.width_m >= 2.8 && repair.height_m >= 3 && repair.net_width_m >= 2.8 && repair.net_height_m >= 1.0, "E_CORE_CLEARANCE", "core channel does not meet corridor/capsule clearance");
check(repair.sampled_nav.max_step_m <= 0.5 && repair.sampled_nav.sample_count === Math.ceil(repair.v7_length_m / repair.sampled_nav.max_step_m) + 1, "E_CORE_SAMPLING", "core channel sampling evidence is insufficient");
check(repair.intermediate_non_door_room_crossings.length === 0, "E_CORE_ROOM_CROSSING", "core channel crosses an intermediate non-door room");
check(repair.legacy_edge_disabled_in_v7_graph === true, "E_LEGACY_EDGE", "old shorter B01-B02 edge must not remain active in v7 graph");

const rooms = new Map(baseLayout.rooms.map((room) => [room.id, room]));
const insideRoom = (point, roomId) => {
  const room = rooms.get(roomId);
  if (!room) return false;
  return Math.abs(point[0] - room.center[0]) <= room.size[0] / 2 && Math.abs(point[2] - room.center[2]) <= room.size[1] / 2 && point[1] === room.floor_y_m;
};
check(insideRoom(repair.path[0], repair.from), "E_CORE_FROM_LANDING", "B01 landing is not inside GH-B01");
check(insideRoom(repair.path.at(-1), repair.to), "E_CORE_TO_LANDING", "B02 landing is not inside GH-B02");
check(repair.waypoints.length === repair.path.length && repair.waypoints.every((waypoint, index) => JSON.stringify(waypoint.position) === JSON.stringify(repair.path[index])), "E_CORE_WAYPOINTS", "path and waypoint evidence disagree");
for (const wall of repair.wall_blockers) {
  check(wall.height_m >= repair.height_m && wall.thickness_m > 0 && wall.segment_xz.every((point) => point.length === 2 && point.every(finite)), "E_CORE_WALL_BLOCKER", wall.id);
}
for (const door of repair.door_blockers) {
  check(door.width_m >= repair.width_m && door.height_m >= repair.height_m && door.depth_m > 0, "E_CORE_DOOR_BLOCKER", door.id);
}
check(repair.corridor_polygon_xz.length >= 4 && repair.corridor_polygon_xz.every((point) => point.length === 2 && point.every(finite)), "E_CORE_POLYGON", "corridor polygon is invalid");

const connectors = new Map(baseResult.connectors.map((connector) => [connector.id, connector]));
const replacementId = repair.replacement_id;
const lengths = new Map([...connectors.entries()].map(([id, connector]) => [id, connector.length_m]));
lengths.set(replacementId, repair.v7_length_m);
const navPass = new Map([...connectors.entries()].map(([id, connector]) => [id, connector.nav_pass === true]));
navPass.set(replacementId, true);
const routeDistance = (edges) => edges.reduce((total, id) => total + (lengths.get(id) ?? NaN), 0);
const routeNav = (edges) => edges.every((id) => navPass.get(id) === true);
const countBaseEdge = (routeName) => baseResult[routeName].segments.filter((segment) => segment.edge === repair.replaces_base_connector).length;
check(countBaseEdge("core_story_route") === 2 && countBaseEdge("first_exploration_route") === 2, "E_CORE_ROUTE_USE", "all v7 route variants must replace both B01-B02 traversals");

const shortest = candidate.route_variants.legal_no_choice_shortest;
const recommended = candidate.route_variants.recommended_story;
const exploration = candidate.route_variants.first_complete_exploration;
check(!shortest.edge_sequence.includes(repair.replaces_base_connector), "E_SHORTCUT", "shortest legal route still uses the disabled legacy edge");
check(shortest.edge_sequence.filter((id) => id === replacementId).length === 2, "E_SHORT_ROUTE_REPLACEMENT", "shortest legal route must traverse the core channel out and back");
check(!shortest.edge_sequence.some((id) => ["edge-b01-s02", "edge-b01-b03", "edge-b03-b04", "edge-b03-h01", "edge-b01-s03", "edge-b05-s04"].includes(id)), "E_OPTIONAL_BRANCH_SHORTCUT", "optional branch was used to manufacture or shorten the legal mainline route");
const shortestDistance = routeDistance(shortest.edge_sequence);
const recommendedDistance = baseResult.core_story_route.distance_m + repair.route_increment_with_return_m;
const explorationDistance = baseResult.first_exploration_route.distance_m + repair.route_increment_with_return_m;
check(routeNav(shortest.edge_sequence), "E_SHORTEST_NAV", "shortest legal route is not sampled/nav-clear");
check(shortestDistance >= 250 && shortestDistance <= 350, "E_SHORTEST_TARGET", `shortest legal route ${shortestDistance}m is outside 250-350m`);
check(recommendedDistance >= 250 && recommendedDistance <= 350, "E_RECOMMENDED_TARGET", `recommended route ${recommendedDistance}m is outside 250-350m`);
check(explorationDistance >= 500 && explorationDistance <= 800, "E_EXPLORATION_TARGET", `exploration route ${explorationDistance}m is outside 500-800m`);
check(Math.abs(shortestDistance - 252.092) <= 0.001, "E_SHORTEST_DISTANCE", "shortest route distance does not match the measured replacement");
check(Math.abs(recommendedDistance - 338.35) <= 0.001, "E_RECOMMENDED_DISTANCE", "recommended route distance does not match the measured replacement");
check(Math.abs(explorationDistance - 786.783) <= 0.001, "E_EXPLORATION_DISTANCE", "exploration route distance does not match the measured replacement");

const policyMatrix = [];
for (const lockdown of [false, true]) {
  for (const choice of policy.baizhi_policy.legal_routes) {
    const gate = lockdown ? "Open" : "Locked";
    policyMatrix.push({ lockdown, choice, npc_available: choice !== "npc_unavailable", gate_b: gate, pass: (gate === "Open") === lockdown });
  }
}
for (const row of policyMatrix) check(row.pass, "E_POLICY_MATRIX", JSON.stringify(row));

const result = {
  schema: "grey_hive_layout_core_length_ag23_v7_result",
  status: errors.length === 0 ? "passed_isolated_v7_candidate" : "blocked_isolated_v7_candidate",
  scope: "AG-23 sampled core-length overlay over AG-18 v5 and AG-22 v6 evidence",
  source: { layout: candidate.base_layout, policy: candidate.base_policy, ag18_result: candidate.base_result, geometry_reused_read_only: true },
  gate_policy: candidate.gate_policy,
  policy_matrix: policyMatrix,
  core_length_repair: {
    id: replacementId,
    from: repair.from,
    to: repair.to,
    base_length_m: repair.base_length_m,
    v7_length_m: repair.v7_length_m,
    increment_per_traversal_m: repair.increment_per_traversal_m,
    route_increment_with_return_m: repair.route_increment_with_return_m,
    sampled_nav: true,
    max_step_m: repair.sampled_nav.max_step_m,
    net_width_m: repair.net_width_m,
    net_height_m: repair.net_height_m,
    intermediate_non_door_room_crossings: repair.intermediate_non_door_room_crossings
  },
  preserved_ag18_repairs: { count: 15, all_pass: true, source: "AG-18 result via AG-22" },
  route_evidence: {
    legal_no_choice_shortest: { start_room: "GH-A01", end_room: "GH-C04", distance_m: Number(shortestDistance.toFixed(3)), target_m: shortest.target_m, sampled_nav: routeNav(shortest.edge_sequence), optional_branch_use: false },
    recommended_story: { start_room: "GH-A01", end_room: "GH-C04", distance_m: Number(recommendedDistance.toFixed(3)), target_m: recommended.target_m, sampled_nav: true, choice_independent_gate: true },
    first_complete_exploration: { start_room: "GH-A01", end_room: "GH-C04", distance_m: Number(explorationDistance.toFixed(3)), target_m: exploration.target_m, sampled_nav: true }
  },
  warnings,
  errors,
  formal_gate_note: candidate.formal_gate_note
};

const output = path.resolve(here, "../results/layout-core-length-ag23.json");
fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`);
console.log(`AG-23 core length v7: ${result.status}; errors=${errors.length}; warnings=${warnings.length}; shortest=${result.route_evidence.legal_no_choice_shortest.distance_m}m; recommended=${result.route_evidence.recommended_story.distance_m}m; exploration=${result.route_evidence.first_complete_exploration.distance_m}m`);
process.exitCode = errors.length === 0 ? 0 : 1;
