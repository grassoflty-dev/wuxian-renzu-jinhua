import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const helper = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/cw_boiler_environment_visual.gd'), 'utf8');
const scene = JSON.parse(readFileSync(resolve(root, 'apps/godot_playable/data/scenes/cw_boiler_chamber.json'), 'utf8'));
const presentation = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/presentation.gd'), 'utf8');
const projection = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/projection.gd'), 'utf8');
const renderer = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/renderer.gd'), 'utf8');

function functionBody(source, name) {
  const marker = 'func ' + name + '(';
  const start = source.indexOf(marker);
  assert.notEqual(start, -1, 'missing source function ' + name);
  const next = /\n(?:static )?func [A-Za-z_][A-Za-z0-9_]*\(/g;
  next.lastIndex = start + marker.length;
  const match = next.exec(source);
  return source.slice(start, match ? match.index : source.length);
}

function assertOrdered(source, first, second) {
  const firstIndex = source.indexOf(first);
  const secondIndex = source.indexOf(second);
  assert.notEqual(firstIndex, -1, 'missing order anchor: ' + first);
  assert.notEqual(secondIndex, -1, 'missing order anchor: ' + second);
  assert.ok(firstIndex < secondIndex, first + ' must precede ' + second);
}

function projectedRing(progress, layer, ppm) {
  const rx = 0.20 + 0.08 * progress;
  const rz = 0.12 + 0.04 * progress;
  const height = 0.10 + 0.12 * layer + 0.60 * progress;
  const points = [];
  for (let index = 0; index < 24; index += 1) {
    const angle = Math.PI * 2 * index / 24;
    const x = 12 + rx * Math.cos(angle);
    const z = 11.5 + rz * Math.sin(angle);
    points.push([400 + ((x - 12) - (z - 11.5)) * ppm,
      300 + (((x - 12) + (z - 11.5)) * 0.5 - height) * ppm]);
  }
  return points;
}

function ringAreaAndConvexity(points) {
  let area2 = 0;
  const turns = [];
  for (let index = 0; index < points.length; index += 1) {
    const a = points[index];
    const b = points[(index + 1) % points.length];
    const c = points[(index + 2) % points.length];
    area2 += a[0] * b[1] - b[0] * a[1];
    turns.push((b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]));
  }
  return {area2, turns};
}

function overlapsViewport(points, rect) {
  const axes = [[1, 0], [0, 1]];
  for (let i = 0; i < points.length; i += 1) {
    const a = points[i];
    const b = points[(i + 1) % points.length];
    axes.push([-(b[1] - a[1]), b[0] - a[0]]);
  }
  const corners = [[rect[0], rect[1]], [rect[2], rect[1]], [rect[2], rect[3]], [rect[0], rect[3]]];
  for (const axis of axes) {
    const project = (point) => point[0] * axis[0] + point[1] * axis[1];
    const polygon = points.map(project);
    const view = corners.map(project);
    if (Math.max(...polygon) <= Math.min(...view) || Math.max(...view) <= Math.min(...polygon)) return false;
  }
  return true;
}

test('steam candidate is bound to the canonical staged vent and Presentation environment contract', () => {
  const vent = scene.hazards.find((row) => row.id === 'cw_boiler_vent_staged');
  assert.ok(vent);
  assert.equal(vent.kind, 'steam_jet');
  assert.deepEqual(vent.polygon, [[11, 10], [13, 10], [13, 13], [11, 13]]);
  assert.deepEqual(vent.environment, {
    warningMs: 900, activeMs: 700, recoveryMs: 3400, damage: 10,
    damageIntervalMs: 700, tag: 'pressure',
  });
  assert.match(helper, /const STEAM_REGION = \[\[11\.0,10\.0\],\[13\.0,10\.0\],\[13\.0,13\.0\],\[11\.0,13\.0\]\]/);
  const frame = functionBody(helper, '_pressure_steam_frame');
  assert.match(frame, /SteamPresentation\.environment\(row\)/);
  assert.match(frame, /Vector3\(12,0,11\.5\)/);
  assert.match(frame, /'warning':900,'active':700,'recovery':3400,'suppressed':6000/);
  assert.match(frame, /P\.counter\(environment\.remainingMs,1,maximum\)/);
  const environment = functionBody(presentation, 'environment');
  assert.match(environment, /'steam_jet':'pressure'/);
  assert.match(environment, /h\.active!=\(e\.phase=='active'\)/);
  assert.match(environment, /h\.get\('phaseActive'\)!=true/);
});

test('active phase is the only emitted phase and reduced motion fixes the same midpoint recipe', () => {
  const draw = functionBody(helper, 'draw_pressure_steam');
  assert.match(draw, /frame\.is_empty\(\) or frame\.phase!='active'/);
  assert.match(draw, /var progress: float = 0\.5 if reduced_motion_value else \(700\.0-float\(frame\.remainingMs\)\)\/700\.0/);
  assert.match(draw, /var radius_x: float = 0\.20\+0\.08\*progress/);
  assert.match(draw, /var radius_z: float = 0\.12\+0\.04\*progress/);
  assert.match(draw, /0\.10\+0\.12\*float\(layer\)\+0\.60\*progress/);
  assert.match(draw, /steam_color\.a = 0\.07\*\(1\.0-0\.6\*progress\)/);
  assert.notDeepEqual(projectedRing(0.1, 0, 48), projectedRing(0.9, 0, 48));
});

test('all three 24-point rings are finite, strictly convex, nonzero, SAT-visible, and staged before emission', () => {
  const polygon = functionBody(helper, '_pressure_steam_polygon_valid');
  assert.match(polygon, /points\.size\(\)!=24/);
  assert.match(polygon, /not _finite_vector2\(point\)/);
  assert.match(polygon, /turn==0\.0/);
  assert.match(polygon, /is_finite\(signed_area\) and signed_area != 0\.0/);
  assert.doesNotMatch(polygon, /0\.000001/);
  const visibility = functionBody(helper, '_pressure_steam_visibility');
  assert.match(visibility, /axes\.append\(Vector2\(-edge\.y,edge\.x\)\)/);
  assert.match(visibility, /polygon_max<=view_min or view_max<=polygon_min/);
  const projectionFunction = functionBody(projection, 'project');
  assert.match(projectionFunction, /screen_center\+Vector2\(\(d\.x-d\.z\)\*ppm/);
  for (const ppm of [0.001, 48]) {
    for (let layer = 0; layer < 3; layer += 1) {
      const points = projectedRing(0.5, layer, ppm);
      const {area2, turns} = ringAreaAndConvexity(points);
      assert.ok(points.length === 24 && points.flat().every(Number.isFinite));
      assert.ok(Number.isFinite(area2) && area2 !== 0);
      assert.ok(turns.every((turn) => Number.isFinite(turn) && turn !== 0));
      assert.ok(turns.every((turn) => Math.sign(turn) === Math.sign(turns[0])));
      assert.ok(overlapsViewport(points, [-1000, -1000, 1000, 1000]));
    }
  }
  const draw = functionBody(helper, 'draw_pressure_steam');
  assert.match(draw, /for layer: int in range\(3\)/);
  assert.match(draw, /for index: int in range\(24\)/);
  assert.match(draw, /plans\.append\(points\)/);
  assert.equal((draw.match(/canvas\.draw_colored_polygon/g) || []).length, 1);
  assertOrdered(draw, 'if not _pressure_steam_polygon_valid(points)', 'var visibility: int = _pressure_steam_visibility(points,viewport_value)');
  assertOrdered(draw, 'plans.append(points)', 'if not any_visible');
  assertOrdered(draw, 'if not any_visible', 'for points: PackedVector2Array in plans');
});

test('canvas bindings, actual viewport and reduced-motion witness are checked before planning and again before drawing', () => {
  const bindings = functionBody(helper, '_pressure_steam_canvas_matches');
  for (const token of ['is_visible_in_tree()', 'get_viewport_rect()', 'viewport.position!=Vector2.ZERO',
    'actual_viewport!=viewport', 'stored_viewport!=viewport', 'typeof(stored_reduced_motion)!=TYPE_BOOL',
    'stored_reduced_motion!=reduced_motion_value', '_outer_face_context_matches(']) {
    assert.ok(bindings.includes(token), 'missing binding requirement ' + token);
  }
  const draw = functionBody(helper, 'draw_pressure_steam');
  const bindingCall = 'if not _pressure_steam_canvas_matches(';
  assert.equal((draw.match(/_pressure_steam_canvas_matches\(/g) || []).length, 2);
  const firstBinding = draw.indexOf(bindingCall);
  const finalBinding = draw.lastIndexOf(bindingCall);
  assert.notEqual(firstBinding, -1);
  assert.notEqual(finalBinding, firstBinding);
  assertOrdered(draw, 'if typeof(projection_value.get(\'initialized\'))', bindingCall);
  assertOrdered(draw.slice(firstBinding), 'if not _pressure_steam_canvas_matches(', 'var frame: Dictionary = _pressure_steam_frame');
  assertOrdered(draw.slice(finalBinding), 'if not _pressure_steam_canvas_matches(', 'canvas.draw_colored_polygon');
});

test('Renderer calls the steam hook exactly once immediately before the existing hazard pass', () => {
  const draw = functionBody(renderer, '_draw');
  const call = 'cw_boiler_environment_visual.draw_pressure_steam(self,snapshot,scene,current_key,camera,viewport_rect,reduced_motion)';
  assert.equal(draw.split(call).length - 1, 1);
  const callIndex = draw.indexOf(call);
  assert.notEqual(callIndex, -1);
  assertOrdered(draw, call, '_hazards()');
  assert.match(draw.slice(callIndex + call.length), /^\r?\n\t_hazards\(\)/);
});
