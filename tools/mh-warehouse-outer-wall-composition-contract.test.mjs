import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const helper = readFileSync(new URL('../apps/godot_playable/scripts/world/mh_tidal_warehouse_environment_visual.gd', import.meta.url), 'utf8');
const scene = JSON.parse(readFileSync(new URL('../apps/godot_playable/data/scenes/mh_tidal_warehouse.json', import.meta.url), 'utf8'));

function functionSource(name) {
  const start = helper.indexOf(`func ${name}(`);
  assert.notEqual(start, -1, `missing function ${name}`);
  const end = helper.indexOf('\nfunc ', start + 1);
  return helper.slice(start, end < 0 ? helper.length : end);
}

const expected = [
  ['warehouse_wall_north', 24, 0, 24, 0.5, 0, 0.8],
  ['warehouse_wall_south', 0, 16, 24, 16, 0, 0.18],
  ['warehouse_wall_south', 24, 15.5, 24, 16, 0, 0.18],
  ['warehouse_wall_west_north', 0, 6, 0.5, 6, 0, 0.8],
  ['warehouse_wall_west_south', 0, 15.5, 0.5, 15.5, 0.18, 0.8],
  ['warehouse_wall_east_north', 24, 0.5, 24, 6, 0, 0.18],
  ['warehouse_wall_east_north', 23.5, 6, 24, 6, 0, 0.18],
  ['warehouse_wall_east_south', 24, 9.5, 24, 15.5, 0, 0.18],
];

function readFaceTable() {
  const block = helper.match(/const OUTER_WALL_FACES: Array = \[([\s\S]*?)\n\]/)?.[1];
  assert.ok(block, 'outer-face table exists');
  return [...block.matchAll(/\{'owner':'([^']+)','a':Vector2\(([-\d.]+),([-\d.]+)\),'b':Vector2\(([-\d.]+),([-\d.]+)\),'low':([-\d.]+),'high':([-\d.]+)\}/g)]
    .map((m) => [m[1], ...m.slice(2).map(Number)]);
}

function project(point, ppm) {
  const [x, y, z] = point;
  return [(x - z) * ppm, ((x + z) / 2 - y) * ppm];
}

function cross(a, b) { return a[0] * b[1] - a[1] * b[0]; }
function subtract(a, b) { return [a[0] - b[0], a[1] - b[1]]; }

function strictQuad(points) {
  if (points.length !== 4 || points.flat().some((n) => !Number.isFinite(n))) return false;
  const origin = points[0];
  const area2 = cross(subtract(points[1], origin), subtract(points[2], origin)) + cross(subtract(points[2], origin), subtract(points[3], origin));
  if (!Number.isFinite(area2) || area2 === 0) return false;
  const turns = points.map((point, i) => cross(subtract(points[(i + 1) % 4], point), subtract(points[(i + 2) % 4], points[(i + 1) % 4])));
  return turns.every((turn) => Number.isFinite(turn) && turn !== 0 && Math.sign(turn) === Math.sign(area2));
}

function contains(poly, point) {
  const signs = poly.map((v, i) => cross(subtract(poly[(i + 1) % poly.length], v), subtract(point, v)));
  return signs.every(Number.isFinite) && (signs.every((v) => v >= 0) || signs.every((v) => v <= 0));
}

function sat(points, rect) {
  const axes = [[1, 0], [0, 1]];
  for (let i = 0; i < 4; i++) {
    const edge = subtract(points[(i + 1) % 4], points[i]);
    axes.push([-edge[1], edge[0]]);
  }
  const [x0, y0, x1, y1] = rect;
  const corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
  let separated = false;
  for (const axis of axes) {
    const origin = points[0];
    const p = points.map((v) => (v[0] - origin[0]) * axis[0] + (v[1] - origin[1]) * axis[1]);
    const r = corners.map((v) => (v[0] - origin[0]) * axis[0] + (v[1] - origin[1]) * axis[1]);
    if ([...p, ...r].some((n) => !Number.isFinite(n))) return -1;
    if (Math.max(...p) <= Math.min(...r) || Math.max(...r) <= Math.min(...p)) separated = true;
  }
  return separated ? 0 : 1;
}

test('the data-only canonical literal and eight exposed edges match the room', () => {
  const literal = helper.match(/^const OUTER_CANONICAL_SCENE: Dictionary = (.+)$/m)?.[1];
  assert.ok(literal, 'full canonical Dictionary literal exists');
  assert.deepEqual(JSON.parse(literal), scene);
  const faces = readFaceTable();
  assert.deepEqual(faces, expected);
  const walls = new Map(scene.collision.map((wall) => [wall.id, wall]));
  for (const [owner] of faces) assert.ok(walls.has(owner), `canonical owner ${owner}`);
  assert.equal(faces.length, 8);
  assert.equal(faces.reduce((sum, face) => sum + 4, 0), 32);
  for (const [owner, ax, az, bx, bz, low, high] of faces) {
    assert.ok(high > low, 'authored low/high endpoints remain ordered');
    assert.ok([ax, az, bx, bz, low, high].every(Number.isFinite));
    if ((ax === 24 || ax === 23.5) && ax === bx) assert.ok(bz <= 6 || az >= 9.5, 'east opening remains clear');
    if (owner.includes('west')) assert.ok(bz <= 6 || az >= 9.5, 'west opening remains clear');
  }
});

test('all authored quads stay strict and the two source counterexamples are covered', () => {
  const faces = readFaceTable();
  for (const face of faces) {
    const [, ax, az, bx, bz, low, high] = face;
    for (const ppm of [24, 48, 96, 256]) {
      const points = [[ax, low, az], [bx, low, bz], [bx, high, bz], [ax, high, az]].map((p) => project(p, ppm));
      assert.ok(strictQuad(points), `${face[0]} at ${ppm} ppm`);
    }
  }
  const east = faces.find((f) => f[0] === 'warehouse_wall_east_north' && f[1] === 24 && f[2] === 0.5);
  const westStep = faces.find((f) => f[0] === 'warehouse_wall_west_south');
  assert.ok(east && westStep);
  const eastQuad = [[24, 0, 0.5], [24, 0, 6], [24, 0.18, 6], [24, 0.18, 0.5]].map((p) => project(p, 48));
  const westQuad = [[0, 0.18, 15.5], [0.5, 0.18, 15.5], [0.5, 0.8, 15.5], [0, 0.8, 15.5]].map((p) => project(p, 48));
  assert.ok(strictQuad(eastQuad) && strictQuad(westQuad));
  const eastPoint = project([24, 0.09, 3], 48);
  const westPoint = project([0.25, 0.49, 15.5], 48);
  assert.ok(contains(eastQuad, eastPoint) && contains(westQuad, westPoint));
  const maxX = Math.max(...eastQuad.map((p) => p[0]));
  const minY = Math.min(...eastQuad.map((p) => p[1]));
  const maxY = Math.max(...eastQuad.map((p) => p[1]));
  assert.equal(sat(eastQuad, [eastPoint[0] - 0.1, eastPoint[1] - 0.1, eastPoint[0] + 0.1, eastPoint[1] + 0.1]), 1);
  assert.equal(sat(eastQuad, [maxX, minY, maxX + 1, maxY]), 0, 'edge contact is not positive overlap');
  assert.equal(sat(eastQuad, [maxX + 1, minY, maxX + 2, maxY]), 0, 'separated viewport rectangle');
});

test('optional staging is bound, transactional, fully validated, and between side and cap', () => {
  const context = functionSource('_outer_canvas_context');
  for (const field of ['snapshot', 'scene', 'current_key', 'scene_hash', 'camera', 'viewport_rect']) assert.ok(context.includes(`canvas.get('${field}')`), `canvas binding ${field}`);
  for (const binding of ['_same(bound_snapshot,snapshot)', '_same(bound_scene,scene)', 'bound_key != key_value', 'bound_hash != OUTER_SCENE_SHA256', 'bound_camera != camera_value', 'bound_viewport != viewport_value']) assert.ok(context.includes(binding), `binding equality ${binding}`);
  assert.ok(context.includes('canvas.get_viewport_rect()'));
  assert.ok(context.includes('snapshot.get(\'entryToken\') != null'));
  assert.ok(context.includes('canvas_value.is_visible_in_tree()'));
  assert.ok(context.includes('_same(scene,OUTER_CANONICAL_SCENE)'));
  assert.ok(context.includes('_projection(camera_value,viewport_value)'));
  assert.ok(!context.includes('FileAccess') && !context.includes('load('));

  const plan = functionSource('_outer_face_plan');
  assert.ok(plan.includes('OUTER_WALL_FACES.size() != 8') && plan.includes('vertices != 32'));
  assert.ok(plan.indexOf('_outer_quad_valid(points)') < plan.indexOf('_outer_sat(points,viewport)'));
  assert.ok(plan.indexOf('_outer_sat(points,viewport)') < plan.indexOf('visible_faces.append'));
  assert.ok(!plan.includes('draw_colored_polygon'));
  const quad = functionSource('_outer_quad_valid');
  assert.ok(quad.includes('points[1]-origin') && quad.includes('area2 == 0.0'));
  assert.ok(!quad.includes('absf(') && quad.includes('turn == 0.0'));
  const visibility = functionSource('_outer_sat');
  assert.ok(visibility.includes('for axis_value: Variant in axes'));
  const separated = visibility.indexOf('if polygon_max <= rect_min');
  const satReturn = visibility.indexOf('return 0 if separated else 1');
  assert.ok(separated >= 0 && separated < satReturn);
  assert.ok(!visibility.slice(separated, satReturn).includes('return'), 'separation still validates remaining axes');

  const draw = functionSource('draw_wall');
  const staged = draw.indexOf('_outer_face_plan(');
  const contact = draw.indexOf('canvas.draw_colored_polygon(contact,');
  const side = draw.indexOf("canvas.draw_colored_polygon(side,Color('#273b40'))");
  const cap = draw.indexOf("canvas.draw_colored_polygon(cap,Color('#506d75'))");
  const afterSide = draw.slice(side + 1);
  const recheck = afterSide.indexOf('_frame_valid(frame_value,snapshot,scene,key,camera,viewport)');
  const bindingRecheck = afterSide.indexOf('_outer_canvas_context(canvas_value,snapshot,scene,key,camera,viewport)');
  const emit = afterSide.indexOf('canvas.draw_colored_polygon(face_value.points,OUTER_WALL_COLOR)');
  assert.ok(staged >= 0 && contact > staged && side > contact && cap > side);
  assert.ok(recheck >= 0 && bindingRecheck > recheck && emit > bindingRecheck && cap - side > emit);
  assert.ok(!draw.slice(contact, cap).includes('return false'), 'optional failure preserves the legacy draw path');
});
