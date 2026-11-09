import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import test from 'node:test';

const root=fileURLToPath(new URL('../',import.meta.url));
const world='apps/godot_playable/scripts/world/';
const hp=world+'gh_deep_decon_alcove_visual.gd', rp=world+'renderer.gd';
function read(path){return readFileSync(new URL('../'+path,import.meta.url));}
function hash(bytes){return createHash('sha256').update(bytes).digest('hex');}
function literal(source,name){for(const line of source.split('\n')){const prefix='const '+name+': ';if(line.startsWith(prefix))return JSON.parse(line.slice(line.indexOf('=')+1));}assert.fail('Missing '+name);}
function body(source,name){const start=source.indexOf('\nfunc '+name+'(');assert.ok(start>=0);const end=source.indexOf('\nfunc ',start+1);return source.slice(start,end<0?source.length:end);}
function project(p,ppm){return [800+(p[0]-12-p[2]+8)*ppm,500+((p[0]-12+p[2]-8)*0.5-p[1])*ppm];}
function screen(points,ppm){return points.map(p=>project(p,ppm));}
function cross(a,b,c){return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]);}
function area(q){let sum=0;for(let i=1;i<q.length-1;i++)sum+=cross(q[0],q[i],q[i+1]);return Math.abs(sum)*0.5;}
function valid(q){if(!Array.isArray(q)||q.length!==4||q.some(p=>!Array.isArray(p)||p.length!==2||!p.every(Number.isFinite)))return false;let sign=0;for(let i=0;i<4;i++){const t=cross(q[i],q[(i+1)%4],q[(i+2)%4]);if(!Number.isFinite(t)||t===0||(sign!==0&&sign*Math.sign(t)<=0))return false;sign=Math.sign(t);}return area(q)>0;}
function visibility(a,b){if(!valid(a)||!valid(b))return -1;let separated=false;for(const q of [a,b])for(let i=0;i<4;i++){const p=q[i],n=q[(i+1)%4],axis=[p[1]-n[1],n[0]-p[0]];let amin=Infinity,amax=-Infinity,bmin=Infinity,bmax=-Infinity;for(const v of a){const d=(v[0]-a[0][0])*axis[0]+(v[1]-a[0][1])*axis[1];if(!Number.isFinite(d))return -1;amin=Math.min(amin,d);amax=Math.max(amax,d);}for(const v of b){const d=(v[0]-a[0][0])*axis[0]+(v[1]-a[0][1])*axis[1];if(!Number.isFinite(d))return -1;bmin=Math.min(bmin,d);bmax=Math.max(bmax,d);}if(amax<=bmin||bmax<=amin)separated=true;}return separated?0:1;}

test('Deep Decon alcove union and atomic renderer adoption',()=>{
  const h=read(hp).toString('utf8'), b=read(world+'gh_deep_decon_environment_visual.gd').toString('utf8');
  const raw=read('apps/godot_playable/data/scenes/gh_deep_decon.json'), scene=JSON.parse(raw.toString('utf8'));
  assert.equal(hash(raw),'65e0cbf11183fe114259d680b0cd84a23a7f8619a25e628f4b30a7a84ce870a8');
  assert.deepEqual(literal(b,'CANONICAL_SCENE'),scene);assert.equal(scene.collision.length,8);
  const ids=['deep_decon_safety_alcove_west','deep_decon_safety_alcove_south'];
  assert.deepEqual(literal(h,'OWNER_IDS'),ids);for(let i=0;i<2;i++)assert.equal(scene.collision[6+i].id,ids[i]);
  const sides=literal(h,'SIDE_ROWS'),caps=literal(h,'CAP_RECTS');
  assert.deepEqual(sides,[{a:[1.5,10.5],b:[1.5,13],clip:true},{a:[4,13],b:[4,13.5],clip:false},{a:[1,13.5],b:[4,13.5],clip:false}]);
  assert.deepEqual(caps,[[1,10.5,1.5,13],[1,13,4,13.5]]);
  assert.ok(b.includes('const HEIGHT = 0.25')&&b.includes('const SIDE_COLOR')&&b.includes('const CAP_COLOR'));
  assert.ok(body(b,'_binding_ok').includes('_context_ok')&&body(b,'_binding_ok').includes('actual_viewport == viewport'));
  assert.ok(h.includes('Boundary.CANONICAL_SCENE.collision[6+index]')&&h.includes('_guard._quad_ok')&&h.includes('_guard._visible'));
  const stagePlan=body(h,'_stage');assert.ok(stagePlan.includes('for row_value: Variant in SIDE_ROWS')&&stagePlan.includes('ground_b-axis*Boundary.HEIGHT if row.clip else ground_b')&&stagePlan.includes('for rect_value: Variant in CAP_RECTS'));
  assert.equal(h.includes('_stage_plan('),false);
  const draw=body(h,'draw_or_consume'),paint=draw.indexOf('canvas.draw_colored_polygon'),stage=draw.indexOf('_stage(');
  assert.ok(stage>=0&&draw.lastIndexOf('_guard._binding_ok',paint)>stage&&draw.lastIndexOf('_guard._capture_matches',paint)>stage&&paint>stage);
  assert.ok(draw.indexOf("receipt['stamp']")>paint&&draw.indexOf("receipt['camera']")>paint&&draw.indexOf("receipt['canvas']")>paint);
  assert.ok(draw.includes('_receipt_matches')&&draw.includes('plan.all.size() != 5'));
  for(let ix=0;ix<12;ix++)for(let iz=0;iz<12;iz++){const x=1+(ix+0.5)/4,z=10.5+(iz+0.5)/4;let count=0;for(const r of caps)if(x>r[0]&&x<r[2]&&z>r[1]&&z<r[3])count++;assert.equal(count,(x<1.5||z>13)?1:0);}
  const shapes=[];let originalWest;
  for(const ppm of [24,48,72]){shapes.length=0;for(const row of sides){const a=[row.a[0],0,row.a[1]],g=[row.b[0],0,row.b[1]],ta=[a[0],0.25,a[2]],tb=[g[0],0.25,g[2]],original=screen([a,g,tb,ta],ppm),end=row.clip?[g[0],0,g[2]-0.25]:g;assert.ok(valid(original));shapes.push(screen([a,end,tb,ta],ppm));if(row.clip)originalWest=original;}for(const r of caps)shapes.push(screen([[r[0],0.25,r[1]],[r[2],0.25,r[1]],[r[2],0.25,r[3]],[r[0],0.25,r[3]]],ppm));assert.equal(shapes.length,5);for(const q of shapes)assert.ok(valid(q));for(let i=0;i<5;i++)for(let j=i+1;j<5;j++)assert.equal(visibility(shapes[i],shapes[j]),0);assert.equal(visibility(originalWest,shapes[4]),1);assert.ok(Math.abs(area(originalWest)-area(shapes[0])-ppm*ppm/32)<1e-9);assert.ok(Math.abs(area(shapes[3])+area(shapes[4])-2.75*ppm*ppm)<1e-9);}
  const r=read(rp),text=r.toString('utf8'),hooks=["const GhDeepDeconAlcoveVisual = preload('res://scripts/world/gh_deep_decon_alcove_visual.gd')\n","var gh_deep_decon_alcove_visual = GhDeepDeconAlcoveVisual.new()\n","\tvar gh_deep_decon_alcove_receipt: Dictionary = {}\n","\t\t\tif gh_deep_decon_alcove_visual.draw_or_consume(self,snapshot,scene,current_key,scene_hash,camera,wall,viewport_rect,gh_deep_decon_alcove_receipt):\n\t\t\t\tcontinue\n"];
  let restored=text;for(const hook of hooks){assert.equal(restored.split(hook).length,2);restored=restored.replace(hook,'');}
  const prior=text.indexOf('\t\t\tif gh_deep_decon_environment_visual.draw_wall('),added=text.indexOf(hooks[3]),fallback=text.indexOf("\t\t\t_poly(wall.polygon,0,Color('#4b5659'),true)",prior);
  assert.ok(prior>=0&&added>prior&&fallback>added);const base=process.env.WUXIAN_ALCOVE_RENDERER_BASE_SHA256;
  if(base)assert.equal(hash(Buffer.from(restored,'utf8')),base.toLowerCase());else{const head=execFileSync('git',['show','HEAD:'+rp],{cwd:root});const committed=head.includes(Buffer.from(hooks[0]));assert.deepEqual(committed?r:Buffer.from(restored,'utf8'),head);}
});
