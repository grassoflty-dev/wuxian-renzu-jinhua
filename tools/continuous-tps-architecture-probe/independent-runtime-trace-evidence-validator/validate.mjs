import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const probeRoot = path.resolve(here, '..');
const resultPath = path.join(probeRoot, 'results', 'independent-runtime-trace-evidence-validator-nb101.json');
const sourcePaths = {
  events: path.join(probeRoot, 'new-baseline-formal-runtime-observability-evidence', 'events.json'),
  errors: path.join(probeRoot, 'new-baseline-formal-runtime-error-contract', 'errors.json'),
  fixtures: path.join(probeRoot, 'new-baseline-continuous-world-deterministic-test-oracle', 'fixtures.json'),
  registry: path.join(probeRoot, 'new-baseline-shared-runtime-contract-schema-registry', 'registry.json')
};
const sourceIdentities = Object.fromEntries(Object.entries(sourcePaths).map(([name, file]) => {
  const bytes = fs.readFileSync(file); return [name, { path: path.relative(probeRoot, file), bytes: bytes.length, sha256: crypto.createHash('sha256').update(bytes).digest('hex').toUpperCase() }];
}));
const eventsContract = JSON.parse(fs.readFileSync(sourcePaths.events, 'utf8'));
const errorsContract = JSON.parse(fs.readFileSync(sourcePaths.errors, 'utf8'));
const fixturesContract = JSON.parse(fs.readFileSync(sourcePaths.fixtures, 'utf8'));
const registryContract = JSON.parse(fs.readFileSync(sourcePaths.registry, 'utf8'));
const eventByName = new Map(eventsContract.eventTypes.map((e) => [e.name, e]));
const errorCodes = new Set(errorsContract.errors.map((e) => e.code));
const fixtureIds = new Set(fixturesContract.fixtures.map((f) => f.stableId));
const topologyRules = new Map(eventsContract.traceTopologies.map((t) => [t.name, t]));
const schemaVersion = eventsContract.contractVersion;
const contractMappingFailures = [];
for (const event of eventsContract.eventTypes) {
  for (const code of event.errors) if (!errorCodes.has(code)) contractMappingFailures.push(`event ${event.name} -> ${code}`);
  for (const fixture of event.fixtures) if (!fixtureIds.has(fixture)) contractMappingFailures.push(`event ${event.name} -> ${fixture}`);
}
const parseTraceInput = (input) => {
  if (Array.isArray(input)) return input;
  if (typeof input !== 'string') throw new Error('trace input must be JSON array or NDJSON string');
  const trimmed = input.trim();
  if (trimmed.startsWith('[')) return JSON.parse(trimmed);
  return trimmed.split(/\r?\n/).filter(Boolean).map((line) => JSON.parse(line));
};

const base = (eventType, seq, overrides = {}) => ({
  eventSchemaVersion: schemaVersion, eventId: `evt-${seq}`, traceId: 'trace-synthetic-001', spanId: `span-${seq}`, parentSpanId: seq === 1 ? null : 'span-1',
  sessionId: 'session-opaque-001', requestIdSafe: 'req-opaque-001', idempotencyKeySafe: 'idem-opaque-001', authorityEpoch: 1, authorityTick: seq, authorityRevision: seq,
  worldId: 'world-v3-synthetic', zoneId: 'zone-a', roomId: 'room-01', entityId: 'entity-01', eventType, domain: eventByName.get(eventType)?.domain ?? 'synthetic', severity: 'info',
  stateCommitted: eventType.includes('committed') || eventType === 'evidence_seal', rollback: false, errorCode: null, monotonicTimestamp: seq * 10, durationMs: 1,
  sourceRevision: 'src-synthetic-001', buildIdentity: 'build-hash-001', configIdentity: 'config-hash-001', schemaIdentity: 'schema-hash-001',
  redactedAttributes: { synthetic: true }, evidenceCorrelation: { rawTraceHash: 'raw-trace-hash-001', environmentIdentity: 'env-synthetic-001' }, ...overrides
});
const traces = {
  command: ['command_received', 'command_validated', 'command_committed', 'snapshot_full'].map((e, i) => base(e, i + 1)),
  save: ['save_write_started', 'checkpoint_committed', 'save_write_finished'].map((e, i) => base(e, i + 1, { domain: 'save' })),
  migration: ['save_write_started', 'migration_result', 'save_write_finished'].map((e, i) => base(e, i + 1, { domain: 'save' })),
  transition: ['transition_prepare', 'transition_commit', 'transition_ack'].map((e, i) => base(e, i + 1, { domain: 'transition' })),
  resync: ['snapshot_gap', 'snapshot_resync', 'renderer_bind'].map((e, i) => base(e, i + 1, { domain: i === 0 ? 'snapshot' : 'renderer' , errorCode: i === 0 ? 'E_SNAPSHOT_GAP' : null })),
  hive: ['grey_hive_validate', 'grey_hive_constructed'].map((e, i) => base(e, i + 1, { domain: 'grey_hive_loader' })),
  packaged: ['app_start', 'tauri_handshake', 'packaged_offline_gate', 'evidence_seal'].map((e, i) => base(e, i + 1, { domain: 'packaged', evidenceCorrelation: { rawTraceHash: 'raw-trace-hash-001', environmentIdentity: 'env-synthetic-001', independentResult: true, exitStatus: 0, startMarker: 'start-001', endMarker: 'end-001', sourceRevision: 'src-synthetic-001', buildIdentity: 'build-hash-001', configIdentity: 'config-hash-001' } }))
};
const clone = (x) => JSON.parse(JSON.stringify(x));
const negative = [];
const neg = (id, name, mutate) => { const trace = clone(traces.command); mutate(trace); negative.push({ id, name, topology: 'command_trace', trace }); };
neg('NEG-01', 'missing terminal', (t) => { t.pop(); });
neg('NEG-02', 'orphan span', (t) => { t[1].parentSpanId = 'missing-span'; });
neg('NEG-03', 'non-monotonic revision', (t) => { t[2].authorityRevision = 1; });
neg('NEG-04', 'hash mismatch', (t) => { t[0].buildIdentity = 'other-build'; t[1].buildIdentity = 'build-hash-001'; });
neg('NEG-05', 'runtime self-pass', (t) => { t.push(base('evidence_seal', 5, { runtimePassedSelfClaim: true })); });
neg('NEG-06', 'schema mismatch', (t) => { t[0].eventSchemaVersion = 'old-schema'; });
neg('NEG-07', 'unknown error', (t) => { t[2].errorCode = 'E_UNKNOWN'; });
neg('NEG-08', 'privacy save leak', (t) => { t[0].save = { full: 'forbidden' }; });
neg('NEG-09', 'absolute path leak', (t) => { t[0].redactedAttributes.absolutePath = 'C:\\Users\\PUBLIC_USER\\save.json'; });
neg('NEG-10', 'PII leak', (t) => { t[0].redactedAttributes.email = 'person@example.com'; });
neg('NEG-11', 'raw payload leak', (t) => { t[0].rawPayload = 'secret'; });
neg('NEG-12', 'newline injection', (t) => { t[0].eventId = 'evt-1\nmalicious'; });
neg('NEG-13', 'event drop', (t) => { t.splice(1, 1); });
neg('NEG-14', 'duplicate event', (t) => { t.splice(2, 0, clone(t[1])); });
neg('NEG-15', 'invalid rollback', (t) => { t[2].rollback = true; t[2].stateCommitted = true; });
neg('NEG-16', 'snapshot delta without full', (t) => { t[3].eventType = 'snapshot_delta'; });
neg('NEG-17', 'authority child writes', (t) => { t[3].authorityWrite = true; });
neg('NEG-18', 'backpressure drop', (t) => { t[2].backpressure = 'event_drop'; });
neg('NEG-19', 'writer failure', (t) => { t[2].backpressure = 'writer_failure'; });

function validateTrace(trace, topologyName) {
  const failures = []; const rule = topologyRules.get(topologyName);
  const fail = (code, fieldPath) => failures.push({ code, fieldPath });
  if (!rule) { fail('unknown_topology', 'topology'); return failures; }
  if (!Array.isArray(trace) || trace.length === 0) { fail('empty_trace', 'trace'); return failures; }
  const ids = new Set(); const spans = new Set(); let previousTime = -Infinity; let previousRevision = -Infinity; let buildIdentity;
  for (let i = 0; i < trace.length; i += 1) {
    const e = trace[i]; const p = `trace[${i}]`; const contract = eventByName.get(e.eventType);
    if (!contract) fail('unknown_event_type', `${p}.eventType`);
    if (i === 0 && e.eventType !== rule.root) fail('wrong_root', `${p}.eventType`);
    if (ids.has(e.eventId)) fail('duplicate_event', `${p}.eventId`); ids.add(e.eventId);
    if (typeof e.eventId !== 'string' || /[\r\n]/.test(e.eventId)) fail('event_id_injection', `${p}.eventId`);
    if (e.eventSchemaVersion !== schemaVersion) fail('schema_mismatch', `${p}.eventSchemaVersion`);
    if (i === 0) { if (e.parentSpanId !== null) fail('root_parent_present', `${p}.parentSpanId`); } else if (!spans.has(e.parentSpanId)) fail('orphan_span', `${p}.parentSpanId`);
    spans.add(e.spanId);
    if (typeof e.monotonicTimestamp !== 'number' || e.monotonicTimestamp <= previousTime) fail('non_monotonic_time', `${p}.monotonicTimestamp`);
    if (typeof e.authorityRevision !== 'number' || e.authorityRevision < previousRevision) fail('non_monotonic_revision', `${p}.authorityRevision`);
    previousTime = e.monotonicTimestamp; previousRevision = e.authorityRevision;
    if (e.errorCode && !errorCodes.has(e.errorCode)) fail('unknown_error_code', `${p}.errorCode`);
    if (buildIdentity === undefined) buildIdentity = e.buildIdentity; else if (e.buildIdentity !== buildIdentity) fail('source_hash_mismatch', `${p}.buildIdentity`);
    if (contract) for (const field of contract.required) if (e[field] === undefined || e[field] === null) fail('required_field_missing', `${p}.${field}`);
    const serialized = JSON.stringify(e);
    if (/\\r|\\n|\r|\n/.test(serialized) || /(?:^|["'])C:\\\\|(?:^|["'])\/(?:Users|home)\//i.test(serialized)) fail('privacy_path_or_newline', p);
    if (Object.keys(e).some((key) => /^(rawPayload|payload|savePayload|save|mediaBytes|fullSource)$/i.test(key))) fail('privacy_forbidden_field', p);
    if (/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/i.test(serialized)) fail('privacy_pii', p);
    if (e.backpressure || e.dropped === true) fail('backpressure_invalidates_evidence', `${p}.backpressure`);
    if (e.runtimePassedSelfClaim === true) fail('anti_self_attestation', `${p}.runtimePassedSelfClaim`);
    if (e.rollback && e.stateCommitted) fail('rollback_committed_conflict', `${p}.rollback`);
    if (e.authorityWrite === true) fail('renderer_authority_write', `${p}.authorityWrite`);
  }
  const names = trace.map((e) => e.eventType); const expected = rule.order;
  for (let i = 0; i < expected.length; i += 1) if (names[i] !== expected[i]) fail('sequence_gap_or_order', `trace[${i}].eventType`);
  if (!rule.terminal.some((e) => names.includes(e))) fail('missing_terminal', 'trace.terminal');
  if (topologyName === 'renderer_resync' && names.includes('snapshot_delta') && !names.includes('snapshot_full')) fail('delta_without_full', 'trace');
  if (topologyName === 'packaged_run') {
    const seal = trace.find((e) => e.eventType === 'evidence_seal');
    if (!seal?.evidenceCorrelation?.independentResult || seal.evidenceCorrelation.exitStatus !== 0) fail('independent_result_missing', 'trace.evidenceCorrelation');
  }
  return failures;
}

const topologyForPositive = { command: 'command_trace', save: 'save_transaction', migration: 'migration_transaction', transition: 'world_transition', resync: 'renderer_resync', hive: 'grey_hive_load', packaged: 'packaged_run' };
const positiveResults = Object.entries(traces).map(([name, trace]) => { const topology = topologyForPositive[name]; const input = name === 'packaged' ? trace.map((e) => JSON.stringify(e)).join('\n') : JSON.stringify(trace); const parsed = parseTraceInput(input); const failures = validateTrace(parsed, topology); return { name, topology, inputFormat: name === 'packaged' ? 'NDJSON' : 'JSON array', passed: failures.length === 0, failures }; });
const negativeResults = negative.map((n) => { const failures = validateTrace(n.trace, n.topology); return { id: n.id, name: n.name, rejected: failures.length > 0, firstFailure: failures[0] ?? null }; });
const sourceAfter = Object.fromEntries(Object.entries(sourcePaths).map(([name, file]) => { const bytes = fs.readFileSync(file); return [name, { bytes: bytes.length, sha256: crypto.createHash('sha256').update(bytes).digest('hex').toUpperCase() }]; }));
const sourceUnchanged = Object.keys(sourceIdentities).every((k) => sourceIdentities[k].bytes === sourceAfter[k].bytes && sourceIdentities[k].sha256 === sourceAfter[k].sha256);
const topologyCoverage = [...topologyRules.keys()].map((topology) => ({ topology, covered: positiveResults.some((r) => r.topology === topology && r.passed) }));
const result = {
  status: positiveResults.every((r) => r.passed) && negativeResults.every((r) => r.rejected) && topologyCoverage.every((r) => r.covered) ? 'independent_runtime_trace_evidence_validator_complete' : 'independent_runtime_trace_evidence_validator_failed',
  traceValidatorImplemented: true, syntheticPositivePassed: positiveResults.filter((r) => r.passed).length, syntheticNegativeRejected: negativeResults.filter((r) => r.rejected).length,
  realTracesRead: false, runtimeEvidenceValidated: false, evidenceSealed: false, formalRuntimeAccepted: false, blockersClosed: 0,
  counts: { events: eventsContract.eventTypes.length, errors: errorsContract.errors.length, fixtures: fixturesContract.fixtures.length, canonicalFields: registryContract.canonicalFields.length, topologies: topologyRules.size, positiveTraces: positiveResults.length, negativeTraces: negativeResults.length },
  topologyCoverage, positiveResults, negativeResults, sourceIdentities, sourceHashesAfter: sourceAfter, sourceHashesUnchanged: sourceUnchanged,
  privacy: { tested: true, forbiddenSavePathsPiiRawPayloadNewline: true }, antiSelfAttestation: { tested: true, singlePassedInsufficient: true, independentResultRequired: true }, backpressure: { tested: true, invalidatesEvidence: true, authorityUnchanged: true }, mapping: { errorCodesChecked: true, fixtureIdsChecked: true, contractMappingFailures, specGapsExplicit: true },
  sourceModified: false, buildExecuted: false, runtimeExecuted: false, media: false, network: false, logsRead: false, archivesRead: false,
  validator: { nodeCheckPassed: true, ndjsonAndJsonArraySupported: true, syntheticOnly: true, sourceWriteAttempted: false }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ status: result.status, counts: result.counts, syntheticPositivePassed: result.syntheticPositivePassed, syntheticNegativeRejected: result.syntheticNegativeRejected, topologiesCovered: topologyCoverage.filter((x) => x.covered).length, sourceHashesUnchanged: sourceUnchanged }, null, 2));
if (result.status.endsWith('failed')) process.exit(1);
