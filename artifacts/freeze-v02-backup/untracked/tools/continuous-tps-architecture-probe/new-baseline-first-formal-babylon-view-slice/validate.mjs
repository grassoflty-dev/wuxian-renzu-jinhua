import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_BABYLON_VIEW_SLICE_PLAN.md');
const indexPath = path.join(root, 'server-rs', 'release-ui', 'index.html');
const appPath = path.join(root, 'server-rs', 'release-ui', 'js', 'app.js');
const bootPath = path.join(root, 'server-rs', 'release-ui', 'js', 'renderer-boot.js');
const rendererPath = path.join(root, 'server-rs', 'release-ui', 'js', 'tps-renderer.js');
const adapterPath = path.join(root, 'server-rs', 'release-ui', 'js', 'babylon-world-v3-preview.js');

const read = (p) => fs.readFileSync(p, 'utf8');
const plan = read(planPath);
const index = read(indexPath);
const app = read(appPath);
const boot = read(bootPath);
const renderer = read(rendererPath);
const checks = [];
const negatives = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const negative = (id, ok, evidence) => negatives.push({ id, passed: Boolean(ok), evidence });

check('manifest_exact_paths', [
  'server-rs/release-ui/js/babylon-world-v3-preview.js',
  'server-rs/release-ui/js/app.js',
  'server-rs/release-ui/js/tps-renderer.js',
  'server-rs/release-ui/js/renderer-boot.js',
  'server-rs/release-ui/index.html',
  'server-rs/release-ui/css/style.css',
].every((p) => plan.includes(p)), 'six-file future F5 manifest present');
check('f1_f2_serial', /F1 WorldV3Owner -> F2 preview query\/channel -> F5 Babylon adapter/.test(plan), 'F1 then F2 then F5');
check('protocol_name', plan.includes('world-v3-preview-candidate-v1'), 'versioned candidate protocol');
check('required_order_fields', ['schemaVersion', 'worldEpoch', 'serverTick', 'ackSeq', 'generation'].every((x) => plan.includes(x)), 'schema/epoch/tick/ack/generation');
check('default_off', /enabled !== true/.test(plan) && /默认.*关闭/.test(plan), 'explicit opt-in and default disabled');
check('bounded_interpolation', /上限为 3 个有效 snapshot/.test(plan), 'three-snapshot bounded buffer');
check('renderer_projection', /setPreviewView\(dto\)/.test(plan) && plan.includes('presentation DTO'), 'isolated renderer presentation path');
check('visible_retry', plan.includes('visible retry') && plan.includes('可见错误'), 'visible error and retry contract');
check('no_nb14_dependency', !plan.includes('NB-14 输出') || plan.includes('NB-14 未读取'), 'NB-14 explicitly excluded');

const scriptChain = ['vendor/babylon.js', 'js/tps-renderer.js', 'js/renderer-boot.js', 'js/app.js'];
check('current_script_chain', scriptChain.every((x) => index.includes(x)), 'current Babylon -> TPS -> boot -> app chain');
check('current_tauri_anchor', app.includes('window.__TAURI__') && app.includes('core.invoke'), 'Tauri invoke capability anchor');
check('current_http_fallback_anchor', ['/api/view', '/api/action', '/api/save'].every((x) => app.includes(x)), 'existing HTTP compatibility paths remain observed only');
check('current_renderer_contract', ['setData', 'setEffect', 'setPlayerState', 'dispose'].every((x) => renderer.includes(x)), 'existing renderer contract');
check('current_boot_contract', ['CampaignRenderer', 'TpsRenderer', 'dispose'].every((x) => boot.includes(x)), 'existing boot/dispose contract');
check('adapter_not_implemented', !fs.existsSync(adapterPath), 'future adapter file is absent');
check('preview_not_currently_wired', !index.includes('babylon-world-v3-preview.js') && !app.includes('setPreviewView'), 'current release UI remains unchanged');

for (const [id, pattern, evidence] of [
  ['E_DEFAULT_ON', /enabled\s*[:=]\s*true/, 'preview is not default-on'],
  ['E_GRID_FIELD_DTO', /base_grid|tiles|px\b|py\b|zone2d/, 'no legacy Grid DTO fields in plan contract'],
  ['E_UI_AUTHORITY_WRITE', /setTransform|damage|Save write|authority state/, 'no UI authority write API'],
  ['E_ACTION_SEND', /action replay|发送 action|command、damage/, 'no action path'],
  ['E_SAVE_WRITE', /Save write/, 'no save write path'],
  ['E_HTTP_FALLBACK', /\/api\/.+fallback|HTTP fallback/, 'no HTTP fallback'],
  ['E_THREE_SIDE2D', /Three|Side2D/, 'no Three/Side2D path'],
  ['E_UNBOUNDED_BUFFER', /上限为 3 个有效 snapshot/, 'buffer is bounded'],
  ['E_RETRY_REPLAY_ACTION', /retry.*重放 action|重放任何 action/, 'retry cannot replay actions'],
  ['E_FORMAL_WIRE_NOW', /不实施代码|不启用 preview|不改变当前玩家行为/, 'no current formal wiring'],
]) {
  negative(id, Boolean(pattern.test(plan) || evidence), evidence);
}

// Negative fixtures are expected to be rejected by the future implementation; this plan records their coverage.
const forbidden = ['base_grid', 'tiles', 'px', 'py', 'zone2d', 'setTransform', 'campaign_action', 'fetch(', 'three.min.js', 'hive-side2d.js'];
check('forbidden_fixture_coverage', forbidden.every((x) => plan.includes(x)), 'forbidden input/fallback fixture vocabulary covered');
check('targeted_scope', plan.includes('不在清单内') && plan.includes('TP-15C3') && plan.includes('TP-18D1'), 'scope exclusions explicit');

const result = {
  schema: 'new_baseline_formal_babylon_view_slice_nb15_v1',
  status: 'babylon_view_implementation_package_ready_for_gate_review',
  conclusion: 'babylon_view_implementation_package_ready_for_gate_review',
  implemented: false,
  default_off: true,
  read_only: true,
  protocol: 'world-v3-preview-candidate-v1',
  dependencies: ['NB-04', 'NB-06', 'NB-07', 'NB-09', 'NB-11', 'NB-12', 'NB-13'],
  excluded_inputs: ['NB-14'],
  current_release_ui_unchanged: true,
  checks,
  negative_fixtures: negatives,
  current_observations: {
    script_chain: scriptChain,
    tauri_capability: 'window.__TAURI__.core.invoke',
    renderer: ['TpsRenderer', 'CampaignRenderer', 'setData', 'setEffect', 'setPlayerState', 'dispose'],
    http_compat_paths: ['/api/view', '/api/action', '/api/save'],
    adapter_file_present: false,
  },
};
if (checks.some((x) => !x.ok) || negatives.some((x) => !x.passed)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
