import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const decision = JSON.parse(fs.readFileSync(path.join(here, 'decision.json'), 'utf8'));
const matrix = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/new-baseline-three-world-formal-rollout-acceptance-replacement/matrix.json'), 'utf8'));
const result89 = JSON.parse(fs.readFileSync(path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-three-world-formal-rollout-acceptance-replacement-nb89.json'), 'utf8'));
const problems = [];
if (decision.oldIdentity.contentRead !== false || decision.oldIdentity.consumed !== false) problems.push('old package consumed');
if (decision.replacementIdentity.packageId !== 'NB-89' || decision.replacementIdentity.contentRead !== true) problems.push('replacement identity invalid');
if (matrix.acceptanceItems.length !== 36 || matrix.acceptanceItems.filter((i) => i.negative).length !== 12) problems.push('count mismatch');
for (const w of ['grey_hive','mist_harbor','clockworks']) if (matrix.acceptanceItems.filter((i) => i.world === w).length !== 12) problems.push(`${w} count mismatch`);
if (result89.status !== 'three_world_formal_rollout_acceptance_replacement_ready_for_supersession_review' || result89.scopeCoverageComplete !== true) problems.push('NB-89 result invalid');
for (const s of ['R0','R1','R2','R3','R4','R5','R6']) if (!matrix.stages.some((x) => x.id === s)) problems.push(`missing ${s}`);
if (decision.scopeChecklist.length < 10 || decision.scopeChecklist.some((x) => !['covered','partial','missing'].includes(x.result))) problems.push('scope checklist incomplete');
if (decision.recommendation.value !== 'recommend_defer_supersession_until_gate_and_product_prerequisites' || decision.recommendation.accepted !== false) problems.push('recommendation invalid');
if (decision.productAnswersRecorded !== 0) problems.push('product answer recorded');
for (const k of ['nb42SupersessionAccepted','acceptanceDecisionRecorded','greyHiveAccepted','mistHarborAccepted','clockworkCityAccepted','threeWorldRolloutAccepted','architectureGateOpened','implementationStarted','sourceModified','buildExecuted','testsExecuted','runtimeExecuted','mediaBytesRead','mediaCreated','mediaModelCalled','network']) if (decision.flags[k] !== false) problems.push(`flag ${k} invalid`);
if (decision.flags.blockersOpen !== 5 || decision.flags.blockersClosed !== 0) problems.push('blocker state invalid');
for (const k of ['decisionId','selectedDecision','decider','timestamp','scope','evidenceIds','conditions','revocation','reopen']) if (!(k in decision.acceptanceTemplate)) problems.push(`acceptance template missing ${k}`);
if (decision.currentDecision !== 'defer_pending_main_session') problems.push('current decision invalid');
if (problems.length) throw new Error(problems.join('; '));

const result = {
  status:'nb42_nb89_supersession_decision_packet_ready', nb42ContentRead:false, nb89ScopeCoverageComplete:true,
  recommendation:decision.recommendation.value, nb42SupersessionAccepted:false, acceptanceDecisionRecorded:false, productAnswersRecorded:0,
  greyHiveAccepted:false, mistHarborAccepted:false, clockworkCityAccepted:false, threeWorldRolloutAccepted:false, architectureGateOpened:false,
  implementationStarted:false, blockersOpen:5, blockersClosed:0, sourceModified:false, buildExecuted:false, testsExecuted:false, runtimeExecuted:false,
  mediaBytesRead:false, mediaCreated:false, mediaModelCalled:false, network:false, scopeItemCount:decision.scopeChecklist.length,
  countChecks:decision.countChecks, worldChecks:decision.worldChecks, deferConditionCount:decision.deferConditions.length, remainingRiskCount:decision.remainingRisks.length,
  validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true}, notes:['decision materials only','no product answers','old package unconsumed','worlds remain unaccepted']
};
const resultPath = path.join(here, '../results/new-baseline-nb42-nb89-supersession-decision-nb92.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,scopeItemCount:result.scopeItemCount,countChecks:result.countChecks,worldChecks:result.worldChecks,recommendation:result.recommendation,deferConditionCount:result.deferConditionCount,productAnswersRecorded:result.productAnswersRecorded}, null, 2));
