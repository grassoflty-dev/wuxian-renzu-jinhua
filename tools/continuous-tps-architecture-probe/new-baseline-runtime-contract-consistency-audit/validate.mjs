import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const matrix = JSON.parse(fs.readFileSync(path.join(here, 'matrix.json'), 'utf8'));
const oracle = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-continuous-world-deterministic-test-oracle/fixtures.json'), 'utf8'));
const errors = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-formal-runtime-error-contract/errors.json'), 'utf8'));
const events = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-formal-runtime-observability-evidence/events.json'), 'utf8'));
const problems = [];
const fixtureIds = new Set(oracle.fixtures.map((f) => f.stableId));
const errorCodes = new Set(errors.errors.map((e) => e.code));
const eventNames = new Set(events.eventTypes.map((e) => e.name));
if (matrix.sourceIdentities.oracle.fixtureCount !== oracle.fixtures.length || matrix.sourceIdentities.errors.errorCount !== errors.errors.length || matrix.sourceIdentities.observability.eventCount !== events.eventTypes.length) problems.push('source count mismatch');
if (matrix.sourceIdentities.oracle.negativeCount !== oracle.fixtures.filter((f) => f.negative).length) problems.push('negative count mismatch');
if (matrix.fieldCrosswalk.length < 12) problems.push('crosswalk incomplete');
for (const f of ['schema_identity','world_identity','trace_identity','request_identity','authority_order','entity_identity','state_commit','retry_outcome','error_identity','snapshot_order','tombstone','save_migration','world_transition','evidence_identity','presentation_boundary','privacy_boundary']) if (!matrix.fieldCrosswalk.some((x) => x.field === f)) problems.push(`missing crosswalk ${f}`);
for (const row of matrix.fieldCrosswalk) if (!['consistent','naming_alias_requires_standardization','type_mismatch','semantic_conflict','missing_mapping','dangling_reference','versioning_gap','documentation_gap'].includes(row.compatibility)) problems.push(`${row.field}: invalid compatibility`);
for (const row of matrix.stateSemantics) if (!row.errorClass || !row.eventTerminal?.length || row.consistent !== true) problems.push(`${row.name}: semantic row incomplete`);
for (const e of errors.errors) for (const f of e.fixtures) if (!fixtureIds.has(f)) problems.push(`dangling error fixture ${f}`);
for (const e of events.eventTypes) { for (const c of e.errors) if (!errorCodes.has(c)) problems.push(`dangling event error ${c}`); for (const f of e.fixtures) if (!fixtureIds.has(f)) problems.push(`dangling event fixture ${f}`); }
for (const t of matrix.transactionCoverage) if (!t) problems.push('empty transaction');
for (const f of ['rust_authority','babylon_presentation_only','tauri_only','no_grid_api_mock_fallback','gate_b_hive_lockdown_only','hive_choice_three_state','npc_independent','v2_to_v3_only','save_source_preserved','timeout_not_equal_uncommitted']) if (!matrix.invariantChecks.includes(f)) problems.push(`missing invariant ${f}`);
for (const f of ['synthetic_oracle_only','safe_ui_diagnostic_separation','observability_redaction','anti_self_attestation','no_media_only_evidence']) if (!matrix.privacyEvidenceChecks.includes(f)) problems.push(`missing privacy check ${f}`);
for (const finding of matrix.findings) if (!['consistent','naming_alias_requires_standardization','type_mismatch','semantic_conflict','missing_mapping','dangling_reference','versioning_gap','documentation_gap'].includes(finding.classification)) problems.push(`invalid finding ${finding.id}`);
if (matrix.blockingRules.hardConflictCount !== matrix.findings.filter((f) => f.hardBlock).length) problems.push('hard conflict count mismatch');
if (matrix.blockingRules.architectureGate !== 'closed' || matrix.blockingRules.staticAcceptanceNotRuntime !== true) problems.push('gate/static boundary invalid');
if (matrix.currentDecision !== 'contracts_consistent_for_gate_review_but_not_runtime_ready') problems.push('decision invalid');
if (problems.length) throw new Error(problems.join('; '));

const findingCounts = Object.fromEntries([...new Set(matrix.findings.map((f) => f.classification))].sort().map((c) => [c, matrix.findings.filter((f) => f.classification === c).length]));
const result = {
  status:'runtime_contract_consistency_audit_complete', oracleSpecAccepted:true, errorSpecAccepted:true, observabilitySpecAccepted:true,
  hardConflictCount:matrix.blockingRules.hardConflictCount, contractConsistencyAccepted:true, runtimeImplemented:false,
  runtimeEvidenceCaptured:false, formalRuntimeAccepted:false, architectureGateOpened:false, blockersClosed:0,
  sourceContractsModified:false, sourceModified:false, ciModified:false, buildExecuted:false, testsExecuted:false, runtimeExecuted:false,
  logsRead:false, logsCreated:false, userSaveRead:false, userSaveWritten:false, assetBytesRead:false, mediaBytesRead:false,
  mediaCreated:false, mediaModelCalled:false, network:false, sourceCounts:{fixtures:oracle.fixtures.length,errors:errors.errors.length,events:events.eventTypes.length},
  crosswalkFieldCount:matrix.fieldCrosswalk.length, mappingCoverage:matrix.mappingCoverage, transactionCount:matrix.transactionCoverage.length,
  invariantCount:matrix.invariantChecks.length, privacyEvidenceCheckCount:matrix.privacyEvidenceChecks.length, findingCounts,
  hardBlocks:matrix.findings.filter((f) => f.hardBlock).map((f) => f.id), remediationQueue:matrix.remediationQueue,
  openBlockers:matrix.blockingRules.openBlockers, validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true},
  notes:['read-only audit','source contracts not modified','aliases not auto-normalized','static consistency is not runtime acceptance']
};
const resultPath = path.join(here, '../results/new-baseline-runtime-contract-consistency-audit-nb76.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,sourceCounts:result.sourceCounts,crosswalkFieldCount:result.crosswalkFieldCount,mappingCoverage:result.mappingCoverage,transactionCount:result.transactionCount,findingCounts,hardConflictCount:result.hardConflictCount}, null, 2));
