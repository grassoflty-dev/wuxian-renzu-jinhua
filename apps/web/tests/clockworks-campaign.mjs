import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import {
  CLOCKWORKS_EPILOGUE, CLOCKWORKS_RETURN_LINE, clockworksCompleted, clockworksStatus,
  confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks,
} from "../dist/game/ClockworksCampaign.js";
import { deriveCorePanel } from "../dist/ui/CorePanelModel.js";
import { missionTerminalSummary } from "../dist/game/MissionTerminal.js";
import { dispatchInteractable } from "../dist/game/SceneInteraction.js";

const EVENTS = ["clockworks_valves", "clockworks_core", "clockworks_shutdown"];
const capability = { capabilityId: "mobility.air_step_i", granted: true, selected: true };
const row = (complete = false) => ({ worldId: "clockworks", completed: complete, firstCompletion: complete,
  completedEvents: complete ? [...EVENTS] : EVENTS.slice(0, 2), visitId: 1, revisitCount: 0 });
function snapshot(complete = false, overrides = {}) {
  return { kind: "full", protocolVersion: 3, worldId: "clockworks", sceneId: "cw_shutdown_exit",
    worldEpoch: 31, serverTick: 700, authorityRevision: complete ? 91 : 90,
    player: { currentHp: 17, maxHp: 100 }, objectives: [], interactables: [],
    capabilities: { items: complete ? [capability] : [] },
    progression: { currentWorldId: "clockworks", eventSeq: complete ? 12 : 10, worlds: [row(complete)] },
    ...overrides };
}
const receipt = (after = snapshot(true), overrides = {}) => ({ applied: true, alreadyApplied: false, errorCode: null, snapshot: after, ...overrides });
const epilogue = (before = snapshot(), result = receipt(), id = "cw_master_shutdown_staged", kind = "terminal") =>
  confirmedClockworksEpilogue(before, id, kind, result);
const network = view => deriveCorePanel("world_network", view).rows;

test("exact epilogue appears only on first accepted real shutdown receipt, including legacy confirmation", () => {
  assert.equal(epilogue(), "原来门一直不止三扇。");
  assert.equal(CLOCKWORKS_EPILOGUE, "原来门一直不止三扇。");
  assert.equal(epilogue(snapshot(true)), null, "loading already-completed state is not a first shutdown");
  const legacy = snapshot(false, { capabilities: { items: [capability] } });
  legacy.progression.worlds[0].completedEvents = [...EVENTS];
  const after = snapshot(true); after.progression.eventSeq = 11;
  assert.equal(epilogue(legacy, receipt(after)), CLOCKWORKS_EPILOGUE);
  assert.equal(epilogue(legacy, receipt()), null, "legacy reconfirm does not replay Progress");
  const missing = snapshot(); missing.progression.worlds[0].completedEvents = [];
  assert.equal(epilogue(missing), null);
});

test("stale mixed invalid rejected duplicate dead and loading receipts cannot show epilogue", () => {
  for (const overrides of [{ applied: false }, { alreadyApplied: true }, { errorCode: "E_REJECT" }]) {
    assert.equal(epilogue(snapshot(), receipt(snapshot(true), overrides)), null);
  }
  for (const overrides of [
    { worldEpoch: 32 }, { worldEpoch: 30 }, { serverTick: 699 }, { authorityRevision: 90 },
    { authorityRevision: 89 }, { sceneId: "cw_regulator_core" }, { worldId: "return_station" },
    { entryToken: { generation: 1 } }, { protocolVersion: 2 }, { kind: "delta" },
    { player: { currentHp: 0 } }, { worldEpoch: Number.NaN },
  ]) assert.equal(epilogue(snapshot(), receipt(snapshot(true, overrides))), null, JSON.stringify(overrides));
  assert.equal(epilogue(snapshot(false, { player: { currentHp: 0 } })), null);
  assert.equal(epilogue(snapshot(), receipt(), "cw_air_step_grant_staged"), null);
  assert.equal(epilogue(snapshot(), receipt(), "cw_master_shutdown_staged", "scene_trigger"), null);
});

test("completion rejects malformed duplicate or incomplete authority and never reveals unknown nodes early", () => {
  assert.equal(clockworksCompleted(snapshot(true)), true);
  assert.equal(network(snapshot(true)).filter(item => item.label === "???").length, 3);
  assert.equal(network(snapshot()).some(item => item.label === "???"), false);
  for (const change of [
    view => { view.progression.worlds.push(row(true)); },
    view => { view.progression.worlds[0].completed = "true"; },
    view => { view.progression.worlds[0].firstCompletion = false; },
    view => { view.progression.worlds[0].completedEvents.pop(); },
    view => { view.progression.worlds[0].completedEvents.push(EVENTS[0]); },
    view => { view.progression.worlds[0].revisitCount = 5; },
    view => { view.progression.worlds[0].visitId = 0; },
    view => { view.capabilities.items = []; },
    view => { view.capabilities.items = [capability, capability]; },
    view => { view.capabilities.items = [{ ...capability, granted: "true" }]; },
  ]) {
    const view = snapshot(true); change(view);
    assert.equal(clockworksCompleted(view), false);
    assert.equal(network(view).some(item => item.label === "???"), false);
    assert.equal(epilogue(snapshot(), receipt(view)), null);
  }
});

test("CW status follows live gate projection and exact four additional revisit policy", () => {
  const view = snapshot(false, { worldId: "return_station", sceneId: "rs_core_room" });
  assert.equal(clockworksStatus(view), "雾港实际撤离后解锁");
  view.interactables = [{ entityId: "rs_cw_world_gate_marker", kind: "world_gate", active: true }];
  assert.equal(clockworksStatus(view), "归航站入口可用");
  view.interactables[0].active = false;
  assert.equal(clockworksStatus(view), "入口当前不可用");
  const completed = snapshot(true, { worldId: "return_station", sceneId: "rs_core_room" });
  assert.equal(clockworksStatus(completed), "已完成 · 前往归航站查询复访", "absent gate does not mean exhausted");
  completed.progression.worlds[0].visitId = 5;
  completed.progression.worlds[0].revisitCount = 4;
  assert.equal(clockworksStatus(completed), "已完成 · 复访次数用尽");
  assert.match(missionTerminalSummary(completed), /钟骨工厂：已完成 · 复访次数用尽/);
  assert.ok(!network(completed).some(item => /3\s*\/\s*3/.test(item.value)));
});

test("first completed CW return requires unchanged settlement and fresh exact destination", () => {
  const before = snapshot(true);
  const after = snapshot(true, { worldId: "return_station", sceneId: "rs_core_room", worldEpoch: 32, authorityRevision: 92 });
  const line = (prior = before, result = receipt(after)) => confirmedReturnStationAfterClockworks(prior, "cw_shutdown_return_to_rs", "world_gate", result);
  assert.equal(line(), CLOCKWORKS_RETURN_LINE);
  for (const overrides of [{ worldEpoch: 31 }, { serverTick: 699 }, { sceneId: "gh_exit" }, { player: { currentHp: 0 } }]) {
    assert.equal(line(before, receipt({ ...after, ...overrides })), null);
  }
  const revisit = structuredClone(before); revisit.progression.worlds[0].visitId = 2; revisit.progression.worlds[0].revisitCount = 1;
  assert.equal(line(revisit), null);
  assert.equal(line(snapshot()), null);
  assert.equal(line(before, receipt(after, { alreadyApplied: true })), null);
});

test("CW world gates use fixed commands and reject wrong source contexts before transport", async () => {
  const calls = [];
  const client = { worldGate: async (...args) => { calls.push(args); return receipt(); } };
  const entry = { entityId: "rs_cw_world_gate_marker", kind: "world_gate", active: true };
  const exit = { entityId: "cw_shutdown_return_to_rs", kind: "world_gate", active: true };
  await dispatchInteractable(client, entry, 18, { worldId: "return_station", sceneId: "rs_core_room" });
  await dispatchInteractable(client, exit, 29, { worldId: "clockworks", sceneId: "cw_shutdown_exit" });
  assert.deepEqual(calls, [["rs_world_gate_to_cw", 18], ["cw_shutdown_return_to_rs", 29]]);
  for (const item of [entry, exit]) {
    await assert.rejects(dispatchInteractable(client, item, 31, { worldId: "grey_hive", sceneId: "gh_exit" }), /SOURCE_MISMATCH/);
    await assert.rejects(dispatchInteractable(client, { ...item, active: false }, 31), /INACTIVE/);
  }
  assert.equal(calls.length, 2);
});

test("main narrative runs only after current live session accepted the authoritative receipt", async () => {
  const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const accepted = source.indexOf("await activeLoop.acceptAuthoritativeSnapshot(result.snapshot)");
  const ownerFence = source.indexOf("if (sessionLoop !== activeLoop || !activeLoop.acceptsExternalResults) return;", accepted);
  const narrative = source.indexOf("confirmedClockworksEpilogue(snapshot", ownerFence);
  assert.ok(accepted >= 0 && ownerFence > accepted && narrative > ownerFence);
});
