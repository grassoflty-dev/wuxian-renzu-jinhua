import test from "node:test";
import assert from "node:assert/strict";
import {readFile} from "node:fs/promises";
import {parseMovingSurfaces,projectMovingSurfaces} from "../dist/renderer/MovingSurfaceModel.js";
import {planScenePresentation} from "../dist/renderer/ScenePresentation.js";
const bounds={x:0,z:0,width:24,depth:16};
const region=()=>({id:"belt",tag:"conveyor",polygon:[[6,6],[18,6],[18,10],[6,10]],surfaceVelocityMps:[-1.2,0]});
const snapshot=(serverTick=0)=>({protocolVersion:3,worldId:"clockworks",sceneId:"cw_conveyor_bridge",worldEpoch:3,serverTick});
const plan=()=>({worldId:"clockworks",sceneId:"cw_conveyor_bridge",movingSurfaces:parseMovingSurfaces([region()],bounds)});

test("compiled conveyor surface carries exact direction and geometry into verified scene plan",async()=>{
 const scene=JSON.parse(await readFile(new URL("../../../content/scenes/compiled/cw_conveyor_bridge.json",import.meta.url),"utf8"));
 const surfaces=parseMovingSurfaces(scene.terrainRegions,scene.boundsM);
 assert.equal(surfaces.length,1);assert.deepEqual(surfaces[0].velocityMps,[-1.2,0]);
 assert.deepEqual(surfaces[0].polygon,region().polygon);
 const renderPlan=planScenePresentation({resolveAsset(){throw new Error("no asset lookup expected");}},
  {schemaVersion:1,worldId:"clockworks",sceneId:"cw_conveyor_bridge",boundsM:bounds,presentation:{},terrainRegions:[region()]});
 assert.deepEqual(renderPlan.movingSurfaces,plan().movingSurfaces);
});

test("static old scenes keep no moving surface and water is not mislabeled conveyor",()=>{
 assert.deepEqual(parseMovingSurfaces(undefined,bounds),[]);
 assert.deepEqual(parseMovingSurfaces([{id:"water",tag:"terrain.water_shallow"}],bounds),[]);
 assert.deepEqual(projectMovingSurfaces({...plan(),movingSurfaces:[]},snapshot(),false),[]);
});

test("malformed region identity velocity and polygons fail closed",()=>{
 for(const alter of [r=>{r.surfaceVelocityMps=[0,0]},r=>{r.surfaceVelocityMps=[NaN,0]},r=>{r.surfaceVelocityMps=[4.1,0]},
  r=>{delete r.surfaceVelocityMps},r=>{r.surfaceVelocityMps=[1]},r=>{r.polygon=[[0,0],[1,0],[2,0]]},
  r=>{r.polygon[0]=[25,0]},r=>{r.polygon[0]=[null,0]},r=>{r.polygon[0]=[1]},r=>{r.id=""},r=>{r.cheatSpeed=10}]) {
  const r=region();alter(r);assert.throws(()=>parseMovingSurfaces([r],bounds),/E_SCENE_SURFACE_/);
 }
 assert.throws(()=>parseMovingSurfaces([region(),region()],bounds),/E_SCENE_SURFACE_ID/);
});

test("arrows are temporary readonly projections moving with server ticks and freezing with pause",()=>{
 const source=plan(),before=structuredClone(source);
 const first=projectMovingSurfaces(source,snapshot(0),false);
 assert.ok(first[0].temporary_visual);assert.ok(first[0].arrows.length>0);
 assert.ok(first[0].arrows.every(a=>a.dx===-1 && a.dz===0));
 assert.notDeepEqual(projectMovingSurfaces(source,snapshot(10),false),first);
 assert.deepEqual(projectMovingSurfaces(source,snapshot(10),false),projectMovingSurfaces(source,snapshot(10),false));
 assert.deepEqual(source,before);
});

test("reduced motion uses static direction and world scene or invalid epoch cannot leak old overlay",()=>{
 assert.deepEqual(projectMovingSurfaces(plan(),snapshot(0),true),projectMovingSurfaces(plan(),snapshot(999),true));
 for(const value of [{...snapshot(),worldId:"grey_hive"},{...snapshot(),sceneId:"cw_boiler_chamber"},
  {...snapshot(),worldEpoch:0},{...snapshot(),serverTick:NaN},{...snapshot(),protocolVersion:2}]) {
  assert.deepEqual(projectMovingSurfaces(plan(),value,false),[]);
 }
 assert.deepEqual(projectMovingSurfaces(null,snapshot(),false),[]);
});

test("large valid surface rendering cost is bounded independently of its area",()=>{
 const r=region();r.polygon=[[0,0],[100000,0],[100000,100000],[0,100000]];
 const movingSurfaces=parseMovingSurfaces([r],{x:0,z:0,width:100000,depth:100000});
 const frames=projectMovingSurfaces({...plan(),movingSurfaces},snapshot(),false);
 assert.equal(frames[0].arrows.length,128);
});
