import fs from "node:fs";
import { execFileSync } from "node:child_process";

const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const planPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-first-formal-3d-return-station-plan-replacement\\plan.json`;
const plan = JSON.parse(fs.readFileSync(planPath, "utf8"));
const doc = fs.readFileSync(`${root}docs\\FIRST_FORMAL_3D_RETURN_STATION_PLAN_REPLACEMENT.md`, "utf8");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const source = fs.readFileSync(`${root}server-rs\\src\\lib.rs`, "utf8");
const requiredWorlds = ["grey_hive", "mist_harbor", "clockworks"];
const requiredBlockers = ["E_LEGACY_GRID_REACHABLE", "E_WORLD_V3_NOT_RUNTIME_WIRED", "E_CONTINUOUS_PRIMITIVES_MISSING", "E_API_FALLBACK_UNBACKED", "E_NO_FULL_SAVE_V3_RUNTIME_RESTORE"];
const requiredBatches = ["RS-B0", "RS-B1", "RS-B2", "RS-B3", "RS-B4"];
const flags = plan.currentDecision;
const checks = {
  status: plan.status === "first_formal_3d_return_station_plan_replacement_ready_for_supersession_review",
  new_identity: plan.planId === "NB-87" && plan.freshReplacementChecks.newPlanId === "NB-87",
  worlds_and_first: requiredWorlds.every(world => plan.sourceRequirements.worlds.includes(world)) && plan.sourceRequirements.firstWorld === "grey_hive",
  station_post_gate: plan.entryDependencies.entryDependenciesSatisfied === false && plan.currentDecision.returnStationApproved === false && plan.currentDecision.returnStationImplemented === false && plan.currentDecision.returnStationRuntimeLoaded === false,
  five_blockers: plan.entryDependencies.openBlockers.length === 5 && requiredBlockers.every(blocker => plan.entryDependencies.openBlockers.includes(blocker)) && flags.blockersOpen === 5 && flags.blockersClosed === 0,
  dependency_set: ["f1ToF7Closure", "greyHiveFormalPlayable", "threeWorldCampaignSaveTransition", "packagedOffline", "babylonPerformance", "artifactContract", "productDecisions"].every(key => plan.entryDependencies[key] === false),
  station_scope: plan.stationContract.mode === "contract_only" && plan.stationContract.minimumScope.includes("Tauri snapshot") && plan.stationContract.minimumScope.includes("Babylon presentation") && plan.stationContract.notWorldAcceptanceSubstitute === true,
  transition_contract: plan.transitionContract.authority === "Rust" && plan.transitionContract.noPartialTransition === true && plan.transitionContract.sourcePreservedOnFailure === true && plan.transitionContract.sequence.length >= 7,
  invariants: plan.invariants.includes("no Grid/API/mock/Three/Side2D fallback") && plan.invariants.includes("UI/Babylon cannot write state") && plan.invariants.includes("three worlds independently accepted"),
  batch_count: plan.batches.length <= 5 && plan.batches.length === 5 && requiredBatches.every(id => plan.batches.some(batch => batch.id === id)),
  batch_sizes: plan.batches.every(batch => batch.status === "not_approved" && batch.logicalUnits.length <= 8),
  first_batch_default_off_data_only: plan.batches[0].id === "RS-B0" && plan.batches[0].defaultOff === true && plan.batches[0].dataOnly === true,
  acceptance_complete: plan.acceptanceLadder.length >= 7 && plan.acceptanceLadder.includes("packaged Tauri offline") && plan.acceptanceLadder.includes("Babylon performance") && plan.acceptanceLadder.includes("immutable release artifact seal"),
  media_boundary: plan.assetMediaBoundary.mediaGenerated === false && plan.assetMediaBoundary.mediaRead === false && plan.assetMediaBoundary.mediaCannotSubstituteTechnicalEvidence === true,
  pending_not_filled: plan.pendingDecisions.length >= 8 && plan.pendingDecisions.some(item => item.includes("spatial dimensions")) && plan.pendingDecisions.some(item => item.includes("NPC/enemy")),
  replacement_safe: plan.freshReplacementChecks.nb37ContentRead === false && plan.freshReplacementChecks.oldPreserved === true && plan.freshReplacementChecks.automaticSupersession === false && plan.recommendation.accepted === false,
  no_old_content_reference: !doc.includes("NB-37 content") && !doc.includes("old NB-37 report"),
  source_unchanged: git("status", "--short", "--", "server-rs/src/lib.rs") === "M server-rs/src/lib.rs",
  no_gate_or_execution: flags.architectureGateOpened === false && flags.implementationStarted === false && flags.sourceModified === false && flags.buildExecuted === false && flags.testsExecuted === false && flags.runtimeExecuted === false,
  no_media_or_network: flags.assetBytesRead === false && flags.mediaBytesRead === false && flags.mediaCreated === false && flags.mediaModelCalled === false && flags.network === false
};
const failures = Object.entries(checks).filter(([, value]) => !value).map(([key]) => key);
const result = {
  schemaVersion: 1,
  reviewId: "NB-87",
  validator: "tools/continuous-tps-architecture-probe/new-baseline-first-formal-3d-return-station-plan-replacement/validate.mjs",
  status: failures.length === 0 ? "pass_gate_blocked" : "fail",
  checks,
  failureCount: failures.length,
  failures,
  decision: failures.length === 0 ? "replacement_ready_for_supersession_review_but_station_remains_post_gate" : "invalid_replacement_plan",
  sideEffectPolicy: { libRsModifiedByTask: false, sourceModified: false, build: false, tests: false, runtime: false, assets: false, media: false, network: false }
};
const resultPath = `${root}tools\\continuous-tps-architecture-probe\\results\\new-baseline-first-formal-3d-return-station-plan-replacement-nb87.json`;
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
