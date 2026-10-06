import test from "node:test";
import assert from "node:assert/strict";
import { setImmediate as nextTurn } from "node:timers/promises";
import { bounded, cleanupBrowserResources, withBrowserCleanup } from "./support/browser-cleanup.mjs";

function fixture({ exited = true } = {}) {
  const events = [];
  const resources = {
    ownedProfile: "/mock-only/profile-consumer-gate",
    browserState: {
      hasExited: async () => exited,
      waitForExit: async () => exited,
      terminate: async signal => { events.push(signal); },
      dispose: async () => { events.push("dispose"); },
    },
    otherResources: [],
    profileVerifier: async profile => {
      assert.equal(profile, resources.ownedProfile);
      events.push("verify");
    },
  };
  const options = {
    profileRemoval: {
      remove: async (profile, removalOptions) => {
        assert.equal(profile, resources.ownedProfile);
        assert.deepEqual(removalOptions, { recursive: true, force: true, maxRetries: 0 });
        events.push("remove");
      },
      sleep: async () => { assert.fail("mock removal should not retry"); },
    },
  };
  return { resources, options, events };
}

function retained(events) {
  assert.equal(events.filter(event => event === "verify").length, 0);
  assert.equal(events.filter(event => event === "remove").length, 0);
  assert.equal(events.filter(event => event === "dispose").length, 1);
}

test("exited browser retains profile when a registered consumer rejects, preserving identity and cause", async () => {
  const { resources, options, events } = fixture();
  const cause = new Error("worker close failed");
  const consumerError = new Error("consumer close failed", { cause });
  resources.otherResources = [{ close: async () => { events.push("consumer"); throw consumerError; } }];
  await assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.ok(error instanceof AggregateError);
    assert.deepEqual(error.errors, [consumerError]);
    assert.equal(error.errors[0], consumerError);
    assert.equal(error.errors[0].cause, cause);
    return true;
  });
  assert.deepEqual(events, ["consumer", "dispose"]);
  retained(events);
});

test("one failed consumer cannot be overridden by a later successful consumer", async () => {
  const { resources, options, events } = fixture();
  const consumerError = new Error("first consumer failed");
  resources.otherResources = [
    { close: async () => { events.push("first"); throw consumerError; } },
    { close: async () => { events.push("second"); } },
  ];
  await assert.rejects(cleanupBrowserResources(resources, options), error => error.errors[0] === consumerError);
  assert.deepEqual(events, ["first", "second", "dispose"]);
  retained(events);
});

test("all consumer failures and disposal failure remain inspectable", async () => {
  const { resources, options, events } = fixture();
  const first = new Error("first close");
  const second = new Error("second close");
  const disposal = new Error("dispose");
  resources.otherResources = [
    { close: () => { events.push("first"); throw first; } },
    { close: async () => { events.push("second"); throw second; } },
  ];
  resources.browserState.dispose = () => { events.push("dispose"); throw disposal; };
  await assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.deepEqual(error.errors, [first, second, disposal]);
    return true;
  });
  assert.deepEqual(events, ["first", "second", "dispose"]);
  retained(events);
});

test("bounded consumer timeout retains profile even after its underlying close completes late", async t => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  t.after(() => t.mock.timers.reset());
  const { resources, options, events } = fixture();
  let finish;
  const pending = new Promise(resolve => { finish = resolve; });
  resources.otherResources = [
    { close: timeout => bounded(async () => { events.push("consumer"); await pending; events.push("late-close"); }, Math.min(25, timeout), "E_VITE_CLOSE_TIMEOUT") },
    { close: async () => { events.push("second"); } },
  ];
  const cleaning = assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.deepEqual(error.errors.map(member => member.code), ["E_VITE_CLOSE_TIMEOUT"]);
    return true;
  });
  await nextTurn();
  assert.deepEqual(events, ["consumer"]);
  t.mock.timers.tick(25);
  await cleaning;
  assert.deepEqual(events, ["consumer", "second", "dispose"]);
  retained(events);
  finish();
  await nextTurn();
  assert.deepEqual(events, ["consumer", "second", "dispose", "late-close"]);
  retained(events);
});

test("exhausted cleanup budget leaves every consumer unconfirmed and skips verifier and removal", async t => {
  t.mock.timers.enable({ apis: ["Date"], now: 1000 });
  t.after(() => t.mock.timers.reset());
  const { resources, options, events } = fixture();
  options.deadline = 1000;
  resources.otherResources = [
    { close: async () => { assert.fail("first close cannot start without budget"); } },
    { close: async () => { assert.fail("second close cannot start without budget"); } },
  ];
  await assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.deepEqual(error.errors.map(member => member.code), ["E_CLEANUP_BUDGET", "E_CLEANUP_BUDGET"]);
    return true;
  });
  assert.deepEqual(events, ["dispose"]);
  retained(events);
});

test("budget consumed by an earlier close does not confirm the remaining consumer", async t => {
  t.mock.timers.enable({ apis: ["Date"], now: 1000 });
  t.after(() => t.mock.timers.reset());
  const { resources, options, events } = fixture();
  options.deadline = 1010;
  resources.otherResources = [
    { close: async timeout => { assert.equal(timeout, 10); events.push("first"); t.mock.timers.tick(10); } },
    { close: async () => { assert.fail("remaining consumer cannot start without budget"); } },
  ];
  await assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.deepEqual(error.errors.map(member => member.code), ["E_CLEANUP_BUDGET"]);
    return true;
  });
  assert.deepEqual(events, ["first", "dispose"]);
  retained(events);
});

test("all registered consumers close completely before profile verification and removal", async () => {
  const { resources, options, events } = fixture();
  resources.otherResources = [
    { close: async timeout => { assert.ok(timeout > 0 && timeout <= 4500); events.push("first-start"); await nextTurn(); events.push("first-closed"); } },
    { close: async () => { events.push("second-start"); await nextTurn(); events.push("second-closed"); } },
  ];
  await cleanupBrowserResources(resources, options);
  assert.deepEqual(events, ["first-start", "first-closed", "second-start", "second-closed", "verify", "remove", "dispose"]);
});

test("without registered consumers the original safe profile removal flow is preserved", async () => {
  const { resources, options, events } = fixture();
  delete resources.otherResources;
  await cleanupBrowserResources(resources, options);
  assert.deepEqual(events, ["verify", "remove", "dispose"]);
});

test("successful consumers cannot permit removal while browser exit remains unconfirmed", async () => {
  const { resources, options, events } = fixture({ exited: false });
  resources.otherResources = [{ close: async () => { events.push("consumer"); } }];
  await assert.rejects(cleanupBrowserResources(resources, options), error => {
    assert.deepEqual(error.errors.map(member => member.code), ["E_BROWSER_EXIT_TIMEOUT"]);
    return true;
  });
  assert.deepEqual(events, ["SIGTERM", "SIGKILL", "consumer", "dispose"]);
  retained(events);
});

test("body assertion and registered consumer cleanup failure are both preserved by the real wrapper", async () => {
  const { resources, options, events } = fixture();
  const bodyError = new assert.AssertionError({ message: "body assertion failed", actual: 1, expected: 2 });
  const consumerError = new Error("consumer cleanup failed");
  resources.otherResources = [{ close: async () => { events.push("consumer"); throw consumerError; } }];
  await assert.rejects(withBrowserCleanup(
    async () => { throw bodyError; },
    () => cleanupBrowserResources(resources, options),
  ), error => {
    assert.ok(error instanceof AggregateError);
    assert.equal(error.cause, bodyError);
    assert.equal(error.errors[0], bodyError);
    assert.ok(error.errors[1] instanceof AggregateError);
    assert.equal(error.errors[1].errors[0], consumerError);
    return true;
  });
  retained(events);
});
