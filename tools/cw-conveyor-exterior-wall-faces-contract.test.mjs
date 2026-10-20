import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const helperPath = fileURLToPath(new URL('../apps/godot_playable/scripts/world/cw_conveyor_environment_visual.gd', import.meta.url));
const scenePath = fileURLToPath(new URL('../apps/godot_playable/data/scenes/cw_conveyor_bridge.json', import.meta.url));
const helper = readFileSync(helperPath, 'utf8');
const scene = JSON.parse(readFileSync(scenePath, 'utf8'));
const expected = [
  ['cw_conveyor_bridge_wall_south', 3, 2, [0, 16], [24, 16]],
  ['cw_conveyor_bridge_wall_east_north', 1, 2, [24, 0.5], [24, 6]],
  ['cw_conveyor_bridge_wall_east_south', 1, 2, [24, 9.5], [24, 15.5]],
];

function project([x, y, z], ppm, screenRaise = 0) {
  return [(x - z) * ppm, ((x + z) / 2 - y) * ppm - screenRaise];
}

function signedArea(points) {
  return points.reduce((area, point, index) => {
    const next = points[(index + 1) % points.length];
    return area + point[0] * next[1] - next[0] * point[1];
  }, 0);
}

function containsConvex(points, point) {
  const signs = points.map((value, index) => {
    const next = points[(index + 1) % points.length];
    return (next[0] - value[0]) * (point[1] - value[1]) - (next[1] - value[1]) * (point[0] - value[0]);
  });
  return signs.every((value) => value >= -1e-9) || signs.every((value) => value <= 1e-9);
}

test('the declarative edges are exactly the three canonical exterior boundaries', () => {
  const block = helper.match(/const EXTERIOR_WALL_EDGES: Dictionary = \{([\s\S]*?)\n\}/);
  assert.ok(block, 'exterior edge map exists');
  const actual = [...block[1].matchAll(/'([^']+)':\[(\d+),(\d+)\]/g)]
    .map((match) => [match[1], Number(match[2]), Number(match[3])]);
  assert.deepEqual(actual, expected.map(([id, first, second]) => [id, first, second]));
  for (const [id, first, second, start, end] of expected) {
    const wall = scene.collision.find((row) => row.id === id);
    assert.ok(wall, id + ' exists in the canonical scene');
    assert.deepEqual(wall.polygon[first], start, id + ' starts at its exterior edge');
    assert.deepEqual(wall.polygon[second], end, id + ' ends at its exterior edge');
  }
  const north = scene.collision.find((row) => row.id === 'cw_conveyor_bridge_wall_east_north').polygon[2][1];
  const south = scene.collision.find((row) => row.id === 'cw_conveyor_bridge_wall_east_south').polygon[0][1];
  assert.equal(south - north, 3.5, 'the east opening remains z=6..9.5');
});

test('all three projected faces are finite and nondegenerate at representative scales', () => {
  const block = helper.match(/const EXTERIOR_WALL_EDGES: Dictionary = \{([\s\S]*?)\n\}/);
  const indices = new Map([...block[1].matchAll(/'([^']+)':\[(\d+),(\d+)\]/g)]
    .map((match) => [match[1], [Number(match[2]), Number(match[3])]]));
  for (const [id, , , start, end] of expected) {
    const edge = indices.get(id);
    assert.ok(edge, id + ' has edge indices');
    const wall = scene.collision.find((row) => row.id === id);
    assert.deepEqual([wall.polygon[edge[0]], wall.polygon[edge[1]]], [start, end]);
    for (const ppm of [24, 48, 96, 256]) {
      const relief = 12 * ppm / 48;
      const quad = [
        project([start[0], 0, start[1]], ppm),
        project([end[0], 0, end[1]], ppm),
        project([end[0], 0, end[1]], ppm, relief),
        project([start[0], 0, start[1]], ppm, relief),
      ];
      assert.ok(quad.flat().every(Number.isFinite), id + ' ppm ' + ppm);
      const area = signedArea(quad);
      assert.ok(Number.isFinite(area) && area !== 0, id + ' ppm ' + ppm + ' has finite area');
    }
  }
  const south = expected[0];
  const quad = [
    project([south[3][0], 0, south[3][1]], 48),
    project([south[4][0], 0, south[4][1]], 48),
    project([south[4][0], 0, south[4][1]], 48, 12),
    project([south[3][0], 0, south[3][1]], 48, 12),
  ];
  const counterexample = project([12, 0.125, 16], 48);
  assert.deepEqual([counterexample[0] / 48, counterexample[1] / 48], [-4, 13.875]);
  assert.equal(project([12, 0, 16], 48, 12)[1] / 48, 13.75, 'the existing cap reaches only v=13.75 there');
  assert.ok(containsConvex(quad, counterexample), 'the new south face covers the analytical gap');
});

test('the selected bound path validates before adding one command and painting once', () => {
  const start = helper.indexOf('func draw_wall(');
  const end = helper.indexOf('\nfunc draw_conveyor(', start);
  assert.ok(start >= 0 && end > start);
  const draw = helper.slice(start, end);
  assert.deepEqual([...helper.matchAll(/^func draw_(\w+)\(/gm)].map((match) => match[1]), ['floor', 'wall', 'conveyor']);
  assert.match(draw, /_commands_valid\(commands,4,true\)/, 'legacy four-command plan is retained');
  for (const field of ['snapshot', 'scene', 'current_key', 'camera', 'viewport_rect']) {
    assert.ok(draw.includes("canvas_item.get('" + field + "')"), field + ' is bound to the actual canvas');
  }
  assert.match(draw, /actual_snapshot==snapshot_value and actual_scene==scene_value and actual_key_value==current_key_value and actual_camera_value==projection_value and actual_viewport==viewport_value and projection_viewport==actual_viewport\.size/);
  assert.match(draw, /if actual_snapshot_value is Dictionary and actual_scene_value is Dictionary and typeof\(actual_key_value\)==TYPE_STRING and actual_camera_value is RefCounted and typeof\(actual_viewport_value\)==TYPE_RECT2 and typeof\(projection_viewport_value\)==TYPE_VECTOR2:/);
  assert.ok(draw.indexOf('if actual_snapshot_value is Dictionary') < draw.indexOf('var actual_snapshot: Dictionary'));
  assert.ok(draw.indexOf('if actual_snapshot_value is Dictionary') < draw.indexOf('var actual_scene: Dictionary'));
  assert.ok(draw.indexOf('if actual_snapshot_value is Dictionary') < draw.indexOf('var actual_viewport: Rect2'));
  assert.ok(draw.indexOf('if not exterior_indices_value is Array') < draw.indexOf('var exterior_indices: Array'));
  assert.ok(draw.indexOf('if not polygon_value is Array') < draw.indexOf('var polygon: Array'));
  assert.ok(draw.indexOf('if not first_vertex_value is Array') < draw.indexOf('var first_vertex: Array'));
  const insert = draw.indexOf('commands.insert(1,exterior_commands[0])');
  const paint = draw.indexOf('_draw_commands(canvas_item,commands)');
  assert.ok(draw.indexOf('if binding_matches:') < insert && insert < paint);
  assert.ok(draw.indexOf('if not _points_finite(exterior_points)') < draw.indexOf('_commands_valid(exterior_commands,1,false)'));
  assert.ok(draw.indexOf('if not is_finite(twice_area) or twice_area==0.0') < draw.indexOf('var exterior_commands: Array[Dictionary]'));
  assert.match(draw, /12\.0\*float\(ppm_value\)\/48\.0/);
  assert.match(draw, /second_ground-Vector2\(0\.0,relief\),first_ground-Vector2\(0\.0,relief\)/);
  assert.match(draw, /2\.0\*\(second_ground-first_ground\)\.cross\(Vector2\(0\.0,-relief\)\)/);
  assert.match(draw, /if binding_matches:\n\t\t\tif not exterior_indices_value is Array[\s\S]*?\n\t\t\tcommands\.insert\(1,exterior_commands\[0\]\)/);
  assert.ok(draw.indexOf('_commands_valid(exterior_commands,1,false)') < insert);
  assert.ok(insert < draw.indexOf('_commands_valid(commands,5,true)'));
  assert.ok(draw.indexOf('_commands_valid(commands,5,true)') < paint);
  assert.equal([...draw.matchAll(/_draw_commands\(canvas_item,commands\)/g)].length, 1);
  assert.match(draw, /Color\('#303633'\)/);
  assert.doesNotMatch(draw.slice(draw.indexOf('var exterior_indices_value: Variant')), /\b(?:load|preload|Time|Engine|OS|ResourceLoader|FileAccess)\b/);
});

test('the existing wall command builder remains byte-identical', () => {
  const start = helper.indexOf('func _build_wall_commands(');
  const end = helper.indexOf('\nfunc _build_conveyor_commands(', start);
  assert.ok(start >= 0 && end > start);
  const digest = createHash('sha256').update(helper.slice(start, end), 'utf8').digest('hex');
  assert.equal(digest, '214984ae17661f10ba3d90fae485efe73ab387af40cc29f3c3b100e603c94407');
});
