import test from 'node:test';
import assert from 'node:assert/strict';
import { SessionLoop } from '../dist/game/SessionLoop.js';
import { TauriClient } from '../dist/bridge/tauri-client.js';
import { assertSnapshotV3 } from '../dist/protocol/types.js';

function snapshot(generation = 1, epoch = generation, scene = `scene_${generation}`) {
  return { kind:'full', protocolVersion:3, schemaVersion:'freeze-v02-interfaces/1.2', worldId:'grey_hive',sceneId:scene,checkpointId:null,
    worldEpoch:epoch,serverTick:1,authorityRevision:1,ackSeq:0,
    entryToken:{generation, worldId:'grey_hive',sceneId:scene,worldEpoch:epoch},
    player:{entityId:'player',transform:{positionM:{xM:0,yM:0,zM:0},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},
    actors:[],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'grey_hive',eventSeq:0,worlds:[]}};
}
function released(value, revision = 2) { const {entryToken,...result}=value; return {...result,authorityRevision:revision}; }
function deferred() { let resolve,reject; const promise=new Promise((yes,no)=>{resolve=yes;reject=no;}); return {promise,resolve,reject}; }
async function flush() { for(let i=0;i<40;i++) await Promise.resolve(); }
class Surface { hidden=false; listeners=new Map(); addEventListener(key,callback){const set=this.listeners.get(key)??new Set();set.add(callback);this.listeners.set(key,set);} removeEventListener(key,callback){this.listeners.get(key)?.delete(callback);} fire(key){for(const callback of this.listeners.get(key)??[])callback({});} }
function harness(options={}) {
  const initial=options.initial??snapshot();const calls=[];const statuses=[]; const intervals=new Map(),frames=new Map(); let next=0;let committed=null;let valid=true; let destroyed=false;
  const windowTarget=new Surface(), documentTarget=new Surface(); documentTarget.hidden=options.hidden??false;
  const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:callback=>{const id=++next;intervals.set(id,callback);return id;},clearInterval:id=>intervals.delete(id),requestAnimationFrame:callback=>{const id=++next;frames.set(id,callback);return id;},cancelAnimationFrame:id=>frames.delete(id)};
  const renderer={render:async value=>{calls.push(['render',value.sceneId]);await options.render?.(value);committed=value;},isReadyFor:value=>valid&&committed?.worldEpoch===value.worldEpoch&&committed?.sceneId===value.sceneId,cancel:()=>{valid=false;options.cancel?.();},destroy:async()=>{destroyed=true;}};
  const client={events:async()=>{calls.push(['events']);return await options.events?.()??[];},submitInput:async value=>{calls.push(['input',value]);return released(initial);},submitAction:async value=>{calls.push(['action',value.kind]);return released(initial);},sceneReady:async(token,remainPaused)=>{calls.push(['ready',token.generation,remainPaused]);return await options.ready?.(token,remainPaused)??released({...initial,worldEpoch:token.worldEpoch,sceneId:token.sceneId});},pause:async()=>{calls.push(['pause']);return await options.pause?.()??released(initial,3);},resume:async()=>{calls.push(['resume']);return await options.resume?.()??released(initial,4);}};
  const loop=new SessionLoop(client,renderer,{scheduler,windowTarget,documentTarget,onPauseState:status=>statuses.push(status),...options.loopOptions});
  return {loop,calls,statuses,intervals,frames,windowTarget,documentTarget,initial,lose:()=>{valid=false;},get destroyed(){return destroyed;}};
}

test('entry locks controls and both schedulers until committed frame and exact ready receipt',async()=>{
  const draw=deferred(), ready=deferred();const h=harness({render:()=>draw.promise,ready:()=>ready.promise});
  const started=h.loop.start(h.initial);await flush();
  assert.equal(h.loop.pausePresentationState,'loading');assert.equal(h.loop.keyDown('w'),null);assert.equal(h.loop.keyDown('e'),null);assert.equal(h.loop.keyDown('f'),null);h.loop.keyUp('e');
  assert.equal(h.intervals.size,0);assert.equal(h.frames.size,0);assert.equal(h.calls.some(x=>x[0]==='ready'),false);
  draw.resolve();await flush();assert.deepEqual(h.calls.find(x=>x[0]==='ready'),['ready',1,false]);assert.equal(h.intervals.size,0);
  ready.resolve(released(h.initial));await started;assert.equal(h.intervals.size,1);assert.equal(h.frames.size,1);assert.equal(h.loop.pausePresentationState,'running');await h.loop.stop();
});

test('initial hidden document and pause during event polling preserve authority pause',async()=>{
  for(const hidden of [true,false]) {const events=deferred();const h=harness({hidden,events:()=>events.promise});const started=h.loop.start(h.initial);await flush();
    if(!hidden)void h.loop.pause();events.resolve([]);await started;
    assert.deepEqual(h.calls.find(x=>x[0]==='ready'),['ready',1,true]);assert.equal(h.intervals.size,0);assert.equal(h.loop.pausePresentationState,'paused');await h.loop.stop();}
});

test('blur during ready IPC cannot reopen controls; formal pause receipt gates state',async()=>{
  const ready=deferred(),pause=deferred();const h=harness({ready:()=>ready.promise,pause:()=>pause.promise});const started=h.loop.start(h.initial);await flush();
  h.windowTarget.fire('blur');ready.resolve(released(h.initial));await flush();
  assert.equal(h.calls.at(-1)[0],'pause');assert.equal(h.intervals.size,0);assert.equal(h.loop.keyDown('j'),null);assert.equal(h.statuses.includes('running'),false);
  pause.resolve(released(h.initial,3));await started;assert.equal(h.loop.pausePresentationState,'paused');await h.loop.stop();
});

test('hidden state is re-read even without a visibility event while ready IPC is pending',async()=>{
  const ready=deferred();const h=harness({ready:()=>ready.promise});const started=h.loop.start(h.initial);await flush();h.documentTarget.hidden=true;ready.resolve(released(h.initial));await started;
  assert.equal(h.calls.at(-1)[0],'pause');assert.equal(h.intervals.size,0);await h.loop.stop();
});

test('manual resume in flight after hidden entry reconciles latest pause before reopening',async()=>{
  const ready=deferred(),resume=deferred();let pauseCount=0;const h=harness({hidden:true,ready:()=>ready.promise,resume:()=>resume.promise,pause:()=>released(snapshot(),++pauseCount===1?3:5)});const started=h.loop.start(h.initial);await flush();
  h.documentTarget.hidden=false;h.documentTarget.fire('visibilitychange');ready.resolve(released(h.initial));await flush();assert.equal(h.calls.at(-1)[0],'resume');
  void h.loop.pause();resume.resolve(released(h.initial,4));await started;assert.equal(h.calls.at(-1)[0],'pause');assert.equal(h.intervals.size,0);assert.equal(h.loop.pausePresentationState,'paused');await h.loop.stop();
});

test('renderer failure, missing proof and context loss during poll never acknowledge entry',async()=>{
  const drawFailure=harness({render:()=>Promise.reject(new Error('asset failure'))});await assert.rejects(drawFailure.loop.start(drawFailure.initial),/asset failure/);assert.equal(drawFailure.calls.some(x=>x[0]==='ready'),false);
  const events=deferred();const h=harness({events:()=>events.promise});const started=h.loop.start(h.initial);await flush();h.lose();events.resolve([]);
  await assert.rejects(started,/FRAME_NOT_READY/);assert.equal(h.calls.some(x=>x[0]==='ready'),false);assert.equal(h.intervals.size,0);
});

test('context loss during ready acknowledgement requests authority pause and fails closed',async()=>{
  const ready=deferred();const h=harness({ready:()=>ready.promise});const started=h.loop.start(h.initial);await flush();h.lose();ready.resolve(released(h.initial));
  await assert.rejects(started,/FRAME_NOT_READY/);assert.equal(h.calls.at(-1)[0],'pause');assert.equal(h.intervals.size,0);assert.equal(h.statuses.includes('running'),false);
});

test('stop before first frame never acknowledges; stop during ready false compensates pause',async()=>{
  const draw=deferred();const h=harness({render:()=>draw.promise,cancel:()=>draw.resolve()});const started=h.loop.start(h.initial);await flush();await h.loop.stop();await started;
  assert.equal(h.calls.some(x=>x[0]==='ready'),false);assert.equal(h.destroyed,true);
  const ready=deferred();const second=harness({ready:()=>ready.promise});const secondStart=second.loop.start(second.initial);await flush();const stop=second.loop.stop();ready.resolve(released(second.initial));await Promise.all([secondStart,stop]);assert.equal(second.calls.at(-1)[0],'pause');assert.equal(second.intervals.size,0);
});

test('rapid navigation rejects stale completion and only latest token can ready',async()=>{
  const draw=deferred();const h=harness({render:value=>value.worldEpoch===1?draw.promise:Promise.resolve()});const started=h.loop.start(h.initial);await flush();const next=snapshot(2);const accepted=h.loop.acceptAuthoritativeSnapshot(next);draw.resolve();await Promise.all([started,accepted]);
  assert.deepEqual(h.calls.filter(x=>x[0]==='ready'),[['ready',2,false]]);assert.equal(h.loop.snapshots.view().sceneId,'scene_2');assert.equal(h.intervals.size,1);await h.loop.stop();
});

test('transition receipt owns readiness without waiting on its own external interaction; queued GuardEnd is discarded',async()=>{
  const interaction=deferred();let h;h=harness({loopOptions:{onInteract:async()=>{await interaction.promise;await h.loop.acceptAuthoritativeSnapshot(snapshot(2));}},ready:token=>released(snapshot(token.generation))});
  await h.loop.start(h.initial);h.loop.keyDown('e');await flush();h.loop.keyDown('f');const paused=h.loop.pause();interaction.resolve();await paused;await flush();
  assert.ok(h.calls.some(x=>x[0]==='ready'&&x[1]===2));assert.deepEqual(h.calls.filter(x=>x[0]==='action').map(x=>x[1]),['guardStart']);assert.equal(h.intervals.size,0);await h.loop.stop();
});

test('entry token validator rejects malformed or mismatched identities, and ready receipt binds all fields',async()=>{
  const source=snapshot();for(const token of [null,{...source.entryToken,generation:0},{...source.entryToken,generation:1.5},{...source.entryToken,worldEpoch:5},{...source.entryToken,sceneId:'wrong'}])assert.throws(()=>assertSnapshotV3({...source,entryToken:token}),/ENTRY_TOKEN/);
  const receipt={commandId:'scene-ready:1',applied:true,alreadyApplied:false,errorCode:null,worldEpoch:1,serverTick:1,authorityRevision:2,snapshot:released(source)};
  const calls=[];const client=new TauriClient(async(command,args)=>{calls.push([command,args]);return receipt;});assert.deepEqual(await client.sceneReady(source.entryToken,true),receipt.snapshot);assert.deepEqual(calls,[['formal_scene_ready',{token:source.entryToken,remainPaused:true}]]);
  for(const bad of [{...receipt,commandId:'scene-ready:2'},{...receipt,authorityRevision:9},{...receipt,applied:false,errorCode:'stale'},{...receipt,snapshot:source,authorityRevision:1},{...receipt,snapshot:{...receipt.snapshot,sceneId:'wrong'}},{...receipt,alreadyApplied:true}])await assert.rejects(new TauriClient(async()=>bad).sceneReady(source.entryToken,false),/SCENE_READY/);
});

test('old in-flight input and action errors or snapshots cannot tear down a newer entry',async()=>{
  for(const command of ['input','action']) for(const outcome of ['reject','resolve']) {
    const pending=deferred();let errors=0;const h=harness({loopOptions:{onError:()=>errors++}});
    // The same dispatch path is exercised without altering renderer readiness.
    h.loop.client[command==='input'?'submitInput':'submitAction']=()=>pending.promise;
    await h.loop.start(h.initial);
    if(command==='input')for(const callback of h.intervals.values())callback();else h.loop.keyDown('j');
    await flush();await h.loop.acceptAuthoritativeSnapshot(snapshot(2));
    if(outcome==='reject')pending.reject(new Error(command==='input'?'E_INPUT_STALE_EPOCH':'E_RUNTIME_PAUSED'));
    else pending.resolve(released(h.initial));
    await flush();assert.equal(errors,0);assert.equal(h.loop.snapshots.view().worldEpoch,2);assert.equal(h.loop.pausePresentationState,'running');assert.equal(h.intervals.size,1);assert.equal(h.destroyed,false);await h.loop.stop();
  }
});

test('current-scene input errors remain observable rather than being silently suppressed',async()=>{
  let errors=0;const h=harness({loopOptions:{onError:()=>errors++}});h.loop.client.submitInput=()=>Promise.reject(new Error('bridge unavailable'));
  await h.loop.start(h.initial);for(const callback of h.intervals.values())callback();await flush();assert.equal(errors,1);assert.equal(h.destroyed,true);assert.equal(h.intervals.size,0);
});


test('stop during a compensating pause cannot publish a late entry error',async()=>{
  const ready=deferred(),pause=deferred();const h=harness({ready:()=>ready.promise,pause:()=>pause.promise});const started=h.loop.start(h.initial);await flush();h.lose();ready.resolve(released(h.initial));await flush();assert.equal(h.calls.at(-1)[0],'pause');
  const stopped=h.loop.stop();pause.resolve(released(h.initial,3));await Promise.all([started,stopped]);assert.equal(h.statuses.includes('error'),false);assert.equal(h.intervals.size,0);assert.equal(h.destroyed,true);
});

test('prior-scene fatal lookup cannot mark a newer entry dead',async()=>{
  const lookup=deferred();const h=harness();h.loop.client.submitInput=()=>Promise.reject(new Error('E_RUNTIME_DEAD'));h.loop.client.snapshot=()=>lookup.promise;
  await h.loop.start(h.initial);for(const callback of h.intervals.values())callback();await flush();await h.loop.acceptAuthoritativeSnapshot(snapshot(2));const fatal=released(h.initial);fatal.player.currentHp=0;lookup.resolve(fatal);await flush();assert.equal(h.loop.isDead,false);assert.equal(h.loop.snapshots.view().worldEpoch,2);assert.equal(h.loop.pausePresentationState,'running');await h.loop.stop();
});
