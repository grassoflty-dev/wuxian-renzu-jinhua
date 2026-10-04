import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { capabilityGroups } from "../dist/ui/components/CapabilityPanel.js";

const src = new URL("../src/", import.meta.url);

test("structured panel components keep the three-column empty inventory contract", async () => {
  const inventory = await readFile(new URL("ui/components/InventoryPanel.ts", src), "utf8");
  assert.match(inventory, /物品/);
  assert.match(inventory, /装备/);
  assert.match(inventory, /描述/);
  assert.doesNotMatch(inventory, /示例|测试物品|虚构/);
  const presenter = await readFile(new URL("ui/CoreUiPresenter.ts", src), "utf8");
  assert.match(presenter, /尚未收到权威背包投影/);
});

test("character and capability panels only render authoritative state", async () => {
  const character = await readFile(new URL("ui/components/CharacterPanel.ts", src), "utf8");
  const capability = await readFile(new URL("ui/components/CapabilityPanel.ts", src), "utf8");
  assert.match(character, /currentHp/);
  assert.match(character, /currentEnergy/);
  assert.match(character, /actionState/);
  assert.match(character, /身体状态/);
  assert.match(character, /已授予能力/);
  assert.match(character, /装备不可用/);
  for (const forbidden of ["STR", "DEX", "INT", "等级", "XP"]) assert.doesNotMatch(character, new RegExp(forbidden));
  assert.match(capability, /item\.granted === true/);
  assert.match(capability, /树节点关系与成本数据尚未接入/);
  const model = await readFile(new URL("ui/CapabilityPresentation.ts", src), "utf8");
  assert.match(model, /信息/);
  assert.match(model, /感知/);
  assert.match(model, /身体/);
  for (const category of ["机动", "认知", "战斗", "空间", "灵魂", "规则", "实用"]) assert.match(model, new RegExp(category));
  assert.doesNotMatch(capability, /cost|children|parent/);
  assert.doesNotMatch(capability, /createElement\("button"\)/);
});

test("capability groups remain visible for empty snapshots and project only granted rows", () => {
  const empty = capabilityGroups({ capabilities: { items: [] } });
  assert.deepEqual(empty.map(group => group.label), ["感知", "信息", "身体", "机动", "认知", "战斗", "空间", "灵魂", "规则", "实用"]);
  assert.ok(empty.every(group => group.items.length === 0));
  const projected = capabilityGroups({ capabilities: { items: [
    { capabilityId: "information.local_map_i", granted: true, selected: true },
    { capabilityId: "information.enemy_vitals_basic", granted: false, selected: false },
    { capabilityId: "mobility.air_step_i", granted: true, selected: true },
  ] } });
  assert.deepEqual(projected.find(group => group.prefix === "information").items.map(item => item.capabilityId), ["information.local_map_i"]);
  assert.deepEqual(projected.find(group => group.prefix === "mobility").items.map(item => item.capabilityId), ["mobility.air_step_i"]);
});

test("acoustic mapping appears in the real capability page only for one valid granted row", () => {
  const acoustic = { capabilityId: "perception.acoustic_mapping_i", granted: true, selected: true };
  const perceptionItems = items => capabilityGroups({ capabilities: { items } })
    .find(group => group.prefix === "perception").items;
  assert.deepEqual(perceptionItems([acoustic]), [acoustic]);
  assert.deepEqual(perceptionItems([{ ...acoustic, selected: false }]), [{ ...acoustic, selected: false }]);
  assert.deepEqual(perceptionItems([{ ...acoustic, granted: false }]), []);
  assert.deepEqual(perceptionItems([{ ...acoustic, granted: false, selected: true }]), []);
  assert.deepEqual(perceptionItems([{ ...acoustic, selected: "yes" }]), []);
  assert.deepEqual(perceptionItems([acoustic, acoustic]), []);
  assert.deepEqual(perceptionItems([]), []);
});

test("actual capability page labels the acoustic mapping projection and unavailable states", async () => {
  const capability = await readFile(new URL("ui/components/CapabilityPanel.ts", src), "utf8");
  const model = await readFile(new URL("ui/CapabilityPresentation.ts", src), "utf8");
  assert.match(model, /"perception\.acoustic_mapping_i": "声学映射"/);
  assert.match(capability, /acousticMappingStatus\(snapshot\)/);
  assert.match(capability, /声学映射：\$\{acousticStatus\}/);
});

test("presenter preserves modal focus trap and routes structured panels", async () => {
  const presenter = await readFile(new URL("ui/CoreUiPresenter.ts", src), "utf8");
  assert.match(presenter, /event\.key !== "Tab"/);
  assert.match(presenter, /renderInventoryPanel/);
  assert.match(presenter, /renderCharacterPanel/);
  assert.match(presenter, /renderCapabilityPanel/);
});
