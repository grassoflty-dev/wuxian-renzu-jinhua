import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../../..");
const inventoryPath = path.join(import.meta.dirname, "inventory.json");
const docPath = path.join(root, "docs/F5_BABYLON_WORLD_V3_SNAPSHOT_VIEW_CODE_INVENTORY.md");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/new-baseline-f5-babylon-world-v3-snapshot-view-inventory-nb59.json");

const errors = [];
const exists = (file) => fs.existsSync(path.join(root, file));
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const requireTrue = (condition, message) => { if (!condition) errors.push(message); };
const requireAnchor = (entry) => {
  if (entry.kind === "new") {
    requireTrue(!exists(entry.path), `new candidate already exists: ${entry.path}`);
    return;
  }
  requireTrue(exists(entry.path), `missing path: ${entry.path}`);
  if (entry.anchor && exists(entry.path)) {
    requireTrue(read(entry.path).includes(entry.anchor), `missing anchor ${entry.anchor} in ${entry.path}`);
  }
};

const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const doc = fs.readFileSync(docPath, "utf8");
const allEntries = [
  ...inventory.bootstrapEntries,
  ...inventory.snapshotEntries,
  ...inventory.viewPathFindings,
  ...inventory.localAuthorityFindings,
  ...inventory.fallbackFindings
];
const ids = allEntries.map((entry) => entry.id);
requireTrue(new Set(ids).size === ids.length, "entry IDs must be unique");
for (const entry of allEntries) requireAnchor(entry);
for (const dep of inventory.dependencies) requireTrue(exists(dep.path), `missing dependency path: ${dep.path}`);

requireTrue(inventory.schemaVersion === 1, "schemaVersion must be 1");
requireTrue(inventory.inventoryId === "NB-59", "inventoryId must be NB-59");
requireTrue(inventory.status === "f5_babylon_world_v3_snapshot_view_inventory_complete_gate_blocked", "status mismatch");
requireTrue(inventory.currentDecision.babylonFormalEntry === true, "Babylon formal entry must be true");
requireTrue(inventory.currentDecision.worldV3SnapshotViewWired === false, "World v3 snapshot view must remain unwired");
requireTrue(inventory.currentDecision.localAuthorityRemoved === false, "local authority must remain unresolved");
requireTrue(inventory.currentDecision.apiFallbackRemoved === false, "API fallback must remain unresolved");
requireTrue(inventory.currentDecision.minimalPatchApproved === false, "minimal patch must remain unapproved");
for (const key of ["sourceModified", "buildExecuted", "runtimeExecuted", "assetBytesRead", "mediaBytesRead", "mediaCreated", "mediaModelCalled", "network"]) {
  requireTrue(inventory.currentDecision[key] === false, `${key} must be false`);
  requireTrue(inventory.scanMetadata[key] === false, `scanMetadata.${key} must be false`);
}
requireTrue(inventory.fallbackFindings.some((finding) => finding.id === "F5-FALLBACK-005" && finding.resolved === true), "Babylon-only formal fallback finding missing");
requireTrue(inventory.fallbackFindings.filter((finding) => finding.classification === "api_fallback").every((finding) => finding.resolved === false), "API fallback finding was incorrectly resolved");
requireTrue(inventory.localAuthorityFindings.some((finding) => finding.resolved === false), "local authority findings must remain open");
requireTrue(inventory.snapshotContract.formal === false, "snapshot contract must not be marked formal");
for (const field of ["schemaVersion", "worldId", "worldVersion", "authorityRevision", "tick", "sequence", "time", "entities", "Transform/Velocity", "actorVisualState", "doors/triggers/interactionMarkers", "cameraTargetHint", "presentationEvents", "tombstones", "semantics", "reconnect/resync"]) {
  requireTrue(inventory.snapshotContract.requiredFields.some((value) => value.includes(field)), `snapshot contract missing ${field}`);
}
requireTrue(inventory.commandViewBoundary.transport.includes("Tauri"), "command/view transport must be Tauri");
requireTrue(inventory.commandViewBoundary.transport.includes("no HTTP/mock fallback"), "command/view must reject HTTP/mock fallback");
requireTrue(inventory.commandViewBoundary.forbiddenClientWrites.includes("Save"), "client Save write must be forbidden");
requireTrue(inventory.reconciliationGaps.length >= 8, "reconciliation gaps incomplete");
requireTrue(inventory.resourceLifecycle.length >= 6, "resource lifecycle incomplete");
for (const id of ["F1", "F2", "F3", "F4", "F6", "F7"]) requireTrue(inventory.dependencies.some((dep) => dep.id === id), `missing dependency ${id}`);

const patch = inventory.minimalPatchBoundary;
requireTrue(patch.logicalUnitCount === patch.candidates.length, "patch count mismatch");
requireTrue(patch.logicalUnitCount <= 8 || patch.requires_split === true, "patch must be <=8 or require split");
requireTrue(patch.approved === false, "patch must not be approved");
for (const candidate of patch.candidates) requireAnchor(candidate);

const forbidden = [
  "TP-" + "15C3",
  "TP-" + "18D1",
  "TP-" + "01R11",
  "TP-" + "18D2",
  "NB-" + "58"
];
for (const token of forbidden) {
  requireTrue(!doc.includes(token), `document contains excluded token: ${token}`);
  requireTrue(!JSON.stringify(inventory).includes(token), `inventory contains excluded token: ${token}`);
}
requireTrue(doc.includes("不读取资产或媒体"), "document must state asset/media bytes were not read");
requireTrue(doc.includes("不执行构建、测试、浏览器、游戏或运行时验证"), "document must state no runtime/build/test execution");
requireTrue(doc.includes("不静默切换旧渲染器"), "document must state no silent renderer fallback");
requireTrue(doc.includes("requires_split=false"), "document must state split decision");
requireTrue(doc.includes("worldV3SnapshotViewWired：false"), "document must state snapshot view is not wired");

const result = {
  schemaVersion: 1,
  inventoryId: "NB-59",
  status: errors.length === 0 ? "pass_gate_blocked" : "fail",
  babylonFormalEntry: true,
  worldV3SnapshotViewWired: false,
  localAuthorityRemoved: false,
  apiFallbackRemoved: false,
  minimalPatchApproved: false,
  sourceModified: false,
  buildExecuted: false,
  runtimeExecuted: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  logicalUnitCount: patch.logicalUnitCount,
  requires_split: patch.requires_split,
  checkedAt: new Date().toISOString(),
  errors
};
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
if (errors.length) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));

