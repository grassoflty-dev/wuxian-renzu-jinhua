import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const helper = readFileSync('apps/godot_playable/scripts/world/mh_signal_yard_environment_visual.gd', 'utf8');
const scene = JSON.parse(readFileSync('apps/godot_playable/data/scenes/mh_signal_yard.json', 'utf8'));
const projectionSource = readFileSync('apps/godot_playable/scripts/world/projection.gd', 'utf8');
function section(start, end) {
  const a = helper.indexOf(start), b = helper.indexOf(end, a + start.length);
  assert.ok(a >= 0 && b > a, `missing source section ${start}`);
  return helper.slice(a, b);
}
function fn(name) {
  const start = helper.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `missing ${name}`);
  const end = helper.indexOf('\nfunc ', start + 1);
  return helper.slice(start, end < 0 ? helper.length : end);
}
const tableText = section('const OUTER_FACES:', 'const OUTER_SCENE_BYTES:');
const table = [...tableText.matchAll(/^\s*\{'id':'([^']+)','a':\[([\d.]+),([\d.]+)\],'b':\[([\d.]+),([\d.]+)\],'low_from':'([^']*)'\},?$/gm)]
  .map(m => ({ id: m[1], a: [+m[2], +m[3]], b: [+m[4], +m[5]], low: m[6] }));
const expected = [
  ['yard_wall_north',[24,0],[24,.5],''], ['yard_wall_south',[0,16],[24,16],''],
  ['yard_wall_south',[24,15.5],[24,16],''], ['yard_wall_west_north',[0,6],[.5,6],''],
  ['yard_wall_west_south',[0,15.5],[.5,15.5],'yard_wall_south'],
  ['yard_wall_east_north',[24,.5],[24,6],''], ['yard_wall_east_north',[23.5,6],[24,6],''],
  ['yard_wall_east_south',[24,9.5],[24,15.5],'']
];
function onSegment(p, a, b) {
  const dx = b[0]-a[0], dz = b[1]-a[1];
  return dx*(p[1]-a[1])-dz*(p[0]-a[0]) === 0 &&
    p[0] >= Math.min(a[0],b[0]) && p[0] <= Math.max(a[0],b[0]) &&
    p[1] >= Math.min(a[1],b[1]) && p[1] <= Math.max(a[1],b[1]);
}
function ownsEdge(face, polygon) {
  return polygon.some((a,i) => {
    const b = polygon[(i+1)%polygon.length];
    return onSegment(face.a,a,b) && onSegment(face.b,a,b);
  });
}
function project([x,y,z], ppm) { return [(x-z)*ppm, ((x+z)/2-y)*ppm]; }
function validConvex(p) {
  let area = 0;
  for (let i=1;i<3;i++) area += (p[i][0]-p[0][0])*(p[i+1][1]-p[0][1])-(p[i][1]-p[0][1])*(p[i+1][0]-p[0][0]);
  if (!Number.isFinite(area) || area === 0) return false;
  for (let i=0;i<4;i++) {
    const a=p[i], b=p[(i+1)%4], c=p[(i+2)%4];
    const ex=b[0]-a[0], ey=b[1]-a[1], fx=c[0]-b[0], fy=c[1]-b[1], turn=ex*fy-ey*fx;
    if (ex===0&&ey===0 || !Number.isFinite(turn) || turn===0 || Math.sign(turn)!==Math.sign(area)) return false;
  }
  return true;
}
function sat(p, r) {
  const rect=[[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]];
  const axes=[[1,0],[0,1],...p.map((a,i)=>{const b=p[(i+1)%p.length];return [-(b[1]-a[1]),b[0]-a[0]];})];
  let separated=false;
  for (const [x,y] of axes) {
    const pp=p.map(q=>q[0]*x+q[1]*y), rr=rect.map(q=>q[0]*x+q[1]*y);
    if (!(Math.max(...pp)>Math.min(...pp) && Math.max(...rr)>Math.min(...rr))) return -1;
    if (Math.max(...pp)<=Math.min(...rr)||Math.max(...rr)<=Math.min(...pp)) separated=true;
  }
  return separated?0:1;
}

test('eight canonical exterior faces preserve openings and cover the source-backed step', () => {
  assert.equal(table.length, 8);
  assert.deepEqual(table.map(f=>[f.id,f.a,f.b,f.low]), expected);
  const walls = new Map(scene.collision.map(w=>[w.id,w]));
  const wallLines = section('const WALLS:', 'const OUTER_FACES:').split('\n');
  const heights = new Map(wallLines.map(line=>{
    const id=line.match(/^\s*'([^']+)':/), h=line.match(/,([\d.]+)\],?$/);
    return id&&h?[id[1],+h[1]]:null;
  }).filter(Boolean));
  const projected=[];
  for (const face of table) {
    const row=walls.get(face.id); assert.ok(row,`missing canonical owner ${face.id}`);
    assert.ok(ownsEdge(face,row.polygon),`edge is not on ${face.id}`);
    const high=heights.get(face.id), low=face.low?heights.get(face.low):0;
    assert.ok(Number.isFinite(high)&&high>low);
    const highWall=face.id==='yard_wall_north'||face.id==='yard_wall_west_north'||face.id==='yard_wall_west_south';
    assert.equal(high,highWall ? 0.8 : 0.18);
    if(face.id==='yard_wall_west_south') { assert.equal(high-low,.62); assert.ok(low<.49&&.49<high); }
    const quad=[[face.a[0],low,face.a[1]],[face.b[0],low,face.b[1]],[face.b[0],high,face.b[1]],[face.a[0],high,face.a[1]]];
    for(const ppm of [24,48,96]) { const p=quad.map(v=>project(v,ppm)); assert.ok(validConvex(p)); if(ppm===48) projected.push(p); }
    if(face.a[0]===24||face.b[0]===24||face.a[0]===0||face.b[0]===0) {
      const lo=Math.min(face.a[1],face.b[1]), hi=Math.max(face.a[1],face.b[1]);
      assert.ok(hi<=6||lo>=9.5,'outer face enters a side opening');
    }
  }
  assert.equal(projected.length,8);
  assert.equal(18+table.length,26);
  const counts=new Map(); for(const f of table) counts.set(f.id,(counts.get(f.id)||3)+1);
  assert.ok(Math.max(...counts.values())<=5);
  assert.ok(table.some(f=>f.id==='yard_wall_west_south'&&onSegment([.25,15.5],f.a,f.b)));
});

test('projection SAT treats positive overlap strictly and rejects touching-only visibility', () => {
  assert.match(projectionSource,/Vector2\(\(d\.x-d\.z\)\*ppm,\(d\.x\+d\.z\)\*ppm\*0\.5-d\.y\*ppm\)/);
  const p=[[0,0],[2,0],[2,2],[0,2]];
  assert.equal(sat(p,[1,1,3,3]),1);
  assert.equal(sat(p,[2,0,4,2]),0);
  assert.equal(sat(p,[2.01,0,4,2]),0);
});

test('extension is fully staged, context-bound, and emitted before only the existing cap', () => {
  const loader=fn('_load_outer_scene_once'), binding=fn('_outer_context_matches'), quad=fn('_outer_quad_valid'), visibility=fn('_outer_visibility'), plan=fn('_outer_plan'), draw=fn('draw_wall');
  const base=fn('_base'), view=fn('_view');
  assert.match(loader,/res:\/\/data\/scenes\/mh_signal_yard\.json/); assert.match(loader,/if _outer_scene_attempted:/); assert.match(loader,/_outer_scene_attempted = true/);
  assert.match(loader,/OUTER_SCENE_BYTES/); assert.match(loader,/OUTER_SCENE_SHA256/); assert.match(loader,/to_utf8_buffer\(\)!=bytes/); assert.match(loader,/JSON\.parse_string/); assert.match(loader,/_outer_scene = decoded/);
  assert.match(base,/P\.snapshot\(snapshot_value\)/); assert.match(base,/entryToken/); assert.match(view,/get_script\(\)!=ProjectionScript/); assert.match(view,/initialized/);
  for(const field of ['snapshot','scene','current_key','scene_hash','camera','viewport_rect']) assert.ok(binding.includes(`get('${field}')`),`missing canvas ${field} binding`);
  for(const guard of ['is_visible_in_tree','get_viewport_rect','Vector2.ZERO','_view(','_equal(scene_value,_outer_scene)']) assert.ok(binding.includes(guard),`missing ${guard}`);
  assert.match(quad,/area2==0\.0/); assert.match(quad,/turn==0\.0/); assert.match(quad,/edge==Vector2\.ZERO/);
  const validateAll=plan.indexOf('_outer_quad_valid(points)'), stageCandidate=plan.indexOf('candidates.append(');
  const cullLoop=plan.indexOf('for candidate: Dictionary in candidates:'), cull=plan.indexOf('_outer_visibility(points,viewport)');
  assert.ok(validateAll>=0&&stageCandidate>=0&&validateAll<stageCandidate);
  assert.ok(cullLoop>=0&&cull>=0&&cullLoop<cull);
  assert.match(plan,/for face: Dictionary in OUTER_FACES/); assert.match(plan,/return \{\}/);
  assert.match(visibility,/p_max<=v_min or v_max<=p_min/);
  const first=draw.indexOf('_outer_context_matches(canvas,snapshot,scene,key,projection,viewport)');
  const stage=draw.indexOf('_outer_plan(projection,viewport)');
  const recheck=draw.indexOf('_outer_context_matches(canvas,snapshot,scene,key,projection,viewport)',first+1);
  const incorporate=draw.indexOf('plan.append_array(outer_for_wall)');
  const emission=draw.indexOf('canvas.draw_colored_polygon(points,colors[1])');
  const cap=draw.indexOf('canvas.draw_colored_polygon(plan[index],colors[index])');
  assert.ok(first>=0&&stage>first&&recheck>stage&&incorporate>recheck&&emission>incorporate&&cap>emission);
  assert.match(draw,/for world: Array\[Vector3\] in \[contact_world,face_world,cap_world\]/);
  assert.match(draw,/for index: int in range\(3\):/);
  assert.match(draw,/if index==2:/);
  assert.ok(!draw.slice(first,incorporate).includes('return false'),'extension errors must preserve the legacy wall path');
});
