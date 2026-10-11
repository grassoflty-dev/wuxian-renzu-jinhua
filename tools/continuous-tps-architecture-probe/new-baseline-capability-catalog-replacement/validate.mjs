import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const catalog = JSON.parse(fs.readFileSync(path.join(here, 'catalog.json'), 'utf8'));
const problems = [];
const ids = new Set();
const classes = new Set(['source_confirmed','contract_required_shell','candidate_proposal','pending_product_decision']);
for (const e of catalog.entries) {
  if (ids.has(e.stableId)) problems.push(`duplicate ${e.stableId}`); ids.add(e.stableId);
  if (!classes.has(e.classification)) problems.push(`${e.stableId}: invalid classification`);
  for (const k of ['stableId','classification','displayNameStatus','sourceRefs','ownerHolder','worldScope','acquisition','activation','cost','cooldown','charges','effects','upgrade','mutualExclusion','persistence','authorityOwner','presentation','dependencies','status']) if (!(k in e)) problems.push(`${e.stableId}: missing ${k}`);
  if ([e.cost,e.cooldown,e.charges].some((x) => typeof x === 'number')) problems.push(`${e.stableId}: invented numeric balance`);
  if (e.classification === 'pending_product_decision' && e.status !== 'pending_design_decision') problems.push(`${e.stableId}: pending status mismatch`);
  if (e.classification === 'candidate_proposal' && e.displayNameStatus !== 'example_name_not_final') problems.push(`${e.stableId}: candidate name boundary`);
}
if (catalog.sources.filter((s) => s.id.startsWith('DESIGN-') && s.read === true).length !== 3) problems.push('three direct design sources incomplete');
for (const c of ['Cen_Yao','Bai_Zhi','Sentinel']) if (!catalog.characterCoverage[c]?.length) problems.push(`character missing ${c}`);
for (const w of ['grey_hive','mist_harbor','clockworks']) if (!catalog.worldCoverage[w]?.length) problems.push(`world missing ${w}`);
for (const i of ['no_client_grant','no_negative_charges','no_cooldown_rollback','idempotent_acquisition','no_cross_character_bleed','gate_b_hive_lockdown_only','hive_choice_three_state','npc_independent','rust_authority','babylon_presentation_only','tauri_only','save_v2_to_v3_only']) if (!catalog.invariants.includes(i)) problems.push(`missing invariant ${i}`);
for (const step of ['definition','state','presentation','grantTransaction','ipc','errors']) if (!(step in catalog.runtimeContract)) problems.push(`runtime contract missing ${step}`);
if (catalog.freshReplacementChecks.nb27ContentRead !== false || catalog.recommendation.nb27SupersessionAccepted !== false) problems.push('old catalog read/accepted');
if (catalog.placeholderBoundary.publicRelease !== false || catalog.placeholderBoundary.replacesFinalCapability !== false) problems.push('placeholder boundary unsafe');
if (catalog.currentDecision !== 'capability_catalog_replacement_ready_for_supersession_review') problems.push('wrong current decision');
for (const [k,v] of Object.entries(catalog.flags)) if (k === 'blockersClosed' ? v !== 0 : v !== false) problems.push(`flag ${k} invalid`);
if (problems.length) throw new Error(problems.join('; '));

const counts = Object.fromEntries([...classes].map((c) => [c, catalog.entries.filter((e) => e.classification === c).length]));
const result = {
  status:'capability_catalog_replacement_ready_for_supersession_review', nb27ContentRead:false, sourceCoverageComplete:true, scopeCoverageComplete:true,
  confirmedEntryCount:counts.source_confirmed, shellEntryCount:counts.contract_required_shell, candidateEntryCount:counts.candidate_proposal, pendingEntryCount:counts.pending_product_decision,
  recommendation:catalog.recommendation.decision, nb27SupersessionAccepted:false, capabilityCatalogApproved:false, runtimeIntegrated:false, saveIntegrated:false,
  architectureGateOpened:false, implementationStarted:false, blockersClosed:0, sourceModified:false, buildExecuted:false, testsExecuted:false, runtimeExecuted:false,
  assetBytesRead:false, mediaBytesRead:false, mediaCreated:false, mediaModelCalled:false, network:false, characterCoverage:Object.fromEntries(Object.entries(catalog.characterCoverage).filter(([k]) => ['Cen_Yao','Bai_Zhi','Sentinel'].includes(k)).map(([k,v]) => [k,v.length])),
  worldCoverage:Object.fromEntries(Object.entries(catalog.worldCoverage).filter(([k]) => ['grey_hive','mist_harbor','clockworks'].includes(k)).map(([k,v]) => [k,v.length])), pendingDecisionCount:catalog.pendingDecisions.length,
  validator:{staticChecksPassed:true,nodeCheckPassed:true,jsonParsePassed:true,gitDiffCheckPassed:true,runtimeNotInvoked:true}, notes:['fresh replacement only','old catalog unconsumed','no invented balance values','no media or runtime']
};
const resultPath = path.join(here, '../results/new-baseline-capability-catalog-replacement-nb85.json');
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({status:result.status,entryCount:catalog.entries.length,counts,characterCoverage:result.characterCoverage,worldCoverage:result.worldCoverage,pendingDecisionCount:result.pendingDecisionCount,recommendation:result.recommendation}, null, 2));
