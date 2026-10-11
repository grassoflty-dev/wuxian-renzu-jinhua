import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-three-world-campaign-save-acceptance');
const planPath = path.join(root, 'docs', 'THREE_WORLD_CAMPAIGN_SAVE_ACCEPTANCE.md');
const matrixPath = path.join(own, 'matrix.json');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-three-world-campaign-save-acceptance-nb44.json');
const plan = fs.readFileSync(planPath, 'utf8');
const matrix = JSON.parse(fs.readFileSync(matrixPath, 'utf8'));
const checks = [];
function check(id, pass, actual, expected) { checks.push({ id, pass, actual, expected }); }
const worlds = ['grey_hive','mist_harbor','clockworks'];
const requiredFields = ['stableId','priority','scope','preconditions','action','expectedAuthoritativeState','expectedPresentation','evidence','automationClass','status','blockingDependencies'];
const forbidden = ['TP-15'+'C3','TP-18'+'D1','TP-01'+'R11','TP-18'+'D2'];
const restoreOrder = ['schema/identity','campaign','target manifest/version','spatial topology','doors/triggers','entities/NPC/enemies','inventory/capability','checkpoint/player transform','presentation snapshot'];

check('matrix_object', matrix.schema === 'three_world_campaign_save_acceptance_matrix_v1' && Array.isArray(matrix.items), matrix.schema, 'schema and items');
check('minimum_count', matrix.items.length >= 24, matrix.items.length, '>=24');
check('negative_count', matrix.items.filter((x) => x.priority === 'negative').length >= 10, matrix.items.filter((x) => x.priority === 'negative').length, '>=10');
check('unique_ids', new Set(matrix.items.map((x) => x.stableId)).size === matrix.items.length, matrix.items.length, 'unique stable IDs');
check('required_fields', matrix.items.every((x) => requiredFields.every((f) => Object.prototype.hasOwnProperty.call(x, f))), 'all matrix fields present', true);
check('world_coverage', worlds.every((w) => matrix.items.some((x) => String(x.scope).includes(w))), worlds, 'all worlds covered');
check('normal_failure_reload_incompat', worlds.every((w) => ['normal','failure','reload','version_incompatible'].every((k) => matrix.items.some((x) => String(x.scope).includes(w) && x.tags?.includes(k)))), 'normal/failure/reload/version cases', true);
check('three_choice_states', ['taken','left','unresolved'].every((s) => plan.includes(s) && matrix.items.some((x) => x.tags?.includes(`choice_${s}`))), 'choice states', true);
check('gate_b_independent', plan.includes('Gate B depends on `hive_lockdown` only') && plan.includes('Gate B') && plan.includes('never hidden Gate B key') && matrix.items.some((x) => x.tags?.includes('gate_b_choice_independence')), 'Gate B/choice/NPC independence', true);
check('restore_order', restoreOrder.every((x) => plan.includes(x)), restoreOrder, 'complete restore order');
check('migration_one_way', plan.includes('`v2 → v3`') && plan.includes('`v3 → v2`') && plan.includes('E_MIGRATION_REPEATED') && matrix.items.some((x) => x.tags?.includes('migration_v2_v3')), 'one-way migration/idempotence', true);
check('cross_world_transition', matrix.items.filter((x) => x.tags?.includes('cross_world_transition')).length >= 3, 'transition cases', '>=3');
check('layers_leakage', ['Campaign-persistent','Per-world persistent','Session/transient','must not leak'].every((x) => plan.includes(x)), 'persistence layers', true);
check('authority_boundary', ['Rust is the sole writer','DOM state','JS caches','Grid/tile/','immutable snapshots','versioned commands/events'].every((x) => plan.includes(x)), 'authority boundary', true);
check('failure_codes', ['E_SCHEMA_UNKNOWN','E_ASSET_HASH','E_REFERENCE_SCOPE','E_TRANSITION_PARTIAL','E_AUTHORITY_DIVERGENCE','E_GRID_REACHABLE','E_RUNTIME_UNWIRED'].every((x) => plan.includes(x)), 'failure code families', true);
check('prepare_commit_ack', ['prepare/commit/ack','before commit','commit-before-ack','idempotent'].every((x) => plan.includes(x)), 'transition recovery', true);
check('dependencies', ['F1','F2','F3','F4','F5','F6','F7','blocked'].every((x) => plan.includes(x)), 'F1-F7 dependency graph', true);
check('pending_decisions', ['canonical cross-world return-station','global','health/status'].every((x) => plan.includes(x)), 'three pending product decisions', true);
check('blockers_open', plan.includes('E_NO_FULL_SAVE_V3_RUNTIME_RESTORE') && plan.includes('entire graph') && plan.includes('`blocked`'), 'blockers remain open', true);
check('no_forbidden_identifiers', !forbidden.some((x) => plan.includes(x) || JSON.stringify(matrix).includes(x)), 'forbidden identifiers absent', true);
check('allowlist_only', plan.includes('four') && plan.includes('NB-44 allowlisted files') && matrixPath.startsWith(own), 'four-file allowlist', true);

const passed = checks.filter((x) => x.pass).length;
const negativeCount = matrix.items.filter((x) => x.priority === 'negative').length;
const payload = {
  schema: 'new_baseline_three_world_campaign_save_acceptance_nb44_v1',
  status: passed === checks.length ? 'three_world_campaign_save_acceptance_ready_gate_blocked' : 'invalid',
  conclusion: passed === checks.length ? 'three_world_campaign_save_acceptance_ready_gate_blocked' : 'invalid',
  formalRestoreAccepted: false,
  runtimeExecuted: false,
  saveWritten: false,
  migrationExecuted: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  worlds,
  matrix: { total: matrix.items.length, negative: negativeCount, worldCoverage: worlds.reduce((o, w) => (o[w] = matrix.items.filter((x) => String(x.scope).includes(w)).length, o), {}), crossWorldTransitions: matrix.items.filter((x) => x.tags?.includes('cross_world_transition')).length },
  restoreOrder,
  blocker: 'E_NO_FULL_SAVE_V3_RUNTIME_RESTORE remains open; formal runtime is not wired',
  pendingProductDecisions: ['canonical cross-world return-station transition', 'global campaign facts on revisit', 'health/status normalization across transition'],
  checks,
  summary: { total: checks.length, passed, failed: checks.length - passed },
  rollback: 'delete only the four NB-44 allowlisted files; no formal source, Save, asset, CI or media state changed'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: checks.length, passed, failed: checks.length - passed, matrixItems: matrix.items.length, negativeCount, resultPath }, null, 2));
if (payload.status === 'invalid') process.exitCode = 1;
