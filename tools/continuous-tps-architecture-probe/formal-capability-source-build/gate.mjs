// Pure evidence validation, not runtime behavior or formal-owner implementation.
const sha = value => typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);
const canonical = p => typeof p === 'string' && p && !p.includes('\\') && !p.startsWith('/') && !p.includes(':') && !p.split('/').some(s => !s || s === '.' || s === '..');
const scope = 'tools/continuous-tps-architecture-probe/formal-capability-source-build/';
const inactive = p => !/tp04d1|tp15c3|tp18c2|optional-elevator-owner|loot-scan-view-candidate|formal-package-ledger-freshness/i.test(p);
const notMedia = p => !/(^|\/)(assets|data|media|public)(\/|$)|\.(png|jpg|webp|ico|wav|mp3|mp4|glb|gltf)$/i.test(p);
export function assess(m, trusted) {
  const errors = [];
  const check = (ok, code) => { if (!ok) errors.push(code); };
  check(m.packageId === 'TP-01R11', 'E_PACKAGE');
  check(m.architecture === 'blocked/no_go' && m.formalReady === false && m.releaseAuthorized === false, 'E_GATE_ESCALATION');
  check(m.network === false && m.offline === true, 'E_NETWORK');
  check(m.media === false, 'E_MEDIA');
  check(m.prebuiltBinaryUsed === false && m.emptyTargetAtStart === true, 'E_PREBUILT');
  check(canonical(m.target) && m.target.startsWith(scope + 'runs/') && trusted.target === m.target, 'E_TARGET_SCOPE');
  check(m.compiledFromSource === true && trusted.compiledFromSource === true, 'E_RUST_BUILD_MISSING');
  check(m.testsRecompiled === true && trusted.testsRecompiled === true, 'E_TESTS_NOT_REBUILT');
  check(m.testPassed === true && trusted.testPassed === true && m.testCount > 0 && m.testCount === trusted.testCount, 'E_TESTS_MISSING');
  check(typeof m.toolchain === 'string' && m.toolchain.includes('rustc') && m.toolchain === trusted.toolchain, 'E_TOOLCHAIN');
  check(sha(m.lockHash) && m.lockHash === trusted.lockHash, 'E_LOCK_HASH');
  check(sha(m.probeLockHash) && m.probeLockHash === trusted.probeLockHash, 'E_PROBE_LOCK_HASH');
  check(sha(m.manifestHash) && m.manifestHash === trusted.manifestHash, 'E_MANIFEST_HASH');
  check(JSON.stringify(m.features) === JSON.stringify(trusted.features), 'E_FEATURE_DRIFT');
  check(JSON.stringify(m.profile) === JSON.stringify(trusted.profile), 'E_PROFILE_DRIFT');
  check(m.sourceHashes?.length > 0 && m.sourceHashes.length === trusted.sourceHashes?.length && m.sourceHashes.every(e => sha(e.sha256) && trusted.sourceHashes.some(t => t.path === e.path && t.sha256 === e.sha256)), 'E_SOURCE_HASH');
  check(m.inputHashesUnchanged === true && trusted.inputHashesUnchanged === true, 'E_SOURCE_DRIFT');
  check(m.symbolMatrix?.length >= 10 && m.symbolMatrix.length === trusted.symbolMatrix?.length && m.symbolMatrix.every(e => e.status === 'missing_formal_api' && trusted.symbolMatrix.some(t => t.id === e.id && t.status === e.status)), 'E_MISSING_SYMBOL_IGNORED');
  check(m.missingCompileExpected === true && trusted.missingCompileExpected === true, 'E_MISSING_COMPILE_PROOF');
  check(m.adapter?.probeOnly === true && m.adapter.implementsOwner === false && m.adapter.formalModule === false, 'E_SHIM_PROMOTION');
  check(m.futureFiles?.length === 14 && m.futureFiles.every(e => trusted.futureFiles.some(t => e.path === t.path && e.exists === t.exists && e.action === t.action)), 'E_FUTURE_FILE_FALSE');
  check(m.proofKind === 'fresh_rust_library_source_only', 'E_JS_AS_RUST');
  check(m.evidenceLevel === 'baseline_only', 'E_CANDIDATE_PROMOTION');
  check(m.commands?.length >= 3 && m.commands.every(c => c.startedAt && c.endedAt && Number.isInteger(c.exitCode) && c.log && sha(c.logSha256)) && trusted.commandLogsVerified === true, 'E_COMMAND_PROVENANCE');
  check(m.rollback?.join('|') === 'stop-probe|preserve-evidence|remove-only-own-generated-artifacts|retain-formal-sources', 'E_ROLLBACK');
  check(m.consumedPaths?.length > 0 && m.consumedPaths.every(p => canonical(p) && inactive(p) && notMedia(p) && trusted.consumedPaths.includes(p)), 'E_INPUT_SCOPE');
  check(m.outputPaths?.length > 0 && m.outputPaths.every(p => canonical(p) && (p.startsWith(scope) || p === 'tools/continuous-tps-architecture-probe/results/formal-capability-source-build-tp01r11.json' || p === 'docs/CONTINUOUS_FORMAL_CAPABILITY_SOURCE_BUILD_REVIEW.md')), 'E_WRITE_SCOPE');
  check(m.lootName === null && m.lootNumericPolicy === null && m.lootImplemented === false, 'E_LOOT_FREEZE');
  check(m.nativeReleaseMeasured === false && m.futureOwnerBuildMeasured === false, 'E_FORMAL_BUILD_PROMOTION');
  check(m.originalBlockersPreserved === true && trusted.originalBlockersPreserved === true, 'E_BLOCKER_ERASURE');
  return { status: errors.length ? 'baseline_build_failed' : 'baseline_built_formal_owner_missing', pass: errors.length === 0, errorCodes: [...new Set(errors)], formalReady: false, architecture: 'blocked/no_go', fixtureOnly: trusted.fixtureOnly === true };
}
export function fixtures() {
  const hash = 'a'.repeat(64), target = scope + 'runs/fixture/target';
  const sourceHashes = [{ path: 'server-rs/src/lib.rs', sha256: hash }];
  const symbolMatrix = Array.from({ length: 10 }, (_, i) => ({ id: 'missing-' + i, status: 'missing_formal_api' }));
  const futureFiles = Array.from({ length: 14 }, (_, i) => ({ path: 'future/file-' + i, exists: false, action: 'create' }));
  const m = { packageId: 'TP-01R11', architecture: 'blocked/no_go', formalReady: false, releaseAuthorized: false, network: false, offline: true, media: false,
    prebuiltBinaryUsed: false, emptyTargetAtStart: true, target, compiledFromSource: true, testsRecompiled: true, testPassed: true, testCount: 1,
    toolchain: 'rustc fixture', lockHash: hash, probeLockHash: hash, manifestHash: hash, features: [], profile: { name: 'test' }, sourceHashes,
    inputHashesUnchanged: true, symbolMatrix, missingCompileExpected: true, adapter: { probeOnly: true, implementsOwner: false, formalModule: false }, futureFiles,
    proofKind: 'fresh_rust_library_source_only', evidenceLevel: 'baseline_only', commands: Array.from({ length: 3 }, () => ({ startedAt: 'fixture', endedAt: 'fixture', exitCode: 0, log: scope + 'fixture.log', logSha256: hash })),
    rollback: ['stop-probe', 'preserve-evidence', 'remove-only-own-generated-artifacts', 'retain-formal-sources'], consumedPaths: ['server-rs/src/lib.rs'], outputPaths: [scope + 'fixture.json'],
    lootName: null, lootNumericPolicy: null, lootImplemented: false, nativeReleaseMeasured: false, futureOwnerBuildMeasured: false, originalBlockersPreserved: true };
  const f = { ...structuredClone(m), fixtureOnly: true, commandLogsVerified: true };
  const cases = [];
  const add = (id, expected, mutate) => cases.push({ id, expected, mutate });
  add('precompiled-binary', 'E_PREBUILT', m => { m.prebuiltBinaryUsed = true; });
  add('nonempty-target', 'E_PREBUILT', m => { m.emptyTargetAtStart = false; });
  add('target-outside', 'E_TARGET_SCOPE', m => { m.target = 'server-rs/target'; });
  add('target-traversal', 'E_TARGET_SCOPE', m => { m.target = scope + '../target'; });
  add('no-rust-compile', 'E_RUST_BUILD_MISSING', m => { m.compiledFromSource = false; });
  add('untrusted-compile', 'E_RUST_BUILD_MISSING', (_, f) => { f.compiledFromSource = false; });
  add('old-test-binary', 'E_TESTS_NOT_REBUILT', m => { m.testsRecompiled = false; });
  add('no-tests', 'E_TESTS_MISSING', m => { m.testCount = 0; });
  add('missing-lock', 'E_LOCK_HASH', m => { delete m.lockHash; });
  add('wrong-lock', 'E_LOCK_HASH', m => { m.lockHash = 'b'.repeat(64); });
  add('missing-probe-lock', 'E_PROBE_LOCK_HASH', m => { delete m.probeLockHash; });
  add('missing-toolchain', 'E_TOOLCHAIN', m => { delete m.toolchain; });
  add('wrong-toolchain', 'E_TOOLCHAIN', m => { m.toolchain = 'rustc invented'; });
  add('manifest-drift', 'E_MANIFEST_HASH', m => { delete m.manifestHash; });
  add('feature-drift', 'E_FEATURE_DRIFT', m => { m.features = ['release']; });
  add('profile-drift', 'E_PROFILE_DRIFT', m => { m.profile.name = 'release'; });
  add('source-hash-missing', 'E_SOURCE_HASH', m => { m.sourceHashes = []; });
  add('source-hash-drift', 'E_SOURCE_HASH', m => { m.sourceHashes[0].sha256 = 'b'.repeat(64); });
  add('concurrent-source-drift', 'E_SOURCE_DRIFT', m => { m.inputHashesUnchanged = false; });
  add('missing-symbol-ignored', 'E_MISSING_SYMBOL_IGNORED', m => { m.symbolMatrix[0].status = 'present'; });
  add('missing-compile-failure', 'E_MISSING_COMPILE_PROOF', m => { m.missingCompileExpected = false; });
  add('shim-as-formal', 'E_SHIM_PROMOTION', m => { m.adapter.formalModule = true; });
  add('shim-owner-implementation', 'E_SHIM_PROMOTION', m => { m.adapter.implementsOwner = true; });
  add('future-as-existing', 'E_FUTURE_FILE_FALSE', m => { m.futureFiles[0].exists = true; });
  add('js-syntax-as-build', 'E_JS_AS_RUST', m => { m.proofKind = 'node_syntax'; });
  add('candidate-to-formal', 'E_CANDIDATE_PROMOTION', m => { m.evidenceLevel = 'formal'; });
  add('command-provenance-missing', 'E_COMMAND_PROVENANCE', m => { m.commands = []; });
  add('command-log-tamper', 'E_COMMAND_PROVENANCE', (_, f) => { f.commandLogsVerified = false; });
  add('rollback-missing', 'E_ROLLBACK', m => { m.rollback = []; });
  add('active-tp04d1', 'E_INPUT_SCOPE', m => { m.consumedPaths.push('tools/continuous-tps-architecture-probe/results/optional-elevator-owner-tp04d1.json'); });
  add('active-tp15c3', 'E_INPUT_SCOPE', m => { m.consumedPaths.push('tools/continuous-tps-architecture-probe/loot-scan-view-candidate/a.json'); });
  add('active-tp18c2', 'E_INPUT_SCOPE', m => { m.consumedPaths.push('tools/continuous-tps-architecture-probe/formal-package-ledger-freshness/a.json'); });
  add('media-path', 'E_INPUT_SCOPE', m => { m.consumedPaths.push('assets/example.png'); });
  add('media-enabled', 'E_MEDIA', m => { m.media = true; });
  add('network-enabled', 'E_NETWORK', m => { m.network = true; });
  add('offline-disabled', 'E_NETWORK', m => { m.offline = false; });
  add('formal-write', 'E_WRITE_SCOPE', m => { m.outputPaths.push('server-rs/src/capability_owner.rs'); });
  add('write-traversal', 'E_WRITE_SCOPE', m => { m.outputPaths.push(scope + '../escape'); });
  add('loot-name-frozen', 'E_LOOT_FREEZE', m => { m.lootName = 'approved'; });
  add('loot-policy-frozen', 'E_LOOT_FREEZE', m => { m.lootNumericPolicy = {}; });
  add('release-build-claimed', 'E_FORMAL_BUILD_PROMOTION', m => { m.nativeReleaseMeasured = true; });
  add('future-owner-build-claimed', 'E_FORMAL_BUILD_PROMOTION', m => { m.futureOwnerBuildMeasured = true; });
  add('blocker-erasure', 'E_BLOCKER_ERASURE', m => { m.originalBlockersPreserved = false; });
  add('global-gate-escalation', 'E_GATE_ESCALATION', m => { m.architecture = 'go'; });
  add('formal-ready', 'E_GATE_ESCALATION', m => { m.formalReady = true; });
  add('release-authorized', 'E_GATE_ESCALATION', m => { m.releaseAuthorized = true; });
  const rows = cases.map(c => { const model = structuredClone(m), facts = structuredClone(f); c.mutate(model, facts); const r = assess(model, facts); return { id: c.id, expected: c.expected, ...r, fixturePassed: !r.pass && r.errorCodes.includes(c.expected) }; });
  const positive = assess(m, f);
  return { fixtureOnly: true, readinessEvidence: false, positive, total: rows.length, passed: rows.filter(r => r.fixturePassed).length, pass: positive.pass && rows.every(r => r.fixturePassed), rows };
}
