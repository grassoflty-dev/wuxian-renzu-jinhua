// Pure boundary review. No runtime owner, capability registration or file I/O.
export const RULES = Object.freeze({
  singleOwner: 'E_DOUBLE_OWNER', epoch: 'E_EPOCH_MISSING', generation: 'E_GENERATION_MISSING',
  staleCallbackBarrier: 'E_STALE_CALLBACK', grantIdempotence: 'E_GRANT_MISSING', revokeIdempotence: 'E_REVOKE_MISSING',
  cleanupOrder: 'E_CLEANUP_MISSING', queueBounded: 'E_QUEUE_BYPASS', cooldownAuthority: 'E_COOLDOWN_BYPASS',
  accessibilityIndependent: 'E_ACCESSIBILITY_MISSING', hudSemanticFields: 'E_HUD_FIELDS_MISSING',
  extensionRegistrationExplicit: 'E_EXTENSION_IMPLICIT', saveSemanticOnly: 'E_SAVE_BOUNDARY_MISSING',
});
const record = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);
const hash = (s) => typeof s === 'string' && /^[a-f0-9]{64}$/i.test(s);
const active = (s) => /ag[-_]?38|tp[-_]?15c2|tp[-_]?18c2r1/i.test(String(s));
const media = (s) => /(^|[/\\])(assets|data|media|public)([/\\]|$)|\.(png|jpg|jpeg|webp|ico|mp3|mp4|wav|glb|gltf|blend)$/i.test(String(s));
const pathKey = (s) => typeof s === 'string' && s && !s.includes('\\') && !s.startsWith('/') && !/[:*?\x00-\x1f]/.test(s) && !s.split('/').some((p) => !p || p === '.' || p === '..' || /[. ]$/.test(p)) ? s.toLowerCase() : null;
export function intersect(a, b) { return [...new Set(a)].filter((p) => b.includes(p)).sort(); }
export function closure(root, graph) {
  const seen = new Set(), stack = new Set(), edges = [], cycles = [], unknownPackages = [];
  function visit(id) {
    if (stack.has(id)) { cycles.push([...stack, id]); return; }
    if (seen.has(id)) return;
    seen.add(id);
    const node = graph[id];
    if (!node) { if (id.startsWith('PKG-')) unknownPackages.push(id); return; }
    stack.add(id);
    for (const dep of node.prerequisites) { edges.push({ from: id, to: dep }); visit(dep); }
    stack.delete(id);
  }
  visit(root); seen.delete(root);
  return { root, nodes: [...seen].sort(), edges, cycles, unknownPackages };
}
export function evaluateBoundary(input, facts) {
  const errors = [];
  const add = (code, detail) => errors.push({ code, detail });
  const m = record(input) ? input : {};
  const f = record(facts) ? facts : {};
  if (m.packageId !== 'TP-01R10') add('E_PACKAGE_ID', 'Exact boundary package required.');
  if (m.architecture !== 'blocked/no_go' || m.formalTakeover !== false || m.releaseAuthorized !== false) add('E_GATE_ESCALATION', 'Review never authorizes formal takeover or release.');
  if (m.media !== false) add('E_MEDIA_ACCESS', 'Media access is forbidden.');
  if (m.network !== false) add('E_NETWORK_ACCESS', 'Network access is forbidden.');
  if (!Array.isArray(m.owners) || m.owners.length !== 1 || m.owners[0]?.authority !== 'future_rust_single_owner') add('E_DOUBLE_OWNER', 'One proposed Rust authority only.');
  for (const [rule, code] of Object.entries(RULES)) if (m.rules?.[rule] !== true || f.verifiedRules?.[rule] !== true) add(code, `Boundary rule absent/unverified: ${rule}`);
  if (m.lootSlot?.name !== null || m.lootSlot?.numericPolicy !== null || m.lootSlot?.registered !== false || m.lootSlot?.implemented !== false) add('E_LOOT_POLICY_FROZEN', 'Loot name/parameters unresolved; no provider registration or implementation.');
  if (!Array.isArray(m.registeredCapabilities) || m.registeredCapabilities.some((id) => id !== 'perception.rear_view')) add('E_LOOT_REGISTRATION', 'No implicit additional capability.');
  const files = Array.isArray(m.files) ? m.files : [];
  if (!files.length) add('E_FILES_MISSING', 'Exact future files required.');
  for (const file of files) {
    if (!pathKey(file.path) || !f.futureWrites?.includes(file.path)) add('E_FUTURE_SCOPE', String(file.path));
    if (file.state === 'existing' && f.existingFiles?.[file.path] !== true) add('E_NONEXISTENT_AS_EXISTING', String(file.path));
    if (!['existing', 'future_proposed'].includes(file.state)) add('E_FILE_STATE', String(file.path));
    if (active(file.path)) add('E_ACTIVE_OUTPUT', String(file.path));
    if (media(file.path)) add('E_MEDIA_ACCESS', String(file.path));
  }
  if (new Set(files.map((v) => v.path)).size !== files.length) add('E_DUPLICATE_FILE', 'Repeated write path.');
  if (m.entry !== 'server-rs/release-ui/js/app.js' || f.actualEntry !== m.entry) add('E_FALSE_ENTRY', 'Actual release app.js required.');
  const deps = Array.isArray(m.dependencies) ? m.dependencies : [];
  for (const id of f.requiredDependencies ?? []) {
    const row = deps.find((d) => d.id === id), trusted = f.dependencyRegistry?.[id];
    if (!row) { add('E_DEPENDENCY_MISSING', id); continue; }
    if (row.status !== 'formal_pass' || trusted?.status !== 'formal_pass') add('E_PREREQUISITE_BLOCKED', id);
    if (trusted?.level !== 'formal' || !hash(trusted?.sourceHash)) add('E_CANDIDATE_FORMAL', id);
  }
  if (!(f.requiredDependencies?.length > 0)) add('E_DEPENDENCY_REGISTRY_MISSING', 'Required closure must be explicit.');
  if (deps.some((d) => !f.requiredDependencies?.includes(d.id)) || new Set(deps.map((d) => d.id)).size !== deps.length) add('E_DEPENDENCY_SET', 'Dependencies must match exact closure.');
  const waves = Array.isArray(m.waves) ? m.waves : [];
  const assignments = new Map();
  waves.forEach((wave, i) => {
    if (!Array.isArray(wave)) { add('E_WAVE_INVALID', String(i)); return; }
    for (const id of wave) {
      if (!f.packageWrites?.[id] || assignments.has(id)) add('E_WAVE_INVALID', id);
      assignments.set(id, i);
    }
    for (let a = 0; a < wave.length; a++) for (let b = a + 1; b < wave.length; b++) {
      const overlap = intersect(f.packageWrites?.[wave[a]] ?? [], f.packageWrites?.[wave[b]] ?? []);
      if (overlap.length) add('E_SAME_WAVE_WRITE_CONFLICT', `${wave[a]} / ${wave[b]}: ${overlap.join(',')}`);
    }
  });
  for (const id of Object.keys(f.packageWrites ?? {})) if (!assignments.has(id)) add('E_WAVE_MISSING', id);
  for (const [id, ds] of Object.entries(f.packageDependencies ?? {})) for (const d of ds) {
    if (f.packageWrites?.[d] && assignments.get(d) >= assignments.get(id)) add('E_DEPENDENCY_ORDER', `${d} before ${id}`);
  }
  if (!waves.length) add('E_WAVE_MISSING', 'Explicit future-only waves required.');
  if (m.dispatchNow !== false) add('E_DISPATCH_UNAUTHORIZED', 'No package dispatched by review.');
  if (!Array.isArray(m.tests) || !m.tests.length || m.tests.some((t) => !t.id || !t.command)) add('E_TEST_PLAN_MISSING', 'Named tests with commands required.');
  if (m.compilePolicy?.sourceRebuildRequired !== true || m.compilePolicy?.exactSourceHashesRequired !== true || m.compilePolicy?.prebuiltBinarySufficient !== false) add('E_TEST_COMPILATION_BYPASS', 'Prebuilt/syntax checks cannot prove fresh formal source compilation.');
  if (!Array.isArray(m.rollback) || m.rollback.length < 5 || m.rollback[0]?.id !== 'stop-intents' || m.rollback[1]?.id !== 'invalidate-generation') add('E_ROLLBACK_MISSING', 'Ordered stop/invalidate/cleanup/restore/verify required.');
  const p = Array.isArray(m.provenance) ? m.provenance : [];
  if (!p.length) add('E_PROVENANCE_MISSING', 'Independent path/hash registry required.');
  for (const e of p) {
    const trusted = f.provenance?.[e.path];
    if (active(e.id) || active(e.path)) add('E_ACTIVE_OUTPUT', e.id);
    if (!pathKey(e.path) || !f.exactReadonlyInputs?.includes(e.path)) add('E_READ_SCOPE', e.path);
    if (media(e.path)) add('E_MEDIA_ACCESS', e.path);
    if (!trusted || !hash(e.sha256) || trusted.sha256 !== e.sha256) add('E_PROVENANCE_MISSING', e.path);
    if (e.level === 'formal' && trusted?.level !== 'formal') add('E_CANDIDATE_FORMAL', e.path);
  }
  for (const p of m.actualWrites ?? []) if (!(p.startsWith('tools/continuous-tps-architecture-probe/formal-capability-owner-boundary/') || p === 'tools/continuous-tps-architecture-probe/results/formal-capability-owner-boundary-tp01r10.json' || p === 'docs/CONTINUOUS_FORMAL_CAPABILITY_OWNER_BOUNDARY_REVIEW.md') || !pathKey(p)) add('E_WRITE_SCOPE', p);
  return { decision: errors.length ? 'blocked_boundary' : 'boundary_ready_candidate_only', pass: errors.length === 0, errors, errorCodes: [...new Set(errors.map((e) => e.code))], architecture: 'blocked/no_go', formalTakeover: false, releaseAuthorized: false, fixtureOnly: f.fixtureOnly === true, readinessEvidence: f.fixtureOnly !== true };
}
export function runNegativeFixtures() {
  const owner = 'server-rs/src/capability_owner.rs', entry = 'server-rs/release-ui/js/app.js', shared = 'server-rs/src/lib.rs';
  const deps = ['PKG-IPC', 'PKG-ASSETS'];
  const evidence = 'fixture-only/completed-evidence.json';
  const sourceHash = 'a'.repeat(64);
  const facts = { fixtureOnly: true, actualEntry: entry, verifiedRules: Object.fromEntries(Object.keys(RULES).map((k) => [k, true])),
    futureWrites: [owner, entry, shared], existingFiles: { [owner]: false, [entry]: true, [shared]: true }, requiredDependencies: deps,
    dependencyRegistry: Object.fromEntries(deps.map((id) => [id, { status: 'formal_pass', level: 'formal', sourceHash }])),
    packageWrites: { 'PKG-IPC': [shared], 'PKG-ASSETS': ['asset-metadata.json'], 'PKG-OWNER': [shared, owner] },
    packageDependencies: { 'PKG-ASSETS': ['PKG-IPC'], 'PKG-OWNER': deps }, exactReadonlyInputs: [evidence], provenance: { [evidence]: { sha256: sourceHash, level: 'candidate' } } };
  const model = { packageId: 'TP-01R10', architecture: 'blocked/no_go', formalTakeover: false, releaseAuthorized: false, media: false, network: false,
    owners: [{ authority: 'future_rust_single_owner' }], rules: { ...facts.verifiedRules }, lootSlot: { name: null, numericPolicy: null, registered: false, implemented: false }, registeredCapabilities: ['perception.rear_view'],
    files: [{ path: owner, state: 'future_proposed' }, { path: entry, state: 'existing' }, { path: shared, state: 'existing' }], entry,
    dependencies: deps.map((id) => ({ id, status: 'formal_pass' })), waves: [['PKG-IPC'], ['PKG-ASSETS'], ['PKG-OWNER']], dispatchNow: false,
    tests: [{ id: 'fixture-build', command: 'fixture-only compilation' }], compilePolicy: { sourceRebuildRequired: true, exactSourceHashesRequired: true, prebuiltBinarySufficient: false },
    rollback: ['stop-intents', 'invalidate-generation', 'cleanup', 'restore', 'verify'].map((id) => ({ id })), provenance: [{ id: 'fixture', path: evidence, sha256: sourceHash, level: 'candidate' }], actualWrites: ['tools/continuous-tps-architecture-probe/formal-capability-owner-boundary/fixture.json'] };
  const positive = evaluateBoundary(model, facts);
  const cases = [];
  const add = (id, code, mutate) => cases.push({ id, code, mutate });
  for (const [rule, code] of Object.entries(RULES)) add(rule, code, (m) => { m.rules[rule] = false; });
  add('nonexistent-file-existing', 'E_NONEXISTENT_AS_EXISTING', (m) => { m.files[0].state = 'existing'; });
  add('double-owner', 'E_DOUBLE_OWNER', (m) => { m.owners.push({ ...m.owners[0] }); });
  add('false-entry', 'E_FALSE_ENTRY', (m) => { m.entry = 'server-rs/release-ui/js/client.js'; });
  add('loot-name-frozen', 'E_LOOT_POLICY_FROZEN', (m) => { m.lootSlot.name = 'frozen-name'; });
  add('loot-numbers-frozen', 'E_LOOT_POLICY_FROZEN', (m) => { m.lootSlot.numericPolicy = { range: 8 }; });
  add('loot-registered', 'E_LOOT_POLICY_FROZEN', (m) => { m.lootSlot.registered = true; });
  add('loot-implemented', 'E_LOOT_POLICY_FROZEN', (m) => { m.lootSlot.implemented = true; });
  add('hidden-provider', 'E_LOOT_REGISTRATION', (m) => { m.registeredCapabilities.push('new-provider'); });
  add('ipc-prereq-bypass', 'E_PREREQUISITE_BLOCKED', (_, f) => { f.dependencyRegistry['PKG-IPC'].status = 'blocked'; });
  add('asset-prereq-bypass', 'E_DEPENDENCY_MISSING', (m) => { m.dependencies.pop(); });
  add('candidate-prereq-promotion', 'E_CANDIDATE_FORMAL', (_, f) => { f.dependencyRegistry['PKG-IPC'].level = 'candidate'; });
  add('same-wave-conflict', 'E_SAME_WAVE_WRITE_CONFLICT', (m) => { m.waves = [['PKG-IPC', 'PKG-OWNER'], ['PKG-ASSETS']]; });
  add('dependency-order', 'E_DEPENDENCY_ORDER', (m) => { m.waves.reverse(); });
  add('active-output', 'E_ACTIVE_OUTPUT', (m) => { m.provenance[0].id = 'TP-15C2'; });
  add('active-ag38', 'E_ACTIVE_OUTPUT', (m) => { m.provenance[0].id = 'AG-38'; });
  add('active-tp18c2r1', 'E_ACTIVE_OUTPUT', (m) => { m.provenance[0].id = 'TP-18C2R1'; });
  add('write-scope', 'E_WRITE_SCOPE', (m) => { m.actualWrites = ['server-rs/src/lib.rs']; });
  add('write-traversal', 'E_WRITE_SCOPE', (m) => { m.actualWrites[0] = 'tools/continuous-tps-architecture-probe/formal-capability-owner-boundary/../escape.json'; });
  add('media', 'E_MEDIA_ACCESS', (m) => { m.media = true; });
  add('network', 'E_NETWORK_ACCESS', (m) => { m.network = true; });
  add('missing-tests', 'E_TEST_PLAN_MISSING', (m) => { m.tests = []; });
  add('no-fresh-build', 'E_TEST_COMPILATION_BYPASS', (m) => { m.compilePolicy.prebuiltBinarySufficient = true; });
  add('missing-rollback', 'E_ROLLBACK_MISSING', (m) => { m.rollback = []; });
  add('rollback-order', 'E_ROLLBACK_MISSING', (m) => { m.rollback.reverse(); });
  add('missing-provenance', 'E_PROVENANCE_MISSING', (m) => { m.provenance = []; });
  add('hash-drift', 'E_PROVENANCE_MISSING', (m) => { m.provenance[0].sha256 = 'b'.repeat(64); });
  add('candidate-promoted', 'E_CANDIDATE_FORMAL', (m) => { m.provenance[0].level = 'formal'; });
  add('overall-gate', 'E_GATE_ESCALATION', (m) => { m.architecture = 'go'; });
  add('takeover', 'E_GATE_ESCALATION', (m) => { m.formalTakeover = true; });
  add('release', 'E_GATE_ESCALATION', (m) => { m.releaseAuthorized = true; });
  add('dispatch', 'E_DISPATCH_UNAUTHORIZED', (m) => { m.dispatchNow = true; });
  add('unverified-rule', 'E_EPOCH_MISSING', (_, f) => { delete f.verifiedRules.epoch; });
  const results = cases.map(({ id, code, mutate }) => {
    const m = structuredClone(model), f = structuredClone(facts); mutate(m, f);
    const r = evaluateBoundary(m, f);
    return { id, expectedCode: code, errorCodes: r.errorCodes, decision: r.decision, pass: r.decision === 'blocked_boundary' && r.errorCodes.includes(code) && r.fixtureOnly && !r.readinessEvidence };
  });
  return { fixtureOnly: true, readinessEvidence: false, positiveControl: positive, total: results.length, passed: results.filter((r) => r.pass).length, pass: positive.pass && !positive.readinessEvidence && results.every((r) => r.pass), results };
}
