import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const helperPath = 'apps/godot_playable/scripts/world/cw_conveyor_environment_visual.gd';
const sourceBytes = readFileSync(path.join(root, helperPath));
const source = sourceBytes.toString('utf8');
const scenePath = 'apps/godot_playable/data/scenes/cw_conveyor_bridge.json';
const sceneBytes = readFileSync(path.join(root, scenePath));
const scene = JSON.parse(sceneBytes.toString('utf8'));

function block(text, name) {
  const start = text.indexOf('func ' + name + '(');
  assert.notEqual(start, -1, 'missing source anchor: ' + name);
  const end = text.indexOf('\nfunc ', start + 1);
  return text.slice(start, end < 0 ? text.length : end);
}

function productionColor(x, z, cellDelta) {
  const smooth = (v) => { const t = Math.max(0, Math.min(1, v)); return t * t * (3 - 2 * t); };
  const edge = 1 - smooth(Math.min(x, 24 - x, z, 16 - z) / 2);
  const bay = 1 - smooth(Math.abs(z - 8) / 6);
  const delta = cellDelta - 0.016 * edge + 0.012 * bay;
  return [0x3b / 255 + delta, 0x38 / 255 + delta, 0x33 / 255 + delta, 1];
}

function lerp(a, b, t) {
  return a.map((v, i) => v + (b[i] - v) * t);
}

function closeColor(a, b) {
  return a.every((v, i) => Number.isFinite(v) && Math.abs(v - b[i]) <= 1e-7);
}

test('the only production delta is the two approved floor-variation lines', () => {
  assert.equal(sceneBytes.length, 7443);
  assert.equal(createHash('sha256').update(sceneBytes).digest('hex'), '845b664b2212c5d9bf55cf72d34c9c69bf0caf345fa90ed9a7173a1df276e0ae');
  assert.equal(scene.worldId, 'clockworks');
  assert.equal(scene.sceneId, 'cw_conveyor_bridge');
  assert.deepEqual(scene.boundsM, {x:0,z:0,width:24,depth:16});
  const head = execFileSync('git', ['show', 'HEAD:' + helperPath], {cwd:root, maxBuffer:2 * 1024 * 1024});
  const headText = head.toString('utf8');
  const oldPair = '\t\t\tvar variation_step: int = ((column+2*row)%3)-1\n\t\t\tvar delta: float = float(variation_step)*0.004\n';
  const newLine = '\t\t\tvar delta: float = 0.0\n';
  if (headText.includes(oldPair)) {
    assert.equal(headText.split(oldPair).length - 1, 1);
    assert.deepEqual(sourceBytes, Buffer.from(headText.replace(oldPair, newLine), 'utf8'));
  } else {
    assert.match(block(headText, '_build_floor_commands'), /var delta: float = 0\.0/);
    assert.doesNotMatch(block(headText, '_build_floor_commands'), /variation_step/);
    assert.deepEqual(sourceBytes, head);
  }
  const floor = block(source, '_build_floor_commands');
  assert.match(floor, /for row: int in range\(4\)/);
  assert.match(floor, /for column: int in range\(6\)/);
  assert.match(floor, /Rect2\(float\(column\*4\),float\(row\*4\),4\.0,4\.0\)/);
  assert.match(floor, /var delta: float = 0\.0/);
  assert.doesNotMatch(floor, /variation_step/);
  const vertexCalls = floor.match(/colors\.append\(_floor_vertex_color\([^)]*\)\)/g) || [];
  assert.deepEqual(vertexCalls, [
    'colors.append(_floor_vertex_color(cell.position.x,cell.position.y,delta))',
    'colors.append(_floor_vertex_color(cell.end.x,cell.position.y,delta))',
    'colors.append(_floor_vertex_color(cell.end.x,cell.end.y,delta))',
    'colors.append(_floor_vertex_color(cell.position.x,cell.end.y,delta))'
  ]);
});

test('the extracted floor color arithmetic is continuous at every shared vertex and edge', () => {
  const color = block(source, '_floor_vertex_color');
  assert.match(color, /minf\(minf\(x,24\.0-x\),minf\(z,16\.0-z\)\)/);
  assert.match(color, /1\.0-_smoothstep01\(edge_distance\/2\.0\)/);
  assert.match(color, /1\.0-_smoothstep01\(absf\(z-8\.0\)\/6\.0\)/);
  assert.match(color, /delta-0\.016\*edge_value\+0\.012\*bay_value/);
  assert.match(color, /Color\('#3b3833'\)/);
  assert.match(color, /base\.r\+value_delta,base\.g\+value_delta,base\.b\+value_delta,1\.0/);
  const smooth = block(source, '_smoothstep01');
  assert.match(smooth, /clampf\(value,0\.0,1\.0\)/);
  assert.match(smooth, /t\*t\*\(3\.0-2\.0\*t\)/);

  const cells = [];
  const instances = new Map();
  for (let row = 0; row < 4; row++) for (let column = 0; column < 6; column++) {
    const x = column * 4, z = row * 4;
    const points = [[x,z],[x+4,z],[x+4,z+4],[x,z+4]];
    const cell = {row,column,points,colors:points.map(([px,pz]) => productionColor(px,pz,0))};
    cells.push(cell);
    points.forEach((point, i) => {
      const key = point.join(',');
      const list = instances.get(key) || [];
      list.push(cell.colors[i]);
      instances.set(key, list);
    });
  }
  assert.equal(cells.length, 24);
  assert.equal(cells.reduce((n, c) => n + c.points.length, 0), 96);
  assert.equal(instances.size, 35);
  for (const colors of instances.values()) for (const value of colors) {
    assert.ok(value.every(Number.isFinite));
    assert.equal(value[3], 1);
    const shift = value[0] - 0x3b / 255;
    assert.ok(shift >= -0.016 - 1e-7 && shift <= 0.012 + 1e-7);
    assert.ok(colors.every((other) => closeColor(value, other)));
  }

  const samples = [0,1/8,1/4,1/2,3/4,7/8,1];
  let vertical = 0, horizontal = 0;
  const cellAt = (row, column) => cells[row * 6 + column];
  for (let row = 0; row < 4; row++) for (let column = 0; column < 5; column++) {
    const left = cellAt(row,column), right = cellAt(row,column+1);
    vertical++;
    for (const t of samples) assert.ok(closeColor(lerp(left.colors[1],left.colors[2],t),lerp(right.colors[0],right.colors[3],t)));
  }
  for (let row = 0; row < 3; row++) for (let column = 0; column < 6; column++) {
    const lower = cellAt(row,column), upper = cellAt(row+1,column);
    horizontal++;
    for (const t of samples) assert.ok(closeColor(lerp(lower.colors[3],lower.colors[2],t),lerp(upper.colors[0],upper.colors[1],t)));
  }
  assert.equal(vertical, 20);
  assert.equal(horizontal, 18);
  assert.equal(vertical + horizontal, 38);

  const old = (column) => (((column + 0) % 3) - 1) * 0.004;
  assert.ok(Math.abs(productionColor(4,4,old(0))[0] - productionColor(4,4,old(1))[0] + 0.004) <= 1e-7);
  assert.ok(Math.abs(productionColor(12,4,old(2))[0] - productionColor(12,4,old(3))[0] - 0.008) <= 1e-7);
  assert.ok(closeColor(cellAt(0,0).colors[2],cellAt(0,1).colors[3]));
  assert.ok(closeColor(cellAt(0,2).colors[2],cellAt(0,3).colors[3]));
});

test('the existing 24 polygons and eight joint commands retain their production order and paint path', () => {
  const floor = block(source, '_build_floor_commands');
  const draw = block(source, 'draw_floor');
  const buildIndex = draw.indexOf('_build_floor_commands(');
  const validateIndex = draw.indexOf('_commands_valid(commands,32,true)');
  const paintIndex = draw.indexOf('_draw_commands(canvas_item,commands)');
  assert.ok(buildIndex >= 0 && validateIndex >= 0 && paintIndex >= 0);
  assert.ok(buildIndex < validateIndex && validateIndex < paintIndex);
  assert.match(floor, /var joint_color: Color = Color\(0\.06,0\.065,0\.065,0\.10\)/);
  assert.match(floor, /var joint_width: float = 0\.6\*ppm\/48\.0/);
  assert.match(floor, /var x_joints: Array\[float\] = \[4\.0,8\.0,12\.0,16\.0,20\.0\]/);
  assert.match(floor, /Vector3\(joint_x,0\.0,0\.0\),Vector3\(joint_x,0\.0,16\.0\),joint_color,joint_width/);
  assert.match(floor, /var z_joints: Array\[float\] = \[4\.0,8\.0,12\.0\]/);
  assert.match(floor, /Vector3\(0\.0,0\.0,joint_z\),Vector3\(24\.0,0\.0,joint_z\),joint_color,joint_width/);
  assert.equal((floor.match(/commands\.append\(\{'kind':'gradient_polygon'/g) || []).length, 1);
  assert.equal((floor.match(/_append_line\(/g) || []).length, 2);
  const polygonIndex = floor.indexOf(\"commands.append({'kind':'gradient_polygon'\");
  const xJointsIndex = floor.indexOf('var x_joints');
  const xLoopIndex = floor.indexOf('for joint_x: float');
  const zJointsIndex = floor.indexOf('var z_joints');
  assert.ok(polygonIndex >= 0 && xJointsIndex >= 0 && xLoopIndex >= 0 && zJointsIndex >= 0);
  assert.ok(polygonIndex < xJointsIndex && xLoopIndex < zJointsIndex);
  assert.match(block(source, '_append_line'), /'kind':'line'.*'width':width/);
});
