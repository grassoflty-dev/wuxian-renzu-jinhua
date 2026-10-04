import test from "node:test";
import assert from "node:assert/strict";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";

function snapshot(overrides = {}) {
  return {
    protocolVersion: 3, worldId: "return_station", sceneId: "rs_core_room",
    player: { currentHp: 42, maxHp: 100, currentEnergy: 70, maxEnergy: 100, actionState: "idle" },
    capabilities: { items: [
      { capabilityId: "information.local_map_i", granted: false, selected: false },
      { capabilityId: "information.enemy_vitals_basic", granted: false, selected: false },
      { capabilityId: "perception.rear_view_i", granted: false, selected: false },
      { capabilityId: "body.regeneration_i", granted: true, selected: true },
    ] },
    progression: { worlds: [{ worldId: "grey_hive", completed: false, firstCompletion: false }] },
    objectives: [{ objectiveId: "reach_gate", state: "active" }],
    interactables: [{ entityId: "rs_world_gate_marker", kind: "world_gate", active: true }],
    ...overrides,
  };
}

test("Core UI character and mission read current Rust snapshot values", () => {
  const view = snapshot();
  assert.deepEqual(deriveCorePanel("character", view).rows.map(row => row.value),
    ["42 / 100", "70 / 100", "idle", "权威身体状态未提供"]);
  assert.match(deriveCorePanel("mission", view).rows.map(row => row.value).join(" "), /reach_gate|active|尚未完成/);
});

test("ungranted Local Map, Enemy Vitals, and Rear View are absent from capability panel", () => {
  const values = deriveCorePanel("capability", snapshot()).rows.map(row => row.label);
  assert.deepEqual(values, ["再生能力"]);
  assert.ok(!values.includes("局部地图") && !values.includes("敌人生命信息") && !values.includes("后方视野"));
});

test("network reports native gate truth and keeps missing world authority unknown", () => {
  const rows = deriveCorePanel("world_network", snapshot()).rows;
  assert.deepEqual(rows.map(row => row.value),
    ["归航站", "归航站入口可用", "灰巢首次撤离后解锁", "状态未报告"]);
  const unknownGate = deriveCorePanel("world_network", snapshot({ interactables: [] })).rows;
  assert.equal(unknownGate[1].value, "灰巢入口状态未报告");
  const unlocked = deriveCorePanel("world_network", snapshot({
    progression: { worlds: [
      { worldId: "grey_hive", completed: true, firstCompletion: true },
      { worldId: "mist_harbor", completed: false, firstCompletion: false },
    ] },
    interactables: [
      { entityId: "rs_world_gate_marker", kind: "world_gate", active: false },
      { entityId: "rs_mh_world_gate_marker", kind: "world_gate", active: true },
    ],
  })).rows;
  assert.equal(unlocked[2].value, "归航站入口可用");
});

test("inventory does not invent items and accessibility settings are always available", () => {
  assert.match(deriveCorePanel("inventory", null).rows[0].value, /尚未收到权威背包投影/);
  assert.match(deriveCorePanel("settings", null).subtitle, /始终可用/);
  assert.match(deriveCorePanel("character", null).subtitle, /尚未连接/);
});

test("archive waits for authoritative event evidence and save remains a separate real entry", () => {
  assert.deepEqual(deriveCorePanel("archive", null), { title: "档案", subtitle: "档案数据尚未同步。", rows: [] });
  assert.match(deriveCorePanel("save", null).rows[0].value, /暂停后/);
});

test("character model excludes progression stats", () => {
  const labels = deriveCorePanel("character", snapshot()).rows.map(row => row.label);
  for (const forbidden of ["STR", "DEX", "INT", "等级", "XP"]) assert.ok(!labels.includes(forbidden));
});
