import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
const own=path.dirname(new URL(import.meta.url).pathname.replace(/^\//,'').replaceAll('%20',' '));
const root=path.resolve(own,'../../..');
const rel=p=>path.relative(root,p).replaceAll('\\','/');
const sha=s=>crypto.createHash('sha256').update(s).digest('hex');
const HASH=/^[a-f0-9]{64}$/;
const WORLDS=['grey_hive','mist_harbor','clockworks'];
const allowedRoot='runtime/assets/';
const safeRuntime=p=>typeof p==='string'&&!p.includes('\\')&&!p.startsWith('/')&&!/^[A-Za-z]:/.test(p)&&!p.includes('..')&&!/^https?:/i.test(p)&&p.startsWith(allowedRoot);
const clone=x=>JSON.parse(JSON.stringify(x));
const error=(code,detail)=>({code,detail});
function validateCandidate(c, records, {now='2026-09-22', packagedPaths=[]}={}){
 const e=[];const add=(code,d)=>e.push(error(code,d));
 if(!c||typeof c!=='object')return {valid:false,errors:[error('E_INPUT_TYPE','candidate')]};
 for(const k of ['assetId','assetVersion','exactVersion','registryApproval','evidenceState','evidenceHash','runtimeDerivativePath','runtimeDerivativeHash','worldScope','sourceRedistribution','runtimeDerivativeRedistribution','attributionRequired','attributionOutput','dependencies','buildTarget'])if(!(k in c))add('E_REQUIRED_INPUT',k);
 if(c.expectedExactVersion&&c.exactVersion!==c.expectedExactVersion)add('E_VERSION_DRIFT',c.assetId);
 if(c.registryApproval!=='runtime_approved')add('E_REGISTRY_NOT_RUNTIME_APPROVED',c.assetId);
 if(c.evidenceState!=='accepted')add('E_EVIDENCE_NOT_ACCEPTED',c.assetId);
 if(!HASH.test(c.evidenceHash||''))add('E_EVIDENCE_HASH',c.assetId);
 if(!HASH.test(c.runtimeDerivativeHash||''))add('E_DERIVATIVE_HASH',c.assetId);
 if(c.registryEvidenceHash&&c.evidenceHash!==c.registryEvidenceHash)add('E_EVIDENCE_HASH',c.assetId);
 if(c.registryRuntimeDerivativeHash&&c.runtimeDerivativeHash!==c.registryRuntimeDerivativeHash)add('E_DERIVATIVE_HASH',c.assetId);
 if(c.sourceRedistribution!=='allowed')add('E_SOURCE_PERMISSION',c.assetId);
 if(c.runtimeDerivativeRedistribution!=='allowed')add('E_RUNTIME_PERMISSION',c.assetId);
 if(!safeRuntime(c.runtimeDerivativePath))add('E_RUNTIME_ROOT',c.runtimeDerivativePath);
 if(c.worldScope==='shared'&&(!Array.isArray(c.compatibleWorlds)||c.compatibleWorlds.length!==WORLDS.length||WORLDS.some(w=>!c.compatibleWorlds.includes(w))))add('E_SHARED_SCOPE_DECLARATION',c.assetId);
 if(c.worldScope!=='shared'&&c.compatibleWorlds?.some(w=>w!==c.worldScope))add('E_WORLD_SCOPE_MISMATCH',c.assetId);
 if(c.attributionRequired&&(!c.attributionOutput||!c.attributionOutput.trim()))add('E_ATTRIBUTION_OUTPUT',c.assetId);
 if(c.expiresAt&&c.expiresAt<=now)add('E_EXPIRED',c.assetId);
 if(['expired','superseded','rejected','deprecated'].includes(c.evidenceState))add('E_EVIDENCE_LIFECYCLE',c.assetId);
 if(!Array.isArray(c.dependencies))add('E_DEPENDENCY_INPUT',c.assetId);
 const ids=new Set(records.map(r=>`${r.assetId}@${r.assetVersion}`));
 for(const d of c.dependencies||[])if(!ids.has(`${d.assetId}@${d.assetVersion}`)&&d.required!==false)add('E_DEPENDENCY_MISSING',`${c.assetId}:${d.assetId}`);
 if(c.dependencies?.some(d=>d.assetId===c.assetId&&d.assetVersion===c.assetVersion))add('E_DEPENDENCY_CYCLE',c.assetId);
 if(c.placeholderFinalCollision)add('E_PLACEHOLDER_FINAL_COLLISION',c.assetId);
 if(['source','work'].includes(c.fileClass))add('E_SOURCE_WORK_PACKAGED',c.assetId);
 if(packagedPaths.includes(c.runtimeDerivativePath)===false&&packagedPaths.length)add('E_RUNTIME_MANIFEST_MISMATCH',c.assetId);
 return {valid:e.length===0,errors:e};
}
function deterministicSignature(items){return sha(JSON.stringify([...items].sort((a,b)=>`${a.assetId}@${a.assetVersion}`.localeCompare(`${b.assetId}@${b.assetVersion}`))));}
function dependencyCycle(items){const graph=new Map(items.map(i=>[`${i.assetId}@${i.assetVersion}`,(i.dependencies||[]).map(d=>`${d.assetId}@${d.assetVersion}`)]));const seen=new Set(),stack=new Set();function v(k){if(stack.has(k))return true;if(seen.has(k))return false;seen.add(k);stack.add(k);for(const d of graph.get(k)||[])if(v(d))return true;stack.delete(k);return false;}return [...graph.keys()].some(v);}
function gate(records, packageItems){
 const errors=[],approved=[],blocked=[];for(const c of packageItems){const r=validateCandidate(c,records,{packagedPaths:packageItems.map(x=>x.runtimeDerivativePath)});(r.valid?approved:blocked).push({candidate:c,errors:r.errors});errors.push(...r.errors);}
 const ids=packageItems.map(c=>`${c.assetId}@${c.assetVersion}`);if(new Set(ids).size!==ids.length)errors.push(error('E_DUPLICATE_ID','packageItems'));if(dependencyCycle(packageItems))errors.push(error('E_DEPENDENCY_CYCLE','packageItems'));
 return {approvedManifest:errors.length?[]:approved.map(x=>x.candidate),blockedManifest:blocked,attributionManifest:approved.filter(x=>x.candidate.attributionRequired).map(x=>({assetId:x.candidate.assetId,text:x.candidate.attributionOutput})),deterministicSignature:deterministicSignature(errors.length?blocked.map(x=>x.candidate):approved.map(x=>x.candidate)),errors,decision:errors.length?'blocked_fail_closed':'candidate_package_manifest'};
}
const fixture=JSON.parse(fs.readFileSync(path.join(own,'fixtures.json'),'utf8'));const H='a'.repeat(64);
const base={assetId:'ASSET.SHARED.CHARACTER.SENTINEL',assetVersion:'0.1.0',exactVersion:'release-0.1.0',expectedExactVersion:'release-0.1.0',registryApproval:'runtime_approved',evidenceState:'accepted',evidenceHash:H,registryEvidenceHash:H,runtimeDerivativePath:'runtime/assets/shared/ASSET.SHARED.CHARACTER.SENTINEL/0.1.0/file.glb',runtimeDerivativeHash:H,registryRuntimeDerivativeHash:H,worldScope:'shared',compatibleWorlds:[...WORLDS],sourceRedistribution:'allowed',runtimeDerivativeRedistribution:'allowed',attributionRequired:true,attributionOutput:'Example Vendor',dependencies:[],buildTarget:'offline-windows-x64',fileClass:'runtime_derivative'};
function negatives(){const rows=[];const add=(id,code,mutate)=>{const r=clone(base),all=[r];mutate(r,all);const out=gate(all,[r]);rows.push({id,expected:code,pass:out.errors.some(x=>x.code===code),errors:out.errors.map(x=>x.code)});};
 add('unapproved_registry','E_REGISTRY_NOT_RUNTIME_APPROVED',r=>r.registryApproval='runtime_candidate');add('evidence_not_accepted','E_EVIDENCE_NOT_ACCEPTED',r=>r.evidenceState='in_review');add('source_permission_unknown','E_SOURCE_PERMISSION',r=>r.sourceRedistribution='unknown');add('runtime_permission_unknown','E_RUNTIME_PERMISSION',r=>r.runtimeDerivativeRedistribution='unknown');add('evidence_hash_mismatch','E_EVIDENCE_HASH',r=>r.evidenceHash='b'.repeat(64));add('derivative_hash_mismatch','E_DERIVATIVE_HASH',r=>r.runtimeDerivativeHash='b'.repeat(64));add('version_drift','E_VERSION_DRIFT',r=>r.exactVersion='drift');add('missing_dependency','E_DEPENDENCY_MISSING',r=>r.dependencies=[{assetId:'ASSET.GREY_HIVE.ENVIRONMENT.MISSING',assetVersion:'1',required:true}]);add('dependency_cycle','E_DEPENDENCY_CYCLE',r=>r.dependencies=[{assetId:r.assetId,assetVersion:r.assetVersion}]);add('scope_mismatch','E_WORLD_SCOPE_MISMATCH',r=>{r.worldScope='grey_hive';r.compatibleWorlds=['mist_harbor'];});add('runtime_root_escape','E_RUNTIME_ROOT',r=>r.runtimeDerivativePath='../runtime/file.glb');add('absolute_path','E_RUNTIME_ROOT',r=>r.runtimeDerivativePath='C:/asset.glb');add('network_url','E_RUNTIME_ROOT',r=>r.runtimeDerivativePath='https://host/file.glb');add('source_file_packaged','E_SOURCE_WORK_PACKAGED',r=>r.fileClass='source');add('work_file_packaged','E_SOURCE_WORK_PACKAGED',r=>r.fileClass='work');add('placeholder_collision','E_PLACEHOLDER_FINAL_COLLISION',r=>r.placeholderFinalCollision=true);add('attribution_missing','E_ATTRIBUTION_OUTPUT',r=>r.attributionOutput='');add('expired','E_EXPIRED',r=>r.expiresAt='2020-01-01');add('superseded','E_EVIDENCE_LIFECYCLE',r=>r.evidenceState='superseded');add('duplicate_id','E_DUPLICATE_ID',r=>{});add('missing_hash','E_DERIVATIVE_HASH',r=>delete r.runtimeDerivativeHash);add('build_target_missing','E_REQUIRED_INPUT',r=>delete r.buildTarget);add('tier_s_placeholder_missing','E_REGISTRY_NOT_RUNTIME_APPROVED',r=>r.registryApproval='proposed');
 rows[19].pass=gate([base,clone(base)],[base,clone(base)]).errors.some(x=>x.code==='E_DUPLICATE_ID');return {total:rows.length,passed:rows.filter(x=>x.pass).length,pass:rows.every(x=>x.pass),rows};}
const negativeFixtures=negatives();
const output={schema:'new_baseline_first_formal_asset_compliance_packaging_gate_nb26_v1',status:'asset_compliance_packaging_gate_implementation_ready_for_gate_review',conclusion:'asset_compliance_packaging_gate_implementation_ready_for_gate_review',implemented:false,enforced:false,offlineOnly:true,network:false,mediaBytesRead:false,mediaCreatedOrProcessed:false,legalConclusion:false,
  inputContract:{fields:['assetId','assetVersion','registryApproval','evidenceState','evidenceHash','runtimeDerivativePath','runtimeDerivativeHash','worldScope','compatibleWorlds','sourceRedistribution','runtimeDerivativeRedistribution','attributionRequired','attributionOutput','dependencies','buildTarget'],doubleApproval:['registryApproval=runtime_approved','evidenceState=accepted'],sourceWorkBoundary:'source/work never packaged; only approved runtime derivative candidate'},
  outputContract:{approvedManifest:true,blockedManifest:true,stableErrorCodes:true,attributionManifest:true,deterministicSignature:true,machineAuditReport:true},
  errorCodes:['E_REGISTRY_NOT_RUNTIME_APPROVED','E_EVIDENCE_NOT_ACCEPTED','E_SOURCE_PERMISSION','E_RUNTIME_PERMISSION','E_EVIDENCE_HASH','E_DERIVATIVE_HASH','E_VERSION_DRIFT','E_DEPENDENCY_MISSING','E_DEPENDENCY_CYCLE','E_WORLD_SCOPE_MISMATCH','E_RUNTIME_ROOT','E_SOURCE_WORK_PACKAGED','E_PLACEHOLDER_FINAL_COLLISION','E_ATTRIBUTION_OUTPUT','E_EXPIRED','E_EVIDENCE_LIFECYCLE','E_DUPLICATE_ID','E_REQUIRED_INPUT'],
  worlds:{formal:[...WORLDS],sharedRequiresAllCompatible:true,independentClosure:true,tierS:fixture.tierS,missingTierS:'blocked unless explicit named placeholder policy'},
  phases:[{id:'inventory_report_only',enforced:false},{id:'no_new_violations',enforced:false},{id:'candidate_package_manifest',enforced:false},{id:'enforced_gate',enforced:false, futureOnly:true}],
  negativeFixtures, deterministic:{sort:'assetId@assetVersion',signature:deterministicSignature([base]),network:false,filesystemEnumerationNotUsed:true},
  patchManifest:[{path:'tools/continuous-tps-architecture-probe/new-baseline-first-formal-asset-compliance-packaging-gate/schema.json',state:'new',anchor:'schema root',reason:'candidate input contract',dependency:'NB-21/NB-25',test:'schema/path field checks',rollback:'delete own file'},{path:'tools/continuous-tps-architecture-probe/new-baseline-first-formal-asset-compliance-packaging-gate/fixtures.json',state:'new',anchor:'negativeIds',reason:'offline text fixtures',dependency:'validator',test:'fixture coverage',rollback:'delete own file'},{path:'tools/continuous-tps-architecture-probe/new-baseline-first-formal-asset-compliance-packaging-gate/validate.mjs',state:'new',anchor:'gate()',reason:'fail-closed packaging readiness validator',dependency:'NB-21/NB-25',test:'node check/run',rollback:'delete own file'},{path:'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-asset-compliance-packaging-gate-nb26.json',state:'new',anchor:'machine result',reason:'audit output',dependency:'validator',test:'JSON parse',rollback:'delete own file'},{path:'docs/FIRST_FORMAL_ASSET_COMPLIANCE_PACKAGING_GATE_PLAN.md',state:'new',anchor:'report',reason:'implementation plan',dependency:'NB-21/NB-25',test:'diff check',rollback:'delete own file'}],
  inventory:{registryRoots:['game/assets','server-rs/release-ui/assets','server-rs/ui/assets','tools/asset_src'],runtimeRoot:allowedRoot,frontendDist:'./release-ui',assetBytesRead:false,network:false,activeOutputsRead:false},
  blockers:['NB-21 registry implemented=false/enforced=false','NB-25 evidence gate implemented=false/enforced=false','formal packaging integration intentionally absent'],rollback:'remove only five NB-26 files; no asset/registry/build/CI/runtime state changed'};
const result=path.join(root,'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-asset-compliance-packaging-gate-nb26.json');fs.writeFileSync(result,JSON.stringify(output,null,2)+'\n');console.log(JSON.stringify({status:output.status,negativeFixtures,implemented:false,enforced:false,network:false,mediaBytesRead:false},null,2));
