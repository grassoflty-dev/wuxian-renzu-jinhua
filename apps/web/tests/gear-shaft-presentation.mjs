import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {loaded,snapshot} from './support/vertical-support-fixture.mjs';
import {parseVerticalSupports,projectVerticalSupports} from '../dist/renderer/VerticalSupportModel.js';
import {verifiedSceneSourceSha256} from '../dist/assets/SceneDefinitionLoader.js';
const raw=JSON.parse(await readFile(new URL('../../../content/scenes/compiled/cw_gear_shaft.json',import.meta.url),'utf8'));
function frame(sha,time){
 const phaseTime=time%12000;let phase,height,velocity,elapsed;
 if(phaseTime<2000){phase='lower_hold';height=0;velocity=0;elapsed=phaseTime;}
 else if(phaseTime<6000){phase='rising';height=Math.fround(2*Math.fround((phaseTime-2000)/4000));velocity=.5;elapsed=phaseTime-2000;}
 else if(phaseTime<8000){phase='upper_hold';height=2;velocity=0;elapsed=phaseTime-6000;}
 else{phase='falling';height=Math.fround(2-Math.fround(2*Math.fround((phaseTime-8000)/4000)));velocity=-.5;elapsed=phaseTime-8000;}
 const s=snapshot(sha,time+1,time);s.worldId='clockworks';s.sceneId='cw_gear_shaft';Object.assign(s.supportScene,{worldId:s.worldId,sceneId:s.sceneId});
 s.player.transform.positionM={xM:12,yM:height,zM:8};s.player.velocityMps={xM:0,yM:velocity,zM:0};
 s.supportScene.rider={positionM:{...s.player.transform.positionM},radiusM:.35,mode:'surface',supportId:'cw_gear_main_lift'};
 s.supportScene.poses=[{supportId:'cw_gear_furnace_landing',phase:'stationary',heightM:2,velocityMps:0,phaseElapsedMs:0},{supportId:'cw_gear_main_lift',phase,heightM:height,velocityMps:velocity,phaseElapsedMs:elapsed},{supportId:'cw_gear_upper_dock',phase:'stationary',heightM:2,velocityMps:0,phaseElapsedMs:0}];return s;
}
test('actual authenticated Gear Shaft lift and both decks project coherent boundary frames',async()=>{
 const {scene,sceneSha}=await loaded(raw);const plan={worldId:scene.worldId,sceneId:scene.sceneId,verticalSupports:parseVerticalSupports(scene.movingSupports,scene.standingDecks,scene.boundsM),verifiedSourceSha256:verifiedSceneSourceSha256(scene)};
 for(const time of [0,1999,2000,4000,5999,6000,7999,8000,10000,11999,12000]){
  const value=frame(sceneSha,time),out=projectVerticalSupports(plan,value);assert.equal(out.frames.length,3);assert.equal(out.frames.filter(f=>f.rider).length,1);assert.equal(out.frames.find(f=>f.rider).heightM,value.player.transform.positionM.yM);
 }
 const current=frame(sceneSha,6000);current.player.transform.positionM={xM:16,yM:2,zM:8};current.supportScene.rider={positionM:{...current.player.transform.positionM},radiusM:.35,mode:'surface',supportId:'cw_gear_upper_dock'};
 assert.equal(projectVerticalSupports(plan,current).frames.find(f=>f.rider).id,'cw_gear_upper_dock');
 current.player.transform.positionM.xM=19;current.supportScene.rider.positionM.xM=19;assert.throws(()=>projectVerticalSupports(plan,current),/SUPPORT_SCENE/,'the actual gap is not a standing surface');
});
