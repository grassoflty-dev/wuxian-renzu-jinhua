import test from "node:test";
import assert from "node:assert/strict";
import { capabilityTerminalSummary } from "../dist/game/CapabilityTerminal.js";

function snapshot({ firstClear = false, firstChoice = null, items } = {}) {
  return {
    protocolVersion: 3,
    progression: { worlds: [{
      worldId: "grey_hive", completed: firstClear, firstCompletion: firstClear,
    }] },
    capabilities: { schemaVersion: 1, firstEnhancementChoice: firstChoice, items: items ?? [
      { capabilityId: "information.local_map_i", granted: false, selected: false },
      { capabilityId: "perception.rear_view_i", granted: false, selected: false },
      { capabilityId: "body.regeneration_i", granted: false, selected: false },
      { capabilityId: "information.enemy_vitals_basic", granted: false, selected: false },
    ] },
  };
}

test("capability terminal reports unopened first-clear choices and ungranted projection rows", () => {
  const view = snapshot();
  const before = JSON.stringify(view);
  const summary = capabilityTerminalSummary(view);
  assert.match(summary, /首次撤离尚未完成（首通选择未开放）/);
  assert.match(summary, /局部地图：未获得/);
  assert.match(summary, /后方视野：未获得/);
  assert.match(summary, /再生能力：未获得/);
  assert.match(summary, /敌人生命信息：未获得/);
  assert.equal(JSON.stringify(view), before, "display must not mutate capability projection");
});

test("only an authoritative granted and selected first-clear row is presented as obtained", () => {
  const summary = capabilityTerminalSummary(snapshot({ firstClear: true, firstChoice: "information.local_map_i", items: [
    { capabilityId: "information.local_map_i", granted: true, selected: true },
    { capabilityId: "perception.rear_view_i", granted: false, selected: false },
    { capabilityId: "body.regeneration_i", granted: false, selected: false },
  ] }));
  assert.match(summary, /首通选择：已获得局部地图/);
  assert.match(summary, /局部地图：已获得·已选中/);
  assert.match(summary, /后方视野：未获得/);
});

test("a selected-but-ungranted row is not promoted to an obtained first-clear choice", () => {
  const summary = capabilityTerminalSummary(snapshot({ firstClear: true, items: [
    { capabilityId: "perception.rear_view_i", granted: false, selected: true },
  ] }));
  assert.match(summary, /首通选择：尚未确认/);
  assert.match(summary, /后方视野：投影状态不一致/);
});

test("acoustic mapping status uses only its exact authoritative capability row", () => {
  const cases = [
    [[], "状态未报告·不可用"],
    [[{ capabilityId: "perception.acoustic_mapping_i", granted: false, selected: false }], "未获得·不可用"],
    [[{ capabilityId: "perception.acoustic_mapping_i", granted: true, selected: false }], "已获得·未选中"],
    [[{ capabilityId: "perception.acoustic_mapping_i", granted: true, selected: true }], "已获得·已选中"],
    [[{ capabilityId: "perception.acoustic_mapping_i", granted: false, selected: true }], "投影状态不一致·不可用"],
    [[{ capabilityId: "perception.acoustic_mapping_i", granted: "yes", selected: true }], "投影状态不一致·不可用"],
    [[
      { capabilityId: "perception.acoustic_mapping_i", granted: true, selected: true },
      { capabilityId: "perception.acoustic_mapping_i", granted: false, selected: false },
    ], "投影状态不一致·不可用"],
  ];
  for (const [items, expected] of cases) {
    const summary = capabilityTerminalSummary(snapshot({ firstClear: true, items }));
    assert.match(summary, /首通选择：尚未确认/);
    assert.ok(summary.includes(`声学映射：${expected}`), summary);
  }
  const missing = capabilityTerminalSummary({ ...snapshot({ firstClear: true }), capabilities: {} });
  assert.match(missing, /声学映射：状态未报告·不可用/);
});

test("acoustic mapping never becomes a first-clear choice, even if selected", () => {
  const summary = capabilityTerminalSummary(snapshot({ firstClear: true, firstChoice: "information.local_map_i", items: [
    { capabilityId: "information.local_map_i", granted: true, selected: true },
    { capabilityId: "perception.acoustic_mapping_i", granted: true, selected: true },
  ] }));
  assert.match(summary, /首通选择：已获得局部地图/);
  assert.match(summary, /声学映射：已获得·已选中/);
});

test("Mist Harbor mission progress does not imply acoustic mapping authorization", () => {
  const view = {
    ...snapshot({ items: [] }),
    worldId: "mist_harbor",
    sceneId: "mh_resonance_tower",
    objectives: [{ objectiveId: "mist_signal", state: "completed" }],
    progression: { worlds: [{ worldId: "mist_harbor", completed: true }] },
  };
  assert.match(capabilityTerminalSummary(view), /声学映射：状态未报告·不可用/);
});


test("first choice comes only from explicit authority, even after unrelated grants or deselection", () => {
  const items = [{ capabilityId: "information.local_map_i", granted: true, selected: true }];
  assert.match(capabilityTerminalSummary(snapshot({ firstClear: true, items })), /首通选择：尚未确认/);
  assert.match(capabilityTerminalSummary(snapshot({ firstClear: true, firstChoice: "body.regeneration_i", items })), /首通选择：已获得再生能力/);
});
