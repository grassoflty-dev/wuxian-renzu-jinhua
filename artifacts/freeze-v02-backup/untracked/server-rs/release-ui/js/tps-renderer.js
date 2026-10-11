/*
 * 无限人族进化 · TpsRenderer (Babylon.js)
 *
 * 第三人称（TPS）渲染器。与 CampaignRenderer 保持完全相同的契约：
 *   window.TpsRenderer = {
 *     init(canvas, { onTarget, onHover }), setData(map), setEffect(effect),
 *     setPlayerState(player), resize(), dispose(), debug(), targetAt(x, y),
 *     setQuality(q)
 *   }
 *
 * 权威状态说明：位置与朝向由 Rust 侧下发，渲染器只做呈现。为了操控手感，
 * 本地会依据 WASD 做短时预测（预测权重 predict 由 1 衰减到 0）；只要 Rust
 * 下发新坐标，predict 立刻归零并直接采用权威坐标，因此不会出现"渲染位置
 * 与逻辑位置长期不符"。
 *
 * 操作：WASD / 方向键 相机相对移动，空格跳跃，按住鼠标右键（或中键）拖拽
 * 旋转视角，左键单击沿用 app.js 的朝向+攻击。
 */
(function installTpsRenderer(global) {
  "use strict";

  var B = null;                       // BABYLON 命名空间，在 init() 中解析
  var TAU = Math.PI * 2;
  var FLOOR_Y = 0;
  var GRAVITY = 21;
  var JUMP_SPEED = 7.2;
  var WALK_SPEED = 4.6;               // 本地预测速度（格/秒）
  var CAM_DISTANCE = 6;               // 角色后上方距离
  var CAM_HEIGHT = 3;                 // 相机相对角色头部的高度
  var CAM_LAG = 0.16;                 // 跟随阻尼
  var HEAD_HEIGHT = 1.5;
  var BASE_PITCH = 0.42;              // 基础俯角 ≈ 24°
  var PITCH_MIN = 0.05;               // 相机不允许穿地
  var PITCH_MAX = 1.25;
  var LOOK_SENSITIVITY = 0.0026;
  var PREDICT_DECAY = 1.6;
  var MAX_DPR = 2;

  var engine = null;
  var scene = null;
  var camera = null;
  var canvas = null;
  var initialized = false;
  var rafId = null;
  var quality = "high";
  var dpr = 1;
  var logicalWidth = 1;
  var logicalHeight = 1;
  var callbacks = { onTarget: null, onHover: null };
  var uninstallInput = null;
  var resizeObserver = null;

  var map = null;
  var tiles = [];
  var mapW = 0;
  var mapH = 0;
  var currentWorldId = "hub";
  var theme = null;
  var topologySignature = "";
  var topologyRoot = null;
  var propRoot = null;
  var actorRoot = null;
  var effectRoot = null;
  var groundPick = null;
  var shadowGenerator = null;
  var keyLight = null;
  var hemiLight = null;
  var playerLight = null;

  var player = null;
  var actorRecords = Object.create(null);
  var effects = [];
  var pendingEffects = [];
  var ownedMaterials = [];

  // --- 相机 / 移动状态 ---
  var camYaw = 0.62;
  var camMood = 0;
  var camTarget = { x: 0, y: HEAD_HEIGHT, z: 0 };
  var keyState = Object.create(null);
  var moveAxis = { x: 0, z: 0 };

  // 权威状态 / 本地预测
  var authority = { x: 0, z: 0, has: false };
  var localPos = { x: 0, z: 0 };
  var localVel = { x: 0, z: 0 };
  var predict = 0;
  var facingYaw = 0;
  var groundspeed = 0;
  var lastFrameMs = 0;
  var jumpOffset = 0;
  var jumpVelocity = 0;
  var airborne = false;
  var swing = 0;
  var lastTargetCell = "";
  var lastTargetValue = null;

  var THEMES = {
    hub: {
      id: "hub",
      background: 0x0b1118, fog: 0x17232c, fogNear: 14, fogFar: 54,
      floor: [0x4d585f, 0x515b61, 0x49545b],
      floorEdge: 0x26323a,
      wall: [0x27343c, 0x2d3a42, 0x222e36],
      water: 0x304f5a,
      trim: 0x98a9ad, accent: 0x73d0c4, warning: 0xe18a43, lamp: 0xf6c75e,
      player: 0x7ac8c5, playerAccent: 0xf1c976, enemy: 0xd05a5d, npc: 0x79b998,
    },
    grey_hive: {
      id: "grey_hive",
      background: 0x0b141d, fog: 0x18252e, fogNear: 14, fogFar: 52,
      floor: [0x4f5b62, 0x535e64, 0x4b575e, 0x566066],
      floorEdge: 0x2a373f,
      wall: [0x26333b, 0x2f3c44, 0x1f2a32],
      water: 0x2e4d56,
      trim: 0x9eafb0, accent: 0x80b6b8, warning: 0xe46e45, lamp: 0xf6c75e,
      player: 0x8fd0c9, playerAccent: 0xf1c976, enemy: 0xcf5f5a, npc: 0x7fb79a,
    },
    mist_harbor: {
      id: "mist_harbor",
      background: 0x0a1418, fog: 0x152428, fogNear: 12, fogFar: 46,
      floor: [0x46575c, 0x4b5c60, 0x415256],
      floorEdge: 0x23343a,
      wall: [0x22333a, 0x2b3d43, 0x1b2b31],
      water: 0x27505c,
      trim: 0x9fc0bd, accent: 0x6ec6b6, warning: 0xe0a34f, lamp: 0xf4d98a,
      player: 0x86cbc4, playerAccent: 0xf1c976, enemy: 0xc75c5c, npc: 0x84b79b,
    },
  };

  var FALLBACK_THEME = {
    id: "campaign",
    background: 0x0a121a, fog: 0x16222c, fogNear: 14, fogFar: 52,
    floor: [0x4b565e, 0x515c62, 0x47525a],
    floorEdge: 0x28353c,
    wall: [0x25323a, 0x2e3b42, 0x1f2b32],
    water: 0x2c4b55,
    trim: 0x9aabae, accent: 0x79c9bd, warning: 0xe08a45, lamp: 0xf5c95f,
    player: 0x83cdc6, playerAccent: 0xf1c976, enemy: 0xd05a5d, npc: 0x79b998,
  };

  // ---------------------------------------------------------------- 基础工具

  function num(value, fallback) {
    var parsed = typeof value === "number" ? value : parseFloat(value);
    return isFinite(parsed) ? parsed : fallback;
  }

  function clamp(value, min, max) {
    return value < min ? min : value > max ? max : value;
  }

  function textOr(value, fallback) {
    return typeof value === "string" && value ? value : fallback;
  }

  function isArray(value) {
    return Object.prototype.toString.call(value) === "[object Array]";
  }

  function performanceNow() {
    return global.performance && global.performance.now ? global.performance.now() : Date.now();
  }

  function color3(hex) {
    return new B.Color3(((hex >> 16) & 255) / 255, ((hex >> 8) & 255) / 255, (hex & 255) / 255);
  }

  function shade(hex, factor) {
    var r = clamp(Math.round(((hex >> 16) & 255) * factor), 0, 255);
    var g = clamp(Math.round(((hex >> 8) & 255) * factor), 0, 255);
    var b = clamp(Math.round((hex & 255) * factor), 0, 255);
    return (r << 16) | (g << 8) | b;
  }

  function worldTheme(worldId) {
    if (THEMES[worldId]) return THEMES[worldId];
    return Object.assign({}, FALLBACK_THEME, { id: textOr(worldId, "campaign") });
  }

  function material(name, hex, options) {
    var opts = options || {};
    var mat = new B.StandardMaterial("shared:" + name, scene);
    mat.diffuseColor = color3(hex);
    mat.specularColor = new B.Color3(0.06, 0.07, 0.08);
    if (opts.emissive != null) mat.emissiveColor = color3(opts.emissive).scale(num(opts.emissiveIntensity, 0.22));
    if (opts.alpha != null && opts.alpha < 1) mat.alpha = opts.alpha;
    if (opts.backFaceCulling === false) mat.backFaceCulling = false;
    ownedMaterials.push(mat);
    return mat;
  }

  function effectMaterial(hex, alpha) {
    var mat = new B.StandardMaterial("effect:" + hex, scene);
    mat.diffuseColor = color3(hex);
    mat.emissiveColor = color3(hex);
    mat.specularColor = new B.Color3(0, 0, 0);
    mat.disableLighting = true;
    mat.alpha = clamp(num(alpha, 0.85), 0.05, 1);
    mat.backFaceCulling = false;
    ownedMaterials.push(mat);
    return mat;
  }

  // ---------------------------------------------------------------- 地图解析

  function rowString(row) {
    if (typeof row === "string") return row;
    if (isArray(row)) return row.join("");
    return "";
  }

  function normalizeMap(input) {
    var source = input && input.map && isArray(input.map.tiles) ? input.map : input;
    if (!source || !isArray(source.tiles)) return null;
    var rows = source.tiles.map(rowString);
    var inferredW = rows.reduce(function (longest, row) { return Math.max(longest, row.length); }, 0);
    var w = Math.max(0, Math.round(num(source.w, inferredW)));
    var h = Math.max(0, Math.round(num(source.h, rows.length)));
    if (!w || !h) return null;
    return {
      world_id: textOr(source.world_id || source.worldId || source.id, "hub"),
      w: w,
      h: h,
      floor: num(source.floor, 0),
      tiles: rows,
      px: num(source.px, 0),
      py: num(source.py, 0),
      revision: source.revision,
      explored: isArray(source.explored) ? source.explored : [],
      enemies: isArray(source.enemies) ? source.enemies : [],
      npcs: isArray(source.npcs) ? source.npcs : [],
      points: isArray(source.points) ? source.points : [],
      zones: isArray(source.zones) ? source.zones : [],
      portals: isArray(source.portals) ? source.portals : [],
      gates: isArray(source.gates) ? source.gates : [],
      props: isArray(source.props) ? source.props : [],
    };
  }

  function tileAt(x, y) {
    if (!map || x < 0 || y < 0 || x >= mapW || y >= mapH) return "#";
    var row = tiles[y] || "";
    return row.charAt(x) || "#";
  }

  function tileSignature(nextMap) {
    var text = nextMap.world_id + ":" + nextMap.w + "x" + nextMap.h + ":" + nextMap.tiles.join("\u001f");
    var hash = 2166136261;
    for (var i = 0; i < text.length; i += 1) {
      hash ^= text.charCodeAt(i);
      hash = Math.imul(hash, 16777619);
    }
    return nextMap.world_id + ":" + nextMap.w + "x" + nextMap.h + ":" + (hash >>> 0).toString(16);
  }

  function floorVariantForCell(x, y) {
    var count = Math.max(1, theme && theme.floor ? theme.floor.length : 1);
    return Math.abs((Math.floor(x / 4) + Math.floor(y / 3) * 2) % count);
  }

  // ---------------------------------------------------------------- 场景构建

  function disposeNode(node) {
    if (!node || !node.dispose) return;
    try { node.dispose(false, false); } catch (error) { /* 释放失败不影响后续 */ }
  }

  function clearRoot(root) {
    if (!root || !root.getChildMeshes) return;
    root.getChildMeshes().slice().forEach(disposeNode);
  }

  function disposeRoot(root) {
    if (!root) return;
    clearRoot(root);
    disposeNode(root);
  }

  function writeInstance(target, matrix, index) {
    if (typeof index === "number" && target.setMatrixAt) { target.setMatrixAt(index, matrix); return; }
    if (target && target.position) {
      var scale = new B.Vector3(), rotation = new B.Quaternion(), position = new B.Vector3();
      matrix.decompose(scale, rotation, position);
      target.position.copyFrom(position);
      target.scaling.copyFrom(scale);
    }
  }

  function buildMerged(name, count, mat, fill, options) {
    if (count <= 0) return null;
    var opts = options || {};
    var source = B.MeshBuilder.CreateBox(name + "-src", { size: 1 }, scene);
    var parts = new Array(count);
    for (var i = 0; i < count; i += 1) {
      var instance = source.createInstance(name + "-i" + i);
      writeInstance(instance, fill(i), i);
      parts[i] = instance;
    }
    var merged = null;
    try {
      merged = B.Mesh.MergeMeshes(parts, true, true, undefined, false, false);
    } catch (error) {
      merged = null;
    }
    if (!merged) {
      // 合并失败时保留第一块实例作为兜底（几何体多但画面正确）。
      merged = B.MeshBuilder.CreateBox(name + "-fallback", { size: 1 }, scene);
      writeInstance(merged, fill(0), -1);
    }
    merged.name = name;
    merged.material = mat;
    merged.isPickable = opts.pickable !== false;
    if (opts.receiveShadows) merged.receiveShadows = true;
    if (opts.castShadows && shadowGenerator) shadowGenerator.addShadowCaster(merged);
    source.dispose();
    return merged;
  }

  function boxMatrix(x, y, z, sx, sy, sz) {
    return B.Matrix.Compose(new B.Vector3(sx, sy, sz), B.Quaternion.Identity(), new B.Vector3(x, y, z));
  }

  function buildTopology() {
    disposeRoot(topologyRoot);
    topologyRoot = new B.TransformNode("tps-topology", scene);
    if (shadowGenerator) shadowGenerator.getShadowMap().renderList = [];
    if (!map) return;

    var floors = [], water = [], walls = [];
    for (var y = 0; y < mapH; y += 1) {
      for (var x = 0; x < mapW; x += 1) {
        var tile = tileAt(x, y);
        if (tile === "#") { walls.push({ x: x, y: y, variant: Math.abs((x + y * 3) % 3) }); continue; }
        if (tile === "~") { water.push({ x: x, y: y }); continue; }
        floors.push({ x: x, y: y, variant: floorVariantForCell(x, y) });
      }
    }
    if (!floors.length) floors.push({ x: Math.round(num(map.px, 0)), y: Math.round(num(map.py, 0)), variant: 0 });

    var buckets = Object.create(null);
    floors.forEach(function (cell) {
      var key = "f" + cell.variant;
      (buckets[key] = buckets[key] || []).push(cell);
    });
    Object.keys(buckets).forEach(function (key) {
      var cells = buckets[key];
      var variant = parseInt(key.slice(1), 10) || 0;
      var mat = material("floor-" + variant, theme.floor[Math.abs(variant) % theme.floor.length], {});
      var mesh = buildMerged("tps-floor-" + key, cells.length, mat, function (index) {
        var cell = cells[index];
        return boxMatrix(cell.x, -0.06, cell.y, 0.995, 0.12, 0.995);
      }, { receiveShadows: true, castShadows: true });
      if (mesh) mesh.parent = topologyRoot;
    });

    if (water.length) {
      var waterMat = material("water", theme.water, { alpha: 0.92, emissive: theme.accent, emissiveIntensity: 0.07 });
      var waterMesh = buildMerged("tps-water", water.length, waterMat, function (index) {
        var cell = water[index];
        return boxMatrix(cell.x, -0.16, cell.y, 0.995, 0.06, 0.995);
      }, {});
      if (waterMesh) waterMesh.parent = topologyRoot;
    }

    var wallBuckets = Object.create(null);
    walls.forEach(function (cell) {
      var key = "w" + cell.variant;
      (wallBuckets[key] = wallBuckets[key] || []).push(cell);
    });
    Object.keys(wallBuckets).forEach(function (key) {
      var cells = wallBuckets[key];
      var variant = parseInt(key.slice(1), 10) || 0;
      var mat = material("wall-" + variant, theme.wall[Math.abs(variant) % theme.wall.length], {});
      // 墙高 1.35：第三人称下能提供真实的遮挡体量。
      var mesh = buildMerged("tps-wall-" + key, cells.length, mat, function (index) {
        var cell = cells[index];
        return boxMatrix(cell.x, 0.675, cell.y, 0.98, 1.35, 0.98);
      }, { receiveShadows: true, castShadows: true });
      if (mesh) mesh.parent = topologyRoot;
    });

    // 不可见拾取地面：鼠标 aim 与默认交互直接用网格坐标。
    if (groundPick) disposeNode(groundPick);
    groundPick = B.MeshBuilder.CreateGround("tps-ground-pick", { width: Math.max(8, mapW + 8), height: Math.max(8, mapH + 8) }, scene);
    groundPick.position.set((mapW - 1) / 2, 0.02, (mapH - 1) / 2);
    groundPick.isVisible = false;
    groundPick.isPickable = true;
    groundPick.parent = topologyRoot;
  }

  function buildDecorations() {
    disposeRoot(propRoot);
    propRoot = new B.TransformNode("tps-props", scene);
    if (!map) return;

    var portalMat = material("marker-portal", theme.accent, { emissive: theme.accent, emissiveIntensity: 0.8 });
    var gateMat = material("marker-gate", theme.warning, { emissive: theme.warning, emissiveIntensity: 0.65 });
    var pointMat = material("marker-point", theme.lamp, { emissive: theme.lamp, emissiveIntensity: 0.55 });
    var propMat = material("prop", theme.floorEdge, { emissive: theme.trim, emissiveIntensity: 0.06 });
    var zoneMat = effectMaterial(theme.accent, 0.15);

    map.portals.forEach(function (portal) { addMarker(portal, "portal", portalMat); });
    map.gates.forEach(function (gate) { addMarker(gate, "gate", gateMat); });
    map.points.forEach(function (point) { addMarker(point, "point", pointMat); });

    map.props.forEach(function (prop) {
      var x = num(prop.x, NaN), y = num(prop.y, NaN);
      if (!isFinite(x) || !isFinite(y)) return;
      var w = clamp(num(prop.w, 1), 0.4, 4);
      var d = clamp(num(prop.h, 1), 0.4, 4);
      var height = clamp(num(prop.height, 1.1), 0.3, 4);
      var box = B.MeshBuilder.CreateBox("tps-prop", { width: w * 0.9, height: height, depth: d * 0.9 }, scene);
      box.position.set(x, height * 0.5, y);
      box.material = propMat;
      box.parent = propRoot;
      box.receiveShadows = true;
      if (shadowGenerator) shadowGenerator.addShadowCaster(box);
    });

    map.zones.forEach(function (zone) {
      var x = num(zone.x, NaN), y = num(zone.y, NaN);
      if (!isFinite(x) || !isFinite(y)) return;
      var disc = B.MeshBuilder.CreateDisc("tps-zone", { radius: clamp(num(zone.r, 2), 0.6, 12), tessellation: 28 }, scene);
      disc.rotation.x = Math.PI / 2;
      disc.position.set(x, 0.07, y);
      disc.material = zoneMat;
      disc.isPickable = false;
      disc.parent = propRoot;
    });
  }

  function addMarker(data, kind, mat) {
    var x = num(data.x, NaN), y = num(data.y, NaN);
    if (!isFinite(x) || !isFinite(y)) return;
    var height = kind === "portal" ? 2 : kind === "gate" ? 1.3 : 0.9;
    var post = B.MeshBuilder.CreateCylinder("tps-marker-" + kind, { height: height, diameter: kind === "portal" ? 0.42 : 0.24, tessellation: 10 }, scene);
    post.position.set(x, height * 0.5, y);
    post.material = mat;
    post.isPickable = true;
    post.parent = propRoot;
    if (shadowGenerator) shadowGenerator.addShadowCaster(post);
  }

  // ---------------------------------------------------------------- 角色

  function buildHumanoid(options) {
    var opts = options || {};
    var bodyHex = num(opts.body, 0x83cdc6);
    var accentHex = num(opts.accent, 0xf1c976);
    var scale = num(opts.scale, 1);
    var emissive = num(opts.emissive, 0.12);
    var bodyColor = material("actor:" + bodyHex + ":body", bodyHex, { emissive: accentHex, emissiveIntensity: emissive });
    var darkColor = material("actor:" + bodyHex + ":dark", shade(bodyHex, 0.6), {});
    var accentColor = material("actor:" + bodyHex + ":accent", accentHex, { emissive: accentHex, emissiveIntensity: 0.4 });

    var root = new B.TransformNode("tps-actor", scene);
    var body = new B.TransformNode("body", scene);
    body.parent = root;

    var hipY = -0.82;

    // 躯干 + 髋部 + 胸甲
    var torso = B.MeshBuilder.CreateBox("torso", { width: 0.5, height: 0.6, depth: 0.3 }, scene);
    torso.position.set(0, hipY + 0.62, 0); torso.material = bodyColor; torso.parent = body;

    var hips = B.MeshBuilder.CreateBox("hips", { width: 0.44, height: 0.22, depth: 0.28 }, scene);
    hips.position.set(0, hipY + 0.24, 0); hips.material = darkColor; hips.parent = body;

    var chest = B.MeshBuilder.CreateBox("chest-plate", { width: 0.24, height: 0.16, depth: 0.06 }, scene);
    chest.position.set(0, hipY + 0.7, 0.16); chest.material = accentColor; chest.parent = body;

    var neck = B.MeshBuilder.CreateCylinder("neck", { height: 0.12, diameter: 0.16, tessellation: 8 }, scene);
    neck.position.set(0, hipY + 0.96, 0); neck.material = darkColor; neck.parent = body;

    // 头 + 目镜（目镜朝 +Z，与 rotation.y = atan2(dx, dz) 的前向一致）
    var head = B.MeshBuilder.CreateBox("head", { width: 0.34, height: 0.34, depth: 0.34 }, scene);
    head.position.set(0, hipY + 1.2, 0); head.material = bodyColor; head.parent = body;

    var visor = B.MeshBuilder.CreateBox("visor", { width: 0.28, height: 0.11, depth: 0.05 }, scene);
    visor.position.set(0, hipY + 1.22, 0.17); visor.material = accentColor; visor.parent = body;

    function limb(name, offsetX, length, diameter, mat, pivotY) {
      var pivot = new B.TransformNode(name + "-pivot", scene);
      pivot.parent = body;
      pivot.position.set(offsetX, pivotY, 0);
      var mesh = B.MeshBuilder.CreateCylinder(name, { height: length, diameter: diameter, tessellation: 8 }, scene);
      mesh.position.set(0, -length * 0.5, 0);
      mesh.material = mat;
      mesh.parent = pivot;
      return pivot;
    }

    var armL = limb("arm-l", -0.34, 0.56, 0.15, bodyColor, hipY + 0.84);
    var armR = limb("arm-r", 0.34, 0.56, 0.15, bodyColor, hipY + 0.84);
    var legL = limb("leg-l", -0.14, 0.72, 0.18, darkColor, hipY + 0.18);
    var legR = limb("leg-r", 0.14, 0.72, 0.18, darkColor, hipY + 0.18);

    [legL, legR].forEach(function (leg, index) {
      var foot = B.MeshBuilder.CreateBox("foot-" + index, { width: 0.2, height: 0.1, depth: 0.3 }, scene);
      foot.position.set(0, -0.72, 0.05);
      foot.material = darkColor;
      foot.parent = leg;
    });

    root.getChildMeshes().forEach(function (mesh) {
      mesh.isPickable = false;
      if (shadowGenerator) shadowGenerator.addShadowCaster(mesh);
    });
    root.scaling.set(scale, scale, scale);

    return {
      root: root,
      body: body,
      parts: { torso: torso, head: head, armL: armL, armR: armR, legL: legL, legR: legR, visor: visor },
    };
  }

  function ensurePlayer() {
    if (player) return player;
    player = buildHumanoid({ body: theme.player, accent: theme.playerAccent, scale: 1, emissive: 0.14 });
    player.phase = 0;
    player.root.name = "tps-player";
    var startX = authority.has ? authority.x : (map ? num(map.px, 0) : 0);
    var startZ = authority.has ? authority.z : (map ? num(map.py, 0) : 0);
    localPos.x = startX;
    localPos.z = startZ;
    player.root.position.set(startX, FLOOR_Y, startZ);
    return player;
  }

  function syncActors() {
    disposeRoot(actorRoot);
    actorRoot = new B.TransformNode("tps-actors", scene);
    actorRecords = Object.create(null);
    if (!map) return;
    map.enemies.forEach(function (enemy) {
      if (enemy && enemy.alive === false) return;
      addActor("enemy:" + textOr(enemy && (enemy.id != null ? String(enemy.id) : ""), "?"), enemy, theme.enemy, 0xf0b45a, 0.96);
    });
    map.npcs.forEach(function (npc) {
      addActor("npc:" + textOr(npc && (npc.id != null ? String(npc.id) : ""), "?"), npc, theme.npc, 0xdff0e4, 0.94);
    });
  }

  function addActor(key, data, bodyHex, accentHex, scale) {
    var x = num(data.x, NaN), y = num(data.y, NaN);
    if (!isFinite(x) || !isFinite(y)) return;
    var built = buildHumanoid({ body: bodyHex, accent: accentHex, scale: scale, emissive: 0.2 });
    built.root.parent = actorRoot;
    built.root.position.set(x, FLOOR_Y, y);
    built.root.rotation.y = num(data.heading, 0);
    actorRecords[key] = { node: built, phase: Math.random() * TAU };
  }

  // ---------------------------------------------------------------- 相机

  function cameraPitch() {
    return clamp(BASE_PITCH + camMood, PITCH_MIN, PITCH_MAX);
  }

  function resolveCameraBlocking(target, desired) {
    // 不允许相机穿进地面或落到地图之外：必要时沿视线向角色拉近。
    var dx = desired.x - target.x, dy = desired.y - target.y, dz = desired.z - target.z;
    var t = 1;
    for (var step = 0; step < 10; step += 1) {
      var px = target.x + dx * t, py = target.y + dy * t, pz = target.z + dz * t;
      var blocked = py < FLOOR_Y + 0.45;
      if (map) {
        var cellX = Math.round(px), cellY = Math.round(pz);
        if (cellX < -2 || cellY < -2 || cellX > mapW + 1 || cellY > mapH + 1) blocked = true;
        else if (py < 1.35 && tileAt(cellX, cellY) === "#") blocked = true;
      }
      if (!blocked) break;
      t -= 0.09;
      if (t <= 0.3) { t = 0.3; break; }
    }
    return {
      x: target.x + dx * t,
      y: Math.max(FLOOR_Y + 0.45, target.y + dy * t),
      z: target.z + dz * t,
      distance: Math.sqrt(dx * dx + dy * dy + dz * dz) * t,
    };
  }

  function updateCamera(dt) {
    var focus = player
      ? { x: player.root.position.x, y: player.root.position.y + HEAD_HEIGHT, z: player.root.position.z }
      : { x: 0, y: HEAD_HEIGHT, z: 0 };
    var follow = clamp(1 - Math.pow(1 - CAM_LAG, dt * 60), 0, 1);
    camTarget.x += (focus.x - camTarget.x) * follow;
    camTarget.y += (focus.y - camTarget.y) * follow * 0.85;
    camTarget.z += (focus.z - camTarget.z) * follow;

    var pitch = cameraPitch();
    var horizontal = CAM_DISTANCE * Math.cos(pitch);
    var desired = {
      x: camTarget.x + Math.sin(camYaw) * horizontal,
      y: camTarget.y + CAM_HEIGHT * Math.sin(pitch) / Math.sin(BASE_PITCH),
      z: camTarget.z + Math.cos(camYaw) * horizontal,
    };
    var resolved = resolveCameraBlocking(camTarget, desired);
    camera.position.set(resolved.x, resolved.y, resolved.z);
    camera.setTarget(new B.Vector3(camTarget.x, camTarget.y - 0.18, camTarget.z));
    if (playerLight) playerLight.position.set(camTarget.x, camTarget.y + 0.6, camTarget.z);
    return resolved;
  }

  // ---------------------------------------------------------------- 移动 / 动画

  function readMoveAxis() {
    var forward = ((keyState.KeyW || keyState.ArrowUp) ? 1 : 0) - ((keyState.KeyS || keyState.ArrowDown) ? 1 : 0);
    var strafe = ((keyState.KeyD || keyState.ArrowRight) ? 1 : 0) - ((keyState.KeyA || keyState.ArrowLeft) ? 1 : 0);
    // 相机相对：W 永远朝相机正前方走。
    var sin = Math.sin(camYaw), cos = Math.cos(camYaw);
    var x = -sin * forward + cos * strafe;
    var z = -cos * forward - sin * strafe;
    var length = Math.sqrt(x * x + z * z);
    if (length > 0.0001) { x /= length; z /= length; }
    moveAxis.x = x;
    moveAxis.z = z;
    return length > 0.0001;
  }

  function approachAngle(current, target, step) {
    var delta = ((target - current + Math.PI) % TAU + TAU) % TAU - Math.PI;
    if (Math.abs(delta) <= step) return target;
    return current + (delta > 0 ? step : -step);
  }

  function approach(current, target, step) {
    if (Math.abs(target - current) <= step) return target;
    return current + (target > current ? step : -step);
  }

  function updateMovement(dt) {
    var moving = readMoveAxis();
    if (moving) {
      localVel.x = moveAxis.x * WALK_SPEED;
      localVel.z = moveAxis.z * WALK_SPEED;
      predict = 1;
      facingYaw = approachAngle(facingYaw, Math.atan2(moveAxis.x, moveAxis.z), dt * 9);
      localPos.x += localVel.x * dt;
      localPos.z += localVel.z * dt;
      groundspeed = WALK_SPEED;
    } else {
      localVel.x = 0;
      localVel.z = 0;
      groundspeed = 0;
      // 松手后预测权重平滑归零，位置收敛回 Rust 下发的权威坐标。
      predict = Math.max(0, predict - (PREDICT_DECAY + PREDICT_DECAY) * dt);
      if (predict <= 0 && authority.has) {
        localPos.x = authority.x;
        localPos.z = authority.z;
      }
    }

    if (airborne) {
      jumpVelocity -= GRAVITY * dt;
      jumpOffset += jumpVelocity * dt;
      if (jumpOffset <= 0) { jumpOffset = 0; jumpVelocity = 0; airborne = false; }
    }
  }

  function updatePlayerVisual(dt) {
    if (!player) return;
    var baseX = authority.has ? authority.x : localPos.x;
    var baseZ = authority.has ? authority.z : localPos.z;
    var x = baseX + predict * (localPos.x - baseX);
    var z = baseZ + predict * (localPos.z - baseZ);
    player.root.position.set(x, FLOOR_Y + jumpOffset, z);
    player.root.rotation.y = facingYaw;

    var parts = player.parts;
    var walking = groundspeed > 0.35 && !airborne;
    if (walking) player.phase += dt * (5.4 + WALK_SPEED * 0.55);

    if (airborne) {
      parts.legL.rotation.x = approach(parts.legL.rotation.x, -0.55, dt * 6);
      parts.legR.rotation.x = approach(parts.legR.rotation.x, 0.3, dt * 6);
      parts.armL.rotation.x = approach(parts.armL.rotation.x, -1.05, dt * 6);
      parts.armR.rotation.x = approach(parts.armR.rotation.x, -1.05, dt * 6);
      player.body.position.y = 0;
    } else if (walking) {
      var step = Math.sin(player.phase) * 0.62;
      parts.legL.rotation.x = step;
      parts.legR.rotation.x = -step;
      parts.armL.rotation.x = -step * 0.85 + 0.06;
      parts.armR.rotation.x = step * 0.85 + 0.06;
      player.body.position.y = Math.abs(Math.sin(player.phase)) * 0.035;
    } else {
      parts.legL.rotation.x = approach(parts.legL.rotation.x, 0, dt * 6);
      parts.legR.rotation.x = approach(parts.legR.rotation.x, 0, dt * 6);
      var idle = Math.sin(performanceNow() / 900) * 0.055;
      parts.armL.rotation.x = approach(parts.armL.rotation.x, idle, dt * 6);
      parts.armR.rotation.x = approach(parts.armR.rotation.x, -idle, dt * 6);
      player.body.position.y = approach(player.body.position.y, 0, dt * 0.6);
    }

    if (swing > 0) {
      swing = Math.max(0, swing - dt * 2.4);
      parts.armR.rotation.x = -Math.sin((1 - swing / 0.3) * Math.PI) * 1.5;
    }
  }

  function updateActors(dt) {
    var keys = Object.keys(actorRecords);
    for (var i = 0; i < keys.length; i += 1) {
      var record = actorRecords[keys[i]];
      if (!record || !record.node) continue;
      record.phase += dt * 1.4;
      record.node.body.position.y = Math.sin(record.phase) * 0.05;
      record.node.parts.armL.rotation.x = Math.sin(record.phase) * 0.12;
      record.node.parts.armR.rotation.x = -Math.sin(record.phase) * 0.12;
    }
  }

  // ---------------------------------------------------------------- 特效

  function supportedEffectKind(kind) {
    var value = textOr(kind, "").toLowerCase();
    return ["attack", "dash", "hit", "impact", "heal", "skill", "death", "muzzle"].indexOf(value) >= 0 ? value : null;
  }

  function flushPendingEffects() {
    if (!pendingEffects.length) return;
    var queue = pendingEffects;
    pendingEffects = [];
    queue.forEach(spawnEffect);
  }

  function spawnEffect(effect) {
    if (!effectRoot || !player) return;
    var actor = effect.actor || effect.target || null;
    var x = player.root.position.x, z = player.root.position.z;
    if (actor && isFinite(num(actor.x, NaN))) {
      x = num(actor.x, x);
      z = num(actor.y, z);
    }
    var color = effect.color != null ? num(effect.color, theme.accent) : theme.accent;
    if (effect.kind === "hit" || effect.kind === "impact" || effect.kind === "death") color = theme.warning;
    if (effect.kind === "heal") color = 0x7de3a8;

    var duration = effect.kind === "dash" ? 0.42 : 0.3;
    var mesh;
    if (effect.kind === "dash") {
      mesh = B.MeshBuilder.CreateBox("fx-dash", { width: 0.5, height: 1.2, depth: 1.4 }, scene);
      mesh.position.set(x, 0.7, z);
      mesh.rotation.y = facingYaw;
    } else {
      mesh = B.MeshBuilder.CreateTorus("fx-" + effect.kind, { diameter: 1.7, thickness: 0.14, tessellation: 20 }, scene);
      mesh.position.set(x, 0.9, z);
      mesh.rotation.x = Math.PI / 2;
    }
    mesh.material = effectMaterial(color, 0.9);
    mesh.isPickable = false;
    mesh.parent = effectRoot;
    effects.push({ node: mesh, age: 0, duration: duration });
  }

  function updateEffects(dt) {
    if (!effects.length) return;
    var keep = [];
    effects.forEach(function (effect) {
      effect.age += dt;
      var t = effect.age / effect.duration;
      if (t >= 1) { disposeNode(effect.node); return; }
      var scale = 0.45 + t * 1.5;
      effect.node.scaling.set(scale, scale, scale);
      if (effect.node.material) effect.node.material.alpha = 0.9 * (1 - t);
      keep.push(effect);
    });
    effects = keep;
  }

  // ---------------------------------------------------------------- 输入

  function applyLook(dx, dy) {
    camYaw -= dx * LOOK_SENSITIVITY;
    camMood = clamp(camMood + dy * LOOK_SENSITIVITY * 0.9, PITCH_MIN - BASE_PITCH, PITCH_MAX - BASE_PITCH);
  }

  function installInput() {
    if (!canvas || !global.addEventListener) return;
    var dragging = false;
    var lastX = 0, lastY = 0;

    function onKeyDown(event) {
      var code = event.code || "";
      if (code === "Space") {
        if (!airborne) { airborne = true; jumpVelocity = JUMP_SPEED; }
        event.preventDefault();
        return;
      }
      keyState[code] = true;
      if (code.indexOf("Arrow") === 0) event.preventDefault();
    }

    function onKeyUp(event) { keyState[event.code || ""] = false; }

    function clearKeys() { keyState = Object.create(null); dragging = false; }

    function onContextMenu(event) { event.preventDefault(); }

    function onPointerDown(event) {
      if (event.button !== 2 && event.button !== 1) return;
      dragging = true;
      lastX = event.clientX;
      lastY = event.clientY;
      if (canvas.setPointerCapture) { try { canvas.setPointerCapture(event.pointerId); } catch (error) { /* 忽略 */ } }
      event.preventDefault();
    }

    function onPointerUp(event) {
      if (event.button === 2 || event.button === 1) dragging = false;
    }

    function onPointerMove(event) {
      if (dragging) {
        applyLook(event.clientX - lastX, event.clientY - lastY);
        lastX = event.clientX;
        lastY = event.clientY;
      }
      if (callbacks.onHover) callbacks.onHover(targetAt(event.clientX, event.clientY));
    }

    function onPointerLeave() {
      dragging = false;
      lastTargetCell = "";
      if (callbacks.onHover) callbacks.onHover(null);
    }

    function onCanvasClick(event) {
      if (callbacks.onTarget) callbacks.onTarget(targetAt(event.clientX, event.clientY));
    }

    canvas.addEventListener("keydown", onKeyDown);
    canvas.addEventListener("keyup", onKeyUp);
    global.addEventListener("keydown", onKeyDown);
    global.addEventListener("keyup", onKeyUp);
    global.addEventListener("blur", clearKeys);
    canvas.addEventListener("contextmenu", onContextMenu);
    canvas.addEventListener("pointerdown", onPointerDown);
    global.addEventListener("pointerup", onPointerUp);
    canvas.addEventListener("pointermove", onPointerMove);
    canvas.addEventListener("pointerleave", onPointerLeave);
    canvas.addEventListener("click", onCanvasClick);

    uninstallInput = function () {
      if (!canvas) return;
      canvas.removeEventListener("keydown", onKeyDown);
      canvas.removeEventListener("keyup", onKeyUp);
      global.removeEventListener("keydown", onKeyDown);
      global.removeEventListener("keyup", onKeyUp);
      global.removeEventListener("blur", clearKeys);
      canvas.removeEventListener("contextmenu", onContextMenu);
      canvas.removeEventListener("pointerdown", onPointerDown);
      global.removeEventListener("pointerup", onPointerUp);
      canvas.removeEventListener("pointermove", onPointerMove);
      canvas.removeEventListener("pointerleave", onPointerLeave);
      canvas.removeEventListener("click", onCanvasClick);
      uninstallInput = null;
    };
  }

  // ---------------------------------------------------------------- 渲染循环

  function frame() {
    if (!initialized || !engine) return;
    var now = performanceNow();
    var dt = lastFrameMs ? clamp((now - lastFrameMs) / 1000, 0.001, 0.08) : 1 / 60;
    lastFrameMs = now;
    if (map) {
      updateMovement(dt);
      updatePlayerVisual(dt);
      updateActors(dt);
      flushPendingEffects();
      updateEffects(dt);
      updateCamera(dt);
    }
    scene.render();
  }

  function startLoop() {
    if (!engine || rafId != null) return;
    lastFrameMs = 0;
    engine.runRenderLoop(frame);
    rafId = 1;
  }

  function stopLoop() {
    if (engine) { try { engine.stopRenderLoop(); } catch (error) { /* 忽略 */ } }
    rafId = null;
  }

  // ---------------------------------------------------------------- 尺寸

  function measure() {
    if (!canvas) return { width: 1, height: 1 };
    var parent = canvas.parentElement;
    var rect = parent && parent.getBoundingClientRect ? parent.getBoundingClientRect() : null;
    logicalWidth = Math.max(1, Math.round(num(rect && rect.width, canvas.clientWidth || 800)));
    logicalHeight = Math.max(1, Math.round(num(rect && rect.height, canvas.clientHeight || 450)));
    return { width: logicalWidth, height: logicalHeight };
  }

  function applyHardwareScaling() {
    if (!engine) return;
    var ratio = quality === "low" ? 1 : quality === "medium" ? 0.8 : Math.min(0.75, 1 / dpr);
    engine.setHardwareScalingLevel(clamp(ratio, 0.5, 1));
  }

  function resize() {
    if (!canvas) return;
    var size = measure();
    dpr = clamp(num(global.devicePixelRatio, 1), 0.5, MAX_DPR);
    if (canvas.style) {
      canvas.style.width = size.width + "px";
      canvas.style.height = size.height + "px";
    }
    if (!engine) return;
    applyHardwareScaling();
    engine.resize(true);
  }

  // ---------------------------------------------------------------- 主题 / 光照

  function applyTheme(worldId) {
    theme = worldTheme(worldId);
    currentWorldId = theme.id;
    if (scene) {
      scene.clearColor = B.Color4.FromColor3(color3(theme.background), 1);
      scene.fogMode = B.Scene.FOGMODE_LINEAR;
      scene.fogColor = color3(theme.fog);
      scene.fogStart = theme.fogNear;
      scene.fogEnd = theme.fogFar;
    }
    if (keyLight) keyLight.diffuse = color3(theme.trim);
    if (hemiLight) {
      hemiLight.diffuse = color3(theme.trim);
      hemiLight.groundColor = color3(theme.background);
    }
    if (playerLight) playerLight.diffuse = color3(theme.lamp);
  }

  function buildLights() {
    hemiLight = new B.HemisphericLight("tps-hemi", new B.Vector3(0.1, 1, 0.1), scene);
    hemiLight.intensity = 1.05;
    hemiLight.specular = new B.Color3(0.1, 0.1, 0.1);

    keyLight = new B.DirectionalLight("tps-key", new B.Vector3(-0.55, -1, -0.45), scene);
    keyLight.position = new B.Vector3(11, 20, 9);
    keyLight.intensity = 1.45;

    // 跟随角色的补光：第三人称下保证角色剪影始终从暗色地面里读得出来。
    playerLight = new B.PointLight("tps-player-light", new B.Vector3(0, 2.4, 0), scene);
    playerLight.intensity = 0.7;
    playerLight.range = 10;

    if (quality !== "low") {
      shadowGenerator = new B.ShadowGenerator(quality === "high" ? 1024 : 512, keyLight);
      shadowGenerator.useBlurExponentialShadowMap = true;
      shadowGenerator.blurKernel = 16;
      shadowGenerator.setDarkness(0.45);
    }
  }

  // ---------------------------------------------------------------- 契约实现

  function init(cv, options) {
    if (!cv) throw new Error("TpsRenderer.init 需要一个 canvas");
    B = global.BABYLON;
    if (!B || !B.Engine || !B.Scene) throw new Error("TpsRenderer 需要同级 vendor/babylon.js（未检测到 BABYLON）");
    if (!B.MeshBuilder || !B.StandardMaterial) throw new Error("BABYLON 版本过旧：缺少 MeshBuilder/StandardMaterial");

    if (initialized) dispose();

    canvas = cv;
    options = options || {};
    callbacks.onTarget = typeof options.onTarget === "function" ? options.onTarget : null;
    callbacks.onHover = typeof options.onHover === "function" ? options.onHover : null;
    if (options.quality === "low" || options.quality === "medium" || options.quality === "high") quality = options.quality;
    dpr = clamp(num(global.devicePixelRatio, 1), 0.5, MAX_DPR);

    engine = new B.Engine(canvas, quality !== "low", {
      preserveDrawingBuffer: true,
      stencil: true,
      doNotHandleContextLost: false,
    });
    engine.enableOfflineSupport = false;

    scene = new B.Scene(engine);
    scene.useRightHandedSystem = true;
    scene.clearColor = B.Color4.FromColor3(color3(0x0b1118), 1);
    if (B.ImageProcessingConfiguration) {
      scene.imageProcessingConfiguration.toneMappingEnabled = true;
      scene.imageProcessingConfiguration.toneMappingType = B.ImageProcessingConfiguration.TONEMAPPING_ACES;
      scene.imageProcessingConfiguration.exposure = quality === "low" ? 1.0 : 1.18;
      scene.imageProcessingConfiguration.contrast = 1.05;
    }

    camera = new B.FreeCamera("tps-camera", new B.Vector3(0, 4, -6), scene);
    camera.fov = 0.92;
    camera.minZ = 0.1;
    camera.maxZ = 220;
    camera.inputs.clear();          // 相机完全由 updateCamera 驱动
    scene.activeCamera = camera;

    topologyRoot = new B.TransformNode("tps-topology", scene);
    propRoot = new B.TransformNode("tps-props", scene);
    actorRoot = new B.TransformNode("tps-actors", scene);
    effectRoot = new B.TransformNode("tps-effects", scene);

    applyTheme(map ? map.world_id : "hub");
    buildLights();
    initialized = true;
    resize();
    installInput();

    if (typeof ResizeObserver === "function" && canvas.parentElement) {
      try {
        resizeObserver = new ResizeObserver(function () { resize(); });
        resizeObserver.observe(canvas.parentElement);
      } catch (error) { resizeObserver = null; }
    }
    if (global.addEventListener) global.addEventListener("resize", resize);

    if (map) {
      applyTheme(map.world_id);
      buildTopology();
      topologySignature = tileSignature(map);
      buildDecorations();
      ensurePlayer();
      syncActors();
    }
    startLoop();
    return api;
  }

  function setData(input) {
    var nextMap = normalizeMap(input);
    if (!nextMap) {
      disposeRoot(topologyRoot);
      topologyRoot = new B.TransformNode("tps-topology", scene);
      disposeRoot(propRoot);
      propRoot = new B.TransformNode("tps-props", scene);
      disposeRoot(actorRoot);
      actorRoot = new B.TransformNode("tps-actors", scene);
      if (player) { disposeNode(player.root); player = null; }
      authority.has = false;
      predict = 0;
      map = null;
      tiles = [];
      mapW = 0;
      mapH = 0;
      topologySignature = "";
      return;
    }

    var worldChanged = !map || map.world_id !== nextMap.world_id;
    map = nextMap;
    tiles = nextMap.tiles;
    mapW = nextMap.w;
    mapH = nextMap.h;
    if (worldChanged) {
      applyTheme(nextMap.world_id);
      if (player) { disposeNode(player.root); player = null; }
      authority.has = false;
      predict = 0;
      camTarget.x = nextMap.px;
      camTarget.z = nextMap.py;
    }
    var signature = tileSignature(nextMap);
    if (signature !== topologySignature) {
      topologySignature = signature;
      buildTopology();
    }
    buildDecorations();
    ensurePlayer();
    if (input && input.player) setPlayerState(input.player);
    if (!authority.has) {
      authority.x = num(nextMap.px, 0);
      authority.z = num(nextMap.py, 0);
      authority.has = true;
      localPos.x = authority.x;
      localPos.z = authority.z;
      predict = 0;
    }
    syncActors();
  }

  function setPlayerState(nextPlayer) {
    if (!nextPlayer || typeof nextPlayer !== "object") return;
    if (!player) ensurePlayer();
    var x = nextPlayer.x != null ? nextPlayer.x : nextPlayer.px;
    var y = nextPlayer.y != null ? nextPlayer.y : nextPlayer.py;
    if (x != null && y != null) {
      var nx = num(x, authority.has ? authority.x : num(map && map.px, 0));
      var nz = num(y, authority.has ? authority.z : num(map && map.py, 0));
      if (!authority.has || Math.abs(nx - authority.x) > 0.001 || Math.abs(nz - authority.z) > 0.001) {
        authority.x = nx;
        authority.z = nz;
        authority.has = true;
        localPos.x = nx;
        localPos.z = nz;
        // 权威坐标更新 => 本地预测立即归零，避免渲染位置与逻辑位置长期不符。
        predict = 0;
      }
    }
    var heading = nextPlayer.heading != null ? nextPlayer.heading : nextPlayer.facing;
    if (heading != null && isFinite(num(heading, NaN))) facingYaw = num(heading, facingYaw);
  }

  function setEffect(effect) {
    if (!effect || typeof effect !== "object") return;
    var kind = supportedEffectKind(effect.kind);
    if (!kind) return;
    if ((kind === "attack" || kind === "dash") && player) swing = 0.3;
    pendingEffects.push(Object.assign({}, effect, { kind: kind }));
    if (pendingEffects.length > 24) pendingEffects.shift();
  }

  function setQuality(nextQuality) {
    if (typeof nextQuality === "number") nextQuality = nextQuality >= 0.85 ? "high" : nextQuality >= 0.55 ? "medium" : "low";
    var value = textOr(nextQuality, "").toLowerCase();
    quality = value === "low" || value === "medium" || value === "high" ? value : "high";
    if (engine) {
      applyHardwareScaling();
      resize();
    }
  }

  function targetAt(clientX, clientY) {
    if (!canvas || !scene || !initialized || !map) return null;
    var rect = canvas.getBoundingClientRect ? canvas.getBoundingClientRect() : { left: 0, top: 0 };
    var x = num(clientX, 0) - num(rect.left, 0);
    var y = num(clientY, 0) - num(rect.top, 0);
    var pick = scene.pick(x, y, function (mesh) {
      return !!mesh && mesh.isPickable && mesh.isEnabled() && mesh.name.indexOf("tps-ground-pick") !== 0;
    });
    var gx, gy;
    if (pick && pick.hit && pick.pickedPoint) {
      gx = Math.round(pick.pickedPoint.x);
      gy = Math.round(pick.pickedPoint.z);
    } else if (groundPick) {
      var groundHit = scene.pick(x, y, function (mesh) { return mesh === groundPick; });
      if (!groundHit || !groundHit.hit || !groundHit.pickedPoint) return null;
      gx = Math.round(groundHit.pickedPoint.x);
      gy = Math.round(groundHit.pickedPoint.z);
    } else {
      return null;
    }
    if (gx < 0 || gy < 0 || gx >= mapW || gy >= mapH) return null;
    var cell = gx + ":" + gy;
    if (cell !== lastTargetCell) {
      lastTargetCell = cell;
      lastTargetValue = { x: gx, y: gy };
    }
    return lastTargetValue;
  }

  function isBehindCamera() {
    if (!camera || !player) return null;
    var toCamera = new B.Vector3(camera.position.x - player.root.position.x, 0, camera.position.z - player.root.position.z);
    var forward = new B.Vector3(Math.sin(facingYaw), 0, Math.cos(facingYaw));
    if (toCamera.lengthSquared() < 0.000001) return null;
    return B.Vector3.Dot(B.Vector3.Normalize(toCamera), forward) < 0;
  }

  function debug() {
    var pitch = null, distance = null;
    if (camera && player) {
      var dx = camera.position.x - player.root.position.x;
      var dz = camera.position.z - player.root.position.z;
      var dy = camera.position.y - (player.root.position.y + HEAD_HEIGHT);
      distance = Math.sqrt(dx * dx + dy * dy + dz * dz);
      pitch = Math.atan2(-dy, Math.sqrt(dx * dx + dz * dz));
    }
    return {
      mode: "babylon-tps-perspective",
      webgl: !!engine,
      initialized: initialized,
      world_id: currentWorldId,
      theme: theme ? theme.id : null,
      map: map ? { w: mapW, h: mapH, revision: map.revision, floor: map.floor } : null,
      topologySignature: topologySignature,
      floorInstances: topologyRoot ? topologyRoot.getChildMeshes().length : 0,
      wallInstances: 0,
      actors: Object.keys(actorRecords).length,
      effects: effects.length + pendingEffects.length,
      viewport: { width: logicalWidth, height: logicalHeight, dpr: dpr },
      quality: quality,
      camera: camera ? {
        type: "PerspectiveCamera",
        mode: "third-person-follow",
        x: +camera.position.x.toFixed(2),
        y: +camera.position.y.toFixed(2),
        z: +camera.position.z.toFixed(2),
        yaw: +camYaw.toFixed(3),
        pitch_degrees: pitch == null ? null : +(pitch * 180 / Math.PI).toFixed(1),
        distance: distance == null ? null : +distance.toFixed(2),
        fov: camera.fov,
        target: { x: +camTarget.x.toFixed(2), y: +camTarget.y.toFixed(2), z: +camTarget.z.toFixed(2) },
      } : null,
      player: player ? {
        x: +player.root.position.x.toFixed(2),
        y: +player.root.position.y.toFixed(2),
        z: +player.root.position.z.toFixed(2),
        yaw: +facingYaw.toFixed(3),
        speed: +groundspeed.toFixed(2),
        airborne: airborne,
        predict: +predict.toFixed(2),
        parts: Object.keys(player.parts).length,
        cameraBehind: isBehindCamera(),
      } : null,
      input: {
        keys: Object.keys(keyState).filter(function (key) { return keyState[key]; }),
        move: { x: +moveAxis.x.toFixed(2), z: +moveAxis.z.toFixed(2) },
      },
      target: lastTargetCell || null,
    };
  }

  function dispose() {
    initialized = false;
    stopLoop();
    if (resizeObserver) { try { resizeObserver.disconnect(); } catch (error) { /* 忽略 */ } resizeObserver = null; }
    if (global.removeEventListener) global.removeEventListener("resize", resize);
    if (uninstallInput) uninstallInput();
    effects.forEach(function (effect) { disposeNode(effect.node); });
    effects = [];
    pendingEffects = [];
    if (scene) { try { scene.dispose(); } catch (error) { /* 忽略 */ } }
    if (engine) { try { engine.dispose(); } catch (error) { /* 忽略 */ } }
    scene = null;
    engine = null;
    camera = null;
    topologyRoot = null;
    propRoot = null;
    actorRoot = null;
    effectRoot = null;
    groundPick = null;
    shadowGenerator = null;
    keyLight = null;
    hemiLight = null;
    playerLight = null;
    player = null;
    actorRecords = Object.create(null);
    ownedMaterials = [];
    map = null;
    tiles = [];
    mapW = 0;
    mapH = 0;
    topologySignature = "";
    authority = { x: 0, z: 0, has: false };
    localPos = { x: 0, z: 0 };
    localVel = { x: 0, z: 0 };
    predict = 0;
    jumpOffset = 0;
    jumpVelocity = 0;
    airborne = false;
    swing = 0;
    keyState = Object.create(null);
    lastTargetCell = "";
    lastTargetValue = null;
    // 注意：这里刻意不清空 canvas —— 它是宿主页面共享的 DOM 节点，
    // 回退渲染器（Three.js）接下来还要在同一个 canvas 上初始化。
  }

  var api = {
    init: init,
    setData: setData,
    setEffect: setEffect,
    setPlayerState: setPlayerState,
    resize: resize,
    dispose: dispose,
    debug: debug,
    targetAt: targetAt,
    setQuality: setQuality,
  };

  global.TpsRenderer = api;
})(typeof window !== "undefined" ? window : this);
