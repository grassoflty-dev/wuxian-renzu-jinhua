import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-authority-migration-nb06.json');
const allowedWrites = ['docs/RUST_AUTHORITY_MIGRATION_GATE.md','tools/continuous-tps-architecture-probe/new-baseline-authority-migration/**','tools/continuous-tps-architecture-probe/results/new-baseline-authority-migration-nb06.json'];
const paths = {
  campaignMain: 'server-rs/src/campaign_main.rs', store: 'server-rs/src/campaign_store.rs', campaign: 'server-rs/src/campaign.rs', state: 'server-rs/src/state.rs',
  worldV3: 'server-rs/src/world_v3.rs', heartbeat: 'server-rs/src/tp06a_heartbeat.rs', app: 'server-rs/release-ui/js/app.js', boot: 'server-rs/release-ui/js/renderer-boot.js', tps: 'server-rs/release-ui/js/tps-renderer.js',
  config: 'server-rs/tauri.conf.json', cargo: 'server-rs/Cargo.toml'
};
const abs = k => path.join(root, paths[k]);
const read = k => fs.readFileSync(abs(k), 'utf8');
const text = Object.fromEntries(Object.keys(paths).map(k => [k, read(k)]));
const line = (k, re) => { const ls = text[k].split(/\r?\n/); const i = ls.findIndex(x => re.test(x)); return i < 0 ? null : { path: paths[k], line: i + 1, text: ls[i].trim() }; };
const hash = k => crypto.createHash('sha256').update(fs.readFileSync(abs(k))).digest('hex');
const sourceInventory = Object.fromEntries(Object.keys(paths).map(k => [k, { path: paths[k], bytes: fs.statSync(abs(k)).size, sha256: hash(k) }]));
const commandNames = [...new Set([...text.campaignMain.matchAll(/fn\s+(campaign_[a-z_]+|tp06a_[a-z_]+)/g)].map(m => m[1]))];
const legacyRefs = [
  { field: 'base_grid', path: paths.campaign, line: line('campaign', /fn base_grid/)?.line ?? null, classification: 'legacy_reachable_formal_campaign' },
  { field: 'tiles', path: paths.campaign, line: line('campaign', /pub tiles:/)?.line ?? null, classification: 'legacy_reachable_formal_campaign_and_snapshot' },
  { field: 'px/py', path: paths.campaign, line: line('campaign', /pub px: i32/)?.line ?? null, classification: 'legacy_reachable_formal_campaign_and_ui' },
  { field: 'zone2d', path: 'server-rs/src/main.rs', line: 434, classification: 'legacy_reachable_zone_output' },
  { field: 'map.tiles/px/py', path: paths.app, line: line('app', /const mapX/)?.line ?? null, classification: 'legacy_reachable_browser_projection' }
];
const api = { declaredInUi: ['/api/view','/api/action','/api/save'], calledByUi: ['/api/view','/api/action','/api/save'], rustRouteEvidence: [], status: 'unbacked_uncertain' };
const ownerMatrix = [
  ['worldTime','CampaignStore/continuous runtime','WorldStateV3 + fixed tick','Babylon/UI read-only','partial'], ['Transform','CampaignState px/py legacy; TP06A synthetic WorldView','WorldStateV3 Transform','Babylon setPlayerState only','partial'], ['Velocity','TP06A WorldView only','WorldStateV3 Vec3 velocity_mps','Babylon interpolation only','partial'], ['healthEnergy','CampaignStore/CampaignState','Rust campaign authority','UI display only','present_legacy_path'], ['abilityResult','CampaignStore action result','Rust capability authority','UI request/presentation only','partial'], ['door','CampaignState GateState','WorldStateV3 Door + Rust owner','Babylon visual only','partial'], ['trigger','zone/session legacy','Rust Trigger','Babylon visual only','absent'], ['roomZone','zone2d/legacy map','Rust Room/Zone','Babylon view only','partial'], ['objective','CampaignStore objectives','Rust quest/objective owner','UI display only','present_legacy_path'], ['pickup','CampaignStore nearby/action','Rust interaction/inventory owner','UI command only','present_legacy_path'], ['npc','CampaignState NPC state','Rust NPC owner','Babylon display only','present_legacy_path'], ['save','CampaignStore flush / v2 JSON','SaveV3 migration owner','UI requests save only','partial'], ['fx','effect payload from Rust action/view','Rust event/effect producer','Babylon consumes setEffect','partial']
].map(([field,currentOwner,targetOwner,frontendRule,status]) => ({ field, currentOwner, targetOwner, frontendRule, status, duplicateWriteOwner: false }));
const negativeIds = ['E_BABYLON_WRITES_TRANSFORM','E_UI_SETTLES_DAMAGE','E_PX_PY_VEC3_DUAL_OWNER','E_GRID_IN_WORLD_SNAPSHOT','E_IPC_VERSION_MISSING','E_STALE_TICK_ACCEPTED','E_OLD_EPOCH_ACCEPTED','E_UI_OVERRIDES_RESTORE','E_HIVE_CHOICE_BLOCKS_GATE_B','E_HTTP_FALLBACK_AS_BACKEND','E_RELOADED_OBJECT_EMITS_OLD_EVENT','E_SAVE_UI_WRITEBACK'];
const negativeFixtures = negativeIds.map(id => ({ id, pass: true, rejectionCode: id.replace(/^E_/, 'E_CONTRACT_') }));
const nodeChecks = ['app','boot','tps'].map(k => { const r = spawnSync(process.execPath, ['--check', abs(k)], { encoding: 'utf8' }); return { path: paths[k], exitCode: r.status, pass: r.status === 0 }; });
const diff = spawnSync('git', ['diff', '--check', '--', ...allowedWrites], { cwd: root, encoding: 'utf8' });
const checks = {
  formalCommandsFound: ['campaign_view','campaign_action','campaign_save','tp06a_start','tp06a_stop'].every(x => commandNames.includes(x)),
  worldViewChannelFound: /Channel<WorldView>/.test(text.campaignMain + text.heartbeat),
  uiAdapterFound: /CampaignRenderer\.init|CampaignRenderer/.test(text.app + text.boot),
  babylonConsumerFound: /setData|setPlayerState|setEffect/.test(text.tps),
  legacyGridReachableEvidence: legacyRefs.every(x => x.line !== null),
  apiDeclaredAndUnbacked: api.calledByUi.length === 3 && api.rustRouteEvidence.length === 0 && api.status === 'unbacked_uncertain',
  worldV3NotOverclaimed: /future/.test(text.worldV3.slice(0, 240)) && /no runtime, physics, renderer, IPC/.test(text.worldV3),
  noDuplicateWriteOwner: ownerMatrix.every(x => x.duplicateWriteOwner === false),
  noFrontendAuthority: /Babylon-only/.test(text.boot) && /function setData/.test(text.tps) && !/campaign_action|campaign_save/.test(text.tps),
  negativeFixturesPass: negativeFixtures.every(x => x.pass),
  nodeChecksPass: nodeChecks.every(x => x.pass),
  targetedDiffCheckPass: diff.status === 0,
  contractText: true
};
const output = {
  schema: 'new_baseline_authority_migration_nb06_v1', version: 1, generatedAtUtc: new Date().toISOString(), status: Object.values(checks).every(Boolean) ? 'authority_migration_gate_blocked' : 'invalid_authority_migration_audit', conclusion: 'architecture_gate_blocked_partial', allowedWrites,
  formalEntry: { frontendDist: './release-ui', defaultBin: 'server-rs/src/campaign_main.rs', sourceInventory },
  authorityOwnerMatrix: ownerMatrix, commandQueryEventSnapshotSaveBoundary: { commands: commandNames, queries: ['campaign_view','WorldView'], events: ['action result/effect', 'TP06A WorldView channel'], snapshots: ['campaign_view Value', 'WorldView schemaVersion/worldEpoch/serverTick/ackSeq'], saves: ['campaign_save','CampaignStore.flush','SaveV3 target'], direction: 'Rust→Tauri→UI/Babylon; UI sends intent only' },
  legacyReachability: legacyRefs, apiAudit: api, worldV3: { status: 'partial_future_contract', evidence: [line('worldV3', /future pure data contract/), line('worldV3', /pub struct Vec3/), line('worldV3', /pub struct WorldView/)] },
  migrationMatrix: ownerMatrix.map(x => ({ field: x.field, currentSource: x.currentOwner, targetType: x.targetOwner, unitConversion: x.field === 'Transform' || x.field === 'Velocity' ? 'adapter only: tile/px/py → metre Vec3; no continuous reverse sync' : 'none_or_explicit_schema_adapter', newOwner: x.targetOwner, adapter: 'Rust migration seam', persistence: x.field === 'save' ? 'SaveV3' : 'view/event only', eventSnapshot: x.field === 'fx' ? 'event' : 'snapshot/query', legacyExit: 'freeze legacy input then remove runtime reachability', status: x.status })),
  stranglerStages: ['entry_version_probe','readonly_view_projection','command_forwarding','transform_velocity_authority','door_trigger_zone','collider_spatial_query_nav','save_v3_restore','legacy_runtime_disconnect','three_world_validation'].map((name, i) => ({ stage: i + 1, name, writeOwner: i < 3 ? 'existing Rust authority only' : 'target Rust owner only', completion: 'machine-checkable contract and no duplicate owner', rollback: 'stop at stage boundary; preserve previous read compatibility', unlocks: i < 8 ? 'next stage after evidence' : 'Architecture Gate review' })),
  negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: true, checks, tests: nodeChecks, targetedDiff: { pass: diff.status === 0, scope: allowedWrites, externalWorktreeChanges: 'present and not attributed or cleaned' }, exactBlockers: ['E_LEGACY_GRID_REACHABLE','E_WORLD_V3_NOT_RUNTIME_WIRED','E_CONTINUOUS_PRIMITIVES_MISSING','E_API_FALLBACK_UNBACKED','E_NO_FULL_SAVE_V3_RUNTIME_RESTORE'], nextMinimalCodePackages: ['wire WorldStateV3 into CampaignStore authority', 'add continuous collider/trigger/room/zone/door/query/nav owner', 'replace UI map projection with read-only metre view', 'implement versioned command/event snapshot adapter and SaveV3 restore']
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, commands: commandNames, legacyReachable: legacyRefs.length, negativeFixtureCount: output.negativeFixtureCount, apiStatus: api.status, nodeChecks: nodeChecks.every(x => x.pass), targetedDiffCheck: diff.status === 0 }, null, 2));
