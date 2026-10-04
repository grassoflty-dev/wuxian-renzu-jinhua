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
const root = fileURLToPath(new URL("../../../", import.meta.url));
const registry = new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(resolve(root, "governance/assets/RUNTIME_ASSET_MANIFEST.json"))),
  new Uint8Array(await readFile(resolve(root, "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"))),
  async path => new Uint8Array(await readFile(resolve(root, path))));
const scene = JSON.parse(await readFile(resolve(root, "content/scenes/compiled/rs_core_room.json"), "utf8"));
function snapshot(epoch = 1) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "return_station", sceneId: "rs_core_room", checkpointId: null, worldEpoch: epoch,
    serverTick: 1, authorityRevision: 1, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 4, yM: 0, zM: 8 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 80, maxHp: 100,
      currentEnergy: 60, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] }, progression: { schemaVersion: 1, currentWorldId: "return_station", eventSeq: 0, worlds: [] } };
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

test("actual renderer commits once, remains idle when paused, and resumes with a fresh frame", async () => fixture(async f => {
  assert.equal(f.options().autoStart, false); assert.equal(f.options().sharedTicker, false);
  assert.equal(f.options().resizeTo, undefined);
  await f.renderer.render(snapshot()); assert.equal(f.draws(), 1);
  f.renderer.clearTransientPresentation(); f.renderer.clearSoundCues(); await f.flush();
  assert.equal(f.draws(), 2); assert.equal(f.frames.size, 0);
  for (let i = 0; i < 20; i++) await f.flush(); assert.equal(f.draws(), 2);
  await f.renderer.render({ ...snapshot(), serverTick: 2 }); assert.equal(f.draws(), 3);
  assert.equal(f.renderer.transientsCleared, false);
}));

test("paused resize reprojects once at all acceptance dimensions and keeps reduced motion frozen", async () => fixture(async f => {
  await f.renderer.render(snapshot()); f.renderer.clearTransientPresentation(); await f.flush();
  const time = f.renderer.lastFrame.timeMs;
  for (const [width, height] of [[882,552], [1458,829], [2034,1107], [882,552]]) {
    const before = f.draws(); const changed = width !== f.app.screen.width || height !== f.app.screen.height;
    Object.assign(f.canvas.parentElement, { clientWidth: width, clientHeight: height });
    f.target.dispatchEvent(new Event("resize")); f.observers[0].callback(); await f.flush();
    assert.deepEqual(f.app.screen, { width, height }); assert.equal(f.draws(), before + (changed ? 1 : 0));
    assert.equal(f.renderer.lastFrame.timeMs, time); assert.equal(f.renderer.transientsCleared, true);
  }
}));

test("context loss revokes readiness and restore cannot automatically revive the renderer", async () => fixture(async f => {
  const value = snapshot();
  assert.equal(f.renderer.isReadyFor(value), false);
  await f.renderer.render(value); const before = f.draws();
  assert.equal(f.renderer.isReadyFor(value), true);
  const lost = new Event("webglcontextlost", { cancelable: true }); f.canvas.dispatchEvent(lost);
  assert.equal(lost.defaultPrevented, true); assert.equal(f.renderer.isReadyFor(value), false);
  await assert.rejects(f.renderer.render({ ...snapshot(), serverTick: 2 }), error =>
    error instanceof RenderCommitError && error.code === "E_RENDERER_CONTEXT_LOST");
  assert.equal(f.draws(), before); assert.equal(f.renderer.lastFrame, null);
  f.canvas.dispatchEvent(new Event("webglcontextrestored")); await f.flush();
  assert.equal(f.draws(), before); assert.equal(f.renderer.isReadyFor(value), false);
  await assert.rejects(f.renderer.render(value), /E_RENDERER_CONTEXT_LOST/);
}));

test("scene change and disposal revoke the exact proof and reject obsolete frames", async () => fixture(async f => {
  const value = snapshot(); await f.renderer.render(value);
  for (const other of [{ ...value, worldId: "grey_hive" }, { ...value, sceneId: "other" }, snapshot(2)]) {
    assert.equal(f.renderer.isReadyFor(other), false);
  }
  f.renderer.expectSceneIdentity(snapshot(2)); assert.equal(f.renderer.isReadyFor(value), false);
  await f.renderer.sceneInvalidationCleanup; f.renderer.setSceneDefinition(scene);
  await assert.rejects(f.renderer.render(value), /E_RENDERER_STALE_FRAME/); assert.equal(f.renderer.lastFrame, null);
  await f.renderer.render(snapshot(2)); assert.equal(f.renderer.isReadyFor(snapshot(2)), true);
  Object.assign(f.canvas.parentElement, { clientWidth: 1000 }); f.target.dispatchEvent(new Event("resize"));
  const pending = [...f.frames.values()]; await f.renderer.destroy(); const total = f.draws();
  assert.equal(f.renderer.isReadyFor(snapshot(2)), false);
  await assert.rejects(f.renderer.render(snapshot(2)), /E_RENDERER_NOT_READY/);
  for (const callback of pending) callback(); f.canvas.dispatchEvent(new Event("webglcontextrestored")); await f.flush();
  assert.equal(f.draws(), total); assert.equal(f.destroyed(), true); assert.equal(f.observers[0].disconnected, true);
}));

test("a frame queued before pause cannot reactivate cleared transient effects", async () => fixture(async f => {
  await f.renderer.render(snapshot());
  let release; f.renderer.renderQueue = new Promise(resolve => { release = resolve; });
  const pending = f.renderer.render({ ...snapshot(), serverTick: 2 });
  f.renderer.clearTransientPresentation(); release(); await pending;
  assert.equal(f.renderer.transientsCleared, true); await f.flush();
  assert.equal(f.frames.size, 0);
}));

test("late atlas completion after destroy releases its lease without reviving textures", async () => fixture(async f => {
  const originalLoad = Assets.load, originalUnload = Assets.unload;
  let resolveLoad; const unloaded = [];
  Assets.load = () => new Promise(resolve => { resolveLoad = resolve; });
  Assets.unload = async url => { unloaded.push(url); };
  try {
    f.renderer.preloadAtlasPages = WorldRenderer.prototype.preloadAtlasPages;
    const pending = f.renderer.preloadAtlasPages([{ atlasUrl: "late.webp" }]);
    await Promise.resolve(); await f.renderer.destroy();
    resolveLoad(new Texture({ source: new TextureSource({ width: 2, height: 2 }) })); await pending;
    assert.equal(f.renderer.atlasTextures.size, 0); assert.equal(f.renderer.atlasLoadOwners.size, 0);
    assert.equal(f.renderer.atlasPendingUnloads.size, 0); assert.deepEqual(unloaded, ["late.webp"]);
    assert.equal(f.draws(), 0);
  } finally { Assets.load = originalLoad; Assets.unload = originalUnload; }
}));

test("retiring an old epoch never unloads an atlas leased by the current epoch", async () => fixture(async f => {
  const originalLoad = Assets.load, originalUnload = Assets.unload;
  let resolveLoad; const unloaded = [];
  const shared = new Promise(resolve => { resolveLoad = resolve; });
  Assets.load = () => shared; Assets.unload = async url => { unloaded.push(url); };
  try {
    f.renderer.preloadAtlasPages = WorldRenderer.prototype.preloadAtlasPages;
    const old = f.renderer.preloadAtlasPages([{ atlasUrl: "shared.webp" }]);
    await Promise.resolve();
    f.renderer.expectSceneIdentity(snapshot(2)); await f.renderer.sceneInvalidationCleanup;
    const current = f.renderer.preloadAtlasPages([{ atlasUrl: "shared.webp" }]);
    const texture = new Texture({ source: new TextureSource({ width: 2, height: 2 }) });
    resolveLoad(texture); await Promise.all([old, current]);
    assert.equal(f.renderer.atlasTextures.get("shared.webp"), texture);
    assert.deepEqual(unloaded, []); assert.equal(f.renderer.atlasLoadOwners.size, 0);
  } finally { Assets.load = originalLoad; Assets.unload = originalUnload; }
}));

test("failed atlas batches drain before retirement and never publish partial textures", async () => fixture(async f => {
  const originalLoad = Assets.load, originalUnload = Assets.unload;
  let resolveOther; const unloaded = [];
  const error = new Error("bad atlas");
  Assets.load = url => url === "bad.webp" ? Promise.reject(error) : new Promise(resolve => { resolveOther = resolve; });
  Assets.unload = async url => { unloaded.push(url); };
  try {
    f.renderer.preloadAtlasPages = WorldRenderer.prototype.preloadAtlasPages;
    const pending = f.renderer.preloadAtlasPages([{ atlasUrl: "bad.webp" }, { atlasUrl: "other.webp" }]);
    const rejected = assert.rejects(pending, /bad atlas/);
    await Promise.resolve(); await Promise.resolve();
    assert.deepEqual(unloaded, []); assert.equal(f.renderer.atlasTextures.size, 0);
    resolveOther(new Texture({ source: new TextureSource({ width: 2, height: 2 }) })); await rejected;
    assert.deepEqual(unloaded.sort(), ["bad.webp", "other.webp"]); assert.equal(f.renderer.atlasTextures.size, 0);
    assert.equal(f.renderer.atlasLoadOwners.size, 0);
  } finally { Assets.load = originalLoad; Assets.unload = originalUnload; }
}));

test("a new epoch waits for an already-started retirement before obtaining a fresh texture", async () => fixture(async f => {
  const originalLoad = Assets.load, originalUnload = Assets.unload;
  let releaseFirst, releaseUnload; let loads = 0, unloads = 0;
  const oldTexture = new Texture({ source: new TextureSource({ width: 2, height: 2 }) });
  const freshTexture = new Texture({ source: new TextureSource({ width: 2, height: 2 }) });
  Assets.load = async () => { loads++; return loads === 1 ? await new Promise(resolve => { releaseFirst = resolve; }) : freshTexture; };
  Assets.unload = async () => { unloads++; await new Promise(resolve => { releaseUnload = resolve; }); };
  try {
    f.renderer.preloadAtlasPages = WorldRenderer.prototype.preloadAtlasPages;
    const old = f.renderer.preloadAtlasPages([{ atlasUrl: "retiring.webp" }]); await Promise.resolve();
    f.renderer.expectSceneIdentity(snapshot(2)); await f.renderer.sceneInvalidationCleanup;
    releaseFirst(oldTexture);
    for (let i = 0; i < 12 && !releaseUnload; i++) await Promise.resolve();
    assert.equal(unloads, 1); assert.ok(releaseUnload);
    const current = f.renderer.preloadAtlasPages([{ atlasUrl: "retiring.webp" }]);
    await Promise.resolve(); await Promise.resolve(); assert.equal(loads, 1);
    releaseUnload(); await Promise.all([old, current]);
    assert.equal(loads, 2); assert.equal(f.renderer.atlasTextures.get("retiring.webp"), freshTexture);
    assert.notEqual(f.renderer.atlasTextures.get("retiring.webp"), oldTexture);
  } finally { Assets.load = originalLoad; Assets.unload = originalUnload; }
}));

test("a destroyed renderer cannot unload a texture acquired by its replacement renderer", async () => fixture(async f => {
  const originalLoad = Assets.load, originalUnload = Assets.unload;
  let release; const shared = new Promise(resolve => { release = resolve; }); const unloaded = [];
  Assets.load = () => shared; Assets.unload = async url => { unloaded.push(url); };
  const replacement = new WorldRenderer(registry); replacement.ready = true;
  try {
    f.renderer.preloadAtlasPages = WorldRenderer.prototype.preloadAtlasPages;
    const old = f.renderer.preloadAtlasPages([{ atlasUrl: "replacement.webp" }]); await Promise.resolve();
    await f.renderer.destroy();
    const current = replacement.preloadAtlasPages([{ atlasUrl: "replacement.webp" }]);
    const texture = new Texture({ source: new TextureSource({ width: 2, height: 2 }) });
    release(texture); await Promise.all([old, current]);
    assert.equal(replacement.atlasTextures.get("replacement.webp"), texture);
    assert.deepEqual(unloaded, []);
    replacement.ready = false; await replacement.destroy();
    assert.deepEqual(unloaded, ["replacement.webp"]);
  } finally { replacement.ready = false; await replacement.destroy(); Assets.load = originalLoad; Assets.unload = originalUnload; }
}));

test("atlas retirement failures are reported instead of swallowed", async () => fixture(async f => {
  const originalLoad = Assets.load, originalUnload = Assets.unload;
  let release; const error = new Error("unload failed");
  Assets.load = () => new Promise(resolve => { release = resolve; }); Assets.unload = async () => { throw error; };
  try {
    f.renderer.preloadAtlasPages = WorldRenderer.prototype.preloadAtlasPages;
    const pending = f.renderer.preloadAtlasPages([{ atlasUrl: "unload-failure.webp" }]);
    const rejected = assert.rejects(pending, /unload failed/); await Promise.resolve(); await f.renderer.destroy();
    release(new Texture({ source: new TextureSource({ width: 2, height: 2 }) })); await rejected;
    assert.equal(f.renderer.atlasTextures.size, 0); assert.equal(f.renderer.atlasLoadOwners.size, 0);
    assert.equal(f.renderer.atlasPendingUnloads.size, 0);
  } finally { Assets.load = originalLoad; Assets.unload = originalUnload; }
}));


test("missing surface, refused presentation and draw exceptions cannot create readiness", async () => fixture(async f => {
  const surface = f.renderer.surface;
  f.renderer.surface = null;
  await assert.rejects(f.renderer.render(snapshot()), error =>
    error instanceof RenderCommitError && error.code === "E_RENDERER_NO_SURFACE");
  assert.equal(f.renderer.lastFrame, null); assert.equal(f.renderer.isReadyFor(snapshot()), false);
  f.renderer.surface = surface;
  const present = surface.present.bind(surface); surface.present = () => false;
  await assert.rejects(f.renderer.render(snapshot()), /E_RENDERER_PRESENT_FAILED/);
  assert.equal(f.draws(), 0); assert.equal(f.renderer.lastFrame, null); assert.equal(f.renderer.isReadyFor(snapshot()), false);
  surface.present = present;
  const draw = f.app.render; f.app.render = () => { throw new Error("draw failed"); };
  await assert.rejects(f.renderer.render(snapshot()), /draw failed/);
  assert.equal(f.renderer.isReadyFor(snapshot()), false);
  f.app.render = draw; await f.renderer.render(snapshot()); assert.equal(f.renderer.isReadyFor(snapshot()), true);
}));

test("a frame queued before definition replacement rejects even for the same identity", async () => fixture(async f => {
  let release; f.renderer.renderQueue = new Promise(resolve => { release = resolve; });
  const pending = assert.rejects(f.renderer.render(snapshot()), /E_RENDERER_STALE_FRAME/);
  f.renderer.setSceneDefinition(scene); release(); await pending;
  assert.equal(f.draws(), 0); assert.equal(f.renderer.isReadyFor(snapshot()), false);
  await f.renderer.render(snapshot()); assert.equal(f.draws(), 1);
}));

test("cancel during atlas loading prevents the first frame and revokes completed readiness immediately", async () => fixture(async f => {
  const preload = f.renderer.preloadAtlasPages.bind(f.renderer); let release; let entered;
  const atLoad = new Promise(resolve => { entered = resolve; });
  f.renderer.preloadAtlasPages = async pages => { entered(); await new Promise(resolve => { release = resolve; }); await preload(pages); };
  const pending = assert.rejects(f.renderer.render(snapshot()), /E_RENDERER_CANCELLED/);
  await atLoad; f.renderer.invalidate();
  assert.equal(f.renderer.isReadyFor(snapshot()), false); release(); await pending;
  assert.equal(f.draws(), 0); assert.equal(f.renderer.lastFrame, null);
}));

test("context loss during drawing is not a successful present", async () => fixture(async f => {
  const draw = f.app.render;
  f.app.render = () => { draw(); f.canvas.dispatchEvent(new Event("webglcontextlost", { cancelable: true })); };
  await assert.rejects(f.renderer.render(snapshot()), /E_RENDERER_CONTEXT_LOST/);
  assert.equal(f.renderer.isReadyFor(snapshot()), false); assert.equal(f.renderer.lastFrame, null);
}));

test("a first frame suspended on atlas work rejects after an authoritative epoch change", async () => fixture(async f => {
  const preload = f.renderer.preloadAtlasPages.bind(f.renderer); let release; let entered;
  const atLoad = new Promise(resolve => { entered = resolve; });
  f.renderer.preloadAtlasPages = async pages => { entered(); await new Promise(resolve => { release = resolve; }); await preload(pages); };
  const old = assert.rejects(f.renderer.render(snapshot()), /E_RENDERER_STALE_FRAME/);
  await atLoad; f.renderer.expectSceneIdentity(snapshot(2)); await f.renderer.sceneInvalidationCleanup;
  f.renderer.setSceneDefinition(scene); release(); await old;
  assert.equal(f.draws(), 0); assert.equal(f.renderer.lastFrame, null);
  f.renderer.preloadAtlasPages = preload;
  await f.renderer.render(snapshot(2)); assert.equal(f.renderer.isReadyFor(snapshot(2)), true);
}));


test("an expected identity without its verified definition cannot commit a fallback scene", async () => fixture(async f => {
  f.renderer.expectSceneIdentity(snapshot(2));
  await assert.rejects(f.renderer.render(snapshot(2)), /E_RENDERER_SCENE_NOT_READY/);
  assert.equal(f.draws(), 0); assert.equal(f.renderer.isReadyFor(snapshot(2)), false);
  f.renderer.setSceneDefinition(scene); await f.renderer.render(snapshot(2));
  assert.equal(f.renderer.isReadyFor(snapshot(2)), true);
}));

test("a failed render revokes earlier readiness even when identity has not changed", async () => fixture(async f => {
  await f.renderer.render(snapshot()); assert.equal(f.renderer.isReadyFor(snapshot()), true);
  f.renderer.preloadAtlasPages = async () => { throw new Error("asset failed"); };
  await assert.rejects(f.renderer.render(snapshot()), /asset failed/);
  assert.equal(f.renderer.isReadyFor(snapshot()), false);
}));

async function waitForState(predicate) {
  for(let i=0;i<150;i++){if(predicate())return;await Promise.resolve();}
  assert.fail('readiness lifecycle did not settle');
}
function entryLoopFixture(f, options={}) {
  const view={...snapshot(),entryToken:{generation:1,worldId:'return_station',sceneId:'rs_core_room',worldEpoch:1}};
  const released={...snapshot(),authorityRevision:2};const order=[];const intervals=new Map(),frames=new Map();let next=0;
  const documentTarget=new EventTarget();documentTarget.hidden=true;documentTarget.hasFocus=()=>!documentTarget.hidden;
  const draw=f.app.render;f.app.render=()=>{order.push(['draw',documentTarget.hidden]);draw();};
  const session=new SceneDefinitionSession(f.renderer,{loadForSnapshot:async()=>scene});
  let lifecycleRevision=released.authorityRevision;
  const client={events:async()=>[],submitInput:async()=>released,submitAction:async()=>released,
    sceneReady:async(_token,remainPaused)=>{order.push(['ready',remainPaused]);return released;},
    resume:async()=>{order.push(['resume']);lifecycleRevision++;return await options.resume?.()??{...released,authorityRevision:lifecycleRevision};},
    pause:async()=>{order.push(['pause']);return {...released,authorityRevision:lifecycleRevision};}};
  const loop=new SessionLoop(client,session,{windowTarget:f.target,documentTarget,onSnapshot:value=>session.acceptSnapshot(value),scheduler:{now:()=>0,clientTimeMs:()=>0,setInterval:cb=>{const id=++next;intervals.set(id,cb);return id;},clearInterval:id=>intervals.delete(id),requestAnimationFrame:cb=>{const id=++next;frames.set(id,cb);return id;},cancelAnimationFrame:id=>frames.delete(id)}});
  return {view,loop,order,intervals,frames,documentTarget,focus:()=>{documentTarget.hidden=false;f.target.dispatchEvent(new Event('focus'));}};
}

test('real scene session and renderer re-present hidden entry visibly before formal resume',async()=>fixture(async f=>{
  const h=entryLoopFixture(f);await h.loop.start(h.view);assert.deepEqual(h.order,[['draw',true],['ready',true]]);assert.equal(h.loop.pausePresentationState,'paused');assert.equal(h.intervals.size,0);assert.equal(h.frames.size,0);
  h.focus();await waitForState(()=>h.loop.pausePresentationState==='running');assert.deepEqual(h.order,[['draw',true],['ready',true],['pause'],['draw',false],['resume']]);assert.equal(h.intervals.size,1);await h.loop.stop();
}));

test('real renderer loss after hidden entry prevents focus resume and keeps schedulers closed',async()=>fixture(async f=>{
  const h=entryLoopFixture(f);await h.loop.start(h.view);f.canvas.dispatchEvent(new Event('webglcontextlost',{cancelable:true}));h.focus();await waitForState(()=>h.loop.pausePresentationState==='error');assert.equal(h.order.some(x=>x[0]==='resume'),false);assert.equal(h.intervals.size,0);assert.equal(h.frames.size,0);await h.loop.stop();
}));

test('real renderer loss while visible resume receipt is delayed compensates authoritative pause',async()=>fixture(async f=>{
  let resolveResume;const resume=new Promise(resolve=>{resolveResume=resolve;});const h=entryLoopFixture(f,{resume:()=>resume});await h.loop.start(h.view);h.focus();await waitForState(()=>h.order.some(x=>x[0]==='resume'));
  assert.equal(h.intervals.size,0);f.canvas.dispatchEvent(new Event('webglcontextlost',{cancelable:true}));resolveResume({...snapshot(),authorityRevision:3});await waitForState(()=>h.loop.pausePresentationState==='error');assert.equal(h.order.at(-1)[0],'pause');assert.equal(h.intervals.size,0);assert.equal(h.frames.size,0);await h.loop.stop();
}));

test('pointer aim uses only the committed drawn footpoint and includes elevated player Y', async () => fixture(async f => {
  const value = snapshot();
  const rect = {left:10,top:20,width:882,height:552};
  assert.equal(f.renderer.pointerAim(value,10,20,rect), null);
  const displayed = {xM:5,yM:2,zM:8};
  await f.renderer.render(value,displayed);
  const anchor=f.renderer.committedFrame.playerAim;
  assert.notEqual(anchor.footX, f.app.screen.width/2);
  assert.deepEqual(f.renderer.pointerAim({...value,player:{...value.player,transform:{...value.player.transform,positionM:{xM:99,yM:0,zM:99}}}},
    rect.left+anchor.footX,rect.top+anchor.footY,rect),{x:0,y:0},'undrawn new position must not steer');
  const player=f.renderer.sprites.get('actor:player').sprite;
  assert.equal(anchor.footX,player.x); assert.equal(anchor.footY,player.y);
  assert.equal(f.renderer.playerContactShadow.y,player.y);
  assert.equal(f.renderer.playerContactShadow.parent,player.parent);
  assert.equal(f.renderer.pointerAim(snapshot(2),10,20,rect),null);
}));

test('pending, failed and resized frames cannot publish an undrawn aim anchor', async () => fixture(async f => {
  const value=snapshot();const rect={left:0,top:0,width:882,height:552};await f.renderer.render(value);
  const anchor={...f.renderer.committedFrame.playerAim};
  const preload=f.renderer.preloadAtlasPages;let release;let entered=false;
  f.renderer.preloadAtlasPages=async pages=>{entered=true;await new Promise(resolve=>{release=resolve;});return preload(pages);};
  const pending=f.renderer.render({...value,serverTick:2}, {xM:8,yM:0,zM:8});
  while(!entered) await Promise.resolve();
  assert.deepEqual(f.renderer.pointerAim(value,anchor.footX,anchor.footY,rect),{x:0,y:0});
  release();await pending;assert.notEqual(f.renderer.committedFrame.playerAim.footX,anchor.footX);
  f.renderer.preloadAtlasPages=preload;
  f.app.renderer.resize(1000,600);
  assert.equal(f.renderer.pointerAim(value,0,0,rect),null,'resized buffer has no committed geometry yet');
  await f.renderer.render(value);
  const oldDraw=f.app.render;f.app.render=()=>{throw new Error('draw failed');};
  await assert.rejects(f.renderer.render({...value,serverTick:3}),/draw failed/);
  assert.equal(f.renderer.pointerAim(value,0,0,rect),null);
  f.app.render=oldDraw;
}));

test('bound static enemy previews do not survive movement or death of the authoritative enemy', async () => fixture(async f => {
  const definition=JSON.parse(await readFile(resolve(root,'content/scenes/compiled/gh_entry_maintenance.json'),'utf8'));
  const value={...snapshot(),worldId:'grey_hive',sceneId:'gh_entry_maintenance'};
  f.renderer.expectSceneIdentity(value);f.renderer.setSceneDefinition(definition);
  const worker={entityId:'gh_entry_worker_01',entityType:'runtime2d.enemy.infected_maintenance_worker.v1',actorKind:'enemy',active:true,
    transform:{positionM:{xM:9,yM:0,zM:5},yawRad:0}};
  await f.renderer.render({...value,actors:[worker]});
  assert.equal(f.renderer.sprites.has('scene:entry_worker_01_visual'),false);
  assert.equal(f.renderer.sprites.has('actor:gh_entry_worker_01'),true);
  await f.renderer.render({...value,serverTick:2,actors:[{...worker,active:false}]});
  assert.equal(f.renderer.sprites.has('scene:entry_worker_01_visual'),false);
  assert.equal(f.renderer.sprites.has('actor:gh_entry_worker_01'),false);
}));

test('scene session exposes pointer geometry only after its own committed entry frame', async () => fixture(async f => {
  const value=snapshot();
  const session=new SceneDefinitionSession(f.renderer,{loadForSnapshot:async()=>scene});
  const rect={left:0,top:0,width:882,height:552};
  await session.prepare(value);
  assert.equal(session.pointerAim(value,10,20,rect),null);
  await session.render(value);
  const anchor=f.renderer.committedFrame.playerAim;
  assert.deepEqual(session.pointerAim(value,anchor.footX,anchor.footY,rect),{x:0,y:0});
  session.acceptSnapshot(snapshot(2));
  assert.equal(session.pointerAim(value,anchor.footX,anchor.footY,rect),null);
  assert.equal(session.pointerAim(snapshot(2),anchor.footX,anchor.footY,rect),null);
  await session.render(snapshot(2));
  session.cancel();assert.equal(session.pointerAim(snapshot(2),10,20,rect),null);
}));

test('walking mesh and contact shadow share actual player feet and stop with authority or pause', async () => fixture(async f => {
  document.documentElement.dataset.motion='full';f.target.matchMedia=()=>({matches:false});
  const moving=(tick,x,speed=3)=>{const s=snapshot();return {...s,serverTick:tick,authorityRevision:tick,
    player:{...s.player,velocityMps:{xM:speed,yM:0,zM:0},transform:{...s.player.transform,positionM:{xM:x,yM:2,zM:8}}}};};
  await f.renderer.render(moving(1,4));
  assert.equal(f.renderer.playerLocomotionMesh,null,'one unexplained initial pose is not a gait');
  await f.renderer.render(moving(2,4.05));
  await f.renderer.render(moving(3,4.10));
  const sprite=f.renderer.sprites.get('actor:player').sprite;
  const mesh=f.renderer.playerLocomotionMesh;assert.ok(mesh);assert.equal(sprite.visible,false);
  assert.equal(mesh.parent,sprite.parent);assert.equal(mesh.x,sprite.x);assert.equal(mesh.y,sprite.y);assert.equal(mesh.zIndex,sprite.zIndex);
  assert.equal(mesh.pivot.x,sprite.texture.width*sprite.anchor.x);assert.equal(mesh.pivot.y,sprite.texture.height*sprite.anchor.y);
  assert.equal(f.renderer.playerContactShadow.y,sprite.y);assert.equal(f.renderer.playerContactShadow.zIndex,sprite.zIndex-.25);
  sprite.tint=0x9fbcd2;mesh.tint=0x9fbcd2;
  f.renderer.setActorFeedbackTint('player',0xff7766);assert.equal(mesh.tint,0xff7766);assert.equal(sprite.tint,0xff7766);
  f.renderer.setActorFeedbackTint('player',null);assert.equal(mesh.tint,0x9fbcd2);assert.equal(sprite.tint,0x9fbcd2);
  f.renderer.setActorFeedbackTint('player',0xff7766);
  f.renderer.clearTransientPresentation();await f.flush();
  assert.equal(f.renderer.playerLocomotionMesh,null);assert.equal(sprite.visible,true);assert.equal(mesh.destroyed,true);
  assert.equal(sprite.tint,0x9fbcd2);assert.equal(f.renderer.actorFeedbackTints.size,0);
  await f.renderer.render(moving(4,4.15));await f.renderer.render(moving(5,4.20));assert.ok(f.renderer.playerLocomotionMesh);
  await f.renderer.render(moving(6,4.20,0));assert.equal(f.renderer.playerLocomotionMesh,null);assert.equal(sprite.visible,true);
}));

test('cropped enemy placeholders retain the selected foot anchor and raised contact shadow', async () => fixture(async f => {
  const definition=JSON.parse(await readFile(resolve(root,'content/scenes/compiled/gh_gate_b.json'),'utf8'));
  const value={...snapshot(),worldId:'grey_hive',sceneId:'gh_gate_b',actors:[{
    entityId:'gh_gate_b_brute_01',entityType:'enemy.grey_hive.brute',actorKind:'enemy',active:true,
    transform:{positionM:{xM:10,yM:2,zM:8},yawRad:0}}]};
  f.renderer.expectSceneIdentity(value);f.renderer.setSceneDefinition(definition);await f.renderer.render(value);
  const sprite=f.renderer.sprites.get('actor:gh_gate_b_brute_01').sprite;
  assert.equal(sprite.anchor.x,(190-14)/332);assert.equal(sprite.anchor.y,537/542);
  const originalTint=sprite.tint;assert.notEqual(originalTint,0xffffff);
  f.renderer.setActorFeedbackTint('gh_gate_b_brute_01',0xffffff);
  f.renderer.setActorFeedbackTint('gh_gate_b_brute_01',0xff7766);
  f.renderer.setActorFeedbackTint('gh_gate_b_brute_01',null);assert.equal(sprite.tint,originalTint);
  f.renderer.setActorFeedbackTint('gh_gate_b_brute_01',0xff7766);
  await f.renderer.render({...value,serverTick:2});
  assert.equal(sprite.tint,originalTint);
  f.renderer.setActorFeedbackTint('gh_gate_b_brute_01',null);assert.equal(sprite.tint,originalTint,'late clear preserves fresh normal style');
  const shadow=f.renderer.clockworksEnemyBodies.get('gh_gate_b_brute_01').shadow;
  assert.equal(shadow.y,sprite.y);assert.equal(shadow.parent,sprite.parent);assert.equal(shadow.zIndex,sprite.zIndex-.25);
  const decon=JSON.parse(await readFile(resolve(root,'content/scenes/compiled/gh_deep_decon.json'),'utf8'));
  const id='gh_deep_decon_swarm_01';const center={xM:10,yM:2,zM:8};
  const swarm={entityId:id,entityType:'enemy.grey_hive.swarm',actorKind:'enemy',active:true,transform:{positionM:center,yawRad:0},
    members:[[-.55,-.55],[.55,-.55],[-.55,.55],[.55,.55]].map(([x,z],i)=>({memberId:`${id}/member/${i+1}`,
      positionM:{xM:center.xM+x,yM:2,zM:center.zM+z},radiusM:.15,active:true}))};
  const next={...value,sceneId:'gh_deep_decon',worldEpoch:2,actors:[swarm]};
  f.renderer.expectSceneIdentity(next);f.renderer.setSceneDefinition(decon);await f.renderer.render(next);
  assert.equal(f.renderer.actorFeedbackTints.size,0,'owner replacement forgets old actor tint');
  f.renderer.setActorFeedbackTint('gh_gate_b_brute_01',null);
  for(const body of f.renderer.swarmMemberBodies.values()){
    assert.equal(body.sprite.anchor.x,(266-17)/428);assert.equal(body.sprite.anchor.y,(511-3)/522);
    assert.equal(body.shadow.y,body.sprite.y);
  }
}));

test("combat contact and phase graphics use physical/ground layers and clear at lifecycle barriers", async () => fixture(async f => {
  await f.renderer.render(snapshot());
  const value = snapshot();
  value.player.actionState = "guard";
  value.player.actionPresentation = { requestId: 1, phase: "windup", elapsedMs: 17, durationMs: 880, rangeM: 0, lineHalfWidthM: 0 };
  await f.renderer.render(value);
  let guard = f.renderer.actionVfx.get("guard");
  assert.equal(guard.parent, f.renderer.layerContainers.get("L3_ACTORS"));
  assert.equal(guard.alpha, .65);
  const active = { ...value, serverTick: 2, player: { ...value.player, actionPresentation: { ...value.player.actionPresentation, phase: "active", elapsedMs: 100 } } };
  await f.renderer.render(active);
  assert.equal(f.renderer.actionVfx.get("guard").alpha, 1);
  const released = { ...active, serverTick: 3, player: { ...active.player, actionPresentation: { ...active.player.actionPresentation, phase: "recovery", elapsedMs: 117 } } };
  await f.renderer.render(released);
  assert.equal(f.renderer.actionVfx.get("guard").alpha, .38, "release cannot remain visually active");
  const pulse = { ...value, serverTick: 4, player: { ...value.player, actionState: "pulse", actionPresentation: { ...value.player.actionPresentation, requestId: 2, durationMs: 590, rangeM: 2.6 } } };
  await f.renderer.render(pulse);
  assert.equal(f.renderer.actionVfx.has("guard"), false);
  assert.equal(f.renderer.actionVfx.get("pulse").parent, f.renderer.layerContainers.get("L1_FLOOR"));
  const hurt = { protocolVersion: 2, eventId: 1, worldEpoch: 1, serverTick: 4, kind: "Damaged", positionM: { ...pulse.player.transform.positionM },
    directionRad: 0, radiusM: 1, intensity: 1, durationMs: 260,
    combatFeedback: { worldId: pulse.worldId, sceneId: pulse.sceneId, outcome: "player_hurt", targetId: "player", sourceId: "enemy_01" } };
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([hurt], pulse), [1]);
  await f.renderer.render(pulse);
  assert.equal(f.renderer.layerContainers.get("L3_ACTORS").children.filter(item => item.label === "temporary_visual:combat:player_hurt:1").length, 1);
  assert.equal(f.renderer.layerContainers.get("L7_VFX").children.some(item => item.label.startsWith("temporary_visual:combat:")), false);
  f.renderer.clearTransientPresentation(); await f.flush();
  assert.equal(f.renderer.actionVfx.size, 0);
  assert.equal(f.renderer.layerContainers.get("L3_ACTORS").children.some(item => item.label.startsWith("temporary_visual:combat:")), false);
  f.canvas.dispatchEvent(new Event("webglcontextlost", { cancelable: true }));
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([{ ...hurt, eventId: 2 }], pulse), []);
}));

test('composed contact events tint the displayed walking mesh and clear on expiry, pause and replacement', async () => fixture(async f => {
  document.documentElement.dataset.motion='full';f.target.matchMedia=()=>({matches:false});
  const moving=(tick,x,speed=3)=>{const s=snapshot();return {...s,serverTick:tick,authorityRevision:tick,
    player:{...s.player,velocityMps:{xM:speed,yM:0,zM:0},transform:{...s.player.transform,positionM:{xM:x,yM:2,zM:8}}}};};
  let s=moving(1,4);await f.renderer.render(s);s=moving(2,4.05);await f.renderer.render(s);s=moving(3,4.1);await f.renderer.render(s);
  const event=(id,view)=>({protocolVersion:2,eventId:id,worldEpoch:view.worldEpoch,serverTick:view.serverTick,kind:'Damaged',
    positionM:{...view.player.transform.positionM},directionRad:0,radiusM:1,intensity:1,durationMs:260,
    combatFeedback:{worldId:view.worldId,sceneId:view.sceneId,outcome:'player_hurt',targetId:'player',sourceId:'enemy_01'}});
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([event(1,s)],s),[1]);
  await f.renderer.render(s);
  const sprite=f.renderer.sprites.get('actor:player').sprite,mesh=f.renderer.playerLocomotionMesh;
  assert.ok(mesh);assert.equal(sprite.tint,0xffbbb5);assert.equal(mesh.tint,0xffbbb5);
  const mark=[...f.renderer.combatFeedbackLayer.marks.values()][0];
  assert.equal(mark.parent,sprite.parent);assert.equal(mark.zIndex,sprite.zIndex+.12);
  assert.equal(mark.y,sprite.y-18*(f.app.screen.width/1280),'contact uses the displayed elevated footpoint');
  s=moving(30,4.1,0);await f.renderer.render(s);
  assert.equal(sprite.tint,0xffffff);assert.equal(f.renderer.playerLocomotionMesh,null);assert.equal(f.renderer.combatFeedbackLayer.marks.size,0);
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([event(2,s)],s),[2]);await f.renderer.render(s);assert.equal(sprite.tint,0xffbbb5);
  f.renderer.clearTransientPresentation();await f.flush();assert.equal(sprite.tint,0xffffff);assert.equal(f.renderer.actorFeedbackTints.size,0);
  assert.equal(f.renderer.combatFeedbackLayer.marks.size,0);
  const next={...s,worldEpoch:2};f.renderer.expectSceneIdentity(next);f.renderer.setSceneDefinition(scene);await f.renderer.render(next);
  assert.equal(f.renderer.sprites.get('actor:player').sprite.tint,0xffffff);assert.equal(f.renderer.actorFeedbackTints.size,0);
}));

test('composed enemy contact keeps crop anchor, actual normal color and shared prop occlusion depth', async () => fixture(async f => {
  const definition=JSON.parse(await readFile(resolve(root,'content/scenes/compiled/gh_gate_b.json'),'utf8'));
  const id='gh_gate_b_brute_01',position={xM:4,yM:2,zM:8};
  const s={...snapshot(),worldId:'grey_hive',sceneId:'gh_gate_b',actors:[{entityId:id,entityType:'enemy.grey_hive.brute',actorKind:'enemy',active:true,
    transform:{positionM:position,yawRad:0}}]};
  f.renderer.expectSceneIdentity(s);f.renderer.setSceneDefinition(definition);await f.renderer.render(s);
  const sprite=f.renderer.sprites.get(`actor:${id}`).sprite,normal=sprite.tint;
  const prop=f.renderer.sprites.get('scene:gate_b_supply_crates').sprite;
  assert.notEqual(normal,0xffffff);assert.equal(sprite.parent,prop.parent);assert.ok(sprite.zIndex<prop.zIndex);
  const hit={protocolVersion:2,eventId:1,worldEpoch:s.worldEpoch,serverTick:s.serverTick,kind:'PierceHit',positionM:position,
    directionRad:0,radiusM:1,intensity:1,durationMs:220,
    combatFeedback:{worldId:s.worldId,sceneId:s.sceneId,outcome:'enemy_hit',targetId:id,sourceId:'player',requestId:11}};
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([hit],s),[1]);await f.renderer.render(s);
  assert.equal(sprite.tint,0xffe1ac);assert.equal(sprite.anchor.x,(190-14)/332);assert.equal(sprite.anchor.y,537/542);
  const mark=[...f.renderer.combatFeedbackLayer.marks.values()][0];
  assert.equal(mark.parent,prop.parent);assert.equal(mark.zIndex,sprite.zIndex+.12);assert.ok(mark.zIndex<prop.zIndex,'foreground prop covers both body and hit mark');
  await f.renderer.render({...s,serverTick:2});assert.equal(sprite.tint,0xffe1ac,'fresh style still receives the unexpired real hit');
  await f.renderer.render({...s,serverTick:30});assert.equal(sprite.tint,normal);assert.equal(f.renderer.combatFeedbackLayer.marks.size,0);
  const moved={...s,serverTick:31,actors:[{...s.actors[0],transform:{positionM:{...position,xM:15},yawRad:0}}]};
  await f.renderer.render(moved);assert.ok(sprite.zIndex>prop.zIndex,'same body crosses to foreground by world foot depth');
  assert.equal(sprite.tint,normal);assert.equal(f.renderer.clockworksEnemyBodies.get(id).shadow.y,sprite.y);
}));


const signalId = 'mh_signal_yard_signal_wraith_01';
const signalPoint = { xM: 3, yM: 0, zM: 4 };
function signalSnapshot(precise = true, tick = 100) {
  const position = precise ? signalPoint : { ...signalPoint, xM: 3.5 };
  return { ...snapshot(), worldId: 'mist_harbor', sceneId: 'mh_signal_yard', serverTick: tick,
    authorityRevision: tick, actors: [{ entityId: signalId, entityType: 'enemy.mist_harbor.signal_wraith',
      actorKind: 'enemy', active: true, transform: { positionM: position, yawRad: 0 },
      signalPerception: { precise, uncertaintyRadiusM: precise ? 0 : 1,
        positionsM: precise ? [signalPoint] : [signalPoint, { ...signalPoint, xM: 4 }] } }] };
}
const signalHit = (view, eventId = 1) => ({ protocolVersion: 2, eventId, worldEpoch: view.worldEpoch,
  serverTick: view.serverTick, kind: 'Hit', positionM: { ...signalPoint }, directionRad: 0,
  radiusM: 1, intensity: 1, durationMs: 220,
  combatFeedback: { worldId: view.worldId, sceneId: view.sceneId, outcome: 'enemy_hit',
    targetId: signalId, sourceId: 'player', requestId: eventId } });
const signalBody = renderer => renderer.swarmMemberBodies.get(`${signalId}/signal/0`).sprite;
async function signalFixture(run) {
  return fixture(async f => {
    const definition = JSON.parse(await readFile(resolve(root, 'content/scenes/compiled/mh_signal_yard.json'), 'utf8'));
    const value = signalSnapshot();
    f.renderer.expectSceneIdentity(value); f.renderer.setSceneDefinition(definition);
    await f.renderer.render(value);
    await run({ ...f, value, definition });
  });
}

test('precise Signal Wraith hit tints its sole displayed body instead of the hidden owner', async () => signalFixture(async f => {
  const owner = f.renderer.sprites.get(`actor:${signalId}`).sprite;
  const body = signalBody(f.renderer), normal = body.tint, ownerNormal = owner.tint;
  assert.equal(owner.visible, false); assert.equal(body.visible, true);
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], f.value), [1]);
  await f.renderer.render(f.value);
  assert.equal(body.tint, 0xffe1ac, 'the actual visible precise body receives hit feedback');
  assert.equal(owner.tint, ownerNormal, 'hidden resource owner is not the feedback target');
  f.renderer.setActorFeedbackTint(signalId, null); assert.equal(body.tint, normal);
}));

test('precise Signal Wraith contact mark shares the displayed body rank and stays behind physical foreground', async () => signalFixture(async f => {
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], f.value), [1]);
  await f.renderer.render(f.value);
  const body = signalBody(f.renderer), mark = [...f.renderer.combatFeedbackLayer.marks.values()][0];
  assert.ok(mark); assert.equal(mark.parent, body.parent);
  assert.equal(mark.zIndex, body.zIndex + .12, 'contact marker is in front of the visible signal body');
  assert.equal(body.zIndex, f.renderer.worldObjectDepths.get(`signal:${signalId}:0`));
  const ahead = [...f.renderer.sprites.entries()].filter(([key, row]) => key !== `actor:${signalId}` && row.sprite.visible && row.sprite.parent === body.parent && row.sprite.zIndex > body.zIndex);
  assert.ok(ahead.length > 0);
  assert.ok(ahead.every(([, row]) => row.sprite.zIndex > mark.zIndex), 'physical foreground still covers the contact mark');
}));


test('Signal Wraith precision transitions never tint or anchor either ambiguous alternative', async () => signalFixture(async f => {
  const body = signalBody(f.renderer), normal = body.tint;
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], f.value), [1]);
  await f.renderer.render(f.value); assert.equal(body.tint, 0xffe1ac);
  const hidden = signalSnapshot(false, 101);
  await f.renderer.render(hidden);
  assert.equal(signalBody(f.renderer), body, 'alternative zero is reused across the transition');
  assert.equal(f.renderer.swarmMemberBodies.size, 2);
  assert.equal(f.renderer.combatFeedbackLayer.marks.size, 0);
  assert.equal(f.renderer.actorFeedbackTints.size, 0);
  const alternatives = [...f.renderer.swarmMemberBodies.values()].map(row => row.sprite);
  f.renderer.setActorFeedbackTint(signalId, 0xffe1ac);
  for (const sprite of alternatives) { assert.equal(sprite.tint, normal); assert.equal(sprite.alpha, .74); }
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(hidden, 999)], hidden), []);
  assert.equal(f.renderer.signalFeedbackTargets.get(signalId), null, 'no ambiguous target anchor is bound');
  const precise = signalSnapshot(true, 102);
  await f.renderer.render(precise);
  assert.equal(alternatives[1].destroyed, true); assert.equal(body.tint, normal); assert.equal(body.alpha, 1);
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], precise), [], 'pruned event cannot restart on reappearance');
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(precise, 2)], precise), [2], 'rejected ambiguity did not consume cursor');
  await f.renderer.render(precise); assert.equal(body.tint, 0xffe1ac);
  assert.equal([...f.renderer.combatFeedbackLayer.marks.values()][0].zIndex, body.zIndex + .12);
}));

test('Signal Wraith duplicate events cannot restart wall-clock or server-tick feedback TTL', async () => signalFixture(async f => {
  await f.renderer.render(f.value);
  const body = signalBody(f.renderer), normal = body.tint, event = signalHit(f.value);
  assert.deepEqual(f.renderer.combatFeedbackModel.accept([event], f.value, 1000), [1]);
  await f.renderer.renderFrame(f.value, undefined, undefined, 1000);
  const mark = [...f.renderer.combatFeedbackLayer.marks.values()][0];
  assert.equal(body.tint, 0xffe1ac);
  assert.deepEqual(f.renderer.combatFeedbackModel.accept([event, event], f.value, 1190), []);
  await f.renderer.renderFrame(f.value, undefined, undefined, 1219);
  assert.equal(f.renderer.combatFeedbackLayer.marks.size, 1); assert.equal(body.tint, 0xffe1ac);
  await f.renderer.renderFrame(f.value, undefined, undefined, 1220);
  assert.equal(mark.destroyed, true); assert.equal(body.tint, normal); assert.equal(f.renderer.actorFeedbackTints.size, 0);
  assert.deepEqual(f.renderer.combatFeedbackModel.accept([event], f.value, 1221), []);
  const next = signalSnapshot(true, 101);
  assert.deepEqual(f.renderer.combatFeedbackModel.accept([signalHit(next, 2)], next, 1300), [2]);
  await f.renderer.renderFrame(next, undefined, undefined, 1300); assert.equal(body.tint, 0xffe1ac);
  await f.renderer.renderFrame(signalSnapshot(true, 115), undefined, undefined, 1301);
  assert.equal(f.renderer.combatFeedbackLayer.marks.size, 0); assert.equal(body.tint, normal);
}));

test('lethal precise Signal Wraith contact survives body removal without tinting a stale or replacement sprite', async () => signalFixture(async f => {
  const body = signalBody(f.renderer), normal = body.tint;
  assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], f.value), [1]);
  await f.renderer.render(f.value); assert.equal(body.tint, 0xffe1ac);
  const dead = signalSnapshot(true, 101); dead.actors[0].active = false;
  await f.renderer.render(dead);
  assert.equal(body.destroyed, true); assert.equal(body.tint, normal, 'old body restores color before destruction');
  assert.equal(f.renderer.swarmMemberBodies.size, 0); assert.equal(f.renderer.signalFeedbackTargets.size, 0);
  assert.equal(f.renderer.actorFeedbackTints.size, 0); assert.equal(f.renderer.combatFeedbackLayer.marks.size, 1, 'precise lethal event retains its public contact');
  const mark = [...f.renderer.combatFeedbackLayer.marks.values()][0];
  const front = f.renderer.sprites.get('actor:player').sprite;
  assert.ok(mark.zIndex < front.zIndex, 'lethal fallback retains physical foreground occlusion');
  await f.renderer.render({ ...dead, serverTick: 102, actors: [] });
  assert.equal(mark.destroyed, true); assert.equal(f.renderer.combatFeedbackLayer.marks.size, 0);
  await f.renderer.render(signalSnapshot(true, 103));
  const replacement = signalBody(f.renderer); assert.notEqual(replacement, body); assert.equal(replacement.tint, normal);
  f.renderer.setActorFeedbackTint(signalId, null); assert.equal(replacement.tint, normal, 'late clear cannot recolor replacement');
}));

for (const boundary of ['pause', 'player death', 'scene', 'epoch', 'body cleanup', 'context loss', 'invalidate', 'destroy']) {
  test(`Signal Wraith feedback clears safely across ${boundary}`, async () => signalFixture(async f => {
    const body = signalBody(f.renderer), normal = body.tint;
    assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], f.value), [1]);
    await f.renderer.render(f.value); assert.equal(body.tint, 0xffe1ac);
    const mark = [...f.renderer.combatFeedbackLayer.marks.values()][0];
    if (boundary === 'pause') {
      let release; f.renderer.renderQueue = new Promise(resolve => { release = resolve; });
      const pending = f.renderer.render({ ...f.value, serverTick: 101 });
      f.renderer.clearTransientPresentation(); release(); await pending; await f.flush();
      assert.equal(f.renderer.transientsCleared, true);
      Object.assign(f.canvas.parentElement, { clientWidth: 1000 }); f.target.dispatchEvent(new Event('resize')); await f.flush();
      assert.equal(body.tint, normal); assert.equal(f.renderer.combatFeedbackLayer.marks.size, 0);
      await f.renderer.render({ ...f.value, serverTick: 102 });
      assert.equal(f.renderer.combatFeedbackLayer.marks.size, 0, 'resume does not replay old event');
    } else if (boundary === 'player death') {
      await f.renderer.render({ ...f.value, player: { ...f.value.player, currentHp: 0 } });
    } else if (boundary === 'scene' || boundary === 'epoch') {
      const next = boundary === 'scene' ? snapshot(2) : { ...f.value, worldEpoch: 2 };
      f.renderer.expectSceneIdentity(next); await f.renderer.sceneInvalidationCleanup;
      f.renderer.setSceneDefinition(boundary === 'scene' ? scene : f.definition); await f.renderer.render(next);
      assert.equal(body.destroyed, true);
      if (boundary === 'epoch') {
        const replacement = signalBody(f.renderer); assert.notEqual(replacement, body);
        assert.equal(replacement.tint, normal); f.renderer.setActorFeedbackTint(signalId, null); assert.equal(replacement.tint, normal);
      }
    } else if (boundary === 'body cleanup') {
      f.renderer.clearClockworksEnemyBodies();
      assert.equal(body.destroyed, true); assert.equal(f.renderer.signalFeedbackTargets.size, 0);
      f.renderer.clearTransientPresentation(); await f.flush(); await f.renderer.render(f.value);
      const replacement = signalBody(f.renderer); assert.notEqual(replacement, body); assert.equal(replacement.tint, normal);
    } else if (boundary === 'context loss') f.canvas.dispatchEvent(new Event('webglcontextlost', { cancelable: true }));
    else if (boundary === 'invalidate') f.renderer.invalidate();
    else await f.renderer.destroy();
    assert.equal(body.tint, normal); assert.equal(mark.destroyed, true);
    assert.equal(f.renderer.actorFeedbackTints.size, 0); assert.equal(f.renderer.combatFeedbackLayer.marks.size, 0);
  }));
}

for (const [width, height, motion] of [[882, 552, 'reduced'], [1458, 829, 'full'], [2034, 1107, 'reduced']]) {
  test(`precise Signal Wraith feedback follows public body at ${width}x${height} with ${motion} motion`, async () => signalFixture(async f => {
    document.documentElement.dataset.motion = motion; f.target.matchMedia = () => ({ matches: false });
    Object.assign(f.canvas.parentElement, { clientWidth: width, clientHeight: height });
    f.target.dispatchEvent(new Event('resize')); await f.flush();
    const hiddenInterpolation = new Map([[signalId, { xM: 999, yM: 99, zM: 999 }]]);
    assert.deepEqual(f.renderer.acceptCombatPresentationEvents([signalHit(f.value)], f.value), [1]);
    await f.renderer.render(f.value, undefined, hiddenInterpolation);
    const body = signalBody(f.renderer), mark = [...f.renderer.combatFeedbackLayer.marks.values()][0];
    assert.equal(body.tint, 0xffe1ac); assert.equal(mark.zIndex, body.zIndex + .12);
    assert.equal(f.renderer.signalFeedbackTargets.get(signalId).sprite, body);
    assert.equal(body.x, mark.x, 'precise public body and contact ignore private/interpolated owner position');
    assert.equal(body.visible, true); assert.equal(f.renderer.sprites.get(`actor:${signalId}`).sprite.visible, false);
  }));
}
