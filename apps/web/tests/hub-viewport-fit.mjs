import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { waitForStableHubLayout } from "./support/hub-layout-readiness.mjs";

const webRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function findEdge() {
  const roots = [process.env["PROGRAMFILES(X86)"], process.env.ProgramFiles].filter(Boolean);
  return roots.map((root) => path.join(root, "Microsoft", "Edge", "Application", "msedge.exe"))
    .find((candidate) => existsSync(candidate));
}

async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => server.once("error", reject).listen(0, "127.0.0.1", resolve));
  const { port } = server.address();
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function waitForPage(port, processHandle) {
  const end = Date.now() + 15_000;
  while (Date.now() < end) {
    if (processHandle.exitCode !== null) throw new Error(`Edge exited with code ${processHandle.exitCode}`);
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      const pages = await response.json();
      const page = pages.find((target) => target.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    await delay(100);
  }
  throw new Error("Timed out waiting for Edge DevTools");
}

async function connectCdp(url) {
  const socket = new WebSocket(url);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  let nextId = 0;
  const pending = new Map();
  socket.addEventListener("message", ({ data }) => {
    const message = JSON.parse(data);
    if (!message.id) return;
    const callbacks = pending.get(message.id);
    if (!callbacks) return;
    pending.delete(message.id);
    if (message.error) callbacks.reject(new Error(message.error.message));
    else callbacks.resolve(message.result);
  });
  return {
    send(method, params = {}) {
      const id = ++nextId;
      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
    close() { socket.close(); },
  };
}

test("hub menu and save picker fit common viewports and remain reachable in short windows", { timeout: 90_000 }, async (t) => {
  const edge = findEdge();
  if (!edge) return t.skip("Microsoft Edge is not installed at its standard Windows path");

  const profile = await mkdtemp(path.join(os.tmpdir(), "hub-viewport-fit-"));
  const [port, vitePort] = await Promise.all([freePort(), freePort()]);
  const vite = spawn(process.execPath, [
    path.join(webRoot, "node_modules", "vite", "bin", "vite.js"),
    "--host", "127.0.0.1", "--port", String(vitePort), "--strictPort",
  ], { cwd: webRoot, stdio: "ignore", windowsHide: true });
  const browser = spawn(edge, [
    "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
    `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, "about:blank",
  ], { stdio: "ignore", windowsHide: true });
  let cdp;
  try {
    const serverDeadline = Date.now() + 15_000;
    while (Date.now() < serverDeadline) {
      if (vite.exitCode !== null) throw new Error(`Vite exited with code ${vite.exitCode}`);
      try { if ((await fetch(`http://127.0.0.1:${vitePort}/`)).ok) break; } catch {}
      await delay(100);
    }
    cdp = await connectCdp(await waitForPage(port, browser));
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Page.navigate", { url: `http://127.0.0.1:${vitePort}/` });
    const appDeadline = Date.now() + 15_000;
    while (Date.now() < appDeadline) {
      const ready = await cdp.send("Runtime.evaluate", {
        expression: "Boolean(document.querySelector('#continue-picker') && document.querySelector('.actions button'))",
        returnByValue: true,
      });
      if (ready.result.value) break;
      await delay(100);
    }
    const hiddenPicker = await cdp.send("Runtime.evaluate", {
      expression: "(() => { const p=document.querySelector('#continue-picker'); return {hidden:p.hidden,display:getComputedStyle(p).display,rect:p.getBoundingClientRect().height}; })()",
      returnByValue: true,
    });
    assert.deepEqual(hiddenPicker.result.value, { hidden: true, display: "none", rect: 0 }, "the native menu starts with its save picker hidden");
    await cdp.send("Runtime.evaluate", {
      expression: `(() => {
        const picker=document.querySelector('#continue-picker');
        picker.hidden=false;
        document.querySelector('#continue-slot').innerHTML='<option>自动存档</option>';
        document.querySelector('#continue-slot-note').textContent='选择要继续的有效存档。';
        document.querySelector('#feedback').textContent='正在读取当前状态…';
      })()`,
    });

    const sizes = [
      { width: 1280, height: 670, scroll: false },
      { width: 1280, height: 672, scroll: false },
      { width: 1024, height: 537, scroll: false },
      { width: 1280, height: 720, scroll: false },
      { width: 1600, height: 937, scroll: false },
      { width: 1920, height: 1080, scroll: false },
      { width: 2560, height: 1440, scroll: false },
      { width: 1280, height: 480, scroll: true },
      { width: 760, height: 720, scroll: true },
    ];
    for (const size of sizes) {
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width: size.width, height: size.height, deviceScaleFactor: 1, mobile: false,
      });
      await cdp.send("Runtime.evaluate", { expression: "document.querySelector('.hub').scrollTop = 0" });
      const settled = await cdp.send("Runtime.evaluate", {
        expression: `(${waitForStableHubLayout.toString()})(${JSON.stringify({ width: size.width, height: size.height })}, 2000)`,
        awaitPromise: true, returnByValue: true,
      });
      const readiness = settled.result?.value;
      t.diagnostic(`hub layout readiness ${size.width}x${size.height}: ${JSON.stringify(readiness ?? settled.exceptionDetails)}`);
      assert.equal(readiness?.ready, true, `${size.width}x${size.height}: font/viewport/layout readiness must settle within 2s`);
      const result = await cdp.send("Runtime.evaluate", {
        expression: `(() => {
          const hub = document.querySelector('.hub');
          const copy = document.querySelector('.hub-copy');
          const rect = (selector) => { const r = document.querySelector(selector).getBoundingClientRect(); return {top:r.top,bottom:r.bottom,left:r.left,right:r.right}; };
          const copyRect = copy.getBoundingClientRect();
          const hubRect = hub.getBoundingClientRect();
          const footerRect = rect('.footer');
          const headerRect = rect('.topbar');
          const content = [...copy.querySelectorAll('.eyebrow,h1,.lead,.actions,.save-picker,.feedback')].map((el) => ({
            name: el.className || el.tagName, top: el.getBoundingClientRect().top, bottom: el.getBoundingClientRect().bottom,
          }));
          const actions = [...copy.querySelectorAll('.actions button')].map((el) => ({top:el.getBoundingClientRect().top,bottom:el.getBoundingClientRect().bottom}));
          const pickerItems = ['label[for="continue-slot"]','#continue-slot','#continue-slot-note','#continue-selected'].map((selector) => {
            const r=document.querySelector(selector).getBoundingClientRect(); return {selector,top:r.top,bottom:r.bottom,left:r.left,right:r.right};
          });
          return {viewport:{width:innerWidth,height:innerHeight,documentWidth:document.documentElement.scrollWidth,documentHeight:document.documentElement.scrollHeight,documentClientWidth:document.documentElement.clientWidth,documentClientHeight:document.documentElement.clientHeight},hubHeight:hub.clientHeight,hubScrollHeight:hub.scrollHeight,hubScrollTop:hub.scrollTop,
            copy:{top:copyRect.top,bottom:copyRect.bottom,left:copyRect.left,right:copyRect.right},
            hub: {top:hubRect.top,bottom:hubRect.bottom}, header:headerRect, footer:footerRect,
            content, actions, picker:rect('.save-picker'), pickerItems, feedback:rect('.feedback'),
            network:rect('.network')};
        })()`, returnByValue: true,
      });
      const metrics = result.result.value;
      const label = `${size.width}x${size.height}`;
      assert.equal(metrics.actions.length, 4, `${label}: all four menu actions should exist`);
      assert.ok(metrics.picker.bottom > metrics.picker.top, `${label}: save picker should be visible`);
      assert.ok(metrics.feedback.bottom > metrics.feedback.top, `${label}: feedback should be visible`);
      assert.ok(metrics.viewport.documentWidth <= metrics.viewport.documentClientWidth, `${label}: no horizontal document overflow`);
      assert.ok(metrics.viewport.documentHeight <= metrics.viewport.documentClientHeight, `${label}: no vertical document overflow`);
      if (size.scroll) {
        assert.ok(metrics.hubScrollHeight > metrics.hubHeight, `${label}: short layout should expose an internal scroll area`);
        await cdp.send("Runtime.evaluate", { expression: "document.querySelector('.hub').scrollTop = document.querySelector('.hub').scrollHeight" });
        const scrolled = await cdp.send("Runtime.evaluate", {
          expression: "(() => { const h=document.querySelector('.hub'), f=document.querySelector('.feedback').getBoundingClientRect(), p=document.querySelector('#continue-selected').getBoundingClientRect(), r=h.getBoundingClientRect(); return {scrollTop:h.scrollTop,feedbackBottom:f.bottom,pickerButtonBottom:p.bottom,hubBottom:r.bottom}; })()",
          returnByValue: true,
        });
        assert.ok(scrolled.result.value.scrollTop > 0, `${label}: the menu can scroll`);
        assert.ok(scrolled.result.value.feedbackBottom <= scrolled.result.value.hubBottom, `${label}: feedback is reachable by scrolling`);
        assert.ok(scrolled.result.value.pickerButtonBottom <= scrolled.result.value.hubBottom, `${label}: save action is reachable by scrolling`);
      } else {
        assert.ok(metrics.hubScrollHeight <= metrics.hubHeight + 1, `${label}: hub must fit without scrolling (${metrics.hubScrollHeight}/${metrics.hubHeight})`);
        assert.ok(metrics.copy.top >= metrics.header.bottom, `${label}: menu must clear the top bar`);
        assert.ok(metrics.copy.bottom <= metrics.footer.top, `${label}: menu must clear the footer`);
        for (const item of metrics.content) {
          assert.ok(item.top >= metrics.header.bottom, `${label}: ${item.name} must clear the top bar`);
          assert.ok(item.bottom <= metrics.footer.top, `${label}: ${item.name} must clear the footer`);
        }
        for (const item of metrics.pickerItems) {
          assert.ok(item.top >= metrics.header.bottom, `${label}: ${item.selector} must clear the top bar`);
          assert.ok(item.bottom <= metrics.footer.top, `${label}: ${item.selector} must clear the footer`);
          assert.ok(item.left >= 0 && item.right <= metrics.viewport.documentClientWidth, `${label}: ${item.selector} must fit horizontally`);
        }
        for (let index = 1; index < metrics.actions.length; index++) {
          assert.ok(metrics.actions[index].top >= metrics.actions[index - 1].bottom, `${label}: menu actions must not overlap`);
        }
      }
    }
  } finally {
    try { await cdp?.send("Browser.close"); } catch {}
    if (browser.exitCode === null && process.platform === "win32") {
      spawnSync("taskkill", ["/PID", String(browser.pid), "/T", "/F"], { windowsHide: true, stdio: "ignore" });
    } else if (browser.exitCode === null) {
      browser.kill();
    }
    if (browser.exitCode === null) {
      await Promise.race([
        new Promise((resolve) => browser.once("exit", resolve)),
        delay(1_000),
      ]);
    }
    cdp?.close();
    if (vite.exitCode === null && process.platform === "win32") {
      spawnSync("taskkill", ["/PID", String(vite.pid), "/T", "/F"], { windowsHide: true, stdio: "ignore" });
    } else if (vite.exitCode === null) {
      vite.kill();
    }
    for (let attempt = 0; attempt < 5; attempt++) {
      try {
        await rm(profile, { recursive: true, force: true, maxRetries: 3, retryDelay: 200 });
        break;
      } catch (error) {
        if (attempt === 4) throw error;
        await delay(200);
      }
    }
  }
});
