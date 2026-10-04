import test from 'node:test';
import assert from 'node:assert/strict';
import { SessionLoop } from '../dist/game/SessionLoop.js';
import { supportGeometryKey } from '../dist/protocol/SupportProjection.js';
import { loaded,snapshot } from './support/vertical-support-fixture.mjs';
function deferred(){let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});return{promise,resolve,reject};}
async function flush(){for(let i=0;i<30;i++)await Promise.resolve();}
class Surface extends EventTarget {hidden=false;fire(name){this.dispatchEvent(new Event(name));}}
function released(value,revision=value.authorityRevision+1){const result=structuredClone(value);delete result.entryToken;result.authorityRevision=revision;result.supportScene.authorityRevision=revision;return result;}
async function harness(options={}){
 const {sceneSha}=await loaded();const initial=snapshot(sceneSha);initial.entryToken={generation:1,worldId:initial.worldId,sceneId:initial.sceneId,worldEpoch:initial.worldEpoch};const calls=[],statuses=[],intervals=new Map(),frames=new Map();let serial=0,committed=null,available=true;const windowTarget=new Surface(),documentTarget=new Surface();documentTarget.hidden=options.hidden??false;
 const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:cb=>{intervals.set(++serial,cb);return serial;},clearInterval:id=>intervals.delete(id),requestAnimationFrame:cb=>{frames.set(++serial,cb);return serial;},cancelAnimationFrame:id=>frames.delete(id)};
 const renderer={render:async value=>{calls.push(['draw',value.supportScene.serverTimeMs]);await options.render?.(value);committed=supportGeometryKey(value.supportScene);},isReadyFor:value=>available&&!!value.supportScene&&committed===supportGeometryKey(value.supportScene),cancel:()=>{available=false;},destroy:async()=>{}};
 const client={events:async()=>[],submitInput:async()=>released(initial),submitAction:async()=>released(initial),sceneReady:async(token,paused)=>{calls.push(['ready',paused]);return await options.ready?.(initial,paused)??released(initial);},pause:async()=>{calls.push(['pause']);return await options.pause?.(initial)??released(initial,100);},resume:async()=>{calls.push(['resume']);return await options.resume?.(initial)??released(initial,101);}};
 const loop=new SessionLoop(client,renderer,{scheduler,windowTarget,documentTarget,onPauseState:s=>statuses.push(s)});return{loop,initial,calls,statuses,intervals,frames,documentTarget,windowTarget,sceneSha};
}
test('deferred ready(false) cannot publish newer support geometry from the previous drawn proof',async()=>{
 const pending=deferred();const h=await harness({ready:()=>pending.promise});const start=h.loop.start(h.initial);await flush();assert.ok(h.calls.some(c=>c[0]==='ready'&&c[1]===false));const newer=released(snapshot(h.sceneSha,45,767),100);pending.resolve(newer);await assert.rejects(start,/FRAME_NOT_READY/);assert.ok(h.calls.some(c=>c[0]==='pause'));assert.equal(h.statuses.includes('running'),false);assert.equal(h.intervals.size,0);assert.equal(h.frames.size,0);
});
test('authority-only Ready metadata changes retain the exact visible geometry proof',async()=>{
 const h=await harness({ready:initial=>released(initial,80)});await h.loop.start(h.initial);assert.equal(h.loop.pausePresentationState,'running');assert.equal(h.intervals.size,1);await h.loop.stop();
});
test('ready(true), late focus and newer held pose must draw that pose before resume',async()=>{
 const ready=deferred(),draw=deferred();let changed;const h=await harness({hidden:true,ready:()=>ready.promise,pause:()=>changed,render:value=>value.supportScene.serverTimeMs===767?draw.promise:Promise.resolve(),resume:()=>released(changed,103)});changed=released(snapshot(h.sceneSha,45,767),102);const start=h.loop.start(h.initial);await flush();assert.ok(h.calls.some(c=>c[0]==='ready'&&c[1]===true));h.documentTarget.hidden=false;h.documentTarget.fire('visibilitychange');ready.resolve(released(h.initial,101));await flush();assert.ok(h.calls.some(c=>c[0]==='draw'&&c[1]===767));assert.equal(h.calls.some(c=>c[0]==='resume'),false);assert.equal(h.intervals.size,0);draw.resolve();await start;assert.equal(h.calls.filter(c=>c[0]==='resume').length,1);assert.equal(h.loop.pausePresentationState,'running');await h.loop.stop();
});
test('a new pause intent during the newer held draw keeps correct proof for a later focus',async()=>{
 const ready=deferred(),draw=deferred();let changed;const h=await harness({hidden:true,ready:()=>ready.promise,pause:()=>changed,render:value=>value.supportScene.serverTimeMs===767?draw.promise:Promise.resolve(),resume:()=>released(changed,103)});changed=released(snapshot(h.sceneSha,45,767),102);const start=h.loop.start(h.initial);await flush();h.documentTarget.hidden=false;h.documentTarget.fire('visibilitychange');ready.resolve(released(h.initial,101));await flush();h.documentTarget.hidden=true;h.documentTarget.fire('visibilitychange');draw.resolve();await start;assert.equal(h.loop.pausePresentationState,'paused');assert.equal(h.calls.some(c=>c[0]==='resume'),false);h.documentTarget.hidden=false;await h.loop.resume('lifecycle');assert.equal(h.loop.pausePresentationState,'running');await h.loop.stop();
});
