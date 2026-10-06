import test from "node:test";
import assert from "node:assert/strict";
import { setImmediate as nextTurn } from "node:timers/promises";
import { createViteSessionResource } from "./support/vite-session-resource.mjs";
import { closeVite } from "./support/browser-session.mjs";
import { cleanupBrowserResources, withBrowserCleanup } from "./support/browser-cleanup.mjs";

function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

function fakeTimers(t) {
  t.mock.timers.enable({ apis: ["Date", "setTimeout"], now: 1000 });
  t.after(() => t.mock.timers.reset());
}

function fixture(factory, { timeoutMs = 20, now = Date.now } = {}) {
  const events = [], consumers = [];
  let closes = 0;
  const server = {
    listen: async () => { events.push("listen"); },
    close: async () => { closes++; events.push("actual-close"); },
  };
  const register = resource => {
    events.push("register");
    consumers.push({ close: timeout => closeVite(resource, undefined, Math.min(30, timeout)) });
  };
  const resource = createViteSessionResource(() => {
    events.push("factory");
    return factory ? factory(server) : server;
  }, register, { timeoutMs, now });
  const resources = {
    browserState: { hasExited: async () => true, dispose: async () => { events.push("dispose"); } },
    ownedProfile: "/mock-only/vite-retention",
    otherResources: consumers,
    profileVerifier: async () => { events.push("verify"); },
  };
  const options = { profileRemoval: { remove: async () => { events.push("remove"); } } };
  return { resource, server, events, consumers, register, resources, options, get closes() { return closes; } };
}

function retained(f) {
  assert.equal(f.events.includes("verify"), false);
  assert.equal(f.events.includes("remove"), false);
}

test("register precedes factory and successful ready/listen/close permits F01 removal", async () => {
  const f = fixture(undefined, { timeoutMs: 15000 });
  assert.deepEqual(f.events, ["register"]);
  assert.equal(f.resource.state.creation, "pending");
  assert.equal(await f.resource.ready(), f.resource);
  await f.resource.listen();
  await cleanupBrowserResources(f.resources, f.options);
  assert.deepEqual(f.events, ["register", "factory", "listen", "actual-close", "verify", "remove", "dispose"]);
  assert.equal(f.closes, 1);
  assert.equal(f.resource.state.closed, true);
  assert.equal(f.resource.state.close, "settled");
});

test("registration failure prevents factory allocation and preserves registration error", async () => {
  const original = new Error("register failed");
  let calls = 0, registered;
  assert.throws(() => createViteSessionResource(() => { calls++; }, resource => { registered = resource; throw original; }), error => error === original);
  await nextTurn();
  assert.equal(calls, 0);
  await assert.rejects(registered.close(), error => error === original);
});

test("create timeout remains registered through bounded cleanup and closes late result without listen or deletion", async t => {
  fakeTimers(t);
  const creation = deferred();
  const f = fixture(() => creation.promise);
  const ready = assert.rejects(f.resource.ready(), error => error.code === "E_VITE_CREATE_TIMEOUT");
  await nextTurn();
  t.mock.timers.tick(20);
  await ready;
  assert.equal(f.consumers.length, 1);
  assert.equal(f.resource.state.closeRequested, true);
  const cleaning = assert.rejects(cleanupBrowserResources(f.resources, f.options), error => error.errors[0].code === "E_VITE_CLOSE_TIMEOUT");
  await nextTurn();
  t.mock.timers.tick(30);
  await cleaning;
  retained(f);
  assert.equal(f.resource.state.creation, "pending");
  assert.equal(f.resource.state.closed, false);
  creation.resolve(f.server);
  await f.resource.close();
  await assert.rejects(f.resource.ready(), error => error.code === "E_VITE_CLOSING");
  await assert.rejects(f.resource.listen(), error => error.code === "E_VITE_CLOSING");
  assert.equal(f.closes, 1);
  assert.equal(f.events.includes("listen"), false);
  retained(f);
});

test("late factory rejection is observed and accessible with body timeout and cleanup error", async t => {
  fakeTimers(t);
  const creation = deferred();
  const original = new Error("late factory rejection");
  const f = fixture(() => creation.promise);
  let combined;
  const running = assert.rejects(withBrowserCleanup(
    () => f.resource.ready(), () => cleanupBrowserResources(f.resources, f.options),
  ), error => {
    combined = error;
    assert.equal(error.cause.code, "E_VITE_CREATE_TIMEOUT");
    assert.equal(error.errors[0], error.cause);
    assert.equal(error.errors[1].errors[0], original);
    return true;
  });
  await nextTurn();
  t.mock.timers.tick(20);
  await nextTurn();
  creation.reject(original);
  await running;
  assert.ok(combined instanceof AggregateError);
  assert.equal(f.resource.state.creationError, original);
  await assert.rejects(f.resource.close(), error => error === original);
  assert.equal(f.closes, 0);
  retained(f);
});

test("factory rejecting after bounded cleanup remains retained and no later automatic deletion occurs", async t => {
  fakeTimers(t);
  const creation = deferred();
  const original = new Error("factory failed after cleanup finished");
  const f = fixture(() => creation.promise);
  const ready = assert.rejects(f.resource.ready(), error => error.code === "E_VITE_CREATE_TIMEOUT");
  await nextTurn(); t.mock.timers.tick(20); await ready;
  const cleaning = assert.rejects(cleanupBrowserResources(f.resources, f.options), error => error.errors[0].code === "E_VITE_CLOSE_TIMEOUT");
  await nextTurn(); t.mock.timers.tick(30); await cleaning;
  creation.reject(original);
  await nextTurn();
  await assert.rejects(f.resource.close(), error => error === original);
  assert.equal(f.resource.state.closeError, original);
  retained(f);
});

test("listen timeout holds closure pending until late listen settles then actually closes once", async t => {
  fakeTimers(t);
  const listening = deferred();
  const f = fixture();
  f.server.listen = () => { f.events.push("listen"); return listening.promise; };
  await f.resource.ready();
  const listen = assert.rejects(f.resource.listen(), error => error.code === "E_VITE_START_TIMEOUT");
  await nextTurn(); t.mock.timers.tick(20); await listen;
  await assert.rejects(f.resource.listen(), error => error.code === "E_VITE_CLOSING");
  const cleaning = assert.rejects(cleanupBrowserResources(f.resources, f.options), error => error.errors[0].code === "E_VITE_CLOSE_TIMEOUT");
  await nextTurn(); t.mock.timers.tick(30); await cleaning;
  assert.equal(f.closes, 0);
  assert.equal(f.resource.state.listen, "pending");
  assert.equal(f.resource.state.closed, false);
  retained(f);
  listening.resolve();
  await f.resource.close();
  assert.equal(f.closes, 1);
  assert.equal(f.resource.state.listen, "settled");
  assert.equal(f.resource.state.closed, true);
  await assert.rejects(f.resource.listen(), error => error.code === "E_VITE_CLOSING");
  assert.equal(f.events.filter(event => event === "listen").length, 1);
  retained(f);
});

test("listen error and actual close error preserve both identities and cause through body/cleanup wrapper", async () => {
  const listenError = new Error("listen rejected"), closeError = new Error("close rejected");
  const f = fixture(undefined, { timeoutMs: 15000 });
  f.server.listen = async () => { throw listenError; };
  f.server.close = async () => { f.events.push("failed-close"); throw closeError; };
  await f.resource.ready();
  await assert.rejects(withBrowserCleanup(
    () => f.resource.listen(), () => cleanupBrowserResources(f.resources, f.options),
  ), error => {
    assert.equal(error.errors[0], listenError);
    assert.equal(error.cause, listenError);
    const closureError = error.errors[1].errors[0];
    assert.deepEqual(closureError.errors, [listenError, closeError]);
    assert.equal(closureError.cause, listenError);
    return true;
  });
  retained(f);
});

test("concurrent and serial close calls and session-style double registration close actual Vite once", async () => {
  const f = fixture(undefined, { timeoutMs: 15000 });
  await f.resource.ready();
  f.register(f.resource);
  await f.resource.listen();
  const first = f.resource.close(), second = f.resource.close();
  assert.equal(first, second);
  await Promise.all([first, second]);
  assert.equal(f.resource.close(), first);
  await cleanupBrowserResources(f.resources, f.options);
  assert.equal(f.closes, 1);
  assert.deepEqual(f.events.slice(-3), ["verify", "remove", "dispose"]);
});

test("double registration cannot replace the first failed close with a passing second close", async () => {
  const original = new Error("actual close failed");
  const f = fixture(undefined, { timeoutMs: 15000 });
  let calls = 0;
  f.server.close = async () => { calls++; throw original; };
  await f.resource.ready(); f.register(f.resource);
  const first = f.resource.close();
  assert.equal(f.resource.close(), first);
  await assert.rejects(first, error => error === original);
  await assert.rejects(cleanupBrowserResources(f.resources, f.options), error => {
    assert.deepEqual(error.errors, [original, original]);
    return true;
  });
  assert.equal(calls, 1);
  retained(f);
});

test("injected clock deadline rejects a settled factory result outside the ready window", async () => {
  let clock = 1000;
  const f = fixture(server => { clock = 1020; return server; }, { now: () => clock });
  await assert.rejects(f.resource.ready(), error => error.code === "E_VITE_CREATE_TIMEOUT");
  await f.resource.close();
  assert.equal(f.closes, 1);
  assert.equal(f.events.includes("listen"), false);
});

test("close requested during pending creation prevents a waiting listen from allocating a listener", async () => {
  const creation = deferred();
  const f = fixture(() => creation.promise, { timeoutMs: 15000 });
  const listening = assert.rejects(f.resource.listen(), error => error.code === "E_VITE_CLOSING");
  await nextTurn();
  const closing = assert.rejects(f.resource.close(), error => error.code === "E_VITE_CLOSING");
  creation.resolve(f.server);
  await Promise.all([listening, closing]);
  assert.equal(f.closes, 1);
  assert.equal(f.events.includes("listen"), false);
});

test("actual close still pending cannot authorize profile removal even after creation and listen settle", async t => {
  fakeTimers(t);
  const closing = deferred();
  const f = fixture();
  f.server.close = () => { f.events.push("close-start"); return closing.promise; };
  await f.resource.ready(); await f.resource.listen();
  const cleaning = assert.rejects(cleanupBrowserResources(f.resources, f.options), error => error.errors[0].code === "E_VITE_CLOSE_TIMEOUT");
  await nextTurn(); t.mock.timers.tick(30); await cleaning;
  assert.equal(f.resource.state.closed, false);
  retained(f);
  closing.resolve(); await f.resource.close();
  assert.equal(f.resource.state.closed, true);
  retained(f);
});
