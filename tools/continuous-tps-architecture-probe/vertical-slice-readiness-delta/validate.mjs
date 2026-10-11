import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../../..");
const baselinePath = path.join(repoRoot, "tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-tp18a1.json");
const baselineValidatorPath = path.join(repoRoot, "tools/continuous-tps-architecture-probe/vertical-slice-readiness/validate.mjs");
const tp16Path = path.join(repoRoot, "tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json");
const resultPath = path.join(repoRoot, "tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json");
const reportPath = path.join(repoRoot, "docs/CONTINUOUS_VERTICAL_SLICE_CRITICAL_PATH_REVIEW.md");
const fixturePath = path.join(scriptDir, "negative-fixtures.json");

const hashFile = (file) => crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const readJson = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const exists = (file) => fs.existsSync(file) && fs.statSync(file).isFile();
const rel = (file) => path.relative(repoRoot, file).replaceAll("\\", "/");
const clone = (value) => JSON.parse(JSON.stringify(value));
const baseline = readJson(baselinePath);
const tp16 = readJson(tp16Path);

const exact = {
  core: ["server-rs/src/lib.rs", "server-rs/src/fixed_step.rs", "server-rs/src/continuous_kcc.rs", "server-rs/Cargo.toml", "server-rs/Cargo.lock", "server-rs/tests/fixed_step_test.rs", "server-rs/tests/continuous_kcc_test.rs"],
  ipc: ["server-rs/src/lib.rs", "server-rs/src/main.rs", "server-rs/src/continuous_input.rs", "server-rs/src/campaign_main.rs", "server-rs/ui/js/client.js", "server-rs/Cargo.toml", "server-rs/Cargo.lock", "server-rs/tests/continuous_input_test.rs"],
  save: ["server-rs/src/lib.rs", "server-rs/src/campaign.rs", "server-rs/src/campaign_store.rs", "server-rs/src/save_v3_migration.rs", "server-rs/src/save_v3_persistence.rs", "server-rs/tests/campaign_store_test.rs", "server-rs/tests/save_v3_migration_test.rs", "server-rs/tests/save_v3_persistence_test.rs"],
  scene: ["server-rs/src/world_v3.rs", "server-rs/src/grey_hive_control_collider.rs", "server-rs/tests/world_v3_contract_test.rs", "server-rs/tests/grey_hive_control_collider_test.rs"],
  assets: ["server-rs/release-ui/asset-manifest.json", "server-rs/release-ui/js/asset-resolver.js", "server-rs/release-ui/js/asset-cache.js", "server-rs/release-ui/THIRD_PARTY_NOTICES.md", "server-rs/tauri.conf.json"],
  capability: ["server-rs/src/world_v3.rs", "server-rs/src/campaign_main.rs", "server-rs/release-ui/js/capability-resolver.js", "server-rs/ui/index.html"],
  hud: ["server-rs/release-ui/index.html", "server-rs/release-ui/js/client.js", "server-rs/release-ui/js/renderer-boot.js", "server-rs/release-ui/css/style.css"],
  perf: ["server-rs/tests/continuous_runtime_soak_test.rs", ".github/workflows/continuous-tps.yml", "server-rs/Cargo.toml", "server-rs/Cargo.lock"]
};

const packageSpecs = [
  {
    id: "PKG-FORMAL-CORE-TICK-KCC",
    goal: "Implement formal Rust fixed-step, KCC and dynamic-door owner without shadow state.",
    prerequisites: ["ADR-A", "ADR-B", "ADR-C", "TP-04", "TP-05"],
    writes: exact.core,
    readonly: ["server-rs/src/world_v3.rs", "docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md", "tools/continuous-tps-architecture-probe/results/tick-ag07.json"],
    commands: ["cargo fmt --manifest-path server-rs/Cargo.toml --check", "cargo test --manifest-path server-rs/Cargo.toml fixed_step continuous_kcc", "cargo test --manifest-path server-rs/Cargo.toml --locked"],
    rollback: "Revert only this package commit; restore the listed Rust/Cargo/test files and rerun the package tests before any dependent package.",
    lifts: ["TP-04 blocked -> formal_pass only after formal KCC/door evidence", "TP-05 blocked -> formal_pass only after release-profile tick evidence"],
    gate: "Architecture B/C and formal contract prerequisites",
    media: false
  },
  {
    id: "PKG-FORMAL-IPC-OWNER",
    goal: "Wire formal input, Tauri entry and WorldView/Channel lifecycle owner with bounded payload and backpressure.",
    prerequisites: ["ADR-D", "TP-05", "TP-06", "PKG-FORMAL-CORE-TICK-KCC"],
    writes: exact.ipc,
    readonly: ["server-rs/src/world_v3.rs", "tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json", "docs/CONTINUOUS_TPS_ARCHITECTURE_GATE_REPORT.md"],
    commands: ["cargo fmt --manifest-path server-rs/Cargo.toml --check", "cargo test --manifest-path server-rs/Cargo.toml continuous_input", "cargo test --manifest-path server-rs/Cargo.toml --locked"],
    rollback: "Revert only IPC owner files and restore the previous command/Channel boundary; keep evidence artifacts immutable.",
    lifts: ["TP-06 blocked -> formal_pass only after real owner/load/lifecycle evidence"],
    gate: "Architecture D and formal tick/KCC owner",
    media: false
  },
  {
    id: "PKG-FORMAL-SAVE-CAMPAIGN",
    goal: "Connect SaveV3 migration/persistence and CampaignStore with checkpoint/reload semantics, without synthesizing choice.",
    prerequisites: ["ADR-E", "TP-03", "TP-14", "PKG-FORMAL-CORE-TICK-KCC"],
    writes: exact.save,
    readonly: ["server-rs/tests/fixtures", "tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json", "docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md"],
    commands: ["cargo fmt --manifest-path server-rs/Cargo.toml --check", "cargo test --manifest-path server-rs/Cargo.toml save_v3 campaign_store", "cargo test --manifest-path server-rs/Cargo.toml --locked"],
    rollback: "Restore SaveV3/CampaignStore files and leave old save readers intact; rerun migration and persistence tests.",
    lifts: ["TP-03 blocked -> formal_pass only after production storage/crash evidence", "TP-14 blocked -> formal_pass only after formal Campaign owner evidence"],
    gate: "Save Gate E and Campaign authority",
    media: false
  },
  {
    id: "PKG-FORMAL-GREY-HIVE-SCENE",
    goal: "Produce formal Grey Hive scene/export/collider/nav inputs with canonical IDs and Gate B geometry.",
    prerequisites: ["ADR-I", "TP-09", "PKG-FORMAL-CORE-TICK-KCC"],
    writes: exact.scene,
    readonly: ["tools/continuous-tps-architecture-probe/results/formal-scene-exporter-tp09b1.json", "docs/GREY_HIVE_LAYOUT_V3_REVIEW.md", "docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md"],
    commands: ["cargo fmt --manifest-path server-rs/Cargo.toml --check", "cargo test --manifest-path server-rs/Cargo.toml world_v3 grey_hive_control_collider", "node tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs"],
    rollback: "Remove only the new formal scene/export owner files and restore the prior legacy world entrypoint; do not alter synthetic fixtures.",
    lifts: ["TP-09 blocked -> formal_pass only after formal scene/hash/collider/nav/camera evidence"],
    gate: "Grey Hive Gate I / TP-09",
    media: false
  },
  {
    id: "PKG-FORMAL-ASSET-RELEASE",
    goal: "Create a verified public manifest/resolver/cache boundary; pending/devOnly assets remain excluded.",
    prerequisites: ["ADR-G", "TP-08", "TP-16", "TP-16A1-OFFLINE-ASSET-GATE-REAL-WEBVIEW", "PKG-FORMAL-IPC-OWNER"],
    writes: exact.assets,
    readonly: ["tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json", "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json", "tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json", "docs/CONTINUOUS_ASSET_POLICY_RECONCILIATION.md"],
    commands: ["node tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs", "cargo test --manifest-path server-rs/Cargo.toml --locked"],
    rollback: "Remove only the new release resolver/manifest files and leave the public manifest empty; no asset bytes may be copied during rollback.",
    lifts: ["TP-08 blocked -> formal_pass only after item authorization and offline release evidence", "TP-16 blocked -> formal_pass only after public license/provenance and bundle evidence"],
    gate: "Gate G; no new authorization is implied by TP-16A1",
    media: false
  },
  {
    id: "PKG-FORMAL-CAPABILITY-HUD",
    goal: "Wire formal capability resolver and release HUD/map adapter while accessibility remains independent.",
    prerequisites: ["ADR-H", "TP-13", "TP-15", "TP-16", "PKG-FORMAL-IPC-OWNER", "PKG-FORMAL-ASSET-RELEASE"],
    writes: exact.capability.concat(exact.hud),
    readonly: ["tools/continuous-tps-architecture-probe/results/tauri-hud-map-tp15b1.json", "tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json", "docs/CONTINUOUS_HUD_MAP_WEBVIEW_REVIEW.md"],
    commands: ["node --check server-rs/release-ui/js/capability-resolver.js", "node --check server-rs/release-ui/js/renderer-boot.js", "cargo test --manifest-path server-rs/Cargo.toml --locked"],
    rollback: "Restore release UI and capability files only; preserve accessibility text fallback and keep candidate harnesses untouched.",
    lifts: ["TP-13 blocked -> formal_pass only after formal resolver/render lifecycle evidence", "TP-15 blocked -> formal_pass only after formal DOM/map/manual evidence"],
    gate: "Capability Gate H and formal UI boundary",
    media: false
  },
  {
    id: "PKG-FORMAL-CAMPAIGN-PLAYTHROUGH",
    goal: "Connect A01→C04 mainline, three choice states, Gate B lockdown-only rule, checkpoint/reload and terminal result.",
    prerequisites: ["TP-03", "TP-09", "TP-10", "TP-11", "TP-12", "TP-14", "PKG-FORMAL-SAVE-CAMPAIGN", "PKG-FORMAL-GREY-HIVE-SCENE"],
    writes: ["server-rs/src/campaign.rs", "server-rs/src/campaign_store.rs", "server-rs/src/campaign_main.rs", "server-rs/src/state.rs", "server-rs/tests/campaign_flow.rs", "server-rs/tests/campaign_store_test.rs", "server-rs/tests/playthrough.rs"],
    readonly: ["docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md", "server-rs/tests/fixtures", "tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json"],
    commands: ["cargo fmt --manifest-path server-rs/Cargo.toml --check", "cargo test --manifest-path server-rs/Cargo.toml campaign_flow playthrough", "cargo test --manifest-path server-rs/Cargo.toml --locked"],
    rollback: "Revert Campaign owner/test files only; retain legacy world behavior and do not rewrite save fixtures.",
    lifts: ["DOD-A01-C04-CHOICES blocked -> formal_pass only after all three states complete without duplicate rewards", "DOD-CHECKPOINT-RELOAD blocked -> formal_pass only after production reload evidence"],
    gate: "Gate B is hive_lockdown_completed only; hive_choice/NPC never a hard prerequisite",
    media: false
  },
  {
    id: "PKG-FORMAL-PERF-SOAK-ACCEPTANCE",
    goal: "Run release-profile 30/60Hz, 30 enter/exit, offline, queue and manual accessibility/performance acceptance.",
    prerequisites: ["TP-17A1", "PKG-FORMAL-CAMPAIGN-PLAYTHROUGH", "PKG-FORMAL-CAPABILITY-HUD", "PKG-FORMAL-ASSET-RELEASE"],
    writes: exact.perf,
    readonly: ["docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md", "tools/continuous-tps-architecture-probe/results/runtime-integration-soak-tp17a1.json", "tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json"],
    commands: ["cargo test --release --manifest-path server-rs/Cargo.toml --locked", "cargo test --release --manifest-path server-rs/Cargo.toml continuous_runtime_soak", "node tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs"],
    rollback: "Remove only new CI/soak test wiring; do not treat debug numbers as release budgets and do not alter production code during rollback.",
    lifts: ["DOD-30-WORLD-ENTER-EXIT unknown -> formal_pass only with 30 real transitions and counters", "DOD-PERFORMANCE blocked -> formal_pass only with environment/p95/p99/RSS evidence", "DOD-FEEL-ACCESSIBILITY unknown -> formal_pass only after authorized human acceptance"],
    gate: "TP-17 and final vertical-slice acceptance",
    media: false
  }
];

function packageMap() { return new Map(packageSpecs.map((pkg) => [pkg.id, pkg])); }

function buildConflictGraph() {
  const owners = new Map();
  for (const pkg of packageSpecs) for (const file of pkg.writes) {
    if (!owners.has(file)) owners.set(file, []);
    owners.get(file).push(pkg.id);
  }
  const edges = [];
  for (const [file, packages] of owners) for (let i = 0; i < packages.length; i++) for (let j = i + 1; j < packages.length; j++) edges.push({ a: packages[i], b: packages[j], sharedFile: file });
  return { nodes: packageSpecs.map((pkg) => pkg.id), sharedFiles: Object.fromEntries([...owners].filter(([, packages]) => packages.length > 1)), edges };
}

const waves = [
  { wave: 1, slots: ["PKG-FORMAL-CORE-TICK-KCC"] },
  { wave: 2, slots: ["PKG-FORMAL-GREY-HIVE-SCENE", "PKG-FORMAL-IPC-OWNER"] },
  { wave: 3, slots: ["PKG-FORMAL-SAVE-CAMPAIGN", "PKG-FORMAL-ASSET-RELEASE"] }
];

function sourceMeta(file) {
  if (!exists(file)) return { path: rel(file), exists: false, sha256: null, mtimeMs: null };
  return { path: rel(file), exists: true, sha256: hashFile(file), mtimeMs: fs.statSync(file).mtimeMs };
}

function buildDeltaEntry() {
  const evidence = {
    evidenceId: "TP-16A1-OFFLINE-ASSET-GATE-REAL-WEBVIEW",
    category: "TP-16/Gate G",
    title: "Real Windows WebView offline/public/development asset policy candidate",
    status: "real_runtime_candidate",
    evidenceLevel: "real_runtime_candidate",
    formalSource: false,
    sourcePath: rel(tp16Path),
    reportPath: "docs/CONTINUOUS_OFFLINE_ASSET_GATE_REVIEW.md",
    machineState: tp16.status,
    candidateOnly: true,
    localDevelopmentOnly: true,
    pass: tp16.pass === true,
    environment: { os: "Windows x86_64", tauri: "2.11.5", webview: tp16.jsEvidence?.browser?.userAgent ?? "real WebView UA recorded in source result", profile: "local-development-only" },
    registry: tp16.jsEvidence.registry,
    networkAttempts: tp16.jsEvidence.network.attempts,
    mediaBytesRead: tp16.jsEvidence.bytes.mediaBytesRead,
    authorizationBytesRead: tp16.jsEvidence.bytes.authorizationBytesRead,
    publicBlockBeforeRead: tp16.publicBlockBeforeRead === true,
    gaps: ["Public authorization/manifest remains empty; no release bundle or human media/font QA.", "pagehide remained unknown in non-navigating close."],
    dependencies: ["TP-08", "TP-16", "ADR-G"],
    freshness: sourceMeta(tp16Path),
    allowedStatusLift: "real_runtime_candidate only; never formal_pass from this delta alone"
  };
  return evidence;
}

function buildModel() {
  const baselineHash = hashFile(baselinePath);
  const baselineValidatorHash = hashFile(baselineValidatorPath);
  const deltaEntries = [buildDeltaEntry(), {
    evidenceId: "TP-17C1-ACTIVE",
    category: "active task",
    title: "TP-17C1 active task placeholder",
    status: "not_evaluated",
    evidenceLevel: "none",
    formalSource: false,
    sourcePath: null,
    reportPath: null,
    machineState: "active_not_evaluated",
    active: true,
    pathRead: false,
    freshness: { kind: "not_observed", sha256: null, mtimeMs: null },
    dependencies: ["TP-17A1"],
    gaps: ["Active output is intentionally not read or used."],
    allowedStatusLift: "not_evaluated until task produces its own evidence"
  }];
  const baselineCounts = clone(baseline.summary.byStatus);
  const mergedCounts = clone(baselineCounts);
  for (const entry of deltaEntries) mergedCounts[entry.status] = (mergedCounts[entry.status] ?? 0) + 1;
  const conflictGraph = buildConflictGraph();
  return {
    schema: "vertical-slice-readiness-delta-tp18b1-v1",
    status: "blocked_critical_path_delta_complete",
    pass: true,
    appendOnly: true,
    baseline: {
      path: rel(baselinePath),
      schema: baseline.schema,
      sha256: baselineHash,
      validatorPath: rel(baselineValidatorPath),
      validatorSha256: baselineValidatorHash,
      total: baseline.summary.total,
      counts: baselineCounts,
      activeNotEvaluated: clone(baseline.activeNotEvaluated),
      minimalBlockingSet: clone(baseline.minimalBlockingSet)
    },
    delta: {
      source: rel(tp16Path),
      sourceSha256: hashFile(tp16Path),
      entries: deltaEntries,
      counts: { total: deltaEntries.length, byStatus: Object.fromEntries([...new Set(deltaEntries.map((entry) => entry.status))].map((status) => [status, deltaEntries.filter((entry) => entry.status === status).length])) },
      tp16Impact: { gateG: "blocked", tp16: "blocked", tp18: "blocked", evidenceUpgrade: "real_runtime_candidate", formalPass: false }
    },
    merged: { total: baseline.summary.total + deltaEntries.length, counts: mergedCounts, baselinePreserved: true },
    gateConclusion: { verticalSliceReady: false, architecture: "no_go", tp18: "blocked", reason: "TP-16A1 adds candidate availability only; formal release/runtime/integration/manual gates remain incomplete." },
    currentActive: [{ evidenceId: "AG-38", status: "not_evaluated", pathRead: false }, { evidenceId: "TP-17C1", status: "not_evaluated", pathRead: false }],
    historicalActivePreserved: clone(baseline.activeNotEvaluated),
    minimalBlockerCutSet: ["ADR-G", "TP-08", "TP-16", "TP-03", "TP-06", "TP-09", "TP-14", "TP-17A1", "DOD-PERFORMANCE", "DOD-FEEL-ACCESSIBILITY"],
    packages: packageSpecs.map((pkg) => ({ ...pkg, scheduled: false, unsatisfiedPrerequisites: pkg.prerequisites.filter((dependency) => !new Set(["TP-16A1", "TP-16A1-OFFLINE-ASSET-GATE-REAL-WEBVIEW"]).has(dependency)), status: "not_schedulable_until_prerequisites" })),
    conflictGraph,
    waves,
    deferredPackages: ["PKG-FORMAL-CAPABILITY-HUD", "PKG-FORMAL-CAMPAIGN-PLAYTHROUGH", "PKG-FORMAL-PERF-SOAK-ACCEPTANCE"],
    forbiddenScheduling: ["AG-38", "TP-17C1", "any package with unsatisfied prerequisite", "any package with write-set conflict", "any package that would promote candidate to formal_pass"],
  };
}

function validateModel(model, { checkFiles = true } = {}) {
  const errors = [];
  const allowed = new Set(["formal_pass", "real_runtime_candidate", "isolated_candidate", "blocked", "unknown", "not_evaluated"]);
  if (!model.appendOnly) errors.push({ code: "E_BASELINE_OVERWRITE", detail: "appendOnly must remain true" });
  if (model.baseline.sha256 !== hashFile(baselinePath)) errors.push({ code: "E_BASELINE_HASH", detail: "baseline hash drift" });
  if (model.baseline.total !== baseline.summary.total || JSON.stringify(model.baseline.counts) !== JSON.stringify(baseline.summary.byStatus)) errors.push({ code: "E_BASELINE_OVERWRITE", detail: "baseline counts changed" });
  if (model.gateConclusion.verticalSliceReady !== false || model.gateConclusion.architecture !== "no_go") errors.push({ code: "E_NO_GO_ESCALATION", detail: "no_go/readiness invariant" });
  const expectedMergedCounts = clone(model.baseline.counts);
  for (const entry of model.delta.entries) expectedMergedCounts[entry.status] = (expectedMergedCounts[entry.status] ?? 0) + 1;
  if (model.merged.total !== model.baseline.total + model.delta.entries.length || JSON.stringify(model.merged.counts) !== JSON.stringify(expectedMergedCounts)) errors.push({ code: "E_MERGED_MISMATCH", detail: "merged view does not equal baseline plus delta" });
  for (const entry of model.delta.entries) {
    if (!allowed.has(entry.status)) errors.push({ code: "E_STATUS_INVALID", detail: entry.evidenceId });
    if (entry.status === "formal_pass" || entry.evidenceLevel === "formal") errors.push({ code: "E_CANDIDATE_FORMAL", detail: entry.evidenceId });
    if (entry.active && entry.sourcePath) errors.push({ code: "E_ACTIVE_CONSUMED", detail: entry.evidenceId });
    if (entry.status === "not_evaluated" && (entry.sourcePath || entry.pathRead !== false)) errors.push({ code: "E_ACTIVE_CONSUMED", detail: entry.evidenceId });
    if (entry.evidenceId.startsWith("TP-16A1") && (entry.formalSource || entry.candidateOnly !== true || entry.status !== "real_runtime_candidate")) errors.push({ code: "E_CANDIDATE_FORMAL", detail: "TP-16A1 lift" });
    if (entry.sourcePath && checkFiles) {
      if (!exists(path.join(repoRoot, entry.sourcePath))) errors.push({ code: "E_PATH_MISSING", detail: entry.sourcePath });
      else if (entry.freshness?.sha256 !== hashFile(path.join(repoRoot, entry.sourcePath))) errors.push({ code: "E_HASH_MISMATCH", detail: entry.sourcePath });
    }
    if (entry.reportPath && checkFiles && !exists(path.join(repoRoot, entry.reportPath))) errors.push({ code: "E_PATH_MISSING", detail: entry.reportPath });
  }
  const tp16Delta = model.delta.entries.find((entry) => entry.evidenceId.startsWith("TP-16A1"));
  if (!tp16Delta || tp16Delta.status !== "real_runtime_candidate" || tp16Delta.formalSource || tp16Delta.candidateOnly !== true || tp16Delta.pass !== true || tp16Delta.publicBlockBeforeRead !== true) errors.push({ code: "E_TP16_LIFT", detail: "TP-16A1 must remain a real candidate only" });
  const ids = new Set(model.packages.map((pkg) => pkg.id));
  for (const pkg of model.packages) {
    if (!pkg.goal || !pkg.rollback) errors.push({ code: "E_ROLLBACK_MISSING", detail: pkg.id });
    if (!pkg.commands?.length) errors.push({ code: "E_PACKAGE_TEST_MISSING", detail: pkg.id });
    if (pkg.writes.some((file) => file.includes("*") || file.endsWith("/"))) errors.push({ code: "E_AMBIGUOUS_WRITE_SET", detail: pkg.id });
    if (pkg.scheduled === true && (pkg.unsatisfiedPrerequisites?.length ?? 0) > 0) errors.push({ code: "E_PREREQUISITE_UNSATISFIED", detail: pkg.id });
    if (pkg.gate.includes("Gate B") && pkg.prerequisites.some((item) => /choice|npc/i.test(item))) errors.push({ code: "E_GATE_B_CHOICE", detail: pkg.id });
    for (const dependency of pkg.prerequisites.filter((item) => item.startsWith("PKG-"))) if (!ids.has(dependency)) errors.push({ code: "E_DANGLING_DEPENDENCY", detail: `${pkg.id}->${dependency}` });
    if (pkg.media && pkg.id === "PKG-FORMAL-ASSET-RELEASE") errors.push({ code: "E_MEDIA_AUTHORIZATION", detail: pkg.id });
  }
  const graph = model.conflictGraph;
  for (const node of graph.nodes) if (!ids.has(node)) errors.push({ code: "E_GRAPH_NODE", detail: node });
  const edgeKeys = new Set(graph.edges.map((edge) => [edge.a, edge.b, edge.sharedFile].sort().join("|")));
  for (const edge of graph.edges) if (edge.a === edge.b) errors.push({ code: "E_SHARED_EDGE_SELF", detail: edge.a });
  for (const [file, owners] of Object.entries(graph.sharedFiles)) for (let i = 0; i < owners.length; i++) for (let j = i + 1; j < owners.length; j++) if (!edgeKeys.has([owners[i], owners[j], file].sort().join("|"))) errors.push({ code: "E_SHARED_EDGE_MISSING", detail: file });
  const waveOf = new Map();
  for (const wave of model.waves) {
    if (wave.slots.length > 3) errors.push({ code: "E_WAVE_CAPACITY", detail: wave.wave });
    for (const packageId of wave.slots) {
      if (!ids.has(packageId)) errors.push({ code: "E_WAVE_PACKAGE", detail: packageId });
      waveOf.set(packageId, wave.wave);
    }
    for (let i = 0; i < wave.slots.length; i++) for (let j = i + 1; j < wave.slots.length; j++) {
      const a = packageMap().get(wave.slots[i]); const b = packageMap().get(wave.slots[j]);
      if (a.writes.some((file) => b.writes.includes(file))) errors.push({ code: "E_WAVE_CONFLICT", detail: `${a.id}/${b.id}` });
    }
  }
  const byId = packageMap();
  for (const pkg of model.packages) for (const dependency of pkg.prerequisites.filter((item) => item.startsWith("PKG-"))) if (pkg.scheduled === true && byId.get(dependency)?.scheduled === true && waveOf.has(pkg.id) && waveOf.has(dependency) && waveOf.get(dependency) >= waveOf.get(pkg.id)) errors.push({ code: "E_DEPENDENCY_ORDER", detail: `${dependency}->${pkg.id}` });
  const allDeps = new Map(model.packages.map((pkg) => [pkg.id, pkg.prerequisites.filter((item) => item.startsWith("PKG-"))]));
  const visiting = new Set(); const visited = new Set();
  function visit(id) { if (visiting.has(id)) return true; if (visited.has(id)) return false; visiting.add(id); for (const dep of allDeps.get(id) ?? []) if (visit(dep)) return true; visiting.delete(id); visited.add(id); return false; }
  for (const id of allDeps.keys()) if (visit(id)) errors.push({ code: "E_DEPENDENCY_CYCLE", detail: id });
  return errors;
}

function mutate(model, mutation) {
  const copy = clone(model);
  if (mutation === "baseline_hash_drift") copy.baseline.sha256 = "tampered";
  if (mutation === "baseline_overwrite") copy.baseline.total += 1;
  if (mutation === "candidate_formal") { copy.delta.entries[0].status = "formal_pass"; copy.delta.entries[0].evidenceLevel = "formal"; }
  if (mutation === "active_path") copy.delta.entries[1].sourcePath = "forbidden/active.json";
  if (mutation === "missing_prerequisite_schedule") copy.packages[0].scheduled = true; copy.packages[0].status = "scheduled";
  if (mutation === "dependency_cycle") { copy.packages[0].prerequisites.push("PKG-FORMAL-PERF-SOAK-ACCEPTANCE"); }
  if (mutation === "same_wave_conflict") copy.waves[0].slots.push("PKG-FORMAL-IPC-OWNER");
  if (mutation === "missing_shared_edge") copy.conflictGraph.edges = copy.conflictGraph.edges.slice(1);
  if (mutation === "wildcard_write_set") copy.packages[0].writes.push("server-rs/src/*.rs");
  if (mutation === "missing_test") copy.packages[0].commands = [];
  if (mutation === "missing_rollback") copy.packages[0].rollback = "";
  if (mutation === "gate_b_choice") copy.packages[6].prerequisites.push("hive_choice");
  if (mutation === "unauthorized_media") copy.packages[4].media = true;
  if (mutation === "no_go_escalation") copy.gateConclusion.architecture = "go";
  return copy;
}

function runFixtures(model) {
  const fixtures = readJson(fixturePath);
  const results = fixtures.map((fixture) => {
    const errors = validateModel(mutate(model, fixture.mutation), { checkFiles: false });
    const hit = errors.find((error) => error.code === fixture.expectedCode);
    return { id: fixture.id, expectedCode: fixture.expectedCode, observedCode: hit?.code ?? null, pass: Boolean(hit), allCodes: [...new Set(errors.map((error) => error.code))] };
  });
  return { count: results.length, passed: results.filter((item) => item.pass).length, results };
}

function summary(model) {
  return { baselineTotal: model.baseline.total, deltaTotal: model.delta.entries.length, mergedTotal: model.merged.total, baselineCounts: model.baseline.counts, deltaCounts: model.delta.counts, mergedCounts: model.merged.counts };
}

function report(model) {
  const negative = model.negativeFixtures;
  const pkgRows = model.packages.map((pkg) => `| ${pkg.id} | ${pkg.status} | ${pkg.prerequisites.join(", ")} | ${pkg.writes.join("; ")} |`).join("\n");
  const waveRows = model.waves.map((wave) => `| ${wave.wave} | ${wave.slots.join(", ")} | none; all packages remain unschedulable until prerequisites pass |`).join("\n");
  const negativeRows = negative.results.map((item) => `| ${item.id} | ${item.expectedCode} | ${item.observedCode ?? "missing"} | ${item.pass ? "pass" : "FAIL"} |`).join("\n");
  return `# TP-18B1 灰巢垂直切片关键路径增量审计

## 结论

本报告是 append-only delta，不覆盖 TP-18A1 baseline。TP-16A1 只把新增证据可用性提升到 real_runtime_candidate，不能提升为 formal_pass。当前 verticalSliceReady=false，Architecture=no_go，TP-18=blocked。

## Baseline / delta / merged

- Baseline：${model.baseline.total} 条，SHA-256 ${model.baseline.sha256}；baseline 文件和 validator 未被修改。
- Delta：${model.delta.entries.length} 条；TP-16A1 real_runtime_candidate 1 条，TP-17C1 active/not_evaluated 1 条。
- Merged：${model.merged.total} 条；formal_pass=0，real_runtime_candidate=${model.merged.counts.real_runtime_candidate}，blocked=${model.merged.counts.blocked}，unknown=${model.merged.counts.unknown}，not_evaluated=${model.merged.counts.not_evaluated}。
- 当前 active overlay 只有 AG-38、TP-17C1；baseline 中历史 TP-17B1 not_evaluated 原样保留。

TP-16A1 对 Gate G、TP-16、TP-18 的影响：evidence availability=real_runtime_candidate；正式 Gate 仍 blocked；未产生授权、publish bundle 或 formal status。

## 最小 blocker cut set

${model.minimalBlockerCutSet.map((item) => `- ${item}`).join("\n")}

## Future formal package graph

每个 package 均有精确文件列表、只读输入、验证命令、rollback 和允许的状态提升；通配符写集、缺测试、缺 rollback 均被 validator 拒绝。

| Package | 当前状态 | 前置 | 精确写集 |
| --- | --- | --- | --- |
${pkgRows}

Conflict graph 以共享精确路径建立边，重点共享点包括 server-rs/src/lib.rs、server-rs/Cargo.toml、server-rs/Cargo.lock、Campaign/Store、Tauri entry、release UI、asset manifest、scene export；每条共享路径的 package pair 都有可重算 edge。

## 三槽 waves

| Wave | Slots | 调度结论 |
| ---: | --- | --- |
${waveRows}

所有 slots 当前均 not schedulable：前置 Gate/证据仍 blocked。waves 仅是未来拓扑计划，不授权任何正式实现。冲突 checker 验证同 wave 写集互斥、依赖无环、依赖 wave 顺序和最多三槽。

三槽之外延期且未调度：${model.deferredPackages.join(", ")}。

## Negative fixtures

共 ${negative.count} 个负例，稳定错误码通过 ${negative.passed}/${negative.count}：

| Fixture | Expected | Observed | Result |
| --- | --- | --- | --- |
${negativeRows}

覆盖 baseline hash/覆盖写、candidate→formal、active 消费、缺前置调度、循环依赖、同 wave 冲突、共享边遗漏、模糊写集、缺测试、缺 rollback、Gate B choice、未授权媒体和 no_go 越级。

## 禁止调度与 Gate

- AG-38、TP-17C1：not_evaluated，禁止消费未完成输出。
- TP-16A1：只能 candidateOnly/localDevelopmentOnly，不得升为 formal_pass。
- Gate B：唯一底层门禁为 hive_lockdown_completed；hive_choice/NPC 不得进入硬依赖。
- pending/local_only/rejected/未授权媒体：不得进入 release manifest。
- 不创建分支、PR 或正式文件；不生成或复制媒体。

验证命令：

node --check tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs

node tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs

git diff --check -- docs/CONTINUOUS_VERTICAL_SLICE_CRITICAL_PATH_REVIEW.md tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json
`;
}

function main() {
  const model = buildModel();
  model.negativeFixtures = runFixtures(model);
  const errors = validateModel(model);
  if (model.negativeFixtures.passed !== model.negativeFixtures.count) errors.push({ code: "E_NEGATIVE_FIXTURE", detail: `${model.negativeFixtures.passed}/${model.negativeFixtures.count}` });
  if (errors.length > 0) {
    console.error(JSON.stringify({ ok: false, errors }, null, 2));
    process.exitCode = 1;
    return;
  }
  model.statistics = summary(model);
  fs.writeFileSync(resultPath, JSON.stringify(model, null, 2) + "\n");
  fs.writeFileSync(reportPath, report(model));
  console.log(JSON.stringify({ ok: true, resultPath: rel(resultPath), reportPath: rel(reportPath), baseline: model.baseline.total, delta: model.delta.entries.length, merged: model.merged.total, counts: model.merged.counts, negatives: `${model.negativeFixtures.passed}/${model.negativeFixtures.count}`, verticalSliceReady: false, architecture: "no_go" }));
}

main();
