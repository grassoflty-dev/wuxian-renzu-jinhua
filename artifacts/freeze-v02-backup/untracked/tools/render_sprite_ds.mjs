// render_sprite_ds.mjs — SpriteStacking 立绘投影多视图渲染（DS 独立通道，Edge headless）
// 与 render_one.mjs / render_ds.mjs 完全独立：独立 HTML(sprite_ds.html)、独立端口(9708)、独立输出前缀 render_sprite_ds_*
// 用法: node render_sprite_ds.mjs <name> [view1,view2,...]
//   name 缺省 zhenghai（郑吒）；views 缺省 front,side,back,face（正/侧/背/脸特写）
// 输出: tools/shots/render_sprite_ds_<name>_<view>.png
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const UI = path.resolve(__dirname, '..', 'server-rs', 'ui');
const HTML = path.join(UI, 'sprite_ds.html');
const SHOTS = path.join(__dirname, 'shots');
const PORT = 9708;
fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

const name = process.argv[2] || 'zhenghai';
const views = (process.argv[3] || 'front,side,back,face').split(',');
// 补：'profile' 为真侧视图(验证侧面是否连续轮廓)；用 `node render_sprite_ds.mjs zhenghai front,side,back,face,profile` 渲染

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
  const userDir = path.join(os.tmpdir(), 'render_sprite_ds_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });

  const child = cleanLaunch(edge, userDir, PORT);
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(1000);
  log('hasSprite=' + await evalJs('!!window.SpriteBuilderDS') + ' hasSetSprite=' + await evalJs('typeof window.__setSprite'));

  await setViewport(1600, 900);
  await sleep(600);

  log('__setSprite(' + name + ") => " + await evalJs(`window.__setSprite(${JSON.stringify(name)})`));
  // 等立绘贴图异步加载 + 层构建完成再截图
  for (let i = 0; i < 30; i++) { await sleep(200); if (await evalJs('window.__spriteReady === true')) break; }
  await sleep(400);
  log('spriteReady=' + await evalJs('window.__spriteReady') + ' layers=' + await evalJs('window.__spriteLayers'));

  for (const v of views) {
    log('setView ' + v + ' => ' + await evalJs(`window.setSpriteView('${v}')`));
    await sleep(500);
    const r = await cdp('Page.captureScreenshot', { format: 'png' });
    const out = path.join(SHOTS, `render_sprite_ds_${name}_${v}.png`);
    fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
    log('SHOT ' + path.basename(out) + ' ' + fs.statSync(out).size + 'B');
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  log('DONE render_sprite_ds ' + name);
  process.exit(0);
}
main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
