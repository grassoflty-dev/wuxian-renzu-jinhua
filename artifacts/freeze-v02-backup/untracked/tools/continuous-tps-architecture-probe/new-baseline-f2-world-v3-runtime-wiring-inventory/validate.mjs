import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../../..");
const inventoryPath = path.join(import.meta.dirname, "inventory.json");
const docPath = path.join(root, "docs/F2_WORLD_V3_RUNTIME_WIRING_CODE_INVENTORY.md");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/new-baseline-f2-world-v3-runtime-wiring-inventory-nb56.json");

const errors = [];
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const exists = (file) => fs.existsSync(path.join(root, file));
const requireTrue = (condition, message) => { if (!condition) errors.push(message); };
const requireAnchor = (entry) => {
  requireTrue(exists(entry.path), `missing source path: ${entry.path}`);
  if (entry.anchor && exists(entry.path)) {
    requireTrue(read(entry.path).includes(entry.anchor), `missing anchor ${entry.anchor} in ${entry.path}`);
  }
};

const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const doc = fs.readFileSync(docPath, "utf8");

requireTrue(inventory.schemaVersion === 1, "schemaVersion must be 1");
requireTrue(inventory.inventoryId === "NB-56", "inventoryId must be NB-56");
requireTrue(inventory.status === "f2_world_v3_runtime_wiring_inventory_complete_gate_blocked", "status must remain gate blocked");
requireTrue(inventory.currentDecision.blocker === "E_WORLD_V3_NOT_RUNTIME_WIRED", "E_WORLD_V3_NOT_RUNTIME_WIRED must remain open");
requireTrue(inventory.currentDecision.worldV3RuntimeWired === false, "worldV3RuntimeWired must be false");
requireTrue(inventory.currentDecision.greyHiveRuntimeLoaded === false, "greyHiveRuntimeLoaded must be false");
requireTrue(inventory.currentDecision.minimalPatchApproved === false, "minimalPatchApproved must be false");
requireTrue(inventory.scanMetadata.sourceModified === false, "sourceModified flag must be false");
requireTrue(inventory.scanMetadata.buildExecuted === false, "buildExecuted flag must be false");
requireTrue(inventory.scanMetadata.runtimeExecuted === false, "runtimeExecuted flag must be false");
requireTrue(inventory.scanMetadata.mediaRead === false, "mediaRead flag must be false");
requireTrue(inventory.scanMetadata.networkAccessed === false, "networkAccessed flag must be false");
requireTrue(inventory.currentDecision.legacyGridFallbackAllowed === false, "legacy fallback must be disallowed");

for (const entry of [...inventory.worldModelEntries, ...inventory.candidateEntries, ...inventory.formalRuntimeRoots, ...inventory.wiringFindings]) {
  requireAnchor(entry);
}
for (const dep of inventory.dependencies) {
  requireTrue(exists(dep.path), `missing dependency evidence path: ${dep.path}`);
}

const patch = inventory.minimalPatchBoundary;
requireTrue(patch.logicalUnitCount === patch.files.length, "logicalUnitCount must match files");
requireTrue(patch.logicalUnitCount <= 8, "minimal patch exceeds eight logical units");
requireTrue(patch.requiresSplit === false, "minimal patch must not require a split");
requireTrue(patch.approved === false, "minimal patch must not be approved");
for (const entry of patch.files) {
  if (entry.kind === "existing") requireTrue(exists(entry.path), `existing patch path missing: ${entry.path}`);
  if (entry.kind === "new") requireTrue(!exists(entry.path), `new patch path already exists: ${entry.path}`);
}

const requiredFindings = [
  "F2-WIRE-001", "F2-WIRE-002", "F2-WIRE-003", "F2-WIRE-004",
  "F2-WIRE-005", "F2-WIRE-006", "F2-WIRE-007", "F2-WIRE-008",
  "F2-WIRE-009", "F2-WIRE-010", "F2-WIRE-011", "F2-WIRE-012"
];
for (const id of requiredFindings) requireTrue(inventory.wiringFindings.some((finding) => finding.id === id), `missing finding: ${id}`);
for (const id of ["F1", "F2", "F3", "F4", "F5", "F6", "F7"]) requireTrue(inventory.dependencies.some((dep) => dep.id === id), `missing dependency: ${id}`);

requireTrue(inventory.snapshotContract.protocol === "continuous-ipc", "snapshot protocol mismatch");
requireTrue(inventory.snapshotContract.version === 1, "snapshot version mismatch");
requireTrue(inventory.snapshotContract.formalWorldOwner === false, "snapshot must not claim formal owner");
requireTrue(inventory.snapshotContract.frontendConnected === false, "frontend must not claim WorldView connection");
requireTrue(inventory.snapshotContract.writeRule.includes("Rust owner only"), "snapshot write rule must remain Rust-only");

const forbidden = ["TP-" + "15C3", "TP-" + "18D1", "TP-" + "01R11", "TP-" + "18D2", "NB-55"];
for (const token of forbidden) {
  requireTrue(!doc.includes(token), `document contains excluded token: ${token}`);
  requireTrue(!JSON.stringify(inventory).includes(token), `inventory contains excluded token: ${token}`);
}
requireTrue(doc.includes("不读取媒体或资产文件内容"), "document must state media/assets are not read");
requireTrue(doc.includes("不执行构建、测试、运行时"), "document must state no execution was performed");
requireTrue(doc.includes("不得静默 fallback"), "document must state no silent fallback");
requireTrue(doc.includes("requiresSplit=false"), "document must state patch split decision");

const result = {
  schemaVersion: 1,
  inventoryId: "NB-56",
  status: errors.length === 0 ? "pass_gate_blocked" : "fail",
  gate: "E_WORLD_V3_NOT_RUNTIME_WIRED",
  worldV3RuntimeWired: false,
  greyHiveRuntimeLoaded: false,
  minimalPatchApproved: false,
  sourceModified: false,
  buildExecuted: false,
  runtimeExecuted: false,
  mediaRead: false,
  networkAccessed: false,
  logicalUnitCount: patch.logicalUnitCount,
  requiresSplit: patch.requiresSplit,
  checkedAt: new Date().toISOString(),
  errors
};
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
if (errors.length) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));

