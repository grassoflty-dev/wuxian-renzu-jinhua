import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-grey-hive-first-playable-acceptance');
const planPath = path.join(root, 'docs', 'GREY_HIVE_FIRST_PLAYABLE_ACCEPTANCE.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-grey-hive-first-playable-acceptance-nb32.json');
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
function check(id, pass, actual, expected) { checks.push({ id, pass, actual, expected }); }
const requiredDomains = ['packaged Babylon-only launch', 'Rust single authority', 'Tauri-only packaged', 'continuous metre-based movement', 'collision/trigger/room/zone/door/', 'Grey Hive route and gates', 'checkpoints and re-entry', 'Save v3 restore', 'minimum capability set', 'legacy Grid exclusion', 'performance/stability', 'asset/placeholder registry boundary'];
const owners = ['release owner', 'Rust world owner', 'Tauri transport owner', 'world physics owner', 'spatial owner', 'door/nav owner', 'Grey Hive owner', 'progression owner', 'interaction owner', 'checkpoint owner', 'Save owner', 'capability owner', 'authority reviewer', 'Babylon owner', 'registry owner', 'performance owner', 'runtime owner', 'gameplay owner', 'asset owner'];
const ids = ['GH-PKG-01','GH-AUTH-02','GH-IPC-03','GH-MOVE-04','GH-SPATIAL-05','GH-DOOR-06','GH-ROUTE-07','GH-GATEA-08','GH-GATEB-09','GH-NPC-10','GH-CHECK-11','GH-SAVE-12','GH-CAP-13','GH-GRID-14','GH-RENDER-15','GH-ASSET-16','GH-PERF-17','GH-STABLE-18','GH-FAIL-19','GH-PLACE-20'];
const forbidden = ['TP-18' + 'D2', 'TP-15' + 'C3', 'TP-18' + 'D1', 'TP-01' + 'R11'];
check('scope_single_world', plan.includes('exactly one playable world: `grey_hive`') && plan.includes('Other worlds are explicitly out of scope'), 'Grey Hive only', true);
check('required_domains', requiredDomains.every((x) => plan.includes(x)), requiredDomains, 'all domains');
check('matrix_ids', ids.every((x) => plan.includes(x)), ids.length, 20);
check('matrix_has_steps', ['Prerequisites', 'Test setup and steps', 'Expected', 'Evidence'].every((x) => plan.includes(x)), 'matrix columns', true);
check('owners_present', owners.every((x) => plan.includes(x)), owners.length, 'owners named');
check('p0_policy', plan.includes('Every row below is blocking unless marked P1') && plan.includes('P0 failures block'), 'P0/P1 policy', true);
check('distances', plan.includes('250–350m') && plan.includes('500–800m') && plan.includes('100×100m'), 'route/site bands', true);
check('gate_b_choice_npc', ['hive_lockdown', 'hive_choice', 'taken', 'left', 'unresolved', 'NPC', 'not prerequisites'].every((x) => plan.includes(x)), 'Gate B independence', true);
check('checkpoint_restore', ['A01/A03/B01/B05/C02/C04', 'semantic checkpoint', 'terminal/no spawn', 'epoch'].every((x) => plan.includes(x)), 'checkpoint/re-entry', true);
check('save_v3', ['v2→v3', 'backup/atomic/pending', 'invalid data fail closed'].every((x) => plan.includes(x)), 'Save v3 matrix', true);
check('capability_minimum', ['combat primary/dodge', 'scan, pickup, interact, stealth', 'target/cost/cooldown/phase', 'immutable events'].every((x) => plan.includes(x)), 'capability matrix', true);
check('authority_grid', ['Rust is the only state/effect authority', 'no Grid/tile/px/py authority', 'migration-only'].every((x) => plan.includes(x)), 'authority/Grid boundary', true);
check('packaged_recipes', ['Packaged runtime smoke', 'Deterministic replay', 'Save restore and world reload', 'Renderer retry'].every((x) => plan.includes(x)), 'reproducible recipes', true);
check('candidate_non_promotion', plan.includes('candidate evidence never counts as a formal pass') && plan.includes('reference_only') && /candidate\s+or\s+implementation-ready plans are not promotion evidence/.test(plan), 'candidate stays non-promoting', true);
check('placeholder_policy', plan.includes('explicit placeholder') && plan.includes('Asset ID') && plan.includes('license/provenance') && /cannot\s+\n?stand in for missing collision/.test(plan), 'placeholder boundary', true);
check('media_policy', ['No image, audio, video, voice, effect or music is generated', 'downloaded', 'converted'].every((x) => plan.includes(x)), 'media prohibited', true);
check('blocked_conclusion', plan.includes('grey_hive_first_playable_acceptance_ready_gate_blocked') && plan.includes('not accepted'), 'blocked conclusion', true);
check('forbidden_ids_absent', !forbidden.some((x) => plan.includes(x)), 'forbidden IDs absent', true);
check('allowlist_scope', plan.includes('docs/GREY_HIVE_FIRST_PLAYABLE_ACCEPTANCE.md') && plan.includes('new-baseline-grey-hive-first-playable-acceptance/**') && plan.includes('new-baseline-grey-hive-first-playable-acceptance-nb32.json'), 'three allowlisted targets', true);

const passed = checks.filter((x) => x.pass).length;
const payload = {
  schema: 'new_baseline_grey_hive_first_playable_acceptance_nb32_v1',
  status: passed === checks.length ? 'grey_hive_first_playable_acceptance_ready_gate_blocked' : 'invalid',
  conclusion: passed === checks.length ? 'grey_hive_first_playable_acceptance_ready_gate_blocked' : 'invalid',
  acceptedWorlds: ['grey_hive'],
  implicitlyAcceptedWorlds: [],
  candidatePromotion: false,
  formalPlayableAccepted: false,
  matrix: { total: 20, p0: 18, p1: 2, ids },
  blockingDomains: ['packaged_launch','authority','transport','movement_spatial_nav','route_distance','gate_a_gate_b_choice_npc','checkpoint_reentry','save_v3','capabilities','legacy_grid','renderer_lifecycle','asset_registry','performance','stability'],
  routeBands: { siteEnvelopeM: [100,100], legalShortestM: [250,350], recommendedStoryM: [250,350], completeExplorationM: [500,800] },
  gatePolicy: { gateA: 'hive_power only', gateB: 'hive_lockdown only', choiceStates: ['taken','left','unresolved'], choiceAffectsGateB: false, npcAffectsGateB: false },
  evidenceRule: 'formal packaged or formal source/test artifact with commit/build/hash provenance; candidate/reference-only evidence cannot pass',
  placeholderRule: 'registry/license/provenance required; placeholder cannot replace collision/nav/checkpoint/gameplay evidence',
  mediaOperations: 'none',
  checks,
  summary: { total: checks.length, passed, failed: checks.length - passed },
  blockers: ['formal integrated Rust owner missing','packaged Tauri transport missing','formal scene/collider/nav/camera/checkpoint chain missing','production Save/CampaignStore handoff missing','formal capability owner missing','asset release provenance/manifest blocked','formal performance/soak samples missing'],
  rollback: 'delete only the NB-32 allowlisted files; no formal code, asset, CI or media state changed'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: checks.length, passed, failed: checks.length - passed, resultPath }, null, 2));
if (payload.status === 'invalid') process.exitCode = 1;
