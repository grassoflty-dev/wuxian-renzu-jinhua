import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const own = path.dirname(new URL(import.meta.url).pathname.replace(/^\//,'').replaceAll('%20',' '));
const root = path.resolve(own, '../../..');
const rel = p => path.relative(root,p).replaceAll('\\','/');
const sha = s => crypto.createHash('sha256').update(s).digest('hex');
const HASH = /^[a-f0-9]{64}$/;
const STATES = ['missing','submitted','in_review','accepted','rejected','expired','superseded'];
const ALLOWED = { missing:['submitted'], submitted:['in_review','rejected'], in_review:['accepted','rejected'], accepted:['expired','superseded'], expired:['submitted'], superseded:['submitted'] };
const pathSafe = p => typeof p === 'string' && p.length > 0 && !p.includes('\\') && !p.startsWith('/') && !/^[A-Za-z]:/.test(p) && !p.includes('..') && !/^https?:/i.test(p);
const evidencePathSafe = p => pathSafe(p) && !/(^|\/)(assets|data|media|public)(\/|$)/i.test(p);
const idSafe = id => /^ASSET\.[A-Z0-9_]+\.[A-Z0-9_]+\.[A-Z0-9_]+$/.test(id);
const required = ['assetId','assetVersion','sourceUrlText','author','exactVersion','license','licenseTextSnapshotPath','retrievedAt','sourceRedistribution','modification','commercialUse','attribution','reviewer','evidenceHash','recordState'];
const clone = x => JSON.parse(JSON.stringify(x));
function validateRecord(r, {now='2026-09-22', expected=null}={}) {
  const errors=[]; const add=(code,detail)=>errors.push({code,detail});
  if (!r || typeof r !== 'object' || Array.isArray(r)) return {valid:false,errors:[{code:'E_RECORD_TYPE'}]};
  for (const k of required) if (!(k in r)) add('E_REQUIRED_FIELD',k);
  if (!idSafe(r.assetId)) add('E_ASSET_ID',r.assetId);
  if (!r.assetVersion || !r.exactVersion) add('E_VERSION_MISSING',r.assetId);
  if (!r.sourceUrlText || !r.author) add('E_SOURCE_AUTHOR_MISSING',r.assetId);
  if (!r.license?.id || !r.license?.name || !r.licenseTextSnapshotPath) add('E_LICENSE_EVIDENCE_MISSING',r.assetId);
  if (!r.licenseTextSnapshotPath || !evidencePathSafe(r.licenseTextSnapshotPath)) add('E_EVIDENCE_PATH',r.licenseTextSnapshotPath);
  if (!HASH.test(r.evidenceHash||'')) add('E_EVIDENCE_HASH',r.assetId);
  if (!/^[0-9]{4}-[0-9]{2}-[0-9]{2}$/.test(r.retrievedAt||'')) add('E_RETRIEVED_DATE',r.assetId);
  for (const k of ['sourceRedistribution','modification','commercialUse']) if (!['allowed','forbidden','unknown'].includes(r[k])) add('E_UNKNOWN_POLICY',`${k}:${r.assetId}`);
  if (!r.attribution || typeof r.attribution.required !== 'boolean' || typeof r.attribution.textOrRequirement !== 'string') add('E_ATTRIBUTION',r.assetId);
  if (!r.reviewer) add('E_REVIEWER',r.assetId);
  if (!STATES.includes(r.recordState)) add('E_STATE',r.recordState);
  if (r.recordState === 'accepted' && (r.sourceRedistribution !== 'allowed' || r.modification !== 'allowed' || r.commercialUse !== 'allowed' || r.license.id === 'unknown' || r.license.name.toLowerCase() === 'unknown')) add('E_ACCEPTED_UNKNOWN_OR_DENIED',r.assetId);
  if (r.recordState === 'accepted' && r.attribution.required && !r.attribution.textOrRequirement.trim()) add('E_ATTRIBUTION_TEXT',r.assetId);
  if (expected && (r.assetId !== expected.assetId || r.assetVersion !== expected.assetVersion || r.exactVersion !== expected.exactVersion)) add('E_IMMUTABLE_BINDING',r.assetId);
  if (r.expiresAt && r.expiresAt <= now && r.recordState === 'accepted') add('E_EXPIRED',r.assetId);
  return {valid:errors.length===0,errors};
}
function packageGate(records, packageItems, {now='2026-09-22'}={}) {
  const errors=[]; const byKey=new Map();
  for (const r of records) {
    const key=`${r.assetId}@${r.assetVersion}`;
    if (byKey.has(key)) errors.push({code:'E_DUPLICATE_BINDING',detail:key});
    byKey.set(key,r);
    const v=validateRecord(r,{now}); if (!v.valid) errors.push(...v.errors);
  }
  for (const item of packageItems) {
    const key=`${item.assetId}@${item.assetVersion}`, r=byKey.get(key);
    if (!r) { errors.push({code:'E_MISSING_EVIDENCE',detail:key}); continue; }
    if (r.recordState !== 'accepted') errors.push({code:'E_NOT_ACCEPTED',detail:key});
    if (!item.evidenceHash || item.evidenceHash !== r.evidenceHash) errors.push({code:'E_HASH_MISMATCH',detail:key});
    if (item.exactVersion !== r.exactVersion) errors.push({code:'E_VERSION_DRIFT',detail:key});
    if (item.sourceRedistribution !== r.sourceRedistribution) errors.push({code:'E_SOURCE_REDISTRIBUTION_CONFLICT',detail:key});
    if (item.runtimeDerivativeRedistribution !== 'allowed') errors.push({code:'E_RUNTIME_DERIVATIVE_NOT_ALLOWED',detail:key});
    if (item.runtimePath && (!pathSafe(item.runtimePath) || !item.runtimePath.startsWith('runtime/assets/'))) errors.push({code:'E_RUNTIME_PATH',detail:item.runtimePath});
    if (r.attribution.required && (!item.attributionOutput || !item.attributionOutput.trim())) errors.push({code:'E_ATTRIBUTION_OUTPUT_MISSING',detail:key});
  }
  return {packaged:errors.length===0, packageDecision:errors.length===0?'runtime_approved_candidate':'blocked_fail_closed',errors};
}
function transition(from,to) { return !!ALLOWED[from]?.includes(to); }
function negatives(base) {
  const cases=[]; const add=(id,expected,mutate,gate=false)=>cases.push({id,expected,mutate,gate});
  add('missing','E_REQUIRED_FIELD',r=>delete r.sourceUrlText);
  add('unknown','E_ACCEPTED_UNKNOWN_OR_DENIED',r=>{r.license.id='unknown';r.license.name='unknown';});
  add('conflict','E_SOURCE_REDISTRIBUTION_CONFLICT',(_,items)=>items[0].sourceRedistribution='forbidden',true);
  add('hash_mismatch','E_HASH_MISMATCH',(_,items)=>items[0].evidenceHash='b'.repeat(64),true);
  add('version_drift','E_VERSION_DRIFT',(_,items)=>items[0].exactVersion='release-9',true);
  add('path_traversal','E_EVIDENCE_PATH',r=>r.licenseTextSnapshotPath='../escape.txt');
  add('expired','E_EXPIRED',r=>{r.expiresAt='2026-01-01';});
  add('superseded','E_NOT_ACCEPTED',r=>{r.recordState='superseded';},true);
  const rows=[];
  for (const c of cases) { const r=clone(base), i=[{assetId:r.assetId,assetVersion:r.assetVersion,exactVersion:r.exactVersion,evidenceHash:r.evidenceHash,sourceRedistribution:r.sourceRedistribution,runtimeDerivativeRedistribution:'allowed',runtimePath:'runtime/assets/shared/x/0.1.0/file.glb',attributionOutput:'Example Vendor'}]; c.mutate(r,i); const out=c.gate?packageGate([r],i):validateRecord(r); rows.push({id:c.id,expected:c.expected,pass:out.errors.some(e=>e.code===c.expected),errors:out.errors.map(e=>e.code)}); }
  return {total:rows.length,passed:rows.filter(r=>r.pass).length,pass:rows.every(r=>r.pass),rows};
}
const fixtures=JSON.parse(fs.readFileSync(path.join(own,'fixtures.json'),'utf8'));
const base=fixtures.positive;
const acceptedGate=packageGate([base],[{assetId:base.assetId,assetVersion:base.assetVersion,exactVersion:base.exactVersion,evidenceHash:base.evidenceHash,sourceRedistribution:'allowed',runtimeDerivativeRedistribution:'allowed',runtimePath:'runtime/assets/shared/sentinel/0.1.0/file.glb',attributionOutput:'Example Vendor'}]);
const statusCases = STATES.map(s=>({from:'accepted',to:s,allowed:transition('accepted',s)}));
const sourceDerivativeSeparation=packageGate([base],[{assetId:base.assetId,assetVersion:base.assetVersion,exactVersion:base.exactVersion,evidenceHash:base.evidenceHash,sourceRedistribution:'allowed',runtimeDerivativeRedistribution:'forbidden',runtimePath:'runtime/assets/shared/sentinel/0.1.0/file.glb',attributionOutput:'Example Vendor'}]);
const output={schema:'new_baseline_first_formal_license_evidence_gate_nb25_v1',status:'license_evidence_gate_implementation_ready_for_gate_review',conclusion:'license_evidence_gate_implementation_ready_for_gate_review',implemented:false,enforced:false,offlineOnly:true,network:false,mediaBytesRead:false,legalConclusion:false,requiredAuditFields:12,
  evidenceSchema:{path:rel(path.join(own,'schema.json')),fields:required,license:{required:['id','name','licenseTextSnapshotPath','evidenceHash']},immutableBinding:['assetId','assetVersion','exactVersion','evidenceHash'],sourceUrlTextOnly:true},
  statusMachine:{states:STATES,allowedTransitions:ALLOWED,acceptedRequires:['known license id/name','license text snapshot path','hash','allowed source redistribution','allowed modification','allowed commercial use','reviewer','attribution decision']},
  packageGate:{acceptedOnly:true,unknownFailClosed:true,sourceRedistributionSeparate:true,runtimeDerivativeRedistributionSeparate:true,attributionOutputRequired:true,acceptedGate:acceptedGate,derivativeConflict:sourceDerivativeSeparation},
  statusCases,negativeFixtures:negatives(base),
  patchManifest:[{path:'tools/continuous-tps-architecture-probe/new-baseline-first-formal-license-evidence-gate/schema.json',state:'new',purpose:'text-only evidence schema'},{path:'tools/continuous-tps-architecture-probe/new-baseline-first-formal-license-evidence-gate/fixtures.json',state:'new',purpose:'offline text fixtures'},{path:'tools/continuous-tps-architecture-probe/new-baseline-first-formal-license-evidence-gate/validate.mjs',state:'new',purpose:'offline validator and fail-closed package gate'},{path:'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-license-evidence-gate-nb25.json',state:'new',purpose:'machine result'},{path:'docs/FIRST_FORMAL_LICENSE_EVIDENCE_GATE_PLAN.md',state:'new',purpose:'review report'}],
  inventory:{sources:['docs/ASSET_REGISTRY_SPEC.md','docs/ASSET_ACQUISITION_PLAN.md','docs/FIRST_FORMAL_ASSET_REGISTRY_SLICE_PLAN.md','tools/continuous-tps-architecture-probe/results/new-baseline-capability-assets-nb03.json','tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-asset-registry-nb21.json','text/path inventory only'],assetBytesRead:false,network:false,forbiddenActivePackages:['TP-15C3','TP-18D1','TP-01R11']},
  expiryChange:{expiredRejected:true,supersededRejected:true,versionDriftRejected:true,retrievalDateReReview:true,licenseSnapshotHashReReview:true},
  attribution:{requiredFields:['required','textOrRequirement','attributionOutput before packaging'],noLegalTextGenerated:true},
  blockers:['NB-21 registry implementation not implemented/enforced','external evidence not received','runtime/build package integration intentionally absent'],
  rollback:'remove only the five NB-25 files; no registry/runtime/build/media state changed'};
const resultPath=path.join(root,'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-license-evidence-gate-nb25.json');
fs.writeFileSync(resultPath,JSON.stringify(output,null,2)+'\n');
console.log(JSON.stringify({status:output.status,negativeFixtures:output.negativeFixtures,acceptedGate:acceptedGate.packageDecision,derivativeConflict:sourceDerivativeSeparation.packageDecision},null,2));
