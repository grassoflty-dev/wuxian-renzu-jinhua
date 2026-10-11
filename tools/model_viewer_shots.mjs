// 体素建模独立展示三视图截图：启动 EXE → 调 window.ModelViewer 渲染干净场景（纯白背景+标准白光）→ CDP 截图
// 输出: tools/shots/mv_front.png / mv_side.png / mv_back.png / mv_face.png（正面脸部特写）
// 说明：不进入游戏世界，脱离游戏场景/光照/后期，单独展示建模，用于与 2D 立绘对比质检。
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const OUT = path.join(__dirname, 'shots');
const PORT = 9702;
fs.mkdirSync(OUT, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const LOG = path.join(__dirname, 'logs', 'model_viewer_shots.log');
fs.mkdirSync(path.dirname(LOG), { recursive: true });
const log = s => { try { fs.appendFileSync(LOG, s + '\n'); } catch {} console.log(s); };

function cleanLaunch() {
  try { execSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul', { stdio: 'pipe' }); } catch {}
  try {
    const out = execSync(`netstat -ano | findstr :${PORT}`, { stdio: 'pipe' }).toString();
    for (const line of out.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  const child = spawn(EXE, [], {
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: 'ignore',
  });
  child.unref();
  return child;
}
async function getPage() {
  for (let i = 0; i < 90; i++) {
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
let ws, nextId = 1;
async function connect(page) {
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
}
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 30000);
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
async function evalJs(expression, timeout = 30000) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) throw new Error('eval exc: ' + JSON.stringify(r.exceptionDetails).slice(0, 500));
  return r.result?.value;
}

async function setViewport(w, h) {
  await cdp('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await evalJs('window.dispatchEvent(new Event("resize")); true');
}

async function main() {
  const child = cleanLaunch();
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); process.exit(2); }
  await connect(page);
  await sleep(2500);
  log('readyState=' + await evalJs('document.readyState'));
  log('hasVoxelPeople=' + await evalJs('!!window.VoxelPeople'));
  log('hasModelViewer=' + await evalJs('!!window.ModelViewer'));

  await setViewport(1600, 900);
  await sleep(800);

  // 展示建模（郑吒，缺省）
  log('show=' + await evalJs(`window.ModelViewer.show()`));
  await sleep(800);

  const views = [
    ['front', 'mv_front.png'],
    ['side', 'mv_side.png'],
    ['back', 'mv_back.png'],
    ['face', 'mv_face.png'],
  ];
  for (const [v, name] of views) {
    log('setView ' + v + ' => ' + await evalJs(`window.ModelViewer.setView('${v}')`));
    await sleep(700);
    const r = await cdp('Page.captureScreenshot', { format: 'png' });
    const p = path.join(OUT, name);
    fs.writeFileSync(p, Buffer.from(r.data, 'base64'));
    log('SHOT ' + name + ' ' + fs.statSync(p).size + 'B');
  }

  ws.close();
  child.kill();
  log('DONE model_viewer 4 views');
  process.exit(0);
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); } catch {} console.error('FATAL', e); process.exit(2); });
