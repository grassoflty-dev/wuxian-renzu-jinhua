import { rm } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";

function failure(code, message, cause) {
  return Object.assign(new Error(message, { cause }), { code });
}

export async function bounded(action, timeoutMs, code) {
  let timer;
  try {
    return await Promise.race([
      Promise.resolve().then(action),
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(failure(code, code)), timeoutMs);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

// Register immediately after spawn: a spawn failure has no exit event, and an
// exit can happen while the CDP command is still being acknowledged.
export function watchBrowserProcess(child) {
  let exited = child.exitCode !== null || child.signalCode !== null;
  let processError;
  const waiters = new Set();
  const onExit = () => {
    exited = true;
    for (const notify of [...waiters]) notify(true);
  };
  const onError = error => {
    processError = error;
    if (child.pid == null) onExit();
  };
  child.on("exit", onExit);
  child.on("error", onError);
  return {
    child,
    get error() { return processError; },
    hasExited: () => exited,
    waitForExit(timeoutMs) {
      if (exited) return Promise.resolve(true);
      return new Promise(resolve => {
        const finish = value => {
          clearTimeout(timer);
          waiters.delete(finish);
          resolve(value);
        };
        const timer = setTimeout(() => finish(false), timeoutMs);
        waiters.add(finish);
      });
    },
    dispose() {
      child.off("exit", onExit);
      child.off("error", onError);
    },
  };
}

export async function removeOwnedProfile(profile, {
  remove = rm,
  sleep = delay,
  attempts = 5,
  retryDelayMs = 100,
  verify,
  attemptTimeoutMs = 1000,
  deadline = Infinity,
} = {}) {
  if (!Number.isInteger(attempts) || attempts < 1) throw new RangeError("Profile removal needs at least one attempt");
  if (remove === rm && typeof verify !== "function") throw failure("E_PROFILE_OWNER", "Profile removal requires a registered ownership verifier");
  for (let attempt = 1; ; attempt++) {
    try {
      if (Date.now() >= deadline) throw failure("E_CLEANUP_BUDGET", "Profile removal budget exhausted");
      if (verify) await verify(profile);
      // Explicit retries keep the maximum attempt count visible and testable.
      await bounded(() => remove(profile, { recursive: true, force: true, maxRetries: 0 }), Math.min(attemptTimeoutMs, Math.max(1, deadline - Date.now())), "E_PROFILE_REMOVE_TIMEOUT");
      return;
    } catch (error) {
      if (!["EBUSY", "EPERM", "ENOTEMPTY"].includes(error.code) || attempt >= attempts) throw error;
      await sleep(retryDelayMs * attempt);
    }
  }
}

export async function cleanupBrowserResources({ browserState, cdp, server, ownedProfile, otherResources = [], profileVerifier }, {
  cdpCloseTimeoutMs = 1500,
  gracefulExitTimeoutMs = 3000,
  terminateExitTimeoutMs = 3000,
  killExitTimeoutMs = 3000,
  serverCloseTimeoutMs = 1500,
  profileRemoval = {},
  deadline = Date.now() + 30000,
} = {}) {
  const errors = [];
  const attempt = async action => {
    try { await action(); } catch (error) { errors.push(error); }
  };
  let exited = false;
  const budget = requested => {
    if (Date.now() >= deadline) throw failure("E_CLEANUP_BUDGET", "Browser cleanup budget exhausted; retaining unverified resources");
    return Math.min(requested, deadline - Date.now());
  };
  try {
    await attempt(async () => { exited = !browserState || await browserState.hasExited(); });
    if (!exited) {
      let closeError;
      if (cdp) {
        try {
          const timeout = budget(cdpCloseTimeoutMs);
          await bounded(() => cdp.send("Browser.close", {}, { timeoutMs: timeout }),
            timeout, "E_CDP_BROWSER_CLOSE_TIMEOUT");
        } catch (error) { closeError = error; }
        await attempt(async () => { exited = await browserState.waitForExit(budget(gracefulExitTimeoutMs)); });
      }
      // Chromium may close the transport before acknowledging Browser.close.
      // Only that specific disconnect, plus confirmed exit, is expected.
      if (closeError && !(exited && closeError.code === "E_CDP_CLOSED")) errors.push(closeError);
      for (const [signal, timeoutMs] of [["SIGTERM", terminateExitTimeoutMs], ["SIGKILL", killExitTimeoutMs]]) {
        if (exited) break;
        await attempt(async () => {
          // This is the exact ChildProcess returned by this test's spawn.
          // Never kill by executable name, PID group, or system-wide command.
          if (browserState.terminate) await browserState.terminate(signal, budget(timeoutMs));
          else if (!profileRemoval.remove) throw failure("E_PROCESS_OWNER", "Real browser termination requires a session ownership registry");
          else if (!browserState.child.kill(signal) && !browserState.hasExited()) {
            throw failure("E_BROWSER_TERMINATE", `Browser rejected ${signal}`, browserState.error);
          }
        });
        await attempt(async () => { exited = await browserState.waitForExit(budget(timeoutMs)); });
      }
      if (!exited) errors.push(failure("E_BROWSER_EXIT_TIMEOUT",
        `Spawned browser did not exit; retaining owned profile: ${ownedProfile ?? "none"}`));
    }
    if (cdp) await attempt(() => bounded(() => cdp.close(), budget(cdpCloseTimeoutMs), "E_CDP_SOCKET_CLOSE_TIMEOUT"));
    if (server?.listening) {
      await attempt(() => bounded(() => new Promise((resolve, reject) => {
        server.close(error => error ? reject(error) : resolve());
        server.closeAllConnections?.();
      }), budget(serverCloseTimeoutMs), "E_BROWSER_SERVER_CLOSE_TIMEOUT"));
    }
    const consumerClosures = [];
    for (const resource of otherResources) {
      const closure = { resource, closed: false };
      consumerClosures.push(closure);
      try {
        await resource.close(budget(4500));
        closure.closed = true;
      } catch (error) {
        closure.error = error;
        errors.push(error);
      }
    }
    // Browser exit alone does not release registered consumers' profile data.
    // A rejected close stays unconfirmed even if its underlying work finishes later.
    if (exited && consumerClosures.every(closure => closure.closed) && ownedProfile) await attempt(() => removeOwnedProfile(ownedProfile, { ...profileRemoval, deadline, verify: profileVerifier ?? profileRemoval.verify }));
  } finally {
    await attempt(() => browserState?.dispose?.());
  }
  if (errors.length) throw new AggregateError(errors, "Browser cleanup failed");
}

// A finally block that throws would hide the assertion failure we need to see.
export async function withBrowserCleanup(body, cleanup) {
  let result;
  let bodyFailed = false;
  let bodyError;
  try { result = await body(); } catch (error) { bodyFailed = true; bodyError = error; }
  try {
    await cleanup();
  } catch (cleanupError) {
    if (bodyFailed) throw new AggregateError([bodyError, cleanupError],
      "Browser test body and cleanup both failed", { cause: bodyError });
    throw cleanupError;
  }
  if (bodyFailed) throw bodyError;
  return result;
}
