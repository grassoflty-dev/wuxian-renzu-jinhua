import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'FIRST_FORMAL_ASSET_REGISTRY_SLICE_PLAN.md'), 'utf8');
const tauri = fs.readFileSync(path.join(root, 'server-rs', 'tauri.conf.json'), 'utf8');
const index = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'index.html'), 'utf8');
const side2d = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'js', 'hive-side2d.js'), 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const manifest = [
  'tools/asset-registry/schema/asset-registry-v1.schema.json',
  'tools/asset-registry/manifest/asset-registry-v1.json',
  'tools/asset-registry/validate.mjs',
  'tools/asset-registry/tests/asset-registry-fixtures.json',
  'tools/asset-registry/README.md',
];
const forbiddenToken = ['TP-', '18D2'].join('');
check('manifest_exact_paths', manifest.every((p) => plan.includes(p)), 'precise offline five-file future manifest');
check('nb03_required_fields', ['assetId:', 'source:', 'author:', 'exactVersion:', 'licenseEvidence:', 'sourceRedistribution:', 'targetWorld:', 'purpose:', 'modification:', 'runtimePath:', 'approvalStatus:', 'stableAuditRecord:'].every((x) => plan.includes(x)), 'all 12 mandatory audit fields');
check('asset_id_rules', ['ASSET.<WORLD_SCOPE>.<CATEGORY>.<SLUG>', 'immutable', 'duplicate ID', 'predecessor/successor'].every((x) => plan.includes(x)), 'stable ID/version replacement rules');
check('schema_extensions', ['assetClass', 'sourcePath', 'workPath', 'runtimeDerivativePath', 'contentHash', 'dependencies[]', 'optionalVariants[]', 'lod[]', 'worldScope', 'approvalHistory[]'].every((x) => plan.includes(x)), 'lineage/hash/dependency/variant fields');
check('approval_state_machine', ['proposed', 'license_pending', 'approved_source', 'work_in_progress', 'runtime_candidate', 'runtime_approved', 'rejected', 'deprecated'].every((x) => plan.includes(x)), 'approval states present');
check('license_evidence', ['licenseName', 'textOrCapturePath', 'sourceReference', 'acquiredAt', 'evidenceHash', 'commercial-use permission', 'attribution'].every((x) => plan.includes(x)), 'license evidence requirements');
check('fail_closed', plan.includes('fail closed') && plan.includes('Unknown license') && plan.includes('missing provenance') && plan.includes('prevents `runtime_approved`'), 'unknown/missing evidence fails closed');
check('path_packaging', plan.includes('runtime/assets/') && plan.includes('contain no `..`') && plan.includes('no drive/UNC/absolute prefix') && plan.includes('no URL scheme') && plan.includes('never automatically copied into runtime packaging'), 'runtime path and packaging boundary');
check('dependency_dag', ['directed acyclic graph', 'skeleton -> animation -> material -> texture', 'Cycles', 'cross-world dependency'].every((x) => plan.includes(x)), 'dependency graph rules');
check('world_scope', ['grey_hive', 'mist_harbor', 'clockworks', 'shared', 'world-specific asset cannot be referenced by another world'].every((x) => plan.includes(x)), 'three worlds and shared scope');
check('tier_s_text_only', ['ASSET.SHARED.CHARACTER.CEN_YAO', 'ASSET.SHARED.CHARACTER.BAI_ZHI', 'ASSET.SHARED.CHARACTER.SENTINEL', 'ASSET.CENTRAL_SHAFT.ENVIRONMENT.CORE', 'ASSET.GATE_A.ENVIRONMENT.ENTRY', 'ASSET.BEACON.INTERACTION.SIGNAL'].every((x) => plan.includes(x)) && plan.includes('文本记录'), 'Tier S examples are text-only');
check('offline_no_runtime', plan.includes('离线、可审计') && plan.includes('默认不改变运行时行为') && plan.includes('does not alter Tauri bundle configuration'), 'offline/no runtime connection');
check('current_inventory_anchors', tauri.includes('frontendDist') && index.includes('assets/characters/cenyao-portrait-wan-v1.png') && side2d.includes('assets/hive2d/'), 'current packaging and reference inventory');
check('media_prohibition', ['不下载', '不生成', '不转换', '不复制', '不提交任何媒体', '不读取媒体字节'].every((x) => plan.includes(x)), 'media operations prohibited');
check('negative_matrix', ['duplicate ID', 'path traversal', 'dependency cycle', 'placeholder/final collision', 'unapproved packaged asset', 'deterministic manifest signature'].every((x) => plan.includes(x)), 'negative and determinism matrix');
check('not_implemented', plan.includes('不实现、不 enforced') && plan.includes('does not claim Registry implemented or enforced'), 'planning only');
check('future_files_absent', manifest.every((p) => !fs.existsSync(path.join(root, p))), 'future registry files not created');
check('forbidden_tp_reference', !plan.includes(forbiddenToken), 'no forbidden reference');

const result = {
  schema: 'new_baseline_first_formal_asset_registry_nb21_v1',
  status: 'asset_registry_implementation_ready_for_gate_review',
  conclusion: 'asset_registry_implementation_ready_for_gate_review',
  implemented: false,
  enforced: false,
  offline_only: true,
  runtime_connected: false,
  media_created_or_processed: false,
  worlds: ['grey_hive', 'mist_harbor', 'clockworks'],
  shared_scope_supported: true,
  required_audit_fields: 12,
  tier_s_examples: ['Cen Yao', 'Bai Zhi', 'Sentinel', 'Central Shaft', 'Gate A', 'Beacon'],
  manifest,
  excluded_active_inputs: ['active/unknown packages', 'TP-15C3', 'TP-18D1'],
  checks,
  current_inventory: {
    roots: ['game/assets', 'server-rs/release-ui/assets', 'server-rs/ui/assets', 'tools/asset_src'],
    tauri_frontend_dist: './release-ui',
    direct_references: ['assets/characters/cenyao-portrait-wan-v1.png', 'assets/hive2d/*'],
    media_bytes_read: false
  }
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
