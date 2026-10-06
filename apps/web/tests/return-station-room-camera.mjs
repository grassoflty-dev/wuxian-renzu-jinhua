import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {returnStationRoomEnvelope,fitReturnStationRoom} from '../dist/renderer/ReturnStationRoomCamera.js';
import {WorldRenderer} from '../dist/renderer/WorldRenderer.js';
import {CameraRig} from '../dist/renderer/CameraRig.js';
import {BossCameraModel} from '../dist/renderer/BossCameraModel.js';
import {deformPlayerLocomotionVertices} from '../dist/renderer/PlayerLocomotionMesh.js';
const root=new URL('../../../',import.meta.url);
const raw=JSON.parse(await readFile(new URL('content/scenes/compiled/rs_core_room.json',root),'utf8'));
const manifest=JSON.parse(await readFile(new URL('governance/assets/RUNTIME_ASSET_MANIFEST.json',root),'utf8'));
const runtime=id=>{const a=manifest.assets.find(a=>a.assetId===id);return {...a,anchorX:a.anchor[0],anchorY:a.anchor[1],kind:id.includes('actor.cenyao')?'Actor':'Prop'};};
const actor=runtime('runtime2d.actor.cenyao.base.v1');
const scene={worldId:raw.worldId,sceneId:raw.sceneId,bounds:raw.boundsM,sprites:raw.presentation.sprites.map(p=>({id:p.id,asset:runtime(p.assetId),position:{xM:p.position[0],yM:p.position[1],zM:p.position[2]}}))};
const snapshot={protocolVersion:3,worldId:raw.worldId,sceneId:raw.sceneId};
const inside=(point,frame)=>{const p=frame.pixelsPerMeter,{xM:x,zM:z}=frame.origin;const sx=frame.width/2+p*(point.u-(x-z)),sy=frame.height/2+p*(point.v-(x+z)/2);assert.ok(sx>=frame.width*.05-1e-8&&sx<=frame.width*.95+1e-8&&sy>=frame.height*.05-1e-8&&sy<=frame.height*.95+1e-8,`${sx},${sy}`);};
const envelope=returnStationRoomEnvelope(scene,actor,snapshot);
test('full room, terminals and actor corner envelope fit exact per-axis 5% margins',()=>{
 const before=JSON.stringify({scene,actor,snapshot});
 for(const[w,h]of[[1280,720],[1920,1080],[960,540],[1000,1000],[2400,720]]){
  const frame=fitReturnStationRoom(envelope,w,h);for(const q of envelope)inside(q,frame);
  for(const[x,z]of[[0,0],[24,0],[24,16],[0,16]])inside({u:x-z,v:(x+z)/2},frame);
  const us=envelope.map(p=>p.u),vs=envelope.map(p=>p.v);
  assert.ok(Math.abs(frame.pixelsPerMeter-Math.min(.9*w/(Math.max(...us)-Math.min(...us)),.9*h/(Math.max(...vs)-Math.min(...vs))))<1e-12);
 }
 assert.equal(JSON.stringify({scene,actor,snapshot}),before);
 const a=fitReturnStationRoom(envelope,1280,720),b=fitReturnStationRoom(envelope,1920,1080);
 assert.ok(Math.abs(b.pixelsPerMeter/a.pixelsPerMeter-1.5)<1e-12);
 assert.ok(Math.abs(a.origin.xM-b.origin.xM)<1e-12);
});
test('all actual idle directions and existing mesh deformations fit all room corners',()=>{
 assert.equal(actor.animation.frames.length,8);
 const frame=fitReturnStationRoom(envelope,1280,720);
 const rest=new Float32Array(Array.from({length:21*33},(_,i)=>[i%21/20*336,Math.floor(i/21)/32*560]).flat());const output=new Float32Array(rest.length);
 for(const f of actor.animation.frames)for(let phase=0;phase<32;phase++)for(const reducedMotion of[false,true]){
  deformPlayerLocomotionVertices(rest,output,336,560,{kind:'temporary_visual',active:true,phaseRad:phase*Math.PI/16,amplitude:1,travelX:Math.sin(phase),travelY:Math.cos(phase),reducedMotion},f.direction);
  for(const[x,z]of[[0,0],[24,0],[24,16],[0,16]])for(let i=0;i<output.length;i+=2)inside({u:x-z+(output[i]-f.anchor[0]*336)*1.72/560,v:(x+z)/2+(output[i+1]-f.anchor[1]*560)*1.72/560},frame);
 }
});
test('fixed renderer camera ignores follow/aim/boss, resizes, exits to original rig, and reenters identically',()=>{
 const r=Object.create(WorldRenderer.prototype);r.cameraRig=new CameraRig();r.bossCameraModel=new BossCameraModel();
 const call=(snap,w,h,origin={xM:4,yM:0,zM:8},aim=0)=>r.updateCameraFrame(snap,origin,aim,1,w,h,.1,48,scene,actor);
 const a=call(snapshot,1280,720);assert.deepEqual(a,call(snapshot,1280,720,{xM:23,yM:0,zM:15},100));
 const b=call(snapshot,1920,1080);assert.notEqual(a.pixelsPerMeter,b.pixelsPerMeter);
 const exited=call({...snapshot,worldId:'grey_hive',sceneId:'gh_entry_maintenance'},1280,720);
 assert.deepEqual(exited.camera,new CameraRig().update({xM:4,yM:0,zM:8},0,1,1280,720,.1,48));assert.equal(r.roomFitActive,false);
 assert.deepEqual(call(snapshot,1280,720),a);
});
test('invalid dimensions, missing terminals and unknown actor geometry fail closed',()=>{
 for(const[w,h]of[[0,720],[1280,0],[NaN,720],[Infinity,720],[-1,720]])assert.throws(()=>fitReturnStationRoom(envelope,w,h),/ROOM_FIT/);
 for(const points of[[],[{u:0,v:0}],[{u:NaN,v:0},{u:1,v:1}]])assert.throws(()=>fitReturnStationRoom(points,1280,720),/ROOM_FIT/);
 assert.throws(()=>returnStationRoomEnvelope({...scene,sprites:[]},actor,snapshot),/ROOM_FIT/);
 assert.throws(()=>returnStationRoomEnvelope(scene,{...actor,animation:undefined},snapshot),/ROOM_FIT/);
});

test('actual pointer boundary uses fitted player feet and rejects stale frame throughout a pending resize',async()=>{
 const {projectWorldPoint}=await import('../dist/renderer/CameraModel.js');
 const r=new WorldRenderer({});
 const value={kind:'full',protocolVersion:3,schemaVersion:'freeze-v02-interfaces/1.2',worldId:'return_station',sceneId:'rs_core_room',worldEpoch:1,checkpointId:null,serverTick:1,authorityRevision:1,ackSeq:0,
  player:{entityId:'player',transform:{positionM:{xM:4,yM:0,zM:8},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:80,maxHp:100,currentEnergy:60,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},actors:[],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'return_station',eventSeq:0,worlds:[]}};
 const host={clientWidth:1280,clientHeight:720};r.surfaceCanvas={parentElement:host};r.ready=true;
 const requests=[];r.surface={isAvailable:()=>true,request:refresh=>requests.push(refresh)};
 r.app={screen:{width:1280,height:720},renderer:{resize(w,h){r.app.screen={width:w,height:h};}}};
 const commit=()=>{const camera=fitReturnStationRoom(envelope,r.app.screen.width,r.app.screen.height),p=projectWorldPoint(value.player.transform.positionM,camera);r.committedFrame={sceneKey:['return_station','rs_core_room','1'].join('\0'),revision:r.sceneRequestRevision,viewportRevision:r.viewportRevision,surface:r.surface,playerAim:{width:camera.width,height:camera.height,footX:p.x,footY:p.y}};return p;};
 // Explicitly modeled successful presentation; no GPU/native execution is claimed.
 const p=commit();r.committedFrame.sceneKey=['return_station','rs_core_room','1'].join('\0');
 const rect={left:20,top:30,width:1280,height:720};assert.deepEqual(r.pointerAim(value,p.x+20,p.y+30,rect),{x:0,y:0});
 assert.deepEqual(r.pointerAim(value,p.x+120,p.y+30,rect),{x:100,y:0});
 host.clientWidth=1920;host.clientHeight=1080;r.resizeSurface();assert.deepEqual(requests,[true]);assert.equal(r.pointerAim(value,p.x+20,p.y+30,rect),null);
 let release;r.renderFrame=async()=>{await new Promise(resolve=>{release=resolve;});commit();r.committedFrame.sceneKey=['return_station','rs_core_room','1'].join('\0');};
 r.lastFrame={snapshot:value,timeMs:42};const pending=r.refreshSurface();await Promise.resolve();
 assert.equal(r.pointerAim(value,p.x+20,p.y+30,rect),null);release();await pending;
 const fresh=r.committedFrame.playerAim;assert.deepEqual(r.pointerAim(value,20+fresh.footX/2,30+fresh.footY/2,{left:20,top:30,width:960,height:540}),{x:0,y:0});
 const code=await readFile(new URL('../src/renderer/WorldRenderer.ts',import.meta.url),'utf8');
 assert.match(code,/const presentedPlayer = projectWorldPoint\(playerPosition \?\? view.playerPosition, camera\)/);
 assert.ok(code.indexOf('if (!surface.present())')<code.indexOf('const presentedPlayer = projectWorldPoint'));
});

async function rendererSessionFixture(run) {
 const {Container,Texture,TextureSource,Assets}=await import('pixi.js');
 const globals=Object.fromEntries(['window','document','ResizeObserver','requestAnimationFrame','cancelAnimationFrame'].map(k=>[k,globalThis[k]]));
 const oldUnload=Assets.unload;const queued=new Map();let nextId=0;
 globalThis.window=new EventTarget();globalThis.window.matchMedia=()=>({matches:true});globalThis.document={documentElement:{dataset:{motion:'reduced'}}};globalThis.ResizeObserver=class{observe(){}disconnect(){}};
 globalThis.requestAnimationFrame=fn=>{queued.set(++nextId,fn);return nextId;};globalThis.cancelAnimationFrame=id=>queued.delete(id);Assets.unload=async()=>{};
 const assets=new Map(manifest.assets.map(a=>{const page=manifest.atlases[a.atlasPage];return[a.assetId,{...runtime(a.assetId),kind:{actor:'Actor',prop:'Prop',world:'World',vfx:'VFX',ui:'UI',boss:'Boss'}[a.category],atlasUrl:page.webpPath,atlasSha256:page.webpSha256,atlasDecodedRgbaBytes:page.size[0]*page.size[1]*4,animation:a.animation?{...a.animation,frames:a.animation.frames.map(f=>({...f,atlasUrl:manifest.atlases[f.atlasPage].webpPath,atlasSha256:manifest.atlases[f.atlasPage].webpSha256}))}:undefined}];}));
 const registry={resolveAsset:id=>{const a=assets.get(id);assert.ok(a,id);return a;}};
 const r=new WorldRenderer(registry);const canvas=new EventTarget();canvas.parentElement={clientWidth:1280,clientHeight:720};let draws=0;
 r.app={stage:new Container(),screen:{width:0,height:0},renderer:{background:{},resize(w,h){r.app.screen={width:w,height:h};}},async init(o){this.renderer.resize(o.width,o.height);},render(){draws++;},destroy(){}};
 const value={kind:'full',protocolVersion:3,schemaVersion:'freeze-v02-interfaces/1.2',worldId:'return_station',sceneId:'rs_core_room',worldEpoch:1,checkpointId:null,serverTick:1,authorityRevision:1,ackSeq:0,player:{entityId:'player',transform:{positionM:{xM:4,yM:0,zM:8},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:80,maxHp:100,currentEnergy:60,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},actors:[],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'return_station',eventSeq:0,worlds:[]}};
 const {SceneDefinitionSession}=await import('../dist/game/SceneDefinitionSession.js');
 const session=new SceneDefinitionSession(r,{loadForSnapshot:async()=>raw});
 const preload=async pages=>{for(const p of pages)if(!r.atlasTextures.has(p.atlasUrl))r.atlasTextures.set(p.atlasUrl,new Texture({source:new TextureSource({width:8192,height:8192})}));};
 r.preloadAtlasPages=preload;
 const deferNextAtlas=()=>{
  let entered,release;const waiting=new Promise(resolve=>{entered=resolve;}),blocked=new Promise(resolve=>{release=resolve;});
  r.preloadAtlasPages=async pages=>{r.preloadAtlasPages=preload;await preload(pages);entered();await blocked;};
  return {waiting,release};
 };
 const resize=(width,height,notify=true)=>{Object.assign(canvas.parentElement,{clientWidth:width,clientHeight:height});if(notify)r.resizeSurface();};
 const rect=()=>({left:0,top:0,width:canvas.parentElement.clientWidth,height:canvas.parentElement.clientHeight});
 try{
  await r.init(canvas);
  await run({r,session,canvas,value,resize,rect,deferNextAtlas,draws:()=>draws});
 }finally{
  if(r.ready)await r.destroy();Assets.unload=oldUnload;for(const[k,v]of Object.entries(globals)){if(v===undefined)delete globalThis[k];else globalThis[k]=v;}
 }
}

test('real scene session commits initial and established frames after atlas-time resize, repeated resize and resize-back',async()=>rendererSessionFixture(async f=>{
 const {r,session,value}=f;
 let gate=f.deferNextAtlas(),settled=false;
 const initial=session.render(value).then(()=>{settled=true;});await gate.waiting;
 f.resize(1920,1080);f.resize(1600,900);
 assert.equal(settled,false);assert.equal(f.draws(),0);assert.equal(session.isReadyFor(value),false);
 assert.equal(session.pointerAim(value,0,0,f.rect()),null);
 gate.release();await initial;
 assert.equal(f.draws(),1);assert.equal(session.isReadyFor(value),true);assert.deepEqual(r.app.screen,{width:1600,height:900});
 const changed={...value,serverTick:2,player:{...value.player,transform:{...value.player.transform,positionM:{xM:7,yM:0,zM:10}}}};
 gate=f.deferNextAtlas();settled=false;const next=session.render(changed).then(()=>{settled=true;});await gate.waiting;
 f.resize(1920,1080);f.resize(1600,900);f.resize(1280,720);
 assert.equal(settled,false);assert.equal(session.isReadyFor(changed),false);assert.equal(session.pointerAim(changed,0,0,f.rect()),null);
 gate.release();await next;
 assert.equal(f.draws(),2);assert.equal(session.isReadyFor(changed),true);assert.equal(r.pendingViewportFrame,null);
 const foot=r.committedFrame.playerAim;assert.deepEqual(session.pointerAim(changed,foot.footX,foot.footY,f.rect()),{x:0,y:0});
 assert.equal(foot.footX,r.sprites.get('actor:player').sprite.x);assert.equal(foot.footY,r.sprites.get('actor:player').sprite.y);
 assert.deepEqual(r.app.screen,{width:1280,height:720});
 // Readiness also rejects a host change before ResizeObserver has delivered it.
 gate=f.deferNextAtlas();const unobserved=session.render(changed);await gate.waiting;f.resize(1920,1080,false);
 assert.equal(session.isReadyFor(changed),false);gate.release();await unobserved;
 assert.equal(session.isReadyFor(changed),true);assert.deepEqual(r.app.screen,{width:1920,height:1080});
}));

test('synchronous resize during draw stays pending and safely yields through repeated redraws before session commit',async()=>rendererSessionFixture(async f=>{
 const {r,session,value}=f;const draw=r.app.render;let remaining=3,started;
 const firstDraw=new Promise(resolve=>{started=resolve;});let settled=false;
 r.app.render=()=>{draw();if(remaining>0){const index=remaining--;f.resize(index%2?1920:1280,index%2?1080:720);assert.equal(session.pointerAim(value,0,0,f.rect()),null);started();}};
 const rendered=session.render(value).then(()=>{settled=true;});await firstDraw;await Promise.resolve();
 assert.equal(settled,false);assert.equal(session.isReadyFor(value),false);
 await rendered;
 assert.equal(f.draws(),4);assert.equal(session.isReadyFor(value),true);assert.equal(r.pendingViewportFrame,null);
 const foot=r.committedFrame.playerAim;assert.deepEqual(session.pointerAim(value,foot.footX,foot.footY,f.rect()),{x:0,y:0});
}));

for(const mode of ['cancel','context-loss','new-epoch'])test(`viewport replay respects ${mode} without reviving old readiness`,async()=>rendererSessionFixture(async f=>{
 const {r,session,value}=f;const draw=r.app.render;let started,first=true;
 const firstDraw=new Promise(resolve=>{started=resolve;});
 r.app.render=()=>{draw();if(first){first=false;f.resize(1920,1080);started();}};
 const pending=session.render(value);
 const rejected=assert.rejects(pending,mode==='cancel'?/CANCELLED/:mode==='context-loss'?/CONTEXT_LOST/:/STALE_FRAME/);
 await firstDraw;
 if(mode==='cancel')session.cancel();
 else if(mode==='context-loss')f.canvas.dispatchEvent(new Event('webglcontextlost',{cancelable:true}));
 else session.acceptSnapshot({...value,worldEpoch:2});
 await rejected;await r.renderQueue;
 assert.equal(session.isReadyFor(value),false);assert.equal(r.committedFrame,null);assert.equal(r.pendingViewportFrame,null);assert.equal(f.draws(),1);
 if(mode==='new-epoch'){
  const next={...value,worldEpoch:2};await session.render(next);assert.equal(session.isReadyFor(next),true);assert.equal(f.draws(),2);
 }
}));
