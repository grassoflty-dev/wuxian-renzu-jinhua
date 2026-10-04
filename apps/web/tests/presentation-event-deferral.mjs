import test from "node:test";
import assert from "node:assert/strict";
import { SnapshotClient, MAX_DEFERRED_PRESENTATION_EVENTS } from "../dist/game/SnapshotClient.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { ClockworksEnemyModel } from "../dist/renderer/ClockworksEnemyModel.js";

function snapshot(tick = 100, overrides = {}) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "scene-v3/1", worldId: "clockworks",
    sceneId: "cw_pressure_hall", checkpointId: null, worldEpoch: 7, serverTick: tick, authorityRevision: tick, ackSeq: tick,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [{ entityId: "cw_drone_01", entityType: "enemy.clockworks.pressure_drone", actorKind: "enemy", active: true,
      transform: { positionM: { xM: 5, yM: 0, zM: 7 }, yawRad: 0 } }], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "clockworks", eventSeq: 0, worlds: [] }, ...overrides };
}
function event(id = 1, tick = 101, overrides = {}) {
  return { protocolVersion: 2, eventId: id, worldEpoch: 7, serverTick: tick, kind: "EnemyAttackTelegraph",
    actorId: "cw_drone_01", attackKind: "pressure_shot", durationMs: 600, radiusM: .16, rangeM: 8,
    directionRad: 0, intensity: 1, positionM: { xM: 5, yM: 0, zM: 7 }, ...overrides };
}
const scheduler = { now: () => 0, clientTimeMs: () => 0, setInterval: () => 1, clearInterval() {}, requestAnimationFrame: () => 1, cancelAnimationFrame() {} };
function deferred() { let resolve; const promise = new Promise(yes => { resolve = yes; }); return { promise, resolve }; }
function session(events) {
  const delivered = [], paired = [], requests = []; const model = new ClockworksEnemyModel();
  const loop = new SessionLoop({ events: async (epoch, cursor) => { requests.push({ epoch, cursor }); return events(epoch, cursor); },
    pause: async () => snapshot() }, { render: async () => {} }, {
    scheduler, onEvents: rows => delivered.push(...rows), onEventsWithSnapshot: (rows, current) => {
      paired.push(current); const ids = new Set(model.accept(rows, current)); return rows.filter(row => ids.has(row.eventId));
    }, onClearTransientPresentation: () => model.reset(),
  });
  return { loop, delivered, paired, requests, model };
}

test("future model cue remains retryable at catch-up and invalid high IDs cannot poison ordering", () => {
  const model = new ClockworksEnemyModel(); const cue = event();
  assert.deepEqual(model.accept([cue], snapshot()), []); assert.deepEqual(model.accept([cue], snapshot(101)), [1]);
  const invalid = [event(999,100,{radiusM:-1}), event(999,100,{durationMs:NaN}), event(999,100,{rangeM:undefined}),
    event(999,100,{actorId:"wrong"}), event(999,100,{kind:"Unknown"}), event(999,101), event(NaN,100), event(Infinity,100),
    event(-1,100), event(.5,100), event(Number.MAX_SAFE_INTEGER+1,100), null, {}];
  for (const row of invalid) { const m = new ClockworksEnemyModel(); assert.deepEqual(m.accept([row],snapshot()),[]); assert.deepEqual(m.accept([event(2,100)],snapshot()),[2]); }
});

test("global cursor validates malformed IDs and ordinary wire fields before mutation", () => {
  const c = new SnapshotClient(); c.accept(snapshot());
  const invalid = [event(999,100,{radiusM:-1}),event(999,100,{durationMs:NaN}),event(999,100,{rangeM:undefined}),
    event(999,100,{actorId:undefined}),event(NaN,100),event(-1,100),event(.5,100),event(Infinity,100),null,{}];
  assert.deepEqual(c.acceptEvents(invalid),[]); assert.equal(c.eventCursor(),0);
  assert.deepEqual(c.acceptEvents([event(2,100)]).map(e=>e.eventId),[2]); assert.equal(c.eventCursor(),2);
});

test("global cursor cannot skip an earlier future cue when a later ID is already current", () => {
  const c=new SnapshotClient();c.accept(snapshot());
  assert.deepEqual(c.acceptEvents([event(3,100),event(2,101),event(1,100)]).map(e=>e.eventId),[1]);
  assert.equal(c.eventCursor(),1);assert.equal(c.deferredEventCount(),2);c.accept(snapshot(101));
  assert.deepEqual(c.acceptEvents([]).map(e=>e.eventId),[2,3]);assert.equal(c.eventCursor(),3);assert.equal(c.deferredEventCount(),0);
});

test("bounded queue overflow stays refetchable at the unchanged cursor without a time-lead cutoff", () => {
  const c=new SnapshotClient();c.accept(snapshot());const count=MAX_DEFERRED_PRESENTATION_EVENTS+17;
  const rows=Array.from({length:count},(_,i)=>event(i+1));assert.deepEqual(c.acceptEvents(rows),[]);
  assert.equal(c.eventCursor(),0);assert.equal(c.deferredEventCount(),MAX_DEFERRED_PRESENTATION_EVENTS);
  c.accept(snapshot(101));const first=c.acceptEvents([]);assert.equal(first.length,MAX_DEFERRED_PRESENTATION_EVENTS);
  const rest=c.acceptEvents(rows.filter(e=>e.eventId>c.eventCursor()));assert.equal(first.length+rest.length,count);assert.equal(c.eventCursor(),count);
  const distant=new SnapshotClient();distant.accept(snapshot());assert.deepEqual(distant.acceptEvents([event(1,100000)]),[]);
  assert.equal(distant.deferredEventCount(),1);assert.equal(distant.eventCursor(),0);
});

test("real SessionLoop delivers queued future warning once after later snapshot catch-up",async()=>{
  let reads=0;const h=session(()=>++reads===1?[event()]:[]);await h.loop.start(snapshot());
  assert.equal(h.delivered.length,0);assert.equal(h.loop.snapshots.eventCursor(),0);assert.equal(h.loop.snapshots.deferredEventCount(),1);
  await h.loop.acceptAuthoritativeSnapshot(snapshot(101));assert.deepEqual(h.delivered.map(e=>e.eventId),[1]);
  assert.equal(h.paired[0].serverTick,101);assert.equal(h.loop.snapshots.eventCursor(),1);assert.equal(h.model.project(snapshot(101),true).length,1);
  await h.loop.acceptAuthoritativeSnapshot(snapshot(102));assert.equal(h.delivered.length,1);assert.deepEqual(h.requests.map(e=>e.cursor),[0,0,1]);await h.loop.stop("unload");
});

test("real SessionLoop drops deferred rows on scene, world and epoch replacement",async()=>{
  for(const change of [{sceneId:"cw_boiler_chamber"},{worldId:"mist_harbor",sceneId:"mh_signal_yard"},{worldEpoch:8}]){
    let reads=0;const h=session(()=>++reads===1?[event()]:[]);await h.loop.start(snapshot());assert.equal(h.loop.snapshots.deferredEventCount(),1);
    await h.loop.acceptAuthoritativeSnapshot(snapshot(101,change));assert.equal(h.loop.snapshots.deferredEventCount(),0);assert.equal(h.delivered.length,0);await h.loop.stop("unload");
  }
});

test("real SessionLoop generation rejects late polls after scene A to B to A replacement",async()=>{
  const late=deferred();let reads=0;const h=session(()=>++reads===2?late.promise:[]);await h.loop.start(snapshot());
  const poll=h.loop.pollEvents(snapshot());await h.loop.acceptAuthoritativeSnapshot(snapshot(101,{sceneId:"cw_boiler_chamber"}));
  await h.loop.acceptAuthoritativeSnapshot(snapshot(102));late.resolve([event()]);await poll;
  assert.equal(h.delivered.length,0);assert.equal(h.loop.snapshots.eventCursor(),0);assert.equal(h.loop.snapshots.deferredEventCount(),0);await h.loop.stop("unload");
});

test("real SessionLoop stop discards queued cues and late event responses",async()=>{
  const late=deferred();let reads=0;const h=session(()=>++reads===1?[event()]:late.promise);await h.loop.start(snapshot());
  assert.equal(h.loop.snapshots.deferredEventCount(),1);const poll=h.loop.pollEvents(snapshot());await h.loop.stop("unload");
  assert.equal(h.loop.snapshots.deferredEventCount(),0);late.resolve([event(2)]);await poll;assert.equal(h.delivered.length,0);assert.equal(h.loop.snapshots.eventCursor(),0);
});

test("real SessionLoop invalid current actor/context high IDs cannot hide a later valid lower ID",async()=>{
  for(const patch of [{actorId:"wrong"},{attackKind:"bite"},{kind:"FurnaceHoundLeapMotion",attackKind:"leap"}]){
    let reads=0;const h=session(()=>++reads===1?[event(999,100,patch)]:[event(2,101)]);
    await h.loop.start(snapshot());assert.equal(h.loop.snapshots.eventCursor(),0);assert.equal(h.delivered.length,0);
    await h.loop.acceptAuthoritativeSnapshot(snapshot(101));assert.deepEqual(h.delivered.map(e=>e.eventId),[2]);
    assert.equal(h.loop.snapshots.eventCursor(),2);await h.loop.stop("unload");
  }
  // Generic unknown kinds retain wire forward compatibility independently.
  const c=new SnapshotClient();c.accept(snapshot());
  assert.deepEqual(c.acceptEvents([event(1,100,{kind:"FutureGenericCue",actorId:undefined})]).map(e=>e.eventId),[1]);
});
