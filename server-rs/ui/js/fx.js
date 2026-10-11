/* =========================================================================
 * 《无限轮回》全局氛围特效层 —— window.FX  v1.0.0
 * -------------------------------------------------------------------------
 * 依据 docs/ART_DIRECTION.md：
 *   「二、统一调色板」  → 冷调基底 #0a0e14 / #111821、科技青 #4fd6e8、危险红 #ff5a4a …
 *   「四、全局氛围层」  → ①暗角 ②胶片颗粒 ③扫描线 ④色彩分级 ⑤色差 ⑥震动/闪白
 *
 * 设计约束（务必遵守）：
 *   1) 不改 index.html / css/style.css —— 本文件自己注入 <style> 与图层 DOM。
 *   2) 所有图层 pointer-events:none，绝不遮挡 UI 点击。
 *   3) 性能友好：
 *      · 颗粒用「离屏 canvas 预渲染噪点贴图」一次性生成，之后只靠 CSS transform 位移播放，
 *        全程合成器合成，不逐帧生成噪点、不重绘。
 *      · 暗角/分级/扫描线是纯 CSS 静态层，零 JS 开销。
 *      · 只有「震动 / 闪白 / 色差脉冲 / 暗角收缩」这类动态效果才开 rAF，且全部结束后立刻停帧。
 *      · FX.setEnabled(false) 一键全关（含恢复被接管的旧 #grain 噪点层）。
 *   4) 强度克制：颗粒/扫描线有效 alpha 落在 0.02 ~ 0.06 量级。
 *
 * 图层 z-index 取 43~51：压在世界地图(30)/副本(35)/HUD(30) 之上，
 * 但在标题画面(60)/设置等弹窗(60~95) 之下，保证菜单文字不被染色。
 *
 * 对外契约：只挂 window.FX，不改任何 Tauri invoke / DSH_BOOT / World2D / Zone2D 契约。
 * ========================================================================= */
"use strict";

(function () {
  if (window.FX && window.FX.__isFX) return;   // 防重复注入

  var VERSION = "1.0.0";
  var STORE_KEY = "wuxian.fx.v1";
  var TILE = 128;                              // 噪点贴图边长（纹理像素）

  /* ---------------- 调色板（ART_DIRECTION 二、统一调色板） ---------------- */
  var PALETTE = {
    baseDark: "#0a0e14", baseDeep: "#111821",
    steel: "#2b3540",    steelHi: "#3d4a58",
    cyan: "#4fd6e8",     cyanHi: "#8fe8f5",
    red: "#c8362f",      redHi: "#ff5a4a",
    green: "#5fbf5a",    greenHi: "#a8e86a",
    amber: "#d8a83c",    amberHi: "#ffd76a"
  };

  /* ---------------- 强度基准（strength = 1.0 时对应图层的不透明度） ----------------
   * 颗粒 0.055 / 第二层颗粒 0.032 / 扫描线 0.16×0.24 = 0.038 有效 alpha —— 均在 0.02~0.06 量级。
   */
  var BASE = {
    tint: 0.24,       // 冷调 multiply 层
    desat: 0.12,      // 降饱和 saturation 层
    contrast: 0.85,   // 中心微提亮 + 边缘压暗（overlay）
    vignette: 0.82,   // 四角压暗
    scan: 0.16,       // 扫描线
    grain: 0.055,     // 主颗粒
    grain2: 0.032,    // 次级粗颗粒
    chroma: 0.5       // 色差脉冲峰值
  };

  var DEFAULTS = {
    enabled: true,
    grade: 1.0,               // 色彩分级总强度（同时作用于 冷调/降饱和/对比）
    vignette: 1.0,            // 暗角
    grain: 1.0,               // 颗粒
    grainLayers: 2,           // 1 = 只留主颗粒（省一点填充率）
    scanline: 1.0,            // 扫描线
    chroma: 0,                // 常驻色差（默认 0，只在受击/恐惧时用 chromatic() 脉冲）
    backdrop: false,          // 真·对比/饱和分级（backdrop-filter，默认关，低端机不背这个开销）
    backdropFilter: "contrast(1.10) saturate(0.90) brightness(0.96) hue-rotate(-5deg)",
    hideLegacyGrain: true,    // FX 接管旧 #grain（避免双重颗粒 + 省掉每 120ms 的全分辨率噪点生成）
    maxShakePx: 14,           // screenShake(power=1) 的峰值位移
    shakeTargets: ["#stage", "#worldView", "#zoneView", "#hud"]
  };

  var P = {};
  for (var dk in DEFAULTS) if (Object.prototype.hasOwnProperty.call(DEFAULTS, dk)) P[dk] = DEFAULTS[dk];

  /* ---------------- 内部状态 ---------------- */
  var L = {};                                  // 图层 DOM 引用
  var styleEl = null;
  var built = false, pending = false;
  var rafId = 0;
  var shakeCur = { x: 0, y: 0 };
  var vigPulse = 0;
  var pendingShakeEls = null;
  var legacySaved = null;
  var reduced = false;
  var dyn = { shake: null, flash: null, chroma: null, vig: null };

  var clamp = function (v, a, b) { return v < a ? a : (v > b ? b : v); };
  var num = function (v, d) { var n = Number(v); return isFinite(n) ? n : d; };
  var r4 = function (v) { return Math.round(v * 10000) / 10000; };
  var now = function () {
    return (window.performance && window.performance.now) ? window.performance.now() : Date.now();
  };

  /* ---------------- 注入样式（自己管自己，不动 css/style.css） ---------------- */
  var CSS = [
    "#fxTint,#fxDesat,#fxContrast,#fxVignette,#fxScan,#fxChromaR,#fxChromaC,#fxFlash{position:fixed;left:0;top:0;right:0;bottom:0;pointer-events:none;}",
    /* ① 色彩分级 · 冷调：multiply 冷蓝灰，压暗 + 统一色温（暗部占 70%+） */
    "#fxTint{z-index:44;mix-blend-mode:multiply;opacity:var(--fx-tint,.24);background:linear-gradient(177deg,rgba(160,188,216,1) 0%,rgba(128,154,182,1) 52%,rgba(96,120,146,1) 100%);}",
    /* ① 色彩分级 · 降饱和：saturation 混合中灰 = 抽掉一部分彩度，避免高饱和平铺 */
    "#fxDesat{z-index:44;mix-blend-mode:saturation;opacity:var(--fx-desat,.12);background:#808080;}",
    /* ① 色彩分级 · 对比：overlay —— 中心微提亮（镜头热区）+ 边缘压暗，观感上「提对比」 */
    "#fxContrast{z-index:45;mix-blend-mode:overlay;opacity:var(--fx-contrast,.85);background:radial-gradient(120% 105% at 50% 46%,rgba(232,240,248,.16) 0%,rgba(200,214,228,.06) 34%,rgba(10,14,20,0) 58%,rgba(6,10,16,.46) 100%);}",
    /* ① 可选：真·对比/饱和分级（backdrop-filter，需显示性能，默认关） */
    "#fxBackdrop{z-index:43;position:fixed;left:0;top:0;right:0;bottom:0;pointer-events:none;display:none;opacity:var(--fx-backdrop,1);}",
    /* ② 暗角：四角压暗，聚焦中心 */
    "#fxVignette{z-index:46;opacity:var(--fx-vig,.82);background:radial-gradient(ellipse 76% 76% at 50% 48%,rgba(0,0,0,0) 40%,rgba(3,6,10,.34) 70%,rgba(2,4,8,.78) 90%,rgba(1,2,5,.94) 100%);}",
    /* ③ 扫描线：极淡横纹 + 缓慢下滚的扫描带（CRT/监控感） */
    "#fxScan{z-index:47;overflow:hidden;opacity:var(--fx-scan,.16);background:repeating-linear-gradient(0deg,rgba(0,0,0,.24) 0px,rgba(0,0,0,.24) 1px,rgba(0,0,0,0) 1px,rgba(0,0,0,0) 3px);}",
    "#fxScan .fxRoll{position:absolute;left:0;right:0;top:0;height:22vh;background:linear-gradient(180deg,rgba(79,214,232,0) 0%,rgba(143,232,245,.14) 50%,rgba(79,214,232,0) 100%);animation:fxRoll 11s linear infinite;}",
    "@keyframes fxRoll{0%{transform:translate3d(0,-30vh,0);}100%{transform:translate3d(0,118vh,0);}}",
    /* ④ 胶片颗粒：预渲染噪点贴图平铺 + 离散位移播放（纯合成器动画，不重绘、不逐帧造噪点） */
    "#fxGrain,#fxGrain2{position:fixed;left:-200px;top:-200px;right:-200px;bottom:-200px;pointer-events:none;background-repeat:repeat;background-position:0 0;mix-blend-mode:overlay;}",
    "#fxGrain{z-index:48;opacity:var(--fx-grain,.055);animation:fxGrainA .48s steps(1,end) infinite;}",
    "#fxGrain2{z-index:49;opacity:var(--fx-grain2,.032);animation:fxGrainB .63s steps(1,end) infinite;}",
    "@keyframes fxGrainA{0%{transform:translate3d(0,0,0);}16%{transform:translate3d(-73px,41px,0);}33%{transform:translate3d(37px,-59px,0);}50%{transform:translate3d(-19px,-23px,0);}66%{transform:translate3d(61px,17px,0);}83%{transform:translate3d(-47px,73px,0);}100%{transform:translate3d(0,0,0);}}",
    "@keyframes fxGrainB{0%{transform:translate3d(0,0,0);}20%{transform:translate3d(53px,-31px,0);}40%{transform:translate3d(-67px,-17px,0);}60%{transform:translate3d(23px,63px,0);}80%{transform:translate3d(-29px,-49px,0);}100%{transform:translate3d(0,0,0);}}",
    /* ⑤ 色差：红/青双环 screen 叠加，反向位移 = 边缘 RGB 分离 */
    "#fxChromaR,#fxChromaC{z-index:50;mix-blend-mode:screen;opacity:0;}",
    "#fxChromaR{background:radial-gradient(ellipse 92% 90% at 50% 50%,rgba(255,90,74,0) 46%,rgba(255,90,74,.42) 88%,rgba(200,54,47,.52) 100%);}",
    "#fxChromaC{background:radial-gradient(ellipse 92% 90% at 50% 50%,rgba(79,214,232,0) 46%,rgba(79,214,232,.42) 88%,rgba(143,232,245,.52) 100%);}",
    /* ⑥ 受击闪白 */
    "#fxFlash{z-index:51;opacity:0;background:#ffffff;}",
    "@media (prefers-reduced-motion: reduce){#fxGrain,#fxGrain2{animation:none;}#fxScan .fxRoll{animation:none;opacity:0;}}"
  ].join("\n");

  /* ---------------- 噪点贴图（离屏 canvas 只生成一次） ---------------- */
  function noiseTile(size, spread) {
    var c = document.createElement("canvas");
    c.width = size; c.height = size;
    try {
      var ctx = c.getContext("2d");
      var img = ctx.createImageData(size, size);
      var d = img.data;
      for (var i = 0; i < d.length; i += 4) {
        var v = (128 + (Math.random() * 2 - 1) * spread) | 0;
        v = v < 0 ? 0 : (v > 255 ? 255 : v);
        d[i] = d[i + 1] = d[i + 2] = v;
        d[i + 3] = 255;
      }
      ctx.putImageData(img, 0, 0);
      return c.toDataURL("image/png");
    } catch (e) {
      return "";
    }
  }

  /* 贴图 1 纹理像素 ≈ 1 设备像素（HiDPI 下颗粒不会变粗） */
  function applyTileScale() {
    var dpr = window.devicePixelRatio || 1;
    var s1 = TILE / dpr;
    var s2 = (TILE * 2.4) / dpr;
    if (L.fxGrain) L.fxGrain.style.backgroundSize = s1 + "px " + s1 + "px";
    if (L.fxGrain2) L.fxGrain2.style.backgroundSize = s2 + "px " + s2 + "px";
  }

  /* ---------------- 强度 → CSS 变量 ---------------- */
  function updateVars() {
    var root = document.documentElement;
    if (!root) return;
    var g = clamp(num(P.grade, 1), 0, 2);
    var s = function (k, v) { root.style.setProperty(k, String(r4(clamp(v, 0, 1)))); };
    s("--fx-tint", BASE.tint * g);
    s("--fx-desat", BASE.desat * g);
    s("--fx-contrast", BASE.contrast * g);
    s("--fx-vig", BASE.vignette * clamp(num(P.vignette, 1), 0, 2) + vigPulse);
    s("--fx-scan", BASE.scan * clamp(num(P.scanline, 1), 0, 2));
    s("--fx-grain", BASE.grain * clamp(num(P.grain, 1), 0, 2));
    s("--fx-grain2", (num(P.grainLayers, 2) > 1 ? BASE.grain2 * clamp(num(P.grain, 1), 0, 2) : 0));
    s("--fx-chroma", BASE.chroma * clamp(num(P.chroma, 0), 0, 2));
    s("--fx-backdrop", clamp(num(P.backdropStrength, 1), 0, 1));
  }

  /* ---------------- 构建图层 ---------------- */
  function mk(id, cls) {
    var el = document.createElement("div");
    el.id = id;
    if (cls) el.className = cls;
    el.setAttribute("aria-hidden", "true");
    return el;
  }

  function build() {
    styleEl = document.createElement("style");
    styleEl.id = "fxStyle";
    styleEl.textContent = CSS;
    document.head.appendChild(styleEl);

    // 注意：混合模式图层必须是 body 的直接子节点（父级若形成隔离组，blend 会失效）
    var order = ["fxBackdrop", "fxTint", "fxDesat", "fxContrast", "fxVignette", "fxScan", "fxGrain", "fxGrain2", "fxChromaR", "fxChromaC", "fxFlash"];
    for (var i = 0; i < order.length; i++) {
      var el = mk(order[i]);
      if (order[i] === "fxScan") {
        var roll = document.createElement("i");
        roll.className = "fxRoll";
        el.appendChild(roll);
      }
      document.body.appendChild(el);
      L[order[i]] = el;
    }

    // 噪点贴图：两层共用一张（粗层用 background-size 放大 2.4 倍）
    var url = noiseTile(TILE, 96);
    if (url && L.fxGrain) {
      L.fxGrain.style.backgroundImage = "url(\"" + url + "\")";
      if (L.fxGrain2) L.fxGrain2.style.backgroundImage = "url(\"" + url + "\")";
    }
    applyTileScale();

    try {
      var mq = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)");
      reduced = !!(mq && mq.matches);
    } catch (e) { reduced = false; }

    window.addEventListener("resize", onResize, { passive: true });
    updateVars();
    applyBackdrop();
  }

  var resizeTimer = 0;
  function onResize() {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(function () { resizeTimer = 0; applyTileScale(); }, 220);
  }

  function applyBackdrop() {
    if (!L.fxBackdrop) return;
    var on = !!P.backdrop && !!P.enabled;
    L.fxBackdrop.style.display = on ? "block" : "none";
    if (on) {
      var f = String(P.backdropFilter || "");
      L.fxBackdrop.style.backdropFilter = f;
      L.fxBackdrop.style.webkitBackdropFilter = f;
    }
  }

  /* ---------------- 旧 #grain 噪点层的接管 / 归还 ---------------- */
  function setLegacyGrain(keep) {
    var el = document.getElementById("grain");
    if (!el) return;
    if (keep) {
      if (legacySaved) {
        el.style.display = legacySaved.display;
        try { if (legacySaved.w) { el.width = legacySaved.w; el.height = legacySaved.h; } } catch (e) {}
        legacySaved = null;
      }
      if (typeof window.grainResize === "function") { try { window.grainResize(); } catch (e) {} }
    } else {
      if (!legacySaved) legacySaved = { display: el.style.display || "", w: el.width, h: el.height };
      el.style.display = "none";
      // 把画布缩到 1x1：client.js 里每 120ms 的全分辨率噪点生成会退化成 1 像素（省 CPU）
      try { el.width = 1; el.height = 1; } catch (e) {}
    }
  }

  /* ---------------- 启停 ---------------- */
  function applyEnabled() {
    var on = !!P.enabled;
    for (var id in L) {
      if (!Object.prototype.hasOwnProperty.call(L, id) || !L[id]) continue;
      if (id === "fxBackdrop") continue;          // 由 applyBackdrop() 单独管
      L[id].style.display = on ? "" : "none";
    }
    if (!on) {
      cancelDynamic();
      if (L.fxFlash) L.fxFlash.style.opacity = "0";
      applyChroma(0);
      vigPulse = 0;
      updateVars();
    }
    applyBackdrop();
    setLegacyGrain(!(on && P.hideLegacyGrain));
    if (on) updateVars();
  }

  /* ---------------- 动态效果引擎（只在有动态效果时跑） ---------------- */
  function kick() {
    if (!rafId && P.enabled) rafId = requestAnimationFrame(loop);
  }

  function cancelDynamic() {
    dyn.shake = null; dyn.flash = null; dyn.chroma = null; dyn.vig = null;
    endShake();
    if (rafId) { cancelAnimationFrame(rafId); rafId = 0; }
  }

  function loop(t) {
    rafId = 0;
    var busy = false;

    var sh = dyn.shake;
    if (sh) {
      var k = (t - sh.start) / sh.dur;
      if (k >= 1) { dyn.shake = null; endShake(); }
      else {
        var decay = (1 - k) * (1 - k);
        var amp = sh.power * num(P.maxShakePx, 14) * decay * (reduced ? 0.25 : 1);
        var tx = (Math.random() * 2 - 1) * amp;
        var ty = (Math.random() * 2 - 1) * amp;
        shakeCur.x = shakeCur.x * 0.55 + tx * 0.45;   // 略作平滑，避免纯随机的抖动噪感
        shakeCur.y = shakeCur.y * 0.55 + ty * 0.45;
        applyShake(shakeCur.x, shakeCur.y, sh.rot * decay);
        busy = true;
      }
    }

    var fl = dyn.flash;
    if (fl && L.fxFlash) {
      var kf = (t - fl.start) / fl.dur;
      if (kf >= 1) { dyn.flash = null; L.fxFlash.style.opacity = "0"; }
      else {
        var pf = kf < 0.14 ? (kf / 0.14) : (1 - (kf - 0.14) / 0.86);
        L.fxFlash.style.opacity = String(r4(fl.peak * clamp(pf, 0, 1)));
        busy = true;
      }
    }

    var ch = dyn.chroma;
    if (ch) {
      var kc = (t - ch.start) / ch.dur;
      if (kc >= 1) { dyn.chroma = null; applyChroma(num(P.chroma, 0)); }
      else { applyChroma(num(P.chroma, 0) + ch.power * (1 - kc)); busy = true; }
    }

    var vg = dyn.vig;
    if (vg) {
      var kv = (t - vg.start) / vg.dur;
      if (kv >= 1) { dyn.vig = null; vigPulse = 0; }
      else { vigPulse = vg.power * (1 - kv); busy = true; }
      updateVars();
    }

    if (busy && P.enabled) rafId = requestAnimationFrame(loop);
    else if (!busy) { shakeCur.x = 0; shakeCur.y = 0; }
  }

  /* ---------------- 屏幕震动（DOM 层，同时给 canvas 层暴露 offset） ---------------- */
  function collectShakeTargets(list) {
    var out = [];
    list = list || P.shakeTargets || [];
    for (var i = 0; i < list.length; i++) {
      try {
        var found = document.querySelectorAll(list[i]);
        for (var j = 0; j < found.length; j++) out.push(found[j]);
      } catch (e) { /* 选择器非法则跳过 */ }
    }
    return out;
  }

  function beginShake(targets) {
    for (var i = 0; i < targets.length; i++) {
      var el = targets[i];
      if (el.__fxTf === undefined) el.__fxTf = el.style.transform || "";
      el.style.willChange = "transform";
    }
  }

  function applyShake(x, y, rot) {
    var sh = dyn.shake;
    if (!sh) return;
    var els = sh.els || [];
    for (var i = 0; i < els.length; i++) {
      var el = els[i];
      var base = el.__fxTf || "";
      el.style.transform = (base ? base + " " : "") +
        "translate3d(" + r4(x) + "px," + r4(y) + "px,0)" +
        (rot ? " rotate(" + r4(rot) + "deg)" : "");
    }
  }

  function endShake() {
    var els = pendingShakeEls || [];
    for (var j = 0; j < els.length; j++) {
      var el = els[j];
      if (el.__fxTf !== undefined) { el.style.transform = el.__fxTf; delete el.__fxTf; }
      el.style.willChange = "";
    }
    pendingShakeEls = null;
    shakeCur.x = 0; shakeCur.y = 0;
  }

  /* ---------------- 色差 ---------------- */
  function applyChroma(strength) {
    var s = clamp(num(strength, 0), 0, 3);
    var d = s <= 0 ? 0 : (1 + 5 * s);
    var o = clamp(BASE.chroma * s, 0, 1);
    if (L.fxChromaR) {
      L.fxChromaR.style.opacity = String(r4(o * 0.9));
      L.fxChromaR.style.transform = "translate3d(" + r4(-d) + "px,0,0)";
    }
    if (L.fxChromaC) {
      L.fxChromaC.style.opacity = String(r4(o));
      L.fxChromaC.style.transform = "translate3d(" + r4(d) + "px,0,0)";
    }
  }

  /* ---------------- 初始化 ---------------- */
  function loadPrefs() {
    try {
      var raw = window.localStorage && window.localStorage.getItem(STORE_KEY);
      if (!raw) return;
      var o = JSON.parse(raw);
      if (o && typeof o === "object") {
        for (var k in DEFAULTS) if (Object.prototype.hasOwnProperty.call(DEFAULTS, k) && o[k] !== undefined) P[k] = o[k];
      }
    } catch (e) { /* 无 localStorage / 坏数据：忽略 */ }
  }

  function savePrefs() {
    try {
      var o = {};
      var keys = ["enabled", "grade", "vignette", "grain", "grainLayers", "scanline", "chroma", "backdrop", "hideLegacyGrain"];
      for (var i = 0; i < keys.length; i++) o[keys[i]] = P[keys[i]];
      window.localStorage.setItem(STORE_KEY, JSON.stringify(o));
    } catch (e) {}
  }

  function init(opts) {
    if (built) {
      if (opts && typeof opts === "object") { for (var k in opts) if (opts[k] !== undefined) P[k] = opts[k]; updateVars(); applyBackdrop(); applyEnabled(); }
      return API;
    }
    if (!document.body) {
      if (!pending) {
        pending = true;
        var run = function () { pending = false; try { init(opts); } catch (e) { warn(e); } };
        if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", run, { once: true });
        else setTimeout(run, 0);
      }
      return API;
    }
    try {
      loadPrefs();
      if (opts && typeof opts === "object") {
        for (var k2 in opts) if (opts[k2] !== undefined) P[k2] = opts[k2];
      }
      build();
      built = true;
      applyEnabled();
      if (P.enabled) updateVars();
    } catch (e) { warn(e); }
    return API;
  }

  function warn(e) {
    try { if (window.console && console.warn) console.warn("[FX] 初始化失败（已静默降级，不影响游戏）:", e); } catch (e2) {}
  }

  function ensure() {
    if (!built) init();
    return built;
  }

  /* ---------------- 对外 API ---------------- */
  var API = {
    __isFX: true,
    version: VERSION,
    VERSION: VERSION,
    PALETTE: PALETTE,

    /** 初始化（幂等）。opts 可覆盖任意默认参数，如 {grade:0.8} */
    init: init,

    /** 一键全关/全开（低端机可关）。关时同时归还被接管的旧 #grain 层。 */
    setEnabled: function (on) {
      P.enabled = !!on;
      if (!ensure()) { return API; }
      applyEnabled();
      kick();
      savePrefs();
      return API;
    },
    isEnabled: function () { return !!P.enabled; },

    /** 色彩分级总强度：冷调 + 降饱和 + 对比，0 = 关，1 = 默认，2 = 上限 */
    setGrade: function (s) { P.grade = clamp(num(s, 1), 0, 2); if (ensure()) updateVars(); savePrefs(); return API; },

    /** 暗角强度 */
    setVignette: function (s) { P.vignette = clamp(num(s, 1), 0, 2); if (ensure()) updateVars(); savePrefs(); return API; },

    /** 颗粒强度 / 层数（1 = 省一点填充率） */
    setGrain: function (s, layers) {
      P.grain = clamp(num(s, 1), 0, 2);
      if (layers !== undefined) P.grainLayers = num(layers, 2) > 1 ? 2 : 1;
      if (ensure()) updateVars();
      savePrefs();
      return API;
    },

    /** 扫描线强度 */
    setScanline: function (s) { P.scanline = clamp(num(s, 1), 0, 2); if (ensure()) updateVars(); savePrefs(); return API; },

    /** 常驻色差强度（恐惧/理智低时可用；受击请用 chromatic 脉冲） */
    setChroma: function (s) { P.chroma = clamp(num(s, 0), 0, 2); if (ensure()) applyChroma(P.chroma); savePrefs(); return API; },

    /**
     * 色差脉冲（受击 / 恐惧）。world2d / zone2d 可调用。
     * @param {number} power 1 = 默认
     * @param {number} ms    持续毫秒，默认 280
     */
    chromatic: function (power, ms) {
      if (!ensure() || !P.enabled) return API;
      dyn.chroma = { power: clamp(num(power, 1), 0, 3), start: now(), dur: Math.max(60, num(ms, 280)) };
      kick();
      return API;
    },

    /**
     * 屏幕震动。DOM 层直接位移 #stage/#worldView/#zoneView/#hud；
     * canvas 层（world2d/zone2d）可用 getShakeOffset() 自己做位移（二者别同时用，会叠加）。
     * @param {number} power 1 = 默认（峰值 maxShakePx=14px）
     * @param {number} ms    持续毫秒，默认 220
     * @param {object} [opts] {targets:[选择器], rot:旋转角度}
     */
    screenShake: function (power, ms, opts) {
      if (!ensure() || !P.enabled) return API;
      opts = opts || {};
      var list = Array.isArray(opts.targets) && opts.targets.length ? opts.targets : P.shakeTargets;
      var els = collectShakeTargets(list);
      var prev = pendingShakeEls || [];
      for (var m = 0; m < prev.length; m++) {
        if (els.indexOf(prev[m]) < 0) {   // 换目标：先归位旧的
          if (prev[m].__fxTf !== undefined) { prev[m].style.transform = prev[m].__fxTf; delete prev[m].__fxTf; }
        }
      }
      beginShake(els);
      pendingShakeEls = els;
      shakeCur.x = 0; shakeCur.y = 0;
      dyn.shake = {
        power: clamp(num(power, 1), 0, 4),
        start: now(),
        dur: Math.max(40, num(ms, 220)),
        rot: clamp(num(opts.rot, 0), -3, 3),
        els: els
      };
      kick();
      return API;
    },

    /**
     * 受击闪白 / 闪色。
     * @param {string} color 默认 '#ffffff'（危险红可用 PALETTE.redHi）
     * @param {number} ms    默认 180
     * @param {number} peak  峰值不透明度，默认 0.5
     * @param {string} blend 'normal' | 'screen' | 'plus-lighter' | 'overlay'
     */
    hitFlash: function (color, ms, peak, blend) {
      if (!ensure() || !P.enabled || !L.fxFlash) return API;
      L.fxFlash.style.background = color || "#ffffff";
      try { L.fxFlash.style.mixBlendMode = blend || "normal"; } catch (e) {}
      dyn.flash = { start: now(), dur: Math.max(30, num(ms, 180)), peak: clamp(num(peak, 0.5), 0, 1) };
      kick();
      return API;
    },

    /** 暗角收缩脉冲（暴击 / 濒死时收紧视野） */
    vignettePulse: function (power, ms) {
      if (!ensure() || !P.enabled) return API;
      dyn.vig = { power: clamp(num(power, 1), 0, 1) * 0.2, start: now(), dur: Math.max(80, num(ms, 450)) };
      kick();
      return API;
    },

    /** canvas 层用：当前震动偏移（CSS 像素）；未震动时 active=false */
    getShakeOffset: function () {
      return {
        x: r4(shakeCur.x), y: r4(shakeCur.y),
        active: !!(dyn.shake && P.enabled)
      };
    },

    /** 自定义震动作用目标（选择器数组） */
    setShakeTargets: function (list) {
      if (Array.isArray(list)) P.shakeTargets = list.slice(0);
      return API;
    },

    /** 真·对比/饱和分级（backdrop-filter）。默认关，开启后观感最「电影」，但吃一点填充率 */
    setBackdropGrade: function (on, filter) {
      P.backdrop = !!on;
      if (typeof filter === "string" && filter) P.backdropFilter = filter;
      if (ensure()) applyBackdrop();
      savePrefs();
      return API;
    },

    /** true = 保留 client.js 里那个旧 #grain 噪点层（默认被 FX 接管隐藏） */
    setLegacyGrain: function (keep) {
      P.hideLegacyGrain = !keep;
      if (ensure()) setLegacyGrain(!(P.enabled && P.hideLegacyGrain));
      savePrefs();
      return API;
    },

    /** 性能档位预设：high / medium（默认）/ low */
    setPerf: function (level) {
      var l = String(level || "medium").toLowerCase();
      if (l === "high") {
        P.grade = 1.15; P.vignette = 1.0; P.grain = 1.15; P.grainLayers = 2; P.scanline = 1.0; P.backdrop = true;
      } else if (l === "low") {
        P.grade = 0.8; P.vignette = 0.9; P.grain = 0.55; P.grainLayers = 1; P.scanline = 0.6; P.backdrop = false;
      } else {
        P.grade = 1.0; P.vignette = 1.0; P.grain = 1.0; P.grainLayers = 2; P.scanline = 1.0; P.backdrop = false;
      }
      if (ensure()) { updateVars(); applyBackdrop(); }
      savePrefs();
      return API;
    },

    /** 当前参数快照（只读副本） */
    params: function () {
      var o = {};
      for (var k in P) if (Object.prototype.hasOwnProperty.call(P, k)) o[k] = Array.isArray(P[k]) ? P[k].slice(0) : P[k];
      o.vars = {
        tint: BASE.tint * num(P.grade, 1),
        desat: BASE.desat * num(P.grade, 1),
        contrast: BASE.contrast * num(P.grade, 1),
        vignette: BASE.vignette * num(P.vignette, 1),
        scanline: BASE.scan * num(P.scanline, 1),
        grain: BASE.grain * num(P.grain, 1)
      };
      return o;
    },

    save: function () { savePrefs(); return API; },
    load: function () { loadPrefs(); if (ensure()) { updateVars(); applyBackdrop(); applyEnabled(); } return API; },

    /** 恢复出厂默认（并持久化） */
    reset: function () {
      for (var k in DEFAULTS) if (Object.prototype.hasOwnProperty.call(DEFAULTS, k)) P[k] = DEFAULTS[k];
      if (ensure()) { updateVars(); applyBackdrop(); applyEnabled(); }
      savePrefs();
      return API;
    },

    /** 彻底移除特效层（还原 DOM/CSS/旧噪点层） */
    destroy: function () {
      cancelDynamic();
      setLegacyGrain(true);
      for (var id in L) if (Object.prototype.hasOwnProperty.call(L, id) && L[id] && L[id].parentNode) L[id].parentNode.removeChild(L[id]);
      L = {};
      if (styleEl && styleEl.parentNode) styleEl.parentNode.removeChild(styleEl);
      styleEl = null;
      window.removeEventListener("resize", onResize);
      var root = document.documentElement;
      if (root) {
        ["--fx-tint", "--fx-desat", "--fx-contrast", "--fx-vig", "--fx-scan", "--fx-grain", "--fx-grain2", "--fx-chroma", "--fx-backdrop"].forEach(function (k) {
          root.style.removeProperty(k);
        });
      }
      built = false;
      return API;
    }
  };

  /* 便捷别名（world2d/zone2d 侧更好读） */
  API.shake = API.screenShake;
  API.chromaPulse = API.chromatic;
  API.flash = API.hitFlash;

  window.FX = API;
})();
