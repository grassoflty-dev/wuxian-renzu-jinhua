import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { Assets, Container, Texture, TextureSource } from "pixi.js";
import { AssetRegistry } from "../dist/assets/AssetRegistry.js";
import { assertSnapshotV3 } from "../dist/protocol/types.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { BAIZHI_ENTITY_ID, projectBaizhiPlaceholder } from "../dist/renderer/BaizhiPresentation.js";
import { actorAssetId, buildRendererResourcePlan } from "../dist/renderer/RendererResourcePlan.js";

const root = new URL("../../../", import.meta.url), registry = new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(new URL("governance/assets/RUNTIME_ASSET_MANIFEST.json", root))),
  new Uint8Array(await readFile(new URL("governance/assets/AI_ASSET_RELEASE_MANIFEST.json", root))),
  async path => new Uint8Array(await readFile(new URL(path, root))));
const bio = JSON.parse(await readFile(new URL("content/scenes/compiled/gh_bio_isolation.json", root), "utf8"));
const hub = JSON.parse(await readFile(new URL("content/scenes/compiled/rs_core_room.json", root), "utf8"));
function snapshot(overrides = {}) {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "gh_bio_isolation", checkpointId: null, worldEpoch: 3,
    serverTick: 1, authorityRevision: 1, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 8, yM: 0, zM: 9.3 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 80, maxHp: 100,
      currentEnergy: 60, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
    baizhi: { schemaVersion: 1, choice: "unresolved", available: true, canInteract: true },
    npcs: [{ entityId: BAIZHI_ENTITY_ID, entityType: "npc.baizhi", position: [8, 0, 10.5],
      yawRad: Math.PI, interactable: true }], ...overrides };
}

// Real WorldRenderer/Pixi object and registry paths; the surface/atlas upload is deliberately CPU-only.
// These are not GPU, browser screenshot, Windows execution, collision, or dialogue E2E tests.
async function fixture(run) {
  const globals = Object.fromEntries(["window", "document", "ResizeObserver", "requestAnimationFrame", "cancelAnimationFrame"].map(k => [k, globalThis[k]]));
  const unload = Assets.unload, pending = new Map(); let frameId = 0, draws = 0;
  const target = new EventTarget(); target.matchMedia = () => ({ matches: true });
  globalThis.window = target;
  globalThis.document = { documentElement: { dataset: { motion: "reduced" } } };
  globalThis.requestAnimationFrame = callback => { pending.set(++frameId, callback); return frameId; };
  globalThis.cancelAnimationFrame = id => pending.delete(id);
  globalThis.ResizeObserver = class { observe() {} disconnect() {} };
  Assets.unload = async () => {};
  const canvas = new EventTarget(); canvas.parentElement = { clientWidth: 1280, clientHeight: 720 };
  const app = { stage: new Container(), screen: { width: 1280, height: 720 },
    renderer: { background: {}, resize(width, height) { app.screen = { width, height }; } },
    async init() {}, render() { draws++; }, destroy() {} };
  const renderer = new WorldRenderer(registry); renderer.app = app;
  const pageRequests = [];
  renderer.preloadAtlasPages = async pages => {
    pageRequests.push(pages.map(p => p.atlasUrl));
    for (const page of pages) if (!renderer.atlasTextures.has(page.atlasUrl)) {
      renderer.atlasTextures.set(page.atlasUrl, new Texture({ source: new TextureSource({ width: 8192, height: 8192 }) }));
    }
  };
  try {
    await renderer.init(canvas);
    renderer.expectSceneIdentity(snapshot()); renderer.setSceneDefinition(bio);
    await run({ renderer, canvas, app, pending, pageRequests, draws: () => draws });
  } finally {
    if (renderer.ready) await renderer.destroy();
    Assets.unload = unload;
    for (const [key, value] of Object.entries(globals)) {
      if (value === undefined) delete globalThis[key]; else globalThis[key] = value;
    }
  }
}

test("real renderer keeps exact development NPC for three results without new assets or actor resources", async () => fixture(async f => {
  const baseline = snapshot({ baizhi: undefined, npcs: undefined });
  await f.renderer.render(baseline);
  const expectedPages = f.pageRequests.at(-1);
  assert.equal(f.renderer.baizhiPlaceholder, null);
  let object;
  for (const choice of ["unresolved", "taken", "left"]) for (const canInteract of [true, false]) {
    const value = snapshot(); Object.assign(value.baizhi, { choice, canInteract });
    value.npcs[0].interactable = canInteract;
    const before = structuredClone(value);
    await f.renderer.render(value);
    const current = f.renderer.baizhiPlaceholder;
    assert.ok(current); assert.equal(current.nameLabel.text, "白芷");
    assert.equal(current.parent, f.renderer.layerContainers.get("L3_ACTORS"));
    assert.deepEqual(f.pageRequests.at(-1), expectedPages, "placeholder never loads a texture or uses an unapproved concept");
    assert.equal(f.renderer.sprites.has(`actor:${BAIZHI_ENTITY_ID}`), false);
    assert.equal(f.renderer.sprites.has(`npc:${BAIZHI_ENTITY_ID}`), false);
    assert.equal(f.renderer.clockworksEnemyBodies.has(BAIZHI_ENTITY_ID), false);
    assert.equal(f.renderer.worldUi.has(`hp:${BAIZHI_ENTITY_ID}`), false);
    assert.equal(f.renderer.isReadyFor(value), true);
    assert.deepEqual(value, before, "presentation does not modify HP, actor arrays, progression, or interaction");
    if (object) assert.equal(current, object, "duplicate and terminal snapshots reuse one object");
    object = current;
  }
}));

test("bad optional wire never crashes real scene rendering and cleans an already-visible NPC", async () => fixture(async f => {
  const invalid = [{ baizhi: undefined }, { npcs: undefined }, { baizhi: null }, { npcs: [] },
    { npcs: [...snapshot().npcs, ...snapshot().npcs] },
    { npcs: [{ ...snapshot().npcs[0], entityType: "npc.unapproved" }] },
    { npcs: [{ ...snapshot().npcs[0], position: [NaN, 0, 10.5] }] },
    { npcs: [{ ...snapshot().npcs[0], yawRad: Infinity }] },
    { npcs: [{ ...snapshot().npcs[0], interactable: false }] },
    { baizhi: { ...snapshot().baizhi, choice: "unknown" } },
    { baizhi: { ...snapshot().baizhi, available: "true" } },
    { baizhi: { ...snapshot().baizhi, canInteract: 1 } }];
  for (const change of invalid) {
    await f.renderer.render(snapshot()); const previous = f.renderer.baizhiPlaceholder;
    const value = snapshot(change);
    assert.equal(assertSnapshotV3(value), value, "optional corruption stays separate from core snapshot acceptance");
    await f.renderer.render(value);
    assert.ok(previous.destroyed);
    assert.equal(f.renderer.baizhiPlaceholder, null);
    assert.ok(f.renderer.sprites.has("actor:player"));
    assert.equal(f.renderer.isReadyFor(value), true);
    assert.equal(f.renderer.worldObjectDepths.has(`npc:${BAIZHI_ENTITY_ID}`), false);
  }
}));

test("actual pipeline depth ranks put NPC behind closer actors/props and ahead of farther objects", async () => fixture(async f => {
  const front = snapshot(); await f.renderer.render(front);
  const key = `npc:${BAIZHI_ENTITY_ID}`, layer = f.renderer.layerContainers.get("L3_ACTORS");
  const npc = f.renderer.baizhiPlaceholder, player = f.renderer.sprites.get("actor:player").sprite;
  assert.equal(npc.zIndex, f.renderer.worldObjectDepths.get(key));
  assert.ok(player.zIndex < npc.zIndex);
  const pod = f.renderer.sprites.get("scene:bio_pod_approved").sprite;
  const crates = f.renderer.sprites.get("scene:bio_supply_crates").sprite;
  assert.equal(pod.parent, layer); assert.equal(crates.parent, layer);
  assert.ok(pod.zIndex < npc.zIndex && npc.zIndex < crates.zIndex);
  const behind = snapshot(); behind.player.transform.positionM.zM = 12;
  await f.renderer.render(behind);
  assert.equal(f.renderer.baizhiPlaceholder, npc);
  assert.ok(player.zIndex > npc.zIndex, "player can cross in front; NPC is not an always-front overlay");
  layer.sortChildren();
  assert.ok(layer.children.indexOf(pod) < layer.children.indexOf(npc));
  assert.ok(layer.children.indexOf(npc) < layer.children.indexOf(player));
  assert.ok(layer.children.indexOf(npc) < layer.children.indexOf(crates));
}));

test("pause/transient clearing and far range preserve visible NPC without injecting input or combat", async () => fixture(async f => {
  await f.renderer.render(snapshot()); const npc = f.renderer.baizhiPlaceholder;
  f.renderer.clearTransientPresentation();
  assert.equal(f.renderer.baizhiPlaceholder, npc); assert.equal(npc.destroyed, false);
  const far = snapshot(); far.baizhi.canInteract = false; far.npcs[0].interactable = false;
  far.player.transform.positionM = { xM: 2, yM: 0, zM: 8 };
  await f.renderer.render(far);
  assert.equal(f.renderer.baizhiPlaceholder, npc);
  assert.equal(npc.eventMode, "none"); assert.equal(npc.interactiveChildren, false);
  const camera = { width: 1280, height: 720, origin: far.player.transform.positionM, pixelsPerMeter: 48 };
  const frame = projectBaizhiPlaceholder(far, camera);
  assert.deepEqual(frame.position, { xM: 8, yM: 0, zM: 10.5 });
}));

test("epoch changes destroy/recreate once and hub remains NPC-free even if optional data leaks", async () => fixture(async f => {
  await f.renderer.render(snapshot()); let previous = f.renderer.baizhiPlaceholder;
  for (const worldEpoch of [4, 5, 6]) {
    const value = snapshot({ worldEpoch });
    f.renderer.expectSceneIdentity(value);
    assert.equal(f.renderer.baizhiPlaceholder, null); assert.ok(previous.destroyed);
    await f.renderer.sceneInvalidationCleanup; f.renderer.setSceneDefinition(bio);
    await f.renderer.render(value); const next = f.renderer.baizhiPlaceholder;
    assert.ok(next && next !== previous);
    assert.equal(f.renderer.layerContainers.get("L3_ACTORS").children.filter(child => child.label === "DEV_WHITEZHI_PLACEHOLDER").length, 1);
    previous = next;
  }
  const home = snapshot({ worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 7 });
  f.renderer.expectSceneIdentity(home); await f.renderer.sceneInvalidationCleanup;
  f.renderer.setSceneDefinition(hub); await f.renderer.render(home);
  assert.ok(previous.destroyed); assert.equal(f.renderer.baizhiPlaceholder, null);
  assert.equal(f.renderer.isReadyFor(home), true);
}));

test("invalidation, context loss, and renderer destroy release private body/name objects", async () => {
  for (const kind of ["invalidate", "context-loss", "destroy"]) await fixture(async f => {
    await f.renderer.render(snapshot()); const npc = f.renderer.baizhiPlaceholder;
    const body = npc.body, label = npc.nameLabel;
    if (kind === "invalidate") f.renderer.invalidate();
    else if (kind === "context-loss") f.canvas.dispatchEvent(new Event("webglcontextlost", { cancelable: true }));
    else await f.renderer.destroy();
    assert.ok(npc.destroyed && body.destroyed && label.destroyed, kind);
    assert.equal(f.renderer.baizhiPlaceholder, null);
    assert.equal(f.renderer.isReadyFor(snapshot()), false);
  });
});

test("ordinary unknown actor/NPC and concept-only names remain behind the unchanged approved-asset gate", () => {
  for (const entityType of ["npc.unknown", "npc.baizhi", "DEV_WHITEZHI_PLACEHOLDER", "runtime2d.npc.baizhi.v1"]) {
    const actor = { entityId: BAIZHI_ENTITY_ID, entityType, actorKind: "npc", active: true };
    assert.equal(actorAssetId(actor), entityType, "no generalized NPC alias was admitted");
    assert.throws(() => registry.resolveAsset(entityType), /E_ASSET_NOT_REGISTERED|E_ASSET_NOT_APPROVED/);
    assert.throws(() => buildRendererResourcePlan(registry, "grey_hive", [actor], []), /E_ASSET_NOT_REGISTERED|E_ASSET_NOT_APPROVED/);
  }
});
