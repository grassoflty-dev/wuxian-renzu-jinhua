import test from "node:test";
import assert from "node:assert/strict";
import { CameraRig } from "../dist/renderer/CameraRig.js";
import { projectWorldPoint } from "../dist/renderer/CameraModel.js";

const viewport = { width: 800, height: 600 };
const defaultConfig = { deadZoneHalfWidthPx: 72, deadZoneHalfHeightPx: 42, smoothTimeSeconds: 0.18, aimOffsetMeters: 1.25 };

test("initial camera centers the authoritative aim target without changing fixed projection scale", () => {
  const rig = new CameraRig(defaultConfig);
  const camera = rig.update({ xM: 4, yM: 2, zM: -3 }, 3, 4, viewport.width, viewport.height, 0);
  assert.deepEqual(camera.origin, { xM: 4.75, yM: 2, zM: -2 });
  assert.deepEqual(projectWorldPoint({ xM: 4, yM: 2, zM: -3 }, camera), { x: 412, y: 258, footY: 258 });
  assert.deepEqual(Object.keys(camera).sort(), ["height", "origin", "width"]);
});

test("player movement inside the dead zone does not move the camera", () => {
  const rig = new CameraRig(defaultConfig);
  rig.update({ xM: 0, yM: 0, zM: 0 }, 0, 0, viewport.width, viewport.height, 0);
  const camera = rig.update({ xM: 0.5, yM: 0, zM: 0 }, 0, 0, viewport.width, viewport.height, 0.016);
  assert.deepEqual(camera.origin, { xM: 0, yM: 0, zM: 0 });
});

test("camera follows only enough to return the player to dead-zone bounds", () => {
  const rig = new CameraRig(defaultConfig);
  rig.update({ xM: 0, yM: 0, zM: 0 }, 0, 0, viewport.width, viewport.height, 0);
  const camera = rig.update({ xM: 10, yM: 0, zM: 0 }, 0, 0, viewport.width, viewport.height, 100);
  const playerScreen = projectWorldPoint({ xM: 10, yM: 0, zM: 0 }, camera);
  assert.ok(Math.abs(playerScreen.x - (viewport.width / 2 + 72)) < 0.001);
  assert.ok(Math.abs(playerScreen.footY - (viewport.height / 2 + 42)) < 0.001);
});

test("smooth follow approaches the dead-zone correction deterministically", () => {
  const rig = new CameraRig({ ...defaultConfig, deadZoneHalfWidthPx: 0, deadZoneHalfHeightPx: 0, smoothTimeSeconds: 1 });
  rig.update({ xM: 0, yM: 0, zM: 0 }, 0, 0, viewport.width, viewport.height, 0);
  const camera = rig.update({ xM: 4, yM: 0, zM: 0 }, 0, 0, viewport.width, viewport.height, 1);
  const expectedAlpha = 1 - Math.exp(-1);
  assert.ok(Math.abs(camera.origin.xM - 4 * expectedAlpha) < 1e-12);
  assert.ok(Math.abs(camera.origin.zM) < 1e-12);
});

test("reset snaps to the new scene target and never carries old camera state", () => {
  const rig = new CameraRig(defaultConfig);
  rig.update({ xM: 100, yM: 4, zM: 50 }, 1, 0, viewport.width, viewport.height, 0);
  rig.reset();
  const camera = rig.update({ xM: -8, yM: 1, zM: 12 }, 0, -1, viewport.width, viewport.height, 0);
  assert.deepEqual(camera.origin, { xM: -8, yM: 1, zM: 10.75 });
});

test("invalid camera rig inputs are rejected", () => {
  const rig = new CameraRig(defaultConfig);
  assert.throws(() => rig.update({ xM: 0, yM: 0, zM: 0 }, Number.NaN, 0, 800, 600, 0), /E_CAMERA_RIG_INPUT/);
  assert.throws(() => new CameraRig({ ...defaultConfig, aimOffsetMeters: -1 }), /E_CAMERA_RIG_CONFIG/);
});
