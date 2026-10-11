// tps_renderer_smoke.mjs — Babylon 第三人称渲染器冒烟验证
//
// 用 headless Edge(Chromium) 通过 CDP 打开真实的 release-ui/index.html，
// 注入一份模拟的 Rust 权威状态，驱动 TPS 渲染器并截图证明：
//   1) 第三人称透视视角成立、相机在角色身后上方；
//   2) 角色可见（低多边形人形）；
//   3) WASD 相机相对移动 + 空格跳跃 + 角色朝向转向生效；
//   4) 正常路径走 Babylon TPS，禁用 Babylon 时自动回退 Three.js。
//
// 用法：node tools/tps_renderer_smoke.mjs
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
const SHOT_DIR = path.join(ROOT, 'tools', 'shots', 'tps-renderer');
const PORT = 4187;
const CDP_PORT = 9411;

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
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'edge-tps-'));
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

// PNG 解码 + 亮度/颜色统计，用来客观判断"是不是白屏/黑屏"。
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

// 与 Rust map 下发同构的测试地图：'#' 墙、'.' 地面、'~' 水
function makeMap() {
  const w = 26, h = 20;
  const rows = [];
  for (let y = 0; y < h; y += 1) {
    let row = '';
    for (let x = 0; x < w; x += 1) {
      const edge = x === 0 || y === 0 || x === w - 1 || y === h - 1;
      if (edge) { row += '#'; continue; }
      if (x >= 12 && x <= 15 && y >= 8 && y <= 11) { row += '#'; continue; }
      if (y === 4 && x >= 6 && x <= 18) { row += '#'; continue; }
      if (x >= 19 && y >= 14) { row += '~'; continue; }
      row += '.';
    }
    rows.push(row);
  }
  // 注意：release-ui/js/hive-side2d.js 对 grey_hive 世界有横版 2D 专用表现层，
  // 会接管 CampaignRenderer。TPS 渲染器覆盖其余全部世界与 hub，因此这里用 hub。
  return {
    world_id: 'hub',
    world_name: 'TPS 冒烟现场',
    w, h, floor: 0, revision: 7,
    tiles: rows,
    px: 3, py: 15,
    explored: [],
    enemies: [
      { id: 'e1', x: 8, y: 11, alive: true, heading: 0.4 },
      { id: 'e2', x: 18, y: 6, alive: true, heading: 2.1 },
    ],
    npcs: [{ id: 'n1', x: 6, y: 14, heading: -0.6 }],
    points: [{ x: 10, y: 15, label: '目标' }],
    portals: [{ x: 22, y: 3, label: '出口' }],
    gates: [{ x: 3, y: 10 }],
    zones: [{ x: 14, y: 16, r: 3 }],
    props: [{ x: 20, y: 8, w: 2, h: 1, height: 1.2 }, { x: 5, y: 8, w: 1, h: 2, height: 1.6 }],
  };
}

const DRIVE_SOURCE = '(() => {' +
  '  const boot = window.RendererBoot;' +
  '  const map = window.__tpsMock;' +
  '  document.getElementById("bootView")?.setAttribute("hidden", "");' +
  '  document.getElementById("hubView")?.setAttribute("hidden", "");' +
  '  document.getElementById("worldView")?.removeAttribute("hidden");' +
  '  document.body.dataset.gameMode = "world";' +
  '  const canvas = document.getElementById("gameCanvas");' +
  '  window.__tpsEvents = [];' +
  '  if (!boot || !window.CampaignRenderer) return { ok: false, reason: "no renderer" };' +
  '  window.CampaignRenderer.init(canvas, {' +
  '    onTarget: (t) => window.__tpsEvents.push({ type: "target", t }),' +
  '    onHover: (t) => { if (t) window.__tpsEvents.push({ type: "hover", t }); },' +
  '  });' +
  '  window.CampaignRenderer.setData(map);' +
  '  window.CampaignRenderer.setPlayerState({ x: map.px, y: map.py, heading: 1.2 });' +
  '  window.CampaignRenderer.resize();' +
  '  return { ok: true, backend: boot.backend, canvas: { w: canvas.width, h: canvas.height, cw: canvas.clientWidth, ch: canvas.clientHeight } };' +
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
  '  const debug = window.CampaignRenderer.debug();' +
  '  return {' +
  '    backend: boot ? boot.backend : "n/a",' +
  '    failed: boot ? boot.debug().failed : null,' +
  '    detail: debug.delegate || debug,' +
  '    events: (window.__tpsEvents || []).slice(-6),' +
  '  };' +
  '})()';

const MOCK_INSTALL = '(() => { window.__tpsMock = ' + JSON.stringify(makeMap()) + '; window.__tpsEvents = []; return true; })()';

async function main() {
  fs.mkdirSync(SHOT_DIR, { recursive: true });
  const server = await serve();
  const report = { startedAt: new Date().toISOString(), steps: [], shots: [], pixels: [] };
  let edge = null;
  let client = null;
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
    // 画面像素统计：直接解码 PNG 截图（页内 readPixels 读的是已被合成过的
    // WebGL 后台缓冲，往往是全黑假象），并且额外统计画布区域的中心裁剪，
    // 用于排除 HUD 覆盖物干扰、证明 3D 画面本身不是纯色/空白。
    const canvasStats = async (label, pngBuffer) => {
      const stats = pngStats(pngBuffer);
      stats.label = label;
      const center = pngStats(pngBuffer, { x0: 0.2, x1: 0.8, y0: 0.2, y1: 0.85 });
      stats.center = { minLuma: center.minLuma, maxLuma: center.maxLuma, meanLuma: center.meanLuma, distinctColors: center.distinctColors };
      report.pixels.push(stats);
      return stats;
    };

    report.steps.push({ step: 'env', value: await evaluate('({' +
      ' babylon: typeof window.BABYLON,' +
      ' babylonVersion: window.BABYLON ? window.BABYLON.Engine.Version : null,' +
      ' tps: typeof window.TpsRenderer,' +
      ' three: window.RendererBoot ? window.RendererBoot.debug().three_available : "n/a",' +
      ' boot: typeof window.RendererBoot,' +
      ' hasCanvas: !!document.getElementById("gameCanvas"),' +
      ' title: document.title,' +
      ' scriptCount: document.scripts.length,' +
      '})') });

    await evaluate(MOCK_INSTALL);
    report.steps.push({ step: 'drive', value: await evaluate(DRIVE_SOURCE) });
    await sleep(1500);
    report.steps.push({ step: 'idle', value: await evaluate(DEBUG_SOURCE) });
    await canvasStats('tps-idle', await shot('01-tps-idle.png'));

    await evaluate(keySource('keydown', 'KeyW', 'w'));
    await evaluate(keySource('keydown', 'KeyD', 'd'));
    await sleep(1000);
    report.steps.push({ step: 'walk', value: await evaluate(DEBUG_SOURCE) });
    await canvasStats('tps-walk', await shot('02-tps-walk.png'));
    await evaluate(keySource('keyup', 'KeyW', 'w'));
    await evaluate(keySource('keyup', 'KeyD', 'd'));
    await sleep(1300);
    report.steps.push({ step: 'after-walk', value: await evaluate(DEBUG_SOURCE) });

    await evaluate(keySource('keydown', 'Space', ' '));
    await sleep(200);
    report.steps.push({ step: 'jump', value: await evaluate(DEBUG_SOURCE) });
    await canvasStats('tps-jump', await shot('03-tps-jump.png'));
    await evaluate(keySource('keyup', 'Space', ' '));
    await sleep(900);

    report.steps.push({ step: 'authority', value: await evaluate('(() => {' +
      '  window.CampaignRenderer.setPlayerState({ x: 9, y: 15, heading: -1.4 });' +
      '  const d = window.CampaignRenderer.debug().delegate;' +
      '  return { player: d.player, camera: d.camera };' +
      '})()') });
    await sleep(900);
    await canvasStats('tps-authority', await shot('04-tps-authority.png'));

    report.steps.push({ step: 'effects', value: await evaluate('(() => {' +
      '  ["attack", "hit", "dash", "heal", "skill", "impact"].forEach((kind) => window.CampaignRenderer.setEffect({ kind, actor: { x: 9, y: 15 } }));' +
      '  window.CampaignRenderer.setQuality("medium");' +
      '  window.CampaignRenderer.resize();' +
      '  window.CampaignRenderer.setQuality("high");' +
      '  const d = window.CampaignRenderer.debug().delegate;' +
      '  return { effects: d.effects, quality: d.quality, viewport: d.viewport, target: window.CampaignRenderer.targetAt(640, 400) };' +
      '})()') });
    await sleep(700);

    // ---- 回归（Babylon-only）：屏蔽 vendor/babylon.js 后重新加载页面。
    // 本游戏已移除 Three.js 渲染器，缺 Babylon 时页面必须显式报错，绝不静默回退。
    await client.send('Network.enable');
    await client.send('Network.setBlockedURLs', { urls: ['*babylon.js'] });
    await client.send('Page.navigate', { url: 'http://127.0.0.1:' + PORT + '/index.html' });
    await sleep(3500);
    report.steps.push({ step: 'fallback-env', value: await evaluate('({' +
      ' babylon: typeof window.BABYLON,' +
      ' tps: typeof window.TpsRenderer,' +
      ' three: typeof window.THREE,' +
      ' boot: typeof window.RendererBoot,' +
      '})') });
    let initErrorText = '';
    try {
      await evaluate('(() => { const canvas = document.getElementById("gameCanvas"); window.CampaignRenderer.init(canvas, { onTarget: () => {}, onHover: () => {} }); return true; })()');
    } catch (initError) { initErrorText = String(initError && initError.message ? initError.message : initError); }
    report.steps.push({ step: 'fallback', value: { initErrorText: initErrorText.slice(0, 300) } });
    report.steps.push({ step: 'fallback-after', value: { initErrorText: initErrorText.slice(0, 200) } });
    if (!/TpsRenderer|Babylon/i.test(initErrorText)) {
      throw new Error('缺 Babylon 时应显式报错（Babylon-only 契约），实际：' + initErrorText.slice(0, 200));
    }

    report.ok = true;
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
  console.log(JSON.stringify(report, null, 2));
  process.exit(report.ok ? 0 : 1);
}

main();
