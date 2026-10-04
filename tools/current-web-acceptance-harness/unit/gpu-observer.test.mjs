import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";
import { installGpuObserver, gpuObserverScript } from "../src/gpu-observer.mjs";
import { encodeBoundedArtifact, scrubProfile, summarizeProfile, bounded } from "../src/profile-renderer.mjs";
function fixture() {
  let clock = 0;
  class GL {
    constructor(canvas) { this.canvas = canvas; }
    drawElements(...args) { clock += 25; this.last = args; return 41; }
    readPixels() { clock += 30; throw this.error; }
    getExtension() { return { UNMASKED_VENDOR_WEBGL: 1, UNMASKED_RENDERER_WEBGL: 2 }; }
    getParameter(key) { return key === 1 ? "vendor" : key === 2 ? "renderer" : "capability"; }
  }
  class Canvas {
    constructor(id) { this.id = id; this.width = 100; this.height = 50; this.gl = new GL(this); }
    getContext(...args) { this.contextArgs = args; return this.gl; }
  }
  return { host: { WebGLRenderingContext: GL, HTMLCanvasElement: Canvas, performance: { now: () => clock } }, GL, Canvas };
}

test("diagnostic WebGL wrappers preserve calls, receiver, return values and original exceptions", () => {
  const f = fixture(); const original = f.GL.prototype.drawElements;
  const descriptor = Object.getOwnPropertyDescriptor(f.GL.prototype, "drawElements");
  const observer = installGpuObserver(f.host);
  assert.equal(Object.getOwnPropertyDescriptor(f.GL.prototype, "drawElements").enumerable, descriptor.enumerable);
  const canvas = new f.Canvas("world-canvas"); const options = { alpha: false };
  assert.equal(canvas.getContext("webgl2", options), canvas.gl); assert.equal(canvas.contextArgs[1], options);
  assert.equal(canvas.gl.drawElements(1, 2, 3, 4), 41); assert.deepEqual(canvas.gl.last, [1, 2, 3, 4]);
  assert.equal(observer.state.graphics.renderer, "renderer"); assert.equal(observer.state.methods.drawElements.calls, 1);
  assert.equal(observer.state.methods.drawElements.totalMs, 25);
  const error = new Error("native error"); canvas.gl.error = error;
  assert.throws(() => canvas.gl.readPixels(), candidate => candidate === error);
  assert.equal(observer.state.methods.readPixels.errors, 1);
  observer.stop(); assert.equal(f.GL.prototype.drawElements, original);
});

test("only the actual game canvas is timed and diagnostic samples are bounded", () => {
  const f = fixture(); const observer = installGpuObserver(f.host);
  const probe = new f.Canvas("probe"); probe.getContext("webgl"); probe.gl.drawElements();
  assert.equal(observer.state.graphics, null); assert.equal(observer.state.methods.drawElements.calls, 0);
  const game = new f.Canvas("world-canvas"); game.getContext("webgl");
  for (let i = 0; i < 1000; i++) game.gl.drawElements();
  assert.equal(observer.state.methods.drawElements.calls, 1000); assert.equal(observer.state.slowCalls.length, 32);
  assert.ok(observer.state.slowCalls.every(call => call.stack.length <= 1600));
  observer.state.methods.drawElements.calls = Number.MAX_SAFE_INTEGER; game.gl.drawElements();
  assert.equal(observer.state.methods.drawElements.calls, Number.MAX_SAFE_INTEGER); observer.stop();
});

test("self-contained GPU observer tolerates no GL APIs and never installs a render/input scheduler", () => {
  const host = {}; const context = vm.createContext({ window: host });
  vm.runInContext(gpuObserverScript(), context);
  assert.equal(host.__CURRENT_WEB_GPU_PROFILE__.state.instrumentedMethods.length, 0);
  assert.equal("__TAURI_INTERNALS__" in host, false); assert.equal("requestAnimationFrame" in host, false);
  host.__CURRENT_WEB_GPU_PROFILE__.stop();
});

test("CPU summaries preserve samples while URLs and artifact sizes have explicit limits", async () => {
  const raw = { startTime: 0, endTime: 5000, nodes: [{ id: 1, callFrame: { functionName: "draw", url: "http://127.0.0.1:4174/assets/app.js?private=1", lineNumber: 1 } },
    { id: 2, callFrame: { functionName: "injected", url: "https://external.example/secret" } }], samples: [1, 2, 1], timeDeltas: [1000, 2000, 1000] };
  const safe = scrubProfile(raw); assert.equal(safe.nodes[0].callFrame.url, "http://127.0.0.1:4174/assets/app.js");
  assert.equal(safe.nodes[1].callFrame.url, "internal-or-injected-script");
  const summary = summarizeProfile(safe); assert.equal(summary.sampleCount, 3); assert.equal(summary.durationMs, 5);
  assert.equal(summary.topSelfTime.reduce((sum, row) => sum + row.selfMs, 0), 4);
  assert.ok(encodeBoundedArtifact(summary).length < 512 * 1024);
  assert.throws(() => encodeBoundedArtifact({ value: "中文中文" }, 10), /ARTIFACT_SIZE/);
  await assert.rejects(bounded(new Promise(() => {}), 5, "probe"), /E_PROFILE_TIMEOUT:probe/);
});

test("diagnostic workflow uses read-only repository permissions and a separate profile command", async () => {
  const { readFile } = await import("node:fs/promises");
  const workflow = await readFile(new URL("../../../.github/workflows/current-web-renderer-profile.yml", import.meta.url), "utf8");
  assert.match(workflow, /^permissions:\n  contents: read\n/m);
  assert.doesNotMatch(workflow, /contents:\s*write|write-all/);
  assert.match(workflow, /run: node tools\/current-web-acceptance-harness\/src\/profile-renderer\.mjs/);
  assert.doesNotMatch(workflow, /npm run test:browser/);
});
