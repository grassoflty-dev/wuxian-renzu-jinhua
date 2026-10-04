import test from "node:test";
import assert from "node:assert/strict";
import { BossCameraModel, BOSS_CAMERA_SMOOTH_TIME_SECONDS, SENTINEL_BOSS_ZOOM } from "../dist/renderer/BossCameraModel.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { CameraRig } from "../dist/renderer/CameraRig.js";
import { projectWorldPoint, viewportPixelsPerMeter } from "../dist/renderer/CameraModel.js";

function snapshot({ worldId = "grey_hive", sceneId = "gh_sentinel_arena", worldEpoch = 3,
  protocolVersion = 3, actors = [] } = {}) {
  return { kind: "full", protocolVersion, schemaVersion: "scene-v3/1", worldId, sceneId, worldEpoch, actors };
}

const sentinel = (overrides = {}) => ({ entityId: "gh_sentinel_arena_sentinel_01",
  entityType: "enemy.grey_hive.sentinel", active: true, ...overrides });

function settle(model, value, seconds = 5) {
  return model.update(value, seconds);
}

test("boss zoom requires the exact v3 world, scene, identity, type, activity, and a unique candidate", () => {
  const cases = [
    snapshot({ worldId: "mist_harbor", actors: [sentinel()] }),
    snapshot({ sceneId: "gh_power_room", actors: [sentinel()] }),
    snapshot({ actors: [sentinel({ entityId: "other_sentinel" })] }),
    snapshot({ actors: [sentinel({ entityType: "enemy.other" })] }),
    snapshot({ actors: [sentinel({ active: false })] }),
    snapshot({ actors: [sentinel(), sentinel()] }),
    snapshot({ actors: [sentinel(), sentinel({ entityType: "enemy.other" })] }),
    snapshot({ protocolVersion: 2, actors: [sentinel()] }),
  ];
  for (const value of cases) assert.equal(settle(new BossCameraModel(), value), 1);
  assert.equal(settle(new BossCameraModel(), snapshot({ actors: [sentinel()] })), SENTINEL_BOSS_ZOOM);
});

test("zoom enters and exits smoothly, stays bounded, and is frame-rate independent", () => {
  const model = new BossCameraModel();
  assert.equal(model.update(snapshot({ actors: [sentinel()] }), 0), 1);
  const afterOneTimeConstant = model.update(snapshot({ actors: [sentinel()] }), BOSS_CAMERA_SMOOTH_TIME_SECONDS);
  assert.ok(Math.abs(afterOneTimeConstant - (1 - (1 - SENTINEL_BOSS_ZOOM) * (1 - Math.exp(-1)))) < 1e-12);
  const atTarget = settle(model, snapshot({ actors: [sentinel()] }));
  assert.equal(atTarget, SENTINEL_BOSS_ZOOM);
  const exit = model.update(snapshot({ actors: [] }), 0.1);
  assert.ok(exit > SENTINEL_BOSS_ZOOM && exit < 1);
  assert.equal(settle(model, snapshot({ actors: [] })), 1);

  const sixty = new BossCameraModel();
  const thirty = new BossCameraModel();
  for (let i = 0; i < 60; i++) sixty.update(snapshot({ actors: [sentinel()] }), 0.01);
  for (let i = 0; i < 30; i++) thirty.update(snapshot({ actors: [sentinel()] }), 0.02);
  assert.ok(Math.abs(sixty.update(snapshot({ actors: [sentinel()] }), 0) -
    thirty.update(snapshot({ actors: [sentinel()] }), 0)) < 1e-12);
  assert.ok(sixty.update(snapshot({ actors: [sentinel()] }), 100) >= SENTINEL_BOSS_ZOOM);
});

test("world, scene, and epoch transitions ease back to base instead of carrying boss zoom", () => {
  for (const next of [
    snapshot({ worldId: "mist_harbor", sceneId: "mh_signal_yard" }),
    snapshot({ sceneId: "gh_power_room" }),
    snapshot({ worldEpoch: 4, actors: [sentinel()] }),
  ]) {
    const model = new BossCameraModel();
    settle(model, snapshot({ actors: [sentinel()] }));
    const startingScale = model.update(snapshot({ actors: [sentinel()] }), 0);
    const duringFade = model.update(next, 0.08);
    assert.ok(duringFade > startingScale && duringFade < 1);
    assert.equal(settle(model, next), 1);
    if (next.worldEpoch === 4) {
      const afterReset = model.update(next, BOSS_CAMERA_SMOOTH_TIME_SECONDS);
      assert.ok(afterReset < 1 && afterReset > SENTINEL_BOSS_ZOOM);
    }
  }
});

test("renderer camera path applies the same zoom to rig pixels-per-meter at different viewport scales", () => {
  for (const viewport of [{ width: 1280, height: 720 }, { width: 800, height: 600 }]) {
    const renderer = Object.create(WorldRenderer.prototype);
    renderer.bossCameraModel = new BossCameraModel();
    renderer.cameraRig = new CameraRig();
    const value = snapshot({ actors: [sentinel()] });
    const result = renderer.updateCameraFrame(value, { xM: 0, yM: 0, zM: 0 }, 0, 0,
      viewport.width, viewport.height, 5);
    const expected = viewportPixelsPerMeter(viewport.width, viewport.height) * SENTINEL_BOSS_ZOOM;
    assert.ok(Math.abs(result.pixelsPerMeter - expected) < 1e-12);
    const projected = projectWorldPoint({ xM: 1, yM: 0, zM: 0 }, result.camera);
    assert.ok(Math.abs(projected.x - viewport.width / 2 - expected) < 1e-12);
  }
});
