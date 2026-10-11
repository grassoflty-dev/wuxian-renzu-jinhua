/** TP-15C1. Pure candidate-contract evaluation; no runtime scan or filesystem I/O. */
export const PACKAGE_ID = 'TP-15C1';
export const PACKAGE_WRITES = Object.freeze([
  'tools/continuous-tps-architecture-probe/loot-scan-contract-readiness/**',
  'tools/continuous-tps-architecture-probe/results/loot-scan-contract-readiness-tp15c1.json',
  'docs/CONTINUOUS_LOOT_SCAN_CONTRACT_READINESS_REVIEW.md',
]);
export const REQUIRED_FIELDS = Object.freeze([
  'capability_id', 'request_id', 'owner_id', 'epoch', 'generation', 'tick',
  'origin', 'range', 'los_required', 'cooldown_until_tick', 'max_results',
  'results', 'result_id', 'distance', 'hud_label', 'accessibility_label',
]);
export const REQUIRED_CONTROLS = Object.freeze({
  singleRustOwner: 'SINGLE_RUST_OWNER_MISSING',
  grantIdempotent: 'DUPLICATE_GRANT',
  revokeIdempotent: 'DUPLICATE_REVOKE',
  epochBound: 'EPOCH_BINDING_MISSING',
  generationBound: 'GENERATION_BINDING_MISSING',
  tickBound: 'TICK_BINDING_MISSING',
  atomicEpochGenerationTick: 'THREE_KEY_BINDING_MISSING',
  staleTickRejected: 'STALE_TICK_ACCEPTED',
  rangeEnforced: 'RANGE_NOT_ENFORCED',
  losEnforced: 'LOS_NOT_ENFORCED',
  cooldownOwnedByRust: 'COOLDOWN_OWNER_MISSING',
  cooldownNotBypassable: 'COOLDOWN_BYPASS',
  stableSort: 'STABLE_SORT_MISSING',
  maxResultsEnforced: 'RESULT_LIMIT_MISSING',
  hiddenItemsExcluded: 'HIDDEN_ITEM_LEAK',
  internalStateExcluded: 'INTERNAL_STATE_LEAK',
  hudFieldsComplete: 'HUD_FIELDS_MISSING',
  accessibilityPresent: 'ACCESSIBILITY_MISSING',
  accessibilityIndependent: 'ACCESSIBILITY_CAPABILITY_GATED',
  saveDerivedResultsExcluded: 'SAVE_DERIVED_RESULTS_POLLUTION',
  saveHandlesExcluded: 'SAVE_HANDLE_POLLUTION',
});

const record = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const text = value => typeof value === 'string' && value.trim().length > 0;
const counter = value => Number.isSafeInteger(value) && value >= 0;
const hash = value => typeof value === 'string' && /^[a-f0-9]{64}$/i.test(value);
const active = value => typeof value === 'string' && /(?:ag[-_]?38(?!\d)|tp[-_]?01r9(?!\d)|tp[-_]?18c1(?!\d))/i.test(value);
const forbiddenSource = value => typeof value === 'string' && /(?:hud[_ -]?map|routing|legacy[_ /-]?q[_ /-]?r)/i.test(value);
const mediaPath = value => typeof value === 'string' && (
  /(?:^|[/\\])(?:assets|data|media|public)(?:[/\\]|$)/i.test(value)
  || /\.(?:png|jpe?g|gif|webp|bmp|svg|ico|avif|heic|mp4|mov|webm|mkv|avi|mp3|wav|ogg|flac|aac|glb|gltf|obj|fbx|blend)$/i.test(value)
);

function pathKey(value, allowPackageGlob = false) {
  if (!text(value) || value !== value.trim()) return null;
  const result = value.replaceAll('\\', '/');
  if (result.startsWith('/') || result.includes(':') || /[\x00-\x1f\x7f]/.test(result)) return null;
  if (result.split('/').some(part => !part || part === '.' || part === '..' || /[. ]$/.test(part))) return null;
  if (/[?*]/.test(result) && !(allowPackageGlob && result === PACKAGE_WRITES[0])) return null;
  return result.toLowerCase();
}

function insidePackage(path) {
  const key = pathKey(path, true);
  if (!key) return false;
  const prefix = PACKAGE_WRITES[0].slice(0, -2);
  return PACKAGE_WRITES.some(item => item.toLowerCase() === key)
    || (key.startsWith(prefix) && key.length > prefix.length);
}

/**
 * model: candidate claims, fields, one proposed Rust owner, provenance references,
 * controls, future package writes, actual readonlyInputs, tests and rollback.
 * context: caller's exactReadonlyInputs, independently checked trustedEvidence and
 * verifiedControls. Never derive trusted context from unverified model claims.
 * Each evidence record binds domain/sourceKind, contractHash, owner, counters, tests.
 */
export function evaluateContract(model, context) {
  const blockers = [];
  const add = (code, detail, extra = {}) => blockers.push({ code, detail, ...extra });
  if (!record(model)) add('MODEL_INVALID', 'Candidate contract must be an object.');
  if (!record(context)) add('CONTEXT_INVALID', 'Independent evidence context is required.');
  model = record(model) ? model : {};
  context = record(context) ? context : {};
  if (model.packageId !== PACKAGE_ID) add('PACKAGE_ID_MISMATCH', 'Only TP-15C1 may use this contract Gate.');
  if (model.capabilityId !== 'perception.loot_scan') add('CAPABILITY_ID_MISMATCH', 'The suggested capability ID must identify loot_scan.');
  if (model.capabilityIdStatus !== 'suggestion_only') add('CAPABILITY_ID_PROMOTION', 'The capability ID remains a suggestion.');
  if (model.scope !== 'candidate_contract') add('CANDIDATE_SCOPE_ESCALATION', 'Review scope must remain candidate_contract.');
  if (model.architecture !== 'blocked/no_go') add('ARCHITECTURE_GATE_ESCALATION', 'Architecture Gate must remain blocked/no_go.');
  if (model.formalTakeover !== false) add('CANDIDATE_FORMAL_PROMOTION', 'Candidate contract cannot authorize formal takeover.');
  if (model.media !== false) add('MEDIA_ACCESS_FORBIDDEN', 'Media access must explicitly remain false.');
  if (model.network !== false) add('NETWORK_ACCESS_FORBIDDEN', 'Network access must explicitly remain false.');

  const fields = Array.isArray(model.fields) ? model.fields : [];
  if (fields.length === 0) add('FIELDS_MISSING', 'An explicit nonempty field contract is required.');
  for (const field of REQUIRED_FIELDS) {
    if (!fields.includes(field)) add('REQUIRED_FIELD_MISSING', 'Required loot_scan field is absent.', { field });
  }
  if (fields.some(field => !text(field)) || new Set(fields).size !== fields.length) add('FIELDS_INVALID', 'Fields must be unique nonempty names.');

  const owners = Array.isArray(model.owners) ? model.owners : [];
  if (owners.length !== 1) add('SINGLE_RUST_OWNER_MISSING', 'Exactly one proposed Rust owner is required.');
  const owner = owners.length === 1 && record(owners[0]) ? owners[0] : {};
  if (!text(owner.id)) add('OWNER_ID_MISSING', 'The proposed owner needs an explicit ID.');
  if (!pathKey(owner.path) || !/\.rs$/i.test(owner.path ?? '') || owner.language !== 'rust') add('OWNER_NOT_RUST', 'Owner must name a Rust source file.');
  if (owner.authority !== 'candidate_proposal') add('OWNER_AUTHORITY_PROMOTION', 'Owner declaration remains a candidate proposal.');
  for (const key of ['epoch', 'generation', 'tick']) {
    if (!counter(owner[key])) add(`${key.toUpperCase()}_BINDING_MISSING`, `Owner ${key} must be an explicit nonnegative safe integer.`);
  }

  const exactInputs = Array.isArray(context.exactReadonlyInputs) ? context.exactReadonlyInputs : [];
  if (exactInputs.length === 0 || exactInputs.some(path => !pathKey(path))) add('READONLY_REGISTRY_MISSING', 'Caller must supply actual exact readonly inputs.');
  const allowedReads = new Set(exactInputs.map(path => pathKey(path)).filter(Boolean));
  const reads = Array.isArray(model.readonlyInputs) ? model.readonlyInputs : [];
  if (reads.length === 0) add('READONLY_INPUTS_MISSING', 'Actual read inputs must be explicitly listed.');
  const declaredReads = new Set();
  for (const path of reads) {
    const key = pathKey(path);
    if (!key || !allowedReads.has(key)) add('READ_OUT_OF_SCOPE', 'Read path is absent from caller exact inputs.', { path });
    if (key) declaredReads.add(key);
    if (active(path)) add('ACTIVE_EVIDENCE_FORBIDDEN', 'Active package output cannot be consumed.', { path });
    if (mediaPath(path)) add('MEDIA_ACCESS_FORBIDDEN', 'Media or protected asset/data paths cannot be consumed.', { path });
  }
  if (pathKey(owner.path) && !declaredReads.has(pathKey(owner.path))) add('OWNER_SOURCE_NOT_READ', 'The proposed owner source must be among the actual readonly inputs.');
  const writes = Array.isArray(model.writes) ? model.writes : [];
  if (writes.length === 0) add('WRITE_PLAN_MISSING', 'The candidate package write scope must be explicit.');
  for (const path of writes) {
    if (!insidePackage(path)) add('WRITE_OUT_OF_SCOPE', 'Writes are restricted to the three TP-15C1 paths.', { path });
    if (active(path)) add('ACTIVE_WRITE_FORBIDDEN', 'Active package paths cannot be written.', { path });
    if (mediaPath(path)) add('MEDIA_ACCESS_FORBIDDEN', 'Media and protected asset/data paths cannot be written.', { path });
  }

  const tests = Array.isArray(model.tests) ? model.tests : [];
  const testMap = new Map();
  if (tests.length === 0) add('TESTS_MISSING', 'Candidate contract requires a nonempty test ledger.');
  for (const test of tests) {
    if (!record(test) || !text(test.id)) { add('TEST_INVALID', 'Test rows need explicit IDs.'); continue; }
    if (testMap.has(test.id)) add('TEST_DUPLICATE', 'Test IDs must be unique.', { testId: test.id });
    testMap.set(test.id, test);
    if (test.pass !== true) add('TEST_FAILED', 'Candidate test did not pass.', { testId: test.id });
  }
  if (!text(model.rollback)) add('ROLLBACK_MISSING', 'The candidate contract needs an explicit rollback plan.');
  if (!hash(model.contractHash)) add('CONTRACT_HASH_MISSING', 'Contract SHA-256 must be explicit.');

  const registry = record(context.trustedEvidence) ? context.trustedEvidence : {};
  if (Object.keys(registry).length === 0) add('PROVENANCE_REGISTRY_MISSING', 'Independent evidence registry is absent.');
  const provenance = Array.isArray(model.provenance) ? model.provenance : [];
  if (provenance.length === 0) add('PROVENANCE_MISSING', 'At least one independent loot_scan evidence reference is required.');
  const seenEvidence = new Set();
  const evidenceResults = [];
  for (const reference of provenance) {
    const start = blockers.length;
    if (!record(reference) || !text(reference.evidenceId)) {
      add('PROVENANCE_INVALID', 'Evidence references need explicit IDs.');
      continue;
    }
    const extra = { evidenceId: reference.evidenceId };
    if (seenEvidence.has(reference.evidenceId)) add('PROVENANCE_DUPLICATE', 'Duplicate evidence cannot inflate coverage.', extra);
    seenEvidence.add(reference.evidenceId);
    if (active(reference.evidenceId) || active(reference.path)) add('ACTIVE_EVIDENCE_FORBIDDEN', 'AG-38, TP-01R9 and TP-18C1 evidence cannot be consumed.', extra);
    if (forbiddenSource(reference.evidenceId)) add('EVIDENCE_DOMAIN_MISMATCH', 'HUD/map, routing and legacy Q/R evidence cannot supply loot_scan provenance.', extra);
    const evidence = Object.hasOwn(registry, reference.evidenceId) ? registry[reference.evidenceId] : null;
    if (!record(evidence)) {
      add('PROVENANCE_MISSING', 'Referenced evidence is absent from the independent registry.', extra);
    } else {
      if (active(evidence.path) || active(evidence.id)) add('ACTIVE_EVIDENCE_FORBIDDEN', 'Active package evidence is forbidden.', extra);
      if (evidence.domain !== 'loot_scan' || forbiddenSource(evidence.sourceKind)
          || !['candidate_contract', 'loot_scan_contract'].includes(evidence.sourceKind)) {
        add('EVIDENCE_DOMAIN_MISMATCH', 'Evidence must explicitly concern the loot_scan candidate contract.', extra);
      }
      const expectedPath = pathKey(evidence.path);
      if (!expectedPath || pathKey(reference.path) !== expectedPath) add('EVIDENCE_PATH_MISMATCH', 'Evidence path does not match independent provenance.', extra);
      if (!expectedPath || !allowedReads.has(expectedPath) || !declaredReads.has(expectedPath)) add('EVIDENCE_READ_OUT_OF_SCOPE', 'Evidence must be among actual exact readonly inputs.', extra);
      if (mediaPath(evidence.path)) add('MEDIA_ACCESS_FORBIDDEN', 'Media evidence cannot be consumed.', extra);
      if (!hash(reference.sha256) || !hash(evidence.sha256) || reference.sha256.toLowerCase() !== evidence.sha256.toLowerCase()) {
        add('EVIDENCE_HASH_MISMATCH', 'Evidence hash is missing or mismatched.', extra);
      }
      if (!hash(evidence.contractHash) || !hash(model.contractHash) || evidence.contractHash.toLowerCase() !== model.contractHash.toLowerCase()) {
        add('CONTRACT_HASH_MISMATCH', 'Independent evidence must bind this exact contract hash.', extra);
      }
      if (!text(evidence.ownerId) || evidence.ownerId !== owner.id) add('OWNER_PROVENANCE_MISMATCH', 'Owner ID is not bound by independent evidence.', extra);
      if (!pathKey(evidence.ownerPath) || pathKey(evidence.ownerPath) !== pathKey(owner.path)) add('OWNER_PATH_BINDING_MISSING', 'Independent evidence must bind the proposed Rust owner path.', extra);
      for (const key of ['epoch', 'generation', 'tick']) {
        if (!counter(evidence[key])) add(`${key.toUpperCase()}_BINDING_MISSING`, `Independent evidence must bind ${key}.`, extra);
        else if (evidence[key] !== owner[key]) add(key === 'tick' ? 'STALE_TICK_ACCEPTED' : `${key.toUpperCase()}_DRIFT`, `Owner ${key} differs from independent evidence.`, extra);
      }
      if (!Array.isArray(evidence.tests) || evidence.tests.length === 0 || evidence.tests.some(id => !text(id))) {
        add('EVIDENCE_TESTS_MISSING', 'Independent evidence must identify supporting tests.', extra);
      } else {
        for (const testId of evidence.tests) {
          if (testMap.get(testId)?.pass !== true) add('REQUIRED_TEST_NOT_PASSED', 'A provenance-bound test is absent or failed.', { ...extra, testId });
        }
      }
    }
    const errors = blockers.slice(start);
    evidenceResults.push({ evidenceId: reference.evidenceId, path: reference.path ?? null, pass: errors.length === 0, errorCodes: [...new Set(errors.map(row => row.code))] });
  }

  const claims = record(model.controls) ? model.controls : {};
  const verified = record(context.verifiedControls) ? context.verifiedControls : {};
  const controlResults = Object.entries(REQUIRED_CONTROLS).map(([control, code]) => {
    const codes = [];
    if (claims[control] !== true) { add(code, `Candidate control ${control} is missing.`, { control }); codes.push(code); }
    if (!Object.hasOwn(verified, control) || verified[control] !== true) {
      const error = !Object.hasOwn(verified, control) ? 'CONTROL_EVIDENCE_MISSING' : 'CONTROL_EVIDENCE_NOT_VERIFIED';
      add(error, `Candidate control ${control} lacks independent verification.`, { control }); codes.push(error);
    }
    return { control, pass: codes.length === 0, errorCodes: codes };
  });
  const pass = blockers.length === 0;
  return {
    packageId: PACKAGE_ID,
    decision: pass ? 'contract_ready_candidate_only' : 'blocked_missing_contract',
    pass, ok: pass,
    architecture: 'blocked/no_go', formalTakeover: false,
    capabilityIdStatus: 'suggestion_only', runtimeScanImplemented: false,
    writesArePackageProposalOnly: true, releaseAuthorizedByThisReview: false,
    fixtureOnly: context.fixtureOnly === true, readinessEvidence: context.fixtureOnly !== true,
    blockers, errorCodes: [...new Set(blockers.map(row => row.code))], evidenceResults, controlResults,
  };
}

function syntheticControl() {
  const evidenceId = 'fixture-loot-scan-contract';
  const evidencePath = 'fixture-only/loot-scan-contract.json';
  const ownerPath = 'server-rs/src/fixture_loot_scan.rs';
  const ownerId = 'fixture-only-rust-owner';
  const sha256 = 'a'.repeat(64);
  const contractHash = 'b'.repeat(64);
  const testId = 'fixture-only-loot-scan-test';
  const verifiedControls = Object.fromEntries(Object.keys(REQUIRED_CONTROLS).map(key => [key, true]));
  const context = {
    fixtureOnly: true,
    exactReadonlyInputs: [evidencePath, ownerPath],
    verifiedControls,
    trustedEvidence: {
      [evidenceId]: {
        path: evidencePath, sha256, domain: 'loot_scan', sourceKind: 'candidate_contract',
        contractHash, ownerId, ownerPath, epoch: 3, generation: 5, tick: 37, tests: [testId],
      },
    },
  };
  const model = {
    packageId: PACKAGE_ID, capabilityId: 'perception.loot_scan', capabilityIdStatus: 'suggestion_only',
    scope: 'candidate_contract', architecture: 'blocked/no_go', formalTakeover: false,
    fields: [...REQUIRED_FIELDS],
    owners: [{ id: ownerId, path: ownerPath, language: 'rust', authority: 'candidate_proposal', epoch: 3, generation: 5, tick: 37 }],
    contractHash, provenance: [{ evidenceId, path: evidencePath, sha256 }],
    controls: { ...verifiedControls }, writes: [...PACKAGE_WRITES],
    readonlyInputs: [evidencePath, ownerPath], media: false, network: false,
    tests: [{ id: testId, pass: true }],
    rollback: 'Fixture-only: discard the candidate contract files; no runtime or formal owner was changed.',
  };
  return { model, context };
}

/** Each fixture mutates exactly one property of a complete synthetic valid input. */
export function runNegativeFixtures() {
  const base = syntheticControl();
  const positive = evaluateContract(base.model, base.context);
  const fixtures = [];
  const add = (id, expectedCode, mutate) => fixtures.push({ id, expectedCode, mutate });
  const evidence = context => context.trustedEvidence['fixture-loot-scan-contract'];
  add('hud-map-domain-substitution', 'EVIDENCE_DOMAIN_MISMATCH', ({ context }) => { evidence(context).domain = 'hud_map'; });
  add('routing-domain-substitution', 'EVIDENCE_DOMAIN_MISMATCH', ({ context }) => { evidence(context).domain = 'routing'; });
  add('legacy-qr-source-substitution', 'EVIDENCE_DOMAIN_MISMATCH', ({ context }) => { evidence(context).sourceKind = 'legacy_QR'; });
  add('missing-owner', 'SINGLE_RUST_OWNER_MISSING', ({ model }) => { model.owners = []; });
  add('duplicate-owner', 'SINGLE_RUST_OWNER_MISSING', ({ model }) => { model.owners = [...model.owners, { ...model.owners[0] }]; });
  add('javascript-owner', 'OWNER_NOT_RUST', ({ model }) => { model.owners[0].language = 'javascript'; });
  add('forged-owner', 'OWNER_PROVENANCE_MISMATCH', ({ model }) => { model.owners[0].id = 'forged'; });
  add('missing-fields', 'FIELDS_MISSING', ({ model }) => { model.fields = []; });
  add('missing-result-field', 'REQUIRED_FIELD_MISSING', ({ model }) => { model.fields = model.fields.filter(field => field !== 'results'); });
  add('epoch-drift', 'EPOCH_DRIFT', ({ model }) => { model.owners[0].epoch += 1; });
  add('generation-drift', 'GENERATION_DRIFT', ({ model }) => { model.owners[0].generation += 1; });
  add('stale-tick', 'STALE_TICK_ACCEPTED', ({ model }) => { model.owners[0].tick -= 1; });
  for (const [control, code] of Object.entries(REQUIRED_CONTROLS)) {
    add(`control-${control}`, code, ({ model }) => { model.controls[control] = false; });
  }
  for (const key of ['epoch', 'generation', 'tick']) {
    add(`missing-independent-${key}`, `${key.toUpperCase()}_BINDING_MISSING`, ({ context }) => { delete evidence(context)[key]; });
  }
  add('missing-independent-owner-path', 'OWNER_PATH_BINDING_MISSING', ({ context }) => { delete evidence(context).ownerPath; });
  add('candidate-to-formal-promotion', 'CANDIDATE_FORMAL_PROMOTION', ({ model }) => { model.formalTakeover = true; });
  add('owner-authority-promotion', 'OWNER_AUTHORITY_PROMOTION', ({ model }) => { model.owners[0].authority = 'formal'; });
  add('capability-id-promotion', 'CAPABILITY_ID_PROMOTION', ({ model }) => { model.capabilityIdStatus = 'approved'; });
  add('overall-gate-escalation', 'ARCHITECTURE_GATE_ESCALATION', ({ model }) => { model.architecture = 'go'; });
  add('scope-escalation', 'CANDIDATE_SCOPE_ESCALATION', ({ model }) => { model.scope = 'formal_contract'; });
  for (const id of ['AG-38', 'TP-01R9', 'TP-18C1']) {
    add(`active-evidence-${id}`, 'ACTIVE_EVIDENCE_FORBIDDEN', ({ model }) => { model.provenance[0].evidenceId = id; });
  }
  add('write-outside-package', 'WRITE_OUT_OF_SCOPE', ({ model }) => { model.writes[0] = 'server-rs/src/world_v3.rs'; });
  add('write-path-traversal', 'WRITE_OUT_OF_SCOPE', ({ model }) => { model.writes[0] = 'tools/continuous-tps-architecture-probe/loot-scan-contract-readiness/../other/validate.mjs'; });
  add('missing-write-plan', 'WRITE_PLAN_MISSING', ({ model }) => { model.writes = []; });
  add('read-outside-actual-inputs', 'READ_OUT_OF_SCOPE', ({ model }) => { model.readonlyInputs = [...model.readonlyInputs, 'docs/not-actually-read.md']; });
  add('missing-exact-readonly-inputs', 'READONLY_REGISTRY_MISSING', ({ context }) => { delete context.exactReadonlyInputs; });
  add('missing-actual-read-inputs', 'READONLY_INPUTS_MISSING', ({ model }) => { model.readonlyInputs = []; });
  add('media-enabled', 'MEDIA_ACCESS_FORBIDDEN', ({ model }) => { model.media = true; });
  add('media-input-path', 'MEDIA_ACCESS_FORBIDDEN', ({ model }) => { model.readonlyInputs = [...model.readonlyInputs, 'media/probe.png']; });
  add('network-enabled', 'NETWORK_ACCESS_FORBIDDEN', ({ model }) => { model.network = true; });
  add('media-policy-missing', 'MEDIA_ACCESS_FORBIDDEN', ({ model }) => { delete model.media; });
  add('network-policy-missing', 'NETWORK_ACCESS_FORBIDDEN', ({ model }) => { delete model.network; });
  add('tests-missing', 'TESTS_MISSING', ({ model }) => { model.tests = []; });
  add('test-failed', 'TEST_FAILED', ({ model }) => { model.tests[0].pass = false; });
  add('rollback-missing', 'ROLLBACK_MISSING', ({ model }) => { delete model.rollback; });
  add('provenance-missing', 'PROVENANCE_MISSING', ({ model }) => { model.provenance = []; });
  add('registry-missing', 'PROVENANCE_REGISTRY_MISSING', ({ context }) => { delete context.trustedEvidence; });
  add('evidence-hash-drift', 'EVIDENCE_HASH_MISMATCH', ({ model }) => { model.provenance[0].sha256 = 'c'.repeat(64); });
  add('contract-hash-drift', 'CONTRACT_HASH_MISMATCH', ({ model }) => { model.contractHash = 'c'.repeat(64); });
  add('independent-contract-hash-missing', 'CONTRACT_HASH_MISMATCH', ({ context }) => { delete evidence(context).contractHash; });
  add('evidence-tests-empty', 'EVIDENCE_TESTS_MISSING', ({ context }) => { evidence(context).tests = []; });
  add('evidence-test-not-run', 'REQUIRED_TEST_NOT_PASSED', ({ context }) => { evidence(context).tests = ['unrun-test']; });
  add('verified-controls-missing', 'CONTROL_EVIDENCE_MISSING', ({ context }) => { delete context.verifiedControls; });
  add('verified-control-missing', 'CONTROL_EVIDENCE_MISSING', ({ context }) => { delete context.verifiedControls.rangeEnforced; });
  add('verified-control-failed', 'CONTROL_EVIDENCE_NOT_VERIFIED', ({ context }) => { context.verifiedControls.rangeEnforced = false; });
  add('truthy-claim-is-not-proof', 'RANGE_NOT_ENFORCED', ({ model }) => { model.controls.rangeEnforced = 'true'; });
  add('truthy-verification-is-not-proof', 'CONTROL_EVIDENCE_NOT_VERIFIED', ({ context }) => { context.verifiedControls.rangeEnforced = 'true'; });
  const results = fixtures.map(({ id, expectedCode, mutate }) => {
    const input = structuredClone(base);
    mutate(input);
    const result = evaluateContract(input.model, input.context);
    const pass = result.decision === 'blocked_missing_contract' && result.errorCodes.includes(expectedCode)
      && result.architecture === 'blocked/no_go' && result.formalTakeover === false
      && result.fixtureOnly === true && result.readinessEvidence === false;
    return { id, expectedCode, pass, decision: result.decision, errorCodes: result.errorCodes, fixtureOnly: result.fixtureOnly };
  });
  const positivePass = positive.decision === 'contract_ready_candidate_only' && positive.blockers.length === 0
    && positive.architecture === 'blocked/no_go' && positive.formalTakeover === false
    && positive.fixtureOnly === true && positive.readinessEvidence === false;
  return {
    fixtureOnly: true, readinessEvidence: false,
    positiveControl: { fixtureOnly: true, readinessEvidence: false, pass: positivePass, decision: positive.decision, errorCodes: positive.errorCodes },
    total: results.length, passed: results.filter(row => row.pass).length, failed: results.filter(row => !row.pass).length,
    pass: positivePass && results.length >= 26 && results.every(row => row.pass), results,
  };
}
