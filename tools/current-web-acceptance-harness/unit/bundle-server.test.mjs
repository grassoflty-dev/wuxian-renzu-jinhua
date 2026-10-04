import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { writeBundleIdentity } from "../../../apps/web/scripts/bundle-identity.mjs";
import { createBundleServer } from "../server.mjs";
import { sha256, verifyCurrentWebBundle } from "../src/verify-bundle.mjs";

const HEAD = "1".repeat(40);
async function fixture(t, options = {}) {
  const root = await mkdtemp(join(os.tmpdir(), "current-web-contract-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const dist = join(root, "apps/web/dist");
  const runtime = Buffer.from('{"test":"runtime-manifest"}\n');
  const scenes = Buffer.from('{"schemaVersion":1,"scenes":[]}\n');
  const files = {
    "index.html": options.html ?? '<div id="app"></div><script type="module" crossorigin src="./assets/index-hash.js"></script>',
    "assets/index-hash.js": 'document.querySelector("#app").textContent = "contract fixture";',
    "governance/assets/RUNTIME_ASSET_MANIFEST.json": runtime,
    "governance/assets/AI_ASSET_RELEASE_MANIFEST.json": "{}\n",
    "scene-definitions/SCENE_DEFINITION_MANIFEST.json": scenes,
  };
  for (const [path, bytes] of Object.entries(files)) {
    await mkdir(dirname(join(dist, path)), { recursive: true }); await writeFile(join(dist, path), bytes);
  }
  const identity = { schemaVersion: 1, gitSha: HEAD, appVersion: "1.0.0", contentVersion: "test-v1",
    saveV6SchemaVersion: 6, runtimeAssetManifestSha256: sha256(runtime), sceneDefinitionManifestSha256: sha256(scenes),
    builtAtUtc: "2026-10-01T00:00:00Z", hasTrackedDiff: false, hasUntrackedFiles: false, sourceTreeDirty: false,
    ...options.identity };
  await writeBundleIdentity(dist, identity, "assets/index-hash.js");
  return { root, dist };
}

test("only the clean HEAD production bundle with matching manifest hashes is admitted", async t => {
  const { root } = await fixture(t); const result = await verifyCurrentWebBundle(root, HEAD);
  assert.equal(result.identity.servedRoot, "apps/web/dist");
  assert.equal(result.identity.nativeGameplayAcceptance, false);
  assert.equal(result.identity.buildIdentity.gitSha, HEAD);
  assert.equal(result.payload.get("assets/index-hash.js").length, result.identity.entry.sizeBytes);
});

test("stale source identity and dirty bundles fail closed", async t => {
  const clean = await fixture(t);
  await assert.rejects(verifyCurrentWebBundle(clean.root, "2".repeat(40)), /E_CURRENT_WEB_STALE_BUILD/);
  const dirty = await fixture(t, { identity: { hasTrackedDiff: true, sourceTreeDirty: true } });
  await assert.rejects(verifyCurrentWebBundle(dirty.root, HEAD), /E_CURRENT_WEB_DIRTY_BUILD/);
});

test("legacy/Vite entry and forged embedded manifest digests cannot become production evidence", async t => {
  const legacy = await fixture(t, { html: '<div id="app"></div><script type="module" src="/src/main.ts"></script>' });
  await assert.rejects(verifyCurrentWebBundle(legacy.root, HEAD), /E_CURRENT_WEB_ENTRY_HTML/);
  const forged = await fixture(t, { identity: { runtimeAssetManifestSha256: "0".repeat(64) } });
  await assert.rejects(verifyCurrentWebBundle(forged.root, HEAD), /E_CURRENT_WEB_MANIFEST_IDENTITY/);
});

test("tampered, extra, absent and stale tsc output are rejected by existing production sidecar verifier", async t => {
  const modified = await fixture(t);
  await writeFile(join(modified.dist, "assets/index-hash.js"), "modified");
  await assert.rejects(verifyCurrentWebBundle(modified.root, HEAD), /E_BUNDLE_IDENTITY_FILE_SHA/);
  const extra = await fixture(t); await writeFile(join(extra.dist, "main.js"), "unbundled tsc");
  await assert.rejects(verifyCurrentWebBundle(extra.root, HEAD), /E_BUNDLE_IDENTITY_FILE_SET/);
  const missing = await fixture(t); await rm(join(missing.dist, "bundle-identity.json"));
  await assert.rejects(verifyCurrentWebBundle(missing.root, HEAD), /E_BUNDLE_IDENTITY_SIDECAR_MISSING/);
});

test("server serves verified immutable dist bytes only and exposes explicit mock-only identity", async t => {
  const { root, dist } = await fixture(t); const verified = await verifyCurrentWebBundle(root, HEAD);
  const server = createBundleServer(verified);
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  t.after(() => new Promise(resolve => { server.closeAllConnections(); server.close(resolve); }));
  const base = `http://127.0.0.1:${server.address().port}`;
  const index = await fetch(`${base}/`); assert.equal(index.status, 200);
  assert.equal(await index.text(), await readFile(join(dist, "index.html"), "utf8"));
  assert.equal(index.headers.get("cache-control"), "no-store");
  await writeFile(join(dist, "assets/index-hash.js"), "changed after startup");
  const entry = await (await fetch(`${base}/assets/index-hash.js`)).arrayBuffer();
  assert.equal(sha256(Buffer.from(entry)), verified.identity.entry.sha256);
  const identity = await (await fetch(`${base}/__current-web-harness__/identity`)).json();
  assert.equal(identity.nativeGameplayAcceptance, false);
  for (const path of ["/src/main.ts", "/server-rs/release-ui/index.html", "/README.md", "/not-a-route"]) {
    assert.equal((await fetch(base + path)).status, 404, path);
  }
  assert.equal((await fetch(`${base}/%00`)).status, 404);
  assert.equal((await fetch(`${base}/%E0%A4%A`)).status, 400);
  assert.equal((await fetch(`${base}/`, { method: "POST" })).status, 405);
  assert.equal((await fetch(`${base}/`, { method: "HEAD" })).headers.get("content-type"), "text/html; charset=utf-8");
});
