import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { evaluateGate, runNegativeFixtures, EXACT_FORMAL_WRITES } from './gate.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const base = 'tools/continuous-tps-architecture-probe/';
const own = `${base}formal-capability-hud-readiness/`;
const resultPath = `${base}results/formal-capability-hud-readiness-tp01r7.json`;
const reportPath = 'docs/CONTINUOUS_FORMAL_CAPABILITY_HUD_READINESS_REVIEW.md';
const allowedWrites = [`${own}**`, resultPath, reportPath];
const activeWrites = [
  `${base}physics-elevators-optional/**`, `${base}results/physics-elevators-optional-ag38.json`, 'docs/CONTINUOUS_OPTIONAL_ELEVATORS_PHYSICS_REVIEW.md',
  `${base}dynamic-door-owner/**`, `${base}results/dynamic-door-owner-tp04c1.json`, 'docs/CONTINUOUS_DYNAMIC_DOOR_OWNER_REVIEW.md',
  `${base}formal-asset-release-readiness/**`, `${base}results/formal-asset-release-readiness-tp01r6.json`, 'docs/CONTINUOUS_FORMAL_ASSET_RELEASE_READINESS_REVIEW.md',
];
const abs = (p) => path.resolve(root, p);
const rel = (p) => path.relative(root, p).replaceAll('\\', '/');
const matches = (p, pattern) => pattern.endsWith('/**') ? p.startsWith(pattern.slice(0, -2)) : p === pattern;
const hash = (value) => crypto.createHash('sha256').update(value).digest('hex');
const snapshots = new Map();
function canonicalInput(p) {
  if (typeof p !== 'string' || !p || p !== p.trim() || /[:\x00-\x1f]/.test(p)) throw new Error('E_INPUT_PATH_INVALID');
  const normalized = p.replaceAll('\\', '/');
  if (normalized.startsWith('/') || normalized.split('/').some((part) => !part || part === '.' || part === '..' || /[. ]$/.test(part))) throw new Error(`E_INPUT_PATH_INVALID ${p}`);
  const resolved = rel(abs(normalized));
  if (resolved.startsWith('../') || resolved === '..' || path.isAbsolute(resolved)) throw new Error(`E_INPUT_PATH_ESCAPE ${p}`);
  return normalized;
}
function assertReadable(p) {
  const lower = p.toLowerCase();
  if (activeWrites.some((pattern) => matches(lower, pattern.toLowerCase())) || /(^|\/)(assets|data|media|public)\//.test(lower) || /\.(png|jpe?g|gif|webp|ico|mp[34]|wav|glb|gltf|blend)$/.test(lower)) throw new Error(`E_FORBIDDEN_READ ${p}`);
}
function read(p) {
  p = canonicalInput(p);
  assertReadable(p);
  const exists = fs.existsSync(abs(p));
  if (exists) assertReadable(canonicalInput(rel(fs.realpathSync(abs(p)))));
  const bytes = exists ? fs.readFileSync(abs(p)) : null;
  const stat = exists ? fs.statSync(abs(p)) : null;
  if (!snapshots.has(p)) snapshots.set(p, { path: p, exists, sha256: bytes ? hash(bytes) : null, bytes: stat?.size ?? null, mtimeMs: stat?.mtimeMs ?? null });
  return bytes?.toString('utf8') ?? null;
}
const json = (p) => JSON.parse(read(p));
function write(p, content) {
  if (!allowedWrites.some((pattern) => matches(p, pattern)) || rel(abs(p)) !== p) throw new Error(`E_WRITE_SCOPE ${p}`);
  fs.mkdirSync(path.dirname(abs(p)), { recursive: true });
  fs.writeFileSync(abs(p), content);
}
const result = (name) => `${base}results/${name}.json`;
const evidenceNames = {
  TP18B1: 'vertical-slice-readiness-delta-tp18b1', AG12: 'capability-ag12',
  TP13A1: 'capability-rear-view-tp13a1', TP13B1: 'tauri-rear-view-tp13b1',
  TP15A1: 'hud-map-view-model-tp15a1', TP15B1: 'tauri-hud-map-tp15b1',
  TP17A1: 'runtime-integration-soak-tp17a1', TP17B1: 'runtime-owner-observability-tp17b1', TP17C1: 'tauri-runtime-observability-tp17c1',
  TP14A1: 'campaign-progression-authority-tp14a1', TP14B1: 'world-runtime-routing-tp14b1', TP14C1: 'tauri-world-routing-tp14c1',
  TP16A1: 'tauri-offline-asset-gate-tp16a1',
};
const evidence = Object.fromEntries(Object.entries(evidenceNames).map(([id, name]) => [id, json(result(name))]));
const pkg = evidence.TP18B1.packages.find((p) => p.id === 'PKG-FORMAL-CAPABILITY-HUD');
if (!pkg) throw new Error('E_PACKAGE_MISSING');
if (!Array.isArray(pkg.writes) || pkg.writes.length !== EXACT_FORMAL_WRITES.length
    || pkg.writes.some((p) => !EXACT_FORMAL_WRITES.includes(p))
    || new Set(pkg.writes).size !== EXACT_FORMAL_WRITES.length) throw new Error('E_PLANNING_WRITE_SET_DRIFT');
const files = [
  ...pkg.writes, 'server-rs/release-ui/js/app.js', 'server-rs/release-ui/js/tps-renderer.js',
  'server-rs/src/campaign.rs', 'server-rs/src/tp06a_heartbeat.rs', 'server-rs/src/continuous_input.rs',
  'server-rs/src/lib.rs', 'server-rs/Cargo.toml', 'server-rs/tauri.conf.json', 'server-rs/build.rs',
  'server-rs/tests/world_v3_contract_test.rs', 'server-rs/tests/tp06_a_heartbeat_test.rs', 'server-rs/tests/continuous_input_test.rs',
];
const sources = Object.fromEntries([...new Set(files)].map((p) => [p, read(p)]));
for (const p of ['CONTINUOUS_CAPABILITY_CONTRACT_REVIEW', 'CONTINUOUS_REAR_VIEW_RENDER_TARGET_REVIEW', 'CONTINUOUS_REAR_VIEW_WEBVIEW_REVIEW', 'CONTINUOUS_HUD_MAP_VIEW_MODEL_REVIEW', 'CONTINUOUS_HUD_MAP_WEBVIEW_REVIEW', 'CONTINUOUS_RUNTIME_INTEGRATION_SOAK_REVIEW', 'CONTINUOUS_RUNTIME_OWNER_OBSERVABILITY_REVIEW', 'CONTINUOUS_TAURI_RUNTIME_OBSERVABILITY_REVIEW', 'CONTINUOUS_WORLD_ROUTING_WEBVIEW_REVIEW']) read(`docs/${p}.md`);

const commands = [];
function run(id, command, args, level, extra = {}) {
  const proc = spawnSync(command, args, { cwd: abs(own), encoding: 'utf8', windowsHide: true, timeout: 60000, maxBuffer: 4 * 1024 * 1024, env: { ...process.env, CARGO_NET_OFFLINE: 'true' } });
  const log = `${own}logs/${id}.txt`;
  write(log, `${proc.stdout ?? ''}${proc.stderr ?? ''}${proc.error ? String(proc.error) : ''}`);
  const item = { id, command: [command, ...args].map((s) => s.includes(' ') ? JSON.stringify(s) : s).join(' '), exitCode: proc.status, signal: proc.signal, pass: proc.status === 0 && !proc.error, evidenceLevel: level, log, ...extra };
  commands.push(item);
  return item;
}

// Original validators write their result unconditionally. Exact-byte source copies
// in an isolated tree preserve their relative paths; Node permissions confine I/O.
const mirror = `${own}replay/`;
const replaySpecs = [
  { id: 'AG12', entry: 'capability/validate.mjs', output: 'capability-ag12', copies: ['capability/validate.mjs'] },
  { id: 'TP13A1', entry: 'capability-rear-view/test.mjs', output: 'capability-rear-view-tp13a1', copies: ['capability-rear-view/test.mjs', 'capability-rear-view/adapter.js'], vendor: true },
  { id: 'TP15A1', entry: 'hud-map-view-model/test.mjs', output: 'hud-map-view-model-tp15a1', copies: ['hud-map-view-model/test.mjs', 'hud-map-view-model/view-model.mjs'] },
];
const replayProvenance = [];
for (const spec of replaySpecs) {
  const copies = spec.copies.map((p) => `${base}${p}`);
  if (spec.vendor) copies.push('server-rs/release-ui/vendor/babylon.js');
  for (const p of copies) {
    read(p);
    const bytes = fs.readFileSync(abs(p));
    write(`${mirror}${p}`, bytes);
    replayProvenance.push({ original: p, copy: `${mirror}${p}`, sha256: hash(bytes), identical: hash(fs.readFileSync(abs(`${mirror}${p}`))) === hash(bytes), contentType: 'JavaScript source, no asset/media bytes' });
  }
  fs.mkdirSync(abs(`${mirror}${base}results`), { recursive: true });
  const cmd = run(`replay-${spec.id}`, process.execPath, ['--permission', `--allow-fs-read=${abs(mirror)}`, `--allow-fs-write=${abs(`${mirror}${base}results`)}`, abs(`${mirror}${base}${spec.entry}`)], 'fresh_isolated_candidate_test_replay', { originalEntry: `${base}${spec.entry}`, originalUnmodified: true });
  const p = `${mirror}${result(spec.output)}`;
  const replay = fs.existsSync(abs(p)) ? JSON.parse(fs.readFileSync(abs(p), 'utf8')) : null;
  cmd.resultPath = p;
  cmd.scenarioCount = replay?.scenarios?.length ?? replay?.assertionCount ?? 0;
  cmd.assertionsPass = replay?.pass ?? replay?.machinePass ?? replay?.allChecksPassed ?? false;
  cmd.pass &&= cmd.assertionsPass;
}

const binarySpecs = [
  ['TP17A1', `${base}runtime-integration-soak`, 'continuous_runtime_integration_soak_tp17a1', '21c62151a55f4c2e', 3],
  ['TP17B1', `${base}runtime-owner-observability`, 'continuous_runtime_owner_observability_tp17b1', 'be5ea5d0c6647390', 3],
  ['TP14A1', `${base}campaign-progression-authority`, 'continuous_campaign_progression_authority_tp14a1', '7548ac2130ed7636', 4],
  ['TP14B1', `${base}world-runtime-routing`, 'continuous_world_runtime_routing_tp14b1', 'befe364ce1c13a52', 4],
  ['formal-world-view', 'server-rs', 'world_v3_contract_test', '1638e47717c39a35', 5, 'world_v3.rs'],
  ['formal-heartbeat', 'server-rs', 'tp06_a_heartbeat_test', 'fa28b9e694a03dec', 6, 'tp06a_heartbeat.rs'],
  ['formal-input', 'server-rs', 'continuous_input_test', '28339edded343604', 8, 'continuous_input.rs'],
];
for (const [id, dir, target, suffix, expectedTests, implementation] of binarySpecs) {
  const formal = dir === 'server-rs';
  const src = formal ? `${dir}/tests/${target}.rs` : `${dir}/src/lib.rs`;
  read(src);
  if (implementation) read(`${dir}/src/${implementation}`);
  const binary = `${dir}/target/debug/deps/${target}-${suffix}.exe`;
  const crate = formal ? 'wuxian-horror-ch1' : target.replaceAll('_', '-');
  const fingerprint = `${dir}/target/debug/.fingerprint/${crate}-${suffix}/test-${formal ? 'integration-test' : 'lib'}-${target}.json`;
  read(fingerprint);
  read(binary);
  const stamp = snapshots.get(binary);
  if (!stamp.exists) { commands.push({ id: `prebuilt-${id}`, pass: false, exitCode: null, errorCode: 'E_PREBUILT_TEST_MISSING', binary }); continue; }
  const cmd = run(`prebuilt-${id}`, abs(binary), ['--test-threads=1'], 'prebuilt_test_binary_reexecution_not_source_recompiled', { binary, binarySha256: stamp.sha256, fingerprint, fingerprintSha256: snapshots.get(fingerprint).sha256, source: src, sourceSha256: snapshots.get(src).sha256, sourceNewerThanBinary: snapshots.get(src).mtimeMs > stamp.mtimeMs, implementation: implementation ? `${dir}/src/${implementation}` : null, expectedTests, buildSourceBinding: 'not cryptographically attested; cannot prove current source compilation' });
  const output = fs.readFileSync(abs(cmd.log), 'utf8');
  cmd.observedTests = Number(output.match(/test result: ok\. (\d+) passed/)?.[1] ?? 0);
  cmd.pass &&= cmd.observedTests === expectedTests;
}
for (const p of ['server-rs/release-ui/js/renderer-boot.js', 'server-rs/release-ui/js/app.js', 'server-rs/release-ui/js/tps-renderer.js']) run(`syntax-${path.basename(p, '.js')}`, process.execPath, ['--check', abs(p)], 'syntax_only_not_runtime_wiring');
const missingResolver = 'server-rs/release-ui/js/capability-resolver.js';
run('syntax-capability-resolver', process.execPath, ['--check', abs(missingResolver)], 'required_package_check_missing_file', { expectedToExist: true, expectedBlocker: 'E_FORMAL_CAPABILITY_OWNER_MISSING' });

const lines = (p, text) => (sources[p] ?? '').split(/\r?\n/).flatMap((line, i) => line.includes(text) ? [i + 1] : []);
const formalGraph = {
  releaseRoot: JSON.parse(sources['server-rs/tauri.conf.json']).build.frontendDist,
  missingExpectedFiles: pkg.writes.filter((p) => sources[p] === null),
  entryScripts: (sources['server-rs/release-ui/index.html'].match(/<script[^>]+src="[^"]+"/g) ?? []),
  worldStructLines: lines('server-rs/src/world_v3.rs', 'pub struct World'),
  formalCapabilityTokenHits: Object.entries(sources).filter(([p]) => /world_v3.rs|campaign_main.rs|release-ui\/js/.test(p)).flatMap(([p, text]) => (text ?? '').split(/\r?\n/).flatMap((line, i) => /CapabilityView|CapabilityResolver|perception\.rear_view|loot_scan|lootScan/.test(line) ? [{ path: p, line: i + 1, text: line.trim() }] : [])),
  caution: 'Source observations plus missing files establish blockers; syntax, token hits and legacy facilities are never sufficient proof of a production owner.',
};
const refs = (...ids) => ids.map((id) => ({ id, path: result(evidenceNames[id]), sha256: snapshots.get(result(evidenceNames[id])).sha256, level: id === 'TP18B1' ? 'planning_only' : /B1|C1/.test(id) && ['TP13B1', 'TP15B1', 'TP17C1', 'TP14C1', 'TP16A1'].includes(id) ? 'real_webview_candidate' : 'isolated_candidate' }));
const rollback = pkg.rollback;
const row = (id, area, ids, owner, formalFiles, tests, code, finding, candidatePass = null) => ({ id, area, evidence: refs(...ids), formalOwner: owner, formalFiles: formalFiles.map((p) => ({ path: p, exists: snapshots.get(p)?.exists ?? fs.existsSync(abs(p)) })), requiredTests: tests, rollback, currentStatus: 'blocked', pass: false, candidatePass, errorCode: code, finding });
const world = 'server-rs/src/world_v3.rs', entry = 'server-rs/src/campaign_main.rs', app = 'server-rs/release-ui/js/app.js', renderer = 'server-rs/release-ui/js/tps-renderer.js';
const matrix = [
  row('ADR-H', 'single authoritative capability owner', ['AG12', 'TP18B1'], 'Rust semantic grant/resolver owner; UI consumes read-only CapabilityView', [world, entry, missingResolver], ['formal grant/revoke/scope/save tests', 'owner identity and one-owner registration'], 'E_FORMAL_CAPABILITY_OWNER_MISSING', 'No formal CapabilityId/Grant/Resolver/CapabilityView; planned release resolver absent.', evidence.AG12.pass),
  row('TP-13', 'rear_view', ['TP13A1', 'TP13B1'], 'Rust effective grant -> renderer-owned presentation', [world, entry, renderer, missingResolver], ['formal no-grant/grant/revoke/regrant', '50 grant/revoke and 30 epoch replacements with formal owner', 'offline/release camera and performance acceptance'], 'E_FORMAL_REAR_VIEW_WIRING_MISSING', 'NullEngine and real WebView lifecycle results remain candidates; production resolver/renderer handoff absent.', evidence.TP13A1.allChecksPassed && evidence.TP13B1.allChecksPassed),
  row('TP-15', 'HUD and map projection', ['TP15A1', 'TP15B1'], 'read-only production HUD adapter consuming authoritative snapshots', [world, app, 'server-rs/release-ui/index.html'], ['formal HUD field schema', 'actual release DOM/renderer connection', 'manual usability and accessibility'], 'E_FORMAL_HUD_TRANSPORT_MISSING', 'TP-15 is HUD/map, not loot_scan. Formal WorldView lacks capability/HUD projection and the release UI uses legacy campaign state.', evidence.TP15A1.machinePass && evidence.TP15B1.pass),
  row('TP-16', 'release authorization', ['TP16A1', 'TP18B1'], 'authorized public release manifest/resolver owner', ['server-rs/tauri.conf.json'], ['item license/provenance approvals', 'offline real release bundle load'], 'E_RELEASE_AUTHORIZATION_MISSING', 'TP-16A1 is metadata-only/local development; public manifest remains empty; no new release authorization.', evidence.TP16A1.pass),
  row('PKG-FORMAL-IPC-OWNER', 'upstream owner prerequisite', ['TP18B1', 'TP17C1', 'TP14C1'], 'production Campaign authority -> fixed tick -> Channel owner', [entry, world, app], ['production owner replacement/close/ack', 'real 30Hz payload load and bounded queues'], 'E_FORMAL_IPC_OWNER_PREREQUISITE', 'TP-18B1 marks prerequisite unsatisfied. TP-17C1/TP-14C1 explicitly lack production CampaignStore/owner handoff.', false),
  row('PKG-FORMAL-ASSET-RELEASE', 'upstream release prerequisite', ['TP18B1', 'TP16A1'], 'authorized release asset owner', ['server-rs/tauri.conf.json'], ['authorized release manifest and bundled offline integration'], 'E_FORMAL_ASSET_RELEASE_PREREQUISITE', 'Historical completed evidence leaves prerequisite unsatisfied; active TP-01R6 files were not read or consumed.', false),
  row('LOOT-SCAN', 'loot_scan', ['AG12', 'TP15A1', 'TP15B1'], 'future Rust loot visibility/query authority; HUD reads results', [world, entry, app], ['formal scan grant/deny/revoke', 'range/visibility/ownership/cooldown and empty-result tests'], 'E_LOOT_SCAN_CONTRACT_EVIDENCE_MISSING', 'No loot_scan implementation or matching evidence found in specified inputs. map.local/HUD/map must not be relabeled loot_scan.'),
  row('EPOCH-GENERATION', 'epoch/generation and stale callbacks', ['AG12', 'TP13A1', 'TP17B1', 'TP17C1'], 'one production owner identity bound to callback generation and world epoch', [world, entry, app], ['old epoch and generation rejected before mutation', 'async-start and action completion after close/replacement', '30 real production world replacements'], 'E_FORMAL_CALLBACK_GENERATION_UNPROVEN', 'Synthetic heartbeat diagnostics are present; release app records generation but does not establish capability callback guards.', true),
  row('GRANT-REVOKE', 'idempotent grant/revoke', ['AG12', 'TP13A1', 'TP13B1'], 'Rust semantic grant ledger with renderer generation', [world, entry, missingResolver, renderer], ['duplicate grant keeps one resource', 'duplicate revoke releases exactly once', 'expiry/death/world-exit removes relevant grants'], 'E_FORMAL_GRANT_REVOKE_MISSING', 'Existing lifecycle tests cover isolated intent/adapter; no formal grant owner is connected.', true),
  row('QUEUE', 'queue capacity and overflow', ['TP17A1', 'TP17B1', 'TP17C1'], 'bounded queue owned by production runtime', [entry, app], ['capacity 8 / payload 64 / reject_newest candidate contract explicitly adopted or revised', 'duplicate sequence/stale epoch/close/reentry', 'no lost authoritative action on overflow'], 'E_FORMAL_QUEUE_BOUNDARY_UNPROVEN', 'TP-17B1/C1 candidate capacity=8, payload=64; legacy app normalQueue limit=3 is a different queue. TP-17A1 lifecycle and queue scenarios remain failed/blocked.', true),
  row('COOLDOWN', 'authoritative cooldown', ['AG12', 'TP17A1'], 'Rust capability action authority, never UI-clock authority', ['server-rs/src/campaign.rs', world, app], ['attempt same capability before expiry', 'reconnect/revoke/regrant cannot bypass cooldown', 'epoch reset follows approved policy'], 'E_CAPABILITY_COOLDOWN_WIRING_MISSING', 'Legacy campaign.rs enforces skill ownership/cooldown/energy. AG-12 validates dash cooldown parameter range only; no capability runtime bypass test exists.'),
  row('ACCESSIBILITY', 'accessibility independent of capabilities', ['AG12', 'TP15A1', 'TP15B1'], 'release UI preferences and baseline accessibility independent of grants', [app, 'server-rs/release-ui/index.html', 'server-rs/release-ui/css/style.css'], ['subtitles/font size/contrast/keybinds without grants', 'revoke does not remove baseline access', 'manual keyboard/screen-reader and unavailable/cooldown announcements'], 'E_FORMAL_ACCESSIBILITY_PROJECTION_MISSING', 'Candidate baseline passes; legacy ARIA/reduced-motion exists. Formal capability projection and manual release acceptance absent.', true),
  row('HUD-FIELDS', 'formal serialized fields', ['AG12', 'TP15A1', 'TP15B1'], 'Rust WorldView/CapabilityView producer -> immutable UI projection', [world, entry, app], ['worldId/worldEpoch/serverTick/ackSeq and ownerGeneration binding', 'protocol/version/effective/baseline schema', 'HP/energy/objectives/checkpoint/result and semantic-only capability data'], 'E_FORMAL_CAPABILITY_FIELDS_MISSING', 'WorldView epoch/tick/ack contract exists but no formal capability view or producer for required HUD semantic projection.', true),
  row('DISPOSE-CLOSE', 'dispose/close and no stale callback', ['TP13A1', 'TP13B1', 'TP15B1', 'TP17C1'], 'presentation resource owner plus runtime/Channel close owner', [renderer, entry, app], ['last revoke returns camera/RTT counts to baseline', 'owner stop/join precedes replacement', 'zero callback/action mutation after close', 'native close/pagehide evidence or explicit unknown'], 'E_FORMAL_CAPABILITY_CLEANUP_UNPROVEN', 'General renderer dispose exists. Candidate stop-after-close passes, but TP-15B1 pagehide is unobserved and formal capability cleanup is absent.', true),
  row('RELEASE-LOAD', 'real release workload', ['TP17A1', 'TP17B1', 'TP17C1', 'TP16A1'], 'production runtime/HUD on authorized published hardware and assets', [entry, renderer, app], ['release-profile 30/60Hz with real assets', 'p95/p99/RSS/GPU and queue counters', '30 world enter/exit and full release soak'], 'E_REAL_RELEASE_LOAD_MISSING', 'Synthetic debug WebView workload is not published hardware/asset load; unobservable dependency counters remain unknown.', false),
  row('BRIDGE-PROVENANCE', 'TP-14 evidence mapping', ['TP14A1', 'TP14B1', 'TP14C1'], 'production Campaign/world router, separate from HUD transport owner', [entry, world], ['formal world-router handoff and state source', 'correctly attributed HUD transport evidence TP-15B1'], 'E_EVIDENCE_MAPPING_MISMATCH', 'TP-14 is Campaign progression/world routing. Its real WebView proof cannot be claimed as HUD transport proof.', true),
];

const requiredFields = {
  worldEnvelope: ['protocol', 'version', 'worldId', 'worldEpoch', 'serverTick', 'ackSeq', 'serverTimeMs'],
  capabilityEnvelope: ['protocol', 'version', 'worldId', 'worldEpoch', 'tick', 'effective', 'baseline'],
  ownerBinding: ['owner identity', 'generation', 'epoch (may be envelope/session binding, requires explicit approved contract)'],
  hudSections: ['connection', 'hud', 'objective', 'map', 'capabilities', 'accessibility', 'player', 'diagnostics'],
  prohibitedSerializedKinds: ['runtime/GPU handles', 'DOM objects', 'commands/authority', 'media bytes'],
};
const controls = Object.fromEntries(['singleOwner', 'epochStable', 'generationStable', 'staleCallbacksRejected', 'grantIdempotent', 'revokeIdempotent', 'queueBounded', 'queueOverflowRejected', 'cooldownAuthoritative', 'accessibilityPresent', 'accessibilityIndependent', 'hudFieldsComplete', 'disposeBaseline', 'closeNoCallbacks', 'realReleaseLoad', 'releaseAuthorized', 'productionWired'].map((key) => [key, false]));
const prereqs = pkg.prerequisites.map((id) => {
  const r = matrix.find((item) => item.id === id);
  return { id, status: 'blocked', evidenceId: r.evidence[0].id, evidencePath: r.evidence[0].path, evidenceSha256: r.evidence[0].sha256, ownerId: null, errorCode: r.errorCode, formalFiles: r.formalFiles, requiredTests: r.requiredTests, rollback: r.rollback };
});
const trustedEvidence = Object.fromEntries(Object.entries(evidenceNames).map(([id, name]) => [id, { path: result(name), sha256: snapshots.get(result(name)).sha256, level: id === 'TP18B1' ? 'planning' : 'candidate', prerequisiteIds: prereqs.filter((p) => p.evidenceId === id).map((p) => p.id), ownerId: null, tests: [] }]));
const gate = evaluateGate({ packageId: pkg.id, prerequisites: prereqs, owners: [], writes: pkg.writes, controls, rollback, tests: commands.map((c) => ({ id: c.id, pass: c.pass })), scope: 'exact_package', architecture: 'blocked/no_go', formalTakeover: false }, { exactFormalWrites: pkg.writes, activeWrites, trustedEvidence, verifiedControls: controls });
const fixtures = runNegativeFixtures();
const pathNegativeFixtures = [
  ['traversal-before-active-read', `${own}../dynamic-door-owner/src/lib.rs`, 'E_INPUT_PATH_INVALID'],
  ['backslash-active-read', 'tools\\continuous-tps-architecture-probe\\dynamic-door-owner\\src\\lib.rs', 'E_FORBIDDEN_READ'],
  ['case-active-read', 'TOOLS/CONTINUOUS-TPS-ARCHITECTURE-PROBE/FORMAL-ASSET-RELEASE-READINESS/validate.mjs', 'E_FORBIDDEN_READ'],
  ['absolute-drive-read', 'C:/outside/source.rs', 'E_INPUT_PATH_INVALID'],
  ['absolute-root-read', '/outside/source.rs', 'E_INPUT_PATH_INVALID'],
  ['parent-escape-read', '../../source.rs', 'E_INPUT_PATH_INVALID'],
  ['case-media-read', 'MEDIA/example.PNG', 'E_FORBIDDEN_READ'],
  ['nested-icon-read', 'server-rs/icons/icon.ico', 'E_FORBIDDEN_READ'],
].map(([id, input, expectedCode]) => {
  let actualCode = null;
  try { assertReadable(canonicalInput(input)); } catch (error) { actualCode = error.message.split(' ')[0]; }
  return { id, input, expectedCode, actualCode, pass: actualCode === expectedCode, filesystemAccessPerformed: false };
});
const unchangedInputs = [...snapshots.values()].map((s) => ({ ...s, unchanged: fs.existsSync(abs(s.path)) === s.exists && (!s.exists || hash(fs.readFileSync(abs(s.path))) === s.sha256) }));
const rereadPass = unchangedInputs.every((s) => s.unchanged);
const blockers = matrix.map((r) => ({ code: r.errorCode, prerequisite: r.id, detail: r.finding, evidencePaths: r.evidence.map((e) => e.path) }));
blockers.push({ code: 'E_FORMAL_TESTS_NOT_RECOMPILED', prerequisite: 'validation', detail: 'Full cargo test was not run: server-rs/build.rs invokes Tauri build and may read icon/media resources and write outside the strict scope. Prebuilt test execution is explicitly weaker than a source rebuild.', evidencePaths: ['server-rs/build.rs', 'server-rs/Cargo.toml'] });
const futureWhitelist = [world, entry, missingResolver, app, renderer, 'server-rs/release-ui/js/renderer-boot.js', 'server-rs/release-ui/index.html', 'server-rs/release-ui/css/style.css', 'server-rs/tests/world_v3_contract_test.rs', 'server-rs/tests/continuous_capability_test.rs', 'server-rs/tests/continuous_capability_hud_test.rs'];
const checks = {
  exactPrerequisitesPreserved: JSON.stringify(prereqs.map((p) => p.id)) === JSON.stringify(pkg.prerequisites),
  originalPackageUnscheduled: pkg.status === 'not_schedulable_until_prerequisites' && pkg.scheduled === false,
  allSourceCopiesIdentical: replayProvenance.every((p) => p.identical),
  rerunsPass: commands.filter((c) => !c.expectedBlocker).every((c) => c.pass),
  missingResolverRecorded: commands.find((c) => c.id === 'syntax-capability-resolver').exitCode !== 0 && sources[missingResolver] === null,
  readOnlyInputsUnchanged: rereadPass,
  noActivePathConsumed: [...snapshots.keys()].every((p) => !activeWrites.some((pattern) => matches(p, pattern))),
  currentWriteSetConflictFree: allowedWrites.every((a) => activeWrites.every((b) => {
    const aa = a.replace(/\*\*$/, '');
    const bb = b.replace(/\*\*$/, '');
    return aa !== bb && !aa.startsWith(bb) && !bb.startsWith(aa);
  })),
  remainsBlocked: gate.decision === 'blocked',
  negativeFixturesPass: fixtures.pass,
  negativeReadPathFixturesPass: pathNegativeFixtures.every((fixture) => fixture.pass),
};
const output = {
  schema: 'formal_capability_hud_readiness_tp01r7_v1', packageId: pkg.id,
  status: 'blocked', decision: 'blocked', auditPass: Object.values(checks).every(Boolean), generatedAtUtc: new Date().toISOString(),
  gateConclusion: { exactPackage: 'blocked', maxPossibleDecision: 'conditional_go_exact_package', formalTakeover: 'blocked/no_go', realReleaseLoad: 'blocked/no_go', architecture: 'blocked/no_go' },
  environment: { node: process.version, platform: process.platform, arch: process.arch },
  scope: { allowedWrites, activeWritesNotConsumed: activeWrites, mediaBytesRead: false, mediaGenerated: false, networkAccess: false, gitHistoryOrRemoteAccess: false, formalFilesModified: false, webviewLaunched: false },
  packageSnapshot: pkg, prerequisites: prereqs, evidenceMatrix: matrix, formalGraph, requiredFields,
  candidateEvidence: Object.entries(evidence).map(([id, v]) => ({ id, path: result(evidenceNames[id]), status: v.status, candidateOnly: id !== 'TP18B1', formalPass: false, formalGaps: v.formalGaps ?? [], scenarioPass: v.scenarios ? v.scenarios.filter((s) => s.pass === true).length : null, scenarioTotal: v.scenarios?.length ?? null })),
  gateEvaluation: gate, controls: Object.fromEntries(Object.entries(controls).map(([k, v]) => [k, { formallyVerified: v, meaning: 'unproven formal prerequisite, not a claim that all legacy/candidate behavior fails' }])),
  commands, replayProvenance, negativeFixtures: fixtures, pathNegativeFixtures, blockers, checks,
  validatorProvenance: ['validate.mjs', 'gate.mjs'].map((file) => ({ path: `${own}${file}`, sha256: hash(fs.readFileSync(abs(`${own}${file}`))) })),
  futureFormalWhitelist: { proposalOnly: true, authorizedNow: false, files: futureWhitelist, replacesPlanningClientPathWithActualAppEntry: true, originalPlanningConflicts: evidence.TP18B1.conflictGraph.edges.filter((edge) => edge.a === pkg.id || edge.b === pkg.id), differences: { removeUnlessExplicitlyRequired: ['server-rs/ui/index.html', 'server-rs/release-ui/js/client.js'], add: [app, renderer, 'server-rs/tests/world_v3_contract_test.rs', 'server-rs/tests/continuous_capability_test.rs', 'server-rs/tests/continuous_capability_hud_test.rs'] }, condition: 'Replan and authorize exact package before implementation; Rust owner stays in world_v3.rs unless a separately specified module/lib.rs change is authorized. Coordinate world_v3.rs/campaign_main.rs overlap with scene/IPC/Campaign packages.' },
  rollback: { currentAudit: 'Remove only the three TP-01R7 whitelist outputs; original evidence and production files remain untouched.', futureFormal: rollback, additionalRequirements: 'Stop/close owner and revoke presentation before rollback; restore baseline accessibility/text fallback; rerun previous formal tests. No media/Git operation performed here.' },
  readOnlyInputs: unchangedInputs,
};
write(resultPath, `${JSON.stringify(output, null, 2)}\n`);
const table = matrix.map((r) => `| ${r.id} | ${r.currentStatus} | ${r.errorCode} | ${r.finding} |`).join('\n');
const testTable = commands.map((c) => `| ${c.id} | ${c.exitCode ?? 'not run'} | ${c.evidenceLevel} | ${c.scenarioCount ?? c.observedTests ?? '—'} |`).join('\n');
const report = `# TP-01R7 Formal Capability HUD exact prerequisite Gate review

## 结论

**blocked**。本包审计通过与正式接管是不同结论：auditPass=${output.auditPass}。六项包级 prerequisite 全部保持 blocked；正式接管、真实发布负载、整体 Architecture Gate 均为 blocked/no_go。即使未来满足精确包的前置条件，最高也只能 conditional_go_exact_package。

本次仅写 TP-01R7 三条白名单。未启动 WebView、联网、截图、读写媒体、改动正式 Rust/UI/renderer/Tauri/config，未读取 AG-38、TP-04C1 或 TP-01R6 的活跃内容。旧结果及读取的正式源码已做前后 SHA256 核验。

## 编号与正式接线核对

- TP-15A1/B1 实际为 HUD/map，不是 loot_scan；TP-14A1/B1/C1 为 Campaign progression/world routing，不是 HUD bridge。loot_scan 没有可复核的对应正式实现或候选契约。
- 发布入口 tauri.conf.json:7 指向 release-ui；index.html:34/40 加载 tps-renderer.js、renderer-boot.js、app.js。规划中的 release-ui/js/capability-resolver.js 与 release-ui/js/client.js 均不存在。
- world_v3.rs:222-232、366-377 有 WorldStateV3/WorldView，未提供正式 capability/grant/revoke/HUD projection。
- app.js:668-679 已有 legacy 动作队列上限 3；TP-17B1/C1 候选队列上限 8、payload 64、reject_newest 属于另一契约。app.js:681-719 的 action completion 不能证明 generation 隔离。
- campaign.rs:2157-2172 已有技能拥有权、冷却与能量校验；本报告只阻断 capability cooldown 接线及绕过测试，绝不声称旧技能没有冷却。
- index.html:13/23-25 有 ARIA，style.css:330-332 有 reduced-motion；仍缺正式 capability 无障碍投影与人工验收。
- app.js:904-910 的 TP06A callback 只更新诊断，未驱动正式 HUD；912-915 的异步启动没有正式代际接管证明。renderer 普通 dispose 存在（tps-renderer.js:1224-1264），但不证明尚不存在的 rear_view/loot_scan 生命周期。

## 精确前置矩阵

机器结果中每行包含 evidence path/hash、formal owner/file、requiredTests、rollback、pass、blocker/error code；前六行逐一对应 TP-18B1 原始 prerequisites。

| 前置项 | 状态 | error code | 当前证据边界 |
|---|---|---|---|
${table}

TP-17A1 的 queue/lifecycle 场景仍是失败/阻断；TP-17B1/C1 补充候选接口和真实 WebView 观测，不能倒改 A1 或取得正式 owner 权限。TP-15B1 的 pagehide 未观测，TP-17C1 的 dependency/Channel/GPU counters 未公开部分仍是 unknown。真实 WebView、605 callback 或 36000 synthetic tick 都不能当作正式发布负载。

## 复跑及边界

AG-12、TP-13A1、TP-15A1 使用 SHA256 相同的原样源码副本，在本白名单内重跑；Node 文件权限只允许读隔离副本、写隔离 results。Babylon 仅复制现有 JavaScript 源码，NullEngine 没有媒体读写或图片生成。原始 validators 不在原位运行，原始结果未改写。

Rust 条目直接重跑已存在、审阅为纯内存的 test binary，记录 binary/source hash 与时间。它们没有重新编译，不能证明当前源码已构建，也不能代替正式 Cargo 全套测试。完整 cargo test 未运行：build.rs 调用 Tauri build，可能读 icon/media 并产生白名单外输出；保留 E_FORMAL_TESTS_NOT_RECOMPILED。真实 WebView 历史 JSON 仅重新审阅，没有把历史退出码伪造成本次运行。

| 复跑 | 退出码 | 证据级别 | 场景/断言/测试数 |
|---|---:|---|---:|
${testTable}

Gate 负向夹具 **${fixtures.passed}/${fixtures.total}** 通过（failed=${fixtures.failed}）；每个夹具从明确标记为 synthetic 的完整正向对照出发，真正修改输入并断言 blocked 与指定 error code。另有 ${pathNegativeFixtures.filter((f) => f.pass).length}/${pathNegativeFixtures.length} 个禁读路径负向夹具通过，覆盖 traversal、反斜杠、大小写、绝对路径及媒体扩展名，测试过程不访问被禁止文件。正向对照只校验 gate evaluator，不证明工程 ready。

## 最小未来正式白名单（建议，未授权、未实施）

${futureWhitelist.map((p) => `- \`${p}\``).join('\n')}

相对 TP-18B1 原计划，应将不存在的 release client.js 替换为实际 app.js；加入 TPS renderer 与准确测试路径。旧 ui/index.html 无需默认纳入。若把 Rust owner 拆成新 module，必须先另列模块及 lib.rs 写入路径。world_v3.rs、campaign_main.rs 与其他正式包存在重叠，须先重新调度，不能直接并行写入。TP-01R6 本身不在此处消费，也不代表发布授权。

## Rollback

当前审计只移除自身白名单产物即可。未来正式包按 TP-18B1 恢复 capability/release UI 文件，先停止 owner、撤销 presentation，保留基础无障碍与文字 fallback，并复跑正式回归；不修改候选 harness 与历史证据。本次没有执行 rollback 或任何 Git 历史操作。

## 验证入口

\`node --check ${own}validate.mjs\`

\`node ${own}validate.mjs\`

\`git diff --check -- ${reportPath} ${own.slice(0, -1)} ${resultPath}\`

机器结果：\`${resultPath}\`。
`;
write(reportPath, report);
console.log(JSON.stringify({ status: output.status, auditPass: output.auditPass, checks, negativeFixtures: { total: fixtures.total, passed: fixtures.passed, failed: fixtures.failed }, commands: commands.map(({ id, exitCode, pass }) => ({ id, exitCode, pass })), resultPath }, null, 2));
if (!output.auditPass) process.exitCode = 1;
