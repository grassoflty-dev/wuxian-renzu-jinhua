import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const root = new URL('../', import.meta.url);
const helper = readFileSync(new URL('apps/godot_playable/scripts/world/cw_furnace_heart_environment_visual.gd', root), 'utf8');
const scene = JSON.parse(readFileSync(new URL('apps/godot_playable/data/scenes/cw_furnace_heart.json', root), 'utf8'));
const projectionSource = readFileSync(new URL('apps/godot_playable/scripts/world/projection.gd', root), 'utf8');

function functionBody(source, name) {
  const start = source.indexOf(`func ${name}(`);
  assert.notEqual(start, -1, `missing function ${name}`);
  const end = source.indexOf('\nfunc ', start + 1);
  return source.slice(start, end < 0 ? source.length : end);
}

function parseWalls(source) {
  const start = source.indexOf('const WALLS = [');
  assert.notEqual(start, -1, 'missing canonical wall table');
  const end = source.indexOf('\n]', start);
  assert.ok(end > start, 'wall table has a closing boundary');
  const table = source.slice(start, end);
  const rows = [...table.matchAll(/\{'id':'([^']+)','polygon':(\[\[[\s\S]*?\]\])\}/g)];
  return rows.map(([, id, polygon]) => ({ id, polygon: JSON.parse(polygon) }));
}

function parseSides(source) {
  const start = source.indexOf('const SIDES = [');
  assert.notEqual(start, -1, 'missing exterior-side table');
  const end = source.indexOf('\n]', start);
  assert.ok(end > start, 'side table has a closing boundary');
  const table = source.slice(start, end);
  const rows = [...table.matchAll(/\{'a':(\[[^\]]+\]),'b':(\[[^\]]+\]),'low':([\d.]+),'high':([\d.]+),'clip_low_end':(true|false)\}/g)];
  return rows.map(([, a, b, low, high, clipped]) => ({
    a: JSON.parse(a), b: JSON.parse(b), low: Number(low), high: Number(high), clip_low_end: clipped === 'true'
  }));
}

function project(point, camera) {
  const [x, y, z] = point;
  const [ox, oy, oz] = camera.origin;
  return [camera.center[0] + ((x - ox) - (z - oz)) * camera.ppm,
    camera.center[1] + (((x - ox) + (z - oz)) * camera.ppm * 0.5) - (y - oy) * camera.ppm];
}

function area2(points) {
  let area = 0;
  for (let i = 0; i < points.length; i++) {
    const a = points[i], b = points[(i + 1) % points.length];
    area += a[0] * b[1] - b[0] * a[1];
  }
  return area;
}

function validConvex(points) {
  if (points.length < 4 || points.length > 5 || points.some(p => p.length !== 2 || !p.every(Number.isFinite))) return false;
  const area = area2(points);
  if (!Number.isFinite(area) || area === 0) return false;
  for (let i = 0; i < points.length; i++) {
    const a = points[i], b = points[(i + 1) % points.length], c = points[(i + 2) % points.length];
    const ex = b[0] - a[0], ey = b[1] - a[1];
    const fx = c[0] - b[0], fy = c[1] - b[1];
    const turn = ex * fy - ey * fx;
    if (![ex, ey, turn].every(Number.isFinite) || (ex === 0 && ey === 0) || turn === 0 || Math.sign(turn) !== Math.sign(area)) return false;
  }
  return true;
}

function overlapsRect(points, rect) {
  const corners = [[rect.x, rect.y], [rect.x + rect.w, rect.y], [rect.x + rect.w, rect.y + rect.h], [rect.x, rect.y + rect.h]];
  const axes = [[1, 0], [0, 1]];
  for (let i = 0; i < points.length; i++) {
    const a = points[i], b = points[(i + 1) % points.length];
    axes.push([-(b[1] - a[1]), b[0] - a[0]]);
  }
  let separated = false;
  for (const [ax, ay] of axes) {
    if (![ax, ay].every(Number.isFinite) || (ax === 0 && ay === 0)) return -1;
    const p = points.map(([x, y]) => (x - points[0][0]) * ax + (y - points[0][1]) * ay);
    const r = corners.map(([x, y]) => (x - points[0][0]) * ax + (y - points[0][1]) * ay);
    if (![...p, ...r].every(Number.isFinite)) return -1;
    const pMin = Math.min(...p), pMax = Math.max(...p), rMin = Math.min(...r), rMax = Math.max(...r);
    if (pMin >= pMax || rMin >= rMax) return -1;
    if (pMax <= rMin || rMax <= pMin) separated = true;
  }
  return separated ? 0 : 1;
}

test('source-static geometry matches the canonical Heart shell', () => {
  const walls = parseWalls(helper);
  assert.equal(walls.length, 6);
  const canonical = scene.collision.filter(row => row.id.startsWith('cw_furnace_heart_wall_'));
  assert.equal(canonical.length, 6);
  assert.deepEqual(walls, canonical);
  assert.deepEqual(walls.map(row => row.id), [
    'cw_furnace_heart_wall_north', 'cw_furnace_heart_wall_south',
    'cw_furnace_heart_wall_west_north', 'cw_furnace_heart_wall_west_south',
    'cw_furnace_heart_wall_east_north', 'cw_furnace_heart_wall_east_south'
  ]);
});

test('the twelve side records retain the two precise cap clips and 74-vertex plan', () => {
  const sides = parseSides(helper);
  assert.equal(sides.length, 12);
  const expected = [
    [[0.5, 0.5], [23.5, 0.5], 0, 1, true], [[23.5, 0.5], [24, 0.5], 0.18, 1, false],
    [[24, 0], [24, 0.5], 0, 1, false], [[0, 16], [24, 16], 0, 0.18, false],
    [[24, 15.5], [24, 16], 0, 0.18, false], [[0.5, 0.5], [0.5, 6], 0, 1, false],
    [[0, 6], [0.5, 6], 0, 1, false], [[0.5, 9.5], [0.5, 15.5], 0, 1, true],
    [[0, 15.5], [0.5, 15.5], 0.18, 1, false], [[24, 0.5], [24, 6], 0, 0.18, false],
    [[23.5, 6], [24, 6], 0, 0.18, false], [[24, 9.5], [24, 15.5], 0, 0.18, false]
  ];
  assert.deepEqual(sides.map(s => [s.a, s.b, s.low, s.high, s.clip_low_end]), expected);
  assert.equal(sides.filter(side => side.clip_low_end).length, 2);
  const capHeights = [...helper.matchAll(/const CAP_HEIGHTS = \[([^\]]+)\]/g)];
  assert.equal(capHeights.length, 1);
  assert.deepEqual(JSON.parse(`[${capHeights[0][1].replaceAll('HIGH_M', '1').replaceAll('LOW_M', '0.18')}]`), [1, 0.18, 1, 1, 0.18, 0.18]);
  assert.equal(12 * 4 + 6 * 4, 72);
  assert.equal(12 * 4 + 6 * 4 + 2, 74);
  const plan = functionBody(helper, '_plan');
  assert.ok(plan.includes('if face.clip_low_end:'));
  assert.ok(plan.includes('final_world = [a,b-axis*LOW_M,Vector3(b.x,LOW_M,b.z),top_b,top_a]'));
  assert.ok(plan.includes('result.append') && plan.includes('CAP_HEIGHTS[index]'));
  assert.ok(functionBody(helper, 'draw_shell').includes('plan.size() != 18'));
  assert.ok(functionBody(helper, 'draw_shell').includes('final_vertex_count != 74'));
});

test('projection and strict polygon/SAT math cover tiny scale, touch, outside and invalid geometry', () => {
  const projection = projectionSource.replaceAll(/\s+/g, '');
  assert.ok(projection.includes('returnscreen_center+Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)'));
  const walls = parseWalls(helper), sides = parseSides(helper);
  const heights = [1, 0.18, 1, 1, 0.18, 0.18];
  for (const ppm of [0.001, 24, 48, 96, 256]) {
    const camera = { origin: [12, 0, 8], center: [0, 0], ppm };
    const faces = sides.map(side => {
      const a = [side.a[0], side.low, side.a[1]], b = [side.b[0], side.low, side.b[1]];
      const topA = [a[0], side.high, a[2]], topB = [b[0], side.high, b[2]];
      let world = [a, b, topB, topA];
      if (side.clip_low_end) {
        const axis = b[0] > a[0] ? [1, 0, 0] : [0, 0, 1];
        world = [a, [b[0] - axis[0] * 0.18, b[1] - axis[1] * 0.18, b[2] - axis[2] * 0.18], [b[0], 0.18, b[2]], topB, topA];
      }
      return world.map(point => project(point, camera));
    });
    for (let index = 0; index < walls.length; index++) {
      const cap = walls[index].polygon.map(([x, z]) => project([x, heights[index], z], camera));
      faces.push(cap);
    }
    assert.equal(faces.length, 18);
    for (const face of faces) assert.equal(validConvex(face), true, `valid face at ${ppm} ppm`);
  }
  assert.equal(validConvex([[0, 0], [1, 0], [1, 1], [0, 1]]), true);
  assert.equal(validConvex([[0, 0], [1, 0], [0, 0], [0, 1]]), false);
  assert.equal(validConvex([[0, 0], [1, 0], [2, 0], [0, 1]]), false);
  assert.equal(validConvex([[0, 0], [1, 0], [1, 0], [0, 1]]), false);
  assert.equal(validConvex([[0, 0], [1, 0], [1, Infinity], [0, 1]]), false);
  const rect = { x: 0, y: 0, w: 10, h: 10 };
  assert.equal(overlapsRect([[2, 2], [4, 2], [4, 4], [2, 4]], rect), 1);
  assert.equal(overlapsRect([[10, 2], [12, 2], [12, 4], [10, 4]], rect), 0);
  assert.equal(overlapsRect([[10.001, 2], [12, 2], [12, 4], [10.001, 4]], rect), 0);
  const polygon = functionBody(helper, '_valid_polygon');
  assert.ok(polygon.includes('not is_finite(area2) or area2 == 0.0'));
  assert.ok(polygon.includes('not is_finite(turn)') && polygon.includes('turn == 0.0'));
  assert.ok(!helper.includes('MIN_AREA2'));
  const sat = functionBody(helper, '_visible');
  assert.ok(sat.includes('polygon_max <= rect_min or rect_max <= polygon_min'));
  assert.ok(sat.includes('return 0 if separated else 1'));
});

test('full canonical source and live canvas/projection bindings guard both sides of staging', () => {
  const context = functionBody(helper, '_context');
  assert.ok(context.includes('not _same_json(scene,_canonical_scene)'));
  const binding = functionBody(helper, '_canvas_matches');
  for (const [field, value] of [
    ['snapshot', 'snapshot_value'], ['scene', 'scene_value'], ['current_key', 'key_value'],
    ['scene_hash', 'scene_hash_value'], ['camera', 'camera_value'], ['viewport_rect', 'viewport']
  ]) assert.ok(binding.includes(`canvas_value.get('${field}') != ${value}`), `binding ${field}`);
  assert.ok(binding.includes('canvas_value.get_viewport_rect() != viewport'));
  assert.ok(binding.includes('camera_value.viewport != viewport.size'));
  assert.ok(binding.includes('viewport.position != Vector2.ZERO'));
  assert.ok(binding.includes('not viewport_value is Rect2') && binding.includes('not _valid_rect(viewport)'));
  assert.ok(binding.includes('not canvas_value.is_inside_tree()') && binding.includes('is_instance_valid(canvas_value)'));
  assert.ok(binding.includes('is_visible_in_tree()') && binding.includes('is_queued_for_deletion()'));
  const camera = functionBody(helper, '_valid_camera');
  assert.ok(camera.includes('camera.get_script() != ProjectionScript'));
  assert.ok(camera.includes('typeof(camera.initialized) != TYPE_BOOL') && camera.includes('not camera.initialized'));
  assert.ok(camera.includes('camera.ppm > 0.0') && camera.includes('camera.ppm,0.0,256.0'));
  const draw = functionBody(helper, 'draw_shell');
  const firstBinding = draw.indexOf('_canvas_matches(');
  const contextCall = draw.indexOf('_context(');
  const planCall = draw.indexOf('_plan(');
  const eachFaceCheck = draw.indexOf('if not _valid_polygon(original) or not _valid_polygon(points):');
  const visibleSelection = draw.indexOf('drawable.append(command)');
  const noIntersectionFallback = draw.indexOf('if final_vertex_count != 74 or drawable.is_empty():');
  const finalBinding = draw.lastIndexOf('_canvas_matches(');
  const firstEmit = draw.indexOf('canvas_value.draw_colored_polygon');
  for (const [label, index] of Object.entries({ firstBinding, contextCall, planCall, eachFaceCheck, visibleSelection, noIntersectionFallback, finalBinding, firstEmit })) assert.notEqual(index, -1, `present ${label} anchor`);
  assert.ok(firstBinding < contextCall && contextCall < planCall && planCall < eachFaceCheck && eachFaceCheck < visibleSelection && visibleSelection < noIntersectionFallback && noIntersectionFallback < finalBinding && finalBinding < firstEmit);
  assert.match(draw, /if not _valid_polygon\(original\) or not _valid_polygon\(points\):\s+return false/);
  assert.match(draw, /if final_vertex_count != 74 or drawable\.is_empty\(\):\s+return false/);
});
