#!/usr/bin/env node
// verify_shenghua3_map.mjs — 独立复核写回后的 shenghua3.rs（不依赖生成器内部函数）
// 检查：行数/行宽/边框/行尾逗号 + 全部对象坐标可走 + 入口 BFS 可达 + 墙占比 25-40%
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RS_PATH = path.resolve(__dirname, '..', '..', 'server-rs', 'src', 'worlds', 'shenghua3.rs');
const W = 40, H = 26;
const t = fs.readFileSync(RS_PATH, 'utf8');

// 1) 地图块
const maps = {};
for (const m of t.matchAll(/pub static SHENGHUA3_L(\d)_MAP: &\[&str\] = &\[\n([\s\S]*?)\n\];/g)) {
  const rows = m[2].split('\n').map((l) => l.match(/^"([^"]*)",\s*$/)?.[1]);
  maps[m[1]] = rows;
}
let ok = true;
const issues = [];
for (const k of ['1', '2', '3']) {
  const rows = maps[k];
  if (!rows || rows.length !== H) { ok = false; issues.push(`L${k}: 行数 != ${H}`); }
  for (let y = 0; y < (rows || []).length; y++) {
    if (rows[y] === undefined) { ok = false; issues.push(`L${k} 行${y}: 格式错误`); continue; }
    if (rows[y].length !== W) { ok = false; issues.push(`L${k} 行${y}: 长度 ${rows[y].length} != ${W}`); }
    if ((y === 0 || y === H - 1) && !/^#+$/.test(rows[y])) { ok = false; issues.push(`L${k} 行${y}: 边框非 #`); }
  }
}

// 2) 对象坐标
const grid = { 0: maps['1'].map((r) => r.split('')), 1: maps['2'].map((r) => r.split('')), 2: maps['3'].map((r) => r.split('')) };
const objs = [];
for (const m of t.matchAll(/maps::PointDef \{ id: "([^"]+)",[^}]*?floor: (\d+), x: (\d+), y: (\d+)/g)) objs.push([m[1], +m[2], +m[3], +m[4]]);
for (const m of t.matchAll(/maps::EnemyDef \{ id: "([^"]+)",[^}]*?floor: (\d+), x: (\d+), y: (\d+)/g)) objs.push([m[1], +m[2], +m[3], +m[4]]);
for (const m of t.matchAll(/maps::NpcDef \{ id: "([^"]+)",[^}]*?floor: (\d+), x: (\d+), y: (\d+)/g)) objs.push([m[1], +m[2], +m[3], +m[4]]);
for (const m of t.matchAll(/maps::ZoneDef \{ id: "([^"]+)",[^}]*?floor: (\d+), x: (\d+), y: (\d+)/g)) objs.push([m[1], +m[2], +m[3], +m[4]]);
for (const m of t.matchAll(/maps::PortalDef \{ id: "([^"]+)", floor: (\d+), x: (\d+), y: (\d+)/g)) objs.push([m[1], +m[2], +m[3], +m[4]]);
for (const m of t.matchAll(/id: "(sh3_g\d)", name: "[^"]*", floor: (\d+), x: (\d+), y: (\d+)/g)) objs.push([m[1], +m[2], +m[3], +m[4]]);
const land = {};
for (const m of t.matchAll(/maps::PortalDef \{ id: "([^"]+)", floor: \d+, x: \d+, y: \d+, to_floor: (\d+), tx: (\d+), ty: (\d+)/g)) land[m[2]] = [+m[3], +m[4]];
const entry = { 0: [5, 4], 1: land['1'], 2: land['2'] };
const DIRS = [[1, 0], [-1, 0], [0, 1], [0, -1]];
function reachable(f, sx, sy) {
  const seen = new Set([`${sx},${sy}`]);
  const q = [[sx, sy]];
  while (q.length) {
    const [cx, cy] = q.shift();
    for (const [dx, dy] of DIRS) {
      const nx = cx + dx, ny = cy + dy, k = `${nx},${ny}`;
      if (nx < 0 || nx >= W || ny < 0 || ny >= H || seen.has(k) || grid[f][ny][nx] === '#') continue;
      seen.add(k); q.push([nx, ny]);
    }
  }
  return seen;
}
const report = [];
for (let f = 0; f < 3; f++) {
  const g = grid[f];
  const [ex, ey] = entry[f];
  let floorOk = true;
  if (!g[ey] || g[ey][ex] === '#') { ok = false; floorOk = false; issues.push(`L${f + 1}: 入口 (${ex},${ey}) 是墙`); }
  const reach = reachable(f, ex, ey);
  let nObj = 0, nWalk = 0, nReach = 0;
  for (const [id, fl, x, y] of objs) {
    if (fl !== f) continue;
    nObj++;
    if (g[y]?.[x] === '#') { ok = false; floorOk = false; issues.push(`L${f + 1} ${id} (${x},${y}) 不可走`); } else nWalk++;
    if (reach.has(`${x},${y}`)) nReach++; else { ok = false; floorOk = false; issues.push(`L${f + 1} ${id} (${x},${y}) 不可达`); }
  }
  let walls = 0;
  for (const r of maps[String(f + 1)]) for (const c of r) if (c === '#') walls++;
  const ratio = walls / (W * H);
  if (ratio < 0.25 || ratio > 0.40) { ok = false; floorOk = false; issues.push(`L${f + 1}: 墙占比 ${(ratio * 100).toFixed(1)}% 超限`); }
  report.push(`L${f + 1}: 对象 ${nObj}（可走 ${nWalk} / 可达 ${nReach}） 墙占比 ${(ratio * 100).toFixed(1)}% ${floorOk ? '✔' : '✘'}`);
}
console.log(report.join('\n'));
console.log(ok ? '\n=== 独立复核: 全部通过 ===' : '\n=== 独立复核: 有失败项 ===\n' + issues.join('\n'));
process.exit(ok ? 0 : 1);
