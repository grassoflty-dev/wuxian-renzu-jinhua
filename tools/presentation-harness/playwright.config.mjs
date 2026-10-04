import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@playwright/test";

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, "../..");
const artifacts = path.join(repoRoot, "artifacts/acceptance/pivot-presentation-harness-01");

export default defineConfig({
  testDir: path.join(here, "tests"),
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 30_000,
  expect: { timeout: 5_000 },
  outputDir: path.join(artifacts, "test-results"),
  reporter: [
    ["list"],
    ["json", { outputFile: path.join(artifacts, "playwright-report.json") }]
  ],
  snapshotPathTemplate: path.join(here, "goldens", "{testFilePath}", "{arg}-{projectName}-{platform}{ext}"),
  projects: [{
    name: "msedge",
    use: {
      browserName: "chromium",
      channel: "msedge",
      headless: true,
      deviceScaleFactor: 1,
      viewport: { width: 1280, height: 720 },
      launchOptions: { timeout: 15_000 }
    }
  }],
  webServer: {
    command: "node server.mjs",
    cwd: here,
    url: "http://127.0.0.1:4173/",
    reuseExistingServer: false,
    timeout: 10_000,
    env: { PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD: "1" }
  }
});
