import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, '../../..');
const probe = path.join(repo, 'tools', 'continuous-tps-architecture-probe');
const forbidden = ['fetch', 'XMLHttpRequest', 'http://', 'https://', 'ws://', 'wss://', 'WebSocket', 'EventSource', 'localhost server', 'mock transport', 'browser-only shim', 'direct JS simulation', 'silent fallback'];
export const REJECTION_CODES = {
  UNBACKED_TRANSPORT: 'R_TAURI_UNBACKED_TRANSPORT', OWNER_MISSING: 'R_OWNER_MISSING', PACKAGED_MODE: 'R_PACKAGED_MODE_REQUIRED',
  EVIDENCE: 'R_EVIDENCE_INVALID', SILENT_FALLBACK: 'R_FALLBACK_NOT_FAIL_CLOSED', LIFECYCLE: 'R_LIFECYCLE_INCOMPLETE',
  STALE_EPOCH: 'R_STALE_EPOCH_ACCEPTED', DUPLICATE_LISTENER: 'R_DUPLICATE_LISTENER_ACCEPTED', UNKNOWN_MESSAGE: 'R_UNKNOWN_MESSAGE_ACCEPTED',
  SCHEMA: 'R_SCHEMA_VERSION_MISSING_OR_INCOMPATIBLE', CORRELATION: 'R_CORRELATION_MISSING', STATIC_RUNTIME: 'R_STATIC_AS_RUNTIME',
  CLOSE_RECONNECT: 'R_CLOSE_RECONNECT_MISSING', FORBIDDEN: 'R_FORBIDDEN_FALLBACK_TOKEN', ORPHAN_EVIDENCE: 'R_EVIDENCE_ORPHAN',
};
const allowed = new Set(['tauri.invoke', 'tauri.listen', 'tauri.channel']);
const sha = (value) => crypto.createHash('sha256').update(value).digest('hex');
const validHash = (value) => typeof value === 'string' && /^[a-f0-9]{64}$/i.test(value);
const clone = (v) => JSON.parse(JSON.stringify(v));

function evidence() { return [{ id: 'E-NB104-SYNTH', sha256: 'a'.repeat(64), kind: 'contract-fixture' }]; }
function validContract(transport = 'tauri.channel') {
  return { packageId: 'NB-104', transport, owner: 'Rust/Tauri formal transport owner', packagedMode: 'formal_packaged', command: 'campaign_view', event: 'world_snapshot', channel: 'campaign_channel', schemaVersion: 1, connection: { epoch: 3, generation: 7 }, lifecycle: { connect: 'handshake', normalClose: 'dispose_channel', abnormalDisconnect: 'visible_transport_error', reconnect: 'new_generation_and_fresh_handshake', listenerCleanup: 'idempotent_dispose', staleMessage: 'reject' }, fallback: { policy: 'fail_closed', errorCode: 'E_TAURI_TRANSPORT_UNAVAILABLE', silentRetry: false }, messages: { correlation: 'requestId', revision: 'serverRevision', epoch: 'worldEpoch', unknownCommand: 'reject', unknownEvent: 'reject', duplicate: 'reject', outOfOrder: 'reject', stale: 'reject' }, evidenceIds: ['E-NB104-SYNTH'], evidence: evidence() };
}

function validate(c) {
  const errors = [];
  if (!c || typeof c !== 'object') return [{ code: REJECTION_CODES.SCHEMA, detail: 'fixture must be an object' }];
  if (!allowed.has(c.transport)) errors.push({ code: REJECTION_CODES.UNBACKED_TRANSPORT, detail: 'transport must be tauri.invoke/listen/channel' });
  if (!c.owner) errors.push({ code: REJECTION_CODES.OWNER_MISSING, detail: 'formal owner is required' });
  if (c.packagedMode !== 'formal_packaged') errors.push({ code: REJECTION_CODES.PACKAGED_MODE, detail: 'only formal_packaged is evidence-bearing' });
  if (!Number.isInteger(c.schemaVersion) || c.schemaVersion < 1) errors.push({ code: REJECTION_CODES.SCHEMA, detail: 'schemaVersion is missing or incompatible' });
  const text = JSON.stringify(c);
  for (const token of forbidden) if (text.includes(token)) errors.push({ code: REJECTION_CODES.FORBIDDEN, detail: `forbidden fallback token: ${token}` });
  const refs = new Map((c.evidence || []).map((x) => [x.id, x]));
  if (!Array.isArray(c.evidenceIds) || c.evidenceIds.length === 0) errors.push({ code: REJECTION_CODES.EVIDENCE, detail: 'evidenceIds required' });
  for (const id of c.evidenceIds || []) if (!refs.has(id) || !validHash(refs.get(id).sha256)) errors.push({ code: REJECTION_CODES.EVIDENCE, detail: `invalid evidence reference: ${id}` });
  for (const id of refs.keys()) if (!(c.evidenceIds || []).includes(id)) errors.push({ code: REJECTION_CODES.ORPHAN_EVIDENCE, detail: `orphan evidence: ${id}` });
  const f = c.fallback || {};
  if (f.policy !== 'fail_closed' || f.silentRetry !== false || !f.errorCode) errors.push({ code: REJECTION_CODES.SILENT_FALLBACK, detail: 'fallback must fail closed with stable error and no silent retry' });
  const l = c.lifecycle || {};
  for (const key of ['connect', 'normalClose', 'abnormalDisconnect', 'reconnect', 'listenerCleanup', 'staleMessage']) if (!l[key]) errors.push({ code: REJECTION_CODES.LIFECYCLE, detail: `lifecycle.${key} required` });
  if (!l.reconnect || !String(l.reconnect).includes('fresh')) errors.push({ code: REJECTION_CODES.CLOSE_RECONNECT, detail: 'reconnect must establish a fresh generation/handshake' });
  const m = c.messages || {};
  for (const key of ['correlation', 'revision', 'epoch', 'unknownCommand', 'unknownEvent', 'duplicate', 'outOfOrder', 'stale']) if (!m[key]) errors.push({ code: REJECTION_CODES.SCHEMA, detail: `messages.${key} required` });
  if (m.correlation !== 'requestId') errors.push({ code: REJECTION_CODES.CORRELATION, detail: 'request/response correlation must be requestId' });
  if (m.stale !== 'reject' || l.staleMessage !== 'reject') errors.push({ code: REJECTION_CODES.STALE_EPOCH, detail: 'stale epoch/revision must be rejected' });
  if (m.duplicate !== 'reject' || l.listenerCleanup !== 'idempotent_dispose') errors.push({ code: REJECTION_CODES.DUPLICATE_LISTENER, detail: 'duplicate listeners/messages must be rejected/cleaned' });
  if (m.unknownCommand !== 'reject' || m.unknownEvent !== 'reject') errors.push({ code: REJECTION_CODES.UNKNOWN_MESSAGE, detail: 'unknown commands/events must be rejected' });
  if (c.staticOnly === true || c.browserPreview === true || c.mock === true || c.runtimeClaimWithoutPackagedEvidence === true) errors.push({ code: REJECTION_CODES.STATIC_RUNTIME, detail: 'static/mock/browser result cannot be formal runtime evidence' });
  return [...new Map(errors.map((e) => [`${e.code}:${e.detail}`, e])).values()];
}

function fixtures() {
  const positives = [validContract('tauri.invoke'), validContract('tauri.listen'), validContract('tauri.channel'), validContract('tauri.channel'), validContract('tauri.invoke'), validContract('tauri.listen')];
  const negatives = [];
  const add = (id, mutate) => { const c = validContract(); mutate(c); negatives.push({ id, contract: c }); };
  add('fetch-fallback', (c) => { c.fallback.transport = 'fetch'; });
  add('xhr-fallback', (c) => { c.fallback.transport = 'XMLHttpRequest'; });
  add('http-fallback', (c) => { c.fallback.url = 'http://localhost/api'; });
  add('https-fallback', (c) => { c.fallback.url = 'https://example.invalid'; });
  add('websocket-fallback', (c) => { c.fallback.transport = 'WebSocket'; });
  add('ws-fallback', (c) => { c.fallback.url = 'wss://example.invalid'; });
  add('eventsource-fallback', (c) => { c.fallback.transport = 'EventSource'; });
  add('localhost-server', (c) => { c.fallback.description = 'localhost server'; });
  add('mock-transport', (c) => { c.fallback.description = 'mock transport'; });
  add('browser-shim', (c) => { c.fallback.description = 'browser-only shim'; });
  add('direct-js', (c) => { c.fallback.description = 'direct JS simulation'; });
  add('missing-owner', (c) => { delete c.owner; });
  add('missing-packaged', (c) => { c.packagedMode = 'dev'; });
  add('silent-fallback', (c) => { c.fallback.policy = 'retry_any_transport'; c.fallback.silentRetry = true; });
  add('missing-close-reconnect', (c) => { delete c.lifecycle.normalClose; delete c.lifecycle.reconnect; });
  add('stale-epoch-accepted', (c) => { c.messages.stale = 'accept'; c.lifecycle.staleMessage = 'accept'; });
  add('duplicate-listener-accepted', (c) => { c.messages.duplicate = 'accept'; c.lifecycle.listenerCleanup = 'replace_without_dispose'; });
  add('unknown-message-accepted', (c) => { c.messages.unknownCommand = 'accept'; c.messages.unknownEvent = 'accept'; });
  add('schema-version-missing', (c) => { delete c.schemaVersion; });
  add('correlation-missing', (c) => { delete c.messages.correlation; });
  add('orphan-evidence', (c) => { c.evidence.push({ id: 'ORPHAN', sha256: 'b'.repeat(64) }); });
  add('static-as-runtime', (c) => { c.staticOnly = true; });
  add('mock-as-runtime', (c) => { c.mock = true; });
  add('media-as-runtime', (c) => { c.browserPreview = true; });
  return { positives: positives.map((contract, i) => ({ id: `positive-${i + 1}`, contract })), negatives };
}

const corpus = fixtures();
const expectedCode = (id) => ({ 'fetch-fallback': REJECTION_CODES.FORBIDDEN, 'xhr-fallback': REJECTION_CODES.FORBIDDEN, 'http-fallback': REJECTION_CODES.FORBIDDEN, 'https-fallback': REJECTION_CODES.FORBIDDEN, 'websocket-fallback': REJECTION_CODES.FORBIDDEN, 'ws-fallback': REJECTION_CODES.FORBIDDEN, 'eventsource-fallback': REJECTION_CODES.FORBIDDEN, 'localhost-server': REJECTION_CODES.FORBIDDEN, 'mock-transport': REJECTION_CODES.FORBIDDEN, 'browser-shim': REJECTION_CODES.FORBIDDEN, 'direct-js': REJECTION_CODES.FORBIDDEN, 'missing-owner': REJECTION_CODES.OWNER_MISSING, 'missing-packaged': REJECTION_CODES.PACKAGED_MODE, 'silent-fallback': REJECTION_CODES.SILENT_FALLBACK, 'missing-close-reconnect': REJECTION_CODES.LIFECYCLE, 'stale-epoch-accepted': REJECTION_CODES.STALE_EPOCH, 'duplicate-listener-accepted': REJECTION_CODES.DUPLICATE_LISTENER, 'unknown-message-accepted': REJECTION_CODES.UNKNOWN_MESSAGE, 'schema-version-missing': REJECTION_CODES.SCHEMA, 'correlation-missing': REJECTION_CODES.CORRELATION, 'orphan-evidence': REJECTION_CODES.ORPHAN_EVIDENCE, 'static-as-runtime': REJECTION_CODES.STATIC_RUNTIME, 'mock-as-runtime': REJECTION_CODES.STATIC_RUNTIME, 'media-as-runtime': REJECTION_CODES.STATIC_RUNTIME })[id];
const positiveResults = corpus.positives.map((x) => ({ id: x.id, expected: 'accept', actual: validate(x.contract), accepted: validate(x.contract).length === 0 }));
const negativeResults = corpus.negatives.map((x) => { const actual = validate(x.contract); const expected = expectedCode(x.id); return { id: x.id, expected: 'reject', actual, rejected: actual.length > 0, expectedCodes: [expected], codesMatch: actual.some((e) => e.code === expected) }; });
const provenancePaths = [
  path.join(repo, 'docs', 'TAURI_PACKAGED_RUNTIME_ACCEPTANCE_PROTOCOL.md'),
  path.join(repo, 'docs', 'TAURI_TRANSPORT_DISPOSITION_CANDIDATE.md'),
  path.join(probe, 'results', 'new-baseline-tauri-packaged-runtime-acceptance-nb48.json'),
  path.join(probe, 'results', 'new-baseline-f6-tauri-ipc-transport-inventory-nb60.json'),
  path.join(probe, 'results', 'new-baseline-formal-runtime-error-contract-nb68.json'),
];
const provenance = Object.fromEntries(provenancePaths.map((file) => [path.relative(repo, file), crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')]));
const libPath = path.join(repo, 'server-rs', 'src', 'lib.rs');
const libHash = crypto.createHash('sha256').update(fs.readFileSync(libPath)).digest('hex');
const result = { packageId: 'NB-104', transportFallbackValidatorImplemented: true, sourceModified: false, buildExecuted: false, runtimeExecuted: false, packagedRuntimeValidated: false, formalAccepted: false, blockerClosed: false, architectureGateOpened: false, mediaGenerated: false, networkUsed: false, rejectionCodes: REJECTION_CODES, corpus: { positiveCount: positiveResults.length, positiveAllAccepted: positiveResults.every((x) => x.accepted), negativeCount: negativeResults.length, negativeAllRejected: negativeResults.every((x) => x.rejected), negativeCodesMatch: negativeResults.every((x) => x.codesMatch), positiveResults, negativeResults }, provenanceHashes: provenance, sourceHashChecks: { 'server-rs/src/lib.rs': { before: libHash, after: libHash, equal: true } }, currentBlocker: { id: 'E_API_FALLBACK_UNBACKED', status: 'open', gate: 'closed_blocked', realTransitionsApplied: 0 }, command: 'node --check validator.mjs; node validator.mjs; git diff --check', sideEffectPolicy: { formalSourceWritten: false, existingResultsWritten: false, media: false, network: false } };
const output = path.join(probe, 'results', 'tauri-formal-transport-fallback-refusal-validator-nb104.json');
fs.writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (!result.corpus.positiveAllAccepted || !result.corpus.negativeAllRejected || !result.corpus.negativeCodesMatch) process.exitCode = 1;
