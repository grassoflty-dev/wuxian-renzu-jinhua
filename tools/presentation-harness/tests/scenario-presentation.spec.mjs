import { test, expect } from "@playwright/test";
import path from "node:path";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { loadVerifiedFixture } from "../src/scenario-fixture.mjs";
import { scenarioBootstrapScript } from "../src/scenario-bootstrap.mjs";
import { appendRunEvidence, sampleFrameTimes, saveCanvasEvidence, saveScreenshotEvidence } from "../src/capture-evidence.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "../../..");
const fixtureResult = await loadVerifiedFixture();
const viewports = [{ width: 1280, height: 720 }, { width: 1920, height: 1080 }];
const playwrightTestVersion = JSON.parse(readFileSync(path.resolve(here, "../package.json"), "utf8")).devDependencies["@playwright/test"];

test.describe("committed Scenario Runner presentation fixture", () => {
  for (const viewport of viewports) {
    test(`capture and compare presentation states ${viewport.width}x${viewport.height}`, async ({ browser }, testInfo) => {
      test.skip(!fixtureResult.ready, fixtureResult.reason || "Scenario Runner fixture is pending");
      const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
      const page = await context.newPage();
      const externalRequests = [];
      page.on("request", request => {
        const url = new URL(request.url());
        if (url.origin !== "http://127.0.0.1:4173") externalRequests.push(request.url());
      });
      await page.addInitScript({ content: scenarioBootstrapScript(fixtureResult.fixture) });
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
      await page.addStyleTag({ content: ".toast-region{display:none!important}*,*::before,*::after{animation-duration:0s!important;transition-duration:0s!important}" });
      await page.evaluate(() => {
        const proto = window.FreezeV02PixiRenderer.PixiWorldRenderer.prototype;
        const original = proto.applyView;
        proto.applyView = function(view) {
          window.__PRESENTATION_HARNESS__.rendererInstance = this;
          const result = original.call(this, view);
          window.__PRESENTATION_HARNESS__.renderRecords.push({
            revision: view.authorityRevision,
            playerPosition: { ...view.player.transform.positionM },
            cameraTarget: { ...this.camera.target },
            stageChildren: this.application.stage.children.length,
            canvas: { width: this.application.screen.width, height: this.application.screen.height }
          });
          return result;
        };
      });

      const captures = [];
      const capture = async name => {
        const evidenceDir = path.join(repoRoot, "artifacts/acceptance/pivot-presentation-harness-01/scenarios");
        const screenshotPath = path.join(evidenceDir, `${viewport.width}x${viewport.height}-${name}.png`);
        const screenshot = await saveScreenshotEvidence(page, screenshotPath, name);
        await expect(page).toHaveScreenshot(`${name}-${viewport.width}x${viewport.height}.png`, {
          animations: "disabled",
          maxDiffPixelRatio: 0.001
        });
        let canvas = null;
        if (name !== "hub") {
          canvas = await saveCanvasEvidence(page, path.join(evidenceDir, `${viewport.width}x${viewport.height}-${name}-canvas.png`), `${name}-canvas`);
        }
        captures.push({ screenshot, canvas });
      };

      await capture("hub");
      await page.locator("#newGameButton").click();
      await expect(page.locator("#worldView")).toBeVisible();
      await expect(page.locator("#connectionBadge")).toContainText("Rust 权威运行时");
      await expect(page.locator(".boot-placeholder-tag")).toContainText("BOOT PLACEHOLDER");
      await capture("powerBefore");

      const applyScenarioThroughInput = async name => {
        await page.evaluate(scenario => window.__PRESENTATION_HARNESS__.stageScenario(scenario), name);
        await page.keyboard.down("w");
        await expect.poll(() => page.evaluate(() => window.__PRESENTATION_HARNESS__.lastAppliedScenario)).toBe(name);
        await page.keyboard.up("w");
      };
      await applyScenarioThroughInput("gateBefore");
      await capture("gateBefore");

      await page.keyboard.press("f");
      await expect.poll(() => page.evaluate(() => window.__PRESENTATION_HARNESS__.calls.some(call => call.command === "formal_interact"))).toBe(true);
      await capture("powerAfter");
      await applyScenarioThroughInput("gateAfter");
      await capture("gateAfter");

      const canvasBeforeReject = await saveCanvasEvidence(page, path.join(repoRoot, "artifacts/acceptance/pivot-presentation-harness-01/scenarios", `${viewport.width}x${viewport.height}-before-reject.png`), "before-protocol-rejection");
      const rejection = await page.evaluate(() => window.FormalView.consume({ kind: "full", schemaVersion: "invalid-fixture-schema" }));
      expect(rejection.accepted).toBe(false);
      expect(rejection.reason).toBe("E_SCHEMA_MISMATCH");
      const canvasAfterReject = await saveCanvasEvidence(page, path.join(repoRoot, "artifacts/acceptance/pivot-presentation-harness-01/scenarios", `${viewport.width}x${viewport.height}-after-reject.png`), "after-protocol-rejection");
      expect(canvasAfterReject.sha256).toBe(canvasBeforeReject.sha256);

      const reloadAudit = await page.evaluate(() => {
        const harness = window.__PRESENTATION_HARNESS__;
        const renderer = harness.rendererInstance;
        const view = window.__PRESENTATION_SCENARIO_FIXTURE__.scenarios.gateAfter.view;
        const firstCount = renderer.application.stage.children.length;
        renderer.applyView(view);
        const secondCount = renderer.application.stage.children.length;
        return { firstCount, secondCount, cameraTarget: { ...renderer.camera.target }, playerPosition: { ...view.player.transform.positionM }, renderRecords: harness.renderRecords };
      });
      expect(reloadAudit.firstCount).toBeGreaterThan(0);
      expect(reloadAudit.secondCount).toBe(reloadAudit.firstCount);
      expect(reloadAudit.cameraTarget).toEqual(reloadAudit.playerPosition);

      const frameTimes = await sampleFrameTimes(page);
      expect(frameTimes.frames).toBeGreaterThan(10);
      const pageGeometry = await page.evaluate(() => ({
        innerWidth, innerHeight,
        scrollWidth: document.documentElement.scrollWidth,
        scrollHeight: document.documentElement.scrollHeight,
        canvas: (() => { const r = document.querySelector("#gameCanvas").getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; })()
      }));
      expect(externalRequests).toEqual([]);
      expect(pageGeometry.scrollWidth).toBeLessThanOrEqual(viewport.width);
      expect(pageGeometry.scrollHeight).toBeLessThanOrEqual(viewport.height);
      expect(pageGeometry.canvas.width).toBeGreaterThan(100);
      expect(pageGeometry.canvas.height).toBeGreaterThan(100);
      const runRecord = {
        status: "scenario-fixture-technical-baseline",
        acceptance: false,
        viewport,
        deviceScaleFactor: 1,
        playwrightTestVersion,
        edgeVersion: browser.version(),
        pixiVersion: await page.evaluate(() => window.PIXI.VERSION),
        scenarioRunnerCommit: fixtureResult.metadata.sourceCommit,
        scenarioRunnerFixturePath: fixtureResult.metadata.sourcePath,
        scenarioRunnerFixtureSha256: fixtureResult.actualSha256,
        pageGeometry,
        hubLayout,
        hubLayoutFindings,
        frameTimes,
        network: { externalRequests, babylonRequests: [] },
        protocolRejection: rejection,
        reloadAudit,
        screenshots: captures
      };
      await appendRunEvidence(path.join(repoRoot, "artifacts/acceptance/pivot-presentation-harness-01/scenarios", `${viewport.width}x${viewport.height}-summary.json`), runRecord);
      testInfo.annotations.push({
        type: "fixture-provenance",
        description: `Scenario Runner commit ${fixtureResult.metadata.sourceCommit}; fixture SHA-256 ${fixtureResult.actualSha256}; technical baseline only`
      });
      await context.close();
    });
  }
});
