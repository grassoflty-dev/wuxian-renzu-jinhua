// ⚠️【废弃标注】游戏已切换到 World2D 引擎（无右键拖拽旋转），本脚本 rotateCam 已失效，三张连拍将完全相同；地图画面评估请用 probe_world2d.mjs。
// 地图 3D 截屏质检：干净启动 → 新局进入世界地图 3D → 旋转视角连拍 3 张
// 输出: tools/shots/map_qc_1.png ~ map_qc_3.png（要求 >20KB，非黑屏）
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const SAVE = path.join(ROOT, 'target', 'release', 'data', 'save.json');
const OUT = path.join(__dirname, 'shots');
const PORT = 9702;
fs.mkdirSync(OUT, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const LOG = path.join(__dirname, 'logs', 'shots_map_qc.log');
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
  fs.rmSync(SAVE, { force: true });
  const child = spawn(EXE, [], {
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: 'ignore',
  });
  child.unref();
  return child;
}
async function getPage() {
  for (let i = 0; i < 60; i++) {
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
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 25000);
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
async function evalJs(expression, timeout = 25000) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) throw new Error('eval exc: ' + JSON.stringify(r.exceptionDetails).slice(0, 400));
  return r.result?.value;
}
const inv = (cmd, args) => evalJs(`(async function(){ try { return await window.__TAURI__.core.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args || {})}); } catch(e){ return {__err:String(e.message||e)}; } })()`);
const readSave = () => { try { return JSON.parse(fs.readFileSync(SAVE, 'utf8')); } catch { return null; } };

async function shot(name) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const p = path.join(OUT, name);
  fs.writeFileSync(p, Buffer.from(r.data, 'base64'));
  const size = fs.statSync(p).size;
  log(`SHOT ${name} ${size}B ${size > 20 * 1024 ? '(OK)' : '(偏小,可能黑屏/未渲染)'}`);
  return size;
}

// 通过合成鼠标右键拖动事件旋转视角（World2D 用右键拖动旋转 cameraYaw）
// pixels>0 向右拖→视角顺时针偏转（改变画面角度）
async function rotateCam(pixels) {
  const done = await evalJs(`(async function(){
    try {
      let cv = document.querySelector('#worldCanvasWrap canvas');
      if (!cv) cv = document.getElementById('worldCanvas');
      if (!cv) return 'no-canvas';
      const cx = cv.getBoundingClientRect().width / 2 || 400;
      const cy = cv.getBoundingClientRect().height / 2 || 300;
      const opts = (x, y) => ({ bubbles: true, cancelable: true, button: 2, buttons: 2, clientX: x, clientY: y, screenX: x, screenY: y, buttonInit: true });
      cv.dispatchEvent(new MouseEvent('mousedown', Object.assign({ button: 2, buttons: 2, clientX: cx, clientY: cy }, opts(cx, cy))));
      const steps = 6;
      const draw = ${JSON.stringify(pixels)};
      for (let i = 1; i <= steps; i++) {
        const x = cx + Math.round(draw * i / steps);
        cv.dispatchEvent(new MouseEvent('mousemove', Object.assign({ button: 2, buttons: 2, clientX: x, clientY: cy }, opts(x, cy))));
      }
      cv.dispatchEvent(new MouseEvent('mouseup', Object.assign({ button: 2, buttons: 0, clientX: cx + draw, clientY: cy }, opts(cx + draw, cy))));
      return 'rotated ' + draw;
    } catch(e){ return 'ERR:' + String(e.message||e); }
  })()`);
  return done;
}

async function diag() {
  return await evalJs(`(function(){
    const cv = document.querySelector('#worldCanvasWrap canvas') || document.getElementById('worldCanvas') || null;
    return { canvas: cv ? { w: cv.width, h: cv.height, cls: cv.className } : null,
             worldView: (document.getElementById('worldView')||{}).style.display||'none' };
  })()`);
}

async function main() {
  const child = cleanLaunch();
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); process.exit(2); }
  await connect(page);
  await sleep(3500);
  log('UI ready; readyState=' + await evalJs('document.readyState'));
  log('diag=' + JSON.stringify(await diag()));

  // 1. 新局进世界地图（btnNew → enterWorld → api_new → World2D.setData 显示 3D）
  const clicked = await evalJs(`(function(){ const b=document.getElementById('btnNew'); if(b && b.offsetParent!==null){ b.click(); return 1; } return 0; })()`);
  log('clicked btnNew=' + clicked);
  await sleep(3000);
  let st = readSave();
  const inWorld = st && (st.world_id || st.scene_id);
  log('[enter] save world_id=' + (st && st.world_id) + ' scene_id=' + (st && st.scene_id) + ' px/py=' + (st && st.px) + '/' + (st && st.py));
  log('diag2=' + JSON.stringify(await diag()));

  // 2. 首张（默认视角）
  await sleep(1500);
  await shot('map_qc_1.png');

  // 3. 旋转视角后各拍一张
  await rotateCam(500);
  await sleep(2200);
  await shot('map_qc_2.png');

  await rotateCam(300);
  await sleep(2200);
  await shot('map_qc_3.png');

  st = readSave();
  log('[RESULT] world_id=' + (st && st.world_id) + ' scene_id=' + (st && st.scene_id));
  log('DONE 进地图=' + (inWorld ? 'yes' : 'no'));

  ws.close();
  child.kill();
  process.exit(0);
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); } catch {} console.error('FATAL', e); process.exit(2); });
