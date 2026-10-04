import test from "node:test";
import assert from "node:assert/strict";
import { projectWorldPoint, resolveSpriteDisplayScale, viewportPixelsPerMeter } from "../dist/renderer/CameraModel.js";

const bounds = { width: 24, depth: 16 };

test("viewport scale stays physically proportional at the three release checkpoints", () => {
  assert.equal(viewportPixelsPerMeter(1280, 720), 48);
  assert.equal(viewportPixelsPerMeter(1600, 937), 60);
  assert.equal(viewportPixelsPerMeter(2560, 1440), 96);
  assert.throws(() => viewportPixelsPerMeter(Number.NaN, 720), /E_CAMERA_VIEWPORT/);
  assert.throws(() => viewportPixelsPerMeter(1280, 0), /E_CAMERA_VIEWPORT/);
});

test("actor, terminal, and floor artwork resolve from authored frames and stable scene bounds", () => {
  const actor = { assetId: "runtime2d.actor.cenyao.base.v1", kind: "Actor", category: "character", anchorY: 0.9839, scale: 1 };
  const terminal = { assetId: "runtime2d.world.returnstation.terminals.save.v1", kind: "Prop", category: "world", anchorY: 1, scale: 1 };
  const floor = { assetId: "runtime2d.world.greyhive.floor_tile.v1", kind: "Prop", category: "world", anchorY: 0.5, scale: 1, layerId: "visual.floor" };
  const expectedPpm = [48, 60, 96];

  for (const pixelsPerMeter of expectedPpm) {
    const actorScale = resolveSpriteDisplayScale(actor, 336, 560, pixelsPerMeter, bounds);
    const terminalScale = resolveSpriteDisplayScale(terminal, 1176, 1065, pixelsPerMeter, bounds);
    const floorScale = resolveSpriteDisplayScale(floor, 1434, 980, pixelsPerMeter, bounds);
    assert.ok(Math.abs(actorScale * 560 / pixelsPerMeter - 1.72) < 1e-12);
    assert.ok(Math.abs(terminalScale * 1065 / pixelsPerMeter - 2.2) < 1e-12);
    assert.ok(floorScale * 1434 <= bounds.width * pixelsPerMeter + 1e-9);
    assert.ok(floorScale * 980 <= bounds.depth * pixelsPerMeter + 1e-9);
  }
});

test("camera projection scales with the viewport while preserving bottom-center foot placement", () => {
  const point = { xM: 2, yM: 0, zM: 3 };
  const baseline = projectWorldPoint(point, { width: 1280, height: 720, origin: { xM: 0, yM: 0, zM: 0 }, pixelsPerMeter: 48 });
  const large = projectWorldPoint(point, { width: 2560, height: 1440, origin: { xM: 0, yM: 0, zM: 0 }, pixelsPerMeter: 96 });
  assert.equal(large.x - 1280, 2 * (baseline.x - 640));
  assert.equal(large.footY - 720, 2 * (baseline.footY - 360));
  assert.equal(baseline.y, baseline.footY);
});
