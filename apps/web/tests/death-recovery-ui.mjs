import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import vm from "node:vm";
import ts from "typescript";
import { HudPresenter } from "../dist/ui/Hud.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { canContinueSaveSlot, canOverwriteSaveSlot } from "../dist/bridge/save-slot-policy.js";

// Run production main functions/listeners with injected DOM, IPC and renderer
// surfaces. The recovery path still uses the real SessionLoop readiness barrier.
const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
const parsed = ts.createSourceFile("main.ts", source, ts.ScriptTarget.ES2022, true);
const functionNames = new Set(["selectedSlot", "renderSlotOptions", "updateSaveControls", "updateEnhancementControls",
  "refreshSaveSlots", "slotStatusText", "resetDeathControls", "updateDeathControls", "showDeathRecovery",
  "snapshotIdentity", "showError", "assertJourneyRequest", "enterJourney", "begin", "continueJourney",
  "saveWhilePaused", "returnToMain", "togglePause"]);
const functions = parsed.statements.filter(node => ts.isFunctionDeclaration(node) && functionNames.has(node.name?.text));
assert.equal(functions.length, functionNames.size);
const handlers = parsed.statements.filter(node => ts.isExpressionStatement(node) &&
  /^(?:deathSlot|deathRetryButton|deathNewButton|deathHubButton|continueButton|continueSlot|continueSelected)\.addEventListener|^root\.querySelector<HTMLButtonElement>\("#hud-(?:resume|pause)"\)|^window\.addEventListener\("pagehide"/.test(node.getText(parsed)));
assert.equal(handlers.length, 10);
const production = ts.transpileModule([...functions, ...handlers].map(node => node.getText(parsed)).join("\n"), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;
function deferred() { let resolve, reject; const promise = new Promise((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; }
async function flush() { for (let i = 0; i < 80; i++) await Promise.resolve(); }
class Element {
  hidden = true; disabled = false; value = ""; textContent = ""; children = [];
  style = {}; attributes = {}; dataset = {}; listeners = new Map(); isConnected = true;
  setAttribute(name, value) { this.attributes[name] = value; }
  replaceChildren(...children) { this.children = children; this.value = ""; }
  append(child) { this.children.push(child); }
  focus() { this.focused = true; }
  addEventListener(name, callback) { const callbacks = this.listeners.get(name) ?? new Set(); callbacks.add(callback); this.listeners.set(name, callbacks); }
  removeEventListener(name, callback) { this.listeners.get(name)?.delete(callback); }
  fire(name) { for (const callback of [...this.listeners.get(name) ?? []]) callback({}); }
}
function snapshot(hp = 100, generation = 2, entry = true) {
  const result = { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "gh_entry_maintenance", checkpointId: null,
    worldEpoch: generation, serverTick: 1, authorityRevision: 1, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 1, yM: 0, zM: 1 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: hp, maxHp: 100, currentEnergy: 25, maxEnergy: 50,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: hp ? "idle" : "dead" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [], capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] } };
  if (entry) result.entryToken = { generation, worldId: result.worldId, sceneId: result.sceneId, worldEpoch: generation };
  return result;
}
const slot = (slotId, updatedAtMs = 1, overrides = {}) => ({ slotId, displayName: slotId, updatedAtMs,
  worldId: "grey_hive", checkpointId: null, playerPositionM: null, currentHp: 80, maxHp: 100, currentEnergy: 25,
  maxEnergy: 50, gateOpen: false, completedEvents: [], readOnly: false, valid: true, errorCode: null, ...overrides });
function harness(options = {}) {
  const calls = [], loops = [], intervals = new Map(), frames = new Map(); let sequence = 0;
  const document = new Element(); document.hidden = false; document.baseURI = "https://example.test/";
  document.createElement = () => new Element(); globalThis.document = document;
  const window = new Element(); window.confirm = () => true;
  const names = ["deathOverlay", "deathTitle", "deathSavePicker", "deathSlot", "deathFeedback", "deathRetryButton",
    "deathNewButton", "deathHubButton", "continueSlot", "continueNote", "continueSelected", "continueButton",
    "continuePicker", "pauseSlot", "newSlotName", "saveNewSlotButton", "overwriteSlotButton", "saveFeedback",
    "coreOpenButton", "newButton", "feedback", "enhancementSection", "enhancementFeedback", "journey", "hub",
    "shell", "worldTitle", "worldDetail", "canvas"];
  const elements = Object.fromEntries(names.map(name => [name, new Element()])); elements.newSlotName.value = "现场记录";
  const hudElements = Object.fromEntries(["portrait", "actionState", "skills", "objectivePanel", "interactionPrompt",
    "hpValue", "hpFill", "energyValue", "energyFill", "world", "scene", "objectives", "interaction", "doorStatus",
    "pumpStatus", "feedback", "pauseButton", "pauseOverlay", "pauseTitle", "pauseDetail", "resumeButton"].map(name => [name, new Element()]));
  hudElements.deathOverlay = elements.deathOverlay;
  const hud = new HudPresenter(hudElements); hud.setPortraitRegistry = () => null;
  const audioCuePlayer = Object.fromEntries(["resume", "stop", "clearScene", "playUiError", "suspend", "setEpoch", "setScene", "dispose", "handlePresentationEvents", "setListenerSnapshot"].map(name => [name, () => {}]));
  const coreUi = { close() {}, apply(value) { this.currentSnapshot = value; }, setPortraitRegistry: () => null };
  const client = Object.fromEntries(["listSaveSlots", "hasSave", "continueSlot", "continueJourney", "newJourney", "returnToHub", "saveSlot", "resume", "pause"].map(name => [name, async (...args) => {
    calls.push([name, ...args]); if (options[name]) return options[name](...args);
    if (name === "listSaveSlots") return []; if (name === "hasSave") return false;
    if (name === "continueSlot") return { snapshot: snapshot() };
    return snapshot(100, name === "returnToHub" ? 3 : 2, name !== "returnToHub");
  }]));
  client.events = async () => [];
  client.sceneReady = async (token, remainPaused) => { calls.push(["sceneReady", token, remainPaused]);
    if (options.sceneReady) return options.sceneReady(token, remainPaused);
    const released = snapshot(100, token.generation, false); released.authorityRevision = 2; return released; };
  const scheduler = { now: () => 0, clientTimeMs: () => 0,
    setInterval(callback) { const id = ++sequence; intervals.set(id, callback); return id; }, clearInterval(id) { intervals.delete(id); },
    requestAnimationFrame(callback) { const id = ++sequence; frames.set(id, callback); return id; }, cancelAnimationFrame(id) { frames.delete(id); } };
  const ctx = vm.createContext({ ...elements, client, hud, hudElements, coreUi, audioCuePlayer, document, window,
    Error, DOMException, AbortController, canContinueSaveSlot, canOverwriteSaveSlot,
    journeyRequestId: 1, activeJourneyLoad: null, slotReadId: 0, deathRecoveryTarget: null,
    saveSlots: [], slotsReady: false, latestSaveAvailable: false, busy: false, saveBusy: false, enhancementBusy: false,
    nativeJourneyUncertain: false, interactionBusy: false, hudIdentity: null,
    renderer: null, sceneSession: null, sessionLoop: null, enhancementButtons: [],
    buildCloseBarrier: { closing: false }, isTauri: () => true,
    waitForHubBuild: () => options.waitForHubBuild?.() ?? Promise.resolve(),
    applyWorldTheme() {}, worldDisplayName: value => value, confirmedReturnStationNewJourney: () => null,
    enhancementStatusHud: { reset() {}, apply() {} },
    runtimeAssetLoader: { async load() { calls.push(["assets"]); return {}; } }, sceneDefinitionLoader: {},
    MistHarborSignalLineState: class { accept() { return null; } }, interactFromSnapshot() { throw new Error("unexpected interaction"); },
    WorldRenderer: class { async init() { calls.push(["rendererInit"]); } async destroy() { calls.push(["rendererDestroy"]); }
      clearSoundCues() {} reconcileSoundCue() {} clearTransientPresentation() {} },
    SceneDefinitionSession: class {
      committed = null; async prepare(value) { calls.push(["prepare", value]); await options.prepare?.(value); }
      async render(value) { calls.push(["render", value]); await options.render?.(value); this.committed = value; }
      acceptSnapshot() {} isReadyFor(value) { return this.committed?.worldEpoch === value.worldEpoch; }
      cancel() {} invalidate() {} async destroy() { calls.push(["sceneDestroy"]); } },
    SessionLoop: class extends SessionLoop { constructor(client, renderer, settings) {
      super(client, renderer, { ...settings, scheduler, documentTarget: document, windowTarget: window }); loops.push(this); } },
    root: { querySelector: id => ({ "#hud-resume": hudElements.resumeButton, "#hud-pause": hudElements.pauseButton })[id] },
  });
  const deadLoop = { isDead: true, pausePresentationState: "dead", acceptsExternalResults: false,
    snapshots: { view: () => snapshot(0, 1, false) },
    async stop(reason) { calls.push(["stop", reason]); await options.stop?.(reason);
      if (ctx.sessionLoop !== deadLoop) return;
      ctx.resetDeathControls(); ctx.sessionLoop = null; ctx.renderer = null; ctx.sceneSession = null; },
    async resume() { calls.push(["deadResume"]); }, async pause() { calls.push(["deadPause"]); },
    async retryPauseState() { calls.push(["deadRetryPause"]); },
    async runWhilePaused(callback) { calls.push(["deadSave"]); return callback(); } };
  ctx.sessionLoop = deadLoop; vm.runInContext(production, ctx, { filename: "main-death-recovery-harness.js" });
  return { ctx, calls, deadLoop, loops, intervals, frames, hudElements,
    show: () => ctx.showDeathRecovery(deadLoop), async close() { await ctx.sessionLoop?.stop("unload"); } };
}
function assertNoWrites(h) {
  assert.equal(h.calls.some(([name]) => ["saveSlot", "deadSave", "deadResume", "deadPause", "deadRetryPause", "resume", "pause"].includes(name)), false);
}

test("death hides Resume and save actions immediately, before recovery lookup resolves", async () => {
  const read = deferred(); const h = harness({ listSaveSlots: () => read.promise });
  h.deadLoop.pausePresentationState = "paused"; // Terminal authority wins over a stale UI status.
  const shown = h.show();
  assert.equal(h.ctx.deathOverlay.hidden, false); assert.equal(h.ctx.deathTitle.focused, true);
  assert.equal(h.hudElements.pauseOverlay.hidden, true); assert.equal(h.hudElements.resumeButton.hidden, true);
  assert.equal(h.hudElements.resumeButton.disabled, true); assert.equal(h.hudElements.pauseButton.disabled, true);
  assert.equal(h.ctx.saveNewSlotButton.disabled, true); assert.equal(h.ctx.overwriteSlotButton.disabled, true);
  h.hudElements.resumeButton.fire("click"); h.hudElements.pauseButton.fire("click");
  await h.ctx.saveWhilePaused(true); await h.ctx.saveWhilePaused(false);
  assertNoWrites(h); read.resolve([]); await shown; await h.close();
});
test("named Retry selects newest living valid slot and awaits shared renderer readiness", async () => {
  const receipt = deferred(), draw = deferred(), ready = deferred();
  const h = harness({ listSaveSlots: () => [slot("older", 10), slot("newest", 20), slot("dead", 100, { currentHp: 0 }),
    slot("corrupt", 200, { valid: false }), slot("unknown", 300, { currentHp: null })],
    continueSlot: () => receipt.promise, render: () => draw.promise, sceneReady: () => ready.promise });
  await h.show();
  assert.equal(h.ctx.deathSlot.value, "newest"); assert.equal(h.ctx.deathSavePicker.hidden, false);
  assert.equal(h.ctx.deathRetryButton.disabled, false); assert.equal(h.ctx.deathNewButton.hidden, true);
  assert.equal(h.calls.some(([name]) => name === "hasSave"), false);
  h.ctx.deathRetryButton.fire("click"); h.ctx.deathRetryButton.fire("click"); await flush();
  assert.deepEqual(h.calls.filter(([name]) => name === "continueSlot"), [["continueSlot", "newest"]]);
  assert.equal(h.calls.some(([name]) => name === "render"), false);
  receipt.resolve({ snapshot: snapshot() }); await flush();
  assert.equal(h.ctx.busy, true); assert.equal(h.loops[0].pausePresentationState, "loading");
  assert.equal(h.intervals.size, 0); assert.equal(h.calls.some(([name]) => name === "sceneReady"), false);
  draw.resolve(); await flush();
  assert.equal(h.calls.filter(([name]) => name === "sceneReady").length, 1); assert.equal(h.intervals.size, 0);
  const released = snapshot(100, 2, false); released.authorityRevision = 2; ready.resolve(released); await flush();
  assert.equal(h.ctx.busy, false); assert.equal(h.loops[0].pausePresentationState, "running");
  assert.equal(h.ctx.deathOverlay.hidden, true); assert.equal(h.intervals.size, 1); assertNoWrites(h); await h.close();
});
test("living default autosave falls back to formal Continue and the same renderer gate", async () => {
  const hasSave = deferred(), draw = deferred();
  const h = harness({ listSaveSlots: () => [slot("dead", 100, { currentHp: 0 })], hasSave: () => hasSave.promise, render: () => draw.promise });
  const shown = h.show(); await flush();
  assert.equal(h.ctx.deathNewButton.hidden, true); assert.equal(h.ctx.deathRetryButton.hidden, true);
  hasSave.resolve(true); await shown;
  assert.equal(h.ctx.deathSavePicker.hidden, true); assert.equal(h.ctx.deathRetryButton.hidden, false);
  assert.match(h.ctx.deathRetryButton.textContent, /最近存档/); assert.equal(h.ctx.deathRetryButton.disabled, false);
  assert.equal(h.ctx.deathNewButton.hidden, true); h.ctx.deathRetryButton.fire("click"); await flush();
  assert.equal(h.calls.filter(([name]) => name === "continueJourney").length, 1);
  assert.equal(h.calls.some(([name]) => name === "continueSlot" || name === "newJourney"), false);
  assert.equal(h.loops[0].pausePresentationState, "loading"); assert.equal(h.intervals.size, 0);
  draw.resolve(); await flush(); assert.equal(h.loops[0].pausePresentationState, "running"); assertNoWrites(h); await h.close();
});
test("New is offered only after named and default saves both confirm no living recovery", async () => {
  const h = harness(); await h.show();
  assert.equal(h.ctx.deathNewButton.hidden, false); assert.equal(h.ctx.deathNewButton.disabled, false);
  assert.equal(h.ctx.deathRetryButton.hidden, true); assert.equal(h.calls.some(([name]) => name === "newJourney"), false);
  h.ctx.deathNewButton.fire("click"); h.ctx.deathNewButton.fire("click"); await flush();
  assert.equal(h.calls.filter(([name]) => name === "newJourney").length, 1);
  assert.equal(h.loops[0].pausePresentationState, "running"); assertNoWrites(h); await h.close();
});
for (const failedRead of ["listSaveSlots", "hasSave"]) {
  test(`${failedRead} failure keeps New hidden because unknown is not absence`, async () => {
    const h = harness({ [failedRead]: async () => { throw new Error("read failed"); } }); await h.show();
    assert.equal(h.ctx.deathRecoveryTarget, null); assert.equal(h.ctx.deathRetryButton.hidden, true);
    assert.equal(h.ctx.deathNewButton.hidden, true); assert.equal(h.ctx.deathHubButton.disabled, false);
    assert.match(h.ctx.deathFeedback.textContent, /无法/);
    h.ctx.deathNewButton.fire("click"); h.ctx.deathRetryButton.fire("click"); await flush();
    assert.equal(h.calls.some(([name]) => ["newJourney", "continueJourney", "continueSlot"].includes(name)), false); await h.close();
  });
}
test("late slot read cannot reopen recovery after returning without saving", async () => {
  const read = deferred(); let reads = 0;
  const h = harness({ listSaveSlots: () => ++reads === 1 ? read.promise : [] });
  const shown = h.show(); h.ctx.deathHubButton.fire("click"); await flush();
  assert.equal(h.ctx.hub.hidden, false); assert.equal(h.ctx.deathOverlay.hidden, true);
  assert.deepEqual(h.calls.filter(([name]) => name === "stop" || name === "returnToHub"), [["stop", "hub"], ["returnToHub"]]);
  read.resolve([slot("obsolete")]); await shown;
  assert.equal(h.ctx.deathOverlay.hidden, true); assert.equal(h.ctx.deathRetryButton.hidden, true);
  assert.equal(h.ctx.deathRecoveryTarget, null); assert.equal(h.ctx.coreUi.currentSnapshot, null); assertNoWrites(h);
});
for (const reject of [false, true]) {
  test(`late autosave ${reject ? "error" : "result"} cannot alter the departed session UI`, async () => {
    const read = deferred(); const h = harness({ hasSave: () => read.promise });
    const shown = h.show(); await flush(); h.ctx.deathHubButton.fire("click"); await flush();
    const feedback = h.ctx.deathFeedback.textContent;
    if (reject) read.reject(new Error("obsolete failure")); else read.resolve(true);
    await shown;
    assert.equal(h.ctx.deathOverlay.hidden, true); assert.equal(h.ctx.deathRetryButton.hidden, true);
    assert.equal(h.ctx.deathNewButton.hidden, true); assert.equal(h.ctx.deathRecoveryTarget, null);
    assert.equal(h.ctx.deathFeedback.textContent, feedback); assertNoWrites(h);
  });
}
test("newer recovery lookup beats older autosave result in the same dead session", async () => {
  const first = deferred(); let reads = 0; const h = harness({ hasSave: () => ++reads === 1 ? first.promise : false });
  const obsolete = h.show(); await flush(); await h.show();
  assert.equal(h.ctx.deathRecoveryTarget, "new"); first.resolve(true); await obsolete;
  assert.equal(h.ctx.deathRecoveryTarget, "new"); assert.equal(h.ctx.deathRetryButton.hidden, true);
  assert.equal(h.ctx.deathNewButton.hidden, false); await h.close();
});
for (const latest of [false, true]) {
  test(`late ${latest ? "autosave" : "named-slot"} Continue after pagehide cannot enter a journey`, async () => {
    const receipt = deferred(); const h = harness({ listSaveSlots: () => latest ? [] : [slot("living")], hasSave: () => true,
      continueJourney: () => receipt.promise, continueSlot: () => receipt.promise });
    await h.show(); h.ctx.deathRetryButton.fire("click"); await flush();
    h.ctx.window.fire("pagehide"); receipt.resolve(latest ? snapshot() : { snapshot: snapshot() }); await flush();
    assert.equal(h.loops.length, 0); assert.equal(h.ctx.deathOverlay.hidden, true);
    assert.equal(h.calls.some(([name]) => ["assets", "prepare", "render", "sceneReady"].includes(name)), false); assertNoWrites(h);
  });
}
test("return interrupted during stop never sends stale hub transition or saves death", async () => {
  const stop = deferred(); const h = harness({ stop: () => stop.promise }); await h.show();
  h.ctx.deathHubButton.fire("click"); await flush(); h.ctx.window.fire("pagehide"); stop.resolve(); await flush();
  assert.equal(h.calls.some(([name]) => name === "returnToHub"), false); assertNoWrites(h);
});
test("obsolete return receipt cannot change current UI", async () => {
  const returned = deferred(); const h = harness({ returnToHub: () => returned.promise }); await h.show();
  h.ctx.deathHubButton.fire("click"); await flush(); h.ctx.window.fire("pagehide");
  h.ctx.feedback.textContent = "current screen"; returned.resolve(snapshot(100, 3, false)); await flush();
  assert.equal(h.ctx.feedback.textContent, "current screen"); assert.equal(h.ctx.deathOverlay.hidden, true); assertNoWrites(h);
});


test("death overlay has a short opacity fade with both reduced-motion overrides", async () => {
  const css = await readFile(new URL("../src/style.css", import.meta.url), "utf8");
  assert.match(css, /\.death-overlay:not\(\[hidden\]\)\{animation:death-overlay-fade 180ms ease-out both\}/);
  assert.match(css, /@keyframes death-overlay-fade\{from\{opacity:0\}to\{opacity:1\}\}/);
  assert.match(css, /html\[data-motion="reduced"\] \.death-overlay\{animation:none\}/);
  assert.match(css, /@media\(prefers-reduced-motion:reduce\)\{\.death-overlay\{animation:none\}\}/);
});


test("default autosave Continue timeout locks later New and retry because native outcome is uncertain", async () => {
  const h = harness({ hasSave: () => true, continueJourney: async () => { throw new Error("E_CONTINUE_TIMEOUT"); } });
  await h.show(); h.ctx.deathRetryButton.fire("click"); await flush();
  assert.equal(h.ctx.nativeJourneyUncertain, true); assert.equal(h.loops.length, 0);
  assert.equal(h.ctx.newButton.disabled, true); assert.equal(h.ctx.continueButton.disabled, true);
  await h.ctx.begin("new"); h.ctx.deathRetryButton.fire("click"); await flush();
  assert.equal(h.calls.filter(([name]) => name === "continueJourney").length, 1);
  assert.equal(h.calls.some(([name]) => name === "newJourney"), false); assertNoWrites(h);
});


test("uncertain Hub stop blocks Return and any later journey mutation", async () => {
  const h = harness({ stop: async () => { throw new Error("E_SESSION_STOP_TIMEOUT"); } }); await h.show();
  h.ctx.deathHubButton.fire("click"); await flush();
  assert.equal(h.ctx.nativeJourneyUncertain, true); assert.equal(h.calls.some(([name]) => name === "returnToHub"), false);
  await h.ctx.begin("new"); assert.equal(h.calls.some(([name]) => name === "newJourney"), false);
  assert.equal(h.ctx.newButton.disabled, true); assertNoWrites(h);
});


test("render error during detached save disables New and Continue before cleanup and ignores the late save", async () => {
  const frame = deferred(), save = deferred(); let renders = 0;
  const paused = { ...snapshot(100, 2, false), authorityRevision: 3 };
  const h = harness({ render: () => ++renders > 2 ? frame.promise : undefined,
    pause: () => paused, saveSlot: () => save.promise });
  await h.show(); h.ctx.deathNewButton.fire("click"); await flush();
  const loop = h.loops[0]; assert.equal(loop.pausePresentationState, "running");
  const [id, callback] = h.frames.entries().next().value; h.frames.delete(id); callback(17); await flush();
  await loop.pause(); const saving = h.ctx.saveWhilePaused(true); await flush();
  assert.equal(h.calls.filter(([name]) => name === "saveSlot").length, 1);
  frame.reject(new Error("draw failed while save pending")); await flush();
  assert.equal(loop.hasUnsettledStopMutation, true); assert.equal(h.ctx.nativeJourneyUncertain, true);
  assert.equal(h.ctx.newButton.disabled, true); assert.equal(h.ctx.continueButton.disabled, true);
  assert.equal(h.ctx.sessionLoop, null); assert.equal(h.calls.filter(([name]) => name === "sceneDestroy").length, 1);
  const feedback = h.ctx.feedback.textContent;
  await h.ctx.begin("new"); await h.ctx.continueJourney(slot("old-slot"));
  assert.equal(h.calls.filter(([name]) => name === "newJourney").length, 1);
  assert.equal(h.calls.some(([name]) => name === "continueSlot" || name === "continueJourney"), false);
  save.resolve({ snapshot: snapshot(100, 99, false) }); await saving; await flush();
  assert.equal(h.ctx.feedback.textContent, feedback); assert.equal(h.ctx.sessionLoop, null); assert.equal(h.ctx.nativeJourneyUncertain, true);
  assert.equal(h.ctx.newButton.disabled, true); assert.equal(h.ctx.continueButton.disabled, true);
});

for (const route of ["new", "continue"]) {
  test(`${route} replacement cannot overtake an unsettled old-session mutation`, async () => {
    const h = harness({ listSaveSlots: () => route === "continue" ? [slot("living")] : [],
      stop: async () => { throw new Error("E_SESSION_STOP_UNCERTAIN_MUTATION"); } });
    await h.show(); h.ctx[route === "new" ? "deathNewButton" : "deathRetryButton"].fire("click"); await flush();
    assert.equal(h.ctx.nativeJourneyUncertain, true); assert.equal(h.ctx.newButton.disabled, true); assert.equal(h.ctx.continueButton.disabled, true);
    assert.equal(h.calls.some(([name]) => ["newJourney", "continueSlot", "continueJourney"].includes(name)), false);
  });
}


// The main menu shares recovery IPC, cancellation, and actual SessionLoop entry.
function menuHarness(options = {}) {
  const h = harness(options);
  h.ctx.sessionLoop = null;
  h.ctx.hub.hidden = false;
  return h;
}

for (const [label, slots, hasSave, expected] of [
  ["default autosave only", [], true, true],
  ["named slot only", [slot("named")], false, true],
  ["named and default saves", [slot("named")], true, true],
  ["corrupt or dead slots only", [slot("bad", 1, { valid: false }), slot("dead", 2, { currentHp: 0 })], false, false],
  ["no saves", [], false, false],
]) {
  test(`main menu availability: ${label}`, async () => {
    const h = menuHarness({ listSaveSlots: () => slots, hasSave: () => hasSave });
    await h.ctx.refreshSaveSlots();
    assert.equal(h.ctx.continueButton.disabled, !expected);
    assert.equal(h.ctx.slotsReady, true);
    if (slots.some(canContinueSaveSlot)) {
      assert.equal(h.calls.some(([name]) => name === "hasSave"), false);
      assert.equal(h.ctx.continueSelected.disabled, true); // Explicit named selection remains required.
      h.ctx.continueSlot.value = "named"; h.ctx.continueSlot.fire("change");
    }
    if (expected) {
      h.ctx.continueSelected.fire("click"); h.ctx.continueSelected.fire("click"); await flush();
      assert.equal(h.calls.filter(([name]) => name === (slots.some(canContinueSaveSlot) ? "continueSlot" : "continueJourney")).length, 1);
      assert.equal(h.loops[0].pausePresentationState, "running");
    } else {
      h.ctx.continueSelected.fire("click"); await flush();
      assert.equal(h.calls.some(([name]) => name === "continueSlot" || name === "continueJourney"), false);
    }
    assertNoWrites(h); await h.close();
  });
}

test("main menu default autosave waits for renderer and scene readiness", async () => {
  const draw = deferred(), ready = deferred();
  const h = menuHarness({ hasSave: () => true, render: () => draw.promise, sceneReady: () => ready.promise });
  await h.ctx.refreshSaveSlots(); h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(h.calls.filter(([name]) => name === "continueJourney").length, 1);
  assert.equal(h.loops[0].pausePresentationState, "loading"); assert.equal(h.intervals.size, 0);
  assert.equal(h.calls.some(([name]) => name === "sceneReady"), false);
  draw.resolve(); await flush(); assert.equal(h.intervals.size, 0);
  const released = snapshot(100, 2, false); released.authorityRevision = 2; ready.resolve(released); await flush();
  assert.equal(h.loops[0].pausePresentationState, "running"); assertNoWrites(h); await h.close();
});

for (const existing of ["default", "named"]) {
  test(`main menu pending refresh invalidates prior ${existing} choice`, async () => {
    const read = deferred(); let reads = 0;
    const h = menuHarness({ hasSave: () => true, listSaveSlots: () => ++reads === 1 ? (existing === "named" ? [slot("named")] : []) : read.promise });
    await h.ctx.refreshSaveSlots();
    if (existing === "named") { h.ctx.continueSlot.value = "named"; h.ctx.continueSlot.fire("change"); }
    const refresh = h.ctx.refreshSaveSlots();
    assert.equal(h.ctx.continueButton.disabled, true); assert.equal(h.ctx.continueSelected.disabled, true);
    h.ctx.continueSlot.fire("change"); h.ctx.continueSelected.fire("click"); await flush();
    assert.equal(h.ctx.continueSelected.disabled, true);
    assert.equal(h.calls.some(([name]) => name === "continueSlot" || name === "continueJourney"), false);
    read.resolve([]); await refresh; await h.close();
  });
}

for (const failedRead of ["listSaveSlots", "hasSave"]) {
  test(`main menu ${failedRead} failure clears old autosave availability`, async () => {
    let fail = false;
    const h = menuHarness({ listSaveSlots: () => { if (fail && failedRead === "listSaveSlots") throw Error("read failed"); return []; },
      hasSave: () => { if (fail && failedRead === "hasSave") throw Error("read failed"); return true; } });
    await h.ctx.refreshSaveSlots(); assert.equal(h.ctx.continueButton.disabled, false);
    fail = true; await h.ctx.refreshSaveSlots();
    assert.equal(h.ctx.slotsReady, false); assert.equal(h.ctx.continueButton.disabled, true);
    h.ctx.continueSelected.fire("click"); await flush();
    assert.equal(h.calls.some(([name]) => name === "continueJourney"), false);
    assert.match(h.ctx.continueNote.textContent, /读取存档失败/); await h.close();
  });
}

for (const reject of [false, true]) {
  test(`main menu older autosave ${reject ? "error" : "result"} cannot beat newer no-save read`, async () => {
    const first = deferred(); first.promise.catch(() => undefined); let reads = 0;
    const h = menuHarness({ hasSave: () => ++reads === 1 ? first.promise : false });
    const obsolete = h.ctx.refreshSaveSlots(); await flush(); await h.ctx.refreshSaveSlots();
    const note = h.ctx.continueNote.textContent;
    if (reject) first.reject(Error("obsolete")); else first.resolve(true);
    await obsolete;
    assert.equal(h.ctx.latestSaveAvailable, false); assert.equal(h.ctx.continueButton.disabled, true);
    assert.equal(h.ctx.continueNote.textContent, note); await h.close();
  });
}

for (const late of ["availability", "load"]) {
  test(`main menu late autosave ${late} after pagehide cannot enable or enter`, async () => {
    const read = deferred(); const h = menuHarness({ hasSave: () => late === "availability" ? read.promise : true,
      continueJourney: () => read.promise });
    const refresh = h.ctx.refreshSaveSlots(); await flush();
    if (late === "load") { await refresh; h.ctx.continueSelected.fire("click"); await flush(); }
    h.ctx.window.fire("pagehide"); read.resolve(late === "availability" ? true : snapshot());
    await refresh; await flush();
    assert.equal(h.loops.length, 0); assert.equal(h.ctx.continueButton.disabled, true);
    assert.equal(h.calls.some(([name]) => ["assets", "prepare", "render", "sceneReady"].includes(name)), false);
  });
}

test("main menu default timeout locks further journey mutations", async () => {
  const h = menuHarness({ hasSave: () => true, continueJourney: () => { throw Error("E_CONTINUE_TIMEOUT"); } });
  await h.ctx.refreshSaveSlots(); h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(h.ctx.nativeJourneyUncertain, true); assert.equal(h.ctx.newButton.disabled, true);
  assert.equal(h.ctx.continueButton.disabled, true);
  h.ctx.continueSelected.fire("click"); await h.ctx.begin("new"); await flush();
  assert.equal(h.calls.filter(([name]) => name === "continueJourney").length, 1);
  assert.equal(h.calls.some(([name]) => name === "newJourney"), false); await h.close();
});

test("main menu failed default load rechecks disk before offering retry", async () => {
  let exists = true;
  const h = menuHarness({ hasSave: () => exists, continueJourney: () => { exists = false; throw Error("E_SAVE_CORRUPT"); } });
  await h.ctx.refreshSaveSlots(); h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(h.ctx.nativeJourneyUncertain, false); assert.equal(h.ctx.busy, false);
  assert.equal(h.ctx.continueButton.disabled, true); assert.equal(h.ctx.continueSelected.disabled, true);
  assert.match(h.ctx.feedback.textContent, /E_SAVE_CORRUPT/); assert.equal(h.loops.length, 0);
  assert.equal(h.calls.filter(([name]) => name === "hasSave").length, 2); await h.close();
});

test("a named slot appearing between reads cannot impersonate a default autosave", async () => {
  const h = menuHarness({ listSaveSlots: () => [], hasSave: defaultOnly => !defaultOnly });
  await h.ctx.refreshSaveSlots();
  assert.deepEqual(h.calls.filter(([name]) => name === "hasSave"), [["hasSave", true]]);
  assert.equal(h.ctx.continueButton.disabled, true); h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(h.calls.some(([name]) => name === "continueJourney"), false); await h.close();
});

test("removing the last named slot exposes only a confirmed valid default fallback", async () => {
  let named = true;
  const h = menuHarness({ listSaveSlots: () => named ? [slot("named")] : [], hasSave: defaultOnly => defaultOnly });
  await h.ctx.refreshSaveSlots(); assert.equal(h.calls.some(([name]) => name === "hasSave"), false);
  named = false; await h.ctx.refreshSaveSlots();
  assert.equal(h.ctx.continueButton.disabled, false); assert.equal(h.ctx.continueSelected.disabled, false);
  h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(h.calls.filter(([name]) => name === "continueJourney").length, 1);
  assert.equal(h.calls.some(([name]) => name === "continueSlot"), false); await h.close();
});

test("main menu cancelling legacy migration keeps the selected save and sends no load", async () => {
  const h = menuHarness({ listSaveSlots: () => [slot("legacy", 1, { readOnly: true })] });
  await h.ctx.refreshSaveSlots(); h.ctx.continueSlot.value = "legacy"; h.ctx.continueSlot.fire("change");
  let confirmed = 0; h.ctx.window.confirm = () => { confirmed++; return false; };
  h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(confirmed, 1); assert.equal(h.ctx.busy, false); assert.equal(h.ctx.continueSelected.disabled, false);
  assert.equal(h.calls.some(([name]) => name === "continueJourney" || name === "continueSlot"), false); await h.close();
});

test("running renderer failure leaves surviving autosave available after cleanup", async () => {
  let failRender = false;
  const h = menuHarness({ hasSave: () => true, render: () => { if (failRender) throw Error("E_TEST_RENDER_FAILURE"); } });
  await h.ctx.refreshSaveSlots(); h.ctx.continueSelected.fire("click"); await flush();
  assert.equal(h.loops[0].pausePresentationState, "running");
  failRender = true;
  for (const frame of [...h.frames.values()]) frame(100);
  await flush();
  assert.equal(h.ctx.sessionLoop, null);
  assert.match(h.ctx.feedback.textContent, /E_TEST_RENDER_FAILURE/);
  assert.equal(h.ctx.nativeJourneyUncertain, false);
  assert.equal(h.ctx.hub.hidden, false);
  assert.equal(h.ctx.slotsReady, true, "cleanup must not discard the only menu refresh");
  assert.equal(h.ctx.continueButton.disabled, false, "a good surviving autosave must remain retryable");
  await h.close();
});
