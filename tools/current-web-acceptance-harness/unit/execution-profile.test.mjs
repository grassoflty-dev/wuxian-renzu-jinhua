import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { executionProfile, validateFingerprint, readGraphicsFingerprint, readActualRendererFingerprint, actualRendererProbe } from "../src/execution-profile.mjs";
const read = path => readFileSync(new URL(path, import.meta.url), "utf8").replace(/\r\n/g, "\n");
const strict = executionProfile(undefined, "functional");
const software = executionProfile("windows-edge-swiftshader-functional", "functional");
test("strict default remains 60s/15s and only explicitly selected functional software CI uses 180s/60s", () => {
  assert.deepEqual([strict.testTimeoutMs, strict.expectTimeoutMs, strict.actionTimeoutMs], [60_000, 15_000, 0]);
  assert.deepEqual([software.testTimeoutMs, software.expectTimeoutMs, software.actionTimeoutMs], [180_000, 60_000, 60_000]);
  for (const mode of ["visual", "candidate", undefined]) {
    assert.equal(executionProfile(undefined, mode), strict);
    assert.throws(() => executionProfile(software.name, mode), /FUNCTIONAL_ONLY/);
  }
  for (const name of ["", "automatic", "fast", "native"]) assert.throws(() => executionProfile(name, "functional"), /UNKNOWN_EXECUTION_PROFILE/);
  for (const profile of [strict, software]) { assert.equal(profile.performanceAcceptance, false); assert.equal(profile.nativeGameplayAcceptance, false); }
});
test("software profile fails closed without actual Windows Edge SwiftShader DPR1 fingerprint", () => {
  const fingerprint = { platform: "win32", browserChannel: "msedge", deviceScaleFactor: 1,
    graphics: { renderer: "ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero)), SwiftShader driver)" } };
  validateFingerprint(software, fingerprint);
  for (const candidate of [null, {}, { ...fingerprint, platform: "linux" }, { ...fingerprint, browserChannel: "chromium" },
    { ...fingerprint, deviceScaleFactor: .5 }, { ...fingerprint, graphics: null }, { ...fingerprint, graphics: { renderer: "NVIDIA" } }]) {
    assert.throws(() => validateFingerprint(software, candidate), /SOFTWARE_FINGERPRINT_REQUIRED/);
  }
});
test("graphics fingerprint observes public parameters without drawing, scheduler or input mutation", () => {
  let parameters = 0; const gl = { VERSION: 3, MAX_TEXTURE_SIZE: 4,
    getExtension(name) { assert.equal(name, "WEBGL_debug_renderer_info"); return { UNMASKED_VENDOR_WEBGL: 1, UNMASKED_RENDERER_WEBGL: 2 }; },
    getParameter(key) { parameters++; return [null, "vendor", "SwiftShader", "WebGL2", 8192][key]; } };
  const canvas = { getContext(type) { assert.equal(type, "webgl2"); return gl; } };
  assert.deepEqual(readGraphicsFingerprint(canvas), { vendor: "vendor", renderer: "SwiftShader", version: "WebGL2", maxTextureSize: 8192 });
  assert.equal(parameters, 4); assert.equal(readGraphicsFingerprint(null), null);
  assert.doesNotMatch(readGraphicsFingerprint.toString(), /requestAnimationFrame|setInterval|drawElements|formal_submit_input|\.width\s*=|\.height\s*=/);
});
test("all prior functional assertion expressions remain after the documented independent-flow counter resets", () => {
  // Only the positive input-readiness transport changed. Normalize those three
  // named observer calls to their old read expression; every comparator,
  // threshold and all negative direct-count checks must remain verbatim.
  const source = read("../tests/current-web.spec.mjs")
    .replace(/observedInputCount\(page, (firstInput|continuedInput|continueCheckpoint)\)/g, 'callCount(page, "formal_submit_input")')
    .replace(/\s+/g, " ");
  const manifest = JSON.parse(read("./functional-assertions.json"));
  assert.equal(manifest.sourceGitSha, "6eb14893bd4d0321defd410e630201293be7f425");
  assert.ok(Object.keys(manifest.assertions).length > 50);
  for (const [expression, minimum] of Object.entries(manifest.assertions)) {
    assert.ok(source.split(expression).length - 1 >= minimum, `retained assertion (${minimum}): ${expression}`);
  }
  assert.equal((source.match(/test\("/g) ?? []).length, 11);
  assert.doesNotMatch(source, /test\.(skip|fixme|only)|test\.setTimeout|\.slow\(/);
});
test("cold 1440p, all resize resolutions, fresh input, negative paths and real production boundaries remain", () => {
  const source = read("../tests/current-web.spec.mjs"); const helpers = read("../tests/helpers.mjs");
  assert.match(source, /setViewportSize\(\{ width: 2560, height: 1440 \}\);\n  await boot/);
  assert.match(source, /\[\{ width: 1280, height: 720 \}, \{ width: 1920, height: 1080 \}, \{ width: 2560, height: 1440 \}, \{ width: 1280, height: 720 \}\]/);
  assert.match(helpers, /const checkpoint = await inputCheckpoint\(page\)/);
  assert.match(helpers, /const before = checkpoint.inputCount/);
  assert.match(helpers, /observedInputCount\(page, checkpoint\)/);
  assert.match(helpers, /toBeGreaterThan\(before\)/);
  assert.match(source, /toBeGreaterThan\(inputsBeforeContinue\)/);
  assert.match(source, /世界操作超时，需退出并重启此程序。", \{ timeout: 20_000 \}/);
  const client = read("../../../apps/web/src/bridge/tauri-client.ts");
  assert.match(client, /NEW_JOURNEY_TIMEOUT_MS = 15_000/); assert.match(client, /CONTINUE_SLOT_TIMEOUT_MS = 15_000/);
  assert.match(helpers, /final mock and renderer observation must be available/);
  assert.match(helpers, /validateFingerprint\(profile, evidence.fingerprint\)/);
  assert.match(helpers, /performanceAcceptance: false, nativeGameplayAcceptance: false/);
});
test("only explicit current-Web functional CI receives the software profile and Edge/trace/FPS are not overridden", () => {
  const config = read("../playwright.config.mjs"); const workflow = read("../../../.github/workflows/ci.yml");
  assert.equal(workflow.split("CURRENT_WEB_EXECUTION_PROFILE:").length - 1, 1);
  const web = workflow.split("\n  current_web:\n")[1].split("\n  baseline:\n")[0];
  assert.match(web, /CURRENT_WEB_EXECUTION_PROFILE: windows-edge-swiftshader-functional/);
  assert.match(config, /timeout: profile.testTimeoutMs, expect: \{ timeout: profile.expectTimeoutMs \}/);
  assert.match(config, /actionTimeout: profile.actionTimeoutMs/);
  assert.match(config, /channel: "msedge", headless: true, deviceScaleFactor: 1/);
  assert.match(config, /snapshots: false/); assert.match(config, /screenshot: "only-on-failure"/);
  assert.doesNotMatch(config, /launchOptions|--use-gl|--disable-gpu|test\.skip|fps|maxFPS/);
  assert.match(read("../src/run.mjs"), /executionProfile\(process.env.CURRENT_WEB_EXECUTION_PROFILE, mode\)/);
});

test("actual-renderer evidence never creates a world-canvas context before entry or after return", () => {
  let calls = 0; let inputs = 0; let journey = true;
  const canvas = { getContext() { calls++; throw new Error("observer must not create a context"); } };
  const host = { devicePixelRatio: 1, __CURRENT_WEB_MOCK__: { state: { ack: 0 }, count(command) { assert.equal(command, "formal_submit_input"); return inputs; } },
    document: { querySelector(selector) { return selector === "#world-canvas" ? canvas : journey ? {} : null; } } };
  assert.equal(readActualRendererFingerprint(host, readGraphicsFingerprint).status, "not_started");
  inputs = 10; journey = false;
  assert.equal(readActualRendererFingerprint(host, readGraphicsFingerprint).status, "inactive");
  assert.equal(calls, 0);
  journey = true;
  // Lifetime counts are stale across a new epoch and a replacement canvas.
  assert.equal(readActualRendererFingerprint(host, readGraphicsFingerprint).status, "not_started");
  assert.equal(calls, 0);
  host.__CURRENT_WEB_MOCK__.state.ack = 1;
  assert.deepEqual(readActualRendererFingerprint(host, actual => { assert.equal(actual, canvas); return { renderer: "SwiftShader" }; }),
    { status: "started", deviceScaleFactor: 1, graphics: { renderer: "SwiftShader" } });
  inputs = 0;
  assert.equal((0, eval)(actualRendererProbe())(host).status, "not_started");
  assert.equal(calls, 0);
});
