import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname);
const resultPath = path.join(root, "../results/save-resume-resolution-tp03d1.json");
const report = JSON.parse(fs.readFileSync(resultPath, "utf8"));
const fail = (message) => { throw new Error(message); };
const assert = (condition, message) => { if (!condition) fail(message); };
const byName = (name) => {
  const found = report.scenarios.find((scenario) => scenario.name === name);
  assert(found, `missing scenario: ${name}`);
  return found;
};

assert(report.schema === "continuous_save_resume_resolution_tp03d1_v1", "schema mismatch");
assert(report.status === "passed_formal_api_plus_isolated_candidate", "status mismatch");
assert(report.machine_pass === true && report.candidate_only === true, "candidate pass mismatch");
assert(report.formal_api_reuse.shadowSchema === false, "shadow schema was used");
assert(report.fixture_ids.length === 6, "six fixture coverage missing");
assert(report.canonical_checkpoints.length === 6, "canonical checkpoint count mismatch");
assert(report.scenarios.length === 25, `scenario count mismatch: ${report.scenarios.length}`);
assert(report.scenarios.every((scenario) => scenario.pass === true), "one or more scenarios failed");

for (const id of report.fixture_ids) {
  assert(byName(`fixture_${id}`), `fixture scenario missing: ${id}`);
}
for (const choice of ["taken", "left", "unresolved"]) {
  const scenario = byName(`fixture_post_baizhi_${choice}`);
  assert(scenario.actual.choice === choice, `choice state mismatch: ${choice}`);
  assert(scenario.actual.gateB === "open", `Gate B mismatch for ${choice}`);
  assert(scenario.actual.resolver.transform.candidateOnly === true, `candidate transform missing: ${choice}`);
}

const initial = byName("fixture_grey_hive_initial");
assert(initial.actual.gateB === "locked", "lockdown=false was not preserved");
const result = byName("fixture_completed_grey_hive");
assert(result.actual.mapNull === true && result.actual.resolver.result === "terminal_not_applicable", "terminal noSpawn policy mismatch");
assert(result.actual.resolver.navCalls === 0, "terminal result called candidate nav");

for (const name of ["wrong_world", "wrong_layout", "wrong_world_epoch", "unknown_checkpoint", "missing_transform_spawn", "spawn_clearance_failure", "unreachable", "projection_failed", "nan_transform_rejected", "infinity_transform_rejected"]) {
  assert(byName(name).pass, `error scenario failed: ${name}`);
}
assert(byName("wrong_world_epoch").actual.error === "E_WORLD_EPOCH", "epoch guard missing");
assert(byName("source_backup_hash_mismatch").actual.targetCommitted === false, "hash mismatch committed target");
assert(byName("persistence_failure_E_PENDING_WRITE").actual.targetCommitted === false, "pending-write failure committed target");
assert(byName("persistence_failure_E_COMMIT_INTERRUPTED").actual.targetCommitted === false, "pre-rename failure committed target");
assert(byName("reload_idempotent").actual.first === "committed" && byName("reload_idempotent").actual.second === "already_present", "reload idempotence mismatch");
assert(byName("reload_idempotent").actual.targetHash1 === byName("reload_idempotent").actual.targetHash2, "reload target hash changed");
assert(byName("legacy_px_py_not_scaled").actual.originalCheckpoint === byName("legacy_px_py_not_scaled").actual.scaledCheckpoint, "legacy px/py changed checkpoint");
assert(JSON.stringify(byName("legacy_px_py_not_scaled").actual.originalTransform) === JSON.stringify(byName("legacy_px_py_not_scaled").actual.scaledTransform), "legacy px/py scaled candidate transform");

const scene = byName("synthetic_scene_chain_and_v7_negative");
assert(scene.actual.positiveChain === true && scene.actual.tamperedChain === false, "scene chain tamper check mismatch");
assert(scene.actual.v7Negative.candidateOnly === true && scene.actual.v7Negative.formalSceneReady === false && scene.actual.v7Negative.expectedFailure === true, "v7 negative was promoted");
assert(report.synthetic_scene_chain.formalSceneReady === false && report.synthetic_scene_chain.syntheticOnly === true, "formal scene status mismatch");

console.log(`TP-03D1 validation passed: ${report.scenarios.length} scenarios; ${report.fixture_ids.length} fixtures`);
