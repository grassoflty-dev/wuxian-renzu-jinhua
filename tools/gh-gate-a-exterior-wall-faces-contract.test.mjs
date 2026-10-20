import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const helper = readFileSync(new URL('../apps/godot_playable/scripts/world/gh_gate_a_environment_visual.gd', import.meta.url), 'utf8');
const scene = JSON.parse(readFileSync(new URL('../apps/godot_playable/data/scenes/gh_gate_a.json', import.meta.url), 'utf8'));
const expected = [
  { id: 'gh_gate_wall_south', indices: [3, 2], points: [[0, 14], [20, 14]] },
  { id: 'gh_gate_wall_east_north', indices: [1, 2], points: [[20, 0.5], [20, 5.5]] },
  { id: 'gh_gate_wall_east_south', indices: [1, 2], points: [[20, 8.5], [20, 13.5]] },
];

function functionSource(name) {
  const start = helper.indexOf(`func ${name}(`);
  assert.notEqual(start, -1, `missing production function ${name}`);
  const next = helper.indexOf('\nfunc ', start + 1);
  return helper.slice(start, next < 0 ? undefined : next);
}

function edgeTable() {
  const table = helper.match(/const EXTERIOR_WALL_EDGES: Dictionary = \{([\s\S]*?)\n\}/);
  assert.ok(table, 'missing production exterior-edge table');
  const entries = new Map();
  const rowPattern = /^\s*'([^']+)': \[\[([-\d.]+),([-\d.]+)\],\[([-\d.]+),([-\d.]+)\]\],?\s*$/gm;
  for (const match of table[1].matchAll(rowPattern)) {
    assert.equal(entries.has(match[1]), false, `duplicate edge ${match[1]}`);
    entries.set(match[1], [[Number(match[2]), Number(match[3])], [Number(match[4]), Number(match[5])]]);
  }
  return entries;
}

function project(point, ppm) {
  const [x, y, z] = point;
  return [800 + ((x - 10) - (z - 7)) * ppm, 500 + (((x - 10) + (z - 7)) * 0.5 - y) * ppm];
}

const cross = (a, b) => a[0] * b[1] - a[1] * b[0];
const subtract = (a, b) => [a[0] - b[0], a[1] - b[1]];

test('the three added edges match only their canonical Gate A collision boundaries', () => {
  const edges = edgeTable();
  assert.equal(edges.size, expected.length);
  assert.ok(Array.isArray(scene.collision), 'canonical collision rows are required');
  for (const item of expected) {
    assert.ok(edges.has(item.id), `missing edge ${item.id}`);
    const actual = edges.get(item.id);
    assert.deepEqual(actual, item.points);
    const rows = scene.collision.filter((row) => row.id === item.id);
    assert.equal(rows.length, 1, `canonical wall ${item.id} must be unique`);
    assert.ok(Array.isArray(rows[0].polygon), `canonical polygon missing for ${item.id}`);
    assert.deepEqual(item.indices.map((index) => rows[0].polygon[index]), item.points);
  }
});

test('projected faces remain nondegenerate and cover the south-face counterexample', () => {
  const edges = edgeTable();
  for (const ppm of [24, 48, 96, 256]) {
    for (const item of expected) {
      const ground = edges.get(item.id).map(([x, z]) => project([x, 0, z], ppm));
      const relief = 12 * ppm / 48;
      const quad = [ground[0], ground[1], [ground[1][0], ground[1][1] - relief], [ground[0][0], ground[0][1] - relief]];
      const area = cross(subtract(quad[1], quad[0]), subtract(quad[3], quad[0]));
      assert.ok(Number.isFinite(area) && area !== 0, `${item.id} collapsed at ${ppm} ppm`);
    }
  }
  const south = edges.get('gh_gate_wall_south');
  const ground = south.map(([x, z]) => project([x, 0, z], 48));
  const quad = [ground[0], ground[1], [ground[1][0], ground[1][1] - 12], [ground[0][0], ground[0][1] - 12]];
  const point = project([10, 0.125, 14], 48);
  const axisA = subtract(quad[1], quad[0]);
  const axisB = subtract(quad[3], quad[0]);
  const relative = subtract(point, quad[0]);
  const determinant = cross(axisA, axisB);
  const along = cross(relative, axisB) / determinant;
  const height = cross(axisA, relative) / determinant;
  assert.ok(along > 0 && along < 1 && height > 0 && height < 1, 'south point must lie inside the added face');
});

test('addition is bound to the current canvas and fully validated before insertion', () => {
  const binding = functionSource('_addition_binding_matches');
  for (const token of [
    'P.snapshot(snapshot)', "snapshot.get('entryToken') != null", 'canvas_snapshot == snapshot',
    'canvas_scene == scene', 'canvas_key == current_key', 'canvas_camera == camera',
    'canvas_viewport == viewport', 'canvas.get_viewport_rect() == viewport',
    'camera.viewport == viewport.size', 'typeof(initialized) == TYPE_BOOL', 'initialized == true',
  ]) assert.ok(binding.includes(token), `missing current-canvas binding: ${token}`);

  const addition = functionSource('_try_add_exterior_face');
  const required = [
    "not EXTERIOR_WALL_EDGES.has(wall_id)", '_addition_binding_matches(',
    'return -1', 'Vector2(0.0,12.0*camera.ppm/48.0)',
    'PackedVector2Array([ground_a,ground_b,ground_b-relief,ground_a-relief])',
    'if not _v2(point)', 'if not is_finite(area)', 'if area == 0.0',
    'Color(0.18,0.25,0.29,1)', '_append(staged,quad,color,true)',
    'if staged.size() != 1', 'plan.insert(1,staged[0])', 'return 1',
  ];
  const positions = required.map((token) => {
    const index = addition.indexOf(token);
    assert.notEqual(index, -1, `missing staged-addition guard: ${token}`);
    return index;
  });
  assert.ok(positions.every((position, index) => index === 0 || positions[index - 1] < position), 'addition must validate before insertion');
  const nonTargetAt = addition.indexOf('if typeof(wall_id) != TYPE_STRING or not EXTERIOR_WALL_EDGES.has(wall_id):');
  const bindingAt = addition.indexOf('_addition_binding_matches(');
  const nonTargetReturn = addition.indexOf('return 0', nonTargetAt);
  const bindingReturn = addition.indexOf('return 0', bindingAt);
  assert.ok(nonTargetAt >= 0 && nonTargetReturn < bindingAt && bindingAt < bindingReturn, 'nontarget and binding mismatch must skip only the addition');
  const zeroAt = addition.indexOf('if area == 0.0');
  const zeroReturn = addition.indexOf('return 0', zeroAt);
  const stagedAt = addition.indexOf('_append(staged,quad,color,true)');
  assert.ok(zeroAt >= 0 && zeroReturn > zeroAt && zeroReturn < stagedAt, 'exact-zero area must omit only the staged face');

  const legacyBuild = functionSource('_build');
  assert.ok(legacyBuild.includes('visible_ground_b'), 'legacy cap-r2 clipping must remain in _build');
  assert.ok(legacyBuild.includes("plan[0]['visibility_points']"), 'legacy visibility metadata must remain attached');
  assert.equal(legacyBuild.includes('EXTERIOR_WALL_EDGES'), false, 'legacy plan builder must stay unchanged by the addition');

  const visible = functionSource('_plan_visible');
  assert.match(visible, /extra_count: int = 0/);
  assert.ok(visible.includes('extra_count < 0 or extra_count > 1'));
  assert.ok(visible.includes('plan.size() > 11+extra_count'));
  const draw = functionSource('draw_wall');
  const buildAt = draw.indexOf('_build(');
  const addAt = draw.indexOf('_try_add_exterior_face(');
  const invalidAt = draw.indexOf('if extra_count < 0:');
  const validateAt = draw.indexOf('_plan_visible(plan,viewport,extra_count)');
  const firstPaintAt = draw.indexOf('canvas.draw_colored_polygon');
  assert.ok(buildAt >= 0 && addAt >= 0 && invalidAt >= 0 && validateAt >= 0 && firstPaintAt >= 0, 'plan, addition, rejection, validation and emission anchors must exist');
  assert.ok(buildAt < addAt && addAt < invalidAt && invalidAt < validateAt && validateAt < firstPaintAt, 'complete plan validation must precede first paint');
  assert.match(draw.slice(invalidAt, validateAt), /if extra_count < 0:\s+return false/);
});
