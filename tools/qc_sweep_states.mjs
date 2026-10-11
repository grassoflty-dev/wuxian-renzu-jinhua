// qc_sweep_states.mjs — 全游戏多状态截图（标题/世界/战斗/背包/技能面板/大地图），供视觉QC
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const EXE = path.resolve(__dirname, '..', 'server-rs', 'target', 'release', 'wuxian-horror-ch1.exe');
const OUT = path.join(__dirname, 'shots', 'qc_states');
fs.mkdirSync(OUT, { recursive: true });
const PORT = 9702;
const sleep = ms => new Promise(r => setTimeout(r, ms));
let ws, nextId = 1;
const errors = [];
async function killPort() { try { execSync(`taskkill /IM wuxian-horror-ch1.exe /T /F 2>nul`, { stdio: 'pipe' }); } catch {} await sleep(900); }
async function getPage() { for (let i = 0; i < 150; i++) { try { const r = await fetch(`http://127.0.0.1:${PORT}/json/list`); const list = await r.json(); const p = list.find(x => x.type === 'page'); if (p) return p; } catch {} await sleep(500); } throw new Error('page not found'); }
async function connect(page) { ws = new WebSocket(page.webSocketDebuggerUrl); await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; }); await cdp('Runtime.enable'); ws.addEventListener('message', ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.method === 'Runtime.exceptionThrown') { const d = m.params.exceptionDetails; errors.push('EXC: ' + (d.exception ? d.exception.description || d.exception.value : d.text)); } else if (m.method === 'Runtime.consoleAPICalled' && m.params.type === 'error') { errors.push('CONSOLE: ' + (m.params.args || []).map(a => a.value || a.description || '').join(' ')); } }); }
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) { return new Promise((resolve, reject) => { const id = nextId++; const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 30000); const onMsg = ev => { let m; try { m = JSON.parse(ev.data); } catch { return; } if (m.id !== id) return; clearTimeout(timer); ws.removeEventListener('message', onMsg); m.error ? reject(new Error(JSON.stringify(m.error))) : resolve(m.result); }; ws.addEventListener('message', onMsg); send({ id, method, params }); }); }
async function evalJs(expression) { const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true }); if (r.exceptionDetails) return 'EXC: ' + JSON.stringify(r.exceptionDetails).slice(0, 300); return r.result?.value; }
async function shot(tag) { await sleep(600); const r = await cdp('Page.captureScreenshot', { format: 'png' }); const out = path.join(OUT, tag + '.png'); fs.writeFileSync(out, Buffer.from(r.data, 'base64')); console.log('SHOT', tag, fs.statSync(out).size); return out; }
async function key(k, down = true) { await evalJs(`window.dispatchEvent(new KeyboardEvent('${down ? 'keydown' : 'keyup'}', { key: '${k}', bubbles: true })); true`); }

(async () => {
  await killPort();
  const child = spawn(EXE, [], { env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}` }, stdio: 'ignore' });
  child.unref();
  await sleep(4000);
  const page = await getPage(); await connect(page);
  for (let i = 0; i < 40; i++) { try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {} await sleep(300); }
  await sleep(1800);
  await shot('0_title');   // 标题

  await evalJs(`(() => { const b=document.getElementById('btnNew'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(2200);
  await evalJs(`(() => { const b=document.getElementById('btnTutSkip')||document.getElementById('btnTutStart'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(2500);
  await shot('1_world');   // 世界地图

  // 走动几步 + 大地图
  for (let i = 0; i < 3; i++) { await key('d', true); await sleep(300); await key('d', false); await sleep(150); }
  await sleep(400);
  await shot('2_world_walk');
  await key('m'); await sleep(900); await shot('3_bigmap'); await key('m'); await sleep(400);

  // 打开技能面板
  await evalJs(`(() => { const b=document.getElementById('btnSkillPanel'); if(b){b.click(); return 1;} return 0; })()`);
  await sleep(700); await shot('4_skillpanel'); 
  await evalJs(`(() => { const b=document.getElementById('skillPanelClose'); if(b){b.click(); return 1;} else { const o=document.getElementById('skillPanelOverlay'); if(o) o.classList.remove('on'); } return 1; })()`);
  await sleep(400);

  // 进战斗
  await evalJs(`(async () => { const w=await window.__TAURI__.core.invoke('api_world'); const en=(w.enemies||[]).find(e=>e.alive); if(!en) return 'no-enemy'; const r=await window.__TAURI__.core.invoke('api_world_interact',{objId:en.id}); await window.enterZone({id:r.zone.id,kind:r.zone.kind,ref:r.zone.ref,name:en.name},r.enemy); return 'ok'; })()`);
  await sleep(2800);
  await shot('5_fight');
  // 战斗中走+攻击
  await key('d', true); await sleep(300); await key('d', false); await sleep(200);
  await key('j', true); await sleep(120); await key('j', false); await sleep(400);
  await shot('6_fight_attack');

  console.log('JSERRORS', JSON.stringify(errors));
  try { ws.close(); } catch {} try { child.kill(); } catch {}
  console.log('DONE');
  process.exit(0);
})().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
