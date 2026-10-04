import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { AssetRegistry } from "../dist/assets/AssetRegistry.js";
import { actorAssetId } from "../dist/renderer/RendererResourcePlan.js";

const testDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(testDir, "../../..");
const manifestPath = resolve(repoRoot, "governance/assets/RUNTIME_ASSET_MANIFEST.json");
const authorityPath = resolve(repoRoot, "governance/assets/AI_ASSET_RELEASE_MANIFEST.json");
const manifestBytes = new Uint8Array(await readFile(manifestPath));
const authorityBytes = new Uint8Array(await readFile(authorityPath));
const manifest = JSON.parse(new TextDecoder().decode(manifestBytes));
const loadBytes = async relativePath => new Uint8Array(await readFile(resolve(repoRoot, relativePath)));
const loadInvalidBytes = async () => new Uint8Array([0, 1, 2, 3]);
const cloneManifest = () => structuredClone(manifest);

test("ADOPT manifest registers typed assets after verifying pinned authority and every output SHA", async () => {
  const registry = new AssetRegistry();
  const count = await registry.registerManifest(manifestBytes, authorityBytes, loadBytes);
  assert.equal(count, 43);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), true);
  assert.equal(registry.resolveAsset("runtime2d.actor.cenyao.base.v1").kind, "Actor");
  assert.deepEqual(registry.resolveAsset("runtime2d.actor.cenyao.base.v1").animation.directions,
    ["south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east"]);
  assert.equal(registry.resolveAsset("runtime2d.actor.cenyao.base.v1").animation.frames.length, 8);
  assert.equal(registry.resolveAsset("runtime2d.actor.cenyao.base.v1").animation.frames[0].atlasPage, 13);
  assert.equal(registry.resolveAsset("runtime2d.actor.cenyao.base.v1").animation.frames[0].atlasUrl,
    "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp");
  assert.equal(registry.resolveAsset("runtime2d.actor.cenyao.base.v1").animation.frames[0].atlasSha256,
    manifest.atlases[13].webpSha256);
  assert.equal(registry.resolveAsset("runtime2d.prop.gate_a.closed_open.v1").kind, "Door");
  assert.equal(registry.resolveAsset("runtime2d.prop.power_console.off_on.v1").kind, "Prop");
  assert.equal(registry.resolveAsset("runtime2d.vfx.dash.v1").kind, "VFX");
  assert.equal(registry.resolveAsset("runtime2d.ui.capability.local_map.v1").kind, "UI");
  assert.equal(registry.resolveAsset("runtime2d.world.grey_hive.sentinel_arena.v1.attempt").kind, "World");
  assert.equal(registry.resolveAsset("runtime2d.enemy.sentinel.base.v1").kind, "Boss");
  const drownedV2 = registry.resolveAsset("runtime2d.enemy.mistharbor.drowned.v2");
  const wraithV2 = registry.resolveAsset("runtime2d.enemy.mistharbor.signal_wraith.v2");
  const forgedGuardV2 = registry.resolveAsset("runtime2d.enemy.clockworks.forged_guard.v2");
  assert.equal(drownedV2.kind, "Actor");
  assert.equal(drownedV2.sourceSha256, "1b57a57c4fc7dc1d853b82cc1c6a3525e97af5853006ad17a8e25d0de6ca993e");
  assert.equal(wraithV2.kind, "Actor");
  assert.equal(wraithV2.sourceSha256, "fcaad0d40dc3a9dca4c24d147c656d4c751b6a729d265ae3f60d421fe69dd055");
  assert.equal(forgedGuardV2.kind, "Actor");
  assert.equal(forgedGuardV2.sourceSha256, "5c542927516e2e393eccada493e5ca4e20d4bc5d75db8431d57f306910c5d0c0");
  assert.equal(actorAssetId({ entityType: "enemy.mist_harbor.drowned", active: true }), drownedV2.assetId);
  assert.equal(actorAssetId({ entityType: "enemy.mist_harbor.signal_wraith", active: true }), wraithV2.assetId);
  assert.equal(actorAssetId({ entityType: "enemy.clockworks.forged_guard", active: true }),
    "runtime2d.enemy.clockworks.forged_guard.v2");
  assert.equal(registry.resolve("player.cenyao").textureUrl, "assets/derived/asset-pipeline-v1/sprites/webp/runtime2d.actor.cenyao.base.v1.webp");
  assert.equal(registry.resolve("door.gh_gate_a").kind, "door");
});

test("registry preserves the legacy direct register and resolve API", () => {
  const registry = new AssetRegistry();
  registry.register("legacy.actor", { kind: "actor", textureUrl: "/approved/actor.webp", anchorX: 0.5, anchorY: 1, scale: 1 });
  assert.equal(registry.resolve("legacy.actor").textureUrl, "/approved/actor.webp");
  assert.throws(() => registry.resolve("unregistered"), /E_ASSET_NOT_APPROVED/);
});

test("candidate manifests, modified authorities, and unapproved asset rows fail closed", async () => {
  const registry = new AssetRegistry();
  const candidate = cloneManifest();
  candidate.manifestState = "CANDIDATE";
  await assert.rejects(registry.registerManifest(new TextEncoder().encode(JSON.stringify(candidate)), authorityBytes, loadBytes), /E_ASSET_MANIFEST_SCHEMA/);
  const modifiedAuthority = authorityBytes.slice();
  modifiedAuthority[modifiedAuthority.length - 2] ^= 1;
  await assert.rejects(registry.registerManifest(manifestBytes, modifiedAuthority, loadBytes), /E_ASSET_AUTHORITY_SHA/);
  const unapproved = cloneManifest();
  unapproved.assets[0].admission = "concept_only";
  await assert.rejects(registry.registerManifest(new TextEncoder().encode(JSON.stringify(unapproved)), authorityBytes, loadBytes), /E_ASSET_NOT_ADOPTED/);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), false);
});

test("source SHA, output SHA, and safe-path violations are rejected atomically", async () => {
  const wrongSource = cloneManifest();
  wrongSource.assets[0].sourceSha256 = "0".repeat(64);
  const registry = new AssetRegistry();
  await assert.rejects(registry.registerManifest(new TextEncoder().encode(JSON.stringify(wrongSource)), authorityBytes, loadBytes), /E_ASSET_SOURCE_SHA/);
  await assert.rejects(registry.registerManifest(manifestBytes, authorityBytes, loadInvalidBytes), /E_ASSET_OUTPUT_SHA/);
  const unsafePath = cloneManifest();
  unsafePath.assets[0].outputs.webp.path = "assets/derived/asset-pipeline-v1/../../outside.webp";
  await assert.rejects(registry.registerManifest(new TextEncoder().encode(JSON.stringify(unsafePath)), authorityBytes, loadBytes), /E_ASSET_OUTPUT_SCHEMA/);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), false);
});

test("atlas category budgets, frame bounds, and non-overlap are enforced", async () => {
  const oversizedUi = cloneManifest();
  const uiPage = oversizedUi.atlases.find(page => page.category === "ui");
  uiPage.size[0] = 2049;
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(oversizedUi)), authorityBytes, loadBytes), /E_ASSET_ATLAS_BUDGET/);

  const outside = cloneManifest();
  const outsideAsset = outside.assets[0];
  outsideAsset.atlasFrame[0] = outside.atlases[outsideAsset.atlasPage].size[0];
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(outside)), authorityBytes, loadBytes), /E_ASSET_FRAME_BOUNDS/);

  const overlap = cloneManifest();
  const first = overlap.assets.find(item => item.atlasPage === overlap.assets[1].atlasPage);
  const second = overlap.assets.find(item => item.assetId !== first.assetId && item.atlasPage === first.atlasPage);
  second.atlasFrame = [...first.atlasFrame];
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(overlap)), authorityBytes, loadBytes), /E_ASSET_FRAME_OVERLAP/);

  const falseResidency = cloneManifest();
  falseResidency.residency.p95MiB = 1;
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(falseResidency)), authorityBytes, loadBytes), /E_ASSET_RESIDENCY_SCHEMA/);
});

test("animation direction/state and shadow metadata are schema-checked", async () => {
  const badDirection = cloneManifest();
  badDirection.assets.find(item => item.assetId === "runtime2d.actor.cenyao.base.v1").animation.directions = ["sideways"];
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(badDirection)), authorityBytes, loadBytes), /E_ASSET_ANIMATION_ENUM/);

  const incompleteStates = cloneManifest();
  incompleteStates.assets.find(item => item.assetId === "runtime2d.actor.cenyao.base.v1").animation.frames.pop();
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(incompleteStates)), authorityBytes, loadBytes), /E_ASSET_ANIMATION_INCOMPLETE/);

  const badFrameSource = cloneManifest();
  badFrameSource.assets.find(item => item.assetId === "runtime2d.actor.cenyao.base.v1").animation.frames[0].sourceFrame.png.sha256 = "0".repeat(64);
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(badFrameSource)), authorityBytes, loadBytes), /E_ASSET_ANIMATION_FRAME_SOURCE/);

  const badPage = cloneManifest();
  badPage.atlases[13].streamingGroup = "mistharbor";
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(badPage)), authorityBytes, loadBytes), /E_ASSET_CENYAO_ATLAS|E_ASSET_ATLAS_SCHEMA/);

  const badShadow = cloneManifest();
  badShadow.assets[0].shadow = { kind: "ellipse", offset: [0, 0], scale: [1, 1], opacity: 1.5 };
  await assert.rejects(new AssetRegistry().registerManifest(new TextEncoder().encode(JSON.stringify(badShadow)), authorityBytes, loadBytes), /E_ASSET_SHADOW_SCHEMA/);
});

function deferred() {
  let resolve; let reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

const expectedOutputPaths = [
  ...manifest.atlases.flatMap(page => [page.pngPath, page.webpPath]),
  ...manifest.assets.flatMap(record => [record.outputs.png.path, record.outputs.webp.path]),
];

test("output verification has exactly four bounded workers and publishes only after every SHA passes", { timeout: 20_000 }, async () => {
  const firstFour = deferred(); const release = deferred();
  const registry = new AssetRegistry(); const requested = [];
  let active = 0; let peak = 0;
  const loading = registry.registerManifest(manifestBytes, authorityBytes, async path => {
    requested.push(path); active++; peak = Math.max(peak, active);
    if (requested.length === 4) firstFour.resolve();
    try {
      if (requested.length <= 4) await release.promise;
      return await loadBytes(path);
    } finally { active--; }
  });
  await firstFour.promise;
  assert.equal(active, 4); assert.equal(requested.length, 4);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), false);
  assert.equal(registry.has("player.cenyao"), false);
  release.resolve();
  assert.equal(await loading, 43);
  assert.equal(peak, 4); assert.equal(active, 0);
  assert.deepEqual(requested, expectedOutputPaths);
  assert.equal(requested.length, 114);
  assert.equal(new Set(requested).size, 114);
});

test("failed verification drains active work, stops scheduling and preserves manifest-order diagnostics before retry", { timeout: 20_000 }, async () => {
  const firstFour = deferred(); const gates = Array.from({ length: 4 }, deferred);
  const registry = new AssetRegistry(); const requested = [];
  let active = 0;
  const loading = registry.registerManifest(manifestBytes, authorityBytes, async path => {
    const index = requested.length; requested.push(path); active++;
    if (requested.length === 4) firstFour.resolve();
    try { await gates[index].promise; return await loadBytes(path); }
    finally { active--; }
  });
  // Attach the rejection handler before releasing failures in adversarial order.
  const rejected = assert.rejects(loading, error => error.message === `E_ASSET_OUTPUT_MISSING:${expectedOutputPaths[0]}`);
  await firstFour.promise;
  gates[1].reject(new Error("later manifest path fails first"));
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(requested.length, 4);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), false);
  assert.equal(registry.has("player.cenyao"), false);
  gates[0].reject(new Error("earlier manifest path fails later"));
  gates[2].resolve(); gates[3].resolve();
  await rejected;
  assert.equal(active, 0); assert.equal(requested.length, 4);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), false);
  assert.equal(await registry.registerManifest(manifestBytes, authorityBytes, loadBytes), 43);
});

test("a late corrupt output cannot publish even when earlier worker hashes all passed", { timeout: 20_000 }, async () => {
  const registry = new AssetRegistry(); const requested = [];
  const lastPath = expectedOutputPaths.at(-1);
  await assert.rejects(registry.registerManifest(manifestBytes, authorityBytes, async path => {
    requested.push(path);
    const bytes = await loadBytes(path);
    if (path === lastPath) bytes[0] ^= 1;
    return bytes;
  }), error => error.message === `E_ASSET_OUTPUT_SHA:${lastPath}`);
  assert.deepEqual(requested, expectedOutputPaths);
  assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), false);
  assert.equal(registry.has("player.cenyao"), false);
});
