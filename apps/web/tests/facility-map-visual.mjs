import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile,readdir} from 'node:fs/promises';
import {Container,Texture,TextureSource} from 'pixi.js';
import {AssetRegistry} from '../dist/assets/AssetRegistry.js';
import {planScenePresentation,SceneResourceTracker} from '../dist/renderer/ScenePresentation.js';
import {SceneDefinitionLoader} from '../dist/assets/SceneDefinitionLoader.js';
import {facilitySlice,isFacilityCollection} from '../dist/renderer/FacilityMapModel.js';
import {facilityFloorGeometry} from '../dist/renderer/FacilityFloorGeometry.js';
import {projectWorldPoint} from '../dist/renderer/CameraModel.js';
import {worldDepthRanks} from '../dist/renderer/WorldDepthModel.js';
import {developerPresentationEnabled} from '../dist/ui/DeveloperPresentation.js';
import {WorldRenderer} from '../dist/renderer/WorldRenderer.js';
import {createHash} from 'node:crypto';
const root=new URL('../../../',import.meta.url), json=async p=>JSON.parse(await readFile(new URL(p,root),'utf8'));
const registry=new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(new URL('governance/assets/RUNTIME_ASSET_MANIFEST.json',root))),new Uint8Array(await readFile(new URL('governance/assets/AI_ASSET_RELEASE_MANIFEST.json',root))),async p=>new Uint8Array(await readFile(new URL(p,root))));
const entry=await json('content/scenes/compiled/gh_entry_maintenance.json');
const plan=planScenePresentation(registry,entry);
const camera={width:1280,height:720,origin:{xM:2,yM:0,zM:7},pixelsPerMeter:48};
const digest=b=>createHash('sha256').update(b).digest('hex');
async function verified(scene){const bytes=Buffer.from(JSON.stringify(scene));const m=Buffer.from(JSON.stringify({schemaVersion:1,scenes:[{worldId:scene.worldId,sceneId:scene.sceneId,path:`scene-definitions/compiled/${scene.worldId}/${scene.sceneId}.json`,sha256:digest(bytes)}]}));return new SceneDefinitionLoader({baseUrl:'https://fixture.invalid/',expectedManifestSha256:digest(m),fetchImpl:async u=>new Response(String(u).endsWith('SCENE_DEFINITION_MANIFEST.json')?m:bytes)}).loadForSnapshot({...scene,worldEpoch:1});}
test('all thirty admitted scenes preserve source data and no facility collection remains an unsliced sprite',async()=>{
 let scenes=0,facilities=0;
 for(const name of await readdir(new URL('content/scenes/compiled/',root))){if(!name.endsWith('.json'))continue;const scene=await json('content/scenes/compiled/'+name),before=JSON.stringify(scene);const p=planScenePresentation(registry,await verified(scene));scenes++;if(p.facilityFloor)facilities++;
  for(const s of p.sprites)if(isFacilityCollection(s.asset.assetId)){assert.ok(s.displaySizeM);assert.notDeepEqual(s.asset.atlasFrame,registry.resolveAsset(s.asset.assetId).atlasFrame);}
  assert.equal(JSON.stringify(scene),before);
 }
 assert.equal(scenes,30);assert.equal(facilities,12);
});
test('atlas child rectangles are contained and malformed replacements fail closed',()=>{
 const asset=registry.resolveAsset('runtime2d.grey_hive.wall_tiles.v1');const copy=structuredClone(asset);const sub=facilitySlice(asset,[36,0,310,470],[.5,.9]);assert.deepEqual(sub.atlasFrame,[asset.atlasFrame[0]+36,asset.atlasFrame[1],310,470]);assert.deepEqual(asset,copy);
 for(const rect of [[-1,0,1,1],[0,0,99999,1],[0,0,1,NaN],[.5,0,1,1],[0,0,0,1]])assert.throws(()=>facilitySlice(asset,rect,[.5,1]),/SUBFRAME/);
 const bad={resolveAsset(id){const a=registry.resolveAsset(id);return id.endsWith('floor_tiles.v1')?{...a,atlasFrame:[0,0,16,16]}:a;}};assert.throws(()=>planScenePresentation(bad,entry),/SUBFRAME/);
});
test('top-only floor mesh covers exact world bounds at three viewport scales without sampling thick sides',()=>{
 for(const ppm of [48,60,96]){const c={...camera,pixelsPerMeter:ppm};const g=facilityFloorGeometry(plan.facilityFloor,c,4096,4096);assert.equal(g.positions.length,70*8);assert.equal(g.indices.length,70*6);
  let area=0;for(let i=0;i<g.positions.length;i+=8){const x=g.positions,y=x;area+=Math.abs((x[i+2]-x[i])*(y[i+5]-y[i+1])-(y[i+3]-y[i+1])*(x[i+4]-x[i]));}
  assert.ok(Math.abs(area-20*14*ppm*ppm)<.01);
  for(const corner of [[0,0],[20,0],[20,14],[0,14]]){const p=projectWorldPoint({xM:corner[0],yM:0,zM:corner[1]},c);assert.ok([...g.positions].some((v,i)=>i%2===0&&v===p.x&&g.positions[i+1]===p.y));}
  const [,ay]=plan.facilityFloor.asset.atlasFrame;for(let i=1;i<g.uvs.length;i+=2)assert.ok(g.uvs[i]*4096>=ay+178&&g.uvs[i]*4096<=ay+424,'no raised side face sampled');
 }
});
test('partial edge cells are clipped geometrically and preserve the actual exit gap',()=>{
 const p={...plan.facilityFloor,bounds:{x:0,z:0,width:19.5,depth:13.25}};const g=facilityFloorGeometry(p,camera,4096,4096);for(let i=0;i<g.positions.length;i+=2){const dx=(g.positions[i]-640)/48,dz=(g.positions[i+1]-360)/24;const x=2+(dx+dz)/2,z=7+(dz-dx)/2;assert.ok(x>=0&&x<=19.5&&z>=0&&z<=13.25);}
 assert.deepEqual(plan.facilityFloor.walls,entry.collision.map(c=>c.polygon));assert.ok(!plan.sprites.filter(s=>s.id.includes(':panel:')).some(s=>s.position.xM>=19.5&&s.position.zM>=5.5&&s.position.zM<=8.5));
});
test('actual Pixi mesh uses one top-face geometry, tracks the camera and releases without retiring shared texture',()=>{
 const renderer=Object.create(WorldRenderer.prototype);renderer.layerContainers=new Map(['L1_FLOOR','L2_BACK_PROPS','L3_ACTORS'].map(k=>[k,new Container()]));const atlas=new Texture({source:new TextureSource({width:4096,height:4096})});renderer.atlasTextures=new Map([[plan.facilityFloor.asset.atlasUrl,atlas]]);
 renderer.renderFacilityFloor(plan.facilityFloor,camera);const mesh=renderer.facilityFloorMesh;assert.deepEqual(mesh.geometry.positions,facilityFloorGeometry(plan.facilityFloor,camera,4096,4096).positions);assert.equal(mesh.parent,renderer.layerContainers.get('L1_FLOOR'));assert.ok(renderer.facilityFloorGraphic.zIndex<mesh.zIndex);
 renderer.renderFacilityFloor(plan.facilityFloor,{...camera,origin:{xM:3,yM:0,zM:7}});assert.equal(renderer.facilityFloorMesh,mesh);assert.equal(mesh.geometry.positions[0],facilityFloorGeometry(plan.facilityFloor,{...camera,origin:{xM:3,yM:0,zM:7}},4096,4096).positions[0]);renderer.clearFacilityFloor();assert.equal(renderer.facilityFloorMesh,null);assert.equal(atlas.destroyed,false);atlas.destroy(true);
});
test('physical actors and devices exchange front/back order by their feet, with stable ties',()=>{
 const prop={key:'scene:console',footY:200,layer:'L4_DYNAMIC_PROPS'},actor={key:'actor:player',footY:190,layer:'L3_ACTORS'};
 assert.ok(worldDepthRanks([prop,actor]).get(actor.key)<worldDepthRanks([prop,actor]).get(prop.key));actor.footY=210;assert.ok(worldDepthRanks([prop,actor]).get(actor.key)>worldDepthRanks([prop,actor]).get(prop.key));actor.footY=200;assert.deepEqual([...worldDepthRanks([prop,actor])],[...worldDepthRanks([actor,prop])]);assert.throws(()=>worldDepthRanks([{...actor,footY:NaN}]),/DEPTH/);
});
test('debug sidebar is opt-in and normal pause still contains return/save controls',async()=>{
 for(const q of ['','?developer=0','?developer=true','?developer=1&developer=0'])assert.equal(developerPresentationEnabled(q),false);assert.equal(developerPresentationEnabled('?developer=1'),true);
 const s=await readFile(new URL('../src/main.ts',import.meta.url),'utf8');assert.match(s,/<div class="journey-panel" id="developer-journey-panel" hidden>/);const pause=s.slice(s.indexOf('id="pause-overlay"'),s.indexOf('id="death-overlay"'));for(const id of ['save-new-slot','overwrite-slot','hud-resume','enhancement-status'])assert.ok(pause.includes(`id="${id}"`));assert.equal(s.match(/id="back-to-hub"/g).length,1);assert.ok(s.indexOf('id="back-to-hub"')<s.indexOf('id="pause-overlay"'),'return remains reachable while loading or running');
});
test('actual Pixi actor and device share a sortable parent while floor and atmospheric foreground remain separated',()=>{
 const r=Object.create(WorldRenderer.prototype);r.layerContainers=new Map(['L1_FLOOR','L2_BACK_PROPS','L3_ACTORS','L4_DYNAMIC_PROPS','L5_FRONT_PROPS'].map(k=>{const c=new Container();c.sortableChildren=true;return[k,c];}));r.sprites=new Map();r.sceneResources=new SceneResourceTracker();r.propCutoutMasks=new Map();r.frameTextures=new Map();r.atlasTextures=new Map();
 const actor=registry.resolveAsset('runtime2d.actor.cenyao.base.v1'),prop=registry.resolveAsset('runtime2d.prop.medical_station.v1'),fog=registry.resolveAsset('runtime2d.world.mistharbor.fog_bank.v1');for(const a of [actor,prop,fog])r.atlasTextures.set(a.atlasUrl,new Texture({source:new TextureSource({width:8192,height:8192})}));
 const a=r.ensureSprite('actor:player',actor,'L3_ACTORS',undefined,.1),p=r.ensureSprite('scene:device',prop,'L4_DYNAMIC_PROPS',undefined,.1),f=r.ensureSprite('scene:fog',fog,'L5_FRONT_PROPS',undefined,.1);assert.equal(a.sprite.parent,p.sprite.parent);assert.equal(f.sprite.parent,r.layerContainers.get('L5_FRONT_PROPS'));
 for(const y of [190,210]){const ranks=worldDepthRanks([{key:'actor:player',footY:y,layer:'L3_ACTORS'},{key:'scene:device',footY:200,layer:'L4_DYNAMIC_PROPS'}]);a.sprite.zIndex=ranks.get('actor:player');p.sprite.zIndex=ranks.get('scene:device');a.sprite.parent.sortChildren();assert.equal(a.sprite.parent.children.indexOf(a.sprite)<a.sprite.parent.children.indexOf(p.sprite),y<200);}
 r.removeSprite('scene:device',p);assert.equal(a.sprite.parent.children.includes(p.sprite),false);for(const t of r.frameTextures.values())t.destroy();for(const t of r.atlasTextures.values())t.destroy(true);
});
test('multi-state props select one authoritative state and single supply crate, without predicting unlocked as open',async()=>{
 const {projectPropPresentation}=await import('../dist/renderer/PropPresentationModel.js');const snapshot={protocolVersion:3,progression:{worlds:[{worldId:'grey_hive',completedEvents:[]}]}};const gate=registry.resolveAsset('runtime2d.prop.gate_a.closed_open.v1');const door={doorId:'gh_gate_a',open:false,locked:false};const closed=projectPropPresentation(gate,snapshot,[door]);const open=projectPropPresentation(gate,snapshot,[{...door,open:true}]);assert.notDeepEqual(closed.asset.atlasFrame,gate.atlasFrame);assert.notDeepEqual(closed.asset.atlasFrame,open.asset.atlasFrame);assert.equal(closed.heightM,2.6);
 const power=registry.resolveAsset('runtime2d.prop.power_console.off_on.v1');const off=projectPropPresentation(power,snapshot,[]);snapshot.progression.worlds[0].completedEvents=['hive_power'];const on=projectPropPresentation(power,snapshot,[]);assert.notDeepEqual(on.asset.atlasFrame,off.asset.atlasFrame);assert.notDeepEqual(on.asset.atlasFrame,power.atlasFrame);
 const gateB=registry.resolveAsset('runtime2d.prop.gate_b.v1');assert.equal(projectPropPresentation(gateB,snapshot,[{doorId:'gh_gate_b',open:true,locked:false}]),null);assert.ok(projectPropPresentation(gateB,snapshot,[{doorId:'gh_gate_b',open:false,locked:false}]));
 const crate=registry.resolveAsset('runtime2d.prop.supply_crates.v1'),single=projectPropPresentation(crate,snapshot,[]);assert.ok(single.cutout.length>=3);assert.notDeepEqual(single.asset.atlasFrame,crate.atlasFrame);assert.ok(single.cutout.every(([x,y])=>x>=0&&y>=0&&x<=540&&y<=550));
 for(const id of ['runtime2d.prop.beacon.folded.v1','runtime2d.prop.beacon.deployed.v1'])assert.deepEqual(projectPropPresentation(registry.resolveAsset(id),snapshot,[]).asset,registry.resolveAsset(id));
});
