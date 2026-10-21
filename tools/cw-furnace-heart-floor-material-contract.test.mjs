import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const root = process.cwd();
const helper = readFileSync(join(root, 'apps/godot_playable/scripts/world/cw_furnace_heart_floor_material_visual.gd'), 'utf8');
const renderer = readFileSync(join(root, 'apps/godot_playable/scripts/world/renderer.gd'), 'utf8');
const scene = JSON.parse(readFileSync(join(root, 'apps/godot_playable/data/scenes/cw_furnace_heart.json'), 'utf8'));
const functionBody = (source, name) => {
  const start = source.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `missing function ${name}`);
  const next = source.indexOf('\nfunc ', start + 1);
  return source.slice(start, next < 0 ? source.length : next);
};
const constant = (name) => {
  const match = helper.match(new RegExp(`^const ${name}: [^=]+ = ([^\\n]+)$`, 'm'));
  assert.ok(match, `missing production constant ${name}`);
  return match[1].trim();
};
const numberConstant = (name) => Number(constant(name));

test('canonical Heart floor layout and material grid are bounded', () => {
  assert.deepEqual(scene.boundsM, { x: 0, z: 0, width: 24, depth: 16 });
  const layers = scene.presentation.layers.filter((row) => row.id === 'visual.floor');
  assert.deepEqual(layers, [{ id: 'visual.floor', zGroup: 0 }]);
  const rows = scene.presentation.sprites.filter((row) => row.id === 'cw_furnace_heart_steel_floor' || row.id === 'cw_furnace_heart_factory_tiles');
  assert.deepEqual(rows.map(({ id, assetId, layer, position }) => ({ id, assetId, layer, position })), [
    { id: 'cw_furnace_heart_steel_floor', assetId: 'runtime2d.world.clockworks.floor_steel.v1', layer: 'visual.floor', position: [12, 0, 8] },
    { id: 'cw_furnace_heart_factory_tiles', assetId: 'runtime2d.world.clockworks.factory_tiles.v1', layer: 'visual.floor', position: [12, 0, 8] },
  ]);
  assert.equal(numberConstant('ROOM_WIDTH'), 24);
  assert.equal(numberConstant('ROOM_DEPTH'), 16);
  assert.equal(numberConstant('CELL_SIZE'), 4);
  assert.equal(numberConstant('CELL_COLUMNS'), 6);
  assert.equal(numberConstant('CELL_ROWS'), 4);
  assert.equal(numberConstant('JOINT_WIDTH'), 0.02);
  assert.equal(constant('VERTICAL_JOINTS'), '[4.0,8.0,12.0,16.0,20.0]');
  assert.equal(constant('HORIZONTAL_JOINTS'), '[4.0,8.0,12.0]');
  assert.match(helper, /Color\(52\.0 \/ 255\.0,58\.0 \/ 255\.0,60\.0 \/ 255\.0,1\.0\)/);
  assert.match(helper, /Color\(0\.04,0\.055,0\.06,0\.075\)/);
  assert.match(helper, /float\(variation_index\)\*0\.006/);
  const build = functionBody(helper, '_build_floor_plan');
  const cellLoop = build.indexOf('for row: int');
  const verticalLoop = build.indexOf('for joint_x: float');
  const horizontalLoop = build.indexOf('for joint_z: float');
  assert.ok(cellLoop >= 0 && verticalLoop > cellLoop && horizontalLoop > verticalLoop);
  assert.ok(build.indexOf("'cell'") < verticalLoop);
  assert.ok(build.lastIndexOf("'joint'") > horizontalLoop);
  for (const construction of ['float(column)*CELL_SIZE', 'float(row)*CELL_SIZE', "x0+CELL_SIZE,z0+CELL_SIZE,cell_color,'cell'", 'joint_x-JOINT_WIDTH*0.5', 'joint_x+JOINT_WIDTH*0.5', 'if x0 > 0.0', 'x0 += JOINT_WIDTH*0.5', 'if x1 < ROOM_WIDTH', 'x1 -= JOINT_WIDTH*0.5']) assert.ok(build.includes(construction), `missing production grid construction ${construction}`);
});

test('the production floor plan covers 24 cells and 23 nonoverlapping seams', () => {
  const width = numberConstant('ROOM_WIDTH');
  const depth = numberConstant('ROOM_DEPTH');
  const cell = numberConstant('CELL_SIZE');
  const columns = numberConstant('CELL_COLUMNS');
  const rows = numberConstant('CELL_ROWS');
  const seam = numberConstant('JOINT_WIDTH');
  const cells = [];
  for (let row = 0; row < rows; row++) for (let column = 0; column < columns; column++) cells.push([column * cell, row * cell, (column + 1) * cell, (row + 1) * cell]);
  assert.equal(cells.length, 24);
  assert.equal(cells.reduce((sum, [x0, z0, x1, z1]) => sum + (x1 - x0) * (z1 - z0), 0), width * depth);
  assert.deepEqual([Math.min(...cells.map((r) => r[0])), Math.min(...cells.map((r) => r[1])), Math.max(...cells.map((r) => r[2])), Math.max(...cells.map((r) => r[3]))], [0, 0, 24, 16]);
  const vertical = [4, 8, 12, 16, 20].map((x) => [x - seam / 2, 0, x + seam / 2, depth]);
  const horizontal = [];
  for (const z of [4, 8, 12]) for (let column = 0; column < columns; column++) {
    let x0 = column * cell;
    let x1 = x0 + cell;
    if (x0 > 0) x0 += seam / 2;
    if (x1 < width) x1 -= seam / 2;
    horizontal.push([x0, z - seam / 2, x1, z + seam / 2]);
  }
  assert.equal(vertical.length + horizontal.length, 23);
  for (const h of horizontal) for (const v of vertical) assert.ok(h[2] <= v[0] || v[2] <= h[0], 'horizontal seam overlaps a vertical crossing');
  const plan = functionBody(helper, '_plan_valid');
  assert.ok(plan.includes('plan.size() != 47'));
  assert.ok(plan.includes('cell_count == 24 and joint_count == 23 and vertex_count == 188'));
  assert.ok(plan.includes("kind == 'cell'") && plan.includes("kind == 'joint'"));
});

test('projected quads reject nonfinite, collapsed geometry and bind the live viewport', () => {
  const quad = functionBody(helper, '_quad_valid');
  assert.ok(quad.includes('if not is_finite(area2) or area2 == 0.0'));
  assert.ok(!quad.includes('MIN_AREA') && !quad.includes('epsilon'));
  assert.ok(quad.includes('if not _finite_v2(point)') && quad.includes('if not is_finite(term)'));
  assert.ok(quad.includes('maximum.x <= minimum.x or maximum.y <= minimum.y'));
  assert.ok(quad.includes('turn == 0.0') && quad.includes('edge == Vector2.ZERO'));
  const collapsed = [[0, 0], [1, 0], [2, 0], [3, 0]];
  const collapsedArea2 = collapsed.slice(1, 3).reduce((sum, point, index) => {
    const a = [point[0] - collapsed[0][0], point[1] - collapsed[0][1]];
    const b = [collapsed[index + 2][0] - collapsed[0][0], collapsed[index + 2][1] - collapsed[0][1]];
    return sum + a[0] * b[1] - a[1] * b[0];
  }, 0);
  assert.equal(collapsedArea2, 0);
  const context = functionBody(helper, '_context_matches');
  for (const guard of ["P.snapshot(snapshot)", "snapshot.get('entryToken') != null", "snapshot.get('worldId') != 'clockworks'", "snapshot.get('sceneId') != 'cw_furnace_heart'", 'P.counter(epoch,1)', "current_key != 'clockworks:cw_furnace_heart:'+str(epoch)", 'scene_hash != SCENE_SHA256']) assert.ok(context.includes(guard));
  const binding = functionBody(helper, '_canvas_matches');
  for (const field of ['snapshot', 'scene', 'current_key', 'scene_hash', 'camera', 'viewport_rect']) assert.ok(binding.includes(`canvas.get('${field}')`), `missing live canvas ${field} binding`);
  assert.ok(binding.includes('canvas.get_viewport_rect() != viewport'));
  assert.ok(binding.includes('canvas.is_visible_in_tree()') && binding.includes('canvas.is_queued_for_deletion()'));
  const projection = functionBody(helper, '_projection_valid');
  assert.ok(projection.includes('projection_value.get_script() != ProjectionScript'));
  assert.ok(projection.includes('camera_viewport_value == viewport.size'));
  assert.ok(projection.includes("projection_value.get('initialized')"));
  const draw = functionBody(helper, 'draw_floor');
  const staged = draw.indexOf('var plan: Array[Dictionary] = _build_floor_plan(projection)');
  const finalBinding = draw.lastIndexOf('_canvas_matches(canvas,snapshot,scene,current_key,scene_hash,projection,viewport)');
  const firstEmit = draw.indexOf('canvas.draw_colored_polygon');
  assert.ok(staged >= 0 && finalBinding > staged && firstEmit > finalBinding);
  assert.ok(draw.includes('floor_bounds.intersects(viewport)'));
  assert.ok(draw.includes('_context_matches(snapshot,scene,current_key,scene_hash)'));
  assert.doesNotMatch(helper, /FileAccess|ResourceLoader|Image\.load|Texture2D/);
});

test('Renderer draws the Heart material after the base and before later floor/world passes', () => {
  assert.equal((renderer.match(/CwFurnaceHeartFloorMaterialVisual = preload/g) ?? []).length, 1);
  assert.equal((renderer.match(/var cw_furnace_heart_floor_material_visual = CwFurnaceHeartFloorMaterialVisual\.new\(\)/g) ?? []).length, 1);
  const draw = functionBody(renderer, '_draw');
  assert.equal((draw.match(/cw_furnace_heart_floor_material_visual\.draw_floor/g) ?? []).length, 1);
  const base = draw.indexOf('_poly(floor_poly,0,floor_color,true)');
  const hook = draw.indexOf('cw_furnace_heart_floor_material_visual.draw_floor(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)');
  const shutdown = draw.indexOf('cw_shutdown_exit_environment_visual.draw_floor(self,snapshot,scene,current_key,scene_hash,camera,viewport_rect)');
  const collision = draw.indexOf("scene.get('collision',[])");
  assert.ok(base >= 0 && hook >= 0 && shutdown >= 0 && collision >= 0);
  assert.ok(base < hook && hook < shutdown && shutdown < collision);
});
