import fs from "node:fs";
import path from "node:path";
import { aggregate, hashFile, root, syntheticAggregate, toolInputs } from "./aggregator.mjs";

const fixtures = JSON.parse(fs.readFileSync(path.join(path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\")), "fixtures.json"), "utf8"));
const watched = toolInputs.flatMap(input => [input.resultPath, input.toolPath]);
const snapshot = () => new Map(watched.map(relative => [relative, { sha256: hashFile(relative), bytes: fs.statSync(path.join(root, relative)).size, mtimeMs: fs.statSync(path.join(root, relative)).mtimeMs }]));
const before = snapshot();
const actual = aggregate();
const positiveResults = Object.fromEntries(fixtures.positive.map(fixture => [fixture.id, syntheticAggregate(fixture)]));
const negativeResults = Object.fromEntries(fixtures.negative.map(fixture => [fixture.id, syntheticAggregate(fixture)]));
const after = snapshot();
const historyUnchanged = watched.every(relative => JSON.stringify(before.get(relative)) === JSON.stringify(after.get(relative)));
const positivePassed = fixtures.positive.every(fixture => { const result = positiveResults[fixture.id]; return result.accepted === true && result.l0 === fixture.expected.l0 && result.l1 === fixture.expected.l1 && result.l2 === fixture.expected.l2 && result.l3 === fixture.expected.l3 && result.full === fixture.expected.full && result.gate === fixture.expected.gate; });
const negativeRejected = fixtures.negative.every(fixture => negativeResults[fixture.id].accepted === false);
const checks = {
  actual_l0_l3: actual.staticInputsL0L3Verified === true,
  actual_full_gate_closed: actual.fullGateEvidenceComplete === false && actual.architectureGateOpened === false,
  actual_blockers_open: actual.blockersOpen === 5 && actual.blockersClosed === 0,
  actual_nb90_pending: actual.nb90Answers === 0,
  actual_global_artifact_closed: actual.globalArtifactContractAccepted === false,
  positive_suite: fixtures.positive.length >= 3 && positivePassed,
  negative_suite: fixtures.negative.length >= 15 && negativeRejected,
  history_unchanged: historyUnchanged,
  missing_layers_explicit: actual.layers.find(layer => layer.id === "L4")?.status === "pending" && actual.layers.find(layer => layer.id === "L5")?.status === "missing" && actual.layers.find(layer => layer.id === "L6")?.status === "missing"
};
const failures = Object.entries(checks).filter(([, value]) => !value).map(([key]) => key);
const result = {
  schemaVersion: 1,
  reviewId: "NB-102",
  status: failures.length === 0 ? "formal_static_gate_aggregator_ready" : "fail",
  artifactVerifierImplemented: true,
  staticInputsL0L3Verified: actual.staticInputsL0L3Verified,
  fullGateEvidenceComplete: actual.fullGateEvidenceComplete,
  architectureGateOpened: actual.architectureGateOpened,
  blockersOpen: actual.blockersOpen,
  blockersClosed: actual.blockersClosed,
  nb90Answers: actual.nb90Answers,
  globalArtifactContractAccepted: actual.globalArtifactContractAccepted,
  sourceIdentities: actual.identities,
  layerMatrix: actual.layers,
  missingEvidence: actual.layers.filter(layer => !layer.complete).map(layer => ({ id: layer.id, status: layer.status, evidence: layer.evidence })),
  positiveFixtures: positiveResults,
  negativeFixtures: negativeResults,
  positiveFixtureCount: fixtures.positive.length,
  negativeFixtureCount: fixtures.negative.length,
  negativeFixturesRejected: negativeRejected,
  checks,
  failureCount: failures.length,
  failures,
  historyUnchanged,
  currentDecision: "static_inputs_ready_l0_l3_only_gate_closed",
  flags: { runtimeExecuted: false, mediaBytesRead: false, mediaCreated: false, mediaModelCalled: false, network: false, sourceModified: false, upstreamResultsModified: false, userSaveRead: false, logsRead: false }
};
const outputPath = path.join(root, "tools", "continuous-tps-architecture-probe", "results", "formal-static-gate-aggregator-nb102.json");
fs.writeFileSync(outputPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
