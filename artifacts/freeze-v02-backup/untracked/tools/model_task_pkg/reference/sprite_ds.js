/* sprite_ds.js — SpriteStacking(2.5D 立绘投影) 模型构建（DS 独立通道，与体素雕刻无关）
 * 原理：把主角三张正交立绘(正/侧/背)拼成"伪 3D"：
 *   正面看 = 正面立绘切片堆叠(毛发/五官/手/脚天然精确)；
 *   侧面看 = 侧面板(纯侧面90°立绘，面向+X)→ 连续侧面轮廓(不再是百叶窗条纹)；
 *   背面看 = 背面板(纯背面立绘，面向-Z)→ 真实后背；
 *   斜视看 = 切片堆叠的"洋葱皮"视差 → 体积感。
 * 依赖全局 THREE；经 window.SpriteBuilderDS 暴露 build(group, opts)。
 * 可选 opts: { url, layers, depth, taper, height, sideUrl, backUrl }
 */
(function () {
  "use strict";

  // 立绘人物包围盒(相对图像归一化)兜底值 —— 来自 pc_zhengzha.png 实测(768×1024,人物 x[228,540] y[40,1008])
  var DEF_BBOX = { left: 228 / 768, right: 540 / 768, top: 40 / 1024, bottom: 1008 / 1024 };

  // 读取源(HTMLImageElement 或 canvas)的宽高
  function srcSize(img) {
    var w = img.naturalWidth || img.width, h = img.naturalHeight || img.height;
    return { w: w, h: h };
  }

  // 尝试读取图像像素求出人物真实包围盒；file:// 无 --allow-file-access-from-files 时会跨域污染→回退兜底。
  function detectBBox(img) {
    try {
      var s = srcSize(img), W = s.w, H = s.h;
      var c = document.createElement("canvas");
      c.width = W; c.height = H;
      var ctx = c.getContext("2d");
      ctx.drawImage(img, 0, 0);
      var d = ctx.getImageData(0, 0, W, H).data;
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
      if (maxX >= 0 && minX <= maxX) {
        return { left: minX / W, right: (maxX + 1) / W, top: minY / H, bottom: (maxY + 1) / H };
      }
    } catch (e) { /* 画布被污染 → 回退 */ }
    return DEF_BBOX;
  }

  // 同目录派生素材 URL
  function sibling(url, fname) {
    var i = url.lastIndexOf('/');
    return i >= 0 ? url.slice(0, i + 1) + fname : fname;
  }

  // 连通域清洗：只保留人物主体(最大连通域)及其相连细部/光晕，剔除背景里孤立的喷墨噪点岛。
  // 素材本身(如背面图)背景附着半透明黑色喷墨噪点 → alphaTest 会保留成黑点，这里在纹理上抹掉。
  function cleanFigure(src) {
    var s = srcSize(src), W = s.w, H = s.h;
    var c = document.createElement("canvas");
    c.width = W; c.height = H;
    var ctx = c.getContext("2d");
    ctx.drawImage(src, 0, 0);
    var id = ctx.getImageData(0, 0, W, H);
    var d = id.data;
    if (!W || !H) { ctx.putImageData(id, 0, 0); return c; }
    var thresh = 0.15 * 255;             // alpha 连通判定阈值
    var visited = new Uint8Array(W * H);
    var comps = [];
    for (var i = 0; i < W * H; i++) {
      if (visited[i] || d[i * 4 + 3] <= thresh) continue;
      var stack = [i]; visited[i] = 1;
      var px = [], size = 0;
      while (stack.length) {
        var p = stack.pop(); px.push(p); size++;
        var x = p % W, y = (p / W) | 0;
        if (x > 0)          { var n = p - 1;   if (!visited[n] && d[n * 4 + 3] > thresh) { visited[n] = 1; stack.push(n); } }
        if (x < W - 1)      { var n = p + 1;   if (!visited[n] && d[n * 4 + 3] > thresh) { visited[n] = 1; stack.push(n); } }
        if (y > 0)          { var n = p - W;   if (!visited[n] && d[n * 4 + 3] > thresh) { visited[n] = 1; stack.push(n); } }
        if (y < H - 1)      { var n = p + W;   if (!visited[n] && d[n * 4 + 3] > thresh) { visited[n] = 1; stack.push(n); } }
      }
      comps.push({ size: size, px: px });
    }
    // 保留最大连通域 + 任何 >= 最大域 2% 的部件（防误删人物独立细部）；其余剔除(alpha=0)
    var max = 0, mi = -1;
    for (var a = 0; a < comps.length; a++) if (comps[a].size > max) { max = comps[a].size; mi = a; }
    if (comps.length > 0) {
      var keepMin = Math.max(150, max * 0.04);   // 提高：剔掉≥人物4%以下的孤立噪点岛(右肩残点)
      for (var b = 0; b < comps.length; b++) {
        if (b === mi || comps[b].size >= keepMin) continue;
        var pxs = comps[b].px;
        for (var q = 0; q < pxs.length; q++) d[pxs[q] * 4 + 3] = 0;
      }
    }
    ctx.putImageData(id, 0, 0);
    return c;
  }

  // 建立一张「裁到人物包围盒 + 镂空」的材质，并返回人物纵横比。src 可为 HTMLImageElement 或 canvas(已清洗)。
  function makeMat(tex, src, alphaTest) {
    var bb = detectBBox(src);
    var figWNorm = bb.right - bb.left;   // 人物占图像宽比
    var figHNorm = bb.bottom - bb.top;   // 人物占图像高比
    tex.wrapS = tex.wrapT = THREE.ClampToEdgeWrapping;
    tex.repeat.set(figWNorm, figHNorm);
    tex.offset.set(bb.left, 1 - bb.bottom);
    var s = srcSize(src);
    var aspect = (figWNorm * s.w) / (figHNorm * s.h);  // 人物宽/高
    var mat = new THREE.MeshBasicMaterial({
      map: tex, transparent: false, alphaTest: alphaTest != null ? alphaTest : 0.5, side: THREE.DoubleSide, depthWrite: true
    });
    return { mat: mat, aspect: aspect };
  }

  // 加载图像(HTMLImageElement)——用于做素材清洗
  function loadImg(url, cb, er) {
    var im = new Image();
    im.crossOrigin = "anonymous";
    im.onload = function () { cb(im); };
    im.onerror = er;
    im.src = url;
  }

  function build(group, opts) {
    opts = opts || {};
    var url = opts.url ||
      ((typeof document !== "undefined") ? new URL("../assets/pc_zhengzha_front.png", document.baseURI).href : "../assets/pc_zhengzha_front.png");
    var N = opts.layers || 48;          // 层数(28→48，减条纹间隙)
    var depth = opts.depth != null ? opts.depth : 1.2;    // 总深度
    var taper = opts.taper != null ? opts.taper : 0.5;    // 最前1.0 → 最后(1-taper)
    var H = opts.height != null ? opts.height : 2.3;      // 人物世界高度(脚0/头H)
    var urlFront = url;
    var urlSide = opts.sideUrl || sibling(url, 'pc_zhengzha_side.png');
    var urlBack = opts.backUrl || sibling(url, 'pc_zhengzha_back.png');

    // 清空已有子件(供重复调用)
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

    window.__spriteReady = false;
    var res = { front: null, side: null, back: null }, pending = 3;

    function maybeBuild() {
      if (--pending > 0) return;
      var fr = res.front;
      if (!fr) return;   // 正面必载(否则无可构建)
      var layerThick = depth / N;
      // 正/侧/背人物纵横比(等高 H)；宽 = H*aspect
      var F = fr.aspect;
      var faceH = H, faceW = faceH * F;
      var matFront = fr.mat;
      var matSide = res.side ? res.side.mat : null;
      var matBack = res.back ? res.back.mat : null;
      var backW = res.back ? faceH * res.back.aspect : 0;
      // 遮断板加宽到超过正/背人物最宽,确保从正/背方向把背后所有内容(切片/侧背面板)全部涂白,边缘不漏
      var coverW = Math.max(faceW, backW) * 1.15;
      var whiteMat = new THREE.MeshBasicMaterial({ color: 0xffffff, side: THREE.DoubleSide });

      // ① 正面切片堆叠(面向 +Z，前大后小，洋葱皮体积)
      var added = 0;
      for (var i = 0; i < N; i++) {
        var s = 1.0 - (i / (N - 1)) * taper;
        var m = new THREE.Mesh(new THREE.PlaneGeometry(faceW * s, faceH * s), matFront);
        m.position.set(0, H / 2, depth / 2 - i * layerThick - layerThick / 2);
        group.add(m);
        added++;
      }
      // ② 正面遮断板(涂白)：盖住切片与侧/背面板在正面的投影(白=背景)
      var fcover = new THREE.Mesh(new THREE.PlaneGeometry(coverW, faceH), whiteMat);
      fcover.position.set(0, H / 2, depth / 2 + layerThick * 0.5);
      group.add(fcover);
      // ③ 最前正面立绘平面：像素级 = 原立绘
      var frontMesh = new THREE.Mesh(new THREE.PlaneGeometry(faceW, faceH), matFront);
      frontMesh.position.set(0, H / 2, depth / 2 + layerThick);
      group.add(frontMesh);
      added += 2;

      // ④ 侧面板：纯侧面立绘，面向 +X，且前移超过切片最宽边→侧面看盖住"梳齿"，只留连续轮廓
      if (matSide) {
        var sideDepth = faceH * (res.side.aspect);
        var sideMesh = new THREE.Mesh(new THREE.PlaneGeometry(sideDepth, faceH), matSide);
        sideMesh.rotation.y = Math.PI / 2;              // 面向 +X（立绘宽沿世界 Z=人物厚度方向）
        sideMesh.position.set(faceW * 0.52, H / 2, 0);
        group.add(sideMesh);
      }
      // ⑤ 背面遮断板(涂白)：在堆叠之后盖住切片/侧面板在背面的投影，让背面只露背面板
      var bcover = new THREE.Mesh(new THREE.PlaneGeometry(coverW, faceH), whiteMat);
      bcover.position.set(0, H / 2, -depth / 2 - 0.01);
      group.add(bcover);
      // ⑥ 背面板：纯背面立绘，置于最深处→背面看=真实后背
      if (matBack) {
        var backMesh = new THREE.Mesh(new THREE.PlaneGeometry(backW, faceH), matBack);
        backMesh.rotation.y = Math.PI;                  // 面向 -Z
        backMesh.position.set(0, H / 2, -depth / 2 - 0.03);
        group.add(backMesh);
      }

      window.__spriteLayers = added;
      window.__spriteReady = true;
    }

    // 正面：用原图(已干净)直接建纹理，保持像素级
    var loader = new THREE.TextureLoader();
    loader.load(urlFront, function (t) { res.front = makeMat(t, t.image, 0.5); maybeBuild(); }, undefined, function () { maybeBuild(); });
    // 侧/背面：先裁掉背景喷墨噪点岛(连通域清洗)→ 建纹理
    function loadClean(u, key, alpha) {
      loadImg(u, function (im) {
        try {
          var cv = cleanFigure(im);
          var tex = new THREE.CanvasTexture(cv);
          res[key] = makeMat(tex, cv, alpha);
        } catch (e) { res[key] = null; }
        maybeBuild();
      }, function () { res[key] = null; maybeBuild(); });
    }
    loadClean(urlSide, 'side', 0.5);
    loadClean(urlBack, 'back', 0.5);
    return "loading";
  }

  window.SpriteBuilderDS = { build: build };
})();
