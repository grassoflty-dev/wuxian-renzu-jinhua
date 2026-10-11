import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'inventory.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-f7-ci-packaged-gate-inventory-nb61.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const repo = path.resolve(here, '..', '..', '..');
const errors = [];
const flags = {status:'f7_ci_packaged_gate_inventory_complete_gate_blocked',formalCiGateWired:false,packagedRuntimeGateWired:false,packagedRuntimeVerified:false,artifactContractAccepted:false,formalRuntimeAccepted:false,minimalPatchApproved:false,sourceModified:false,ciModified:false,buildExecuted:false,testsExecuted:false,packagedRuntimeExecuted:false,network:false,assetBytesRead:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const ids = new Set();
for (const list of ['buildEntries','testEntries']) for (const item of data[list] ?? []) {
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  for (const k of ['id','file','anchor']) if (!(k in item)) errors.push(`${item.id ?? 'unknown'}:missing_${k}`);
  const filePath = path.join(repo, item.file);
  if (!fs.existsSync(filePath)) errors.push(`${item.id}:file_missing`); else if (!fs.readFileSync(filePath,'utf8').includes(item.anchor)) errors.push(`${item.id}:anchor_missing`);
}
const gates = new Set((data.gateMatrix ?? []).map(x=>x.gate));
for (const g of ['F1','F2','F3','F4','F5','F6','F7']) if (!gates.has(g)) errors.push(`gate:${g}`);
for (const blocker of data.formalBlockerGate ?? []) if (blocker.status !== 'open' || blocker.blocksFormalAcceptance !== true) errors.push(`blocker:${blocker.id}`);
if ((data.falsePositiveFindings ?? []).some(x=>x.resolved === true)) errors.push('false_positive_resolved');
if (!data.artifactContract?.accepted === false || data.artifactContract?.accepted !== false) errors.push('artifact_contract_acceptance');
if (!(data.falsePositiveFindings ?? []).some(x=>x.kind === 'output_path_mismatch' && x.resolved === false)) errors.push('path_mismatch');
if ((data.minimalPatchBoundary?.candidateCount ?? 99) > 8 && !data.minimalPatchBoundary?.requires_split) errors.push('patch:max8');
if (!data.minimalPatchBoundary?.approval || /approved|implemented/i.test(data.minimalPatchBoundary.approval)) errors.push('patch:approval');
if ((data.workflowEntries ?? []).length !== 0) errors.push('workflow_entries_unexpected');
for (const token of [['NB','-48'].join(''),['NB','-59'].join(''),['TP','-18D2'].join(''),['formal-capability','-source-build'].join('')]) if (fs.readFileSync(input,'utf8').includes(token)) errors.push(`forbidden:${token}`);
const result = {schema:'nb61.f7_ci_packaged_gate_inventory.result.v1',status:flags.status,formalCiGateWired:false,packagedRuntimeGateWired:false,packagedRuntimeVerified:false,artifactContractAccepted:false,formalRuntimeAccepted:false,minimalPatchApproved:false,sourceModified:false,ciModified:false,buildExecuted:false,testsExecuted:false,packagedRuntimeExecuted:false,network:false,assetBytesRead:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,buildEntries:data.buildEntries?.length ?? 0,workflowEntries:data.workflowEntries?.length ?? 0,testEntries:data.testEntries?.length ?? 0,gateMatrixUnits:data.gateMatrix?.length ?? 0,falsePositiveFindings:data.falsePositiveFindings?.length ?? 0,artifactFields:data.artifactContract?.fields?.length ?? 0,packagedGateStages:data.packagedGateStages?.length ?? 0,formalBlockers:data.formalBlockerGate?.length ?? 0,patchCandidates:data.minimalPatchBoundary?.candidateCount ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
