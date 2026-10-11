import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'FIRST_FORMAL_CHARACTER_PRODUCTION_GATE_PLAN.md'), 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const futureFiles = [
  'tools/character-production-gate/schema/character-production-gate-v1.schema.json',
  'tools/character-production-gate/manifest/tier-s-production-gate-v1.json',
  'tools/character-production-gate/validate.mjs',
  'tools/character-production-gate/tests/production-gate-fixtures.json',
  'tools/character-production-gate/README.md',
];
const stages = ['concept_received', 'human_approved', 'model', 'topology_uv', 'material', 'rig', 'animation', 'lod_collider_socket', 'runtime_metadata_valid', 'registry_candidate', 'runtime_smoke'];
check('manifest_exact_paths', futureFiles.every((p) => plan.includes(p)), 'precise future five-file gate manifest');
check('stage_graph', stages.every((x) => plan.includes(x)) && plan.includes('concept_received\n  -> human_approved'), 'ordered stage graph');
check('stage_controls', ['input', 'evidence', 'reviewer', 'pass', 'fail', 'rollback', 'supersession'].every((x) => plan.includes(x)), 'per-stage evidence/reviewer/pass/fail/rollback/supersession');
check('three_roles', ['Cen Yao', 'Bai Zhi', 'Sentinel', 'ASSET.SHARED.CHARACTER.CEN_YAO', 'ASSET.SHARED.CHARACTER.BAI_ZHI', 'ASSET.SHARED.CHARACTER.SENTINEL'].every((x) => plan.includes(x)), 'three independent role checklists');
check('sentinel_extensions', ['socket_weapon_primary', 'socket_weapon_secondary', 'attack/hit/stagger/disabled/death', 'collision proxy'].every((x) => plan.includes(x)), 'Sentinel structure/attack/collision/socket requirements');
check('human_boundary', ['人工 sign-off', '视觉质量', '自动工具只能验证', '不得读取媒体'].every((x) => plan.includes(x)), 'human approval vs static automation boundary');
check('lineage_immutable', ['Asset ID is immutable', 'Hash/version drift is a hard fail', 'predecessor/successor', 'no silent repair or overwrite'].every((x) => plan.includes(x)), 'lineage/version immutability');
check('serial_gates', ['production_approved', 'registryApproval=runtime_approved', 'evidenceState=accepted', 'Registry, license and packaging are serial gates'].every((x) => plan.includes(x)), 'Registry/license/packaging serial prerequisites');
check('negative_cases', ['skipped `human_approved`', 'hash/version drift', 'role/Asset ID mismatch', 'missing/unapproved dependency', 'placeholder/final ID collision', 'expired license evidence', 'fake hash promoted to candidate'].every((x) => plan.includes(x)), 'required negative fixtures');
check('runtime_boundary', ['不读取、生成、转换、复制、导入或评价媒体', '不批准资产', 'does not claim production complete', 'runtime smoke'].every((x) => plan.includes(x)), 'non-goal/runtime boundary');
check('persistent_requirements', plan.includes('USER_PERSISTENT_REQUIREMENTS.md') && plan.includes('用户手工'), 'persistent media constraints carried forward');
check('future_files_absent', futureFiles.every((p) => !fs.existsSync(path.join(root, p))), 'future gate files not created');
check('forbidden_tp_reference', !plan.includes(['TP-', '18D2'].join('')), 'no forbidden TP reference');

const result = {
  schema: 'new_baseline_first_formal_character_production_gate_nb30_v1',
  status: 'character_production_gate_implementation_ready_for_gate_review',
  conclusion: 'character_production_gate_implementation_ready_for_gate_review',
  implemented: false,
  productionComplete: false,
  assetApproved: false,
  runtimeCandidateAccepted: false,
  runtimeSmokePassed: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaConverted: false,
  mediaImported: false,
  network: false,
  modelCalled: false,
  roles: ['cen_yao', 'bai_zhi', 'sentinel'],
  stages,
  prerequisites: ['NB-21 registry', 'NB-24 intake', 'NB-26 packaging gate', 'NB-28 runtime metadata', 'human approval'],
  excluded_active_inputs: ['NB-22', 'NB-27'],
  manifest: futureFiles,
  checks,
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
