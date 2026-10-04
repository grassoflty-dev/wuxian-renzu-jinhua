import test from "node:test";
import assert from "node:assert/strict";
import { FirstEvolution, pendingFirstEvolution } from "../dist/game/FirstEvolution.js";

const id = "information.local_map_i";
const deferred = () => { let resolve, reject; const promise = new Promise((a,b) => { resolve=a; reject=b; }); return {promise,resolve,reject}; };
function fixture() {
  let current = { protocolVersion: 3, worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 4, authorityRevision: 10,
    capabilities: { firstEnhancementChoice: null, items: [] }, progression: { worlds: [{ worldId: "grey_hive", completed: true, firstCompletion: false }] } };
  const calls = [];
  const loop = { pausePresentationState: "running", acceptsExternalResults: true, isDead: false,
    snapshots: { view: () => current },
    async pause() { calls.push("pause"); this.pausePresentationState = "paused"; },
    async resume() { calls.push("resume"); this.pausePresentationState = "running"; },
    async runWhilePaused(fn) { calls.push("runWhilePaused"); return fn(); },
    async acceptAuthoritativeSnapshot(snapshot) { calls.push("accept"); current = snapshot; },
  };
  let sameOwner = true;
  const owner = { loop, isCurrent: () => sameOwner };
  const client = { confirmedPauseSequence() { calls.push("sequence"); return 7; },
    async chooseFirstEnhancement(capabilityId, context) {
      calls.push({capabilityId,context});
      return { applied: true, authorityRevision: 11, snapshot: { ...current, authorityRevision: 11,
        capabilities: {firstEnhancementChoice: capabilityId, items:[{capabilityId,granted:true,selected:true}]} } };
    } };
  const controller = new FirstEvolution(client, () => {});
  const receipt = {applied:true,commandId:"terminal-4",requestId:"terminal-4",snapshot:current};
  return { controller, loop, owner, client, calls, receipt, setSnapshot(v) {current=v;}, replace() { sameOwner=false; } };
}

test("pending derives only completed GH and explicit null, regardless of grants, firstCompletion, or other selected", () => {
  const f=fixture(), snapshot=f.receipt.snapshot;
  snapshot.capabilities.items=[{capabilityId:id,granted:true,selected:true}];
  assert.equal(pendingFirstEvolution(snapshot),true);
  for(const value of [undefined,id,"other"]){snapshot.capabilities.firstEnhancementChoice=value;assert.equal(pendingFirstEvolution(snapshot),false);}
  snapshot.capabilities.firstEnhancementChoice=null;snapshot.progression.worlds[0].completed=false;
  assert.equal(pendingFirstEvolution(snapshot),false);
});
test("valid RS receipt then confirmed pause opens modal; success accepts receipt, closes, and resumes once", async()=>{
  const f=fixture();await f.controller.open(f.owner,f.receipt);
  assert.equal(f.controller.ready,true);assert.deepEqual(f.calls,["pause","sequence"]);
  const result=await f.controller.choose(id);assert.equal(result.snapshot.capabilities.firstEnhancementChoice,id);
  assert.deepEqual(f.calls.slice(2),["runWhilePaused",{capabilityId:id,context:{id:"rs_capability_terminal_marker",requestId:"terminal-4",worldEpoch:4,pauseCommandSequence:7}},"accept","resume"]);
  assert.equal(f.controller.visible,false);
});
test("close/later/escape path grants nothing and resumes exact modal once",async()=>{
  const f=fixture();await f.controller.open(f.owner,f.receipt);await f.controller.close();await f.controller.close();
  assert.deepEqual(f.calls,["pause","sequence","resume"]);assert.equal(f.receipt.snapshot.capabilities.firstEnhancementChoice,null);
});
test("unapplied, forged, GH and ordinary pause cannot open Evolution",async()=>{
  for(const kind of ["unapplied","forged","GH","ordinaryPause","alreadyChosen"]){
    const f=fixture();if(kind==="unapplied")f.receipt.applied=false;if(kind==="forged")f.receipt.commandId="other";
    if(kind==="GH")f.receipt.snapshot.worldId="grey_hive";if(kind==="ordinaryPause")f.loop.pausePresentationState="paused";
    if(kind==="alreadyChosen")f.receipt.snapshot.capabilities.firstEnhancementChoice=id;
    await f.controller.open(f.owner,f.receipt);assert.equal(f.controller.visible,false,kind);assert.deepEqual(f.calls,[],kind);
  }
});
test("pause failure is never followed by unsafe resume; no reward is submitted",async()=>{
  const f=fixture();f.loop.pause=async()=>{f.loop.pausePresentationState="error";};
  await f.controller.open(f.owner,f.receipt);assert.equal(f.controller.ready,false);await f.controller.choose(id);await f.controller.close();
  assert.deepEqual(f.calls,[]);
});
test("double click and close while saving cannot duplicate or overtake confirmation",async()=>{
  const f=fixture(), pending=deferred();await f.controller.open(f.owner,f.receipt);
  const normal=f.client.chooseFirstEnhancement;f.client.chooseFirstEnhancement=async(...args)=>{await pending.promise;return normal(...args);};
  const first=f.controller.choose(id);assert.equal(await f.controller.choose(id),null);await f.controller.close();
  assert.equal(f.controller.visible,true);assert.equal(f.calls.includes("resume"),false);
  pending.resolve();await first;assert.equal(f.calls.filter(x=>typeof x==="object").length,1);assert.equal(f.calls.filter(x=>x==="resume").length,1);
});
test("save rejection keeps modal paused and same ticket retryable",async()=>{
  for (const code of ["E_SAVE_WRITE", "E_SLOT_NOT_FOUND", "E_SLOT_CORRUPT", "E_SLOT_TEMP"]) {
    const f=fixture();await f.controller.open(f.owner,f.receipt);const normal=f.client.chooseFirstEnhancement;
    const attempts=[];
    f.client.chooseFirstEnhancement=async(capabilityId,context)=>{
      attempts.push({capabilityId,context:{...context}});throw new Error(code);
    };
    assert.equal(await f.controller.choose(id),null);assert.equal(f.controller.visible,true);assert.equal(f.controller.ready,true);
    assert.ok(f.controller.message.includes(code));assert.equal(f.calls.includes("resume"),false);
    assert.equal(f.loop.pausePresentationState,"paused");
    assert.equal(f.receipt.snapshot.capabilities.firstEnhancementChoice,null);
    f.client.chooseFirstEnhancement=async(capabilityId,context)=>{
      attempts.push({capabilityId,context:{...context}});return normal(capabilityId,context);
    };
    assert.ok(await f.controller.choose(id));
    assert.deepEqual(attempts[1],attempts[0],"Same chosen capability and modal ticket survive a target failure");
  }
});
test("stale pause receipt and scene/epoch/session replacement never resume another journey",async()=>{
  for(const kind of ["scene","epoch","owner","dead"]){
    const f=fixture(),pending=deferred();f.loop.pause=async()=>{await pending.promise;f.loop.pausePresentationState="paused";};
    const opening=f.controller.open(f.owner,f.receipt);
    if(kind==="scene")f.setSnapshot({...f.receipt.snapshot,sceneId:"other"});
    if(kind==="epoch")f.setSnapshot({...f.receipt.snapshot,worldEpoch:5});
    if(kind==="owner")f.replace();if(kind==="dead")f.loop.isDead=true;
    pending.resolve();await opening;await f.controller.close();assert.equal(f.controller.visible,false);assert.deepEqual(f.calls,[]);
  }
});
test("late or foreign selection receipt is discarded without accepting or resuming",async()=>{
  for(const kind of ["owner","epoch","receipt"]){
    const f=fixture(),pending=deferred();await f.controller.open(f.owner,f.receipt);
    const normal=f.client.chooseFirstEnhancement;f.client.chooseFirstEnhancement=async(...args)=>{await pending.promise;const r=await normal(...args);if(kind==="receipt")r.snapshot={...r.snapshot,worldEpoch:5};return r;};
    const choosing=f.controller.choose(id);if(kind==="owner")f.replace();if(kind==="epoch")f.setSnapshot({...f.receipt.snapshot,worldEpoch:5});
    pending.resolve();await choosing;assert.equal(f.calls.includes("accept"),false);assert.equal(f.calls.includes("resume"),false);
  }
});
test("a late opening or choice cannot overwrite or dismiss a newer modal operation",async()=>{
  const f=fixture(),old=deferred();f.loop.pause=async()=>{await old.promise;f.loop.pausePresentationState="paused";};
  const opening=f.controller.open(f.owner,f.receipt);f.controller.reset();
  const b=fixture();b.receipt.snapshot={...b.receipt.snapshot,worldEpoch:5};b.setSnapshot(b.receipt.snapshot);b.receipt.requestId=b.receipt.commandId="terminal-5";
  await f.controller.open(b.owner,b.receipt);old.resolve();await opening;
  assert.equal(f.controller.ready,true);
  const receipt=await f.controller.choose(id);assert.equal(receipt,null); // f client's foreign receipt must fail, never overwrite B.
  assert.equal(b.calls.includes("resume"),false);
  assert.equal(f.controller.visible,true);
});
test("a rejected pause leaves close available and never resumes unconfirmed authority",async()=>{
  const f=fixture();f.loop.pause=async()=>{throw Error("pause transport failed");};
  await f.controller.open(f.owner,f.receipt);assert.equal(f.controller.busy,false);assert.equal(f.controller.ready,false);
  await f.controller.close();assert.deepEqual(f.calls,[]);assert.equal(f.controller.visible,false);
});
