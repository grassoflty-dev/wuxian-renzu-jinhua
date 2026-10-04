import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const load=path=>import(pathToFileURL(root+path).href);
const {Assets,Container,Texture,TextureSource}=await load('apps/web/node_modules/pixi.js/lib/index.mjs');
const {AssetRegistry}=await load('apps/web/dist/assets/AssetRegistry.js');
const {WorldRenderer}=await load('apps/web/dist/renderer/WorldRenderer.js');
const {projectPropPresentation}=await load('apps/web/dist/renderer/PropPresentationModel.js');
test('registered console replaces actor whole-sheet selection on power state changes', async () => {
const priorWindow=globalThis.window, priorDocument=globalThis.document;
try {
const registry=new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(root+'governance/assets/RUNTIME_ASSET_MANIFEST.json')),
 new Uint8Array(await readFile(root+'governance/assets/AI_ASSET_RELEASE_MANIFEST.json')),async path=>new Uint8Array(await readFile(root+path)));
const scene=JSON.parse(await readFile(root+'content/scenes/compiled/gh_power_room.json','utf8'));
const s={kind:'full',protocolVersion:3,schemaVersion:'freeze-v02-interfaces/1.2',worldId:'grey_hive',sceneId:scene.sceneId,checkpointId:null,worldEpoch:1,serverTick:1,authorityRevision:1,ackSeq:1,
 player:{entityId:'player',transform:{positionM:{xM:2,yM:0,zM:7},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},
 actors:[{entityId:'power_console',entityType:'prop.grey_hive.power_console',actorKind:'prop',active:true,transform:{positionM:{xM:8,yM:0,zM:8},yawRad:0}}],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[{worldId:'grey_hive',completedEvents:[]}]}};
globalThis.window={matchMedia:()=>({matches:false})};globalThis.document={documentElement:{dataset:{motion:'full'}}};
const renderer=new WorldRenderer(registry);renderer.app={screen:{width:1280,height:720},renderer:{background:{}}};renderer.ready=true;
renderer.surface={isAvailable:()=>true,present:()=>true,invalidate(){},request(){}};
for(const layer of ['L0_BACKGROUND','L1_FLOOR','L2_BACK_PROPS','L3_ACTORS','L4_DYNAMIC_PROPS','L5_FRONT_PROPS','L6_OCCLUDERS','L7_VFX','L8_WORLD_UI']){const c=new Container();c.sortableChildren=true;renderer.layerContainers.set(layer,c);}
renderer.preloadAtlasPages=async pages=>{for(const page of pages)if(!renderer.atlasTextures.has(page.atlasUrl))renderer.atlasTextures.set(page.atlasUrl,new Texture({source:new TextureSource({width:8192,height:8192})}));};
renderer.expectSceneIdentity(s);renderer.setSceneDefinition(scene);await renderer.render(s);
const asset=registry.resolveAsset('runtime2d.prop.power_console.off_on.v1');
const frames=[];
for (const powered of [false,true,false]) {
 s.serverTick++; s.authorityRevision++;
 s.progression.worlds[0].completedEvents=powered?['hive_power']:[];
 await renderer.render(s);
 const sprite=renderer.sprites.get('actor:power_console').sprite;
 const f=sprite.texture.frame;
 const observed=[f.x,f.y,f.width,f.height];
 const expected=projectPropPresentation(asset,s,[]).asset.atlasFrame;
 assert.deepEqual(observed,expected);
 assert.notDeepEqual(observed,asset.atlasFrame);
 assert.ok(sprite.scale.x>0 && sprite.scale.y>0);
 frames.push(observed);
}
assert.notDeepEqual(frames[0],frames[1]);
assert.deepEqual(frames[0],frames[2]);
const mesh=renderer.facilityFloorMesh;
const buffers=[...new Set(mesh.geometry.buffers)];
const sharedTexture=mesh.texture;
renderer.clearFacilityFloor();
assert.equal(mesh.destroyed,true);
assert.ok(buffers.length>=3 && buffers.every(buffer=>buffer.destroyed));
assert.equal(sharedTexture.destroyed,false);
assert.equal(sharedTexture.source.destroyed,false);
} finally {
 if(priorWindow===undefined) delete globalThis.window; else globalThis.window=priorWindow;
 if(priorDocument===undefined) delete globalThis.document; else globalThis.document=priorDocument;
}
});
