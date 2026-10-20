import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const helper = readFileSync(new URL('../apps/godot_playable/scripts/world/cw_pressure_environment_visual.gd', import.meta.url), 'utf8');
const scene = JSON.parse(readFileSync(new URL('../apps/godot_playable/data/scenes/cw_pressure_hall.json', import.meta.url), 'utf8'));
const protocol = readFileSync(new URL('../apps/godot_playable/scripts/bridge/protocol.gd', import.meta.url), 'utf8');
const projection = readFileSync(new URL('../apps/godot_playable/scripts/world/projection.gd', import.meta.url), 'utf8');
const edges = new Map([
  ['cw_pressure_hall_wall_south', { indices: [3, 2], points: [[0, 16], [24, 16]] }],
  ['cw_pressure_hall_wall_east_north', { indices: [1, 2], points: [[24, 0.5], [24, 6]] }],
  ['cw_pressure_hall_wall_east_south', { indices: [1, 2], points: [[24, 9.5], [24, 15.5]] }],
]);
const close = (a, b) => Math.abs(a - b) < 1e-9;
const project = ([x, y, z], ppm) => [(x - z) * ppm, ((x + z) / 2 - y) * ppm];

test('selected exterior edges match only the canonical wall boundaries and preserve the east opening', () => {
  assert.equal(scene.worldId, 'clockworks');
  assert.equal(scene.sceneId, 'cw_pressure_hall');
  for (const [id, edge] of edges) {
    const rows = scene.collision.filter(row => row.id === id);
    assert.equal(rows.length, 1, `${id} must be unique`);
    const polygon = rows[0].polygon;
    assert.deepEqual(edge.indices.map(index => polygon[index]), edge.points);
  }
  const northEnd = scene.collision.find(row => row.id === 'cw_pressure_hall_wall_east_north').polygon[2][1];
  const southStart = scene.collision.find(row => row.id === 'cw_pressure_hall_wall_east_south').polygon[1][1];
  assert.equal(southStart - northEnd, 3.5);
  assert.ok([...edges.get('cw_pressure_hall_wall_east_north').points, ...edges.get('cw_pressure_hall_wall_east_south').points]
    .every(([, z]) => z <= 6 || z >= 9.5));
});

test('projected quads are finite and nondegenerate at representative scales; south counterexample is covered', () => {
  for (const edge of edges.values()) for (const ppm of [24, 48, 96, 256]) {
    const [a, b] = edge.points;
    const ga = project([a[0], 0, a[1]], ppm);
    const gb = project([b[0], 0, b[1]], ppm);
    const rise = 12 * ppm / 48;
    const quad = [ga, gb, [gb[0], gb[1] - rise], [ga[0], ga[1] - rise]];
    assert.ok(quad.flat().every(Number.isFinite));
    const twiceArea = quad.reduce((sum, p, i) => sum + p[0] * quad[(i + 1) % 4][1] - quad[(i + 1) % 4][0] * p[1], 0);
    assert.notEqual(twiceArea, 0);
  }
  assert.ok(projection.includes('Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)'));
  const counterexample = project([12, 0.125, 16], 1);
  assert.ok(close(counterexample[0], -4) && close(counterexample[1], 13.875));
  assert.ok(counterexample[0] >= -16 && counterexample[0] <= 8);
  assert.ok(counterexample[1] >= 14 - 0.25 && counterexample[1] <= 14);
});

test('source binds and validates the conditional face before insertion while preserving legacy wall builders', () => {
  assert.ok(protocol.includes('static func snapshot(s) -> bool'));
  const mapBlock = helper.match(/const EXTERIOR_WALL_EDGES = \{([\s\S]*?)\n\}/);
  assert.ok(mapBlock, 'exterior edge map exists');
  for (const [id, edge] of edges) {
    const escaped = id.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    assert.match(mapBlock[1], new RegExp(`'${escaped}'\\s*:\\s*\\[${edge.indices[0]},\\s*${edge.indices[1]}\\]`));
  }
  const body = name => {
    const start = helper.indexOf(`func ${name}(`);
    assert.ok(start >= 0, `${name} source anchor exists`);
    const end = helper.indexOf('\nfunc ', start + 1);
    assert.ok(end > start, `${name} end anchor exists`);
    return helper.slice(start, end);
  };
  assert.equal(createHash('sha256').update(body('_add_wall')).digest('hex'), '387d761fbf363500b261e4e2ba9e863cac63da9e11f12a47abb2d6150aaf6784');
  assert.equal(createHash('sha256').update(body('_push')).digest('hex'), 'ecd8c31ae73e6064ef17a489b90676b63a6c756657bb5244a4f5c0c4c59de167');
  assert.equal(scene.collision.filter(row => row.id.startsWith('cw_pressure_hall_wall_')).length, 6);
  const build = body('build_packets');
  const anchors = {
    layout: build.indexOf('_ensure_layout(scene_data,snapshot,current_key)'), fullSnapshot: build.indexOf('P.snapshot(snapshot)'),
    nullToken: build.indexOf("snapshot.get('entryToken')==null"), projectionGuard: build.indexOf('_floor_projection_valid(projection)'),
    initialized: build.indexOf("projection.get('initialized')"), loop: build.indexOf('for wall in WALLS:'),
    start: build.indexOf('var wall_packet_start: int'), legacy: build.indexOf('if not _add_wall(packets,wall,projection):'),
    selected: build.indexOf('EXTERIOR_WALL_EDGES.has(wall.get(\'id\'))'),
    points: build.indexOf('_exterior_face_points(wall,projection)'), color: build.indexOf('not _floor_color_finite(exterior_color)'),
    push: build.indexOf("_push(exterior_packets,wall.id+':exterior_face','wall_face'"), insert: build.indexOf('packets.insert(wall_packet_start,exterior_packets[0])'),
  };
  for (const [name, index] of Object.entries(anchors)) assert.ok(index >= 0, `${name} source anchor exists`);
  assert.ok(anchors.layout < anchors.loop && anchors.fullSnapshot < anchors.loop && anchors.nullToken < anchors.loop && anchors.projectionGuard < anchors.loop && anchors.initialized < anchors.loop);
  assert.ok(anchors.loop < anchors.start && anchors.start < anchors.legacy && anchors.legacy < anchors.selected);
  assert.ok(anchors.selected < anchors.points && anchors.points < anchors.color && anchors.color < anchors.push && anchors.push < anchors.insert);
  assert.ok(build.includes('typeof(initialized_value)==TYPE_BOOL and initialized_value==true'));
  assert.ok(helper.includes("wall.id+':exterior_face','wall_face'"));
  assert.ok(build.includes('exterior_points,exterior_color,true,wall.id,wall.footprint'));
  const projectionGuard = body('_floor_projection_valid');
  assert.ok(projectionGuard.includes('projection.get_script()!=ProjectionScript'));
  const pointBuilder = body('_exterior_face_points');
  assert.ok(pointBuilder.includes('RELIEF_PIXELS*(float(ppm_value)/48.0)'));
  assert.ok(pointBuilder.includes('if not is_finite(point_a.x) or not is_finite(point_a.y)'));
  assert.ok(pointBuilder.includes('if not is_finite(twice_area) or twice_area==0.0'));
  assert.ok(pointBuilder.indexOf('if not is_finite(twice_area) or twice_area==0.0') < pointBuilder.indexOf('return points'));
  assert.ok(!/Time\.|FileAccess|ResourceLoader|load\(/.test(pointBuilder));
  assert.ok(!helper.includes('func draw_exterior'));
  const edgeKeys = [...mapBlock[1].matchAll(/'cw_pressure_hall_wall_/g)];
  assert.equal(edgeKeys.length, 3);
  assert.equal(scene.collision.length, 6);
});

// SOURCE_STATIC_GEOMETRY_NOT_GDSCRIPT_OR_VISUAL
