// Build orchestration is probeOnly. Never executes formal application binaries/build.rs.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export const own = path.dirname(fileURLToPath(import.meta.url));
export const root = path.resolve(own, '../../..');
export const relative = p => path.relative(root, p).replaceAll('\\', '/');
export const digest = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
export const inside = p => { const r = path.relative(own, path.resolve(p)); return r !== '..' && !r.startsWith(`..${path.sep}`) && !path.isAbsolute(r); };
export function write(p, content) {
  if (!inside(p)) throw new Error(`E_WRITE_SCOPE: ${p}`);
  fs.mkdirSync(path.dirname(p), { recursive: true }); fs.writeFileSync(p, content);
}
export const activePrefixes = [
  'tools/continuous-tps-architecture-probe/optional-elevator-owner/',
  'tools/continuous-tps-architecture-probe/loot-scan-view-candidate/',
  'tools/continuous-tps-architecture-probe/formal-package-ledger-freshness/',
];
export function forbidden(p) {
  return /(^|\/)(assets|data|media|public)(\/|$)|\.(png|jpg|jpeg|webp|ico|wav|mp3|mp4|glb|gltf)$/i.test(p)
    || activePrefixes.some(s => p.startsWith(s)) || /tp04d1|tp15c3|tp18c2|OPTIONAL_ELEVATOR_OWNER|LOOT_SCAN_VIEW_CANDIDATE|FORMAL_PACKAGE_LEDGER_FRESHNESS/i.test(p);
}
const results = 'tools/continuous-tps-architecture-probe/results/';
export const upstream = [
  'formal-capability-owner-boundary-tp01r10.json', 'formal-capability-hud-readiness-tp01r7.json',
  'loot-scan-contract-readiness-tp15c1.json', 'loot-scan-query-candidate-tp15c2.json',
  'formal-ipc-owner-readiness-tp01r4.json', 'formal-asset-release-readiness-tp01r6.json',
  'formal-perf-soak-readiness-tp01r9.json', 'capability-ag12.json',
].map(p => results + p);
export const selectedTests = ['world_v3_contract_test.rs', 'campaign_flow.rs', 'continuous_input_test.rs'];
export function inputPaths() {
  // All directly declared library modules are required by compiling the unchanged lib.rs.
  const lib = fs.readFileSync(path.join(root, 'server-rs/src/lib.rs'), 'utf8');
  const modules = [];
  function moduleSources(dir) {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) moduleSources(p);
      else if (e.isFile() && e.name.endsWith('.rs')) modules.push(relative(p));
    }
  }
  for (const m of lib.matchAll(/^pub mod (\w+);/gm)) {
    const file = `server-rs/src/${m[1]}.rs`;
    if (fs.existsSync(path.join(root, file))) modules.push(file);
    else moduleSources(path.join(root, 'server-rs/src', m[1]));
  }
  const list = [...upstream, 'tools/continuous-tps-architecture-probe/formal-capability-owner-boundary/future-boundary.json',
    'server-rs/Cargo.toml', 'server-rs/Cargo.lock', 'server-rs/build.rs', 'server-rs/src/lib.rs', 'server-rs/src/campaign_main.rs', ...modules,
    ...selectedTests.map(t => 'server-rs/tests/' + t), ...['js/app.js', 'js/tps-renderer.js', 'js/renderer-boot.js', 'index.html', 'css/style.css'].map(p => 'server-rs/release-ui/' + p)];
  return [...new Set(list)].sort();
}
export function snapshot() {
  return inputPaths().map(p => {
    if (forbidden(p) || p.includes('..')) throw new Error(`E_READ_SCOPE: ${p}`);
    return { path: p, sha256: digest(path.join(root, p)) };
  });
}
export function runBuild() {
  const startedAt = new Date().toISOString();
  const runDir = path.join(own, 'runs', startedAt.replaceAll(/[:.]/g, '-'));
  const target = path.join(runDir, 'target'), temp = path.join(runDir, 'temp'), cargoHome = path.join(own, 'cargo-home');
  for (const p of [target, temp, cargoHome]) { if (!inside(p)) throw new Error('E_TARGET_SCOPE'); fs.mkdirSync(p, { recursive: true }); }
  const emptyTargetAtStart = fs.readdirSync(target).length === 0;
  // Read already cached Rust dependencies offline. Locks/global-cache/TEMP stay in this package.
  // Cache archives are dependency inputs, not game media; no app resources are accessed.
  const originalCargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
  for (const kind of ['src', 'index', 'cache']) {
    const link = path.join(cargoHome, 'registry', kind), source = path.join(originalCargoHome, 'registry', kind);
    fs.mkdirSync(path.dirname(link), { recursive: true });
    if (!fs.existsSync(link)) fs.symlinkSync(source, link, 'junction');
  }
  const env = { ...process.env, CARGO_HOME: cargoHome, CARGO_TARGET_DIR: target, CARGO_NET_OFFLINE: 'true',
    CARGO_CACHE_AUTO_CLEAN_FREQUENCY: 'never', CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '2',
    TMP: temp, TEMP: temp, TMPDIR: temp, RUSTFLAGS: '', RUSTDOCFLAGS: '', RUSTC_WRAPPER: '', RUSTC_WORKSPACE_WRAPPER: '' };
  const inputsBefore = snapshot();
  const commands = [];
  function command(id, exe, args) {
    const start = new Date().toISOString();
    const r = spawnSync(exe, args, { cwd: root, env, encoding: 'utf8', maxBuffer: 100 * 1024 * 1024, timeout: 30 * 60 * 1000, windowsHide: true });
    const log = path.join(runDir, id + '.log');
    write(log, (r.stdout || '') + '\n--- STDERR ---\n' + (r.stderr || '') + (r.error ? '\n' + String(r.error) : ''));
    const entry = { id, exe, args, cwd: relative(root) || '.', startedAt: start, endedAt: new Date().toISOString(), exitCode: r.status, signal: r.signal, error: r.error ? String(r.error) : null, log: relative(log), logSha256: digest(log) };
    commands.push(entry);
    process.stdout.write(`${id}: exit=${r.status}\n`);
    write(path.join(runDir, 'progress.json'), JSON.stringify({ startedAt, target: relative(target), commands }, null, 2));
    return r;
  }
  const toolchain = {
    rustc: command('rustc-version', 'rustc', ['-vV']).stdout?.trim(),
    cargo: command('cargo-version', 'cargo', ['-V']).stdout?.trim(),
    active: command('active-toolchain', 'rustup', ['show', 'active-toolchain']).stdout?.trim(),
    node: process.version, hostPlatform: process.platform, hostArch: process.arch,
  };
  const manifest = relative(path.join(own, 'Cargo.toml'));
  const probeLock = path.join(own, 'Cargo.lock');
  if (!fs.existsSync(probeLock)) write(probeLock, fs.readFileSync(path.join(root, 'server-rs/Cargo.lock')));
  const metadataRun = command('metadata-offline', 'cargo', ['metadata', '--offline', '--manifest-path', manifest, '--filter-platform', 'x86_64-pc-windows-msvc', '--format-version', '1']);
  let metadata = null;
  try { metadata = JSON.parse(metadataRun.stdout); } catch { /* Recorded as baseline failure. */ }
  const fmt = command('cargo-fmt-check', 'cargo', ['fmt', '--manifest-path', manifest, '--check']);
  const test = command('cargo-test-locked', 'cargo', ['test', '--manifest-path', manifest, '--locked', '--message-format=json']);
  const messages = (test.stdout || '').split(/\r?\n/).flatMap(line => { try { return [JSON.parse(line)]; } catch { return []; } });
  const artifacts = messages.filter(m => m.reason === 'compiler-artifact').map(m => ({ packageId: m.package_id, target: m.target, profile: m.profile, features: m.features, fresh: m.fresh, filenames: m.filenames, executable: m.executable }));
  const compileFail = command('expect-owner-missing', 'cargo', ['check', '--manifest-path', manifest, '--locked', '--features', 'missing-api-expectation', '--bin', 'expect-formal-owner', '--message-format=json']);
  const missingConfirmed = compileFail.status !== 0 && /E0432/.test(compileFail.stdout || '') && /could not find.*capability_owner/.test(compileFail.stdout || '');
  const inputsAfter = snapshot();
  const hashesUnchanged = JSON.stringify(inputsBefore) === JSON.stringify(inputsAfter);
  const compiledLibrary = artifacts.find(a => a.target.name === 'wuxian_horror_ch1' && a.target.kind.includes('lib'));
  const expectedTestNames = ['formal_capability_source_build', 'world_v3_contract', 'campaign_flow', 'continuous_input'];
  const freshTestArtifacts = expectedTestNames.every(n => artifacts.some(a => a.target.name === n && a.profile.test && a.fresh === false));
  const artifactPathsInside = artifacts.every(a => [...a.filenames, ...(a.executable ? [a.executable] : [])].every(inside));
  const counts = [...(test.stdout || '').matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;/g)].map(m => ({ passed: +m[1], failed: +m[2], ignored: +m[3] }));
  const result = {
    schema: 'tp01r11_source_build_v1', probeOnly: true, applicationBuildExcluded: true,
    exclusionReason: 'Unconditional tauri_build application build/resource pipeline excluded to respect media policy; unchanged complete formal lib.rs compiled through source-only manifest. No native release claim.',
    startedAt, endedAt: new Date().toISOString(), runDir: relative(runDir), target: relative(target), temp: relative(temp), cargoHome: relative(cargoHome),
    env: Object.fromEntries(['CARGO_TARGET_DIR','CARGO_HOME','CARGO_NET_OFFLINE','CARGO_CACHE_AUTO_CLEAN_FREQUENCY','CARGO_INCREMENTAL','CARGO_BUILD_JOBS','TMP','TEMP','TMPDIR','RUSTFLAGS','RUSTDOCFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'].map(k => [k, env[k]])),
    registryMode: 'offline existing dependency source/index/cache read junctions; own Cargo locks/global-cache; no network or application media access',
    toolchain, commands, metadataTarget: metadata?.target_directory, formalLockSha256: digest(path.join(root, 'server-rs/Cargo.lock')),
    probeLockSha256: digest(probeLock), probeManifestSha256: digest(path.join(own, 'Cargo.toml')), baselineManifestSha256: digest(path.join(own, 'baseline/Cargo.toml')),
    resolvedFeatures: metadata?.resolve?.nodes.map(n => ({ id: n.id, features: n.features })) || [],
    selectedPackages: metadata?.packages.map(p => ({ id: p.id, name: p.name, version: p.version, source: p.source })) || [],
    requestedFeatures: [], profile: { name: 'test', optLevel: 0, debug: 0, incremental: false, releaseMeasured: false },
    inputsBefore, inputsAfter, hashesUnchanged, emptyTargetAtStart, compiledLibrary, artifacts, freshTestArtifacts, artifactPathsInside,
    testCounts: { suites: counts, totalPassed: counts.reduce((s,c) => s+c.passed,0), totalFailed: counts.reduce((s,c) => s+c.failed,0) },
    missingOwnerCompileFailure: { expected: 'E0432 unresolved capability_owner module', confirmed: missingConfirmed },
    buildPassed: metadataRun.status === 0 && fmt.status === 0 && test.status === 0 && compiledLibrary?.fresh === false && freshTestArtifacts && artifactPathsInside && hashesUnchanged && missingConfirmed && counts.length >= 4,
    architecture: 'blocked/no_go', formalReady: false,
  };
  write(path.join(own, 'build-evidence.json'), JSON.stringify(result, null, 2) + '\n');
  return result;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = runBuild();
  console.log(JSON.stringify({ buildPassed: result.buildPassed, runDir: result.runDir, tests: result.testCounts, missingOwner: result.missingOwnerCompileFailure, hashesUnchanged: result.hashesUnchanged }));
  if (!result.buildPassed) process.exitCode = 1;
}
