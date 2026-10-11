#!/usr/bin/env node

import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { createReadStream } from 'node:fs';
import * as fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import readline from 'node:readline';

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(SCRIPT_DIR, '..');
const STATIC_ROOT = path.join(REPO_ROOT, 'server-rs', 'release-ui');
const DEFAULT_PROBE = path.join(
  REPO_ROOT,
  'server-rs',
  'target',
  'debug',
  process.platform === 'win32' ? 'campaign-probe.exe' : 'campaign-probe',
);
const DEFAULT_PORT = 9781;
const MAX_BODY_BYTES = 64 * 1024;
const PROBE_TIMEOUT_MS = 5000;
const REQUEST_TIMEOUT_MS = 8000;

class HttpError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

class ProbeError extends HttpError {
  constructor(status, message) {
    super(status, message);
  }
}

function parsePort(value) {
  const port = Number(value);
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    throw new Error('port must be an integer from 0 to 65535');
  }
  return port;
}

function parseArgs(argv) {
  let port = DEFAULT_PORT;
  let probe = DEFAULT_PROBE;
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      console.log('Usage: node tools/campaign-dev-server.mjs [--port PORT] [--probe PATH]');
      process.exit(0);
    }
    if (arg === '--port') {
      if (index + 1 >= argv.length) throw new Error('--port needs a value');
      port = parsePort(argv[++index]);
      continue;
    }
    if (arg.startsWith('--port=')) {
      port = parsePort(arg.slice('--port='.length));
      continue;
    }
    if (arg === '--probe') {
      if (index + 1 >= argv.length) throw new Error('--probe needs a path');
      probe = argv[++index];
      continue;
    }
    if (arg.startsWith('--probe=')) {
      probe = arg.slice('--probe='.length);
      continue;
    }
    throw new Error('unknown argument: ' + arg);
  }
  return { port, probe: path.resolve(process.cwd(), probe) };
}

function pathInside(parent, child) {
  const relative = path.relative(parent, child);
  return (
    relative === '' ||
    (!relative.startsWith('..' + path.sep) &&
      relative !== '..' &&
      !path.isAbsolute(relative))
  );
}

function localHostHeader(req) {
  const host = req.headers.host;
  if (typeof host !== 'string' || host.length === 0) return false;
  let parsed;
  try {
    parsed = new URL('http://' + host);
  } catch {
    return false;
  }
  const hostname = parsed.hostname.replace(/^\[|\]$/g, '').toLowerCase();
  return hostname === '127.0.0.1' || hostname === 'localhost' || hostname === '::1';
}

function allowedOrigin(req, actualPort) {
  const origin = req.headers.origin;
  if (origin === undefined) return null;
  if (Array.isArray(origin) || typeof origin !== 'string') return false;
  let parsed;
  try {
    parsed = new URL(origin);
  } catch {
    return false;
  }
  const hostname = parsed.hostname.replace(/^\[|\]$/g, '').toLowerCase();
  const expectedPort = String(actualPort);
  const originPort = parsed.port || (parsed.protocol === 'http:' ? '80' : '443');
  if (parsed.protocol !== 'http:') return false;
  if (hostname !== '127.0.0.1' && hostname !== 'localhost') return false;
  if (originPort !== expectedPort) return false;
  return origin;
}

function corsHeaders(origin) {
  if (typeof origin !== 'string') return {};
  return {
    'Access-Control-Allow-Origin': origin,
    'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
    Vary: 'Origin',
  };
}

function sendJson(res, status, payload, origin = null) {
  const body = Buffer.from(JSON.stringify(payload));
  res.writeHead(status, {
    ...corsHeaders(origin),
    'Content-Type': 'application/json; charset=utf-8',
    'Content-Length': body.length,
    'Cache-Control': 'no-store',
  });
  res.end(body);
}

function contentTypeIsJson(req) {
  const value = req.headers['content-type'];
  return value === undefined || String(value).toLowerCase().startsWith('application/json');
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let settled = false;
    let total = 0;
    const chunks = [];
    const contentLength = req.headers['content-length'];

    if (contentLength !== undefined) {
      if (!/^\d+$/.test(String(contentLength))) {
        req.resume();
        reject(new HttpError(400, 'invalid Content-Length'));
        return;
      }
      if (Number(contentLength) > MAX_BODY_BYTES) {
        req.resume();
        reject(new HttpError(413, 'request body is too large'));
        return;
      }
    }

    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      req.resume();
      reject(new HttpError(408, 'request body timed out'));
    }, REQUEST_TIMEOUT_MS);
    timer.unref();

    const finish = (callback, value) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      callback(value);
    };

    req.on('data', (chunk) => {
      if (settled) return;
      total += chunk.length;
      if (total > MAX_BODY_BYTES) {
        req.resume();
        finish(reject, new HttpError(413, 'request body is too large'));
        return;
      }
      chunks.push(chunk);
    });
    req.on('end', () => finish(resolve, Buffer.concat(chunks).toString('utf8')));
    req.on('aborted', () => finish(reject, new HttpError(400, 'request was aborted')));
    req.on('error', (error) =>
      finish(reject, new HttpError(400, 'request body failed: ' + error.message)),
    );
  });
}

async function readJson(req, allowEmpty = false) {
  if (!contentTypeIsJson(req)) {
    throw new HttpError(415, 'JSON request body required');
  }
  const text = await readBody(req);
  if (text.trim() === '' && allowEmpty) return {};
  if (text.trim() === '') throw new HttpError(400, 'JSON request body is empty');
  let value;
  try {
    value = JSON.parse(text);
  } catch (error) {
    throw new HttpError(400, 'invalid JSON: ' + error.message);
  }
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new HttpError(400, 'JSON request must be an object');
  }
  return value;
}

function validateActionRequest(value) {
  if (typeof value.action !== 'string' || value.action.trim() === '') {
    throw new HttpError(400, 'action must be a non-empty string');
  }
  if (value.action.length > 128) {
    throw new HttpError(400, 'action is too long');
  }
  const args = value.args === undefined ? {} : value.args;
  if (args === null || typeof args !== 'object' || Array.isArray(args)) {
    throw new HttpError(400, 'args must be a JSON object');
  }
  return { action: value.action, args };
}

let staticRootReal = null;
let actualPort = DEFAULT_PORT;
let configuredProbePath = DEFAULT_PROBE;
let probeChild = null;
let probeReader = null;
let probeExtraLines = [];
let probeLineWaiters = [];
let probeQueue = Promise.resolve();
let shuttingDown = false;

function failProbe(child, error) {
  if (probeChild !== child) return;
  probeChild = null;
  if (probeReader) {
    probeReader.close();
    probeReader = null;
  }
  probeExtraLines = [];
  const waiters = probeLineWaiters;
  probeLineWaiters = [];
  for (const waiter of waiters) {
    clearTimeout(waiter.timer);
    waiter.reject(error);
  }
}

function stopProbe(reason) {
  const child = probeChild;
  if (!child) return;
  const error = reason instanceof Error ? reason : new Error(String(reason));
  failProbe(child, error);
  if (child.stdin && !child.stdin.destroyed) child.stdin.end();
  if (child.exitCode === null) child.kill();
}

function handleProbeLine(line) {
  const waiter = probeLineWaiters.shift();
  if (waiter) {
    clearTimeout(waiter.timer);
    waiter.resolve(line);
  } else {
    probeExtraLines.push(line);
  }
}

async function ensureProbe() {
  if (shuttingDown) {
    throw new ProbeError(503, 'development bridge is stopping');
  }
  if (probeChild && probeChild.exitCode === null && !probeChild.killed) {
    return probeChild;
  }
  let stats;
  try {
    stats = await fs.stat(configuredProbePath);
  } catch (error) {
    throw new ProbeError(
      503,
      'campaign probe is unavailable at ' + configuredProbePath + ': ' + error.message,
    );
  }
  if (!stats.isFile()) {
    throw new ProbeError(503, 'campaign probe path is not a file: ' + configuredProbePath);
  }

  const child = spawn(configuredProbePath, [], {
    stdio: ['pipe', 'pipe', 'inherit'],
    windowsHide: true,
  });
  probeChild = child;
  probeReader = readline.createInterface({ input: child.stdout, crlfDelay: Infinity });
  probeReader.on('line', handleProbeLine);
  child.once('error', (error) => failProbe(child, error));
  child.once('exit', (code, signal) => {
    if (probeChild !== child) return;
    failProbe(
      child,
      new Error(
        'campaign probe stopped before replying' +
          (code === null ? '' : ' with exit code ' + code) +
          (signal === null ? '' : ' after ' + signal),
      ),
    );
  });
  return child;
}

function writeProbeLine(child, line) {
  return new Promise((resolve, reject) => {
    if (!child.stdin || child.stdin.destroyed || !child.stdin.writable) {
      reject(new Error('campaign probe stdin is not writable'));
      return;
    }
    let settled = false;
    const cleanup = () => {
      child.stdin.removeListener('drain', onDrain);
      child.removeListener('error', onError);
      child.removeListener('exit', onExit);
    };
    const finish = (callback, value) => {
      if (settled) return;
      settled = true;
      cleanup();
      callback(value);
    };
    const onDrain = () => finish(resolve);
    const onError = (error) => finish(reject, error);
    const onExit = () => finish(reject, new Error('campaign probe stopped while writing'));
    const canContinue = child.stdin.write(line);
    if (canContinue) {
      resolve();
      return;
    }
    child.stdin.once('drain', onDrain);
    child.once('error', onError);
    child.once('exit', onExit);
  });
}

function nextProbeLine() {
  if (probeExtraLines.length > 0) {
    return Promise.resolve(probeExtraLines.shift());
  }
  return new Promise((resolve, reject) => {
    const waiter = {
      resolve,
      reject,
      timer: setTimeout(() => {
        const index = probeLineWaiters.indexOf(waiter);
        if (index >= 0) probeLineWaiters.splice(index, 1);
        reject(new ProbeError(504, 'campaign probe did not return NDJSON in time'));
      }, PROBE_TIMEOUT_MS),
    };
    waiter.timer.unref();
    probeLineWaiters.push(waiter);
  });
}

async function probeRoundTrip(payload) {
  const child = await ensureProbe();
  try {
    await writeProbeLine(child, JSON.stringify(payload) + '\n');
    const line = await nextProbeLine();
    try {
      return JSON.parse(line);
    } catch (error) {
      throw new ProbeError(502, 'campaign probe returned invalid NDJSON: ' + error.message);
    }
  } catch (error) {
    if (error instanceof ProbeError) {
      if (error.status === 504) stopProbe(error);
      throw error;
    }
    stopProbe(error);
    throw new ProbeError(502, 'campaign probe transport failed: ' + error.message);
  }
}

function queuedProbe(payload) {
  const result = probeQueue.then(() => probeRoundTrip(payload));
  probeQueue = result.catch(() => undefined);
  return result;
}

async function resolveStaticFile(pathname) {
  if (!staticRootReal) throw new HttpError(404, 'release-ui directory is not available');
  let decoded;
  try {
    decoded = decodeURIComponent(pathname);
  } catch {
    throw new HttpError(400, 'invalid URL encoding');
  }
  if (decoded.includes('\0')) throw new HttpError(400, 'invalid URL path');
  const relative = decoded === '/' ? 'index.html' : decoded.replace(/^\/+/, '');
  const candidate = path.resolve(STATIC_ROOT, relative);
  if (!pathInside(STATIC_ROOT, candidate)) {
    throw new HttpError(403, 'path traversal is not allowed');
  }
  let realPath;
  try {
    realPath = await fs.realpath(candidate);
  } catch {
    throw new HttpError(404, 'static file not found');
  }
  if (!pathInside(staticRootReal, realPath)) {
    throw new HttpError(403, 'static symlink escapes release-ui');
  }
  let stats;
  try {
    stats = await fs.stat(realPath);
  } catch {
    throw new HttpError(404, 'static file not found');
  }
  if (!stats.isFile()) throw new HttpError(404, 'static file not found');
  return { realPath, stats };
}

function mimeType(filePath) {
  const ext = path.extname(filePath).toLowerCase();
  const values = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'text/javascript; charset=utf-8',
    '.mjs': 'text/javascript; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.json': 'application/json; charset=utf-8',
    '.svg': 'image/svg+xml',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.webp': 'image/webp',
    '.ico': 'image/x-icon',
    '.woff': 'font/woff',
    '.woff2': 'font/woff2',
  };
  return values[ext] || 'application/octet-stream';
}

async function serveStatic(req, res, url) {
  if (req.method !== 'GET' && req.method !== 'HEAD') {
    throw new HttpError(405, 'static server accepts GET and HEAD only');
  }
  const file = await resolveStaticFile(url.pathname);
  res.writeHead(200, {
    'Content-Type': mimeType(file.realPath),
    'Content-Length': file.stats.size,
    'Cache-Control': 'no-cache',
  });
  if (req.method === 'HEAD') {
    res.end();
    return;
  }
  const stream = createReadStream(file.realPath);
  stream.on('error', () => {
    if (!res.headersSent) res.writeHead(404);
    res.destroy();
  });
  stream.pipe(res);
}

const MEMORY_SAVE_REPLY = {
  memory_only: true,
  saved: false,
  explanation:
    '本地开发桥只连接内存中的 campaign-probe；POST /api/save 不写入 data/campaign-v2.json，也不产生永久存档。请在桌面 Tauri 版本中使用 campaign_save 进行真实保存。',
  message: '内存预览不会永久保存。',
};

const MEMORY_STATE_FIELDS = {
  memory_only: true,
  saved: false,
  save_explanation: '本地开发桥的 probe 状态只存在于内存；view/action 的结果不会写入永久存档。',
};

function markMemoryState(reply) {
  if (reply !== null && typeof reply === 'object' && !Array.isArray(reply)) {
    return { ...reply, ...MEMORY_STATE_FIELDS };
  }
  return { state: reply, ...MEMORY_STATE_FIELDS };
}

async function handleRequest(req, res) {
  if (!localHostHeader(req)) {
    throw new HttpError(403, 'only localhost requests are allowed');
  }
  const origin = allowedOrigin(req, actualPort);
  if (origin === false) {
    throw new HttpError(403, 'Origin must be the same localhost origin');
  }
  const url = new URL(req.url || '/', 'http://127.0.0.1');

  if (url.pathname.startsWith('/api/')) {
    if (req.method === 'OPTIONS') {
      res.writeHead(204, {
        ...corsHeaders(origin),
        'Content-Length': 0,
      });
      res.end();
      return;
    }
    if (url.pathname === '/api/view') {
      if (req.method !== 'GET') throw new HttpError(405, 'GET required for /api/view');
      const reply = await queuedProbe({ action: 'view', args: {} });
      sendJson(res, 200, markMemoryState(reply), origin);
      return;
    }
    if (url.pathname === '/api/action') {
      if (req.method !== 'POST') throw new HttpError(405, 'POST required for /api/action');
      const request = validateActionRequest(await readJson(req));
      const reply = await queuedProbe(request);
      sendJson(res, 200, markMemoryState(reply), origin);
      return;
    }
    if (url.pathname === '/api/save') {
      if (req.method !== 'POST') throw new HttpError(405, 'POST required for /api/save');
      await readJson(req, true);
      sendJson(res, 200, MEMORY_SAVE_REPLY, origin);
      return;
    }
    throw new HttpError(404, 'API route not found');
  }

  await serveStatic(req, res, url);
}

function errorPayload(error) {
  if (error instanceof HttpError) {
    return { status: error.status, error: error.message };
  }
  return { status: 500, error: 'internal bridge error: ' + error.message };
}

async function main() {
  const config = parseArgs(process.argv.slice(2));
  configuredProbePath = config.probe;
  actualPort = config.port;
  try {
    staticRootReal = await fs.realpath(STATIC_ROOT);
  } catch {
    staticRootReal = null;
  }

  const server = createServer((req, res) => {
    handleRequest(req, res).catch((error) => {
      if (res.headersSent) {
        res.destroy();
        return;
      }
      const payload = errorPayload(error);
      sendJson(res, payload.status, payload);
    });
  });
  server.headersTimeout = REQUEST_TIMEOUT_MS + 1000;
  server.requestTimeout = REQUEST_TIMEOUT_MS + 1000;
  server.keepAliveTimeout = 5000;
  server.on('clientError', (_error, socket) => {
    if (!socket.destroyed) socket.end('HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n');
  });

  await new Promise((resolve, reject) => {
    const onError = (error) => {
      server.removeListener('listening', onListening);
      reject(error);
    };
    const onListening = () => {
      server.removeListener('error', onError);
      resolve();
    };
    server.once('error', onError);
    server.once('listening', onListening);
    server.listen(config.port, '127.0.0.1');
  });
  actualPort = server.address().port;
  console.log('campaign dev server: http://127.0.0.1:' + actualPort);
  console.log('static root: ' + (staticRootReal || STATIC_ROOT));
  console.log('campaign probe: ' + configuredProbePath);
  console.log('POST /api/save is memory-only and always returns saved:false');

  const shutdown = () => {
    if (shuttingDown) return;
    shuttingDown = true;
    stopProbe(new Error('development bridge is stopping'));
    server.close(() => {
      process.exitCode = 0;
    });
    const forceExit = setTimeout(() => {
      if (probeChild) stopProbe(new Error('forced bridge shutdown'));
      process.exitCode = 0;
    }, 1500);
    forceExit.unref();
  };
  process.once('SIGINT', shutdown);
  process.once('SIGTERM', shutdown);
  process.once('exit', () => {
    if (probeChild && probeChild.exitCode === null) probeChild.kill();
  });
}

main().catch((error) => {
  console.error('campaign dev server failed:', error.message);
  process.exitCode = 1;
});
