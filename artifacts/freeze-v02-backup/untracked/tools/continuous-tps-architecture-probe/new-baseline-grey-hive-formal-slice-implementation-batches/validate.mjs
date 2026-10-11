import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../../..");
const planPath = path.join(import.meta.dirname, "plan.json");
const docPath = path.join(root, "docs/GREY_HIVE_FORMAL_SLICE_IMPLEMENTATION_BATCH_PLAN.md");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-formal-slice-implementation-batches-nb66.json");

const errors = [];
const exists = (file) => fs.existsSync(path.join(root, file));
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const requireTrue = (condition, message) => { if (!condition) errors.push(message); };
const plan = JSON.parse(fs.readFileSync(planPath, "utf8"));
const doc = fs.readFileSync(docPath, "utf8");

for (const entry of plan.sourceEvidence) {
  requireTrue(exists(entry.path), "missing evidence path: " + entry.path);
  if (entry.anchor && exists(entry.path)) requireTrue(read(entry.path).includes(entry.anchor), "missing evidence anchor " + entry.anchor + " in " + entry.path);
}
for (const dep of plan.dependencyMatrix) {
  requireTrue(["F1", "F2", "F3", "F4", "F5", "F6", "F7"].includes(dep.id), "unknown dependency: " + dep.id);
  requireTrue(dep.status === "open", dep.id + " must remain open");
}
requireTrue(plan.schemaVersion === 1, "schemaVersion must be 1");
requireTrue(plan.planId === "NB-66", "planId must be NB-66");
requireTrue(plan.status === "grey_hive_formal_slice_implementation_batch_plan_ready_gate_blocked", "status mismatch");
requireTrue(plan.candidateRegistry.length >= 18, "candidate registry incomplete");
const registryIds = plan.candidateRegistry.map((x) => x.id);
requireTrue(new Set(registryIds).size === registryIds.length, "candidate registry IDs must be unique");
requireTrue(plan.candidateRegistry.every((x) => x.approvalState === "not_approved"), "candidate approval state must be not_approved");

const expectedCounts = { rooms: 18, connectors: 17, doors: 17, triggers: 15, checkpoints: 6 };
for (const section of plan.topologyInvariants) {
  if (section.baseline) requireTrue(JSON.stringify(section.baseline) === JSON.stringify(expectedCounts), "baseline count invariant mismatch");
  if (section.observed) requireTrue(JSON.stringify(section.observed) === JSON.stringify(expectedCounts), "observed count invariant mismatch");
}
requireTrue(plan.topologyInvariants.some((x) => x.id === "TOPO-002"), "reference closure invariant missing");
requireTrue(plan.topologyInvariants.some((x) => x.id === "TOPO-005" && x.rule.includes("no Grid fallback")), "no Grid fallback invariant missing");

const batches = plan.batches;
requireTrue(batches.length <= 6, "more than six batches");
requireTrue(batches.length === plan.dag.nodeCount, "DAG node count mismatch");
requireTrue(batches.every((b) => b.approvalState === "not_approved"), "batch approval state must be not_approved");
requireTrue(batches.every((b) => b.logicalUnits.length <= 8), "batch exceeds eight logical units");
requireTrue(batches[0].batchId === "GH-B0", "first batch must be GH-B0");
requireTrue(batches[0].forbiddenWork.some((x) => x.includes("start world")), "first batch must not start runtime");
requireTrue(batches[0].forbiddenWork.some((x) => x.includes("read/write user save")), "first batch must not touch user save");
requireTrue(batches[0].approvalState === "not_approved", "first batch must remain unapproved");

const batchIds = new Set(batches.map((x) => x.batchId));
for (const node of plan.dag.nodes) requireTrue(batchIds.has(node), "DAG node missing batch " + node);
requireTrue(plan.dag.edgeCount === plan.dag.edges.length, "DAG edge count mismatch");
const graph = new Map(plan.dag.nodes.map((n) => [n, []]));
for (const [from, to] of plan.dag.edges) {
  requireTrue(batchIds.has(from) && batchIds.has(to), "DAG edge references unknown node");
  graph.get(from).push(to);
}
const visiting = new Set();
const visited = new Set();
const visit = (node) => {
  if (visiting.has(node)) return false;
  if (visited.has(node)) return true;
  visiting.add(node);
  for (const next of graph.get(node) || []) if (!visit(next)) return false;
  visiting.delete(node);
  visited.add(node);
  return true;
};
for (const node of plan.dag.nodes) requireTrue(visit(node), "DAG contains a cycle");
requireTrue(plan.dag.acyclic === true, "DAG acyclic flag must be true");

for (const invariant of plan.gameplayInvariants) requireTrue(invariant.status !== "implemented" && invariant.status !== "approved", "gameplay invariant cannot be implemented/approved");
requireTrue(plan.gameplayInvariants.some((x) => x.id === "GAME-001" && x.rule.includes("hive_lockdown only")), "Gate B invariant missing");
requireTrue(plan.gameplayInvariants.some((x) => x.id === "GAME-002" && x.rule.includes("taken/left/unresolved")), "choice invariant missing");
requireTrue(plan.gameplayInvariants.some((x) => x.id === "GAME-003" && x.rule.includes("independent")), "NPC invariant missing");
requireTrue(plan.sharedFileLocks.length >= 8, "shared file locks incomplete");
requireTrue(plan.evidenceLadder.length === 7, "evidence ladder incomplete");
requireTrue(plan.productDecisionGates.every((x) => x.status === "pending"), "product decisions must remain pending");
requireTrue(plan.artifactContractRules.accepted === false, "artifact contract must remain unaccepted");
requireTrue(plan.artifactContractRules.literalComparison === true, "artifact contract must use literal comparison");
requireTrue(plan.artifactContractRules.mismatchPolicy === "fail_closed", "artifact mismatch must fail closed");
requireTrue(plan.mediaReleaseBoundary.technicalBlockoutMayProceedWithoutFinalMedia === true, "technical blockout media boundary missing");
requireTrue(plan.mediaReleaseBoundary.publicReleaseRequires.length >= 4, "public media/release gates incomplete");

const expectedFalse = {
  architectureGateOpened: false,
  greyHiveBatchesApproved: false,
  implementationStarted: false,
  greyHiveRuntimeLoaded: false,
  greyHiveFormalPlayableAccepted: false,
  blockersClosed: 0,
  artifactContractAccepted: false,
  sourceModified: false,
  dataModified: false,
  ciModified: false,
  buildExecuted: false,
  testsExecuted: false,
  packagedRuntimeExecuted: false,
  manualPlaytestExecuted: false,
  userSaveRead: false,
  userSaveWritten: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false
};
for (const [key, value] of Object.entries(expectedFalse)) requireTrue(plan.currentDecision[key] === value, key + " flag mismatch");
requireTrue(plan.currentDecision.blockersClosed === 0, "blockersClosed must be zero");
requireTrue(plan.currentDecision.architectureGateOpened === false, "Architecture Gate must remain closed");

const forbidden = [
  "TP-" + "15C3",
  "TP-" + "18D1",
  "TP-" + "01R11",
  "TP-" + "18D2",
  "NB-" + "47",
  "NB-" + "64",
  "NB-" + "65"
];
for (const token of forbidden) {
  requireTrue(!doc.includes(token), "document contains excluded token: " + token);
  requireTrue(!JSON.stringify(plan).includes(token), "plan contains excluded token: " + token);
}
requireTrue(doc.includes("不批准、不实施"), "document must state plan-only boundary");
requireTrue(doc.includes("候选共 6 个逻辑单元"), "document must state first loader batch size");
requireTrue(doc.includes("artifact mismatch"), "document must state artifact mismatch policy");
requireTrue(doc.includes("产品决策"), "document must state product decision gates");
requireTrue(doc.includes("no Grid/tile/px/py fallback"), "document must state no Grid fallback");
requireTrue(doc.includes("Architecture Gate 仍关闭"), "document must state Gate closed");

const result = {
  schemaVersion: 1,
  planId: "NB-66",
  status: errors.length === 0 ? "pass_gate_blocked" : "fail",
  architectureGateOpened: false,
  greyHiveBatchesApproved: false,
  implementationStarted: false,
  greyHiveRuntimeLoaded: false,
  greyHiveFormalPlayableAccepted: false,
  blockersClosed: 0,
  artifactContractAccepted: false,
  sourceModified: false,
  dataModified: false,
  ciModified: false,
  buildExecuted: false,
  testsExecuted: false,
  packagedRuntimeExecuted: false,
  manualPlaytestExecuted: false,
  userSaveRead: false,
  userSaveWritten: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  batchCount: batches.length,
  batchLogicalUnitCounts: Object.fromEntries(batches.map((b) => [b.batchId, b.logicalUnits.length])),
  dagNodeCount: plan.dag.nodeCount,
  dagEdgeCount: plan.dag.edgeCount,
  candidateCount: plan.candidateRegistry.length,
  checkedAt: new Date().toISOString(),
  errors
};
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
if (errors.length) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
