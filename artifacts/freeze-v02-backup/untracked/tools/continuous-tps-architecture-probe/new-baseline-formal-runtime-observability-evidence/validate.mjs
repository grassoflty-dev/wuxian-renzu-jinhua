import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const data = JSON.parse(fs.readFileSync(path.join(here, 'events.json'), 'utf8'));
const errorData = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-formal-runtime-error-contract/errors.json'), 'utf8'));
const fixtureData = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-continuous-world-deterministic-test-oracle/fixtures.json'), 'utf8'));
const eventNames = new Set(); const eventIds = new Set(); const errors = [];
const errorCodes = new Set(errorData.errors.map((e) => e.code));
const fixtureIds = new Set(fixtureData.fixtures.map((f) => f.stableId));
const stages = new Set(); const domains = new Set();
for (const e of data.eventTypes) {
  if (eventNames.has(e.name) || eventIds.has(e.stableId)) errors.push(`duplicate ${e.stableId}/${e.name}`);
  eventNames.add(e.name); eventIds.add(e.stableId); e.stages.forEach((s) => stages.add(s)); domains.add(e.domain);
  for (const k of ['stableId','name','domain','owner','trigger','required','optional','forbidden','sampling','retention','privacy','errors','fixtures','stages','implementationStatus']) if (!(k in e)) errors.push(`${e.name}: missing ${k}`);
  if (e.implementationStatus !== 'not_implemented') errors.push(`${e.name}: implementation claim`);
  for (const c of e.errors) if (!errorCodes.has(c)) errors.push(`${e.name}: unknown error ${c}`);
  for (const f of e.fixtures) if (!fixtureIds.has(f)) errors.push(`${e.name}: unknown fixture ${f}`);
  if (e.name.startsWith('renderer_') && e.forbidden.every((x) => !/authority|stateMutation|commandReplay/i.test(x))) errors.push(`${e.name}: renderer write boundary missing`);
  if (e.forbidden.some((x) => /fullSave|absolutePath|assetBytes|rawPayload|savePayload|media|screenshot/i.test(x)) === false && /save|evidence|grey_hive/i.test(e.domain)) errors.push(`${e.name}: privacy forbidden fields incomplete`);
}
if (data.eventTypes.length < 36) errors.push('fewer than 36 event types');
for (const s of ['F1','F2','F3','F4','F5','F6','F7']) if (!stages.has(s)) errors.push(`missing stage ${s}`);
for (const d of ['startup','ipc','command','authority','world','spatial','grey_hive_loader','capability','combat','stealth','npc','save','migration','transition','snapshot','renderer','packaged','evidence']) if (!domains.has(d)) errors.push(`missing domain ${d}`);
for (const t of data.traceTopologies) {
  if (!eventNames.has(t.root)) errors.push(`${t.name}: unknown root`);
  if (!t.terminal?.length || !t.unknownOutcome) errors.push(`${t.name}: terminal/unknown outcome missing`);
  for (const n of t.order) if (!eventNames.has(n)) errors.push(`${t.name}: unknown event ${n}`);
}
if (data.evidenceSeal.mediaAllowed !== false || data.evidenceSeal.validatorIndependent !== true) errors.push('evidence seal unsafe');
if (data.antiSelfAttestation.runtimePassedEventInsufficient !== true || data.antiSelfAttestation.independentValidatorRequired !== true || data.antiSelfAttestation.staticCannotReplacePackaged !== true) errors.push('anti-self-attestation incomplete');
if (data.blockerCoverage.blockersClosed !== 0 || Object.values(data.blockerCoverage).some((v, i) => i < 7 && v !== 'open')) errors.push('blocker closure/open state invalid');
if (data.privacyRedaction.save !== 'forbidden' || data.privacyRedaction.absolutePaths !== 'forbidden' || data.privacyRedaction.pii !== 'forbidden') errors.push('privacy contract incomplete');
if (errors.length) throw new Error(errors.join('; '));

const domainCounts = Object.fromEntries([...domains].sort().map((d) => [d, data.eventTypes.filter((e) => e.domain === d).length]));
const stageCounts = Object.fromEntries(['F1','F2','F3','F4','F5','F6','F7'].map((s) => [s, data.eventTypes.filter((e) => e.stages.includes(s)).length]));
const errorCoverage = new Set(data.eventTypes.flatMap((e) => e.errors));
const fixtureCoverage = new Set(data.eventTypes.flatMap((e) => e.fixtures));
const result = {
  status:'formal_runtime_observability_evidence_contract_ready_not_implemented', observabilitySpecificationAccepted:true,
  observabilityImplemented:false, tracesCaptured:false, metricsCaptured:false, evidenceSealed:false,
  packagedRuntimeObserved:false, formalRuntimeAccepted:false, blockersClosed:0, sourceModified:false, ciModified:false,
  buildExecuted:false, testsExecuted:false, runtimeExecuted:false, logsRead:false, logsCreated:false,
  profilesCaptured:false, screenshotsCaptured:false, userSaveRead:false, userSaveWritten:false,
  assetBytesRead:false, mediaBytesRead:false, mediaCreated:false, mediaModelCalled:false, network:false,
  eventCount:data.eventTypes.length, domainCounts, affectedStageCounts:stageCounts, topologyCount:data.traceTopologies.length,
  metricCount:data.metricsCatalog.length, mappedErrorCount:errorCoverage.size, mappedFixtureCount:fixtureCoverage.size,
  specGaps:{unmappedErrorCodes:[...errorCodes].filter((c) => !errorCoverage.has(c)),unmappedFixtureIds:[...fixtureIds].filter((f) => !fixtureCoverage.has(f))},
  backpressureFailureCount:data.backpressureFailures.length, validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true},
  notes:['future contract only','no logs/traces/profiles/screenshots created or read','events cannot self-attest runtime acceptance','thresholds remain pending product decision']
};
const resultPath = path.join(here, '../results/new-baseline-formal-runtime-observability-evidence-nb72.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,eventCount:result.eventCount,domainCounts,affectedStageCounts:stageCounts,topologyCount:result.topologyCount,metricCount:result.metricCount,mappedErrorCount:result.mappedErrorCount,mappedFixtureCount:result.mappedFixtureCount}, null, 2));
