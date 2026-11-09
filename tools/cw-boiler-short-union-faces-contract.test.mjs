import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const leaf = basename(fileURLToPath(import.meta.url));
const boiler = leaf === 'cw-boiler-short-union-faces-contract.test.mjs';
assert.ok(boiler || leaf === 'gh-lockdown-short-union-faces-contract.test.mjs');
const cfg = boiler ? {
  room: 'cw_boiler_chamber', helper: 'cw_boiler_environment_visual.gd',
  sha: '04b00520e16435d159f1a459016083020fd40d11a2f8086d2f78dcbd0e483b39',
  high: 0.75, low: 0.22, opening: 6, projection: 'ProjectionScript'
} : {
  room: 'gh_lockdown', helper: 'gh_lockdown_environment_visual.gd',
  sha: '718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850',
  high: 0.9, low: 0.18, opening: 6.5, projection: 'PROJECTION_SCRIPT'
};
const world = 'apps/godot_playable/scripts/world/';
function read(path) {
  return readFileSync(new URL('../' + path, import.meta.url));
}
function literal(source, name) {
  const line = source.split(/\r?\n/).find(x => x.startsWith('const ' + name + ':'));
  assert.ok(line, 'Missing literal ' + name);
  return JSON.parse(line.slice(line.indexOf('=') + 1));
}
function methods(source) {
  const starts = [...source.matchAll(/^func ([A-Za-z_]\w*)\(/gm)];
  const result = new Map();
  for (let i = 0; i < starts.length; i++) {
    result.set(starts[i][1],
      source.slice(starts[i].index, starts[i + 1]?.index ?? source.length));
  }
  return result;
}
function fn(map, name) {
  assert.ok(map.has(name), 'Missing method ' + name);
  return map.get(name);
}
function reachable(seed, map) {
  const pending = [seed], seen = new Set(), parts = [];
  while (pending.length) {
    const text = pending.pop();
    parts.push(text);
    const calls = text.replace(/^func [^\n]*\n/gm, '');
    for (const match of calls.matchAll(/\b(_[A-Za-z0-9_]+)\s*\(/g)) {
      if (map.has(match[1]) && !seen.has(match[1])) {
        seen.add(match[1]);
        pending.push(map.get(match[1]));
      }
    }
  }
  return parts.join('\n');
}
function cross(a, b, c) {
  return (b[0] - a[0]) * (c[1] - a[1]) -
    (b[1] - a[1]) * (c[0] - a[0]);
}
function area(q) {
  let sum = 0;
  for (let i = 1; i < q.length - 1; i++) sum += cross(q[0], q[i], q[i + 1]);
  return Math.abs(sum) / 2;
}
function quad(q) {
  if (!Array.isArray(q) || q.length !== 4) return false;
  for (const p of q) {
    if (!Array.isArray(p) || p.length !== 2 || !p.every(Number.isFinite)) return false;
  }
  let sign = 0;
  for (let i = 0; i < 4; i++) {
    const turn = cross(q[i], q[(i + 1) % 4], q[(i + 2) % 4]);
    if (!Number.isFinite(turn) || turn === 0 || (sign && sign * turn <= 0)) return false;
    sign = Math.sign(turn);
  }
  return Number.isFinite(area(q)) && area(q) > 0;
}
function sat(q, rect) {
  if (!quad(q) || rect.length !== 4 || !rect.every(Number.isFinite) ||
      rect[2] <= 0 || rect[3] <= 0) return -1;
  const [x, y, w, h] = rect;
  const corners = [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
  const axes = [[1, 0], [0, 1]];
  for (let i = 0; i < 4; i++) {
    const a = q[i], b = q[(i + 1) % 4];
    axes.push([a[1] - b[1], b[0] - a[0]]);
  }
  let separated = false;
  for (const axis of axes) {
    if (!axis.every(Number.isFinite) || (axis[0] === 0 && axis[1] === 0)) return -1;
    const pa = [], pb = [];
    for (const p of q) pa.push((p[0] - q[0][0]) * axis[0] + (p[1] - q[0][1]) * axis[1]);
    for (const p of corners) pb.push((p[0] - q[0][0]) * axis[0] + (p[1] - q[0][1]) * axis[1]);
    if (!pa.every(Number.isFinite) || !pb.every(Number.isFinite)) return -1;
    const lo = Math.min(...pa), hi = Math.max(...pa);
    const rlo = Math.min(...pb), rhi = Math.max(...pb);
    if (lo >= hi || rlo >= rhi) return -1;
    if (hi <= rlo || rhi <= lo) separated = true;
  }
  return separated ? 0 : 1;
}
function project(p, ppm) {
  return [640 + (p[0] - 12 - p[2] + 8) * ppm,
    400 + ((p[0] - 12 + p[2] - 8) * 0.5 - p[1]) * ppm];
}
function worldQuad(r) {
  return [[r.a[0], r.low, r.a[1]], [r.b[0], r.low, r.b[1]],
    [r.b[0], r.high, r.b[1]], [r.a[0], r.high, r.a[1]]];
}
function bounds(row) {
  const xs = row.polygon.map(p => p[0]), zs = row.polygon.map(p => p[1]);
  return [Math.min(...xs), Math.min(...zs), Math.max(...xs), Math.max(...zs)];
}

test(cfg.room + ' short exposed union faces', () => {
  const source = read(world + cfg.helper).toString('utf8');
  const raw = read('apps/godot_playable/data/scenes/' + cfg.room + '.json');
  const scene = JSON.parse(raw.toString('utf8'));
  assert.equal(createHash('sha256').update(raw).digest('hex'), cfg.sha);
  assert.equal(literal(source, 'SHORT_SCENE_SHA'), cfg.sha);
  assert.equal(scene.collision.length, boiler ? 6 : 7);
  const map = methods(source), draw = fn(map, 'draw_wall');
  const stage = fn(map, '_short_stage'), emit = fn(map, '_short_emit');
  const compact = text => text.replace(/\s+/g, '');
  const equal = compact(fn(map, '_short_equal'));
  assert.ok(equal.includes('P.number(actual,float(expected),float(expected))'));
  const canonical = reachable(fn(map, '_short_canonical'), map);
  if (boiler) {
    assert.deepEqual(literal(source, 'SHORT_CANONICAL_SCENE'), scene);
    assert.ok(canonical.includes('SHORT_CANONICAL_SCENE'));
    assert.ok(compact(draw).includes('0.75ifhigh_wallelse0.22'));
  } else {
    assert.ok(canonical.includes('_canonical_scene') && canonical.includes('_context_ready'));
    const prep = compact(fn(map, 'prepare'));
    assert.ok(prep.includes('_read_exact(SCENE_RESOURCE,SCENE_BYTES,SCENE_SHA256)'));
    assert.ok(prep.includes('_canonical_scene=parsed_scene_dictionary'));
  }
  const bind = reachable(fn(map, '_short_bindings'), map);
  for (const token of ['P.snapshot', 'worldEpoch', 'entryToken', 'is_instance_valid',
    'is_inside_tree', 'is_visible_in_tree', 'is_queued_for_deletion',
    'get_script', cfg.projection, 'initialized', 'get_viewport_rect', 'scene_hash']) {
    assert.ok(bind.includes(token), 'Missing binding ' + token);
  }
  const cb = compact(bind);
  assert.ok(cb.includes('stored_viewport==viewport') && cb.includes('actual_viewport==viewport'));
  const stageReach = reachable(stage, map);
  assert.ok(stageReach.includes('_short_world') && stageReach.includes('_short_quad'));
  assert.ok(stageReach.includes('_short_visible') && stage.includes('_short_capture('));
  assert.ok(compact(stage).includes('all.size()!=5'));
  assert.doesNotMatch(stageReach, /FileAccess|load_png_from_buffer|prepare_resources/);
  const paint = emit.indexOf('canvas.draw_colored_polygon(');
  assert.ok(paint >= 0);
  const beforePaint = reachable(emit.slice(0, paint), map);
  assert.ok(beforePaint.includes('_short_bindings('));
  assert.ok(beforePaint.includes('_short_capture_matches('));
  assert.ok(fn(map, '_short_capture').includes('duplicate(true)'));
  const capture = fn(map, '_short_capture_matches');
  for (let i = 0; i < 9; i++) assert.ok(capture.includes('stamp[' + i + ']'));
  const satSource = fn(map, '_short_visible');
  assert.ok(satSource.includes('separated = true'));
  assert.ok(compact(satSource).includes('return0ifseparatedelse1'));
  assert.equal((satSource.match(/return 0/g) || []).length, 1);
  const first = draw.indexOf('canvas.draw_colored_polygon(');
  const cap = draw.indexOf('canvas.draw_colored_polygon(cap_points');
  assert.ok(draw.indexOf('_short_stage(') < first);
  assert.ok(draw.indexOf('_short_emit(') > first && draw.indexOf('_short_emit(') < cap);
  const hook = draw.split(/\r?\n/).find(x => x.includes('_short_emit('));
  assert.ok(hook.trim().startsWith('_short_emit('));
  if (draw.includes('_draw_alarm_body(')) assert.ok(draw.indexOf('_draw_alarm_body(') > cap);

  const ids = scene.collision.slice(0, 6).map(r => r.id);
  const boxes = scene.collision.slice(0, 6).map(bounds);
  const heights = [cfg.high, cfg.low, cfg.high, cfg.high, cfg.low, cfg.low];
  if (!boiler) {
    const spec = fn(map, '_wall_spec');
    for (let i = 0; i < 6; i++) {
      const start = spec.indexOf('"' + ids[i] + '"');
      assert.ok(start >= 0);
      const end = spec.indexOf('\n\t\t"', start + ids[i].length + 2);
      const block = spec.slice(start, end < 0 ? spec.length : end);
      const value = block.match(/"height"\s*:\s*(\d+(?:\.\d+)?)/);
      assert.ok(value);
      assert.equal(Number(value[1]), heights[i]);
    }
  }
  const [n, s, wn, ws, en] = boxes;
  const expected = [
    {id:'north_east_end',owner:ids[0],a:[n[2],n[1]],b:[n[2],n[3]],low:0,high:cfg.high},
    {id:'south_east_end',owner:ids[1],a:[s[2],s[1]],b:[s[2],s[3]],low:0,high:cfg.low},
    {id:'west_north_jamb',owner:ids[2],a:[wn[0],wn[3]],b:[wn[2],wn[3]],low:0,high:cfg.high},
    {id:'west_south_step',owner:ids[3],a:[ws[0],ws[3]],b:[ws[2],ws[3]],low:cfg.low,high:cfg.high},
    {id:'east_north_jamb',owner:ids[4],a:[en[0],en[3]],b:[en[2],en[3]],low:0,high:cfg.low}
  ];
  const faces = literal(source, 'SHORT_FACES');
  assert.deepEqual(faces, expected);
  function solid(x, y, z) {
    return boxes.some((r, i) => x > r[0] && x < r[2] &&
      z > r[1] && z < r[3] && y > 0 && y < heights[i]);
  }
  for (const face of faces) {
    const x = (face.a[0] + face.b[0]) / 2, z = (face.a[1] + face.b[1]) / 2;
    const y = (face.low + face.high) / 2, dx = face.a[0] === face.b[0] ? 1 : 0;
    const dz = dx ? 0 : 1, e = 1e-6;
    assert.equal(solid(x - dx * e, y, z - dz * e), true);
    assert.equal(solid(x + dx * e, y, z + dz * e), false);
  }
  const square = [[0,0],[1,0],[1,1],[0,1]];
  assert.equal(sat(square, [1,0,1,1]), 0);
  assert.equal(sat(square, [0,0,1,1]), 1);
  assert.equal(sat(square, [0,0,0,1]), -1);
  assert.equal(sat([[0,0],[0,0],[1,1],[0,1]], [0,0,1,1]), -1);
  assert.equal(sat([[0,0],[2,0],[0.5,0.5],[0,2]], [0,0,1,1]), -1);
  assert.equal(sat([[0,0],[Infinity,0],[1,1],[0,1]], [0,0,1,1]), -1);
  for (const ppm of [24,48,72]) {
    let total = 0;
    for (const face of faces) {
      const q = worldQuad(face).map(p => project(p, ppm));
      assert.equal(quad(q), true);
      assert.ok(sat(q, [0,0,1280,800]) >= 0);
      total += area(q);
    }
    assert.ok(Math.abs(total - (1.5 * cfg.high + 0.5 * cfg.low) * ppm * ppm) < 1e-8);
  }
  const point = project([0.25, cfg.high / 2, cfg.opening], 48);
  const expectedPoint = boiler ? [172,52] : [148,60.4];
  for (let i = 0; i < 2; i++) assert.ok(Math.abs(point[i] - expectedPoint[i]) < 1e-9);
});
