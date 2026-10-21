import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const root = new URL('../', import.meta.url);
const read = p => readFileSync(new URL(p, root), 'utf8');
const source = read('apps/godot_playable/scripts/world/mh_signal_yard_environment_visual.gd');
const scene = JSON.parse(read('apps/godot_playable/data/scenes/mh_signal_yard.json'));
const projection = read('apps/godot_playable/scripts/world/projection.gd');
const LABEL = 'SOURCE_CONTRACT_MODEL_NOT_GDSCRIPT_RUNTIME_OR_VISUAL';

function fn(name, text = source) {
  const m = new RegExp(`^(?:static\\s+)?func\\s+${name}\\s*\\(`, 'm').exec(text);
  assert.ok(m, `missing function ${name}`);
  const from = m.index + m[0].length;
  const next = text.slice(from).search(/^(?:static\s+)?func\s+\w+\s*\(/m);
  return text.slice(m.index, next < 0 ? text.length : from + next);
}
function literal(name) {
  const m = new RegExp(`^const\\s+${name}(?:\\s*:[^=\\n]+)?\\s*=\\s*`, 'm').exec(source);
  assert.ok(m, `missing constant ${name}`);
  const from = m.index + m[0].length, start = source.indexOf('[', from);
  assert.ok(start >= from);
  assert.equal(source.slice(from, start).trim(), '');
  let depth = 0, quote = '', escaped = false, comment = false, text = '';
  for (let i = start; i < source.length; i++) {
    const c = source[i];
    if (comment) { if (c !== '\n') continue; comment = false; }
    if (quote) {
      text += c;
      if (escaped) escaped = false;
      else if (c === '\\') escaped = true;
      else if (c === quote) quote = '';
      continue;
    }
    if (c === '#') { comment = true; continue; }
    if (c === "'" || c === '"') { quote = c; text += c; continue; }
    text += c;
    if (c === '[') depth++;
    if (c === ']' && --depth === 0) return JSON.parse(text);
  }
  assert.fail(`unbalanced array ${name}`);
}
const compact = s => s.replace(/\s+/g, '');
function order(text, ...anchors) {
  let previous = -1;
  for (const anchor of anchors) {
    const at = text.indexOf(anchor);
    assert.ok(at >= 0, `missing anchor ${anchor}`);
    assert.ok(at > previous, `out of order ${anchor}`);
    previous = at;
  }
}
function alias(body, parameter, type) {
  const m = new RegExp(`\\bvar\\s+(\\w+)\\s*:\\s*${type}\\s*=\\s*${parameter}\\b`).exec(body);
  assert.ok(m, `missing typed alias ${parameter}`);
  return m[1];
}
const xz = literal('GROUND_AMBIENT_XZ');
const rgb = literal('GROUND_AMBIENT_RGB');
const alpha = literal('GROUND_AMBIENT_ALPHA');
const depth = literal('GROUND_AMBIENT_DEPTH');
const sub = (a, b) => [a[0] - b[0], a[1] - b[1]];
const cross = (a, b) => a[0] * b[1] - a[1] * b[0];
const dot = (a, b) => a[0] * b[0] + a[1] * b[1];
const finite = p => p.length === 2 && p.every(Number.isFinite);
const close = (a, b) => assert.ok(Math.abs(a - b) <= 2e-7);
const rgba = z => [...rgb, alpha[0] + (alpha[1] - alpha[0]) * (z - depth[0]) / depth[1]].map(Math.fround);
function valid(q) {
  if (q.length !== 4 || !q.every(finite)) return false;
  const area2 = cross(sub(q[1], q[0]), sub(q[2], q[0])) +
    cross(sub(q[2], q[0]), sub(q[3], q[0]));
  if (!Number.isFinite(area2) || area2 === 0) return false;
  const xs = q.map(p => p[0]), ys = q.map(p => p[1]);
  if (!(Math.max(...xs) > Math.min(...xs) && Math.max(...ys) > Math.min(...ys))) return false;
  return q.every((p, i) => {
    const e = sub(q[(i + 1) % 4], p), n = sub(q[(i + 2) % 4], q[(i + 1) % 4]), turn = cross(e, n);
    return finite(e) && e.some(v => v !== 0) && Number.isFinite(turn) &&
      turn !== 0 && (turn > 0) === (area2 > 0);
  });
}
function overlap(q, r) {
  if (!valid(q) || !r.every(Number.isFinite) || r[2] <= r[0] || r[3] <= r[1]) return -1;
  const rc = [[r[0], r[1]], [r[2], r[1]], [r[2], r[3]], [r[0], r[3]]];
  const axes = [[1, 0], [0, 1], ...q.map((p, i) => {
    const e = sub(q[(i + 1) % 4], p); return [-e[1], e[0]];
  })];
  let separated = false;
  for (const axis of axes) {
    if (!finite(axis) || axis.every(v => v === 0)) return -1;
    const a = q.map(p => dot(sub(p, q[0]), axis)), b = rc.map(p => dot(sub(p, q[0]), axis));
    if (![...a, ...b].every(Number.isFinite)) return -1;
    const loA = Math.min(...a), hiA = Math.max(...a), loB = Math.min(...b), hiB = Math.max(...b);
    if (loA >= hiA || loB >= hiB) return -1;
    if (hiA <= loB || hiB <= loA) separated = true;
  }
  return separated ? 0 : 1;
}
const projectionText = compact(fn('project', projection));
const projectionMatch = /Vector2\(\(d\.x-d\.z\)\*ppm,\(d\.x\+d\.z\)\*ppm\*([0-9.]+)-d\.y\*ppm\)/.exec(projectionText);
assert.ok(projectionMatch, 'unsupported projection formula');
const half = Number(projectionMatch[1]);
function project(ppm, float32 = false) {
  return xz.map(([x, z]) => {
    const dx = x - 12, dz = z - 8, p = [640 + (dx - dz) * ppm, 400 + (dx + dz) * ppm * half];
    return float32 ? p.map(Math.fround) : p;
  });
}

test(`${LABEL}: actual canonical quad and independent depth field`, () => {
  assert.equal(scene.worldId, 'mist_harbor');
  assert.equal(scene.sceneId, 'mh_signal_yard');
  const rows = scene.walkablePolygons.filter(r => r.id === 'mh_signal_yard_walkable_area');
  assert.equal(rows.length, 1);
  assert.deepEqual(xz, rows[0].polygon);
  assert.deepEqual(xz, [[0.5, 0.5], [23.5, 0.5], [23.5, 15.5], [0.5, 15.5]]);
  assert.deepEqual(rgb, [0.07, 0.12, 0.16]);
  assert.deepEqual(alpha, [0.01, 0.04]);
  assert.deepEqual(depth, [0.5, 15]);
  const area2 = xz.reduce((sum, p, i) => sum + cross(p, xz[(i + 1) % 4]), 0);
  close(Math.abs(area2) / 2, 345);
  assert.equal((scene.terrainRegions ?? []).some(r => String(r.tag).includes('water')), false);
  for (const z of [0.5, 8, 15.5]) {
    const c = rgba(z);
    c.slice(0, 3).forEach((v, i) => close(v, rgb[i]));
    assert.ok(c.every(Number.isFinite));
    assert.ok(c[3] >= Math.fround(alpha[0]) && c[3] <= Math.fround(alpha[1]));
  }
  close(rgba(0.5)[3], 0.01); close(rgba(8)[3], 0.025); close(rgba(15.5)[3], 0.04);
  const color = compact(fn('_ground_ambient_color'));
  for (const token of ['z-float(GROUND_AMBIENT_DEPTH[0])', '/float(GROUND_AMBIENT_DEPTH[1])', 'lerpf(',
    'GROUND_AMBIENT_RGB', 'GROUND_AMBIENT_ALPHA[0]', 'GROUND_AMBIENT_ALPHA[1]']) assert.ok(color.includes(token));
});

test(`${LABEL}: new-quad projection and strict-overlap model`, () => {
  assert.equal(half, 0.5);
  for (const ppm of [0.001, 24, 48, 96, 256]) {
    assert.ok(valid(project(ppm)));
    assert.ok(valid(project(ppm, true)));
  }
  assert.equal(valid(project(1e-12, true)), false);
  assert.equal(valid([[0, 0], [0, 0], [0, 0], [0, 0]]), false);
  assert.equal(valid([[0, 0], [1, 0], [1, Infinity], [0, 1]]), false);
  const square = (a, b, c, d) => [[a, b], [c, b], [c, d], [a, d]];
  assert.equal(overlap(square(0.1, 0.1, 0.9, 0.9), [0, 0, 1, 1]), 1);
  assert.equal(overlap(square(1, 0, 2, 1), [0, 0, 1, 1]), 0);
  assert.equal(overlap(square(2, 0, 3, 1), [0, 0, 1, 1]), 0);
  const quad = compact(fn('_outer_quad_valid'));
  for (const token of ['points[index]-points[0]', 'cross(', 'is_finite(area2)', 'area2==0.0', 'turn==0.0']) assert.ok(quad.includes(token));
  const sat = compact(fn('_outer_visibility'));
  order(sat, 'foraxis:Vector2inaxes:', 'p_max<=v_min', 'separated=true', 'return0ifseparatedelse1');
});

test(`${LABEL}: explicit native/live binding and deep captured values`, () => {
  const binding = compact(fn('_ground_ambient_binding'));
  order(binding, "has('entryToken')", "get('entryToken')!=null", '_outer_context_matches(');
  assert.ok(binding.includes('get_global_transform()==Transform2D.IDENTITY'));
  assert.ok(binding.includes('get_global_transform_with_canvas()==Transform2D.IDENTITY'));
  const context = fn('_outer_context_matches');
  for (const field of ['snapshot', 'scene', 'current_key', 'scene_hash', 'camera', 'viewport_rect'])
    assert.match(context, new RegExp(`get\\(['"]${field}['"]\\)`));
  for (const token of ['_view(', 'is_visible_in_tree()', 'get_viewport_rect()', 'Vector2.ZERO',
    '_load_outer_scene_once()', '_equal(scene_value,_outer_scene)']) assert.ok(context.includes(token));
  const view = fn('_view'), base = fn('_base');
  for (const token of ['ProjectionScript', 'TYPE_BOOL', 'initialized', 'is_finite', 'size==viewport.size']) assert.ok(view.includes(token));
  assert.ok(base.includes('P.snapshot(') && base.includes("P.counter(snapshot_value.get('worldEpoch'),1)"));
  const cache = compact(fn('_load_outer_scene_once'));
  order(cache, '_read(', '_sha(bytes)', 'to_utf8_buffer()', 'JSON.parse_string(', '_outer_scene=decoded');
  const capture = fn('_ground_ambient_capture');
  const sa = alias(capture, 'snapshot_value', 'Dictionary'), sc = alias(capture, 'scene_value', 'Dictionary');
  assert.ok(compact(capture).includes(`${sa}.duplicate(true)`));
  assert.ok(compact(capture).includes(`${sc}.duplicate(true)`));
  for (const key of ['canvas_id', 'snapshot', 'scene', 'key', 'scene_hash', 'camera', 'origin', 'center', 'ppm',
    'initialized', 'projection_viewport', 'viewport', 'global_transform', 'canvas_transform'])
    assert.match(capture, new RegExp(`['"]${key}['"]\\s*:`));
  const same = fn('_ground_ambient_same'), ca = alias(same, 'canvas_value', 'CanvasItem');
  assert.ok(same.includes('_ground_ambient_binding('));
  for (const key of ['canvas_id', 'key', 'scene_hash', 'camera', 'origin', 'center', 'ppm',
    'initialized', 'projection_viewport', 'viewport', 'global_transform', 'canvas_transform'])
    assert.match(same, new RegExp(`==\\s*capture\\.${key}\\b`));
  for (const field of ['snapshot', 'scene'])
    assert.match(same, new RegExp(`_equal\\(\\s*${ca}\\.get\\(['"]${field}['"]\\)\\s*,\\s*capture\\.${field}\\s*\\)`));
  const guard = compact(fn('_ground_ambient_color_valid'));
  for (const ch of ['r', 'g', 'b', 'a']) assert.ok(compact(fn('_color_valid')).includes(`is_finite(color.${ch})`));
  assert.ok(guard.includes('_color_valid(color)'));
  assert.ok(guard.includes('Color(0,0,0,float(GROUND_AMBIENT_ALPHA[0])).a'));
  assert.ok(guard.includes('Color(0,0,0,float(GROUND_AMBIENT_ALPHA[1])).a'));
  assert.ok(guard.includes('color.a>=low') && guard.includes('color.a<=high'));
});

test(`${LABEL}: complete staging, final recheck and unchanged legacy result`, () => {
  const draw = compact(fn('_ground_ambient_draw'));
  order(draw, 'ifnotlegacy_drawn:', '_ground_ambient_capture(',
    '_equal(GROUND_AMBIENT_XZ,DEFINITION.stone.walkablePolygon)',
    'Vector3(float(pair[0]),0.0,z)', '_ground_ambient_color(', '_ground_ambient_color_valid(', 'colors.append(',
    '_project(', '_outer_quad_valid(', '_outer_visibility(', '_ground_ambient_same(', '.draw_polygon(');
  assert.ok(draw.includes('colors.size()!=4') && draw.includes('visibility!=1'));
  assert.equal((draw.match(/\.draw_polygon\(/g) ?? []).length, 1);
  const floor = fn('draw_floor');
  order(compact(floor), 'forcommand:Dictionaryinplan', 'canvas.draw_polygon(', 'drawn=true',
    '_ground_ambient_draw(canvas,snapshot,scene,key,projection,viewport,drawn)', 'returndrawn');
  assert.match(floor, /^\t_ground_ambient_draw\(/m);
  const names = ['_ground_ambient_binding', '_ground_ambient_capture', '_ground_ambient_same',
    '_ground_ambient_color', '_ground_ambient_color_valid', '_ground_ambient_draw'];
  for (const name of names)
    assert.doesNotMatch(fn(name), /\b(?:FileAccess|Image|ImageTexture|Time)\b|\b(?:load|preload|randf|randi)\s*\(/);
});
