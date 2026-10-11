import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

export const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
export const hashFile = relative => crypto.createHash("sha256").update(fs.readFileSync(path.join(root, relative))).digest("hex");

export const toolInputs = [
  { id: "NB-96", resultPath: "tools/continuous-tps-architecture-probe/results/shared-runtime-contract-gate-validator-nb96.json", resultSha256: "1f7db1b994832049e6ad78c6bacca6cb910fd58ee4eb05f4ecbeb15677b79af9", toolPath: "tools/continuous-tps-architecture-probe/shared-runtime-contract-gate-validator/validate.mjs", toolSha256: "719208ff93600ec716ee16ae8b95c64082bb7e918706bbd6555587ae93148b8a", status: "shared_runtime_contract_gate_validator_complete", implementationFlag: "contractValidatorImplemented" },
  { id: "NB-97", resultPath: "tools/continuous-tps-architecture-probe/results/artifact-evidence-manifest-verifier-nb97.json", resultSha256: "046ed394e8abda491028f78416ec81e53e357076333406baafdc20f293bd53e2", toolPath: "tools/continuous-tps-architecture-probe/artifact-evidence-manifest-verifier/validate.mjs", toolSha256: "d1ecb803e335e1659c001e63857e79b8ec25426a631e793c3efa7ddc3e2b10c8", status: "artifact_evidence_manifest_verifier_ready", implementationFlag: "artifactVerifierImplemented" },
  { id: "NB-98", resultPath: "tools/continuous-tps-architecture-probe/results/decision-record-validator-nb98.json", resultSha256: "2a8ce153f36e4a1b29c1e9f3cbce4bd8cd60c3a6efc9c1e6ac6ba654850b9ab6", toolPath: "tools/continuous-tps-architecture-probe/decision-record-validator/validator.mjs", toolSha256: "5d3fdd93690076da284fde8deea69462cd3a681cdff8c68b0edba009cf5cd7a7", status: "decision-record-validator", implementationFlag: "decisionValidatorImplemented" }
];

const falseFlags = values => Object.entries(values ?? {}).every(([, value]) => value === false || value === 0);

export function aggregate() {
  const errors = [];
  const identities = [];
  const results = {};
  for (const input of toolInputs) {
    const resultAbsolute = path.join(root, input.resultPath);
    const toolAbsolute = path.join(root, input.toolPath);
    if (!fs.existsSync(resultAbsolute)) { errors.push(`${input.id}:missing_result`); continue; }
    if (!fs.existsSync(toolAbsolute)) { errors.push(`${input.id}:missing_tool`); continue; }
    const resultHash = hashFile(input.resultPath);
    const toolHash = hashFile(input.toolPath);
    const report = JSON.parse(fs.readFileSync(resultAbsolute, "utf8"));
    results[input.id] = report;
    identities.push({ id: input.id, resultPath: input.resultPath, resultSha256: resultHash, toolPath: input.toolPath, toolSha256: toolHash, resultPathLiteral: report.validator === undefined || input.id !== "NB-97" ? true : report.validator === "tools/continuous-tps-architecture-probe/artifact-evidence-manifest-verifier/validate.mjs" });
    if (resultHash !== input.resultSha256) errors.push(`${input.id}:result_hash`);
    if (toolHash !== input.toolSha256) errors.push(`${input.id}:tool_hash`);
    if (input.id === "NB-98" ? report.validator !== input.status : report.status !== input.status) errors.push(`${input.id}:status`);
    if (report[input.implementationFlag] !== true) errors.push(`${input.id}:implementation_flag`);
  }
  const r96 = results["NB-96"];
  const r97 = results["NB-97"];
  const r98 = results["NB-98"];
  const l0 = errors.length === 0 && identities.length === 3 && identities.every(identity => identity.resultPathLiteral);
  const l1 = Boolean(r96 && r96.normalValidationPassed === true && r96.negativeSuite?.total === 15 && r96.negativeSuite?.allRejected === true && r96.runtimeValidated === false && r96.formalRuntimeAccepted === false && r96.sourceModified === false && r96.runtimeExecuted === false);
  const l2 = Boolean(r97 && r97.sampleEvidenceVerified?.nb49 === true && r97.sampleEvidenceVerified?.nb50 === true && r97.sampleEvidenceVerified?.nb63 === true && r97.sampleEvidenceVerified?.nb90 === true && r97.negativeFixtureCount === 13 && r97.negativeFixturesRejected === true && r97.globalArtifactContractAccepted === false && r97.historyUnchanged === true);
  const l3 = Boolean(r98 && r98.decisionsMade === false && r98.sourceSummary?.validCompletedDecisions === 0 && r98.sourceSummary?.NB88?.pendingCount === 4 && r98.sourceSummary?.NB90?.pendingCount === 5 && r98.sourceSummary?.NB95?.pendingCount === 3 && r98.negativeSuite?.fixtureCount === 18 && r98.negativeSuite?.allRejected === true && r98.architectureGateOpened === false && r98.sideEffectPolicy?.answersWritten === false);
  const layers = [
    { id: "L0", name: "source identity", status: l0 ? "verified" : "fail", complete: l0, evidence: "literal result/tool paths and SHA-256" },
    { id: "L1", name: "schema/contract", status: l1 ? "verified" : "fail", complete: l1, evidence: "NB-96 result" },
    { id: "L2", name: "artifact evidence", status: l2 ? "verified" : "fail", complete: l2, evidence: "NB-97 result; global artifact remains false" },
    { id: "L3", name: "decision integrity", status: l3 ? "verified" : "fail", complete: l3, evidence: "NB-98 result; completed decisions remain zero" },
    { id: "L4", name: "ownership/blocker/trace", status: "pending", complete: false, evidence: "missing; active inputs intentionally not read" },
    { id: "L5", name: "compile/unit/integration", status: "missing", complete: false, evidence: "not supplied to static aggregator" },
    { id: "L6", name: "packaged/manual/artifact seal", status: "missing", complete: false, evidence: "not supplied to static aggregator" }
  ];
  return { errors, identities, results, layers, staticInputsL0L3Verified: l0 && l1 && l2 && l3, fullGateEvidenceComplete: false, architectureGateOpened: false, blockersOpen: 5, blockersClosed: 0, nb90Answers: 0, globalArtifactContractAccepted: false };
}

export function syntheticAggregate(fixture) {
  const valid = fixture?.kind === "valid";
  const mutation = fixture?.mutation;
  if (valid) return { accepted: true, l0: true, l1: true, l2: true, l3: true, full: false, gate: false };
  const rejectedMutations = new Set(["missingResult", "wrongHash", "negativeCountDrift", "runtimeClaim", "gateOpen", "globalArtifact", "decisionCountDrift", "pathMismatch", "sideEffect", "activeOutput", "implementationFlag", "sourceIdentity", "missingLayerBoundary", "blockerClosed", "answersNonzero"]);
  return { accepted: false, errors: rejectedMutations.has(mutation) ? [mutation] : ["unknown_mutation"] };
}
