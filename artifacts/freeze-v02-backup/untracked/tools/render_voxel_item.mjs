// render_voxel_item.mjs — 道具/武器/装备/法宝/血统 图标 → 3D 体素方块批量渲染(Edge headless CDP)
// ============================================================================
// 用法: node render_voxel_item.mjs [nameFilter] [views]
//   nameFilter 名称子串过滤(缺省: 全部 item_*/item_wp_*/gear_*/tr_*/bl_*/access_*)
//   views      逗号分隔视角(缺省: front,orbit45,top)
// 输出: tools/shots/render_voxel_item_<name>_<view>.png (1024×1024, 纯白背景)
// 依赖: server-rs/ui/vendor/three.min.js + server-rs/ui/js/voxel_item.js
// 建模: 纯颜色体素(MC 方块), 严禁贴图/纹理。
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const GAME = path.join(__dirname, '..', 'server-rs', 'ui');
const IMG = path.join(GAME, 'assets', 'img');
const SHOTS = path.join(__dirname, 'shots');
const THREE_JS = path.join(GAME, 'vendor', 'three.min.js');
const BUILDER_JS = path.join(GAME, 'js', 'voxel_item.js');
const PORT = 9696;
const SIZE = 1024;

fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

const nameFilter = process.argv[2] || '';
const views = (process.argv[3] || 'front,orbit45,top').split(',');
// each view: azimuth degrees / elevation degrees (az around Y, el above ground)
const VIEW_DEF = { front: [0, 8], orbit45: [45, 26], top: [45, 78], side: [90, 8] };

function fileUrl(p) {
  return 'file:///' + p.replace(/\\/g, '/').replace(/^\/+/, '');
}

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

function cleanLaunch(edge, userDir, port, html) {
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
    html,
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
  if (r.exceptionDetails) throw new Error('eval exc: ' + JSON.stringify(r.exceptionDetails).slice(0, 600));
  return r.result?.value;
}
async function setViewport(w, h) {
  await cdp('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await evalJs('window.dispatchEvent(new Event("resize")); true');
}

function buildHostHtml() {
  const three = fs.readFileSync(THREE_JS, 'utf8');
  const builder = fs.readFileSync(BUILDER_JS, 'utf8');
  return `<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="utf-8">
<style>html,body{margin:0;padding:0;overflow:hidden;background:#ffffff}</style>
</head><body>
<script>${three}</script>
<script>${builder}</script>
<script>
var renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(1);
renderer.setSize(${SIZE}, ${SIZE});
document.body.appendChild(renderer.domElement);
var scene = new THREE.Scene();
scene.background = new THREE.Color(0xffffff);
var camera = new THREE.PerspectiveCamera(34, 1, 0.1, 300);
// 光照: 环境光 + 上/前侧 定向光(让方块各面明暗可辨, MC 立体感)
scene.add(new THREE.AmbientLight(0xffffff, 0.62));
var key = new THREE.DirectionalLight(0xffffff, 0.85);
key.position.set(3, 5, 4);
scene.add(key);
var fill = new THREE.DirectionalLight(0xffffff, 0.35);
fill.position.set(-3, -1, -2);
scene.add(fill);
var group = new THREE.Group();
scene.add(group);

window.__setVoxel = function (url, kind, disc) {
  if (window.__rstats) { window.__rstats = null; }
  var opts = { url: url, kind: kind };
  if (disc) opts.disc = true;
  window.VoxelItem.build(group, opts);
  renderer.render(scene, camera);
  return 'ok';
};

window.__fit = function () {
  var b = window.__voxelBounds || { minX:-1,maxX:1,minY:-1,maxY:1,minZ:-1,maxZ:1, };
  var sx = b.maxX - b.minX, sy = b.maxY - b.minY, sz = b.maxZ - b.minZ;
  var cx = (b.minX + b.maxX) / 2, cy = (b.minY + b.maxY) / 2, cz = (b.minZ + b.maxZ) / 2;
  var r = Math.max(sx, Math.max(sy, sz)) / 2;
  var dist = r / Math.sin(17 * Math.PI / 180) * 1.45;
  window.__center = new THREE.Vector3(cx, cy, cz);
  window.__dist = dist;
  return JSON.stringify({ r: r, dist: dist, cx: cx, cy: cy, cz: cz });
};

window.__setView = function (azDeg, elDeg) {
  var az = azDeg * Math.PI / 180, el = elDeg * Math.PI / 180;
  var c = window.__center || { x: 0, y: 0, z: 0 };
  var d = window.__dist || 4;
  camera.position.set(
    c.x + d * Math.cos(el) * Math.sin(az),
    c.y + d * Math.sin(el),
    c.z + d * Math.cos(el) * Math.cos(az)
  );
  camera.lookAt(c.x, c.y, c.z);
  camera.updateMatrixWorld(true);
  renderer.render(scene, camera);
  return 'ok';
};

function tick() { if (window.__voxelReady) renderer.render(scene, camera); requestAnimationFrame(tick); }
requestAnimationFrame(tick);
</script></body></html>`;
}

function listTargets() {
  const out = [];
  for (const f of fs.readdirSync(IMG)) {
    if (!f.toLowerCase().endsWith('.png')) continue;
    if (!/^(item|item_wp|gear|tr|bl|access)_/i.test(f)) continue;
    // 排除 item_viewer 用的额外命名不存在的: 仅保留上述前缀
    out.push(f);
  }
  out.sort();
  if (nameFilter) return out.filter(f => f.toLowerCase().includes(nameFilter.toLowerCase()));
  return out;
}

async function main() {
  const edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe'); process.exit(2); }
  const targets = listTargets();
  if (!targets.length) { console.error('没有匹配的图标 (filter=' + nameFilter + ')'); process.exit(2); }
  log('targets=' + targets.length + ' views=' + views.join(',') + ' -> ' + SHOTS);

  const html = buildHostHtml();
  const htmlPath = path.join(os.tmpdir(), 'voxel_render_host_' + process.pid + '.html');
  fs.writeFileSync(htmlPath, html, 'utf8');
  const userDir = path.join(os.tmpdir(), 'voxel_render_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });

  const child = cleanLaunch(edge, userDir, PORT, htmlPath);
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(800);
  log('hasVoxelItem=' + await evalJs('!!window.VoxelItem'));

  await setViewport(SIZE, SIZE);
  await sleep(400);

  const results = [];
  for (const full of targets) {
    const name = full.replace(/\.png$/i, '');
    const url = fileUrl(path.join(IMG, full));
    // 判断是否 bl_/tr_ 以启用圆盘底座
    const isDisc = /^bl_|^tr_/i.test(name) ? 1 : 0;
    try {
      await evalJs(`window.__setVoxel(${JSON.stringify(url)}, null, ${isDisc})`);
      for (let i = 0; i < 80; i++) {
        await sleep(150);
        const ready = await evalJs('window.__voxelReady === true');
        if (ready) break;
      }
      const err = await evalJs('window.__voxelErr || null');
      const stats = JSON.parse(await evalJs('JSON.stringify(window.__voxelStats)') || 'null');
      if (err) { log(`SKIP ${name}: ${err}`); results.push({ name, ok: false, err }); continue; }
      if (!stats) { log(`SKIP ${name}: no stats`); results.push({ name, ok: false, err: 'no stats' }); continue; }
      await evalJs('window.__fit()');
      await sleep(120);
      const shots = [];
      for (const v of views) {
        const def = VIEW_DEF[v] || VIEW_DEF.front;
        await evalJs(`window.__setView(${def[0]}, ${def[1]})`);
        await sleep(220);
        const r = await cdp('Page.captureScreenshot', { format: 'png' });
        const out = path.join(SHOTS, `render_voxel_item_${name}_${v}.png`);
        fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
        shots.push(path.basename(out));
      }
      log(`OK ${name} cat=${stats.cat} grid=${stats.gridW}x${stats.gridH} layers=${stats.layers} voxels=${stats.voxels} fg=${stats.fgCells} ` + shots.join(' '));
      results.push({ name, ok: true, cat: stats.cat, voxels: stats.voxels, shots });
    } catch (e) {
      log(`FAIL ${name}: ${e.message}`);
      results.push({ name, ok: false, err: e.message });
    }
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  try { fs.unlinkSync(htmlPath); } catch {}

  const okN = results.filter(r => r.ok).length;
  const voxelsAll = results.reduce((s, r) => s + (r.voxels || 0), 0);
  log(`DONE render_voxel_item: ${okN}/${results.length} ok, total voxels=${voxelsAll}, shots=${okN * views.length}`);
  process.exit(0);
}

main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
