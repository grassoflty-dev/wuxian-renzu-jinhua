import { createBrowserSession, freePort, startHttpServer, withBrowserCleanup } from "./support/browser-session.mjs";
import { bounded } from "./support/browser-cleanup.mjs";
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
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

test("hub menu and save picker fit common viewports and remain reachable in short windows", { timeout: 90_000 }, async (t) => {
  const edge = findEdge();
  if (!edge) return t.skip("Microsoft Edge is not installed at its standard Windows path");

  const session = await createBrowserSession(edge, "hub-viewport");
  const vitePort = await freePort();
  let cdp;
  await withBrowserCleanup(async () => {
  try {
    await session.spawnVite(webRoot, vitePort);
    await session.start();
    cdp = await session.connect();
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
    // Resource cleanup follows through withBrowserCleanup, including body failure.
  }
  }, () => session.cleanup());
});
