/* Exercise the TPS adapter against Rust-shaped full-view fixtures in NullEngine. */
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { fileURLToPath, pathToFileURL } = require("node:url");
const root = path.resolve(__dirname, "../..");
const vendor = path.join(root, "server-rs/release-ui/vendor");
require(path.join(vendor, "babylon.js"));
const B = globalThis.BABYLON;
if (B.Logger) B.Logger.LogLevels = 0;
const loaderContext = { BABYLON: B, console, setTimeout, clearTimeout, Promise, Uint8Array, TextEncoder, TextDecoder, URL, Blob };
loaderContext.self = loaderContext;
vm.runInNewContext(fs.readFileSync(path.join(vendor, "babylonjs.loaders.min.js"), "utf8"), loaderContext);
globalThis.XMLHttpRequest = class FileXhr {
  constructor() { this.readyState = 0; this.status = 0; this.responseType = ""; this.listeners = {}; }
  open(_method, url) { this.url = url; this.readyState = 1; }
  setRequestHeader() {}
  addEventListener(name, handler) { (this.listeners[name] ||= []).push(handler); }
  removeEventListener(name, handler) { this.listeners[name] = (this.listeners[name] || []).filter((item) => item !== handler); }
  getResponseHeader(name) { return name.toLowerCase() === "content-type" ? "application/octet-stream" : null; }
  getAllResponseHeaders() { return "content-type: application/octet-stream\r\n"; }
  dispatch(name, extra = {}) {
    const event = { type: name, target: this, lengthComputable: true, loaded: 1, total: 1, ...extra };
    for (const fn of this.listeners[name] || []) fn.call(this, event);
    if (typeof this["on" + name] === "function") this["on" + name].call(this, event);
  }
  send() {
    fs.readFile(fileURLToPath(this.url), (error, bytes) => {
      if (error) { this.status = 404; this.dispatch("error", { error }); return; }
      this.status = 200; this.statusText = "OK"; this.responseURL = this.url;
      this.response = this.responseType === "arraybuffer" ? bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) : bytes.toString("utf8");
      this.readyState = 4; this.dispatch("readystatechange"); this.dispatch("progress", { loaded: bytes.byteLength, total: bytes.byteLength }); this.dispatch("load"); this.dispatch("loadend");
    });
  }
  abort() { this.dispatch("abort"); }
};

class NullRenderEngine extends B.NullEngine {
  runRenderLoop() {}
  stopRenderLoop() {}
}
B.Engine = NullRenderEngine;
const { FormalTpsRenderer } = require(path.join(root, "server-rs/release-ui/js/tps-renderer.js"));
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const transform = (xM, yM, zM) => ({ positionM: { xM, yM, zM }, yawRad: 0 });

async function main() {
  const assetRoot = pathToFileURL(path.join(root, "server-rs/release-ui/assets/grey-hive-first-flow") + path.sep).href;
  const renderer = new FormalTpsRenderer({ canvas: {}, babylon: B, assetRoot });
  const view = {
    worldId: "grey_hive",
    player: { transform: transform(0, 0, 0), velocityMps: { xM: 0, yM: 0, zM: 0 } },
    actors: [{ entityId: "gh_power_console", actorKind: "power_console", transform: transform(2, 0, 3), active: true }],
    doors: [{ doorId: "gh_gate_a", transform: transform(0, 0, 8), open: false, locked: true }],
    capabilities: { rearView: { granted: false } },
  };
  renderer.applyView(view);
  const deadline = Date.now() + 8000;
  while (Date.now() < deadline && (!renderer.roomContainer || !renderer.playerContainer || !renderer.actorMeshes.get("gh_power_console")?.loaded || !renderer.doorMeshes.get("gh_gate_a")?.loaded)) await wait(20);
  if (!renderer.roomContainer || !renderer.playerContainer || !renderer.actorMeshes.get("gh_power_console")?.loaded || !renderer.doorMeshes.get("gh_gate_a")?.loaded) throw new Error("E_FORMAL_VIEW_ASSETS_NOT_READY");
  const actor = renderer.actorMeshes.get("gh_power_console");
  const gate = renderer.doorMeshes.get("gh_gate_a");
  renderer.applyView(view);
  if (!renderer.hiveCoolFill.isEnabled() || !renderer.hiveAmberPractical.isEnabled()) throw new Error("E_GREY_HIVE_LIGHTING");
  if (Math.abs(actor.root.position.x - 2) > 1e-5 || Math.abs(actor.root.position.z - 3) > 1e-5) throw new Error("E_ACTOR_TRANSFORM");
  if (Math.abs(gate.root.position.z - 8) > 1e-5 || gate.slide.position.x !== 0) throw new Error("E_CLOSED_GATE_TRANSFORM");
  renderer.scene.render();
  view.doors[0].open = true;
  view.doors[0].locked = false;
  view.player.velocityMps.zM = 1.2;
  renderer.applyView(view);
  if (gate.slide.position.x !== 3.25 || renderer.activePlayerAnimation?.name !== "Walk") throw new Error("E_OPEN_GATE_OR_WALK_MAPPING");
  view.player.velocityMps.zM = 3.0;
  renderer.applyView(view);
  if (renderer.activePlayerAnimation?.name !== "Run") throw new Error("E_RUN_MAPPING");
  view.player.velocityMps.zM = 0;
  renderer.applyView(view);
  if (renderer.activePlayerAnimation?.name !== "Idle") throw new Error("E_IDLE_MAPPING");
  view.worldId = "mist_harbor";
  renderer.applyView(view);
  if (renderer.hiveCoolFill.isEnabled() || renderer.hiveAmberPractical.isEnabled()) throw new Error("E_NEUTRAL_LIGHTING");
  const result = {
    babylon: B.Engine.Version,
    roomLoaded: Boolean(renderer.roomContainer),
    consoleLoaded: actor.loaded,
    gateLoaded: gate.loaded,
    actorWorld: { x: actor.root.position.x, z: actor.root.position.z },
    gateWorldZ: gate.root.position.z,
    closedSlideX: 0,
    rustOpenSlideX: gate.slide.position.x,
    animationGroups: [...renderer.playerAnimations.keys()],
    selectedByVelocityMps: { "0": "Idle", "1.2": "Walk", "3.0": "Run" },
    lightingWorld: "grey_hive",
    lightingSwitch: "grey_hive cool/amber enabled; non-Hive neutral practicals disabled",
    authority: "input DTO fixture only; no Rust state changed",
  };
  renderer.dispose();
  const output = JSON.stringify(result, null, 2) + "\n";
  if (process.argv[2]) fs.writeFileSync(path.resolve(process.argv[2]), output);
  process.stdout.write(output);
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
