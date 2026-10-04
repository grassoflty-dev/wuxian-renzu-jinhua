import test from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import { CoreUiPresenter } from "../dist/ui/CoreUiPresenter.js";
import { AccessibilityPreferenceStore } from "../dist/ui/AccessibilityPreferences.js";
import { inventorySnapshot } from "./fixtures/inventory-snapshot.mjs";

const compiled = await readFile(new URL("../dist/main.js", import.meta.url), "utf8");
function section(startText, endText) {
  const start = compiled.indexOf(startText), end = compiled.indexOf(endText, start);
  assert.ok(start >= 0 && end > start, `compiled main section ${startText}`);
  return compiled.slice(start, end);
}
const opener = section('coreOpenButton.addEventListener("click", () => {', '\ncontinueButton.addEventListener(');
const snapshotCallback = section('onSnapshot: latest => {', '\n        onInteract:');
const cleanupCallback = section('onCleanup: () => {', '\n    });');
const continueHandler = section('async function continueJourney(', '\ncontinueSelected.addEventListener(');
const beginHandler = section('async function begin(', '\nnewButton.addEventListener(');
const flush = () => new Promise(resolve => setImmediate(resolve));
const row = (worldId, completedEvents = []) => ({ worldId, completedEvents });
const snapshot = (worlds = [row("grey_hive"), row("mist_harbor")], overrides = {}) => inventorySnapshot({
  worldId: "return_station", sceneId: "rs_core_room", progression: { schemaVersion: 1,
    currentWorldId: "clockworks", eventSeq: 20, worlds }, ...overrides,
});
const complete = () => snapshot([row("grey_hive", ["hive_lockdown", "hive_power", "hive_extraction"]),
  row("mist_harbor", ["mist_beacon_east", "mist_beacon_west", "mist_signal"])]);
const expected = [
  ["灰巢：主电恢复记录", "主电还活着，只是被人为切断。"],
  ["灰巢：隔离协议记录", "隔离协议已被局部覆盖。"],
  ["雾港：双基准同步记录", "双基准建立，中心干扰源可定位。"],
];
function freeze(value) {
  if (value && typeof value === "object") { Object.values(value).forEach(freeze); Object.freeze(value); }
  return value;
}
class Element {
  children = []; dataset = {}; attributes = {}; listeners = {}; hidden = false; disabled = false; tabIndex = 0;
  constructor(tagName = "div") { this.tagName = tagName; }
  append(...children) { this.children.push(...children); }
  prepend(...children) { this.children.unshift(...children); }
  replaceChildren(...children) { this.children = children; }
  setAttribute(key, value) { this.attributes[key] = value; }
  getAttribute(key) { return this.attributes[key]; }
  addEventListener(key, handler) { (this.listeners[key] ??= []).push(handler); }
  focus() { document.activeElement = this; }
  contains(node) { return node === this || this.children.some(child => child.contains(node)); }
  fire(type, event = {}) { for (const handler of this.listeners[type] ?? []) handler(event); }
}
const descendants = node => [node, ...node.children.flatMap(descendants)];
function harness(t) {
  const oldDocument = globalThis.document, oldElement = globalThis.HTMLElement;
  globalThis.HTMLElement = Element;
  globalThis.document = { activeElement: null, createElement: tag => new Element(tag), createDocumentFragment: () => new Element("fragment") };
  t.after(() => { globalThis.document = oldDocument; globalThis.HTMLElement = oldElement; });
  const root = new Element(); root.hidden = true;
  const nodes = Object.fromEntries(["title", "subtitle", "content", "close"].map(id => [id, new Element()]));
  // Read the actual main markup's tab order, not a separately invented navigation list.
  const tabs = [...compiled.matchAll(/data-core-panel="([a-z_]+)"/g)].map(match => {
    const tab = new Element("button"); tab.dataset.corePanel = match[1]; return tab;
  });
  assert.equal(tabs.filter(tab => tab.dataset.corePanel === "archive").length, 1);
  root.append(nodes.close, ...tabs, nodes.title, nodes.subtitle, nodes.content);
  root.querySelector = selector => nodes[selector.replace("#core-panel-", "")];
  root.querySelectorAll = selector => selector === "[data-core-panel]" ? tabs
    : [nodes.close, ...tabs.filter(tab => tab.tabIndex !== -1)];
  const preferences = new AccessibilityPreferenceStore(() => null, { dataset: {} });
  preferences.restore();
  const commands = [], presenter = new CoreUiPresenter(root, preferences, undefined, action => { commands.push(action); throw Error("unexpected write"); });
  const trigger = new Element("button"), feedback = [];
  const noOp = () => {}, audio = new Proxy({}, { get: () => noOp });
  const loop = { isDead: false, pausePresentationState: "running", async pause(reason) { loop.pauseCalls.push(reason); loop.pausePresentationState = "paused"; }, pauseCalls: [] };
  const context = vm.createContext({ coreUi: presenter, coreOpenButton: trigger,
    busy: false, buildCloseBarrier: { closing: false }, sessionLoop: loop, ownedLoop: loop,
    audioCuePlayer: audio, renderer: null, worldDetail: {}, worldTitle: {}, shell: { dataset: { view: "journey" } },
    worldDisplayName: id => id, applyWorldTheme: noOp, snapshotIdentity: s => `${s.worldId}:${s.worldEpoch}`,
    hudIdentity: null, hud: { apply: noOp, reset: noOp, setFeedback: value => feedback.push(value) },
    enhancementStatusHud: { apply: noOp, reset: noOp }, signalLine: { accept: () => null },
    interactionBusy: false, evolution: { reconcile: noOp, reset: noOp }, updateEnhancementControls: noOp,
    // This Archive fixture has no Bio dialogue. Keep the new main lifecycle dependency explicit.
    baizhi: { visible: false, reconcile: noOp, reset() { this.visible = false; } }, updateBaizhiControls: noOp,
    sceneSession: null, resetDeathControls: noOp, nativeJourneyUncertain: false, signal: { aborted: false },
    requestId: 1, journeyRequestId: 1, refreshSaveSlots: noOp,
  });
  vm.runInContext(opener, context);
  const onSnapshot = vm.runInContext(`({${snapshotCallback}}).onSnapshot`, context);
  const onCleanup = vm.runInContext(`({${cleanupCallback}}).onCleanup`, context);
  const tab = id => tabs.find(item => item.dataset.corePanel === id);
  return { root, nodes, presenter, trigger, commands, loop, context, feedback, onSnapshot, onCleanup, tab,
    rows: () => nodes.content.children[0]?.children.map(line => line.children.map(node => node.textContent)) ?? [],
    browse() { tab("archive").fire("click"); },
  };
}

test("actual main terminal handler pauses before opening; real Archive tab renders frozen text without commands", async t => {
  const h = harness(t), source = freeze(complete()), before = JSON.stringify(source);
  h.onSnapshot(source); h.trigger.fire("click"); await flush();
  assert.deepEqual(h.loop.pauseCalls, ["manual"]); assert.equal(h.presenter.isOpen, true);
  h.browse(); assert.deepEqual(h.rows(), expected);
  assert.equal(h.nodes.title.textContent, "档案"); assert.equal(h.nodes.subtitle.textContent, "已确认事件资料");
  assert.equal(h.tab("archive").attributes["aria-selected"], "true");
  assert.equal(h.nodes.content.className, "core-panel-content core-panel-content-archive");
  h.onSnapshot(source); h.browse(); assert.deepEqual(h.rows(), expected);
  h.tab("save").fire("click"); assert.equal(h.nodes.title.textContent, "存档");
  h.browse(); assert.deepEqual(h.rows(), expected);
  h.root.fire("keydown", { key: "Escape", preventDefault() {} });
  assert.equal(h.presenter.isOpen, false); assert.equal(document.activeElement, h.trigger);
  h.trigger.fire("click"); await flush(); h.browse(); assert.deepEqual(h.rows(), expected);
  assert.deepEqual(h.commands, []); assert.equal(JSON.stringify(source), before);
  assert.equal(h.presenter.currentSnapshot, source);
});

test("real presenter keyboard navigation, empty states and current main snapshots replace all previous rows", async t => {
  const h = harness(t); h.onSnapshot(complete()); h.trigger.fire("click"); await flush(); h.browse();
  h.tab("mission").fire("keydown", { key: "ArrowRight", preventDefault() {} });
  assert.equal(document.activeElement, h.tab("archive")); assert.deepEqual(h.rows(), expected);
  h.onSnapshot(snapshot([row("grey_hive", ["hive_power"]), { worldId: "mist_harbor" }]));
  assert.deepEqual(h.rows(), expected.slice(0, 1)); assert.equal(h.nodes.subtitle.textContent, "已确认事件资料");
  h.onSnapshot(snapshot()); assert.deepEqual(h.rows(), []); assert.equal(h.nodes.subtitle.textContent, "已确认事件资料");
  h.onSnapshot(snapshot([])); assert.deepEqual(h.rows(), []); assert.equal(h.nodes.subtitle.textContent, "档案数据尚未同步。");
  h.onSnapshot(complete()); assert.deepEqual(h.rows(), expected);
  h.onSnapshot({ protocolVersion: 2 }); assert.deepEqual(h.rows(), []); assert.equal(h.presenter.currentSnapshot, null);
  assert.equal(h.nodes.subtitle.textContent, "档案数据尚未同步。"); assert.deepEqual(h.commands, []);
});

test("main owned cleanup clears old Archive, and obsolete cleanup cannot erase a newer session", async t => {
  const h = harness(t); h.onSnapshot(complete()); h.trigger.fire("click"); await flush(); h.browse();
  h.context.sessionLoop = {}; h.onCleanup(); assert.equal(h.presenter.currentSnapshot.progression.eventSeq, 20);
  h.context.sessionLoop = h.loop; h.onCleanup();
  assert.equal(h.presenter.isOpen, false); assert.equal(h.presenter.currentSnapshot, null);
  h.presenter.open(h.trigger, "archive"); assert.deepEqual(h.rows(), []);
  assert.equal(h.nodes.subtitle.textContent, "档案数据尚未同步。");
  assert.deepEqual(h.commands, []);
});

function loadContext(h, source, deferred = false) {
  let resolve;
  const pending = new Promise(yes => { resolve = yes; });
  const calls = [];
  Object.assign(h.context, { AbortController, Error, busy: false, sessionLoop: null,
    isTauri: () => true, slotsReady: true, latestSaveAvailable: true, canContinueSaveSlot: () => true,
    activeJourneyLoad: null, continueSelected: {}, newButton: {}, continueButton: {},
    feedback: {}, deathFeedback: {}, updateDeathControls() {}, assertJourneyRequest() {}, waitForHubBuild: async () => {},
    client: {
      async continueJourney() { calls.push("latest"); return deferred ? pending : source; },
      async continueSlot(id) { calls.push(id); return { snapshot: deferred ? await pending : source }; },
      async newJourney() { calls.push("new"); return deferred ? pending : source; },
    },
    enterJourney: async loaded => { h.onSnapshot(loaded); }, confirmedReturnStationNewJourney: () => null,
    showError(error) { throw error; },
  });
  vm.runInContext(continueHandler + "\n" + beginHandler, h.context);
  return { calls, resolve };
}

test("actual Continue/latest/new handlers clear stale Archive while pending, then render only restored events", async t => {
  const h = harness(t);
  for (const path of ["slot", "latest", "new"]) {
    h.presenter.apply(complete()); h.presenter.open(h.trigger, "archive");
    const restored = path === "new" ? snapshot() : snapshot([row("grey_hive", ["hive_power"])]);
    const load = loadContext(h, restored, true);
    const pending = path === "new" ? h.context.begin("new")
      : h.context.continueJourney({ slotId: "older-slot", displayName: "旧进度" }, undefined, path === "latest");
    assert.equal(h.presenter.isOpen, false); assert.equal(h.presenter.currentSnapshot, null);
    await flush(); load.resolve(restored); await pending;
    h.presenter.open(h.trigger, "archive");
    assert.deepEqual(h.rows(), path === "new" ? [] : expected.slice(0, 1));
    assert.equal(h.nodes.subtitle.textContent, "已确认事件资料");
    assert.deepEqual(load.calls, [path === "slot" ? "older-slot" : path]);
  }
  assert.deepEqual(h.commands, []);
});

test("main terminal rejects busy, dead, unconfirmed pause and superseded sessions", async t => {
  const h = harness(t); h.onSnapshot(complete());
  h.context.busy = true; h.trigger.fire("click"); await flush(); assert.equal(h.presenter.isOpen, false);
  h.context.busy = false; h.loop.isDead = true; h.trigger.fire("click"); await flush(); assert.equal(h.presenter.isOpen, false);
  h.loop.isDead = false; h.loop.pause = async () => {}; h.trigger.fire("click"); await flush(); assert.equal(h.presenter.isOpen, false);
  h.loop.pause = async () => { h.context.sessionLoop = {}; h.loop.pausePresentationState = "paused"; };
  h.trigger.fire("click"); await flush(); assert.equal(h.presenter.isOpen, false); assert.deepEqual(h.commands, []);
});
