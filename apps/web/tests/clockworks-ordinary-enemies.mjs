import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { Container, Polygon, Sprite, Texture } from "pixi.js";
import { ClockworksEnemyModel, CLOCKWORKS_ENEMY_PLACEHOLDERS, clockworksEnemyForEvent, enemyWarningGroundPoints } from "../dist/renderer/ClockworksEnemyModel.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { projectWorldPoint } from "../dist/renderer/CameraModel.js";
import { projectWorldUi } from "../dist/renderer/WorldUiModel.js";
import { actorAssetId, buildRendererResourcePlan } from "../dist/renderer/RendererResourcePlan.js";
import { AssetRegistry } from "../dist/assets/AssetRegistry.js";
import { AudioCuePlayer } from "../dist/audio/AudioCuePlayer.js";
import { audioCueForPresentationEvent } from "../dist/audio/AudioEventMap.js";

const droneType = "enemy.clockworks.pressure_drone", houndType = "enemy.clockworks.furnace_hound";
const point = { xM: 5, yM: 0, zM: 7 };
const actor = (entityId = "cw_drone_01", entityType = droneType, active = true) => ({
  entityId, entityType, active, actorKind: "enemy", transform: { positionM: { ...point }, yawRad: 0 },
});
const drone = actor(), hound = actor("cw_hound_01", houndType);
function snapshot(overrides = {}) {
  return { protocolVersion: 3, worldId: "clockworks", sceneId: "cw_pressure_hall", worldEpoch: 7, serverTick: 100,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 } },
    actors: [structuredClone(drone), structuredClone(hound)], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] }, ...overrides };
}
function event(eventId = 1, overrides = {}) {
  return { protocolVersion: 2, worldEpoch: 7, eventId, serverTick: 100, kind: "EnemyAttackTelegraph",
    actorId: drone.entityId, attackKind: "pressure_shot", durationMs: 500, rangeM: 6, radiusM: .25,
    intensity: 1, positionM: { ...point }, directionRad: Math.PI / 2, ...overrides };
}
const camera = { width: 1200, height: 800, origin: { xM: 0, yM: 0, zM: 0 }, pixelsPerMeter: 48 };

function renderer(model = new ClockworksEnemyModel()) {
  const r = Object.create(WorldRenderer.prototype);
  r.layerContainers = new Map([["L3_ACTORS", new Container()], ["L7_VFX", new Container()]]);
  r.clockworksEnemyModel = model; r.clockworksEnemyGraphics = new Map(); r.clockworksEnemyGlows = new Map();
  r.clockworksEnemyBodies = new Map(); r.sprites = new Map(); r.sentinelTelegraphGraphics = new Map();
  r.sentinelGlowGraphics = new Map(); r.sentinelTelegraphModel = { reset() {} }; r.transientPresentationRevision = 0;
  r.machineryGraphics = new Map(); return r;
}

test("temporary semantic aliases resolve only to admitted existing actor assets", async () => {
  const root = new URL("../../../", import.meta.url);
  const registry = new AssetRegistry();
  await registry.registerManifest(new Uint8Array(await readFile(new URL("governance/assets/RUNTIME_ASSET_MANIFEST.json", root))),
    new Uint8Array(await readFile(new URL("governance/assets/AI_ASSET_RELEASE_MANIFEST.json", root))),
    async path => new Uint8Array(await readFile(new URL(path, root))));
  const plan = buildRendererResourcePlan(registry, "clockworks", [drone, hound], []);
  for (const enemy of [drone, hound]) {
    const style = CLOCKWORKS_ENEMY_PLACEHOLDERS[enemy.entityType];
    assert.equal(style.temporary_visual, true);
    assert.equal(actorAssetId(enemy), style.assetId);
    assert.ok(plan.assets.some(item => item.assetId === style.assetId));
    assert.equal(registry.resolveAsset(style.assetId).kind, "Actor");
  }
  assert.notEqual(CLOCKWORKS_ENEMY_PLACEHOLDERS[droneType].silhouette, CLOCKWORKS_ENEMY_PLACEHOLDERS[houndType].silhouette);
  assert.ok(CLOCKWORKS_ENEMY_PLACEHOLDERS[droneType].widthPx < CLOCKWORKS_ENEMY_PLACEHOLDERS[houndType].widthPx);
});

test("malformed, stale, future, wrong epoch/type/actor/scene and duplicate identity cues fail closed", () => {
  const invalid = [
    [{ protocolVersion: 1 }], [{ actorId: undefined }], [{ actorId: "wrong" }], [{ attackKind: "bite" }],
    [{ durationMs: undefined }], [{ durationMs: 0 }], [{ durationMs: NaN }], [{ durationMs: Infinity }],
    [{ rangeM: undefined }], [{ rangeM: -1 }], [{ radiusM: NaN }], [{ radiusM: -1 }], [{ directionRad: NaN }],
    [{ intensity: Infinity }], [{ positionM: { ...point, zM: NaN } }], [{ eventId: 0 }], [{ eventId: 1.5 }],
    [{ serverTick: 101 }], [{ serverTick: 60 }], [{ worldEpoch: 6 }], [{ kind: "Unknown" }],
    [{}, { worldId: "mist_harbor" }], [{}, { sceneId: "mh_signal_yard" }], [{}, { protocolVersion: 2 }],
    [{}, { actors: [drone, drone] }], [{}, { actors: [{ ...drone, actorKind: "player" }] }],
    [{}, { actors: [{ ...drone, entityType: houndType }] }], [{}, { actors: [{ ...drone, active: false }] }],
    [{ kind: "PressureShotMotion", attackKind: "leap" }], [{ kind: "FurnaceHoundLeapMotion" }],
  ];
  for (const [cue, state] of invalid) {
    const model = new ClockworksEnemyModel();
    assert.deepEqual(model.accept([event(1, cue)], snapshot(state)), [], JSON.stringify([cue, state]));
    assert.deepEqual(model.project(snapshot(state), false), []);
  }
});

test("durationMs is consumed on the 60 Hz event timeline without copied combat tuning", () => {
  const model = new ClockworksEnemyModel();
  assert.deepEqual(model.accept([event(1, { durationMs: 1250 })], snapshot()), [1]);
  const later = model.project(snapshot({ serverTick: 130 }), true)[0];
  assert.equal(later.remainingMs, 750); assert.equal(later.progress, .4);
  assert.deepEqual(model.project(snapshot({ serverTick: 175 }), false), []);
  assert.deepEqual(new ClockworksEnemyModel().accept([event()], snapshot({ serverTick: 130 })), []);
});

test("committed corridor direction, range and radius project precisely without chasing player aim", () => {
  const model = new ClockworksEnemyModel(); model.accept([event()], snapshot());
  const frame = model.project(snapshot(), false)[0];
  const [a, b, c, d] = enemyWarningGroundPoints(frame);
  assert.deepEqual([a.xM, b.xM, c.xM, d.xM], [5, 11, 11, 5]);
  assert.deepEqual([a.zM, b.zM, c.zM, d.zM].map(x => Math.round(x * 100) / 100), [6.75, 6.75, 7.25, 7.25]);
  const moved = snapshot(); moved.player.transform.positionM = { xM: 100, yM: 0, zM: -100 };
  assert.deepEqual(model.project(moved, false)[0], frame);
});

test("Bite warning uses authoritative radial reach for rear and side points, independent of committed yaw", () => {
  const model = new ClockworksEnemyModel();
  model.accept([event(1, { actorId: hound.entityId, attackKind: "bite", rangeM: 1.7, radiusM: .25 })], snapshot());
  const frame = model.project(snapshot(), true)[0];
  const ring = enemyWarningGroundPoints(frame);
  assert.equal(ring.length, 64);
  assert.deepEqual(enemyWarningGroundPoints({ ...frame, directionRad: -2.3 }), ring);
  for (const p of ring) {
    // Rust checks center distance <= attack_range_m plus LOS, without adding a body radius.
    assert.ok(Math.abs(Math.hypot(p.xM - point.xM, p.zM - point.zM) - frame.rangeM) < 1e-12);
    assert.equal(p.yM, point.yM);
  }
  // Facing +X: -X is behind, both +/-Z are beside the Hound.
  for (const [x, z] of [[-1.7, 0], [0, -1.7], [0, 1.7]]) {
    assert.ok(ring.some(p => Math.abs(p.xM - point.xM - x) < 1e-12 && Math.abs(p.zM - point.zM - z) < 1e-12));
  }
  const shot = enemyWarningGroundPoints({ ...frame, attackKind: "pressure_shot" });
  assert.deepEqual(enemyWarningGroundPoints({ ...frame, attackKind: "leap" }), shot);
  assert.equal(shot.length, 4);
  assert.ok(shot.every(p => p.xM >= point.xM), "Shot/Leap keep their committed forward corridors");
});

test("real WorldRenderer draws a full Bite ring enclosing rear/side danger without a forward arrow", () => {
  for (const pixelsPerMeter of [24, 48, 72]) for (const reducedMotion of [false, true]) {
    const model = new ClockworksEnemyModel();
    model.accept([event(1, { actorId: hound.entityId, attackKind: "bite", rangeM: 1.7, radiusM: .25 })], snapshot());
    const r = renderer(model), view = { ...camera, pixelsPerMeter };
    r.renderClockworksEnemyCues(model.project(snapshot(), reducedMotion), view);
    const cue = r.clockworksEnemyGraphics.get(1), instructions = cue.context.instructions;
    assert.equal(instructions.length, 1, "only the radial boundary, no forward-only arrow or centerline");
    assert.equal(instructions[0].action, "stroke");
    assert.equal(instructions[0].data.style.width, 1.3);
    const paths = instructions[0].data.path.instructions;
    assert.equal(paths.length, 1); assert.equal(paths[0].action, "poly");
    const polygon = new Polygon(paths[0].data[0]);
    for (const [x, z] of [[-1, 0], [0, -1], [0, 1], [1, 0]]) {
      for (const [distance, inside] of [[1.6, true], [1.8, false]]) {
        const p = projectWorldPoint({ ...point, xM: point.xM + x * distance, zM: point.zM + z * distance }, view);
        assert.equal(polygon.contains((p.x - cue.position.x) / cue.scale.x, (p.footY - cue.position.y) / cue.scale.y), inside);
      }
    }
    assert.ok(r.clockworksEnemyGlows.has(1), "body warning glow remains present");
    r.clearTransientPresentation();
  }
  for (const [attackKind, actorId, count] of [["pressure_shot", drone.entityId, 4], ["leap", hound.entityId, 5]]) {
    const model = new ClockworksEnemyModel(); model.accept([event(1, { actorId, attackKind })], snapshot());
    const r = renderer(model); r.renderClockworksEnemyCues(model.project(snapshot(), true), camera);
    const instructions = r.clockworksEnemyGraphics.get(1).context.instructions;
    assert.equal(instructions.length, count, "Shot/Leap keep corridor, direction line, arrow and endpoint marks");
    assert.equal(instructions[0].data.path.instructions[0].data[0].length, 8);
    r.clearTransientPresentation();
  }
});

test("motion replaces windup, updates only server positions, and carries its remaining lifetime", () => {
  const model = new ClockworksEnemyModel(); model.accept([event()], snapshot());
  const motion = event(2, { kind: "PressureShotMotion", serverTick: 101, durationMs: 300, positionM: { ...point, xM: 8 } });
  assert.deepEqual(model.accept([motion], snapshot({ serverTick: 101 })), [2]);
  const frame = model.project(snapshot({ serverTick: 105 }), false)[0];
  assert.equal(frame.kind, "PressureShotMotion"); assert.deepEqual(frame.positionM, motion.positionM);
  assert.ok(Math.abs(frame.remainingMs - (300 - 4 * 1000 / 60)) < .0001);
  const next = event(3, { ...motion, eventId: 3, serverTick: 106, durationMs: 200, positionM: { ...point, xM: 9 } });
  model.accept([next], snapshot({ serverTick: 106 }));
  assert.deepEqual(model.project(snapshot({ serverTick: 106 }), true).map(item => item.positionM), [next.positionM]);
  model.accept([event(4, { kind: "EnemyAttackImpact", serverTick: 107 })], snapshot({ serverTick: 107 }));
  assert.deepEqual(model.project(snapshot({ serverTick: 107 }), false).map(item => item.kind), ["EnemyAttackImpact"]);
});

test("death/stagger replace an actor's old warnings and preserve unrelated actors", () => {
  const model = new ClockworksEnemyModel();
  model.accept([event(), event(2, { actorId: hound.entityId, attackKind: "leap" })], snapshot());
  model.accept([event(3, { kind: "EnemyStagger" })], snapshot());
  assert.deepEqual(model.project(snapshot(), false).map(item => item.kind), ["EnemyAttackTelegraph", "EnemyStagger"]);
  const dead = snapshot({ actors: [{ ...drone, active: false }, hound] });
  model.accept([event(4, { kind: "EnemyDeath" })], dead);
  assert.deepEqual(model.project(dead, false).map(item => item.kind), ["EnemyAttackTelegraph", "EnemyDeath"]);
  assert.equal(clockworksEnemyForEvent(event(5, { kind: "EnemyDeath" }), snapshot()), null);
  assert.equal(model.project(snapshot(), false).some(item => item.actorId === drone.entityId), false);
});

test("IDs dedupe, unaccepted unknown kinds cannot poison IDs, and actor replacement clears stored alert", () => {
  const model = new ClockworksEnemyModel();
  assert.deepEqual(model.accept([event(2), event(1)], snapshot()), [2]);
  assert.deepEqual(model.accept([event(2)], snapshot()), []);
  assert.deepEqual(model.accept([event(3, { kind: "Unknown" }), event(3)], snapshot()), [3]);
  model.accept([event(4, { kind: "EnemyAlert", attackKind: undefined })], snapshot());
  assert.equal(model.project(snapshot({ actors: [{ ...drone, entityType: houndType }] }), false).length, 0);
});

test("scene/world/epoch/actor removal, rollback, reset and old protocol remove active effects", () => {
  for (const changed of [{ sceneId: "cw_boiler_chamber" }, { worldId: "grey_hive" }, { worldEpoch: 8 },
    { actors: [] }, { serverTick: 99 }, { protocolVersion: 2 }]) {
    const model = new ClockworksEnemyModel(); model.accept([event()], snapshot());
    assert.deepEqual(model.project(snapshot(changed), false), []);
  }
  const model = new ClockworksEnemyModel(); model.accept([event()], snapshot()); model.reset();
  assert.deepEqual(model.project(snapshot(), true), []);
});

test("renderer warning/glow projection and reduced-motion opacity stay steady; motion has no invented path", () => {
  const model = new ClockworksEnemyModel(); model.accept([event()], snapshot()); const r = renderer(model);
  r.renderClockworksEnemyCues(model.project(snapshot(), true), camera);
  const cue = r.clockworksEnemyGraphics.get(1), glow = r.clockworksEnemyGlows.get(1);
  const position = projectWorldPoint(point, camera);
  assert.equal(cue.position.x, position.x); assert.equal(cue.position.y, position.footY); assert.equal(cue.alpha, .9);
  assert.equal(glow.position.x, position.x); assert.ok(glow.position.y < position.y);
  r.renderClockworksEnemyCues(model.project(snapshot({ serverTick: 103 }), true), { ...camera, pixelsPerMeter: 24 });
  assert.equal(cue.scale.x, .5); assert.equal(cue.alpha, .9); assert.equal(glow.alpha, .9);
  model.accept([event(2, { kind: "PressureShotMotion", positionM: { ...point, yM: 1 } })], snapshot({ serverTick: 103 }));
  r.renderClockworksEnemyCues(model.project(snapshot({ serverTick: 103 }), true), camera);
  assert.equal(r.clockworksEnemyGlows.size, 0);
  assert.equal(r.clockworksEnemyGraphics.get(2).position.y, projectWorldPoint({ ...point, yM: 1 }, camera).y);
  r.clearTransientPresentation(); assert.equal(r.layerContainers.get("L7_VFX").children.length, 0);
  assert.deepEqual(model.project(snapshot(), true), []);
});

test("placeholder sprites have distinct low silhouettes, bottom anchors and elliptical ground shadows", () => {
  const r = renderer();
  for (const enemy of [drone, hound]) {
    const sprite = new Sprite(Texture.WHITE); sprite.zIndex = 2; sprite.anchor.set(.5,1);
    r.sprites.set(`actor:${enemy.entityId}`, { sprite }); r.layerContainers.get("L3_ACTORS").addChild(sprite);
  }
  const elevated = [{ ...drone, transform: { ...drone.transform, positionM: { ...point, yM: 2 } } }, hound];
  r.renderClockworksEnemyBodies(elevated, camera);
  const d = r.sprites.get(`actor:${drone.entityId}`).sprite, h = r.sprites.get(`actor:${hound.entityId}`).sprite;
  assert.equal(d.width, 32); assert.equal(h.width, 62); assert.equal(h.height, 30);
  assert.deepEqual([d.anchor.x, d.anchor.y], [.5, 1]); assert.match(d.label, /temporary_visual=true/);
  assert.equal(r.clockworksEnemyBodies.get(drone.entityId).shadow.position.y, projectWorldPoint(elevated[0].transform.positionM, camera).y);
  assert.ok(d.position.y < r.clockworksEnemyBodies.get(drone.entityId).shadow.position.y);
  r.renderClockworksEnemyBodies([], camera); assert.equal(r.clockworksEnemyBodies.size, 0);
});

test("Enemy Vitals remain authority-granted coarse text only for both semantic types", () => {
  const value = snapshot();
  value.capabilities.enemyVitals = [{ entityId: drone.entityId, tier: "critical", currentHp: 1, maxHp: 99 },
    { entityId: hound.entityId, tier: "wounded", currentHp: 11, maxHp: 77 }];
  assert.deepEqual(projectWorldUi(value, camera), []);
  value.capabilities.items = [{ capabilityId: "information.enemy_vitals_basic", granted: true }];
  const markers = projectWorldUi(value, camera);
  assert.deepEqual(markers.map(item => item.label).sort(), ["Critical", "Wounded"]);
  assert.equal(JSON.stringify(markers).includes("currentHp"), false);
  const model = new ClockworksEnemyModel(); model.accept([event()], value);
  assert.equal(JSON.stringify(model.project(value, true)).includes("Hp"), false);
  value.actors[0].active = false; assert.equal(projectWorldUi(value, camera).length, 1);
});

function audioHarness() {
  const played = [], stopped = [];
  const runtime = { state: () => "running", unlock: async () => true,
    play(cue, volume, pan, actorId) { played.push({ cue, volume, pan, actorId }); return true; },
    stopActorVoices(id) { stopped.push(id); }, stopVoices() { stopped.push("all"); },
    startAmbient: () => true, stopAmbient() {}, close() {} };
  const audio = new AudioCuePlayer({ getStorage: () => null, createRuntime: () => runtime });
  return { audio, played, stopped };
}

test("temporary warning and impact recipes distinguish all attacks; motion never maps a sound", () => {
  for (const kind of ["EnemyAttackTelegraph", "EnemyAttackImpact"]) {
    const recipes = ["pressure_shot", "bite", "leap"].map(attackKind => audioCueForPresentationEvent(event(1, { kind, attackKind })));
    assert.equal(new Set(recipes.map(item => JSON.stringify(item.tones))).size, 3);
    assert.ok(recipes.every(item => item.temporary_audio && item.peakGain <= .22));
  }
  assert.equal(audioCueForPresentationEvent(event(1, { kind: "PressureShotMotion" })), null);
  assert.equal(audioCueForPresentationEvent(event(1, { kind: "FurnaceHoundLeapMotion", attackKind: "leap" })), null);
});

test("audio independently checks actor/context, dedupes, truncates to remaining duration, and stops on death", async () => {
  const { audio, played, stopped } = audioHarness(); await audio.unlock();
  assert.equal(audio.handlePresentationEvents([null, {}, event()]), 0);
  audio.setListenerSnapshot(snapshot());
  assert.equal(audio.handlePresentationEvents([event(2, { actorId: "wrong" })]), 0);
  assert.equal(audio.handlePresentationEvents([event(3, { durationMs: 80 })]), 1);
  assert.equal(played[0].actorId, drone.entityId);
  assert.ok(played[0].cue.tones.every(tone => tone.durationMs + tone.offsetMs <= 80));
  assert.equal(audio.handlePresentationEvents([event(3)]), 0);
  assert.equal(audio.handlePresentationEvents([event(4, { kind: "PressureShotMotion" })]), 0);
  assert.equal(played.length, 1); assert.ok(stopped.includes(drone.entityId));
  const dead = snapshot({ actors: [{ ...drone, active: false }, hound] }); audio.setListenerSnapshot(dead);
  assert.equal(audio.handlePresentationEvents([event(5, { kind: "EnemyAttackTelegraph" })]), 0);
  assert.equal(audio.handlePresentationEvents([event(6, { kind: "EnemyDeath" })]), 1);
  audio.stop(); assert.equal(audio.handlePresentationEvents([event(7)]), 0);
  audio.setListenerSnapshot(snapshot({ worldEpoch: 8 })); assert.equal(audio.handlePresentationEvents([event(1, { worldEpoch: 7 })]), 0);
  await audio.dispose();
});

test("audio scene loss, suspend, mute, invalid snapshot and stale cue never replay a warning", async () => {
  const { audio, played, stopped } = audioHarness(); await audio.unlock(); audio.setListenerSnapshot(snapshot());
  audio.suspend(); assert.equal(audio.handlePresentationEvents([event()]), 0); audio.resume();
  assert.equal(audio.handlePresentationEvents([event()]), 0);
  audio.setMuted(true); assert.equal(audio.handlePresentationEvents([event(2)]), 0); audio.setMuted(false);
  audio.setListenerSnapshot(snapshot({ serverTick: 131 })); assert.equal(audio.handlePresentationEvents([event(3)]), 0);
  audio.setListenerSnapshot(snapshot({ sceneId: "cw_boiler_chamber", actors: [] }));
  assert.equal(audio.handlePresentationEvents([event(4)]), 0); assert.ok(stopped.includes(drone.entityId));
  audio.setListenerSnapshot(snapshot({ protocolVersion: 2 })); assert.equal(audio.handlePresentationEvents([event(5)]), 0);
  assert.equal(played.length, 0); await audio.dispose();
});

test("main forwards only model-authorized ordinary enemy events and stops audio on transient/context loss", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(main, /acceptClockworksEnemyPresentationEvents\(events, latest\)/);
  assert.match(main, /!isClockworksEnemyCue\(event.kind\)/);
  assert.match(main, /onClearTransientPresentation:.*audioCuePlayer.stop\(\)/);
  assert.match(main, /webglcontextlost.*audioCuePlayer.stop\(\)/);
  const source = await readFile(new URL("../src/renderer/WorldRenderer.ts", import.meta.url), "utf8");
  assert.match(source, /this.surface\?\.lost\(\);\s*this.clearTransientPresentation\(\)/);
});

test("ordinary audio cannot advance its epoch from an unpaired future event", async () => {
  const { audio, played } = audioHarness(); await audio.unlock(); audio.setListenerSnapshot(snapshot());
  assert.equal(audio.handlePresentationEvents([event(1, { worldEpoch: 999 })]), 0);
  assert.equal(audio.handlePresentationEvents([event(2)]), 1);
  assert.equal(played.length, 1); await audio.dispose();
});

test("alert, stagger and death need no invented attackKind/range or hidden enemy information", () => {
  const privateRead = () => { throw new Error("hidden enemy field was read"); };
  const value = snapshot();
  for (const enemy of value.actors) {
    Object.defineProperties(enemy, { currentHp: { get: privateRead }, maxHp: { get: privateRead },
      ai: { get: privateRead }, attackCooldown: { get: privateRead } });
  }
  const model = new ClockworksEnemyModel();
  for (const [index, kind] of ["EnemyAlert", "EnemyStagger", "EnemyDeath"].entries()) {
    if (kind === "EnemyDeath") value.actors[0].active = false;
    const cue = event(index + 1, { kind, attackKind: undefined, rangeM: undefined });
    assert.deepEqual(model.accept([cue], value), [index + 1]);
    assert.ok(model.project(value, true).some(item => item.kind === kind));
  }
});

test("hound bite and leap are scoped to their semantic actor and stale airborne cues expire", () => {
  const model = new ClockworksEnemyModel(); const value = snapshot({ sceneId: "cw_boiler_chamber" });
  const warning = event(1, { actorId: hound.entityId, attackKind: "bite" });
  assert.deepEqual(model.accept([warning], value), [1]);
  const leap = event(2, { actorId: hound.entityId, kind: "FurnaceHoundLeapMotion", attackKind: "leap",
    positionM: { ...point, yM: .6 }, durationMs: 100 });
  assert.deepEqual(model.accept([leap], value), [2]);
  assert.deepEqual(model.project(value, true)[0].positionM, leap.positionM);
  assert.deepEqual(model.project({ ...value, serverTick: 106 }, true), []);
});

test("terminal impact wins over same-tick higher-ID trailing motion in renderer and audio batches", async () => {
  const model = new ClockworksEnemyModel(); const value = snapshot();
  const impact = event(2, { kind: "EnemyAttackImpact" });
  const trailing = event(3, { kind: "PressureShotMotion" });
  assert.deepEqual(model.accept([event(), impact, trailing], value), [2]);
  assert.deepEqual(model.project(value, true).map(item => item.kind), ["EnemyAttackImpact"]);
  const { audio, played, stopped } = audioHarness(); await audio.unlock(); audio.setListenerSnapshot(value);
  assert.equal(audio.handlePresentationEvents([event(), impact, trailing]), 1);
  assert.deepEqual(played.map(item => item.cue.id), ["audio.cw.enemy.drone.impact"]);
  assert.equal(stopped.filter(id => id === drone.entityId).length, 1, "trailing motion cannot silence the impact");
  // Fresh Continue epochs can republish currently active motion without synthesizing a windup.
  const restored = snapshot({ worldEpoch: 8, serverTick: 101 });
  assert.deepEqual(model.accept([event(1, { kind: "PressureShotMotion", worldEpoch: 8, serverTick: 101 })], restored), [1]);
  assert.equal(model.project(restored, true)[0].kind, "PressureShotMotion");
  await audio.dispose();
});
