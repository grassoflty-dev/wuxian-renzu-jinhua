import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'inventory.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-f3-continuous-spatial-primitives-inventory-nb57.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const repo = path.resolve(here, '..', '..', '..');
const errors = [];
const flags = {status:'f3_continuous_spatial_primitives_inventory_complete_gate_blocked',E_CONTINUOUS_PRIMITIVES_MISSING:'open',continuousPrimitivesImplemented:false,runtimeIntegrated:false,greyHiveSpatialLoaded:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const ids = new Set();
for (const item of [...(data.spatialEntries ?? []), ...(data.legacySpatialFindings ?? []), ...(data.clientSpatialAuthorityFindings ?? [])]) {
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  for (const key of ['id','file','symbol','severity']) if (!(key in item)) errors.push(`${item.id ?? 'unknown'}:missing_${key}`);
  if (!('reachability' in item) && !('classification' in item)) errors.push(`${item.id ?? 'unknown'}:missing_reachability`);
  const filePath = path.join(repo, item.file);
  if (!fs.existsSync(filePath)) errors.push(`${item.id}:file_missing`); else if (!fs.readFileSync(filePath,'utf8').includes(item.symbol)) errors.push(`${item.id}:anchor_missing`);
}
const primitives = new Set((data.primitiveContracts ?? []).map(x=>x.primitive));
for (const p of ['Vec3','Transform','Velocity','Collider','Trigger','Room','Zone','Door','Spatial Query','Nav','Checkpoint']) if (!primitives.has(p)) errors.push(`primitive:${p}`);
for (const c of data.candidateEntries ?? []) if (c.integration !== false || /formal|integrated|loaded/i.test(c.classification ?? '')) errors.push(`${c.id}:candidate_promoted`);
for (const c of data.clientSpatialAuthorityFindings ?? []) if (c.resolved === true) errors.push(`${c.id}:resolved`);
if ((data.minimalPatchBoundary?.candidateCount ?? 99) > 8 && !data.minimalPatchBoundary?.requires_split) errors.push('patch:max8');
if (!data.minimalPatchBoundary?.approval || /approved|implemented/i.test(data.minimalPatchBoundary.approval)) errors.push('patch:approval');
for (const f of ['F1 authority cutover','F2 World v3 ownership boundary','F4 Save v3 persistence','F5 snapshot/transition lifecycle','F6 command and snapshot transport','F7 compile/integration/packaged evidence']) if (!(data.dependencies?.required ?? []).includes(f)) errors.push(`dependency:${f}`);
if (/Grid fallback enabled|formal fallback enabled/i.test(JSON.stringify(data))) errors.push('grid_fallback');
const result = {schema:'nb57.f3_continuous_spatial_primitives_inventory.result.v1',status:flags.status,E_CONTINUOUS_PRIMITIVES_MISSING:'open',continuousPrimitivesImplemented:false,runtimeIntegrated:false,greyHiveSpatialLoaded:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,spatialEntries:data.spatialEntries?.length ?? 0,candidateEntries:data.candidateEntries?.length ?? 0,legacySpatialFindings:data.legacySpatialFindings?.length ?? 0,clientSpatialAuthorityFindings:data.clientSpatialAuthorityFindings?.length ?? 0,primitiveContracts:data.primitiveContracts?.length ?? 0,patchCandidates:data.minimalPatchBoundary?.candidateCount ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
