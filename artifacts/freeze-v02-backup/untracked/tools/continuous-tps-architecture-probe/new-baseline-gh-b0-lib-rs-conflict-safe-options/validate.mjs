import fs from "node:fs";
import crypto from "node:crypto";
import { execFileSync } from "node:child_process";

const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const optionsPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-gh-b0-lib-rs-conflict-safe-options\\options.json`;
const nb79Path = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-lib-rs-change-provenance-review\\review.json`;
const sourcePath = `${root}server-rs\\src\\lib.rs`;
const options = JSON.parse(fs.readFileSync(optionsPath, "utf8"));
const nb79 = JSON.parse(fs.readFileSync(nb79Path, "utf8"));
const sourceBytes = fs.readFileSync(sourcePath);
const sha256 = value => crypto.createHash("sha256").update(value).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const status = git("status", "--short", "--", "server-rs/src/lib.rs");
const currentHash = sha256(sourceBytes);
const branch = git("branch", "--show-current");
const optionsIds = options.options.map(option => option.id);
const expectedIds = ["wait_for_owner_and_reverify", "fresh_clean_worktree_after_gate", "separate_commit_from_verified_base", "nested_module_under_existing_export", "dedicated_crate_internal_module_boundary", "defer_GH_B0_until_core_ownership_settles"];
const forbidden = ["git stash", "git checkout", "git cherry-pick", "force checkout", "automatic patch", "强制checkout", "自动stash"];
const serialized = fs.readFileSync(`${root}docs\\GH_B0_LIB_RS_CONFLICT_SAFE_EXECUTION_OPTIONS.md`, "utf8").toLowerCase();
const checks = {
  status: options.status === "gh_b0_lib_rs_conflict_safe_options_ready",
  source_status: status === "M server-rs/src/lib.rs",
  source_hash_unchanged: currentHash === options.conflictIdentity.currentSha256 && currentHash === nb79.currentFingerprint.fileSha256,
  source_anchor: fs.readFileSync(sourcePath, "utf8").includes("pub mod world_v3;"),
  real_world_v3_anchor: fs.readFileSync(`${root}server-rs\\src\\world_v3.rs`, "utf8").includes("pub struct WorldStateV3"),
  real_cargo_anchor: fs.readFileSync(`${root}server-rs\\Cargo.toml`, "utf8").includes("[package]"),
  exact_scope: options.ghB0Scope.exactFiles.length === 6 && options.ghB0Scope.plannedNewFiles.length === 5,
  six_options: expectedIds.every(id => optionsIds.includes(id)) && optionsIds.length === 6,
  options_complete: options.options.every(option => ["preconditions", "touchesLibRs", "dependsOnUncommitted", "sourceOfTruth", "mergeRisk", "rollback", "evidence", "gateCompatibility", "recommendationRank", "viability"].every(field => Object.hasOwn(option, field))),
  viability_evidence: options.viabilityEvidence.realAnchors.length >= 3 && options.viabilityEvidence.missingAnchors.length >= 2,
  unique_recommendation: options.recommendedOption.unique === true && options.recommendedOption.id === "wait_for_owner_and_reverify" && options.options.filter(option => option.recommendationRank === 1).length === 1,
  contracts: options.ownershipResolutionContract.recorded === false && options.freshBaselineContract.created === false,
  gate_closed: options.ghB0Scope.architectureGateOpened === false && options.ghB0Scope.ghB0Approved === false,
  no_dangerous_instruction: forbidden.every(phrase => !serialized.includes(phrase)),
  no_branch_creation: branch === "master",
  no_runtime_flags: Object.entries(options.currentDecision).filter(([key]) => /Executed|Read|Created|Approved|Opened|Modified|Resolved|Authorized|Recorded|branch|worktree|stash|patch|implementation|sourceModified|network/.test(key)).every(([, value]) => value === false || value === 0),
  required_flags: options.currentDecision.libRsConflictResolved === false && options.currentDecision.safeExecutionAuthorized === false && options.currentDecision.blockersClosed === 0 && options.currentDecision.network === false
};
const failures = Object.entries(checks).filter(([, pass]) => !pass).map(([name]) => name);
const result = {
  schemaVersion: 1,
  reviewId: "NB-84",
  validator: "tools/continuous-tps-architecture-probe/new-baseline-gh-b0-lib-rs-conflict-safe-options/validate.mjs",
  status: failures.length === 0 ? "pass_gate_blocked" : "fail",
  checks,
  failureCount: failures.length,
  failures,
  decision: failures.length === 0 ? "safe_options_ready_but_not_authorized" : "invalid_options_review",
  sourceSnapshot: { path: "server-rs/src/lib.rs", gitStatus: status, branch, sha256: currentHash },
  sideEffectPolicy: { libRsModifiedByTask: false, branchCreated: false, worktreeCreated: false, stashCreated: false, patchCreated: false, sourceModified: false, build: false, tests: false, runtime: false, media: false, network: false }
};
const resultPath = `${root}tools\\continuous-tps-architecture-probe\\results\\new-baseline-gh-b0-lib-rs-conflict-safe-options-nb84.json`;
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
