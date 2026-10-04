import { scannerRewardFeedback } from "../dist/game/ScannerReward.js";
import test from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import { isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback, interactionErrorText } from "../dist/game/SceneInteraction.js";
import { confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks } from "../dist/game/ClockworksCampaign.js";

const compiled = await readFile(new URL("../dist/main.js", import.meta.url), "utf8");
const start = compiled.indexOf("async function interactFromSnapshot(");
const end = compiled.indexOf("\nasync function begin(", start);
assert.ok(start >= 0 && end > start);
const handler = compiled.slice(start, end);
const flush = () => new Promise(resolve => setImmediate(resolve));
function deferred() { let resolve, reject; const promise = new Promise((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; }
function snapshot(completed = false, overrides = {}) {
  return { kind: "full", protocolVersion: 3, worldId: "clockworks", sceneId: "cw_shutdown_exit",
    worldEpoch: 31, serverTick: 700, authorityRevision: completed ? 91 : 90,
    player: { currentHp: 17 }, capabilities: { items: completed ? [{ capabilityId: "mobility.air_step_i", granted: true }] : [] },
    interactables: [{ entityId: "cw_master_shutdown_staged", kind: "terminal", active: true }],
    progression: { currentWorldId: "clockworks", eventSeq: completed ? 12 : 10, worlds: [{
      worldId: "clockworks", completed, firstCompletion: completed, visitId: 1, revisitCount: 0,
      completedEvents: completed ? ["clockworks_valves", "clockworks_core", "clockworks_shutdown"] : ["clockworks_valves", "clockworks_core"],
    }] }, ...overrides };
}
function harness({ dispatchDeferred = false, acceptanceDeferred = false, navigation = false } = {}) {
  const dispatch = deferred(), acceptance = deferred(), feedback = [];
  const before = snapshot(navigation);
  if (navigation) before.interactables = [{ entityId: "cw_shutdown_return_to_rs", kind: "world_gate", active: true }];
  const after = navigation ? snapshot(true, { worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 32,
    authorityRevision: 92, entryToken: { generation: 9, worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 32 } }) : snapshot(true);
  const result = { applied: true, alreadyApplied: false, errorCode: null, snapshot: after };
  const loop = {
    current: before, sceneEntrySequence: 7, pausePresentationState: "running", acceptsExternalResults: true, isDead: false,
    snapshots: { view() { return loop.current; } },
    async acceptAuthoritativeSnapshot(value) {
      loop.current = value;
      if (value.entryToken) { loop.sceneEntrySequence++; loop.pausePresentationState = "loading"; }
      if (acceptanceDeferred) await acceptance.promise;
      if (loop.current === value) { loop.current = { ...value, entryToken: undefined }; loop.pausePresentationState = "running"; }
    },
  };
  const context = vm.createContext({
    Error, sessionLoop: loop, interactionBusy: false, client: {},
    hud: { apply: value => ({ interactionId: value.interactables[0].entityId }), setFeedback: value => feedback.push(value) },
    dispatchInteractable: () => dispatchDeferred ? dispatch.promise : Promise.resolve(result),
    isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback, interactionErrorText,
    scannerRewardFeedback, confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks,
    confirmedGreyHiveNarrative: () => null, confirmedMistHarborBeaconSync: () => null,
    confirmedMistHarborAcousticMappingLine: () => null, confirmedReturnStationAfterGreyHive: () => null,
    missionTerminalSummary: () => "mission", capabilityTerminalSummary: () => "capability",
  });
  vm.runInContext(handler, context);
  return { context, loop, before, after, result, dispatch, acceptance, feedback,
    run: () => context.interactFromSnapshot(before) };
}

test("exact compiled handler shows fresh CW epilogue and source errors", async () => {
  const normal = harness(); await normal.run();
  assert.equal(normal.feedback.at(-1), "原来门一直不止三扇。");
  const failed = harness({ dispatchDeferred: true }); const pending = failed.run();
  failed.dispatch.reject(new Error("E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED")); await pending;
  assert.match(failed.feedback.at(-1), /先击败主调节者/);
});

test("old CW epilogue cannot cross same-loop scene or entry-generation replacement during acceptance", async () => {
  for (const sameIdentity of [false, true]) {
    const h = harness({ acceptanceDeferred: true }); const pending = h.run(); await flush();
    h.loop.current = sameIdentity ? { ...h.after } : snapshot(true, { worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 32,
      entryToken: { generation: 9 } });
    h.loop.sceneEntrySequence++;
    h.loop.pausePresentationState = sameIdentity ? "running" : "loading";
    h.acceptance.resolve(); await pending;
    assert.ok(!h.feedback.includes("原来门一直不止三扇。"));
  }
});

test("late dispatch error stays with source scene and original entry generation", async () => {
  for (const changed of ["scene", "generation", "loading", "dead"]) {
    const h = harness({ dispatchDeferred: true }); const pending = h.run();
    if (changed === "scene") h.loop.current = snapshot(true, { worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 32 });
    if (changed === "generation") h.loop.sceneEntrySequence++;
    if (changed === "loading") h.loop.pausePresentationState = "loading";
    if (changed === "dead") { h.loop.current = snapshot(true, { player: { currentHp: 0 } }); h.loop.acceptsExternalResults = false; }
    h.dispatch.reject(new Error("E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED")); await pending;
    assert.ok(!h.feedback.some(value => value.includes("先击败主调节者")), changed);
  }
});

test("destination acceptance errors remain visible only for their current ready destination", async () => {
  const current = harness({ acceptanceDeferred: true, navigation: true }); const pending = current.run(); await flush();
  current.loop.current = { ...current.after, entryToken: undefined };
  current.loop.pausePresentationState = "running";
  current.acceptance.reject(new Error("E_DESTINATION_RECEIPT")); await pending;
  assert.match(current.feedback.at(-1), /E_DESTINATION_RECEIPT/);
  const replaced = harness({ acceptanceDeferred: true, navigation: true }); const old = replaced.run(); await flush();
  replaced.loop.current = snapshot(false, { worldEpoch: 33, sceneId: "cw_entry_foundry" });
  replaced.loop.sceneEntrySequence++;
  replaced.loop.pausePresentationState = "running";
  replaced.acceptance.reject(new Error("E_DESTINATION_RECEIPT")); await old;
  assert.ok(!replaced.feedback.some(value => value.includes("E_DESTINATION_RECEIPT")));
});

test("a ready accepted destination presents its first CW return line", async () => {
  const h = harness({ acceptanceDeferred: true, navigation: true }); const pending = h.run(); await flush();
  h.acceptance.resolve(); await pending;
  assert.equal(h.feedback.at(-1), "三处坐标不是孤立事件。网络中仍存在未识别节点。");
});

test("old session completion cannot clear a newer session's busy guard", async () => {
  const h = harness({ dispatchDeferred: true }); const old = h.run();
  h.context.sessionLoop = { ...h.loop };
  h.context.interactionBusy = true;
  h.dispatch.resolve(h.result); await old;
  assert.equal(h.context.interactionBusy, true);
});
