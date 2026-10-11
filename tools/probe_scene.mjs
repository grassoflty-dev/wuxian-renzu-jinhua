// ⚠️【废弃标注】游戏已切换到 World2D 引擎（index.html 不再加载 world3d.js），window.__w3d 已不存在，本脚本只能输出 NO __w3d，已不可用。地图画面体检请用 probe_world2d.mjs。
// probe_scene.mjs — 诊断墙/地板不渲染: 查 __w3d.data.tiles + scene InstancedMesh
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
  await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 'ok';} return 'nobtn'; })()`);
  await sleep(2500);
  await evalJs(`(() => { const b=document.getElementById('btnTutSkip')||document.getElementById('btnTutStart'); if(b){b.click(); return 'ok';} return 'notut'; })()`);
  await sleep(3000);
  const info = await evalJs(`(() => {
    const w = window.__w3d;
    if (!w) return 'NO __w3d';
    const d = w.data || {};
    const tiles = d.tiles;
    let inst = 0, instVisible = 0, instCounts = [], meshes = 0, groups = 0;
    w.scene.traverse(o => {
      if (o.isInstancedMesh) { inst++; if (o.visible) instVisible++; instCounts.push(o.count); }
      if (o.isMesh) meshes++;
      if (o.isGroup) groups++;
    });
    return JSON.stringify({
      hasTiles: !!tiles, tilesLen: tiles ? tiles.length : 0,
      row0: tiles && tiles[0] ? String(tiles[0]).slice(0, 44) : null,
      row5: tiles && tiles[5] ? String(tiles[5]).slice(0, 44) : null,
      w: d.w, h: d.h, px: d.px, py: d.py, floor_name: d.floor_name,
      instMeshes: inst, instVisible, instCounts: instCounts.slice(0, 20),
      meshes, groups, chunks: Object.keys(w.chunks || {}).length
    });
  })()`);
  console.log('SCENE ' + info);
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
