import test from "node:test";
import { createHash } from "node:crypto";
import { SceneDefinitionLoader } from "../dist/assets/SceneDefinitionLoader.js";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { planScenePresentation } from "../dist/renderer/ScenePresentation.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const json = async path => JSON.parse(await readFile(resolve(root, path), "utf8"));
const manifest = await json("governance/assets/RUNTIME_ASSET_MANIFEST.json");
const ledger = await json("governance/assets/AI_ASSET_RELEASE_MANIFEST.json");
const approved = new Set(ledger.assets.filter(asset => asset.release_status === "release_approved").map(asset => asset.asset_id));
const assets = new Map(manifest.assets.map(asset => [asset.assetId, asset]));
const registry = {
  resolveAsset(assetId) {
    assert.ok(approved.has(assetId), `unapproved asset: ${assetId}`);
    const record = assets.get(assetId);
    assert.ok(record, `missing runtime asset: ${assetId}`);
    const page = manifest.atlases[record.atlasPage];
    assert.ok(page, `missing atlas page: ${assetId}`);
    return {
      assetId, kind: record.category === "vfx" ? "VFX" : "World",
      category: record.category, streamingGroup: record.streamingGroup,
      atlasUrl: page.webpPath, atlasSha256: page.webpSha256, atlasPage: record.atlasPage,
      atlasFrame: record.atlasFrame, textureUrl: record.outputs.webp.path,
      textureSha256: record.outputs.webp.sha256, sourcePath: record.sourcePath,
      sourceSha256: record.sourceSha256, anchorX: record.anchor[0], anchorY: record.anchor[1], scale: record.scale,
    };
  },
};

for (const [sceneId, worldId] of [
  ["gh_entry_maintenance", "grey_hive"], ["gh_power_room", "grey_hive"], ["gh_gate_a", "grey_hive"],
  ["gh_central_shaft", "grey_hive"], ["gh_lockdown", "grey_hive"], ["gh_bio_isolation", "grey_hive"],
  ["gh_gate_b", "grey_hive"], ["gh_deep_decon", "grey_hive"], ["gh_sentinel_arena", "grey_hive"],
  ["gh_beacon", "grey_hive"], ["gh_exit", "grey_hive"], ["rs_core_room", "return_station"],
  ["mh_fog_pier", "mist_harbor"], ["mh_tidal_warehouse", "mist_harbor"],
  ["mh_signal_yard", "mist_harbor"], ["mh_drowned_quay", "mist_harbor"],
  ["mh_breakwater", "mist_harbor"], ["mh_pump_station", "mist_harbor"],
  ["mh_resonance_tower", "mist_harbor"], ["mh_warden_arena", "mist_harbor"],
  ["mh_extraction", "mist_harbor"], ["cw_entry_foundry", "clockworks"],
  ["cw_pressure_hall", "clockworks"], ["cw_conveyor_bridge", "clockworks"],
  ["cw_boiler_chamber", "clockworks"], ["cw_gear_shaft", "clockworks"],
  ["cw_furnace_heart", "clockworks"], ["cw_forged_guard_arena", "clockworks"],
  ["cw_regulator_core", "clockworks"], ["cw_shutdown_exit", "clockworks"],
]) {
  test(`production ${sceneId} compiles into a renderable approved scene plan`, async () => {
    const bytes = await readFile(resolve(root, `content/scenes/compiled/${sceneId}.json`));
    const digest = value => createHash("sha256").update(value).digest("hex");
    const manifestBytes = Buffer.from(JSON.stringify({schemaVersion:1,scenes:[{worldId,sceneId,
      path:`scene-definitions/compiled/${worldId}/${sceneId}.json`,sha256:digest(bytes)}]}));
    const loader = new SceneDefinitionLoader({baseUrl:"https://fixture.invalid/",expectedManifestSha256:digest(manifestBytes),
      fetchImpl:async url=>new Response(new URL(url).pathname.endsWith("SCENE_DEFINITION_MANIFEST.json")?manifestBytes:bytes)});
    const scene = await loader.loadForSnapshot({worldId,sceneId,worldEpoch:1});
    const plan = planScenePresentation(registry, scene);
    assert.equal(plan.worldId, worldId);
    assert.equal(plan.sceneId, sceneId);
    assert.ok(plan.sprites.length > 0);
    assert.ok(plan.atlasPages.length > 0);
  });
}
