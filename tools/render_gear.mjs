// render_gear.mjs — 装备/法宝/血统图标 3D 渲染(Edge headless CDP)
// ============================================================================
// 使用: node render_gear.mjs [name1,name2,...]
//   name 省略 -> 渲染全部 gear_*/tr_*/bl_* 图标。
//   每个图标渲染 3 视角: front(正面) / 45(45°斜视) / top(俯视)。
// 输出: tools/shots/render_gear_<name>_<view>.png
//
// 说明: 本脚本运行时把独立渲染壳(纯白背景 + three + js/gear_builder.js)写为临时
//       HTML 到 tools/shots/_render_gear_harness.html, 用 Edge headless CDP 加载它,
//       逐图标调用 window.__setModel(name) 重建模型并按包围盒自动取景截图。
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const UI = path.join(__dirname, '..', 'server-rs', 'ui');
const SHOTS = path.join(__dirname, 'shots');
const IMG_DIR = path.join(UI, 'assets', 'img');
const HARNESS = path.join(SHOTS, '_render_gear_harness.html');
const PORT = 9730;
fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

/* ---------- 目标图标清单 ---------- */
function iconList() {
  const names = [];
  const files = fs.readdirSync(IMG_DIR).filter(f => /\.(png)$/i.test(f));
  for (const f of files) {
    if (/^gear_/i.test(f) || /^tr_/i.test(f) || /^bl_/i.test(f)) {
      names.push(f.replace(/\.png$/i, ''));
    }
  }
  return names.sort();
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

/* ---------- 渲染壳 HTML(白底 + gear_builder + 包围盒自动取景) ---------- */
function harnessHTML() {
  const fileUrl = p => 'file:///' + path.join(UI, p).replace(/\\/g, '/').replace(/^\/+/, '');
  return `<!DOCTYPE html>
<html lang="zh-CN">
<head><meta charset="UTF-8"><title>gear_render</title>
<style>html,body{margin:0;padding:0;width:100%;height:100%;overflow:hidden;background:#ffffff}#host{position:fixed;left:0;top:0;right:0;bottom:0}</style>
</head>
<body><div id="host"></div>
<script src="${fileUrl('vendor/three.min.js')}"></script>
<script src="${fileUrl('js/gear_builder.js')}"></script>
<script>
var host=document.getElementById('host');
var renderer=new THREE.WebGLRenderer({antialias:true});
renderer.setPixelRatio(1);
renderer.setSize(window.innerWidth, window.innerHeight, true);
renderer.setClearColor(0xffffff,1);
renderer.outputColorSpace=THREE.SRGBColorSpace;
host.appendChild(renderer.domElement);
var scene=new THREE.Scene(); scene.background=new THREE.Color(0xffffff);
var camera=new THREE.PerspectiveCamera(30, window.innerWidth/window.innerHeight, 0.05, 500);
var group=new THREE.Group(); scene.add(group);

function fitBox(){
  group.updateMatrixWorld(true);
  var box=new THREE.Box3().setFromObject(group);
  if(box.isEmpty()){box=new THREE.Box3(new THREE.Vector3(-1,-1,-1),new THREE.Vector3(1,1,1));}
  var c=box.getCenter(new THREE.Vector3());
  var s=box.getSize(new THREE.Vector3());
  var maxDim=Math.max(s.x, s.y, s.z);
  var d=Math.max(1.2, maxDim*1.9);
  return {c:c, d:d};
}
window.setModelView=function(v){
  var fb=fitBox(), c=fb.c, d=fb.d;
  // 三种视角: front 正面 / 45 斜视 / top 俯视(bird's eye 显示顶边厚度)
  var p, look=[c.x,c.y,c.z];
  if(v==='front'){ p=[c.x, c.y, c.z+d]; }
  else if(v==='45'){ var a=Math.PI/4, e=0.31; p=[c.x - d*Math.cos(e)*Math.sin(a), c.y + d*Math.sin(e), c.z + d*Math.cos(e)*Math.cos(a)]; }
  else { var a2=0.52, e2=1.05; p=[c.x - d*Math.cos(e2)*Math.sin(a2), c.y + d*Math.sin(e2), c.z + d*Math.cos(e2)*Math.cos(a2)]; }
  // 上方补一点点 look 偏角
  camera.near=0.05; camera.far=500;
  camera.position.set(p[0],p[1],p[2]);
  camera.lookAt(look[0],look[1],look[2]);
  camera.updateProjectionMatrix();
  camera.updateMatrixWorld(true);
  renderer.render(scene,camera);
  return 'ok:'+v;
};
window.__setModel=function(name, extraOpts){
  if(!window.GearBuilder) return 'no GearBuilder';
  window.__gearReady=false;
  var opts={url:'${fileUrl('assets/img/')}'+name+'.png'};
  if(extraOpts) for(var k in extraOpts) opts[k]=extraOpts[k];
  var r=window.GearBuilder.build(group, opts);
  renderer.render(scene,camera);
  return (r||'')+':'+name;
};
function tick(){ if(window.__gearReady) renderer.render(scene,camera); requestAnimationFrame(tick); }
requestAnimationFrame(tick);
function resize(){ renderer.setSize(window.innerWidth, window.innerHeight, true); camera.aspect=window.innerWidth/window.innerHeight; camera.updateProjectionMatrix(); }
window.addEventListener('resize', resize);
window.ready=function(){ return !!window.GearBuilder; };
</script>
</body></html>`;
}

/* ---------- CDP 驱动 ---------- */
function cleanLaunch(edge, userDir, port) {
  try {
    const o = execSync(`netstat -ano | findstr :${port}`, { stdio: 'pipe' }).toString();
    for (const line of o.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  const htmlUrl = 'file:///' + HARNESS.replace(/\\/g, '/').replace(/^\/+/, '');
  const child = spawn(edge, [
    '--headless=new',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${userDir}`,
    '--no-first-run',
    '--disable-gpu',
    '--allow-file-access-from-files',
    htmlUrl,
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
async function connect(page) { ws = new WebSocket(page.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; }); }
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 30000);
    const onMsg = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== id) return; clearTimeout(timer); ws.removeEventListener('message', onMsg); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); };
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
  let names = process.argv.slice(2).filter(a => !a.startsWith('--'));
  if (!names.length) names = iconList();
  log(`icons(${names.length}): ${names.join(', ')}`);

  fs.writeFileSync(HARNESS, harnessHTML());
  const edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe'); process.exit(2); }
  const userDir = path.join(os.tmpdir(), 'render_gear_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });

  const child = cleanLaunch(edge, userDir, PORT);
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
  await connect(page);
  for (let i = 0; i < 40; i++) { try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {} await sleep(300); }
  await sleep(800);
  log('hasBuilder=' + await evalJs('!!window.GearBuilder'));
  await setViewport(2400, 1350);
  await sleep(500);

  const views = ['front', '45', 'top'];
  let ok = 0, fail = [];
  for (const name of names) {
    try {
      const r = await evalJs(`window.__setModel(${JSON.stringify(name)})`);
      for (let i = 0; i < 60; i++) { await sleep(250); if (await evalJs('window.__gearReady === true')) break; }
      const stats = await evalJs('JSON.stringify(window.__gearRes && window.__gearRes.stats || null)');
      const dbg = await evalJs('JSON.stringify(window.__gearDbg || null)');
      const err = await evalJs('window.__gearErr');
      if (err) { fail.push(name + ': ' + err); log(`  [${name}] ERR ${err}`); continue; }
      for (const v of views) {
        await evalJs(`window.setModelView(${JSON.stringify(v)})`);
        await sleep(380);
        const r2 = await cdp('Page.captureScreenshot', { format: 'png' });
        const out = path.join(SHOTS, `render_gear_${name}_${v}.png`);
        fs.writeFileSync(out, Buffer.from(r2.data, 'base64'));
      }
      ok++;
      log(`  [${name}] ok stats=${stats} dbg=${dbg}`);
    } catch (e) {
      fail.push(name + ': ' + (e && e.message));
      log(`  [${name}] FAIL ${e && e.message}`);
    }
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  log(`DONE render_gear ok=${ok} fail=${fail.length}`);
  if (fail.length) log('FAILURES:\n' + fail.join('\n'));
  process.exit(0);
}
main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
