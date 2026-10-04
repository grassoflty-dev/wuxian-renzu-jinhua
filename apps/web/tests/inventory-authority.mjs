import test from "node:test";
import assert from "node:assert/strict";
import { assertBuildProjection, assertBuildAction } from "../dist/protocol/BuildProjection.js";
import { assertSnapshotV3 } from "../dist/protocol/types.js";
import { TauriClient } from "../dist/bridge/tauri-client.js";
import { BuildCloseBarrier, InventoryCommandController, isCurrentBuildReceipt } from "../dist/ui/InventoryCommandController.js";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";
import { inventorySnapshot, equippedSnapshot, receipt } from "./fixtures/inventory-snapshot.mjs";

const action = { kind: "equip", itemId: "rear_view_lens", expectedItemId: null };
function deferred() { let resolve, reject; const promise = new Promise((a,b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; }

test("optional Build projection distinguishes unavailable, empty, and internally authorized owned content", () => {
  const old = inventorySnapshot(); delete old.build;
  assert.doesNotThrow(() => assertSnapshotV3(old));
  assert.match(deriveCorePanel("inventory", old).rows[0].value, /尚未收到/);
  const empty = inventorySnapshot({ build: { schemaVersion: 1, revision: 0, items: [], equipment: [] } });
  assertSnapshotV3(empty); assert.match(deriveCorePanel("inventory", empty).rows[0].value, /0 种/);
  const projected = assertBuildProjection(inventorySnapshot().build);
  assert.equal(projected.items[0].releaseEligible, false);
  assert.equal(projected.items[0].quantity, 2);
  assertBuildProjection(equippedSnapshot().build);
});

test("malformed Build data never fabricates ownership or accepts client effect definitions", () => {
  for (const change of [
    value => { value.schemaVersion = 2; }, value => { value.revision = Number.MAX_SAFE_INTEGER + 1; },
    value => { value.items.push({ ...value.items[0] }); }, value => { value.items[0].quantity = 0; },
    value => { value.items[0].effects = []; }, value => { value.items[0].releaseEligible = "yes"; },
    value => { value.items[0].equipped = true; }, value => { value.items[0].itemId = "<script>"; },
    value => { value.equipment = [{ itemId: "not_owned", slotId: "lens", label: "fake" }]; },
  ]) { const value = structuredClone(inventorySnapshot().build); change(value); assert.throws(() => assertBuildProjection(value), /E_BUILD_PROJECTION_INVALID/); }
  assert.throws(() => assertSnapshotV3(inventorySnapshot({ build: null })), /E_BUILD_PROJECTION_INVALID/);
  for (const bad of [{ ...action, effects: [] }, { kind: "grant", itemId: "rear_view_lens" }, { kind: "equip", itemId: "rear_view_lens" }]) {
    assert.throws(() => assertBuildAction(bad), /E_BUILD_ACTION_INVALID/);
  }
});

test("Tauri Build requests carry only guarded IDs and validate receipt identity and revision", async () => {
  const calls = [];
  const client = new TauriClient(async (name, args) => {
    calls.push([name,args]); return receipt(equippedSnapshot(), { commandId: `build:${args.command.requestId}` });
  });
  const result = await client.buildCommand(action, inventorySnapshot());
  assert.equal(result.applied, true); assert.equal(calls[0][0], "formal_build_command");
  const command = calls[0][1].command;
  assert.deepEqual(Object.keys(command).sort(), ["action","expectedBuildRevision","requestId","worldEpoch"]);
  assert.deepEqual(command.action, action); assert.equal(command.expectedBuildRevision, 2); assert.equal(command.worldEpoch, 4);
  for (const mutate of [r => { r.commandId = "wrong"; }, r => { r.snapshot.build.revision = 2; },
    r => { r.worldEpoch++; }, r => { r.applied = true; r.alreadyApplied = true; }]) {
    const invalid = new TauriClient(async (name,args) => { const r = receipt(equippedSnapshot(), { commandId: `build:${args.command.requestId}` }); mutate(r); return r; });
    await assert.rejects(invalid.buildCommand(action, inventorySnapshot()), /E_BUILD_RECEIPT_INVALID/);
  }
});

test("rejected Build receipts remain authoritative data and do not masquerade as applied", async () => {
  const client = new TauriClient(async (name,args) => receipt(inventorySnapshot(), {
    commandId: `build:${args.command.requestId}`, applied: false, errorCode: "E_BUILD_STALE_REVISION",
  }));
  const result = await client.buildCommand(action, inventorySnapshot());
  assert.equal(result.applied, false); assert.equal(result.errorCode, "E_BUILD_STALE_REVISION");
});

test("single-flight UI waits for authority and never optimistically equips", async () => {
  const d = deferred(); let calls = 0, renders = 0;
  const controller = new InventoryCommandController(() => { calls++; return d.promise; }, () => renders++);
  const source = inventorySnapshot(); controller.apply(source);
  const pending = controller.submit(action); await controller.submit(action);
  assert.equal(calls, 1); assert.equal(controller.pending, true); assert.equal(source.build.equipment.length, 0);
  d.resolve(receipt()); await pending;
  assert.equal(controller.pending, false); assert.match(controller.feedback, /已更新/); assert.equal(renders, 2);
});

test("close, navigation, and new epochs discard pending feedback without reopening or poisoning a new action", async () => {
  for (const reset of [c => c.cancel(), c => c.apply(null), c => c.apply(inventorySnapshot({ worldEpoch: 5 })), c => c.apply(inventorySnapshot({ sceneId: "same-epoch-context" }))]) {
    const d = deferred(); let renders = 0;
    const controller = new InventoryCommandController(() => d.promise, () => renders++);
    controller.apply(inventorySnapshot()); const p = controller.submit(action); reset(controller);
    d.resolve(receipt()); await p; assert.equal(controller.pending, false); assert.equal(controller.feedback, ""); assert.equal(renders, 1);
  }
});

test("failure clears pending state and allows a fresh explicitly requested retry", async () => {
  let calls = 0; const controller = new InventoryCommandController(async () => {
    calls++; if (calls === 1) throw new Error("transport failed"); return receipt();
  }, () => {});
  controller.apply(inventorySnapshot()); await controller.submit(action);
  assert.equal(controller.pending, false); assert.match(controller.feedback, /transport failed/);
  await controller.submit(action); assert.equal(calls, 2); assert.match(controller.feedback, /已更新/);
});

test("receipt gate rejects stale epoch, scene, frame, Build revision and closed sessions", () => {
  const source = inventorySnapshot(), r = receipt();
  assert.equal(isCurrentBuildReceipt(true, source, source, r), true);
  for (const current of [null, inventorySnapshot({worldEpoch: 5}), inventorySnapshot({sceneId: "next_scene"}),
    inventorySnapshot({serverTick: 11}), inventorySnapshot({authorityRevision: 12}),
    inventorySnapshot({build:{...source.build,revision:5}})]) {
    assert.equal(isCurrentBuildReceipt(true, current, source, r), false);
  }
  assert.equal(isCurrentBuildReceipt(false, source, source, r), false);
});


test("newly applied receipts must satisfy the requested equipment postcondition", async () => {
  for(const action of [{kind:"equip",itemId:"rear_view_lens",expectedItemId:null},{kind:"unequip",slotId:"lens",expectedItemId:"rear_view_lens"}]) {
    const source=action.kind==="equip"?inventorySnapshot():equippedSnapshot();
    const wrong={...source,authorityRevision:source.authorityRevision+1,build:{...source.build,revision:source.build.revision+1}};
    const client=new TauriClient(async(name,args)=>receipt(wrong,{commandId:`build:${args.command.requestId}`}));
    await assert.rejects(client.buildCommand(action,source),/E_BUILD_RECEIPT_POSTCONDITION/);
    const duplicate=new TauriClient(async(name,args)=>receipt(wrong,{commandId:`build:${args.command.requestId}`,applied:false,alreadyApplied:true}));
    assert.equal((await duplicate.buildCommand(action,source)).alreadyApplied,true);
  }
});


test("Exit closes admission synchronously while draining pending hub equipment, and failures release the barrier", async () => {
  const barrier=new BuildCloseBarrier();const pending=deferred();const order=[];
  const exiting=barrier.close(async()=>{order.push("drain");await pending.promise;},async()=>{order.push("close");});
  assert.equal(barrier.closing,true);
  for(const intent of ["equip","new","continue"]) { if(!barrier.closing)order.push(intent); }
  assert.deepEqual(order,["drain"]);pending.resolve();await exiting;assert.deepEqual(order,["drain","close"]);
  assert.equal(barrier.closing,true);await assert.rejects(barrier.close(async()=>{},async()=>{}),/E_BUILD_CLOSING/);
  const failed=new BuildCloseBarrier();await assert.rejects(failed.close(async()=>{throw new Error("transport failed");},async()=>assert.fail("must not close")),/transport failed/);
  assert.equal(failed.closing,false);
  await assert.rejects(failed.close(async()=>{},async()=>{throw new Error("close rejected");}),/close rejected/);assert.equal(failed.closing,false);
});
