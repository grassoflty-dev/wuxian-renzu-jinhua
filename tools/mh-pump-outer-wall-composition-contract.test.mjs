import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import test from 'node:test';

const root = process.cwd();
const helperPath = resolve(root, 'apps/godot_playable/scripts/world/mh_pump_environment_visual.gd');
const rendererPath = resolve(root, 'apps/godot_playable/scripts/world/renderer.gd');
const scenePath = resolve(root, 'apps/godot_playable/data/scenes/mh_pump_station.json');
const projectionPath = resolve(root, 'apps/godot_playable/scripts/world/projection.gd');
const baselineHelper = { bytes: 21610, sha256: '2f5ebfc890df85c507d9333bb30f2726979b6c012db70979bbeceb20f985bb48' };
const baselineRenderer = { bytes: 151597, sha256: 'e3699529a86292c788e827d6d1b6be6d40052058ba941967a04be11f121cebf3' };
const sha256 = (text) => createHash('sha256').update(text, 'utf8').digest('hex');
const helper = readFileSync(helperPath, 'utf8');
const renderer = readFileSync(rendererPath, 'utf8');
const sceneText = readFileSync(scenePath, 'utf8');
const scene = JSON.parse(sceneText);
const projectionSource = readFileSync(projectionPath, 'utf8');

function functionSource(source, name) {
  const start = source.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `missing function ${name}`);
  const next = source.slice(start + 1).search(/^func /m);
  return source.slice(start, next < 0 ? source.length : start + 1 + next);
}

test('Pump outer-wall source/math contract and exact legacy inverse', () => {
  assert.equal(Buffer.byteLength(sceneText), 7826);
  assert.equal(sha256(sceneText), '557c180648bc8c73621b444dd108f2207535750947e9a7738a5716dd1b2a5437');
  assert.equal(Buffer.byteLength(projectionSource), 3301);
  assert.equal(sha256(projectionSource), '45f518f6dddebaebedd6bc1b5eb66476ebd3f039779cb353cc6124fbc5c9c82e');
  assert.match(functionSource(projectionSource, 'project'), /Vector2\(\(d\.x-d\.z\)\*ppm,\(d\.x\+d\.z\)\*ppm\*0\.5-d\.y\*ppm\)/);
  const beginMarker = '# BEGIN Pump outer-wall composition additions\n';
  const endMarker = '# END Pump outer-wall composition additions\n';
  const begin = helper.indexOf(beginMarker);
  const end = helper.indexOf(endMarker, begin);
  assert.ok(begin >= 0 && end > begin, 'addition block markers must be present');
  const restoredHelper = helper.slice(0, begin) + helper.slice(end + endMarker.length);
  assert.equal(Buffer.byteLength(restoredHelper), baselineHelper.bytes);
  assert.equal(sha256(restoredHelper), baselineHelper.sha256);

  const rendererCall = '\tpackets = mh_pump_environment_visual.compose_outer_walls(packets,self,snapshot,scene,current_key,camera,viewport_rect)';
  const callAt = renderer.indexOf(rendererCall);
  assert.ok(callAt >= 0 && renderer.indexOf(rendererCall, callAt + 1) < 0, 'one consumer call must exist');
  const callLineStart = renderer.lastIndexOf('\n', callAt) + 1;
  const callLineEnd = renderer.indexOf('\n', callAt) + 1;
  assert.ok(callLineEnd > callAt, 'call line must terminate');
  const restoredRenderer = renderer.slice(0, callLineStart) + renderer.slice(callLineEnd);
  assert.equal(Buffer.byteLength(restoredRenderer), baselineRenderer.bytes);
  assert.equal(sha256(restoredRenderer), baselineRenderer.sha256);
  const consumer = functionSource(renderer, '_draw_mh_pump_packets');
  const statements = consumer.split('\n').slice(1).map((line) => line.trim()).filter(Boolean);
  assert.match(statements[0], /^packets = mh_pump_environment_visual\.compose_outer_walls\(/);
  assert.match(statements[1], /^for packet in packets:/);

  const tableMatch = helper.match(/const OUTER_WALL_EDGES = \[\n([\s\S]*?)\n\]/);
  assert.ok(tableMatch, 'edge table must exist');
  const rows = [...tableMatch[1].matchAll(/^\s*\{'wall_id':'([^']+)','a':Vector2\(([-\d.]+),([-\d.]+)\),'b':Vector2\(([-\d.]+),([-\d.]+)\)\},?$/gm)]
    .map((m) => ({ wall: m[1], a: [+m[2], +m[3]], b: [+m[4], +m[5]] }));
  const expected = [
    ['pump_wall_north', [24, 0], [24, 0.5]],
    ['pump_wall_south', [0, 16], [24, 16]],
    ['pump_wall_south', [24, 15.5], [24, 16]],
    ['pump_wall_west_north', [0, 6], [0.5, 6]],
    ['pump_wall_east_north', [24, 0.5], [24, 6]],
    ['pump_wall_east_north', [23.5, 6], [24, 6]],
    ['pump_wall_east_south', [24, 9.5], [24, 15.5]],
  ];
  assert.deepEqual(rows, expected.map(([wall, a, b]) => ({ wall, a, b })));
  const walls = new Map(scene.collision.map((row) => [row.id, row.polygon]));
  const samePoint = (a, b) => a[0] === b[0] && a[1] === b[1];
  for (const edge of rows) {
    const polygon = walls.get(edge.wall);
    assert.ok(polygon, `canonical owner ${edge.wall}`);
    const boundary = polygon.some((point, i) => {
      const next = polygon[(i + 1) % polygon.length];
      return (samePoint(point, edge.a) && samePoint(next, edge.b)) || (samePoint(point, edge.b) && samePoint(next, edge.a));
    });
    assert.ok(boundary, `edge belongs to canonical boundary ${edge.wall}`);
    if ((edge.a[0] === 0 && edge.b[0] === 0) || (edge.a[0] === 24 && edge.b[0] === 24)) {
      const low = Math.min(edge.a[1], edge.b[1]);
      const high = Math.max(edge.a[1], edge.b[1]);
      assert.ok(high <= 6 || low >= 9.5, 'side openings remain clear');
    }
  }

  const project = (x, y, z, ppm) => [(x - z) * ppm, ((x + z) * 0.5 - y) * ppm];
  const cross = (a, b, c) => (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
  const area2 = (points) => points.reduce((sum, p, i) => {
    const q = points[(i + 1) % points.length];
    return sum + p[0] * q[1] - p[1] * q[0];
  }, 0);
  for (const ppm of [24, 48, 96, 256]) {
    for (const edge of rows) {
      const a = project(edge.a[0], 0, edge.a[1], ppm);
      const b = project(edge.b[0], 0, edge.b[1], ppm);
      const points = [a, b, [b[0], b[1] - 12 * ppm / 48], [a[0], a[1] - 12 * ppm / 48]];
      const turns = points.map((point, i) => cross(point, points[(i + 1) % 4], points[(i + 2) % 4]));
      assert.ok(area2(points) !== 0 && turns.every((turn) => Number.isFinite(turn) && turn !== 0));
      assert.ok(turns.every((turn) => Math.sign(turn) === Math.sign(turns[0])));
    }
  }

  const wallsBlock = helper.match(/const WALLS = \[\n([\s\S]*?)\n\]/)?.[1];
  assert.ok(wallsBlock, 'legacy wall table must exist');
  const wallRows = [...wallsBlock.matchAll(/\{'id':'(pump_wall_[^']+)'.*?,'seams':(\d+),'footprint':Rect2\(/g)]
    .map((m) => ({ id: m[1], seams: Number(m[2]) }));
  assert.equal(wallRows.length, 6);
  const channelBuilder = functionSource(helper, '_add_channel_edge');
  const channelEdges = [...channelBuilder.matchAll(/^\s*\['(north|east_north|east_south|south|west)'/gm)];
  assert.equal(channelEdges.length, 5);
  const channelId = helper.match(/const CHANNEL_ID = '([^']+)'/)?.[1];
  assert.ok(channelId, 'legacy channel ID must exist');
  const legacyIds = [];
  for (const { id, seams } of wallRows) {
    legacyIds.push(`${id}:face`, `${id}:cap`, `${id}:fascia`, `${id}:weather`);
    for (let n = 1; n <= seams; n++) legacyIds.push(`${id}:seam:${n}`, `${id}:fastener:${n}`);
  }
  for (const [, edge] of channelEdges) legacyIds.push(`${channelId}:water_edge:${edge}`);
  const composedIds = [...legacyIds];
  const perOwner = new Map();
  rows.forEach((edge, index) => {
    const faceIndex = composedIds.indexOf(`${edge.wall}:face`);
    const capIndex = composedIds.indexOf(`${edge.wall}:cap`);
    assert.ok(faceIndex >= 0 && capIndex > faceIndex, 'positive owner-face/cap anchors');
    const insertion = faceIndex + 1 + (perOwner.get(edge.wall) ?? 0);
    assert.equal(insertion, capIndex, 'new face remains after owner face and before cap');
    composedIds.splice(insertion, 0, `${edge.wall}:outer_face:${index}`);
    perOwner.set(edge.wall, (perOwner.get(edge.wall) ?? 0) + 1);
  });
  const legacyCount = legacyIds.length;
  assert.equal(legacyCount, 61);
  assert.equal(composedIds.length, 68);
  assert.deepEqual(composedIds.filter((id) => !id.includes(':outer_face:')), legacyIds);

  const compose = functionSource(helper, 'compose_outer_walls');
  const context = functionSource(helper, '_outer_context_valid');
  assert.equal((compose.match(/_outer_context_valid\(/g) || []).length, 2, 'context is checked before staging and before return');
  assert.match(compose, /legacy_packets\s*!=\s*current_packets/);
  assert.match(compose, /legacy_packets\.duplicate\(true\)/);
  assert.match(compose, /var insert_index: int = face_index\+1\+prior_count/);
  assert.match(compose, /cap_index!=insert_index/);
  assert.match(compose, /composed\.insert\(insert_index,packet\)/);
  assert.doesNotMatch(compose, /return\s+\[\]/, 'all composition failures preserve legacy packets');
  const stagingAt = compose.indexOf('staged.append(one_packet[0])');
  const cloneAt = compose.indexOf('legacy_packets.duplicate(true)');
  const insertAt = compose.indexOf('composed.insert(insert_index,packet)');
  assert.ok(stagingAt >= 0 && cloneAt > stagingAt && insertAt > cloneAt, 'stage all additions before insertion');
  assert.match(context, /canvas_value\.get_viewport_rect\(\)\s*!=\s*viewport/);
  for (const binding of ['snapshot', 'scene', 'current_key', 'scene_hash', 'camera', 'viewport_rect']) {
    assert.ok(context.includes(`canvas_value.get('${binding}')`), `live ${binding} binding`);
  }
  assert.match(context, /_outer_json_equal\(scene_data,_outer_canonical_scene\)/);
  const projectionGuard = functionSource(helper, '_outer_projection_valid');
  assert.match(projectionGuard, /get_script\(\)!=ProjectionScript/);
  assert.match(projectionGuard, /initialized_value\)!=TYPE_BOOL/);
  assert.match(projectionGuard, /size==viewport\.size/);
});
