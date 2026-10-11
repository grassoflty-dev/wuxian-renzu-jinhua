// viewer_probe2.mjs — 验证双模型查看器: zz3d + ds0731 都能构建
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HTML = path.resolve(__dirname, '..', 'server-rs', 'ui', 'zz3d_viewer.html');
const PORT = 9722;
const sleep = ms => new Promise(r => setTimeout(r, ms));
function findEdge() {
  const p32 = process.env['PROGRAMFILES(X86)'] || 'C:\\Program Files (x86)';
  const p64 = process.env.PROGRAMFILES || 'C:\\Program Files';
  for (const c of [path.join(p64,'Microsoft','Edge','Application','msedge.exe'), path.join(p32,'Microsoft','Edge','Application','msedge.exe')]) if (fs.existsSync(c)) return c;
  return null;
}
(async () => {
  const edge = findEdge();
  const userDir = path.join(os.tmpdir(), 'viewerprobe2_' + process.pid);
  const child = spawn(edge, ['--headless=new', `--remote-debugging-port=${PORT}`, `--user-data-dir=${userDir}`, '--no-first-run', '--disable-gpu', '--allow-file-access-from-files', '--disable-sync', `file:///${HTML.replace(/\\/g,'/')}`], { stdio: 'ignore' });
  child.unref();
  let page;
  for (let i = 0; i < 60; i++) {
    try { const r = await fetch(`http://127.0.0.1:${PORT}/json/list`); const l = await r.json(); const p = l.find(x => x.type === 'page'); if (p) { page = p; break; } } catch {}
    await sleep(500);
  }
  if (!page) { console.log('NO PAGE'); process.exit(1); }
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  let id = 1;
  const cdp = (method, params = {}) => new Promise((resolve, reject) => {
    const mid = id++;
    const t = setTimeout(() => reject(new Error('timeout')), 15000);
    const on = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== mid) return; clearTimeout(t); ws.removeEventListener('message', on); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); };
    ws.addEventListener('message', on);
    ws.send(JSON.stringify({ id: mid, method, params }));
  });
  await cdp('Runtime.enable');
  const ev = (expr) => cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true }).then(r => r.result?.value);
  await sleep(4000);
  console.log('ZZ3DBuilder =', await ev('typeof window.ZZ3DBuilder'));
  console.log('SpriteBuilderDS0731 =', await ev('typeof window.SpriteBuilderDS0731'));
  console.log('stat after zz3d =', await ev('document.getElementById("stat").textContent'));
  // 切到 ds0731
  await ev('(function(){ var b=document.querySelector(\'#modelSw button[data-m="ds0731"]\'); if(b) b.click(); return 1; })()');
  await sleep(4000);
  console.log('stat after ds0731 =', await ev('document.getElementById("stat").textContent'));
  // 截图 ds0731 front
  await ev('document.querySelector(\'#ctrl button[data-v="front"]\').click()');
  await sleep(500);
  const shot = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(__dirname, 'shots', 'viewer_ds0731_front.png'), Buffer.from(shot.data, 'base64'));
  console.log('saved viewer_ds0731_front.png', Buffer.from(shot.data,'base64').length);
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})();
