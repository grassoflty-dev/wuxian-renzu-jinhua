/**
 * TP-01R7 exact-package prerequisite evaluator.
 * Pure data evaluation: this module never reads files, runs tools, or applies writes.
 * A successful evaluation only qualifies the proposed package for conditional review.
 */
export const PACKAGE_ID = 'PKG-FORMAL-CAPABILITY-HUD';
export const REQUIRED_PREREQUISITES = Object.freeze([
  'ADR-H', 'TP-13', 'TP-15', 'TP-16',
  'PKG-FORMAL-IPC-OWNER', 'PKG-FORMAL-ASSET-RELEASE',
]);
export const EXACT_FORMAL_WRITES = Object.freeze([
  'server-rs/src/world_v3.rs',
  'server-rs/src/campaign_main.rs',
  'server-rs/release-ui/js/capability-resolver.js',
  'server-rs/ui/index.html',
  'server-rs/release-ui/index.html',
  'server-rs/release-ui/js/client.js',
  'server-rs/release-ui/js/renderer-boot.js',
  'server-rs/release-ui/css/style.css',
]);
export const REQUIRED_CONTROLS = Object.freeze({
  singleOwner: 'SINGLE_OWNER_NOT_PROVEN',
  epochStable: 'EPOCH_DRIFT',
  generationStable: 'GENERATION_DRIFT',
  staleCallbacksRejected: 'STALE_CALLBACK_ACCEPTED',
  grantIdempotent: 'DUPLICATE_GRANT',
  revokeIdempotent: 'DUPLICATE_REVOKE',
  queueBounded: 'QUEUE_UNBOUNDED',
  queueOverflowRejected: 'QUEUE_OVERFLOW_ACCEPTED',
  cooldownAuthoritative: 'COOLDOWN_BYPASS',
  accessibilityPresent: 'ACCESSIBILITY_MISSING',
  accessibilityIndependent: 'ACCESSIBILITY_CAPABILITY_GATED',
  hudFieldsComplete: 'HUD_FIELDS_MISSING',
  disposeBaseline: 'DISPOSE_LEAK',
  closeNoCallbacks: 'CLOSE_CALLBACK_LEAK',
  realReleaseLoad: 'REAL_RELEASE_LOAD_MISSING',
  releaseAuthorized: 'RELEASE_AUTHORIZATION_MISSING',
  productionWired: 'FORMAL_WIRING_MISSING',
});

const isRecord = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const nonempty = value => typeof value === 'string' && value.trim().length > 0;
const validCounter = value => Number.isSafeInteger(value) && value >= 0;
const validHash = value => typeof value === 'string' && /^[a-f0-9]{64}$/i.test(value);

function normalizedPath(value, allowGlob = false) {
  if (!nonempty(value) || value !== value.trim()) return null;
  const path = value.replaceAll('\\', '/');
  if (path.startsWith('/') || path.includes(':') || /[\x00-\x1f\x7f]/.test(path)) return null;
  const parts = path.split('/');
  if (parts.some(part => !part || part === '.' || part === '..' || /[. ]$/.test(part))) return null;
  if (!allowGlob && /[*?]/.test(path)) return null;
  return path.toLowerCase();
}

function globMatches(path, pattern) {
  // '**' spans directories; '*' and '?' are confined to one path segment.
  const source = pattern.split(/(\*\*|\*|\?)/).map(part => {
    if (part === '**') return '.*';
    if (part === '*') return '[^/]*';
    if (part === '?') return '[^/]';
    return part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }).join('');
  return new RegExp(`^${source}$`, 'u').test(path);
}

/**
 * @param {object} model Package proposal, including claimed controls and prerequisites.
 * Each prerequisite must include evidencePath/evidenceSha256 from its cited artifact.
 * @param {object} context Independent validator-generated evidence registry. Do not
 * construct this registry from the proposal's claims. Required ownerPath,
 * expectedEpoch, and expectedGeneration bind each owner to independently measured
 * identity and counters. prerequisiteIds binds evidence to the reviewed prerequisite.
 * verifiedControls binds claimed controls to verified results.
 * @returns {object} Stable decision, error codes, and per-prerequisite details.
 */
export function evaluateGate(model, context) {
  const blockers = [];
  const add = (code, detail, extra = {}) => blockers.push({ code, detail, ...extra });
  if (!isRecord(model)) add('MODEL_INVALID', 'Package model must be an object.');
  if (!isRecord(context)) add('CONTEXT_INVALID', 'Independent evidence context is required.');
  model = isRecord(model) ? model : {};
  context = isRecord(context) ? context : {};

  if (model.packageId !== PACKAGE_ID) add('PACKAGE_ID_MISMATCH', `Only ${PACKAGE_ID} may be evaluated.`);
  if (model.scope !== 'exact_package') add('PACKAGE_SCOPE_ESCALATION', 'Scope must remain exact_package.');
  if (model.architecture !== 'blocked/no_go') add('ARCHITECTURE_GATE_ESCALATION', 'Architecture Gate must remain blocked/no_go.');
  if (model.formalTakeover !== false) add('FORMAL_TAKEOVER_ESCALATION', 'This review cannot authorize formal takeover.');

  const canonicalWrites = new Set(EXACT_FORMAL_WRITES.map(path => normalizedPath(path)));
  const suppliedWrites = Array.isArray(context.exactFormalWrites)
    ? context.exactFormalWrites.map(path => normalizedPath(path)) : [];
  if (suppliedWrites.length !== canonicalWrites.size || new Set(suppliedWrites).size !== canonicalWrites.size
      || suppliedWrites.some(path => !canonicalWrites.has(path))) {
    add('CONTEXT_WHITELIST_MISMATCH', 'Trusted whitelist must exactly match the original TP-18B1 package.');
  }
  const patterns = Array.isArray(context.activeWrites)
    ? context.activeWrites.map(path => normalizedPath(path, true)) : [];
  if (!Array.isArray(context.activeWrites) || patterns.some(path => path === null)) {
    add('CONTEXT_ACTIVE_WRITES_INVALID', 'An explicit valid active-write conflict registry is required.');
  }
  const writes = Array.isArray(model.writes) ? model.writes : [];
  if (writes.length === 0) add('WRITE_PLAN_MISSING', 'The future package write proposal must be explicit.');
  const seenWrites = new Set();
  for (const rawPath of writes) {
    const path = normalizedPath(rawPath);
    if (!path || !canonicalWrites.has(path)) add('WRITE_OUT_OF_SCOPE', 'Proposed write is outside the exact package.', { path: rawPath });
    if (path && seenWrites.has(path)) add('WRITE_DUPLICATE', 'Proposed write is duplicated.', { path: rawPath });
    if (path) seenWrites.add(path);
    if (path && patterns.some(pattern => pattern && globMatches(path, pattern))) {
      add('WRITE_SET_CONFLICT', 'Proposed write conflicts with an active package.', { path: rawPath });
    }
  }

  const registry = isRecord(context.trustedEvidence) ? context.trustedEvidence : {};
  if (!isRecord(context.trustedEvidence) || Object.keys(registry).length === 0) {
    add('PROVENANCE_REGISTRY_MISSING', 'Independently checked evidence registry is missing.');
  }
  const tests = Array.isArray(model.tests) ? model.tests : [];
  if (tests.length === 0) add('TESTS_MISSING', 'A nonempty test ledger is required.');
  const testsById = new Map();
  for (const test of tests) {
    if (!isRecord(test) || !nonempty(test.id)) {
      add('TEST_INVALID', 'Test rows must have an explicit identifier.');
      continue;
    }
    if (testsById.has(test.id)) add('TEST_DUPLICATE', 'Test identifiers must be unique.', { testId: test.id });
    testsById.set(test.id, test);
    if (test.pass !== true) add('TEST_FAILED', 'Referenced test did not pass.', { testId: test.id });
  }
  if (!nonempty(model.rollback)) add('ROLLBACK_MISSING', 'A concrete rollback description is required.');

  const owners = Array.isArray(model.owners) ? model.owners : [];
  if (owners.length === 0) add('FORMAL_OWNER_MISSING', 'At least one identified formal owner is required.');
  const ownersById = new Map();
  const ownerPaths = new Set();
  const registryOwners = new Set(Object.values(registry).filter(isRecord).map(row => row.ownerId));
  for (const owner of owners) {
    if (!isRecord(owner) || !nonempty(owner.id)) {
      add('OWNER_INVALID', 'Owner rows require an explicit identifier.');
      continue;
    }
    if (ownersById.has(owner.id)) add('OWNER_DUPLICATE', 'An owner identifier is declared more than once.', { ownerId: owner.id });
    ownersById.set(owner.id, owner);
    const path = normalizedPath(owner.path);
    if (!path || !canonicalWrites.has(path)) add('OWNER_PATH_INVALID', 'Formal owner must name an original package file.', { ownerId: owner.id, path: owner.path });
    if (path && ownerPaths.has(path)) add('OWNER_DUPLICATE', 'Multiple owner declarations claim the same formal file.', { ownerId: owner.id, path: owner.path });
    if (path) ownerPaths.add(path);
    if (owner.authority !== 'formal') add('OWNER_NOT_FORMAL', 'Owner authority is not formal.', { ownerId: owner.id });
    if (!registryOwners.has(owner.id)) add('OWNER_UNTRUSTED', 'Owner is absent from independent provenance.', { ownerId: owner.id });
    if (!validCounter(owner.epoch)) add('OWNER_EPOCH_INVALID', 'Owner epoch must be a nonnegative safe integer.', { ownerId: owner.id });
    if (!validCounter(owner.generation)) add('OWNER_GENERATION_INVALID', 'Owner generation must be a nonnegative safe integer.', { ownerId: owner.id });
  }

  const rows = Array.isArray(model.prerequisites) ? model.prerequisites : [];
  const seenPrerequisites = new Set();
  const prerequisiteResults = [];
  for (const row of rows) {
    if (!isRecord(row) || !nonempty(row.id)) {
      add('PREREQUISITE_INVALID', 'Prerequisite rows require an explicit identifier.');
      continue;
    }
    const start = blockers.length;
    const extra = { prerequisiteId: row.id };
    if (!REQUIRED_PREREQUISITES.includes(row.id)) add('PREREQUISITE_OUT_OF_SCOPE', 'Unknown prerequisite cannot expand this package.', extra);
    if (seenPrerequisites.has(row.id)) add('PREREQUISITE_DUPLICATE', 'Prerequisite appears more than once.', extra);
    seenPrerequisites.add(row.id);
    if (row.status !== 'formal_pass') add('PREREQUISITE_NOT_FORMAL', 'Prerequisite requires formal_pass evidence.', extra);
    const evidence = nonempty(row.evidenceId) && Object.hasOwn(registry, row.evidenceId) ? registry[row.evidenceId] : null;
    const owner = ownersById.get(row.ownerId);
    if (!owner) add('FORMAL_OWNER_MISSING', 'Prerequisite has no declared formal owner.', extra);
    if (!isRecord(evidence)) {
      add('PROVENANCE_MISSING', 'Cited evidence has no independently checked provenance.', extra);
    } else {
      if (evidence.level !== 'formal') add('CANDIDATE_PROMOTION_FORBIDDEN', 'Candidate or isolated evidence cannot satisfy a formal prerequisite.', extra);
      if (!Array.isArray(evidence.prerequisiteIds) || evidence.prerequisiteIds.length === 0
          || evidence.prerequisiteIds.some(id => !REQUIRED_PREREQUISITES.includes(id))
          || !evidence.prerequisiteIds.includes(row.id)) {
        add('EVIDENCE_PREREQUISITE_MISMATCH', 'Independent evidence must explicitly bind this exact prerequisite.', extra);
      }
      const evidencePath = normalizedPath(evidence.path);
      const rowPath = normalizedPath(row.evidencePath);
      if (!evidencePath || !rowPath || rowPath !== evidencePath) add('EVIDENCE_PATH_MISMATCH', 'Cited evidence path does not match independent provenance.', extra);
      if (!validHash(evidence.sha256) || !validHash(row.evidenceSha256)
          || evidence.sha256.toLowerCase() !== row.evidenceSha256.toLowerCase()) {
        add('EVIDENCE_HASH_MISMATCH', 'Evidence SHA-256 is missing or does not match independent provenance.', extra);
      }
      if (!nonempty(evidence.ownerId) || evidence.ownerId !== row.ownerId) add('OWNER_PROVENANCE_MISMATCH', 'Owner does not match independently checked evidence.', extra);
      if (!normalizedPath(evidence.ownerPath)) {
        add('OWNER_PATH_BINDING_MISSING', 'Independent evidence must bind a valid formal owner path.', extra);
      } else if (!owner || normalizedPath(evidence.ownerPath) !== normalizedPath(owner.path)) {
        add('OWNER_PATH_MISMATCH', 'Owner path differs from the independent evidence binding.', extra);
      }
      if (!validCounter(evidence.expectedEpoch)) {
        add('EPOCH_BINDING_MISSING', 'Independent evidence must bind a valid measured owner epoch.', extra);
      } else if (owner?.epoch !== evidence.expectedEpoch) {
        add('EPOCH_DRIFT', 'Owner epoch differs from the evidence epoch.', extra);
      }
      if (!validCounter(evidence.expectedGeneration)) {
        add('GENERATION_BINDING_MISSING', 'Independent evidence must bind a valid measured owner generation.', extra);
      } else if (owner?.generation !== evidence.expectedGeneration) {
        add('GENERATION_DRIFT', 'Owner generation differs from the evidence generation.', extra);
      }
      if (!Array.isArray(evidence.tests) || evidence.tests.length === 0 || evidence.tests.some(id => !nonempty(id))) {
        add('EVIDENCE_TESTS_MISSING', 'Provenance must identify the tests supporting this prerequisite.', extra);
      } else {
        for (const id of evidence.tests) {
          if (testsById.get(id)?.pass !== true) add('REQUIRED_TEST_NOT_PASSED', 'A provenance-bound required test is missing or failed.', { ...extra, testId: id });
        }
      }
    }
    const rowBlockers = blockers.slice(start);
    prerequisiteResults.push({
      id: row.id,
      evidenceId: row.evidenceId ?? null,
      evidencePath: row.evidencePath ?? null,
      ownerId: row.ownerId ?? null,
      pass: rowBlockers.length === 0,
      status: rowBlockers.length === 0 ? 'formal_pass' : 'blocked',
      errorCodes: [...new Set(rowBlockers.map(blocker => blocker.code))],
    });
  }
  for (const id of REQUIRED_PREREQUISITES) {
    if (!seenPrerequisites.has(id)) {
      add('PREREQUISITE_MISSING', 'Required prerequisite is absent.', { prerequisiteId: id });
      prerequisiteResults.push({ id, evidenceId: null, evidencePath: null, ownerId: null, pass: false, status: 'blocked', errorCodes: ['PREREQUISITE_MISSING'] });
    }
  }
  const controls = isRecord(model.controls) ? model.controls : {};
  const verifiedControls = isRecord(context.verifiedControls) ? context.verifiedControls : {};
  const controlResults = Object.entries(REQUIRED_CONTROLS).map(([control, code]) => {
    const claimedPass = controls[control] === true;
    const verifiedPass = Object.hasOwn(verifiedControls, control) && verifiedControls[control] === true;
    const errorCodes = [];
    if (!claimedPass) {
      add(code, `Formal control ${control} has not been proven.`, { control });
      errorCodes.push(code);
    }
    if (!verifiedPass) {
      const evidenceCode = !Object.hasOwn(verifiedControls, control) || verifiedControls[control] === undefined
        ? 'CONTROL_EVIDENCE_MISSING' : 'CONTROL_EVIDENCE_NOT_VERIFIED';
      add(evidenceCode, `Formal control ${control} lacks an independent passing verification.`, { control });
      errorCodes.push(evidenceCode);
    }
    return { control, pass: claimedPass && verifiedPass, errorCode: errorCodes[0] ?? null, errorCodes };
  });
  const pass = blockers.length === 0;
  return {
    packageId: PACKAGE_ID,
    decision: pass ? 'conditional_go_exact_package' : 'blocked',
    pass,
    ok: pass,
    architecture: 'blocked/no_go',
    formalTakeover: false,
    writesAreProposalOnly: true,
    releaseAuthorizedByThisReview: false,
    fixtureOnly: context.fixtureOnly === true,
    readinessEvidence: context.fixtureOnly !== true,
    blockers,
    errorCodes: [...new Set(blockers.map(blocker => blocker.code))],
    prerequisiteResults,
    controlResults,
  };
}

function syntheticControl() {
  const context = {
    fixtureOnly: true,
    exactFormalWrites: [...EXACT_FORMAL_WRITES],
    activeWrites: ['tools/continuous-tps-architecture-probe/other-active/**'],
    verifiedControls: Object.fromEntries(Object.keys(REQUIRED_CONTROLS).map(key => [key, true])),
    trustedEvidence: {},
  };
  const ownerId = 'fixture-only-formal-owner';
  const model = {
    packageId: PACKAGE_ID,
    scope: 'exact_package',
    architecture: 'blocked/no_go',
    formalTakeover: false,
    writes: [...EXACT_FORMAL_WRITES],
    owners: [{ id: ownerId, path: EXACT_FORMAL_WRITES[0], authority: 'formal', epoch: 7, generation: 11 }],
    prerequisites: [],
    controls: Object.fromEntries(Object.keys(REQUIRED_CONTROLS).map(key => [key, true])),
    rollback: 'Fixture-only plan: restore the pre-package revision of the eight allowlisted files, revoke the owner, then rerun required tests.',
    tests: [],
  };
  REQUIRED_PREREQUISITES.forEach((id, index) => {
    const evidenceId = `fixture-only-evidence-${index}`;
    const path = `fixture-only/evidence-${index}.json`;
    const sha256 = (index + 1).toString(16).padStart(64, '0');
    const testId = `fixture-only-test-${index}`;
    context.trustedEvidence[evidenceId] = {
      path, sha256, level: 'formal', ownerId, ownerPath: EXACT_FORMAL_WRITES[0],
      expectedEpoch: 7, expectedGeneration: 11, prerequisiteIds: [id], tests: [testId],
    };
    model.prerequisites.push({ id, status: 'formal_pass', evidenceId, evidencePath: path, evidenceSha256: sha256, ownerId });
    model.tests.push({ id: testId, pass: true });
  });
  return { model, context };
}

/** Run actual one-property mutations of a complete synthetic positive model. */
export function runNegativeFixtures() {
  const base = syntheticControl();
  const positive = evaluateGate(base.model, base.context);
  const cases = [];
  const fixture = (id, expectedCode, mutate) => cases.push({ id, expectedCode, mutate });
  fixture('candidate-to-formal-promotion', 'CANDIDATE_PROMOTION_FORBIDDEN', ({ context }) => { context.trustedEvidence['fixture-only-evidence-0'].level = 'isolated_candidate'; });
  fixture('real-webview-candidate-promotion', 'CANDIDATE_PROMOTION_FORBIDDEN', ({ context }) => { context.trustedEvidence['fixture-only-evidence-0'].level = 'real_webview_candidate'; });
  fixture('fake-owner', 'OWNER_PROVENANCE_MISMATCH', ({ model }) => { model.prerequisites[0].ownerId = 'fabricated-owner'; });
  fixture('duplicate-owner', 'OWNER_DUPLICATE', ({ model }) => { model.owners = [...model.owners, { ...model.owners[0] }]; });
  fixture('nonformal-owner', 'OWNER_NOT_FORMAL', ({ model }) => { model.owners[0].authority = 'candidate'; });
  fixture('owner-epoch-drift', 'EPOCH_DRIFT', ({ model }) => { model.owners[0].epoch += 1; });
  fixture('owner-generation-drift', 'GENERATION_DRIFT', ({ model }) => { model.owners[0].generation += 1; });
  fixture('owner-path-forgery', 'OWNER_PATH_MISMATCH', ({ model }) => { model.owners[0].path = EXACT_FORMAL_WRITES[1]; });
  fixture('missing-owner-path-binding', 'OWNER_PATH_BINDING_MISSING', ({ context }) => { delete context.trustedEvidence['fixture-only-evidence-0'].ownerPath; });
  fixture('missing-epoch-binding', 'EPOCH_BINDING_MISSING', ({ context }) => { delete context.trustedEvidence['fixture-only-evidence-0'].expectedEpoch; });
  fixture('missing-generation-binding', 'GENERATION_BINDING_MISSING', ({ context }) => { delete context.trustedEvidence['fixture-only-evidence-0'].expectedGeneration; });
  fixture('wrong-prerequisite-binding', 'EVIDENCE_PREREQUISITE_MISMATCH', ({ context }) => { context.trustedEvidence['fixture-only-evidence-0'].prerequisiteIds = ['TP-13']; });
  fixture('missing-prerequisite-binding', 'EVIDENCE_PREREQUISITE_MISMATCH', ({ context }) => { delete context.trustedEvidence['fixture-only-evidence-0'].prerequisiteIds; });
  fixture('one-artifact-reused-for-six-prerequisites', 'EVIDENCE_PREREQUISITE_MISMATCH', ({ model }) => {
    const first = model.prerequisites[0];
    model.prerequisites = model.prerequisites.map(row => ({
      ...row, evidenceId: first.evidenceId,
      evidencePath: first.evidencePath, evidenceSha256: first.evidenceSha256,
    }));
  });
  for (const [control, code] of Object.entries(REQUIRED_CONTROLS)) {
    fixture(`control-${control}`, code, ({ model }) => { model.controls[control] = false; });
  }
  fixture('write-out-of-scope', 'WRITE_OUT_OF_SCOPE', ({ model }) => { model.writes[0] = 'server-rs/src/unapproved.rs'; });
  fixture('write-path-traversal', 'WRITE_OUT_OF_SCOPE', ({ model }) => { model.writes[0] = 'server-rs/src/../src/world_v3.rs'; });
  fixture('write-absolute-path', 'WRITE_OUT_OF_SCOPE', ({ model }) => { model.writes[0] = 'C:/server-rs/src/world_v3.rs'; });
  fixture('write-set-conflict', 'WRITE_SET_CONFLICT', ({ context }) => { context.activeWrites = ['server-rs/src/**']; });
  fixture('missing-active-write-registry', 'CONTEXT_ACTIVE_WRITES_INVALID', ({ context }) => { delete context.activeWrites; });
  fixture('whitelist-expansion', 'CONTEXT_WHITELIST_MISMATCH', ({ context }) => { context.exactFormalWrites = [...context.exactFormalWrites, 'server-rs/src/unapproved.rs']; });
  fixture('missing-write-plan', 'WRITE_PLAN_MISSING', ({ model }) => { model.writes = []; });
  fixture('duplicate-write', 'WRITE_DUPLICATE', ({ model }) => { model.writes = [...model.writes, model.writes[0]]; });
  fixture('missing-rollback', 'ROLLBACK_MISSING', ({ model }) => { delete model.rollback; });
  fixture('missing-tests', 'TESTS_MISSING', ({ model }) => { model.tests = []; });
  fixture('failed-test', 'TEST_FAILED', ({ model }) => { model.tests[0].pass = false; });
  fixture('missing-required-test', 'REQUIRED_TEST_NOT_PASSED', ({ model }) => { model.tests = model.tests.slice(1); });
  fixture('missing-provenance-tests', 'EVIDENCE_TESTS_MISSING', ({ context }) => { context.trustedEvidence['fixture-only-evidence-0'].tests = []; });
  fixture('missing-provenance', 'PROVENANCE_MISSING', ({ model }) => { delete model.prerequisites[0].evidenceId; });
  fixture('missing-provenance-registry', 'PROVENANCE_REGISTRY_MISSING', ({ context }) => { delete context.trustedEvidence; });
  fixture('hash-drift', 'EVIDENCE_HASH_MISMATCH', ({ model }) => { model.prerequisites[0].evidenceSha256 = 'f'.repeat(64); });
  fixture('missing-hash', 'EVIDENCE_HASH_MISMATCH', ({ model }) => { delete model.prerequisites[0].evidenceSha256; });
  fixture('evidence-path-drift', 'EVIDENCE_PATH_MISMATCH', ({ model }) => { model.prerequisites[0].evidencePath = 'fixture-only/forged.json'; });
  fixture('missing-evidence-path', 'EVIDENCE_PATH_MISMATCH', ({ model }) => { delete model.prerequisites[0].evidencePath; });
  fixture('missing-prerequisite', 'PREREQUISITE_MISSING', ({ model }) => { model.prerequisites = model.prerequisites.slice(1); });
  fixture('candidate-prerequisite-status', 'PREREQUISITE_NOT_FORMAL', ({ model }) => { model.prerequisites[0].status = 'candidate_pass'; });
  fixture('duplicate-prerequisite', 'PREREQUISITE_DUPLICATE', ({ model }) => { model.prerequisites = [...model.prerequisites, { ...model.prerequisites[0] }]; });
  fixture('unknown-prerequisite', 'PREREQUISITE_OUT_OF_SCOPE', ({ model }) => { model.prerequisites[0].id = 'OTHER-PACKAGE'; });
  fixture('missing-formal-owner', 'FORMAL_OWNER_MISSING', ({ model }) => { model.owners = []; });
  fixture('overall-gate-override', 'ARCHITECTURE_GATE_ESCALATION', ({ model }) => { model.architecture = 'go'; });
  fixture('formal-takeover-override', 'FORMAL_TAKEOVER_ESCALATION', ({ model }) => { model.formalTakeover = true; });
  fixture('scope-override', 'PACKAGE_SCOPE_ESCALATION', ({ model }) => { model.scope = 'whole_architecture'; });
  fixture('wrong-package', 'PACKAGE_ID_MISMATCH', ({ model }) => { model.packageId = 'PKG-FORMAL-ASSET-RELEASE'; });
  fixture('truthy-control-is-not-proof', 'REAL_RELEASE_LOAD_MISSING', ({ model }) => { model.controls.realReleaseLoad = 'true'; });
  fixture('missing-independent-control-registry', 'CONTROL_EVIDENCE_MISSING', ({ context }) => { delete context.verifiedControls; });
  fixture('missing-independent-control', 'CONTROL_EVIDENCE_MISSING', ({ context }) => { delete context.verifiedControls.realReleaseLoad; });
  fixture('failed-independent-control', 'CONTROL_EVIDENCE_NOT_VERIFIED', ({ context }) => { context.verifiedControls.realReleaseLoad = false; });
  fixture('truthy-independent-control-is-not-proof', 'CONTROL_EVIDENCE_NOT_VERIFIED', ({ context }) => { context.verifiedControls.realReleaseLoad = 'true'; });
  const results = cases.map(({ id, expectedCode, mutate }) => {
    const input = structuredClone(base);
    mutate(input);
    const result = evaluateGate(input.model, input.context);
    const pass = result.decision === 'blocked' && result.errorCodes.includes(expectedCode)
      && result.architecture === 'blocked/no_go' && result.formalTakeover === false
      && result.fixtureOnly === true && result.readinessEvidence === false;
    return { id, expectedCode, pass, decision: result.decision, errorCodes: result.errorCodes, fixtureOnly: result.fixtureOnly };
  });
  const positivePass = positive.decision === 'conditional_go_exact_package' && positive.blockers.length === 0
    && positive.architecture === 'blocked/no_go' && positive.formalTakeover === false
    && positive.fixtureOnly === true && positive.readinessEvidence === false;
  return {
    fixtureOnly: true,
    readinessEvidence: false,
    positiveControl: {
      fixtureOnly: true, readinessEvidence: false, pass: positivePass,
      decision: positive.decision, errorCodes: positive.errorCodes,
      architecture: positive.architecture, formalTakeover: positive.formalTakeover,
    },
    total: results.length,
    passed: results.filter(result => result.pass).length,
    failed: results.filter(result => !result.pass).length,
    pass: positivePass && results.length >= 26 && results.every(result => result.pass),
    results,
  };
}
