import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const helperPath = resolve('apps/godot_playable/scripts/world/mh_fog_pier_environment_visual.gd');
const scenePath = resolve('apps/godot_playable/data/scenes/mh_fog_pier.json');
const helperBytes = readFileSync(helperPath);
const helper = helperBytes.toString('utf8');
const sceneBytes = readFileSync(scenePath);
const scene = JSON.parse(sceneBytes.toString('utf8'));
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');

function markedBlock(source, label) {
  const re = new RegExp(`# FP_OUTER_${label}_BEGIN\\r?\\n([\\s\\S]*?)# FP_OUTER_${label}_END`, 'g');
  const matches = [...source.matchAll(re)];
  assert.equal(matches.length, 1, `one ${label} block must exist`);
  return matches[0][1];
}

function functionBlock(source, name) {
  const start = new RegExp(`^func ${name}\\b`, 'm');
  const match = start.exec(source);
  assert.ok(match, `function ${name} must exist before source-order checks`);
  const tail = source.slice(match.index);
  const next = /^func \w+\(/m.exec(tail.slice(match[0].length));
  return next ? tail.slice(0, match[0].length + next.index) : tail;
}

function pointKey(a, b) {
  const pair = [a, b].sort((p, q) => p[0] - q[0] || p[1] - q[1]);
  return `${pair[0].join(',')}|${pair[1].join(',')}`;
}

function project([x, y, z], camera) {
  return [camera.center[0] + ((x - z) - (camera.origin[0] - camera.origin[2])) * camera.ppm,
    camera.center[1] + (((x + z) / 2 - y) - (camera.origin[0] + camera.origin[2]) / 2) * camera.ppm];
}

function validQuad(points) {
  if (points.length !== 4 || points.some((p) => p.length !== 2 || !p.every(Number.isFinite))) return false;
  const area2 = points.slice(1, 3).reduce((sum, p, i) => {
    const q = points[i + 2];
    return sum + (p[0] - points[0][0]) * (q[1] - points[0][1]) - (p[1] - points[0][1]) * (q[0] - points[0][0]);
  }, 0);
  if (!Number.isFinite(area2) || area2 === 0) return false;
  const turns = points.map((p, i) => {
    const a = points[(i + 1) % 4].map((v, j) => v - p[j]);
    const b = points[(i + 2) % 4].map((v, j) => v - points[(i + 1) % 4][j]);
    return a[0] * b[1] - a[1] * b[0];
  });
  return turns.every((v) => Number.isFinite(v) && v !== 0 && Math.sign(v) === Math.sign(turns[0]));
}

function containsConvex(points, point) {
  const signs = points.map((p, i) => {
    const q = points[(i + 1) % points.length];
    return (q[0] - p[0]) * (point[1] - p[1]) - (q[1] - p[1]) * (point[0] - p[0]);
  });
  return signs.every((v) => v >= 0) || signs.every((v) => v <= 0);
}

function overlapsRect(points, rect) {
  const axes = [[1, 0], [0, 1]];
  for (let i = 0; i < points.length; i++) {
    const a = points[i], b = points[(i + 1) % points.length];
    axes.push([-(b[1] - a[1]), b[0] - a[0]]);
  }
  const corners = [[rect[0], rect[1]], [rect[2], rect[1]], [rect[2], rect[3]], [rect[0], rect[3]]];
  let separated = false;
  for (const [x, y] of axes) {
    if (!Number.isFinite(x) || !Number.isFinite(y) || (x === 0 && y === 0)) return null;
    const p = points.map(([px, py]) => px * x + py * y);
    const r = corners.map(([px, py]) => px * x + py * y);
    if (![...p, ...r].every(Number.isFinite)) return null;
    if (Math.max(...p) <= Math.min(...r) || Math.max(...r) <= Math.min(...p)) separated = true;
  }
  return !separated;
}
test('Fog Pier outer-face union, projection math, guards, emission order, and exact inverse', () => {
  const data = markedBlock(helper, 'DATA');
  const additions = markedBlock(helper, 'FUNCTION');
  const edgeText = data.match(/const OUTER_EDGES: Array = (\[[\s\S]*?\n\])/);
  assert.ok(edgeText, 'OUTER_EDGES declaration must exist');
  const edges = JSON.parse(edgeText[1].replaceAll("'", '"'));
  const sceneHash = data.match(/OUTER_SCENE_SHA256: String = '([0-9a-f]{64})'/)?.[1];
  const sceneSize = Number(data.match(/OUTER_SCENE_BYTES: int = (\d+)/)?.[1]);
  assert.equal(sceneSize, sceneBytes.length);
  assert.equal(sceneHash, sha256(sceneBytes));
  assert.equal(edges.length, 7);

  const candidates = [];
  for (const wall of scene.collision) {
    for (let i = 0; i < wall.polygon.length; i++) {
      const a = wall.polygon[i], b = wall.polygon[(i + 1) % wall.polygon.length];
      if ((a[0] === 24 && b[0] === 24) || (a[1] === 16 && b[1] === 16) || (a[1] === 6 && b[1] === 6)) {
        candidates.push(`${wall.id}|${pointKey(a, b)}`);
      }
    }
  }
  const actual = edges.map((e) => `${e.id}|${pointKey(e.a, e.b)}`);
  assert.deepEqual([...actual].sort(), [...candidates].sort(), 'table is the canonical +X/+Z exposed union');
  const coveredInner = ['pier_wall_north', 'pier_wall_west_north', 'pier_wall_west_south'].map((id) => {
    const start = helper.indexOf(`'id': '${id}'`);
    assert.ok(start >= 0, `${id} layout row exists`);
    const end = helper.indexOf('\n\t},', start);
    assert.ok(end > start, `${id} layout row is complete`);
    const line = helper.slice(start, end).match(/'inner_line': \[\[([^\]]+)\], \[([^\]]+)\]\]/);
    assert.ok(line, `${id} inner side is explicit`);
    const a = line[1].split(',').map(Number), b = line[2].split(',').map(Number);
    assert.ok(scene.collision.some((w) => w.id === id && w.polygon.some((p, i) => pointKey(p, w.polygon[(i + 1) % w.polygon.length]) === pointKey(a, b))));
    return `${id}|${pointKey(a, b)}`;
  });
  const exposedUnion = [...candidates, ...coveredInner].filter((edge) => !coveredInner.includes(edge));
  assert.deepEqual([...actual].sort(), [...exposedUnion].sort(), 'subtract the three existing inner-side faces');
  for (const edge of coveredInner) assert.ok(!actual.includes(edge), `${edge} is already covered`);
  const ownerCounts = edges.reduce((counts, edge) => {
    counts[edge.id] = (counts[edge.id] ?? 0) + 1;
    return counts;
  }, {});
  assert.ok(Object.values(ownerCounts).every((count) => count <= 2));
  assert.ok(edges.every((e) => scene.collision.some((w) => w.id === e.id && w.polygon.some((p, i) => pointKey(p, w.polygon[(i + 1) % w.polygon.length]) === pointKey(e.a, e.b))));
  const east = edges.filter((e) => e.a[0] === 24 && e.b[0] === 24).map((e) => [Math.min(e.a[1], e.b[1]), Math.max(e.a[1], e.b[1])]).sort((a, b) => a[0] - b[0]);
  assert.deepEqual(east, [[0, 0.5], [0.5, 6], [9.5, 15.5], [15.5, 16]]);
  assert.equal(east.some(([a, b]) => a < 9.5 && b > 6), false, 'the z=6..9.5 opening stays clear');

  const camera = { origin: [20, 8], center: [640, 400], ppm: 48 };
  assert.match(helper, /const RELIEF: float = 0\.25/);
  assert.match(helper, /const SIDE_COLOR: Color/);
  const quads = edges.map((e) => [
    project([e.a[0], 0, e.a[1]], camera), project([e.b[0], 0, e.b[1]], camera),
    project([e.b[0], 0.25, e.b[1]], camera), project([e.a[0], 0.25, e.a[1]], camera),
  ]);
  assert.equal(quads.flat().length, 28);
  assert.ok(quads.every(validQuad), 'all seven projected quads are finite, nonzero, and strictly convex');
  const counterexample = project([24, 0.125, 3], camera);
  assert.deepEqual(counterexample, [1072, 370]);
  assert.ok(counterexample[0] > 640 && counterexample[0] < 1600);
  const eastFace = edges.findIndex((e) => e.id === 'pier_wall_east_north' && e.a[1] === 0.5);
  assert.ok(eastFace >= 0 && containsConvex(quads[eastFace], counterexample));
  assert.equal(24 > 23.5 && 0.125 < 0.25, true, 'the point lies beyond the old inner side and below the cap');
  assert.equal(validQuad([[0, 0], [1, 0], [2, 0], [0, 1]]), false);
  assert.equal(validQuad([[0, 0], [1, 0], [1, Number.NaN], [0, 1]]), false);
  assert.equal(overlapsRect([[0, 20], [100, 20], [100, 80], [0, 80]], [0, 0, 100, 100]), true);
  assert.equal(overlapsRect([[100, 20], [120, 20], [120, 80], [100, 80]], [0, 0, 100, 100]), false, 'touching only is outside');
  assert.equal(overlapsRect([[101, 20], [120, 20], [120, 80], [101, 80]], [0, 0, 100, 100]), false);

  const context = functionBlock(helper, '_valid_context');
  const projectionGuard = functionBlock(helper, '_valid_projection');
  assert.ok(context.includes('P.snapshot') && context.includes('_valid_projection('));
  assert.ok(projectionGuard.includes('get_script() != ProjectionScript') && projectionGuard.includes("get('initialized')"));
  const binding = functionBlock(additions, '_outer_binding_matches');
  for (const anchor of ["get('entryToken')", "get('snapshot')", "get('scene')", "get('current_key')", "get('scene_hash')", "get('camera')", "get('viewport_rect')", 'get_viewport_rect()', 'is_visible_in_tree()', 'is_queued_for_deletion()']) assert.ok(binding.includes(anchor), `binding contains ${anchor}`);
  assert.ok(binding.includes('viewport.position != Vector2.ZERO') && binding.includes('canvas.get_viewport_rect() != viewport') && binding.includes("projection_value.get('viewport') != viewport.size"));
  const draw = functionBlock(additions, '_draw_outer_faces');
  for (const anchor of ['_outer_binding_matches(', 'OUTER_EDGES', '_outer_polygon_valid(points)', '_outer_visibility(points, viewport_value)', 'staged.size() != OUTER_EDGES.size()', 'emission.is_empty()', 'draw_colored_polygon(points, SIDE_COLOR)']) assert.ok(draw.includes(anchor), `draw planner contains ${anchor}`);
  assert.ok(draw.indexOf('_outer_polygon_valid(points)') < draw.indexOf('_outer_visibility(points, viewport_value)'));
  const finalGuard = draw.lastIndexOf('_outer_binding_matches(');
  const firstPaint = draw.indexOf('draw_colored_polygon(points, SIDE_COLOR)');
  assert.ok(finalGuard >= 0 && firstPaint >= 0 && finalGuard < firstPaint, 'final binding is rechecked before any new paint');
  assert.equal(/return\s+false/.test(draw), false, 'the extension cannot change draw_wall success/fallback');
  assert.equal((additions.match(/FileAccess\.open/g) ?? []).length, 1);
  assert.equal(/Texture|Time\.|randf|randomize|ResourceLoader/.test(additions), false, 'only canonical scene bytes are read');

  const wall = functionBlock(helper, 'draw_wall');
  const anchors = ['drawing_canvas.draw_colored_polygon(side_points, SIDE_COLOR)', '_draw_outer_faces(', 'drawing_canvas.draw_colored_polygon(cap_points, CAP_COLOR)', 'drawing_canvas.draw_polyline(outline_points, OUTLINE_COLOR, outline_width, true)', 'return true'];
  const positions = anchors.map((a) => wall.indexOf(a));
  assert.ok(positions.every((p) => p >= 0), 'legacy and extension anchors all exist');
  assert.deepEqual(positions, [...positions].sort((a, b) => a - b), 'new faces sit between unchanged side and cap');
  const callMatches = [...helper.matchAll(/^\t_draw_outer_faces\(drawing_canvas, snapshot, scene, current_key, projection, wall_data, viewport\) # FP_OUTER_CALL\r?$/gm)];
  assert.equal(callMatches.length, 1);

  let inverse = helper;
  for (const label of ['DATA', 'FUNCTION']) inverse = inverse.replace(new RegExp(`# FP_OUTER_${label}_BEGIN\\r?\\n[\\s\\S]*?# FP_OUTER_${label}_END\\r?\\n`), '');
  inverse = inverse.replace(/^\t_draw_outer_faces\(drawing_canvas, snapshot, scene, current_key, projection, wall_data, viewport\) # FP_OUTER_CALL\r?\n/m, '');
  const inverseBytes = Buffer.from(inverse, 'utf8');
  assert.equal(inverseBytes.length, 12398);
  assert.equal(sha256(inverseBytes), '5a2d0d3078d971236d17320eaf7adb3cbcb279b208088e0eb0e3f06427b610b7');
});
