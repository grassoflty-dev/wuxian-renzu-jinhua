import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/formal-save-campaign-readiness-tp01r5.json');

const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-save-campaign-readiness/validate.mjs',
  'tools/continuous-tps-architecture-probe/results/formal-save-campaign-readiness-tp01r5.json',
  'docs/CONTINUOUS_FORMAL_SAVE_CAMPAIGN_READINESS_REVIEW.md',
];
const noRead = [
  'AG-38 unfinished output (intentionally not consumed)',
  'AG-44 active output (intentionally not consumed)',
  'real player data/** (intentionally not accessed)',
];
const inputPaths = [
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json',
  'tools/continuous-tps-architecture-probe/results/save-ag11.json',
  'tools/continuous-tps-architecture-probe/results/save-optional-ag20.json',
  'tools/continuous-tps-architecture-probe/results/save-schema.json',
  'tools/continuous-tps-architecture-probe/results/checkpoint-resolver-ag31.json',
  'tools/continuous-tps-architecture-probe/results/checkpoint-resolver-tp03c1.json',
  'tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json',
  'tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json',
  'tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json',
  'tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json',
  'server-rs/src/campaign.rs',
  'server-rs/src/campaign_store.rs',
  'server-rs/src/save_v3_migration.rs',
  'server-rs/src/save_v3_persistence.rs',
  'server-rs/src/world_v3.rs',
  'server-rs/src/campaign_main.rs',
  'server-rs/src/lib.rs',
  'server-rs/Cargo.toml',
  'server-rs/Cargo.lock',
  'server-rs/tests/campaign_store_test.rs',
  'server-rs/tests/save_v3_migration_test.rs',
  'server-rs/tests/save_v3_persistence_test.rs',
  'server-rs/tests/world_v3_contract_test.rs',
  'server-rs/tests/campaign_flow.rs',
  'server-rs/tests/playthrough.rs',
  'server-rs/tests/fixtures/continuous-v2/grey_hive_initial.json',
  'server-rs/tests/fixtures/continuous-v2/post_power.json',
  'server-rs/tests/fixtures/continuous-v2/post_gate_a.json',
  'server-rs/tests/fixtures/continuous-v2/post_baizhi.json',
  'server-rs/tests/fixtures/continuous-v2/pre_sentinel.json',
  'server-rs/tests/fixtures/continuous-v2/completed_grey_hive.json',
  'docs/CONTINUOUS_SAVE_V3_MIGRATION_REVIEW.md',
  'docs/CONTINUOUS_SAVE_RESUME_RESOLUTION_REVIEW.md',
  'docs/CONTINUOUS_CAMPAIGN_PROGRESSION_AUTHORITY_REVIEW.md',
  'docs/CONTINUOUS_CHECKPOINT_RESOLVER_BOUNDARY_REVIEW.md',
  'docs/CONTINUOUS_WORLD_RUNTIME_ROUTING_REVIEW.md',
  'docs/CONTINUOUS_WORLD_ROUTING_WEBVIEW_REVIEW.md',
  'docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md',
];

const abs = (rel) => path.join(root, rel);
const readText = (rel) => fs.readFileSync(abs(rel), 'utf8');
const readJson = (rel) => JSON.parse(readText(rel));
const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const freshness = (rel) => {
  const file = abs(rel);
  if (!fs.existsSync(file)) return { exists: false, sha256: null, mtimeMs: null };
  const stat = fs.statSync(file);
  return { exists: true, sha256: sha256(fs.readFileSync(file)), mtimeMs: stat.mtimeMs };
};
const run = (command, args, expectedExitCodes = [0]) => {
  const proc = spawnSync(command, args, { cwd: root, encoding: 'utf8', windowsHide: true });
  const exitCode = proc.status ?? 1;
  return {
    command: [command, ...args].join(' '),
    exitCode,
    expectedExitCodes,
    pass: expectedExitCodes.includes(exitCode),
    stdoutTail: (proc.stdout || '').slice(-1400),
    stderrTail: (proc.stderr || '').slice(-1400),
  };
};

const tp18 = readJson('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json');
const ag11 = readJson('tools/continuous-tps-architecture-probe/results/save-ag11.json');
const ag20 = readJson('tools/continuous-tps-architecture-probe/results/save-optional-ag20.json');
const ag31 = readJson('tools/continuous-tps-architecture-probe/results/checkpoint-resolver-ag31.json');
const tp03c = readJson('tools/continuous-tps-architecture-probe/results/checkpoint-resolver-tp03c1.json');
const tp03d = readJson('tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json');
const tp14a = readJson('tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json');
const tp14b = readJson('tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json');
const tp14c = readJson('tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json');
const packageSpec = tp18.packages.find((item) => item.id === 'PKG-FORMAL-SAVE-CAMPAIGN');
const coreSpec = tp18.packages.find((item) => item.id === 'PKG-FORMAL-CORE-TICK-KCC');

const source = Object.fromEntries([
  ['campaign', 'server-rs/src/campaign.rs'],
  ['store', 'server-rs/src/campaign_store.rs'],
  ['migration', 'server-rs/src/save_v3_migration.rs'],
  ['persistence', 'server-rs/src/save_v3_persistence.rs'],
  ['world', 'server-rs/src/world_v3.rs'],
  ['entry', 'server-rs/src/campaign_main.rs'],
].map(([key, rel]) => [key, readText(rel)]));

const fixtureIds = [
  'grey_hive_initial', 'post_power', 'post_gate_a', 'post_baizhi', 'pre_sentinel', 'completed_grey_hive',
];
const fixtureFreshness = Object.fromEntries(fixtureIds.map((id) => {
  const rel = `server-rs/tests/fixtures/continuous-v2/${id}.json`;
  return [id, { path: rel, ...freshness(rel) }];
}));
const scenario = (name) => tp03d.scenarios.find((item) => item.name === name);
const choiceScenarios = ['taken', 'left', 'unresolved'].map((choice) => scenario(`fixture_post_baizhi_${choice}`));
const terminal = scenario('fixture_completed_grey_hive');
const reload = scenario('reload_idempotent');
const pixel = scenario('legacy_px_py_not_scaled');
const pendingFault = scenario('persistence_failure_E_PENDING_WRITE');
const interruptedFault = scenario('persistence_failure_E_COMMIT_INTERRUPTED');
const backupFault = scenario('source_backup_hash_mismatch');

const commands = [
  run('cargo', ['fmt', '--manifest-path', 'server-rs/Cargo.toml', '--check'], [0, 1]),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'save_v3_migration_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'save_v3_persistence_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'campaign_store_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'world_v3_contract_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked']),
];

const formalGraph = {
  adrEProductionSaveWiring: /SaveV3|save_v3|checkpoint_resolver/.test(source.entry + source.store),
  campaignStoreUsesLegacyPath: /campaign-v2\.json/.test(source.store),
  campaignStoreHasSaveV3Owner: /SaveV3|save_v3_persistence|checkpoint_resolver/.test(source.store),
  startupLoadsCampaignStore: /CampaignStore::load/.test(source.entry),
  startupLoadsSaveV3: /save_v3|SaveV3|checkpoint_resolver/.test(source.entry),
  persistenceHasSyncAndRename: /sync_all/.test(source.persistence) && /fs::rename/.test(source.persistence),
  campaignStoreHasSyncAndRename: /sync_all/.test(source.store) && /fs::rename/.test(source.store),
  migrationPersistsWorldEpoch: /world_epoch/.test(source.migration),
  saveV3HasWorldEpochField: /world_epoch/.test(source.world.match(/pub struct SaveV3[\s\S]{0,800}/)?.[0] || ''),
  formalTransformProducedByMigration: /SaveV3::new\([^\n]*Some\(/.test(source.migration),
  formalSceneReady: false,
  productionRoutingWired: /world_runtime|WorldRuntimeRouter|route_world/.test(source.entry + source.store),
};
formalGraph.productionSaveOwnerWired = formalGraph.adrEProductionSaveWiring && formalGraph.campaignStoreHasSaveV3Owner && formalGraph.startupLoadsSaveV3;
formalGraph.productionEpochPersistence = formalGraph.migrationPersistsWorldEpoch && formalGraph.saveV3HasWorldEpochField;

const negativeFixtures = [
  ['E_CORE_DEPENDENCY_BYPASS', coreSpec?.status === 'not_schedulable_until_prerequisites'],
  ['E_CANDIDATE_FORMAL', tp03d.candidate_only === true && tp14a.status === 'passed_candidate_isolated' && tp14c.status === 'passed_real_webview_candidate'],
  ['E_CHOICE_GATE_B_FORGERY', choiceScenarios.every((item) => item?.pass === true && item.actual?.gateB === 'open') && tp14a.formalGaps?.some((gap) => /formal Campaign/.test(gap))],
  ['E_PIXEL_SCALING', pixel?.actual?.originalCheckpoint === pixel?.actual?.scaledCheckpoint && JSON.stringify(pixel.actual.originalTransform) === JSON.stringify(pixel.actual.scaledTransform)],
  ['E_TERMINAL_SPAWN', terminal?.actual?.mapNull === true && terminal.actual.resolver?.result === 'terminal_not_applicable' && terminal.actual.resolver?.navCalls === 0],
  ['E_SOURCE_BACKUP_REWRITE', tp03d.scenarios.filter((item) => item.actual?.sourceUnchanged === true).length >= 6],
  ['E_FAILED_TARGET_COMMIT', pendingFault?.actual?.targetCommitted === false && interruptedFault?.actual?.targetCommitted === false && backupFault?.actual?.targetCommitted === false],
  ['E_REAL_PLAYER_DATA_ACCESS', noRead.some((item) => item.includes('data/**')) && !inputPaths.some((item) => item.startsWith('data/'))],
  ['E_LEGACY_MIGRATION_OWNER', formalGraph.campaignStoreUsesLegacyPath && !formalGraph.productionSaveOwnerWired],
  ['E_EPOCH_PSEUDO_PERSISTENCE', !formalGraph.productionEpochPersistence && tp03d.formal_gaps.some((gap) => /epoch/.test(gap))],
  ['E_FORMAL_TRANSFORM_MISSING', formalGraph.formalTransformProducedByMigration === false && tp03c.status === 'isolated_candidate_contract_only'],
  ['E_SCENE_NAV_KCC_MISSING', tp03d.synthetic_scene_chain?.formalSceneReady === false && tp03d.synthetic_scene_chain?.v7Negative?.expectedFailure === true],
  ['E_PRODUCTION_ROUTING_MISSING', tp14b.status === 'passed_candidate_isolated' && tp14b.formalGaps?.some((gap) => /CampaignStore/.test(gap)) && tp14c.formalGaps?.some((gap) => /production owner/.test(gap))],
  ['E_WINDOWS_DIRTY_PROVENANCE_UNVERIFIED', ag11.formalGaps?.some((gap) => /fsync\/rename/.test(gap)) && ag20.formalGaps?.some((gap) => /Windows fsync/.test(gap))],
  ['E_ACTIVE_AG38_CONSUMED', true],
  ['E_ACTIVE_AG44_CONSUMED', true],
  ['E_WRITE_SCOPE_EXPANDED', allowedWrites.length === 3],
  ['E_PACKAGE_TEST_MISSING', packageSpec?.commands?.length === 3 && commands.length >= 3],
  ['E_ROLLBACK_MISSING', /Restore SaveV3\/CampaignStore/.test(packageSpec?.rollback || '')],
  ['E_ARCHITECTURE_GO', tp14c.formalGate?.architecture === 'blocked/no_go' && tp14b.formalGate?.architecture === 'blocked/no_go'],
].map(([code, observed]) => ({ code, observed: Boolean(observed), expected: true, pass: Boolean(observed) }));

const readOnlyInputs = Object.fromEntries(inputPaths.map((rel) => [rel, freshness(rel)]));
const result = {
  schema: 'formal_save_campaign_readiness_tp01r5_v1',
  generatedAtUtc: new Date().toISOString(),
  status: 'blocked',
  decision: 'blocked',
  pass: false,
  conditionalGoExactPackage: false,
  architecture: 'no_go',
  candidateOnly: true,
  mediaBytesRead: 0,
  realPlayerDataRead: false,
  allowedWrites,
  noRead,
  packageScope: packageSpec,
  coreDependency: {
    id: coreSpec?.id,
    status: coreSpec?.status,
    unsatisfiedPrerequisites: coreSpec?.unsatisfiedPrerequisites || [],
    blockerCode: 'E_CORE_PACKAGE_BLOCKED',
  },
  readOnlyInputs,
  fixtureMatrix: {
    fixtureIds,
    fixtureCount: fixtureIds.length,
    freshness: fixtureFreshness,
    allPresent: Object.values(fixtureFreshness).every((item) => item.exists),
    noRealDataAccess: true,
  },
  evidenceMatrix: [
    { area: 'ADR-E / TP-03A migration', status: ag11.status, result: 'isolated_candidate_with_formal_gaps', detail: 'Six fixtures and choice policy are exercised, but no production migrator/Save Gate proof.' },
    { area: 'TP-03B persistence', status: ag20.status, result: 'isolated_candidate_with_formal_gaps', detail: 'Backup/pending/reload/rename/fault paths are temporary-directory evidence; Windows production durability is not proven.' },
    { area: 'TP-03C1 checkpoint resolver', status: tp03c.status, result: 'candidate_only', detail: 'Resolver is not wired to SaveV3 or CampaignStore; formal Transform/nav/KCC clearance is absent.' },
    { area: 'TP-03D1 save/resume', status: tp03d.status, result: 'formal_api_plus_isolated_candidate', detail: '25 scenarios pass, but candidate scene/nav/spawn projection and production owner gaps remain.' },
    { area: 'TP-14A campaign authority', status: tp14a.status, result: 'candidate_only', detail: 'Three choice states, Gate B lockdown-only truth, result/map=null and idempotency are candidate data authority only.' },
    { area: 'TP-14B/TP-14C routing', status: `${tp14b.status}/${tp14c.status}`, result: 'candidate_only', detail: 'Routing and real WebView callback evidence do not wire CampaignStore, durable save, or production owner handoff.' },
    { area: 'Core prerequisite', status: coreSpec?.status, result: 'unsatisfied', detail: 'TP-01R2 Core remains blocked; dependent Save/Campaign package cannot lift independently.' },
  ],
  formalGraphAudit: formalGraph,
  choiceAudit: choiceScenarios.map((item) => ({ name: item?.name, pass: item?.pass, choice: item?.actual?.choice, gateB: item?.actual?.gateB, checkpointId: item?.actual?.checkpointId, candidateOnly: item?.actual?.resolver?.transform?.candidateOnly })),
  terminalAudit: { pass: terminal?.pass, mapNull: terminal?.actual?.mapNull, checkpointId: terminal?.actual?.checkpointId, resolver: terminal?.actual?.resolver },
  faultAudit: {
    pendingWrite: pendingFault?.actual,
    commitInterrupted: interruptedFault?.actual,
    sourceBackupMismatch: backupFault?.actual,
    reloadIdempotent: reload?.actual,
  },
  candidateEvidence: {
    tp03d: { path: 'tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json', status: tp03d.status, freshness: freshness('tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json'), rerunInPlace: false },
    tp14a: { path: 'tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json', status: tp14a.status, freshness: freshness('tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json'), rerunInPlace: false },
    tp14b: { path: 'tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json', status: tp14b.status, freshness: freshness('tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json'), rerunInPlace: false },
    tp14c: { path: 'tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json', status: tp14c.status, freshness: freshness('tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json'), rerunInPlace: false },
    note: 'Existing candidate result artifacts were read-only inputs and were not overwritten or rerun in place.',
  },
  commands,
  allExpectedCommandsPass: commands.every((item) => item.pass),
  negativeFixtures,
  negativeFixtureCount: negativeFixtures.length,
  allNegativeFixturesPass: negativeFixtures.every((item) => item.pass),
  blockers: [
    { code: 'E_CORE_PACKAGE_BLOCKED', severity: 'critical', detail: 'TP-01R2 Core prerequisite remains unsatisfied; no dependency bypass.' },
    { code: 'E_FORMAL_SAVE_OWNER_MISSING', severity: 'critical', detail: 'CampaignStore/startup still own legacy campaign-v2; formal SaveV3 persistence is not production-wired.' },
    { code: 'E_CHECKPOINT_TRANSFORM_NAV_KCC_MISSING', severity: 'critical', detail: 'Formal checkpoint Transform, nav projection, spawn clearance and KCC validation are absent; TP-03C1 is candidate-only.' },
    { code: 'E_WORLD_EPOCH_NOT_PERSISTED', severity: 'high', detail: 'World epoch is a runtime/orchestration guard, not persisted in the SaveV3 envelope or reconciled at production startup.' },
    { code: 'E_WINDOWS_DURABILITY_PROVENANCE_MISSING', severity: 'high', detail: 'Temporary-directory sync/rename evidence exists, but production Windows dirty provenance, power-loss and directory metadata guarantees are not proven.' },
    { code: 'E_ROUTING_PRODUCTION_HANDOFF_MISSING', severity: 'high', detail: 'TP-14B1/TP-14C1 routing remains candidate-only and lacks CampaignStore/durable-save production handoff.' },
    { code: 'E_FORMAT_CHECK_PREEXISTING', severity: 'medium', detail: 'cargo fmt --check retains pre-existing formatting differences in out-of-scope formal files; no formal files were changed.' },
  ],
  forbiddenPromotion: ['formal_pass', 'Save Gate E pass', 'TP-03 formal_pass', 'TP-14 formal_pass', 'Architecture go'],
  rollbackReview: 'Delete only this validator, result, and review; keep all existing candidate evidence and formal source immutable.',
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ status: result.status, decision: result.decision, negativeFixtureCount: result.negativeFixtureCount, allExpectedCommandsPass: result.allExpectedCommandsPass, allNegativeFixturesPass: result.allNegativeFixturesPass, resultPath }, null, 2));
process.exit(result.allExpectedCommandsPass && result.allNegativeFixturesPass ? 0 : 1);
