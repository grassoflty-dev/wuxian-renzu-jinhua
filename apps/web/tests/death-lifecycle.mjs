import test from 'node:test';
import assert from 'node:assert/strict';
import { SessionLoop } from '../dist/game/SessionLoop.js';
import { SnapshotClient } from '../dist/game/SnapshotClient.js';

function snapshot(hp=100,tick=1,revision=tick,epoch=1,sceneId='gh_entry') {
  return {kind:'full',protocolVersion:3,schemaVersion:'freeze-v02-interfaces/1.2',worldId:'grey_hive',sceneId,checkpointId:null,worldEpoch:epoch,serverTick:tick,authorityRevision:revision,ackSeq:tick,
    player:{entityId:'player',transform:{positionM:{xM:0,yM:0,zM:0},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:hp,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:hp===0?'dead':'idle'},actors:[],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[]}};
}
function deferred(){let resolve,reject;const promise=new Promise((yes,no)=>{resolve=yes;reject=no;});return{promise,resolve,reject};}
async function flush(){for(let i=0;i<40;i++)await Promise.resolve();}
class Surface{listeners=new Map();hidden=false;addEventListener(name,cb){const list=this.listeners.get(name)??new Set();list.add(cb);this.listeners.set(name,list);}removeEventListener(name,cb){this.listeners.get(name)?.delete(cb);}fire(name){for(const cb of this.listeners.get(name)??[])cb({});}}
function harness(overrides={}){
 const calls=[],views=[],deaths=[],statuses=[],draws=[];const intervals=new Map(),frames=new Map();let next=0;const windowTarget=new Surface(),documentTarget=new Surface();
 const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:cb=>{const id=++next;intervals.set(id,cb);return id;},clearInterval:id=>intervals.delete(id),requestAnimationFrame:cb=>{const id=++next;frames.set(id,cb);return id;},cancelAnimationFrame:id=>frames.delete(id)};
 const client={events:async()=>[],submitInput:async()=>snapshot(),submitAction:async action=>{calls.push(action.kind);return snapshot();},pause:async()=>{calls.push('pause');return snapshot();},resume:async()=>{calls.push('resume');return snapshot();},...overrides.client};
 const loop=new SessionLoop(client,{render:async value=>{await overrides.render?.(value);draws.push(value);},isReadyFor:()=>true},{scheduler,windowTarget,documentTarget,onSnapshot:value=>views.push(value),onDeath:value=>deaths.push(value),onPauseState:value=>statuses.push(value)});
 return{loop,calls,views,deaths,statuses,draws,intervals,frames,windowTarget,documentTarget,tick:()=>{for(const cb of intervals.values())cb();}};
}

test('fatal input receipt locks actions, drops held guard and queued intents, and renders one terminal frame',async()=>{
 const input=deferred();const h=harness({client:{submitInput:()=>input.promise}});await h.loop.start(snapshot());h.loop.keyDown('e');await flush();h.loop.keyDown('w');h.tick();h.loop.keyUp('e');h.loop.keyDown('r');input.resolve(snapshot(0,2));await flush();
 assert.equal(h.loop.isDead,true);assert.equal(h.loop.pausePresentationState,'dead');assert.equal(h.loop.acceptsExternalResults,false);assert.equal(h.loop.input.isHeld('w'),false);assert.equal(h.intervals.size,0);assert.equal(h.frames.size,0);assert.deepEqual(h.calls,['guardStart']);assert.equal(h.deaths.length,1);assert.equal(h.draws.at(-1).player.currentHp,0);
 for(const key of ['w','j','e','q','r','f','Escape'])assert.equal(h.loop.keyDown(key),null);await h.loop.resume();await h.loop.pause();await h.loop.retryPauseState();h.windowTarget.fire('focus');h.windowTarget.fire('blur');h.documentTarget.fire('visibilitychange');await flush();
 await assert.rejects(h.loop.runWhilePaused(async()=>h.calls.push('save')),/SAVE_REQUIRES/);await assert.rejects(h.loop.runWithSession(async()=>h.calls.push('build')),/BUILD_SESSION_CHANGED/);assert.deepEqual(h.calls,['guardStart']);await h.loop.stop();assert.deepEqual(h.calls,['guardStart']);
});

test('fatal result wins over an already-dispatched priority pause',async()=>{
 const input=deferred();const h=harness({client:{submitInput:()=>input.promise}});await h.loop.start(snapshot());h.tick();const pause=h.loop.pause();input.resolve(snapshot(0,3));await pause;await flush();assert.equal(h.loop.pausePresentationState,'dead');assert.equal(h.calls.filter(x=>x==='pause').length,1);assert.equal(h.intervals.size,0);await h.loop.stop();
});

test('late resume receipt cannot revive fatal view or overwrite consumer state',async()=>{
 const resume=deferred();const h=harness({client:{pause:async()=>snapshot(100,2),resume:()=>resume.promise}});await h.loop.start(snapshot());await h.loop.pause();const resumed=h.loop.resume();await flush();await h.loop.acceptAuthoritativeSnapshot(snapshot(0,4));const viewCount=h.views.length;
 resume.resolve(snapshot(100,3));await resumed;assert.equal(h.views.length,viewCount);assert.equal(h.views.at(-1).player.currentHp,0);assert.equal(h.loop.pausePresentationState,'dead');assert.equal(h.intervals.size,0);await h.loop.stop();
});

test('fatal snapshot is terminal before any HUD callback; stale alive receipts are never presented',async()=>{
 const h=harness();await h.loop.start(snapshot());await h.loop.acceptAuthoritativeSnapshot(snapshot(0,5,5));const count=h.views.length;
 for(const alive of [snapshot(100,4,4),snapshot(100,5,5),snapshot(100,6,4),snapshot(100,4,6),snapshot(100,50,50,0)])await h.loop.acceptAuthoritativeSnapshot(alive);
 assert.equal(h.views.length,count);assert.equal(h.deaths.length,1);assert.equal(h.loop.snapshots.view().player.currentHp,0);
 for(const alive of [snapshot(100,6,6),snapshot(100,5,6),snapshot(100,6,5),snapshot(100,6,6,1,'gh_other'),snapshot(100,1,1,2)])await assert.rejects(h.loop.acceptAuthoritativeSnapshot(alive),/DEAD_REVIVAL|DEAD_SESSION_REPLACEMENT/);
 assert.equal(h.views.length,count);assert.equal(h.loop.snapshots.view().player.currentHp,0);assert.equal(h.loop.pausePresentationState,'dead');await h.loop.stop();
});

test('SnapshotClient rejects revival before mutation and a fresh client admits explicit Continue/New',()=>{
 const client=new SnapshotClient();assert.equal(client.accept(snapshot()),true);assert.equal(client.accept(snapshot(0,5)),true);const fatal=client.view();
 assert.equal(client.accept(snapshot(100,6,4)),false);assert.equal(client.view(),fatal);assert.throws(()=>client.accept(snapshot(100,6)),/DEAD_REVIVAL/);assert.equal(client.view(),fatal);assert.throws(()=>client.accept(snapshot(100,1,1,2)),/DEAD_SESSION_REPLACEMENT/);assert.equal(client.view(),fatal);
 const replacement=new SnapshotClient();assert.equal(replacement.accept(snapshot(100,1,1,2)),true);assert.equal(replacement.view().player.currentHp,100);
});

test('starting with fatal authority never sends readiness, schedules input or exposes save/resume',async()=>{
 const h=harness({client:{sceneReady:async()=>{h.calls.push('ready');return snapshot();}}});const initial=snapshot(0);initial.entryToken={generation:1,worldId:initial.worldId,sceneId:initial.sceneId,worldEpoch:1};await h.loop.start(initial);assert.equal(h.loop.isDead,true);assert.equal(h.calls.length,0);assert.equal(h.intervals.size,0);assert.equal(h.draws.at(-1).player.currentHp,0);assert.equal(h.deaths.length,1);await h.loop.stop();
});

test('fatal frame is committed after already-running living render finishes',async()=>{
 const draw=deferred();let count=0;const h=harness({render:async()=>{if(++count===2)await draw.promise;}});await h.loop.start(snapshot());const cb=h.frames.values().next().value;cb(17);await flush();const death=h.loop.acceptAuthoritativeSnapshot(snapshot(0,2));assert.equal(h.loop.isDead,true);assert.equal(h.intervals.size,0);draw.resolve();await death;assert.equal(h.draws.at(-1).player.currentHp,0);assert.equal(h.draws.length,3);await h.loop.stop();
});

test('late build result after fatal snapshot cannot be applied; stop still drains registered work',async()=>{
 const build=deferred();const h=harness();await h.loop.start(snapshot());const operation=h.loop.runWithSession(()=>build.promise);await flush();await h.loop.acceptAuthoritativeSnapshot(snapshot(0,3));const stopped=h.loop.stop();build.resolve(snapshot(100,2));await assert.rejects(operation,/BUILD_SESSION_CHANGED/);await stopped;assert.equal(h.views.at(-1).player.currentHp,0);assert.equal(h.calls.length,0);
});


test('later still-dead receipts cannot replace or re-render the committed fatal view',async()=>{
 const h=harness();await h.loop.start(snapshot());const fatal=snapshot(0,5);await h.loop.acceptAuthoritativeSnapshot(fatal);const views=h.views.length,draws=h.draws.length;
 const mutated=snapshot(0,6,9);mutated.player.transform.positionM.xM=99;await h.loop.acceptAuthoritativeSnapshot(mutated);
 assert.equal(h.loop.snapshots.view(),fatal);assert.equal(h.views.length,views);assert.equal(h.draws.length,draws);assert.equal(h.deaths.length,1);await h.loop.stop();
});

test('current input or action E_RUNTIME_DEAD fetches exact fatal authority and presents death',async()=>{
 for(const kind of ['input','action']){
  let reads=0;const h=harness({client:{[kind==='input'?'submitInput':'submitAction']:async()=>{throw new Error('E_RUNTIME_DEAD');},snapshot:async()=>{reads++;return snapshot(0,3);}}});await h.loop.start(snapshot());if(kind==='input')h.tick();else h.loop.keyDown('j');await flush();
  assert.equal(reads,1);assert.equal(h.loop.isDead,true);assert.equal(h.loop.pausePresentationState,'dead');assert.equal(h.deaths.length,1);assert.equal(h.intervals.size,0);await h.loop.stop();
 }
});

test('death lookup rejects living and wrong-context responses without fabricating a fatal state',async()=>{
 for(const returned of [snapshot(100,3),snapshot(0,3,3,2),snapshot(0,3,3,1,'other')]){
  const h=harness({client:{submitInput:async()=>{throw 'E_RUNTIME_DEAD';},snapshot:async()=>returned}});await h.loop.start(snapshot());h.tick();await flush();assert.equal(h.loop.isDead,false);assert.equal(h.deaths.length,0);assert.equal(h.loop.snapshots.view().player.currentHp,100);assert.equal(h.intervals.size,0);await h.loop.stop();
 }
});

test('stop while fatal lookup is pending ignores late fatal state and does not reopen UI',async()=>{
 const lookup=deferred();const h=harness({client:{submitInput:async()=>{throw new Error('E_RUNTIME_DEAD');},snapshot:()=>lookup.promise}});await h.loop.start(snapshot());h.tick();await flush();const stopped=h.loop.stop();lookup.resolve(snapshot(0,3));await stopped;assert.equal(h.deaths.length,0);assert.equal(h.loop.isDead,false);assert.equal(h.intervals.size,0);
});
