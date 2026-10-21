import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import test from 'node:test';
const read = p => readFileSync(new URL('../' + p, import.meta.url));
const helper = read('apps/godot_playable/scripts/world/cw_pressure_environment_visual.gd').toString();
const renderer = read('apps/godot_playable/scripts/world/renderer.gd').toString();
const raw = read('apps/godot_playable/data/scenes/cw_pressure_hall.json');
const scene = JSON.parse(raw);
const sha = '53a7dc2925d5210da366abe8d03c07334c39cfe8b7da12f238090e27b1e5017a';
const marker = 'pressure_short_union_v1';
const clone = x => structuredClone(x);
const equal = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const norm = s => s.replace(/\s+/g, '');
function literal(name) {
  const line = helper.split(/\r?\n/).find(s => s.startsWith('const ' + name + ':'));
  assert.ok(line, 'Missing constant ' + name);
  return JSON.parse(line.slice(line.indexOf('=') + 1));
}
function methods(source) {
  const starts = [...source.matchAll(/^func ([A-Za-z_]\w*)\(/gm)];
  return new Map(starts.map((m, i) =>
    [m[1], source.slice(m.index, starts[i + 1]?.index ?? source.length)]));
}
function method(map, name) { assert.ok(map.has(name), 'Missing method ' + name); return map.get(name); }
function reachable(seed, map) {
  const queue = [seed], seen = new Set(), output = [];
  while (queue.length) {
    const body = queue.pop();
    output.push(body);
    const calls = body.replace(/^func [^\n]*\n/gm, '');
    for (const m of calls.matchAll(/\b([A-Za-z_]\w*)\s*\(/g)) {
      if (m[1] !== 'build_packets' && map.has(m[1]) && !seen.has(m[1])) {
        seen.add(m[1]);
        queue.push(map.get(m[1]));
      }
    }
  }
  return output.join('\n');
}
function cross(a, b, c) {
  return (b[0] - a[0]) * (c[1] - a[1]) -
    (b[1] - a[1]) * (c[0] - a[0]);
}
function area2(q) {
  let value = 0;
  for (let i = 1; i < q.length - 1; i++) value += cross(q[0], q[i], q[i + 1]);
  return value;
}
function quad(q) {
  if (!Array.isArray(q) || q.length !== 4 ||
      !q.every(p => p.length === 2 && p.every(Number.isFinite))) return false;
  let sign = 0;
  for (let i = 0; i < 4; i++) {
    const turn = cross(q[i], q[(i + 1) % 4], q[(i + 2) % 4]);
    if (!Number.isFinite(turn) || turn === 0 || (sign && sign * turn <= 0)) return false;
    sign = Math.sign(turn);
  }
  return Number.isFinite(area2(q)) && area2(q) !== 0;
}
function sat(q, r) {
  if (!quad(q) || r.length !== 4 || !r.every(Number.isFinite) ||
      r[2] <= 0 || r[3] <= 0) return -1;
  const [x, y, w, h] = r;
  const corners = [[x,y],[x+w,y],[x+w,y+h],[x,y+h]];
  const axes = [[1,0],[0,1]];
  for (let i = 0; i < 4; i++) {
    const a = q[i], b = q[(i + 1) % 4];
    axes.push([a[1] - b[1], b[0] - a[0]]);
  }
  let separated = false;
  for (const axis of axes) {
    if (!axis.every(Number.isFinite) || axis.every(v => v === 0)) return -1;
    const project = p => (p[0]-q[0][0])*axis[0] + (p[1]-q[0][1])*axis[1];
    const a = q.map(project), b = corners.map(project);
    if (!a.every(Number.isFinite) || !b.every(Number.isFinite)) return -1;
    const lo = Math.min(...a), hi = Math.max(...a);
    const rlo = Math.min(...b), rhi = Math.max(...b);
    if (lo >= hi || rlo >= rhi) return -1;
    if (hi <= rlo || rhi <= lo) separated = true;
  }
  return separated ? 0 : 1;
}
function project(p, ppm) {
  return [640+(p[0]-12-p[2]+8)*ppm,
    400+((p[0]-12+p[2]-8)*0.5-p[1])*ppm];
}
function points(row, ppm) {
  return [[row.a[0],row.low,row.a[1]],
    [row.b[0],row.low,row.b[1]],[row.b[0],row.high,row.b[1]],
    [row.a[0],row.high,row.a[1]]].map(p => project(p, ppm));
}
function compose(source, family) {
  if (family.length !== 4 || new Set(family.map(f => f.owner)).size !== 4 ||
      family.some(f => !quad(f.points) || sat(f.points,[0,0,1280,800]) < 0) ||
      source.some(p => p[marker])) return source;
  const result = [], seen = new Set();
  for (const packet of source) {
    const face = family.find(f => packet.id === f.owner + ':cap' &&
      packet.role === 'wall_cap' && packet.wall_id === f.owner);
    if (face) {
      if (seen.has(face.owner)) return source;
      seen.add(face.owner);
      if (face.visible) result.push({id:face.owner+':short_union_face',
        wall_id:face.owner,role:'wall_face',points:face.points,
        footprint:packet.footprint,color:[0.19,0.22,0.23,1],filled:true,[marker]:true});
    }
    result.push(packet);
  }
  return seen.size === 4 ? result : source;
}
// native records the actual P.snapshot result; this contract never executes GDScript.
function bound(q) {
  const s = q.snapshot, v = q.viewport;
  return q.native && q.live && q.visible && q.authentic && q.initialized &&
    s.worldId === 'clockworks' && s.sceneId === 'cw_pressure_hall' &&
    Number.isInteger(s.worldEpoch) && s.worldEpoch > 0 && s.entryToken === null &&
    q.key === 'clockworks:cw_pressure_hall:' + s.worldEpoch &&
    q.hash === sha && q.canvasHash === q.hash && q.canvasKey === q.key &&
    equal(q.scene,scene) && equal(q.canvasScene,q.scene) &&
    equal(q.canvasSnapshot,s) && q.canvasCamera === q.camera &&
    q.origin.every(Number.isFinite) && q.center.every(Number.isFinite) &&
    Number.isFinite(q.ppm) && q.ppm > 0 && q.ppm <= 256 &&
    v.every(Number.isFinite) && v[0] === 0 && v[1] === 0 && v[2] > 0 && v[3] > 0 &&
    equal(q.stored,v) && equal(q.actual,v) && equal(q.size,v.slice(2));
}
function current(frame, q, source, output) {
  return bound(q) && equal(frame.state,q) &&
    equal(frame.source,source) && equal(frame.output,output);
}

test('production source binds the four-face transaction to its consumer', () => {
  assert.equal(createHash('sha256').update(raw).digest('hex'), sha);
  assert.equal(literal('SHORT_SCENE_SHA'), sha);
  assert.deepEqual(literal('SHORT_CANONICAL_SCENE'), scene);
  assert.deepEqual(literal('SHORT_UNION_FACES'), [
    {id:'north_east_end',owner:'cw_pressure_hall_wall_north',a:[24,0],b:[24,0.5],low:0,high:0.25},
    {id:'south_east_end',owner:'cw_pressure_hall_wall_south',a:[24,15.5],b:[24,16],low:0,high:0.25},
    {id:'west_north_jamb',owner:'cw_pressure_hall_wall_west_north',a:[0,6],b:[0.5,6],low:0,high:0.25},
    {id:'east_north_jamb',owner:'cw_pressure_hall_wall_east_north',a:[23.5,6],b:[24,6],low:0,high:0.25}
  ]);
  assert.deepEqual(literal('SHORT_UNION_COLOR'), [0.19,0.22,0.23,1]);
  const h = methods(helper), r = methods(renderer);
  const stage = method(h,'stage_short_union'), guard = method(h,'short_union_current');
  const sr = reachable(stage,h), gr = reachable(guard,h), n = norm(sr + gr);
  assert.ok(n.includes('P.snapshot(snapshot)'));
  assert.ok(n.includes('_short_same(scene,SHORT_CANONICAL_SCENE)'));
  assert.ok(n.includes('P.number(actual,float(expected),float(expected))'));
  assert.ok(n.includes('actual.size()!=expected.size()'));
  for (const field of ['snapshot','scene','current_key','scene_hash','camera','viewport_rect']) {
    assert.match(sr + gr, new RegExp("get\\(['\"]" + field + "['\"]\\)"));
  }
  for (const token of ['worldEpoch','entryToken','is_instance_valid','is_inside_tree',
    'is_visible_in_tree','is_queued_for_deletion','get_script','ProjectionScript',
    'initialized','get_viewport_rect','SHORT_SCENE_SHA']) assert.ok(n.includes(token));
  for (const token of ['stored_viewport!=viewport','actual_viewport!=viewport',
    'projection_size==viewport.size']) assert.ok(n.includes(token));
  assert.ok(norm(sr).includes('build_packets(scene,snapshot,key,projection)'));
  assert.ok((sr.match(/duplicate\(true\)/g) || []).length >= 3);
  for (const token of ['source','output','origin','center','size','ppm']) assert.ok(gr.includes(token));
  assert.doesNotMatch((sr + gr).replace(/#.*$/gm,''), /\b(await|yield|FileAccess|call_deferred)\b/);
  const qb = [...h.values()].find(b => sr.includes(b) && /\barea2\b/.test(b) && /\bturn\b/.test(b));
  assert.ok(qb);
  assert.ok(norm(qb).includes('(points[index]-origin).cross(points[index+1]-origin)'));
  assert.match(qb, /area2\s*==\s*0(?:\.0)?/);
  assert.match(qb, /turn\s*==\s*0(?:\.0)?/);
  const sb = [...h.values()].find(b => sr.includes(b) && b.includes('Vector2.RIGHT') &&
    b.includes('polygon_min') && b.includes('separated'));
  assert.ok(sb);
  assert.match(sb, /polygon_max\s*<=\s*rect_min\s+or\s+rect_max\s*<=\s*polygon_min/);
  assert.match(sb, /return\s+0\s+if\s+separated\s+else\s+1/);
  assert.equal((sb.match(/return\s+0\b/g) || []).length, 1);
  const consumer = method(r,'_draw_cw_pressure_packets'), c = norm(consumer);
  for (const token of ['varshort_source:Array=packets','stage_short_union(',
    'packets=short_frame.packets','ifnotshort_checked:','short_union_current(',
    'short_checked=true','ifnotshort_allowed:continue',marker]) assert.ok(c.includes(token));
  const stageAt = consumer.indexOf('stage_short_union('), loop = consumer.indexOf('for packet in packets:');
  const guardAt = consumer.indexOf('short_union_current('), paint = consumer.indexOf('draw_colored_polygon(');
  for (const index of [stageAt,loop,guardAt,paint]) assert.ok(index >= 0);
  assert.ok(stageAt < loop && loop < guardAt && guardAt < paint);
  assert.ok(norm(consumer.slice(guardAt,paint)).includes('short_checked=true'));
  assert.doesNotMatch(consumer, /^\s*return\b/m);
  assert.match(helper, /func build_packets\(scene_data:\s*Dictionary,\s*snapshot:\s*Dictionary,\s*current_key:\s*String,\s*projection\)\s*->\s*Array/);
});

test('canonical short faces, translated geometry and owner-cap composition', () => {
  const ids = scene.collision.map(w => w.id), faces = literal('SHORT_UNION_FACES');
  assert.deepEqual(faces, [
    {id:'north_east_end',owner:ids[0],a:[24,0],b:[24,0.5],low:0,high:0.25},
    {id:'south_east_end',owner:ids[1],a:[24,15.5],b:[24,16],low:0,high:0.25},
    {id:'west_north_jamb',owner:ids[2],a:[0,6],b:[0.5,6],low:0,high:0.25},
    {id:'east_north_jamb',owner:ids[4],a:[23.5,6],b:[24,6],low:0,high:0.25}
  ]);
  for (const ppm of [24,48,72]) {
    let area = 0;
    for (const face of faces) {
      const q = points(face,ppm);
      assert.ok(quad(q));
      assert.ok(sat(q,[0,0,1280,800]) >= 0);
      area += Math.abs(area2(q)) / 2;
    }
    assert.ok(Math.abs(area - 0.5*ppm*ppm) < 1e-8);
  }
  const p = project([0.25,0.125,6],48);
  assert.ok(Math.abs(p[0]-172) < 1e-9 && Math.abs(p[1]-64) < 1e-9);
  const old = ids.flatMap(id => [{id:id+':face',role:'wall_face',wall_id:id},
    {id:id+':cap',role:'wall_cap',wall_id:id,footprint:[0,0,1,1]}]);
  const family = faces.map(f => ({...f,points:points(f,48),visible:true}));
  const joined = compose(old,family);
  assert.equal(joined.length, old.length+4);
  assert.ok(joined.filter(p => !p[marker]).every((p,i) => p === old[i]));
  for (const f of family) {
    const i = joined.findIndex(p => p[marker] && p.wall_id === f.owner);
    assert.ok(i >= 0);
    assert.equal(joined[i].footprint, old.find(p => p.id === f.owner+':cap').footprint);
    assert.equal(joined[i+1].id, f.owner+':cap');
  }
  assert.equal(compose(old.filter(p => p.id !== ids[0]+':cap'),family).length, old.length-1);
  const duplicate = [...old,old[1]];
  assert.equal(compose(duplicate,family), duplicate);
  const invalid = clone(family); invalid[0].points[0][0] = Infinity;
  assert.equal(compose(old,invalid), old);
  const square = [[0,0],[1,0],[1,1],[0,1]];
  assert.equal(sat(square,[1,0,1,1]),0);
  assert.equal(sat(square,[0,0,1,1]),1);
  assert.equal(sat(square,[0,0,0,1]),-1);
  assert.equal(sat([[0,0],[0,0],[1,1],[0,1]],[0,0,1,1]),-1);
  assert.equal(sat([[0,0],[2,0],[0.5,0.5],[0,2]],[0,0,1,1]),-1);
  assert.equal(sat([[0,0],[Infinity,0],[1,1],[0,1]],[0,0,1,1]),-1);
});

test('negative live bindings and captured-family mutations omit the whole extension', () => {
  const snapshot = {worldId:'clockworks',sceneId:'cw_pressure_hall',worldEpoch:2,entryToken:null,serverTick:4};
  const base = {native:true,live:true,visible:true,authentic:true,initialized:true,
    snapshot,canvasSnapshot:clone(snapshot),scene,canvasScene:clone(scene),
    key:'clockworks:cw_pressure_hall:2',canvasKey:'clockworks:cw_pressure_hall:2',
    hash:sha,canvasHash:sha,camera:'authentic-projection',canvasCamera:'authentic-projection',
    origin:[12,0,8],center:[640,400],ppm:48,size:[1280,800],
    viewport:[0,0,1280,800],stored:[0,0,1280,800],actual:[0,0,1280,800]};
  assert.ok(bound(base));
  const changes = [q=>q.native=false,q=>q.live=false,q=>q.visible=false,
    q=>q.authentic=false,q=>q.initialized=false,q=>q.snapshot.entryToken={},
    q=>q.snapshot.worldEpoch=0,q=>q.canvasHash='wrong',q=>q.canvasKey='wrong',
    q=>q.canvasCamera='other',q=>q.actual[3]--,q=>q.stored[2]--,
    q=>q.size[0]--,q=>q.ppm=0,q=>q.origin[0]=Infinity,
    q=>q.scene.boundsM.width=24.000001,q=>q.canvasSnapshot.serverTick++];
  for (const change of changes) {
    const q = clone(base); change(q); assert.equal(bound(q),false);
  }
  const source = [{id:'old'}], output = [{id:'old'},{id:'new',[marker]:true}];
  const frame = {state:clone(base),source:clone(source),output:clone(output)};
  assert.ok(current(frame,base,source,output));
  const moved = clone(base); moved.origin[0]++;
  assert.equal(current(frame,moved,source,output),false);
  assert.equal(current(frame,base,[{id:'changed'}],output),false);
  assert.equal(current(frame,base,source,[{id:'changed'}]),false);
  const accepted = false;
  const painted = output.filter(p => !p[marker] || accepted);
  assert.deepEqual(painted,[source[0]]);
});
