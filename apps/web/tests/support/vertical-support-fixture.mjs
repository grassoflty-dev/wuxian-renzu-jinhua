import { createHash } from 'node:crypto';
import { SceneDefinitionLoader } from '../../dist/assets/SceneDefinitionLoader.js';
export const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
export const definition=()=>({schemaVersion:1,worldId:'grey_hive',sceneId:'gh_support',boundsM:{x:0,z:0,width:10,depth:10},presentation:{cameraProfile:'oblique_default',layers:[],sprites:[]},movingSupports:[{id:'lift',polygon:[[1,1],[4,1],[4,4],[1,4]],lowerM:0,upperM:2,travelMs:1000,endpointHoldMs:250}],standingDecks:[{id:'deck',polygon:[[3,1],[6,1],[6,4],[3,4]],heightM:2}]});
export async function loaded(source=definition()) {
 const bytes=Buffer.from(JSON.stringify(source));const sceneSha=digest(bytes);const name=`scene-definitions/compiled/${source.worldId}/${source.sceneId}.json`;
 const manifest=Buffer.from(JSON.stringify({schemaVersion:1,scenes:[{worldId:source.worldId,sceneId:source.sceneId,path:name,sha256:sceneSha}]}));
 const loader=new SceneDefinitionLoader({baseUrl:'https://fixture.invalid/',expectedManifestSha256:digest(manifest),fetchImpl:async url=>new Response(new URL(url).pathname.endsWith('SCENE_DEFINITION_MANIFEST.json')?manifest:bytes)});
 return {scene:await loader.loadForSnapshot({...source,worldEpoch:4}),sceneSha};
}
export function snapshot(sourceSha,tick=44,time=750) {
 const t=time%2500;let phase,height,velocity,elapsed;
 if(t<250){phase='lower_hold';height=0;velocity=0;elapsed=t;}
 else if(t<1250){phase='rising';height=Math.fround(2*Math.fround((t-250)/1000));velocity=2;elapsed=t-250;}
 else if(t<1500){phase='upper_hold';height=2;velocity=0;elapsed=t-1250;}
 else{phase='falling';height=Math.fround(2-Math.fround(2*Math.fround((t-1500)/1000)));velocity=-2;elapsed=t-1500;}
 const position={xM:2,yM:height,zM:2};
 return {kind:'full',protocolVersion:3,schemaVersion:'freeze-v02-interfaces/1.2',worldId:'grey_hive',sceneId:'gh_support',checkpointId:null,worldEpoch:4,serverTick:tick,authorityRevision:tick,ackSeq:tick,
 player:{entityId:'player',transform:{positionM:position,yawRad:0},velocityMps:{xM:0,yM:velocity,zM:0},currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:1,facingZ:0,aimX:1,aimZ:0,actionState:'idle'},actors:[],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[]},
 supportScene:{schemaVersion:1,worldId:'grey_hive',sceneId:'gh_support',worldEpoch:4,serverTick:tick,authorityRevision:tick,serverTimeMs:time,sceneSourceSha256:sourceSha,
 poses:[{supportId:'deck',phase:'stationary',heightM:2,velocityMps:0,phaseElapsedMs:0},{supportId:'lift',phase,heightM:height,velocityMps:velocity,phaseElapsedMs:elapsed}],rider:{positionM:{...position},radiusM:0.35,mode:'surface',supportId:'lift'}}};
}
