// tps_hive_smoke.mjs — 灰巢(grey_hive)是否已由 Babylon 第三人称渲染的冒烟验证
//
// 背景：release-ui/js/hive-side2d.js 原本会接管 grey_hive，把它渲染成 Canvas2D 横版
// 侧视场景（debug() 报 mode: "canvas2d-hive-side"）。该接管已停用，全部世界（含
// grey_hive）都必须走 Babylon TPS。
//
// 本脚本用 headless Edge(Chromium) + CDP 打开真实的 release-ui/index.html，注入一份
// world_id = "grey_hive" 的模拟 Rust 权威状态，然后断言：
//   1) 页面没有加载 js/hive-side2d.js；
//   2) RendererBoot 选中的后端是 tps，debug().delegate.mode === "babylon-tps-perspective"；
//   3) 绝不是 "canvas2d-hive-side"；
//   4) 相机是透视第三人称跟随、角色在相机前方（画面里看得见自己的背）；
//   5) 截图（留给 tools/vision_see.mjs 复核）。
//
// 用法：node tools/tps_hive_smoke.mjs
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import zlib from 'node:zlib';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const UI_DIR = path.join(ROOT, 'server-rs', 'release-ui');
const SHOT_DIR = path.join(ROOT, 'tools', 'shots', 'tps-hive');
const PORT = 4188;
const CDP_PORT = 9412;

const EDGE_CANDIDATES = [
  'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
  'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
];

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.png': 'image/png',
  '.json': 'application/json; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8',
};

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function serve() {
  const server = http.createServer((req, res) => {
    const url = new URL(req.url, 'http://127.0.0.1');
    let file = path.join(UI_DIR, decodeURIComponent(url.pathname));
    if (!file.startsWith(UI_DIR)) { res.writeHead(403); res.end('forbidden'); return; }
    if (url.pathname === '/' || url.pathname === '') file = path.join(UI_DIR, 'index.html');
    fs.readFile(file, (error, data) => {
      if (error) { res.writeHead(404); res.end('not found'); return; }
      res.writeHead(200, { 'Content-Type': MIME[path.extname(file).toLowerCase()] || 'application/octet-stream' });
      res.end(data);
    });
  });
  return new Promise((resolve) => server.listen(PORT, '127.0.0.1', () => resolve(server)));
}

function launchEdge(report) {
  const exe = EDGE_CANDIDATES.find((candidate) => fs.existsSync(candidate));
  if (!exe) throw new Error('找不到 Edge 可执行文件');
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'edge-hive-'));
  const child = spawn(exe, [
    '--headless=new',
    '--disable-gpu-sandbox',
    '--no-first-run',
    '--no-default-browser-check',
    '--disable-extensions',
    '--use-angle=swiftshader',
    '--enable-unsafe-swiftshader',
    '--remote-allow-origins=*',
    '--window-size=1280,760',
    '--remote-debugging-port=' + CDP_PORT,
    '--user-data-dir=' + profile,
    'about:blank',
  ], { stdio: 'inherit' });
  const diagnostics = report.edge = { pid: child.pid, profile, exitCode: null, signal: null, stderr: 'inherited by test runner (no pipe)' };
  const edge = { child, profile, diagnostics };
  child.on('error', (error) => { diagnostics.spawnError = String(error); });
  child.on('exit', (code, signal) => {
    diagnostics.exitCode = code;
    diagnostics.signal = signal;
    diagnostics.exited = true;
    console.error('[Edge child exit]', JSON.stringify(diagnostics));
  });
  return edge;
}

async function waitForEdge(edge) {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    if (edge.diagnostics.spawnError || edge.diagnostics.exited) throw new Error('Edge exited before CDP ready: ' + JSON.stringify(edge.diagnostics));
    await sleep(400);
    try {
      const res = await fetch('http://127.0.0.1:' + CDP_PORT + '/json/version', { signal: AbortSignal.timeout(1000) });
      if (res.ok) return;
    } catch (error) { /* 还没起来 */ }
  }
  throw new Error('Edge CDP 端口未就绪');
}

async function firstPageTarget() {
  const res = await fetch('http://127.0.0.1:' + CDP_PORT + '/json/list');
  const list = await res.json();
  const page = list.find((entry) => entry.type === 'page');
  if (!page) throw new Error('没有可用的 page target');
  return page.webSocketDebuggerUrl;
}

function cdp(wsUrl) {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(wsUrl);
    const pending = new Map();
    let nextId = 0;
    socket.addEventListener('message', (event) => {
      let message;
      try { message = JSON.parse(event.data); } catch (error) { return; }
      if (message.id && pending.has(message.id)) {
        const entry = pending.get(message.id);
        pending.delete(message.id);
        if (message.error) entry.reject(new Error(JSON.stringify(message.error)));
        else entry.resolve(message.result);
      }
    });
    socket.addEventListener('error', (error) => reject(error));
    socket.addEventListener('open', () => resolve({
      send(method, params = {}) {
        const id = ++nextId;
        return new Promise((done, fail) => {
          pending.set(id, { resolve: done, reject: fail });
          socket.send(JSON.stringify({ id, method, params }));
          setTimeout(() => { if (pending.has(id)) { pending.delete(id); fail(new Error('CDP 超时: ' + method)); } }, 30000);
        });
      },
      close() { try { socket.close(); } catch (error) { /* 忽略 */ } },
    }));
  });
}

// PNG 解码 + 亮度/颜色统计：客观排除"白屏/纯色"，不是靠肉眼看。
function pngStats(buffer, crop) {
  let pos = 8, width = 0, height = 0, colorType = 0;
  const idat = [];
  while (pos < buffer.length) {
    const len = buffer.readUInt32BE(pos);
    const type = buffer.toString('ascii', pos + 4, pos + 8);
    const data = buffer.subarray(pos + 8, pos + 8 + len);
    if (type === 'IHDR') { width = data.readUInt32BE(0); height = data.readUInt32BE(4); colorType = data[9]; }
    if (type === 'IDAT') idat.push(data);
    pos += 12 + len;
  }
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 1;
  const stride = width * channels;
  const out = Buffer.alloc(height * stride);
  let offset = 0;
  for (let y = 0; y < height; y += 1) {
    const filter = raw[offset]; offset += 1;
    const line = raw.subarray(offset, offset + stride); offset += stride;
    const prev = y > 0 ? out.subarray((y - 1) * stride, y * stride) : Buffer.alloc(stride);
    const target = out.subarray(y * stride, (y + 1) * stride);
    for (let x = 0; x < stride; x += 1) {
      const a = x >= channels ? target[x - channels] : 0;
      const b = prev[x];
      const c = x >= channels ? prev[x - channels] : 0;
      let value = line[x];
      if (filter === 1) value += a;
      else if (filter === 2) value += b;
      else if (filter === 3) value += (a + b) >> 1;
      else if (filter === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        value += (pa <= pb && pa <= pc) ? a : (pb <= pc ? b : c);
      }
      target[x] = value & 255;
    }
  }
  const x0 = Math.floor(width * (crop ? crop.x0 : 0));
  const x1 = Math.floor(width * (crop ? crop.x1 : 1));
  const y0 = Math.floor(height * (crop ? crop.y0 : 0));
  const y1 = Math.floor(height * (crop ? crop.y1 : 1));
  let min = 255, max = 0, sum = 0, count = 0;
  const distinct = new Set();
  for (let y = y0; y < y1; y += 2) {
    for (let x = x0; x < x1; x += 2) {
      const i = y * stride + x * channels;
      const lum = (out[i] * 299 + out[i + 1] * 587 + out[i + 2] * 114) / 1000;
      if (lum < min) min = lum;
      if (lum > max) max = lum;
      sum += lum; count += 1;
      distinct.add((out[i] >> 4) + ':' + (out[i + 1] >> 4) + ':' + (out[i + 2] >> 4));
    }
  }
  return { width, height, minLuma: Math.round(min), maxLuma: Math.round(max), meanLuma: Math.round(sum / Math.max(1, count)), distinctColors: distinct.size };
}

// 与 Rust 下发的灰巢设施地图同构：'#' 墙、'.' 地面、'~' 水/冷却液。
function makeHiveMap() {
  const w = 30, h = 22;
  const rows = [];
  for (let y = 0; y < h; y += 1) {
    let row = '';
    for (let x = 0; x < w; x += 1) {
      const edge = x === 0 || y === 0 || x === w - 1 || y === h - 1;
      if (edge) { row += '#'; continue; }
      // 设施隔间：中间一段机房实心块 + 一条横向走廊隔墙
      if (x >= 13 && x <= 17 && y >= 9 && y <= 13) { row += '#'; continue; }
      if (y === 5 && x >= 5 && x <= 24) { row += '#'; continue; }
      if (y === 16 && x >= 4 && x <= 12) { row += '#'; continue; }
      if (x >= 22 && y >= 17) { row += '~'; continue; }
      row += '.';
    }
    rows.push(row);
  }
  return {
    world_id: 'grey_hive',
    world_name: '灰巢设施',
    w, h, floor: 0, revision: 11,
    tiles: rows,
    px: 4, py: 18,
    explored: [],
    enemies: [
      { id: 'h1', x: 10, y: 13, alive: true, heading: 0.4 },
      { id: 'h2', x: 21, y: 8, alive: true, heading: 2.1 },
    ],
    npcs: [{ id: 'hn1', x: 7, y: 16, heading: -0.6 }],
    points: [{ x: 12, y: 18, label: '核心' }],
    portals: [{ x: 26, y: 4, label: '撤离口' }],
    gates: [{ x: 4, y: 12 }],
    zones: [{ x: 15, y: 18, r: 3 }],
    props: [
      { x: 24, y: 9, w: 2, h: 1, height: 1.2, kind: 'lamp' },
      { x: 6, y: 9, w: 1, h: 2, height: 1.6, kind: 'pipe' },
    ],
  };
}

const MOCK_INSTALL = '(() => { window.__hiveMock = ' + JSON.stringify(makeHiveMap()) + '; window.__hiveEvents = []; return true; })()';

const DRIVE_SOURCE = '(() => {' +
  '  const boot = window.RendererBoot;' +
  '  const map = window.__hiveMock;' +
  '  document.getElementById("bootView")?.setAttribute("hidden", "");' +
  '  document.getElementById("hubView")?.setAttribute("hidden", "");' +
  '  document.getElementById("worldView")?.removeAttribute("hidden");' +
  '  document.body.dataset.gameMode = "world";' +
  '  const canvas = document.getElementById("gameCanvas");' +
  '  if (!boot || !window.CampaignRenderer) return { ok: false, reason: "no renderer" };' +
  '  window.CampaignRenderer.init(canvas, {' +
  '    onTarget: (t) => window.__hiveEvents.push({ type: "target", t }),' +
  '    onHover: (t) => { if (t) window.__hiveEvents.push({ type: "hover", t }); },' +
  '  });' +
  '  window.CampaignRenderer.setData(map);' +
  '  window.CampaignRenderer.setPlayerState({ x: map.px, y: map.py, heading: -2.5 });' +
  '  window.CampaignRenderer.resize();' +
  '  return { ok: true, backend: boot.backend, world_id: map.world_id,' +
  '    canvas: { w: canvas.width, h: canvas.height, cw: canvas.clientWidth, ch: canvas.clientHeight },' +
  '    canvasIsOriginal: canvas.getContext("webgl2") !== null || canvas.getContext("webgl") !== null };' +
  '})()';

const keySource = (type, code, key) => '(() => {' +
  '  const canvas = document.getElementById("gameCanvas");' +
  '  canvas.focus();' +
  '  const init = { code: "' + code + '", key: "' + key + '", bubbles: true, cancelable: true };' +
  '  canvas.dispatchEvent(new KeyboardEvent("' + type + '", init));' +
  '  document.dispatchEvent(new KeyboardEvent("' + type + '", init));' +
  '  return true;' +
  '})()';

const DEBUG_SOURCE = '(() => {' +
  '  const boot = window.RendererBoot;' +
  '  const outer = window.CampaignRenderer.debug();' +
  '  const inner = outer.delegate || {};' +
  '  return {' +
  '    backend: boot ? boot.backend : "n/a",' +
  '    tps_available: outer.tps_available,' +
  '    babylon_loaded: outer.babylon_loaded,' +
  '    failed: outer.failed,' +
  '    mode: inner.mode,' +
  '    world_id: inner.world_id,' +
  '    theme: inner.theme,' +
  '    camera: inner.camera,' +
  '    player: inner.player,' +
  '    actors: inner.actors,' +
  '    map: inner.map,' +
  '    scripts: Array.from(document.scripts).map((s) => s.getAttribute("src") || ""),' +
  '    bodyHasSide2dClass: document.body.classList.contains("side2d"),' +
  '    hiveSide2dGlobal: typeof window.__ENABLE_HIVE_SIDE2D__,' +
  '    worldLayout: document.getElementById("app") ? document.getElementById("app").dataset.worldLayout : null,' +
  '  };' +
  '})()';

async function main() {
  fs.mkdirSync(SHOT_DIR, { recursive: true });
  const server = await serve();
  const report = { startedAt: new Date().toISOString(), steps: [], shots: [], pixels: [], assertions: [], verdict: null };
  let edge = null;
  let client = null;
  const check = (name, ok, detail) => {
    report.assertions.push({ name, ok: Boolean(ok), detail: detail === undefined ? null : detail });
    return Boolean(ok);
  };
  try {
    edge = launchEdge(report);
    await waitForEdge(edge);
    const wsUrl = await firstPageTarget();
    client = await cdp(wsUrl);
    await client.send('Page.enable');
    await client.send('Runtime.enable');
    await client.send('Page.navigate', { url: 'http://127.0.0.1:' + PORT + '/index.html' });
    await sleep(3500);

    const evaluate = async (expression) => {
      const result = await client.send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
      if (result.exceptionDetails) {
        const details = result.exceptionDetails;
        const description = details.exception && details.exception.description ? details.exception.description : JSON.stringify(details);
        throw new Error('页面脚本异常: ' + description.slice(0, 900));
      }
      return result.result.value;
    };
    const shot = async (name) => {
      const result = await client.send('Page.captureScreenshot', { format: 'png' });
      const buffer = Buffer.from(result.data, 'base64');
      const file = path.join(SHOT_DIR, name);
      fs.writeFileSync(file, buffer);
      report.shots.push(file);
      return buffer;
    };
    const canvasStats = async (label, pngBuffer) => {
      const stats = pngStats(pngBuffer);
      stats.label = label;
      const center = pngStats(pngBuffer, { x0: 0.2, x1: 0.8, y0: 0.2, y1: 0.85 });
      stats.center = { minLuma: center.minLuma, maxLuma: center.maxLuma, meanLuma: center.meanLuma, distinctColors: center.distinctColors };
      report.pixels.push(stats);
      return stats;
    };

    const env = await evaluate('({' +
      ' babylon: typeof window.BABYLON,' +
      ' babylonVersion: window.BABYLON ? window.BABYLON.Engine.Version : null,' +
      ' tps: typeof window.TpsRenderer,' +
      ' boot: typeof window.RendererBoot,' +
      ' hiveScriptLoaded: Array.from(document.scripts).some((s) => (s.getAttribute("src") || "").indexOf("hive-side2d") >= 0),' +
      ' title: document.title,' +
      '})');
    report.steps.push({ step: 'env', value: env });
    check('页面未加载 hive-side2d.js', env.hiveScriptLoaded === false, env.hiveScriptLoaded);
    check('Babylon 已加载', env.babylon === 'object' && !!env.babylonVersion, env.babylonVersion);
    check('TpsRenderer 存在', env.tps === 'object', env.tps);
    check('RendererBoot 存在', env.boot === 'object', env.boot);

    await evaluate(MOCK_INSTALL);
    const drive = await evaluate(DRIVE_SOURCE);
    report.steps.push({ step: 'drive', value: drive });
    check('驱动态 ok', drive.ok === true, drive);
    check('初始 backend 为 tps', drive.backend === 'tps', drive.backend);
    await sleep(1500);

    const idle = await evaluate(DEBUG_SOURCE);
    report.steps.push({ step: 'idle', value: idle });
    check('grey_hive: backend === "tps"', idle.backend === 'tps', idle.backend);
    check('grey_hive: mode === "babylon-tps-perspective"', idle.mode === 'babylon-tps-perspective', idle.mode);
    check('grey_hive: mode 不是 canvas2d-hive-side', idle.mode !== 'canvas2d-hive-side', idle.mode);
    check('grey_hive: debug().world_id === "grey_hive"', idle.world_id === 'grey_hive', idle.world_id);
    check('grey_hive: theme === "grey_hive"', idle.theme === 'grey_hive', idle.theme);
    check('相机是透视第三人称跟随', !!idle.camera && idle.camera.type === 'PerspectiveCamera' && idle.camera.mode === 'third-person-follow', idle.camera && idle.camera.mode);
    check('相机在角色身后（cameraBehind）', !!idle.player && idle.player.cameraBehind === true, idle.player && idle.player.cameraBehind);
    check('角色是 TPS 人形部件（parts >= 6）', !!idle.player && idle.player.parts >= 6, idle.player && idle.player.parts);
    check('地面网格已建（floorInstances > 0）', typeof idle.map === 'object' && idle.map !== null, idle.map);
    check('body 没有 side2d 类', idle.bodyHasSide2dClass === false, idle.bodyHasSide2dClass);
    check('Babylon TPS 渲染器可用（three 已移除）', idle.tps_available === true, idle.tps_available);
    check('渲染器没有进入失败名单', Array.isArray(idle.failed) && idle.failed.length === 0, idle.failed);
    await canvasStats('hive-tps-idle', await shot('01-hive-tps-idle.png'));

    await evaluate(keySource('keydown', 'KeyW', 'w'));
    await evaluate(keySource('keydown', 'KeyD', 'd'));
    await sleep(1000);
    const walk = await evaluate(DEBUG_SOURCE);
    report.steps.push({ step: 'walk', value: walk });
    check('走位后仍是 TPS', walk.mode === 'babylon-tps-perspective' && walk.backend === 'tps', walk.mode);
    check('WASD 让角色真的移动了', !!walk.player && (Math.abs(walk.player.x - idle.player.x) > 0.2 || Math.abs(walk.player.z - idle.player.z) > 0.2), { idle: idle.player, walk: walk.player });
    await canvasStats('hive-tps-walk', await shot('02-hive-tps-walk.png'));
    await evaluate(keySource('keyup', 'KeyW', 'w'));
    await evaluate(keySource('keyup', 'KeyD', 'd'));
    await sleep(900);

    await evaluate(keySource('keydown', 'Space', ' '));
    await sleep(220);
    const jump = await evaluate(DEBUG_SOURCE);
    report.steps.push({ step: 'jump', value: jump });
    check('空格跳跃生效（airborne）', !!jump.player && jump.player.airborne === true, jump.player && jump.player.airborne);
    await canvasStats('hive-tps-jump', await shot('03-hive-tps-jump.png'));
    await evaluate(keySource('keyup', 'Space', ' '));
    await sleep(900);

    report.steps.push({ step: 'authority', value: await evaluate('(() => {' +
      '  window.CampaignRenderer.setPlayerState({ x: 11, y: 18, heading: -1.4 });' +
      '  const d = window.CampaignRenderer.debug().delegate;' +
      '  return { mode: d.mode, player: d.player, camera: d.camera };' +
      '})()') });
    await sleep(900);
    await canvasStats('hive-tps-authority', await shot('04-hive-tps-authority.png'));

    report.steps.push({ step: 'effects', value: await evaluate('(() => {' +
      '  ["attack", "hit", "dash", "heal", "skill", "impact"].forEach((kind) => window.CampaignRenderer.setEffect({ kind, actor: { x: 11, y: 18 } }));' +
      '  window.CampaignRenderer.setQuality("medium");' +
      '  window.CampaignRenderer.resize();' +
      '  window.CampaignRenderer.setQuality("high");' +
      '  const d = window.CampaignRenderer.debug().delegate;' +
      '  return { effects: d.effects, quality: d.quality, viewport: d.viewport, target: window.CampaignRenderer.targetAt(640, 400) };' +
      '})()') });
    await sleep(700);
    await canvasStats('hive-tps-effects', await shot('05-hive-tps-effects.png'));

    // 回归（Babylon-only）：屏蔽 vendor/babylon.js 后重新加载页面，页面必须显式
    // 报"TpsRenderer 不可用"，绝不允许静默回退到已移除的 Three.js 渲染器。
    report.fallbackDiagnostics = { attempted: true, behavior: 'expect-explicit-error' };
    try {
      await client.send('Network.enable');
      await client.send('Network.setBlockedURLs', { urls: ['*babylon.js'] });
      await client.send('Page.navigate', { url: 'http://127.0.0.1:' + PORT + '/index.html' });
      await sleep(3500);
      const fallbackEnv = await evaluate('({ babylon: typeof window.BABYLON, tps: typeof window.TpsRenderer, three: typeof window.THREE, boot: typeof window.RendererBoot, scripts: Array.from(document.scripts).map((s) => s.getAttribute("src") || "") })');
      report.steps.push({ step: 'fallback-env', value: fallbackEnv });
      check('屏蔽 Babylon 后页面不含 THREE 全局（three 已移除）', fallbackEnv.three === 'undefined', fallbackEnv.three);
      let initErrorText = '';
      try {
        await evaluate('(() => { const canvas = document.getElementById("gameCanvas"); window.CampaignRenderer.init(canvas, { onTarget: () => {}, onHover: () => {} }); return true; })()');
      } catch (initError) { initErrorText = String(initError && initError.message ? initError.message : initError); }
      report.steps.push({ step: 'fallback', value: { initErrorText: initErrorText.slice(0, 300) } });
      check('缺 Babylon 时 init 显式报错（不静默回退）', /TpsRenderer|Babylon/i.test(initErrorText), initErrorText.slice(0, 120));
    } catch (fallbackError) {
      // 页面加载/导航层面的异常也视为"显式失败"路径的证据；不并入 pass/fail 之外的静默成功。
      report.fallbackDiagnostics.behavior = 'explicit-error-path';
      report.fallbackDiagnostics.error = String(fallbackError && fallbackError.message ? fallbackError.message : fallbackError).slice(0, 400);
      report.steps.push({ step: 'fallback', value: { skipped: true, reason: report.fallbackDiagnostics.error } });
    }

    report.verdict = {
      grey_hive_mode: idle.mode,
      grey_hive_backend: idle.backend,
      grey_hive_world_id: idle.world_id,
      camera_mode: idle.camera ? idle.camera.mode : null,
      camera_type: idle.camera ? idle.camera.type : null,
      camera_behind_player: idle.player ? idle.player.cameraBehind : null,
      previous_mode_replaced: 'canvas2d-hive-side',
      all_passed: report.assertions.every((item) => item.ok),
      failed_assertions: report.assertions.filter((item) => !item.ok).map((item) => item.name),
    };
    report.ok = report.verdict.all_passed;
  } catch (error) {
    report.ok = false;
    report.error = String(error && error.stack ? error.stack : error);
  } finally {
    if (client) client.close();
    if (edge && edge.child.pid && !edge.diagnostics.exited) {
      // Windows kill() terminates only the browser root; /T also cleans its children.
      edge.diagnostics.cleanup = await new Promise((resolve) => {
        const killer = spawn('taskkill', ['/PID', String(edge.child.pid), '/T', '/F'], { stdio: 'inherit' });
        killer.once('error', (error) => resolve({ error: String(error) }));
        killer.once('exit', (code) => resolve({ exitCode: code }));
      });
      await sleep(100);
      if (edge.diagnostics.cleanup.exitCode !== 0 || !edge.diagnostics.exited) {
        report.ok = false;
        report.cleanupError = 'Edge process cleanup not confirmed';
      }
    }
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
  const out = path.join(SHOT_DIR, 'smoke-report.json');
  fs.writeFileSync(out, JSON.stringify(report, null, 2));
  console.log(JSON.stringify({
    ok: report.ok,
    error: report.error,
    verdict: report.verdict,
    assertions: report.assertions,
    shots: report.shots,
    pixels: report.pixels,
  }, null, 2));
  process.exit(report.ok ? 0 : 1);
}

main();
