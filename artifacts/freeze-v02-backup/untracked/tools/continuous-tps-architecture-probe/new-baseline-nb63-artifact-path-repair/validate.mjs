import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const base=path.resolve(root,'..','..','..');
const repair=JSON.parse(fs.readFileSync(path.join(root,'repair.json'),'utf8'));
const errors=[];
const o=path.join(base,repair.originalPath), c=path.join(base,repair.canonicalPath);
if(!fs.existsSync(o)||!fs.existsSync(c)) errors.push('both result paths must exist');
if(repair.canonicalPath!==repair.pathLiteralExpected||!repair.canonicalPath.endsWith('new-baseline-f1-f7-minimal-implementation-batches-nb63.json')) errors.push('canonical literal mismatch');
if(fs.existsSync(o)&&fs.existsSync(c)){
 const ob=fs.readFileSync(o), cb=fs.readFileSync(c); const oh=crypto.createHash('sha256').update(ob).digest('hex').toUpperCase(), ch=crypto.createHash('sha256').update(cb).digest('hex').toUpperCase();
 if(oh!==ch||ob.length!==cb.length||!ob.equals(cb)) errors.push('canonical differs from original');
 for(const [label,b] of [['original',ob],['canonical',cb]]){try{const j=JSON.parse(b.toString('utf8')); if(j.status!=='pass'||j.planStatus!=='f1_f7_minimal_implementation_batch_plan_ready_gate_blocked'||j.batches!==7||j.candidateCount!==7||j.blockersOpen!==5||j.architectureGateOpened!==false||j.implementationStarted!==false) errors.push(`${label} key fields mismatch`)}catch{errors.push(`${label} JSON parse failed`)}}
}
if(!repair.originalPreserved||repair.operation!=='additive_canonical_copy'||repair.currentDecision.nb63PathMismatchRemediated!==true) errors.push('repair metadata mismatch');
if(repair.remainingMismatches.nb49!=='unresolved'||repair.remainingMismatches.nb50!=='unresolved'||repair.remainingMismatches.artifactContractAccepted!==false||repair.remainingMismatches.architectureGate!=='closed') errors.push('remaining mismatch state changed');
if(repair.currentDecision.artifactContractAccepted!==false||repair.currentDecision.architectureGateOpened!==false||repair.currentDecision.b0Approved!==false||repair.currentDecision.implementationStarted!==false||repair.currentDecision.blockersClosed!==0) errors.push('gate state changed');
const f=repair.flags; for(const k of ['historicalPlanModified','sourceModified','ciModified','buildExecuted','testsExecuted','runtimeExecuted','userSaveRead','userSaveWritten','assetBytesRead','mediaBytesRead','mediaCreated','mediaModelCalled','network']) if(f[k]!==false) errors.push(`${k} must be false`);
const forbidden=[['NB','-47'].join(''),['NB','-64'].join(''),['NB','-66'].join(''),['TP','-15C3'].join(''),['TP','-18D1'].join(''),['TP','-01R11'].join(''),['TP','-18D2'].join('')]; const raw=fs.readFileSync(path.join(root,'repair.json'),'utf8'); for(const s of forbidden) if(raw.includes(s)) errors.push(`forbidden literal ${s}`);
const result={status:errors.length?'fail':'pass',repairStatus:repair.currentDecision.status,errorCount:errors.length,errors,nb63PathMismatchRemediated:repair.currentDecision.nb63PathMismatchRemediated,repairMode:repair.operation,originalPreserved:repair.originalPreserved,byteIdentical:repair.byteIdentical,canonicalPathVerified:errors.length===0,nb49MismatchResolved:false,nb50MismatchResolved:false,artifactContractAccepted:false,architectureGateOpened:false,b0Approved:false,implementationStarted:false,blockersClosed:0,flags:f};
fs.writeFileSync(path.join(base,'tools/continuous-tps-architecture-probe/results/new-baseline-nb63-artifact-path-repair-nb67.json'),JSON.stringify(result,null,2)+'\n'); console.log(JSON.stringify(result,null,2)); if(errors.length) process.exit(1);
