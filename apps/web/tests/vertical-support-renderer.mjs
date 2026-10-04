import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { Assets,Container,Texture,TextureSource } from 'pixi.js';
import { AssetRegistry } from '../dist/assets/AssetRegistry.js';
import { WorldRenderer } from '../dist/renderer/WorldRenderer.js';
import { loaded,definition,snapshot } from './support/vertical-support-fixture.mjs';
const root=new URL('../../../',import.meta.url);const registry=new AssetRegistry();
await registry.registerManifest(new Uint8Array(await readFile(new URL('governance/assets/RUNTIME_ASSET_MANIFEST.json',root))),new Uint8Array(await readFile(new URL('governance/assets/AI_ASSET_RELEASE_MANIFEST.json',root))),async path=>new Uint8Array(await readFile(new URL(path,root))));
const source=JSON.parse(await readFile(new URL('content/scenes/compiled/rs_core_room.json',root),'utf8'));source.movingSupports=definition().movingSupports;source.standingDecks=definition().standingDecks;
function view(sha,tick=44,time=750){const value=snapshot(sha,tick,time);value.worldId=value.supportScene.worldId='return_station';value.sceneId=value.supportScene.sceneId='rs_core_room';value.progression.currentWorldId='return_station';return value;}
async function fixture(run){
 const original=Object.fromEntries(['window','document','ResizeObserver','requestAnimationFrame','cancelAnimationFrame'].map(k=>[k,globalThis[k]]));const unload=Assets.unload;const frames=new Map();let next=0;const target=new EventTarget();target.matchMedia=()=>({matches:true});globalThis.window=target;globalThis.document={documentElement:{dataset:{motion:'reduced'}}};globalThis.requestAnimationFrame=cb=>{frames.set(++next,cb);return next;};globalThis.cancelAnimationFrame=id=>frames.delete(id);globalThis.ResizeObserver=class{observe(){}disconnect(){}};Assets.unload=async()=>{};
 const canvas=new EventTarget();canvas.parentElement={clientWidth:882,clientHeight:552};let draws=0,fail=false;const app={stage:new Container(),screen:{width:0,height:0},renderer:{background:{},resize(width,height){app.screen={width,height};}},async init(options){this.renderer.resize(options.width,options.height);},render(){if(fail)throw Error('fixture draw failure');draws++;},destroy(){}};
 const renderer=new WorldRenderer(registry);renderer.app=app;renderer.preloadAtlasPages=async pages=>{for(const page of pages)if(!renderer.atlasTextures.has(page.atlasUrl))renderer.atlasTextures.set(page.atlasUrl,new Texture({source:new TextureSource({width:8192,height:8192})}));};
 try{const {scene,sceneSha}=await loaded(source);const initial=view(sceneSha);await renderer.init(canvas);renderer.expectSceneIdentity(initial);renderer.setSceneDefinition(scene);await run({renderer,initial,sceneSha,draws:()=>draws,fail:()=>{fail=true;}});}finally{if(renderer.ready)await renderer.destroy();Assets.unload=unload;for(const[k,v]of Object.entries(original)){if(v===undefined)delete globalThis[k];else globalThis[k]=v;}}
}
test('actual Pixi path uses one platform/rider sample even when caller supplies independent player overrides',async()=>fixture(async f=>{
 assert.equal(f.renderer.isReadyFor(f.initial),false);await f.renderer.render(f.initial);assert.equal(f.draws(),1);assert.equal(f.renderer.isReadyFor(f.initial),true);const sprite=f.renderer.sprites.get('actor:player').sprite;const position=[sprite.position.x,sprite.position.y];assert.equal(f.renderer.verticalSupportGraphic.context.instructions.filter(i=>i.action==='fill').length,2);
 await f.renderer.render(f.initial,{xM:99,yM:0,zM:99},new Map([['player',{xM:99,yM:0,zM:99}]]));assert.deepEqual([sprite.position.x,sprite.position.y],position);assert.deepEqual(f.renderer.lastFrame.playerPosition,f.initial.player.transform.positionM);
 const newer=view(f.sceneSha,45,767);assert.equal(f.renderer.isReadyFor(newer),false);await f.renderer.render(newer);assert.equal(f.renderer.isReadyFor(newer),true);assert.equal(f.renderer.isReadyFor(f.initial),false);
}));
test('a draw failure revokes prior support readiness and cannot leave a usable old proof',async()=>fixture(async f=>{
 await f.renderer.render(f.initial);assert.equal(f.renderer.isReadyFor(f.initial),true);f.fail();await assert.rejects(f.renderer.render(view(f.sceneSha,45,767)));assert.equal(f.renderer.isReadyFor(f.initial),false);assert.equal(f.renderer.committedFrame,null);
}));
test('missing or stale source-bound support records fail before actual draw and clear prior readiness',async()=>fixture(async f=>{
 await f.renderer.render(f.initial);const draws=f.draws();const bad=structuredClone(f.initial);bad.supportScene.sceneSourceSha256='0'.repeat(64);await assert.rejects(f.renderer.render(bad),/SOURCE_MISMATCH/);assert.equal(f.draws(),draws);assert.equal(f.renderer.isReadyFor(f.initial),false);
}));
