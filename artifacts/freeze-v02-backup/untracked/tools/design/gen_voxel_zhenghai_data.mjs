// gen_voxel_zhenghai_data.mjs — 郑吒高分辨率体素建模数据生成器（Node 版，无外部依赖）
// ============================================================================
// 流水线：
//   1) 解析 makehuman_base.obj（y-up 四边形）→ 三角形网格，应用 muscle target
//   2) 缩放至 2.3 世界高、脚底贴地；解剖地标检测（颈/肩/手/裆/踝/下颌）
//   3) 头部放大（贴合立绘 5 头身大头的头宽/头高）+ 手臂垂放手术（臂降至立绘拳位）
//   4) 纵向比例 warp（mesh 关键点 → 立绘关键点：下颌 23% / 肩 25% / 裆 55% / 踝 91%）
//   5) 网格体素化：粗网格（全身 SV=0.055，内部实心填充）＋ 头部细网格（SVH=0.0275）
//   6) 立绘剪影贴合（trim 超出立绘轮廓的体素列；肩部合并区补列）
//   7) 逐体素取色：按暴露面方向投影采样正/侧/背立绘（run 夹取 + 部位·v 区中位色回退）
//   8) 烘焙方向明暗（MC 风格面着色，无光照依赖）→ 输出 voxel_zhenghai_data.js
// 用法：node gen_voxel_zhenghai_data.mjs [--sv 0.055] [--no-warp] [--head-scale 1.28]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { decodePng, dehalo, ccClean, erodeAlpha, detectBBox, rowRuns, alphaMask, sample, pxAt } from './pnglib.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..', '..');
const OBJ = path.join(ROOT, 'tools', 'model_task_pkg', 'human_ref', 'makehuman_base.obj');
const TARGET = path.join(ROOT, 'tools', 'model_task_pkg', 'human_ref', 'makehuman_targets', 'universal-male-young-maxmuscle-averageweight.target');
const IMG = path.join(ROOT, 'server-rs', 'ui', 'assets', 'img');
const OUT = path.join(ROOT, 'server-rs', 'ui', 'js', 'voxel_zhenghai_data.js');

const args = process.argv.slice(2);
const ARG = {};
for (let i = 0; i < args.length; i += 2) {
  if (args[i].startsWith('--') && args[i + 1] !== undefined) ARG[args[i].slice(2)] = args[i + 1];
}
// ===================== 可调参数 =====================
const H = 2.3;
const SV = ARG.sv ? parseFloat(ARG.sv) : 0.055;
const SVH = SV / 2;
const HS_XZ = ARG['head-scale'] ? parseFloat(ARG['head-scale']) : 1.28;  // 头宽
const HS_Z = 1.18;   // 头深
const HS_Y = 1.28;   // 头高（warp 会进一步精确校正）
const ARM_RAD = 0.185;       // 手臂手术半径阈值（maxmuscle 使上臂外缘达 0.17）
const ARM_HAND_Y = 1.07;     // 手术后（warp 前）拳底目标 y
const TRIM_MARGIN = 0.45;    // 剪影 trim 容差（×体素边长；收紧使肩/臂轮廓贴合立绘）
const FILL_V0 = 0.215, FILL_V1 = 0.33;  // 肩部合并补列立绘 v 区间（肩 0.228 上下的单 run 行）
const NO_WARP = ARG['no-warp'] === '1';
// 立绘关键点（bbox v 自上而下；由立绘逐像素实测校准——深色五官带 v0.15~0.19）：
//   发顶 0 → 眉 0.145 → 眼 0.16 → 嘴 0.18 → 下巴 0.19 → 肩 0.228 → 裆 0.546 → 踝 0.927
// 世界 y 由 vOf 逆映射计算（vOf 含 bbox top/bottom 偏移，直接用 y=H(1-v) 会差 ~0.08）
const ART_V = { brow: 0.145, eye: 0.16, mouth: 0.18, chin: 0.19, shoulder: 0.228, crotch: 0.546, ankle: 0.927 };
const SHADE = { '+y': 1.10, '-y': 0.88, '+z': 1.03, '-z': 0.94, '+x': 0.99, '-x': 0.99 };
const PRIORITY = ['+z', '-z', '+x', '-x', '+y', '-y'];
const FACE_BIT = { '+x': 1, '-x': 2, '+y': 4, '-y': 8, '+z': 16, '-z': 32 };

const clamp = (v, a, b) => v < a ? a : (v > b ? b : v);
const smooth = (t) => { t = clamp(t, 0, 1); return t * t * (3 - 2 * t); };

// ===================== 网格解析 =====================
function parseObj(objPath) {
  const lines = fs.readFileSync(objPath, 'utf8').split(/\r?\n/);
  const verts = [];
  const faces = [];
  for (const line of lines) {
    if (line.startsWith('v ')) {
      const t = line.split(/\s+/);
      verts.push([parseFloat(t[1]), parseFloat(t[2]), parseFloat(t[3])]);
    } else if (line.startsWith('f ')) {
      const t = line.split(/\s+/).slice(1);
      faces.push(t.map(s => parseInt(s.split('/')[0], 10) - 1));
    }
  }
  const tris = [];
  for (const f of faces) for (let i = 1; i < f.length - 1; i++) tris.push([f[0], f[i], f[i + 1]]);
  return { verts, tris };
}
function parseTarget(p) {
  const lines = fs.readFileSync(p, 'utf8').split(/\r?\n/);
  const out = [];
  for (const line of lines) {
    if (!line.trim() || line.startsWith('#')) continue;
    const t = line.trim().split(/\s+/);
    out.push([parseInt(t[0], 10), parseFloat(t[1]), parseFloat(t[2]), parseFloat(t[3])]);
  }
  return out;
}

// ===================== 三角形 vs AABB（Akenine-Möller）=====================
const CROSS = (ax, ay, az, bx, by, bz) => [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx];
const DOT = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
function triBoxOverlap(bc, bh, v0, v1, v2) {
  const v0x = v0[0] - bc[0], v0y = v0[1] - bc[1], v0z = v0[2] - bc[2];
  const v1x = v1[0] - bc[0], v1y = v1[1] - bc[1], v1z = v1[2] - bc[2];
  const v2x = v2[0] - bc[0], v2y = v2[1] - bc[1], v2z = v2[2] - bc[2];
  if (Math.min(v0x, v1x, v2x) > bh[0] || Math.max(v0x, v1x, v2x) < -bh[0]) return false;
  if (Math.min(v0y, v1y, v2y) > bh[1] || Math.max(v0y, v1y, v2y) < -bh[1]) return false;
  if (Math.min(v0z, v1z, v2z) > bh[2] || Math.max(v0z, v1z, v2z) < -bh[2]) return false;
  const e0 = [v1x - v0x, v1y - v0y, v1z - v0z];
  const e1 = [v2x - v1x, v2y - v1y, v2z - v1z];
  const e2 = [v0x - v2x, v0y - v2y, v0z - v2z];
  const n = CROSS(e0[0], e0[1], e0[2], e1[0], e1[1], e1[2]);
  const nd = Math.abs(DOT(n, [v0x, v0y, v0z]));
  const np = bh[0] * Math.abs(n[0]) + bh[1] * Math.abs(n[1]) + bh[2] * Math.abs(n[2]);
  if (nd > np) return false;
  const axs = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
  const edges = [e0, e1, e2];
  for (let i = 0; i < 3; i++) for (let j = 0; j < 3; j++) {
    const l = CROSS(edges[i][0], edges[i][1], edges[i][2], axs[j][0], axs[j][1], axs[j][2]);
    if (!l[0] && !l[1] && !l[2]) continue;
    const p0 = DOT(l, [v0x, v0y, v0z]), p1 = DOT(l, [v1x, v1y, v1z]), p2 = DOT(l, [v2x, v2y, v2z]);
    const r = bh[0] * Math.abs(l[0]) + bh[1] * Math.abs(l[1]) + bh[2] * Math.abs(l[2]);
    if (Math.min(p0, p1, p2) > r || Math.max(p0, p1, p2) < -r) return false;
  }
  return true;
}

// ===================== 体素化（表面 + 内部填充）=====================
function voxelize(verts, tris, sv, x0, y0, z0, nx, ny, nz) {
  const occ = new Uint8Array(nx * ny * nz);
  const bh = [sv / 2, sv / 2, sv / 2];
  // 边界 epsilon：三角形恰落在体素边界（如脚底 y=0）时仍标记相邻单元，
  // 否则脚底/薄壁处会漏表面体素，flood fill 从缺口灌入内部。
  const eps = sv * 0.08;
  const cellOf = (x, y, z) => [Math.floor((x - x0) / sv), Math.floor((y - y0) / sv), Math.floor((z - z0) / sv)];
  for (const tri of tris) {
    const a = verts[tri[0]], b = verts[tri[1]], c = verts[tri[2]];
    const [ix0, iy0, iz0] = cellOf(Math.min(a[0], b[0], c[0]) - eps, Math.min(a[1], b[1], c[1]) - eps, Math.min(a[2], b[2], c[2]) - eps);
    const [ix1, iy1, iz1] = cellOf(Math.max(a[0], b[0], c[0]) + eps, Math.max(a[1], b[1], c[1]) + eps, Math.max(a[2], b[2], c[2]) + eps);
    for (let iz = Math.max(0, iz0); iz <= Math.min(nz - 1, iz1); iz++) {
      for (let iy = Math.max(0, iy0); iy <= Math.min(ny - 1, iy1); iy++) {
        for (let ix = Math.max(0, ix0); ix <= Math.min(nx - 1, ix1); ix++) {
          const bc = [x0 + (ix + 0.5) * sv, y0 + (iy + 0.5) * sv, z0 + (iz + 0.5) * sv];
          // 盒测试用微扩半尺寸：三角形恰贴盒面（脚底 y=0）也算重叠，防内部填充漏灌
          const bhe = [bh[0] + sv * 0.02, bh[1] + sv * 0.02, bh[2] + sv * 0.02];
          if (triBoxOverlap(bc, bhe, a, b, c)) occ[iz * ny * nx + iy * nx + ix] = 1;
        }
      }
    }
  }
  const filled = new Uint8Array(nx * ny * nz);
  const q = [];
  const push = (ix, iy, iz) => {
    const p = iz * ny * nx + iy * nx + ix;
    if (occ[p] || filled[p]) return;
    filled[p] = 1; q.push(p);
  };
  for (let ix = 0; ix < nx; ix++) for (let iy = 0; iy < ny; iy++) { push(ix, iy, 0); push(ix, iy, nz - 1); }
  for (let iz = 0; iz < nz; iz++) for (let iy = 0; iy < ny; iy++) { push(0, iy, iz); push(nx - 1, iy, iz); }
  for (let iz = 0; iz < nz; iz++) for (let ix = 0; ix < nx; ix++) { push(ix, 0, iz); push(ix, ny - 1, iz); }
  while (q.length) {
    const p = q.pop();
    const iz = (p / (ny * nx)) | 0, rem = p % (ny * nx), iy = (rem / nx) | 0, ix = rem % nx;
    if (ix > 0) push(ix - 1, iy, iz);
    if (ix < nx - 1) push(ix + 1, iy, iz);
    if (iy > 0) push(ix, iy - 1, iz);
    if (iy < ny - 1) push(ix, iy + 1, iz);
    if (iz > 0) push(ix, iy, iz - 1);
    if (iz < nz - 1) push(ix, iy, iz + 1);
  }
  for (let i = 0; i < nx * ny * nz; i++) if (!filled[i]) occ[i] = 1;
  return { occ, nx, ny, nz };
}

// ===================== 立绘 =====================
function loadPortrait(fname) {
  const img = decodePng(fs.readFileSync(path.join(IMG, fname)));
  dehalo(img);
  ccClean(img);
  erodeAlpha(img, 2);
  const bb = detectBBox(img);
  const runs = rowRuns(img, bb);
  const mask = alphaMask(img);
  return { img, bb, runs, mask };
}
const vOf = (bb, y) => bb.top + (1 - y / H) * (bb.bottom - bb.top);
const rowOf = (runs, v) => clamp(Math.floor(v * runs.hn), 0, runs.hn - 1);

function main() {
  const t0 = Date.now();
  const { verts, tris } = parseObj(OBJ);
  const tgt = parseTarget(TARGET);
  console.log(`OBJ: verts=${verts.length} tris=${tris.length} targetLines=${tgt.length}`);
  for (const [i, dx, dy, dz] of tgt) {
    if (i >= 0 && i < verts.length) { verts[i][0] += dx; verts[i][1] += dy; verts[i][2] += dz; }
  }
  const yMin = Math.min(...verts.map(v => v[1])), yMax = Math.max(...verts.map(v => v[1]));
  const s = H / (yMax - yMin);
  for (const v of verts) { v[0] *= s; v[1] = (v[1] - yMin) * s; v[2] *= s; }

  // ---- 地标检测（0.02 世界宽的顶点带）----
  const BAND = 0.02;
  const bands = [];
  for (let y = 0; y <= H; y += BAND) {
    const vs = [];
    let mx = 0, mz = 0, cw = 0;
    for (const v of verts) {
      if (v[1] < y - BAND / 2 || v[1] > y + BAND / 2) continue;
      vs.push(v);
      const ax = Math.abs(v[0]);
      if (ax > mx) { mx = ax; mz = v[2]; }
      if (ax < 0.25 && ax > cw) cw = ax;   // 中央宽度（排除手臂）
    }
    bands.push({ y, n: vs.length, mx, mz, cw });
  }
  let neckY = 1.95, neckW = 1e9;
  for (const b of bands) if (b.y > 1.5 && b.y < 2.2 && b.n > 15 && b.cw < neckW) { neckW = b.cw; neckY = b.y; }
  // 臂外缘轨迹：max|x|>0.30 的连续带（y 1.2~1.95），最低=拳、最高=肩
  const armTrace = bands.filter(b => b.y > 1.15 && b.y < 1.95 && b.mx > 0.30);
  const handB = armTrace.length ? armTrace[0] : null;           // 最低带 = 拳
  const shB = armTrace.length ? armTrace[armTrace.length - 1] : null; // 最高带 = 肩
  const shoulderY = shB ? shB.y : neckY - 0.12;
  const shoulderX = shB ? shB.mx : 0.35;
  const shoulderZ = 0.30; // 手术用肩点 z：前于胸（侧视手臂可见，贴合立绘）
  const handY = handB ? handB.y : 1.30;
  const handX = handB ? handB.mx : 0.68;
  const handZ = handB ? handB.mz : 0.44;
  console.log(`armTrace: ${armTrace.map(b => b.y.toFixed(2) + ':' + b.mx.toFixed(2) + ':' + b.mz.toFixed(2)).join(' ')}`);
  // 裆：拳下方（< handY-0.15）最高的 2-blob 带（腿分叉）
  let crotchY = 1.05;
  {
    const found = [];
    for (const b of bands) {
      if (b.y < 0.3 || b.y > handY - 0.15 || b.n < 40) continue;
      const xs = [];
      for (const v of verts) if (Math.abs(v[1] - b.y) <= BAND / 2 && Math.abs(v[0]) < 0.5) xs.push(v[0]);
      xs.sort((a, b2) => a - b2);
      const blobs = [];
      for (const x of xs) {
        if (blobs.length && x - blobs[blobs.length - 1][1] < 0.09) blobs[blobs.length - 1][1] = x;
        else blobs.push([x, x]);
      }
      const big = blobs.filter(b2 => b2[1] - b2[0] > 0.05);
      if (big.length >= 2 && Math.abs(big[0][0] + big[0][1] - big[1][0] - big[1][1]) > 0.1) found.push(b.y);
    }
    if (found.length) crotchY = found[found.length - 1]; // 最高 y（向下扫时最后遇到的=最靠上）
  }
  const ankleY = 0.09; // 网格脚顶（mesh 解剖固定值）
  // 下颌：颈上第一段 cw 越过 0.075 的带（头放大后重新检测）
  let jawY = neckY + 0.10;
  for (const b of bands) if (b.y > neckY + 0.01 && b.y < neckY + 0.45 && b.cw > 0.075) { jawY = b.y; break; }
  console.log(`landmarks: neckY=${neckY.toFixed(3)} cw@neck=${neckW.toFixed(3)} jawY=${jawY.toFixed(3)} shoulder=(${shoulderX.toFixed(3)},${shoulderY.toFixed(3)},${shoulderZ.toFixed(3)}) hand=(${handX.toFixed(3)},${handY.toFixed(3)},${handZ.toFixed(3)}) crotchY=${crotchY.toFixed(3)} ankleY=${ankleY.toFixed(3)}`);

  // ---- 立绘（提前加载，供 ART 关键点/后续取色）----
  const fr = loadPortrait('pc_zhengzha.png');
  const sd = loadPortrait('pc_zhengzha_side.png');
  const bk = loadPortrait('pc_zhengzha_back.png');
  // ART 关键点世界坐标：vOf 逆映射（vOf(y)=bb.top+(1-y/H)*(bb.bottom-bb.top)）
  const ART = {};
  const vInv = (v) => H * (1 - (v - fr.bb.top) / (fr.bb.bottom - fr.bb.top));
  for (const [k, vv] of Object.entries(ART_V)) ART[k] = vInv(vv);
  console.log(`ART keypoints: ${JSON.stringify(Object.fromEntries(Object.entries(ART).map(([k, v]) => [k, +v.toFixed(3)])))}`);

  // ---- 头部放大（仅 |x|<0.22 的头颈区，颈下 0.14 衰减）----
  const neckCy = neckY - 0.02, neckCz = 0.10, HEAD_FALLOFF = 0.14;
  for (const v of verts) {
    if (v[1] < neckCy - HEAD_FALLOFF || Math.abs(v[0]) > 0.22) continue;
    const f = smooth((v[1] - (neckCy - HEAD_FALLOFF)) / HEAD_FALLOFF);
    v[0] *= 1 + (HS_XZ - 1) * f;
    v[1] = neckCy + (v[1] - neckCy) * (1 + (HS_Y - 1) * f);
    v[2] = neckCz + (v[2] - neckCz) * (1 + (HS_Z - 1) * f);
  }
  // 头放大后重新归一化到 H（顶可能超出）
  {
    const yMx = Math.max(...verts.map(v => v[1]));
    if (yMx > H) { const k = H / yMx; for (const v of verts) v[1] *= k; }
    const yMn = Math.min(...verts.map(v => v[1]));
    if (yMn < 0) { for (const v of verts) v[1] -= yMn; }
  }
  // 头放大后重测下颌（cw 阈值随头宽放大）
  {
    let jw = neckY + 0.10;
    for (const b of bands) if (b.y > neckY + 0.01 && b.y < neckY + 0.45 && b.cw > 0.075) { jw = b.y; break; }
    // bands 基于原顶点；按缩放后比例重算：头高方向近似等比
    jawY = neckY + (jw - neckY) * (1 + (HS_Y - 1) * 0.5);
  }
  // 头面特征检测（头放大后，warp 前）：下巴/嘴/眼/眉 用脸前部 maxZ 剖面
  // （立绘五官集中在头部下半部，需头部内部关键点 warp 对齐，否则五官采样错位）
  {
    const prof = [];
    for (let y = 1.55; y <= H; y += 0.01) {
      let mz = -1;
      for (const v of verts) {
        if (Math.abs(v[1] - y) > 0.005 || Math.abs(v[0]) > 0.09 || v[2] <= 0.05) continue;
        if (v[2] > mz) mz = v[2];
      }
      if (mz > 0) prof.push({ y, mz });
    }
    const at = (y) => { let best = null, bd = 1e9; for (const p of prof) { const d = Math.abs(p.y - y); if (d < bd) { bd = d; best = p; } } return best; };
    // 眼：脸前部 maxZ 峰值（1.85~2.15）
    let eyeY = 2.0, eyeMax = -1;
    for (const p of prof) if (p.y > 1.85 && p.y < 2.15 && p.mz > eyeMax) { eyeMax = p.mz; eyeY = p.y; }
    // 下巴：眼下方 maxZ 谷值
    let jaw2Y = jawY, jawMin = 1e9;
    for (const p of prof) if (p.y > 1.70 && p.y < eyeY - 0.02 && p.mz < jawMin) { jawMin = p.mz; jaw2Y = p.y; }
    // 嘴：下巴与眼之间的局部谷值
    let mouthY = (jaw2Y + eyeY) / 2, mouthMin = 1e9;
    for (const p of prof) if (p.y > jaw2Y + 0.01 && p.y < eyeY - 0.01 && p.mz < mouthMin) { mouthMin = p.mz; mouthY = p.y; }
    // 眉：眼上方 maxZ 接近眼部峰值的第一个带
    let browY = eyeY + 0.05;
    for (const p of prof) if (p.y > eyeY + 0.005 && p.y < eyeY + 0.12 && p.mz >= eyeMax - 0.02) { browY = p.y; break; }
    // 用剖面检测结果替换 cw 估测的下颌
    if (Math.abs(jaw2Y - jawY) > 0.02) jawY = jaw2Y;
    console.log(`head features (post-scale): jaw=${jawY.toFixed(3)} mouth=${mouthY.toFixed(3)} eye=${eyeY.toFixed(3)} brow=${browY.toFixed(3)}`);
    const headFeat = { jaw: jawY, mouth: mouthY, eye: eyeY, brow: browY };
    globalThis.__headFeat = headFeat;
  }

  // ---- 手臂垂放手术（绕肩转正 + 拳降至目标位；两遍：核心 + 外缘衰减）----
  const armLen = Math.hypot(handX - shoulderX, handY - shoulderY, handZ - shoulderZ);
  const shift = shoulderY - armLen - ARM_HAND_Y;
  console.log(`arm surgery: len=${armLen.toFixed(3)} shift=${shift.toFixed(3)}`);
  for (const side of [-1, 1]) {
    const S = [side * shoulderX, shoulderY, shoulderZ];
    const d = [side * (handX - shoulderX) / armLen, (handY - shoulderY) / armLen, (handZ - shoulderZ) / armLen];
    // 第一遍：核心（radial<0.14）完全转正
    const moved = new Uint8Array(verts.length);
    for (let vi = 0; vi < verts.length; vi++) {
      const v = verts[vi];
      if (v[1] < handY - 0.10 || v[1] > shoulderY + 0.16) continue;
      if (side * v[0] < 0.19) continue;
      const vx = v[0] - S[0], vy = v[1] - S[1], vz = v[2] - S[2];
      const t = vx * d[0] + vy * d[1] + vz * d[2];
      if (t < -0.10 || t > armLen + 0.12) continue;
      const rx = vx - d[0] * t, ry = vy - d[1] * t, rz = vz - d[2] * t;
      const rl = Math.hypot(rx, ry, rz);
      if (rl > 0.14) continue;
      const b = smooth(clamp((t - 0.06) / 0.22, 0, 1));
      const px = S[0] + rx, py = S[1] - t - shift * b, pz = S[2] + rz;
      v[0] += (px - v[0]) * b;
      v[1] += (py - v[1]) * b;
      v[2] += (pz - v[2]) * b;
      moved[vi] = 1;
    }
    // 第二遍：外缘肌肉壳（radial 0.14~0.25）带径向衰减混合（不拽躯干/胸）
    for (let vi = 0; vi < verts.length; vi++) {
      if (moved[vi]) continue;
      const v = verts[vi];
      if (v[1] < handY - 0.10 || v[1] > shoulderY + 0.16) continue;
      if (side * v[0] < 0.19) continue;
      const vx = v[0] - S[0], vy = v[1] - S[1], vz = v[2] - S[2];
      const t = vx * d[0] + vy * d[1] + vz * d[2];
      if (t < -0.10 || t > armLen + 0.12) continue;
      const rx = vx - d[0] * t, ry = vy - d[1] * t, rz = vz - d[2] * t;
      const rl = Math.hypot(rx, ry, rz);
      if (rl < 0.14 || rl > 0.25) continue;
      const damp = smooth((0.25 - rl) / 0.11); // 外缘越远衰减越大，0.25 处归零
      const b = smooth(clamp((t - 0.06) / 0.22, 0, 1)) * damp;
      const px = S[0] + rx, py = S[1] - t - shift * b, pz = S[2] + rz;
      v[0] += (px - v[0]) * b;
      v[1] += (py - v[1]) * b;
      v[2] += (pz - v[2]) * b;
    }
  }

  // ---- 纵向比例 warp（mesh 关键点 → 立绘关键点；含头部内部五官关键点）----
  if (!NO_WARP) {
    const hf = globalThis.__headFeat || { jaw: jawY, mouth: jawY + 0.02, eye: jawY + 0.06, brow: jawY + 0.10 };
    const KEY = [
      [0, 0],
      [ankleY, ART.ankle],
      [crotchY, ART.crotch],
      [shoulderY, ART.shoulder],
      [hf.jaw, ART.chin],
      [hf.mouth, ART.mouth],
      [hf.eye, ART.eye],
      [hf.brow, ART.brow],
      [H, H],
    ].sort((a, b) => a[0] - b[0]);
    const warpY = (y) => {
      if (y <= KEY[0][0]) return KEY[0][1] + y * (KEY[1][1] - KEY[0][1]) / Math.max(1e-6, KEY[1][0] - KEY[0][0]);
      for (let i = 0; i < KEY.length - 1; i++) {
        if (y >= KEY[i][0] && y <= KEY[i + 1][0]) {
          const t = (y - KEY[i][0]) / Math.max(1e-6, KEY[i + 1][0] - KEY[i][0]);
          return KEY[i][1] + t * (KEY[i + 1][1] - KEY[i][1]);
        }
      }
      return y;
    };
    console.log(`warp keypoints: ${KEY.map(k => `${k[0].toFixed(2)}→${k[1].toFixed(2)}`).join(' ')}`);
    for (const v of verts) v[1] = warpY(v[1]);
  }

  const xMinAll = Math.min(...verts.map(v => v[0])), xMaxAll = Math.max(...verts.map(v => v[0]));
  const zMinAll = Math.min(...verts.map(v => v[2])), zMaxAll = Math.max(...verts.map(v => v[2]));
  // ---- 脚底平板（封闭足弓，贴合立绘平底军靴）----
  // 人体底模足弓镂空；补两块薄板（脚底 y≈0.004）封住足底，内部填充才能实心，且底视呈平底靴。
  {
    const base = verts.length;
    const slab = (x0, x1, z0, z1) => {
      verts.push([x0, 0.004, z0], [x1, 0.004, z0], [x1, 0.004, z1], [x0, 0.004, z1]);
      const b = verts.length - 4;
      tris.push([b, b + 1, b + 2], [b, b + 2, b + 3]);
    };
    slab(-0.40, -0.08, -0.09, 0.30);
    slab(0.08, 0.40, -0.09, 0.30);
    void base;
  }
  console.log(`post-surgery x[${xMinAll.toFixed(2)},${xMaxAll.toFixed(2)}] z[${zMinAll.toFixed(2)},${zMaxAll.toFixed(2)}] yMax=${Math.max(...verts.map(v => v[1])).toFixed(2)}`);

  const fAsp = ((fr.bb.right - fr.bb.left) * fr.img.width) / ((fr.bb.bottom - fr.bb.top) * fr.img.height);
  const sAsp = ((sd.bb.right - sd.bb.left) * sd.img.width) / ((sd.bb.bottom - sd.bb.top) * sd.img.height);
  const bAsp = ((bk.bb.right - bk.bb.left) * bk.img.width) / ((bk.bb.bottom - bk.bb.top) * bk.img.height);
  const figW = H * fAsp, figD = H * sAsp, Wb = H * bAsp;
  // 行 runs → u 空间（0..1）与 世界 x 空间
  const rowsU = (runs) => runs.rows.map(rd => rd ? rd.runs.map(r => [r[0] / runs.wn, r[1] / runs.wn]) : null);
  const rowsX = (runs, fw) => runs.rows.map(rd => rd ? rd.runs.map(r => [(r[0] / runs.wn - 0.5) * fw, (r[1] / runs.wn - 0.5) * fw]) : null);
  const frU = rowsU(fr.runs), frX = rowsX(fr.runs, figW);
  const sdU = rowsU(sd.runs), bkU = rowsU(bk.runs).map(row => row ? row.map(r => [1 - r[1], 1 - r[0]]) : null);
  console.log(`figW=${figW.toFixed(3)} figD=${figD.toFixed(3)} Wb=${Wb.toFixed(3)}`);

  // ---- 粗网格体素化（全身，warp 后）----
  const cutY = NO_WARP ? (neckY - 0.05 - SV / 2) : ART.shoulder;  const xMinG = xMinAll - SV, xMaxG = xMaxAll + SV;
  const yMinG = -SV, yMaxG = Math.max(...verts.map(v => v[1])) + SV;
  const zMinG = zMinAll - SV, zMaxG = zMaxAll + SV;
  const nx = Math.ceil((xMaxG - xMinG) / SV), ny = Math.ceil((yMaxG - yMinG) / SV), nz = Math.ceil((zMaxG - zMinG) / SV);
  console.log(`coarse grid ${nx}x${ny}x${nz} cutY=${cutY.toFixed(3)}`);
  const gA = voxelize(verts, tris, SV, xMinG, yMinG, zMinG, nx, ny, nz);
  const bodyCells = [];
  for (let iz = 0; iz < nz; iz++) for (let iy = 0; iy < ny; iy++) for (let ix = 0; ix < nx; ix++) {
    if (!gA.occ[iz * ny * nx + iy * nx + ix]) continue;
    const cy = yMinG + (iy + 0.5) * SV;
    if (cy >= cutY) continue;
    bodyCells.push([ix, iy, iz]);
  }
  console.log(`body cells: ${bodyCells.length}`);

  // ---- 头部细网格（头+颈+三角肌顶带，x∈[-0.24,0.24]）----
  const hx0 = -0.24, hx1 = 0.24;
  const hy0 = cutY, hy1 = Math.max(...verts.map(v => v[1])) + SVH * 2;
  const hz0 = Math.min(...verts.filter(v => v[1] > hy0 - 0.1).map(v => v[2])) - SVH * 2;
  const hz1 = Math.max(...verts.filter(v => v[1] > hy0 - 0.1).map(v => v[2])) + SVH * 2;
  const hnx = Math.ceil((hx1 - hx0) / SVH), hny = Math.ceil((hy1 - hy0) / SVH), hnz = Math.ceil((hz1 - hz0) / SVH);
  console.log(`head grid ${hnx}x${hny}x${hnz} (${hx0},${hy0},${hz0})`);
  const gB = voxelize(verts, tris, SVH, hx0, hy0, hz0, hnx, hny, hnz);
  const headCells = [];
  for (let iz = 0; iz < hnz; iz++) for (let iy = 0; iy < hny; iy++) for (let ix = 0; ix < hnx; ix++) {
    if (gB.occ[iz * hny * hnx + iy * hnx + ix]) headCells.push([ix, iy, iz]);
  }
  console.log(`head cells: ${headCells.length}`);

  const voxels = [];
  for (const [ix, iy, iz] of bodyCells) {
    voxels.push({ x: xMinG + (ix + 0.5) * SV, y: yMinG + (iy + 0.5) * SV, z: zMinG + (iz + 0.5) * SV, size: SV, g: 'A', ix, iy, iz });
  }
  for (const [ix, iy, iz] of headCells) {
    voxels.push({ x: hx0 + (ix + 0.5) * SVH, y: hy0 + (iy + 0.5) * SVH, z: hz0 + (iz + 0.5) * SVH, size: SVH, g: 'B', ix, iy, iz });
  }
  console.log(`total voxels: ${voxels.length}`);

  // ---- 剪影贴合：trim 超出立绘轮廓的体素列 ----
  const removed = new Uint8Array(voxels.length);
  let trimCnt = 0;
  for (let i = 0; i < voxels.length; i++) {
    const vx = voxels[i];
    const v = vOf(fr.bb, vx.y);
    if (v < 0.012 || v > 0.99) continue;
    const row = frX[rowOf(fr.runs, v)];
    if (!row || !row.length) continue;
    const m = vx.size * TRIM_MARGIN;
    const minX = Math.min(...row.map(r => r[0])) - m;
    const maxX = Math.max(...row.map(r => r[1])) + m;
    if (vx.x < minX || vx.x > maxX) { removed[i] = 1; trimCnt++; }
  }
  console.log(`trim removed: ${trimCnt}`);

  // ---- 肩部合并补列 ----
  const added = [];
  const layerMap = new Map();
  for (let i = 0; i < voxels.length; i++) {
    if (removed[i] || voxels[i].g !== 'A') continue;
    const key = Math.round(voxels[i].y / SV);
    if (!layerMap.has(key)) layerMap.set(key, []);
    layerMap.get(key).push(i);
  }
  for (const [key, idxs] of layerMap) {
    const yMid = (key + 0.5) * SV;
    const v = vOf(fr.bb, yMid);
    if (v < FILL_V0 || v > FILL_V1) continue;
    const row = frX[rowOf(fr.runs, v)];
    if (!row || row.length !== 1) continue;
    const cols = new Set(idxs.map(i => Math.round(voxels[i].x / SV)));
    const xMinCol = Math.min(...cols), xMaxCol = Math.max(...cols);
    for (let c = xMinCol + 1; c < xMaxCol; c++) {
      if (cols.has(c)) continue;
      const cxw = (c + 0.5) * SV;
      if (cxw < row[0][0] - SV || cxw > row[0][1] + SV) continue;
      if (!cols.has(c - 1) && !cols.has(c + 1)) continue;
      let zRef = 0.15;
      for (const cand of [c - 1, c + 1]) {
        if (!cols.has(cand)) continue;
        for (const i2 of idxs) if (Math.round(voxels[i2].x / SV) === cand) { zRef = voxels[i2].z; break; }
        if (zRef !== 0.15) break;
      }
      added.push({ x: cxw, y: yMid, z: zRef, size: SV, g: 'A' });
    }
  }
  console.log(`shoulder fill added: ${added.length}`);
  const allVoxels = [];
  for (let i = 0; i < voxels.length; i++) if (!removed[i]) allVoxels.push(voxels[i]);
  for (const a of added) allVoxels.push(a);

  // ---- 部位标注（对称立姿：按层主分量 + x 阈值；避免腋下臂/躯干列合并误标）----
  const partOf = new Array(allVoxels.length).fill('torso');
  // 细网格（头+颈+可能混入的肩顶）：|x|<0.20 → head，否则按侧 → armL/armR
  for (let i = 0; i < allVoxels.length; i++) {
    const vx = allVoxels[i];
    if (vx.g !== 'B') continue;
    partOf[i] = Math.abs(vx.x) < 0.20 ? 'head' : (vx.x < 0 ? 'armL' : 'armR');
  }
  {
    const byLayer = new Map();
    for (let i = 0; i < allVoxels.length; i++) {
      const vx = allVoxels[i];
      if (vx.g !== 'A') continue;
      const key = Math.round(vx.y / SV);
      if (!byLayer.has(key)) byLayer.set(key, []);
      byLayer.get(key).push(i);
    }
    for (const [key, idxs] of byLayer) {
      const yMid = (key + 0.5) * SV;
      // 该层 x 列（臂/腿取列中心 x 即可）
      const colCx = new Map();
      for (const i of idxs) {
        const cx = Math.round(allVoxels[i].x / SV);
        if (!colCx.has(cx)) colCx.set(cx, 0);
        colCx.set(cx, colCx.get(cx) + 1);
      }
      const cols = [...colCx.keys()].sort((a, b) => a - b);
      // 主分量：含 |x| 最小列的连通（按列邻接 Δ=1）
      const colComp = new Array(cols.length).fill(-1);
      const compCols = [];
      let nc = 0;
      for (let k = 0; k < cols.length; k++) {
        if (colComp[k] >= 0) continue;
        const stack = [k]; colComp[k] = nc;
        const members = [];
        while (stack.length) {
          const kk = stack.pop(); members.push(kk);
          for (const d of [1, -1]) {
            const j = kk + d;
            if (j >= 0 && j < cols.length && colComp[j] < 0 && Math.abs(cols[j] - cols[kk]) === 1) { colComp[j] = nc; stack.push(j); }
          }
        }
        compCols.push(members);
        nc++;
      }
      const compCx = compCols.map(m => cols[m.reduce((a, k) => a + k, 0) / m.length]);
      const absCx = compCx.map(c => Math.abs(c));
      const mainIdx = absCx.indexOf(Math.min(...absCx));
      const yPart = yMid > crotchY ? 'torso' : (yMid > ankleY ? 'legs' : 'feet');
      // 逐列赋值（主分量列与 |x| 阈值）
      for (let k = 0; k < cols.length; k++) {
        const isMain = colComp[k] === mainIdx;
        const cxw = (cols[k] + 0.5) * SV;
        let part;
        if (yMid > crotchY) {
          part = isMain ? 'torso' : (Math.abs(cxw) > 0.21 ? (cxw < 0 ? 'armL' : 'armR') : 'torso');
        } else if (yMid > ankleY) {
          part = isMain ? 'legs' : (Math.abs(cxw) > 0.27 ? (cxw < 0 ? 'legL' : 'legR') : 'legs');
        } else {
          part = isMain ? 'feet' : (Math.abs(cxw) > 0.24 ? (cxw < 0 ? 'footL' : 'footR') : 'feet');
        }
        for (const i of idxs) if (Math.round(allVoxels[i].x / SV) === cols[k]) partOf[i] = part;
      }
    }
  }
  const partCnt = {};
  for (const p of partOf) partCnt[p] = (partCnt[p] || 0) + 1;
  console.log('parts:', JSON.stringify(partCnt));

  // ---- 面方向 ----
  function facesFor(vx) {
    const faces = [];
    const occ = vx.g === 'A' ? gA.occ : gB.occ;
    const nxi = vx.g === 'A' ? nx : hnx, nyi = vx.g === 'A' ? ny : hny, nzi = vx.g === 'A' ? nz : hnz;
    const x0i = vx.g === 'A' ? xMinG : hx0, y0i = vx.g === 'A' ? yMinG : hy0, z0i = vx.g === 'A' ? zMinG : hz0;
    const svv = vx.size;
    const ix = Math.round((vx.x - x0i) / svv), iy = Math.round((vx.y - y0i) / svv), iz = Math.round((vx.z - z0i) / svv);
    const at = (a, b, c) => (a < 0 || a >= nxi || b < 0 || b >= nyi || c < 0 || c >= nzi) ? 0 : occ[c * nyi * nxi + b * nxi + a];
    if (!at(ix + 1, iy, iz)) faces.push('+x');
    if (!at(ix - 1, iy, iz)) faces.push('-x');
    if (!at(ix, iy + 1, iz)) faces.push('+y');
    if (!at(ix, iy - 1, iz)) faces.push('-y');
    if (!at(ix, iy, iz + 1)) faces.push('+z');
    if (!at(ix, iy, iz - 1)) faces.push('-z');
    return faces;
  }
  for (let i = 0; i < allVoxels.length; i++) allVoxels[i].faces = facesFor(allVoxels[i]);

  // ---- 取色 ----
  // 头部侧面取色用分层 z 重映射（头 z 区间 → 侧图 run，避免后脑采到脸像素）
  const headZExt = new Map(); // layerKey -> [zMin, zMax]
  for (let i = 0; i < allVoxels.length; i++) {
    const vx = allVoxels[i];
    if (vx.g !== 'B') continue;
    const key = Math.round(vx.y / SVH);
    const e = headZExt.get(key);
    if (e) { if (vx.z < e[0]) e[0] = vx.z; if (vx.z > e[1]) e[1] = vx.z; }
    else headZExt.set(key, [vx.z, vx.z]);
  }
  const zoneOf = (part, v) => {
    if (part === 'head') return v < 0.095 ? 'hair' : (v < 0.115 ? 'forehead' : (v < 0.14 ? 'brow' : (v < 0.16 ? 'eye' : (v < 0.185 ? 'nose' : (v < 0.195 ? 'mouth' : (v < 0.20 ? 'chin' : 'neck'))))));
    if (part === 'torso') return 'shirt';
    if (part === 'armL' || part === 'armR') return v < 0.28 ? 'sleeve' : (v < 0.48 ? 'forearm' : 'fist');
    if (part === 'legL' || part === 'legR') return v < 0.92 ? 'pants' : 'boot';
    if (part === 'footL' || part === 'footR') return 'boot';
    return 'other';
  };
  function clampU(u, row, inset) {
    if (!row || !row.length) return u;
    let best = row[0], bd = 1e9;
    for (const r of row) {
      const d0 = Math.abs(u - r[0]), d1 = Math.abs(u - r[1]);
      const d = Math.min(d0, d1);
      if (d < bd) { bd = d; best = r; }
    }
    const l = best[0] + inset, r2 = best[1] - inset;
    if (l > r2) return (best[0] + best[1]) / 2;
    return u < l ? l : (u > r2 ? r2 : u);
  }
  function sideUSpace(vx) {
    const vS = vOf(sd.bb, vx.y);
    const row = sdU[rowOf(sd.runs, vS)];
    let us = vx.z / figD + 0.5;
    if (row && row.length) {
      let l = row[0][0] + 2 / sd.runs.wn, r2 = row[0][1] - 2 / sd.runs.wn;
      if (vx.g === 'B') {
        // 头：分层 z → run 重映射（后脑=run 左=图左，鼻=run 右=图右）
        const e = headZExt.get(Math.round(vx.y / SVH));
        if (e && e[1] > e[0] + 1e-6) {
          const t = clamp((vx.z - e[0]) / (e[1] - e[0]), 0, 1);
          us = row[0][0] + t * (row[0][1] - row[0][0]);
          return us;
        }
      }
      if (l > r2) us = (row[0][0] + row[0][1]) / 2;
      else us = us < l ? l : (us > r2 ? r2 : us);
    }
    return us;
  }
  function sampleByFace(vx, face) {
    const vF = vOf(fr.bb, vx.y);
    const insetF = 2 / fr.runs.wn;
    if (face === '+z') {
      const u = clampU(vx.x / figW + 0.5, frU[rowOf(fr.runs, vF)], insetF);
      return sample(fr.img, fr.bb, u, vF);
    }
    if (face === '-z') {
      const vB = vOf(bk.bb, vx.y);
      const ub = clampU(0.5 - vx.x / Wb, bkU[rowOf(bk.runs, vB)], 2 / bk.runs.wn);
      return sample(bk.img, bk.bb, ub, vB);
    }
    if (face === '+x' || face === '-x') {
      const vS = vOf(sd.bb, vx.y);
      return sample(sd.img, sd.bb, sideUSpace(vx), vS);
    }
    const u = clampU(vx.x / figW + 0.5, frU[rowOf(fr.runs, vF)], insetF);
    return sample(fr.img, fr.bb, u, vF);
  }
  // 采样点 alpha 置信（用于中位数统计与回退）
  function sampleAlpha(vx, dom, vF) {
    const img = dom === '-z' ? bk.img : (dom === '+x' || dom === '-x') ? sd.img : fr.img;
    const bbx = dom === '-z' ? bk.bb : (dom === '+x' || dom === '-x') ? sd.bb : fr.bb;
    const uu = dom === '-z' ? 0.5 - vx.x / Wb : (dom === '+x' || dom === '-x') ? sideUSpace(vx) : vx.x / figW + 0.5;
    const fx = (bbx.left + clamp(uu, 0, 1) * (bbx.right - bbx.left)) * img.width - 0.5;
    const fy = (bbx.top + vF * (bbx.bottom - bbx.top)) * img.height - 0.5;
    const px = pxAt(img, fx, fy);
    return px ? px[3] : 0;
  }
  // 第一遍：主面取色 + 部位·v区中位数累积（仅置信像素）
  const col = new Array(allVoxels.length).fill(null);
  const zoneSamples = {};
  for (let i = 0; i < allVoxels.length; i++) {
    const vx = allVoxels[i];
    if (!vx.faces.length) continue;
    let dom = null;
    for (const p of PRIORITY) if (vx.faces.includes(p)) { dom = p; break; }
    const c = sampleByFace(vx, dom);
    col[i] = c;
    const vF = vOf(fr.bb, vx.y);
    const key = partOf[i] + ':' + zoneOf(partOf[i], vF);
    if (sampleAlpha(vx, dom, vF) > 100) {
      if (!zoneSamples[key]) zoneSamples[key] = [];
      zoneSamples[key].push(c);
    }
  }
  const zoneMedian = {};
  for (const [k, arr] of Object.entries(zoneSamples)) {
    const med = (ch) => arr.map(c => c[ch]).sort((a, b) => a - b)[Math.floor(arr.length / 2)];
    zoneMedian[k] = [med(0), med(1), med(2)];
  }
  console.log('zone medians:', Object.fromEntries(Object.entries(zoneMedian).map(([k, v]) => [k, '#' + v.map(x => Math.round(x).toString(16).padStart(2, '0')).join('')])));
  // 调试：打印头部正面采样若干
  {
    let n = 0;
    for (let i = 0; i < allVoxels.length && n < 8; i++) {
      const vx = allVoxels[i];
      if (vx.g !== 'B' || !vx.faces.includes('+z')) continue;
      const vF = vOf(fr.bb, vx.y);
      const u = clampU(vx.x / figW + 0.5, frU[rowOf(fr.runs, vF)], 2 / fr.runs.wn);
      const c = sample(fr.img, fr.bb, u, vF);
      console.log(`head +z sample y=${vx.y.toFixed(2)} x=${vx.x.toFixed(2)} u=${u.toFixed(3)} v=${vF.toFixed(3)} #${c.map(x => Math.round(x).toString(16).padStart(2, '0')).join('')}`);
      n++;
    }
  }

  // 第二遍：低置信回退 + 明暗烘焙
  const out = [];
  let nSurface = 0, nFallback = 0;
  for (let i = 0; i < allVoxels.length; i++) {
    const vx = allVoxels[i];
    const faceM = vx.faces.reduce((m, f) => m | FACE_BIT[f], 0);
    let rgb;
    const zoneKey = partOf[i] + ':' + zoneOf(partOf[i], vOf(fr.bb, vx.y));
    if (!vx.faces.length) {
      rgb = zoneMedian[zoneKey] || [120, 120, 120];
    } else {
      let dom = null;
      for (const p of PRIORITY) if (vx.faces.includes(p)) { dom = p; break; }
      let c = col[i];
      const vF = vOf(fr.bb, vx.y);
      const img = dom === '-z' ? bk.img : (dom === '+x' || dom === '-x') ? sd.img : fr.img;
      const bbx = dom === '-z' ? bk.bb : (dom === '+x' || dom === '-x') ? sd.bb : fr.bb;
      const uu = dom === '-z' ? 0.5 - vx.x / Wb : (dom === '+x' || dom === '-x') ? vx.z / figD + 0.5 : vx.x / figW + 0.5;
      const fx = (bbx.left + clamp(uu, 0, 1) * (bbx.right - bbx.left)) * img.width - 0.5;
      const fy = (bbx.top + vF * (bbx.bottom - bbx.top)) * img.height - 0.5;
      const px = pxAt(img, fx, fy);
      if (!px || px[3] <= 100) {
        c = zoneMedian[zoneKey] || c;
        nFallback++;
      }
      const sh = SHADE[dom] * (0.985 + 0.045 * clamp(vF, 0, 1));
      rgb = c.map(x => clamp(Math.round(x * sh), 0, 255));
      nSurface++;
    }
    out.push(Math.round(vx.x * 1000) / 1000, Math.round(vx.y * 1000) / 1000, Math.round(vx.z * 1000) / 1000,
      rgb[0], rgb[1], rgb[2], faceM, vx.g === 'B' ? 1 : 0);
  }
  const meta = {
    h: H, sv: SV, svh: SVH, count: allVoxels.length, surface: nSurface, fallback: nFallback,
    figW: +figW.toFixed(4), figD: +figD.toFixed(4), wb: +Wb.toFixed(4), layers: Math.round(H / SV),
    parts: partCnt,
    landmarks: { neckY: +neckY.toFixed(3), jawY: +jawY.toFixed(3), shoulderY: +shoulderY.toFixed(3), handY: +handY.toFixed(3), crotchY: +crotchY.toFixed(3), ankleY: +ankleY.toFixed(3) },
    colors: Object.fromEntries(Object.entries(zoneMedian).map(([k, v]) => [k, '#' + v.map(x => Math.round(x).toString(16).padStart(2, '0')).join('')]))
  };
  const js = `// voxel_zhenghai_data.js — 郑吒体素模型数据（由 tools/design/gen_voxel_zhenghai_data.mjs 生成）
// 纯颜色体素，无贴图。v 数组每 8 项一个体素：[x, y, z, r, g, b, faceMask, size]
//   x/y/z = 体素中心（世界坐标，总高 ${H}，脚底 y=0）
//   r/g/b = 烘焙后的 sRGB 颜色（已含 MC 面方向明暗）
//   faceMask 位：1=+x 2=-x 4=+y 8=-y 16=+z 32=-z（0=内部体素）
//   size：0=身体体素(边长${SV}) 1=头部细体素(边长${SVH})
window.VoxelZhenghaiData = ${JSON.stringify({ meta, v: out })};
`;
  fs.writeFileSync(OUT, js);
  console.log(`WROTE ${OUT} (${(fs.statSync(OUT).size / 1024).toFixed(1)} KB, ${allVoxels.length} voxels, surface=${nSurface}, fallback=${nFallback})`);
  console.log(`elapsed ${((Date.now() - t0) / 1000).toFixed(1)}s`);
}

main();
