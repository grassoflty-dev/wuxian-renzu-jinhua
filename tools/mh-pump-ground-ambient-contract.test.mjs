import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const root = new URL('../', import.meta.url);
const read = p => readFileSync(new URL(p, root), 'utf8');
const source = read('apps/godot_playable/scripts/world/mh_pump_environment_visual.gd');
const scene = JSON.parse(read('apps/godot_playable/data/scenes/mh_pump_station.json'));
const projection = read('apps/godot_playable/scripts/world/projection.gd');
const LABEL = 'SOURCE_STATIC_MODEL_NOT_GDSCRIPT_RUNTIME_OR_VISUAL';

function literal(name) {
  const declaration = new RegExp('^const\\s+' + name + '(?:\\s*:[^=\\n]+)?\\s*=\\s*', 'm');
  const match = declaration.exec(source);
  assert.ok(match, 'missing constant ' + name);
  const from = match.index + match[0].length;
  const start = source.indexOf('[', from);
  assert.ok(start >= from);
  assert.equal(source.slice(from, start).trim(), '');
  let depth = 0, quote = '', escaped = false, comment = false, text = '';
  for (let i = start; i < source.length; i++) {
    const c = source[i];
    if (comment) {
      if (c !== '\n') continue;
      comment = false;
    }
    if (quote) {
      text += c;
      if (escaped) escaped = false;
      else if (c === '\\') escaped = true;
      else if (c === quote) quote = '';
      continue;
    }
    if (c === '#') { comment = true; continue; }
    if (c === '"' || c === "'") { quote = c; text += c; continue; }
    text += c;
    if (c === '[') depth++;
    if (c === ']' && --depth === 0) return JSON.parse(text);
  }
  assert.fail('unbalanced numeric literal ' + name);
}
function fn(name, text = source) {
  const match = new RegExp('^func\\s+' + name + '\\s*\\(', 'm').exec(text);
  assert.ok(match, 'missing function ' + name);
  const from = match.index + match[0].length;
  const next = text.slice(from).search(/^func\s+\w+\s*\(/m);
  return text.slice(match.index, next < 0 ? text.length : from + next);
}
function flat(name) { return fn(name).replace(/\s+/g, ''); }
function ordered(text, ...anchors) {
  let previous = -1;
  for (const anchor of anchors) {
    const index = text.indexOf(anchor);
    assert.ok(index >= 0, 'missing anchor ' + anchor);
    assert.ok(index > previous, 'out-of-order anchor ' + anchor);
    previous = index;
  }
}
function one(rows, predicate) {
  const found = rows.filter(predicate);
  assert.equal(found.length, 1);
  return found[0];
}
const rectangles = literal('AMBIENT_RECTS');
const rgb = literal('AMBIENT_RGB');
const alpha = literal('AMBIENT_ALPHA');
const sum = literal('AMBIENT_SUM');
const corners = r => [[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]];
const area = r => (r[2]-r[0])*(r[3]-r[1]);
const intersectArea = (a,b) => Math.max(0,Math.min(a[2],b[2])-Math.max(a[0],b[0]))*
  Math.max(0,Math.min(a[3],b[3])-Math.max(a[1],b[1]));
const close = (a,b) => assert.ok(Math.abs(a-b)<=2e-7, a + ' != ' + b);
const color = (x,z) => [...rgb,Math.fround(alpha[0]+(alpha[1]-alpha[0])*(x+z-sum[0])/sum[1])];
const sub = (a,b) => [a[0]-b[0],a[1]-b[1]];
const cross = (a,b) => a[0]*b[1]-a[1]*b[0];
const dot = (a,b) => a[0]*b[0]+a[1]*b[1];
const finite = p => p.length===2 && p.every(Number.isFinite);
const bounds = q => [Math.min(...q.map(p=>p[0])),Math.min(...q.map(p=>p[1])),
  Math.max(...q.map(p=>p[0])),Math.max(...q.map(p=>p[1]))];
function projected(r,ppm,f32=false) {
  return corners(r).map(([x,z]) => {
    const dx=x-12, dz=z-8, p=[640+(dx-dz)*ppm,400+(dx+dz)*ppm*0.5];
    return f32 ? p.map(Math.fround) : p;
  });
}
function valid(q) {
  if (q.length!==4 || !q.every(finite)) return false;
  const o=q[0], a=cross(sub(q[1],o),sub(q[2],o))+cross(sub(q[2],o),sub(q[3],o));
  if (!Number.isFinite(a)||a===0) return false;
  const b=bounds(q);
  if (!b.every(Number.isFinite)||b[2]<=b[0]||b[3]<=b[1]) return false;
  return q.every((p,i)=>{
    const e=sub(q[(i+1)%4],p), n=sub(q[(i+2)%4],q[(i+1)%4]), t=cross(e,n);
    return finite(e)&&e.some(v=>v!==0)&&Number.isFinite(t)&&t!==0&&(t>0)===(a>0);
  });
}
function overlap(q,r) {
  if (!valid(q)||!r.every(Number.isFinite)||r[2]<=r[0]||r[3]<=r[1]) return -1;
  const rc=corners(r),o=q[0],axes=[[1,0],[0,1],...q.map((p,i)=>{
    const e=sub(q[(i+1)%4],p); return [-e[1],e[0]];
  })];
  let separated=false;
  for (const axis of axes) {
    if (!finite(axis)||axis.every(v=>v===0)) return -1;
    const a=q.map(p=>dot(sub(p,o),axis)),b=rc.map(p=>dot(sub(p,o),axis));
    if (![...a,...b].every(Number.isFinite)) return -1;
    const [al,ah,bl,bh]=[Math.min(...a),Math.max(...a),Math.min(...b),Math.max(...b)];
    if (al>=ah||bl>=bh) return -1;
    if (ah<=bl||bh<=al) separated=true;
  }
  return separated?0:1;
}

test(LABEL + ': canonical land partition and affine palette',()=>{
  assert.equal(scene.worldId,'mist_harbor');
  assert.equal(scene.sceneId,'mh_pump_station');
  const walk=one(scene.walkablePolygons,r=>r.id==='mh_pump_station_walkable_area').polygon;
  const water=one(scene.terrainRegions,r=>r.id==='mh_pump_east_channel_water').polygon;
  assert.deepEqual(walk,[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]);
  assert.deepEqual(water,[[18,5.5],[23.5,5.5],[23.5,10.5],[18,10.5]]);
  const expected=[[0.5,0.5,18,15.5],[18,0.5,23.5,5.5],[18,10.5,23.5,15.5]];
  assert.deepEqual(rectangles,expected);
  assert.deepEqual(rgb,[0.08,0.13,0.16]); assert.deepEqual(alpha,[0.02,0.06]); assert.deepEqual(sum,[1,38]);
  const wb=[0.5,0.5,23.5,15.5], waterb=[18,5.5,23.5,10.5];
  for (const [i,r] of rectangles.entries()) {
    assert.ok(r.every(Number.isFinite)&&area(r)>0); assert.equal(intersectArea(r,waterb),0);
    for (const other of rectangles.slice(i+1)) assert.equal(intersectArea(r,other),0);
    for (const [x,z] of corners(r)) {
      const c=color(x,z); c.slice(0,3).forEach((v,k)=>close(v,rgb[k]));
      assert.ok(c[3]>=Math.fround(alpha[0])&&c[3]<=Math.fround(alpha[1]));
    }
  }
  close(rectangles.reduce((n,r)=>n+area(r),0),area(wb)-area(waterb));
  for (const [x,z] of [[0.5,0.5],[18,0.5],[18,5.5],[18,10.5],[23.5,15.5]])
    close(color(x,z)[3],Math.fround(alpha[0]+(alpha[1]-alpha[0])*(x+z-1)/38));
  const c=flat('_ambient_color');
  for (const token of ['x+z','AMBIENT_SUM[0]','AMBIENT_SUM[1]','AMBIENT_ALPHA[0]','AMBIENT_ALPHA[1]','AMBIENT_RGB'])
    assert.ok(c.includes(token));
  const cv=flat('_ambient_color_valid');
  for (const token of ['_material_color_finite(color)','Color(','AMBIENT_ALPHA[0]','AMBIENT_ALPHA[1]','.a'])
    assert.ok(cv.includes(token));
  const rgba=flat('_material_color_finite');
  for (const channel of ['r','g','b','a']) assert.ok(rgba.includes('is_finite(color.'+channel+')'));
});

test(LABEL + ': strict projected geometry and SAT model',()=>{
  assert.ok(fn('project',projection).replace(/\s+/g,'').includes(
    'Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)'));
  for (const ppm of [0.001,24,48,96,256]) for (const r of rectangles) {
    assert.ok(valid(projected(r,ppm))); assert.ok(valid(projected(r,ppm,true)));
  }
  assert.equal(valid(projected(rectangles[0],1e-12,true)),false);
  assert.equal(valid([[0,0],[0,0],[0,0],[0,0]]),false);
  assert.equal(valid([[0,0],[1,0],[1,NaN],[0,1]]),false);
  assert.equal(overlap(corners([0.1,0.1,0.9,0.9]),[0,0,1,1]),1);
  assert.equal(overlap(corners([1,0,2,1]),[0,0,1,1]),0);
  assert.equal(overlap(corners([2,0,3,1]),[0,0,1,1]),0);
  const q=flat('_ambient_quad_valid');
  assert.ok(q.includes('points[0]')&&q.includes('cross(')&&q.includes('is_finite('));
  assert.match(q,/area2(?:==|!=)0\.0/); assert.doesNotMatch(q,/0\.00000|1e-/i);
  const sat=flat('_ambient_overlap'); assert.ok(sat.includes('dot(')&&sat.includes('is_finite('));
  assert.match(sat,/<=/);
});

test(LABEL + ': capture and final live-context recheck',()=>{
  const context=fn('_outer_context_valid');
  for (const field of ['snapshot','scene','current_key','scene_hash','camera','viewport_rect'])
    assert.match(context,new RegExp('get\\([\'\"]'+field+'[\'\"]\\)'));
  for (const token of ['P.snapshot','entryToken','get_viewport_rect()','Vector2.ZERO','_outer_json_equal','_outer_projection_valid'])
    assert.ok(context.includes(token));
  assert.ok(context.includes("snapshot_data.has('entryToken')"));
  for (const binding of ["canvas_value.get('snapshot')!=snapshot_data","canvas_value.get('scene')!=scene_data",
    "canvas_value.get('current_key')!=current_key"]) assert.ok(context.includes(binding));
  const captureRaw=fn('_ambient_capture');
  assert.match(captureRaw,/var snapshot:\s*Dictionary\s*=\s*snapshot_value/);
  assert.match(captureRaw,/var scene:\s*Dictionary\s*=\s*scene_value/);
  assert.match(captureRaw,/'snapshot'\s*:\s*snapshot\.duplicate\(true\)/);
  assert.match(captureRaw,/'scene'\s*:\s*scene\.duplicate\(true\)/);
  const capture=flat('_ambient_capture');
  for (const token of ['_outer_context_valid(','is_visible_in_tree()','Transform2D.IDENTITY'])
    assert.ok(capture.includes(token));
  const match=flat('_ambient_matches');
  for (const token of ['_outer_context_valid(','_outer_json_equal(','is_visible_in_tree()',
    'get_instance_id()','Transform2D.IDENTITY','get_viewport_rect()']) assert.ok(match.includes(token));
  for (const binding of ['_outer_json_equal(snapshot_value,capture.snapshot)',
    '_outer_json_equal(scene_value,capture.scene)']) assert.ok(match.includes(binding));
  for (const key of ['canvas_id','snapshot','scene','key','scene_hash','camera','origin','center','ppm',
    'initialized','projection_viewport','viewport']) assert.ok(match.includes('capture.'+key));
  const camera=fn('_outer_projection_valid');
  for (const token of ['ProjectionScript','TYPE_BOOL','initialized','origin','screen_center','viewport','ppm'])
    assert.ok(camera.includes(token));
  const rect=flat('_ambient_rect_valid');
  for (const token of ['x0<0.5','z0<0.5','x1>23.5','z1>15.5','water_x<=0.0','water_z<=0.0'])
    assert.ok(rect.includes(token));
});

test(LABEL + ': staged optional draw preserves legacy success',()=>{
  const draw=flat('_draw_ground_ambient');
  ordered(draw,'_ambient_capture(','forrectangle:ArrayinAMBIENT_RECTS','_ambient_rect_valid(',
    '_ambient_color(','_ambient_quad_valid(','plan.append(','plan.size()!=3',
    'forcommand:Dictionaryinplan','_ambient_overlap(','visible.is_empty()','_ambient_matches(','draw_polygon(');
  assert.equal((draw.match(/\.draw_polygon\(/g)||[]).length,1);
  assert.match(draw,/points\.size\(\)!=4/); assert.match(draw,/colors\.size\(\)!=4/);
  assert.ok(draw.includes('_ambient_color_valid('));
  assert.ok(draw.includes('Vector3(point.x,0.0,point.y)'));
  const land=flat('draw_land_material');
  ordered(land,'_draw_material_commands(canvas_item,commands)',
    '_draw_ground_ambient(canvas,snapshot_value,scene_value,current_key_value,projection_value,viewport_value)','returntrue');
  assert.match(fn('draw_land_material'),/^\t_draw_ground_ambient\(/m);
  for (const name of ['_ambient_capture','_ambient_matches','_ambient_color','_ambient_color_valid',
    '_ambient_rect_valid','_ambient_quad_valid','_ambient_overlap','_draw_ground_ambient'])
    assert.doesNotMatch(fn(name),/\b(?:FileAccess|Image|ImageTexture|Time)\b|\b(?:load|preload|randf|randi)\s*\(/);
});
