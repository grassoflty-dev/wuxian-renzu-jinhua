// pnglib.mjs — 最小依赖 PNG 解码 + 立绘取色预处理工具（gen_voxel_zhenghai_data 共用）
// 仅用 node:zlib 解码 8bit RGBA 非隔行 PNG；提供去光晕/腐蚀/连通域/行剪影/采样等
// 与 sprite_ds0731.js 同思路的图像工具（纯 Node 实现，无外部库）。
import zlib from 'node:zlib';

// ---- PNG 解码：返回 { width, height, data: Uint8ClampedArray(RGBA), premultiplied } ----
export function decodePng(buf) {
  if (buf.readUInt32BE(0) !== 0x89504e47) throw new Error('not a PNG');
  let pos = 8;
  let width = 0, height = 0, bitDepth = 0, colorType = 0, interlace = 0;
  const idat = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos);
    const type = buf.toString('ascii', pos + 4, pos + 8);
    if (type === 'IHDR') {
      width = buf.readUInt32BE(pos + 8);
      height = buf.readUInt32BE(pos + 12);
      bitDepth = buf[pos + 16];
      colorType = buf[pos + 17];
      interlace = buf[pos + 20];
    } else if (type === 'IDAT') {
      idat.push(buf.subarray(pos + 8, pos + 8 + len));
    } else if (type === 'IEND') break;
    pos += 12 + len;
  }
  if (bitDepth !== 8 || (colorType !== 6 && colorType !== 2)) {
    throw new Error(`unsupported PNG: bitDepth=${bitDepth} colorType=${colorType}`);
  }
  if (interlace !== 0) throw new Error('interlaced PNG not supported');
  const channels = colorType === 6 ? 4 : 3;
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const out = new Uint8ClampedArray(width * height * 4);
  const prev = new Uint8ClampedArray(stride);
  let p = 0;
  for (let y = 0; y < height; y++) {
    const ft = raw[p++];
    const row = raw.subarray(p, p + stride);
    p += stride;
    const cur = new Uint8ClampedArray(stride);
    for (let i = 0; i < stride; i++) {
      const a = i >= channels ? cur[i - channels] : 0;
      const b = prev[i];
      const c = i >= channels ? prev[i - channels] : 0;
      let v = row[i];
      switch (ft) {
        case 0: break;
        case 1: v = (v + a) & 255; break;
        case 2: v = (v + b) & 255; break;
        case 3: v = (v + ((a + b) >> 1)) & 255; break;
        case 4: {
          const pa = Math.abs(b - c), pb = Math.abs(a - c), pc = Math.abs(a + b - 2 * c);
          const pr = (pa <= pb && pa <= pc) ? a : (pb <= pc ? b : c);
          v = (v + pr) & 255;
          break;
        }
        default: throw new Error('bad filter ' + ft);
      }
      cur[i] = v;
    }
    for (let i = 0; i < stride; i++) prev[i] = cur[i];
    for (let x = 0; x < width; x++) {
      const s = x * channels, d = (y * width + x) * 4;
      out[d] = cur[s];
      out[d + 1] = cur[s + 1];
      out[d + 2] = cur[s + 2];
      out[d + 3] = channels === 4 ? cur[s + 3] : 255;
    }
  }
  return { width, height, data: out };
}

// ---- 反预乘去光晕：半透明边缘还原真实色（sprite dehalo 同思路） ----
export function dehalo(img) {
  const d = img.data, n = img.width * img.height;
  let gr = 0, gg = 0, gb = 0, gc = 0;
  for (let i = 0; i < n; i++) {
    const a = d[i * 4 + 3];
    if (a > 8 && a < 90) { gr += d[i * 4]; gg += d[i * 4 + 1]; gb += d[i * 4 + 2]; gc++; }
  }
  if (gc < 50) return;
  gr /= gc; gg /= gc; gb /= gc;
  for (let j = 0; j < n; j++) {
    const o = j * 4, al = d[o + 3];
    if (al === 0) continue;
    if (al < 60) { d[o + 3] = 0; continue; }
    if (al < 245) {
      const t = al / 255, it = 1 - t;
      d[o] = Math.max(0, Math.min(255, (d[o] - gr * it) / t));
      d[o + 1] = Math.max(0, Math.min(255, (d[o + 1] - gg * it) / t));
      d[o + 2] = Math.max(0, Math.min(255, (d[o + 2] - gb * it) / t));
    }
  }
}

// ---- alpha 腐蚀 rad 轮（去掉描边残留，采样时避免边缘光晕） ----
export function erodeAlpha(img, rad) {
  const d = img.data, W = img.width, H = img.height;
  let a = new Uint8Array(W * H);
  for (let i = 0; i < W * H; i++) a[i] = d[i * 4 + 3];
  for (let r = 0; r < rad; r++) {
    const b = new Uint8Array(W * H);
    for (let y = 0; y < H; y++) {
      for (let x = 0; x < W; x++) {
        const p = y * W + x;
        let m = a[p];
        if (x > 0 && a[p - 1] < m) m = a[p - 1];
        if (x < W - 1 && a[p + 1] < m) m = a[p + 1];
        if (y > 0 && a[p - W] < m) m = a[p - W];
        if (y < H - 1 && a[p + W] < m) m = a[p + W];
        b[p] = m;
      }
    }
    a = b;
  }
  for (let k = 0; k < W * H; k++) d[k * 4 + 3] = a[k];
}

// ---- 连通域清洗：保留最大连通域（及 >=4% 的部件），其余置透明 ----
export function ccClean(img) {
  const d = img.data, W = img.width, H = img.height, N = W * H;
  const thresh = 0.15 * 255;
  const visited = new Uint8Array(N);
  const comps = [];
  for (let i = 0; i < N; i++) {
    if (visited[i] || d[i * 4 + 3] <= thresh) continue;
    const stack = [i]; visited[i] = 1;
    const px = []; let size = 0;
    while (stack.length) {
      const p = stack.pop(); px.push(p); size++;
      const x = p % W, y = (p / W) | 0;
      if (x > 0 && !visited[p - 1] && d[(p - 1) * 4 + 3] > thresh) { visited[p - 1] = 1; stack.push(p - 1); }
      if (x < W - 1 && !visited[p + 1] && d[(p + 1) * 4 + 3] > thresh) { visited[p + 1] = 1; stack.push(p + 1); }
      if (y > 0 && !visited[p - W] && d[(p - W) * 4 + 3] > thresh) { visited[p - W] = 1; stack.push(p - W); }
      if (y < H - 1 && !visited[p + W] && d[(p + W) * 4 + 3] > thresh) { visited[p + W] = 1; stack.push(p + W); }
    }
    comps.push({ size, px });
  }
  let max = 0, mi = -1;
  for (let a = 0; a < comps.length; a++) if (comps[a].size > max) { max = comps[a].size; mi = a; }
  if (comps.length > 0) {
    const keepMin = Math.max(150, max * 0.04);
    for (let b = 0; b < comps.length; b++) {
      if (b === mi || comps[b].size >= keepMin) continue;
      for (const q of comps[b].px) d[q * 4 + 3] = 0;
    }
  }
}

// ---- 剪影 bbox（alpha>15 的包围盒，归一化 0..1） ----
export function detectBBox(img) {
  const d = img.data, W = img.width, H = img.height;
  let minX = W, minY = H, maxX = -1, maxY = -1;
  for (let y = 0; y < H; y++) {
    const row = y * W;
    for (let x = 0; x < W; x++) {
      if (d[(row + x) * 4 + 3] > 15) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
  }
  if (maxX < 0) return null;
  return { left: minX / W, right: (maxX + 1) / W, top: minY / H, bottom: (maxY + 1) / H };
}

// ---- 每行剪影 runs（像素空间，x 相对 bbox 内左缘偏移） ----
// 返回 { wn, hn, rows: [{runs:[[l,r],...], mn, mx}] }，rows 为 null 表示该行无 alpha。
export function rowRuns(img, bb, gap = 3) {
  const d = img.data, W = img.width, H = img.height;
  const x0 = Math.round(bb.left * W), x1 = Math.round(bb.right * W) - 1;
  const y0 = Math.round(bb.top * H), y1 = Math.round(bb.bottom * H) - 1;
  const hn = y1 - y0 + 1;
  const rows = new Array(hn);
  for (let ry = 0; ry < hn; ry++) {
    const py = y0 + ry, rowBase = py * W;
    const runs = [];
    let l = -1, gapCnt = 0;
    for (let x = x0; x <= x1; x++) {
      const on = d[(rowBase + x) * 4 + 3] > 127;
      if (on) {
        if (l < 0) l = x;
        gapCnt = 0;
      } else if (l >= 0) {
        gapCnt++;
        if (gapCnt > gap) {
          if (x - gapCnt - l >= 3) runs.push([l - x0, x - gapCnt - x0]);
          l = -1; gapCnt = 0;
        }
      }
    }
    if (l >= 0 && x1 - l + 1 >= 3) runs.push([l - x0, x1 - x0]);
    let mn = -1, mx = -1;
    for (const rn of runs) { if (mn < 0 || rn[0] < mn) mn = rn[0]; if (rn[1] > mx) mx = rn[1]; }
    rows[ry] = { runs, mn, mx };
  }
  return { wn: x1 - x0 + 1, hn, rows };
}

// ---- 双线性采样（u,v ∈ 0..1 归一化到 bbox 内，返回 [r,g,b]） ----
export function sample(img, bb, u, v) {
  const W = img.width, H = img.height, d = img.data;
  const fx = (bb.left + u * (bb.right - bb.left)) * W - 0.5;
  const fy = (bb.top + v * (bb.bottom - bb.top)) * H - 0.5;
  const x0 = Math.max(0, Math.min(W - 1, Math.floor(fx)));
  const y0 = Math.max(0, Math.min(H - 1, Math.floor(fy)));
  const x1 = Math.min(W - 1, x0 + 1), y1 = Math.min(H - 1, y0 + 1);
  const tx = Math.max(0, Math.min(1, fx - x0)), ty = Math.max(0, Math.min(1, fy - y0));
  const i00 = (y0 * W + x0) * 4, i10 = (y0 * W + x1) * 4, i01 = (y1 * W + x0) * 4, i11 = (y1 * W + x1) * 4;
  const out = [0, 0, 0];
  for (let c = 0; c < 3; c++) {
    const v00 = d[i00 + c], v10 = d[i10 + c], v01 = d[i01 + c], v11 = d[i11 + c];
    out[c] = (v00 * (1 - tx) + v10 * tx) * (1 - ty) + (v01 * (1 - tx) + v11 * tx) * ty;
  }
  return out;
}

// ---- 直接取像素（px,py 为图像坐标；越界返回 null） ----
export function pxAt(img, px, py) {
  const W = img.width, H = img.height;
  const x = Math.floor(px), y = Math.floor(py);
  if (x < 0 || x >= W || y < 0 || y >= H) return null;
  const o = (y * W + x) * 4;
  return [img.data[o], img.data[o + 1], img.data[o + 2], img.data[o + 3]];
}

// ---- 预计算 alpha 掩码（供行级查询） ----
export function alphaMask(img) {
  const d = img.data, N = img.width * img.height;
  const m = new Uint8Array(N);
  for (let i = 0; i < N; i++) m[i] = d[i * 4 + 3] > 127 ? 1 : 0;
  return m;
}
