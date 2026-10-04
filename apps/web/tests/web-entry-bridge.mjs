import test from "node:test";
import assert from "node:assert/strict";
import { TauriClient } from "../dist/bridge/tauri-client.js";

function snapshot() {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "grey_hive", checkpointId: null,
    worldEpoch: 1, serverTick: 2, authorityRevision: 2, ackSeq: 0,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
      currentEnergy: 100, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1 },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
  };
}

test("desktop bridge accepts authoritative v3 snapshot and receipts", async () => {
  const calls = [];
  const client = new TauriClient(async command => {
    calls.push(command);
    if (command === "formal_snapshot") return snapshot();
    const value = command === "formal_new"
      ? { ...snapshot(), worldId: "return_station", sceneId: "rs_core_room" }
      : snapshot();
    return { snapshot: value };
  });
  assert.equal((await client.snapshot()).protocolVersion, 3);
  assert.equal((await client.newJourney()).sceneId, "rs_core_room");
  assert.equal((await client.continueJourney()).worldId, "grey_hive");
  assert.equal((await client.returnToHub()).protocolVersion, 3);
  assert.deepEqual(calls, ["formal_snapshot", "formal_new", "formal_continue", "formal_return"]);
});

test("new journey rejects a stalled native call and ignores its late receipt", async () => {
  let resolveNative;
  const client = new TauriClient(command => {
    assert.equal(command, "formal_new");
    return new Promise(resolve => { resolveNative = resolve; });
  });
  await assert.rejects(client.newJourney(20), /E_NEW_JOURNEY_TIMEOUT/);
  resolveNative({ snapshot: { ...snapshot(), worldId: "return_station", sceneId: "rs_core_room" } });
  await new Promise(resolve => setTimeout(resolve, 0));
  await assert.rejects(client.newJourney(0), /E_NEW_JOURNEY_TIMEOUT_INVALID/);
});

test("selected continue times out, ignores a late receipt, and sends only one native command", async () => {
  let resolveNative;
  const calls = [];
  const client = new TauriClient((command, args) => {
    calls.push({ command, args });
    return new Promise(resolve => { resolveNative = resolve; });
  });
  await assert.rejects(client.continueSlot("slot-a", 20), /E_CONTINUE_SLOT_TIMEOUT/);
  resolveNative(slotReceipt("continue-slot"));
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.deepEqual(calls, [{ command: "formal_continue_slot", args: { slotId: "slot-a" } }]);
  await assert.rejects(client.continueSlot("slot-a", 0), /E_CONTINUE_SLOT_TIMEOUT_INVALID/);
});

test("desktop bridge rejects a v2 response from formal runtime", async () => {
  const client = new TauriClient(async () => ({ ...snapshot(), protocolVersion: 2 }));
  await assert.rejects(client.snapshot(), /E_SNAPSHOT_PROTOCOL/);
});

test("scene interaction uses the formal command and refreshes from an authoritative v3 snapshot", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args });
    if (command === "formal_interact") return { applied: true, errorCode: null };
    if (command === "formal_snapshot") return snapshot();
    throw new Error(`unexpected command ${command}`);
  });
  const result = await client.interact("gh_power_console");
  assert.equal(result.applied, true);
  assert.equal(result.snapshot.protocolVersion, 3);
  assert.deepEqual(calls.map(call => call.command), ["formal_interact", "formal_snapshot"]);
  assert.deepEqual(calls[0].args && Object.keys(calls[0].args).sort(), ["actorId", "requestId"]);
  assert.match(calls[0].args.requestId, /^web-interaction-/);
});

test("formal pause and resume parse matching authoritative v3 receipt snapshots", async () => {
  const calls = [];
  const client = new TauriClient(async command => {
    calls.push(command);
    const view = snapshot();
    return {
      commandId: command === "formal_pause" ? "pause" : "resume",
      applied: true, alreadyApplied: false, errorCode: null,
      worldEpoch: view.worldEpoch, serverTick: view.serverTick, authorityRevision: view.authorityRevision,
      snapshot: view,
    };
  });
  assert.equal((await client.pause()).protocolVersion, 3);
  assert.equal((await client.resume()).protocolVersion, 3);
  assert.deepEqual(calls, ["formal_pause", "formal_resume"]);
});

test("lifecycle bridge rejects failed, malformed, and mismatched receipts", async () => {
  const base = snapshot();
  const receipt = { commandId: "pause", applied: true, alreadyApplied: false, errorCode: null,
    worldEpoch: base.worldEpoch, serverTick: base.serverTick, authorityRevision: base.authorityRevision, snapshot: base };
  const failed = new TauriClient(async () => ({ ...receipt, applied: false, errorCode: "E_RUNTIME_PAUSED" }));
  await assert.rejects(failed.pause(), /E_LIFECYCLE_COMMAND_REJECTED:E_RUNTIME_PAUSED/);
  const mismatched = new TauriClient(async () => ({ ...receipt, serverTick: 999 }));
  await assert.rejects(mismatched.pause(), /E_LIFECYCLE_RECEIPT_SNAPSHOT_MISMATCH/);
  const invalid = new TauriClient(async () => ({ ...receipt, snapshot: { ...base, protocolVersion: 2 } }));
  await assert.rejects(invalid.resume(), /E_SNAPSHOT_PROTOCOL/);
});

function slot(overrides = {}) {
  return { slotId: "slot-a", displayName: "灰巢进度", updatedAtMs: 1000, worldId: "grey_hive",
    checkpointId: null, playerPositionM: null, currentHp: 90, maxHp: 100, currentEnergy: 60,
    maxEnergy: 100, gateOpen: false, completedEvents: [], readOnly: false, valid: true, errorCode: null, ...overrides };
}

function slotReceipt(commandId = "save-slot") {
  const view = snapshot();
  return { commandId, applied: true, alreadyApplied: false, errorCode: null, worldEpoch: view.worldEpoch,
    serverTick: view.serverTick, authorityRevision: view.authorityRevision, snapshot: view };
}

function enhancementReceipt(capabilityId = "information.local_map_i") {
  const view = snapshot();
  view.progression.worlds = [{ worldId: "grey_hive", completed: true, firstCompletion: true }];
  view.capabilities.items = [{ capabilityId, granted: true, selected: true }];
  view.authorityRevision++;
  return { commandId: `enhancement:${capabilityId}`, applied: true, alreadyApplied: false, errorCode: null,
    worldEpoch: view.worldEpoch, serverTick: view.serverTick, authorityRevision: view.authorityRevision, snapshot: view };
}

test("first-clear enhancement invokes explicit capability choice and validates granted receipt", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args });
    return enhancementReceipt(args.capabilityId);
  });
  const receipt = await client.chooseFirstEnhancement("information.local_map_i");
  assert.equal(receipt.commandId, "enhancement:information.local_map_i");
  assert.equal(receipt.snapshot.capabilities.items[0].granted, true);
  assert.deepEqual(calls, [{ command: "formal_choose_first_enhancement", args: { capabilityId: "information.local_map_i" } }]);

  const wrongCommand = new TauriClient(async () => ({ ...enhancementReceipt(), commandId: "enhancement:perception.rear_view_i" }));
  await assert.rejects(wrongCommand.chooseFirstEnhancement("information.local_map_i"), /E_ENHANCEMENT_RECEIPT_COMMAND_MISMATCH/);
  const wrongCounters = new TauriClient(async () => ({ ...enhancementReceipt(), authorityRevision: 999 }));
  await assert.rejects(wrongCounters.chooseFirstEnhancement("information.local_map_i"), /E_ENHANCEMENT_RECEIPT_SNAPSHOT_MISMATCH/);
  const noGrant = new TauriClient(async () => {
    const receipt = enhancementReceipt();
    return { ...receipt, snapshot: { ...receipt.snapshot, capabilities: { schemaVersion: 1, items: [] } } };
  });
  await assert.rejects(noGrant.chooseFirstEnhancement("information.local_map_i"), /E_ENHANCEMENT_RECEIPT_NOT_GRANTED/);
});

test("slot bridge lists slots and sends exact create, overwrite, and selected-continue protocols", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args });
    if (command === "formal_list_save_slots") return [slot()];
    return slotReceipt(command === "formal_continue_slot" ? "continue-slot" : "save-slot");
  });
  assert.equal((await client.listSaveSlots())[0].slotId, "slot-a");
  assert.equal((await client.saveSlot("slot-b", "新记录", true)).snapshot.protocolVersion, 3);
  assert.equal((await client.saveSlot("slot-a", "灰巢进度", false)).applied, true);
  assert.equal((await client.continueSlot("slot-a")).snapshot.sceneId, "grey_hive");
  assert.deepEqual(calls, [
    { command: "formal_list_save_slots", args: undefined },
    { command: "formal_save_slot", args: { slotId: "slot-b", displayName: "新记录", create: true } },
    { command: "formal_save_slot", args: { slotId: "slot-a", displayName: "灰巢进度", create: false } },
    { command: "formal_continue_slot", args: { slotId: "slot-a" } },
  ]);
});

test("slot bridge rejects corrupt list rows, failed receipts, and delayed IPC failures", async () => {
  const malformed = new TauriClient(async () => [{ slotId: "bad", displayName: "坏档", valid: true }]);
  await assert.rejects(malformed.listSaveSlots(), /E_SAVE_SLOT_SUMMARY_INVALID/);
  const failed = new TauriClient(async () => ({ ...slotReceipt(), applied: false, errorCode: "E_SLOT_READ_ONLY" }));
  await assert.rejects(failed.saveSlot("legacy", "旧版", false), /E_SLOT_COMMAND_REJECTED:E_SLOT_READ_ONLY/);
  let finish;
  const delayed = new TauriClient(() => new Promise((_, reject) => { finish = reject; }));
  const pending = delayed.continueSlot("slot-a");
  assert.equal(typeof finish, "function");
  finish(new Error("IPC timeout"));
  await assert.rejects(pending, /IPC timeout/);
});

test("scene bridge sends camelCase identity and source epoch for route, checkpoint, and trigger receipts", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args });
    return { ...slotReceipt(command.replace("formal_", "")), commandId: args.requestId };
  });
  const route = await client.sceneTransition("gh_exit", 4);
  const checkpoint = await client.sceneCheckpoint("gh_checkpoint", 5);
  const trigger = await client.sceneTrigger("gh_trigger", 6);
  const gate = await client.worldGate("rs_world_gate_to_gh", 7);
  const mission = await client.missionTerminalStatus("rs_mission_terminal_marker", 8);
  const capability = await client.capabilityTerminalStatus("rs_capability_terminal_marker", 9);
  const rest = await client.saveRestTerminal("rs_save_rest_terminal_marker", 8);
  assert.match(route.requestId, /^web-scene-/);
  assert.equal(route.snapshot.worldEpoch, 1);
  assert.equal(checkpoint.commandId, checkpoint.requestId);
  assert.equal(trigger.commandId, trigger.requestId);
  assert.equal(gate.commandId, gate.requestId);
  assert.equal(mission.commandId, mission.requestId);
  assert.equal(capability.commandId, capability.requestId);
  assert.equal(rest.commandId, rest.requestId);
  assert.deepEqual(calls.map(({ command, args }) => ({ command, keys: Object.keys(args).sort(), id: args.id, epoch: args.worldEpoch })), [
    { command: "formal_scene_transition", keys: ["id", "requestId", "worldEpoch"], id: "gh_exit", epoch: 4 },
    { command: "formal_scene_checkpoint", keys: ["id", "requestId", "worldEpoch"], id: "gh_checkpoint", epoch: 5 },
    { command: "formal_scene_trigger", keys: ["id", "requestId", "worldEpoch"], id: "gh_trigger", epoch: 6 },
    { command: "formal_world_gate", keys: ["id", "requestId", "worldEpoch"], id: "rs_world_gate_to_gh", epoch: 7 },
    { command: "formal_mission_terminal_status", keys: ["id", "requestId", "worldEpoch"], id: "rs_mission_terminal_marker", epoch: 8 },
    { command: "formal_capability_terminal_status", keys: ["id", "requestId", "worldEpoch"], id: "rs_capability_terminal_marker", epoch: 9 },
    { command: "formal_save_rest_terminal", keys: ["id", "requestId", "worldEpoch"], id: "rs_save_rest_terminal_marker", epoch: 8 },
  ]);
  assert.equal(new Set(calls.map(call => call.args.requestId)).size, 7);
});

test("scene bridge preserves locked receipts and rejects stale, malformed, and delayed IPC failures", async () => {
  const locked = new TauriClient(async (_command, args) => ({ ...slotReceipt("scene-transition"), commandId: args.requestId, applied: false, errorCode: "E_SCENE_RUNTIME_ProgressionLocked" }));
  const lockedReceipt = await locked.sceneTransition("gh_locked_exit", 4);
  assert.equal(lockedReceipt.applied, false);
  assert.match(lockedReceipt.errorCode, /ProgressionLocked/);
  const rejectedTrigger = new TauriClient(async (_command, args) => ({
    ...slotReceipt(), commandId: args.requestId, applied: false, errorCode: "E_SCENE_RUNTIME_OutOfRange",
  }));
  const triggerReceipt = await rejectedTrigger.sceneTrigger("gh_trigger", 4);
  assert.equal(triggerReceipt.applied, false, "authoritative rejection remains visible to the interaction handler");
  assert.equal(triggerReceipt.errorCode, "E_SCENE_RUNTIME_OutOfRange");
  const stale = new TauriClient(async (_command, args) => ({ ...slotReceipt(), commandId: args.requestId, serverTick: 99 }));
  await assert.rejects(stale.sceneCheckpoint("cp", 4), /E_SCENE_RECEIPT_SNAPSHOT_MISMATCH/);
  const wrongRequest = new TauriClient(async () => slotReceipt("old-request"));
  await assert.rejects(wrongRequest.sceneTransition("exit", 4), /E_SCENE_RECEIPT_REQUEST_MISMATCH/);
  let fail;
  const delayed = new TauriClient(() => new Promise((_, reject) => { fail = reject; }));
  const pending = delayed.sceneTransition("exit", 4);
  fail(new Error("route IPC timeout"));
  await assert.rejects(pending, /route IPC timeout/);
});

test("environment control keeps source epoch across delayed scene change and validates atomic receipt identity", async () => {
  const { dispatchInteractable, isCurrentSceneInteractionResult } = await import("../dist/game/SceneInteraction.js");
  let resolveCommand; const calls=[];
  const client=new TauriClient((command,args)=>{calls.push({command,args});return new Promise(resolve=>{resolveCommand=resolve;});});
  const source={...snapshot(),worldId:"clockworks",sceneId:"cw_boiler_chamber",worldEpoch:4};
  const control={entityId:"cw_coolant_valve_staged",kind:"environment_control",active:true,transform:{positionM:{xM:18,yM:0,zM:8},yawRad:0}};
  const pending=dispatchInteractable(client,control,source.worldEpoch,source);
  const current={...source,sceneId:"cw_furnace_heart",worldEpoch:6};
  assert.equal(calls.length,1);assert.equal(calls[0].command,"formal_environment_control");
  assert.deepEqual(Object.keys(calls[0].args).sort(),["id","requestId","worldEpoch"]);
  assert.equal(calls[0].args.worldEpoch,4);assert.equal(calls[0].args.id,control.entityId);
  resolveCommand({commandId:calls[0].args.requestId,applied:false,alreadyApplied:false,errorCode:"E_SCENE_RUNTIME_StaleEpoch",
    worldEpoch:6,serverTick:current.serverTick,authorityRevision:current.authorityRevision,snapshot:current});
  const result=await pending;assert.equal(result.applied,false);
  assert.equal(isCurrentSceneInteractionResult(true,current,source,result),false);
  assert.equal(calls.length,1,"no follow-up snapshot or rebound mutation");
  const wrong=new TauriClient(async()=>({...slotReceipt(),commandId:"different-request"}));
  await assert.rejects(wrong.environmentControl(control.entityId,4),/E_SCENE_RECEIPT_REQUEST_MISMATCH/);
  const mismatched=new TauriClient(async(_command,args)=>({...slotReceipt(),commandId:args.requestId,authorityRevision:999}));
  await assert.rejects(mismatched.environmentControl(control.entityId,4),/E_SCENE_RECEIPT_SNAPSHOT_MISMATCH/);
});

test("epoch-bound ordinary interaction uses its atomic command receipt and never refetches a later world", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => {
    calls.push({ command, args });
    assert.equal(command, "formal_interact");
    const view = snapshot();
    return { applied: true, alreadyApplied: false, errorCode: null,
      receipt: { commandId: args.requestId, applied: true, alreadyApplied: false, errorCode: null,
        worldEpoch: view.worldEpoch, serverTick: view.serverTick, authorityRevision: view.authorityRevision, snapshot: view } };
  });
  const result = await client.interact("cw_master_shutdown_staged", snapshot().worldEpoch);
  assert.equal(result.applied, true);
  assert.equal(calls.length, 1);
  assert.deepEqual(Object.keys(calls[0].args).sort(), ["actorId", "requestId", "worldEpoch"]);
  assert.equal(calls[0].args.worldEpoch, snapshot().worldEpoch);
  assert.equal(result.snapshot.worldEpoch, snapshot().worldEpoch);
});

test("bound ordinary receipts reject wrong request, mixed metadata, missing receipt, and later epoch", async () => {
  for (const invalid of ["request", "metadata", "outcome", "missing", "later-epoch"]) {
    const client = new TauriClient(async (command, args) => {
      assert.equal(command, "formal_interact");
      const view = snapshot();
      if (invalid === "later-epoch") view.worldEpoch++;
      const receipt = { commandId: args.requestId, applied: true, alreadyApplied: false, errorCode: null,
        worldEpoch: view.worldEpoch, serverTick: view.serverTick, authorityRevision: view.authorityRevision, snapshot: view };
      if (invalid === "request") receipt.commandId = "old-request";
      if (invalid === "metadata") receipt.authorityRevision++;
      return invalid === "missing" ? { applied: true, errorCode: null }
        : { applied: invalid !== "outcome", alreadyApplied: false, errorCode: null, receipt };
    });
    await assert.rejects(client.interact("cw_pressure_valve_01_staged", snapshot().worldEpoch), /E_INTERACTION_RECEIPT_/);
  }
  let calls = 0;
  const client = new TauriClient(async () => { calls++; });
  for (const epoch of [-1, 0, Number.NaN, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    await assert.rejects(client.interact("cw_master_shutdown_staged", epoch), /E_INTERACTION_ARGUMENT/);
  }
  assert.equal(calls, 0);
});

test("native scene dispatcher retains the source epoch on a delayed same-scene ordinary command", async () => {
  const { dispatchInteractable } = await import("../dist/game/SceneInteraction.js");
  let resolve, args;
  const client = new TauriClient((command, supplied) => {
    assert.equal(command, "formal_interact"); args = supplied;
    return new Promise(done => { resolve = done; });
  });
  const source = { ...snapshot(), worldId: "clockworks", sceneId: "cw_shutdown_exit" };
  const pending = dispatchInteractable(client, { entityId: "cw_master_shutdown_staged", kind: "terminal", active: true }, source.worldEpoch, source);
  assert.equal(args.worldEpoch, source.worldEpoch);
  // A later Continue returns the same scene with a new owner epoch.
  const later = { ...source, worldEpoch: source.worldEpoch + 1 };
  resolve({ applied: false, alreadyApplied: false, errorCode: "E_SCENE_RUNTIME_StaleEpoch",
    receipt: { commandId: args.requestId, applied: false, alreadyApplied: false, errorCode: "E_SCENE_RUNTIME_StaleEpoch",
      worldEpoch: later.worldEpoch, serverTick: later.serverTick, authorityRevision: later.authorityRevision, snapshot: later } });
  await assert.rejects(pending, /E_INTERACTION_RECEIPT_STALE_EPOCH/);
});

test("save availability probe preserves omitted/false mode and supports default-only mode", async () => {
  const calls = [];
  const client = new TauriClient(async (command, args) => { calls.push({ command, args }); return args?.defaultOnly !== true; });
  assert.equal(await client.hasSave(), true);
  assert.equal(await client.hasSave(false), true);
  assert.equal(await client.hasSave(true), false);
  assert.deepEqual(calls, [
    { command: "formal_has_save", args: undefined },
    { command: "formal_has_save", args: { defaultOnly: false } },
    { command: "formal_has_save", args: { defaultOnly: true } },
  ]);
  for (const value of [null, 1, "true", {}, []]) await assert.rejects(client.hasSave(value), /E_SAVE_PROBE_MODE_INVALID/);
  assert.equal(calls.length, 3);
});

test("save availability rejects non-boolean native replies and propagates read failures", async () => {
  for (const value of [null, undefined, 0, 1, "false", {}, []]) {
    await assert.rejects(new TauriClient(async () => value).hasSave(true), /E_SAVE_PROBE_RESULT_INVALID/);
  }
  await assert.rejects(new TauriClient(async () => { throw Error("native read failed"); }).hasSave(true), /native read failed/);
});
