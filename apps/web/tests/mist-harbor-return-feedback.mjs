import { confirmedClockworksFurnaceDialogue } from "../dist/game/ClockworksFurnaceDialogue.js";
import test from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import { confirmedReturnStationAfterMistHarbor, confirmedReturnStationAfterGreyHive,
  confirmedReturnStationNewJourney } from "../dist/game/ReturnStationNarrative.js";
import { confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks } from "../dist/game/ClockworksCampaign.js";
import { confirmedGreyHiveNarrative } from "../dist/game/GreyHiveNarrative.js";
import { confirmedMistHarborBeaconSync, confirmedMistHarborAcousticMappingLine } from "../dist/game/MistHarborNarrative.js";
import { scannerRewardFeedback } from "../dist/game/ScannerReward.js";
import { dispatchInteractable, isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback,
  interactionErrorText } from "../dist/game/SceneInteraction.js";

const compiled = await readFile(new URL("../dist/main.js", import.meta.url), "utf8");
function compiledSection(startText, endText) {
  const start = compiled.indexOf(startText), end = compiled.indexOf(endText, start);
  assert.ok(start >= 0 && end > start);
  return compiled.slice(start, end);
}
const handler = compiledSection("async function interactFromSnapshot(", "\nasync function begin(");
const continueHandler = compiledSection("async function continueJourney(", "\ncontinueSelected.addEventListener(");
const catalog = JSON.parse(await readFile(new URL("../../../content/dialogue/return_station_narrative_catalog_zh-CN_v1.json", import.meta.url), "utf8"));
const LINE = "声学映射已固化。第三坐标的机械周期与信号高度同步。";
const EVENTS = ["mist_beacon_west", "mist_beacon_east", "mist_signal"];
const gate = { entityId: "mh_extraction_return_to_rs", kind: "world_gate", active: true };
const flush = () => new Promise(resolve => setImmediate(resolve));
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
function progress(completed = false, overrides = {}) {
  return { worldId: "mist_harbor", completed, firstCompletion: completed,
    visitId: 1, cycleId: 1, revisitCount: 0, completedEvents: [...EVENTS], ...overrides };
}
function snapshot(completed = false, overrides = {}) {
  return { kind: "full", protocolVersion: 3, worldId: "mist_harbor", sceneId: "mh_extraction",
    worldEpoch: 20, serverTick: 500, authorityRevision: 80,
    player: { currentHp: 100 }, capabilities: { items: [] }, interactables: [gate],
    progression: { currentWorldId: "mist_harbor", eventSeq: 9, worlds: [progress(completed)] }, ...overrides };
}
function destination(overrides = {}) {
  return snapshot(true, { worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 21,
    authorityRevision: 81, progression: { currentWorldId: "mist_harbor", eventSeq: 10, worlds: [progress(true)] },
    entryToken: { generation: 3, worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 21 },
    ...overrides });
}
const receipt = (overrides = {}) => ({ applied: true, alreadyApplied: false, errorCode: null,
  snapshot: destination(), ...overrides });
const line = (before = snapshot(), result = receipt(), target = gate.entityId, kind = gate.kind) =>
  confirmedReturnStationAfterMistHarbor(before, target, kind, result);

function harness({ before = snapshot(), result = receipt(), dispatchDeferred = false,
  acceptanceDeferred = false, leaveUnready = false } = {}) {
  const dispatch = deferred(), acceptance = deferred(), feedback = [], calls = [];
  const loop = { current: before, sceneEntrySequence: 7, pausePresentationState: "running",
    acceptsExternalResults: true, isDead: false,
    snapshots: { view() { return loop.current; } },
    async acceptAuthoritativeSnapshot(value) {
      loop.current = value;
      if (value.entryToken) { loop.sceneEntrySequence++; loop.pausePresentationState = "loading"; }
      if (acceptanceDeferred) await acceptance.promise;
      if (loop.current === value && !leaveUnready) {
        loop.current = { ...value, entryToken: undefined };
        loop.pausePresentationState = "running";
      }
    } };
  const context = vm.createContext({ Error, sessionLoop: loop, interactionBusy: false,
    client: { async worldGate(targetId, epoch) {
      calls.push({ targetId, epoch });
      return dispatchDeferred ? dispatch.promise : result;
    } },
    hud: { apply: value => ({ interactionId: value.interactables[0]?.entityId }), setFeedback: value => feedback.push(value) },
    dispatchInteractable, isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback, interactionErrorText,
    confirmedClockworksFurnaceDialogue, scannerRewardFeedback, confirmedGreyHiveNarrative, confirmedMistHarborBeaconSync, confirmedMistHarborAcousticMappingLine,
    confirmedReturnStationAfterGreyHive, confirmedReturnStationAfterMistHarbor,
    confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks,
  });
  vm.runInContext(handler, context);
  return { before, result, context, loop, dispatch, acceptance, feedback, calls,
    run: (source = before) => context.interactFromSnapshot(source),
    count: () => feedback.filter(text => text === LINE).length };
}

test("MH first-return line matches the frozen catalog and requires the real completion transition", () => {
  assert.equal(catalog.entries.find(entry => entry.id === "rs_after_mh").body, LINE);
  assert.equal(line(), LINE);
  assert.equal(line(snapshot(true)), null);
  assert.equal(line(snapshot(false, { progression: { currentWorldId: "mist_harbor", eventSeq: 9,
    worlds: [progress(false, { completed: true, firstCompletion: false })] } })), null);
  // The route keeps MH as current, and Complete adds exactly one route event.
  assert.equal(line(snapshot(), receipt({ snapshot: destination({ progression: {
    currentWorldId: "mist_harbor", eventSeq: 9, worlds: [progress(true)] } }) })), null);
});

test("MH predicate rejects failed, duplicate, mismatched, malformed and non-first route receipts", () => {
  for (const change of [{ applied: false }, { alreadyApplied: true }, { errorCode: "E_GATE" }]) {
    assert.equal(line(snapshot(), receipt(change)), null);
  }
  for (const change of [{ worldId: "grey_hive" }, { sceneId: "mh_signal_tower" },
    { worldEpoch: NaN }, { authorityRevision: -1 }]) assert.equal(line(snapshot(false, change)), null);
  for (const change of [{ worldId: "mist_harbor" }, { sceneId: "mh_extraction" }, { worldEpoch: 20 },
    { worldEpoch: 22 }, { worldEpoch: NaN }, { authorityRevision: 80 }, { authorityRevision: "81" }]) {
    assert.equal(line(snapshot(), receipt({ snapshot: destination(change) })), null);
  }
  for (const target of ["rs_after_mh", "rs_world_gate_marker", "gh_extraction_return_to_rs", "cw_shutdown_return_to_rs"]) {
    assert.equal(line(snapshot(), receipt(), target), null);
  }
  assert.equal(line(snapshot(), receipt(), gate.entityId, "scene_trigger"), null);
  for (const side of ["before", "after"]) {
    const completed = side === "after";
    for (const row of [progress(completed, { visitId: 2 }), progress(completed, { revisitCount: 1 }),
      progress(completed, { cycleId: 2 }), progress(completed, { completed: !completed }),
      progress(completed, { firstCompletion: !completed }), progress(completed, { completedEvents: null }),
      progress(completed, { completedEvents: EVENTS.slice(1) }),
      progress(completed, { completedEvents: [...EVENTS, "unexpected"] }),
      progress(completed, { completedEvents: [EVENTS[0], EVENTS[0], EVENTS[2]] })]) {
      const route = { currentWorldId: "mist_harbor", eventSeq: completed ? 10 : 9, worlds: [row] };
      assert.equal(side === "before" ? line(snapshot(false, { progression: route }))
        : line(snapshot(), receipt({ snapshot: destination({ progression: route }) })), null);
    }
    for (const route of [null, { currentWorldId: "mist_harbor", eventSeq: NaN, worlds: [progress(completed)] },
      { currentWorldId: "return_station", eventSeq: completed ? 10 : 9, worlds: [progress(completed)] },
      { currentWorldId: "mist_harbor", eventSeq: completed ? 10 : 9, worlds: [progress(completed), progress(completed)] }]) {
      assert.equal(side === "before" ? line(snapshot(false, { progression: route }))
        : line(snapshot(), receipt({ snapshot: destination({ progression: route }) })), null);
    }
  }
});

test("compiled main dispatches MH gate once and shows the line only after accepted destination ready", async () => {
  const h = harness({ acceptanceDeferred: true });
  const pending = h.run(); await flush();
  assert.deepEqual(h.calls, [{ targetId: gate.entityId, epoch: 20 }]);
  assert.equal(h.loop.pausePresentationState, "loading"); assert.equal(h.count(), 0);
  await h.run(); // Holding F while loading cannot create a second transaction.
  assert.equal(h.calls.length, 1);
  h.acceptance.resolve(); await pending;
  assert.equal(h.count(), 1); assert.equal(h.feedback.at(-1), LINE);
  await h.run(); // A repeated callback with the source snapshot is now stale.
  assert.equal(h.count(), 1);
  assert.equal(h.context.interactionBusy, false);
});

test("compiled main never replays alreadyApplied, completed saves, revisits or ordinary returns", async () => {
  for (const options of [
    { result: receipt({ alreadyApplied: true }) },
    { result: receipt({ applied: false, alreadyApplied: true }) },
    { before: snapshot(true) },
    { before: snapshot(true, { progression: { currentWorldId: "mist_harbor", eventSeq: 9,
      worlds: [progress(true, { visitId: 2, revisitCount: 1 })] } }) },
    { before: snapshot(false, { interactables: [{ ...gate, entityId: "rs_after_mh" }] }) },
    { before: snapshot(false, { worldId: "return_station", sceneId: "rs_core_room" }) },
  ]) { const h = harness(options); await h.run(); assert.equal(h.count(), 0); }
});

test("compiled main suppresses failed, thrown, wrong-epoch and wrong-destination receipts", async () => {
  for (const result of [receipt({ applied: false, errorCode: "E_MH_EXTRACTION_REQUIRED", snapshot: snapshot() }),
    receipt({ errorCode: "E_MH_WARDEN_DEFEAT_REQUIRED" }),
    receipt({ snapshot: destination({ worldEpoch: 20 }) }), receipt({ snapshot: destination({ worldEpoch: 22 }) }),
    receipt({ snapshot: destination({ sceneId: "gh_exit" }) })]) {
    const h = harness({ result }); await h.run(); assert.equal(h.count(), 0);
  }
  const h = harness({ dispatchDeferred: true }); const pending = h.run();
  h.dispatch.reject(new Error("E_MH_WARDEN_DEFEAT_REQUIRED")); await pending;
  assert.equal(h.count(), 0); assert.equal(h.context.interactionBusy, false);
});

test("compiled main drops late results when source scene, epoch or session is replaced", async () => {
  for (const replace of [h => { h.loop.current = snapshot(false, { sceneId: "mh_fog_pier" }); },
    h => { h.loop.current = snapshot(false, { worldEpoch: 22 }); },
    h => { h.context.sessionLoop = { ...h.loop }; },
    h => { h.loop.acceptsExternalResults = false; }]) {
    const h = harness({ dispatchDeferred: true }); const pending = h.run(); replace(h);
    h.dispatch.resolve(h.result); await pending; assert.equal(h.count(), 0);
  }
});

test("compiled main invalidates feedback during destination loading without replay or catch-up", async () => {
  for (const replace of [h => { h.loop.sceneEntrySequence++; },
    h => { h.loop.current = destination({ worldEpoch: 22, entryToken: undefined }); },
    h => { h.loop.current = destination({ sceneId: "rs_other_room", entryToken: undefined }); },
    h => { h.context.sessionLoop = { ...h.loop }; },
    h => { h.loop.acceptsExternalResults = false; },
    h => { h.loop.current = destination({ player: { currentHp: 0 }, entryToken: undefined }); }]) {
    const h = harness({ acceptanceDeferred: true }); const pending = h.run(); await flush(); replace(h);
    h.acceptance.resolve(); await pending; assert.equal(h.count(), 0); assert.equal(h.calls.length, 1);
    await flush(); assert.equal(h.count(), 0);
  }
});

test("unready, aborted acceptance, stale revision and stale tick never produce MH feedback", async () => {
  const unready = harness({ leaveUnready: true }); await unready.run();
  assert.equal(unready.count(), 0);
  unready.loop.current = { ...unready.result.snapshot, entryToken: undefined };
  unready.loop.pausePresentationState = "running"; await flush(); assert.equal(unready.count(), 0);
  for (const field of ["authorityRevision", "serverTick"]) {
    const h = harness({ acceptanceDeferred: true }); const pending = h.run(); await flush();
    h.loop.current = { ...h.result.snapshot, entryToken: undefined, [field]: h.result.snapshot[field] - 1 };
    h.loop.pausePresentationState = "running"; h.acceptance.resolve(); await pending;
    assert.equal(h.count(), 0);
  }
  const aborted = harness({ acceptanceDeferred: true }); const pending = aborted.run(); await flush();
  aborted.acceptance.reject(new Error("E_SCENE_ENTRY_ABORTED")); await pending;
  assert.equal(aborted.count(), 0); assert.equal(aborted.calls.length, 1);
});

async function continueInMain(loaded, latestSave) {
  const entries = [], calls = [];
  const context = vm.createContext({ Error, AbortController, busy: false, buildCloseBarrier: { closing: false },
    nativeJourneyUncertain: false, isTauri: () => true, slotsReady: true, latestSaveAvailable: true,
    canContinueSaveSlot: () => true, sessionLoop: null, journeyRequestId: 0, activeJourneyLoad: null,
    continueSelected: {}, newButton: {}, continueButton: {}, feedback: {}, deathFeedback: {},
    coreUi: { close() {}, apply() {} },
    updateDeathControls() {}, assertJourneyRequest() {}, waitForHubBuild: async () => {},
    client: { async continueJourney() { calls.push("latest"); return loaded; },
      async continueSlot(id) { calls.push(id); return { snapshot: loaded }; } },
    enterJourney: async (...args) => { entries.push(args); },
    showError(error) { throw error; },
  });
  vm.runInContext(continueHandler, context);
  await context.continueJourney({ slotId: "old-progress", displayName: "旧进度" }, undefined, latestSave);
  assert.equal(entries.length, 1); assert.equal(entries[0][0], loaded);
  assert.equal(entries[0].length, 3, "Continue supplies no entry narrative or completion receipt");
  assert.deepEqual(calls, [latestSave ? "latest" : "old-progress"]);
}

test("compiled Continue loads completed MH saves silently through both save entry paths", async () => {
  const loaded = destination({ entryToken: undefined });
  for (const latest of [false, true]) await continueInMain(loaded, latest);
  assert.equal(confirmedReturnStationNewJourney(loaded), null);
  assert.equal(line(loaded, receipt()), null);
});

test("loading old unfinished progress then genuinely completing again may show a new first-return line", async () => {
  const oldProgress = snapshot();
  await continueInMain(oldProgress, false);
  const first = harness({ before: oldProgress }); await first.run(); assert.equal(first.count(), 1);
  // A fresh session loaded from the same unfinished save owns a new real completion.
  await continueInMain(oldProgress, true);
  const restarted = harness({ before: oldProgress }); await restarted.run(); assert.equal(restarted.count(), 1);
});

test("compiled main preserves Grey Hive and Clockworks first-return feedback", async () => {
  for (const [worldId, sceneId, targetId, events, expected] of [
    ["grey_hive", "gh_exit", "gh_extraction_return_to_rs", ["hive_power", "hive_lockdown", "hive_extraction"],
      "灰巢信标已解析。检测到第二组低可信坐标：雾港余烬。"],
    ["clockworks", "cw_shutdown_exit", "cw_shutdown_return_to_rs", ["clockworks_valves", "clockworks_core", "clockworks_shutdown"],
      "三处坐标不是孤立事件。网络中仍存在未识别节点。"],
  ]) {
    const route = { currentWorldId: worldId, eventSeq: 9, worlds: [progress(true, { worldId, completedEvents: events })] };
    const capabilities = { items: [{ capabilityId: "mobility.air_step_i", granted: true }] };
    const before = snapshot(true, { worldId, sceneId, progression: route, capabilities,
      interactables: [{ entityId: targetId, kind: "world_gate", active: true }] });
    const h = harness({ before, result: receipt({ snapshot: destination({ progression: route, capabilities }) }) });
    await h.run(); assert.equal(h.feedback.at(-1), expected); assert.equal(h.count(), 0);
  }
});
