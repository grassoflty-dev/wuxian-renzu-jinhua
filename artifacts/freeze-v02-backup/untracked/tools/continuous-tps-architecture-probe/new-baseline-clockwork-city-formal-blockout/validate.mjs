import fs from 'node:fs';
import path from 'node:path';

const root = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const matrixPath = path.join(root, 'matrix.json');
const resultPath = path.resolve(root, '..', 'results', 'new-baseline-clockwork-city-formal-blockout-nb46.json');
const matrix = JSON.parse(fs.readFileSync(matrixPath, 'utf8'));
const errors = [];
const requiredFlags = {status:'clockwork_city_blockout_requirements_ready_not_implemented',formalPlayableAccepted:false,runtimeExecuted:false,worldDataCreated:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [key, value] of Object.entries(requiredFlags)) if (matrix[key] !== value) errors.push(`flag:${key}`);
if (!Array.isArray(matrix.items) || matrix.items.length < 22) errors.push('items:min_22');
const ids = new Set();
const priorities = new Set(['P0','P1','P2']);
const confidences = new Set(['high','medium','low']);
for (const item of matrix.items ?? []) {
  for (const field of ['id','source','scope','priority','confidence','conflictResolution','preconditions','expected']) if (!(field in item)) errors.push(`${item.id ?? 'unknown'}:missing_${field}`);
  if (ids.has(item.id)) errors.push(`duplicate:${item.id}`); ids.add(item.id);
  if (!priorities.has(item.priority)) errors.push(`${item.id}:priority`);
  if (!confidences.has(item.confidence)) errors.push(`${item.id}:confidence`);
  for (const field of ['topology','authority','persistence','evidence','automation','status','blockingDeps']) if (!(field in (item.expected ?? {}))) errors.push(`${item.id}:expected_${field}`);
  if (item.scope === 'dynamic') {
    if (item.expected.authority !== 'rust_authority') errors.push(`${item.id}:dynamic_authority`);
    if (!/fixed-step|replay|drift|pause|reload|recovery|collision/i.test(item.expected.automation)) errors.push(`${item.id}:dynamic_automation`);
    if (!/pending|proposal/i.test(item.expected.status)) errors.push(`${item.id}:dynamic_status`);
  }
}
if (/Grid|Side2D|Three\.js/.test(JSON.stringify(matrix.formalPath))) errors.push('formal_path_forbidden_abstraction');
if (matrix.negativeFixtures?.length < 9) errors.push('negative_fixtures:min_9');
const forbiddenSelf = [['TP','-18D2'].join(''), ['formal-capability','-source-build'].join('')];
for (const token of forbiddenSelf) if (fs.readFileSync(matrixPath, 'utf8').includes(token)) errors.push(`forbidden_self:${token}`);
const result = {schema:'nb46.clockwork_city_formal_blockout.result.v1',status:requiredFlags.status,formalPlayableAccepted:false,runtimeExecuted:false,worldDataCreated:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,items:matrix.items?.length ?? 0,uniqueIds:ids.size,negativeFixtures:matrix.negativeFixtures?.length ?? 0,pendingItems:(matrix.items ?? []).filter(x=>/pending|proposal/.test(x.expected?.status ?? '')).length,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
