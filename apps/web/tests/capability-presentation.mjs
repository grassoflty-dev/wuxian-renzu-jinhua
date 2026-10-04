import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { CAPABILITY_CATEGORIES, CAPABILITY_LABELS, capabilityState, capabilityStatus, knownCapabilityRows } from "../dist/ui/CapabilityPresentation.js";
import { capabilityGroups, renderCapabilityPanel } from "../dist/ui/components/CapabilityPanel.js";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";
import { capabilityTerminalSummary } from "../dist/game/CapabilityTerminal.js";

const snapshot = items => ({ protocolVersion: 3, capabilities: { items } });
const granted = id => ({ capabilityId: id, granted: true, selected: true, cooldownRemainingMs: 0 });
const AIR_STEP = "mobility.air_step_i";

test("ten formal categories are organizational and empty categories do not fabricate unlocks", () => {
  assert.deepEqual(CAPABILITY_CATEGORIES.map(([id]) => id.toUpperCase()), [
    "PERCEPTION", "INFORMATION", "BODY", "MOBILITY", "COGNITION", "COMBAT", "SPACE", "SOUL", "RULE", "UTILITY",
  ]);
  assert.equal(Object.keys(CAPABILITY_LABELS).length, 6);
  assert.ok(capabilityGroups(snapshot([])).every(group => group.items.length === 0));
});

test("real Air Step is consistently labeled in the page, summary and terminal", () => {
  for (const selected of [true, false]) {
    const row = { ...granted(AIR_STEP), selected };
    const view = snapshot([row]);
    const before = JSON.stringify(view);
    assert.deepEqual(capabilityGroups(view).find(group => group.prefix === "mobility").items, [row]);
    assert.deepEqual(deriveCorePanel("capability", view).rows, [
      { label: "空中踏步", value: selected ? "已获得 · 已选中" : "已获得" },
    ]);
    assert.ok(capabilityTerminalSummary(view).includes(`空中踏步：已获得·${selected ? "已选中" : "未选中"}`));
    assert.equal(JSON.stringify(view), before);
  }
});

test("every supported capability rejects duplicates, malformed flags and invalid cooldowns", () => {
  for (const id of Object.keys(CAPABILITY_LABELS)) {
    const row = granted(id);
    for (const items of [
      [row, row], [row, { capabilityId: id }], [row, { ...row, granted: false }],
      [{ ...row, granted: "yes" }], [{ ...row, selected: 1 }],
      [{ ...row, granted: false, selected: true }],
      [{ ...row, cooldownRemainingMs: -1 }], [{ ...row, cooldownRemainingMs: NaN }],
      [{ ...row, cooldownRemainingMs: 0.5 }], [{ ...row, cooldownRemainingMs: Number.MAX_SAFE_INTEGER + 1 }],
    ]) {
      const view = snapshot(items);
      assert.equal(capabilityState(view, id).status, "invalid", id);
      assert.deepEqual(knownCapabilityRows(view), []);
      assert.ok(capabilityGroups(view).every(group => group.items.length === 0));
      assert.deepEqual(deriveCorePanel("capability", view).rows, [{ label: "能力", value: "尚无已授予能力" }]);
      assert.ok(!capabilityTerminalSummary(view).includes(`${CAPABILITY_LABELS[id]}：已获得`));
    }
  }
});

test("unknown content, future movement modes and progression do not imply capability grants", () => {
  const view = { ...snapshot([
    granted("mobility.flight"), granted("mobility.hover"), granted("body.unknown"),
    granted("soul.unknown"), null, "invalid",
  ]), progression: { worlds: [{ worldId: "clockworks", completed: true, completedEvents: ["clockworks_shutdown"] }] } };
  assert.deepEqual(knownCapabilityRows(view), []);
  assert.equal(capabilityState(view, AIR_STEP).status, "missing");
  assert.equal(capabilityState(view, "mobility.flight").status, "invalid");
  assert.ok(!capabilityTerminalSummary(view).includes("mobility.flight"));
  assert.ok(!capabilityTerminalSummary(view).includes("空中踏步：已获得"));
});

test("a malformed duplicate hides only its ability, without hiding other valid grants", () => {
  const map = granted("information.local_map_i");
  const view = snapshot([granted(AIR_STEP), { capabilityId: AIR_STEP }, map]);
  assert.deepEqual(knownCapabilityRows(view), [map]);
  assert.equal(capabilityGroups(view).find(group => group.prefix === "mobility").items.length, 0);
  assert.match(capabilityTerminalSummary(view), /空中踏步：投影状态不一致·不可用/);
});

test("ungranted and absent projections remain unavailable across all capability types", () => {
  for (const id of Object.keys(CAPABILITY_LABELS)) {
    const view = snapshot([{ ...granted(id), granted: false, selected: false }]);
    assert.ok(capabilityGroups(view).every(group => group.items.length === 0));
    assert.equal(capabilityStatus(view, id), "未获得·不可用");
    assert.equal(capabilityStatus({ capabilities: {} }, id), "状态未报告·不可用");
    assert.equal(capabilityStatus({ capabilities: { items: "bad" } }, id), "状态未报告·不可用");
  }
});

test("known presentation IDs exactly match the current Rust capability catalog", async () => {
  const rust = await readFile(new URL("../../../server-rs/src/capability_v1.rs", import.meta.url), "utf8");
  const ids = [...rust.matchAll(/pub const CAP_[A-Z_]+: &str = "([^"]+)";/g)].map(match => match[1]);
  assert.deepEqual(Object.keys(CAPABILITY_LABELS).sort(), ids.sort());
});

test("the real capability panel renders all ten headings and only the genuine Air Step row", () => {
  class Element {
    children = [];
    textContent = "";
    className = "";
    append(...children) { this.children.push(...children); }
    replaceChildren(...children) { this.children = children; }
  }
  const prior = globalThis.document;
  globalThis.document = { createElement: () => new Element(), createDocumentFragment: () => new Element() };
  try {
    const host = new Element();
    renderCapabilityPanel(host, snapshot([granted(AIR_STEP), granted("mobility.flight")]));
    const flatten = element => [element, ...element.children.flatMap(flatten)];
    const elements = flatten(host);
    const groups = elements.filter(element => element.className === "capability-group");
    assert.equal(groups.length, 10);
    assert.deepEqual(groups.map(group => group.children[0].textContent.split(" · ")[1]),
      CAPABILITY_CATEGORIES.map(([id]) => id.toUpperCase()));
    const rows = elements.filter(element => element.className === "capability-row");
    assert.equal(rows.length, 1);
    assert.equal(rows[0].children[0].textContent, "空中踏步");
    assert.equal(rows[0].children[1].textContent, "已获得 · 已选中");
    assert.ok(!elements.some(element => String(element.textContent).includes("mobility.flight")));
  } finally {
    globalThis.document = prior;
  }
});

test("an ambiguous first-clear choice cannot become unique by filtering a malformed duplicate", () => {
  const map = granted("information.local_map_i");
  const regeneration = granted("body.regeneration_i");
  for (const duplicate of [map, { capabilityId: map.capabilityId }, { ...map, selected: "invalid" }]) {
    const view = { ...snapshot([map, duplicate, regeneration]),
      progression: { worlds: [{ worldId: "grey_hive", completed: true, firstCompletion: true }] } };
    assert.deepEqual(knownCapabilityRows(view), [regeneration]);
    const summary = capabilityTerminalSummary(view);
    assert.match(summary, /首通选择状态需复核/);
    assert.doesNotMatch(summary, /首通选择：已获得/);
    assert.match(summary, /再生能力：已获得·已选中/);
  }
});
