import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  buildIdentityPlugin,
  collectBuildIdentity,
  parseGitStatus,
  parseSaveVersions,
  parseTauriVersion,
} from "../scripts/build-identity.mjs";
import { formatBuildIdentity, isBuildIdentity, shouldToggleBuildIdentity } from "../dist/ui/BuildIdentityOverlay.js";
import { collectSceneDefinitionBundle } from "../scripts/scene-definition-bundle.mjs";

const testsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(testsDir, "../../..");
const fixedTime = new Date("2026-09-26T12:34:56.000Z");

async function createFixture() {
  const root = await mkdtemp(join(tmpdir(), "wuxian-build-identity-"));
  await mkdir(join(root, "server-rs", "src"), { recursive: true });
  await mkdir(join(root, "governance", "assets"), { recursive: true });
  await mkdir(join(root, "content", "scenes", "compiled"), { recursive: true });
  await writeFile(join(root, "server-rs", "tauri.conf.json"), JSON.stringify({ version: "1.2.3" }));
  await writeFile(join(root, "server-rs", "src", "save_v5.rs"),
    'pub const SAVE_V5_CONTENT_VERSION: &str = "campaign-test-v1";\n');
  await writeFile(join(root, "server-rs", "src", "save_v6.rs"),
    "pub const SAVE_V6_SCHEMA_VERSION: u32 = 6;\n");
  await writeFile(join(root, "governance", "assets", "RUNTIME_ASSET_MANIFEST.json"), Buffer.from([123, 10, 125, 10]));
  await writeFile(join(root, "content", "scenes", "compiled", "gh_identity_test.json"),
    JSON.stringify({ schemaVersion: 1, worldId: "grey_hive", sceneId: "gh_identity_test" }));
  execFileSync("git", ["-C", root, "init", "-q"]);
  execFileSync("git", ["-C", root, "config", "user.email", "identity-test@example.invalid"]);
  execFileSync("git", ["-C", root, "config", "user.name", "Build Identity Test"]);
  execFileSync("git", ["-C", root, "add", "."]);
  execFileSync("git", ["-C", root, "commit", "-q", "-m", "fixture"]);
  return root;
}

test("version parsers accept the authoritative source formats and reject malformed values", () => {
  assert.equal(parseTauriVersion('{"version":"2.4.0-beta.1"}'), "2.4.0-beta.1");
  assert.throws(() => parseTauriVersion('{"version":"latest"}'), /E_BUILD_IDENTITY_TAURI_VERSION/);
  assert.throws(() => parseTauriVersion("{"), /E_BUILD_IDENTITY_TAURI_JSON/);
  assert.deepEqual(parseSaveVersions(
    'pub const SAVE_V5_CONTENT_VERSION: &str = "campaign-content-v1";',
    "pub const SAVE_V6_SCHEMA_VERSION: u32 = 6;",
  ), { contentVersion: "campaign-content-v1", saveV6SchemaVersion: 6 });
  assert.throws(() => parseSaveVersions("const OTHER: &str = \"x\";", "const SAVE_V6_SCHEMA_VERSION = 6;"),
    /E_BUILD_IDENTITY_CONTENT_VERSION/);
  assert.throws(() => parseSaveVersions(
    'pub const SAVE_V5_CONTENT_VERSION: &str = "campaign-content-v1";',
    "pub const SAVE_V6_SCHEMA_VERSION: u32 = 0;",
  ), /E_BUILD_IDENTITY_SAVE_V6_SCHEMA/);
});

test("git cleanliness parser separates tracked changes from untracked paths", () => {
  assert.deepEqual(parseGitStatus(""), { hasTrackedDiff: false, hasUntrackedFiles: false, sourceTreeDirty: false });
  assert.deepEqual(parseGitStatus(" M tracked.ts\n?? new.ts\n"),
    { hasTrackedDiff: true, hasUntrackedFiles: true, sourceTreeDirty: true });
});

test("build identity binds HEAD, raw runtime manifest bytes and the existing scene bundle manifest", async () => {
  const root = await createFixture();
  const identity = await collectBuildIdentity(root, { now: () => fixedTime });
  const expectedHead = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  const assetBytes = await readFile(join(root, "governance", "assets", "RUNTIME_ASSET_MANIFEST.json"));
  const sceneBundle = await collectSceneDefinitionBundle(root);
  assert.equal(identity.gitSha, expectedHead);
  assert.match(identity.gitSha, /^[0-9a-f]{40}$/);
  assert.equal(identity.appVersion, "1.2.3");
  assert.equal(identity.contentVersion, "campaign-test-v1");
  assert.equal(identity.saveV6SchemaVersion, 6);
  assert.equal(identity.runtimeAssetManifestSha256, createHash("sha256").update(assetBytes).digest("hex"));
  assert.equal(identity.sceneDefinitionManifestSha256, sceneBundle.manifest.sha256);
  assert.equal(identity.builtAtUtc, fixedTime.toISOString());
  assert.equal(identity.sourceTreeDirty, false);
  assert.equal(identity.hasTrackedDiff, false);
  assert.equal(identity.hasUntrackedFiles, false);

  const plugin = buildIdentityPlugin({ repoRoot: root, now: () => fixedTime });
  const config = await plugin.config();
  assert.deepEqual(JSON.parse(config.define.__BUILD_IDENTITY__), identity);
});

test("dirty tracked files and untracked files are both exposed for the overlay warning", async () => {
  const root = await createFixture();
  await writeFile(join(root, "server-rs", "tauri.conf.json"), '{"version":"1.2.3"}\n');
  await writeFile(join(root, "untracked-proof.txt"), "untracked");
  const identity = await collectBuildIdentity(root, { now: () => fixedTime });
  assert.equal(identity.hasTrackedDiff, true);
  assert.equal(identity.hasUntrackedFiles, true);
  assert.equal(identity.sourceTreeDirty, true);
});

test("missing required identity source files fail closed", async () => {
  const root = await createFixture();
  await writeFile(join(root, "server-rs", "src", "save_v6.rs"), "");
  execFileSync("git", ["-C", root, "add", "."]);
  execFileSync("git", ["-C", root, "commit", "-q", "-m", "remove save schema"]);
  const { unlink } = await import("node:fs/promises");
  await unlink(join(root, "server-rs", "src", "save_v6.rs"));
  await assert.rejects(collectBuildIdentity(root, { now: () => fixedTime }),
    /E_BUILD_IDENTITY_REQUIRED_FILE_MISSING:server-rs\/src\/save_v6\.rs/);
});

test("build script reads the current worktree and existing source helpers without inventing versions", async () => {
  const identity = await collectBuildIdentity(repoRoot);
  assert.equal(identity.gitSha, execFileSync("git", ["-C", repoRoot, "rev-parse", "HEAD"], { encoding: "utf8" }).trim());
  assert.equal(identity.sceneDefinitionManifestSha256, (await collectSceneDefinitionBundle(repoRoot)).manifest.sha256);
  assert.match(identity.runtimeAssetManifestSha256, /^[0-9a-f]{64}$/);
  assert.ok(Number.isFinite(Date.parse(identity.builtAtUtc)));
});



function identityFixture(overrides = {}) {
  return {
    schemaVersion: 1,
    gitSha: "a".repeat(40),
    appVersion: "1.0.0",
    contentVersion: "campaign-content-v1",
    saveV6SchemaVersion: 6,
    runtimeAssetManifestSha256: "b".repeat(64),
    sceneDefinitionManifestSha256: "c".repeat(64),
    builtAtUtc: "2026-09-26T12:34:56.000Z",
    hasTrackedDiff: false,
    hasUntrackedFiles: false,
    sourceTreeDirty: false,
    ...overrides,
  };
}

test("overlay model validates identity formats and clearly warns when HEAD is not the whole source state", () => {
  const clean = identityFixture();
  assert.equal(isBuildIdentity(clean), true);
  const cleanText = formatBuildIdentity(clean);
  assert.match(cleanText, /Git HEAD: /);
  assert.match(cleanText, /Scene Definition Manifest SHA-256: /);
  assert.match(cleanText, /尚未证明当前 EXE\/安装包嵌入此 bundle/);
  assert.match(cleanText, /源码状态：HEAD 工作树干净/);

  const dirty = identityFixture({ hasTrackedDiff: true, hasUntrackedFiles: true, sourceTreeDirty: true });
  const dirtyText = formatBuildIdentity(dirty);
  assert.match(dirtyText, /SHA 只指向 HEAD，不能代表当前全部源码/);
  assert.equal(formatBuildIdentity(identityFixture({ gitSha: "bad" })), null);
  assert.equal(formatBuildIdentity(identityFixture({ sourceTreeDirty: true })), null);
  assert.equal(formatBuildIdentity(identityFixture({ contentVersion: "<img>" })), null);
});

test("F3 overlay toggle ignores modifiers, repeats, prevented events, and editable targets", () => {
  const base = {
    key: "F3", ctrlKey: false, altKey: false, metaKey: false, shiftKey: false,
    repeat: false, defaultPrevented: false, target: null,
  };
  assert.equal(shouldToggleBuildIdentity(base), true);
  for (const override of [
    { key: "f3" }, { ctrlKey: true }, { altKey: true }, { metaKey: true },
    { shiftKey: true }, { repeat: true }, { defaultPrevented: true },
    { target: { tagName: "INPUT" } }, { target: { tagName: "TEXTAREA" } },
    { target: { isContentEditable: true } },
    { target: { closest: () => ({}) } },
  ]) {
    assert.equal(shouldToggleBuildIdentity({ ...base, ...override }), false, JSON.stringify(override));
  }
});
