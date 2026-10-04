import assert from "node:assert/strict";
import { test } from "node:test";
import vm from "node:vm";
import { createMockBridge } from "../src/mock-ipc.mjs";
import { installPassiveObserver, passiveObserverScript } from "../src/passive-observer.mjs";

function host() {
  let frame;
  let performanceCallback;
  let cancelled = 0;
  let disconnected = 0;
  return { value: { performance: { now: () => 0 },
    requestAnimationFrame(callback) { frame = callback; return 1; },
    cancelAnimationFrame() { cancelled++; },
    PerformanceObserver: class { constructor(callback) { performanceCallback = callback; }
      observe() {} disconnect() { disconnected++; } },
  }, frame: timestamp => frame(timestamp),
  tasks: entries => performanceCallback({ getEntries: () => entries }),
  stopped: () => cancelled > 0 && disconnected > 0 };
}

test("passive frame/long-task observation is bounded and cannot manufacture an IPC count", () => {
  const h = host(); const mock = createMockBridge(); h.value.__CURRENT_WEB_MOCK__ = mock;
  const observer = installPassiveObserver(h.value);
  for (let i = 0; i < 10000; i++) h.frame(i * 16);
  h.tasks(Array.from({ length: 5000 }, (_, i) => ({ startTime: i, duration: i + 50 })));
  assert.equal(observer.state.frames, 10000); assert.equal(observer.state.frameSamples.length, 64);
  assert.equal(observer.state.longTaskCount, 5000); assert.equal(observer.state.longTasks.length, 64);
  assert.equal(observer.state.maxFrameGapMs, 16); assert.equal(observer.state.maxLongTaskMs, 5049);
  assert.equal(mock.count("formal_submit_input"), 0); assert.equal(mock.state.calls.length, 0);
  observer.state.frames = Number.MAX_SAFE_INTEGER; h.frame(160000);
  assert.equal(observer.state.frames, Number.MAX_SAFE_INTEGER);
  observer.stop(); const saved = structuredClone(observer.state); h.frame(200000); h.tasks([{ startTime: 200000, duration: 9999 }]);
  assert.deepEqual(observer.state, saved); assert.ok(h.stopped());
});

test("serialized observer is self-contained and tolerates unavailable observation APIs", () => {
  const context = vm.createContext({ window: {} }); vm.runInContext(passiveObserverScript(), context);
  assert.equal(context.window.__CURRENT_WEB_PASSIVE__.state.frames, 0);
  context.window.__CURRENT_WEB_PASSIVE__.stop();
});
