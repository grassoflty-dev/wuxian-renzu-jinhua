import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const helper = readFileSync('apps/godot_playable/scripts/world/gh_deep_decon_environment_visual.gd', 'utf8');
const renderer = readFileSync('apps/godot_playable/scripts/world/renderer.gd', 'utf8').replace(/\r\n/g, '\n');
const canonical = JSON.parse(readFileSync('apps/godot_playable/data/scenes/gh_deep_decon.json', 'utf8'));

function constant(name, kind) {
  const prefix = `const ${name}: ${kind} = `;
  const matches = helper.split(/\r?\n/).filter(line => line.startsWith(prefix));
  assert.equal(matches.length, 1, `${name} production constant`);
  return JSON.parse(matches[0].slice(prefix.length));
}

function block(name, next) {
  const start = helper.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `${name} source anchor`);
  const end = helper.indexOf(`\nfunc ${next}(`, start);
  assert.ok(end > start, `${next} source boundary`);
  return helper.slice(start, end);
}

function project([x, z], y, ppm) { return [(x - z) * ppm, ((x + z) / 2 - y) * ppm]; }
function cross(a, b) { return a[0] * b[1] - a[1] * b[0]; }
function convex(points) {
  if (points.length !== 4 || points.flat().some(value => !Number.isFinite(value))) return false;
  const origin = points[0];
  let area = 0;
  for (let i = 1; i < 3; i++) area += cross([points[i][0] - origin[0], points[i][1] - origin[1]], [points[i + 1][0] - origin[0], points[i + 1][1] - origin[1]]);
  const turns = points.map((point, i) => {
    const next = points[(i + 1) % 4], after = points[(i + 2) % 4];
    return cross([next[0] - point[0], next[1] - point[1]], [after[0] - next[0], after[1] - next[1]]);
  });
  return Number.isFinite(area) && area !== 0 && turns.every(turn => Number.isFinite(turn) && turn !== 0 && (turn > 0) === (area > 0));
}

function sat(points, [x, y, width, height]) {
  const rect = [[x, y], [x + width, y], [x + width, y + height], [x, y + height]];
  const axes = [[1, 0], [0, 1], ...points.map((point, i) => {
    const next = points[(i + 1) % points.length];
    return [-(next[1] - point[1]), next[0] - point[0]];
  })];
  return axes.every(([ax, ay]) => {
    if (!Number.isFinite(ax) || !Number.isFinite(ay) || ax === 0 && ay === 0) return false;
    const p = points.map(point => point[0] * ax + point[1] * ay);
    const r = rect.map(point => point[0] * ax + point[1] * ay);
    return Math.max(...p) > Math.min(...r) && Math.max(...r) > Math.min(...p);
  });
}

test('Deep Decon boundary walls are canonical, clipped, staged and source-bound', () => {
  const plan = constant('WALL_PLAN', 'Array');
  assert.deepEqual(constant('CANONICAL_SCENE', 'Dictionary'), canonical);
  const expected = [
    { id: 'deep_decon_wall_north', collision_index: 0, sides: [{ a: [.5,.5], b: [23.5,.5], clip: true }, { a: [24,0], b: [24,.5], clip: false }], contact: [.5,.5,23.25,.53] },
    { id: 'deep_decon_wall_south', collision_index: 1, sides: [{ a: [0,16], b: [24,16], clip: false }, { a: [24,15.5], b: [24,16], clip: false }], contact: [] },
    { id: 'deep_decon_wall_west_north', collision_index: 2, sides: [{ a: [.5,.5], b: [.5,6.5], clip: false }, { a: [0,6.5], b: [.5,6.5], clip: false }], contact: [.5,.53,.53,6.5] },
    { id: 'deep_decon_wall_west_south', collision_index: 3, sides: [{ a: [.5,9.5], b: [.5,15.5], clip: true }], contact: [.5,9.5,.53,15.25] },
    { id: 'deep_decon_wall_east_north', collision_index: 4, sides: [{ a: [24,.5], b: [24,6.5], clip: false }, { a: [23.5,6.5], b: [24,6.5], clip: false }], contact: [] },
    { id: 'deep_decon_wall_east_south', collision_index: 5, sides: [{ a: [24,9.5], b: [24,15.5], clip: false }], contact: [] }
  ];
  assert.deepEqual(plan, expected);
  assert.equal(canonical.collision.length, 8, 'both alcove collisions remain in canonical context');
  assert.deepEqual(plan.map(row => row.id), canonical.collision.slice(0, 6).map(row => row.id));
  assert.ok(plan.every(row => row.collision_index < 6));
  assert.equal(plan.reduce((n, row) => n + row.sides.length, 0), 10);
  assert.equal(plan.filter(row => row.contact.length).length, 3);
  assert.equal(plan.reduce((n, row) => n + row.sides.length + 1 + (row.contact.length ? 1 : 0), 0), 19);
  assert.ok(plan.every(row => row.sides.length + 1 + (row.contact.length ? 1 : 0) <= 4));
  assert.equal(plan.flatMap(row => row.sides).filter(side => side.clip).length, 2);

  for (const row of plan) {
    const owner = canonical.collision[row.collision_index].polygon;
    for (const side of row.sides) {
      const mid = [(side.a[0] + side.b[0]) / 2, (side.a[1] + side.b[1]) / 2];
      const onOwnerEdge = owner.some((p, i) => {
        const q = owner[(i + 1) % owner.length];
        return (p[0] === q[0] && mid[0] === p[0] && mid[1] >= Math.min(p[1],q[1]) && mid[1] <= Math.max(p[1],q[1])) || (p[1] === q[1] && mid[1] === p[1] && mid[0] >= Math.min(p[0],q[0]) && mid[0] <= Math.max(p[0],q[0]));
      });
      assert.ok(onOwnerEdge, `${row.id} exposed segment belongs to its canonical wall`);
      if (row.id.includes('east') || row.id.includes('west')) assert.ok(!(Math.min(side.a[1],side.b[1]) < 9.5 && Math.max(side.a[1],side.b[1]) > 6.5), 'the east/west aperture stays open');
    }
  }

  const quads = [];
  for (const row of plan) {
    if (row.contact.length) {
      const [x0,z0,x1,z1] = row.contact;
      quads.push({ owner: row.id, kind: 'contact', world: [[x0,0,z0],[x1,0,z0],[x1,0,z1],[x0,0,z1]] });
    }
    for (const side of row.sides) {
      const dx = side.b[0] - side.a[0], dz = side.b[1] - side.a[1], length = Math.hypot(dx,dz);
      const lowerB = side.clip ? [side.b[0] - dx / length * .25, side.b[1] - dz / length * .25] : side.b;
      quads.push({ owner: row.id, kind: 'side', world: [[side.a[0],0,side.a[1]],[lowerB[0],0,lowerB[1]],[side.b[0],.25,side.b[1]],[side.a[0],.25,side.a[1]]] });
    }
    quads.push({ owner: row.id, kind: 'cap', world: canonical.collision[row.collision_index].polygon.map(([x,z]) => [x,.25,z]) });
  }
  assert.equal(quads.length, 19);
  assert.equal(quads.reduce((sum, quad) => sum + quad.world.length, 0), 76);
  for (const ppm of [24,48,96,256]) for (const quad of quads) assert.ok(convex(quad.world.map(([x,y,z]) => project([x,z],y,ppm))), `${quad.owner}/${quad.kind}@${ppm}`);

  const square = [[0,0],[2,0],[2,2],[0,2]];
  assert.equal(sat(square,[.5,.5,1,1]),true);
  assert.equal(sat(square,[2,0,1,2]),false,'touching is outside');
  assert.equal(sat(square,[3,0,1,2]),false);

  const context = block('_context_ok','_binding_ok');
  const binding = block('_binding_ok','_capture');
  const ownership = block('_owner_index','_project_world');
  const visible = block('_visible','_stage_plan');
  const staged = block('_stage_plan','draw_wall');
  const draw = helper.slice(helper.indexOf('func draw_wall('));
  for (const [name, text, anchors] of [
    ['context',context,['P.snapshot(snapshot)','entryToken','_same_json(scene,CANONICAL_SCENE)','SCENE_SHA256']],
    ['binding',binding,["canvas.get('snapshot')","canvas.get('scene')","canvas.get('current_key')","canvas.get('scene_hash')","canvas.get('camera')","canvas.get('viewport_rect')","canvas.get_viewport_rect()",'actual_viewport == viewport']],
    ['ownership',ownership,["wall.get('id') == plan.id",'CANONICAL_SCENE.collision[plan.collision_index]']],
    ['SAT',visible,['for axis: Vector2 in axes','polygon_max <= viewport_min or viewport_max <= polygon_min','separated = true','return 0 if separated else 1']],
    ['complete staging',staged,['if commands.size() != 19','ground_b-axis*HEIGHT if side_value.clip else ground_b',"'kind':'contact'","'kind':'side'","'kind':'cap'",'_quad_ok(command.points,command.color)','_visible(command.points,viewport)']]
  ]) {
    const positions = anchors.map(anchor => text.indexOf(anchor));
    assert.ok(positions.every(position => position >= 0), `${name} anchors exist`);
  }
  const satLoop = visible.indexOf('for axis: Vector2 in axes'), satSeparated = visible.indexOf('separated = true'), satReturn = visible.indexOf('return 0 if separated else 1');
  assert.ok(satLoop >= 0 && satSeparated >= 0 && satReturn >= 0 && satLoop < satSeparated && satSeparated < satReturn);
  assert.ok(helper.includes('const HEIGHT = 0.25') && helper.includes("const SIDE_COLOR: Color = Color('#384246')") && helper.includes("const CAP_COLOR: Color = Color('#4b5659')") && helper.includes('const CONTACT_COLOR: Color = Color(17.0/255.0,26.0/255.0,29.0/255.0,0.14)'));
  assert.ok(helper.includes('area == 0.0') && helper.includes('turn == 0.0') && helper.includes('commands.size() != 19'));
  const drawAnchors = ['var plan: Dictionary = _stage_plan(projection,viewport)','not _binding_ok(canvas_value,snapshot_value,scene_value,current_key_value,scene_hash_value,projection_value,viewport_value)','canvas.draw_colored_polygon(command.points,command.color)'];
  const drawPositions = [draw.indexOf(drawAnchors[0]),draw.lastIndexOf(drawAnchors[1]),draw.indexOf(drawAnchors[2])];
  assert.ok(drawPositions.every(position => position >= 0), 'staging, final binding and emission anchors exist');
  assert.ok(drawPositions[0] < drawPositions[1] && drawPositions[1] < drawPositions[2], 'full staging and final recheck precede first paint');
  assert.ok(draw.includes('_owner_index(wall_value)'));

  const preload = "const GhDeepDeconEnvironmentVisual = preload('res://scripts/world/gh_deep_decon_environment_visual.gd')";
  const instance = 'var gh_deep_decon_environment_visual = GhDeepDeconEnvironmentVisual.new()';
  const hook = 'if gh_deep_decon_environment_visual.draw_wall(self,snapshot,scene,current_key,scene_hash,camera,wall,viewport_rect):';
  for (const line of [preload,instance,hook]) assert.equal(renderer.split(line).length - 1, 1, `single caller addition: ${line}`);
  const loop = renderer.indexOf("for wall in scene.get('collision',[]):");
  const hookAt = renderer.indexOf(hook,loop);
  const fallback = renderer.indexOf("_poly(wall.polygon,0,Color('#4b5659'),true)",loop);
  assert.ok(loop >= 0 && hookAt > loop && fallback > hookAt);
  assert.equal(renderer.slice(hookAt + hook.length, fallback).trim(), 'continue', 'successful owner paint skips only the legacy fallback');
});
