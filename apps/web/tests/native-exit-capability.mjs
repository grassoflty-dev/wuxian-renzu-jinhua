import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { BuildCloseBarrier } from "../dist/ui/InventoryCommandController.js";

// Configuration/transport regressions only: these do not execute Tauri's native
// ACL resolver or establish that a Windows process exited.
const config = JSON.parse(await readFile(new URL("../../../server-rs/tauri.conf.json", import.meta.url), "utf8"));
const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

test("native menu close has one explicit local-only main-window capability", () => {
  assert.deepEqual(config.app.security.capabilities, [{
    identifier: "main-window-close",
    description: "Allow the bundled main menu to close its window after pending game commands drain.",
    local: true,
    windows: ["main"],
    permissions: ["core:window:allow-close"],
  }]);
  assert.equal(config.app.windows.length, 1);
  assert.equal(config.app.windows[0].label, "main");
  assert.equal(config.build.frontendDist, "../apps/web/dist");
});

test("menu exit still drains authority before closing and recovers from rejection", () => {
  const exit = main.slice(main.indexOf('exitButton.addEventListener("click"'), main.indexOf('coreOpenButton.addEventListener("click"'));
  assert.match(exit, /if \(buildCloseBarrier.closing \|\| busy\) return/);
  assert.match(exit, /if \(!isTauri\(\)\)/);
  assert.match(exit, /buildCloseBarrier.close\(async \(\) => \{\s*await waitForHubBuild\(\);\s*if \(sessionLoop\?\.acceptsExternalResults\) await sessionLoop.runWithSession\(async \(\) => undefined\);\s*\}, \(\) => getCurrentWindow\(\).close\(\)\)/);
  assert.match(exit, /catch\(error => \{\s*exitButton.disabled = false/);
  assert.doesNotMatch(exit, /destroy\(|process.exit|window.close\(/);
});

test("installed Tauri API emits the close IPC for the configured main window", async () => {
  const previous = globalThis.window;
  globalThis.window = {};
  try {
    const calls = [];
    mockWindows(config.app.windows[0].label ?? "main");
    mockIPC((command, args) => { calls.push([command, args]); });
    const barrier = new BuildCloseBarrier();
    const order = [];
    let release;
    const pending = new Promise(resolve => { release = resolve; });
    const closing = barrier.close(async () => {
      order.push("drain"); await pending; order.push("drained");
    }, async () => { await getCurrentWindow().close(); order.push("close"); });
    assert.equal(barrier.closing, true);
    assert.deepEqual(calls, []);
    await assert.rejects(barrier.close(async () => {}, async () => assert.fail("duplicate close")), /E_BUILD_CLOSING/);
    release();
    await closing;
    assert.deepEqual(order, ["drain", "drained", "close"]);
    assert.deepEqual(calls, [["plugin:window|close", { label: "main" }]]);
  } finally {
    clearMocks();
    if (previous === undefined) delete globalThis.window;
    else globalThis.window = previous;
  }
});

test("failed drain cannot close; rejected close allows an explicit fresh retry", async () => {
  const barrier = new BuildCloseBarrier();
  let closeCalls = 0;
  await assert.rejects(barrier.close(async () => { throw new Error("save failed"); },
    async () => { closeCalls++; }), /save failed/);
  assert.equal(closeCalls, 0);
  assert.equal(barrier.closing, false);
  await assert.rejects(barrier.close(async () => {},
    async () => { closeCalls++; throw new Error("Command plugin:window|close not allowed by ACL"); }), /not allowed by ACL/);
  assert.equal(barrier.closing, false);
  await barrier.close(async () => {}, async () => { closeCalls++; });
  assert.equal(closeCalls, 2);
  assert.equal(barrier.closing, true);
});
