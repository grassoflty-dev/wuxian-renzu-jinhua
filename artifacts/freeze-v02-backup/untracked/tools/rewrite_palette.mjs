// rewrite_palette.mjs — 重写 applyPortraitPalette 函数
import fs from 'node:fs';
const file = 'server-rs/ui/js/voxel_people.js';
let src = fs.readFileSync(file, 'utf8');

// Find function start
const startMarker = '// 主染色入口';
const endMarker = '\n\n// 体型映射';
const start = src.indexOf(startMarker);
const end = src.indexOf(endMarker, start);
if (start < 0 || end < 0) { console.error('markers not found', start, end); process.exit(1); }

const newFunc = `// 主染色入口 v5（基于身体段包围盒映射 + 保留原纹理 map）：
//   - 按 body part tag 分组（torso/armL/armR/legL/legR），每段用自己的世界包围盒映射到
//     立绘对应 UV 条带区域（硬编码 UV 分区，基于 analyze_portrait.mjs 实测剪影数据）。
//   - 保留原材质的 .map（布料织纹）不被洗掉，只改 color —— PBR 自然光照处理明暗，去掉假暗化。
//   - T-pose 手臂外展问题：每段独立包围盒 → 局部 x∈[0,1] 映射到立绘袖/臂条带，不会跨到白色背景。
//   - 跳过 AO seam/gear/hair/head（由 excludeHead 控制）。
function applyPortraitPalette(g, url, opts) {
  const o = opts || {};
  if (!g || !url) return Promise.resolve(0);
  return loadPortraitImageData(url).then((img) => {
    if (!img) { g.userData.portraitColored = 0; return 0; }
    let bx0 = img.w, by0 = img.h, bx1 = -1, by1 = -1;
    for (let y = 0; y < img.h; y++) {
      const row = y * img.w * 4;
      for (let x = 0; x < img.w; x++) {
        if (img.data[row + x * 4 + 3] >= 24) {
          if (x < bx0) bx0 = x; if (x > bx1) bx1 = x;
          if (y < by0) by0 = y; if (y > by1) by1 = y;
        }
      }
    }
    if (bx1 < bx0 || by1 < by0) { g.userData.portraitColored = 0; return 0; }
    const fw = bx1 - bx0, fh = by1 - by0;

    g.updateMatrixWorld(true);
    const { parts } = collectBodyMeshes(g, o);

    const matCache = {};
    const makeMat = (oldMat, hex, rough, metal) => {
      const key = hex + "|" + rough + "|" + metal + "|" + (oldMat.map ? oldMat.map.uuid.slice(0,8) : "nomap");
      if (matCache[key]) return matCache[key];
      const m = new THREE.MeshStandardMaterial({
        color: hex, roughness: rough, metalness: metal,
        map: oldMat.map || null,
      });
      matCache[key] = m;
      return m;
    };

    let count = 0;
    const partBounds = (list) => {
      let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
      for (const m of list) {
        if (m.wx < x0) x0 = m.wx; if (m.wx > x1) x1 = m.wx;
        if (m.wy < y0) y0 = m.wy; if (m.wy > y1) y1 = m.wy;
      }
      return { x0, x1, y0, y1, xr: Math.max(1e-6, x1 - x0), yr: Math.max(1e-6, y1 - y0) };
    };
    const paintPart = (list, u0, u1, v0, v1, roughOverride, metalOverride) => {
      if (!list.length) return;
      const b = partBounds(list);
      for (const m of list) {
        const fu = (m.wx - b.x0) / b.xr;
        const fv = 1 - (m.wy - b.y0) / b.yr;
        const u = (bx0 + (u0 + fu * (u1 - u0)) * fw) / img.w;
        const v = (by0 + (v0 + fv * (v1 - v0)) * fh) / img.h;
        const col = samplePortrait(img, u, v);
        if (!col) continue;
        const old = m.mesh.material;
        const rough = (roughOverride != null) ? roughOverride : ((old && typeof old.roughness === "number") ? old.roughness : 0.82);
        const metal = (metalOverride != null) ? metalOverride : ((old && typeof old.metalness === "number") ? Math.min(0.15, old.metalness) : 0.05);
        const hex = (col.r << 16) | (col.g << 8) | col.b;
        m.mesh.material = makeMat(old, hex, rough, metal);
        count++;
      }
    };

    // 立绘 UV 分区（alpha bbox 归一化，v=0头→v=1脚）——基于 analyze_portrait.mjs 实测
    paintPart(parts.torso, 0.18, 0.82, 0.22, 0.54, 0.82, 0.05);   // 躯干/T恤
    paintPart(parts.armL,  0.02, 0.22, 0.20, 0.72, 0.70, 0.02);   // 左臂（袖→手）
    paintPart(parts.armR,  0.78, 0.98, 0.20, 0.72, 0.70, 0.02);   // 右臂
    paintPart(parts.legL,  0.10, 0.44, 0.52, 0.97, 0.85, 0.10);   // 左腿（裤+靴）
    paintPart(parts.legR,  0.56, 0.90, 0.52, 0.97, 0.85, 0.10);   // 右腿

    g.userData.portraitColored = count;
    g.userData.paletteReady = true;
    if (typeof window !== "undefined") window.__paletteReady = true;
    return count;
  });
}`;

src = src.slice(0, start) + newFunc + src.slice(end);
fs.writeFileSync(file, src);
console.log('rewrote applyPortraitPalette, new size:', src.length);
