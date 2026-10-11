// render_voxel_viewer.mjs — 体素素材库全部模型渲染（Edge headless CDP，纯白背景 1024）
//  对每个模型渲染 3 视图: front / orbit45 / side → tools/shots/render_voxel_viewer_<cat>_<name>_<view>.png
// 用法: node render_voxel_viewer.mjs [cat] [name]
//   - 带参数: 仅渲染指定模型（如 node render_voxel_viewer.mjs item item_health）
//   - 无参数: 渲染全部 99 个模型 × 3 视图
// 分类 id: char / item / weapon / gear
import { spawn, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HTML = path.join(__dirname, '..', 'server-rs', 'ui', 'voxel_viewer.html');
const SHOTS = path.join(__dirname, 'shots');
const PORT = 9729;
const VIEWS = ['front', 'orbit45', 'side'];
fs.mkdirSync(SHOTS, { recursive: true });
const sleep = ms => new Promise(r => setTimeout(r, ms));
const log = s => console.log(s);

const argCat = process.argv[2];
const argName = process.argv[3];

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

function cleanLaunch(edge, userDir, port) {
  try {
    const o = execSync(`netstat -ano | findstr :${port}`, { stdio: 'pipe' }).toString();
    for (const line of o.split(/\r?\n/)) {
      const m = line.trim().match(/(\d+)\s*$/);
      if (m) try { execSync(`taskkill /PID ${m[1]} /T /F 2>nul`, { stdio: 'pipe' }); } catch {}
    }
  } catch {}
  const child = spawn(edge, [
    '--headless=new',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${userDir}`,
    '--no-first-run',
    '--disable-gpu',
    '--allow-file-access-from-files',
    `file:///${HTML.replace(/\\/g, '/').replace(/^\/+/, '')}?render=1`,
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
async function connect(page) {
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
}
function send(o) { ws.send(JSON.stringify(o)); }
function cdp(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => reject(new Error('cdp timeout ' + method)), 40000);
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
async function evalJs(expression, timeout = 30000) {
  const r = await cdp('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true, userGesture: true });
  if (r.exceptionDetails) throw new Error('eval exc: ' + JSON.stringify(r.exceptionDetails).slice(0, 500));
  return r.result?.value;
}
async function setViewport(w, h) {
  await cdp('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: 1, mobile: false });
  await evalJs('window.dispatchEvent(new Event("resize")); true');
}
async function waitReady() {
  for (let i = 0; i < 120; i++) {
    await sleep(200);
    if (await evalJs('window.__modelReady === true') === true) return true;
  }
  return false;
}

async function renderOne(cat, name) {
  const tag = cat + '_' + name;
  log('\n=== ' + tag + ' ===');
  const res = await evalJs(
    `window.__setModel(${JSON.stringify(cat)}, ${JSON.stringify(name)})` +
    ` .then(function(s){ return (window.__modelReady===true?'OK':'NOTREADY') + ' | ' + s; })` +
    ` .catch(function(e){ return 'THROW | ' + String(e && e.stack || e); })`
  );
  log('setModel => ' + res);
  const ready = await waitReady();
  await sleep(350);
  const stats = await evalJs('JSON.stringify(window.__modelStats||null)');
  log('ready=' + ready + ' stats=' + stats);
  if (res.indexOf('THROW') >= 0 || res.indexOf('ERR:') >= 0) {
    log('SKIP (build error)');
    return { tag, ok: false, err: res };
  }
  for (const v of VIEWS) {
    log('setModelView ' + v + ' => ' + await evalJs(`window.setModelView('${v}')`));
    await sleep(380);
    const r = await cdp('Page.captureScreenshot', { format: 'png' });
    const out = path.join(SHOTS, `render_voxel_viewer_${cat}_${name}_${v}.png`);
    fs.writeFileSync(out, Buffer.from(r.data, 'base64'));
    log('SHOT ' + path.basename(out) + ' ' + fs.statSync(out).size + 'B');
  }
  return { tag, ok: true };
}

async function main() {
  const edge = findEdge();
  if (!edge) { console.error('找不到 msedge.exe'); process.exit(2); }
  const userDir = path.join(os.tmpdir(), 'render_voxel_viewer_' + process.pid);
  fs.mkdirSync(userDir, { recursive: true });

  const child = cleanLaunch(edge, userDir, PORT);
  let page;
  try { page = await getPage(); } catch (e) { log('FATAL getPage ' + e.message); child.kill(); process.exit(2); }
  await connect(page);
  for (let i = 0; i < 40; i++) {
    try { if (await evalJs('document.readyState==="complete"') === true) break; } catch {}
    await sleep(300);
  }
  await sleep(900);
  log('hasMC=' + await evalJs('!!window.VoxelZhenghaiMC')
    + ' hasBase=' + await evalJs('!!window.VoxelZhenghai')
    + ' hasItem=' + await evalJs('!!window.VoxelItem')
    + ' hasSet=' + await evalJs('typeof window.__setModel'));

  await setViewport(1024, 1024);
  await sleep(500);

  // 待渲染模型清单: 从页面读取分类及清单（保证与页面一致）
  const cats = (await evalJs(`window.Viewer.getCats().map(c => c.id + '|' + c.n).join('\\n')`) || '').trim();
  const catCount = cats ? cats.split('\n').filter(Boolean) : [];
  log('cats=' + catCount.join(' | '));

  // 选定要渲染的分类/名称
  let targets = [];
  if (argCat && argName) {
    targets.push({ cat: argCat, name: argName });
  } else {
    const names = [
      'zz_mc', 'zz_base',
      'item_antidote', 'item_bandage', 'item_bottle_water', 'item_core', 'item_fragment',
      'item_grenade', 'item_health', 'item_holy', 'item_holy_water', 'item_jiezhou_fu',
      'item_medkit', 'item_quzhen_fu', 'item_rune', 'item_sedative', 'item_silver_bullet',
      'item_stone', 'item_torch',
      'item_wp_antimatter_round', 'item_wp_arcan_staff', 'item_wp_beam_saber', 'item_wp_beiming_jian',
      'item_wp_dagou_bang', 'item_wp_dragon_lance', 'item_wp_dugu_jiujian', 'item_wp_electromag_gun',
      'item_wp_excalibur_holy', 'item_wp_fantian_yin', 'item_wp_feijian_qingyun', 'item_wp_gauss_rifle',
      'item_wp_guaili_jian', 'item_wp_laser_sword', 'item_wp_leidun_chui', 'item_wp_liumai_jian',
      'item_wp_lixiao_feidao', 'item_wp_madoushu_grimoire', 'item_wp_mo_jian_zhl', 'item_wp_nano_blade',
      'item_wp_orbital_gun', 'item_wp_pangu_fu', 'item_wp_particle_cannon', 'item_wp_plasma_dagger',
      'item_wp_qianbenying', 'item_wp_ruyibang', 'item_wp_shanhe_shetu', 'item_wp_sheng_jian_mj',
      'item_wp_shuang_zhi_aisang', 'item_wp_taiji_tu', 'item_wp_tulong_dao', 'item_wp_wang_zhicai',
      'item_wp_xianzhe_zhi_shi', 'item_wp_xuantie_jian', 'item_wp_xuanyuan_jian', 'item_wp_yitian_jian',
      'item_wp_zanjingdao_he', 'item_wp_zanyue', 'item_wp_zhanxian_feidao', 'item_wp_zhuxian_sijian',
      'gear_access_hades_cloak', 'gear_access_will_anchor', 'gear_gear_adamant_cuirass',
      'gear_gear_holy_plate_armor', 'gear_gear_ice_dragon_scale', 'gear_gear_leidun_armor',
      'gear_gear_longlin_jia', 'gear_gear_nano_mecha_suit', 'gear_gear_sanctum_plate',
      'gear_gear_shadow_cloak_armor', 'gear_gear_shengclothes_shooter', 'gear_gear_shengguang_fapao',
      'gear_gear_tian_yi', 'gear_gear_wh_warframe', 'gear_gear_zero_absorb',
      'tr_tr_bahuang_longyin', 'tr_tr_blood_banner', 'tr_tr_danxin_mirror', 'tr_tr_duantou_mojing',
      'tr_tr_leishen_xianglu', 'tr_tr_longzu_shengyi', 'tr_tr_mo_jie_jiujie', 'tr_tr_shengbei_shengtian',
      'tr_tr_shenlei_pendant', 'tr_tr_sisin_luandao', 'tr_tr_taixu_shield', 'tr_tr_undo_pillowstone',
      'tr_tr_xianzhe_ziliao', 'tr_tr_yinyang_jing', 'tr_tr_zhuxian_calendar',
      'bl_demon_bloodline', 'bl_hollow_bloodline', 'bl_mitsurugi_bloodline', 'bl_otsutsuki_bloodline',
      'bl_quincy_bloodline', 'bl_saint_bloodline', 'bl_saiyan_bloodline', 'bl_sharingan_bloodline',
      'bl_shinigami_bloodline', 'bl_uchiha_bloodline'
    ];
    names.forEach(function (name) {
      let cat;
      if (name === 'zz_mc' || name === 'zz_base') cat = 'char';
      else if (name.indexOf('item_wp_') === 0) cat = 'weapon';
      else if (name.indexOf('item_') === 0) cat = 'item';
      else cat = 'gear';
      targets.push({ cat: cat, name: name });
    });
  }
  log('total targets=' + targets.length);

  const ok = [], fail = [];
  for (const t of targets) {
    try { const r = await renderOne(t.cat, t.name); (r.ok ? ok : fail).push(r.tag); }
    catch (e) { log('EXC ' + t.cat + '/' + t.name + ' ' + (e.message || e)); fail.push(t.cat + '_' + t.name); }
  }

  try { ws.close(); } catch {}
  try { child.kill(); } catch {}
  log('\n==== DONE ====');
  log('OK ' + ok.length + ' models / ' + (ok.length * VIEWS.length) + ' shots');
  log('FAIL ' + fail.length + ': ' + fail.join(', '));
  process.exit(0);
}
main().catch(e => { console.error('FATAL', e.stack || e.message); process.exit(2); });
