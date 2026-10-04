import test from "node:test";
import assert from "node:assert/strict";
import { Container } from "pixi.js";
import { SentinelTelegraphModel } from "../dist/renderer/SentinelTelegraphModel.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { projectWorldPoint } from "../dist/renderer/CameraModel.js";

const sentinel = (active = true, overrides = {}) => ({ entityId: "gh_sentinel_arena_sentinel_01",
  entityType: "enemy.grey_hive.sentinel", actorKind: "enemy", active,
  transform: { positionM: { xM: 7.5, yM: 0, zM: 4.25 }, yawRad: 0 }, ...overrides });

function snapshot({ protocolVersion = 3, worldId = "grey_hive", sceneId = "gh_sentinel_arena",
  worldEpoch = 8, serverTick = 100, actors = [sentinel()] } = {}) {
  return { kind: "full", protocolVersion, schemaVersion: "scene-v3/1", worldId, sceneId,
    checkpointId: null, worldEpoch, serverTick, authorityRevision: serverTick, ackSeq: serverTick,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100, currentEnergy: 100,
      maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors, doors: [], interactables: [], hazards: [], objectives: [], capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: worldId, eventSeq: 0, worlds: [] } };
}

function event(eventId, kind = "SentinelAttackWindup", overrides = {}) {
  return { protocolVersion: 2, eventId, worldEpoch: 8, serverTick: 100, kind,
    positionM: { xM: 7.5, yM: 0, zM: 4.25 }, directionRad: 0, radiusM: 1, intensity: 1, ...overrides };
}

function frames(model, value = snapshot(), reducedMotion = false) { return model.project(value, reducedMotion); }

test("only exact v2 windup and death events in the registered Sentinel encounter can render", () => {
  const invalidCases = [
    [event(1), snapshot({ worldId: "mist_harbor", sceneId: "mh_signal_yard" })],
    [event(1), snapshot({ sceneId: "gh_power_room" })],
    [event(1, "SentinelAttackWindup", { worldEpoch: 7 }), snapshot()],
    [event(1, "SentinelAttackWindup", { protocolVersion: 1 }), snapshot()],
    [event(1, "PlayerDamaged"), snapshot()],
    [event(1, "SentinelAttackWindup", { radiusM: Number.NaN }), snapshot()],
    [event(1, "SentinelAttackWindup", { positionM: { xM: 0, yM: 0, zM: Infinity } }), snapshot()],
    [event(1), snapshot({ actors: [sentinel(false)] })],
    [event(1), snapshot({ actors: [sentinel(), sentinel()] })],
    [event(1), snapshot({ actors: [sentinel(true, { entityId: "wrong-id" })] })],
    [event(1), snapshot({ actors: [sentinel(true, { entityType: "enemy.other" })] })],
  ];
  for (const [cue, value] of invalidCases) {
    const model = new SentinelTelegraphModel();
    model.accept([cue], value);
    assert.deepEqual(frames(model, value), []);
  }

  const death = new SentinelTelegraphModel();
  death.accept([event(1, "SentinelDeath")], snapshot({ actors: [sentinel(false)] }));
  assert.deepEqual(frames(death, snapshot({ actors: [sentinel(false)] })).map(frame => frame.kind), ["SentinelDeath"]);
  const liveDeath = new SentinelTelegraphModel();
  liveDeath.accept([event(1, "SentinelDeath")], snapshot());
  assert.deepEqual(frames(liveDeath), []);
});

test("windup age uses 60 Hz snapshot ticks and drops stale or future events", () => {
  for (const [kind, liveAge, expiredAge] of [
    ["SentinelAttackWindup", 14, 15],
    ["SentinelHeavyWindup", 23, 24],
  ]) {
    const live = new SentinelTelegraphModel();
    live.accept([event(1, kind)], snapshot({ serverTick: 100 + liveAge }));
    assert.equal(frames(live, snapshot({ serverTick: 100 + liveAge })).length, 1);

    const expired = new SentinelTelegraphModel();
    expired.accept([event(1, kind)], snapshot({ serverTick: 100 + expiredAge }));
    assert.deepEqual(frames(expired, snapshot({ serverTick: 100 + expiredAge })), []);

    const future = new SentinelTelegraphModel();
    future.accept([event(1, kind)], snapshot({ serverTick: 99 }));
    assert.deepEqual(frames(future, snapshot({ serverTick: 99 })), []);
  }
});

test("only visible warning events are authorized for matching warning audio", () => {
  const valid = new SentinelTelegraphModel();
  assert.deepEqual(valid.accept([event(1)], snapshot()), [1]);

  const expired = new SentinelTelegraphModel();
  assert.deepEqual(expired.accept([event(1)], snapshot({ serverTick: 115 })), []);

  const wrongScene = new SentinelTelegraphModel();
  assert.deepEqual(wrongScene.accept([event(1)], snapshot({ sceneId: "gh_power_room" })), []);
});

test("event IDs deduplicate and reject out-of-order rows without letting unknown kinds render", () => {
  const model = new SentinelTelegraphModel();
  model.accept([event(2), event(1, "SentinelHeavyWindup")], snapshot());
  assert.deepEqual(frames(model).map(frame => frame.eventId), [2]);
  model.accept([event(2), event(3, "UnknownKind"), event(3), event(4, "SentinelHeavyWindup", { radiusM: 0 })], snapshot());
  assert.deepEqual(frames(model).map(frame => frame.eventId), [2]);
  model.accept([event(4, "SentinelHeavyWindup")], snapshot());
  assert.deepEqual(frames(model).map(frame => frame.eventId), [2, 4]);
});

test("world, scene, epoch, v2, and explicit reset clear queued warnings", () => {
  const model = new SentinelTelegraphModel();
  model.accept([event(1)], snapshot());
  assert.equal(frames(model).length, 1);
  for (const value of [
    snapshot({ sceneId: "gh_power_room" }),
    snapshot({ worldId: "mist_harbor", sceneId: "mh_signal_yard" }),
    snapshot({ worldEpoch: 9 }),
    snapshot({ protocolVersion: 2 }),
  ]) assert.deepEqual(frames(model, value), []);
  model.accept([event(1)], snapshot());
  assert.equal(frames(model).length, 1);
  model.reset();
  assert.deepEqual(frames(model), []);
});

test("death replaces earlier windups, and an inactive actor cannot keep a red warning", () => {
  const model = new SentinelTelegraphModel();
  model.accept([event(1, "SentinelHeavyWindup")], snapshot());
  assert.deepEqual(frames(model).map(frame => frame.kind), ["SentinelHeavyWindup"]);
  const dead = snapshot({ serverTick: 102, actors: [sentinel(false)] });
  model.accept([event(2, "SentinelDeath", { serverTick: 102 })], dead);
  assert.deepEqual(frames(model, dead).map(frame => frame.kind), ["SentinelDeath"]);
  assert.deepEqual(frames(model, snapshot({ serverTick: 103 })), [], "death and prior warnings vanish if the actor is active again");
});

test("rendered warnings follow CameraFrame scale and projection, while reduced motion stays steady", () => {
  const value = snapshot();
  const model = new SentinelTelegraphModel();
  model.accept([event(1)], value);
  const worldRenderer = Object.create(WorldRenderer.prototype);
  const layer = new Container();
  worldRenderer.layerContainers = new Map([["L7_VFX", layer]]);
  worldRenderer.sentinelTelegraphGraphics = new Map();
  worldRenderer.sentinelGlowGraphics = new Map();
  worldRenderer.machineryGraphics = new Map();
  worldRenderer.sentinelTelegraphModel = model;
  worldRenderer.transientPresentationRevision = 0;

  const camera = { width: 1280, height: 720, origin: { xM: 0, yM: 0, zM: 0 }, pixelsPerMeter: 40 };
  const reduced = frames(model, value, true);
  worldRenderer.renderSentinelTelegraphs(reduced, camera);
  const warning = worldRenderer.sentinelTelegraphGraphics.get(1);
  const glow = worldRenderer.sentinelGlowGraphics.get(1);
  const point = projectWorldPoint(event(1).positionM, camera);
  assert.ok(warning && glow);
  assert.equal(warning.scale.x, 40 / 48);
  assert.equal(glow.scale.x, 40 / 48);
  assert.equal(warning.position.x, point.x);
  assert.equal(warning.position.y, point.footY + 2 * 40 / 48);
  assert.equal(warning.alpha, 0.67);
  assert.equal(layer.children.length, 2);

  const later = snapshot({ serverTick: 104 });
  const steady = frames(model, later, true);
  worldRenderer.renderSentinelTelegraphs(steady, camera);
  assert.equal(worldRenderer.sentinelTelegraphGraphics.get(1).alpha, warning.alpha);
  const zoomedCamera = { ...camera, pixelsPerMeter: 34.4 };
  worldRenderer.renderSentinelTelegraphs(steady, zoomedCamera);
  assert.equal(worldRenderer.sentinelTelegraphGraphics.get(1).scale.x, 34.4 / 48);

  const dead = snapshot({ serverTick: 105, actors: [sentinel(false)] });
  model.accept([event(2, "SentinelDeath", { serverTick: 105 })], dead);
  worldRenderer.renderSentinelTelegraphs(frames(model, dead, true), zoomedCamera);
  assert.equal(worldRenderer.sentinelTelegraphGraphics.size, 1);
  assert.equal(worldRenderer.sentinelTelegraphGraphics.get(2).alpha, 0.62);
  assert.equal(worldRenderer.sentinelGlowGraphics.size, 0);

  worldRenderer.clearTransientPresentation();
  assert.equal(layer.children.length, 0);
  assert.equal(worldRenderer.sentinelTelegraphGraphics.size, 0);
  assert.deepEqual(frames(model, dead), []);
});

function nativeSnapshot(value = snapshot(), phase = "light_windup") {
  if (value.worldId !== "grey_hive" || value.sceneId !== "gh_sentinel_arena") return value;
  return {...value, actors:value.actors.map(a=>({...a,actorKind:"sentinel"})), sentinelEncounter:{
    actorId:"gh_sentinel_arena_sentinel_01",phase,remainingMs:phase==="light_windup"?250:0,attackSerial:0,
    positionM:{xM:7.5,yM:0,zM:4.25},warning:phase==="light_windup"?{shape:"circle",originM:{xM:7.5,yM:0,zM:4.25},directionRad:0,radiusM:1.8,rangeM:0}:null}};
}
function nativeWarning(){return event(1,"SentinelAttackWindup",{actorId:"gh_sentinel_arena_sentinel_01",attackId:0,durationMs:250,radiusM:1.8,rangeM:0});}

test("SessionLoop pairs accepted events with their current v3 snapshot and drops presentation delivery while paused", async () => {
  const current = nativeSnapshot();
  const calls = [];
  const audioEvents = [];
  let clearCount = 0;
  const loop = new SessionLoop({ events: async () => [nativeWarning()], pause: async () => current }, { render: async () => {} }, {
    onEvents: events => audioEvents.push(...events),
    onEventsWithSnapshot: (events, pairedSnapshot) => calls.push({ events, pairedSnapshot }),
    onClearTransientPresentation: () => clearCount++,
    scheduler: { now: () => 0, clientTimeMs: () => 0, setInterval: () => 1, clearInterval: () => {},
      requestAnimationFrame: () => 1, cancelAnimationFrame: () => {} },
  });
  loop.snapshots.accept(current);
  await loop.pollEvents(current);
  assert.deepEqual(audioEvents.map(item => item.eventId), [1]);
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0].events.map(item => item.eventId), [1]);
  assert.equal(calls[0].pairedSnapshot, current);
  loop.paused = true;
  await loop.pollEvents(snapshot({ serverTick: 101 }));
  assert.equal(calls.length, 1);
  loop.active = true;
  await loop.pause();
  assert.equal(clearCount, 1);
  await loop.stop("unload");
  assert.equal(clearCount, 2);
});

test("SessionLoop sends only snapshot-authorized Sentinel warnings to the shared audio path", async () => {
  for (const pairedSnapshot of [nativeSnapshot(snapshot({ serverTick: 115 }), "chase"), snapshot({ sceneId: "gh_power_room" })]) {
    const current = pairedSnapshot;
    const warningModel = new SentinelTelegraphModel();
    const warning = event(1);
    const regular = event(2, "PlayerDamaged");
    const audioEvents = [];
    const loop = new SessionLoop({ events: async () => [warning, regular] }, { render: async () => {} }, {
      onEvents: events => audioEvents.push(...events),
      onEventsWithSnapshot: (events, latest) => {
        const authorized = new Set(warningModel.accept(events, latest));
        return events.filter(item =>
          (item.kind !== "SentinelAttackWindup" && item.kind !== "SentinelHeavyWindup") || authorized.has(item.eventId));
      },
      scheduler: { now: () => 0, clientTimeMs: () => 0, setInterval: () => 1, clearInterval: () => {},
        requestAnimationFrame: () => 1, cancelAnimationFrame: () => {} },
    });
    loop.snapshots.accept(current);
    await loop.pollEvents(current);
    assert.deepEqual(audioEvents.map(item => item.eventId), [2]);
  }
});
