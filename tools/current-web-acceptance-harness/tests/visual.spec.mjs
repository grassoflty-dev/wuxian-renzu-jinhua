import { readFile } from "node:fs/promises";
import { test, expect, boot, enterNew, paused } from "./helpers.mjs";

const baseline = JSON.parse(await readFile(new URL("../baseline.json", import.meta.url)));

async function environment(page, evidence) {
  const graphics = await page.evaluate(() => {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl2") ?? canvas.getContext("webgl");
    if (!gl) return { renderer: null, vendor: null };
    const info = gl.getExtension("WEBGL_debug_renderer_info");
    const result = { renderer: info ? gl.getParameter(info.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
      vendor: info ? gl.getParameter(info.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR) };
    gl.getExtension("WEBGL_lose_context")?.loseContext();
    return result;
  });
  return { platform: evidence.platform, osRelease: evidence.osRelease, arch: evidence.arch,
    browserVersion: evidence.browserVersion, playwrightVersion: "1.63.0", channel: "msedge",
    locale: "zh-CN", timezoneId: "UTC", deviceScaleFactor: 1, graphics };
}

for (const viewport of baseline.viewports) {
  test(`separate current-app visual evidence ${viewport.width}x${viewport.height}`, async ({ page, evidence }, testInfo) => {
    await page.setViewportSize(viewport); await boot(page, evidence);
    const observed = await environment(page, evidence);
    const candidate = process.env.CURRENT_WEB_MODE === "candidate";
    if (!candidate) {
      expect(baseline.status, "no implicit golden creation or old-harness baseline reuse").toBe("reviewed");
      expect(observed, "fixed Windows/Edge/render environment must match reviewed baseline exactly").toEqual(baseline.environment);
    }
    await testInfo.attach("visual-environment", { body: JSON.stringify({ ...evidence.identity, environment: observed,
      viewport, status: candidate ? "unreviewed-candidate-not-visual-pass" : "comparison" }, null, 2), contentType: "application/json" });
    const capture = async name => {
      const filename = `${name}-${viewport.width}x${viewport.height}.png`;
      const options = { animations: "disabled", caret: "hide", fullPage: false,
        // Tick text depends on browser command cadence, not the UI layout under review.
        mask: [page.locator("#world-detail")], maskColor: "#091720" };
      if (candidate) {
        await page.screenshot({ ...options, path: testInfo.outputPath(filename) });
        await testInfo.attach(filename, { path: testInfo.outputPath(filename), contentType: "image/png" });
      } else await expect(page).toHaveScreenshot(filename, { ...options, maxDiffPixels: 0 });
    };
    await capture("hub");
    await page.locator("#continue-journey").click();
    await page.locator("#continue-slot").selectOption("valid-slot");
    await capture("continue-picker");
    await page.locator("#continue-journey").click();
    await page.locator("#core-ui-open").click();
    for (const panel of ["character", "inventory", "capability", "settings"]) {
      await page.locator(`[data-core-panel="${panel}"]`).click();
      await capture(`core-${panel}`);
    }
    await page.locator("#core-panel-close").click();
    await enterNew(page); await paused(page); await capture("return-station-paused");
  });
}
