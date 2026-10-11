import fs from "node:fs";
import { execFileSync } from "node:child_process";

const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const bundlePath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-world-production-supersession-decision-bundle\\decisions.json`;
const bundle = JSON.parse(fs.readFileSync(bundlePath, "utf8"));
const w1 = JSON.parse(fs.readFileSync(`${root}tools\\continuous-tps-architecture-probe\\new-baseline-nb27-nb85-supersession-decision\\decision.json`, "utf8"));
const w2 = JSON.parse(fs.readFileSync(`${root}tools\\continuous-tps-architecture-probe\\new-baseline-nb37-nb87-supersession-decision\\decision.json`, "utf8"));
const w3 = JSON.parse(fs.readFileSync(`${root}tools\\continuous-tps-architecture-probe\\new-baseline-nb42-nb89-supersession-decision\\decision.json`, "utf8"));
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const topoSort = (nodes, edges) => { const indegree = new Map(nodes.map(node => [node, 0])); const next = new Map(nodes.map(node => [node, []])); for (const [from, to] of edges) { indegree.set(to, indegree.get(to) + 1); next.get(from).push(to); } const q = nodes.filter(node => indegree.get(node) === 0); let seen = 0; while (q.length) { const node = q.shift(); seen++; for (const to of next.get(node)) { indegree.set(to, indegree.get(to) - 1); if (indegree.get(to) === 0) q.push(to); } } return seen === nodes.length; };
const checks = {
  status: bundle.status === "world_production_supersession_decision_bundle_ready",
  three_records: bundle.records.length === 3 && ["W1", "W2", "W3"].every(id => bundle.records.some(record => record.id === id)),
  upstream_identity: bundle.records[0].oldPackage.id === "NB-27" && bundle.records[0].replacement.id === "NB-85" && bundle.records[1].oldPackage.id === "NB-37" && bundle.records[1].replacement.id === "NB-87" && bundle.records[2].oldPackage.id === "NB-42" && bundle.records[2].replacement.id === "NB-89",
  upstream_coverage: w1.coverageMatrix.scopeCoverageComplete === true && w2.coverageMatrix.scopeCoverageComplete === true && w3.scopeChecklist.every(item => ["covered", "partial"].includes(item.result)) && bundle.records.every(record => record.scopeCoverage.complete === true),
  recommendations_match: bundle.records[0].recommendation === w1.recommendation.id && bundle.records[1].recommendation === w2.recommendation.id && bundle.records[2].recommendation === w3.recommendation.value,
  quarantines_and_unconsumed: bundle.records.every(record => record.oldPackage.contentRead === false && record.oldPackage.quarantined === true) && bundle.postDecisionBoundaries.oldPackagesRemainQuarantined === true,
  dependencies_complete: bundle.dependencyDag.acyclic === true && topoSort(bundle.dependencyDag.nodes, bundle.dependencyDag.edges) && bundle.dependencyDag.edges.some(edge => edge[0] === "NB-90" && edge[1] === "W1") && bundle.dependencyDag.edges.some(edge => edge[0] === "FiveBlockers" && edge[1] === "ArchitectureGateBundle"),
  review_order: bundle.reviewOrder.length === 3 && bundle.reviewOrder[0].record === "W2" && bundle.reviewOrder[0].automaticDecision === false && bundle.reviewOrder[1].record === "W1" && bundle.reviewOrder[2].record === "W3",
  product_pending: bundle.productDecisionLinks.answersRecorded === 0 && bundle.productDecisionLinks.allPending === true && bundle.productDecisionLinks.nb90Read === false,
  acceptance_pending: bundle.currentDecision.decisionRecordCount === 3 && bundle.currentDecision.pendingCount === 3 && bundle.currentDecision.acceptedCount === 0 && bundle.currentDecision.rejectedCount === 0 && bundle.currentDecision.deferredCount === 0 && bundle.acceptanceSchema.decisionId === null && bundle.acceptanceSchema.selectedDecision === null,
  no_automatic_accept: bundle.records.every(record => record.postState.replacementScopeEvidenceOnly === true) && bundle.postDecisionBoundaries.capabilityCatalogApproved === false && bundle.postDecisionBoundaries.returnStationApproved === false && bundle.postDecisionBoundaries.threeWorldRolloutAccepted === false,
  current_flags: bundle.currentDecision.oldPackageContentRead === false && bundle.currentDecision.nb94Read === false && bundle.currentDecision.architectureGateOpened === false && bundle.currentDecision.implementationStarted === false && bundle.currentDecision.blockersClosed === 0,
  no_execution: ["sourceModified", "buildExecuted", "testsExecuted", "runtimeExecuted", "mediaBytesRead", "mediaCreated", "mediaModelCalled", "network"].every(key => bundle.currentDecision[key] === false),
  source_unchanged: git("status", "--short", "--", "server-rs/src/lib.rs") === "M server-rs/src/lib.rs"
};
const failures = Object.entries(checks).filter(([, value]) => !value).map(([key]) => key);
const result = {
  schemaVersion: 1,
  reviewId: "NB-95",
  validator: "tools/continuous-tps-architecture-probe/new-baseline-world-production-supersession-decision-bundle/validate.mjs",
  status: failures.length === 0 ? "pass_gate_blocked" : "fail",
  checks,
  failureCount: failures.length,
  failures,
  decision: failures.length === 0 ? "three_records_ready_all_decisions_pending" : "invalid_world_production_bundle",
  sideEffectPolicy: { oldPackageContentRead: false, sourceModified: false, build: false, tests: false, runtime: false, media: false, network: false }
};
const resultPath = `${root}tools\\continuous-tps-architecture-probe\\results\\new-baseline-world-production-supersession-decision-bundle-nb95.json`;
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
