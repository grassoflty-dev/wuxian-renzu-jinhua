import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const resultPath = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "results",
  "physics-evidence-reconciliation-ag40.json",
);

const report = JSON.parse(fs.readFileSync(resultPath, "utf8"));
const requiredClaimFields = [
  "claimId",
  "claim",
  "source",
  "evidenceLevel",
  "status",
  "conflictOrMissingReason",
  "allowedUse",
];
const allowedLevels = new Set([
  "formal source+test",
  "isolated code+machine",
  "report-only",
  "missing",
]);
const failures = [];

if (report.schema !== "continuous_physics_evidence_reconciliation_ag40_v1") {
  failures.push("unexpected schema");
}
if (report.status !== "blocked_no_go") failures.push("status must remain blocked_no_go");
if (report.scope?.gateC !== "blocked") failures.push("Gate C must remain blocked");
if (report.scope?.gateI !== "blocked") failures.push("Gate I must remain blocked");
if (report.scope?.architectureGate !== "blocked_no_go") {
  failures.push("Architecture Gate must remain blocked_no_go");
}

const claims = report.evidenceLedger;
if (!Array.isArray(claims) || claims.length < 10) {
  failures.push("evidence ledger is missing or too short");
} else {
  const ids = new Set();
  for (const claim of claims) {
    for (const field of requiredClaimFields) {
      if (!(field in claim)) failures.push(`${claim.claimId ?? "unknown"} missing ${field}`);
    }
    if (ids.has(claim.claimId)) failures.push(`duplicate claimId ${claim.claimId}`);
    ids.add(claim.claimId);
    if (!allowedLevels.has(claim.evidenceLevel)) {
      failures.push(`${claim.claimId} has invalid evidence level`);
    }
    if (!Array.isArray(claim.source) || claim.source.length === 0) {
      failures.push(`${claim.claimId} has no source pointer`);
    }
    if (claim.source?.some((pointer) => !pointer.path || !pointer.pointer)) {
      failures.push(`${claim.claimId} has an incomplete source pointer`);
    }
  }
}

const tests = report.reproducedTests ?? {};
if (tests.continuous_kcc_test?.exitCode !== 0 || tests.continuous_kcc_test?.testsPassed !== 14) {
  failures.push("continuous_kcc_test reproduction mismatch");
}
if (tests.grey_hive_control_collider_test?.exitCode !== 0 || tests.grey_hive_control_collider_test?.testsPassed !== 5) {
  failures.push("grey_hive_control_collider_test reproduction mismatch");
}

const mismatch = report.scopeReconciliation?.tp04b2ScopeMismatch;
if (mismatch?.tp04b2Connector !== "edge-b01-b02-core-v7") failures.push("TP-04B2 connector mismatch input changed");
if (mismatch?.v7GateBConnector !== "edge-b05-c01") failures.push("v7 Gate B connector mismatch input changed");
if (mismatch?.classification !== "scope_mismatch_blocked") failures.push("TP-04B2 mismatch classification changed");

const length = report.lengthReconciliation;
if (length?.v7NominalLengthM !== 40 || Math.abs(length?.tp04b1ReferenceLengthM - 39.998558044433594) > 1e-9) {
  failures.push("length reconciliation values changed");
}
if (Math.abs(length?.absoluteDeltaM - 0.00144195556640625) > 1e-9) {
  failures.push("length reconciliation delta changed");
}
if (length?.classification !== "blocked_reference_mismatch") {
  failures.push("length reconciliation must remain blocked_reference_mismatch");
}

if (report.artifactGaps?.tp04aStandaloneJson !== "artifact_gap") failures.push("TP-04A artifact gap missing");
if (report.artifactGaps?.tp04b1StandaloneJson !== "artifact_gap") failures.push("TP-04B1 artifact gap missing");
if (report.nextEvidencePackage?.mutuallyExclusiveWriteSet !== true) {
  failures.push("next evidence package is not marked mutually exclusive");
}

const output = {
  schema: "continuous_physics_evidence_reconciliation_ag40_validator_v1",
  allChecksPassed: failures.length === 0,
  claimCount: Array.isArray(claims) ? claims.length : 0,
  checks: {
    ledgerFields: failures.filter((failure) => failure.includes("missing") || failure.includes("source pointer") || failure.includes("claimId")).length === 0,
    formalTestCounts: failures.every((failure) => !failure.includes("reproduction mismatch")),
    tp04b2Scope: failures.every((failure) => !failure.includes("TP-04B2") && !failure.includes("v7 Gate B")),
    lengthReconciliation: failures.every((failure) => !failure.includes("length reconciliation")),
    artifactGaps: failures.every((failure) => !failure.includes("artifact gap")),
    nextWriteSet: failures.every((failure) => !failure.includes("mutually exclusive")),
  },
  failures,
};
console.log(JSON.stringify(output, null, 2));
if (failures.length > 0) process.exitCode = 1;
