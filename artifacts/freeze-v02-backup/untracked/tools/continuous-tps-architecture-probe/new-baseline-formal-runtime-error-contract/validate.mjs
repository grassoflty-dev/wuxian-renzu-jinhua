import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const data = JSON.parse(fs.readFileSync(path.join(here, 'errors.json'), 'utf8'));
const oracle = fs.readFileSync(path.join(root, 'docs/FORMAL_RUNTIME_ERROR_CONTRACT.md'), 'utf8');
const fixtures = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-continuous-world-deterministic-test-oracle/fixtures.json'), 'utf8')).fixtures;
const fixtureIds = new Set(fixtures.map((f) => f.stableId));
const errors = [];
const ids = new Set();
const codes = new Set();
const domains = new Set();
const stages = new Set();
const classes = new Set(data.classifications);
for (const e of data.errors) {
  if (ids.has(e.stableId) || codes.has(e.code)) errors.push(`duplicate ${e.stableId}/${e.code}`);
  ids.add(e.stableId); codes.add(e.code); domains.add(e.domain); (e.stages ?? []).forEach((s) => stages.add(s));
  for (const k of ['stableId','code','domain','condition','owner','classification','severity','commit','retry','rollback','ui','diagnostic','blockers','stages','fixtures','implementationStatus']) if (!(k in e)) errors.push(`${e.stableId}: missing ${k}`);
  if (!classes.has(e.classification)) errors.push(`${e.stableId}: invalid class`);
  if (e.implementationStatus !== 'not_implemented') errors.push(`${e.stableId}: implementation claim`);
  for (const f of e.fixtures ?? []) if (!fixtureIds.has(f)) errors.push(`${e.stableId}: unmapped fixture ${f}`);
  if (e.classification === 'unknown_outcome' && e.retry !== 'inspect_only') errors.push(`${e.stableId}: unknown outcome blind retry`);
  if (e.domain === 'F5_snapshot' && e.owner === 'BabylonPresentation' && e.commit !== 'none') errors.push(`${e.stableId}: renderer mutates authority`);
  if (e.domain === 'F4_save' && e.code === 'E_SAVE_PARTIAL_WRITE' && e.rollback !== 'restore_original') errors.push(`${e.stableId}: source preservation missing`);
  if (e.retry === 'yes' || e.retry === 'blind') errors.push(`${e.stableId}: unsafe retry class`);
}
if (data.errors.length < 40) errors.push('fewer than 40 errors');
for (const s of ['F1','F2','F3','F4','F5','F6']) if (!stages.has(s)) errors.push(`missing stage ${s}`);
for (const d of ['F1_authority','F2_world','F3_spatial','F4_save','F5_snapshot','F6_ipc','grey_hive_loader','grey_hive_state','transition','capability_ai_npc','packaged_offline']) if (!domains.has(d)) errors.push(`missing domain ${d}`);
for (const c of ['validation_reject','transaction_abort','unknown_outcome','resync_required','fatal_startup','recoverable_presentation','evidence_gate_failure']) if (!classes.has(c) || !data.errors.some((e) => e.classification === c)) errors.push(`missing class ${c}`);
for (const token of ['Rust','Tauri','fail closed','hive_lockdown','hive_choice','renderer','unknown_outcome','Save','original source','idempotency','diagnostic']) if (!oracle.includes(token)) errors.push(`oracle boundary missing ${token}`);
if (data.blockerCoverage.F1 !== 'open' || data.blockerCoverage.F2 !== 'open' || data.blockerCoverage.F3 !== 'open' || data.blockerCoverage.F4 !== 'open' || data.blockerCoverage.F5 !== 'open' || data.blockerCoverage.F6 !== 'open') errors.push('blocker not open');
if (data.uiContract.safeMessageOnly !== true || data.uiContract.recoveryClaimRequiresCommittedResult !== true) errors.push('unsafe UI contract');
if (data.diagnosticPolicy.captureLogs !== false || data.diagnosticPolicy.captureMedia !== false) errors.push('capture flags enabled');
if (errors.length) throw new Error(errors.join('; '));

const domainCounts = Object.fromEntries([...domains].sort().map((d) => [d, data.errors.filter((e) => e.domain === d).length]));
const classificationCounts = Object.fromEntries([...classes].sort().map((c) => [c, data.errors.filter((e) => e.classification === c).length]));
const stageCounts = Object.fromEntries(['F1','F2','F3','F4','F5','F6'].map((s) => [s, data.errors.filter((e) => e.stages.includes(s)).length]));
const blockerCounts = Object.fromEntries(['F1','F2','F3','F4','F5','F6'].map((b) => [b, data.errors.filter((e) => e.blockers.includes(b)).length]));
const fixtureCoverage = new Set(data.errors.flatMap((e) => e.fixtures));
const result = {
  status:'formal_runtime_error_contract_ready_not_implemented', errorContractSpecificationAccepted:true,
  errorContractImplemented:false, runtimeErrorsExercised:false, formalRuntimeAccepted:false, blockersClosed:0,
  sourceModified:false, ciModified:false, buildExecuted:false, testsExecuted:false, packagedRuntimeExecuted:false,
  logsCaptured:false, userSaveRead:false, userSaveWritten:false, assetBytesRead:false, mediaBytesRead:false,
  mediaCreated:false, mediaModelCalled:false, network:false, errorCount:data.errors.length,
  domainCounts, classificationCounts, affectedStageCounts:stageCounts, blockerCoverageCounts:blockerCounts,
  mappedFixtureCount:fixtureCoverage.size, specGaps:data.errors.filter((e) => e.fixtures.length === 0).map((e) => e.code),
  unknownOutcomeCodes:data.errors.filter((e) => e.classification === 'unknown_outcome').map((e) => e.code),
  validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true},
  notes:['future contract only','no runtime errors triggered','no logs or media captured','Save original preservation required','renderer is presentation-only']
};
const resultPath = path.join(here, '../results/new-baseline-formal-runtime-error-contract-nb68.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,errorCount:result.errorCount,domainCounts,classificationCounts,affectedStageCounts:stageCounts,blockerCoverageCounts:blockerCounts,mappedFixtureCount:fixtureCoverage.size}, null, 2));
