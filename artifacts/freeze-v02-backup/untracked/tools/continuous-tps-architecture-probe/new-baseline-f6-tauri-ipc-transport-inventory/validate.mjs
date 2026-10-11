import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'inventory.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-f6-tauri-ipc-transport-inventory-nb60.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const repo = path.resolve(here, '..', '..', '..');
const errors = [];
const flags = {status:'f6_tauri_ipc_transport_inventory_complete_gate_blocked',E_API_FALLBACK_UNBACKED:'open',tauriIpcFormalTransportWired:false,apiFallbackRemoved:false,commandEnvelopeImplemented:false,packagedRuntimeVerified:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,network:false,assetBytesRead:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const ids = new Set();
for (const list of ['bootstrapEntries','commandEntries','bridgeEntries','eventSnapshotEntries','fallbackFindings','clientAuthorityFindings']) for (const item of data[list] ?? []) {
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  for (const k of ['id','file','symbol']) if (!(k in item)) errors.push(`${item.id ?? 'unknown'}:missing_${k}`);
  const filePath = path.join(repo, item.file);
  if (!fs.existsSync(filePath)) errors.push(`${item.id}:file_missing`); else if (!fs.readFileSync(filePath,'utf8').includes(item.symbol)) errors.push(`${item.id}:anchor_missing`);
  if ((item.resolved === true) || /removed|wired|implemented|verified|approved/i.test(item.status ?? '')) errors.push(`${item.id}:resolved`);
}
for (const f of data.fallbackFindings ?? []) if (f.resolved !== false) errors.push(`${f.id}:fallback_resolved`);
for (const c of data.clientAuthorityFindings ?? []) if (c.resolved !== false) errors.push(`${c.id}:client_resolved`);
const envelope = new Set((data.envelopeContract?.fields ?? []).map(x=>x.name));
for (const field of ['protocol/schemaVersion','requestId','client/session identity','command type/payload','expected authority revision','result/error code','committed revision/tick','snapshot sequence full/delta','timeout/cancel/retry/idempotency key','reconnect/resync','capability/compatibility negotiation']) if (!envelope.has(field)) errors.push(`envelope:${field}`);
for (const f of ['F1 authority cutover','F2 World v3 runtime wiring','F3 continuous spatial primitives','F4 Save v3 runtime restore','F5 transition/IPC lifecycle','F7 compile/integration/packaged evidence']) if (!(data.dependencies?.required ?? []).includes(f)) errors.push(`dependency:${f}`);
if ((data.minimalPatchBoundary?.candidateCount ?? 99) > 8 && !data.minimalPatchBoundary?.requires_split) errors.push('patch:max8');
if (!data.minimalPatchBoundary?.approval || /approved|implemented/i.test(data.minimalPatchBoundary.approval)) errors.push('patch:approval');
for (const token of [['NB','-48'].join(''),['NB','-59'].join(''),['TP','-18D2'].join(''),['formal-capability','-source-build'].join('')]) if (fs.readFileSync(input,'utf8').includes(token)) errors.push(`forbidden:${token}`);
const result = {schema:'nb60.f6_tauri_ipc_transport_inventory.result.v1',status:flags.status,E_API_FALLBACK_UNBACKED:'open',tauriIpcFormalTransportWired:false,apiFallbackRemoved:false,commandEnvelopeImplemented:false,packagedRuntimeVerified:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,network:false,assetBytesRead:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,bootstrapEntries:data.bootstrapEntries?.length ?? 0,commandEntries:data.commandEntries?.length ?? 0,bridgeEntries:data.bridgeEntries?.length ?? 0,eventSnapshotEntries:data.eventSnapshotEntries?.length ?? 0,fallbackFindings:data.fallbackFindings?.length ?? 0,clientAuthorityFindings:data.clientAuthorityFindings?.length ?? 0,envelopeFields:data.envelopeContract?.fields?.length ?? 0,lifecycleStages:data.lifecycle?.length ?? 0,orderingRisks:data.orderingRisks?.length ?? 0,patchCandidates:data.minimalPatchBoundary?.candidateCount ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
