import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, '../../..');
const probe = path.join(repo, 'tools', 'continuous-tps-architecture-probe');
const allowedKinds = new Set(['product_answer', 'supersession_acceptance', 'defer_reject']);
const allowedDecisions = new Set(['accept', 'reject', 'defer']);
const required = ['decider', 'timestamp', 'scope', 'effectiveVersion', 'evidenceIds', 'conditions', 'reason', 'reopen', 'revoke'];

const isObject = (v) => v !== null && typeof v === 'object' && !Array.isArray(v);
const nonEmpty = (v) => typeof v === 'string' && v.trim().length > 0;

export function validateDecisionRecord(record, context = {}) {
  const errors = [];
  if (!isObject(record)) return ['record must be an object'];
  if (!allowedKinds.has(record.kind)) errors.push('kind must be product_answer, supersession_acceptance, or defer_reject');
  if (!nonEmpty(record.decisionId)) errors.push('decisionId is required');
  if (!allowedDecisions.has(record.decision)) errors.push('decision must be accept, reject, or defer');
  for (const key of required) {
    if (record[key] === undefined || record[key] === null || (typeof record[key] === 'string' && !record[key].trim())) errors.push(`${key} is required`);
  }
  if (record.timestamp && Number.isNaN(Date.parse(record.timestamp))) errors.push('timestamp must be an ISO date');
  if (record.scope !== undefined && !nonEmpty(record.scope) && !isObject(record.scope)) errors.push('scope must be non-empty');
  if (record.effectiveVersion !== undefined && !nonEmpty(record.effectiveVersion)) errors.push('effectiveVersion must be non-empty');
  if (!Array.isArray(record.evidenceIds) || record.evidenceIds.length === 0 || record.evidenceIds.some((x) => !nonEmpty(x))) errors.push('evidenceIds must be non-empty strings');
  if (!Array.isArray(record.conditions)) errors.push('conditions must be an array');
  if (record.reopen !== undefined && typeof record.reopen !== 'boolean' && !isObject(record.reopen)) errors.push('reopen must be boolean or object');
  if (record.revoke !== undefined && typeof record.revoke !== 'boolean' && !isObject(record.revoke)) errors.push('revoke must be boolean or object');
  if (record.kind === 'product_answer') {
    if (!nonEmpty(record.selectedOption)) errors.push('product answer selectedOption is required');
    if (context.allowedOptions && !context.allowedOptions.includes(record.selectedOption)) errors.push('selectedOption is not one of the decision card options');
  }
  if (record.kind === 'supersession_acceptance' && !nonEmpty(record.replacementId)) errors.push('supersession replacementId is required');
  if (record.kind === 'defer_reject' && !nonEmpty(record.reason)) errors.push('defer/reject reason is required');
  if (record.recommendationUsedAsDecision === true) errors.push('recommendation cannot be treated as a decision');
  if (record.mediaAuthorized === true || record.mediaGeneration === true) errors.push('media authorization is outside this validator contract');
  if (record.architectureGateOpened === true || record.runtimeImplemented === true) errors.push('decision cannot open Gate or claim runtime');
  return errors;
}

function sourceSummary() {
  const nb88 = JSON.parse(fs.readFileSync(path.join(probe, 'new-baseline-architecture-gate-supersession-decision-bundle', 'decisions.json'), 'utf8'));
  const nb90 = JSON.parse(fs.readFileSync(path.join(probe, 'new-baseline-product-decision-batch-1', 'packet.json'), 'utf8'));
  const nb95 = JSON.parse(fs.readFileSync(path.join(probe, 'new-baseline-world-production-supersession-decision-bundle', 'decisions.json'), 'utf8'));
  return {
    NB88: { pendingCount: nb88.decisionRecords.filter((x) => x.decision === null && x.acceptanceRecorded === false).length, records: nb88.decisionRecords.length },
    NB90: { pendingCount: nb90.decisionCards.filter((x) => x.answer === 'pending').length, records: nb90.decisionCards.length },
    NB95: { pendingCount: nb95.records.length, records: nb95.records.length },
    validCompletedDecisions: 0,
    sourceModified: false,
  };
}

function dependencyErrors(records) {
  const errors = [];
  const byId = new Map(records.map((r) => [r.decisionId, r]));
  const d4 = byId.get('D4');
  if (d4?.decision === 'accept' && !['D1', 'D2', 'D3'].every((id) => byId.get(id)?.decision === 'accept')) errors.push('NB-22/D4 cannot be accepted before D1-D3');
  for (const id of ['W1', 'W2', 'W3']) {
    const r = byId.get(id);
    if (r?.decision === 'accept' && r.productAnswersRecorded !== true) errors.push(`${id} requires related NB-90 answers`);
  }
  if (records.some((r) => r.returnStationApproved === true && r.returnStationImplemented === true)) errors.push('Return Station acceptance does not imply implementation');
  if (records.some((r) => r.architectureGateOpened === true)) errors.push('supersession/product decision cannot open Architecture Gate');
  return errors;
}

export function validateBundle(records) {
  const errors = [];
  const packet = JSON.parse(fs.readFileSync(path.join(probe, 'new-baseline-product-decision-batch-1', 'packet.json'), 'utf8'));
  const cards = new Map(packet.decisionCards.map((card) => [card.decisionId, card.options]));
  for (const record of records) errors.push(...validateDecisionRecord(record, { allowedOptions: cards.get(record.decisionId) }));
  errors.push(...dependencyErrors(records));
  return [...new Set(errors)];
}

function loadFixtures() {
  return JSON.parse(fs.readFileSync(path.join(here, 'negative-fixtures.json'), 'utf8'));
}

const sources = sourceSummary();
const fixtures = loadFixtures();
const fixtureResults = fixtures.map((fixture) => ({ id: fixture.id, rejected: validateBundle([fixture.record]).length > 0, errors: validateBundle([fixture.record]) }));
const allNegativeRejected = fixtureResults.length >= 15 && fixtureResults.every((x) => x.rejected);
const result = {
  schemaVersion: 1,
  validator: 'decision-record-validator',
  decisionValidatorImplemented: true,
  decisionsMade: false,
  architectureGateOpened: false,
  sourceSummary: sources,
  negativeSuite: { fixtureCount: fixtureResults.length, allRejected: allNegativeRejected, results: fixtureResults },
  dependencyChecks: { nb22RequiresD1D3: true, nb27Nb42RequireNb90Answers: true, returnStationAcceptanceIsNotImplementation: true },
  sideEffectPolicy: { sourceFilesModified: false, answersWritten: false, decisionsWritten: false, media: false, network: false, runtime: false },
};
const out = path.join(probe, 'results', 'decision-record-validator-nb98.json');
fs.writeFileSync(out, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (!allNegativeRejected || sources.NB88.pendingCount !== 4 || sources.NB90.pendingCount !== 5 || sources.NB95.pendingCount !== 3) process.exitCode = 1;
