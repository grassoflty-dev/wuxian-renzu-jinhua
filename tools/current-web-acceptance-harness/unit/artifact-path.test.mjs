import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { test } from "node:test";
import { resolveArtifactDirectory } from "../src/artifact-path.mjs";

test("evidence requires an absolute outside-repository directory", () => {
  const repo = resolve("example-repo");
  const outside = resolve("external-evidence");
  assert.equal(resolveArtifactDirectory(outside, repo), outside);
  assert.throws(() => resolveArtifactDirectory(undefined, repo), /ABSOLUTE_REQUIRED/);
  assert.throws(() => resolveArtifactDirectory("relative", repo), /ABSOLUTE_REQUIRED/);
  assert.throws(() => resolveArtifactDirectory(repo, repo), /INSIDE_REPO/);
  assert.throws(() => resolveArtifactDirectory(resolve(repo, "artifacts"), repo), /INSIDE_REPO/);
});

test("runner and config share the same explicit evidence path", async () => {
  const runner = await readFile(new URL("../src/run.mjs", import.meta.url), "utf8");
  const config = await readFile(new URL("../playwright.config.mjs", import.meta.url), "utf8");
  assert.match(runner, /mkdtempSync\(join\(tmpdir\(\)/);
  assert.match(runner, /CURRENT_WEB_ARTIFACTS_DIR: artifacts/);
  assert.match(config, /resolveArtifactDirectory\(process\.env\.CURRENT_WEB_ARTIFACTS_DIR/);
  assert.doesNotMatch(config, /\.\.\/\.\.\/artifacts\/acceptance/);
});
