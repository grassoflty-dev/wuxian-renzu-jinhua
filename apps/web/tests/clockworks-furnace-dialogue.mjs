import test from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import { TauriClient } from "../dist/bridge/tauri-client.js";
import { deriveHudState } from "../dist/ui/Hud.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { confirmedClockworksFurnaceDialogue } from "../dist/game/ClockworksFurnaceDialogue.js";
import { dispatchInteractable, interactionErrorText, isCurrentSceneInteractionResult,
  isCurrentSceneInteractionFeedback } from "../dist/game/SceneInteraction.js";
import { scannerRewardFeedback } from "../dist/game/ScannerReward.js";
import { confirmedGreyHiveNarrative } from "../dist/game/GreyHiveNarrative.js";
import { confirmedMistHarborAcousticMappingLine, confirmedMistHarborBeaconSync } from "../dist/game/MistHarborNarrative.js";
import { confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks } from "../dist/game/ClockworksCampaign.js";
import { confirmedReturnStationAfterGreyHive, confirmedReturnStationAfterMistHarbor } from "../dist/game/ReturnStationNarrative.js";

const ID = "cw_cy_heat_01_static_dialogue_marker";
const LINE = "炉心不是失控——它被维持在过载边缘。";
const KIND = "static_dialogue_marker";
const compiled = await readFile(new URL("../dist/main.js", import.meta.url), "utf8");
const start = compiled.indexOf("async function interactFromSnapshot("), end = compiled.indexOf("\nasync function begin(", start);
assert.ok(start >= 0 && end > start);
const handler = compiled.slice(start, end);
const flush = () => new Promise(resolve => setImmediate(resolve));
function deferred() { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; }
const position = (xM = 12, yM = 0, zM = 10) => ({ xM, yM, zM });
const target = (overrides = {}) => ({ entityId: ID, kind: KIND, active: true,
  transform: { positionM: position(), yawRad: 0 }, ...overrides });
function snapshot(active = true, overrides = {}) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "clockworks", sceneId: "cw_furnace_heart", checkpointId: null,
    worldEpoch: 8, serverTick: 70, authorityRevision: active ? 9 : 10, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: position(), yawRad: 0 },
      velocityMps: position(0, 0, 0), currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    interactables: [target({ active })], actors: [], doors: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "clockworks", eventSeq: 3, worlds: [] }, ...overrides };
}
const receipt = (overrides = {}) => ({ applied: true, alreadyApplied: false, errorCode: null, snapshot: snapshot(false), ...overrides });
function harness({ before = snapshot(), result = receipt(), dispatchDeferred = false, acceptanceDeferred = false } = {}) {
  const dispatch = deferred(), acceptance = deferred(), feedback = [], calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args }); assert.equal(command, "formal_interact");
    const response = dispatchDeferred ? await dispatch.promise : result;
    return { ...response, receipt: { ...response, commandId: args.requestId,
      worldEpoch: response.snapshot.worldEpoch, serverTick: response.snapshot.serverTick,
      authorityRevision: response.snapshot.authorityRevision } };
  });
  const loop = { current: before, sceneEntrySequence: 7, pausePresentationState: "running", acceptsExternalResults: true, isDead: false,
    snapshots: { view: () => loop.current },
    async acceptAuthoritativeSnapshot(value) { loop.current = value; if (acceptanceDeferred) await acceptance.promise; } };
  const context = vm.createContext({ Error, sessionLoop: loop, interactionBusy: false, client,
    hud: { apply: deriveHudState, setFeedback: value => feedback.push(value) },
    dispatchInteractable, interactionErrorText, isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback,
    confirmedClockworksFurnaceDialogue, scannerRewardFeedback, confirmedGreyHiveNarrative,
    confirmedMistHarborAcousticMappingLine, confirmedMistHarborBeaconSync, confirmedClockworksEpilogue,
    confirmedReturnStationAfterClockworks, confirmedReturnStationAfterGreyHive, confirmedReturnStationAfterMistHarbor,
  });
  vm.runInContext(handler, context);
  return { before, result, context, loop, calls, feedback, dispatch, acceptance,
    run: (source = loop.current) => context.interactFromSnapshot(source), count: () => feedback.filter(value => value === LINE).length };
}

test("the frozen Furnace line matches authored content without adding runtime dialogue fields", async () => {
  const scene = JSON.parse(await readFile(new URL("../../../content/scenes/compiled/cw_furnace_heart.json", import.meta.url), "utf8"));
  const marker = scene.interactions.find(item => item.id === ID);
  assert.deepEqual(marker, { event: null, id: ID, kind: KIND, position: [12, 0, 10] });
  const catalog = JSON.parse(await readFile(new URL("../../../content/dialogue/clockworks_narrative_catalog_zh-CN_v1.json", import.meta.url), "utf8"));
  assert.equal(catalog.entries.find(item => item.id === "cw_cy_heat_01").body, LINE);
});

test("real HUD uses exact identity, active projection and inclusive 2.5m 3D distance", () => {
  for (const pos of [position(), position(14.5), position(12, 2.5)]) {
    const value = snapshot(); value.player.transform.positionM = pos;
    assert.equal(deriveHudState(value).interactionId, ID);
    assert.equal(deriveHudState(value).interactionText, "[F] 查看炉心状态");
  }
  for (const mutate of [s => { s.worldId = "grey_hive"; }, s => { s.sceneId = "cw_entry_foundry"; },
    s => { s.interactables[0].entityId = "cw_sys_entry_static_dialogue_marker"; },
    s => { s.interactables[0].kind = "terminal"; }, s => { s.interactables[0].active = false; },
    s => { s.interactables[0].active = "true"; }, s => { s.interactables.push(target()); },
    s => { s.player.currentHp = 0; }, s => { s.player.transform.positionM = position(14.5001); },
    s => { s.player.transform.positionM = position(12, 2.5001); },
    s => { s.player.transform.positionM.xM = "12"; }, s => { s.interactables[0].transform.positionM.xM = NaN; }]) {
    const value = snapshot(); mutate(value);
    assert.equal(deriveHudState(value).interactionId, null); assert.equal(deriveHudState(value).interactionText, "");
  }
});

test("dispatcher rejects every unrelated static marker and forged Furnace source or kind", async () => {
  const calls = [], client = { interact: async (...args) => { calls.push(args); return receipt(); } };
  await dispatchInteractable(client, target(), 8, snapshot());
  assert.deepEqual(calls, [[ID, 8]]);
  for (const [item, source] of [[target(), undefined], [target(), { worldId: "clockworks", sceneId: "cw_entry_foundry" }],
    [target(), { worldId: "grey_hive", sceneId: "cw_furnace_heart" }], [target({ kind: "terminal" }), snapshot()],
    [target({ entityId: "another_static_dialogue_marker" }), snapshot()], [target({ active: false }), snapshot()]]) {
    await assert.rejects(dispatchInteractable(client, item, 8, source), /E_INTERACTION_(SOURCE_MISMATCH|INACTIVE)/);
  }
  assert.equal(calls.length, 1);
});

test("compiled main, real HUD, dispatcher and TauriClient show only accepted formal_interact success", async () => {
  const h = harness({ acceptanceDeferred: true }); const pending = h.run(); await flush();
  assert.equal(h.count(), 0); assert.equal(h.calls.length, 1);
  assert.equal(h.calls[0].args.actorId, ID); assert.equal(h.calls[0].args.worldEpoch, 8);
  assert.deepEqual(Object.keys(h.calls[0].args).sort(), ["actorId", "requestId", "worldEpoch"]);
  await h.run(); assert.equal(h.calls.length, 1, "busy guard does not dispatch twice");
  h.acceptance.resolve(); await pending;
  assert.equal(h.count(), 1); assert.equal(h.feedback.at(-1), LINE);
  assert.equal(deriveHudState(h.loop.current).interactionText, "");
  await h.run(); assert.equal(h.calls.length, 1); assert.equal(h.count(), 1, "inactive marker cannot dispatch or replay");
});

test("battle action remains a normal interaction without pause or a combat lock", async () => {
  const before = snapshot(); before.player.actionState = "attack";
  const h = harness({ before }); await h.run();
  assert.equal(h.count(), 1); assert.equal(h.loop.pausePresentationState, "running");
  assert.equal(h.calls.length, 1); assert.equal(before.player.actionState, "attack");
});

test("compiled main rejects failed, already-applied, malformed activation and wrong-identity receipts", async () => {
  for (const result of [receipt({ applied: false, errorCode: "E_SCENE_RUNTIME_AlreadyApplied" }),
    receipt({ alreadyApplied: true }), receipt({ applied: false, alreadyApplied: true }),
    receipt({ errorCode: "E_SCENE_RUNTIME_OutOfRange" }), receipt({ applied: false, errorCode: "E_RUNTIME_PAUSED" }),
    receipt({ snapshot: snapshot(false, { worldEpoch: 9 }) }), receipt({ snapshot: snapshot(false, { sceneId: "cw_regulator_core" }) }),
    receipt({ snapshot: snapshot(false, { worldId: "grey_hive" }) }), receipt({ snapshot: snapshot(true, { authorityRevision: 10 }) }),
    receipt({ snapshot: snapshot(false, { authorityRevision: 9 }) }), receipt({ snapshot: snapshot(false, { serverTick: 69 }) }),
    receipt({ snapshot: snapshot(false, { interactables: [] }) }),
    receipt({ snapshot: snapshot(false, { interactables: [target({ active: false, kind: "terminal" })] }) }),
    receipt({ snapshot: snapshot(false, { entryToken: { generation: 1 } }) })]) {
    const h = harness({ result }); await h.run(); assert.equal(h.count(), 0);
  }
  const h = harness({ dispatchDeferred: true }); const pending = h.run();
  h.dispatch.reject(new Error("E_SCENE_RUNTIME_StaleEpoch")); await pending; assert.equal(h.count(), 0);
});

test("late success is dropped across session, source, revision and acceptance/ready fences with no catch-up", async () => {
  for (const stage of ["dispatch", "acceptance"]) {
    for (const change of ["session", "inactive", "scene", "epoch", "generation", "dead", "loading", "entry", "revision"]) {
      const h = harness({ dispatchDeferred: stage === "dispatch", acceptanceDeferred: stage === "acceptance" });
      const pending = h.run(); await flush();
      if (change === "session") { h.context.sessionLoop = { ...h.loop }; h.context.interactionBusy = true; }
      if (change === "inactive") h.loop.acceptsExternalResults = false;
      if (change === "scene") h.loop.current = snapshot(false, { sceneId: "cw_gear_shaft" });
      if (change === "epoch") h.loop.current = snapshot(false, { worldEpoch: 9 });
      if (change === "generation") h.loop.sceneEntrySequence++;
      if (change === "dead") { h.loop.current = snapshot(false); h.loop.current.player.currentHp = 0; h.loop.acceptsExternalResults = false; }
      if (change === "loading") h.loop.pausePresentationState = "loading";
      if (change === "entry") { h.loop.sceneEntrySequence++; h.loop.current = snapshot(false, { entryToken: { generation: 2 } }); }
      if (change === "revision") h.loop.current = snapshot(false, { authorityRevision: stage === "dispatch" ? 11 : 9 });
      if (stage === "dispatch") h.dispatch.resolve(h.result); else h.acceptance.resolve();
      await pending; assert.equal(h.count(), 0, `${stage}/${change}`); assert.equal(h.calls.length, 1);
      if (change === "session") assert.equal(h.context.interactionBusy, true);
      h.loop.current = snapshot(false); h.loop.pausePresentationState = "running";
      await flush(); assert.equal(h.count(), 0, "there is no delayed queue or replay");
    }
  }
  const h = harness({ acceptanceDeferred: true }); const pending = h.run(); await flush();
  h.acceptance.reject(new Error("E_SCENE_ENTRY_ABORTED")); await pending;
  assert.equal(h.count(), 0); assert.equal(h.calls.length, 1);
});

test("loaded activated state stays silent; earlier unactivated saves and a fresh return can interact again", async () => {
  const saved = harness({ before: snapshot(false) }); await saved.run();
  assert.equal(saved.calls.length, 0); assert.equal(saved.count(), 0);
  for (const epoch of [8, 9]) {
    const fresh = harness({ before: snapshot(true, { worldEpoch: epoch }),
      result: receipt({ snapshot: snapshot(false, { worldEpoch: epoch }) }) });
    await fresh.run(); assert.equal(fresh.count(), 1); assert.equal(fresh.calls.length, 1);
  }
});

test("real SessionLoop fresh F respects held-key, paused, loading and dead gates", async () => {
  const calls = [], scheduler = { now: () => 0, clientTimeMs: () => 0, setInterval: () => 1, clearInterval() {},
    requestAnimationFrame: () => 1, cancelAnimationFrame() {} };
  const surface = { addEventListener() {}, removeEventListener() {} };
  const loop = new SessionLoop({ events: async () => [], pause: async () => snapshot(), resume: async () => snapshot() },
    { render: async () => {}, destroy: async () => {} }, { scheduler, windowTarget: surface, documentTarget: surface,
      onInteract: value => { calls.push(value); } });
  await loop.start(snapshot());
  loop.keyDown("f"); loop.keyDown("f"); assert.equal(calls.length, 1);
  loop.keyUp("f"); loop.keyDown("f"); assert.equal(calls.length, 2); loop.keyUp("f");
  await loop.pause(); loop.keyDown("f"); assert.equal(calls.length, 2); await loop.resume();
  // Exercise the compiled keyDown gates with the same lifecycle fields set by entry/death handlers.
  loop.entryLoading = true; loop.keyDown("f"); assert.equal(calls.length, 2); loop.entryLoading = false;
  loop.dead = true; loop.keyDown("f"); assert.equal(calls.length, 2); loop.dead = false;
  await loop.stop();
});

test("real SessionLoop F runs the compiled main chain and stops selecting the accepted inactive marker", async () => {
  const h = harness({ dispatchDeferred: true });
  const scheduler = { now: () => 0, clientTimeMs: () => 0, setInterval: () => 1, clearInterval() {},
    requestAnimationFrame: () => 1, cancelAnimationFrame() {} };
  const surface = { addEventListener() {}, removeEventListener() {} };
  const loop = new SessionLoop({ events: async () => [], pause: async () => loop.snapshots.view(),
      resume: async () => loop.snapshots.view() },
    { render: async () => {}, destroy: async () => {} }, { scheduler, windowTarget: surface, documentTarget: surface,
      onInteract: value => h.context.interactFromSnapshot(value) });
  h.context.sessionLoop = loop;
  await loop.start(h.before);
  loop.keyDown("f"); loop.keyDown("f");
  assert.equal(h.calls.length, 1); assert.equal(h.count(), 0);
  h.dispatch.resolve(h.result);
  for (let attempt = 0; attempt < 20 && h.count() === 0; attempt++) await flush();
  assert.equal(h.count(), 1); assert.equal(deriveHudState(loop.snapshots.view()).interactionId, null);
  loop.keyUp("f"); loop.keyDown("f"); await flush(); assert.equal(h.calls.length, 1);
  await loop.pause(); loop.keyUp("f"); loop.keyDown("f"); await flush(); assert.equal(h.calls.length, 1);
  await loop.stop();
});
