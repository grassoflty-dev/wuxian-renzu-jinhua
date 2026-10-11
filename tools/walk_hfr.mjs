// walk_hfr.mjs — 高帧率录制行走动画：进游戏→走到开阔处→按住方向键，页内 canvas 以 ~16ms 间隔连拍玩家区域 12 帧
// 输出：shots/walk_hfr/ 下 12 张玩家特写帧 + 一张拼贴 contact_sheet.png
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const EXE = path.resolve(__dirname, '..', 'server-rs', 'target', 'release', 'wuxian-horror-ch1.exe');
const OUT = path.join(__dirname, 'shots', 'walk_hfr');
fs.mkdirSync(OUT, { recursive: true });
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, nextId = 1;
async function killPort() { try { execSync(`taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul`, { stdio: 'pipe' }); } catch {} await sleep(900); }
async function getPage() { for (let i = 0; i < 150; i++) { try { const r = await fetch(`http://127.0.0.1:${PORT}/json/list`); const list = await r.json(); const p = list.find(x => x.type === 'page'); if (p) return p; } catch {} await sleep(500); } throw new Error('page not found'); }
async function connect(page) { ws = new WebSocket(page.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; }); await cdp('Runtime.enable'); }
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) { return new Promise((resolve, reject) => { const id = nextId++; const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 30000); const onMsg = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== id) return; clearTimeout(timer); ws.removeEventListener('message', onMsg); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); }; ws.addEventListener('message', onMsg); send({ id, method, params }); }); }
async function evalJs(expression) { const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true }); if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 300); return r.result?.value; }
async function key(k, down = true) { await evalJs(`window.dispatchEvent(new KeyboardEvent('${down ? 'keydown' : 'keyup'}', { key: '${k}', bubbles: true })); true`); }

// 页内高帧率连拍：抓玩家区域(屏幕中心) N 帧，间隔 ~16ms，返回 dataURL 数组
const CAPTURE = `(async (N) => {
  const c = document.getElementById('worldCanvas'); if(!c) return { err:'no canvas' };
  const g = c.getContext('2d');
  const W = 130, H = 150;                       // 玩家特写尺寸
  const cx = Math.floor(c.width/2), cy = Math.floor(c.height/2) - 10;
  const frames = [];
  for (let i = 0; i < N; i++) {
    const tmp = document.createElement('canvas'); tmp.width = W; tmp.height = H;
    tmp.getContext('2d').drawImage(c, cx - W/2, cy - H/2, W, H, 0, 0, W, H);
    frames.push(tmp.toDataURL('image/png'));
    await new Promise(r => setTimeout(r, 16));
  }
  return frames;
})`;

(async () => {
  await killPort();
  const child = spawn(EXE, [], { env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` }, stdio: 'ignore' });
  child.unref();
  await sleep(4000);
  const page = await getPage(); await connect(page);
  for (let i = 0; i < 40; i++) { try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {} await sleep(300); }
  await sleep(1500);
  await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(2200);
  await evalJs(`(() => { const b=document.getElementById('btnTutSkip')||document.getElementById('btnTutStart'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(2200);

  // 走到开阔处：先向下+右各走 4 步，离开左上角墙
  for (let i = 0; i < 4; i++) { await key('s', true); await sleep(260); await key('s', false); await sleep(160); }
  for (let i = 0; i < 4; i++) { await key('d', true); await sleep(260); await key('d', false); await sleep(160); }
  await sleep(400);

  // 高帧率连拍：按住 d(向右) 连拍 12 帧
  await key('d', true);
  await sleep(80);
  const frames = await evalJs(`(${CAPTURE})(12)`);
  await key('d', false);
  if (!Array.isArray(frames)) { console.log('CAPTURE_ERR', JSON.stringify(frames)); process.exit(2); }
  // 落盘单帧
  frames.forEach((d, i) => {
    if (d && d.startsWith('data:image/png')) fs.writeFileSync(path.join(OUT, `f${String(i).padStart(2,'0')}.png`), Buffer.from(d.split(',')[1], 'base64'));
  });
  console.log('FRAMES', frames.length);
  // 拼贴 contact sheet（3 行 x 4 列，每帧缩到 130x150）
  const sheet = await evalJs(`(async () => {
    const c = document.getElementById('worldCanvas'); if(!c) return null;
    const W=130,H=150,CX=Math.floor(c.width/2),CY=Math.floor(c.height/2)-10;
    const tmp=document.createElement('canvas'); tmp.width=W*4+12; tmp.height=H*3+12;
    const tg=tmp.getContext('2d'); tg.fillStyle='#111'; tg.fillRect(0,0,tmp.width,tmp.height);
    // 重新抓一次用于拼贴（12 帧）
    for(let i=0;i<12;i++){ const x=i%4, y=Math.floor(i/4); tg.drawImage(c, CX-W/2, CY-H/2, W, H, x*(W+3), y*(H+3), W, H); await new Promise(r=>setTimeout(r,16)); }
    return tmp.toDataURL('image/png');
  })()`);
  if (sheet && sheet.startsWith('data:image/png')) { fs.writeFileSync(path.join(OUT, 'contact_sheet.png'), Buffer.from(sheet.split(',')[1], 'base64')); console.log('SHEET saved'); }

  try { ws.close(); } catch {} try { child.kill(); } catch {}
  console.log('DONE');
  process.exit(0);
})().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
