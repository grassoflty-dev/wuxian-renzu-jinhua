import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { FogLayerModel, fogDepth } from "../dist/renderer/FogLayerModel.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const BANK = "runtime2d.world.mistharbor.fog_bank.v1";
const VFX = "runtime2d.world.mistharbor.fog_vfx.v1";
const scene = (sceneId = "mh_breakwater", worldEpoch = 1, worldId = "mist_harbor") => ({ worldId, sceneId, worldEpoch });
const sprite = (id, assetId, layerId, layer) => ({ id, asset: { assetId }, layerId, layer, screenX: 50, screenY: 30 });
const placements = [
  sprite("back_fog", BANK, "visual.props_back", "L2_BACK_PROPS"),
  sprite("mid_fog", VFX, "visual.vfx_markers", "L4_DYNAMIC_PROPS"),
  sprite("front_fog", BANK, "visual.foreground", "L5_FRONT_PROPS"),
];

test("fog depth follows exact approved asset and authored layer identity", () => {
  assert.deepEqual(placements.map(item => fogDepth("mist_harbor", item)), ["background", "mid", "foreground"]);
  assert.equal(fogDepth("mist_harbor", sprite("wrong", BANK, "visual.floor", "L1_FLOOR")), null);
  assert.equal(fogDepth("mist_harbor", sprite("wrong", "runtime2d.prop.other.v1", "visual.foreground", "L5_FRONT_PROPS")), null);
  assert.equal(fogDepth("mist_harbor", sprite("wrong", VFX, "visual.vfx_markers", "L7_VFX")), null);
  assert.equal(fogDepth("clockworks", placements[0]), null);
});

test("three authored fog sources move at different speeds, deterministically and within bounds", () => {
  const model = new FogLayerModel();
  model.project(scene(), placements, 1000, false);
  const movement = model.project(scene(), placements, 2250, false);
  assert.deepEqual(movement.coverage, { background: true, mid: true, foreground: true });
  const speeds = placements.map(item => movement.offsets.get(item.id).x);
  assert.equal(new Set(speeds.map(value => value.toFixed(5))).size, 3);
  const foregroundCycle = model.project(scene(), placements, 8500, false);
  assert.ok(Math.abs(foregroundCycle.offsets.get("front_fog").x) < 1e-9);
  assert.ok(Math.abs(foregroundCycle.offsets.get("back_fog").x) > 1e-6);
  assert.ok(Math.abs(foregroundCycle.offsets.get("mid_fog").x) > 1e-6);
  const replay = new FogLayerModel();
  replay.project(scene(), placements, 1000, false);
  assert.deepEqual(replay.project(scene(), placements, 2250, false), movement);
  for (const time of [0, 1e6, 1e9]) {
    const frame = model.project(scene(), placements, time, false);
    for (const item of frame.offsets.values()) {
      assert.ok(Number.isFinite(item.x) && Number.isFinite(item.y));
      assert.ok(Math.abs(item.x) <= 14 && Math.abs(item.y) <= 6);
    }
  }
});

test("reduced motion is fully static and scene or epoch reset restarts at authored positions", () => {
  const model = new FogLayerModel();
  model.project(scene(), placements, 100, false);
  const staticFrame = model.project(scene(), placements, 9000, true);
  for (const item of staticFrame.offsets.values()) assert.deepEqual([item.x, item.y], [0, 0]);
  const nextEpoch = model.project(scene("mh_breakwater", 2), placements, 9000, false);
  for (const item of nextEpoch.offsets.values()) assert.deepEqual([item.x, item.y], [0, 0]);
  model.project(scene(), placements, 12000, false);
  model.reset();
  const reset = model.project(scene(), placements, 12000, false);
  for (const item of reset.offsets.values()) assert.deepEqual([item.x, item.y], [0, 0]);
});

test("nonfinite clocks fail closed without passing NaN into Pixi positions", () => {
  const model = new FogLayerModel();
  for (const time of [Number.NaN, Infinity, -Infinity]) {
    const frame = model.project(scene(), placements, time, false);
    for (const item of frame.offsets.values()) assert.deepEqual([item.x, item.y], [0, 0]);
  }
  const recovered = model.project(scene(), placements, 1000, false);
  for (const item of recovered.offsets.values()) assert.deepEqual([item.x, item.y], [0, 0]);
});

test("extreme finite clocks remain bounded after phase wrapping", () => {
  const model = new FogLayerModel();
  model.project(scene(), placements, 0, false);
  for (const time of [Number.MAX_VALUE, Number.MAX_VALUE / 2, -Number.MAX_VALUE]) {
    const frame = model.project(scene(), placements, time, false);
    for (const item of frame.offsets.values()) {
      assert.ok(Number.isFinite(item.x) && Number.isFinite(item.y));
      assert.ok(Math.abs(item.x) <= 14 && Math.abs(item.y) <= 6);
    }
  }
  model.reset();
  model.project(scene(), placements, -Number.MAX_VALUE, false);
  const overflowedDifference = model.project(scene(), placements, Number.MAX_VALUE, false);
  for (const item of overflowedDifference.offsets.values()) assert.deepEqual([item.x, item.y], [0, 0]);
});

test("non Mist Harbor and scenes missing a layer receive no invented fog placement", () => {
  const model = new FogLayerModel();
  const otherWorld = model.project(scene("cw_entry_foundry", 1, "clockworks"), placements, 100, false);
  assert.equal(otherWorld.offsets.size, 0);
  assert.deepEqual(otherWorld.coverage, { background: false, mid: false, foreground: false });
  const missing = model.project(scene("mh_fog_pier"), placements.slice(0, 2), 200, false);
  assert.equal(missing.offsets.size, 2);
  assert.deepEqual(missing.coverage, { background: true, mid: true, foreground: false });
});

test("compiled Mist Harbor scenes retain their actual fog coverage", async () => {
  const expectedForeground = new Set(["mh_breakwater", "mh_tidal_warehouse"]);
  const scenes = ["mh_breakwater", "mh_drowned_quay", "mh_extraction", "mh_fog_pier", "mh_pump_station",
    "mh_resonance_tower", "mh_signal_yard", "mh_tidal_warehouse", "mh_warden_arena"];
  for (const sceneId of scenes) {
    const raw = JSON.parse(await readFile(resolve(root, `content/scenes/compiled/${sceneId}.json`), "utf8"));
    const authored = [
      ...raw.presentation.sprites.map(item => sprite(item.id, item.assetId, item.layer,
        item.layer === "visual.foreground" ? "L5_FRONT_PROPS" : "L2_BACK_PROPS")),
      ...raw.vfxMarkers.map(item => sprite(item.id, item.assetId, "visual.vfx_markers", "L4_DYNAMIC_PROPS")),
    ];
    const frame = new FogLayerModel().project(scene(sceneId), authored, 1, false);
    assert.deepEqual(frame.coverage, { background: true, mid: true,
      foreground: expectedForeground.has(sceneId) }, sceneId);
  }
});
