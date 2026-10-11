import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, '../../..');
const probe = path.join(repo, 'tools', 'continuous-tps-architecture-probe');
const nb82Path = path.join(probe, 'new-baseline-architecture-closure-spec-replacement', 'spec.json');
const nb75Path = path.join(probe, 'new-baseline-formal-runtime-gate-readiness-replacement', 'ledger.json');
const spec = JSON.parse(fs.readFileSync(nb82Path, 'utf8'));
const ledger = JSON.parse(fs.readFileSync(nb75Path, 'utf8'));

export const STATES = ['open', 'implementation_candidate', 'source_implemented', 'compile_verified', 'unit_verified', 'integration_verified', 'packaged_verified', 'manual_or_required_acceptance', 'closed'];
const core = ['compile_verified', 'unit_verified', 'integration_verified', 'packaged_verified'];
const blockerIds = spec.blockerClosureRules.map((x) => x.code);
const closureAuthority = 'primary session Gate';
const hashes = (text) => /^[a-f0-9]{64}$/i.test(text);

const transitions = new Map(STATES.slice(0, -1).map((state, i) => [state, STATES[i + 1]]));
const blockerRules = new Map(spec.blockerClosureRules.map((x) => [x.code, x]));
const ledgerBlockers = new Map(ledger.blockers.map((x) => [x.id, x]));
const dependencies = new Map([
  ['E_LEGACY_GRID_REACHABLE', ['E_API_FALLBACK_UNBACKED', 'E_NO_FULL_SAVE_V3_RUNTIME_RESTORE']],
  ['E_WORLD_V3_NOT_RUNTIME_WIRED', ['E_CONTINUOUS_PRIMITIVES_MISSING']],
  ['E_CONTINUOUS_PRIMITIVES_MISSING', []],
  ['E_API_FALLBACK_UNBACKED', ['E_WORLD_V3_NOT_RUNTIME_WIRED']],
  ['E_NO_FULL_SAVE_V3_RUNTIME_RESTORE', ['E_WORLD_V3_NOT_RUNTIME_WIRED']],
]);
function dagIsAcyclic() {
  const visiting = new Set(); const visited = new Set();
  const visit = (node) => { if (visiting.has(node)) return false; if (visited.has(node)) return true; visiting.add(node); for (const dep of dependencies.get(node) || []) if (!visit(dep)) return false; visiting.delete(node); visited.add(node); return true; };
  return [...dependencies.keys()].every(visit);
}

function requiredFields(t) {
  const errors = [];
  for (const key of ['blockerId', 'from', 'to', 'owner', 'closureAuthority', 'sourceRevision', 'timestamp', 'conditions', 'rollbackRules', 'reopenRules']) {
    if (t[key] === undefined || t[key] === null || (typeof t[key] === 'string' && !t[key].trim())) errors.push(`missing ${key}`);
  }
  if (!Array.isArray(t.evidenceIds) || t.evidenceIds.length === 0) errors.push('missing evidenceIds');
  if (!Array.isArray(t.artifacts) || t.artifacts.length === 0) errors.push('missing artifact hashes');
  if (!Array.isArray(t.conditions)) errors.push('conditions must be an array');
  if (!Array.isArray(t.reopenRules) && typeof t.reopenRules !== 'object') errors.push('reopen rules must be declared');
  if (t.timestamp && Number.isNaN(Date.parse(t.timestamp))) errors.push('invalid timestamp');
  return errors;
}

function evidenceErrors(t, finalState = false) {
  const errors = [];
  const ids = new Set(t.evidenceIds || []);
  const evidence = t.evidence || [];
  const byId = new Map(evidence.map((e) => [e.id, e]));
  for (const id of ids) {
    const e = byId.get(id);
    if (!e) errors.push(`evidence ${id} not supplied`);
    else if (!hashes(e.sha256)) errors.push(`evidence ${id} has invalid hash`);
  }
  const artifactById = new Map((t.artifacts || []).map((a) => [a.id, a.sha256]));
  for (const e of evidence) if (artifactById.get(e.id) !== e.sha256) errors.push(`artifact hash mismatch for ${e.id}`);
  const rule = blockerRules.get(t.blockerId);
  if (finalState && rule) {
    const levels = new Set(evidence.map((e) => e.level));
    for (const required of rule.requiredEvidence) {
      const token = required.toLowerCase();
      if (!evidence.some((e) => e.description?.toLowerCase().includes(token) || e.level?.toLowerCase().includes(token))) errors.push(`required evidence missing: ${required}`);
    }
    const ledgerRule = ledgerBlockers.get(t.blockerId);
    const ledgerLevels = ledgerRule?.requiredEvidence || [];
    for (const level of ledgerLevels) if (!levels.has(level) && !evidence.some((e) => e.description?.toLowerCase().includes(level.toLowerCase()))) errors.push(`ledger evidence level missing: ${level}`);
    if (!t.manualSkipJustification && !levels.has('manual')) errors.push('manual evidence required unless explicit rule declares skip');
    if (t.manualSkipJustification && t.manualSkipJustification.allowed !== true) errors.push('manual skip is not explicitly allowed');
  }
  return errors;
}

export function validateTransition(t, context = {}) {
  const errors = requiredFields(t);
  if (!blockerIds.includes(t.blockerId)) errors.push('unknown blockerId');
  if (!STATES.includes(t.from) || !STATES.includes(t.to)) errors.push('unknown state');
  if (STATES.includes(t.from) && STATES.includes(t.to)) {
    if (t.to === 'open') {
      if (!t.reopen || t.reopen.trigger === undefined || !t.reopen.reason) errors.push('reopen to open requires trigger and reason');
    } else if (transitions.get(t.from) !== t.to) errors.push(`illegal transition ${t.from}->${t.to}`);
  }
  if (t.closureAuthority !== closureAuthority) errors.push('unauthorized closure authority');
  if (t.from === 'open' && t.to === 'closed') errors.push('direct open->closed forbidden');
  if (core.includes(t.to)) {
    const level = t.to.replace('_verified', '');
    if (!t.evidence?.some((e) => e.level === level || e.description?.toLowerCase().includes(level))) errors.push(`missing ${level} evidence`);
  }
  errors.push(...evidenceErrors(t, t.to === 'closed'));
  if (t.staticOnly === true || t.mock === true || t.mediaOnly === true || t.candidateOnly === true) errors.push('static/mock/media/candidate evidence cannot close blocker');
  if (context.closedBlockers && t.to === 'closed') {
    for (const dep of dependencies.get(t.blockerId) || []) if (!context.closedBlockers.has(dep)) errors.push(`dependency open: ${dep}`);
  }
  return [...new Set(errors)];
}

export function validateLifecycle(lifecycle, context = {}) {
  const errors = [];
  if (!Array.isArray(lifecycle) || lifecycle.length === 0) return ['lifecycle must contain transitions'];
  let state = 'open';
  for (const transition of lifecycle) {
    if (transition.from !== state && !transition.reopen) errors.push(`state continuity expected ${state}, got ${transition.from}`);
    errors.push(...validateTransition(transition, context));
    state = transition.to;
  }
  if (state !== 'closed') errors.push('lifecycle does not reach closed');
  return [...new Set(errors)];
}

function makeEvidence(blockerId) {
  const descriptions = blockerRules.get(blockerId).requiredEvidence;
  const required = descriptions.map((description, i) => ({ id: `${blockerId}-e${i}`, level: ['source', 'compile', 'unit', 'integration', 'packaged', 'manual'][i] || 'manual', description, sha256: 'a'.repeat(64) }));
  const supplemental = ['static', 'compile', 'unit', 'integration', 'packaged', 'manual'].map((level, i) => ({ id: `${blockerId}-supplement-${level}`, level, description: `${level} evidence`, sha256: 'a'.repeat(64) }));
  return [...required, ...supplemental];
}
function base(blockerId, from, to, evidence, extra = {}) {
  return { blockerId, from, to, owner: 'synthetic owner', closureAuthority, sourceRevision: 'synthetic-revision-1', evidenceIds: evidence.map((e) => e.id), artifacts: evidence.map((e) => ({ id: e.id, sha256: e.sha256 })), timestamp: '2026-01-01T00:00:00Z', conditions: ['synthetic fixture only'], rollbackRules: { onFailure: 'reopen' }, reopenRules: { triggers: ['source drift', 'test regression', 'artifact hash mismatch'] }, evidence, ...extra };
}

function positiveFixtures() {
  return blockerIds.slice(0, 5).map((blockerId) => {
    const e = makeEvidence(blockerId);
    const sequence = [];
    let from = 'open';
    for (const to of STATES.slice(1)) {
      const stageEvidence = e.filter((x) => to === 'closed' || x.level === to.replace('_verified', '') || to === 'implementation_candidate' || to === 'source_implemented' || to === 'manual_or_required_acceptance');
      sequence.push(base(blockerId, from, to, stageEvidence.length ? stageEvidence : [e[0]], to === 'closed' ? { manualSkipJustification: { allowed: true, rule: 'synthetic closure rule explicitly permits manual omission for this fixture' } } : {}));
      from = to;
    }
    return { id: `positive-${blockerId}`, lifecycle: sequence };
  });
}

function negativeFixtures() {
  const e = makeEvidence('E_CONTINUOUS_PRIMITIVES_MISSING');
  const valid = base('E_CONTINUOUS_PRIMITIVES_MISSING', 'implementation_candidate', 'source_implemented', e);
  const cases = [
    ['direct-open-closed', base(valid.blockerId, 'open', 'closed', e)],
    ['missing-owner', { ...valid, owner: '' }],
    ['missing-compile', { ...base(valid.blockerId, 'source_implemented', 'unit_verified', e.filter((x) => x.level !== 'compile')), from: 'source_implemented', to: 'unit_verified' }],
    ['static-only', { ...valid, staticOnly: true }], ['mock-evidence', { ...valid, mock: true }], ['media-only', { ...valid, mediaOnly: true }],
    ['candidate-only', { ...valid, candidateOnly: true }], ['wrong-hash', { ...valid, artifacts: [{ id: e[0].id, sha256: 'bad' }] }], ['stale-revision', { ...valid, sourceRevision: '' }],
    ['dependency-open', { ...base('E_WORLD_V3_NOT_RUNTIME_WIRED', 'manual_or_required_acceptance', 'closed', makeEvidence('E_WORLD_V3_NOT_RUNTIME_WIRED'), { manualSkipJustification: { allowed: true, rule: 'synthetic' } }) }],
    ['unauthorized-closure', { ...valid, closureAuthority: 'candidate validator' }], ['invalid-reopen', { ...valid, from: 'closed', to: 'open', reopen: { trigger: 'unknown' } }],
    ['skip-integration', { ...base(valid.blockerId, 'unit_verified', 'packaged_verified', e) }], ['no-evidence-ids', { ...valid, evidenceIds: [] }], ['unknown-blocker', { ...valid, blockerId: 'E_UNKNOWN' }],
    ['wrong-state', { ...valid, from: 'open', to: 'unit_verified' }], ['missing-conditions', { ...valid, conditions: null }], ['missing-rollback', { ...valid, rollbackRules: null }],
    ['artifact-mismatch', { ...valid, evidence: [{ ...e[0], sha256: 'b'.repeat(64) }], artifacts: [{ id: e[0].id, sha256: 'c'.repeat(64) }] }],
    ['reopen-without-reason', { ...valid, from: 'closed', to: 'open', reopen: { trigger: 'source drift' } }], ['no-manual-skip-rule', { ...base(valid.blockerId, 'manual_or_required_acceptance', 'closed', e.filter((x) => x.level !== 'manual')) }],
  ];
  return cases.map(([id, transition]) => ({ id, transition }));
}

const positive = positiveFixtures();
const negative = negativeFixtures();
const negativeResults = negative.map((x) => ({ id: x.id, rejected: validateTransition(x.transition, { closedBlockers: new Set() }).length > 0, errors: validateTransition(x.transition, { closedBlockers: new Set() }) }));
const positiveResults = positive.map((x) => {
  const blockerId = x.lifecycle[0].blockerId;
  const closedBlockers = new Set(dependencies.get(blockerId) || []);
  const errors = validateLifecycle(x.lifecycle, { closedBlockers });
  return { id: x.id, accepted: errors.length === 0, errors };
});
const current = Object.fromEntries(blockerIds.map((id) => [id, 'open']));
const result = {
  schemaVersion: 1,
  validator: 'formal-blocker-state-machine-validator',
  stateModel: STATES,
  blockerValidatorImplemented: true,
  realTransitionsApplied: 0,
  currentBlockerSnapshot: current,
  blockersOpen: Object.values(current).filter((x) => x === 'open').length,
  blockersClosed: Object.values(current).filter((x) => x === 'closed').length,
  architectureGateOpened: false,
  requiredEvidence: Object.fromEntries(spec.blockerClosureRules.map((x) => [x.code, x.requiredEvidence])),
  dependencyDag: { edges: Object.fromEntries([...dependencies].map(([k, v]) => [k, v])), acyclic: dagIsAcyclic() },
  transitionCoverage: { legalEdges: STATES.length - 1, reopenToOpen: true, coreEvidenceRequired: core, manualSkipRequiresExplicitRule: true },
  fixtures: { negativeCount: negativeResults.length, negativeAllRejected: negativeResults.every((x) => x.rejected), positiveCount: positiveResults.length, positiveAllAccepted: positiveResults.every((x) => x.accepted), negativeResults, positiveResults },
  gateDecision: 'closed_blocked',
  sourceHashes: { nb82Spec: crypto.createHash('sha256').update(fs.readFileSync(nb82Path)).digest('hex'), nb75Ledger: crypto.createHash('sha256').update(fs.readFileSync(nb75Path)).digest('hex') },
  flags: { sourceFilesModified: false, buildExecuted: false, runtimeExecuted: false, media: false, network: false, ledgerWritten: false, closureSpecWritten: false },
};
const out = path.join(probe, 'results', 'formal-blocker-state-machine-validator-nb100.json');
fs.writeFileSync(out, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (result.blockersOpen !== 5 || result.blockersClosed !== 0 || !result.fixtures.negativeAllRejected || !result.fixtures.positiveAllAccepted) process.exitCode = 1;
