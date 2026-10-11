import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const planPath = path.join(root, 'plan.json');
const plan = JSON.parse(fs.readFileSync(planPath, 'utf8'));
const errors = [];
const reqFalse = ['architectureGateOpened','batchesApproved','implementationStarted','artifactContractAccepted','sourceModified','ciModified','buildExecuted','testsExecuted','packagedRuntimeExecuted','userSaveRead','userSaveWritten','assetBytesRead','mediaBytesRead','mediaCreated','mediaModelCalled','network'];
for (const k of reqFalse) if (plan[k] !== false) errors.push(`${k} must be false`);
if (plan.status !== 'f1_f7_minimal_implementation_batch_plan_ready_gate_blocked') errors.push('status mismatch');
if (plan.blockersClosed !== 0) errors.push('blockersClosed must be 0');
const allowed = new Set(['NB-55','NB-56','NB-57','NB-58','NB-59','NB-60','NB-61','NB-52','NB-53']);
for (const s of plan.sourceInventories ?? []) if (!allowed.has(s.sourcePackage)) errors.push(`unapproved source ${s.sourcePackage}`);
const ids = new Set();
for (const c of plan.candidateRegistry ?? []) { if (ids.has(c.candidateId)) errors.push(`duplicate candidate ${c.candidateId}`); ids.add(c.candidateId); if (!allowed.has(c.sourcePackage)) errors.push(`candidate source ${c.sourcePackage}`); if (c.approvalState !== 'not_approved') errors.push(`candidate approved ${c.candidateId}`); }
const batches = plan.batches ?? [];
if (batches.length > 7) errors.push('more than 7 batches');
if (batches.filter(b => b.recommendedFirst).length !== 1 || plan.currentDecision?.firstBatchId !== batches.find(b => b.recommendedFirst)?.batchId) errors.push('first batch recommendation is not unique');
for (const b of batches) { if (b.logicalUnitCount > 8) errors.push(`${b.batchId} exceeds 8 units`); if (b.logicalUnitCount !== b.logicalUnits?.length) errors.push(`${b.batchId} unit count mismatch`); if (b.approvalState !== 'not_approved') errors.push(`${b.batchId} approved`); }
const nodes = new Set(plan.dependencyDag?.nodes ?? []); const indegree = new Map([...nodes].map(n => [n,0]));
for (const [a,z] of plan.dependencyDag?.edges ?? []) { if (!nodes.has(a)||!nodes.has(z)) errors.push('DAG edge references unknown node'); else indegree.set(z, indegree.get(z)+1); }
const q = [...indegree].filter(([,d])=>d===0).map(([n])=>n); let seen=0; while(q.length){const n=q.shift();seen++; for(const [a,z] of plan.dependencyDag.edges) if(a===n && indegree.set(z,indegree.get(z)-1).get(z)===0) q.push(z);} if(seen!==nodes.size) errors.push('dependency DAG contains a cycle');
for (const l of plan.sharedFileLocks ?? []) if (l.mode !== 'serial') errors.push(`shared file not serial ${l.path}`);
const blockers = new Set(['E_LEGACY_GRID_REACHABLE','E_WORLD_V3_NOT_RUNTIME_WIRED','E_CONTINUOUS_PRIMITIVES_MISSING','E_API_FALLBACK_UNBACKED','E_NO_FULL_SAVE_V3_RUNTIME_RESTORE']);
if (plan.blockerClosureLedger?.length !== 5) errors.push('blocker ledger count');
for (const b of plan.blockerClosureLedger ?? []) { if (!blockers.has(b.id)||b.currentState!=='open'||!b.blocksFormalAcceptance||b.closedByThisPlan) errors.push(`invalid blocker ${b.id}`); }
if (!plan.artifactContractRules?.failClosed || plan.artifactContractRules.autoNormalizePaths || !plan.artifactContractRules.mismatchBlocksAcceptance) errors.push('artifact contract is not fail-closed');
for (const g of plan.productDecisionGates ?? []) if (g.status !== 'pending' || g.valuesSelected !== false) errors.push(`product decision prematurely selected ${g.id}`);
const forbidden = [['NB','-47'].join(''),['NB','-48'].join(''),['NB','-62'].join(''),['TP','-15C3'].join(''),['TP','-18D1'].join(''),['TP','-01R11'].join(''),['TP','-18D2'].join('')];
const raw = fs.readFileSync(planPath,'utf8'); for (const x of forbidden) if(raw.includes(x)) errors.push(`forbidden reference ${x}`);
const result = {status: errors.length ? 'fail' : 'pass', planStatus: plan.status, errorCount: errors.length, errors, batches: batches.length, candidateCount: plan.candidateRegistry?.length ?? 0, blockersOpen: plan.blockerClosureLedger?.filter(b=>b.currentState==='open').length ?? 0, architectureGateOpened: plan.architectureGateOpened, implementationStarted: plan.implementationStarted};
fs.writeFileSync(path.join(root,'..','results','f1-f7-minimal-implementation-batches-nb63.json'), JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result,null,2));
if (errors.length) process.exit(1);
