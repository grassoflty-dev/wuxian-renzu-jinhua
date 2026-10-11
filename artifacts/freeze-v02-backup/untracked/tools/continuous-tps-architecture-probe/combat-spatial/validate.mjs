import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname);
const resultPath = path.join(root, "../results/combat-spatial-tp12a1.json");
const report = JSON.parse(fs.readFileSync(resultPath, "utf8"));
const expectedNames = [
  "forward_hit",
  "distance_boundary_in_and_out",
  "cone_angle_boundary_in_and_out",
  "behind_rejected",
  "vertical_tolerance_boundary",
  "wall_blocks_los",
  "closed_door_blocks_open_door_hits",
  "stable_multi_target_sort",
  "duplicate_attack_event_idempotent",
  "old_epoch_and_tick_rejected",
  "nan_infinity_rejected",
  "thirty_sixty_hz_same_hit_set",
];

const fail = (message) => {
  throw new Error(message);
};
const assert = (condition, message) => {
  if (!condition) fail(message);
};
const scenario = (name) => {
  const found = report.scenarios.find((item) => item.name === name);
  assert(found, `missing scenario: ${name}`);
  return found;
};
const auditNumbersAreFinite = (scenarioItem) => {
  for (const audit of scenarioItem.actual.audit ?? scenarioItem.actual.audits ?? []) {
    for (const field of ["distance_m", "horizontal_angle_rad", "horizontal_angle_deg", "vertical_delta_m"]) {
      assert(Number.isFinite(audit[field]), `${scenarioItem.name}: non-finite ${field}`);
    }
  }
};

assert(report.schema === "continuous_combat_spatial_authority_tp12a1_v1", "schema mismatch");
assert(report.status === "passed_isolated_candidate", "candidate probe did not pass");
assert(report.machine_pass === true && report.candidate_only === true, "candidate status mismatch");
assert(report.scenarios.length === expectedNames.length, "scenario count mismatch");
assert(JSON.stringify(report.scenarios.map((item) => item.name)) === JSON.stringify(expectedNames), "scenario order mismatch");
assert(report.coordinate_contract.units === "metres", "metre contract missing");
assert(report.coordinate_contract.handedness === "right_handed", "handedness contract missing");
assert(report.coordinate_contract.axes === "x=east,y=up,z=depth", "axis contract missing");
assert(report.parameters.los.includes("Rapier 0.22 QueryPipeline::cast_ray"), "Rapier LOS evidence missing");
assert(report.parameters.sorting === "distance ascending, targetId ascending tie-break", "sort contract missing");
assert(report.parameters.duplicatePolicy.includes("consumed once"), "duplicate policy missing");

for (const item of report.scenarios) {
  assert(item.pass === true, `scenario failed: ${item.name}`);
  auditNumbersAreFinite(item);
}

const boundary = scenario("distance_boundary_in_and_out");
assert(boundary.actual.atRangeHit === true && boundary.actual.outsideRangeHit === false, "range boundary mismatch");
const angle = scenario("cone_angle_boundary_in_and_out");
assert(angle.actual.atAngleHit === true && angle.actual.outsideAngleHit === false, "cone boundary mismatch");
const vertical = scenario("vertical_tolerance_boundary");
assert(vertical.actual.atVerticalHit === true && vertical.actual.outsideVerticalHit === false, "vertical boundary mismatch");
const wall = scenario("wall_blocks_los");
assert(wall.actual.targetIds.length === 0 && wall.actual.audit[0].los_blocker_id === "wall-static", "wall LOS mismatch");
const door = scenario("closed_door_blocks_open_door_hits");
assert(door.actual.closed.length === 0 && JSON.stringify(door.actual.open) === JSON.stringify(["target-door"]), "door LOS mismatch");
const sorted = scenario("stable_multi_target_sort");
assert(JSON.stringify(sorted.actual.targetIds) === JSON.stringify(["target-c", "target-a", "target-b"]), "stable sorting mismatch");
const duplicate = scenario("duplicate_attack_event_idempotent");
assert(duplicate.actual.duplicateError === "duplicate_attack_event:13", "duplicate event policy mismatch");
const epochTick = scenario("old_epoch_and_tick_rejected");
assert(epochTick.actual.epoch === "old_epoch:expected=7:actual=6", "epoch rejection mismatch");
assert(epochTick.actual.tick === "old_tick:minimum=100:actual=99", "tick rejection mismatch");
const invalid = scenario("nan_infinity_rejected");
assert(invalid.actual.nan === "non_finite:rangeM" && invalid.actual.infinity === "non_finite:halfAngleRad", "finite input rejection mismatch");
const rate = scenario("thirty_sixty_hz_same_hit_set");
assert(rate.actual.same === true && JSON.stringify(rate.actual.at30Hz) === JSON.stringify(rate.actual.at60Hz), "30/60 hit-set mismatch");

console.log(`TP-12A1 validation passed: ${report.scenarios.length} scenarios`);
