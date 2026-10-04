import { test, expect } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { makeSyntheticView, syntheticBootstrapScript } from "../src/synthetic-ipc.mjs";
import { appendRunEvidence, sampleFrameTimes, saveCanvasEvidence, saveScreenshotEvidence } from "../src/capture-evidence.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "../../..");
const evidenceRoot = path.join(repoRoot, "artifacts/acceptance/pivot-presentation-harness-01/preflight");
const playwrightTestVersion = JSON.parse(readFileSync(path.resolve(here, "../package.json"), "utf8")).devDependencies["@playwright/test"];

for (const viewport of [{ width: 1280, height: 720 }, { width: 1920, height: 1080 }]) {
  test(`synthetic preflight boot and Pixi capture ${viewport.width}x${viewport.height}`, async ({ browser }, testInfo) => {
    const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
    const page = await context.newPage();
    const externalRequests = [];
    const allRequests = [];
    page.on("request", request => {
      const url = new URL(request.url());
      allRequests.push(request.url());
      if (url.origin !== "http://127.0.0.1:4173") externalRequests.push(request.url());
    });
    await page.addInitScript({ content: syntheticBootstrapScript() });
    await page.goto("http://127.0.0.1:4173/", { waitUntil: "networkidle" });
    await expect(page.locator("#hubView")).toBeVisible();
    await expect.poll(() => page.evaluate(() => window.PIXI?.VERSION)).toBe("8.21.0");
    await expect(page.locator("#hubSubtitle")).toContainText("Rust FormalRuntime 已连接");
    const hubLayout = await page.evaluate(() => ({
      innerWidth, innerHeight,
      scrollWidth: document.documentElement.scrollWidth,
      scrollHeight: document.documentElement.scrollHeight
    }));
    expect(hubLayout.scrollWidth).toBeLessThanOrEqual(viewport.width);
    const hubLayoutFindings = hubLayout.scrollHeight > viewport.height
      ? [`Hub vertical overflow: document is ${hubLayout.scrollHeight}px tall in a ${viewport.height}px viewport.`]
      : [];
    const browserVersion = browser.version();
    const hubPath = path.join(evidenceRoot, `${viewport.width}x${viewport.height}-hub-synthetic.png`);
    const hubEvidence = await saveScreenshotEvidence(page, hubPath, "synthetic-hub-preflight");

    await page.locator("#newGameButton").click();
    await expect(page.locator("#worldView")).toBeVisible();
    await expect(page.locator("#connectionBadge")).toContainText("Rust 权威运行时");
    await expect(page.locator(".boot-placeholder-tag")).toContainText("BOOT PLACEHOLDER");
    const canvas = await page.locator("#gameCanvas").evaluate(element => {
      const rect = element.getBoundingClientRect();
      return { x: rect.x, y: rect.y, width: rect.width, height: rect.height, backingWidth: element.width, backingHeight: element.height };
    });
    expect(canvas.width).toBeGreaterThan(100);
    expect(canvas.height).toBeGreaterThan(100);
    expect(canvas.backingWidth).toBeGreaterThan(100);
    expect(canvas.backingHeight).toBeGreaterThan(100);
    const dimensions = await page.evaluate(() => ({
      innerWidth,
      innerHeight,
      scrollWidth: document.documentElement.scrollWidth,
      scrollHeight: document.documentElement.scrollHeight
    }));
    expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.innerWidth);
    expect(dimensions.scrollHeight).toBeLessThanOrEqual(dimensions.innerHeight);

    const worldPath = path.join(evidenceRoot, `${viewport.width}x${viewport.height}-world-synthetic.png`);
    const worldEvidence = await saveScreenshotEvidence(page, worldPath, "synthetic-world-preflight");
    expect(worldEvidence.sampledDistinctRgba).toBeGreaterThan(8);
    const canvasPath = path.join(evidenceRoot, `${viewport.width}x${viewport.height}-canvas-synthetic.png`);
    const canvasEvidence = await saveCanvasEvidence(page, canvasPath, "synthetic-canvas-preflight");
    const frameTimes = await sampleFrameTimes(page);
    expect(frameTimes.frames).toBeGreaterThan(10);
    expect(externalRequests).toEqual([]);
    const babylonRequests = allRequests.filter(url => /babylon/i.test(url));
    expect(babylonRequests).toEqual([]);
    const report = {
      status: "synthetic-preflight-only",
      acceptance: false,
      viewport,
      deviceScaleFactor: 1,
      playwrightTestVersion,
      edgeVersion: browserVersion,
      pixiVersion: await page.evaluate(() => window.PIXI.VERSION),
      hubLayout,
      hubLayoutFindings,
      canvas,
      dimensions,
      frameTimes,
      network: { externalRequests, babylonRequests },
      screenshots: [hubEvidence, worldEvidence, canvasEvidence],
      sourceFixture: "inline synthetic IPC; not the required committed Scenario Runner replay"
    };
    await appendRunEvidence(path.join(evidenceRoot, `${viewport.width}x${viewport.height}-summary.json`), report);
    await context.close();
    testInfo.annotations.push({ type: "classification", description: "Synthetic readiness smoke only; not scenario acceptance or art approval." });
  });
}
