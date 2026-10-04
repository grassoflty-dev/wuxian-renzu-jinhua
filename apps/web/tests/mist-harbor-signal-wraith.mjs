import test from 'node:test';
import assert from 'node:assert/strict';
import {Container,Sprite,Texture,Polygon} from 'pixi.js';
import {ClockworksEnemyModel,CLOCKWORKS_ENEMY_PLACEHOLDERS,enemyPlaceholderStyle,enemyWarningGroundPoints} from '../dist/renderer/ClockworksEnemyModel.js';
import {WorldRenderer} from '../dist/renderer/WorldRenderer.js';
import {projectWorldPoint} from '../dist/renderer/CameraModel.js';
import {actorAssetId} from '../dist/renderer/RendererResourcePlan.js';
import {audioCueForPresentationEvent} from '../dist/audio/AudioEventMap.js';
import {validatedSignalPerception,signalWraithInterference} from '../dist/protocol/PresentationEventValidation.js';
import {SnapshotClient} from '../dist/game/SnapshotClient.js';
import {SessionLoop} from '../dist/game/SessionLoop.js';
import {projectWorldUi} from '../dist/renderer/WorldUiModel.js';
import {deriveMistHarborSonarMap} from '../dist/ui/components/MistHarborSonarMap.js';
import {deriveMistHarborStatus} from '../dist/ui/components/MistHarborStatusPanel.js';
const type='enemy.mist_harbor.signal_wraith',id='mh_tidal_warehouse_signal_wraith_01',point={xM:19,yM:0,zM:5};
function actor(precise=true){return {entityId:id,entityType:type,actorKind:'enemy',active:true,transform:{positionM:precise?point:{...point,xM:19.325},yawRad:0},signalPerception:{positionsM:precise?[point]:[point,{...point,xM:19.65}],precise,uncertaintyRadiusM:precise?0:.65}};}
function snapshot(overrides={}){return {kind:'full',protocolVersion:3,schemaVersion:'scene-v3/1',worldId:'mist_harbor',sceneId:'mh_tidal_warehouse',worldEpoch:7,serverTick:100,authorityRevision:100,ackSeq:100,checkpointId:null,
 player:{entityId:'player',transform:{positionM:{xM:20,yM:0,zM:5},yawRad:0},velocityMps:{xM:0,yM:0,zM:0},currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:'idle'},actors:[actor()],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},progression:{schemaVersion:1,currentWorldId:'mist_harbor',eventSeq:0,worlds:[]},...overrides};}
function event(eventId=1,overrides={}){return {protocolVersion:2,eventId,worldEpoch:7,serverTick:100,kind:'EnemyAttackTelegraph',actorId:id,attackKind:'signal_shot',durationMs:1000,rangeM:8,radiusM:.18,intensity:1,positionM:point,directionRad:Math.PI/2,...overrides};}
const camera={width:1200,height:800,origin:{xM:0,yM:0,zM:0},pixelsPerMeter:48};
function renderer(){const r=Object.create(WorldRenderer.prototype);r.layerContainers=new Map([['L3_ACTORS',new Container()],['L7_VFX',new Container()]]);r.clockworksEnemyModel=new ClockworksEnemyModel();r.clockworksEnemyGraphics=new Map();r.clockworksEnemyGlows=new Map();r.clockworksEnemyBodies=new Map();r.sprites=new Map();r.sentinelTelegraphGraphics=new Map();r.sentinelGlowGraphics=new Map();r.sentinelTelegraphModel={reset(){}};r.transientPresentationRevision=0;r.machineryGraphics=new Map();return r;}

test('One Wraith has identical stable visual alternatives, one precise mapped body and no Warden alias override',()=>{
 const r=renderer();r.sprites.set(`actor:${id}`,{sprite:new Sprite(Texture.WHITE)});const ambiguous=actor(false);assert.equal(actorAssetId(ambiguous),'runtime2d.enemy.mistharbor.signal_wraith.v2');assert.equal(CLOCKWORKS_ENEMY_PLACEHOLDERS[type].temporary_visual,true);assert.equal(enemyPlaceholderStyle('runtime2d.enemy.mistharbor.signal_wraith.v2'),null);
 r.renderClockworksEnemyBodies([ambiguous],camera,new Map([[id,{xM:999,yM:0,zM:999}]]));assert.equal(r.sprites.get(`actor:${id}`).sprite.visible,false);assert.equal(r.swarmMemberBodies.size,2);const rows=[...r.swarmMemberBodies.values()];assert.equal(rows[0].sprite.alpha,rows[1].sprite.alpha);assert.equal(rows[0].sprite.width,rows[1].sprite.width);for(let i=0;i<2;i++){const p=projectWorldPoint(ambiguous.signalPerception.positionsM[i],camera);assert.equal(rows[i].sprite.x,p.x);assert.equal(rows[i].shadow.y,p.footY);assert.ok(rows[i].sprite.label.includes('temporary_visual=true'));}
 const before=rows.map(b=>[b.sprite.x,b.sprite.y]);r.renderClockworksEnemyBodies([ambiguous],camera);assert.deepEqual([...r.swarmMemberBodies.values()].map(b=>[b.sprite.x,b.sprite.y]),before);r.renderClockworksEnemyBodies([actor()],camera);assert.equal(r.swarmMemberBodies.size,1);assert.equal(rows[1].sprite.destroyed,true);r.clearClockworksEnemyBodies();assert.equal(rows[0].sprite.destroyed,true);
});

test('Malformed projection, source, Warden alias and hidden fields never poison the event cursor',()=>{
 const cases=[];for(const mutate of [a=>delete a.signalPerception,a=>a.signalPerception.positionsM=[],a=>a.signalPerception.actualIndex=0,a=>a.signalPerception.positionsM=[{...point,xM:NaN}],a=>a.signalPerception.uncertaintyRadiusM=1,a=>a.entityId='mh_tidal_warehouse_signal_wraith_02',a=>a.entityType='runtime2d.enemy.mistharbor.signal_wraith.v2']){const a=actor();mutate(a);cases.push(snapshot({actors:[a]}));}
 cases.push(snapshot({actors:[actor(false)]}),snapshot({sceneId:'mh_warden_arena'}),snapshot({actors:[actor(),actor()]}));
 for(const state of cases){const client=new SnapshotClient();client.accept(state);assert.deepEqual(client.acceptEvents([event(999)]),[]);assert.equal(client.eventCursor(),0);const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([event(999)],state),[]);assert.deepEqual(model.accept([event(2)],snapshot()),[2]);}
 assert.equal(validatedSignalPerception({...actor(false),transform:{positionM:point,yawRad:0}}),null);
});

test('Precise ordinary cast warning and projectile use committed public geometry at every camera scale',()=>{
 for(const scale of [24,48,72]){const model=new ClockworksEnemyModel();assert.deepEqual(model.accept([event()],snapshot()),[1]);const frame=model.project(snapshot(),true)[0],points=enemyWarningGroundPoints(frame);assert.equal(points.length,4);assert.deepEqual(points.map(p=>p.xM),[19,27,27,19]);const r=renderer(),cam={...camera,pixelsPerMeter:scale};r.renderClockworksEnemyCues([frame],cam);const g=r.clockworksEnemyGraphics.get(1),polygon=new Polygon(g.context.instructions[0].data.path.instructions[0].data[0]);const center=projectWorldPoint({xM:23,yM:0,zM:5},cam);assert.ok(polygon.contains((center.x-g.x)/g.scale.x,(center.footY-g.y)/g.scale.y));assert.ok(r.clockworksEnemyGlows.has(1));assert.equal(audioCueForPresentationEvent(event()).id,'audio.mh.wraith.cast.warning');
  const moved=snapshot();moved.player.transform.positionM={xM:19,yM:0,zM:12};assert.deepEqual(model.project(moved,true)[0],frame);const motion=event(2,{kind:'SignalShotMotion',positionM:{...point,xM:19.5},durationMs:1500});assert.deepEqual(model.accept([motion],snapshot()),[2]);const flying=model.project(snapshot(),true)[0];assert.deepEqual(flying.positionM,motion.positionM);r.renderClockworksEnemyCues([flying],cam);assert.equal(audioCueForPresentationEvent(motion),null);r.clearTransientPresentation();}
});

test('Impact terminates motion even when recovery immediately restores ambiguity; blink has its own bounded cue',()=>{
 const model=new ClockworksEnemyModel(),state=snapshot({actors:[actor(false)]});const impact=event(1,{kind:'EnemyAttackImpact',durationMs:170,positionM:{...point,xM:24}});assert.deepEqual(model.accept([impact,event(2,{kind:'SignalShotMotion'})],state),[1]);assert.equal(model.project(state,true)[0].kind,'EnemyAttackImpact');assert.equal(audioCueForPresentationEvent(impact).id,'audio.mh.wraith.cast.impact');
 const blink=event(3,{kind:'SignalBlink',attackKind:undefined,rangeM:1.25,radiusM:.3,durationMs:255});assert.deepEqual(model.accept([blink],state),[3]);const frame=model.project(state,true).find(f=>f.kind==='SignalBlink'),r=renderer();r.renderClockworksEnemyCues([frame],camera);assert.ok(r.clockworksEnemyGraphics.get(3).context.instructions.length>0);assert.equal(audioCueForPresentationEvent(blink).id,'audio.mh.wraith.blink');assert.deepEqual(new ClockworksEnemyModel().accept([event(999,{...blink,rangeM:2.1})],state),[]);
});

test('Rust-projected interference reduces known map precision without granting capability or leaking hit points',()=>{
 const state=snapshot({actors:[actor(false)]});state.capabilities.items=[{capabilityId:'information.local_map_i',granted:true,selected:true}];state.capabilities.exploredMap={worldId:'mist_harbor',playerPositionM:state.player.transform.positionM,rooms:[],connections:[],objectives:[{objectiveId:'known',positionM:{xM:22,yM:0,zM:9}}]};const before=structuredClone(state);assert.equal(signalWraithInterference(state).entityId,id);const markers=projectWorldUi(state,camera);assert.equal(markers.filter(m=>m.kind==='signal_region').length,1);assert.equal(markers.filter(m=>m.kind==='known_objective').length,0);assert.equal(markers.filter(m=>m.kind==='enemy_vital').length,0);assert.ok(deriveMistHarborStatus(state).signalInterference.includes('灵影'));assert.deepEqual(state,before);
 state.capabilities.exploredMap.rooms=[{roomId:'warehouse',outlineM:[{xM:0,yM:0,zM:0},{xM:24,yM:0,zM:0},{xM:24,yM:0,zM:16},{xM:0,yM:0,zM:16}]}];assert.equal(deriveMistHarborSonarMap(state).markers[0].kind,'signal_region');
 state.actors[0].signalPerception.positionsM=[];assert.deepEqual(projectWorldUi(state,camera),[]);assert.deepEqual(deriveMistHarborSonarMap(state).markers,[]);assert.ok(deriveMistHarborStatus(state).signalInterference.includes('未报告'));
 state.actors=[actor()];assert.equal(signalWraithInterference(state),null);assert.equal(projectWorldUi(state,camera).filter(m=>m.kind==='known_objective').length,1);assert.equal(state.capabilities.items.length,1);
});

test('Future precise casts wait for their matching snapshot and replacements discard old queued cues',()=>{
 const client=new SnapshotClient();client.accept(snapshot({actors:[actor(false)]}));assert.deepEqual(client.acceptEvents([event(999,{rangeM:NaN}),event(2,{serverTick:101})]),[]);assert.equal(client.eventCursor(),0);client.accept(snapshot({serverTick:101,authorityRevision:101,ackSeq:101}));assert.deepEqual(client.acceptEvents([]).map(e=>e.eventId),[2]);const next=new SnapshotClient();next.accept(snapshot());next.acceptEvents([event(2,{serverTick:101})]);next.accept(snapshot({worldEpoch:8,serverTick:0,authorityRevision:0,ackSeq:0}));assert.deepEqual(next.acceptEvents([]),[]);assert.equal(next.eventCursor(),0);
});

test('Restored one-tick casts and zero-elapsed flight draw precise bodies plus actual Pixi cues before ready and ordered resume',async()=>{
 for(const kind of ['EnemyAttackTelegraph','SignalShotMotion']){let held=true,counter=1,current=snapshot(),committed=null;const model=new ClockworksEnemyModel(),r=renderer();r.sprites.set(`actor:${id}`,{sprite:new Sprite(Texture.WHITE)});const row=event(1,{kind,durationMs:1000/60}),events=[row],calls=[];const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:()=>1,clearInterval(){},requestAnimationFrame:()=>1,cancelAnimationFrame(){}};
  const client={events:async(epoch,cursor)=>events.filter(e=>e.eventId>cursor),sceneReady:async()=>{assert.deepEqual(calls.at(-1),['draw',true,1,1]);held=false;return current;},pause:async()=>{held=true;events.push({...row,eventId:++counter});return current;},resume:async()=>{assert.deepEqual(calls.at(-1),['draw',true,1,1]);held=false;return current;}};
  const render={render:async s=>{const frames=model.project(s,true);r.renderClockworksEnemyBodies(s.actors,camera);r.renderClockworksEnemyCues(frames,camera);calls.push(['draw',held,r.swarmMemberBodies.size,frames.length]);if(frames.length){assert.equal(frames[0].remainingMs,1000/60);assert.deepEqual(frames[0].actorPositionM,point);}committed=s;},isReadyFor:s=>committed?.worldEpoch===s.worldEpoch&&committed?.sceneId===s.sceneId,destroy(){r.clearClockworksEnemyBodies();}};
  const loop=new SessionLoop(client,render,{scheduler,onEventsWithSnapshot:(rows,s)=>{const ids=new Set(model.accept(rows,s));return rows.filter(e=>ids.has(e.eventId));},onClearTransientPresentation:()=>{model.reset();r.clearTransientPresentation();}});await loop.start({...current,entryToken:{generation:7,worldId:'mist_harbor',sceneId:'mh_tidal_warehouse',worldEpoch:7}});assert.equal(loop.pausePresentationState,'running');await loop.pause();await loop.resume();assert.equal(loop.pausePresentationState,'running');await loop.stop('unload');}
});
