import test from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import { TauriClient } from "../dist/bridge/tauri-client.js";
import { BaizhiDialogue, baizhiInteractionPrompt, BAIZHI_RESULTS } from "../dist/game/BaizhiDialogue.js";
import { BaizhiDialoguePanel } from "../dist/ui/components/BaizhiDialoguePanel.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";
import { deriveHudState } from "../dist/ui/Hud.js";
import { assertSnapshotV3 } from "../dist/protocol/types.js";

const compiled = await readFile(new URL("../dist/main.js", import.meta.url), "utf8");
const start = compiled.indexOf("async function interactFromSnapshot("), end = compiled.indexOf("\nasync function begin(", start);
assert.ok(start >= 0 && end > start);
const handler = compiled.slice(start, end);
const flush = () => new Promise(resolve => setImmediate(resolve));
const deferred = () => { let resolve, reject; const promise = new Promise((a,b) => { resolve = a; reject = b; }); return {promise, resolve, reject}; };
const pos = (xM=8,yM=0,zM=9.3) => ({xM,yM,zM});
function snapshot(choice="unresolved", changes={}) {
  return { kind:"full", protocolVersion:3, schemaVersion:"freeze-v02-interfaces/1.2", worldId:"grey_hive", sceneId:"gh_bio_isolation",
    checkpointId:null, worldEpoch:8, serverTick:70, authorityRevision:9, ackSeq:0,
    player:{entityId:"player",transform:{positionM:pos(),yawRad:0},velocityMps:pos(0,0,0),currentHp:100,maxHp:100,currentEnergy:100,maxEnergy:100,
      facingX:0,facingZ:1,aimX:0,aimZ:1,actionState:"idle"},
    actors:[],doors:[],interactables:[],hazards:[],objectives:[],capabilities:{schemaVersion:1,items:[]},
    progression:{schemaVersion:1,currentWorldId:"grey_hive",eventSeq:3,worlds:[]},
    baizhi:{schemaVersion:1,choice,available:true,canInteract:true},
    npcs:[{entityId:"gh_bz_whitezhi_v1",entityType:"npc.baizhi",position:[8,0,10.5],yawRad:Math.PI,interactable:true}],...changes };
}
function harness({choice="unresolved", beginDelay, commitDelay, closeDelay, commitFailures=0, closeFailures=0, closeResumed=true}={}) {
  let current=snapshot(choice), ticket=null, revision=9, saved=0, lifecycle=0;
  const calls=[], changes=[], errors=[];
  function view(paused=true) { return {...current, authorityRevision:++revision,
    baizhi:{...current.baizhi,canInteract:!paused},npcs:[{...current.npcs[0],interactable:!paused}]}; }
  function result(commandId, paused=true, resumed=false) {current=view(paused); return {receipt:{commandId,applied:true,alreadyApplied:false,errorCode:null,
    worldEpoch:current.worldEpoch,serverTick:current.serverTick,authorityRevision:current.authorityRevision,snapshot:current},ticket,resumed};}
  const client = new TauriClient(async (command,args) => {
    calls.push({command,args});
    if(command==="formal_baizhi_begin") {
      assert.deepEqual(Object.keys(args.request.context).sort(),["sceneId","worldEpoch","worldId"], "Rust SessionContext denies unknown fields");
      lifecycle=args.request.commandSequence;
      ticket={...args.request.context,entityId:args.request.entityId,interactionId:args.request.interactionId,ownerId:args.request.ownerId,
        generation:args.request.generation,pauseCommandSequence:lifecycle};
      const response=result(`baizhi-begin:${ticket.ownerId}:${ticket.generation}`);
      if(beginDelay) await beginDelay.promise;
      return response;
    }
    if(command==="formal_baizhi_commit") {
      if(commitFailures-->0) throw new Error("E_SAVE_WRITE_FAILED");
      assert.ok(ticket); assert.deepEqual(Object.keys(args.ticket).sort(),["entityId","generation","interactionId","ownerId","pauseCommandSequence","sceneId","worldEpoch","worldId"], "Rust ticket denies unknown fields"); assert.deepEqual(args.ticket,ticket); assert.equal(args.ticket.pauseCommandSequence,lifecycle);
      current={...current,baizhi:{...current.baizhi,choice:args.choice}}; saved++;
      const response=result(args.requestId);
      if(commitDelay) await commitDelay.promise;
      return response;
    }
    if(command==="formal_baizhi_close") {
      if(closeFailures-->0) throw new Error("E_BAIZHI_OWNER_UNAVAILABLE");
      assert.ok(ticket); const commandId=`baizhi-close:${ticket.ownerId}:${ticket.generation}`; ticket=null;
      const response=result(commandId,!closeResumed,closeResumed);
      if(closeDelay) await closeDelay.promise;
      return response;
    }
    if(command==="formal_pause" || command==="formal_resume") {
      lifecycle=args.commandSequence; const receipt=result(command==="formal_pause"?"pause":"resume",command==="formal_pause").receipt; return receipt;
    }
    if(command==="formal_presentation_events" || command==="formal_sound_cues") return [];
    if(command==="formal_snapshot") return current;
    throw new Error(`unexpected command ${command}`);
  });
  const surface={addEventListener(){},removeEventListener(){}};
  const scheduler={now:()=>0,clientTimeMs:()=>0,setInterval:()=>1,clearInterval(){},requestAnimationFrame:()=>1,cancelAnimationFrame(){}};
  let model, context;
  const loop=new SessionLoop(client,{render:async()=>{},destroy:async()=>{}},{scheduler,windowTarget:surface,documentTarget:surface,
    onInteract:value=>context.interactFromSnapshot(value), onPauseState:()=>model?.reconcile(),onSnapshot:()=>model?.reconcile(),
    onError:error=>errors.push(error), onCleanup:()=>model?.reset()});
  model=new BaizhiDialogue(client,()=>changes.push({page:model.page,busy:model.busy,visible:model.visible,message:model.message}));
  context=vm.createContext({Error,sessionLoop:loop,interactionBusy:false,client,baizhi:model,coreUi:{isOpen:false},evolution:{visible:false},
    hud:{apply:deriveHudState,setFeedback(){}},dispatchInteractable(){throw new Error("Baizhi must not dispatch ordinary F");}});
  vm.runInContext(handler,context);
  return {loop,model,client,context,calls,changes,errors,start:()=>loop.start(current),saved:()=>saved,current:()=>current,
    async open(){loop.keyDown("f");await flush();await flush();},
    count:command=>calls.filter(c=>c.command===command).length};
}

test("compiled main F uses real bridge atomic begin and real SessionLoop pause before selectable UI",async()=>{
  const beginDelay=deferred(),h=harness({beginDelay});await h.start();await h.open();
  assert.equal(h.model.page,"opening");assert.equal(h.model.ready,false);assert.equal(h.model.busy,true);
  assert.equal(h.loop.pausePresentationState,"pausing");assert.equal(h.count("formal_pause"),0);
  h.loop.keyDown("f");h.loop.keyDown("j");h.model.next();assert.equal(h.count("formal_baizhi_begin"),1);
  beginDelay.resolve();await flush();await flush();assert.equal(h.model.page,"dialogue");assert.equal(h.model.ready,true);
  assert.equal(h.loop.pausePresentationState,"paused");assert.equal(h.count("formal_interact"),0);
  assert.deepEqual(Object.keys(h.calls.find(c=>c.command==="formal_baizhi_begin").args.request).sort(),["commandSequence","context","entityId","generation","interactionId","ownerId"]);
  await h.model.close();assert.equal(h.loop.pausePresentationState,"running");assert.equal(h.count("formal_resume"),0);await h.loop.stop();
});

test("taken and left persist only on confirm, stay held at result, then dedicated close releases",async()=>{
  for(const choice of ["taken","left"]){const h=harness();await h.start();await h.open();h.model.next();h.model.select(choice);
    assert.equal(h.model.page,"confirmation");assert.equal(h.saved(),0);await h.model.confirm();
    assert.equal(h.saved(),1);assert.equal(h.model.page,"result");assert.equal(h.loop.pausePresentationState,"paused");
    assert.equal(h.current().progression.eventSeq,3);assert.equal(h.count("formal_resume"),0);
    await h.model.close();assert.equal(h.model.visible,false);assert.equal(h.loop.pausePresentationState,"running");await h.loop.stop();}
});

test("Unresolved uses result without commit and can choose again; terminal Continue is read-only",async()=>{
  const h=harness();await h.start();await h.open();h.model.next();h.model.select("unresolved");assert.equal(h.model.page,"result");
  assert.equal(h.saved(),0);assert.equal(h.count("formal_baizhi_commit"),0);await h.model.close();await h.open();assert.equal(h.model.page,"dialogue");await h.model.close();await h.loop.stop();
  for(const choice of ["taken","left"]){const h=harness({choice});await h.start();await h.open();assert.equal(h.model.page,"result");
    assert.equal(h.model.choice,choice);h.model.next();h.model.select("unresolved");await h.model.confirm();assert.equal(h.model.page,"result");
    assert.equal(h.saved(),0);await h.model.close();await h.loop.stop();}
});

test("pending blocks all page changes, close, F, and ordinary resume; failure retries identical request payload",async()=>{
  const commitDelay=deferred(),h=harness({commitFailures:1,commitDelay});await h.start();await h.open();h.model.next();h.model.select("taken");
  await h.model.confirm();assert.equal(h.model.page,"confirmation");assert.equal(h.model.choice,"taken");assert.equal(h.model.message,"选择保存失败，请重试。");
  const pending=h.model.confirm();await flush();assert.equal(h.model.message,"正在保存选择…");
  h.model.back();h.model.escape();h.model.select("left");await h.model.close();await h.model.confirm();await h.loop.resume();h.loop.keyDown("f");
  assert.equal(h.model.page,"confirmation");assert.equal(h.model.busy,true);assert.equal(h.count("formal_baizhi_close"),0);assert.equal(h.count("formal_resume"),0);
  const requests=h.calls.filter(c=>c.command==="formal_baizhi_commit");assert.deepEqual(requests[0].args,requests[1].args);
  commitDelay.resolve();await pending;assert.equal(h.model.page,"result");assert.equal(h.saved(),1);await h.model.close();await h.loop.stop();
});

test("close failure preserves ticket and paused UI; retry alone restores movement",async()=>{
  const h=harness({closeFailures:1});await h.start();await h.open();await h.model.close();assert.equal(h.model.visible,true);
  assert.equal(h.model.message,"对话关闭失败，请重试。");assert.equal(h.loop.pausePresentationState,"paused");
  await h.model.close();const closes=h.calls.filter(c=>c.command==="formal_baizhi_close");assert.deepEqual(closes[0].args,closes[1].args);
  assert.equal(h.loop.pausePresentationState,"running");await h.loop.stop();
});

test("close receipt reporting revoked ownership clears dialog but never releases input",async()=>{
  const h=harness({closeResumed:false});await h.start();await h.open();await h.model.close();assert.equal(h.model.visible,false);
  assert.equal(h.loop.pausePresentationState,"paused");h.loop.keyDown("f");assert.equal(h.count("formal_baizhi_begin"),1);await h.loop.stop();
});

test("later lifecycle/manual pause supersedes begin, pending commit or pending close and sends higher native sequence",async()=>{
  for(const stage of ["begin","commit","close"]){
    const gate=deferred(),h=harness({[`${stage}Delay`]:gate});await h.start();await h.open();let pending;
    if(stage==="commit"){h.model.next();h.model.select("left");pending=h.model.confirm();}
    if(stage==="close")pending=h.model.close();
    await flush();await h.loop.pause(stage==="close"?"manual":"lifecycle");
    assert.equal(h.model.visible,false);assert.equal(h.loop.pausePresentationState,"paused");
    const begin=h.calls.find(c=>c.command==="formal_baizhi_begin"),pause=h.calls.find(c=>c.command==="formal_pause");
    assert.ok(pause.args.commandSequence>begin.args.request.commandSequence);
    gate.resolve();await pending;await flush();await flush();assert.equal(h.model.visible,false);assert.equal(h.loop.pausePresentationState,"paused");
    assert.equal(h.count("formal_resume"),0);if(stage==="commit")assert.equal(h.saved(),1,"late saved result is not cancelled or rolled back");await h.loop.stop();
  }
});

test("new session and forced cleanup discard late success without sending resume into replacement",async()=>{
  const commitDelay=deferred(),h=harness({commitDelay});await h.start();await h.open();h.model.next();h.model.select("taken");const pending=h.model.confirm();await flush();
  h.context.sessionLoop={};h.model.reconcile();assert.equal(h.model.visible,false);commitDelay.resolve();await pending;assert.equal(h.saved(),1);
  assert.equal(h.model.visible,false);assert.equal(h.count("formal_baizhi_close"),0);assert.equal(h.count("formal_resume"),0);await h.loop.stop();
});

test("exact projection drives F, all terminal states work, corrupted optional data leaves main snapshot valid",()=>{
  for(const choice of ["unresolved","taken","left"]){const s=snapshot(choice);assert.equal(deriveHudState(s).interactionId,"gh_bz_first_contact");
    assert.equal(baizhiInteractionPrompt(s),choice==="unresolved"?"[F] 与白芷交谈":"[F] 查看白芷状态");}
  for(const mutate of [s=>delete s.baizhi,s=>delete s.npcs,s=>{s.baizhi.choice="bogus";},s=>{s.baizhi.schemaVersion=2;},s=>{s.npcs.push(s.npcs[0]);},
    s=>{s.npcs[0].entityId="unknown";},s=>{s.npcs[0].interactable=false;},s=>{s.npcs[0].position[0]=NaN;},s=>{s.baizhi.available=false;},
    s=>{s.worldId="mist_harbor";},s=>{s.player.currentHp=0;},s=>{s.player.transform.positionM=pos(8,0,13.01);},
    s=>{s.baizhi.canInteract=false;s.npcs[0].interactable=false;}]){
    const s=snapshot();mutate(s);assert.doesNotThrow(()=>assertSnapshotV3(s));assert.equal(deriveHudState(s).interactionId,null);assert.equal(baizhiInteractionPrompt(s),null);
  }
});

// Deterministic DOM model for markup/focus/event semantics, not browser geometry evidence.
class ElementModel {
  constructor(tag="div"){this.tagName=tag.toUpperCase();this.children=[];this.dataset={};this.hidden=false;this.disabled=false;this.isConnected=true;this.listeners={};this.textContent="";}
  set innerHTML(value){this.markup=value;this.map=new Map();for(const id of ["baizhi-title","baizhi-lines","baizhi-status"]){const e=new ElementModel(id==="baizhi-title"?"h2":"div");this.map.set(`#${id}`,e);this.children.push(e);}const actions=new ElementModel();this.map.set(".baizhi-actions",actions);this.children.push(actions);this.map.set("h2",this.map.get("#baizhi-title"));}
  querySelector(s){return this.map?.get(s)??null;}
  querySelectorAll(s){const all=this.children.flatMap(c=>[c,...c.querySelectorAll(s)]);return all.filter(e=>e.tagName==="BUTTON"&&(!s.includes(":not(:disabled)")||!e.disabled));}
  addEventListener(k,v){this.listeners[k]=v;}
  append(...e){this.children.push(...e);}
  replaceChildren(...e){this.children=e;}
  contains(e){return e===this||this.children.some(c=>c.contains(e));}
  setAttribute(k,v){this[k]=v;}
  focus(){globalThis.document.activeElement=this;}
}
function key(key,shiftKey=false){return{key,shiftKey,repeat:false,preventDefault(){this.prevented=true;},stopImmediatePropagation(){this.stopped=true;}};}

test("DOM model verifies four frozen pages, default focus, Tab boundaries, exact copy and pending all-disabled",async()=>{
  const oldDoc=globalThis.document,oldHTMLElement=globalThis.HTMLElement;
  globalThis.HTMLElement=ElementModel;globalThis.document={activeElement:null,createElement:tag=>new ElementModel(tag)};
  try{
    const h=harness(),el=new ElementModel();el.hidden=true;const panel=new BaizhiDialoguePanel(el,h.model);
    await h.start();await h.open();panel.render();assert.equal(document.activeElement.textContent,"继续");
    assert.match(el.markup,/肖像待补·开发占位/);assert.equal(el.querySelector("#baizhi-lines").children[0].textContent,"别开那扇门。先听我说。");
    let event=key("Tab",true);panel.onKeyDown(event);assert.equal(document.activeElement.textContent,"关闭");assert.equal(event.prevented,true);
    event=key("Tab");panel.onKeyDown(event);assert.equal(document.activeElement.textContent,"继续");
    h.model.next();panel.render();assert.equal(document.activeElement.textContent,"暂不决定");assert.equal(el.querySelector("#baizhi-title").textContent,"你的决定");
    h.model.select("taken");panel.render();assert.equal(document.activeElement.textContent,"返回");assert.equal(el.querySelector("#baizhi-lines").children[0].textContent,"你决定帮助白芷撤离。");
    panel.onKeyDown(key("Escape"));assert.equal(h.model.page,"choices");panel.onKeyDown(key("Escape"));assert.equal(h.model.page,"dialogue");
    h.model.next();h.model.select("left");const pending=h.model.confirm();panel.render();assert.equal(el.querySelectorAll("button:not(:disabled)").length,0);
    event=key("Tab");panel.onKeyDown(event);assert.equal(event.prevented,true);assert.equal(document.activeElement.tagName,"H2");
    event=key("Escape");panel.onKeyDown(event);assert.equal(event.stopped,true);assert.equal(h.model.page,"confirmation");
    await pending;panel.render();assert.equal(document.activeElement.textContent,"关闭");assert.equal(el.querySelector("#baizhi-lines").children[1].textContent,`白芷：${BAIZHI_RESULTS.left.baizhi}`);
    panel.onKeyDown(key("Escape"));await flush();assert.equal(h.model.visible,false);await h.loop.stop();
  }finally{globalThis.document=oldDoc;globalThis.HTMLElement=oldHTMLElement;}
});

test("responsive modal styles bound overflow to frame; 720p/1080p actual browser geometry remains unverified",async()=>{
  const css=await readFile(new URL("../src/style.css",import.meta.url),"utf8");assert.match(css,/\.baizhi-dialogue-frame\s*\{[^}]*width: min\(760px, 100%\); max-height: 100%; overflow-y: auto/s);
  assert.match(compiled,/baizhi\.reconcile\(\)/);assert.match(compiled,/baizhi\.reset\(\)/);assert.match(compiled,/if \(baizhi\.visible\)\s+return/);
});

test("other modal, manual pause, dead and loading do not dispatch begin",async()=>{
  for(const gate of ["core","evolution","pause","dead","loading"]){
    const h=harness();await h.start();
    if(gate==="core")h.context.coreUi.isOpen=true;
    if(gate==="evolution")h.context.evolution.visible=true;
    if(gate==="pause")await h.loop.pause();
    if(gate==="dead")h.loop.dead=true;
    if(gate==="loading")h.loop.entryLoading=true;
    await h.open();assert.equal(h.count("formal_baizhi_begin"),0,gate);
    h.loop.dead=false;h.loop.entryLoading=false;await h.loop.stop();
  }
});

test("bridge rejects malformed begin identities, paused projection, ticket and receipt metadata",async()=>{
  for(const mutate of [r=>{r.ticket=null;},r=>{r.resumed=true;},r=>{r.ticket.generation++;},r=>{r.ticket.pauseCommandSequence++;},
    r=>{r.ticket.ownerId="other";},r=>{r.receipt.commandId="wrong";},r=>{r.receipt.worldEpoch++;},r=>{r.receipt.applied=false;},
    r=>{r.receipt.alreadyApplied=true;},r=>{r.receipt.snapshot.worldId="mist_harbor";},r=>{r.receipt.snapshot.baizhi.choice="wrong";},
    r=>{r.receipt.snapshot.npcs[0].interactable=true;},r=>{r.receipt.snapshot.player.currentHp=0;}]){
    const client=new TauriClient(async(command,args)=>{
      assert.equal(command,"formal_baizhi_begin");const request=args.request;
      const snapshotValue=snapshot("unresolved",{baizhi:{schemaVersion:1,choice:"unresolved",available:true,canInteract:false},
        npcs:[{entityId:"gh_bz_whitezhi_v1",entityType:"npc.baizhi",position:[8,0,10.5],yawRad:Math.PI,interactable:false}]});
      const result={receipt:{commandId:`baizhi-begin:${request.ownerId}:${request.generation}`,applied:true,alreadyApplied:false,errorCode:null,
        worldEpoch:8,serverTick:70,authorityRevision:9,snapshot:snapshotValue},resumed:false,
        ticket:{...request.context,ownerId:request.ownerId,generation:request.generation,entityId:request.entityId,
          interactionId:request.interactionId,pauseCommandSequence:request.commandSequence}};mutate(result);return result;
    });
    await assert.rejects(client.beginBaizhi(snapshot(),"owner",1));
  }
});

test("native close success followed by event read failure is safety-paused and never unlocks input",async()=>{
  const h=harness();await h.start();await h.open();h.client.events=async()=>{throw new Error("E_EVENT_READ_FAILED");};
  await h.model.close();assert.equal(h.model.visible,false);assert.equal(h.count("formal_baizhi_close"),1);
  assert.ok(h.count("formal_pause")>=1);assert.notEqual(h.loop.pausePresentationState,"running");
  h.loop.keyDown("f");assert.equal(h.count("formal_baizhi_begin"),1);
  h.client.events=async()=>[];await h.loop.stop();
});

test("confirmed saved choice remains result when presentation events fail, with no duplicate retry",async()=>{
  const h=harness();await h.start();await h.open();h.model.next();h.model.select("taken");
  h.client.events=async()=>{throw new Error("E_EVENT_READ_FAILED");};await h.model.confirm();
  assert.equal(h.saved(),1);assert.equal(h.model.page,"result");assert.equal(h.model.choice,"taken");assert.equal(h.model.message,"");
  assert.equal(h.loop.pausePresentationState,"paused");await h.model.confirm();assert.equal(h.count("formal_baizhi_commit"),1);
  h.client.events=async()=>[];await h.model.close();await h.loop.stop();
});

test("definite out-of-range preflight rejection leaves no new pause; unknown begin stays safely held",async()=>{
  for(const code of ["E_BAIZHI_OUT_OF_RANGE","E_BAIZHI_CONFIGURATION","E_BAIZHI_REQUEST_INVALID","E_RUNTIME_PAUSED"]){
    const h=harness();await h.start();const native=h.client.invoke;
    h.client.invoke=async(command,args)=>{if(command==="formal_baizhi_begin"){h.calls.push({command,args});throw code;}return native(command,args);};
    await h.open();assert.equal(h.model.visible,false);
    if(code==="E_RUNTIME_PAUSED"){assert.equal(h.count("formal_pause"),1);assert.equal(h.loop.pausePresentationState,"paused");}
    else{assert.equal(h.count("formal_pause"),0);assert.equal(h.loop.pausePresentationState,"running");}
    assert.equal(h.count("formal_resume"),0);await h.loop.stop();
  }
});

test("close redraws a renderer-verified held frame before native resume; failed draw keeps retry ticket",async()=>{
  const h=harness();await h.start();await h.open();h.loop.rendererEntryVerified=true;
  let drawable=false,draws=0;h.loop.renderer.render=async()=>{draws++;if(!drawable)throw new Error("E_RESIZE_NOT_READY");};
  h.loop.renderer.isReadyFor=()=>drawable;
  await h.model.close();assert.equal(h.count("formal_baizhi_close"),0);assert.equal(h.model.visible,true);assert.equal(h.model.message,"对话关闭失败，请重试。");
  assert.equal(h.loop.pausePresentationState,"paused");assert.equal(h.count("formal_pause"),0);
  drawable=true;await h.model.close();assert.equal(h.count("formal_baizhi_close"),1);assert.equal(draws,2);assert.equal(h.loop.pausePresentationState,"running");await h.loop.stop();
});

test("resize during native close receipt wait forces safety pause and never uses stale drawn proof",async()=>{
  const closeDelay=deferred(),h=harness({closeDelay});await h.start();await h.open();h.loop.rendererEntryVerified=true;
  let ready=true;h.loop.renderer.isReadyFor=()=>ready;
  const pending=h.model.close();await flush();assert.equal(h.count("formal_baizhi_close"),1);ready=false;closeDelay.resolve();await pending;
  assert.equal(h.model.visible,false);assert.notEqual(h.loop.pausePresentationState,"running");assert.equal(h.count("formal_pause"),1);
  ready=true;await h.loop.stop();
});

test("unknown IPC error after native close cannot leave a running invisible world",async()=>{
  const h=harness();await h.start();await h.open();const close=h.client.closeBaizhi.bind(h.client);
  h.client.closeBaizhi=async ticket=>{await close(ticket);throw new Error("E_IPC_CONNECTION_LOST");};
  await h.model.close();assert.equal(h.count("formal_baizhi_close"),1);assert.equal(h.count("formal_pause"),1);
  assert.equal(h.current().baizhi.canInteract,false);assert.equal(h.loop.pausePresentationState,"paused");assert.equal(h.model.visible,false);
  await h.loop.stop();
});

test("lost successful commit response reconciles terminal authority read-only without duplicate write",async()=>{
  const h=harness();await h.start();await h.open();h.model.next();h.model.select("taken");
  const commit=h.client.commitBaizhi.bind(h.client);
  h.client.commitBaizhi=async(...args)=>{await commit(...args);throw new Error("E_IPC_CONNECTION_LOST");};
  await h.model.confirm();assert.equal(h.saved(),1);assert.equal(h.model.page,"result");assert.equal(h.model.message,"");
  assert.equal(h.count("formal_snapshot"),1);assert.equal(h.count("formal_baizhi_commit"),1);assert.equal(h.loop.pausePresentationState,"paused");
  await h.model.confirm();assert.equal(h.saved(),1);await h.model.close();await h.loop.stop();
});

test("unavailable authority after lost commit keeps same payload; duplicate reply can reconcile later",async()=>{
  const h=harness();await h.start();await h.open();h.model.next();h.model.select("left");const commit=h.client.commitBaizhi.bind(h.client);
  let first=true;h.client.commitBaizhi=async(...args)=>{if(first){first=false;await commit(...args);throw new Error("E_IPC_CONNECTION_LOST");}throw new Error("E_BAIZHI_DUPLICATE_REQUEST");};
  const read=h.client.snapshot.bind(h.client);h.client.snapshot=async()=>{throw new Error("E_IPC_READ_FAILED");};
  await h.model.confirm();assert.equal(h.saved(),1);assert.equal(h.model.page,"confirmation");assert.equal(h.model.choice,"left");
  h.client.snapshot=read;await h.model.confirm();assert.equal(h.model.page,"result");assert.equal(h.saved(),1);
  await h.model.close();await h.loop.stop();
});

test("late read-only reconciliation cannot publish an old saved result over a new session or later pause",async()=>{
  for(const change of ["session","pause"]){
    const h=harness();await h.start();await h.open();h.model.next();h.model.select("taken");const commit=h.client.commitBaizhi.bind(h.client),gate=deferred();
    h.client.commitBaizhi=async(...args)=>{await commit(...args);throw new Error("E_IPC_CONNECTION_LOST");};
    h.client.snapshot=()=>gate.promise;const pending=h.model.confirm();await flush();
    if(change==="session"){h.context.sessionLoop={};h.model.reconcile();}else await h.loop.pause("lifecycle");
    gate.resolve(h.current());await pending;assert.equal(h.model.visible,false);assert.equal(h.saved(),1);assert.equal(h.count("formal_resume"),0);
    assert.notEqual(h.loop.pausePresentationState,"running");await h.loop.stop();
  }
});

test("actual compiled bridge request JSON equals the fixture parsed strictly by Rust",async()=>{
  const fixture=JSON.parse(await readFile(new URL("../../../server-rs/tests/fixtures/baizhi-wire-v1.json",import.meta.url),"utf8"));
  const captured=[];let ticket,current=snapshot();
  const client=new TauriClient(async(command,args)=>{
    captured.push(structuredClone({command,args}));let commandId;
    if(command==="formal_baizhi_begin"){
      const r=args.request;ticket={...r.context,ownerId:r.ownerId,generation:r.generation,entityId:r.entityId,interactionId:r.interactionId,pauseCommandSequence:r.commandSequence};
      commandId=`baizhi-begin:${r.ownerId}:${r.generation}`;
    }else if(command==="formal_baizhi_commit"){commandId=args.requestId;current.baizhi.choice=args.choice;}
    else commandId=`baizhi-close:${ticket.ownerId}:${ticket.generation}`;
    const closing=command==="formal_baizhi_close";current={...current,authorityRevision:current.authorityRevision+1,
      baizhi:{...current.baizhi,canInteract:closing},npcs:[{...current.npcs[0],interactable:closing}]};
    return{receipt:{commandId,applied:true,alreadyApplied:false,errorCode:null,worldEpoch:8,serverTick:70,authorityRevision:current.authorityRevision,snapshot:current},ticket:closing?null:ticket,resumed:closing};
  });
  const begun=await client.beginBaizhi(snapshot(),"baizhi-wire-fixture",7);
  const polluted={...snapshot(),...begun.ticket,accidentalSaveField:"must-not-transmit"};
  await client.commitBaizhi(polluted,"baizhi-wire-choice-1","taken");await client.closeBaizhi(polluted);
  assert.deepEqual(captured,fixture);
});
