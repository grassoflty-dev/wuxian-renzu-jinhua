const endpoint = process.env.ENTRY01_CDP_ENDPOINT || "http://127.0.0.1:9222";
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));

async function waitForPage() {
  const deadline = Date.now() + 60_000;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const targets = await fetch(`${endpoint}/json`).then(response => {
        if (!response.ok) throw new Error(`CDP_HTTP_${response.status}`);
        return response.json();
      });
      const page = targets.find(target => target.type === "page" && target.webSocketDebuggerUrl);
      if (page) return page;
    } catch (error) { lastError = error; }
    await pause(500);
  }
  throw new Error(`E_CDP_PAGE_UNAVAILABLE: ${String(lastError || "timeout")}`);
}

const page = await waitForPage();
const socket = new WebSocket(page.webSocketDebuggerUrl);
const pending = new Map();
const consoleErrors = [];
let nextId = 0;

await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});

socket.addEventListener("message", event => {
  const message = JSON.parse(event.data);
  if (message.method === "Runtime.exceptionThrown") {
    consoleErrors.push(message.params.exceptionDetails.text || "Runtime exception");
  }
  if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") {
    consoleErrors.push(message.params.args.map(arg => arg.value || arg.description || "").join(" "));
  }
  if (message.id && pending.has(message.id)) {
    const entry = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) entry.reject(new Error(message.error.message));
    else entry.resolve(message.result);
  }
});

function command(method, params = {}) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`E_CDP_TIMEOUT:${method}`));
    }, 10_000);
    pending.set(id, {
      resolve: value => { clearTimeout(timeout); resolve(value); },
      reject: error => { clearTimeout(timeout); reject(error); },
    });
    socket.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(expression) {
  const response = await command("Runtime.evaluate", {
    expression, awaitPromise: true, returnByValue: true, userGesture: true,
  });
  if (response.exceptionDetails) {
    const detail = response.exceptionDetails.exception?.description || response.exceptionDetails.text;
    throw new Error(detail);
  }
  if (response.result?.subtype === "error") throw new Error(response.result.description);
  return response.result?.value;
}

try {
  await command("Runtime.enable");
  await command("Page.enable");
  const ready = await evaluate(`({
    title: document.title,
    tauri: Boolean(window.__TAURI__?.core?.invoke),
    babylon: Boolean(window.BABYLON?.Engine),
    formalView: Boolean(window.FormalView && window.FreezeV02RendererBoot),
    configuredScripts: [...document.scripts].map(script => script.getAttribute('src')).filter(Boolean)
  })`);
  if (!ready.tauri || !ready.formalView || !ready.babylon) {
    throw new Error(`E_ENTRY_PRECONDITION:${JSON.stringify(ready)}`);
  }
  await evaluate(`(() => {
    window.__entry01GlDraws = 0;
    const proto = WebGL2RenderingContext.prototype;
    for (const name of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced']) {
      const original = proto[name];
      if (typeof original === 'function') proto[name] = function(...args) {
        window.__entry01GlDraws++;
        return original.apply(this, args);
      };
    }
  })()`);
  await evaluate(`document.getElementById('newGameButton').click()`);

  let result;
  const deadline = Date.now() + 12_000;
  while (Date.now() < deadline) {
    result = await evaluate(`(() => {
      const canvas = document.getElementById('gameCanvas');
      const gl = canvas && canvas.getContext('webgl2');
      return {
        worldVisible: !document.getElementById('worldView').hidden,
        title: document.getElementById('worldTitle').textContent,
        appServerTick: Number(document.getElementById('worldView').dataset.serverTick || 0),
        appAckSeq: Number(document.getElementById('worldView').dataset.ackSeq || 0),
        view: window.__formalSmokeView || null,
        glVersion: gl?.getParameter(gl.VERSION) || null,
        glRenderer: gl?.getParameter(gl.RENDERER) || null,
        contextLost: gl?.isContextLost() ?? true,
        drawCalls: window.__entry01GlDraws,
        toast: document.getElementById('toastRegion').textContent
      };
    })()`);
    if (result.worldVisible && result.glVersion && result.drawCalls > 0) break;
    await pause(250);
  }
  await pause(500);
  process.stdout.write(`RENDERER_STATE ${JSON.stringify(result)}\n`);

  const backend = await evaluate(`(async () => {
    const before = await window.__TAURI__.core.invoke('formal_snapshot');
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'w', bubbles: true }));
    await new Promise(resolve => setTimeout(resolve, 350));
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'w', bubbles: true }));
    await new Promise(resolve => setTimeout(resolve, 150));
    document.getElementById('worldView').hidden = true;
    await new Promise(resolve => setTimeout(resolve, 125));
    const view = await window.__TAURI__.core.invoke('formal_snapshot');
    const after = await window.__TAURI__.core.invoke('formal_submit_input', {
      sample: {
        protocol: 'continuous-input', worldEpoch: view.worldEpoch,
        seq: view.ackSeq + 1, clientTimeMs: Math.max(view.serverTimeMs, Math.floor(performance.now())) + 1000,
        moveX: 0, moveZ: 0
      },
      combat: []
    });
    return { beforeAckSeq: before.ackSeq, beforeZ: before.player.transform.positionM.zM,
      movedZ: view.player.transform.positionM.zM, movementDeltaZ: view.player.transform.positionM.zM - before.player.transform.positionM.zM,
      worldId: after.worldId, worldEpoch: after.worldEpoch, serverTick: after.serverTick,
      authorityRevision: after.authorityRevision, ackSeq: after.ackSeq };
  })()`);

  result = { ...result, pageTitle: ready.title, tauri: ready.tauri, babylon: ready.babylon,
    formalView: ready.formalView, configuredScripts: ready.configuredScripts, backend, consoleErrors };
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  if (!result.worldVisible || !result.glVersion?.includes("WebGL 2") || result.contextLost ||
      result.drawCalls < 1 || backend.worldId !== "grey_hive" || backend.serverTick < 1 ||
      backend.ackSeq < 1 || backend.authorityRevision < 1 || backend.beforeAckSeq < 1 ||
      backend.movementDeltaZ < 0.05 || consoleErrors.length > 0) {
    throw new Error("E_ENTRY_SMOKE_ASSERTION");
  }
} finally {
  socket.close();
}
