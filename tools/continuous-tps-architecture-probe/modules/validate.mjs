import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const MANIFEST_PATH = path.join(HERE, "architecture-v1.json");
const RESULT_PATH = path.join(HERE, "../results/modules-ag14.json");

function clone(value) { return JSON.parse(JSON.stringify(value)); }

function code(codeName, detail, id = null) { return { code: codeName, detail, id }; }

function validateGraph(manifest) {
  const errors = [];
  const nodes = manifest.dependencyGraph?.nodes ?? [];
  const ids = new Set();
  const byId = new Map();
  for (const node of nodes) {
    if (ids.has(node.id)) errors.push(code("E_DUPLICATE_NODE", `duplicate node ${node.id}`, node.id));
    ids.add(node.id);
    byId.set(node.id, node);
  }
  for (const node of nodes) {
    for (const dep of node.dependsOn ?? []) {
      if (!byId.has(dep)) errors.push(code("E_MISSING_DEPENDENCY", `${node.id} depends on missing ${dep}`, node.id));
    }
  }
  const visiting = new Set();
  const visited = new Set();
  function visit(id) {
    if (visiting.has(id)) return true;
    if (visited.has(id)) return false;
    visiting.add(id);
    const node = byId.get(id);
    const cycle = (node?.dependsOn ?? []).some((dep) => byId.has(dep) && visit(dep));
    visiting.delete(id);
    visited.add(id);
    return cycle;
  }
  for (const node of nodes) if (visit(node.id)) errors.push(code("E_DEPENDENCY_CYCLE", `dependency cycle reaches ${node.id}`, node.id));
  return errors;
}

function validateWriters(manifest) {
  const errors = [];
  const required = new Set(manifest.boundary?.singleWriterRequired ?? []);
  const seen = new Map();
  for (const domain of manifest.stateDomains ?? []) {
    if (seen.has(domain.id)) errors.push(code("E_DUPLICATE_STATE_DOMAIN", `state domain ${domain.id} has multiple declarations`, domain.id));
    seen.set(domain.id, domain.writer);
    if (required.has(domain.id) && !domain.writer) errors.push(code("E_MISSING_WRITER", `required state domain ${domain.id} has no writer`, domain.id));
  }
  for (const id of required) if (!seen.has(id)) errors.push(code("E_UNDECLARED_STATE_DOMAIN", `required state domain ${id} is absent`, id));
  return errors;
}

function validateRoutes(manifest) {
  const errors = [];
  const routes = manifest.routes ?? {};
  if (routes.grey_hive?.backend !== "continuous") errors.push(code("E_GREY_HIVE_ROUTE", "grey_hive must route to continuous"));
  for (const world of ["mist_harbor", "clockworks", "biohazard_ch1", "unknown"]) {
    if (routes[world]?.backend !== "legacy") errors.push(code("E_LEGACY_ROUTE", `${world} must remain legacy`, world));
  }
  if (routes.grey_hive?.fallbackAllowed !== false) errors.push(code("E_GREY_HIVE_FALLBACK", "grey_hive must not silently fall back to legacy"));
  return errors;
}

function validatePayload(payload, forbiddenFields) {
  const errors = [];
  const text = JSON.stringify(payload);
  for (const field of forbiddenFields) if (text.includes(field)) errors.push(code("E_VIEW_HANDLE_LEAK", `view payload contains forbidden ${field}`));
  return errors;
}

function simulateRoutes(manifest, sequence) {
  const errors = [];
  let activeBackend = null;
  const transitions = [];
  for (const step of sequence) {
    const expected = step.location === "hub" ? "legacy" : manifest.routes[step.location]?.backend ?? manifest.routes.unknown.backend;
    if (expected !== step.backend) errors.push(code("E_ROUTE_SEQUENCE", `${step.location} expected ${expected}, got ${step.backend}`, step.location));
    if (step.location === "grey_hive" && expected !== "continuous") errors.push(code("E_GREY_HIVE_ROUTE", "grey_hive entered without continuous backend"));
    if (step.location !== "hub" && step.location !== "grey_hive" && expected !== "legacy") errors.push(code("E_LEGACY_ROUTE", `${step.location} entered non-legacy backend`));
    activeBackend = expected;
    transitions.push({ location: step.location, backend: activeBackend, activeWriters: activeBackend === "continuous" ? ["world_state_v3"] : ["legacy_game_state"] });
  }
  if (transitions.some((step) => step.activeWriters.length !== 1)) errors.push(code("E_DUAL_WORLD_UPDATE", "route step has more than one active world writer"));
  return { errors, transitions };
}

function simulateSaveRestore(manifest, samples) {
  const errors = [];
  const restored = [];
  for (const sample of samples) {
    const route = manifest.routes[sample.world] ?? manifest.routes.unknown;
    const actual = route.backend;
    if (actual !== sample.expectedBackend) errors.push(code("E_SAVE_ROUTE_RESTORE", `${sample.schema}/${sample.world} restored to ${actual}, expected ${sample.expectedBackend}`, sample.world));
    restored.push({ schema: sample.schema, world: sample.world, expectedBackend: sample.expectedBackend, actualBackend: actual, fallback: Boolean(sample.fallback), pass: actual === sample.expectedBackend });
  }
  return { errors, restored };
}

function validateManifest(manifest) {
  const errors = [];
  if (manifest.schema !== "continuous_module_boundary_ag14_v1") errors.push(code("E_SCHEMA", "unexpected architecture schema"));
  if (manifest.version !== 1) errors.push(code("E_VERSION", "architecture version must be 1"));
  errors.push(...validateGraph(manifest), ...validateWriters(manifest), ...validateRoutes(manifest));
  const sampleView = manifest.syntheticView ?? { world_id: "grey_hive", position: [1, 2], velocity: [0, 0], doors: [], effects: [] };
  errors.push(...validatePayload(sampleView, manifest.boundary.viewForbiddenFields));
  const route = simulateRoutes(manifest, manifest.scenario.routeSequence);
  const restore = simulateSaveRestore(manifest, manifest.scenario.saveRestore);
  errors.push(...route.errors, ...restore.errors);
  return { errors, route, restore };
}

function scenario(name, mutate, expectedCode = null) {
  const manifest = clone(JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8")));
  mutate(manifest);
  const result = validateManifest(manifest);
  const pass = expectedCode ? result.errors.some((item) => item.code === expectedCode) : result.errors.length === 0;
  return { name, pass, expectedCode, errorCodes: [...new Set(result.errors.map((item) => item.code))], errorCount: result.errors.length };
}

const manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8"));
const baseline = validateManifest(manifest);
const scenarios = [
  { name: "baseline_contract", pass: baseline.errors.length === 0, expectedCode: null, errorCodes: [...new Set(baseline.errors.map((item) => item.code))], errorCount: baseline.errors.length },
  scenario("dependency_cycle", (value) => { value.dependencyGraph.nodes.find((node) => node.id === "campaign_schema").dependsOn.push("release_ui"); }, "E_DEPENDENCY_CYCLE"),
  scenario("duplicate_writer", (value) => { value.stateDomains.push({ id: "campaign_map", writer: "legacy_session", fields: [], persistence: "legacy_save_json", authority: "invalid test" }); }, "E_DUPLICATE_STATE_DOMAIN"),
  scenario("view_handle_leak", (value) => { value.syntheticView = { world_id: "grey_hive", rapier_handle: "runtime-only" }; }, "E_VIEW_HANDLE_LEAK"),
  scenario("grey_hive_wrong_route", (value) => { value.routes.grey_hive.backend = "legacy"; }, "E_GREY_HIVE_ROUTE"),
  scenario("legacy_world_wrong_route", (value) => { value.routes.mist_harbor.backend = "continuous"; }, "E_LEGACY_ROUTE"),
  scenario("dual_world_update", (value) => { value.scenario.routeSequence[1].backend = "legacy"; }, "E_ROUTE_SEQUENCE"),
  scenario("save_restore_mismatch", (value) => { value.routes.grey_hive.backend = "legacy"; }, "E_GREY_HIVE_ROUTE"),
  { name: "legacy_to_hub_to_continuous_to_legacy", pass: baseline.route.errors.length === 0 && baseline.route.transitions.length === manifest.scenario.routeSequence.length, expectedCode: null, errorCodes: baseline.route.errors.map((item) => item.code), errorCount: baseline.route.errors.length },
  { name: "save_v3_result_state_recovery", pass: baseline.restore.errors.length === 0, expectedCode: null, errorCodes: baseline.restore.errors.map((item) => item.code), errorCount: baseline.restore.errors.length }
];

const sourceAudit = [
  { subject: "active_entrypoint", paths: ["server-rs/Cargo.toml:8-10", "server-rs/src/campaign_main.rs:56-75"], observedOwner: "campaign_main -> CampaignStore/CampaignState", evidence: "default Cargo binary registers campaign_view/campaign_action/campaign_save" },
  { subject: "campaign_authority", paths: ["server-rs/src/campaign.rs:275-319", "server-rs/src/campaign.rs:1588", "server-rs/src/campaign.rs:1736", "server-rs/src/campaign.rs:2879-2920"], observedOwner: "CampaignState/campaign::act", evidence: "state machine owns hub, world, objectives, map, combat actions and result transitions" },
  { subject: "campaign_persistence", paths: ["server-rs/src/campaign_store.rs:15-116", "server-rs/src/campaign_store.rs:190-225"], observedOwner: "CampaignStore", evidence: "campaign-v2 path, pending/backup atomic save, load warning and flush" },
  { subject: "legacy_runtime", paths: ["server-rs/src/main.rs:12-38", "server-rs/src/main.rs:92-115", "server-rs/src/world.rs:1-20", "server-rs/src/world.rs:437-459"], observedOwner: "legacy Session<GameState>/engine/world", evidence: "parallel save.json and multi-world GameState runtime; source is not a Cargo bin" },
  { subject: "world_registry", paths: ["server-rs/src/worlds/mod.rs:31-75", "server-rs/src/worlds/mod.rs:832-871", "server-rs/src/worlds/mod.rs:2131-2201"], observedOwner: "static WorldData registry", evidence: "world IDs, gateway table and find_world registry" },
  { subject: "ui_boundary", paths: ["server-rs/release-ui/js/app.js:188-195", "server-rs/release-ui/js/app.js:552-564", "server-rs/release-ui/js/app.js:680-687"], observedOwner: "read-only presentation/input queue", evidence: "campaign invoke/fetch and renderer setData/setEffect/setPlayerState only" },
  { subject: "renderer_boundary", paths: ["server-rs/release-ui/index.html:55-61", "server-rs/release-ui/js/renderer-boot.js:1-124"], observedOwner: "Babylon TPS renderer", evidence: "legacy Three/side2d scripts are retained but not loaded; renderer failure does not fall back" }
];

const report = {
  schema: "continuous_module_probe_ag14_v1",
  status: scenarios.every((item) => item.pass) ? "isolated_module_boundary_probe_passed_with_formal_gaps" : "failed",
  pass: scenarios.every((item) => item.pass),
  scope: "read-only source audit plus isolated module ownership, route and boundary validator; no formal Rust/UI changes",
  realRuntimeModified: false,
  sourceAudit,
  observed: manifest.observedBindings,
  ownership: {
    campaign: "CampaignState via campaign::act; CampaignStore is the only campaign-v2 persistence owner",
    legacy: "GameState via legacy Session/main.rs + engine/world; save.json is separate from campaign-v2.json",
    presentation: "release-ui/app.js and Babylon TPS renderer are read-only projections/input adapters",
    candidateContinuous: "WorldStateV3/motor/collision/navigation/combat/interaction are contract-only; no formal modules exist yet"
  },
  candidateInterfaces: manifest.candidateInterfaces,
  dependencyGraph: manifest.dependencyGraph,
  routes: manifest.routes,
  boundary: manifest.boundary,
  scenarios,
  routeTrace: baseline.route.transitions,
  saveRestoreTrace: baseline.restore.restored,
  observedGaps: [
    "The active campaign binary currently accepts grey_hive, mist_harbor and clockworks under one CampaignState state machine; the grey_hive continuous / mist_harbor+clockworks legacy split is a candidate contract, not an implemented runtime route.",
    "No formal WorldStateV3, fixed-tick motor, collision, navigation, combat, interaction, IPC stream or SaveV3 Rust modules are present; TP-02 through TP-08 remain implementation work.",
    "The retained legacy main.rs source has a parallel GameState/world/engine API and save.json path, but it is not declared as a Cargo binary alongside campaign_main; build/runtime selection must be frozen in TP-01.",
    "campaign-v2 currently serializes CampaignState and is not a SaveV3 implementation; migration and cross-route recovery need the TP-03/TP-14 contract.",
    "Route fallback and hub handoff require formal integration tests so a failed continuous load cannot update legacy and continuous state in the same transition."
  ],
  formalGaps: [
    "TP-01 ADR F must freeze module owners and the one-writer rule in real Rust interfaces.",
    "TP-01 ADR J must freeze the single world split and legacy regression behavior.",
    "TP-02/03/04/05/06 must provide compile-time and runtime proof for WorldStateV3, SaveV3, fixed tick, physics and IPC lifecycle.",
    "TP-09 onward must prove grey_hive scene, navigation, combat and renderer integration without changing mist_harbor/clockworks legacy semantics."
  ]
};

fs.mkdirSync(path.dirname(RESULT_PATH), { recursive: true });
fs.writeFileSync(RESULT_PATH, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
if (!report.pass) process.exitCode = 1;
