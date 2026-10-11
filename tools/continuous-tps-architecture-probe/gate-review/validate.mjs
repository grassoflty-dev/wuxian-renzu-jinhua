#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(scriptDir, "..", "..", "..");
const manifestPath = path.join(scriptDir, "review-v1.json");
const resultPath = path.join(scriptDir, "..", "results", "gate-ag19.json");
const readJson = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const manifest = readJson(manifestPath);
const ag15 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "gate-ag15.json"));
const ag16 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "tauri-channel-ag16.json"));
const ag17 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "layout-geometry-ag17.json"));
const ag18 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "layout-repair-ag18.json"));
const ag11 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "save-ag11.json"));
const ag13 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "assets-ag13.json"));
const ag14 = readJson(path.join(root, "tools", "continuous-tps-architecture-probe", "results", "modules-ag14.json"));
const errors = [];
const err = (code, detail) => errors.push({ code, detail });
const has = (list, text) => Array.isArray(list) && list.some((item) => String(item).toLowerCase().includes(text.toLowerCase()));

if (manifest.schema !== "continuous_tp01_conditional_gate_review_ag19_v1") err("E_SCHEMA", "wrong AG-19 manifest schema");
if (manifest.decision.overall !== "no_go") err("E_OVERALL_NOT_NO_GO", "overall Architecture Gate must remain no_go");
if (manifest.decision.architecture_gate_passed !== false) err("E_FALSE_GATE_PASS", "overall gate cannot be marked passed");
if (manifest.decision.tp02_specific !== "conditional_go") err("E_TP02_DECISION", "TP-02 must be exactly conditional_go or the package must be stopped");
if (manifest.decision.publish_eligible !== false) err("E_PUBLISH_CONFLICT", "publish eligibility must remain false");

if (ag15.gateDecision !== "no_go" || ag15.publishDecision?.eligible !== false) err("E_AG15_BASELINE_CHANGED", "AG-15 no_go and publish ineligible baseline must be preserved");
if (ag16.api_compiles !== true || ag16.native_channel_exercised !== true || ag16.webview_roundtrip_exercised !== false || ag16.gate_d !== "pending") err("E_AG16_OVERCLAIM", "AG-16 must remain native-only with WebView false and Gate D pending");
if (ag17.status !== "blocked_geometry_candidate") err("E_AG17_HISTORY_LOST", "AG-17 blocked geometry failure must remain preserved");
if (ag18.status !== "passed_isolated_v5_candidate" || !Array.isArray(ag18.errors) || ag18.errors.length !== 0 || ag18.formal_gate_note?.toLowerCase().includes("gate i") === false) err("E_AG18_OVERCLAIM", "AG-18 must be an isolated pass only, not formal Gate I");
if (ag11.status !== "isolated_save_v3_probe_passed_with_formal_gaps" || ag11.currentV2Evidence?.v2ChoiceIsOptional !== true || ag11.currentV2Evidence?.v4GateBChoiceCandidateIsRequired !== true) err("E_AG11_POLICY_DRIFT", "AG-11 SaveV3 and Baizhi choice policy evidence must remain unresolved");
if (ag13.publishDecision?.eligible !== false) err("E_AG13_PUBLISH_DRIFT", "AG-13 publish eligibility must remain false");
if (ag14.status !== "isolated_module_boundary_probe_passed_with_formal_gaps" || ag14.observed?.activeCampaignState !== "CampaignState") err("E_AG14_ROUTER_DRIFT", "AG-14 active Campaign/legacy split must remain unimplemented");

const expectedAdr = new Map(manifest.adr_review.map((adr) => [adr.id, adr.status]));
for (const [id, status] of [["ADR-A", "conditionally_frozen"], ["ADR-B", "conditionally_frozen"], ["ADR-C", "conditionally_frozen"], ["ADR-D", "conditionally_frozen"], ["ADR-E", "blocked"], ["ADR-F", "blocked"], ["ADR-G", "blocked"], ["ADR-H", "conditionally_frozen"], ["ADR-I", "blocked"], ["ADR-J", "blocked"]]) {
  if (expectedAdr.get(id) !== status) err("E_ADR_STATUS", `${id} expected ${status}`);
}
if (manifest.adr_review.length !== 10) err("E_ADR_COUNT", "AG-19 must review all ten ADRs");

const pkg = manifest.tp02_package;
if (pkg.status !== "conditional_go") err("E_TP02_PACKAGE_STATUS", "TP-02 package status mismatch");
for (const required of ["server-rs/src/world_v3.rs", "server-rs/src/lib.rs", "server-rs/tests/world_v3_contract_test.rs"]) if (!pkg.allowed_writes.some((entry) => entry.startsWith(required))) err("E_TP02_SCOPE", `missing allowed TP-02 path ${required}`);
for (const forbidden of ["Rapier", "Tauri Channel", "Babylon", "campaign.rs", "SaveV3 filesystem migration", "Baizhi choice policy", "asset provenance"]) if (!has(pkg.forbidden, forbidden)) err("E_TP02_FORBIDDEN_GAP", `missing TP-02 forbidden item ${forbidden}`);
for (const required of ["Transform serde roundtrip", "WorldStateV3 and WorldView serde roundtrip", "SaveV3 DTO roundtrip", "serialized JSON denylist", "one-writer API test", "existing real workspace cargo test"]) if (!has(pkg.required_tests, required)) err("E_TP02_TEST_SCOPE", `missing TP-02 test ${required}`);
for (const gate of ["TP-03 SaveV3 production migrator", "TP-04 Rapier/KCC", "TP-05 fixed tick", "TP-06 real Tauri/WebView", "TP-07 Babylon adapter", "TP-08 asset publication", "TP-09 formal Grey Hive blockout/colliders/nav/camera"]) if (!pkg.downstream_hard_gates.includes(gate)) err("E_HARD_GATE_SCOPE", `missing downstream hard gate ${gate}`);
if (pkg.minimal_contract?.WorldStateV3?.authority?.includes("one mutable WorldStateV3 owner") !== true) err("E_SINGLE_WRITER", "WorldStateV3 single writer is not explicit");
if (pkg.minimal_contract?.WorldView?.authority?.includes("no mutable writer") !== true) err("E_VIEW_BOUNDARY", "WorldView read-only boundary is not explicit");
if (pkg.minimal_contract?.IPC_read_only?.forbidden?.toLowerCase().includes("tauri::ipc::channel") !== true) err("E_IPC_BOUNDARY", "TP-02 must not implement Tauri Channel");
if (pkg.minimal_contract?.SaveV3_read_only?.forbidden?.toLowerCase().includes("filesystem writes") !== true) err("E_SAVE_BOUNDARY", "TP-02 must not write SaveV3 files");

if (manifest.policy_open.baizhi_choice_gate_b.status !== "unresolved" || manifest.policy_open.baizhi_choice_gate_b.do_not_assume !== true) err("E_POLICY_ASSUMED", "Baizhi choice policy was silently decided");
if (manifest.policy_open.legacy_entrypoint.status !== "unresolved" || manifest.policy_open.legacy_entrypoint.do_not_assume !== true) err("E_LEGACY_POLICY_ASSUMED", "legacy entrypoint policy was silently decided");
if (manifest.policy_open.asset_provenance.publish_eligible !== false) err("E_ASSET_POLICY_ASSUMED", "asset provenance policy was silently upgraded");

const result = {
  schema: "continuous_tp01_conditional_gate_review_ag19_result_v1",
  status: errors.length ? "invalid_decision_review" : "passed_decision_review",
  errors,
  overall: manifest.decision.overall,
  architecture_gate_passed: manifest.decision.architecture_gate_passed,
  tp02_specific: manifest.decision.tp02_specific,
  publish_eligible: manifest.decision.publish_eligible,
  evidence: {
    ag15_overall: ag15.gateDecision,
    ag16: { api_compiles: ag16.api_compiles, native_channel_exercised: ag16.native_channel_exercised, webview_roundtrip_exercised: ag16.webview_roundtrip_exercised, gate_d: ag16.gate_d },
    ag17: { status: ag17.status, preserved: true },
    ag18: { status: ag18.status, errors: ag18.errors.length, formal_blockout: false, gate_i: "pending" },
    ag11: { status: ag11.status, production_save_v3: false, baizhi_choice_unresolved: true },
    ag13: { publish_eligible: ag13.publishDecision?.eligible === true },
    ag14: { status: ag14.status, active_legacy_split_implemented: false }
  },
  adr_statuses: Object.fromEntries(manifest.adr_review.map((adr) => [adr.id, adr.status])),
  tp02_package: {
    status: pkg.status,
    allowed_writes: pkg.allowed_writes,
    forbidden_count: pkg.forbidden.length,
    required_tests: pkg.required_tests,
    downstream_hard_gates: pkg.downstream_hard_gates,
    rollback: pkg.rollback
  },
  policy_open: manifest.policy_open,
  formal_claims: manifest.formal_claims,
  validator_note: "errors=[] proves only that this decision review is internally consistent; it does not pass the Architecture Gate or run the proposed TP-02 tests."
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(`AG-19 gate review: ${result.status}; errors=${errors.length}; overall=${result.overall}; tp02=${result.tp02_specific}; publish=${result.publish_eligible}`);
if (errors.length) process.exitCode = 1;
