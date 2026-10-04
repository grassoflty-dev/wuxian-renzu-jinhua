import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { planScenePresentation, projectScenePresentation, SceneResourceTracker } from "../dist/renderer/ScenePresentation.js";
import { projectSceneGlows } from "../dist/renderer/SceneGlowModel.js";
import { RENDER_LAYERS } from "../dist/renderer/LayerModel.js";

const testDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(testDir, "../../..");
const fixture = JSON.parse(await readFile(resolve(testDir, "fixtures/scene-presentation-core-v1.json"), "utf8"));
const rendererSource = await readFile(resolve(testDir, "../src/renderer/WorldRenderer.ts"), "utf8");
const knownAssetIds = new Set([
  "runtime2d.grey_hive.floor_tiles.v1", "runtime2d.grey_hive.wall_tiles.v1",
  "runtime2d.prop.medical_station.v1", "runtime2d.vfx.pulse.v1",
]);

function assetFor(assetId) {
  const category = assetId.includes(".vfx.") ? "vfx" : assetId.includes(".prop.") ? "prop" : "world";
  const streamingGroup = category === "vfx" ? "shared" : "grey_hive";
  const atlasPage = category === "vfx" ? 8 : category === "prop" ? 5 : 10;
  return { assetId, kind: category === "vfx" ? "VFX" : category === "prop" ? "Prop" : "World",
    category, streamingGroup, textureUrl: `single/${assetId}.webp`, textureSha256: "a".repeat(64),
    sourcePath: "fixture", sourceSha256: "b".repeat(64), anchorX: 0.5, anchorY: 1, scale: 1,
    atlasUrl: `atlas/${category}-${streamingGroup}-${atlasPage}.webp`, atlasSha256: "c".repeat(64), atlasPage,
    atlasFrame: [0, 0, 16, 16] };
}
const registry = { resolveAsset(assetId) {
  if (!knownAssetIds.has(assetId)) throw new Error(`E_ASSET_NOT_APPROVED:${assetId}`);
  return assetFor(assetId);
} };
const clone = value => structuredClone(value);

test("compiled Mist Harbor fog VFX alone moves from authored L7 to actual L4", async () => {
  const json = async path => JSON.parse(await readFile(resolve(root, path), "utf8"));
  const manifest = await json("governance/assets/RUNTIME_ASSET_MANIFEST.json");
  const ledger = await json("governance/assets/AI_ASSET_RELEASE_MANIFEST.json");
  const approved = new Set(ledger.assets.filter(item => item.release_status === "release_approved").map(item => item.asset_id));
  const assets = new Map(manifest.assets.map(item => [item.assetId, item]));
  const realRegistry = { resolveAsset(assetId) {
    if (!approved.has(assetId)) throw new Error(`E_ASSET_NOT_APPROVED:${assetId}`);
    const record = assets.get(assetId);
    if (!record) throw new Error(`E_ASSET_NOT_APPROVED:${assetId}`);
    const page = manifest.atlases[record.atlasPage];
    assert.ok(page);
    return { assetId, kind: record.category === "vfx" ? "VFX" : "World",
      category: record.category, streamingGroup: record.streamingGroup,
      atlasUrl: page.webpPath, atlasSha256: page.webpSha256, atlasPage: record.atlasPage,
      atlasFrame: record.atlasFrame, textureUrl: record.outputs.webp.path,
      textureSha256: record.outputs.webp.sha256, sourcePath: record.sourcePath,
      sourceSha256: record.sourceSha256, anchorX: record.anchor[0], anchorY: record.anchor[1], scale: record.scale };
  } };
  const scene = await json("content/scenes/compiled/mh_breakwater.json");
  const plan = planScenePresentation(realRegistry, scene);
  const fogId = "runtime2d.world.mistharbor.fog_vfx.v1";
  const fogMarkers = scene.vfxMarkers.filter(item => item.assetId === fogId);
  assert.ok(fogMarkers.length > 0);
  assert.equal(plan.layers.find(item => item.id === "visual.vfx_markers").renderLayer, "L7_VFX");
  for (const authored of fogMarkers) {
    const marker = plan.vfxMarkers.find(item => item.id === authored.id);
    assert.ok(marker);
    assert.equal(marker.layerId, "visual.vfx_markers");
    assert.equal(marker.layer, "L4_DYNAMIC_PROPS");
    assert.deepEqual(marker.position, { xM: authored.position[0], yM: authored.position[1], zM: authored.position[2] });
    assert.deepEqual(marker.asset, realRegistry.resolveAsset(authored.assetId));
    assert.ok(plan.assets.includes(marker.asset));
    assert.ok(plan.atlasPages.some(page => page.atlasUrl === marker.asset.atlasUrl && page.atlasSha256 === marker.asset.atlasSha256));
  }
  const projected = projectScenePresentation(plan, { width: 800, height: 600, origin: { xM: 0, yM: 0, zM: 0 } });
  const depths = projected.sprites.map(item => item.layer);
  assert.ok(depths.indexOf("L2_BACK_PROPS") < depths.indexOf("L4_DYNAMIC_PROPS"));
  assert.ok(depths.lastIndexOf("L4_DYNAMIC_PROPS") < depths.indexOf("L5_FRONT_PROPS"));
  assert.deepEqual(RENDER_LAYERS, ["L0_BACKGROUND", "L1_FLOOR", "L2_BACK_PROPS", "L3_ACTORS",
    "L4_DYNAMIC_PROPS", "L5_FRONT_PROPS", "L6_OCCLUDERS", "L7_VFX", "L8_WORLD_UI"]);

  const otherWorld = clone(scene); otherWorld.worldId = "clockworks";
  assert.ok(planScenePresentation(realRegistry, otherWorld).vfxMarkers.every(item => item.layer === "L7_VFX"));
  const otherVfx = clone(scene); otherVfx.vfxMarkers[0].assetId = "runtime2d.vfx.pulse.v1";
  const withOtherVfx = planScenePresentation({ resolveAsset(assetId) {
    return assetId === "runtime2d.vfx.pulse.v1" ? assetFor(assetId) : realRegistry.resolveAsset(assetId);
  } }, otherVfx);
  assert.equal(withOtherVfx.vfxMarkers[0].layer, "L7_VFX");
  assert.ok(withOtherVfx.vfxMarkers.slice(1).every(item => item.layer === "L4_DYNAMIC_PROPS"));
  const unapproved = clone(scene); unapproved.vfxMarkers[0].assetId = "concept.unapproved";
  assert.throws(() => planScenePresentation(realRegistry, unapproved), /E_SCENE_PRESENTATION_ASSET_NOT_APPROVED/);
  const wrongKind = clone(scene);
  assert.throws(() => planScenePresentation({ resolveAsset(assetId) {
    const asset = realRegistry.resolveAsset(assetId);
    return assetId === fogId ? { ...asset, kind: "World" } : asset;
  } }, wrongKind), /E_SCENE_PRESENTATION_VFX_ASSET/);
});

test("compiler SceneDefinition fixture resolves approved assets and maps static geometry to L0-L8", () => {
  const plan = planScenePresentation(registry, fixture);
  assert.equal(plan.worldId, "grey_hive");
  assert.equal(plan.sceneId, "gh_presentation_fixture");
  assert.deepEqual(plan.sprites.map(sprite => [sprite.id, sprite.layer]), [
    ["background.tile", "L0_BACKGROUND"], ["floor.main", "L1_FLOOR"], ["floor.detail", "L1_FLOOR"],
    ["props.back", "L2_BACK_PROPS"], ["console.static", "L4_DYNAMIC_PROPS"], ["wall.foreground", "L5_FRONT_PROPS"],
  ]);
  assert.deepEqual(plan.vfxMarkers.map(marker => [marker.id, marker.layer]), [["pulse.marker", "L7_VFX"]]);
  assert.equal(plan.occluders[0].fadeTo, 0.3);
  assert.equal(plan.assets.length, 4);
  assert.equal(plan.atlasPages.length, 3); // shared floor/wall atlas page is loaded only once.
  assert.ok(plan.assets.every(item => knownAssetIds.has(item.assetId)));
});

test("an approved scene plate stays on L0 beneath modular props", () => {
  const withPlate = clone(fixture);
  withPlate.presentation.backgroundAsset = "runtime2d.grey_hive.floor_tiles.v1";
  const plan = planScenePresentation(registry, withPlate);
  assert.deepEqual(plan.sprites.slice(0, 2).map(item => [item.id, item.layer]), [
    ["__scene_background", "L0_BACKGROUND"], ["background.tile", "L0_BACKGROUND"],
  ]);
  assert.ok(plan.assets.every(asset => knownAssetIds.has(asset.assetId)));
});

test("scene sprites and occluder polygons use the fixed isometric projection and depth order", () => {
  const plan = planScenePresentation(registry, fixture);
  const projected = projectScenePresentation(plan, { width: 800, height: 600, origin: { xM: 0, yM: 0, zM: 0 } });
  const floor = projected.sprites.find(item => item.id === "floor.main");
  assert.deepEqual([floor.screenX, floor.screenY, floor.footY], [352, 420, 420]);
  assert.deepEqual(projected.sprites.map(item => item.id), [
    "background.tile", "floor.main", "floor.detail", "props.back", "console.static", "wall.foreground", "pulse.marker",
  ]);
  assert.deepEqual(projected.occluders[0].points[0], [400, 348]);
  assert.equal(projected.occluders[0].id, "pillar.occluder");
  const withDepth = clone(fixture);
  withDepth.occluders.push({ id: "rear.occluder", polygon: [[1, 6], [2, 6], [2, 7], [1, 7]], fadeTo: 0.5 });
  assert.deepEqual(projectScenePresentation(planScenePresentation(registry, withDepth), { width: 800, height: 600, origin: { xM: 0, yM: 0, zM: 0 } }).occluders.map(item => item.id), ["pillar.occluder", "rear.occluder"]);
});

test("only exact approved scene placements receive small world-matched additive glow cues", () => {
  const placement = { id: "cw_heart_furnace_v1", layer: "L2_BACK_PROPS",
    asset: { assetId: "runtime2d.world.clockworks.furnace.v1" }, screenX: 250, screenY: 300 };
  const glow = projectSceneGlows("clockworks", "cw_furnace_heart", [placement]);
  assert.deepEqual(glow, [{ id: placement.id, screenX: 250, screenY: 265,
    radiusX: 34, radiusY: 18, color: 0xffa94a, alpha: 0.12 }]);
  assert.deepEqual(projectSceneGlows("grey_hive", "cw_furnace_heart", [placement]), []);
  assert.deepEqual(projectSceneGlows("clockworks", "cw_furnace_heart", [
    { ...placement, asset: { assetId: "concept.unapproved" } },
    { ...placement, id: "unapproved.furnace" },
    { ...placement, layer: "L4_DYNAMIC_PROPS" },
  ]), []);
  const fixturePlan = planScenePresentation(registry, fixture);
  const fixtureProjection = projectScenePresentation(fixturePlan,
    { width: 800, height: 600, origin: { xM: 0, yM: 0, zM: 0 } });
  assert.deepEqual(projectSceneGlows(fixturePlan.worldId, fixturePlan.sceneId, fixtureProjection.sprites), []);
});

test("merged L1 floor keeps authored detail above its base even when the detail is farther away", () => {
  const withFarDetail = clone(fixture);
  withFarDetail.presentation.sprites.find(item => item.id === "floor.detail").position = [0, 0, 0];
  const projected = projectScenePresentation(planScenePresentation(registry, withFarDetail),
    { width: 800, height: 600, origin: { xM: 0, yM: 0, zM: 0 } });
  const floor = projected.sprites.filter(item => item.layer === "L1_FLOOR");
  assert.deepEqual(floor.map(item => item.id), ["floor.main", "floor.detail"]);
  assert.ok(floor[1].footY < floor[0].footY, "zGroup, not depth, keeps floor detail on top");
});

test("malformed identities, unknown/unapproved layers or assets, bounds, polygons, and duplicate IDs fail closed", () => {
  const badIdentity = clone(fixture); badIdentity.sceneId = "bad scene";
  assert.throws(() => planScenePresentation(registry, badIdentity), /E_SCENE_PRESENTATION_SCENE/);
  const badCamera = clone(fixture); badCamera.presentation.cameraProfile = "perspective_experimental";
  assert.throws(() => planScenePresentation(registry, badCamera), /E_SCENE_PRESENTATION_CAMERA_PROFILE/);

  const unknownLayer = clone(fixture); unknownLayer.presentation.layers[0].id = "visual.roof";
  assert.throws(() => planScenePresentation(registry, unknownLayer), /E_SCENE_PRESENTATION_LAYER/);
  const missingLayer = clone(fixture); missingLayer.presentation.layers.splice(0, 1);
  assert.throws(() => planScenePresentation(registry, missingLayer), /E_SCENE_PRESENTATION_LAYER_MISSING/);

  const unknownAsset = clone(fixture); unknownAsset.presentation.sprites[0].assetId = "concept.unapproved";
  assert.throws(() => planScenePresentation(registry, unknownAsset), /E_SCENE_PRESENTATION_ASSET_NOT_APPROVED/);
  const outsideBounds = clone(fixture); outsideBounds.presentation.sprites[0].position[0] = 11;
  assert.throws(() => planScenePresentation(registry, outsideBounds), /E_SCENE_PRESENTATION_OUT_OF_BOUNDS/);
  const badPolygon = clone(fixture); badPolygon.occluders[0].polygon = [[1, 1], [2, 2], [3, 3]];
  assert.throws(() => planScenePresentation(registry, badPolygon), /E_SCENE_PRESENTATION_POLYGON/);
  const unknownField = clone(fixture); unknownField.presentation.sprites[0].textureUrl = "unverified.png";
  assert.throws(() => planScenePresentation(registry, unknownField), /E_SCENE_PRESENTATION_SPRITE/);

  const duplicatePlacementId = clone(fixture); duplicatePlacementId.presentation.sprites[0].id = "pillar.occluder";
  assert.throws(() => planScenePresentation(registry, duplicatePlacementId), /E_SCENE_PRESENTATION_DUPLICATE_ID/);
  const duplicateLayerId = clone(fixture); duplicateLayerId.presentation.sprites[0].id = "visual.floor";
  assert.throws(() => planScenePresentation(registry, duplicateLayerId), /E_SCENE_PRESENTATION_DUPLICATE_ID/);
  const duplicateRustId = clone(fixture); duplicateRustId.collision.push({ id: "floor.main", polygon: [] });
  assert.throws(() => planScenePresentation(registry, duplicateRustId), /E_SCENE_PRESENTATION_DUPLICATE_ID/);
});

test("VFX marker assets must be typed as approved VFX resources", () => {
  const vfxFixture = clone(fixture);
  vfxFixture.vfxMarkers[0].assetId = "runtime2d.grey_hive.floor_tiles.v1";
  assert.throws(() => planScenePresentation(registry, vfxFixture), /E_SCENE_PRESENTATION_VFX_ASSET/);
});

test("scene resource tracker releases dropped objects and clears all remaining epoch resources once", () => {
  const resources = new SceneResourceTracker();
  const released = [];
  resources.trackIfAbsent("sprite:scene:floor.main", () => released.push("floor"));
  resources.trackIfAbsent("occluder:pillar.occluder", () => released.push("occluder"));
  resources.trackIfAbsent("scene-glow:cw_heart_furnace_v1", () => released.push("glow"));
  resources.trackIfAbsent("occluder:pillar.occluder", () => released.push("duplicate"));
  resources.dispose("occluder:pillar.occluder");
  resources.clear();
  resources.clear();
  assert.equal(resources.size, 0);
  assert.deepEqual(released, ["occluder", "floor", "glow"]);
});

test("WorldRenderer exposes the injection hook and ties scene sprite, occluder, and atlas cleanup to scene changes", () => {
  assert.match(rendererSource, /setSceneDefinition\(definition: VerifiedSceneDefinition\): void/);
  assert.match(rendererSource, /planScenePresentation\(this\.registry, definition\)/);
  assert.match(rendererSource, /this\.sceneEpoch\.enter\(scene\)\) await this\.clearSceneResources\(\)/);
  assert.match(rendererSource, /this\.sceneResources\.trackIfAbsent\(`sprite:\$\{key\}`/);
  assert.match(rendererSource, /this\.sceneResources\.trackIfAbsent\(`occluder:\$\{occluder\.id\}`/);
  assert.match(rendererSource, /this\.sceneResources\.trackIfAbsent\(`scene-glow:\$\{glow\.id\}`/);
  assert.match(rendererSource, /graphic\.blendMode = "add"/);
  assert.match(rendererSource, /projectSceneGlows\(view\.worldId, view\.sceneId, projectedScene\.sprites\)/);
  assert.match(rendererSource, /graphic\.fill\(\{ color, alpha: this\.occluderFader\.alpha\(/);
  assert.match(rendererSource, /residents\?\.delete\(this\.atlasResidentOwner\)/);
  assert.match(rendererSource, /await Promise\.all\(retirement\)/);
  assert.match(rendererSource, /for \(const layerId of RENDER_LAYERS\)/);
});
