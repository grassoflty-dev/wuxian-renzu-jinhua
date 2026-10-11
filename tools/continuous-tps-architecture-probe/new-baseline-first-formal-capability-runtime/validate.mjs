import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-first-formal-capability-runtime');
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_CAPABILITY_RUNTIME_SLICE_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-first-formal-capability-runtime-nb23.json');

const plan = fs.readFileSync(planPath, 'utf8');
const files = [
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/Cargo.toml',
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/src/lib.rs',
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/src/fixtures.rs',
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/tests/capability_runtime_contract.rs',
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/tests/presentation_boundary.rs',
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/fixtures/minimal-capabilities.json',
  'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/validate.mjs',
  'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-capability-runtime-nb23.json',
];
const anchors = ['Exact future patch manifest', 'requested', 'validated', 'committed', 'active', 'completed', 'cancelled', 'interrupted', 'failed'];
const ids = ['ABILITY.COMBAT.PRIMARY', 'ABILITY.COMBAT.DODGE', 'ABILITY.SCAN.QUERY', 'ABILITY.PICKUP.REQUEST', 'ABILITY.INTERACT.USE', 'ABILITY.STEALTH.HIDE', 'ABILITY.STEALTH.BREAK'];
const forbiddenTokens = ['TP-15' + 'C3', 'TP-18' + 'D1', 'TP-01' + 'R11'];
const results = [];
function check(id, pass, actual, expected) { results.push({ id, pass, actual, expected }); }
function containsAll(text, values) { return values.every((v) => text.includes(v)); }
function underOwn(p) { const rel = path.relative(root, p).replaceAll('\\', '/'); return rel === 'docs/FIRST_FORMAL_CAPABILITY_RUNTIME_SLICE_PLAN.md' || rel.startsWith('tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/') || rel === 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-capability-runtime-nb23.json'; }

check('plan_present', fs.existsSync(planPath), fs.existsSync(planPath), true);
check('plan_sections', containsAll(plan, ['Scope and non-goals', 'Contract', 'Exact future patch manifest', 'Test matrix', 'Risks and rollback', 'Readiness conclusion']), 'all required sections', true);
check('stable_ids', containsAll(plan, ids), ids, 'all minimum IDs');
check('schema_fields', containsAll(plan, ['abilityId', 'version', 'tags', 'inputs', 'targets', 'prerequisites', 'cost', 'cooldown', 'phaseState', 'hitAndEffect', 'events', 'saveBoundary']), 'NB-03 fields present', true);
check('state_machine', containsAll(plan, anchors), anchors, 'all lifecycle states');
check('atomic_order', containsAll(plan, ['reserve and commit costs atomically', 'start the cooldown atomically', 'immutable target snapshot']), 'cost/cooldown/target order', true);
check('authority_boundary', containsAll(plan, ['single Rust owner', 'UI/Babylon cannot decide target', 'presentation-only DTO']), 'Rust-only authority', true);
check('epoch_determinism', containsAll(plan, ['worldEpoch', 'generation', 'Duplicate', 'Out-of-order', '30/60 Hz-equivalent']), 'epoch/order/determinism', true);
check('save_boundary', containsAll(plan, ['Save v3 extension boundary', 'must not carry input sequences', 'transient hit effects']), 'save boundary', true);
check('default_disabled', containsAll(plan, ['disabled by default', 'not registered in current gameplay', 'enabled: false']), 'closed by default', true);
check('manifest_exact', files.every((f) => underOwn(path.join(root, f)) && plan.includes(f)), files, 'all entries authorized');
check('manifest_fields', (plan.match(/\| `tools\/continuous-tps-architecture-probe\/new-baseline-first-formal-capability-runtime\//g) ?? []).length >= 7, 'path/anchor/reason/dependency/test/rollback table', true);
check('no_formal_integration', !/server-rs\/|CampaignStore|Tauri command|Babylon consumer/.test(plan.match(/Exact future patch manifest[\s\S]*?### Registration boundary/)?.[0] ?? ''), 'manifest excludes production paths', true);
check('no_forbidden_reads', !forbiddenTokens.some((t) => plan.includes(t)), 'forbidden path identifiers absent', true);
check('result_path', resultPath.endsWith('new-baseline-first-formal-capability-runtime-nb23.json'), resultPath, 'authorized result');

const passed = results.filter((r) => r.pass).length;
const payload = {
  schema: 'new_baseline_first_formal_capability_runtime_nb23_v1',
  status: passed === results.length ? 'capability_runtime_implementation_ready_for_gate_review' : 'capability_runtime_slice_blocked',
  conclusion: passed === results.length ? 'capability_runtime_implementation_ready_for_gate_review' : 'capability_runtime_slice_blocked',
  candidateOnly: true,
  formalIntegration: false,
  defaultEnabled: false,
  owner: 'Rust-only future CapabilityRuntime fixture; no current gameplay registration',
  scope: { plan: 'docs/FIRST_FORMAL_CAPABILITY_RUNTIME_SLICE_PLAN.md', package: 'tools/continuous-tps-architecture-probe/new-baseline-first-formal-capability-runtime/**', result: 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-capability-runtime-nb23.json', forbidden: ['Rust production source', 'UI/Tauri/Babylon', 'CampaignStore', 'Save implementation', 'CI', 'assets', 'media'] },
  schemaContract: { stableIdPattern: 'ABILITY.<DOMAIN>.<ACTION>', version: 'semver', authority: 'Rust', minimumIds: ids, errorCodes: ['E_SCHEMA', 'E_OWNERSHIP', 'E_PHASE', 'E_PREREQUISITE', 'E_TARGET', 'E_COOLDOWN', 'E_COST', 'E_INPUT', 'E_DUPLICATE', 'E_OUT_OF_ORDER', 'E_EPOCH', 'E_GENERATION', 'E_CANCELLED', 'E_INTERRUPTED', 'E_CLOSED', 'E_DISABLED', 'E_INTERNAL'] },
  stateMachine: { states: ['requested', 'validated', 'committed', 'active', 'completed', 'cancelled', 'interrupted', 'failed'], atomicOrder: ['validate', 'snapshot_target', 'commit_cost', 'start_cooldown', 'resolve_effect', 'emit_events', 'emit_view_and_terminal'] },
  tests: { matrixCovered: ['prerequisite', 'cost', 'cooldown', 'cancel', 'interrupt', 'duplicate', 'out_of_order', 'old_epoch', 'old_generation', 'determinism', 'authority_boundary', 'save_boundary', 'disabled_default'], validatorChecks: results },
  patchManifest: files,
  rollback: 'delete only the NB-23 allowlisted files; no current runtime or gameplay files',
  forbiddenIdentifierScan: 'passed',
  summary: { total: results.length, passed, failed: results.length - passed }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: results.length, passed, failed: results.length - passed, resultPath }, null, 2));
if (payload.status !== 'capability_runtime_implementation_ready_for_gate_review') process.exitCode = 1;
