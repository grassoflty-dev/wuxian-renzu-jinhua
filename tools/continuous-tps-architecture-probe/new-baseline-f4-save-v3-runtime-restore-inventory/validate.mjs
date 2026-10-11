import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'inventory.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-f4-save-v3-runtime-restore-inventory-nb58.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const repo = path.resolve(here, '..', '..', '..');
const errors = [];
const flags = {status:'f4_save_v3_runtime_restore_inventory_complete_gate_blocked',E_NO_FULL_SAVE_V3_RUNTIME_RESTORE:'open',saveV3RuntimeWired:false,fullRestoreImplemented:false,migrationImplemented:false,migrationExecuted:false,userSaveRead:false,userSaveWritten:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const ids = new Set();
for (const item of [...(data.saveEntries ?? []), ...(data.candidateEntries ?? []), ...(data.migrationFindings ?? []), ...(data.clientSaveAuthorityFindings ?? [])]) {
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  for (const k of ['id','file','symbol']) if (!(k in item)) errors.push(`${item.id ?? 'unknown'}:missing_${k}`);
  const filePath = path.join(repo, item.file);
  if (!fs.existsSync(filePath)) errors.push(`${item.id}:file_missing`); else if (!fs.readFileSync(filePath,'utf8').includes(item.symbol)) errors.push(`${item.id}:anchor_missing`);
  if (item.integration === true || /wired|implemented|executed/i.test(item.classification ?? '')) errors.push(`${item.id}:candidate_promoted`);
}
if ((data.restoreDomains ?? []).length < 13) errors.push('restore_domains:min13');
if ((data.restoreStages ?? []).length !== 9) errors.push('restore_stages:exact9');
for (const stage of ['schema_identity','campaign','target_manifest_version','spatial_topology','doors_triggers','entities_npc_enemies','inventory_capability','checkpoint_player_transform','presentation_snapshot']) if (!(data.restoreStages ?? []).some(x=>x.stage===stage)) errors.push(`stage:${stage}`);
for (const domain of ['campaign identity/version','target world/zone/room','World v3/spatial topology identity','doors/triggers/checkpoints','entities/enemies/NPC','player Transform/Velocity/health/status','inventory/capability','quest/choice/flags','hive_lockdown/Gate B','hive_choice three-state','NPC independent state','content/asset exactVersion/hash','authority tick/revision','pending transition/transaction recovery']) if (!(data.restoreDomains ?? []).some(x=>x.name===domain)) errors.push(`domain:${domain}`);
const m = data.migrationContract ?? {};
if (m.direction !== 'v2_to_v3_only' || m.repeated !== 'reject_or_idempotent_with_explicit_marker' || m.reverse !== 'v3_to_v2_rejected' || !/untouched/.test(m.original)) errors.push('migration_contract');
if (!/hive_lockdown|Gate B/.test(JSON.stringify(data)) || !/hive_choice/.test(JSON.stringify(data)) || !/NPC independent/.test(JSON.stringify(data))) errors.push('state_independence');
for (const f of ['F1 authority cutover','F2 World v3 ownership boundary','F3 continuous spatial primitives','F5 transition/IPC lifecycle','F6 command/snapshot transport','F7 compile/integration/packaged evidence']) if (!(data.dependencies?.required ?? []).includes(f)) errors.push(`dependency:${f}`);
if ((data.minimalPatchBoundary?.candidateCount ?? 99) > 8 && !data.minimalPatchBoundary?.requires_split) errors.push('patch:max8');
if (!data.minimalPatchBoundary?.approval || /approved|implemented/i.test(data.minimalPatchBoundary.approval)) errors.push('patch:approval');
if (/Grid runtime fallback enabled|formal Grid fallback enabled/i.test(JSON.stringify(data))) errors.push('grid_fallback');
for (const c of data.clientSaveAuthorityFindings ?? []) if (c.resolved === true) errors.push(`${c.id}:resolved`);
const result = {schema:'nb58.f4_save_v3_runtime_restore_inventory.result.v1',status:flags.status,E_NO_FULL_SAVE_V3_RUNTIME_RESTORE:'open',saveV3RuntimeWired:false,fullRestoreImplemented:false,migrationImplemented:false,migrationExecuted:false,userSaveRead:false,userSaveWritten:false,minimalPatchApproved:false,sourceModified:false,buildExecuted:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,saveEntries:data.saveEntries?.length ?? 0,candidateEntries:data.candidateEntries?.length ?? 0,schemaFields:data.currentSchemaFields?.length ?? 0,restoreDomains:data.restoreDomains?.length ?? 0,restoreStages:data.restoreStages?.length ?? 0,migrationFindings:data.migrationFindings?.length ?? 0,clientSaveAuthorityFindings:data.clientSaveAuthorityFindings?.length ?? 0,storageRisks:data.storageRisks?.length ?? 0,patchCandidates:data.minimalPatchBoundary?.candidateCount ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
