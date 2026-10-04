import { defineConfig } from "@playwright/test";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { resolveArtifactDirectory } from "./src/artifact-path.mjs";

import { executionProfile } from "./src/execution-profile.mjs";

const profile = executionProfile(process.env.CURRENT_WEB_EXECUTION_PROFILE, process.env.CURRENT_WEB_MODE);
const here = dirname(fileURLToPath(import.meta.url));
export const artifacts = resolveArtifactDirectory(process.env.CURRENT_WEB_ARTIFACTS_DIR, resolve(here, "../.."));
if (process.platform !== "win32") throw new Error("E_CURRENT_WEB_WINDOWS_EDGE_REQUIRED");
if (!["functional", "visual", "candidate"].includes(process.env.CURRENT_WEB_MODE)) throw new Error("E_CURRENT_WEB_USE_NPM_RUNNER");

export default defineConfig({
  testDir: resolve(here, "tests"), fullyParallel: false, workers: 1, retries: 0,
  timeout: profile.testTimeoutMs, expect: { timeout: profile.expectTimeoutMs }, forbidOnly: true,
  metadata: { executionProfile: profile },
  outputDir: resolve(artifacts, process.env.CURRENT_WEB_MODE, "test-results"),
  reporter: [["list"], ["json", { outputFile: resolve(artifacts, process.env.CURRENT_WEB_MODE, "report.json") }]],
  updateSnapshots: "none",
  snapshotPathTemplate: resolve(here, "goldens", "{arg}-{projectName}-{platform}{ext}"),
  projects: [{ name: "msedge", use: {
    actionTimeout: profile.actionTimeoutMs,
    browserName: "chromium", channel: "msedge", headless: true, deviceScaleFactor: 1,
    viewport: { width: 1280, height: 720 }, locale: "zh-CN", timezoneId: "UTC",
    colorScheme: "dark", reducedMotion: "reduce", serviceWorkers: "block",
    baseURL: "http://127.0.0.1:4174",
    // Automatic DOM/ARIA/screen snapshots execute before and after every action.
    // Keep action/console/source trace and actual failure screenshots without
    // repeatedly querying layout on the active 60 Hz game or archiving imagery.
    trace: { mode: "retain-on-failure", snapshots: false,
      screenshots: false, sources: true, attachments: true }, screenshot: "only-on-failure",
  } }],
  webServer: { command: "node server.mjs", cwd: here, url: "http://127.0.0.1:4174/__current-web-harness__/identity",
    reuseExistingServer: false, timeout: 60_000 },
});
