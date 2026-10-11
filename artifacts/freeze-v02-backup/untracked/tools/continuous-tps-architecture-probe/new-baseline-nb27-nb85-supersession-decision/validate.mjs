import fs from "node:fs";
import { execFileSync } from "node:child_process";

const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const decisionPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-nb27-nb85-supersession-decision\\decision.json`;
const catalogPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-capability-catalog-replacement\\catalog.json`;
const decision = JSON.parse(fs.readFileSync(decisionPath, "utf8"));
const catalog = JSON.parse(fs.readFileSync(catalogPath, "utf8"));
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const checklistIds = ["sources_classification", "catalog_entries", "three_characters", "three_worlds", "definition_state_presentation", "grant_transaction", "save_ipc_error_schema", "progression", "invariants", "placeholder_final", "pending_decisions"];
const flags = decision.currentDecision;
const checks = {
  status: decision.status === "nb27_nb85_supersession_decision_packet_ready",
  old_identity_only: decision.oldPackageIdentity.id === "NB-27" && decision.oldPackageIdentity.contentRead === false && decision.oldPackageIdentity.hashComputed === false && decision.oldPackageIdentity.quarantined === true,
  replacement_identity: decision.replacementIdentity.id === "NB-85" && decision.replacementIdentity.newPaths === true,
  scope_complete: decision.coverageMatrix.scopeCoverageComplete === true && decision.coverageMatrix.covered === 11 && decision.coverageMatrix.partial === 0 && decision.coverageMatrix.missing === 0 && checklistIds.every(id => decision.requiredScopeChecklist.some(item => item.id === id && item.coverage === "covered")),
  classification_boundary: decision.classificationChecks.entryCount === 16 && decision.classificationChecks.sourceConfirmed === 4 && decision.classificationChecks.contractRequiredShell === 5 && decision.classificationChecks.candidateProposal === 5 && decision.classificationChecks.pendingProductDecision === 2 && decision.classificationChecks.boundaryValid === true && decision.classificationChecks.shellsNotUpgraded === true && decision.classificationChecks.candidatesNotUpgraded === true && decision.classificationChecks.numericBalanceInvented === false,
  character_world_checks: decision.characterChecks.Cen_Yao === 7 && decision.characterChecks.Bai_Zhi === 1 && decision.characterChecks.Sentinel === 1 && decision.worldChecks.grey_hive === 3 && decision.worldChecks.mist_harbor === 1 && decision.worldChecks.clockworks === 1 && decision.worldChecks.gateB === "hive_lockdown only" && decision.worldChecks.npcIndependent === true,
  fresh_checks: decision.freshChecks.nb27Unconsumed === true && decision.freshChecks.oldPreserved === true && decision.freshChecks.noStatusUpgrade === true && decision.freshChecks.noRuntimeClaims === true && decision.freshChecks.nb92Read === false,
  defer_recommendation: decision.recommendation.unique === true && decision.recommendation.accepted === false && decision.recommendation.id === "recommend_defer_supersession_until_capability_scope_decision" && flags.recommendation === decision.recommendation.id,
  product_answers_zero: flags.productAnswersRecorded === 0 && decision.acceptanceRecordTemplate.productAnswersRecorded === 0,
  future_accept_conditions: decision.deferConditions.length >= 4 && decision.deferConditions.some(item => item.includes("NB-90")) && decision.deferConditions.some(item => item.includes("main-session explicit acceptance")),
  post_state: decision.postState.nb27RemainsQuarantined === true && decision.postState.nb85OnlyScopeEvidenceIfAccepted === true && decision.postState.catalogApproved === false && decision.postState.runtimeIntegrated === false && decision.postState.saveIntegrated === false,
  nb85_real_alignment: catalog.catalogVersion === "nb85-capability-catalog-replacement-v1" && catalog.entries.length === 16 && catalog.freshReplacementChecks.nb27ContentRead === false && catalog.recommendation.decision === "defer",
  source_unchanged: git("status", "--short", "--", "server-rs/src/lib.rs") === "M server-rs/src/lib.rs",
  no_execution: ["capabilityCatalogApproved", "runtimeIntegrated", "saveIntegrated", "architectureGateOpened", "implementationStarted", "sourceModified", "buildExecuted", "testsExecuted", "runtimeExecuted", "mediaBytesRead", "mediaCreated", "mediaModelCalled", "network"].every(key => flags[key] === false),
  no_old_or_active_reads: flags.nb27ContentRead === false && decision.freshChecks.nb92Read === false
};
const failures = Object.entries(checks).filter(([, value]) => !value).map(([key]) => key);
const result = {
  schemaVersion: 1,
  reviewId: "NB-93",
  validator: "tools/continuous-tps-architecture-probe/new-baseline-nb27-nb85-supersession-decision/validate.mjs",
  status: failures.length === 0 ? "pass_gate_blocked" : "fail",
  checks,
  failureCount: failures.length,
  failures,
  decision: failures.length === 0 ? "defer_supersession_pending_product_scope" : "invalid_capability_supersession_packet",
  sideEffectPolicy: { nb27ContentRead: false, sourceModified: false, build: false, tests: false, runtime: false, media: false, network: false }
};
const resultPath = `${root}tools\\continuous-tps-architecture-probe\\results\\new-baseline-nb27-nb85-supersession-decision-nb93.json`;
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
