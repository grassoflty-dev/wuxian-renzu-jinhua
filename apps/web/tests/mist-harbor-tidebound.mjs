import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {Container,Polygon,Sprite,Texture} from 'pixi.js';
import {ClockworksEnemyModel,CLOCKWORKS_ENEMY_PLACEHOLDERS,enemyWarningGroundPoints} from '../dist/renderer/ClockworksEnemyModel.js';
import {WorldRenderer} from '../dist/renderer/WorldRenderer.js';
import {projectWorldPoint} from '../dist/renderer/CameraModel.js';
import {actorAssetId,buildRendererResourcePlan} from '../dist/renderer/RendererResourcePlan.js';
import {AssetRegistry} from '../dist/assets/AssetRegistry.js';
import {audioCueForPresentationEvent} from '../dist/audio/AudioEventMap.js';
import {SnapshotClient} from '../dist/game/SnapshotClient.js';
import {SessionLoop} from '../dist/game/SessionLoop.js';
const type='enemy.mist_harbor.tidebound',id='mh_drowned_quay_tidebound_01',point={xM:16,yM:0,zM:8};
function snapshot(overrides={}){return {kind:'full',protocolVersion:3,schemaVersion:'scene-v3/1',worldId:'mist_harbor',sceneId:'mh_drowned_quay',worldEpoch:7,serverTick:100,authorityRevision:100,ackSeq:100,checkpointId:null,
 player:{entityId:'player',transform:{positionM:{xM:17,yM:0,zM:8},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},actors:[{entityId:id,entityType:type,actorKind:'enemy',active:true,transform:{positionM:point,yawRad:0}}],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'mist_harbor',eventSeq:0,worlds:[]},...overrides};}
function event(eventId=1,overrides={}){return {protocolVersion:2,eventId,worldEpoch:7,serverTick:100,kind:'EnemyAttackTelegraph',actorId:id,attackKind:'tide_swing',durationMs:900,rangeM:2.2,radiusM:2.2,halfAngleRad:.8,intensity:1,positionM:point,directionRad:Math.PI/2,...overrides};}
function charge(eventId=1,overrides={}){return event(eventId,{attackKind:'tide_charge',rangeM:3.85,radiusM:.55,halfAngleRad:undefined,durationMs:1100,...overrides});}
const camera={width:1200,height:800,origin:{xM:0,yM:0,zM:0},pixelsPerMeter:48};
function renderer(){const r=Object.create(WorldRenderer.prototype);r.layerContainers=new Map([['L3_ACTORS',new Container()],['L7_VFX',new Container()]]);r.clockworksEnemyModel=new ClockworksEnemyModel();r.clockworksEnemyGraphics=new Map();r.clockworksEnemyGlows=new Map();r.clockworksEnemyBodies=new Map();r.sprites=new Map();r.sentinelTelegraphGraphics=new Map();r.sentinelGlowGraphics=new Map();r.sentinelTelegraphModel={reset(){}};r.transientPresentationRevision=0;r.machineryGraphics=new Map();return r;}

test('Tidebound uses an admitted distinct temporary humanoid with its own contact shadow',async()=>{
 const root=new URL('../../../',import.meta.url),registry=new AssetRegistry();await registry.registerManifest(new Uint8Array(await readFile(new URL('governance/assets/RUNTIME_ASSET_MANIFEST.json',root))),new Uint8Array(await readFile(new URL('governance/assets/AI_ASSET_RELEASE_MANIFEST.json',root))),async path=>new Uint8Array(await readFile(new URL(path,root))));
 const actor=snapshot().actors[0],style=CLOCKWORKS_ENEMY_PLACEHOLDERS[type];assert.equal(style.temporary_visual,true);assert.equal(style.silhouette,'tidal');assert.equal(actorAssetId(actor),'runtime2d.enemy.mistharbor.drowned.v2');assert.ok(buildRendererResourcePlan(registry,'mist_harbor',[actor],[]).assets.some(a=>a.assetId===style.assetId));
 const r=renderer();r.sprites.set(`actor:${id}`,{sprite:new Sprite(Texture.WHITE)});r.renderClockworksEnemyBodies([actor],camera);const body=r.clockworksEnemyBodies.get(id);assert.ok(body.outline.label.includes('enemy-tidal'));assert.ok(body.shadow.label.includes('enemy-contact-shadow'));assert.equal(r.sprites.get(`actor:${id}`).sprite.width,56);r.clearClockworksEnemyBodies();assert.equal(body.shadow.destroyed,true);
});

test('Tidebound validates exact source identities, attack family and sector shape before cursor admission',()=>{
 for(const [bad,state]of [[{halfAngleRad:undefined},{}],[{halfAngleRad:0},{}],[{halfAngleRad:NaN},{}],[{halfAngleRad:Math.PI+.1},{}],[{rangeM:0},{}],[{attackKind:'charge'},{}],[{}, {sceneId:'mh_fog_pier'}],[{}, {worldId:'clockworks',sceneId:'cw_pressure_hall'}],[{}, {actors:[...snapshot().actors,...snapshot().actors]}]]){
  const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([event(999,bad)],snapshot(state)),[]);assert.deepEqual(model.accept([event(2)],snapshot()),[2]);
 }
 const invalid=snapshot();invalid.actors[0].entityId='mh_drowned_quay_tidebound_02';const client=new SnapshotClient();client.accept(invalid);assert.deepEqual(client.acceptEvents([event(999,{actorId:invalid.actors[0].entityId})]),[]);assert.equal(client.eventCursor(),0);
 const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([charge(999,{halfAngleRad:.8})],snapshot()),[]);assert.deepEqual(model.accept([charge(2)],snapshot()),[2]);
});

test('Normal and wet Swing warnings are exact committed sectors, including safe rear and side space',()=>{
 for(const reach of [1.4,2.2])for(const scale of [24,48,72]){
  const model=new ClockworksEnemyModel();const cue=event(1,{radiusM:reach,rangeM:reach});model.accept([cue],snapshot());const frame=model.project(snapshot(),true)[0],points=enemyWarningGroundPoints(frame);assert.equal(points.length,34);assert.deepEqual(points[0],point);assert.ok(points.slice(1).every(p=>Math.abs(Math.hypot(p.xM-16,p.zM-8)-reach)<1e-10));
  const r=renderer(),cam={...camera,pixelsPerMeter:scale};r.renderClockworksEnemyCues([frame],cam);const g=r.clockworksEnemyGraphics.get(1),polygon=new Polygon(g.context.instructions[0].data.path.instructions[0].data[0]);const contains=p=>{const q=projectWorldPoint(p,cam);return polygon.contains((q.x-g.position.x)/g.scale.x,(q.footY-g.position.y)/g.scale.y);};
  assert.ok(contains({...point,xM:16+reach*.8}));assert.equal(contains({...point,xM:16-reach*.3}),false);assert.equal(contains({...point,xM:16+.05,zM:8+reach*.8}),false);assert.ok(r.clockworksEnemyGlows.has(1));
  const moved=snapshot();moved.player.transform.positionM={xM:16,yM:0,zM:12};assert.deepEqual(model.project(moved,true)[0],frame);assert.equal(audioCueForPresentationEvent(cue).id,'audio.mh.tidebound.swing.warning');r.clearTransientPresentation();
 }
});

test('Normal and wet Charge use committed swept capsules and exact server motion without local retargeting',()=>{
 for(const [reach,radius]of [[2.365,.45],[3.85,.55]]){
  const model=new ClockworksEnemyModel();const cue=charge(1,{rangeM:reach,radiusM:radius});model.accept([cue],snapshot());const frame=model.project(snapshot(),false)[0],points=enemyWarningGroundPoints(frame),xs=points.map(p=>p.xM);assert.equal(points.length,66);assert.ok(Math.abs(Math.min(...xs)-(16-radius))<1e-10);assert.ok(Math.abs(Math.max(...xs)-(16+reach+radius))<1e-10);assert.equal(audioCueForPresentationEvent(cue).id,'audio.mh.tidebound.charge.warning');
  const motion=charge(2,{kind:'EnemyChargeMotion',positionM:{...point,xM:16.2},durationMs:400,rangeM:reach,radiusM:radius});assert.deepEqual(model.accept([motion],snapshot()),[2]);assert.deepEqual(model.project(snapshot(),false)[0].positionM,motion.positionM);assert.equal(audioCueForPresentationEvent(motion),null);
 }
});

test('Recovered wet impacts preserve their public geometry and terminal receipts suppress late motion',()=>{
 const model=new ClockworksEnemyModel();const wet=event(1,{kind:'EnemyAttackImpact',durationMs:170});assert.deepEqual(model.accept([wet],snapshot()),[1]);const frame=model.project(snapshot(),true)[0];assert.equal(frame.rangeM,2.2);assert.equal(frame.radiusM,2.2);assert.equal(frame.halfAngleRad,.8);assert.equal(frame.directionRad,Math.PI/2);assert.equal(audioCueForPresentationEvent(wet).id,'audio.mh.tidebound.swing.impact');assert.ok(!JSON.stringify(frame).match(/currentHp|maxHp|damage|terrainVariant|AIState/));
 assert.deepEqual(model.accept([charge(2,{kind:'EnemyAttackImpact',durationMs:170}),charge(3,{kind:'EnemyChargeMotion',durationMs:300})],snapshot()),[2]);assert.equal(model.project(snapshot(),true)[0].kind,'EnemyAttackImpact');assert.equal(audioCueForPresentationEvent(charge(2,{kind:'EnemyAttackImpact'})).id,'audio.mh.tidebound.charge.impact');
 assert.deepEqual(model.project(snapshot({worldEpoch:8,sceneId:'mh_breakwater'}),true),[]);
});

test('Future sector events wait for catch-up and malformed angle cannot poison ordering',()=>{
 const client=new SnapshotClient();client.accept(snapshot());assert.deepEqual(client.acceptEvents([event(999,{halfAngleRad:undefined}),event(2,{serverTick:101})]),[]);assert.equal(client.eventCursor(),0);client.accept(snapshot({serverTick:101,authorityRevision:101,ackSeq:101}));assert.deepEqual(client.acceptEvents([]).map(e=>e.eventId),[2]);
 const next=new SnapshotClient();next.accept(snapshot());next.acceptEvents([event(2,{serverTick:101})]);next.accept(snapshot({worldEpoch:8,serverTick:0,authorityRevision:0,ackSeq:0}));assert.deepEqual(next.acceptEvents([]),[]);assert.equal(next.eventCursor(),0);
});

test('Saved one-tick normal/wet Swing and Charge phases are shown while held before entry and ordered resume',async()=>{
 for(const row of [event(1,{rangeM:1.4,radiusM:1.4}),event(),charge(),charge(1,{kind:'EnemyChargeMotion'})]){
  let held=true,counter=1,current=snapshot(),committed=null;const model=new ClockworksEnemyModel(),rows=[{...row,durationMs:1000/60}],calls=[];const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:()=>1,clearInterval(){},requestAnimationFrame:()=>1,cancelAnimationFrame(){}};
  const client={events:async(epoch,cursor)=>rows.filter(e=>e.eventId>cursor),sceneReady:async()=>{assert.deepEqual(calls.at(-1).slice(0,3),['draw',true,1]);held=false;return current;},pause:async()=>{held=true;rows.push({...row,eventId:++counter,durationMs:1000/60});return current;},resume:async()=>{assert.equal(held,true);assert.deepEqual(calls.at(-1).slice(0,3),['draw',true,1]);held=false;return current;}};
  const render={render:async s=>{const frames=model.project(s,true);calls.push(['draw',held,frames.length]);if(frames.length){assert.equal(frames[0].remainingMs,1000/60);assert.equal(frames[0].rangeM,row.rangeM);assert.equal(frames[0].halfAngleRad,row.halfAngleRad);}committed=s;},isReadyFor:s=>committed?.worldEpoch===s.worldEpoch&&committed?.sceneId===s.sceneId,destroy(){}};
  const loop=new SessionLoop(client,render,{scheduler,onEventsWithSnapshot:(events,s)=>{const accepted=new Set(model.accept(events,s));return events.filter(e=>accepted.has(e.eventId));},onClearTransientPresentation:()=>model.reset()});await loop.start({...current,entryToken:{generation:7,worldId:'mist_harbor',sceneId:'mh_drowned_quay',worldEpoch:7}});assert.equal(loop.pausePresentationState,'running');await loop.pause();await loop.resume();assert.equal(loop.pausePresentationState,'running');await loop.stop('unload');
 }
});
