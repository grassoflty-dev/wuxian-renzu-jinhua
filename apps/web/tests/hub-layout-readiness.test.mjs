import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { waitForStableHubLayout } from "./support/hub-layout-readiness.mjs";

function host({ sizes = [[1024, 537]], heights = [537], fontReady = Promise.resolve(), fonts = "loaded", noFrames = false, frameTimes = null } = {}) {
  let index = -1; let now = 0; let callbacks = 0; let cancelled = 0;
  const pending = new Map(); let next = 0;
  const result = {
    performance: { now: () => now }, setTimeout, clearTimeout, Promise,
    get innerWidth() { return sizes[Math.max(0, Math.min(index, sizes.length - 1))][0]; },
    get innerHeight() { return sizes[Math.max(0, Math.min(index, sizes.length - 1))][1]; },
    requestAnimationFrame(callback) {
      const id = ++next;
      if (!noFrames) pending.set(id, setImmediate(() => { pending.delete(id); index++; now = frameTimes ? frameTimes[Math.min(index, frameTimes.length - 1)] : now + 16; callbacks++; callback(now); }));
      return id;
    },
    cancelAnimationFrame(id) { cancelled++; clearImmediate(pending.get(id)); pending.delete(id); },
    getComputedStyle: () => ({ paddingTop: "58px", paddingBottom: "27px", gridTemplateColumns: "400px 450px", alignContent: "center", overflowY: "auto", fontSize: "35.84px", lineHeight: "39.424px", fontFamily: "Microsoft YaHei UI" }),
    matchMedia: () => ({ matches: true }),
    document: { fonts: { status: fonts, ready: fontReady }, querySelector() { return {
      get clientWidth() { return result.innerWidth; }, get clientHeight() { return result.innerHeight; },
      get scrollHeight() { return heights[Math.max(0, Math.min(index, heights.length - 1))]; },
      getBoundingClientRect: () => ({ top: 0, bottom: 100, left: 0, right: 300 }),
    }; } },
    counts: () => ({ callbacks, cancelled }),
  };
  return result;
}
const run = (context, expected = { width: 1024, height: 537 }, budget = 2000) => vm.runInNewContext(
  `(${waitForStableHubLayout.toString()})(${JSON.stringify(expected)}, ${budget})`, context);

test("readiness waits for requested dimensions and two unchanged consecutive frames", async () => {
  const context = host({ sizes: [[1280, 672], [1024, 537]], heights: [650, 574, 500, 500] });
  const result = await run(context);
  assert.equal(result.ready, true); assert.equal(result.frames, 4); assert.equal(result.stableFrames, 2);
  assert.equal(result.last.viewport.height, 537); assert.equal(result.last.hub.scrollHeight, 500);
  assert.equal(result.last.computed.paddingTop, "58px");
});

test("a stable574px overflow is returned unchanged and still fails the original fit condition", async () => {
  const result = await run(host({ heights: [574, 574] }));
  assert.equal(result.ready, true); assert.equal(result.last.hub.height, 537); assert.equal(result.last.hub.scrollHeight, 574);
  assert.equal(result.last.hub.scrollHeight <= result.last.hub.height + 1, false);
});

test("font readiness must resolve before any layout frame is measured", async () => {
  let resolve; const context = host({ fontReady: new Promise(r => { resolve = r; }), fonts: "loading" });
  const pending = run(context); await new Promise(r => setImmediate(r));
  assert.equal(context.counts().callbacks, 0);
  context.document.fonts.status = "loaded"; resolve();
  assert.equal((await pending).ready, true);
});

test("unresolved fonts and suppressed animation frames fail within the finite budget and clean up", async () => {
  for (const context of [host({ fontReady: new Promise(() => {}), fonts: "loading" }), host({ noFrames: true })]) {
    const result = await run(context, undefined, 15);
    assert.equal(result.ready, false); assert.equal(result.reason, "layout-readiness-deadline");
  }
});

test("wrong dimensions never become ready just because geometry stops changing", async () => {
  const result = await run(host({ sizes: [[1280, 720]] }), undefined, 48);
  assert.equal(result.ready, false); assert.equal(result.stableFrames, 0);
});

test("readiness cannot change DOM, font, dimensions, overflow or the existing assertion budgets", () => {
  const source = waitForStableHubLayout.toString();
  assert.doesNotMatch(source, /setDeviceMetricsOverride|style\.[a-zA-Z]+\s*=|\.innerHTML\s*=|\.textContent\s*=|\.scrollTop\s*=/);
  const testSource = readFileSync(new URL("./hub-viewport-fit.mjs", import.meta.url), "utf8").replace(/\r\n/g, "\n");
  assert.match(testSource, /timeout: 90_000/);
  assert.match(testSource, /awaitPromise: true, returnByValue: true/);
  const assertions = testSource.slice(testSource.indexOf("      const metrics = result.result.value;"), testSource.indexOf("  } finally {"));
  assert.equal(createHash("sha256").update(assertions).digest("hex"), "3370307197674d7d092d95257902cadef767680c696f698bfd2b365365b18eae");
  const sizes = testSource.slice(testSource.indexOf("    const sizes = ["), testSource.indexOf("    for (const size of sizes)"));
  assert.equal(createHash("sha256").update(sizes).digest("hex"), "dbb31b4ee3008c40d23a23c10c385235ec3ad498e9a607e775f036f28cb1f070");
});


test("an overdue frame cannot report ready ahead of its queued deadline timer", async () => {
  const result = await run(host({ frameTimes: [16, 2500] }));
  assert.equal(result.ready, false);
  assert.equal(result.reason, "layout-readiness-deadline");
});

test("font completion after timeout cannot leave a new frame loop running", async () => {
  let resolve; const context = host({ fontReady: new Promise(r => { resolve = r; }) });
  const result = await run(context, undefined, 10);
  assert.equal(result.ready, false);
  resolve(); await new Promise(r => setImmediate(r));
  assert.equal(context.counts().callbacks, 0);
});
