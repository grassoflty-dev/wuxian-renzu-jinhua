/* zz3d.js — 郑吒「剪影视觉外壳 + 三视图皮肤」3D 角色建模器(ZZ3D 独立通道)
 * 原理：
 *   1) 从正/侧/背三张立绘的 alpha 通道提取逐行剪影：
 *      - 正面：每行若干 x 区间(runs，可分出 手臂/躯干/双腿)；
 *      - 侧面：每行 z 纵深区间(连续轮廓，杜绝百叶窗条纹)；
 *      - 背面：用于背面皮肤的横向映射。
 *   2) 视觉外壳(visual hull)：每行每个 run 生成超椭圆(p≈2.6)截面的 loft 网格，
 *      顶点色按法线方向从三视图立绘投影采样(已膨胀填色，采样无空洞)。
 *   3) 三张「皮肤」网格(正/背/侧×2)按外壳表面位移贴合，立绘贴图 alphaTest 镂空：
 *      正面=立绘像素级还原、侧面=连续侧立绘轮廓、背面=真实后背(去发光边)。
 *   4) 皮肤 FrontSide 单面+外壳包管：从正面镂空缝隙看进去是白底，不穿帮。
 * 依赖全局 THREE(r150)；经 window.ZZ3DBuilder.build(group, opts) 暴露。
 * opts: { url, sideUrl, backUrl, height, rows, segs, depthScale }
 */
(function () {
  "use strict";

  var DEF_BBOX = { left: 228 / 768, right: 540 / 768, top: 40 / 1024, bottom: 1008 / 1024 };

  function srcSize(img) {
    return { w: img.naturalWidth || img.width, h: img.naturalHeight || img.height };
  }

  function loadImg(url, cb, er) {
    var im = new Image();
    im.crossOrigin = "anonymous";
    im.onload = function () { cb(im); };
    im.onerror = er;
    im.src = url;
  }

  function sibling(url, fname) {
    var i = url.lastIndexOf('/');
    return i >= 0 ? url.slice(0, i + 1) + fname : fname;
  }

  // 图像 -> canvas + ImageData
  function toCanvas(src) {
    var s = srcSize(src);
    var c = document.createElement("canvas");
    c.width = s.w; c.height = s.h;
    var ctx = c.getContext("2d");
    ctx.drawImage(src, 0, 0);
    return { c: c, ctx: ctx, id: ctx.getImageData(0, 0, s.w, s.h), w: s.w, h: s.h };
  }

  // 人物包围盒(alpha>15)，失败回退兜底
  function detectBBox(d, W, H) {
    var minX = W, minY = H, maxX = -1, maxY = -1;
    for (var y = 0; y < H; y++) {
      var row = y * W;
      for (var x = 0; x < W; x++) {
        if (d[(row + x) * 4 + 3] > 15) {
          if (x < minX) minX = x;
          if (x > maxX) maxX = x;
          if (y < minY) minY = y;
          if (y > maxY) maxY = y;
        }
      }
    }
    if (maxX >= 0) return { left: minX / W, right: (maxX + 1) / W, top: minY / H, bottom: (maxY + 1) / H };
    return DEF_BBOX;
  }

  // 连通域清洗：保留最大连通域及 >= 其 4% 的部件，其余置透明(去喷墨噪点岛)
  function ccClean(id) {
    var d = id.data, W = id.width, H = id.height;
    var thresh = 0.15 * 255;
    var visited = new Uint8Array(W * H);
    var comps = [];
    for (var i = 0; i < W * H; i++) {
      if (visited[i] || d[i * 4 + 3] <= thresh) continue;
      var stack = [i]; visited[i] = 1;
      var px = [], size = 0;
      while (stack.length) {
        var p = stack.pop(); px.push(p); size++;
        var x = p % W, y = (p / W) | 0;
        if (x > 0)     { var n1 = p - 1; if (!visited[n1] && d[n1 * 4 + 3] > thresh) { visited[n1] = 1; stack.push(n1); } }
        if (x < W - 1) { var n2 = p + 1; if (!visited[n2] && d[n2 * 4 + 3] > thresh) { visited[n2] = 1; stack.push(n2); } }
        if (y > 0)     { var n3 = p - W; if (!visited[n3] && d[n3 * 4 + 3] > thresh) { visited[n3] = 1; stack.push(n3); } }
        if (y < H - 1) { var n4 = p + W; if (!visited[n4] && d[n4 * 4 + 3] > thresh) { visited[n4] = 1; stack.push(n4); } }
      }
      comps.push({ size: size, px: px });
    }
    var max = 0, mi = -1;
    for (var a = 0; a < comps.length; a++) if (comps[a].size > max) { max = comps[a].size; mi = a; }
    if (comps.length > 0) {
      var keepMin = Math.max(150, max * 0.04);
      for (var b = 0; b < comps.length; b++) {
        if (b === mi || comps[b].size >= keepMin) continue;
        var pxs = comps[b].px;
        for (var q = 0; q < pxs.length; q++) d[pxs[q] * 4 + 3] = 0;
      }
    }
  }

  // 去发光边(rim light 白/青晕)：
  //  1) 估计光晕色 = 低 alpha(8~90)像素的平均色；
  //  2) 半透明像素按光晕色做反预乘还原真实色：src = (c - glow*(1-a))/a；
  //  3) a<60 直接透明(光晕主体即低 alpha 区)。
  function dehalo(id) {
    var d = id.data, n = id.width * id.height;
    var gr = 0, gg = 0, gb = 0, gc = 0;
    for (var i = 0; i < n; i++) {
      var a = d[i * 4 + 3];
      if (a > 8 && a < 90) { gr += d[i * 4]; gg += d[i * 4 + 1]; gb += d[i * 4 + 2]; gc++; }
    }
    if (gc < 50) return; // 无明显光晕
    gr /= gc; gg /= gc; gb /= gc;
    for (var j = 0; j < n; j++) {
      var o = j * 4, al = d[o + 3];
      if (al === 0) continue;
      if (al < 60) { d[o + 3] = 0; continue; }
      if (al < 245) {
        var t = al / 255, it = 1 - t;
        d[o]     = Math.max(0, Math.min(255, (d[o]     - gr * it) / t));
        d[o + 1] = Math.max(0, Math.min(255, (d[o + 1] - gg * it) / t));
        d[o + 2] = Math.max(0, Math.min(255, (d[o + 2] - gb * it) / t));
      }
    }
  }

  // 腐蚀 alpha：4 邻域 min 滤波 r 轮(约 r px)，剥掉贴图边缘残留的描边/光晕像素
  function erodeAlpha(id, rad) {
    var d = id.data, W = id.width, H = id.height;
    var a = new Uint8Array(W * H);
    for (var i = 0; i < W * H; i++) a[i] = d[i * 4 + 3];
    for (var r = 0; r < rad; r++) {
      var b = new Uint8Array(W * H);
      for (var y = 0; y < H; y++) {
        for (var x = 0; x < W; x++) {
          var p = y * W + x;
          var m = a[p];
          if (x > 0 && a[p - 1] < m) m = a[p - 1];
          if (x < W - 1 && a[p + 1] < m) m = a[p + 1];
          if (y > 0 && a[p - W] < m) m = a[p - W];
          if (y < H - 1 && a[p + W] < m) m = a[p + W];
          b[p] = m;
        }
      }
      a = b;
    }
    for (var k = 0; k < W * H; k++) d[k * 4 + 3] = a[k];
  }

  // 杀掉「靠近背景的亮像素」(白光描边/腿间辉光)：把透明 mask 膨胀 rad 轮，
  // 范围内 蓝白/中性白(min>bright 且 b>=r-12，避开暖肤色)像素置透明
  function killGlowNearBg(id, rad, bright) {
    var d = id.data, W = id.width, H = id.height;
    var t = new Uint8Array(W * H);
    for (var i = 0; i < W * H; i++) t[i] = d[i * 4 + 3] < 60 ? 1 : 0;
    for (var r = 0; r < rad; r++) {
      var b = new Uint8Array(W * H);
      for (var y = 0; y < H; y++) {
        for (var x = 0; x < W; x++) {
          var p = y * W + x;
          var v = t[p];
          if (!v) {
            if (x > 0 && t[p - 1]) v = 1;
            else if (x < W - 1 && t[p + 1]) v = 1;
            else if (y > 0 && t[p - W]) v = 1;
            else if (y < H - 1 && t[p + W]) v = 1;
          }
          b[p] = v;
        }
      }
      t = b;
    }
    for (var k = 0; k < W * H; k++) {
      if (!t[k]) continue;
      var o = k * 4;
      var mn = Math.min(d[o], d[o + 1], d[o + 2]);
      if (d[o + 3] > 0 && mn > bright && d[o + 2] >= d[o] - 12) d[o + 3] = 0;
    }
  }

  // 去素材内部 rim light 亮线：亮白细线(比 ±5px 邻域亮 >55)压到邻域色
  function deRim(id) {
    var d = id.data, W = id.width, H = id.height;
    var src = new Uint8ClampedArray(d); // 快照
    function op(x, y) { return src[(y * W + x) * 4 + 3] > 40; }
    for (var y = 0; y < H; y++) {
      for (var x = 0; x < W; x++) {
        var o = (y * W + x) * 4;
        if (src[o + 3] <= 40) continue;
        var r0 = src[o], g0 = src[o + 1], b0 = src[o + 2];
        var mn = Math.min(r0, g0, b0);
        // 亮白(mn>185) 或 青色rim(b 显著高于 r)：才处理；且必须偏蓝/中性(避开暖肤色)
        var isWhite = mn > 185, isCyan = b0 > 195 && (b0 - r0) > 35;
        if ((!isWhite && !isCyan) || b0 < r0 - 12) continue;
        var br = (r0 + g0 + b0) / 3;
        // ±5px 四向邻域(取不透明者)
        var nb = [];
        var xs = [x - 5, x + 5], ys = [y - 5, y + 5];
        for (var q = 0; q < 2; q++) {
          var xx = xs[q];
          if (xx >= 0 && xx < W && op(xx, y)) nb.push((y * W + xx) * 4);
          var yy = ys[q];
          if (yy >= 0 && yy < H && op(x, yy)) nb.push((yy * W + x) * 4);
        }
        if (nb.length < 2) continue;
        var dim = 1e9, dr = 0, dg = 0, db = 0;
        for (var s = 0; s < nb.length; s++) {
          var no = nb[s];
          var nbr = (src[no] + src[no + 1] + src[no + 2]) / 3;
          if (nbr < dim) { dim = nbr; dr = src[no]; dg = src[no + 1]; db = src[no + 2]; }
        }
        if (br - dim > 55) { d[o] = dr; d[o + 1] = dg; d[o + 2] = db; }
      }
    }
  }

  // 细节(梯度)驱动身体分割：烟雾/光晕是平滑低对比区,身体有轮廓线/褶皱强边缘。
  // grad>45 → 膨胀5 → 最大连通域 → 填孔 → ∩alpha>100 → 腐蚀1。返回 Uint8Array(1=身体)。
  function detailBodyMask(id) {
    var d = id.data, W = id.width, H = id.height, N = W * H;
    var strong = new Uint8Array(N);
    for (var y = 0; y < H; y++) {
      for (var x = 0; x < W; x++) {
        var p = y * W + x;
        var g0 = (d[p * 4] + d[p * 4 + 1] + d[p * 4 + 2]) / 3;
        var gm = 0;
        if (x < W - 1) { var q1 = p + 1; var g1 = (d[q1 * 4] + d[q1 * 4 + 1] + d[q1 * 4 + 2]) / 3; if (Math.abs(g1 - g0) > gm) gm = Math.abs(g1 - g0); }
        if (y < H - 1) { var q2 = p + W; var g2 = (d[q2 * 4] + d[q2 * 4 + 1] + d[q2 * 4 + 2]) / 3; if (Math.abs(g2 - g0) > gm) gm = Math.abs(g2 - g0); }
        if (gm > 45) strong[p] = 1;
      }
    }
    // 二值膨胀 5
    var m = strong;
    for (var r = 0; r < 5; r++) {
      var b = new Uint8Array(N);
      for (var y2 = 0; y2 < H; y2++) {
        for (var x2 = 0; x2 < W; x2++) {
          var p2 = y2 * W + x2;
          var v = m[p2];
          if (!v) {
            if (x2 > 0 && m[p2 - 1]) v = 1;
            else if (x2 < W - 1 && m[p2 + 1]) v = 1;
            else if (y2 > 0 && m[p2 - W]) v = 1;
            else if (y2 < H - 1 && m[p2 + W]) v = 1;
          }
          b[p2] = v;
        }
      }
      m = b;
    }
    // 最大连通域
    var visited = new Uint8Array(N), best = null, bestSize = 0;
    for (var i = 0; i < N; i++) {
      if (!m[i] || visited[i]) continue;
      var stack = [i]; visited[i] = 1;
      var px = [], size = 0;
      while (stack.length) {
        var p3 = stack.pop(); px.push(p3); size++;
        var x3 = p3 % W, y3 = (p3 / W) | 0;
        if (x3 > 0)     { var n1 = p3 - 1; if (m[n1] && !visited[n1]) { visited[n1] = 1; stack.push(n1); } }
        if (x3 < W - 1) { var n2 = p3 + 1; if (m[n2] && !visited[n2]) { visited[n2] = 1; stack.push(n2); } }
        if (y3 > 0)     { var n3 = p3 - W; if (m[n3] && !visited[n3]) { visited[n3] = 1; stack.push(n3); } }
        if (y3 < H - 1) { var n4 = p3 + W; if (m[n4] && !visited[n4]) { visited[n4] = 1; stack.push(n4); } }
      }
      if (size > bestSize) { bestSize = size; best = px; }
    }
    var mask = new Uint8Array(N);
    if (best) for (var k = 0; k < best.length; k++) mask[best[k]] = 1;
    // 填孔：从边界洪泛标记外部,孔(未到达的 0)变 1
    var ext = new Uint8Array(N), q = [];
    for (var x4 = 0; x4 < W; x4++) {
      if (!mask[x4] && !ext[x4]) { ext[x4] = 1; q.push(x4); }
      var pb = (H - 1) * W + x4;
      if (!mask[pb] && !ext[pb]) { ext[pb] = 1; q.push(pb); }
    }
    for (var y5 = 0; y5 < H; y5++) {
      var pl = y5 * W, pr = y5 * W + W - 1;
      if (!mask[pl] && !ext[pl]) { ext[pl] = 1; q.push(pl); }
      if (!mask[pr] && !ext[pr]) { ext[pr] = 1; q.push(pr); }
    }
    while (q.length) {
      var p5 = q.pop();
      var x5 = p5 % W, y6 = (p5 / W) | 0;
      if (x5 > 0)     { var m1 = p5 - 1; if (!mask[m1] && !ext[m1]) { ext[m1] = 1; q.push(m1); } }
      if (x5 < W - 1) { var m2 = p5 + 1; if (!mask[m2] && !ext[m2]) { ext[m2] = 1; q.push(m2); } }
      if (y6 > 0)     { var m3 = p5 - W; if (!mask[m3] && !ext[m3]) { ext[m3] = 1; q.push(m3); } }
      if (y6 < H - 1) { var m4 = p5 + W; if (!mask[m4] && !ext[m4]) { ext[m4] = 1; q.push(m4); } }
    }
    for (var j = 0; j < N; j++) if (!ext[j]) mask[j] = 1;
    // ∩ alpha>100
    for (var j2 = 0; j2 < N; j2++) if (d[j2 * 4 + 3] <= 100) mask[j2] = 0;
    // 腐蚀 1
    var e = new Uint8Array(N);
    for (var y7 = 0; y7 < H; y7++) {
      for (var x7 = 0; x7 < W; x7++) {
        var p7 = y7 * W + x7;
        var v2 = mask[p7];
        if (v2) {
          if (x7 > 0 && !mask[p7 - 1]) v2 = 0;
          else if (x7 < W - 1 && !mask[p7 + 1]) v2 = 0;
          else if (y7 > 0 && !mask[p7 - W]) v2 = 0;
          else if (y7 < H - 1 && !mask[p7 + W]) v2 = 0;
        }
        e[p7] = v2;
      }
    }
    return e;
  }

  // 中值滤波 alpha(3x3 多数表决)：抚平噪声毛刺边
  function medianAlpha(id) {
    var d = id.data, W = id.width, H = id.height;
    var a = new Uint8Array(W * H), b = new Uint8Array(W * H);
    for (var i = 0; i < W * H; i++) a[i] = d[i * 4 + 3] > 0 ? 1 : 0;
    for (var y = 0; y < H; y++) {
      for (var x = 0; x < W; x++) {
        var cnt = 0;
        for (var dy = -1; dy <= 1; dy++) {
          var yy = y + dy; if (yy < 0 || yy >= H) continue;
          for (var dx = -1; dx <= 1; dx++) {
            var xx = x + dx; if (xx < 0 || xx >= W) continue;
            cnt += a[yy * W + xx];
          }
        }
        b[y * W + x] = cnt >= 5 ? 255 : 0;
      }
    }
    for (var k = 0; k < W * H; k++) d[k * 4 + 3] = b[k];
  }

  // rim 填充：贴边 rad px 内的 rim 像素(蓝白 或 任何高亮,min>120)，
  // 用最亮的非 rim 内部邻域色逐轮填充(rounds 轮,每轮 1px)——改色不改 alpha,不留洞、不搅浑色
  function inpaintRim(id, rad, rounds) {
    var d = id.data, W = id.width, H = id.height, N = W * H;
    var t = new Uint8Array(N);
    for (var i = 0; i < N; i++) t[i] = d[i * 4 + 3] < 60 ? 1 : 0;
    for (var r = 0; r < rad; r++) {
      var b = new Uint8Array(N);
      for (var y = 0; y < H; y++) {
        for (var x = 0; x < W; x++) {
          var p = y * W + x;
          var v = t[p];
          if (!v) {
            if (x > 0 && t[p - 1]) v = 1;
            else if (x < W - 1 && t[p + 1]) v = 1;
            else if (y > 0 && t[p - W]) v = 1;
            else if (y < H - 1 && t[p + W]) v = 1;
          }
          b[p] = v;
        }
      }
      t = b;
    }
    var rim = new Uint8Array(N);
    for (var k = 0; k < N; k++) {
      if (!t[k]) continue;
      var o = k * 4;
      if (d[o + 3] <= 0) continue;
      var mn = Math.min(d[o], d[o + 1], d[o + 2]);
      var br = (d[o] + d[o + 1] + d[o + 2]) / 3;
      // 蓝白 rim 或 极亮(br>215) warm rim；皮肤肌肉高光(br≈200)保留
      if (mn > 120 && (d[o + 2] >= d[o] - 10 || br > 215)) rim[k] = 1;
    }
    var filled = new Uint8Array(N);
    for (var rd = 0; rd < rounds; rd++) {
      var any = false;
      var mark = new Uint8Array(N);
      var fr = new Uint8Array(N), fg = new Uint8Array(N), fb = new Uint8Array(N);
      for (var y2 = 0; y2 < H; y2++) {
        for (var x2 = 0; x2 < W; x2++) {
          var p2 = y2 * W + x2;
          if (!rim[p2] || filled[p2]) continue;
          var cands = [];
          for (var dy = -1; dy <= 1; dy++) {
            var yy = y2 + dy; if (yy < 0 || yy >= H) continue;
            for (var dx = -1; dx <= 1; dx++) {
              var xx = x2 + dx; if (xx < 0 || xx >= W) continue;
              var q2 = yy * W + xx;
              var ok = (d[q2 * 4 + 3] > 0 && !rim[q2]) || (rim[q2] && filled[q2]);
              if (!ok) continue;
              cands.push({ br: (d[q2 * 4] + d[q2 * 4 + 1] + d[q2 * 4 + 2]) / 3, o: q2 * 4 });
            }
          }
          var bo = cands.length ? 1 : -1;
          if (bo >= 0) {
            // 取候选邻域亮度的中位数(避免"平均搅浑"与"最亮洗白")
            cands.sort(function (a, b2) { return a.br - b2.br; });
            var mid = cands[cands.length >> 1];
            fr[p2] = d[mid.o]; fg[p2] = d[mid.o + 1]; fb[p2] = d[mid.o + 2];
            mark[p2] = 1; any = true;
          }
        }
      }
      for (var k2 = 0; k2 < N; k2++) {
        if (mark[k2]) { d[k2 * 4] = fr[k2]; d[k2 * 4 + 1] = fg[k2]; d[k2 * 4 + 2] = fb[k2]; filled[k2] = 1; }
      }
      if (!any) break;
    }
  }

  // 软压高光：c>th 的通道压缩为 th+(c-th)*f —— 把素材画进去的白色 rim light 压暗成浅灰蓝
  function softClamp(id, th, f) {
    var d = id.data;
    for (var i = 0; i < id.width * id.height; i++) {
      var o = i * 4;
      if (!d[o + 3]) continue;
      for (var c = 0; c < 3; c++) {
        if (d[o + c] > th) d[o + c] = th + (d[o + c] - th) * f;
      }
    }
  }

  // 硬切 alpha：a<th → 0
  function hardCut(id, th) {
    var d = id.data;
    for (var i = 0; i < id.width * id.height; i++) if (d[i * 4 + 3] < th) d[i * 4 + 3] = 0;
  }

  // 颜色膨胀填色：把透明像素填上最近不透明邻域色(迭代 6 轮)，供顶点色采样无空洞。
  // 返回 { d:Uint8ClampedArray(仅颜色可靠), W, H }（alpha 不变，颜色被扩散）
  function dilateColors(id, rounds) {
    var d = id.data, W = id.width, H = id.height;
    var solid = new Uint8Array(W * H);
    for (var i = 0; i < W * H; i++) solid[i] = d[i * 4 + 3] > 40 ? 1 : 0;
    for (var r = 0; r < rounds; r++) {
      var filled = 0;
      var mark = new Uint8Array(W * H);
      for (var y = 0; y < H; y++) {
        for (var x = 0; x < W; x++) {
          var p = y * W + x;
          if (solid[p]) continue;
          var cnt = 0, sr = 0, sg = 0, sb = 0;
          for (var dy = -1; dy <= 1; dy++) {
            var yy = y + dy; if (yy < 0 || yy >= H) continue;
            for (var dx = -1; dx <= 1; dx++) {
              var xx = x + dx; if (xx < 0 || xx >= W) continue;
              var q = yy * W + xx;
              if (solid[q]) { sr += d[q * 4]; sg += d[q * 4 + 1]; sb += d[q * 4 + 2]; cnt++; }
            }
          }
          if (cnt > 0) {
            d[p * 4] = sr / cnt; d[p * 4 + 1] = sg / cnt; d[p * 4 + 2] = sb / cnt;
            mark[p] = 1; filled++;
          }
        }
      }
      for (var k = 0; k < W * H; k++) if (mark[k]) solid[k] = 1;
      if (!filled) break;
    }
    return { d: d, W: W, H: H };
  }

  // 逐行剪影提取(在人物 bbox 内)：
  //  mode 'x': 返回 runs[py] = [[l,r]...]（bbox 内像素坐标，含端点）
  //  mode 'z': 返回 range[py] = [min,max]（bbox 内像素坐标）
  function extractRows(imgData, bb, mode) {
    var d = imgData.data, W = imgData.width;
    var x0 = Math.round(bb.left * W), x1 = Math.round(bb.right * W) - 1;
    var y0 = Math.round(bb.top * imgData.height), y1 = Math.round(bb.bottom * imgData.height) - 1;
    var Hn = y1 - y0 + 1;
    var out = new Array(Hn);
    for (var ry = 0; ry < Hn; ry++) {
      var py = y0 + ry, rowBase = py * W;
      if (mode === 'x') {
        var runs = [], l = -1, gap = 0;
        for (var x = x0; x <= x1; x++) {
          var on = d[(rowBase + x) * 4 + 3] > 127;
          if (on) {
            if (l < 0) l = x;
            gap = 0;
          } else if (l >= 0) {
            gap++;
            if (gap > 2) { // 允许 <=2px 小缝
              if (x - gap - l >= 3) runs.push([l - x0, x - gap - x0]);
              l = -1; gap = 0;
            }
          }
        }
        if (l >= 0 && x1 - l + 1 >= 3) runs.push([l - x0, x1 - x0]);
        out[ry] = runs.length ? runs : null;
      } else {
        var mn = -1, mx = -1;
        for (var x2 = x0; x2 <= x1; x2++) {
          if (d[(rowBase + x2) * 4 + 3] > 127) { if (mn < 0) mn = x2; mx = x2; }
        }
        out[ry] = mn >= 0 ? [mn - x0, mx - x0] : null;
      }
    }
    return { rows: out, wn: (x1 - x0 + 1), hn: Hn };
  }

  function spow(a, b) { return (a < 0 ? -1 : 1) * Math.pow(Math.abs(a), b); }

  // 双线性采样(已膨胀色板)，u,v∈[0,1] bbox 内
  function sample(map, bb, u, v) {
    var W = map.W, H = map.H, d = map.d;
    var fx = (bb.left + u * (bb.right - bb.left)) * W - 0.5;
    var fy = (bb.top + v * (bb.bottom - bb.top)) * H - 0.5;
    var x0 = Math.max(0, Math.min(W - 1, Math.floor(fx)));
    var y0 = Math.max(0, Math.min(H - 1, Math.floor(fy)));
    var x1 = Math.min(W - 1, x0 + 1), y1 = Math.min(H - 1, y0 + 1);
    var tx = Math.max(0, Math.min(1, fx - x0)), ty = Math.max(0, Math.min(1, fy - y0));
    var i00 = (y0 * W + x0) * 4, i10 = (y0 * W + x1) * 4, i01 = (y1 * W + x0) * 4, i11 = (y1 * W + x1) * 4;
    var out = [0, 0, 0];
    for (var c = 0; c < 3; c++) {
      var v00 = d[i00 + c], v10 = d[i10 + c], v01 = d[i01 + c], v11 = d[i11 + c];
      out[c] = (v00 * (1 - tx) + v10 * tx) * (1 - ty) + (v01 * (1 - tx) + v11 * tx) * ty;
    }
    return out;
  }

  function build(group, opts) {
    opts = opts || {};
    var url = opts.url ||
      ((typeof document !== "undefined") ? new URL("assets/pc_zhengzha_front.png", document.baseURI).href : "assets/pc_zhengzha_front.png");
    var urlSide = opts.sideUrl || sibling(url, 'pc_zhengzha_side.png');
    var urlBack = opts.backUrl || sibling(url, 'pc_zhengzha_back.png');
    var H = opts.height != null ? opts.height : 2.3;
    var NR = opts.rows || 260;      // 网格行数
    var NS = opts.segs || 48;       // 截面分段
    var PE = 2.6;                   // 超椭圆指数
    var depthScale = opts.depthScale != null ? opts.depthScale : 1.0;

    // 清空旧子件
    for (var k = group.children.length - 1; k >= 0; k--) {
      var o = group.children[k];
      if (o.geometry) o.geometry.dispose();
      if (o.material) {
        (Array.isArray(o.material) ? o.material : [o.material]).forEach(function (m) {
          if (m.map) m.map.dispose();
          m.dispose();
        });
      }
      group.remove(o);
    }
    window.__modelReady = false;

    var res = { front: null, side: null, back: null }, pending = 3;
    function maybe() { if (--pending <= 0) assemble(); }

    loadImg(url, function (im) {
      try {
        var t = toCanvas(im);
        ccClean(t.id);
        t.ctx.putImageData(t.id, 0, 0);
        var bb = detectBBox(t.id.data, t.w, t.h);
        res.front = { canvas: t.c, id: t.id, bb: bb, map: dilateColors(t.id, 6) };
      } catch (e) { console.error('front prep fail', e); }
      maybe();
    }, function () { maybe(); });

    function prepSideBack(u, key) {
      loadImg(u, function (im) {
        try {
          var t = toCanvas(im);
          // 身体分割流水线(实测素材:黑雾近黑 alpha<200;背面另有 alpha=255 蓝灰烟雾带)：
          // 反预乘 → 硬切200 → 梯度细节身体分割(烟雾平滑无边缘被排除) → 连通域去噪
          // → 腐蚀2 → 中值抚平 → rim 填充16 → 软压高光 → 去内部亮线
          dehalo(t.id);
          hardCut(t.id, 200);
          var bm = detailBodyMask(t.id);   // 只保留人物身体
          for (var bi = 0; bi < t.w * t.h; bi++) if (!bm[bi]) t.id.data[bi * 4 + 3] = 0;
          ccClean(t.id);
          erodeAlpha(t.id, 2);
          medianAlpha(t.id);
          inpaintRim(t.id, 16, 16);        // 白/青 rim 用内部色填充
          softClamp(t.id, 208, 0.18);
          deRim(t.id);
          t.ctx.putImageData(t.id, 0, 0);
          var bb = detectBBox(t.id.data, t.w, t.h);
          res[key] = { canvas: t.c, id: t.id, bb: bb, map: dilateColors(t.id, 6) };
        } catch (e) { console.error(key + ' prep fail', e); }
        maybe();
      }, function () { maybe(); });
    }
    prepSideBack(urlSide, 'side');
    prepSideBack(urlBack, 'back');

    function assemble() {
      var fr = res.front;
      if (!fr || !res.side) { window.__modelReady = true; return; } // 缺素材则放弃(正面/侧面必需)
      var sd = res.side, bk = res.back;

      // 世界尺寸(等高 H)
      var fbb = fr.bb, sbb = sd.bb;
      var fAsp = ((fbb.right - fbb.left) * fr.id.width) / ((fbb.bottom - fbb.top) * fr.id.height);
      var sAsp = ((sbb.right - sbb.left) * sd.id.width) / ((sbb.bottom - sbb.top) * sd.id.height);
      var Wf = H * fAsp, Ws = H * sAsp * depthScale;
      var Wb = Wf;
      if (bk) {
        var bbb = bk.bb;
        var bAsp = ((bbb.right - bbb.left) * bk.id.width) / ((bbb.bottom - bbb.top) * bk.id.height);
        Wb = H * bAsp;
      }

      // 逐行剪影(像素行)
      var fx = extractRows(fr.id, fbb, 'x');
      var sz = extractRows(sd.id, sbb, 'z');
      var bxRows = bk ? extractRows(bk.id, bk.bb, 'z') : null;
      var figW = Wf, figD = Ws;
      var inF = 10 / fx.wn, inS = 10 / sz.wn, inB = bxRows ? 10 / bxRows.wn : 0;

      // 取第 i 网格行(0..NR-1, 顶->底)的 runs / zrange，世界坐标
      function rowData(i) {
        var v = (i + 0.5) / NR;
        var y = H * (1 - v);
        var fi = Math.min(fx.hn - 1, Math.max(0, Math.round(v * (fx.hn - 1))));
        var si = Math.min(sz.hn - 1, Math.max(0, Math.round(v * (sz.hn - 1))));
        var runsPx = fx.rows[fi];
        var zr = sz.rows[si];
        // zrange 缺失时向上下借
        if (!zr) {
          for (var off = 1; off < 12 && !zr; off++) {
            zr = sz.rows[Math.min(sz.hn - 1, si + off)] || sz.rows[Math.max(0, si - off)];
          }
        }
        if (!runsPx || !zr) return null;
        // 背面逐行 x 范围(bbox u 坐标,供顶点色采样夹取) + 世界坐标范围(收外壳/背皮肤裁剪)
        var backN = null, backW = null;
        if (bxRows) {
          var bi2 = Math.min(bxRows.hn - 1, Math.max(0, Math.round(v * (bxRows.hn - 1))));
          var br = bxRows.rows[bi2];
          if (!br) {
            for (var off2 = 1; off2 < 12 && !br; off2++) {
              br = bxRows.rows[Math.min(bxRows.hn - 1, bi2 + off2)] || bxRows.rows[Math.max(0, bi2 - off2)];
            }
          }
          if (br) {
            backN = [br[0] / bxRows.wn, br[1] / bxRows.wn];
            // back bbox u=0(图像左) ↔ 世界 +X
            backW = [(0.5 - backN[1]) * Wb, (0.5 - backN[0]) * Wb];
          }
        }
        var z0 = (zr[0] / sz.wn - 0.5) * figD, z1 = (zr[1] / sz.wn - 0.5) * figD;
        var zc = (z0 + z1) / 2, zrad = Math.max(0.012, (z1 - z0) / 2) * 0.99; // 略缩:外壳收进皮肤内
        var mW = backW ? 2 / bxRows.wn * Wb : 0; // 2px 世界边距
        var runs = [];
        for (var q = 0; q < runsPx.length; q++) {
          var l = (runsPx[q][0] / fx.wn - 0.5) * figW;
          var r = (runsPx[q][1] / fx.wn - 0.5) * figW;
          // 与背面剪影求交:外壳 x 永远不超出背面人物(防止背视角露出暗色"回声"裙边)
          if (backW) {
            var l2 = Math.max(l, backW[0] + mW), r2 = Math.min(r, backW[1] - mW);
            if (r2 - l2 >= 0.004) { l = l2; r = r2; }
          }
          runs.push({ l: l, r: r, xc: (l + r) / 2, xr: Math.max(0.008, (r - l) / 2) * 0.99 });
        }
        // 手臂/手部细 run 纵深瘦身(腿不瘦：两腿宽度相近时都不瘦)
        if (runs.length >= 2) {
          var sorted = runs.slice().sort(function (a, b) { return (b.r - b.l) - (a.r - a.l); });
          var widest = sorted[0], second = sorted[1];
          var isLegs = runs.length === 2 && (second.r - second.l) >= 0.5 * (widest.r - widest.l);
          if (!isLegs) {
            for (var m = 0; m < runs.length; m++) {
              if (runs[m] !== widest) { runs[m].zrad = zrad * 0.6; } else { runs[m].zrad = zrad; }
              runs[m].zc = zc;
            }
          } else {
            for (var n = 0; n < runs.length; n++) { runs[n].zrad = zrad; runs[n].zc = zc; }
          }
        } else {
          runs[0].zrad = zrad; runs[0].zc = zc;
        }
        return {
          v: v, y: y, runs: runs, zc: zc, zrad: zrad,
          frontN: [runsPx[0][0] / fx.wn, runsPx[runsPx.length - 1][1] / fx.wn],
          sideN: [zr[0] / sz.wn, zr[1] / sz.wn],
          backN: backN, backW: backW
        };
      }

      /* ---------- ① 外壳 loft 网格(顶点色) ---------- */
      var pos = [], col = [], idx = [];
      var ringInfo = []; // 每行: [{base, run}...] 供皮肤查询
      var prevRow = null; // [{run, ringStart}]
      var vertCount = 0;

      function addRing(rd, ri) {
        var rings = [];
        for (var q = 0; q < rd.runs.length; q++) {
          var rn = rd.runs[q];
          var start = vertCount;
          for (var j = 0; j < NS; j++) {
            var th = 2 * Math.PI * j / NS;
            var cx = spow(Math.cos(th), 2 / PE), sz2 = spow(Math.sin(th), 2 / PE);
            pos.push(rn.xc + rn.xr * cx, rd.y, rn.zc + rn.zrad * sz2);
            col.push(0.8, 0.8, 0.8); // 占位，后填
            vertCount++;
          }
          rings.push({ run: rn, ringStart: start, y: rd.y, v: rd.v });
        }
        return rings;
      }

      function strip(r0, r1) {
        // r0 上(高 y), r1 下(低 y)；三角 (a,b,c),(b,d,c)
        for (var j = 0; j < NS; j++) {
          var j1 = (j + 1) % NS;
          var a = r0.ringStart + j, b = r0.ringStart + j1;
          var c = r1.ringStart + j, d = r1.ringStart + j1;
          idx.push(a, b, c, b, d, c);
        }
      }

      function cap(ring, isTop) {
        var ci = vertCount;
        pos.push(ring.run.xc, ring.y, ring.run.zc);
        col.push(0.8, 0.8, 0.8);
        vertCount++;
        for (var j = 0; j < NS; j++) {
          var j1 = (j + 1) % NS;
          if (isTop) idx.push(ci, ring.ringStart + j1, ring.ringStart + j);
          else idx.push(ci, ring.ringStart + j, ring.ringStart + j1);
        }
      }

      function overlap(rA, rB) {
        var l = Math.max(rA.l, rB.l), r = Math.min(rA.r, rB.r);
        return r - l;
      }

      var rowsCache = [];
      for (var i = 0; i < NR; i++) rowsCache.push(rowData(i));

      for (var i2 = 0; i2 < NR; i2++) {
        var rd = rowsCache[i2];
        if (!rd) { prevRow = null; continue; }
        var rings = addRing(rd, i2);
        ringInfo.push({ y: rd.y, v: rd.v, runs: rd.runs });
        if (prevRow) {
          // 匹配上一行 runs(按区间重叠)
          var used = {};
          for (var q2 = 0; q2 < rings.length; q2++) {
            var best = -1, bestOv = 0;
            for (var p2 = 0; p2 < prevRow.length; p2++) {
              if (used[p2]) continue;
              var ov = overlap(rings[q2].run, prevRow[p2].run);
              if (ov > bestOv) { bestOv = ov; best = p2; }
            }
            if (best >= 0 && bestOv > 0) { used[best] = 1; strip(prevRow[best], rings[q2]); }
            else cap(rings[q2], true); // 新出现(分叉) -> 顶帽
          }
          for (var p3 = 0; p3 < prevRow.length; p3++) if (!used[p3]) cap(prevRow[p3], false); // 消失(合并) -> 底帽
        } else {
          for (var q3 = 0; q3 < rings.length; q3++) cap(rings[q3], true);
        }
        prevRow = rings;
      }
      if (prevRow) for (var q4 = 0; q4 < prevRow.length; q4++) cap(prevRow[q4], false);

      var hullGeo = new THREE.BufferGeometry();
      hullGeo.setAttribute('position', new THREE.Float32BufferAttribute(pos, 3));
      hullGeo.setAttribute('color', new THREE.Float32BufferAttribute(col, 3));
      hullGeo.setIndex(idx);
      hullGeo.computeVertexNormals();

      // 顶点色：按法线方向从三视图投影采样(夹到各视图剪影内缩 3px,防止外壳边缘采到轮廓线/描边色)
      var nrm = hullGeo.getAttribute('normal');
      var posA = hullGeo.getAttribute('position');
      var colA = hullGeo.getAttribute('color');
      function clampR(u, rg, inset) {
        if (!rg) return u;
        var l = rg[0] + inset, r = rg[1] - inset;
        if (l > r) return (rg[0] + rg[1]) / 2;
        return u < l ? l : (u > r ? r : u);
      }
      for (var vi = 0; vi < posA.count; vi++) {
        var px = posA.getX(vi), py = posA.getY(vi), pz = posA.getZ(vi);
        var nx = nrm.getX(vi), nz = nrm.getZ(vi);
        var vv = Math.max(0, Math.min(1, 1 - py / H));
        var rd2 = rowsCache[Math.max(0, Math.min(NR - 1, Math.floor(vv * NR)))];
        var uu = Math.max(0, Math.min(1, px / figW + 0.5));
        var ss = Math.max(0, Math.min(1, pz / figD + 0.5));
        if (rd2) {
          ss = clampR(ss, rd2.sideN, inS);
          uu = clampR(uu, rd2.frontN, inF);
        }
        var cS = sample(sd.map, sbb, ss, vv);
        var cF;
        if (nz >= 0) cF = sample(fr.map, fbb, uu, vv);
        else if (bk) {
          var ub = Math.max(0, Math.min(1, 0.5 - px / Wb));
          if (rd2) ub = clampR(ub, rd2.backN, inB);
          cF = sample(bk.map, bk.bb, ub, vv);
        } else cF = sample(fr.map, fbb, uu, vv);
        var wS = Math.abs(nx), wFB = Math.abs(nz);
        var t = wS / (wS + wFB + 1e-6);
        t = t * t * (3 - 2 * t); // smoothstep
        colA.setXYZ(vi,
          (cF[0] * (1 - t) + cS[0] * t) / 255,
          (cF[1] * (1 - t) + cS[1] * t) / 255,
          (cF[2] * (1 - t) + cS[2] * t) / 255);
      }

      var hullMat = new THREE.MeshBasicMaterial({ vertexColors: true, side: THREE.FrontSide });
      var hull = new THREE.Mesh(hullGeo, hullMat);
      group.add(hull);

      /* ---------- 行查询(供皮肤位移) ---------- */
      function rowAt(y) {
        // ringInfo 按行递减 y；找最近
        var bestK = null, bd = 1e9;
        for (var k2 = 0; k2 < ringInfo.length; k2++) {
          var dd = Math.abs(ringInfo[k2].y - y);
          if (dd < bd) { bd = dd; bestK = ringInfo[k2]; }
        }
        return bestK;
      }
      // 外壳前表面 z(x,y)（在 run 内）
      function hullFrontZ(row, x) {
        for (var q5 = 0; q5 < row.runs.length; q5++) {
          var rn = row.runs[q5];
          var tt = (x - rn.xc) / rn.xr;
          if (Math.abs(tt) <= 1) {
            return rn.zc + rn.zrad * Math.pow(Math.max(0, 1 - Math.pow(Math.abs(tt), PE)), 1 / PE);
          }
        }
        return null;
      }
      function hullBackZ(row, x) {
        for (var q6 = 0; q6 < row.runs.length; q6++) {
          var rn = row.runs[q6];
          var tt = (x - rn.xc) / rn.xr;
          if (Math.abs(tt) <= 1) {
            return rn.zc - rn.zrad * Math.pow(Math.max(0, 1 - Math.pow(Math.abs(tt), PE)), 1 / PE);
          }
        }
        return null;
      }
      function rowMaxFrontZ(row) {
        var m = -1e9;
        for (var q7 = 0; q7 < row.runs.length; q7++) m = Math.max(m, row.runs[q7].zc + row.runs[q7].zrad);
        return m;
      }
      function rowMinBackZ(row) {
        var m = 1e9;
        for (var q8 = 0; q8 < row.runs.length; q8++) m = Math.min(m, row.runs[q8].zc - row.runs[q8].zrad);
        return m;
      }
      // 外壳最外 +X 表面 x(z,y)
      function hullOuterX(row, z) {
        var m = null;
        for (var q9 = 0; q9 < row.runs.length; q9++) {
          var rn = row.runs[q9];
          var tt = (z - rn.zc) / rn.zrad;
          if (Math.abs(tt) <= 1) {
            var xs = rn.xc + rn.xr * Math.pow(Math.max(0, 1 - Math.pow(Math.abs(tt), PE)), 1 / PE);
            if (m === null || xs > m) m = xs;
          }
        }
        return m;
      }
      function rowMaxX(row) {
        var m = 0;
        for (var qa = 0; qa < row.runs.length; qa++) m = Math.max(m, row.runs[qa].xc + row.runs[qa].xr);
        return m;
      }
      // 斜面 z：x 超出 run 时夹到 run 边缘求 z(边缘斜收到 zc),避免皮肤边缘成为前/后戳出的鳍片
      function bevelZ(row, x, sign) {
        var rn = row.runs[0], bd = 1e9;
        for (var q = 0; q < row.runs.length; q++) {
          var dd = Math.abs(x - row.runs[q].xc) - row.runs[q].xr;
          if (dd < bd) { bd = dd; rn = row.runs[q]; }
        }
        var tt = (x - rn.xc) / rn.xr;
        if (tt > 1) tt = 1; if (tt < -1) tt = -1;
        return rn.zc + sign * rn.zrad * Math.pow(Math.max(0, 1 - Math.pow(Math.abs(tt), PE)), 1 / PE);
      }

      /* ---------- ② 皮肤网格 ---------- */
      // viewCut: [vx,vy,vz,cosCut] —— 只保留面法线与 viewDir 夹角 < cut 的三角形，
      // 防止皮肤"卷边鳍片"在斜视/边缘视角产生重影与白边。
      // keepFn(u0,v0)→bool：逐三角形几何裁剪(如背面皮肤裁到正面剪影范围内,去掉素材暗色烟雾晕)。
      // uvFix(u0,v0)→[u,v](bbox 坐标)：逐顶点 UV 修正(如侧皮肤采样夹到剪影内缩,卷边带取衣服内部色)。
      function makeSkin(NU, NV, texCanvas, bb, posFn, viewCut, keepFn, uvFix) {
        var g = new THREE.BufferGeometry();
        var p = [], uv = [], id2 = [];
        for (var j2 = 0; j2 < NV; j2++) {
          for (var i3 = 0; i3 < NU; i3++) {
            var u0 = i3 / (NU - 1), v0 = j2 / (NV - 1);
            var P = posFn(u0, v0);
            p.push(P[0], P[1], P[2]);
            var uu2 = u0, vv2 = v0;
            if (uvFix) { var f2 = uvFix(u0, v0); uu2 = f2[0]; vv2 = f2[1]; }
            uv.push(bb.left + uu2 * (bb.right - bb.left),
                    1 - (bb.top + vv2 * (bb.bottom - bb.top)));
          }
        }
        for (var j3 = 0; j3 < NV - 1; j3++) {
          for (var i4 = 0; i4 < NU - 1; i4++) {
            var a2 = j3 * NU + i4, b2 = a2 + 1, c2 = a2 + NU, d2 = c2 + 1;
            id2.push(a2, c2, b2, b2, c2, d2);
          }
        }
        // 几何裁剪：按三角形中心(u,v)判定,边界误差 < 半格(防止贴边白缝)
        if (keepFn) {
          var keep0 = [];
          for (var t0 = 0; t0 < id2.length; t0 += 3) {
            var iA = id2[t0], iB = id2[t0 + 1], iC = id2[t0 + 2];
            var cu = ((iA % NU) + (iB % NU) + (iC % NU)) / 3 / (NU - 1);
            var cv = (Math.floor(iA / NU) + Math.floor(iB / NU) + Math.floor(iC / NU)) / 3 / (NV - 1);
            if (keepFn(cu, cv)) keep0.push(iA, iB, iC);
          }
          id2 = keep0;
        }
        // 法线裁切：逐三角形计算几何法线
        if (viewCut) {
          var keep = [];
          for (var t2 = 0; t2 < id2.length; t2 += 3) {
            var ia = id2[t2] * 3, ib = id2[t2 + 1] * 3, ic = id2[t2 + 2] * 3;
            var ux = p[ib] - p[ia], uy = p[ib + 1] - p[ia + 1], uz = p[ib + 2] - p[ia + 2];
            var vx = p[ic] - p[ia], vy = p[ic + 1] - p[ia + 1], vz = p[ic + 2] - p[ia + 2];
            var nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
            var nl = Math.sqrt(nx * nx + ny * ny + nz * nz) || 1;
            var dot = (nx * viewCut[0] + ny * viewCut[1] + nz * viewCut[2]) / nl;
            if (dot > viewCut[3]) keep.push(id2[t2], id2[t2 + 1], id2[t2 + 2]);
          }
          id2 = keep;
        }
        g.setAttribute('position', new THREE.Float32BufferAttribute(p, 3));
        g.setAttribute('uv', new THREE.Float32BufferAttribute(uv, 2));
        g.setIndex(id2);
        g.computeVertexNormals();
        var tex = new THREE.CanvasTexture(texCanvas);
        tex.wrapS = tex.wrapT = THREE.ClampToEdgeWrapping;
        var mat = new THREE.MeshBasicMaterial({ map: tex, alphaTest: 0.5, side: THREE.FrontSide });
        return new THREE.Mesh(g, mat);
      }

      var EPS = 0.004;

      // 正面皮肤(面向 +Z)：x=(u-0.5)*Wf，z=外壳前表面斜面+EPS；法线裁切收紧(nz<=0.6)防斜面边条
      var frontSkin = makeSkin(150, 260, fr.canvas, fbb, function (u0, v0) {
        var x = (u0 - 0.5) * figW;
        var y = H * (1 - v0);
        var row = rowAt(y);
        var z = row ? bevelZ(row, x, 1) + EPS : 0;
        return [x, y, z];
      }, [0, 0, 1, 0.6]);
      group.add(frontSkin);

      // 背面皮肤(面向 -Z)：x=(0.5-u)*Wb；几何裁到正面剪影∪背面剪影范围(去背素材烟雾晕)
      if (bk) {
        var backSkin = makeSkin(130, 240, bk.canvas, bk.bb, function (u0, v0) {
          var x = (0.5 - u0) * Wb;
          var y = H * (1 - v0);
          var row = rowAt(y);
          var z = row ? bevelZ(row, x, -1) - EPS : 0;
          return [x, y, z];
        }, [0, 0, -1, 0.6], function (u0, v0) {
          var x = (0.5 - u0) * Wb;
          var y = H * (1 - v0);
          var row = rowAt(y);
          if (!row) return false;
          for (var q = 0; q < row.runs.length; q++) {
            var rn = row.runs[q];
            if (x >= rn.l - 0.012 && x <= rn.r + 0.012) return true;
          }
          if (row.backW && x >= row.backW[0] - 0.012 && x <= row.backW[1] + 0.012) return true;
          return false;
        });
        // x=(0.5-u)*Wb 使标准网格序天然面向 -Z，无需翻转
        group.add(backSkin);
      }

      // 侧面皮肤 ×2：u->z(侧 bbox 左=后,右=前)，x=外壳最外表面±EPS
      function sidePosFn(sign) {
        return function (u0, v0) {
          var z = (u0 - 0.5) * figD;
          var y = H * (1 - v0);
          var row = rowAt(y);
          var x;
          if (row) {
            var xx = hullOuterX(row, z);
            x = (xx !== null ? xx : rowMaxX(row) * 0.95) + EPS;
          } else x = 0;
          return [sign * x, y, z];
        };
      }
      // 侧皮肤 UV 修正：采样夹到该行侧剪影内缩 12px —— 前/后轮廓的投影拉伸区取衣料/肌肉内部色，
      // 避免边缘 rim 填充区被拉成"色带"(只夹剪影范围内的列；范围外保持原采样=透明)
      var inS4 = 12 / sz.wn;
      function sideUvFix(u0, v0) {
        var i = Math.max(0, Math.min(NR - 1, Math.floor(v0 * NR)));
        var rd = rowsCache[i];
        var su = u0;
        if (rd && rd.sideN && u0 >= rd.sideN[0] && u0 <= rd.sideN[1]) {
          var l = rd.sideN[0] + inS4, r = rd.sideN[1] - inS4;
          if (l > r) su = (rd.sideN[0] + rd.sideN[1]) / 2;
          else su = su < l ? l : (su > r ? r : su);
        }
        return [su, v0];
      }
      // 侧皮肤几何裁剪：剔除该行侧剪影范围外的列(防止塌缩列形成深色帘幕)
      function sideKeep(u0, v0) {
        var i = Math.max(0, Math.min(NR - 1, Math.floor(v0 * NR)));
        var rd = rowsCache[i];
        if (!rd || !rd.sideN) return false;
        return u0 >= rd.sideN[0] - 0.004 && u0 <= rd.sideN[1] + 0.004;
      }
      // 标准网格序 (a,c,b),(b,c,d) 天然面向 -X：-X 侧直接用；-X 侧=侧立绘同向(profile 相机在 -X)
      var sideSkinL = makeSkin(96, 260, sd.canvas, sbb, sidePosFn(-1), [-1, 0, 0, 0.5], sideKeep, sideUvFix);
      group.add(sideSkinL);
      // +X 侧(镜像)：裁切在翻转前进行(几何法线仍 -X)，之后再翻顶点序面向 +X
      var sideSkinR = makeSkin(96, 260, sd.canvas, sbb, sidePosFn(1), [-1, 0, 0, 0.5], sideKeep, sideUvFix);
      var li = sideSkinR.geometry.getIndex().array;
      for (var t3 = 0; t3 < li.length; t3 += 3) { var tp = li[t3 + 1]; li[t3 + 1] = li[t3 + 2]; li[t3 + 2] = tp; }
      sideSkinR.geometry.setIndex(new THREE.BufferAttribute(li, 1));
      sideSkinR.geometry.computeVertexNormals();
      group.add(sideSkinR);

      window.__modelLayers = { hullVerts: posA.count, rows: NR, segs: NS };
      window.__modelReady = true;
    }

    return "loading";
  }

  window.ZZ3DBuilder = { build: build };
})();
