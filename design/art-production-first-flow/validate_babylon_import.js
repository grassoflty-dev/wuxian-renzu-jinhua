/* Reproducible Babylon NullEngine import smoke for the authored GLBs. */
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

// Babylon's browser FileTools path uses XHR. This narrowly scoped shim maps
// file:// reads to Node fs so the production loader can parse the exact files.
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
    for (const handler of this.listeners[name] || []) handler.call(this, event);
    const callback = this["on" + name];
    if (typeof callback === "function") callback.call(this, event);
  }
  send() {
    const filename = fileURLToPath(this.url);
    this.dispatch("loadstart");
    fs.readFile(filename, (error, bytes) => {
      if (error) { this.status = 404; this.dispatch("error", { error }); this.dispatch("loadend"); return; }
      this.status = 200;
      this.statusText = "OK";
      this.responseURL = this.url;
      this.response = this.responseType === "arraybuffer"
        ? bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
        : bytes.toString("utf8");
      this.readyState = 4;
      this.dispatch("readystatechange");
      this.dispatch("progress", { loaded: bytes.byteLength, total: bytes.byteLength });
      this.dispatch("load");
      this.dispatch("loadend");
    });
  }
  abort() { this.dispatch("abort"); }
};

async function main() {
  const assetDir = path.join(root, "server-rs/release-ui/assets/grey-hive-first-flow");
  const engine = new B.NullEngine();
  const scene = new B.Scene(engine);
  scene.activeCamera = new B.FreeCamera("validation-camera", new B.Vector3(0, 1, -4), scene);
  scene.activeCamera.setTarget(B.Vector3.Zero());
  const results = [];
  for (const file of ["grey-hive-first-flow-room.glb", "power-console.glb", "gate-a.glb", "field-operative-rig.glb"]) {
    const container = await B.SceneLoader.LoadAssetContainerAsync(
      pathToFileURL(assetDir + path.sep).href,
      file,
      scene,
    );
    for (const skeleton of container.skeletons || []) skeleton.useTextureToStoreBoneMatrices = true;
    container.addAllToScene();
    const result = {
      file,
      meshes: container.meshes.length,
      skeletons: container.skeletons.length,
      animationGroups: container.animationGroups.map((group) => group.name),
    };
    if (file === "field-operative-rig.glb") {
      const expected = ["idle", "walk", "run"];
      const actual = result.animationGroups.map((name) => name.toLowerCase());
      if (container.skeletons.length !== 1 || expected.some((name) => !actual.includes(name))) {
        throw new Error(`Humanoid import contract failed: ${JSON.stringify(result)}`);
      }
      for (const name of expected) {
        const group = container.animationGroups.find((item) => item.name.toLowerCase() === name);
        group.start(true);
        scene.render();
        group.stop();
      }
    }
    results.push(result);
    container.dispose();
  }
  const report = { babylon: B.Engine.Version, loader: !!B.GLTFFileLoader, results };
  const output = JSON.stringify(report, null, 2) + "\n";
  if (process.argv[2]) fs.writeFileSync(path.resolve(process.argv[2]), output);
  process.stdout.write(output);
  scene.dispose();
  engine.dispose();
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
