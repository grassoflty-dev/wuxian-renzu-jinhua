import fs from "node:fs";
import crypto from "node:crypto";
import { execFileSync } from "node:child_process";

const root = new URL("../../../", import.meta.url).pathname.replace(/^\//, "").replaceAll("/", "\\");
const reviewPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-lib-rs-change-provenance-review\\review.json`;
const preflightPath = `${root}tools\\continuous-tps-architecture-probe\\new-baseline-grey-hive-gh-b0-implementation-preflight\\preflight.json`;
const source = "server-rs/src/lib.rs";
const hash = bytes => crypto.createHash("sha256").update(bytes).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" });
const review = JSON.parse(fs.readFileSync(reviewPath, "utf8"));
const preflight = JSON.parse(fs.readFileSync(preflightPath, "utf8"));
const current = fs.readFileSync(`${root}${source.replaceAll("/", "\\")}`);
const diff = git("diff", "--unified=0", "--", source);
const hunks = diff.split(/\r?\n/).filter(line => line.startsWith("@@"));
const status = git("status", "--short", "--", source).trim();
const exactFiles = preflight.changeManifestProposal.exactFiles;
const planned = review.ghB0OverlapAnalysis.plannedExactFile;
const expectedAnchor = "pub mod world_v3;";
const checks = [
  [review.status === "lib_rs_change_provenance_review_complete", "status"],
  [source === review.repositoryIdentity.sourcePath, "source path"],
  [exactFiles.includes(source) && planned, "GH-B0 exact file"],
  [review.ghB0OverlapAnalysis.exactAnchor === expectedAnchor, "anchor"],
  [review.ghB0OverlapAnalysis.exactHunkOverlap === true, "hunk overlap"],
  [review.ghB0OverlapAnalysis.unrelatedDeclarations === true, "unrelated declarations"],
  [status === "M server-rs/src/lib.rs", "worktree status"],
  [review.statusSnapshot.staged === false && review.statusSnapshot.unstaged === true, "staged state"],
  [hash(current) === review.currentFingerprint.fileSha256, "current SHA-256"],
  [review.baselineFingerprint.fileSha256 === review.currentFingerprint.fileSha256, "baseline fingerprint"],
  [hunks.length === 22 && review.diffSummary.hunkCount === 22, "hunk count"],
  [review.diffSummary.added === 54 && review.diffSummary.deleted === 43, "diff summary"],
  [git("diff", "--cached", "--quiet", "--", source) === "", "staged diff empty"],
  [review.provenanceEvidence.resolved === false && review.classification === "unattributed_worktree_change", "provenance"],
  [review.lockDecision.decision === "conflict" && review.lockDecision.activeWriteConflict === true, "lock decision"],
  [review.flags.libRsModified === true && review.flags.ghB0LockConflict === true, "flags"],
  [review.flags.architectureGateOpened === false && review.flags.ghB0Approved === false && review.flags.implementationStarted === false, "gate closed"],
  [review.flags.dataModified === false && review.flags.buildExecuted === false && review.flags.testsExecuted === false && review.flags.runtimeExecuted === false && review.flags.network === false, "no side effects"]
];
const failures = checks.filter(([, name]) => !checks.find(([ok, n]) => n === name)?.[0]).map(([, name]) => name);
const result = {
  schemaVersion: 1,
  reviewId: "NB-79",
  validator: "tools/continuous-tps-architecture-probe/new-baseline-lib-rs-change-provenance-review/validate.mjs",
  status: failures.length === 0 ? "pass_gate_blocked" : "fail",
  checks: Object.fromEntries(checks.map(([ok, name]) => [name, ok])),
  failureCount: failures.length,
  failures,
  decision: failures.length === 0 ? "preserve_unattributed_change_and_keep_gh_b0_locked" : "invalid_review_artifact",
  source: { path: source, sha256: hash(current), gitStatus: status, hunkCount: hunks.length },
  sideEffectPolicy: { sourceModified: false, lockCreated: false, staged: false, committed: false, build: false, test: false, runtime: false, network: false }
};
const resultPath = `${root}tools\\continuous-tps-architecture-probe\\results\\new-baseline-lib-rs-change-provenance-review-nb79.json`;
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + "\n", "utf8");
console.log(JSON.stringify(result, null, 2));
if (failures.length) process.exit(1);
