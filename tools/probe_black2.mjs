// probe_black2.mjs — 诊断黑屏: 自动带CDP启动游戏, 查渲染器尺寸/画面
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
  if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 400);
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
  await sleep(2500);
  console.log('INFO ' + await evalJs(`JSON.stringify({
    title: document.title,
    titleVisible: (document.getElementById('titleScreen')||{}).style?.display,
    worldVisible: (document.getElementById('worldView')||{}).style?.display,
    canvases: [...document.querySelectorAll('canvas')].map((c,i)=>({i, id:c.id||'', w:c.width, h:c.height, sw:c.style.width, sh:c.style.height, rect:(()=>{const r=c.getBoundingClientRect();return Math.round(r.width)+'x'+Math.round(r.height)})()})),
    wrapRect: (()=>{const w=document.getElementById('worldCanvasWrap'); if(!w) return null; const r=w.getBoundingClientRect(); return Math.round(r.width)+'x'+Math.round(r.height);})(),
    err: window.__err || 'none'
  })`));
  // 点进入轮回看世界视图
  console.log('CLICK ' + await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 'clicked';} return 'nobtn'; })()`));
  await sleep(4500);
  console.log('INFO2 ' + await evalJs(`JSON.stringify({
    worldVisible: (document.getElementById('worldView')||{}).style?.display,
    canvases: [...document.querySelectorAll('canvas')].map((c,i)=>({i, id:c.id||'', w:c.width, h:c.height, sw:c.style.width, sh:c.style.height, rect:(()=>{const r=c.getBoundingClientRect();return Math.round(r.width)+'x'+Math.round(r.height)})()})),
    wrapRect: (()=>{const w=document.getElementById('worldCanvasWrap'); if(!w) return null; const r=w.getBoundingClientRect(); return Math.round(r.width)+'x'+Math.round(r.height);})()
  })`));
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const out = path.join(__dirname, 'shots', 'black_diag2.png');
  fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
  console.log('SHOT ' + out + ' ' + fs.statSync(out).size + 'B');
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
