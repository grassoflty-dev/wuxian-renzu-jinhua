import test from "node:test";
import assert from "node:assert/strict";
import { TauriClient } from "../dist/bridge/tauri-client.js";
import { SessionLoop } from "../dist/game/SessionLoop.js";

function snapshot(epoch = 1, tick = 1, sceneId = "gh_entry") {
  return { kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2", worldId: "grey_hive", sceneId,
    checkpointId: null, worldEpoch: epoch, serverTick: tick, authorityRevision: tick, ackSeq: tick,
    player: { entityId: "player", transform: { positionM: { xM: 0, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100, currentEnergy: 100, maxEnergy: 100,
      facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" }, actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1 }, progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] } };
}
function context(view = snapshot()) { return { worldId: view.worldId, sceneId: view.sceneId, worldEpoch: view.worldEpoch }; }
function receipt(view = snapshot(), commandId = "pause") {
  return { commandId, applied: true, alreadyApplied: false, errorCode: null, worldEpoch: view.worldEpoch,
    serverTick: view.serverTick, authorityRevision: view.authorityRevision, snapshot: view };
}
function entry(epoch = 1) { const view = snapshot(epoch, 1, `scene_${epoch}`); return { ...view, entryToken: { ...context(view), generation: epoch } }; }
function released(view) { const { entryToken, ...result } = view; return { ...result, authorityRevision: 2 }; }
function deferred() { let resolve, reject; const promise = new Promise((yes, no) => { resolve = yes; reject = no; }); return { promise, resolve, reject }; }
async function flush() { for (let i = 0; i < 50; i++) await Promise.resolve(); }
class Surface {
  hidden = false; focused = true; listeners = new Map(); hasFocus = () => this.focused;
  addEventListener(name, callback) { const set = this.listeners.get(name) ?? new Set(); set.add(callback); this.listeners.set(name, set); }
  removeEventListener(name, callback) { this.listeners.get(name)?.delete(callback); }
  fire(name) { for (const callback of this.listeners.get(name) ?? []) callback({}); }
}
function harness(overrides = {}) {
  const calls = [], statuses = [], views = [], errors = []; const intervals = new Map(), frames = new Map(); let next = 0, destroyed = 0, cleaned = 0;
  const windowTarget = new Surface(), documentTarget = new Surface(); let current = overrides.initial ?? snapshot(); let ready;
  const scheduler = { now: () => 0, clientTimeMs: () => 0, setInterval: cb => { const id = ++next; intervals.set(id, cb); return id; }, clearInterval: id => intervals.delete(id),
    requestAnimationFrame: cb => { const id = ++next; frames.set(id, cb); return id; }, cancelAnimationFrame: id => frames.delete(id) };
  const client = { events: async () => [], submitInput: async () => current, submitAction: async action => { calls.push(["action", action.kind]); return current; },
    pause: async ctx => { calls.push(["pause", ctx]); return current; }, resume: async ctx => { calls.push(["resume", ctx]); return current; },
    sceneReady: async token => { calls.push(["ready", token]); return released(entry(token.worldEpoch)); }, ...overrides.client };
  const renderer = { render: async view => { await overrides.render?.(view); ready = view; }, isReadyFor: view => ready?.sceneId === view.sceneId && ready?.worldEpoch === view.worldEpoch,
    cancel: () => overrides.cancel?.(), destroy: async () => { destroyed++; await overrides.destroy?.(); } };
  const loop = new SessionLoop(client, renderer, { scheduler, windowTarget, documentTarget,
    onPauseState: (state, message) => statuses.push([state, message]), onSnapshot: view => { current = view; views.push(view); },
    onError: error => errors.push(error), onCleanup: () => cleaned++, ...overrides.options });
  return { loop, calls, statuses, views, errors, intervals, frames, windowTarget, documentTarget, get destroyed() { return destroyed; }, get cleaned() { return cleaned; },
    tick() { for (const cb of intervals.values()) cb(); }, frame() { const first = frames.entries().next().value; if (first) { frames.delete(first[0]); first[1](16); } } };
}

test("lifecycle wire copies all context fields and supplies monotonic commandSequence", async () => {
  const calls = [], pending = deferred();
  const client = new TauriClient((command, args) => { calls.push([command, args]); return calls.length === 1 ? pending.promise : Promise.resolve(receipt(snapshot(2), "resume")); });
  const source = context(); const paused = client.pause(source); source.sceneId = "mutated"; source.worldEpoch = 999;
  assert.deepEqual(calls[0], ["formal_pause", { context: context(), commandSequence: 1 }]);
  pending.resolve(receipt()); await paused; await client.resume(context(snapshot(2)));
  assert.deepEqual(calls[1], ["formal_resume", { context: context(snapshot(2)), commandSequence: 2 }]);
  for (const invalid of [null, {}, { ...context(), worldEpoch: -1 }, { ...context(), sceneId: "" }, { ...context(), worldEpoch: 1.5 }]) {
    await assert.rejects(client.pause(invalid), /SESSION_CONTEXT_INVALID/);
  }
  assert.equal(calls.length, 2);
  await assert.rejects(new TauriClient(async () => receipt(snapshot(2))).pause(context()), /RECEIPT_CONTEXT_MISMATCH/);
});

test("pause and resume deadlines leave context uncertain; late resume only compensates its original context", async () => {
  for (const command of ["pause", "resume"]) {
    const pending = deferred(), calls = [];
    const client = new TauriClient((name, args) => {
      calls.push([name, args]);
      if (calls.length === 1) return pending.promise;
      return Promise.resolve(receipt(snapshot(args.context.worldEpoch), name === "formal_pause" ? "pause" : "resume"));
    });
    await assert.rejects(client[command](context(), 10), /E_LIFECYCLE_TIMEOUT$/);
    await assert.rejects(client.resume(context(), 10), /E_LIFECYCLE_UNCERTAIN/);
    assert.equal(calls.length, 1, "uncertain context cannot dispatch another resume");
    await client.resume(context(snapshot(2)), 10);
    pending.resolve(receipt()); await flush();
    const compensation = calls.filter(([name, args]) => name === "formal_pause" && args.context.worldEpoch === 1);
    assert.equal(compensation.length, 1);
    if (command === "resume") assert.equal(compensation[0][1].commandSequence, 3);
    assert.equal(calls.some(([, args]) => args.context.worldEpoch !== 1 && args.context.worldEpoch !== 2), false);
  }
});

test("ready and default Continue have finite deadlines and ignore late receipts", async () => {
  const ready = deferred(), calls = [];
  const client = new TauriClient((command, args) => { calls.push([command, args]); return command === "formal_scene_ready" ? ready.promise : Promise.resolve(receipt()); });
  const token = { ...context(), generation: 4 };
  await assert.rejects(client.sceneReady(token, false, 10), /E_SCENE_READY_TIMEOUT$/);
  token.sceneId = "mutated";
  await assert.rejects(client.resume(context(), 10), /E_LIFECYCLE_UNCERTAIN/);
  ready.resolve(receipt(snapshot(), "scene-ready:4")); await flush();
  assert.deepEqual(calls.at(-1), ["formal_pause", { context: context(), commandSequence: 1 }]);
  const load = deferred(); let loadCalls = 0;
  const continueClient = new TauriClient(command => { assert.equal(command, "formal_continue"); loadCalls++; return load.promise; });
  await assert.rejects(continueClient.continueJourney(10), /E_CONTINUE_TIMEOUT$/);
  load.resolve({ snapshot: snapshot(99) }); await flush(); assert.equal(loadCalls, 1);
  await assert.rejects(continueClient.continueJourney(0), /E_CONTINUE_TIMEOUT_INVALID/);
  await assert.rejects(client.sceneReady({ ...context(), generation: 1 }, true, 0), /E_SCENE_READY_TIMEOUT_INVALID/);
  await assert.rejects(client.pause(context(), 0), /E_LIFECYCLE_TIMEOUT_INVALID/);
});

test("pagehide preempts pending input and guard, detaches IPC and never applies late receipts", async () => {
  const input = deferred(), pause = deferred(), calls = []; const h = harness({ client: {
    submitInput: () => input.promise, pause: ctx => { calls.push(ctx); return pause.promise; },
  } });
  await h.loop.start(snapshot()); h.loop.keyDown("w"); h.loop.keyDown("e"); await flush(); h.tick(); h.loop.keyUp("e"); h.loop.keyDown("r");
  h.windowTarget.fire("pagehide");
  assert.deepEqual(calls, [context()]); assert.equal(h.loop.input.isHeld("w"), false); assert.equal(h.loop.input.isHeld("e"), false);
  assert.equal(h.loop.keyDown("j"), null); assert.equal(h.intervals.size, 0); assert.equal(h.frames.size, 0);
  await h.loop.stop("unload"); assert.equal(h.destroyed, 1); assert.equal(h.cleaned, 1); assert.equal(h.loop.stopAuthorityState, "pending");
  const count = h.views.length, statuses = h.statuses.length;
  input.resolve(snapshot(2, 9)); pause.resolve(snapshot(2, 10)); await flush();
  assert.equal(h.views.length, count); assert.equal(h.statuses.length, statuses); assert.equal(h.loop.stopAuthorityState, "confirmed");
  assert.deepEqual(h.calls, [["action", "guardStart"]], "queued release/combat never drains into another world");
});

test("failed stop pause is truthful and Hub rejects only after renderer cleanup", async () => {
  const h = harness({ client: { pause: async () => { throw new Error("bridge disconnected"); } } });
  await h.loop.start(snapshot()); await assert.rejects(h.loop.stop("hub"), /bridge disconnected/);
  assert.equal(h.destroyed, 1); assert.equal(h.cleaned, 1); assert.equal(h.loop.stopAuthorityState, "failed");
  assert.match(String(h.loop.stopAuthorityError), /bridge disconnected/); assert.equal(h.statuses.some(([state]) => state === "paused"), false);
  const delayed = deferred(); const next = harness({ client: { pause: () => delayed.promise } });
  await next.loop.start(snapshot()); await next.loop.stop("error"); assert.equal(next.destroyed, 1); assert.equal(next.loop.stopAuthorityState, "pending");
  const statuses = next.statuses.length, errors = next.errors.length;
  delayed.reject(new Error("late failure")); await flush();
  assert.equal(next.loop.stopAuthorityState, "failed"); assert.equal(next.statuses.length, statuses); assert.equal(next.errors.length, errors);
});

test("running render failure pauses immediately while transport hangs; pagehide cannot duplicate teardown", async () => {
  const input = deferred(), pause = deferred(), contexts = []; let draws = 0;
  const h = harness({ render: async () => { if (++draws > 1) throw new Error("GPU lost"); }, client: {
    submitInput: () => input.promise, pause: ctx => { contexts.push(ctx); return pause.promise; },
  } });
  await h.loop.start(snapshot()); h.loop.keyDown("w"); h.tick(); h.frame(); await flush();
  assert.deepEqual(contexts, [context()]); assert.match(String(h.errors[0]), /GPU lost/); assert.equal(h.destroyed, 1);
  h.windowTarget.fire("pagehide"); await h.loop.stop("unload"); assert.equal(h.destroyed, 1); assert.equal(contexts.length, 1);
  pause.reject(new Error("bridge failed")); input.reject(new Error("old input failure")); await flush(); assert.equal(h.errors.length, 1);
});

test("pause overtakes old living input without regression; unexpected running paused errors stay observable", async () => {
  const input = deferred(); const h = harness({ client: { submitInput: () => input.promise, pause: async () => snapshot(1, 4) } });
  await h.loop.start(snapshot()); h.tick(); await h.loop.pause(); input.resolve(snapshot(1, 2)); await flush();
  assert.equal(h.loop.snapshots.view().serverTick, 4); assert.equal(h.errors.length, 0); assert.equal(h.loop.pausePresentationState, "paused"); await h.loop.stop();
  const unexpected = harness({ client: { submitInput: async () => { throw new Error("E_RUNTIME_PAUSED"); } } });
  await unexpected.loop.start(snapshot()); unexpected.tick(); await flush();
  assert.match(String(unexpected.errors[0]), /E_RUNTIME_PAUSED/); assert.equal(unexpected.destroyed, 1);
});

test("new entry owns a fresh pause lane and old lifecycle failure cannot pause or corrupt it", async () => {
  const old = deferred(), contexts = []; const h = harness({ initial: entry(), client: {
    pause: ctx => { contexts.push(ctx); return ctx.worldEpoch === 1 ? old.promise : Promise.resolve(released(entry(ctx.worldEpoch))); },
  } });
  await h.loop.start(entry()); const pausing = h.loop.pause(); await h.loop.acceptAuthoritativeSnapshot(entry(2));
  assert.equal(h.loop.snapshots.view().worldEpoch, 2); await h.loop.resume(); await h.loop.pause();
  assert.ok(contexts.some(ctx => ctx.worldEpoch === 2)); const statuses = h.statuses.length;
  old.reject(new Error("old context stale")); await pausing; await flush();
  assert.equal(h.statuses.length, statuses); assert.equal(h.errors.length, 0); assert.equal(h.loop.pausePresentationState, "paused"); await h.loop.stop();
});

test("stale render failure cannot tear down a newer entry", async () => {
  const oldFrame = deferred(); let draws = 0; const h = harness({ render: value => { if (value.worldEpoch === 1 && ++draws === 2) return oldFrame.promise; } });
  await h.loop.start(snapshot()); h.frame(); await flush(); const transition = h.loop.acceptAuthoritativeSnapshot(entry(2));
  oldFrame.reject(new Error("old renderer invalidated")); await transition; await flush();
  assert.equal(h.errors.length, 0); assert.equal(h.destroyed, 0); assert.equal(h.loop.pausePresentationState, "running"); await h.loop.stop();
});

test("stop captures identity before cancellation and compensates late ready without reopening", async () => {
  const ready = deferred(), contexts = []; const source = entry(); const h = harness({ initial: source, cancel: () => { source.sceneId = "mutated"; }, client: {
    sceneReady: () => ready.promise, pause: async ctx => { contexts.push(ctx); return released(entry()); },
  } });
  const started = h.loop.start(source); await flush(); await h.loop.stop("unload"); assert.equal(h.destroyed, 1);
  ready.resolve(released(entry())); await started; await flush();
  assert.ok(contexts.length >= 2); assert.ok(contexts.every(ctx => ctx.sceneId === "scene_1" && ctx.worldEpoch === 1));
  assert.equal(h.intervals.size, 0); assert.equal(h.statuses.some(([state]) => state === "running"), false);
});

test("bounded ready timeout compensates and destroys even when its reply never arrives", async () => {
  const ready = deferred(), contexts = []; const native = new TauriClient((command, args) => {
    if (command === "formal_scene_ready") return ready.promise;
    if (command === "formal_pause") { contexts.push(args.context); return Promise.resolve(receipt(released(entry()))); }
    throw new Error(command);
  });
  const h = harness({ initial: entry(), client: { sceneReady: (token, paused) => native.sceneReady(token, paused, 10), pause: ctx => native.pause(ctx, 10) } });
  await assert.rejects(h.loop.start(entry()), /E_SCENE_READY_TIMEOUT/); assert.equal(h.destroyed, 1); assert.equal(h.intervals.size, 0);
  ready.resolve(receipt(released(entry()), "scene-ready:1")); await flush();
  assert.ok(contexts.length >= 3); assert.ok(contexts.every(ctx => ctx.sceneId === "scene_1")); assert.equal(h.statuses.some(([state]) => state === "running"), false);
});

test("bounded resume timeout never reopens controls and retry cannot dispatch into uncertain authority", async () => {
  const pending = deferred(); let resumeCalls = 0;
  const native = new TauriClient((command, args) => command === "formal_resume" ? (resumeCalls++, pending.promise) : Promise.resolve(receipt(snapshot())));
  const h = harness({ client: { pause: ctx => native.pause(ctx, 10), resume: ctx => native.resume(ctx, 10) } });
  await h.loop.start(snapshot()); await h.loop.pause(); await h.loop.resume();
  assert.equal(h.loop.pausePresentationState, "error"); assert.equal(h.intervals.size, 0); assert.equal(h.loop.keyDown("w"), null);
  await h.loop.retryPauseState(); assert.equal(resumeCalls, 1); assert.match(h.statuses.at(-1)[1], /E_LIFECYCLE_UNCERTAIN/);
  await h.loop.stop("unload"); pending.resolve(receipt(snapshot(), "resume")); await flush(); assert.equal(h.intervals.size, 0); assert.equal(h.destroyed, 1);
});

test("cleanup always runs when renderer destruction rejects", async () => {
  const h = harness({ destroy: async () => { throw new Error("destroy failed"); } }); await h.loop.start(snapshot());
  await assert.rejects(h.loop.stop("hub"), /destroy failed/); assert.equal(h.cleaned, 1); assert.equal(h.destroyed, 1); assert.equal(h.loop.stopAuthorityState, "confirmed");
});


test("pause reversal supersedes pending resume and stale rejection cannot publish error", async () => {
  const resume = deferred(), contexts = []; const h = harness({ client: {
    pause: async ctx => { contexts.push(ctx); return snapshot(); }, resume: () => resume.promise,
  } });
  await h.loop.start(snapshot()); await h.loop.pause(); const resumed = h.loop.resume(); await flush();
  await h.loop.pause(); assert.equal(contexts.length, 2, "priority pause dispatches before resume settles");
  const statuses = h.statuses.length;
  resume.reject(new Error("E_LIFECYCLE_STALE_COMMAND")); await resumed; await flush();
  assert.equal(h.loop.pausePresentationState, "paused"); assert.equal(h.statuses.length, statuses); assert.equal(h.intervals.size, 0); await h.loop.stop();
});


test("Hub stop bounds pending input and registered writes, rejects uncertainty, and always cleans up", async () => {
  for (const mode of ["input", "write"]) {
    const pending = deferred(); const h = harness({ client: { submitInput: () => pending.promise }, options: { stopTimeoutMs: 10 } });
    await h.loop.start(snapshot());
    let command;
    if (mode === "input") h.tick();
    else { command = h.loop.runWithSession(() => pending.promise); await flush(); }
    let returns = 0;
    await assert.rejects(h.loop.stop("hub").then(() => returns++), /E_SESSION_STOP_TIMEOUT/);
    assert.equal(returns, 0); assert.equal(h.destroyed, 1); assert.equal(h.cleaned, 1); assert.equal(h.loop.stopAuthorityState, "confirmed");
    const states = h.statuses.length;
    pending.resolve(snapshot(2));
    if (command) await assert.rejects(command, /E_BUILD_SESSION_CHANGED/);
    await flush(); assert.equal(h.statuses.length, states); assert.equal(h.loop.snapshots.view().worldEpoch, 1);
  }
});


test("replacement stop with unsettled external mutation rejects after cleanup and never accepts its late receipt", async () => {
  const pending = deferred(); const h = harness(); await h.loop.start(snapshot());
  const command = h.loop.runWithSession(() => pending.promise); await flush();
  await assert.rejects(h.loop.stop("replace"), /E_SESSION_STOP_UNCERTAIN_MUTATION/);
  assert.equal(h.loop.hasUnsettledStopMutation, true); assert.equal(h.destroyed, 1); assert.equal(h.cleaned, 1);
  const views = h.views.length; pending.resolve(snapshot(2)); await assert.rejects(command, /E_BUILD_SESSION_CHANGED/);
  assert.equal(h.views.length, views); assert.equal(h.loop.hasUnsettledStopMutation, true);
});
