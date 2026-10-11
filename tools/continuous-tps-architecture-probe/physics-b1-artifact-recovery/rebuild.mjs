import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const recoveryDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(recoveryDir, "..", "..", "..");
const resultPath = path.resolve(
  repoRoot,
  "tools",
  "continuous-tps-architecture-probe",
  "results",
  "physics-b1-artifact-recovery.json",
);

const sourceFiles = [
  "server-rs/src/continuous_kcc.rs",
  "server-rs/tests/continuous_kcc_test.rs",
  "server-rs/src/grey_hive_control_collider.rs",
  "server-rs/tests/grey_hive_control_collider_test.rs",
  "server-rs/Cargo.toml",
  "server-rs/Cargo.lock",
];

function run(command, args) {
  const child = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true,
  });
  const output = `${child.stdout ?? ""}${child.stderr ?? ""}`;
  const match = output.match(/(\d+) passed; (\d+) failed/);
  return {
    command: [command, ...args].join(" "),
    exitCode: child.status ?? 1,
    passed: match ? Number(match[1]) : null,
    failed: match ? Number(match[2]) : null,
    outputTail: output.slice(-1200),
  };
}

function sha256(relativePath) {
  const absolutePath = path.resolve(repoRoot, relativePath);
  return crypto.createHash("sha256").update(fs.readFileSync(absolutePath)).digest("hex");
}

function git(args) {
  const child = spawnSync("git", args, { cwd: repoRoot, encoding: "utf8", windowsHide: true });
  return (child.stdout ?? "").trim();
}

const ag40Path = path.resolve(
  repoRoot,
  "tools",
  "continuous-tps-architecture-probe",
  "results",
  "physics-evidence-reconciliation-ag40.json",
);
const ag40 = JSON.parse(fs.readFileSync(ag40Path, "utf8"));
const referenceLengthM = ag40.lengthReconciliation.tp04b1ReferenceLengthM;
const nominalLengthM = ag40.lengthReconciliation.v7NominalLengthM;
const absoluteDeltaM = Math.abs(nominalLengthM - referenceLengthM);
if (referenceLengthM !== 39.998558044433594 || nominalLengthM !== 40) {
  throw new Error("read-only length references changed unexpectedly");
}

const kccTest = run("cargo", ["test", "--manifest-path", "server-rs/Cargo.toml", "--locked", "--test", "continuous_kcc_test"]);
const colliderTest = run("cargo", ["test", "--manifest-path", "server-rs/Cargo.toml", "--locked", "--test", "grey_hive_control_collider_test"]);
const formalReplayPass = kccTest.exitCode === 0 && kccTest.passed === 14 && kccTest.failed === 0 && colliderTest.exitCode === 0 && colliderTest.passed === 5 && colliderTest.failed === 0;

const report = {
  schema: "continuous_physics_b1_artifact_recovery_ag42_v1",
  status: formalReplayPass ? "recovered_formal_replay_with_preserved_b1_reference" : "failed_formal_replay",
  machinePass: formalReplayPass,
  recoveryMode: "formal_test_replay_plus_preserved_readonly_reference_scalar",
  artifactClaim: "This is a new reproducible machine record for TP-04A/B1 formal tests and the preserved TP-04B1 scalar; it does not claim to recover an unavailable original B1 trajectory log.",
  provenance: {
    repoHead: git(["rev-parse", "HEAD"]),
    dirtyStatus: git(["status", "--porcelain", "--", ...sourceFiles]),
    generatedAtUtc: new Date().toISOString(),
    rustc: run("rustc", ["--version"]).outputTail.trim(),
    cargo: run("cargo", ["--version"]).outputTail.trim(),
    rapier: "rapier3d 0.22.0 (server-rs/Cargo.lock)",
    sourceFiles: Object.fromEntries(sourceFiles.map((file) => [file, { sha256: sha256(file) }])),
  },
  reproducedTests: {
    tp04a_continuous_kcc: {
      command: kccTest.command,
      exitCode: kccTest.exitCode,
      testsPassed: kccTest.passed,
      testsFailed: kccTest.failed,
      sourcePaths: ["server-rs/src/continuous_kcc.rs", "server-rs/tests/continuous_kcc_test.rs"],
    },
    tp04b1_grey_hive_control_collider: {
      command: colliderTest.command,
      exitCode: colliderTest.exitCode,
      testsPassed: colliderTest.passed,
      testsFailed: colliderTest.failed,
      sourcePaths: ["server-rs/src/grey_hive_control_collider.rs", "server-rs/tests/grey_hive_control_collider_test.rs"],
    },
  },
  tp04b1Reference: {
    connectorId: "edge-b01-b02-core-v7",
    from: "GH-B01",
    to: "GH-B02",
    nominalLengthM,
    referenceLengthM,
    absoluteDeltaM,
    absoluteDeltaMm: absoluteDeltaM * 1000,
    sourcePath: "tools/continuous-tps-architecture-probe/results/physics-evidence-reconciliation-ag40.json",
    sourcePointer: "/lengthReconciliation/tp04b1ReferenceLengthM",
    status: "preserved_readonly_scalar_reference",
    comparisonClassification: "blocked_reference_mismatch",
    originalValuesOverwritten: false,
  },
  scopeGuard: {
    trueGateBConnector: "edge-b05-c01",
    tp04b1Connector: "edge-b01-b02-core-v7",
    gateBPromotion: "forbidden",
    note: "TP-04B1 replay is the B01↔B02 formal static slice; it is not the true Gate B B05↔C01 connector.",
  },
  formalBlockers: [
    "original TP-04B1 standalone trajectory JSON is unavailable; only the preserved scalar reference is carried forward",
    "TP-04A/B1 replay does not prove final Blockout, nav, camera, resolver, save/replay, or runtime integration",
    "true Gate B edge-b05-c01 requires its own source-derived physics candidate",
  ],
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ status: report.status, machinePass: report.machinePass, resultPath, tp04a: kccTest, tp04b1: colliderTest }, null, 2));
if (!report.machinePass) process.exitCode = 1;
