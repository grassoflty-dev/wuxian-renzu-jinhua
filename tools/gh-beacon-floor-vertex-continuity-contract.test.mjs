import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';

const helperPath = 'apps/godot_playable/scripts/world/gh_beacon_environment_visual.gd';
const helper = readFileSync(helperPath);
const source = helper.toString('utf8');
const scene = JSON.parse(readFileSync('apps/godot_playable/data/scenes/gh_beacon.json', 'utf8'));

function smoothstep(a, b, x) {
  const t = Math.max(0, Math.min(1, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
}

function rgb(x, z, delta = 0) {
  const edge = 1 - smoothstep(0, 2, Math.min(x, 24 - x, z, 16 - z));
  const nx = (x - 12.5) / 6;
  const nz = (z - 8) / 3.5;
  const bay = 1 - smoothstep(0, 1, Math.sqrt(nx * nx + nz * nz));
  const value = delta - 0.020 * edge + 0.018 * bay;
  return [39 / 255 + value, 58 / 255 + value, 55 / 255 + value, 1];
}

function closeColor(a, b) {
  assert.equal(a.length, 4);
  for (let i = 0; i < 4; i++) assert.ok(Number.isFinite(a[i]) && Math.abs(a[i] - b[i]) <= 1e-12);
}

function interpolated(a, b, t) {
  return a.map((value, i) => value + (b[i] - value) * t);
}

test('Beacon floor vertex continuity is source-bound and geometry-static', () => {
  const head = execFileSync('git', ['show', `HEAD:${helperPath}`], { encoding: 'buffer' });
  const oldDelta = Buffer.from('\t\t\tvar delta: float = float((column+2*row)%3-1)*0.004\n');
  const flatDelta = Buffer.from('\t\t\tvar delta: float = 0.0\n');
  const oldAt = head.indexOf(oldDelta);
  if (oldAt >= 0) {
    assert.equal(head.indexOf(oldDelta, oldAt + 1), -1, 'baseline has one targeted expression');
    assert.deepEqual(helper, Buffer.concat([head.subarray(0, oldAt), flatDelta, head.subarray(oldAt + oldDelta.length)]));
  } else {
    const flatAt = head.indexOf(flatDelta);
    assert.ok(flatAt >= 0, 'baseline is either the original or already-qualified flat tint');
    assert.equal(head.indexOf(flatDelta, flatAt + 1), -1);
    assert.deepEqual(helper, head, 'after integration, the complete helper remains byte-identical to HEAD');
  }
  assert.equal(helper.includes(13), false, 'the frozen helper is LF-only');

  assert.deepEqual(scene.boundsM, { depth: 16, width: 24, x: 0, z: 0 });
  const floor = scene.presentation.sprites.find(row => row.id === 'beacon_approved_floor');
  assert.deepEqual(floor, { assetId: 'runtime2d.grey_hive.floor_tiles.v1', id: 'beacon_approved_floor', layer: 'visual.floor', position: [12, 0, 8] });
  assert.ok(scene.presentation.layers.some(layer => layer.id === 'visual.floor' && layer.zGroup === 0));

  const start = source.indexOf('func draw_floor(');
  assert.ok(start >= 0, 'actual draw_floor anchor exists');
  const end = source.indexOf('\nfunc draw_wall(', start);
  assert.ok(end > start, 'actual draw_wall terminates the inspected function scope');
  const body = source.slice(start, end);
  const anchors = [
    'if not _canvas(canvas) or not _context(snapshot,scene,current_key,projection,viewport):',
    'var commands: Array[Dictionary] = []',
    "var base: Color = Color('#273a37')",
    'for column: int in range(6):\n\t\tfor row: int in range(4):',
    'var corners: Array = [[x,z],[x+4,z],[x+4,z+4],[x,z+4]]',
    'var delta: float = 0.0',
    'var edge: float = 1.0-smoothstep(0.0,2.0,minf(minf(px,24.0-px),minf(pz,16.0-pz)))',
    'var nx: float = (px-12.5)/6.0',
    'var nz: float = (pz-8.0)/3.5',
    'var bay: float = 1.0-smoothstep(0.0,1.0,sqrt(nx*nx+nz*nz))',
    'var value: float = delta-0.020*edge+0.018*bay',
    'colors.append(Color(base.r+value,base.g+value,base.b+value,1))',
    "commands.append({'kind':'polygon','points':_project(projection,corners),'colors':colors})",
    'var width: float = 0.65*float(projection.ppm)/48.0',
    'var joint: PackedColorArray = PackedColorArray([Color(0.05,0.08,0.075,0.10)])',
    'for x: int in [4,8,12,16,20]:',
    "commands.append({'kind':'line','points':_project(projection,[[x,0],[x,16]]),'colors':joint,'width':width})",
    'for z: int in [4,8,12]:',
    "commands.append({'kind':'line','points':_project(projection,[[0,z],[24,z]]),'colors':joint,'width':width})",
    'return _paint(canvas,commands)'
  ];
  const positions = anchors.map(anchor => body.indexOf(anchor));
  assert.ok(positions.every(position => position >= 0), 'all source anchors exist in draw_floor');
  assert.deepEqual([...positions].sort((a, b) => a - b), positions, 'production floor plan keeps its validated cell-then-joint order');
  assert.equal((body.match(/commands\.append\(\{'kind':'polygon'/g) || []).length, 1);
  assert.equal((body.match(/commands\.append\(\{'kind':'line'/g) || []).length, 2);

  const cells = [];
  const vertices = new Map();
  for (let column = 0; column < 6; column++) for (let row = 0; row < 4; row++) {
    const x = column * 4, z = row * 4;
    const corners = [[x, z], [x + 4, z], [x + 4, z + 4], [x, z + 4]];
    cells.push({ column, row, corners });
    for (const [px, pz] of corners) {
      const key = `${px},${pz}`;
      const color = rgb(px, pz);
      assert.ok(color.every(Number.isFinite) && color[3] === 1);
      for (let i = 0; i < 3; i++) assert.ok(color[i] >= 0 && color[i] <= 1);
      if (vertices.has(key)) closeColor(vertices.get(key), color);
      else vertices.set(key, color);
    }
  }
  assert.equal(cells.length, 24);
  assert.equal(cells.reduce((sum, cell) => sum + 16, 0), 384, 'cells tile the canonical 24 by 16 floor');
  assert.equal(cells.reduce((sum, cell) => sum + cell.corners.length, 0), 96);
  assert.equal(vertices.size, 35);

  const samples = [0, 1 / 8, 1 / 4, 1 / 2, 3 / 4, 7 / 8, 1];
  let sharedEdges = 0;
  for (let column = 1; column < 6; column++) for (let row = 0; row < 4; row++) {
    const x = column * 4, z = row * 4;
    const left0 = rgb(x, z), left1 = rgb(x, z + 4);
    const right0 = rgb(x, z), right1 = rgb(x, z + 4);
    for (const t of samples) closeColor(interpolated(left0, left1, t), interpolated(right0, right1, t));
    sharedEdges++;
  }
  for (let row = 1; row < 4; row++) for (let column = 0; column < 6; column++) {
    const x = column * 4, z = row * 4;
    const lower0 = rgb(x, z), lower1 = rgb(x + 4, z);
    const upper0 = rgb(x, z), upper1 = rgb(x + 4, z);
    for (const t of samples) closeColor(interpolated(lower0, lower1, t), interpolated(upper0, upper1, t));
    sharedEdges++;
  }
  assert.equal(sharedEdges, 38);
  const oldDelta = (column, row) => (((column + 2 * row) % 3) - 1) * 0.004;
  const firstJump = Math.abs(rgb(4, 2, oldDelta(0, 0))[0] - rgb(4, 2, oldDelta(1, 0))[0]);
  const secondJump = Math.abs(rgb(12, 2, oldDelta(2, 0))[0] - rgb(12, 2, oldDelta(3, 0))[0]);
  assert.ok(Math.abs(firstJump - 0.004) < 1e-12, 'old checker offset jumps at the x=4 shared edge');
  assert.ok(Math.abs(secondJump - 0.008) < 1e-12, 'old checker offset jumps at the x=12 shared edge');
});
