// run_ops_acceptance.mjs — 玩家常规操作清单 + CDP 执行 + 前后截图(主线自己识图评估)
// 用法: node run_ops_acceptance.mjs  → 输出 tools/op_feedback/shots/op_<id>_{before,after}.png + 操作清单 md
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const OUT = path.join(__dirname, 'op_feedback', 'shots');
fs.mkdirSync(OUT, { recursive: true });
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
  if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 300);
  return r.result?.value;
}
async function shot(tag) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  const out = path.join(OUT, tag + '.png');
  fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
  return out;
}
async function key(k, down = true) {
  await evalJs(`window.dispatchEvent(new KeyboardEvent('${down ? 'keydown' : 'keyup'}', { key: '${k}', keyCode: ${k.toUpperCase().charCodeAt(0)}, bubbles: true })); true`);
}

const OPS = [
  { id: 'move_w', name: 'W 前进×3', run: async () => { for (let i = 0; i < 3; i++) { await key('w'); await sleep(350); await key('w', false); await sleep(250); } } },
  { id: 'move_a', name: 'A 左移×2', run: async () => { for (let i = 0; i < 2; i++) { await key('a'); await sleep(350); await key('a', false); await sleep(250); } } },
  { id: 'bigmap', name: 'M 大地图', run: async () => { await key('m'); await sleep(800); } },
  { id: 'bigmap_close', name: 'M 关大地图', run: async () => { await key('m'); await sleep(600); } },
  { id: 'skill_tab', name: 'Tab 技能面板', run: async () => { await evalJs(`window.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',keyCode:9,bubbles:true})); true`); await sleep(800); } },
  { id: 'skill_close', name: 'Tab 关技能面板', run: async () => { await evalJs(`window.dispatchEvent(new KeyboardEvent('keydown',{key:'Tab',keyCode:9,bubbles:true})); true`); await sleep(600); } },
  { id: 'settings', name: '设置面板', run: async () => { await evalJs(`(() => { const b = document.getElementById('btnSettings'); if (b) { b.click(); return 1; } return 0; })()`); await sleep(800); } },
  { id: 'settings_close', name: '关设置', run: async () => { await evalJs(`(() => { const b = document.getElementById('btnSettingsClose'); if (b) { b.click(); return 1; } return 0; })()`); await sleep(600); } },
  { id: 'wheel_zoom', name: '滚轮拉近', run: async () => { await evalJs(`(() => { const cs=[...document.querySelectorAll('canvas')]; const c = cs.find(x=>x.style && x.style.position==='fixed') || cs[cs.length-1]; if (c) { c.dispatchEvent(new WheelEvent('wheel', { deltaY: -240, bubbles: true, cancelable: true })); return c.id||'fixed-canvas'; } return 0; })()`); await sleep(700); } },
  { id: 'wheel_out', name: '滚轮拉远', run: async () => { await evalJs(`(() => { const cs=[...document.querySelectorAll('canvas')]; const c = cs.find(x=>x.style && x.style.position==='fixed') || cs[cs.length-1]; if (c) { c.dispatchEvent(new WheelEvent('wheel', { deltaY: 240, bubbles: true, cancelable: true })); return c.id||'fixed-canvas'; } return 0; })()`); await sleep(700); } },
];

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
  // 进游戏 + 跳新手引导
  await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(2500);
  await evalJs(`(() => { const b=document.getElementById('btnTutSkip')||document.getElementById('btnTutStart'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(2500);
  const md = ['# 玩家常规操作清单 + 画面反馈验收', '', '| 操作 | 预期反馈 | 截图 |', '|---|---|---|'];
  for (const op of OPS) {
    await shot(`op_${op.id}_before`);
    await op.run();
    await sleep(500);
    await shot(`op_${op.id}_after`);
    md.push(`| ${op.name} | (主线识图评估) | op_${op.id}_before/after |`);
    console.log('OP ' + op.id + ' done');
  }
  fs.writeFileSync(path.join(__dirname, 'op_feedback', 'operations_list.md'), md.join('\n'), 'utf8');
  console.log('DONE ' + OPS.length + ' ops');
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); process.exit(2); });
