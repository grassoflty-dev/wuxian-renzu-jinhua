import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
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

async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => server.once("error", reject).listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function waitForPage(port, processHandle) {
  const end = Date.now() + 15_000;
  while (Date.now() < end) {
    if (processHandle.exitCode !== null) throw new Error(`Edge exited with code ${processHandle.exitCode}`);
    try {
      const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const page = pages.find((target) => target.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    await delay(100);
  }
  throw new Error("Timed out waiting for Edge DevTools");
}

async function connectCdp(url) {
  const socket = new WebSocket(url);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  let nextId = 0;
  const pending = new Map();
  socket.addEventListener("message", ({ data }) => {
    const message = JSON.parse(data);
    if (!message.id) return;
    const callbacks = pending.get(message.id);
    if (!callbacks) return;
    pending.delete(message.id);
    if (message.error) callbacks.reject(new Error(message.error.message));
    else callbacks.resolve(message.result);
  });
  return {
    send(method, params = {}) {
      const id = ++nextId;
      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
    close() { socket.close(); },
  };
}

function installMockIpc() {
  const state = {
    calls: [], saved: null, latest: null, epoch: 1, tick: 1, newFailures: 1,
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
    async render() {}
    reconcileSoundCue() {}
    acceptSoundCues() {}
    clearSoundCues() {}
    cancel() {}
    async destroy() {}
  }`,
};

test("hub lifecycle: two new journeys, save, return, continue, and recover after IPC failure", { timeout: 120_000 }, async (t) => {
  const edge = findEdge();
  if (!edge) return t.skip("Microsoft Edge is not installed at its standard Windows path");

  const profile = await mkdtemp(path.join(os.tmpdir(), "hub-lifecycle-regression-"));
  const [debugPort, vitePort] = await Promise.all([freePort(), freePort()]);
  const vite = await createServer({
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
  });
  const browser = spawn(edge, [
    "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
    `--remote-debugging-port=${debugPort}`, `--user-data-dir=${profile}`, "about:blank",
  ], { stdio: "ignore", windowsHide: true });
  let cdp;
  try {
    await vite.listen();
    cdp = await connectCdp(await waitForPage(debugPort, browser));
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
      throw new Error(`Timed out waiting for ${description}`);
    };
    const click = async (selector) => evalValue(`(() => { const e=document.querySelector(${JSON.stringify(selector)}); if(!e) throw new Error("missing ${selector}"); e.click(); return true; })()`);
    const state = async () => evalValue(`(() => ({
      view: document.querySelector(".shell")?.dataset.view,
      hubHidden: document.querySelector("#hub")?.hidden,
      journeyHidden: document.querySelector("#journey")?.hidden,
      world: document.querySelector("#hud-world")?.textContent,
      scene: document.querySelector("#hud-scene")?.textContent,
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
    await waitFor("document.querySelector('.shell').dataset.view === 'journey' && document.querySelector('#hud-world').textContent === '灰巢设施'", "first successful New Journey");
    current = await state();
    assert.equal(current.hubHidden, true);
    assert.equal(current.journeyHidden, false);
    assert.equal(current.scene, "gh_entry");

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
    await waitFor("document.querySelector('.shell').dataset.view === 'journey' && document.querySelector('#hud-world').textContent === '灰巢设施'", "second successful New Journey");
    await click("#back-to-hub");
    await waitFor("document.querySelector('.shell').dataset.view === 'hub' && document.querySelector('#continue-journey').disabled === false", "second return to hub");

    await click("#continue-journey");
    await waitFor("document.querySelector('#continue-picker').hidden === false && document.querySelector('#continue-slot').options.length === 1", "Continue save picker");
    await evalValue(`(() => { const e=document.querySelector('#continue-slot'); e.value='saved-grey-hive'; e.dispatchEvent(new Event('change',{bubbles:true})); return true; })()`);
    await waitFor("document.querySelector('#continue-selected').disabled === false", "valid saved slot selection");
    await click("#continue-selected");
    await waitFor("document.querySelector('.shell').dataset.view === 'journey' && document.querySelector('#hud-world').textContent === '灰巢设施' && document.querySelector('#hud-scene').textContent === 'gh_entry'", "Continue from saved journey");
    current = await state();
    assert.equal(current.hubHidden, true);
    assert.equal(current.journeyHidden, false);
    assert.equal(current.newDisabled, true, "hub entry controls stay disabled while inside a journey");

    const calls = await evalValue("window.__hubLifecycleMock.calls.map(x=>x.command)");
    assert.equal(calls.filter(command => command === "formal_new").length, 3, "one failed attempt plus two successful new journeys");
    assert.equal(calls.filter(command => command === "formal_save_slot").length, 1);
    assert.equal(calls.filter(command => command === "formal_return").length, 2);
    assert.equal(calls.filter(command => command === "formal_continue_slot").length, 1);
    assert.equal(await evalValue("window.__hubLifecycleMock.saved.worldId"), "grey_hive");
  } finally {
    try { await cdp?.send("Browser.close"); } catch {}
    if (browser.exitCode === null && process.platform === "win32") {
      spawnSync("taskkill", ["/PID", String(browser.pid), "/T", "/F"], { windowsHide: true, stdio: "ignore" });
    } else if (browser.exitCode === null) browser.kill();
    if (browser.exitCode === null) await Promise.race([
      new Promise((resolve) => browser.once("exit", resolve)), delay(1_000),
    ]);
    cdp?.close();
    await vite.close();
    for (let attempt = 0; attempt < 5; attempt++) {
      try {
        await rm(profile, { recursive: true, force: true, maxRetries: 3, retryDelay: 200 });
        break;
      } catch (error) {
        if (attempt === 4) throw error;
        await delay(200);
      }
    }
  }
});
