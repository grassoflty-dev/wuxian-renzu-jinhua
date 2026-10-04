import test from "node:test";
import assert from "node:assert/strict";
import { SoundCueModel, SOUND_CUE_DURATION_MS } from "../dist/renderer/SoundCueModel.js";
import { SnapshotClient } from "../dist/game/SnapshotClient.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { TauriClient } from "../dist/bridge/tauri-client.js";

function snapshot(epoch = 7, sceneId = "mh_tidal_warehouse", granted = true, selected = true) {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "mist_harbor", sceneId, checkpointId: null, worldEpoch: epoch,
    serverTick: 2, authorityRevision: 2, ackSeq: 2,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
      currentEnergy: 100, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [{ capabilityId: "perception.acoustic_mapping_i", granted, selected }] },
    progression: { schemaVersion: 1, currentWorldId: "mist_harbor", eventSeq: 0, worlds: [] },
  };
}

function cue(epoch = 7, eventId = 1, directionRad = 0, distanceM = 13) {
  return { protocolVersion: 1, eventId, worldEpoch: epoch, serverTick: 2,
    worldId: "mist_harbor", sceneId: "mh_tidal_warehouse", kind: "signal_ping", directionRad, distanceM };
}

test("acoustic mapping needs exactly one granted and selected row plus a finite event projection", () => {
  const model = new SoundCueModel();
  const mapped = snapshot();
  for (const projection of [snapshot(7, "mh_tidal_warehouse", false, true), snapshot(7, "mh_tidal_warehouse", true, false),
    { ...mapped, capabilities: { schemaVersion: 1, items: [] } },
    { ...mapped, capabilities: { schemaVersion: 1, items: [mapped.capabilities.items[0], mapped.capabilities.items[0]] } }]) {
    model.accept([cue()], projection, 100);
    assert.equal(model.view(projection, 101), null);
  }
  for (const invalid of [cue(7, 1, null, 5), cue(7, 1, 0, null), cue(7, 1, NaN, 5), cue(7, 1, 0, Infinity), cue(7, 1, 0, -1)]) {
    model.accept([invalid], mapped, 100);
    assert.equal(model.view(mapped, 101), null);
  }
  model.accept([cue(7, 1, 0, 13)], mapped, 100);
  const visible = model.view(mapped, 101);
  assert.equal(visible.distanceLabel, "约 10 米");
  assert.ok(Math.abs(Math.hypot(visible.screenDirectionX, visible.screenDirectionY) - 1) < 1e-12);
  assert.equal(model.view(mapped, 100 + SOUND_CUE_DURATION_MS), null);
});

test("late grant, scene changes, and stale identity never reveal a previous sound", () => {
  const model = new SoundCueModel();
  model.accept([cue()], snapshot(7, "mh_tidal_warehouse", false, false), 0);
  assert.equal(model.view(snapshot(), 1), null);
  model.accept([cue()], snapshot(), 2);
  assert.equal(model.view(snapshot(7, "mh_breakwater"), 3), null);
  model.accept([cue(6)], snapshot(), 4);
  assert.equal(model.view(snapshot(), 5), null);
});

test("snapshot client validates batches, deduplicates IDs, rejects stale epochs, and resets cursor", () => {
  const client = new SnapshotClient();
  client.accept(snapshot());
  assert.deepEqual(client.acceptSoundCues([cue(7, 1), cue(7, 1), cue(6, 2), cue(7, 2)]).map(item => item.eventId), [1, 2]);
  assert.equal(client.soundCueCursor(), 2);
  assert.deepEqual(client.acceptSoundCues([cue(7, 2)]), []);
  assert.throws(() => client.acceptSoundCues([cue(7, 3), { ...cue(7, 4), eventId: "bad" }]), /E_SOUND_CUE_PROTOCOL/);
  assert.equal(client.soundCueCursor(), 2);
  assert.deepEqual(client.acceptSoundCues([{ ...cue(7, 3), sceneId: "mh_breakwater" }]), []);
  assert.equal(client.soundCueCursor(), 3);
  client.accept(snapshot(8));
  assert.equal(client.soundCueCursor(), 0);
  assert.deepEqual(client.acceptSoundCues([cue(7, 1), cue(8, 1)]).map(item => item.eventId), [1]);
});

test("bridge and session loop poll Rust sound cues after authoritative snapshots", async () => {
  const invoked = [];
  const bridge = new TauriClient(async (command, args) => { invoked.push([command, args]); return [cue()]; });
  assert.deepEqual(await bridge.soundCues(7, 0), [cue()]);
  assert.deepEqual(invoked, [["formal_sound_cues", { worldEpoch: 7, afterEventId: 0 }]]);

  const polls = [];
  const delivered = [];
  const server = { events: async () => [], soundCues: async (epoch, cursor) => {
    polls.push([epoch, cursor]);
    return [cue(epoch, 1)];
  }, submitInput: async () => snapshot(), submitAction: async () => snapshot() };
  const scheduler = { now: () => 0, clientTimeMs: () => 0, setInterval: () => 1, clearInterval: () => {},
    requestAnimationFrame: () => 1, cancelAnimationFrame: () => {} };
  const loop = new SessionLoop(server, { render: async () => {} }, {
    scheduler, onSoundCues: events => delivered.push(...events),
  });
  await loop.start(snapshot());
  await loop.acceptAuthoritativeSnapshot({ ...snapshot(), serverTick: 3, authorityRevision: 3 });
  assert.deepEqual(polls, [[7, 0], [7, 1]]);
  assert.deepEqual(delivered.map(item => item.eventId), [1]);
  await loop.stop();
});
