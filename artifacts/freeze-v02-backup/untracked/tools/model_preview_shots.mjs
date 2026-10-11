// model_preview_shots.mjs — 独立体素建模批量截图（Edge headless 版，无需 cargo build / Tauri EXE）
// 用 Windows 自带 Edge headless 打开 server-rs/ui/model_preview.html，复用 CDP(9703) 截图。
// 循环 20 个建模（队友3 + 敌人17），每个正/侧/背 3 张 = 60 张 PNG 到 tools/shots/audit_{name}_{view}.png，
// 命名与 model_audit_shots.mjs（Tauri 版）一致，qwen_audit.mjs 直接复用。
// 相对路径 file:// 下 <script src> 依赖 --allow-file-access-from-files（已加）。若 Edge 安全限制导致脚本
// 加载失败（hasVoxelPeople=false），可退回"内联拼装"兜底（把三个脚本 readFile 内联进 HTML）。
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const UI = path.resolve(__dirname, '..', 'server-rs', 'ui');
const HTML = path.join(UI, 'model_preview.html');
const OUT = path.join(__dirname, 'shots');
const PORT = 9703;
fs.mkdirSync(OUT, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const LOG = path.join(__dirname, 'logs', 'model_preview_shots.log');
fs.mkdirSync(path.dirname(LOG), { recursive: true });
const log = s => { try { fs.appendFileSync(LOG, s + '\n'); } catch {} console.log(s); };

const MIN_PNG = 50 * 1024;   // 硬阈值（保守参考）；真正判"非空/非黑屏"用下面的像素检测

// 非空/非黑屏判定（浏览器内把渲染器 canvas 拷到 2D canvas 采样，兼容 WebGL/WebGL2 headless）：
// 白色背景纯色图 sd≈0；有体素内容则存在深色/彩色像素 → 采若干点算亮度方差与 darkPct。
// （白色背景上浅色体素 PNG 体积小，固定 50KB 会误报，故以像素方差为准。）
async function evalNonBlank() {
  const expr = `(() => {
    const cv = document.querySelector('#modelViewerHost canvas');
    if (!cv) return false;
    const w = cv.width || 1600, h = cv.height || 900;
    const tmp = document.createElement('canvas');
    tmp.width = w; tmp.height = h;
    const ctx = tmp.getContext('2d');
    ctx.drawImage(cv, 0, 0);
    const img = ctx.getImageData(0, 0, w, h).data;
    let n = 0, sum = 0, sum2 = 0, dark = 0;
    for (let i = 0; i < img.length; i += 16) {   // 每4像素采1个
      const r = img[i], g = img[i + 1], b = img[i + 2];
      const l = 0.299 * r + 0.587 * g + 0.114 * b;
      sum += l; sum2 += l * l; n++;
      if (l < 235) dark++;                        // 非白像素（体素轮廓/深色块）
    }
    const mean = sum / n;
    const sd = Math.sqrt(Math.max(0, sum2 / n - mean * mean));
    return { sd, darkPct: dark / n, w, h };
  })()`;
  return evalJs(expr);
}

// 名单（与 model_audit_shots.mjs / audit_manifest.md 一致）
const TEAM = ['chuxuan', 'zhanlan', 'zhaoyingkong'];
const ENEMY = ['sanjiaotou', 'yiy_queen', 'brain_bug', 'fulaidi', 'tyrant', 'barbossa',
  'dragon', 'demon', 'werewolf', 'golem', 'tentacle', 'undead',
  'guard', 'hunter', 'licker', 'zombie', 'horde'];

// 定位 msedge.exe：常见安装位置
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

function buildPreviewHtml() {
  // 把三个外部脚本内联进一份临时 HTML（兜底规避 file:// 对 <script src> 的安全/跨源限制）。
  const order = ['vendor/three.min.js', 'js/voxel_people.js', 'js/model_viewer.js'];
  let inlineBody = '';
  for (const rel of order) {
    const fp = path.join(UI, rel);
    if (!fs.existsSync(fp)) throw new Error('missing script ' + rel);
    inlineBody += `<script>\n${fs.readFileSync(fp, 'utf8')}\n</script>\n`;
  }
  const src = fs.readFileSync(HTML, 'utf8');
  // 移除原三个外部 <script src>（保留 __MODEL_TEAM / __setModel / 初始化段），换成内联脚本。
  let out = src.replace(/<script src="vendor\/three\.min\.js"><\/script>/g, '')
    .replace(/<script src="js\/voxel_people\.js"><\/script>/g, '')
    .replace(/<script src="js\/model_viewer\.js"><\/script>/g, '');
  // 把内联 three/voxel/model 脚本插到 </body> 前（在 __setModel 定义之后、初始化 init() 之前无影响，
  // 因 init 由 __setModel 调 ModelViewer，只要 ModelViewer 已就绪即可；故插在最后一个 script 前）。
  out = out.replace(/<script>\s*\/\/\s*页面加载后按 URL 参数/, inlineBody + '\n\n  <script>\n  // 页面加载后按 URL 参数');
  return out;
}

function cleanLaunch(edge, userDir, port) {
  // 只清理占用本端口(9703)的进程，不全局 taskkill msedge.exe（避免打断用户正常浏览）。
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

async function snapshot(outFile) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const p = path.join(OUT, outFile);
  fs.writeFileSync(p, Buffer.from(r.data, 'base64'));
  const size = fs.statSync(p).size;
  const px = await evalNonBlank();                 // 浏览器内像素判定（权威）
  const okPix = !!(px && (px.sd > 6 || px.darkPct > 0.01));   // 有实质内容
  const okSize = size > MIN_PNG;
  const verdict = okPix ? 'OK' : '⚠️ 空白/黑屏(像素)';
  log(`SHOT ${outFile} ${size}B ${verdict} sd=${px ? px.sd.toFixed(1) : 'na'} darkPct=${px ? (px.darkPct * 100).toFixed(1) : 'na'}% ${okSize ? '' : '(<50KB 因浅色体素压缩小,非故障)'}`);
  return { size, okPix };
}

async function main() {
  let edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe（请确认 Edge 已安装或手动指定路径）'); process.exit(2); }
  log('edge=' + edge);

  // 优先直接 file:// 打开原 HTML；若脚本加载失败（hasVoxelPeople=false），用内联兜底。
  const userDir = path.join(os.tmpdir(), 'mv_edge_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });
  let htmlToOpen = HTML;
  let inline = false;

  const startAndShot = async (htmlFile) => {
    // 用单独的临时 user-data-dir 打开本次 html
    const child = cleanLaunch(edge, userDir, PORT);
    let page;
    try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
    await connect(page);

    // 等待页面脚本加载完成
    for (let i = 0; i < 40; i++) {
      try {
        if (await evalJs('document.readyState==="complete"') === true) break;
      } catch {}
      await sleep(300);
    }
    await sleep(1200);
    const hasVox = await evalJs('!!window.VoxelPeople');
    const hasMV = await evalJs('!!window.ModelViewer');
    log('readyState=' + await evalJs('document.readyState') + ' hasVoxelPeople=' + hasVox + ' hasModelViewer=' + hasMV);
    return { hasVox, hasMV, child, page };
  };

  let run = await startAndShot(htmlToOpen);
  if (!(run.hasVox && run.hasMV) && !inline) {
    // 兜底：内联拼装三个脚本
    log('file:// 直连加载失败，退回内联拼装');
    const inlineHtml = buildPreviewHtml();
    const inlineFile = path.join(userDir, 'model_preview_inline.html');
    fs.writeFileSync(inlineFile, inlineHtml, 'utf8');
    try { run.child.kill(); } catch {} 
    run = await startAndShot(inlineFile);
    inline = true;
  }
  if (!(run.hasVox && run.hasMV)) { log('仍无 VoxelPeople/ModelViewer，无法截图'); process.exit(2); }

  await setViewport(1600, 900);
  await sleep(800);

  const views = [
    ['front', 'front'],
    ['side', 'side'],
    ['back', 'back'],
  ];
  let ok = 0, warn = 0;

  const renderOne = async (model, viewsArr, onRes) => {
    log('=== 渲染建模: ' + model + ' ===');
    log('__setModel => ' + await evalJs(`window.__setModel(${JSON.stringify(model)}, 'front')`));
    await sleep(600);
    for (const [v, suffix] of viewsArr) {
      log('setView ' + v + ' => ' + await evalJs(`window.ModelViewer.setView('${v}')`));
      await sleep(500);
      const res = await snapshot(`audit_${model}_${suffix}.png`);
      onRes(res);
    }
  };

  for (const m of TEAM) await renderOne(m, views, res => { res.okPix ? ok++ : warn++; });
  for (const m of ENEMY) await renderOne(m, views, res => { res.okPix ? ok++ : warn++; });

  try { run.child.kill(); } catch {}
  log(`DONE model_preview_shots ${TEAM.length + ENEMY.length} models x ${views.length} views; 像素OK=${ok}/60, 空白=${warn}` + (inline ? ' (内联模式)' : ' (file://直连)'));
  process.exit(0);
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); } catch {} console.error('FATAL', e); process.exit(2); });
