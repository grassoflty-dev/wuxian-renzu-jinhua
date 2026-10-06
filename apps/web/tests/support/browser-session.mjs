import { spawn } from "node:child_process";
import { mkdir, mkdtemp, realpath, lstat, rmdir } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { bounded, cleanupBrowserResources, watchBrowserProcess, withBrowserCleanup } from "./browser-cleanup.mjs";
import { createProcessOwnership, windowsProcesses, ownershipError } from "./browser-process-ownership.mjs";

export { withBrowserCleanup };
export async function closeOwnedProcess(owner, timeout = 4500) {
  const end = Date.now() + timeout;
  await owner.terminate(Math.max(1, timeout - 1500));
  if (!await owner.waitForExit(Math.max(1, end - Date.now()))) throw ownershipError("E_VITE_EXIT", "Owned server process did not exit");
}
export async function closeVite(vite, owner, timeout = 4500) {
  const end = Date.now() + timeout;
  await withBrowserCleanup(
    () => bounded(() => vite.close(), Math.min(3000, timeout), "E_VITE_CLOSE_TIMEOUT"),
    async () => {
      if (!owner) return;
      if (Date.now() >= end) throw ownershipError("E_VITE_WORKER_EXIT", "Vite close budget exhausted; retaining worker");
      await closeOwnedProcess(owner, Math.max(1, end - Date.now()));
    });
}
export async function freePort() {
  const server = net.createServer();
  await bounded(() => new Promise((resolve, reject) => server.once("error", reject).listen(0, "127.0.0.1", resolve)), 1500, "E_PORT_LISTEN_TIMEOUT");
  const port = server.address().port;
  await bounded(() => new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve())), 1500, "E_PORT_CLOSE_TIMEOUT");
  return port;
}
export async function startHttpServer(server) {
  try {
    await bounded(() => new Promise((resolve, reject) => {
      const fail = error => { server.off("listening", ready); reject(error); };
      const ready = () => { server.off("error", fail); resolve(); };
      server.once("error", fail); server.once("listening", ready); server.listen(0, "127.0.0.1");
    }), 1500, "E_SERVER_START_TIMEOUT");
  } catch (error) { server.closeAllConnections?.(); server.close(); throw error; }
}

// The same registry is used by real sessions and injected transport regressions.
// A failed close remains registered so later session cleanup can try again.
export function createAttemptedTransportRegistry() {
  const attempts = new Set();
  return {
    register(connection) { attempts.add(connection); },
    get retainedCount() { return attempts.size; },
    async close(timeoutMs = 1500) {
      const errors = [];
      for (const connection of attempts) {
        try { await connection.close(Math.min(1500, timeoutMs)); attempts.delete(connection); }
        catch (error) { errors.push(error); }
      }
      if (errors.length) throw new AggregateError(errors, "Attempted CDP transports did not close");
    },
  };
}

export async function connectCdp(url, { Socket = WebSocket, timeoutMs = 5000, signal,
  onAttempt = () => {}, disposalTimeoutMs = 1500 } = {}) {
  signal?.throwIfAborted();
  const parsed = new URL(url);
  if (parsed.protocol !== "ws:" || !["127.0.0.1", "localhost", "[::1]"].includes(parsed.hostname)) throw ownershipError("E_CDP_URL", "CDP must be a local WebSocket");
  const socket = new Socket(url);
  let next = 0, closed = false;
  const transport = new AbortController();
  const pending = new Map();
  const disconnected = () => ownershipError("E_CDP_CLOSED", "CDP connection closed");
  const rejectPending = error => { closed = true; transport.abort(error); for (const request of pending.values()) request.reject(error); pending.clear(); };
  const onClose = () => rejectPending(disconnected());
  const onError = event => rejectPending(ownershipError("E_CDP_SOCKET", "CDP socket failed", event.error));
  const onMessage = ({ data }) => {
    let message;
    try { message = JSON.parse(data); } catch (error) { rejectPending(ownershipError("E_CDP_PROTOCOL", "Invalid CDP message", error)); return; }
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    message.error ? request.reject(ownershipError("E_CDP_PROTOCOL", message.error.message)) : request.resolve(message.result);
  };
  socket.addEventListener("close", onClose);
  socket.addEventListener("error", onError);
  socket.addEventListener("message", onMessage);
  const detach = () => {
    socket.removeEventListener("close", onClose);
    socket.removeEventListener("error", onError);
    socket.removeEventListener("message", onMessage);
  };
  const connection = {
    transportSignal: transport.signal,
    send(method, params = {}, { timeoutMs: requestTimeout = 5000 } = {}) {
      if (closed || socket.readyState !== Socket.OPEN) return Promise.reject(disconnected());
      const id = ++next;
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => { pending.delete(id); reject(ownershipError("E_CDP_TIMEOUT", `CDP request timed out: ${method}`)); }, requestTimeout);
        const request = { resolve: result => { clearTimeout(timer); resolve(result); }, reject: error => { clearTimeout(timer); reject(error); } };
        pending.set(id, request);
        try { socket.send(JSON.stringify({ id, method, params })); } catch (error) { pending.delete(id); request.reject(error); }
      });
    },
    async close(closeTimeoutMs = disposalTimeoutMs) {
      rejectPending(disconnected());
      if (socket.readyState === Socket.CLOSED) { detach(); return; }
      try { await new Promise((resolve, reject) => {
        const finish = error => { clearTimeout(timer); socket.removeEventListener("close", ended); error ? reject(error) : resolve(); };
        const ended = () => queueMicrotask(() => finish());
        const timer = setTimeout(() => finish(ownershipError("E_CDP_SOCKET_CLOSE_TIMEOUT", "CDP socket did not close")), Math.min(1500, Math.max(1, closeTimeoutMs)));
        socket.addEventListener("close", ended, { once: true });
        try { socket.close(); } catch (error) { finish(error); }
      }); } finally { detach(); }
    },
  };
  try {
    // Registration precedes all handshake waits and any asynchronous exception.
    onAttempt(connection);
    await new Promise((resolve, reject) => {
      let settled = false;
      const finish = error => { if (settled) return; settled = true; clearTimeout(timer); signal?.removeEventListener("abort", aborted); socket.removeEventListener("open", opened); socket.removeEventListener("error", failed); socket.removeEventListener("close", ended); error ? reject(error) : resolve(); };
      const opened = () => finish(signal?.aborted ? signal.reason : closed ? disconnected() : undefined);
      const failed = event => finish(transport.signal.reason ?? ownershipError("E_CDP_SOCKET", "CDP socket failed", event.error));
      const ended = () => finish(disconnected());
      const aborted = () => finish(signal.reason);
      const timer = setTimeout(() => finish(ownershipError("E_CDP_CONNECT_TIMEOUT", "CDP handshake timed out")), timeoutMs);
      socket.addEventListener("open", opened, { once: true }); socket.addEventListener("error", failed, { once: true }); socket.addEventListener("close", ended, { once: true });
      signal?.addEventListener("abort", aborted, { once: true });
      if (signal?.aborted) aborted();
      else if (Number.isInteger(socket.readyState) && Number.isInteger(Socket.OPEN) && socket.readyState === Socket.OPEN) opened();
      else if (Number.isInteger(socket.readyState) && Number.isInteger(Socket.CLOSED) && socket.readyState === Socket.CLOSED) ended();
    });
    signal?.throwIfAborted();
  } catch (error) {
    await withBrowserCleanup(async () => { throw error; }, () => connection.close());
  }
  return connection;
}

export async function waitForPage(port, state, { fetcher = fetch, connectTimeoutMs = 15000, deadline = Date.now() + connectTimeoutMs, signal, now = Date.now, sleep = delay, onOwnershipError = () => {} } = {}) {
  let missingSince, lastOwnershipError;
  const options = () => { signal?.throwIfAborted(); if (now() >= deadline) throw ownershipError("E_BROWSER_CONNECT_TIMEOUT", "Page discovery deadline expired", lastOwnershipError); return { signal, timeoutMs: Math.max(1, deadline - now()) }; };
  while (now() < deadline) {
    options();
    if (state.error) throw state.error;
    let gone;
    try { gone = await state.hasExited(options()); options(); }
    catch (error) {
      if (!["E_PROCESS_INCOMPLETE", "E_PROCESS_QUERY", "E_PROCESS_QUERY_TIMEOUT"].includes(error.code)) throw error;
      lastOwnershipError = error;
      onOwnershipError(error);
      await sleep(Math.min(50, Math.max(0, deadline - now())), undefined, { signal });
      continue;
    }
    if (gone) { missingSince ??= now(); if (now() - missingSince >= 1000) throw ownershipError("E_BROWSER_EXIT", "Browser session exited before CDP became ready"); }
    else missingSince = undefined;
    let page;
    try {
      const requestSignal = AbortSignal.timeout(Math.min(1000, options().timeoutMs));
      const response = await fetcher(`http://127.0.0.1:${port}/json/list`, { signal: signal ? AbortSignal.any([signal, requestSignal]) : requestSignal });
      if (response.ok) page = (await response.json()).find(p => p.type === "page");
    } catch {}
    options();
    if (page?.webSocketDebuggerUrl) {
      if (new URL(page.webSocketDebuggerUrl).port !== String(port)) throw ownershipError("E_CDP_URL", "Unexpected CDP debug port");
      try { const verified = await state.ownership.verifyPort(options()); options(); if (verified) return page.webSocketDebuggerUrl; }
      catch (error) {
        if (!["E_PROCESS_INCOMPLETE", "E_PROCESS_QUERY", "E_PROCESS_QUERY_TIMEOUT"].includes(error.code)) throw error;
        lastOwnershipError = error;
        onOwnershipError(error);
      }
    }
    await sleep(Math.min(50, Math.max(0, deadline - now())), undefined, { signal });
  }
  throw ownershipError("E_BROWSER_CONNECT_TIMEOUT", "Browser session CDP readiness timed out", lastOwnershipError);
}

// This is the connection path used by real sessions and injected regressions.
// A single cancellation boundary spans discovery, handshake and fresh identity.
export async function connectOwnedBrowser(port, state, { Socket = WebSocket, fetcher = fetch,
  timeoutMs = 15000, now = Date.now, sleep = delay, deadline = now() + Math.min(15000, timeoutMs),
  onAttempt = () => {}, onConnection = () => {}, onReady = () => {}, signal,
  disposalTimeoutMs = 1500 } = {}) {
  const controller = new AbortController();
  let phase = "page", lastError, connection;
  const expired = () => ownershipError("E_BROWSER_CONNECT_TIMEOUT", `Browser connection deadline expired during ${phase}`, lastError);
  const remaining = () => {
    controller.signal.throwIfAborted();
    if (now() >= deadline) throw expired();
    return Math.max(1, deadline - now());
  };
  const timer = setTimeout(() => controller.abort(expired()), Math.max(1, deadline - now()));
  const externallyAborted = () => controller.abort(signal.reason);
  signal?.addEventListener("abort", externallyAborted, { once: true });
  if (signal?.aborted) externallyAborted();
  let transportSignal;
  const transportFailed = () => controller.abort(transportSignal.reason);
  const run = async action => {
    remaining();
    let aborted;
    try {
      const value = await Promise.race([Promise.resolve().then(() => { remaining(); return action(); }),
        new Promise((resolve, reject) => { aborted = () => reject(controller.signal.reason); controller.signal.addEventListener("abort", aborted, { once: true }); })]);
      remaining();
      return value;
    } finally { controller.signal.removeEventListener("abort", aborted); }
  };
  try {
    const url = await run(() => waitForPage(port, state, { fetcher, deadline, signal: controller.signal, now, sleep, onOwnershipError: error => { lastError = error; } }));
    phase = "handshake";
    connection = await connectCdp(url, { Socket, timeoutMs: remaining(), signal: controller.signal, onAttempt, disposalTimeoutMs });
    // Register before any later check can throw; cleanup always knows the socket.
    onConnection(connection);
    transportSignal = connection.transportSignal;
    transportSignal.addEventListener("abort", transportFailed, { once: true });
    if (transportSignal.aborted) transportFailed();
    remaining();
    phase = "fresh-ownership";
    while (true) {
      let sample;
      try { sample = await run(() => state.ownership.inspectPort({ signal: controller.signal, timeoutMs: remaining() })); }
      catch (error) {
        if (!["E_PROCESS_INCOMPLETE", "E_PROCESS_QUERY", "E_PROCESS_QUERY_TIMEOUT"].includes(error.code)) throw error;
        lastError = error;
      }
      remaining();
      if (sample?.ready) {
        onReady(sample.owned);
        return connection;
      }
      await run(() => sleep(Math.min(50, remaining()), undefined, { signal: controller.signal }));
    }
  } catch (error) {
    controller.abort(error);
    await withBrowserCleanup(async () => { throw error; }, async () => { if (connection) await connection.close(); });
  } finally {
    clearTimeout(timer);
    signal?.removeEventListener("abort", externallyAborted);
    transportSignal?.removeEventListener("abort", transportFailed);
    controller.abort(ownershipError("E_CONNECT_FINISHED", "Connection polling finished"));
  }
}

export async function createBrowserSession(binary, label) {
  if (process.platform !== "win32") throw ownershipError("E_BROWSER_PLATFORM", "This ownership harness requires Windows Edge");
  const boundary = await realpath(await mkdtemp(path.join(os.tmpdir(), `wuxian-${label}-`)));
  const profile = path.join(boundary, "profile");
  await mkdir(profile);
  let browserState, cdp, server;
  const attemptedTransports = createAttemptedTransportRegistry();
  const otherResources = [attemptedTransports];
  const before = await windowsProcesses();
  const startedAt = Date.now();
  const sessionRoot = before.find(p => p.pid === process.pid);
  if (!sessionRoot?.exe || !sessionRoot?.command) throw ownershipError("E_PROCESS_IDENTITY", "Cannot identify the test worker");
  let workerOwnership;
  const port = await freePort();
  // The debug port used to authorize roots is fixed before spawning.
  const browserOwnership = createProcessOwnership({ profile, binary, port, startedAt, before });
  return {
    profile, port,
    setServer(value) { server = value; },
    async start(extraArgs = []) {
      const child = spawn(binary, ["--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
        ...extraArgs, `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, "about:blank"], { stdio: ["ignore", "ignore", "pipe"], windowsHide: true });
      const launcher = watchBrowserProcess(child);
      child.stderr.resume(); browserOwnership.registerLauncher(child);
      browserState = {
        ownership: browserOwnership, child,
        get error() { return launcher.error; },
        hasExited: options => browserOwnership.hasExited(options),
        waitForExit: timeout => browserOwnership.waitForExit(timeout),
        terminate: (signal, timeout) => browserOwnership.terminate(timeout),
        dispose: () => launcher.dispose(),
      };
      console.log(`browser-session ${label}: launcher=${child.pid ?? "spawn-failed"} port=${port}`);
      child.once("exit", (code, signal) => console.log(`browser-session ${label}: launcher-exit=${code}/${signal}`));
      return child;
    },
    async connect() {
      cdp = await connectOwnedBrowser(port, browserState, {
        onAttempt: connection => attemptedTransports.register(connection),
        onConnection: connection => { cdp = connection; },
        onReady: owned => console.log(`browser-session ${label}: CDP-ready owners=${JSON.stringify(owned.map(p => ({ pid: p.pid, parent: p.parent, created: p.created })))}`),
      });
      return cdp;
    },
    trackVite(vite) {
      otherResources.push({ close: async (timeout = 4500) => {
        await closeVite(vite, workerOwnership, timeout);
      } });
    },
    async captureViteWorkers(webRoot) {
      const rows = await windowsProcesses();
      const service = rows.find(p => p.parent === process.pid && !before.some(old => old.pid === p.pid && old.created === p.created)
        && p.exe && path.resolve(p.exe).toLowerCase().startsWith(path.resolve(webRoot, "node_modules").toLowerCase() + path.sep)
        && /(?:esbuild|rolldown).*\.exe$/i.test(p.exe));
      if (service) {
        workerOwnership = createProcessOwnership({ startedAt, before, workerRoot: sessionRoot, workerExecutable: service.exe });
        await workerOwnership.refresh();
      }
    },
    async spawnVite(webRoot, vitePort) {
      const rows = await windowsProcesses(), spawnAt = Date.now();
      const child = spawn(process.execPath, [path.join(webRoot, "node_modules/vite/bin/vite.js"), "--host", "127.0.0.1", "--port", String(vitePort), "--strictPort"], { cwd: webRoot, stdio: "ignore", windowsHide: true });
      const watched = watchBrowserProcess(child);
      const owner = createProcessOwnership({ startedAt: spawnAt, before: rows, workerRoot: sessionRoot, workerExecutable: process.execPath });
      otherResources.push({ close: async (timeout = 4500) => {
        try { await closeOwnedProcess(owner, timeout); }
        finally { watched.dispose(); }
      } });
      const deadline = Date.now() + 15000;
      while (Date.now() < deadline) {
        if (watched.error) throw watched.error;
        if (watched.hasExited()) throw ownershipError("E_VITE_EXIT", `Vite exited ${child.exitCode}`);
        await owner.refresh();
        try { if ((await fetch(`http://127.0.0.1:${vitePort}/`, { signal: AbortSignal.timeout(1000) })).ok) return; } catch {}
        await delay(50);
      }
      throw ownershipError("E_VITE_START_TIMEOUT", "Vite readiness timed out");
    },
    async listenVite(vite, webRoot) {
      this.trackVite(vite);
      await withBrowserCleanup(() => bounded(() => vite.listen(), 15000, "E_VITE_START_TIMEOUT"),
        () => this.captureViteWorkers(webRoot));
    },
    async cleanup() {
      // Leave two seconds for the last bounded OS inventory/termination call.
      const deadline = Date.now() + 28000;
      // Connection failures can leave a relaunched browser without a page socket.
      await withBrowserCleanup(async () => { if (browserState && !cdp) {
        if (await browserOwnership.verifyPort()) {
          const response = await fetch(`http://127.0.0.1:${port}/json/version`, { signal: AbortSignal.timeout(1000) });
          const version = await response.json();
          if (new URL(version.webSocketDebuggerUrl).port !== String(port)) throw ownershipError("E_CDP_URL", "Unexpected browser CDP debug port");
          cdp = await connectCdp(version.webSocketDebuggerUrl, { timeoutMs: 1500, onAttempt: connection => attemptedTransports.register(connection) });
        }
      } }, () => cleanupBrowserResources({ browserState, cdp, server, ownedProfile: profile, otherResources,
        profileVerifier: async value => {
          if (value !== profile || path.dirname(value) !== boundary || await realpath(boundary) !== boundary || (await lstat(value)).isSymbolicLink()
            || await realpath(value) !== value || (browserState && !await browserState.hasExited())) throw ownershipError("E_PROFILE_OWNER", "Cannot safely remove the registered session profile");
        } }, { deadline }));
      // Boundary was created by this session and is empty after profile removal.
      await rmdir(boundary);
      console.log(`browser-session ${label}: profile-released`);
    },
  };
}
