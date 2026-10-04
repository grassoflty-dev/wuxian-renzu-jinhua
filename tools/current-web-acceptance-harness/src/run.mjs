import { spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { resolveArtifactDirectory } from "./artifact-path.mjs";

import { executionProfile } from "./execution-profile.mjs";

const here = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const mode = process.argv[2];
if (!["functional", "visual", "candidate"].includes(mode) || process.argv.length !== 3) {
  throw new Error("Usage: node src/run.mjs functional|visual|candidate (no browser or snapshot overrides)");
}
const profile = executionProfile(process.env.CURRENT_WEB_EXECUTION_PROFILE, mode);
console.log(`Execution profile: ${JSON.stringify(profile)}; functional evidence is not native or performance acceptance`);
if (process.platform !== "win32") throw new Error("E_CURRENT_WEB_WINDOWS_EDGE_REQUIRED: browser acceptance is not run or skipped on another platform");
if (mode === "visual") {
  const baseline = JSON.parse(await readFile(resolve(here, "baseline.json")));
  if (baseline.status !== "reviewed" || !baseline.environment || !/^[0-9a-f]{40}$/.test(baseline.reviewedSourceGitSha ?? "") ||
      !/^[0-9a-f]{64}$/.test(baseline.reviewedBundleSha256 ?? "")) {
    throw new Error("E_CURRENT_WEB_VISUAL_BASELINE_PENDING: review current-app candidates on the fixed Windows Edge environment first");
  }
}
const artifacts = resolveArtifactDirectory(process.env.CURRENT_WEB_ARTIFACTS_DIR ||
  mkdtempSync(join(tmpdir(), "wuxian-current-web-acceptance-")), resolve(here, "../.."));
console.log(`Current Web evidence directory: ${artifacts}`);
const result = spawnSync(process.execPath, [resolve(here, "node_modules/@playwright/test/cli.js"),
  "test", mode === "functional" ? "current-web.spec.mjs" : "visual.spec.mjs"], {
  cwd: here, stdio: "inherit", env: { ...process.env, CURRENT_WEB_MODE: mode, CURRENT_WEB_ARTIFACTS_DIR: artifacts },
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
