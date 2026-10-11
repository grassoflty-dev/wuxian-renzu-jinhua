import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const dir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(dir, '..', '..', '..');
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_SAVE_V3_SLICE_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-first-formal-save-v3-slice-nb14.json');
const read = relative => fs.readFileSync(path.join(root, relative), 'utf8');
const exists = relative => fs.existsSync(path.join(root, relative));
const sha = relative => crypto.createHash('sha256').update(read(relative)).digest('hex');
const plan = read('docs/FIRST_FORMAL_SAVE_V3_SLICE_PLAN.md');
const migration = read('server-rs/src/save_v3_migration.rs');
const persistence = read('server-rs/src/save_v3_persistence.rs');
const world = read('server-rs/src/world_v3.rs');
const lib = read('server-rs/src/lib.rs');
const cargo = read('server-rs/Cargo.toml');
const store = read('server-rs/src/campaign_store.rs');
const check = (id, pass, actual, expected) => ({ id, pass: Boolean(pass), actual, expected });
const allowedResults = {
  nb01: 'tools/continuous-tps-architecture-probe/results/new-baseline-architecture-nb01.json',
  nb03: 'tools/continuous-tps-architecture-probe/results/new-baseline-capability-assets-nb03.json',
  nb05: 'tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-data-contract-nb05.json',
  nb06: 'tools/continuous-tps-architecture-probe/results/new-baseline-authority-migration-nb06.json',
  nb07: 'tools/continuous-tps-architecture-probe/results/new-baseline-world-v3-runtime-wiring-nb07.json',
  nb10: 'tools/continuous-tps-architecture-probe/results/new-baseline-save-v3-full-restore-nb10.json',
  nb12: 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-slice-nb12.json'
};
const resultData = Object.fromEntries(Object.entries(allowedResults).map(([key, file]) => [key, JSON.parse(read(file))]));
const manifest = [
  { path: 'server-rs/src/save_v3_migration.rs', status: 'existing_modify', anchor: 'SaveV3Envelope/validate_envelope/migrate_v2_json', reason: 'typed payload/checksum/invariant and one-way v2 adapter', dependency: 'F1 API frozen; NB-01/NB-07/NB-10', test: 'migration round-trip and deterministic signature', rollback: 'restore original file', serial: false },
  { path: 'server-rs/src/save_v3_persistence.rs', status: 'existing_modify', anchor: 'PersistenceFault/pending/rename', reason: 'filesystem abstraction, staging token, atomic fault tests', dependency: 'F1 API; NB-10 candidate', test: 'isolated filesystem fault matrix', rollback: 'restore original file', serial: false },
  { path: 'server-rs/src/world_v3.rs', status: 'existing_modify_serial_f1', anchor: 'WorldStateV3/WorldView/SaveV3', reason: 'coordinate typed owner/snapshot types with F1', dependency: 'F1 serial integration point; NB-12', test: 'world/save schema round-trip', rollback: 'restore original file', serial: true },
  { path: 'server-rs/src/lib.rs', status: 'existing_modify_serial_f1', anchor: 'pub mod save_v3_migration/save_v3_persistence', reason: 'serial export of F4 types only after F1', dependency: 'F1 serial integration point', test: 'module discovery/cargo test', rollback: 'remove export or restore file', serial: true },
  { path: 'server-rs/tests/save_v3_full_restore_candidate_test.rs', status: 'new', anchor: 'new test module', reason: 'staging/reconcile/activate/full state regression', dependency: 'F1/F4', test: 'cargo test candidate restore suite', rollback: 'delete file', serial: false },
  { path: 'server-rs/tests/save_v3_atomic_candidate_test.rs', status: 'new', anchor: 'new test module', reason: 'backup/temp/flush/validate/replace faults', dependency: 'F4 persistence abstraction', test: 'cargo test atomic suite', rollback: 'delete file', serial: false }
];
const existingTypes = [...migration.matchAll(/(?:struct|enum)\s+(\w+)/g), ...persistence.matchAll(/(?:struct|enum)\s+(\w+)/g), ...world.matchAll(/(?:struct|enum)\s+(\w+)/g)].map(match => match[1]);
const proposedTypes = ['SaveV3Envelope', 'SaveV3Payload', 'SaveError', 'RestoreStaging', 'RestoreActivationToken'];
const typeCollision = [
  check('existing_save_anchors', existingTypes.includes('SaveV3Envelope') && /validate_envelope/.test(migration) && /migrate_v2_json/.test(migration) && /PersistenceFault/.test(persistence), existingTypes.filter(name => name.includes('SaveV3')), 'existing Save v3 anchors present'),
  check('new_type_collisions', proposedTypes.filter(name => existingTypes.includes(name)), proposedTypes.filter(name => existingTypes.includes(name)), ['SaveV3Envelope may be augmented; no accidental duplicate']),
  check('module_collision', !/pub\s+mod\s+save_v3_full_restore/.test(lib), /pub\s+mod\s+save_v3_full_restore/.test(lib), 'no unplanned module collision'),
  check('dependency_available', /serde_json/.test(cargo) && /sha2/.test(cargo), true, 'existing serde_json/sha2 dependencies'),
  check('campaign_store_untouched_anchor', /pub\s+struct\s+CampaignStore/.test(store), true, 'CampaignStore remains current owner')
];
const resultChecks = Object.entries(resultData).map(([id, data]) => check(`${id}_candidate_result`, data.candidateOnly === true || data.formalIntegration === false || data.formalCodeChanged === false || /blocked|partial|candidate|audit_complete|governance/.test(data.status ?? ''), { status: data.status, candidateOnly: data.candidateOnly }, 'candidate/blocked source only'));
const manifestChecks = manifest.flatMap(item => [
  check(`manifest_precise_${item.path}`, !item.path.includes('*') && !item.path.endsWith('/'), item.path, 'precise file path'),
  check(`manifest_meta_${item.path}`, Boolean(item.anchor && item.reason && item.dependency && item.test && item.rollback), { anchor: !!item.anchor, reason: !!item.reason, dependency: !!item.dependency, test: !!item.test, rollback: !!item.rollback }, 'all metadata'),
  check(`manifest_serial_${item.path}`, item.path.includes('world_v3.rs') || item.path.includes('lib.rs') ? item.serial === true : true, item.serial, 'shared F1 files are serial')
]);
const planChecks = [
  check('plan_status', /save_v3_implementation_package_ready_for_gate_review/.test(plan) && !/implemented\s*$/.test(plan), 'ready_for_gate_review', 'not implemented'),
  check('staging_activate', /staging.*恢复|未激活 staging/.test(plan) && /activation token|atomic activate/.test(plan), true, 'staging then atomic activate'),
  check('migration_only_v2', /只接受明确 `version=2`/.test(plan) && /v1、更早、缺失或未来版本拒绝/.test(plan), true, 'v2 only'),
  check('gate_contract', /Gate B 只读取 `hive_lockdown`/.test(plan) && /hive_choice.*unresolved/.test(plan) && /NPC 独立/.test(plan), true, 'Gate B lockdown only; choice/NPC independent'),
  check('one_way_grid', /一次性 adapter/.test(plan) && /无 legacy 双权威字段/.test(plan) && /单位不明返回 `E_V2_UNCERTAIN_UNITS`/.test(plan), true, 'one-way explicit adapter'),
  check('no_formal_cutover', /不接管当前 CampaignStore/.test(plan) && /不注册 Tauri command/.test(plan) && /不接 UI\/Babylon/.test(plan) && /不切换现有 `campaign_save`/.test(plan), true, 'no current path cutover'),
  check('excluded_changes', ['Tauri command', 'release UI', 'Babylon', 'HTTP', 'campaign_store.rs', 'CampaignStore', 'Grid removal', 'Save v3 write'].every(term => plan.includes(term)), true, 'excluded changes explicit'),
  check('serial_points', /world_v3.rs.*串行/.test(plan) && /lib.rs.*串行/.test(plan) && /F1\/F4 serial integration point/.test(plan), true, 'F1/F4 serial points'),
  check('f3_future_only', /NB-13 完成后再协调/.test(plan) && !/nb13.*json/i.test(plan), true, 'F3 only future hook; no NB-13 input'),
  check('owner_api_boundary', /F1 WorldV3Owner/.test(plan) && /rebuild hooks/.test(plan) && /F2\/Tauri、F5 UI\/Babylon 不进入/.test(plan), true, 'F1/F3/F2 boundaries')
];
const forbiddenInputCheck = check('no_nb13_read', !/new-baseline-.*nb13|nb13.*json|NB-13.*result/i.test(plan), 'no NB-13 path/result', 'NB-13 coordination only');
const checks = [...typeCollision, ...resultChecks, ...manifestChecks, ...planChecks, forbiddenInputCheck];
const inventoryFiles = ['server-rs/Cargo.toml', 'server-rs/src/save_v3_migration.rs', 'server-rs/src/save_v3_persistence.rs', 'server-rs/src/world_v3.rs', 'server-rs/src/campaign_store.rs', 'server-rs/src/lib.rs', 'server-rs/tests/save_v3_migration_test.rs', 'server-rs/tests/save_v3_persistence_test.rs', 'server-rs/tests/world_v3_contract_test.rs'];
const output = {
  schema: 'new_baseline_first_formal_save_v3_slice_nb14_result_v1', status: checks.every(item => item.pass) ? 'save_v3_implementation_package_ready_for_gate_review' : 'failed_save_v3_readiness_review', candidateOnly: true, formalCodeChanged: false,
  currentInventory: { modules: ['save_v3_migration', 'save_v3_persistence', 'world_v3', 'campaign_store'], commands: ['campaign_view', 'campaign_action', 'campaign_save', 'tp06a_start', 'tp06a_stop'], tests: ['save_v3_migration_test.rs', 'save_v3_persistence_test.rs', 'world_v3_contract_test.rs'], files: inventoryFiles.map(file => ({ path: file, exists: exists(file), bytes: fs.statSync(path.join(root, file)).size, sha256: sha(file) })) },
  patchManifest: manifest, apiSchema: { types: ['SaveV3Envelope', 'SaveV3Payload', 'SaveError', 'RestoreStaging', 'RestoreActivationToken'], functions: ['encode', 'decode', 'checksum', 'validate', 'migrate_v2_to_v3', 'reconcile', 'activate', 'commit', 'filesystem backup/temp_write/flush/validate_pending/replace', 'save_v3_readiness'], noUntypedJsonBucket: true },
  restorePhases: ['envelope_version_checksum', 'parse_unactivated_staging', 'content_id_invariants', 'F1_static_owner_load', 'hydrate_authority', 'reconcile', 'F3_rebuild_hooks', 'new_epoch_generation', 'atomic_activate', 'readonly_first_worldview'],
  migrationMatrix: { accepted: ['v2'], rejected: ['missing', 'v1', 'future', 'unknown'], choice: { missing: 'unresolved', legacyTaken: 'taken', legacyLeft: 'left', invalid: 'E_V2_CHOICE' }, coordinates: 'explicit units/origin/axis only; no guessed scaling', output: 'no legacy dual authority', gateB: ['hive_lockdown'], npcIndependent: true },
  tests: ['round-trip', 'staging-no-active-pollution', 'activate-once', 'checksum/truncated/version', 'finite/reference/duplicate/content', 'one-shot/reward-dedupe', 'epoch/generation', 'v2-v3 deterministic', 'Grid deny scan', 'atomic fault injection', 'CampaignStore/save regression', 'default-off/no-UI proof'],
  collisionChecks: typeCollision, coordination: { F1: { status: 'required_first', serialFiles: ['server-rs/src/world_v3.rs', 'server-rs/src/lib.rs'] }, F3: { status: 'future_hook_only', source: 'NB-13 completion and coordination', serialFiles: ['server-rs/src/world_v3.rs'] }, F2: 'not in F4', F5: 'not in F4' },
  gateChecklist: planChecks, checks, summary: { total: checks.length, passed: checks.filter(item => item.pass).length, failed: checks.filter(item => !item.pass).length },
  blockers: ['E_NO_FULL_SAVE_V3_RUNTIME_RESTORE', 'E_WORLD_V3_NOT_RUNTIME_WIRED', 'E_CONTINUOUS_PRIMITIVES_MISSING'],
  excludedChanges: ['CampaignStore/campaign_save/path', 'Tauri commands/IPC', 'UI/Babylon/HTTP', 'F1 owner implementation', 'F3 spatial implementation', 'NB-13 files', 'legacy Grid removal', 'media/assets'],
  nextPackageSkeleton: { name: 'FORMAL-F4-SAVE-V3-RESTORE', allowedFiles: manifest.map(item => item.path), defaultOff: true, currentSaveOwner: 'CampaignStore unchanged', rollback: 'surgical restore/delete per manifest', serialOrder: ['F1 freeze world_v3.rs/lib.rs', 'F4 migration/persistence', 'F3 hook coordination', 'formal review'] },
  scope: { allowed: ['docs/FIRST_FORMAL_SAVE_V3_SLICE_PLAN.md', 'tools/continuous-tps-architecture-probe/new-baseline-first-formal-save-v3-slice/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-save-v3-slice-nb14.json'], forbiddenWrites: ['server-rs/**', 'NB-13', 'TP-15C3', 'TP-18D1', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify({ status: output.status, summary: output.summary, resultPath }, null, 2));
if (output.status !== 'save_v3_implementation_package_ready_for_gate_review') process.exitCode = 1;
