import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const layoutPath = path.resolve(here, "..", "layout", "layout-v2.json");
const resultPath = path.resolve(here, "..", "results", "coordinates-ag05.json");
const EPS = 1e-9;

const layout = JSON.parse(fs.readFileSync(layoutPath, "utf8"));

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function close(a, b, label) {
  assert(Math.abs(a - b) <= EPS, `${label}: ${a} != ${b}`);
}

function vecClose(a, b, label) {
  close(a.x, b.x, `${label}.x`);
  close(a.y, b.y, `${label}.y`);
  close(a.z, b.z, `${label}.z`);
}

function distance(a, b) {
  return Math.hypot(a.x - b.x, a.y - b.y, a.z - b.z);
}

function dot(a, b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

function cross(a, b) {
  return {
    x: a.y * b.z - a.z * b.y,
    y: a.z * b.x - a.x * b.z,
    z: a.x * b.y - a.y * b.x,
  };
}

function normalizeYaw(yaw) {
  let value = yaw;
  while (value <= -Math.PI) value += Math.PI * 2;
  while (value > Math.PI) value -= Math.PI * 2;
  return value;
}

// The only candidate adapter under review. It is deliberately kept as a
// pure data boundary: business code must not duplicate these axis decisions.
const adapter = {
  positionToBabylon(position) {
    return { x: position.x, y: position.y, z: position.z };
  },
  positionFromBabylon(position) {
    return { x: position.x, y: position.y, z: position.z };
  },
  yawToBabylon(yaw) {
    return normalizeYaw(yaw);
  },
  yawFromBabylon(yaw) {
    return normalizeYaw(yaw);
  },
  forwardFromYaw(yaw) {
    return { x: Math.sin(yaw), y: 0, z: Math.cos(yaw) };
  },
};

const positions = [
  { id: "origin", x: 0, y: 0, z: 0 },
  { id: "+x", x: 1, y: 0, z: 0 },
  { id: "-x", x: -1, y: 0, z: 0 },
  { id: "+z", x: 0, y: 0, z: 1 },
  { id: "-z", x: 0, y: 0, z: -1 },
  { id: "+y", x: 0, y: 1, z: 0 },
  { id: "-y", x: 0, y: -1, z: 0 },
  { id: "asymmetric", x: 12.375, y: 1.25, z: -7.625 },
];

const yaws = [0, Math.PI / 2, Math.PI, -Math.PI / 2, -2.356194490192345];
const cardinalForward = [
  { yaw: 0, forward: { x: 0, y: 0, z: 1 } },
  { yaw: Math.PI / 2, forward: { x: 1, y: 0, z: 0 } },
  { yaw: Math.PI, forward: { x: 0, y: 0, z: -1 } },
  { yaw: -Math.PI / 2, forward: { x: -1, y: 0, z: 0 } },
];

const layoutEntity = (id) => layout.entities.find((entity) => entity.id === id);
const ag02Projections = ["gh_cp_airlock", "gh_cp_post_bio"].map((id) => {
  const entity = layoutEntity(id);
  assert(entity && entity.type === "checkpoint", `missing AG-02 checkpoint ${id}`);
  const [x, y, z] = entity.position;
  const rust = { x, y, z };
  return {
    id,
    room: entity.room,
    candidate_layout_position_m: rust,
    babylon_candidate_position_m: adapter.positionToBabylon(rust),
    note: "read-only projection; not a frozen Transform",
  };
});

const roundtrips = positions.map((input) => {
  const babylon = adapter.positionToBabylon(input);
  const roundtrip = adapter.positionFromBabylon(babylon);
  vecClose(roundtrip, input, `${input.id}.position_roundtrip`);
  close(distance(input, { x: 0, y: 0, z: 0 }), distance(babylon, { x: 0, y: 0, z: 0 }), `${input.id}.origin_distance`);
  return { input, babylon, roundtrip };
});

const yawRoundtrips = yaws.map((input) => {
  const babylon = adapter.yawToBabylon(input);
  const roundtrip = adapter.yawFromBabylon(babylon);
  close(roundtrip, normalizeYaw(input), `yaw(${input}).roundtrip`);
  const forward = adapter.forwardFromYaw(roundtrip);
  close(Math.hypot(forward.x, forward.z), 1, `yaw(${input}).forward_length`);
  return { input, babylon, roundtrip, forward };
});

for (const sample of cardinalForward) {
  vecClose(adapter.forwardFromYaw(sample.yaw), sample.forward, `cardinal_yaw(${sample.yaw}).forward`);
}

const basisX = adapter.positionToBabylon({ x: 1, y: 0, z: 0 });
const basisY = adapter.positionToBabylon({ x: 0, y: 1, z: 0 });
const basisZ = adapter.positionToBabylon({ x: 0, y: 0, z: 1 });
const handednessDeterminant = dot(basisX, cross(basisY, basisZ));
close(handednessDeterminant, 1, "basis.handedness_determinant");

const result = {
  schema: "continuous_coordinate_probe_ag05_v1",
  status: "candidate_adapter_only",
  exit_code: 0,
  assertions: {
    all_passed: true,
    epsilon: EPS,
    position_roundtrips: roundtrips.length,
    yaw_roundtrips: yawRoundtrips.length,
    cardinal_forward_checks: cardinalForward.length,
    handedness_determinant: handednessDeterminant,
  },
  units: {
    rust_position: "metres",
    babylon_position: "metres",
    yaw: "radians",
    speed: "metres_per_second",
  },
  candidate_contract: {
    foot_anchor: "Rust/Babylon actor root position has y=0 on the floor; vertical model geometry rises above the root.",
    axes: "Rust x=east, y=up, z=depth/forward; Babylon x/y/z preserve that numeric basis.",
    handedness: "Babylon scene.useRightHandedSystem=true; the candidate adapter is a proper rotation-preserving basis with determinant +1.",
    position_formula: "B.x=R.x; B.y=R.y; B.z=R.z",
    yaw_formula: "B.rotation.y=R.yaw; forward=(sin(yaw),0,cos(yaw)); yaw=0 faces +Z and +pi/2 faces +X",
    unique_adapter_boundary: "toBabylonTransform/fromBabylonTransform (candidate boundary; no business-layer axis flips)",
  },
  source_evidence: {
    rust_map_fields: "server-rs/src/campaign.rs:250-268",
    rust_enemy_fields: "server-rs/src/campaign.rs:169-180",
    rust_player_view: "server-rs/src/campaign.rs:1671-1698",
    babylon_tps_scene: "server-rs/release-ui/js/tps-renderer.js:992-1007",
    babylon_tps_map_and_actor: "server-rs/release-ui/js/tps-renderer.js:213-238,519-554,1080-1111",
    babylon_tps_heading_and_forward: "server-rs/release-ui/js/tps-renderer.js:611-642,665-672,1162-1167",
  },
  position_roundtrips: roundtrips,
  yaw_roundtrips: yawRoundtrips,
  ag02_read_only_projections: ag02Projections,
  boundary: {
    legacy_grid_coordinates: "v2 px/py are integer tile cells and remain checkpoint-semantic evidence only; this probe never scales or promotes them to metres.",
    formal_blockout: "AG-02 candidate points are examples only; final scene transforms require TP-09 export and TP-02/TP-09 validation.",
  },
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(`AG-05 coordinate probe passed: ${roundtrips.length} position roundtrips, ${yawRoundtrips.length} yaw roundtrips, ${cardinalForward.length} cardinal forward checks, determinant=${handednessDeterminant}`);
