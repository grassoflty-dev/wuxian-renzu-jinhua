import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { confirmedMistHarborAcousticMappingLine, confirmedMistHarborBeaconSync, MistHarborSignalLineState } from "../dist/game/MistHarborNarrative.js";

const root = new URL("../../../", import.meta.url);
const source = JSON.parse(await readFile(new URL("content/dialogue/mist_harbor_narrative_catalog_zh-CN_v1.json", root), "utf8"));
const westScene = JSON.parse(await readFile(new URL("content/scenes/compiled/mh_tidal_warehouse.json", root), "utf8"));
const eastScene = JSON.parse(await readFile(new URL("content/scenes/compiled/mh_breakwater.json", root), "utf8"));
const towerScene = JSON.parse(await readFile(new URL("content/scenes/compiled/mh_resonance_tower.json", root), "utf8"));
const signalScene = JSON.parse(await readFile(new URL("content/scenes/compiled/mh_signal_yard.json", root), "utf8"));

function snapshot(events, overrides = {}) {
  return {
    worldId: "mist_harbor", sceneId: "mh_breakwater", worldEpoch: 8,
    progression: {
      currentWorldId: "mist_harbor", eventSeq: 12,
      worlds: [{ worldId: "mist_harbor", completedEvents: events }],
    },
    ...overrides,
  };
}

const before = snapshot(["mist_beacon_west"]);
const after = snapshot(["mist_beacon_west", "mist_beacon_east"], {
  progression: {
    currentWorldId: "mist_harbor", eventSeq: 13,
    worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_beacon_west", "mist_beacon_east"] }],
  },
});
const receipt = { applied: true, errorCode: null, snapshot: after };
const sync = (prior = before, target = "mh_east_beacon", kind = "beacon", result = receipt) =>
  confirmedMistHarborBeaconSync(prior, target, kind, result);

test("either second Beacon's new authoritative event binds the exact confirmed source line", () => {
  const line = source.entries.find(entry => entry.id === "mh_sys_beacon_sync");
  assert.equal(line.sceneId, "mh_breakwater");
  assert.equal(line.triggerId, "mh_sys_beacon_sync");
  assert.equal(line.triggerKind, "system_message");
  assert.equal(line.triggerStatus, "pending_scene_hook");
  assert.equal(line.bodyStatus, "confirmed");
  assert.deepEqual(line.relatedEventIds, ["mist_beacon_west", "mist_beacon_east"]);
  const west = source.progressionEvents.find(item => item.id === "mist_beacon_west");
  const east = source.progressionEvents.find(item => item.id === "mist_beacon_east");
  assert.deepEqual([west.sceneId, west.triggerId, east.sceneId, east.triggerId],
    ["mh_tidal_warehouse", "mh_west_beacon", "mh_breakwater", "mh_east_beacon"]);
  assert.deepEqual([westScene.interactions.find(item => item.id === west.triggerId)?.event,
    eastScene.interactions.find(item => item.id === east.triggerId)?.event], line.relatedEventIds);
  assert.equal(sync(), line.body);
  const westSecondBefore = snapshot(["mist_beacon_east"], { sceneId: "mh_tidal_warehouse" });
  const westSecondAfter = snapshot(["mist_beacon_east", "mist_beacon_west"], {
    sceneId: "mh_tidal_warehouse",
    progression: { currentWorldId: "mist_harbor", eventSeq: 13,
      worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_beacon_east", "mist_beacon_west"] }] },
  });
  assert.equal(sync(westSecondBefore, "mh_west_beacon", "beacon",
    { applied: true, errorCode: null, snapshot: westSecondAfter }), line.body);
});

test("failed, duplicate, stale, mismatched, or event-incomplete receipts stay silent", () => {
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, applied: false }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, alreadyApplied: true }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, errorCode: "E_FAILED" }), null);
  assert.equal(sync(snapshot(["mist_beacon_west", "mist_beacon_east"])), null);
  assert.equal(sync(snapshot([])), null);
  assert.equal(sync(before, "mh_west_beacon"), null);
  assert.equal(sync(before, "mh_sys_beacon_sync"), null);
  assert.equal(sync(before, "mh_east_beacon", "scene_trigger"), null);
  assert.equal(sync(snapshot(["mist_beacon_west"], { worldId: "grey_hive" })), null);
  assert.equal(sync(snapshot(["mist_beacon_west"], { sceneId: "mh_tidal_warehouse" })), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after, worldId: "grey_hive" } }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after, sceneId: "mh_fog_pier" } }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after, worldEpoch: 7 } }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after, progression: { ...after.progression, eventSeq: 12 } } }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: snapshot(["mist_beacon_west"], { progression: {
    currentWorldId: "mist_harbor", eventSeq: 13, worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_beacon_west"] }],
  } }) }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after,
    progression: { ...after.progression, currentWorldId: "grey_hive" },
  } }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after,
    progression: { ...after.progression, worlds: [{ worldId: "grey_hive", completedEvents: ["mist_beacon_west", "mist_beacon_east"] }] },
  } }), null);
  assert.equal(sync(before, "mh_east_beacon", "beacon", { ...receipt, snapshot: { ...after,
    progression: { ...after.progression, worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_beacon_west", 1] }] },
  } }), null);
});

test("static markers and pending lines have no runtime narrative", () => {
  for (const entry of source.entries.filter(item => item.triggerStatus === "staged_marker")) {
    assert.equal(sync(before, entry.triggerId, entry.triggerKind), null, entry.id);
  }
  for (const entry of source.entries.filter(item => item.bodyStatus === "pending" && item.triggerStatus === "compiled_event")) {
    assert.equal(sync(snapshot([]), entry.triggerId, entry.triggerKind), null, entry.id);
  }
});

const mappingItem = { capabilityId: "perception.acoustic_mapping_i", granted: true, selected: true };
const towerBefore = {
  ...snapshot(["mist_beacon_west", "mist_beacon_east"], { sceneId: "mh_resonance_tower" }),
  capabilities: { items: [{ ...mappingItem, granted: false, selected: false }] },
};
const towerAfter = {
  ...towerBefore,
  progression: { currentWorldId: "mist_harbor", eventSeq: 13,
    worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_beacon_west", "mist_beacon_east", "mist_signal"] }] },
  capabilities: { items: [mappingItem] },
};
const towerReceipt = { applied: true, errorCode: null, snapshot: towerAfter };
const mapping = (prior = towerBefore, target = "mh_signal_console_staged", kind = "terminal", result = towerReceipt) =>
  confirmedMistHarborAcousticMappingLine(prior, target, kind, result);

test("only the compiled console's first authoritative event and selected grant delivers mh_cy_map_01", () => {
  const line = source.entries.find(entry => entry.id === "mh_cy_map_01");
  const console = towerScene.interactions.find(item => item.id === "mh_signal_console_staged");
  const marker = towerScene.interactions.find(item => item.id === "mh_cy_map_01_staged_marker");
  assert.equal(console.kind, "terminal");
  assert.equal(console.event, "mist_signal");
  assert.equal(marker.kind, "dialogue_content_marker");
  assert.equal(marker.event, null);
  assert.equal(line.bodyStatus, "confirmed");
  assert.equal(line.sourceLine, 656);
  assert.equal(mapping(), line.body);
});

test("mapping line rejects wrong world, scene, epoch, target and kind", () => {
  for (const changed of [
    { worldId: "grey_hive" }, { sceneId: "mh_breakwater" }, { worldEpoch: 7 },
  ]) assert.equal(mapping({ ...towerBefore, ...changed }), null);
  for (const changed of [
    { worldId: "grey_hive" }, { sceneId: "mh_breakwater" }, { worldEpoch: 9 },
  ]) assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt, snapshot: { ...towerAfter, ...changed } }), null);
  assert.equal(mapping(towerBefore, "mh_cy_map_01_staged_marker", "dialogue_content_marker"), null);
  assert.equal(mapping(towerBefore, "mh_acoustic_mapping_staged_marker", "capability_staged_marker"), null);
  assert.equal(mapping(towerBefore, "mh_signal_console_staged", "scene_trigger"), null);
});

test("mapping line rejects failed, duplicate and stale receipts", () => {
  assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt, applied: false }), null);
  assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt, alreadyApplied: true }), null);
  assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt, errorCode: "E_FAILED" }), null);
  assert.equal(mapping({ ...towerBefore, progression: towerAfter.progression }), null);
  assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt,
    snapshot: { ...towerAfter, progression: { ...towerAfter.progression, eventSeq: 12 } } }), null);
  assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt,
    snapshot: { ...towerAfter, progression: { ...towerAfter.progression, eventSeq: undefined } } }), null);
});

test("mapping line needs one newly added MH signal in valid progression", () => {
  assert.equal(mapping({ ...towerBefore, progression: undefined }), null);
  assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt, snapshot: { ...towerAfter, progression: undefined } }), null);
  for (const progression of [
    { ...towerAfter.progression, currentWorldId: "grey_hive" },
    { ...towerAfter.progression, worlds: [] },
    { ...towerAfter.progression, worlds: [
      ...towerAfter.progression.worlds, ...towerAfter.progression.worlds] },
    { ...towerAfter.progression, worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_beacon_west", "mist_beacon_east"] }] },
    { ...towerAfter.progression, worlds: [{ worldId: "mist_harbor", completedEvents: ["mist_signal", "other_event"] }] },
  ]) assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt,
    snapshot: { ...towerAfter, progression } }), null);
});

test("mapping line needs exactly one newly granted and selected capability", () => {
  assert.equal(mapping({ ...towerBefore, capabilities: { items: [mappingItem] } }), null);
  assert.equal(mapping({ ...towerBefore, capabilities: undefined }), null);
  for (const capabilities of [
    undefined, {}, { items: [] }, { items: [mappingItem, mappingItem] },
    { items: [{ ...mappingItem, granted: false }] },
    { items: [{ ...mappingItem, selected: false }] },
  ]) assert.equal(mapping(towerBefore, undefined, undefined, { ...towerReceipt,
    snapshot: { ...towerAfter, capabilities } }), null);
});

test("main shows mapping only after the current-session receipt gate and preserves existing priority", async () => {
  const main = await readFile(new URL("apps/web/src/main.ts", root), "utf8");
  const gate = main.indexOf("isCurrentSceneInteractionResult(sessionLoop === activeLoop && activeLoop.acceptsExternalResults");
  const accepted = main.indexOf("await activeLoop.acceptAuthoritativeSnapshot(result.snapshot)", gate);
  const mappingCall = main.indexOf("confirmedMistHarborAcousticMappingLine(snapshot, interactable.entityId, interactable.kind, result)", accepted);
  assert.ok(gate >= 0 && accepted > gate && mappingCall > accepted);
  assert.ok(main.indexOf("confirmedGreyHiveNarrative(snapshot.worldId", accepted) < mappingCall);
  assert.ok(main.indexOf("confirmedMistHarborBeaconSync(snapshot", accepted) < mappingCall);
  assert.ok(main.indexOf("confirmedReturnStationAfterGreyHive(snapshot", mappingCall) > mappingCall);
  assert.ok(main.indexOf("if (!applied) hud.setFeedback(interactionErrorText", mappingCall) > mappingCall);
  assert.ok(main.indexOf("else if (narrative) hud.setFeedback(narrative)", mappingCall) > mappingCall);
});

const signalHazard = active => ({
  entityId: "mh_signal_interference_region", kind: "signal_interference_zone", active,
  transform: { positionM: { xM: 16, yM: 0, zM: 8 }, yawRad: 0 },
});
const signalSnapshot = (active, overrides = {}) => ({
  worldId: "mist_harbor", sceneId: "mh_signal_yard", worldEpoch: 8,
  hazards: [signalHazard(active)], ...overrides,
});

test("Signal Yard line uses the authored text and the unique Rust hazard, not the static marker", () => {
  const line = source.entries.find(entry => entry.id === "mh_cy_signal_01");
  const authored = signalScene.hazards.find(item => item.id === "mh_signal_interference_region");
  const marker = signalScene.interactions.find(item => item.id === "mh_cy_signal_01_static_marker");
  assert.equal(line.body, "不是没信号，是有人在用噪声盖住它。");
  assert.equal(line.sourceLine, 652);
  assert.equal(line.bodyStatus, "confirmed");
  assert.equal(authored.kind, "signal_interference_zone");
  assert.equal(marker.kind, "dialogue_content_marker");
  assert.equal(marker.event, null);
  assert.equal(new MistHarborSignalLineState(signalSnapshot(false)).accept(signalSnapshot(true)), line.body);
});

test("initial and Continue snapshots establish a silent baseline; first later entry speaks once", () => {
  const outside = new MistHarborSignalLineState(signalSnapshot(false));
  assert.equal(outside.accept(signalSnapshot(false)), null);
  assert.equal(outside.accept(signalSnapshot(true)), "不是没信号，是有人在用噪声盖住它。");
  assert.equal(outside.accept(signalSnapshot(true)), null);
  assert.equal(outside.accept(signalSnapshot(false)), null);
  assert.equal(outside.accept(signalSnapshot(true)), null);
  const continuedInside = new MistHarborSignalLineState(signalSnapshot(true));
  assert.equal(continuedInside.accept(signalSnapshot(true)), null);
  assert.equal(continuedInside.accept(signalSnapshot(false)), null);
  assert.equal(continuedInside.accept(signalSnapshot(true)), "不是没信号，是有人在用噪声盖住它。");
});

test("scene, world and epoch changes establish a fresh baseline", () => {
  const state = new MistHarborSignalLineState(signalSnapshot(false));
  assert.equal(state.accept(signalSnapshot(true, { sceneId: "mh_breakwater" })), null);
  assert.equal(state.accept(signalSnapshot(true)), null);
  assert.equal(state.accept(signalSnapshot(false)), null);
  assert.equal(state.accept(signalSnapshot(true)), "不是没信号，是有人在用噪声盖住它。");
  assert.equal(state.accept(signalSnapshot(true, { worldEpoch: 9 })), null);
  assert.equal(state.accept(signalSnapshot(false, { worldEpoch: 9 })), null);
  assert.equal(state.accept(signalSnapshot(true, { worldEpoch: 9 })), "不是没信号，是有人在用噪声盖住它。");
  assert.equal(state.accept(signalSnapshot(false, { worldId: "grey_hive" })), null);
  assert.equal(state.accept(signalSnapshot(true, { worldEpoch: 9 })), null);
  assert.equal(state.accept(signalSnapshot(false, { worldEpoch: NaN })), null);
});

test("a spoken line stays spent after leaving and returning to Signal Yard in the same epoch", () => {
  const state = new MistHarborSignalLineState(signalSnapshot(false));
  assert.equal(state.accept(signalSnapshot(true)), "不是没信号，是有人在用噪声盖住它。");
  assert.equal(state.accept(signalSnapshot(false, { sceneId: "mh_breakwater" })), null);
  assert.equal(state.accept(signalSnapshot(false)), null);
  assert.equal(state.accept(signalSnapshot(true)), null);
  assert.equal(state.accept(signalSnapshot(false, { worldId: "grey_hive" })), null);
  assert.equal(state.accept(signalSnapshot(false)), null);
  assert.equal(state.accept(signalSnapshot(true)), null);
  assert.equal(state.accept(signalSnapshot(false, { worldEpoch: 9 })), null);
  assert.equal(state.accept(signalSnapshot(true, { worldEpoch: 9 })), "不是没信号，是有人在用噪声盖住它。");
});

test("busy and paused receipt frames defer the line until a later eligible snapshot", () => {
  for (const reason of ["interactionBusy", "paused"]) {
    const state = new MistHarborSignalLineState(signalSnapshot(false));
    assert.equal(state.accept(signalSnapshot(true), false), null, reason);
    assert.equal(state.accept(signalSnapshot(true), false), null, reason);
    assert.equal(state.accept(signalSnapshot(true), true), "不是没信号，是有人在用噪声盖住它。", reason);
    assert.equal(state.accept(signalSnapshot(true), true), null, reason);
    assert.equal(state.accept(signalSnapshot(false), true), null, reason);
    assert.equal(state.accept(signalSnapshot(true), true), null, reason);
  }
});

test("missing, duplicate, wrong-kind and malformed hazards never create a false entry", () => {
  const invalidHazards = [
    undefined, [], [signalHazard(true), signalHazard(true)],
    [{ ...signalHazard(true), kind: "dialogue_content_marker" }],
    [{ ...signalHazard(true), active: "true" }],
    [{ ...signalHazard(true), transform: undefined }],
    [{ ...signalHazard(true), transform: { positionM: { xM: NaN, yM: 0, zM: 8 }, yawRad: 0 } }],
    [{ ...signalHazard(true), transform: { positionM: { xM: 16, yM: 0, zM: 8 }, yawRad: NaN } }],
  ];
  for (const hazards of invalidHazards) {
    const state = new MistHarborSignalLineState(signalSnapshot(false));
    assert.equal(state.accept(signalSnapshot(true, { hazards })), null);
    assert.equal(state.accept(signalSnapshot(true)), null);
    assert.equal(state.accept(signalSnapshot(false)), null);
    assert.equal(state.accept(signalSnapshot(true)), "不是没信号，是有人在用噪声盖住它。");
    assert.equal(new MistHarborSignalLineState(signalSnapshot(false, { hazards })).accept(signalSnapshot(true)), null);
  }
});

test("signal line ignores marker, progression text, map ring and Local Map capability", () => {
  const state = new MistHarborSignalLineState(signalSnapshot(false));
  assert.equal(state.accept(signalSnapshot(false, {
    interactables: [{ entityId: "mh_cy_signal_01_static_marker", kind: "dialogue_content_marker" }],
    progression: { completedEvents: ["mist_signal"], text: "不是没信号，是有人在用噪声盖住它。" },
    capabilities: { items: [{ capabilityId: "information.local_map_i", granted: true, selected: true }] },
    mapRing: true,
  })), null);
  assert.equal(state.accept(signalSnapshot(true, { capabilities: { items: [] } })), "不是没信号，是有人在用噪声盖住它。");
});

test("main initializes from the entry snapshot and keeps receipt and pause feedback priority", async () => {
  const main = await readFile(new URL("apps/web/src/main.ts", root), "utf8");
  const entry = main.indexOf("async function enterJourney(");
  const baseline = main.indexOf("new MistHarborSignalLineState(snapshot)", entry);
  const start = main.indexOf("await sessionLoop.start(snapshot)", baseline);
  const onSnapshot = main.indexOf("onSnapshot: latest =>", baseline);
  const hudUpdate = main.indexOf("hud.apply(latest)", onSnapshot);
  const signalUpdate = main.indexOf("signalLine.accept(latest,", hudUpdate);
  const displayGate = main.indexOf("!interactionBusy && sessionLoop?.pausePresentationState === \"running\"", signalUpdate);
  const signalFeedback = main.indexOf("if (signalNarrative) hud.setFeedback(signalNarrative)", displayGate);
  const receipt = main.indexOf("await activeLoop.acceptAuthoritativeSnapshot(result.snapshot)", start);
  const receiptFeedback = main.indexOf("if (!applied) hud.setFeedback(interactionErrorText", receipt);
  assert.ok(entry >= 0 && baseline > entry && onSnapshot > baseline && hudUpdate > onSnapshot);
  assert.ok(signalUpdate > hudUpdate && displayGate > signalUpdate && signalFeedback > displayGate && start > signalFeedback);
  assert.ok(receipt > start && receiptFeedback > receipt);
  assert.match(main.slice(receipt, receiptFeedback), /confirmedGreyHiveNarrative|confirmedMistHarborBeaconSync/);
});
