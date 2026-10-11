// CPU voxel OBJ renderer -> multi-view PNG (front/side/back/45x45) with zlib PNG writer.
// Reads a Wavefront OBJ (quad faces, one normal per vertex optional) and writes images.
const fs = require('fs');
const zlib = require('zlib');
const path = require('path');

function parseOBJ(text) {
  const verts = [];
  const faces = []; // each: array of vertex indices (0-based)
  for (const line of text.split(/\r?\n/)) {
    const t = line.trim();
    if (t.startsWith('v ')) {
      const p = t.split(/\s+/).slice(1).map(Number);
      verts.push([p[0], p[1], p[2]]);
    } else if (t.startsWith('f ')) {
      const idx = t.split(/\s+/).slice(1).map((tk) => parseInt(tk.split('/')[0], 10) - 1);
      faces.push(idx);
    }
  }
  return { verts, faces };
}

// face normal for quad/tri
function normal(V, f) {
  const a = V[f[0]], b = V[f[1]], c = V[f[2]];
  const u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
  const v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
  const nx = u[1] * v[2] - u[2] * v[1];
  const ny = u[2] * v[0] - u[0] * v[2];
  const nz = u[0] * v[1] - u[1] * v[0];
  const l = Math.hypot(nx, ny, nz) || 1;
  return [nx / l, ny / l, nz / l];
}

// assign a color to a face based on its normal + world Y position (block part)
const SKIN = [0xF0B98A, 0xD69467];        // skin tones
const HAIR = [0x3C2B1E, 0x000000];        // brown/black
const SHIRT = [0x00A2E8, 0x1FC3FF];       // steve-ish teal shirt
const PANTS = [0x3E4348, 0x444A50];       // indigo/denim
const SHOE = [0x7A5A3B, 0x6B4E33];        // trousers/shoes
const ARM = [0x00A2E8, 0x1FC3FF];

function classifyBox(V, f, n) {
  const ys = f.map((i) => V[i][1]);
  const cy = ys.reduce((a, b) => a + b, 0) / ys.length;
  const xs = f.map((i) => V[i][0]);
  const cx = xs.reduce((a, b) => a + b, 0) / xs.length;
  const zs = f.map((i) => V[i][2]);
  const cz = zs.reduce((a, b) => a + b, 0) / zs.length;
  // determine which axis the face's normal dominates to pick palette
  const absn = [Math.abs(n[0]), Math.abs(n[1]), Math.abs(n[2])];
  const axis = absn.indexOf(Math.max(...absn));
  // height bands (model is ~ / Steve proportions). Assume origin near feet,
  // total height ~ 32 units with head top near 32.
  const height = 28; // nominal
  const relH = cy / 32; // 0..1 from feet
  let part;
  if (relH > 0.78) part = 'head';
  else if (relH > 0.62) part = 'shoulder';
  else if (relH > 0.34) part = 'body';
  else if (relH > 0.12) part = 'pelvis';
  else if (relH > 0.02) part = 'leg';
  else part = 'foot';
  // top faces (axis==1, n[1]>0) of head -> hair
  let shade;
  switch (part) {
    case 'head': shade = axis === 1 && n[1] > 0 ? HAIR[0] : SKIN[0]; break;
    case 'shoulder': shade = ARM[0]; break;
    case 'body': shade = SHIRT[0]; break;
    case 'pelvis': shade = SHIRT[0]; break;
    case 'leg': shade = PANTS[0]; break;
    case 'foot': shade = SHOE[0]; break;
    default: shade = SKIN[0];
  }
  // simple directional lighting by normal orientation (top lighter)
  let light = 1.0;
  if (axis === 1) light = n[1] > 0 ? 1.0 : 0.62;
  else light = 0.82;
  const r = ((shade >> 16) & 255) * light;
  const g = ((shade >> 8) & 255) * light;
  const b = (shade & 255) * light;
  return [Math.round(r), Math.round(g), Math.round(b)];
}

function render({ verts, faces }, view, size = 900) {
  // view: 'front' | 'back' | 'side' | 'diag'
  // model coords: x right, y up, z toward viewer. 
  const cam = (pt) => {
    let [x, y, z] = pt;
    let X, Y, D;
    switch (view) {
      case 'front': X = x; Y = y; D = z; break;
      case 'back': X = -x; Y = y; D = -z; break;
      case 'side': X = -z; Y = y; D = x; break;   // right side view
      case 'diag': { // rotate -45 about Y then ortho front
        const a = -Math.PI / 4;
        const xr = x * Math.cos(a) + z * Math.sin(a);
        const zr = -x * Math.sin(a) + z * Math.cos(a);
        X = xr; Y = y; D = zr; break;
      }
      default: throw new Error('bad view');
    }
    return [X, Y, D];
  };

  // compute bounds
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity, maxD = -Infinity;
  for (const v of verts) {
    const [X, Y, D] = cam(v);
    if (X < minX) minX = X; if (X > maxX) maxX = X;
    if (Y < minY) minY = Y; if (Y > maxY) maxY = Y;
    if (D > maxD) maxD = D;
  }
  const pad = 1;
  const scale = (size - 2 * pad) / Math.max(maxX - minX, maxY - minY);
  const cx = (maxX + minX) / 2, cy = (maxY + minY) / 2;
  const toScreen = ([X, Y]) => [
    Math.round((X - cx) * scale + size / 2),
    Math.round(size / 2 - (Y - cy) * scale),
  ];

  // depth sort faces (painter's algorithm), plus per-face fill
  // precompute projected polygons
  const polys = [];
  for (const f of faces) {
    const n = normal(verts, f);
    // backface cull: if view n dot cam dir <= 0 skip (approx, only for non-diag)
    const drawable = true;
    const proj = f.map((i) => {
      const [X, Y, D] = cam(verts[i]);
      return { X, Y, D };
    });
    const avgD = proj.reduce((a, p) => a + p.D, 0) / proj.length;
    polys.push({ f, proj, avgD, color: classifyBox(verts, f, n) });
  }
  polys.sort((a, b) => b.avgD - a.avgD); // far first

  // rasterize scanline polygon fill
  const img = Buffer.alloc(size * size * 4);
  img.fill(255);          // background RGB white
  for (let i = 3; i < img.length; i += 4) img[i] = 0; // alpha 0 = transparent background
  const put = (x, y, r, g, b) => {
    if (x < 0 || y < 0 || x >= size || y >= size) return;
    const o = (y * size + x) * 4;
    img[o] = r; img[o + 1] = g; img[o + 2] = b; img[o + 3] = 255;
  };
  for (const poly of polys) {
    // build screen-space polygon
    const pts = [];
    for (const p of poly.proj) {
      const s = toScreen([p.X, p.Y]);
      pts.push([s[0], s[1]]);
    }
    // scanline fill
    const minY = Math.max(0, Math.min(...pts.map((p) => p[1])));
    const maxY = Math.min(size - 1, Math.max(...pts.map((p) => p[1])));
    for (let y = Math.floor(minY); y <= Math.ceil(maxY); y++) {
      // compute intersections with edges
      const xs = [];
      for (let i = 0; i < pts.length; i++) {
        const a = pts[i], b = pts[(i + 1) % pts.length];
        if ((a[1] <= y && b[1] > y) || (b[1] <= y && a[1] > y)) {
          const t = (y - a[1]) / (b[1] - a[1]);
          xs.push(a[0] + t * (b[0] - a[0]));
        }
      }
      xs.sort((p, q) => p - q);
      for (let i = 0; i + 1 < xs.length; i += 2) {
        const x0 = Math.max(0, Math.round(xs[i]));
        const x1 = Math.min(size - 1, Math.round(xs[i + 1]));
        for (let x = x0; x <= x1; x++) put(x, y, poly.color[0], poly.color[1], poly.color[2]);
      }
    }
  }
  return img;
}

function writePNG(file, size, img) {
  // raw scanlines (filter 0)
  const stride = size * 4 + 1;
  const raw = Buffer.alloc(stride * size);
  for (let y = 0; y < size; y++) {
    raw[y * stride] = 0;
    img.copy(raw, y * stride + 1, y * size * 4, (y + 1) * size * 4);
  }
  const idat = zlib.deflateSync(raw);
  const chunk = (type, data) => {
    const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
    const td = Buffer.concat([Buffer.from(type), data]);
    const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td) >>> 0);
    return Buffer.concat([len, td, crc]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0); ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  const sig = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  fs.writeFileSync(file, Buffer.concat([sig, chunk('IHDR', ihdr), chunk('IDAT', idat), chunk('IEND', Buffer.alloc(0))]));
}

let crcTable;
function crc32(buf) {
  if (!crcTable) {
    crcTable = new Int32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      crcTable[n] = c;
    }
  }
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = crcTable[(c ^ buf[i]) & 255] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

// ---- main ----
const src = process.argv[2];
const outDir = process.argv[3];
const size = parseInt(process.argv[4] || '900', 10);
fs.mkdirSync(outDir, { recursive: true });
const { verts, faces } = parseOBJ(fs.readFileSync(src, 'utf8'));
console.log(`loaded ${verts.length} verts, ${faces.length} faces`);
for (const view of ['front', 'side', 'back', 'diag']) {
  const img = render({ verts, faces }, view, size);
  const file = path.join(outDir, `${path.basename(src, '.obj')}_${view}.png`);
  writePNG(file, size, img);
  console.log('wrote', file);
}
