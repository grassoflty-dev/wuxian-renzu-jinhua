import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { Assets, Container, Texture, TextureSource } from "pixi.js";
import { AssetRegistry } from "../dist/assets/AssetRegistry.js";
import { assertSnapshotV3 } from "../dist/protocol/types.js";
import { dispatchInteractable, interactionErrorText } from "../dist/game/SceneInteraction.js";
import { deriveHudState } from "../dist/ui/Hud.js";
import { greyHiveBeaconStatus } from "../dist/game/GreyHiveBeacon.js";
import { confirmedGreyHiveNarrative } from "../dist/game/GreyHiveNarrative.js";
import { SceneDefinitionSession } from "../dist/game/SceneDefinitionSession.js";
import { WorldRenderer, RenderCommitError } from "../dist/renderer/WorldRenderer.js";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const registry = new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(resolve(root, "governance/assets/RUNTIME_ASSET_MANIFEST.json"))),
  new Uint8Array(await readFile(resolve(root, "governance/assets/AI_ASSET_RELEASE_MANIFEST.json"))),
  async path => new Uint8Array(await readFile(resolve(root, path))));
const scene = JSON.parse(await readFile(resolve(root, "content/scenes/compiled/gh_beacon.json"), "utf8"));
function snapshot(epoch = 1) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "gh_beacon", checkpointId: null, worldEpoch: epoch,
    serverTick: 1, authorityRevision: 1, ackSeq: 0,
    greyHiveBeacon: {state:"uncollected",legacyCompletedWithoutReceipt:false},
    player: { entityId: "player", transform: { positionM: { xM: 4, yM: 0, zM: 8 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 80, maxHp: 100,
      currentEnergy: 60, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
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


test("Beacon projection rejects malformed ownership before snapshot admission", () => {
  for (const state of ["uncollected", "carried", "mounted"]) assert.equal(assertSnapshotV3({...snapshot(),greyHiveBeacon:{state,legacyCompletedWithoutReceipt:false}}).greyHiveBeacon.state,state);
  assert.doesNotThrow(()=>assertSnapshotV3({...snapshot(),greyHiveBeacon:{state:"uncollected",legacyCompletedWithoutReceipt:true}}));
  const old=snapshot();delete old.greyHiveBeacon;assert.doesNotThrow(()=>assertSnapshotV3(old));
  for(const bad of [null,{},[],{state:"carried"},{state:"unknown",legacyCompletedWithoutReceipt:false},{state:"mounted",legacyCompletedWithoutReceipt:true},{state:"carried",legacyCompletedWithoutReceipt:false,extra:1}]) assert.throws(()=>assertSnapshotV3({...snapshot(),greyHiveBeacon:bad}),/E_GH_BEACON_PROJECTION/);
  assert.throws(()=>assertSnapshotV3({...snapshot(),worldId:"mist_harbor"}),/E_GH_BEACON_PROJECTION/);
});

test("Beacon F commands preserve exact source ID, kind and epoch", async () => {
  const calls=[]; const client={interact:async(id,epoch)=>{calls.push([id,epoch]);return {};}};
  for(const [kind,entityId] of [["beacon_collect","gh_beacon_deploy_marker"],["beacon_mount","gh_beacon_storage_mount_marker"]]) {
    const item={kind,entityId,active:true,transform:{positionM:{xM:4,yM:0,zM:8},yawRad:0}};
    const view={...snapshot(),interactables:[item]};
    assert.equal(deriveHudState(view).interactionId,entityId);
    assert.match(deriveHudState(view).interactionText,kind==="beacon_collect"?/收取便携信标/:/部署并挂载信标/);
    await dispatchInteractable(client,item,17,view);
    for(const source of [undefined,{worldId:"mist_harbor",sceneId:"gh_beacon"},{worldId:"grey_hive",sceneId:"gh_exit"}]) await assert.rejects(dispatchInteractable(client,item,17,source),/SOURCE_MISMATCH/);
    await assert.rejects(dispatchInteractable(client,{...item,entityId:"forged"},17,view),/SOURCE_MISMATCH/);
    await assert.rejects(dispatchInteractable(client,{...item,active:false},17,view),/INACTIVE/);
    assert.equal(deriveHudState({...view,player:{...view.player,currentHp:0}}).interactionId,null);
  }
  assert.deepEqual(calls,[["gh_beacon_deploy_marker",17],["gh_beacon_storage_mount_marker",17]]);
});

test("Beacon status distinguishes carried, mounted and historical completion", () => {
  assert.match(greyHiveBeaconStatus({...snapshot(),sceneId:"gh_exit"}),/返回信标室/);
  assert.match(deriveHudState({...snapshot(),greyHiveBeacon:{state:"carried",legacyCompletedWithoutReceipt:false}}).environmentText,/已携带/);
  assert.match(greyHiveBeaconStatus({...snapshot(),greyHiveBeacon:{state:"mounted",legacyCompletedWithoutReceipt:false}}),/已部署/);
  assert.match(greyHiveBeaconStatus({...snapshot(),greyHiveBeacon:{state:"uncollected",legacyCompletedWithoutReceipt:true}}),/尚未重新确认/);
  assert.match(interactionErrorText("E_BEACON_REQUIRED"),/返回信标室/);
  const old=snapshot();delete old.greyHiveBeacon;assert.equal(greyHiveBeaconStatus(old),"");
});

test("only a newly accepted collect speaks the exact authored Beacon line",()=>{
  const args=["grey_hive","gh_beacon","gh_beacon_deploy_marker","beacon_collect"];
  assert.equal(confirmedGreyHiveNarrative(...args,true),"这就是信号源。带回去，也许能解释这里为什么还在呼叫。");
  assert.equal(confirmedGreyHiveNarrative(...args,false),null);
  assert.equal(confirmedGreyHiveNarrative("grey_hive","gh_beacon","gh_beacon_storage_mount_marker","beacon_mount",true),null);
});

test("real Pixi sprites follow collection, mounting and authoritative Continue",async()=>fixture(async f=>{
  await f.renderer.render(snapshot());assert.ok(f.renderer.sprites.has("scene:beacon_folded_approved"));assert.equal(f.renderer.sprites.has("scene:beacon_mounted_approved"),false);
  const carried={...snapshot(),serverTick:2,authorityRevision:2,greyHiveBeacon:{state:"carried",legacyCompletedWithoutReceipt:false}};
  await f.renderer.render(carried);assert.equal(f.renderer.sprites.has("scene:beacon_folded_approved"),false);assert.equal(f.renderer.sprites.has("scene:beacon_mounted_approved"),false);
  const mounted={...carried,serverTick:3,authorityRevision:3,greyHiveBeacon:{state:"mounted",legacyCompletedWithoutReceipt:false}};
  await f.renderer.render(mounted);assert.equal(f.renderer.sprites.get("scene:beacon_mounted_approved").assetId,"runtime2d.prop.beacon.deployed.v1");assert.equal(f.renderer.sprites.has("scene:beacon_folded_approved"),false);
  const continued={...mounted,worldEpoch:2};f.renderer.expectSceneIdentity(continued);f.renderer.setSceneDefinition(scene);await f.renderer.render(continued);assert.ok(f.renderer.sprites.has("scene:beacon_mounted_approved"));
  const unknown={...continued,worldEpoch:3};delete unknown.greyHiveBeacon;f.renderer.expectSceneIdentity(unknown);f.renderer.setSceneDefinition(scene);await f.renderer.render(unknown);assert.equal(f.renderer.sprites.has("scene:beacon_folded_approved"),false);assert.equal(f.renderer.sprites.has("scene:beacon_mounted_approved"),false);
}));

test("exit has a real recovery route and no unconditional deployed prop",async()=>{
  const exit=JSON.parse(await readFile(resolve(root,"content/scenes/compiled/gh_exit.json"),"utf8"));
  assert.deepEqual(exit.transitions.map(t=>[t.id,t.toSceneId,t.spawnId]),[["gh_exit_to_beacon","gh_beacon","gh_beacon_spawn"]]);
  assert.equal(exit.presentation.sprites.some(s=>s.assetId==="runtime2d.prop.beacon.deployed.v1"),false);
  assert.deepEqual(scene.interactions.map(i=>[i.kind,i.event]),[["beacon_collect",null],["beacon_mount",null]]);
});
