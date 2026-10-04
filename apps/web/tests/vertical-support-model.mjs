import test from 'node:test';
import assert from 'node:assert/strict';
import { loaded,definition,snapshot } from './support/vertical-support-fixture.mjs';
import { assertSnapshotV3 } from '../dist/protocol/types.js';
import { SnapshotClient } from '../dist/game/SnapshotClient.js';
import { verifiedSceneSourceSha256 } from '../dist/assets/SceneDefinitionLoader.js';
import { planScenePresentation } from '../dist/renderer/ScenePresentation.js';
import { projectVerticalSupports,parseVerticalSupports } from '../dist/renderer/VerticalSupportModel.js';
import { supportGeometryKey } from '../dist/protocol/SupportProjection.js';
const lookup={resolveAsset(){throw Error('unexpected asset');}};

test('authenticated source proof is exact-object bound, immutable, and required for supports',async()=>{
 const {scene,sceneSha}=await loaded();assert.equal(verifiedSceneSourceSha256(scene),sceneSha);assert.ok(Object.isFrozen(scene.movingSupports[0].polygon));
 assert.throws(()=>scene.movingSupports[0].upperM=3,TypeError);assert.equal(verifiedSceneSourceSha256({...scene}),undefined);
 assert.throws(()=>planScenePresentation(lookup,definition()),/SOURCE_UNVERIFIED/);const plan=planScenePresentation(lookup,scene);assert.equal(plan.verifiedSourceSha256,sceneSha);
});
test('phase and rider projection match all cycle boundaries without a local clock',async()=>{
 const {scene,sceneSha}=await loaded();const plan=planScenePresentation(lookup,scene);
 for(const time of [0,249,250,750,1249,1250,1499,1500,2000,2499,2500,1000000]){const value=snapshot(sceneSha,time+1,time);assert.equal(assertSnapshotV3(value),value);const a=projectVerticalSupports(plan,value),b=projectVerticalSupports(plan,value);assert.deepEqual(a,b);assert.equal(a.frames[1].heightM,value.player.transform.positionM.yM);assert.ok(a.frames[1].rider);}
});
test('malformed identity, phase, duplicate/reordered/extra catalog, and rider shapes fail closed',async()=>{
 const {scene,sceneSha}=await loaded();const plan=planScenePresentation(lookup,scene);
 const mutate=[s=>s.supportScene.worldEpoch++,s=>s.supportScene.serverTick++,s=>s.supportScene.serverTimeMs=NaN,s=>s.supportScene.rider.radiusM=0,s=>s.supportScene.rider.positionM.yM+=.1,s=>s.supportScene.poses.reverse(),s=>s.supportScene.poses.push({...s.supportScene.poses[1]}),s=>s.supportScene.poses[1].phase='stationary',s=>s.supportScene.poses[1].phase={toString:()=> 'rising'},s=>s.supportScene.rider.mode={toString:()=> 'surface'},s=>s.supportScene.rider.supportId='missing',s=>s.supportScene.poses[0].phaseElapsedMs=1,s=>s.supportScene.extra=true];
 for(const change of mutate){const value=snapshot(sceneSha);change(value);assert.throws(()=>projectVerticalSupports(plan,value),/SUPPORT/);}
});
test('stale scene-file proof and absent projection never authorize new support geometry',async()=>{
 const {scene,sceneSha}=await loaded();const plan=planScenePresentation(lookup,scene);const value=snapshot(sceneSha);delete value.supportScene;assert.throws(()=>projectVerticalSupports(plan,value),/REQUIRED/);
 const changed=definition();changed.presentation.cameraProfile='facility_oblique';const next=await loaded(changed);assert.throws(()=>projectVerticalSupports(plan,snapshot(next.sceneSha)),/SOURCE_MISMATCH/);
 assert.throws(()=>projectVerticalSupports({...plan,verifiedSourceSha256:undefined},snapshot(sceneSha)),/SOURCE_MISMATCH/);
 assert.throws(()=>projectVerticalSupports(null,snapshot(sceneSha)),/UNEXPECTED/);
});
test('individually plausible but source-mismatched phase/time and edge contact are rejected',async()=>{
 const {scene,sceneSha}=await loaded();const plan=planScenePresentation(lookup,scene);
 for(const change of [s=>s.supportScene.serverTimeMs++,s=>s.supportScene.poses[1].velocityMps=3,s=>{s.player.transform.positionM.xM=1.1;s.supportScene.rider.positionM.xM=1.1;},s=>{s.supportScene.poses[1].supportId='renamed';s.supportScene.rider.supportId='renamed';}]){const value=snapshot(sceneSha);change(value);assert.throws(()=>projectVerticalSupports(plan,value),/SUPPORT_SCENE/);}
});
test('snapshot ownership uses one current rider pose and rejects same-context phase withdrawal/regression',async()=>{
 const {sceneSha}=await loaded();const client=new SnapshotClient(),before=snapshot(sceneSha),after=snapshot(sceneSha,45,767);client.accept(before);client.accept(after);for(const alpha of [0,.2,.5,1])assert.deepEqual(client.interpolatedPlayer(alpha),after.player.transform.positionM);
 const missing=snapshot(sceneSha,46,784);delete missing.supportScene;assert.throws(()=>client.accept(missing),/CONTINUITY/);
 const wrongClock=snapshot(sceneSha,46,766);assert.throws(()=>client.accept(wrongClock),/CONTINUITY/);
 const unchangedTick=snapshot(sceneSha,45,768);unchangedTick.authorityRevision++;unchangedTick.supportScene.authorityRevision++;assert.throws(()=>client.accept(unchangedTick),/CONTINUITY/);
});
test('authority-only Ready or pause receipts keep geometry proof while changed held poses require a new key',async()=>{
 const {sceneSha}=await loaded();const before=snapshot(sceneSha),receipt=structuredClone(before);receipt.authorityRevision++;receipt.supportScene.authorityRevision++;assert.equal(supportGeometryKey(before.supportScene),supportGeometryKey(receipt.supportScene));assert.notEqual(supportGeometryKey(before.supportScene),supportGeometryKey(snapshot(sceneSha,45,767).supportScene));
 const client=new SnapshotClient();client.accept(before);assert.equal(client.accept(receipt),true);
});
test('support geometry parser rejects malformed definitions without changing canonical empty scenes',()=>{
 assert.deepEqual(parseVerticalSupports(undefined,undefined,{x:0,z:0,width:10,depth:10}),[]);
 for(const change of [d=>d.movingSupports[0].travelMs=249,d=>d.movingSupports[0].extra=true,d=>d.standingDecks[0].id='lift',d=>d.standingDecks[0].heightM=3.1,d=>d.standingDecks[0].polygon=[[1,1],[4,4],[4,1],[1,4]]]){const d=definition();change(d);assert.throws(()=>parseVerticalSupports(d.movingSupports,d.standingDecks,d.boundsM),/GEOMETRY/);}
});
