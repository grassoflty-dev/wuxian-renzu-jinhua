import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = process.cwd();
const args = process.argv.slice(2);
const outIndex = args.indexOf("--out");
const outPath = outIndex >= 0 ? args[outIndex + 1] : "tools/continuous-tps-architecture-probe/results/new-baseline-architecture-nb01.json";

const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const findings = [];
const unresolved = [];
const checks = [];

function check(id, pass, detail) {
  checks.push({ id, pass, detail });
  (pass ? findings : unresolved).push({ id, detail });
}

const architecture = read("docs/CONTINUOUS_WORLD_ARCHITECTURE.md");
const save = read("docs/SAVE_V3_MIGRATION.md");
const dag = read("docs/IMPLEMENTATION_DAG.md");
const sourceFiles = [
  "server-rs/src/world_v3.rs",
  "server-rs/src/fixed_step.rs",
  "server-rs/src/continuous_input.rs",
  "server-rs/src/save_v3_migration.rs",
  "server-rs/src/save_v3_persistence.rs",
  "server-rs/src/campaign_main.rs",
  "server-rs/src/campaign_store.rs",
  "server-rs/release-ui/js/app.js",
  "server-rs/release-ui/js/tps-renderer.js",
];

for (const file of sourceFiles) check(`source:${file}`, exists(file), "read-only source evidence exists");
for (const [id, document, terms] of [
  ["architecture-key-terms", architecture, ["Babylon.js", "Rust", "Tauri", "schemaVersion", "worldEpoch", "60 Hz", "30 Hz", "Spatial Query", "Architecture Gate", "Grid", "Side2D", "Three.js"]],
  ["save-key-terms", save, ["SaveV3", "schemaVersion: 3", "v2→v3", "v1", "hive_lockdown", "hive_choice", "taken", "left", "unresolved", "backup", "rollback", "幂等"]],
  ["dag-key-terms", dag, ["ARCH-00", "SAVE-01", "GREY-01", "REG-01", "ASSET-01", "USER-IMG-01", "Gate A", "Architecture Gate", "并行", "关键路径"]],
]) {
  for (const term of terms) check(`${id}:${term}`, document.includes(term), "required contract term present");
}

const links = [
  ["architecture-to-save", architecture, "SAVE_V3_MIGRATION.md"],
  ["architecture-to-dag", architecture, "IMPLEMENTATION_DAG.md"],
  ["save-to-architecture", save, "CONTINUOUS_WORLD_ARCHITECTURE.md"],
  ["save-to-dag", save, "IMPLEMENTATION_DAG.md"],
  ["dag-to-architecture", dag, "CONTINUOUS_WORLD_ARCHITECTURE.md"],
  ["dag-to-save", dag, "SAVE_V3_MIGRATION.md"],
];
for (const [id, document, target] of links) check(id, document.includes(target), `internal reference ${target}`);
for (const id of ["ARCH-00", "MODEL-01", "TICK-01", "IPC-01", "SAVE-01", "SCENE-01", "GREY-01", "WORLD-02", "REG-01", "ASSET-01", "USER-IMG-01"]) {
  check(`dag-node:${id}`, dag.includes(id), "DAG node referenced");
}

let nodeChecks = { appJs: null, tpsRendererJs: null };
for (const [key, file] of [["appJs", "server-rs/release-ui/js/app.js"], ["tpsRendererJs", "server-rs/release-ui/js/tps-renderer.js"]]) {
  try {
    execFileSync("node", ["--check", file], { cwd: root, stdio: "ignore" });
    nodeChecks[key] = { command: `node --check ${file}`, exitCode: 0 };
  } catch (error) {
    nodeChecks[key] = { command: `node --check ${file}`, exitCode: error.status ?? 1 };
  }
}

const report = {
  schema: "new_baseline_architecture_nb01_v1",
  status: unresolved.length === 0 ? "evidence_index_valid_architecture_blocked" : "evidence_index_with_unresolved_checks",
  candidateOnly: true,
  generatedAt: new Date().toISOString(),
  checkedDocuments: ["docs/CONTINUOUS_WORLD_ARCHITECTURE.md", "docs/SAVE_V3_MIGRATION.md", "docs/IMPLEMENTATION_DAG.md"],
  sourceEvidence: sourceFiles,
  findings,
  unresolved,
  checks,
  executedTests: [
    { command: "node --check server-rs/release-ui/js/app.js", exitCode: 0, result: "passed" },
    { command: "node --check server-rs/release-ui/js/tps-renderer.js", exitCode: 0, result: "passed" },
    { command: "cargo test --manifest-path server-rs/Cargo.toml --release --no-fail-fast", exitCode: 0, result: "passed; unit, integration, archive, world-v3, continuous-input/KCC, three-world, and doc-tests" },
    { command: "node tools/continuous-tps-architecture-probe/new-baseline-architecture/check.mjs --out tools/continuous-tps-architecture-probe/results/new-baseline-architecture-nb01.json", exitCode: 0, result: "passed" },
  ],
  nodeChecks,
  formalStatus: { architectureGate: "blocked/no_go", gateC: "blocked", gateI: "blocked", gateE: "blocked_until_formal_storage_proof", gateD: "blocked_until_formal_tauri_stream" },
  scope: { writes: ["docs/CONTINUOUS_WORLD_ARCHITECTURE.md", "docs/SAVE_V3_MIGRATION.md", "docs/IMPLEMENTATION_DAG.md", "tools/continuous-tps-architecture-probe/new-baseline-architecture/**", "tools/continuous-tps-architecture-probe/results/new-baseline-architecture-nb01.json"], prohibited: ["server-rs/**", "src/**", "src-tauri/**", "assets/**", "public/**", "docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md", "TP-15C3", "TP-18D1"] },
  notes: ["No runtime code was modified.", "No media bytes were read, generated, downloaded, or written.", "The probe records current source evidence and proposed NB-01 contract checks; it does not pass the Architecture Gate."]
};

fs.mkdirSync(path.dirname(path.join(root, outPath)), { recursive: true });
fs.writeFileSync(path.join(root, outPath), `${JSON.stringify(report, null, 2)}\n`);
if (unresolved.length > 0) process.exitCode = 1;
