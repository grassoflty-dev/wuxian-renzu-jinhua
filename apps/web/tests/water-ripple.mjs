import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { Container } from "pixi.js";
import { WaterRippleModel } from "../dist/renderer/WaterRippleModel.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { RENDER_LAYERS } from "../dist/renderer/LayerModel.js";

const testDir = dirname(fileURLToPath(import.meta.url));
const point = (x = 15, z = 8) => ({ xM: x, yM: 0, zM: z });
function snapshot() {
  return {
    protocolVersion: 3, worldId: "mist_harbor", sceneId: "mh_drowned_quay", worldEpoch: 2,
    player: { transform: { positionM: point(), yawRad: 0 } },
    hazards: [{ entityId: "mh_water_depth_region", kind: "water_depth_slowdown", active: true,
      transform: { positionM: point(16, 8), yawRad: 0 } }],
  };
}
const clone = value => structuredClone(value);

test("only the exact active authoritative v3 water hazard permits a ripple", () => {
  const changes = [
    value => { value.protocolVersion = 2; },
    value => { value.worldId = "grey_hive"; },
    value => { value.sceneId = "mh_signal_yard"; },
    value => { value.hazards = []; },
    value => { value.hazards.push(clone(value.hazards[0])); },
    value => { value.hazards[0].entityId = "mh_water_depth_slowdown_marker"; },
    value => { value.hazards[0].kind = "water_depth_slowdown_marker"; },
    value => { value.hazards[0].active = false; },
    value => { value.hazards[0].active = 1; },
    value => { delete value.hazards[0].transform; },
    value => { value.hazards[0].transform.yawRad = Number.NaN; },
    value => { value.hazards[0].transform.positionM.xM = Infinity; },
    value => { value.player.transform.positionM.zM = Number.NaN; },
    value => { value.player.transform.yawRad = Infinity; },
  ];
  for (const change of changes) {
    const value = snapshot(); change(value);
    assert.equal(new WaterRippleModel().project(value, 1000, false), null);
  }
  const result = new WaterRippleModel().project(snapshot(), 1000, false);
  assert.deepEqual(result?.position, point());
  assert.equal(result?.progress, 0);
});

test("same epoch advances safely, scene and epoch changes reset, and reduced motion stays static", () => {
  const model = new WaterRippleModel();
  const value = snapshot();
  assert.equal(model.project(value, 100, false)?.progress, 0);
  assert.ok(model.project(value, 550, false).progress > 0);
  assert.equal(model.project(value, 550, true)?.progress, 0);
  assert.equal(model.project(value, 950, true)?.progress, 0);
  assert.ok(model.project(value, 1000, false).progress > 0);
  value.worldEpoch++;
  assert.equal(model.project(value, 1000, false)?.progress, 0);
  value.sceneId = "mh_breakwater";
  assert.equal(model.project(value, 1100, false), null);
  value.sceneId = "mh_drowned_quay";
  assert.equal(model.project(value, 1200, false)?.progress, 0);
  value.hazards[0].active = false;
  assert.equal(model.project(value, 1300, false), null);
  value.hazards[0].active = true;
  assert.equal(model.project(value, 1400, false)?.progress, 0);
});

test("nonfinite, reversed, and extreme finite clocks never emit invalid geometry", () => {
  const model = new WaterRippleModel();
  const value = snapshot();
  for (const time of [Number.NaN, Infinity, -Infinity]) assert.equal(model.project(value, time, false), null);
  assert.equal(model.project(value, -Number.MAX_VALUE, false)?.progress, 0);
  const overflow = model.project(value, Number.MAX_VALUE, false);
  assert.equal(overflow?.progress, 0);
  for (const time of [1e6, 1e9, Number.MAX_VALUE / 2]) {
    const frame = model.project(value, time, false);
    assert.ok(frame && Number.isFinite(frame.progress) && frame.progress >= 0 && frame.progress < 1);
  }
  assert.equal(model.project(value, 0, false)?.progress, 0);
});

test("renderer reuses one Graphics, releases it on deactivation and scene cleanup, and keeps L8 above water", async () => {
  const renderer = new WorldRenderer({});
  const layer = new Container();
  renderer.layerContainers.set("L7_VFX", layer);
  const camera = { width: 800, height: 600, origin: point() };
  const model = new WaterRippleModel();
  const value = snapshot();
  renderer.renderWaterRipple(model.project(value, 100, false), camera);
  const first = renderer.waterRippleGraphic;
  assert.ok(first);
  assert.equal(layer.children.length, 1);
  for (let time = 101; time < 160; time++) renderer.renderWaterRipple(model.project(value, time, false), camera);
  assert.equal(renderer.waterRippleGraphic, first);
  assert.equal(layer.children.length, 1);
  assert.equal(first.label, "water-ripple:mh_drowned_quay");
  assert.equal(first.zIndex, 9000);
  assert.deepEqual([first.position.x, first.position.y], [400, 303]);
  assert.ok(RENDER_LAYERS.indexOf("L7_VFX") < RENDER_LAYERS.indexOf("L8_WORLD_UI"));
  value.hazards[0].active = false;
  renderer.renderWaterRipple(model.project(value, 160, false), camera);
  assert.equal(renderer.waterRippleGraphic, null);
  assert.equal(layer.children.length, 0);
  assert.equal(first.destroyed, true);
  value.hazards[0].active = true;
  renderer.renderWaterRipple(model.project(value, 170, true), camera);
  assert.equal(layer.children.length, 1);
  const second = renderer.waterRippleGraphic;
  renderer.expectSceneIdentity({ worldId: "mist_harbor", sceneId: "mh_breakwater", worldEpoch: 3 });
  await renderer.sceneInvalidationCleanup;
  assert.equal(renderer.waterRippleGraphic, null);
  assert.equal(layer.children.length, 0);
  assert.equal(second.destroyed, true);
  renderer.renderWaterRipple(model.project(value, 180, false), camera);
  assert.equal(layer.children.length, 1);
  const third = renderer.waterRippleGraphic;
  await renderer.destroy();
  assert.equal(third.destroyed, true);
  assert.equal(layer.children.length, 0);
});

test("water integration coexists with fog and action VFX without replacing existing layers", async () => {
  const source = await readFile(resolve(testDir, "../src/renderer/WorldRenderer.ts"), "utf8");
  assert.match(source, /this\.fogLayerModel\.project\(scene, projectedScene\.sprites/);
  assert.match(source, /this\.renderActionVfx\(/);
  assert.match(source, /this\.renderWaterRipple\(this\.waterRippleModel\.project\(snapshot/);
  assert.match(source, /this\.renderOccluders\(/);
  assert.match(source, /this\.renderWorldUi\(snapshot, camera\)/);
  assert.deepEqual(RENDER_LAYERS, ["L0_BACKGROUND", "L1_FLOOR", "L2_BACK_PROPS", "L3_ACTORS",
    "L4_DYNAMIC_PROPS", "L5_FRONT_PROPS", "L6_OCCLUDERS", "L7_VFX", "L8_WORLD_UI"]);
});
