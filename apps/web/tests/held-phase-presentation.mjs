import test from 'node:test';
import assert from 'node:assert/strict';
import { SessionLoop } from '../dist/game/SessionLoop.js';
import { ClockworksEnemyModel } from '../dist/renderer/ClockworksEnemyModel.js';
const pos={xM:5,yM:0,zM:7};
function snapshot(epoch=7,sceneId='cw_pressure_hall',entry=false){
 const s={kind:'full',protocolVersion:3,schemaVersion:'scene-v3/1',worldId:'clockworks',sceneId,worldEpoch:epoch,
  checkpointId:null,serverTick:100,authorityRevision:100,ackSeq:100,
  player:{entityId:'player',transform:{positionM:{xM:0,yM:0,zM:0},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},
   currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},
  actors:[{entityId:'cw_drone_01',entityType:'enemy.clockworks.pressure_drone',actorKind:'enemy',active:true,transform:{positionM:pos,yawRad:0}}],
  doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},
  progression:{schemaVersion:1,currentWorldId:'clockworks',eventSeq:0,worlds:[]}};
 if(entry)s.entryToken={generation:epoch,worldId:s.worldId,sceneId,worldEpoch:epoch};return s;
}
function cue(id,epoch=7){return {protocolVersion:2,eventId:id,worldEpoch:epoch,serverTick:100,kind:'EnemyAttackTelegraph',
 actorId:'cw_drone_01',attackKind:'pressure_shot',durationMs:1000/60,rangeM:8,radiusM:.16,intensity:1,directionRad:1.2,positionM:pos};}
function deferred(){let resolve;const promise=new Promise(yes=>{resolve=yes;});return {promise,resolve};}
async function flush(){for(let i=0;i<80;i++)await Promise.resolve();}
class Surface{hidden=false;listeners=new Map();addEventListener(k,f){const s=this.listeners.get(k)??new Set();s.add(f);this.listeners.set(k,s);}removeEventListener(k,f){this.listeners.get(k)?.delete(f);}fire(k){for(const f of this.listeners.get(k)??[])f({});}}
function harness(options={}){
 let state=snapshot(),held=true,id=1,committed=null,valid=true;const events=[cue(id)],calls=[],heard=[],draws=[];
 const model=new ClockworksEnemyModel(),documentTarget=new Surface(),windowTarget=new Surface();documentTarget.hidden=options.hidden??false;
 const intervals=new Map(),frames=new Map();let sequence=0;
 const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:f=>{const n=++sequence;intervals.set(n,f);return n;},clearInterval:n=>intervals.delete(n),
  requestAnimationFrame:f=>{const n=++sequence;frames.set(n,f);return n;},cancelAnimationFrame:n=>frames.delete(n)};
 const client={events:async(epoch,cursor)=>{calls.push(['events',held,cursor]);await options.events?.();return events.filter(e=>e.worldEpoch===epoch&&e.eventId>cursor);},
  sceneReady:async(token,remainPaused)=>{calls.push(['ready',remainPaused]);if(!remainPaused)assert.ok(draws.at(-1)?.frames.length,'owner cannot release before cue-bearing draw');
   held=remainPaused;state=snapshot(token.worldEpoch,token.sceneId);return state;},
  pause:async context=>{calls.push(['pause',context]);await options.pause?.();held=true;events.push(cue(++id,state.worldEpoch));return options.pauseReceipt?.()??state;},
  resume:async context=>{calls.push(['resume',context]);assert.equal(held,true);assert.ok(draws.at(-1)?.held);assert.equal(draws.at(-1)?.frames.length,1);
   assert.equal(draws.at(-1).frames[0].remainingMs,1000/60,'one-tick phase was not consumed during held drawing');
   held=false;events.push(cue(++id,state.worldEpoch));return state;}};
 const renderer={render:async value=>{const projected=model.project(value,true);calls.push(['draw',held,projected.length]);await options.render?.({value,frames:projected,held});
   draws.push({held,frames:projected});committed=value;},isReadyFor:value=>valid&&committed?.worldEpoch===value.worldEpoch&&committed?.sceneId===value.sceneId,
  cancel(){valid=false;},destroy(){}};
 const loop=new SessionLoop(client,renderer,{scheduler,documentTarget,windowTarget,
  presentationTimeoutMs:options.presentationTimeoutMs??5000,
  onEventsWithSnapshot:(rows,s)=>{const ids=new Set(model.accept(rows,s));return rows.filter(e=>ids.has(e.eventId));},
  onEvents:rows=>heard.push(...rows),onClearTransientPresentation:()=>model.reset()});
 return {loop,model,client,events,calls,heard,draws,intervals,frames,documentTarget,windowTarget,initial:snapshot(7,'cw_pressure_hall',true),lose:()=>{valid=false;},setState:s=>{state=s;}};
}

test('Continue admits one-tick phase after base setup and draws it before sceneReady release; loading sound stays silent',async()=>{
 const h=harness();await h.loop.start(h.initial);
 assert.deepEqual(h.calls.slice(0,4).map(c=>c[0]),['draw','events','draw','ready']);
 assert.equal(h.draws[0].frames.length,0);assert.equal(h.draws[1].frames.length,1);assert.equal(h.draws[1].held,true);
 assert.equal(h.heard.length,0);assert.equal(h.loop.pausePresentationState,'running');await h.loop.stop('unload');
});

test('formal resume refreshes unchanged-counter paused receipt and validates one-tick cue before owner release',async()=>{
 const h=harness();await h.loop.start(h.initial);await h.loop.pause();const start=h.calls.length;await h.loop.resume();
 assert.deepEqual(h.calls.slice(start).map(c=>c[0]),['pause','events','draw','resume','events']);
 const resume=h.calls.findLastIndex(c=>c[0]==='resume');assert.equal(h.calls[resume-1][0],'draw');assert.equal(h.calls[resume-1][1],true);
 assert.equal(h.heard.length,1,'only fresh real-resume ID sounds');assert.equal(h.heard[0].eventId,4);
 assert.equal(h.loop.snapshots.view().authorityRevision,100,'visual preparation needs no gameplay revision bump');
 assert.equal(h.loop.pausePresentationState,'running');await h.loop.stop('unload');
});

test('background paused polling leaves cursor intact until an explicit held visual refresh',async()=>{
 const h=harness();await h.loop.start(h.initial);await h.loop.pause();const cursor=h.loop.snapshots.eventCursor();
 await h.loop.pollEvents(h.loop.snapshots.view());assert.equal(h.loop.snapshots.eventCursor(),cursor);assert.equal(h.heard.length,0);
 await h.loop.resume();assert.equal(h.heard.length,1);await h.loop.stop('unload');
});

test('manual pause or hidden state during held cue draw cannot dispatch resume or stale sound',async()=>{
 for(const mode of ['manual','hidden']){
  const draw=deferred();let block=false;const h=harness({render:()=>block?draw.promise:undefined});await h.loop.start(h.initial);await h.loop.pause();block=true;
  const resumed=h.loop.resume();await flush();assert.equal(h.loop.keyDown('j'),null);assert.equal(h.intervals.size,0);
  if(mode==='manual')void h.loop.pause();else{h.documentTarget.hidden=true;h.documentTarget.fire('visibilitychange');}
  draw.resolve();await resumed;await flush();assert.equal(h.calls.some(c=>c[0]==='resume'),false);assert.equal(h.heard.length,0);assert.notEqual(h.loop.pausePresentationState,'running');await h.loop.stop('unload');
 }
});

test('hidden Continue stays held; returning visibility performs a fresh cue-bearing preparation',async()=>{
 const h=harness({hidden:true});await h.loop.start(h.initial);assert.deepEqual(h.calls.find(c=>c[0]==='ready'),['ready',true]);assert.equal(h.heard.length,0);
 h.documentTarget.hidden=false;h.documentTarget.fire('visibilitychange');await flush();assert.ok(h.calls.some(c=>c[0]==='resume'));assert.equal(h.heard.length,1);await h.loop.stop('unload');
});

test('context loss, wrong held receipt, or future held cue fails closed before resume',async()=>{
 for(const mode of ['context','receipt','future']){
  let h;h=harness({pauseReceipt:()=>mode==='receipt'?snapshot(8):snapshot(),events:()=>{if(mode==='context'&&h?.loop.pausePresentationState==='resuming')h.lose();}});
  await h.loop.start(h.initial);
  // Keep the deliberate invalid receipt scoped to preparation, not the initial pause.
  if(mode==='receipt'){h.client.pause=async()=>snapshot();await h.loop.pause();h.client.pause=async()=>snapshot(8);}
  else await h.loop.pause();
  if(mode==='future')h.events.push({...cue(99),serverTick:101});
  await h.loop.resume();assert.equal(h.calls.some(c=>c[0]==='resume'),false);assert.equal(h.loop.pausePresentationState,'error');assert.equal(h.intervals.size,0);await h.loop.stop('unload');
 }
});

test('stop during held phase fetch invalidates pending admission and cannot release owner',async()=>{
 const fetched=deferred();let block=false;const h=harness({events:()=>block?fetched.promise:undefined});await h.loop.start(h.initial);await h.loop.pause();block=true;
 const resumed=h.loop.resume();await flush();const stopped=h.loop.stop('unload');fetched.resolve();await Promise.all([resumed,stopped]);
 assert.equal(h.calls.some(c=>c[0]==='resume'),false);assert.equal(h.heard.length,0);assert.equal(h.model.project(snapshot(),true).length,0);
});

test('new scene entry supersedes old held resume draw without releasing the old context',async()=>{
 const draw=deferred();let block=false;const h=harness({render:({value})=>block&&value.worldEpoch===7?draw.promise:undefined});
 await h.loop.start(h.initial);await h.loop.pause();block=true;const resumed=h.loop.resume();await flush();
 const next=snapshot(8,'cw_boiler_chamber',true);h.setState(snapshot(8,'cw_boiler_chamber'));h.events.push(cue(9,8));
 const entry=h.loop.acceptAuthoritativeSnapshot(next);draw.resolve();await Promise.all([resumed,entry]);
 assert.equal(h.calls.some(c=>c[0]==='resume'),false);assert.equal(h.loop.snapshots.view().worldEpoch,8);
 assert.equal(h.loop.pausePresentationState,'running');assert.equal(h.heard.length,0);await h.loop.stop('unload');
});

test('proof lost after preparation resolves is rechecked immediately before resume dispatch',async()=>{
 const h=harness();await h.loop.start(h.initial);await h.loop.pause();
 const prepare=h.loop.preparePausedPresentation.bind(h.loop);
 h.loop.preparePausedPresentation=async(...args)=>{const ready=await prepare(...args);h.lose();return ready;};
 await h.loop.resume();assert.equal(h.calls.some(c=>c[0]==='resume'),false);
 assert.equal(h.loop.pausePresentationState,'error');assert.equal(h.heard.length,0);await h.loop.stop('unload');
});

test('repeated resume during held cue draw preserves the admitted warning and releases once',async()=>{
 const draw=deferred();let block=false;const h=harness({render:()=>block?draw.promise:undefined});
 await h.loop.start(h.initial);await h.loop.pause();block=true;const first=h.loop.resume();await flush();
 assert.equal(h.model.project(snapshot(),true).length,1);const second=h.loop.resume();
 assert.equal(h.model.project(snapshot(),true).length,1,'duplicate intent must not erase the prepared cue');
 draw.resolve();await Promise.all([first,second]);assert.equal(h.calls.filter(c=>c[0]==='resume').length,1);
 assert.equal(h.heard.length,1);await h.loop.stop('unload');
});

test('focus resume during entry cue draw preserves the held warning before ready',async()=>{
 const draw=deferred();const h=harness({render:({frames})=>frames.length?draw.promise:undefined});
 const entry=h.loop.start(h.initial);await flush();assert.equal(h.model.project(snapshot(),true).length,1);
 const focus=h.loop.resume('lifecycle');assert.equal(h.model.project(snapshot(),true).length,1);
 draw.resolve();await Promise.all([entry,focus]);assert.equal(h.loop.pausePresentationState,'running');
 assert.equal(h.calls.filter(c=>c[0]==='ready').length,1);assert.equal(h.heard.length,0);await h.loop.stop('unload');
});

test('held entry event read times out without ready; late response cannot admit cues or advance cursor',async()=>{
 const late=deferred();const h=harness({events:()=>late.promise,presentationTimeoutMs:10});
 await assert.rejects(h.loop.start(h.initial),/E_PRESENTATION_EVENTS_TIMEOUT/);
 assert.equal(h.calls.some(c=>c[0]==='ready'||c[0]==='resume'),false);assert.equal(h.intervals.size,0);
 const cursor=h.loop.snapshots.eventCursor();late.resolve();await flush();
 assert.equal(h.loop.snapshots.eventCursor(),cursor);assert.equal(h.model.project(snapshot(),true).length,0);assert.equal(h.heard.length,0);
});

test('held resume event read times out before release and a late response stays inert',async()=>{
 const late=deferred();let block=false;const h=harness({events:()=>block?late.promise:undefined,presentationTimeoutMs:10});
 await h.loop.start(h.initial);await h.loop.pause();block=true;const cursor=h.loop.snapshots.eventCursor();
 await h.loop.resume();assert.equal(h.loop.pausePresentationState,'error');assert.equal(h.calls.some(c=>c[0]==='resume'),false);
 assert.equal(h.intervals.size,0);late.resolve();await flush();
 assert.equal(h.loop.snapshots.eventCursor(),cursor);assert.equal(h.model.project(snapshot(),true).length,0);assert.equal(h.heard.length,0);
 await h.loop.stop('unload');
});
