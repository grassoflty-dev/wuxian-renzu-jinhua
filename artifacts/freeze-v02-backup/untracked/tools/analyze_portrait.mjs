// analyze_portrait.mjs — 分析立绘不透明像素分布，找出身体各部分在 UV 空间的位置
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Read PNG - need a PNG decoder. Use a simple approach: get PNG raw pixels via Canvas? No. Use the existing loadPortraitImageData logic? No, that runs in browser.
// Let's use a pure JS PNG parser. Check if pngjs or similar is available, or write minimal decoder.
// Actually let's use the already-generated render to inspect. For now, let me read the PNG and write to a canvas using a simple approach.
// Simpler: use node:canvaskit? No. Let me use sharp if available, or do it via puppeteer/edge.
// Simpler still: use sync-fetch to load via simple PNG decoder.

// Let's just manually check: load the image data using Node's built-in zlib to decompress PNG.
import zlib from 'node:zlib';

function readPNG(filepath) {
  const buf = fs.readFileSync(filepath);
  // Check signature
  if (buf.slice(0, 8).toString('hex') !== '89504e470d0a1a0a') throw new Error('not PNG');
  let pos = 8;
  const chunks = {};
  let width = 0, height = 0, bitDepth = 0, colorType = 0;
  let idat = Buffer.alloc(0);
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos); pos += 4;
    const type = buf.slice(pos, pos + 4).toString('ascii'); pos += 4;
    const data = buf.slice(pos, pos + len); pos += len;
    pos += 4; // crc
    if (type === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      bitDepth = data[8];
      colorType = data[9];
    } else if (type === 'IDAT') {
      idat = Buffer.concat([idat, data]);
    } else if (type === 'IEND') {
      break;
    }
  }
  const raw = zlib.inflateSync(idat);
  // colorType 6 = RGBA, 8-bit
  const bpp = 4;
  const stride = width * bpp;
  const pixels = Buffer.alloc(stride * height);
  let rpos = 0, wpos = 0;
  const prev = Buffer.alloc(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[rpos++];
    const cur = Buffer.alloc(stride);
    for (let x = 0; x < stride; x++) {
      const filtX = raw[rpos++];
      let a = x >= bpp ? cur[x - bpp] : 0;
      let b = y > 0 ? prev[x] : 0;
      let c = (x >= bpp && y > 0) ? prev[x - bpp] : 0;
      let out;
      switch (filter) {
        case 0: out = filtX; break;
        case 1: out = (filtX + a) & 0xff; break;
        case 2: out = (filtX + b) & 0xff; break;
        case 3: out = (filtX + ((a + b) >> 1)) & 0xff; break;
        case 4: {
          const p = a + b - c;
          const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
          const pr = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
          out = (filtX + pr) & 0xff; break;
        }
        default: out = filtX;
      }
      cur[x] = out;
    }
    cur.copy(pixels, wpos);
    wpos += stride;
    cur.copy(prev);
  }
  return { width, height, pixels };
}

const portrait = path.resolve(__dirname, '..', 'server-rs', 'ui', 'assets', 'img', 'pc_zhengzha.png');
const img = readPNG(portrait);
console.log(`Portrait: ${img.width}x${img.height}`);

// Find alpha bbox
let bx0 = img.width, by0 = img.height, bx1 = -1, by1 = -1;
for (let y = 0; y < img.height; y++) {
  for (let x = 0; x < img.width; x++) {
    const a = img.pixels[(y * img.width + x) * 4 + 3];
    if (a >= 24) {
      if (x < bx0) bx0 = x; if (x > bx1) bx1 = x;
      if (y < by0) by0 = y; if (y > by1) by1 = y;
    }
  }
}
console.log(`Alpha bbox: x=[${bx0},${bx1}] y=[${by0},${by1}]  w=${bx1-bx0} h=${by1-by0}`);

// Analyze per-row opaque pixel column ranges (to see where body/arms/legs are)
function colSpanAtY(y) {
  let l = img.width, r = -1;
  let darkCount = 0;  // 像素偏暗(T恤/裤)的数量
  let skinCount = 0;  // 偏肤色的数量
  for (let x = bx0; x <= bx1; x++) {
    const i = (y * img.width + x) * 4;
    const a = img.pixels[i + 3];
    if (a < 24) continue;
    if (x < l) l = x; if (x > r) x;
    if (x > r) r = x;
    const rC = img.pixels[i], gC = img.pixels[i + 1], bC = img.pixels[i + 2];
    // Simple skin detect: R>G>B and R>100
    if (rC > gC && gC > bC && rC > 120 && rC - bC > 25) skinCount++;
    // Dark cloth: all channels < 100
    else if (rC < 100 && gC < 100 && bC < 100) darkCount++;
  }
  return { l, r, w: r - l, skinCount, darkCount };
}

// Print key horizontal bands
const bands = [
  ['top of head/hair', 0.02],
  ['forehead/eyes', 0.10],
  ['nose/mouth', 0.17],
  ['chin/neck', 0.22],
  ['shoulders', 0.24],
  ['upper chest', 0.30],
  ['mid chest/pecs', 0.36],
  ['lower chest/abs', 0.44],
  ['waist/belt', 0.50],
  ['hip/pants top', 0.55],
  ['upper thigh', 0.62],
  ['mid thigh', 0.68],
  ['knee', 0.74],
  ['shin', 0.82],
  ['ankle/boot', 0.92],
  ['sole', 0.98],
];
console.log('\nHorizontal bands (V in normalized alpha-bbox):');
for (const [name, vv] of bands) {
  const y = Math.round(by0 + vv * (by1 - by0));
  const s = colSpanAtY(y);
  const totalSpan = s.l >= img.width ? 0 : s.w;
  const lNorm = s.l >= img.width ? 0 : (s.l - bx0) / (bx1 - bx0);
  const rNorm = s.r < 0 ? 1 : (s.r - bx0) / (bx1 - bx0);
  const bar = Math.round(totalSpan / (bx1-bx0) * 60);
  console.log(`V=${vv.toFixed(2).padStart(4)} y=${y} ${name.padEnd(18)} u=[${lNorm.toFixed(2)},${rNorm.toFixed(2)}] w=${(totalSpan/(bx1-bx0)).toFixed(2)} ${'█'.repeat(bar)}`);
}

// Also print average column occupation
console.log('\nFull column analysis at key rows (showing where body edges are):');
const keyVs = [0.24, 0.30, 0.44, 0.50, 0.55, 0.68, 0.82, 0.92];
for (const vv of keyVs) {
  const y = Math.round(by0 + vv * (by1 - by0));
  // Find left/right edges per-"segment" by scanning horizontally
  const cols = [];
  let inSeg = false, segL = 0;
  for (let x = bx0; x <= bx1; x++) {
    const a = img.pixels[(y * img.width + x) * 4 + 3];
    const opaque = a >= 24;
    if (opaque && !inSeg) { inSeg = true; segL = x; }
    else if (!opaque && inSeg) { inSeg = false; cols.push([segL, x-1]); }
  }
  if (inSeg) cols.push([segL, bx1]);
  const segs = cols.map(([l, r]) => `u=[${((l-bx0)/(bx1-bx0)).toFixed(2)},${((r-bx0)/(bx1-bx0)).toFixed(2)}] w=${((r-l)/(bx1-bx0)).toFixed(2)}`).join(' | ');
  console.log(`V=${vv.toFixed(2)} y=${y}: segments: ${segs}`);
}
