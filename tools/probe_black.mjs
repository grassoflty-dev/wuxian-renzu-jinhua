// probe_black.mjs — 诊断游戏黑屏: 查渲染器尺寸/画面状态
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, nextId = 1;
async function getPage() {
  for (let i = 0; i < 30; i++) {
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
  const page = await getPage();
  await connect(page);
  const info = await evalJs(`JSON.stringify({
    title: document.title,
    titleVisible: (document.getElementById('titleScreen')||{}).style?.display,
    worldVisible: (document.getElementById('worldView')||{}).style?.display,
    canvases: [...document.querySelectorAll('canvas')].map((c,i)=>({i, id:c.id||'', w:c.width, h:c.height, sw:c.style.width, sh:c.style.height, rect:(()=>{const r=c.getBoundingClientRect();return Math.round(r.width)+'x'+Math.round(r.height)})()})),
    wrapRect: (()=>{const w=document.getElementById('worldCanvasWrap'); if(!w) return null; const r=w.getBoundingClientRect(); return Math.round(r.width)+'x'+Math.round(r.height);})(),
    bodyBg: getComputedStyle(document.body).backgroundColor
  })`);
  console.log('INFO ' + info);
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const out = path.join(__dirname, 'shots', 'black_diag.png');
  fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
  console.log('SHOT ' + out + ' ' + fs.statSync(out).size + 'B');
  try { ws.close(); } catch {}
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
