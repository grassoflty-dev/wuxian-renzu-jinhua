import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { Container, Polygon, Sprite, Texture } from 'pixi.js';
import { ClockworksEnemyModel, CLOCKWORKS_ENEMY_PLACEHOLDERS, clockworksEnemyForEvent, enemyWarningGroundPoints } from '../dist/renderer/ClockworksEnemyModel.js';
import { WorldRenderer } from '../dist/renderer/WorldRenderer.js';
import { projectWorldPoint } from '../dist/renderer/CameraModel.js';
import { actorAssetId, buildRendererResourcePlan } from '../dist/renderer/RendererResourcePlan.js';
import { AssetRegistry } from '../dist/assets/AssetRegistry.js';
import { audioCueForPresentationEvent } from '../dist/audio/AudioEventMap.js';
import { SnapshotClient } from '../dist/game/SnapshotClient.js';
import { SessionLoop } from '../dist/game/SessionLoop.js';
const type='enemy.grey_hive.brute', id='gh_gate_b_brute_01', point={xM:17,yM:0,zM:8};
function snapshot(overrides={}) { return {kind:'full',protocolVersion:3,schemaVersion:'scene-v3/1',worldId:'grey_hive',sceneId:'gh_gate_b',worldEpoch:7,
 serverTick:100,authorityRevision:100,ackSeq:100,checkpointId:null,
 player:{entityId:'player',transform:{positionM:{xM:13.5,yM:0,zM:8},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:100,maxHp:100,
  currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},
 actors:[{entityId:id,entityType:type,actorKind:'enemy',active:true,transform:{positionM:point,yawRad:0}}],
 doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},
 progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[]},...overrides}; }
function event(eventId=1,overrides={}) { return {protocolVersion:2,eventId,worldEpoch:7,serverTick:100,kind:'EnemyAttackTelegraph',actorId:id,
 attackKind:'charge',durationMs:1100,rangeM:4.4,radiusM:.55,intensity:1,positionM:point,directionRad:-Math.PI/2,...overrides}; }
const camera={width:1200,height:800,origin:{xM:0,yM:0,zM:0},pixelsPerMeter:48};
function renderer(model=new ClockworksEnemyModel()) {
 const r=Object.create(WorldRenderer.prototype); r.layerContainers=new Map([['L3_ACTORS',new Container()],['L7_VFX',new Container()]]);
 r.clockworksEnemyModel=model;r.clockworksEnemyGraphics=new Map();r.clockworksEnemyGlows=new Map();r.clockworksEnemyBodies=new Map();r.sprites=new Map();
 r.sentinelTelegraphGraphics=new Map();r.sentinelGlowGraphics=new Map();r.sentinelTelegraphModel={reset(){}};r.transientPresentationRevision=0;r.machineryGraphics=new Map();return r;
}

test('Brute uses only an admitted temporary heavy humanoid and retains its contact shadow',async()=>{
 const root=new URL('../../../',import.meta.url), registry=new AssetRegistry();
 await registry.registerManifest(new Uint8Array(await readFile(new URL('governance/assets/RUNTIME_ASSET_MANIFEST.json',root))),
  new Uint8Array(await readFile(new URL('governance/assets/AI_ASSET_RELEASE_MANIFEST.json',root))),async path=>new Uint8Array(await readFile(new URL(path,root))));
 const actor=snapshot().actors[0],style=CLOCKWORKS_ENEMY_PLACEHOLDERS[type];assert.equal(style.temporary_visual,true);assert.equal(style.silhouette,'heavy');
 assert.equal(actorAssetId(actor),'runtime2d.enemy.infected_security.v1');assert.notEqual(style.assetId,'runtime2d.enemy.brute.v1');
 assert.ok(buildRendererResourcePlan(registry,'grey_hive',[actor],[]).assets.some(a=>a.assetId===style.assetId));
 const r=renderer();r.sprites.set(`actor:${id}`,{sprite:new Sprite(Texture.WHITE)});r.renderClockworksEnemyBodies([actor],camera);
 assert.ok(r.clockworksEnemyBodies.get(id).shadow.label.includes('enemy-contact-shadow'));
 assert.ok(r.clockworksEnemyBodies.get(id).outline.label.includes('enemy-heavy'));
 assert.equal(r.sprites.get(`actor:${id}`).sprite.height,86);r.clearClockworksEnemyBodies();
});

test('Brute actor, world, scene, attack family and positive geometry bind before cursor mutation',()=>{
 for(const [cue,state] of [[{actorId:'unknown'},{}],[{attackKind:'bite'},{}],[{attackKind:'pressure_shot'},{}],[{kind:'FurnaceHoundLeapMotion',attackKind:'charge'},{}],
  [{radiusM:0},{}],[{}, {worldId:'clockworks',sceneId:'cw_pressure_hall'}],[{}, {sceneId:'gh_power_room'}],[{}, {actors:[...snapshot().actors,...snapshot().actors]}]]){
  const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([event(999,cue)],snapshot(state)),[]);
  assert.deepEqual(model.accept([event(2)],snapshot()),[2]);
 }
 const client=new SnapshotClient();client.accept(snapshot());
 assert.deepEqual(client.acceptEvents([event(999,{actorId:'unknown'}),event(2,{radiusM:0}),event(1)]).map(e=>e.eventId),[1]);
 assert.equal(client.eventCursor(),1);
});

test('Charge capsule uses committed yaw, travel and hit width including both swept endpoint discs',()=>{
 const model=new ClockworksEnemyModel();model.accept([event()],snapshot());const frame=model.project(snapshot(),false)[0];
 const points=enemyWarningGroundPoints(frame);assert.equal(points.length,66);
 const xs=points.map(p=>p.xM),zs=points.map(p=>p.zM);
 assert.ok(Math.abs(Math.max(...xs)-(17+.55))<1e-10);assert.ok(Math.abs(Math.min(...xs)-(17-4.4-.55))<1e-10);
 assert.ok(Math.abs(Math.max(...zs)-8.55)<1e-10);assert.ok(Math.abs(Math.min(...zs)-7.45)<1e-10);
 const moved=snapshot();moved.player.transform.positionM={xM:100,yM:0,zM:-100};assert.deepEqual(model.project(moved,false)[0],frame);
 const r=renderer(model);r.renderClockworksEnemyCues([frame],camera);assert.ok(r.clockworksEnemyGlows.has(1));
 const graphic=r.clockworksEnemyGraphics.get(1),polygon=new Polygon(graphic.context.instructions[0].data.path.instructions[0].data[0]);
 for(const x of [17.5,17-4.4-.5]){const p=projectWorldPoint({...point,xM:x},camera);assert.ok(polygon.contains((p.x-graphic.position.x)/graphic.scale.x,(p.footY-graphic.position.y)/graphic.scale.y));}
 r.clearTransientPresentation();
});

test('Slam is a full radial warning with body glow and distinct temporary warning/impact sounds',()=>{
 for(const scale of [24,48,72]){
  const model=new ClockworksEnemyModel();const cue=event(1,{attackKind:'slam',rangeM:2,radiusM:2});model.accept([cue],snapshot());
  const frame=model.project(snapshot(),true)[0],ring=enemyWarningGroundPoints(frame);assert.equal(ring.length,64);
  assert.deepEqual(enemyWarningGroundPoints({...frame,directionRad:1.9}),ring);
  assert.ok(ring.every(p=>Math.abs(Math.hypot(p.xM-17,p.zM-8)-2)<1e-10));
  const r=renderer(model);r.renderClockworksEnemyCues([frame],{...camera,pixelsPerMeter:scale});
  assert.equal(r.clockworksEnemyGraphics.get(1).context.instructions.length,1,'radial ring has no misleading forward arrow');
  assert.ok(r.clockworksEnemyGlows.has(1));r.clearTransientPresentation();
  assert.equal(audioCueForPresentationEvent(cue).id,'audio.gh.brute.slam.warning');
  assert.equal(audioCueForPresentationEvent({...cue,kind:'EnemyAttackImpact'}).id,'audio.gh.brute.slam.impact');
  assert.equal(audioCueForPresentationEvent(cue).temporary_audio,true);
 }
 assert.equal(audioCueForPresentationEvent(event()).id,'audio.gh.brute.charge.warning');
 assert.equal(audioCueForPresentationEvent(event(2,{kind:'EnemyAttackImpact'})).id,'audio.gh.brute.charge.impact');
});

test('Actual charge points, stagger, death and scene replacement never extrapolate or leak numeric HP',()=>{
 const model=new ClockworksEnemyModel();model.accept([event()],snapshot());
 const motion=event(2,{kind:'EnemyChargeMotion',positionM:{...point,xM:16},durationMs:400});model.accept([motion],snapshot());
 assert.deepEqual(model.project(snapshot(),false)[0].positionM,motion.positionM);
 assert.equal(audioCueForPresentationEvent(motion),null,'no sound on every motion tick');
 assert.deepEqual(model.accept([event(3,{kind:'EnemyStagger',attackKind:undefined,rangeM:undefined})],snapshot()),[3]);
 const dead=snapshot({actors:snapshot().actors.map(a=>({...a,active:false}))});
 assert.deepEqual(model.accept([event(4,{kind:'EnemyDeath',attackKind:undefined,rangeM:undefined})],dead),[4]);
 assert.ok(!JSON.stringify(model.project(dead,true)).match(/currentHp|maxHp|damage|AIState/));
 assert.deepEqual(model.project(snapshot({worldEpoch:8,sceneId:'gh_deep_decon'}),false),[]);
});

test('One-tick restored Brute Charge and Slam are drawn while held before entry and ordered resume',async()=>{
 for(const [kind,attackKind] of [['EnemyAttackTelegraph','charge'],['EnemyChargeMotion','charge'],['EnemyAttackTelegraph','slam']]){
  let held=true,counter=1,current=snapshot(),committed=null;const model=new ClockworksEnemyModel(),rows=[event(counter,{kind,attackKind,durationMs:1000/60,rangeM:attackKind==='slam'?2:4.4})],calls=[];
  const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:()=>1,clearInterval(){},requestAnimationFrame:()=>1,cancelAnimationFrame(){}};
  const client={events:async(epoch,cursor)=>rows.filter(e=>e.eventId>cursor),sceneReady:async()=>{assert.equal(calls.at(-1)[0],'draw');assert.equal(calls.at(-1)[1],true);assert.equal(calls.at(-1)[2],1);held=false;return current;},
   pause:async()=>{held=true;rows.push(event(++counter,{kind,attackKind,durationMs:1000/60,rangeM:attackKind==='slam'?2:4.4}));return current;},
   resume:async()=>{assert.equal(held,true);assert.deepEqual(calls.at(-1).slice(0,3),['draw',true,1]);held=false;return current;}};
  const render={render:async s=>{const frames=model.project(s,true);calls.push(['draw',held,frames.length]);if(frames.length)assert.equal(frames[0].remainingMs,1000/60);committed=s;},isReadyFor:s=>committed?.worldEpoch===s.worldEpoch&&committed?.sceneId===s.sceneId,destroy(){}};
  const loop=new SessionLoop(client,render,{scheduler,onEventsWithSnapshot:(events,s)=>{const accepted=new Set(model.accept(events,s));return events.filter(e=>accepted.has(e.eventId));},onClearTransientPresentation:()=>model.reset()});
  const initial={...current,entryToken:{generation:7,worldId:'grey_hive',sceneId:'gh_gate_b',worldEpoch:7}};
  await loop.start(initial);assert.equal(loop.pausePresentationState,'running');await loop.pause();await loop.resume();assert.equal(loop.pausePresentationState,'running');await loop.stop('unload');
 }
});
