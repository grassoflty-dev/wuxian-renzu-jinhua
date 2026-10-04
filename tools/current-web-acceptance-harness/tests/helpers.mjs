import { test as base, expect } from "@playwright/test";
import { createHash, randomUUID } from "node:crypto";
import os from "node:os";
import { mockBootstrapScript } from "../src/mock-ipc.mjs";
import { passiveObserverScript } from "../src/passive-observer.mjs";
import { createInputProgressStore, INPUT_PROGRESS_BINDING } from "../src/input-progress.mjs";
const inputObservers = new WeakMap();

import { executionProfile, validateFingerprint, readGraphicsFingerprint, actualRendererProbe } from "../src/execution-profile.mjs";
const profile = executionProfile(process.env.CURRENT_WEB_EXECUTION_PROFILE, process.env.CURRENT_WEB_MODE);

export { expect };
export const test = base.extend({
  evidence: [async ({ page, browser }, use, testInfo) => {
    const streamId = randomUUID();
    const inputProgress = createInputProgressStore(streamId);
    inputObservers.set(page, { streamId, inputProgress });
    const dispose = () => { inputProgress.dispose(); inputObservers.delete(page); };
    try {
      await page.exposeBinding(INPUT_PROGRESS_BINDING, (source, record) => {
        if (source.page === page && source.frame === page.mainFrame()) inputProgress.accept(record);
      });
      page.once("close", dispose);
      const errors = [];
      const requests = [];
      const network = [];
      const consoleMessages = [];
      const startedAt = Date.now();
      await page.route("**/*", async route => {
        const url = new URL(route.request().url());
        if (["http:", "https:"].includes(url.protocol) && url.origin !== "http://127.0.0.1:4174") {
          errors.push(`external request forbidden: ${url.href}`);
          await route.abort("blockedbyclient");
        } else await route.continue();
      });
      page.on("pageerror", error => errors.push(error.message));
      page.on("console", message => {
        if (consoleMessages.length < 200) consoleMessages.push({ type: message.type(), text: message.text(), elapsedMs: Date.now() - startedAt });
      });
      page.on("requestfailed", request => errors.push(`request failed: ${request.failure()?.errorText} ${request.url()}`));
      page.on("response", response => {
        network.push({ event: "response", url: response.url(), status: response.status(), elapsedMs: Date.now() - startedAt });
        if (response.status() >= 400) errors.push(`HTTP ${response.status()} ${response.url()}`);
      });
      page.on("request", request => {
        requests.push(request.url());
        network.push({ event: "request", url: request.url(), elapsedMs: Date.now() - startedAt });
      });
      const response = await page.request.get("/__current-web-harness__/identity");
      expect(response.ok()).toBeTruthy();
      const identity = await response.json();
      expect(identity.servedRoot).toBe("apps/web/dist");
      expect(identity.evidenceKind).toBe("current-web-production-bundle-mocked-ipc");
      expect(identity.nativeGameplayAcceptance).toBe(false);
      const evidence = { identity, browserVersion: browser.version(), platform: process.platform,
        osRelease: os.release(), arch: os.arch(), cpuCount: os.cpus().length,
        cpuModel: os.cpus()[0]?.model, totalMemoryBytes: os.totalmem(), executionProfile: profile,
        performanceAcceptance: false, nativeGameplayAcceptance: false, requests, network, consoleMessages, errors };
      await use(evidence);
      const observed = await page.evaluate(source => {
        const publisher = window.__CURRENT_WEB_INPUT_OBSERVER__;
        const inputObserver = publisher?.diagnostics();
        publisher?.dispose();
        return {
        inputObserver,
        actualRenderer: (0, eval)(source)(window),
        mock: window.__CURRENT_WEB_MOCK__?.captureEvidence(),
        timing: window.__CURRENT_WEB_PASSIVE__?.state,
        canvas: (() => { const canvas = document.querySelector("#world-canvas");
          return canvas ? { width: canvas.width, height: canvas.height } : null; })(),
      }; }, actualRendererProbe()).catch(() => null);
      const mock = observed?.mock;
      // A missing observation cannot make unexpected-command checks silently pass.
      await testInfo.attach("current-web-mocked-ipc-evidence", { body: JSON.stringify({ ...evidence, ...observed,
        acceptedInputObserver: inputProgress.diagnostics() }, null, 2), contentType: "application/json" });
      expect(observed?.mock, "final mock and renderer observation must be available").toBeTruthy();
      if (observed?.actualRenderer?.status === "started") validateFingerprint(profile, { ...observed.actualRenderer, platform: process.platform, browserChannel: "msedge" });
      expect(errors, "production bundle must not produce page or HTTP errors").toEqual([]);
      expect(mock?.unexpectedCount ?? 0, "all unexpected commands remain counted even if examples are capped").toBe(0);
      expect(mock?.unexpected ?? [], "unmapped native commands are failures, never silently successful").toEqual([]);
      expect(mock?.observerErrors ?? 0, "accepted-input observation must not throw").toBe(0);
      expect(observed?.inputObserver?.failures ?? 0, "passive notification transport must not reject").toBe(0);
    } finally { page.off("close", dispose); dispose(); }
  }, { auto: true }],
});

export async function boot(page, evidence, options = {}) {
  const observer = inputObservers.get(page);
  if (!observer) throw new Error("E_INPUT_OBSERVER_NOT_INSTALLED");
  await page.addInitScript({ content: mockBootstrapScript(options, observer.streamId) });
  await page.addInitScript({ content: passiveObserverScript() });
  await page.goto("/");
  await expect(page.locator("#new-journey")).toBeEnabled();
  await expect(page.locator("#connection-label")).toHaveText("归航链路已连接");
  await expect(page.locator("#continue-slot-note")).toHaveText(options.withSlots === false
    ? "当前没有可继续的有效存档。" : "旧版只读档会在明确选择续玩时由 Rust 备份迁移。");
  const graphics = await page.evaluate(source => {
    const canvas = document.createElement("canvas");
    try { return (0, eval)(`(${source})`)(canvas); }
    finally { (canvas.getContext("webgl2") || canvas.getContext("webgl"))?.getExtension("WEBGL_lose_context")?.loseContext(); }
  }, readGraphicsFingerprint.toString());
  evidence.fingerprint = { platform: process.platform, browserChannel: "msedge", browserVersion: evidence.browserVersion,
    deviceScaleFactor: await page.evaluate(() => devicePixelRatio), graphics, origin: "temporary-preflight-canvas" };
  validateFingerprint(profile, evidence.fingerprint);
  const entryUrl = new URL(evidence.identity.entry.path, page.url()).href;
  expect(evidence.requests).toContain(entryUrl);
  expect(evidence.requests.filter(url => /\/src\/|\/release-ui\//.test(url))).toEqual([]);
  const entry = await page.request.get(entryUrl);
  expect(createHash("sha256").update(await entry.body()).digest("hex")).toBe(evidence.identity.entry.sha256);
}

export async function calls(page, command) {
  return page.evaluate(command => window.__CURRENT_WEB_MOCK__.callsFor(command), command);
}

export async function callCount(page, command) {
  return page.evaluate(command => window.__CURRENT_WEB_MOCK__.count(command), command);
}

export async function control(page, method, command, message) {
  await page.evaluate(({ method, command, message }) => window.__CURRENT_WEB_MOCK__[method](command, message), { method, command, message });
}

export async function actualRenderer(page) {
  const fingerprint = await page.evaluate(source => (0, eval)(source)(window), actualRendererProbe());
  expect(fingerprint.status, "actual-canvas observation requires real production entry and input").toBe("started");
  validateFingerprint(profile, { ...fingerprint, platform: process.platform, browserChannel: "msedge" });
  return fingerprint;
}

export async function inputCheckpoint(page) {
  const observer = inputObservers.get(page);
  if (!observer) throw new Error("E_INPUT_OBSERVER_NOT_INSTALLED");
  // One idle-phase read before dispatch; arm the current document generation.
  // No transport polling is needed while rendering.
  const checkpoint = await page.evaluate(() => window.__CURRENT_WEB_MOCK__.inputCheckpoint());
  return observer.inputProgress.arm(checkpoint);
}

export function observedInputCount(page, checkpoint) {
  const observer = inputObservers.get(page);
  if (!observer) throw new Error("E_INPUT_OBSERVER_NOT_INSTALLED");
  return observer.inputProgress.countAfter(checkpoint);
}

export async function enterNew(page) {
  const checkpoint = await inputCheckpoint(page);
  const before = checkpoint.inputCount;
  await page.locator("#new-journey").click();
  await expect(page.locator(".shell")).toHaveAttribute("data-view", "journey");
  await expect.poll(async () => await observedInputCount(page, checkpoint)).toBeGreaterThan(before);
  await expect(page.locator("#hud-scene")).toHaveText("rs_core_room");
  await actualRenderer(page);
}

export async function paused(page) {
  await page.locator("#hud-pause").click();
  await expect(page.locator("#pause-title")).toHaveText("旅程已暂停");
  await expect(page.locator("#save-new-slot")).toBeEnabled();
}

export async function noDocumentScroll(page) {
  const metrics = await page.evaluate(() => ({ w: document.documentElement.scrollWidth,
    h: document.documentElement.scrollHeight, cw: document.documentElement.clientWidth, ch: document.documentElement.clientHeight }));
  expect(metrics.w).toBeLessThanOrEqual(metrics.cw);
  expect(metrics.h).toBeLessThanOrEqual(metrics.ch);
}

export async function inViewport(page, selector) {
  const box = await page.locator(selector).boundingBox();
  expect(box, `${selector} must be visible`).not.toBeNull();
  const viewport = page.viewportSize();
  expect(box.x).toBeGreaterThanOrEqual(-1); expect(box.y).toBeGreaterThanOrEqual(-1);
  expect(box.x + box.width).toBeLessThanOrEqual(viewport.width + 1);
  expect(box.y + box.height).toBeLessThanOrEqual(viewport.height + 1);
}
