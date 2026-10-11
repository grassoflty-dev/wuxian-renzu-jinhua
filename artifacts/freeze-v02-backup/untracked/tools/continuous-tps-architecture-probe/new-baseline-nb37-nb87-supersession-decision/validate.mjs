import fs from "node:fs";
import { execFileSync } from "node:child_process";

const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const decisionPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-nb37-nb87-supersession-decision\\decision.json`;
const nb87Path = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-first-formal-3d-return-station-plan-replacement\\plan.json`;
const decision = JSON.parse(fs.readFileSync(decisionPath, "utf8"));
const nb87 = JSON.parse(fs.readFileSync(nb87Path, "utf8"));
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const requiredScope = ["source_requirements_pending", "strict_post_gate_dependencies", "station_minimum_contract", "transition_transaction", "save_ipc_babylon_boundary", "failure_crash_reload", "five_batches", "acceptance_ladder", "media_release_boundary", "no_fallback"];
const allFalse = ["returnStationApproved", "returnStationImplemented", "returnStationRuntimeLoaded", "architectureGateOpened", "implementationStarted", "sourceModified", "buildExecuted", "testsExecuted", "runtimeExecuted", "mediaBytesRead", "mediaCreated", "mediaModelCalled", "network"].every(key => decision.currentDecision[key] === false);
const checks = {
  status: decision.status === "nb37_nb87_supersession_decision_packet_ready",
  old_identity_only: decision.oldPackageIdentity.id === "NB-37" && decision.oldPackageIdentity.contentRead === false && decision.oldPackageIdentity.hashComputed === false && decision.oldPackageIdentity.quarantined === true,
  replacement_identity: decision.replacementIdentity.id === "NB-87" && decision.replacementIdentity.newPaths === true,
  scope_complete: decision.coverageMatrix.scopeCoverageComplete === true && decision.coverageMatrix.covered === 10 && decision.coverageMatrix.partial === 0 && decision.coverageMatrix.missing === 0 && requiredScope.every(id => decision.requiredScopeChecklist.some(item => item.id === id && item.coverage === "covered")),
  post_gate_preserved: Object.values(decision.entryDependencyChecks).slice(0, 8).every(value => value === false) && decision.entryDependencyChecks.entryDependenciesPreserved === true && decision.entryDependencyChecks.blockersOpen === 5,
  batch_contract: decision.batchChecks.batchCount === 5 && decision.batchChecks.allNotApproved === true && decision.batchChecks.eachAtMostEightUnits === true && decision.batchChecks.firstBatchDefaultOff === true && decision.batchChecks.firstBatchDataOnly === true,
  fresh_checks: decision.freshReplacementChecks.nb37Unconsumed === true && decision.freshReplacementChecks.oldPreserved === true && decision.freshReplacementChecks.noStatusUpgrade === true && decision.freshReplacementChecks.noRuntimeClaims === true && decision.freshReplacementChecks.nb89Read === false && decision.freshReplacementChecks.nb90Read === false,
  recommendation_traceable: decision.recommendation.unique === true && decision.recommendation.accepted === false && decision.recommendation.id === "accept_fresh_replacement_for_supersession_review" && decision.currentDecision.nb37SupersessionAccepted === false && decision.currentDecision.acceptanceDecisionRecorded === false,
  acceptance_template_pending: decision.acceptanceRecordTemplate.decision === null && decision.acceptanceRecordTemplate.decider === null && decision.acceptanceRecordTemplate.entryDependencies === null,
  post_state: decision.postAcceptanceState.nb37RemainsQuarantined === true && decision.postAcceptanceState.nb87OnlyPlanScopeAccepted === true && decision.postAcceptanceState.entryDependenciesSatisfied === false && decision.postAcceptanceState.blockersOpen === 5 && decision.postAcceptanceState.returnStationApproved === false,
  nb87_real_alignment: nb87.planId === "NB-87" && nb87.currentDecision.nb37ContentRead === false && nb87.currentDecision.entryDependenciesSatisfied === false && nb87.batches.length === 5,
  source_unchanged: git("status", "--short", "--", "server-rs/src/lib.rs") === "M server-rs/src/lib.rs",
  no_execution: allFalse,
  no_historical_modification: decision.currentDecision.historicalFilesModified === false
};
const failures = Object.entries(checks).filter(([, value]) => !value).map(([key]) => key);
const result = {
  schemaVersion: 1,
  reviewId: "NB-91",
  validator: "tools/continuous-tps-architecture-probe/new-baseline-nb37-nb87-supersession-decision/validate.mjs",
  status: failures.length === 0 ? "pass_gate_blocked" : "fail",
  checks,
  failureCount: failures.length,
  failures,
  decision: failures.length === 0 ? "replacement_decision_ready_not_accepted" : "invalid_supersession_packet",
  sideEffectPolicy: { nb37ContentRead: false, historicalFilesModified: false, sourceModified: false, build: false, tests: false, runtime: false, media: false, network: false }
};
const resultPath = `${root}tools\\continuous-tps-architecture-probe\\results\\new-baseline-nb37-nb87-supersession-decision-nb91.json`;
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
