import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const LABEL='SOURCE_STATIC_FIELD_MODEL_NOT_GDSCRIPT_OR_VISUAL';
const root=new URL('../',import.meta.url);
const helper=readFileSync(new URL('apps/godot_playable/scripts/world/mh_tidal_warehouse_environment_visual.gd',root),'utf8');
const scene=JSON.parse(readFileSync(new URL('apps/godot_playable/data/scenes/mh_tidal_warehouse.json',root),'utf8'));
const projection=readFileSync(new URL('apps/godot_playable/scripts/world/projection.gd',root),'utf8');
const own=readFileSync(import.meta.url,'utf8');

function lex(s) {
  const code=s.split(''), raw=s.split('');
  const blank=(a,b,both)=>{for(let k=a;k<b;k++)if(s[k]!=='\n'&&s[k]!=='\r'){code[k]=' ';if(both)raw[k]=' ';}};
  for(let i=0;i<s.length;) {
    if(s[i]==='#'){let j=s.indexOf('\n',i);if(j<0)j=s.length;blank(i,j,true);i=j;continue;}
    if(s[i]==="'"||s[i]==='"') {
      const a=i,q=s[i++];let closed=false;
      assert(!s.startsWith(q.repeat(3),a),'unsupported triple-quoted literal');
      while(i<s.length) {
        if(s[i]==='\\'){assert(i+1<s.length);i+=2;continue;}
        if(s[i++]===q){closed=true;break;}
      }
      assert(closed,'unterminated literal');blank(a,i,false);continue;
    }
    i++;
  }
  return {code:code.join(''),raw:raw.join('')};
}
const H=lex(helper), J=lex(projection);
function blockEnd(code,a) {
  const stack=[],pairs={'}':'{',']':'[',')':'('};
  for(let i=a;i<code.length;i++) {
    const c=code[i];
    if('{[('.includes(c))stack.push(c);
    else if('}])'.includes(c)){assert.equal(stack.pop(),pairs[c]);if(!stack.length)return i+1;}
  }
  assert.fail('unclosed literal');
}
function fn(name,text=H) {
  const starts=[...text.code.matchAll(/^func\s+(\w+)\s*\(/gm)];
  const i=starts.findIndex(m=>m[1]===name);assert(i>=0,`missing function ${name}`);
  const a=starts[i].index,b=starts[i+1]?.index??text.code.length;
  return {raw:text.raw.slice(a,b),code:text.code.slice(a,b)};
}
function at(body,token){const i=body.raw.indexOf(token);assert(i>=0,`missing ${token}`);return i;}
function ordered(body,...tokens) {
  const positions=tokens.map(t=>at(body,t));
  for(let i=1;i<positions.length;i++)assert(positions[i]>positions[i-1],tokens.join(' -> '));
}
function literal(name) {
  const m=new RegExp(`^const\\s+${name}\\b[^=\\n]*=`,'m').exec(H.code);assert(m);
  const a=H.code.indexOf('{',m.index+m[0].length);assert(a>=0);
  assert(!H.raw.slice(m.index+m[0].length,a).trim());
  return JSON.parse(H.raw.slice(a,blockEnd(H.code,a)));
}
const field=literal('GROUND_AMBIENT_FIELD');
const newNames=['_ground_ambient_capture','_ground_ambient_current',
  '_ground_ambient_color','_ground_ambient_plan','_draw_ground_ambient'];
const keys=['snapshot','scene','frame','key','hash','camera',
  'origin','center','ppm','camera_viewport','viewport'];
const sub=(a,b)=>a.map((v,i)=>v-b[i]);
const dot=(a,b)=>a[0]*b[0]+a[1]*b[1];
const cross=(a,b)=>a[0]*b[1]-a[1]*b[0];
const finite=p=>Array.isArray(p)&&p.length===2&&p.every(Number.isFinite);
function bounds(p) {
  const lo=[Math.min(...p.map(v=>v[0])),Math.min(...p.map(v=>v[1]))];
  const hi=[Math.max(...p.map(v=>v[0])),Math.max(...p.map(v=>v[1]))];
  return {lo,hi};
}
function validQuad(p) {
  if(!Array.isArray(p)||p.length!==4||!p.every(finite))return false;
  const a=cross(sub(p[1],p[0]),sub(p[2],p[0]))+cross(sub(p[2],p[0]),sub(p[3],p[0]));
  if(!Number.isFinite(a)||a===0)return false;
  for(let i=0;i<4;i++) {
    const e=sub(p[(i+1)%4],p[i]),n=sub(p[(i+2)%4],p[(i+1)%4]),turn=cross(e,n);
    if(!finite(e)||(e[0]===0&&e[1]===0)||!Number.isFinite(turn)||turn===0||(turn>0)!==(a>0))return false;
  }
  const {lo,hi}=bounds(p),size=sub(hi,lo);
  return finite(lo)&&finite(hi)&&finite(size)&&size.every(v=>v>0);
}
function sat(p,r) {
  if(!validQuad(p)||!r.every(Number.isFinite)||r[2]<=0||r[3]<=0||
    !Number.isFinite(r[0]+r[2])||!Number.isFinite(r[1]+r[3]))return -1;
  const corners=[[r[0],r[1]],[r[0]+r[2],r[1]],[r[0]+r[2],r[1]+r[3]],[r[0],r[1]+r[3]]];
  const axes=[[1,0],[0,1]];
  for(let i=0;i<4;i++){const e=sub(p[(i+1)%4],p[i]);axes.push([-e[1],e[0]]);}
  let separated=false;
  for(const axis of axes) {
    if(!finite(axis)||(axis[0]===0&&axis[1]===0))return -1;
    const a=p.map(v=>dot(sub(v,p[0]),axis)),b=corners.map(v=>dot(sub(v,p[0]),axis));
    if(![...a,...b].every(Number.isFinite))return -1;
    const amin=Math.min(...a),amax=Math.max(...a),bmin=Math.min(...b),bmax=Math.max(...b);
    if(amin>=amax||bmin>=bmax)return -1;
    if(amax<=bmin||bmax<=amin)separated=true;
  }
  return separated?0:1;
}
assert(Array.isArray(scene.walkablePolygons)&&scene.walkablePolygons.length===1);
const walk=scene.walkablePolygons[0];
const polygon=walk.polygon;
const extent=bounds(polygon),span=sub(extent.hi,extent.lo);
const alpha=p=>field.alpha[0]+(field.alpha[1]-field.alpha[0])*
  dot(sub(p,extent.lo),field.axis)/dot(span,field.axis);
const project=(p,ppm)=>[(p[0]-p[1])*ppm,(p[0]+p[1])*ppm*0.5];

test(`${LABEL}: production-bound single affine field`,()=>{
  assert.equal(walk.id,'mh_tidal_warehouse_walkable_area');
  assert.deepEqual(polygon,[[0.5,0.5],[23.5,0.5],[23.5,15.5],[0.5,15.5]]);
  assert.deepEqual(field,{rgb:[0.05,0.10,0.13],alpha:[0.02,0.10],axis:[1,1]});
  const color=fn('_ground_ambient_color').code.replace(/\s/g,'');
  assert(color.includes('(Vector2(world.x,world.z)-minimum).dot(axis)/(maximum-minimum).dot(axis)'));
  assert(color.includes('GROUND_AMBIENT_FIELD.rgb[0]'));
  assert(color.includes('GROUND_AMBIENT_FIELD.rgb[1]'));
  assert(color.includes('GROUND_AMBIENT_FIELD.rgb[2]'));
  assert(color.includes('lerpf(GROUND_AMBIENT_FIELD.alpha[0],GROUND_AMBIENT_FIELD.alpha[1],t)'));
  const projected=fn('project',J).code.replace(/\s/g,'');
  assert(projected.includes('(d.x-d.z)*ppm'));
  assert(projected.includes('(d.x+d.z)*ppm*0.5-d.y*ppm'));
  const values=polygon.map(alpha);
  const expected=[0.02,0.06842105263157895,0.10,0.05157894736842105];
  values.forEach((v,i)=>assert(Math.abs(v-expected[i])<1e-14));
  assert(Math.abs(values[0]+values[2]-values[1]-values[3])<1e-14);
  for(const u of [0,.2,.5,.9,1])for(const v of [0,.3,.7,1]) {
    const p=[extent.lo[0]+span[0]*u,extent.lo[1]+span[1]*v];
    const bilinear=(1-u)*(1-v)*values[0]+u*(1-v)*values[1]+u*v*values[2]+(1-u)*v*values[3];
    assert(Math.abs(alpha(p)-bilinear)<1e-14);
    assert(alpha(p)>=field.alpha[0]&&alpha(p)<=field.alpha[1]);
  }
});

test(`${LABEL}: new quad finite, exact-nonzero and strict SAT model`,()=>{
  for(const ppm of [.001,24,48,96,256])assert(validQuad(polygon.map(p=>project(p,ppm))));
  const p=polygon.map(v=>project(v,48)),{lo,hi}=bounds(p);
  const middle=project([12,8],48);
  assert.equal(sat(p,[middle[0]-2,middle[1]-2,4,4]),1);
  assert.equal(sat(p,[hi[0],lo[1],10,hi[1]-lo[1]]),0);
  assert.equal(sat(p,[hi[0]+1,lo[1],10,10]),0);
  assert.equal(sat(p,[lo[0],lo[1],1,1]),0,'AABB overlap alone is insufficient');
  assert(!validQuad([[0,0],[0,0],[0,0],[0,0]]));
  assert(!validQuad([[0,0],[1,0],[NaN,1],[0,1]]));
  assert(!validQuad([[0,0],[1,1],[1,0],[0,1]]));
  assert.equal(sat(p,[0,0,Infinity,1]),-1);
  const plan=fn('_ground_ambient_plan');
  for(const token of ['walkablePolygons','_ground_ambient_color','_rect'])at(plan,token);
  ordered(plan,"camera.call('project'",'_outer_quad_valid','_outer_sat');
  assert(/Vector3\([^,\n]+,\s*0(?:\.0)?\s*,/.test(plan.raw),'ground Y remains zero');
});

test(`${LABEL}: capture and final current-context recheck source`,()=>{
  const capture=fn('_ground_ambient_capture'),current=fn('_ground_ambient_current');
  ordered(capture,'_outer_canvas_context','_frame_valid','duplicate(true)');
  at(capture,'Vector2.ZERO');at(current,'Vector2.ZERO');
  const actualKeys=[...capture.raw.matchAll(/['"]([a-z_]+)['"]\s*:/g)].map(m=>m[1]);
  assert.deepEqual([...new Set(actualKeys)].sort(),[...keys].sort());
  for(const k of ['snapshot','scene','frame'])
    assert(new RegExp(`\\b${k}\\.duplicate\\(\\s*true\\s*\\)`).test(capture.raw));
  at(current,'_outer_canvas_context');at(current,'_frame_valid');
  for(const key of keys)at(current,`capture.${key}`);
  at(fn('_outer_canvas_context'),'get_viewport_rect()');
  at(fn('_outer_canvas_context'),'_same(scene,OUTER_CANONICAL_SCENE)');
  const draw=fn('_draw_ground_ambient');
  ordered(draw,'_ground_ambient_capture','_ground_ambient_plan','_ground_ambient_current','.draw_polygon(');
  assert.equal([...draw.code.matchAll(/\.draw_polygon\s*\(/g)].length,1);
});

test(`${LABEL}: optional insertion preserves legacy floor result`,()=>{
  const floor=fn('draw_floor');
  const paint=at(floor,'canvas.draw_polygon(points,colors,uvs,texture)');
  const hook=at(floor,'_draw_ground_ambient(');
  const success=floor.raw.lastIndexOf('\treturn true');
  assert(success>=0&&paint<hook&&hook<success);
  assert(/^\t_draw_ground_ambient\(canvas_value,frame_value,snapshot,scene,key,camera,viewport\)\s*$/m.test(floor.raw));
  for(const name of newNames) {
    const body=fn(name);
    assert(!/\b(?:FileAccess|Image|ImageTexture|load|preload|_read|_hash|prepare_resources|Time|OS|randf|randi)\b/.test(body.code));
  }
  assert(own.trimEnd().split(/\r?\n/).length<=240);
});

test(`${LABEL}: captured-value mutation rejection model only`,()=>{
  const live={snapshot:{epoch:1},scene:{schema:1},frame:{floor:[1]},key:'room:1',hash:'canonical',
    camera:{},origin:[12,0,8],center:[800,500],ppm:48,camera_viewport:[1600,1000],viewport:[0,0,1600,1000]};
  const clone=v=>JSON.parse(JSON.stringify(v));
  const saved=Object.fromEntries(keys.map(k=>[k,k==='camera'?live[k]:clone(live[k])]));
  const matches=value=>keys.every(k=>k==='camera'?saved[k]===value[k]:JSON.stringify(saved[k])===JSON.stringify(value[k]));
  assert(matches(live));
  for(const key of keys) {
    const changed={...live};
    changed[key]=key==='camera'?{}:typeof live[key]==='number'?live[key]+1:
      typeof live[key]==='string'?`${live[key]}-changed`:Array.isArray(live[key])?[...live[key],1]:{changed:true};
    assert(!matches(changed),key);
  }
  live.snapshot.epoch=2;assert(!matches(live),'deep capture cannot share the live snapshot');
});
