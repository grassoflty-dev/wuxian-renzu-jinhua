import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import vm from "node:vm";
import ts from "typescript";
import { canContinueSaveSlot, canOverwriteSaveSlot, saveSlotAvailabilityLabel } from "../dist/bridge/save-slot-policy.js";

// Execute the real main.ts functions, with the compiled production slot policy.
const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
const parsed = ts.createSourceFile("main.ts", source, ts.ScriptTarget.ES2022, true);
const functionNames = new Set(["worldDisplayName", "renderSlotOptions", "slotStatusText"]);
const functions = parsed.statements.filter(node => ts.isFunctionDeclaration(node) && functionNames.has(node.name?.text));
assert.equal(functions.length, functionNames.size);
const production = ts.transpileModule(functions.map(node => node.getText(parsed)).join("\n"), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;

class Select {
  children = [];
  selectedIndex = -1;
  replaceChildren() { this.children = []; this.selectedIndex = -1; }
  append(option) { this.children.push(option); if (this.children.length === 1) this.selectedIndex = 0; }
  get value() { return this.children[this.selectedIndex]?.value ?? ""; }
  set value(value) { this.selectedIndex = this.children.findIndex(option => option.value === value); }
}
const slot = (overrides = {}) => ({ slotId: "slot-original", displayName: "  第一段旅程 · 岑遥「现场」  ",
  updatedAtMs: 1, worldId: "grey_hive", checkpointId: null, playerPositionM: null, currentHp: 80,
  maxHp: 100, currentEnergy: 25, maxEnergy: 50, gateOpen: false, completedEvents: [],
  readOnly: false, valid: true, errorCode: null, ...overrides });
function harness(saveSlots) {
  const context = vm.createContext({ saveSlots, canContinueSaveSlot, canOverwriteSaveSlot, saveSlotAvailabilityLabel,
    document: { createElement(tag) { assert.equal(tag, "option"); return { value: "", textContent: "", disabled: false }; } } });
  vm.runInContext(production, context, { filename: "main-save-slot-world-labels-harness.js" });
  return context;
}
function assertLabels(worldId, displayName) {
  const original = slot({ worldId });
  const before = structuredClone(original);
  const context = harness([original]);
  const select = new Select();
  context.renderSlotOptions(select, original.slotId);
  assert.equal(select.children[0].textContent, `${original.displayName} · ${displayName} · 可读写`);
  assert.equal(context.slotStatusText(original), `将从「${original.displayName}」恢复 ${displayName}。`);
  assert.equal(select.children[0].value, original.slotId);
  assert.equal(select.children[0].disabled, false);
  assert.equal(select.value, original.slotId);
  assert.deepEqual(original, before, "presentation must preserve all save metadata");
}
for (const [worldId, label] of [["return_station", "归航站"], ["grey_hive", "灰巢设施"],
  ["mist_harbor", "雾港余烬"], ["clockworks", "钟骨工厂"]]) {
  test(`${worldId} uses the existing Chinese name in the option and ordinary recovery hint`, () => assertLabels(worldId, label));
}
for (const worldId of [null, ""]) {
  test(`${worldId === null ? "null" : "empty"} world keeps the two original fallback labels`, () => {
    const original = slot({ worldId });
    const before = structuredClone(original);
    const context = harness([original]);
    const select = new Select(); context.renderSlotOptions(select);
    assert.equal(select.children[0].textContent, `${original.displayName} · 未知世界 · 可读写`);
    assert.equal(context.slotStatusText(original), `将从「${original.displayName}」恢复 当前世界。`);
    assert.equal(select.children[0].value, original.slotId);
    assert.equal(select.selectedIndex, -1);
    assert.deepEqual(original, before);
  });
}
test("unknown nonempty world retains the existing worldDisplayName behavior", () => assertLabels("future_world_42", "future_world_42"));

const states = [
  ["valid writable", {}, false, true, "可读写", null],
  ["invalid", { valid: false, errorCode: "E_SLOT_CORRUPT", availability: "unavailable" }, true, false,
    "不可继续 · E_SLOT_CORRUPT", "此存档不可继续：E_SLOT_CORRUPT"],
  ["dead", { currentHp: 0 }, true, true, "不可继续 · 无有效生命值", "此存档不可继续：没有有效的存活状态"],
  ["unknown HP", { currentHp: null }, true, true, "不可继续 · 无有效生命值", "此存档不可继续：没有有效的存活状态"],
  ["legacy read-only", { readOnly: true }, false, false, "旧版只读档 · 续玩时安全迁移",
    "这是旧版只读存档。继续时 Rust 会先备份并安全迁移；不能覆盖。"],
  ["recoverable backup", { readOnly: true, recoverable: true }, false, false, "可恢复 · 继续时安全恢复",
    "可从唯一有效事务备份恢复。点击继续后安全提交，原备份与临时文件保留；不能覆盖。"],
  ["recoverable never overwriteable", { recoverable: true }, false, false, "可恢复 · 继续时安全恢复",
    "可从唯一有效事务备份恢复。点击继续后安全提交，原备份与临时文件保留；不能覆盖。"],
  ["invalid backup", { valid: false, readOnly: true, recoverable: true, errorCode: "E_SLOT_RECOVERY_AMBIGUOUS" },
    true, false, "不可继续 · E_SLOT_RECOVERY_AMBIGUOUS", "此存档不可继续：E_SLOT_RECOVERY_AMBIGUOUS"],
];
for (const [name, overrides, disabled, overwriteable, availability, status] of states) {
  test(`${name} preserves option policy and recovery status semantics`, () => {
    const original = slot(overrides), before = structuredClone(original);
    const context = harness([original]), select = new Select();
    context.renderSlotOptions(select, original.slotId);
    assert.equal(select.children[0].textContent, `${original.displayName} · 灰巢设施 · ${availability}`);
    assert.equal(select.children[0].value, original.slotId);
    assert.equal(select.children[0].disabled, disabled);
    assert.equal(canOverwriteSaveSlot(original), overwriteable);
    assert.equal(context.slotStatusText(original), status ?? `将从「${original.displayName}」恢复 灰巢设施。`);
    assert.equal(select.value, original.slotId);
    assert.deepEqual(original, before);
  });
}
test("specified selection and input order remain unchanged across rerenders", () => {
  const originals = [slot({ slotId: "second", worldId: "mist_harbor" }), slot({ slotId: "first", worldId: "return_station" })];
  const before = structuredClone(originals), context = harness(originals), select = new Select();
  context.renderSlotOptions(select, "first");
  assert.deepEqual(select.children.map(option => option.value), ["second", "first"]);
  assert.equal(select.selectedIndex, 1); assert.equal(select.value, "first");
  context.renderSlotOptions(select, "second");
  assert.equal(select.children.length, 2); assert.equal(select.selectedIndex, 0); assert.equal(select.value, "second");
  assert.deepEqual(originals, before);
});
test("missing or absent selected ID clears selection without falling back to the first slot", () => {
  const context = harness([slot()]), select = new Select();
  for (const selectedId of ["slot-original", undefined, "missing", ""]) {
    context.renderSlotOptions(select, selectedId);
    assert.equal(select.selectedIndex, selectedId === "slot-original" ? 0 : -1);
    assert.equal(select.value, selectedId === "slot-original" ? "slot-original" : "");
  }
});
test("empty slot list replaces stale options and leaves no selection", () => {
  const context = harness([]), select = new Select(); select.append({ value: "stale" });
  context.renderSlotOptions(select, "stale");
  assert.deepEqual(select.children, []); assert.equal(select.selectedIndex, -1); assert.equal(select.value, "");
});
