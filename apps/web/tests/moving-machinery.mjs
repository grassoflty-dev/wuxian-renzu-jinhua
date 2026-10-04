import test from "node:test";
import assert from "node:assert/strict";
import { projectMovingMachinery } from "../dist/renderer/MovingMachineryModel.js";
import fs from "node:fs";

const hazard = (overrides = {}) => ({ entityId: "press", kind: "moving_machinery", active: true, phaseActive: true,
  polygonM: [[8, 11.5], [10, 11.5], [10, 13], [8, 13]], warningRemainingMs: 900, ...overrides });
const snapshot = (overrides = {}) => ({ protocolVersion: 3, worldId: "clockworks", sceneId: "cw_regulator_core", worldEpoch: 7, serverTick: 60,
  actors: [{ entityId: "cw_prime_regulator", entityType: "runtime2d.enemy.clockworks.forged_guard.v2", active: true }],
  hazards: [hazard()], ...overrides });

test("machinery projects authority geometry and safe warning without simulating motion", () => {
  const frame = projectMovingMachinery(snapshot(), false)[0];
  assert.equal(frame.warning, true);
  assert.deepEqual(frame.polygonM, hazard().polygonM);
  const moving = hazard({ warningRemainingMs: null, polygonM: [[10,11.5],[12,11.5],[12,13],[10,13]] });
  assert.deepEqual(projectMovingMachinery(snapshot({ hazards: [moving] }), true)[0], {
    entityId: "press", polygonM: moving.polygonM, warning: false, reducedMotion: true,
  });
});

test("machinery rejects inactive phases, dead or wrong actors, wrong scene/world/epoch, bad and duplicate data", () => {
  for (const patch of [{ protocolVersion: 2 }, { worldId: "grey_hive" }, { sceneId: "cw_furnace_heart" }, { worldEpoch: -1 },
    { serverTick: NaN }, { actors: [] }, { actors: [{ ...snapshot().actors[0], active: false }] },
    { actors: [{ ...snapshot().actors[0], entityId: "forged" }] }, { actors: [...snapshot().actors, ...snapshot().actors] },
    { hazards: [hazard(), hazard()] }, { hazards: [hazard({ phaseActive: false })] }, { hazards: [hazard({ active: false })] },
    { hazards: [hazard({ warningRemainingMs: 10001 })] }, { hazards: [hazard({ polygonM: [[NaN,0],[1,0],[1,1]] })] }]) {
    assert.deepEqual(projectMovingMachinery(snapshot(patch), false), []);
  }
});

test("machinery cooling remains independent and newer epochs use only their snapshot", () => {
  const cooledHeat = { entityId: "heat", kind: "heat_zone", active: false, phaseActive: true };
  assert.equal(projectMovingMachinery(snapshot({ hazards: [cooledHeat, hazard()] }), false).length, 1);
  assert.deepEqual(projectMovingMachinery(snapshot({ worldEpoch: 8, hazards: [] }), false), []);
  assert.equal(projectMovingMachinery(snapshot({ worldEpoch: 9 }), false).length, 1);
});

test("renderer wires authority machinery, clears it, and marks art as temporary", () => {
  const source = fs.readFileSync(new URL("../src/renderer/WorldRenderer.ts", import.meta.url), "utf8");
  assert.match(source, /renderMovingMachinery\(projectMovingMachinery\(snapshot, reduceFogMotion\), camera\)/);
  assert.match(source, /this\.clearMovingMachinery\(\)/);
  assert.match(source, /temporary_visual=true:moving-machinery/);
});

test("Pixi machinery follows current authority footprint and releases on phase exit", async () => {
  const { WorldRenderer } = await import("../dist/renderer/WorldRenderer.js");
  const { Container } = await import("pixi.js");
  const renderer = Object.create(WorldRenderer.prototype);
  const layer = new Container();
  renderer.layerContainers = new Map([["L7_VFX", layer]]);
  renderer.machineryGraphics = new Map();
  const camera = { width: 1280, height: 720, origin: { xM: 0, yM: 0, zM: 0 }, pixelsPerMeter: 40 };
  renderer.renderMovingMachinery(projectMovingMachinery(snapshot(), true), camera);
  assert.equal(layer.children.length, 1);
  const graphic = renderer.machineryGraphics.get("press");
  const before = graphic.getLocalBounds().minX;
  const translated = hazard({ polygonM: [[14,11.5],[16,11.5],[16,13],[14,13]], warningRemainingMs: null });
  renderer.renderMovingMachinery(projectMovingMachinery(snapshot({ hazards: [translated] }), true), camera);
  assert.equal(renderer.machineryGraphics.get("press"), graphic);
  assert.ok(graphic.getLocalBounds().minX > before, "authoritative motion is retained with reduced effects");
  renderer.renderMovingMachinery([], camera);
  assert.equal(layer.children.length, 0);
  assert.equal(renderer.machineryGraphics.size, 0);
});
