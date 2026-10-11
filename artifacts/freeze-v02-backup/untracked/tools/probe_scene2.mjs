// ⚠️【废弃标注】游戏已切换到 World2D 引擎（index.html 不再加载 world3d.js），window.__w3d 已不存在，本脚本只能输出 NO __w3d，已不可用。地图画面体检请用 probe_world2d.mjs。
// probe_scene2.mjs — 深入诊断 InstancedMesh(墙/地板)为何不可见
import { spawn, execSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, nextId = 1;
async function killPort() {
  try {
    const out = execSync(`netstat -ano | findstr :${PORT}`, { stdio: 'pipe' }).toString();
    for (const line of out.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  try { execSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul', { stdio: 'pipe' }); } catch {}
  await sleep(800);
}
async function getPage() {
  for (let i = 0; i < 150; i++) {
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
async function connect(page) {
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
}
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 20000);
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
async function evalJs(expression) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 400);
  return r.result?.value;
}
(async () => {
  await killPort();
  const child = spawn(EXE, [], {
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` },
    stdio: 'ignore',
  });
  child.unref();
  await sleep(4000);
  const page = await getPage();
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(2000);
  await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 'ok';} return 'nobtn'; })()`);
  await sleep(2500);
  await evalJs(`(() => { const b=document.getElementById('btnTutSkip')||document.getElementById('btnTutStart'); if(b){b.click(); return 'ok';} return 'notut'; })()`);
  await sleep(3000);
  const info = await evalJs(`(() => {
    const w = window.__w3d; if (!w) return 'NO __w3d';
    const out = [];
    const m4 = new THREE.Matrix4(); const v3 = new THREE.Vector3();
    let n = 0;
    w.scene.traverse(o => {
      if (o.isInstancedMesh && n < 8) {
        n++;
        o.getMatrixAt(0, m4);
        const pos = v3.setFromMatrixPosition(m4);
        const mat = Array.isArray(o.material) ? o.material[0] : o.material;
        const bs = o.boundingSphere;
        out.push({
          count: o.count, visible: o.visible, frustumCulled: o.frustumCulled,
          firstPos: [Math.round(pos.x), Math.round(pos.y), Math.round(pos.z)],
          matColor: mat && mat.color ? '#' + mat.color.getHexString() : null,
          matTransparent: mat ? mat.transparent : null, matOpacity: mat ? mat.opacity : null,
          bsCenter: bs ? [Math.round(bs.center.x), Math.round(bs.center.y), Math.round(bs.center.z)] : null,
          bsRadius: bs ? Math.round(bs.radius) : null,
          geoH: o.geometry.parameters ? o.geometry.parameters.height : null,
          inScene: !!o.parent
        });
      }
    });
    // 相机位置
    let cam = null;
    w.scene.traverse(o => { if (o.isCamera) cam = [Math.round(o.position.x), Math.round(o.position.y), Math.round(o.position.z)]; });
    return JSON.stringify({ inst: out, cam });
  })()`);
  console.log('DETAIL ' + info);
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
