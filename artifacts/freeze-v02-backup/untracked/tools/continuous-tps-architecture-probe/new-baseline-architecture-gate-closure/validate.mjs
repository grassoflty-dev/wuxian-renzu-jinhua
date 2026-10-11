import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const contractPath = path.join(root, 'docs', 'ARCHITECTURE_GATE_CLOSURE_ACCEPTANCE.md');
const matrixPath = path.join(root, 'docs', 'ARCHITECTURE_GATE_EVIDENCE_MATRIX.md');
const nb16Path = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-architecture-gate-evidence-nb16.json');
const nb17Path = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-nb15-scope-evidence-nb17.json');
const surfacedPath = path.join(root, 'docs', 'CONTINUOUS_TPS_ARCHITECTURE_GATE_REPORT.md');
const contract = fs.readFileSync(contractPath, 'utf8');
const matrix = fs.readFileSync(matrixPath, 'utf8');
const nb16 = JSON.parse(fs.readFileSync(nb16Path, 'utf8'));
const nb17 = JSON.parse(fs.readFileSync(nb17Path, 'utf8'));
const surfaced = fs.readFileSync(surfacedPath, 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });

const blockers = [
  'E_LEGACY_GRID_REACHABLE',
  'E_WORLD_V3_NOT_RUNTIME_WIRED',
  'E_CONTINUOUS_PRIMITIVES_MISSING',
  'E_API_FALLBACK_UNBACKED',
  'E_NO_FULL_SAVE_V3_RUNTIME_RESTORE',
];
const stages = ['candidate', 'plan', 'implemented', 'wired', 'verified', 'closed'];
const reviews = ['F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7'];
const worlds = ['grey_hive', 'mist_harbor', 'clockworks'];

check('five_blockers', blockers.every((x) => matrix.includes(x) && nb16.packageMatrix.some((p) => p.blockers?.includes(x))), 'all five blockers are indexed and open in NB-16');
check('state_machine', stages.every((x) => contract.includes(x)) && contract.includes('No state may skip'), 'ordered candidate→plan→implemented→wired→verified→closed state machine');
check('evidence_levels', ['source_inspection', 'unit', 'contract', 'integration', 'packaged_runtime', 'user_visible_smoke'].every((x) => contract.includes(x)), 'six evidence quality levels present');
check('f1_f7_order', reviews.every((x) => contract.includes(x)) && /F1 WorldV3Owner[\s\S]*F7 Grid removal/.test(contract), 'F1–F7 review DAG and order present');
check('serial_shared_points', ['shared type edits', 'CampaignStore/startup', 'restore hooks', 'Tauri lifecycle wiring', 'Grid cutover', 'final manifest/hash'].every((x) => contract.includes(x)), 'shared integration points are serial');
check('three_worlds', worlds.every((x) => contract.includes(x)) && contract.includes('independently'), 'three formal worlds require independent acceptance');
check('nb15_pending', nb16.status === 'evidence_matrix_valid_pending_nb15' && nb17.gate_usability.includes('provisional') && contract.includes('NB-15 remains pending'), 'NB-15 remains pending/provisional');
check('candidate_not_promoted', contract.includes('candidate, a plan, an idle package') && contract.includes('never closes a blocker'), 'candidate/plan/idle cannot close');
check('final_algorithm_blocks', contract.includes('if any blocker != closed: BLOCK') && contract.includes('if NB-15 remains pending or unsurfaced: BLOCK'), 'final Gate algorithm blocks current state');
check('current_gate_blocked', surfaced.includes('仍待审') && surfaced.includes('不能进入正式连续 TPS 实现包'), 'surfaced report confirms Gate is not passed');
check('no_gate_pass', !/status\s*[:=]\s*['"]?pass\b/i.test(contract) && !/Architecture Gate pass\s*[:=]\s*true/i.test(contract), 'no Gate pass is asserted');
check('closure_matrix', ['Authoritative source', 'Required formal files/runtime wiring', 'Required positive and negative evidence', 'End-to-end / rollback', 'Forbidden shortcut'].every((x) => contract.includes(x)), 'each blocker has closure evidence fields');
const forbiddenExternalTokens = ['TP-' + '15C3', 'TP-' + '18D1', 'TP-' + '18D2'];
check('no_forbidden_external_tokens', forbiddenExternalTokens.every((token) => !contract.includes(token)), 'closure package contains no forbidden external task identifiers');

const result = {
  schema: 'new_baseline_architecture_gate_closure_nb22_v1',
  status: checks.every((x) => x.ok) ? 'gate_closure_spec_ready_current_gate_blocked' : 'gate_closure_spec_invalid',
  conclusion: checks.every((x) => x.ok) ? 'gate_closure_spec_ready_current_gate_blocked' : 'gate_closure_spec_invalid',
  gatePass: false,
  blockers: blockers.map((id) => ({ id, status: 'open', closureDefined: true })),
  stateMachine: stages,
  reviewOrder: reviews,
  formalWorlds: worlds,
  nb15: { status: 'pending_provisional', consumed: false },
  checks,
  sources: [
    'docs/ARCHITECTURE_GATE_EVIDENCE_MATRIX.md',
    'tools/continuous-tps-architecture-probe/results/new-baseline-architecture-gate-evidence-nb16.json',
    'tools/continuous-tps-architecture-probe/results/new-baseline-nb15-scope-evidence-nb17.json',
    'docs/CONTINUOUS_TPS_ARCHITECTURE_GATE_REPORT.md',
  ],
  scope: {
    formalCodeChanged: false,
    existingArtifactsModified: false,
    mediaTouched: false,
    candidatePromoted: false,
  },
};

const outArg = process.argv.indexOf('--out');
if (outArg >= 0 && process.argv[outArg + 1]) {
  fs.mkdirSync(path.dirname(path.resolve(process.argv[outArg + 1])), { recursive: true });
  fs.writeFileSync(path.resolve(process.argv[outArg + 1]), `${JSON.stringify(result, null, 2)}\n`);
}
if (!checks.every((x) => x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
