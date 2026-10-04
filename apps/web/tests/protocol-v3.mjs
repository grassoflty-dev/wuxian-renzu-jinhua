import test from "node:test";
import assert from "node:assert/strict";
import { InputController } from "../dist/game/InputController.js";
import { SnapshotClient } from "../dist/game/SnapshotClient.js";
import { assertActionV2, assertInputV2, assertSnapshotV3 } from "../dist/protocol/types.js";

const contractKeys = {
  snapshot: ["ackSeq", "actors", "authorityRevision", "capabilities", "checkpointId", "doors", "hazards", "interactables", "kind", "objectives", "player", "progression", "protocolVersion", "sceneId", "schemaVersion", "serverTick", "worldEpoch", "worldId"],
  player: ["actionState", "aimX", "aimZ", "currentEnergy", "currentHp", "entityId", "facingX", "facingZ", "maxEnergy", "maxHp", "transform", "velocityMps"],
  input: ["aimX", "aimZ", "clientTimeMs", "moveX", "moveZ", "protocol", "protocolVersion", "seq", "worldEpoch"],
  action: ["clientTimeMs", "kind", "protocolVersion", "requestId", "worldEpoch"],
  event: ["directionRad", "eventId", "intensity", "kind", "positionM", "protocolVersion", "radiusM", "serverTick", "worldEpoch"],
};

function v3(epoch = 4, tick = 1) {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "grey_hive", checkpointId: null,
    worldEpoch: epoch, serverTick: tick, authorityRevision: tick, ackSeq: 7,
    player: {
      entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
      currentEnergy: 100, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle",
    },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
  };
}

test("frozen v3 snapshot, v2 input/action, and v2 event fixtures share Rust wire keys", () => {
  const snapshot = assertSnapshotV3(v3());
  const controller = new InputController();
  const input = assertInputV2(controller.sample(4, 120));
  const action = assertActionV2(controller.keyDown("j", 4, 121));
  const event = { protocolVersion: 2, eventId: 1, worldEpoch: 4, serverTick: 1, kind: "Hit",
    positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 1, intensity: 1 };
  assert.deepEqual(Object.keys(snapshot).sort(), contractKeys.snapshot);
  assert.deepEqual(Object.keys(snapshot.player).sort(), contractKeys.player);
  assert.deepEqual(Object.keys(input).sort(), contractKeys.input);
  assert.deepEqual(Object.keys(action).sort(), contractKeys.action);
  assert.deepEqual(Object.keys(event).sort(), contractKeys.event);
});

test("v3 snapshot rejects invalid facing vectors; input rejects non-finite and out-of-range values", () => {
  const invalidFacing = v3();
  invalidFacing.player.aimX = Number.NaN;
  assert.throws(() => assertSnapshotV3(invalidFacing), /E_SNAPSHOT_PROTOCOL/);
  assert.throws(() => assertInputV2({ protocol: "continuous-input", protocolVersion: 2,
    worldEpoch: 1, seq: 1, clientTimeMs: 1, moveX: 2, moveZ: 0, aimX: 0, aimZ: 0 }), /E_INPUT_PROTOCOL/);
  assert.throws(() => assertInputV2({ protocol: "continuous-input", protocolVersion: 2,
    worldEpoch: 1, seq: 1, clientTimeMs: 1, moveX: Infinity, moveZ: 0, aimX: 0, aimZ: 0 }), /E_INPUT_PROTOCOL/);
});

test("snapshot client rejects older epochs and preserves v2 migration events", () => {
  const client = new SnapshotClient();
  client.accept(v3());
  assert.throws(() => client.accept(v3(3, 2)), /E_SNAPSHOT_STALE_EPOCH/);
  const oldEvent = { protocolVersion: 1, eventId: 1, worldEpoch: 4, serverTick: 1,
    kind: "Legacy", positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 1, intensity: 1 };
  const newEvent = { ...oldEvent, protocolVersion: 2, eventId: 2 };
  assert.equal(client.acceptEvents([oldEvent, newEvent]).length, 2);
});
