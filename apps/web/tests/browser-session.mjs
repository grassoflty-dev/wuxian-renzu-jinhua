import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { createProcessOwnership, commandFlag, sameProcess } from "./support/browser-process-ownership.mjs";
import { connectCdp, waitForPage, closeVite, closeOwnedProcess } from "./support/browser-session.mjs";
import { cleanupBrowserResources, removeOwnedProfile, withBrowserCleanup } from "./support/browser-cleanup.mjs";

const profile = path.resolve("owned-session", "unique-run", "profile");
const binary = path.resolve("edge", "msedge.exe");
const start = Date.parse("2026-10-06T00:00:00.000Z");
const processRow = (pid, parent = 10, command = `"${binary}" --user-data-dir="${profile}" --remote-debugging-port=12345`) => ({
  pid, parent, created: new Date(start + pid).toISOString(), exe: binary, command, name: "msedge.exe",
});
function ownershipFixture(initial = [processRow(11)]) {
  let rows = initial;
  const stopped = [];
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [],
    query: async () => rows, listener: async () => [11], stop: async p => { stopped.push(p.pid); rows = rows.filter(row => row.pid !== p.pid); } });
  return { owner, stopped, setRows(value) { rows = value; } };
}

test("Windows command flags compare the entire quoted profile and port", () => {
  assert.equal(commandFlag('--user-data-dir="C:\\a b\\profile" --remote-debugging-port=12345', "user-data-dir"), "C:\\a b\\profile");
  assert.equal(commandFlag('"--user-data-dir=C:\\a b\\profile"', "user-data-dir"), "C:\\a b\\profile");
  assert.equal(commandFlag('--remote-debugging-port=123456', "remote-debugging-port"), "123456");
});

test("an exited launcher does not hide an independently verified relaunched root", async () => {
  const { owner } = ownershipFixture();
  owner.registerLauncher({ pid: 10, exitCode: 0, signalCode: null });
  assert.equal(await owner.hasExited(), false);
  assert.equal(await owner.verifyPort(), true);
});

test("a failed spawn without a PID cannot strand an unused profile", async () => {
  const { owner, stopped } = ownershipFixture([]);
  owner.registerLauncher({ pid: undefined, exitCode: null, signalCode: null });
  assert.equal(await owner.hasExited(), true);
  assert.deepEqual(stopped, []);
});

test("process identity comparison includes PID, creation time, executable and command", () => {
  const row = processRow(11);
  assert.equal(sameProcess(row, { ...row }), true);
  for (const field of ["pid", "created", "exe", "command"]) assert.equal(sameProcess(row, { ...row, [field]: "different" }), false);
});

test("ownership excludes pre-existing users, profile prefix collisions and another debug port", async () => {
  const prior = processRow(11);
  const collision = processRow(12, 10, `edge --user-data-dir="${profile}-other" --remote-debugging-port=12345`);
  const otherPort = processRow(13, 10, `edge --user-data-dir="${profile}-different" --remote-debugging-port=12346`);
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [prior], query: async () => [prior, collision, otherPort], stop: () => assert.fail("must not terminate users") });
  await assert.rejects(owner.refresh(), { code: "E_PROCESS_IDENTITY" });
});

test("termination revalidates every owned descendant and excludes other profiles", async () => {
  const child = processRow(12, 11, "edge --type=renderer");
  const other = processRow(13, 10, `edge --user-data-dir="${profile}-other" --remote-debugging-port=12345`);
  const { owner, stopped } = ownershipFixture([processRow(11), child, other]);
  await owner.terminate();
  assert.deepEqual(stopped, [12, 11]);
  assert.equal(await owner.hasExited(), true);
});

test("a different executable using the same profile cannot become an owned root", async () => {
  const row = { ...processRow(11), exe: path.resolve("other", "msedge.exe") };
  const { owner, stopped } = ownershipFixture([row]);
  await assert.rejects(owner.terminate(), { code: "E_PROCESS_IDENTITY" });
  assert.deepEqual(stopped, []);
});

test("PID reuse rejects cleanup without stopping the replacement process", async () => {
  const { owner, stopped, setRows } = ownershipFixture();
  await owner.refresh();
  setRows([{ ...processRow(11), created: new Date(start + 1000).toISOString() }]);
  await assert.rejects(owner.terminate(), { code: "E_PROCESS_IDENTITY" });
  assert.deepEqual(stopped, []);
});

test("identity changing between inventory and stop fails closed", async () => {
  let calls = 0;
  const row = processRow(11);
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [],
    query: async () => [++calls === 1 ? row : { ...row, command: "another process" }], stop: () => assert.fail("changed identity must not be stopped") });
  await assert.rejects(owner.terminate(), { code: "E_PROCESS_IDENTITY" });
});

for (const field of ["exe", "command"]) test(`missing ${field} cannot authorize browser termination`, async () => {
  const { owner, stopped } = ownershipFixture([{ ...processRow(11), [field]: null }]);
  await assert.rejects(owner.terminate(), { code: "E_PROCESS_INCOMPLETE" });
  assert.deepEqual(stopped, []);
});

test("malformed or failed process inventory never proves safe profile removal", async () => {
  for (const query of [async () => null, async () => [{ pid: 11 }], async () => { throw new Error("inventory failed"); }]) {
    const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [], query, stop: () => assert.fail("unknown process must survive") });
    await assert.rejects(owner.hasExited());
  }
});

test("foreign listener on the selected CDP port is rejected", async () => {
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [], query: async () => [processRow(11)], listener: async () => [999] });
  await assert.rejects(owner.verifyPort(), { code: "E_CDP_OWNER" });
});

test("an unverified orphan still referencing the profile fails closed", async () => {
  const orphan = processRow(12, 999, `edge --type=renderer --user-data-dir="${profile}"`);
  const { owner } = ownershipFixture([orphan]);
  await assert.rejects(owner.hasExited(), { code: "E_PROCESS_IDENTITY" });
});

test("a reused parent PID cannot authorize a new descendant", async () => {
  const { owner, setRows } = ownershipFixture();
  await owner.refresh();
  setRows([{ ...processRow(11), command: "replacement parent" }, processRow(12, 11, "child")]);
  await assert.rejects(owner.refresh(), { code: "E_PROCESS_IDENTITY" });
});

test("exited launcher with an authorized not-yet-ready session waits for real CDP", async () => {
  let fetches = 0;
  const state = { child: { exitCode: 0 }, hasExited: async () => false, ownership: { verifyPort: async () => true } };
  const url = "ws://127.0.0.1:12345/devtools/page/1";
  assert.equal(await waitForPage(12345, state, { fetcher: async () => ({ ok: true, json: async () => ++fetches === 1 ? [] : [{ type: "page", webSocketDebuggerUrl: url }] }) }), url);
  assert.equal(fetches, 2);
});

test("spawn failure preserves the original error before fetching CDP", async () => {
  const failure = Object.assign(new Error("spawn failed"), { code: "ENOENT" });
  await assert.rejects(waitForPage(12345, { error: failure }, { fetcher: () => assert.fail("spawn failure must not fetch") }), error => error === failure);
});

test("unavailable CDP has a finite connection deadline", async () => {
  const state = { hasExited: async () => false, ownership: { verifyPort: async () => true } };
  await assert.rejects(waitForPage(12345, state, { connectTimeoutMs: 5, fetcher: async () => ({ ok: true, json: async () => [] }) }), { code: "E_BROWSER_CONNECT_TIMEOUT" });
});

test("incomplete ownership can be reread, but never authorizes a socket itself", async () => {
  let calls = 0;
  const state = { hasExited: async () => { if (++calls === 1) throw Object.assign(new Error("incomplete"), { code: "E_PROCESS_INCOMPLETE" }); return false; }, ownership: { verifyPort: async () => true } };
  const url = "ws://127.0.0.1:12345/devtools/page/1";
  assert.equal(await waitForPage(12345, state, { fetcher: async () => ({ ok: true, json: async () => [{ type: "page", webSocketDebuggerUrl: url }] }) }), url);
  assert.equal(calls, 2);
});

test("CDP URL using a different port is rejected before transport creation", async () => {
  await assert.rejects(waitForPage(12345, { hasExited: async () => false, ownership: { verifyPort: () => assert.fail("wrong URL must not connect") } },
    { fetcher: async () => ({ ok: true, json: async () => [{ type: "page", webSocketDebuggerUrl: "ws://127.0.0.1:999/devtools/page/1" }] }) }), { code: "E_CDP_URL" });
});

class FakeSocket extends EventTarget {
  static OPEN = 1; static CLOSED = 3;
  static latest;
  constructor() { super(); FakeSocket.latest = this; this.readyState = 0; queueMicrotask(() => { this.readyState = 1; this.dispatchEvent(new Event("open")); }); }
  send(message) { this.sent = JSON.parse(message); }
  close() { this.readyState = 3; this.dispatchEvent(new Event("close")); }
  message(data) { const event = new Event("message"); event.data = data; this.dispatchEvent(event); }
}
const socketUrl = "ws://127.0.0.1:12345/devtools/page/1";

test("real browser transport exit rejects every pending CDP request", async () => {
  const cdp = await connectCdp(socketUrl, { Socket: FakeSocket });
  const requests = [cdp.send("Page.enable"), cdp.send("Runtime.evaluate")];
  const rejected = requests.map(request => assert.rejects(request, { code: "E_CDP_CLOSED" }));
  FakeSocket.latest.close();
  await Promise.all(rejected); await cdp.close();
});

test("CDP requests time out, and a late response cannot resolve them", async () => {
  const cdp = await connectCdp(socketUrl, { Socket: FakeSocket });
  await assert.rejects(cdp.send("Runtime.evaluate", {}, { timeoutMs: 5 }), { code: "E_CDP_TIMEOUT" });
  FakeSocket.latest.message(JSON.stringify({ id: 1, result: {} }));
  await cdp.close();
});

test("CDP protocol errors and malformed responses remain failures", async () => {
  const cdp = await connectCdp(socketUrl, { Socket: FakeSocket });
  const first = assert.rejects(cdp.send("Browser.close"), { code: "E_CDP_PROTOCOL" });
  FakeSocket.latest.message(JSON.stringify({ id: 1, error: { message: "genuine protocol error" } })); await first;
  const second = assert.rejects(cdp.send("Page.enable"), { code: "E_CDP_PROTOCOL" });
  FakeSocket.latest.message("invalid JSON"); await second; await cdp.close();
});

test("failed CDP handshake has a deadline and closes the attempted socket", async () => {
  class NotOpening extends EventTarget {
    static CLOSED = 3;
    constructor() { super(); NotOpening.latest = this; }
    close() { this.closed = true; this.dispatchEvent(new Event("close")); }
  }
  await assert.rejects(connectCdp(socketUrl, { Socket: NotOpening, timeoutMs: 5 }), { code: "E_CDP_CONNECT_TIMEOUT" });
  assert.equal(NotOpening.latest.closed, true);
});

test("send failure and a closed socket reject requests without leaked timers", async () => {
  const cdp = await connectCdp(socketUrl, { Socket: FakeSocket });
  const failure = new Error("send failed"); FakeSocket.latest.send = () => { throw failure; };
  await assert.rejects(cdp.send("Page.enable"), error => error === failure);
  await cdp.close(); await assert.rejects(cdp.send("Page.enable"), { code: "E_CDP_CLOSED" });
});

test("non-local CDP URLs cannot create sockets", async () => {
  await assert.rejects(connectCdp("ws://example.com/devtools/page/1", { Socket: () => assert.fail("external socket") }), { code: "E_CDP_URL" });
});

test("in-process Vite close timeout remains visible", async () => {
  await assert.rejects(closeVite({ close: () => new Promise(() => {}) }, undefined, 5), { code: "E_VITE_CLOSE_TIMEOUT" });
});

test("Vite close and owned worker failure preserve both errors", async () => {
  const original = new Error("Vite close failed"), worker = new Error("worker terminate failed");
  await assert.rejects(closeVite({ close: async () => { throw original; } }, { terminate: async () => { throw worker; } }), error => error.cause === original && error.errors[0] === original && error.errors[1] === worker);
});

test("a server process which does not exit remains a failure", async () => {
  await assert.rejects(closeOwnedProcess({ terminate: async () => {}, waitForExit: async () => false }, 5), { code: "E_VITE_EXIT" });
});

test("alive or unverifiable session retains the profile while server cleanup still runs", async () => {
  for (const state of [
    { hasExited: async () => false, terminate: async () => {}, waitForExit: async () => false },
    { hasExited: async () => { throw new Error("inventory failed"); }, terminate: async () => { throw new Error("cannot authorize"); }, waitForExit: async () => false },
  ]) {
    let closed = false;
    await assert.rejects(cleanupBrowserResources({ browserState: state, ownedProfile: profile, otherResources: [{ close: async () => { closed = true; } }] }, { profileRemoval: { remove: () => assert.fail("live profile must survive") }, terminateExitTimeoutMs: 1, killExitTimeoutMs: 1 }));
    assert.equal(closed, true);
  }
});

test("profile registration failure prevents any removal attempt", async () => {
  await assert.rejects(removeOwnedProfile(profile, { verify: async () => { throw Object.assign(new Error("outside registered run"), { code: "E_PROFILE_OWNER" }); }, remove: () => assert.fail("unregistered path must survive") }), { code: "E_PROFILE_OWNER" });
  await assert.rejects(removeOwnedProfile(profile), { code: "E_PROFILE_OWNER" });
});

test("profile removal itself has a bounded attempt timeout", async () => {
  await assert.rejects(removeOwnedProfile(profile, { remove: () => new Promise(() => {}), attemptTimeoutMs: 5 }), { code: "E_PROFILE_REMOVE_TIMEOUT" });
});

test("expired cleanup budget retains profile and reports failure", async () => {
  await assert.rejects(cleanupBrowserResources({ browserState: { hasExited: () => true }, ownedProfile: profile }, { deadline: Date.now() - 1, profileRemoval: { remove: () => assert.fail("expired budget must retain profile") } }), error => error.errors.some(e => e.code === "E_CLEANUP_BUDGET"));
});

test("expired cleanup budget and dispose failure preserve both original errors", async () => {
  const sentinel = new Error("dispose failed after expired budget");
  let removals = 0, disposals = 0;
  await assert.rejects(cleanupBrowserResources({
    browserState: { hasExited: () => true, dispose() { disposals++; throw sentinel; } }, ownedProfile: profile,
  }, { deadline: Date.now() - 1, profileRemoval: { remove: async () => { removals++; } } }), error => {
    assert.ok(error instanceof AggregateError);
    assert.ok(error.errors.some(e => e.code === "E_CLEANUP_BUDGET"));
    assert.ok(error.errors.includes(sentinel));
    return true;
  });
  assert.equal(removals, 0);
  assert.equal(disposals, 1);
});

test("dispose-only failure cannot turn successful resource cleanup into PASS", async () => {
  const sentinel = new Error("dispose failed after normal cleanup");
  let removals = 0, disposals = 0;
  await assert.rejects(cleanupBrowserResources({
    browserState: { hasExited: () => true, dispose() { disposals++; throw sentinel; } }, ownedProfile: profile,
  }, { profileRemoval: { remove: async value => { assert.equal(value, profile); removals++; } } }), error => {
    assert.ok(error instanceof AggregateError);
    assert.deepEqual(error.errors, [sentinel]);
    assert.equal(error.errors[0], sentinel);
    return true;
  });
  assert.equal(removals, 1);
  assert.equal(disposals, 1);
});

test("body assertion and session cleanup failure remain inspectable together", async () => {
  const body = new assert.AssertionError({ message: "real behavior failed" }), cleanup = new Error("ownership failed");
  await assert.rejects(withBrowserCleanup(async () => { throw body; }, async () => { throw cleanup; }), error => error.cause === body && error.errors[0] === body && error.errors[1] === cleanup);
});
