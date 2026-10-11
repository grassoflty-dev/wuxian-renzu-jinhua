import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../../..");
const outPath = path.join(root, "tools/continuous-tps-architecture-probe/results/babylon-adapter-tp07a1.json");
const require = createRequire(import.meta.url);
const { adaptWorldViewPlayer, createTp07aPlaceholder } = require("./adapter.js");

const assertions = [];
const counts = Object.create(null);
function record(id, pass, detail = "") {
  assertions.push({ id, pass, detail });
  if (!pass) throw new Error(`${id}: ${detail || "assertion failed"}`);
}
function count(kind) {
  counts[kind] = (counts[kind] || 0) + 1;
}
function close(a, b, label, epsilon = 1e-9) {
  record(label, Math.abs(a - b) <= epsilon, `${a} != ${b}`);
}
function vectorClose(actual, expected, label) {
  close(actual.x, expected.x, `${label}.x`);
  close(actual.y, expected.y, `${label}.y`);
  close(actual.z, expected.z, `${label}.z`);
}
function view({ epoch = 7, tick = 1, x = 0, y = 0, z = 0, yaw = 0, vx = 0, vy = 0, vz = 0, protocol = "continuous-ipc", version = 1 } = {}) {
  return {
    protocol,
    version,
    worldEpoch: epoch,
    serverTick: tick,
    ackSeq: tick,
    serverTimeMs: tick * 16,
    worldId: "grey_hive",
    player: {
      transform: { positionM: { xM: x, yM: y, zM: z }, yawRad: yaw },
      velocityMps: { xM: vx, yM: vy, zM: vz },
    },
    actors: [],
    doors: [],
  };
}

class FakeTransformNode {
  constructor(name, scene) {
    this.name = name;
    this.scene = scene;
    this.position = { x: 0, y: 0, z: 0, set: (x, y, z) => { this.position.x = x; this.position.y = y; this.position.z = z; } };
    this.rotation = { x: 0, y: 0, z: 0 };
    this.disposed = false;
    scene.nodes.push(this);
  }
  dispose() { this.disposed = true; }
  isDisposed() { return this.disposed; }
}
class FakeScene { constructor() { this.nodes = []; } }

let Babylon = null;
let babylonLoadError = null;
let scene;
let evidenceLevel;
try {
  Babylon = require(path.join(root, "server-rs/release-ui/vendor/babylon.js"));
  const engine = new Babylon.NullEngine();
  scene = new Babylon.Scene(engine);
  scene.useRightHandedSystem = true;
  evidenceLevel = "real_babylon_null_engine_scene_transform_node";
} catch (error) {
  babylonLoadError = String(error && (error.stack || error.message) || error);
  scene = new FakeScene();
  evidenceLevel = "partial_blocked_fake_node_only";
}

const nodeOptions = Babylon ? { BABYLON: Babylon, name: "tp07a-placeholder-player" } : { TransformNode: FakeTransformNode, name: "tp07a-placeholder-player" };
const placeholder = createTp07aPlaceholder(scene, nodeOptions);
const formalSentinel = Babylon ? new Babylon.TransformNode("formal-sentinel-not-owned", scene) : new FakeTransformNode("formal-sentinel-not-owned", scene);
formalSentinel.position.x = 99;
formalSentinel.position.y = 88;
formalSentinel.position.z = 77;
formalSentinel.rotation.y = 2.5;

const adapted = adaptWorldViewPlayer(view({ x: 12.375, y: 1.25, z: -7.625, yaw: -2.356194490192345, vx: 1, vy: 2, vz: 3 }));
vectorClose(adapted.position, { x: 12.375, y: 1.25, z: -7.625 }, "asymmetric_negative_identity");
close(adapted.yaw, -2.356194490192345, "asymmetric_negative_yaw");
record("no_grid_scaling", adapted.position.x === 12.375 && adapted.position.z === -7.625);

for (const [id, yaw] of [["zero", 0], ["plus_x", Math.PI / 2], ["minus_z", Math.PI], ["minus_x", -Math.PI / 2]]) {
  const sample = adaptWorldViewPlayer(view({ yaw }));
  close(sample.yaw, yaw, `cardinal_${id}`);
}

let result = placeholder.accept(view({ epoch: 7, tick: 1, x: 0, y: 0, z: 0, yaw: 0 }), 0);
count(result.kind);
record("first_frame_snap", result.kind === "accepted_snap");
let node = scene.getTransformNodeByName ? scene.getTransformNodeByName("tp07a-placeholder-player") : scene.nodes.find((item) => item.name === "tp07a-placeholder-player");
vectorClose(node.position, { x: 0, y: 0, z: 0 }, "snap_position");
close(node.rotation.y, 0, "snap_yaw");

result = placeholder.accept(view({ epoch: 7, tick: 2, x: 10, y: 2, z: -4, yaw: Math.PI / 2 }), 100);
count(result.kind);
record("same_epoch_accept", result.kind === "accepted_interpolation");
result = placeholder.render(50);
count(result.kind);
record("explicit_render_interpolation", result.kind === "accepted_interpolation");
vectorClose(node.position, { x: 5, y: 1, z: -2 }, "interpolation_midpoint");
close(node.rotation.y, Math.PI / 4, "interpolation_yaw");

const beforeDrop = { x: node.position.x, y: node.position.y, z: node.position.z, yaw: node.rotation.y };
for (const dropped of [
  ["duplicate_tick", view({ epoch: 7, tick: 2, x: 100 }), "drop_tick_regression"],
  ["lower_tick", view({ epoch: 7, tick: 1, x: 200 }), "drop_tick_regression"],
  ["old_epoch", view({ epoch: 6, tick: 9, x: 300 }), "drop_stale_epoch"],
]) {
  result = placeholder.accept(dropped[1], 120);
  count(result.kind);
  record(dropped[0], result.kind === dropped[2]);
}
vectorClose(node.position, beforeDrop, "drops_do_not_mutate_node");
close(node.rotation.y, beforeDrop.yaw, "drops_do_not_mutate_yaw");

result = placeholder.accept(view({ epoch: 8, tick: 1, x: -3, y: 4, z: 9, yaw: -Math.PI / 2 }), 200);
count(result.kind);
record("higher_epoch_reset_snap", result.kind === "accepted_snap" && result.epochReset === true);
vectorClose(node.position, { x: -3, y: 4, z: 9 }, "new_epoch_snap_position");
close(node.rotation.y, -Math.PI / 2, "new_epoch_snap_yaw");

const unchanged = { x: node.position.x, y: node.position.y, z: node.position.z, yaw: node.rotation.y };
for (const invalid of [
  ["wrong_protocol", view({ protocol: "wrong" }), "reject_schema"],
  ["wrong_version", view({ version: 99 }), "reject_schema"],
  ["nonfinite_position", view({ x: NaN }), "reject_non_finite"],
  ["nonfinite_yaw", view({ yaw: Infinity }), "reject_non_finite"],
]) {
  result = placeholder.accept(invalid[1], 300);
  count(result.kind);
  record(invalid[0], result.kind === invalid[2]);
}
vectorClose(node.position, unchanged, "invalid_does_not_mutate_node");
close(node.rotation.y, unchanged.yaw, "invalid_does_not_mutate_yaw");

const extrapolationScene = Babylon ? new Babylon.Scene(new Babylon.NullEngine()) : new FakeScene();
const extrapolation = createTp07aPlaceholder(extrapolationScene, Babylon ? { BABYLON: Babylon, name: "tp07a-placeholder-extrapolation" } : { TransformNode: FakeTransformNode, name: "tp07a-placeholder-extrapolation" });
result = extrapolation.accept(view({ epoch: 1, tick: 1, x: 1, vx: 2 }), 0);
count(result.kind);
record("tentative_default_extrapolation_off_snap", result.kind === "accepted_snap");
result = extrapolation.accept(view({ epoch: 1, tick: 2, x: 2, vx: 2 }), 100);
count(result.kind);
const held = extrapolation.render(150);
count(held.kind);
record("extrapolation_default_off", held.kind === "accepted_interpolation");

const tentativeScene = Babylon ? new Babylon.Scene(new Babylon.NullEngine()) : new FakeScene();
const tentative = createTp07aPlaceholder(tentativeScene, Babylon ? { BABYLON: Babylon, name: "tp07a-placeholder-tentative", allowExtrapolation: true, horizonMs: 50, correctionMeters: 0.25 } : { TransformNode: FakeTransformNode, name: "tp07a-placeholder-tentative", allowExtrapolation: true, horizonMs: 50, correctionMeters: 0.25 });
tentative.accept(view({ epoch: 1, tick: 1, x: 1, vx: 2 }), 0);
tentative.accept(view({ epoch: 1, tick: 2, x: 2, vx: 2 }), 100);
result = tentative.render(125);
count(result.kind);
record("tentative_extrapolation_opt_in", result.kind === "accepted_extrapolation" && result.horizonMs === 25);

placeholder.dispose();
const disposed = node.isDisposed ? node.isDisposed() : node.disposed;
record("dispose_disposes_placeholder_only", disposed === true);
result = placeholder.accept(view({ epoch: 9, tick: 1, x: 1000 }), 400);
count(result.kind);
record("dispose_rejects_late_accept", result.kind === "closed");
result = placeholder.render(400);
count(result.kind);
record("dispose_rejects_late_render", result.kind === "closed");
record("formal_node_untouched", formalSentinel.position.x === 99 && formalSentinel.position.y === 88 && formalSentinel.position.z === 77 && formalSentinel.rotation.y === 2.5);

if (Babylon) {
  scene.dispose();
  formalSentinel.dispose();
}

const pass = assertions.every((item) => item.pass);
const output = {
  schema: "continuous_tp07a_placeholder_adapter_tp07a1_v1",
  status: evidenceLevel === "real_babylon_null_engine_scene_transform_node" ? "isolated_placeholder_real_babylon" : "partial_blocked_babylon_vendor_load",
  evidenceLevel,
  babylonVendor: "server-rs/release-ui/vendor/babylon.js",
  babylonLoadError,
  allChecksPassed: pass,
  assertionCount: assertions.length,
  assertions,
  filterCounts: counts,
  coordinateContract: {
    position: "xM/yM/zM -> position.x; yM -> position.y; zM -> position.z",
    yaw: "yawRad -> rotation.y",
    units: "metres and radians",
    footPivot: "placeholder TransformNode position is the candidate foot anchor",
    gridScale: "forbidden; no px/py input or scale",
    axisFlip: "forbidden; numeric identity only",
  },
  statePolicy: {
    firstSample: "accepted_snap",
    oldEpoch: "drop_stale_epoch",
    higherEpoch: "reset_then_first_sample_snap",
    sameEpochTickLessOrEqual: "drop_tick_regression",
    interpolation: "explicit render(nowMs) between latest two samples",
    extrapolationDefault: false,
    tentativeExtrapolation: { enabledOnlyByOption: true, horizonMs: 50, correctionMeters: 0.25, measured: false },
    dispose: "closed; late accept/render cannot mutate node",
  },
  ownership: {
    nodeName: "tp07a-placeholder-player",
    onlyDedicatedPlaceholderMutated: true,
    formalTpsRendererReferenced: false,
    campaignDelegateReferenced: false,
    cameraReferenced: false,
    actionQueueReferenced: false,
    tp06aCallbackReferenced: false,
  },
  risksNotMeasured: [
    "formal WebView callback delivery and close/process-exit",
    "formal camera/player pivot and mesh visual offset",
    "formal scene export and production nav projection",
    "Rapier/KCC collision clearance",
  ],
  nextFormalIntegrationAllowlist: [
    "server-rs/release-ui/js/tp07a-heartbeat-bridge.js",
    "server-rs/release-ui/index.html: one opt-in script line only if required",
    "server-rs/release-ui/js/renderer-boot.js: placeholder factory only if unavoidable",
    "new TP-07A-only integration test/result/report",
  ],
  saveGate: "not_applicable; isolated placeholder only",
  tp07Gate: "not_passed",
  architectureGate: "unchanged_no_go",
};
fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(`TP-07A1 ${pass ? "passed" : "failed"}: ${assertions.length} assertions; evidence=${evidenceLevel}; result=${outPath}`);
