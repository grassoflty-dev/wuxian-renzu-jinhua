import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { collectRuntimeBundle, runtimeAssetBundlePlugin, B0_AUTHORITY_PATH, RUNTIME_MANIFEST_PATH } from "../scripts/runtime-asset-bundle.mjs";

const testsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(testsDir, "../../..");
const authorityBytes = await readFile(resolve(repoRoot, B0_AUTHORITY_PATH));
const runtimeManifestBytes = await readFile(resolve(repoRoot, RUNTIME_MANIFEST_PATH));
const runtimeManifest = JSON.parse(runtimeManifestBytes.toString("utf8"));
const manifestOverride = value => Buffer.from(JSON.stringify(value));

test("canonical bundle is exactly two manifests plus 114 approved-derived outputs", async () => {
  const payload = await collectRuntimeBundle(repoRoot);
  assert.equal(payload.approvedAssets, 43);
  assert.equal(payload.atlasPages, 14);
  assert.equal(payload.manifests.length, 2);
  assert.equal(payload.outputs.length, 114);
  assert.equal(payload.outputs.filter(output => output.role === "sprite/png").length, 43);
  assert.equal(payload.outputs.filter(output => output.role === "sprite/webp").length, 43);
  assert.equal(payload.outputs.filter(output => output.role === "atlas/png").length, 14);
  assert.equal(payload.outputs.filter(output => output.role === "atlas/webp").length, 14);
  assert.ok(payload.outputs.some(output => output.path === "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.png"));
  assert.equal(payload.outputs.some(output => output.path.startsWith("assets/source/approved/")), false);
  assert.equal(payload.outputs.some(output => /concept|reject|release-ui|\.glb$/i.test(output.path)), false);
  for (const file of payload.outputs) assert.equal(file.bytes.length > 0, true);
});

test("authority changes, concept/reject admission, missing/duplicate outputs, and path traversal fail closed", async () => {
  const badAuthority = Buffer.from(authorityBytes);
  badAuthority[badAuthority.length - 2] ^= 1;
  await assert.rejects(collectRuntimeBundle(repoRoot, { authorityBytes: badAuthority }), /E_B0_MANIFEST_SHA/);

  const unapproved = structuredClone(runtimeManifest);
  unapproved.assets[0].assetId = "concept.only.fixture";
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(unapproved) }), /E_RUNTIME_ASSET_NOT_APPROVED/);

  const traversal = structuredClone(runtimeManifest);
  traversal.assets[0].outputs.png.path = "assets/derived/asset-pipeline-v1/../../outside.png";
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(traversal) }), /E_RUNTIME_OUTPUT_SCHEMA/);

  const duplicate = structuredClone(runtimeManifest);
  duplicate.assets[0].outputs.webp.path = duplicate.assets[0].outputs.png.path;
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(duplicate) }), /E_RUNTIME_OUTPUT_DUPLICATE/);
});

test("source identity, output SHA, and atlas category budgets are hard gates", async () => {
  const wrongSource = structuredClone(runtimeManifest);
  wrongSource.assets[0].sourceSha256 = "0".repeat(64);
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(wrongSource) }), /E_RUNTIME_ASSET_SOURCE/);

  const wrongOutput = structuredClone(runtimeManifest);
  wrongOutput.assets[0].outputs.png.sha256 = "0".repeat(64);
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(wrongOutput) }), /E_RUNTIME_OUTPUT_SHA/);

  const overBudget = structuredClone(runtimeManifest);
  const ui = overBudget.atlases.find(page => page.category === "ui");
  ui.size[0] = 2049;
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(overBudget) }), /E_RUNTIME_ATLAS_BUDGET/);
});

test("Cenyao idle metadata is pinned and bundle frames must match its directions, rects, and provenance", async () => {
  assert.equal(runtimeManifest.atlases.length, 14);
  const cenyao = runtimeManifest.assets.find(asset => asset.assetId === "runtime2d.actor.cenyao.base.v1");
  assert.equal(cenyao.animation.frames.length, 8);
  assert.equal(cenyao.animation.frames.every(frame => frame.atlasPage === 13 && frame.state === "idle"), true);
  const wrongPage = structuredClone(runtimeManifest);
  wrongPage.assets.find(asset => asset.assetId === cenyao.assetId).animation.frames[0].atlasPage = 12;
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(wrongPage) }), /E_RUNTIME_CENYAO_FRAME/);
  const wrongSha = structuredClone(runtimeManifest);
  wrongSha.assets.find(asset => asset.assetId === cenyao.assetId).animation.frames[0].sourceFrame.png.sha256 = "0".repeat(64);
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(wrongSha) }), /E_RUNTIME_CENYAO_FRAME/);
  const wrongAtlas = structuredClone(runtimeManifest);
  wrongAtlas.atlases[13].pngSha256 = "0".repeat(64);
  await assert.rejects(collectRuntimeBundle(repoRoot, { runtimeManifestBytes: manifestOverride(wrongAtlas) }), /E_RUNTIME_CENYAO_ATLAS|E_RUNTIME_ATLAS_SCHEMA|E_RUNTIME_OUTPUT_SHA/);
});

test("Vite plugin emits every validated file under its manifest-relative path", async () => {
  const payload = {
    manifests: [{ path: B0_AUTHORITY_PATH, bytes: Buffer.from("authority"), sourcePath: "authority" },
      { path: RUNTIME_MANIFEST_PATH, bytes: Buffer.from("runtime"), sourcePath: "runtime" }],
    outputs: [{ path: "assets/derived/asset-pipeline-v1/atlases/test.webp", bytes: Buffer.from("atlas"), sourcePath: "atlas" }],
  };
  const plugin = runtimeAssetBundlePlugin({ collectBundle: async () => payload });
  const watched = [];
  await plugin.buildStart.call({ addWatchFile: file => watched.push(file) });
  const emitted = [];
  plugin.generateBundle.call({ emitFile: file => emitted.push(file) });
  assert.deepEqual(emitted.map(file => file.fileName), [...payload.manifests, ...payload.outputs].map(file => file.path));
  assert.deepEqual(watched, [...payload.manifests, ...payload.outputs].map(file => file.sourcePath));
});
