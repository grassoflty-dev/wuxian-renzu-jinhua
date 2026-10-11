// probe_zhenghai.mjs — 分析人体底模姿势 + 立绘统计（指导体素化参数）
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { decodePng, dehalo, ccClean, erodeAlpha, detectBBox, rowRuns, alphaMask, pxAt } from './pnglib.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..', '..');
const OBJ = path.join(ROOT, 'tools', 'model_task_pkg', 'human_ref', 'makehuman_base.obj');
const TARGET = path.join(ROOT, 'tools', 'model_task_pkg', 'human_ref', 'makehuman_targets', 'universal-male-young-maxmuscle-averageweight.target');
const IMG = path.join(ROOT, 'server-rs', 'ui', 'assets', 'img');

function parseObj(objPath) {
  const lines = fs.readFileSync(objPath, 'utf8').split(/\r?\n/);
  const verts = [];
  const quads = [];
  for (const line of lines) {
    if (line.startsWith('v ')) {
      const t = line.split(/\s+/);
      verts.push([parseFloat(t[1]), parseFloat(t[2]), parseFloat(t[3])]);
    } else if (line.startsWith('f ')) {
      const t = line.split(/\s+/).slice(1);
      const idx = t.map(s => parseInt(s.split('/')[0], 10) - 1);
      quads.push(idx);
    }
  }
  return { verts, quads };
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

const { verts, quads } = parseObj(OBJ);
const tgt = parseTarget(TARGET);
console.log(`OBJ: verts=${verts.length} quads=${quads.length} targetLines=${tgt.length}`);
for (const [i, dx, dy, dz] of tgt) {
  if (i >= 0 && i < verts.length) { verts[i][0] += dx; verts[i][1] += dy; verts[i][2] += dz; }
}
const yMin = Math.min(...verts.map(v => v[1])), yMax = Math.max(...verts.map(v => v[1]));
console.log(`mesh yMin=${yMin.toFixed(3)} yMax=${yMax.toFixed(3)} height=${(yMax - yMin).toFixed(3)}`);

// 姿势分析：按世界高度分层统计 x 外延（臂是否外张）、中央主体宽、前后纵深
const H = 2.3;
const s = H / (yMax - yMin);
const BANDS = 46;
const xMinA = Math.min(...verts.map(v => v[0])), xMaxA = Math.max(...verts.map(v => v[0]));
const zMinA = Math.min(...verts.map(v => v[2])), zMaxA = Math.max(...verts.map(v => v[2]));
console.log(`world scale=${s.toFixed(4)}  x[${(xMinA*s).toFixed(2)},${(xMaxA*s).toFixed(2)}] z[${(zMinA*s).toFixed(2)},${(zMaxA*s).toFixed(2)}]`);
console.log('--- per-band (world y)  max|x| / central halfwidth / z-extent ---');
const bandStat = [];
for (let b = 0; b < BANDS; b++) {
  const y0 = (yMax - (b + 1) * (yMax - yMin) / BANDS), y1 = yMax - b * (yMax - yMin) / BANDS;
  const vs = [];
  for (const v of verts) if (v[1] >= y0 && v[1] <= y1) vs.push(v);
  if (!vs.length) continue;
  const mx = Math.max(...vs.map(v => Math.abs(v[0])));
  const zs = vs.map(v => v[2]);
  const zlo = Math.min(...zs), zhi = Math.max(...zs);
  // 中央主体：x 在 [-0.35,0.35] 世界内的顶点
  const core = vs.filter(v => Math.abs(v[0]) <= 0.35 * 7.4 / s * 0.13 || true); // placeholder
  const ys = (y0 + y1) / 2 * s;
  bandStat.push({ y: ys, mx: mx * s, zlo: zlo * s, zhi: zhi * s, n: vs.length });
}
for (const st of bandStat) {
  console.log(`y=${st.y.toFixed(2)} max|x|=${st.mx.toFixed(2)} z[${st.zlo.toFixed(2)},${st.zhi.toFixed(2)}] n=${st.n}`);
}

// ---- 立绘统计 ----
for (const f of ['pc_zhengzha.png', 'pc_zhengzha_side.png', 'pc_zhengzha_back.png', 'pc_zhengzha_oblique45.png', 'pc_zhengzha_oblique135.png']) {
  const img = decodePng(fs.readFileSync(path.join(IMG, f)));
  dehalo(img); ccClean(img); erodeAlpha(img, 1);
  const bb = detectBBox(img);
  console.log(`\n=== ${f} ${img.width}x${img.height} bb=${JSON.stringify(bb && bb.left.toFixed(3))},${bb && bb.right.toFixed(3)},${bb && bb.top.toFixed(3)},${bb && bb.bottom.toFixed(3)}`);
  if (!bb) continue;
  // 关键高度行的剪影 + 主色
  const rows = rowRuns(img, bb);
  const Wpx = bb.right * img.width - bb.left * img.width;
  const Hpx = bb.bottom * img.height - bb.top * img.height;
  const keyV = [0.03, 0.10, 0.18, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.93, 0.98];
  for (const v of keyV) {
    const ry = Math.max(0, Math.min(rows.hn - 1, Math.floor(v * rows.hn)));
    const rd = rows.rows[ry];
    if (!rd) continue;
    // 该行主色（取该行所有不透明像素的平均）
    const y0 = Math.round(bb.top * img.height) + ry;
    let sr = 0, sg = 0, sb = 0, cnt = 0;
    for (let x = 0; x < img.width; x++) {
      const c = pxAt(img, x, y0);
      if (c && c[3] > 127) { sr += c[0]; sg += c[1]; sb += c[2]; cnt++; }
    }
    const runsStr = rd.runs.map(r => `[${(r[0] / Wpx * 100).toFixed(0)}-${(r[1] / Wpx * 100).toFixed(0)}%]`).join(' ');
    console.log(`v=${v.toFixed(2)} runs:${runsStr || 'none'} w=${(Wpx / Hpx * 100).toFixed(1)}% avgCol=#${cnt ? [sr, sg, sb].map(c => Math.round(c / cnt).toString(16).padStart(2, '0')).join('') : '---'}`);
  }
}
