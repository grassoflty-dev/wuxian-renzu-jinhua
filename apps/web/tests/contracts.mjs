import test from "node:test";
import assert from "node:assert/strict";
import { AssetRegistry } from "../dist/assets/AssetRegistry.js";
import { SnapshotClient } from "../dist/game/SnapshotClient.js";
import { screenDirectionToWorld } from "../dist/game/InputController.js";

function snapshot(epoch, tick, x) {
  const positionM = { xM: x, yM: 0, zM: 0 };
  return {
    kind: "full", protocolVersion: 2, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", worldEpoch: epoch, serverTick: tick, authorityRevision: tick,
    ackSeq: 0,
    view: {
      protocol: "continuous-ipc", version: 1, schemaVersion: "freeze-v02-interfaces/1.2",
      worldId: "grey_hive", worldEpoch: epoch, serverTick: tick, authorityRevision: tick,
      ackSeq: 0, serverTimeMs: tick * 16,
      player: { entityId: "player", transform: { positionM, yawRad: 0 }, velocityMps: positionM,
        currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100 },
      actors: [], doors: [],
    },
  };
}

test("snapshot interpolation never carries state across epochs", () => {
  const client = new SnapshotClient();
  client.accept(snapshot(1, 10, 0));
  client.accept(snapshot(1, 11, 10));
  assert.equal(client.interpolatedPlayer(0.5).xM, 5);
  assert.throws(() => client.accept(snapshot(1, 9, 0)), /E_SNAPSHOT_REGRESSION/);
  client.accept(snapshot(2, 0, 50));
  assert.equal(client.interpolatedPlayer(0.5).xM, 50);
  assert.equal(client.eventCursor(), 0);
});

test("presentation events are consumed once per world epoch", () => {
  const client = new SnapshotClient();
  client.accept(snapshot(1, 1, 0));
  const event = { protocolVersion: 1, eventId: 1, worldEpoch: 1, serverTick: 1,
    kind: "AttackStarted", positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 1, intensity: 1 };
  assert.equal(client.acceptEvents([event]).length, 1);
  assert.equal(client.acceptEvents([event]).length, 0);
  client.accept(snapshot(2, 0, 0));
  assert.equal(client.acceptEvents([event]).length, 0);
});

test("registry fails closed until an asset is explicitly registered", () => {
  const registry = new AssetRegistry();
  assert.throws(() => registry.resolve("player.cenyao"), /E_ASSET_NOT_APPROVED/);
  registry.register("player.cenyao", { kind: "actor", textureUrl: "/approved/actor.png", anchorX: 0.5, anchorY: 1, scale: 1 });
  assert.equal(registry.resolve("player.cenyao").textureUrl, "/approved/actor.png");
  assert.throws(() => registry.register("player.cenyao", { kind: "actor", textureUrl: "/other.png", anchorX: 0, anchorY: 0, scale: 1 }), /E_ASSET_REGISTRY_INVALID/);
});

test("fixed-camera movement maps WASD to the expected screen direction", () => {
  const up = screenDirectionToWorld(0, -1);
  assert.ok(up.x < 0 && up.z < 0);
  const right = screenDirectionToWorld(1, 0);
  assert.ok(right.x > 0 && right.z < 0);
  assert.ok(Math.abs(Math.hypot(up.x, up.z) - 1) < 1e-9);
});
