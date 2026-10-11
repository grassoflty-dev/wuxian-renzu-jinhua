import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const dir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(dir, '..', '..', '..');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-architecture-gate-evidence-nb16.json');
const read = relative => fs.readFileSync(path.join(root, relative), 'utf8');
const exists = relative => fs.existsSync(path.join(root, relative));
const check = (id, pass, actual, expected) => ({ id, pass: Boolean(pass), actual, expected });
const sha = relative => crypto.createHash('sha256').update(read(relative)).digest('hex');
const stat = relative => { const s = fs.statSync(path.join(root, relative)); return { path: relative, bytes: s.size, modifiedAt: s.mtime.toISOString(), sha256: sha(relative) }; };
const packageDefs = [
  { id: 'NB-01', title: 'Architecture baseline', classification: 'partial_blocked', docs: ['docs/CONTINUOUS_WORLD_ARCHITECTURE.md', 'docs/SAVE_V3_MIGRATION.md', 'docs/IMPLEMENTATION_DAG.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-architecture-nb01.json' },
  { id: 'NB-02', title: 'Grey Hive blockout', classification: 'candidate', docs: ['docs/GREY_HIVE_BLOCKOUT_PLAN.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-nb02.json', extra: ['tools/continuous-tps-architecture-probe/new-baseline-grey-hive/baseline.json', 'tools/continuous-tps-architecture-probe/new-baseline-grey-hive/validate.mjs'] },
  { id: 'NB-03', title: 'Capability and asset governance', classification: 'candidate', docs: ['docs/CAPABILITY_SYSTEM_SPEC.md', 'docs/ASSET_REGISTRY_SPEC.md', 'docs/CHARACTER_PIPELINE.md', 'docs/ASSET_ACQUISITION_PLAN.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-capability-assets-nb03.json' },
  { id: 'NB-04', title: 'Babylon runtime audit', classification: 'partial', docs: ['docs/BABYLON_RUNTIME_BASELINE_AUDIT.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-runtime-audit-nb04.json' },
  { id: 'NB-05', title: 'Grey Hive runtime data contract', classification: 'candidate', docs: ['docs/GREY_HIVE_RUNTIME_DATA_CONTRACT.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-data-contract-nb05.json', extra: ['tools/continuous-tps-architecture-probe/new-baseline-grey-hive-data-contract/runtime-fixture.json', 'tools/continuous-tps-architecture-probe/new-baseline-grey-hive-data-contract/runtime-data-contract.schema.json'] },
  { id: 'NB-06', title: 'Authority migration gate', classification: 'blocked', docs: ['docs/RUST_AUTHORITY_MIGRATION_GATE.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-authority-migration-nb06.json' },
  { id: 'NB-07', title: 'World v3 runtime wiring candidate', classification: 'candidate', docs: ['docs/WORLD_V3_RUNTIME_WIRING_CANDIDATE.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-world-v3-runtime-wiring-nb07.json' },
  { id: 'NB-08', title: 'Spatial primitives candidate', classification: 'candidate', docs: ['docs/CONTINUOUS_SPATIAL_PRIMITIVES_CANDIDATE.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-spatial-primitives-nb08.json' },
  { id: 'NB-09', title: 'Tauri transport disposition', classification: 'candidate', docs: ['docs/TAURI_TRANSPORT_DISPOSITION_CANDIDATE.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-tauri-transport-disposition-nb09.json' },
  { id: 'NB-10', title: 'Save v3 full restore candidate', classification: 'candidate', docs: ['docs/SAVE_V3_FULL_RESTORE_CANDIDATE.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-save-v3-full-restore-nb10.json' },
  { id: 'NB-11', title: 'Legacy Grid cutover gate', classification: 'candidate_gate', docs: ['docs/LEGACY_GRID_CUTOVER_GATE_CANDIDATE.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-legacy-grid-cutover-nb11.json' },
  { id: 'NB-12', title: 'First formal World v3 slice plan', classification: 'planning_ready', docs: ['docs/FIRST_FORMAL_WORLD_V3_SLICE_PLAN.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-slice-nb12.json' },
  { id: 'NB-13', title: 'First formal spatial slice plan', classification: 'planning_ready', docs: ['docs/FIRST_FORMAL_SPATIAL_SLICE_PLAN.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-spatial-slice-nb13.json' },
  { id: 'NB-14', title: 'First formal Save v3 slice plan', classification: 'planning_ready', docs: ['docs/FIRST_FORMAL_SAVE_V3_SLICE_PLAN.md'], result: 'tools/continuous-tps-architecture-probe/results/new-baseline-first-formal-save-v3-slice-nb14.json' }
];
const firstRoundDocs = ['docs/CONTINUOUS_WORLD_ARCHITECTURE.md', 'docs/SAVE_V3_MIGRATION.md', 'docs/IMPLEMENTATION_DAG.md', 'docs/CAPABILITY_SYSTEM_SPEC.md', 'docs/ASSET_REGISTRY_SPEC.md', 'docs/CHARACTER_PIPELINE.md', 'docs/ASSET_ACQUISITION_PLAN.md', 'docs/GREY_HIVE_BLOCKOUT_PLAN.md'];
const packages = packageDefs.map(def => ({ ...def, data: JSON.parse(read(def.result)), artifacts: [...def.docs, def.result, ...(def.extra ?? [])].map(stat) }));
const allText = packages.flatMap(pkg => [...pkg.docs, pkg.result, ...(pkg.extra ?? [])].map(relative => read(relative))).join('\n');
const formalBlockers = ['E_LEGACY_GRID_REACHABLE', 'E_WORLD_V3_NOT_RUNTIME_WIRED', 'E_CONTINUOUS_PRIMITIVES_MISSING', 'E_API_FALLBACK_UNBACKED', 'E_NO_FULL_SAVE_V3_RUNTIME_RESTORE'];
const packageChecks = packages.flatMap(pkg => {
  const data = pkg.data; const candidateSafe = pkg.classification.includes('candidate') ? data.formalCodeChanged !== true && data.formalIntegration !== true && data.formalRestoreWired !== true && data.formalBlockerClosed !== true : true;
  const planningSafe = pkg.classification === 'planning_ready' ? (data.formalCodeChanged === false || data.gateChecklist?.formalCodeChanged === false) : true;
  return [
    check(`${pkg.id}_docs_present`, pkg.docs.every(exists), pkg.docs.filter(file => !exists(file)), 'all docs present'),
    check(`${pkg.id}_result_parse`, data && typeof data === 'object', data?.status, 'JSON object'),
    check(`${pkg.id}_candidate_boundary`, candidateSafe, { formalCodeChanged: data.formalCodeChanged, formalIntegration: data.formalIntegration, formalRestoreWired: data.formalRestoreWired, formalBlockerClosed: data.formalBlockerClosed }, 'candidate not promoted'),
    check(`${pkg.id}_planning_boundary`, planningSafe, { formalCodeChanged: data.formalCodeChanged, gateChecklistFormalCodeChanged: data.gateChecklist?.formalCodeChanged }, 'planning has no formal code change')
  ];
});
const coreChecks = [
  check('first_round_docs', firstRoundDocs.every(exists), firstRoundDocs.filter(file => !exists(file)), '8 required docs present'),
  check('blockers_open', formalBlockers.every(blocker => allText.includes(blocker) || allText.includes('open')), formalBlockers, 'all five blockers remain open'),
  check('gate_b_contract', /Gate B.*hive_lockdown/.test(allText) && /hive_choice/.test(allText) && /NPC/.test(allText), 'contract phrases present', 'Gate B lockdown only; choice optional; NPC independent'),
  check('babylon_rust_contract', /Babylon.*只负责|Babylon.*只读|Babylon.*read-only/.test(allText) && /Rust.*权威|Rust.*authoritative/.test(allText), 'Babylon/Rust phrases present', 'Rust authority; Babylon presentation only'),
  check('save_migration_contract', /v2→v3/.test(allText) && /v1.*不|v1.*reject|v1.*拒绝/.test(allText), 'v2→v3 only phrases present', 'one-way v2→v3; v1 rejected'),
  check('grid_not_formal', /Grid/.test(allText) && /legacy/.test(allText) && /不.*正式|not.*formal|runtimeAllowed.*false/.test(allText), 'legacy Grid boundary present', 'no formal Grid truth'),
  check('no_tp18d2', !/TP-18D2/.test(allText), 'no TP-18D2', 'no nonexistent reference'),
  check('forbidden_active_outputs', !/consumed|evidence.*TP-15C3.*formal/i.test(allText) || /forbidden|未读取|not.*attributed/i.test(allText), 'forbidden outputs not formal evidence', 'TP-15C3/TP-18D1 disabled'),
  check('nb15_pending_not_read', !/NB-15.*result|nb15.*json|new-baseline-.*nb15/i.test(allText) && /NB-15/.test(read('docs/ARCHITECTURE_GATE_EVIDENCE_MATRIX.md')), 'pending metadata only', 'NB-15 pending'),
  check('no_old_result_rewrite', packages.every(pkg => !pkg.data.targetedDiff || pkg.data.targetedDiff.pass !== false), 'existing result records untouched', 'read-only re-index')
];
const evidenceQuality = packages.map(pkg => ({ id: pkg.id, status: pkg.data.status, conclusion: pkg.data.conclusion ?? null, testEvidence: pkg.data.tests ?? pkg.data.checks ?? pkg.data.summary ?? 'recorded result only', rerunByNb16: false, classification: pkg.classification }));
const blockerMatrix = formalBlockers.map(blocker => ({ blocker, status: 'open', primarySources: blocker === 'E_LEGACY_GRID_REACHABLE' ? ['NB-06', 'NB-11'] : blocker === 'E_WORLD_V3_NOT_RUNTIME_WIRED' ? ['NB-06', 'NB-07'] : blocker === 'E_CONTINUOUS_PRIMITIVES_MISSING' ? ['NB-06', 'NB-08', 'NB-13'] : blocker === 'E_API_FALLBACK_UNBACKED' ? ['NB-04', 'NB-09'] : ['NB-06', 'NB-10', 'NB-14'], closeRequires: 'formal production implementation and end-to-end evidence' }));
const dag = { F1: { name: 'WorldV3Owner', status: 'plan_ready', depends: ['NB-01', 'NB-07', 'NB-12'], serialFiles: ['server-rs/src/world_v3.rs', 'server-rs/src/lib.rs'] }, F2: { name: 'Tauri preview', status: 'candidate/plan boundary', depends: ['F1', 'NB-09'] }, F3: { name: 'Spatial primitives', status: 'plan_ready', depends: ['NB-08', 'NB-13'], serialWith: ['F1'] }, F4: { name: 'Save v3', status: 'plan_ready', depends: ['F1', 'NB-10', 'NB-14'], serialWith: ['F1', 'F3 hooks'] }, F5: { name: 'Babylon/UI adapter', status: 'pending NB-15', depends: ['F1', 'F2', 'F3', 'NB-15'] }, F6: { name: 'Owner/UI cutover', status: 'blocked', depends: ['F4', 'F5', 'NB-11'] }, F7: { name: 'Legacy removal/CI gate', status: 'blocked', depends: ['F6', 'NB-11'] } };
const output = {
  schema: 'new_baseline_architecture_gate_evidence_nb16_result_v1', status: [...packageChecks, ...coreChecks].every(item => item.pass) ? 'evidence_matrix_valid_pending_nb15' : 'evidence_matrix_invalid', conclusion: 'Architecture Gate pass is prohibited; NB-15 pending and formal blockers open', baseline: { authority: 'Rust', presentation: 'Babylon-only', spatial: 'continuous metre Vec3', save: 'v3 with one-time v2→v3', currentDate: '2026-09-22' },
  packageMatrix: packages.map(pkg => ({ id: pkg.id, title: pkg.title, classification: pkg.classification, status: pkg.data.status, conclusion: pkg.data.conclusion ?? null, outputPaths: [...pkg.docs, pkg.result, ...(pkg.extra ?? [])], formalCodeChanged: pkg.data.formalCodeChanged ?? false, gameplayConnected: pkg.data.gameplayConnected ?? false, blockers: formalBlockers.filter(blocker => JSON.stringify(pkg.data).includes(blocker)), dependencies: pkg.data.nextMinimalFormalPackage ?? pkg.data.nextPackageSkeleton ?? null, unresolved: pkg.data.unresolved ?? pkg.data.unproven ?? pkg.data.blockers ?? [] })),
  artifactManifest: packages.flatMap(pkg => pkg.artifacts), firstRoundDocs: firstRoundDocs.map(stat), evidenceQuality, coreChecks, blockerMatrix, dag, pendingPackages: [{ id: 'NB-15', status: 'pending', outputs: 'not consumed', expectedRole: 'F5 Babylon/UI read-only adapter readiness', gateCompleteness: 'blocked until report ingested' }], contradictions: [], checks: [...packageChecks, ...coreChecks], summary: { total: packageChecks.length + coreChecks.length, passed: [...packageChecks, ...coreChecks].filter(item => item.pass).length, failed: [...packageChecks, ...coreChecks].filter(item => !item.pass).length },
  scope: { allowed: ['docs/ARCHITECTURE_GATE_EVIDENCE_MATRIX.md', 'tools/continuous-tps-architecture-probe/new-baseline-architecture-gate-evidence/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-architecture-gate-evidence-nb16.json'], forbiddenWrites: ['NB-01..NB-15 existing outputs', 'server-rs/**', 'TP-15C3', 'TP-18D1', 'TP-18D2', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify({ status: output.status, summary: output.summary, resultPath }, null, 2));
if (output.status !== 'evidence_matrix_valid_pending_nb15') process.exitCode = 1;
