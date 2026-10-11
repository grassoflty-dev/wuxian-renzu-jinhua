import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_AUTHORITY_CUTOVER_PLAN.md');
const plan = fs.readFileSync(planPath, 'utf8');
const current = {
  campaignMain: fs.readFileSync(path.join(root, 'server-rs', 'src', 'campaign_main.rs'), 'utf8'),
  campaignStore: fs.readFileSync(path.join(root, 'server-rs', 'src', 'campaign_store.rs'), 'utf8'),
  campaign: fs.readFileSync(path.join(root, 'server-rs', 'src', 'campaign.rs'), 'utf8'),
  worldV3: fs.readFileSync(path.join(root, 'server-rs', 'src', 'world_v3.rs'), 'utf8'),
  tp06a: fs.readFileSync(path.join(root, 'server-rs', 'src', 'tp06a_heartbeat.rs'), 'utf8'),
  app: fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'js', 'app.js'), 'utf8'),
  renderer: fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'js', 'tps-renderer.js'), 'utf8'),
};
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const requiredPaths = [
  'server-rs/src/authority_cutover.rs', 'server-rs/src/lib.rs', 'server-rs/src/campaign_main.rs',
  'server-rs/src/campaign_store.rs', 'server-rs/src/world_v3.rs', 'server-rs/src/tp06a_heartbeat.rs',
  'server-rs/src/save_v3_migration.rs', 'server-rs/src/save_v3_persistence.rs',
  'server-rs/release-ui/js/app.js', 'server-rs/release-ui/js/tps-renderer.js',
  'server-rs/tests/authority_cutover_test.rs', 'server-rs/tests/save_v3_cutover_test.rs',
];
check('manifest_exact_paths', requiredPaths.every((p) => plan.includes(p)), 'F6 precise file manifest');
check('current_inventory_anchors', ['CampaignStore', 'campaign_view/action/save', 'TP06A', 'WorldView', 'base_grid/tiles/px/py', 'save_v3_migration', 'save_v3_persistence'].every((x) => plan.includes(x)), 'current authority inventory');
check('current_symbol_anchors', current.campaignMain.includes('struct Session') && current.campaignMain.includes('campaign_view') && current.campaignMain.includes('campaign_action') && current.campaignMain.includes('campaign_save') && current.campaignMain.includes('invoke_handler') && current.campaignStore.includes('pub struct CampaignStore') && current.campaignStore.includes('pub fn view') && current.campaignStore.includes('pub fn action') && current.campaignStore.includes('pub fn flush') && current.campaign.includes('fn base_grid') && current.worldV3.includes('pub struct WorldView') && current.tp06a.includes('pub fn validate_view') && current.app.includes('campaign_view') && current.app.includes('tp06a_start') && current.renderer.includes('tiles') && current.renderer.includes('px'), 'current source symbol anchors verified');
check('phases_complete', ['P0 prerequisites', 'P1 shadow-read only', 'P2 deterministic comparison', 'P3 preview UI', 'P4 command-routing dry run', 'P5 explicit single-owner switch', 'P6 observation window', 'P7 legacy runtime disconnect', 'P8 rollback window close'].every((x) => plan.includes(x)), 'all cutover phases present');
check('phase_controls', ['entry', 'exit evidence', 'rollback', 'Long-term dual write is forbidden'].every((x) => plan.includes(x)), 'phase entry/exit/rollback and no long-term dual write');
check('shadow_read_only', plan.includes('shadow-read only') && plan.includes('never write both'), 'shadow is read-only');
check('single_owner', plan.includes('one immutable startup configuration') && plan.includes('No command may mutate it at runtime') && plan.includes('exactly one write target'), 'single explicit immutable owner');
check('fail_closed', plan.includes('There is no fallback to legacy, HTTP, Three or Side2D'), 'no silent fallback');
check('parity_matrix', ['Transform', 'Velocity', 'HP / energy', 'Doors / triggers / zones', 'Objectives', 'Pickups', 'NPC', 'Gate', 'Events', 'Save identity'].every((x) => plan.includes(x)), 'required comparison domains');
check('tolerance_rules', plan.includes('exact') && plan.includes('no generic numeric tolerance'), 'field-specific exactness and bounded tolerance');
check('save_cutover', ['v2 backup', 'v2→v3 exactly once', 'only writable format', 'unknown outcome'].every((x) => plan.includes(x)), 'Save v3 migration and unknown-outcome rules');
check('gate_b_choice_npc', ['hive_lockdown', 'choice is optional', 'NPC independent'].every((x) => plan.includes(x)), 'Gate B/choice/NPC semantics');
check('ui_renderer_boundary', ['WorldView snapshots', 'stale snapshots are dropped', 'never replays action', 'Tauri is the only packaged transport'].every((x) => plan.includes(x)), 'read-only UI/Babylon boundary');
check('f7_grid_gate', plan.includes('F7 is the hard gate') && plan.includes('zero formal reachability'), 'legacy Grid disconnect conditions');
check('three_world_rollout', ['Grey Hive first', 'Mist Harbor next', 'Clockworks last', 'generic Babylon shell coverage is not completion evidence'].every((x) => plan.includes(x)), 'per-world staged rollout');
check('nb15r1_not_assumed', plan.includes('NB-15R1 路径修复未完成'), 'NB-15R1 remains prerequisite/disposition');
check('planning_only', plan.includes('不执行 cutover') && plan.includes('不修改正式代码') && plan.includes('不 claim') || plan.includes('planning package only'), 'not implemented/cutover complete');
check('forbidden_tp_reference', !plan.includes('TP-18D2'), 'no forbidden reference');

const result = {
  schema: 'new_baseline_first_formal_authority_cutover_nb19_v1',
  status: 'authority_cutover_implementation_ready_for_gate_review',
  conclusion: 'authority_cutover_implementation_ready_for_gate_review',
  implemented: false,
  cutover_complete: false,
  shadow_read_only: true,
  single_owner_switch: true,
  long_term_dual_write: false,
  no_silent_fallback: true,
  dependencies: ['F1', 'F2', 'F3', 'F4', 'F5', 'NB-15 scope disposition'],
  excluded_active_inputs: ['NB-15R1', 'NB-18'],
  checks,
  manifest: requiredPaths,
  phases: ['P0 prerequisites', 'P1 shadow-read only', 'P2 deterministic comparison', 'P3 preview UI', 'P4 command-routing dry run', 'P5 explicit single-owner switch', 'P6 observation window', 'P7 legacy runtime disconnect', 'P8 rollback window close'],
  rollout: ['grey_hive', 'mist_harbor', 'clockworks'],
  gate_b: 'hive_lockdown',
  choice_policy: 'optional',
  npc_policy: 'independent',
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
