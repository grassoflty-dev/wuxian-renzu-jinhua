import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const path = 'apps/godot_playable/scripts/world/cw_shutdown_exit_environment_visual.gd';
const source = readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

function balanced(text, start) {
  const opens = new Map([['[', ']'], ['(', ')'], ['{', '}']]);
  const closes = new Map([...opens].map(([a, b]) => [b, a]));
  const stack = [];
  let quote = '';
  for (let i = start; i < text.length; i++) {
    const c = text[i];
    if (quote) {
      if (c === '\\') { i++; continue; }
      if (c === quote) quote = '';
      continue;
    }
    if (c === "'" || c === '"') { quote = c; continue; }
    if (c === '#') { while (i < text.length && text[i] !== '\n') i++; continue; }
    if (opens.has(c)) stack.push(c);
    else if (closes.has(c)) {
      assert.equal(stack.pop(), closes.get(c), 'balanced GDScript literal');
      if (stack.length === 0) return i + 1;
    }
  }
  assert.fail('unterminated GDScript literal');
}

function literal(name, opener) {
  const re = new RegExp(`^const\\s+${name}\\b[^=\\n]*=`, 'm');
  const m = re.exec(source); assert(m, `missing ${name}`);
  const start = source.indexOf(opener, m.index + m[0].length); assert(start >= 0);
  return source.slice(start, balanced(source, start));
}

function values(text) {
  return [...text.matchAll(/-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/g)].map(m => Number(m[0]));
}

function body(name) {
  const re = new RegExp(`^func\\s+${name}\\s*\\([^\\n]*\\)[^\\n]*\\n`, 'm');
  const m = re.exec(source); assert(m, `missing ${name}`);
  const start = m.index;
  const next = source.slice(m.index + m[0].length).search(/^func\s+/m);
  return source.slice(start, next < 0 ? source.length : m.index + m[0].length + next);
}

function ordered(text, ...needles) {
  let at = -1;
  for (const needle of needles) {
    const next = text.indexOf(needle, at + 1);
    assert(next > at, `required order: ${needles.join(' -> ')}`);
    at = next;
  }
}

test('Shutdown ground field literals cover the canonical ground and stay affine/subtle', () => {
  const world = literal('GROUND_LIGHT_WORLD', '[');
  const vectors = [...world.matchAll(/Vector3\s*\(([^)]*)\)/g)].map(m => values(m[1]));
  assert.deepEqual(vectors, [[0,0,0],[24,0,0],[24,0,16],[0,0,16]]);
  assert.equal(world.replace(/Vector3\s*\([^)]*\)/g, '').replace(/[\s,\[\]]/g, ''), '');
  const rgbText = literal('GROUND_LIGHT_RGB', '(');
  assert.deepEqual(values(rgbText), [0.32,0.38,0.40,1.0]);
  const alphaText = literal('GROUND_LIGHT_ALPHA', '[');
  const alpha = values(alphaText);
  assert.deepEqual(alpha.map(a => Math.round(a * 100)), [4,3,1,2]);
  assert.equal(alphaText.replace(/-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/g, '').replace(/[\s,\[\]]/g, ''), '');
  assert(alpha.every(a => Number.isFinite(a) && a > 0 && a <= 0.04));
  const units = alpha.map(a => Math.round(a * 100));
  assert.equal(units[0] + units[2], units[1] + units[3]);
  const field = (x,z) => 4 - x/24 - 2*z/16;
  for (const [i,[x,,z]] of vectors.entries()) assert.equal(field(x,z), units[i]);
  for (const [x,z] of [[6,4],[12,8],[18,12]]) {
    const expected = 4 - x/24 - 2*z/16;
    const bilinear = units[0]*(1-x/24)*(1-z/16) + units[1]*(x/24)*(1-z/16) + units[2]*(x/24)*(z/16) + units[3]*(1-x/24)*(z/16);
    assert(Math.abs(bilinear - expected) < 1e-12);
  }
});

test('capture binds the current canonical frame, authentic camera and identity canvas', () => {
  const capture = body('_ground_light_capture');
  for (const token of ["has('entryToken')", "entryToken') != null", '_canvas_matches(', '_context(', 'get_global_transform_with_canvas()', 'Transform2D.IDENTITY', 'duplicate(true)', "get('viewport_rect')", 'get_viewport_rect()']) assert(capture.includes(token), token);
  ordered(capture, '_canvas_matches(', '_context(', 'get_global_transform_with_canvas()', '_ground_light_plan(');
  const match = body('_ground_light_capture_matches');
  for (const token of ["has('entryToken')", "entryToken') != null", '_canvas_matches(', '_context(', "capture.get('snapshot')", "capture.get('scene')", "camera.get('origin')", "camera.get('ppm')", "camera.get('screen_center')", "camera.get('viewport')", 'get_viewport_rect()', 'get_global_transform_with_canvas()', 'Transform2D.IDENTITY']) assert(match.includes(token), token);
});

test('all geometry and colors validate before one rechecked texture-free draw', () => {
  const plan = body('_ground_light_plan');
  assert(plan.includes('_project_world(GROUND_LIGHT_WORLD,camera)'));
  assert(plan.includes('for alpha: float in GROUND_LIGHT_ALPHA'));
  assert.equal((plan.match(/colors\.append\(Color\(/g) || []).length, 1);
  const draw = body('_draw_ground_light_field');
  for (const token of ['world != GROUND_LIGHT_WORLD', 'points.size() != 4', 'colors.size() != 4', 'color.a <= 0.0', 'color.a > 0.04']) assert(draw.includes(token), token);
  ordered(draw, '_ground_light_capture(', 'world.size() != 4', '_valid_polygon(projected)', '_visible(projected,viewport) != 1', 'color.a > 0.04', '_ground_light_capture_matches(', 'canvas.draw_polygon(projected,vertex_colors)');
  assert(draw.includes('Color(GROUND_LIGHT_RGB.r,GROUND_LIGHT_RGB.g,GROUND_LIGHT_RGB.b,GROUND_LIGHT_ALPHA[index]).a'));
  assert.equal((source.match(/\.draw_polygon\s*\(/g) || []).length, 1);
  assert(!/draw_polygon\s*\([^\n]*texture|draw_polygon\s*\([^\n]*shader/i.test(draw));
});

test('ignored optional hook follows legacy floor emission and preserves its success return', () => {
  const floor = body('draw_floor');
  const hook = '_draw_ground_light_field(canvas_value,snapshot_value,scene_value,key_value,scene_hash_value,camera_value,viewport)';
  const emit = floor.lastIndexOf('canvas_value.draw_colored_polygon(command.points,command.color)');
  const call = floor.indexOf(hook);
  const success = floor.indexOf('\treturn true', call);
  assert(emit >= 0 && call > emit && success > call);
  assert.match(floor.slice(call, success), /^\s*_draw_ground_light_field\([^\n]+\)\s*$/);
  assert(!/\bif\b|\breturn\b/.test(floor.slice(call, success)));
  assert.equal((source.match(/_draw_ground_light_field\(/g) || []).length, 2);
  assert(source.includes('# BEGIN OPTIONAL GROUND LIGHT FIELD v1'));
  assert(source.includes('# END OPTIONAL GROUND LIGHT FIELD v1'));
});
