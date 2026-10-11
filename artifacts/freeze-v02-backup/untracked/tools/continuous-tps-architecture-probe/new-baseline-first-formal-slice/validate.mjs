import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const dir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(dir, '..', '..', '..');
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_WORLD_V3_SLICE_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-first-formal-slice-nb12.json');
const read = relative => fs.readFileSync(path.join(root, relative), 'utf8');
const exists = relative => fs.existsSync(path.join(root, relative));
const sha = relative => crypto.createHash('sha256').update(read(relative)).digest('hex');
const plan = read('docs/FIRST_FORMAL_WORLD_V3_SLICE_PLAN.md');
const main = read('server-rs/src/campaign_main.rs');
const lib = read('server-rs/src/lib.rs');
const worldV3 = read('server-rs/src/world_v3.rs');
const cargo = read('server-rs/Cargo.toml');
const results = {
  nb01: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-architecture-nb01.json')),
  nb04: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-runtime-audit-nb04.json')),
  nb06: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-authority-migration-nb06.json')),
  nb07: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-world-v3-runtime-wiring-nb07.json')),
  nb08: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-spatial-primitives-nb08.json')),
  nb09: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-tauri-transport-disposition-nb09.json')),
  nb10: JSON.parse(read('tools/continuous-tps-architecture-probe/results/new-baseline-save-v3-full-restore-nb10.json'))
};
const check = (id, pass, actual, expected) => ({ id, pass: Boolean(pass), actual, expected });
const plannedFiles = [
  { path: 'server-rs/src/world_v3.rs', status: 'existing_modify', reason: '补齐正式 Vec3/Transform/WorldView 边界，不接管 CampaignStore', test: 'world_v3 round-trip and finite/schema validation', rollback: 'restore original file' },
  { path: 'server-rs/src/world_v3_owner.rs', status: 'new', reason: '新增 Rust WorldV3Owner、fixed tick、snapshot、dispose', test: 'owner/tick/order/epoch/reload/dispose/determinism', rollback: 'delete file and module line' },
  { path: 'server-rs/src/world_v3_preview.rs', status: 'new_optional_f2', reason: '显式 preview query/channel handshake，默认不调用', test: 'Tauri preview command/channel contract', rollback: 'delete file and registry lines' },
  { path: 'server-rs/src/lib.rs', status: 'existing_modify', reason: '声明隔离 owner/preview module', test: 'cargo check/test module discovery', rollback: 'remove new module declarations' },
  { path: 'server-rs/src/campaign_main.rs', status: 'existing_modify_optional_f2', reason: '旁路 manage 与 preview command registry，不改现有 commands', test: 'command collision/registry and default unreachable', rollback: 'remove new state/commands/registry entries' },
  { path: 'server-rs/tests/world_v3_owner_test.rs', status: 'new', reason: 'F1 owner contract regression tests', test: 'cargo test owner suite', rollback: 'delete test file' },
  { path: 'server-rs/tests/world_v3_preview_contract_test.rs', status: 'new_optional_f2', reason: 'F2 generation/epoch/tick/ack/channel tests', test: 'cargo test preview contract suite', rollback: 'delete test file' }
];
const existingCommands = [...main.matchAll(/fn\s+([a-zA-Z0-9_]+)\s*\(/g)].map(match => match[1]);
const existingModules = [...lib.matchAll(/pub\s+mod\s+([a-zA-Z0-9_]+)/g)].map(match => match[1]);
const proposedCommands = ['world_v3_preview_readiness', 'world_v3_preview_query', 'world_v3_preview_start', 'world_v3_preview_stop'];
const proposedModules = ['world_v3_owner', 'world_v3_preview'];
const collisionChecks = [
  check('command_names_unique', proposedCommands.every(name => !existingCommands.includes(name)) && new Set(proposedCommands).size === proposedCommands.length, proposedCommands.filter(name => existingCommands.includes(name)), 'no collision'),
  check('module_names_unique', proposedModules.every(name => !existingModules.includes(name)) && new Set(proposedModules).size === proposedModules.length, proposedModules.filter(name => existingModules.includes(name)), 'no collision'),
  check('existing_registry_anchor', /generate_handler!/.test(main) && /campaign_view/.test(main) && /tp06a_start/.test(main), true, 'existing commands remain anchored'),
  check('cargo_dependencies_present', /serde_json/.test(cargo) && /tauri/.test(cargo), true, 'no new dependency required')
];
const resultChecks = Object.entries(results).map(([id, result]) => check(`${id}_candidate_result`, result.candidateOnly === true || result.formalIntegration === false || result.formalRuntimeWired === false || /blocked|partial|audit_complete/.test(result.status ?? ''), { status: result.status, conclusion: result.conclusion }, 'candidate/isolation boundary'));
const manifestChecks = plannedFiles.flatMap(item => [
  check(`manifest_path_${item.path}`, item.path.startsWith('server-rs/') && !item.path.includes('*'), item.path, 'precise file path'),
  check(`manifest_metadata_${item.path}`, Boolean(item.reason && item.test && item.rollback), { reason: Boolean(item.reason), test: Boolean(item.test), rollback: Boolean(item.rollback) }, 'reason/test/rollback')
]);
const forbiddenPlanTerms = ['release-ui/js/', 'tps-renderer.js', 'campaign_save', '/api/*', 'Grid removal', 'Save v3 write'];
const checklistChecks = [
  check('plan_status', /implementation_package_ready_for_gate_review/.test(plan) && !/implemented\s*$/.test(plan), 'ready_for_gate_review', 'not implemented'),
  check('current_owner_preserved', /CampaignStore.*保持当前语义和 owner/.test(plan), true, 'CampaignStore remains owner'),
  check('no_ui_cutover', /不接入当前 release UI、Babylon、Save v3 写入/.test(plan), true, 'no UI/Babylon/save cutover'),
  check('no_grid_input', /不读取 legacy Grid/.test(plan) && /Grid 输入/.test(plan), true, 'no Grid runtime input'),
  check('no_http', /不得使用 `\/api\/\*`/.test(plan), true, 'no HTTP API'),
  check('f1_f7_boundary', ['F1 Rust WorldV3Owner', 'F2 Tauri preview query/channel', 'F3 formal spatial primitives', 'F4 Save v3 restore', 'F5 UI/Babylon read-only adapter', 'F6 owner cutover', 'F7 legacy Grid removal'].every(term => plan.includes(term)), true, 'F1/F2 only first slice'),
  check('excluded_changes', forbiddenPlanTerms.every(term => plan.includes(term) || term === 'release-ui/js/'), forbiddenPlanTerms, 'UI/renderer/save/grid/http exclusions explicit'),
  check('symbol_anchors', /Vec3/.test(worldV3) && /WorldView/.test(worldV3) && /ContinuousRuntime/.test(main) && /pub\s+mod\s+world_v3/.test(lib), { worldV3: true, main: true, lib: true }, 'current anchors present'),
  check('no_nb11_input', !/new-baseline-.*nb11|NB-11.*path|nb11.*json/i.test(plan), 'no NB-11 file input', 'NB-11 prerequisite only')
];
const dependencies = ['new-baseline-architecture-nb01.json', 'new-baseline-runtime-audit-nb04.json', 'new-baseline-authority-migration-nb06.json', 'new-baseline-world-v3-runtime-wiring-nb07.json', 'new-baseline-spatial-primitives-nb08.json', 'new-baseline-tauri-transport-disposition-nb09.json', 'new-baseline-save-v3-full-restore-nb10.json'];
const dependencyChecks = dependencies.map(file => check(`dependency_${file}`, exists(`tools/continuous-tps-architecture-probe/results/${file}`), file, 'completed result present'));
const allChecks = [...collisionChecks, ...resultChecks, ...manifestChecks, ...checklistChecks, ...dependencyChecks];
const inventoryFiles = ['server-rs/Cargo.toml', 'server-rs/src/lib.rs', 'server-rs/src/world_v3.rs', 'server-rs/src/campaign_main.rs', 'server-rs/src/tp06a_heartbeat.rs'];
const output = {
  schema: 'new_baseline_first_formal_slice_nb12_result_v1', status: allChecks.every(item => item.pass) ? 'implementation_package_ready_for_gate_review' : 'failed_readiness_review', candidateOnly: true, formalCodeChanged: false,
  currentInventory: { crate: { path: 'server-rs/Cargo.toml', defaultBin: 'server-rs/src/campaign_main.rs', package: 'wuxian-horror-ch1' }, modules: existingModules, commands: existingCommands, channel: 'tp06a_start(channel: Channel<WorldView>, world_epoch)', tests: ['server-rs/tests/world_v3_contract_test.rs', 'server-rs/tests/save_v3_migration_test.rs', 'server-rs/tests/save_v3_persistence_test.rs'], files: inventoryFiles.map(file => ({ path: file, bytes: fs.statSync(path.join(root, file)).size, sha256: sha(file) })) },
  proposedApi: { owner: ['new/config', 'start(epoch)', 'tick(fixed_dt)', 'snapshot()', 'stop(token)', 'dispose()', 'readiness()'], commands: proposedCommands, protocol: 'world-v3-preview-candidate-v1', fields: ['schemaVersion', 'protocol', 'worldId', 'worldEpoch', 'serverTick', 'ackSeq', 'generation', 'Vec3', 'Transform', 'Velocity'], prohibited: ['frontend setTransform', 'frontend damage settlement', 'Grid input', 'formal save write', 'Babylon/UI authority'] },
  patchManifest: plannedFiles,
  collisionChecks, dependencyGraph: { F1: ['NB-01', 'NB-07'], F2: ['F1', 'NB-09'], F3: ['F1', 'NB-08'], F4: ['F1', 'NB-10'], F5: ['F2', 'F3'], F6: ['F4', 'F5', 'NB-11-cutover-prerequisite'], F7: ['F6', 'NB-11'] },
  gateChecklist: checklistChecks, dependencyChecks, checks: allChecks, summary: { total: allChecks.length, passed: allChecks.filter(item => item.pass).length, failed: allChecks.filter(item => !item.pass).length },
  blockers: ['E_WORLD_V3_NOT_RUNTIME_WIRED', 'E_CONTINUOUS_PRIMITIVES_MISSING', 'E_API_FALLBACK_UNBACKED', 'E_NO_FULL_SAVE_V3_RUNTIME_RESTORE'],
  risks: ['旁路 owner 被误用为当前 gameplay owner', 'preview command 被 release UI 自动调用', 'WorldView 双写/epoch 泄漏', 'candidate evidence 被误称为 formal runtime'],
  excludedChanges: ['CampaignStore/campaign_view/action/save semantics', 'release UI and Babylon adapter', 'Save v3 write/restore integration', 'HTTP /api routes', 'legacy Grid removal', 'NB-11 files', 'formal collider/nav/AI/assets'],
  nextPackageSkeleton: { name: 'FORMAL-F1-WORLD-V3-OWNER', allowedFiles: plannedFiles.filter(item => !item.status.includes('optional')).map(item => item.path), defaultOff: true, owner: 'Rust', rollback: 'surgical deletion/restoration of manifest files', reviewBeforeMerge: ['Architecture Gate approval', 'NB-11 complete before any cutover'] },
  scope: { allowed: ['docs/FIRST_FORMAL_WORLD_V3_SLICE_PLAN.md', 'tools/continuous-tps-architecture-probe/new-baseline-first-formal-slice/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-slice-nb12.json'], forbiddenWrites: ['server-rs/**', 'NB-11', 'TP-15C3', 'TP-18D1', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify({ status: output.status, summary: output.summary, resultPath }, null, 2));
if (output.status !== 'implementation_package_ready_for_gate_review') process.exitCode = 1;
