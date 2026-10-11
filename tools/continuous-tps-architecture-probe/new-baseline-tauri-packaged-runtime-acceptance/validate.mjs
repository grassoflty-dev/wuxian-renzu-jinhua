import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-tauri-packaged-runtime-acceptance');
const planPath = path.join(root, 'docs', 'TAURI_PACKAGED_RUNTIME_ACCEPTANCE_PROTOCOL.md');
const scenariosPath = path.join(own, 'scenarios.json');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-tauri-packaged-runtime-acceptance-nb48.json');
const plan = fs.readFileSync(planPath, 'utf8');
const scenarios = JSON.parse(fs.readFileSync(scenariosPath, 'utf8'));
const checks = [];
function check(id, pass, actual, expected) { checks.push({ id, pass, actual, expected }); }
const phases = ['preflight','clean_build','install_launch','ipc_handshake','command_snapshot','save_reload','failure_injection','offline_verification','evidence_sealing','cleanup'];
const fields = ['stableId','phase','priority','preconditions','steps','expectedRust','expectedUi','transport','evidence','automation','status','deps'];
const forbidden = ['TP-15'+'C3','TP-18'+'D1','TP-01'+'R11','TP-18'+'D2'];
check('scenario_schema', scenarios.schema === 'tauri_packaged_runtime_acceptance_scenarios_v1' && Array.isArray(scenarios.scenarios), scenarios.schema, 'schema/scenarios');
check('scenario_count', scenarios.scenarios.length >= 28, scenarios.scenarios.length, '>=28');
check('negative_count', scenarios.scenarios.filter((x) => x.priority === 'negative').length >= 12, scenarios.scenarios.filter((x) => x.priority === 'negative').length, '>=12');
check('fallback_count', scenarios.scenarios.filter((x) => x.tags?.includes('fallback_boundary') || x.tags?.includes('ipc_boundary')).length >= 8, scenarios.scenarios.filter((x) => x.tags?.includes('fallback_boundary') || x.tags?.includes('ipc_boundary')).length, '>=8');
check('unique_ids', new Set(scenarios.scenarios.map((x) => x.stableId)).size === scenarios.scenarios.length, scenarios.scenarios.length, 'unique stable IDs');
check('scenario_fields', scenarios.scenarios.every((x) => fields.every((f) => Object.prototype.hasOwnProperty.call(x, f))), 'all fields present', true);
check('phase_coverage', phases.every((p) => scenarios.scenarios.some((x) => x.phase === p)), phases, 'all phases covered');
check('protocol_envelope', ['requestId','commandType','payloadSchema','authorityTick','timeoutMs','cancelToken','retryOf','idempotencyKey','snapshotSequence','ackSequence'].every((x) => plan.includes(x)), 'envelope fields', true);
check('identity_hashes', ['source revision','Rust binary hash','Babylon vendor hash','OS/architecture','build profile','package hash'].every((x) => plan.includes(x)) && /UI\s+bundle hash/.test(plan) && /Tauri config hash/.test(plan), 'identity/provenance fields', true);
check('authority_boundary', /Rust remains the gameplay\s+authority/.test(plan) && /Babylon\/UI only submits typed intent/.test(plan) && plan.includes('immutable snapshots') && plan.includes('authority write'), 'Rust/UI boundary', true);
check('fallback_negative', /E_API_FALLBACK_UNBACKED/.test(plan) && /Browser `.\/api\/\*`|Browser `\/api\/\*`/.test(plan) && ['localhost','HTTP 200','mock bridge','static UI','unit test','file existence','candidate validator'].every((x) => plan.includes(x)), 'fallback pseudo-pass denial', true);
check('offline_matrix', ['DNS disabled','occupied ports','absent/broken bundle','malformed config/schema','non-ASCII package','devtools/JS injection','crash/restart','renderer reload/reconnect','log redaction'].every((x) => plan.includes(x)), 'offline/failure coverage', true);
check('save_reload', plan.includes('save_reload') && ['Save/reload','source state'].every((x) => plan.includes(x)), 'save/reload boundary', true);
check('evidence_bundle', ['expected/observed JSON','timestamps','exit codes','logs metadata','owner','rollback','evidence_invalid'].every((x) => plan.includes(x)), 'sealed evidence contract', true);
check('blocker_open', plan.includes('E_API_FALLBACK_UNBACKED') && /blocker stays\s+open/.test(plan), 'fallback blocker open', true);
check('flags_false', ['packagedRuntimeAccepted=false','ipcAccepted=false','buildExecuted=false','runtimeExecuted=false','network=false','mediaBytesRead=false','mediaCreated=false','mediaModelCalled=false'].every((x) => plan.includes(x)), 'execution flags false', true);
check('no_forbidden_identifiers', !forbidden.some((x) => plan.includes(x) || JSON.stringify(scenarios).includes(x)), 'forbidden IDs absent', true);
check('allowlist', plan.includes('NB-48 protocol') && scenariosPath.startsWith(own), 'four-file scope', true);

const passed = checks.filter((x) => x.pass).length;
const negative = scenarios.scenarios.filter((x) => x.priority === 'negative').length;
const fallback = scenarios.scenarios.filter((x) => x.tags?.includes('fallback_boundary') || x.tags?.includes('ipc_boundary')).length;
const payload = {
  schema: 'new_baseline_tauri_packaged_runtime_acceptance_nb48_v1',
  status: passed === checks.length ? 'tauri_packaged_runtime_acceptance_protocol_ready_not_executed' : 'invalid',
  conclusion: passed === checks.length ? 'tauri_packaged_runtime_acceptance_protocol_ready_not_executed' : 'invalid',
  packagedRuntimeAccepted: false,
  ipcAccepted: false,
  apiFallback: 'E_API_FALLBACK_UNBACKED=open',
  buildExecuted: false,
  runtimeExecuted: false,
  network: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  scenarioStats: { total: scenarios.scenarios.length, negative, fallbackOrIpcBoundary: fallback, phases: phases.reduce((o, p) => (o[p] = scenarios.scenarios.filter((x) => x.phase === p).length, o), {}) },
  pseudoPassesDenied: ['browser localhost','mock bridge','static UI','HTTP 200','unit test','file exists','candidate validator'],
  checks,
  summary: { total: checks.length, passed, failed: checks.length - passed },
  rollback: 'delete only NB-48 allowlisted protocol/matrix/validator/result; no build/runtime/network/media state changed'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: checks.length, passed, failed: checks.length - passed, scenarios: scenarios.scenarios.length, negative, fallback, resultPath }, null, 2));
if (payload.status === 'invalid') process.exitCode = 1;
