import { bounded, withBrowserCleanup } from "./browser-cleanup.mjs";

const failure = (code, message) => Object.assign(new Error(message), { code });

// Registration precedes allocation. The bounded waits never own or discard the
// factory/listen tasks: the registered adapter retains their settlement records.
export function createViteSessionResource(createServer, register, { timeoutMs = 15000, now = Date.now } = {}) {
  const deadline = now() + timeoutMs;
  const state = { creation: "pending", listen: "idle", close: "idle", closeRequested: false, closed: false };
  let server, creation, creationSettlement, listenSettlement, listenWait, closeTask;
  const closing = () => failure("E_VITE_CLOSING", "Vite resource has been requested to close");
  const ensureOpen = () => { if (state.closeRequested) throw closing(); };
  const observe = (task, phase) => task.then(value => {
    state[phase] = "settled";
    return { ok: true, value };
  }, error => {
    state[phase] = "failed";
    state[`${phase}Error`] = error;
    return { ok: false, error };
  });
  const valueOf = outcome => { if (!outcome.ok) throw outcome.error; return outcome.value; };
  const resource = {
    get state() { return { ...state }; },
    async ready() {
      try {
        ensureOpen();
        if (now() >= deadline) throw failure("E_VITE_CREATE_TIMEOUT", "Vite creation deadline expired");
        await bounded(async () => valueOf(await creationSettlement), deadline - now(), "E_VITE_CREATE_TIMEOUT");
        if (now() >= deadline) throw failure("E_VITE_CREATE_TIMEOUT", "Vite creation deadline expired");
        ensureOpen();
        return resource;
      } catch (error) {
        state.readyError ??= error;
        resource.close();
        throw error;
      }
    },
    listen() {
      if (state.closeRequested) return Promise.reject(closing());
      if (listenWait) return listenWait;
      const listenDeadline = now() + timeoutMs;
      const task = Promise.resolve().then(async () => {
        await resource.ready();
        ensureOpen();
        return server.listen();
      });
      state.listen = "pending";
      listenSettlement = observe(task, "listen");
      listenWait = (async () => {
        try {
          if (now() >= listenDeadline) throw failure("E_VITE_START_TIMEOUT", "Vite listen deadline expired");
          const result = await bounded(async () => valueOf(await listenSettlement), listenDeadline - now(), "E_VITE_START_TIMEOUT");
          if (now() >= listenDeadline) throw failure("E_VITE_START_TIMEOUT", "Vite listen deadline expired");
          ensureOpen();
          return result;
        } catch (error) {
          state.listenWaitError = error;
          resource.close();
          throw error;
        }
      })();
      return listenWait;
    },
    close() {
      state.closeRequested = true;
      if (closeTask) return closeTask;
      state.close = "pending";
      closeTask = withBrowserCleanup(async () => {
        valueOf(await creationSettlement);
        if (listenSettlement) valueOf(await listenSettlement);
      }, async () => {
        if (server) {
          await server.close();
          state.closed = true;
        }
      }).then(() => { state.close = "settled"; }, error => {
        state.close = "failed";
        state.closeError = error;
        throw error;
      });
      // Automatic close requests can outlive a timed-out caller. Keep rejection
      // observed here; every close caller still receives this same rejecting task.
      closeTask.catch(() => {});
      return closeTask;
    },
  };
  let start, rejectRegistration;
  creation = new Promise((resolve, reject) => { start = resolve; rejectRegistration = reject; }).then(createServer);
  creationSettlement = observe(creation.then(value => { server = value; return value; }), "creation");
  try { register(resource); }
  catch (error) { rejectRegistration(error); throw error; }
  start();
  return resource;
}
