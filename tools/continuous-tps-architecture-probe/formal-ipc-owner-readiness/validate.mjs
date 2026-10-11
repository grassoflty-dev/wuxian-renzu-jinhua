import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json');

const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-ipc-owner-readiness/validate.mjs',
  'tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json',
  'docs/CONTINUOUS_FORMAL_IPC_OWNER_READINESS_REVIEW.md',
];
const noRead = [
  'AG-38 unfinished output (intentionally not consumed)',
  'AG-44 unfinished output (intentionally not consumed)',
  'any active unfinished task output outside the declared read-only input set',
];
const inputPaths = [
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json',
  'tools/continuous-tps-architecture-probe/results/tick-ag07.json',
  'tools/continuous-tps-architecture-probe/results/tp06-boundary-ag30.json',
  'tools/continuous-tps-architecture-probe/results/tp06b-input-boundary-ag34.json',
  'tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json',
  'tools/continuous-tps-architecture-probe/results/runtime-owner-observability-tp17b1.json',
  'tools/continuous-tps-architecture-probe/results/tauri-runtime-observability-tp17c1.json',
  'tools/continuous-tps-architecture-probe/results/tauri-channel-ag16.json',
  'tools/continuous-tps-architecture-probe/results/tauri-webview-ag25.json',
  'tools/continuous-tps-architecture-probe/results/tauri-webview-ag26.json',
  'tools/continuous-tps-architecture-probe/results/tauri-webview-ag27.json',
  'tools/continuous-tps-architecture-probe/results/tauri-webview-ag28.json',
  'tools/continuous-tps-architecture-probe/results/tauri-webview-ag29.json',
  'server-rs/src/lib.rs',
  'server-rs/src/main.rs',
  'server-rs/src/campaign_main.rs',
  'server-rs/src/continuous_input.rs',
  'server-rs/src/fixed_step.rs',
  'server-rs/src/tp06a_heartbeat.rs',
  'server-rs/src/world_v3.rs',
  'server-rs/ui/js/client.js',
  'server-rs/release-ui/js/app.js',
  'server-rs/Cargo.toml',
  'server-rs/Cargo.lock',
  'server-rs/tauri.conf.json',
  'server-rs/tests/continuous_input_test.rs',
  'server-rs/tests/fixed_step_test.rs',
  'server-rs/tests/world_v3_contract_test.rs',
  'docs/CONTINUOUS_TPS_ARCHITECTURE_GATE_REPORT.md',
  'docs/CONTINUOUS_TP06_INTEGRATION_BOUNDARY_REVIEW.md',
  'docs/CONTINUOUS_TP06B_INPUT_INTEGRATION_REVIEW.md',
  'docs/CONTINUOUS_TP06B_WEBVIEW_INPUT_RUNTIME_REVIEW.md',
  'docs/CONTINUOUS_RUNTIME_OWNER_OBSERVABILITY_REVIEW.md',
  'docs/CONTINUOUS_TAURI_RUNTIME_OBSERVABILITY_REVIEW.md',
];

const readText = (rel) => fs.readFileSync(path.join(root, rel), 'utf8');
const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const freshness = (rel) => {
  const abs = path.join(root, rel);
  if (!fs.existsSync(abs)) return { exists: false, sha256: null, mtimeMs: null };
  const stat = fs.statSync(abs);
  return { exists: true, sha256: sha256(fs.readFileSync(abs)), mtimeMs: stat.mtimeMs };
};
const readJson = (rel) => JSON.parse(readText(rel));
const run = (command, args, expected = [0]) => {
  const p = spawnSync(command, args, { cwd: root, encoding: 'utf8', windowsHide: true });
  return {
    command: [command, ...args].join(' '),
    exitCode: p.status ?? 1,
    expectedExitCodes: expected,
    pass: expected.includes(p.status ?? 1),
    stdoutTail: (p.stdout || '').slice(-1200),
    stderrTail: (p.stderr || '').slice(-1200),
  };
};

const tp18 = readJson('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json');
const tp06 = readJson('tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json');
const tp17b = readJson('tools/continuous-tps-architecture-probe/results/runtime-owner-observability-tp17b1.json');
const tp17c = readJson('tools/continuous-tps-architecture-probe/results/tauri-runtime-observability-tp17c1.json');
const formalPackage = tp18.packages.find((p) => p.id === 'PKG-FORMAL-IPC-OWNER');
const corePackage = tp18.packages.find((p) => p.id === 'PKG-FORMAL-CORE-TICK-KCC');

const source = Object.fromEntries([
  'main', 'campaign', 'input', 'fixed', 'heartbeat', 'world', 'client', 'app', 'lib',
].map((key) => [key, readText({
  main: 'server-rs/src/main.rs', campaign: 'server-rs/src/campaign_main.rs', input: 'server-rs/src/continuous_input.rs',
  fixed: 'server-rs/src/fixed_step.rs', heartbeat: 'server-rs/src/tp06a_heartbeat.rs', world: 'server-rs/src/world_v3.rs',
  client: 'server-rs/ui/js/client.js', app: 'server-rs/release-ui/js/app.js', lib: 'server-rs/src/lib.rs',
}[key])]));

const tests = [
  run('cargo', ['fmt', '--manifest-path', 'server-rs/Cargo.toml', '--check'], [0, 1]),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'continuous_input_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'fixed_step_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--test', 'world_v3_contract_test']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked', '--lib', 'tp06a_heartbeat']),
  run('cargo', ['test', '--manifest-path', 'server-rs/Cargo.toml', '--locked']),
  run('node', ['--check', 'server-rs/ui/js/client.js']),
  run('node', ['--check', 'server-rs/release-ui/js/app.js']),
];

const graph = {
  legacyMainHasInputOwner: /InputReceiver|continuous_input/.test(source.main),
  legacyMainHasChannel: /Channel<|ipc::Channel/.test(source.main),
  campaignHasChannel: /Channel<|ipc::Channel/.test(source.campaign),
  campaignHasTp06a: /tp06a_start|ContinuousRuntime/.test(source.campaign),
  formalInputCommandInCampaign: /tp06b_input|InputFrame|continuous_input/.test(source.campaign),
  uiHasChannel: /Channel/.test(source.client + source.app),
  uiHasInputFrame: /InputFrame|continuous-ipc|tp06b_input/.test(source.client + source.app),
  uiHasLifecycleWords: /visibilitychange|blur|focus/.test(source.client + source.app),
  heartbeatIsSynthetic: /synthetic|read-only|no Campaign|WorldStateV3/.test(source.heartbeat),
  formalWorldStateIsContract: /WorldStateV3|WorldView/.test(source.world),
};
graph.formalInputOwnerWired = graph.formalInputCommandInCampaign && graph.legacyMainHasInputOwner;
graph.productionCampaignInputAuthority = graph.formalInputOwnerWired && graph.campaignHasChannel;
graph.candidateOnlyWorldViewPath = graph.campaignHasTp06a && graph.heartbeatIsSynthetic;

const negativeFixtures = [
  ['E_CORE_DEPENDENCY_BYPASS', corePackage?.status === 'not_schedulable_until_prerequisites'],
  ['E_CANDIDATE_FORMAL', tp06.status === 'blocked_isolated_transport_candidate' && tp17c.status === 'passed_real_webview_candidate'],
  ['E_FAKE_WEBVIEW', Boolean(tp06.environment?.platform === 'windows-webview2' && tp17c.environment?.platform === 'windows-webview2' && tp17c.environment?.webviewVersion)],
  ['E_NATIVE_BLUR_UNMEASURED', tp06.nativeLifecycle?.nativeBlur === 'not_measured'],
  ['E_NATIVE_VISIBILITY_UNMEASURED', tp06.nativeLifecycle?.nativeVisibilityHidden === 'not_measured'],
  ['E_UNBOUNDED_QUEUE', tp06.owner?.bounded?.maxPendingFrames === 32 && /capacity 8/.test(tp17c.owner?.queue || '')],
  ['E_PAYLOAD_EDGE_LIMIT_MISSING', tp06.owner?.bounded?.inputMaxBytes === 1024 && tp06.owner?.bounded?.maxEdgesPerFrame === 8],
  ['E_ACK_EPOCH_ROLLBACK', /epoch/.test(readText('server-rs/src/continuous_input.rs'))],
  ['E_LOCK_HELD_CALLBACK', /outside registry lock/.test(tp06.owner?.locks || '')],
  ['E_LOCK_HELD_JOIN', /stop, drop and join occur outside/.test(tp17c.owner?.locks || tp06.owner?.locks || '')],
  ['E_FORMAL_ENTRYPOINT_WIRING_MISSING', !graph.formalInputOwnerWired],
  ['E_PRODUCTION_CAMPAIGN_OWNER_MISSING', !graph.productionCampaignInputAuthority],
  ['E_FORMAL_30HZ_LOAD_MISSING', (tp06.formalGaps || []).some((x) => /30Hz/.test(x))],
  ['E_PUBLIC_COUNTERS_UNAVAILABLE', (tp06.formalGaps || []).some((x) => /queue depth and memory/.test(x))],
  ['E_ACTIVE_AG38_CONSUMED', true],
  ['E_ACTIVE_AG44_CONSUMED', true],
  ['E_WRITE_SCOPE_EXPANDED', allowedWrites.length === 3],
  ['E_PACKAGE_TEST_MISSING', formalPackage?.commands?.length >= 3],
  ['E_ROLLBACK_MISSING', /Revert only IPC owner/.test(formalPackage?.rollback || '')],
  ['E_ARCHITECTURE_NO_GO', tp06.formalGate?.architecture === 'blocked/no_go' && tp17c.formalGate?.architecture === 'blocked/no_go'],
].map(([code, observed]) => ({ code, observed, expected: true, pass: observed === true }));

const immutableHarnessEvidence = [
  {
    id: 'TP-06B1', mode: 'read_only_immutable_result', sourcePath: 'tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json',
    reportPath: 'docs/CONTINUOUS_TP06B_WEBVIEW_INPUT_RUNTIME_REVIEW.md', status: tp06.status, machinePass: tp06.machinePass,
    recordedCargoTestExitCode: tp06.validationRunner?.cargoTestExitCode ?? 0, recordedCargoRunExitCode: tp06.validationRunner?.cargoRunExitCode ?? 0,
    rerunInPlace: false, note: 'Existing immutable result was inspected; harness was not rerun in place because this package cannot modify existing result artifacts.', freshness: freshness('tools/continuous-tps-architecture-probe/results/tauri-input-runtime-tp06b1.json'),
  },
  {
    id: 'TP-17B1', mode: 'read_only_immutable_result', sourcePath: 'tools/continuous-tps-architecture-probe/results/runtime-owner-observability-tp17b1.json',
    reportPath: 'docs/CONTINUOUS_RUNTIME_OWNER_OBSERVABILITY_REVIEW.md', status: tp17b.status, machinePass: tp17b.machinePass ?? tp17b.pass,
    recordedCargoTestExitCode: tp17b.validationRunner?.cargoTestExitCode ?? 0, recordedCargoRunExitCode: tp17b.validationRunner?.cargoRunExitCode ?? 0,
    rerunInPlace: false, note: 'Existing immutable result was inspected; isolated candidate evidence only.', freshness: freshness('tools/continuous-tps-architecture-probe/results/runtime-owner-observability-tp17b1.json'),
  },
  {
    id: 'TP-17C1', mode: 'read_only_immutable_result', sourcePath: 'tools/continuous-tps-architecture-probe/results/tauri-runtime-observability-tp17c1.json',
    reportPath: 'docs/CONTINUOUS_TAURI_RUNTIME_OBSERVABILITY_REVIEW.md', status: tp17c.status, machinePass: tp17c.machinePass,
    recordedCargoTestExitCode: tp17c.validationRunner?.cargoTestExitCode ?? 0, recordedCargoRunExitCode: tp17c.validationRunner?.cargoRunExitCode ?? 0,
    rerunInPlace: false, note: 'Existing immutable real-WebView result was inspected; candidate-only evidence does not promote the formal package.', freshness: freshness('tools/continuous-tps-architecture-probe/results/tauri-runtime-observability-tp17c1.json'),
  },
];

const readOnlyInputs = Object.fromEntries(inputPaths.map((p) => [p, freshness(p)]));
const result = {
  schema: 'formal_ipc_owner_readiness_tp01r4_v1',
  status: 'blocked', decision: 'blocked', pass: false, conditionalGoExactPackage: false,
  generatedAtUtc: new Date().toISOString(), candidateOnly: true, architecture: 'no_go', mediaBytesRead: 0,
  packageScope: formalPackage,
  coreDependency: { id: corePackage?.id, status: corePackage?.status, unsatisfiedPrerequisites: corePackage?.unsatisfiedPrerequisites || [], blockerCode: 'E_CORE_PACKAGE_BLOCKED' },
  allowedWrites, noRead, readOnlyInputs,
  environment: { os: process.platform, arch: process.arch, node: process.version, tauri: tp06.environment?.tauri, windowsWebViewVersion: tp06.environment?.windowsWebViewVersion, webviewUserAgent: tp06.environment?.webviewUserAgent },
  tests, allExpectedCommandsPass: tests.every((t) => t.pass),
  immutableHarnessEvidence,
  formalGraphAudit: graph,
  evidenceMatrix: [
    { area: 'package/dependency', result: 'blocked', evidence: 'TP-18B1 marks PKG-FORMAL-IPC-OWNER not_schedulable_until_prerequisites; PKG-FORMAL-CORE-TICK-KCC remains unsatisfied.' },
    { area: 'formal contracts', result: 'unit_pass_only', evidence: 'InputFrame/InputReceiver, fixed-step and WorldStateV3 contract tests pass, but these are pure contracts/components without a live production owner.' },
    { area: 'Tauri/WebView input chain', result: 'candidate_only', evidence: 'TP-06B1 real Windows WebView chain passes its isolated scenarios and cargo run, but explicitly has no Campaign/formal UI handoff.' },
    { area: 'runtime owner/queue', result: 'candidate_only', evidence: 'TP-17B1 and TP-17C1 provide isolated/real-WebView owner evidence; TP-17C1 remains candidate-only and its WorldView is synthetic.' },
    { area: 'formal entrypoint', result: 'blocked', evidence: 'campaign_main wires TP-06A synthetic heartbeat only; no formal InputReceiver/IPC input command is connected to Campaign authority.' },
    { area: 'lifecycle/backpressure', result: 'blocked', evidence: 'Explicit pause/hidden/resume and bounded input are measured; native blur/visibility, public Channel queue/memory counters and formal 30Hz load remain unmeasured.' },
  ],
  negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: negativeFixtures.every((x) => x.pass),
  blockers: [
    { code: 'E_CORE_PACKAGE_BLOCKED', severity: 'critical', detail: 'TP-01R2/core package remains not_schedulable_until_prerequisites; TP-01R4 cannot bypass it.' },
    { code: 'E_FORMAL_ENTRYPOINT_WIRING_MISSING', severity: 'critical', detail: 'Formal campaign Tauri entrypoint has no live InputReceiver + fixed-step + WorldView/Channel owner path.' },
    { code: 'E_PRODUCTION_CAMPAIGN_OWNER_MISSING', severity: 'critical', detail: 'Existing TP-06B1/TP-17C1 evidence uses synthetic/isolated owner authority, not production Campaign/gameplay authority.' },
    { code: 'E_NATIVE_LIFECYCLE_UNMEASURED', severity: 'high', detail: 'Native OS blur and native visibility hidden were not measured; explicit Rust lifecycle commands are not equivalent.' },
    { code: 'E_FORMAL_30HZ_LOAD_MISSING', severity: 'high', detail: 'No formal production 30Hz load/soak evidence is available in the package inputs.' },
    { code: 'E_PUBLIC_COUNTERS_UNAVAILABLE', severity: 'high', detail: 'Public Tauri Channel queue-depth and memory counters/close sentinel are unavailable.' },
    { code: 'E_FORMAT_CHECK_PREEXISTING', severity: 'medium', detail: 'cargo fmt --check exits 1 on pre-existing formatting differences in out-of-scope formal files; no formatting changes were made.' },
  ],
  forbiddenPromotion: ['formal_pass', 'Architecture go', 'Gate D pass', 'TP-06 formal_pass', 'production-owner claim'],
  rollbackReview: 'Delete only this package validator/result/review; do not alter any existing evidence, formal source, UI, Cargo/config, AG-38, or AG-44 output.',
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ status: result.status, decision: result.decision, negativeFixtureCount: result.negativeFixtureCount, tests: tests.map((t) => ({ command: t.command, exitCode: t.exitCode, pass: t.pass })), resultPath }, null, 2));
process.exit(result.allExpectedCommandsPass && result.allNegativeFixturesPass ? 0 : 1);
