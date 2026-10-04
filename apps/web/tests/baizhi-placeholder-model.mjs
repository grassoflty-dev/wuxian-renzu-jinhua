import test from "node:test";
import assert from "node:assert/strict";
import { BaizhiPlaceholder } from "../dist/renderer/BaizhiPlaceholder.js";
import { BAIZHI_ENTITY_ID, BAIZHI_PLACEHOLDER_HEIGHT_M, BAIZHI_PLACEHOLDER_WIDTH_M,
  projectBaizhiPlaceholder, resolveBaizhiPresentation } from "../dist/renderer/BaizhiPresentation.js";
import { projectWorldPoint, viewportPixelsPerMeter } from "../dist/renderer/CameraModel.js";
import { sharesWorldDepth, worldDepthRanks } from "../dist/renderer/WorldDepthModel.js";

function wire(overrides = {}) {
  return { protocolVersion: 3, worldId: "grey_hive", sceneId: "gh_bio_isolation", worldEpoch: 3,
    baizhi: { schemaVersion: 1, choice: "unresolved", available: true, canInteract: true },
    npcs: [{ entityId: BAIZHI_ENTITY_ID, entityType: "npc.baizhi", position: [8, 0, 10.5],
      yawRad: Math.PI, interactable: true }], ...overrides };
}
const camera = { width: 1280, height: 720, origin: { xM: 8, yM: 0, zM: 9.3 }, pixelsPerMeter: 48 };

test("all three choices retain the one exact Bio NPC, including far-distance and held-dialogue states", () => {
  for (const choice of ["unresolved", "taken", "left"]) for (const canInteract of [true, false]) {
    const value = wire(); value.baizhi.choice = choice;
    value.baizhi.canInteract = canInteract; value.npcs[0].interactable = canInteract;
    value.player = { transform: { positionM: { xM: 100, yM: 0, zM: 100 } } };
    value.paused = !canInteract;
    const result = resolveBaizhiPresentation(value);
    assert.equal(result.baizhi.choice, choice);
    assert.equal(result.baizhi.canInteract, canInteract);
    assert.equal(result.npc.interactable, canInteract);
    assert.ok(projectBaizhiPlaceholder(value, camera), "interaction eligibility never controls visibility");
  }
});

test("old sender, absent fields, partial wire and non-Bio states hide only this optional NPC", () => {
  const missing = [{}, { baizhi: undefined }, { npcs: undefined }, { baizhi: null }, { npcs: null },
    { npcs: [] }, { npcs: {} }, { npcs: [null] }, { baizhi: [] }, { protocolVersion: 2 },
    { worldId: "return_station", sceneId: "rs_core_room" }, { worldId: "mist_harbor" },
    { worldId: "clockworks" }, { sceneId: "gh_gate_b" }, { sceneId: "gh_lockdown" },
    ...[undefined, null, NaN, Infinity, -1, 0, 1.25, "3", true].map(worldEpoch => ({ worldEpoch }))];
  assert.equal(resolveBaizhiPresentation({}), null);
  for (const change of missing.slice(1)) assert.equal(resolveBaizhiPresentation(wire(change)), null, JSON.stringify(change));
  for (const key of ["schemaVersion", "choice", "available", "canInteract"]) {
    const value = wire(); delete value.baizhi[key]; assert.equal(resolveBaizhiPresentation(value), null, key);
  }
  for (const key of ["entityId", "entityType", "position", "yawRad", "interactable"]) {
    const value = wire(); delete value.npcs[0][key]; assert.equal(resolveBaizhiPresentation(value), null, key);
  }
});

test("epoch zero cannot produce a visible NPC or a usable F/UI projection", () => {
  const value = wire({ worldEpoch: 0 });
  assert.equal(resolveBaizhiPresentation(value), null);
  assert.equal(projectBaizhiPlaceholder(value, camera), null);
  assert.ok(resolveBaizhiPresentation(wire({ worldEpoch: 1 })), "first valid authority epoch remains supported");
});

test("schema, enum, numeric and boolean malformations fail isolated without coercion", () => {
  for (const schemaVersion of ["1", true, 0, 2, NaN, Infinity, null]) {
    const value = wire(); value.baizhi.schemaVersion = schemaVersion;
    assert.equal(resolveBaizhiPresentation(value), null);
  }
  for (const choice of ["Taken", "", "unknown", 0, true, null, [], {}]) {
    const value = wire(); value.baizhi.choice = choice; assert.equal(resolveBaizhiPresentation(value), null);
  }
  for (const field of ["available", "canInteract"]) for (const invalid of [0, 1, "true", "false", null, NaN, {}, []]) {
    const value = wire(); value.baizhi[field] = invalid;
    assert.equal(resolveBaizhiPresentation(value), null, `${field}: ${String(invalid)}`);
  }
  for (const invalid of [0, 1, "true", null, undefined, NaN]) {
    const value = wire(); value.npcs[0].interactable = invalid;
    assert.equal(resolveBaizhiPresentation(value), null);
  }
  for (const available of [true, false]) for (const canInteract of [true, false]) for (const interactable of [true, false]) {
    const value = wire(); Object.assign(value.baizhi, { available, canInteract });
    value.npcs[0].interactable = interactable;
    assert.equal(resolveBaizhiPresentation(value) !== null, available && canInteract === interactable);
  }
});

test("duplicates, extra unknown NPCs and wrong exact identity cannot use the special code path", () => {
  const value = wire(), npc = value.npcs[0];
  for (const npcs of [[npc, structuredClone(npc)], [npc, { ...npc, entityId: "unknown" }],
    [{ ...npc, entityId: "other" }], [{ ...npc, entityType: "npc.unknown" }],
    [{ ...npc, entityType: "enemy.npc.baizhi" }]]) {
    assert.equal(resolveBaizhiPresentation(wire({ npcs })), null);
    assert.equal(projectBaizhiPlaceholder(wire({ npcs }), camera), null);
  }
  assert.equal(resolveBaizhiPresentation({ protocolVersion: 3, worldId: "grey_hive", sceneId: "gh_bio_isolation",
    worldEpoch: 3, dialogFlags: { baizhi: "taken" }, progression: { hiveChoice: "taken", completed: true } }), null);
});

test("frozen finite foot position and facing -Z are checked before projection", () => {
  for (const position of [[8, 0], [8, 0, 10.5, 1], ["8", 0, 10.5], [8, NaN, 10.5],
    [8, 0, Infinity], [8.01, 0, 10.5], [8, 0.01, 10.5], [8, 0, 10.49], [8, false, 10.5],
    [8, , 10.5], new Array(3), {}, null]) {
    const value = wire(); value.npcs[0].position = position;
    assert.equal(projectBaizhiPlaceholder(value, camera), null);
  }
  for (const yawRad of [NaN, Infinity, -Infinity, "3.141592653589793", null, true, 0, Math.PI / 2, Math.PI + 0.01]) {
    const value = wire(); value.npcs[0].yawRad = yawRad;
    assert.equal(projectBaizhiPlaceholder(value, camera), null);
  }
  for (const yawRad of [Math.PI, -Math.PI, 3 * Math.PI]) {
    const value = wire(); value.npcs[0].yawRad = yawRad;
    const frame = projectBaizhiPlaceholder(value, camera);
    assert.ok(frame.facing.x > 0 && frame.facing.y < 0, "world -Z faces screen north-east under the existing projection");
  }
});

test("presentation is detached from wire and adds no state writes, combat identity, or resource", () => {
  const value = wire(), before = structuredClone(value), result = resolveBaizhiPresentation(value);
  assert.deepEqual(value, before);
  result.npc.position[0] = 100;
  result.baizhi.choice = "taken";
  assert.deepEqual(value, before);
  const frame = projectBaizhiPlaceholder(value, camera);
  assert.deepEqual(Object.keys(frame).sort(), ["displayScale", "facing", "footY", "key", "layer", "position", "screenX", "screenY", "yawRad"]);
});

test("720p/1080p geometry uses exact 1.7m x 0.45m bottom-center body and current projection scale", () => {
  for (const [width, height] of [[1280, 720], [1920, 1080]]) {
    const ppm = viewportPixelsPerMeter(width, height), view = { ...camera, width, height, pixelsPerMeter: ppm };
    const frame = projectBaizhiPlaceholder(wire(), view), body = new BaizhiPlaceholder();
    body.applyFrame(frame, 30);
    const bounds = body.body.getLocalBounds(), foot = projectWorldPoint({ xM: 8, yM: 0, zM: 10.5 }, view);
    const epsilon = 1e-10;
    assert.ok(Math.abs(bounds.width * body.scale.x - BAIZHI_PLACEHOLDER_WIDTH_M * ppm) < epsilon);
    assert.ok(Math.abs(bounds.height * body.scale.y - BAIZHI_PLACEHOLDER_HEIGHT_M * ppm) < epsilon);
    assert.ok(Math.abs(bounds.minX + bounds.maxX) < epsilon);
    assert.equal(bounds.maxY, 0);
    assert.deepEqual([body.x, body.y, frame.footY], [foot.x, foot.y, foot.footY]);
    assert.equal(body.rotation, 0, "standing height never rotates with facing");
    assert.equal(body.pivot.x, 0); assert.equal(body.pivot.y, 0);
    assert.equal(body.nameLabel.text, "白芷"); assert.equal(body.nameLabel.anchor.x, 0.5);
    assert.equal(body.label, "DEV_WHITEZHI_PLACEHOLDER");
    assert.ok(body.children.every(child => child.text === undefined || !child.text.includes("DEV_")));
    assert.equal(body.eventMode, "none"); assert.equal(body.interactiveChildren, false);
    assert.equal(body.children.length, 2, "no invented shadow, hit area, health bar, or attack marker");
    assert.equal(body.body.context.instructions.length, 1);
    assert.equal(body.body.context.instructions[0].action, "fill", "body and head marker share one monochrome fill");
    body.destroy(); body.destroy();
    assert.ok(body.destroyed && body.body.destroyed && body.nameLabel.destroyed);
  }
});

test("NPC crosses actors and physical props in one stable foot-depth order instead of drawing on top", () => {
  const npc = projectBaizhiPlaceholder(wire(), camera);
  const objects = [{ key: "scene:back", footY: npc.footY - 1, layer: "L2_BACK_PROPS" },
    npc, { key: "actor:player", footY: npc.footY, layer: "L3_ACTORS" },
    { key: "scene:front", footY: npc.footY + 1, layer: "L4_DYNAMIC_PROPS" }];
  assert.ok(sharesWorldDepth(npc.layer));
  const ranks = worldDepthRanks(objects);
  assert.deepEqual([...ranks.keys()], ["scene:back", "actor:player", npc.key, "scene:front"]);
  assert.deepEqual(worldDepthRanks([...objects].reverse()), ranks);
  for (const delta of [-2, 2]) {
    const moved = objects.map(o => o.key === "actor:player" ? { ...o, footY: npc.footY + delta } : o);
    const next = worldDepthRanks(moved);
    assert.equal(next.get("actor:player") < next.get(npc.key), delta < 0);
  }
});

test("invalid camera values cannot leak non-finite geometry", () => {
  for (const change of [{ pixelsPerMeter: NaN }, { pixelsPerMeter: Infinity }, { pixelsPerMeter: 0 },
    { pixelsPerMeter: -48 }, { width: NaN }, { height: Infinity }, { width: 0 }, { height: -1 },
    { origin: { xM: NaN, yM: 0, zM: 0 } }]) {
    assert.equal(projectBaizhiPlaceholder(wire(), { ...camera, ...change }), null);
  }
});
