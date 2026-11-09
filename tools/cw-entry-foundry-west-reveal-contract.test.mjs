import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const helper = readFileSync(new URL('../apps/godot_playable/scripts/world/cw_entry_foundry_environment_visual.gd', import.meta.url), 'utf8');
const scene = JSON.parse(readFileSync(new URL('../apps/godot_playable/data/scenes/cw_entry_foundry.json', import.meta.url), 'utf8'));
const wallId = 'cw_entry_foundry_wall_west_north';
const project = ([x, y, z], ppm) => [(x - z) * ppm, ((x + z) / 2 - y) * ppm];
const area = (points) => Math.abs(points.reduce((sum, point, i) => {
  const next = points[(i + 1) % points.length];
  return sum + point[0] * next[1] - next[0] * point[1];
}, 0)) / 2;
const contains = (polygon, point) => {
  let direction = 0;
  for (let i = 0; i < polygon.length; i++) {
    const a = polygon[i], b = polygon[(i + 1) % polygon.length];
    const cross = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
    if (Math.abs(cross) < 1e-9) continue;
    if (direction && Math.sign(cross) !== direction) return false;
    direction = Math.sign(cross);
  }
  return direction !== 0;
};

test('SOURCE_STATIC_GEOMETRY_NOT_GDSCRIPT_OR_VISUAL: west portal reveal contract', () => {
  const sceneWalls = scene.collision.filter((row) => row.id === wallId);
  assert.equal(sceneWalls.length, 1);
  const sceneWall = sceneWalls[0];
  const sourceWall = helper.match(/'cw_entry_foundry_wall_west_north':\{'polygon':(\[\[.*?\]\]),'height':([0-9.]+)/s);
  assert.ok(sourceWall, 'helper must retain the selected canonical wall and height');
  assert.deepEqual(JSON.parse(sourceWall[1]), sceneWall.polygon);
  const height = Number(sourceWall[2]);
  const xs = sceneWall.polygon.map(([x]) => x), zs = sceneWall.polygon.map(([, z]) => z);
  const x0 = Math.min(...xs), x1 = Math.max(...xs), zEnd = Math.max(...zs);
  assert.equal(x1 - x0, 0.5);
  assert.equal(height, 1);
  const vertexSource = helper.match(/^const WEST_NORTH_REVEAL_WORLD = \[([^\]]+)\]$/m);
  assert.ok(vertexSource, 'one fixed world-space reveal quad is required');
  const vertices = [...vertexSource[1].matchAll(/Vector3\(([^)]+)\)/g)].map((m) => m[1].split(',').map(Number));
  assert.deepEqual(vertices, [[x0, 0, zEnd], [x1, 0, zEnd], [x1, height, zEnd], [x0, height, zEnd]]);
  const south = scene.collision.find((row) => row.id === 'cw_entry_foundry_wall_west_south');
  const openingStart = Math.min(...south.polygon.map(([, z]) => z));
  assert.equal(openingStart - zEnd, 3.5);
  assert.ok(vertices.every(([, , z]) => z <= zEnd && z < openingStart));
  for (const ppm of [24, 48, 72]) {
    const screen = vertices.map((point) => project(point, ppm));
    assert.ok(area(screen) > 0, `quad must have area at ${ppm} ppm`);
    assert.ok(contains(screen, project([0.25, 0.5, zEnd], ppm)), `interior point must be covered at ${ppm} ppm`);
  }

  const body = helper.slice(helper.indexOf('func draw_wall('));
  const ready = body.indexOf('if not _draw_ready(canvas,snapshot,scene,current_key,camera,viewport):');
  const row = body.indexOf("if not _same_contract(wall.get('polygon'),info.polygon):");
  const firstDraw = body.indexOf('canvas.draw_colored_polygon(contact_screen');
  assert.ok(ready >= 0 && ready < row && row < firstDraw);
  assert.equal((body.match(/wall_id_value == 'cw_entry_foundry_wall_west_north'/g) ?? []).length, 2);
  for (const text of [
    "canvas.get('snapshot')", "canvas.get('scene')", "canvas.get('current_key')",
    'not canvas_snapshot_value is Dictionary', 'not canvas_scene_value is Dictionary', 'typeof(canvas_key_value)!=TYPE_STRING',
    '_same_contract(canvas_snapshot,snapshot)', '_same_contract(canvas_scene,scene)', 'canvas_key!=current_key'
  ]) assert.ok(body.indexOf(text) >= 0 && body.indexOf(text) < firstDraw, `${text} must guard before any draw`);
  const loop = body.indexOf('for point_value in WEST_NORTH_REVEAL_WORLD');
  const projection = body.indexOf('_project(camera,reveal_world_point)');
  const finite = body.indexOf('if not is_finite(reveal_screen_point.x) or not is_finite(reveal_screen_point.y):');
  assert.ok(loop >= 0 && loop < projection && projection < finite && finite < firstDraw);
  const side = body.indexOf('canvas.draw_colored_polygon(side_screen,Color(0.18,0.22,0.24,1.0))');
  const reveal = body.indexOf('canvas.draw_colored_polygon(west_north_reveal_screen,Color(0.18,0.22,0.24,1.0))');
  const cap = body.indexOf('canvas.draw_colored_polygon(cap_screen,Color(0.25,0.29,0.31,1.0))');
  assert.ok(firstDraw < side && side < reveal && reveal < cap);
  assert.match(body, /if wall_id_value == 'cw_entry_foundry_wall_west_north':\s*\n\s*canvas\.draw_colored_polygon\(west_north_reveal_screen,/);
  assert.equal((helper.match(/^func draw_[^(]+\(/gm) ?? []).length, 2, 'no new public draw entry point');
  assert.doesNotMatch(body, /\b(?:FileAccess|ResourceLoader|Time|delta|preparation_attempted|stone_texture)\b/);
});
