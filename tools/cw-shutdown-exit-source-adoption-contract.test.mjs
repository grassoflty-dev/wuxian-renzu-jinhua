import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const helperPath = resolve(here, '../apps/godot_playable/scripts/world/cw_shutdown_exit_environment_visual.gd');
const scenePath = resolve(here, '../apps/godot_playable/data/scenes/cw_shutdown_exit.json');
const rendererPath = resolve(here, '../apps/godot_playable/scripts/world/renderer.gd');
const source = readFileSync(helperPath, 'utf8');
const scene = JSON.parse(readFileSync(scenePath, 'utf8'));
const renderer = readFileSync(rendererPath, 'utf8');

function functionBody(name) {
  const start = source.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `missing function ${name}`);
  const next = source.indexOf('\nfunc ', start + 1);
  return source.slice(start, next < 0 ? source.length : next);
}

function rendererFunctionBody(name) {
  const start = renderer.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `missing Renderer function ${name}`);
  const next = renderer.indexOf('\nfunc ', start + 1);
  return renderer.slice(start, next < 0 ? renderer.length : next);
}

function constantArray(name) {
  const match = new RegExp(`^const ${name} = \\[`, 'm').exec(source);
  assert.ok(match, `missing ${name} constant`);
  const start = source.indexOf('[', match.index);
  let depth = 0;
  let quote = '';
  let escaped = false;
  let end = -1;
  for (let i = start; i < source.length; i += 1) {
    const ch = source[i];
    if (quote) {
      if (escaped) escaped = false;
      else if (ch === '\\') escaped = true;
      else if (ch === quote) quote = '';
      continue;
    }
    if (ch === "'" || ch === '"') { quote = ch; continue; }
    if (ch === '[') depth += 1;
    if (ch === ']') {
      depth -= 1;
      if (depth === 0) { end = i + 1; break; }
    }
  }
  assert.ok(end > start, `unterminated ${name} constant`);
  const literal = source.slice(start, end)
    .replace(/\bHIGH_M\b/g, '1.0')
    .replace(/\bLOW_M\b/g, '0.18')
    .replace(/'/g, '"');
  return JSON.parse(literal);
}

function area2(points) {
  let sum = 0;
  for (let i = 1; i < points.length - 1; i += 1) {
    const ax = points[i][0] - points[0][0];
    const ay = points[i][1] - points[0][1];
    const bx = points[i + 1][0] - points[0][0];
    const by = points[i + 1][1] - points[0][1];
    sum += ax * by - ay * bx;
  }
  return sum;
}

test('canonical Shutdown Exit shell has five walls, ten sides, two clipped ends and bounded plans', () => {
  const walls = constantArray('WALLS');
  const sides = constantArray('SIDES');
  assert.equal(walls.length, 5);
  assert.deepEqual(walls.map(({ id, polygon }) => ({ id, polygon })),
    scene.collision.map(({ id, polygon }) => ({ id, polygon })));
  assert.equal(sides.length, 10);
  assert.deepEqual(sides.map(({ a, b, low, high, clip_low_end }) => [a, b, low, high, clip_low_end]), [
    [[0.5, 0.5], [23.5, 0.5], 0, 1, true],
    [[23.5, 0.5], [24, 0.5], 0.18, 1, false],
    [[24, 0], [24, 0.5], 0, 1, false],
    [[0, 16], [24, 16], 0, 0.18, false],
    [[24, 15.5], [24, 16], 0, 0.18, false],
    [[0.5, 0.5], [0.5, 6], 0, 1, false],
    [[0, 6], [0.5, 6], 0, 1, false],
    [[0.5, 9.5], [0.5, 15.5], 0, 1, true],
    [[0, 15.5], [0.5, 15.5], 0.18, 1, false],
    [[24, 0.5], [24, 15.5], 0, 0.18, false],
  ]);
  assert.equal(sides.filter((side) => side.clip_low_end).length, 2);
  assert.equal(sides.length + walls.length, 15);
  assert.equal(sides.reduce((n, side) => n + (side.clip_low_end ? 5 : 4), 0) + walls.length * 4, 62);
  const floorCells = 6 * 4;
  const verticalJoints = 5;
  const horizontalJoints = 3 * 6;
  assert.equal(floorCells + verticalJoints + horizontalJoints, 47);
  assert.equal(47 * 4, 188);
});

test('projected-face validation is strict-convex and rejects exact zero without an epsilon cutoff', () => {
  const valid = functionBody('_valid_polygon');
  assert.ok(valid.includes('not is_finite(area2) or area2 == 0.0'));
  assert.ok(valid.includes('turn == 0.0'));
  assert.ok(valid.includes('(turn > 0.0) != (area2 > 0.0)'));
  assert.ok(!source.includes('MIN_AREA2'));
  assert.ok(!valid.includes('absf(area2)'));
  const sat = functionBody('_visible');
  assert.ok(sat.includes('polygon_max <= rect_min or rect_max <= polygon_min'));
  assert.ok(sat.includes('return 0 if separated else 1'));
  const tiny = area2([[0, 0], [1e-12, 0], [1e-12, 1e-12], [0, 1e-12]]);
  assert.ok(tiny > 0, 'tiny positive projected area remains nonzero');
  assert.equal(area2([[0, 0], [1, 0], [2, 0], [3, 0]]), 0);
});

test('canonical scene equality is checked after the guarded load', () => {
  const context = functionBody('_context');
  const loadAt = context.indexOf('_load_scene_once()');
  const compareAt = context.indexOf('_same_json(scene,_canonical_scene)');
  assert.ok(loadAt >= 0, 'canonical load anchor exists');
  assert.ok(compareAt >= 0, 'exact canonical comparison exists');
  assert.ok(compareAt > loadAt);
});

test('canvas bindings are complete and rechecked after staging before either draw emits', () => {
  const binding = functionBody('_canvas_matches');
  for (const anchor of [
    "canvas.get('snapshot')", "canvas.get('scene')", "canvas.get('current_key')",
    "canvas.get('scene_hash')", "canvas.get('camera')", "canvas.get('viewport_rect')",
    'bound_snapshot != snapshot', 'bound_scene != scene', 'bound_key != key_value',
    'bound_hash != hash_value', 'bound_camera != camera', 'bound_viewport != viewport',
    'canvas.get_viewport_rect()', 'actual_viewport == viewport', 'camera.viewport == viewport.size',
    'viewport.position != Vector2.ZERO', 'canvas.is_inside_tree()', 'canvas.is_visible_in_tree()',
    'canvas.is_queued_for_deletion()', '_valid_camera(camera,viewport)',
  ]) assert.ok(binding.includes(anchor), `missing binding guard: ${anchor}`);
  for (const [name, staged] of [['draw_shell', 'final_vertex_count != 62'], ['draw_floor', 'vertex_count != 188']]) {
    const body = functionBody(name);
    const binds = [...body.matchAll(/_canvas_matches\(/g)].map((match) => match.index);
    const contextAt = body.indexOf('_context(');
    const planAt = body.indexOf('_plan(');
    const geometryAt = name === 'draw_shell' ? body.indexOf('if not _valid_polygon(original) or not _valid_polygon(points)') : body.indexOf('if points.size() != 4 or not _valid_polygon(points)');
    const visibilityAt = body.indexOf('var visible: int = _visible(points,viewport)');
    const invalidVisibilityAt = body.indexOf('if visible < 0:', visibilityAt);
    const positiveVisibilityAt = body.indexOf('if visible > 0:', invalidVisibilityAt);
    const appendAt = body.indexOf('drawable.append(command)', positiveVisibilityAt);
    const stageAt = body.indexOf(staged);
    const paletteAt = body.indexOf('_apply_completed_palette(');
    const paintAt = body.indexOf('draw_colored_polygon(');
    assert.equal(binds.length, 2, `${name} has entry and final binding checks`);
    for (const [label, position] of [['context', contextAt], ['plan', planAt], ['geometry', geometryAt], ['visibility', visibilityAt], ['invalid visibility', invalidVisibilityAt], ['positive visibility', positiveVisibilityAt], ['append', appendAt], ['staging', stageAt], ['palette', paletteAt], ['paint', paintAt]]) {
      assert.ok(position >= 0, `${name} ${label} anchor exists`);
    }
    assert.ok(binds[0] < contextAt && contextAt < planAt && planAt < geometryAt && geometryAt < visibilityAt);
    assert.ok(visibilityAt < invalidVisibilityAt && invalidVisibilityAt < positiveVisibilityAt && positiveVisibilityAt < appendAt && appendAt < stageAt);
    assert.ok(stageAt < paletteAt && paletteAt < binds[1] && binds[1] < paintAt);
    assert.ok(body.includes('return false'), `${name} fails closed before emission`);
  }
});

test('Renderer wires both shell witnesses and the Shutdown floor hook at the admitted sites', () => {
  const heartPreload = /const\s+CwFurnaceHeartEnvironmentVisual\s*=\s*preload\('res:\/\/scripts\/world\/cw_furnace_heart_environment_visual\.gd'\)/;
  const shutdownPreload = /const\s+CwShutdownExitEnvironmentVisual\s*=\s*preload\('res:\/\/scripts\/world\/cw_shutdown_exit_environment_visual\.gd'\)/;
  const heartInstance = /var\s+cw_furnace_heart_environment_visual\s*=\s*CwFurnaceHeartEnvironmentVisual\.new\(\)/;
  const shutdownInstance = /var\s+cw_shutdown_exit_environment_visual\s*=\s*CwShutdownExitEnvironmentVisual\.new\(\)/;
  for (const [label, pattern] of [['Heart preload', heartPreload], ['Shutdown preload', shutdownPreload], ['Heart instance', heartInstance], ['Shutdown instance', shutdownInstance]]) {
    assert.equal([...renderer.matchAll(new RegExp(pattern.source, 'g'))].length, 1, `${label} appears once`);
  }
  const cwPreload = renderer.indexOf("const CwFurnaceVisual = preload(");
  const heartPreloadAt = renderer.search(heartPreload);
  const shutdownPreloadAt = renderer.search(shutdownPreload);
  const cwInstance = renderer.indexOf('var cw_furnace_visual');
  const heartInstanceAt = renderer.search(heartInstance);
  const shutdownInstanceAt = renderer.search(shutdownInstance);
  for (const [label, position] of Object.entries({ cwPreload, heartPreloadAt, shutdownPreloadAt, cwInstance, heartInstanceAt, shutdownInstanceAt })) assert.ok(position >= 0, `${label} anchor exists`);
  assert.ok(cwPreload < heartPreloadAt && heartPreloadAt < shutdownPreloadAt);
  assert.ok(cwInstance < heartInstanceAt && heartInstanceAt < shutdownInstanceAt);

  const draw = rendererFunctionBody('_draw');
  const baseFloor = draw.indexOf('_poly(floor_poly,0,floor_color,true)');
  const floorHook = draw.indexOf('cw_shutdown_exit_environment_visual.draw_floor(');
  const objects = draw.indexOf('var objects: Array = []');
  for (const [label, position] of Object.entries({ baseFloor, floorHook, objects })) assert.ok(position >= 0, `${label} anchor exists`);
  assert.ok(baseFloor < floorHook && floorHook < objects);
  assert.match(draw, /^\s*cw_shutdown_exit_environment_visual\.draw_floor\(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect\)\s*$/m);
  assert.ok(!draw.includes('cw_shutdown_exit_floor_drawn'), 'floor result does not suppress legacy geometry');

  const sentinelDraw = draw.indexOf('var gh_sentinel_shell_drawn: bool');
  const heartDraw = draw.indexOf('var cw_furnace_heart_shell_drawn: bool');
  const shutdownDraw = draw.indexOf('var cw_shutdown_exit_shell_drawn: bool');
  const wallLoop = draw.indexOf("for wall in scene.get('collision',[])");
  const shutdownGate = draw.indexOf('if cw_shutdown_exit_shell_drawn and cw_shutdown_exit_environment_visual.owns_collision(wall):', wallLoop);
  const heartGate = draw.indexOf('if cw_furnace_heart_shell_drawn and cw_furnace_heart_environment_visual.owns_collision(wall):', wallLoop);
  const sentinelGate = draw.indexOf('if gh_sentinel_shell_drawn and gh_sentinel_environment_visual.owns_collision(wall):', wallLoop);
  for (const [label, position] of Object.entries({ sentinelDraw, heartDraw, shutdownDraw, wallLoop, shutdownGate, heartGate, sentinelGate })) assert.ok(position >= 0, `${label} anchor exists`);
  assert.ok(sentinelDraw < heartDraw && heartDraw < shutdownDraw && shutdownDraw < wallLoop);
  assert.ok(wallLoop < shutdownGate && shutdownGate < heartGate && heartGate < sentinelGate);
  assert.match(draw.slice(shutdownGate, heartGate), /if cw_shutdown_exit_shell_drawn and cw_shutdown_exit_environment_visual\.owns_collision\(wall\):\s*continue/);
  assert.match(draw.slice(heartGate, sentinelGate), /if cw_furnace_heart_shell_drawn and cw_furnace_heart_environment_visual\.owns_collision\(wall\):\s*continue/);
  assert.match(draw, /var\s+cw_furnace_heart_shell_drawn\s*:\s*bool\s*=\s*cw_furnace_heart_environment_visual\.draw_shell\(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect\)/);
  assert.match(draw, /var\s+cw_shutdown_exit_shell_drawn\s*:\s*bool\s*=\s*cw_shutdown_exit_environment_visual\.draw_shell\(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect\)/);
});
