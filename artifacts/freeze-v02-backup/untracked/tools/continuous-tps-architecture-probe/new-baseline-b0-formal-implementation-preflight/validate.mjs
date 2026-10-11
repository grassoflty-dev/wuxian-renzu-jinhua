import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import crypto from 'node:crypto';
const root = path.dirname(fileURLToPath(import.meta.url));
const plan = JSON.parse(fs.readFileSync(path.join(root, 'preflight.json'), 'utf8'));
const base = path.resolve(root, '..', '..', '..');
const errors = [];
const upstream = JSON.parse(fs.readFileSync(path.join(base, 'tools/continuous-tps-architecture-probe/new-baseline-f1-f7-minimal-implementation-batches/plan.json'), 'utf8'));
const b0 = upstream.batches.find(b => b.batchId === 'B0');
if (!b0 || plan.b0Definition.batchId !== 'B0') errors.push('B0 missing');
if (plan.logicalUnits.length !== 8 || plan.b0Definition.logicalUnitCount !== 8) errors.push('B0 must contain exactly 8 logical units');
if (JSON.stringify(plan.logicalUnits.map(x=>x.name)) !== JSON.stringify(b0.logicalUnits)) errors.push('logical units differ from NB-63');
for (const k of ['name','entry','exit','rollback','approvalState']) if (plan.b0Definition[k] !== b0[k]) errors.push(`B0 ${k} differs from NB-63`);
if (!plan.b0Definition.recommendedFirst || b0.recommendedFirst !== true) errors.push('B0 is not the unique recommendation');
for (const u of plan.logicalUnits) {
  const p = path.join(base, u.fileOrLogicalUnit.split('::')[0]);
  if (!fs.existsSync(p)) errors.push(`missing anchor file ${u.fileOrLogicalUnit}`);
  else { const raw=fs.readFileSync(p,'utf8'); if (!raw.includes(u.anchor)) errors.push(`missing anchor ${u.anchor}`); const hash=crypto.createHash('sha256').update(raw).digest('hex').toUpperCase(); if(hash!==u.fingerprint.textFileHash) errors.push(`fingerprint drift ${u.fileOrLogicalUnit}`); }
  if (!u.fingerprint?.algorithm || !u.fingerprint?.textFileHash) errors.push(`fingerprint incomplete ${u.id}`);
}
if (!plan.sourceAnchors.verified || !plan.sourceAnchors.allFilesExist || plan.sourceAnchors.driftDetected) errors.push('source anchors not verified');
if (!plan.sharedFileLocks.allSerial || plan.conflictMatrix.b0InternalOverlap || plan.conflictMatrix.activeWriterOverlap) errors.push('shared or active conflict');
if (plan.productDecisionCheck.productDecisionBlocked || !plan.productDecisionCheck.allEightUnitsValueFree) errors.push('product decision selected or blocking');
if (plan.sourcePlanIdentity.artifactPathMismatch.resolved !== false || !plan.sourcePlanIdentity.artifactPathMismatch.blocking) errors.push('NB-63 path mismatch was not preserved as blocking');
if (plan.conflictMatrix.unverifiedPriorPackage.confirmed || plan.conflictMatrix.unverifiedPriorPackage.usedAsEvidence) errors.push('unverified prior package treated as confirmed');
if (plan.currentDecision.b0DefinitionMatched !== true || plan.currentDecision.sourceAnchorsVerified !== true || plan.currentDecision.activeWriteConflict !== false || plan.currentDecision.productDecisionBlocked !== false) errors.push('decision flags mismatch');
if (plan.currentDecision.conclusion.includes('ready_for_implementation')) errors.push('invalid implementation-ready conclusion');
const f=plan.flags; for (const k of ['architectureGateOpened','b0Approved','implementationStarted','sourceModified','ciModified','buildExecuted','testsExecuted','packagedRuntimeExecuted','userSaveRead','userSaveWritten','assetBytesRead','mediaBytesRead','mediaCreated','mediaModelCalled','network']) if(f[k]!==false) errors.push(`${k} must be false`); if(f.blockersClosed!==0) errors.push('blockersClosed must be zero');
const raw = fs.readFileSync(path.join(root,'preflight.json'),'utf8');
for (const s of [['NB','-47'].join(''),['NB','-62'].join(''),['NB','-64'].join(''),['TP','-15C3'].join(''),['TP','-18D1'].join(''),['TP','-01R11'].join(''),['TP','-18D2'].join('')]) if(raw.includes(s)) errors.push(`forbidden literal ${s}`);
const result={status:errors.length?'fail':'pass',preflightStatus:plan.status,errorCount:errors.length,errors,b0DefinitionMatched:plan.currentDecision.b0DefinitionMatched,sourceAnchorsVerified:plan.currentDecision.sourceAnchorsVerified,activeWriteConflict:plan.currentDecision.activeWriteConflict,productDecisionBlocked:plan.currentDecision.productDecisionBlocked,nb63ArtifactPathMismatchResolved:plan.sourcePlanIdentity.artifactPathMismatch.resolved,architectureGateOpened:f.architectureGateOpened,b0Approved:f.b0Approved,implementationStarted:f.implementationStarted,blockersClosed:f.blockersClosed,flags:f};
fs.writeFileSync(path.join(base,'tools/continuous-tps-architecture-probe/results/new-baseline-b0-formal-implementation-preflight-nb65.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result,null,2)); if(errors.length) process.exit(1);
