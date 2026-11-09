import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const helperPath = resolve(root, 'apps/godot_playable/scripts/world/gh_lockdown_environment_visual.gd');
const scenePath = resolve(root, 'apps/godot_playable/data/scenes/gh_lockdown.json');
const source = readFileSync(helperPath, 'utf8');
const sceneBytes = readFileSync(scenePath);
const scene = JSON.parse(sceneBytes.toString('utf8'));
const sceneSha = createHash('sha256').update(sceneBytes).digest('hex');

function functionBody(name, text = source) {
  const lines = text.split(/\r?\n/);
  const start = lines.findIndex((line) => line.startsWith(`func ${name}(`));
  assert.ok(start >= 0, `missing function ${name}`);
  let end = start + 1;
  while (end < lines.length && !lines[end].startsWith('func ')) end++;
  return lines.slice(start, end).join('\n');
}

function shortFaces() {
  const match = source.match(/const SHORT_FACES: Array = \[([\s\S]*?)\]\s*# GB_SHORT_DATA_END/);
  assert.ok(match, 'positive SHORT_FACES declaration anchor');
  return JSON.parse(`[${match[1]}]`);
}

function samePoint(a, b) {
  return a.length === 2 && b.length === 2 && a[0] === b[0] && a[1] === b[1];
}

function projected([x, y, z], ppm) {
  return [(x - z) * ppm, ((x + z) / 2 - y) * ppm];
}

function signedArea(points) {
  let twice = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i];
    const b = points[(i + 1) % points.length];
    twice += a[0] * b[1] - a[1] * b[0];
  }
  return twice / 2;
}

function insideConvex(point, poly) {
  const signs = poly.map((a, i) => {
    const b = poly[(i + 1) % poly.length];
    return (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
  });
  return signs.every((v) => v >= 0) || signs.every((v) => v <= 0);
}

test('five short faces follow canonical wall edges and preserve the west/east opening', () => {
  assert.equal(sceneSha, '718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850');
  assert.match(source, /const SHORT_SCENE_SHA: String = "718489a1ea4ec4622ccdcefc8459a1dc1bd4f66ccf26abc813b6a06519197850"/);
  const faces = shortFaces();
  assert.equal(faces.length, 5);
  const expected = [
    ['north_east_end', 'lockdown_wall_north', [24, 0], [24, 0.5], 0, 0.9],
    ['south_east_end', 'lockdown_wall_south', [24, 15.5], [24, 16], 0, 0.18],
    ['west_north_jamb', 'lockdown_wall_west_north', [0, 6.5], [0.5, 6.5], 0, 0.9],
    ['west_south_step', 'lockdown_wall_west_south', [0, 15.5], [0.5, 15.5], 0.18, 0.9],
    ['east_north_jamb', 'lockdown_wall_east_north', [23.5, 6.5], [24, 6.5], 0, 0.18],
  ];
  const specs = new Map([
    ['lockdown_wall_north', 0.9], ['lockdown_wall_south', 0.18],
    ['lockdown_wall_west_north', 0.9], ['lockdown_wall_west_south', 0.9],
    ['lockdown_wall_east_north', 0.18], ['lockdown_wall_east_south', 0.18],
  ]);
  const seen = new Set();
  for (const [i, face] of faces.entries()) {
    const [id, owner, a, b, low, high] = expected[i];
    assert.deepEqual([face.id, face.owner, face.a, face.b, face.low, face.high], [id, owner, a, b, low, high]);
    assert.ok(!seen.has(face.id), `unique face ${face.id}`);
    seen.add(face.id);
    const wall = scene.collision.find((row) => row.id === owner);
    assert.ok(wall, `canonical owner ${owner}`);
    const poly = wall.polygon;
    const boundary = poly.some((p, n) => {
      const q = poly[(n + 1) % poly.length];
      return (samePoint(p, a) && samePoint(q, b)) || (samePoint(p, b) && samePoint(q, a));
    });
    assert.ok(boundary, `${id} is an actual canonical edge`);
    assert.equal(high, specs.get(owner));
    assert.equal(low, id === 'west_south_step' ? specs.get('lockdown_wall_south') : 0);
    const zValues = [a[1], b[1]];
    assert.ok(Math.max(...zValues) <= 6.5 || Math.min(...zValues) >= 9.5, `${id} does not enter the opening`);
  }
  assert.equal(new Set(faces.map((face) => face.id)).size, 5);
});

test('projected quads retain positive area across representative scales and cover a jamb point', () => {
  for (const face of shortFaces()) {
    const world = [
      [face.a[0], face.low, face.a[1]], [face.b[0], face.low, face.b[1]],
      [face.b[0], face.high, face.b[1]], [face.a[0], face.high, face.a[1]],
    ];
    for (const ppm of [24, 48, 96, 256]) {
      const quad = world.map((point) => projected(point, ppm));
      assert.ok(quad.flat().every(Number.isFinite));
      assert.notEqual(signedArea(quad), 0, `${face.id} at ${ppm} ppm`);
    }
  }
  const jamb = shortFaces().find((face) => face.id === 'west_north_jamb');
  const jambQuad = [
    [jamb.a[0], jamb.low, jamb.a[1]], [jamb.b[0], jamb.low, jamb.b[1]],
    [jamb.b[0], jamb.high, jamb.b[1]], [jamb.a[0], jamb.high, jamb.a[1]],
  ].map((point) => projected(point, 48));
  const counterexample = projected([0.25, 0.45, 6.5], 48);
  assert.deepEqual(counterexample, [-300, 140.4]);
  assert.ok(insideConvex(counterexample, jambQuad), 'point lies within the short jamb face');
});

test('production staging validates the whole family, uses strict SAT, and emits only after live bindings', () => {
  const equal = functionBody('_short_equal');
  const bindings = functionBody('_short_bindings');
  const quad = functionBody('_short_quad');
  const visible = functionBody('_short_visible');
  const stage = functionBody('_short_stage');
  const emit = functionBody('_short_emit');
  const draw = functionBody('draw_wall');

  assert.match(equal, /P\.number\(actual,float\(expected\),float\(expected\)\)/);
  for (const token of ['_valid_context(', '_valid_projection(', '_short_canonical(', 'is_instance_valid(', 'is_visible_in_tree()', 'get("snapshot")', 'get("scene")', 'get("current_key")', 'get("scene_hash")', 'get("camera")', 'get("viewport_rect")', 'get_viewport_rect()', 'actual_viewport == view', 'view.position != Vector2.ZERO', 'camera_viewport == view.size']) {
    assert.ok(bindings.includes(token), `binding guard ${token}`);
  }
  assert.match(quad, /_nonzero_polygon_area\(points\)/);
  assert.match(quad, /turn == 0\.0/);
  assert.match(visible, /polygon_max <= view_min \|\| view_max <= polygon_min/);
  assert.match(visible, /separated = true/);
  assert.match(visible, /return 0 if separated else 1/);
  assert.ok(visible.indexOf('for axis: Vector2 in axes') >= 0);
  assert.ok(visible.indexOf('return 0 if separated else 1') > visible.indexOf('for axis: Vector2 in axes'));

  for (const token of ['_short_bindings(', '_short_capture(', '_short_quad(', '_short_visible(', 'all.size() != 5', 'visible.append(face)']) {
    assert.ok(stage.includes(token), `staging invariant ${token}`);
  }
  assert.ok(stage.indexOf('all.size() != 5') >= 0);
  assert.ok(stage.indexOf('all.size() != 5') < stage.indexOf('visible.append(face)'));
  assert.doesNotMatch(stage, /FileAccess|load_png_from_buffer|prepare_resources/);
  for (let i = 0; i < 9; i++) assert.ok(functionBody('_short_capture_matches').includes(`stamp[${i}]`));
  assert.ok(emit.includes('_short_bindings(') && emit.includes('_short_capture_matches('));
  const emitDraw = emit.indexOf('canvas.draw_colored_polygon(');
  assert.ok(emitDraw > emit.indexOf('_short_capture_matches('));
  assert.match(emit, /if face\.get\("owner"\) == wall_id:/);
  assert.match(emit, /Color\(0\.15,0\.21,0\.22,1\.0\)/);
  assert.equal((emit.match(/canvas\.draw_colored_polygon\(/g) ?? []).length, 1);

  const stageHook = draw.indexOf('_short_stage(');
  const contact = draw.indexOf('canvas.draw_colored_polygon(contact_points,');
  const inner = draw.indexOf('canvas.draw_colored_polygon(face_points,');
  const outer = draw.indexOf('canvas.draw_colored_polygon(outer_face_points,');
  const emitHook = draw.indexOf('_short_emit(');
  const cap = draw.indexOf('canvas.draw_colored_polygon(cap_points,');
  for (const [name, index] of Object.entries({ stageHook, contact, inner, outer, emitHook, cap })) assert.ok(index >= 0, `positive source anchor ${name}`);
  assert.ok(stageHook < contact && contact < inner && inner < outer && outer < emitHook && emitHook < cap);
  assert.match(draw, /return true\s*$/);
});
