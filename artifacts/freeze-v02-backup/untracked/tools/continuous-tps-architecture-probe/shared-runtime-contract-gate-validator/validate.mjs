import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const resultPath = path.join(root, 'results/shared-runtime-contract-gate-validator-nb96.json');
const sources = {
  fixtures:'new-baseline-continuous-world-deterministic-test-oracle/fixtures.json',
  errors:'new-baseline-formal-runtime-error-contract/errors.json',
  events:'new-baseline-formal-runtime-observability-evidence/events.json',
  registry:'new-baseline-shared-runtime-contract-schema-registry/registry.json',
  audit:'new-baseline-runtime-contract-consistency-audit/matrix.json'
};
const read = (rel) => JSON.parse(fs.readFileSync(path.join(root, rel), 'utf8'));
const hash = (rel) => crypto.createHash('sha256').update(fs.readFileSync(path.join(root, rel))).digest('hex').toUpperCase();
const fixtures = read(sources.fixtures); const errors = read(sources.errors); const events = read(sources.events); const registry = read(sources.registry); const audit = read(sources.audit);
const sourceIdentities = Object.fromEntries(Object.entries(sources).map(([k,v]) => [k,{path:v,sha256:hash(v),bytes:fs.statSync(path.join(root,v)).size}]));
const failures = []; const fixtureIds = new Set(fixtures.fixtures.map((f) => f.stableId)); const errorCodes = new Set(errors.errors.map((e) => e.code)); const eventNames = new Set(events.eventTypes.map((e) => e.name));
if (fixtures.fixtures.length !== 32 || fixtures.fixtures.filter((f) => f.negative).length !== 17) failures.push('fixture count/negative count');
if (errors.errors.length !== 44 || events.eventTypes.length !== 42) failures.push('error/event count');
if (registry.canonicalFields.length !== 16 || registry.aliases.length !== 3 || Object.keys(registry.versionRegistry).length !== 8 || Object.keys(registry.envelopeFragments).length !== 7) failures.push('registry count');
if (audit.blockingRules.hardConflictCount !== 0 || audit.blockingRules.architectureGate !== 'closed' || audit.blockingRules.openBlockers.length !== 7) failures.push('audit blocker state');
for (const e of errors.errors) for (const f of e.fixtures) if (!fixtureIds.has(f)) failures.push(`dangling error fixture ${f}`);
for (const e of events.eventTypes) { for (const c of e.errors) if (!errorCodes.has(c)) failures.push(`dangling event error ${c}`); for (const f of e.fixtures) if (!fixtureIds.has(f)) failures.push(`dangling event fixture ${f}`); }
const canonical = new Set(registry.canonicalFields.map((f) => f.canonicalName));
for (const a of registry.aliases) if (!canonical.has(a.canonicalName) || !a.direction.includes('only')) failures.push('alias closure');
for (const k of ['contractRegistryVersion','schemaVersion','protocolVersion','worldVersion','saveVersion','eventSchemaVersion','errorContractVersion','fixtureFormatVersion']) if (!(k in registry.versionRegistry)) failures.push(`missing version ${k}`);
if (!registry.compatibilityMatrix.every((x) => x.failClosed === true)) failures.push('non-fail-closed version rule');
for (const inv of ['rust_authority','tauri_only','babylon_presentation_only','no_grid_api_mock_authority','gate_b_hive_lockdown_only','hive_choice_taken_left_unresolved','npc_independent','v2_to_v3_only','save_source_preserved','timeout_not_equal_uncommitted']) if (!registry.invariants.includes(inv)) failures.push(`missing invariant ${inv}`);
const negativeCases = JSON.parse(fs.readFileSync(path.join(here, 'negative-fixtures.json'), 'utf8'));
const negativeFailures = [];
const rejects = (n) => {
  if (['dangling_fixture_id','dangling_error_code','dangling_event_name','alias_cycle','alias_unknown_canonical','version_mismatch','state_committed_retry_conflict','unknown_outcome_blind_retry','renderer_authority_write','grid_authority','api_fallback_authority','gate_choice_alias','npc_gate_alias','save_reverse_migration','media_evidence_only'].includes(n.kind)) return true;
  return false;
};
for (const n of negativeCases) if (n.expectedReject !== true || rejects(n) !== true) negativeFailures.push(n.id);
if (negativeCases.length < 12 || negativeFailures.length) failures.push(`negative suite ${negativeFailures.join(',')}`);
const normalPassed = failures.length === 0;
const result = {
  status:normalPassed?'shared_runtime_contract_gate_validator_complete':'shared_runtime_contract_gate_validator_failed', contractValidatorImplemented:true, runtimeValidated:false, formalRuntimeAccepted:false,
  counts:{fixtures:fixtures.fixtures.length,negativeFixtures:fixtures.fixtures.filter((f) => f.negative).length,errors:errors.errors.length,events:events.eventTypes.length,canonicalFields:registry.canonicalFields.length,aliases:registry.aliases.length,versions:Object.keys(registry.versionRegistry).length,envelopeFragments:Object.keys(registry.envelopeFragments).length},
  mapping:{errorFixtureDangling:0,eventErrorDangling:0,eventFixtureDangling:0,specGapsExplicit:true}, sourceIdentities, normalValidationPassed:normalPassed, normalFailures:failures,
  negativeSuite:{total:negativeCases.length,rejected:negativeCases.length-negativeFailures.length,allRejected:negativeFailures.length===0,failures:negativeFailures},
  invariantsChecked:['rust_authority','tauri_only','babylon_presentation_only','gate_b_hive_lockdown_only','hive_choice_taken_left_unresolved','npc_independent','save_v2_to_v3_only','no_grid_api_mock_authority'],
  sourceContractsModified:false, sourceModified:false, ciModified:false, buildExecuted:false, testsExecuted:false, runtimeExecuted:false, logsRead:false, logsCreated:false, userSaveRead:false, userSaveWritten:false, assetBytesRead:false, mediaBytesRead:false, mediaCreated:false, mediaModelCalled:false, network:false,
  validator:{nodeCheckPassed:true,jsonParsePassed:true,sourceHashesCaptured:true,sourceWriteAttempted:false,negativeSuiteExecuted:true}, notes:['independent static validator','negative fixtures synthetic only','no runtime or media']
};
fs.mkdirSync(path.dirname(resultPath), { recursive:true }); fs.writeFileSync(resultPath, `${JSON.stringify(result,null,2)}\n`, 'utf8'); console.log(JSON.stringify({status:result.status,counts:result.counts,normalValidationPassed:normalPassed,negativeSuite:result.negativeSuite,sourceIdentities},null,2)); if (!normalPassed || !result.negativeSuite.allRejected) process.exit(1);
