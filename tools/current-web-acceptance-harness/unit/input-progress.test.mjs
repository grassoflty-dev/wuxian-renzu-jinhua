import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";
import { readFileSync } from "node:fs";
import { createMockBridge, mockBootstrapScript } from "../src/mock-ipc.mjs";
import { createInputProgressPublisher, createInputProgressStore, INPUT_PROGRESS_BINDING } from "../src/input-progress.mjs";
const streamId = "test-input-stream-0001";
const documentId = "test-document-00000001";
const checkpoint = overrides => ({ documentId, worldEpoch: 1, inputCount: 0, ...overrides });
const armedStore = () => { const store = createInputProgressStore(streamId); store.arm(checkpoint()); return store; };
const flush = () => new Promise(resolve => setImmediate(resolve));
const sample = (mock, seq = 1, overrides = {}) => ({ protocol: "continuous-input", protocolVersion: 2,
  worldEpoch: mock.state.epoch, seq, clientTimeMs: seq * 50, moveX: 0, moveZ: 0, aimX: 0, aimZ: 1, ...overrides });
const record = overrides => ({ streamId, documentId, worldEpoch: 2, ackSeq: 1, inputCount: 1, serverTick: 3, authorityRevision: 3, ...overrides });

test("notifications come only from successful actual validated input and match its captured receipt", async () => {
  const records = [];
  const mock = createMockBridge({}, value => records.push(value), documentId);
  const checkpoint = mock.inputCheckpoint();
  assert.equal(mock.count("formal_submit_input"), 0);
  await mock.bridge.invoke("formal_snapshot");
  await mock.bridge.invoke("formal_new");
  assert.equal(records.length, 0);
  const receipt = await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  assert.equal(records.length, 1);
  assert.ok(Object.isFrozen(records[0]));
  assert.deepEqual(records[0], { documentId, worldEpoch: receipt.worldEpoch, ackSeq: receipt.snapshot.ackSeq,
    inputCount: 1, serverTick: receipt.serverTick, authorityRevision: receipt.authorityRevision });
  assert.equal(receipt.snapshot.worldEpoch, records[0].worldEpoch);
  const store = armedStore();
  store.accept({ ...records[0], streamId });
  assert.equal(store.countAfter(checkpoint), 1);
  const captured = { ...records[0] };
  await mock.bridge.invoke("formal_return");
  assert.deepEqual(records[0], captured, "later epoch mutation cannot rewrite a captured input result");
  assert.equal(mock.inputCheckpoint().inputCount, 1);
});

test("setup, control reads, blocked, failed, paused and stale-epoch calls cannot manufacture progress", async () => {
  const records = [];
  const mock = createMockBridge({}, value => records.push(value), documentId);
  mock.count("formal_submit_input"); mock.inputCheckpoint(); mock.captureEvidence();
  mock.block("formal_submit_input");
  const blocked = mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  await flush(); assert.equal(records.length, 0); assert.equal(mock.count("formal_submit_input"), 1);
  mock.failNext("formal_submit_input"); mock.release("formal_submit_input");
  await assert.rejects(blocked, /E_MOCK_REQUEST_REJECTED/); assert.equal(records.length, 0);
  await mock.bridge.invoke("formal_pause");
  await assert.rejects(mock.bridge.invoke("formal_submit_input", { sample: sample(mock) }), /WHILE_PAUSED/);
  await mock.bridge.invoke("formal_resume");
  await assert.rejects(mock.bridge.invoke("formal_submit_input", { sample: sample(mock, 1, { worldEpoch: 999 }) }), /E_MOCK_INPUT_EPOCH/);
  assert.equal(records.length, 0);
});

test("malformed protocol, sequence, clock, axes and nonfresh inputs never qualify for notification", async () => {
  for (const invalid of [{ protocol: "other" }, { protocolVersion: 1 }, { seq: 0 }, { seq: NaN }, { seq: 1.5 },
    { clientTimeMs: -1 }, { clientTimeMs: Infinity }, { moveX: 2 }, { moveZ: NaN }, { aimX: Infinity }, { aimZ: undefined }]) {
    const records = []; const mock = createMockBridge({}, value => records.push(value), documentId);
    await mock.bridge.invoke("formal_submit_input", { sample: sample(mock, 1, invalid) });
    assert.equal(records.length, 0, JSON.stringify(invalid));
  }
  const records = []; const mock = createMockBridge({}, value => records.push(value), documentId);
  await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  await mock.bridge.invoke("formal_submit_input", { sample: sample(mock, 2, { clientTimeMs: 1 }) });
  assert.equal(records.length, 1);
  assert.equal(mock.count("formal_submit_input"), 3, "observation does not rewrite the original invocation counter");
});

test("Node store rejects foreign, malformed, duplicate and regressing records without dropping epoch freshness", () => {
  const store = armedStore();
  for (const invalid of [null, [], {}, record({ streamId: "foreign-stream-000" }), record({ extra: true }),
    record({ ackSeq: 0 }), record({ inputCount: -1 }), record({ serverTick: Infinity }), record({ authorityRevision: 1 })]) {
    assert.equal(store.accept(invalid), false);
  }
  assert.equal(store.accept(record()), true);
  assert.equal(store.accept(record()), false);
  assert.equal(store.accept(record({ inputCount: 2, ackSeq: 2, serverTick: 4, authorityRevision: 4 })), true);
  assert.equal(store.accept(record({ inputCount: 3, ackSeq: 1, serverTick: 5, authorityRevision: 5 })), false);
  assert.equal(store.countAfter({ documentId, worldEpoch: 2, inputCount: 2 }), 0, "same-epoch input cannot count as New/Continue readiness");
  assert.equal(store.accept(record({ worldEpoch: 4, inputCount: 3, ackSeq: 1, serverTick: 1, authorityRevision: 1 })), true);
  assert.equal(store.countAfter({ documentId, worldEpoch: 3, inputCount: 2 }), 3);
  assert.equal(store.accept(record({ worldEpoch: 2, inputCount: 99, ackSeq: 99, serverTick: 99, authorityRevision: 99 })), false);
  assert.equal(store.countAfter({ documentId, worldEpoch: 4, inputCount: 3 }), 0);
  assert.throws(() => store.countAfter({ documentId, worldEpoch: 0, inputCount: 0 }), /CHECKPOINT/);
  store.dispose(); assert.equal(store.accept(record({ worldEpoch: 5, inputCount: 100 })), false);
  assert.equal(store.diagnostics().latest, null);
  assert.equal(store.countAfter({ documentId, worldEpoch: 3, inputCount: 2 }), 0);
});

test("coalescing is bounded to one in flight plus one latest immutable record", async () => {
  const delivered = []; let release;
  const publisher = createInputProgressPublisher(value => {
    delivered.push(value); return new Promise(resolve => { release = resolve; });
  });
  const first = record(); publisher.publish(first); first.ackSeq = 99;
  await flush(); assert.equal(delivered[0].ackSeq, 1);
  for (let i = 2; i <= 1000; i++) publisher.publish(record({ inputCount: i, ackSeq: i }));
  assert.deepEqual([publisher.diagnostics().inFlight, publisher.diagnostics().pending, publisher.diagnostics().coalesced], [1, 1, 998]);
  release(); await flush();
  assert.equal(delivered.length, 2); assert.equal(delivered[1].inputCount, 1000);
  publisher.publish(record({ worldEpoch: 3, inputCount: 1001 }));
  publisher.dispose(); release(); await flush();
  assert.equal(delivered.length, 2); assert.equal(publisher.diagnostics().pending, 0);
  publisher.publish(record({ worldEpoch: 4, inputCount: 1002 }));
  await flush(); assert.equal(delivered.length, 2);
});

test("pending failures are handled and disposal prevents scheduled callbacks", async () => {
  const publisher = createInputProgressPublisher(async () => { throw Error("transport failed"); });
  publisher.publish(record()); await flush();
  assert.equal(publisher.diagnostics().failures, 1);
  assert.equal(publisher.diagnostics().inFlight, 0);
  let calls = 0; const disposed = createInputProgressPublisher(() => { calls++; });
  disposed.publish(record()); disposed.dispose(); await flush();
  assert.equal(calls, 0);
});

test("serialized bootstrap emits only actual input, exposes no publish control, and disposes safely", async () => {
  const records = [];
  const context = { window: { crypto: { randomUUID: () => documentId }, [INPUT_PROGRESS_BINDING]: value => { records.push(value); return Promise.resolve(); } }, structuredClone };
  vm.runInNewContext(mockBootstrapScript({}, streamId), context);
  const mock = context.window.__CURRENT_WEB_MOCK__;
  assert.deepEqual(Object.keys(context.window.__CURRENT_WEB_INPUT_OBSERVER__).sort(), ["diagnostics", "dispose"]);
  const checkpoint = mock.inputCheckpoint();
  await mock.bridge.invoke("formal_new");
  await flush(); assert.equal(records.length, 0);
  await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  await flush(); assert.equal(records.length, 1);
  const store = armedStore(); assert.equal(store.accept(records[0]), true);
  assert.equal(store.countAfter(checkpoint), 1);
  context.window.__CURRENT_WEB_INPUT_OBSERVER__.dispose();
  await mock.bridge.invoke("formal_new");
  await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  await flush(); assert.equal(records.length, 1);
  assert.equal(mock.count("formal_submit_input"), 2, "disposing telemetry cannot stop actual input");
});

test("readiness reads stay off the busy-page transport while negative counts and budgets stay authoritative", () => {
  const source = readFileSync(new URL("../tests/helpers.mjs", import.meta.url), "utf8");
  const positive = source.split("export function observedInputCount")[1].split("export async function enterNew")[0];
  assert.doesNotMatch(positive, /page\.evaluate|send\(|invoke\(/);
  assert.match(positive, /countAfter\(checkpoint\)/);
  assert.match(source, /source.page === page && source.frame === page.mainFrame\(\)/);
  assert.match(source, /finally \{ page.off\("close", dispose\); dispose\(\); \}/);
  assert.match(source, /callCount\(page, command\)[\s\S]*?page.evaluate\(command => window.__CURRENT_WEB_MOCK__.count\(command\)/);
  const publisher = createInputProgressPublisher.toString();
  assert.doesNotMatch(publisher, /formal_submit_input|requestAnimationFrame|setTimeout|setInterval|worldEpoch\s*[+]?=/);
});


test("old-epoch input cannot satisfy even a zero-threshold readiness assertion", () => {
  const store = armedStore();
  store.accept(record({ inputCount: 113, ackSeq: 113, serverTick: 114, authorityRevision: 114 }));
  assert.equal(store.countAfter({ documentId, worldEpoch: 2, inputCount: 100 }), 0);
  store.accept(record({ worldEpoch: 3, inputCount: 114, ackSeq: 1, serverTick: 1, authorityRevision: 1 }));
  assert.equal(store.countAfter({ documentId, worldEpoch: 2, inputCount: 113 }), 114);
});


test("reload on the same Page cannot reuse or revive a prior document generation", () => {
  const store = armedStore();
  store.accept(record({ worldEpoch: 4, inputCount: 114, ackSeq: 2 }));
  const nextDocument = "test-document-00000002";
  const fresh = store.arm(checkpoint({ documentId: nextDocument }));
  assert.equal(store.countAfter(fresh), 0);
  assert.equal(store.accept(record({ worldEpoch: 5, inputCount: 115, ackSeq: 3, serverTick: 4, authorityRevision: 4 })), false);
  assert.equal(store.countAfter(fresh), 0, "delayed old document cannot change the armed generation");
  assert.equal(store.accept(record({ documentId: nextDocument })), true);
  assert.equal(store.countAfter(fresh), 1);
  assert.equal(store.countAfter(checkpoint()), 0);
  assert.throws(() => store.arm(checkpoint({ documentId: nextDocument, worldEpoch: 1, inputCount: 0 })), /REGRESSION/);
});

test("only the first qualified input per epoch crosses telemetry; every invocation stays counted", async () => {
  const records = []; const mock = createMockBridge({}, value => records.push(value), documentId);
  for (let seq = 1; seq <= 1000; seq++) await mock.bridge.invoke("formal_submit_input", { sample: sample(mock, seq) });
  assert.equal(records.length, 1); assert.equal(mock.count("formal_submit_input"), 1000);
  await mock.bridge.invoke("formal_new");
  await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  assert.equal(records.length, 2); assert.equal(records[1].inputCount, 1001);
});

test("serialized boot gives every new document a different identity shared by checkpoint and record", async () => {
  const records = []; let generation = 0;
  const host = () => ({ window: { crypto: { randomUUID: () => `document-generation-${++generation}` },
    [INPUT_PROGRESS_BINDING]: value => { records.push(value); return Promise.resolve(); } }, structuredClone });
  const first = host(); const second = host();
  vm.runInNewContext(mockBootstrapScript({}, streamId), first);
  vm.runInNewContext(mockBootstrapScript({}, streamId), second);
  for (const context of [first, second]) {
    const mock = context.window.__CURRENT_WEB_MOCK__;
    await mock.bridge.invoke("formal_new");
    await mock.bridge.invoke("formal_submit_input", { sample: sample(mock) });
  }
  await flush();
  assert.equal(records.length, 2);
  assert.notEqual(records[0].documentId, records[1].documentId);
  assert.equal(records[0].documentId, first.window.__CURRENT_WEB_MOCK__.inputCheckpoint().documentId);
  assert.equal(records[1].documentId, second.window.__CURRENT_WEB_MOCK__.inputCheckpoint().documentId);
});
