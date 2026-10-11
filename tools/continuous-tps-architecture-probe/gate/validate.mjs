import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const MANIFEST_PATH = path.join(HERE, "decision-v1.json");
const RESULT_PATH = path.join(HERE, "../results/gate-ag15.json");
const VALID_STATUSES = new Set(["candidate", "conditionally_frozen", "blocked"]);

function clone(value) { return JSON.parse(JSON.stringify(value)); }
function err(code, detail, id = null) { return { code, detail, id }; }

function resolveEvidenceRef(value) {
  const raw = String(value).trim();
  const match = raw.match(/^([^:]+):\d/);
  return path.join(ROOT, match ? match[1] : raw);
}

function checkPath(value, errors, owner) {
  if (!value) {
    errors.push(err("E_MISSING_EVIDENCE_REF", "empty evidence reference", owner));
    return;
  }
  for (const piece of String(value).split(";")) {
    const ref = piece.trim();
    if (!ref) continue;
    const target = resolveEvidenceRef(ref);
    if (!fs.existsSync(target)) errors.push(err("E_MISSING_EVIDENCE_FILE", `${ref} does not exist`, owner));
  }
}

function validateGate(manifest) {
  const errors = [];
  if (manifest.schema !== "continuous_architecture_gate_ag15_v1") errors.push(err("E_SCHEMA", "unexpected gate schema"));
  if (manifest.version !== 1) errors.push(err("E_VERSION", "gate version must be 1"));
  const evidenceIds = new Set((manifest.evidenceIndex ?? []).map((item) => item.id));
  if (evidenceIds.size !== (manifest.evidenceIndex ?? []).length) errors.push(err("E_DUPLICATE_EVIDENCE_ID", "evidence index IDs must be unique"));
  for (const item of manifest.evidenceIndex ?? []) {
    checkPath(item.report, errors, item.id);
    checkPath(item.machine, errors, item.id);
    if (!Array.isArray(item.commands) || item.commands.length === 0) errors.push(err("E_MISSING_COMMAND_RECORD", "evidence item needs at least one command", item.id));
  }
  const adrs = manifest.adrs ?? [];
  const adrIds = new Set();
  for (const adr of adrs) {
    if (adrIds.has(adr.id)) errors.push(err("E_DUPLICATE_ADR", `duplicate ${adr.id}`, adr.id));
    adrIds.add(adr.id);
    if (!VALID_STATUSES.has(adr.status)) errors.push(err("E_BAD_ADR_STATUS", `unsupported status ${adr.status}`, adr.id));
    if (!adr.candidateDecision || !adr.owner || !adr.acceptance || !adr.remainingEvidence?.length) errors.push(err("E_INCOMPLETE_ADR", "ADR needs candidate decision, owner, acceptance and remaining evidence", adr.id));
    if (!adr.evidence?.length) errors.push(err("E_ADR_NO_EVIDENCE", "ADR needs evidence index references", adr.id));
    for (const ref of adr.evidence ?? []) {
      if (evidenceIds.has(ref)) continue;
      if (/^(docs|tools|server-rs)\//.test(ref)) checkPath(ref, errors, adr.id);
      else errors.push(err("E_UNKNOWN_EVIDENCE_ID", `${ref} is not in evidence index`, adr.id));
    }
    if (adr.status === "blocked" && (!adr.remainingEvidence.length || !adr.owner)) errors.push(err("E_BLOCKED_NO_EXIT", "blocked ADR must state exit evidence and owner", adr.id));
  }
  for (const id of ["ADR-A", "ADR-B", "ADR-C", "ADR-D", "ADR-E", "ADR-F", "ADR-G", "ADR-H", "ADR-I", "ADR-J"]) if (!adrIds.has(id)) errors.push(err("E_MISSING_ADR", `${id} is absent`, id));

  const blocked = adrs.filter((adr) => adr.status === "blocked").map((adr) => adr.id);
  if (manifest.decision.overall === "go" && blocked.length) errors.push(err("E_GO_BLOCK_CONFLICT", `go conflicts with blocked ${blocked.join(", ")}`));
  if (manifest.decision.overall === "go" && manifest.decision.publishEligibleRequired && manifest.decision.assetPublishEligible === false) errors.push(err("E_PUBLISH_CONFLICT", "go cannot claim release while asset publication is ineligible"));
  if (manifest.decision.overall !== "no_go" && blocked.length) errors.push(err("E_BLOCKING_DECISION", "blocked ADRs require no_go"));
  if (!manifest.decision.minimumNextPackage || !manifest.decision.minimumNextPackageExit?.length) errors.push(err("E_NO_NEXT_PACKAGE", "no_go needs a minimum next package and exit criteria"));

  const matrix = manifest.tpMatrix ?? [];
  const packages = new Set(matrix.map((item) => item.package));
  for (const id of ["TP-02", "TP-03", "TP-04", "TP-05", "TP-06", "TP-07", "TP-08", "TP-09"]) {
    const item = matrix.find((row) => row.package === id);
    if (!item) errors.push(err("E_MISSING_TP_MATRIX", `${id} is absent`, id));
    else if (!item.writes?.length || !item.dependsOn || !item.parallelStatus) errors.push(err("E_INCOMPLETE_TP_MATRIX", `${id} needs writes, dependencies and parallel status`, id));
  }
  if (packages.size !== matrix.length) errors.push(err("E_DUPLICATE_TP_MATRIX", "TP matrix package IDs must be unique"));
  if (manifest.futureWork?.["AG-16"] !== "pending; not started; result must be marked pending" || manifest.futureWork?.["AG-17"] !== "pending; not started; result must be marked pending") errors.push(err("E_PENDING_CONFLICT", "AG-16/17 must remain pending and unstarted"));
  if ((manifest.policyDecisionsPending ?? []).some((item) => item.status === "unresolved" && item.doNotAssume !== true)) errors.push(err("E_UNRESOLVED_POLICY_ASSUMED", "unresolved user policy may not be assumed"));
  return { errors, blocked, adrCount: adrs.length, tpCount: matrix.length };
}

function scenario(name, mutate, expectedCode) {
  const manifest = clone(JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8")));
  mutate(manifest);
  const result = validateGate(manifest);
  return { name, pass: result.errors.some((item) => item.code === expectedCode), expectedCode, errorCodes: [...new Set(result.errors.map((item) => item.code))], errorCount: result.errors.length };
}

const manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8"));
const validation = validateGate(manifest);
const scenarios = [
  { name: "baseline_evidence_index", pass: validation.errors.length === 0, expectedCode: null, errorCodes: [...new Set(validation.errors.map((item) => item.code))], errorCount: validation.errors.length },
  scenario("missing_evidence_exit", (value) => { value.adrs[0].remainingEvidence = []; }, "E_INCOMPLETE_ADR"),
  scenario("go_conflicts_with_blocked", (value) => { value.decision.overall = "go"; }, "E_GO_BLOCK_CONFLICT"),
  scenario("publish_conflicts_with_false_eligibility", (value) => { value.adrs.forEach((adr) => { if (adr.status === "blocked") adr.status = "conditionally_frozen"; }); value.decision.overall = "go"; value.decision.assetPublishEligible = false; }, "E_PUBLISH_CONFLICT"),
  scenario("pending_work_assumed_complete", (value) => { value.futureWork["AG-16"] = "complete"; }, "E_PENDING_CONFLICT"),
  scenario("missing_tp09_matrix", (value) => { value.tpMatrix = value.tpMatrix.filter((item) => item.package !== "TP-09"); }, "E_MISSING_TP_MATRIX"),
  scenario("unresolved_policy_assumed", (value) => { value.policyDecisionsPending[0].doNotAssume = false; }, "E_UNRESOLVED_POLICY_ASSUMED")
];

const assetResult = JSON.parse(fs.readFileSync(path.join(ROOT, "tools/continuous-tps-architecture-probe/results/assets-ag13.json"), "utf8"));
const moduleResult = JSON.parse(fs.readFileSync(path.join(ROOT, "tools/continuous-tps-architecture-probe/results/modules-ag14.json"), "utf8"));
const report = {
  schema: "continuous_architecture_gate_probe_ag15_v1",
  status: validation.errors.length === 0 && scenarios.every((item) => item.pass) ? "architecture_gate_no_go_with_validated_evidence_summary" : "failed",
  pass: validation.errors.length === 0 && scenarios.every((item) => item.pass),
  gateDecision: manifest.decision.overall,
  errors: validation.errors,
  evidenceIndexValid: validation.errors.length === 0,
  adrCount: validation.adrCount,
  blockedAdrs: validation.blocked,
  conditionallyFrozenAdrs: manifest.adrs.filter((adr) => adr.status === "conditionally_frozen").map((adr) => adr.id),
  candidateAdrs: manifest.adrs.filter((adr) => adr.status === "candidate").map((adr) => adr.id),
  scenarioCount: scenarios.length,
  scenarios,
  publishDecision: { eligible: assetResult.publishDecision?.eligible === true, observed: assetResult.publishDecision, conflictChecked: true },
  activeLegacySplit: { observed: moduleResult.observed, formalSplitImplemented: false, conflictChecked: true },
  pending: { AG16: "pending", AG17: "pending", agentsStarted: false },
  noGoNextPackage: manifest.decision.minimumNextPackage,
  noGoExitCriteria: manifest.decision.minimumNextPackageExit,
  tpMatrixCount: validation.tpCount,
  note: "pass=true means the evidence summary and contradiction checks are valid; it does not mean the Architecture Gate passed."
};

fs.mkdirSync(path.dirname(RESULT_PATH), { recursive: true });
fs.writeFileSync(RESULT_PATH, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
if (!report.pass) process.exitCode = 1;
