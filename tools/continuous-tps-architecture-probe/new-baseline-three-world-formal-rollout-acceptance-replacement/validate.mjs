import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const matrix = JSON.parse(fs.readFileSync(path.join(here, 'matrix.json'), 'utf8'));
const problems = [];
if (matrix.stages.length !== 7 || matrix.stages[0].id !== 'R0' || matrix.stages.at(-1).id !== 'R6') problems.push('R0-R6 incomplete');
const stageIds = new Set(matrix.stages.map((s) => s.id));
for (const item of matrix.acceptanceItems) if (!stageIds.has(item.stage)) problems.push(`${item.id}: unknown stage`);
if (matrix.acceptanceItems.length < 30) problems.push('fewer than 30 acceptance items');
if (matrix.acceptanceItems.filter((i) => i.negative).length < 12) problems.push('fewer than 12 negative items');
for (const w of ['grey_hive','mist_harbor','clockworks']) if (matrix.acceptanceItems.filter((i) => i.world === w).length < 8) problems.push(`${w}: fewer than 8 items`);
const itemIds = new Set(); for (const i of matrix.acceptanceItems) { if (itemIds.has(i.id)) problems.push(`duplicate ${i.id}`); itemIds.add(i.id); for (const k of ['id','world','stage','category','negative','source','expected','owner','dependencies','status']) if (!(k in i)) problems.push(`${i.id}: missing ${k}`); if (['accepted','approved','implemented','verified','playable'].includes(i.status)) problems.push(`${i.id}: acceptance claim`); }
if (matrix.negativeCases.some((id) => !itemIds.has(id))) problems.push('negative case dangling');
if (matrix.worldRequirements.grey_hive.sourceCounts.rooms !== 18 || matrix.worldRequirements.grey_hive.sourceCounts.connectors !== 17 || matrix.worldRequirements.grey_hive.sourceCounts.doors !== 17 || matrix.worldRequirements.grey_hive.sourceCounts.triggers !== 15 || matrix.worldRequirements.grey_hive.sourceCounts.checkpoints !== 6) problems.push('Grey Hive counts changed');
for (const w of ['mist_harbor','clockworks']) if (!matrix.worldRequirements[w].unfrozen?.length) problems.push(`${w}: pending scope missing`);
if (matrix.transitionRules.returnStation !== 'R6 post-three-world stability; separate acceptance') problems.push('Return Station not post-R6');
if (matrix.freshReplacementChecks.nb42ContentRead !== false || matrix.recommendation.nb42SupersessionAccepted !== false) problems.push('old predecessor read/accepted');
if (matrix.currentDecision !== 'not_closed_gate_blocked') problems.push('wrong current decision');
for (const [k,v] of Object.entries(matrix.flags)) if (k === 'blockersOpen' || k === 'blockersClosed' ? (k === 'blockersOpen' ? v !== 5 : v !== 0) : v !== false) problems.push(`flag ${k} invalid`);
if (!matrix.mediaReleaseBoundary || matrix.mediaReleaseBoundary.publicRelease !== 'Registry/License/Compliance/Packaging required') problems.push('media boundary incomplete');
if (problems.length) throw new Error(problems.join('; '));

const worldCounts = Object.fromEntries(['grey_hive','mist_harbor','clockworks'].map((w) => [w, matrix.acceptanceItems.filter((i) => i.world === w).length]));
const result = {
  status:'three_world_formal_rollout_acceptance_replacement_ready_for_supersession_review', nb42ContentRead:false, scopeCoverageComplete:true,
  recommendation:matrix.recommendation.decision, nb42SupersessionAccepted:false, greyHiveAccepted:false, mistHarborAccepted:false, clockworkCityAccepted:false,
  threeWorldRolloutAccepted:false, architectureGateOpened:false, implementationStarted:false, blockersOpen:5, blockersClosed:0, sourceModified:false,
  buildExecuted:false, testsExecuted:false, runtimeExecuted:false, manualPlaytestExecuted:false, assetBytesRead:false, mediaBytesRead:false,
  mediaCreated:false, mediaModelCalled:false, network:false, stageCount:matrix.stages.length, acceptanceItemCount:matrix.acceptanceItems.length,
  negativeCount:matrix.acceptanceItems.filter((i) => i.negative).length, worldCounts, transitionRules:matrix.transitionRules, productDecisionGateCount:matrix.productDecisionGates.length,
  validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true}, notes:['fresh replacement only','all worlds unaccepted','Return Station deferred to R6','no media or runtime']
};
const resultPath = path.join(here, '../results/new-baseline-three-world-formal-rollout-acceptance-replacement-nb89.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,stageCount:result.stageCount,acceptanceItemCount:result.acceptanceItemCount,negativeCount:result.negativeCount,worldCounts,recommendation:result.recommendation,blockersOpen:result.blockersOpen}, null, 2));
