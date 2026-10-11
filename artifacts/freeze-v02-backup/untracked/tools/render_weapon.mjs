// render_weapon.mjs — 武器图标 3D 立体渲染(Edge headless CDP)
// ============================================================================
// 用法: node render_weapon.mjs [nameFilter] [views]
//   nameFilter 武器名子串过滤(缺省: 全部 wp_*/item_wp_*)
//   views      逗号分隔视角列表(缺省: front,s45,side)
// 输出: tools/shots/render_weapon_<name>_<view>.png (纯白背景, 1024×1024)
// 依赖: server-rs/ui/vendor/three.min.js + server-rs/ui/js/weapon_builder.js
// 本脚本运行时在系统临时目录生成 HTML host, 结束后删除; 不触碰 item_viewer.html。
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
const BUILDER_JS = path.join(GAME, 'js', 'weapon_builder.js');
const PORT = 9731;
const SIZE = 1024;

fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

const nameFilter = process.argv[2] || '';
const views = (process.argv[3] || 'front,s45,side').split(',');
const VIEW_DEF = { front: [0, 8], s45: [45, 12], side: [90, 8] };

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
var camera = new THREE.PerspectiveCamera(30, 1, 0.1, 300);
var group = new THREE.Group();
scene.add(group);

window.__setWeapon = function (url, kind) {
  window.__modelReady = false;
  window.__modelErr = null;
  var opts = { url: url };
  if (kind) opts.kind = kind;
  var res = window.WeaponBuilder.build(group, opts);
  renderer.render(scene, camera);
  return res || '';
};

window.__fit = function () {
  var box = new THREE.Box3().setFromObject(group);
  var center = box.getCenter(new THREE.Vector3());
  var size = box.getSize(new THREE.Vector3());
  var r = Math.max(size.x, Math.max(size.y, size.z)) / 2;
  var dist = r / Math.sin(15 * Math.PI / 180) * 1.35;
  window.__center = center;
  window.__dist = dist;
  return JSON.stringify({ r: r, dist: dist, cx: center.x, cy: center.y, cz: center.z });
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

function tick() { if (window.__modelReady) renderer.render(scene, camera); requestAnimationFrame(tick); }
requestAnimationFrame(tick);
</script></body></html>`;
}

function listWeapons() {
  const out = [];
  for (const f of fs.readdirSync(IMG)) {
    if (!/^wp_.*\.png$/i.test(f) && !/^item_wp_.*\.png$/i.test(f)) continue;
    out.push(f);
  }
  out.sort();
  if (nameFilter) return out.filter(f => f.toLowerCase().includes(nameFilter.toLowerCase()));
  return out;
}

async function main() {
  const edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe'); process.exit(2); }
  const weapons = listWeapons();
  if (!weapons.length) { console.error('没有匹配的武器图标 (filter=' + nameFilter + ')'); process.exit(2); }
  log('weapons=' + weapons.length + ' views=' + views.join(','));

  const html = buildHostHtml();
  const htmlPath = path.join(os.tmpdir(), 'weapon_render_host_' + process.pid + '.html');
  fs.writeFileSync(htmlPath, html, 'utf8');

  const userDir = path.join(os.tmpdir(), 'weapon_render_' + process.pid);
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
  log('hasBuilder=' + await evalJs('!!window.WeaponBuilder'));

  await setViewport(SIZE, SIZE);
  await sleep(400);

  const results = [];
  for (const w of weapons) {
    const name = w.replace(/\.png$/i, '');
    const url = fileUrl(path.join(IMG, w));
    try {
      const kind = await evalJs(`window.__setWeapon(${JSON.stringify(url)}, null)`);
      for (let i = 0; i < 80; i++) {
        await sleep(200);
        const ready = await evalJs('window.__modelReady === true');
        if (ready) break;
      }
      const err = await evalJs('window.__modelErr || null');
      const stats = await evalJs('window.__modelStats || null');
      if (err) { log(`SKIP ${name}: ${err}`); results.push({ name, ok: false, err }); continue; }
      if (!stats) { log(`SKIP ${name}: no stats`); results.push({ name, ok: false, err: 'no stats' }); continue; }
      await evalJs('window.__fit()');
      await sleep(150);
      const shots = [];
      for (const v of views) {
        const def = VIEW_DEF[v] || VIEW_DEF.front;
        await evalJs(`window.__setView(${def[0]}, ${def[1]})`);
        await sleep(250);
        const r = await cdp('Page.captureScreenshot', { format: 'png' });
        const out = path.join(SHOTS, `render_weapon_${name}_${v}.png`);
        fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
        shots.push(path.basename(out));
      }
      log(`OK ${name} kind=${stats.kind} verts=${stats.totalVerts} tris=${stats.tris} angle=${stats.angleDeg}° ` + shots.join(' '));
      results.push({ name, ok: true, kind: stats.kind, verts: stats.totalVerts, shots });
    } catch (e) {
      log(`FAIL ${name}: ${e.message}`);
      results.push({ name, ok: false, err: e.message });
    }
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  try { fs.unlinkSync(htmlPath); } catch {}

  const okN = results.filter(r => r.ok).length;
  log(`DONE render_weapon: ${okN}/${results.length} ok`);
  process.exit(0);
}

main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
