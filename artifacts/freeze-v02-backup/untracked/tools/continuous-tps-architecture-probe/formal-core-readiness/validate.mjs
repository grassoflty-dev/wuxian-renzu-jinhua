import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const HERE = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, "").replace(/^([A-Za-z]):/, "$1:"));
const ROOT = path.resolve(HERE, "../../..");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/formal-core-readiness-tp01r2.json");
const FIXTURE_PATH = path.join(HERE, "negative-fixtures.json");

function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function clone(value) { return JSON.parse(JSON.stringify(value)); }
function getAt(obj, dotted) { return dotted.split(".").reduce((v, key) => v?.[key], obj); }
function setAt(obj, dotted, value) {
  const parts = dotted.split(".");
  const last = parts.pop();
  const parent = parts.reduce((v, key) => v[key] ??= {}, obj);
  parent[last] = value;
}
function sha256(file) { return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function hasString(value, needle) {
  if (typeof value === "string") return value.toLowerCase().includes(needle.toLowerCase());
  if (Array.isArray(value)) return value.some((item) => hasString(item, needle));
  if (value && typeof value === "object") return Object.values(value).some((item) => hasString(item, needle));
  return false;
}

function validateModel(model, { checkSources = true } = {}) {
  const errors = [];
  const add = (code, detail) => errors.push({ code, detail });
  if (model.schema !== "formal-core-readiness-tp01r2-v1") add("E_SCHEMA", "unexpected schema");
  if (model.status !== "blocked" || model.conclusion !== "blocked") add("E_CONCLUSION_NOT_BLOCKED", "the audit must end blocked");
  if (model.architectureGate !== "no_go") add("E_ARCHITECTURE_GO", "Architecture must remain no_go");
  if (model.allowedWrites?.length) add("E_WRITE_SCOPE_EXPANDED", "blocked package must not authorize writes");
  if (model.candidatePackageWriteSet?.some((p) => /elevator|media|assets|public/i.test(p))) add("E_DYNAMIC_ELEVATOR_SCOPE", "candidate package contains forbidden elevator/media scope");
  if (model.package?.dynamicElevatorResponsibilities) add("E_DYNAMIC_ELEVATOR_SCOPE", "dynamic elevator responsibility was included");
  if (hasString(model, "formal_pass")) add("E_CANDIDATE_TO_FORMAL", "candidate evidence was promoted to a formal gate result");
  if (hasString(model, "ag-38") || hasString(model, "ag38") || hasString(model, "tp-14c1") || hasString(model, "tp14c1")) add("E_UNAUTHORIZED_ACTIVE_OUTPUT", "active out-of-scope output was consumed");
  if (model.conditionalGate && model.dependencyReview?.licenseProvenance !== true) add("E_CARGO_LOCK_LICENSE_MISSING", "Cargo/lock dependency license provenance is not complete");
  if (model.conditionalGate && model.performance?.profileUsedForRelease === "debug") add("E_DEBUG_AS_RELEASE", "debug timings cannot support release performance");
  if (model.conditionalGate && model.rollback?.complete !== true) add("E_ROLLBACK_INCOMPLETE", "rollback scope is not complete");
  if (model.conditionalGate && model.gateB?.choiceNpcIndependent !== true) add("E_GATE_B_TRUTH_NOT_FREE", "Gate B choice/NPC truth is not independent");
  if (model.dirtyBaseline?.disclosed !== true) add("E_DIRTY_BASELINE_UNDISCLOSED", "dirty baseline must be disclosed");
  const commandIds = new Set((model.commands ?? []).map((c) => c.id));
  for (const id of ["fixed_step_specialized", "continuous_kcc_specialized", "world_v3_specialized", "full_debug_regression", "full_release_regression"]) {
    const command = model.commands?.find((c) => c.id === id);
    if (!command || command.exitCode !== 0) add("E_FULL_REGRESSION_UNOBSERVED", `${id} did not complete with exit code 0`);
  }
  if (!model.specializedTests?.everyRequired) add("E_MISSING_SPECIALIZED_TESTS", "formal WorldV3/KCC/fixed-step tests are not all required");
  if (model.conditionalGate && (!model.gateB?.choiceNpcIndependent || !model.gateB?.threeChoiceMatrix)) add("E_GATE_B_TRUTH_NOT_FREE", "Gate B matrix is incomplete");
  if (checkSources) {
    for (const source of model.sourceInputs ?? []) {
      const absolute = path.join(ROOT, source.path);
      if (!fs.existsSync(absolute)) { add("E_SOURCE_MISSING", source.path); continue; }
      if (sha256(absolute) !== source.sha256) add("E_SOURCE_HASH_CHANGED", source.path);
    }
  }
  return errors;
}

const model = readJson(RESULT_PATH);
const fixtureSpec = readJson(FIXTURE_PATH);
const errors = validateModel(model);
const fixtureResults = [];
for (const fixture of fixtureSpec.fixtures) {
  const mutated = clone(model);
  for (const change of fixture.patches ?? []) setAt(mutated, change.path, change.value);
  if (fixture.path) setAt(mutated, fixture.path, fixture.value);
  if (fixture.deletePath) {
    const parts = fixture.deletePath.split(".");
    const last = parts.pop();
    const parent = parts.reduce((v, key) => v?.[key], mutated);
    if (parent) delete parent[last];
  }
  const actual = validateModel(mutated, { checkSources: false }).map((e) => e.code);
  const pass = actual.includes(fixture.expectedCode);
  fixtureResults.push({ id: fixture.id, expectedCode: fixture.expectedCode, observedCodes: actual, pass });
}
const fixturePass = fixtureResults.length >= 10 && fixtureResults.every((entry) => entry.pass);
const output = {
  ...model,
  validation: {
    validator: "tools/continuous-tps-architecture-probe/formal-core-readiness/validate.mjs",
    ok: errors.length === 0,
    errors,
    negativeFixtureCount: fixtureResults.length,
    negativeFixturesPass: fixturePass,
    negativeFixtureResults: fixtureResults,
    generatedAtUtc: new Date().toISOString()
  }
};
if (process.argv.includes("--write")) fs.writeFileSync(RESULT_PATH, `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify({ ok: errors.length === 0 && fixturePass, status: model.status, architectureGate: model.architectureGate, negativeFixtureCount: fixtureResults.length, negativeFixturesPass: fixturePass, errors }, null, 2));
process.exitCode = errors.length === 0 && fixturePass ? 0 : 1;
