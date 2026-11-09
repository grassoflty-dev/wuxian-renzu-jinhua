import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const source = readFileSync('apps/godot_playable/scripts/world/gh_gate_b_environment_visual.gd', 'utf8');
const scene = JSON.parse(readFileSync('apps/godot_playable/data/scenes/gh_gate_b.json', 'utf8'));
function declaration(name, end) {
  const match = source.match(new RegExp(`^const ${name}[^=]*=\\s*([\\s\\S]*?^${end})`, 'm'));
  assert.ok(match, `${name} exists`);
  return JSON.parse(match[1].replace(/'/g, '"'));
}
function fn(text, name) {
  const lines = text.split(/\r?\n/), start = lines.findIndex((line) => line.startsWith(`func ${name}(`));
  assert.ok(start >= 0, `${name} exists`);
  let end = start + 1;
  while (end < lines.length && !lines[end].startsWith('func ')) end++;
  return lines.slice(start, end).join('\n');
}
function at(text, re) { const m = re.exec(text); assert.ok(m, `${re} exists`); return m.index; }
const walls = declaration('WALLS', '\\}'), oldLong = declaration('EXPOSED_FACES', '\\}');
const table = declaration('SHORT_UNION_FACES', '\\]');
function span(f) {
  if (f.a[0] === f.b[0]) return [0,f.a[0],Math.min(f.a[1],f.b[1]),Math.max(f.a[1],f.b[1])];
  assert.equal(f.a[1],f.b[1]);
  return [1,f.a[1],Math.min(f.a[0],f.b[0]),Math.max(f.a[0],f.b[0])];
}
function key(f) { return JSON.stringify([f.owner,...span(f),f.low,f.high]); }
function covers(a,b) {
  const x=span(a), y=span(b);
  return x[0]===y[0] && x[1]===y[1] && x[2]<=y[2] && x[3]>=y[3] && a.low<=b.low && a.high>=b.high;
}
function derive() {
  const rects=[];
  for (const [owner,w] of Object.entries(walls)) {
    const matches=scene.collision.filter((r)=>r.id===owner); assert.equal(matches.length,1); const row=matches[0]; assert.deepEqual(w.polygon,row.polygon);
    const xs=row.polygon.map(p=>p[0]), zs=row.polygon.map(p=>p[1]);
    rects.push({owner,x0:Math.min(...xs),x1:Math.max(...xs),z0:Math.min(...zs),z1:Math.max(...zs),high:w.height});
  }
  const candidates=[];
  for(const r of rects) for(const axis of [0,1]) {
    const plane=axis===0?r.x1:r.z1, lo=axis===0?r.z0:r.x0, hi=axis===0?r.z1:r.x1;
    const neighbors=rects.filter(n=>n!==r&&(axis===0?n.x0:n.z0)===plane), cuts=new Set([lo,hi]);
    for(const n of neighbors) for(const end of (axis===0?[n.z0,n.z1]:[n.x0,n.x1])) cuts.add(Math.max(lo,Math.min(hi,end)));
    const points=[...cuts].sort((a,b)=>a-b);
    for(let i=0;i+1<points.length;i++) {
      const a=points[i],b=points[i+1],mid=(a+b)/2; if(a===b) continue;
      const touching=neighbors.filter(n=>(axis===0?n.z0:n.x0)<mid&&(axis===0?n.z1:n.x1)>mid);
      const low=Math.max(0,...touching.map(n=>n.high)); if(low>=r.high) continue;
      const lowOwner=touching.find(n=>n.high===low)?.owner||'';
      candidates.push({owner:r.owner,a:axis===0?[plane,a]:[a,plane],b:axis===0?[plane,b]:[b,plane],low,high:r.high,lowOwner});
    }
  }
  const covered=Object.entries(walls).map(([owner,w])=>({owner,a:w.inner[0],b:w.inner[1],low:0,high:w.height}));
  for(const [owner,ends] of Object.entries(oldLong)) covered.push({owner,a:ends[0],b:ends[1],low:0,high:walls[owner].height});
  return candidates.filter(f=>!covered.some(old=>covers(old,f)));
}
function project([x,y,z],ppm,origin=[0,0,0],center=[0,0]) {
  return [center[0]+((x-origin[0])-(z-origin[2]))*ppm,center[1]+((x-origin[0]+z-origin[2])/2-(y-origin[1]))*ppm];
}
function quad(f,ppm) {
  return [[f.a[0],f.low,f.a[1]],[f.b[0],f.low,f.b[1]],[f.b[0],f.high,f.b[1]],[f.a[0],f.high,f.a[1]]].map(p=>project(p,ppm));
}
function valid(p) {
  if(p.length!==4||p.some(q=>!q.every(Number.isFinite))) return false;
  const sub=(a,b)=>[a[0]-b[0],a[1]-b[1]], cross=(a,b)=>a[0]*b[1]-a[1]*b[0], o=p[0];
  const area=cross(sub(p[1],o),sub(p[2],o))+cross(sub(p[2],o),sub(p[3],o));
  if(!Number.isFinite(area)||area===0||Math.min(...p.map(q=>q[0]))>=Math.max(...p.map(q=>q[0]))||Math.min(...p.map(q=>q[1]))>=Math.max(...p.map(q=>q[1]))) return false;
  const turns=p.map((q,i)=>cross(sub(p[(i+1)%4],q),sub(p[(i+2)%4],p[(i+1)%4])));
  return turns.every(v=>Number.isFinite(v)&&v!==0&&Math.sign(v)===Math.sign(area));
}
function sat(p,r) {
  const axes=[[1,0],[0,1]], corners=[[r[0],r[1]],[r[2],r[1]],[r[2],r[3]],[r[0],r[3]]];
  for(let i=0;i<4;i++) axes.push([p[i][1]-p[(i+1)%4][1],p[(i+1)%4][0]-p[i][0]]);
  let separated=false;
  for(const a of axes) {
    if(!a.every(Number.isFinite)||a.every(v=>v===0)) return -1;
    const dot=q=>(q[0]-p[0][0])*a[0]+(q[1]-p[0][1])*a[1], x=p.map(dot), y=corners.map(dot);
    if(![...x,...y].every(Number.isFinite)) return -1;
    if(Math.max(...x)<=Math.min(...y)||Math.max(...y)<=Math.min(...x)) separated=true;
  }
  return separated?0:1;
}
test('SOURCE_STATIC_GEOMETRY_NOT_GDSCRIPT_VISUAL: five exposed faces and step',()=>{
  assert.equal(scene.worldId,'grey_hive'); assert.equal(scene.sceneId,'gh_gate_b');
  const faces=table.map(f=>({...f,low:f.low_owner?walls[f.low_owner].height:0,high:walls[f.owner].height,lowOwner:f.low_owner}));
  assert.equal(faces.length,5); assert.deepEqual(faces.map(key).sort(),derive().map(key).sort()); assert.equal(new Set(faces.map(key)).size,5);
  assert.deepEqual(faces.map(({owner,a,b,lowOwner})=>({owner,a,b,lowOwner})).sort((a,b)=>a.owner.localeCompare(b.owner)),derive().map(({owner,a,b,lowOwner})=>({owner,a,b,lowOwner})).sort((a,b)=>a.owner.localeCompare(b.owner)));
  let area=0;
  for(const f of faces) { area+=Math.hypot(f.b[0]-f.a[0],f.b[1]-f.a[1])*(f.high-f.low); for(const ppm of [.001,24,48,72,256]) assert.ok(valid(quad(f,ppm))); }
  assert.ok(Math.abs(area-1.15)<1e-12);
  const step=faces.find(f=>f.owner==='gh_gate_b_wall_west_south'); assert.ok(step); assert.equal(step.low,.2); assert.equal(step.high,.7);
  assert.deepEqual(step.a,[0,15.5]); assert.deepEqual(step.b,[.5,15.5]);
  for(const f of faces) for(const p of [f.a,f.b]) assert.ok(!(p[1]>6.5&&p[1]<9.5));
  const point=project([.25,.45,15.5],48,[3,0,12],[640,400]); assert.ok(Math.abs(point[0]-340)<1e-9&&Math.abs(point[1]-396.4)<1e-9);
  assert.equal(valid([[0,0],[1,0],[2,0],[0,1]]),false); assert.equal(valid([[0,0],[2,0],[.5,.5],[0,2]]),false);
  assert.equal(valid([[0,0],[1,0],[1,Infinity],[0,1]]),false);
  assert.equal(sat([[10,10],[20,10],[20,20],[10,20]],[0,0,100,100]),1);
  assert.equal(sat([[100,10],[110,10],[110,20],[100,20]],[0,0,100,100]),0);
  assert.equal(sat([[101,10],[110,10],[110,20],[101,20]],[0,0,100,100]),0);
  assert.equal(sat([[200,0],[300,0],[300,1e308],[200,1e308]],[0,0,100,100]),-1);
});
test('SOURCE_STATIC_ONLY: context chain, staging and cable/normal insertion',()=>{
  const bind=fn(source,'_short_bindings'), canonical=fn(source,'_short_canonical_matches'), capture=fn(source,'_short_capture');
  const prepare=fn(source,'_short_prepare'), recheck=fn(source,'_short_recheck'), apply=fn(source,'_short_apply'), wall=fn(source,'draw_wall');
  at(bind,/_valid_context\(/); at(bind,/is_inside_tree\(/); at(bind,/is_visible_in_tree\(/); at(bind,/is_queued_for_deletion\(/);
  at(bind,/get_viewport_rect\(/); at(bind,/Vector2\.ZERO/); at(bind,/entryToken/); at(bind,/get\('viewport'\)/); at(bind,/_short_canonical_matches\(/);
  assert.ok(at(bind,/entryToken/)<at(bind,/_valid_context\(/)); assert.ok(at(bind,/get_viewport_rect\(/)<at(bind,/_short_canonical_matches\(/));
  for(const field of ['snapshot','scene','current_key','scene_hash','camera','viewport_rect']) at(bind,new RegExp(`get\\(['"]${field}['"]\\)`));
  for(const [field,value] of [['snapshot','snapshot_value'],['scene','scene_value'],['key','current_key_value'],['hash','SHORT_SCENE_SHA256'],['camera','projection_value'],['viewport','viewport']]) at(bind,new RegExp(`bound_${field}\\s*!=\\s*${value}`));
  at(bind,/actual_viewport\s*!=\s*viewport/); at(bind,/projection_value\.get\('viewport'\)\s*!=\s*viewport\.size/);
  at(canonical,/FileAccess\.open/); at(canonical,/HashingContext\.HASH_SHA256/); at(canonical,/get_string_from_utf8/); at(canonical,/to_utf8_buffer/); at(canonical,/_short_json_equal/);
  at(canonical,/_short_scene_attempted/); assert.match(canonical,/_short_json_equal\(scene_value,_short_canonical_scene\)/);
  assert.match(source,/SHORT_SCENE_BYTES:\s*int\s*=\s*6855/); assert.match(source,/SHORT_SCENE_SHA256:\s*String\s*=\s*'a82c10a6fa50129c48267e255d2ca86f4ebaca43e0e0da81b639ee88228312c1'/);
  at(fn(source,'_valid_context'),/P\.snapshot\(/); at(fn(source,'_valid_projection'),/get_script\(\)\s*!=\s*ProjectionScript/);
  at(capture,/duplicate\(true\)/); for(const field of ['origin','screen_center','viewport','ppm','initialized']) at(capture,new RegExp(field));
  at(prepare,/_short_bindings\(/); for(const token of ['SHORT_UNION_FACES','_short_quad_valid','_short_visible','WALL_FACE']) at(prepare,new RegExp(token));
  assert.ok(at(prepare,/_short_quad_valid\(/)<at(prepare,/staged\.append\(/));
  assert.ok(at(prepare,/staged\.append\(/)<at(prepare,/for command: Dictionary in staged/));
  assert.ok(at(prepare,/for command: Dictionary in staged/)<at(prepare,/_short_recheck\(/));
  assert.equal(/_draw_commands\(|draw_colored_polygon\(/.test(prepare),false);
  const quadCheck=fn(source,'_short_quad_valid'), satCheck=fn(source,'_short_visible');
  at(quadCheck,/area == 0\.0/); at(quadCheck,/turn == 0\.0/); assert.equal(/EPSILON/.test(quadCheck),false);
  at(satCheck,/polygon_max <= view_min or view_max <= polygon_min/); at(satCheck,/separated = true/); at(satCheck,/return 0 if separated else 1/);
  at(recheck,/_short_bindings\(/); at(recheck,/_short_capture\(/);
  at(apply,/_short_prepare\(/); const insert=at(apply,/commands\.insert\(/), emit=at(apply,/_draw_commands\(/);
  const normalGuard=apply.lastIndexOf('_short_recheck(',insert); assert.ok(normalGuard>=0);
  assert.match(apply.slice(normalGuard,insert),/^_short_recheck\([\s\S]*?\): return\n\tcommands\.insert\(/);
  const cableGuard=apply.indexOf('_short_recheck('), cableEmit=apply.indexOf('_draw_commands(',cableGuard); assert.ok(cableGuard>=0&&cableEmit>cableGuard);
  assert.match(apply.slice(cableGuard,cableEmit),/^_short_recheck\([\s\S]*?\): return\n\t\t_draw_commands\(/);
  assert.ok(apply.indexOf('_short_recheck(')<emit); at(apply,/commands\.size\(\)\s*-\s*2/);
  const legacy=at(wall,/if EXPOSED_FACES\.has/), normal=at(wall,/# GB_SHORT_NORMAL/), paint=at(wall,/_draw_commands\(canvas,commands\)/); assert.ok(legacy<normal&&normal<paint);
  const cable=at(wall,/draw_wall_cable/), cableHook=at(wall,/# GB_SHORT_CABLE/), cap=at(wall,/_draw_commands\(canvas,cap_outline\)/);
  assert.ok(cable<cableHook&&cableHook<cap&&at(wall,/return true/)<normal);
});
