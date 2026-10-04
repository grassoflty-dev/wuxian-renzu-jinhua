import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {Container,Sprite,Texture} from 'pixi.js';
import {ClockworksEnemyModel,CLOCKWORKS_ENEMY_PLACEHOLDERS,enemyWarningGroundPoints} from '../dist/renderer/ClockworksEnemyModel.js';
import {validatedSwarmMembers} from '../dist/protocol/PresentationEventValidation.js';
import {WorldRenderer} from '../dist/renderer/WorldRenderer.js';
import {projectWorldPoint} from '../dist/renderer/CameraModel.js';
import {actorAssetId,buildRendererResourcePlan} from '../dist/renderer/RendererResourcePlan.js';
import {AssetRegistry} from '../dist/assets/AssetRegistry.js';
import {audioCueForPresentationEvent} from '../dist/audio/AudioEventMap.js';
import {SnapshotClient} from '../dist/game/SnapshotClient.js';
import {SessionLoop} from '../dist/game/SessionLoop.js';
const type='enemy.grey_hive.swarm',id='gh_lockdown_swarm_01',point={xM:7,yM:0,zM:8};
function actor(){return {entityId:id,entityType:type,actorKind:'enemy',active:true,transform:{positionM:point,yawRad:0},members:[[-.55,-.55],[.55,-.55],[-.55,.55],[.55,.55]].map(([x,z],i)=>({memberId:`${id}/member/${i+1}`,positionM:{xM:point.xM+x,yM:0,zM:point.zM+z},radiusM:.15,active:true}))};}
function snapshot(overrides={}){return {kind:'full',protocolVersion:3,schemaVersion:'scene-v3/1',worldId:'grey_hive',sceneId:'gh_lockdown',worldEpoch:7,serverTick:100,authorityRevision:100,ackSeq:100,checkpointId:null,
 player:{entityId:'player',transform:{positionM:{xM:5,yM:0,zM:8},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},actors:[actor()],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[]},...overrides};}
function event(eventId=1,overrides={}){return {protocolVersion:2,eventId,worldEpoch:7,serverTick:100,kind:'EnemyAttackTelegraph',actorId:id,attackKind:'lunge',durationMs:650,rangeM:2,radiusM:.95,intensity:1,positionM:point,directionRad:Math.PI/2,...overrides};}
function memberEvent(eventId=1,index=0,overrides={}){return event(eventId,{kind:'EnemyMemberHit',memberId:`${id}/member/${index+1}`,positionM:actor().members[index].positionM,attackKind:undefined,rangeM:undefined,radiusM:.15,durationMs:170,...overrides});}
const camera={width:1200,height:800,origin:{xM:0,yM:0,zM:0},pixelsPerMeter:48};
function renderer(){const r=Object.create(WorldRenderer.prototype);r.layerContainers=new Map([['L3_ACTORS',new Container()],['L7_VFX',new Container()]]);r.clockworksEnemyModel=new ClockworksEnemyModel();r.clockworksEnemyGraphics=new Map();r.clockworksEnemyGlows=new Map();r.clockworksEnemyBodies=new Map();r.sprites=new Map();r.sentinelTelegraphGraphics=new Map();r.sentinelGlowGraphics=new Map();r.sentinelTelegraphModel={reset(){}};r.transientPresentationRevision=0;r.machineryGraphics=new Map();return r;}

test('Swarm renders four admitted temporary members at public points, with individual shadows and authority-owned dispersal',async()=>{
 const root=new URL('../../../',import.meta.url),registry=new AssetRegistry();await registry.registerManifest(new Uint8Array(await readFile(new URL('governance/assets/RUNTIME_ASSET_MANIFEST.json',root))),new Uint8Array(await readFile(new URL('governance/assets/AI_ASSET_RELEASE_MANIFEST.json',root))),async path=>new Uint8Array(await readFile(new URL(path,root))));
 const a=actor(),style=CLOCKWORKS_ENEMY_PLACEHOLDERS[type];assert.equal(style.temporary_visual,true);assert.equal(style.silhouette,'cluster');assert.equal(actorAssetId(a),'runtime2d.enemy.infected_maintenance_worker.v1');assert.ok(buildRendererResourcePlan(registry,'grey_hive',[a],[]).assets.some(row=>row.assetId===style.assetId));
 const r=renderer(),rootSprite=new Sprite(Texture.WHITE);r.sprites.set(`actor:${id}`,{sprite:rootSprite});r.renderClockworksEnemyBodies([a],camera,new Map([[id,{xM:90,yM:0,zM:90}]]));assert.equal(rootSprite.visible,false);assert.equal(r.swarmMemberBodies.size,4);
 for(const m of a.members){const body=r.swarmMemberBodies.get(m.memberId),p=projectWorldPoint(m.positionM,camera);assert.deepEqual([body.sprite.position.x,body.sprite.position.y],[p.x,p.y]);assert.ok(body.shadow.label.includes(m.memberId));assert.ok(body.outline.label.includes('temporary_visual=true'));}
 const removed=r.swarmMemberBodies.get(a.members[0].memberId);a.members[0].active=false;r.renderClockworksEnemyBodies([a],camera);assert.equal(r.swarmMemberBodies.size,3);assert.equal(removed.sprite.destroyed,true);
 a.members.forEach(m=>m.active=false);a.active=false;r.renderClockworksEnemyBodies([a],camera);assert.equal(r.swarmMemberBodies.size,0);r.clearClockworksEnemyBodies();
});

test('Malformed member arrays and wrong member/alive/source associations cannot consume the event cursor',()=>{
 const faults=[a=>a.members.pop(),a=>a.members.push({...a.members[0]}),a=>a.members[0].memberId=a.members[1].memberId,a=>a.members[0].positionM.xM=Infinity,a=>a.members[0].positionM.xM+=10,a=>a.members[0].radiusM=0,a=>a.members.forEach(m=>m.active=false)];
 for(const mutate of faults){const a=actor();mutate(a);assert.equal(validatedSwarmMembers(a),null);const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([event(999)],snapshot({actors:[a]})),[]);assert.deepEqual(model.accept([event(2)],snapshot()),[2]);}
 for(const [bad,state] of [[{memberId:`${id}/member/99`},{}],[{kind:'EnemyMemberDisperse'},{}],[{}, {sceneId:'gh_gate_b'}],[{}, {worldId:'clockworks',sceneId:'cw_pressure_hall'}],[{}, {actors:[actor(),actor()]}]]){
  const client=new SnapshotClient();client.accept(snapshot(state));assert.deepEqual(client.acceptEvents([memberEvent(999,0,bad)]),[]);assert.equal(client.eventCursor(),0);
 }
 const client=new SnapshotClient();client.accept(snapshot());assert.deepEqual(client.acceptEvents([memberEvent(999,0,{memberId:`${id}/member/99`}),memberEvent(2)]).map(e=>e.eventId),[2]);assert.equal(client.eventCursor(),2);
});

test('Distinct member hit/disperse cues coexist with correct audio and never carry HP',()=>{
 const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([memberEvent(1,0),memberEvent(2,2)],snapshot()),[1,2]);let frames=model.project(snapshot(),false);assert.equal(frames.length,2);assert.equal(new Set(frames.map(f=>f.memberId)).size,2);assert.ok(!JSON.stringify(frames).match(/currentHp|maxHp|memberHp|damage|ordinary/));
 const s=snapshot();s.actors[0].members[0].active=false;assert.deepEqual(model.accept([memberEvent(3,0,{kind:'EnemyMemberDisperse'})],s),[3]);frames=model.project(s,true);assert.ok(frames.some(f=>f.kind==='EnemyMemberDisperse'));assert.ok(frames.every(f=>f.temporary_visual));
 assert.equal(audioCueForPresentationEvent(memberEvent()).id,'audio.gh.swarm.member.hit');assert.equal(audioCueForPresentationEvent(memberEvent(1,0,{kind:'EnemyMemberDisperse'})).id,'audio.gh.swarm.member.disperse');
 const r=renderer();r.renderClockworksEnemyCues(frames,camera);assert.equal(r.clockworksEnemyGraphics.size,2);r.clearTransientPresentation();
});

test('Lunge warning is a committed capsule with body glow, sound, exact motion and terminal suppression',()=>{
 const model=new ClockworksEnemyModel();model.accept([event()],snapshot());const frame=model.project(snapshot(),true)[0],points=enemyWarningGroundPoints(frame);assert.equal(points.length,66);const xs=points.map(p=>p.xM);assert.ok(Math.abs(Math.min(...xs)-(7-.95))<1e-10);assert.ok(Math.abs(Math.max(...xs)-(7+2+.95))<1e-10);
 const moved=snapshot();moved.player.transform.positionM={xM:20,yM:0,zM:1};assert.deepEqual(model.project(moved,true)[0],frame);
 const r=renderer();r.renderClockworksEnemyCues([frame],camera);assert.ok(r.clockworksEnemyGlows.has(1));assert.equal(audioCueForPresentationEvent(event()).id,'audio.gh.swarm.lunge.warning');assert.equal(audioCueForPresentationEvent(event(2,{kind:'EnemyAttackImpact'})).id,'audio.gh.swarm.lunge.impact');
 const motion=event(2,{kind:'EnemyLungeMotion',positionM:{...point,xM:7.3},durationMs:400});model.accept([motion],snapshot());assert.deepEqual(model.project(snapshot(),false)[0].positionM,motion.positionM);assert.equal(audioCueForPresentationEvent(motion),null);
 assert.deepEqual(model.accept([event(3,{kind:'EnemyAttackImpact',durationMs:170}),event(4,{kind:'EnemyLungeMotion'})],snapshot()),[3]);assert.equal(model.project(snapshot(),true)[0].kind,'EnemyAttackImpact');r.clearTransientPresentation();
});

test('Future member cues wait for their exact snapshot and cannot leak across replacement',()=>{
 const client=new SnapshotClient();client.accept(snapshot());const future=memberEvent(1,0,{serverTick:101});assert.deepEqual(client.acceptEvents([future]),[]);assert.equal(client.eventCursor(),0);client.accept(snapshot({serverTick:101,authorityRevision:101,ackSeq:101}));assert.deepEqual(client.acceptEvents([]).map(e=>e.eventId),[1]);
 const second=new SnapshotClient();second.accept(snapshot());second.acceptEvents([future]);second.accept(snapshot({worldEpoch:8,serverTick:0,authorityRevision:0,ackSeq:0}));assert.deepEqual(second.acceptEvents([]),[]);assert.equal(second.eventCursor(),0);
});

test('One-tick saved Swarm phase is drawn with surviving members before ready and ordered resume',async()=>{
 for(const kind of ['EnemyAttackTelegraph','EnemyLungeMotion']){
  let held=true,counter=1,current=snapshot(),committed=null;current.actors[0].members[0].active=false;const model=new ClockworksEnemyModel(),rows=[event(counter,{kind,durationMs:1000/60})],calls=[];
  const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:()=>1,clearInterval(){},requestAnimationFrame:()=>1,cancelAnimationFrame(){}};
  const client={events:async(epoch,cursor)=>rows.filter(e=>e.eventId>cursor),sceneReady:async()=>{assert.deepEqual(calls.at(-1).slice(0,4),['draw',true,1,3]);held=false;return current;},pause:async()=>{held=true;rows.push(event(++counter,{kind,durationMs:1000/60}));return current;},resume:async()=>{assert.equal(held,true);assert.deepEqual(calls.at(-1).slice(0,4),['draw',true,1,3]);held=false;return current;}};
  const render={render:async s=>{const frames=model.project(s,true);calls.push(['draw',held,frames.length,validatedSwarmMembers(s.actors[0]).filter(m=>m.active).length]);if(frames.length)assert.equal(frames[0].remainingMs,1000/60);committed=s;},isReadyFor:s=>committed?.worldEpoch===s.worldEpoch&&committed?.sceneId===s.sceneId,destroy(){}};
  const loop=new SessionLoop(client,render,{scheduler,onEventsWithSnapshot:(events,s)=>{const accepted=new Set(model.accept(events,s));return events.filter(e=>accepted.has(e.eventId));},onClearTransientPresentation:()=>model.reset()});await loop.start({...current,entryToken:{generation:7,worldId:'grey_hive',sceneId:'gh_lockdown',worldEpoch:7}});assert.equal(loop.pausePresentationState,'running');await loop.pause();await loop.resume();assert.equal(loop.pausePresentationState,'running');await loop.stop('unload');
 }
});

test('A grouped Pulse keeps every same-tick member hit or dispersal alongside its one aggregate terminal cue',()=>{
 for(const mode of ['hit-two','disperse-two','disperse-all']){
  const s=snapshot(),count=mode==='disperse-all'?4:2,dead=mode!=='hit-two';
  if(dead)for(let i=0;i<count;i++)s.actors[0].members[i].active=false;
  if(count===4)s.actors[0].active=false;
  const rows=Array.from({length:count},(_,i)=>memberEvent(i+1,i,{kind:dead?'EnemyMemberDisperse':'EnemyMemberHit'}));
  rows.push(event(count+1,{kind:count===4?'EnemyDeath':'EnemyStagger',attackKind:undefined,rangeM:undefined,durationMs:340}));
  const model=new ClockworksEnemyModel();assert.deepEqual(model.accept(rows,s),rows.map(e=>e.eventId));
  const frames=model.project(s,true);assert.equal(frames.length,count+1);assert.equal(new Set(frames.filter(f=>f.memberId).map(f=>f.memberId)).size,count);
  const r=renderer();r.renderClockworksEnemyCues(frames,camera);assert.equal(r.clockworksEnemyGraphics.size,count+1);
  for(let i=0;i<count;i++){const p=projectWorldPoint(s.actors[0].members[i].positionM,camera),g=r.clockworksEnemyGraphics.get(i+1);assert.deepEqual([g.position.x,g.position.y],[p.x,p.footY]);}
  assert.ok(rows.every(e=>audioCueForPresentationEvent(e)?.temporary_audio));r.clearTransientPresentation();
 }
});
