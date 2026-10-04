import test from "node:test";
import assert from "node:assert/strict";
import { Rectangle, Texture, TextureSource } from "pixi.js";
import { PlayerLocomotionModel, PLAYER_LOCOMOTION_KIND, PLAYER_LOCOMOTION_TICK_HZ } from "../dist/renderer/PlayerLocomotionModel.js";
import { PlayerLocomotionMesh, deformPlayerLocomotionVertices, PLAYER_LOCOMOTION_MAX_X,
  PLAYER_LOCOMOTION_MAX_Y, PLAYER_LOCOMOTION_HIP_Y } from "../dist/renderer/PlayerLocomotionMesh.js";

const point = (x = 0, z = 0, y = 0) => ({ xM: x, yM: y, zM: z });
function input(overrides = {}) {
  return { worldId: "grey_hive", sceneId: "gh_lower_hall", worldEpoch: 3, playerEntityId: "player-1",
    serverTick: 60, position: point(), velocity: point(4), actionState: "idle", alive: true,
    paused: false, loading: false, reducedMotion: false, ...overrides };
}
function moving(model = new PlayerLocomotionModel()) {
  model.project(input());
  return { model, frame: model.project(input({ serverTick: 66, position: point(0.4) })) };
}
function gait(overrides = {}) {
  return { kind: PLAYER_LOCOMOTION_KIND, active: true, phaseRad: Math.PI / 2, amplitude: 1,
    travelX: 1, travelY: 0, reducedMotion: false, ...overrides };
}
const directions = ["south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east"];
const texture = (width = 336, height = 560) => new Texture({ source: new TextureSource({ width, height }) });

// These checks use authoritative samples; input command objects have no role in the model.
test("first samples and blocked movement never invent walking from commands or velocity", () => {
  const model = new PlayerLocomotionModel();
  assert.equal(model.project(input()).active, false);
  assert.equal(model.project(input({ serverTick: 66, command: { forward: true } })).active, false);
  assert.equal(model.project(input({ serverTick: 72, velocity: point() })).active, false);
  assert.equal(model.project(input({ serverTick: 78, position: point(0, 0, 0.1), velocity: point(0, 0, 1) })).active, false);
  const result = model.project(input({ serverTick: 84, position: point(0.4, 0, 0.1) }));
  assert.equal(result.kind, "temporary_visual");
  assert.equal(result.active, true);
  assert.equal(PLAYER_LOCOMOTION_TICK_HZ, 60);
});

test("tick and real distance advance gait; identical snapshot renders stay frozen", () => {
  const { model, frame } = moving();
  assert.ok(frame.phaseRad > 0 && frame.phaseRad < Math.PI);
  assert.equal(frame.amplitude, 1);
  for (let repeat = 0; repeat < 20; repeat++) {
    assert.deepEqual(model.project(input({ serverTick: 66, position: point(0.4) })), frame);
  }
  const next = model.project(input({ serverTick: 72, position: point(0.8) }));
  assert.ok(next.phaseRad > frame.phaseRad);
});

test("stops, every non-idle combat state, death, pause, and loading immediately restore idle geometry", () => {
  const stopped = [{ velocity: point() }, { paused: true }, { loading: true }, { alive: false },
    ...["attack", "dodge", "guard", "parry", "skill_q", "skill_e", "skill_r", "dead", "unknown"].map(actionState => ({ actionState }))];
  for (const change of stopped) {
    const { model } = moving();
    const result = model.project(input({ serverTick: 72, position: point(0.8), ...change }));
    assert.equal(result.active, false, JSON.stringify(change));
    assert.equal(result.amplitude, 0);
    assert.equal(result.phaseRad, 0);
  }
});

test("all exact scene, epoch, entity transitions and explicit reset require a fresh movement baseline", () => {
  for (const change of [{ worldId: "return_station" }, { sceneId: "gh_upper_hall" }, { worldEpoch: 4 }, { playerEntityId: "player-2" }]) {
    const { model } = moving();
    assert.equal(model.project(input({ serverTick: 72, position: point(0.8), ...change })).active, false);
    assert.equal(model.project(input({ serverTick: 78, position: point(1.2), ...change })).active, true);
  }
  const { model } = moving();
  model.reset();
  assert.equal(model.project(input({ serverTick: 72, position: point(0.8) })).active, false);
});

test("teleports, clock reversals, contradictory same-tick samples, long gaps and malformed data fail neutral", () => {
  const changes = [
    { serverTick: 72, position: point(9) }, { serverTick: 72, position: point(0.8, 0, 3) },
    { serverTick: 65, position: point(0.45) }, { serverTick: 66, position: point(0.45) },
    { serverTick: 90, position: point(0.5) }, { serverTick: Infinity }, { serverTick: 66.5 },
    { worldEpoch: Number.NaN }, { worldId: "" }, { position: point(Number.NaN) },
    { velocity: point(Infinity) }, { velocity: point(10) }, { position: point(0.8), paused: undefined },
  ];
  for (const change of changes) {
    const { model } = moving();
    const frame = model.project(input({ serverTick: 72, position: point(0.8), ...change }));
    assert.equal(frame.active, false, JSON.stringify(change));
    assert.ok([frame.phaseRad, frame.amplitude, frame.travelX, frame.travelY].every(Number.isFinite));
  }
});

test("travel is projected from actual movement, not authoritative combat facing or aim", () => {
  const a = moving().frame;
  assert.ok(a.travelX > 0 && a.travelY > 0);
  const model = new PlayerLocomotionModel();
  model.project(input({ velocity: point(0, 4) }));
  const b = model.project(input({ serverTick: 66, position: point(0, 0.4), velocity: point(0, 4), facingX: 1, facingZ: 0, aimX: 1, aimZ: 0 }));
  assert.ok(b.travelX < 0 && b.travelY > 0);
  const { model: turning, frame: before } = moving();
  const after = turning.project(input({ serverTick: 67, position: point(0.4 - 4 / 60), velocity: point(-4), facingX: 0, facingZ: 1 }));
  assert.ok(after.active);
  assert.ok(after.phaseRad > before.phaseRad);
  assert.ok(after.travelX > 0, "a one-tick reversal blends the leg swing without flipping combat facing");
});

test("reduced motion is one small fixed stance, with no phase or direction flicker", () => {
  const model = new PlayerLocomotionModel();
  model.project(input({ reducedMotion: true }));
  const first = model.project(input({ serverTick: 66, position: point(0.4), reducedMotion: true }));
  for (let step = 2; step < 20; step++) {
    assert.deepEqual(model.project(input({ serverTick: 60 + step * 6, position: point(step * 0.4), reducedMotion: true })), first);
  }
  assert.equal(first.amplitude, 0.18);
  assert.equal(first.phaseRad, Math.PI / 2);
  assert.equal(model.project(input({ serverTick: 180, position: point(7.6), velocity: point(), reducedMotion: true })).active, false);
});

test("projection copies its baseline and never mutates or adds authoritative game state", () => {
  const model = new PlayerLocomotionModel();
  const original = input();
  const before = structuredClone(original);
  model.project(original);
  assert.deepEqual(original, before);
  original.position.xM = 100;
  const next = input({ serverTick: 66, position: point(0.4) });
  const nextBefore = structuredClone(next);
  assert.equal(model.project(next).active, true);
  assert.deepEqual(next, nextBefore);
});

test("opposite ankles really swing; alternating knee and ankle lifts leave torso exactly stable", () => {
  const rest = new Float32Array([336 * 0.30, 560 * 0.965, 336 * 0.70, 560 * 0.965,
    336 * 0.38, 560 * 0.75, 336 * 0.67, 560 * 0.75, 168, 140, 168, 280]);
  const output = new Float32Array(rest.length);
  deformPlayerLocomotionVertices(rest, output, 336, 560, gait(), "south");
  assert.ok(output[0] > rest[0] + 8, "left ankle advances");
  assert.ok(output[2] < rest[2] - 8, "right ankle retreats");
  const first = output.slice();
  deformPlayerLocomotionVertices(rest, output, 336, 560, gait({ phaseRad: 3 * Math.PI / 2 }), "south");
  assert.ok(output[0] < rest[0] - 8 && output[2] > rest[2] + 8, "opposite half-cycle swaps each leg");
  assert.notDeepEqual(output, first);
  deformPlayerLocomotionVertices(rest, output, 336, 560, gait({ phaseRad: 0 }), "south");
  assert.ok(output[1] < rest[1] - 4 && output[5] < rest[5] - 4, "swing leg bends at knee and lifts its ankle");
  assert.equal(output[3], rest[3], "other ankle stays planted in this half-step");
  assert.deepEqual(output.slice(8), rest.slice(8));
});

test("all eight directions keep full upper half fixed and deformations bounded over complete cycles", () => {
  const art = texture();
  const mesh = new PlayerLocomotionMesh(art);
  const rest = mesh.geometry.positions.slice();
  for (const direction of directions) {
    let moved = false;
    for (let step = 0; step < 24; step++) {
      mesh.applyLocomotion(gait({ phaseRad: step * Math.PI / 12, travelX: Math.SQRT1_2, travelY: Math.SQRT1_2 }), direction);
      const positions = mesh.geometry.positions;
      for (let index = 0; index < rest.length; index += 2) {
        assert.ok(Number.isFinite(positions[index]) && Number.isFinite(positions[index + 1]));
        assert.ok(Math.abs(positions[index] - rest[index]) <= 336 * PLAYER_LOCOMOTION_MAX_X + 1e-4);
        assert.ok(Math.abs(positions[index + 1] - rest[index + 1]) <= 560 * PLAYER_LOCOMOTION_MAX_Y + 1e-4);
        if (rest[index + 1] <= 560 * PLAYER_LOCOMOTION_HIP_Y) {
          assert.equal(positions[index], rest[index]);
          assert.equal(positions[index + 1], rest[index + 1]);
        } else if (positions[index] !== rest[index] || positions[index + 1] !== rest[index + 1]) moved = true;
      }
    }
    assert.ok(moved, direction);
  }
  mesh.destroy();
  art.destroy(true);
});

test("mesh updates preserve the original foot anchor, placement, platform height, scale and sorting", () => {
  const art = texture();
  const anchor = [0.5, 0.98392857];
  const mesh = new PlayerLocomotionMesh(art, anchor);
  assert.match(mesh.label, /temporary_visual/);
  mesh.position.set(420, 271); // The renderer already projects support/platform Y into this point.
  mesh.scale.set(0.35);
  mesh.zIndex = 812;
  const rest = mesh.geometry.positions.slice();
  const placement = { x: mesh.x, y: mesh.y, scaleX: mesh.scale.x, scaleY: mesh.scale.y,
    pivotX: mesh.pivot.x, pivotY: mesh.pivot.y, zIndex: mesh.zIndex };
  assert.equal(mesh.pivot.x, 336 * anchor[0]);
  assert.equal(mesh.pivot.y, 560 * anchor[1]);
  for (const direction of directions) mesh.applyLocomotion(gait(), direction);
  assert.deepEqual({ x: mesh.x, y: mesh.y, scaleX: mesh.scale.x, scaleY: mesh.scale.y,
    pivotX: mesh.pivot.x, pivotY: mesh.pivot.y, zIndex: mesh.zIndex }, placement);
  mesh.applyLocomotion(null, "south");
  assert.deepEqual(mesh.geometry.positions, rest);
  mesh.destroy();
  art.destroy(true);
});

test("every direction and stride stays free of inverted or collapsed mesh triangles", () => {
  const art = texture();
  const mesh = new PlayerLocomotionMesh(art);
  const indices = mesh.geometry.indices;
  let minimumArea = Infinity;
  for (const direction of directions) for (let phase = 0; phase < 48; phase++) for (let travel = 0; travel < 16; travel++) {
    mesh.applyLocomotion(gait({ phaseRad: phase * Math.PI / 24,
      travelX: Math.cos(travel * Math.PI / 8), travelY: Math.sin(travel * Math.PI / 8) }), direction);
    const p = mesh.geometry.positions;
    for (let index = 0; index < indices.length; index += 3) {
      const a = indices[index] * 2, b = indices[index + 1] * 2, c = indices[index + 2] * 2;
      const area = (p[b] - p[a]) * (p[c + 1] - p[a + 1]) - (p[b + 1] - p[a + 1]) * (p[c] - p[a]);
      minimumArea = Math.min(minimumArea, area);
    }
  }
  assert.ok(minimumArea > 0, `every triangle retains its winding and visible area; minimum ${minimumArea}`);
  mesh.destroy(); art.destroy(true);
});

test("same-sized atlas changes keep gait rest geometry and UVs; mesh cleanup preserves shared textures", () => {
  const source = new TextureSource({ width: 1024, height: 1024 });
  const first = new Texture({ source, frame: new Rectangle(2, 2, 336, 560) });
  const second = new Texture({ source, frame: new Rectangle(342, 2, 336, 560) });
  const mesh = new PlayerLocomotionMesh(first);
  const rest = mesh.geometry.positions.slice();
  const uvs = mesh.geometry.uvs.slice();
  const geometry = mesh.geometry;
  const buffers = [...geometry.buffers];
  mesh.applyLocomotion(gait(), "south");
  mesh.setFrame(second, [0.5, 0.98392857]);
  mesh.applyLocomotion(gait(), "north");
  assert.equal(mesh.texture, second);
  assert.deepEqual(mesh.geometry.uvs, uvs);
  mesh.applyLocomotion(null, "north");
  assert.deepEqual(mesh.geometry.positions, rest, "texture turn never accumulates an old pose");
  mesh.destroy();
  mesh.destroy();
  assert.equal(geometry.buffers, null, "private mesh geometry is released");
  assert.ok(buffers.every(buffer => buffer.destroyed), "private mesh buffers are released");
  assert.equal(first.destroyed, false);
  assert.equal(second.destroyed, false);
  assert.equal(source.destroyed, false);
  first.destroy(false); second.destroy(true);
});

test("texture dimension replacement rebuilds neutral geometry and preserves explicit anchor", () => {
  const first = texture();
  const second = texture(168, 280);
  const mesh = new PlayerLocomotionMesh(first);
  mesh.applyLocomotion(gait(), "south");
  mesh.setFrame(second, [0.5, 1]);
  mesh.applyLocomotion(null, "east");
  assert.equal(mesh.pivot.x, 84);
  assert.equal(mesh.pivot.y, 280);
  assert.equal(Math.max(...mesh.geometry.positions.filter((_, index) => index % 2 === 0)), 168);
  assert.equal(Math.max(...mesh.geometry.positions.filter((_, index) => index % 2 === 1)), 280);
  mesh.destroy(); first.destroy(true); second.destroy(true);
});

test("malformed pose values fail to the unchanged idle shape and extreme valid values stay bounded", () => {
  const rest = new Float32Array([100, 400, 200, 500]);
  const output = new Float32Array(rest.length);
  for (const frame of [null, gait({ phaseRad: Infinity }), gait({ travelY: Number.NaN }), gait({ active: false })]) {
    deformPlayerLocomotionVertices(rest, output, 336, 560, frame, "south");
    assert.deepEqual(output, rest);
  }
  deformPlayerLocomotionVertices(rest, output, 336, 560, gait({ amplitude: 1000, travelX: 1000, travelY: -1000 }), "south");
  for (let index = 0; index < rest.length; index++) {
    assert.ok(Number.isFinite(output[index]));
    assert.ok(Math.abs(output[index] - rest[index]) <= (index % 2 ? 560 * PLAYER_LOCOMOTION_MAX_Y : 336 * PLAYER_LOCOMOTION_MAX_X) + 1e-4);
  }
});
