#!/usr/bin/env node
// ============================================================================
// gen_shenghua3_map.mjs — 《无限轮回》蜂巢 shenghua3 三层地图程序化重画
// ----------------------------------------------------------------------------
// 背景：原 SHENGHUA3_L1/L2/L3_MAP 是 40×26 空心大矩形（纯边界墙 + 一片空地），
//       被反馈“太垃圾”。本脚本用「房间 + 走廊」生成器重画三层地图，并自动校验：
//         · 全部对象坐标（调查点/敌人/NPC/传送门/门禁/战圈）可走（非 '#'）
//         · 出生点/传送落点 BFS 可达全部对象坐标
//       校验失败自动修复：坐标落在墙上 → 微调至最近可走格（≤3 格）；不可达 →
//       在对象与最近走廊之间凿 1 格通道。最后把新地图 + 微调坐标写回 shenghua3.rs。
//
// 用法：node tools/design/gen_shenghua3_map.mjs
// 说明：本机 Python 不可用（WindowsApps 存根损坏），故用 Node（任务允许 Python 或 Node）。
// 确定性：种子由世界 slug "shenghua3" 派生，同一版本每次生成结果一致。
// ============================================================================

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..', '..'); // games/wuxian-horror-ch1
const RS_PATH = path.join(ROOT, 'server-rs', 'src', 'worlds', 'shenghua3.rs');
const REPORT_PATH = path.join(__dirname, 'shenghua3_map_gen_report.md');

const W = 40; // 列
const H = 26; // 行
const WALL = '#';
const FLOOR = '.';

// ============================================================================
// 1) 确定性随机
// ============================================================================
function hashStr(s) {
  let h = 2166136261 >>> 0;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619) >>> 0;
  }
  return h >>> 0;
}
function mulberry32(a) {
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const makeRng = (seed) => mulberry32(hashStr(seed));
const ri = (rng, lo, hi) => lo + Math.floor(rng() * (hi - lo + 1)); // [lo, hi]

// ============================================================================
// 2) 网格工具
// ============================================================================
const makeGrid = () => Array.from({ length: H }, () => Array.from({ length: W }, () => WALL));
const inB = (x, y) => x >= 1 && x <= W - 2 && y >= 1 && y <= H - 2; // 内区（不含边界）
const isWalk = (grid, x, y) => grid[y][x] !== WALL;
const setWalk = (grid, x, y) => { if (grid[y][x] === WALL) grid[y][x] = FLOOR; };
const setWall = (grid, x, y) => { grid[y][x] = WALL; };
const wallCount = (grid) => {
  let n = 0;
  for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) if (grid[y][x] === WALL) n++;
  return n;
};
const wallRatio = (grid) => wallCount(grid) / (W * H);
const walkableCells = (grid) => {
  const out = [];
  for (let y = 1; y <= H - 2; y++) for (let x = 1; x <= W - 2; x++) if (grid[y][x] !== WALL) out.push({ x, y });
  return out;
};

// ============================================================================
// 3) 解析 shenghua3.rs 坐标表
// ============================================================================
function parseRs(text) {
  const maps = {};
  const mapRe = /pub static SHENGHUA3_L(\d)_MAP: &\[&str\] = &\[\n([\s\S]*?)\n\];/g;
  let m;
  while ((m = mapRe.exec(text)) !== null) {
    const rows = [];
    for (const line of m[2].split('\n')) {
      const mm = /^"([^"]*)"\s*,?\s*$/.exec(line.trim());
      if (mm) rows.push(mm[1]);
    }
    maps[m[1]] = rows;
  }

  const grab = (re) => {
    const out = [];
    let mm;
    while ((mm = re.exec(text)) !== null) out.push(mm);
    return out;
  };

  const points = grab(/maps::PointDef \{ id: "([^"]+)", name: "([^"]*)", floor: (\d+), x: (\d+), y: (\d+), route: "([^"]+)" \}/g)
    .map((a) => ({ id: a[1], name: a[2], floor: +a[3], x: +a[4], y: +a[5], route: a[6] }));
  const enemies = grab(/maps::EnemyDef \{ id: "([^"]+)", name: "([^"]*)", floor: (\d+), x: (\d+), y: (\d+), radius: (\d+), fight: "([^"]+)" \}/g)
    .map((a) => ({ id: a[1], name: a[2], floor: +a[3], x: +a[4], y: +a[5], radius: +a[6], fight: a[7] }));
  const npcs = grab(/maps::NpcDef \{ id: "([^"]+)", name: "([^"]*)", floor: (\d+), x: (\d+), y: (\d+), talk: "([^"]+)" \}/g)
    .map((a) => ({ id: a[1], name: a[2], floor: +a[3], x: +a[4], y: +a[5], talk: a[6] }));
  const zones = grab(/maps::ZoneDef \{ id: "([^"]+)", name: "([^"]*)", floor: (\d+), x: (\d+), y: (\d+), kind: "([^"]+)", ref_id: "([^"]+)" \}/g)
    .map((a) => ({ id: a[1], name: a[2], floor: +a[3], x: +a[4], y: +a[5], kind: a[6], ref_id: a[7] }));
  const portals = grab(/maps::PortalDef \{ id: "([^"]+)", floor: (\d+), x: (\d+), y: (\d+), to_floor: (\d+), tx: (\d+), ty: (\d+) \}/g)
    .map((a) => ({ id: a[1], floor: +a[2], x: +a[3], y: +a[4], to_floor: +a[5], tx: +a[6], ty: +a[7] }));
  const gates = [];
  {
    const block = /pub static GATES: &\[maps::GateDef\] = &\[([\s\S]*?)\n\];/.exec(text);
    if (block) {
      const re = /id: "([^"]+)", name: "([^"]*)", floor: (\d+), x: (\d+), y: (\d+),/g;
      let mm;
      while ((mm = re.exec(block[1])) !== null) {
        gates.push({ id: mm[1], name: mm[2], floor: +mm[3], x: +mm[4], y: +mm[5] });
      }
    }
  }
  return { maps, points, enemies, npcs, zones, portals, gates };
}

// ============================================================================
// 4) 房间 + 走廊生成
// ============================================================================
// 在桶（bucket）内放置一个矩形房间（footprint 含四周 1 格墙，内部可走）。
// 桶格布点保证房间均匀铺满全图，避免大片死墙区。
function placeRoomInBucket(rng, interiorW, interiorH, b, placed) {
  const fw = interiorW + 2;
  const fh = interiorH + 2;
  if (fw > b.x1 - b.x0 + 1 || fh > b.y1 - b.y0 + 1) return null;
  for (let attempt = 0; attempt < 200; attempt++) {
    const x0 = b.x0 + Math.floor(rng() * (b.x1 - b.x0 - fw + 2));
    const y0 = b.y0 + Math.floor(rng() * (b.y1 - b.y0 - fh + 2));
    const x1 = x0 + fw - 1;
    const y1 = y0 + fh - 1;
    let ok = true;
    for (const r of placed) {
      if (x0 <= r.x1 + 1 && x1 >= r.x0 - 1 && y0 <= r.y1 + 1 && y1 >= r.y0 - 1) { ok = false; break; }
    }
    if (ok) {
      placed.push({ x0, y0, x1, y1 });
      return { x0, y0, x1, y1 };
    }
  }
  return null;
}
// 在全区（pxMin..pxMax × pyMin..pyMax）随机放置房间（供额外房间使用）
function placeRoomFull(rng, interiorW, interiorH, pxMin, pyMin, pxMax, pyMax, placed) {
  const fw = interiorW + 2;
  const fh = interiorH + 2;
  if (fw > pxMax - pxMin + 1 || fh > pyMax - pyMin + 1) return null;
  for (let attempt = 0; attempt < 400; attempt++) {
    const x0 = pxMin + Math.floor(rng() * (pxMax - pxMin - fw + 2));
    const y0 = pyMin + Math.floor(rng() * (pyMax - pyMin - fh + 2));
    const x1 = x0 + fw - 1;
    const y1 = y0 + fh - 1;
    let ok = true;
    for (const r of placed) {
      if (x0 <= r.x1 + 1 && x1 >= r.x0 - 1 && y0 <= r.y1 + 1 && y1 >= r.y0 - 1) { ok = false; break; }
    }
    if (ok) {
      placed.push({ x0, y0, x1, y1 });
      return { x0, y0, x1, y1 };
    }
  }
  return null;
}

const roomCenter = (r) => ({
  x: Math.floor((r.x0 + 1 + r.x1 - 1) / 2),
  y: Math.floor((r.y0 + 1 + r.y1 - 1) / 2),
});
function carveRoom(grid, r) {
  for (let y = r.y0 + 1; y <= r.y1 - 1; y++) for (let x = r.x0 + 1; x <= r.x1 - 1; x++) setWalk(grid, x, y);
}
// L 形走廊（宽 1-2 格）
function carveH(grid, xa, xb, y, w) {
  const lo = Math.min(xa, xb), hi = Math.max(xa, xb);
  for (let x = lo; x <= hi; x++) for (let d = 0; d < w; d++) {
    const yy = y + d;
    if (yy >= 1 && yy <= H - 2) setWalk(grid, x, yy);
  }
}
function carveV(grid, ya, yb, x, w) {
  const lo = Math.min(ya, yb), hi = Math.max(ya, yb);
  for (let y = lo; y <= hi; y++) for (let d = 0; d < w; d++) {
    const xx = x + d;
    if (xx >= 1 && xx <= W - 2) setWalk(grid, xx, y);
  }
}
function carveCorridor(grid, x1, y1, x2, y2, w, rng) {
  if (rng() < 0.5) { carveH(grid, x1, x2, y1, w); carveV(grid, y1, y2, x2, w); }
  else { carveV(grid, y1, y2, x1, w); carveH(grid, x1, x2, y2, w); }
}

// 主题块
// L1 污水池：墙块围成 2-3 格池（在房间内部靠边放置，1 格留白保证可绕行；避开主干走廊）
function addL1Pools(grid, rng, rooms, avoidSet, hallLines, count) {
  const pools = [];
  for (let i = 0; i < count; i++) {
    if (!rooms.length) break;
    const r = rooms[Math.floor(rng() * rooms.length)];
    const iw = r.x1 - r.x0 - 1, ih = r.y1 - r.y0 - 1;
    if (iw < 3 || ih < 2) continue;
    const cx = r.x0 + 1 + ri(rng, 1, iw - 2); // 距房间墙 ≥1
    const cy = r.y0 + 1 + ri(rng, 1, ih - 2);
    const key = `${cx},${cy}`;
    if (avoidSet.has(key) || grid[cy][cx] === WALL || hallLines.has(key)) continue;
    const shape = rng();
    const cells = [[cx, cy]];
    if (shape < 0.35) cells.push([cx + 1, cy]);
    else if (shape < 0.6) cells.push([cx, cy + 1]);
    else if (shape < 0.8) cells.push([cx + 1, cy], [cx, cy + 1]);
    else cells.push([cx + 1, cy], [cx + 2, cy]);
    let ok = true;
    for (const [x, y] of cells) {
      if (grid[y][x] === WALL || avoidSet.has(`${x},${y}`) || hallLines.has(`${x},${y}`)) { ok = false; break; }
    }
    if (!ok) continue;
    for (const [x, y] of cells) setWall(grid, x, y);
    pools.push(cells.length);
  }
  return pools;
}
// L2 审讯室/档案室：1) 审讯室中央桌台墙块 2) 档案室书架隔墙（留 1 格门洞；避开主干走廊）
function addL2Partitions(grid, rng, rooms, hallLines) {
  const done = [];
  const roomW = (r) => r.x1 - r.x0 - 1;
  const roomH = (r) => r.y1 - r.y0 - 1;
  // 审讯室：中央 2×1 桌台
  const tableRoom = rooms.find((r) => roomW(r) >= 5 && roomH(r) >= 4);
  if (tableRoom) {
    const iw = roomW(tableRoom), ih = roomH(tableRoom);
    const cx = tableRoom.x0 + 1 + Math.floor(iw / 2);
    const cy = tableRoom.y0 + 1 + Math.floor(ih / 2);
    if (grid[cy][cx] !== WALL && !hallLines.has(`${cx},${cy}`) && !hallLines.has(`${cx + 1},${cy}`)) {
      setWall(grid, cx, cy);
      if (grid[cy][cx + 1] !== WALL) setWall(grid, cx + 1, cy);
      done.push('审讯室(中央桌台)');
    }
  }
  // 档案室：书架隔墙（横贯房间、留 1 格门洞）
  const shelfRoom = rooms.find((r) => r !== tableRoom && roomW(r) >= 6);
  if (shelfRoom) {
    const iw = roomW(shelfRoom), ih = roomH(shelfRoom);
    const wx = shelfRoom.x0 + 1 + Math.floor(iw / 2); // 书架墙列
    const doorY = shelfRoom.y0 + 1 + ri(rng, 1, ih - 1);
    let placed = 0;
    for (let y = shelfRoom.y0 + 1; y <= shelfRoom.y1 - 1; y++) {
      if (y === doorY) continue;
      if (grid[y][wx] !== WALL && !hallLines.has(`${wx},${y}`)) { setWall(grid, wx, y); placed++; }
    }
    if (placed > 0) done.push('档案室(书架隔墙+门洞)');
  }
  return done;
}

// ============================================================================
// 5) 生成一层
// ============================================================================
function genFloor(floorIdx, rng, rawObjs) {
  const grid = makeGrid();
  const placed = []; // footprints
  const rooms = [];
  const stats = { rooms: [], corridors: 0, corrLen: 0 };

  // 避免池/隔墙落在对象原坐标上
  const avoidSet = new Set();
  for (const o of rawObjs) avoidSet.add(`${o.x},${o.y}`);

  const pxMin = floorIdx === 2 ? 2 : 1;
  const pyMin = floorIdx === 2 ? 2 : 1;
  const pxMax = floorIdx === 2 ? 37 : 38;
  const pyMax = floorIdx === 2 ? 23 : 24;

  if (floorIdx === 2) {
    // L3 外围走廊：沿内圈凿 1 格宽回廊
    for (let x = 1; x <= W - 2; x++) { setWalk(grid, x, 1); setWalk(grid, x, H - 2); }
    for (let y = 1; y <= H - 2; y++) { setWalk(grid, 1, y); setWalk(grid, W - 2, y); }
    stats.ring = true;
  }

  // ---- 桶格布点：内区切成 3 列 × 2 行，每桶放 1 个房间（L3 核心区放中心桶）----
  const BW = Math.floor((pxMax - pxMin + 1) / 3); // 列宽
  const BH = Math.floor((pyMax - pyMin + 1) / 2); // 行高
  const buckets = [];
  for (let bi = 0; bi < 3; bi++) {
    for (let bj = 0; bj < 2; bj++) {
      const bx0 = pxMin + bi * BW;
      const by0 = pyMin + bj * BH;
      const bx1 = bi === 2 ? pxMax : bx0 + BW - 1;
      const by1 = bj === 1 ? pyMax : by0 + BH - 1;
      buckets.push({ x0: bx0, y0: by0, x1: bx1, y1: by1, rooms: [] });
    }
  }
  // 找中心最接近地图中心 (20,13) 的桶 → L3 核心区
  let coreBucket = null, bestD = Infinity;
  for (const b of buckets) {
    const cx = (b.x0 + b.x1) / 2, cy = (b.y0 + b.y1) / 2;
    const d = (cx - 20) ** 2 + (cy - 13) ** 2;
    if (d < bestD) { bestD = d; coreBucket = b; }
  }

  const placeOne = (b, second) => {
    let iw, ih;
    if (floorIdx === 2 && b === coreBucket) { iw = 10; ih = 7; } // 核心区大房间
    else if (second) { iw = ri(rng, 4, 6); ih = ri(rng, 4, 6); }
    else { iw = ri(rng, 5, 8); ih = ri(rng, 5, 8); }
    const r = placeRoomInBucket(rng, iw, ih, b, placed);
    if (r) {
      carveRoom(grid, r);
      rooms.push(r);
      b.rooms.push(r);
      stats.rooms.push(`${r.x1 - r.x0 - 1}×${r.y1 - r.y0 - 1}${iw >= 10 ? '(核心)' : ''}`);
    }
    return r;
  };

  for (const b of buckets) placeOne(b, false); // 6 个基础房间（每桶 1 个）
  // 补 1-2 个房间（全区随机放置，避开已有房间）→ 共 7-8 个
  const extraRooms = [];
  const extras = ri(rng, 1, 2);
  for (let i = 0; i < extras; i++) {
    const iw = ri(rng, 4, 6), ih = ri(rng, 4, 6);
    const r = placeRoomFull(rng, iw, ih, pxMin, pyMin, pxMax, pyMax, placed);
    if (r) {
      carveRoom(grid, r);
      rooms.push(r);
      extraRooms.push(r);
      stats.rooms.push(`${r.x1 - r.x0 - 1}×${r.y1 - r.y0 - 1}`);
    }
  }

  // ---- 连接：桶邻接（右/下）+ 同桶第 2 房间 + 额外房间连最近房间，L 形走廊宽 1-2 ----
  const connectPair = (r1, r2) => {
    if (!r1 || !r2) return;
    const c1 = roomCenter(r1), c2 = roomCenter(r2);
    const w = rng() < 0.55 ? 2 : 1;
    carveCorridor(grid, c1.x, c1.y, c2.x, c2.y, w, rng);
    stats.corridors++;
  };
  for (let bi = 0; bi < 3; bi++) {
    for (let bj = 0; bj < 2; bj++) {
      const b = buckets[bi * 2 + bj];
      if (b.rooms.length >= 2) connectPair(b.rooms[0], b.rooms[1]);
      if (bi < 2) { const rb = buckets[(bi + 1) * 2 + bj]; if (b.rooms.length && rb.rooms.length) connectPair(b.rooms[0], rb.rooms[0]); }
      if (bj < 1) { const db = buckets[bi * 2 + bj + 1]; if (b.rooms.length && db.rooms.length) connectPair(b.rooms[0], db.rooms[0]); }
    }
  }
  // 额外房间连最近房间
  for (const er of extraRooms) {
    let best = null, bestD = Infinity;
    for (const r of rooms) {
      if (r === er) continue;
      const c1 = roomCenter(er), c2 = roomCenter(r);
      const d = Math.abs(c1.x - c2.x) + Math.abs(c1.y - c2.y);
      if (d < bestD) { bestD = d; best = r; }
    }
    if (best) connectPair(er, best);
  }
  // 额外连接：随机挑 2-4 个不同桶的房间对，加走廊（丰富通路 + 提升可走面积）
  const extraPairs = ri(rng, 2, 4);
  for (let i = 0; i < extraPairs; i++) {
    const withRooms = buckets.filter((b) => b.rooms.length > 0);
    if (withRooms.length < 2) break;
    const b1 = withRooms[Math.floor(rng() * withRooms.length)];
    const b2 = withRooms[Math.floor(rng() * withRooms.length)];
    if (b1 === b2) continue;
    connectPair(b1.rooms[0], b2.rooms[0]);
  }
  // ---- 主干走廊：2-3 条横贯 + 2-3 条纵贯（宽 1，设施主通道，大幅提升可走面积）----
  const hallLines = new Set();
  const hCount = ri(rng, 2, 3), vCount = ri(rng, 2, 3);
  for (let i = 0; i < hCount; i++) {
    const y = ri(rng, 1, H - 2);
    for (let x = 1; x <= W - 2; x++) { setWalk(grid, x, y); hallLines.add(`${x},${y}`); }
    stats.halls = (stats.halls || 0) + 1;
  }
  for (let i = 0; i < vCount; i++) {
    const x = ri(rng, 1, W - 2);
    for (let y = 1; y <= H - 2; y++) { setWalk(grid, x, y); hallLines.add(`${x},${y}`); }
    stats.halls = (stats.halls || 0) + 1;
  }
  stats.corrLen = stats.corridors;

  // 墙占比调优：目标 27%-35%（主题与对象凿墙后仍落在 25%-40%）
  for (let i = 0; i < 40; i++) {
    const r = wallRatio(grid);
    if (r >= 0.27 && r <= 0.35) break;
    if (r > 0.35) {
      // 太拥挤：抽样 4 条候选行/列，在墙最密的连续段上开一段走廊（12-24 格，保留房间轮廓）
      let bestLine = null, bestWalls = -1;
      for (let s = 0; s < 4; s++) {
        const horiz = rng() < 0.5;
        const idx = ri(rng, 1, H - 2);
        let walls = 0;
        if (horiz) { for (let x = 1; x <= W - 2; x++) if (grid[idx][x] === WALL) walls++; }
        else { for (let y = 1; y <= H - 2; y++) if (grid[y][idx] === WALL) walls++; }
        if (walls > bestWalls) { bestWalls = walls; bestLine = { horiz, idx }; }
      }
      if (bestWalls <= 0) break;
      const len = ri(rng, 12, 24);
      const limit = bestLine.horiz ? W - 2 - len + 1 : H - 2 - len + 1;
      if (limit < 1) break;
      let bestStart = -1, segWalls = -1;
      for (let s0 = 1; s0 <= limit; s0++) {
        let walls = 0;
        if (bestLine.horiz) { for (let x = s0; x < s0 + len; x++) if (grid[bestLine.idx][x] === WALL) walls++; }
        else { for (let y = s0; y < s0 + len; y++) if (grid[y][bestLine.idx] === WALL) walls++; }
        if (walls > segWalls) { segWalls = walls; bestStart = s0; }
      }
      if (segWalls <= 0) break;
      if (bestLine.horiz) { for (let x = bestStart; x < bestStart + len; x++) setWalk(grid, x, bestLine.idx); }
      else { for (let y = bestStart; y < bestStart + len; y++) setWalk(grid, bestLine.idx, y); }
      stats.extraCorr = (stats.extraCorr || 0) + 1;
    } else {
      // 太开阔：在房间内部补墙块（不堵门、不占对象格、避开主干走廊）
      let added = false;
      for (let t = 0; t < 20 && !added; t++) {
        if (!rooms.length) break;
        const r = rooms[Math.floor(rng() * rooms.length)];
        const iw = r.x1 - r.x0 - 1, ih = r.y1 - r.y0 - 1;
        if (iw < 3 || ih < 3) continue;
        const cx = r.x0 + 1 + ri(rng, 1, iw - 2);
        const cy = r.y0 + 1 + ri(rng, 1, ih - 2);
        const k = `${cx},${cy}`;
        if (grid[cy][cx] === WALL || avoidSet.has(k) || hallLines.has(k)) continue;
        // 避免把房间唯一通道堵死：只挑四邻都可走的格
        if (isWalk(grid, cx - 1, cy) && isWalk(grid, cx + 1, cy) && isWalk(grid, cx, cy - 1) && isWalk(grid, cx, cy + 1)) {
          setWall(grid, cx, cy);
          added = true;
        }
      }
      if (!added) break;
    }
  }

  // 主题（避开主干走廊格）
  if (floorIdx === 0) {
    stats.pools = addL1Pools(grid, rng, rooms, avoidSet, hallLines, ri(rng, 3, 4));
  } else if (floorIdx === 1) {
    stats.partitions = addL2Partitions(grid, rng, rooms, hallLines);
  }

  if (process.env.DEBUG_GEN) {
    console.error(`[dbg] floor ${floorIdx}: rooms=${rooms.length} corr=${stats.corridors} halls=${stats.halls} extraCorr=${stats.extraCorr || 0} ratio=${(wallRatio(grid) * 100).toFixed(1)}%`);
  }

  return { grid, rooms, stats };
}

// ============================================================================
// 6) 坐标校验 + 修复
// ============================================================================
const DIRS = [[1, 0], [-1, 0], [0, 1], [0, -1]];

// 最近可走格：先找 3 步内；找不到放宽到 8 步并在报告标 WARN
function bfsSnap(grid, x, y, maxD) {
  const seen = new Set([`${x},${y}`]);
  let q = [[x, y, 0]];
  let best = null;
  while (q.length) {
    const nq = [];
    for (const [cx, cy, d] of q) {
      for (const [dx, dy] of DIRS) {
        const nx = cx + dx, ny = cy + dy;
        if (nx < 1 || nx > W - 2 || ny < 1 || ny > H - 2) continue;
        const k = `${nx},${ny}`;
        if (seen.has(k)) continue;
        seen.add(k);
        if (grid[ny][nx] !== WALL) {
          if (best === null || d + 1 < best.dist) best = { x: nx, y: ny, dist: d + 1 };
        } else if (d + 1 < maxD) {
          nq.push([nx, ny, d + 1]);
        }
      }
    }
    q = nq;
  }
  return best;
}
function snapCoord(grid, x, y, maxD = 3) {
  if (grid[y][x] !== WALL) return { x, y, moved: false, dist: 0, warn: false };
  let res = bfsSnap(grid, x, y, maxD);
  if (!res) {
    res = bfsSnap(grid, x, y, 8);
    if (res) return { x: res.x, y: res.y, moved: true, dist: res.dist, warn: true };
    return { x, y, moved: false, dist: -1, warn: true }; // 找不到（理论上不会）
  }
  return { x: res.x, y: res.y, moved: true, dist: res.dist, warn: false };
}

// 从 entry 出发的 BFS 可达集
function reachableSet(grid, entry) {
  const seen = new Set([`${entry.x},${entry.y}`]);
  const q = [[entry.x, entry.y]];
  while (q.length) {
    const [cx, cy] = q.shift();
    for (const [dx, dy] of DIRS) {
      const nx = cx + dx, ny = cy + dy;
      if (nx < 0 || nx >= W || ny < 0 || ny >= H) continue;
      const k = `${nx},${ny}`;
      if (seen.has(k) || grid[ny][nx] === WALL) continue;
      seen.add(k);
      q.push([nx, ny]);
    }
  }
  return seen;
}

// 从 obj 到最近可达格凿 1 格通道（BFS 全格找路，回放路径凿开，含 obj 自身格）
function carvePathTo(grid, obj, reachable) {
  const start = `${obj.x},${obj.y}`;
  if (reachable.has(start)) return 0;
  const seen = new Set([start]);
  const parent = new Map();
  let q = [[obj.x, obj.y]];
  let goal = null;
  while (q.length && goal === null) {
    const nq = [];
    for (const [cx, cy] of q) {
      for (const [dx, dy] of DIRS) {
        const nx = cx + dx, ny = cy + dy;
        if (nx < 1 || nx > W - 2 || ny < 1 || ny > H - 2) continue;
        const k = `${nx},${ny}`;
        if (seen.has(k)) continue;
        seen.add(k);
        parent.set(k, `${cx},${cy}`);
        if (reachable.has(k)) { goal = [nx, ny]; break; }
        nq.push([nx, ny]);
      }
      if (goal) break;
    }
    q = nq;
  }
  if (!goal) return 0;
  let cur = `${goal[0]},${goal[1]}`;
  let n = 0;
  while (true) {
    const [x, y] = cur.split(',').map(Number);
    if (grid[y][x] === WALL) { setWalk(grid, x, y); n++; }
    if (cur === start) break;
    cur = parent.get(cur);
    if (cur === undefined) break;
  }
  return n;
}

// 离可走区最远的内部墙格（多源 BFS 距离最大值；用于向墙区开凿长通道）
function deepestWall(grid) {
  const dist = new Map();
  let q = [];
  for (let y = 1; y <= H - 2; y++) {
    for (let x = 1; x <= W - 2; x++) {
      if (grid[y][x] !== WALL) { dist.set(`${x},${y}`, 0); q.push([x, y]); }
    }
  }
  let best = null, bestD = -1;
  while (q.length) {
    const nq = [];
    for (const [cx, cy] of q) {
      const d = dist.get(`${cx},${cy}`);
      for (const [dx, dy] of DIRS) {
        const nx = cx + dx, ny = cy + dy;
        if (nx < 1 || nx > W - 2 || ny < 1 || ny > H - 2) continue;
        const k = `${nx},${ny}`;
        if (dist.has(k)) continue;
        dist.set(k, d + 1);
        nq.push([nx, ny]);
        if (d + 1 > bestD) { bestD = d + 1; best = [nx, ny]; }
      }
    }
    q = nq;
  }
  return best ? { x: best[0], y: best[1] } : null;
}

// 从任意墙格向最近可走格凿 1 格通道（扩展可走面积，用于墙占比调优）
function carveToAnyWalkable(grid, x, y) {
  if (grid[y][x] !== WALL) return 0;
  const start = `${x},${y}`;
  const seen = new Set([start]);
  const parent = new Map();
  let q = [[x, y]];
  let goal = null;
  while (q.length && goal === null) {
    const nq = [];
    for (const [cx, cy] of q) {
      for (const [dx, dy] of DIRS) {
        const nx = cx + dx, ny = cy + dy;
        if (nx < 1 || nx > W - 2 || ny < 1 || ny > H - 2) continue;
        const k = `${nx},${ny}`;
        if (seen.has(k)) continue;
        seen.add(k);
        parent.set(k, `${cx},${cy}`);
        if (grid[ny][nx] !== WALL) { goal = [nx, ny]; break; }
        nq.push([nx, ny]);
      }
      if (goal) break;
    }
    q = nq;
  }
  if (!goal) return 0;
  let cur = `${goal[0]},${goal[1]}`;
  let n = 0;
  while (true) {
    const [gx, gy] = cur.split(',').map(Number);
    if (grid[gy][gx] === WALL) { setWalk(grid, gx, gy); n++; }
    if (cur === start) break;
    cur = parent.get(cur);
    if (cur === undefined) break;
  }
  return n;
}

// 全图可走分量连通：把孤立可走块接到主分量（入口所在），避免装饰性死区
function connectAllComponents(grid, entry) {
  const visited = new Set();
  const comps = [];
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      if (grid[y][x] === WALL || visited.has(`${x},${y}`)) continue;
      const comp = [];
      const q = [[x, y]];
      visited.add(`${x},${y}`);
      while (q.length) {
        const [cx, cy] = q.shift();
        comp.push([cx, cy]);
        for (const [dx, dy] of DIRS) {
          const nx = cx + dx, ny = cy + dy;
          if (nx < 0 || nx >= W || ny < 0 || ny >= H) continue;
          const k = `${nx},${ny}`;
          if (visited.has(k) || grid[ny][nx] === WALL) continue;
          visited.add(k);
          q.push([nx, ny]);
        }
      }
      comps.push(comp);
    }
  }
  const mainComp = comps.find((c) => c.some(([cx, cy]) => cx === entry.x && cy === entry.y));
  if (!mainComp) return 0;
  const mainSet = new Set(mainComp.map(([cx, cy]) => `${cx},${cy}`));
  let carved = 0;
  for (const comp of comps) {
    if (comp === mainComp) continue;
    const [sx, sy] = comp[0];
    carved += carvePathTo(grid, { x: sx, y: sy }, mainSet);
  }
  return carved;
}

// 校验一层：全部对象可走 + BFS 可达；返回报告
function validateFloor(floorIdx, grid, objs, entry) {
  const report = { floor: floorIdx, snaps: [], carved: 0, objs: [], entry };
  // 1) 坐标可走（微调；同步回 src，供写回使用）
  for (const o of objs) {
    const s = snapCoord(grid, o.x, o.y);
    if (s.moved) {
      report.snaps.push({ id: o.id, kind: o.kind, from: [o.x, o.y], to: [s.x, s.y], dist: s.dist, warn: s.warn });
      o.x = s.x; o.y = s.y;
      if (o.src) { o.src.x = s.x; o.src.y = s.y; }
    }
    if (s.warn) report.warn = true;
  }
  // 2) BFS 可达 + 凿墙修复
  for (let iter = 0; iter < 30; iter++) {
    let reachable = reachableSet(grid, entry);
    let bad = [];
    for (const o of objs) if (!reachable.has(`${o.x},${o.y}`)) bad.push(o);
    if (!bad.length) break;
    for (const o of bad) {
      const carved = carvePathTo(grid, o, reachable);
      if (carved > 0) report.carved += carved;
    }
    if (iter === 29) report.carveFail = true;
  }
  // 3) 全图可走分量连通（消除孤立装饰区）
  report.compCarved = connectAllComponents(grid, entry);
  // 4) 复核
  report.objs = objs.map((o) => {
    const reachable = reachableSet(grid, entry);
    return { id: o.id, kind: o.kind, x: o.x, y: o.y, walk: grid[o.y][o.x] !== WALL, reach: reachable.has(`${o.x},${o.y}`) };
  });
  report.wallRatio = wallRatio(grid);
  report.walkCount = W * H - wallCount(grid);
  return report;
}

// ============================================================================
// 7) 写回 shenghua3.rs
// ============================================================================
function escRe(s) { return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); }

function setXY(text, id, x, y) {
  const re = new RegExp(`(id: "${escRe(id)}"[^}]*?x: )\\d+(, y: )\\d+`);
  if (!re.test(text)) throw new Error(`setXY: 找不到 ${id} 的坐标`);
  return text.replace(re, (m, a, b) => `${a}${x}${b}${y}`);
}
function setPortalXY(text, id, x, y, tx, ty) {
  const re = new RegExp(`(id: "${escRe(id)}"[^\n]*?x: )\\d+(, y: )\\d+(, to_floor: \\d, tx: )\\d+(, ty: )\\d+`);
  if (!re.test(text)) throw new Error(`setPortalXY: 找不到 ${id} 的坐标`);
  return text.replace(re, (m, a, b, c, d) => `${a}${x}${b}${y}${c}${tx}${d}${ty}`);
}

function rowsToBlock(rows) {
  return rows.map((r) => `"${r}",`).join('\n');
}

// ============================================================================
// 8) 主流程
// ============================================================================
const text = fs.readFileSync(RS_PATH, 'utf8');
const data = parseRs(text);

// 收集各层对象（含来源坐标）
const byFloor = { 0: [], 1: [], 2: [] };
const addObj = (f, o, kind) => byFloor[f].push({ id: o.id, kind, x: o.x, y: o.y, src: o });
for (const o of data.points) addObj(o.floor, o, '调查点');
for (const o of data.enemies) addObj(o.floor, o, '敌人');
for (const o of data.npcs) addObj(o.floor, o, 'NPC');
for (const o of data.zones) addObj(o.floor, o, '战圈');
for (const o of data.gates) addObj(o.floor, o, '门禁');
const portalSrc = { 0: [], 1: [], 2: [] };
for (const o of data.portals) portalSrc[o.floor].push({ id: o.id, kind: '传送门', x: o.x, y: o.y, src: o });

// 传送落点（目标层，可微调 → 更新 tx/ty）
const landings = { 0: [], 1: [], 2: [] };
for (const o of data.portals) landings[o.to_floor].push({ id: o.id, kind: '传送落点', x: o.tx, y: o.ty, src: o, isLanding: true });

// 入口：
//   L1：出生点 P = (5,4)（跨世界网关 gw_shenghua3 落点，mod.rs 固定不可改 → 强制凿可走）
//   L2/L3：上层传送落点
const ENTRY_L1 = { x: 5, y: 4 };
const L1_ANCHOR = [ENTRY_L1]; // 强制可走（凿墙）

const newMaps = { 1: null, 2: null, 3: null };
const floorReports = [];
const allSnaps = [];

for (let fi = 0; fi < 3; fi++) {
  const rng = makeRng(`shenghua3_L${fi + 1}`);
  const rawObjs = [...byFloor[fi].map((o) => o.src), ...portalSrc[fi].map((o) => o.src)];
  const { grid, rooms, stats } = genFloor(fi, rng, rawObjs);

  // 强制锚点可走（L1 网关落点）
  if (fi === 0) {
    for (const a of L1_ANCHOR) {
      if (grid[a.y][a.x] === WALL) setWalk(grid, a.x, a.y);
    }
  }

  // 本层全部需可走/可达的对象 = 表内对象 + 传送门源
  const objs = [...byFloor[fi], ...portalSrc[fi]];
  // 入口
  let entry;
  if (fi === 0) entry = { ...ENTRY_L1 };
  else {
    const ld = landings[fi];
    if (!ld.length) throw new Error(`第 ${fi} 层没有传送落点`);
    // 落点先快照（可微调 → 更新 tx/ty）
    const s = snapCoord(grid, ld[0].x, ld[0].y);
    if (s.moved) {
      allSnaps.push({ id: ld[0].id, kind: '传送落点', layer: fi, from: [ld[0].x, ld[0].y], to: [s.x, s.y], dist: s.dist, warn: s.warn });
      ld[0].src.tx = s.x; ld[0].src.ty = s.y;
      ld[0].x = s.x; ld[0].y = s.y;
    }
    entry = { x: ld[0].x, y: ld[0].y };
  }

  const rep = validateFloor(fi, grid, objs, entry);
  rep.rooms = stats.rooms;
  rep.corridors = stats.corridors;
  rep.pools = stats.pools || [];
  rep.partitions = stats.partitions || [];
  rep.extraCorr = stats.extraCorr || 0;
  rep.ring = !!stats.ring;
  floorReports.push(rep);

  // 放标记：P 出生 / I 调查点 / E 敌人（装饰，非墙即可走）
  const markerCells = new Set();
  if (fi === 0) {
    grid[ENTRY_L1.y][ENTRY_L1.x] = 'P';
    markerCells.add(`${ENTRY_L1.x},${ENTRY_L1.y}`);
  }
  for (const o of byFloor[fi]) {
    if (o.kind === '调查点' && !markerCells.has(`${o.x},${o.y}`)) {
      grid[o.y][o.x] = 'I';
      markerCells.add(`${o.x},${o.y}`);
    }
  }
  for (const o of byFloor[fi]) {
    if (o.kind === '敌人' && !markerCells.has(`${o.x},${o.y}`)) {
      grid[o.y][o.x] = 'E';
      markerCells.add(`${o.x},${o.y}`);
    }
  }

  newMaps[fi + 1] = grid.map((row) => row.join(''));
}

// ---- 写回 shenghua3.rs ----
let out = text;
const mapRe = /pub static SHENGHUA3_L(\d)_MAP: &\[&str\] = &\[\n[\s\S]*?\n\];/g;
out = out.replace(mapRe, (m, num) => {
  const rows = newMaps[num];
  return `pub static SHENGHUA3_L${num}_MAP: &[&str] = &[\n${rowsToBlock(rows)}\n];`;
});
// 对象坐标更新（调查点/敌人/NPC/战圈/门禁 的 x,y）
for (const list of [data.points, data.enemies, data.npcs, data.zones, data.gates]) {
  for (const o of list) out = setXY(out, o.id, o.x, o.y);
}
// 传送门 x,y + 落点 tx,ty
for (const o of data.portals) out = setPortalXY(out, o.id, o.x, o.y, o.tx, o.ty);

fs.writeFileSync(RS_PATH, out, 'utf8');

// ---- 自校验写回结果 ----
const back = parseRs(out);
const verify = { ok: true, issues: [] };
// 行宽 / 行数 / 边框 / Rust 语法（行尾逗号）
for (const k of ['1', '2', '3']) {
  const rows = back.maps[k];
  if (!rows || rows.length !== H) { verify.ok = false; verify.issues.push(`L${k}: 行数 != ${H}`); }
  for (let y = 0; y < (rows || []).length; y++) {
    if (rows[y].length !== W) { verify.ok = false; verify.issues.push(`L${k} 行${y}: 长度 ${rows[y].length} != ${W}`); }
    if (y === 0 || y === rows.length - 1) {
      for (const ch of rows[y]) if (ch !== '#') { verify.ok = false; verify.issues.push(`L${k} 行${y}: 边框非 #`); break; }
    }
  }
}
// Rust 语法：地图数组每一行必须是 "..." ,（行尾逗号，数组元素分隔符）
{
  const mapBlockRe = /pub static SHENGHUA3_L\d_MAP: &\[&str\] = &\[\n([\s\S]*?)\n\];/g;
  let mm;
  while ((mm = mapBlockRe.exec(out)) !== null) {
    const lines = mm[1].split('\n');
    for (let i = 0; i < lines.length; i++) {
      if (!/^"[^"]*",\s*$/.test(lines[i])) {
        verify.ok = false;
        verify.issues.push(`地图块第 ${i + 1} 行缺行尾逗号或格式错误: ${lines[i].slice(0, 40)}`);
      }
    }
  }
}
// 坐标可走 + 可达（用写回后的表 + 地图）
for (let fi = 0; fi < 3; fi++) {
  const grid = back.maps[String(fi + 1)].map((r) => r.split(''));
  const objs = [];
  for (const o of back.points) if (o.floor === fi) objs.push({ id: o.id, kind: '调查点', x: o.x, y: o.y });
  for (const o of back.enemies) if (o.floor === fi) objs.push({ id: o.id, kind: '敌人', x: o.x, y: o.y });
  for (const o of back.npcs) if (o.floor === fi) objs.push({ id: o.id, kind: 'NPC', x: o.x, y: o.y });
  for (const o of back.zones) if (o.floor === fi) objs.push({ id: o.id, kind: '战圈', x: o.x, y: o.y });
  for (const o of back.gates) if (o.floor === fi) objs.push({ id: o.id, kind: '门禁', x: o.x, y: o.y });
  for (const o of back.portals) if (o.floor === fi) objs.push({ id: o.id, kind: '传送门', x: o.x, y: o.y });
  let entry;
  if (fi === 0) entry = ENTRY_L1;
  else {
    const ld = back.portals.find((p) => p.to_floor === fi);
    entry = { x: ld.tx, y: ld.ty };
  }
  if (grid[entry.y][entry.x] === WALL) { verify.ok = false; verify.issues.push(`L${fi + 1}: 入口 ${entry.x},${entry.y} 是墙`); }
  const reach = reachableSet(grid, entry);
  for (const o of objs) {
    if (grid[o.y][o.x] === WALL) { verify.ok = false; verify.issues.push(`L${fi + 1} ${o.id}: 坐标不可走`); }
    if (!reach.has(`${o.x},${o.y}`)) { verify.ok = false; verify.issues.push(`L${fi + 1} ${o.id}: 不可达`); }
  }
  const ratio = wallRatio(grid);
  if (ratio < 0.25 || ratio > 0.40) {
    verify.ok = false;
    verify.issues.push(`L${fi + 1}: 墙占比 ${(ratio * 100).toFixed(1)}% 超出 25-40%`);
  }
}
if (!verify.issues.length) verify.issues.push('全部通过');

// ---- 报告 ----
const L = [
  'L1 浣熊市地下水道（下水道主题：污水池障碍）',
  'L2 警察局地下（审讯室/档案室）',
  'L3 实验室·孵化室（核心区 + 外围走廊）',
];
const md = [];
md.push('# 蜂巢 shenghua3 三层地图程序化重画报告');
md.push('');
md.push(`- 脚本：\`tools/design/gen_shenghua3_map.mjs\`（Node；本机 Python 不可用，任务允许 Python 或 Node）`);
md.push(`- 目标文件：\`server-rs/src/worlds/shenghua3.rs\`（仅替换 MAP 字符串 + 微调坐标数值，表结构/id/类型不动）`);
md.push(`- 种子：确定性（\`shenghua3_L1/L2/L3\`），同版本每次生成一致`);
md.push(`- 网格：40×26，边框墙保留，行宽固定 40`);
md.push('');
md.push('## 生成方案');
md.push('');
md.push('| 层 | 房间 | 走廊 | 主题 | 墙占比(修复后) |');
md.push('|---|---|---|---|---|');
for (let fi = 0; fi < 3; fi++) {
  const rep = floorReports[fi];
  const theme = fi === 0 ? `污水池 ×${rep.pools.length}` : fi === 1 ? rep.partitions.join('、') : '核心区大房间 + 内圈回廊';
  md.push(`| ${L[fi].slice(0, 8)} | ${rep.rooms.length} 个（${rep.rooms.slice(0, 3).join('、')}…） | ${rep.corridors} 段 L 形（宽 1-2）${rep.extraCorr ? `+补 ${rep.extraCorr} 段` : ''} | ${theme} | ${(rep.wallRatio * 100).toFixed(1)}% |`);
}
md.push('');
md.push('> 房间内部 4-8 格、墙 #、内部可走；走廊 1-2 格宽可走；生成后自动调优墙占比到 25-40%。');
md.push('');
md.push('## 校验结果');
md.push('');
md.push('| 层 | 对象数 | 坐标微调(个) | 凿墙格数 | 全部可走 | 全部可达 | 墙占比 |');
md.push('|---|---|---|---|---|---|---|');
let allOk = true;
for (let fi = 0; fi < 3; fi++) {
  const rep = floorReports[fi];
  const walkOk = rep.objs.every((o) => o.walk);
  const reachOk = rep.objs.every((o) => o.reach);
  if (!walkOk || !reachOk) allOk = false;
  md.push(`| ${L[fi].slice(0, 8)} | ${rep.objs.length} | ${rep.snaps.length} | ${rep.carved + (rep.compCarved || 0)} | ${walkOk ? '✔' : '✘'} | ${reachOk ? '✔' : '✘'} | ${(rep.wallRatio * 100).toFixed(1)}% |`);
}
md.push('');
md.push(`**整体：${allOk ? '✔ 校验全部通过' : '✘ 存在失败项'}**；写回自校验：${verify.ok ? '✔ 通过' : '✘ 失败'}`);
if (!verify.ok) for (const i of verify.issues) md.push(`- ⚠ ${i}`);
md.push('');
md.push('### 坐标微调明细（对象原坐标落在墙上 → 最近可走格）');
md.push('');
const snapRows = [];
for (const rep of floorReports) for (const s of rep.snaps) snapRows.push({ ...s, layer: rep.floor });
for (const s of allSnaps) snapRows.push(s);
if (!snapRows.length) md.push('无（全部对象原坐标即落在可走格）');
else {
  md.push('| 层 | 对象 | 原坐标 | 新坐标 | 距离 |');
  md.push('|---|---|---|---|---|');
  for (const s of snapRows) md.push(`| ${s.layer !== undefined ? `L${s.layer + 1}` : '落点'} | ${s.id} | (${s.from[0]},${s.from[1]}) | (${s.to[0]},${s.to[1]}) | ${s.dist}${s.warn ? ' ⚠' : ''} |`);
}
md.push('');
md.push('## 新地图 ASCII（40×26，#墙 .可走 P出生 I调查点 E敌人）');
md.push('');
for (let fi = 0; fi < 3; fi++) {
  md.push(`### ${L[fi]}`);
  md.push('');
  md.push('```');
  md.push(newMaps[fi + 1].join('\n'));
  md.push('```');
  md.push('');
}
md.push('## 更新确认');
md.push('');
md.push(`- \`${RS_PATH}\` 已写入：3 张新地图 + 微调坐标（仅数值变更，结构/id/类型/注释未动）`);
md.push(`- 行宽自校验：${verify.ok ? '每行 40 字符、26 行、边框 # ✔' : '存在异常'}`);
md.push(`- 生成时间：${new Date().toISOString()}`);

fs.writeFileSync(REPORT_PATH, md.join('\n'), 'utf8');

// ---- 控制台输出 ----
console.log('===== 蜂巢 shenghua3 三层地图重画 =====');
for (let fi = 0; fi < 3; fi++) {
  const rep = floorReports[fi];
  console.log(`\n[${L[fi]}]`);
  console.log(`  房间 ${rep.rooms.length} 个 | 走廊 ${rep.corridors} 段${rep.extraCorr ? `(+补 ${rep.extraCorr} 条)` : ''} | 对象 ${rep.objs.length} 个`);
  console.log(`  微调 ${rep.snaps.length} 个坐标 | 对象凿墙 ${rep.carved} 格 + 连通清理 ${rep.compCarved || 0} 格 | 墙占比 ${(rep.wallRatio * 100).toFixed(1)}%`);
  console.log(`  可走 ${rep.objs.every((o) => o.walk) ? '✔' : '✘'} | 可达 ${rep.objs.every((o) => o.reach) ? '✔' : '✘'}`);
  console.log(newMaps[fi + 1].join('\n'));
}
console.log(`\n写回自校验: ${verify.ok ? '✔ 通过' : '✘ 失败: ' + verify.issues.join('; ')}`);
console.log(`报告: ${REPORT_PATH}`);
if (!verify.ok) process.exit(1);
