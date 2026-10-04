import test from 'node:test';
import assert from 'node:assert/strict';
import { Container, Graphics } from 'pixi.js';
import { SnapshotClient } from '../dist/game/SnapshotClient.js';
import { validPresentationEventEnvelope, combatFeedbackMatchesSnapshot } from '../dist/protocol/PresentationEventValidation.js';
import { assertSnapshotV3 } from '../dist/protocol/types.js';
import { CombatFeedbackModel, combatRejectionText, MAX_COMBAT_FEEDBACK } from '../dist/renderer/CombatFeedbackModel.js';
import { CombatPresentationLayer, combatContactDepth, drawActionPresentation } from '../dist/renderer/CombatPresentationLayer.js';
import { ActionVfxModel } from '../dist/renderer/ActionVfxModel.js';
import { projectWorldPoint } from '../dist/renderer/CameraModel.js';
import { audioCueForPresentationEvent } from '../dist/audio/AudioEventMap.js';

const position = {xM:3,yM:2,zM:4};
const snapshot = (tick=100, changes={}) => ({kind:'full',protocolVersion:3,schemaVersion:'freeze-v02-interfaces/1.2',
 worldId:'grey_hive',sceneId:'gh_gate_b',checkpointId:null,worldEpoch:7,serverTick:tick,authorityRevision:tick,ackSeq:tick,
 player:{entityId:'player',transform:{positionM:{xM:0,yM:2,zM:0},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},
 currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:1,facingZ:0,aimX:1,aimZ:0,actionState:'idle'},
 actors:[{entityId:'enemy_01',entityType:'enemy.grey_hive.brute',actorKind:'enemy',active:true,transform:{positionM:position,yawRad:0}}],
 doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[]},...changes});
const event = (id=1,tick=100,changes={}) => ({protocolVersion:2,eventId:id,worldEpoch:7,serverTick:tick,kind:'Hit',
 positionM:{...position},directionRad:0,radiusM:1,intensity:1,durationMs:220,
 combatFeedback:{worldId:'grey_hive',sceneId:'gh_gate_b',outcome:'enemy_hit',targetId:'enemy_01',sourceId:'player',requestId:11},...changes});
const outcome = (name,kind,duration=260) => event(1,100,{kind,durationMs:duration,combatFeedback:{worldId:'grey_hive',sceneId:'gh_gate_b',outcome:name,targetId:'player',sourceId:'enemy_01',...(name==='absorbed'?{reason:'invulnerable'}:{})}});
const phase = (changes={}) => ({requestId:11,phase:'windup',elapsedMs:17,durationMs:590,rangeM:2.6,lineHalfWidthM:0,...changes});
const action = (changes={}) => ({worldId:'grey_hive',sceneId:'gh_gate_b',worldEpoch:7,serverTick:100,actionState:'pulse',
 actionPresentation:phase(),position:{...position},facingX:1,facingZ:0,nowMs:1000,...changes});

test('owner result determines distinct hit, hurt, block, absorbed and rejection; absent metadata never invents contact',()=>{
 const cases=[event(),outcome('player_hurt','Damaged'),outcome('blocked','GuardImpact'),outcome('absorbed','DamageAbsorbed',160),
 event(1,100,{kind:'ActionRejected',durationMs:1000,combatFeedback:{worldId:'grey_hive',sceneId:'gh_gate_b',outcome:'rejected',sourceId:'player',requestId:11,reason:'cooldown'}})];
 for(const e of cases){const m=new CombatFeedbackModel();assert.deepEqual(m.accept([e],snapshot(),1000),[1]);assert.equal(m.project(snapshot(),true,1000)[0].outcome,e.combatFeedback.outcome);}
 const m=new CombatFeedbackModel();assert.deepEqual(m.accept([event(1,100,{combatFeedback:undefined})],snapshot(),1000),[]);
 const changed=snapshot();changed.player.currentHp=65;assert.deepEqual(m.project(changed,false,1000),[]);
 assert.equal(audioCueForPresentationEvent(event()).id,'audio.combat.hit');
 assert.equal(audioCueForPresentationEvent(cases[1]).id,'audio.combat.hurt');
 assert.equal(audioCueForPresentationEvent(cases[2]).id,'audio.skill.guard.hit');
 assert.equal(audioCueForPresentationEvent(cases[3]).id,'audio.combat.absorbed');
 assert.equal(audioCueForPresentationEvent(cases[4]),null);
 assert.equal(combatRejectionText([cases[4]],[1]),'技能仍在冷却');assert.equal(combatRejectionText([cases[4]],[]),null);
});

test('malformed context, unsafe request IDs and false kind/outcome cannot consume the transport cursor',()=>{
 const c=new SnapshotClient();c.accept(snapshot());
 for(const change of [{worldId:'clockworks'},{sceneId:'gh_other'},{requestId:Number.MAX_SAFE_INTEGER+1},{sourceId:'intruder'},{outcome:'blocked'},
 {targetId:'unknown'},{targetId:'enemy 01'}]){
 const invalid=event(999,100,{combatFeedback:{...event().combatFeedback,...change}});
 assert.deepEqual(c.acceptEvents([invalid]),[]);assert.equal(c.eventCursor(),0);
 }
 assert.equal(validPresentationEventEnvelope(event(1,100,{durationMs:2001})),false);
 assert.deepEqual(c.acceptEvents([event()]).map(e=>e.eventId),[1]);
});

test('future events remain deferred, dispatch once at their tick, and delayed/duplicate batches cannot restart TTL',()=>{
 const c=new SnapshotClient(),m=new CombatFeedbackModel();c.accept(snapshot());
 assert.deepEqual(c.acceptEvents([event(1,101)]),[]);assert.equal(c.eventCursor(),0);
 c.accept(snapshot(105));const received=c.acceptEvents([]);assert.equal(received.length,1);
 assert.deepEqual(m.accept(received,snapshot(105),1000),[1]);assert.ok(m.project(snapshot(105),false,1000)[0].remainingMs<160);
 assert.deepEqual(m.accept(received,snapshot(105),1100),[]);
 assert.equal(m.project(snapshot(105),false,1160).length,0,'wall clock expiry also removes a mark while transport stops advancing');
 assert.deepEqual(m.accept([event(2,100)],snapshot(114),1200),[],'expired rows cannot replay in a late response');
});

test('same-tick lethal enemy remains anchored; hidden signal-wraith true positions fail closed',()=>{
 const dead=snapshot();dead.actors[0].active=false;
 assert.equal(combatFeedbackMatchesSnapshot(event(),dead),true);
 const m=new CombatFeedbackModel();m.accept([event()],dead,1000);assert.deepEqual(m.project(dead,true,1000)[0].positionM,position);
 const hidden=snapshot(100,{worldId:'mist_harbor',sceneId:'mh_signal_yard'});
 hidden.actors=[{entityId:'mh_signal_yard_signal_wraith_01',entityType:'enemy.mist_harbor.signal_wraith',actorKind:'enemy',active:true,
 transform:{positionM:{xM:3,yM:0,zM:4},yawRad:0},signalPerception:{precise:false,uncertaintyRadiusM:1,positionsM:[{xM:2.5,yM:0,zM:4},{xM:3.5,yM:0,zM:4}]}}];
 const hit=event(1,100,{combatFeedback:{...event().combatFeedback,worldId:hidden.worldId,sceneId:hidden.sceneId,targetId:hidden.actors[0].entityId}});
 assert.equal(combatFeedbackMatchesSnapshot(hit,hidden),false);
 hidden.actors[0].signalPerception={precise:true,uncertaintyRadiusM:0,positionsM:[{xM:3,yM:0,zM:4}]};
 assert.equal(combatFeedbackMatchesSnapshot(hit,hidden),true);
});

test('scene, epoch, fatal snapshot and explicit lifecycle clear remove every bounded feedback',()=>{
 for(const change of [{sceneId:'gh_other'},{worldEpoch:8},{player:{...snapshot().player,currentHp:0}}]){
 const m=new CombatFeedbackModel();m.accept([event()],snapshot(),1000);assert.deepEqual(m.project(snapshot(101,change),true,1001),[]);
 }
 const m=new CombatFeedbackModel();m.accept(Array.from({length:MAX_COMBAT_FEEDBACK+20},(_,i)=>event(i+1)),snapshot(),1000);
 assert.equal(m.project(snapshot(),true,1000).length,MAX_COMBAT_FEEDBACK);m.reset();assert.deepEqual(m.project(snapshot(),true,1000),[]);
});

test('QER presentation requires actual phase; no local clock can authorize protection or impact',()=>{
 const m=new ActionVfxModel();assert.equal(m.update(action({actionPresentation:undefined})),null);
 assert.equal(m.update(action()).phase,'windup');assert.equal(m.update(action({nowMs:1200})).phase,'windup');assert.equal(m.update(action({nowMs:1250})),null);
 const guard=action({actionState:'guard',actionPresentation:phase({phase:'active',elapsedMs:100,durationMs:880,rangeM:0}),serverTick:101,nowMs:1300});
 assert.equal(m.update(guard).phase,'active');
 assert.equal(m.update({...guard,serverTick:102,actionPresentation:{...guard.actionPresentation,phase:'recovery',elapsedMs:117},nowMs:1310}).phase,'recovery');
 for(const broken of [{requestId:Number.MAX_SAFE_INTEGER+1},{elapsedMs:590},{lineHalfWidthM:-1},{phase:'guardStart'}]){
 const view=snapshot();view.player.actionState='pulse';view.player.actionPresentation=phase(broken);assert.throws(()=>assertSnapshotV3(view));
 }
});

test('contact marks use actor/prop depth, retain elevated contact height, and reduced-motion symbols remain visible',()=>{
 const camera={width:800,height:600,origin:{xM:0,yM:0,zM:0},pixelsPerMeter:48};
 const m=new CombatFeedbackModel();m.accept([event()],snapshot(),1000);const frames=m.project(snapshot(),true,1000);
 const anchors=[{targetId:'enemy_01',footY:468,zIndex:4},{footY:490,zIndex:5}];
 assert.equal(combatContactDepth(frames[0],camera,anchors),4.12);
 assert.ok(combatContactDepth({...frames[0],targetId:'dead'},camera,anchors)<5,'lethal contact stays behind the wall');
 const layer=new Container(),renderer=new CombatPresentationLayer();renderer.render(frames,layer,camera,f=>combatContactDepth(f,camera,anchors));
 assert.equal(layer.children[0].zIndex,4.12);assert.equal(layer.children[0].position.y,projectWorldPoint(position,camera).y-18);assert.equal(layer.children[0].alpha,1);
 renderer.render(m.project(snapshot(105),true,1020),layer,camera,()=>4.12);assert.equal(layer.children[0].alpha,1);
 renderer.render(frames,layer,camera,()=>null);assert.equal(layer.children.length,0,'hidden placement cannot tint or draw');
 const cue=new Graphics();cue.zIndex=4.1;drawActionPresentation(cue,new ActionVfxModel().update(action()),camera,true);
 assert.equal(cue.zIndex,4.1,'helper preserves owner depth instead of forcing top layer');assert.equal(cue.position.y,projectWorldPoint(position,camera).y);
 renderer.clear();cue.destroy();layer.destroy();
});
