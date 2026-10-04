import test from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import { TauriClient } from "../dist/bridge/tauri-client.js";
import { scannerRewardFeedback, scannerTerminalLabel } from "../dist/game/ScannerReward.js";
import { deriveHudState } from "../dist/ui/Hud.js";
import { dispatchInteractable, interactionErrorText, isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback } from "../dist/game/SceneInteraction.js";
import { confirmedGreyHiveNarrative } from "../dist/game/GreyHiveNarrative.js";
const compiled = await readFile(new URL("../dist/main.js", import.meta.url), "utf8");
const start = compiled.indexOf("async function interactFromSnapshot(");
const end = compiled.indexOf("\nasync function begin(", start);
assert.ok(start >= 0 && end > start);
const handler = compiled.slice(start, end);
const flush = () => new Promise(resolve => setImmediate(resolve));
function deferred() { let resolve, reject; const promise = new Promise((y, n) => { resolve = y; reject = n; }); return { promise, resolve, reject }; }
const cap = "information.enemy_vitals_basic";
function snapshot(granted = false, overrides = {}) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2", worldId: "grey_hive", sceneId: "gh_central_shaft", checkpointId: null,
    worldEpoch: 4, serverTick: 70, authorityRevision: granted ? 10 : 9, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 3, yM: 0, zM: 8 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [{ capabilityId: cap, granted, selected: false }], enemyVitals: [] },
    interactables: [{ entityId: "gh_log_shaft_01", kind: "facility_log", active: true,
      transform: { positionM: { xM: 3, yM: 0, zM: 8 }, yawRad: 0 } }],
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 3,
      worlds: [{ worldId: "grey_hive", completed: false, firstCompletion: false, visitId: 1, cycleId: 1, revisitCount: 0, completedEvents: ["hive_power"] }] }, ...overrides };
}
function harness({ online = false, acceptanceDeferred = false, failure = null } = {}) {
  const before = snapshot(online), after = snapshot(true), feedback = [], calls = [], acceptance = deferred();
  const result = { applied: !online, alreadyApplied: online, errorCode: null, snapshot: after };
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args }); assert.equal(command, "formal_interact");
    if (failure) throw new Error(failure);
    return { ...result, receipt: { ...result, commandId: args.requestId,
      worldEpoch: after.worldEpoch, serverTick: after.serverTick, authorityRevision: after.authorityRevision } };
  });
  const loop = { current: before, sceneEntrySequence: 7, pausePresentationState: "running", acceptsExternalResults: true, isDead: false,
    snapshots: { view() { return loop.current; } },
    async acceptAuthoritativeSnapshot(value) { loop.current = value; if (acceptanceDeferred) await acceptance.promise; } };
  const context = vm.createContext({ Error, sessionLoop: loop, interactionBusy: false, client,
    hud: { apply: deriveHudState, setFeedback: value => feedback.push(value) },
    dispatchInteractable, scannerRewardFeedback, isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback, interactionErrorText,
    confirmedGreyHiveNarrative, confirmedMistHarborBeaconSync: () => null, confirmedMistHarborAcousticMappingLine: () => null,
    confirmedClockworksEpilogue: () => null, confirmedReturnStationAfterGreyHive: () => null, confirmedReturnStationAfterClockworks: () => null,
  });
  vm.runInContext(handler, context);
  return { context, loop, before, after, result, calls, acceptance, feedback, run: () => context.interactFromSnapshot(before) };
}

test("Scanner exact source HUD and compiled F handler use formal_interact with source epoch", async () => {
  const h = harness(); assert.equal(deriveHudState(h.before).interactionText, "F 交互 · 恢复扫描模块");
  await h.run(); assert.equal(h.feedback.at(-1), "扫描模块已恢复，敌人生命状态信息已启用。");
  assert.equal(h.calls.length, 1); assert.equal(h.calls[0].command, "formal_interact");
  assert.equal(h.calls[0].args.actorId, "gh_log_shaft_01"); assert.equal(h.calls[0].args.worldEpoch, 4);
  assert.deepEqual(Object.keys(h.calls[0].args).sort(), ["actorId", "requestId", "worldEpoch"]);
  assert.equal(deriveHudState(h.after).interactionText, "F 交互 · 扫描模块已在线");
});

test("Scanner already-applied receipt only shows online and keeps old log narrative unaffected", async () => {
  const h = harness({ online: true }); await h.run(); assert.equal(h.feedback.at(-1), "扫描模块已在线。");
  assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_sentinel_arena", "gh_sys_sentinel_01", "facility_log", true), "自动防卫单元已接管本区。");
  const before = snapshot(); const item = before.interactables[0];
  for (const bad of [{ ...item, entityId: "other" }, { ...item, kind: "terminal" }]) {
    assert.equal(scannerRewardFeedback(before, bad, h.result), null); assert.equal(scannerTerminalLabel(before, bad), null);
  }
  for (const change of [{ worldId: "mist_harbor" }, { sceneId: "gh_sentinel_arena" }]) {
    assert.equal(scannerRewardFeedback({ ...before, ...change }, item, h.result), null);
    assert.equal(scannerTerminalLabel({ ...before, ...change }, item), null);
  }
});

test("Scanner failed or ungranted receipts never claim acquisition", async () => {
  const before = snapshot(); const item = before.interactables[0];
  for (const result of [
    { applied: false, alreadyApplied: false, errorCode: "E_SCANNER_POWER_REQUIRED", snapshot: snapshot() },
    { applied: true, errorCode: null, snapshot: snapshot() },
    { applied: true, errorCode: "E_SAVE_TEMP", snapshot: snapshot(true) },
    { applied: true, errorCode: null, snapshot: snapshot(true, { worldEpoch: 5 }) },
  ]) assert.equal(scannerRewardFeedback(before, item, result), null);
  const failed = harness({ failure: "E_SAVE_TEMP: permission denied" }); await failed.run();
  assert.match(failed.feedback.at(-1), /E_SAVE_TEMP/); assert.ok(!failed.feedback.some(value => value.includes("已恢复")));
  assert.match(interactionErrorText("E_SCANNER_POWER_REQUIRED"), /先恢复灰巢供电/);
});

test("Scanner pending F is single-flight and late reward cannot cross session/scene/lifecycle replacement", async () => {
  for (const replacement of ["session", "scene", "generation", "loading", "dead"]) {
    const h = harness({ acceptanceDeferred: true }); const pending = h.run(); await flush();
    await h.run(); assert.equal(h.calls.length, 1);
    if (replacement === "session") h.context.sessionLoop = { ...h.loop };
    if (replacement === "scene") h.loop.current = snapshot(true, { sceneId: "gh_gate_a", worldEpoch: 5 });
    if (replacement === "generation") h.loop.sceneEntrySequence++;
    if (replacement === "loading") h.loop.pausePresentationState = "loading";
    if (replacement === "dead") { h.loop.current = { ...h.after, player: { ...h.after.player, currentHp: 0 } }; h.loop.acceptsExternalResults = false; }
    h.acceptance.resolve(); await pending; assert.ok(!h.feedback.some(value => value.includes("已恢复")), replacement);
  }
});

test("Scanner prompt honors exact 2.5m range, authoritative inactive, and death", () => {
  const value = snapshot(); value.player.transform.positionM.xM = 5.5;
  assert.equal(deriveHudState(value).interactionId, "gh_log_shaft_01");
  value.player.transform.positionM.xM = 5.501; assert.equal(deriveHudState(value).interactionId, null);
  value.player.transform.positionM.xM = 3; value.interactables[0].active = false; assert.equal(deriveHudState(value).interactionId, null);
  value.interactables[0].active = true; value.player.currentHp = 0; assert.equal(deriveHudState(value).interactionId, null);
});
