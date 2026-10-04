import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { collectSceneDefinitionBundle } from "./scene-definition-bundle.mjs";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = resolve(SCRIPT_DIR, "../../..");
export const RUNTIME_ASSET_MANIFEST_PATH = "governance/assets/RUNTIME_ASSET_MANIFEST.json";
const TAURI_CONFIG_PATH = "server-rs/tauri.conf.json";
const SAVE_V5_PATH = "server-rs/src/save_v5.rs";
const SAVE_V6_PATH = "server-rs/src/save_v6.rs";
const SHA40 = /^[0-9a-f]{40}$/;
const SHA64 = /^[0-9a-f]{64}$/;
const VERSION = /^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const CONTENT_VERSION = /^[A-Za-z0-9._+-]{1,96}$/;

function fail(code, detail) {
  throw new Error(detail ? code + ":" + detail : code);
}

export function parseTauriVersion(text) {
  let config;
  try { config = JSON.parse(text); } catch { fail("E_BUILD_IDENTITY_TAURI_JSON"); }
  if (!config || typeof config.version !== "string" || !VERSION.test(config.version)) {
    fail("E_BUILD_IDENTITY_TAURI_VERSION");
  }
  return config.version;
}

export function parseSaveVersions(saveV5Text, saveV6Text) {
  const contentMatch = saveV5Text.match(/pub\s+const\s+SAVE_V5_CONTENT_VERSION\s*:\s*&str\s*=\s*"([^"]+)"\s*;/);
  if (!contentMatch || !CONTENT_VERSION.test(contentMatch[1])) fail("E_BUILD_IDENTITY_CONTENT_VERSION");
  const schemaMatch = saveV6Text.match(/pub\s+const\s+SAVE_V6_SCHEMA_VERSION\s*:\s*u32\s*=\s*([0-9]+)\s*;/);
  if (!schemaMatch) fail("E_BUILD_IDENTITY_SAVE_V6_SCHEMA");
  const schemaVersion = Number(schemaMatch[1]);
  if (!Number.isSafeInteger(schemaVersion) || schemaVersion <= 0) fail("E_BUILD_IDENTITY_SAVE_V6_SCHEMA");
  return { contentVersion: contentMatch[1], saveV6SchemaVersion: schemaVersion };
}

export function parseGitStatus(text) {
  if (typeof text !== "string") fail("E_BUILD_IDENTITY_GIT_STATUS");
  const entries = text.split(/\r?\n/).filter(Boolean);
  const hasUntrackedFiles = entries.some(line => line.startsWith("??"));
  const hasTrackedDiff = entries.some(line => !line.startsWith("??"));
  return { hasTrackedDiff, hasUntrackedFiles, sourceTreeDirty: hasTrackedDiff || hasUntrackedFiles };
}

async function readRequired(repoRoot, relativePath) {
  try { return await readFile(resolve(repoRoot, relativePath)); }
  catch (error) {
    if (error && error.code === "ENOENT") fail("E_BUILD_IDENTITY_REQUIRED_FILE_MISSING", relativePath);
    throw error;
  }
}

function gitText(repoRoot, args) {
  try {
    return execFileSync("git", ["-C", repoRoot, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  } catch {
    fail("E_BUILD_IDENTITY_GIT_COMMAND");
  }
}

export async function collectBuildIdentity(repoRoot = REPO_ROOT, options = {}) {
  const root = resolve(repoRoot);
  const gitSha = gitText(root, ["rev-parse", "HEAD"]);
  if (!SHA40.test(gitSha)) fail("E_BUILD_IDENTITY_GIT_SHA");
  const gitStatus = gitText(root, ["status", "--porcelain=v1", "--untracked-files=all"]);
  const cleanliness = parseGitStatus(gitStatus);

  const tauriBytes = await readRequired(root, TAURI_CONFIG_PATH);
  const saveV5Bytes = await readRequired(root, SAVE_V5_PATH);
  const saveV6Bytes = await readRequired(root, SAVE_V6_PATH);
  const assetManifestBytes = await readRequired(root, RUNTIME_ASSET_MANIFEST_PATH);

  const appVersion = parseTauriVersion(tauriBytes.toString("utf8"));
  const saveVersions = parseSaveVersions(saveV5Bytes.toString("utf8"), saveV6Bytes.toString("utf8"));
  const sceneBundle = await collectSceneDefinitionBundle(root);
  const sceneManifestSha256 = sceneBundle && sceneBundle.manifest && sceneBundle.manifest.sha256;
  if (typeof sceneManifestSha256 !== "string" || !SHA64.test(sceneManifestSha256)) {
    fail("E_BUILD_IDENTITY_SCENE_MANIFEST_SHA");
  }

  const builtAtUtc = (options.now ? options.now() : new Date()).toISOString();
  if (Number.isNaN(Date.parse(builtAtUtc)) || !builtAtUtc.endsWith("Z")) fail("E_BUILD_IDENTITY_BUILD_TIME");
  const runtimeAssetManifestSha256 = createHash("sha256").update(assetManifestBytes).digest("hex");
  if (!SHA64.test(runtimeAssetManifestSha256)) fail("E_BUILD_IDENTITY_ASSET_MANIFEST_SHA");

  return {
    schemaVersion: 1,
    gitSha,
    appVersion,
    contentVersion: saveVersions.contentVersion,
    saveV6SchemaVersion: saveVersions.saveV6SchemaVersion,
    runtimeAssetManifestSha256,
    sceneDefinitionManifestSha256: sceneManifestSha256,
    builtAtUtc,
    hasTrackedDiff: cleanliness.hasTrackedDiff,
    hasUntrackedFiles: cleanliness.hasUntrackedFiles,
    sourceTreeDirty: cleanliness.sourceTreeDirty,
  };
}

export function buildIdentityPlugin(options = {}) {
  const repoRoot = options.repoRoot || REPO_ROOT;
  const collect = options.collectBuildIdentity || collectBuildIdentity;
  return {
    name: "build-identity-v1",
    enforce: "pre",
    async config() {
      const identity = await collect(repoRoot, options);
      return { define: { __BUILD_IDENTITY__: JSON.stringify(identity) } };
    },
  };
}
