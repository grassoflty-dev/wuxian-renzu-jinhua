import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const root = path.resolve(import.meta.dirname, "../../..");
const preflightPath = path.join(import.meta.dirname, "preflight.json");
const docPath = path.join(root, "docs/GREY_HIVE_GH_B0_IMPLEMENTATION_PREFLIGHT.md");
const planPath = path.join(root, "tools/continuous-tps-architecture-probe/new-baseline-grey-hive-formal-slice-implementation-batches/plan.json");
const planDocPath = path.join(root, "docs/GREY_HIVE_FORMAL_SLICE_IMPLEMENTATION_BATCH_PLAN.md");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-gh-b0-implementation-preflight-nb70.json");

const errors = [];
const exists = (file) => fs.existsSync(path.join(root, file));
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const hash = (value) => crypto.createHash("sha256").update(value).digest("hex");
const requireTrue = (condition, message) => { if (!condition) errors.push(message); };

const preflight = JSON.parse(fs.readFileSync(preflightPath, "utf8"));
const plan = JSON.parse(fs.readFileSync(planPath, "utf8"));
const doc = fs.readFileSync(docPath, "utf8");
const planDoc = fs.readFileSync(planDocPath, "utf8");

requireTrue(preflight.schemaVersion === 1, "schemaVersion must be 1");
requireTrue(preflight.preflightId === "NB-70", "preflightId must be NB-70");
requireTrue(preflight.status === "grey_hive_gh_b0_preflight_complete_gate_blocked", "status mismatch");
requireTrue(preflight.sourcePlanIdentity.planId === "NB-66", "source plan identity mismatch");
requireTrue(preflight.sourcePlanIdentity.batchId === "GH-B0", "batch identity mismatch");
requireTrue(preflight.sourcePlanIdentity.logicalUnitCount === 6, "GH-B0 must contain exactly six logical units");
requireTrue(preflight.sourcePlanIdentity.approvalState === "not_approved", "source batch must remain unapproved");

const expectedUnits = plan.batches.find((batch) => batch.batchId === "GH-B0");
requireTrue(Boolean(expectedUnits), "GH-B0 missing from NB-66 plan");
if (expectedUnits) {
  requireTrue(JSON.stringify(preflight.ghB0Definition.logicalUnits) === JSON.stringify(expectedUnits.logicalUnits), "GH-B0 logical units differ from NB-66");
  requireTrue(preflight.ghB0Definition.name === expectedUnits.name, "GH-B0 name differs from NB-66");
  requireTrue(preflight.ghB0Definition.objective === expectedUnits.objective, "GH-B0 objective differs from NB-66");
  requireTrue(JSON.stringify(preflight.ghB0Definition.forbiddenWork) === JSON.stringify(expectedUnits.forbiddenWork), "GH-B0 forbidden work differs from NB-66");
  requireTrue(preflight.ghB0Definition.approvalState === expectedUnits.approvalState, "GH-B0 approval differs from NB-66");
  requireTrue(preflight.ghB0Definition.logicalUnits.length === expectedUnits.logicalUnits.length, "GH-B0 unit count differs from NB-66");
}
requireTrue(planDoc.includes("GH-B0 候选共 6 个逻辑单元"), "GH-B0 document unit declaration missing");
requireTrue(planDoc.includes("Data/Loader Static Foundation"), "GH-B0 document name missing");

const unitIds = preflight.logicalUnits.map((unit) => unit.id);
requireTrue(new Set(unitIds).size === unitIds.length, "logical unit IDs must be unique");
requireTrue(preflight.logicalUnits.length === 6, "logical unit list must have six entries");
requireTrue(preflight.logicalUnits.every((unit) => unit.files.length > 0), "logical unit files missing");

for (const anchor of preflight.sourceAnchors) {
  requireTrue(exists(anchor.path), "source anchor path missing: " + anchor.path);
  if (exists(anchor.path)) {
    const text = read(anchor.path);
    requireTrue(text.includes(anchor.anchor), "source anchor missing: " + anchor.anchor + " in " + anchor.path);
    requireTrue(hash(anchor.anchor) === anchor.anchorSha256, "anchor fingerprint formula mismatch: " + anchor.path);
    requireTrue(hash(text) === anchor.fileSha256, "source file drift: " + anchor.path);
    requireTrue(anchor.verified === true, "source anchor not verified: " + anchor.path);
  }
}
requireTrue(preflight.sourceAnchors.length === preflight.fingerprints.length, "fingerprint list mismatch");

for (const item of preflight.plannedNewFiles) {
  requireTrue(exists(item.parent), "planned parent missing: " + item.parent);
  requireTrue(exists(item.registrationPath), "planned registration path missing: " + item.registrationPath);
  if (exists(item.registrationPath)) requireTrue(read(item.registrationPath).includes(item.registrationAnchor), "planned registration anchor missing: " + item.registrationAnchor);
  requireTrue(item.parentExists === exists(item.parent), "parentExists flag mismatch: " + item.path);
  requireTrue(item.existing === exists(item.path), "planned new existence flag mismatch: " + item.path);
  requireTrue(item.create === false, "preflight may not create new files");
  requireTrue(item.existing === false, "planned new file already exists: " + item.path);
}
requireTrue(preflight.plannedNewFiles.length === 5, "planned new file count must be five");
requireTrue(preflight.changeManifestProposal.existingSourceEdits.length === 1, "existing source edit count must be one");
requireTrue(preflight.changeManifestProposal.noDeletion === true, "deletion must be forbidden");
requireTrue(preflight.changeManifestProposal.noRename === true, "rename must be forbidden");
requireTrue(preflight.changeManifestProposal.noBootstrapActivation === true, "bootstrap activation must be forbidden");
requireTrue(preflight.changeManifestProposal.noGridFallback === true, "Grid fallback must be forbidden");

const expectedCounts = { rooms: 18, connectors: 17, doors: 17, triggers: 15, checkpoints: 6 };
requireTrue(JSON.stringify(preflight.dataInvariants.counts.baseline) === JSON.stringify(expectedCounts), "baseline counts mismatch");
requireTrue(JSON.stringify(preflight.dataInvariants.counts.observed) === JSON.stringify(expectedCounts), "observed counts mismatch");
requireTrue(preflight.dataInvariants.stableIdsUnique === true, "stable ID invariant missing");
requireTrue(preflight.dataInvariants.referenceClosure === "candidate_static_only", "reference closure scope mismatch");
requireTrue(preflight.dataInvariants.gateB === "hive_lockdown_only", "Gate B invariant mismatch");
requireTrue(JSON.stringify(preflight.dataInvariants.choiceStates) === JSON.stringify(["taken", "left", "unresolved"]), "choice states mismatch");
requireTrue(preflight.dataInvariants.choiceGatesB === false, "choice must not gate B");
requireTrue(preflight.dataInvariants.npcIndependent === true, "NPC independence missing");
requireTrue(preflight.dataInvariants.noPartialLoad === true, "no partial load invariant missing");
requireTrue(preflight.dataInvariants.defaultOff === true, "default off invariant missing");
requireTrue(preflight.dataInvariants.noGridFallback === true, "no Grid fallback invariant missing");

const conflict68 = preflight.conflictMatrix.find((x) => x.actor === "NB-68");
const conflict69 = preflight.conflictMatrix.find((x) => x.actor === "NB-69");
const conflict47 = preflight.conflictMatrix.find((x) => x.actor === "NB-47");
requireTrue(conflict68 && conflict68.outputRead === false && conflict68.overlapKnown === false, "NB-68 conflict boundary invalid");
requireTrue(conflict69 && conflict69.outputRead === false && conflict69.overlapKnown === false, "NB-69 conflict boundary invalid");
requireTrue(conflict47 && conflict47.outputRead === false, "NB-47 must not be consumed");
requireTrue(preflight.currentDecision.activeWriteConflict === true, "unknown active writer scope must fail closed");
requireTrue(preflight.currentDecision.productDecisionBlocked === false, "GH-B0 should not depend on product values");
requireTrue(preflight.productDecisionCheck.productDecisionBlocked === false, "product decision check mismatch");
requireTrue(preflight.productDecisionCheck.explicitlyNotFilled.length >= 5, "pending product values must be listed");

const gate = preflight.currentDecision;
const expectedFlags = {
  ghB0DefinitionMatched: true,
  sourceAnchorsVerified: true,
  plannedNewFileParentsVerified: true,
  productDecisionBlocked: false,
  defaultOff: true,
  runtimeEntryRegistered: false,
  architectureGateOpened: false,
  ghB0Approved: false,
  implementationStarted: false,
  greyHiveRuntimeLoaded: false,
  blockersClosed: 0,
  artifactContractAccepted: false,
  sourceModified: false,
  dataModified: false,
  ciModified: false,
  buildExecuted: false,
  testsExecuted: false,
  runtimeExecuted: false,
  userSaveRead: false,
  userSaveWritten: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false
};
for (const [key, value] of Object.entries(expectedFlags)) requireTrue(gate[key] === value, key + " flag mismatch");
requireTrue(gate.status === "blocked_by_active_conflict", "conclusion must be blocked_by_active_conflict");

const forbidden = [
  "TP-" + "15C3",
  "TP-" + "18D1",
  "TP-" + "01R11",
  "TP-" + "18D2"
];
for (const token of forbidden) {
  requireTrue(!doc.includes(token), "document contains forbidden token: " + token);
  requireTrue(!JSON.stringify(preflight).includes(token), "preflight contains forbidden token: " + token);
}
const forbiddenOutputPaths = ["results/nb47", "results/nb68", "results/nb69"];
for (const token of forbiddenOutputPaths) {
  requireTrue(!doc.toLowerCase().includes(token), "document contains forbidden output path: " + token);
  requireTrue(!JSON.stringify(preflight).toLowerCase().includes(token), "preflight contains forbidden output path: " + token);
}
requireTrue(doc.includes("不批准、不实施"), "document must state no approval/implementation");
requireTrue(doc.includes("activeWriteConflict=true"), "document must state active conflict");
requireTrue(doc.includes("SHA-256"), "document must state fingerprints");
requireTrue(doc.includes("不创建 planned new files"), "document must state no file creation");
requireTrue(doc.includes("productDecisionBlocked=false"), "document must state product decision result");
requireTrue(doc.includes("defaultOff=true"), "document must state default off");
requireTrue(doc.includes("runtimeEntryRegistered=false"), "document must state no runtime entry");

const result = {
  schemaVersion: 1,
  preflightId: "NB-70",
  status: errors.length === 0 ? "pass_gate_blocked" : "fail",
  conclusion: "blocked_by_active_conflict",
  ghB0DefinitionMatched: true,
  sourceAnchorsVerified: true,
  plannedNewFileParentsVerified: true,
  activeWriteConflict: true,
  productDecisionBlocked: false,
  defaultOff: true,
  runtimeEntryRegistered: false,
  architectureGateOpened: false,
  ghB0Approved: false,
  implementationStarted: false,
  greyHiveRuntimeLoaded: false,
  blockersClosed: 0,
  artifactContractAccepted: false,
  sourceModified: false,
  dataModified: false,
  ciModified: false,
  buildExecuted: false,
  testsExecuted: false,
  runtimeExecuted: false,
  userSaveRead: false,
  userSaveWritten: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  logicalUnitCount: preflight.logicalUnits.length,
  existingSourceAnchorCount: preflight.sourceAnchors.length,
  plannedNewFileCount: preflight.plannedNewFiles.length,
  checkedAt: new Date().toISOString(),
  errors
};
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
if (errors.length) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));

