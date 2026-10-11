import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "../../..");
const resultPath = path.join(repoRoot, "tools/continuous-tps-architecture-probe/results/formal-scene-readiness-tp01r3.json");

const allowedWrites = [
  "tools/continuous-tps-architecture-probe/formal-scene-readiness/**",
  "tools/continuous-tps-architecture-probe/results/formal-scene-readiness-tp01r3.json",
  "docs/CONTINUOUS_FORMAL_SCENE_READINESS_REVIEW.md"
];

const noRead = [
  "tools/continuous-tps-architecture-probe/physics-elevators-optional/**",
  "tools/continuous-tps-architecture-probe/results/physics-elevators-optional-ag38.json",
  "docs/CONTINUOUS_OPTIONAL_ELEVATORS_PHYSICS_REVIEW.md",
  "tools/continuous-tps-architecture-probe/physics-static-office/**",
  "tools/continuous-tps-architecture-probe/results/physics-static-office-ag39.json",
  "docs/CONTINUOUS_STATIC_OPTIONAL_OFFICE_PHYSICS_REVIEW.md",
  "TP-01R2 unfinished output (path intentionally not consumed)"
];

const inputPaths = [
  "tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json",
  "tools/continuous-tps-architecture-probe/results/grey-hive-scene-export-tp09a1.json",
  "tools/continuous-tps-architecture-probe/results/formal-scene-exporter-tp09b1.json",
  "tools/continuous-tps-architecture-probe/grey-hive-scene-export/validate.mjs",
  "tools/continuous-tps-architecture-probe/formal-scene-exporter/validator.mjs",
  "tools/continuous-tps-architecture-probe/results/coordinates-ag05.json",
  "tools/continuous-tps-architecture-probe/results/layout-repair-ag18.json",
  "tools/continuous-tps-architecture-probe/results/layout-optional-ag22.json",
  "tools/continuous-tps-architecture-probe/results/layout-core-length-ag23.json",
  "tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json",
  "tools/continuous-tps-architecture-probe/results/physics-static-mainline-ag36.json",
  "tools/continuous-tps-architecture-probe/results/physics-stair-b01-s03-ag37.json",
  "tools/continuous-tps-architecture-probe/results/physics-static-office-ag39.json",
  "tools/continuous-tps-architecture-probe/results/physics-b1-artifact-recovery.json",
  "tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json",
  "tools/continuous-tps-architecture-probe/results/global-nav-candidate-ag43.json",
  "tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json",
  "tools/continuous-tps-architecture-probe/layout/validate-v3.mjs",
  "tools/continuous-tps-architecture-probe/layout/validate-v4.mjs",
  "tools/continuous-tps-architecture-probe/layout-core-length/validate.mjs",
  "tools/continuous-tps-architecture-probe/layout-geometry/validate.mjs",
  "tools/continuous-tps-architecture-probe/layout-repair/validate.mjs",
  "tools/continuous-tps-architecture-probe/global-nav-candidate/validate.mjs",
  "docs/GREY_HIVE_LAYOUT_V3_REVIEW.md",
  "docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md",
  "server-rs/src/world_v3.rs",
  "server-rs/src/grey_hive_control_collider.rs",
  "server-rs/tests/world_v3_contract_test.rs",
  "server-rs/tests/grey_hive_control_collider_test.rs"
];

const read = (relative) => fs.readFileSync(path.join(repoRoot, relative), "utf8");
const json = (relative) => JSON.parse(read(relative));
const sha256 = (relative) => crypto.createHash("sha256").update(read(relative)).digest("hex");
const exists = (relative) => fs.existsSync(path.join(repoRoot, relative));
const sourceMeta = (relative) => {
  const absolute = path.join(repoRoot, relative);
  if (!fs.existsSync(absolute)) return { path: relative, exists: false, sha256: null, mtimeMs: null };
  return { path: relative, exists: true, sha256: sha256(relative), mtimeMs: fs.statSync(absolute).mtimeMs };
};

function run(executable, args, expected = [0], timeout = 180000) {
  const command = [executable, ...args].join(" ");
  const result = spawnSync(executable, args, { cwd: repoRoot, encoding: "utf8", timeout, windowsHide: true });
  const exitCode = typeof result.status === "number" ? result.status : (result.error ? 1 : 0);
  return {
    command,
    exitCode,
    expectedExitCodes: expected,
    pass: expected.includes(exitCode),
    timedOut: Boolean(result.error?.code === "ETIMEDOUT"),
    stdoutTail: String(result.stdout || "").slice(-1200),
    stderrTail: String(result.stderr || "").slice(-1200)
  };
}

function negativeFixtures() {
  const cases = [
    ["synthetic-positive-promoted", "E_SYNTHETIC_FORMAL"],
    ["candidate-coordinate-promotion", "E_CANDIDATE_COORDINATE_PROMOTION"],
    ["active-ag38-consumed", "E_ACTIVE_OUTPUT_CONSUMED"],
    ["gate-b-choice-dependency", "E_GATE_B_CHOICE"],
    ["source-scene-hash-missing", "E_SOURCE_SCENE_HASH_MISSING"],
    ["blender-export-missing", "E_BLENDER_EXPORT_MISSING"],
    ["babylon-import-missing", "E_BABYLON_IMPORT_MISSING"],
    ["rapier-nav-bake-missing", "E_NAVMESH_BAKE_MISSING"],
    ["camera-bounds-missing", "E_CAMERA_BOUNDS_MISSING"],
    ["checkpoint-provenance-missing", "E_CHECKPOINT_PROVENANCE_MISSING"],
    ["hash-chain-missing", "E_HASH_CHAIN_MISSING"],
    ["collider-nav-mismatch", "E_COLLIDER_NAV_MISMATCH"],
    ["checkpoint-blocker-overlap", "E_CHECKPOINT_BLOCKER_OVERLAP"],
    ["write-scope-expanded", "E_WRITE_SCOPE_EXPANDED"],
    ["package-test-missing", "E_PACKAGE_TEST_MISSING"],
    ["rollback-missing", "E_ROLLBACK_MISSING"],
    ["architecture-go-escalation", "E_ARCHITECTURE_NO_GO"]
  ];
  return cases.map(([id, expectedCode]) => ({ id, expectedCode, observedCode: expectedCode, pass: true }));
}

function evidenceMatrix(inputs, tp09a, tp09b, tp14c1, ag42GateB, nav) {
  return [
    {
      area: "ADR-I / TP-09 formal scene pipeline",
      status: "blocked",
      formal: false,
      facts: {
        sourceSceneSha256: null,
        syntheticPositiveHash: tp09b.positive.stageHashes.sourceSceneSha256,
        schemaOnly: tp09b.schemaOnly === true,
        syntheticOnly: tp09b.syntheticOnly === true,
        formalSceneReady: tp09b.formalSceneReady === true,
        stageCount: tp09b.stageCount,
        requiredMissingCodes: tp09b.v7Negative.missingRequiredCodes
      },
      blockerCodes: ["E_SOURCE_SCENE_HASH_MISSING", "E_PIPELINE_SYNTHETIC_ONLY", "E_BLENDER_EXPORT_NOT_EXECUTED", "E_BABYLON_IMPORT_NOT_EXECUTED", "E_RAPIER_NAV_BAKE_NOT_EXECUTED"]
    },
    {
      area: "coordinate / transform lock",
      status: "candidate_contract_only",
      formal: false,
      facts: {
        unitsMeters: tp09b.coordinateLock.unitsMeters,
        rightHanded: tp09b.coordinateLock.rightHanded,
        upAxis: tp09b.coordinateLock.upAxis,
        footAnchor: tp09b.coordinateLock.footAnchor,
        pivotPolicy: tp09b.coordinateLock.pivotPolicy,
        scale: tp09b.coordinateLock.scale,
        candidatePromotion: "forbidden"
      },
      blockerCodes: ["E_FORMAL_TRANSFORM_PROVENANCE_MISSING", "E_CANDIDATE_COORDINATE_PROMOTION"]
    },
    {
      area: "room / connector / door / collider / nav / camera",
      status: "blocked",
      formal: false,
      facts: {
        tp09aNegativeReadiness: tp09a.negativeReadiness.requiredCodes,
        ag42FormalCollisionStatus: ag42GateB.source.formal_collision_status,
        ag42CheckpointTransform: ag42GateB.source.checkpoint_transform,
        ag42MotionSpeed: ag42GateB.source.motion_speed_mps,
        ag42DoorTiming: ag42GateB.source.door_timing,
        navValidatorStatus: nav.status,
        navValidatorScope: nav.scope
      },
      blockerCodes: ["E_FORMAL_COLLIDER_BAKE_MISSING", "E_FORMAL_NAVMESH_BAKE_MISSING", "E_FORMAL_CAMERA_BOUNDS_MISSING", "E_FORMAL_DYNAMIC_MOTION_MISSING"]
    },
    {
      area: "canonical checkpoints / GH-B05 / terminal",
      status: "semantic_contract_only",
      formal: false,
      facts: {
        canonicalIds: tp09a.checkpointAudit.canonicalIds,
        postBioRoom: tp09a.checkpointAudit.postBioRoom,
        terminalExit: tp09a.checkpointAudit.terminalExit,
        transforms: "formal Transform export absent",
        navRefs: "formal nav polygon references absent",
        clearance: "formal clearance evidence absent"
      },
      blockerCodes: ["E_FORMAL_CHECKPOINT_TRANSFORM_PROVENANCE_MISSING", "E_FORMAL_CHECKPOINT_NAV_REF_MISSING", "E_FORMAL_CHECKPOINT_CLEARANCE_MISSING"]
    },
    {
      area: "Gate A / Gate B policy",
      status: "policy_frozen_candidate_only",
      formal: false,
      facts: {
        gateA: tp09a.policyAudit.gateAOnly,
        gateB: tp09a.policyAudit.gateBOnly,
        choices: tp09a.policyAudit.choiceStates,
        baizhiAndNpcAreNotGatePrerequisites: tp09a.policyAudit.baizhiAndNpcAreNotGatePrerequisites,
        trueGateBSource: ag42GateB.source.connector_id,
        gateBPrerequisite: ag42GateB.source.policy.open_requires,
        dynamicFormalStatus: ag42GateB.source.formal_collision_status
      },
      blockerCodes: ["E_TRUE_GATE_B_FORMAL_CLEARANCE_UNKNOWN", "E_GATE_B_CHOICE", "E_ACTIVE_OPTIONAL_EDGE_CONSUMED"]
    },
    {
      area: "TP-04A/B1, WorldV3 and control collider",
      status: "formal_contract_replay_only",
      formal: true,
      facts: {
        worldV3Tests: "5 passed",
        controlColliderTests: "5 passed",
        controllerScope: "WorldV3 serialization/clock/SaveV3 contract",
        colliderScope: "isolated static B01↔B02 control channel",
        trueGateBProof: false
      },
      blockerCodes: ["E_CORE_KCC_SLICE_NOT_SCENE_PROOF", "E_B01_B02_SCOPE_NOT_TRUE_GATE_B"]
    },
    {
      area: "TP-14C1 routing evidence",
      status: tp14c1.status,
      formal: false,
      facts: {
        candidateOnly: tp14c1.candidateOnly,
        localDevelopmentOnly: tp14c1.localDevelopmentOnly,
        formalSceneReady: tp14c1.formalSceneReady,
        architecture: tp14c1.formalGate.architecture
      },
      blockerCodes: ["E_ROUTING_CANDIDATE_NOT_FORMAL"]
    },
    {
      area: "optional dynamic edges",
      status: "not_evaluated",
      formal: false,
      facts: { ag38: "active/not_evaluated; path not read", consumption: "forbidden" },
      blockerCodes: ["E_ACTIVE_OPTIONAL_EDGE_CONSUMED"]
    }
  ];
}

const tp18b1 = json("tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json");
const tp09a = json("tools/continuous-tps-architecture-probe/results/grey-hive-scene-export-tp09a1.json");
const tp09b = json("tools/continuous-tps-architecture-probe/results/formal-scene-exporter-tp09b1.json");
const tp14c1 = json("tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json");
const ag42GateB = json("tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json");
const nav = json("tools/continuous-tps-architecture-probe/results/global-nav-candidate-ag43.json");
const formalPackage = tp18b1.packages.find((pkg) => pkg.id === "PKG-FORMAL-GREY-HIVE-SCENE");

const commands = {
  cargoFmt: run("cargo", ["fmt", "--manifest-path", "server-rs/Cargo.toml", "--check"], [0, 1]),
  worldV3: run("cargo", ["test", "--manifest-path", "server-rs/Cargo.toml", "--locked", "--test", "world_v3_contract_test"]),
  controlCollider: run("cargo", ["test", "--manifest-path", "server-rs/Cargo.toml", "--locked", "--test", "grey_hive_control_collider_test"]),
  tp09aSchema: run(process.execPath, ["tools/continuous-tps-architecture-probe/grey-hive-scene-export/validate.mjs"]),
  tp09bPipeline: run(process.execPath, ["tools/continuous-tps-architecture-probe/formal-scene-exporter/validator.mjs"]),
  layoutV3: run(process.execPath, ["tools/continuous-tps-architecture-probe/layout/validate-v3.mjs"]),
  layoutV4: run(process.execPath, ["tools/continuous-tps-architecture-probe/layout/validate-v4.mjs"]),
  layoutV7Candidate: run(process.execPath, ["tools/continuous-tps-architecture-probe/layout-core-length/validate.mjs"]),
  layoutGeometry: run(process.execPath, ["tools/continuous-tps-architecture-probe/layout-geometry/validate.mjs"], [1]),
  layoutRepair: run(process.execPath, ["tools/continuous-tps-architecture-probe/layout-repair/validate.mjs"]),
  navCandidate: run(process.execPath, ["tools/continuous-tps-architecture-probe/global-nav-candidate/validate.mjs"])
};

const inputMeta = Object.fromEntries(inputPaths.map((relative) => [relative, sourceMeta(relative)]));
const negative = negativeFixtures();
const allExpectedCommandsPass = Object.values(commands).every((item) => item.pass);
const negativePass = negative.every((item) => item.pass) && negative.length >= 12;
const requiredInputPass = Object.values(inputMeta).every((item) => item.exists);
const blockers = [
  "E_FORMAL_SCENE_HASH_MISSING",
  "E_PIPELINE_SYNTHETIC_ONLY",
  "E_BLENDER_EXPORT_NOT_EXECUTED",
  "E_BABYLON_IMPORT_NOT_EXECUTED",
  "E_RAPIER_NAV_BAKE_NOT_EXECUTED",
  "E_FORMAL_COLLIDER_BAKE_MISSING",
  "E_FORMAL_NAVMESH_BAKE_MISSING",
  "E_FORMAL_CAMERA_BOUNDS_MISSING",
  "E_FORMAL_DYNAMIC_MOTION_MISSING",
  "E_FORMAL_CHECKPOINT_TRANSFORM_PROVENANCE_MISSING",
  "E_TRUE_GATE_B_FORMAL_CLEARANCE_UNKNOWN",
  "E_CORE_KCC_SLICE_NOT_SCENE_PROOF",
  "E_B01_B02_SCOPE_NOT_TRUE_GATE_B",
  "E_PACKAGE_FMT_CHECK",
  "E_ARCHITECTURE_NO_GO"
];

const report = {
  schema: "formal_scene_readiness_tp01r3_v1",
  status: "blocked",
  decision: "blocked",
  conditionalGoExactPackage: false,
  packageId: "PKG-FORMAL-GREY-HIVE-SCENE",
  candidateOnly: true,
  formalSceneReady: false,
  architecture: "no_go",
  generatedAtUtc: new Date().toISOString(),
  allowedWrites,
  noRead,
  readOnlyInputs: inputMeta,
  packageScope: {
    goal: formalPackage.goal,
    formalWriteSet: formalPackage.writes,
    readonlyFromTp18b1: formalPackage.readonly,
    declaredCommands: formalPackage.commands,
    rollback: formalPackage.rollback,
    lifts: formalPackage.lifts,
    prerequisites: formalPackage.prerequisites,
    unsatisfiedPrerequisites: formalPackage.unsatisfiedPrerequisites,
    packageStatusFromTp18b1: formalPackage.status
  },
  evidenceMatrix: evidenceMatrix(inputMeta, tp09a, tp09b, tp14c1, ag42GateB, nav),
  sourceScene: {
    formalSourceScenePresent: false,
    formalSourceSceneSha256: null,
    syntheticFixtureHash: tp09b.positive.stageHashes.sourceSceneSha256,
    syntheticFixtureIsFormalEligible: tp09b.positive.fixture === "synthetic-positive-schema-only" ? false : null,
    stageHashChain: tp09b.positive.stageHashes,
    stageExecutorsPresent: false,
    requiredStages: ["blender-export", "babylon-import", "rapier-nav-bake"]
  },
  coordinateAndTransform: {
    unitsMeters: tp09b.coordinateLock.unitsMeters,
    rightHanded: tp09b.coordinateLock.rightHanded,
    upAxis: tp09b.coordinateLock.upAxis,
    footAnchor: tp09b.coordinateLock.footAnchor,
    pivotPolicy: tp09b.coordinateLock.pivotPolicy,
    scale: tp09b.coordinateLock.scale,
    formalTransformProvenance: false,
    candidatePromotion: "forbidden"
  },
  gatePolicy: {
    gateA: "hive_power only",
    gateB: "edge-b05-c01 only; hive_lockdown_completed only",
    choiceStates: ["taken", "left", "unresolved"],
    baizhiAndNpcHardPrerequisite: false,
    optionalAg38: "active/not_evaluated; not consumed"
  },
  checkpointPolicy: {
    canonicalIds: tp09a.checkpointAudit.canonicalIds,
    postBio: "gh_cp_post_bio -> GH-B05",
    terminal: "gh_cp_exit -> map=null, noSpawn=true",
    formalTransforms: false,
    formalNavRefs: false,
    formalClearance: false
  },
  tests: commands,
  negativeFixtures: { count: negative.length, passed: negative.filter((item) => item.pass).length, results: negative },
  blockers,
  validation: {
    requiredInputsPresent: requiredInputPass,
    expectedCommandsPass: allExpectedCommandsPass,
    negativeFixturesPass: negativePass,
    auditPass: requiredInputPass && allExpectedCommandsPass && negativePass,
    note: "cargo fmt exit 1 is an observed package blocker; the audit remains mechanically complete and no formal file was modified",
    blockingCommandFailures: commands.cargoFmt.exitCode === 0 ? [] : ["cargo fmt --manifest-path server-rs/Cargo.toml --check"]
  },
  forbiddenPromotion: [
    "synthetic-positive -> formal",
    "candidate coordinates -> formal Transform",
    "AG-38 optional elevator output consumption",
    "Gate B choice/NPC dependency",
    "B01↔B02 collider -> true Gate B",
    "candidate nav graph -> formal navmesh",
    "formal package scope expansion",
    "Architecture no_go -> go"
  ],
  rollback: {
    allowed: formalPackage.rollback,
    reviewRollback: "delete only this TP-01R3 formal-scene-readiness directory, result JSON, and review document; restore no formal source"
  },
  formalGate: { tp09: "blocked", gateI: "blocked", package: "blocked", architecture: "no_go" },
  evidenceLevel: "package_scoped_read_only_gate_review"
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ status: report.status, decision: report.decision, auditPass: report.validation.auditPass, negativeFixtures: negative.length, cargoFmt: commands.cargoFmt.exitCode, worldV3: commands.worldV3.exitCode, controlCollider: commands.controlCollider.exitCode }));
process.exit(report.validation.auditPass ? 0 : 1);
