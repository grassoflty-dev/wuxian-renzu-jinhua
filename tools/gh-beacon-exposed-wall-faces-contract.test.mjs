import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

// SOURCE_STATIC_MATH_NOT_GDSCRIPT_VISUAL: source structure and projection math only.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const helper = readFileSync(path.join(root, 'apps/godot_playable/scripts/world/gh_beacon_environment_visual.gd'), 'utf8');
const scene = JSON.parse(readFileSync(path.join(root, 'apps/godot_playable/data/scenes/gh_beacon.json'), 'utf8'));

function functionBlock(name) {
  const lines = helper.split(/\r?\n/);
  const start = lines.findIndex((line) => line.startsWith(`func ${name}(`));
  assert.notEqual(start, -1, `missing ${name}`);
  let end = start + 1;
  while (end < lines.length && (lines[end].startsWith('\t') || lines[end] === '')) end += 1;
  return lines.slice(start, end).join('\n');
}

function exposedTable() {
  const start = helper.indexOf('const EXPOSED_FACES = {');
  assert.notEqual(start, -1, 'missing exposed-face table');
  const end = helper.indexOf('\n}', start);
  assert.notEqual(end, -1, 'unterminated exposed-face table');
  const table = new Map();
  for (const line of helper.slice(start, end).split(/\r?\n/).slice(1)) {
    if (!line.trim()) continue;
    const row = line.match(/^\s*'([^']+)':\s*(\[.*\]),?\s*$/);
    assert.ok(row, `malformed table row: ${line}`);
    const points = [...row[2].matchAll(/\[\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\]/g)]
      .map((match) => [Number(match[1]), Number(match[2])]);
    assert.equal(points.length % 2, 0, `unpaired endpoints for ${row[1]}`);
    const segments = [];
    for (let index = 0; index < points.length; index += 2) segments.push([points[index], points[index + 1]]);
    assert.equal(table.has(row[1]), false, `duplicate wall ${row[1]}`);
    table.set(row[1], segments);
  }
  return table;
}

function edgeKey(a, b) {
  return [a, b].map((point) => JSON.stringify(point)).sort().join('|');
}

function project(point, y, ppm) {
  const [x, z] = point;
  return [(x - z - 4) * ppm + 640, ((x + z) / 2 - y - 10) * ppm + 400];
}

function faceQuad(segment, ppm) {
  const relief = 12 * ppm / 48;
  const [a, b] = segment.map((point) => project(point, 0, ppm));
  return [a, b, [b[0], b[1] - relief], [a[0], a[1] - relief]];
}

function twiceArea(points) {
  return points.reduce((area, point, index) => {
    const next = points[(index + 1) % points.length];
    return area + point[0] * next[1] - next[0] * point[1];
  }, 0);
}

test('exposed segments are exactly the seven non-shared Beacon boundary edges', () => {
  const actual = exposedTable();
  const expected = new Map([
    ['beacon_wall_north', [[[24, 0], [24, 0.5]]]],
    ['beacon_wall_south', [[[0, 16], [24, 16]], [[24, 15.5], [24, 16]]]],
    ['beacon_wall_west_north', [[[0, 6.5], [0.5, 6.5]]]],
    ['beacon_wall_west_south', []],
    ['beacon_wall_east_north', [[[24, 0.5], [24, 6.5]], [[23.5, 6.5], [24, 6.5]]]],
    ['beacon_wall_east_south', [[[24, 9.5], [24, 15.5]]]],
  ]);
  assert.deepEqual([...actual], [...expected]);
  const counts = new Map();
  for (const wall of scene.collision) {
    for (let index = 0; index < wall.polygon.length; index += 1) {
      const key = edgeKey(wall.polygon[index], wall.polygon[(index + 1) % wall.polygon.length]);
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }
  let total = 0;
  for (const segments of actual.values()) {
    for (const [a, b] of segments) {
      total += 1;
      assert.equal(counts.get(edgeKey(a, b)), 1, `segment is absent or shared: ${JSON.stringify([a, b])}`);
    }
  }
  assert.equal(actual.size, 6);
  assert.equal(total, 7);
});

test('each exposed face has positive finite area at the supported ppm limits', () => {
  for (const segments of exposedTable().values()) {
    for (const segment of segments) {
      for (const ppm of [24, 48, 96, 256]) {
        const quad = faceQuad(segment, ppm);
        assert.equal(quad.length, 4);
        assert.ok(quad.flat().every(Number.isFinite));
        const area = Math.abs(twiceArea(quad));
        assert.ok(Number.isFinite(area) && area > 0, `${JSON.stringify(segment)} collapsed at ${ppm} ppm`);
      }
    }
  }
  const southMidpoint = project([12, 16], 0.125, 48);
  assert.deepEqual(southMidpoint, [256, 586]);
  const southFace = faceQuad(exposedTable().get('beacon_wall_south')[0], 48);
  const faceCenter = [0, 1].map((axis) => southFace.reduce((sum, point) => sum + point[axis], 0) / 4);
  assert.deepEqual(faceCenter, southMidpoint);
  assert.ok(southMidpoint[1] > project([12, 15.5], 0, 48)[1], 'legacy inner face cannot cover the exposed south midpoint');
});

test('added faces require current canvas bindings and stage before the untouched cap and outline', () => {
  const binding = functionBlock('_faces_bound');
  const reads = [...binding.matchAll(/var (actual_\w+): Variant = canvas\.get\('([^']+)'\)/g)]
    .map((match) => [match[1], match[2]]);
  assert.deepEqual(reads, [
    ['actual_snapshot', 'snapshot'], ['actual_scene', 'scene'], ['actual_key', 'current_key'],
    ['actual_camera', 'camera'], ['actual_viewport', 'viewport_rect'],
  ]);
  for (const condition of [
    'actual_snapshot is Dictionary and actual_snapshot == snapshot',
    'actual_scene is Dictionary and actual_scene == scene',
    'typeof(actual_key)==TYPE_STRING and actual_key==key',
    'actual_camera is RefCounted and actual_camera==projection',
    'typeof(actual_viewport)==TYPE_RECT2 and actual_viewport==viewport',
  ]) assert.ok(binding.includes(condition), `missing binding condition: ${condition}`);

  const wall = functionBlock('draw_wall');
  const commandBlock = wall.match(/var commands: Array\[Dictionary\] = \[([\s\S]*?)\n\t\]/)?.[1];
  assert.ok(commandBlock, 'missing legacy wall command list');
  const records = commandBlock.split(/\r?\n/).map((line) => line.trim().replace(/,$/, '')).filter(Boolean);
  assert.deepEqual(records, [
    "{'kind':'polygon','points':_project(projection,[[a.x,a.y],[b.x,b.y],[b.x+inside.x,b.y+inside.y],[a.x+inside.x,a.y+inside.y]]),'colors':PackedColorArray([Color(0.02,0.06,0.045,0.12)])}",
    "{'kind':'polygon','points':face,'colors':PackedColorArray([Color('#26352f')])}",
    "{'kind':'polygon','points':cap,'colors':PackedColorArray([Color('#35483e')])}",
    "{'kind':'line','points':outline,'colors':PackedColorArray([Color(0.42745098,0.50980392,0.46666667,0.35)]),'width':float(projection.ppm)/48.0}",
  ]);
  const bindAt = wall.indexOf('if _faces_bound(canvas,snapshot,scene,current_key,projection,viewport):');
  const insertAt = wall.indexOf('commands.insert(commands.size()-2,');
  const paintAt = wall.indexOf('return _paint(canvas,commands)');
  assert.ok(bindAt > wall.indexOf('var commands: Array[Dictionary] = [') && insertAt > bindAt && paintAt > insertAt);
  assert.equal((wall.match(/_paint\(canvas,commands\)/g) ?? []).length, 1);
  assert.ok(wall.includes("Color('#26352f')"));
});
