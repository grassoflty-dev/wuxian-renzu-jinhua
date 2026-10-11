// quick_nexus_shot.mjs — 启动游戏抓第一视角截图(定位"满宽左右拖条"bug)
// 用法: node quick_nexus_shot.mjs
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));

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

async function startGame() {
  const child = spawn(EXE, [], {
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: 'ignore',
  });
  child.unref();
  await sleep(3000);
  return child;
}

let ws, nextId = 1;
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
  if (r.exceptionDetails) throw new Error('eval exc: ' + JSON.stringify(r.exceptionDetails).slice(0, 300));
  return r.result?.value;
}
async function shot(tag) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const out = path.join(__dirname, 'shots', `nexus_first_${tag}.png`);
  fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
  console.log('SHOT ' + out + ' ' + fs.statSync(out).size + 'B');
  return out;
}

(async () => {
  await killPort();
  const child = await startGame();
  const page = await getPage();
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(2500);
  // 第一视角: 当前画面(可能标题/主神)
  const info = await evalJs(`JSON.stringify({
    body: document.body ? document.body.scrollWidth + 'x' + document.body.scrollHeight : null,
    title: document.title,
    hasNarr: !!document.getElementById('narrBox'),
    narr: (document.getElementById('narrBox')||{}).innerText?.slice(0,200),
    canvases: [...document.querySelectorAll('canvas')].map(c => c.width + 'x' + c.height + '@' + (c.style.width||''))
  })`);
  console.log('INFO ' + info);
  await shot('t0');
  // 点「进入轮回」
  try {
    const c = await evalJs(`(() => {
      const b = document.getElementById('btnNew') || [...document.querySelectorAll('button')].find(x=>/进入|开始/i.test((x.textContent||'').replace(/\\s/g,'')));
      if (b) { b.click(); return 'clicked:' + (b.id||b.textContent); }
      return 'nobtn';
    })()`);
    console.log('CLICK ' + c);
  } catch (e) { console.log('CLICK fail ' + e.message); }
  await sleep(4000);
  await shot('t1_afterEnter');
  // 世界视图信息
  const info2 = await evalJs(`JSON.stringify({
    mode: (window.__mode||''),
    titleVisible: (document.getElementById('titleScreen')||{}).style?.display,
    worldLoc: (document.getElementById('worldLoc')||{}).textContent,
    narr: (document.getElementById('narrBox')||{}).innerText?.slice(0,150),
    canvases: [...document.querySelectorAll('canvas')].map(c => c.width + 'x' + c.height + '@' + (c.style.width||'')),
    bodyScroll: document.body ? document.body.scrollWidth + 'x' + document.body.scrollHeight : null
  })`);
  console.log('INFO2 ' + info2);
  await shot('t2_world');
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  console.log('DONE');
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); try { child.kill(); } catch {} process.exit(2); });
