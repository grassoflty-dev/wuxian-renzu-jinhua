// render_item.mjs — 道具 3D 模型批量渲染(Edge headless CDP, 纯白背景)
// 用法: node render_item.mjs [names|all] [views]
//   names 缺省 all(扫描 assets/img/item_*.png); 逗号分隔可指定子集
//   views 缺省 front,orbit45,side(正面 / 45° / 侧视); 也可 back|top|orbit<az>_<el>
// 输出: shots/render_item_<name>_<view>.png
// 页面: server-rs/ui/item_viewer.html?render=1
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HTML = path.join(__dirname, '..', 'server-rs', 'ui', 'item_viewer.html');
const ASSETS = path.join(__dirname, '..', 'server-rs', 'ui', 'assets', 'img');
const SHOTS = path.join(__dirname, 'shots');
const PORT = 9721;
fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

const nameArg = process.argv[2] || 'all';
const viewsArg = (process.argv[3] || 'front,orbit45,side').split(',');

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
    `file:///${HTML.replace(/\\/g, '/').replace(/^\/+/, '')}?render=1`,
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
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 60000);
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
async function evalJs(expression, timeout = 60000) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) throw new Error('eval exc: ' + JSON.stringify(r.exceptionDetails).slice(0, 500));
  return r.result?.value;
}
async function setViewport(w, h) {
  await cdp('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await evalJs('window.dispatchEvent(new Event("resize")); true');
}
async function captureShot(outPath) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(outPath, Buffer.from(r.data, 'base64'));
  return fs.statSync(outPath).size;
}

async function applyView(v) {
  if (v.startsWith('orbit')) {
    const rest = v.slice(5);
    let az = parseInt(rest, 10), el = 20;
    if (rest.indexOf('_') >= 0) { const p = rest.split('_'); az = parseInt(p[0], 10); el = parseInt(p[1], 10); }
    if (isNaN(az)) return 'ERR:bad-orbit:' + v;
    return await evalJs(`window.setItemOrbit(${az},${el})`);
  }
  return await evalJs(`window.setItemView(${JSON.stringify(v)})`);
}

async function main() {
  const edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe'); process.exit(2); }

  let all = [];
  try { all = fs.readdirSync(ASSETS).filter(f => /^item_.*\.png$/.test(f)).map(f => f.slice(0, -4)).sort(); }
  catch (e) { console.error('扫描 assets/img 失败: ' + e.message); process.exit(2); }
  const names = nameArg === 'all' ? all : nameArg.split(',');
  const unknown = names.filter(n => !all.includes(n));
  if (unknown.length) { console.error('未知道具(忽略): ' + unknown.join(', ')); }
  const targets = names.filter(n => all.includes(n));
  if (!targets.length) { console.error('没有可渲染的道具'); process.exit(2); }
  log(`ITEMS=${targets.length} VIEWS=${viewsArg.join(',')} -> shots/`);

  const userDir = path.join(os.tmpdir(), 'render_item_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });
  const child = cleanLaunch(edge, userDir, PORT);

  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(1200);
  const hasBuilder = await evalJs('!!window.ItemBuilder');
  const hasSet = await evalJs('typeof window.__setItem === "function"');
  log('ItemBuilder=' + hasBuilder + ' __setItem=' + hasSet);
  if (!hasBuilder || !hasSet) { console.error('页面未就绪(缺 ItemBuilder 或 __setItem)'); try { ws.close(); } catch {} child.kill(); process.exit(2); }

  await setViewport(1024, 1024);
  await sleep(500);

  let ok = 0, fail = 0;
  const failList = [];
  for (let k = 0; k < targets.length; k++) {
    const name = targets[k];
    const r = await evalJs(`window.__setItem(${JSON.stringify(name)})`);
    if (typeof r !== 'string' || r.indexOf('ok:') !== 0) {
      fail++; failList.push(name + ':' + r);
      log(`FAIL ${name} -> ${r}`);
      continue;
    }
    for (const v of viewsArg) {
      const vr = await applyView(v);
      await sleep(450);
      const out = path.join(SHOTS, `render_item_${name}_${v}.png`);
      const sz = await captureShot(out);
      ok++;
      log(`SHOT ${path.basename(out)} ${sz}B  [${r}]`);
    }
    if ((k + 1) % 10 === 0) log(`--- progress ${k + 1}/${targets.length} ---`);
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  log(`DONE render_item items=${targets.length} shots=${ok} fail=${fail}`);
  if (failList.length) log('FAIL_LIST: ' + failList.join(' | '));
  process.exit(fail ? 1 : 0);
}
main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
