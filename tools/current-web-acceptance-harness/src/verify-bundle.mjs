import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { verifyBundleIdentity } from "../../../apps/web/scripts/bundle-identity.mjs";

export const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
export const EVIDENCE_KIND = "current-web-production-bundle-mocked-ipc";
export const sha256 = bytes => createHash("sha256").update(bytes).digest("hex");

// Test callers may use an isolated repository root; the CLI/server never accept a
// dist override. A stale release-ui/Vite/TypeScript-only build must fail closed.
export async function verifyCurrentWebBundle(repoRoot = REPO_ROOT, expectedGitSha) {
  const dist = resolve(repoRoot, "apps/web/dist");
  const verified = await verifyBundleIdentity(dist);
  const sidecarBytes = await readFile(resolve(dist, "bundle-identity.json"));
  if (sha256(sidecarBytes) !== verified.sidecarSha256) throw new Error("E_CURRENT_WEB_SIDECAR_CHANGED");
  const manifest = JSON.parse(sidecarBytes);
  const head = expectedGitSha ?? execFileSync("git", ["-C", repoRoot, "rev-parse", "HEAD"], {
    encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
  }).trim();
  if (!/^[0-9a-f]{40}$/.test(head) || manifest.buildIdentity.gitSha !== head) {
    throw new Error("E_CURRENT_WEB_STALE_BUILD");
  }
  if (expectedGitSha === undefined && execFileSync("git", ["-C", repoRoot, "status", "--porcelain=v1", "--untracked-files=all"], {
    encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
  }).trim()) throw new Error("E_CURRENT_WEB_DIRTY_CHECKOUT");
  if (manifest.buildIdentity.sourceTreeDirty) throw new Error("E_CURRENT_WEB_DIRTY_BUILD");
  const payload = new Map();
  // Read and hash again to prevent serving bytes changed after verification.
  for (const entry of manifest.files) {
    const bytes = await readFile(resolve(dist, entry.path));
    if (bytes.length !== entry.sizeBytes || sha256(bytes) !== entry.sha256) {
      throw new Error(`E_CURRENT_WEB_PAYLOAD_CHANGED:${entry.path}`);
    }
    payload.set(entry.path, bytes);
  }
  payload.set("bundle-identity.json", sidecarBytes);
  const html = payload.get("index.html")?.toString("utf8") ?? "";
  const scriptTags = html.match(/<script\b[^>]*>/gi) ?? [];
  const moduleSources = scriptTags.filter(tag => /\btype=["']module["']/.test(tag))
    .map(tag => tag.match(/\bsrc=["']([^"']+)["']/)?.[1]);
  if (!/\bid=["']app["']/.test(html) || moduleSources.length !== 1 ||
      moduleSources[0]?.replace(/^\.\//, "").replace(/^\//, "") !== manifest.entry.path ||
      /\/src\/|release-ui|babylon/i.test(html)) throw new Error("E_CURRENT_WEB_ENTRY_HTML");
  for (const required of ["governance/assets/AI_ASSET_RELEASE_MANIFEST.json", "governance/assets/RUNTIME_ASSET_MANIFEST.json", "scene-definitions/SCENE_DEFINITION_MANIFEST.json"]) {
    if (!payload.has(required)) throw new Error(`E_CURRENT_WEB_REQUIRED_PAYLOAD:${required}`);
  }
  if (sha256(payload.get("governance/assets/RUNTIME_ASSET_MANIFEST.json")) !== manifest.buildIdentity.runtimeAssetManifestSha256 ||
      sha256(payload.get("scene-definitions/SCENE_DEFINITION_MANIFEST.json")) !== manifest.buildIdentity.sceneDefinitionManifestSha256) {
    throw new Error("E_CURRENT_WEB_MANIFEST_IDENTITY");
  }
  return { payload, identity: {
    schemaVersion: 1, evidenceKind: EVIDENCE_KIND, nativeGameplayAcceptance: false,
    servedRoot: "apps/web/dist", buildIdentity: manifest.buildIdentity, ...verified,
  } };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { console.log(JSON.stringify((await verifyCurrentWebBundle()).identity, null, 2)); }
  catch (error) { console.error(error.message); process.exitCode = 1; }
}
