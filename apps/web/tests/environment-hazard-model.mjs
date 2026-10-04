import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { projectEnvironmentHazards } from "../dist/renderer/EnvironmentHazardModel.js";
import { WorldRenderer } from "../dist/renderer/WorldRenderer.js";
import { Container } from "pixi.js";
const scene = JSON.parse(await readFile(new URL("../../../content/scenes/compiled/cw_boiler_chamber.json", import.meta.url), "utf8"));
function snapshot(phase="warning", overrides={}) {
  return { kind:"full", protocolVersion:3, worldId:scene.worldId,sceneId:scene.sceneId,
    worldEpoch:2,serverTick:60, hazards:scene.hazards.map(h=>({entityId:h.id,kind:h.kind,
      active:phase==="active",phaseActive:true,polygonM:h.polygon,
      environment:{tag:h.environment.tag,phase,remainingMs:600,exposureBps:h.environment.tag==="heat"?4500:0}})), ...overrides };
}
test("five authored hazard regions project readable Rust phases without actor or completion assumptions", async()=>{
  for(const name of ["gh_deep_decon","cw_boiler_chamber","cw_furnace_heart"]){
    const content=JSON.parse(await readFile(new URL(`../../../content/scenes/compiled/${name}.json`,import.meta.url),"utf8"));
    const input=snapshot("warning",{worldId:content.worldId,sceneId:content.sceneId,hazards:content.hazards.map(h=>({entityId:h.id,kind:h.kind,active:false,phaseActive:true,polygonM:h.polygon,
      environment:{tag:h.environment.tag,phase:"warning",remainingMs:h.environment.warningMs,exposureBps:0}}))});
    const frames=projectEnvironmentHazards(input,true);assert.equal(frames.length,content.hazards.length);
    assert.ok(frames.every(f=>f.label.includes("预警")&&f.label.includes("秒")));
    assert.deepEqual(frames.map(f=>f.polygonM),content.hazards.map(h=>h.polygon));
  }
  for(const phase of ["warning","active","recovery","suppressed"]){
    const input=snapshot(phase);const before=structuredClone(input);const frames=projectEnvironmentHazards(input,true);
    assert.equal(frames.length,2);assert.equal(frames[0].phase,phase);assert.match(frames[0].label,/热量 45%/);
    assert.deepEqual(input,before);frames[0].polygonM[0][0]=999;assert.notEqual(input.hazards[0].polygonM[0][0],999);
  }
});
test("paused projection is deterministic and reduced motion removes visual pulse without changing state",()=>{
  const input=snapshot();assert.deepEqual(projectEnvironmentHazards(input,false),projectEnvironmentHazards(input,false));
  const later=snapshot("warning",{serverTick:70});
  assert.deepEqual(projectEnvironmentHazards(input,true),projectEnvironmentHazards(later,true));
  const moving=projectEnvironmentHazards(input,false);const laterMoving=projectEnvironmentHazards(later,false);
  assert.notEqual(moving[0].alpha,laterMoving[0].alpha);assert.equal(moving[0].remainingMs,laterMoving[0].remainingMs);
});
test("malformed, duplicated and mismatched hazard projections fail closed",()=>{
  for(const patch of [{phase:"invented"},{phase:"active"},{tag:"toxin"},{remainingMs:NaN},{remainingMs:0},{remainingMs:60001},{exposureBps:10001},{exposureBps:-1}]){
    const input=snapshot();input.hazards[0].environment={...input.hazards[0].environment,...patch};
    assert.equal(projectEnvironmentHazards(input,true).length,1,JSON.stringify(patch));
  }
  for(const polygon of [[],[[0,0],[1,1],[2,2]],[[0,0],[1,Infinity],[2,0]],[[0,0],[5000,0],[1,1]]]){
    const input=snapshot();input.hazards[0].polygonM=polygon;assert.equal(projectEnvironmentHazards(input,true).length,1);
  }
  const duplicate=snapshot();duplicate.hazards.push(duplicate.hazards[0]);assert.deepEqual(projectEnvironmentHazards(duplicate,true),[]);
  for(const input of [snapshot("warning",{protocolVersion:2}),snapshot("warning",{serverTick:NaN}),snapshot("warning",{worldEpoch:0})])assert.deepEqual(projectEnvironmentHazards(input,true),[]);
});
test("renderer releases stale environment graphics and labels without touching shared textures",()=>{
  const renderer=Object.create(WorldRenderer.prototype);
  renderer.environmentHazards=new Map();renderer.layerContainers=new Map([["L7_VFX",new Container()]]);
  const camera={origin:{xM:0,yM:0,zM:0},width:1280,height:720,pixelsPerMeter:48};
  const frames=projectEnvironmentHazards(snapshot(),true);
  renderer.renderEnvironmentHazards(frames,camera);
  assert.equal(renderer.environmentHazards.size,2);
  const records=[...renderer.environmentHazards.values()];
  renderer.renderEnvironmentHazards([],camera);assert.equal(renderer.environmentHazards.size,0);
  assert.ok(records.every(r=>r.graphic.destroyed&&r.label.destroyed));
  renderer.renderEnvironmentHazards(frames,camera);renderer.clearEnvironmentHazards();
  assert.equal(renderer.environmentHazards.size,0);assert.equal(renderer.layerContainers.get("L7_VFX").children.length,0);
});
test("scene cleanup owns environment overlays; pause clears only transients and cannot restart hazard clocks",async()=>{
  const source=await readFile(new URL("../src/renderer/WorldRenderer.ts",import.meta.url),"utf8");
  assert.match(source,/clearSceneResources\(\): Promise<void> \{\s*this.clearEnvironmentHazards\(\)/);
  const start=source.indexOf("  clearTransientPresentation():");
  const transient=source.slice(start,source.indexOf("\n  }",start)+4);
  assert.doesNotMatch(transient,/clearEnvironmentHazards/);
  assert.match(source,/renderEnvironmentHazards\(projectEnvironmentHazards\(snapshot, reduceFogMotion\), camera\)/);
});
