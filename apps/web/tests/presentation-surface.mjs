import test from "node:test";
import assert from "node:assert/strict";
import { PresentationSurface } from "../dist/renderer/PresentationSurface.js";
function fixture() {
  const queued = new Map(); let next = 0; let draws = 0; let refreshes = 0;
  const errors = []; let reject;
  const surface = new PresentationSurface(() => { draws++; }, async () => {
    refreshes++; if (reject) throw reject; surface.present();
  }, error => errors.push(error), { request(callback) { queued.set(++next, callback); return next; }, cancel(id) { queued.delete(id); } });
  return { surface, queued, errors, counts: () => ({ draws, refreshes }), fail: error => { reject = error; },
    async flush() { const callbacks = [...queued.values()]; queued.clear(); for (const callback of callbacks) callback(); await Promise.resolve(); await Promise.resolve(); } };
}

test("presentation has no private ticking loop and one committed frame gives one GPU draw", async () => {
  const f = fixture(); assert.equal(f.queued.size, 0);
  for (let i = 0; i < 60; i++) f.surface.present();
  assert.deepEqual(f.counts(), { draws: 60, refreshes: 0 }); assert.equal(f.queued.size, 0);
  f.surface.request(); f.surface.request(); f.surface.request();
  assert.equal(f.queued.size, 1); await f.flush();
  assert.deepEqual(f.counts(), { draws: 61, refreshes: 0 }); assert.equal(f.queued.size, 0);
  for (let i = 0; i < 60; i++) await f.flush(); // Paused/hidden/idle: no autonomous redraw.
  assert.equal(f.counts().draws, 61);
  f.surface.present(); assert.equal(f.counts().draws, 62); // Next resumed frame remains live.
});

test("resize requests coalesce, reproject once and never duplicate their committed draw", async () => {
  const f = fixture(); f.surface.request(); f.surface.request(true); f.surface.request(true);
  assert.equal(f.queued.size, 1); await f.flush();
  assert.deepEqual(f.counts(), { draws: 1, refreshes: 1 }); assert.equal(f.queued.size, 0);
  f.surface.request(); f.surface.present(); await f.flush();
  assert.deepEqual(f.counts(), { draws: 2, refreshes: 1 });
});

test("context loss, identity invalidation and disposal cancel stale scheduled work", async () => {
  const f = fixture(); f.surface.request(true); const stale = [...f.queued.values()][0];
  f.surface.lost(); assert.equal(f.surface.present(), false); f.surface.request(true); stale();
  assert.deepEqual(f.counts(), { draws: 0, refreshes: 0 });
  f.surface.restored(); await f.flush(); assert.deepEqual(f.counts(), { draws: 1, refreshes: 1 });
  f.surface.request(true); const previous = [...f.queued.values()][0]; f.surface.invalidate(); previous();
  assert.equal(f.counts().draws, 1);
  f.surface.request(true); f.surface.dispose(); f.surface.restored(); f.surface.present(); await f.flush();
  assert.equal(f.counts().draws, 1);
});

test("surface failures remain observable and retired surface failures cannot poison a new identity", async () => {
  const f = fixture(); const error = new Error("resize failed"); f.fail(error); f.surface.request(true); await f.flush();
  assert.deepEqual(f.errors, [error]);
  let reject; const errors = []; let callback;
  const surface = new PresentationSurface(() => {}, () => new Promise((_, fail) => { reject = fail; }),
    error => errors.push(error), { request(fn) { callback = fn; return 1; }, cancel() {} });
  surface.request(true); callback(); surface.invalidate(); reject(error); await Promise.resolve(); await Promise.resolve();
  assert.deepEqual(errors, []);
});


test("a cancelled callback cannot erase a newer pending resize and synchronous failures are reported", async () => {
  const f = fixture(); f.surface.request(); const stale = [...f.queued.values()][0];
  f.surface.invalidate(); f.surface.request(true); stale();
  f.surface.request(true); assert.equal(f.queued.size, 1);
  await f.flush(); assert.deepEqual(f.counts(), { draws: 1, refreshes: 1 });
  const errors = []; let callback;
  const error = new Error("synchronous refresh failure");
  const surface = new PresentationSurface(() => {}, () => { throw error; }, e => errors.push(e),
    { request(fn) { callback = fn; return 1; }, cancel() {} });
  surface.request(true); callback(); assert.deepEqual(errors, [error]);
});
