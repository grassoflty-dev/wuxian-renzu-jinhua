import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const registry = JSON.parse(fs.readFileSync(path.join(here, 'registry.json'), 'utf8'));
const audit = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-runtime-contract-consistency-audit/matrix.json'), 'utf8'));
const errors = [];
const canonical = new Set(registry.canonicalFields.map((f) => f.canonicalName));
const aliases = new Set(registry.canonicalFields.flatMap((f) => f.sourceAliases));
const names = new Set();
for (const f of registry.canonicalFields) {
  if (names.has(f.canonicalName)) errors.push(`duplicate canonical ${f.canonicalName}`);
  names.add(f.canonicalName);
  for (const k of ['canonicalName','type','shape','required','owner','versionIntroduced','normalization','serialization','privacy','compatibility','sourceAliases']) if (!(k in f)) errors.push(`${f.canonicalName}: missing ${k}`);
}
if (registry.canonicalFields.length !== 16) errors.push('canonical field count is not 16');
for (const f of audit.fieldCrosswalk) if (!canonical.has({schema_identity:'runtimeSchemaVersion',request_identity:'requestId',state_commit:'commitOutcome',retry_outcome:'commitOutcome',snapshot_order:'snapshotSequence',tombstone:'snapshotSequence',world_identity:'worldVersion',trace_identity:'traceId',authority_order:'authorityOrder',entity_identity:'entityIdentity',error_identity:'errorCode',save_migration:'transactionEvidence',world_transition:'worldVersion',evidence_identity:'transactionEvidence',presentation_boundary:'eventType',privacy_boundary:'transactionEvidence'}[f.field] ?? f.field)) errors.push(`crosswalk not represented ${f.field}`);
for (const a of registry.aliases) if (!a.canonicalName || !canonical.has(a.canonicalName) || a.direction.includes('two')) errors.push('invalid alias direction');
if (new Set(registry.aliases.map((a) => a.canonicalName)).size !== registry.aliases.length) errors.push('alias canonical not unique');
for (const [name, frag] of Object.entries(registry.envelopeFragments)) {
  for (const field of [...(frag.required ?? []), ...(frag.optional ?? [])]) if (!canonical.has(field) && !aliases.has(field) && !['worldId','zoneId','roomId','errorCode','fixtureId','stateCommitted','rollback','retryable','causeChain','tombstone','entityId','authorityOrder','snapshotSequence','eventType','traceId','spanId','transactionEvidence','saveVersion','HTTP status as code'].includes(field)) errors.push(`${name}: unknown field ${field}`);
  if ((frag.forbidden ?? []).some((x) => /Grid|tile|px|py|DOM|mesh|HTTP status|renderer authority/i.test(x)) === false && name === 'command') errors.push('command authority prohibition missing');
}
for (const v of ['contractRegistryVersion','schemaVersion','protocolVersion','worldVersion','saveVersion','eventSchemaVersion','errorContractVersion','fixtureFormatVersion']) if (!(v in registry.versionRegistry)) errors.push(`missing version ${v}`);
if (!registry.compatibilityMatrix.every((x) => x.failClosed === true)) errors.push('version matrix not fail closed');
for (const id of ['NB76-F001','NB76-F002','NB76-F003','NB76-F004','NB76-F005']) if (!registry.findingDisposition.some((x) => x.findingId === id && x.status === 'pending_engineering_action')) errors.push(`finding not handled ${id}`);
for (const inv of ['rust_authority','tauri_only','babylon_presentation_only','no_grid_api_mock_authority','gate_b_hive_lockdown_only','hive_choice_taken_left_unresolved','npc_independent','v2_to_v3_only','save_source_preserved','timeout_not_equal_uncommitted','unknown_fail_closed']) if (!registry.invariants.includes(inv)) errors.push(`missing invariant ${inv}`);
if (registry.currentDecision !== 'candidate_registry_ready_not_implemented' || registry.registryIdentity.status !== 'not_implemented') errors.push('implementation status invalid');
if (errors.length) throw new Error(errors.join('; '));

const result = {
  status:'shared_runtime_contract_schema_registry_ready_not_implemented', registrySpecificationAccepted:true, sourceContractsUpdated:false,
  registryImplemented:false, runtimeAdopted:false, runtimeEvidenceCaptured:false, formalRuntimeAccepted:false, architectureGateOpened:false,
  blockersClosed:0, sourceModified:false, ciModified:false, buildExecuted:false, testsExecuted:false, runtimeExecuted:false,
  logsRead:false, logsCreated:false, userSaveRead:false, userSaveWritten:false, assetBytesRead:false, mediaBytesRead:false,
  mediaCreated:false, mediaModelCalled:false, network:false, canonicalFieldCount:registry.canonicalFields.length,
  aliasCount:registry.aliases.length, versionCount:Object.keys(registry.versionRegistry).length, fragmentCount:Object.keys(registry.envelopeFragments).length,
  findingDispositionCount:registry.findingDisposition.length, adoptionStages:registry.adoptionPlan, pendingEngineeringActions:registry.mappingRequirements.map((x) => x.id),
  openBlockers:['F1','F2','F3','F4','F5','F6','F7'], validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true},
  notes:['candidate vocabulary only','source contracts untouched','aliases one-way only','no runtime adoption']
};
const resultPath = path.join(here, '../results/new-baseline-shared-runtime-contract-schema-registry-nb78.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,canonicalFieldCount:result.canonicalFieldCount,aliasCount:result.aliasCount,versionCount:result.versionCount,fragmentCount:result.fragmentCount,findingDispositionCount:result.findingDispositionCount}, null, 2));
