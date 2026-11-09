import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const read = (name) => readFileSync(path.join(root, name), 'utf8');
const helper = read('apps/godot_playable/scripts/world/mh_warehouse_crates_visual.gd');
const manifestBytes = readFileSync(path.join(root, 'apps/godot_playable/data/mh_warehouse_crates.internal-use.json'));
const manifest = JSON.parse(manifestBytes.toString('utf8'));
const scene = JSON.parse(read('apps/godot_playable/data/scenes/mh_tidal_warehouse.json'));
const warehouse = read('apps/godot_playable/scripts/world/mh_tidal_warehouse_environment_visual.gd');
const renderer = read('apps/godot_playable/scripts/world/renderer.gd');

function functionBody(source, name) {
  const anchor = source.indexOf('func ' + name + '(');
  assert.notEqual(anchor, -1, 'missing ' + name);
  const next = source.indexOf('\nfunc ', anchor + 1);
  return source.slice(anchor, next < 0 ? source.length : next);
}

function constantLine(name) {
  const match = helper.match(new RegExp('^const ' + name + ' = (.+)$', 'm'));
  assert.ok(match, 'missing ' + name);
  return match[1];
}

test('the admitted manifest is exact, private-use metadata and names only two fixed north-wall bodies', () => {
  assert.equal(manifestBytes.length, 1325);
  const expected = helper.match(/^const EXPECTED_JSON = '(.+)'$/m);
  assert.ok(expected);
  assert.deepEqual(JSON.parse(expected[1]), manifest);
  assert.equal(manifest.schema, 'wuxian.mh-warehouse-crates.internal-use/v1');
  assert.equal(manifest.registeredSourceReuse, true);
  assert.equal(manifest.approval, 'INTERNAL_REGISTERED_SOURCE_REUSE');
  assert.equal(manifest.GodotDisplay, 'NOT_RUN');
  assert.equal(manifest.publicRelease, 'NOT_ESTABLISHED');
  assert.equal(manifest.roomUse, 'NORTH_WALL_BEHIND_LID_DISPLAY_ONLY');
  assert.deepEqual(manifest.asset.canvas, [1238, 1233]);
  assert.equal(manifest.asset.bytes, 2067549);
  assert.equal(manifest.asset.sha256, '0dbad1364c14649b021f20510b9faf970ff317e6202131bd4cb2c32648b34b00');
  assert.deepEqual(manifest.asset.region, [0, 640, 540, 550]);
  assert.deepEqual(manifest.asset.anchor, [270, 550]);
  assert.deepEqual(manifest.asset.cutout, [[250,0],[539,20],[539,440],[295,549],[15,420],[15,20]]);
  assert.deepEqual(manifest.installation.instances.map((row) => row.position), [[6,0,0],[7.2,0,0]]);
});

test('the produced hexagon and UV/destination equations stay finite and strictly convex', () => {
  const points = JSON.parse(constantLine('CUTOUT'));
  assert.equal(points.length, 6);
  const uvs = points.map(([x, y]) => [x / 1238, (y + 640) / 1233]);
  assert.ok(uvs.flat().every((v) => Number.isFinite(v) && v >= 0 && v <= 1));
  let area = 0;
  for (let i = 1; i < points.length - 1; i++) {
    area += (points[i][0] - points[0][0]) * (points[i + 1][1] - points[0][1]) -
      (points[i][1] - points[0][1]) * (points[i + 1][0] - points[0][0]);
  }
  assert.notEqual(area, 0);
  const turns = points.map((p, i) => {
    const a = points[(i + 1) % 6].map((v, j) => v - p[j]);
    const q = points[(i + 2) % 6].map((v, j) => v - points[(i + 1) % 6][j]);
    return a[0] * q[1] - a[1] * q[0];
  });
  assert.ok(turns.every((turn) => Number.isFinite(turn) && turn !== 0 && Math.sign(turn) === Math.sign(area)));
  const origins = [...helper.matchAll(/Vector3\(([-\d.]+),([-\d.]+),([-\d.]+)\)/g)]
    .slice(0, 2).map((m) => m.slice(1).map(Number));
  assert.deepEqual(origins, [[6,0,0],[7.2,0,0]]);
  assert.match(helper, /foot\+\(q-Vector2\(270,550\)\)\*factor/);
  assert.match(helper, /\(q\+Vector2\(0,640\)\)\/Vector2\(1238,1233\)/);
});

test('live context and one-shot preparation guard all source and texture reads', () => {
  assert.equal(scene.worldId, 'mist_harbor');
  assert.equal(scene.sceneId, 'mh_tidal_warehouse');
  const north = scene.collision.find((row) => row.id === 'warehouse_wall_north');
  assert.deepEqual(north.polygon, [[0,0],[24,0],[24,0.5],[0,0.5]]);
  const live = functionBody(helper, '_live_context');
  for (const token of ["canvas.get('snapshot')", "canvas.get('scene')", "canvas.get('current_key')",
    "canvas.get('scene_hash')", "canvas.get('camera')", "canvas.get('viewport_rect')",
    'canvas.get_viewport_rect()', '_load_scene_once()', '_same(scene_value,_canonical_scene)']) assert.ok(live.includes(token), token);
  assert.ok(live.indexOf('canvas.get_viewport_rect()') < live.indexOf('_load_scene_once()'));
  assert.ok(live.includes('not _projection(camera_value,viewport_value)'));
  assert.ok(live.includes('viewport.position != Vector2.ZERO'));
  const context = functionBody(helper, '_context');
  assert.ok(context.includes('P.snapshot(snapshot)') && context.includes("snapshot.has('entryToken')"));
  assert.ok(context.includes("P.counter(snapshot.get('worldEpoch'),1)"));
  const projection = functionBody(helper, '_projection');
  assert.ok(projection.includes('camera_value.get_script() != ProjectionScript'));
  assert.ok(projection.includes('initialized_value != true'));
  const canonical = functionBody(helper, '_load_scene_once');
  assert.ok(canonical.includes('SCENE_BYTES') && canonical.includes('SCENE_SHA256'));
  assert.ok(canonical.includes('source_text.to_utf8_buffer() != bytes'));
  const prep = functionBody(helper, 'prepare_resources');
  assert.ok(prep.indexOf('_live_context(') >= 0);
  assert.ok(prep.indexOf('_live_context(') < prep.indexOf('_attempted = true'));
  assert.ok(prep.indexOf('_attempted = true') < prep.indexOf('_read(MAP_PATH'));
  assert.ok(prep.indexOf('_read(MAP_PATH') < prep.indexOf("_read('res://assets/props/grey_hive/supply_crates_sheet.png'"));
  assert.match(prep, /load_png_from_buffer/);
  assert.match(prep, /Image\.FORMAT_RGBA8/);
});

test('both bodies stage atomically and the optional adapter preserves the legacy wall path', () => {
  const draw = functionBody(helper, 'draw_north_backdrop');
  const firstGuard = draw.indexOf('_live_context(');
  const stage = draw.indexOf("commands.append({'points':points,'uvs':uvs})");
  const finalGuard = draw.lastIndexOf('_live_context(');
  const paint = draw.indexOf('canvas.draw_colored_polygon');
  assert.ok(firstGuard >= 0 && firstGuard < stage && stage < finalGuard && finalGuard < paint);
  assert.ok(draw.includes('commands.size() != 2'));
  assert.match(draw, /bounds\.position\.x < viewport\.end\.x/);
  assert.match(functionBody(helper, '_valid_body'), /area == 0\.0/);
  assert.match(functionBody(helper, '_valid_body'), /edge == Vector2\.ZERO/);
  assert.match(functionBody(helper, '_valid_body'), /turn == 0\.0/);
  const wall = functionBody(warehouse, 'draw_wall');
  const plan = wall.indexOf('_outer_face_plan(');
  const crate = wall.indexOf("id_value == 'warehouse_wall_north'");
  const contact = wall.indexOf('canvas.draw_colored_polygon(contact');
  const side = wall.indexOf('canvas.draw_colored_polygon(side');
  const outer = wall.indexOf('canvas.draw_colored_polygon(face_value.points');
  const cap = wall.indexOf('canvas.draw_colored_polygon(cap');
  assert.ok(plan >= 0 && plan < crate && crate < contact && contact < side && side < outer && outer < cap);
  assert.ok(wall.includes('crate_visual.draw_north_backdrop('));
  assert.ok(!wall.includes('if crate_visual.draw_north_backdrop'));
  assert.match(renderer, /MhWarehouseCratesVisual = preload/);
  assert.match(renderer, /mh_warehouse_crates_visual = MhWarehouseCratesVisual\.new\(\)/);
  assert.match(renderer, /mh_warehouse_crates_visual\.prepare_resources\(self,snapshot,scene,current_key,camera,viewport_rect\)/);
  assert.match(renderer, /mh_tidal_warehouse_environment_visual\.draw_wall\([^\n]*mh_warehouse_crates_visual\)/);
  const commit = functionBody(renderer, 'commit');
  const basePrep = commit.indexOf('mh_tidal_warehouse_environment_visual.prepare_resources(');
  const cratePrep = commit.indexOf('mh_warehouse_crates_visual.prepare_resources(');
  assert.ok(basePrep >= 0 && basePrep < cratePrep);
});
