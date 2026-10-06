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
  return [process.env.ACCESSIBILITY_TEST_BROWSER,
    ...[process.env["PROGRAMFILES(X86)"], process.env.ProgramFiles].filter(Boolean).map(p => path.join(p,"Microsoft","Edge","Application","msedge.exe")),
    // Native acceptance targets Windows Edge. Other browsers are explicit opt-in.
  ].filter(Boolean).find(p => existsSync(p));
}
const mainSource=await readFile(path.join(root,"src/main.ts"),"utf8");
const markup=mainSource.match(/root\.innerHTML = `([\s\S]*?)`;/)[1];
const html=`<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="/src/style.css"><link rel="stylesheet" href="/src/visual-authority.css"><div id="app">${markup}</div>`;
test("real browser settings panel scales preserve panel layout, focus and restart at supported viewports",{timeout:90000},async t=>{
  const binary=browserPath();if(!binary)return t.skip("Browser unavailable; ACCESSIBILITY_TEST_BROWSER enables the authorized real-browser harness");
  const server=createServer(async(req,res)=>{try{const pathname=new URL(req.url,"http://127.0.0.1").pathname;
    if(pathname==="/"){res.setHeader("Content-Type","text/html; charset=utf-8");res.end(html);return;}
    if(!/^\/(dist\/.*\.js|src\/(style|visual-authority)\.css)$/.test(pathname)||pathname.includes("..")){res.writeHead(404);res.end();return;}
    res.setHeader("Content-Type",pathname.endsWith(".css")?"text/css":"text/javascript");res.end(await readFile(path.join(root,pathname.slice(1))));
  }catch{res.writeHead(404);res.end();}});
  let cdp;
  const session = await createBrowserSession(binary, "accessibility");
  session.setServer(server);
  const evaluate=async expression=>{const result=await cdp.send("Runtime.evaluate",{expression,awaitPromise:true,returnByValue:true});if(result.exceptionDetails)throw Error(result.exceptionDetails.exception?.description||result.exceptionDetails.text);return result.result.value;};
  await withBrowserCleanup(async () => {
    await startHttpServer(server);
    await session.start();
    cdp=await session.connect();await cdp.send("Page.enable");await cdp.send("Runtime.enable");
    await cdp.send("Page.navigate",{url:`http://127.0.0.1:${server.address().port}/`});
    for(let n=0;n<100&&!(await evaluate("!!document.querySelector('#core-panel-content')"));n++)await delay(20);
    await evaluate(`(async()=>{const {CoreUiPresenter}=await import('/dist/ui/CoreUiPresenter.js');const {AccessibilityPreferenceStore}=await import('/dist/ui/AccessibilityPreferences.js');window.prefs=new AccessibilityPreferenceStore(()=>localStorage,document.documentElement);prefs.restore();window.presenter=new CoreUiPresenter(document.querySelector('#core-panel'),prefs);presenter.open(document.querySelector('#main-settings'),'settings');})()`);
    for(const [width,height] of [[1280,720],[1920,1080],[2560,1440]]){
      await cdp.send("Emulation.setDeviceMetricsOverride",{width,height,deviceScaleFactor:1,mobile:false});
      for(const text of [100,115,130])for(const ui of [100,110,125]){
        await evaluate(`(()=>{for(const [id,value] of [['textScalePercent',${text}],['uiScalePercent',${ui}]]){const el=document.querySelector('#settings-'+id);el.value=String(value);el.dispatchEvent(new Event('change'));}return new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));})()`);
        const result=await evaluate(`(()=>{const frame=document.querySelector('.core-panel-frame');const rect=frame.getBoundingClientRect();const content=document.querySelector('#core-panel-content');return {inside:rect.left>=0&&rect.top>=0&&rect.right<=innerWidth+1&&rect.bottom<=innerHeight+1,horizontal:content.scrollWidth<=content.clientWidth+1,scale:prefs.getScale()};})()`);
        assert.ok(result.inside&&result.horizontal,JSON.stringify({width,height,text,ui,result}));
        assert.deepEqual(result.scale,{textScalePercent:text,uiScalePercent:ui});
      }
    }
    await evaluate("document.querySelector('#settings-uiScalePercent').focus();presenter.apply(null)");
    assert.equal(await evaluate("document.activeElement.id"),"settings-uiScalePercent");
    await cdp.send("Input.dispatchKeyEvent",{type:"keyDown",key:"Tab",code:"Tab",windowsVirtualKeyCode:9});
    await cdp.send("Input.dispatchKeyEvent",{type:"keyUp",key:"Tab",code:"Tab",windowsVirtualKeyCode:9});
    assert.equal(await evaluate("document.activeElement.id"),"core-panel-close");
    await evaluate("presenter.close()");assert.equal(await evaluate("document.activeElement.id"),"main-settings");
    await evaluate(`(async()=>{const {AccessibilityPreferenceStore}=await import('/dist/ui/AccessibilityPreferences.js');window.restored=new AccessibilityPreferenceStore(()=>localStorage,document.documentElement);restored.restore();})()`);
    assert.deepEqual(await evaluate("restored.getScale()"),{textScalePercent:130,uiScalePercent:125});
  }, () => session.cleanup());
});
