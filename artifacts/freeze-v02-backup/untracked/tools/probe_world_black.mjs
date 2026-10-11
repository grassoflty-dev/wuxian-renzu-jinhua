// probe_world_black.mjs — 进世界+跳过新手引导后, 诊断世界视图黑屏
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, nextId = 1;

async function killPort() {
  try {
    const out = execSync(`netstat -ano | findstr :${PORT}`, { stdio: 'pipe' }).toString();
    for (const line of out.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  try { execSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul', { stdio: 'pipe' }); } catch {}
  await sleep(800);
}
async function getPage() {
  for (let i = 0; i < 150; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}/json/list`);
      const list = await r.json();
      const p = list.find(x => x.type === 'page');
      if (p) return p;
    } catch {}
    await sleep(500);
  }
  throw new Error('page not found');
}
async function connect(page) {
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
}
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 20000);
    const onMsg = ev => {
      let m; try { m = JSON.parse(ev.data); } catch { return; }
      if (m.id !== id) return;
      clearTimeout(timer); ws.removeEventListener('message', onMsg);
      m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result);
    };
    ws.addEventListener('message', onMsg);
    send({ id, method, params });
  });
}
async function evalJs(expression) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 300);
  return r.result?.value;
}
(async () => {
  await killPort();
  const child = spawn(EXE, [], {
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: 'ignore',
  });
  child.unref();
  await sleep(4000);
  const page = await getPage();
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(2000);
  // 捕获 JS 错误
  await evalJs(`window.__errs=[]; window.addEventListener('error', e=>window.__errs.push(String(e.message).slice(0,200))); true`);
  // 进世界
  await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 'ok';} return 'nobtn'; })()`);
  await sleep(2500);
  // 新手引导: 点跳过
  const tut = await evalJs(`(() => { const b=document.getElementById('btnTutSkip')||document.getElementById('btnTutStart'); if(b){b.click(); return 'clicked:'+b.id;} return 'notut'; })()`);
  console.log('TUT ' + tut);
  await sleep(3500);
  // 世界视图状态
  console.log('STATE ' + await evalJs(`JSON.stringify({
    worldVisible: (document.getElementById('worldView')||{}).style?.display,
    loc: (document.getElementById('worldLoc')||{}).textContent,
    msg: (document.getElementById('worldMsg')||{}).textContent,
    hud: (document.getElementById('hud')||{}).style?.display,
    canvases: [...document.querySelectorAll('canvas')].map(c=>({id:c.id||'', w:c.width, h:c.height, sw:c.style.width, sh:c.style.height})),
    errs: window.__errs || []
  })`));
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const out = path.join(__dirname, 'shots', 'world_black_diag.png');
  fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
  console.log('SHOT ' + out + ' ' + fs.statSync(out).size + 'B');
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
