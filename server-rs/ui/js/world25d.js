/* world25d.js — 通用 2.5D 世界渲染层
 *
 * 逻辑权威仍在 Rust/world2d：本模块只负责把 tiles + entities 投影成
 * Three.js 正交伪 3D 画面。保留 World2D 的公开契约，失败时自动回退到
 * Canvas 渲染，因此不会改变移动、碰撞、任务、交互或存档语义。
 *
 * 约定：
 *  - OrthographicCamera，约 42° 斜俯视，地板是薄盒、墙体是挤出盒；
 *  - 人物/敌人/NPC 使用透明 PNG billboard，脚底有轻量接触阴影；
 *  - 只生成相机附近窗口，640×640 世界不会缩成角落，也不会制造大片黑边；
 *  - 蜂巢红色应急灯、冷色雾、局部闪烁仅属于主题配置，不写死地图坐标。
 */
"use strict";

window.World25D = (function () {
  const ROOT = window.WORLD25D_ASSET_ROOT || "assets/img/";
  const UNIT = 1.42;
  const FLOOR_H = 0.12;
  const WALL_H = 1.62;
  const RENDER_RADIUS = 14;
  const MINIMAP_RANGE = 19;

  let canvas = null, renderer = null, scene = null, camera = null;
  let root = null, floorRoot = null, wallRoot = null, propRoot = null, actorRoot = null;
  let data = null, fallback = false, fallbackApi = null;
  let opts = null, raf = null, dpr = 1, lastT = 0, lastMove = 0, lastRender = 0;
  let px = 0, py = 0, targetPx = 0, targetPy = 0;
  let keys = {}, lastDir = "down", moving = false, isFlying = false, detectBonus = 0;
  let webMode = false, webCb = null, webLine = null, webAnim = null, mouse = null;
  let raycaster = null, pointer = null, groundPlane = null;
  let onCanvasWheel = null, onCanvasContext = null, onCanvasMove = null, onCanvasClick = null;
  let visibleBounds = null;
  let wallOcclusionX = NaN, wallOcclusionY = NaN;
  let playerSprite = null, playerShadow = null, playerAura = null;
  let redLight = null, blueLight = null;
  let spriteCache = new Map(), textureCache = new Map(), currentThemeId = "default";
  let floorGeo = null, wallGeo = null;
  let floorMats = [], wallMats = [], wallFadeMats = [];

  // 可扩展角色 rig：以后可替换成分层身体/装备部件，当前先提供朝向贴图 + 步态参数。
  const ACTOR_RIGS = {
    player: { views: { down: "pc_zhengzha.png", up: "pc_zhengzha_back.png", side: "pc_zhengzha_side.png" }, stride: 0.055, bob: 0.10 },
  };

  const THEMES = {
    biohazard_ch1: {
      fog: 0x081016, floor: [0x35434d, 0x3f4e59, 0x2e3a44, 0x46535d],
      wall: [0x202b33, 0x29363f, 0x343d43, 0x3d3531],
      red: 0xff2538, blue: 0x5e9dff,
    },
    default: {
      fog: 0x0b1017, floor: [0x3b4650, 0x46515d],
      wall: [0x252d35, 0x313b45], red: 0xe83e4c, blue: 0x6aa9ff,
    },
  };

  function themeId() { return (data && (data.world_id || (data.world && data.world.id))) || "default"; }
  function theme() { return THEMES[themeId()] || THEMES.default; }
  function hash(x, y, s) { let h = (x * 73856093) ^ (y * 19349663) ^ (s * 83492791); return (h ^ (h >>> 13)) >>> 0; }
  function hasThree() { return !!(window.THREE && window.THREE.WebGLRenderer); }
  function fallbackCall(name, args) { if (fallbackApi && typeof fallbackApi[name] === "function") return fallbackApi[name].apply(fallbackApi, args || []); }
  function worldPos(x, y, lift) { return { x: x * UNIT, y: lift || 0, z: y * UNIT }; }
  function tileAt(x, y) {
    if (!data || x < 0 || y < 0 || x >= data.w || y >= data.h) return "#";
    const row = data.tiles[y] || "";
    return row[x] || "#";
  }

  function makeTileTexture(kind, variant) {
    const key = currentThemeId + ":tile:" + kind + ":" + variant;
    if (textureCache.has(key)) return textureCache.get(key);
    const c = document.createElement("canvas"); c.width = 96; c.height = 96;
    const g = c.getContext("2d");
    const t = theme();
    if (kind === "floor") {
      const base = "#" + (t.floor[variant % t.floor.length]).toString(16).padStart(6, "0");
      g.fillStyle = base; g.fillRect(0, 0, 96, 96);
      const grad = g.createLinearGradient(0, 0, 96, 96);
      grad.addColorStop(0, "rgba(190,215,225,.20)"); grad.addColorStop(0.5, "rgba(30,45,55,.03)"); grad.addColorStop(1, "rgba(0,0,0,.30)");
      g.fillStyle = grad; g.fillRect(0, 0, 96, 96);
      g.strokeStyle = "rgba(9,15,20,.55)"; g.lineWidth = 3;
      g.strokeRect(2, 2, 92, 92); g.beginPath(); g.moveTo(48, 5); g.lineTo(48, 91); g.moveTo(5, 48); g.lineTo(91, 48); g.stroke();
      g.fillStyle = "rgba(176,190,197,.20)"; g.fillRect(8 + (variant * 11) % 48, 14 + (variant * 17) % 50, 14, 3);
      g.fillStyle = "rgba(125,61,42,.38)"; g.fillRect(62 - (variant * 7) % 20, 68, 18, 6);
      g.fillStyle = "rgba(210,220,225,.22)"; g.fillRect(11, 9, 3, 3); g.fillRect(82, 82, 3, 3);
    } else {
      const base = "#" + (t.wall[variant % t.wall.length]).toString(16).padStart(6, "0");
      g.fillStyle = base; g.fillRect(0, 0, 96, 96);
      g.fillStyle = "rgba(158,180,193,.16)"; g.fillRect(0, 0, 96, 12);
      g.fillStyle = "rgba(0,0,0,.35)"; g.fillRect(0, 78, 96, 18);
      g.strokeStyle = "rgba(5,9,12,.65)"; g.lineWidth = 3;
      g.beginPath(); g.moveTo(4, 31); g.lineTo(92, 31); g.moveTo(4, 63); g.lineTo(92, 63); g.stroke();
      g.fillStyle = variant === 3 ? "rgba(164,72,42,.55)" : "rgba(135,155,168,.35)";
      g.fillRect(12 + (variant * 9) % 24, 42, 22, 5); g.fillRect(62, 72 - (variant * 7) % 16, 16, 5);
      g.fillStyle = "rgba(226,235,240,.42)"; g.fillRect(18, 4, 4, 4); g.fillRect(74, 34, 4, 4);
    }
    const tex = new THREE.CanvasTexture(c);
    tex.colorSpace = THREE.SRGBColorSpace || tex.colorSpace;
    tex.anisotropy = Math.min(4, renderer && renderer.capabilities ? renderer.capabilities.getMaxAnisotropy() : 1);
    textureCache.set(key, tex); return tex;
  }

  function makeMaterials() {
    if (!hasThree()) return;
    const t = theme();
    floorMats = t.floor.map((_, i) => new THREE.MeshStandardMaterial({ map: makeTileTexture("floor", i), color: 0xffffff, roughness: 0.92, metalness: 0.18 }));
    wallMats = t.wall.map((_, i) => new THREE.MeshStandardMaterial({ map: makeTileTexture("wall", i), color: 0xffffff, roughness: 0.84, metalness: 0.26, transparent: true, opacity: 0.94 }));
    wallFadeMats = t.wall.map((_, i) => new THREE.MeshStandardMaterial({ map: makeTileTexture("wall", i), color: 0xffffff, roughness: 0.84, metalness: 0.26, transparent: true, opacity: 0.25, depthWrite: false }));
    floorMats.concat(wallMats, wallFadeMats).forEach(m => { m.userData.shared25d = true; });
    floorGeo = new THREE.BoxGeometry(UNIT * 0.98, FLOOR_H, UNIT * 0.98);
    wallGeo = new THREE.BoxGeometry(UNIT * 0.98, WALL_H, UNIT * 0.98);
  }

  function loadTexture(path) {
    const key = currentThemeId + ":img:" + path;
    if (textureCache.has(key)) return textureCache.get(key);
    const tex = new THREE.TextureLoader().load(ROOT + path);
    tex.colorSpace = THREE.SRGBColorSpace || tex.colorSpace;
    tex.minFilter = THREE.LinearFilter; tex.magFilter = THREE.LinearFilter;
    textureCache.set(key, tex); return tex;
  }
  function actorSprite(path, w, h, tint) {
    const key = currentThemeId + ":" + path + ":" + (tint || "");
    let tex = spriteCache.get(key);
    if (!tex) { tex = loadTexture(path); spriteCache.set(key, tex); }
    const mat = new THREE.SpriteMaterial({ map: tex, color: tint || 0xffffff, transparent: true, alphaTest: 0.04, depthTest: true, depthWrite: false });
    const s = new THREE.Sprite(mat); s.scale.set(w, h, 1); s.center.set(0.5, 0); s.renderOrder = 20; return s;
  }

  function releaseTree(node) {
    if (!node) return;
    const disposedMaterials = new Set();
    node.traverse(obj => {
      if (obj.geometry && !obj.userData.shared25d) obj.geometry.dispose();
      const materials = Array.isArray(obj.material) ? obj.material : (obj.material ? [obj.material] : []);
      materials.forEach(m => { if (m && !m.userData.shared25d && !disposedMaterials.has(m)) { disposedMaterials.add(m); m.dispose(); } });
    });
  }
  function clearRoot(node, disposeOwned) {
    if (!node) return;
    if (disposeOwned) releaseTree(node);
    while (node.children.length) node.remove(node.children[node.children.length - 1]);
  }
  function makeInstances(node, geo, materials, buckets, cast, receive) {
    materials.forEach((mat, variant) => {
      const list = buckets[variant] || [];
      if (!list.length) return;
      const inst = new THREE.InstancedMesh(geo, mat, list.length);
      const matrix = new THREE.Matrix4();
      list.forEach((q, i) => { const p = worldPos(q.x, q.y, 0); matrix.makeTranslation(p.x, q.yLift, p.z); inst.setMatrixAt(i, matrix); });
      inst.instanceMatrix.needsUpdate = true; inst.castShadow = !!cast; inst.receiveShadow = !!receive;
      inst.userData.cells = list; inst.userData.shared25d = true; node.add(inst);
    });
  }
  function buildWallInstances(force) {
    if (!wallRoot || !visibleBounds) return;
    const ox = Math.round(targetPx), oy = Math.round(targetPy);
    if (!force && ox === wallOcclusionX && oy === wallOcclusionY) return;
    wallOcclusionX = ox; wallOcclusionY = oy;
    clearRoot(wallRoot);
    const normal = wallMats.map(() => []), faded = wallFadeMats.map(() => []);
    const sum = targetPx + targetPy;
    for (let y = visibleBounds.minY; y <= visibleBounds.maxY; y++) for (let x = visibleBounds.minX; x <= visibleBounds.maxX; x++) {
      if (tileAt(x, y) !== "#") continue;
      const variant = hash(x, y, 7) % wallMats.length;
      const near = x + y > sum + 0.5 && Math.abs(x - targetPx) + Math.abs(y - targetPy) < 10;
      (near ? faded[variant] : normal[variant]).push({ x, y, yLift: WALL_H / 2 });
    }
    makeInstances(wallRoot, wallGeo, wallMats, normal, true, true);
    makeInstances(wallRoot, wallGeo, wallFadeMats, faded, false, true);
  }
  function buildVisible() {
    if (fallback || !data || !floorRoot) return;
    const cx = Math.round(targetPx), cy = Math.round(targetPy);
    const minX = Math.max(0, cx - RENDER_RADIUS), maxX = Math.min(data.w - 1, cx + RENDER_RADIUS);
    const minY = Math.max(0, cy - RENDER_RADIUS), maxY = Math.min(data.h - 1, cy + RENDER_RADIUS);
    const next = { minX, maxX, minY, maxY };
    if (visibleBounds && visibleBounds.minX === minX && visibleBounds.maxX === maxX && visibleBounds.minY === minY && visibleBounds.maxY === maxY) return;
    visibleBounds = next; clearRoot(floorRoot); clearRoot(wallRoot);
    const floorBuckets = floorMats.map(() => []);
    for (let y = minY; y <= maxY; y++) for (let x = minX; x <= maxX; x++) {
      if (tileAt(x, y) !== "#") floorBuckets[hash(x, y, 3) % floorMats.length].push({ x, y, yLift: -FLOOR_H / 2 });
    }
    makeInstances(floorRoot, floorGeo, floorMats, floorBuckets, false, true);
    buildWallInstances(true); syncProps();
  }

  function contactShadow(x, z, scale) {
    const geo = new THREE.CircleGeometry(0.34 * (scale || 1), 18);
    const mat = new THREE.MeshBasicMaterial({ color: 0x020407, transparent: true, opacity: 0.55, depthWrite: false, side: THREE.DoubleSide });
    const m = new THREE.Mesh(geo, mat); m.rotation.x = -Math.PI / 2; m.position.set(x, 0.045, z); m.scale.y = 0.42; m.renderOrder = 10; return m;
  }
  function addBeacon(x, y, color, kind) {
    const p = worldPos(x, y, 0), group = new THREE.Group();
    const base = new THREE.Mesh(new THREE.CylinderGeometry(0.22, 0.30, 0.26, 8), new THREE.MeshStandardMaterial({ color: 0x25313a, roughness: 0.7, metalness: 0.6 }));
    base.position.set(p.x, 0.14, p.z); group.add(base);
    const ring = new THREE.Mesh(new THREE.TorusGeometry(kind === "zone" ? 0.54 : 0.38, 0.055, 6, 24), new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.88 }));
    ring.rotation.x = -Math.PI / 2; ring.position.set(p.x, 0.22, p.z); group.add(ring);
    const core = new THREE.Mesh(new THREE.CylinderGeometry(kind === "zone" ? 0.25 : 0.14, 0.18, kind === "zone" ? 0.18 : 0.48, 8), new THREE.MeshStandardMaterial({ color, emissive: color, emissiveIntensity: 1.2, roughness: 0.4 }));
    core.position.set(p.x, kind === "zone" ? 0.19 : 0.42, p.z); group.add(core);
    group.userData.phase = hash(x, y, kind === "zone" ? 4 : 9) / 1000; propRoot.add(group);
  }
  function addHiveFloorDetail(x, y, variant) {
    const p = worldPos(x, y, 0), group = new THREE.Group();
    if (variant === 0) {
      const grate = new THREE.Mesh(
        new THREE.BoxGeometry(UNIT * 0.72, 0.035, UNIT * 0.72),
        new THREE.MeshStandardMaterial({ color: 0x17242c, metalness: 0.72, roughness: 0.48 })
      );
      grate.position.set(p.x, 0.025, p.z); group.add(grate);
      for (let i = -2; i <= 2; i++) {
        const slit = new THREE.Mesh(
          new THREE.BoxGeometry(UNIT * 0.58, 0.012, 0.025),
          new THREE.MeshBasicMaterial({ color: 0x6f8793 })
        );
        slit.position.set(p.x, 0.052, p.z + i * 0.12); group.add(slit);
      }
    } else {
      const dark = new THREE.MeshStandardMaterial({ color: 0x151d22, metalness: 0.35, roughness: 0.72 });
      const amber = new THREE.MeshBasicMaterial({ color: 0xd69a35 });
      for (let i = -2; i <= 2; i++) {
        const stripe = new THREE.Mesh(new THREE.BoxGeometry(UNIT * 0.18, 0.025, 0.10), i % 2 ? dark : amber);
        stripe.position.set(p.x + i * UNIT * 0.16, 0.035, p.z); stripe.rotation.y = -0.52; group.add(stripe);
      }
    }
    propRoot.add(group);
  }
  function addHiveWallLight(x, y) {
    const p = worldPos(x, y, 0), group = new THREE.Group();
    const cage = new THREE.Mesh(
      new THREE.BoxGeometry(0.46, 0.18, 0.18),
      new THREE.MeshStandardMaterial({ color: 0x26343d, metalness: 0.72, roughness: 0.46 })
    );
    const lamp = new THREE.Mesh(
      new THREE.BoxGeometry(0.30, 0.08, 0.19),
      new THREE.MeshBasicMaterial({ color: 0xff3044 })
    );
    cage.position.set(p.x, 1.18, p.z); lamp.position.set(p.x, 1.18, p.z - 0.10); group.add(cage, lamp);
    group.userData.emergencyLamp = true; group.userData.phase = hash(x, y, 81) / 1000; propRoot.add(group);
  }
  function decorateHive() {
    if (!visibleBounds || themeId() !== "biohazard_ch1") return;
    for (let y = visibleBounds.minY; y <= visibleBounds.maxY; y++) for (let x = visibleBounds.minX; x <= visibleBounds.maxX; x++) {
      const ch = tileAt(x, y), h = hash(x, y, 23);
      if (ch !== "#" && h % 79 === 0) addHiveFloorDetail(x, y, 0);
      else if (ch !== "#" && h % 97 === 0) addHiveFloorDetail(x, y, 1);
      else if (ch === "#" && h % 109 === 0 && tileAt(x, y + 1) !== "#") addHiveWallLight(x, y);
    }
  }
  function syncProps() {
    if (!propRoot || !data) return;
    clearRoot(propRoot, true);
    decorateHive();
    (data.points || []).forEach(p => addBeacon(p.x, p.y, 0xffc35a, "point"));
    (data.zones || []).forEach(z => addBeacon(z.x, z.y, 0x6c79ff, "zone"));
    (data.portals || []).forEach(p => addBeacon(p.x, p.y, 0x35c9ff, "portal"));
    (data.gates || []).forEach(g => {
      const q = worldPos(g.x, g.y, 0);
      const mat = new THREE.MeshStandardMaterial({ color: g.locked ? 0x742533 : 0x2b805d, emissive: g.locked ? 0x3a0810 : 0x092a1d, emissiveIntensity: 0.9, roughness: 0.65, metalness: 0.35 });
      const door = new THREE.Mesh(new THREE.BoxGeometry(UNIT * 0.72, 1.28, 0.18), mat); door.position.set(q.x, 0.66, q.z); propRoot.add(door);
      const light = new THREE.PointLight(g.locked ? 0xff2438 : 0x4cff9c, 0.8, 3.2); light.position.set(q.x, 1.2, q.z - 0.28); propRoot.add(light);
    });
  }
  function portraitFor(name, npc) {
    const n = String(name || "");
    if (npc) {
      if (n.indexOf("蕾恩") >= 0 || /rain/i.test(n)) return "npc_soldier.png";
      if (n.indexOf("张杰") >= 0) return "npc_guard.png";
      if (n.indexOf("卡普兰") >= 0) return "npc_doctor.png";
      return "npc_survivor.png";
    }
    if (n.indexOf("舔食") >= 0 || /licker/i.test(n)) return "enemy_licker.png";
    if (n.indexOf("猎杀") >= 0 || /hunter/i.test(n)) return "enemy_hunter.png";
    if (n.indexOf("暴君") >= 0 || /tyrant/i.test(n)) return "enemy_tyrant.png";
    return "enemy_zombie.png";
  }
  function bloodlineColor() {
    const id = data && data.hud && data.hud.bloodline;
    const colors = { vampire: 0xcc2233, demon_bloodline: 0xff4466, dragon_bloodline: 0xffcf3a, saiyan_bloodline: 0xffdf3a, saint_bloodline: 0x7fd4ff, cyber_prosthetic: 0x42c8ff, shinigami_bloodline: 0x5aa2ff };
    return colors[id] || null;
  }
  function addActor(sprite, x, y, kind) {
    const p = worldPos(x, y, 0), g = new THREE.Group();
    g.add(contactShadow(p.x, p.z, kind === "enemy" ? 1.1 : 1));
    sprite.position.set(p.x, 0.05, p.z); g.add(sprite); g.userData.grid = { x, y, kind }; actorRoot.add(g); return g;
  }
  function syncActors() {
    if (!actorRoot || !data) return;
    clearRoot(actorRoot, true); playerSprite = null; playerShadow = null;
    const pp = worldPos(px, py, 0);
    playerShadow = contactShadow(pp.x, pp.z, 1.16);
    actorRoot.add(playerShadow);
    playerSprite = actorSprite(ACTOR_RIGS.player.views.down, 2.10, 3.35);
    playerSprite.position.set(pp.x, 0.06, pp.z); actorRoot.add(playerSprite);
    const bc = bloodlineColor();
    if (bc) {
      playerAura = new THREE.Mesh(new THREE.TorusGeometry(0.62, 0.045, 6, 28), new THREE.MeshBasicMaterial({ color: bc, transparent: true, opacity: 0.65 }));
      playerAura.rotation.x = -Math.PI / 2; playerAura.position.set(pp.x, 0.10, pp.z); actorRoot.add(playerAura);
    } else playerAura = null;
    (data.enemies || []).filter(e => e.alive !== false).forEach(e => addActor(actorSprite(portraitFor(e.name, false), 1.75, 2.85), e.x, e.y, "enemy"));
    (data.npcs || []).forEach(n => addActor(actorSprite(portraitFor(n.name, true), 1.80, 2.95), n.x, n.y, "npc"));
  }

  function playerAsset() {
    if (lastDir === "up") return ACTOR_RIGS.player.views.up;
    if (lastDir === "sideL" || lastDir === "sideR") return ACTOR_RIGS.player.views.side;
    return ACTOR_RIGS.player.views.down;
  }
  function updatePlayerView() {
    if (!playerSprite) return;
    const next = loadTexture(playerAsset());
    if (playerSprite.material.map !== next) { playerSprite.material.map = next; playerSprite.material.needsUpdate = true; }
    playerSprite.scale.x = lastDir === "sideL" ? -2.10 : 2.10;
  }

  function updateWallOcclusion() { buildWallInstances(false); }
  function updateCamera(now, dt) {
    if (!camera || !data) return;
    const k = 1 - Math.exp(-Math.min(0.08, dt || 1 / 60) * 8);
    px += (targetPx - px) * k; py += (targetPy - py) * k;
    const t = worldPos(px, py, 0), pos = { x: t.x + 8.5, y: 11.8, z: t.z + 8.5 };
    camera.position.set(pos.x, pos.y, pos.z); camera.lookAt(t.x, 0, t.z);
    if (playerSprite) {
      const bob = moving ? Math.abs(Math.sin(now / 120)) * ACTOR_RIGS.player.bob : Math.sin(now / 800) * 0.025;
      const sway = moving ? Math.sin(now / 120) * ACTOR_RIGS.player.stride : 0;
      playerSprite.position.set(t.x + sway, (isFlying ? 0.44 : 0.06) + bob, t.z);
      updatePlayerView();
      if (playerShadow) { playerShadow.position.set(t.x, 0.045, t.z); playerShadow.material.opacity = isFlying ? 0.22 : 0.55; playerShadow.scale.set(isFlying ? 0.78 : 1.16, isFlying ? 0.30 : 0.42, 1); }
      if (playerAura) { playerAura.position.set(t.x, 0.10, t.z); playerAura.rotation.z = now / 900; playerAura.scale.setScalar(1 + Math.sin(now / 260) * 0.08); }
    }
    if (redLight) { redLight.position.set(t.x + 1.8, 2.5, t.z + 1.8); redLight.intensity = 1.55 + 0.36 * (0.5 + 0.5 * Math.sin(now / 820)); }
    if (blueLight) blueLight.position.set(t.x - 2.6, 3.8, t.z - 2.6);
    if (propRoot) propRoot.children.forEach(g => {
      if (g.userData.emergencyLamp) g.scale.setScalar(0.96 + 0.06 * (0.5 + 0.5 * Math.sin(now / 430 + g.userData.phase)));
    });
    updateWebLine(now);
  }

  function webPoint(x, y, lift) { const p = worldPos(x, y, lift || 0.04); return new THREE.Vector3(p.x, p.y, p.z); }
  function updateWebLine(now) {
    if (!webLine || !webAnim || !camera) return;
    const dt = webAnim.last ? Math.min(0.08, Math.max(0.001, (now - webAnim.last) / 1000)) : 1 / 60;
    webAnim.last = now; webAnim.t = Math.min(1.2, webAnim.t + dt);
    const q = Math.min(1, webAnim.t / 0.45), ease = q * q * (3 - 2 * q);
    const from = webAnim.from.clone().lerp(webAnim.to, ease); from.y += Math.sin(q * Math.PI) * 0.65;
    const mid = from.clone().lerp(webAnim.to, 0.5); mid.y += 0.5;
    webLine.geometry.setFromPoints([from, mid, webAnim.to]); webLine.geometry.computeBoundingSphere();
    webLine.material.opacity = webAnim.t > 1 ? Math.max(0, 1 - (webAnim.t - 1) * 5) : 0.92;
    if (webAnim.t >= 1.2) { webLine.removeFromParent(); webLine.geometry.dispose(); webLine.material.dispose(); webLine = null; webAnim = null; }
  }
  function updateMouse(e) {
    if (!camera || !raycaster || !pointer || !groundPlane || !canvas) return;
    const r = canvas.getBoundingClientRect();
    pointer.set(((e.clientX - r.left) / Math.max(1, r.width)) * 2 - 1, -((e.clientY - r.top) / Math.max(1, r.height)) * 2 + 1);
    raycaster.setFromCamera(pointer, camera);
    const hit = new THREE.Vector3();
    if (raycaster.ray.intersectPlane(groundPlane, hit)) mouse = { x: hit.x, z: hit.z, tx: Math.floor(hit.x / UNIT), ty: Math.floor(hit.z / UNIT) };
  }
  function clickWeb(e) {
    if (!webMode || !webCb) return;
    updateMouse(e);
    if (!mouse) return;
    webLine = webLine || new THREE.Line(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.92, depthTest: false }));
    scene.add(webLine);
    webAnim = { from: webPoint(px, py, 1.35), to: webPoint(mouse.tx, mouse.ty, 1.2), t: 0, last: 0 };
    webCb(mouse.tx, mouse.ty);
  }

  function exploredSet() { return new Set((data && data.explored || []).map(String)); }
  function mapCell(g, tx, ty, cell, ox, oy, explored) {
    if (!data || tx < 0 || ty < 0 || tx >= data.w || ty >= data.h) return;
    if (!explored.has(tx + ":" + ty)) return;
    g.fillStyle = tileAt(tx, ty) === "#" ? "#111a20" : "#425665";
    g.fillRect(ox + tx * cell, oy + ty * cell, Math.ceil(cell + 0.5), Math.ceil(cell + 0.5));
  }
  function drawMap(canvas2, full) {
    if (!data || !canvas2) return;
    const g = canvas2.getContext("2d"), explored = exploredSet(); g.clearRect(0, 0, canvas2.width, canvas2.height); g.fillStyle = "#05080b"; g.fillRect(0, 0, canvas2.width, canvas2.height);
    if (full) {
      const cell = Math.min(canvas2.width / data.w, canvas2.height / data.h); const ox = (canvas2.width - data.w * cell) / 2, oy = (canvas2.height - data.h * cell) / 2;
      for (let y = 0; y < data.h; y++) for (let x = 0; x < data.w; x++) mapCell(g, x, y, cell, ox, oy, explored);
      drawMarkers(g, cell, ox, oy, 0, 0, data.w, data.h);
    } else {
      const range = Math.min(MINIMAP_RANGE + detectBonus * 2, Math.max(data.w, data.h)); const cell = Math.min(canvas2.width, canvas2.height) / range;
      const ox = (canvas2.width - cell * range) / 2 - (Math.round(px) - Math.floor(range / 2)) * cell;
      const oy = (canvas2.height - cell * range) / 2 - (Math.round(py) - Math.floor(range / 2)) * cell;
      const x0 = Math.max(0, Math.round(px) - Math.ceil(range / 2) - 1), x1 = Math.min(data.w - 1, Math.round(px) + Math.ceil(range / 2) + 1);
      const y0 = Math.max(0, Math.round(py) - Math.ceil(range / 2) - 1), y1 = Math.min(data.h - 1, Math.round(py) + Math.ceil(range / 2) + 1);
      for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) mapCell(g, x, y, cell, ox, oy, explored);
      drawMarkers(g, cell, ox, oy, x0, y0, x1, y1);
    }
  }
  function drawMarkers(g, cell, ox, oy, x0, y0, x1, y1) {
    const dot = (x, y, c, r) => { if (x < x0 || y < y0 || x > x1 || y > y1) return; g.fillStyle = c; g.beginPath(); g.arc(ox + (x + 0.5) * cell, oy + (y + 0.5) * cell, Math.max(2, r), 0, Math.PI * 2); g.fill(); };
    (data.enemies || []).forEach(e => { if (e.alive !== false) dot(e.x, e.y, "#ff4654", cell * 0.3); });
    (data.npcs || []).forEach(e => dot(e.x, e.y, "#4ee58b", cell * 0.28));
    (data.points || []).forEach(e => dot(e.x, e.y, "#ffd05e", cell * 0.25));
    (data.zones || []).forEach(e => dot(e.x, e.y, "#7b83ff", cell * 0.30));
    (data.portals || []).forEach(e => dot(e.x, e.y, "#4bd7ff", cell * 0.28));
    const r = Math.max(3, cell * 0.36), x = ox + (Math.round(px) + 0.5) * cell, y = oy + (Math.round(py) + 0.5) * cell;
    g.fillStyle = "#fff"; g.beginPath(); g.moveTo(x, y - r); g.lineTo(x + r * .8, y + r * .65); g.lineTo(x - r * .8, y + r * .65); g.closePath(); g.fill();
  }
  function drawMinimap() { const c = document.getElementById("minimap"); if (c) drawMap(c, false); }
  function drawBigmap() { const c = document.getElementById("bigmap"); if (c) drawMap(c, true); }

  function resize() {
    if (fallback) return fallbackCall("resize");
    if (!renderer || !canvas) return;
    renderer.setPixelRatio(Math.max(0.5, dpr)); renderer.setSize(window.innerWidth, window.innerHeight, false);
    const aspect = Math.max(0.4, window.innerWidth / Math.max(1, window.innerHeight));
    const span = 11.5; camera.left = -span * aspect; camera.right = span * aspect; camera.top = span; camera.bottom = -span; camera.updateProjectionMatrix();
  }
  function render(now) {
    if (fallback || !renderer || !scene) return;
    const fps = typeof window.getFpsLimit === "function" ? (+window.getFpsLimit() || 0) : 0;
    if (fps > 0 && now - lastRender < 1000 / fps) return;
    const dt = lastT ? Math.min(0.08, Math.max(0.001, (now - lastT) / 1000)) : 1 / 60;
    lastT = now; lastRender = now; updateCamera(now, dt);
    renderer.render(scene, camera);
    if (now - lastMove > 120) { const intent = moveIntent(); if ((intent[0] || intent[1]) && opts && opts.onMove) { opts.onMove(intent[0], intent[1]); lastMove = now; moving = true; } else moving = false; }
    drawMinimap();
  }
  function loop(now) { raf = requestAnimationFrame(loop); if (!data) return; buildVisible(); render(now); }
  function start() { if (fallback) return fallbackCall("start"); if (!raf) { lastT = 0; raf = requestAnimationFrame(loop); } }
  function stop() { if (fallback) return fallbackCall("stop"); if (raf) { cancelAnimationFrame(raf); raf = null; } }

  function moveIntent() {
    let dx = 0, dy = 0;
    if (keys.arrowup || keys.w) dy = -1; if (keys.arrowdown || keys.s) dy = 1; if (keys.arrowleft || keys.a) dx = -1; if (keys.arrowright || keys.d) dx = 1;
    if (dx < 0) lastDir = "sideL"; else if (dx > 0) lastDir = "sideR"; else if (dy < 0) lastDir = "up"; else if (dy > 0) lastDir = "down";
    return [dx, dy];
  }
  function nearbyList() { return data ? (data.nearby || []).filter(o => Math.abs(o.dx) + Math.abs(o.dy) <= 1) : []; }
  function keydown(e) { if (fallback) return fallbackCall("keydown", [e]); const k = String(e.key || "").toLowerCase(); if (["arrowup", "arrowdown", "arrowleft", "arrowright", "w", "a", "s", "d"].includes(k)) { e.preventDefault(); keys[k] = true; } if (k === "e" && opts && opts.onInteract) { const n = nearbyList(); if (n.length) opts.onInteract(n[0].id); } }
  function keyup(e) { if (fallback) return fallbackCall("keyup", [e]); keys[String(e.key || "").toLowerCase()] = false; }
  function setData(d) {
    data = d || {}; px = targetPx = +data.px || 0; py = targetPy = +data.py || 0;
    const nextTheme = themeId();
    if (fallback) return fallbackCall("setData", [d]);
    if (nextTheme !== currentThemeId) {
      currentThemeId = nextTheme;
      const t = theme();
      if (scene) { scene.background = new THREE.Color(t.fog); scene.fog = new THREE.Fog(t.fog, 18, 42); }
      if (redLight) redLight.color.setHex(t.red); if (blueLight) blueLight.color.setHex(t.blue);
      floorMats.concat(wallMats, wallFadeMats).forEach(m => m.dispose());
      floorMats = []; wallMats = []; wallFadeMats = [];
    }
    if (!floorMats.length) makeMaterials();
    visibleBounds = null; buildVisible(); syncActors(); updatePlayerView(); drawMinimap(); start();
  }
  function setPlayer(x, y) {
    if (fallback) return fallbackCall("setPlayer", [x, y]);
    targetPx = +x || 0; targetPy = +y || 0;
    const pad = 6;
    if (!visibleBounds || targetPx < visibleBounds.minX + pad || targetPx > visibleBounds.maxX - pad || targetPy < visibleBounds.minY + pad || targetPy > visibleBounds.maxY - pad) visibleBounds = null;
    drawMinimap();
  }
  function init(cv, o) {
    canvas = cv; opts = o || {}; webCb = opts.onWebZip || null;
    if (!hasThree()) { fallback = true; fallbackApi = window.World2D; return fallbackCall("init", [cv, o]); }
    try { renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, powerPreference: "high-performance", preserveDrawingBuffer: !!window.WORLD25D_CAPTURE }); }
    catch (e) { fallback = true; fallbackApi = window.World2D; return fallbackCall("init", [cv, o]); }
    renderer.outputColorSpace = THREE.SRGBColorSpace || renderer.outputColorSpace; renderer.shadowMap.enabled = true; renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    currentThemeId = themeId();
    scene = new THREE.Scene(); scene.background = new THREE.Color(theme().fog); scene.fog = new THREE.Fog(theme().fog, 18, 42);
    camera = new THREE.OrthographicCamera(-11.5, 11.5, 11.5, -11.5, 0.1, 150); camera.position.set(8.5, 11.8, 8.5);
    raycaster = new THREE.Raycaster(); pointer = new THREE.Vector2(); groundPlane = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
    root = new THREE.Group(); floorRoot = new THREE.Group(); wallRoot = new THREE.Group(); propRoot = new THREE.Group(); actorRoot = new THREE.Group(); root.add(floorRoot, wallRoot, propRoot, actorRoot); scene.add(root);
    scene.add(new THREE.HemisphereLight(0xaec3d4, 0x111820, 1.65));
    const key = new THREE.DirectionalLight(0xd9e8ff, 1.55); key.position.set(12, 28, 10); key.castShadow = true; key.shadow.mapSize.set(1024, 1024); scene.add(key);
    redLight = new THREE.PointLight(theme().red, 2.0, 14); scene.add(redLight); blueLight = new THREE.PointLight(theme().blue, 1.2, 24); scene.add(blueLight);
    canvas.style.position = "fixed"; canvas.style.left = "0"; canvas.style.top = "0"; canvas.style.width = "100vw"; canvas.style.height = "100vh"; canvas.style.zIndex = "1"; canvas.style.imageRendering = "auto"; canvas.style.background = "#081016";
    onCanvasWheel = e => { if (e.cancelable) e.preventDefault(); };
    onCanvasContext = e => e.preventDefault(); onCanvasMove = updateMouse; onCanvasClick = clickWeb;
    canvas.addEventListener("wheel", onCanvasWheel, { passive: false }); canvas.addEventListener("contextmenu", onCanvasContext);
    canvas.addEventListener("mousemove", onCanvasMove); canvas.addEventListener("click", onCanvasClick);
    window.addEventListener("resize", resize); resize(); start();
  }
  function dispose() {
    if (fallback) return fallbackCall("dispose");
    stop(); window.removeEventListener("resize", resize);
    if (canvas) { canvas.removeEventListener("wheel", onCanvasWheel); canvas.removeEventListener("contextmenu", onCanvasContext); canvas.removeEventListener("mousemove", onCanvasMove); canvas.removeEventListener("click", onCanvasClick); }
    if (webLine) { webLine.removeFromParent(); webLine.geometry.dispose(); webLine.material.dispose(); webLine = null; webAnim = null; }
    clearRoot(propRoot, true); clearRoot(actorRoot, true); clearRoot(floorRoot); clearRoot(wallRoot);
    floorMats.concat(wallMats, wallFadeMats).forEach(m => m.dispose());
    if (floorGeo) floorGeo.dispose(); if (wallGeo) wallGeo.dispose();
    if (renderer) renderer.dispose(); renderer = null; scene = null; camera = null; canvas = null; root = null; floorRoot = null; wallRoot = null; propRoot = null; actorRoot = null; floorGeo = null; wallGeo = null; floorMats = []; wallMats = []; wallFadeMats = []; playerSprite = null; playerShadow = null; playerAura = null; raycaster = null; pointer = null; groundPlane = null;
  }
  function setDpr(v) { dpr = Math.max(0.5, +v || 1); if (fallback) return fallbackCall("setDpr", [v]); resize(); }
  function clearKeys() { keys = {}; moving = false; if (fallback) return fallbackCall("clearKeys"); }
  function setFlying(v) { isFlying = !!v; if (fallback) return fallbackCall("setFlying", [v]); }
  function setDetectBonus(v) { detectBonus = Math.max(0, +v || 0); if (fallback) return fallbackCall("setDetectBonus", [v]); drawMinimap(); }
  function setWebMode(v) { webMode = !!v; if (fallback) return fallbackCall("setWebMode", [v]); if (!webMode) { mouse = null; webAnim = null; if (webLine) { webLine.removeFromParent(); webLine.geometry.dispose(); webLine.material.dispose(); webLine = null; } } }
  function webZipStart(d) {
    if (fallback) return fallbackCall("webZipStart", [d]);
    if (!d || !d.from || !d.to || !scene) return;
    if (webLine) { webLine.removeFromParent(); webLine.geometry.dispose(); webLine.material.dispose(); }
    webLine = new THREE.Line(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: 0xffffff, transparent: true, opacity: 0.92, depthTest: false }));
    scene.add(webLine); webAnim = { from: webPoint(d.from[0], d.from[1], 1.35), to: webPoint(d.to[0], d.to[1], 1.2), t: 0, last: 0 };
  }
  function debug() { if (fallback) return fallbackCall("debug"); return { renderer: "three-orthographic-25d", w: data ? data.w : -1, h: data ? data.h : -1, px: Math.round(px * 100) / 100, py: Math.round(py * 100) / 100, targetPx, targetPy, visibleBounds, floorMeshes: floorRoot ? floorRoot.children.length : 0, wallMeshes: wallRoot ? wallRoot.children.length : 0, actors: actorRoot ? actorRoot.children.length : 0, camera: camera ? { type: "OrthographicCamera", x: +camera.position.x.toFixed(2), y: +camera.position.y.toFixed(2), z: +camera.position.z.toFixed(2) } : null, dpr };
  }
  return { init, setData, setPlayer, keydown, keyup, start, stop, dispose, nearbyList, moveIntent, drawMinimap, drawBigmap, clearKeys, setDpr, setFlying, setDetectBonus, setWebMode, webZipStart, resize, debug };
})();
