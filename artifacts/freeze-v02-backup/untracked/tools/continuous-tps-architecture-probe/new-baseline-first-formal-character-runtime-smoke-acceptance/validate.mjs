import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'FIRST_FORMAL_CHARACTER_RUNTIME_SMOKE_ACCEPTANCE.md'), 'utf8');
const index = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'index.html'), 'utf8');
const boot = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'js', 'renderer-boot.js'), 'utf8');
const renderer = fs.readFileSync(path.join(root, 'server-rs', 'release-ui', 'js', 'tps-renderer.js'), 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const futureFiles = [
  'tools/character-runtime-smoke-acceptance/schema/runtime-smoke-acceptance-v1.schema.json',
  'tools/character-runtime-smoke-acceptance/manifest/tier-s-runtime-smoke-v1.json',
  'tools/character-runtime-smoke-acceptance/validate.mjs',
  'tools/character-runtime-smoke-acceptance/tests/runtime-smoke-fixtures.json',
  'tools/character-runtime-smoke-acceptance/README.md',
];
const testIds = ['CHAR-SPAWN-DESPAWN', 'CHAR-RELOAD-GENERATION', 'CHAR-SCALE-AXIS-ORIGIN', 'CHAR-SKELETON-BIND', 'CHAR-CLIPS-CORE', 'CHAR-CLIPS-INTERACTION', 'CHAR-MATERIAL-SLOTS', 'CHAR-LOD-012', 'CHAR-COLLIDER-BOUNDS', 'CHAR-SOCKETS-CULLING', 'CHAR-LIGHT-SHADOW', 'CHAR-MISSING-ASSET', 'CHAR-RETRY-DISPOSE', 'CHAR-CONTEXT-LOSS', 'SENTINEL-ATTACK-ATTACHMENT', 'SENTINEL-COLLIDER-CLEARANCE', 'SENTINEL-STRUCTURE-NO-INTERPENETRATION', 'SENTINEL-SPECIAL-SOCKETS'];
check('manifest_exact_paths', futureFiles.every((p) => plan.includes(p)), 'precise future five-file smoke manifest');
check('prerequisites', ['NB-24', 'NB-21', 'NB-26', 'NB-28', 'NB-30', 'runtimeSmokePassed=false'].every((x) => plan.includes(x)), 'all future prerequisites and current not-passed state');
check('unified_scene', ['neutral_character_room', '10m×10m', 'meter reference', 'fixed lighting/camera'].every((x) => plan.includes(x)), 'minimal neutral smoke scene');
check('test_manifest', testIds.every((x) => plan.includes(x)) && ['Cen Yao', 'Bai Zhi', 'Sentinel'].every((x) => plan.includes(x)), 'three role test IDs');
check('evidence_schema', ['RuntimeSmokeEvidence', 'testId', 'assetVersion', 'contentHash', 'automatedEvidence[]', 'humanEvidence[]', 'metrics', 'reviewer', 'severity', 'rollback'].every((x) => plan.includes(x)), 'evidence bundle schema');
check('auto_human_boundary', ['自动证据', '人工证据', '视觉身份', '不得推断 pass'].every((x) => plan.includes(x)), 'automatic vs human evidence boundary');
check('performance_method', ['frame-time P50/P95', 'draw calls', 'triangle count', 'texture memory', 'hardware profile', 'does not claim current FPS'].every((x) => plan.includes(x)), 'performance measurement without fabricated FPS');
check('severity', ['P0:', 'P1:', 'P2:', 'P3:'].every((x) => plan.includes(x)), 'P0-P3 failure severity');
check('lifecycle', ['initial load', 'world reload with new generation', 'renderer retry', 'context loss', 'dispose', 'never replay action'].every((x) => plan.includes(x)), 'lifecycle and no action replay');
check('no_fallback', ['Three', 'Side2D', 'HTTP', 'legacy asset path fallback', 'must appear visible stable error'].every((x) => plan.includes(x)), 'no fallback contract');
check('sentinel_extra', ['SENTINEL-ATTACK-ATTACHMENT', 'SENTINEL-COLLIDER-CLEARANCE', 'SENTINEL-STRUCTURE-NO-INTERPENETRATION', 'SENTINEL-SPECIAL-SOCKETS'].every((x) => plan.includes(x)), 'Sentinel-specific coverage');
check('renderer_inventory', ['Babylon-only chain', 'CampaignRenderer', 'TpsRenderer', 'init', 'setData', 'setPlayerState', 'dispose'].every((x) => plan.includes(x)) && index.includes('vendor/babylon.js') && boot.includes('CampaignRenderer') && renderer.includes('TpsRenderer'), 'current Babylon API inventory');
check('media_flags', ['不读取 GLB/图片/纹理/动画/音频', '不启动 runtime', '不生成媒体'].every((x) => plan.includes(x)), 'media/runtime prohibition');
check('not_executed', plan.includes('ready_not_executed') && plan.includes('does not claim smoke passed'), 'not executed/not passed');
check('future_files_absent', futureFiles.every((p) => !fs.existsSync(path.join(root, p))), 'future acceptance files not created');
check('forbidden_tp_reference', !plan.includes(['TP-', '18D2'].join('')), 'no forbidden reference');

const result = {
  schema: 'new_baseline_first_formal_character_runtime_smoke_acceptance_nb34_v1',
  status: 'character_runtime_smoke_acceptance_ready_not_executed',
  conclusion: 'character_runtime_smoke_acceptance_ready_not_executed',
  implemented: false,
  runtimeSmokePassed: false,
  performanceMeasured: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaImported: false,
  modelCalled: false,
  network: false,
  roles: ['cen_yao', 'bai_zhi', 'sentinel'],
  testIds,
  severity: ['P0', 'P1', 'P2', 'P3'],
  manifest: futureFiles,
  renderer: { formalChain: 'Babylon-only', api: ['init', 'setData', 'setPlayerState', 'dispose', 'debug'], noFallback: ['Three', 'Side2D', 'HTTP', 'legacy'] },
  checks,
};
if (checks.some((x) => !x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
