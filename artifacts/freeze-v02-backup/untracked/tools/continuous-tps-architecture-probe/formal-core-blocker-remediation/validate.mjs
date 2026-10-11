import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";

const HERE = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "").replace(/^([A-Za-z]):/, "$1:"));
const ROOT = path.resolve(HERE, "../../..");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/formal-core-blocker-remediation-ag44.json");
const FIXTURE_PATH = path.join(HERE, "negative-fixtures.json");
const PACKAGE_WRITE_SET = [
  "server-rs/src/lib.rs",
  "server-rs/src/fixed_step.rs",
  "server-rs/src/continuous_kcc.rs",
  "server-rs/Cargo.toml",
  "server-rs/Cargo.lock",
  "server-rs/tests/fixed_step_test.rs",
  "server-rs/tests/continuous_kcc_test.rs"
];
const ARTIFACT_PATHS = [
  "tools/continuous-tps-architecture-probe/formal-core-blocker-remediation/validate.mjs",
  "tools/continuous-tps-architecture-probe/formal-core-blocker-remediation/negative-fixtures.json",
  "tools/continuous-tps-architecture-probe/formal-core-blocker-remediation/",
  "tools/continuous-tps-architecture-probe/results/formal-core-blocker-remediation-ag44.json",
  "docs/CONTINUOUS_FORMAL_CORE_BLOCKER_REMEDIATION.md"
];

function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function clone(value) { return JSON.parse(JSON.stringify(value)); }
function setAt(obj, dotted, value) {
  const parts = dotted.split(".");
  const last = parts.pop();
  const parent = parts.reduce((v, key) => v[key] ??= {}, obj);
  parent[last] = value;
}
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function canonical(value) { return JSON.stringify(value); }
function hasString(value, needle) {
  if (typeof value === "string") return value.toLowerCase().includes(needle.toLowerCase());
  if (Array.isArray(value)) return value.some((item) => hasString(item, needle));
  if (value && typeof value === "object") return Object.values(value).some((item) => hasString(item, needle));
  return false;
}

function cargoMetadata() {
  const proc = spawnSync("cargo", ["metadata", "--manifest-path", "server-rs/Cargo.toml", "--locked", "--offline", "--format-version", "1"], { cwd: ROOT, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  if (proc.status !== 0) return { exitCode: proc.status ?? 1, error: (proc.error?.message || proc.stderr || "cargo metadata failed").trim(), packages: [] };
  try { return { exitCode: 0, error: null, packages: JSON.parse(proc.stdout).packages }; }
  catch (error) { return { exitCode: 1, error: `metadata JSON parse failed: ${error.message}`, packages: [] }; }
}

function buildLicenseLedger(metadata) {
  const root = metadata.find((pkg) => pkg.name === "wuxian-horror-ch1" && !pkg.source);
  const direct = new Set((root?.dependencies ?? []).map((dep) => dep.name));
  const packages = metadata.filter((pkg) => pkg !== root);
  const ledger = packages.map((pkg) => {
    const manifestPath = pkg.manifest_path;
    const manifestExists = Boolean(manifestPath && fs.existsSync(manifestPath));
    const licenseFilePath = pkg.license_file ? path.resolve(path.dirname(manifestPath), pkg.license_file) : null;
    const licenseEvidence = pkg.license ? "manifest_license_field" : pkg.license_file ? "manifest_license_file" : "missing";
    return {
      name: pkg.name,
      version: pkg.version,
      kind: direct.has(pkg.name) ? "direct" : "transitive",
      source: pkg.source ?? "path",
      license: pkg.license ?? null,
      licenseFile: pkg.license_file ?? null,
      licenseEvidence,
      manifestPath,
      manifestSha256: manifestExists ? sha256(manifestPath) : null,
      licenseFilePresent: Boolean(licenseFilePath && fs.existsSync(licenseFilePath))
    };
  }).sort((a, b) => `${a.name}@${a.version}`.localeCompare(`${b.name}@${b.version}`));
  return {
    metadataCommand: "cargo metadata --manifest-path server-rs/Cargo.toml --locked --offline --format-version 1",
    metadataExitCode: 0,
    rootPackage: { name: root?.name ?? "wuxian-horror-ch1", version: root?.version ?? null, license: root?.license ?? null, licenseFile: root?.license_file ?? null, licenseEvidence: root?.license ? "manifest_license_field" : root?.license_file ? "manifest_license_file" : "missing", manifestPath: root?.manifest_path ?? null },
    directDependencyNames: [...direct].sort(),
    dependencyCount: ledger.length,
    missingLicenseCount: ledger.filter((entry) => entry.licenseEvidence === "missing").length,
    provenanceComplete: ledger.every((entry) => entry.licenseEvidence !== "missing" && entry.manifestSha256),
    ledgerSha256: crypto.createHash("sha256").update(canonical(ledger)).digest("hex"),
    ledger
  };
}

function gitLedger() {
  const proc = spawnSync("git", ["status", "--porcelain=v1"], { cwd: ROOT, encoding: "utf8" });
  if (proc.status !== 0) return { exitCode: proc.status ?? 1, entries: [], error: (proc.stderr || proc.stdout || "").trim() };
  const entries = proc.stdout.split(/\r?\n/).filter(Boolean).map((line) => {
    const status = line.slice(0, 2);
    const file = line.startsWith("??") ? line.slice(3) : line.slice(3);
    return { status, path: file };
  });
  return { exitCode: 0, entries };
}

function summarizeGit(entries, excluded = []) {
  const excludedSet = new Set(excluded);
  const filtered = entries.filter((entry) => !excludedSet.has(entry.path));
  let indexChanged = 0, worktreeChanged = 0, untracked = 0;
  for (const entry of filtered) {
    if (entry.status === "??") { untracked++; worktreeChanged++; continue; }
    if (entry.status[0] !== " ") indexChanged++;
    if (entry.status[1] !== " ") worktreeChanged++;
  }
  return { total: filtered.length, indexChanged, worktreeChanged, untracked, trackedOrStaged: filtered.length - untracked };
}

function validateModel(model, { checkSources = true, allowMissingLedger = false } = {}) {
  const errors = [];
  const add = (code, detail) => errors.push({ code, detail });
  if (model.schema !== "formal-core-blocker-remediation-ag44-v1") add("E_SCHEMA", "unexpected schema");
  if (model.status !== "blocked" || model.architectureGate !== "no_go") add("E_GATE_ESCALATION", "AG-44 must preserve blocked/no_go");
  if (model.allowedWrites?.length) add("E_WRITE_SCOPE_EXPANDED", "no formal implementation write is authorized by this evidence package");
  if (model.grounded?.authoritativeField !== "grounded" || model.grounded?.rawKccFieldRole !== "diagnostic_only") add("E_GROUNDED_DUAL_AUTHORITY", "grounded must have one authority and raw KCC must remain diagnostic");
  if (model.grounded?.dynamicBodiesIncluded === true || model.dynamicElevatorInCore === true) add("E_DYNAMIC_ELEVATOR_SCOPE", "dynamic elevator or dynamic-body scope entered the core package");
  if (model.actions?.automaticFormatOrCleanup === true) add("E_AUTOMATIC_CLEANUP", "formatting/cleanup mutation is forbidden");
  if (hasString(model, "formal_pass")) add("E_CANDIDATE_TO_FORMAL", "candidate evidence was promoted");
  if (hasString(model, "ag-38") || hasString(model, "ag38") || hasString(model, "tp-14c1") || hasString(model, "tp14c1")) add("E_UNAUTHORIZED_ACTIVE_OUTPUT", "active out-of-scope output was consumed");
  if (model.dirtyLedger?.classification?.formalPackage?.provenance !== "preexisting_before_ag44") add("E_DIRTY_MISATTRIBUTED", "formal package dirty state must remain preexisting and unattributed");
  if (model.licenseReview?.provenanceComplete !== true) add("E_LICENSE_PROVENANCE_INCOMPLETE", "license evidence is incomplete");
  if ((model.licenseLedger ?? []).some((entry) => entry.licenseEvidence === "inferred")) add("E_LICENSE_GUESS", "license cannot be inferred without a manifest field or license file");
  if ((model.licenseLedger ?? []).some((entry) => entry.licenseEvidence === "missing")) add("E_LICENSE_MISSING", "a locked dependency has no local license evidence");
  if (model.rollback?.everyFuturePackageHasScopedRollback !== true) add("E_ROLLBACK_INCOMPLETE", "each proposed package needs a scoped rollback");
  if (!Array.isArray(model.blockerLedger) || model.blockerLedger.length < 7) add("E_BLOCKER_LEDGER_INCOMPLETE", "TP-01R2 blockers are not all classified");
  if (!Array.isArray(model.futurePackages) || model.futurePackages.length < 5) add("E_REMEDIATION_SPLIT_INCOMPLETE", "minimum future package split is incomplete");
  if (checkSources) {
    for (const source of model.sourceInputs ?? []) {
      const absolute = path.join(ROOT, source.path);
      if (!fs.existsSync(absolute)) add("E_SOURCE_MISSING", source.path);
      else if (source.sha256 && sha256(absolute) !== source.sha256) add("E_SOURCE_HASH_CHANGED", source.path);
    }
    const git = gitLedger();
    if (git.exitCode !== 0) add("E_GIT_STATUS_UNAVAILABLE", git.error);
    else {
      const observed = summarizeGit(git.entries, ARTIFACT_PATHS);
      if (JSON.stringify(observed) !== JSON.stringify(model.dirtyLedger.baselineBeforeAg44)) add("E_DIRTY_COUNT_DRIFT", `expected ${JSON.stringify(model.dirtyLedger.baselineBeforeAg44)} observed ${JSON.stringify(observed)}`);
      const packageMap = new Map(git.entries.map((entry) => [entry.path, entry]));
      for (const record of model.dirtyLedger.formalPackageRecords ?? []) {
        const observedEntry = packageMap.get(record.path);
        if (!observedEntry || observedEntry.status !== record.status) add("E_DIRTY_PACKAGE_STATUS_DRIFT", record.path);
        else if (sha256(path.join(ROOT, record.path)) !== record.sha256) add("E_DIRTY_PACKAGE_HASH_DRIFT", record.path);
      }
    }
  }
  const metadata = cargoMetadata();
  if (metadata.exitCode !== 0) add("E_CARGO_METADATA_OFFLINE", metadata.error);
  else if (!allowMissingLedger) {
    const fresh = buildLicenseLedger(metadata.packages);
    if (fresh.ledgerSha256 !== model.licenseReview?.ledgerSha256) add("E_LICENSE_LEDGER_STALE", "local locked manifest/license ledger changed");
  }
  return errors;
}

const model = readJson(RESULT_PATH);
const fixtures = readJson(FIXTURE_PATH).fixtures;
const initialErrors = validateModel(model, { allowMissingLedger: process.argv.includes("--write") });
const metadata = cargoMetadata();
if (process.argv.includes("--write") && metadata.exitCode === 0) {
  model.licenseReview = buildLicenseLedger(metadata.packages);
  model.licenseLedger = model.licenseReview.ledger;
}
const errors = process.argv.includes("--write") ? validateModel(model) : initialErrors;
const fixtureResults = [];
for (const fixture of fixtures) {
  const mutated = clone(model);
  for (const change of fixture.patches ?? []) setAt(mutated, change.path, change.value);
  if (fixture.path) setAt(mutated, fixture.path, fixture.value);
  const actual = validateModel(mutated, { checkSources: false, allowMissingLedger: false }).map((entry) => entry.code);
  fixtureResults.push({ id: fixture.id, expectedCode: fixture.expectedCode, observedCodes: actual, pass: actual.includes(fixture.expectedCode) });
}
const fixturePass = fixtureResults.length >= 10 && fixtureResults.every((entry) => entry.pass);
const output = { ...model, validation: { validator: "tools/continuous-tps-architecture-probe/formal-core-blocker-remediation/validate.mjs", ok: errors.length === 0, errors, negativeFixtureCount: fixtureResults.length, negativeFixturesPass: fixturePass, negativeFixtureResults: fixtureResults, generatedAtUtc: new Date().toISOString() } };
if (process.argv.includes("--write")) fs.writeFileSync(RESULT_PATH, `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify({ ok: errors.length === 0 && fixturePass, status: model.status, architectureGate: model.architectureGate, dependencyCount: model.licenseReview?.dependencyCount ?? 0, missingLicenseCount: model.licenseReview?.missingLicenseCount ?? null, negativeFixtureCount: fixtureResults.length, negativeFixturesPass: fixturePass, errors }, null, 2));
process.exitCode = errors.length === 0 && fixturePass ? 0 : 1;
