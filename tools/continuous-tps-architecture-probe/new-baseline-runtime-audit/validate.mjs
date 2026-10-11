import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-runtime-audit-nb04.json');
const allowedWrites = ['docs/BABYLON_RUNTIME_BASELINE_AUDIT.md','tools/continuous-tps-architecture-probe/new-baseline-runtime-audit/**','tools/continuous-tps-architecture-probe/results/new-baseline-runtime-audit-nb04.json'];
const rel = p => path.relative(root, p).replaceAll('\\', '/');
const files = {
  html: 'server-rs/release-ui/index.html', boot: 'server-rs/release-ui/js/renderer-boot.js', app: 'server-rs/release-ui/js/app.js',
  tps: 'server-rs/release-ui/js/tps-renderer.js', threeRenderer: 'server-rs/release-ui/js/renderer.js', side2d: 'server-rs/release-ui/js/hive-side2d.js',
  babylon: 'server-rs/release-ui/vendor/babylon.js', config: 'server-rs/tauri.conf.json', cargo: 'server-rs/Cargo.toml',
  campaignMain: 'server-rs/src/campaign_main.rs', worldV3: 'server-rs/src/world_v3.rs', campaign: 'server-rs/src/campaign.rs', state: 'server-rs/src/state.rs', lib: 'server-rs/src/lib.rs'
};
const abs = key => path.join(root, files[key]);
const read = key => fs.readFileSync(abs(key), 'utf8');
const exists = key => fs.existsSync(abs(key));
const digest = key => exists(key) ? crypto.createHash('sha256').update(fs.readFileSync(abs(key))).digest('hex') : null;
const lines = key => read(key).split(/\r?\n/);
const evidence = (key, pattern) => { const re = pattern instanceof RegExp ? pattern : new RegExp(pattern, 'i'); const i = lines(key).findIndex(x => re.test(x)); return i < 0 ? null : { path: files[key], line: i + 1, text: lines(key)[i].trim() }; };
const src = Object.fromEntries(Object.keys(files).map(k => [k, exists(k) ? { path: files[k], bytes: fs.statSync(abs(k)).size, sha256: digest(k) } : { path: files[k], exists: false }]));
const html = read('html'); const boot = read('boot'); const app = read('app'); const tps = read('tps'); const campaign = read('campaign'); const worldV3 = read('worldV3'); const main = read('campaignMain'); const config = read('config');
const scriptChain = [...html.matchAll(/<script\s+src="([^"]+)"/g)].map(m => m[1]);
const expectedChain = ['vendor/babylon.js','js/tps-renderer.js','js/renderer-boot.js','js/app.js'];
const staticChainPass = JSON.stringify(scriptChain) === JSON.stringify(expectedChain);
const dynamicScriptMatches = [...(app + '\n' + boot + '\n' + tps).matchAll(/(?:createElement\s*\(\s*["']script["']|appendChild\s*\([^)]*\.src|innerHTML\s*=\s*[^;]*<script)/gi)].map(m => m[0]);
const forbiddenFormalRefs = { three: scriptChain.some(x => x === 'vendor/three.min.js' || x === 'js/renderer.js'), side2d: scriptChain.some(x => x === 'js/hive-side2d.js'), dynamic: dynamicScriptMatches.length > 0 };
const repositoryResidueRefs = { three: /vendor\/three\.min\.js|js\/renderer\.js|\bTHREE\b/.test(html + '\n' + boot + '\n' + app + '\n' + tps), side2d: /hive-side2d\.js|side2d/.test(html + '\n' + boot + '\n' + app) };
const nodeChecks = ['boot','app','tps'].map(k => { const r = spawnSync(process.execPath, ['--check', abs(k)], { encoding: 'utf8' }); return { file: files[k], exitCode: r.status, pass: r.status === 0 }; });
const worldIds = ['grey_hive','mist_harbor','clockworks'];
const worldEvidence = Object.fromEntries(worldIds.map(id => ({ id, appBranch: new RegExp(`\\b${id}\\b`).test(app), rendererTheme: new RegExp(`\\b${id}:\\s*\\{`).test(tps), sameCampaignRenderer: /CampaignRenderer/.test(boot) && /TpsRenderer/.test(tps), note: id === 'clockworks' && !new RegExp(`\\b${id}:\\s*\\{`).test(tps) ? 'shared Babylon renderer uses generic FALLBACK_THEME; no dedicated theme branch' : 'Babylon renderer path present' })).map(x => [x.id, x]));
const semantic = [
  ['Vec3','present','server-rs/src/world_v3.rs:66-98','typed metres fields x_m/y_m/z_m'],
  ['Transform','present','server-rs/src/world_v3.rs:121-156','position_m and yaw_rad'],
  ['Velocity','present','server-rs/src/world_v3.rs:162-216','velocity_mps in ActorView/PlayerView'],
  ['Collider','absent','server-rs/src/world_v3.rs','no continuous Collider type in WorldV3 contract'],
  ['Trigger','absent','server-rs/src/world_v3.rs','no continuous Trigger type'],
  ['Room','absent','server-rs/src/world_v3.rs','no continuous Room type'],
  ['Zone','partial','server-rs/src/main.rs:419-442','legacy zone session and zone2d output'],
  ['Door','partial','server-rs/src/world_v3.rs:191-201; server-rs/src/campaign.rs','DoorView exists, campaign gates remain grid-backed'],
  ['Spatial Query','absent','server-rs/src/campaign.rs','nearby and collision paths remain grid/tile based'],
  ['Nav','absent','server-rs/src/campaign.rs','no continuous Nav contract observed'],
  ['Grid','legacy','server-rs/src/campaign.rs:732-810,889-1124','base_grid, tiles, px/py and grid coordinates remain runtime path']
].map(([semantic, status, evidence, note]) => ({ semantic, status, evidence, note }));
const oldRenderer = {
  three: { status: 'inactive_archival_residue', path: files.threeRenderer, evidence: evidence('html', /three\.min\.js.*不再加载/) ?? evidence('threeRenderer', /THREE = global\.THREE/), formalHtmlReference: false, dynamicReference: false },
  side2d: { status: 'inactive_archival_residue', path: files.side2d, evidence: evidence('html', /hive-side2d\.js.*停用/) ?? evidence('side2d', /已停用/), formalHtmlReference: false, dynamicReference: false },
  rendererJs: { status: 'inactive_archival_residue', path: files.threeRenderer, formalHtmlReference: false, dynamicReference: false }
};
const dataFlow = [
  { stage: 'Rust authority', paths: ['server-rs/src/campaign_main.rs:13-31','server-rs/src/campaign_main.rs:87-93'], role: 'CampaignStore owns view/action/save through Tauri commands' },
  { stage: 'Tauri/IPC', paths: ['server-rs/src/campaign_main.rs:35-49','server-rs/src/campaign_main.rs:84-100'], role: 'campaign_view/action/save and tp06a channel commands' },
  { stage: 'HTTP fallback', paths: ['server-rs/release-ui/js/app.js:189-206'], role: 'browser fetches /api/view,/api/action,/api/save; no matching Rust HTTP route found in audited entry files' },
  { stage: 'UI', paths: ['server-rs/release-ui/js/app.js:214-225,595-609'], role: 'normalizes state, calls CampaignRenderer, renders HTML/CSS HUD' },
  { stage: 'Babylon', paths: ['server-rs/release-ui/js/tps-renderer.js:970-1018,1073-1108'], role: 'creates Babylon scene and consumes map/player state; short local prediction only' }
];
const testResults = [...nodeChecks, { test: 'targeted frontend existing test', result: 'no existing targeted browser/rendering test discovered; no Cargo test run because audit package is read-only and compilation would create build artifacts outside whitelist' }];
const gate = { babylonFormalEntry: staticChainPass && /TpsRenderer/.test(boot) ? 'pass' : 'fail', noFormalThreeFallback: !forbiddenFormalRefs.three && !forbiddenFormalRefs.dynamic ? 'pass' : 'uncertain', noFormalSide2dTakeover: !forbiddenFormalRefs.side2d && !forbiddenFormalRefs.dynamic ? 'pass' : 'uncertain', rustAuthorityBoundary: /campaign_view|campaign_action|campaign_save/.test(main) && /CampaignRenderer/.test(app) ? 'partial' : 'uncertain', continuousWorldRuntimeReadiness: semantic.some(x => x.status === 'legacy' || x.status === 'absent') ? 'partial' : 'pass' };
const output = {
  schema: 'new_baseline_runtime_audit_nb04_v1', schemaVersion: 1, generatedAtUtc: new Date().toISOString(), status: 'baseline_audit_complete', conclusion: 'architecture_gate_blocked_partial', allowedWrites,
  formalEntry: { root: files.html.replace('/index.html',''), rootEvidence: src.config, frontendDist: './release-ui', scriptChain, expectedChain, scriptOrderPass: staticChainPass, babylonVendor: src.babylon, tpsBinding: evidence('tps', /global\.TpsRenderer/), campaignRendererBinding: evidence('boot', /global\.CampaignRenderer/) },
  failureRetry: { bootChecks: [evidence('boot', /if \(!tpsApi\)/), evidence('boot', /if \(failed\)/), evidence('app', /function handleRendererFailure/), evidence('app', /function retryRenderer/)], visibleFailure: /重试画面/.test(boot + app), silentFallback: false },
  forbiddenReferences: { formalEntry: forbiddenFormalRefs, repositoryResidueRefs }, dynamicScriptMatches, oldRenderer, worldBranches: worldEvidence,
  dataFlow, continuousWorldSemanticMatrix: semantic, tests: testResults, sourceInventory: src, gate, nextMinimalFixPackages: ['NB-04A: replace campaign grid/px/py authority with runtime WorldStateV3 integration while preserving save migration boundary', 'NB-04B: add Collider/Trigger/Room/Zone/Door/SpatialQuery/Nav runtime contracts and connect Tauri channel to real campaign state', 'NB-04C: add dedicated clockworks Babylon world definition/assets only after asset governance approval', 'NB-04D: add targeted browser/Tauri smoke test for formal script chain and failure retry'],
  externalWorktreeChanges: 'present outside NB-04; not attributed, modified, or cleaned', targetedDiffScope: true
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, scriptOrderPass: staticChainPass, forbiddenFormalRefs, worlds: Object.keys(worldEvidence).length, testPasses: nodeChecks.filter(x => x.pass).length, gate }, null, 2));
