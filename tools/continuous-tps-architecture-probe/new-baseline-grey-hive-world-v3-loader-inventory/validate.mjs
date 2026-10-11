import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../../..");
const inventoryPath = path.join(import.meta.dirname, "inventory.json");
const docPath = path.join(root, "docs/GREY_HIVE_WORLD_V3_LOADER_CODE_INVENTORY.md");
const resultPath = path.join(root, "tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-world-v3-loader-inventory-nb62.json");

const errors = [];
const exists = (file) => fs.existsSync(path.join(root, file));
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const requireTrue = (condition, message) => { if (!condition) errors.push(message); };

const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const doc = fs.readFileSync(docPath, "utf8");

const checkEntry = (entry) => {
  if (entry.classification === "missing" || entry.exists === false || entry.kind === "new") {
    requireTrue(!exists(entry.path), "missing/new entry unexpectedly exists: " + entry.path);
    return;
  }
  requireTrue(exists(entry.path), "missing path: " + entry.path);
  if (entry.anchor && exists(entry.path)) {
    requireTrue(read(entry.path).includes(entry.anchor), "missing anchor " + entry.anchor + " in " + entry.path);
  }
};
for (const entry of [...inventory.dataSources, ...inventory.loaderEntries, ...inventory.bootstrapEntries]) checkEntry(entry);
for (const dep of inventory.dependencies) requireTrue(exists(dep.path), "missing dependency path: " + dep.path);

const entryIds = [
  ...inventory.dataSources.map((x) => x.id),
  ...inventory.loaderEntries.map((x) => x.id),
  ...inventory.bootstrapEntries.map((x) => x.id),
  ...inventory.contractFields.map((x) => x.id),
  ...inventory.contractViolations.map((x) => x.id),
  ...inventory.referenceChecks.map((x) => x.id)
];
requireTrue(new Set(entryIds).size === entryIds.length, "inventory IDs must be unique");
requireTrue(inventory.schemaVersion === 1, "schemaVersion must be 1");
requireTrue(inventory.inventoryId === "NB-62", "inventoryId must be NB-62");
requireTrue(inventory.status === "grey_hive_world_v3_loader_inventory_complete_gate_blocked", "status mismatch");

const fixturePath = "tools/continuous-tps-architecture-probe/new-baseline-grey-hive-data-contract/runtime-fixture.json";
const fixture = JSON.parse(read(fixturePath));
const staticData = fixture.static;
const source = fixture.source;
requireTrue(fixture.schemaVersion === "grey_hive_runtime_data_contract_v1", "fixture schema mismatch");
requireTrue(fixture.worldId === "grey_hive", "fixture worldId mismatch");
requireTrue(source.candidateOnly === true, "fixture must remain candidateOnly");
requireTrue(fixture.dynamic.owner === "Rust", "dynamic owner must be Rust");
requireTrue(fixture.dynamic.choice === "unresolved", "hive_choice must be unresolved");
requireTrue(fixture.dynamic.npc.isGatePrerequisite === false, "NPC must not gate");
requireTrue(fixture.dynamic.flags.hive_lockdown === false, "candidate default lockdown must be false");

const counts = {
  rooms: staticData.rooms.length,
  connectors: staticData.connectors.length,
  doors: staticData.doors.length,
  triggers: staticData.triggers.length,
  checkpoints: staticData.checkpoints.length
};
const expected = inventory.recordCounts.baseline;
for (const key of Object.keys(expected)) {
  requireTrue(counts[key] === expected[key], "observed count mismatch for " + key);
  requireTrue(inventory.recordCounts.observed[key] === counts[key], "inventory observed count mismatch for " + key);
}
requireTrue(Array.isArray(inventory.recordCounts.mismatches) && inventory.recordCounts.mismatches.length === 0, "record count mismatches must be explicit and empty");

const namespaces = [staticData.rooms, staticData.connectors, staticData.doors, staticData.triggers, staticData.checkpoints];
for (const records of namespaces) requireTrue(new Set(records.map((x) => x.id)).size === records.length, "duplicate stable ID in candidate namespace");

const rooms = new Set(staticData.rooms.map((x) => x.id));
const doors = new Set(staticData.doors.map((x) => x.id));
const triggers = new Set(staticData.triggers.map((x) => x.id));
const connectors = new Set(staticData.connectors.map((x) => x.id));
let dangling = 0;
for (const zone of staticData.zones) for (const roomId of zone.roomIds || []) if (!rooms.has(roomId)) dangling += 1;
for (const connector of staticData.connectors) {
  if (!rooms.has(connector.from) || !rooms.has(connector.to)) dangling += 1;
  if (!doors.has(connector.doorId)) dangling += 1;
  if (connector.triggerId && !triggers.has(connector.triggerId)) dangling += 1;
}
for (const door of staticData.doors) if (!connectors.has(door.connectorId)) dangling += 1;
for (const trigger of staticData.triggers) if (!rooms.has(trigger.roomId)) dangling += 1;
for (const checkpoint of staticData.checkpoints) if (!rooms.has(checkpoint.roomId)) dangling += 1;
requireTrue(dangling === 0, "candidate reference graph is not closed");
requireTrue(inventory.referenceChecks.find((x) => x.id === "REF-008").status === "not_accepted", "runtime acceptance must be not_accepted");
requireTrue(inventory.currentDecision.referencesAccepted === true, "static referencesAccepted should be true");
requireTrue(inventory.currentDecision.referencesAcceptedScope === "static_candidate_only", "references scope must be static_candidate_only");

const requiredFlags = {
  greyHiveRuntimeLoaded: false,
  greyHiveLoaderImplemented: false,
  greyHiveDataApproved: false,
  defaultEnabled: false,
  minimalPatchApproved: false,
  sourceModified: false,
  dataModified: false,
  implementationExecuted: false,
  buildExecuted: false,
  testsExecuted: false,
  runtimeExecuted: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false
};
for (const [key, value] of Object.entries(requiredFlags)) {
  requireTrue(inventory.currentDecision[key] === value, key + " must be " + value);
  if (key in inventory.scanMetadata) requireTrue(inventory.scanMetadata[key] === value, "scanMetadata." + key + " must be " + value);
}

requireTrue(inventory.minimalPatchBoundary.logicalUnitCount === inventory.minimalPatchBoundary.candidates.length, "patch count mismatch");
requireTrue(inventory.minimalPatchBoundary.logicalUnitCount <= 8 || inventory.minimalPatchBoundary.requires_split === true, "patch must be <=8 or require split");
requireTrue(inventory.minimalPatchBoundary.approved === false, "patch must remain unapproved");
for (const candidate of inventory.minimalPatchBoundary.candidates) {
  if (candidate.kind === "new") requireTrue(!exists(candidate.path), "new patch candidate exists: " + candidate.path);
  else requireTrue(exists(candidate.path), "existing patch candidate missing: " + candidate.path);
}
requireTrue(inventory.rollback.noGridFallback === true, "rollback must prohibit Grid fallback");
requireTrue(inventory.rollback.noPartialWorld === true, "rollback must prohibit partial World");
requireTrue(inventory.rollback.noSaveOverwrite === true, "rollback must prohibit Save overwrite");

const forbidden = [
  "TP-" + "15C3",
  "TP-" + "18D1",
  "TP-" + "01R11",
  "TP-" + "18D2",
  "NB-" + "47",
  "NB-" + "48",
  "NB-" + "61"
];
for (const token of forbidden) {
  requireTrue(!doc.includes(token), "document contains excluded token: " + token);
  requireTrue(!JSON.stringify(inventory).includes(token), "inventory contains excluded token: " + token);
}
requireTrue(doc.includes("不读取资产或媒体"), "document must state no asset/media read");
requireTrue(doc.includes("不执行构建、测试或运行时"), "document must state no execution");
requireTrue(doc.includes("不回退 Grid"), "document must state no Grid fallback");
requireTrue(doc.includes("Gate B：唯一 prerequisite"), "document must state Gate B policy");
requireTrue(doc.includes("hive_choice：仅 taken|left|unresolved"), "document must state choice states");
requireTrue(doc.includes("候选共 6 个逻辑单元"), "document must state patch count");

const result = {
  schemaVersion: 1,
  inventoryId: "NB-62",
  status: errors.length === 0 ? "pass_gate_blocked" : "fail",
  greyHiveRuntimeLoaded: false,
  greyHiveLoaderImplemented: false,
  greyHiveDataApproved: false,
  defaultEnabled: false,
  referencesAccepted: true,
  referencesAcceptedScope: "static_candidate_only",
  minimalPatchApproved: false,
  sourceModified: false,
  dataModified: false,
  buildExecuted: false,
  testsExecuted: false,
  runtimeExecuted: false,
  assetBytesRead: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  baselineCounts: expected,
  observedCounts: counts,
  danglingReferences: dangling,
  logicalUnitCount: inventory.minimalPatchBoundary.logicalUnitCount,
  requires_split: inventory.minimalPatchBoundary.requires_split,
  checkedAt: new Date().toISOString(),
  errors
};
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
if (errors.length) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));


