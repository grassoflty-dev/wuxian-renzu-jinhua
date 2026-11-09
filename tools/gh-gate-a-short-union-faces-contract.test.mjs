import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
const helper = 'apps/godot_playable/scripts/world/gh_gate_a_environment_visual.gd';
const canonical = 'apps/godot_playable/data/scenes/gh_gate_a.json';
const expectedSha = '3c62f3c35a2c9f2e935707e7a93c34a8d6f5696ed730b8487c4efe0b6d1ee484';
function read(path) { return readFileSync(new URL('../' + path, import.meta.url)); }
function literal(source, name) {
  for (const line of source.split(/\r?\n/)) {
    if (line.startsWith('const ' + name + ':')) return JSON.parse(line.slice(line.indexOf('=') + 1));
  }
  assert.fail('Missing literal ' + name);
}
function methods(source) {
  const starts = [...source.matchAll(/^func ([A-Za-z_]\w*)\(/gm)];
  const out = new Map();
  for (let i = 0; i < starts.length; i++) {
    const end = i + 1 < starts.length ? starts[i + 1].index : source.length;
    out.set(starts[i][1], source.slice(starts[i].index, end).trim());
  }
  return out;
}
function fn(map, name) { assert.ok(map.has(name), 'Missing method ' + name); return map.get(name); }
function reachable(seed, map) {
  const pending = [seed], seen = new Set(), parts = [];
  while (pending.length > 0) {
    const text = pending.pop();
    parts.push(text);
    const calls = text.replace(/^func [^\n]*\n/gm, '');
    for (const call of calls.matchAll(/\b(_[A-Za-z0-9_]+)\s*\(/g)) {
      const name = call[1];
      if (map.has(name) && !seen.has(name)) { seen.add(name); pending.push(map.get(name)); }
    }
  }
  return parts.join('\n');
}
function legacy(source) {
  let out = source.replace(/\r\n/g, '\n');
  for (const kind of ['DATA', 'FUNCTION', 'HOOK']) {
    const begin = '# GA_SHORT_' + kind + '_BEGIN', end = '# GA_SHORT_' + kind + '_END';
    const start = out.indexOf(begin);
    if (start < 0) continue;
    const finish = out.indexOf(end, start);
    assert.ok(finish > start, 'Unclosed ' + kind);
    const first = out.lastIndexOf('\n', start) + 1;
    const next = out.indexOf('\n', finish);
    out = out.slice(0, first) + out.slice(next < 0 ? out.length : next + 1);
  }
  return out;
}
function cross(a, b, c) { return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]); }
function area(q) {
  let sum = 0;
  for (let i = 1; i < q.length - 1; i++) sum += cross(q[0], q[i], q[i + 1]);
  return Math.abs(sum) * 0.5;
}
function quad(q) {
  if (!Array.isArray(q) || q.length !== 4) return false;
  for (const p of q) if (!Array.isArray(p) || p.length !== 2 || !Number.isFinite(p[0]) || !Number.isFinite(p[1])) return false;
  let sign = 0;
  for (let i = 0; i < 4; i++) {
    const turn = cross(q[i], q[(i + 1) % 4], q[(i + 2) % 4]);
    if (!Number.isFinite(turn) || turn === 0 || (sign !== 0 && sign * turn <= 0)) return false;
    sign = Math.sign(turn);
  }
  return Number.isFinite(area(q)) && area(q) > 0;
}
function sat(q, r) {
  if (!quad(q) || !Array.isArray(r) || r.length !== 4 || !r.every(Number.isFinite) || r[2] <= 0 || r[3] <= 0) return -1;
  const x = r[0], y = r[1], right = x + r[2], bottom = y + r[3];
  if (!Number.isFinite(right) || !Number.isFinite(bottom)) return -1;
  const corners = [[x, y], [right, y], [right, bottom], [x, bottom]];
  const axes = [[1, 0], [0, 1]];
  for (let i = 0; i < 4; i++) axes.push([q[i][1] - q[(i + 1) % 4][1], q[(i + 1) % 4][0] - q[i][0]]);
  let separated = false;
  for (const axis of axes) {
    if (!axis.every(Number.isFinite) || (axis[0] === 0 && axis[1] === 0)) return -1;
    let lo = Infinity, hi = -Infinity, rlo = Infinity, rhi = -Infinity;
    for (const p of q) {
      const d = (p[0] - q[0][0]) * axis[0] + (p[1] - q[0][1]) * axis[1];
      if (!Number.isFinite(d)) return -1;
      lo = Math.min(lo, d); hi = Math.max(hi, d);
    }
    for (const p of corners) {
      const d = (p[0] - q[0][0]) * axis[0] + (p[1] - q[0][1]) * axis[1];
      if (!Number.isFinite(d)) return -1;
      rlo = Math.min(rlo, d); rhi = Math.max(rhi, d);
    }
    if (lo >= hi || rlo >= rhi) return -1;
    if (hi <= rlo || rhi <= lo) separated = true;
  }
  return separated ? 0 : 1;
}
function project(p, ppm) { return [640 + (p[0] - 10 - p[2] + 7) * ppm, 400 + ((p[0] - 10 + p[2] - 7) * 0.5 - p[1]) * ppm]; }
function worldQuad(r) { return [[r.a[0], r.low, r.a[1]], [r.b[0], r.low, r.b[1]], [r.b[0], r.high, r.b[1]], [r.a[0], r.high, r.a[1]]]; }

test('Gate A new short union family and preserved legacy path', () => {
  const source = read(helper).toString('utf8');
  const raw = read(canonical), scene = JSON.parse(raw.toString('utf8'));
  assert.equal(createHash('sha256').update(raw).digest('hex'), expectedSha);
  assert.equal(literal(source, 'SHORT_SCENE_SHA'), expectedSha);
  assert.deepEqual(literal(source, 'SHORT_CANONICAL_SCENE'), scene);
  assert.match(source, /const SHORT_HEIGHT: float = 12\.0\/48\.0/);
  assert.match(source, /const SHORT_SIDE_COLOR: Color = Color\(0\.18,0\.25,0\.29,1\.0\)/);
  assert.equal(scene.collision.length, 7);
  const ids = scene.collision.slice(0, 6).map((wall) => wall.id);
  const expected = [
    { id: 'north_east_end', owner: ids[0], a: [20, 0], b: [20, 0.5], low: 0, high: 0.25 },
    { id: 'south_east_end', owner: ids[1], a: [20, 13.5], b: [20, 14], low: 0, high: 0.25 },
    { id: 'west_north_jamb', owner: ids[2], a: [0, 5.5], b: [0.5, 5.5], low: 0, high: 0.25 },
    { id: 'east_north_jamb', owner: ids[4], a: [19.5, 5.5], b: [20, 5.5], low: 0, high: 0.25 },
  ];
  const faces = literal(source, 'SHORT_FACES');
  assert.deepEqual(faces, expected);
  for (const face of faces) {
    const wall = scene.collision.find((row) => row.id === face.owner);
    assert.ok(wall, 'canonical owner ' + face.owner);
    const edge = wall.polygon.some((a, i, poly) => {
      const b = poly[(i + 1) % poly.length];
      return (JSON.stringify(a) === JSON.stringify(face.a) && JSON.stringify(b) === JSON.stringify(face.b)) ||
        (JSON.stringify(a) === JSON.stringify(face.b) && JSON.stringify(b) === JSON.stringify(face.a));
    });
    assert.ok(edge, 'short face follows its owner boundary');
    const zs = [face.a[1], face.b[1]];
    assert.ok(Math.max(...zs) <= 5.5 || Math.min(...zs) >= 8.5, 'face remains outside the opening');
  }

  const map = methods(source), draw = fn(map, 'draw_wall'), extension = fn(map, '_short_draw');
  const equal = fn(map, '_short_equal');
  assert.match(equal, /typeof\(expected\) == TYPE_INT or typeof\(expected\) == TYPE_FLOAT/);
  assert.match(equal, /P\.number\(actual,float\(expected\),float\(expected\)\)/);
  assert.match(equal, /typeof\(actual\) != typeof\(expected\)/);
  assert.match(equal, /return actual == expected/);
  const bindings = reachable(fn(map, '_short_bindings'), map);
  for (const token of ['P.snapshot', 'entryToken', 'worldEpoch', 'scene_hash', 'get_script', 'ProjectionScript', 'initialized', 'get_viewport_rect', 'is_visible_in_tree', 'is_inside_tree', 'is_queued_for_deletion']) {
    assert.ok(bindings.includes(token), 'Missing binding ' + token);
  }
  assert.ok(bindings.includes('stored_viewport') && bindings.includes('actual_viewport'));
  const bindingBody = fn(map, '_short_bindings').replace(/\s+/g, '');
  for (const token of ['view.position!=Vector2.ZERO', 'stored_viewport==view', 'actual_viewport==view', 'camera_viewport==view.size', 'canvas.get(\'snapshot\')', 'canvas.get(\'scene\')', 'canvas.get(\'current_key\')', 'canvas.get(\'scene_hash\')', 'canvas.get(\'camera\')']) {
    assert.ok(bindingBody.includes(token), 'Missing exact binding ' + token);
  }
  const recheck = reachable(fn(map, '_short_unchanged'), map);
  assert.ok(recheck.includes('_short_bindings('));
  const capture = reachable(fn(map, '_short_capture'), map);
  for (const field of ['snapshot', 'scene', 'key', 'hash', 'canvas', 'camera', 'origin', 'center', 'viewport', 'ppm']) assert.ok(capture.includes(field));
  assert.ok(capture.includes('duplicate(true)'));

  const qsource = reachable(fn(map, '_short_quad'), map);
  assert.match(qsource, /\.cross\s*\(/);
  assert.match(qsource, /-\s*(origin|points\s*\[\s*0\s*\])/);
  assert.match(qsource, /area2\s*==\s*0\.0/);
  assert.match(qsource, /turn\s*==\s*0\.0/);
  assert.match(qsource, /\(turn\s*>\s*0\.0\)\s*!=\s*\(winding\s*>\s*0\.0\)/);
  assert.equal(qsource.includes('_nonzero_polygon_area('), false);
  const visible = fn(map, '_short_visible').replace(/\s+/g, '');
  assert.ok(visible.includes('polygon_max<=view_minorview_max<=polygon_min'));
  assert.ok(visible.includes('foraxis:Vector2inaxes'));
  assert.equal(visible.includes('break'), false);
  const lastReturn = visible.lastIndexOf('return0ifseparatedelse1');
  assert.ok(lastReturn >= 0 && visible.includes('separated=true'));
  assert.equal(visible.slice(0, lastReturn).includes('return0'), false);
  const stage = reachable(fn(map, '_short_stage'), map).replace(/\s+/g, '');
  assert.ok(stage.includes('SHORT_FACES') && stage.includes('all.size()!=4') && stage.includes('_short_visible('));
  assert.ok(stage.includes('_short_unchanged('));
  assert.ok(stage.replace(/\s+/g, '').includes('owner==owner_idandvisibility==1'));
  const composite = reachable(fn(map, '_short_composite'), map);
  assert.ok(composite.includes('duplicate(true)'));
  assert.ok(composite.includes('visibility_points') && composite.includes('_plan_visible('));
  assert.ok(composite.includes('_plan_visible(copy,viewport,extra_count)'));
  assert.match(composite, /insert\s*\(\s*1\s*\+\s*extra_count\s*,/);
  const hook = draw.indexOf('_short_draw('), oldGuard = draw.indexOf('if not _plan_visible(');
  assert.ok(hook >= 0 && oldGuard > hook);
  const paint = extension.indexOf('canvas.draw_colored_polygon(');
  assert.ok(paint >= 0);
  const beforePaint = reachable(extension.slice(0, paint), map);
  assert.ok(beforePaint.includes('_short_unchanged(') && beforePaint.includes('_short_composite('));

  const head = execFileSync('git', ['show', 'HEAD:' + helper], { cwd: root }).toString('utf8');
  const nowLegacy = methods(legacy(source)), headLegacy = methods(legacy(head));
  for (const name of ['_build', '_layout', '_camera', '_plan_visible', '_addition_binding_matches', '_try_add_exterior_face', 'draw_wall']) {
    assert.equal(fn(nowLegacy, name), fn(headLegacy, name), 'Legacy changed: ' + name);
  }

  const square = [[0, 0], [1, 0], [1, 1], [0, 1]];
  assert.equal(sat(square, [1, 0, 1, 1]), 0);
  assert.equal(sat(square, [0, 0, 1, 1]), 1);
  assert.equal(sat(square, [0, 0, 0, 1]), -1);
  assert.equal(sat(square, [1e308, 0, 1e308, 1]), -1);
  assert.equal(sat([[0, 0], [0, 0], [1, 1], [0, 1]], [0, 0, 1, 1]), -1);
  assert.equal(sat([[0, 0], [2, 0], [0.5, 0.5], [0, 2]], [0, 0, 1, 1]), -1);
  assert.equal(sat([[0, 0], [Infinity, 0], [1, 1], [0, 1]], [0, 0, 1, 1]), -1);
  const far = square.map(([x, y]) => [x + 1e12, y + 1e12]);
  assert.ok(quad(far));
  assert.ok(Math.abs(area(far) - 1) < 1e-12);
  for (const ppm of [24, 48, 72]) {
    let total = 0;
    for (const face of faces) {
      const projected = worldQuad(face).map((point) => project(point, ppm));
      assert.ok(quad(projected));
      total += area(projected);
    }
    assert.ok(Math.abs(total - 0.5 * ppm * ppm) < 1e-8);
  }
  const point = project([0.25, 0.0625, 5.5], 48);
  assert.ok(Math.abs(point[0] - 244) < 1e-9 && Math.abs(point[1] - 127) < 1e-9);
  assert.equal(sat(worldQuad(faces[2]).map((p) => project(p, 48)), [243, 126, 2, 2]), 1);
  for (const extra of [0, 1]) {
    const old = [{ id: 'inner' }];
    if (extra === 1) old.push({ id: 'long' });
    old.push({ id: 'cap' }, { id: 'outline' });
    const saved = old.slice(), merged = old.slice(), added = { id: 'new-short' };
    merged.splice(1 + extra, 0, added);
    assert.deepEqual(old, saved);
    assert.equal(merged[1 + extra], added);
    merged.splice(1 + extra, 1);
    assert.deepEqual(merged, old);
  }
});
