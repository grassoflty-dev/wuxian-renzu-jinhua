import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { ActionVfxModel } from '../dist/renderer/ActionVfxModel.js';
const source = (overrides={}) => ({worldId:'grey_hive',sceneId:'gh_gate_b',worldEpoch:3,serverTick:10,
 actionState:'pulse',actionPresentation:{requestId:1,phase:'windup',elapsedMs:17,durationMs:590,rangeM:2.6,lineHalfWidthM:0},
 position:{xM:1,yM:0,zM:2},facingX:1,facingZ:0,nowMs:1000,...overrides});

test('five owner-authored action identities retain distinct phases and authoritative dimensions',()=>{
 const m=new ActionVfxModel();
 for(const [index,actionState] of ['primaryAttack','pulse','guard','pierce','dash'].entries()){
  const s=source({actionState,serverTick:10+index,nowMs:1000+index*20});const v=m.update(s);
  assert.equal(v.kind,actionState);assert.equal(v.phase,'windup');assert.equal(v.rangeM,2.6);
 }
});

test('guard shield validity uses committed phase and normalizes owner facing',()=>{
 const v=new ActionVfxModel().update(source({actionState:'guard',facingX:.6,facingZ:.8,
 actionPresentation:{requestId:1,phase:'recovery',elapsedMs:117,durationMs:880,rangeM:0,lineHalfWidthM:0}}));
 assert.equal(v.phase,'recovery');assert.ok(Math.abs(Math.hypot(v.directionX,v.directionZ)-1)<1e-9);
});

test('late snapshots expire locally without progressing windup or reviving from redraws',()=>{
 const m=new ActionVfxModel();assert.equal(m.update(source()).phase,'windup');
 assert.equal(m.update(source({nowMs:1249})).phase,'windup');assert.equal(m.update(source({nowMs:1250})),null);
 assert.equal(m.update(source({nowMs:1800})),null);
 assert.equal(m.update(source({serverTick:11,nowMs:1800,actionPresentation:{...source().actionPresentation,elapsedMs:34}})).phase,'windup');
});

test('inactive, missing phase, impossible duration and unrecognized authority states authorize no VFX',()=>{
 const m=new ActionVfxModel();
 for(const actionState of ['idle','walk','guardStart','unknown'])assert.equal(m.update(source({actionState})),null);
 assert.equal(m.update(source({actionPresentation:undefined})),null);
 assert.equal(m.update(source({actionPresentation:{...source().actionPresentation,elapsedMs:590}})),null);
});

test('epoch/lifecycle reset requires a fresh authoritative presentation receipt',()=>{
 const m=new ActionVfxModel();m.update(source());assert.equal(m.update(source({nowMs:1500})),null);
 assert.equal(m.update(source({worldEpoch:4,nowMs:1500})).phase,'windup');
 m.reset();assert.equal(m.update(source({actionState:'idle',actionPresentation:undefined,nowMs:1500})),null);
});

test('renderer destroys phase graphics on transient cleanup and uses terrain/body layers',async()=>{
 const renderer=await readFile(fileURLToPath(new URL('../src/renderer/WorldRenderer.ts',import.meta.url)),'utf8');
 assert.match(renderer,/this\.actionVfxModel\.reset\(\)/);assert.match(renderer,/for \(const graphic of this\.actionVfx\.values\(\)\) graphic\.destroy\(\)/);
 assert.match(renderer,/vfx\.kind === "guard" \? "L3_ACTORS" : "L1_FLOOR"/);
});
