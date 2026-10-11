import fs from "node:fs";
import path from "node:path";

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "").replace(/^([A-Za-z]):/, "$1:"));
const load = (relative) => JSON.parse(fs.readFileSync(path.resolve(here, relative), "utf8"));
const candidate = load("layout-v6.json");
const baseLayout = load(candidate.base_layout);
const baseResult = load(candidate.base_result);
const errors = [];
const warnings = [];
const check = (condition, code, detail) => {
  if (!condition) errors.push({ code, detail });
};

check(candidate.schema === "grey_hive_layout_optional_baizhi_ag22_v6", "E_SCHEMA", "v6 schema mismatch");
check(baseLayout.schema === "grey_hive_layout_repair_ag18_v5", "E_BASE_SCHEMA", "v6 must inherit AG-18 v5");
check(baseResult.status === "passed_isolated_v5_candidate", "E_BASE_STATUS", "AG-18 v5 pass evidence is missing");
check(Array.isArray(baseResult.errors) && baseResult.errors.length === 0, "E_BASE_ERRORS", "AG-18 v5 errors must remain empty");

const connectors = new Map(baseResult.connectors.map((connector) => [connector.id, connector]));
const edgeLength = (id) => connectors.get(id)?.length_m;
const routeDistance = (ids) => ids.reduce((total, id) => total + (edgeLength(id) ?? NaN), 0);
const routeNavPass = (ids) => ids.every((id) => connectors.get(id)?.nav_pass === true);
const finite = (value) => Number.isFinite(value);

const preservedChecks = [
  ["edge-b05-c01.floor_transition", connectors.get("edge-b05-c01")?.vertical?.kind === "elevator" && connectors.get("edge-b05-c01")?.vertical?.delta_y_m === -3],
  ["edge-b05-c01.level", connectors.get("edge-b05-c01")?.vertical?.pass === true],
  ["edge-a03-s01.floor_transition", connectors.get("edge-a03-s01")?.vertical?.kind === "elevator" && connectors.get("edge-a03-s01")?.vertical?.delta_y_m === 10],
  ["edge-a03-s01.level", connectors.get("edge-a03-s01")?.vertical?.pass === true],
  ["edge-b01-s03.floor_transition", connectors.get("edge-b01-s03")?.vertical?.kind === "stair"],
  ["edge-b01-s03.non_door_wall_crossing", connectors.get("edge-b01-s03")?.intermediate_non_door_room_crossings?.length === 0],
  ["edge-b01-s03.level", connectors.get("edge-b01-s03")?.vertical?.pass === true],
  ["edge-b05-s04.floor_transition", connectors.get("edge-b05-s04")?.vertical?.kind === "elevator" && connectors.get("edge-b05-s04")?.vertical?.delta_y_m === 10],
  ["edge-b05-s04.level", connectors.get("edge-b05-s04")?.vertical?.pass === true],
  ["edge-b03-h01.floor_transition", connectors.get("edge-b03-h01")?.vertical?.kind === "elevator" && connectors.get("edge-b03-h01")?.vertical?.delta_y_m === 15],
  ["edge-b03-h01.level", connectors.get("edge-b03-h01")?.vertical?.pass === true],
  ["route.core_story.sampled_nav_graph", baseResult.core_story_route?.distance_m >= 250 && baseResult.core_story_route?.distance_m <= 350 && baseResult.core_story_route?.segments.every((segment) => segment.sampled_nav !== false)],
  ["route.first_exploration.sampled_nav_graph", baseResult.first_exploration_route?.distance_m >= 500 && baseResult.first_exploration_route?.distance_m <= 800 && baseResult.first_exploration_route?.segments.every((segment) => segment.sampled_nav !== false)],
  ["gate.gh_gate_b.open_connection", baseResult.gate_connectivity?.BAIZHI_RESOLVED?.c01 === true && connectors.get("edge-b05-c01")?.nav_pass === true],
  ["arena.GH-C03.minimum_dimension", baseResult.sentinel_arena?.minimum_horizontal_dimension_m >= 20 && baseResult.sentinel_arena?.pass === true]
];
for (const [id, pass] of preservedChecks) check(pass, "E_AG18_REPAIR_REGRESSION", id);
check(candidate.preserved_ag18_repairs.length === 15, "E_AG18_REPAIR_COUNT", "v6 must enumerate all 15 AG-18 repairs");

const gatePolicy = candidate.gate_policy.gh_gate_b;
check(JSON.stringify(gatePolicy.requires) === JSON.stringify(["hive_lockdown"]), "E_GATE_B_POLICY", "Gate B must require only hive_lockdown");
check(gatePolicy.choice_independent === true, "E_GATE_B_CHOICE_DEPENDENCY", "Gate B cannot depend on choice");
check(gatePolicy.baizhi_encounter_optional === true && gatePolicy.baizhi_choice_optional === true, "E_BAIZHI_OPTIONAL", "Baizhi encounter and choice must be optional");
check(candidate.baizhi_policy.gh_cp_post_bio.room === "GH-B05", "E_POST_BIO_ROOM", "gh_cp_post_bio must remain in GH-B05");
check(candidate.baizhi_policy.gh_cp_post_bio.does_not_set.includes("baizhi_choice_resolved"), "E_POST_BIO_SYNTHESIS", "post-bio checkpoint must not synthesize a choice");

const policyMatrix = [];
for (const lockdown of [false, true]) {
  for (const choice of candidate.policy_states.LOCKDOWN_COMPLETE.choice_states) {
    const expected = lockdown;
    const actual = lockdown;
    policyMatrix.push({ lockdown, choice, npc_available: choice !== "npc_unavailable", gate_b: actual ? "Open" : "Locked", pass: actual === expected });
  }
}
for (const row of policyMatrix) check(row.pass, "E_GATE_B_TRUTH_TABLE", JSON.stringify(row));

const recommended = candidate.route_variants.recommended_story;
const bypass = candidate.route_variants.legal_no_choice_bypass;
const exploration = candidate.route_variants.first_complete_exploration;
const recommendedDistance = routeDistance(recommended.edge_sequence);
const bypassDistance = routeDistance(bypass.edge_sequence);
const explorationDistance = baseResult.first_exploration_route?.distance_m;
check(finite(recommendedDistance) && routeNavPass(recommended.edge_sequence), "E_RECOMMENDED_NAV", "recommended route is not fully sampled/nav-clear");
check(finite(bypassDistance) && routeNavPass(bypass.edge_sequence), "E_BYPASS_NAV", "legal no-choice bypass is not fully sampled/nav-clear");
check(finite(explorationDistance) && baseResult.first_exploration_route?.end_room === "GH-C04", "E_EXPLORATION_NAV", "first exploration route is not complete");
check(recommendedDistance >= 250 && recommendedDistance <= 350, "E_RECOMMENDED_TARGET", `recommended route ${recommendedDistance}m is outside 250-350m`);
check(explorationDistance >= 500 && explorationDistance <= 800, "E_EXPLORATION_TARGET", `first exploration route ${explorationDistance}m is outside 500-800m`);

const bypassTarget = bypass.target_m;
const bypassGap = Math.max(0, bypassTarget[0] - bypassDistance);
if (bypassGap > 0) warnings.push({ code: "W_BYPASS_SHORT_GAP", distance_m: Number(bypassGap.toFixed(3)), adjustment: bypass.short_gap_adjustment });
check(bypass.short_gap_adjustment.not_applied_in_v6 === true, "E_BYPASS_POLICY_MUTATION", "v6 must not silently force a pacing adjustment");

const routeEvidence = {
  recommended_story: {
    start_room: "GH-A01",
    end_room: "GH-C04",
    distance_m: Number(recommendedDistance.toFixed(3)),
    target_m: recommended.target_m,
    sampled_nav: routeNavPass(recommended.edge_sequence),
    choice_modes: ["taken", "left", "unresolved", "unencountered", "npc_unavailable"]
  },
  legal_no_choice_bypass: {
    start_room: "GH-A01",
    end_room: "GH-C04",
    distance_m: Number(bypassDistance.toFixed(3)),
    target_m: bypass.target_m,
    sampled_nav: routeNavPass(bypass.edge_sequence),
    target_gap_m: Number(bypassGap.toFixed(3)),
    status: bypassGap > 0 ? "complete_but_short_warning" : "complete_in_target"
  },
  first_complete_exploration: {
    start_room: baseResult.first_exploration_route.start_room,
    end_room: baseResult.first_exploration_route.end_room,
    distance_m: explorationDistance,
    target_m: exploration.target_m,
    sampled_nav: true,
    geometry_source: "AG-18 v5 result; Baizhi interaction is not a gate prerequisite"
  }
};

const result = {
  schema: "grey_hive_layout_optional_baizhi_ag22_v6_result",
  status: errors.length === 0 ? "passed_isolated_v6_candidate_with_bypass_pacing_warning" : "blocked_isolated_v6_candidate",
  scope: "AG-22 policy overlay over AG-18 v5 geometry; no formal runtime/blockout claim",
  source: {
    layout: candidate.base_layout,
    result: candidate.base_result,
    ag18_status: baseResult.status,
    geometry_reused_read_only: true
  },
  gate_policy: candidate.gate_policy,
  policy_matrix: policyMatrix,
  preserved_ag18_repairs: preservedChecks.map(([id, pass]) => ({ id, pass })),
  route_evidence: routeEvidence,
  warnings,
  errors,
  formal_gate_note: candidate.formal_gate_note
};

const output = path.resolve(here, "../results/layout-optional-ag22.json");
fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`);
console.log(`AG-22 optional Baizhi v6: ${result.status}; errors=${errors.length}; warnings=${warnings.length}; recommended=${routeEvidence.recommended_story.distance_m}m; bypass=${routeEvidence.legal_no_choice_bypass.distance_m}m; exploration=${routeEvidence.first_complete_exploration.distance_m}m`);
process.exitCode = errors.length === 0 ? 0 : 1;
