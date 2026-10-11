// walk_hfr2.mjs — 高帧率行走动画录制（CDP 截图版）：进游戏→走到开阔处→按住方向键，连续截 8 张玩家特写
// 输出 shots/walk_hfr2/f00..f07.png + contact_sheet.png（供视觉模型逐帧看动作）
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const EXE = path.resolve(__dirname, '..', 'server-rs', 'target', 'release', 'wuxian-horror-ch1.exe');
const OUT = path.join(__dirname, 'shots', 'walk_hfr2');
fs.mkdirSync(OUT, { recursive: true });
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, nextId = 1;
async function killPort() { try { execSync(`taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul`, { stdio: 'pipe' }); } catch {} await sleep(900); }
async function getPage() { for (let i = 0; i < 150; i++) { try { const r = await fetch(`http://127.0.0.1:${PORT}/json/list`); const list = await r.json(); const p = list.find(x => x.type === 'page'); if (p) return p; } catch {} await sleep(500); } throw new Error('page not found'); }
async function connect(page) { ws = new WebSocket(page.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; }); await cdp('Runtime.enable'); await cdp('Page.enable'); }
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) { return new Promise((resolve, reject) => { const id = nextId++; const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 30000); const onMsg = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== id) return; clearTimeout(timer); ws.removeEventListener('message', onMsg); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); }; ws.addEventListener('message', onMsg); send({ id, method, params }); }); }
async function evalJs(expression) { const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true }); if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 300); return r.result?.value; }
async function key(k, down = true) { await evalJs(`window.dispatchEvent(new KeyboardEvent('${down ? 'keydown' : 'keyup'}', { key: '${k}', bubbles: true })); true`); }

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

  // 走到开阔处（下4 右4，离开左上角）
  for (let i = 0; i < 4; i++) { await key('s', true); await sleep(240); await key('s', false); await sleep(140); }
  for (let i = 0; i < 4; i++) { await key('d', true); await sleep(240); await key('d', false); await sleep(140); }
  await sleep(300);
  const pos0 = await evalJs(`(() => { const d=window.World3D.debug(); return d.px+','+d.py; })()`);
  console.log('POS_BEFORE', pos0);

  // 按住 d（向右走），CDP 连拍 8 张（~90ms 一张）
  await key('d', true);
  await sleep(60);
  for (let i = 0; i < 8; i++) {
    const r = await cdp('Page.captureScreenshot', { format: 'png' });
    fs.writeFileSync(path.join(OUT, `full${i}.png`), Buffer.from(r.data, 'base64'));
    await sleep(90);
  }
  await key('d', false);
  const pos1 = await evalJs(`(() => { const d=window.World3D.debug(); return d.px+','+d.py; })()`);
  console.log('POS_AFTER', pos1);

  // 页内裁剪玩家特写（屏幕中心 160x180 → 存 f00..f07 + 拼贴）
  const crop = await evalJs(`(async () => {
    const c = document.getElementById('worldCanvas'); if(!c) return null;
    const W=160,H=180, CX=Math.floor(c.width/2), CY=Math.floor(c.height/2)-6;
    const tmp=document.createElement('canvas'); tmp.width=W; tmp.height=H; const tg=tmp.getContext('2d');
    const out=[]; const sheet=document.createElement('canvas'); sheet.width=W*4+15; sheet.height=H*2+15; const sg=sheet.getContext('2d'); sg.fillStyle='#111'; sg.fillRect(0,0,sheet.width,sheet.height);
    for(let i=0;i<8;i++){ tg.clearRect(0,0,W,H); tg.drawImage(c,CX-W/2,CY-H/2,W,H,0,0,W,H); out.push(tmp.toDataURL('image/png')); sg.drawImage(tmp, (i%4)*(W+3), Math.floor(i/4)*(H+3)); }
    return { frames: out, sheet: sheet.toDataURL('image/png') };
  })()`);
  if (crop && crop.frames) {
    crop.frames.forEach((d, i) => { if (d && d.startsWith('data:image/png')) fs.writeFileSync(path.join(OUT, `f${String(i).padStart(2,'0')}.png`), Buffer.from(d.split(',')[1], 'base64')); });
    if (crop.sheet && crop.sheet.startsWith('data:image/png')) fs.writeFileSync(path.join(OUT, 'contact_sheet.png'), Buffer.from(crop.sheet.split(',')[1], 'base64'));
    console.log('CROP saved', crop.frames.length);
  }

  try { ws.close(); } catch {} try { child.kill(); } catch {}
  console.log('DONE');
  process.exit(0);
})().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
