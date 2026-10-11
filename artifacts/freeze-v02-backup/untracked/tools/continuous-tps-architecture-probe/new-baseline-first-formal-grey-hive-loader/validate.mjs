import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'FIRST_FORMAL_GREY_HIVE_LOADER_SLICE_PLAN.md'), 'utf8');
const lib = fs.readFileSync(path.join(root, 'server-rs', 'src', 'lib.rs'), 'utf8');
const currentWorldV3 = fs.readFileSync(path.join(root, 'server-rs', 'src', 'world_v3.rs'), 'utf8');
const currentCampaignMain = fs.readFileSync(path.join(root, 'server-rs', 'src', 'campaign_main.rs'), 'utf8');
const currentCampaignStore = fs.readFileSync(path.join(root, 'server-rs', 'src', 'campaign_store.rs'), 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const manifest = [
  'server-rs/src/grey_hive_authoring.rs',
  'server-rs/src/grey_hive_loader.rs',
  'server-rs/src/grey_hive_definition.rs',
  'server-rs/src/lib.rs',
  'server-rs/tests/grey_hive_loader_test.rs',
  'server-rs/tests/grey_hive_loader_fixture_test.rs',
];
check('manifest_exact_paths', manifest.every((p) => plan.includes(p)), 'precise six-file future manifest');
check('rust_only_pure_data', plan.includes('Rust-only pure-data') && plan.includes('不可变纯数据定义'), 'Rust-only pure-data boundary');
check('schema_versioned', plan.includes('grey_hive_authoring_v1') && plan.includes('contentVersion') && plan.includes('sourceSha256'), 'version/content/provenance schema');
check('required_types', ['Zone', 'Room', 'Connector', 'Door', 'Trigger', 'Checkpoint', 'Landmark', 'Route', 'NavHint'].every((x) => plan.includes(x)), 'authoring object schema');
check('counts', ['18 rooms', '17 connectors', '17 doors', '15 triggers', '6 checkpoints'].every((x) => plan.includes(x)), 'golden fixture counts');
check('bounds', ['100×100m', '60×80m', '[0,-3,0]–[100,15,100]', '[16,-3,8]–[76,15,88]'].every((x) => plan.includes(x)), 'site/playable bounds');
check('ids_refs', plan.includes('E_DUPLICATE_ID') && plan.includes('E_DANGLING_REFERENCE') && plan.includes('resolve_references'), 'stable ID/reference validation');
check('finite', plan.includes('E_NON_FINITE') && plan.includes('E_OUT_OF_BOUNDS') && plan.includes('E_INVALID_AABB'), 'finite and geometry validation');
check('gate_contract', ['Gate A 只依赖 `hive_power=true`', 'Gate B 只依赖 `hive_lockdown=true`', 'hive_choice', 'NPC（含 `hive_baizhi`）不拥有 Gate B 解锁权'].every((x) => plan.includes(x)), 'Gate A/B choice/NPC contract');
check('route_not_navmesh', plan.includes('candidate_target_band') && plan.includes('E_ROUTE_FORMAL_NAVMESH') && plan.includes('不冒充'), 'route evidence boundary');
check('loader_api', ['decode(source:', 'validate(authoring:', 'resolve_references', 'build_definition', 'signature(', 'diagnostics()', 'dispose_staging'].every((x) => plan.includes(x)), 'loader lifecycle API');
check('determinism', plan.includes('DefinitionSignature') && plan.includes('same `DefinitionSignature`'), 'canonical deterministic signature');
check('pure_output_boundary', ['Save handle', 'Tauri transport', 'Babylon mesh/material/camera', 'Grid/tile/`px`/`py`/`zone2d`'].every((x) => plan.includes(x)) && plan.includes('contains no dynamic door state'), 'pure output exclusions');
check('default_off', plan.includes('不注册当前 world entry') && plan.includes('默认不注册到当前 world entry') || plan.includes('default-off/no current world-entry registration'), 'default-off/no entry registration');
check('serial_dependencies', ['F1 `WorldV3Owner`', 'F3 `continuous_spatial`', 'F4 Save', 'F5/F6'].every((x) => plan.includes(x)), 'F1/F3/F4/F5/F6 serial points');
check('current_conflict_scan', lib.includes('pub mod world_v3') && currentWorldV3.includes('WorldView') && currentCampaignMain.includes('CampaignStore') && currentCampaignMain.includes('campaign_view') && currentCampaignStore.includes('pub struct CampaignStore') && !fs.existsSync(path.join(root, 'server-rs', 'src', 'grey_hive_loader.rs')), 'current modules scanned; proposed loader absent');
check('current_unchanged_boundary', plan.includes('CampaignStore unchanged') && plan.includes('不修改正式代码或数据'), 'current gameplay unchanged');
check('negative_tests', ['unknown version/content', 'duplicate ID and dangling reference', 'route-as-navmesh rejection', 'Grid/runtime-field deny scan', 'no F1/F3 runtime caller'].every((x) => plan.includes(x)), 'negative test matrix');
check('planning_only', plan.includes('不实现 loader') && plan.includes('not `implemented` or `loaded`'), 'not implemented/loaded');
check('forbidden_tp_reference', !plan.includes('TP-18D2'), 'no forbidden reference');

const result = {
  schema: 'new_baseline_first_formal_grey_hive_loader_nb20_v1',
  status: 'grey_hive_loader_implementation_ready_for_gate_review',
  conclusion: 'grey_hive_loader_implementation_ready_for_gate_review',
  implemented: false,
  loaded: false,
  rust_only: true,
  pure_data_only: true,
  default_off: true,
  current_world_entry_registered: false,
  counts: { rooms: 18, connectors: 17, doors: 17, triggers: 15, checkpoints: 6 },
  bounds_m: { site: [100, 100], playable: [60, 80] },
  dependencies: ['NB-02', 'NB-05', 'NB-08', 'NB-11', 'NB-12', 'NB-13', 'NB-16', 'NB-19', 'F1/F3 API freeze'],
  excluded_active_inputs: ['NB-15R1', 'NB-18'],
  checks,
  manifest,
  route_evidence: 'candidate_target_band_only',
  forbidden_outputs: ['Grid', 'tile', 'px', 'py', 'zone2d', 'Babylon mesh/material/camera', 'UI/DOM', 'Tauri transport', 'Save runtime handles', 'collider bake', 'navmesh', 'AI'],
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
