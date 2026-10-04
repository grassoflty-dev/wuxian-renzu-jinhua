import test from "node:test";
import assert from "node:assert/strict";
import {projectWardenEncounter,WARDEN_ID} from "../dist/renderer/WardenEncounterModel.js";
import {SoundCueModel} from "../dist/renderer/SoundCueModel.js";
import {WorldRenderer} from "../dist/renderer/WorldRenderer.js";
import {AudioCuePlayer} from "../dist/audio/AudioCuePlayer.js";
import {AUDIO_CUE_LIBRARY} from "../dist/audio/AudioEventMap.js";
import {Container} from "pixi.js";
function snapshot(mapped=false){return {protocolVersion:3,worldId:"mist_harbor",sceneId:"mh_warden_arena",worldEpoch:4,serverTick:60,
  player:{transform:{positionM:{xM:16,yM:0,zM:8}}},actors:[{entityId:WARDEN_ID,entityType:"runtime2d.enemy.mistharbor.signal_wraith.v2",active:true}],
  capabilities:{items:[{capabilityId:"perception.acoustic_mapping_i",granted:mapped,selected:mapped}]},
  bossEncounter:{entityId:WARDEN_ID,entityType:"enemy.mist_harbor.resonance_warden",displayName:"Resonance Warden",currentHp:240,maxHp:360,phase:1,state:"windup",temporaryVisual:true,publicReleaseEligible:false,mappedTrueSource:mapped,
    warning:{attackSerial:1,kind:"strike",originM:[18,0,8],directionRad:-Math.PI/2,radiusM:5.5,halfAngleRad:.55,remainingMs:mapped?1000:500,ordinaryVisible:!mapped}}};}
test("ordinary and mapped warnings share real geometry; mapping supplies an earlier true-source outline",()=>{
  const normal=projectWardenEncounter(snapshot(),true),mapped=projectWardenEncounter(snapshot(true),true);
  assert.equal(normal.source,null);assert.deepEqual(normal.warning.polygonM,mapped.warning.polygonM);
  assert.deepEqual(mapped.source,{xM:18,yM:0,zM:8});assert.equal(normal.hpRatio,2/3);assert.equal(normal.warning.kind,"strike");
  const early=snapshot();early.bossEncounter.warning.remainingMs=1000;early.bossEncounter.warning.ordinaryVisible=false;assert.equal(projectWardenEncounter(early).warning,null);
  const lost=snapshot(true);lost.capabilities.items[0].selected=false;assert.equal(projectWardenEncounter(lost),null);
});
test("decoys have distinct inert visual feedback and never receive the mapped true-source outline",()=>{
  const snap=snapshot();snap.bossEncounter.state="decoy";snap.bossEncounter.phase=2;snap.bossEncounter.warning={...snap.bossEncounter.warning,kind:"decoy",originM:[18,0,6],remainingMs:400,radiusM:1,halfAngleRad:Math.fround(Math.PI)};
  const frame=projectWardenEncounter(snap);assert.equal(frame.warning.label,"回声");assert.equal(frame.source,null);
  assert.notEqual(frame.warning.color,projectWardenEncounter(snapshot()).warning.color);
  snap.bossEncounter.mappedTrueSource=true;assert.equal(projectWardenEncounter(snap),null);
});
test("projection rejects ambiguous identity, malformed geometry and release-promotion flags",()=>{
  for(const patch of [{entityId:"fake"},{currentHp:0},{currentHp:361},{maxHp:0},{phase:3},{state:"dead"},{temporaryVisual:false},{publicReleaseEligible:true},{mappedTrueSource:"yes"}]){
    const snap=snapshot();Object.assign(snap.bossEncounter,patch);assert.equal(projectWardenEncounter(snap),null,JSON.stringify(patch));}
  for(const patch of [{originM:[NaN,0,8]},{originM:[18,1,8]},{radiusM:Infinity},{attackSerial:0},{remainingMs:0},{kind:"damage"},{halfAngleRad:0}]){
    const snap=snapshot();Object.assign(snap.bossEncounter.warning,patch);assert.equal(projectWardenEncounter(snap),null);}
  const duplicate=snapshot();duplicate.actors.push(duplicate.actors[0]);assert.equal(projectWardenEncounter(duplicate),null);
  for(const patch of [{worldId:"grey_hive"},{sceneId:"mh_resonance_tower"},{worldEpoch:0},{bossEncounter:undefined}])assert.equal(projectWardenEncounter({...snapshot(),...patch}),null);
});
test("pause is deterministic, reduced motion removes pulse, and renderer clears warning and outline on death or exit",()=>{
  const snap=snapshot(true),frame=projectWardenEncounter(snap,true);assert.deepEqual(projectWardenEncounter(snap,true),frame);
  assert.deepEqual(projectWardenEncounter({...snap,serverTick:90},true),frame);
  assert.notEqual(projectWardenEncounter(snap,false).warning.alpha,projectWardenEncounter({...snap,serverTick:90},false).warning.alpha);
  const renderer=Object.create(WorldRenderer.prototype);renderer.wardenWarning=null;renderer.layerContainers=new Map([["L7_VFX",new Container()]]);
  const camera={origin:{xM:0,yM:0,zM:0},width:1280,height:720,pixelsPerMeter:48};
  renderer.renderWardenWarning(frame,camera);const old=renderer.wardenWarning;assert.ok(old.graphic);
  renderer.renderWardenWarning(null,camera);assert.equal(renderer.wardenWarning,null);assert.ok(old.graphic.destroyed&&old.label.destroyed);
  renderer.renderWardenWarning(frame,camera);renderer.clearWardenWarning();assert.equal(renderer.layerContainers.get("L7_VFX").children.length,0);
});
test("true call mapping is ephemeral, permission-bound and cannot reveal a prior unmapped call",()=>{
  const model=new SoundCueModel(),snap=snapshot(true);
  const event={protocolVersion:1,eventId:1,worldEpoch:4,serverTick:60,worldId:snap.worldId,sceneId:snap.sceneId,kind:"warden_true_call",directionRad:Math.PI/2,distanceM:2};
  model.accept([event],snap,0);assert.equal(model.view(snap,1).sourceLabel,"真实声源");
  model.accept([{...event,directionRad:null,distanceM:null}],snapshot(),2);assert.equal(model.view(snap,3),null);
  model.accept([event],snap,4);assert.equal(model.view({...snap,worldEpoch:5},5),null);
});
test("ordinary Warden audio is distinct, panned from the current player, deduplicated, and stopped by pause",async()=>{
  const played=[];const runtime={state:()=>"running",unlock:async()=>true,play:(cue,volume,pan)=>{played.push({id:cue.id,volume,pan});return true;},startAmbient:()=>true,stopVoices(){},stopAmbient(){},close(){}};
  const audio=new AudioCuePlayer({getStorage:()=>null,createRuntime:()=>runtime});await audio.unlock();audio.setListenerSnapshot(snapshot());
  const event=(kind,id,x=18,z=8)=>({protocolVersion:2,worldEpoch:4,eventId:id,serverTick:60,kind,positionM:{xM:x,yM:0,zM:z},directionRad:0,radiusM:3,intensity:1});
  assert.equal(audio.handlePresentationEvents([event("ResonanceWardenStrikeWindup",1)]),1);assert.ok(played[0].pan>0);
  assert.equal(audio.handlePresentationEvents([event("ResonanceWardenDecoy",2,14,8)]),1);assert.ok(played[1].pan<0);
  assert.notEqual(played[0].id,played[1].id);assert.equal(audio.handlePresentationEvents([event("ResonanceWardenDecoy",2)]),0);
  audio.suspend();assert.equal(audio.handlePresentationEvents([event("ResonanceWardenPulseWindup",3)]),0);audio.resume();
  audio.setListenerSnapshot({...snapshot(),sceneId:"mh_extraction"});assert.equal(audio.handlePresentationEvents([event("ResonanceWardenPulseWindup",4)]),0);
  const recipes=Object.entries(AUDIO_CUE_LIBRARY).filter(([id])=>id.startsWith("audio.mh.warden."));assert.equal(recipes.length,6);
  assert.equal(new Set(recipes.map(([,cue])=>JSON.stringify(cue.tones))).size,6);assert.ok(recipes.every(([,cue])=>cue.temporary_audio&&cue.peakGain<=.22));
  await audio.dispose();
});

test("explicit resolved mapping permission overrides legacy selection metadata fail-closed",()=>{
  const off=snapshot(true);off.capabilities.acousticMappingAuthorized=false;assert.equal(projectWardenEncounter(off),null);
  const on=snapshot(true);on.capabilities.items=[];on.capabilities.acousticMappingAuthorized=true;assert.ok(projectWardenEncounter(on).source);
});
