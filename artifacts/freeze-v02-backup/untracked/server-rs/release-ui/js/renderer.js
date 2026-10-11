/*
 * 余界：归航协议 · CampaignRenderer
 *
 * One renderer for the hub and the three original campaign worlds.  The scene
 * is deliberately asset-free: every readable prop and actor is made from
 * shared low-poly geometry, so the release UI never reaches for an image,
 * a remote URL, or a legacy character pack.
 *
 * Public contract:
 *   window.CampaignRenderer = {
 *     init(canvas, { onTarget, onHover }), setData(map), setEffect(effect),
 *     setPlayerState(player), resize(), dispose(), debug(), targetAt(x, y),
 *     setQuality(q)
 *   }
 */
(function installCampaignRenderer(global) {
  "use strict";

  var THREE = global && global.THREE;
  var UNIT = 1;
  var VIEW_W = 15;
  var VIEW_H = 12;
  var MAX_DPR = 2;
  var TAU = Math.PI * 2;

  var canvas = null;
  var renderer = null;
  var scene = null;
  var camera = null;
  var raycaster = null;
  var pointer = null;
  var groundPlane = null;
  var fallback = false;
  var fallbackContext = null;
  var initialized = false;
  var rafId = null;
  var lastFrame = 0;
  var dpr = 1;
  var logicalWidth = 1;
  var logicalHeight = 1;
  var quality = "high";
  var callbacks = { onTarget: null, onHover: null };
  var hoverTarget = null;

  var map = null;
  var playerState = {};
  var exploredLookup = Object.create(null);
  var exploredAll = true;
  var currentWorldId = "hub";
  var theme = null;
  var topologyKey = "";
  var decorationKey = "";
  var staticPropKey = "";
  var dynamicDecorationKey = "";
  var topologyRebuilds = 0;
  var decorationRebuilds = 0;
  var topologyRoot = null;
  var decorationRoot = null;
  var staticPropRoot = null;
  var dynamicDecorationRoot = null;
  var actorRoot = null;
  var effectRoot = null;
  var baseMesh = null;
  var floorInstances = [];
  var wallInstances = [];
  var wallCells = [];
  var actorRecords = Object.create(null);
  var playerRecord = null;
  var effects = [];
  var dynamicMarkers = [];
  var hazardMarkers = [];
  var interactionMarkers = [];
  var decorProps = [];
  var fallbackMarkerData = [];
  var fallbackHazardData = [];
  var fallbackInteractionData = [];
  var pendingEffects = [];
  var ownedMaterials = [];
  var ownedGeometries = [];
  var sharedMaterials = Object.create(null);
  var sharedGeometries = Object.create(null);
  var lastCutawayCell = "";

  var fallbackView = {
    scale: 32,
    tileW: 23,
    tileH: 13,
  };

  var THEMES = {
    hub: {
      id: "hub",
      background: 0x0b1118,
      fog: 0x17232c,
      floor: [0x4d585f, 0x515b61, 0x49545b],
      floorEdge: 0x26323a,
      wall: [0x27343c, 0x303e45, 0x202b33],
      water: 0x304f5a,
      trim: 0x98a9ad,
      accent: 0x73d0c4,
      warning: 0xe18a43,
      lamp: 0xf6c75e,
      steam: 0xc9e7e3,
      player: 0x7ac8c5,
      enemy: 0xd05a5d,
      npc: 0x79b998,
      fogNear: 19,
      fogFar: 50,
    },
    grey_hive: {
      id: "grey_hive",
      background: 0x0b141d,
      fog: 0x18252e,
      floor: [0x4f5b62, 0x535e64, 0x4b575e, 0x566066],
      floorEdge: 0x2a373f,
      wall: [0x26333b, 0x303e45, 0x202b32, 0x3b4141],
      water: 0x2e4d56,
      trim: 0x9eafb0,
      accent: 0x80b6b8,
      warning: 0xe46e45,
      lamp: 0xf3c25d,
      steam: 0xd7d9c5,
      player: 0x75b7b8,
      enemy: 0xc8545e,
      npc: 0x78a984,
      fogNear: 17,
      fogFar: 42,
    },
    mist_harbor: {
      id: "mist_harbor",
      background: 0x0c1b21,
      fog: 0x193b3c,
      floor: [0x4c6263, 0x516869, 0x475b5d, 0x566c6b],
      floorEdge: 0x2e474b,
      wall: [0x30474a, 0x3b5555, 0x263e42, 0x536462],
      water: 0x24656b,
      trim: 0xb7d1cb,
      accent: 0x6bd1c2,
      warning: 0xd9a05f,
      lamp: 0xf1e2b2,
      steam: 0xe7f2eb,
      player: 0x70c8bd,
      enemy: 0xa9515d,
      npc: 0x82c59d,
      fogNear: 13,
      fogFar: 35,
    },
    clockworks: {
      id: "clockworks",
      background: 0x17100d,
      fog: 0x3a2821,
      floor: [0x68574b, 0x6d5b4f, 0x625047, 0x715d51],
      floorEdge: 0x3a2b23,
      wall: [0x4a372d, 0x5a4232, 0x382a24, 0x76523b],
      water: 0x31504d,
      trim: 0xc28a55,
      accent: 0xe0a05d,
      warning: 0xec7647,
      lamp: 0xffc36b,
      steam: 0xf0c49a,
      player: 0xd88b58,
      enemy: 0xbd5b49,
      npc: 0xb7a17a,
      fogNear: 15,
      fogFar: 40,
    },
  };

  function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
  }

  function num(value, fallbackValue) {
    return typeof value === "number" && isFinite(value) ? value : fallbackValue;
  }

  function hash(x, y, seed) {
    var a = (x | 0) * 374761393;
    var b = (y | 0) * 668265263;
    var c = (seed | 0) * 1442695041;
    var h = (a ^ b ^ c) | 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177);
    return (h ^ (h >>> 16)) >>> 0;
  }

  function colorToHex(value) {
    return (value >>> 0).toString(16).padStart(6, "0");
  }

  function safeString(value) {
    return value == null ? "" : String(value);
  }

  function worldTheme(worldId) {
    var id = safeString(worldId).toLowerCase();
    if (id === "grey_hive" || id.indexOf("grey") >= 0 || id.indexOf("hive") >= 0 || id.indexOf("灰巢") >= 0) return THEMES.grey_hive;
    if (id === "mist_harbor" || id.indexOf("mist") >= 0 || id.indexOf("harbor") >= 0 || id.indexOf("雾港") >= 0) return THEMES.mist_harbor;
    if (id === "clockworks" || id.indexOf("clock") >= 0 || id.indexOf("work") >= 0 || id.indexOf("钟骨") >= 0) return THEMES.clockworks;
    return THEMES.hub;
  }

  function mapWorldId(value) {
    if (!value) return "hub";
    return safeString(value.world_id || value.worldId || value.id || "hub");
  }

  function rowString(row) {
    if (typeof row === "string") return row;
    if (Array.isArray(row)) return row.join("");
    return "";
  }

  function normalizeMap(input) {
    var source = input && input.map && Array.isArray(input.map.tiles) ? input.map : input;
    if (!source || !Array.isArray(source.tiles)) return null;
    var rows = source.tiles.map(rowString);
    var inferredW = rows.reduce(function (longest, row) { return Math.max(longest, row.length); }, 0);
    var w = Math.max(0, Math.round(num(source.w, inferredW)));
    var h = Math.max(0, Math.round(num(source.h, rows.length)));
    if (!w || !h) return null;
    return {
      version: num(source.version, 2),
      world_id: mapWorldId(source),
      world_name: safeString(source.world_name || source.name || ""),
      w: w,
      h: h,
      floor: num(source.floor, 0),
      tiles: rows,
      px: num(source.px, 0),
      py: num(source.py, 0),
      revision: source.revision,
      explored: Array.isArray(source.explored) ? source.explored : [],
      enemies: Array.isArray(source.enemies) ? source.enemies : [],
      npcs: Array.isArray(source.npcs) ? source.npcs : [],
      points: Array.isArray(source.points) ? source.points : [],
      zones: Array.isArray(source.zones) ? source.zones : [],
      portals: Array.isArray(source.portals) ? source.portals : [],
      gates: Array.isArray(source.gates) ? source.gates : [],
      props: Array.isArray(source.props) ? source.props : [],
      nearby: Array.isArray(source.nearby) ? source.nearby : [],
      objectives: Array.isArray(source.objectives) ? source.objectives : [],
      telegraph: source.telegraph || null,
    };
  }

  function tileAt(x, y) {
    if (!map || x < 0 || y < 0 || x >= map.w || y >= map.h) return "#";
    var row = map.tiles[y] || "";
    return row[x] || "#";
  }

  function isWalkableTile(x, y) {
    var tile = tileAt(x, y);
    return tile !== "#" && tile !== "~";
  }

  function rebuildExploredLookup() {
    exploredLookup = Object.create(null);
    exploredAll = !map || !map.explored || map.explored.length === 0;
    if (exploredAll) return;
    map.explored.forEach(function (cell) { exploredLookup[String(cell)] = true; });
  }

  function isExploredCell(x, y) {
    return exploredAll || !!exploredLookup[x + ":" + y];
  }

  function tileSignature(nextMap) {
    if (!nextMap) return "";
    var text = safeString(nextMap.world_id) + ":" + nextMap.w + ":" + nextMap.h + ":" + nextMap.tiles.join("\u001f");
    var h = 2166136261;
    for (var i = 0; i < text.length; i += 1) {
      h ^= text.charCodeAt(i);
      h = Math.imul(h, 16777619);
    }
    return safeString(nextMap.world_id) + ":" + nextMap.w + "x" + nextMap.h + ":" + (h >>> 0).toString(16);
  }

  function listSignature(list) {
    try { return JSON.stringify(list || []); } catch (error) { return String(list || ""); }
  }

  function hasWebGL() {
    return !!(THREE && THREE.WebGLRenderer && THREE.Scene && THREE.OrthographicCamera);
  }

  function ownGeometry(geometry) {
    if (!geometry) return geometry;
    geometry.userData = geometry.userData || {};
    geometry.userData.sharedCampaign = true;
    ownedGeometries.push(geometry);
    return geometry;
  }

  function ownMaterial(material) {
    if (!material) return material;
    material.userData = material.userData || {};
    material.userData.sharedCampaign = true;
    ownedMaterials.push(material);
    return material;
  }

  function geometry(name, creator) {
    if (!THREE) return null;
    if (!sharedGeometries[name]) sharedGeometries[name] = ownGeometry(creator());
    return sharedGeometries[name];
  }

  function material(name, color, options) {
    if (!THREE) return null;
    var opts = options || {};
    var key = name + ":" + colorToHex(color) + ":" + (opts.basic ? "b" : opts.lambert ? "l" : "s") + ":" + (opts.transparent ? "t" : "o") + ":" + num(opts.opacity, 1) + ":" + num(opts.emissive, 0) + ":" + num(opts.emissiveIntensity, 0) + ":" + (opts.vertexColors ? "v" : "n");
    if (sharedMaterials[key]) return sharedMaterials[key];
    var params = { color: color, transparent: !!opts.transparent, opacity: num(opts.opacity, 1) };
    if (!opts.basic) {
      if (!opts.lambert) {
        params.roughness = num(opts.roughness, 0.78);
        params.metalness = num(opts.metalness, 0.12);
      }
      // Only opt into vertex colors when a material explicitly needs them.
      // Terrain uses InstancedMesh.instanceColor instead; its shared plate
      // geometry intentionally has no color attribute.
      if (opts.vertexColors) params.vertexColors = true;
      if (!opts.lambert) {
        if (opts.emissive != null) params.emissive = opts.emissive;
        if (opts.emissiveIntensity != null) params.emissiveIntensity = opts.emissiveIntensity;
      }
    }
    var created = opts.basic ? new THREE.MeshBasicMaterial(params) : opts.lambert && THREE.MeshLambertMaterial ? new THREE.MeshLambertMaterial(params) : new THREE.MeshStandardMaterial(params);
    sharedMaterials[key] = ownMaterial(created);
    return created;
  }

  function effectMaterial(color, opacity, line) {
    // Combat feedback must stay readable over the floor and actor silhouettes.
    // Effects are short-lived and deliberately small, so drawing them as an
    // overlay is preferable to losing them in terrain depth at game zoom.
    var params = { color: color, transparent: true, opacity: opacity, depthWrite: false, depthTest: false };
    var created = line ? new THREE.LineBasicMaterial(params) : new THREE.MeshBasicMaterial(params);
    created.userData = created.userData || {};
    created.userData.effectOwned = true;
    ownedMaterials.push(created);
    return created;
  }

  function boxGeometry() { return geometry("box", function () { return new THREE.BoxGeometry(1, 1, 1); }); }
  function floorPlateGeometry() { return geometry("floor-plate", function () { return new THREE.BoxGeometry(0.995, 0.12, 0.995); }); }
  function wallBlockGeometry() { return geometry("wall-block", function () { return new THREE.BoxGeometry(0.98, 0.76, 0.98); }); }
  function cylinderGeometry() { return geometry("cylinder", function () { return new THREE.CylinderGeometry(1, 1, 1, 12, 1, false); }); }
  function sphereGeometry() { return geometry("sphere", function () { return new THREE.SphereGeometry(1, 12, 8); }); }
  function ringGeometry() { return geometry("ring", function () { return new THREE.TorusGeometry(0.5, 0.055, 7, 28); }); }
  function capsuleGeometry() {
    return geometry("capsule", function () {
      if (THREE.CapsuleGeometry) return new THREE.CapsuleGeometry(1, 1, 6, 12);
      return new THREE.CylinderGeometry(1, 1, 2, 12, 3, false);
    });
  }
  function icoGeometry() { return geometry("ico", function () { return new THREE.IcosahedronGeometry(1, 1); }); }

  function addMesh(parent, geo, mat, position, scale, rotation) {
    if (!parent || !geo || !mat) return null;
    var mesh = new THREE.Mesh(geo, mat);
    if (position) mesh.position.set(num(position.x, 0), num(position.y, 0), num(position.z, 0));
    if (scale) mesh.scale.set(num(scale.x, 1), num(scale.y, 1), num(scale.z, 1));
    if (rotation) mesh.rotation.set(num(rotation.x, 0), num(rotation.y, 0), num(rotation.z, 0));
    mesh.userData.sharedCampaign = true;
    parent.add(mesh);
    return mesh;
  }

  function clearGroup(group) {
    if (!group) return;
    while (group.children.length) {
      var child = group.children[group.children.length - 1];
      if (child.traverse) child.traverse(function (node) {
        if (!node || !node.isInstancedMesh) return;
        // InstancedMesh does not own the shared geometry/material.  It does
        // own these per-instance attributes; release them before dropping the
        // node so a level change cannot retain a large matrix buffer.
        if (node.instanceMatrix) {
          if (typeof node.instanceMatrix.dispose === "function") node.instanceMatrix.dispose();
          if (node.instanceMatrix.array) node.instanceMatrix.array = new Float32Array(0);
        }
        if (node.instanceColor) {
          if (typeof node.instanceColor.dispose === "function") node.instanceColor.dispose();
          if (node.instanceColor.array) node.instanceColor.array = new Float32Array(0);
        }
      });
      group.remove(child);
    }
  }

  function gridWorld(x, y, lift) {
    return { x: num(x, 0) * UNIT, y: num(lift, 0), z: num(y, 0) * UNIT };
  }

  function setColorSpace() {
    if (!renderer) return;
    if (THREE.SRGBColorSpace) renderer.outputColorSpace = THREE.SRGBColorSpace;
    if (THREE.ACESFilmicToneMapping != null) renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = quality === "low" ? 0.96 : 1.02;
  }

  function updateTheme(force) {
    var nextTheme = worldTheme(currentWorldId);
    if (!force && theme && theme.id === nextTheme.id) return;
    theme = nextTheme;
    if (!scene) return;
    scene.background = new THREE.Color(theme.background);
    scene.fog = new THREE.Fog(theme.fog, theme.fogNear, theme.fogFar);
    if (scene.userData.keyLight) {
      scene.userData.keyLight.color.setHex(theme.trim);
      scene.userData.keyLight.intensity = quality === "low" ? 1.48 : 1.72;
    }
    if (scene.userData.fillLight) scene.userData.fillLight.color.setHex(theme.accent);
    if (scene.userData.playerLight) scene.userData.playerLight.color.setHex(theme.lamp);
    setColorSpace();
  }

  function materialForFloor(variant) {
    var count = Math.max(1, Math.min(3, theme.floor.length));
    var index = Math.abs(Math.round(num(variant, 0))) % count;
    var color = theme.floor[index];
    return material("floor" + index, color, { lambert: true });
  }

  function materialForWall(variant) {
    var color = theme.wall[0];
    return material("wall" + variant, color, { lambert: true });
  }

  function materialForActor(kind, variant) {
    var base = kind === "player" ? theme.player : kind === "npc" ? theme.npc : theme.enemy;
    var options = {
      roughness: kind === "enemy" ? 0.72 : 0.65,
      metalness: kind === "enemy" ? 0.16 : 0.28,
    };
    if (kind === "player") options.emissive = theme.accent;
    if (kind === "enemy") options.emissive = theme.warning;
    options.emissiveIntensity = kind === "enemy" ? 0.08 : 0.05;
    return material("actor-" + kind + "-" + variant, (base + variant * 0x111111) & 0xffffff, options);
  }

  function floorVariantForCell(x, y) {
    var count = Math.max(1, Math.min(3, theme && theme.floor ? theme.floor.length : 1));
    // Broad 4x3 paving fields keep the miniature readable without a noisy
    // per-cell checkerboard.  Props, rails and authored set dressing carry
    // the spatial rhythm instead of random floor colors.
    var blockX = Math.floor(num(x, 0) / 4);
    var blockY = Math.floor(num(y, 0) / 3);
    return (blockX + blockY * 2) % count;
  }

  function instanceTiles(parent, geo, buckets, yLift, wall) {
    var created = [];
    if (!parent || !geo) return created;
    buckets.forEach(function (cells, variant) {
      if (!cells.length) return;
      var instanced = new THREE.InstancedMesh(geo, wall ? materialForWall(variant) : materialForFloor(variant), cells.length);
      instanced.castShadow = !!wall;
      instanced.receiveShadow = true;
      instanced.userData.sharedCampaign = true;
      var matrix = new THREE.Matrix4();
      var identity = new THREE.Quaternion();
      var scale = new THREE.Vector3(1, 1, 1);
      var instanceColor = new THREE.Color();
      cells.forEach(function (cell, index) {
        // floorPlateGeometry is 0.12 high: center -0.06 puts its top at y=0.
        // wallBlockGeometry is 0.76 high and therefore starts at y=0 too.
        var h = wall ? 0.76 : 0.12;
        var y = wall ? h * 0.5 : yLift;
        scale.set(1, 1, 1);
        matrix.compose(new THREE.Vector3(cell.x * UNIT, y, cell.y * UNIT), identity, scale);
        instanced.setMatrixAt(index, matrix);
        if (instanced.setColorAt) {
          var shade = isExploredCell(cell.x, cell.y) ? 1 : (wall ? 0.74 : 0.66);
          instanceColor.setRGB(shade, shade, shade);
          instanced.setColorAt(index, instanceColor);
        }
        if (wall) wallCells.push({ instance: instanced, index: index, x: cell.x, y: cell.y });
      });
      instanced.instanceMatrix.needsUpdate = true;
      if (instanced.instanceColor) instanced.instanceColor.needsUpdate = true;
      instanced.userData.cells = cells;
      instanced.userData.wall = !!wall;
      parent.add(instanced);
      created.push(instanced);
    });
    return created;
  }

  function buildTopology() {
    if (!map || !topologyRoot || !THREE) return;
    clearGroup(topologyRoot);
    floorInstances = [];
    wallInstances = [];
    wallCells = [];
    lastCutawayCell = "";

    var floorBucketCount = Math.max(1, Math.min(3, theme.floor.length));
    var floorBuckets = [];
    for (var bucketIndex = 0; bucketIndex < floorBucketCount; bucketIndex += 1) floorBuckets.push([]);
    var wallBuckets = [[]];
    var waterBuckets = [];
    for (var y = 0; y < map.h; y += 1) {
      for (var x = 0; x < map.w; x += 1) {
        var tile = tileAt(x, y);
        if (tile === "#") {
          wallBuckets[0].push({ x: x, y: y });
        } else if (tile === "~") {
          waterBuckets.push({ x: x, y: y });
        } else {
          floorBuckets[floorVariantForCell(x, y)].push({ x: x, y: y });
        }
      }
    }
    floorInstances = instanceTiles(topologyRoot, floorPlateGeometry(), floorBuckets, -0.06, false);
    wallInstances = instanceTiles(topologyRoot, wallBlockGeometry(), wallBuckets, 0, true);

    if (waterBuckets.length) {
      var waterMesh = new THREE.InstancedMesh(floorPlateGeometry(), material("water", theme.water, {
        roughness: 0.42,
        metalness: 0.18,
        emissive: theme.accent,
        emissiveIntensity: 0.08,
      }), waterBuckets.length);
      var waterMatrix = new THREE.Matrix4();
      var waterQuat = new THREE.Quaternion();
      var waterScale = new THREE.Vector3(1, 1, 1);
      var waterColor = new THREE.Color();
      waterBuckets.forEach(function (cell, index) {
        // Water has the same thin plate; its top is deliberately -0.06.
        waterMatrix.compose(new THREE.Vector3(cell.x, -0.12, cell.y), waterQuat, waterScale);
        waterMesh.setMatrixAt(index, waterMatrix);
        if (waterMesh.setColorAt) {
          var waterShade = isExploredCell(cell.x, cell.y) ? 1 : 0.70;
          waterColor.setRGB(waterShade, waterShade, waterShade);
          waterMesh.setColorAt(index, waterColor);
        }
      });
      waterMesh.instanceMatrix.needsUpdate = true;
      if (waterMesh.instanceColor) waterMesh.instanceColor.needsUpdate = true;
      waterMesh.userData.cells = waterBuckets;
      waterMesh.userData.water = true;
      waterMesh.userData.sharedCampaign = true;
      topologyRoot.add(waterMesh);
    }

    // A finite plinth makes the scene read as a desktop miniature instead of an
    // endless tiled plane.  The camera never receives a second, infinite grid.
    baseMesh = addMesh(topologyRoot, boxGeometry(), material("plinth", theme.floorEdge, { roughness: 0.98, metalness: 0.02 }),
      { x: (map.w - 1) * 0.5, y: -0.20, z: (map.h - 1) * 0.5 },
      { x: map.w + 1.4, y: 0.16, z: map.h + 1.4 });
    if (baseMesh) baseMesh.renderOrder = -10;
    addHarborWaterDetails();
    topologyRebuilds += 1;
    refreshCutaway(true);
  }

  function refreshExploredColors() {
    if (fallback || !topologyRoot || !THREE) return;
    var instanceColor = new THREE.Color();
    topologyRoot.children.forEach(function (node) {
      if (!node || !node.isInstancedMesh || !node.userData || !node.userData.cells || !node.setColorAt) return;
      var wall = !!node.userData.wall;
      node.userData.cells.forEach(function (cell, index) {
        var shade = isExploredCell(cell.x, cell.y) ? 1 : (node.userData.water ? 0.70 : wall ? 0.74 : 0.66);
        instanceColor.setRGB(shade, shade, shade);
        node.setColorAt(index, instanceColor);
      });
      if (node.instanceColor) node.instanceColor.needsUpdate = true;
    });
  }

  function normalizeKind(kind) {
    return safeString(kind).toLowerCase().replace(/[\s_\-]+/g, "");
  }

  function propCategory(kind) {
    var k = normalizeKind(kind);
    if (k.indexOf("maintenance") >= 0 || k.indexOf("pump") >= 0 || k.indexOf("shaft") >= 0 || k.indexOf("井") >= 0 || k.indexOf("泵") >= 0 || k.indexOf("维护") >= 0) return "maintenance";
    if (k.indexOf("rack") >= 0 || k.indexOf("cabinet") >= 0 || k.indexOf("equipmentcabinet") >= 0 || k.indexOf("机柜") >= 0 || k.indexOf("设备柜") >= 0) return "rack";
    if (k.indexOf("lamp") >= 0 || k.indexOf("lightstand") >= 0 || k.indexOf("灯架") >= 0 || k.indexOf("照明") >= 0) return "lamp";
    if (k.indexOf("console") >= 0 || k.indexOf("terminal") >= 0 || k.indexOf("controlpanel") >= 0 || k.indexOf("控制台") >= 0 || k.indexOf("终端") >= 0) return "console";
    if (k.indexOf("pipe") >= 0 || k.indexOf("pipeline") >= 0 || k.indexOf("管") >= 0) return "pipe";
    if (k.indexOf("power") >= 0 || k.indexOf("station") >= 0 || k.indexOf("substation") >= 0 || k.indexOf("电站") >= 0 || k.indexOf("配电") >= 0) return "power";
    if (k.indexOf("warehouse") >= 0 || k.indexOf("storage") >= 0 || k.indexOf("仓") >= 0 || k.indexOf("库") >= 0) return "warehouse";
    if (k.indexOf("crane") >= 0 || k.indexOf("dock") >= 0 || k.indexOf("码头") >= 0 || k.indexOf("吊机") >= 0) return "crane";
    if (k.indexOf("gear") >= 0 || k.indexOf("factory") >= 0 || k.indexOf("clock") >= 0 || k.indexOf("valve") >= 0 || k.indexOf("wheel") >= 0 || k.indexOf("齿轮") >= 0 || k.indexOf("阀") >= 0 || k.indexOf("工厂") >= 0) return "gear";
    if (k.indexOf("furnace") >= 0 || k.indexOf("炉") >= 0 || k.indexOf("steam") >= 0 || k.indexOf("蒸汽") >= 0) return "furnace";
    return "crate";
  }

  function propSize(prop, category) {
    var defaults = {
      maintenance: { w: 2, h: 2 },
      rack: { w: 3, h: 2 },
      lamp: { w: 1, h: 1 },
      console: { w: 2, h: 2 },
      pipe: { w: 3, h: 1 },
      power: { w: 3, h: 2 },
      warehouse: { w: 4, h: 3 },
      crane: { w: 4, h: 3 },
      gear: { w: 3, h: 2 },
      furnace: { w: 2, h: 2 },
      crate: { w: 2, h: 2 },
    };
    var d = defaults[category] || defaults.crate;
    return { w: Math.max(1, num(prop.w, d.w)), h: Math.max(1, num(prop.h, d.h)) };
  }

  function putBox(parent, mat, x, y, z, sx, sy, sz, rotationY) {
    return addMesh(parent, boxGeometry(), mat, { x: x, y: y, z: z }, { x: sx, y: sy, z: sz }, { x: 0, y: num(rotationY, 0), z: 0 });
  }

  function putCylinder(parent, mat, x, y, z, radius, height, rotationX, rotationZ) {
    return addMesh(parent, cylinderGeometry(), mat, { x: x, y: y, z: z }, { x: radius, y: height, z: radius }, { x: num(rotationX, 0), y: 0, z: num(rotationZ, 0) });
  }

  function putRing(parent, mat, x, y, z, scale, rotationX) {
    return addMesh(parent, ringGeometry(), mat, { x: x, y: y, z: z }, { x: scale, y: scale, z: scale }, { x: num(rotationX, -Math.PI / 2), y: 0, z: 0 });
  }

  function horizontalCylinder(parent, mat, x, y, z, length, radius, axis) {
    if (axis === "z") return putCylinder(parent, mat, x, y, z, radius, length, Math.PI / 2, 0);
    return putCylinder(parent, mat, x, y, z, radius, length, 0, Math.PI / 2);
  }

  function addPlatform(parent, mat, width, depth, height) {
    putBox(parent, mat, 0, height * 0.5, 0, width, height, depth);
  }

  function addMaintenanceStation(parent, size) {
    // Grey Hive is a fictional underground maintenance level: a hatch,
    // pump housing and service pipes, never a hexagonal hive motif.
    var concrete = material("maintenance-concrete", theme.wall[1], { roughness: 0.90, metalness: 0.16 });
    var steel = material("maintenance-steel", theme.trim, { roughness: 0.38, metalness: 0.78 });
    var lamp = material("maintenance-lamp", theme.lamp, { basic: true, transparent: true, opacity: 0.94 });
    var dark = material("maintenance-dark", theme.floorEdge, { roughness: 0.66, metalness: 0.60 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    addPlatform(parent, dark, w, h, 0.10);
    putRing(parent, steel, -w * 0.20, 0.13, 0, 0.43, -Math.PI / 2);
    putCylinder(parent, dark, -w * 0.20, 0.15, 0, 0.28, 0.10);
    putCylinder(parent, concrete, w * 0.22, 0.38, 0, 0.27, 0.64);
    putRing(parent, steel, w * 0.22, 0.69, 0, 0.28, -Math.PI / 2);
    horizontalCylinder(parent, steel, -w * 0.10, 0.73, 0, w * 0.52, 0.07, "x");
    horizontalCylinder(parent, steel, w * 0.27, 0.52, 0.02, 0.38, 0.07, "z");
    putBox(parent, lamp, -w * 0.32, 0.69, h * 0.34, 0.18, 0.08, 0.05);
    putBox(parent, lamp, w * 0.32, 0.69, -h * 0.34, 0.18, 0.08, 0.05);
  }

  function addPipeAssembly(parent, size) {
    var metal = material("pipe-metal", theme.trim, { roughness: 0.38, metalness: 0.78 });
    var dark = material("pipe-dark", theme.floorEdge, { roughness: 0.55, metalness: 0.64 });
    var light = material("pipe-light", theme.lamp, { basic: true, emissive: theme.lamp, emissiveIntensity: 0.4 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    addPlatform(parent, material("platform", theme.floorEdge, { roughness: 0.87, metalness: 0.22 }), w, h, 0.08);
    horizontalCylinder(parent, metal, -w * 0.25, 0.48, 0, w * 0.56, 0.075, "x");
    horizontalCylinder(parent, metal, w * 0.26, 0.67, 0.18, 0.34, 0.075, "z");
    putCylinder(parent, dark, -w * 0.25, 0.25, 0, 0.11, 0.5);
    putCylinder(parent, dark, w * 0.26, 0.34, 0.18, 0.11, 0.68);
    putRing(parent, light, w * 0.25, 0.74, 0.18, 0.25, -Math.PI / 2);
    putRing(parent, metal, -w * 0.25, 0.28, 0, 0.19, -Math.PI / 2);
  }

  function addPowerStation(parent, size) {
    var steel = material("power-steel", theme.trim, { roughness: 0.46, metalness: 0.7 });
    var dark = material("power-dark", theme.floorEdge, { roughness: 0.7, metalness: 0.52 });
    var amber = material("power-amber", theme.lamp, { basic: true, emissive: theme.lamp, emissiveIntensity: 0.7 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    addPlatform(parent, dark, w, h, 0.10);
    putBox(parent, dark, -w * 0.22, 0.43, 0, w * 0.38, 0.72, h * 0.58);
    putBox(parent, steel, w * 0.20, 0.30, 0, w * 0.24, 0.48, h * 0.52);
    [-0.38, 0, 0.38].forEach(function (x) {
      putCylinder(parent, steel, x * w, 0.95, 0, 0.045, 0.92);
      putRing(parent, amber, x * w, 1.43, 0, 0.105, -Math.PI / 2);
    });
    horizontalCylinder(parent, steel, -w * 0.36, 1.16, 0, w * 0.72, 0.035, "x");
    putRing(parent, amber, -w * 0.28, 0.42, -h * 0.34, 0.11, -Math.PI / 2);
  }

  function addRackCabinets(parent, size) {
    var body = material("rack-body", theme.floorEdge, { roughness: 0.72, metalness: 0.52 });
    var panelA = material("rack-panel-a", theme.wall[1], { roughness: 0.64, metalness: 0.38 });
    var panelB = material("rack-panel-b", theme.trim, { roughness: 0.56, metalness: 0.46 });
    var vent = material("rack-vent", theme.background, { roughness: 0.84, metalness: 0.28 });
    var status = material("rack-status", theme.lamp, { basic: true, transparent: true, opacity: 0.92 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    var depth = Math.min(0.58, h * 0.72);
    var platformW = Math.max(0.48, w * 0.90);
    var platformD = Math.max(0.42, h * 0.82);
    addPlatform(parent, body, platformW, platformD, 0.07);
    var count = w >= 1.55 ? 3 : 2;
    var gap = Math.min(0.10, w * 0.07);
    var cabinetW = Math.min(0.44, Math.max(0.22, (platformW * 0.84 - gap * (count - 1)) / count));
    var span = cabinetW * count + gap * (count - 1);
    for (var i = 0; i < count; i += 1) {
      var x = (i - (count - 1) * 0.5) * (cabinetW + gap);
      putBox(parent, body, x, 0.49, 0, cabinetW, 0.88, depth);
      putBox(parent, i % 2 ? panelB : panelA, x, 0.50, depth * 0.51, cabinetW * 0.78, 0.72, 0.026);
      for (var row = 0; row < 4; row += 1) {
        putBox(parent, vent, x, 0.29 + row * 0.13, depth * 0.535, cabinetW * 0.52, 0.025, 0.018);
      }
      putBox(parent, status, x - cabinetW * 0.29, 0.88, depth * 0.54, 0.035, 0.035, 0.026);
    }
    // A top rail makes the separate cabinets read as one authored equipment
    // bank while staying inside the backend-provided footprint.
    putBox(parent, panelB, 0, 0.96, 0, Math.min(platformW * 0.92, span + 0.12), 0.065, depth * 0.84);
  }

  function addLampStand(parent, size) {
    var metal = material("lamp-stand-metal", theme.trim, { roughness: 0.46, metalness: 0.70 });
    var dark = material("lamp-stand-dark", theme.floorEdge, { roughness: 0.72, metalness: 0.46 });
    var glow = material("lamp-stand-glow", theme.lamp, { basic: true, transparent: true, opacity: 0.96 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    var span = Math.min(0.72, Math.max(0.24, w * 0.70));
    var depth = Math.min(0.46, Math.max(0.22, h * 0.64));
    addPlatform(parent, dark, Math.max(0.38, w * 0.74), Math.max(0.32, h * 0.72), 0.06);
    [-0.5, 0.5].forEach(function (side) {
      putCylinder(parent, metal, side * span, 0.65, 0, 0.045, 1.20);
      putBox(parent, glow, side * span, 1.30, depth * 0.18, 0.14, 0.09, 0.10);
    });
    horizontalCylinder(parent, metal, 0, 1.24, 0, span * 2, 0.045, "x");
    putBox(parent, glow, 0, 1.20, depth * 0.18, span * 1.14, 0.07, 0.08);
    horizontalCylinder(parent, dark, 0, 0.30, 0, span * 1.65, 0.028, "x");
  }

  function addConsole(parent, size) {
    var dark = material("console-dark", theme.floorEdge, { roughness: 0.72, metalness: 0.48 });
    var body = material("console-body", theme.wall[1], { roughness: 0.66, metalness: 0.38 });
    var screen = material("console-screen", theme.accent, { basic: true, transparent: true, opacity: 0.90 });
    var key = material("console-key", theme.lamp, { basic: true, transparent: true, opacity: 0.88 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    var depth = Math.min(0.62, Math.max(0.30, h * 0.70));
    var width = Math.max(0.44, w * 0.72);
    addPlatform(parent, dark, Math.max(0.52, w * 0.82), Math.max(0.42, h * 0.76), 0.07);
    putBox(parent, dark, 0, 0.30, -depth * 0.12, width * 0.78, 0.42, depth * 0.42);
    addMesh(parent, boxGeometry(), body, { x: 0, y: 0.56, z: depth * 0.05 }, { x: width, y: 0.42, z: depth * 0.56 }, { x: -0.22, y: 0, z: 0 });
    addMesh(parent, boxGeometry(), screen, { x: 0, y: 0.66, z: depth * 0.28 }, { x: width * 0.72, y: 0.16, z: 0.026 }, { x: -0.22, y: 0, z: 0 });
    putBox(parent, key, -width * 0.22, 0.44, depth * 0.30, width * 0.22, 0.035, 0.055);
    putBox(parent, key, width * 0.23, 0.44, depth * 0.30, width * 0.14, 0.035, 0.055);
  }

  function addWarehouse(parent, size) {
    var wall = material("warehouse-wall", theme.wall[1], { roughness: 0.88, metalness: 0.22 });
    var roof = material("warehouse-roof", theme.trim, { roughness: 0.52, metalness: 0.6 });
    var door = material("warehouse-door", theme.floorEdge, { roughness: 0.62, metalness: 0.52 });
    var lamp = material("warehouse-lamp", theme.lamp, { basic: true, emissive: theme.lamp, emissiveIntensity: 0.9 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    addPlatform(parent, material("platform-warehouse", theme.floorEdge, { roughness: 0.9, metalness: 0.12 }), w, h, 0.10);
    putBox(parent, wall, 0, 0.48, 0, w * 0.88, 0.86, h * 0.82);
    putBox(parent, roof, 0, 0.96, 0, w * 0.96, 0.10, h * 0.90);
    putBox(parent, door, 0, 0.38, h * 0.42, w * 0.28, 0.58, 0.045);
    putBox(parent, lamp, -w * 0.26, 0.78, h * 0.46, 0.18, 0.08, 0.035);
    putBox(parent, lamp, w * 0.26, 0.78, h * 0.46, 0.18, 0.08, 0.035);
    [-0.30, -0.03, 0.24].forEach(function (x) { putBox(parent, door, x * w, 0.20, -h * 0.49, 0.24, 0.20, 0.24); });
  }

  function addCrane(parent, size) {
    var steel = material("crane-steel", theme.trim, { roughness: 0.4, metalness: 0.82 });
    var rust = material("crane-rust", theme.warning, { roughness: 0.6, metalness: 0.68 });
    var lamp = material("crane-lamp", theme.lamp, { basic: true, emissive: theme.lamp, emissiveIntensity: 0.85 });
    var w = size.w * 0.78;
    var h = size.h * 0.78;
    addPlatform(parent, material("platform-crane", theme.floorEdge, { roughness: 0.87, metalness: 0.28 }), w, h, 0.08);
    [-0.38, 0.38].forEach(function (x) {
      putBox(parent, steel, x * w, 0.82, 0, 0.10, 1.62, 0.10);
      putRing(parent, rust, x * w, 1.53, 0, 0.16, -Math.PI / 2);
    });
    putBox(parent, steel, 0, 1.56, 0, w * 0.94, 0.12, 0.12);
    horizontalCylinder(parent, rust, w * 0.18, 1.39, 0, 0.22, 0.055, "x");
    putCylinder(parent, steel, w * 0.18, 1.04, 0, 0.025, 0.72);
    putBox(parent, lamp, 0, 1.61, 0, 0.22, 0.035, 0.035);
  }

  function addGearAssembly(parent, size) {
    var copper = material("gear-copper", theme.trim, { roughness: 0.32, metalness: 0.88 });
    var bronze = material("gear-bronze", theme.warning, { roughness: 0.44, metalness: 0.78 });
    var dark = material("gear-dark", theme.floorEdge, { roughness: 0.62, metalness: 0.64 });
    var w = size.w * 0.74;
    var h = size.h * 0.74;
    addPlatform(parent, dark, w, h, 0.10);
    putRing(parent, copper, -w * 0.18, 0.12, 0, 0.52, -Math.PI / 2);
    putRing(parent, bronze, w * 0.28, 0.12, 0.05, 0.34, -Math.PI / 2);
    putCylinder(parent, dark, -w * 0.18, 0.15, 0, 0.11, 0.20);
    putCylinder(parent, dark, w * 0.28, 0.15, 0.05, 0.08, 0.20);
    for (var i = 0; i < 8; i += 1) {
      var angle = i * TAU / 8;
      putBox(parent, bronze, -w * 0.18 + Math.cos(angle) * 0.42, 0.14, Math.sin(angle) * 0.42, 0.08, 0.16, 0.22, angle);
    }
    horizontalCylinder(parent, copper, 0, 0.58, 0, w * 0.82, 0.065, "x");
    putRing(parent, bronze, 0, 0.58, 0, 0.17, -Math.PI / 2);
  }

  function addFurnace(parent, size) {
    var metal = material("furnace-metal", theme.wall[1], { roughness: 0.48, metalness: 0.76 });
    var hot = material("furnace-hot", theme.lamp, { basic: true, emissive: theme.warning, emissiveIntensity: 1.8 });
    var dark = material("furnace-dark", theme.floorEdge, { roughness: 0.72, metalness: 0.5 });
    var w = size.w * 0.72;
    var h = size.h * 0.72;
    addPlatform(parent, dark, w, h, 0.10);
    putCylinder(parent, metal, 0, 0.53, 0, 0.35, 0.92);
    putRing(parent, dark, 0, 0.88, 0, 0.37, -Math.PI / 2);
    putRing(parent, hot, 0, 0.91, 0, 0.19, -Math.PI / 2);
    putCylinder(parent, hot, 0, 1.12, 0, 0.16, 0.15);
    horizontalCylinder(parent, metal, 0, 1.12, 0, 0.68, 0.09, "z");
  }

  function addCrates(parent, size) {
    var wood = material("crate-wood", theme.warning, { roughness: 0.82, metalness: 0.16 });
    var dark = material("crate-dark", theme.floorEdge, { roughness: 0.86, metalness: 0.12 });
    var w = size.w * 0.74;
    var h = size.h * 0.74;
    addPlatform(parent, dark, w, h, 0.08);
    putBox(parent, wood, -w * 0.20, 0.18, -h * 0.14, 0.34, 0.34, 0.34);
    putBox(parent, wood, w * 0.18, 0.18, h * 0.10, 0.34, 0.34, 0.34);
    putBox(parent, dark, 0, 0.48, 0, 0.36, 0.28, 0.36);
  }

  function buildPropMesh(prop) {
    var category = propCategory(prop.kind);
    var size = propSize(prop, category);
    var group = new THREE.Group();
    group.userData.sharedCampaign = true;
    if (category === "maintenance") addMaintenanceStation(group, size);
    else if (category === "rack") addRackCabinets(group, size);
    else if (category === "lamp") addLampStand(group, size);
    else if (category === "console") addConsole(group, size);
    else if (category === "pipe") addPipeAssembly(group, size);
    else if (category === "power") addPowerStation(group, size);
    else if (category === "warehouse") addWarehouse(group, size);
    else if (category === "crane") addCrane(group, size);
    else if (category === "gear") addGearAssembly(group, size);
    else if (category === "furnace") addFurnace(group, size);
    else addCrates(group, size);
    // map.props.rotation is the shared JSON degree field (90 means quarter
    // turn).  Keep the conversion at this boundary so all prop builders use
    // the same convention.
    var rot = num(prop.rotation, 0) * Math.PI / 180;
    group.rotation.y = rot;
    var p = gridWorld(prop.x, prop.y, 0);
    group.position.set(p.x, 0, p.z);
    group.userData.category = category;
    group.userData.grid = { x: Math.round(num(prop.x, 0)), y: Math.round(num(prop.y, 0)) };
    return group;
  }

  function effectiveProps() {
    // Props are authored by the backend.  Do not invent large facilities on
    // arbitrary walkable cells: the renderer must not block paths or make a
    // sparse birth map look accidentally cluttered.
    return (map && map.props ? map.props : []).map(function (prop) {
      return Object.assign({}, prop, { x: Math.round(num(prop.x, 0)), y: Math.round(num(prop.y, 0)) });
    });
  }

  function markerMaterial(kind, done) {
    var color = kind === "portal" ? theme.accent : kind === "gate" ? theme.warning : theme.lamp;
    if (done) color = theme.floorEdge;
    return material("marker-" + kind + (done ? "-done" : ""), color, { basic: true, transparent: true, opacity: done ? 0.28 : 0.86, emissive: color, emissiveIntensity: done ? 0.05 : 0.9 });
  }

  function addMarker(marker, kind, index) {
    if (!dynamicDecorationRoot) return;
    var group = new THREE.Group();
    var done = !!marker.done;
    var mat = markerMaterial(kind, done);
    var p = gridWorld(marker.x, marker.y, 0);
    group.position.set(p.x, 0, p.z);
    putCylinder(group, material("marker-base", theme.floorEdge, { roughness: 0.62, metalness: 0.72 }), 0, 0.13, 0, 0.22, 0.26);
    var ring = putRing(group, mat, 0, 0.23, 0, kind === "portal" ? 0.46 : 0.35, -Math.PI / 2);
    var column = putCylinder(group, mat, 0, 0.70, 0, kind === "portal" ? 0.14 : 0.095, kind === "portal" ? 1.00 : 0.82);
    var diamond = addMesh(group, icoGeometry(), mat, { x: 0, y: kind === "portal" ? 1.28 : 1.12, z: 0 }, { x: 0.13, y: 0.18, z: 0.13 });
    if (ring) ring.userData.markerRing = true;
    if (column) column.userData.markerColumn = true;
    if (diamond) diamond.userData.markerDiamond = true;
    group.userData.marker = marker;
    group.userData.kind = kind;
    group.userData.phase = hash(Math.round(num(marker.x, 0)), Math.round(num(marker.y, 0)), index + 11) / 1000;
    group.userData.sharedCampaign = true;
    dynamicDecorationRoot.add(group);
    dynamicMarkers.push(group);
  }

  function addDangerRing(zone, index, telegraph) {
    if (!dynamicDecorationRoot) return;
    var radius = Math.max(0.35, num(zone.radius, telegraph ? 1.4 : 1.0));
    var group = new THREE.Group();
    var mat = material("danger-ring" + (telegraph ? "-telegraph" : ""), telegraph ? theme.warning : theme.warning, { basic: true, transparent: true, opacity: telegraph ? 0.78 : 0.54, emissive: theme.warning, emissiveIntensity: 0.75 });
    var ring = putRing(group, mat, 0, 0.035, 0, radius * 1.35, -Math.PI / 2);
    var inner = putRing(group, mat, 0, 0.04, 0, radius, -Math.PI / 2);
    var p = gridWorld(zone.x, zone.y, 0);
    group.position.set(p.x, 0, p.z);
    group.userData.zone = zone;
    group.userData.phase = hash(Math.round(num(zone.x, 0)), Math.round(num(zone.y, 0)), index + 61) / 1000;
    group.userData.telegraph = !!telegraph;
    if (ring) ring.userData.dangerRing = true;
    if (inner) inner.userData.dangerRing = true;
    dynamicDecorationRoot.add(group);
    hazardMarkers.push(group);
  }

  function isWaterFog(zone) {
    var kind = normalizeKind(zone && zone.kind);
    return kind === "waterfog" || (kind.indexOf("water") >= 0 && kind.indexOf("fog") >= 0);
  }

  function isSteamVent(zone) {
    var kind = normalizeKind(zone && zone.kind);
    return kind === "steamventa" || kind === "steamventb" || kind.indexOf("steamvent") >= 0;
  }

  function telegraphIsActive(telegraph) {
    return !!telegraph && (telegraph.remaining == null || num(telegraph.remaining, 0) > 0);
  }

  function addActiveSteamArea(zone, index) {
    if (!dynamicDecorationRoot || !zone || zone.active === false || isWaterFog(zone)) return;
    var group = new THREE.Group();
    var mat = material("steam-area", theme.steam, { basic: true, transparent: true, opacity: 0.26 });
    var ring = putRing(group, mat, 0, 0.035, 0, 0.50, -Math.PI / 2);
    if (ring) {
      ring.scale.set(Math.max(0.72, num(zone.w, 1) * 0.72), 1, Math.max(0.72, num(zone.h, 1) * 0.72));
      ring.userData.steamArea = true;
    }
    var p = gridWorld(zone.x, zone.y, 0);
    group.position.set(p.x, 0, p.z);
    group.userData.zone = zone;
    group.userData.area = true;
    group.userData.phase = hash(Math.round(num(zone.x, 0)), Math.round(num(zone.y, 0)), index + 71) / 1000;
    group.userData.sharedCampaign = true;
    dynamicDecorationRoot.add(group);
    hazardMarkers.push(group);
  }

  function nearbyPosition(nearby) {
    if (nearby && nearby.x != null && nearby.y != null) return { x: Math.round(num(nearby.x, 0)), y: Math.round(num(nearby.y, 0)) };
    return {
      x: Math.round(num(map && map.px, 0) + num(nearby && nearby.dx, 0)),
      y: Math.round(num(map && map.py, 0) + num(nearby && nearby.dy, 0)),
    };
  }

  function addInteractionMarker(nearby, index) {
    if (!dynamicDecorationRoot) return;
    var cell = nearbyPosition(nearby);
    var group = new THREE.Group();
    var mat = material("interaction", theme.accent, { basic: true, transparent: true, opacity: 0.82, emissive: theme.accent, emissiveIntensity: 1.1 });
    putRing(group, mat, 0, 0.045, 0, 0.44, -Math.PI / 2);
    putCylinder(group, mat, 0, 0.70, 0, 0.06, 1.26);
    putRing(group, mat, 0, 1.32, 0, 0.16, -Math.PI / 2);
    var p = gridWorld(cell.x, cell.y, 0);
    group.position.set(p.x, 0, p.z);
    group.userData.phase = hash(cell.x, cell.y, index + 81) / 1000;
    group.userData.cell = cell;
    group.userData.nearby = nearby;
    group.userData.sharedCampaign = true;
    dynamicDecorationRoot.add(group);
    interactionMarkers.push(group);
  }

  function addHarborWaterDetails() {
    if (!map || theme.id !== "mist_harbor" || !topologyRoot) return;
    var mat = material("water-sheen", theme.trim, { basic: true, transparent: true, opacity: 0.32 });
    var count = 0;
    for (var y = 0; y < map.h && count < 72; y += 1) {
      for (var x = 0; x < map.w && count < 72; x += 1) {
        if (tileAt(x, y) !== "~" || hash(x, y, 121) % 11 !== 0) continue;
        var p = gridWorld(x, y, 0);
        var line = putBox(topologyRoot, mat, p.x, -0.045, p.z, 0.42, 0.012, 0.025, (hash(x, y, 7) % 2 ? 0.18 : -0.18));
        if (line) line.userData.waterDetail = true;
        count += 1;
      }
    }
  }

  function staticDecorationSignature(nextMap) {
    if (!nextMap) return "";
    return [nextMap.world_id, listSignature(nextMap.props)].join("|");
  }

  function dynamicDecorationSignature(nextMap) {
    if (!nextMap) return "";
    return [
      nextMap.world_id,
      Math.round(num(nextMap.px, 0)),
      Math.round(num(nextMap.py, 0)),
      listSignature(nextMap.points),
      listSignature(nextMap.portals),
      listSignature(nextMap.gates),
      listSignature(nextMap.nearby),
      listSignature(nextMap.zones),
      listSignature(nextMap.enemies.map(function (enemy) { return { id: enemy.id, alive: enemy.alive !== false, telegraph: enemy.telegraph || null }; })),
      listSignature(nextMap.telegraph),
    ].join("|");
  }

  function syncStaticDecorations() {
    if (!map) return;
    var nextKey = staticDecorationSignature(map);
    if (nextKey === staticPropKey) return;
    staticPropKey = nextKey;
    decorProps = effectiveProps();
    if (!fallback && staticPropRoot) {
      clearGroup(staticPropRoot);
      decorProps.forEach(function (prop) {
        var mesh = buildPropMesh(prop);
        if (mesh) staticPropRoot.add(mesh);
      });
    }
    decorationRebuilds += 1;
  }

  function syncDynamicDecorations() {
    if (!map) return;
    var nextKey = dynamicDecorationSignature(map);
    if (nextKey === dynamicDecorationKey) return;
    dynamicDecorationKey = nextKey;
    decorationKey = staticPropKey + "|" + dynamicDecorationKey;
    fallbackMarkerData = [];
    fallbackHazardData = [];
    fallbackInteractionData = [];
    (map.points || []).forEach(function (point) { fallbackMarkerData.push({ marker: point, kind: "point" }); });
    (map.portals || []).forEach(function (portal) { fallbackMarkerData.push({ marker: portal, kind: "portal" }); });
    (map.gates || []).forEach(function (gate) { fallbackMarkerData.push({ marker: gate, kind: "gate" }); });
    (map.zones || []).forEach(function (zone) {
      if (!zone || zone.active === false || isWaterFog(zone)) return;
      if (isSteamVent(zone)) fallbackHazardData.push({ zone: zone, telegraph: false, area: true });
      if (telegraphIsActive(zone.telegraph)) fallbackHazardData.push({ zone: zone.telegraph, telegraph: true, area: false });
      else if (!isSteamVent(zone)) fallbackHazardData.push({ zone: zone, telegraph: false, area: false });
    });
    (map.enemies || []).forEach(function (enemy) {
      if (enemy.alive !== false && telegraphIsActive(enemy.telegraph)) fallbackHazardData.push({ zone: enemy.telegraph, telegraph: true, area: false, enemyId: enemy.id || null });
    });
    if (telegraphIsActive(map.telegraph)) fallbackHazardData.push({ zone: map.telegraph, telegraph: true, area: false });
    (map.nearby || []).forEach(function (nearby) { fallbackInteractionData.push({ nearby: nearby, cell: nearbyPosition(nearby) }); });

    if (!fallback && dynamicDecorationRoot) {
      clearGroup(dynamicDecorationRoot);
      dynamicMarkers = [];
      hazardMarkers = [];
      interactionMarkers = [];
      (map.points || []).forEach(function (point, index) { addMarker(point, "point", index); });
      (map.portals || []).forEach(function (portal, index) { addMarker(portal, "portal", index + 13); });
      (map.gates || []).forEach(function (gate, index) { addMarker(gate, "gate", index + 27); });
      (map.zones || []).forEach(function (zone, index) {
        if (!zone || zone.active === false || isWaterFog(zone)) return;
        if (isSteamVent(zone)) addActiveSteamArea(zone, index);
        if (telegraphIsActive(zone.telegraph)) addDangerRing(zone.telegraph, index + 101, true);
        else if (!isSteamVent(zone)) addDangerRing(zone, index + 101, false);
      });
      (map.enemies || []).forEach(function (enemy, index) {
        if (enemy.alive !== false && telegraphIsActive(enemy.telegraph)) addDangerRing(enemy.telegraph, index + 151, true);
      });
      if (telegraphIsActive(map.telegraph)) addDangerRing(map.telegraph, 191, true);
      (map.nearby || []).forEach(addInteractionMarker);
    }
    decorationRebuilds += 1;
  }

  function actorId(item, kind, index) {
    return safeString(item && item.id) || (kind + "-" + index);
  }

  function actorPalette(kind, item) {
    var n = normalizeKind(item && (item.kind || item.name));
    if (kind === "player") return 0;
    if (kind === "npc") return n.indexOf("engineer") >= 0 || n.indexOf("工") >= 0 ? 1 : 0;
    if (n.indexOf("brute") >= 0 || n.indexOf("heavy") >= 0 || n.indexOf("重") >= 0) return 2;
    if (n.indexOf("scout") >= 0 || n.indexOf("fast") >= 0 || n.indexOf("迅") >= 0) return 1;
    return 0;
  }

  function addHealthBar(parent, kind) {
    var bar = new THREE.Group();
    var back = addMesh(bar, boxGeometry(), material("hp-back", 0x1a1f24, { basic: true, transparent: true, opacity: 0.92 }), { x: 0, y: 1.78, z: 0 }, { x: 0.94, y: 0.045, z: 0.035 });
    var color = kind === "enemy" ? theme.warning : kind === "player" ? theme.accent : theme.lamp;
    var fill = addMesh(bar, boxGeometry(), material("hp-fill-" + kind, color, { basic: true, emissive: color, emissiveIntensity: 0.7 }), { x: 0, y: 1.78, z: 0.006 }, { x: 0.88, y: 0.028, z: 0.04 });
    bar.userData.back = back;
    bar.userData.fill = fill;
    bar.userData.kind = kind;
    bar.userData.sharedCampaign = true;
    parent.add(bar);
    return bar;
  }

  function makeCenYaoModel() {
    var root = new THREE.Group();
    var rig = new THREE.Group();
    var coat = material("cenyao-coat", 0x202a31, { roughness: 0.74, metalness: 0.24 });
    var coatEdge = material("cenyao-coat-edge", 0x35424a, { roughness: 0.68, metalness: 0.22 });
    var inner = material("cenyao-inner", 0x60777d, { roughness: 0.72, metalness: 0.12 });
    var hair = material("cenyao-hair", 0x10151b, { roughness: 0.88, metalness: 0.05 });
    var skin = material("cenyao-skin", 0xc59e87, { roughness: 0.70, metalness: 0.04 });
    var gold = material("cenyao-gold", 0xd5a653, { roughness: 0.38, metalness: 0.72, emissive: 0x6d4312, emissiveIntensity: 0.14 });
    var beacon = material("cenyao-beacon", 0xffd36e, { basic: true, transparent: true, opacity: 0.97, emissive: 0xffb629, emissiveIntensity: 1.7 });
    var boot = material("cenyao-boot", 0x151b20, { roughness: 0.62, metalness: 0.48 });
    rig.position.y = 0.04;
    root.add(rig);

    // Tall, narrow torso with the asymmetric jacket silhouette from the
    // portrait: raised collar, split lapels and one plated shoulder.
    addMesh(rig, capsuleGeometry(), coat, { x: 0, y: 1.07, z: 0 }, { x: 0.31, y: 0.48, z: 0.25 });
    putBox(rig, inner, 0, 1.13, 0.235, 0.20, 0.42, 0.055);
    putBox(rig, coatEdge, -0.20, 1.30, 0.20, 0.10, 0.46, 0.07, -0.13);
    putBox(rig, coatEdge, 0.20, 1.28, 0.20, 0.10, 0.43, 0.07, 0.18);
    putBox(rig, coat, -0.29, 1.20, 0, 0.16, 0.42, 0.22, -0.08);
    putBox(rig, coat, 0.30, 1.12, 0, 0.16, 0.32, 0.22, 0.16);
    putBox(rig, gold, -0.33, 1.49, 0.01, 0.20, 0.10, 0.30, -0.18);
    putBox(rig, gold, 0.13, 1.52, 0.02, 0.11, 0.055, 0.26, 0.10);
    putBox(rig, coatEdge, 0, 1.53, -0.02, 0.30, 0.10, 0.23);

    // Face and short black hair; deliberately no helmet/visor so the model
    // reads as the original 岑遥 portrait instead of the old armored actor.
    addMesh(rig, sphereGeometry(), skin, { x: 0, y: 1.72, z: 0.01 }, { x: 0.225, y: 0.29, z: 0.205 });
    putBox(rig, hair, 0, 1.91, -0.01, 0.40, 0.18, 0.34);
    putBox(rig, hair, -0.17, 1.80, 0.18, 0.12, 0.23, 0.10, -0.18);
    putBox(rig, hair, 0.14, 1.83, 0.18, 0.10, 0.21, 0.10, 0.16);
    putBox(rig, coatEdge, 0, 1.50, 0.26, 0.08, 0.16, 0.035);
    var beaconMesh = addMesh(rig, icoGeometry(), beacon, { x: 0, y: 1.22, z: 0.31 }, { x: 0.105, y: 0.15, z: 0.035 });
    if (beaconMesh) beaconMesh.rotation.z = Math.PI / 4;
    putBox(rig, coatEdge, 0, 0.76, 0, 0.55, 0.14, 0.34);

    var leftLeg = new THREE.Group();
    var rightLeg = new THREE.Group();
    leftLeg.position.set(-0.14, 0.80, 0);
    rightLeg.position.set(0.14, 0.80, 0);
    rig.add(leftLeg, rightLeg);
    [leftLeg, rightLeg].forEach(function (leg, index) {
      addMesh(leg, capsuleGeometry(), coat, { x: 0, y: -0.36, z: 0 }, { x: 0.10, y: 0.31, z: 0.10 });
      putBox(leg, coatEdge, 0, -0.43, 0.02, 0.16, 0.10, 0.13);
      putBox(leg, boot, 0, -0.72, 0.09, 0.13, 0.20, 0.26, index ? -0.06 : 0.06);
      putBox(leg, gold, index ? -0.035 : 0.035, -0.61, 0.19, 0.05, 0.12, 0.045);
    });

    var leftArm = new THREE.Group();
    var rightArm = new THREE.Group();
    leftArm.position.set(-0.34, 1.25, 0);
    rightArm.position.set(0.34, 1.25, 0);
    rig.add(leftArm, rightArm);
    [leftArm, rightArm].forEach(function (arm) {
      addMesh(arm, capsuleGeometry(), coat, { x: 0, y: -0.18, z: 0 }, { x: 0.09, y: 0.22, z: 0.09 });
      addMesh(arm, sphereGeometry(), boot, { x: 0, y: -0.43, z: 0 }, { x: 0.095, y: 0.10, z: 0.095 });
    });
    var weapon = putBox(rightArm, beacon, 0, -0.56, 0.16, 0.035, 0.40, 0.035);
    if (weapon) weapon.rotation.z = -0.22;
    var lamp = addMesh(rig, sphereGeometry(), beacon, { x: 0, y: 1.22, z: 0.33 }, { x: 0.035, y: 0.035, z: 0.035 });
    var health = addHealthBar(root, "player");
    var shadow = addMesh(root, cylinderGeometry(), material("cenyao-shadow", 0x05080b, { basic: true, transparent: true, opacity: 0.45 }), { x: 0, y: 0.025, z: 0 }, { x: 0.38, y: 0.012, z: 0.22 });
    if (shadow) shadow.rotation.x = 0;
    root.userData.rig = rig;
    root.userData.leftLeg = leftLeg;
    root.userData.rightLeg = rightLeg;
    root.userData.leftArm = leftArm;
    root.userData.rightArm = rightArm;
    root.userData.weapon = weapon;
    root.userData.health = health;
    root.userData.lamp = lamp;
    root.userData.kind = "player";
    root.userData.palette = 0;
    root.scale.set(0.92, 1.10, 0.92);
    return root;
  }

  function makeActorModel(kind, item) {
    if (kind === "player") return makeCenYaoModel();
    var palette = actorPalette(kind, item);
    var root = new THREE.Group();
    var rig = new THREE.Group();
    var coat = materialForActor(kind, palette);
    var darker = material("actor-dark-" + kind + "-" + palette, (theme.floorEdge + 0x101010) & 0xffffff, { roughness: 0.72, metalness: 0.28 });
    var skin = material("actor-skin-" + kind, kind === "enemy" ? 0x6e4c4d : 0xc4a48c, { roughness: 0.66, metalness: 0.08 });
    var visor = material("actor-visor-" + kind, kind === "enemy" ? theme.warning : theme.accent, { basic: true, transparent: true, opacity: 0.9, emissive: theme.accent, emissiveIntensity: 1.0 });
    var boot = material("actor-boot-" + kind, 0x171d22, { roughness: 0.58, metalness: 0.54 });
    var helmet = material("actor-helmet-" + kind, kind === "enemy" ? theme.wall[0] : theme.trim, { roughness: 0.40, metalness: 0.72 });

    rig.position.y = 0.04;
    root.add(rig);
    addMesh(rig, capsuleGeometry(), coat, { x: 0, y: 1.03, z: 0 }, { x: 0.34, y: 0.43, z: 0.27 });
    addMesh(rig, cylinderGeometry(), darker, { x: 0, y: 0.72, z: 0 }, { x: 0.39, y: 0.12, z: 0.31 });
    addMesh(rig, sphereGeometry(), skin, { x: 0, y: 1.64, z: 0 }, { x: 0.245, y: 0.28, z: 0.235 });
    addMesh(rig, sphereGeometry(), helmet, { x: 0, y: 1.79, z: 0 }, { x: 0.30, y: 0.18, z: 0.29 });
    putRing(rig, helmet, 0, 1.71, 0, 0.31, -Math.PI / 2);
    putBox(rig, visor, 0, 1.70, 0.22, 0.23, 0.065, 0.025);
    putBox(rig, darker, 0, 1.10, -0.30, 0.29, 0.42, 0.16);
    putBox(rig, darker, 0, 1.29, -0.42, 0.22, 0.23, 0.12);
    putCylinder(rig, visor, 0, 2.00, 0.02, 0.045, 0.08);

    var leftLeg = new THREE.Group();
    var rightLeg = new THREE.Group();
    // The lowest boot face is rig .04 + leg .79 - .78 - .05 = y 0.
    // Keeping the actor root at the integer tile center makes feet and the
    // ground contact shadow share exactly the same grid anchor.
    leftLeg.position.set(-0.16, 0.79, 0);
    rightLeg.position.set(0.16, 0.79, 0);
    rig.add(leftLeg, rightLeg);
    [leftLeg, rightLeg].forEach(function (leg) {
      addMesh(leg, cylinderGeometry(), darker, { x: 0, y: -0.22, z: 0 }, { x: 0.095, y: 0.23, z: 0.095 });
      addMesh(leg, sphereGeometry(), coat, { x: 0, y: -0.45, z: 0 }, { x: 0.115, y: 0.115, z: 0.115 });
      addMesh(leg, cylinderGeometry(), boot, { x: 0, y: -0.64, z: 0 }, { x: 0.08, y: 0.19, z: 0.08 });
      addMesh(leg, boxGeometry(), boot, { x: 0, y: -0.78, z: 0.10 }, { x: 0.14, y: 0.10, z: 0.26 });
    });

    var leftArm = new THREE.Group();
    var rightArm = new THREE.Group();
    leftArm.position.set(-0.37, 1.22, 0);
    rightArm.position.set(0.37, 1.22, 0);
    rig.add(leftArm, rightArm);
    [leftArm, rightArm].forEach(function (arm) {
      addMesh(arm, sphereGeometry(), coat, { x: 0, y: 0, z: 0 }, { x: 0.13, y: 0.13, z: 0.13 });
      addMesh(arm, cylinderGeometry(), coat, { x: 0, y: -0.20, z: 0 }, { x: 0.075, y: 0.20, z: 0.075 });
      addMesh(arm, sphereGeometry(), skin, { x: 0, y: -0.43, z: 0 }, { x: 0.09, y: 0.09, z: 0.09 });
    });
    var weapon = putBox(rightArm, visor, 0, -0.50, 0.18, 0.035, 0.42, 0.035);
    if (weapon) weapon.rotation.z = -0.22;
    var lamp = addMesh(rig, sphereGeometry(), material("actor-lamp-" + kind, theme.lamp, { basic: true, emissive: theme.lamp, emissiveIntensity: 1.8 }), { x: 0, y: 1.95, z: 0.04 }, { x: 0.045, y: 0.045, z: 0.045 });
    if (kind === "enemy") {
      addMesh(rig, icoGeometry(), visor, { x: -0.14, y: 1.63, z: 0.19 }, { x: 0.055, y: 0.055, z: 0.055 });
      addMesh(rig, icoGeometry(), visor, { x: 0.14, y: 1.63, z: 0.19 }, { x: 0.055, y: 0.055, z: 0.055 });
    }
    var health = addHealthBar(root, kind);
    var shadow = addMesh(root, cylinderGeometry(), material("actor-shadow", 0x05080b, { basic: true, transparent: true, opacity: 0.45 }), { x: 0, y: 0.025, z: 0 }, { x: 0.42, y: 0.012, z: 0.24 });
    if (shadow) shadow.rotation.x = 0;
    root.userData.rig = rig;
    root.userData.leftLeg = leftLeg;
    root.userData.rightLeg = rightLeg;
    root.userData.leftArm = leftArm;
    root.userData.rightArm = rightArm;
    root.userData.weapon = weapon;
    root.userData.health = health;
    root.userData.lamp = lamp;
    root.userData.kind = kind;
    root.userData.palette = palette;
    root.userData.sharedCampaign = true;
    var scale = kind === "enemy" ? (palette === 2 ? 1.12 : 0.96) : kind === "npc" ? 0.98 : 1;
    root.scale.setScalar(scale);
    return root;
  }

  function recordPoint(record) {
    return {
      x: num(record.target.x, 0),
      y: num(record.target.y, 0),
    };
  }

  function updateRecordData(record, item) {
    if (!record || !item) return;
    var nx = Math.round(num(item.x, record.target.x));
    var ny = Math.round(num(item.y, record.target.y));
    if (record.target.x !== nx || record.target.y !== ny) {
      record.lastTarget = { x: record.target.x, y: record.target.y };
      record.target = { x: nx, y: ny };
      var dx = nx - record.current.x;
      var dy = ny - record.current.y;
      if (Math.abs(dx) + Math.abs(dy) > 0.01) record.heading = Math.atan2(dx, dy);
    }
    record.data = item;
    updateHealthBar(record);
  }

  function updateHealthBar(record) {
    if (!record || !record.model || !record.model.userData.health) return;
    var data = record.kind === "player" ? playerState : record.data || {};
    var hp = num(data.hp, NaN);
    var max = num(data.max_hp, num(data.maxHp, NaN));
    var visible = isFinite(hp) && isFinite(max) && max > 0;
    var bar = record.model.userData.health;
    bar.visible = visible;
    if (!visible) return;
    var ratio = clamp(hp / max, 0, 1);
    var fill = bar.userData.fill;
    if (fill) {
      fill.scale.x = 0.88 * ratio;
      fill.position.x = -0.44 * (1 - ratio);
    }
  }

  function createRecord(id, kind, item) {
    var x = Math.round(num(item && item.x, kind === "player" ? num(map && map.px, 0) : 0));
    var y = Math.round(num(item && item.y, kind === "player" ? num(map && map.py, 0) : 0));
    var model = (!fallback && actorRoot && THREE) ? makeActorModel(kind, item || {}) : null;
    var record = {
      id: id,
      kind: kind,
      data: item || {},
      current: { x: x, y: y },
      target: { x: x, y: y },
      heading: 0,
      phase: (hash(x, y, id.length + kind.length) % 1000) / 1000 * TAU,
      swing: 0,
      model: model,
    };
    if (record.model) {
      record.model.position.set(x * UNIT, 0, y * UNIT);
      record.model.rotation.y = record.heading;
      record.model.userData.record = record;
      actorRoot.add(record.model);
    }
    updateHealthBar(record);
    return record;
  }

  function ensureRecordModel(record) {
    if (!record || record.model || fallback || !actorRoot || !THREE) return;
    record.model = makeActorModel(record.kind, record.data || {});
    record.model.position.set(record.current.x * UNIT, 0, record.current.y * UNIT);
    record.model.rotation.y = record.heading;
    record.model.userData.record = record;
    actorRoot.add(record.model);
    updateHealthBar(record);
  }

  function updateOrCreateRecord(id, kind, item) {
    var record = actorRecords[id];
    if (!record || record.kind !== kind) {
      if (record && record.model && actorRoot) actorRoot.remove(record.model);
      record = createRecord(id, kind, item || {});
      actorRecords[id] = record;
    }
    updateRecordData(record, item || {});
    return record;
  }

  function clearStaleEnemyEffects(aliveRecords) {
    if (!effects.length && !pendingEffects.length) return;
    var nextEffects = [];
    effects.forEach(function (effect) {
      if (effect.enemyId && !aliveRecords[effect.enemyId]) disposeEffect(effect);
      else nextEffects.push(effect);
    });
    effects = nextEffects;
    pendingEffects = pendingEffects.filter(function (effect) {
      return !effect.enemy_id && !effect.enemyId || aliveRecords[effect.enemy_id || effect.enemyId];
    });
  }

  function syncActors() {
    if (!map) return;
    var keep = Object.create(null);
    var px = Math.round(num(map.px, 0));
    var py = Math.round(num(map.py, 0));
    if (!playerRecord) {
      playerRecord = updateOrCreateRecord("player", "player", { x: px, y: py, hp: playerState.hp, max_hp: playerState.max_hp });
    } else {
      updateRecordData(playerRecord, { x: px, y: py, hp: playerState.hp, max_hp: playerState.max_hp });
    }
    ensureRecordModel(playerRecord);
    playerRecord.data = playerState;
    updateHealthBar(playerRecord);
    keep.player = true;

    (map.enemies || []).forEach(function (enemy, index) {
      if (enemy.alive === false) return;
      var id = actorId(enemy, "enemy", index);
      keep[id] = true;
      ensureRecordModel(updateOrCreateRecord(id, "enemy", enemy));
    });
    (map.npcs || []).forEach(function (npc, index) {
      var id = actorId(npc, "npc", index);
      keep[id] = true;
      ensureRecordModel(updateOrCreateRecord(id, "npc", npc));
    });
    Object.keys(actorRecords).forEach(function (id) {
      if (keep[id]) return;
      var record = actorRecords[id];
      if (record && record.model && actorRoot) actorRoot.remove(record.model);
      delete actorRecords[id];
    });
    clearStaleEnemyEffects(keep);
  }

  function refreshCutaway(force) {
    if (!wallCells.length || !playerRecord) return;
    var cell = Math.round(playerRecord.current.x) + ":" + Math.round(playerRecord.current.y);
    if (!force && cell === lastCutawayCell) return;
    lastCutawayCell = cell;
    var matrix = new THREE.Matrix4();
    var identity = new THREE.Quaternion();
    var scale = new THREE.Vector3(1, 1, 1);
    wallCells.forEach(function (entry) {
      var distance = Math.abs(entry.x - playerRecord.current.x) + Math.abs(entry.y - playerRecord.current.y);
      var front = entry.x + entry.y > playerRecord.current.x + playerRecord.current.y + 0.4;
      var height = front && distance < 8 ? 0.40 : 0.76;
      scale.set(1, height / 0.76, 1);
      matrix.compose(new THREE.Vector3(entry.x, height * 0.5, entry.y), identity, scale);
      entry.instance.setMatrixAt(entry.index, matrix);
    });
    var updated = [];
    wallCells.forEach(function (entry) {
      if (updated.indexOf(entry.instance) >= 0) return;
      updated.push(entry.instance);
      entry.instance.instanceMatrix.needsUpdate = true;
    });
  }

  function updateActorMotion(now, dt) {
    var records = Object.keys(actorRecords).map(function (id) { return actorRecords[id]; });
    records.forEach(function (record) {
      var ease = 1 - Math.exp(-Math.min(0.08, dt) * 12);
      record.current.x += (record.target.x - record.current.x) * ease;
      record.current.y += (record.target.y - record.current.y) * ease;
      if (record.model) record.model.position.set(record.current.x * UNIT, 0, record.current.y * UNIT);
      var moving = Math.abs(record.target.x - record.current.x) + Math.abs(record.target.y - record.current.y) > 0.018;
      var speed = record.kind === "player" ? 0.012 : 0.009;
      var phase = now * speed + record.phase;
      var stride = moving ? Math.sin(phase) * 0.62 : Math.sin(now * 0.002 + record.phase) * 0.045;
      var rig = record.model && record.model.userData.rig;
      if (rig) {
        record.model.userData.leftLeg.rotation.x = stride;
        record.model.userData.rightLeg.rotation.x = -stride;
        record.model.userData.leftArm.rotation.x = -stride * 0.55;
        record.model.userData.rightArm.rotation.x = stride * 0.55;
        rig.position.y = 0.04 + (moving ? Math.abs(Math.sin(phase)) * 0.035 : 0);
      }
      if (record.swing > 0 && record.model) {
        record.swing = Math.max(0, record.swing - dt);
        var swingT = 1 - record.swing / 0.34;
        var swingCurve = Math.sin(clamp(swingT, 0, 1) * Math.PI);
        record.model.userData.rightArm.rotation.x -= swingCurve * 1.55;
        record.model.userData.rightArm.rotation.z = -0.16 - swingCurve * 0.28;
        if (record.model.userData.weapon) record.model.userData.weapon.rotation.z = -0.22 - swingCurve * 0.36;
      } else {
        record.model.userData.rightArm.rotation.z = 0;
        if (record.model.userData.weapon) record.model.userData.weapon.rotation.z = -0.22;
      }
      if (record.model) record.model.rotation.y += (record.heading - record.model.rotation.y) * Math.min(1, dt * 12);
      updateHealthBar(record);
    });
    refreshCutaway(false);
  }

  function updateCamera() {
    if (!camera || !playerRecord) return;
    var wx = playerRecord.current.x * UNIT;
    var wz = playerRecord.current.y * UNIT;
    camera.position.set(wx + 8.1, 10.7, wz + 8.1);
    camera.lookAt(wx, 0, wz);
  }

  function updateDecorations(now) {
    dynamicMarkers.forEach(function (group) {
      var phase = num(group.userData.phase, 0);
      var pulse = 0.86 + Math.sin(now * 0.003 + phase) * 0.14;
      group.scale.setScalar(pulse);
      if (group.userData.marker && group.userData.marker.done) group.visible = pulse > 0.75;
    });
    interactionMarkers.forEach(function (group) {
      var pulse = 0.92 + Math.sin(now * 0.004 + num(group.userData.phase, 0)) * 0.12;
      group.scale.setScalar(pulse);
    });
    hazardMarkers.forEach(function (group) {
      var zone = group.userData.zone || {};
      var pulse = 1 + Math.sin(now * 0.006 + num(group.userData.phase, 0)) * (group.userData.telegraph ? 0.10 : 0.04);
      group.scale.setScalar(pulse);
      group.visible = zone.remaining == null || num(zone.remaining, 0) > 0;
    });
  }

  function effectPosition(effect) {
    var x = num(effect.x, playerRecord ? playerRecord.current.x : 0);
    var y = num(effect.y, playerRecord ? playerRecord.current.y : 0);
    return gridWorld(x, y, 0.08);
  }

  function supportedEffectKind(kind) {
    var value = normalizeKind(kind);
    if (["attack", "pulse", "heal", "dash", "shot"].indexOf(value) >= 0) return value;
    if (value === "skill0") return "skill_0";
    if (value === "skill1") return "skill_1";
    if (value === "skill2") return "skill_2";
    // Skills/casts are represented by the restrained pulse treatment.  Keep
    // the allow-list explicit so movement telemetry and future unknown event
    // kinds can never become visible particle blobs by accident.
    if (["skill", "ability", "cast"].indexOf(value) >= 0) return "pulse";
    return null;
  }

  // Manual smoke check: in a live campaign cast Q/E/R once each, wait for
  // the effect to finish, then repeat the same key. Q must show three rings,
  // E must keep its hex dome visible for about one second, and R must show a
  // directional beam ending in a burst. Repeat with WebGL disabled to verify
  // the corresponding canvas fallback drawings; after each cast debug().effects
  // must return to zero. This is intentionally kept beside the allow-list so
  // backend skill_0/skill_1/skill_2 changes remain easy to verify.

  function effectDuration(kind) {
    if (kind === "skill_0") return 1.10;
    if (kind === "skill_1") return 1.28;
    if (kind === "skill_2") return 0.90;
    return kind === "dash" ? 0.42 : kind === "attack" ? 0.48 : kind === "shot" ? 0.36 : 0.76;
  }

  function addEffectMesh(effect, geo, mat, pos, scale) {
    var mesh = addMesh(effect.group, geo, mat, pos, scale);
    if (mesh) {
      // Keep the authored particle dimensions separate from animation.  A
      // generic shrink/fade must never reset a 0.05m particle to unit scale.
      mesh.userData.effectBaseScale = { x: mesh.scale.x, y: mesh.scale.y, z: mesh.scale.z };
      mesh.renderOrder = 70;
      effect.meshes.push(mesh);
      if (geo && !(geo.userData && geo.userData.sharedCampaign)) {
        effect.geometries.push(geo);
      }
    }
    return mesh;
  }

  function createEffect(effect) {
    if (!THREE || !effectRoot) return null;
    var kind = supportedEffectKind(effect && effect.kind);
    if (!kind) return null;
    var origin = effectPosition(effect);
    var duration = effectDuration(kind);
    var color = kind === "skill_0" ? 0x71d9ff : kind === "skill_1" ? 0xb28cff : kind === "skill_2" ? 0xffcf68 : kind === "heal" ? 0x73e7ae : kind === "shot" ? theme.accent : kind === "attack" ? theme.warning : kind === "dash" ? theme.trim : theme.accent;
    var created = {
      kind: kind,
      enemyId: effect.enemy_id || effect.enemyId || null,
      x: origin.x,
      y: origin.z,
      dx: num(effect.dx, 1),
      dy: num(effect.dy, 0),
      started: 0,
      duration: duration,
      meshes: [],
      lines: [],
      geometries: [],
      group: new THREE.Group(),
    };
    created.group.position.set(origin.x, 0, origin.z);
    created.group.userData.effect = true;
    effectRoot.add(created.group);
    var mat = effectMaterial(color, 0.88, false);
    created.material = mat;
    var ringMat = effectMaterial(color, 0.72, false);
    if (kind === "skill_0") {
      // Q: thick horizontal rings start large enough to be readable in the
      // very first captured frame, then expand into a restrained 2.4m wave.
      for (var q = 0; q < 3; q += 1) {
        var qRing = addEffectMesh(created, new THREE.TorusGeometry(0.5, 0.10 - q * 0.012, 8, 32), effectMaterial(q === 1 ? 0xb9f4ff : color, 0.92 - q * 0.13, false), { x: 0, y: 0.10 + q * 0.055, z: 0 }, { x: 1.15 + q * 0.32, y: 1.15 + q * 0.32, z: 1.15 + q * 0.32 });
        if (qRing) {
          qRing.rotation.x = Math.PI / 2;
          qRing.userData.effectRole = "qRing";
          qRing.userData.effectLayer = q;
        }
      }
      var wave = addEffectMesh(created, new THREE.CylinderGeometry(0.5, 0.5, 0.028, 32, 1, false), effectMaterial(0x2fbce8, 0.34, false), { x: 0, y: 0.055, z: 0 }, { x: 1.05, y: 1, z: 1.05 });
      if (wave) wave.userData.effectRole = "qWave";
      var qParticleCount = quality === "low" ? 7 : 11;
      for (var qp = 0; qp < qParticleCount; qp += 1) {
        var qpAngle = qp * TAU / qParticleCount;
        var pillar = addEffectMesh(created, boxGeometry(), effectMaterial(qp % 2 ? 0xb9f4ff : color, 0.84, false), { x: Math.cos(qpAngle) * 0.72, y: 0.34, z: Math.sin(qpAngle) * 0.72 }, { x: 0.055, y: 0.28, z: 0.055 });
        if (pillar) {
          pillar.userData.effectRole = "qPillar";
          pillar.userData.effectAngle = qpAngle;
          pillar.userData.effectLayer = qp;
        }
      }
    } else if (kind === "skill_1") {
      // E: stacked hexagonal energy shells, plus six vertical facets making
      // the protection read as a dome rather than a flat status ring.
      for (var h = 0; h < 3; h += 1) {
        var shell = addEffectMesh(created, new THREE.TorusGeometry(0.56, 0.045, 6, 6), effectMaterial(h === 1 ? 0xdfcaff : color, 0.55 - h * 0.07, false), { x: 0, y: 0.24 + h * 0.48, z: 0 }, { x: 1.30 - h * 0.16, y: 1.30 - h * 0.16, z: 1.30 - h * 0.16 });
        if (shell) {
          shell.rotation.x = Math.PI / 2;
          shell.userData.effectRole = "shieldRing";
          shell.userData.effectLayer = 10 + h;
        }
      }
      var domeMaterial = effectMaterial(color, 0.18, false);
      if (THREE.DoubleSide != null) domeMaterial.side = THREE.DoubleSide;
      var dome = addEffectMesh(created, new THREE.SphereGeometry(0.80, 12, 6, 0, TAU, 0, Math.PI / 2), domeMaterial, { x: 0, y: 0.05, z: 0 }, { x: 1.18, y: 1.55, z: 1.18 });
      if (dome) dome.userData.effectRole = "shieldDome";
      for (var facet = 0; facet < 6; facet += 1) {
        var angleFacet = facet * TAU / 6;
        var panel = addEffectMesh(created, boxGeometry(), effectMaterial(color, 0.25, false), { x: Math.cos(angleFacet) * 0.54, y: 0.78, z: Math.sin(angleFacet) * 0.54 }, { x: 0.035, y: 0.52, z: 0.27 });
        if (panel) {
          panel.rotation.y = -angleFacet;
          panel.userData.effectRole = "shieldFacet";
          panel.userData.effectLayer = 20 + facet;
        }
      }
    } else if (kind === "skill_2") {
      // R: use solid transparent boxes for width (WebGL ignores lineWidth),
      // with lines retained only as crisp highlights and a trailing filament.
      var beamAngle = Math.atan2(created.dx, created.dy);
      var beamLength = 5.1;
      var beamBody = addEffectMesh(created, boxGeometry(), effectMaterial(color, 0.62, false), { x: created.dx * beamLength * 0.5, y: 0.56, z: created.dy * beamLength * 0.5 }, { x: 0.18, y: 0.11, z: beamLength });
      if (beamBody) {
        beamBody.rotation.y = beamAngle;
        beamBody.userData.effectRole = "beamBody";
      }
      var beamCore = addEffectMesh(created, boxGeometry(), effectMaterial(0xfff5bd, 0.92, false), { x: created.dx * beamLength * 0.5, y: 0.58, z: created.dy * beamLength * 0.5 }, { x: 0.065, y: 0.055, z: beamLength * 1.01 });
      if (beamCore) {
        beamCore.rotation.y = beamAngle;
        beamCore.userData.effectRole = "beamCore";
      }
      for (var r = 0; r < 9; r += 1) {
        var ra = r * TAU / 9;
        var impact = addEffectMesh(created, icoGeometry(), effectMaterial(color, 0.82, false), { x: Math.cos(ra) * 0.16, y: 0.52 + (r % 3) * 0.09, z: Math.sin(ra) * 0.16 }, { x: 0.06, y: 0.06, z: 0.06 });
        if (impact) {
          impact.userData.effectRole = "impact";
          impact.userData.effectLayer = 30 + r;
        }
      }
    } else if (kind === "pulse" || kind === "attack" || kind === "heal") {
      addEffectMesh(created, ringGeometry(), ringMat, { x: 0, y: 0.055, z: 0 }, { x: 0.46, y: 0.46, z: 0.46 });
    }
    if (["skill_0", "skill_1", "skill_2"].indexOf(kind) < 0) {
      var particleCount = quality === "low" ? 5 : quality === "medium" ? 8 : 11;
      for (var i = 0; i < particleCount; i += 1) {
        var angle = i * TAU / particleCount + (hash(Math.round(origin.x), Math.round(origin.z), i + 19) % 100) * 0.01;
        var radius = 0.20 + (i % 5) * 0.075;
        var particle = addEffectMesh(created, kind === "dash" ? boxGeometry() : icoGeometry(), mat,
          { x: Math.cos(angle) * radius, y: 0.18 + (i % 3) * 0.15, z: Math.sin(angle) * radius },
          { x: kind === "dash" ? 0.035 : 0.052, y: kind === "dash" ? 0.035 : 0.052, z: kind === "dash" ? 0.17 : 0.052 });
        if (particle) particle.userData.effectAngle = angle;
      }
    }
    if (kind === "skill_2" || kind === "shot" || kind === "dash") {
      var lineGeometry = new THREE.BufferGeometry();
      var line = new THREE.Line(lineGeometry, effectMaterial(color, kind === "skill_2" ? 0.95 : 0.9, true));
      line.userData.effectLine = true;
      line.renderOrder = 72;
      effectRoot.add(line);
      created.lines.push(line);
      if (kind === "skill_2") {
        var glowGeometry = new THREE.BufferGeometry();
        var glow = new THREE.Line(glowGeometry, effectMaterial(0xfff0b0, 0.52, true));
        glow.userData.effectLine = true;
        glow.renderOrder = 73;
        effectRoot.add(glow);
        created.lines.push(glow);
        var trailGeometry = new THREE.BufferGeometry();
        var trail = new THREE.Line(trailGeometry, effectMaterial(color, 0.38, true));
        trail.userData.effectLine = true;
        trail.renderOrder = 71;
        effectRoot.add(trail);
        created.lines.push(trail);
      }
    }
    if (kind === "attack" && playerRecord) playerRecord.swing = Math.max(playerRecord.swing, 0.34);
    if (kind === "dash" && playerRecord) playerRecord.swing = Math.max(playerRecord.swing, 0.22);
    return created;
  }

  function createFallbackEffect(effect) {
    var kind = supportedEffectKind(effect && effect.kind);
    if (!kind) return null;
    return {
      kind: kind,
      enemyId: effect.enemy_id || effect.enemyId || null,
      x: num(effect.x, playerRecord ? playerRecord.current.x : 0),
      y: num(effect.y, playerRecord ? playerRecord.current.y : 0),
      dx: num(effect.dx, 1),
      dy: num(effect.dy, 0),
      started: 0,
      duration: effectDuration(kind),
      meshes: [],
      lines: [],
      geometries: [],
      group: null,
    };
  }

  function updateEffectGeometry(line, points) {
    if (!line || !line.geometry || !THREE) return;
    line.geometry.setFromPoints(points);
    line.geometry.computeBoundingSphere();
  }

  function disposeEffect(effect) {
    if (!effect) return;
    if (effect.group && effectRoot) effectRoot.remove(effect.group);
    var releasedMaterials = [];
    function releaseMaterial(mat) {
      if (!mat || !mat.userData || !mat.userData.effectOwned || releasedMaterials.indexOf(mat) >= 0) return;
      releasedMaterials.push(mat);
      if (mat.dispose) mat.dispose();
      var index = ownedMaterials.indexOf(mat);
      if (index >= 0) ownedMaterials.splice(index, 1);
    }
    (effect.lines || []).forEach(function (line) {
      if (effectRoot) effectRoot.remove(line);
      if (line.geometry) line.geometry.dispose();
      releaseMaterial(line.material);
    });
    (effect.geometries || []).forEach(function (geo) {
      if (geo && geo.dispose) geo.dispose();
    });
    (effect.meshes || []).forEach(function (mesh) {
      if (mesh && mesh.material) releaseMaterial(mesh.material);
    });
    releaseMaterial(effect.material);
  }

  function clearEffects() {
    effects.forEach(disposeEffect);
    effects = [];
    pendingEffects = [];
  }

  function clearActorRecords() {
    if (actorRoot) clearGroup(actorRoot);
    actorRecords = Object.create(null);
    playerRecord = null;
  }

  function clearWorldScene() {
    clearEffects();
    clearActorRecords();
    if (topologyRoot) clearGroup(topologyRoot);
    if (staticPropRoot) clearGroup(staticPropRoot);
    if (dynamicDecorationRoot) clearGroup(dynamicDecorationRoot);
    floorInstances = [];
    wallInstances = [];
    wallCells = [];
    dynamicMarkers = [];
    hazardMarkers = [];
    interactionMarkers = [];
    fallbackMarkerData = [];
    fallbackHazardData = [];
    fallbackInteractionData = [];
    decorProps = [];
    baseMesh = null;
    map = null;
    currentWorldId = "hub";
    theme = null;
    topologyKey = "";
    decorationKey = "";
    staticPropKey = "";
    dynamicDecorationKey = "";
    rebuildExploredLookup();
    if (scene) updateTheme(true);
  }

  function clearEffectsForWorldChange() {
    clearEffects();
    clearActorRecords();
    topologyKey = "";
    decorationKey = "";
    staticPropKey = "";
    dynamicDecorationKey = "";
  }

  function updateEffects(now) {
    var next = [];
    effects.forEach(function (effect) {
      if (!effect.started) effect.started = now;
      var t = clamp((now - effect.started) / (effect.duration * 1000), 0, 1);
      var ease = t * t * (3 - 2 * t);
      var centerX = effect.x;
      var centerZ = effect.y;
      if (effect.kind === "skill_0") {
        effect.meshes.forEach(function (mesh) {
          var role = mesh.userData.effectRole;
          var layer = num(mesh.userData.effectLayer, 0);
          var base = mesh.userData.effectBaseScale || { x: 1, y: 1, z: 1 };
          if (role === "qRing") {
            var delayed = clamp((t - layer * 0.055) / 0.80, 0, 1);
            var expansion = 1 + delayed * (2.35 + layer * 0.16);
            mesh.scale.set(base.x * expansion, base.y * expansion, base.z * expansion);
            mesh.rotation.y = now * (0.001 + layer * 0.00035);
            mesh.material.opacity = (0.92 - layer * 0.13) * (1 - clamp((t - 0.62) / 0.38, 0, 1));
          } else if (role === "qWave") {
            var waveScale = 1 + ease * 3.55;
            mesh.scale.set(base.x * waveScale, base.y, base.z * waveScale);
            mesh.material.opacity = 0.34 * (1 - t);
          } else if (role === "qPillar") {
            var pa = num(mesh.userData.effectAngle, 0);
            var outward = 0.72 + ease * 1.70;
            mesh.position.set(Math.cos(pa) * outward, 0.30 + Math.sin(Math.PI * t) * (0.65 + (layer % 3) * 0.14), Math.sin(pa) * outward);
            mesh.scale.set(base.x, base.y * (1.0 + Math.sin(Math.PI * t) * 1.4), base.z);
            mesh.material.opacity = 0.84 * (1 - clamp((t - 0.66) / 0.34, 0, 1));
          }
        });
      } else if (effect.kind === "skill_1") {
        effect.meshes.forEach(function (mesh) {
          var role = mesh.userData.effectRole;
          var layer = num(mesh.userData.effectLayer, 0);
          var fade = 1 - clamp((t - 0.72) / 0.28, 0, 1);
          var shimmer = 0.92 + Math.sin(now * 0.008 + layer) * 0.08;
          mesh.material.opacity = (role === "shieldDome" ? 0.18 : role === "shieldRing" ? 0.55 : 0.30) * fade * shimmer;
          if (role === "shieldRing") mesh.rotation.y += 0.006 * (layer % 2 ? 1 : -1);
          if (role === "shieldFacet") mesh.position.y = 0.78 + Math.sin(now * 0.006 + layer) * 0.06;
        });
      } else if (effect.kind === "skill_2") {
        effect.meshes.forEach(function (mesh, index) {
          var role = mesh.userData.effectRole;
          if (role === "beamBody" || role === "beamCore") {
            // 0.74 * 0.90s = 0.666s at full authored opacity, leaving a
            // short fade tail that survives delayed screenshots without
            // changing any gameplay or damage timing.
            var beamFade = 1 - clamp((t - 0.74) / 0.26, 0, 1);
            mesh.material.opacity = (role === "beamCore" ? 0.92 : 0.62) * beamFade;
            var beamBase = mesh.userData.effectBaseScale || { x: 1, y: 1, z: 1 };
            mesh.scale.set(beamBase.x * (1 + Math.sin(Math.PI * t) * 0.34), beamBase.y, beamBase.z);
            return;
          }
          var layer = num(mesh.userData.effectLayer, index);
          var burst = clamp((t - 0.52) / 0.48, 0, 1);
          var angle = layer * TAU / 9;
          var distance = burst * (0.50 + (layer % 3) * 0.18);
          var hitX = num(effect.dx, 1) * 5.1 * (0.90 + t * 0.10);
          var hitZ = num(effect.dy, 0) * 5.1 * (0.90 + t * 0.10);
          mesh.position.set(hitX + Math.cos(angle) * distance, 0.50 + (layer % 3) * 0.11, hitZ + Math.sin(angle) * distance);
          mesh.scale.setScalar(0.06 + burst * 0.08);
        });
      } else if (effect.kind === "pulse" || effect.kind === "attack" || effect.kind === "heal") {
        effect.meshes.forEach(function (mesh, index) {
          if (index === 0) {
            var ringScale = 0.45 + ease * (effect.kind === "pulse" ? 1.8 : 1.0);
            mesh.scale.set(ringScale, ringScale, ringScale);
            mesh.rotation.z = now * 0.0015;
          } else {
            var angle = num(mesh.userData.effectAngle, index);
            var spread = effect.kind === "heal" ? 0.42 : 0.80;
            var distance = 0.16 + ease * (0.48 + (index % 4) * spread * 0.12);
            mesh.position.set(Math.cos(angle) * distance, 0.14 + ease * (0.70 + (index % 3) * 0.16), Math.sin(angle) * distance);
            mesh.rotation.x += 0.08;
            mesh.rotation.y += 0.12;
          }
        });
      } else if (effect.kind === "dash") {
        effect.meshes.forEach(function (mesh, index) {
          var angle = num(mesh.userData.effectAngle, index);
          var distance = -ease * (0.55 + (index % 4) * 0.22);
          mesh.position.set(Math.cos(angle) * distance, 0.12 + (index % 3) * 0.10, Math.sin(angle) * distance);
          mesh.rotation.y += 0.20;
        });
      } else {
        effect.meshes.forEach(function (mesh, index) {
          mesh.position.y += 0.004 + index * 0.0002;
          var base = mesh.userData.effectBaseScale || { x: 1, y: 1, z: 1 };
          var shrink = 1 - ease * 0.4;
          mesh.scale.set(base.x * shrink, base.y * shrink, base.z * shrink);
        });
      }
      if (effect.lines.length) {
        var dx = num(effect.dx, 1);
        var dy = num(effect.dy, 0);
        var length = effect.kind === "skill_2" ? 5.1 : effect.kind === "dash" ? 1.2 : 4.2;
        var start = new THREE.Vector3(centerX - dx * 0.18, 0.55, centerZ - dy * 0.18);
        var end = new THREE.Vector3(centerX + dx * length, 0.55, centerZ + dy * length);
        var mid = start.clone().lerp(end, 0.5);
        updateEffectGeometry(effect.lines[0], [start, mid, end]);
        if (effect.kind === "skill_2" && effect.lines.length >= 3) {
          var beamStart = new THREE.Vector3(centerX - dx * 0.10, 0.62, centerZ - dy * 0.10);
          var beamEnd = new THREE.Vector3(centerX + dx * length * (0.90 + t * 0.10), 0.62, centerZ + dy * length * (0.90 + t * 0.10));
          updateEffectGeometry(effect.lines[1], [beamStart, beamEnd]);
          var trailStart = new THREE.Vector3(centerX - dx * 0.70, 0.46, centerZ - dy * 0.70);
          updateEffectGeometry(effect.lines[2], [trailStart, start]);
        }
      }
      if (t < 1) next.push(effect);
      else disposeEffect(effect);
    });
    effects = next;
  }

  function flushPendingEffects() {
    if (!pendingEffects.length || (!effectRoot && !fallback)) return;
    pendingEffects.splice(0).forEach(function (effect) {
      var created = fallback ? createFallbackEffect(effect) : createEffect(effect);
      if (created) effects.push(created);
    });
  }

  function currentPlayerCell() {
    if (playerRecord) return { x: Math.round(playerRecord.current.x), y: Math.round(playerRecord.current.y) };
    return { x: Math.round(num(map && map.px, 0)), y: Math.round(num(map && map.py, 0)) };
  }

  function canvasRect() {
    if (!canvas || !canvas.getBoundingClientRect) return { left: 0, top: 0, width: logicalWidth, height: logicalHeight };
    var rect = canvas.getBoundingClientRect();
    return { left: rect.left, top: rect.top, width: Math.max(1, rect.width), height: Math.max(1, rect.height) };
  }

  function targetAt(clientX, clientY) {
    if (!canvas || !map) return null;
    var rect = canvasRect();
    var nx = (num(clientX, rect.left) - rect.left) / rect.width;
    var ny = (num(clientY, rect.top) - rect.top) / rect.height;
    if (fallback || !raycaster || !camera || !THREE) return fallbackTargetAt(nx, ny);
    pointer.set(nx * 2 - 1, -(ny * 2 - 1));
    raycaster.setFromCamera(pointer, camera);
    var hit = new THREE.Vector3();
    if (!raycaster.ray.intersectPlane(groundPlane, hit)) return null;
    // Grid coordinates are centers.  Deliberately use round, never floor, so
    // a click on either side of a cell's center chooses the nearest tile.
    var x = Math.round(hit.x / UNIT);
    var y = Math.round(hit.z / UNIT);
    if (x < 0 || y < 0 || x >= map.w || y >= map.h) return null;
    return { x: x, y: y };
  }

  function fallbackProjection(gx, gy, elevation) {
    var center = currentPlayerCell();
    var dx = num(gx, 0) - center.x;
    var dy = num(gy, 0) - center.y;
    return {
      x: logicalWidth * 0.5 + (dx - dy) * fallbackView.tileW,
      y: logicalHeight * 0.5 + (dx + dy) * fallbackView.tileH - num(elevation, 0),
    };
  }

  function fallbackTargetAt(nx, ny) {
    var center = currentPlayerCell();
    var sx = nx * logicalWidth;
    var sy = ny * logicalHeight;
    var u = (sx - logicalWidth * 0.5) / fallbackView.tileW;
    var v = (sy - logicalHeight * 0.5) / fallbackView.tileH;
    var x = Math.round(center.x + (u + v) * 0.5);
    var y = Math.round(center.y + (v - u) * 0.5);
    if (x < 0 || y < 0 || x >= map.w || y >= map.h) return null;
    return { x: x, y: y };
  }

  function quadPath(context, points) {
    context.beginPath();
    context.moveTo(points[0].x, points[0].y);
    for (var i = 1; i < points.length; i += 1) context.lineTo(points[i].x, points[i].y);
    context.closePath();
  }

  function fallbackCellQuad(x, y, elevation) {
    return [
      fallbackProjection(x - 0.5, y - 0.5, elevation),
      fallbackProjection(x + 0.5, y - 0.5, elevation),
      fallbackProjection(x + 0.5, y + 0.5, elevation),
      fallbackProjection(x - 0.5, y + 0.5, elevation),
    ];
  }

  function rgba(hex, alpha) {
    return "rgba(" + ((hex >> 16) & 255) + "," + ((hex >> 8) & 255) + "," + (hex & 255) + "," + alpha + ")";
  }

  function shadeHex(hex, factor) {
    var r = clamp(Math.round(((hex >> 16) & 255) * factor), 0, 255);
    var g = clamp(Math.round(((hex >> 8) & 255) * factor), 0, 255);
    var b = clamp(Math.round((hex & 255) * factor), 0, 255);
    return (r << 16) | (g << 8) | b;
  }

  function drawFallbackFloor(context, x, y, tile) {
    var quad = fallbackCellQuad(x, y, 0);
    var color = tile === "~" ? theme.water : theme.floor[floorVariantForCell(x, y)];
    if (!isExploredCell(x, y)) color = shadeHex(color, tile === "~" ? 0.70 : 0.66);
    quadPath(context, quad);
    context.fillStyle = "#" + colorToHex(color);
    context.fill();
    context.strokeStyle = rgba(theme.floorEdge, tile === "~" ? 0.20 : 0.34);
    context.lineWidth = 1;
    context.stroke();
    if (tile === "~" && hash(x, y, 131) % 5 === 0) {
      var mid = fallbackProjection(x + 0.1, y + 0.08, 1);
      context.strokeStyle = rgba(theme.trim, 0.34);
      context.beginPath();
      context.moveTo(mid.x - fallbackView.tileW * 0.25, mid.y);
      context.lineTo(mid.x + fallbackView.tileW * 0.25, mid.y);
      context.stroke();
    }
  }

  function drawFallbackWall(context, x, y) {
    var low = Math.max(4, fallbackView.tileH * 0.44);
    var top = fallbackCellQuad(x, y, low);
    var ground = fallbackCellQuad(x, y, 0);
    quadPath(context, [ground[0], ground[1], top[1], top[0]]);
    var wallColor = theme.wall[hash(x, y, 31) % theme.wall.length];
    if (!isExploredCell(x, y)) wallColor = shadeHex(wallColor, 0.74);
    context.fillStyle = "#" + colorToHex(wallColor);
    context.fill();
    quadPath(context, [ground[1], ground[2], top[2], top[1]]);
    context.fillStyle = rgba(theme.floorEdge, 0.78);
    context.fill();
    quadPath(context, top);
    context.fillStyle = rgba(theme.wall[1], 0.96);
    context.fill();
    context.strokeStyle = rgba(theme.floorEdge, 0.72);
    context.stroke();
  }

  function drawFallbackRing(context, x, y, radius, color, alpha, phase) {
    var p = fallbackProjection(x, y, 2);
    var pulse = 1 + Math.sin(performanceNow() * 0.004 + num(phase, 0)) * 0.05;
    context.save();
    context.strokeStyle = rgba(color, alpha);
    context.lineWidth = 2;
    context.setLineDash([5, 4]);
    context.beginPath();
    context.ellipse(p.x, p.y, radius * fallbackView.tileW * 0.76 * pulse, radius * fallbackView.tileH * 0.72 * pulse, 0, 0, TAU);
    context.stroke();
    context.restore();
  }

  function drawFallbackProp(context, prop) {
    var category = propCategory(prop.kind);
    var p = fallbackProjection(prop.x, prop.y, 4);
    var size = propSize(prop, category);
    var sx = size.w * fallbackView.tileW * 0.30;
    var sy = size.h * fallbackView.tileH * 0.60;
    context.save();
    context.translate(p.x, p.y);
    context.strokeStyle = rgba(theme.floorEdge, 0.85);
    context.lineWidth = Math.max(1, fallbackView.scale * 0.055);
    context.fillStyle = rgba(theme.wall[1], 0.92);
    context.rotate(num(prop.rotation, 0) * Math.PI / 180);
    if (category === "rack") {
      var rackCount = size.w >= 3 ? 3 : 2;
      var rackGap = sx * 0.16;
      var rackWidth = Math.min(sx * 0.46, (sx * 1.58 - rackGap * (rackCount - 1)) / rackCount);
      for (var rackIndex = 0; rackIndex < rackCount; rackIndex += 1) {
        var rackX = (rackIndex - (rackCount - 1) * 0.5) * (rackWidth + rackGap);
        context.fillStyle = rgba(theme.floorEdge, 0.96);
        context.fillRect(rackX - rackWidth * 0.5, -sy * 1.48, rackWidth, sy * 1.38);
        context.fillStyle = rgba(rackIndex % 2 ? theme.trim : theme.wall[1], 0.90);
        context.fillRect(rackX - rackWidth * 0.37, -sy * 1.38, rackWidth * 0.74, sy * 1.10);
        context.strokeStyle = rgba(theme.background, 0.92);
        for (var ventIndex = 0; ventIndex < 4; ventIndex += 1) {
          context.beginPath();
          context.moveTo(rackX - rackWidth * 0.25, -sy * 1.27 + ventIndex * sy * 0.19);
          context.lineTo(rackX + rackWidth * 0.25, -sy * 1.27 + ventIndex * sy * 0.19);
          context.stroke();
        }
        context.fillStyle = rgba(theme.lamp, 0.94);
        context.beginPath();
        context.arc(rackX - rackWidth * 0.25, -sy * 1.34, Math.max(2, sx * 0.08), 0, TAU);
        context.fill();
      }
    } else if (category === "lamp") {
      context.strokeStyle = rgba(theme.trim, 0.96);
      context.beginPath();
      context.moveTo(-sx * 0.40, 0);
      context.lineTo(-sx * 0.40, -sy * 1.88);
      context.moveTo(sx * 0.40, 0);
      context.lineTo(sx * 0.40, -sy * 1.88);
      context.moveTo(-sx * 0.40, -sy * 1.88);
      context.lineTo(sx * 0.40, -sy * 1.88);
      context.stroke();
      context.fillStyle = rgba(theme.lamp, 0.98);
      [-0.40, 0.40, 0].forEach(function (lampX) {
        context.beginPath();
        context.arc(sx * lampX, -sy * 1.86, Math.max(2, sx * 0.12), 0, TAU);
        context.fill();
      });
    } else if (category === "console") {
      context.save();
      context.translate(0, -sy * 0.40);
      context.rotate(-0.22);
      context.fillStyle = rgba(theme.floorEdge, 0.96);
      context.fillRect(-sx * 0.78, -sy * 0.22, sx * 1.56, sy * 0.64);
      context.fillStyle = rgba(theme.accent, 0.88);
      context.fillRect(-sx * 0.54, -sy * 0.14, sx * 1.08, sy * 0.20);
      context.fillStyle = rgba(theme.lamp, 0.92);
      context.fillRect(-sx * 0.38, sy * 0.18, sx * 0.30, sy * 0.06);
      context.fillRect(sx * 0.16, sy * 0.18, sx * 0.18, sy * 0.06);
      context.restore();
    } else if (category === "warehouse") {
      context.fillRect(-sx, -sy * 1.45, sx * 2, sy * 1.4);
      context.fillStyle = rgba(theme.trim, 0.75);
      context.fillRect(-sx * 0.34, -sy * 1.38, sx * 0.68, sy * 1.2);
    } else if (category === "crane") {
      context.strokeStyle = rgba(theme.trim, 0.95);
      context.beginPath();
      context.moveTo(-sx * 0.7, 0);
      context.lineTo(-sx * 0.7, -sy * 2.2);
      context.moveTo(sx * 0.7, 0);
      context.lineTo(sx * 0.7, -sy * 2.2);
      context.moveTo(-sx * 0.7, -sy * 2.2);
      context.lineTo(sx * 0.7, -sy * 2.2);
      context.stroke();
      context.strokeStyle = rgba(theme.warning, 0.9);
      context.beginPath();
      context.moveTo(sx * 0.20, -sy * 2.2);
      context.lineTo(sx * 0.20, -sy * 0.55);
      context.stroke();
    } else if (category === "gear") {
      context.strokeStyle = rgba(theme.trim, 0.95);
      context.beginPath();
      context.arc(-sx * 0.18, -sy * 0.3, Math.max(5, sx * 0.72), 0, TAU);
      context.stroke();
      context.strokeStyle = rgba(theme.warning, 0.95);
      for (var i = 0; i < 8; i += 1) {
        var a = i * TAU / 8;
        context.beginPath();
        context.moveTo(-sx * 0.18 + Math.cos(a) * sx * 0.45, -sy * 0.3 + Math.sin(a) * sy * 0.45);
        context.lineTo(-sx * 0.18 + Math.cos(a) * sx * 0.86, -sy * 0.3 + Math.sin(a) * sy * 0.86);
        context.stroke();
      }
    } else if (category === "pipe" || category === "power") {
      context.strokeStyle = rgba(theme.trim, 0.92);
      context.beginPath();
      context.moveTo(-sx, 0);
      context.lineTo(sx, -sy * 0.7);
      context.lineTo(sx, -sy * 1.6);
      context.stroke();
      context.fillStyle = rgba(theme.lamp, 0.9);
      context.beginPath();
      context.arc(sx * 0.4, -sy * 1.0, Math.max(2, sx * 0.13), 0, TAU);
      context.fill();
    } else if (category === "furnace") {
      context.fillStyle = rgba(theme.wall[1], 0.95);
      context.beginPath();
      context.ellipse(0, -sy * 0.8, sx * 0.65, sy * 0.95, 0, 0, TAU);
      context.fill();
      context.fillStyle = rgba(theme.warning, 0.95);
      context.beginPath();
      context.ellipse(0, -sy * 0.85, sx * 0.25, sy * 0.34, 0, 0, TAU);
      context.fill();
    } else {
      context.fillStyle = rgba(theme.warning, 0.82);
      context.fillRect(-sx * 0.9, -sy * 0.7, sx * 0.8, sy * 0.72);
      context.fillStyle = rgba(theme.floorEdge, 0.9);
      context.fillRect(sx * 0.15, -sy * 1.0, sx * 0.78, sy * 0.78);
    }
    context.restore();
  }

  function drawFallbackActor(context, record, now) {
    var p = fallbackProjection(record.current.x, record.current.y, 2);
    var kind = record.kind;
    var base = kind === "player" ? theme.player : kind === "npc" ? theme.npc : theme.enemy;
    var moving = Math.abs(record.target.x - record.current.x) + Math.abs(record.target.y - record.current.y) > 0.018;
    var stride = Math.sin(now * (kind === "player" ? 0.012 : 0.009) + record.phase) * (moving ? 6 : 1.4);
    var scale = kind === "enemy" ? 1.04 : 1;
    context.save();
    context.translate(p.x, p.y);
    context.lineCap = "round";
    context.strokeStyle = "#12191d";
    context.lineWidth = 4 * scale;
    context.beginPath();
    context.moveTo(-5 * scale, 2);
    context.lineTo(-5 * scale + stride, 16 * scale);
    context.moveTo(5 * scale, 2);
    context.lineTo(5 * scale - stride, 16 * scale);
    context.stroke();
    context.fillStyle = rgba(0x151b20, 0.96);
    context.fillRect(-7 * scale + stride * 0.1, 13 * scale, 10 * scale, 4 * scale);
    context.fillRect(0 * scale - stride * 0.1, 13 * scale, 10 * scale, 4 * scale);
    if (kind === "player") {
      // 岑遥：瘦高短外套、青灰内层、暖金肩章/胸口信标、短发。
      context.fillStyle = rgba(0x182128, 0.98);
      context.beginPath();
      context.moveTo(-11 * scale, 15 * scale);
      context.lineTo(-9 * scale, -11 * scale);
      context.lineTo(-6 * scale, -17 * scale);
      context.lineTo(6 * scale, -17 * scale);
      context.lineTo(10 * scale, -11 * scale);
      context.lineTo(11 * scale, 15 * scale);
      context.closePath();
      context.fill();
      context.fillStyle = rgba(0x6b8589, 0.98);
      context.fillRect(-5 * scale, -12 * scale, 10 * scale, 22 * scale);
      context.fillStyle = rgba(0xd6a85c, 0.98);
      context.fillRect(-13 * scale, -9 * scale, 5 * scale, 3 * scale);
      context.fillRect(8 * scale, -9 * scale, 5 * scale, 3 * scale);
      context.beginPath();
      context.arc(0, -23 * scale, 7 * scale, 0, TAU);
      context.fill();
      context.fillStyle = rgba(0x242f35, 1);
      context.beginPath();
      context.arc(0, -28 * scale, 7 * scale, Math.PI, TAU);
      context.fill();
      context.fillStyle = rgba(0xf5c865, 1);
      context.beginPath();
      context.arc(0, -13 * scale, 2.5 * scale, 0, TAU);
      context.fill();
    } else {
      context.fillStyle = "#" + colorToHex(base);
      context.beginPath();
      context.ellipse(0, -1, 13 * scale, 17 * scale, 0, 0, TAU);
      context.fill();
      context.fillStyle = rgba(theme.floorEdge, 0.72);
      context.fillRect(-14 * scale, -8 * scale, 28 * scale, 6 * scale);
      context.fillStyle = rgba(kind === "enemy" ? theme.warning : theme.trim, 0.98);
      context.beginPath();
      context.arc(0, -21 * scale, 8 * scale, 0, TAU);
      context.fill();
      context.fillStyle = rgba(kind === "enemy" ? theme.warning : theme.accent, 0.95);
      context.fillRect(-7 * scale, -22 * scale, 14 * scale, 3 * scale);
      context.fillStyle = rgba(theme.floorEdge, 0.94);
      context.fillRect(11 * scale, -4 * scale, 4 * scale, 17 * scale);
      context.fillStyle = rgba(theme.lamp, 0.95);
      context.beginPath();
      context.arc(0, -30 * scale, 2.2 * scale, 0, TAU);
      context.fill();
    }
    drawFallbackHealth(context, record, scale);
    context.restore();
  }

  function drawFallbackHealth(context, record, scale) {
    var data = record.kind === "player" ? playerState : record.data || {};
    var hp = num(data.hp, NaN);
    var max = num(data.max_hp, num(data.maxHp, NaN));
    if (!isFinite(hp) || !isFinite(max) || max <= 0) return;
    var ratio = clamp(hp / max, 0, 1);
    context.fillStyle = "rgba(8,12,15,.82)";
    context.fillRect(-18 * scale, -40 * scale, 36 * scale, 3 * scale);
    context.fillStyle = "#" + colorToHex(record.kind === "enemy" ? theme.warning : theme.accent);
    context.fillRect(-17 * scale, -39 * scale, 34 * ratio * scale, 1 * scale);
  }

  function drawFallbackColumn(context, marker, color) {
    var cell = marker && marker.cell ? marker.cell : { x: 0, y: 0 };
    var p = fallbackProjection(cell.x, cell.y, 2);
    var h = fallbackView.tileH * 2.3;
    context.strokeStyle = rgba(color, 0.60);
    context.lineWidth = 1.5;
    context.beginPath();
    context.moveTo(p.x, p.y);
    context.lineTo(p.x, p.y - h);
    context.stroke();
    context.strokeStyle = rgba(color, 0.82);
    context.beginPath();
    context.ellipse(p.x, p.y, fallbackView.tileW * 0.35, fallbackView.tileH * 0.34, 0, 0, TAU);
    context.stroke();
  }

  function drawFallbackEffects(context, now) {
    effects.forEach(function (effect) {
      var age = clamp((now - effect.started) / (effect.duration * 1000), 0, 1);
      var p = fallbackProjection(effect.x, effect.y, 3);
      var radius = (0.35 + age * (effect.kind === "skill_0" ? 2.2 : effect.kind === "pulse" ? 1.9 : 1.1)) * fallbackView.tileW;
      var color = effect.kind === "skill_0" ? 0x71d9ff : effect.kind === "skill_1" ? 0xb28cff : effect.kind === "skill_2" ? 0xffcf68 : effect.kind === "heal" ? 0x73e7ae : effect.kind === "shot" ? theme.accent : effect.kind === "attack" ? theme.warning : theme.trim;
      context.save();
      context.globalAlpha = 1 - age;
      context.strokeStyle = rgba(color, 0.92);
      context.lineWidth = Math.max(1, fallbackView.scale * 0.07);
      if (effect.kind === "skill_0") {
        for (var q = 0; q < 3; q += 1) {
          context.beginPath();
          var qRadius = radius * (0.54 + q * 0.24);
          context.ellipse(p.x, p.y, qRadius, qRadius * 0.46, 0, 0, TAU);
          context.stroke();
        }
        context.globalAlpha *= 0.7;
        context.fillStyle = rgba(color, 0.22);
        context.beginPath();
        context.ellipse(p.x, p.y, radius * 0.72, radius * 0.24, 0, 0, TAU);
        context.fill();
      } else if (effect.kind === "skill_1") {
        // Layered hexagons remain legible in the canvas fallback.
        for (var h = 0; h < 3; h += 1) {
          context.beginPath();
          for (var side = 0; side < 6; side += 1) {
            var ha = side * TAU / 6 - Math.PI / 6;
            var hx = p.x + Math.cos(ha) * radius * (0.52 + h * 0.12);
            var hy = p.y - fallbackView.tileH * (0.4 + h * 0.22) + Math.sin(ha) * radius * (0.30 + h * 0.08);
            if (side === 0) context.moveTo(hx, hy); else context.lineTo(hx, hy);
          }
          context.closePath();
          context.stroke();
        }
      } else if (effect.kind === "skill_2") {
        var rdx = num(effect.dx, 1);
        var rdy = num(effect.dy, 0);
        var endX = p.x + rdx * fallbackView.tileW * 4.8;
        var endY = p.y + rdy * fallbackView.tileH * 4.8 - fallbackView.tileH;
        context.lineWidth = Math.max(2, fallbackView.scale * 0.13);
        context.beginPath();
        context.moveTo(p.x - rdx * fallbackView.tileW * 0.3, p.y - rdy * fallbackView.tileH * 0.3 - fallbackView.tileH);
        context.lineTo(endX, endY);
        context.stroke();
        context.lineWidth = Math.max(1, fallbackView.scale * 0.04);
        context.strokeStyle = "rgba(255,245,190,.9)";
        context.beginPath();
        context.moveTo(p.x, p.y - fallbackView.tileH);
        context.lineTo(endX, endY);
        context.stroke();
        context.beginPath();
        context.arc(endX, endY, radius * 0.18, 0, TAU);
        context.stroke();
      } else {
        context.beginPath();
        context.ellipse(p.x, p.y, radius, radius * 0.46, 0, 0, TAU);
        context.stroke();
      }
      if ((effect.kind === "shot" || effect.kind === "dash") && effect.kind !== "skill_2") {
        context.beginPath();
        context.moveTo(p.x, p.y - fallbackView.tileH);
        context.lineTo(p.x + num(effect.dx, 1) * fallbackView.tileW * 3, p.y + num(effect.dy, 0) * fallbackView.tileH * 3 - fallbackView.tileH);
        context.stroke();
      }
      context.restore();
    });
  }

  function performanceNow() {
    return typeof global.performance !== "undefined" && global.performance.now ? global.performance.now() : Date.now();
  }

  function drawFallback(now) {
    if (!fallbackContext || !map) return;
    var context = fallbackContext;
    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    context.clearRect(0, 0, logicalWidth, logicalHeight);
    context.fillStyle = "#" + colorToHex(theme.background);
    context.fillRect(0, 0, logicalWidth, logicalHeight);
    var center = currentPlayerCell();
    var range = 20;
    var x0 = Math.max(0, center.x - range);
    var x1 = Math.min(map.w - 1, center.x + range);
    var y0 = Math.max(0, center.y - range);
    var y1 = Math.min(map.h - 1, center.y + range);
    for (var sum = x0 + y0; sum <= x1 + y1; sum += 1) {
      for (var yy = y0; yy <= y1; yy += 1) {
        var xx = sum - yy;
        if (xx < x0 || xx > x1) continue;
        var tile = tileAt(xx, yy);
        if (tile === "#") drawFallbackWall(context, xx, yy);
        else drawFallbackFloor(context, xx, yy, tile);
      }
    }
    if (fallback) {
      fallbackHazardData.forEach(function (hazard, index) {
        var z = hazard.zone || {};
        var radius = hazard.area ? Math.max(num(z.w, 1), num(z.h, 1)) * 0.58 : num(z.radius, 1);
        drawFallbackRing(context, num(z.x, 0), num(z.y, 0), radius, hazard.area ? theme.steam : theme.warning, hazard.telegraph ? 0.86 : hazard.area ? 0.34 : 0.48, index);
      });
    } else {
      hazardMarkers.forEach(function (hazard) {
        var z = hazard.userData.zone || {};
        var radius = hazard.userData.area ? Math.max(num(z.w, 1), num(z.h, 1)) * 0.58 : num(z.radius, 1);
        drawFallbackRing(context, num(z.x, 0), num(z.y, 0), radius, hazard.userData.area ? theme.steam : theme.warning, hazard.userData.telegraph ? 0.86 : hazard.userData.area ? 0.34 : 0.48, hazard.userData.phase);
      });
    }
    decorProps.forEach(function (prop) { drawFallbackProp(context, prop); });
    if (fallback) fallbackInteractionData.forEach(function (marker) { drawFallbackColumn(context, marker, theme.accent); });
    else interactionMarkers.forEach(function (marker) { drawFallbackColumn(context, { cell: marker.userData.cell }, theme.accent); });
    var records = Object.keys(actorRecords).map(function (id) { return actorRecords[id]; }).sort(function (a, b) {
      return (a.current.x + a.current.y) - (b.current.x + b.current.y);
    });
    records.forEach(function (record) { drawFallbackActor(context, record, now); });
    var markerData = fallback ? fallbackMarkerData : dynamicMarkers.map(function (marker) { return { marker: marker.userData.marker || {}, kind: marker.userData.kind }; });
    markerData.forEach(function (entry) {
      var data = entry.marker || {};
      var p = fallbackProjection(num(data.x, 0), num(data.y, 0), 2);
      var color = entry.kind === "portal" ? theme.accent : entry.kind === "gate" ? theme.warning : theme.lamp;
      context.fillStyle = rgba(color, data.done ? 0.28 : 0.92);
      context.beginPath();
      context.arc(p.x, p.y - fallbackView.tileH * 1.0, entry.kind === "portal" ? 5 : 3, 0, TAU);
      context.fill();
    });
    if (hoverTarget) {
      var hover = fallbackProjection(hoverTarget.x, hoverTarget.y, 1);
      var hq = fallbackCellQuad(hoverTarget.x, hoverTarget.y, 1);
      context.strokeStyle = rgba(theme.accent, 0.9);
      context.lineWidth = 2;
      quadPath(context, hq);
      context.stroke();
      context.beginPath();
      context.arc(hover.x, hover.y, 3, 0, TAU);
      context.fillStyle = rgba(theme.accent, 0.95);
      context.fill();
    }
    drawFallbackEffects(context, now);
  }

  function resize() {
    if (!canvas) return;
    var parent = canvas.parentElement;
    var rect = parent && parent.getBoundingClientRect ? parent.getBoundingClientRect() : canvasRect();
    var width = Math.max(1, Math.round(num(rect && rect.width, canvas.clientWidth || canvas.width || 1)));
    var height = Math.max(1, Math.round(num(rect && rect.height, canvas.clientHeight || canvas.height || 1)));
    logicalWidth = width;
    logicalHeight = height;
    dpr = clamp(num(global.devicePixelRatio, 1), 0.5, MAX_DPR);
    if (quality === "low") dpr = Math.min(dpr, 1);
    else if (quality === "medium") dpr = Math.min(dpr, 1.5);
    if (fallback) {
      canvas.width = Math.max(1, Math.round(width * dpr));
      canvas.height = Math.max(1, Math.round(height * dpr));
      fallbackView.scale = Math.max(18, Math.min(width / 17, height / 13));
      fallbackView.tileW = fallbackView.scale * 0.74;
      fallbackView.tileH = fallbackView.scale * 0.43;
      if (fallbackContext) fallbackContext.imageSmoothingEnabled = true;
      return;
    }
    renderer.setPixelRatio(dpr);
    renderer.setSize(width, height, false);
    var aspect = width / Math.max(1, height);
    var halfH = VIEW_H * 0.5;
    var halfW = Math.max(VIEW_W * 0.5, halfH * aspect);
    camera.left = -halfW;
    camera.right = halfW;
    camera.top = halfH;
    camera.bottom = -halfH;
    camera.updateProjectionMatrix();
  }

  function onCanvasMove(event) {
    var target = targetAt(event.clientX, event.clientY);
    hoverTarget = target;
    if (callbacks.onHover) callbacks.onHover(target);
  }

  function onCanvasClick(event) {
    var target = targetAt(event.clientX, event.clientY);
    if (target && callbacks.onTarget) callbacks.onTarget(target);
  }

  function onCanvasLeave() {
    hoverTarget = null;
    if (callbacks.onHover) callbacks.onHover(null);
  }

  function frame(now) {
    if (!initialized) return;
    var dt = lastFrame ? clamp((now - lastFrame) / 1000, 0.001, 0.08) : 1 / 60;
    lastFrame = now;
    if (map) {
      updateActorMotion(now, dt);
      updateCamera();
      updateDecorations(now);
      flushPendingEffects();
      updateEffects(now);
      if (fallback) drawFallback(now);
      else if (renderer && scene && camera) renderer.render(scene, camera);
    }
    rafId = (global.requestAnimationFrame || function (callback) { return global.setTimeout(function () { callback(performanceNow()); }, 16); })(frame);
  }

  function startLoop() {
    if (rafId != null) return;
    lastFrame = 0;
    rafId = (global.requestAnimationFrame || function (callback) { return global.setTimeout(function () { callback(performanceNow()); }, 16); })(frame);
  }

  function stopLoop() {
    if (rafId == null) return;
    if (global.cancelAnimationFrame) global.cancelAnimationFrame(rafId);
    else if (global.clearTimeout) global.clearTimeout(rafId);
    rafId = null;
  }

  function init(cv, options) {
    if (!cv) throw new Error("CampaignRenderer.init requires a canvas");
    if (initialized) dispose();
    THREE = global.THREE || THREE;
    canvas = cv;
    options = options || {};
    callbacks.onTarget = typeof options.onTarget === "function" ? options.onTarget : null;
    callbacks.onHover = typeof options.onHover === "function" ? options.onHover : null;
    fallback = false;
    initialized = true;
    canvas.addEventListener("mousemove", onCanvasMove);
    canvas.addEventListener("click", onCanvasClick);
    canvas.addEventListener("mouseleave", onCanvasLeave);
    if (global.addEventListener) global.addEventListener("resize", resize);
    var webglAcquired = false;
    if (hasWebGL()) {
      try {
        renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: quality !== "low", alpha: false, powerPreference: "high-performance" });
        webglAcquired = true;
      } catch (error) {
        // Context creation failed before ownership was established; 2D is a
        // legitimate fallback in this narrow case.
        renderer = null;
        fallback = true;
      }
    } else {
      fallback = true;
    }
    if (webglAcquired) {
      try {
        setColorSpace();
        scene = new THREE.Scene();
        camera = new THREE.OrthographicCamera(-7.5, 7.5, 6, -6, 0.1, 160);
        raycaster = new THREE.Raycaster();
        pointer = new THREE.Vector2();
        groundPlane = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
        topologyRoot = new THREE.Group();
        decorationRoot = new THREE.Group();
        staticPropRoot = new THREE.Group();
        dynamicDecorationRoot = new THREE.Group();
        actorRoot = new THREE.Group();
        effectRoot = new THREE.Group();
        topologyRoot.name = "finite-map-topology";
        decorationRoot.name = "hand-placed-decorations";
        staticPropRoot.name = "authored-static-props";
        dynamicDecorationRoot.name = "dynamic-markers-and-hazards";
        actorRoot.name = "interpolated-actors";
        effectRoot.name = "procedural-effects";
        decorationRoot.add(staticPropRoot, dynamicDecorationRoot);
        scene.add(topologyRoot, decorationRoot, actorRoot, effectRoot);
        scene.userData.ambient = new THREE.HemisphereLight(0xb9d0d5, 0x13191e, 1.28);
        scene.add(scene.userData.ambient);
        scene.userData.keyLight = new THREE.DirectionalLight(0xfff2d4, 1.72);
        scene.userData.keyLight.position.set(11, 20, 9);
        scene.userData.keyLight.castShadow = quality !== "low";
        if (scene.userData.keyLight.shadow && scene.userData.keyLight.shadow.mapSize) scene.userData.keyLight.shadow.mapSize.set(1024, 1024);
        scene.add(scene.userData.keyLight);
        scene.userData.fillLight = new THREE.DirectionalLight(0x85bbc2, 0.62);
        scene.userData.fillLight.position.set(-12, 9, -10);
        scene.add(scene.userData.fillLight);
        scene.userData.playerLight = new THREE.PointLight(0xf6c75e, 1.1, 9, 2);
        scene.add(scene.userData.playerLight);
        updateTheme(true);
        resize();
      } catch (error) {
        // A WebGL context already exists.  Do not call getContext("2d") on
        // this canvas or hide the real initialization error behind fallback.
        dispose();
        throw error;
      }
    }
    if (fallback) {
      fallbackContext = canvas.getContext && canvas.getContext("2d");
      if (!fallbackContext) {
        var contextError = new Error("CampaignRenderer requires WebGL or a 2D canvas context");
        dispose();
        throw contextError;
      }
      resize();
    }
    if (map) {
      topologyKey = "";
      decorationKey = "";
      staticPropKey = "";
      dynamicDecorationKey = "";
      syncDataAfterMapChange();
    }
    startLoop();
    return api;
  }

  function syncDataAfterMapChange() {
    if (!map) return;
    currentWorldId = map.world_id || "hub";
    updateTheme(true);
    rebuildExploredLookup();
    var nextKey = tileSignature(map);
    if (nextKey !== topologyKey) {
      topologyKey = nextKey;
      if (!fallback) buildTopology();
      else topologyRebuilds += 1;
    }
    syncStaticDecorations();
    syncDynamicDecorations();
    refreshExploredColors();
    syncActors();
    if (scene && playerRecord && scene.userData.playerLight) {
      scene.userData.playerLight.position.set(playerRecord.current.x, 2.2, playerRecord.current.y);
    }
  }

  function setData(input) {
    var nextMap = normalizeMap(input);
    if (!nextMap) {
      clearWorldScene();
      return;
    }
    var previousWorld = currentWorldId;
    var isNewWorld = previousWorld !== nextMap.world_id;
    if (isNewWorld) clearEffectsForWorldChange();
    map = nextMap;
    currentWorldId = nextMap.world_id || "hub";
    if (input && input.player) setPlayerState(input.player);
    var targetX = Math.round(num(nextMap.px, 0));
    var targetY = Math.round(num(nextMap.py, 0));
    if (playerRecord && isNewWorld) {
      playerRecord.current.x = targetX;
      playerRecord.current.y = targetY;
    }
    if (playerRecord) {
      playerRecord.target.x = targetX;
      playerRecord.target.y = targetY;
      if (Math.abs(targetX - playerRecord.current.x) + Math.abs(targetY - playerRecord.current.y) > 0.01) playerRecord.heading = Math.atan2(targetX - playerRecord.current.x, targetY - playerRecord.current.y);
    }
    syncDataAfterMapChange();
    if (fallback) drawFallback(performanceNow());
  }

  function setPlayerState(nextPlayer) {
    if (!nextPlayer || typeof nextPlayer !== "object") return;
    playerState = Object.assign({}, playerState, nextPlayer);
    var x = nextPlayer.x != null ? nextPlayer.x : nextPlayer.px;
    var y = nextPlayer.y != null ? nextPlayer.y : nextPlayer.py;
    if (playerRecord && x != null && y != null) {
      playerRecord.target.x = Math.round(num(x, playerRecord.target.x));
      playerRecord.target.y = Math.round(num(y, playerRecord.target.y));
      if (Math.abs(playerRecord.target.x - playerRecord.current.x) + Math.abs(playerRecord.target.y - playerRecord.current.y) > 0.01) playerRecord.heading = Math.atan2(playerRecord.target.x - playerRecord.current.x, playerRecord.target.y - playerRecord.current.y);
    }
    if (playerRecord) {
      playerRecord.data = playerState;
      updateHealthBar(playerRecord);
    }
  }

  function setEffect(effect) {
    if (!effect || typeof effect !== "object") return;
    var kind = supportedEffectKind(effect.kind);
    if (!kind) return;
    pendingEffects.push(Object.assign({}, effect, { kind: kind }));
    if (playerRecord && ["attack", "dash"].indexOf(kind) >= 0) playerRecord.swing = Math.max(playerRecord.swing, 0.30);
  }

  function setQuality(nextQuality) {
    if (typeof nextQuality === "number") nextQuality = nextQuality >= 0.85 ? "high" : nextQuality >= 0.55 ? "medium" : "low";
    var value = safeString(nextQuality).toLowerCase();
    quality = value === "low" || value === "medium" || value === "high" ? value : "high";
    if (renderer) {
      renderer.shadowMap.enabled = quality !== "low";
      setColorSpace();
      resize();
    }
  }

  function dispose() {
    stopLoop();
    if (canvas) {
      canvas.removeEventListener("mousemove", onCanvasMove);
      canvas.removeEventListener("click", onCanvasClick);
      canvas.removeEventListener("mouseleave", onCanvasLeave);
    }
    if (global.removeEventListener) global.removeEventListener("resize", resize);
    effects.forEach(disposeEffect);
    effects = [];
    pendingEffects = [];
    if (renderer && renderer.renderLists && renderer.renderLists.dispose) renderer.renderLists.dispose();
    [topologyRoot, staticPropRoot, dynamicDecorationRoot, actorRoot, effectRoot].forEach(clearGroup);
    ownedMaterials.slice().forEach(function (mat) { if (mat && mat.dispose) mat.dispose(); });
    ownedGeometries.slice().forEach(function (geo) { if (geo && geo.dispose) geo.dispose(); });
    if (renderer) {
      if (renderer.forceContextLoss) renderer.forceContextLoss();
      renderer.dispose();
    }
    ownedMaterials = [];
    ownedGeometries = [];
    sharedMaterials = Object.create(null);
    sharedGeometries = Object.create(null);
    canvas = null;
    renderer = null;
    scene = null;
    camera = null;
    raycaster = null;
    pointer = null;
    groundPlane = null;
    fallbackContext = null;
    topologyRoot = null;
    decorationRoot = null;
    staticPropRoot = null;
    dynamicDecorationRoot = null;
    actorRoot = null;
    effectRoot = null;
    baseMesh = null;
    floorInstances = [];
    wallInstances = [];
    wallCells = [];
    actorRecords = Object.create(null);
    playerRecord = null;
    dynamicMarkers = [];
    hazardMarkers = [];
    interactionMarkers = [];
    decorProps = [];
    fallbackMarkerData = [];
    fallbackHazardData = [];
    fallbackInteractionData = [];
    rebuildExploredLookup();
    hoverTarget = null;
    initialized = false;
    topologyKey = "";
    decorationKey = "";
    staticPropKey = "";
    dynamicDecorationKey = "";
  }

  function debug() {
    return {
      mode: fallback ? "canvas2d-isometric-fallback" : "webgl-orthographic-25d",
      webgl: !fallback && !!renderer,
      initialized: initialized,
      world_id: currentWorldId,
      theme: theme ? theme.id : null,
      map: map ? { w: map.w, h: map.h, revision: map.revision, floor: map.floor } : null,
      topologyKey: topologyKey,
      topologyRebuilds: topologyRebuilds,
      decorationRebuilds: decorationRebuilds,
      floorInstances: floorInstances.reduce(function (total, instanced) { return total + (instanced.count || 0); }, 0),
      wallInstances: wallCells.length,
      actors: Object.keys(actorRecords).length,
      effects: effects.length + pendingEffects.length,
      viewport: { width: logicalWidth, height: logicalHeight, dpr: dpr },
      quality: quality,
      camera: camera ? {
        type: "OrthographicCamera",
        x: +camera.position.x.toFixed(2),
        y: +camera.position.y.toFixed(2),
        z: +camera.position.z.toFixed(2),
        tilt_degrees: 38,
        view: { width: VIEW_W, height: VIEW_H },
      } : null,
      target: hoverTarget ? { x: hoverTarget.x, y: hoverTarget.y } : null,
    };
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

  global.CampaignRenderer = api;
})(typeof window !== "undefined" ? window : this);
