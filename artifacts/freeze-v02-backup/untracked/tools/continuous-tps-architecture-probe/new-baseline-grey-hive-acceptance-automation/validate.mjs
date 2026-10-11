import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const own = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-grey-hive-acceptance-automation');
const planPath = path.join(root, 'docs', 'GREY_HIVE_ACCEPTANCE_AUTOMATION_SLICE_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-grey-hive-acceptance-automation-nb36.json');
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
function check(id, pass, actual, expected) { checks.push({ id, pass, actual, expected }); }
const rows = [
  ['GH-PKG-01','packaged','P0'],['GH-AUTH-02','integration','P0'],['GH-IPC-03','packaged','P0'],['GH-MOVE-04','integration','P0'],['GH-SPATIAL-05','integration','P0'],['GH-DOOR-06','integration','P0'],['GH-ROUTE-07','packaged','P0'],['GH-GATEA-08','integration','P0'],['GH-GATEB-09','integration','P0'],['GH-NPC-10','integration','P0'],['GH-CHECK-11','packaged','P0'],['GH-SAVE-12','integration','P0'],['GH-CAP-13','unit','P0'],['GH-GRID-14','static','P0'],['GH-RENDER-15','packaged','P0'],['GH-ASSET-16','static','P0'],['GH-PERF-17','packaged','P0'],['GH-STABLE-18','packaged','P0'],['GH-FAIL-19','unit','P1'],['GH-PLACE-20','manual','P1']
];
const forbidden = ['TP-18'+'D2','TP-15'+'C3','TP-18'+'D1','TP-01'+'R11'];
const classes = new Set(['static','unit','integration','packaged','manual']);
check('scope_single_world', plan.includes('only accepted world is `grey_hive`'), 'grey_hive only', true);
check('twenty_rows', rows.every(([id]) => plan.includes(id)) && rows.length === 20, rows.length, 20);
check('class_mapping', rows.every(([id, cls]) => classes.has(cls) && plan.includes(`| ${id} | ${cls} |`)), rows.map((r) => r.slice(0,2)), 'one class per row');
check('severity_mapping', rows.every(([id, , sev]) => plan.includes(`| ${id} |`) && plan.includes(`${sev} /`)), 'P0/P1 mapped', true);
check('owner_mapping', rows.every(([id]) => plan.includes(`| ${id} |`) && /\| [^|]+ owner \|/.test(plan.slice(plan.indexOf(`| ${id} |`), plan.indexOf(`| ${id} |`) + 500))), 'owner per row', true);
check('evidence_fields', ['setup', 'inputs', 'timestamps', 'owner', 'rollback'].every((x) => plan.includes(x)) && /command\/steps/.test(plan) && /expected\/observed/.test(plan) && /artifact/.test(plan) && /anchors/.test(plan) && /hash/.test(plan), 'bundle fields', true);
check('harness_manifest', ['manifest.json','evidence-schema.json','static-contract.mjs','unit-contract.rs','integration-contract.rs','packaged-smoke.mjs','manual-checklist.md','ci-phases.json'].every((x) => plan.includes(x)), 'future harness files', true);
check('packaged_recipes', ['launch-load','replay-reload','save-restore','renderer-retry'].every((x) => plan.includes(x)), 'packaged recipes', true);
check('exit_codes', ['`0` pass','`10` assertion failure','`20` invalid fixture rejected','`30` prerequisite unavailable','`40` evidence/provenance invalid','scope violation'].every((x) => plan.includes(x)), 'exit code contract', true);
check('phase_policy', ['report-only','candidate','required','not enabled','not required/enforced'].every((x) => plan.includes(x)), 'staged policy', true);
check('flaky_policy', plan.includes('retry budget') && /second\s+failure\s+remains\s+a\s+failure/.test(plan) && plan.includes('P0 acceptance row') && /quarantined P0 is/.test(plan) && plan.includes('hard failures'), 'flaky/quarantine policy', true);
check('manual_boundary', ['hand-feel','visual readability','route pacing','cannot be honestly reduced to a script','never auto-pass'].every((x) => plan.includes(x)), 'manual validation boundary', true);
check('gate_contracts', ['Gate A/B predicates','choice states','legacy Grid denial','250–350m','500–800m'].every((x) => plan.includes(x)), 'static/data gate contracts', true);
check('authority_transport_save', ['Rust owner/event trace','real packaged Tauri stream','Save v2→v3','capability authority'].every((x) => plan.includes(x)), 'automation domains', true);
check('no_promotion', plan.includes('cannot satisfy a formal row') && plan.includes('cannot enable required/enforced CI') && plan.includes('cannot promote `formalPlayableAccepted`') && /no game, harness, packaged run, CI enforcement, or formal playable pass/.test(plan), 'non-promoting state', true);
check('media_none', /Media bytes\s+are never collected/.test(plan) && plan.includes('model call') && plan.includes('media'), 'media boundary', true);
check('rollback', plan.includes('Rollback deletes only the NB-36 plan') && plan.includes('does not touch formal code, CI, assets'), 'rollback', true);
check('forbidden_identifiers', !forbidden.some((x) => plan.includes(x)), 'forbidden identifiers absent', true);
check('allowlist', plan.includes('docs/GREY_HIVE_ACCEPTANCE_AUTOMATION_SLICE_PLAN.md') && plan.includes('new-baseline-grey-hive-acceptance-automation/**') && plan.includes('new-baseline-grey-hive-acceptance-automation-nb36.json'), 'allowlisted targets', true);

const passed = checks.filter((x) => x.pass).length;
const payload = {
  schema: 'new_baseline_grey_hive_acceptance_automation_nb36_v1',
  status: passed === checks.length ? 'grey_hive_acceptance_automation_implementation_ready_gate_blocked' : 'invalid',
  conclusion: passed === checks.length ? 'grey_hive_acceptance_automation_implementation_ready_gate_blocked' : 'invalid',
  acceptedWorlds: ['grey_hive'],
  formalPlayableAccepted: false,
  ciRequiredOrEnforced: false,
  matrix: { total: 20, mapped: 20, p0: 18, p1: 2, classes: { static: 2, unit: 2, integration: 8, packaged: 7, manual: 1 }, rows },
  phases: { reportOnly: true, candidate: true, required: false, enforced: false },
  packagedRecipes: ['launch-load','replay-reload','save-restore','renderer-retry'],
  exitCodes: { pass: 0, assertionFailure: 10, invalidFixtureRejected: 20, prerequisiteUnavailable: 30, evidenceInvalid: 40, scopeViolation: 50 },
  evidenceRequired: ['setup','commandOrSteps','inputs','expected','observed','timestamps','artifactPaths','sourceTestAnchors','hashProvenance','owner','rollback'],
  manualChecks: ['movement/combat hand-feel','visual readability','route pacing','retry clarity'],
  quarantine: { p0CanQuarantineToPass: false, retrySameImmutableHash: true, secondFailure: 'failure' },
  mediaOperations: 'none',
  checks,
  summary: { total: checks.length, passed, failed: checks.length - passed },
  blockers: ['formal packaged runtime unavailable','formal Rust/IPC/Save/scene/capability owners blocked','required CI phase intentionally disabled'],
  rollback: 'delete only NB-36 allowlisted files; no formal code/CI/assets/media changed'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(payload, null, 2) + '\n');
console.log(JSON.stringify({ status: payload.status, total: checks.length, passed, failed: checks.length - passed, resultPath }, null, 2));
if (payload.status === 'invalid') process.exitCode = 1;
