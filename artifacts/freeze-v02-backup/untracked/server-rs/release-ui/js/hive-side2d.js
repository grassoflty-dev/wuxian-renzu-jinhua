/*
 * 【已停用 · DISABLED】余界 · 灰巢横版 2D 适配层 —— 改用 Babylon 第三人称（TPS）
 *
 * 停用原因：游戏已改为全流程 Babylon.js 第三人称，grey_hive（灰巢设施）也必须由
 * js/tps-renderer.js 渲染，不能再被本文件抢走。
 *
 * 它原本怎样接管（供追溯）：
 *   本文件在 index.html 中排在 js/renderer-boot.js 之前加载，进入时先抓走
 *   window.CampaignRenderer（当时是 js/renderer.js 的 Three.js 等距渲染器）存为
 *   legacy，然后把 window.CampaignRenderer 换成自己的适配层 api。之后 app.js 拿到
 *   的是本适配层：只要 setData() 里 isHive(next) 命中（world_id === "grey_hive"
 *   或名字含「灰巢」），就 dispose 掉 legacy、用 cloneNode 换掉 canvas 表面（丢掉
 *   WebGL 上下文），改由 Canvas2D 画横版侧视场景，debug() 返回
 *   mode: "canvas2d-hive-side"。其余世界才转发给 legacy。
 *
 * 现在的加载顺序（release-ui/index.html 实际顺序，本文件不在其中）：
 *   vendor/three.min.js -> js/renderer.js -> vendor/babylon.js ->
 *   js/tps-renderer.js -> js/renderer-boot.js -> js/app.js
 * 由此 renderer-boot.js 的 three_available 回退目标恢复为真正的 Three.js 等距渲染器
 * （此前会被本适配层顶替），其防递归三重保险（加载时抓 api 快照 / init 返回自引用即
 * 拒绝 / 调用栈深度保护）不受影响。
 *
 * 文件保留备查，请勿删除。若将来确实要恢复横版：必须把
 * <script src="js/hive-side2d.js"></script> 插回 js/renderer-boot.js 之前，否则两层
 * 会互相包装成自引用递归；并且要显式设置 window.__ENABLE_HIVE_SIDE2D__ = true
 * 才能通过下面的停用开关。
 *
 * 原文件头（保留原意，供追溯）：
 * This file is intentionally loaded after renderer.js.  It keeps the legacy
 * renderer intact for every world except grey_hive, while giving the first
 * world a deterministic Canvas2D presentation with no cutaway geometry.
 */
(function installHiveSide2D(global) {
  "use strict";

  // 停用开关：全流程 TPS，本适配层默认不再接管任何世界。即便有人把脚本标签插回来，
  // 这里也会直接让位给 window.CampaignRenderer（renderer-boot.js 的 TPS 引导层）。
  if (global.__ENABLE_HIVE_SIDE2D__ !== true) {
    if (global.console && global.console.warn) {
      global.console.warn("[hive-side2d] 已停用：灰巢改由 Babylon 第三人称（TPS）渲染，本适配层不接管。");
    }
    return;
  }

  var legacy = global.CampaignRenderer;
  var canvas = null;
  var ctx = null;
  var map = null;
  var player = {};
  var callbacks = { onTarget: null, onHover: null };
  var currentMode = "uninitialized";
  var legacyReady = false;
  var initialized = false;
  var raf = null;
  var lastTime = 0;
  var width = 1;
  var height = 1;
  var dpr = 1;
  var quality = "high";
  var camera = { x: 0, y: 0 };
  var effects = [];
  var images = {};
  var imageSources = {
    far: "assets/hive2d/hive-bg-far.png",
    mid: "assets/hive2d/hive-bg-mid.png",
    idle: "assets/hive2d/cenyao-side-idle.png",
    runA: "assets/hive2d/cenyao-side-run-a.png",
    runB: "assets/hive2d/cenyao-side-run-b.png",
  };
  var playerVisual = { x: 0, y: 0, tx: 0, ty: 0, facing: 1, moving: false, gait: 0 };
  var pointer = { x: 0, y: 0, inside: false };
  var hoveredTarget = null;
  var frameCount = 0;
  var worldStart = 0;

  var TAU = Math.PI * 2;
  var COLORS = {
    ink: "#101820",
    void: "#081016",
    concrete: "#2c3b40",
    concreteDark: "#1b282e",
    edge: "#607076",
    cyan: "#86d4d2",
    cyanBright: "#d4ffff",
    amber: "#e4b863",
    warning: "#df7657",
    purple: "#b79aff",
    white: "#eef7f2",
  };

  function now() { return (global.performance && global.performance.now) ? global.performance.now() : Date.now(); }
  function n(value, fallback) { return typeof value === "number" && isFinite(value) ? value : fallback; }
  function clamp(value, min, max) { return Math.max(min, Math.min(max, value)); }
  function text(value) { return value == null ? "" : String(value); }
  function isHive(value) {
    var id = text(value && (value.world_id || value.worldId || value.id || value));
    return id === "grey_hive" || id.indexOf("灰巢") >= 0;
  }
  function row(value) { return Array.isArray(value) ? value.join("") : text(value); }
  function normalized(input) {
    var source = input && input.map && Array.isArray(input.map.tiles) ? input.map : input;
    if (!source || !Array.isArray(source.tiles)) return null;
    var tiles = source.tiles.map(row);
    var inferredW = tiles.reduce(function (max, item) { return Math.max(max, item.length); }, 0);
    var w = Math.max(1, Math.round(n(source.w, inferredW)));
    var h = Math.max(1, Math.round(n(source.h, tiles.length)));
    return {
      world_id: text(source.world_id || source.worldId || source.id),
      world_name: text(source.world_name || source.name || "灰巢设施"),
      w: w, h: h, tiles: tiles,
      px: n(source.px, 0), py: n(source.py, 0), revision: source.revision,
      enemies: Array.isArray(source.enemies) ? source.enemies : [],
      npcs: Array.isArray(source.npcs) ? source.npcs : [],
      points: Array.isArray(source.points) ? source.points : [],
      gates: Array.isArray(source.gates) ? source.gates : [],
      portals: Array.isArray(source.portals) ? source.portals : [],
      props: Array.isArray(source.props) ? source.props : [],
    };
  }
  function tileAt(x, y) {
    if (!map || x < 0 || y < 0 || x >= map.w || y >= map.h) return "#";
    return (map.tiles[y] || "")[x] || "#";
  }
  function loadImages() {
    if (!global.Image) return;
    Object.keys(imageSources).forEach(function (key) {
      if (images[key]) return;
      var image = new global.Image();
      image.decoding = "async";
      image.onload = function () { images[key] = image; };
      image.onerror = function () { images[key] = null; };
      image.src = imageSources[key];
    });
  }
  function ensureLegacy() {
    if (legacyReady || !legacy || typeof legacy.init !== "function") return;
    legacy.init(canvas, callbacks);
    legacyReady = true;
    currentMode = "legacy";
  }
  function ensureCanvas() {
    if (!canvas) return null;
    if (!ctx && canvas.getContext) ctx = canvas.getContext("2d");
    return ctx;
  }
  function replaceCanvasSurface() {
    if (!canvas || !canvas.parentNode || !canvas.cloneNode) return;
    var old = canvas;
    var replacement = old.cloneNode(false);
    replacement.width = old.width;
    replacement.height = old.height;
    old.parentNode.replaceChild(replacement, old);
    old.removeEventListener("mousemove", onMove);
    old.removeEventListener("mouseleave", onLeave);
    old.removeEventListener("click", onClick);
    canvas = replacement;
    canvas.addEventListener("mousemove", onMove);
    canvas.addEventListener("mouseleave", onLeave);
    canvas.addEventListener("click", onClick);
    ctx = null;
    resize();
  }
  function isSide() { return currentMode === "hive-side2d"; }
  function prepareSide() {
    ensureCanvas();
    currentMode = "hive-side2d";
    worldStart = worldStart || now();
    loadImages();
    resize();
    start();
  }
  function resize() {
    if (currentMode === "legacy" && legacy && legacy.resize) return legacy.resize();
    if (!canvas) return;
    var rect = canvas.getBoundingClientRect ? canvas.getBoundingClientRect() : { width: canvas.clientWidth || 1, height: canvas.clientHeight || 1 };
    width = Math.max(1, rect.width || canvas.clientWidth || 1);
    height = Math.max(1, rect.height || canvas.clientHeight || 1);
    dpr = quality === "low" ? 1 : Math.min(2, global.devicePixelRatio || 1);
    canvas.width = Math.max(1, Math.round(width * dpr));
    canvas.height = Math.max(1, Math.round(height * dpr));
    if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }
  function start() {
    if (raf != null) return;
    raf = (global.requestAnimationFrame || function (fn) { return global.setTimeout(function () { fn(now()); }, 16); })(frame);
  }
  function stop() {
    if (raf == null) return;
    if (global.cancelAnimationFrame) global.cancelAnimationFrame(raf); else global.clearTimeout(raf);
    raf = null;
  }
  function updatePlayer(dt) {
    if (!map) return;
    var targetX = n(playerVisual.tx, n(map.px, 0));
    var targetY = n(playerVisual.ty, n(map.py, 0));
    var oldX = playerVisual.x;
    var oldY = playerVisual.y;
    var blend = 1 - Math.pow(0.0008, Math.max(0.001, dt));
    playerVisual.x += (targetX - playerVisual.x) * blend;
    playerVisual.y += (targetY - playerVisual.y) * blend;
    var moving = Math.abs(targetX - playerVisual.x) + Math.abs(targetY - playerVisual.y) > 0.025 || Math.abs(oldX - playerVisual.x) + Math.abs(oldY - playerVisual.y) > 0.002;
    playerVisual.moving = moving;
    if (moving) playerVisual.gait += dt * 8.5;
    if (targetX !== oldX) playerVisual.facing = targetX > oldX ? 1 : -1;
  }
  function worldScale() {
    if (!map) return 56;
    // A side-scroller needs a character-sized camera, not a bird's-eye view
    // of the whole grid. The backend grid remains authoritative while the
    // presentation shows roughly fourteen columns and seven facility levels.
    return clamp(Math.min(width / 14, height / 7), 42, 74);
  }
  function scenePoint(x, y) {
    var s = worldScale();
    return { x: width * 0.5 + (x - camera.x) * s, y: height * 0.76 - (y - camera.y) * s * 0.72 };
  }
  function updateCamera() {
    if (!map) return;
    var targetX = playerVisual.x;
    var targetY = playerVisual.y;
    var s = worldScale();
    var visibleW = width / s;
    var visibleH = height / (s * 0.72);
    var desiredX = targetX + (playerVisual.facing * 1.4);
    camera.x += (desiredX - camera.x) * 0.08;
    camera.y += (targetY - camera.y) * 0.07;
    camera.x = clamp(camera.x, visibleW * 0.5 - 1, Math.max(visibleW * 0.5, map.w - visibleW * 0.5 + 1));
    camera.y = clamp(camera.y, visibleH * 0.5 - 1, Math.max(visibleH * 0.5, map.h - visibleH * 0.5 + 1));
  }
  function fillRound(x, y, w, h, r, fill, stroke, line) {
    if (!ctx) return;
    ctx.beginPath();
    ctx.roundRect ? ctx.roundRect(x, y, w, h, r) : ctx.rect(x, y, w, h);
    if (fill) { ctx.fillStyle = fill; ctx.fill(); }
    if (stroke) { ctx.strokeStyle = stroke; ctx.lineWidth = line || 1; ctx.stroke(); }
  }
  function drawBackground(time) {
    var g = ctx.createLinearGradient(0, 0, 0, height);
    g.addColorStop(0, "#09131b"); g.addColorStop(0.55, "#12232b"); g.addColorStop(1, "#070d12");
    ctx.fillStyle = g; ctx.fillRect(0, 0, width, height);
    if (images.far) {
      var farX = -((camera.x * 8) % Math.max(1, width));
      ctx.globalAlpha = 0.58; ctx.drawImage(images.far, farX, 0, width, height * 0.86); ctx.drawImage(images.far, farX + width, 0, width, height * 0.86); ctx.globalAlpha = 1;
    } else {
      ctx.fillStyle = "#15252b";
      for (var i = 0; i < 9; i += 1) {
        var bx = ((i * 167 - camera.x * 12) % (width + 240)) - 120;
        var bh = 100 + (i % 4) * 43;
        ctx.fillRect(bx, height * 0.25 - bh, 84 + (i % 3) * 30, bh);
        ctx.fillStyle = i % 2 ? "#c39b52" : "#557b78";
        ctx.fillRect(bx + 18, height * 0.25 - bh + 26, 4, 18);
        ctx.fillStyle = "#15252b";
      }
    }
    if (images.mid) {
      var midX = -((camera.x * 20) % Math.max(1, width));
      ctx.globalAlpha = 0.46; ctx.drawImage(images.mid, midX, height * 0.06, width, height * 0.82); ctx.drawImage(images.mid, midX + width, height * 0.06, width, height * 0.82); ctx.globalAlpha = 1;
    } else {
      ctx.strokeStyle = "rgba(105,157,157,.22)"; ctx.lineWidth = 3;
      for (var p = -1; p < 10; p += 1) {
        var px = ((p * 210 - camera.x * 24) % (width + 260)) - 130;
        ctx.beginPath(); ctx.moveTo(px, height * 0.22); ctx.bezierCurveTo(px + 30, height * 0.38, px - 32, height * 0.55, px + 8, height * 0.7); ctx.stroke();
      }
      ctx.fillStyle = "rgba(224,179,94,.15)";
      for (var l = 0; l < 8; l += 1) { var lx = ((l * 233 - camera.x * 12) % (width + 180)) - 90; ctx.fillRect(lx, height * 0.34 + (l % 3) * 42, 3, 22); }
    }
    ctx.fillStyle = "rgba(8,16,20,.42)"; ctx.fillRect(0, height * 0.78, width, height * 0.22);
    ctx.fillStyle = "rgba(139,207,198,.045)";
    for (var scan = 0; scan < height; scan += 5) ctx.fillRect(0, scan, width, 1);
    ctx.globalAlpha = 0.18 + Math.sin(time * 0.001) * 0.035;
    ctx.fillStyle = "#d6c18b"; ctx.fillRect(width * 0.05, height * 0.22, 3, 55); ctx.fillRect(width * 0.82, height * 0.16, 2, 78);
    ctx.globalAlpha = 1;
  }
  function drawTileWorld(time) {
    if (!map) return;
    var s = worldScale();
    var minX = Math.floor(camera.x - width / s / 2) - 1;
    var maxX = Math.ceil(camera.x + width / s / 2) + 1;
    var minY = Math.floor(camera.y - height / s / 2) - 1;
    var maxY = Math.ceil(camera.y + height / s / 2) + 1;
    for (var y = minY; y <= maxY; y += 1) {
      for (var x = minX; x <= maxX; x += 1) {
        var tile = tileAt(x, y); if (tile === "#") continue;
        var point = scenePoint(x, y);
        // Walkable cells are negative space. Only their deck edge is drawn,
        // turning the old navigation grid into authored-looking corridors.
        if (tile === "~") {
          ctx.fillStyle = "rgba(33,107,113,.25)";
          ctx.fillRect(point.x - s * .52, point.y + s * .17, s * 1.04, s * .32);
        }
        var belowSolid = tileAt(x, y + 1) === "#" || y + 1 >= map.h;
        if (belowSolid) {
          var deck = ctx.createLinearGradient(0, point.y + s * .18, 0, point.y + s * .42);
          deck.addColorStop(0, "#819493"); deck.addColorStop(.16, "#4e6264"); deck.addColorStop(1, "#18262b");
          ctx.fillStyle = deck;
          ctx.fillRect(point.x - s * .53, point.y + s * .20, s * 1.06, s * .22);
          ctx.fillStyle = "rgba(231,190,105,.30)";
          ctx.fillRect(point.x - s * .48, point.y + s * .20, s * .28, 2);
          ctx.fillStyle = "rgba(4,9,12,.55)";
          ctx.fillRect(point.x - s * .53, point.y + s * .40, s * 1.06, s * .07);
        }
        if (tileAt(x - 1, y) === "#") {
          ctx.fillStyle = "#46595c"; ctx.fillRect(point.x - s * .52, point.y - s * .58, s * .13, s * .84);
          ctx.fillStyle = "rgba(223,181,95,.25)"; ctx.fillRect(point.x - s * .49, point.y - s * .50, 2, s * .58);
        }
        if (tileAt(x + 1, y) === "#") {
          ctx.fillStyle = "#34474b"; ctx.fillRect(point.x + s * .39, point.y - s * .58, s * .13, s * .84);
        }
      }
    }
    // Ceiling beams separate stacked routes without filling the view with
    // opaque wall squares.
    for (var wy = minY; wy <= maxY; wy += 1) {
      for (var wx = minX; wx <= maxX; wx += 1) {
        if (tileAt(wx, wy) !== "#" || tileAt(wx, wy + 1) === "#") continue;
        var wall = scenePoint(wx, wy);
        ctx.fillStyle = "#283b40";
        ctx.fillRect(wall.x - s * .53, wall.y + s * .30, s * 1.06, s * .16);
        ctx.fillStyle = "#718282";
        ctx.fillRect(wall.x - s * .53, wall.y + s * .30, s * 1.06, 2);
      }
    }
    drawIndustrialProps(time, s);
  }
  function drawIndustrialProps(time, s) {
    var list = (map && map.props) || [];
    list.forEach(function (prop, index) {
      var x = n(prop.x, 0), y = n(prop.y, 0), p = scenePoint(x, y);
      var kind = text(prop.kind || prop.type || prop.id).toLowerCase();
      ctx.save(); ctx.translate(p.x, p.y - s * .18);
      if (kind.indexOf("lamp") >= 0 || kind.indexOf("light") >= 0) {
        ctx.fillStyle = "#d7ad5d"; ctx.fillRect(-2, -s * .7, 4, s * .7);
        ctx.fillStyle = "rgba(228,184,99,.18)"; ctx.beginPath(); ctx.arc(0, -s * .72, s * .55, 0, TAU); ctx.fill();
      } else if (kind.indexOf("pipe") >= 0 || kind.indexOf("vent") >= 0) {
        ctx.strokeStyle = index % 2 ? "#687d7e" : "#8e9a92"; ctx.lineWidth = Math.max(3, s * .12); ctx.beginPath(); ctx.moveTo(-s * .44, -s * .4); ctx.lineTo(-s * .1, -s * .4); ctx.lineTo(s * .2, s * .15); ctx.lineTo(s * .5, s * .15); ctx.stroke();
      } else {
        // A readable maintenance console instead of a featureless map block.
        ctx.fillStyle = "#26383c"; ctx.beginPath(); ctx.moveTo(-s * .34, 0); ctx.lineTo(-s * .27, -s * .53); ctx.lineTo(s * .22, -s * .53); ctx.lineTo(s * .34, 0); ctx.closePath(); ctx.fill();
        ctx.strokeStyle = "#657879"; ctx.lineWidth = 2; ctx.stroke();
        ctx.fillStyle = "#10272c"; ctx.fillRect(-s * .18, -s * .43, s * .32, s * .20);
        ctx.fillStyle = index % 3 ? "#74c9c7" : "#e4b863"; ctx.beginPath(); ctx.arc(-s * .10, -s * .33, Math.max(2, s * .035), 0, TAU); ctx.fill();
        ctx.fillStyle = "rgba(212,255,255,.45)"; ctx.fillRect(-s * .02, -s * .35, s * .10, 2);
        ctx.strokeStyle = "rgba(228,184,99,.34)"; ctx.beginPath(); ctx.moveTo(-s * .22, -s * .12); ctx.lineTo(s * .22, -s * .12); ctx.stroke();
      }
      ctx.restore();
    });
  }
  function drawMarker(item, kind, time) {
    var x = n(item.x, 0), y = n(item.y, 0), p = scenePoint(x, y), s = worldScale();
    var pulse = 0.75 + Math.sin(time * .004 + x + y) * .15;
    ctx.save(); ctx.translate(p.x, p.y - s * .22);
    if (kind === "enemy") {
      // Side-view hostile silhouette: head, hunched torso and separated limbs.
      // The red light is an identification accent, not the body itself.
      ctx.fillStyle = "rgba(223,118,87,.13)"; ctx.beginPath(); ctx.ellipse(0, -s * .22, s * .43 * pulse, s * .58 * pulse, 0, 0, TAU); ctx.fill();
      ctx.strokeStyle = "#141b1e"; ctx.lineWidth = Math.max(4, s * .13); ctx.lineCap = "round";
      ctx.beginPath(); ctx.moveTo(-s * .08, s * .02); ctx.lineTo(-s * .17, s * .36); ctx.moveTo(s * .08, s * .02); ctx.lineTo(s * .21, s * .34); ctx.moveTo(-s * .16, -s * .22); ctx.lineTo(-s * .31, s * .02); ctx.moveTo(s * .15, -s * .20); ctx.lineTo(s * .30, -s * .02); ctx.stroke();
      ctx.fillStyle = "#283338"; ctx.beginPath(); ctx.moveTo(-s * .19, -s * .31); ctx.quadraticCurveTo(0, -s * .41, s * .20, -s * .25); ctx.lineTo(s * .14, s * .04); ctx.lineTo(-s * .14, s * .05); ctx.closePath(); ctx.fill();
      ctx.fillStyle = "#20292c"; ctx.beginPath(); ctx.arc(0, -s * .48, s * .13, 0, TAU); ctx.fill();
      ctx.fillStyle = COLORS.warning; ctx.beginPath(); ctx.arc(s * .07, -s * .49, Math.max(2, s * .035), 0, TAU); ctx.fill();
      ctx.strokeStyle = "rgba(239,137,102,.72)"; ctx.lineWidth = 1.5; ctx.beginPath(); ctx.moveTo(-s * .20, s * .43); ctx.lineTo(s * .23, s * .43); ctx.stroke();
    } else if (kind === "npc") {
      ctx.fillStyle = "rgba(99,174,141,.14)"; ctx.beginPath(); ctx.ellipse(0, -s * .16, s * .32, s * .48, 0, 0, TAU); ctx.fill();
      ctx.strokeStyle = "#1a282a"; ctx.lineWidth = Math.max(3, s * .10); ctx.lineCap = "round"; ctx.beginPath(); ctx.moveTo(-s * .06, 0); ctx.lineTo(-s * .12, s * .31); ctx.moveTo(s * .06, 0); ctx.lineTo(s * .12, s * .31); ctx.stroke();
      ctx.fillStyle = "#3d5551"; ctx.beginPath(); ctx.moveTo(-s * .15, -s * .25); ctx.lineTo(s * .14, -s * .25); ctx.lineTo(s * .11, s * .04); ctx.lineTo(-s * .12, s * .04); ctx.closePath(); ctx.fill();
      ctx.fillStyle = "#a9bbb0"; ctx.beginPath(); ctx.arc(0, -s * .39, s * .11, 0, TAU); ctx.fill();
      ctx.strokeStyle = "#88c4a0"; ctx.lineWidth = 2; ctx.beginPath(); ctx.arc(0, -s * .42, s * .20 * pulse, Math.PI * 1.10, Math.PI * 1.90); ctx.stroke();
    } else {
      // Objective / gate beacon with a narrow vertical signal profile.
      ctx.globalCompositeOperation = "lighter";
      ctx.fillStyle = "rgba(228,184,99,.10)"; ctx.beginPath(); ctx.ellipse(0, -s * .17, s * .28 * pulse, s * .62 * pulse, 0, 0, TAU); ctx.fill();
      ctx.strokeStyle = COLORS.amber; ctx.lineWidth = Math.max(2, s * .035); ctx.beginPath(); ctx.moveTo(0, s * .30); ctx.lineTo(0, -s * .37); ctx.stroke();
      ctx.beginPath(); ctx.moveTo(-s * .11, -s * .23); ctx.lineTo(0, -s * .37); ctx.lineTo(s * .11, -s * .23); ctx.lineTo(0, -s * .09); ctx.closePath(); ctx.stroke();
      ctx.fillStyle = COLORS.amber; ctx.beginPath(); ctx.arc(0, -s * .23, Math.max(2, s * .03), 0, TAU); ctx.fill();
    }
    ctx.restore();
  }
  function drawActors(time) {
    if (!map) return;
    var actors = [];
    (map.points || []).forEach(function (item) { actors.push({ item: item, kind: "point" }); });
    (map.portals || []).forEach(function (item) { actors.push({ item: item, kind: "point" }); });
    (map.gates || []).forEach(function (item) { actors.push({ item: item, kind: "point" }); });
    (map.npcs || []).forEach(function (item) { actors.push({ item: item, kind: "npc" }); });
    (map.enemies || []).forEach(function (item) { actors.push({ item: item, kind: "enemy" }); });
    actors.sort(function (a, b) { return n(a.item.y, 0) - n(b.item.y, 0); });
    actors.forEach(function (entry) { drawMarker(entry.item, entry.kind, time); });
    drawPlayer(time);
  }
  function drawHumanFallback(time) {
    var s = worldScale(), bob = playerVisual.moving ? Math.sin(playerVisual.gait) * s * .035 : Math.sin(time * .003) * s * .012;
    ctx.save(); ctx.translate(0, bob); ctx.scale(playerVisual.facing, 1);
    ctx.fillStyle = "rgba(0,0,0,.38)"; ctx.beginPath(); ctx.ellipse(0, s * .34, s * .31, s * .09, 0, 0, TAU); ctx.fill();
    ctx.strokeStyle = "#11191d"; ctx.lineWidth = s * .13; ctx.lineCap = "round";
    ctx.beginPath(); ctx.moveTo(-s * .09, s * .12); ctx.lineTo(-s * .14, s * .38 + (playerVisual.moving ? Math.sin(playerVisual.gait) * s * .08 : 0)); ctx.moveTo(s * .09, s * .12); ctx.lineTo(s * .14, s * .38 - (playerVisual.moving ? Math.sin(playerVisual.gait) * s * .08 : 0)); ctx.stroke();
    ctx.fillStyle = "#24363b"; ctx.beginPath(); ctx.moveTo(-s * .25, -s * .17); ctx.lineTo(s * .18, -s * .17); ctx.lineTo(s * .25, s * .18); ctx.lineTo(-s * .2, s * .18); ctx.closePath(); ctx.fill();
    ctx.fillStyle = "#789395"; ctx.fillRect(-s * .07, -s * .2, s * .16, s * .28);
    ctx.fillStyle = "#d9b15f"; ctx.beginPath(); ctx.moveTo(-s * .21, -s * .18); ctx.lineTo(-s * .04, -s * .2); ctx.lineTo(-s * .08, -s * .03); ctx.lineTo(-s * .26, -s * .01); ctx.closePath(); ctx.fill();
    ctx.fillStyle = "#d1a95b"; ctx.beginPath(); ctx.moveTo(s * .02, -s * .03); ctx.lineTo(s * .11, s * .04); ctx.lineTo(s * .02, s * .13); ctx.lineTo(-s * .07, s * .04); ctx.closePath(); ctx.fill();
    ctx.fillStyle = "#b9c4bd"; ctx.beginPath(); ctx.arc(0, -s * .38, s * .16, 0, TAU); ctx.fill();
    ctx.fillStyle = "#11191d"; ctx.beginPath(); ctx.arc(-s * .02, -s * .43, s * .17, Math.PI, TAU); ctx.fill(); ctx.fillRect(-s * .17, -s * .43, s * .27, s * .08);
    ctx.restore();
  }
  function drawPlayer(time) {
    var p = scenePoint(playerVisual.x, playerVisual.y);
    ctx.save(); ctx.translate(p.x, p.y - worldScale() * .22);
    var image = playerVisual.moving ? (Math.floor(playerVisual.gait) % 2 ? images.runA : images.runB) : images.idle;
    if (image && image.complete && image.naturalWidth) {
      var s = worldScale(), h = s * 3.05, w = h * image.naturalWidth / image.naturalHeight;
      ctx.globalAlpha = .22; ctx.fillStyle = "#000"; ctx.beginPath(); ctx.ellipse(0, h * .48, w * .28, h * .06, 0, 0, TAU); ctx.fill(); ctx.globalAlpha = 1;
      ctx.scale(playerVisual.facing, 1); ctx.drawImage(image, -w * .5, -h * .82, w, h);
    } else {
      drawHumanFallback(time);
    }
    ctx.restore();
  }
  function effectOrigin(effect) {
    var x = n(effect && (effect.x != null ? effect.x : effect.px), playerVisual.x);
    var y = n(effect && (effect.y != null ? effect.y : effect.py), playerVisual.y);
    var p = scenePoint(x, y);
    // The backend coordinate is the occupied map cell. Lift spell origins to
    // the character's torso so pulses, shields and lances do not read as
    // floor decals beneath the feet.
    return { x: p.x, y: p.y - worldScale() * 1.10 };
  }
  function drawPulse(effect, age) {
    var o = effectOrigin(effect), s = worldScale(), progress = clamp(age / 1.6, 0, 1), radius = s * (.22 + progress * 2.35);
    ctx.save(); ctx.translate(o.x, o.y); ctx.globalCompositeOperation = "lighter";
    for (var ring = 0; ring < 3; ring += 1) {
      var r = radius * (0.58 + ring * .19); ctx.globalAlpha = (1 - progress) * (.92 - ring * .14); ctx.strokeStyle = ring === 1 ? COLORS.cyanBright : COLORS.cyan; ctx.lineWidth = Math.max(3, s * .072);
      ctx.beginPath();
      for (var i = 0; i <= 20; i += 1) { var a = i / 20 * TAU; var wobble = 1 + Math.sin(a * (3 + ring) + ring) * .08 + Math.sin(a * 7 + age * 5) * .035; var px = Math.cos(a) * r * wobble; var py = Math.sin(a) * r * wobble * .55; if (i === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py); }
      ctx.stroke();
    }
    for (var dot = 0; dot < 18; dot += 1) { var da = dot / 18 * TAU + age * .9; var dr = radius * (.45 + (dot % 4) * .12); ctx.globalAlpha = (1 - progress) * .7; ctx.fillStyle = dot % 3 ? COLORS.cyan : COLORS.cyanBright; ctx.beginPath(); ctx.arc(Math.cos(da) * dr, Math.sin(da) * dr * .52, Math.max(1.5, s * .035), 0, TAU); ctx.fill(); }
    ctx.restore();
  }
  function drawShield(effect, age) {
    var o = effectOrigin(effect), s = worldScale(), progress = clamp(age / 1.8, 0, 1), alpha = Math.min(1, progress * 7, (1 - progress) * 4);
    ctx.save(); ctx.translate(o.x, o.y); ctx.globalCompositeOperation = "lighter"; ctx.globalAlpha = alpha * .92;
    ctx.fillStyle = "rgba(154,125,255,.25)"; ctx.beginPath(); ctx.arc(0, 0, s * 1.02, Math.PI, TAU); ctx.lineTo(s * 1.02, s * .18); ctx.quadraticCurveTo(0, s * .72, -s * 1.02, s * .18); ctx.closePath(); ctx.fill();
    ctx.strokeStyle = COLORS.purple; ctx.lineWidth = Math.max(3, s * .075); ctx.beginPath(); ctx.arc(0, 0, s * (.88 + Math.sin(age * 7) * .05), Math.PI * 1.08, Math.PI * 1.92); ctx.stroke();
    ctx.strokeStyle = "rgba(222,210,255,.7)"; ctx.lineWidth = 1.5; for (var i = 0; i < 7; i += 1) { var a = Math.PI * 1.08 + i / 6 * Math.PI * .84; var r = s * .72; ctx.beginPath(); ctx.moveTo(Math.cos(a) * r, Math.sin(a) * r); ctx.lineTo(Math.cos(a + .08) * r * .74, Math.sin(a + .08) * r * .74); ctx.stroke(); }
    ctx.restore();
  }
  function drawPierce(effect, age) {
    var o = effectOrigin(effect), s = worldScale(), progress = clamp(age / 1.4, 0, 1), dx = n(effect.dx, effect.direction_x), dy = n(effect.dy, effect.direction_y), dir = dx < 0 ? -1 : 1;
    ctx.save(); ctx.translate(o.x, o.y); ctx.scale(dir, 1); ctx.globalCompositeOperation = "lighter"; ctx.globalAlpha = progress < .62 ? 1 : Math.max(0, (1 - progress) / .38);
    var reach = s * (1.1 + progress * 2.8); ctx.strokeStyle = "rgba(255,223,138,.45)"; ctx.lineWidth = s * .18; ctx.beginPath(); ctx.moveTo(-s * .12, 0); ctx.lineTo(reach, 0); ctx.stroke();
    ctx.fillStyle = "#ffe6a4"; ctx.beginPath(); ctx.moveTo(-s * .16, -s * .13); ctx.lineTo(reach, 0); ctx.lineTo(-s * .16, s * .13); ctx.lineTo(s * .08, 0); ctx.closePath(); ctx.fill();
    ctx.strokeStyle = "#fff5d2"; ctx.lineWidth = Math.max(2, s * .045); ctx.beginPath(); ctx.moveTo(-s * .1, 0); ctx.lineTo(reach, 0); ctx.stroke();
    ctx.strokeStyle = "rgba(255,168,88,.85)"; ctx.lineWidth = s * .035; ctx.beginPath(); ctx.moveTo(reach, -s * .28); ctx.lineTo(reach + s * .22, 0); ctx.lineTo(reach, s * .28); ctx.moveTo(reach - s * .15, -s * .2); ctx.lineTo(reach + s * .12, 0); ctx.lineTo(reach - s * .15, s * .2); ctx.stroke();
    ctx.restore();
  }
  function drawEffects(time) {
    effects = effects.filter(function (effect) { var age = (time - effect.born) / 1000; return age < n(effect.duration, effect.kind === "skill_1" ? 1.8 : effect.kind === "skill_2" ? 1.4 : 1.6); });
    effects.forEach(function (effect) { var age = (time - effect.born) / 1000; if (effect.kind === "skill_0" || effect.kind === "pulse") drawPulse(effect, age); else if (effect.kind === "skill_1") drawShield(effect, age); else if (effect.kind === "skill_2") drawPierce(effect, age); });
  }
  function drawForeground(time) {
    ctx.save(); ctx.globalAlpha = .23; ctx.strokeStyle = "#7da09b"; ctx.lineWidth = 2;
    for (var i = -1; i < 7; i += 1) { var x = ((i * 260 - camera.x * 34) % (width + 300)) - 120; ctx.beginPath(); ctx.moveTo(x, 0); ctx.bezierCurveTo(x + 25, height * .18, x - 18, height * .35, x + 12, height * .52); ctx.stroke(); }
    ctx.globalAlpha = .18; ctx.fillStyle = "#d7e4dc"; ctx.fillRect(0, height * .77, width, 2); ctx.restore();
  }
  function draw(time) {
    if (!ctx || !isSide()) return;
    var dt = lastTime ? Math.min(.05, (time - lastTime) / 1000) : .016; lastTime = time; frameCount += 1;
    updatePlayer(dt); updateCamera();
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0); ctx.clearRect(0, 0, width, height);
    drawBackground(time); drawTileWorld(time); drawActors(time); drawEffects(time); drawForeground(time);
  }
  function frame(time) {
    raf = null;
    if (isSide()) { draw(time || now()); start(); }
  }
  function eventPosition(event) {
    var rect = canvas.getBoundingClientRect(); return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }
  function targetAt(clientX, clientY) {
    if (currentMode === "legacy" && legacy && legacy.targetAt) return legacy.targetAt(clientX, clientY);
    if (!map || !canvas) return null;
    var rect = canvas.getBoundingClientRect(); var sx = clientX - rect.left, sy = clientY - rect.top, s = worldScale();
    var wx = camera.x + (sx - width * .5) / s; var wy = camera.y - (sy - height * .80) / s;
    var dx = Math.sign(wx - playerVisual.x), dy = Math.sign(wy - playerVisual.y);
    if (!dx && !dy) return null;
    return { x: Math.round(wx), y: Math.round(wy), dx: dx, dy: dy };
  }
  function onMove(event) {
    if (!isSide()) return;
    pointer = eventPosition(event); pointer.inside = true;
    hoveredTarget = targetAt(event.clientX, event.clientY);
    if (callbacks.onHover) callbacks.onHover(hoveredTarget);
  }
  function onLeave() { pointer.inside = false; hoveredTarget = null; if (callbacks.onHover) callbacks.onHover(null); }
  function onClick(event) { if (!isSide()) return; var target = targetAt(event.clientX, event.clientY); if (target && callbacks.onTarget) callbacks.onTarget(target); }
  function init(cv, options) {
    if (!cv) throw new Error("CampaignRenderer.init requires a canvas");
    dispose(); canvas = cv; options = options || {}; callbacks.onTarget = options.onTarget || null; callbacks.onHover = options.onHover || null; initialized = true;
    canvas.addEventListener("mousemove", onMove); canvas.addEventListener("mouseleave", onLeave); canvas.addEventListener("click", onClick);
    if (global.addEventListener) global.addEventListener("resize", resize);
    resize();
    return api;
  }
  function setData(input) {
    var next = normalized(input);
    if (!next) { map = null; if (currentMode === "legacy" && legacy && legacy.setData) return legacy.setData(input); return; }
    if (isHive(next)) {
      if (currentMode === "legacy" && legacy && legacy.dispose) { legacy.dispose(); legacyReady = false; }
      if (currentMode === "legacy") replaceCanvasSurface();
      map = next; prepareSide();
      var tx = n(next.px, 0), ty = n(next.py, 0);
      if (!playerVisual.x && !playerVisual.y) { playerVisual.x = tx; playerVisual.y = ty; }
      playerVisual.tx = tx; playerVisual.ty = ty;
      return;
    }
    if (isSide()) { stop(); map = null; replaceCanvasSurface(); ctx = null; currentMode = "uninitialized"; }
    ensureLegacy();
    if (legacy && legacy.setData) legacy.setData(input);
  }
  function setEffect(effect) {
    if (currentMode === "legacy" && legacy && legacy.setEffect) return legacy.setEffect(effect);
    if (!isSide() || !effect || typeof effect !== "object") return;
    var kind = text(effect.kind); if (["skill_0", "skill_1", "skill_2", "pulse"].indexOf(kind) < 0) return;
    effects.push(Object.assign({}, effect, { born: now() })); start();
  }
  function setPlayerState(next) {
    if (currentMode === "legacy" && legacy && legacy.setPlayerState) return legacy.setPlayerState(next);
    if (!next || typeof next !== "object") return;
    player = Object.assign({}, player, next);
    var x = next.x != null ? next.x : next.px, y = next.y != null ? next.y : next.py;
    if (x != null) playerVisual.tx = n(x, playerVisual.tx);
    if (y != null) playerVisual.ty = n(y, playerVisual.ty);
    if (next.dx) playerVisual.facing = n(next.dx, playerVisual.facing) < 0 ? -1 : 1;
  }
  function dispose() {
    stop();
    if (canvas) { canvas.removeEventListener("mousemove", onMove); canvas.removeEventListener("mouseleave", onLeave); canvas.removeEventListener("click", onClick); }
    if (global.removeEventListener) global.removeEventListener("resize", resize);
    if (currentMode === "legacy" && legacyReady && legacy && legacy.dispose) legacy.dispose();
    canvas = null; ctx = null; map = null; effects = []; legacyReady = false; initialized = false; currentMode = "uninitialized"; hoveredTarget = null; lastTime = 0;
  }
  function debug() {
    if (currentMode === "legacy" && legacy && legacy.debug) return legacy.debug();
    return { mode: "canvas2d-hive-side", initialized: initialized, world_id: map ? map.world_id : null, map: map ? { w: map.w, h: map.h, revision: map.revision } : null, actors: map ? map.enemies.length + map.npcs.length : 0, effects: effects.length, viewport: { width: width, height: height, dpr: dpr }, quality: quality, camera: { x: +camera.x.toFixed(2), y: +camera.y.toFixed(2) }, frame: frameCount, target: hoveredTarget };
  }
  function setQuality(next) {
    if (currentMode === "legacy" && legacy && legacy.setQuality) return legacy.setQuality(next);
    quality = typeof next === "number" ? (next < .55 ? "low" : next < .85 ? "medium" : "high") : (next === "low" || next === "medium" ? next : "high");
    resize();
  }
  var api = { init: init, setData: setData, setEffect: setEffect, setPlayerState: setPlayerState, resize: resize, dispose: dispose, debug: debug, targetAt: targetAt, setQuality: setQuality };
  global.CampaignRenderer = api;
})(typeof window !== "undefined" ? window : this);
