import test from "node:test";
import assert from "node:assert/strict";
import { projectClockworksHeat } from "../dist/renderer/ClockworksHeatModel.js";

const zone = {
  entityId: "cw_regulator_furnace_heat_zone",
  kind: "heat_zone",
  active: true,
  phaseActive: true,
  warningRemainingMs: 600,
  polygonM: [[15.5, 3.5], [20.5, 3.5], [20.5, 6], [15.5, 6]],
  transform: { positionM: { xM: 18, yM: 0, zM: 4.75 }, yawRad: 0 },
};
function snapshot(overrides = {}) {
  return {
    protocolVersion: 3,
    worldId: "clockworks",
    sceneId: "cw_regulator_core",
    worldEpoch: 3,
    serverTick: 120,
    actors: [{ entityId: "cw_prime_regulator", entityType: "runtime2d.enemy.clockworks.forged_guard.v2", active: true }],
    hazards: [zone],
    ...overrides,
  };
}

test("projects the unique authorized regulator furnace zone and respects reduced motion", () => {
  assert.deepEqual(projectClockworksHeat(snapshot(), true), {
    polygonM: zone.polygonM,
    active: true,
    warningRemainingMs: 600,
    reducedMotion: true,
  });
  assert.equal(projectClockworksHeat(snapshot({ hazards: [{ ...zone, active: false, warningRemainingMs: null }] }), false).active, false);
});

test("rejects client-shaped, wrong-scene, duplicate, dead-boss and malformed heat status", () => {
  assert.equal(projectClockworksHeat(snapshot({ sceneId: "cw_forged_guard_arena" }), false), null);
  assert.equal(projectClockworksHeat(snapshot({ actors: [{ ...snapshot().actors[0], active: false }] }), false), null);
  assert.equal(projectClockworksHeat(snapshot({ actors: [snapshot().actors[0], snapshot().actors[0]] }), false), null);
  assert.equal(projectClockworksHeat(snapshot({ hazards: [zone, zone] }), false), null);
  assert.equal(projectClockworksHeat(snapshot({ hazards: [{ ...zone, phaseActive: false }] }), false), null);
  assert.equal(projectClockworksHeat(snapshot({ hazards: [{ ...zone, warningRemainingMs: 601 }] }), false), null);
  assert.equal(projectClockworksHeat(snapshot({ hazards: [{ ...zone, polygonM: [[0, 0], [1, 1]] }] }), false), null);
});

test("cooling and its warning boundary keep the authorized heat zone visible", () => {
  // Rust projects cooling as inactive with no warning; only the final 600ms
  // warm-up carries a countdown. This also describes Continue mid-cooling.
  for (const [active, warningRemainingMs] of [
    [false, null], [true, 600], [true, 1], [true, null],
  ]) {
    const frame = projectClockworksHeat(snapshot({ hazards: [{ ...zone, active, warningRemainingMs }] }), false);
    assert.ok(frame);
    assert.equal(frame.active, active);
    assert.equal(frame.warningRemainingMs, warningRemainingMs);
  }
});
