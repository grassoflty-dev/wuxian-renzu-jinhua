import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\/+/, '').replaceAll('/', path.sep));
const input = path.join(here, 'scenarios.json');
const resultPath = path.resolve(here, '..', 'results', 'new-baseline-babylon-continuous-world-performance-nb51.json');
const data = JSON.parse(fs.readFileSync(input, 'utf8'));
const errors = [];
const flags = {status:'babylon_continuous_world_performance_acceptance_ready_not_executed',performanceAccepted:false,stabilityAccepted:false,packagedRuntimeExecuted:false,profilesCaptured:false,screenshotsCaptured:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false};
for (const [k,v] of Object.entries(flags)) if (data[k] !== v) errors.push(`flag:${k}`);
const requiredPhases = ['correctness_preflight','startup_warmup','steady_traversal','stress','world_transition','save_reload','reconnect','long_soak','degraded_profile','device_lost','evidence_sealing'];
const ids = new Set(); const worlds = new Map(); const phases = new Set();
if (!Array.isArray(data.scenarios) || data.scenarios.length < 34) errors.push('scenarios:min34');
for (const s of data.scenarios ?? []) {
  for (const key of ['id','world','profile','phase','priority','preconditions','procedure','metrics','thresholdStatus','authorityInvariants','presentation','evidence','automation','status','deps']) if (!(key in s)) errors.push(`${s.id ?? 'unknown'}:missing_${key}`);
  if (ids.has(s.id)) errors.push(`duplicate:${s.id}`); ids.add(s.id);
  phases.add(s.phase); worlds.set(s.world, (worlds.get(s.world) ?? 0) + 1);
  if (!['P0','P1','P2'].includes(s.priority)) errors.push(`${s.id}:priority`);
  if (s.thresholdStatus !== 'pending_product_decision') errors.push(`${s.id}:threshold_not_pending`);
  if (!Array.isArray(s.metrics) || s.metrics.length < 2) errors.push(`${s.id}:metrics`);
  if (!Array.isArray(s.authorityInvariants) || s.authorityInvariants.length === 0) errors.push(`${s.id}:authority`);
  if (!s.evidence || !s.automation) errors.push(`${s.id}:evidence_automation`);
}
for (const phase of requiredPhases) if (!phases.has(phase)) errors.push(`phase:${phase}`);
for (const world of ['grey_hive','mist_harbor','clockworks']) if ((worlds.get(world) ?? 0) < 7) errors.push(`world:${world}:min7`);
if (!Array.isArray(data.negativeFixtures) || data.negativeFixtures.length < 14) errors.push('negative:min14');
for (const n of ['leak','duplicate','undisposed_resource','shader_storm','texture_thrash','snapshot_backlog','extrapolation_overrun','device_lost','context_restore','lod_pop','culling_hides_interaction','streaming_writes_authority','reload_duplicate_entity','three_fallback','side2d_fallback','grid_authority']) if (!data.negativeFixtures.includes(n)) errors.push(`negative:${n}`);
if (!/Rust/i.test(data.authorityContract?.owner ?? '') || !/babylon/i.test(data.authorityContract?.presentation ?? '') || !/never changes collider/i.test(data.authorityContract?.forbidden ?? '')) errors.push('authority_contract');
if (!/Grid, Side2D, Three\.js fallback/i.test(data.authorityContract?.forbidden ?? '')) errors.push('fallback_boundary');
const result = {schema:'nb51.babylon_continuous_world_performance.result.v1',status:flags.status,performanceAccepted:false,stabilityAccepted:false,packagedRuntimeExecuted:false,profilesCaptured:false,screenshotsCaptured:false,mediaBytesRead:false,mediaCreated:false,mediaModelCalled:false,network:false,scenarios:data.scenarios?.length ?? 0,uniqueIds:ids.size,negativeFixtures:data.negativeFixtures?.length ?? 0,worldCounts:Object.fromEntries(worlds),phases:[...phases],pendingThresholds:data.pendingThresholds?.length ?? 0,errors,passed:errors.length===0};
fs.mkdirSync(path.dirname(resultPath), {recursive:true});
fs.writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
if (errors.length) process.exitCode = 1;
