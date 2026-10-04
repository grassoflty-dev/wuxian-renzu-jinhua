import assert from "node:assert/strict";
import { readFile, mkdtemp, mkdir, writeFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { stripTypeScriptTypes } from "node:module";
import { after, test } from "node:test";
import vm from "node:vm";
import { createMockBridge, mockBootstrapScript } from "../src/mock-ipc.mjs";

// Keep the production ES module graph and relative imports intact, without
// producing tsc files in apps/web/dist or replacing the production bundle.
const moduleRoot = await mkdtemp(path.join(os.tmpdir(), "current-web-protocol-"));
after(() => rm(moduleRoot, { recursive: true, force: true }));
await writeFile(path.join(moduleRoot, "package.json"), '{"type":"module"}');
for (const relative of ["protocol/BuildProjection", "protocol/SupportProjection", "protocol/SentinelEncounter", "game/GreyHiveBeacon", "protocol/types", "bridge/tauri-client"]) {
  const source = await readFile(new URL(`../../../apps/web/src/${relative}.ts`, import.meta.url), "utf8");
  const output = path.join(moduleRoot, `${relative}.js`);
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, stripTypeScriptTypes(source, { mode: "transform" }));
}
const protocol = await import(pathToFileURL(path.join(moduleRoot, "protocol/types.js")).href);
const { TauriClient } = await import(pathToFileURL(path.join(moduleRoot, "bridge/tauri-client.js")).href);

test("fixture is flattened v3 and the production bridge reads New Journey, pause and Continue", async () => {
  const mock = createMockBridge(); const client = new TauriClient(mock.bridge.invoke);
  const boot = await client.snapshot();
  assert.equal(protocol.assertSnapshotV3(boot), boot); assert.equal(boot.sceneId, "rs_core_room");
  assert.equal("view" in boot, false); assert.equal(mock.nativeGameplayAcceptance, false);
  const start = await client.newJourney(); assert.equal(start.worldId, "return_station");
  assert.ok(start.worldEpoch > boot.worldEpoch);
  const paused = await client.pause(); assert.equal(mock.state.paused, true);
  const save = await client.saveSlot("new-slot", "mock only", true);
  assert.equal(save.snapshot.worldEpoch, paused.worldEpoch);
  await client.resume(); await client.returnToHub();
  const restored = await client.continueSlot("new-slot");
  assert.equal(restored.snapshot.sceneId, "rs_core_room");
  assert.ok(restored.snapshot.worldEpoch > start.worldEpoch);
  assert.equal((await client.listSaveSlots()).length, 4);
});

test("production receipt guards reject wrong counters and old envelope", async () => {
  const mock = createMockBridge();
  const client = new TauriClient(async (command, args) => {
    const result = await mock.bridge.invoke(command, args);
    if (command === "formal_pause") result.worldEpoch++;
    if (command === "formal_continue_slot") result.snapshot = { kind: "full", protocolVersion: 2, view: result.snapshot };
    return result;
  });
  await assert.rejects(client.pause(), /E_LIFECYCLE_RECEIPT_SNAPSHOT_MISMATCH/);
  await assert.rejects(client.continueSlot("valid-slot"), /E_SNAPSHOT_PROTOCOL/);
});

test("mock refuses invalid saves, corrupt Continue, unknown gameplay and input while paused", async () => {
  const mock = createMockBridge(); const client = new TauriClient(mock.bridge.invoke);
  await assert.rejects(client.saveSlot("x", "x", true), /E_MOCK_SAVE_REQUIRES_PAUSE/);
  await client.pause();
  await assert.rejects(client.saveSlot("legacy-slot", "x", false), /E_MOCK_INVALID_SAVE_TARGET/);
  await assert.rejects(client.continueSlot("corrupt-slot"), /E_MOCK_INVALID_CONTINUE_TARGET/);
  await assert.rejects(mock.bridge.invoke("formal_world_gate"), /E_CURRENT_WEB_UNMAPPED_IPC/);
  await assert.rejects(mock.bridge.invoke("formal_submit_input", { sample: { worldEpoch: mock.state.epoch } }), /E_MOCK_INPUT_WHILE_PAUSED/);
  assert.deepEqual(mock.state.unexpected, ["formal_world_gate"]);
});

test("deferred and failed IPC supports bounded UI race checks", async () => {
  const mock = createMockBridge(); const client = new TauriClient(mock.bridge.invoke);
  mock.block("formal_new"); let finished = false;
  const request = client.newJourney().then(() => { finished = true; });
  await Promise.resolve(); assert.equal(finished, false); assert.equal(mock.state.epoch, 1);
  mock.release("formal_new"); await request; assert.equal(finished, true);
  mock.failNext("formal_continue_slot", "E_TEST_INTERRUPTED");
  await assert.rejects(client.continueSlot("valid-slot"), /E_TEST_INTERRUPTED/);
  assert.equal((await client.continueSlot("valid-slot")).snapshot.sceneId, "gh_entry_maintenance");
});

test("browser bootstrap installs only the v2 Tauri internals API before production boot", async () => {
  const context = vm.createContext({ window: {}, structuredClone });
  vm.runInContext(mockBootstrapScript({ withSlots: false }), context);
  assert.equal(context.window.isTauri, true); assert.equal(context.window.__TAURI__, undefined);
  assert.equal((await context.window.__TAURI_INTERNALS__.invoke("formal_snapshot")).protocolVersion, 3);
  assert.equal((await context.window.__TAURI_INTERNALS__.invoke("formal_list_save_slots")).length, 0);
});

test("every default New Journey and Continue identity exists in the production compiled scene set", async () => {
  const mock = createMockBridge(); const client = new TauriClient(mock.bridge.invoke);
  const snapshots = [await client.snapshot(), await client.newJourney()];
  for (const summary of await client.listSaveSlots()) {
    if (summary.valid) snapshots.push((await client.continueSlot(summary.slotId)).snapshot);
  }
  for (const snapshot of snapshots) {
    const scene = JSON.parse(await readFile(new URL(`../../../content/scenes/compiled/${snapshot.sceneId}.json`, import.meta.url), "utf8"));
    assert.equal(scene.worldId, snapshot.worldId);
    assert.equal(scene.sceneId, snapshot.sceneId);
    const spawn = scene.spawns.find(spawn => spawn.kind === "player");
    assert.ok(spawn, "fixture scene must have an authored player entry");
    const position = snapshot.player.transform.positionM;
    assert.deepEqual([position.xM, position.yM, position.zM], spawn.position);
  }
});

test("primitive counts change only on actual bridge invocation, never setup, waits or observers", async () => {
  const mock = createMockBridge();
  assert.equal(mock.count("formal_submit_input"), 0);
  mock.block("formal_new"); mock.failNext("formal_new", "expected rejection");
  mock.release("formal_submit_input"); mock.captureEvidence();
  await new Promise(resolve => setTimeout(resolve, 5));
  assert.equal(mock.count("formal_new"), 0); assert.equal(mock.count("formal_submit_input"), 0);
  const pending = mock.bridge.invoke("formal_new");
  assert.equal(mock.count("formal_new"), 1);
  mock.release("formal_new"); await assert.rejects(pending, /expected rejection/);
  assert.equal(mock.count("formal_new"), 1); assert.equal(mock.count("formal_submit_input"), 0);
  await mock.bridge.invoke("formal_submit_input", { sample: { worldEpoch: 1, seq: 1 } });
  assert.equal(mock.count("formal_submit_input"), 1);
  for (let i = 0; i < 100; i++) { mock.count("formal_submit_input"); mock.captureEvidence(); }
  assert.equal(mock.count("formal_submit_input"), 1);
});

test("failure evidence keeps bounded examples and exact counters including freshness across New/Continue", async () => {
  const mock = createMockBridge();
  for (let seq = 1; seq <= 1500; seq++) await mock.bridge.invoke("formal_submit_input", { sample: { worldEpoch: 1, seq } });
  const before = mock.count("formal_submit_input");
  await mock.bridge.invoke("formal_new");
  assert.equal(mock.count("formal_submit_input"), before, "New alone cannot satisfy fresh-input readiness");
  await mock.bridge.invoke("formal_continue_slot", { slotId: "valid-slot" });
  assert.equal(mock.count("formal_submit_input"), before, "Continue alone cannot satisfy fresh-input readiness");
  await mock.bridge.invoke("formal_submit_input", { sample: { worldEpoch: mock.state.epoch, seq: 1 } });
  const evidence = mock.captureEvidence();
  assert.equal(evidence.observations.formal_submit_input.count, before + 1);
  assert.equal(evidence.observations.formal_new.count, 1); assert.equal(evidence.observations.formal_continue_slot.count, 1);
  assert.equal(evidence.totalCalls, 1503); assert.ok(evidence.calls.length <= 107);
  assert.equal(evidence.callLedgerSampled, true);
  assert.equal(mock.state.calls.length, 512); assert.equal(evidence.ledgerDropped, 991);
  assert.ok(evidence.observations.formal_submit_input.lastAtMs >= evidence.observations.formal_submit_input.firstAtMs);
  assert.equal(evidence.unexpectedCount, 0);
});


test("bounded samples retain control arguments and never hide an unmapped-command count", async () => {
  const mock = createMockBridge();
  await mock.bridge.invoke("formal_continue_slot", { slotId: "valid-slot" });
  for (let seq = 1; seq <= 600; seq++) await mock.bridge.invoke("formal_submit_input", { sample: { worldEpoch: mock.state.epoch, seq } });
  assert.equal(mock.callsFor("formal_continue_slot")[0].args.slotId, "valid-slot");
  for (let i = 0; i < 80; i++) await assert.rejects(mock.bridge.invoke("unknown_command"), /UNMAPPED_IPC/);
  const evidence = mock.captureEvidence();
  assert.equal(evidence.unexpected.length, 64); assert.equal(evidence.unexpectedCount, 80);
  assert.equal(evidence.observations.formal_submit_input.count, 600);
  assert.ok(evidence.calls.length <= 107);
});


test("one chatty command cannot evict the arguments of a later Continue or Save", async () => {
  const mock = createMockBridge();
  for (let i = 0; i < 64; i++) await mock.bridge.invoke("formal_snapshot");
  await mock.bridge.invoke("formal_continue_slot", { slotId: "valid-slot" });
  for (let i = 0; i < 64; i++) await mock.bridge.invoke("formal_list_save_slots");
  assert.equal(mock.count("formal_snapshot"), 64);
  assert.equal(mock.callsFor("formal_snapshot").length, 8);
  assert.equal(mock.count("formal_continue_slot"), 1);
  assert.equal(mock.callsFor("formal_continue_slot")[0].args.slotId, "valid-slot");
  await mock.bridge.invoke("formal_pause");
  await mock.bridge.invoke("formal_save_slot", { slotId: "late-save", displayName: "late", create: true });
  assert.equal(mock.callsFor("formal_save_slot")[0].args.slotId, "late-save");
  assert.ok(mock.captureEvidence().calls.length <= 107);
});


test("production protocol follows its BuildProjection dependency and rejects forged projections", async () => {
  const mock = createMockBridge();
  const snapshot = await mock.bridge.invoke("formal_snapshot");
  snapshot.build = { schemaVersion: 1, revision: 0, items: [], equipment: [] };
  assert.equal(protocol.assertSnapshotV3(snapshot), snapshot);
  assert.throws(() => protocol.assertSnapshotV3({ ...snapshot, build: { ...snapshot.build, grant: "invented" } }), /E_BUILD_PROJECTION/);
  assert.throws(() => protocol.assertSnapshotV3({ ...snapshot, build: null }), /E_BUILD_PROJECTION/);
  const client = new TauriClient(mock.bridge.invoke);
  await assert.rejects(client.buildCommand({ kind: "equip", itemId: "", expectedItemId: null }, snapshot), /E_BUILD_ACTION/);
  assert.equal(mock.count("formal_build_command"), 0, "the actual client dependency rejects invalid actions before IPC");
});
