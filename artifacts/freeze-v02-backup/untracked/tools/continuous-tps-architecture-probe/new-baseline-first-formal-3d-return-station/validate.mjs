import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const planPath = path.join(root, 'docs', 'FIRST_FORMAL_3D_RETURN_STATION_SLICE_PLAN.md');
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const manifest = [
  'server-rs/src/return_station.rs', 'server-rs/src/lib.rs', 'server-rs/src/campaign_main.rs',
  'server-rs/tests/return_station_test.rs', 'server-rs/tests/return_station_integration_test.rs',
  'server-rs/release-ui/js/return-station-view.js', 'server-rs/release-ui/js/app.js',
  'server-rs/release-ui/index.html', 'server-rs/release-ui/css/style.css',
  'server-rs/release-ui/js/renderer-boot.js', 'server-rs/release-ui/js/tps-renderer.js',
  'server-rs/tests/return_station_packaged_smoke_test.rs', 'tools/asset-registry/return-station.manifest.json',
];
const spaceIds = ['return_station.room.arrival', 'return_station.pad.return', 'return_station.terminal.tasks', 'return_station.terminal.save', 'return_station.terminal.exchange', 'return_station.npc.curator', 'return_station.room.world_select', 'return_station.terminal.world_select', 'return_station.door.world_gate', 'return_station.trigger.exit'];
const dependencies = ['F1', 'F2', 'F3', 'F4', 'F5', 'Save v3', 'transport', 'asset provenance', 'Architecture Gate'];

check('dependency_blocked', plan.includes('return_station_3d_implementation_ready_dependency_blocked') && plan.includes('implementation is blocked'), 'current conclusion is dependency blocked');
check('formal_boundaries', ['Rust authority', 'Babylon', 'Tauri', 'HTML/CSS/JS UI', 'does not own combat'].every((x) => plan.includes(x)), 'station responsibility and non-responsibility defined');
check('future_manifest', manifest.every((x) => plan.includes(x)) && plan.includes('No file in this manifest is changed'), 'future patch manifest is complete and non-executing');
check('metric_space', ['metres', 'Y` up', 'radians', '24m × 18m × 6m'].every((x) => plan.includes(x)) && spaceIds.every((x) => plan.includes(x)), 'space IDs and metre contract present');
check('station_points', ['entrance/return pad', 'task terminal', 'save/checkpoint terminal', 'reward exchange', 'character/NPC', 'world-selection terminal', 'return pad'].every((x) => plan.includes(x)), 'required station points present');
check('authority_projection', ['Rust / Tauri authority', 'Babylon / UI presentation', 'never decides passability', 'no file writes'].every((x) => plan.includes(x)), 'state and presentation boundary present');
check('flows', ['Return from Grey Hive', 'Save, close, and restore', 'World selection', 'Failure, disconnect, and renderer retry'].every((x) => plan.includes(x)), 'required flows present');
check('minimal_primitives', ['Interaction:', 'Camera:', 'Collision:', 'Navigation:', 'Door:', 'Triggers:'].every((x) => plan.includes(x)), 'minimal interaction/camera/collision/nav/door/trigger bar present');
check('hub_strategy', ['Phase 1 keeps the current Web Hub', 'Phase 2 adds an explicit station mode', 'Phase 3 retires the legacy Hub', 'never silently'].every((x) => plan.includes(x)), 'Hub coexistence and retirement are explicit');
check('placeholder_registry_license', ['text-only IDs', 'Asset ID', 'SHA-256', 'license', 'redistributable flag', 'No image/audio/video bytes'].every((x) => plan.includes(x)), 'placeholder and registry/license gate present');
check('dependency_unlock', dependencies.every((x) => plan.includes(x)) && plan.includes('No station implementation starts'), 'all dependency unlocks are blocking');
check('packaged_acceptance', ['packaged build', 'epoch barrier', 'close/reload/restore', 'three world selections', 'renderer retry', 'no Grid', 'rollback'].every((x) => plan.includes(x)), 'packaged smoke/reload/save/retry/no-Grid/rollback acceptance present');
check('no_fallback', ['fail closed', 'no silent HTTP', 'no legacy renderer fallback', 'no duplicate intent replay'].every((x) => plan.includes(x)), 'no silent fallback or duplicate authority');
const forbiddenTaskTokens = ['TP-' + '15C3', 'TP-' + '18D1', 'TP-' + '18D2'];
check('no_media_and_no_unapproved_tasks', forbiddenTaskTokens.every((token) => !plan.includes(token)) && plan.includes('No image/audio/video bytes'), 'no forbidden task references or media operations');
check('not_formal_implementation', plan.includes('not an implementation') && plan.includes('Architecture Gate remains not passed'), 'plan cannot be mistaken for implementation');

const result = {
  schema: 'new_baseline_first_formal_3d_return_station_nb37_v1',
  status: checks.every((x) => x.ok) ? 'return_station_3d_implementation_ready_dependency_blocked' : 'return_station_3d_plan_invalid',
  conclusion: checks.every((x) => x.ok) ? 'return_station_3d_implementation_ready_dependency_blocked' : 'return_station_3d_plan_invalid',
  implemented: false,
  defaultOffUntilDependencies: true,
  gatePass: false,
  manifestCount: manifest.length,
  spaceIdCount: spaceIds.length,
  dependencies,
  checks,
  currentUiInventory: ['campaign_view', 'campaign_action', 'campaign_save', '/api/view', '/api/action', '/api/save', 'CampaignRenderer', 'TpsRenderer'],
  scope: { formalCodeChanged: false, runtimeChanged: false, assetsChanged: false, mediaTouched: false, unverifiedPackagesConsumed: false },
};
const outArg = process.argv.indexOf('--out');
if (outArg >= 0 && process.argv[outArg + 1]) {
  fs.mkdirSync(path.dirname(path.resolve(process.argv[outArg + 1])), { recursive: true });
  fs.writeFileSync(path.resolve(process.argv[outArg + 1]), `${JSON.stringify(result, null, 2)}\n`);
}
if (!checks.every((x) => x.ok)) {
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
console.log(JSON.stringify(result, null, 2));
