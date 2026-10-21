import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';

const root = new URL('../', import.meta.url);
const read = path => readFileSync(new URL(path, root));
const source = read('apps/godot_playable/scripts/world/cw_entry_foundry_environment_visual.gd')
  .toString('utf8').replace(/\r\n/g, '\n');
const sceneBytes = read('apps/godot_playable/data/scenes/cw_entry_foundry.json');
const scene = JSON.parse(sceneBytes);
const projection = read('apps/godot_playable/scripts/world/projection.gd').toString('utf8');
const pin = 'd90f852367b998cf5874b82fed583f1254664d4a88a8175bb3c94524b8f496b1';
const label = 'SOURCE_STATIC_GEOMETRY_NOT_GDSCRIPT_OR_VISUAL';
const compact = text => text.replace(/^\s*#.*$/gm, '').replace(/\s+/g, '');

function literal(name) {
  const match = new RegExp(
    '^const\\s+' + name + '(?:\\s*:\\s*[^=\\n]+)?\\s*=\\s*', 'm'
  ).exec(source);
  assert.ok(match, 'missing constant ' + name);
  const start = match.index + match[0].length;
  assert.ok('[{'.includes(source[start]), 'constant must be data: ' + name);
  const stack = [];
  let quote = '', escaped = false;
  for (let i = start; i < source.length; i++) {
    const c = source[i];
    if (quote) {
      if (escaped) escaped = false;
      else if (c === '\\') escaped = true;
      else if (c === quote) quote = '';
      continue;
    }
    if (c === "'" || c === '"') { quote = c; continue; }
    if (c === '[' || c === '{') stack.push(c);
    else if (c === ']' || c === '}') {
      assert.equal(stack.pop(), c === ']' ? '[' : '{', name);
      if (!stack.length) {
        return JSON.parse(source.slice(start, i + 1).replaceAll("'", '"'));
      }
    }
  }
  assert.fail('unterminated data constant ' + name);
}
const starts = [...source.matchAll(/^func\s+([A-Za-z_]\w*)\s*\(/gm)];
const functions = new Map(starts.map((m, i) =>
  [m[1], source.slice(m.index, starts[i + 1]?.index ?? source.length)]));
function fn(name) {
  assert.ok(functions.has(name), 'missing function ' + name);
  return functions.get(name);
}
function reachable(name) {
  const seen = new Set(), pending = [name], bodies = [];
  while (pending.length) {
    const next = pending.pop();
    if (seen.has(next)) continue;
    seen.add(next);
    const body = fn(next);
    bodies.push(body);
    for (const m of body.matchAll(/\b(_[A-Za-z_]\w*)\s*\(/g))
      if (functions.has(m[1]) && !seen.has(m[1])) pending.push(m[1]);
  }
  return bodies.join('\n');
}
function at(body, text) {
  const index = body.indexOf(text);
  assert.ok(index >= 0, 'missing source anchor ' + text);
  return index;
}
function before(body, first, second) {
  assert.ok(at(body, first) < at(body, second), first + ' before ' + second);
}
function alias(body, field, receiver = 'canvas') {
  const pattern = new RegExp(
    '\\bvar\\s+(\\w+)\\b[^\\n=]*=\\s*' + receiver +
    "\\.get\\('" + field + "'\\)");
  const match = pattern.exec(body);
  assert.ok(match, 'missing field binding ' + receiver + '.' + field);
  return match[1];
}
const expected = [
  {id:'north_east_end',owner:'cw_entry_foundry_wall_north',edge:[1,2],low_owner:''},
  {id:'south_east_end',owner:'cw_entry_foundry_wall_south',edge:[1,2],low_owner:''},
  {id:'east_north_jamb',owner:'cw_entry_foundry_wall_east_north',edge:[3,2],low_owner:''},
  {id:'west_south_step',owner:'cw_entry_foundry_wall_west_south',edge:[3,2],
    low_owner:'cw_entry_foundry_wall_south'}
];
const endpoints = [
  [[24,0],[24,0.5]], [[24,15.5],[24,16]],
  [[23.5,6],[24,6]], [[0,15.5],[0.5,15.5]]
];
const sub = (a,b) => [a[0]-b[0],a[1]-b[1]];
const cross = (a,b) => a[0]*b[1]-a[1]*b[0];
const finite = p => p.length === 2 && p.every(Number.isFinite);
function project([x,y,z], ppm = 1, center = [0,0]) {
  return [center[0]+(x-z)*ppm, center[1]+((x+z)/2-y)*ppm];
}
function area2(q) {
  return cross(sub(q[1],q[0]),sub(q[2],q[0])) +
    cross(sub(q[2],q[0]),sub(q[3],q[0]));
}
function valid(q) {
  if (q.length !== 4 || !q.every(finite)) return false;
  const a = area2(q);
  if (!Number.isFinite(a) || a === 0) return false;
  const xs = q.map(p=>p[0]), ys = q.map(p=>p[1]);
  if (!(Math.max(...xs)>Math.min(...xs) && Math.max(...ys)>Math.min(...ys)))
    return false;
  return q.every((p,i) => {
    const e = sub(q[(i+1)%4],p), n = sub(q[(i+2)%4],q[(i+1)%4]);
    const turn = cross(e,n);
    return finite(e) && (e[0]!==0 || e[1]!==0) &&
      Number.isFinite(turn) && turn!==0 && (turn>0)===(a>0);
  });
}
function sat(q, r) {
  if (!valid(q) || !r.every(Number.isFinite) || r[2]<=0 || r[3]<=0) return -1;
  const corners = [[r[0],r[1]],[r[0]+r[2],r[1]],
    [r[0]+r[2],r[1]+r[3]],[r[0],r[1]+r[3]]];
  if (!corners.every(finite)) return -1;
  const axes = [[1,0],[0,1],...q.map((p,i)=>{
    const e=sub(q[(i+1)%4],p); return [-e[1],e[0]];
  })];
  let separated = false;
  for (const axis of axes) {
    if (!finite(axis) || axis.every(v=>v===0)) return -1;
    const dots = points => points.map(p=>{
      const d=sub(p,q[0]); return d[0]*axis[0]+d[1]*axis[1];
    });
    const a=dots(q), b=dots(corners);
    if (![...a,...b].every(Number.isFinite)) return -1;
    const lo=Math.min(...a), hi=Math.max(...a);
    const vl=Math.min(...b), vh=Math.max(...b);
    if (!(lo<hi && vl<vh)) return -1;
    if (hi<=vl || vh<=lo) separated=true;
  }
  return separated ? 0 : 1;
}
function contains(q, point) {
  const turns=q.map((p,i)=>cross(sub(q[(i+1)%q.length],p),sub(point,p)));
  return turns.every(v=>v>=0) || turns.every(v=>v<=0);
}
function family() {
  const rows=literal('SHORT_UNION_FACES'), walls=literal('WALLS');
  assert.deepEqual(rows,expected);
  return rows.map((row,i)=>{
    const info=walls[row.owner];
    const canonical=scene.collision.find(w=>w.id===row.owner);
    assert.ok(canonical,row.owner);
    assert.deepEqual(info.polygon,canonical.polygon);
    const [a,b]=row.edge.map(index=>canonical.polygon[index]);
    assert.deepEqual([a,b],endpoints[i]);
    const low=row.low_owner ? walls[row.low_owner].height : 0;
    const high=info.height;
    assert.ok(high>low);
    assert.ok(Math.max(a[1],b[1])<=6 || Math.min(a[1],b[1])>=9.5);
    return {row,info,low,high,world:[
      [a[0],low,a[1]],[b[0],low,b[1]],
      [b[0],high,b[1]],[a[0],high,a[1]]
    ]};
  });
}

test(label + ': canonical family, projection and uncovered step',()=>{
  assert.equal(createHash('sha256').update(sceneBytes).digest('hex'),pin);
  assert.equal(scene.worldId,'clockworks');
  assert.equal(scene.sceneId,'cw_entry_foundry');
  assert.ok(compact(projection).includes(
    'Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)'));
  const faces=family(), walls=literal('WALLS');
  assert.equal(faces.length,4);
  assert.equal(faces.reduce((n,f)=>n+f.world.length,0),16);
  assert.deepEqual(faces.map(f=>[f.low,f.high]),[[0,1],[0,0.18],[0,0.18],[0.18,1]]);
  for (const f of faces) for (const ppm of [0.001,24,48,72,256]) {
    const q=f.world.map(p=>project(p,ppm));
    assert.ok(valid(q),f.row.id+' at '+ppm);
    const xs=q.map(p=>p[0]), ys=q.map(p=>p[1]);
    const box=[Math.min(...xs)-1,Math.min(...ys)-1,
      Math.max(...xs)-Math.min(...xs)+2,Math.max(...ys)-Math.min(...ys)+2];
    assert.equal(sat(q,box),1);
    assert.equal(sat(q,[Math.max(...xs),box[1],2,box[3]]),0);
    assert.equal(sat(q,[Math.max(...xs)+1,box[1],2,box[3]]),0);
  }
  const f=faces[3], point=project([0.25,(f.low+f.high)/2,15.5]);
  assert.ok(contains(f.world.map(p=>project(p)),point));
  const info=f.info, [ax,az,bx,bz]=info.inner, h=f.high, low=f.low;
  const end=bz-low;
  const side=[[ax,0,az],[bx,0,end],[bx,low,bz],[bx,h,bz],[ax,h,az]];
  const cap=info.polygon.map(([x,z])=>[x,h,z]);
  const [dx,dz]=info.contact.map(v=>v*0.05);
  const contact=[[ax,0,az],[bx,0,end],[bx+dx,0,end+dz],[ax+dx,0,az+dz]];
  const south=walls.cw_entry_foundry_wall_south;
  const southSide=[[0,0,15.5],[24,0,15.5],[24,low,15.5],[0,low,15.5]];
  const southCap=south.polygon.map(([x,z])=>[x,low,z]);
  for (const old of [side,cap,contact,southSide,southCap])
    assert.equal(contains(old.map(p=>project(p)),point),false);
  assert.equal(valid(f.world.map(p=>project(p,1,[1e30,1e30]))),false);
  assert.equal(valid([[0,0],[0,0],[0,0],[0,0]]),false);
  assert.equal(valid([[0,0],[1,0],[Infinity,1],[0,1]]),false);
});

test(label + ': current binding and deep capture chain',()=>{
  const raw=fn('_short_bindings'), b=compact(raw);
  const chain=compact(reachable('_short_bindings'));
  for (const field of ['snapshot','scene','current_key','scene_hash','camera','viewport_rect'])
    alias(raw,field);
  for (const field of ['snapshot','scene']) {
    const a=alias(raw,field);
    assert.ok(b.includes('not'+a+'isDictionary'));
    assert.ok(b.includes('_same_contract('+a+','+field+'_value)') ||
      b.includes('_same_contract('+field+'_value,'+a+')'));
  }
  const key=alias(raw,'current_key'), hash=alias(raw,'scene_hash');
  const camera=alias(raw,'camera'), viewport=alias(raw,'viewport_rect');
  assert.ok(b.includes(key+'!=current_key_value'));
  assert.ok(b.includes(hash+'!=SCENE_SHA256'));
  assert.ok(b.includes(camera+'!=camera_value'));
  assert.ok(b.includes(viewport+'!=viewport_value'));
  assert.ok(b.includes('viewport.position!=Vector2.ZERO'));
  assert.match(raw,/var\s+(\w+)[^\n=]*=\s*canvas_value\.get_viewport_rect\(\)/);
  const actual=/var\s+(\w+)[^\n=]*=\s*canvas_value\.get_viewport_rect\(\)/.exec(raw)[1];
  assert.ok(b.includes(actual+'!=viewport_value'));
  const cv=alias(raw,'viewport','camera_value');
  assert.ok(b.includes(cv+'!=viewport.size'));
  assert.ok(b.includes('is_finite('+cv+'.x)') && b.includes('is_finite('+cv+'.y)'));
  for (const text of ['P.snapshot(','_same_contract(scene,canonical_scene)',
    'get_script()!=ProjectionScript','TYPE_BOOL','is_visible_in_tree()',
    'is_queued_for_deletion()',"has('entryToken')","get('entryToken')!=null"])
    assert.ok(chain.includes(text),text);
  const capture=compact(fn('_short_capture'));
  assert.ok(capture.includes('snapshot_value.duplicate(true)'));
  assert.ok(capture.includes('scene_value.duplicate(true)'));
  const unchanged=compact(fn('_short_unchanged'));
  assert.ok(unchanged.includes('_short_bindings('));
  for (const field of ['snapshot','scene','origin','center','ppm','camera_viewport','viewport'])
    assert.ok(unchanged.includes(field),field);
  assert.match(unchanged,/_same_contract\([^)]*snapshot/);
  assert.match(unchanged,/_same_contract\([^)]*scene/);
});

test(label + ': complete staging, strict culling and unchanged fallback',()=>{
  const q=compact(fn('_short_quad_valid')), s=fn('_short_sat'), sc=compact(s);
  assert.ok(q.includes('points[1]-origin') && q.includes('points[2]-origin'));
  for (const text of ['is_finite(area2)','area2==0.0','turn==0.0','Vector2.ZERO'])
    assert.ok(q.includes(text),text);
  assert.ok(!q.includes('0.000001'));
  assert.ok(sc.includes('polygon_max<=rect_min') || sc.includes('polygon_max<=view_min'));
  assert.ok(sc.includes('rect_max<=polygon_min') || sc.includes('view_max<=polygon_min'));
  assert.match(s,/^\treturn 0 if separated else 1\s*$/m);
  assert.doesNotMatch(s,/^\t{2,}return 0(?:\s|$)/m);
  const stage=compact(fn('_short_plan'));
  for (const text of ['SHORT_UNION_FACES','WALLS[','.polygon','.edge','low_owner','_project('])
    assert.ok(stage.includes(text),text);
  before(stage,'_short_capture(','_project(');
  before(stage,'_short_quad_valid(','plans.append(');
  before(stage,'plans.size()!=4','_short_sat(');
  before(stage,'_short_sat(','_short_unchanged(');
  assert.ok(stage.includes('return{}'));
  const emit=compact(fn('_short_emit'));
  before(emit,'_short_unchanged(','draw_colored_polygon(');
  assert.ok(emit.includes('Color(0.18,0.22,0.24,1.0)'));
  const draw=compact(fn('draw_wall'));
  before(draw,'_short_plan(','draw_colored_polygon(contact_screen,');
  before(draw,'draw_colored_polygon(side_screen,','_short_emit(');
  before(draw,'_short_emit(','draw_colored_polygon(cap_screen,');
  assert.ok(!draw.includes('return_short_'));
  assert.ok(draw.endsWith('returntrue'));
});

