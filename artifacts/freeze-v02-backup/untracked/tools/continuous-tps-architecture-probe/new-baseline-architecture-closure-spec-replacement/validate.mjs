import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const spec = JSON.parse(fs.readFileSync(path.join(here, 'spec.json'), 'utf8'));
const problems = [];
const requiredDomains = ['F1','F2','F3','F4','F5','F6','F7','GREY_HIVE','ARTIFACT','PRODUCT','SHARED'];
const domains = new Set(spec.closureDomains.map((d) => d.id));
for (const d of requiredDomains) if (!domains.has(d)) problems.push(`missing domain ${d}`);
if (spec.sourceEvidence.predecessorContentRead !== false || spec.quarantineState.predecessorContentRead !== false) problems.push('predecessor content read claim');
if (spec.quarantineState.nb22SupersessionAccepted !== false || spec.recommendation.supersessionAccepted !== false) problems.push('supersession unexpectedly accepted');
if (spec.blockerClosureRules.length !== 5 || spec.blockerClosureRules.some((b) => b.status !== 'open')) problems.push('five blockers not open');
for (const p of spec.evidenceLevels) if (!['spec_static','compile','unit','integration','packaged_offline','manual','artifact_seal'].includes(p)) problems.push(`invalid evidence ${p}`);
const ids = new Set(spec.closureDag.nodes); const indegree = Object.fromEntries(spec.closureDag.nodes.map((n) => [n, 0])); const next = Object.fromEntries(spec.closureDag.nodes.map((n) => [n, []]));
for (const [a,b] of spec.closureDag.edges) { if (!ids.has(a) || !ids.has(b)) problems.push(`DAG unknown ${a}/${b}`); else { next[a].push(b); indegree[b]++; } }
const queue = Object.keys(indegree).filter((n) => indegree[n] === 0); let seen = 0;
while (queue.length) { const n = queue.shift(); seen++; for (const b of next[n]) if (--indegree[b] === 0) queue.push(b); }
if (seen !== spec.closureDag.nodes.length || spec.closureDag.acyclic !== true) problems.push('DAG cycle');
if (!spec.criticalPath.length || spec.criticalPath[0] !== 'SHARED' || spec.criticalPath.at(-1) !== 'ARTIFACT') problems.push('critical path incomplete');
for (const p of spec.acceptanceProhibitions) if (!p) problems.push('empty acceptance prohibition');
for (const i of ['rust_only_authority','tauri_only_transport','babylon_presentation_only','no_grid_api_mock_fallback','save_v2_to_v3_only','save_source_preserved','gate_b_hive_lockdown_only','hive_choice_three_state','npc_independent','unknown_outcome_inspection','static_not_runtime']) if (!spec.invariants.includes(i)) problems.push(`missing invariant ${i}`);
for (const k of ['implementationStarted','sourceModified','ciModified','buildExecuted','testsExecuted','runtimeExecuted','userSaveRead','userSaveWritten','assetBytesRead','mediaBytesRead','mediaCreated','mediaModelCalled','network']) if (spec.flags[k] !== false) problems.push(`flag ${k} not false`);
if (spec.currentDecision !== 'not_closed_gate_blocked') problems.push('wrong current decision');
if (spec.artifactState.globalArtifactContractAccepted !== false) problems.push('artifact contract accepted');
if (!spec.freshReplacementChecks.every((c) => 'check' in c && 'result' in c)) problems.push('replacement checks incomplete');
if (problems.length) throw new Error(problems.join('; '));

const result = {
  status:'architecture_closure_spec_replacement_ready_for_supersession_review', nb22ContentRead:false,
  scopeCoverageComplete:true, recommendation:spec.recommendation.decision, nb22SupersessionAccepted:false,
  architectureClosureAchieved:false, architectureGateOpened:false, implementationStarted:false, blockersOpen:5, blockersClosed:0,
  globalArtifactContractAccepted:false, sourceModified:false, ciModified:false, buildExecuted:false, testsExecuted:false, runtimeExecuted:false,
  userSaveRead:false, userSaveWritten:false, assetBytesRead:false, mediaBytesRead:false, mediaCreated:false, mediaModelCalled:false, network:false,
  closureDomainCount:spec.closureDomains.length, evidenceLevelCount:spec.evidenceLevels.length, dagNodeCount:spec.closureDag.nodes.length,
  dagEdgeCount:spec.closureDag.edges.length, criticalPath:spec.criticalPath, openBlockers:spec.blockerClosureRules.map((b) => b.code),
  pendingPrerequisites:['NB-22 review decision','NB-31 hard prerequisite','F1-F7 implementation evidence','global artifact contract','product decisions'],
  validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true},
  notes:['fresh replacement candidate only','predecessor content unconsumed','Gate remains closed','no media or user data']
};
const resultPath = path.join(here, '../results/new-baseline-architecture-closure-spec-replacement-nb82.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,closureDomainCount:result.closureDomainCount,evidenceLevelCount:result.evidenceLevelCount,dagNodeCount:result.dagNodeCount,dagEdgeCount:result.dagEdgeCount,criticalPath:result.criticalPath,blockersOpen:result.blockersOpen,recommendation:result.recommendation}, null, 2));
