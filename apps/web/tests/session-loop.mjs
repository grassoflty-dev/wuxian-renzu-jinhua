import test from "node:test";
import assert from "node:assert/strict";
import { SessionLoop } from "../dist/game/SessionLoop.js";

function snapshot(epoch = 4, tick = 1) {
  return {
    kind: "full", protocolVersion: 3, schemaVersion: "freeze-v02-interfaces/1.2",
    worldId: "grey_hive", sceneId: "grey_hive", checkpointId: null,
    worldEpoch: epoch, serverTick: tick, authorityRevision: tick, ackSeq: tick,
    player: { entityId: "player", transform: { positionM: { xM: tick, yM: 0, zM: 0 }, yawRad: 0 },
      velocityMps: { xM: 0, yM: 0, zM: 0 }, currentHp: 100, maxHp: 100,
      currentEnergy: 100, maxEnergy: 100, facingX: 0, facingZ: 1, aimX: 0, aimZ: 1, actionState: "idle" },
    actors: [], doors: [], interactables: [], hazards: [], objectives: [],
    capabilities: { schemaVersion: 1, items: [] },
    progression: { schemaVersion: 1, currentWorldId: "grey_hive", eventSeq: 0, worlds: [] },
  };
}

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

class Scheduler {
  time = 0;
  next = 0;
  intervals = new Map();
  frames = new Map();
  clearedIntervals = 0;
  cancelledFrames = 0;
  now = () => this.time;
  clientTimeMs = () => this.time;
  setInterval = (callback, delay) => { const id = ++this.next; this.intervals.set(id, { callback, delay }); return id; };
  clearInterval = id => { this.clearedIntervals++; this.intervals.delete(id); };
  requestAnimationFrame = callback => { const id = ++this.next; this.frames.set(id, callback); return id; };
  cancelAnimationFrame = id => { this.cancelledFrames++; this.frames.delete(id); };
  tick() { for (const timer of [...this.intervals.values()]) timer.callback(); }
  frame() { const entry = this.frames.entries().next().value; if (!entry) return; const [id, callback] = entry; this.frames.delete(id); callback(this.time); }
}

class EventSurface {
  listeners = new Map();
  addEventListener(type, listener) { const list = this.listeners.get(type) ?? new Set(); list.add(listener); this.listeners.set(type, list); }
  removeEventListener(type, listener) { this.listeners.get(type)?.delete(listener); }
  fire(type, event = {}) { for (const listener of this.listeners.get(type) ?? []) listener(event); }
  count() { return [...this.listeners.values()].reduce((sum, list) => sum + list.size, 0); }
}

async function flush() { for (let index = 0; index < 12; index++) await Promise.resolve(); }
async function waitFor(predicate) {
  for (let index = 0; index < 100; index++) {
    if (predicate()) return;
    await new Promise(resolve => setTimeout(resolve, 0));
  }
  assert.fail("timed out waiting for lifecycle IPC");
}
function setup(client, options = {}) {
  const scheduler = options.scheduler ?? new Scheduler();
  const windowTarget = new EventSurface();
  const documentTarget = new EventSurface();
  const rendered = [];
  const destroyed = { value: false };
  const renderer = { render: async (state, player) => { rendered.push({ state, player }); }, destroy: async () => { destroyed.value = true; } };
  const loop = new SessionLoop({ ...eventClient, ...client }, renderer, { scheduler, windowTarget, documentTarget, ...options });
  return { loop, scheduler, windowTarget, documentTarget, rendered, destroyed };
}
const eventClient = { events: async () => [], pause: async () => snapshot(), resume: async () => snapshot() };

function pointerSetup(options = {}) {
  const pointerTarget = new EventSurface();
  pointerTarget.getBoundingClientRect = () => ({ left: 20, top: 30, width: 800, height: 600 });
  const calls = [];
  let tick = 1;
  const state = setup({
    submitInput: async input => { calls.push({ kind: "input", input }); return snapshot(4, ++tick); },
    submitAction: async action => { calls.push({ kind: "action", action }); return snapshot(4, ++tick); },
    pause: async () => snapshot(4, ++tick), resume: async () => snapshot(4, ++tick),
    ...options.client,
  }, { ...options, pointerTarget });
  state.loop.renderer.pointerAim = (_snapshot, x, y) => ({ x: x - 420, y: y - 330 });
  return { ...state, pointerTarget, calls };
}

function click(surface, overrides = {}) {
  let prevented = false;
  surface.fire("pointerdown", { button: 0, isPrimary: true, pointerType: "mouse", clientX: 520, clientY: 330,
    preventDefault() { prevented = true; }, ...overrides });
  return prevented;
}

test("left click submits its displayed-pose aim before primary attack without a preceding pointermove", async () => {
  const state = pointerSetup();
  await state.loop.start(snapshot());
  assert.equal(click(state.pointerTarget), true);
  await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["input", "action"]);
  assert.ok(state.calls[0].input.aimX > 0.7);
  assert.ok(state.calls[0].input.aimZ < -0.7);
  assert.equal(state.calls[1].action.kind, "primaryAttack");
  assert.equal(state.calls[1].action.worldEpoch, 4);
  assert.equal(state.loop.input.isHeld("j"), false);
  state.loop.keyDown("j");
  await flush();
  click(state.pointerTarget, { clientX: 320 });
  await flush();
  assert.deepEqual(state.calls.filter(call => call.kind === "action").map(call => call.action.requestId), [1, 2, 3]);
  assert.equal(state.loop.input.isHeld("j"), true, "mouse clicks must not release a held J key");
  await state.loop.stop();
  assert.equal(state.pointerTarget.count(), 0);
});

for (const [key, kind] of [["j", "primaryAttack"], ["r", "pierce"]]) {
  test(`keyboard ${key.toUpperCase()} submits its current aim before ${kind} without waiting for an input tick`, async () => {
    const state = pointerSetup();
    await state.loop.start(snapshot());
    state.pointerTarget.fire("pointermove", { clientX: 320, clientY: 330 });
    assert.equal(state.loop.keyDown(key)?.kind, kind);
    await flush();
    assert.deepEqual(state.calls.map(call => call.kind), ["input", "action"]);
    assert.ok(state.calls[0].input.aimX < -0.7);
    assert.ok(state.calls[0].input.aimZ > 0.7);
    assert.equal(state.calls[1].action.kind, kind);
    assert.equal(state.calls[0].input.clientTimeMs, state.calls[1].action.clientTimeMs);
    await state.loop.stop();
  });
}

test("Shift submits newly pressed movement before locking dash direction without an input tick", async () => {
  const state = pointerSetup();
  await state.loop.start(snapshot());
  state.loop.keyDown("w");
  state.loop.keyDown("Shift");
  await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["input", "action"]);
  assert.ok(state.calls[0].input.moveX < -0.7);
  assert.ok(state.calls[0].input.moveZ < -0.7);
  assert.equal(state.calls[1].action.kind, "dash");
  await state.loop.stop();
});

for (const key of ["j", "r", "Shift"]) {
  test(`directional ${key} reprojects a stationary cursor when its key edge arrives`, async () => {
    const state = pointerSetup();
    await state.loop.start(snapshot());
    let playerX = 420;
    state.loop.renderer.pointerAim = (_snapshot, x, y) => ({ x: x - playerX, y: y - 330 });
    state.pointerTarget.fire("pointermove", { clientX: 520, clientY: 330 });
    assert.ok(state.loop.input.aim().aimX > 0);
    playerX = 620;
    state.loop.keyDown(key);
    await flush();
    assert.ok(state.calls[0].input.aimX < -0.7, "the last displayed pose must be used at keydown");
    assert.ok(state.calls[0].input.aimZ > 0.7);
    await state.loop.stop();
  });

  test(`directional ${key} works without a pointer and keeps zero aim when no displayed pose is usable`, async () => {
    const state = pointerSetup();
    await state.loop.start(snapshot());
    state.loop.keyDown(key); await flush();
    assert.equal(state.calls[0].input.aimX, 0);
    assert.equal(state.calls[0].input.aimZ, 0);
    assert.equal(state.calls[1].kind, "action");
    state.loop.keyUp(key);
    state.pointerTarget.fire("pointermove", { clientX: 520, clientY: 330 });
    state.loop.renderer.pointerAim = () => null;
    state.loop.keyDown(key); await flush();
    assert.equal(state.calls[2].input.aimX, 0);
    assert.equal(state.calls[2].input.aimZ, 0);
    assert.equal(state.calls[3].kind, "action", "no pointer aim preserves keyboard access and authority facing");
    await state.loop.stop();
  });

  test(`directional ${key} cannot reuse cached canvas aim after its surface becomes unusable`, async () => {
    for (const rect of [
      { left: 20, top: 30, width: 0, height: 600 },
      { left: 20, top: 30, width: 800, height: NaN },
    ]) {
      const state = pointerSetup();
      await state.loop.start(snapshot());
      state.pointerTarget.fire("pointermove", { clientX: 520, clientY: 330 });
      assert.ok(state.loop.input.aim().aimX > 0);
      state.pointerTarget.getBoundingClientRect = () => rect;
      state.loop.keyDown(key); await flush();
      assert.equal(state.calls[0].input.aimX, 0);
      assert.equal(state.calls[0].input.aimZ, 0);
      assert.equal(state.calls[1].kind, "action");
      await state.loop.stop();
    }
  });

  test(`directional ${key} retains manually supplied aim for hosts without a canvas pointer`, async () => {
    const inputs = [];
    const actions = [];
    const state = setup({
      submitInput: async input => { inputs.push(input); return snapshot(4, 2); },
      submitAction: async action => { actions.push(action); return snapshot(4, 3); },
    });
    await state.loop.start(snapshot());
    state.loop.pointer(100, 0); state.loop.keyDown(key); await flush();
    assert.ok(inputs[0].aimX > 0.7); assert.ok(inputs[0].aimZ < -0.7);
    assert.equal(actions.length, 1);
    await state.loop.stop();
  });

  test(`queued ${key} keeps its edge aim/time without replaying released movement`, async () => {
    const pending = deferred();
    const state = pointerSetup();
    const submitInput = state.loop.client.submitInput;
    let first = true;
    state.loop.client.submitInput = input => {
      if (!first) return submitInput(input);
      first = false; state.calls.push({ kind: "input", input }); return pending.promise;
    };
    await state.loop.start(snapshot());
    state.loop.keyDown("w"); state.scheduler.tick();
    state.scheduler.time = 10;
    state.pointerTarget.fire("pointermove", { clientX: 520, clientY: 330 });
    state.loop.keyDown(key);
    state.loop.keyUp("w");
    state.pointerTarget.fire("pointermove", { clientX: 320, clientY: 330 });
    state.scheduler.time = 20;
    pending.resolve(snapshot(4, 2));
    await waitFor(() => state.calls.length === 3);
    assert.equal(state.calls[1].input.moveX, 0);
    assert.equal(state.calls[1].input.moveZ, 0);
    assert.ok(state.calls[1].input.aimX > 0);
    assert.deepEqual(state.calls.filter(call => call.kind === "input").map(call => call.input.seq), [1, 2]);
    assert.equal(state.calls[1].input.clientTimeMs, 10);
    assert.equal(state.calls[1].input.clientTimeMs, state.calls[2].action.clientTimeMs);
    await state.loop.stop();
  });

  test(`directional ${key} suppresses held and OS repeats until a fresh press`, async () => {
    const state = pointerSetup();
    await state.loop.start(snapshot());
    state.windowTarget.fire("keydown", { key, preventDefault() {} });
    assert.equal(state.loop.keyDown(key.toUpperCase()), null);
    state.windowTarget.fire("keydown", { key, repeat: true, preventDefault() {} });
    await flush();
    assert.deepEqual(state.calls.map(call => call.kind), ["input", "action"]);
    state.loop.keyUp(key);
    state.windowTarget.fire("keydown", { key, preventDefault() {} });
    await flush();
    assert.deepEqual(state.calls.map(call => call.kind), ["input", "action", "input", "action"]);
    assert.deepEqual(state.calls.filter(call => call.kind === "action").map(call => call.action.requestId), [1, 2]);
    await state.loop.stop();
  });

  test(`directional ${key} is canceled by pause or blur during its input receipt and cannot reappear on resume`, async () => {
    for (const source of ["manual", "blur"]) {
      const pending = deferred();
      const state = pointerSetup();
      const submitInput = state.loop.client.submitInput;
      state.loop.client.submitInput = input => { state.calls.push({ kind: "input", input }); return pending.promise; };
      await state.loop.start(snapshot());
      state.loop.keyDown(key);
      if (source === "manual") await state.loop.pause();
      else { state.windowTarget.fire("blur"); await waitFor(() => state.loop.pausePresentationState === "paused"); }
      assert.equal(state.loop.keyDown(key), null);
      pending.resolve(snapshot(4, 2)); await flush();
      if (source === "manual") await state.loop.resume();
      else { state.windowTarget.fire("focus"); await waitFor(() => state.loop.pausePresentationState === "running"); }
      state.windowTarget.fire("keydown", { key, repeat: true, preventDefault() {} });
      await flush();
      assert.deepEqual(state.calls.map(call => call.kind), ["input"]);
      state.loop.client.submitInput = submitInput;
      state.loop.keyUp(key); state.loop.keyDown(key); await flush();
      assert.deepEqual(state.calls.map(call => call.kind), ["input", "input", "action"]);
      await state.loop.stop();
    }
  });

  test(`directional ${key} cannot escape a fatal, replaced, or failed input receipt`, async () => {
    for (const outcome of ["dead", "epoch", "scene", "failure"]) {
      const errors = [];
      const state = pointerSetup({ onError: error => errors.push(String(error)) });
      state.loop.client.submitInput = async input => {
        state.calls.push({ kind: "input", input });
        if (outcome === "failure") throw new Error("directional input failed");
        const result = snapshot(outcome === "epoch" ? 5 : 4, 2);
        if (outcome === "dead") result.player.currentHp = 0;
        if (outcome === "scene") result.sceneId = "replacement_scene";
        return result;
      };
      await state.loop.start(snapshot());
      state.loop.keyDown(key); await flush();
      assert.deepEqual(state.calls.map(call => call.kind), ["input"], outcome);
      if (outcome === "dead") { assert.equal(state.loop.isDead, true); assert.equal(state.loop.keyDown(key), null); }
      if (outcome === "failure") assert.ok(errors.some(error => error.includes("directional input failed")));
      await state.loop.stop().catch(() => undefined);
    }
  });

  test(`directional ${key} is blocked during loading and dropped by stop during its input receipt`, async () => {
    const loading = deferred();
    const pending = deferred();
    const state = pointerSetup();
    state.loop.renderer.render = () => loading.promise;
    const started = state.loop.start(snapshot());
    assert.equal(state.loop.keyDown(key), null);
    assert.deepEqual(state.calls, []);
    loading.resolve(); await started;
    state.loop.client.submitInput = input => { state.calls.push({ kind: "input", input }); return pending.promise; };
    state.loop.keyDown(key);
    const stopped = state.loop.stop("hub");
    pending.resolve(snapshot(4, 2)); await stopped; await flush();
    assert.deepEqual(state.calls.map(call => call.kind), ["input"]);
    assert.equal(state.loop.keyDown(key), null);
  });
}

test("mixed mouse and keyboard edges preserve queue order and each directional edge's own aim", async () => {
  const pending = deferred();
  const state = pointerSetup();
  const submitInput = state.loop.client.submitInput;
  let first = true;
  state.loop.client.submitInput = input => {
    if (!first) return submitInput(input);
    first = false; state.calls.push({ kind: "input", input }); return pending.promise;
  };
  await state.loop.start(snapshot()); state.scheduler.tick();
  state.pointerTarget.fire("pointermove", { clientX: 320, clientY: 330 }); state.loop.keyDown("j");
  click(state.pointerTarget, { clientX: 520 });
  state.loop.keyDown("e"); state.loop.keyUp("e");
  state.pointerTarget.fire("pointermove", { clientX: 320, clientY: 330 }); state.loop.keyDown("r");
  state.pointerTarget.fire("pointermove", { clientX: 520, clientY: 330 }); state.loop.keyDown("Shift");
  pending.resolve(snapshot(4, 2));
  await waitFor(() => state.calls.length === 11);
  assert.deepEqual(state.calls.map(call => call.kind), ["input", "input", "action", "input", "action", "action", "action", "input", "action", "input", "action"]);
  assert.deepEqual(state.calls.filter(call => call.kind === "action").map(call => call.action.kind), ["primaryAttack", "primaryAttack", "guardStart", "guardEnd", "pierce", "dash"]);
  assert.deepEqual(state.calls.filter(call => call.kind === "action").map(call => call.action.requestId), [1, 2, 3, 4, 5, 6]);
  assert.deepEqual(state.calls.filter(call => call.kind === "input").slice(1).map(call => Math.sign(call.input.aimX)), [-1, 1, -1, 1]);
  assert.equal(state.loop.input.isHeld("j"), true, "click still leaves held J untouched");
  await state.loop.stop();
});

test("Q E guard release Space and F never introduce an aim input transaction", async () => {
  let interactions = 0;
  const state = pointerSetup({ onInteract: () => interactions++ });
  await state.loop.start(snapshot());
  state.pointerTarget.fire("pointermove", { clientX: 520, clientY: 330 });
  state.loop.client.submitInput = () => { assert.fail("non-directional actions must not wait for aim IPC"); };
  state.loop.keyDown("q"); await flush();
  state.loop.keyDown("e"); await flush();
  assert.equal(state.loop.input.isHeld("e"), true);
  assert.equal(state.loop.keyDown("e"), null);
  state.loop.keyUp("e"); await flush();
  state.loop.keyDown(" "); await flush();
  state.loop.keyDown("f"); await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["action", "action", "action", "action"]);
  assert.deepEqual(state.calls.map(call => call.action.kind), ["pulse", "guardStart", "guardEnd", "contextTraversal"]);
  assert.equal(interactions, 1);
  assert.equal(state.loop.input.isHeld("e"), false);
  await state.loop.stop();
});

test("directional keyboard edges leave consumed events and editable UI controls untouched", async () => {
  const state = pointerSetup();
  await state.loop.start(snapshot());
  for (const key of ["j", "r", "Shift"]) {
    for (const overrides of [
      { defaultPrevented: true }, { isComposing: true }, { ctrlKey: true }, { altKey: true }, { metaKey: true },
      { target: { isContentEditable: true } }, { target: { closest: () => ({ tagName: "INPUT" }) } },
      { target: { closest: () => ({ tagName: "BUTTON" }) } },
    ]) state.windowTarget.fire("keydown", { key, preventDefault() { assert.fail("native UI event was consumed"); }, ...overrides });
    assert.equal(state.loop.input.isHeld(key), false);
  }
  await flush(); assert.deepEqual(state.calls, []);
  await state.loop.stop();
});

test("pointer attack ignores other buttons, modified clicks, consumed events, UI controls and invalid positions", async () => {
  const state = pointerSetup();
  await state.loop.start(snapshot());
  for (const overrides of [
    { button: 1 }, { button: 2 }, { isPrimary: false }, { ctrlKey: true }, { altKey: true }, { metaKey: true },
    { defaultPrevented: true }, { target: { closest: () => ({ tagName: "BUTTON" }) } },
    { target: { isContentEditable: true } }, { clientX: NaN }, { clientY: Infinity },
  ]) assert.equal(click(state.pointerTarget, overrides), false);
  await flush();
  assert.deepEqual(state.calls, []);
  assert.equal(click(state.pointerTarget, { shiftKey: true }), true, "Shift remains compatible with dash and attack");
  await flush();
  assert.equal(state.calls.at(-1).action.kind, "primaryAttack");
  await state.loop.stop();
});

test("pointer attacks remain blocked before start, during pause, after death and after stop", async () => {
  const state = pointerSetup();
  assert.equal(click(state.pointerTarget), false);
  await state.loop.start(snapshot());
  await state.loop.pause();
  assert.equal(click(state.pointerTarget), false);
  await state.loop.resume();
  state.loop.client.submitInput = async input => {
    state.calls.push({ kind: "input", input });
    const fatal = snapshot(4, 10); fatal.player.currentHp = 0; return fatal;
  };
  click(state.pointerTarget);
  await flush();
  assert.equal(state.loop.pausePresentationState, "dead");
  assert.equal(click(state.pointerTarget), false);
  assert.deepEqual(state.calls.map(call => call.kind), ["input"], "death in the aim receipt cancels the click");
  await state.loop.stop();
  assert.equal(click(state.pointerTarget), false);
});

test("pause during pointer aim IPC cancels the attack and it does not reappear after resume", async () => {
  const pending = deferred();
  const state = pointerSetup();
  state.loop.client.submitInput = input => { state.calls.push({ kind: "input", input }); return pending.promise; };
  await state.loop.start(snapshot());
  click(state.pointerTarget);
  await state.loop.pause();
  pending.resolve(snapshot(4, 2));
  await flush();
  await state.loop.resume();
  await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["input"]);
  await state.loop.stop();
});

test("pointer aim failure or replacement scene cannot submit the queued attack", async () => {
  for (const replacement of [true, false]) {
    const errors = [];
    const state = pointerSetup({ onError: error => errors.push(String(error)) });
    state.loop.client.submitInput = async input => {
      state.calls.push({ kind: "input", input });
      if (!replacement) throw new Error("aim transport failed");
      return snapshot(5, 2);
    };
    await state.loop.start(snapshot());
    click(state.pointerTarget);
    await flush();
    assert.deepEqual(state.calls.map(call => call.kind), ["input"]);
    if (!replacement) assert.ok(errors.some(error => error.includes("aim transport failed")));
    await state.loop.stop().catch(() => undefined);
  }
});

test("clicks queued behind another IPC preserve each click's aim and action order", async () => {
  const pending = deferred();
  const state = pointerSetup();
  const submitInput = state.loop.client.submitInput;
  let first = true;
  state.loop.client.submitInput = input => {
    if (!first) return submitInput(input);
    first = false; state.calls.push({ kind: "input", input }); return pending.promise;
  };
  await state.loop.start(snapshot());
  state.scheduler.tick();
  click(state.pointerTarget, { clientX: 520 });
  click(state.pointerTarget, { clientX: 320 });
  pending.resolve(snapshot(4, 2));
  await waitFor(() => state.calls.length === 5);
  assert.deepEqual(state.calls.map(call => call.kind), ["input", "input", "action", "input", "action"]);
  assert.ok(state.calls[1].input.aimX > 0);
  assert.ok(state.calls[3].input.aimX < 0);
  assert.deepEqual(state.calls.filter(call => call.kind === "input").map(call => call.input.seq), [1, 2, 3]);
  await state.loop.stop();
});

test("held F interacts once until keyup even after the previous callback has resolved", async () => {
  const pending = deferred();
  let interactions = 0;
  const state = setup({}, { onInteract: () => { interactions++; return interactions === 1 ? pending.promise : Promise.resolve(); } });
  await state.loop.start(snapshot());
  state.loop.keyDown("f"); state.loop.keyDown("F");
  assert.equal(interactions, 1);
  pending.resolve(); await flush();
  state.loop.keyDown("f");
  state.windowTarget.fire("keydown", { key: "f", repeat: true, preventDefault() {} });
  assert.equal(interactions, 1);
  state.loop.keyUp("f"); state.loop.keyDown("f");
  assert.equal(interactions, 2);
  await state.loop.stop();
});

test("Ctrl Alt and Meta shortcuts do not trigger gameplay while Shift remains the dash key", async () => {
  let interactions = 0;
  const state = pointerSetup({ onInteract: () => interactions++ });
  await state.loop.start(snapshot());
  for (const modifier of ["ctrlKey", "altKey", "metaKey"]) {
    for (const key of ["f", "r", "w", "j"]) state.windowTarget.fire("keydown", {
      key, [modifier]: true, preventDefault() { assert.fail("native shortcut was consumed"); },
    });
  }
  assert.equal(interactions, 0); assert.deepEqual(state.calls, []);
  assert.equal(state.loop.input.isHeld("w"), false);
  state.windowTarget.fire("keydown", { key: "Shift", shiftKey: true, preventDefault() {} });
  await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["input", "action"]);
  assert.equal(state.calls[1].action.kind, "dash");
  await state.loop.stop();
});

test("loading and an unavailable displayed aim frame reject pointer attack until readiness returns", async () => {
  const loading = deferred();
  const state = pointerSetup();
  state.loop.renderer.render = () => loading.promise;
  const started = state.loop.start(snapshot());
  assert.equal(click(state.pointerTarget), false);
  loading.resolve(); await started;
  const pointerAim = state.loop.renderer.pointerAim;
  state.loop.renderer.pointerAim = () => null;
  assert.equal(click(state.pointerTarget), false);
  state.loop.renderer.pointerAim = pointerAim;
  const rect = state.pointerTarget.getBoundingClientRect;
  state.pointerTarget.getBoundingClientRect = () => ({ left: 20, top: 30, width: 0, height: 600 });
  assert.equal(click(state.pointerTarget), false);
  assert.deepEqual(state.calls, []);
  state.pointerTarget.getBoundingClientRect = rect;
  assert.equal(click(state.pointerTarget), true);
  await flush();
  assert.equal(state.calls.at(-1).action.kind, "primaryAttack");
  await state.loop.stop();
});

test("UI pointer presses and held-button dragging across the canvas do not create attacks", async () => {
  const state = pointerSetup();
  await state.loop.start(snapshot());
  assert.equal(click(state.windowTarget), false, "the attack listener belongs only to the game canvas");
  assert.equal(click(state.documentTarget), false);
  for (const target of [
    { closest: () => ({ tagName: "INPUT" }) }, { closest: () => ({ tagName: "TEXTAREA" }) },
    { closest: () => ({ tagName: "SELECT" }) }, { isContentEditable: true },
  ]) assert.equal(click(state.pointerTarget, { target }), false);
  for (let step = 0; step < 4; step++) state.pointerTarget.fire("pointermove", {
    clientX: 480 + step, clientY: 330, buttons: 1,
  });
  await flush(); assert.deepEqual(state.calls, []);
  await state.loop.stop();
});

test("blur cancels an in-flight pointer attack and focus requires a new click", async () => {
  const pending = deferred();
  const state = pointerSetup();
  const submitInput = state.loop.client.submitInput;
  state.loop.client.submitInput = input => { state.calls.push({ kind: "input", input }); return pending.promise; };
  await state.loop.start(snapshot());
  click(state.pointerTarget);
  state.windowTarget.fire("blur");
  assert.equal(click(state.pointerTarget), false);
  await waitFor(() => state.loop.pausePresentationState === "paused");
  pending.resolve(snapshot(4, 2)); await flush();
  state.windowTarget.fire("focus");
  await waitFor(() => state.loop.pausePresentationState === "running");
  assert.deepEqual(state.calls.map(call => call.kind), ["input"]);
  state.loop.client.submitInput = submitInput;
  click(state.pointerTarget); await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["input", "input", "action"]);
  await state.loop.stop();
});

test("a stopped session never attacks when its delayed pointer aim receipt arrives", async () => {
  const pending = deferred();
  const state = pointerSetup();
  state.loop.client.submitInput = input => { state.calls.push({ kind: "input", input }); return pending.promise; };
  await state.loop.start(snapshot());
  click(state.pointerTarget);
  const stopped = state.loop.stop("hub");
  pending.resolve(snapshot(4, 2));
  await stopped; await flush();
  assert.deepEqual(state.calls.map(call => call.kind), ["input"]);
  assert.equal(state.pointerTarget.count(), 0);
});

test("OS repeats after focus resume cannot re-arm interaction until a new physical key press", async () => {
  let interactions = 0;
  const state = pointerSetup({ onInteract: () => interactions++ });
  await state.loop.start(snapshot());
  state.windowTarget.fire("keydown", { key: "f", preventDefault() {} });
  await state.loop.pause(); await state.loop.resume();
  state.windowTarget.fire("keydown", { key: "f", repeat: true, preventDefault() {} });
  assert.equal(interactions, 1);
  state.windowTarget.fire("keyup", { key: "f" });
  state.windowTarget.fire("keydown", { key: "f", repeat: false, preventDefault() {} });
  assert.equal(interactions, 2);
  await state.loop.stop();
});

test("queued click retains its aim but never replays movement released while another IPC was pending", async () => {
  const pending = deferred();
  const state = pointerSetup();
  const submitInput = state.loop.client.submitInput;
  let first = true;
  state.loop.client.submitInput = input => {
    if (!first) return submitInput(input);
    first = false; state.calls.push({ kind: "input", input }); return pending.promise;
  };
  await state.loop.start(snapshot());
  state.loop.keyDown("w"); state.scheduler.tick();
  state.scheduler.time = 10;
  click(state.pointerTarget, { clientX: 520 });
  state.loop.keyUp("w");
  state.pointerTarget.fire("pointermove", { clientX: 320, clientY: 330 });
  state.scheduler.time = 20;
  pending.resolve(snapshot(4, 2));
  await waitFor(() => state.calls.length === 3);
  assert.equal(state.calls[1].input.moveX, 0);
  assert.equal(state.calls[1].input.moveZ, 0);
  assert.ok(state.calls[1].input.aimX > 0, "the original click aim survives the later pointer move");
  assert.equal(state.calls[1].input.clientTimeMs, state.calls[2].action.clientTimeMs);
  await state.loop.stop();
});

test("uses nominal 60 Hz input ticks and drops overdue samples while IPC is busy", async () => {
  const pending = deferred();
  const submitted = [];
  let calls = 0;
  const client = { ...eventClient, submitInput: sample => { submitted.push(sample); calls++; return calls === 1 ? pending.promise : Promise.resolve(snapshot(4, calls + 1)); }, submitAction: async () => snapshot() };
  const state = setup(client);
  await state.loop.start(snapshot());
  assert.equal([...state.scheduler.intervals.values()][0].delay, 1000 / 60);
  state.scheduler.tick();
  state.scheduler.time += 50;
  state.scheduler.tick();
  state.scheduler.tick();
  assert.equal(submitted.length, 1);
  pending.resolve(snapshot(4, 2));
  await flush();
  state.scheduler.tick();
  await flush();
  assert.equal(submitted.length, 2);
  assert.deepEqual(submitted.map(sample => sample.seq), [1, 2]);
  await state.loop.stop();
});

test("serializes action edges, suppresses repeated keydown, and releases guard on pause", async () => {
  const actions = [];
  const pending = deferred();
  let submittedInput = 0;
  const client = { ...eventClient,
    submitInput: async () => { submittedInput++; return snapshot(4, 2); },
    submitAction: action => { actions.push(action); return actions.length === 1 ? pending.promise : Promise.resolve(snapshot(4, actions.length + 2)); },
    pause: async () => snapshot(4, 6),
  };
  const state = setup(client);
  await state.loop.start(snapshot());
  assert.equal(state.loop.keyDown("e")?.kind, "guardStart");
  assert.equal(state.loop.keyDown("e"), null);
  state.loop.keyUp("e");
  assert.deepEqual(actions.map(action => action.kind), ["guardStart"]);
  pending.resolve(snapshot(4, 2));
  await flush();
  assert.deepEqual(actions.map(action => action.kind), ["guardStart", "guardEnd"]);
  assert.deepEqual(actions.map(action => action.requestId), [1, 2]);
  assert.equal(submittedInput, 0);
  state.loop.keyDown("e");
  await flush();
  await state.loop.pause();
  assert.equal(actions.at(-1).kind, "guardStart", "pause neutralizes guard without another combat command");
  assert.equal(state.loop.input.isHeld("e"), false);
  await state.loop.stop();
});

test("updates the active epoch, resets event cursor, and drops duplicate presentation events", async () => {
  const requests = [];
  const delivered = [];
  const makeEvent = (epoch, eventId) => ({ protocolVersion: 2, eventId, worldEpoch: epoch, serverTick: eventId, kind: "Hit", positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 1, intensity: 1 });
  let actionCount = 0;
  const client = { submitAction: async () => ++actionCount === 1 ? snapshot(4, 2) : snapshot(5, 3), submitInput: async () => snapshot(4, actionCount + 1),
    events: async (epoch, cursor) => {
      requests.push([epoch, cursor]);
      return epoch === 4 ? (cursor === 0 ? [makeEvent(epoch, 1)] : [makeEvent(epoch, 1), makeEvent(epoch, 2)]) : [makeEvent(epoch, 1)];
    } };
  const state = setup(client, { onEvents: events => delivered.push(...events) });
  await state.loop.start(snapshot(4));
  state.loop.keyDown("j");
  await flush();
  state.loop.keyDown("r");
  await flush();
  assert.equal(state.loop.snapshots.view().worldEpoch, 5);
  assert.deepEqual(requests, [[4, 0], [4, 1], [5, 0]]);
  assert.deepEqual(delivered.map(event => [event.worldEpoch, event.eventId]), [[4, 1], [4, 2], [5, 1]]);
  await state.loop.stop();
});

test("hub stop and epoch change discard queued actions from the previous scene", async () => {
  const first = deferred();
  const sent = [];
  const client = { ...eventClient,
    submitInput: async () => snapshot(4, 1),
    submitAction: action => { sent.push(action); return sent.length === 1 ? first.promise : Promise.resolve(snapshot(5, 3)); },
  };
  const state = setup(client);
  await state.loop.start(snapshot(4));
  state.loop.keyDown("j");
  await waitFor(() => sent.length === 1);
  state.loop.keyDown("r");
  first.resolve(snapshot(5, 2));
  await flush();
  assert.deepEqual(sent.map(action => action.kind), ["primaryAttack"]);
  assert.equal(state.loop.snapshots.view().worldEpoch, 5);
  await state.loop.stop("hub");
  assert.deepEqual(sent.map(action => action.kind), ["primaryAttack"]);
});

test("returning to the hub does not flush queued combat actions", async () => {
  const pending = deferred();
  const sent = [];
  const client = { ...eventClient,
    submitInput: async () => snapshot(4, 2),
    submitAction: action => { sent.push(action); return sent.length === 1 ? pending.promise : Promise.resolve(snapshot(4, 3)); },
  };
  const state = setup(client);
  await state.loop.start(snapshot(4));
  state.loop.keyDown("j");
  await waitFor(() => sent.length === 1);
  state.loop.keyDown("r");
  const stopping = state.loop.stop("hub");
  pending.resolve(snapshot(4, 2));
  await stopping;
  assert.deepEqual(sent.map(action => action.kind), ["primaryAttack"]);
});

test("stop cancels scheduling, removes listeners, drops guard edges, and ignores late events", async () => {
  const eventPending = deferred();
  const actions = [];
  let eventCalls = 0;
  const client = { submitInput: async () => snapshot(4, 2), submitAction: async action => { actions.push(action); return snapshot(4, 2); },
    events: () => ++eventCalls === 1 ? Promise.resolve([]) : eventPending.promise };
  const state = setup(client);
  await state.loop.start(snapshot());
  state.loop.keyDown("e");
  state.scheduler.tick();
  await flush();
  state.loop.keyDown("e");
  const stopping = state.loop.stop("hub");
  await flush();
  eventPending.resolve([{ protocolVersion: 2, eventId: 1, worldEpoch: 4, serverTick: 2, kind: "Late", positionM: { xM: 0, yM: 0, zM: 0 }, directionRad: 0, radiusM: 1, intensity: 1 }]);
  await stopping;
  assert.equal(actions.some(action => action.kind === "guardEnd"), false);
  assert.equal(state.loop.input.isHeld("e"), false);
  assert.equal(state.scheduler.intervals.size, 0);
  assert.equal(state.scheduler.frames.size, 0);
  assert.equal(state.windowTarget.count(), 0);
  assert.equal(state.documentTarget.count(), 0);
  assert.equal(state.destroyed.value, true);
});

test("pause/resume suspends schedulers, and epoch changes clear held controls", async () => {
  let current = snapshot(4, 1);
  const client = { ...eventClient, submitAction: async () => { current = snapshot(5, 2); return current; }, submitInput: async () => current };
  const state = setup(client);
  await state.loop.start(current);
  state.loop.keyDown("w");
  assert.equal(state.loop.input.isHeld("w"), true);
  await state.loop.pause();
  assert.equal(state.loop.input.isHeld("w"), false);
  assert.equal(state.scheduler.intervals.size, 0);
  await state.loop.resume();
  assert.equal(state.scheduler.intervals.size, 1);
  state.loop.keyDown("j");
  await flush();
  assert.equal(state.loop.snapshots.view().worldEpoch, 5);
  assert.equal(state.loop.input.isHeld("w"), false);
  assert.equal(state.loop.input.isHeld("e"), false);
  await state.loop.stop();
});

test("animation frames do not queue overlapping renderer work, and stop waits for the active draw", async () => {
  const draw = deferred();
  let draws = 0;
  let destroyed = false;
  const client = { ...eventClient, submitInput: async () => snapshot(4, 2), submitAction: async () => snapshot() };
  const state = setup(client);
  state.loop = new SessionLoop(client,
    { render: async () => { draws++; if (draws > 1) await draw.promise; }, destroy: async () => { destroyed = true; } },
    { scheduler: state.scheduler });
  await state.loop.start(snapshot());
  state.scheduler.frame();
  await flush();
  state.scheduler.frame();
  state.scheduler.frame();
  assert.equal(draws, 2); // one initial frame and one active animation draw; later frames skip the busy draw.
  const stopping = state.loop.stop();
  await flush();
  assert.equal(destroyed, false);
  draw.resolve();
  await stopping;
  assert.equal(destroyed, true);
});

test("stop cancels an in-flight scene load before waiting for the renderer", async () => {
  const loading = deferred();
  let draws = 0;
  let cancellations = 0;
  let destroyed = false;
  const client = { ...eventClient, submitInput: async () => snapshot(4, 2), submitAction: async () => snapshot() };
  const state = setup(client);
  state.loop = new SessionLoop(client, {
    render: async () => { draws++; if (draws > 1) await loading.promise; },
    cancel: () => { cancellations++; loading.resolve(); },
    destroy: async () => { destroyed = true; },
  }, { scheduler: state.scheduler });
  await state.loop.start(snapshot());
  state.scheduler.frame();
  await flush();
  assert.equal(draws, 2);
  await state.loop.stop("unload");
  assert.equal(cancellations, 1);
  assert.equal(destroyed, true);
});

test("F invokes the snapshot-backed scene interaction hook, while pause and resume update the HUD state", async () => {
  const actions = [];
  const interactions = [];
  const pauses = [];
  const client = { ...eventClient, submitInput: async () => snapshot(4, 2), submitAction: async action => { actions.push(action); return snapshot(); } };
  const state = setup(client, { onInteract: value => interactions.push(value), onPauseState: value => pauses.push(value) });
  await state.loop.start(snapshot());
  assert.equal(state.loop.keyDown("f"), null);
  assert.equal(interactions.length, 1);
  assert.equal(interactions[0].worldEpoch, 4);
  assert.deepEqual(actions, [], "F does not use the unimplemented generic interact action");
  await state.loop.pause();
  assert.equal(state.loop.isPaused, true);
  assert.equal(state.loop.keyDown("f"), null);
  assert.equal(interactions.length, 1, "paused input cannot interact");
  await state.loop.resume();
  assert.equal(state.loop.isPaused, false);
  assert.deepEqual(pauses, ["pausing", "paused", "resuming", "running"]);
  await state.loop.stop();
});

test("paused save is serialized before resume; unload ignores its late receipt", async () => {
  const save = deferred();
  const calls = [];
  const client = { ...eventClient,
    pause: async () => { calls.push("pause"); return snapshot(4, 2); },
    resume: async () => { calls.push("resume"); return snapshot(4, 4); },
  };
  const state = setup(client);
  await state.loop.start(snapshot());
  await state.loop.pause();
  await flush();
  const saved = state.loop.runWhilePaused(() => save.promise);
  const resumed = state.loop.resume();
  await flush();
  assert.deepEqual(calls, ["pause"]);
  save.resolve("saved");
  assert.equal(await saved, "saved");
  await resumed;
  assert.deepEqual(calls, ["pause", "resume"]);
  await state.loop.stop();

  const lateSave = deferred();
  const second = setup(client);
  await second.loop.start(snapshot());
  await second.loop.pause();
  const late = second.loop.runWhilePaused(() => lateSave.promise);
  const stopping = second.loop.stop("unload");
  lateSave.resolve("too-late");
  await assert.rejects(late, /E_SAVE_SESSION_CHANGED/);
  await stopping;
});

test("pause atomically neutralizes guard before pending input settles; only receipt confirms pause", async () => {
  const guardStart = deferred();
  const pauseReceipt = deferred();
  const resumeReceipt = deferred();
  const calls = [], contexts = [], statuses = [];
  const client = { ...eventClient,
    submitAction: action => { calls.push(action.kind); return guardStart.promise; },
    pause: context => { calls.push("formal_pause"); contexts.push(context); return pauseReceipt.promise; },
    resume: context => { calls.push("formal_resume"); contexts.push(context); return resumeReceipt.promise; },
  };
  const state = setup(client, { onPauseState: value => statuses.push(value) });
  await state.loop.start(snapshot());
  state.loop.keyDown("e");
  const pause = state.loop.pause();
  assert.equal(state.scheduler.intervals.size, 0);
  assert.equal(state.loop.input.isHeld("e"), false);
  assert.deepEqual(calls, ["guardStart", "formal_pause"], "pause never waits for guard IPC");
  assert.equal(statuses.includes("paused"), false);
  pauseReceipt.resolve(snapshot(4, 4));
  await pause;
  assert.equal(statuses.at(-1), "paused");
  const resume = state.loop.resume();
  await flush();
  assert.equal(calls.includes("formal_resume"), false, "resume keeps prior command ordering");
  guardStart.reject(new Error("E_RUNTIME_PAUSED"));
  await flush();
  assert.deepEqual(calls, ["guardStart", "formal_pause", "formal_resume"]);
  assert.equal(statuses.at(-1), "resuming");
  resumeReceipt.resolve(snapshot(4, 5));
  await resume;
  assert.equal(statuses.at(-1), "running");
  assert.equal(state.scheduler.intervals.size, 1);
  assert.deepEqual(contexts, Array(2).fill({ worldId: "grey_hive", sceneId: "grey_hive", worldEpoch: 4 }));
  await state.loop.stop();
});

test("rapid pause/resume intents serialize IPC and apply only the final running state", async () => {
  const pauseReceipt = deferred();
  const resumeReceipt = deferred();
  const calls = [];
  const client = { ...eventClient,
    pause: () => { calls.push("pause"); return pauseReceipt.promise; },
    resume: () => { calls.push("resume"); return resumeReceipt.promise; },
  };
  const state = setup(client);
  await state.loop.start(snapshot());
  const pause = state.loop.pause();
  await flush();
  assert.deepEqual(calls, ["pause"]);
  const resume = state.loop.resume();
  assert.deepEqual(calls, ["pause"], "resume cannot overtake pending pause IPC");
  const pauseAgain = state.loop.pause();
  const finalResume = state.loop.resume();
  assert.equal(state.scheduler.intervals.size, 0);
  pauseReceipt.resolve(snapshot(4, 2));
  await flush();
  assert.deepEqual(calls, ["pause", "resume"]);
  assert.equal(state.scheduler.intervals.size, 0);
  resumeReceipt.resolve(snapshot(4, 2));
  await Promise.all([pause, resume, pauseAgain, finalResume]);
  assert.equal(state.loop.isPaused, false);
  assert.equal(state.scheduler.intervals.size, 1);
  await state.loop.stop();
});

test("pause/resume IPC failures remain visible and locally locked until an acknowledged retry", async () => {
  let pauseCalls = 0;
  let resumeCalls = 0;
  const statuses = [];
  const client = { ...eventClient,
    pause: async () => { if (++pauseCalls === 1) throw new Error("ipc pause disconnected"); return snapshot(); },
    resume: async () => { if (++resumeCalls === 1) throw new Error("ipc resume disconnected"); return snapshot(); },
  };
  const state = setup(client, { onPauseState: (status, message) => statuses.push([status, message]) });
  await state.loop.start(snapshot());
  await state.loop.pause();
  assert.equal(state.loop.pausePresentationState, "error");
  assert.equal(state.scheduler.intervals.size, 0);
  assert.match(statuses.at(-1)[1], /ipc pause disconnected/);
  await state.loop.retryPauseState();
  assert.equal(state.loop.pausePresentationState, "paused");

  await state.loop.resume();
  assert.equal(state.loop.pausePresentationState, "error");
  assert.equal(state.scheduler.intervals.size, 0);
  assert.match(statuses.at(-1)[1], /ipc resume disconnected/);
  await state.loop.retryPauseState();
  assert.equal(state.loop.pausePresentationState, "running");
  assert.equal(state.scheduler.intervals.size, 1);
  await state.loop.stop();
});

test("pause preempts concurrent scene interaction and ignores stopped late receipts", async () => {
  const interaction = deferred();
  const pauseReceipt = deferred();
  const calls = [];
  const client = { ...eventClient, pause: () => { calls.push("pause"); return pauseReceipt.promise; } };
  const state = setup(client, { onInteract: async () => { calls.push("interact"); await interaction.promise; } });
  await state.loop.start(snapshot());
  state.loop.keyDown("f");
  const pause = state.loop.pause();
  await flush();
  assert.deepEqual(calls, ["interact", "pause"]);

  interaction.resolve();
  await flush();
  assert.deepEqual(calls, ["interact", "pause"]);
  const stopping = state.loop.stop("hub");
  pauseReceipt.resolve(snapshot(4, 2));
  await Promise.all([pause, stopping]);
  assert.equal(state.loop.snapshots.view().serverTick, 1, "late lifecycle receipt must not replace the stopped session snapshot");
  assert.equal(state.scheduler.intervals.size, 0);
});

test("page unload ignores a late resume receipt and cannot restart the stopped session", async () => {
  const resumeReceipt = deferred();
  const destroyed = { value: false };
  const client = { ...eventClient, resume: () => resumeReceipt.promise };
  const state = setup(client);
  state.loop = new SessionLoop(client, { render: async () => {}, destroy: async () => { destroyed.value = true; } }, { scheduler: state.scheduler });
  await state.loop.start(snapshot());
  await state.loop.pause();
  const resume = state.loop.resume();
  await flush();
  assert.equal(state.scheduler.intervals.size, 0);
  const unload = state.loop.stop("unload");
  resumeReceipt.resolve(snapshot(4, 2));
  await Promise.all([resume, unload]);
  assert.equal(state.scheduler.intervals.size, 0);
  assert.equal(state.scheduler.frames.size, 0);
  assert.equal(state.loop.snapshots.view().serverTick, 1);
  assert.equal(destroyed.value, true);
});

test("a stale epoch pause receipt is rejected without opening local input", async () => {
  const statuses = [];
  const client = { ...eventClient, pause: async () => snapshot(4, 1) };
  const state = setup(client, { onPauseState: (status, message) => statuses.push([status, message]) });
  await state.loop.start(snapshot(4));
  await state.loop.acceptAuthoritativeSnapshot(snapshot(5, 2));
  await state.loop.pause();
  assert.equal(state.loop.pausePresentationState, "error");
  assert.match(statuses.at(-1)[1], /E_LIFECYCLE_RECEIPT_CONTEXT_MISMATCH/);
  assert.equal(state.scheduler.intervals.size, 0);
  await state.loop.stop();
});

test("blur/visibility intents serialize, while focus cannot undo an explicit manual pause", async () => {
  const pauseReceipt = deferred();
  const resumeReceipt = deferred();
  const calls = [];
  const client = { ...eventClient,
    pause: () => { calls.push("pause"); return pauseReceipt.promise; },
    resume: () => { calls.push("resume"); return resumeReceipt.promise; },
  };
  const state = setup(client);
  state.documentTarget.hidden = false;
  await state.loop.start(snapshot());
  state.windowTarget.fire("blur");
  await waitFor(() => calls.includes("pause"));
  assert.equal(state.scheduler.intervals.size, 0);
  state.windowTarget.fire("focus");
  assert.deepEqual(calls, ["pause"], "resume waits behind the pending pause receipt");
  pauseReceipt.resolve(snapshot(4, 2));
  await waitFor(() => calls.includes("resume"));
  assert.equal(state.scheduler.intervals.size, 0);
  resumeReceipt.resolve(snapshot(4, 2));
  await waitFor(() => state.loop.pausePresentationState === "running");
  assert.equal(state.scheduler.intervals.size, 1);

  state.documentTarget.hidden = true;
  state.documentTarget.fire("visibilitychange");
  await waitFor(() => calls.length === 3);
  assert.equal(calls.at(-1), "pause");
  state.documentTarget.hidden = false;
  state.documentTarget.fire("visibilitychange");
  await waitFor(() => calls.length === 4);
  assert.equal(calls.at(-1), "resume");

  await state.loop.pause("manual");
  const callsAfterManualPause = calls.length;
  state.windowTarget.fire("blur");
  state.windowTarget.fire("focus");
  await flush();
  assert.equal(calls.length, callsAfterManualPause, "focus does not undo a manual pause");
  assert.equal(state.loop.pausePresentationState, "paused");
  await state.loop.resume("manual");
  await state.loop.stop();
});

test("Escape pauses and resumes through authoritative receipts without repeat toggles", async () => {
  const pauseReceipt = deferred();
  const calls = [];
  const state = setup({ ...eventClient,
    submitInput: async () => snapshot(), submitAction: async () => snapshot(),
    pause: () => { calls.push("pause"); return pauseReceipt.promise; },
    resume: async () => { calls.push("resume"); return snapshot(4, 3); },
  });
  await state.loop.start(snapshot());
  state.loop.keyDown("w");
  let prevented = 0;
  const escape = { key: "Escape", preventDefault() { prevented++; } };
  state.windowTarget.fire("keydown", escape);
  assert.equal(state.loop.pausePresentationState, "pausing");
  assert.equal(state.scheduler.intervals.size, 0);
  assert.equal(state.loop.input.isHeld("w"), false);
  state.windowTarget.fire("keydown", { ...escape, repeat: true });
  await flush();
  assert.deepEqual(calls, ["pause"]);
  pauseReceipt.resolve(snapshot(4, 2));
  await waitFor(() => state.loop.pausePresentationState === "paused");
  state.windowTarget.fire("keydown", { ...escape, repeat: true });
  assert.equal(state.loop.pausePresentationState, "paused");
  // An Escape already used by the terminal must not resume the journey.
  state.windowTarget.fire("keydown", { ...escape, defaultPrevented: true });
  assert.equal(state.loop.pausePresentationState, "paused");
  // Focus stays on the resume button after mouse interaction; Escape still works.
  state.windowTarget.fire("keydown", { ...escape, target: { closest: () => ({ tagName: "BUTTON" }) } });
  await waitFor(() => state.loop.pausePresentationState === "running");
  assert.deepEqual(calls, ["pause", "resume"]);
  assert.equal(prevented, 4);
  await state.loop.stop();
});

test("text entry, UI controls and paused input retain native keyboard behavior", async () => {
  const actions = [];
  const state = setup({ ...eventClient, submitInput: async () => snapshot(),
    submitAction: async action => { actions.push(action); return snapshot(); },
  });
  await state.loop.start(snapshot());
  for (const target of [
    { closest: () => ({ tagName: "INPUT" }) },
    { closest: () => ({ tagName: "BUTTON" }) },
    { isContentEditable: true },
  ]) {
    for (const key of ["s", "a", "f", "e", " ", "r"]) {
      state.windowTarget.fire("keydown", { key, target,
        preventDefault() { assert.fail(`UI key ${key} was consumed`); } });
    }
  }
  state.windowTarget.fire("keydown", { key: "e", isComposing: true,
    preventDefault() { assert.fail("composition key was consumed"); } });
  assert.equal(state.loop.input.isHeld("e"), false);
  assert.deepEqual(actions, []);
  await state.loop.pause();
  for (const key of ["s", " "]) {
    state.windowTarget.fire("keydown", { key,
      preventDefault() { assert.fail(`paused key ${key} was consumed`); } });
  }
  assert.deepEqual(actions, []);
  await state.loop.stop();
});

test("Escape retries a failed pause instead of reopening gameplay", async () => {
  let pauseCalls = 0;
  let resumeCalls = 0;
  const state = setup({ ...eventClient,
    submitInput: async () => snapshot(), submitAction: async () => snapshot(),
    pause: async () => { if (++pauseCalls === 1) throw new Error("pause transport unavailable"); return snapshot(4, 2); },
    resume: async () => { resumeCalls++; return snapshot(4, 3); },
  });
  await state.loop.start(snapshot());
  state.windowTarget.fire("keydown", { key: "Escape", preventDefault() {} });
  await waitFor(() => state.loop.pausePresentationState === "error");
  state.windowTarget.fire("keydown", { key: "Escape", preventDefault() {} });
  await waitFor(() => state.loop.pausePresentationState === "paused");
  assert.equal(pauseCalls, 2);
  assert.equal(resumeCalls, 0);
  assert.equal(state.scheduler.intervals.size, 0);
  await state.loop.stop();
});


test("mode-neutral Build IPC participates in Return and following save ordering", async () => {
  const pending = deferred(); const order=[];
  const state=setup({ ...eventClient, submitInput:async()=>snapshot(4,2), submitAction:async()=>snapshot(4,2) });
  await state.loop.start(snapshot());
  const command=state.loop.runWithSession(async()=>{ order.push("equip-start"); await pending.promise; order.push("equip-committed"); return "receipt"; });
  const rejected=assert.rejects(command,/E_BUILD_SESSION_CHANGED/);
  await waitFor(()=>order.includes("equip-start"));
  const returning=(async()=>{await state.loop.stop("hub");order.push("return-save");})();
  await flush(); assert.deepEqual(order,["equip-start"]); assert.equal(state.destroyed.value,false);
  pending.resolve(); await rejected; await returning;
  assert.deepEqual(order,["equip-start","equip-committed","return-save"]); assert.equal(state.destroyed.value,true);
});

test("mode-neutral Build commands allow paused execution and serialize save/resume", async () => {
  const pending=deferred();const order=[];
  const state=setup({...eventClient,pause:async()=>snapshot(4,2),resume:async()=>{order.push("resume");return snapshot(4,3);}});
  await state.loop.start(snapshot());await state.loop.pause();
  const command=state.loop.runWithSession(async()=>{order.push("equip-start");await pending.promise;order.push("equip-end");return 1;});
  await waitFor(()=>order.includes("equip-start"));
  const saving=state.loop.runWhilePaused(async()=>{order.push("save");return 2;});
  const resumed=state.loop.resume();await flush();assert.deepEqual(order,["equip-start"]);
  pending.resolve();assert.equal(await command,1);assert.equal(await saving,2);await resumed;
  assert.deepEqual(order,["equip-start","equip-end","save","resume"]);await state.loop.stop();
});

test("mode-neutral failure unblocks stop and same-turn stop prevents dispatch", async () => {
  const pending=deferred();let dispatched=false;
  const state=setup({...eventClient});await state.loop.start(snapshot());
  const command=state.loop.runWithSession(()=>{dispatched=true;return pending.promise;});
  const rejected=assert.rejects(command,/transport failed/);await waitFor(()=>dispatched);
  const stopped=state.loop.stop();pending.reject(new Error("transport failed"));await rejected;await stopped;
  const second=setup({...eventClient});await second.loop.start(snapshot());let called=false;
  const raced=second.loop.runWithSession(async()=>{called=true;});const denied=assert.rejects(raced,/E_BUILD_SESSION_CHANGED/);
  const stop=second.loop.stop();await denied;await stop;assert.equal(called,false);
});

test('native pointer input reprojects a stationary cursor from each committed renderer pose', async () => {
  const samples=[];let tick=1;const pointerTarget=new EventSurface();
  pointerTarget.getBoundingClientRect=()=>({left:0,top:0,width:800,height:600});
  const f=setup({submitInput:async input=>{samples.push(input);return snapshot(++tick);},
    submitAction:async()=>snapshot(tick),pause:async()=>snapshot(++tick),resume:async()=>snapshot(++tick)}, {pointerTarget});
  let committed={x:400,y:300};let available=false;
  f.loop.renderer.pointerAim=(_snapshot,x,y)=>available?{x:x-committed.x,y:y-committed.y}:null;
  await f.loop.start(snapshot(tick));
  pointerTarget.fire('pointermove',{clientX:500,clientY:300});f.scheduler.tick();await flush();
  assert.equal(samples.at(-1).aimX,0);assert.equal(samples.at(-1).aimZ,0,'no displayed aim frame means no synthetic steering');
  available=true;f.scheduler.tick();await flush();const first=samples.at(-1);
  committed={x:400,y:250};f.scheduler.tick();await flush();const second=samples.at(-1);
  assert.notDeepEqual([first.aimX,first.aimZ],[second.aimX,second.aimZ],'fixed mouse is reprojected when the actually drawn pose moves');
  await f.loop.pause();const count=samples.length;
  pointerTarget.fire('pointermove',{clientX:0,clientY:0});f.scheduler.tick();await flush();assert.equal(samples.length,count);
  await f.loop.stop();
});
