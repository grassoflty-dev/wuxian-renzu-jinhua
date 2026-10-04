import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { confirmedGreyHiveNarrative } from "../dist/game/GreyHiveNarrative.js";

const source = JSON.parse(await readFile(new URL("../../../content/dialogue/grey_hive_narrative_catalog_zh-CN_v1.json", import.meta.url), "utf8"));
const sentinelArena = JSON.parse(await readFile(new URL("../../../content/scenes/compiled/gh_sentinel_arena.json", import.meta.url), "utf8"));
const mainSource = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
const boundCases = [
  ["gh_sys_arrival", "gh_entry_maintenance", "gh_entry_checkpoint", "scene_checkpoint"],
  ["gh_cy_power_01", "gh_power_room", "gh_power_console", "power_console"],
  ["gh_sys_lockdown", "gh_lockdown", "gh_lockdown_terminal", "lockdown_terminal"],
  ["gh_sys_sentinel_01", "gh_sentinel_arena", "gh_sys_sentinel_01", "facility_log"],
];

test("runtime Grey Hive feedback matches confirmed bound source lines", () => {
  for (const [id, sceneId, targetId, kind] of boundCases) {
    const entry = source.entries.find(item => item.id === id);
    assert.ok(entry, `missing catalog entry ${id}`);
    assert.equal(entry.sceneId, sceneId);
    assert.equal(entry.triggerId, targetId);
    assert.equal(entry.triggerStatus, "bound");
    assert.equal(entry.bodyStatus, "confirmed");
    assert.equal(confirmedGreyHiveNarrative("grey_hive", sceneId, targetId, kind, true), entry.body);
  }
});

test("Sentinel Arena facility log is bound to the production compiled interaction", () => {
  const entry = source.entries.find(item => item.id === "gh_sys_sentinel_01");
  const interaction = sentinelArena.interactions.find(item => item.id === "gh_sys_sentinel_01");
  assert.equal(sentinelArena.sceneId, "gh_sentinel_arena");
  assert.equal(entry.sceneId, "gh_sentinel_arena");
  assert.equal(entry.triggerId, "gh_sys_sentinel_01");
  assert.equal(entry.kind, "system_message");
  assert.equal(entry.bodyStatus, "confirmed");
  assert.equal(entry.triggerStatus, "bound");
  assert.equal(entry.body, "自动防卫单元已接管本区。");
  assert.deepEqual(interaction && { id: interaction.id, kind: interaction.kind }, {
    id: "gh_sys_sentinel_01", kind: "facility_log",
  });
});

test("narrative feedback stays behind the current receipt and applied-result guards", () => {
  const receiptGuardIndex = mainSource.indexOf("if (!isCurrentSceneInteractionResult(");
  const narrativeCallIndex = mainSource.indexOf("const narrative = confirmedGreyHiveNarrative(snapshot.worldId");
  assert.ok(receiptGuardIndex >= 0);
  assert.ok(narrativeCallIndex > receiptGuardIndex);
  assert.ok(mainSource.includes('"alreadyApplied" in result && result.alreadyApplied'));
  assert.ok(mainSource.includes("result.applied === true"));
});

test("unapplied, mismatched, and pending interactions produce no narrative", () => {
  assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_power_room", "gh_power_console", "power_console", false), null);
  for (const receipt of [{ applied: false, alreadyApplied: false }, { applied: false, alreadyApplied: true }]) {
    // main.ts passes result.applied === true, so only alreadyApplied remains a false gate.
    assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_sentinel_arena", "gh_sys_sentinel_01", "facility_log", receipt.applied === true), null);
  }
  assert.equal(confirmedGreyHiveNarrative("mist_harbor", "gh_power_room", "gh_power_console", "power_console", true), null);
  assert.equal(confirmedGreyHiveNarrative("mist_harbor", "gh_sentinel_arena", "gh_sys_sentinel_01", "facility_log", true), null);
  assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_gate_a", "gh_power_console", "power_console", true), null);
  assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_power_room", "gh_power_console", "terminal", true), null);
  assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_sentinel_arena", "gh_sys_sentinel_01", "system_message", true), null);
  assert.equal(confirmedGreyHiveNarrative("grey_hive", "gh_sentinel_arena", "wrong_id", "facility_log", true), null);
  for (const entry of source.entries.filter(item => item.bodyStatus === "pending" || item.triggerStatus !== "bound")) {
    assert.equal(confirmedGreyHiveNarrative("grey_hive", entry.sceneId, entry.triggerId, "scene_trigger", true), null);
  }
});
