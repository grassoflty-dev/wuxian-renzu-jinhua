import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const helper = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/gh_power_environment_visual.gd'), 'utf8');
const projection = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/projection.gd'), 'utf8');
const protocol = readFileSync(resolve(root, 'apps/godot_playable/scripts/bridge/protocol.gd'), 'utf8');
const scene = JSON.parse(readFileSync(resolve(root, 'apps/godot_playable/data/scenes/gh_power_room.json'), 'utf8'));
const edges = [
  ['gh_power_wall_south', 3, 2, [0, 14], [20, 14]],
  ['gh_power_wall_east_north', 1, 2, [20, 0.5], [20, 5.5]],
  ['gh_power_wall_east_south', 1, 2, [20, 8.5], [20, 13.5]],
];

test('outer edge table matches only the three canonical exposed boundaries', () => {
  const tableAt = helper.indexOf('const OUTER_WALL_EDGES: Dictionary = {');
  assert.ok(tableAt >= 0, 'production edge table must exist');
  const tableEnd = helper.indexOf('\n}', tableAt);
  assert.ok(tableEnd > tableAt, 'production edge table must close');
  const table = helper.slice(tableAt, tableEnd);
  const rows = [...table.matchAll(/^\s*'([^']+)':\s*\[Vector2\(([-+]?\d+(?:\.\d+)?),([-+]?\d+(?:\.\d+)?)\),Vector2\(([-+]?\d+(?:\.\d+)?),([-+]?\d+(?:\.\d+)?)\)\],?\s*$/gm)];
  const actual = rows.map((m) => [m[1], [Number(m[2]), Number(m[3])], [Number(m[4]), Number(m[5])]]);
  assert.equal(actual.length, 3);
  assert.deepEqual(actual, edges.map(([id, , , a, b]) => [id, a, b]));
  for (const [id, first, second, a, b] of edges) {
    const row = scene.collision.find((candidate) => candidate.id === id);
    assert.ok(row, `canonical collision row ${id} must exist`);
    assert.deepEqual(row.polygon[first], a);
    assert.deepEqual(row.polygon[second], b);
  }
});

test('projected faces remain finite and nondegenerate; south counterexample is covered', () => {
  assert.ok(projection.includes('screen_center+Vector2((d.x-d.z)*ppm'));
  const project = (x, y, z, ppm) => [(x - z) * ppm, ((x + z) * 0.5 - y) * ppm];
  const area = (points) => points.reduce((sum, point, i) => {
    const next = points[(i + 1) % points.length];
    return sum + point[0] * next[1] - point[1] * next[0];
  }, 0);
  for (const ppm of [24, 48, 96, 256]) {
    for (const [, , , a, b] of edges) {
      const ga = project(a[0], 0, a[1], ppm);
      const gb = project(b[0], 0, b[1], ppm);
      const lift = 12 * ppm / 48;
      const quad = [ga, gb, [gb[0], gb[1] - lift], [ga[0], ga[1] - lift]];
      assert.ok(quad.flat().every(Number.isFinite));
      assert.notEqual(area(quad), 0);
    }
  }
  const groundV = project(10, 0, 14, 48)[1];
  const topV = groundV - 12 * 48 / 48;
  const counterexampleV = project(10, 0.125, 14, 48)[1];
  assert.ok(counterexampleV > topV && counterexampleV < groundV);
});

test('additions are context-gated, staged, and inserted after the inner face', () => {
  assert.ok(protocol.includes('static func snapshot(s)'));
  assert.ok(helper.includes("const PROJECTION_SCRIPT = preload('res://scripts/world/projection.gd')"));
  const contextAt = helper.indexOf('func _outer_context_valid');
  const builderAt = helper.indexOf('func _build_outer_wall_packet');
  assert.ok(contextAt >= 0, 'strict photo-context guard must exist');
  assert.ok(builderAt > contextAt, 'outer packet builder must follow its guard');
  const context = helper.slice(contextAt, builderAt);
  for (const anchor of ['P.snapshot(snapshot)', "snapshot.get('entryToken')!=null", '_ensure_layout(scene_data,snapshot,current_key)', 'projection.get_script()!=PROJECTION_SCRIPT', "projection.get('initialized')", "projection.get('origin')", "projection.get('screen_center')", "projection.get('viewport')", "projection.get('ppm')"]) {
    assert.ok(context.includes(anchor), `missing context binding ${anchor}`);
  }
  const loopAt = helper.indexOf('for wall in WALLS');
  const startAt = helper.indexOf('var wall_start: int = packets.size()', loopAt);
  const addAt = helper.indexOf('_add_wall(packets,wall,projection)', loopAt);
  const stageAt = helper.indexOf('_build_outer_wall_packet(scene_data,snapshot,current_key,projection,wall)', loopAt);
  const insertAt = helper.indexOf('packets.insert(wall_start+1,exterior_packet)', loopAt);
  for (const [name, position] of Object.entries({ loopAt, startAt, addAt, stageAt, insertAt })) {
    assert.ok(position >= 0, `${name} source anchor must exist`);
  }
  assert.ok(loopAt < startAt && startAt < addAt && addAt < stageAt && stageAt < insertAt);
  const builder = helper.slice(builderAt).replace(/\r\n/g, '\n');
  assert.match(builder, /if not _outer_context_valid\([^)]*\):\s*return null/);
  const nonfiniteAt = builder.indexOf('if not is_finite(twice_area)');
  const zeroAt = builder.indexOf('if twice_area==0.0');
  const stagingAt = builder.indexOf('var staged_packets: Array = []');
  const pushAt = builder.indexOf('_push_polygon(staged_packets');
  for (const [name, position] of Object.entries({ nonfiniteAt, zeroAt, stagingAt, pushAt })) {
    assert.ok(position >= 0, `${name} validation/staging anchor must exist`);
  }
  assert.ok(nonfiniteAt < zeroAt && zeroAt < stagingAt && stagingAt < pushAt);
  assert.match(builder, /if twice_area==0\.0:\s*return null/);
  assert.doesNotMatch(builder.slice(nonfiniteAt, stagingAt), /[<>]\s*0/);
  assert.ok(builder.includes("':exterior_face','wall_face'"));
  assert.ok(builder.includes('Color(0.18,0.25,0.29,1.0)'));
});
