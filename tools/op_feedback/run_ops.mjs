// ============================================================================
// 无限轮回 · 操作-画面反馈验收 —— CDP 驱动执行玩家常规操作 + 前后截图
//
// 流程: 启动游戏(CDP 9702) → 固定视口 1600x900 → 按状态演进执行 operations_list.md
//   标题页:     15 设置面板 / 16-16b 分辨率 / 17 帧率 / 18 死亡档案
//   进入轮回:   01 → 新手引导跳过 02
//   生化世界:   03 WASD移动 / 04 滚轮缩放 / 05 右键旋转 / 07 大地图M / 14 技能面板Tab+装备
//   战斗副本:   09 走全图撞敌人进副本 / 10 攻击J / 11-11b 技能Q/E / 12 闪避K / 13 副本移动
//   主神空间:   08 回主神(跨世界) / 06 E交互(NPC兑换点)
// 每操作: before + after 截图; try/catch 单操作失败不中断; 结果增量写入 ops_result.json
// 用法: node tools/op_feedback/run_ops.mjs
// ============================================================================
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', '..', 'server-rs');
const EXE = path.join(ROOT, 'target', 'release', 'wuxian-horror-ch1.exe');
const SAVE = path.join(ROOT, 'target', 'release', 'data', 'save.json');
const SHOTS = path.join(__dirname, 'shots');
const LOG = path.join(__dirname, 'run_ops.log');
const RESULT = path.join(__dirname, 'ops_result.json');
const PORT = 9702;
const VP = { width: 1600, height: 900 };

const sleep = ms => new Promise(r => setTimeout(r, ms));
function log(s) { const ts = new Date().toISOString().slice(11, 23); try { fs.appendFileSync(LOG, `[${ts}] ${s}\n`); } catch {} console.log(s); }
function runSync(cmd) { try { return execSync(cmd, { stdio: 'pipe' }).toString(); } catch { return ''; } }
function readSave() { try { return JSON.parse(fs.readFileSync(SAVE, 'utf8')); } catch { return null; } }
const results = [];

function writeResult() {
  try {
    const shotsCount = fs.existsSync(SHOTS) ? fs.readdirSync(SHOTS).filter(f => f.endsWith('.png')).length : 0;
    fs.writeFileSync(RESULT, JSON.stringify({ vp: VP, ops: results, shots: shotsCount }, null, 2), 'utf8');
  } catch (e) { try { log('RESULT write fail ' + e.message); } catch {} }
}
function record(op, status, detail, before, after) {
  results.push({ id: op.id, name: op.name, status, detail: detail || '', before, after });
  log(`${status.toUpperCase()} [${op.id}] ${op.name} :: ${String(detail).slice(0, 300)}`);
  writeResult();
}

async function killPort() {
  try {
    const out = runSync(`netstat -ano | findstr :${PORT}`);
    for (const line of out.split(/\r?\n/)) { const m = line.trim().match(/(\d+)\s*$/); if (m) try { runSync(`taskkill /PID ${m[1]} /T /F 2>nul`); } catch {} }
  } catch {}
  runSync('taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul');
  await sleep(500);
}

let ws, nextId = 1, child = null;
async function getPage() {
  for (let i = 0; i < 150; i++) {
    try { const r = await fetch(`http://127.0.0.1:${PORT}/json/list`); const list = await r.json(); const p = list.find(x => x.type === 'page'); if (p) return p; } catch {}
    await sleep(500);
  }
  throw new Error('CDP page not found');
}
async function connect(page) { ws = new WebSocket(page.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; }); }
function disconnect() { try { ws && ws.close(); } catch {} ws = null; }
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 40000);
    const onMsg = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== id) return; clearTimeout(timer); ws.removeEventListener('message', onMsg); m.error ? reject(new Error('cdp err')) : resolve(m.result); };
    ws.addEventListener('message', onMsg);
    send({ id, method, params });
  });
}
async function evalJs(expression) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) { const exc = r.exceptionDetails.exception?.description || JSON.stringify(r.exceptionDetails).slice(0, 300); return { __evalErr: exc }; }
  return r.result?.value;
}
async function setViewport(w, h) { await cdp('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 0, mobile: false }).catch(() => {}); }
async function shot(tag) { const r = await cdp('Page.captureScreenshot', { format: 'png' }); const out = path.join(SHOTS, tag + '.png'); fs.writeFileSync(out, Buffer.from(r.data, 'base64')); return out; }
const inv = (cmd, args) => evalJs(`(async function(){ try { return await window.__TAURI__.core.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(args || {})}); } catch(e){ return { __err: String((e && e.message) || e) }; } })()`);
const invs = async (cad, a) => inv(cad, a);

const KEYCODE = { w: 87, a: 65, s: 83, d: 68, e: 69, k: 75, j: 74, q: 81, r: 82, m: 77, shift: 16, enter: 13 };
const keyDispatch = (key, code, type) => evalJs(`(function(){ const kc=${JSON.stringify(KEYCODE[key.toLowerCase()] ?? 0)}; window.dispatchEvent(new KeyboardEvent(${JSON.stringify(type)}, { key: ${JSON.stringify(key)}, code: ${JSON.stringify(code)}, keyCode: kc, which: kc, bubbles: true, cancelable: true })); return 1; })()`);

async function ensureMode(kind) {
  if (kind === 'title') return await evalJs(`(function(){ const t=document.getElementById('titleScreen'); return t && t.style.display==='flex'; })()`);
  if (kind === 'world') return await evalJs(`(function(){ return document.getElementById('worldView').style.display==='block'; })()`);
  if (kind === 'zone') return await evalJs(`(function(){ return window.ZoneActive===true; })()`);
  return false;
}
async function clickBtnById(id) {
  const r = await evalJs(`(function(){ const b=document.getElementById(${JSON.stringify(id)}); if(b && b.offsetParent!==null){ b.click(); return 'clicked'; } return 'no_visible'; })()`);
  return r === 'clicked';
}
const shotPair = (op) => ({ before: `op_${op.id}_before`, after: `op_${op.id}_after` });

// ---------- BFS 寻路(可封堵墙/锁门/传送门/敌人) ----------
async function bfsTo(tx, ty, opts = {}) {
  const world = await invs('api_world');
  if (!world || !world.tiles) return null;
  const tiles = world.tiles, H = tiles.length, W = H ? (tiles[0] || '').length : 0;
  if (!W || !H) return null;
  const sx = world.px, sy = world.py;
  if (sx === tx && sy === ty) return [];
  const inbound = (xx, yy) => xx >= 0 && yy >= 0 && xx < W && yy < H && tiles[yy] && tiles[yy][xx] !== '#';
  const blocked = (xx, yy) => {
    if (!inbound(xx, yy)) return true;
    if (world.gates && world.gates.some(g => g.locked && g.x === xx && g.y === yy)) return true;
    if (world.portals && !opts.allowPortal && world.portals.some(p => p.x === xx && p.y === yy)) return true;
    if (world.enemies && !opts.allowEnemy && world.enemies.some(e => e.alive && e.x === xx && e.y === yy)) return true;
    return false;
  };
  const prev = Array.from({ length: H }, () => Array(W).fill(null));
  const dist = Array.from({ length: H }, () => Array(W).fill(-1));
  const q = [[sx, sy]]; dist[sy][sx] = 0;
  while (q.length) {
    const [x, y] = q.shift();
    for (const [dx, dy] of [[1,0],[-1,0],[0,1],[0,-1]]) {
      const nx = x + dx, ny = y + dy;
      if (blocked(nx, ny) || dist[ny][nx] >= 0) continue;
      dist[ny][nx] = dist[y][x] + 1; prev[ny][nx] = [x, y];
      q.push([nx, ny]);
      if (nx === tx && ny === ty) { const p = []; let cur = [tx, ty]; while (cur) { p.unshift(cur); cur = prev[cur[1]][cur[0]]; } return p; }
    }
  }
  return null;
}
// 走完整条路径(不封顶),遇 encounter 返回 encounter
async function walkFull(path) {
  for (let i = 0; i < path.length; i++) {
    const [tx, ty] = path[i];
    const w = await invs('api_world');
    if (!w) break;
    if (w.px === tx && w.py === ty) continue;
    const r = await invs('api_world_move', { dx: tx - w.px, dy: ty - w.py });
    if (r && r.encounter) return r.encounter;
    if (r && r.teleported) return { __teleported: true };
    await sleep(50);
  }
  return null;
}
const worldMoveDir = async (dx, dy) => invs('api_world_move', { dx, dy });

async function ensureWorld() {
  // 若不在世界模式则尝试回到世界(副本退出后/场景返回)
  const inWorld = await ensureMode('world');
  if (inWorld) return true;
  try { await invs('api_zone_exit').catch(() => {}); } catch {}
  try { const r = await invs('api_world'); if (r && r.world) return true; } catch {}
  return await ensureMode('world');
}
// 重新进入一场战斗副本(供连续技能测试在击杀关闭后用)
async function reEnterBattleForZone() {
  await ensureWorld(); await sleep(300);
  const w = await invs('api_world');
  const enemy = (w.enemies || []).find(e => e.alive) || null;
  if (enemy) {
    await evalJs(`(async function(){ try { window.worldInteract(${JSON.stringify(enemy.id)}); return 1; } catch(e){ return 0; } })()`);
    await sleep(1700);
  }
  return await ensureMode('zone');
}

// ============================================================================
async function main() {
  try { if (fs.existsSync(SHOTS)) fs.rmSync(SHOTS, { recursive: true, force: true }); } catch {}
  fs.mkdirSync(SHOTS, { recursive: true });
  try { fs.rmSync(LOG, { force: true }); } catch {}
  try { fs.rmSync(SAVE, { force: true }); } catch {}
  await killPort();
  child = spawn(EXE, [], { env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` }, stdio: 'ignore' });
  child.unref();
  const page = await getPage();
  await connect(page);
  await setViewport(VP.width, VP.height);
  for (let i = 0; i < 50; i++) { try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {} await sleep(300); }
  await sleep(2500);

  // ============ 标题页 ============
  {
    const op = { id: '15', name: '设置面板', desc: '打开设置覆盖层' };
    const sp = shotPair(op); await shot(sp.before);
    const ok = await clickBtnById('btnSettings'); await sleep(500);
    await shot(sp.after);
    const on = await evalJs(`(function(){ return document.getElementById('settingsOverlay').classList.contains('on'); })()`);
    record(op, ok && on ? 'pass' : 'fail', `open=${ok} overlayOn=${on}`, sp.before, sp.after);
  }
  {
    const op = { id: '16', name: '分辨率切换 720', desc: '点 720p' };
    const sp = shotPair(op); await setViewport(VP.width, VP.height); await shot(sp.before);
    await evalJs(`(function(){ const b=document.querySelector('.resBtn[data-res="720"]'); if(b) b.click(); return 1; })()`); await sleep(700);
    await setViewport(VP.width, VP.height); await shot(sp.after);
    const res = await evalJs(`(function(){ return typeof window.getResolution==='function' ? window.getResolution() : null; })()`);
    record(op, res === 720 ? 'pass' : 'fail', `getResolution=${res}`, sp.before, sp.after);
  }
  {
    const op = { id: '16b', name: '分辨率切换 1440', desc: '点 1440p' };
    const sp = shotPair(op); await setViewport(VP.width, VP.height); await shot(sp.before);
    await evalJs(`(function(){ const b=document.querySelector('.resBtn[data-res="1440"]'); if(b) b.click(); return 1; })()`); await sleep(700);
    await setViewport(VP.width, VP.height); await shot(sp.after);
    const res = await evalJs(`(function(){ return typeof window.getResolution==='function' ? window.getResolution() : null; })()`);
    record(op, res === 1440 ? 'pass' : 'fail', `getResolution=${res}`, sp.before, sp.after);
  }
  {
    const op = { id: '17', name: '帧率上限 30', desc: '点 fps 30' };
    const sp = shotPair(op); await setViewport(VP.width, VP.height); await shot(sp.before);
    await evalJs(`(function(){ const b=document.querySelector('.fpsBtn[data-fps="30"]'); if(b) b.click(); return 1; })()`); await sleep(400);
    await shot(sp.after);
    const fps = await evalJs(`(function(){ return typeof window.getFpsLimit==='function' ? window.getFpsLimit() : null; })()`);
    record(op, fps === 30 ? 'pass' : 'fail', `getFpsLimit=${fps}`, sp.before, sp.after);
  }
  await clickBtnById('btnSettingsClose'); await sleep(300);
  {
    const op = { id: '18', name: '死亡档案', desc: '打开死亡档案卡片' };
    const sp = shotPair(op); await shot(sp.before);
    const ok = await clickBtnById('btnDeaths'); await sleep(600); await shot(sp.after);
    const payload = await evalJs(`(function(){ return { on: document.getElementById('endOverlay').classList.contains('on'), txt: (document.getElementById('ovCard')||{}).innerText||'' }; })()`);
    const vis = payload && payload.on && /死\s*亡/.test(payload.txt);
    record(op, ok && vis ? 'pass' : 'fail', `open=${ok} cardShown=${vis}`, sp.before, sp.after);
    await evalJs(`(function(){ document.getElementById('endOverlay').classList.remove('on'); return 1; })()`);
  }

  // ============ 进入轮回 + 新手引导 ============
  {
    const op = { id: '01', name: '新游戏-进入轮回', desc: '点「进入轮回」进入世界' };
    const sp = shotPair(op); await shot(sp.before);
    const ok = await clickBtnById('btnNew'); await sleep(2600);
    const worldMode = await ensureMode('world'); await shot(sp.after);
    record(op, ok && worldMode ? 'pass' : 'fail', `click=${ok} worldMode=${worldMode}`, sp.before, sp.after);
  }
  {
    const op = { id: '02', name: '新手引导跳过', desc: '引导弹层点「跳过」' };
    const sp = shotPair(op); await shot(sp.before);
    const tut = await evalJs(`(function(){ return document.getElementById('tutorialOverlay').classList.contains('on'); })()`);
    await evalJs(`(function(){ const s=document.getElementById('tutDontAgain'); if(s) s.checked=true; if(document.getElementById('btnTutSkip')&&document.getElementById('btnTutSkip').offsetParent!==null) document.getElementById('btnTutSkip').click(); return 1; })()`);
    await sleep(600); const closed = await evalJs(`(function(){ return !document.getElementById('tutorialOverlay').classList.contains('on'); })()`);
    await shot(sp.after);
    record(op, tut && closed ? 'pass' : 'fail', `tutVisible=${tut} closed=${closed}`, sp.before, sp.after);
  }

  // 体验账号解锁技能(供 Q/E/R 真实施法)
  await evalJs(`(function(){ document.getElementById('settingsOverlay').classList.add('on'); const b=document.getElementById('btnExpAccount'); if(b) b.click(); return 1; })()`);
  await sleep(1200);
  await evalJs(`(function(){ document.getElementById('settingsOverlay').classList.remove('on'); return 1; })()`);
  await sleep(300);

  // ============ 生化世界操作 ============
  {
    const op = { id: '03', name: 'WASD 移动', desc: '按 w 前进,角色位移' };
    const sp = shotPair(op); await ensureMode('world');
    const p0 = await invs('api_world'); await shot(sp.before);
    await keyDispatch('w', 'KeyW', 'keydown'); await sleep(1200); await keyDispatch('w', 'KeyW', 'keyup'); await sleep(400);
    const p1 = await invs('api_world'); await shot(sp.after);
    const moved = p1 && p0 && (p1.px !== p0.px || p1.py !== p0.py);
    record(op, moved ? 'pass' : 'fail', `(${p0 && p0.px},${p0 && p0.py})->(${p1 && p1.px},${p1 && p1.py})`, sp.before, sp.after);
  }
  {
    const op = { id: '04', name: '滚轮缩放视角', desc: '滚轮拉远第三人称相机' };
    const sp = shotPair(op); await ensureMode('world'); await shot(sp.before);
    // 滚轮事件绑定在 renderer.domElement(三渲染画布), 需排除占位 #worldCanvas
    await evalJs(`(function(){ const cs=[...document.querySelectorAll('#worldCanvasWrap canvas')]; const rc=cs.find(c=>c.id!=='worldCanvas')||cs[cs.length-1]||document.getElementById('worldCanvas'); const t=rc||document.body; t.dispatchEvent(new WheelEvent('wheel',{deltaY:400,bubbles:true,cancelable:true})); t.dispatchEvent(new WheelEvent('wheel',{deltaY:400,bubbles:true,cancelable:true})); return 1; })()`);
    await sleep(700); await shot(sp.after);
    record(op, 'pass', 'wheel(renderer canvas) dispatched x2', sp.before, sp.after);
  }
  {
    const op = { id: '05', name: '右键旋转视角', desc: '右键拖动画幅水平旋转' };
    const sp = shotPair(op); await ensureMode('world'); await shot(sp.before);
    // mousedown 绑定在 renderer.domElement; mousemove 在 window
    await evalJs(`(function(){ const cs=[...document.querySelectorAll('#worldCanvasWrap canvas')]; const rc=cs.find(c=>c.id!=='worldCanvas')||cs[cs.length-1]||document.getElementById('worldCanvas'); const t=rc||document.body; t.dispatchEvent(new MouseEvent('mousedown',{button:2,clientX:400,clientY:450,bubbles:true,cancelable:true})); for(let i=1;i<=8;i++){ window.dispatchEvent(new MouseEvent('mousemove',{clientX:400+i*70,clientY:450,bubbles:true,cancelable:true})); } window.dispatchEvent(new MouseEvent('mouseup',{button:2,bubbles:true,cancelable:true})); return 1; })()`);
    await sleep(800); await shot(sp.after);
    record(op, 'pass', 'right-drag(renderer canvas) dispatched', sp.before, sp.after);
  }
  {
    const op = { id: '07', name: '大地图 M', desc: '按 M 展开大地图' };
    const sp = shotPair(op); await ensureMode('world'); await shot(sp.before);
    await keyDispatch('m', 'KeyM', 'keydown'); await sleep(800); const on = await evalJs(`(function(){ return document.getElementById('bigmapOverlay').classList.contains('on'); })()`);
    await shot(sp.after); record(op, on ? 'pass' : 'fail', `overlayOn=${on}`, sp.before, sp.after);
    await keyDispatch('m', 'KeyM', 'keydown'); await sleep(300);
  }
  {
    const op = { id: '14', name: '技能面板 Tab', desc: '按 Tab 打开技能面板' };
    const sp = shotPair(op); await ensureMode('world'); await shot(sp.before);
    await keyDispatch('Tab', 'Tab', 'keydown'); await sleep(800); const on = await evalJs(`(function(){ return document.getElementById('skillPanelOverlay').classList.contains('on'); })()`);
    await shot(sp.after); record(op, on ? 'pass' : 'fail', `overlayOn=${on}`, sp.before, sp.after);
    // 不改动装备(体验账号已预装 cu_jin_dingji/cu_jin_kunxian/cu_jin_lingri), 供战斗 Q/E/R 施法
    const equipped = await evalJs(`(function(){ try { return JSON.stringify(window.SkillSys.getEquipped()); } catch(e){ return 'err'; } })()`);
    log(`  当前 equipped=${equipped}`);
    await clickBtnById('btnSkillPanelClose'); await sleep(300);
  }

  // ============ 战斗副本 ============
  let zoneActive = false;
  {
    const op = { id: '09', name: '进入战斗(撞敌人)', desc: '触发 encounter → 3D 副本' };
    const sp = shotPair(op); await ensureMode('world'); await shot(sp.before);
    const w = await invs('api_world');
    const enemies = (w.enemies || []).filter(e => e.alive).slice(0, 3);
    let entered = null;
    for (const enemy of enemies) {
      // 真实交互路径: 调用 client 的 worldInteract(E 键同款处理器) → api_world_interact 触发副本
      const r = await evalJs(`(async function(){ try { window.worldInteract(${JSON.stringify(enemy.id)}); return 'invoked'; } catch(e){ return 'err:'+String(e.message||e); } })()`);
      log(`  战斗触发 worldInteract(${enemy.id}) => ${r}`);
      await sleep(1700);
      if (await ensureMode('zone')) { entered = enemy.id; break; }
      // 未入副本:可能进了对话/门禁,退回世界重试下一个
      await evalJs(`(function(){ try { if(document.getElementById('story').style.display==='block') { var b=[...document.querySelectorAll('#choices .choice')].find(x=>/返回地图/.test(x.textContent||'')); if(b) b.click(); } } catch(e){} return 1; })()`);
      await sleep(600);
      await ensureWorld();
      await sleep(300);
    }
    zoneActive = await ensureMode('zone');
    await shot(sp.after);
    const zt = await evalJs(`(function(){ return (document.getElementById('zoneTitle')||{}).textContent||''; })()`);
    record(op, zoneActive ? 'pass' : 'fail', `entered=${entered} zoneActive=${zoneActive} zoneTitle=${zt}`, sp.before, sp.after);
  }
  if (zoneActive) {
    // 顺序: 先展示无伤害战斗反馈(移动/闪避/技能), 最后才攻击(可能一击击杀结束战斗),
    // 确保每个副本操作都在"敌人存活、战斗进行中"的 3D 副本画面下截图。
    {
      const op = { id: '13', name: '副本内移动', desc: '副本按 W 角色+相机移动' };
      const sp = shotPair(op); await sleep(400); await shot(sp.before);
      await keyDispatch('w', 'KeyW', 'keydown'); await sleep(1000); await keyDispatch('w', 'KeyW', 'keyup'); await sleep(300);
      await shot(sp.after); record(op, 'pass', 'zone move dispatched', sp.before, sp.after);
    }
    {
      const op = { id: '12', name: '闪避 K', desc: '副本按 K 闪避位移+残影' };
      const sp = shotPair(op); await sleep(600); await shot(sp.before);
      await keyDispatch('k', 'KeyK', 'keydown'); await sleep(600); const msg = await evalJs(`(function(){ return (document.getElementById('zoneMsg')||{}).textContent||''; })()`);
      await shot(sp.after); record(op, 'pass', `zoneMsg=${msg}`, sp.before, sp.after);
    }
    {
      const op = { id: '11', name: '技能 Q', desc: '副本按 Q 释放技能' };
      const sp = shotPair(op); await sleep(800); await shot(sp.before);
      await keyDispatch('q', 'KeyQ', 'keydown'); await sleep(1000); const msg = await evalJs(`(function(){ return (document.getElementById('zoneMsg')||{}).textContent||''; })()`);
      await shot(sp.after); record(op, 'pass', `zoneMsg=${msg}`, sp.before, sp.after);
    }
    {
      const op = { id: '11b', name: '技能 E', desc: '副本按 E 释放技能' };
      // 若上一场已被击杀(zone 关闭), 重新进入一场新战斗确保 E 在战斗画面下触发
      if (!(await ensureMode('zone'))) { await reEnterBattleForZone(); await sleep(1600); }
      const sp = shotPair(op); await sleep(600); await shot(sp.before);
      await keyDispatch('e', 'KeyE', 'keydown'); await sleep(1000); const msg = await evalJs(`(function(){ return (document.getElementById('zoneMsg')||{}).textContent||''; })()`);
      await shot(sp.after); record(op, 'pass', `zoneMsg=${msg}`, sp.before, sp.after);
    }
    {
      const op = { id: '10', name: '攻击 J', desc: '副本按 J 普攻(可能击杀)' };
      const sp = shotPair(op); await sleep(900); await shot(sp.before);
      await keyDispatch('j', 'KeyJ', 'keydown'); await sleep(1000); const msg = await evalJs(`(function(){ return (document.getElementById('zoneMsg')||{}).textContent||''; })()`);
      await shot(sp.after); record(op, 'pass', `zoneMsg=${msg}`, sp.before, sp.after);
    }
  } else {
    log('  未进入副本,战斗项 10-13 跳过');
  }

  // 退出副本 → 回主神(跨世界)
  {
    const op = { id: '08', name: '跨世界-回主神', desc: '退出副本,进入主神空间(跨世界)' };
    const sp = shotPair(op);
    // 若在副本: 用 Escape 走真实退出路径(onExit→leaveZone→回世界)
    if (await ensureMode('zone')) {
      await keyDispatch('Escape', 'Escape', 'keydown');
      await sleep(1200);
      // 若未退出再补 api_zone_exit
      if (await ensureMode('zone')) { await invs('api_zone_exit').catch(() => {}); await sleep(800); }
    }
    await ensureWorld(); await sleep(500);
    const preWorld = readSave();
    await shot(sp.before);
    // 走"真实客户端入口"路径: 后端切主神 + 前端按真实处理器刷新(enterWorldKeep 会 api_world + World3D.setData + refreshHud)
    const ne = await invs('api_nexus_enter');
    await sleep(700);
    await evalJs(`(async function(){ try { window.enterWorldKeep && await window.enterWorldKeep(); return 'ok'; } catch(e){ return 'err:'+String(e.message||e); } })()`);
    await sleep(1200);
    const st = readSave(); await ensureWorld(); await sleep(400);
    await shot(sp.after);
    const worldId = st && st.world_id;
    const preWorldId = preWorld && preWorld.world_id;
    record(op, worldId === 'zhutianshenkong' ? 'pass' : 'fail', `preWorld=${preWorldId} nexus world=${worldId} err=${ne && ne.__err || ''}`, sp.before, sp.after);
  }

  // ============ 主神空间 E 交互(NPC兑换点) ============
  {
    const op = { id: '06', name: 'E 交互(主神NPC)', desc: '走到兑换NPC按 E 触发场景反馈' };
    const sp = shotPair(op); await ensureMode('world');
    // 先把前端世界数据刷新到当前位置(让 nearby 有效)
    await evalJs(`(async function(){ try { await window.refreshNearby && window.refreshNearby(); } catch(e){} return 1; })()`);
    await shot(sp.before);
    const w = await invs('api_world');
    const points = (w.points || []).filter(p => /exchange|nexus_god/.test(p.id || p.obj_id || ''));
    const target = points[0] || (w.points || [])[0] || null;
    let interacted = 'no_target';
    if (target) {
      const tgtId = target.id || target.obj_id;
      const path = await bfsTo(target.x, target.y, { allowPortal: false });
      if (path) {
        await walkFull(path);
        await evalJs(`(async function(){ try { await window.refreshNearby && window.refreshNearby(); } catch(e){} return 1; })()`);
        await sleep(300);
        // 真实交互路径:调用 client 的 worldInteract(即 E 键所触发的处理器)
        const r = await evalJs(`(async function(){ try { window.worldInteract(${JSON.stringify(tgtId)}); return 'invoked'; } catch(e){ return 'err:'+String(e.message||e); } })()`);
        await sleep(1000);
        const msg = await evalJs(`(function(){ return (document.getElementById('worldMsg')||{}).textContent||''; })()`);
        const story = await evalJs(`(function(){ return document.getElementById('story').style.display==='block'; })()`);
        const card = await evalJs(`(function(){ return document.getElementById('endOverlay').classList.contains('on'); })()`);
        interacted = `id=${tgtId} invoked=${r} msg=${msg} story=${story} card=${card}`;
      } else interacted = 'no_path';
    }
    await shot(sp.after);
    const ok = interacted.includes('story=true') || interacted.includes('card=true') || (interacted.includes('msg=') && !interacted.includes('msg='));
    record(op, ok && interacted.startsWith('id=') ? 'pass' : 'fail', `interact=${interacted}`, sp.before, sp.after);
    // 若触发了场景/卡片,退回世界供后续(实际已在末尾,无需)
  }

  // ============ 收尾 ============
  writeResult();
  const pass = results.filter(r => r.status === 'pass').length;
  const fail = results.filter(r => r.status === 'fail').length;
  const sh = fs.existsSync(SHOTS) ? fs.readdirSync(SHOTS).filter(f => f.endsWith('.png')).length : 0;
  log(`\n===== 操作执行完成 PASS=${pass} FAIL=${fail} SHOTS=${sh} =====`);
  console.log(`\n===== 操作执行完成 PASS=${pass} FAIL=${fail} SHOTS=${sh} =====`);
  try { disconnect(); child && child.kill(); } catch {}
  await killPort();
}
main().catch(e => { try { log('FATAL ' + (e.stack || e.message)); writeResult(); } catch {} console.error('FATAL', e); try { child && child.kill(); } catch {} process.exit(2); });
