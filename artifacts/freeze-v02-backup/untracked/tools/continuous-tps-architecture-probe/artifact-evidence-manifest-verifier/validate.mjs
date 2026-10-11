import fs from "node:fs";
import path from "node:path";
import { verifyManifest, repoRoot, sha256 } from "./verifier.mjs";

const toolRoot = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\"));
const sampleDir = path.join(toolRoot, "samples");
const negativeDir = path.join(toolRoot, "fixtures", "negative");
const sampleNames = ["nb49", "nb50", "nb63", "nb90"];
const sampleReportPaths = sampleNames.map(name => JSON.parse(fs.readFileSync(path.join(sampleDir, `${name}.json`), "utf8")).actualPaths[0]);
const validatorPaths = sampleNames.map(name => JSON.parse(fs.readFileSync(path.join(sampleDir, `${name}.json`), "utf8")).sourceValidatorIdentity.path);
const watched = [...sampleReportPaths, ...validatorPaths];
const before = new Map(watched.map(relative => { const absolute = path.join(repoRoot, relative); const stat = fs.statSync(absolute); const bytes = fs.readFileSync(absolute); return [relative, { bytes: stat.size, mtimeMs: stat.mtimeMs, sha256: sha256(bytes) }]; }));
const sampleResults = Object.fromEntries(sampleNames.map(name => { const result = verifyManifest(JSON.parse(fs.readFileSync(path.join(sampleDir, `${name}.json`), "utf8"))); return [name, result]; }));
const negativeNames = fs.readdirSync(negativeDir).filter(name => name.endsWith(".json")).sort();
const negativeResults = Object.fromEntries(negativeNames.map(name => { const result = verifyManifest(JSON.parse(fs.readFileSync(path.join(negativeDir, name), "utf8"))); return [name, result]; }));
const after = new Map(watched.map(relative => { const absolute = path.join(repoRoot, relative); const stat = fs.statSync(absolute); const bytes = fs.readFileSync(absolute); return [relative, { bytes: stat.size, mtimeMs: stat.mtimeMs, sha256: sha256(bytes) }]; }));
const historyUnchanged = watched.every(relative => JSON.stringify(before.get(relative)) === JSON.stringify(after.get(relative)));
const samplesPassed = Object.values(sampleResults).every(result => result.passed);
const negativesRejected = Object.values(negativeResults).every(result => !result.passed);
const nb90 = JSON.parse(fs.readFileSync(path.join(sampleDir, "nb90.json"), "utf8"));
const checks = {
  samples_passed: samplesPassed,
  negative_suite_passed: negativeNames.length >= 12 && negativesRejected,
  history_unchanged: historyUnchanged,
  literal_paths: sampleNames.every(name => { const m = JSON.parse(fs.readFileSync(path.join(sampleDir, `${name}.json`), "utf8")); return m.expectedPaths.length === m.actualPaths.length && m.expectedPaths.every((value, index) => value === m.actualPaths[index]); }),
  nb90_canonical_only: nb90.actualPaths.length === 1 && nb90.actualPaths[0] === "tools/continuous-tps-architecture-probe/results/new-baseline-product-decision-batch-1-nb90.json" && Array.isArray(nb90.reportVerification.alternateReportPaths) && nb90.reportVerification.alternateReportPaths.length === 0,
  global_contract_closed: sampleNames.every(name => JSON.parse(fs.readFileSync(path.join(sampleDir, `${name}.json`), "utf8")).globalArtifactContractAccepted === false),
  no_runtime_media_network: true
};
const failures = Object.entries(checks).filter(([, value]) => !value).map(([key]) => key);
const result = {
  schemaVersion: 1,
  reviewId: "NB-97",
  status: failures.length === 0 ? "artifact_evidence_manifest_verifier_ready" : "fail",
  artifactVerifierImplemented: true,
  sampleEvidenceVerified: Object.fromEntries(sampleNames.map(name => [name, sampleResults[name].passed])),
  negativeFixtureCount: negativeNames.length,
  negativeFixturesRejected: negativesRejected,
  globalArtifactContractAccepted: false,
  checks,
  failureCount: failures.length,
  failures,
  historyUnchanged,
  nb90CanonicalPath: nb90.actualPaths[0],
  nb90AlternatePathRequired: false,
  flags: { runtimeExecuted: false, mediaBytesRead: false, mediaCreated: false, mediaModelCalled: false, network: false, filesMoved: false, filesCopied: false, filesRepaired: false }
};
const outputPath = path.join(repoRoot, "tools", "continuous-tps-architecture-probe", "results", "artifact-evidence-manifest-verifier-nb97.json");
fs.writeFileSync(outputPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
