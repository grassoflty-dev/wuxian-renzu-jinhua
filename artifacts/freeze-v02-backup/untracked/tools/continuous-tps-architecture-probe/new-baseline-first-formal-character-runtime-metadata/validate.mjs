import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'FIRST_FORMAL_CHARACTER_RUNTIME_METADATA_SLICE_PLAN.md'), 'utf8');
const index = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'index.html'), 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const futureFiles = [
  'tools/character-runtime-metadata/schema/character-runtime-metadata-v1.schema.json',
  'tools/character-runtime-metadata/manifest/tier-s-runtime-metadata-v1.json',
  'tools/character-runtime-metadata/validate.mjs',
  'tools/character-runtime-metadata/tests/character-runtime-metadata-fixtures.json',
  'tools/character-runtime-metadata/README.md',
];
check('manifest_exact_paths', futureFiles.every((p) => plan.includes(p)), 'precise future five-file metadata manifest');
check('three_roles', ['cen_yao', 'bai_zhi', 'sentinel', 'ASSET.SHARED.CHARACTER.CEN_YAO', 'ASSET.SHARED.CHARACTER.BAI_ZHI', 'ASSET.SHARED.CHARACTER.SENTINEL'].every((x) => plan.includes(x)), 'three role metadata fixtures');
check('metadata_envelope', ['schemaVersion', 'characterId', 'modelAssetId', 'modelVersion', 'modelHash', 'sourceWorkRuntimeLineage', 'transform', 'skeleton', 'animations', 'materials', 'lod', 'collider', 'sockets', 'bounds', 'approval'].every((x) => plan.includes(x)), 'metadata envelope fields');
check('canonical_bones_sockets', ['root', 'pelvis', 'spine_01', 'neck', 'head', 'socket_hand_l', 'socket_hand_r', 'socket_back', 'socket_head'].every((x) => plan.includes(x)), 'canonical bones and sockets');
check('role_extensions', ['socket_beacon', 'socket_medkit', 'socket_weapon_primary', 'socket_weapon_secondary', 'beacon_sync', 'heal_cast', 'sentry_charge'].every((x) => plan.includes(x)), 'role-specific extensions');
check('required_animations', ['idle', 'walk', 'run', 'jump', 'attack', 'hit_react', 'death', 'interact_start', 'interact_loop', 'interact_end', 'ability_primary', 'ability_secondary'].every((x) => plan.includes(x)), 'required animation set');
check('material_registry_refs', ['materialAssetId', 'textureAssetIds[]', 'allReferencesUseRegistryIds', '禁止裸路径'].every((x) => plan.includes(x)), 'Registry-only material/texture references');
check('lod_rules', ['LOD0 ≤80k', 'LOD1 ≤40k', 'LOD2 ≤15k', 'material slots', '纹理目标预算'].every((x) => plan.includes(x)), 'LOD/material budgets');
check('collider_socket_bounds', ['collision proxy', 'one collider proxy', 'aabbMinM', 'aabbMaxM', 'groundContactM', 'scale≠1'].every((x) => plan.includes(x)), 'collider/socket/bounds validation');
check('approval_gate', ['runtime_approved', 'hash/version drift', 'unapproved/rejected/deprecated asset', 'Registry `runtime_approved`'].every((x) => plan.includes(x)), 'approval/hash/version gate');
check('fake_hash_fixture', plan.includes('sha256:PLACEHOLDER_NOT_VERIFIED') && plan.includes('假 hash'), 'fake hash placeholders only');
check('fail_closed', ['必须拒绝', 'missing canonical bone', 'missing required animation', 'duplicate/unresolved socket', '非有限或越界 bounds'].every((x) => plan.includes(x)), 'fail-closed negative rules');
check('no_media_boundary', ['不读取 GLB/图片/音频字节', '不生成、转换、复制、导入或修改媒体', 'mediaBytesRead=false', '不打开 GLB'].every((x) => plan.includes(x)), 'media bytes prohibited');
check('current_reference_inventory', index.includes('assets/characters/cenyao-portrait-wan-v1.png') && plan.includes('server-rs/release-ui/assets/characters/'), 'current text/path reference only');
check('future_files_absent', futureFiles.every((p) => !fs.existsSync(path.join(root, p))), 'future metadata files not created');
check('runtime_boundary', plan.includes('不接 Babylon/runtime') && plan.includes('不实现 loader'), 'no loader/runtime integration');
check('planning_only', plan.includes('不是 implemented') && plan.includes('runtime-integrated'), 'planning only');
check('forbidden_tp_reference', !plan.includes(['TP-', '18D2'].join('')), 'no forbidden reference');

const result = {
  schema: 'new_baseline_first_formal_character_runtime_metadata_nb28_v1',
  status: 'character_runtime_metadata_implementation_ready_for_gate_review',
  conclusion: 'character_runtime_metadata_implementation_ready_for_gate_review',
  implemented: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaConverted: false,
  mediaImported: false,
  runtimeConnected: false,
  glbValidated: false,
  roles: ['cen_yao', 'bai_zhi', 'sentinel'],
  registryGate: 'runtime_approved_only',
  fakeHashFixtures: true,
  manifest: futureFiles,
  checks,
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
