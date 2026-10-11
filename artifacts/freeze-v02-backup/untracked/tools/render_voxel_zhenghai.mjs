// render_voxel_zhenghai.mjs — 郑吒体素模型 11 视图渲染（Edge headless CDP，纯白背景）
// 用法: node render_voxel_zhenghai.mjs [view1,view2,...]
//   views 缺省 front,side,profile,back,face,top,bottom,upper,head45,head135,foot
// 输出: tools/shots/render_voxel_zhenghai_<view>.png (2400×1350)
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HTML = path.join(__dirname, '..', 'server-rs', 'ui', 'voxel_zhenghai.html');
const SHOTS = path.join(__dirname, 'shots');
const PORT = 9721;
fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

const views = (process.argv[2] || 'front,side,profile,back,face,top,bottom,upper,head45,head135,foot').split(',');

function findEdge() {
  const p32 = process.env['PROGRAMFILES(X86)'] || 'C:\\Program Files (x86)';
  const p64 = process.env.PROGRAMFILES || 'C:\\Program Files';
  const cands = [
    path.join(p64, 'Microsoft', 'Edge', 'Application', 'msedge.exe'),
    path.join(p32, 'Microsoft', 'Edge', 'Application', 'msedge.exe'),
  ];
  for (const c of cands) if (fs.existsSync(c)) return c;
  return null;
}

function cleanLaunch(edge, userDir, port) {
  try {
    const o = execSync(`netstat -ano | findstr :${port}`, { stdio: 'pipe' }).toString();
    for (const line of o.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  const child = spawn(edge, [
    '--headless=new',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${userDir}`,
    '--no-first-run',
    '--disable-gpu',
    '--allow-file-access-from-files',
    `file:///${HTML.replace(/\\/g, '/').replace(/^\/+/, '')}`,
  ], { stdio: 'ignore' });
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
  const edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe'); process.exit(2); }
  const userDir = path.join(os.tmpdir(), 'render_voxel_zhenghai_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });

  const child = cleanLaunch(edge, userDir, PORT);
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(800);
  log('hasData=' + await evalJs('!!window.VoxelZhenghaiData') + ' hasBuilder=' + await evalJs('!!window.VoxelZhenghai'));

  await setViewport(2400, 1350);
  await sleep(600);

  log('build => ' + await evalJs('window.__buildModel()'));
  await sleep(500);
  log('modelReady=' + await evalJs('window.__modelReady') + ' stats=' + await evalJs('JSON.stringify(window.__modelStats||null)'));

  for (const v of views) {
    log('setView ' + v + ' => ' + await evalJs(`window.setModelView('${v}')`));
    await sleep(450);
    const r = await cdp('Page.captureScreenshot', { format: 'png' });
    const out = path.join(SHOTS, `render_voxel_zhenghai_${v}.png`);
    fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
    log('SHOT ' + path.basename(out) + ' ' + fs.statSync(out).size + 'B');
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  log('DONE render_voxel_zhenghai');
  process.exit(0);
}
main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
