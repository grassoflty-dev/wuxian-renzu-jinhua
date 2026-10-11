/* zone2d.js — 2D 横版战斗副本引擎（魂斗罗/双截龙手感，判定权威在 Rust）
 * window.Zone2D，契约对齐 Zone3D：init/setData/start/stop/dispose/onZoneUpdate/keydown/keyup/
 * setResolution/castEquippedSkill + 新增 setFlying/setSwingMode/debug。
 * 蛛丝摆荡（血统 spiderman_bloodline 或 setSwingMode(true) 启用）：按住鼠标左键或 J
 *       朝准星发射蛛丝命中实心格顶部作锚点 → 钟摆运动 → 松开沿切向抛出惯性飞行。
 * 镜头：FOLLOW（走廊滚动，55/45% 前瞻 + 垂直带 62~68%）/ SHAFT（竖井连续跟 60%）/
 *       LOCKED（小房间锁中心，房间宽/高 < 视口×1.1）/ BOSS_INTRO（关门+震屏+推近+红框）。
 * 物理：run 2.4 px/f、跳 9.6/重 0.48、爬墙 1.0、墙跳 5.5/7.5、浮空 1.6/1.2（art px @60fps，dt 无关）。
 * 战斗：鼠标左键 → onAction("attack",0) 回合结算；Shift → dodge；Q/E/R → castEquippedSkill；
 *       onZoneUpdate({win}) → 胜利/死亡表现。前端只做表现，不判伤害。
 */
"use strict";

window.Zone2D = (function () {
  const ART = 48, TILE = 48, CHUNK = 16;
  const ROOT = "assets/img/";
  const clamp = (v, a, b) => (v < a ? a : v > b ? b : v);

  let cv = null, ctx = null, container = null;
  let dpr = 1, zoom = 2.0, level = 1080;
  let data = null;              // setData 全量
  let map = null;               // map 数据（tiles/rooms/spawns）
  let opts = null;
  let raf = null, keys = {};
  let lastT = 0, lastRenderMs = 0;
  let flying = false;
  let zoneActive = false;

  // 玩家（art px 坐标，脚锚）
  let P = { x: 0, y: 0, vx: 0, vy: 0, facing: 1, onGround: true, climbing: false, wallSide: 0, flyOff: 0 };
  let state = "idle", stateT = 0;      // idle/run/jump/climb/attack/hurt/dodge/die/fly
  let attackCd = 0, dodgeCd = 0, wallJumpLock = 0, dropT = 0;
  let hurtFlash = 0, redFlash = 0;       // 表现计时（atkFlash/dmgFloats 无消费方，已清理）
  let afterImages = [];
  let fx = [];                           // 瞬发特效（技能法阵/弹道残影），render 中按帧消耗
  let castAnim = null;                   // 技能/道具"取物→举起→释放"短演出 { t, color }（纯表现）

  // 蜘蛛侠蛛丝摆荡（前端表现；data.bloodline === "spiderman_bloodline" 或 setSwingMode 强制开关）
  let swingEnabled = false;          // 当前是否允许摆荡
  let swingForced = null;            // null=自动(读血统) / true|false=强制覆盖
  let swing = null;                  // {ax,ay,len:绳长,theta:从正下方角,vT:切向速度} 摆荡态；null=未摆荡
  let swingHold = false;             // 鼠标左键或 J 是否按住
  let swingFireCd = 0;               // 重发射冷却（防键盘重复触发锚点抖动）
  let mouseScreen = null;            // 鼠标物理坐标 {x,y}（瞄准用，CSS*dpr）
  let curTrans = { s: 1, tx: 0, ty: 0 };   // 渲染变换（鼠标反算世界坐标）
  const SWING_MIN = 120, SWING_MAX = 280;   // 绳长下限/上限（art px）
  const SWING_SHOOT_MAX = 1240;             // 射程上限
  const SWING_G = 0.96 * 60 * 60;          // 重力系数（与跳跃一致 px/s²）→ 摆荡周期≈1.5s

  /* ---------------- 血统激活外观（2D 光环） ----------------
   * 由 ui_3d_archive/js/aura.js 的 BLOODLINE_AURAS 映射复刻到 Canvas 2D。
   * 每个血统：color 主色(hex)、form 粒子形态（ring 环带 / column 升腾柱 / helix 螺旋 /
   * sphere 球壳）、n 粒子数、op 基础不透明度、spin 转速、expand 半径脉动幅度、
   * expandFreq/opFreq 脉动/呼吸频率。未命中血统（无 data.bloodline 或不在表内）不画任何光环。
   */
  const BL_ALPHA = (hex, a) => "rgba(" + ((hex >> 16) & 255) + "," + ((hex >> 8) & 255) + "," + (hex & 255) + "," + a + ")";
  const BLOODLINE_AURAS = {
    vampire:            { color: 0xcc2233, n: 26, op: 0.50, spin: 0.45, form: "column", expand: 0.18, expandFreq: 1.2, opFreq: 2.0 }, // 暗红血雾升腾
    werewolf:           { color: 0xcdd3da, n: 28, op: 0.55, spin: 0.30, form: "sphere", expand: 0.14, expandFreq: 1.0, opFreq: 1.6, glow: 0x39ff4f }, // 银灰毛发+幽绿狼眼
    zuwu:               { color: 0xd8a83a, n: 22, op: 0.52, spin: 0.22, form: "ring",   expand: 0.30, expandFreq: 0.55, opFreq: 0.8 }, // 土黄巫纹+大地脉动
    zhanshi_blood:      { color: 0xffe37a, n: 32, op: 0.60, spin: 0.50, form: "ring",   expand: 0.20, expandFreq: 1.6, opFreq: 2.4 }, // 金色圣光十字辉光
    cyber_prosthetic:   { color: 0x42c8ff, n: 30, op: 0.55, spin: 0.85, form: "helix",  expand: 0.16, expandFreq: 2.2, opFreq: 5.0 }, // 蓝白电流
    gauss_cyber:        { color: 0xcfe4ff, n: 34, op: 0.42, spin: 0.35, form: "sphere", expand: 0.22, expandFreq: 0.9, opFreq: 2.0 }, // 银色纳米粒子雾
    angel_bloodline:    { color: 0xeaf4ff, n: 32, op: 0.60, spin: 0.35, form: "ring",   expand: 0.12, expandFreq: 1.4, opFreq: 2.2 }, // 白色光环+圣羽
    demon_bloodline:    { color: 0xff4466, n: 30, op: 0.62, spin: 0.70, form: "helix",  expand: 0.22, expandFreq: 1.8, opFreq: 3.2 }, // 暗红烈焰
    dragon_bloodline:   { color: 0xffcf3a, n: 36, op: 0.60, spin: 0.50, form: "helix",  expand: 0.18, expandFreq: 1.4, opFreq: 2.6 }, // 金色龙鳞闪光
    saiyan_bloodline:   { color: 0xffdf3a, n: 40, op: 0.62, spin: 0.60, form: "column", expand: 0.30, expandFreq: 1.9, opFreq: 3.0 }, // 金色气焰冲天
    sharingan_bloodline:{ color: 0xff2244, n: 24, op: 0.58, spin: 0.90, form: "ring",   expand: 0.10, expandFreq: 1.1, opFreq: 2.0 }, // 红色瞳光+勾玉旋转
    hollow_bloodline:   { color: 0xffffff, n: 22, op: 0.40, spin: 0.28, form: "sphere", expand: 0.24, expandFreq: 0.7, opFreq: 1.2 }, // 苍白/黑灵压
    saint_bloodline:    { color: 0x7fd4ff, n: 28, op: 0.55, spin: 0.40, form: "ring",   expand: 0.15, expandFreq: 1.2, opFreq: 1.8 }, // 星座圣衣冷蓝辉光
    shinigami_bloodline:{ color: 0x5aa2ff, n: 26, op: 0.50, spin: 0.50, form: "column", expand: 0.16, expandFreq: 1.3, opFreq: 2.2 }, // 蓝色灵压
    quincy_bloodline:   { color: 0x67d8ff, n: 24, op: 0.55, spin: 0.60, form: "ring",   expand: 0.12, expandFreq: 1.5, opFreq: 2.8 }, // 青色灵子箭光
    uchiha_bloodline:   { color: 0xff3355, n: 26, op: 0.58, spin: 0.80, form: "helix",  expand: 0.14, expandFreq: 1.6, opFreq: 2.4 }, // 写轮眼红+火遁纹
    senju_bloodline:    { color: 0x66ff88, n: 28, op: 0.55, spin: 0.30, form: "column", expand: 0.20, expandFreq: 0.9, opFreq: 1.6 }, // 绿色木遁生机光
    otsutsuki_bloodline:{ color: 0xefe7ff, n: 30, op: 0.58, spin: 0.35, form: "sphere", expand: 0.14, expandFreq: 1.0, opFreq: 1.8 }, // 苍白轮回眼辉光
    mitsurugi_bloodline:{ color: 0xff7a3f, n: 34, op: 0.60, spin: 0.60, form: "helix",  expand: 0.20, expandFreq: 1.7, opFreq: 2.8 }, // 日之呼吸焰纹
  };

  // 敌人
  let enemies = [];                  // {x,y,spr,name,hp,flash,dead,deadT,scale}
  let enemySprite = null;
  let zoneProps = [];                // 房间道具 {bx,by,spr,y}
  let bgUrl = "img_corridor.png";    // 横版视差背景（默认生化走廊，map.background 可覆盖）

  // 相机
  let cam = { x: 0, y: 0, mode: "FOLLOW", targetX: 0, targetY: 0, shake: 0 };
  let bossState = "idle";            // idle/intro/fight/clear
  let bossIntroT = 0;

  // 房间/走廊判定
  let rooms = [];
  let currentRoom = null;

  /* ---------------- 工具 ---------------- */
  function makeCanvas(w, h) { const c = document.createElement("canvas"); c.width = w; c.height = h; return c; }
  const hash2 = (x, y, s) => { let h = (x * 73856093) ^ (y * 19349663) ^ (s * 0x9e3779b9); h = (h ^ (h >>> 13)) >>> 0; return h; };

  /* ---------------- 图片/立绘 ---------------- */
  const imgCache = new Map();
  function loadImg(url) {
    let e = imgCache.get(url);
    if (e) return e;
    e = { img: new Image(), ready: false };
    e.img.onload = () => { e.ready = true; };
    e.img.src = ROOT + url;
    imgCache.set(url, e);
    return e;
  }
  const portraitCache = new Map();
  function makePortrait(url, w, h) {
    let e = portraitCache.get(url);
    if (e) return e;
    e = { cv: null, ready: false };
    const imgE = loadImg(url);
    const build = () => {
      if (!imgE.img.naturalWidth) return;
      try {
        const iw = imgE.img.naturalWidth, ih = imgE.img.naturalHeight;
        const tmp = makeCanvas(w * 2, h * 2);
        const tg = tmp.getContext("2d");
        tg.imageSmoothingEnabled = true;
        const s = Math.max(w * 2 / iw, h * 2 / ih);
        tg.clearRect(0, 0, tmp.width, tmp.height);          // 透明底（保留立绘 alpha）
        tg.drawImage(imgE.img, (tmp.width - iw * s) / 2, (tmp.height - ih * s) / 2, iw * s, ih * s);
        const td = tg.getImageData(0, 0, tmp.width, tmp.height);
        const out = makeCanvas(w, h);
        const og = out.getContext("2d");
        const oi = og.createImageData(w, h);
        for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
          // 预乘 alpha 平均（透明像素不污染颜色 → 无黑底框）
          let r = 0, g = 0, b = 0, a = 0, cnt = 0;
          for (let dy = 0; dy < 2; dy++) for (let dx = 0; dx < 2; dx++) {
            const p = ((y * 2 + dy) * tmp.width + (x * 2 + dx)) * 4;
            const sa = td.data[p + 3];
            r += td.data[p] * sa; g += td.data[p + 1] * sa; b += td.data[p + 2] * sa; a += sa; cnt++;
          }
          const i = (y * w + x) * 4;
          if (a > 0) { oi.data[i] = r / a; oi.data[i + 1] = g / a; oi.data[i + 2] = b / a; oi.data[i + 3] = a / cnt; }
          else oi.data[i + 3] = 0;
        }
        og.putImageData(oi, 0, 0);
        const img = og.getImageData(0, 0, w, h);
        const d = img.data;
        for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
          const i = (y * w + x) * 4;
          if (d[i + 3] < 24) {
            let edge = false;
            for (let dy = -1; dy <= 1 && !edge; dy++) for (let dx = -1; dx <= 1 && !edge; dx++) {
              if (!dx && !dy) continue;
              const nx = x + dx, ny = y + dy;
              if (nx < 0 || ny < 0 || nx >= w || ny >= h) { edge = true; break; }
              if (d[(ny * w + nx) * 4 + 3] > 96) { edge = true; break; }
            }
            // 极淡暗描边（alpha≈90，仅勾勒轮廓，不再形成黑/白矩形框）
            if (edge) { d[i] = 14; d[i + 1] = 17; d[i + 2] = 22; d[i + 3] = 26; }
          } else {
            // 白描边（rim light）贴透明背景的边缘像素 → 压暗成描边，消除"白色方框"
            const nearWhite = d[i] > 225 && d[i + 1] > 225 && d[i + 2] > 225;
            let adjacentTransparent = false;
            if (nearWhite) {
              for (let dy = -1; dy <= 1 && !adjacentTransparent; dy++) for (let dx = -1; dx <= 1 && !adjacentTransparent; dx++) {
                if (!dx && !dy) continue;
                const nx = x + dx, ny = y + dy;
                if (nx < 0 || ny < 0 || nx >= w || ny >= h) continue;
                if (d[(ny * w + nx) * 4 + 3] < 24) adjacentTransparent = true;
              }
            }
            if (nearWhite && adjacentTransparent) {
              d[i] = 14; d[i + 1] = 17; d[i + 2] = 22; d[i + 3] = 26;
            } else {
              // 明显提亮+增强对比：角色靠亮度在暗背景上分离，不再依赖描边
              const avg = (d[i] + d[i + 1] + d[i + 2]) / 3;
              d[i] = clamp(avg + (d[i] - avg) * 1.22 + 22, 0, 255);
              d[i + 1] = clamp(avg + (d[i + 1] - avg) * 1.22 + 22, 0, 255);
              d[i + 2] = clamp(avg + (d[i + 2] - avg) * 1.22 + 22, 0, 255);
            }
          }
        }
        og.putImageData(img, 0, 0);
        // 锐化（Laplacian unsharp，抵消 2× 平滑下采样的糊感，贴近像素风）
        const sh = og.getImageData(0, 0, w, h);
        const sd = sh.data;
        const amt = 0.4;
        for (let y = 1; y < h - 1; y++) for (let x = 1; x < w - 1; x++) {
          const i = (y * w + x) * 4;
          if (sd[i + 3] < 40) continue;
          for (let cc = 0; cc < 3; cc++) {
            const v = sd[i + cc];
            const l = sd[(y * w + x - 1) * 4 + cc], r = sd[(y * w + x + 1) * 4 + cc];
            const u = sd[((y - 1) * w + x) * 4 + cc], dd = sd[((y + 1) * w + x) * 4 + cc];
            sd[i + cc] = clamp(v + amt * (4 * v - l - r - u - dd), 0, 255);
          }
        }
        og.putImageData(sh, 0, 0);
        e.cv = out; e.ready = true;
      } catch (err) { e.err = String(err); }
    };
    imgE.img.onload = () => { imgE.ready = true; build(); };
    build();
    portraitCache.set(url, e);
    return e;
  }

  /* ---------------- 瓦片 ---------------- */
  const tileCache = new Map();
  function makeTile(kind) {
    const key = kind;
    if (tileCache.has(key)) return tileCache.get(key);
    const c = makeCanvas(ART, ART);
    const g = c.getContext("2d");
    if (window.PIX2D && window.PIX2D.makeTile) {
      try {
        const t = kind === "wall" ? window.PIX2D.makeTile("wall", 0) : window.PIX2D.makeTile("floor", 0);
        if (t) g.drawImage(t, 0, 0);
      } catch (e) {}
    }
    if (kind === "floor") {
      // 横版地板：平坦金属（亮中灰 + 细横缝 + 少量污渍，可走感）
      g.fillStyle = "#5a6673"; g.fillRect(0, 0, 48, 48);
      g.fillStyle = "#6d7a88"; g.fillRect(0, 0, 48, 6);
      g.fillStyle = "#48525e"; g.fillRect(0, 44, 48, 4);
      g.fillStyle = "#0d1116"; g.fillRect(0, 0, 48, 2);
      g.fillStyle = "rgba(10,14,18,0.5)"; g.fillRect(0, 42, 48, 2);
      g.fillStyle = "rgba(255,255,255,0.10)"; g.fillRect(2, 8, 44, 2);
      g.fillStyle = "rgba(10,14,18,0.25)"; g.fillRect(0, 16, 48, 2); g.fillRect(0, 32, 48, 2);
      g.fillStyle = "rgba(138,74,42,0.4)"; g.fillRect(6, 24, 8, 4); g.fillRect(32, 36, 6, 4);
    } else if (kind === "wall") {
      // 横版工业墙：亮顶面 + 中灰墙面 + 暗底（比俯视版更亮，保证横版场景可读）
      g.fillStyle = "#6b7a88"; g.fillRect(0, 0, 48, 12);
      g.fillStyle = "#4a5866"; g.fillRect(0, 12, 48, 36);
      g.fillStyle = "#0d1116"; g.fillRect(0, 10, 48, 2);
      g.fillStyle = "rgba(255,255,255,0.14)"; g.fillRect(2, 2, 44, 4);
      g.fillStyle = "#333f4b"; g.fillRect(0, 14, 48, 34);
      g.fillStyle = "#93a3b4"; g.fillRect(14, 16, 2, 30); g.fillRect(30, 16, 2, 30);
      g.fillStyle = "rgba(210,220,230,0.35)"; g.fillRect(4, 20, 6, 2); g.fillRect(20, 30, 6, 2); g.fillRect(36, 40, 6, 2);
      g.fillStyle = "rgba(10,14,18,0.5)"; g.fillRect(0, 44, 48, 4);
    } else if (kind === "wall2") {
      // 墙变体 2：锈迹与不同铆钉排布
      g.fillStyle = "#6b7a88"; g.fillRect(0, 0, 48, 12);
      g.fillStyle = "#4a5866"; g.fillRect(0, 12, 48, 36);
      g.fillStyle = "#0d1116"; g.fillRect(0, 10, 48, 2);
      g.fillStyle = "rgba(255,255,255,0.14)"; g.fillRect(2, 2, 44, 4);
      g.fillStyle = "#333f4b"; g.fillRect(0, 14, 48, 34);
      g.fillStyle = "#93a3b4"; g.fillRect(28, 16, 2, 30);
      g.fillStyle = "rgba(210,220,230,0.35)"; g.fillRect(10, 24, 6, 2); g.fillRect(38, 18, 6, 2);
      g.fillStyle = "rgba(138,74,42,0.55)"; g.fillRect(4, 32, 10, 6); g.fillRect(34, 36, 8, 4);
      g.fillStyle = "rgba(10,14,18,0.5)"; g.fillRect(0, 44, 48, 4);
    } else if (kind === "platform") {
      // 平台砖：灰砖块（顶亮、侧暗、底影）
      g.fillStyle = "#3d4753"; g.fillRect(0, 0, 48, 48);
      g.fillStyle = "#596675"; g.fillRect(0, 0, 48, 10);
      g.fillStyle = "#232b33"; g.fillRect(0, 36, 48, 12);
      g.fillStyle = "#0d1116"; g.fillRect(0, 0, 48, 2);
      g.fillStyle = "rgba(120,140,160,0.5)"; g.fillRect(2, 12, 44, 2);
      g.fillStyle = "rgba(10,14,18,0.6)"; g.fillRect(0, 14, 48, 2);
    } else if (kind === "climb") {
      // 爬墙砖：竖向管道纹理
      g.fillStyle = "#2c343d"; g.fillRect(0, 0, 48, 48);
      g.fillStyle = "#46525e"; g.fillRect(16, 0, 16, 48);
      g.fillStyle = "#93a3b4"; g.fillRect(16, 0, 4, 48);
      g.fillStyle = "#1a2026"; g.fillRect(32, 0, 4, 48);
      g.fillStyle = "rgba(138,74,42,0.5)"; g.fillRect(4, 6, 8, 4); g.fillRect(4, 28, 6, 4);
    }
    tileCache.set(key, c);
    return c;
  }

  /* ---------------- 地图 ---------------- */
  function tileAt(tx, ty) {
    if (!map || tx < 0 || ty < 0 || ty >= map.h || tx >= map.w) return '#';
    const row = map.tiles[ty] || "";
    return (row[tx] || '#');
  }
  function solidAt(tx, ty) {
    const c = tileAt(tx, ty);
    return c === '#' || c === 'C';
  }

  function buildMap() {
    if (!map) return;
    rooms = (map.rooms || []).map(r => ({ x: r.x, y: r.y, w: r.w, h: r.h, type: r.type }));
    // 房间道具（破背景单调：地板格上确定性放箱子/管道/控制台）
    zoneProps = [];
    if (window.PIX2D && window.PIX2D.makeProp) {
      for (let ty = 0; ty < map.h; ty++) {
        for (let tx = 0; tx < map.w; tx++) {
          if (tileAt(tx, ty) !== '.') continue;
          const h = hash2(tx, ty, 23);
          if (h % 11 !== 0) continue;                       // 稀疏
          const kind = (h % 3 === 0) ? "crate" : (h % 3 === 1) ? "pipe" : "console";
          let spr = null;
          try { spr = window.PIX2D.makeProp(kind, 0, (h % 2) ? "h" : "v"); } catch (e) {}
          if (!spr) continue;
          zoneProps.push({
            bx: tx * ART + (ART - spr.width) / 2,
            by: ty * ART + ART - spr.height,
            spr: spr,
            y: ty * ART + ART,
          });
        }
      }
    }
    // 玩家出生
    const ps = map.player_start || {};
    P.x = (typeof ps.x === "number" ? ps.x : 5) * ART + ART / 2;
    P.y = (typeof ps.y === "number" ? ps.y : 6) * ART + ART;
    P.vx = 0; P.vy = 0; P.onGround = true; P.climbing = false;
    state = "idle"; stateT = 0;
    // 敌人（boss 刷点渲染 fight 敌人）
    enemies = [];
    const bossSpawn = (map.enemy_spawns || []).find(s => s.type === "boss");
    if (bossSpawn && data && data.enemy) {
      const en = data.enemy;
      enemySprite = makePortrait(enemyPortraitFor(en.name), 60, 92);
      enemies.push({
        x: bossSpawn.x * ART + ART / 2, y: bossSpawn.y * ART + ART,
        name: en.name, hp: en.hp || 100, maxHp: en.hp || 100,
        flash: 0, dead: false, deadT: 0, boss: true,
      });
    }
    cam.x = P.x; cam.y = P.y;
    cam.mode = "FOLLOW"; bossState = "idle";
    currentRoom = null;
  }
  function enemyPortraitFor(name) {
    const n = name || "";
    if (n.indexOf("舔食者") >= 0 || n.indexOf("licker") >= 0) return "enemy_licker.png";
    if (n.indexOf("暴君") >= 0 || n.indexOf("tyrant") >= 0) return "enemy_tyrant.png";
    if (n.indexOf("猎杀者") >= 0 || n.indexOf("hunter") >= 0) return "enemy_hunter.png";
    if (n.indexOf("尸潮") >= 0 || n.indexOf("horde") >= 0 || n.indexOf("兽潮") >= 0) return "img_horde.png";
    if (n.indexOf("蛮兽") >= 0 || n.indexOf("brute") >= 0) return "enemy_brute.png";
    if (n.indexOf("皇后") >= 0) return "enemy_yiy_queen.png";
    return "enemy_zombie.png";
  }

  /* ---------------- 镜头 ---------------- */
  function viewport() {
    if (!cv) return { w: 960, h: 540 };
    return { w: cv.width / (dpr * zoom * (TILE / ART)), h: cv.height / (dpr * zoom * (TILE / ART)) };
  }
  function roomForPoint(x, y) {
    for (const r of rooms) {
      // 底部边界放宽 1 格（玩家站在房间底沿也算房内）
      if (x >= r.x * ART - ART / 2 && x < (r.x + r.w) * ART + ART / 2 && y >= r.y * ART - ART / 2 && y < (r.y + r.h) * ART + ART) return r;
    }
    return null;
  }
  function roomLocked(r) {
    if (!r) return false;
    const vp = viewport();
    return (r.w * ART < vp.w * 1.1) || (r.h * ART < vp.h * 1.1);
  }
  function updateCamera(dt) {
    const vp = viewport();
    const mapW = (map ? map.w : 60) * ART, mapH = (map ? map.h : 36) * ART;
    const r = roomForPoint(P.x, P.y);
    currentRoom = r;
    const K = (flying || cam.mode === "SHAFT") ? 6.0 : 8.0;
    const ease = 1 - Math.exp(-K * dt);

    // 模式判定
    let mode = "FOLLOW";
    if (bossState === "intro" || bossState === "fight") {
      mode = "BOSS";
      const br = rooms.find(x => x.type === "boss");
      if (br) { cam.targetX = br.x * ART + br.w * ART / 2; cam.targetY = br.y * ART + br.h * ART / 2; }
    } else if (r && r.type === "shaft") {
      mode = "SHAFT";
    } else if (r && roomLocked(r)) {
      mode = "LOCKED";
      cam.targetX = r.x * ART + r.w * ART / 2;
      cam.targetY = r.y * ART + r.h * ART / 2;
    }
    cam.mode = mode;

    if (mode !== "BOSS") {
      if (mode === "LOCKED") {
        // 目标已定（房间中心）
      } else if (mode === "SHAFT") {
        cam.targetX = r.x * ART + r.w * ART / 2;
        cam.targetY = P.y - vp.h * 0.40;          // 屏幕 y 60%
      } else {
        // FOLLOW：水平 55/45% 前瞻（按朝向），垂直带 62~68%（目标 65%），跳跃窥屏阈值=半屏
        const lead = P.facing > 0 ? 0.55 : 0.45;
        cam.targetX = P.x - vp.w * lead;
        // 垂直：记录本跳峰高；< 半屏不跟随
        if (!flying && jumpApex < vp.h / 2) {
          // 只在跌破 68% 带或高于 62% 带时动
          const anchorY = P.y;
          const topBand = cam.y + vp.h * 0.62, botBand = cam.y + vp.h * 0.68;
          if (anchorY < topBand) cam.targetY = P.y - vp.h * 0.65;
          else if (anchorY > botBand) cam.targetY = P.y - vp.h * 0.65;
          else cam.targetY = cam.y;
        } else {
          cam.targetY = P.y - (flying ? vp.h * 0.42 : vp.h * 0.65);   // flying 58%：P.y - 0.42H
        }
      }
    }

    // 平滑
    cam.x += (cam.targetX - cam.x) * ease;
    cam.y += (cam.targetY - cam.y) * ease;
    // clamp
    cam.x = clamp(cam.x, 0, Math.max(0, mapW - vp.w));
    cam.y = clamp(cam.y, 0, Math.max(0, mapH - vp.h));
    if (mapW <= vp.w) cam.x = (mapW - vp.w) / 2;
    if (mapH <= vp.h) cam.y = (mapH - vp.h) / 2;
    // 震屏
    if (cam.shake > 0) cam.shake = Math.max(0, cam.shake - dt * 18);
  }
  let jumpApex = 0, jumpStartY = 0;

  /* ---------------- 物理 ---------------- */
  function updatePlayer(dt) {
    const walk = 7.2;                       // px/f 基准（60fps）；左右移速提高
    const move = walk * dt * 60;
    const left = keys["a"] || keys["arrowleft"];
    const right = keys["d"] || keys["arrowright"];
    const up = keys["w"] || keys["arrowup"];
    const down = keys["s"] || keys["arrowdown"];
    const jumpKey = keys[" "] || up;

    // 攻击/受击/死亡高优先级状态
    if (state === "attack" || state === "hurt" || state === "die") {
      stateT += dt * 1000;
      if (state === "attack" && stateT >= 150) state = P.onGround ? "idle" : "jump";
      if (state === "hurt" && stateT >= 80) state = P.onGround ? "idle" : "jump";
      if (state === "die" && stateT >= 700) { /* 等待外部 onExit */ }
      return;
    }

    // 蛛丝摆荡（前端表现：锚点钟摆运动，重力沿切向积分）
    if (swing) {
      updateSwing(dt);
      clampToMap();
      state = "swing";
      stateT += dt * 1000;
      return;
    }

    // 浮空
    if (flying) {
      P.vx = 0; P.vy = 0;
      if (left) P.x -= 3.2 * dt * 60;
      if (right) P.x += 3.2 * dt * 60;
      if (up) P.y -= 2.4 * dt * 60;
      if (down) P.y += 2.4 * dt * 60;
      P.facing = right ? 1 : left ? -1 : P.facing;
      P.onGround = false;
      state = "fly";
      clampToMap();
      return;
    }

    // 爬墙
    const wallDir = facingWall();
    P.climbing = false;
    if (wallDir !== 0 && !P.onGround && wallJumpLock <= 0 && up) {
      P.climbing = true;
      P.y -= 2.0 * dt * 60;
      P.facing = wallDir;
      P.vy = 0;
      state = "climb";
      // 墙跳
      if (jumpKey && wallJumpLock <= 0) {
        P.vx = -wallDir * 11.0 * dt * 60 * 60;   // 11.0 px/f 水平冲量（按帧归一）
        P.vy = -15.0 * dt * 60 * 60;
        wallJumpLock = 0.25;
        P.climbing = false;
        P.onGround = false;
        state = "jump";
      }
    } else if (wallDir !== 0 && !P.onGround && down) {
      // 贴墙下滑（可控）
      P.y += 1.8 * dt * 60;
      P.vy = 1.8 * dt * 60 * 60;
      P.facing = wallDir;
      state = "climb";
    } else {
      // 水平移动（空中 0.6 倍）
      const acc = P.onGround ? 1 : 0.6;
      if (left) { P.vx = -move * 60 * acc; P.facing = -1; }
      else if (right) { P.vx = move * 60 * acc; P.facing = 1; }
      else P.vx *= (P.onGround ? 0.5 : 0.9);

      // 跳跃（降低初速，避免上跳过快）
      if (jumpKey && P.onGround) {
        P.vy = -13.5 * dt * 60 * 60;
        P.onGround = false;
        jumpStartY = P.y; jumpApex = 0;
        state = "jump";
      }
      // 重力
      P.vy += 0.96 * dt * 60 * 60;
      if (P.vy > 28 * dt * 60 * 60) P.vy = 28 * dt * 60 * 60;

      // 垂直移动 + 碰撞
      P.y += P.vy * dt;
      if (P.vy < 0) {
        // 上升：头顶碰撞（仅实心 #/C 阻挡，平台砖 P 可下穿）
        if (solidAt(Math.floor(P.x / ART), Math.floor((P.y - 80) / ART))) {
          P.y = (Math.floor((P.y - 80) / ART) + 1) * ART + 80;
          P.vy = 0;
        }
      } else {
        // 下落：脚底支撑格（脚锚正下方一行，'.'/'P'/'#'/'C' 均可站；越界不算支撑）
        const belowRow = Math.floor((P.y + 1) / ART);
        let support = false;
        if (belowRow >= 0 && belowRow < map.h) {
          const c = tileAt(Math.floor(P.x / ART), belowRow);
          support = (c === '.' || c === 'P' || c === '#' || c === 'C');
          // 下键穿过平台 P（短时忽略 P 支撑，实现"向下/下跳"）
          if (c === 'P' && down && dropT <= 0) { support = false; dropT = 0.22; }
        }
        if (dropT > 0) dropT -= dt;
        if (support) {
          P.y = belowRow * ART;               // 脚锚 = 支撑格顶边（不再下沉进格）
          P.vy = 0;
          if (!P.onGround) state = "idle";
          P.onGround = true;
        } else {
          P.onGround = false;
        }
      }
      // 记录跳跃峰高（仅上升段）
      if (!P.onGround && P.vy < 0) jumpApex = Math.max(jumpApex, jumpStartY - P.y);
      if (P.onGround) jumpApex = 0;
    }

    // 水平移动 + 墙碰撞（仅 #/C 阻挡，平台砖 P 不挡水平）
    P.x += P.vx * dt;
    if (P.vx > 0 && solidAt(Math.floor((P.x + 24) / ART), Math.floor(P.y / ART))) {
      P.x = Math.floor((P.x + 24) / ART) * ART - 24; P.vx = 0;
    } else if (P.vx < 0 && solidAt(Math.floor((P.x - 24) / ART), Math.floor(P.y / ART))) {
      P.x = (Math.floor((P.x - 24) / ART) + 1) * ART + 24; P.vx = 0;
    }

    // 掉落出图 → 回出生点
    if (P.y > map.h * ART + 400) { P.x = 5 * ART + ART / 2; P.y = 6 * ART + ART; P.vy = 0; }

    if (wallJumpLock > 0) wallJumpLock -= dt;
    if (P.onGround && state !== "climb") state = (Math.abs(P.vx) > 1) ? "run" : "idle";
    if (!P.onGround && state !== "climb") state = "jump";
    clampToMap();
  }
  function clampToMap() {
    if (!map) return;
    P.x = clamp(P.x, 24, map.w * ART - 24);
    P.y = clamp(P.y, -2000, map.h * ART + 400);
  }
  function facingWall() {
    const dir = P.facing;
    const tx = Math.floor((P.x + dir * 24) / ART);
    const tyTop = Math.floor((P.y - 68) / ART), tyBot = Math.floor((P.y - 8) / ART);
    for (let ty = tyTop; ty <= tyBot; ty++) {
      if (solidAt(tx, ty)) return dir;
    }
    return 0;
  }

  /* ---------------- 战斗动作 ---------------- */
  function doAttack() {
    if (attackCd > 0 || state === "die") return;
    attackCd = 0.7;
    state = "attack"; stateT = 0;
    if (opts && opts.onAction) opts.onAction("attack", 0);
  }
  function doDodge() {
    if (dodgeCd > 0 || state === "die") return;
    dodgeCd = 1.0;
    state = "dodge"; stateT = 0;
    // 侧滚残影
    for (let i = 0; i < 3; i++) afterImages.push({ x: P.x - P.facing * i * 12, y: P.y, t: 0.2 });
    if (opts && opts.onAction) opts.onAction("dodge", 0);
  }
  function castEquippedSkill(slot) {
    // 技能释放法阵（前端视觉；判定在 Rust onAction）。读 data.skills[slot].name/kind 微调颜色。
    const sk = (data && data.skills) ? data.skills[slot] : null;
    fx.push(spawnSkillCircle(sk));
    // 道具使用瞬间："取物→举起→释放"的短演出（纯表现）
    castAnim = { t: 0, cfg: skillFx(sk && sk.kind) };
    if (opts && opts.onAction) opts.onAction("skill", slot);
  }

  /* ---------------- 武器风格与特效（纯前端表现；判定/结算一律在 Rust） ---------------- */
  // 武器名（中文）→ 攻击视觉类别
  const WEAPON_FX = {
    melee:   { key: "melee",   color: "#7fe3ff", sub: "#c8f4ff", tag: "挥砍弧光+刃光拖尾" },
    gun:     { key: "gun",     color: "#ffcf5e", sub: "#fff3c4", tag: "瞄准线→枪口闪光→弹道" },
    bow:     { key: "bow",     color: "#ffe08a", sub: "#fff7d6", tag: "瞄准准星→箭矢光束残影" },
    magic:   { key: "magic",   color: "#c9a4ff", sub: "#f0e4ff", tag: "小法阵+粒子" },
    unarmed: { key: "unarmed", color: "#d8e6ff", sub: "#ffffff", tag: "短促拳击弧光" },
  };
  function weaponStyle(name) {
    const n = name || "";
    if (/军刀|剑|斧|刀|拳|棍|刃|匕首|刀刺/.test(n)) return "melee";
    if (/枪|炮|狙击|弩|电磁|炮塔|发射器|弹道/.test(n)) return "gun";
    if (/弓/.test(n)) return "bow";
    if (/法杖|法器|符|咒|术|印|阵|卷轴|权杖/.test(n)) return "magic";
    return "unarmed";
  }
  // hex → rgba（加透明度），非法返回白色
  function hexA(hex, a) {
    if (!hex) return "rgba(255,255,255," + a + ")";
    const c = hex.charAt(0) === "#" ? hex.slice(1) : hex;
    const num = parseInt(c, 16);
    if (isNaN(num)) return "rgba(255,255,255," + a + ")";
    return "rgba(" + ((num >> 16) & 255) + "," + ((num >> 8) & 255) + "," + (num & 255) + "," + a + ")";
  }
  // 技能 kind → 主导色（微调法阵配色；技能释放线已由 skillFx 接管，此处保留作兜底）
  function skillColor(kind) {
    const k = kind || "";
    if (/火|炎|燃|爆/.test(k)) return "#ff7a4a";
    if (/冰|霜|寒|雪/.test(k)) return "#7fdcff";
    if (/雷|电|霹|闪/.test(k)) return "#ffe95e";
    if (/毒|暗|影|咒|死|血/.test(k)) return "#a78bff";
    if (/圣|光明|治愈|守护|光|盾/.test(k)) return "#fff0a0";
    if (/风|气|卷/.test(k)) return "#b9f2ff";
    return "#c9a4ff";
  }
  // 14 大类 → 特效配置（颜色 + 粒子形状 + 释放形态）。粒子本体一律 fillRect，光晕用 radialGradient。
  const SKILL_FX = {
    sword:  { color: "#e8f4ff", sub: "#7fc4ff", glow: "#cfe9ff", shape: "slash",  form: "swing",  life: 0.45, tag: "剑光：冷白月牙弧光+剑气横斩" },
    blade:  { color: "#d8e8ff", sub: "#9aa9ff", glow: "#c4d8ff", shape: "slash",  form: "swing",  life: 0.45, tag: "刀光：银蓝弧光+刀气横斩" },
    xian:   { color: "#d9efff", sub: "#6fb6ff", glow: "#c8ecff", shape: "slash",  form: "sweep",  life: 0.50, tag: "剑仙：御剑剑光+剑阵横斩" },
    tixiu:  { color: "#ffd9a8", sub: "#c98a3f", glow: "#ffdfae", shape: "ring",   form: "shock",  life: 0.40, tag: "体修：肉色/土黄拳劲冲击波" },
    gun:    { color: "#ffcf5e", sub: "#ff9a3f", glow: "#ffe9b0", shape: "beam",   form: "burst",  life: 0.40, tag: "枪械：暖黄弹道+枪口闪光" },
    laser:  { color: "#ff6a5e", sub: "#ffcf5e", glow: "#ff9a7f", shape: "beam",   form: "burst",  life: 0.40, tag: "激光：红热激光束+能量脉冲" },
    tech:   { color: "#7fe3ff", sub: "#2ec0ff", glow: "#bfefff", shape: "pulse",  form: "surge",  life: 0.50, tag: "科技：青色能量脉冲+电子轨迹" },
    light:  { color: "#fff3c4", sub: "#ffe08a", glow: "#fffbe8", shape: "column", form: "ascend", life: 0.60, tag: "光明：白金色圣光柱+光点升腾" },
    dark:   { color: "#b088ff", sub: "#5c2e8a", glow: "#8a5cff", shape: "mist",   form: "spread", life: 0.60, tag: "黑暗：紫黑暗影扩散+紫粒子" },
    element:{ color: "#7fdcff", sub: "#3fbfff", glow: "#bfefff", shape: "burst",  form: "burst",  life: 0.50, tag: "元素：冰/火/雷按hash或关键字" },
    fafa:   { color: "#7fffd0", sub: "#3fcf9f", glow: "#bfffe8", shape: "circle", form: "spin",   life: 0.50, tag: "法修：青绿灵力法阵(旋转符文圈)" },
    blood:  { color: "#ff5e5e", sub: "#8a1f2a", glow: "#ff8a8a", shape: "helix",  form: "throb",  life: 0.60, tag: "血统：血红血丝脉冲" },
    gene:   { color: "#8aff9a", sub: "#2ecf5e", glow: "#bfffd0", shape: "helix",  form: "spin",   life: 0.60, tag: "基因：绿光基因螺旋脉冲" },
    normal: { color: "#c9a4ff", sub: "#f0e4ff", glow: "#e0c8ff", shape: "circle", form: "spin",   life: 0.50, tag: "通用：紫白色小法阵" },
  };
  // 简单字符串哈希（元素魔法 冰/火/雷 三选一用）
  function hashStr(s) {
    let h = 5381;
    for (let i = 0; i < s.length; i++) h = ((h * 33) ^ s.charCodeAt(i)) >>> 0;
    return h;
  }
  // 技能 kind → 特效配置（14 大类差异化；优先精确大类名，再按关键字兜底）
  function skillFx(kind) {
    const k = (kind || "").trim();
    const exact = {
      "剑": SKILL_FX.sword, "刀": SKILL_FX.blade, "剑仙": SKILL_FX.xian,
      "体修": SKILL_FX.tixiu, "枪械": SKILL_FX.gun, "激光": SKILL_FX.laser,
      "科技": SKILL_FX.tech, "光明魔法": SKILL_FX.light, "黑暗魔法": SKILL_FX.dark,
      "法修": SKILL_FX.fafa, "血统": SKILL_FX.blood, "基因": SKILL_FX.gene,
      "通用": SKILL_FX.normal,
    };
    if (exact[k]) return exact[k];
    // 元素魔法：按名字关键字或 hash 三选一（冰/火/雷）
    if (/元素|冰|火|雷|电|风|水/.test(k) || k === "element") {
      const pick = ["ice", "fire", "thunder"][hashStr(k || "element") % 3];
      const pal = pick === "ice" ? { c: "#7fdcff", s: "#3fbfff", g: "#bfefff" }
        : pick === "fire" ? { c: "#ff7a4a", s: "#ffcf5e", g: "#ffd8b0" }
        : { c: "#ffe95e", s: "#ffd84a", g: "#fff2b0" };
      return Object.assign({}, SKILL_FX.element, { elem: pick, color: pal.c, sub: pal.s, glow: pal.g });
    }
    // 关键字兜底
    if (/剑仙|仙|飞剑/.test(k)) return SKILL_FX.xian;
    if (/剑/.test(k)) return SKILL_FX.sword;
    if (/刀|刃|匕首/.test(k)) return SKILL_FX.blade;
    if (/体修|拳|体术/.test(k)) return SKILL_FX.tixiu;
    if (/枪械|枪|炮|狙击|弹道/.test(k)) return SKILL_FX.gun;
    if (/激光/.test(k)) return SKILL_FX.laser;
    if (/科技|械|智能|机甲/.test(k)) return SKILL_FX.tech;
    if (/光明|圣|治愈|守护|光盾/.test(k)) return SKILL_FX.light;
    if (/黑暗|暗影|冥|咒/.test(k)) return SKILL_FX.dark;
    if (/法修|法阵|术|符|咒/.test(k)) return SKILL_FX.fafa;
    if (/血统|血/.test(k)) return SKILL_FX.blood;
    if (/基因|细胞|生物|进化/.test(k)) return SKILL_FX.gene;
    return SKILL_FX.normal;
  }
  // 生成技能释放特效对象（携带 14 大类配置）
  function spawnSkillCircle(sk) {
    const cfg = skillFx(sk && sk.kind);
    const parts = [];
    for (let i = 0; i < 20; i++) {
      parts.push({ off: (i / 20) * Math.PI * 2 + Math.random() * 0.5, dist: 26 + Math.random() * 22, spd: 0.5 + Math.random() * 1.0, siz: 1.6 + Math.random() * 2.4 });
    }
    return { kind: "skillcircle", x: P.x, y: P.y - 40, facing: P.facing, t: 0, life: cfg.life || 0.5, color: cfg.color, sub: cfg.sub, glow: cfg.glow, shape: cfg.shape, form: cfg.form, elem: cfg.elem, parts: parts };
  }
  // 按 14 大类释放形态差异化渲染。粒子本体一律 fillRect，光晕用 radialGradient，禁止 arc/ellipse 画粒子。
  function drawSkillCircle(g, e, now) {
    const p = Math.min(1, e.t / e.life);
    const alpha = (1 - p) * (p < 0.15 ? p / 0.15 : 1);      // 渐入后渐出
    const r = 30 + p * 28;
    const col = e.color, sub = e.sub, glow = e.glow;
    g.save();
    g.translate(e.x, e.y);
    // 光晕：radialGradient 圆形渐变（非粒子本体）
    const hal = r * 1.9 * (1 + p * 0.3);
    const gr = g.createRadialGradient(0, 0, 0, 0, 0, hal);
    gr.addColorStop(0, hexA(glow, 0.30 * alpha));
    gr.addColorStop(0.55, hexA(glow, 0.09 * alpha));
    gr.addColorStop(1, hexA(glow, 0));
    g.fillStyle = gr;
    g.fillRect(-hal, -hal, hal * 2, hal * 2);
    // 按形态分发
    if (e.shape === "slash") drawSlashSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "ring") drawRingSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "beam") drawBeamSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "column") drawColumnSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "mist") drawMistSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "burst") drawBurstSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "pulse") drawPulseSkill(g, e, p, r, col, sub, now);
    else if (e.shape === "helix") drawHelixSkill(g, e, p, r, col, sub, now);
    else drawCircleSkill(g, e, p, r, col, sub, now);
    g.restore();
  }
  // 剑/刀/剑仙：月牙弧光 + 剑气横斩（fillRect 拼形，旋转）
  function drawSlashSkill(g, e, p, r, col, sub, now) {
    g.save();
    g.rotate(e.t * 3.2 * e.facing);
    // 月牙弧光环（fillRect 小段组成圆环，随旋转）
    for (let i = 0; i < 24; i++) {
      const ang = (i / 24) * Math.PI * 2;
      const rr = r * (1 - p * 0.2);
      g.fillStyle = hexA((i % 2) ? col : sub, (1 - p) * 0.7);
      g.fillRect(Math.cos(ang) * rr - 1.4, Math.sin(ang) * rr - 1.4, 2.8, 2.8);
    }
    g.restore();
    // 剑气横斩：横扫刃面（面向方向，fillRect 堆叠 + 闪光边）
    const sweep = p * 2.4;
    const bx = Math.sin(sweep) * e.facing * r * 1.5;
    g.fillStyle = hexA(col, (1 - p) * 0.85);
    g.fillRect(bx - e.facing * r * 0.9, -3, e.facing * r * 1.8, 6);
    g.fillStyle = hexA("#ffffff", (1 - p) * 0.6);
    g.fillRect(bx - e.facing * r * 0.9, -1, e.facing * r * 1.8, 2);
  }
  // 体修：肉色/土黄冲击环扩散 + 中心拳劲闪光
  function drawRingSkill(g, e, p, r, col, sub, now) {
    const rr = r + p * 60;
    for (let i = 0; i < 26; i++) {
      const ang = (i / 26) * Math.PI * 2 + e.t * 0.4;
      g.fillStyle = hexA((i % 3) ? col : sub, (1 - p) * 0.8);
      g.fillRect(Math.cos(ang) * rr - 2, Math.sin(ang) * rr - 2, 4, 4);
    }
    const cs = r * (0.5 + p);
    g.fillStyle = hexA("#fff2d8", (1 - p) * 0.9);
    g.fillRect(-cs / 2, -cs / 2, cs, cs);
  }
  // 枪械/激光：朝面前方激光束/弹道 + 枪口闪光 + 能量脉冲（fillRect 叠层）
  function drawBeamSkill(g, e, p, r, col, sub, now) {
    g.save();
    g.scale(e.facing, 1);
    const len = r * 2.2 * (0.4 + p);
    for (let k = 0; k < 3; k++) {
      const w = 6 + k * 7;
      const l2 = len * (0.7 + k * 0.15);
      g.fillStyle = hexA(k === 0 ? col : sub, (1 - p) * (0.8 - k * 0.2));
      g.fillRect(0, -w / 2, l2, w);
    }
    const fs = r * 0.6 * (1 + 0.5 * Math.sin(now / 40));
    g.fillStyle = hexA("#ffffff", (1 - p) * 0.9);
    g.fillRect(-fs / 2, -fs / 2, fs, fs);
    for (let i = 0; i < 6; i++) {
      const q = (p + i / 6) % 1;
      g.fillStyle = hexA(col, (1 - p) * 0.7);
      g.fillRect(q * len - 2, -3, 4, 6);
    }
    g.restore();
  }
  // 光明：白金色圣光柱 + 光点升腾
  function drawColumnSkill(g, e, p, r, col, sub, now) {
    const hgt = r * 2.4 * (0.5 + p);
    g.fillStyle = hexA(col, (1 - p) * 0.85);
    g.fillRect(-6, -hgt, 12, hgt);
    g.fillStyle = hexA("#ffffff", (1 - p) * 0.7);
    g.fillRect(-3, -hgt, 6, hgt);
    g.fillStyle = hexA(sub, (1 - p) * 0.4);
    g.fillRect(-12, -hgt, 24, hgt * 0.5);
    for (let i = 0; i < 10; i++) {
      const q = ((p * 1.6 + i / 10) % 1);
      const x = Math.sin(now / 90 + i * 1.7) * r * 0.7;
      const y = -q * hgt;
      g.fillStyle = hexA((i % 2) ? col : "#ffffff", (1 - p) * 0.8);
      g.fillRect(x - 1.5, y - 1.5, 3, 3);
    }
  }
  // 黑暗：扩散黑雾 + 紫粒子（radialGradient 暗晕 + fillRect 斑点）
  function drawMistSkill(g, e, p, r, col, sub, now) {
    const gr = g.createRadialGradient(0, 0, 0, 0, 0, r * (1 + p * 1.6));
    gr.addColorStop(0, hexA(sub, 0.5 * (1 - p)));
    gr.addColorStop(1, hexA(sub, 0));
    g.fillStyle = gr;
    g.fillRect(-r * 2, -r * 2, r * 4, r * 4);
    for (let i = 0; i < 14; i++) {
      const q = (p + i / 14) % 1;
      const ang = i * 2.4 + e.t * 0.8;
      const d = q * r * 1.6;
      const s = 6 + Math.sin(now / 60 + i) * 4;
      g.fillStyle = hexA((i % 3) ? col : sub, (1 - p) * 0.5);
      g.fillRect(Math.cos(ang) * d - s / 2, Math.sin(ang) * d - s / 2, s, s);
    }
  }
  // 元素：冰/火/雷爆发（形状随元素：冰刺/焰块/雷锯齿）
  function drawBurstSkill(g, e, p, r, col, sub, now) {
    const elem = e.elem || "ice";
    for (let i = 0; i < e.parts.length; i++) {
      const pt = e.parts[i];
      const d = pt.dist + p * 50;
      const a = pt.off + e.t * pt.spd * 0.6;
      const x = Math.cos(a) * d, y = Math.sin(a) * d;
      if (elem === "ice") {
        g.save(); g.translate(x, y); g.rotate(a + Math.PI / 2);
        g.fillStyle = hexA(col, (1 - p) * 0.85);
        g.fillRect(-1.5, -5, 3, 10); g.restore();
      } else if (elem === "fire") {
        const s = pt.siz * (1 + Math.sin(now / 40 + i) * 0.3);
        g.fillStyle = hexA((i % 2) ? col : sub, (1 - p) * 0.8);
        g.fillRect(x - s / 2, y - s / 2, s, s);
      } else {
        g.fillStyle = hexA((i % 2) ? col : sub, (1 - p) * 0.8);
        g.fillRect(x - 2, y - 4, 4, 8);
      }
    }
  }
  // 科技：青色能量脉冲环 + 电子轨迹折线（fillRect 组成）
  function drawPulseSkill(g, e, p, r, col, sub, now) {
    const pul = 0.5 + 0.5 * Math.sin(now / 30);
    const rr = r * (0.8 + p * 0.6 + pul * 0.2);
    for (let i = 0; i < 30; i++) {
      const ang = (i / 30) * Math.PI * 2 + e.t * 2.2;
      g.fillStyle = hexA((i % 3) ? col : sub, (1 - p) * (0.5 + pul * 0.4));
      g.fillRect(Math.cos(ang) * rr - 1.5, Math.sin(ang) * rr - 1.5, 3, 3);
    }
    let px = -r * 0.8, py = -r * 0.3;
    for (let i = 0; i < 8; i++) {
      const nx = px + r * 0.3, ny = py + (i % 2 ? 1 : -1) * r * 0.25;
      g.fillStyle = hexA(sub, (1 - p) * 0.8);
      g.fillRect(nx - 2, ny - 2, 4, 4);
      px = nx; py = ny;
    }
  }
  // 血统/基因：血红/绿光基因螺旋或血丝脉冲（fillRect 光点 + 生物光晕）
  function drawHelixSkill(g, e, p, r, col, sub, now) {
    const turns = 3;
    for (let i = 0; i < 24; i++) {
      const q = i / 24;
      const ang = q * turns * Math.PI * 2 + e.t * 3;
      const x = Math.cos(ang) * r * 0.7;
      const y = (q - 0.5) * r * 1.6;
      g.fillStyle = hexA((i % 2) ? col : sub, (1 - p) * 0.9);
      g.fillRect(x - 1.8, y - 1.8, 3.6, 3.6);
      if (e.form === "throb" && i > 0) {
        g.fillStyle = hexA(sub, (1 - p) * 0.4);
        g.fillRect(x - 1, y - 1, 2, 2);
      }
    }
    const pul = 0.5 + 0.5 * Math.sin(now / 20);
    g.fillStyle = hexA(sub, (1 - p) * pul * 0.25);
    g.fillRect(-r, -r, r * 2, r * 2);
  }
  // 法修/通用：旋转符文圈（外环刻度 + 内环 + 中心星形符文，全 fillRect）
  function drawCircleSkill(g, e, p, r, col, sub, now) {
    g.save();
    g.rotate(e.t * 2.6 * e.facing + now / 420);
    for (let i = 0; i < 32; i++) {
      const ang = (i / 32) * Math.PI * 2;
      g.fillStyle = hexA(col, (1 - p) * 0.85);
      g.fillRect(Math.cos(ang) * r - 1.6, Math.sin(ang) * r - 1.6, 3.2, 3.2);
    }
    g.rotate(Math.PI / 6);
    for (let i = 0; i < 24; i++) {
      const ang = (i / 24) * Math.PI * 2;
      g.fillStyle = hexA(sub, (1 - p) * 0.5);
      g.fillRect(Math.cos(ang) * r * 0.72 - 1.2, Math.sin(ang) * r * 0.72 - 1.2, 2.4, 2.4);
    }
    g.fillStyle = hexA(col, (1 - p));
    for (let i = 0; i < 8; i++) {
      const a = (i / 8) * Math.PI * 2;
      g.fillRect(Math.cos(a) * r - 1.6, Math.sin(a) * r - 1.6, 3.2, 3.2);
    }
    // 中心星形符文（fillRect 交叉十字 + 对角）
    g.fillStyle = hexA("#ffffff", (1 - p) * 0.9);
    g.fillRect(-r * 0.5, -1, r, 2);
    g.fillRect(-1, -r * 0.5, 2, r);
    g.fillRect(-r * 0.35, -r * 0.35, r * 0.7, 2);
    g.fillRect(-r * 0.35, r * 0.33, r * 0.7, 2);
    for (let i = 0; i < e.parts.length; i++) {
      const pt = e.parts[i];
      const d = pt.dist + p * 32 + Math.sin(e.t * 5 + i) * 4;
      const a = pt.off + e.t * pt.spd * 0.5 * e.facing;
      g.fillStyle = hexA((i % 3 === 0) ? "#ffffff" : col, (1 - p) * 0.8);
      g.fillRect(Math.cos(a) * d - pt.siz / 2, Math.sin(a) * d - pt.siz / 2, pt.siz, pt.siz);
    }
    g.restore();
  }
  // 消耗全部瞬发特效
  function drawFx(g, now) {
    for (let i = fx.length - 1; i >= 0; i--) {
      const e = fx[i];
      e.t += 1 / 60;
      if (e.t >= e.life) { fx.splice(i, 1); continue; }
      if (e.kind === "skillcircle") drawSkillCircle(g, e, now);
    }
  }
  // 普通攻击视觉：按武器类别分帧（stateT ∈ ms，attack 持续 150ms）
  function drawAttackFX(g, now) {
    const style = weaponStyle(data && data.weapon);
    const f = P.facing;
    const cx = P.x, cy = P.y - 36;
    const t = stateT;
    if (style === "melee") {
      // 挥砍弧光 + 刃光拖尾（近战最抢镜）
      const prog = clamp(t / 150, 0, 1);
      if (t > 20 && t < 130) {
        g.strokeStyle = "rgba(127,227,255,0.92)";
        g.lineWidth = 2.2;
        g.beginPath(); g.arc(cx, cy, 32, -0.9 * f, 0.5 * f); g.stroke();
        g.strokeStyle = "rgba(200,244,255,0.55)";
        g.lineWidth = 4;
        g.stroke();
        const k = Math.sin(prog * Math.PI);
        if (k > 0.05) {
          const ext = f * 44;
          // 刃光拖尾曲线
          g.strokeStyle = "rgba(180,240,255," + (0.5 * k) + ")";
          g.lineWidth = 1.6;
          g.beginPath();
          g.moveTo(cx, cy);
          g.quadraticCurveTo(cx + ext * 0.5, cy - 12, cx + ext * k, cy - 4 - 16 * k);
          g.stroke();
          // 拖尾残影粒子
          g.fillStyle = "rgba(140,230,255," + (0.4 * k) + ")";
          for (let i = 0; i < 3; i++) {
            const q = (i + 1) / 4;
            g.beginPath();
            g.arc(cx + ext * k * q - (f * q) * 6, cy - 16 * k * q, 3, 0, Math.PI * 2);
            g.fill();
          }
        }
      }
    } else if (style === "gun" || style === "bow") {
      // 阶段A(0~70ms)：瞄准线/准星；阶段B(≥70ms)：枪口闪光 + 弹道/光束残影
      if (t < 70) {
        const wob = Math.sin(now / 30) * 3;
        const aimX = cx + f * 80, aimY = cy - 4 + wob;
        const lineAlpha = 0.7 - (t / 70) * 0.3;
        g.strokeStyle = "rgba(255,207,94," + lineAlpha + ")";
        g.lineWidth = 1;
        g.setLineDash([8, 8]);
        g.beginPath(); g.moveTo(cx, cy); g.lineTo(aimX, aimY); g.stroke();
        g.setLineDash([]);
        g.strokeStyle = "rgba(255,240,196,0.9)";
        g.lineWidth = 1.2;
        g.beginPath(); g.arc(aimX, aimY, 6, 0, Math.PI * 2); g.stroke();
        g.beginPath();
        g.moveTo(aimX - 12, aimY); g.lineTo(aimX - 4, aimY);
        g.moveTo(aimX + 4, aimY); g.lineTo(aimX + 12, aimY);
        g.moveTo(aimX, aimY - 12); g.lineTo(aimX, aimY - 4);
        g.moveTo(aimX, aimY + 4); g.lineTo(aimX, aimY + 12);
        g.stroke();
      } else {
        const fireT = clamp((t - 70) / 25, 0, 1);      // 闪光 0..1
        if (fireT < 1) {
          const fa = 1 - fireT;
          g.fillStyle = "rgba(255,235,160," + fa * 0.95 + ")";
          g.beginPath(); g.arc(cx + f * 20, cy, 8 + fireT * 6, 0, Math.PI * 2); g.fill();
          g.strokeStyle = "rgba(255,207,94," + fa * 0.7 + ")";
          g.lineWidth = 2;
          g.beginPath(); g.arc(cx + f * 20, cy, 16 + fireT * 8, 0, Math.PI * 2); g.stroke();
        }
        const trail = clamp((t - 70) / 50, 0, 1);       // 弹道推进 0..1
        if (trail > 0) {
          const len = 68 + trail * 52;
          const gradX = cx + f * 20;
          const grad = g.createLinearGradient(gradX, cy, gradX + f * len, cy);
          grad.addColorStop(0, "rgba(255,230,150,0.85)");
          grad.addColorStop(1, "rgba(255,207,94,0)");
          g.strokeStyle = grad;
          g.lineWidth = 2.4;
          g.beginPath(); g.moveTo(gradX, cy); g.lineTo(gradX + f * len, cy); g.stroke();
          g.strokeStyle = "rgba(255,250,220," + (0.9 - trail * 0.5) + ")";
          g.lineWidth = 1;
          g.beginPath(); g.moveTo(gradX, cy); g.lineTo(gradX + f * len * 0.8, cy); g.stroke();
        }
      }
    } else if (style === "magic") {
      // 近程小法阵 + 上浮粒子
      const prog = clamp(t / 150, 0, 1);
      if (t > 30 && t < 140) {
        const pulse = Math.sin(prog * Math.PI);
        const r = 24 + pulse * 8;
        g.strokeStyle = "rgba(201,164,255," + (0.5 + pulse * 0.4) + ")";
        g.lineWidth = 1.6;
        g.beginPath(); g.arc(cx, cy, r, 0, Math.PI * 2); g.stroke();
        g.strokeStyle = "rgba(240,228,255," + (0.35 + pulse * 0.35) + ")";
        g.beginPath(); g.arc(cx, cy, r * 0.6, 0, Math.PI * 2); g.stroke();
        g.fillStyle = "rgba(240,228,255," + (0.5 + pulse * 0.4) + ")";
        for (let i = 0; i < 5; i++) {
          const a = now / 300 + (i / 5) * Math.PI * 2;
          g.beginPath(); g.arc(cx + Math.cos(a) * r * 0.8, cy + Math.sin(a) * r * 0.8, 2.8, 0, Math.PI * 2); g.fill();
        }
        g.fillStyle = "rgba(201,164,255," + (0.4 + pulse * 0.4) + ")";
        for (let i = 0; i < 4; i++) {
          const a = now / 120 + i * 1.7;
          const py = cy - 20 + ((now / 90 + i * 3) % 20);
          g.beginPath(); g.arc(cx + Math.sin(a) * 10, py, 2.4, 0, Math.PI * 2); g.fill();
        }
      }
    } else {
      // unarmed：短促拳击弧光
      if (t > 45 && t < 95) {
        const k = Math.sin(((t - 45) / 50) * Math.PI);
        g.strokeStyle = "rgba(216,230,255," + (0.4 + k * 0.5) + ")";
        g.lineWidth = 2;
        g.beginPath(); g.arc(cx, cy, 24, -0.6 * f, 0.4 * f); g.stroke();
        g.strokeStyle = "rgba(255,255,255," + (k * 0.5) + ")";
        g.lineWidth = 3.4;
        g.stroke();
      }
    }
  }

  /* ---------------- 蜘蛛侠蛛丝摆荡（前端表现） ---------------- */
  // 目标点：优先鼠标世界坐标，无鼠标时朝面朝方向高处
  function aimPoint() {
    if (!mouseScreen) return { x: P.x + P.facing * 600, y: P.y - 520, fromMouse: false };
    const wx = mouseScreen.x / curTrans.s - curTrans.tx;
    const wy = mouseScreen.y / curTrans.s - curTrans.ty;
    return { x: wx, y: wy, fromMouse: true };
  }
  // 向目标方向射线：命中最近实心格（'#'/'C'/'P'），锚点=命中面表面点（朝玩家侧）
  function castWeb(x0, y0, x1, y1) {
    const dx = x1 - x0, dy = y1 - y0;
    const d = Math.hypot(dx, dy);
    if (d < 12 || d > SWING_SHOOT_MAX) return null;
    const steps = Math.min(Math.max(8, Math.ceil(d / 3)), 400);
    const sx = dx / steps, sy = dy / steps;
    let px = x0, py = y0;
    for (let i = 1; i <= steps; i++) {
      px += sx; py += sy;
      const tx = Math.floor(px / ART), ty = Math.floor(py / ART);
      const c = tileAt(tx, ty);
      if (c === '#' || c === 'C' || c === 'P') {
        return { x: px - sx * 2, y: py - sy * 2 };   // 命中前表面点
      }
    }
    return null;
  }
  // 附着到 (ax, ay) 锚点并进入摆荡
  function attachSwing(ax, ay) {
    const dx = P.x - ax, dy = P.y - ay;
    let R = Math.hypot(dx, dy);
    if (R < 8) return;
    R = clamp(R, SWING_MIN, SWING_MAX);
    const ux = -dx / Math.hypot(dx, dy), uy = -dy / Math.hypot(dx, dy);
    P.x = ax + ux * R; P.y = ay + uy * R;
    const theta = Math.atan2(P.x - ax, P.y - ay);       // 从正下方角
    const tangX = Math.cos(theta), tangY = -Math.sin(theta);  // 切向单位
    const vT0 = P.vx * tangX + P.vy * tangY;            // 当前速度切向分量作初始摆速
    swing = { ax: ax, ay: ay, len: R, theta: theta, vT: vT0 };
    P.onGround = false;
    state = "swing"; stateT = 0;
  }
  // 触发/切换锚点：朝当前准星发射（已摆荡则原地切换到新锚点）
  function swingFire() {
    if (!swingEnabled || flying || state === "die") return;
    if (swingFireCd > 0) return;                 // 防键盘重复触发
    swingFireCd = 0.18;
    const am = aimPoint();
    if (!am) return;
    const hit = castWeb(P.x, P.y - 36, am.x, am.y);
    if (hit) attachSwing(hit.x, hit.y);
  }
  // 松开：沿当前切向速度抛出（惯性飞行），落地恢复常规移动
  function swingRelease() {
    if (!swing) return;
    const a = swing;
    const theta = Math.atan2(P.x - a.ax, P.y - a.ay);
    P.vx = a.vT * Math.cos(theta);
    P.vy = -a.vT * Math.sin(theta);
    swing = null;
    P.onGround = false;
    state = "jump"; stateT = 0;
  }
  // 摆荡物理：锚点圆心钟摆（60fps dt 无关）
  function updateSwing(dt) {
    const a = swing;
    const dx = P.x - a.ax, dy = P.y - a.ay;
    let R = Math.hypot(dx, dy);
    if (R < 4) R = 4;
    let theta = Math.atan2(dx, dy);
    let w = a.vT / R;                                  // 切向速度 → 角速度
    w += (-SWING_G * Math.sin(theta)) / R * dt;        // 重力沿切向恢复项
    w *= 0.9995;                                       // 轻阻尼（防无限摆）
    theta += w * dt;
    R = a.len;
    P.x = a.ax + Math.sin(theta) * R;
    P.y = a.ay + Math.cos(theta) * R;
    a.vT = w * R;
    // 摆荡低点防穿地：脚锚进入实心格 → 抬到格顶 + 下摆减震（纯表现层）
    const fty = Math.floor(P.y / ART), ftx = Math.floor(P.x / ART);
    const ft = tileAt(ftx, fty);
    if (ft === '#' || ft === 'C') {
      P.y = fty * ART - 2;
      if (Math.sin(theta) > 0) a.vT *= 0.35;
      theta = Math.atan2(P.x - a.ax, P.y - a.ay);
      R = Math.hypot(P.x - a.ax, P.y - a.ay);
    }
    const tx = a.vT * Math.cos(theta);                 // 切向水平分量 → 朝向
    if (Math.abs(tx) > 1) P.facing = tx > 0 ? 1 : -1;
    // 记录真实瞬时速度（供切换锚点/其它系统取动量），仍以 P.vy 下落感为负
    P.vx = a.vT * Math.cos(theta);
    P.vy = -a.vT * Math.sin(theta);
    P.onGround = false;
  }

  /* ---------------- onZoneUpdate（胜利/死亡/受击表现） ---------------- */
  function onZoneUpdate(update) {
    if (!update) return;
    if (update.win) {
      enemies.forEach(e => { if (!e.dead) { e.dead = true; e.deadT = 0; } });
      state = "idle";
      bossState = "clear";
      if (opts && opts.onWin) opts.onWin();
    } else if (update.hurt) {
      // 玩家受击表现（服务器回合驱动）
      hurtFlash = 1; redFlash = 1;
      swing = null;                                   // 受击断开蛛丝
      state = "hurt"; stateT = 0;
    }
  }

  /* ---------------- 输入 ---------------- */
  function onKeyDown(e) {
    if (!zoneActive) return;
    const k = e.key.toLowerCase();
    if (["arrowleft", "arrowright", "arrowup", "arrowdown", "a", "d", "w", "s", " "].indexOf(k) >= 0) {
      e.preventDefault();
      keys[k] = true;
    }
    if (k === "j" || k === "enter") {
      if (swingEnabled && swing) {
        // 摆荡中 J = 切锚（不再误发攻击回合）
        swingHold = true;
        swingFire();
      } else {
        doAttack();                                    // 常规 J = 攻击回合（契约不变）
      }
    }
    if (k === "shift") doDodge();
    if (k === "q") castEquippedSkill(0);
    if (k === "e" && !keys["w"]) castEquippedSkill(1);
    if (k === "r") castEquippedSkill(2);
    if (k === "escape") { if (opts && opts.onExit) opts.onExit(); }
  }
  function onKeyUp(e) {
    const k = e.key.toLowerCase();
    keys[k] = false;
    if (k === "j") { swingHold = false; swingRelease(); }
  }
  function keydown(e) { onKeyDown(e); }
  function keyup(e) { onKeyUp(e); }

  // 鼠标瞄准 + 左键发射/松开（血统下可替代 J）
  function onMouseMove(e) {
    if (!swingEnabled || !cv) return;
    const r = cv.getBoundingClientRect();
    mouseScreen = { x: (e.clientX - r.left) * dpr, y: (e.clientY - r.top) * dpr };
  }
  function onMouseLeave(e) {
    mouseScreen = null;
  }
  function onMouseDown(e) {
    if (e.button !== 0) return;
    e.preventDefault();
    if (swingEnabled) {
      swingHold = true;
      swingFire();      // 未摆荡=起摆；已摆荡=切锚（0.18s 冷却防抖动）
      return;
    }
    doAttack();         // 鼠标左键 = 普攻（蛛丝摆荡模式除外）
  }
  function onMouseUp(e) {
    if (e.button !== 0) return;
    swingHold = false;
    swingRelease();
  }

  /* ---------------- 渲染 ---------------- */
  const chunkCache = new Map();
  // 横版视差背景：远景图按 0.35×/0.6× 相机移动，水平平铺，暗化铺满
  function drawParallaxBg(g, vw, vh) {
    const e = imgCache.get(bgUrl || "");
    if (!e || !e.ready || !e.img.naturalWidth) return;
    const iw = e.img.naturalWidth, ih = e.img.naturalHeight;
    const sc = dpr * zoom * 1;
    g.save();
    g.setTransform(sc, 0, 0, sc, 0, 0);
    g.translate(Math.round(vw / 2 - cam.x * 0.35), Math.round(vh / 2 - cam.y * 0.6));
    const hh = vh * 1.8;
    const bs = hh / ih;
    const bw = iw * bs;
    const startX = Math.floor((cam.x * 0.35 - vw) / bw) * bw;
    g.globalAlpha = 0.18;
    for (let x = startX; x < cam.x * 0.35 + vw + bw; x += bw) {
      g.drawImage(e.img, x, cam.y * 0.6 - hh / 2, bw, hh);
    }
    g.globalAlpha = 1;
    g.restore();
  }
  function buildChunk(cx, cy) {
    const key = cx + ":" + cy;
    if (chunkCache.has(key)) return;
    const c = makeCanvas(CHUNK * ART, CHUNK * ART);
    const g = c.getContext("2d");
    for (let dy = 0; dy < CHUNK; dy++) {
      const ty = cy * CHUNK + dy;
      if (ty < 0 || ty >= map.h) continue;
      for (let dx = 0; dx < CHUNK; dx++) {
        const tx = cx * CHUNK + dx;
        if (tx < 0 || tx >= map.w) continue;
        const ch = tileAt(tx, ty);
        const bx = dx * ART, by = dy * ART;
        if (ch === '#') g.drawImage(makeTile(hash2(tx, ty, 3) % 2 === 0 ? "wall" : "wall2"), bx, by);
        else if (ch === '.') g.drawImage(makeTile("floor"), bx, by);
        else if (ch === 'P') g.drawImage(makeTile("platform"), bx, by);
        else if (ch === 'C') g.drawImage(makeTile("climb"), bx, by);
        else { g.fillStyle = "#0d1216"; g.fillRect(bx, by, ART, ART); }
      }
    }
    chunkCache.set(key, c);
  }
  function syncSize() {
    if (!cv) return;
    const bw = Math.max(1, Math.round(window.innerWidth * dpr));
    const bh = Math.max(1, Math.round(window.innerHeight * dpr));
    if (cv.width !== bw) cv.width = bw;
    if (cv.height !== bh) cv.height = bh;
  }
  function render(now) {
    const g = ctx;
    const scale = dpr * zoom * (TILE / ART);
    const vw = cv.width / scale, vh = cv.height / scale;
    // 清屏（设备坐标全幅覆盖，防重影/残帧——原 -60 局部 fillRect 在相机偏移后不再覆盖视口）
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.fillStyle = "#070a0e";
    g.fillRect(0, 0, cv.width, cv.height);
    g.setTransform(scale, 0, 0, scale, 0, 0);
    let sx = 0, sy = 0;
    if (cam.shake > 0) { sx = (Math.random() - 0.5) * cam.shake * 2; sy = (Math.random() - 0.5) * cam.shake * 2; }
    curTrans.s = scale;
    curTrans.tx = Math.round(vw / 2 - cam.x + sx);
    curTrans.ty = Math.round(vh / 2 - cam.y + sy);
    g.translate(curTrans.tx, curTrans.ty);
    // 视差背景层（0.35× 水平 / 0.6× 垂直，暗化铺满——给横版纵深）
    drawParallaxBg(g, vw, vh);
    // 背景：深色渐变 + 淡网格（场景不显空洞）
    const bg = g.createLinearGradient(0, -60, 0, vh + 60);
    bg.addColorStop(0, "#0c1118");
    bg.addColorStop(0.6, "#0a0e13");
    bg.addColorStop(1, "#080b0f");
    g.fillStyle = bg;
    g.fillRect(-60, -60, vw + 120, vh + 120);
    g.strokeStyle = "rgba(120,150,190,0.05)";
    g.lineWidth = 1;
    for (let gx = Math.floor(cam.x / ART) * ART; gx < cam.x + vw; gx += ART * 2) {
      g.beginPath(); g.moveTo(gx, -60); g.lineTo(gx, vh + 60); g.stroke();
    }
    for (let gy = Math.floor(cam.y / ART) * ART; gy < cam.y + vh; gy += ART * 2) {
      g.beginPath(); g.moveTo(-60, gy); g.lineTo(vw + 60, gy); g.stroke();
    }
    // 房间锁定：极淡暗角示意（不画蓝框，避免像开发选框）
    if (cam.mode === "LOCKED" && currentRoom) {
      g.fillStyle = "rgba(0,0,0,0.10)";
      g.fillRect(currentRoom.x * ART - 4, currentRoom.y * ART - 4, currentRoom.w * ART + 8, 4);
      g.fillRect(currentRoom.x * ART - 4, (currentRoom.y + currentRoom.h) * ART, currentRoom.w * ART + 8, 4);
      g.fillRect(currentRoom.x * ART - 4, currentRoom.y * ART, 4, currentRoom.h * ART);
      g.fillRect((currentRoom.x + currentRoom.w) * ART, currentRoom.y * ART, 4, currentRoom.h * ART);
    }
    // chunks（相机相对剔除——勿用 vw/vh 直接对比世界坐标）
    const cxp = Math.floor(cam.x / (CHUNK * ART)), cyp = Math.floor(cam.y / (CHUNK * ART));
    const half = Math.ceil(vw / (CHUNK * ART)) + 1;
    const camL = cam.x - vw / 2 - 60, camR = cam.x + vw / 2 + 60;
    const camT = cam.y - vh / 2 - 60, camB = cam.y + vh / 2 + 60;
    for (let cy = cyp - half; cy <= cyp + half; cy++) {
      for (let cx = cxp - half; cx <= cxp + half; cx++) {
        buildChunk(cx, cy);
        const c = chunkCache.get(cx + ":" + cy);
        if (!c) continue;
        const bx = cx * CHUNK * ART, by = cy * CHUNK * ART;
        if (bx + c.width < camL || by + c.height < camT || bx > camR || by > camB) continue;
        g.drawImage(c, bx, by);
      }
    }
    // 房间道具（破单调：地板格上的箱子/管道/控制台）
    for (const p of zoneProps) {
      if (p.bx + p.spr.width < camL || p.bx > camR || p.by + p.spr.height < camT || p.by > camB) continue;
      g.drawImage(p.spr, p.bx, p.by);
    }
    // 敌人
    enemies.forEach(e => drawEnemy(g, e, now));
    // 玩家
    drawPlayer(g, now);
    // 瞬发特效（技能法阵等）
    drawFx(g, now);
    // BOSS 房红框脉冲
    if (bossState === "intro" || bossState === "fight") {
      const br = rooms.find(x => x.type === "boss");
      if (br) {
        const a = 0.25 + 0.45 * Math.abs(Math.sin(now / 600));
        g.strokeStyle = "rgba(255,60,60," + (bossState === "fight" ? 0.35 : a) + ")";
        g.lineWidth = 3;
        g.strokeRect(br.x * ART - 3, br.y * ART - 3, br.w * ART + 6, br.h * ART + 6);
      }
    }
    // 屏幕红闪
    if (redFlash > 0) {
      g.fillStyle = "rgba(255,40,40," + redFlash * 0.18 + ")";
      g.fillRect(-60, -60, vw + 120, vh + 120);
      redFlash -= 1 / 60;
    }
    // 输入锁定提示（BOSS intro）
    if (bossState === "intro") {
      g.font = "bold 8px sans-serif"; g.textAlign = "center";
      g.fillStyle = "rgba(255,80,80,0.9)";
      g.fillText("⚔ BOSS 战", vw / 2, vh * 0.2);
    }
    // 蛛丝瞄准准星（血统启用且鼠标在屏幕内）
    if (swingEnabled && mouseScreen) {
      const am = aimPoint();
      // bug 修复：aimPoint 返回 {x, y, fromMouse}，此前误用 am.mx/am.my（undefined），
      // 使 Math.abs(am.mx)<99998 恒为 false，准星从未绘制——改回 am.x/am.y 并以 fromMouse 判定
      if (am && am.fromMouse) {
        g.strokeStyle = "rgba(255,255,255,0.9)";
        g.lineWidth = 1.2;
        g.beginPath(); g.arc(am.x, am.y, 10, 0, Math.PI * 2); g.stroke();
        g.beginPath();
        g.moveTo(am.x - 18, am.y); g.lineTo(am.x - 6, am.y);
        g.moveTo(am.x + 6, am.y); g.lineTo(am.x + 18, am.y);
        g.moveTo(am.x, am.y - 18); g.lineTo(am.x, am.y - 6);
        g.moveTo(am.x, am.y + 6); g.lineTo(am.x, am.y + 18);
        g.stroke();
      }
    }
  }
  /* 常驻像素武器 + 道具使用瞬间（纯表现；判定/结算一律在 Rust） */
  // 武器名中文 → 持握视觉类别（与武器挥动共用；未命中默认近战）
  function heldWeaponStyle(name) {
    const n = name || "";
    if (/剑|刀|刃|匕首|军刀|刀刺/.test(n) && !/枪/.test(n)) return "sword";
    if (/枪|炮|狙击|弩|电磁|炮塔|发射器|弹道/.test(n)) return "gun";
    if (/斧/.test(n)) return "axe";
    if (/法杖|法器|符|咒|术|印|阵|卷轴|权杖/.test(n)) return "magic";
    if (/弓/.test(n)) return "bow";
    return "unarmed";
  }
  // 在角色本地坐标（已统一朝右、原点为头顶）手持处画像素武器；fillRect 拼形，禁止 arc/ellipse
  function drawHeldWeapon(g, now, tms, w, h) {
    const st = heldWeaponStyle(data && data.weapon);
    if (st === "unarmed") return;
    const px = w * 0.34, py = h * 0.16;        // 手部（肩前侧）
    const atk = (state === "attack");
    let ang = 0, hx = px, hy = py;
    if (atk) {
      const p = clamp(tms / 150, 0, 1);
      const k = Math.sin(p * Math.PI);          // 峰在挥动片段中部
      if (st === "sword" || st === "axe") {
        // 近战：从后上方挥向前下方（绕肩部旋转）
        ang = -1.85 + 2.3 * p;
        hx = px - (1 - k) * 4;                   // 肩部先微收再前送
        hy = py - k * 3;
      } else if (st === "gun" || st === "bow") {
        ang = -0.75 * k;                         // 枪/弓后坐上仰
        hx = px - k * 5;                         // 后坐后退
        hy = py - k * 2;                         // 枪口抬升
      } else if (st === "magic") {
        ang = 0.62 * k;                          // 法杖举起
        hy = py - k * 7;                         // 杖上举
      }
    } else {
      // 移动/待机：自然持握 + 轻微呼吸摆动
      hy += Math.sin(now / 420) * 1.2;
      if (st === "sword") ang = Math.sin(now / 500) * 0.06;
      else if (st === "magic") hy -= Math.sin(now / 430) * 1.0;
    }
    g.save();
    g.translate(hx, hy);
    g.rotate(ang);
    g.translate(-px, -py);
    if (st === "sword") {
      // 剑：柄在上、刃向下垂伸 + 护手
      g.fillStyle = "#0d1116"; g.fillRect(px - 3, py - 6, 6, 34);
      g.fillStyle = "#6b7a88"; g.fillRect(px - 2, py - 4, 4, 30);
      g.fillStyle = "#93a3b4"; g.fillRect(px - 3, py - 4, 1.6, 30);
      g.fillStyle = "#0d1116"; g.fillRect(px - 6, py - 7, 12, 3);
      g.fillStyle = "#8a4a2a"; g.fillRect(px - 1.5, py - 11, 3, 5);
    } else if (st === "gun") {
      // 枪：短横枪管朝前 + 握把
      g.fillStyle = "#0d1116"; g.fillRect(px - 3, py - 4, 11, 2); g.fillRect(px - 3, py - 6, 2, 8);
      g.fillStyle = "#46525e"; g.fillRect(px - 3, py - 4, 10, 1.6); g.fillRect(px - 3, py - 6, 1.6, 8);
      g.fillStyle = "#0d1116"; g.fillRect(px + 5, py - 4, 2, 1.6);
    } else if (st === "axe") {
      // 斧：斜探头 + 顶横刃
      g.fillStyle = "#8a4a2a"; g.fillRect(px - 1, py - 2, 2, 22);
      g.fillStyle = "#0d1116"; g.fillRect(px - 7, py - 24, 14, 5); g.fillRect(px - 7, py - 21, 2, 2);
      g.fillStyle = "#93a3b4"; g.fillRect(px - 6, py - 23, 11, 3);
    } else if (st === "magic") {
      // 法杖：木杆 + 顶端菱形(三段矩形拼) + 发光
      g.fillStyle = "#8a4a2a"; g.fillRect(px - 1, py, 2, 26);
      const glow = atk ? (0.4 + k * 0.6) : (0.5 + 0.5 * Math.sin(now / 260));
      g.fillStyle = "rgba(180,120,255," + glow + ")";
      g.fillRect(px - 3, py - 8, 6, 2); g.fillRect(px - 2, py - 10, 4, 2); g.fillRect(px - 3, py - 6, 6, 2);
      g.fillStyle = "rgba(230,200,255," + (glow * 0.8) + ")";
      g.fillRect(px - 2, py - 9, 4, 2);
    } else if (st === "bow") {
      // 弓：竖弓臂 + 弦(竖直细线)暗示（禁止 arc，故用矩形拼）
      g.fillStyle = "#8a4a2a"; g.fillRect(px - 2, py - 12, 4, 30);
      g.fillStyle = "#d8e6ff"; g.fillRect(px - 1, py - 10, 1, 26);
    }
    g.restore();
  }
  // 技能/道具"取物→举起→释放"瞬间（约 200ms；身前短过渡，非贴图）
  function drawCastAnim(g, w, h) {
    if (!castAnim) return;
    const t = castAnim.t, WAIT = 205;
    if (t >= WAIT) { castAnim = null; return; }
    const px = w * 0.10, py0 = h * 0.62;         // 腰间起点
    const cfg = castAnim.cfg || SKILL_FX.normal; // 按 14 大类取颜色+形态
    const col = cfg.color, sub = cfg.sub, glow = cfg.glow || "#ffffff";
    if (t < 70) {
      // 取出：腰侧显形小块
      const a = t / 70;
      g.fillStyle = hexA("#0d1116", 0.5 + a * 0.5); g.fillRect(px - 5, py0 - 5, 10, 10);
      g.fillStyle = hexA(col, 0.35 + a * 0.5); g.fillRect(px - 4, py0 - 4, 8, 8);
      g.fillStyle = hexA(glow, a * 0.4); g.fillRect(px - 2, py0 - 2, 4, 4);
    } else if (t < 140) {
      // 举起：抬到胸前、放大增亮、显符印（符形随大类）
      const q = (t - 70) / 70;
      const cyh = py0 - q * (py0 - h * 0.30);
      const s = 8 * (1 + q * 1.2);
      g.fillStyle = hexA("#0d1116", 0.85); g.fillRect(px - s / 2 - 1, cyh - s / 2 - 1, s + 2, s + 2);
      g.fillStyle = hexA(col, 0.6 + q * 0.4); g.fillRect(px - s / 2, cyh - s / 2, s, s);
      g.fillStyle = hexA(glow, q * 0.5);
      if (cfg.shape === "slash") {
        // 剑/刀/剑仙：横斩剑光符印
        g.fillRect(px - s / 2, cyh - 1, s, 2);
        g.fillRect(px - 1, cyh - s * 0.35, 2, s * 0.7);
      } else if (cfg.shape === "helix") {
        // 血统/基因：螺旋符印
        g.fillRect(px - 1, cyh - s / 2, 2, s);
        g.fillRect(px - s / 2, cyh - 1, s, 2);
        g.fillStyle = hexA(sub, q * 0.5);
        g.fillRect(px - s * 0.3, cyh - s * 0.3, 2, s * 0.6);
      } else if (cfg.shape === "column") {
        // 光明：竖立光柱符印
        g.fillRect(px - 1, cyh - s / 2, 2, s);
        g.fillRect(px - s * 0.3, cyh - 1, s * 0.6, 2);
      } else if (cfg.shape === "ring") {
        // 体修：圆环符印
        g.fillRect(px - s * 0.3, cyh - 1, s * 0.6, 2);
        g.fillRect(px - 1, cyh - s * 0.3, 2, s * 0.6);
      } else {
        // 法修/通用/其他：十字符印
        g.fillRect(px - 1, cyh - s / 2, 2, s); g.fillRect(px - s / 2, cyh - 1, s, 2);
      }
    } else {
      // 释放：向身前扩散消散成微粒
      const q = (t - 140) / (WAIT - 140);
      const size = 8 + q * 24, alpha = 1 - q;
      const fx0 = w * 0.44, fy0 = h * 0.40;
      g.fillStyle = hexA(col, alpha * 0.7); g.fillRect(fx0 - size / 2, fy0 - size / 2, size, size);
      g.fillStyle = hexA(glow, alpha * 0.5); g.fillRect(fx0 - 2, fy0 - size / 2, 4, size);
      const ps = [[-q * 10, -q * 6], [q * 12, q * 8], [q * 8, -q * 10], [-q * 8, q * 12]];
      for (const d of ps) {
        g.fillStyle = hexA(sub, alpha * 0.5);
        g.fillRect(fx0 + d[0] - 2, fy0 + d[1] - 2, 4, 4);
      }
    }
  }
  /* ---------------- 血统激活外观：2D 光环绘制 ----------------
   * 分两层：drawAuraGlow 呼吸光晕（radialGradient，画在角色立绘身后）；
   * drawAuraParticles 粒子光环（纯 fillRect 小方块，additive 叠加，不盖脸/武器）。
   * 未命中血统（data.bloodline 不在映射表内）时 bloodlineSpec() 返回 null，两层均跳过。
   */
  function bloodlineSpec() { return (data && data.bloodline && BLOODLINE_AURAS[data.bloodline]) || null; }

  // 呼吸光晕：径向渐变，半径与亮度随 opFreq 呼吸；色随血统。cx=角色中心 x，topY=角色顶 y（绝对 art px）。
  function drawAuraGlow(g, now, cx, topY, h) {
    const s = bloodlineSpec();
    if (!s) return;
    const t = now / 1000;
    const breathe = 0.75 + 0.25 * Math.sin(t * s.opFreq);
    const cy = topY + h * 0.45;
    const R = h * (0.8 + s.expand * breathe);
    const grad = g.createRadialGradient(cx, cy, h * 0.15, cx, cy, R);
    const a = clamp(s.op * 0.24 * breathe, 0.04, 0.5);
    grad.addColorStop(0, BL_ALPHA(s.color, a));
    grad.addColorStop(0.55, BL_ALPHA(s.color, a * 0.45));
    grad.addColorStop(1, BL_ALPHA(s.color, 0));
    g.fillStyle = grad;
    g.beginPath(); g.arc(cx, cy, R, 0, Math.PI * 2); g.fill();
  }

  // 粒子光环：绕身旋转/升腾，纯 fillRect 小方块；additive(lighter) 只增亮不盖脸。
  // 在角色本地坐标调用：cx=0（角色水平中），topY=-h（角色顶），w/h=立绘尺寸。
  function drawAuraParticles(g, now, cx, topY, w, h) {
    const s = bloodlineSpec();
    if (!s) return;
    const t = now / 1000;
    const n = s.n, form = s.form, spin = s.spin, ex = s.expand, exF = s.expandFreq;
    const breathe = 0.7 + 0.3 * Math.sin(t * s.opFreq);
    const size = Math.max(1.5, w * 0.055);
    g.globalCompositeOperation = "lighter";
    for (let i = 0; i < n; i++) {
      const ph = (i / n) * 6.2832 + (i % 7) * 0.9;
      let x, y;
      if (form === "ring") {
        // 绕身平环带：旋转体现勾玉/圣衣/灵子环
        const a = ph + t * spin;
        const r = w * 0.62 + Math.sin(t * exF + i * 0.8) * ex * w * 0.5;
        x = cx + Math.cos(a) * r;
        y = topY + h * (0.30 + ((i % 5) / 5) * 0.55) + Math.sin(t * 1.0 + i * 0.4) * 2;
      } else if (form === "column") {
        // 升腾柱：自下而上 + 顶部扩散（气焰/血雾/灵压/木遁）
        const a = ph + t * spin * 0.5;
        const life = (i % 6) / 6; // 0→1 自下而上
        const r = w * 0.30 + life * (w * 0.40 + Math.sin(t * exF + ph) * ex * w);
        x = cx + Math.cos(a) * r;
        y = topY + h * (0.55 + life * 0.50) + Math.sin(t * 1.2 + i * 0.6) * 2;
      } else if (form === "helix") {
        // 双螺旋上升火焰/电流（恶魔/龙族/宇智波/鬼灭）
        const a = ph + t * spin;
        const r = w * 0.55 + Math.sin(t * exF + i * 0.9) * ex * w;
        const rise = (((i / n) + t * 0.30) % 1);
        x = cx + Math.cos(a) * r;
        y = topY + h * rise * 0.85;
      } else {
        // sphere：全包裹球壳呼吸（狼人/纳米雾/虚化/大筒木）
        const az = ph;
        const el = (i / n) * 3.1416 + t * 0.2;
        const r = w * 0.62 + Math.sin(t * exF + ph) * ex * w;
        x = cx + Math.sin(el) * Math.cos(az + t * spin * 0.6) * r;
        y = topY + h * 0.5 + Math.cos(el) * r * 0.55 + Math.sin(t * 1.1 + i * 0.5) * 1.5;
      }
      const pa = clamp(s.op * breathe * 0.55, 0.05, 0.85);
      g.fillStyle = BL_ALPHA(s.color, pa);
      g.fillRect(x - size / 2, y - size / 2, size, size);
    }
    g.globalCompositeOperation = "source-over";
  }

  /* ---------------- 腿部：立绘下半身像素蒙皮 ----------------
   * 贴图 assets/img/pc_legs_side.png 由 tools/make_pc_legs.py 从 pc_zhengzha_side.png 的
   * 下半身抠出（按立绘白色描边 rim light 逐行取真实轮廓 → 沿腿后缘拉直 → 裁掉轮廓外的
   * 柔光/远侧腿残影），再交给 makePortrait() 走与上身完全相同的像素管线
   * （提亮 / 对比 / 整平描边 / 锐化）。所以裤腿与靴子的色块、描边、明暗与上身同源同调，
   * 不再是程序化几何梯形。
   * 绘制按 髋 / 膝 / 踝 三级关节把贴图切成三段刚性旋转（sprite skinning）：
   *   大腿 = 贴图行 [0, 膝)，小腿 = [膝, 踝)，脚 = [踝, 48)；
   * 脚段做部分反向旋转，避免"整条腿僵直地随髋甩动"，落地更稳。
   * 贴图以「腿后缘」左对齐（后缘竖直、靴尖向前自然前伸），髋关节在贴图内的 x = LEG_PIVOT。
   */
  const LEG_W = 17, LEG_H = 48;                    // 贴图 art px 尺寸（与上身 64×104 同尺度）
  const LEG_KNEE = 18.5, LEG_ANKLE = 36.9;         // 膝 / 踝 在贴图内的行（art px）
  const LEG_PIVOT = 6.75;                          // 髋关节在贴图内的 x（art px）
  const LEG_HX_NEAR = -0.25, LEG_HX_FAR = -4.85;   // 髋关节在角色本地坐标的 x
  let legSprE = null, legFarCv = null;
  function legSprite() {
    if (!legSprE) legSprE = makePortrait("pc_legs_side.png", LEG_W, LEG_H);
    return legSprE;
  }
  // 远侧腿 = 同贴图整体压暗（离屏一次性生成，不逐帧上滤镜）
  function legFarCanvas() {
    if (legFarCv) return legFarCv;
    const s = legSprite();
    if (!s || !s.cv) return null;
    const c = makeCanvas(LEG_W, LEG_H), cg = c.getContext("2d");
    cg.drawImage(s.cv, 0, 0);
    cg.globalCompositeOperation = "source-atop";
    cg.fillStyle = "rgba(6,9,14,0.40)";
    cg.fillRect(0, 0, LEG_W, LEG_H);
    legFarCv = c;
    return c;
  }
  // 一条腿：hx=髋关节本地 x，a=大腿绝对角（正=向前摆），kb=膝弯角（>=0 向后折）
  function drawOneLeg(g, img, hx, hipY, a, kb) {
    const ov = 1.2;                                // 关节上下各多画一点，盖住接缝
    const s = a - kb;                              // 小腿绝对角
    const kx = hx + Math.sin(a) * LEG_KNEE, ky = hipY + Math.cos(a) * LEG_KNEE;
    const L2 = LEG_ANKLE - LEG_KNEE, L3 = LEG_H - LEG_ANKLE;
    const ax = kx + Math.sin(s) * L2, ay = ky + Math.cos(s) * L2;
    const f = s * 0.25;                            // 脚：部分反向旋转 → 贴地
    g.save(); g.translate(hx, hipY); g.rotate(a);
    g.drawImage(img, 0, 0, LEG_W, LEG_KNEE + ov, -LEG_PIVOT, 0, LEG_W, LEG_KNEE + ov);
    g.restore();
    g.save(); g.translate(kx, ky); g.rotate(s);
    g.drawImage(img, 0, LEG_KNEE - ov, LEG_W, L2 + ov * 2, -LEG_PIVOT, -ov, LEG_W, L2 + ov * 2);
    g.restore();
    g.save(); g.translate(ax, ay); g.rotate(f);
    g.drawImage(img, 0, LEG_ANKLE - ov, LEG_W, L3 + ov, -LEG_PIVOT, -ov, LEG_W, L3 + ov);
    g.restore();
  }
  function drawLegs(g, now, w, h) {
    const spr = legSprite();
    if (!spr || !spr.cv) { drawLegsProcedural(g, now, h); return; }
    const far = legFarCanvas() || spr.cv;
    const hipY = Math.round(h * 0.54) + 1;
    // 姿态：近腿 n / 远腿 f 的大腿角与膝弯角（覆盖全部状态，不再只有 run 才有腿）
    let nA, nK, fA, fK;
    if (state === "run") {
      const t = now / 90;
      nA = Math.sin(t) * 0.62;              nK = Math.max(0, Math.sin(t + 1.35)) * 0.95;
      fA = Math.sin(t + Math.PI) * 0.62;    fK = Math.max(0, Math.sin(t + Math.PI + 1.35)) * 0.95;
    } else if (state === "jump" || state === "fly") {
      const up = P.vy < 0;
      nA = up ? 0.66 : 0.34;  nK = up ? 1.30 : 0.62;
      fA = up ? -0.44 : -0.18; fK = up ? 0.80 : 0.34;
    } else if (state === "climb") {
      const t = now / 260;
      nA = 0.44 + Math.sin(t) * 0.24;  nK = 0.90 + Math.sin(t) * 0.28;
      fA = 0.10 - Math.sin(t) * 0.24;  fK = 0.50;
    } else if (state === "dodge") {
      nA = 0.54; nK = 1.00; fA = -0.34; fK = 0.44;
    } else if (state === "swing" && swing) {
      const t = now / 200;
      nA = 0.32 + Math.sin(t) * 0.26;  nK = 0.58;
      fA = -0.26 - Math.sin(t) * 0.26; fK = 0.48;
    } else if (state === "die") {
      nA = 0.36; nK = 0.60; fA = -0.30; fK = 0.38;
    } else {                                 // idle / attack / hurt / hurt：自然站姿 + 轻微重心摆
      const s2 = Math.sin(now / 640) * 0.035;
      nA = 0.06 + s2; nK = 0.07;
      fA = -0.08 - s2; fK = 0.06;
    }
    drawOneLeg(g, far, LEG_HX_FAR, hipY, fA, fK);   // 远腿先画（被近腿遮住）
    drawOneLeg(g, spr.cv, LEG_HX_NEAR, hipY, nA, nK);
  }
  // 贴图未就绪时的兜底：仍按三关节画锥形腿（覆盖所有状态，避免"角色没有腿"）
  function drawLegsProcedural(g, now, h) {
    const hipY = Math.round(h * 0.54) + 1;
    const seg = (x0, y0, x1, y1, w0, w1, color) => {
      const ang = Math.atan2(x1 - x0, y1 - y0);
      const len = Math.hypot(x1 - x0, y1 - y0) || 1;
      g.save(); g.translate(x0, y0); g.rotate(ang);
      g.fillStyle = color;
      g.beginPath();
      g.moveTo(-w0 / 2, 0); g.lineTo(w0 / 2, 0);
      g.lineTo(w1 / 2, len); g.lineTo(-w1 / 2, len);
      g.closePath(); g.fill();
      g.restore();
    };
    const t = state === "run" ? now / 90 : 0.35;
    for (const [hx, ph, body, shoe] of [
      [LEG_HX_FAR, t + Math.PI, "#39485a", "#2c3846"],
      [LEG_HX_NEAR, t, "#4b5d72", "#3a4a5c"],
    ]) {
      const a = state === "run" ? Math.sin(ph) * 0.62 : (hx === LEG_HX_NEAR ? 0.06 : -0.08);
      const kb = state === "run" ? Math.max(0, Math.sin(ph + 1.35)) * 0.95 : 0.07;
      const kx = hx + Math.sin(a) * LEG_KNEE, ky = hipY + Math.cos(a) * LEG_KNEE;
      const s = a - kb, L2 = LEG_ANKLE - LEG_KNEE, L3 = LEG_H - LEG_ANKLE;
      const ax = kx + Math.sin(s) * L2, ay = ky + Math.cos(s) * L2;
      seg(hx, hipY, kx, ky, 11, 7, body);
      seg(kx, ky, ax, ay, 7, 5, body);
      g.fillStyle = shoe;
      g.save(); g.translate(ax, ay); g.rotate(s * 0.25);
      g.beginPath();
      g.moveTo(-4, -3); g.lineTo(9, -3); g.lineTo(12, 4); g.lineTo(-5, 4);
      g.closePath(); g.fill(); g.restore();
    }
  }

  function drawPlayer(g, now) {
    const spr = makePortrait("pc_zhengzha_side.png", 64, 104);
    const w = 64, h = 104;
    // 姿势程序化
    let ox = 0, oy = 0, rot = 0, squash = 1;
    if (state === "run") { oy = Math.sin(now / 60) * 4; ox += Math.sin(now / 60) * 4; }   // 上下浮动 + 左右 4px 步幅
    else if (state === "jump") { oy = P.vy < 0 ? -6 : 4; }
    else if (state === "climb") { oy = Math.sin(now / 90) * 4; }
    else if (state === "attack") {
      rot = -0.18 * Math.sin(Math.min(1, stateT / 150) * Math.PI);
      ox = P.facing * 4 * Math.sin(Math.min(1, stateT / 150) * Math.PI);
    }
    else if (state === "hurt") { rot = -0.22 * P.facing * -1; oy = -4; }
    else if (state === "dodge") { rot = P.facing * 0.9 * Math.min(1, stateT / 200); }
    else if (state === "fly") { oy = Math.sin(now / 250) * 6; }
    else if (state === "swing" && swing) {
      // 悬空摆荡姿态：身体向绳切向轻微前倾 + 上下浮动
      const tx = Math.cos(swing.theta), ty = -Math.sin(swing.theta);
      rot = clamp(tx * 0.22, -0.35, 0.35) * P.facing;
      oy = Math.sin(now / 130) * 4;
      ox = Math.sin(now / 170) * 3 - P.facing * 2;
    }
    else if (state === "die") { rot = 1.5 * Math.min(1, stateT / 700); }

    const px = P.x + ox, py = P.y - h + 1 + oy - (flying ? 28 : 0);
    // 身后柔光高亮（暗背景上角色必可见；压低透明度避免读成"方框"）
    const hg = g.createRadialGradient(px, py + h * 0.55, 6, px, py + h * 0.55, h * 0.55);
    hg.addColorStop(0, "rgba(160,200,240,0.06)");
    hg.addColorStop(1, "rgba(160,200,240,0)");
    g.fillStyle = hg;
    g.beginPath(); g.arc(px, py + h * 0.45, h * 0.9, 0, Math.PI * 2); g.fill();
    // 血统激活外观：呼吸光晕画在角色身后（色随血统，未命中不画）
    drawAuraGlow(g, now, px, py, h);
    // 影子（浮空分离；柔和椭圆，避免读成黑块）
    g.fillStyle = "rgba(8,10,14," + (flying ? 0.18 : 0.30) + ")";
    g.beginPath(); g.ellipse(P.x, P.y + 4, flying ? 12 : 16, flying ? 4 : 5.2, 0, 0, Math.PI * 2); g.fill();
    g.save();
    g.translate(px, py + h);
    g.rotate(rot * P.facing);
    g.translate(0, -h);
    g.scale(P.facing * 1, 1);
    if (hurtFlash > 0) {
      hurtFlash -= 1 / 60;
      g.globalAlpha = 0.75;
      g.globalCompositeOperation = "screen";
    }
    if (spr && spr.cv) {
      // 角色上半身（头+躯干）：只画立绘顶部 54%，下半身交给上面的腿部蒙皮
      const HIP = 0.54;
      const upperH = Math.round(h * HIP);
      const srcH = Math.max(1, Math.round(spr.cv.height * HIP));
      g.drawImage(spr.cv, 0, 0, spr.cv.width, srcH, -w / 2, 0, w, upperH);
    } else {
      g.fillStyle = "#3a4658"; g.fillRect(-8, 0, 16, h);
      g.fillStyle = "#e8b98a"; g.fillRect(-6, 0, 12, 12);
    }
    // 腿部动画：双腿两段式（大腿+小腿）剪刀式迈步，带提膝弯曲（拳皇/魂斗罗/我的世界式行走）
    if (state === "run") {
      const t = now / 90;                          // 步频相位（90ms 一弧度）
      const hipY = Math.round(h * 0.54), thighLen = 24, shinLen = 24;
      // 锥形腿段：从(x0,y0)到(x1,y1)，起始宽 w0 → 末端宽 w1（髋宽膝窄踝更窄，非方块）
      const seg = (x0, y0, x1, y1, w0, w1, color) => {
        const ang = Math.atan2(x1 - x0, y1 - y0);
        const len = Math.hypot(x1 - x0, y1 - y0) || 1;
        g.save(); g.translate(x0, y0); g.rotate(ang);
        g.fillStyle = color;
        g.beginPath();
        g.moveTo(-w0 / 2, 0); g.lineTo(w0 / 2, 0);
        g.lineTo(w1 / 2, len); g.lineTo(-w1 / 2, len);
        g.closePath(); g.fill();
        // 前缘高光（细亮带，去"平块"感，带圆柱体积）
        g.fillStyle = "rgba(255,255,255,0.10)";
        g.beginPath();
        g.moveTo(-w0 / 2 + 1, 1); g.lineTo(-w0 / 2 + 1 + Math.max(1.5, w0 * 0.25), 1);
        g.lineTo(-w1 / 2 + 1 + Math.max(1.5, w1 * 0.25), len - 1); g.lineTo(-w1 / 2 + 1, len - 1);
        g.closePath(); g.fill();
        g.restore();
      };
      // 前腿（近侧）与后腿（远侧），相位差 π 反相摆动
      for (const [hx, ph, body, shoe] of [
        [-9, t + Math.PI, "#34434f", "#293742"],   // 后腿（远侧，深裤色）
        [1, t, "#47596c", "#384859"],              // 前腿（近侧，略亮辨前后，贴近上身亮度）
      ]) {
        const thighAng = Math.sin(ph) * 0.55;      // 大腿摆角 ±0.55 rad
        const kneeX = hx + Math.sin(thighAng) * thighLen;
        const kneeY = hipY + Math.cos(thighAng) * thighLen;
        const kneeBend = Math.max(0, Math.sin(ph + Math.PI / 2)) * 0.7;   // 提膝弯曲（略柔和）
        const shinAng = thighAng - kneeBend;       // 小腿相对大腿后折
        const footX = kneeX + Math.sin(shinAng) * shinLen;
        const footY = kneeY + Math.cos(shinAng) * shinLen;
        // 大腿（髋13→膝7）+ 小腿（膝7→踝4）：细长锥形，贴近真腿
        seg(hx, hipY, kneeX, kneeY, 13, 7, body);
        seg(kneeX, kneeY, footX, footY, 7, 4, body);
        // 膝盖（略亮小圆，突出膝盖弯曲）
        g.fillStyle = shoe;
        g.beginPath(); g.arc(kneeX, kneeY, 3.5, 0, Math.PI * 2); g.fill();
        // 鞋：沿小腿方向前伸，鞋头朝前（非方块）
        g.fillStyle = shoe;
        g.save(); g.translate(footX, footY); g.rotate(shinAng);
        g.beginPath();
        g.moveTo(-3, -2); g.lineTo(10, -2); g.lineTo(13, 2); g.lineTo(-4, 2);
        g.closePath(); g.fill(); g.restore();
      }
    }
    // 常驻武器在手（随 attack 做挥动/后坐/举杖）；道具使用"取物→举起→释放"瞬间
    drawHeldWeapon(g, now, stateT, w, h);
    drawCastAnim(g, w, h);
    // 血统激活外观：粒子光环叠加在立绘之上（additive，纯 fillRect，不盖脸/武器；未命中不画）
    drawAuraParticles(g, now, 0, -h, w, h);
    g.restore();
    // 摆荡：蛛丝线 + 锚点 + 四肢张开的蛛形姿态
    if (state === "swing" && swing) {
      const hx = P.x - P.facing * 6, hy = P.y - h * 0.62;   // 出丝手部
      const ax = swing.ax, ay = swing.ay;
      // 蛛丝（1px 白线，二次贝塞尔轻微下垂）
      const mx = (hx + ax) / 2, my = Math.max(hy, ay) + 18;
      g.strokeStyle = "rgba(255,255,255,0.92)";
      g.lineWidth = 1;
      g.beginPath(); g.moveTo(hx, hy); g.quadraticCurveTo(mx, my, ax, ay); g.stroke();
      // 锚点小白点
      g.fillStyle = "rgba(255,255,255,0.98)";
      g.beginPath(); g.arc(ax, ay, 5, 0, Math.PI * 2); g.fill();
      g.strokeStyle = "rgba(255,255,255,0.5)";
      g.lineWidth = 1; g.beginPath(); g.arc(ax, ay, 9, 0, Math.PI * 2); g.stroke();
      // 四肢张开（程序化蛛形姿态：两臂上抬、两腿后弯，随摆动摆动）
      const sx = P.x, sy = P.y;
      const sway = Math.sin(now / 120) * 4;
      g.strokeStyle = "rgba(230,235,240,0.85)";
      g.lineWidth = 1.6;
      // 双臂（伸向锚点方向）
      const axx = ax >= sx ? 1 : -1;
      g.beginPath(); g.moveTo(sx - axx * 8, sy - h * 0.8); g.quadraticCurveTo(sx + axx * 12, sy - h * 1.15, sx + axx * 18, sy - h * 1.0 + sway * 0.5); g.stroke();
      g.beginPath(); g.moveTo(sx - axx * 8, sy - h * 0.75); g.quadraticCurveTo(sx - axx * 4, sy - h * 0.95, sx - axx * 14, sy - h * 0.85 - sway * 0.4); g.stroke();
      // 双腿（张开弯曲，反向蹬）
      g.beginPath(); g.moveTo(sx + 6, sy - 8); g.quadraticCurveTo(sx + 16, sy - 16, sx + 12, sy + 4 + sway * 0.4); g.stroke();
      g.beginPath(); g.moveTo(sx - 6, sy - 8); g.quadraticCurveTo(sx - 18, sy - 12, sx - 14, sy + 6 - sway * 0.4); g.stroke();
    }
    // 武器弧光（攻击）——按装备武器类别分帧（melee/gun/bow/magic/unarmed）
    if (state === "attack") drawAttackFX(g, now);
    // 残影
    afterImages = afterImages.filter(a => { a.t -= 1 / 60; return a.t > 0; });
    afterImages.forEach(a => {
      if (spr && spr.cv) {
        g.globalAlpha = 0.35 * a.t * 5;
        g.drawImage(spr.cv, a.x - w / 2, a.y - h, w, h);
        g.globalAlpha = 1;
      }
    });
  }
  function drawEnemy(g, e, now) {
    if (e.dead) {
      e.deadT += 1 / 60;
      if (e.deadT > 0.6) return;
      g.globalAlpha = 1 - e.deadT / 0.6;
    }
    const bob = Math.sin(now / 400 + e.x) * 3;
    const w = 60, h = 92;
    // 受击闪白
    g.save();
    if (e.flash > 0) { e.flash -= 1 / 60; g.globalAlpha = 0.8; g.globalCompositeOperation = "screen"; }
    if (enemySprite && enemySprite.cv) {
      g.drawImage(enemySprite.cv, e.x - w / 2, e.y - h + bob, w, h);
    } else {
      g.fillStyle = "#3f7a3a"; g.fillRect(e.x - 10, e.y - 28 + bob, 20, 32);
      g.fillStyle = "#ff4040"; g.fillRect(e.x - 6, e.y - 28 + bob, 4, 4); g.fillRect(e.x + 2, e.y - 28 + bob, 4, 4);
    }
    g.restore();
    // 血条
    if (e.boss || e.hp < e.maxHp) {
      const bw = 80, bx = e.x - bw / 2, by = e.y - h - 16;
      g.fillStyle = "rgba(0,0,0,0.6)"; g.fillRect(bx - 2, by - 2, bw + 4, 12);
      g.fillStyle = "#7a1f1f"; g.fillRect(bx, by, bw, 8);
      g.fillStyle = "#ff4a4a"; g.fillRect(bx, by, Math.max(0, bw * (e.hp / e.maxHp)), 8);
    }
    if (e.dead) g.globalAlpha = 1;
    // 名字
    g.font = "bold 10px sans-serif"; g.textAlign = "center";
    g.fillStyle = "rgba(0,0,0,0.7)"; g.fillText(e.name || "", e.x + 1, e.y - h - 24 + 1);
    g.fillStyle = "#ffb0b0"; g.fillText(e.name || "", e.x, e.y - h - 24);
  }

  /* ---------------- BOSS 演出 ---------------- */
  function updateBoss(dt, now) {
    if (bossState === "idle") {
      const br = rooms.find(x => x.type === "boss");
      if (br && P.x >= br.x * ART && P.x <= (br.x + br.w) * ART && P.y >= br.y * ART && P.y <= (br.y + br.h) * ART) {
        bossState = "intro"; bossIntroT = 0;
        cam.shake = 6;
        keys = {};   // 0.5s 输入锁
      }
    } else if (bossState === "intro") {
      bossIntroT += dt;
      if (bossIntroT > 0.5) { bossState = "fight"; }
    } else if (bossState === "clear") {
      bossIntroT += dt;
      if (bossIntroT > 1.2) bossState = "idle";
    }
  }

  /* ---------------- 主循环 ---------------- */
  function loop(now) {
    raf = requestAnimationFrame(loop);
    if (!zoneActive || !map) return;
    const dt = Math.min(0.05, (now - lastT) / 1000 || 0.016);
    lastT = now;
    // 帧率上限（与俯视地图一致：仅节流渲染，物理按 dt 全速推进，避免改帧率影响移动速度）
    let fps = 0;
    try { fps = (typeof window.getFpsLimit === "function") ? window.getFpsLimit() : 0; } catch (e) { fps = 0; }
    if (fps > 0 && now - lastRenderMs < 1000 / fps) return;
    lastRenderMs = now;
    if (castAnim) castAnim.t += dt * 1000;       // 道具使用演出计时
    if (attackCd > 0) attackCd -= dt;
    if (dodgeCd > 0) dodgeCd -= dt;
    if (swingFireCd > 0) swingFireCd -= dt;
    updateBoss(dt, now);
    updatePlayer(dt);
    updateCamera(dt);
    render(now);
  }
  function start() {
    if (raf || !zoneActive) return;
    lastT = performance.now();
    raf = requestAnimationFrame(loop);
  }
  function stop() {
    if (raf) { cancelAnimationFrame(raf); raf = null; }
  }

  /* ---------------- 契约 ---------------- */
  function init(containerEl, o) {
    container = containerEl;
    opts = o || {};
    container.innerHTML = "";
    cv = makeCanvas(1, 1);
    cv.style.position = "fixed";
    cv.style.left = "0"; cv.style.top = "0";
    cv.style.width = "100vw"; cv.style.height = "100vh";
    cv.style.zIndex = "1";
    cv.style.imageRendering = "pixelated";
    cv.style.background = "#070a0e";
    container.appendChild(cv);
    ctx = cv.getContext("2d");
    ctx.imageSmoothingEnabled = false;
    syncSize();
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("resize", syncSize);
    cv.addEventListener("mousemove", onMouseMove);
    cv.addEventListener("mouseleave", onMouseLeave);
    cv.addEventListener("mousedown", onMouseDown);
    window.addEventListener("mouseup", onMouseUp);
    zoneActive = true;
  }
  function setData(d) {
    data = d;
    map = (d && d.map) || null;
    bgUrl = (map && map.background) || (d && d.background) || "img_corridor.png";
    if (bgUrl) loadImg(bgUrl);
    chunkCache.clear();
    if (map) buildMap();
    // 血统启用蛛丝摆荡（setSwingMode 强制定值优先）
    swingEnabled = (swingForced != null) ? !!swingForced : !!(data && data.bloodline === "spiderman_bloodline");
    mouseScreen = null;
    swing = null;
    zoneActive = true;
    start();
  }
  function dispose() {
    zoneActive = false;
    stop();
    try { window.removeEventListener("keydown", onKeyDown); } catch (e) {}
    try { window.removeEventListener("keyup", onKeyUp); } catch (e) {}
    try { window.removeEventListener("resize", syncSize); } catch (e) {}
    try { cv && cv.removeEventListener("mousemove", onMouseMove); } catch (e) {}
    try { cv && cv.removeEventListener("mouseleave", onMouseLeave); } catch (e) {}
    try { cv && cv.removeEventListener("mousedown", onMouseDown); } catch (e) {}
    try { window.removeEventListener("mouseup", onMouseUp); } catch (e) {}
    swing = null; swingHold = false; mouseScreen = null;
    if (container) { try { container.innerHTML = ""; } catch (e) {} }
    container = null; cv = null; ctx = null;
  }
  function setResolution(lvl) {
    level = lvl;
    zoom = 2.0 * level / 1080;
    syncSize();
  }
  function setFlying(f) { flying = !!f; }
  // 强制开关蛛丝摆荡（null/undefined → 回退读 data.bloodline）
  function setSwingMode(on) {
    swingForced = (on === null || on === undefined) ? null : !!on;
    swingEnabled = (swingForced != null) ? swingForced : !!(data && data.bloodline === "spiderman_bloodline");
    if (!swingEnabled) { swing = null; swingHold = false; mouseScreen = null; }
  }
  function debug() {
    return {
      state: state, pos: { x: Math.round(P.x), y: Math.round(P.y) }, facing: P.facing,
      onGround: P.onGround, climbing: P.climbing, flying: flying,
      swing: { enabled: swingEnabled, active: !!swing, anchor: swing ? { x: Math.round(swing.ax), y: Math.round(swing.ay), len: Math.round(swing.len), theta: swing.theta.toFixed(2) } : null },
      cam: { mode: cam.mode, x: Math.round(cam.x), y: Math.round(cam.y) },
      boss: bossState, enemies: enemies.length, rooms: rooms.length,
      zoom: zoom, dpr: dpr, map: map ? { w: map.w, h: map.h } : null,
      // 玩家附近瓦片文本（验收用）
      tilesNear: map ? (() => {
        const lines = [];
        const y0 = Math.max(0, Math.floor(P.y / ART) - 4);
        for (let y = y0; y < Math.min(map.h, y0 + 10); y++) {
          const x0 = Math.max(0, Math.floor(P.x / ART) - 10);
          lines.push((map.tiles[y] || '').slice(x0, x0 + 22));
        }
        return lines;
      })() : null,
      fullTiles: map ? map.tiles : null,
      portrait: portraitCache.get("pc_zhengzha_side.png")
        ? { ready: portraitCache.get("pc_zhengzha_side.png").ready, hasCv: !!portraitCache.get("pc_zhengzha_side.png").cv, err: portraitCache.get("pc_zhengzha_side.png").err || null }
        : null,
      legs: legSprE ? { ready: legSprE.ready, hasCv: !!legSprE.cv, err: legSprE.err || null } : null,
    };
  }

  return {
    init, setData, start, stop, dispose, onZoneUpdate, keydown, keyup,
    setResolution, castEquippedSkill, setFlying, setSwingMode, debug,
  };
})();
