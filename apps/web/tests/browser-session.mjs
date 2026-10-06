import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { createHash } from "node:crypto";
import { createProcessOwnership, commandFlag, sameProcess } from "./support/browser-process-ownership.mjs";
import { connectCdp, connectOwnedBrowser, createAttemptedTransportRegistry, waitForPage, closeVite, closeOwnedProcess } from "./support/browser-session.mjs";
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

function connectionFixture(inspectPort, extra = {}) {
  return { Socket: FakeSocket, fetcher: async () => ({ ok: true, json: async () => [{ type: "page", webSocketDebuggerUrl: socketUrl }] }),
    state: { hasExited: async () => false, ownership: { verifyPort: async () => true, inspectPort } }, ...extra };
}
async function fixtureConnect(fixture) {
  const { state, ...options } = fixture;
  return connectOwnedBrowser(12345, state, options);
}

test("connected socket waits for fresh complete ownership before returning its actual snapshot", async () => {
  const incomplete = Object.assign(new Error("missing executable"), { code: "E_PROCESS_INCOMPLETE" });
  const snapshot = [processRow(11)]; let queries = 0, ready = 0, clock = 0;
  const fixture = connectionFixture(async () => {
    assert.equal(FakeSocket.latest.readyState, FakeSocket.OPEN);
    assert.equal(ready, 0);
    if (++queries === 1) throw incomplete;
    return { ready: true, owned: snapshot };
  }, { now: () => clock, sleep: async ms => { clock += ms; }, onReady: owned => { ready++; assert.equal(owned, snapshot); } });
  const connection = await fixtureConnect(fixture);
  assert.equal(queries, 2); assert.equal(ready, 1); await connection.close();
});

for (const code of ["E_PROCESS_INCOMPLETE", "E_PROCESS_QUERY", "E_PROCESS_QUERY_TIMEOUT"]) {
  test(`persistent ${code} fails within one connection deadline and closes the registered socket`, async () => {
    const original = Object.assign(new Error("sample unavailable"), { code });
    let clock = 0, queries = 0, registered;
    const fixture = connectionFixture(async () => { queries++; throw original; }, {
      timeoutMs: 100, now: () => clock, sleep: async ms => { clock += ms; },
      onConnection: connection => { registered = connection; }, onReady: () => assert.fail("incomplete sample cannot be ready"),
    });
    await assert.rejects(fixtureConnect(fixture), error => error.code === "E_BROWSER_CONNECT_TIMEOUT" && error.cause === original && /fresh-ownership/.test(error.message));
    assert.equal(clock, 100); assert.equal(queries, 2);
    assert.ok(registered); assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
  });
}

for (const code of ["E_PROCESS_IDENTITY", "E_CDP_OWNER", "E_CDP_URL"]) {
  test(`hard ${code} after handshake fails immediately without accepting a later sample`, async () => {
    const original = Object.assign(new Error("hard identity failure"), { code }); let queries = 0;
    await assert.rejects(fixtureConnect(connectionFixture(async () => { if (++queries === 1) throw original; return { ready: true, owned: [processRow(11)] }; },
      { sleep: () => assert.fail("hard failure must not poll"), onReady: () => assert.fail("hard failure cannot be ready") })), error => error === original);
    assert.equal(queries, 1); assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
  });
}

test("PID reuse through the real registry immediately fails the connected path", async () => {
  const { owner, setRows } = ownershipFixture(); await owner.refresh();
  setRows([{ ...processRow(11), created: new Date(start + 1000).toISOString() }]);
  await assert.rejects(fixtureConnect(connectionFixture(options => owner.inspectPort(options), { sleep: () => assert.fail("PID reuse must not poll") })), { code: "E_PROCESS_IDENTITY" });
  assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
});

test("foreign listener through the real registry fails after handshake", async () => {
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [], query: async () => [processRow(11)], listener: async () => [999] });
  await assert.rejects(fixtureConnect(connectionFixture(options => owner.inspectPort(options))), { code: "E_CDP_OWNER" });
  assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
});

test("wrong discovered port never starts the socket handshake", async () => {
  await assert.rejects(fixtureConnect(connectionFixture(() => assert.fail("no ownership phase"), {
    Socket: class { constructor() { assert.fail("wrong port must not allocate socket"); } },
    fetcher: async () => ({ ok: true, json: async () => [{ type: "page", webSocketDebuggerUrl: "ws://127.0.0.1:999/devtools/page/1" }] }),
  })), { code: "E_CDP_URL" });
});

test("expired connection deadline cannot begin page discovery", async () => {
  const fixture = connectionFixture(() => assert.fail("must not query"), { deadline: 5, now: () => 5,
    fetcher: () => assert.fail("must not fetch"), Socket: class { constructor() { assert.fail("must not allocate socket"); } } });
  await assert.rejects(fixtureConnect(fixture), { code: "E_BROWSER_CONNECT_TIMEOUT" });
});

test("page discovery consumes the same budget and cannot start a late handshake", async () => {
  let clock = 0;
  const fixture = connectionFixture(() => assert.fail("must not reach fresh ownership"), { timeoutMs: 100, now: () => clock,
    Socket: class { constructor() { assert.fail("must not start late handshake"); } },
    fetcher: async () => { clock = 100; return { ok: true, json: async () => [{ type: "page", webSocketDebuggerUrl: socketUrl }] }; } });
  await assert.rejects(fixtureConnect(fixture), { code: "E_BROWSER_CONNECT_TIMEOUT" });
});

test("late complete ownership cannot emit ready or return a connection", async () => {
  let clock = 0;
  await assert.rejects(fixtureConnect(connectionFixture(async () => { clock = 100; return { ready: true, owned: [processRow(11)] }; },
    { timeoutMs: 100, now: () => clock, onReady: () => assert.fail("late readiness") })), { code: "E_BROWSER_CONNECT_TIMEOUT" });
  assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
});

for (const event of ["close", "error"]) test(`transport ${event} during ownership sampling aborts polling and remains visible`, async () => {
  let queries = 0;
  await assert.rejects(fixtureConnect(connectionFixture(async () => {
    queries++; FakeSocket.latest.dispatchEvent(new Event(event));
    return { ready: true, owned: [processRow(11)] };
  }, { onReady: () => assert.fail("disconnected transport cannot be ready"), sleep: () => assert.fail("must not poll") })),
  { code: event === "close" ? "E_CDP_CLOSED" : "E_CDP_SOCKET" });
  assert.equal(queries, 1); assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
});

test("original ownership failure and socket close failure remain inspectable together", async () => {
  const original = Object.assign(new Error("identity failed"), { code: "E_PROCESS_IDENTITY" }), cleanup = new Error("close failed");
  class FailedClose extends FakeSocket { close() { super.close(); throw cleanup; } }
  await assert.rejects(fixtureConnect(connectionFixture(async () => { throw original; }, { Socket: FailedClose })),
    error => error instanceof AggregateError && error.cause === original && error.errors[0] === original && error.errors[1] === cleanup);
});

test("deadline cancellation reaches an in-flight query and cannot later emit ready", async () => {
  let cancelled = false, ready = 0, queries = 0;
  await assert.rejects(fixtureConnect(connectionFixture(({ signal }) => new Promise((resolve, reject) => {
    queries++;
    signal.addEventListener("abort", () => { cancelled = true; reject(signal.reason); }, { once: true });
  }), { timeoutMs: 10, onReady: () => { ready++; } })), { code: "E_BROWSER_CONNECT_TIMEOUT" });
  assert.equal(cancelled, true); assert.equal(queries, 1); assert.equal(ready, 0);
  assert.equal(FakeSocket.latest.readyState, FakeSocket.CLOSED);
});

const diagnosticSummary = value => ({
  state: value === null ? 'null' : value === undefined ? 'missing' : value.length === 0 ? 'empty' : 'present',
  length: typeof value === 'string' ? value.length : null,
  sha256: typeof value === 'string' ? createHash('sha256').update(value, 'utf8').digest('hex') : null,
});
function checkIdentityDiagnostic(error, stage, expected, actual, changedFields) {
  assert.equal(error.code, 'E_PROCESS_IDENTITY');
  assert.ok(error.message.startsWith(stage === 'refresh' ? 'Owned PID was reused or changed' : 'Process changed before termination'));
  const diagnostic = error.diagnostic;
  assert.ok(Object.keys(error).includes('diagnostic'));
  assert.equal(diagnostic.stage, stage);
  assert.equal(diagnostic.pid, expected.pid);
  assert.ok(Number.isFinite(Date.parse(diagnostic.utc)) && diagnostic.utc.endsWith('Z'));
  assert.deepEqual(diagnostic.changedFields, changedFields);
  for (const [side, row] of [['expected', expected], ['actual', actual]]) {
    assert.deepEqual(diagnostic[side], { pid: row.pid, parent: row.parent, created: row.created,
      exe: diagnosticSummary(row.exe), command: diagnosticSummary(row.command) });
    assert.ok(Object.isFrozen(diagnostic[side]) && Object.isFrozen(diagnostic[side].exe) && Object.isFrozen(diagnostic[side].command));
  }
  assert.deepEqual(JSON.parse(error.message.split(' identityDiagnostic=')[1]), diagnostic);
  for (const privateValue of [profile, binary, expected.command, actual.command, actual.exe]) {
    if (typeof privateValue === 'string' && privateValue.length) assert.equal(error.message.includes(privateValue), false);
  }
  assert.ok(Object.isFrozen(diagnostic) && Object.isFrozen(diagnostic.changedFields));
  return true;
}

test('refresh identity diagnostics preserve the registered identity and hard refusal for every field change', async () => {
  for (const patch of [
    { created: new Date(start + 1000).toISOString() },
    { exe: path.resolve('different-edge', 'msedge.exe') }, { exe: null }, { exe: '' }, { exe: undefined },
    { command: 'edge --type=renderer --private-test=secret' }, { command: null }, { command: '' }, { command: undefined },
    { parent: 999, exe: null, command: null },
  ]) {
    const expected = Object.freeze(processRow(11));
    const actual = { ...expected, ...patch };
    const saved = { ...actual };
    const fixture = ownershipFixture([expected]);
    await fixture.owner.refresh();
    fixture.setRows([actual]);
    const changedFields = ['pid', 'created', 'exe', 'command'].filter(field => expected[field] !== actual[field]);
    let firstError;
    await assert.rejects(fixture.owner.terminate(), error => {
      firstError = error;
      return checkIdentityDiagnostic(error, 'refresh', expected, saved, changedFields);
    });
    assert.deepEqual(actual, saved);
    assert.deepEqual(fixture.stopped, []);
    // A second mismatch must still compare against the original registered row.
    await assert.rejects(fixture.owner.refresh(), error => checkIdentityDiagnostic(error, 'refresh', expected, saved, changedFields));
    actual.command = 'later mutable query row';
    assert.deepEqual(firstError.diagnostic.actual.command, diagnosticSummary(saved.command));
    fixture.setRows([expected]);
    assert.deepEqual(await fixture.owner.refresh(), [expected]);
  }
});

test('pre-stop second query mismatch keeps exact diagnostic and never invokes stop', async () => {
  for (const patch of [{ created: new Date(start + 999).toISOString() }, { exe: null }, { command: null }, { exe: '', command: '' }]) {
    const expected = Object.freeze(processRow(11));
    const actual = { ...expected, ...patch };
    let queries = 0;
    const stopped = [];
    const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [],
      query: async () => (++queries === 1 ? [expected] : [actual]), stop: async row => stopped.push(row.pid) });
    await assert.rejects(owner.terminate(), error => checkIdentityDiagnostic(error, 'pre-stop', expected, actual,
      ['pid', 'created', 'exe', 'command'].filter(field => expected[field] !== actual[field])));
    assert.equal(queries, 2);
    assert.deepEqual(stopped, []);
  }
});

class AttemptSocket extends EventTarget {
  static OPEN = 1; static CLOSED = 3;
  static latest;
  constructor() { super(); AttemptSocket.latest = this; this.readyState = 0; this.listeners = new Map(); this.closeCalls = 0; }
  addEventListener(type, fn, options) { this.listeners.set(fn, type); super.addEventListener(type, fn, options); }
  removeEventListener(type, fn) { this.listeners.delete(fn); super.removeEventListener(type, fn); }
  emit(type, error) { const event = new Event(type); if (error) event.error = error; this.dispatchEvent(event); }
  close() { this.closeCalls++; this.readyState = 3; this.emit('close'); }
}
const containsError = (error, target) => error === target || error?.cause === target || error?.errors?.some(member => containsError(member, target));

for (const mode of ['throws', 'delayed', 'never']) test(`failed handshake ${mode} disposal remains reachable in the real session registry`, async () => {
  const registry = createAttemptedTransportRegistry();
  const socketFailure = new Error('original socket failure');
  const closeFailure = new Error('original disposal failure');
  let resolveClose, caught, returned = false;
  class BrokenSocket extends AttemptSocket {
    constructor() { super(); queueMicrotask(() => this.emit('error', socketFailure)); }
    close() {
      this.closeCalls++;
      if (mode === 'throws') throw closeFailure;
      if (mode === 'delayed') resolveClose = () => { this.readyState = 3; this.emit('close'); };
    }
  }
  const run = connectOwnedBrowser(12345, connectionFixture(() => assert.fail('handshake must not reach sample')).state, {
    ...connectionFixture(() => assert.fail('no ready')),
    Socket: BrokenSocket, disposalTimeoutMs: 15,
    onAttempt: handle => registry.register(handle), onConnection: () => assert.fail('failed handshake must not register success'),
  }).catch(error => { caught = error; returned = true; });
  // Flush allocation, handshake rejection and entry into disposal, without a timer race.
  for (let i = 0; i < 20; i++) await Promise.resolve();
  assert.equal(registry.retainedCount, 1);
  if (mode === 'delayed') {
    assert.equal(returned, false); assert.ok(resolveClose);
    resolveClose();
  }
  await run;
  assert.ok(containsError(caught, socketFailure));
  if (mode === 'throws') assert.ok(containsError(caught, closeFailure));
  if (mode === 'never') assert.ok(caught instanceof AggregateError && caught.errors[1].code === 'E_CDP_SOCKET_CLOSE_TIMEOUT');
  const attempted = AttemptSocket.latest;
  assert.equal(attempted.listeners.size, 0);
  assert.equal(attempted.closeCalls, 1);
  // Session cleanup can reach exactly the same attempted transport after rejection.
  attempted.close = () => { attempted.closeCalls++; attempted.readyState = 3; attempted.emit('close'); };
  await registry.close(15);
  assert.equal(registry.retainedCount, 0);
  assert.equal(attempted.closeCalls, mode === 'delayed' ? 1 : 2);
  assert.equal(attempted.listeners.size, 0);
});

for (const mode of ['deadline', 'abort']) test(`${mode} during handshake drains listeners and rejects a late open`, async () => {
  const controller = new AbortController(), registry = createAttemptedTransportRegistry();
  const original = new Error('explicit cancellation');
  let ready = 0, registered;
  const fixture = connectionFixture(() => assert.fail('late socket cannot sample'), {
    Socket: AttemptSocket, timeoutMs: 15, signal: controller.signal, disposalTimeoutMs: 15,
    onAttempt: handle => { registered = handle; registry.register(handle); if (mode === 'abort') queueMicrotask(() => controller.abort(original)); },
    onReady: () => { ready++; },
  });
  await assert.rejects(fixtureConnect(fixture), error => mode === 'abort' ? error === original : error.code === 'E_BROWSER_CONNECT_TIMEOUT');
  assert.ok(registered); assert.equal(ready, 0); assert.equal(AttemptSocket.latest.listeners.size, 0);
  AttemptSocket.latest.readyState = 1; AttemptSocket.latest.emit('open');
  await Promise.resolve();
  assert.equal(ready, 0);
  await assert.rejects(registered.send('Page.enable'), { code: 'E_CDP_CLOSED' });
  await registry.close(15);
  assert.equal(registry.retainedCount, 0); assert.equal(AttemptSocket.latest.listeners.size, 0);
});

for (const code of ['E_PROCESS_INCOMPLETE', 'E_PROCESS_QUERY', 'E_PROCESS_QUERY_TIMEOUT']) test(`page outer timer preserves the latest original ${code} cause`, async () => {
  const first = Object.assign(new Error('first unavailable query'), { code });
  const latest = Object.assign(new Error('last unavailable query'), { code });
  let calls = 0;
  const fixture = connectionFixture(() => assert.fail('page failure must not handshake'), {
    timeoutMs: 15,
    state: { hasExited: async () => { throw ++calls === 1 ? first : latest; } },
    sleep: async (ms, value, { signal }) => {
      if (calls === 1) return;
      await new Promise((resolve, reject) => signal.addEventListener('abort', () => reject(signal.reason), { once: true }));
    },
    fetcher: () => assert.fail('failed identity cannot fetch'), Socket: class { constructor() { assert.fail('no socket'); } },
  });
  await assert.rejects(fixtureConnect(fixture), error => error.code === 'E_BROWSER_CONNECT_TIMEOUT' && error.cause === latest && /page/.test(error.message));
  assert.equal(calls, 2);
});

test('page hard identity errors retain the original object without polling', async () => {
  const original = Object.assign(new Error('hard page identity'), { code: 'E_PROCESS_IDENTITY' });
  await assert.rejects(fixtureConnect(connectionFixture(() => assert.fail('no sample'), {
    state: { hasExited: async () => { throw original; } }, sleep: () => assert.fail('hard page failure must not poll'),
  })), error => error === original);
});

for (const patch of [{ created: new Date(start + 1000).toISOString() }, { exe: path.resolve('new', 'msedge.exe') },
  { command: 'replacement command' }, { exe: null }, { command: null }]) test(`listener-side replacement is refused: ${Object.keys(patch)[0]} ${patch[Object.keys(patch)[0]] === null ? 'missing' : 'changed'}`, async () => {
  const expected = processRow(11); let rows = [expected], queries = 0, stopped = 0;
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [],
    query: async () => { queries++; return rows; }, listener: async () => { rows = [{ ...expected, ...patch }]; return [11]; }, stop: () => { stopped++; } });
  await assert.rejects(owner.verifyPort({ timeoutMs: 100 }), error => checkIdentityDiagnostic(error, 'refresh', expected, rows[0], Object.keys(patch)));
  assert.equal(queries, 2); assert.equal(stopped, 0);
});

test('a root disappearing while listener is queried cannot use a stale snapshot', async () => {
  let rows = [processRow(11)];
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [],
    query: async () => rows, listener: async () => { rows = []; return [11]; }, stop: () => assert.fail('no stop') });
  await assert.rejects(owner.inspectPort({ timeoutMs: 100 }), { code: 'E_CDP_OWNER' });
});

test('stable listener returns the last exact snapshot with remaining budgets and cancellation', async () => {
  const initial = processRow(11), final = { ...initial }; let queries = 0, clock = 0;
  const controller = new AbortController(), budgets = [];
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [],
    query: async options => { assert.equal(options.signal, controller.signal); budgets.push(options.timeoutMs); clock += 10; return ++queries === 1 ? [initial] : [final]; },
    listener: async (port, options) => { assert.equal(port, 12345); assert.equal(options.signal, controller.signal); budgets.push(options.timeoutMs); clock += 10; return [11]; } });
  const sample = await owner.inspectPort({ timeoutMs: 100, signal: controller.signal, now: () => clock });
  assert.equal(sample.ready, true); assert.equal(sample.owned[0], final); assert.deepEqual(budgets, [100, 90, 80]);
});

for (const mode of ['late', 'aborted']) test(`listener ${mode} success never authorizes readiness`, async () => {
  let clock = 0; const controller = new AbortController();
  const owner = createProcessOwnership({ profile, binary, port: 12345, startedAt: start, before: [], query: async () => [processRow(11)],
    listener: async () => { if (mode === 'late') clock = 100; else controller.abort(new Error('cancel listener')); return [11]; } });
  await assert.rejects(owner.inspectPort({ timeoutMs: 100, signal: controller.signal, now: () => clock }));
});

test('registered attempted transport can synchronously open before handshake listeners attach', async () => {
  let registered;
  const connection = await connectCdp(socketUrl, { Socket: AttemptSocket, onAttempt: handle => {
    registered = handle;
    AttemptSocket.latest.readyState = AttemptSocket.OPEN;
    AttemptSocket.latest.emit('open');
  } });
  assert.equal(connection, registered);
  assert.equal(AttemptSocket.latest.readyState, AttemptSocket.OPEN);
  await connection.close();
  assert.equal(AttemptSocket.latest.readyState, AttemptSocket.CLOSED);
  assert.equal(AttemptSocket.latest.listeners.size, 0);
});
