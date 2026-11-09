import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const helperPath = path.join(root, 'apps/godot_playable/scripts/world/gh_gate_b_environment_visual.gd');
const scenePath = path.join(root, 'apps/godot_playable/data/scenes/gh_gate_b.json');
const projectionPath = path.join(root, 'apps/godot_playable/scripts/world/projection.gd');
const helper = readFileSync(helperPath, 'utf8');
const scene = JSON.parse(readFileSync(scenePath, 'utf8'));
const projection = readFileSync(projectionPath, 'utf8');

const expected = new Map([
  ['gh_gate_b_wall_south', [3, 2]],
  ['gh_gate_b_wall_east_north', [1, 2]],
  ['gh_gate_b_wall_east_south', [1, 2]],
]);

function signedArea(points) {
  return points.reduce((sum, point, index) => {
    const next = points[(index + 1) % points.length];
    return sum + point[0] * next[1] - next[0] * point[1];
  }, 0);
}

function inside(point, polygon) {
  let result = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const a = polygon[i], b = polygon[j];
    if ((a[1] > point[1]) !== (b[1] > point[1]) &&
        point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]) result = !result;
  }
  return result;
}

test('exposed table contains only the three canonical non-shared outer edges', () => {
  const match = helper.match(/const EXPOSED_FACES: Dictionary = \{([\s\S]*?)\n\}/);
  assert.ok(match, 'EXPOSED_FACES table exists');
  const rows = [...match[1].matchAll(/'([^']+)':\s*\[\[\s*(-?[\d.]+),\s*(-?[\d.]+)\],\s*\[\s*(-?[\d.]+),\s*(-?[\d.]+)\]\]/g)];
  assert.equal(rows.length, expected.size);
  const actual = new Map(rows.map(row => [row[1], [[Number(row[2]), Number(row[3])], [Number(row[4]), Number(row[5])]]]));
  assert.deepEqual([...actual.keys()].sort(), [...expected.keys()].sort());
  const collisions = new Map(scene.collision.map(row => [row.id, row.polygon]));
  for (const [id, indexes] of expected) {
    assert.ok(collisions.has(id), `canonical collision row ${id} exists`);
    const polygon = collisions.get(id);
    assert.deepEqual(actual.get(id), indexes.map(index => polygon[index]));
  }
  assert.equal(actual.has('gh_gate_b_wall_west_south'), false);
});

test('projected faces are finite and nonzero at supported scales; south counterexample is covered', () => {
  assert.ok(projection.includes('func project(p: Vector3)'), 'production projection method exists');
  assert.ok(projection.includes('Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)'), 'test uses the production projection formula');
  const project = ([x, y, z], ppm) => [640 + ((x - 12) - (z - 8)) * ppm, 400 + ((x - 12 + z - 8) * 0.5 - y) * ppm];
  for (const ppm of [24, 48, 96, 256]) {
    for (const [id, indexes] of expected) {
      const polygon = new Map(scene.collision.map(row => [row.id, row.polygon])).get(id);
      const [a, b] = indexes.map(index => polygon[index]);
      const height = 0.2;
      const quad = [[a[0], 0, a[1]], [b[0], 0, b[1]], [b[0], height, b[1]], [a[0], height, a[1]]].map(point => project(point, ppm));
      assert.ok(quad.flat().every(Number.isFinite), `${id} finite at ${ppm} ppm`);
      assert.notEqual(signedArea(quad), 0, `${id} nondegenerate at ${ppm} ppm`);
    }
  }
  const point = project([12, 0.1, 16], 48);
  const south = scene.collision.find(row => row.id === 'gh_gate_b_wall_south').polygon;
  const added = [[south[3][0], 0, south[3][1]], [south[2][0], 0, south[2][1]], [south[2][0], 0.2, south[2][1]], [south[3][0], 0.2, south[3][1]]].map(p => project(p, 48));
  const cap = south.map(([x, z]) => project([x, 0.2, z], 48));
  assert.equal(inside(point, added), true);
  assert.equal(inside(point, cap), false);
});

test('wall bindings gate only the added face and insertion preserves the four legacy records', () => {
  const start = helper.indexOf('func draw_wall(');
  const stageAnchor = helper.indexOf('\nfunc _stage_exposed_face(', start);
  assert.ok(start >= 0 && stageAnchor > start, 'draw_wall and following staging anchor exist');
  const builderStart = helper.indexOf('func _append_wall_commands(');
  const builderEnd = helper.indexOf('\nfunc _draw_commands(', builderStart);
  assert.ok(builderStart >= 0 && builderEnd > builderStart, 'legacy four-command builder anchors exist');
  const builder = helper.slice(builderStart, builderEnd);
  const records = [
    "commands.append({'kind':'solid','points':contact_points,'color':FLOOR_CONTACT})",
    "commands.append({'kind':'solid','points':side_points,'color':WALL_FACE})",
    "commands.append({'kind':'solid','points':cap_points,'color':WALL_CAP})",
    "commands.append({'kind':'polyline','points':outline_points,'color':WALL_OUTLINE,'width':outline_width})",
  ];
  for (const record of records) assert.ok(builder.includes(record), `legacy record remains: ${record}`);
  assert.ok(records.map(record => builder.indexOf(record)).every((index, i, all) => i === 0 || index > all[i - 1]));
  const drawWall = helper.slice(start, stageAnchor);
  const cableBranch = "\tif id_value == 'gh_gate_b_wall_west_north' and cable_visual is RefCounted and is_instance_valid(cable_visual) and cable_visual.get_script() == GateBWestCableVisual and cable_visual.has_method('draw_wall_cable') and receipt_value is Dictionary and receipt_value.is_empty():\n\t\tvar contact_face: Array[Dictionary] = [commands[0],commands[1]]\n\t\t_draw_commands(canvas,contact_face)\n\t\tvar cable_submitted: Variant = cable_visual.call('draw_wall_cable',canvas,snapshot,scene,current_key,projection,wall,viewport)\n\t\tvar cap_outline: Array[Dictionary] = [commands[2],commands[3]]\n\t\t_draw_commands(canvas,cap_outline)\n\t\tif typeof(cable_submitted) == TYPE_BOOL and cable_submitted:\n\t\t\tvar receipt: Dictionary = receipt_value\n\t\t\treceipt['pipe_submitted'] = true\n\t\treturn true";
  assert.ok(drawWall.includes(cableBranch), 'complete legacy cable branch and receipt remain present');
  const extension = drawWall.indexOf('if EXPOSED_FACES.has(id_value)');
  const finalDraw = drawWall.indexOf('\n\t_draw_commands(canvas,commands)', extension);
  assert.ok(extension >= 0 && finalDraw > extension, 'extension is between cable return and final draw');
  for (const field of ["canvas.get('snapshot')", "canvas.get('scene')", "canvas.get('current_key')", "canvas.get('camera')", "canvas.get('viewport_rect')", "projection.get('viewport')"]) {
    assert.ok(drawWall.includes(field), `actual binding ${field} is checked`);
  }
  const bindingGate = drawWall.indexOf('if bindings_match:');
  const stageCall = drawWall.indexOf('_stage_exposed_face(');
  const insert = drawWall.indexOf('commands.insert(1,exposed_command)');
  const combinedCheck = drawWall.indexOf('commands.size() != 5', insert);
  assert.ok(bindingGate >= 0 && stageCall >= 0 && insert >= 0 && combinedCheck >= 0, 'binding, staging, insertion and five-record validation anchors exist');
  assert.ok(extension < bindingGate && bindingGate < stageCall && stageCall < insert && insert < combinedCheck && combinedCheck < finalDraw);
  const stageStart = helper.indexOf('func _stage_exposed_face(');
  assert.ok(stageStart >= 0, 'staging helper exists');
  const stage = helper.slice(stageStart);
  assert.ok(stage.includes('_finite_vec2(screen)') && stage.includes('_finite_color(WALL_FACE)'));
  assert.ok(stage.includes('twice_area == 0.0'), 'zero-area quads are rejected without a small-area threshold');
});
