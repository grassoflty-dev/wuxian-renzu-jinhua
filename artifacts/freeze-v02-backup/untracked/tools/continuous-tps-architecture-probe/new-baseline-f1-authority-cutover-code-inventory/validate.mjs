import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'inventory.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-f1-authority-cutover-code-inventory-nb55.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const repo = path.resolve(here, '..', '..', '..');
const errors = [];
const flags = {status:'f1_authority_cutover_code_inventory_complete_gate_blocked',E_LEGACY_GRID_REACHABLE:'open',authorityCutoverImplemented:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const ids = new Set(); const all = [...(data.authorityEntries ?? []), ...(data.gridFindings ?? []), ...(data.clientAuthorityFindings ?? [])];
const reach = new Set(['compile_time_dead','test_dev_only','candidate_shadow_read_only','formal_runtime_reachable','unknown_not_enough_evidence']);
for (const item of all) {
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  for (const key of ['id','file','symbol','line','category','reachability','evidence','severity','formalBlocker','proposedDisposition']) if (!(key in item)) errors.push(`${item.id ?? 'unknown'}:missing_${key}`);
  if (!reach.has(item.reachability)) errors.push(`${item.id}:reachability`);
  if (item.reachability === 'formal_runtime_reachable' || item.reachability === 'unknown_not_enough_evidence') if (item.resolved === true || /resolved|implemented|approved/i.test(item.status ?? '')) errors.push(`${item.id}:reachable_resolved`);
  const filePath = path.join(repo, item.file);
  if (!fs.existsSync(filePath)) errors.push(`${item.id}:file_missing`);
  else {
    const text = fs.readFileSync(filePath, 'utf8');
    if (!text.includes(item.symbol)) errors.push(`${item.id}:symbol_anchor_missing`);
  }
}
for (const ref of data.scanMetadata?.sourceRefs ?? []) if (!fs.existsSync(path.join(repo, ref))) errors.push(`source_missing:${ref}`);
if (!data.minimalPatchBoundary?.requires_split && (data.minimalPatchBoundary?.candidateCount ?? 99) > 8) errors.push('minimal_patch:max8');
if (data.minimalPatchBoundary?.requires_split && !data.minimalPatchBoundary?.splitOrder) errors.push('requires_split:missing_order');
if (!data.minimalPatchBoundary?.approval || /approved|implemented/i.test(data.minimalPatchBoundary.approval)) errors.push('minimal_patch:approval');
if ((data.authorityEntries ?? []).length < 15) errors.push('authority_entries:min15');
if ((data.gridFindings ?? []).length < 10) errors.push('grid_findings:min10');
if ((data.clientAuthorityFindings ?? []).length < 5) errors.push('client_findings:min5');
for (const token of [['TP','-18D2'].join(''),['TP','-15C3'].join(''),['TP','-18D1'].join(''),['TP','-01R11'].join('')]) if (fs.readFileSync(input,'utf8').includes(token)) errors.push(`forbidden_token:${token}`);
const counts = Object.fromEntries([...reach].map(x=>[x,all.filter(i=>i.reachability===x).length]));
const result = {schema:'nb55.f1_authority_cutover_inventory.result.v1',status:flags.status,E_LEGACY_GRID_REACHABLE:'open',authorityCutoverImplemented:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,authorityEntries:data.authorityEntries?.length ?? 0,gridFindings:data.gridFindings?.length ?? 0,gridReachability:counts,clientAuthorityFindings:data.clientAuthorityFindings?.length ?? 0,ownershipDomains:data.ownershipTable?.length ?? 0,patchCandidates:data.minimalPatchBoundary?.candidateCount ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
