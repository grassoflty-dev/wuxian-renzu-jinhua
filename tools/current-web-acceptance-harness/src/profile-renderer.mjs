import { chromium } from "@playwright/test";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve, relative } from "node:path";
import { fileURLToPath } from "node:url";
import os from "node:os";
import { createBundleServer } from "../server.mjs";
import { verifyCurrentWebBundle, REPO_ROOT } from "./verify-bundle.mjs";
import { mockBootstrapScript } from "./mock-ipc.mjs";
import { passiveObserverScript } from "./passive-observer.mjs";
import { gpuObserverScript } from "./gpu-observer.mjs";

export const PROFILE_LIMIT_BYTES = 3 * 1024 * 1024;
export const SUMMARY_LIMIT_BYTES = 512 * 1024;
export function encodeBoundedArtifact(value, limit = SUMMARY_LIMIT_BYTES) {
  const bytes = Buffer.from(JSON.stringify(value, null, 2));
  if (bytes.length > limit) throw new Error("E_PROFILE_ARTIFACT_SIZE");
  return bytes;
}
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
export async function bounded(promise, ms, label) {
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error(`E_PROFILE_TIMEOUT:${label}`)), ms);
  })]); } finally { clearTimeout(timer); }
}
export function summarizeProfile(profile) {
  const nodes = new Map(profile.nodes.map(node => [node.id, node])); const time = new Map();
  for (let i = 0; i < (profile.samples?.length ?? 0); i++) {
    const id = profile.samples[i]; time.set(id, (time.get(id) ?? 0) + (profile.timeDeltas?.[i] ?? 0));
  }
  return { durationMs: (profile.endTime - profile.startTime) / 1000, sampleCount: profile.samples?.length ?? 0,
    topSelfTime: [...time].sort((a, b) => b[1] - a[1]).slice(0, 40).map(([id, microseconds]) => ({
      ...nodes.get(id)?.callFrame, selfMs: microseconds / 1000 })) };
}
export function scrubProfile(profile) {
  return { ...profile, nodes: profile.nodes.map(node => ({ ...node, callFrame: { ...node.callFrame,
    url: node.callFrame.url.startsWith("http://127.0.0.1:4174/")
      ? node.callFrame.url.split(/[?#]/)[0] : node.callFrame.url ? "internal-or-injected-script" : "" } })) };
}

async function main() {
  if (process.platform !== "win32") throw new Error("E_PROFILE_WINDOWS_EDGE_REQUIRED");
  const height = Number(process.env.CURRENT_WEB_PROFILE_HEIGHT);
  if (![720, 1440].includes(height)) throw new Error("E_PROFILE_VIEWPORT_REQUIRED");
  const out = resolve(process.env.CURRENT_WEB_PROFILE_DIR ?? "");
  const relation = relative(REPO_ROOT, out);
  if (!process.env.CURRENT_WEB_PROFILE_DIR || !relation || (!relation.startsWith("..") && !/^[A-Za-z]:/.test(relation))) {
    throw new Error("E_PROFILE_ARTIFACTS_OUTSIDE_REPO_REQUIRED");
  }
  await mkdir(out, { recursive: true });
  const bundle = await verifyCurrentWebBundle();
  const report = { evidenceKind: "diagnostic-only-renderer-profile", acceptancePass: false,
    nativeGameplayAcceptance: false, profilingOverhead: "V8 1ms CPU sampling and public WebGL call timing wrappers are enabled; these timings are not a performance pass",
    productionTargetFrameMs: 1000 / 60, identity: bundle.identity,
    environment: { platform: process.platform, osRelease: os.release(), arch: os.arch(), cpuCount: os.cpus().length,
      cpuModel: os.cpus()[0]?.model, totalMemoryBytes: os.totalmem(), viewport: { width: height * 16 / 9, height },
      deviceScaleFactor: 1, channel: "msedge", playwrightVersion: "1.63.0", reducedMotion: "reduce" },
    phases: [], errors: [] };
  const persist = async () => {
    try { await writeFile(resolve(out, "diagnostic-summary.json"), encodeBoundedArtifact(report)); }
    catch (error) {
      if (error.message !== "E_PROFILE_ARTIFACT_SIZE") throw error;
      process.exitCode = 1;
      await writeFile(resolve(out, "diagnostic-summary.json"), encodeBoundedArtifact({
        evidenceKind: report.evidenceKind, acceptancePass: false, nativeGameplayAcceptance: false,
        error: "summary exceeded fixed 512 KiB cap", environment: report.environment,
        identity: report.identity, phases: report.phases.map(phase => ({ name: phase.name, status: phase.status,
          profileFile: phase.profileFile, profileBytes: phase.profileBytes })) }));
    }
  };
  await persist();
  const server = createBundleServer(bundle);
  await new Promise((done, reject) => { server.once("error", reject); server.listen(4174, "127.0.0.1", done); });
  let browser;
  try {
    browser = await chromium.launch({ channel: "msedge", headless: true, timeout: 15000 });
    report.environment.browserVersion = browser.version();
    const context = await browser.newContext({ viewport: report.environment.viewport, deviceScaleFactor: 1,
      locale: "zh-CN", timezoneId: "UTC", colorScheme: "dark", reducedMotion: "reduce", serviceWorkers: "block" });
    const page = await context.newPage();
    const consoleMessages = []; const pageErrors = [];
    page.on("console", message => { if (consoleMessages.length < 64) consoleMessages.push({ type: message.type(), text: message.text().slice(0, 400) }); });
    page.on("pageerror", error => { if (pageErrors.length < 32) pageErrors.push(error.message.slice(0, 400)); });
    await page.route("**/*", async route => {
      const url = new URL(route.request().url());
      if (["http:", "https:"].includes(url.protocol) && url.origin !== "http://127.0.0.1:4174") await route.abort("blockedbyclient");
      else await route.continue();
    });
    await page.addInitScript({ content: mockBootstrapScript() });
    await page.addInitScript({ content: passiveObserverScript() });
    await page.addInitScript({ content: gpuObserverScript() });
    await page.goto("http://127.0.0.1:4174", { timeout: 15000 });
    const cdp = await context.newCDPSession(page);
    await cdp.send("Profiler.enable"); await cdp.send("Profiler.setSamplingInterval", { interval: 1000 });
    for (const name of ["entry-and-active", "pause-request"]) {
      const phase = { name, requestedSampleMs: 8000, status: "collecting" }; report.phases.push(phase); await persist();
      let profiling = false;
      try {
        await bounded(cdp.send("Profiler.start"), 5000, `${name}:start`); profiling = true;
        const actionStart = Date.now();
        const action = page.locator(name === "entry-and-active" ? "#new-journey" : "#hud-pause").click({ timeout: 15000 });
        // Include real action latency in the profile, without invoking IPC or relaxing a functional check.
        await bounded(action, 16000, `${name}:action`).then(() => { phase.actionElapsedMs = Date.now() - actionStart; },
          error => { phase.actionError = String(error).slice(0, 1200); });
        await delay(8000);
        const { profile } = await bounded(cdp.send("Profiler.stop"), 12000, `${name}:stop`); profiling = false;
        const safe = scrubProfile(profile); const bytes = Buffer.from(JSON.stringify(safe));
        phase.cpu = summarizeProfile(safe); phase.profileBytes = bytes.length;
        if (bytes.length <= PROFILE_LIMIT_BYTES) { phase.profileFile = `${name}.cpuprofile`; await writeFile(resolve(out, phase.profileFile), bytes); }
        else phase.profileOmitted = "profile exceeds fixed 3 MiB cap; top-self-time summary retained";
        phase.observed = await bounded(page.evaluate(() => ({
          mock: window.__CURRENT_WEB_MOCK__?.captureEvidence(), timing: window.__CURRENT_WEB_PASSIVE__?.state,
          graphics: window.__CURRENT_WEB_GPU_PROFILE__?.state,
          view: document.querySelector(".shell")?.dataset.view,
          pauseTitle: document.querySelector("#pause-title")?.textContent,
          canvas: (() => { const c = document.querySelector("#world-canvas"); return c ? { width: c.width, height: c.height } : null; })(),
        })), 8000, `${name}:observations`).catch(error => ({ observationError: String(error).slice(0, 400) }));
        phase.status = "collected-diagnostic-only";
      } catch (error) { phase.status = "incomplete"; phase.error = String(error).slice(0, 1200); }
      finally {
        if (profiling) await bounded(cdp.send("Profiler.stop"), 3000, `${name}:stop-cleanup`).catch(() => {});
        report.consoleMessages = consoleMessages; report.pageErrors = pageErrors; await persist();
      }
    }
  } catch (error) { report.errors.push(String(error).slice(0, 1200)); }
  finally {
    if (browser) await bounded(browser.close(), 15000, "browser-close").catch(error => report.errors.push(String(error).slice(0, 400)));
    server.closeAllConnections(); server.close();
    await persist();
  }
  if (report.errors.length || report.phases.some(phase => phase.status === "incomplete")) process.exitCode = 1;
  console.log(`Diagnostic-only renderer evidence: ${out}; no functional or native acceptance result`);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().then(() => process.exit(process.exitCode ?? 0)).catch(error => {
    console.error(error.message); process.exit(1);
  });
}
