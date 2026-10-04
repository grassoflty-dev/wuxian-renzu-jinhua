import test from "node:test";
import assert from "node:assert/strict";
import { projectWorldPoint } from "../dist/renderer/CameraModel.js";
import { RENDER_LAYERS, SceneEpochTracker, sortByFootDepth } from "../dist/renderer/LayerModel.js";
import { normalizeRenderSnapshot, screenFacingDirection } from "../dist/renderer/RenderSnapshot.js";
import { buildRendererResourcePlan, selectSpriteFrame } from "../dist/renderer/RendererResourcePlan.js";

test("fixed L0-L8 order includes world UI above VFX while HTML Core UI remains separate", () => {
  assert.deepEqual(RENDER_LAYERS, [
    "L0_BACKGROUND", "L1_FLOOR", "L2_BACK_PROPS", "L3_ACTORS",
    "L4_DYNAMIC_PROPS", "L5_FRONT_PROPS", "L6_OCCLUDERS", "L7_VFX", "L8_WORLD_UI",
  ]);
  assert.equal(RENDER_LAYERS.at(-1), "L8_WORLD_UI");
});

test("camera uses the fixed isometric projection and reports ground foot depth", () => {
  const camera = { width: 800, height: 600, origin: { xM: 10, yM: 1, zM: 20 } };
  assert.deepEqual(projectWorldPoint(camera.origin, camera), { x: 400, y: 300, footY: 300 });
  assert.deepEqual(projectWorldPoint({ xM: 11, yM: 1, zM: 20 }, camera), { x: 448, y: 324, footY: 324 });
  assert.deepEqual(projectWorldPoint({ xM: 11, yM: 2, zM: 20 }, camera), { x: 448, y: 276, footY: 324 });
});

test("same-depth actor ordering uses a stable key tie-break independent of input order", () => {
  const input = [{ stableKey: "actor:z", footY: 100 }, { stableKey: "actor:a", footY: 100 }, { stableKey: "actor:m", footY: 99 }];
  assert.deepEqual(sortByFootDepth(input).map(item => item.stableKey), ["actor:m", "actor:a", "actor:z"]);
  assert.deepEqual(sortByFootDepth([...input].reverse()).map(item => item.stableKey), ["actor:m", "actor:a", "actor:z"]);
});

test("world, scene, and epoch transitions trigger one explicit lifecycle reset", () => {
  const tracker = new SceneEpochTracker();
  assert.equal(tracker.enter({ worldId: "grey_hive", sceneId: "legacy-v2", worldEpoch: 1 }), true);
  assert.equal(tracker.enter({ worldId: "grey_hive", sceneId: "legacy-v2", worldEpoch: 1 }), false);
  assert.equal(tracker.enter({ worldId: "grey_hive", sceneId: "gh_power", worldEpoch: 1 }), true);
  assert.equal(tracker.enter({ worldId: "grey_hive", sceneId: "gh_power", worldEpoch: 2 }), true);
  assert.equal(tracker.enter({ worldId: "mist_harbor", sceneId: "mh_docks", worldEpoch: 1 }), true);
});

test("v2 nested-view snapshots and v3 flattened snapshots normalize to one render shape", () => {
  const positionM = { xM: 1, yM: 0, zM: 2 };
  const v2 = {
    kind: "full", protocolVersion: 2, schemaVersion: "freeze-v02-interfaces/1.2", worldId: "grey_hive",
    worldEpoch: 4, serverTick: 7, authorityRevision: 7, ackSeq: 1,
    view: {
      protocol: "continuous-ipc", version: 1, schemaVersion: "freeze-v02-interfaces/1.2", worldId: "grey_hive",
      worldEpoch: 4, serverTick: 7, authorityRevision: 7, ackSeq: 1, serverTimeMs: 112,
      player: { entityId: "player", transform: { positionM, yawRad: 0 }, velocityMps: positionM, currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100 },
      actors: [], doors: [],
    },
  };
  const v3 = {
    kind: "full", protocolVersion: 3, schemaVersion: "scene-v3/1", worldId: "grey_hive", sceneId: "gh_power",
    checkpointId: null, worldEpoch: 5, serverTick: 8, authorityRevision: 8, ackSeq: 2,
    player: { entityId: "player", transform: { positionM, yawRad: 0 }, velocityMps: positionM, currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100,
      facingX: 1, facingZ: 0, aimX: 1, aimZ: 0, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1 }, progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
  };
  assert.deepEqual(normalizeRenderSnapshot(v2), {
    worldId: "grey_hive", sceneId: "legacy-v2", worldEpoch: 4, serverTick: 7,
    playerEntityId: "player", playerPosition: positionM, playerVelocity: positionM, playerYawRad: 0, playerFacingX: 0, playerFacingZ: 1,
    playerAimX: 0, playerAimZ: 0,
    playerFacingSource: "v2-yaw-migration", playerActionState: "idle", actors: [], doors: [],
  });
  assert.equal(normalizeRenderSnapshot(v3).sceneId, "gh_power");
  assert.equal(normalizeRenderSnapshot(v3).worldEpoch, 5);
  assert.equal(normalizeRenderSnapshot(v3).playerYawRad, 0);
  assert.deepEqual([normalizeRenderSnapshot(v3).playerFacingX, normalizeRenderSnapshot(v3).playerFacingZ], [1, 0]);
  assert.equal(normalizeRenderSnapshot(v3).playerFacingSource, "v3-authoritative");
  assert.deepEqual([normalizeRenderSnapshot(v3).playerAimX, normalizeRenderSnapshot(v3).playerAimZ], [1, 0]);
  assert.equal(normalizeRenderSnapshot(v3).playerActionState, "idle");
  assert.equal(screenFacingDirection(normalizeRenderSnapshot(v2).playerFacingX, normalizeRenderSnapshot(v2).playerFacingZ), "south_west");
});

test("fixed isometric projection quantizes v3 facing to all eight screen directions and rejects zero vectors", () => {
  const fixtures = [
    [[1, -1], "east"], [[1, 0], "south_east"], [[1, 1], "south"], [[0, 1], "south_west"],
    [[-1, 1], "west"], [[-1, 0], "north_west"], [[-1, -1], "north"], [[0, -1], "north_east"],
  ];
  for (const [[facingX, facingZ], expected] of fixtures) {
    assert.equal(screenFacingDirection(facingX, facingZ), expected);
  }
  assert.throws(() => screenFacingDirection(0, 0), /E_RENDERER_FACING_VECTOR/);
  assert.throws(() => screenFacingDirection(Number.NaN, 1), /E_RENDERER_FACING_VECTOR/);
});

test("resource planning resolves only approved typed assets and deduplicates atlas page preloads", () => {
  const requested = [];
  const registry = {
    resolveAsset(assetId) {
      requested.push(assetId);
      if (assetId === "enemy.unknown") throw new Error(`E_ASSET_NOT_APPROVED:${assetId}`);
      const category = assetId.startsWith("runtime2d.actor.") || assetId.startsWith("runtime2d.enemy.") ? "actor"
        : assetId.startsWith("runtime2d.prop.") ? "prop"
          : assetId.startsWith("runtime2d.vfx.") ? "vfx" : "world";
      const streamingGroup = category === "actor" ? (assetId.includes("sentinel") ? "sentinel" : "shared")
        : category === "prop" ? "grey_hive" : category === "vfx" ? "shared" : "grey_hive";
      const atlasPage = category === "actor" && streamingGroup === "sentinel" ? 4 : category === "actor" ? 3 : category === "vfx" ? 8 : category === "world" ? 10 : 5;
      const atlasUrl = `assets/derived/${category}-${streamingGroup}-${atlasPage}.webp`;
      const asset = { assetId, category, streamingGroup, atlasPage, atlasUrl, atlasSha256: "a".repeat(64), textureUrl: "must-not-load-single-sprite.webp", anchorX: 0.5, anchorY: 1, atlasFrame: [1, 1, 8, 8] };
      if (assetId === "runtime2d.actor.cenyao.base.v1") asset.animation = { frames: ["south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east"].map((direction, index) => ({
        direction, state: "idle", atlasPage: 13, atlasUrl: "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp",
        atlasSha256: "b".repeat(64), atlasFrame: [index * 340 + 2, 2, 336, 560], anchor: [0.5, 0.98392857],
      })) };
      return asset;
    },
  };
  const plan = buildRendererResourcePlan(registry, "grey_hive", [
    { entityId: "player", entityType: "player.cenyao", actorKind: "player", active: true },
    { entityId: "sentinel", entityType: "enemy.grey_hive.sentinel", actorKind: "enemy", active: true },
    { entityId: "inactive", entityType: "enemy.unknown", actorKind: "enemy", active: false },
  ], [{ doorId: "gh_gate_a" }]);
  assert.ok(requested.includes("runtime2d.actor.cenyao.base.v1"));
  assert.ok(requested.includes("runtime2d.enemy.sentinel.base.v1"));
  const requestedPages = plan.assets.flatMap(asset => [
    `${asset.atlasUrl}\0${asset.atlasPage}`,
    ...(asset.animation?.frames ?? []).map(frame => `${frame.atlasUrl}\0${frame.atlasPage}`),
  ]);
  assert.equal(plan.atlasPages.length, new Set(requestedPages).size);
  assert.ok(plan.atlasPages.some(page => page.streamingGroup === "grey_hive"));
  assert.ok(plan.atlasPages.some(page => page.streamingGroup === "shared" && page.category === "vfx"));
  assert.ok(plan.atlasPages.some(page => page.atlasPage === 13 && page.atlasUrl === "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp"));
  assert.ok(plan.atlasPages.every(page => !page.atlasUrl.includes("must-not-load-single-sprite")));
});

test("Cenyao frame selection changes atlas rectangles by direction and keeps non-idle actions static", () => {
  const frames = [
    { direction: "east", state: "idle", atlasPage: 13, atlasUrl: "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp", atlasSha256: "b".repeat(64), atlasFrame: [682, 566, 336, 560], anchor: [0.5, 0.98392857] },
    { direction: "north", state: "idle", atlasPage: 13, atlasUrl: "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp", atlasSha256: "b".repeat(64), atlasFrame: [2, 566, 336, 560], anchor: [0.5, 0.98392857] },
  ];
  const cenyao = { assetId: "runtime2d.actor.cenyao.base.v1", atlasUrl: "assets/derived/asset-pipeline-v1/atlases/actor-shared-00.webp", atlasSha256: "a".repeat(64), atlasPage: 3, atlasFrame: [2, 2, 1422, 1086], anchorX: 0.5, anchorY: 1, animation: { frames } };
  const east = selectSpriteFrame(cenyao, "east", "idle");
  const north = selectSpriteFrame(cenyao, "north", "idle");
  assert.equal(east.mode, "idle-animation");
  assert.equal(east.atlasPage, 13);
  assert.equal(east.atlasFrame[0], 682);
  assert.equal(north.atlasFrame[0], 2);
  assert.notDeepEqual(east.atlasFrame, north.atlasFrame);
  assert.equal(selectSpriteFrame(cenyao, "east", "dash").mode, "static-action-fallback");
  assert.equal(selectSpriteFrame(cenyao, "east", "dash").atlasPage, 13);
  assert.equal(selectSpriteFrame(cenyao, "east", "dash").atlasFrame[0], 682);
});

test("unknown world or active entity assets fail closed before rendering", () => {
  const registry = { resolveAsset: id => { throw new Error(`E_ASSET_NOT_APPROVED:${id}`); } };
  assert.throws(() => buildRendererResourcePlan(registry, "unknown_world", [], []), /E_RENDERER_UNKNOWN_WORLD/);
  assert.throws(() => buildRendererResourcePlan(registry, "grey_hive", [
    { entityId: "unknown", entityType: "enemy.unknown", actorKind: "enemy", active: true },
  ], []), /E_ASSET_NOT_APPROVED/);
});

test('native enemy render sources are single reviewed actors, never full multi-pose sheets', async () => {
  const { actorAssetId, selectSpriteFrame } = await import('../dist/renderer/RendererResourcePlan.js');
  const { isolatedActorFrame, TEMPORARY_ACTOR_CROPS } = await import('../dist/renderer/ActorFrameLayout.js');
  const { AssetRegistry } = await import('../dist/assets/AssetRegistry.js');
  const real = new AssetRegistry();
  const { readFile } = await import('node:fs/promises');
  const { fileURLToPath } = await import('node:url');
  const { resolve } = await import('node:path');
  const base=fileURLToPath(new URL('../../../',import.meta.url));
  await real.registerManifest(new Uint8Array(await readFile(resolve(base,'governance/assets/RUNTIME_ASSET_MANIFEST.json'))),
    new Uint8Array(await readFile(resolve(base,'governance/assets/AI_ASSET_RELEASE_MANIFEST.json'))),async path=>new Uint8Array(await readFile(resolve(base,path))));
  const profiles=JSON.parse(await readFile(resolve(base,'server-rs/data/actor_profiles_v1.json'),'utf8')).profiles;
  const renderTypes=new Set([...profiles.map(p=>p.renderType),'enemy.grey_hive.sentinel']);
  for(const entityType of renderTypes){
    const asset=real.resolveAsset(actorAssetId({entityType}));const frame=selectSpriteFrame(asset,'south','idle');
    if(Object.hasOwn(TEMPORARY_ACTOR_CROPS,asset.assetId)){
      assert.equal(frame.mode,'temporary-static-actor');assert.ok(frame.atlasFrame[2]<asset.atlasFrame[2]);assert.ok(frame.atlasFrame[3]<asset.atlasFrame[3]);
      assert.ok(frame.anchor[0]>0&&frame.anchor[0]<1);assert.ok(frame.anchor[1]>0&&frame.anchor[1]<=1);
    } else assert.equal(frame.mode,'static');
  }
  const worker=real.resolveAsset('runtime2d.enemy.infected_maintenance_worker.v1');
  assert.deepEqual(selectSpriteFrame(worker,'south','idle').atlasFrame,[19,5,428,522]);
  assert.throws(()=>isolatedActorFrame({...worker,sourceSha256:'0'.repeat(64)}),/E_RENDERER_ACTOR_CROP_SOURCE/);
  assert.throws(()=>isolatedActorFrame({...worker,assetId:'runtime2d.enemy.unreviewed_sheet'}),/E_RENDERER_ACTOR_FRAME_UNREVIEWED/);
});
