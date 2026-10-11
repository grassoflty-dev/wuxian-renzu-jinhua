import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const EXE = path.resolve(__dirname, '..', 'server-rs', 'target', 'release', 'wuxian-horror-ch1.exe');
const OUT = path.join(__dirname, 'shots');
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
fs.mkdirSync(OUT, { recursive: true });
function cleanLaunch() {
  try { execSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul', { stdio: 'pipe' }); } catch {}
  const child = spawn(EXE, [], { env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=' + PORT }, stdio: 'ignore' });
  child.unref(); return child;
}
async function getPage() {
  for (let i = 0; i < 60; i++) {
    try { const r = await fetch('http://127.0.0.1:' + PORT + '/json/list'); const l = await r.json(); const p = l.find(x => x.type === 'page'); if (p) return p; } catch {}
    await sleep(500);
  }
  throw new Error('no page');
}
let ws, nextId = 1;
async function connect(p) { ws = new WebSocket(p.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; }); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const t = setTimeout(() => reject(new Error('cdp timeout ' + method)), 25000);
    const on = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== id) return; clearTimeout(t); ws.removeEventListener('message', on); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); };
    ws.addEventListener('message', on);
    ws.send(JSON.stringify({ id, method, params }));
  });
}
async function evalJs(expr) {
  const r = await cdp('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) return 'EXC:' + JSON.stringify(r.exceptionDetails).slice(0, 400);
  return r.result ? r.result.value : undefined;
}
const inv = (cmd, args) => evalJs('(async function(){ try { return await window.__TAURI__.core.invoke(' + JSON.stringify(cmd) + ', ' + JSON.stringify(args || {}) + '); } catch(e){ return {__err:String(e.message||e)}; } })()');
async function shot(name) {
  const r = await cdp('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(path.join(OUT, name), Buffer.from(r.data, 'base64'));
  console.log('[shot]', name, fs.statSync(path.join(OUT, name)).size);
}
(async () => {
  const child = cleanLaunch();
  let page = await getPage();
  await connect(page);
  await sleep(3000);
  await evalJs('(function(){var b=document.getElementById("btnNew"); if(b){b.click(); return 1;} return 0;})()');
  await sleep(1800);
  await evalJs('(function(){var b=document.getElementById("btnTutStart")||document.getElementById("btnTutSkip"); if(b){b.click(); return 1;} return 0;})()');
  await sleep(800);
  let world = await inv('api_world');
  if (world.__err) { console.log('ERR api_world', world.__err); process.exit(2); }
  const tiles = world.tiles, W = world.w, H = world.h, px = world.px, py = world.py;
  const enemies = (world.enemies || []).filter(e => e.alive);
  console.log('world', world.floor_name, '@(' + px + ',' + py + ') enemies=', enemies.length, 'size', W + 'x' + H);
  if (!enemies.length) { console.log('no alive enemies'); process.exit(3); }
  const target = enemies.slice().sort((a,b) => (Math.abs(a.x-px)+Math.abs(a.y-py)) - (Math.abs(b.x-px)+Math.abs(b.y-py)))[0];
  console.log('target', JSON.stringify(target));
  const g = Array.from({length:H}, () => Array(W).fill(false));
  for (let y=0;y<H;y++) for (let x=0;x<W;x++) if ((tiles[y]||'')[x] === '#') g[y][x] = true;
  (world.gates||[]).forEach(gt => { if (gt.locked) g[gt.y][gt.x] = true; });
  (world.enemies||[]).forEach(e => { if (e.alive) g[e.y][e.x] = true; });
  const prev = Array.from({length:H}, () => Array(W).fill(null));
  const dist = Array.from({length:H}, () => Array(W).fill(-1));
  const q = [[px,py]]; dist[py][px] = 0;
  while (q.length) {
    const [x,y] = q.shift();
    for (const [dx,dy] of [[1,0],[-1,0],[0,1],[0,-1]]) {
      const nx=x+dx, ny=y+dy;
      if (nx<0||ny<0||nx>=W||ny>=H) continue;
      if (g[ny][nx] || dist[ny][nx]>=0) continue;
      dist[ny][nx]=dist[y][x]+1; prev[ny][nx]=[x,y]; q.push([nx,ny]);
    }
  }
  const near = [[target.x-1,target.y],[target.x+1,target.y],[target.x,target.y-1],[target.x,target.y+1]]
    .filter(([x,y]) => x>=0 && y>=0 && x<W && y<H && dist[y][x] >= 0)
    .sort((a,b) => dist[a[1]][a[0]] - dist[b[1]][b[0]]);
  if (!near.length) { console.log('enemy unreachable'); process.exit(4); }
  const [gx, gy] = near[0];
  const pathRev = []; let cur = [gx,gy];
  while (cur && !(cur[0]===px && cur[1]===py)) { pathRev.push(cur); cur = prev[cur[1]][cur[0]]; }
  const path = pathRev.reverse();
  console.log('path steps', path.length);
  let entered = false;
  for (const [tx,ty] of path) {
    const r = await inv('api_world_move', { dx: tx - world.px, dy: ty - world.py });
    if (r.__err) { console.log('move err', r.__err); break; }
    world = await inv('api_world');
    if (r.encounter) {
      console.log('ENCOUNTER', JSON.stringify(r.encounter));
      const eid = r.encounter.enemy_id, ref = r.encounter.fight_id, nm = r.encounter.name;
      await evalJs('(async function(){ try { await window.enterZone({id:' + JSON.stringify(eid) + ', kind:"fight", ref:' + JSON.stringify(ref) + ', name:' + JSON.stringify(nm) + '}, null); return "ok"; } catch(e){ return "ERR:"+String(e.message||e); } })()');
      await sleep(2200); entered = true; break;
    }
  }
  if (!entered) {
    const r = await inv('api_world_move', { dx: target.x - world.px, dy: target.y - world.py });
    if (r.encounter) {
      const eid = r.encounter.enemy_id, ref = r.encounter.fight_id, nm = r.encounter.name;
      await evalJs('(async function(){ try { await window.enterZone({id:' + JSON.stringify(eid) + ', kind:"fight", ref:' + JSON.stringify(ref) + ', name:' + JSON.stringify(nm) + '}, null); return "ok"; } catch(e){ return "ERR:"+String(e.message||e); } })()');
      await sleep(2200); entered = true;
    }
  }
  const zi = await evalJs('(function(){ return { zoneActive: window.ZoneActive===true, title:(document.getElementById("zoneTitle")||{}).textContent }; })()');
  console.log('zone', JSON.stringify(zi));
  // 站立截图
  await shot('combat_stand.png');
  // 向左移动（触发行走腿部动画）
  await evalJs('(function(){ window.dispatchEvent(new KeyboardEvent("keydown",{key:"a",bubbles:true})); return 1; })()');
  await sleep(450);
  await shot('combat_walk.png');
  await evalJs('(function(){ window.dispatchEvent(new KeyboardEvent("keyup",{key:"a",bubbles:true})); return 1; })()');
  await sleep(200);
  // 角色特写裁剪（中央偏右 ~55%/65%）
  const dim = await evalJs('(function(){ return {w:window.innerWidth, h:window.innerHeight}; })()');
  const clip = await cdp('Page.captureScreenshot', { format:'png', clip: { x: Math.max(0, Math.round(dim.w*0.45)-200), y: Math.max(0, Math.round(dim.h*0.55)-220), width: 420, height: 440, scale: 1 } });
  fs.writeFileSync(path.join(OUT, 'combat_crop.png'), Buffer.from(clip.data, 'base64'));
  console.log('[shot] combat_crop.png', fs.statSync(path.join(OUT, 'combat_crop.png')).size);
  ws.close(); child.kill();
  console.log('DONE entered=' + entered);
  process.exit(0);
})().catch(e => { console.error('FATAL', e.message); try { ws && ws.close(); } catch {} process.exit(2); });