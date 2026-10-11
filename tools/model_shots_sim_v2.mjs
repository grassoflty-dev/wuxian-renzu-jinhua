// ⚠️【废弃标注】游戏已切换到 World2D/Zone2D 引擎（index.html 不再加载 js/postfx.js），window.PostFX 已不存在，本脚本的相机覆写/后处理链路失效，输出已无意义。画面评估请用 probe_world2d.mjs / probe_zone2d.mjs。
// 立绘与建模 相似度复评（v2）三视图截图：干净启动 → 新局进生化世界（biohazard_ch1）→ 玩家郑吒三视图特写
// 输出: tools/shots/sim_v2_front.png（正面）、sim_v2_side.png（侧面）、sim_v2_back.png（背面）
// 要求：每张 >50KB 且非黑屏；尽量正/侧/背三视角。
// 复用 model_shots_sim_base.mjs 逻辑（cleanLaunch/cdp/相机覆盖）；biohazard_ch1 为新局缺省世界。
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
const LOG = path.join(__dirname, 'logs', 'model_shots_sim_v2.log');
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
const readSave = () => { try { return JSON.parse(fs.readFileSync(SAVE, 'utf8')); } catch { return null; } };

async function setViewport(w, h) {
  await cdp('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await evalJs('window.dispatchEvent(new Event("resize")); true');
}

async function maximizeCanvas(w, h) {
  return await evalJs(`(function(){
    const pf = window.PostFX;
    if(!pf || !pf._renderer) return null;
    try {
      pf._renderer.setPixelRatio(1);
      pf._renderer.setSize(${w}, ${h}, true);
      if (pf._mainCam) { pf._mainCam.aspect = ${w} / ${h}; pf._mainCam.updateProjectionMatrix(); }
      const cv = pf._renderer.domElement;
      cv.style.margin = '0';
      window.dispatchEvent(new Event('resize'));
      const r = cv.getBoundingClientRect();
      return { x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height),
               cw: cv.width, ch: cv.height };
    } catch(e){ return { err: String(e.message||e) }; }
  })()`);
}

const INSTALL_CAM = `(function(){
  if (window.__camInstalled) return 'already';
  window.__camInstalled = true;
  window.__camOverride = null;
  const orig = window.PostFX.render.bind(window.PostFX);
  window.PostFX.render = function(){
    const o = window.__camOverride;
    if (o && this.ready && this._mainCam){
      this._mainCam.position.set(o.px, o.py, o.pz);
      this._mainCam.up.set(0,1,0);
      this._mainCam.lookAt(o.lx, o.ly, o.lz);
      if (o.fov && this._mainCam.fov !== o.fov){
        this._mainCam.fov = o.fov;
        this._mainCam.aspect = this._mainCam.aspect || 1;
        this._mainCam.updateProjectionMatrix();
      }
      this._mainCam.updateMatrixWorld && this._mainCam.updateMatrixWorld(true);
    }
    return orig.call(this);
  };
  return 'ok';
})()`;
const setCam = (px, py, pz, lx, ly, lz, fov) => evalJs(`window.__camOverride = { px:${px}, py:${py}, pz:${pz}, lx:${lx}, ly:${ly}, lz:${lz}, fov:${fov || 0} }; 'set'`);

async function shot(name, clip) {
  await sleep(1100);
  const params = { format: 'png' };
  if (clip) {
    params.clip = { x: clip.x, y: clip.y, width: clip.w, height: clip.h, scale: 1 };
    params.fromSurface = true;
  }
  const r = await cdp('Page.captureScreenshot', params);
  const p = path.join(OUT, name);
  fs.writeFileSync(p, Buffer.from(r.data, 'base64'));
  const size = fs.statSync(p).size;
  log(`SHOT ${name} ${size}B ${size > 50 * 1024 ? '(OK >50KB)' : '(<50KB!)'} clip=${clip ? clip.w + 'x' + clip.h : 'full'}`);
  return { name, path: p, size };
}

async function playerCam() {
  const c = await evalJs(`(function(){
    const pf = window.PostFX;
    if(!pf || !pf._mainCam) return null;
    const cam = pf._mainCam;
    const p = { x: cam.position.x, y: cam.position.y, z: cam.position.z };
    let pl = null;
    pf._scene && pf._scene.traverse(o => { if(o.userData && o.userData.rig && o.userData.rig.head && !pl) pl = o; });
    let headY = null;
    if (pl && pl.userData.rig.head){
      try {
        const v = new THREE.Vector3();
        pl.updateMatrixWorld(true);
        pl.userData.rig.head.getWorldPosition(v);
        headY = v.y;
      } catch(e) { headY = pl.position.y + 2.0; }
    }
    return { cam: p, player: pl ? { x: pl.position.x, y: pl.position.y, z: pl.position.z } : null, headY: headY, ready: pf.ready };
  })()`);
  return c;
}

async function diag() {
  return await evalJs(`(function(){
    return { pfReady: window.PostFX ? !!window.PostFX.ready : false,
             camOverride: window.__camOverride ? 'set' : 'null' };
  })()`);
}

async function blackCheck() {
  return await evalJs(`(function(){
    const c = (window.PostFX && window.PostFX._renderer && window.PostFX._renderer.domElement) || null;
    return c ? { w: c.width, h: c.height, srcW: c.style.width, srcH: c.style.height } : null;
  })()`);
}

async function main() {
  const child = cleanLaunch();
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); process.exit(2); }
  await connect(page);
  await sleep(2000);
  log('readyState=' + await evalJs('document.readyState'));
  log('hasPostFX=' + await evalJs('!!window.PostFX'));

  try { await setViewport(1600, 900); log('setViewport ok'); } catch (e) { log('setViewport warn ' + e.message); }
  await sleep(1500);

  await evalJs(`(function(){ if(window.__showTutorial) window.__showTutorial = function(){}; return 'suppressed'; })()`);

  const clicked = await evalJs(`(function(){ const b=document.getElementById('btnNew'); if(b && b.offsetParent!==null){ b.click(); return 1; } return 0; })()`);
  log('clicked btnNew=' + clicked);
  await sleep(3500);
  await evalJs(`(function(){
    const o = document.getElementById('tutorialOverlay');
    if (o) o.classList.remove('on');
    if (window.__closeTutorial) try { window.__closeTutorial(); } catch(e){}
    return 'closed';
  })()`);
  await sleep(400);
  let st = readSave();
  log('[enter] world_id=' + (st && st.world_id) + ' scene_id=' + (st && st.scene_id) + ' px/py=' + (st && st.px) + '/' + (st && st.py));
  if (!st || st.world_id !== 'biohazard_ch1') { log('WARN 非 biohazard_ch1 世界 world_id=' + (st && st.world_id)); }
  log('diag=' + JSON.stringify(await diag()));

  log('installCam=' + await evalJs(INSTALL_CAM));
  await sleep(200);

  let clip;
  try {
    const m = await maximizeCanvas(1580, 840);
    log('maximizeCanvas=' + JSON.stringify(m));
    clip = (m && m.w) ? { x: Math.max(0, m.x - 4), y: Math.max(0, m.y - 4), w: m.w + 8, h: m.h + 8 } : null;
  } catch (e) { log('maximizeCanvas warn ' + e.message); }

  await sleep(600);
  let pc = await playerCam();
  log('playerCam=' + JSON.stringify(pc));
  if (!pc || !pc.player) { log('FATAL no player cam'); process.exit(2); }
  const P = pc.player;
  const headY = (pc.headY != null) ? pc.headY : P.y + 2.2;
  const camY = headY - 0.9;      // 相机在上胸高度，全身入镜且头部突出
  const lookY = headY - 0.1;     // 看向脸部（略低于头心）

  // FOV: 35° 望远拉近，突出体素细节
  const FOV = 35;
  const D = 5.2;   // 正/背 距离
  const DS = 5.4;  // 侧向距离

  // 正面特写：相机在 +z 正对脸部/胸口
  await setCam(P.x, camY, P.z + D, P.x, lookY, P.z, FOV);
  await shot('sim_v2_front.png', clip);

  // 侧面特写：相机绕 +x 侧移 90°
  await setCam(P.x + DS, camY, P.z, P.x, lookY, P.z, FOV);
  await shot('sim_v2_side.png', clip);

  // 背面特写：相机在 -z 正对后脑勺/背
  await setCam(P.x, camY, P.z - D, P.x, lookY, P.z, FOV);
  await shot('sim_v2_back.png', clip);

  log('canvas=' + JSON.stringify(await blackCheck()));

  ws.close();
  child.kill();
  log('DONE sim_v2 三视图 3 shots (front/side/back)');
  process.exit(0);
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); } catch {} console.error('FATAL', e); process.exit(2); });
