import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'FIRST_FORMAL_CHARACTER_INTAKE_SLICE_PLAN.md'), 'utf8');
const index = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'index.html'), 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const futureFiles = [
  'tools/character-intake/schema/character-intake-v1.schema.json',
  'tools/character-intake/manifest/tier-s-intake-v1.json',
  'tools/character-intake/validate.mjs',
  'tools/character-intake/tests/character-intake-fixtures.json',
  'tools/character-intake/README.md',
];
const roles = [
  ['cen-yao', 'ASSET.SHARED.CHARACTER.CEN_YAO', 'cen-yao_concept_front_v01.<ext>', 'cen-yao_concept_v01.meta.json'],
  ['bai-zhi', 'ASSET.SHARED.CHARACTER.BAI_ZHI', 'bai-zhi_concept_front_v01.<ext>', 'bai-zhi_concept_v01.meta.json'],
  ['sentinel', 'ASSET.SHARED.CHARACTER.SENTINEL', 'sentinel_concept_front_v01.<ext>', 'sentinel_concept_v01.meta.json'],
];
check('manifest_exact_paths', futureFiles.every((p) => plan.includes(p)), 'precise future five-file intake manifest');
check('three_roles', roles.flat().every((x) => plan.includes(x)) && ['Cen Yao', 'Bai Zhi', 'Sentinel'].every((x) => plan.includes(x)), 'three Tier S roles and IDs');
check('directory_naming', ['user-intake/concepts/cen-yao/', 'user-intake/concepts/bai-zhi/', 'user-intake/concepts/sentinel/'].every((x) => plan.includes(x)), 'expected directories');
check('view_naming', ['front_v01', 'side_v01', 'back_v01'].every((x) => plan.includes(x)), 'front/side/back naming');
check('sidecar_schema', ['schemaVersion', 'intakeVersion', 'files', 'sha256', 'sizeBytes', 'source', 'authorOrOwner', 'exactVersionOrDelivery', 'generatedOrDeliveredAt', 'promptOrInputRecord', 'license', 'approval', 'productionHandoff'].every((x) => plan.includes(x)), 'sidecar metadata fields');
check('status_machine', ['missing -> received_unverified -> metadata_valid -> production_approved', 'rejected', 'superseded'].every((x) => plan.includes(x)), 'intake state machine');
check('human_quality_boundary', ['视觉质量', '人工审批', 'never auto-passed'].every((x) => plan.includes(x)), 'visual quality not automated');
check('production_handoff', ['root', 'pelvis', 'spine_01', 'socket_hand_l', '`idle`', '`ability_secondary`', 'LOD0 ≤80k', '≤4 material slots', 'GLB deliverable'].every((x) => plan.includes(x)), 'model production handoff');
check('runtime_gate', ['runtime_candidate', 'runtime_approved', 'no automatic packaging', 'fail closed'].every((x) => plan.includes(x)), 'registry runtime gate');
check('media_prohibitions', ['不生成', '查看、读取', '不转换', '不修改 NB-21', 'media bytes were opened'].every((x) => plan.includes(x)), 'media read/create/convert prohibition');
check('current_reference_inventory', index.includes('assets/characters/cenyao-portrait-wan-v1.png') && plan.includes('portraits_library/'), 'current text/path inventory');
check('future_files_absent', futureFiles.every((p) => !fs.existsSync(path.join(root, p))), 'future intake files not created');
check('planning_only', plan.includes('does not claim intake implemented') && plan.includes('runtime approved'), 'not implemented/approved');
check('forbidden_tp_reference', !plan.includes(['TP-', '18D2'].join('')), 'no forbidden TP reference');

const result = {
  schema: 'new_baseline_first_formal_character_intake_nb24_v1',
  status: 'character_intake_implementation_ready_for_gate_review',
  conclusion: 'character_intake_implementation_ready_for_gate_review',
  implemented: false,
  media_read: false,
  media_created: false,
  media_converted: false,
  media_model_called: false,
  runtime_connected: false,
  roles: ['cen_yao', 'bai_zhi', 'sentinel'],
  states: ['missing', 'received_unverified', 'metadata_valid', 'production_approved', 'rejected', 'superseded'],
  runtime_gate: 'registry_runtime_approved_only',
  manifest: futureFiles,
  checks,
  current_inventory: {
    user_intake_present: false,
    portraits_library_present: true,
    current_release_cenyao_reference: 'assets/characters/cenyao-portrait-wan-v1.png',
    media_bytes_read: false
  },
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
