import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const A_COMMIT = "ac89e851d6911ce4b9a5967e3c17d7c3c3e7cac1";
const ACTION_PATH = "tools/scenario-runner/fixtures/grey-hive-power-gate-a-v1.json";
const REPLAY_PATH = "artifacts/acceptance/pivot-scenario-runner-01/runs/run-31024-1790319951058508300-0/scenario.jsonl";
const EXPECTED_ACTION_SHA = "bc830cb31d61e7eb04bc1580e08721db47f3dcb6e56ae49a32469405e27e1edd";
const EXPECTED_REPLAY_SHA = "ebcc833d0f476cf47a58a38c3519ebe0bb5dbc173692c96005512440a33258df";
const here = path.dirname(fileURLToPath(import.meta.url));
const fixturePath = path.resolve(here, "../fixtures/scenario-runner-presentation.json");
const metadataPath = path.resolve(here, "../fixtures/scenario-runner-presentation.meta.json");

const blob = (repoPath) => execFileSync("git", ["show", `${A_COMMIT}:${repoPath}`], { encoding: "buffer" });
const sha256 = bytes => createHash("sha256").update(bytes).digest("hex");
const actionBytes = blob(ACTION_PATH);
const replayBytes = blob(REPLAY_PATH);
if (sha256(actionBytes) !== EXPECTED_ACTION_SHA) throw new Error("E_SCENARIO_ACTION_SHA_MISMATCH");
if (sha256(replayBytes) !== EXPECTED_REPLAY_SHA) throw new Error("E_SCENARIO_REPLAY_SHA_MISMATCH");

const action = JSON.parse(actionBytes.toString("utf8"));
const rows = replayBytes.toString("utf8").trimEnd().split(/\r?\n/).map(JSON.parse);
const select = (phase, event) => {
  const matches = rows.map((row, index) => ({ row, index })).filter(item => item.row.phase === phase && item.row.event === event);
  if (matches.length !== 1) throw new Error(`E_SCENARIO_ROW_NOT_UNIQUE:${phase}:${event}:${matches.length}`);
  return matches[0];
};
const selected = {
  hub: select("seed", "initial"),
  powerBefore: select("seed", "initial"),
  gateBefore: select("seed", "closed-gate-blocked"),
  powerAfter: select("seed", "power-applied"),
  gateAfter: select("seed", "open-gate-passed")
};
for (const [name, selection] of Object.entries(selected)) {
  if (!selection.row.worldView || !selection.row.worldSnapshot) throw new Error(`E_SCENARIO_STATE_MISSING:${name}`);
  const view = selection.row.worldView;
  const snapshotView = selection.row.worldSnapshot.view;
  if (JSON.stringify(view) !== JSON.stringify(snapshotView)) throw new Error(`E_SCENARIO_SNAPSHOT_VIEW_MISMATCH:${name}`);
}

const initial = selected.hub.row;
const powered = selected.powerAfter.row;
if (!initial.commandReceipt || !initial.worldSnapshot) throw new Error("E_SCENARIO_NEW_RECEIPT_MISSING");
if (!powered.commandReceipt || powered.commandReceipt.applied !== true) throw new Error("E_SCENARIO_POWER_RECEIPT_MISSING");
if (selected.gateBefore.row.worldView.doors?.find(door => door.doorId === action.gateDoorId)?.open !== false) throw new Error("E_SCENARIO_GATE_BEFORE_INVALID");
if (selected.gateAfter.row.worldView.doors?.find(door => door.doorId === action.gateDoorId)?.open !== true) throw new Error("E_SCENARIO_GATE_AFTER_INVALID");
if (selected.powerAfter.row.worldView.actors?.find(actor => actor.entityId === action.consoleActorId)?.active !== true) throw new Error("E_SCENARIO_POWER_ACTOR_MISSING");

function worldSnapshot(view) {
  return {
    kind: "full",
    schemaVersion: view.schemaVersion,
    worldId: view.worldId,
    worldEpoch: view.worldEpoch,
    serverTick: view.serverTick,
    authorityRevision: view.authorityRevision,
    ackSeq: view.ackSeq,
    view
  };
}

function receiptFromRecordedView(view, commandId) {
  return {
    commandId,
    applied: true,
    alreadyApplied: false,
    errorCode: null,
    worldEpoch: view.worldEpoch,
    serverTick: view.serverTick,
    authorityRevision: view.authorityRevision,
    snapshot: worldSnapshot(view)
  };
}

const fixture = {
  schema: "pivot-presentation-scenario-fixture/1",
  scenarioId: action.scenarioId,
  worldId: action.worldId,
  scenarios: Object.fromEntries(Object.entries(selected).map(([name, selection]) => [name, {
    sourcePhase: selection.row.phase,
    sourceEvent: selection.row.event,
    sourceLineIndex: selection.index,
    view: selection.row.worldView,
    worldSnapshot: selection.row.worldSnapshot,
    ...(selection.row.commandReceipt ? { commandReceipt: selection.row.commandReceipt } : {})
  }])),
  ipc: {
    formalSnapshot: initial.worldSnapshot,
    formalListSaveSlots: [],
    formalNew: initial.commandReceipt,
    formalInteractPower: {
      applied: powered.commandReceipt.applied,
      alreadyApplied: powered.commandReceipt.alreadyApplied,
      errorCode: powered.commandReceipt.errorCode,
      events: ["hive_power"],
      receipt: powered.commandReceipt,
      view: powered.worldView
    },
    formalSubmitInput: {
      powerBefore: receiptFromRecordedView(selected.powerBefore.row.worldView, "scenario-input-initial"),
      powerAfter: receiptFromRecordedView(selected.powerAfter.row.worldView, "scenario-input-power-applied"),
      gateBefore: receiptFromRecordedView(selected.gateBefore.row.worldView, "scenario-input-closed-gate-blocked"),
      gateAfter: receiptFromRecordedView(selected.gateAfter.row.worldView, "scenario-input-open-gate-passed")
    }
  }
};

const fixtureBytes = Buffer.from(`${JSON.stringify(fixture, null, 2)}\n`, "utf8");
const metadata = {
  sourceCommit: A_COMMIT,
  sourcePath: REPLAY_PATH,
  fixtureSha256: sha256(fixtureBytes),
  sources: [
    { path: ACTION_PATH, sha256: EXPECTED_ACTION_SHA },
    { path: REPLAY_PATH, sha256: EXPECTED_REPLAY_SHA }
  ],
  selectedRows: Object.fromEntries(Object.entries(selected).map(([name, selection]) => [name, {
    lineIndexZeroBased: selection.index,
    phase: selection.row.phase,
    event: selection.row.event,
    worldSnapshotCopied: true,
    worldViewCopied: true
  }])),
  selectionRules: [
    "hub and powerBefore use the seed/initial WorldView and WorldSnapshot (closed Gate A, console actor present).",
    "gateBefore uses seed/closed-gate-blocked (closed Gate A at the approach limit).",
    "powerAfter uses seed/power-applied and its actual applied CommandReceipt.",
    "gateAfter uses seed/open-gate-passed (open Gate A after passage).",
    "gateBefore/gateAfter input CommandReceipts are deterministic CommandReceipt.from_view-shaped envelopes made from the exact recorded WorldViews because those JSONL rows do not include commandReceipt objects.",
    "The initial slot-list response is an empty array because the Scenario Runner JSONL does not record formal_list_save_slots; slot UI is not part of these screenshots.",
    "The F interaction is replayed as the committed power-applied response; the omitted movement steps between selected snapshots mean this browser harness is visual protocol replay, not live interaction-distance acceptance."
  ]
};

await mkdir(path.dirname(fixturePath), { recursive: true });
await writeFile(fixturePath, fixtureBytes);
await writeFile(metadataPath, `${JSON.stringify(metadata, null, 2)}\n`, "utf8");
console.log(JSON.stringify({ fixturePath, metadataPath, fixtureSha256: metadata.fixtureSha256, sourceCommit: A_COMMIT, selectedRows: metadata.selectedRows }, null, 2));
