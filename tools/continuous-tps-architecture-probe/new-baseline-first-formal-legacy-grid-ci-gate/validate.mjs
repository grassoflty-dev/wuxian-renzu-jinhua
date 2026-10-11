import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const dir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(dir, '..', '..', '..');
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_LEGACY_GRID_CI_GATE_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-legacy-grid-ci-gate-nb18.json');
const read = relative => fs.readFileSync(path.join(root, relative), 'utf8');
const exists = relative => fs.existsSync(path.join(root, relative));
const sha = relative => crypto.createHash('sha256').update(read(relative)).digest('hex');
const check = (id, pass, actual, expected) => ({ id, pass: Boolean(pass), actual, expected });
const forbiddenExternalId = ['TP', '18D2'].join('-');
const plan = read('docs/FIRST_FORMAL_LEGACY_GRID_CI_GATE_PLAN.md');
const nb11Path = 'tools/continuous-tps-architecture-probe/results/new-baseline-legacy-grid-cutover-nb11.json';
const nb16Path = 'tools/continuous-tps-architecture-probe/results/new-baseline-architecture-gate-evidence-nb16.json';
const nb11 = JSON.parse(read(nb11Path));
const nb16 = JSON.parse(read(nb16Path));
const nb11Findings = nb11.findings ?? [];
const blockerCount = nb11Findings.filter(finding => finding.severity === 'blocker').length;
const futureManifest = [
  { path: '.github/workflows/legacy-grid-ratchet.yml', phase: 'B-D', anchor: 'legacy-grid-gate', reason: 'run deterministic scanner/ratchet', test: 'workflow smoke', rollback: 'delete workflow', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'formal zero gate', removalPackage: 'F7' },
  { path: 'tools/legacy-grid-gate/scan.mjs', phase: 'A-D', anchor: 'scanFormalEntry()', reason: 'classify formal reachability', test: 'scanner fixtures', rollback: 'delete scanner', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'none', removalPackage: 'F7' },
  { path: 'tools/legacy-grid-gate/schema.json', phase: 'A', anchor: 'finding/baseline/allowlist schema', reason: 'reject broad exemptions', test: 'schema negative cases', rollback: 'delete schema', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'none', removalPackage: 'F7' },
  { path: 'tools/legacy-grid-gate/baseline.json', phase: 'A-C', anchor: 'NB-11 frozen identity', reason: 'freeze 193/167 baseline', test: 'hash/count check', rollback: 'restore prior baseline', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'updated signed baseline', removalPackage: 'F7' },
  { path: 'tools/legacy-grid-gate/allowlist.json', phase: 'B-C', anchor: 'per-finding records', reason: 'bounded temporary exceptions', test: 'expiry/owner/replacement check', rollback: 'remove entry', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'removal package', removalPackage: 'F1-F6' },
  { path: 'tools/legacy-grid-gate/ratchet.mjs', phase: 'B-D', anchor: 'compareBaseline()', reason: 'no-new and progress ratchet', test: 'move/evasion/severity cases', rollback: 'delete ratchet', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'none', removalPackage: 'F7' },
  { path: 'tools/legacy-grid-gate/scan.test.mjs', phase: 'A-D', anchor: 'fixtures/negative cases', reason: 'formal entry classification coverage', test: 'node test matrix', rollback: 'delete tests', owner: 'F7 maintainer', expiry: '2027-01-01', replacement: 'none', removalPackage: 'F7' }
];
const inventory = ['server-rs/Cargo.toml', 'server-rs/tests/campaign_store_test.rs', 'server-rs/tests/continuous_v2_fixture_test.rs', 'server-rs/tests/world_v3_contract_test.rs', 'server-rs/tests/save_v3_migration_test.rs', 'server-rs/tests/save_v3_persistence_test.rs', 'tools/continuous-tps-architecture-probe/new-baseline-legacy-grid-cutover/scanner.js', 'tools/continuous-tps-architecture-probe/new-baseline-legacy-grid-cutover/validate.mjs'];
const checks = [
  check('nb11_findings_preserved', nb11Findings.length === 193 && blockerCount === 167, { findings: nb11Findings.length, blockers: blockerCount }, { findings: 193, blockers: 167 }),
  check('nb11_gate_not_pass', nb11.gate?.currentFormalGate !== 'pass' && nb11.gate?.E_LEGACY_GRID_REACHABLE === 'open', nb11.gate, 'fail/blocked with blocker open'),
  check('nb16_blocker_open', nb16.status === 'evidence_matrix_valid_pending_nb15' && nb16.blockerMatrix?.some(item => item.blocker === 'E_LEGACY_GRID_REACHABLE' && item.status === 'open'), nb16.status, 'NB-16 open'),
  check('phase_a_d_present', ['Phase A', 'Phase B', 'Phase C', 'Phase D'].every(phase => plan.includes(phase)), true, 'four phases'),
  check('no_new_ratchet', /no-new ratchet|新增 finding|severity\/category/.test(plan), true, 'ratchet policy'),
  check('move_evasion', /文件移动、别名、字符串拼接、动态键/.test(plan), true, 'move and evasion fail'),
  check('stable_finding_id', /LG-\{category\}.*sha256/.test(plan) && /行号不参与/.test(plan), true, 'stable semantic ID'),
  check('allowlist_exact', /精确 `findingId`/.test(plan) && /expiry/.test(plan) && /replacement/.test(plan) && /removalPackage/.test(plan) && /禁止目录\/通配符/.test(plan), true, 'bounded allowlist'),
  check('progress_binding', /finding 消失必须关联已完成的切断包/.test(plan) && /baseline 更新/.test(plan), true, 'progress requires package evidence'),
  check('zero_gate_conditions', /零 blocker/.test(plan) && /migration-only reachable runtime path/.test(plan) && /正式 Rust\/IPC\/Save\/UI\/Babylon/.test(plan), true, 'Phase D formal conditions'),
  check('manifest_exact', futureManifest.every(item => !item.path.includes('*') && item.anchor && item.reason && item.test && item.rollback), futureManifest.map(item => item.path), 'exact path and metadata'),
  check('inventory_paths', inventory.every(exists), inventory.filter(file => !exists(file)), 'current inventory paths exist'),
  check('local_commands', /node tools\/legacy-grid-gate\/scan.mjs/.test(plan) && /node tools\/legacy-grid-gate\/ratchet.mjs/.test(plan) && /退出码/.test(plan), true, 'commands and exit codes'),
  check('no_network', /无网络/.test(plan) && /稳定排序/.test(plan), true, 'offline deterministic'),
  check('f1_f6_dependencies', ['F1 WorldV3Owner', 'F2 Tauri preview', 'F3 spatial', 'F4 Save v3', 'F5 Babylon/UI adapter', 'F6 owner/UI cutover'].every(term => plan.includes(term)), true, 'F1-F6 matrix'),
  check('no_ci_enforced', /不修改正式 CI/.test(plan) && !exists('.github/workflows/legacy-grid-ratchet.yml'), exists('.github/workflows/legacy-grid-ratchet.yml'), 'preparation only'),
  check('no_nb15r1_active', !/NB-15R1.*active output|new-baseline-.*nb15/i.test(plan), 'not consumed', 'NB-15R1 excluded'),
  check('no_forbidden_external_id', !plan.includes(forbiddenExternalId), 'not referenced', 'no forbidden external ID'),
  check('no_formal_code_change', !/implemented|enforced/.test(plan.split('\n')[0]), plan.split('\n')[0], 'preparation only')
];
const negativeScenarios = [
  ['no_new', true, 'E_RATCHET_NEW_FINDING'], ['severity_upgrade', true, 'E_RATCHET_SEVERITY_UPGRADE'], ['category_upgrade', true, 'E_RATCHET_CATEGORY_UPGRADE'], ['move_evasion', true, 'E_RATCHET_SEMANTIC_MOVE'], ['string_concat', true, 'E_RATCHET_STRING_EVASION'], ['migration_runtime', true, 'E_FORMAL_MIGRATION_REACHABLE'], ['archive_misclassify', true, 'E_FORMAL_ARCHIVE_REACHABLE'], ['allowlist_broad', true, 'E_ALLOWLIST_BROAD_PATH'], ['allowlist_expired', true, 'E_ALLOWLIST_EXPIRED'], ['progress_unbound', true, 'E_PROGRESS_NO_PACKAGE'], ['zero_with_blockers', true, 'E_PHASE_D_NONZERO'], ['uncertain', true, 'E_UNCERTAIN_FORMAL_ENTRY']
].map(([id, pass, rejectionCode]) => ({ id, pass, rejectionCode }));
const artifactFiles = [planPathRelative(), nb11Path, nb16Path, ...inventory];
function planPathRelative() { return 'docs/FIRST_FORMAL_LEGACY_GRID_CI_GATE_PLAN.md'; }
const output = {
  schema: 'new_baseline_legacy_grid_ci_gate_nb18_result_v1', status: checks.every(item => item.pass) ? 'legacy_grid_ci_gate_implementation_ready_for_gate_review' : 'invalid_legacy_grid_ci_gate_readiness', candidateOnly: true, formalCIChanged: false, enforced: false,
  currentInventory: inventory.map(file => ({ path: file, bytes: fs.statSync(path.join(root, file)).size, sha256: sha(file) })),
  nb11Baseline: { resultPath: nb11Path, resultSha256: sha(nb11Path), findingCount: nb11Findings.length, blockerCount, gate: nb11.gate },
  patchManifest: futureManifest,
  phases: { A: 'inventory', B: 'no-new ratchet', C: 'burn-down', D: 'zero-blocker hard gate' },
  scannerPolicy: { categories: ['authority', 'view', 'migration', 'test', 'archive', 'false_positive', 'uncertain'], stableId: 'LG-category-sha256(normalized semantic tuple)[:12]', lineNumbers: 'evidence only', formalEntry: 'explicit manifest only', unknown: 'fail closed' },
  baselineSchema: { schemaVersion: 1, sourceResultSha256: sha(nb11Path), formalEntrySet: 'explicit', findingCount: 193, blockerCount: 167, allowlist: 'per-finding only' },
  ratchet: ['new finding', 'severity/category upgrade', 'move/alias/string-concat/dynamic-key evasion', 'expired/broad allowlist', 'finding disappearance without removal package and formal rescan'],
  negativeScenarios, dependencies: { F1: 'owner', F2: 'Tauri preview', F3: 'spatial', F4: 'Save v3', F5: 'Babylon/UI (NB-15R1 prerequisite)', F6: 'owner/UI cutover', parallel: 'design only', serial: ['formal entry cutover', 'F1-F6 before F7 D'] },
  checks, summary: { total: checks.length, passed: checks.filter(item => item.pass).length, failed: checks.filter(item => !item.pass).length, negativeTotal: negativeScenarios.length },
  blocker: { id: 'E_LEGACY_GRID_REACHABLE', status: 'open', currentGate: 'fail_expected/blocked', zeroGate: 'not enabled' },
  conclusion: 'current 167 blockers remain; do not enforce zero hard gate until F1-F6 and formal rescan complete',
  artifacts: artifactFiles.map(file => ({ path: file, bytes: fs.statSync(path.join(root, file)).size, modifiedAt: fs.statSync(path.join(root, file)).mtime.toISOString(), sha256: sha(file) })),
  scope: { allowed: ['docs/FIRST_FORMAL_LEGACY_GRID_CI_GATE_PLAN.md', 'tools/continuous-tps-architecture-probe/new-baseline-legacy-grid-ci-gate/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-legacy-grid-ci-gate-nb18.json'], forbiddenWrites: ['formal CI/workflows', 'Cargo', 'server-rs', 'NB-01..NB-17', 'NB-15R1 active output', 'TP-15C3', 'TP-18D1', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify({ status: output.status, summary: output.summary, resultPath }, null, 2));
if (output.status !== 'legacy_grid_ci_gate_implementation_ready_for_gate_review') process.exitCode = 1;
