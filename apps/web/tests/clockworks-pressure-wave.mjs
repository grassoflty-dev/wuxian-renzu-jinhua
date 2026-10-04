import test from "node:test";
import assert from "node:assert/strict";
import { ClockworksPressureWaveModel, projectPrimeRegulatorBossMark } from "../dist/renderer/ClockworksPressureWaveModel.js";

function snapshot(overrides = {}) {
  return {
    protocolVersion: 3,
    worldId: "clockworks",
    sceneId: "cw_forged_guard_arena",
    worldEpoch: 7,
    serverTick: 120,
    actors: [{
      entityId: "cw_forged_guard_elite",
      entityType: "runtime2d.enemy.clockworks.forged_guard.v2",
      active: true,
      transform: { positionM: { xM: 12, yM: 0, zM: 8 }, yawRad: 0 },
    }],
    ...overrides,
  };
}

function event(overrides = {}) {
  return {
    protocolVersion: 2,
    eventId: 1,
    worldEpoch: 7,
    serverTick: 120,
    kind: "ForgedGuardPressureWindup",
    positionM: { xM: 12, yM: 0, zM: 8 },
    directionRad: 0,
    radiusM: 3,
    intensity: 1,
    ...overrides,
  };
}

test("accepts only current-epoch pressure wave events from the registered live Arena elite", () => {
  const model = new ClockworksPressureWaveModel();
  const current = snapshot();
  assert.deepEqual(model.accept([event()], current), [1]);
  assert.deepEqual(model.accept([event()], current), []);
  assert.deepEqual(model.project(current, true), [{
    eventId: 1,
    kind: "ForgedGuardPressureWindup",
    originM: { xM: 12, yM: 0, zM: 8 },
    radiusM: 3,
    ageTicks: 0,
    progress: 0,
    reducedMotion: true,
  }]);

  assert.deepEqual(model.accept([event({ eventId: 2, worldEpoch: 6 })], current), []);
  assert.deepEqual(model.accept([event({ eventId: 3 })], snapshot({
    actors: [snapshot().actors[0], { ...snapshot().actors[0], entityId: "duplicate" }],
  })), []);
  assert.deepEqual(model.accept([event({ eventId: 4 })], snapshot({
    sceneId: "cw_pressure_hall",
  })), []);
  assert.deepEqual(model.project(snapshot({ worldEpoch: 8 }), false), []);
});

test("impact presentation expires quickly and never survives a scene or elite change", () => {
  const model = new ClockworksPressureWaveModel();
  const impact = event({ eventId: 8, kind: "ForgedGuardPressureImpact" });
  assert.deepEqual(model.accept([impact], snapshot()), [8]);
  assert.equal(model.project(snapshot({ serverTick: 131 }), false).length, 1);
  assert.deepEqual(model.project(snapshot({ serverTick: 132 }), false), []);
  assert.deepEqual(model.accept([event({ eventId: 9 })], snapshot({
    actors: [{ ...snapshot().actors[0], active: false }],
  })), []);
});

test("Prime Regulator pressure waves require the live unique core actor and matching event family", () => {
  const model = new ClockworksPressureWaveModel();
  const regulator = snapshot({
    sceneId: "cw_regulator_core",
    actors: [{
      entityId: "cw_prime_regulator",
      entityType: "runtime2d.enemy.clockworks.forged_guard.v2",
      active: true,
      transform: { positionM: { xM: 14, yM: 0, zM: 8 }, yawRad: 0 },
    }],
  });
  const warning = event({
    kind: "PrimeRegulatorPressureWindup",
    positionM: { xM: 14, yM: 0, zM: 8 },
    radiusM: 4.5,
  });
  assert.deepEqual(model.accept([warning], regulator), [1]);
  assert.deepEqual(model.accept([event({ eventId: 2 })], regulator), []);
  assert.deepEqual(model.accept([event({ eventId: 3, kind: "PrimeRegulatorPressureImpact", worldEpoch: 6 })], regulator), []);
  assert.deepEqual(model.accept([event({ eventId: 4, kind: "PrimeRegulatorPressureImpact" })], snapshot({
    sceneId: "cw_regulator_core",
    actors: [regulator.actors[0], { ...regulator.actors[0], entityId: "duplicate" }],
  })), []);
  assert.deepEqual(model.project(regulator, true), [{
    eventId: 1,
    kind: "PrimeRegulatorPressureWindup",
    originM: { xM: 14, yM: 0, zM: 8 },
    radiusM: 4.5,
    ageTicks: 0,
    progress: 0,
    reducedMotion: true,
  }]);
  assert.equal(model.project({ ...regulator, serverTick: 180 }, false)[0]?.progress, 60 / 84,
    "the 1.4 second Prime Regulator windup remains visible through its Rust-authoritative warning interval");
});

test("the temporary Boss mark projects only for the current unique active Prime Regulator", () => {
  const regulator = snapshot({
    sceneId: "cw_regulator_core",
    actors: [{
      entityId: "cw_prime_regulator",
      entityType: "runtime2d.enemy.clockworks.forged_guard.v2",
      active: true,
      transform: { positionM: { xM: 14, yM: 0, zM: 8 }, yawRad: 0 },
    }],
  });
  assert.deepEqual(projectPrimeRegulatorBossMark(regulator, false), {
    entityId: "cw_prime_regulator",
    entityType: "runtime2d.enemy.clockworks.forged_guard.v2",
    positionM: { xM: 14, yM: 0, zM: 8 },
    worldEpoch: 7,
    reducedMotion: false,
  });
  assert.equal(projectPrimeRegulatorBossMark({ ...regulator, worldEpoch: 8 }, true)?.worldEpoch, 8,
    "the mark is rebuilt from the current epoch snapshot rather than retaining an old frame");
  assert.equal(projectPrimeRegulatorBossMark({ ...regulator, protocolVersion: 2 }, false), null);
  assert.equal(projectPrimeRegulatorBossMark({ ...regulator, worldId: "grey_hive" }, false), null);
  assert.equal(projectPrimeRegulatorBossMark({ ...regulator, sceneId: "cw_forged_guard_arena" }, false), null);
  assert.equal(projectPrimeRegulatorBossMark({ ...regulator, worldEpoch: -1 }, false), null);
  assert.equal(projectPrimeRegulatorBossMark({
    ...regulator,
    actors: [{ ...regulator.actors[0], active: false }],
  }, false), null);
  assert.equal(projectPrimeRegulatorBossMark({
    ...regulator,
    actors: [{ ...regulator.actors[0], entityType: "runtime2d.enemy.clockworks.other.v1" }],
  }, false), null);
  assert.equal(projectPrimeRegulatorBossMark({
    ...regulator,
    actors: [regulator.actors[0], { ...regulator.actors[0], entityId: "duplicate" }],
  }, false), null);
});
