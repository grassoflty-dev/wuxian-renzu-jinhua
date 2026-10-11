import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "../../..");
const reviewPath = path.join(import.meta.dirname, "review.json");
const docPath = path.join(root, "docs/GREY_HIVE_GH_B0_WRITE_CONFLICT_REVIEW.md");
const nb70Path = path.join(root, "tools/continuous-tps-architecture-probe/new-baseline-grey-hive-gh-b0-implementation-preflight/preflight.json");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-gh-b0-write-conflict-review-nb74.json");

const errors = [];
const exists = (file) => fs.existsSync(path.join(root, file));
const readJson = (file) => JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
const requireTrue = (condition, message) => { if (!condition) errors.push(message); };
const review = JSON.parse(fs.readFileSync(reviewPath, "utf8"));
const nb70 = readJson("tools/continuous-tps-architecture-probe/new-baseline-grey-hive-gh-b0-implementation-preflight/preflight.json");
const doc = fs.readFileSync(docPath, "utf8");

requireTrue(review.schemaVersion === 1, "schemaVersion must be 1");
requireTrue(review.reviewId === "NB-74", "reviewId must be NB-74");
requireTrue(review.status === "grey_hive_gh_b0_write_conflict_review_complete", "status mismatch");
requireTrue(review.activeScopes.length === 3, "activeScopes must contain three known scopes");
requireTrue(review.flags.activeScopeContentRead === false, "active scope content must not be read");
requireTrue(review.flags.exclusiveLocksCreated === false, "actual locks must not be created");
requireTrue(review.conflictDecision.activeWriteConflict === true, "conflict flag must be true");
requireTrue(review.conflictDecision.value === "indeterminate", "conflict value must be indeterminate");
requireTrue(review.conflictDecision.conclusion === "indeterminate", "conclusion must be indeterminate");
requireTrue(review.conflictDecision.conflictClearForGateReviewSnapshot === false, "snapshot cannot be conflict-clear");

const nb70Files = nb70.changeManifestProposal.exactFiles;
requireTrue(JSON.stringify(review.ghB0WriteSet.exactFiles) === JSON.stringify(nb70Files), "GH-B0 write set differs from NB-70");
requireTrue(review.ghB0WriteSet.logicalUnitCount === 6, "GH-B0 logical unit count must be six");
requireTrue(review.ghB0WriteSet.existingSourceEdits.length === 1, "GH-B0 existing edit count must be one");
requireTrue(review.ghB0WriteSet.plannedNewFiles.length === 5, "GH-B0 planned new file count must be five");
requireTrue(JSON.stringify(review.ghB0WriteSet.fingerprintsFromNB70) === JSON.stringify(nb70.fingerprints), "NB-70 fingerprints were not carried exactly");
requireTrue(review.ghB0WriteSet.logicalUnits.map((x) => x.name).join("|") === nb70.ghB0Definition.logicalUnits.join("|"), "logical unit names differ from NB-70");

const knownActors = ["NB-72", "NB-73", "NB-47"];
for (const actor of knownActors) {
  const scope = review.activeScopes.find((x) => x.actor === actor);
  requireTrue(Boolean(scope), "missing active scope " + actor);
  if (scope) {
    requireTrue(scope.contentRead === false, actor + " contentRead must be false");
    requireTrue(scope.paths.length === 3, actor + " scope must have three known paths");
  }
}
const expectedScopePaths = {
  "NB-72": ["docs/FORMAL_RUNTIME_OBSERVABILITY_EVIDENCE_CONTRACT.md", "tools/continuous-tps-architecture-probe/new-baseline-formal-runtime-observability-evidence/**", "tools/continuous-tps-architecture-probe/results/new-baseline-formal-runtime-observability-evidence-nb72.json"],
  "NB-73": ["docs/UNVERIFIED_PACKAGE_QUARANTINE_DISPOSITION.md", "tools/continuous-tps-architecture-probe/new-baseline-unverified-package-quarantine-disposition/**", "tools/continuous-tps-architecture-probe/results/new-baseline-unverified-package-quarantine-disposition-nb73.json"],
  "NB-47": ["docs/FORMAL_RUNTIME_GATE_READINESS_LEDGER.md", "tools/continuous-tps-architecture-probe/new-baseline-formal-runtime-gate-readiness/**", "tools/continuous-tps-architecture-probe/results/new-baseline-formal-runtime-gate-readiness-nb47.json"]
};
for (const [actor, paths] of Object.entries(expectedScopePaths)) {
  const scope = review.activeScopes.find((x) => x.actor === actor);
  if (scope) requireTrue(JSON.stringify(scope.paths) === JSON.stringify(paths), actor + " scope path mismatch");
}

for (const item of review.workspacePathMetadata) {
  const actualExists = exists(item.path);
  requireTrue(actualExists === item.exists, "workspace existence drift: " + item.path);
  let status = "";
  try {
    status = execFileSync("git", ["status", "--short", "--", item.path], { cwd: root, encoding: "utf8" }).trim();
  } catch {
    errors.push("git status metadata unavailable: " + item.path);
  }
  if (!actualExists) requireTrue(status === "", "planned path has unexpected Git status: " + item.path);
  if (item.path === "server-rs/src/lib.rs") requireTrue(status.includes("M"), "lib.rs modified metadata no longer present");
  requireTrue(item.contentRead === false, "workspace contentRead must be false: " + item.path);
}
const planned = review.ghB0WriteSet.plannedNewFiles;
for (const file of planned) requireTrue(!exists(file), "planned file unexpectedly exists: " + file);
requireTrue(exists("server-rs/src/lib.rs"), "existing source edit target missing");

requireTrue(review.conflictMatrix.length >= 6, "conflict matrix incomplete");
for (const row of review.conflictMatrix) requireTrue(row.contentRead === false, "conflict matrix contentRead must be false");
requireTrue(review.conflictMatrix.some((x) => x.classification === "exact_overlap" && x.path === "server-rs/src/lib.rs"), "exact workspace overlap missing");
requireTrue(review.conflictMatrix.filter((x) => x.right === "NB-72" || x.right === "NB-73" || x.right === "NB-47").every((x) => x.classification === "no_overlap"), "known scope path comparison must be no_overlap");
requireTrue(review.conflictMatrix.some((x) => x.logicalOverlap === "unknown_writer_conflict"), "unknown writer conflict missing");
requireTrue(review.exclusiveLockProposal.length === 6, "lock proposal must cover six logical units");
for (const lock of review.exclusiveLockProposal) requireTrue(lock.futureOwner && lock.forbiddenConcurrentBatches && lock.staleFingerprint, "lock proposal incomplete");
requireTrue(review.snapshotValidity.contentRead === false, "snapshot contentRead must be false");
requireTrue(review.snapshotValidity.validForImplementation === false, "snapshot cannot authorize implementation");

const falseFlags = ["architectureGateOpened", "ghB0Approved", "implementationStarted", "blockersClosed", "artifactContractAccepted", "sourceModified", "dataModified", "ciModified", "buildExecuted", "testsExecuted", "runtimeExecuted", "userSaveRead", "userSaveWritten", "assetBytesRead", "mediaBytesRead", "mediaCreated", "mediaModelCalled", "network"];
for (const key of falseFlags) requireTrue(review.flags[key] === false || (key === "blockersClosed" && review.flags[key] === 0), key + " flag mismatch");

const forbidden = ["TP-" + "15C3", "TP-" + "18D1", "TP-" + "01R11", "TP-" + "18D2"];
for (const token of forbidden) {
  requireTrue(!doc.includes(token), "document contains forbidden token: " + token);
  requireTrue(!JSON.stringify(review).includes(token), "review contains forbidden token: " + token);
}
requireTrue(doc.includes("activeWriteConflict=true"), "document must record active conflict");
requireTrue(doc.includes("没有读取活动包内容"), "document must state active content was not read");
requireTrue(doc.includes("没有创建实际锁"), "document must state no actual lock");
requireTrue(doc.includes("server-rs/src/lib.rs"), "document must list exact GH-B0 target");
requireTrue(doc.includes("exact path"), "document must state exact path comparison");

const result = {
  schemaVersion: 1,
  reviewId: "NB-74",
  status: "grey_hive_gh_b0_write_conflict_review_complete",
  activeWriteConflict: "indeterminate",
  activeWriteConflictFlag: true,
  activeScopeContentRead: false,
  exclusiveLocksCreated: false,
  architectureGateOpened: false,
  ghB0Approved: false,
  implementationStarted: false,
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
  ghB0FileCount: review.ghB0WriteSet.exactFiles.length,
  activeScopeCount: review.activeScopes.length,
  conflictRowCount: review.conflictMatrix.length,
  snapshotValidForImplementation: false,
  checkedAt: new Date().toISOString(),
  errors
};
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
if (errors.length) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
