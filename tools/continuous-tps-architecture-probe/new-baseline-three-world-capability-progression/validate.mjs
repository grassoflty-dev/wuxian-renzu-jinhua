import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const matrixPath = path.join(here, 'matrix.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-three-world-capability-progression-nb49.json');
const matrix = JSON.parse(fs.readFileSync(matrixPath, 'utf8'));
const errors = [];
const flags = {status:'three_world_capability_progression_acceptance_ready_gate_blocked',formalCapabilityAccepted:false,runtimeIntegrated:false,saveIntegrated:false,characterAssetsReady:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [k,v] of Object.entries(flags)) if (matrix[k] !== v) errors.push(`flag:${k}`);
if (!Array.isArray(matrix.items) || matrix.items.length < 30) errors.push('items:min30');
if (!Array.isArray(matrix.negativeFixtures) || matrix.negativeFixtures.length < 12) errors.push('negative:min12');
const ids = new Set(); const worlds = new Set(); const characters = new Set();
for (const item of matrix.items ?? []) {
  for (const key of ['id','source','world','character','scope','priority','preconditions','action','authority','persistence','presentation','evidence','automation','status','deps']) if (!(key in item)) errors.push(`${item.id ?? 'unknown'}:missing_${key}`);
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  if (!['P0','P1','P2'].includes(item.priority)) errors.push(`${item.id}:priority`);
  for (const world of String(item.world).split('|')) if (['grey_hive','mist_harbor','clockworks'].includes(world)) worlds.add(world);
  for (const character of String(item.character).split('|')) if (['Cen Yao','Bai Zhi','Sentinel'].includes(character)) characters.add(character);
  if (!/rust/i.test(item.authority)) errors.push(`${item.id}:authority`);
  if (!item.evidence || !item.automation) errors.push(`${item.id}:evidence_automation`);
}
for (const world of ['grey_hive','mist_harbor','clockworks']) if (!worlds.has(world)) errors.push(`coverage:world:${world}`);
for (const character of ['Cen Yao','Bai Zhi','Sentinel']) if (!characters.has(character)) errors.push(`coverage:character:${character}`);
if (matrix.layers?.definition !== 'versioned capability definition and prerequisites' || matrix.layers?.authoritativeRuntimeState !== 'Rust owns grants, costs, cooldowns, effects, progression and persistence' || matrix.layers?.presentationState !== 'Babylon/UI receive immutable view and submit intent only') errors.push('three_layers');
const order = matrix.transaction?.order ?? [];
if (order.join('>') !== 'precondition>authority_grant>inventory_capability_commit>save_v3_checkpoint>snapshot_ui_ack') errors.push('transaction_order');
if (matrix.worldGates?.gateB !== 'hive_lockdown only') errors.push('gate_b_binding');
if (!/never Gate B/.test(matrix.worldGates?.hiveChoice ?? '')) errors.push('choice_independence');
if (matrix.worldGates?.npc !== 'independent from Gate B and choice') errors.push('npc_independence');
if (matrix.sources?.find(x=>x.id==='nb27_catalog')?.confidence !== 'low') errors.push('nb27_confidence');
if (!/unverified/i.test(JSON.stringify(matrix))) errors.push('nb27_unverified_marker');
const requiredNeg = ['client_self_grant','unknown_id','version_incompatible','duplicate_grant','illegal_equip','cooldown_rollback','negative_charges','cross_character','npc_choice_mix','gate_b_wrong_binding','save_missing_field','world_bypass'];
for (const n of requiredNeg) if (!matrix.negativeFixtures.includes(n)) errors.push(`negative:${n}`);
const result = {schema:'nb49.three_world_capability_progression.result.v1',status:flags.status,formalCapabilityAccepted:false,runtimeIntegrated:false,saveIntegrated:false,characterAssetsReady:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,items:matrix.items?.length ?? 0,uniqueIds:ids.size,negativeFixtures:matrix.negativeFixtures?.length ?? 0,worldCoverage:[...worlds],characterCoverage:[...characters],committedItems:(matrix.items ?? []).filter(x=>x.status==='committed_boundary').length,pendingItems:(matrix.items ?? []).filter(x=>/pending/.test(x.status)).length,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
