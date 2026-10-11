import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'decisions.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-product-decision-register-nb53.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const errors = [];
const flags = {status:'product_decision_register_ready_awaiting_user_decisions',decisionsMade:false,architectureGateOpened:false,mediaAuthorized:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const ids = new Set(); const cats = new Map(); const confirmed = new Set(data.sourcePolicy?.confirmedPackages ?? []);
const excluded = new Set(data.sourcePolicy?.excludedPackages ?? []);
for (const d of data.decisions ?? []) {
  for (const k of ['decisionId','title','sourcePackages','sourceDesignRefs','whyUserDecision','affectedWorlds','systems','blockingLevel','deadlineGate','options','recommendedOption','defaultIfDeferred','userDecision','downstreamPackages']) if (!(k in d)) errors.push(`${d.decisionId ?? 'unknown'}:missing_${k}`);
  if (ids.has(d.decisionId)) errors.push(`duplicate:${d.decisionId}`); ids.add(d.decisionId);
  if (d.userDecision !== 'pending') errors.push(`${d.decisionId}:not_pending`);
  if (!Array.isArray(d.options) || d.options.length < 2 || new Set(d.options).size !== d.options.length) errors.push(`${d.decisionId}:options_not_mutually_exclusive`);
  if (!d.defaultIfDeferred || /choose|select|use recommended/i.test(d.defaultIfDeferred)) errors.push(`${d.decisionId}:unsafe_defer`);
  for (const pkg of d.sourcePackages ?? []) {
    if (excluded.has(pkg)) errors.push(`${d.decisionId}:excluded_source:${pkg}`);
    if (!confirmed.has(pkg) && pkg !== 'NB-43') errors.push(`${d.decisionId}:unconfirmed_source:${pkg}`);
  }
  cats.set(d.blockingLevel, (cats.get(d.blockingLevel) ?? 0) + 1);
}
if (!Array.isArray(data.firstBatch) || data.firstBatch.length > 5) errors.push('firstBatch:max5');
for (const id of data.firstBatch ?? []) if (!ids.has(id)) errors.push(`firstBatch:unknown:${id}`);
if ((data.decisions ?? []).filter(d=>d.sourcePackages?.includes('NB-43')).length !== 1) errors.push('media:must_be_single_reference');
const media = data.decisions?.find(d=>d.decisionId === 'DEC-MEDIA-01');
if (!media || media.userDecision !== 'pending' || !/not authorize|不授权|不生成|不读取/i.test(media.defaultIfDeferred)) errors.push('media:no_authorization');
for (const eng of data.engineeringResolvable ?? []) if (data.decisions?.some(d=>d.title === eng.decision)) errors.push(`engineering_promoted:${eng.id}`);
const result = {schema:'product_decision_register.result.v1',status:flags.status,decisionsMade:false,architectureGateOpened:false,mediaAuthorized:false,runtimeExecuted:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,totalDecisions:data.decisions?.length ?? 0,categoryCounts:Object.fromEntries(cats),engineeringResolvable:data.engineeringResolvable?.length ?? 0,mergedDuplicateCount:0,firstBatchCount:data.firstBatch?.length ?? 0,mediaReferenceCount:data.decisions?.filter(d=>d.sourcePackages?.includes('NB-43')).length ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
