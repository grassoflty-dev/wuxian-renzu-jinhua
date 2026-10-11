// 体素建模批量审核截图：启动一次 EXE → 循环渲染 20 个建模（队友 3 + 敌人 17）→ 每个输出正/侧/背 3 张 PNG。
// 输出: tools/shots/audit_{name}_front.png / _side.png / _back.png（共 60 张），供 qwen_audit.mjs 批量送检。
// 队友用 buildVoxelPlayer / ModelViewer.show(name)；敌人用 buildVoxelEnemy / ModelViewer.showEnemy(kind)。
// 每个建模正/侧/背只做全身三视图（敌人/队友都不做脸部特写，保持干净横评）。
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
const LOG = path.join(__dirname, 'logs', 'model_audit_shots.log');
fs.mkdirSync(path.dirname(LOG), { recursive: true });
const log = s => { try { fs.appendFileSync(LOG, s + '\n'); } catch {} console.log(s); };

const MIN_PNG = 50 * 1024;   // 非空/非黑屏阈值：>50KB

// 名单：队友（buildVoxelPlayer / ModelViewer.show），敌人（buildVoxelEnemy / ModelViewer.showEnemy）
const TEAM = ['chuxuan', 'zhanlan', 'zhaoyingkong'];
const ENEMY = ['sanjiaotou', 'yiy_queen', 'brain_bug', 'fulaidi', 'tyrant', 'barbossa',
  'dragon', 'demon', 'werewolf', 'golem', 'tentacle', 'undead',
  'guard', 'hunter', 'licker', 'zombie', 'horde'];

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

async function snapshot(outFile) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const p = path.join(OUT, outFile);
  fs.writeFileSync(p, Buffer.from(r.data, 'base64'));
  const size = fs.statSync(p).size;
  log('SHOT ' + outFile + ' ' + size + 'B ' + (size > MIN_PNG ? 'OK' : '⚠️ <50KB 疑似空/黑屏'));
  return size;
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
  log('hasShowEnemy=' + await evalJs('!!window.ModelViewer && typeof window.ModelViewer.showEnemy === "function"'));

  await setViewport(1600, 900);
  await sleep(800);

  // 统一：每个建模正/侧/背 3 张
  const views = [
    ['front', 'front'],
    ['side', 'side'],
    ['back', 'back'],
  ];
  let ok = 0, warn = 0;

  const audit = (name, isEnemy) => {
    const showCall = isEnemy
      ? `window.ModelViewer.showEnemy(${JSON.stringify(name)})`
      : `window.ModelViewer.show(${JSON.stringify(name)})`;
    return { name, showCall };
  };

  for (const t of TEAM) await renderOne(audit(t, false), views, s => {
    if (s > MIN_PNG) ok++; else warn++;
  });
  for (const e of ENEMY) await renderOne(audit(e, true), views, s => {
    if (s > MIN_PNG) ok++; else warn++;
  });

  async function renderOne(a, viewsArr, onSize) {
    log('=== 渲染建模: ' + a.name + (a.showCall.includes('showEnemy') ? ' (enemy)' : ' (teammate)') + ' ===');
    const res = await evalJs(a.showCall);
    log('show => ' + res);
    await sleep(500);           // 构建后稳定
    for (const [v, suffix] of viewsArr) {
      log('setView ' + v + ' => ' + await evalJs(`window.ModelViewer.setView('${v}')`));
      await sleep(500);          // 每张截图后稳定渲染
      const size = await snapshot(`audit_${a.name}_${suffix}.png`);
      onSize(size);
    }
    // 释放当前建模，避免跨建模时旧 body 残留（show/showEnemy 内部会 disposeGroup）
  }

  ws.close();
  child.kill();
  log(`DONE model_audit_shots ${TEAM.length + ENEMY.length} models x ${views.length} views; OK>50KB=${ok}, warn<=50KB=${warn}`);
  process.exit(0);
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); } catch {} console.error('FATAL', e); process.exit(2); });
