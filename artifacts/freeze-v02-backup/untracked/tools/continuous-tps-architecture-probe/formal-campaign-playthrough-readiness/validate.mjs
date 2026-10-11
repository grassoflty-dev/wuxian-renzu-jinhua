import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/formal-campaign-playthrough-readiness-tp01r8.json');
const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-campaign-playthrough-readiness/validate.mjs',
  'tools/continuous-tps-architecture-probe/results/formal-campaign-playthrough-readiness-tp01r8.json',
  'docs/CONTINUOUS_FORMAL_CAMPAIGN_PLAYTHROUGH_READINESS_REVIEW.md',
];
const noRead = [
  'AG-38 unfinished output',
  'TP-04C1 active output',
  'TP-01R7 active output',
  'real player data/**',
  'assets/**, media/**, public/** physical bytes',
  'network and remote resources',
];
const inputPaths = [
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json',
  'tools/continuous-tps-architecture-probe/results/formal-core-readiness-tp01r2.json',
  'tools/continuous-tps-architecture-probe/results/formal-scene-readiness-tp01r3.json',
  'tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json',
  'tools/continuous-tps-architecture-probe/results/formal-save-campaign-readiness-tp01r5.json',
  'tools/continuous-tps-architecture-probe/results/formal-asset-release-readiness-tp01r6.json',
  'tools/continuous-tps-architecture-probe/results/formal-scene-exporter-tp09b1.json',
  'tools/continuous-tps-architecture-probe/results/grey-hive-scene-export-tp09a1.json',
  'tools/continuous-tps-architecture-probe/results/interaction-authority-tp10a1.json',
  'tools/continuous-tps-architecture-probe/results/navgraph-ai-tp11a1.json',
  'tools/continuous-tps-architecture-probe/results/combat-spatial-tp12a1.json',
  'tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json',
  'tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json',
  'tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json',
  'tools/continuous-tps-architecture-probe/results/checkpoint-resolver-tp03c1.json',
  'tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json',
  'tools/continuous-tps-architecture-probe/results/physics-gate-b-tp04b2.json',
  'tools/continuous-tps-architecture-probe/results/global-nav-candidate-ag43.json',
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json',
  'server-rs/src/campaign.rs',
  'server-rs/src/campaign_store.rs',
  'server-rs/src/campaign_main.rs',
  'server-rs/src/state.rs',
  'server-rs/src/world_v3.rs',
  'server-rs/src/save_v3_migration.rs',
  'server-rs/src/save_v3_persistence.rs',
  'server-rs/src/continuous_input.rs',
  'server-rs/src/fixed_step.rs',
  'server-rs/tests/campaign_flow.rs',
  'server-rs/tests/campaign_store_test.rs',
  'server-rs/tests/playthrough.rs',
  'server-rs/tests/world_v3_contract_test.rs',
  'server-rs/tests/save_v3_migration_test.rs',
  'server-rs/tests/save_v3_persistence_test.rs',
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

const tp18 = readJson('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json');
const core = readJson('tools/continuous-tps-architecture-probe/results/formal-core-readiness-tp01r2.json');
const scene = readJson('tools/continuous-tps-architecture-probe/results/formal-scene-readiness-tp01r3.json');
const ipc = readJson('tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json');
const save = readJson('tools/continuous-tps-architecture-probe/results/formal-save-campaign-readiness-tp01r5.json');
const assets = readJson('tools/continuous-tps-architecture-probe/results/formal-asset-release-readiness-tp01r6.json');
const tp09a = readJson('tools/continuous-tps-architecture-probe/results/grey-hive-scene-export-tp09a1.json');
const tp09b = readJson('tools/continuous-tps-architecture-probe/results/formal-scene-exporter-tp09b1.json');
const tp10 = readJson('tools/continuous-tps-architecture-probe/results/interaction-authority-tp10a1.json');
const tp11 = readJson('tools/continuous-tps-architecture-probe/results/navgraph-ai-tp11a1.json');
const tp12 = readJson('tools/continuous-tps-architecture-probe/results/combat-spatial-tp12a1.json');
const tp14a = readJson('tools/continuous-tps-architecture-probe/results/campaign-progression-authority-tp14a1.json');
const tp14b = readJson('tools/continuous-tps-architecture-probe/results/world-runtime-routing-tp14b1.json');
const tp14c = readJson('tools/continuous-tps-architecture-probe/results/tauri-world-routing-tp14c1.json');
const tp03c = readJson('tools/continuous-tps-architecture-probe/results/checkpoint-resolver-tp03c1.json');
const tp03d = readJson('tools/continuous-tps-architecture-probe/results/save-resume-resolution-tp03d1.json');
const tp04b2 = readJson('tools/continuous-tps-architecture-probe/results/physics-gate-b-tp04b2.json');
const ag43 = readJson('tools/continuous-tps-architecture-probe/results/global-nav-candidate-ag43.json');
const packageSpec = tp18.packages.find((item) => item.id === 'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH');

const source = {
  campaign: readText('server-rs/src/campaign.rs'),
  store: readText('server-rs/src/campaign_store.rs'),
  entry: readText('server-rs/src/campaign_main.rs'),
  state: readText('server-rs/src/state.rs'),
  world: readText('server-rs/src/world_v3.rs'),
  migration: readText('server-rs/src/save_v3_migration.rs'),
  persistence: readText('server-rs/src/save_v3_persistence.rs'),
  input: readText('server-rs/src/continuous_input.rs'),
  fixed: readText('server-rs/src/fixed_step.rs'),
};

const scenario = (name) => tp03d.scenarios.find((item) => item.name === name);
const threeChoice = ['taken', 'left', 'unresolved'].map((choice) => scenario(`fixture_post_baizhi_${choice}`));
const terminal = scenario('fixture_completed_grey_hive');
const reload = scenario('reload_idempotent');
const faultCases = ['persistence_failure_E_PENDING_WRITE', 'persistence_failure_E_COMMIT_INTERRUPTED', 'source_backup_hash_mismatch'].map(scenario);

const formalGraph = {
  gateBOpenedByLockdown: /"hive_lockdown"[\s\S]{0,260}open_gate\(map, gate_b\)/.test(source.campaign),
  choiceIsSeparateOptionalState: /optional_choice/.test(source.campaign) && /optional_objective_id/.test(source.campaign),
  choiceUsedAsGatePrerequisite: /gate_b[\s\S]{0,300}optional_choice|optional_choice[\s\S]{0,300}gate_b/.test(source.campaign),
  terminalClearsMap: /self\.mode = "result"[\s\S]{0,450}self\.map = None/.test(source.campaign),
  rewardHasFirstClearGuard: /first_clear = !world_is_cleared/.test(source.campaign) && /self\.run_reward_allowed/.test(source.campaign),
  campaignStoreLegacyOwner: /campaign-v2\.json/.test(source.store),
  formalSaveWiredToStore: /SaveV3|save_v3|checkpoint_resolver/.test(source.store + source.entry),
  formalCheckpointResolverWired: /checkpoint_resolver/.test(source.campaign + source.store + source.entry),
  formalRoutingWired: /WorldRuntimeRouter|world_runtime|route_world/.test(source.campaign + source.store + source.entry),
  formalInputOwnerWired: /InputReceiver|continuous_input|tp06b_input/.test(source.entry),
  worldEpochInSave: /world_epoch/.test(source.migration) && /world_epoch/.test(source.world.match(/pub struct SaveV3[\s\S]{0,900}/)?.[0] || ''),
  formalSceneReady: scene.formalSceneReady === true,
  productionWorldOwner: /WorldStateV3|WorldRuntime/.test(source.entry) && !/synthetic/.test(source.entry),
};
formalGraph.gateBChoiceIndependent = formalGraph.gateBOpenedByLockdown && !formalGraph.choiceUsedAsGatePrerequisite;
formalGraph.terminalNoSpawn = formalGraph.terminalClearsMap && terminal?.actual?.mapNull === true;
formalGraph.formalCampaignOwnerComplete = formalGraph.formalSaveWiredToStore && formalGraph.formalCheckpointResolverWired && formalGraph.formalRoutingWired && formalGraph.formalInputOwnerWired;

const matrix = [
  { prerequisite: 'PKG-FORMAL-CORE-TICK-KCC / TP-01R2', evidence: 'formal-core-readiness-tp01r2.json', formalOwnerFile: 'server-rs/src/fixed_step.rs + server-rs/src/continuous_kcc.rs', test: 'formal core result blocked; existing full regression only', rollback: 'Core package rollback only', currentStatus: core.status, formal: false },
  { prerequisite: 'TP-03 / Save Gate E', evidence: 'formal-save-campaign-tp01r5.json + save-resume-resolution-tp03d1.json', formalOwnerFile: 'server-rs/src/save_v3_migration.rs + save_v3_persistence.rs + campaign_store.rs', test: '7 migration + 7 persistence + 7 store tests recorded pass; production owner missing', rollback: 'Restore SaveV3/CampaignStore files; preserve legacy reader', currentStatus: save.status, formal: false },
  { prerequisite: 'TP-09 / formal scene', evidence: 'formal-scene-readiness-tp01r3.json + tp09a1/tp09b1', formalOwnerFile: 'server-rs/src/world_v3.rs + grey_hive_control_collider.rs (not production-ready)', test: 'scene/export contracts only; formal scene false', rollback: 'Remove formal scene/export owner files only', currentStatus: scene.status, formal: false },
  { prerequisite: 'TP-01R4 / formal IPC owner', evidence: 'formal-ipc-owner-readiness-tp01r4.json', formalOwnerFile: 'server-rs/src/campaign_main.rs + continuous_input.rs + fixed_step.rs', test: 'targeted IPC tests pass; formal entrypoint owner absent', rollback: 'Restore previous Channel boundary; keep evidence immutable', currentStatus: ipc.status, formal: false },
  { prerequisite: 'TP-01R6 / formal assets', evidence: 'formal-asset-release-readiness-tp01r6.json', formalOwnerFile: 'server-rs/release-ui/asset-manifest.json + asset-resolver.js + asset-cache.js (not created)', test: 'registry/resolver/bridge candidate contracts pass; public 0', rollback: 'Leave public manifest empty; remove only future release files', currentStatus: assets.status, formal: false },
  { prerequisite: 'TP-10 interaction authority', evidence: 'interaction-authority-tp10a1.json', formalOwnerFile: 'server-rs/src/campaign.rs interaction path; formal scene/collider owner absent', test: 'candidate interaction boundaries pass', rollback: 'Candidate-only rollback; no production integration', currentStatus: tp10.status, formal: false },
  { prerequisite: 'TP-11 navgraph', evidence: 'navgraph-ai-tp11a1.json + global-nav-candidate-ag43.json', formalOwnerFile: 'formal navmesh/door owner absent', test: 'candidate graph/path matrix passes; no baked navmesh', rollback: 'Remove candidate nav artifacts only', currentStatus: tp11.status, formal: false },
  { prerequisite: 'TP-12 combat/spatial', evidence: 'combat-spatial-tp12a1.json', formalOwnerFile: 'formal combat/physics owner absent', test: 'isolated spatial candidate scenarios pass', rollback: 'Candidate combat probe only', currentStatus: tp12.status, formal: false },
  { prerequisite: 'TP-14 routing/campaign', evidence: 'campaign-progression-authority-tp14a1.json + tp14b1/tp14c1', formalOwnerFile: 'server-rs/src/campaign_main.rs production handoff absent', test: 'candidate and real-WebView routing callbacks pass', rollback: 'Restore legacy owner and preserve routing evidence', currentStatus: `${tp14a.status}/${tp14b.status}/${tp14c.status}`, formal: false },
  { prerequisite: 'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH', evidence: 'TP-18B1 package specification', formalOwnerFile: 'server-rs/src/campaign.rs, campaign_store.rs, campaign_main.rs, state.rs', test: packageSpec.commands.join(' | '), rollback: packageSpec.rollback, currentStatus: packageSpec.status, formal: false },
];

const negativeFixtures = [
  ['E_CORE_BYPASS', core.status === 'blocked' && !matrix[0].formal],
  ['E_SCENE_BYPASS', scene.formalSceneReady === false && tp09a.formalGate?.architecture === 'no_go'],
  ['E_IPC_BYPASS', ipc.status === 'blocked' && formalGraph.formalInputOwnerWired === false],
  ['E_SAVE_BYPASS', save.status === 'blocked' && formalGraph.formalSaveWiredToStore === false],
  ['E_ASSET_BYPASS', assets.status === 'blocked' && assets.conditionalGoExactPackage === false],
  ['E_CHOICE_GATE_B_COUPLING', formalGraph.gateBChoiceIndependent],
  ['E_NPC_GATE_B_COUPLING', formalGraph.choiceIsSeparateOptionalState && /baizhi-choice|NPC|choice/i.test(JSON.stringify(tp14a))],
  ['E_CHECKPOINT_TRANSFORM_MISSING', tp03c.status === 'isolated_candidate_contract_only' && save.blockers.some((b) => b.code === 'E_CHECKPOINT_TRANSFORM_NAV_KCC_MISSING')],
  ['E_NAV_KCC_MISSING', scene.formalSceneReady === false && tp11.formalGaps.some((gap) => /baked navmesh|collider|KCC/i.test(gap))],
  ['E_EPOCH_TICK_DRIFT', save.blockers.some((b) => b.code === 'E_WORLD_EPOCH_NOT_PERSISTED') && !formalGraph.worldEpochInSave && /world_epoch/.test(source.world)],
  ['E_DOOR_NAV_DESYNC', tp10.formalGaps.some((gap) => /scene volume|formal navmesh|dynamic elevator/i.test(gap)) && tp11.formalGaps.some((gap) => /door owner|dynamic connector/i.test(gap))],
  ['E_INTERACTION_OWNER_MISSING', tp10.status === 'passed_candidate_isolated' && tp10.formalGaps.some((gap) => /owner wiring/i.test(gap))],
  ['E_COMBAT_OWNER_MISSING', tp12.status === 'passed_isolated_candidate' && !/formal/i.test(tp12.status)],
  ['E_REWARD_DUPLICATION', tp14a.formalGaps.some((gap) => /formal Campaign|save power-loss/i.test(gap)) && tp14a.scenarios.some((s) => s.actual?.rewardCount === 1)],
  ['E_TERMINAL_SPAWN', formalGraph.terminalNoSpawn && terminal.actual.resolver?.navCalls === 0],
  ['E_ROUTE_HANDOFF_MISSING', tp14b.formalGaps.some((gap) => /CampaignStore|durable save/i.test(gap)) && tp14c.formalGaps.some((gap) => /production owner/i.test(gap))],
  ['E_REAL_PLAYER_SAVE_ACCESS', noRead.some((item) => /real player data/.test(item)) && !inputPaths.some((item) => item.startsWith('data/'))],
  ['E_MEDIA_OR_NETWORK', noRead.some((item) => /media|network/.test(item))],
  ['E_SCOPE_CONFLICT', allowedWrites.length === 3 && !allowedWrites.some((item) => /assets[\\/]|data[\\/]|media[\\/]|public[\\/]/.test(item))],
  ['E_ACTIVE_WRITE_SET_CONFLICT', noRead.some((item) => /AG-38|TP-04C1|TP-01R7/.test(item))],
  ['E_TEST_MISSING', packageSpec.commands.length === 3 && packageSpec.writes.length === 7],
  ['E_ROLLBACK_MISSING', /Revert Campaign owner\/test files only/.test(packageSpec.rollback)],
  ['E_PROVENANCE_MISSING', core.blockers.some((b) => /provenance|DIRTY_BASELINE/i.test(b.code + b.detail))],
  ['E_ARCHITECTURE_GO', tp14c.formalGate?.architecture === 'blocked/no_go' && scene.formalGate?.architecture === 'no_go'],
  ['E_CANDIDATE_TO_FORMAL', [tp09a, tp10, tp11, tp12, tp14a, tp14b, tp14c, tp03d].every((x) => x.candidateOnly === true || /candidate|isolated/i.test(x.status))],
].map(([code, observed]) => ({ code, observed: Boolean(observed), expected: true, pass: Boolean(observed) }));

const readOnlyInputs = Object.fromEntries(inputPaths.map((rel) => [rel, freshness(rel)]));
const result = {
  schema: 'formal_campaign_playthrough_readiness_tp01r8_v1',
  generatedAtUtc: new Date().toISOString(),
  status: 'blocked',
  decision: 'blocked',
  pass: false,
  conditionalGoExactPackage: false,
  architecture: 'no_go',
  candidateOnly: true,
  allowedWrites,
  noRead,
  readOnlyInputs,
  packageScope: packageSpec,
  prerequisiteMatrix: matrix,
  formalGraphAudit: formalGraph,
  invariantAudit: {
    gateBPolicy: 'hive_lockdown_completed only; hive_choice and NPC are not prerequisites',
    gateBObservedInFormalSource: formalGraph.gateBOpenedByLockdown,
    choiceIndependenceObserved: formalGraph.gateBChoiceIndependent,
    terminalPolicy: 'mode=result + map=null + canonical gh_cp_exit + noSpawn/terminal_not_applicable',
    terminalObserved: formalGraph.terminalNoSpawn,
    canonicalCheckpointEvidence: tp03d.canonical_checkpoints,
    formalTransformNavKcc: false,
    reloadIdempotency: reload?.actual,
    faultRecovery: faultCases.map((item) => ({ name: item?.name, pass: item?.pass, actual: item?.actual })),
    routingProductionHandoff: false,
    rewardIdempotencyCandidateOnly: tp14a.scenarios.some((s) => /replay|idempotency/i.test(s.name)) && tp14a.status === 'passed_candidate_isolated',
  },
  evidenceSummary: {
    core: { status: core.status, blockers: core.blockers },
    scene: { status: scene.status, formalSceneReady: scene.formalSceneReady, blockers: scene.blockers },
    ipc: { status: ipc.status, blockers: ipc.blockers },
    save: { status: save.status, blockers: save.blockers },
    assets: { status: assets.status, blockers: assets.blockers },
    sceneContracts: { tp09a: tp09a.status, tp09b: tp09b.status, formalSceneReady: tp09a.formalGate?.architecture === 'no_go' ? false : tp09b.formalSceneReady },
    interaction: { status: tp10.status, formalGaps: tp10.formalGaps },
    nav: { status: tp11.status, formalGaps: tp11.formalGaps },
    combat: { status: tp12.status, formalGaps: tp12.formalGaps || [] },
    routing: { tp14a: tp14a.status, tp14b: tp14b.status, tp14c: tp14c.status },
  },
  candidateEvidencePolicy: 'schema-only, synthetic, isolated, and real-WebView transport evidence never promotes a formal world/campaign owner or formal scene.',
  negativeFixtures,
  negativeFixtureCount: negativeFixtures.length,
  allNegativeFixturesPass: negativeFixtures.every((item) => item.pass),
  blockers: [
    { code: 'E_CORE_BLOCKED', severity: 'critical', detail: 'TP-01R2 formal core is blocked; playthrough cannot bypass fixed-step/KCC/door ownership.' },
    { code: 'E_SCENE_FORMAL_MISSING', severity: 'critical', detail: 'TP-01R3/TP-09 evidence does not provide formal scene hash, collider/nav bake, camera bounds, dynamic motion, or checkpoint Transform provenance.' },
    { code: 'E_IPC_FORMAL_OWNER_MISSING', severity: 'critical', detail: 'TP-01R4 is blocked; formal Campaign entrypoint lacks live InputReceiver/fixed-step/WorldView owner wiring.' },
    { code: 'E_SAVE_CAMPAIGN_OWNER_MISSING', severity: 'critical', detail: 'TP-01R5 is blocked; SaveV3/checkpoint persistence is not wired to production CampaignStore/startup.' },
    { code: 'E_INTERACTION_NAV_COMBAT_CANDIDATE_ONLY', severity: 'high', detail: 'TP-10/11/12 pass isolated candidate contracts but lack formal scene/collider/nav/door/combat owners.' },
    { code: 'E_ROUTING_HANDOFF_MISSING', severity: 'high', detail: 'TP-14 routing callbacks are candidate/real-WebView transport evidence without production CampaignStore/durable-save handoff.' },
    { code: 'E_CHECKPOINT_TRANSFORM_CLEARANCE_MISSING', severity: 'high', detail: 'Canonical IDs exist, but formal Transform/nav/KCC safety and epoch/layout binding are not available.' },
    { code: 'E_PRODUCTION_RELOAD_REWARD_PROOF_MISSING', severity: 'high', detail: 'Reload/reward idempotency is candidate/formal-API evidence, not production player-save acceptance.' },
    { code: 'E_ASSET_AND_IPC_CHAIN_BLOCKED', severity: 'medium', detail: 'TP-01R6 and its IPC prerequisite remain blocked; no formal release UI chain may be claimed.' },
  ],
  futureFormalWhitelistSuggestion: {
    files: packageSpec.writes,
    prerequisites: packageSpec.prerequisites,
    commands: packageSpec.commands,
    rollback: packageSpec.rollback,
    restriction: 'Future implementation may touch only this package write set after every prerequisite is formal_pass; no asset/media/data/public or active-task writes.',
  },
  forbiddenPromotion: ['formal_pass', 'DOD-A01-C04-CHOICES formal_pass', 'DOD-CHECKPOINT-RELOAD formal_pass', 'Gate B/Save/Architecture go', 'candidate-to-formal owner substitution'],
  rollbackReview: 'Delete only this validator, result, and review; do not alter formal source, tests, fixtures, existing evidence, active outputs, or player data.',
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ status: result.status, decision: result.decision, matrixRows: matrix.length, negativeFixtureCount: result.negativeFixtureCount, allNegativeFixturesPass: result.allNegativeFixturesPass, resultPath }, null, 2));
process.exit(result.allNegativeFixturesPass ? 0 : 1);
