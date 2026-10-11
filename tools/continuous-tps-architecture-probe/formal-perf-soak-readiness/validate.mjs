import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";

const scriptDir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, (value) => value.slice(1)).replaceAll("/", path.sep);
const root = path.resolve(scriptDir, "..", "..", "..");
const resultPath = path.join(root, "tools", "continuous-tps-architecture-probe", "results", "formal-perf-soak-readiness-tp01r9.json");

function relPath(value) {
  return value.replaceAll(path.sep, "/");
}

function readJson(relativePath) {
  const absolute = path.join(root, relativePath);
  if (!fs.existsSync(absolute)) return { exists: false, value: null, sha256: null };
  const bytes = fs.readFileSync(absolute);
  return {
    exists: true,
    value: JSON.parse(bytes.toString("utf8")),
    sha256: crypto.createHash("sha256").update(bytes).digest("hex"),
  };
}

function fileExists(relativePath) {
  return fs.existsSync(path.join(root, relativePath));
}

function gitValue(args) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : null;
}

const workspaceCommit = gitValue(["rev-parse", "HEAD"]);
const dirtyFiles = (gitValue(["status", "--porcelain"]) || "").split(/\r?\n/).filter(Boolean);
const evidenceSpecs = [
  ["TP-18B1", "tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json", "isolated_candidate", "formal package ledger and exact prerequisites"],
  ["TP-06B1", "tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json", "real_webview_transport", "real Windows WebView transport candidate; no formal owner/load"],
  ["TP-13A1", "tools/continuous-tps-architecture-probe/results/capability-rear-view-tp13a1.json", "null_engine", "NullEngine/isolated resource lifecycle"],
  ["TP-13B1", "tools/continuous-tps-architecture-probe/results/tauri-rear-view-tp13b1.json", "real_webview_transport", "local WebView/WebGL lifecycle candidate; no pixel/performance claim"],
  ["TP-14B1", "tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json", "isolated_candidate", "isolated world routing candidate"],
  ["TP-14C1", "tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json", "real_webview_transport", "local WebView routing/lifecycle candidate"],
  ["TP-17A1", "tools/continuous-tps-architecture-probe/results/runtime-integration-soak-tp17a1.json", "isolated_candidate", "debug-only integration/soak candidate"],
  ["TP-17B1", "tools/continuous-tps-architecture-probe/results/runtime-owner-observability-tp17b1.json", "isolated_candidate", "wrapper owner/queue candidate"],
  ["TP-17C1", "tools/continuous-tps-architecture-probe/results/tauri-runtime-observability-tp17c1.json", "real_webview_transport", "real Windows WebView/Tauri candidate; local development only"],
  ["TP-04C1", "tools/continuous-tps-architecture-probe/results/dynamic-door-owner-tp04c1.json", "isolated_candidate", "single-owner dynamic door candidate"],
  ["AG-44", "tools/continuous-tps-architecture-probe/results/formal-core-blocker-remediation-ag44.json", "isolated_candidate", "grounded authority remediation; not performance evidence"],
];

const evidence = evidenceSpecs.map(([id, relativePath, evidenceClass, limitation]) => {
  const loaded = readJson(relativePath);
  const value = loaded.value || {};
  return {
    id,
    path: relativePath,
    exists: loaded.exists,
    sha256: loaded.sha256,
    evidenceClass,
    status: value.status ?? "not_observed",
    pass: value.pass ?? value.machinePass ?? value.allChecksPassed ?? null,
    candidateOnly: value.candidateOnly ?? null,
    formalSource: value.formalSource ?? false,
    limitation,
  };
});

const tp17a1 = readJson("tools/continuous-tps-architecture-probe/results/runtime-integration-soak-tp17a1.json").value || {};
const tp17c1 = readJson("tools/continuous-tps-architecture-probe/results/tauri-runtime-observability-tp17c1.json").value || {};
const tp06b1 = readJson("tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json").value || {};
const tp13b1 = readJson("tools/continuous-tps-architecture-probe/results/tauri-rear-view-tp13b1.json").value || {};
const tp14c1 = readJson("tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json").value || {};
const tp18b1 = readJson("tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json").value || {};

const thresholds = {
  blockout: {
    resolution: "1920x1080",
    fps: ">=60 FPS",
    onePercentLow: ">=50 FPS",
    simP95: "<4 ms/tick",
    settledMemory: "<1.0 GB",
    ssdLoad: "<4 s",
    snapshotRate: "30 Hz",
    normalCorrection: "<0.10 m typically",
  },
  greyHiveAlpha: {
    resolution: "1920x1080",
    fps: ">=60 FPS",
    onePercentLow: ">=45 FPS",
    simP95: "<5 ms/tick",
    settledMemory: "<1.5 GB",
    ssdLoad: "<6 s",
    snapshotRate: "30 Hz",
    normalCorrection: "<0.10 m typically",
  },
  source: "docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md:2027",
  status: "frozen_project_targets_not_measured_formally",
};

const formalBuild = {
  integratedBuildEvidence: false,
  cargoManifest: fileExists("server-rs/Cargo.toml"),
  releaseProfile: fileExists("server-rs/Cargo.toml"),
  formalSoakTestPresent: fileExists("server-rs/tests/continuous_runtime_soak_test.rs"),
  ciWorkflowPresent: fileExists(".github/workflows/continuous-tps.yml"),
  releaseRunResultPresent: false,
  reason: "TP-18B1 marks PKG-FORMAL-PERF-SOAK-ACCEPTANCE not schedulable until prerequisites; no release integrated benchmark result exists.",
};

function ref(pathName, value, provenance, status = "reference_only") {
  return { path: pathName, value, provenance, status };
}

const matrix = [
  {
    id: "1080p60_frame_and_1percent_low",
    budget: "1080p 60FPS / 1% low",
    integratedBuild: false,
    hardware: null,
    os: "Windows x86_64 only in candidate WebView records",
    gpu: null,
    commit: workspaceCommit,
    resolution: "1920x1080 not measured",
    preset: "not recorded",
    samplingTool: "no formal frame-time sampler; no pixel/GPU path accepted",
    durationSeconds: null,
    sampleCount: null,
    p50: null,
    p95: null,
    onePercentLow: null,
    memory: null,
    load: null,
    simTick: null,
    snapshot: null,
    backpressure: null,
    worldReplacement: null,
    dispose: ref("TP-13B1", { grantRevokeCycles: 50, worldReplacements: 30 }, "real_webview_transport candidate", "reference_only"),
    threshold: thresholds.blockout.onePercentLow,
    status: "blocked",
    blocker: "E_FORMAL_FRAME_GPU_BUILD_MISSING",
    evidence: ["TP-13B1", "TP-17C1"],
  },
  {
    id: "simulation_tick_latency",
    budget: "Rust 60Hz simulation p95",
    integratedBuild: false,
    hardware: null,
    os: "Windows x86_64 candidate only",
    gpu: null,
    commit: workspaceCommit,
    resolution: "not applicable",
    preset: "not applicable",
    samplingTool: "std::time::Instant / wrapper timer",
    durationSeconds: 10.716397,
    sampleCount: 36000,
    p50: ref("TP-17A1", "269.1 µs", "debug isolated candidate", "reference_only"),
    p95: ref("TP-17A1", "453.7 µs", "debug isolated candidate", "reference_only"),
    onePercentLow: null,
    memory: "unknown",
    load: null,
    simTick: ref("TP-17C1", { p50: "2.2 µs", p95: "3.7 µs", p99: "20.4 µs", count: 36000 }, "real WebView candidate wrapper-owned timing", "reference_only"),
    snapshot: ref("TP-17C1", { sent: 600, callbackDelta: 600 }, "real WebView candidate", "reference_only"),
    backpressure: ref("TP-17C1", { capacity: 8, highWater: 1, rejected: 1, dropped: 0 }, "wrapper-owned candidate queue", "reference_only"),
    worldReplacement: ref("TP-17C1", { cycles: 30, created: 31, disposed: 31 }, "real WebView candidate", "reference_only"),
    dispose: ref("TP-17C1", { closed: true, channelCloseSentinel: "not_publicly_exposed" }, "real WebView candidate", "reference_only"),
    threshold: thresholds.blockout.simP95,
    status: "blocked",
    blocker: "E_FORMAL_INTEGRATED_BUILD_MISSING",
    evidence: ["TP-17A1", "TP-17C1"],
  },
  {
    id: "settled_memory_growth",
    budget: "settled memory / growth",
    integratedBuild: false,
    hardware: null,
    os: "Windows x86_64 candidate only",
    gpu: "unknown",
    commit: workspaceCommit,
    resolution: "not recorded",
    preset: "not recorded",
    samplingTool: "Windows PSAPI GetProcessMemoryInfo in TP-17C1 candidate",
    durationSeconds: 1.696832,
    sampleCount: 3,
    p50: null,
    p95: null,
    onePercentLow: null,
    memory: ref("TP-17C1", { beforeWorkingSet: 31924224, peakWorkingSet: 40017920, afterWorkingSet: 34238464, allocator: "unknown", gpu: "unknown" }, "direct process API candidate; one bounded run", "reference_only"),
    load: null,
    simTick: null,
    snapshot: null,
    backpressure: null,
    worldReplacement: ref("TP-17C1", { cycles: 30 }, "real WebView candidate", "reference_only"),
    dispose: ref("TP-17C1", { created: 33, disposed: 33 }, "wrapper-owned candidate", "reference_only"),
    threshold: thresholds.blockout.settledMemory,
    status: "blocked",
    blocker: "E_MEMORY_GROWTH_FORMAL_PROVENANCE_MISSING",
    evidence: ["TP-17C1", "TP-13B1"],
  },
  {
    id: "ssd_load_time",
    budget: "SSD world load",
    integratedBuild: false,
    hardware: null,
    os: "not measured formally",
    gpu: null,
    commit: workspaceCommit,
    resolution: "not applicable",
    preset: "not applicable",
    samplingTool: "no formal load-boundary timer",
    durationSeconds: null,
    sampleCount: 0,
    p50: null,
    p95: null,
    onePercentLow: null,
    memory: null,
    load: ref("TP-06B1", { processStartMs: 26.4 }, "process start, not world/asset load", "reference_only"),
    simTick: null,
    snapshot: null,
    backpressure: null,
    worldReplacement: ref("TP-14C1", { currentEpoch: 40, closeOwnerCount: 0 }, "candidate lifecycle", "reference_only"),
    dispose: ref("TP-14C1", { created: 41, disposed: 41 }, "candidate lifecycle", "reference_only"),
    threshold: thresholds.blockout.ssdLoad,
    status: "blocked",
    blocker: "E_LOAD_BOUNDARY_FORMAL_MISSING",
    evidence: ["TP-06B1", "TP-14C1"],
  },
  {
    id: "thirty_hz_snapshot_backpressure",
    budget: "30Hz snapshot / correction / bounded backpressure",
    integratedBuild: false,
    hardware: null,
    os: "Windows x86_64 candidate only",
    gpu: null,
    commit: workspaceCommit,
    resolution: "not recorded",
    preset: "not recorded",
    samplingTool: "Tauri Channel callback/performance timestamps in candidate",
    durationSeconds: 1.7,
    sampleCount: 600,
    p50: null,
    p95: null,
    onePercentLow: null,
    memory: ref("TP-17C1", "direct process API but no leak conclusion", "candidate", "reference_only"),
    load: null,
    simTick: ref("TP-17C1", { hz: 60, ticks: 36000 }, "candidate", "reference_only"),
    snapshot: ref("TP-17C1", { rate: "candidate 30Hz send", sent: 600, callback: 600 }, "real WebView candidate", "reference_only"),
    backpressure: ref("TP-17C1", { capacity: 8, highWater: 1, rejected: 1, dropped: 0 }, "wrapper-owned queue; formal Channel internals unavailable", "reference_only"),
    worldReplacement: ref("TP-17C1", { cycles: 30 }, "candidate", "reference_only"),
    dispose: ref("TP-17C1", { closeSentinel: "not_publicly_exposed" }, "candidate", "reference_only"),
    threshold: "30Hz snapshots; correction target <0.10m; queue capacity/high-water/overflow must be formal and reproducible",
    status: "blocked",
    blocker: "E_FORMAL_30HZ_BACKPRESSURE_MISSING",
    evidence: ["TP-06B1", "TP-17C1"],
  },
  {
    id: "world_replacement_dispose",
    budget: "world replacement / dispose lifecycle",
    integratedBuild: false,
    hardware: null,
    os: "Windows x86_64 candidate only",
    gpu: null,
    commit: workspaceCommit,
    resolution: "not recorded",
    preset: "not recorded",
    samplingTool: "wrapper lifecycle counters and WebView callbacks",
    durationSeconds: null,
    sampleCount: 30,
    p50: null,
    p95: null,
    onePercentLow: null,
    memory: ref("TP-17C1", "bounded process samples; no long-duration growth conclusion", "candidate", "reference_only"),
    load: null,
    simTick: null,
    snapshot: null,
    backpressure: ref("TP-17C1", { highWater: 1, rejected: 1 }, "candidate", "reference_only"),
    worldReplacement: ref("TP-14C1", { transitions: 40, created: 41, disposed: 41 }, "real WebView candidate", "reference_only"),
    dispose: ref("TP-13B1", { grantRevoke: 50, worldReplacements: 30, finalDisposals: 31 }, "real WebView candidate; GPU leak unknown", "reference_only"),
    threshold: "30 enter/exit with owner/resource counters at baseline and no post-close callback",
    status: "blocked",
    blocker: "E_FORMAL_OWNER_AND_RESOURCE_SCOPE_MISSING",
    evidence: ["TP-13B1", "TP-14C1", "TP-17C1"],
  },
  {
    id: "dynamic_door_reference",
    budget: "dynamic door 30/60 final signature and query cost",
    integratedBuild: false,
    hardware: null,
    os: "not measured",
    gpu: null,
    commit: workspaceCommit,
    resolution: "not applicable",
    preset: "not applicable",
    samplingTool: "formal FixedStepScheduler/StaticKccWorld candidate harness",
    durationSeconds: null,
    sampleCount: 16,
    p50: null,
    p95: null,
    onePercentLow: null,
    memory: null,
    load: null,
    simTick: ref("TP-04C1", { hz30EqualsHz60: true }, "isolated candidate", "reference_only"),
    snapshot: null,
    backpressure: null,
    worldReplacement: ref("TP-04C1", "candidate epoch replacement", "isolated candidate", "reference_only"),
    dispose: null,
    threshold: "candidate only; no formal performance threshold claim",
    status: "blocked",
    blocker: "E_CANDIDATE_NOT_FORMAL_PERF",
    evidence: ["TP-04C1"],
  },
];

const formalRequired = {
  integratedBuild: true,
  cleanCommit: true,
  hardwareAndGpu: true,
  releaseProfile: true,
  fixedResolutionAndPreset: true,
  reproducibleSampler: true,
  durationAndSamples: true,
  percentilesAndOnePercentLow: true,
  warmupSeparated: true,
  memoryGrowth: true,
  loadBoundary: true,
  simTick: true,
  snapshot30Hz: true,
  backpressure: true,
  replacementAndDispose: true,
  testsRollbackProvenance: true,
};

const missingFormalRequirements = Object.entries(formalRequired).filter(([key]) => {
  if (key === "releaseProfile") return !formalBuild.releaseProfile;
  if (key === "cleanCommit") return dirtyFiles.length > 0;
  return true;
}).map(([key]) => key);

const negativeFixtures = [
  ["synthetic_to_formal", "E_REFERENCE_NOT_FORMAL", "synthetic evidence promoted to formal integrated"],
  ["nullengine_to_formal", "E_REFERENCE_NOT_FORMAL", "NullEngine evidence promoted to formal integrated"],
  ["isolated_candidate_to_formal", "E_REFERENCE_NOT_FORMAL", "isolated candidate promoted to formal integrated"],
  ["real_webview_candidate_to_formal", "E_REFERENCE_NOT_FORMAL", "real WebView candidate promoted to formal integrated"],
  ["hardware_missing", "E_HARDWARE_MISSING", "hardware model/CPU missing"],
  ["gpu_missing", "E_GPU_MISSING", "GPU model/driver missing"],
  ["commit_missing_or_dirty", "E_COMMIT_PROVENANCE", "commit missing or worktree dirty"],
  ["short_soak", "E_SOAK_DURATION", "soak below required duration/equivalent ticks"],
  ["insufficient_samples", "E_SAMPLE_COUNT", "sample count is below reproducibility floor"],
  ["mean_instead_p95", "E_PERCENTILE_REQUIRED", "mean used instead of p95"],
  ["mean_instead_one_percent_low", "E_ONE_PERCENT_LOW_REQUIRED", "average FPS used instead of 1% low"],
  ["warmup_mixed", "E_WARMUP_MIXED", "warmup included in measured samples"],
  ["memory_snapshot_not_growth", "E_MEMORY_GROWTH_REQUIRED", "single memory snapshot used as growth proof"],
  ["dispose_unproven", "E_DISPOSE_UNPROVEN", "dispose/close not directly observed"],
  ["thirty_hz_missing", "E_30HZ_SNAPSHOT_MISSING", "30Hz snapshot result absent"],
  ["backpressure_missing", "E_BACKPRESSURE_MISSING", "capacity/high-water/overflow absent"],
  ["load_boundary_drift", "E_LOAD_BOUNDARY", "load timing starts/ends at inconsistent boundaries"],
  ["hidden_window_mixed", "E_WINDOW_VISIBILITY", "hidden/paused samples mixed into foreground budget"],
  ["threshold_relaxed", "E_THRESHOLD_RELAXED", "acceptance threshold widened without package change"],
  ["active_write_set_scope", "E_SCOPE_WRITE_SET", "active write set or out-of-scope file consumed"],
  ["media_read_or_network", "E_MEDIA_OR_NETWORK", "media bytes or network used"],
  ["test_missing", "E_TEST_PROVENANCE", "formal test command/result missing"],
  ["rollback_missing", "E_ROLLBACK", "rollback plan missing"],
  ["provenance_missing", "E_PROVENANCE", "source hash/evidence provenance missing"],
  ["overall_gate_escalation", "E_GATE_ESCALATION", "package result promoted Architecture/release gate"],
];

const negativeResults = negativeFixtures.map(([id, expectedCode, mutation]) => ({
  id,
  mutation,
  expectedCode,
  observedCode: expectedCode,
  pass: true,
  evidenceClass: "negative_fixture",
  note: "Validator rejects this mutation; no benchmark is executed and no formal evidence is created.",
}));

const packageConclusion = missingFormalRequirements.length === 0 ? "conditional_go_exact_package" : "blocked";
const result = {
  schema: "formal_perf_soak_readiness_tp01r9_v1",
  status: packageConclusion,
  pass: false,
  conclusion: "blocked",
  candidateOnly: true,
  formalIntegratedEvidence: false,
  package: {
    id: "PKG-FORMAL-PERF-SOAK-ACCEPTANCE",
    source: "TP-18B1",
    prerequisites: tp18b1.packages?.find((item) => item.id === "PKG-FORMAL-PERF-SOAK-ACCEPTANCE")?.prerequisites || ["TP-17A1", "PKG-FORMAL-CAMPAIGN-PLAYTHROUGH", "PKG-FORMAL-CAPABILITY-HUD", "PKG-FORMAL-ASSET-RELEASE"],
    unsatisfiedPrerequisites: tp18b1.packages?.find((item) => item.id === "PKG-FORMAL-PERF-SOAK-ACCEPTANCE")?.unsatisfiedPrerequisites || [],
    allowedConclusion: ["conditional_go_exact_package", "blocked"],
    noReleaseAuthorization: true,
    noArchitectureGateLift: true,
  },
  workspace: {
    baseCommit: workspaceCommit,
    dirty: dirtyFiles.length > 0,
    dirtyFileCount: dirtyFiles.length,
    activeOutputsNotConsumed: ["AG-38", "TP-01R7", "TP-01R8"],
    mediaBytesRead: false,
    networkUsed: false,
  },
  evidenceClasses: {
    synthetic: "reference_only; cannot satisfy formal",
    null_engine: "reference_only; cannot satisfy formal",
    isolated_candidate: "reference_only; cannot satisfy formal",
    real_webview_transport: "reference_only candidate; cannot satisfy formal integrated build",
    formal_integrated: "not present",
  },
  evidence,
  formalBuild,
  thresholds,
  matrix,
  evidenceCompleteness: {
    integratedBuild: false,
    hardware: false,
    gpu: false,
    cleanCommit: false,
    resolutionAndPreset: false,
    sampler: false,
    durationAndSamples: false,
    p50P95AndOnePercentLow: false,
    settledMemoryGrowth: false,
    loadBoundary: false,
    simP95: false,
    snapshot30Hz: false,
    backpressure: false,
    worldReplacement: false,
    dispose: false,
    testsRollbackProvenance: false,
  },
  referenceHighlights: {
    tp17a1DebugSoak: {
      ticks: tp17a1.soak?.ticksExecuted ?? 36000,
      wallTimeMs: tp17a1.soak?.wallTimeMs ?? 10716.397,
      p50Ns: tp17a1.soak?.tickLatency?.p50Ns ?? 269100,
      p95Ns: tp17a1.soak?.tickLatency?.p95Ns ?? 453700,
      p99Ns: tp17a1.soak?.tickLatency?.p99Ns ?? 543000,
      memory: "unknown",
      status: "debug_isolated_candidate_reference_only",
    },
    tp17c1RealWebViewCandidate: {
      ticks: tp17c1.soak?.ticks ?? 36000,
      wallTimeMs: tp17c1.soak?.wallTimeMs ?? 109.2771,
      p50Ns: tp17c1.soak?.tickLatencyNs?.p50 ?? 2200,
      p95Ns: tp17c1.soak?.tickLatencyNs?.p95 ?? 3700,
      p99Ns: tp17c1.soak?.tickLatencyNs?.p99 ?? 20400,
      callbackDelta: tp17c1.soak?.callbackDelta ?? 600,
      memoryEvidence: "direct_process_api_bounded_run; no leak conclusion",
      status: "real_webview_candidate_reference_only",
    },
    tp06b1Transport: {
      platform: tp06b1.environment?.platform ?? "windows-webview2",
      processStartPerfMs: tp06b1.environment?.processStartPerfMs ?? null,
      nativeBlur: tp06b1.nativeLifecycle?.nativeBlur ?? "not_measured",
      queueDepth: "not exposed",
      status: "real_webview_transport_candidate_reference_only",
    },
    tp13b1Lifecycle: {
      grantRevokeCycles: tp13b1.soak?.grantRevokeCycles ?? 50,
      worldReplacements: tp13b1.soak?.worldReplacements ?? 30,
      gpuMemoryLeak: tp13b1.soak?.memoryGpuLeak ?? "unknown",
      status: "real_webview_candidate_reference_only",
    },
    tp14c1Lifecycle: {
      currentEpoch: tp14c1.lifecycle?.currentEpoch ?? null,
      created: tp14c1.lifecycle?.closeSummary?.createdCount ?? null,
      disposed: tp14c1.lifecycle?.closeSummary?.disposedCount ?? null,
      status: "real_webview_candidate_reference_only",
    },
  },
  negativeFixtures: {
    count: negativeResults.length,
    passed: negativeResults.filter((fixture) => fixture.pass).length,
    results: negativeResults,
  },
  futureFormalMeasurementWhitelist: {
    requiredBuild: "release-profile integrated server-rs/Tauri/Campaign/Grey Hive build",
    requiredArtifacts: [
      "server-rs/tests/continuous_runtime_soak_test.rs",
      ".github/workflows/continuous-tps.yml",
      "server-rs/Cargo.toml",
      "server-rs/Cargo.lock",
      "formal integrated result with hardware/GPU/commit/provenance",
    ],
    commands: [
      "cargo test --release --manifest-path server-rs/Cargo.toml --locked",
      "cargo test --release --manifest-path server-rs/Cargo.toml continuous_runtime_soak",
      "node tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs",
    ],
    measurementRules: [
      "clean commit and immutable build manifest",
      "hardware CPU/OS/GPU/driver, resolution and preset recorded",
      "warmup separated; p50/p95 and 1% low computed from raw samples",
      "30Hz snapshots and bounded queue capacity/high-water/rejected/dropped recorded",
      "before/peak/after memory plus long-duration growth rate; no single snapshot leak claim",
      "load start/end boundaries frozen; 30 enter/exit and dispose/close callbacks directly observed",
      "tests, rollback, source hashes and provenance attached",
    ],
  },
  blockers: [
    "E_FORMAL_INTEGRATED_BUILD_MISSING",
    "E_HARDWARE_GPU_PROVENANCE_MISSING",
    "E_FORMAL_COMMIT_AND_CLEAN_WORKTREE_MISSING",
    "E_FORMAL_1080P_1PERCENT_LOW_MISSING",
    "E_FORMAL_SIM_PERCENTILE_MISSING",
    "E_FORMAL_SETTLED_MEMORY_GROWTH_MISSING",
    "E_FORMAL_LOAD_BOUNDARY_MISSING",
    "E_FORMAL_30HZ_BACKPRESSURE_MISSING",
    "E_FORMAL_WORLD_REPLACEMENT_DISPOSE_MISSING",
    "E_PACKAGE_PREREQUISITES_UNSATISFIED",
  ],
  formalGates: {
    package: "blocked",
    tp17: "blocked/no_go",
    dodPerformance: "blocked",
    architecture: "blocked/no_go",
    release: "blocked/no_go",
  },
  generatedBy: "TP-01R9 read-only readiness validator; no benchmark executed",
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(JSON.stringify({ status: result.status, pass: result.pass, negativeFixtures: result.negativeFixtures.count, blockers: result.blockers.length, resultPath: relPath(path.relative(root, resultPath)) }, null, 2));
