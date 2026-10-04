import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { confirmedReturnStationAfterGreyHive, confirmedReturnStationNewJourney } from "../dist/game/ReturnStationNarrative.js";

const root = new URL("../../../", import.meta.url);
const catalog = JSON.parse(await readFile(new URL("content/dialogue/return_station_narrative_catalog_zh-CN_v1.json", root), "utf8"));
const exitScene = JSON.parse(await readFile(new URL("content/scenes/compiled/gh_exit.json", root), "utf8"));
const events = ["hive_power", "hive_lockdown", "hive_extraction"];

function progress(overrides = {}) {
  return { worldId: "grey_hive", completed: true, firstCompletion: true,
    visitId: 1, revisitCount: 0, completedEvents: events, ...overrides };
}

function snapshot(overrides = {}) {
  return { worldId: "grey_hive", sceneId: "gh_exit", worldEpoch: 8, authorityRevision: 31,
    progression: { currentWorldId: "grey_hive", eventSeq: 7, worlds: [progress()] }, ...overrides };
}

const before = snapshot();
const after = snapshot({ worldId: "return_station", sceneId: "rs_core_room",
  worldEpoch: 9, authorityRevision: 32 });
const receipt = { applied: true, alreadyApplied: false, errorCode: null, snapshot: after };
const line = (prior = before, target = "gh_extraction_return_to_rs", kind = "world_gate", result = receipt) =>
  confirmedReturnStationAfterGreyHive(prior, target, kind, result);

test("first applied Grey Hive extraction return binds the confirmed Return Station line", () => {
  const entry = catalog.entries.find(item => item.id === "rs_after_gh");
  assert.equal(entry.sceneId, "rs_core_room");
  assert.equal(entry.triggerStatus, "pending_scene_hook");
  assert.equal(entry.bodyStatus, "confirmed");
  assert.equal(exitScene.interactions.find(item => item.id === "gh_exit_extraction_console")?.kind, "extraction_console");
  assert.equal(line(), entry.body);
  // Rust keeps the route's currentWorldId at grey_hive during the return gate.
  assert.equal(after.progression.currentWorldId, "grey_hive");
});

test("revisit, rejected, duplicate, wrong route, or unconfirmed progress stays silent", () => {
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt, applied: false }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt, alreadyApplied: true }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt, errorCode: "E_GATE" }), null);
  for (const progressOverride of [
    { visitId: 2, revisitCount: 1 }, { visitId: 1, revisitCount: 1 },
    { completed: false }, { firstCompletion: false },
    { completedEvents: ["hive_power", "hive_lockdown"] }, { completedEvents: null },
  ]) {
    assert.equal(line(snapshot({ progression: { ...before.progression, worlds: [progress(progressOverride)] } })), null);
    assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
      snapshot: { ...after, progression: { ...after.progression, worlds: [progress(progressOverride)] } } }), null);
  }
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, progression: { ...after.progression, worlds: [progress(), progress()] } } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, progression: { ...after.progression, currentWorldId: "return_station" } } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, progression: { ...after.progression, eventSeq: 8 } } }), null);
});

test("source and receipt identities must be the fresh first extraction return", () => {
  assert.equal(line(snapshot({ worldId: "return_station" })), null);
  assert.equal(line(snapshot({ sceneId: "gh_power_room" })), null);
  assert.equal(line(before, "rs_after_gh"), null);
  assert.equal(line(before, "rs_world_gate_marker"), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "scene_trigger"), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, worldId: "grey_hive" } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, sceneId: "gh_exit" } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, worldEpoch: 8 } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, worldEpoch: 10 } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, authorityRevision: 31 } }), null);
  assert.equal(line(before, "gh_extraction_return_to_rs", "world_gate", { ...receipt,
    snapshot: { ...after, progression: { ...after.progression,
      worlds: [progress({ completedEvents: [...events, "unexpected"] })] } } }), null);
});

test("unbound Return Station catalog entries never map to the first return receipt", () => {
  for (const entry of catalog.entries.filter(item => item.id !== "rs_after_gh")) {
    assert.equal(line(before, entry.triggerId, entry.triggerKind), null, entry.id);
  }
});

function freshProgress(overrides = {}) {
  return { worldId: "grey_hive", completed: false, firstCompletion: false,
    visitId: 1, cycleId: 1, revisitCount: 0, completedEvents: [], ...overrides };
}

function freshSnapshot(overrides = {}) {
  return { kind: "full", protocolVersion: 3, worldId: "return_station", sceneId: "rs_core_room",
    worldEpoch: 5, authorityRevision: 3,
    progression: { currentWorldId: "grey_hive", eventSeq: 1, worlds: [freshProgress()] },
    ...overrides };
}

test("new journey in Return Station presents only the confirmed source line", () => {
  const entry = catalog.entries.find(item => item.id === "rs_intro");
  assert.equal(entry.sceneId, "rs_core_room");
  assert.equal(entry.triggerStatus, "pending_scene_hook");
  assert.equal(entry.bodyStatus, "confirmed");
  assert.equal(confirmedReturnStationNewJourney(freshSnapshot()), entry.body);
});

test("intro fails closed for wrong scene, protocol, or stale Grey Hive progress", () => {
  for (const overrides of [
    { kind: "delta" }, { protocolVersion: 2 }, { worldId: "grey_hive" },
    { sceneId: "gh_entry_maintenance" }, { worldEpoch: NaN }, { authorityRevision: "3" },
    { progression: null },
    { progression: { currentWorldId: "return_station", eventSeq: 1, worlds: [freshProgress()] } },
    { progression: { currentWorldId: "grey_hive", eventSeq: 2, worlds: [freshProgress()] } },
    { progression: { currentWorldId: "grey_hive", eventSeq: 1, worlds: [] } },
    { progression: { currentWorldId: "grey_hive", eventSeq: 1, worlds: [null] } },
    { progression: { currentWorldId: "grey_hive", eventSeq: 1, worlds: [{ worldId: "grey_hive" }] } },
    { progression: { currentWorldId: "grey_hive", eventSeq: 1, worlds: [freshProgress(), freshProgress()] } },
  ]) assert.equal(confirmedReturnStationNewJourney(freshSnapshot(overrides)), null);
  for (const progressOverride of [
    { completed: true }, { firstCompletion: true }, { visitId: 0 }, { visitId: 2 },
    { cycleId: null }, { revisitCount: 1 }, { completedEvents: ["hive_power"] },
    { completedEvents: null },
  ]) {
    const candidate = freshSnapshot({ progression: { currentWorldId: "grey_hive", eventSeq: 1,
      worlds: [freshProgress(progressOverride)] } });
    assert.equal(confirmedReturnStationNewJourney(candidate), null);
  }
});

test("new journey alone passes intro to HUD before portrait errors may replace it", async () => {
  const main = await readFile(new URL("apps/web/src/main.ts", root), "utf8");
  assert.match(main, /const snapshot = await client\.newJourney\(\);\s*assertJourneyRequest\(controller\.signal, requestId\);\s*await enterJourney\(snapshot, controller\.signal, requestId, confirmedReturnStationNewJourney\(snapshot\)\)/);
  assert.match(main, /const receipt = latestSave \? \{ snapshot: await client\.continueJourney\(\) \}\s*: await client\.continueSlot\(slot!\.slotId\);\s*assertJourneyRequest\(controller\.signal, requestId\);\s*await enterJourney\(receipt\.snapshot, controller\.signal, requestId\)/);
  assert.ok(main.includes("if (entryNarrative) hud.setFeedback(entryNarrative)"));
  assert.ok(main.indexOf("if (entryNarrative) hud.setFeedback(entryNarrative)") <
    main.indexOf("const portraitError = hud.setPortraitRegistry(registry)"));
});
