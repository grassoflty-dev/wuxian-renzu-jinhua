import { createBrowserSession, freePort, startHttpServer, withBrowserCleanup } from "./support/browser-session.mjs";
import { bounded } from "./support/browser-cleanup.mjs";
import test from "node:test";
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
function browserPath() {
  return [process.env.INVENTORY_TEST_BROWSER,
    ...[process.env["PROGRAMFILES(X86)"], process.env.ProgramFiles].filter(Boolean).map(p => path.join(p,"Microsoft","Edge","Application","msedge.exe")),
    // Native acceptance targets Windows Edge. Other browsers are explicit opt-in.
  ].filter(Boolean).find(p => existsSync(p));
}
const html=`<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="/src/style.css"><link rel="stylesheet" href="/src/visual-authority.css">
<button id="trigger">背包入口</button><section class="core-panel" id="core-panel" hidden role="dialog" aria-modal="true" aria-labelledby="core-panel-title">
<div class="core-panel-frame"><header class="core-panel-header"><div><h2 id="core-panel-title"></h2><p id="core-panel-subtitle"></p></div><button id="core-panel-close">关闭</button></header>
<div class="core-panel-layout"><div class="core-panel-navigation"><nav class="core-panel-tabs" role="tablist"><button role="tab" data-core-panel="inventory">背包</button><button role="tab" data-core-panel="character">角色</button></nav></div><div class="core-panel-content" id="core-panel-content"></div></div></div></section>`;

test("real browser Inventory panel preserves authoritative rows, keyboard focus, pending state and close/epoch safety", {timeout:60000}, async t => {
  const binary=browserPath(); if(!binary)return t.skip("Microsoft Edge is not installed; INVENTORY_TEST_BROWSER may select an explicit test browser");
  const server=createServer(async(req,res)=>{try{
    const pathname=new URL(req.url,"http://127.0.0.1").pathname;
    if(pathname==="/"){res.setHeader("Content-Type","text/html; charset=utf-8");res.end(html);return;}
    if(!/^\/(dist\/.*\.js|src\/(style|visual-authority)\.css|tests\/fixtures\/inventory-snapshot\.mjs)$/.test(pathname)||pathname.includes("..")){res.writeHead(404);res.end();return;}
    res.setHeader("Content-Type",pathname.endsWith(".css")?"text/css":"text/javascript");res.end(await readFile(path.join(root,pathname.slice(1))));
  }catch{res.writeHead(404);res.end();}});
  const session=await createBrowserSession(binary,"inventory");
  session.setServer(server);
  let cdp;const evaluate=async expression=>{const result=await cdp.send("Runtime.evaluate",{expression,awaitPromise:true,returnByValue:true});if(result.exceptionDetails)throw new Error(result.exceptionDetails.exception?.description||result.exceptionDetails.text);return result.result.value;};
  const waitFor = async (expression, label) => {
    const deadline = Date.now() + 5000;
    while (!(await evaluate(expression))) {
      if (Date.now() >= deadline) {
        const diagnostic = await evaluate(`({focus:document.activeElement?.outerHTML,calls:window.fixture?.calls.length,keys:window.fixture?.keys,busy:document.querySelector('#core-panel-content')?.getAttribute('aria-busy')})`);
        throw new Error(`E_INVENTORY_READINESS:${label}:${JSON.stringify(diagnostic)}`);
      }
      await delay(20);
    }
  };
  await withBrowserCleanup(async () => {
    await startHttpServer(server);
    await session.start();
    console.log("inventory-browser: spawned");
    cdp=await session.connect(); console.log("inventory-browser: connected");await cdp.send("Page.enable");await cdp.send("Runtime.enable");
    await cdp.send("Emulation.setDeviceMetricsOverride",{width:1280,height:720,deviceScaleFactor:1,mobile:false});
    await cdp.send("Page.navigate",{url:`http://127.0.0.1:${server.address().port}/`});
    await waitFor("!!document.querySelector('#core-panel-content')", "panel document");
    await cdp.send("Page.bringToFront");
    console.log("inventory-browser: page ready");
    await evaluate(`(async()=>{
      const {CoreUiPresenter}=await import('/dist/ui/CoreUiPresenter.js');
      const {isCurrentBuildReceipt}=await import('/dist/ui/InventoryCommandController.js');
      const f=await import('/tests/fixtures/inventory-snapshot.mjs');
      const data=window.fixture={...f,calls:[],resolvers:[],latest:null,keys:[],nativeClicks:[]};
      for (const type of ['keydown','keypress','keyup']) document.addEventListener(type,event=>data.keys.push({type:event.type,key:event.key,trusted:event.isTrusted}));
      document.addEventListener('click',event=>data.nativeClicks.push({trusted:event.isTrusted,detail:event.detail,target:event.target.dataset.inventoryFocus}),true);
      data.presenter=new CoreUiPresenter(document.querySelector('#core-panel'),{},undefined,(action,source)=>{
        data.calls.push({action,source});return new Promise(resolve=>data.resolvers.push(receipt=>{
          if(isCurrentBuildReceipt(true,data.latest,source,receipt))data.apply(receipt.snapshot);
          resolve(receipt);
        }));
      });
      data.apply=snapshot=>{data.latest=snapshot;data.presenter.apply(snapshot);};
      data.apply(f.inventorySnapshot());data.presenter.open(document.querySelector('#trigger'),'inventory');
    })()`);
    console.log("inventory-browser: fixture ready");
    assert.deepEqual(await evaluate(`({headings:[...document.querySelectorAll('.inventory-column h3')].map(n=>n.textContent),items:document.querySelectorAll('.inventory-item').length,equip:document.querySelector('[data-inventory-focus^="equip:"]').disabled,busy:document.querySelector('#core-panel-content').getAttribute('aria-busy'),focus:document.activeElement.id})`),
      {headings:["物品","装备","描述"],items:1,equip:false,busy:"false",focus:"core-panel-close"});
    // Native Enter activates the real button; no synthetic click handler shortcut.
    await evaluate(`document.querySelector('[data-inventory-focus^="equip:"]').focus()`);
    assert.equal(await evaluate("document.activeElement.dataset.inventoryFocus"), "equip:rear_view_lens");
    // Enter includes its character payload so Chromium performs native button
    // activation, as a real keyboard does. A raw keydown without text is incomplete.
    await cdp.send("Input.dispatchKeyEvent",{type:"keyDown",key:"Enter",code:"Enter",windowsVirtualKeyCode:13,text:"\r",unmodifiedText:"\r"});
    await cdp.send("Input.dispatchKeyEvent",{type:"keyUp",key:"Enter",code:"Enter",windowsVirtualKeyCode:13});
    await waitFor("fixture.calls.length > 0", "native Enter command");
    assert.deepEqual(await evaluate("fixture.nativeClicks"), [{trusted:true,detail:0,target:"equip:rear_view_lens"}]);
    assert.ok(await evaluate("fixture.keys.some(event=>event.type==='keypress' && event.key==='Enter' && event.trusted)"));
    assert.deepEqual(await evaluate(`({calls:fixture.calls.length,action:fixture.calls[0].action,pending:document.querySelector('#core-panel-content').getAttribute('aria-busy'),unchanged:fixture.latest.build.equipment.length})`),
      {calls:1,action:{kind:"equip",itemId:"rear_view_lens",expectedItemId:null},pending:"true",unchanged:0});
    await evaluate(`document.querySelector('[data-inventory-focus^="equip:"]').click()`);
    assert.equal(await evaluate("fixture.calls.length"),1);
    await evaluate(`fixture.resolvers.shift()(fixture.receipt())`);
    await waitFor("document.querySelector('#core-panel-content').getAttribute('aria-busy') === 'false'", "equipment receipt");
    assert.deepEqual(await evaluate(`({busy:document.querySelector('#core-panel-content').getAttribute('aria-busy'),remove:document.querySelector('[data-inventory-focus^="remove:"]').disabled,feedback:document.querySelector('.inventory-feedback').textContent,focused:document.activeElement.dataset.inventoryFocus})`),
      {busy:"false",remove:false,feedback:"装备已更新。",focused:"item:rear_view_lens"});
    // Shift-Tab from the close button wraps to the last enabled control inside this dialog.
    await evaluate("document.querySelector('#core-panel-close').focus()");
    await cdp.send("Input.dispatchKeyEvent",{type:"keyDown",key:"Tab",code:"Tab",windowsVirtualKeyCode:9,modifiers:8});
    await cdp.send("Input.dispatchKeyEvent",{type:"keyUp",key:"Tab",code:"Tab",windowsVirtualKeyCode:9,modifiers:8});
    assert.equal(await evaluate("document.activeElement.dataset.inventoryFocus"),"remove:lens");
    await evaluate(`document.querySelector('[data-inventory-focus="remove:lens"]').click();fixture.presenter.close()`);
    assert.equal(await evaluate("document.activeElement.id"),"trigger");
    await evaluate(`fixture.resolvers.shift()(fixture.receipt(fixture.inventorySnapshot({authorityRevision:12,build:{...fixture.inventorySnapshot().build,revision:4}})))`);
    assert.equal(await evaluate("document.querySelector('#core-panel').hidden"),true);
    await evaluate(`fixture.presenter.open(document.querySelector('#trigger'),'inventory');document.querySelector('[data-inventory-focus^="equip:"]').click();fixture.apply(fixture.inventorySnapshot({worldEpoch:5,build:{schemaVersion:1,revision:0,items:[],equipment:[]}}));fixture.resolvers.shift()(fixture.receipt())`);
    assert.deepEqual(await evaluate(`({epoch:fixture.latest.worldEpoch,items:document.querySelectorAll('.inventory-item').length,feedback:document.querySelector('.inventory-feedback').textContent})`),{epoch:5,items:0,feedback:""});
    await evaluate(`fixture.apply(fixture.inventorySnapshot({worldEpoch:5}));document.querySelector('[data-inventory-focus^="equip:"]').click();fixture.apply(fixture.inventorySnapshot({worldEpoch:5,sceneId:'same-epoch-context'}));fixture.resolvers.shift()(fixture.receipt(fixture.equippedSnapshot(fixture.inventorySnapshot({worldEpoch:5}))))`);
    assert.deepEqual(await evaluate(`({busy:document.querySelector('#core-panel-content').getAttribute('aria-busy'),feedback:document.querySelector('.inventory-feedback').textContent,enabled:!document.querySelector('[data-inventory-focus^="equip:"]').disabled})`),{busy:"false",feedback:"",enabled:true});
    // Large real projection scrolls only inside the existing three-column panel.
    await evaluate(`const s=fixture.inventorySnapshot({worldEpoch:5});s.build.items=Array.from({length:100},(_,n)=>({...s.build.items[0],itemId:'owned_'+n}));fixture.apply(s)`);
    const bounds=await evaluate(`({documentWidth:document.documentElement.scrollWidth,documentHeight:document.documentElement.scrollHeight,width:innerWidth,height:innerHeight,scroll:document.querySelector('#core-panel-content').scrollHeight,client:document.querySelector('#core-panel-content').clientHeight,columns:getComputedStyle(document.querySelector('#core-panel-content')).gridTemplateColumns.split(' ').length})`);
    assert.ok(bounds.documentWidth<=bounds.width && bounds.documentHeight<=bounds.height,JSON.stringify(bounds));assert.ok(bounds.scroll>bounds.client);assert.equal(bounds.columns,3);
    await cdp.send("Input.dispatchKeyEvent",{type:"keyDown",key:"Escape",code:"Escape",windowsVirtualKeyCode:27});
    await cdp.send("Input.dispatchKeyEvent",{type:"keyUp",key:"Escape",code:"Escape",windowsVirtualKeyCode:27});
    await waitFor("document.querySelector('#core-panel').hidden", "Escape dismissal");
    assert.equal(await evaluate("document.querySelector('#core-panel').hidden"),true);
  }, () => session.cleanup());
});
