import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const dir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dir, "..", "..", "..");
const resultPath = path.resolve(repoRoot, "tools", "continuous-tps-architecture-probe", "results", "physics-b1-artifact-recovery.json");
const report = JSON.parse(fs.readFileSync(resultPath, "utf8"));
const failures = [];

if (report.schema !== "continuous_physics_b1_artifact_recovery_ag42_v1") failures.push("schema");
if (report.machinePass !== true || report.status !== "recovered_formal_replay_with_preserved_b1_reference") failures.push("machine status");
if (report.reproducedTests?.tp04a_continuous_kcc?.exitCode !== 0 || report.reproducedTests.tp04a_continuous_kcc.testsPassed !== 14 || report.reproducedTests.tp04a_continuous_kcc.testsFailed !== 0) failures.push("TP-04A test replay");
if (report.reproducedTests?.tp04b1_grey_hive_control_collider?.exitCode !== 0 || report.reproducedTests.tp04b1_grey_hive_control_collider.testsPassed !== 5 || report.reproducedTests.tp04b1_grey_hive_control_collider.testsFailed !== 0) failures.push("TP-04B1 test replay");
if (report.tp04b1Reference?.connectorId !== "edge-b01-b02-core-v7") failures.push("TP-04B1 connector scope");
if (report.scopeGuard?.trueGateBConnector !== "edge-b05-c01") failures.push("true Gate B scope");
if (report.tp04b1Reference?.nominalLengthM !== 40 || report.tp04b1Reference?.referenceLengthM !== 39.998558044433594) failures.push("reference length");
if (Math.abs(report.tp04b1Reference?.absoluteDeltaM - 0.00144195556640625) > 1e-12) failures.push("reference delta");
if (report.tp04b1Reference?.originalValuesOverwritten !== false) failures.push("historical value overwrite");

for (const [relativePath, record] of Object.entries(report.provenance?.sourceFiles ?? {})) {
  const absolutePath = path.resolve(repoRoot, relativePath);
  const hash = crypto.createHash("sha256").update(fs.readFileSync(absolutePath)).digest("hex");
  if (hash !== record.sha256) failures.push(`source hash ${relativePath}`);
}

const output = {
  schema: "continuous_physics_b1_artifact_recovery_validator_ag42_v1",
  allChecksPassed: failures.length === 0,
  checks: {
    provenance: failures.every((item) => !item.startsWith("source hash")),
    formalTestReplay: failures.every((item) => !item.includes("test replay")),
    referenceAndDelta: failures.every((item) => !item.includes("reference")),
    scopeGuard: failures.every((item) => !item.includes("scope")),
    noOverwrite: failures.every((item) => !item.includes("overwrite")),
  },
  failures,
};
console.log(JSON.stringify(output, null, 2));
if (failures.length > 0) process.exitCode = 1;
