import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";

const GH = ["hive_power", "hive_lockdown", "hive_extraction"];
const MH = ["mist_beacon_west", "mist_beacon_east", "mist_signal"];
const rows = [
  { label: "灰巢：主电恢复记录", value: "主电还活着，只是被人为切断。" },
  { label: "灰巢：隔离协议记录", value: "隔离协议已被局部覆盖。" },
  { label: "雾港：双基准同步记录", value: "双基准建立，中心干扰源可定位。" },
];
const unavailable = { title: "档案", subtitle: "档案数据尚未同步。", rows: [] };
const available = (entries = []) => ({ title: "档案", subtitle: "已确认事件资料", rows: entries });
const world = (worldId, completedEvents = []) => ({ worldId, completedEvents });
const snapshot = (worlds = [world("grey_hive"), world("mist_harbor")], overrides = {}) =>
  ({ kind: "full", protocolVersion: 3, progression: { schemaVersion: 1, worlds }, ...overrides });
const archive = value => deriveCorePanel("archive", value);
const all = () => snapshot([world("grey_hive", GH), world("mist_harbor", MH)]);
function freeze(value) {
  if (value && typeof value === "object") { Object.values(value).forEach(freeze); Object.freeze(value); }
  return value;
}

test("Archive fixed titles and original bodies match the three frozen catalog entries", async () => {
  const gh = JSON.parse(await readFile(new URL("../../../content/dialogue/grey_hive_narrative_catalog_zh-CN_v1.json", import.meta.url)));
  const mh = JSON.parse(await readFile(new URL("../../../content/dialogue/mist_harbor_narrative_catalog_zh-CN_v1.json", import.meta.url)));
  assert.deepEqual(rows.map(row => row.value), [gh.entries.find(e => e.id === "gh_cy_power_01").body,
    gh.entries.find(e => e.id === "gh_sys_lockdown").body, mh.entries.find(e => e.id === "mh_sys_beacon_sync").body]);
  assert.deepEqual(archive(all()), available(rows));
  const catalog = JSON.parse(await readFile(new URL("../../../server-rs/data/world_progression_v1.json", import.meta.url)));
  assert.deepEqual(catalog.worlds.find(w => w.worldId === "grey_hive").requiredEvents, GH);
  assert.deepEqual(catalog.worlds.find(w => w.worldId === "mist_harbor").requiredEvents, MH);
});

test("Archive fails closed for unavailable global snapshot/progression/worlds shapes and non-v3", () => {
  for (const value of [undefined, null, false, 3, "snapshot", [], {}, { protocolVersion: 2 },
    snapshot(undefined, { kind: undefined }), snapshot(undefined, { kind: "delta" }),
    snapshot([], { protocolVersion: "3" }), snapshot([], { protocolVersion: 4 }),
    ...[undefined, null, false, "route", [], {}, { worlds: null }, { worlds: {} }, { worlds: "worlds" }]
      .map(progression => snapshot([], { progression }))]) assert.deepEqual(archive(value), unavailable);
  for (const schemaVersion of [undefined, null, "1", 0, 2, 999]) {
    assert.deepEqual(archive(snapshot(undefined, { progression: { schemaVersion, worlds: all().progression.worlds } })), unavailable);
  }
});

test("Archive final empty matrix distinguishes unavailable worlds from valid empty evidence", () => {
  assert.deepEqual(archive(snapshot()), available());
  for (const worlds of [[], [null, false, {}, []], [world("clockworks", ["clockworks_core"])],
    [{ worldId: "grey_hive" }, { worldId: "mist_harbor", completedEvents: "bad" }]]) {
    assert.deepEqual(archive(snapshot(worlds)), unavailable);
  }
  for (const worlds of [[world("grey_hive")], [world("mist_harbor")],
    [world("grey_hive"), { worldId: "mist_harbor" }],
    [{ worldId: "grey_hive" }, world("mist_harbor")]]) assert.deepEqual(archive(snapshot(worlds)), available());
});

test("all 64 legal event subsets unlock only exact events, with both beacon orders and fixed display order", () => {
  const subset = (values, mask) => values.filter((_, i) => mask & (1 << i));
  for (let ghMask = 0; ghMask < 8; ghMask++) for (let mhMask = 0; mhMask < 8; mhMask++) {
    const gh = subset(GH, ghMask).reverse(), mh = subset(MH, mhMask).reverse();
    const expected = [gh.includes(GH[0]) && rows[0], gh.includes(GH[1]) && rows[1],
      mh.includes(MH[0]) && mh.includes(MH[1]) && rows[2]].filter(Boolean);
    assert.deepEqual(archive(snapshot([world("mist_harbor", mh), world("grey_hive", gh)])), available(expected));
    assert.deepEqual(archive(snapshot([world("grey_hive", [...gh].reverse()), world("mist_harbor", [...mh].reverse())])), available(expected));
  }
});

test("missing, non-array, non-string, duplicate and foreign events hide only their target world", () => {
  for (const [id, goodId, goodEvents, survivingRows, ownEvents] of [
    ["grey_hive", "mist_harbor", MH, [rows[2]], GH],
    ["mist_harbor", "grey_hive", GH, rows.slice(0, 2), MH],
  ]) for (const completedEvents of [undefined, null, {}, "hive_power", 9,
    [ownEvents[0], 1], [ownEvents[0], null], [ownEvents[0], {}], [...ownEvents, ownEvents[0]],
    [...ownEvents, "unknown"], [...ownEvents, goodEvents[0]], [""], new Array(1)]) {
    const value = snapshot([{ worldId: id, completedEvents }, world(goodId, goodEvents)]);
    assert.deepEqual(archive(value), available(survivingRows));
  }
});

test("duplicate target world rows fail closed even when one or both duplicates look valid", () => {
  for (const duplicates of [[world("grey_hive", GH), world("grey_hive", GH)],
    [world("grey_hive", GH), { worldId: "grey_hive" }]]) {
    assert.deepEqual(archive(snapshot([...duplicates, world("mist_harbor", MH)])), available([rows[2]]));
    assert.deepEqual(archive(snapshot([...duplicates, world("mist_harbor"), world("mist_harbor", MH)])), unavailable);
  }
  assert.deepEqual(archive(snapshot([world("grey_hive", GH), world("mist_harbor", MH), world("mist_harbor")])), available(rows.slice(0, 2)));
});

test("unrelated malformed or duplicate worlds neither unlock entries nor poison valid target evidence", () => {
  const unknown = [{ worldId: "clockworks", completedEvents: ["unknown"] },
    { worldId: "clockworks", completedEvents: null }, { worldId: "future", completedEvents: [...GH, ...MH] }, null, false, []];
  assert.deepEqual(archive(snapshot([...unknown, ...all().progression.worlds])), available(rows));
  assert.deepEqual(archive(snapshot(unknown)), unavailable);
});

test("world completion, capabilities, current location and visit metadata are not archive evidence", () => {
  const value = snapshot([
    { ...world("grey_hive"), completed: true, firstCompletion: true },
    { ...world("mist_harbor"), completed: true, firstCompletion: true },
  ], { worldId: "mist_harbor", capabilities: { items: [{ capabilityId: "perception.acoustic_mapping_i", granted: true }] } });
  assert.deepEqual(archive(value), available());
  for (const worldId of ["return_station", "clockworks", "grey_hive", "mist_harbor"]) {
    const crossWorld = { ...all(), worldId, sceneId: "other_scene" };
    crossWorld.progression.currentWorldId = worldId;
    crossWorld.progression.worlds.forEach(w => Object.assign(w, { completed: false, firstCompletion: false, visitId: 7, revisitCount: 2 }));
    assert.deepEqual(archive(crossWorld), available(rows));
  }
});

test("old restored evidence is sufficient without new fields, and missing evidence is never backfilled", () => {
  assert.deepEqual(archive(snapshot([world("grey_hive", ["hive_lockdown"])])), available([rows[1]]));
  assert.deepEqual(archive(snapshot([{ worldId: "grey_hive", completed: true }, { worldId: "mist_harbor", completed: true }])), unavailable);
});

test("repeated snapshots, slot changes, unsaved rollback and new journeys never accumulate or mutate evidence", () => {
  const value = freeze(all()), before = JSON.stringify(value);
  for (let i = 0; i < 3; i++) assert.deepEqual(archive(value), available(rows));
  const output = archive(value); output.rows[0].value = "local output mutation"; output.rows.push({ label: "extra", value: "extra" });
  assert.deepEqual(archive(value), available(rows));
  assert.deepEqual(archive(null), unavailable);
  assert.deepEqual(archive(snapshot([world("grey_hive", ["hive_power"])])), available([rows[0]]));
  assert.deepEqual(archive(snapshot()), available());
  assert.equal(JSON.stringify(value), before);
});
