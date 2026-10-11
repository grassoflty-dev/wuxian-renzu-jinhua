// viewer_probe.mjs — 验证 zz3d_viewer.html 可加载(无 JS 错误,模型构建成功)
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HTML = path.resolve(__dirname, '..', 'server-rs', 'ui', 'zz3d_viewer.html');
const PORT = 9721;
const sleep = ms => new Promise(r => setTimeout(r, ms));
function findEdge() {
  const p32 = process.env['PROGRAMFILES(X86)'] || 'C:\\Program Files (x86)';
  const p64 = process.env.PROGRAMFILES || 'C:\\Program Files';
  for (const c of [path.join(p64,'Microsoft','Edge','Application','msedge.exe'), path.join(p32,'Microsoft','Edge','Application','msedge.exe')]) if (fs.existsSync(c)) return c;
  return null;
}
(async () => {
  const edge = findEdge();
  const userDir = path.join(os.tmpdir(), 'viewerprobe_' + process.pid);
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
  console.log('__modelReady =', await ev('window.__modelReady'));
  console.log('renderer =', await ev('typeof (window.__zz3d_r || null)'));
  // 模型顶点数
  console.log('group children =', await ev('(function(){ try { var g = document.querySelector("#host").__scene; return "?"; } catch(e){ return "n/a"; } })()'));
  // 直接查渲染器是否运行: 检查 canvas 尺寸
  console.log('canvas =', await ev('(function(){ var c=document.querySelector("canvas"); return c ? c.width+"x"+c.height : "none"; })()'));
  // 查模型构建返回
  console.log('stat text =', await ev('document.getElementById("stat").textContent'));
  // 截图确认
  const shot = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(__dirname, 'shots', 'viewer_probe.png'), Buffer.from(shot.data, 'base64'));
  console.log('screenshot saved viewer_probe.png', Buffer.from(shot.data,'base64').length);
  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  process.exit(0);
})();
