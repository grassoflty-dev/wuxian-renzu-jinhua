import { createBrowserSession, freePort, startHttpServer, withBrowserCleanup } from "./support/browser-session.mjs";
import { bounded } from "./support/browser-cleanup.mjs";
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const webRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function findEdge() {
  const roots = [process.env["PROGRAMFILES(X86)"], process.env.ProgramFiles].filter(Boolean);
  return roots.map((root) => path.join(root, "Microsoft", "Edge", "Application", "msedge.exe"))
    .find((candidate) => existsSync(candidate));
}

function installMockIpc() {
  const state = {
    calls: [], saved: null, latest: null, rendered: null, renderHistory: [], epoch: 1, tick: 1, newFailures: 1,
  };
  const snapshot = (worldId, sceneId) => ({
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId, sceneId, checkpointId: null, worldEpoch: state.epoch, serverTick: state.tick,
    authorityRevision: state.tick, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
      currentEnergy: 100, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: worldId, eventSeq: 0, worlds: [] },
  });
  const slot = () => ({ slotId: "saved-grey-hive", displayName: "第一段旅程", updatedAtMs: 1000,
    worldId: "grey_hive", checkpointId: null, playerPositionM: null, currentHp: 100, maxHp: 100,
    currentEnergy: 100, maxEnergy: 100, gateOpen: false, completedEvents: [],
    readOnly: false, valid: true, errorCode: null });
  const receipt = (commandId, view) => ({ commandId, applied: true, alreadyApplied: false,
    errorCode: null, worldEpoch: view.worldEpoch, serverTick: view.serverTick,
    authorityRevision: view.authorityRevision, snapshot: view });
  const bridge = {
    metadata: { currentWindow: { label: "main" } },
    async invoke(command, args) {
      state.calls.push({ command, args: args || null });
      if (command === "formal_snapshot") {
        state.latest = snapshot("return_station", "rs_core_room");
        return state.latest;
      }
      if (command === "formal_list_save_slots") return state.saved ? [slot()] : [];
      if (command === "formal_has_save") return args?.defaultOnly === true ? false : Boolean(state.saved);
      if (command === "formal_new") {
        if (state.newFailures > 0) { state.newFailures--; throw new Error("mock reset rejection"); }
        state.epoch++;
        state.tick++;
        state.latest = snapshot("grey_hive", "gh_entry");
        return { snapshot: state.latest };
      }
      if (command === "formal_pause" || command === "formal_resume") {
        state.tick++;
        state.latest = snapshot(state.latest.worldId, state.latest.sceneId);
        return receipt(command === "formal_pause" ? "pause" : "resume", state.latest);
      }
      if (command === "formal_presentation_events" || command === "formal_sound_cues") return [];
      if (command === "formal_submit_input" || command === "formal_submit_action") {
        state.latest = snapshot(state.latest.worldId, state.latest.sceneId);
        return { snapshot: state.latest };
      }
      if (command === "formal_save_slot") {
        state.saved = structuredClone(state.latest);
        return receipt("save-slot", state.latest);
      }
      if (command === "formal_return") {
        state.epoch++;
        state.tick++;
        state.latest = snapshot("return_station", "rs_core_room");
        return { snapshot: state.latest };
      }
      if (command === "formal_continue_slot") {
        if (args?.slotId !== "saved-grey-hive" || !state.saved) throw new Error("mock slot missing");
        state.epoch++;
        state.tick++;
        state.latest = { ...structuredClone(state.saved), worldEpoch: state.epoch,
          serverTick: state.tick, authorityRevision: state.tick };
        return receipt("continue-slot", state.latest);
      }
      throw new Error(`Unexpected mocked IPC command: ${command}`);
    },
    transformCallback(callback) {
      const id = Math.random().toString(36).slice(2);
      bridge[`callback_${id}`] = callback;
      return id;
    },
    unregisterCallback(id) { delete bridge[`callback_${id}`]; },
  };
  window.isTauri = true;
  window.__TAURI_INTERNALS__ = bridge;
  window.__hubLifecycleMock = state;
}

test("hub IPC fixture distinguishes named saves from default automatic saves", async () => {
  const previousWindow = globalThis.window;
  globalThis.window = {};
  try {
    installMockIpc();
    const bridge = window.__TAURI_INTERNALS__;
    assert.equal(await bridge.invoke("formal_has_save", { defaultOnly: true }), false);
    assert.equal(await bridge.invoke("formal_has_save"), false);
    await bridge.invoke("formal_snapshot");
    await bridge.invoke("formal_save_slot", { displayName: "现场记录" });
    assert.equal(await bridge.invoke("formal_has_save"), true);
    assert.equal(await bridge.invoke("formal_has_save", { defaultOnly: true }), false);
    await assert.rejects(bridge.invoke("unknown_command"), /Unexpected mocked IPC command: unknown_command/);
    assert.deepEqual(window.__hubLifecycleMock.calls.map(call => call.command),
      ["formal_has_save", "formal_has_save", "formal_snapshot", "formal_save_slot", "formal_has_save", "formal_has_save", "unknown_command"]);
    assert.deepEqual(window.__hubLifecycleMock.calls[0].args, { defaultOnly: true });
    assert.deepEqual(window.__hubLifecycleMock.calls[5].args, { defaultOnly: true });
  } finally {
    if (previousWindow === undefined) delete globalThis.window;
    else globalThis.window = previousWindow;
  }
});

const rendererMocks = {
  "./assets/RuntimeAssetLoader.js": `export class RuntimeAssetLoader {
    async load() { return { resolveAsset: () => ({ textureUrl: "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'/%3E" }) }; }
  }`,
  "./assets/SceneDefinitionLoader.js": `export class SceneDefinitionLoader { async loadForSnapshot(identity) { return identity; } }`,
  "./game/SceneDefinitionSession.js": `export class SceneDefinitionSession {
    constructor(renderer) { this.renderer = renderer; this.snapshot = null; }
    async prepare(snapshot) { this.acceptSnapshot(snapshot); }
    acceptSnapshot(snapshot) { this.snapshot = snapshot; this.renderer.expectSceneIdentity(snapshot); }
    async render(snapshot) { this.acceptSnapshot(snapshot); await this.renderer.render(snapshot); }
    cancel() {}
    invalidate() { this.snapshot = null; }
    async destroy() { await this.renderer.destroy(); }
  }`,
  "./renderer/WorldRenderer.js": `export class WorldRenderer {
    constructor() {}
    async init() {}
    expectSceneIdentity() {}
    setSceneDefinition() {}
    async render(snapshot) {
      // Observe the renderer boundary, not the IPC producer's latest snapshot.
      const identity = { worldId: snapshot.worldId, sceneId: snapshot.sceneId,
        worldEpoch: snapshot.worldEpoch, serverTick: snapshot.serverTick };
      window.__hubLifecycleMock.rendered = identity;
      window.__hubLifecycleMock.renderHistory.push(identity);
    }
    reconcileSoundCue() {}
    acceptSoundCues() {}
    clearSoundCues() {}
    cancel() {}
    async destroy() {}
  }`,
};

test("hub render probe records only snapshots passed to render", async () => {
  const previousWindow = globalThis.window;
  const state = { rendered: null, renderHistory: [], latest: {
    worldId: "grey_hive", sceneId: "gh_entry", worldEpoch: 6, serverTick: 20,
  } };
  globalThis.window = { __hubLifecycleMock: state };
  try {
    const { WorldRenderer } = await import(`data:text/javascript,${encodeURIComponent(rendererMocks["./renderer/WorldRenderer.js"])}`);
    const renderer = new WorldRenderer();
    renderer.expectSceneIdentity(state.latest);
    assert.equal(state.rendered, null, "an accepted IPC snapshot is not a rendered frame");
    const frame = { ...state.latest, worldEpoch: 2, serverTick: 3 };
    await renderer.render(frame);
    assert.deepEqual(state.rendered, frame, "the probe must report the actual render argument");
    assert.notEqual(state.rendered.worldEpoch, state.latest.worldEpoch, "a stale render cannot pass as the latest IPC epoch");
    frame.sceneId = "changed_after_render";
    assert.equal(state.rendered.sceneId, "gh_entry", "probe stores a value copy");
    assert.equal(state.renderHistory.length, 1);
  } finally {
    if (previousWindow === undefined) delete globalThis.window;
    else globalThis.window = previousWindow;
  }
});

test("hub lifecycle: two new journeys, save, return, continue, and recover after IPC failure", { timeout: 120_000 }, async (t) => {
  const edge = findEdge();
  if (!edge) return t.skip("Microsoft Edge is not installed at its standard Windows path");

  const session = await createBrowserSession(edge, "hub-lifecycle");
  const profile = session.profile;
  const vitePort = await freePort();
  let cdp;
  await withBrowserCleanup(async () => {
  const vite = await bounded(() => createServer({
    configFile: path.join(webRoot, "vite.config.ts"),
    root: webRoot,
    cacheDir: path.join(profile, "vite-cache"),
    server: { host: "127.0.0.1", port: vitePort, strictPort: true },
    plugins: [{
      name: "hub-lifecycle-test-render-boundaries",
      enforce: "pre",
      resolveId(source, importer) {
        if (!importer?.replaceAll("\\", "/").endsWith("/src/main.ts")) return null;
        return Object.hasOwn(rendererMocks, source) ? `\0hub-lifecycle:${source}` : null;
      },
      load(id) {
        if (!id.startsWith("\0hub-lifecycle:")) return null;
        return rendererMocks[id.slice("\0hub-lifecycle:".length)];
      },
    }],
  }), 15000, "E_VITE_CREATE_TIMEOUT");
    await session.listenVite(vite, webRoot);
    await session.start();
    cdp = await session.connect();
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: `(${installMockIpc.toString()})();` });
    await cdp.send("Page.navigate", { url: `http://127.0.0.1:${vitePort}/` });

    const evalValue = async (expression) => {
      const result = await cdp.send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
      if (result.exceptionDetails) throw new Error(result.exceptionDetails.text || "Browser evaluation failed");
      return result.result.value;
    };
    const waitFor = async (expression, description, timeoutMs = 15_000) => {
      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) {
        const value = await evalValue(expression);
        if (value) return value;
        await delay(40);
      }
      const timeoutError = new Error(`Timed out waiting for ${description}`);
      try {
        const diagnostic = await evalValue(`(() => ({
          pause: { hidden: document.querySelector('#pause-overlay')?.hidden,
            title: document.querySelector('#pause-title')?.textContent,
            detail: document.querySelector('#pause-detail')?.textContent },
          saveDisabled: document.querySelector('#save-new-slot')?.disabled,
          continueNote: document.querySelector('#continue-slot-note')?.textContent,
          saveFeedback: document.querySelector('#save-feedback')?.textContent,
          feedback: document.querySelector('#feedback')?.textContent,
          calls: window.__hubLifecycleMock.calls.map(({command,args}) => ({command,args})),
          hasNamedSave: Boolean(window.__hubLifecycleMock.saved),
          rendered: window.__hubLifecycleMock.rendered,
        }))()`);
        t.diagnostic(`hub lifecycle timeout ${description}: ${JSON.stringify(diagnostic)}`);
      } catch (diagnosticError) {
        throw new AggregateError([timeoutError, diagnosticError], "Hub wait and diagnostic both failed", { cause: timeoutError });
      }
      throw timeoutError;
    };
    const click = async (selector) => evalValue(`(() => { const e=document.querySelector(${JSON.stringify(selector)}); if(!e) throw new Error("missing ${selector}"); e.click(); return true; })()`);
    const state = async () => evalValue(`(() => ({
      view: document.querySelector(".shell")?.dataset.view,
      hubHidden: document.querySelector("#hub")?.hidden,
      journeyHidden: document.querySelector("#journey")?.hidden,
      world: document.querySelector("#hud-world")?.textContent,
      rendered: window.__hubLifecycleMock.rendered,
      hudText: document.querySelector("#game-hud")?.textContent,
      hasSceneLabel: document.querySelector("#hud-scene") !== null,
      feedback: document.querySelector("#feedback")?.textContent,
      newDisabled: document.querySelector("#new-journey")?.disabled,
      continueDisabled: document.querySelector("#continue-journey")?.disabled,
      pickerHidden: document.querySelector("#continue-picker")?.hidden,
      selectedDisabled: document.querySelector("#continue-selected")?.disabled,
      pauseHidden: document.querySelector("#pause-overlay")?.hidden,
      saveFeedback: document.querySelector("#save-feedback")?.textContent,
      selectedSlot: document.querySelector("#continue-slot")?.value,
    }))()`);

    await waitFor("document.querySelector('#new-journey') && document.querySelector('#new-journey').disabled === false", "hub boot and initial IPC snapshot");
    await click("#new-journey");
    await waitFor("document.querySelector('#feedback').textContent.includes('mock reset rejection') && !document.querySelector('#new-journey').disabled", "failed New Journey to restore menu controls");
    let current = await state();
    assert.equal(current.view, "hub");
    assert.equal(current.hubHidden, false);
    assert.equal(current.journeyHidden, true);
    assert.equal(current.newDisabled, false, "New Journey is re-enabled after the mocked IPC failure");
    assert.equal(current.continueDisabled, true, "Continue remains unavailable without a valid save");

    await click("#new-journey");
    await waitFor("document.querySelector('.shell').dataset.view === 'journey' && document.querySelector('#hud-world').textContent === '灰巢设施' && window.__hubLifecycleMock.rendered?.worldEpoch === 2", "first successful New Journey reaches the renderer");
    current = await state();
    assert.equal(current.hubHidden, true);
    assert.equal(current.journeyHidden, false);
    const assertRenderedJourney = (current, epoch) => {
      assert.equal(current.rendered?.worldId, "grey_hive");
      assert.equal(current.rendered?.sceneId, "gh_entry");
      assert.equal(current.rendered?.worldEpoch, epoch, "the current journey was actually rendered");
      assert.equal(current.hasSceneLabel, false, "internal scene label stays absent from ordinary HUD");
      assert.equal(typeof current.hudText, "string");
      assert.doesNotMatch(current.hudText, /gh_entry|rs_core_room|grey_hive|return_station/,
        "ordinary HUD must not expose internal world or scene IDs");
    };
    assertRenderedJourney(current, 2);

    await click("#hud-pause");
    await waitFor("document.querySelector('#pause-overlay').hidden === false && document.querySelector('#save-new-slot').disabled === false", "pause receipt and save controls");
    await click("#save-new-slot");
    await waitFor("document.querySelector('#save-feedback').textContent.startsWith('已保存')", "new save slot receipt");
    assert.match((await state()).saveFeedback, /已保存「现场记录」/);

    await click("#back-to-hub");
    await waitFor("document.querySelector('.shell').dataset.view === 'hub' && document.querySelector('#new-journey').disabled === false && document.querySelector('#continue-journey').disabled === false", "return to hub with save available");
    current = await state();
    assert.equal(current.hubHidden, false);
    assert.equal(current.journeyHidden, true);

    await click("#new-journey");
    await waitFor("document.querySelector('.shell').dataset.view === 'journey' && document.querySelector('#hud-world').textContent === '灰巢设施' && window.__hubLifecycleMock.rendered?.worldEpoch === 4", "second successful New Journey reaches the renderer");
    assertRenderedJourney(await state(), 4);
    await click("#back-to-hub");
    await waitFor("document.querySelector('.shell').dataset.view === 'hub' && document.querySelector('#continue-journey').disabled === false", "second return to hub");

    await click("#continue-journey");
    await waitFor("document.querySelector('#continue-picker').hidden === false && document.querySelector('#continue-slot').options.length === 1", "Continue save picker");
    await evalValue(`(() => { const e=document.querySelector('#continue-slot'); e.value='saved-grey-hive'; e.dispatchEvent(new Event('change',{bubbles:true})); return true; })()`);
    await waitFor("document.querySelector('#continue-selected').disabled === false", "valid saved slot selection");
    await click("#continue-selected");
    await waitFor("document.querySelector('.shell').dataset.view === 'journey' && document.querySelector('#hud-world').textContent === '灰巢设施' && window.__hubLifecycleMock.rendered?.worldEpoch === 6", "Continue from saved journey");
    current = await state();
    assert.equal(current.hubHidden, true);
    assert.equal(current.journeyHidden, false);
    assert.equal(current.newDisabled, true, "hub entry controls stay disabled while inside a journey");
    assertRenderedJourney(current, 6);
    assert.deepEqual(await evalValue("[...new Set(window.__hubLifecycleMock.renderHistory.map(frame => frame.worldEpoch))]"),
      [2, 4, 6], "both new journeys and the restored journey reached the renderer as distinct epochs");

    const calls = await evalValue("window.__hubLifecycleMock.calls.map(x=>x.command)");
    assert.equal(calls.filter(command => command === "formal_new").length, 3, "one failed attempt plus two successful new journeys");
    assert.equal(calls.filter(command => command === "formal_save_slot").length, 1);
    assert.equal(calls.filter(command => command === "formal_return").length, 2);
    assert.equal(calls.filter(command => command === "formal_continue_slot").length, 1);
    assert.equal(await evalValue("window.__hubLifecycleMock.saved.worldId"), "grey_hive");
  }, () => session.cleanup());
});
