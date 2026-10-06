import test from "node:test";
import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { setImmediate as nextTurn } from "node:timers/promises";
import { cleanupBrowserResources, removeOwnedProfile, watchBrowserProcess, withBrowserCleanup } from "./support/browser-cleanup.mjs";

function fixture({ exited = false } = {}) {
  const events = [];
  const child = new EventEmitter();
  Object.assign(child, {
    pid: 12345,
    exitCode: exited ? 0 : null,
    signalCode: null,
    kill(signal) { events.push(signal); return true; },
    exit() { this.exitCode = 0; events.push("exit"); this.emit("exit", 0, null); },
  });
  const browserState = watchBrowserProcess(child);
  const resources = {
    browserState,
    ownedProfile: "/owned/accessibility-browser-test",
    cdp: {
      async send(method) { events.push(method); },
      async close() { events.push("socket.close"); },
    },
    server: {
      listening: true,
      close(callback) { events.push("server.close"); this.listening = false; callback(); },
      closeAllConnections() { events.push("server.closeAllConnections"); },
    },
  };
  const options = {
    cdpCloseTimeoutMs: 10,
    gracefulExitTimeoutMs: 20,
    terminateExitTimeoutMs: 30,
    killExitTimeoutMs: 40,
    serverCloseTimeoutMs: 50,
    profileRemoval: {
      async remove(profile, removeOptions) {
        assert.equal(profile, resources.ownedProfile);
        assert.deepEqual(removeOptions, { recursive: true, force: true, maxRetries: 0 });
        assert.equal(browserState.hasExited(), true, "profile cannot be removed while the spawned browser is alive");
        events.push("rm");
      },
      async sleep(ms) { events.push(`retry:${ms}`); },
    },
  };
  return { child, resources, options, events };
}

test("graceful shutdown waits for delayed browser exit before removing its owned profile", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { child, resources, options, events } = fixture();
  let complete = false;
  const cleaning = cleanupBrowserResources(resources, options).then(() => { complete = true; });
  await nextTurn();
  assert.deepEqual(events, ["Browser.close"]);
  assert.equal(complete, false);
  child.exit();
  await cleaning;
  assert.deepEqual(events, ["Browser.close", "exit", "socket.close", "server.close", "server.closeAllConnections", "rm"]);
  assert.equal(child.listenerCount("exit"), 0);
  assert.equal(child.listenerCount("error"), 0);
});

test("SIGTERM fallback also waits for a delayed exit before profile removal", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { child, resources, options, events } = fixture();
  const cleaning = cleanupBrowserResources(resources, options);
  await nextTurn();
  t.mock.timers.tick(options.gracefulExitTimeoutMs);
  await nextTurn();
  assert.deepEqual(events, ["Browser.close", "SIGTERM"]);
  child.exit();
  await cleaning;
  assert.ok(events.indexOf("rm") > events.indexOf("exit"));
  assert.ok(!events.includes("SIGKILL"));
});

test("unresponsive graceful CDP close is bounded, reported, and followed by owned-child termination", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { child, resources, options, events } = fixture();
  resources.cdp.send = () => new Promise(() => {});
  child.kill = signal => { events.push(signal); child.exit(); return true; };
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.ok(error instanceof AggregateError);
    assert.deepEqual(error.errors.map(e => e.code), ["E_CDP_BROWSER_CLOSE_TIMEOUT"]);
    return true;
  });
  await nextTurn();
  t.mock.timers.tick(options.cdpCloseTimeoutMs);
  await nextTurn();
  assert.deepEqual(events, []);
  t.mock.timers.tick(options.gracefulExitTimeoutMs);
  await cleaning;
  assert.deepEqual(events, ["SIGTERM", "exit", "socket.close", "server.close", "server.closeAllConnections", "rm"]);
});

test("SIGKILL escalation targets the same spawned ChildProcess and awaits its exit", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { child, resources, options, events } = fixture();
  resources.cdp = undefined;
  child.kill = signal => {
    events.push(signal);
    if (signal === "SIGKILL") child.exit();
    return true;
  };
  const cleaning = cleanupBrowserResources(resources, options);
  await nextTurn();
  assert.deepEqual(events, ["SIGTERM"]);
  t.mock.timers.tick(options.terminateExitTimeoutMs);
  await cleaning;
  assert.deepEqual(events, ["SIGTERM", "SIGKILL", "exit", "server.close", "server.closeAllConnections", "rm"]);
});

test("a browser that never exits retains its profile and reports bounded cleanup failure", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { resources, options, events } = fixture();
  resources.cdp = undefined;
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.ok(error instanceof AggregateError);
    assert.equal(error.errors[0].code, "E_BROWSER_EXIT_TIMEOUT");
    assert.ok(error.errors[0].message.includes(resources.ownedProfile));
    return true;
  });
  await nextTurn();
  t.mock.timers.tick(options.terminateExitTimeoutMs);
  await nextTurn();
  t.mock.timers.tick(options.killExitTimeoutMs);
  await cleaning;
  assert.deepEqual(events, ["SIGTERM", "SIGKILL", "server.close", "server.closeAllConnections"]);
});

for (const code of ["EBUSY", "EPERM", "ENOTEMPTY"]) {
  test(`transient ${code} profile locks use bounded backoff and eventually remove the profile`, async () => {
    let attempts = 0;
    const waits = [];
    await removeOwnedProfile("/owned/accessibility-browser-test", {
      async remove(profile) {
        assert.equal(profile, "/owned/accessibility-browser-test");
        if (++attempts < 3) throw Object.assign(new Error("transient profile lock"), { code });
      },
      async sleep(ms) { waits.push(ms); },
      attempts: 5,
      retryDelayMs: 100,
    });
    assert.equal(attempts, 3);
    assert.deepEqual(waits, [100, 200]);
  });
}

test("persistent EBUSY exhausts the exact retry budget and preserves the original error", async () => {
  const busy = Object.assign(new Error("profile still locked"), { code: "EBUSY" });
  let attempts = 0;
  const waits = [];
  await assert.rejects(removeOwnedProfile("/owned/accessibility-browser-test", {
    async remove() { attempts++; throw busy; },
    async sleep(ms) { waits.push(ms); },
  }), error => error === busy);
  assert.equal(attempts, 5);
  assert.deepEqual(waits, [100, 200, 300, 400]);
});

test("non-transient profile errors are not retried or swallowed", async () => {
  const denied = Object.assign(new Error("profile permission denied"), { code: "EACCES" });
  let attempts = 0;
  await assert.rejects(removeOwnedProfile("/owned/accessibility-browser-test", {
    async remove() { attempts++; throw denied; },
    async sleep() { assert.fail("non-transient errors must not sleep or retry"); },
  }), error => error === denied);
  assert.equal(attempts, 1);
});

test("a body assertion and persistent cleanup EBUSY remain inspectable together", async () => {
  const { resources, options, events } = fixture({ exited: true });
  const bodyError = new assert.AssertionError({ message: "scale assertion failed", actual: 115, expected: 130 });
  const busy = Object.assign(new Error("profile still locked"), { code: "EBUSY" });
  let attempts = 0;
  options.profileRemoval.remove = async () => { attempts++; throw busy; };
  await assert.rejects(withBrowserCleanup(
    async () => { throw bodyError; },
    () => cleanupBrowserResources(resources, options),
  ), error => {
    assert.ok(error instanceof AggregateError);
    assert.equal(error.cause, bodyError);
    assert.equal(error.errors[0], bodyError);
    assert.ok(error.errors[1] instanceof AggregateError);
    assert.equal(error.errors[1].errors[0], busy);
    return true;
  });
  assert.equal(attempts, 5);
  assert.ok(events.includes("server.close"));
});

test("body-only failure preserves its original identity after successful cleanup", async () => {
  const bodyError = new Error("original assertion");
  let cleaned = false;
  await assert.rejects(withBrowserCleanup(async () => { throw bodyError; }, async () => { cleaned = true; }), error => error === bodyError);
  assert.equal(cleaned, true);
  assert.equal(await withBrowserCleanup(async () => 42, async () => {}), 42);
});

test("cleanup-only failure is never converted into a passing body result", async () => {
  const cleanupError = new Error("cleanup failed");
  await assert.rejects(withBrowserCleanup(async () => "body passed", async () => { throw cleanupError; }), error => error === cleanupError);
});

test("spawn failure is observed without waiting for an exit event that will never arrive", async () => {
  const { child, resources, options, events } = fixture();
  resources.cdp = undefined;
  child.pid = undefined;
  const spawnError = Object.assign(new Error("spawn ENOENT"), { code: "ENOENT" });
  child.emit("error", spawnError);
  assert.equal(resources.browserState.error, spawnError);
  await cleanupBrowserResources(resources, options);
  assert.deepEqual(events, ["server.close", "server.closeAllConnections", "rm"]);
});

test("CDP transport closing before Browser.close acknowledgement is expected only after confirmed exit", async () => {
  const { child, resources, options, events } = fixture();
  resources.cdp.send = async () => {
    child.exit();
    throw Object.assign(new Error("CDP connection closed"), { code: "E_CDP_CLOSED" });
  };
  await cleanupBrowserResources(resources, options);
  assert.ok(events.includes("rm"));
  assert.ok(!events.includes("SIGTERM"));
});

test("a CDP disconnect without graceful process exit is reported even when fallback succeeds", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { child, resources, options, events } = fixture();
  const disconnect = Object.assign(new Error("CDP connection closed"), { code: "E_CDP_CLOSED" });
  resources.cdp.send = async () => { throw disconnect; };
  child.kill = signal => { events.push(signal); child.exit(); return true; };
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => error.errors[0] === disconnect);
  await nextTurn();
  assert.ok(!events.includes("rm"));
  t.mock.timers.tick(options.gracefulExitTimeoutMs);
  await cleaning;
  assert.ok(events.includes("SIGTERM"));
  assert.ok(events.indexOf("rm") > events.indexOf("exit"));
});

test("a genuine CDP failure is retained even when the browser exits", async () => {
  const { child, resources, options, events } = fixture();
  const protocolError = new Error("Browser.close protocol failed");
  resources.cdp.send = async () => { child.exit(); throw protocolError; };
  await assert.rejects(cleanupBrowserResources(resources, options), error => error.errors[0] === protocolError);
  assert.ok(events.includes("rm"));
});

test("socket-close timeout and server-close failure are both retained while profile cleanup still runs", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { resources, options, events } = fixture({ exited: true });
  const serverError = new Error("server close failed");
  resources.cdp.close = () => new Promise(() => {});
  resources.server.close = callback => callback(serverError);
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.equal(error.errors[0].code, "E_CDP_SOCKET_CLOSE_TIMEOUT");
    assert.equal(error.errors[1], serverError);
    return true;
  });
  await nextTurn();
  t.mock.timers.tick(options.cdpCloseTimeoutMs);
  await cleaning;
  assert.ok(events.includes("rm"));
});

test("server-close timeout is bounded and does not suppress safe profile removal", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { resources, options, events } = fixture({ exited: true });
  resources.server.close = () => {};
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => error.errors[0].code === "E_BROWSER_SERVER_CLOSE_TIMEOUT");
  await nextTurn();
  t.mock.timers.tick(options.serverCloseTimeoutMs);
  await cleaning;
  assert.ok(events.includes("server.closeAllConnections"));
  assert.ok(events.includes("rm"));
});

test("a failed owned-child termination is reported even if later escalation exits", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  const { child, resources, options, events } = fixture();
  resources.cdp = undefined;
  child.kill = signal => {
    events.push(signal);
    if (signal === "SIGKILL") { child.exit(); return true; }
    return false;
  };
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => error.errors[0].code === "E_BROWSER_TERMINATE");
  await nextTurn();
  t.mock.timers.tick(options.terminateExitTimeoutMs);
  await cleaning;
  assert.deepEqual(events.slice(0, 3), ["SIGTERM", "SIGKILL", "exit"]);
  assert.ok(events.includes("rm"));
});
