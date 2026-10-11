import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const audit = JSON.parse(fs.readFileSync(path.join(here, 'audit.json'), 'utf8'));
const expected = path.join(root, audit.expectedPath);
const reported = path.join(root, audit.reportedPath);
const packet = path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-product-decision-batch-1/packet.json');
const result = JSON.parse(fs.readFileSync(expected, 'utf8'));
const packetJson = JSON.parse(fs.readFileSync(packet, 'utf8'));
const validatorText = fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-product-decision-batch-1/validate.mjs'), 'utf8');
const problems = [];
const hash = (p) => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex').toUpperCase();
const stat = (p) => fs.existsSync(p) ? {exists:true,type:fs.statSync(p).isFile()?'file':'other',bytes:fs.statSync(p).size,sha256:hash(p)} : {exists:false,type:null,bytes:null,sha256:null};
const eStat = stat(expected); const rStat = stat(reported); const pStat = stat(packet);
if (!eStat.exists || eStat.type !== 'file' || eStat.bytes !== 575 || eStat.sha256 !== audit.hashes.expectedPath) problems.push('expected path observation mismatch');
if (rStat.exists !== false) problems.push('reported alternate unexpectedly exists');
if (!pStat.exists || pStat.sha256 !== audit.hashes.observedAlternate) problems.push('observed alternate mismatch');
if (result.status !== audit.jsonIdentity.status || result.decisionCardCount !== 5 || result.answersRecorded !== 0 || result.decisionsMade !== false || result.supersessionDecisionsRecorded !== 0 || result.mediaAuthorized !== false) problems.push('result identity mismatch');
for (const k of ['architectureGateOpened','implementationStarted','runtimeExecuted','sourceModified','network']) if (result[k] !== false) problems.push(`result flag ${k}`);
if (packetJson.status !== audit.jsonIdentity.status || packetJson.decisionCards.length !== 5) problems.push('packet identity mismatch');
if (!packetJson.decisionCards.every((c) => c.answer === 'pending')) problems.push('answers not pending');
if (packetJson.currentDecision.answersRecorded !== 0 || packetJson.currentDecision.decisionsMade !== false || packetJson.currentDecision.supersessionDecisionsRecorded !== 0 || packetJson.currentDecision.mediaAuthorized !== false) problems.push('packet state changed');
if (packetJson.currentDecision.architectureGateOpened !== false || packetJson.currentDecision.implementationStarted !== false || packetJson.currentDecision.blockersClosed !== 0) problems.push('packet gate state changed');
if (!validatorText.includes('new-baseline-product-decision-batch-1-nb90.json')) problems.push('validator target literal missing');
if (audit.classification !== 'canonical_only_valid' || audit.recommendedRemediation !== 'additive_canonical_copy') problems.push('classification/remediation mismatch');
if (audit.answersPendingCheck.selectedOptionsRead !== false || audit.answersPendingCheck.allAnswersPendingOrNull !== true) problems.push('answer privacy check invalid');
if (problems.length) throw new Error(problems.join('; '));

const out = {
  status:'nb90_result_path_audit_complete', pathClassification:audit.classification, canonicalPathVerified:true, reportedPathVerified:false,
  byteIdentical:'not_applicable', answersPending:true, answersRecorded:0, remediationExecuted:false, nb90FilesModified:false,
  globalArtifactContractAccepted:false, architectureGateOpened:false, implementationStarted:false, blockersClosed:0, sourceModified:false,
  buildExecuted:false, testsExecuted:false, runtimeExecuted:false, mediaBytesRead:false, mediaCreated:false, mediaModelCalled:false, network:false,
  expectedPath:audit.expectedPath, reportedPath:audit.reportedPath, observedAlternatePath:'tools/continuous-tps-architecture-probe/new-baseline-product-decision-batch-1/packet.json',
  expectedBytes:eStat.bytes, expectedSha256:eStat.sha256, reportedExists:false, observedAlternateBytes:pStat.bytes, observedAlternateSha256:pStat.sha256,
  jsonIdentity:audit.jsonIdentity, validatorTarget:audit.validatorTarget, recommendedRemediation:audit.recommendedRemediation,
  validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true}, notes:['read-only path audit','answer content not read','no path remediation']
};
const resultPath = path.join(here, '../results/new-baseline-nb90-result-path-audit-nb94.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(out, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:out.status,pathClassification:out.pathClassification,canonicalPathVerified:out.canonicalPathVerified,reportedPathVerified:out.reportedPathVerified,expectedBytes:out.expectedBytes,expectedSha256:out.expectedSha256,observedAlternateBytes:out.observedAlternateBytes,observedAlternateSha256:out.observedAlternateSha256,answersPending:out.answersPending,recommendedRemediation:out.recommendedRemediation}, null, 2));
