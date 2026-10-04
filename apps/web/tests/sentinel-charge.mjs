import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { Assets, Container, Texture, TextureSource } from "pixi.js";
import { AssetRegistry } from "../dist/assets/AssetRegistry.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { SceneDefinitionSession } from "../dist/game/SceneDefinitionSession.js";
import { WorldRenderer, RenderCommitError } from "../dist/renderer/WorldRenderer.js";
import { assertSnapshotV3 } from "../dist/protocol/types.js";
import { assertSentinelEncounter, sentinelGeometryKey, sentinelWarningMatches } from "../dist/protocol/SentinelEncounter.js";
import { SnapshotClient } from "../dist/game/SnapshotClient.js";
import { SentinelTelegraphModel } from "../dist/renderer/SentinelTelegraphModel.js";
import { audioCueForPresentationEvent } from "../dist/audio/AudioEventMap.js";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const registry = new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(resolve(root, "governance/assets/RUNTIME_ASSET_MANIFEST.json"))),
  new Uint8Array(await readFile(resolve(root, "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"))),
  async path => new Uint8Array(await readFile(resolve(root, path))));
const scene = JSON.parse(await readFile(resolve(root, "content/scenes/compiled/gh_sentinel_arena.json"), "utf8"));
function snapshot(epoch = 1) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "gh_sentinel_arena", checkpointId: null, worldEpoch: epoch,
    serverTick: 1, authorityRevision: 1, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 4, yM: 0, zM: 8 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 80, maxHp: 100,
      currentEnergy: 60, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    sentinelEncounter: {actorId:"gh_sentinel_arena_sentinel_01",phase:"chase",remainingMs:0,attackSerial:0,positionM:{xM:12,yM:0,zM:8},warning:null},
    actors: [{entityId:"gh_sentinel_arena_sentinel_01",entityType:"enemy.grey_hive.sentinel",actorKind:"sentinel",active:true,transform:{positionM:{xM:12,yM:0,zM:8},yawRad:0}}], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] }, progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] } };
}
async function fixture(run) {
  const originals = Object.fromEntries(["window", "document", "ResizeObserver", "requestAnimationFrame", "cancelAnimationFrame"].map(k => [k, globalThis[k]]));
  const unload = Assets.unload;
  const frames = new Map(); let id = 0; const observers = [];
  const target = new EventTarget(); target.matchMedia = () => ({ matches: true });
  globalThis.window = target; globalThis.document = { documentElement: { dataset: { motion: "reduced" } } };
  globalThis.requestAnimationFrame = callback => { frames.set(++id, callback); return id; };
  globalThis.cancelAnimationFrame = frame => frames.delete(frame);
  globalThis.ResizeObserver = class { constructor(callback) { this.callback = callback; observers.push(this); } observe() {} disconnect() { this.disconnected = true; } };
  Assets.unload = async () => {};
  const canvas = new EventTarget(); canvas.parentElement = { clientWidth: 882, clientHeight: 552 };
  let draws = 0; let destroyed = false; let initOptions;
  const app = { stage: new Container(), screen: { width: 0, height: 0 },
    renderer: { background: {}, resize(width, height) { app.screen = { width, height }; canvas.width = width; canvas.height = height; } },
    async init(options) { initOptions = options; this.renderer.resize(options.width, options.height); },
    render() { assert.equal(destroyed, false); draws++; }, destroy() { destroyed = true; } };
  const renderer = new WorldRenderer(registry); renderer.app = app;
  renderer.preloadAtlasPages = async pages => {
    for (const page of pages) if (!renderer.atlasTextures.has(page.atlasUrl)) {
      renderer.atlasTextures.set(page.atlasUrl, new Texture({ source: new TextureSource({ width: 8192, height: 8192 }) }));
    }
  };
  const flush = async () => { const pending = [...frames.values()]; frames.clear(); for (const cb of pending) cb();
    await Promise.resolve(); await renderer.renderQueue; await Promise.resolve(); };
  try {
    await renderer.init(canvas); renderer.expectSceneIdentity(snapshot()); renderer.setSceneDefinition(scene);
    await run({ renderer, canvas, app, target, observers, frames, flush,
      options: () => initOptions, draws: () => draws, destroyed: () => destroyed });
  } finally {
    if (renderer.ready) await renderer.destroy();
    Assets.unload = unload;
    for (const [key, value] of Object.entries(originals)) { if (value === undefined) delete globalThis[key]; else globalThis[key] = value; }
  }
}


function phase(name="charge_windup",remainingMs=884,overrides={}) {
 const s=snapshot();s.sentinelEncounter={...s.sentinelEncounter,phase:name,remainingMs,attackSerial:1,
 warning:name.startsWith("charge")?{shape:"corridor",originM:{xM:12,yM:0,zM:8},directionRad:Math.PI/2,radiusM:.6,rangeM:3.6}:null,...overrides};return s;
}
function event(id=1,overrides={}) {return {protocolVersion:2,eventId:id,worldEpoch:1,serverTick:1,kind:"SentinelChargeWindup",actorId:"gh_sentinel_arena_sentinel_01",attackId:1,durationMs:884,positionM:{xM:12,yM:0,zM:8},directionRad:Math.PI/2,radiusM:.6,rangeM:3.6,intensity:1,...overrides};}

test("native Sentinel snapshot requires one exact actor and complete phase geometry",()=>{
 assert.doesNotThrow(()=>assertSnapshotV3(phase()));
 const missing=snapshot();delete missing.sentinelEncounter;assert.throws(()=>assertSnapshotV3(missing),/SENTINEL/);
 for(const change of [{phase:"unknown"},{remainingMs:-1},{remainingMs:885},{remainingMs:.5},{attackSerial:-1},{actorId:"wrong"},{positionM:{xM:13,yM:0,zM:8}},{warning:null},
 {warning:{...phase().sentinelEncounter.warning,rangeM:6.5}},{warning:{...phase().sentinelEncounter.warning,radiusM:.9}},{warning:{...phase().sentinelEncounter.warning,directionRad:NaN}}]) assert.throws(()=>assertSnapshotV3(phase("charge_windup",884,change)),/SENTINEL/);
 const duplicate=phase();duplicate.actors.push(duplicate.actors[0]);assert.throws(()=>assertSnapshotV3(duplicate),/SENTINEL/);
 const inactive=phase();inactive.actors[0].active=false;assert.throws(()=>assertSnapshotV3(inactive),/SENTINEL/);
 const outside=phase();outside.sceneId="gh_beacon";assert.throws(()=>assertSnapshotV3(outside),/SENTINEL/);
 for(const [name,radiusMs,time] of [["light_windup",1.8,250],["heavy_windup",2.2,400]]) assert.doesNotThrow(()=>assertSnapshotV3(phase(name,time,{warning:{shape:"circle",originM:{xM:12,yM:0,zM:8},radiusM:radiusMs,rangeM:0,directionRad:0}})));
});

test("charge motion retains its committed start, while hit/stagger discard warnings",()=>{
 const active=phase("charge",300);active.actors[0].transform.positionM={xM:13.8,yM:0,zM:8};active.sentinelEncounter.positionM={...active.actors[0].transform.positionM};assert.doesNotThrow(()=>assertSnapshotV3(active));
 assert.throws(()=>assertSnapshotV3({...active,sentinelEncounter:{...active.sentinelEncounter,positionM:{xM:13.8,yM:0,zM:9}}}),/SENTINEL/);
 for(const name of ["hit","stagger"]) {assert.doesNotThrow(()=>assertSnapshotV3(phase(name,17)));assert.throws(()=>assertSnapshotV3(phase(name,17,{warning:phase().sentinelEncounter.warning})),/SENTINEL/);}
});

test("warning audio is exact phase/serial geometry once, and malformed future IDs do not poison cursor",()=>{
 const model=new SentinelTelegraphModel(),s=phase();assert.deepEqual(model.accept([event()],s),[1]);assert.deepEqual(model.accept([event()],s),[]);assert.deepEqual(model.project(s,true),[],"native warning comes directly from snapshot");
 for(const change of [{attackId:2},{actorId:"wrong"},{radiusM:.9},{positionM:{xM:13,yM:0,zM:8}},{durationMs:0}]) assert.equal(sentinelWarningMatches(event(2,change),s),false);
 assert.equal(sentinelWarningMatches(event(),phase("stagger",200)),false);
 const client=new SnapshotClient();client.accept(s);
 assert.deepEqual(client.acceptEvents([event(100,{serverTick:999,actorId:"wrong"}),event(2)]).map(e=>e.eventId),[2]);
 assert.equal(audioCueForPresentationEvent(event()).id,"audio.enemy.sentinel.warning.charge");
});

test("future valid warning waits for its exact current phase and disappears on replacement",()=>{
 const client=new SnapshotClient();client.accept(snapshot());assert.deepEqual(client.acceptEvents([event(1,{serverTick:2})]),[]);
 client.accept({...phase(),serverTick:2,authorityRevision:2});assert.deepEqual(client.acceptEvents([]).map(e=>e.eventId),[1]);
 const other=new SnapshotClient();other.accept(snapshot());other.acceptEvents([event(1,{serverTick:2})]);other.accept({...snapshot(2),serverTick:2,authorityRevision:2});assert.deepEqual(other.acceptEvents([]),[]);
});

test("actual Pixi draw keeps one-tick warnings while paused and revokes stale phase proofs",async()=>fixture(async f=>{
 const value=phase("charge_windup",17);await f.renderer.render(value);
 assert.ok(f.renderer.sentinelEncounterGraphic);assert.match(f.renderer.sentinelEncounterLabel.text,/冲压预警/);
 assert.equal(f.renderer.sentinelEncounterGraphic.parent,f.renderer.layerContainers.get("L1_FLOOR"));
 assert.equal(f.renderer.sentinelEncounterGlow.parent,f.renderer.layerContainers.get("L3_ACTORS"));
 assert.equal(f.renderer.sentinelEncounterLabel.parent,f.renderer.layerContainers.get("L8_WORLD_UI"));
 assert.equal(f.renderer.sentinelEncounterGlow.zIndex,f.renderer.sprites.get("actor:gh_sentinel_arena_sentinel_01").sprite.zIndex+0.15);assert.equal(f.renderer.isReadyFor(value),true);
 f.renderer.clearTransientPresentation();await f.flush();assert.ok(f.renderer.sentinelEncounterGraphic);assert.equal(f.renderer.isReadyFor(value),true);
 assert.equal(f.renderer.isReadyFor({...value,authorityRevision:7}),true);
 const stagger=phase("stagger",120);assert.equal(f.renderer.isReadyFor(stagger),false);await f.renderer.render(stagger);assert.match(f.renderer.sentinelEncounterLabel.text,/硬直/);assert.equal(f.renderer.isReadyFor(stagger),true);
 await f.renderer.render(phase("stagger",17));assert.match(f.renderer.sentinelEncounterLabel.text,/硬直/);
 await f.renderer.render(phase("hit",17));assert.match(f.renderer.sentinelEncounterLabel.text,/受击/);
 await f.renderer.render(snapshot());assert.equal(f.renderer.sentinelEncounterGraphic,null);assert.equal(f.renderer.sentinelEncounterLabel,null);
}));

test("actual Sentinel body ignores unrelated interpolation and invalid render revokes earlier proof",async()=>fixture(async f=>{
 const original=f.renderer.updateCameraFrame;let fixed;f.renderer.updateCameraFrame=function(...args){return fixed??=(original.apply(this,args));};
 const value=phase();await f.renderer.render(value);const before={...f.renderer.sprites.get("actor:gh_sentinel_arena_sentinel_01").sprite.position};
 await f.renderer.render(value,undefined,new Map([["gh_sentinel_arena_sentinel_01",{xM:23,yM:0,zM:15}]]));const after=f.renderer.sprites.get("actor:gh_sentinel_arena_sentinel_01").sprite.position;assert.equal(after.x,before._x??before.x);assert.equal(after.y,before._y??before.y);
 const bad=structuredClone(value);delete bad.sentinelEncounter;await assert.rejects(f.renderer.render(bad),/SENTINEL/);assert.equal(f.renderer.isReadyFor(value),false);
}));

class TestSurface extends EventTarget { hidden=false; fire(name){this.dispatchEvent(new Event(name));} }
function released(value,revision=value.authorityRevision+1){const result=structuredClone(value);delete result.entryToken;result.authorityRevision=revision;return result;}
async function microtasks(){for(let i=0;i<60;i++)await Promise.resolve();}
function pending(){let resolve;const promise=new Promise(yes=>{resolve=yes;});return {promise,resolve};}

test("actual renderer held entry/resume proves one-tick and active-zero warning frames",async()=>fixture(async f=>{
 for(const [name,remaining] of [["charge_windup",17],["charge",600],["light_windup",17],["heavy_windup",17]]) {
  let state=name==="light_windup"||name==="heavy_windup"?phase(name,remaining,{warning:{shape:"circle",originM:{xM:12,yM:0,zM:8},directionRad:0,radiusM:name==="light_windup"?1.8:2.2,rangeM:0}}):phase(name,remaining);
  const initial=structuredClone(state);initial.entryToken={generation:1,worldId:initial.worldId,sceneId:initial.sceneId,worldEpoch:initial.worldEpoch};
  let held=true,resumes=0,ready=0;const frames=new Map(),intervals=new Map();let seq=0;
  const session=new SceneDefinitionSession(f.renderer,{loadForSnapshot:async()=>scene});
  const adapter={render:(...args)=>session.render(...args),isReadyFor:s=>session.isReadyFor(s),cancel(){},destroy(){}};
  const client={events:async()=>[],sceneReady:async(token,paused)=>{assert.equal(held,true);assert.ok(f.renderer.sentinelEncounterGraphic);assert.equal(f.renderer.isReadyFor(state),true);ready++;held=paused;state=released(state);return state;},
   pause:async()=>{held=true;return state;},resume:async()=>{assert.equal(held,true);assert.ok(f.renderer.sentinelEncounterGraphic);assert.equal(f.renderer.isReadyFor(state),true);resumes++;held=false;state=released(state);return state;}};
  const loop=new SessionLoop(client,adapter,{scheduler:{now:()=>0,clientTimeMs:()=>0,setInterval:cb=>{intervals.set(++seq,cb);return seq;},clearInterval:id=>intervals.delete(id),requestAnimationFrame:cb=>{frames.set(++seq,cb);return seq;},cancelAnimationFrame:id=>frames.delete(id)},documentTarget:new TestSurface(),windowTarget:new TestSurface(),onClearTransientPresentation:()=>f.renderer.clearTransientPresentation()});
  await loop.start(initial);assert.equal(ready,1);assert.equal(loop.pausePresentationState,"running");
  await loop.pause();assert.equal(held,true);assert.ok(f.renderer.sentinelEncounterGraphic);await loop.resume();assert.equal(resumes,1);assert.equal(loop.pausePresentationState,"running");await loop.stop();
 }
}));

test("deferred Ready with a new phase cannot reuse the prior real warning draw",async()=>fixture(async f=>{
 const initial=phase("charge_windup",17);initial.entryToken={generation:1,worldId:initial.worldId,sceneId:initial.sceneId,worldEpoch:initial.worldEpoch};
 const response=pending();const statuses=[];let pauses=0;const session=new SceneDefinitionSession(f.renderer,{loadForSnapshot:async()=>scene});
 const client={events:async()=>[],sceneReady:()=>response.promise,pause:async()=>{pauses++;return released(phase("stagger",200),5);}};
 const loop=new SessionLoop(client,{render:(...args)=>session.render(...args),isReadyFor:s=>session.isReadyFor(s),cancel(){},destroy(){}},{scheduler:{now:()=>0,clientTimeMs:()=>0,setInterval:()=>1,clearInterval(){},requestAnimationFrame:()=>1,cancelAnimationFrame(){}},documentTarget:new TestSurface(),windowTarget:new TestSurface(),onPauseState:s=>statuses.push(s)});
 const start=loop.start(initial);await microtasks();response.resolve(released(phase("stagger",200),5));await assert.rejects(start,/FRAME_NOT_READY/);assert.equal(statuses.includes("running"),false);assert.ok(pauses>=1);await loop.stop();
}));

test("inactive living Sentinel has no threat/death claim and clears a prior charge frame",async()=>fixture(async f=>{
 const active=phase();await f.renderer.render(active);assert.ok(f.renderer.sentinelEncounterGraphic);
 const inactive=snapshot();inactive.actors[0].active=false;inactive.sentinelEncounter.phase="inactive";
 assert.doesNotThrow(()=>assertSnapshotV3(inactive));await f.renderer.render(inactive);
 assert.equal(f.renderer.sentinelEncounterGraphic,null);assert.equal(f.renderer.sentinelEncounterLabel,null);assert.equal(f.renderer.isReadyFor(inactive),true);
 for(const bad of [{remainingMs:1},{warning:active.sentinelEncounter.warning}]) assert.throws(()=>assertSnapshotV3({...inactive,sentinelEncounter:{...inactive.sentinelEncounter,...bad}}),/SENTINEL/);
 const fake={...inactive,actors:[{...inactive.actors[0],active:true}]};assert.throws(()=>assertSnapshotV3(fake),/SENTINEL/);
}));


test("inactive native phase rejects new death cues and removes stale death presentation",()=>{
 const inactive=snapshot();inactive.actors[0].active=false;inactive.sentinelEncounter.phase="inactive";
 const model=new SentinelTelegraphModel();const death=event(1,{kind:"SentinelDeath"});
 model.accept([death],inactive);assert.deepEqual(model.project(inactive,true),[]);
 const dead=structuredClone(inactive);dead.sentinelEncounter.phase="dead";
 model.accept([death],dead);assert.equal(model.project(dead,true).length,1,"invalid inactive cue did not poison model cursor");
 assert.deepEqual(model.project(inactive,true),[],"stop displaying death when authority says inactive");
});
