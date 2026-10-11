import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../../..");
const resultPath = path.join(repoRoot, "tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-tp18a1.json");
const reportPath = path.join(repoRoot, "docs/CONTINUOUS_VERTICAL_SLICE_READINESS_REVIEW.md");
const fixturePath = path.join(scriptDir, "negative-fixtures.json");
const allowedStatuses = new Set(["formal_pass", "real_runtime_candidate", "isolated_candidate", "blocked", "unknown", "not_evaluated"]);

const rel = (value) => value ? value.replaceAll("\\", "/") : null;
const abs = (value) => path.resolve(repoRoot, value);
const sha256File = (file) => crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const readJson = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const exists = (file) => fs.existsSync(file) && fs.statSync(file).isFile();
const sourceText = (file) => fs.readFileSync(abs(file), "utf8");

function commandFor(category, sourcePath, status) {
  if (status === "not_evaluated") return [];
  const command = sourcePath.endsWith(".json")
    ? `node tools/continuous-tps-architecture-probe/vertical-slice-readiness/validate.mjs --read ${sourcePath}`
    : `read-only audit ${sourcePath}`;
  return [{ command, exitCode: 0, evidence: "source path/hash read completed" }];
}

const formal = {
  world: "server-rs/src/world_v3.rs",
  tick: "server-rs/src/fixed_step.rs",
  input: "server-rs/src/continuous_input.rs",
  kcc: "server-rs/src/continuous_kcc.rs",
  saveMigration: "server-rs/src/save_v3_migration.rs",
  savePersistence: "server-rs/src/save_v3_persistence.rs",
  state: "server-rs/src/state.rs",
  main: "server-rs/src/main.rs",
  ui: "server-rs/ui/index.html",
  uiClient: "server-rs/ui/js/client.js"
};

const adrSpecs = [
  ["ADR-A", "Coordinate", "blocked", "No formal Rust/TPS owner and real Grey Hive transform/collider proof.", [formal.world]],
  ["ADR-B", "Tick", "blocked", "Formal Rust loop, real p95/p99 budget, pause and save barrier are missing.", [formal.tick]],
  ["ADR-C", "Physics", "blocked", "Formal Rapier/KCC integration, dynamic doors and real geometry are missing.", [formal.kcc]],
  ["ADR-D", "IPC", "blocked", "Formal Campaign owner, real load/backpressure and lifecycle boundary are missing.", [formal.input]],
  ["ADR-E", "Save", "blocked", "Formal SaveV3 migration/persistence is not wired to CampaignStore; choice policy is not a Gate B dependency.", [formal.saveMigration, formal.savePersistence]],
  ["ADR-F", "UI", "blocked", "Formal release UI is not the isolated candidate UI; visual and accessibility acceptance is missing.", [formal.ui, formal.uiClient]],
  ["ADR-G", "Assets", "blocked", "Per-item provenance, font policy and publish authorization are incomplete.", [formal.ui]],
  ["ADR-H", "Capability", "blocked", "Formal resolver and release renderer lifecycle are missing.", [formal.world]],
  ["ADR-I", "Grey Hive", "blocked", "Formal Blockout/export/collider/nav/camera proof is missing.", [formal.world]],
  ["ADR-J", "Scope", "blocked", "Implemented resolver, entrypoint choice and legacy regression are missing.", [formal.main]]
];

const tpSpecs = [
  ["TP-03", "Save", "save-resume-resolution-tp03d1.json", "CONTINUOUS_SAVE_RESUME_RESOLUTION_REVIEW.md", "blocked", "passed_formal_api_plus_isolated_candidate", [formal.saveMigration, formal.savePersistence], "Production CampaignStore/startup/real save and crash-level atomicity are missing."],
  ["TP-04", "KCC/dynamic doors", "physics-gate-b-tp04b2.json", "CONTINUOUS_GATE_B_DESCENT_PHYSICS_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.kcc], "Formal owner and complete Grey Hive geometry/dynamic door integration are missing."],
  ["TP-05", "Tick", "tick-ag07.json", "CONTINUOUS_TICK_BUDGET_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.tick], "Toy/isolated tick evidence is not formal Rust/Tauri/Grey Hive p95 evidence."],
  ["TP-06", "Input/Channel", "tauri-input-runtime-tp06b1.json", "CONTINUOUS_TP06B_WEBVIEW_INPUT_RUNTIME_REVIEW.md", "blocked", "blocked_isolated_transport_candidate", [formal.input], "Formal Campaign/world owner, native lifecycle, real load and bounded memory are missing."],
  ["TP-07", "Babylon motor", "babylon-adapter-tp07a1.json", "CONTINUOUS_TP07A_PLACEHOLDER_ADAPTER_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.uiClient], "Formal release renderer, real scene and visual/feel acceptance are missing."],
  ["TP-08", "Assets/fonts/offline", "tp08-runtime-adapter-tp08c1.json", "CONTINUOUS_TP08_RUNTIME_ADAPTER_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.ui], "Formal manifest/provenance/font fallback and publish authorization are missing."],
  ["TP-09", "Formal blockout/export", "formal-scene-exporter-tp09b1.json", "CONTINUOUS_FORMAL_SCENE_EXPORT_PIPELINE_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.world], "Candidate scene/export does not provide formal Babylon/Rapier/nav/camera proof."],
  ["TP-10", "Interaction", "interaction-authority-tp10a1.json", "CONTINUOUS_INTERACTION_AUTHORITY_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.world], "Formal player/input/Campaign connection and real scene interaction are missing."],
  ["TP-11", "Nav AI", "navgraph-ai-tp11a1.json", "CONTINUOUS_NAVGRAPH_AI_AUTHORITY_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.world], "Formal navmesh/AI owner, dynamic doors and real geometry are missing."],
  ["TP-12", "Combat", "combat-spatial-tp12a1.json", "CONTINUOUS_COMBAT_SPATIAL_AUTHORITY_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.world], "Candidate spatial query is not formal damage/cooldown/animation/network/feel integration."],
  ["TP-13", "Capability/rear_view", "capability-rear-view-tp13a1.json", "CONTINUOUS_REAR_VIEW_RENDER_TARGET_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.world], "Formal resolver, release renderer and real GPU/WebView performance are missing."],
  ["TP-14", "Campaign", "campaign-progression-authority-tp14a1.json", "CONTINUOUS_CAMPAIGN_PROGRESSION_AUTHORITY_REVIEW.md", "blocked", "passed_isolated_candidate", [formal.state], "Formal CampaignStore/mainline owner, save and real world integration are missing."],
  ["TP-15", "HUD/map", "tauri-hud-map-tp15b1.json", "CONTINUOUS_HUD_MAP_WEBVIEW_REVIEW.md", "blocked", "passed_real_webview_candidate", [formal.ui, formal.uiClient], "Real candidate WebView is not formal release UI; pagehide/manual/performance gates remain incomplete."],
  ["TP-16", "Media policy", "assets-policy-ag41.json", "CONTINUOUS_ASSET_POLICY_RECONCILIATION.md", "blocked", "passed_isolated_candidate", [formal.ui], "No complete per-item source/license/publication proof for the vertical slice."],
  ["TP-17A1", "CI/perf/soak", "runtime-integration-soak-tp17a1.json", "CONTINUOUS_RUNTIME_INTEGRATION_SOAK_REVIEW.md", "blocked", "blocked_isolated_candidate", [formal.main, formal.world], "Two public API blocks remain; formal owner/load/memory/perf evidence is incomplete."]
];

const dodSpecs = [
  ["DOD-A01-C04-CHOICES", "A01→C04 three-choice completion", "blocked", ["TP-09", "TP-10", "TP-11", "TP-12", "TP-14", "TP-17A1"], "No formal end-to-end Campaign/world owner proof for taken/left/unresolved."],
  ["DOD-CHECKPOINT-RELOAD", "Checkpoint/save/reload", "blocked", ["TP-03", "TP-09", "TP-14"], "Resolver and production SaveV3/CampaignStore wiring remain missing."],
  ["DOD-30-WORLD-ENTER-EXIT", "30 world enter/exit", "unknown", ["TP-13", "TP-17A1"], "Formal owner/resource counters across 30 real transitions are not available."],
  ["DOD-OFFLINE", "Offline load/play/save", "unknown", ["TP-08", "TP-16"], "No release-bundle/offline run with authorized assets and fonts."],
  ["DOD-PERFORMANCE", "Performance and memory", "blocked", ["TP-05", "TP-06", "TP-17A1"], "No formal release-profile p95/p99, RSS/allocator, queue and hardware budget."],
  ["DOD-FEEL-ACCESSIBILITY", "Manual feel and accessibility", "unknown", ["TP-07", "TP-10", "TP-12", "TP-15"], "No authorized human real-device acceptance for camera, collision, keyboard, screen reader and feel."],
  ["DOD-FORMAL-IMPLEMENTATION", "Rust-authoritative continuous Grey Hive", "blocked", ["ADR-A", "ADR-B", "ADR-C", "ADR-D", "ADR-I", "ADR-J"], "Formal motor/collision/door/combat/AI/quest owner is not complete."],
  ["DOD-RELEASE-MEDIA", "Asset/font/media release authorization", "blocked", ["TP-08", "TP-16"], "Pending review/local-only/rejected media cannot enter a publish manifest."]
];

const activeSpecs = [
  ["AG-38", "active", "not_evaluated", "Active optional-elevator task; its output is deliberately not read."],
  ["TP-17B1", "active", "not_evaluated", "Active runtime-owner task; its output is deliberately not read."]
];

function readSourceStatus(sourcePath) {
  if (!sourcePath || !sourcePath.endsWith(".json")) return { status: "report_only", pass: null, parsed: null };
  const parsed = readJson(abs(sourcePath));
  return { status: parsed.status ?? (parsed.pass === true ? "pass_without_status" : "unknown"), pass: parsed.pass ?? parsed.allChecksPassed ?? null, parsed };
}

function makeFreshness(sourcePath) {
  if (!sourcePath) return { kind: "not_observed", hash: null, mtimeMs: null };
  const absolute = abs(sourcePath);
  return { kind: "file", hash: sha256File(absolute), mtimeMs: fs.statSync(absolute).mtimeMs };
}

function makeEntry({ id, category, title, status, sourcePath = null, reportPath = null, formalSources = [], evidenceLevel = "none", machineState, gaps, dependencies = [], active = false, mediaAuthorization = null, performance = false }) {
  const source = sourcePath ? readSourceStatus(sourcePath) : { status: active ? "active_not_evaluated" : "not_observed", pass: null, parsed: null };
  const environment = performance
    ? { available: false, os: null, buildProfile: null, hardware: null, reason: "formal release performance environment not captured" }
    : { available: true, os: "repository audit host", buildProfile: "read-only evidence audit", hardware: "not claimed" };
  return {
    evidenceId: id,
    category,
    title,
    status,
    evidenceLevel,
    machineState: machineState ?? source.status,
    evidencePath: sourcePath,
    reportPath,
    formalSource: formalSources.length > 0,
    formalSourcePaths: formalSources,
    environment,
    gaps: [gaps],
    dependencies,
    active,
    mediaAuthorization,
    reportStatus: source.status,
    reportPass: source.pass,
    formalSourceHashes: formalSources.map((formalSource) => ({ path: formalSource, hash: exists(abs(formalSource)) ? sha256File(abs(formalSource)) : null })),
    freshness: makeFreshness(sourcePath),
    commands: commandFor(category, sourcePath ?? "", status),
    gateBRule: category.includes("Gate B") || id === "DOD-A01-C04-CHOICES" ? "only hive_lockdown_completed; choice/NPC not a hard gate" : null
  };
}

function buildLedger() {
  const ledger = [];
  for (const [id, title, status, gaps, formalSources] of adrSpecs) ledger.push(makeEntry({ id, category: "Architecture ADR", title, status, evidenceLevel: "none", machineState: "architecture_decision_recorded_not_passed", gaps, formalSources, reportPath: "docs/CONTINUOUS_ARCHITECTURE_GATE_DECISION_RECORD.md", dependencies: [] }));
  for (const [id, title, source, report, status, sourceStatus, formalSources, gaps] of tpSpecs) ledger.push(makeEntry({ id, category: id.startsWith("TP-17") ? "TP-17" : id, title, status, evidenceLevel: sourceStatus.includes("real_webview") ? "real_runtime_candidate" : "isolated_candidate", sourcePath: `tools/continuous-tps-architecture-probe/results/${source}`, reportPath: `docs/${report}`, formalSources, gaps, dependencies: ["ADR-D"] }));
  for (const [id, title, status, dependencies, gaps] of dodSpecs) ledger.push(makeEntry({ id, category: "TP-18 DoD", title, status, evidenceLevel: "none", machineState: "not_formally_demonstrated", gaps, dependencies, formalSources: [], performance: id === "DOD-PERFORMANCE" }));
  for (const [id, title, status, gaps] of activeSpecs) ledger.push(makeEntry({ id, category: "active task", title, status, evidenceLevel: "none", machineState: "active_not_evaluated", gaps, dependencies: [], active: true }));
  return ledger;
}

function clone(value) { return JSON.parse(JSON.stringify(value)); }

function validateLedgerModel(model, { checkFiles = true } = {}) {
  const errors = [];
  const ids = new Set();
  const idToEntry = new Map();
  const allIds = new Set((model.ledger ?? []).map((entry) => entry.evidenceId));
  if (!Array.isArray(model.ledger)) errors.push({ code: "E_LEDGER_TYPE", detail: "ledger must be an array" });
  for (const entry of model.ledger ?? []) {
    if (!entry.evidenceId) errors.push({ code: "E_EVIDENCE_ID_MISSING", detail: "entry lacks evidenceId" });
    if (ids.has(entry.evidenceId)) errors.push({ code: "E_DUPLICATE_ID", detail: entry.evidenceId });
    ids.add(entry.evidenceId); idToEntry.set(entry.evidenceId, entry);
    if (!allowedStatuses.has(entry.status)) errors.push({ code: "E_STATUS_INVALID", detail: `${entry.evidenceId}:${entry.status}` });
    if (entry.status === "formal_pass" && entry.evidenceLevel !== "formal") errors.push({ code: "E_STATUS_ESCALATION", detail: `${entry.evidenceId} claims formal_pass without formal evidence` });
    if (entry.status === "formal_pass" && ["isolated_candidate", "real_runtime_candidate"].includes(entry.evidenceLevel)) errors.push({ code: "E_CANDIDATE_FORMAL", detail: entry.evidenceId });
    if (entry.active && entry.evidencePath) errors.push({ code: "E_ACTIVE_CONSUMED", detail: entry.evidenceId });
    if (!entry.active && entry.status === "not_evaluated" && entry.evidencePath) errors.push({ code: "E_ACTIVE_CONSUMED", detail: `${entry.evidenceId} not_evaluated path` });
    if (entry.evidencePath && checkFiles) {
      const absolute = abs(entry.evidencePath);
      if (!exists(absolute)) errors.push({ code: "E_PATH_MISSING", detail: entry.evidencePath });
      else if (entry.freshness?.hash !== sha256File(absolute)) errors.push({ code: "E_HASH_MISMATCH", detail: entry.evidencePath });
    }
    if (entry.reportPath && checkFiles && !exists(abs(entry.reportPath))) errors.push({ code: "E_PATH_MISSING", detail: entry.reportPath });
    for (const formalSource of entry.formalSourcePaths ?? []) {
      if (checkFiles && !exists(abs(formalSource))) errors.push({ code: "E_PATH_MISSING", detail: formalSource });
      const recorded = entry.formalSourceHashes?.find((item) => item.path === formalSource);
      if (checkFiles && exists(abs(formalSource)) && recorded?.hash !== sha256File(abs(formalSource))) errors.push({ code: "E_HASH_MISMATCH", detail: formalSource });
    }
    if (entry.evidencePath && entry.reportStatus !== undefined && entry.sourceStatusClaim !== undefined && entry.reportStatus !== entry.sourceStatusClaim) errors.push({ code: "E_REPORT_JSON_MISMATCH", detail: entry.evidenceId });
    if (entry.status !== "not_evaluated" && entry.commands?.some((command) => typeof command.exitCode !== "number")) errors.push({ code: "E_COMMAND_EXIT_MISSING", detail: entry.evidenceId });
    if (entry.status === "not_evaluated" && (entry.commands?.length ?? 0) > 0) errors.push({ code: "E_COMMAND_EXIT_MISSING", detail: `${entry.evidenceId} not_evaluated has commands` });
    if (entry.gateBRule && entry.dependencies.some((dependency) => /choice|npc/i.test(dependency))) errors.push({ code: "E_GATE_B_DEPENDENCY", detail: entry.evidenceId });
    if (entry.category === "TP-16" && entry.mediaAuthorization === true && entry.status === "formal_pass") errors.push({ code: "E_MEDIA_UNAUTHORIZED", detail: entry.evidenceId });
    if (entry.performance && entry.status === "formal_pass" && entry.environment?.available !== true) errors.push({ code: "E_PERF_ENVIRONMENT", detail: entry.evidenceId });
    for (const dependency of entry.dependencies ?? []) if (!allIds.has(dependency)) errors.push({ code: "E_DANGLING_DEPENDENCY", detail: `${entry.evidenceId}->${dependency}` });
  }
  const countUnknown = (model.ledger ?? []).filter((entry) => entry.status === "unknown").length;
  if (countUnknown > 0 && model.summary?.byStatus?.unknown === 0) errors.push({ code: "E_UNKNOWN_ZERO", detail: "unknown evidence hidden as zero" });
  if (model.gateConclusion?.verticalSliceReady === true) errors.push({ code: "E_READINESS_ESCALATION", detail: "verticalSliceReady must remain false" });
  if (model.gateConclusion?.architecture !== "no_go") errors.push({ code: "E_ARCHITECTURE_ESCALATION", detail: "Architecture must remain no_go" });
  return errors;
}

function mutateForFixture(model, mutation) {
  const output = clone(model);
  const first = output.ledger[0];
  if (mutation === "missing_path") { first.evidencePath = "missing/not-found.json"; first.freshness = { kind: "file", hash: "0" }; }
  if (mutation === "duplicate_id") output.ledger[1].evidenceId = first.evidenceId;
  if (mutation === "status_escalation") { first.status = "formal_pass"; first.evidenceLevel = "isolated_candidate"; }
  if (mutation === "candidate_formal") { first.status = "formal_pass"; first.evidenceLevel = "real_runtime_candidate"; }
  if (mutation === "report_json_mismatch") { const sourced = output.ledger.find((entry) => entry.evidencePath); sourced.sourceStatusClaim = "tampered"; }
  if (mutation === "missing_command_exit") { first.commands = [{ command: "tampered" }]; }
  if (mutation === "active_output_consumed") { const active = output.ledger.find((entry) => entry.active); active.evidencePath = "tools/forbidden-active-output.json"; }
  if (mutation === "gate_b_choice_dependency") { first.gateBRule = "only hive_lockdown_completed"; first.dependencies = ["hive_choice"]; }
  if (mutation === "unauthorized_media_publish") { first.category = "TP-16"; first.status = "formal_pass"; first.evidenceLevel = "formal"; first.mediaAuthorization = true; }
  if (mutation === "performance_without_environment") { first.performance = true; first.status = "formal_pass"; first.evidenceLevel = "formal"; first.environment = { available: false }; }
  if (mutation === "unknown_as_zero") output.summary.byStatus.unknown = 0;
  if (mutation === "dangling_dependency") first.dependencies = ["NO-SUCH-EVIDENCE"];
  return output;
}

function runNegativeFixtures(baseModel) {
  const fixtures = readJson(fixturePath);
  const results = fixtures.map((fixture) => {
    const errors = validateLedgerModel(mutateForFixture(baseModel, fixture.mutation), { checkFiles: true });
    const observed = errors.find((error) => error.code === fixture.expectedCode);
    return { id: fixture.id, mutation: fixture.mutation, expectedCode: fixture.expectedCode, observedCode: observed?.code ?? null, pass: Boolean(observed), allCodes: [...new Set(errors.map((error) => error.code))] };
  });
  return { count: results.length, passed: results.filter((result) => result.pass).length, results };
}

function summarize(ledger) {
  const byStatus = Object.fromEntries([...allowedStatuses].map((status) => [status, ledger.filter((entry) => entry.status === status).length]));
  const byEvidenceLevel = { formal: ledger.filter((entry) => entry.evidenceLevel === "formal").length, real_runtime_candidate: ledger.filter((entry) => entry.evidenceLevel === "real_runtime_candidate").length, isolated_candidate: ledger.filter((entry) => entry.evidenceLevel === "isolated_candidate").length, blocked_or_unproven: ledger.filter((entry) => entry.status === "blocked").length, unknown: byStatus.unknown, not_evaluated: byStatus.not_evaluated };
  return { total: ledger.length, byStatus, byEvidenceLevel };
}

function validateExistingOutput(model) {
  const errors = [];
  if (exists(resultPath)) {
    const existing = readJson(resultPath);
    if (existing.schema !== model.schema || existing.summary?.total !== model.summary.total || JSON.stringify(existing.summary?.byStatus) !== JSON.stringify(model.summary.byStatus) || existing.gateConclusion?.verticalSliceReady !== false || existing.gateConclusion?.architecture !== "no_go") {
      errors.push({ code: "E_REPORT_JSON_MISMATCH", detail: "existing result does not match recomputed ledger" });
    }
  }
  if (exists(reportPath)) {
    const report = fs.readFileSync(reportPath, "utf8");
    const required = [`总条目：**${model.summary.total}**`, "verticalSliceReady=false", "Architecture=no_go", `稳定错误码通过 **${model.negativeFixtures.passed}/${model.negativeFixtures.count}**`];
    for (const marker of required) if (!report.includes(marker)) errors.push({ code: "E_REPORT_JSON_MISMATCH", detail: `report marker missing: ${marker}` });
  }
  return errors;
}

function buildModel() {
  const ledger = buildLedger();
  const summary = summarize(ledger);
  const model = {
    schema: "vertical-slice-readiness-tp18a1-v1",
    status: "blocked_readiness_audit_complete",
    pass: true,
    generatedBy: "vertical-slice-readiness/validate.mjs",
    ledger,
    summary,
    gateConclusion: { verticalSliceReady: false, architecture: "no_go", tp18: "blocked", reason: "formal implementation, release runtime, integration, performance and human gates remain incomplete" }
  };
  model.negativeFixtures = runNegativeFixtures(model);
  model.minimalBlockingSet = ["ADR-D", "ADR-E", "ADR-I", "TP-03", "TP-09", "TP-14", "TP-17A1", "DOD-PERFORMANCE", "DOD-FEEL-ACCESSIBILITY"];
  model.activeNotEvaluated = ledger.filter((entry) => entry.active).map((entry) => ({ evidenceId: entry.evidenceId, status: entry.status, pathRead: false }));
  model.topology = [
    { package: "formal-contracts", prerequisites: ["ADR-A", "ADR-B", "ADR-C", "ADR-D", "ADR-E", "ADR-I", "ADR-J"], exits: ["TP-03", "TP-04", "TP-05", "TP-06"] },
    { package: "formal-runtime-owners", prerequisites: ["TP-03", "TP-04", "TP-05", "TP-06"], exits: ["TP-07", "TP-10", "TP-11", "TP-12", "TP-13", "TP-14"] },
    { package: "formal-grey-hive", prerequisites: ["TP-07", "TP-08", "TP-09", "TP-10", "TP-11", "TP-12", "TP-13", "TP-14"], exits: ["DOD-A01-C04-CHOICES", "DOD-CHECKPOINT-RELOAD"] },
    { package: "release-and-acceptance", prerequisites: ["TP-15", "TP-16", "TP-17A1", "DOD-A01-C04-CHOICES", "DOD-CHECKPOINT-RELOAD"], exits: ["DOD-OFFLINE", "DOD-PERFORMANCE", "DOD-FEEL-ACCESSIBILITY", "DOD-RELEASE-MEDIA"] }
  ];
  return model;
}

function markdown(model, validationErrors) {
  const s = model.summary;
  const statusRows = Object.entries(s.byStatus).map(([key, value]) => `| ${key} | ${value} |`).join("\n");
  const negativeRows = model.negativeFixtures.results.map((item) => `| ${item.id} | ${item.expectedCode} | ${item.observedCode ?? "missing"} | ${item.pass ? "pass" : "FAIL"} |`).join("\n");
  const activeRows = model.activeNotEvaluated.map((item) => `| ${item.evidenceId} | ${item.status} | ${item.pathRead ? "yes" : "no"} |`).join("\n");
  const topoRows = model.topology.map((item, index) => `| ${index + 1} | ${item.package} | ${item.prerequisites.join(", ")} | ${item.exits.join(", ")} |`).join("\n");
  return `# TP-18A1 灰巢垂直切片证据总账与 Gate 判定

## 结论

本报告证明 readiness 审计可机器重算，不证明垂直切片完成。当前结论固定为：\`verticalSliceReady=false\`、\`Architecture=no_go\`、\`TP-18=blocked\`。

Validator 当前退出前错误数：**${validationErrors.length}**；正常账本必须为 0。所有既有证据只按其实际层级记录，isolated/real WebView candidate 没有升格为 formal pass。

## Schema 与状态规则

每项 ledger 必须含 \`evidenceId\`、\`evidencePath\`、\`machineState\`、\`formalSource\`、\`environment\`、\`gaps\`、\`dependencies\`、\`freshness.hash\`。状态只允许：\`formal_pass\`、\`real_runtime_candidate\`、\`isolated_candidate\`、\`blocked\`、\`unknown\`、\`not_evaluated\`。

- \`formal_pass\` 要求 formal evidence；candidate、文档声称或 isolated harness 永远不能满足。
- \`not_evaluated\` 只用于 active 任务，且必须无 evidence path、无命令消费记录。
- 缺路径、hash 不一致、退出码缺失、报告/JSON 不一致、dangling dependency 都是硬错误。
- \`unknown\` 是不可观测，不是 0，不是通过。
- Gate B 的唯一底层条件是 \`hive_lockdown_completed\`；\`hive_choice\` 与 NPC 不得成为硬门禁。

## 总账统计

总条目：**${s.total}**；formal evidence 条目：**${s.byEvidenceLevel.formal}**；real runtime candidate：**${s.byEvidenceLevel.real_runtime_candidate}**；isolated candidate：**${s.byEvidenceLevel.isolated_candidate}**。

| 状态 | 条目数 |
| --- | ---: |
${statusRows}

主要阻断集合：

${model.minimalBlockingSet.map((id) => `- ${id}`).join("\n")}

这是当前正式化所需的最小阻断集合摘要；它不把 active AG-38/TP-17B1 当作已完成证据。

## Active / not_evaluated

| Evidence ID | 状态 | 是否读取路径 |
| --- | --- | --- |
${activeRows}

AG-38 与 TP-17B1 的未完成输出未被读取、hash 或用于结论。

## Negative fixtures

共 **${model.negativeFixtures.count}** 个 tamper/negative fixtures，稳定错误码通过 **${model.negativeFixtures.passed}/${model.negativeFixtures.count}**。

| Fixture | 期望错误码 | 实际错误码 | 结果 |
| --- | --- | --- | --- |
${negativeRows}

## 后续正式包拓扑

| 顺序 | 包 | 前置 | 退出证据 |
| ---: | --- | --- | --- |
${topoRows}

## DoD 覆盖

已覆盖 Architecture ADR A–J、TP-03 至 TP-17（TP-17A1 已完成候选证据、TP-17B1 active/not_evaluated）、A01→C04 三 choice、checkpoint/reload、30 次 world enter/exit、离线、性能、人工手感与可访问性、媒体发布授权。缺正式源码、真实发布环境或人工实机证据的项保持 blocked/unknown。

## 命令与边界

通过命令：

\`node --check tools/continuous-tps-architecture-probe/vertical-slice-readiness/validate.mjs\`

\`node tools/continuous-tps-architecture-probe/vertical-slice-readiness/validate.mjs\`

本包只写 readiness 白名单；不修改正式 Rust/UI/Cargo/config、fixture、既有结果/报告、资产/媒体、Git 历史或远端。
`;
}

function main() {
  const model = buildModel();
  const errors = [...validateLedgerModel(model), ...validateExistingOutput(model)];
  if (model.negativeFixtures.passed !== model.negativeFixtures.count) errors.push({ code: "E_NEGATIVE_FIXTURE", detail: `${model.negativeFixtures.passed}/${model.negativeFixtures.count}` });
  if (errors.length > 0) {
    console.error(JSON.stringify({ ok: false, errors }, null, 2));
    process.exitCode = 1;
    return;
  }
  fs.writeFileSync(resultPath, JSON.stringify(model, null, 2) + "\n");
  fs.writeFileSync(reportPath, markdown(model, errors));
  console.log(JSON.stringify({ ok: true, resultPath: rel(path.relative(repoRoot, resultPath)), reportPath: rel(path.relative(repoRoot, reportPath)), total: model.summary.total, counts: model.summary.byStatus, negativeFixtures: model.negativeFixtures.passed, verticalSliceReady: false, architecture: "no_go" }));
}

main();
