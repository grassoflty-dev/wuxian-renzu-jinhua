const assert = require("node:assert/strict");
const { createFormalView, SCHEMA, PROTOCOL, VERSION } = require("../../release-ui/js/presentation-v3.js");

function view(revision, tick = revision, epoch = 4, worldId = "grey_hive") {
  const payload = {
    protocol: PROTOCOL, version: VERSION, schemaVersion: SCHEMA, worldId, worldEpoch: epoch,
    serverTick: tick, authorityRevision: revision, ackSeq: revision, serverTimeMs: tick * 16,
    player: { entityId: "player", transform: { positionM: { xM: 1, yM: 0, zM: 2 }, yawRad: 0 }, velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 90, maxHp: 100 },
    actors: [], doors: [],
    capabilities: { schemaVersion: 1, items: [], exploredMap: { worldId, playerPositionM: { xM: 1, yM: 0, zM: 2 }, rooms: [], connections: [], objectives: [] }, enemyVitals: [], rearView: { granted: false, grantId: null, grantedAtRevision: null } },
    progression: { schemaVersion: 1, currentWorldId: worldId, eventSeq: 0, worlds: [] }
  };
  return { kind: "full", schemaVersion: SCHEMA, worldId, worldEpoch: epoch, serverTick: tick, authorityRevision: revision, ackSeq: revision, view: payload };
}

module.exports = function runProtocolTest() {
  const calls = { apply: [], hud: [], reset: 0, dispose: 0, full: [] };
  const formal = createFormalView({
    createRenderer: () => ({ applyView: v => calls.apply.push(v), setQuality() {}, reset: () => calls.reset++, dispose: () => calls.dispose++ }),
    createHud: () => ({ apply: v => calls.hud.push(v), reset: () => {}, dispose: () => {} })
  });
  formal.init({ requestFullSnapshot: reason => calls.full.push(reason) });
  formal.reset({ worldId: "grey_hive", worldEpoch: 4 });
  assert.deepEqual(formal.consume(view(10)), { accepted: true, reason: "OK", expectedRevision: 11 });
  assert.equal(calls.apply.length, 1); assert.equal(calls.hud.length, 1);
  assert.equal(formal.consume(view(10, 11)).reason, "E_STALE_REVISION");
  assert.equal(formal.consume(view(12, 12)).reason, "E_REVISION_GAP");
  assert.equal(calls.full.at(-1).reason, "E_REVISION_GAP"); assert.equal(calls.apply.length, 1);
  const delta = { kind: "delta", schemaVersion: SCHEMA };
  assert.equal(formal.consume(delta).reason, "E_DELTA_UNSUPPORTED"); assert.equal(calls.full.at(-1).reason, "E_DELTA_UNSUPPORTED");
  assert.equal(formal.consume(view(11, 10)).reason, "E_STALE_TICK");
  assert.equal(formal.consume(view(11, 11, 4, "mist_harbor")).reason, "E_WORLD_IDENTITY_MISMATCH");
  const invalid = view(11); invalid.view.player.transform.positionM.xM = Number.NaN;
  assert.equal(formal.consume(invalid).reason, "E_NON_FINITE"); assert.equal(calls.apply.length, 1);
  assert.equal(formal.consume(view(11)).accepted, true);
  const staleAck = view(12); staleAck.ackSeq = 1; staleAck.view.ackSeq = 1;
  assert.equal(formal.consume(staleAck).reason, "E_STALE_ACK");
  const staleTime = view(12); staleTime.view.serverTimeMs = 1;
  assert.equal(formal.consume(staleTime).reason, "E_STALE_SERVER_TIME");
  formal.reset({ worldId: "grey_hive", worldEpoch: 5 });
  assert.equal(formal.consume(view(1, 1, 5)).accepted, true);
  formal.dispose(); formal.dispose(); assert.equal(calls.dispose, 1);
  assert.equal(formal.consume(view(2, 2, 5)).reason, "E_NOT_INITIALIZED");
  return { name: "protocol", passed: true, resyncRequests: calls.full.length };
};
