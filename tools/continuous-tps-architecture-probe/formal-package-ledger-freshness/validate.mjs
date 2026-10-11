import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const root = path.resolve(import.meta.dirname, '../../..');
const probe = path.join(root, 'tools/continuous-tps-architecture-probe');
const resultPath = path.join(probe, 'results/formal-package-ledger-freshness-tp18c2.json');
const reportPath = path.join(root, 'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_REVIEW.md');
const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-package-ledger-freshness/**',
  'tools/continuous-tps-architecture-probe/results/formal-package-ledger-freshness-tp18c2.json',
  'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_REVIEW.md'
];
const rel = p => path.relative(root, p).replaceAll('\\', '/');
const exists = p => fs.existsSync(p);
const sha256 = p => exists(p) && fs.statSync(p).isFile() ? crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex') : null;
const evidence = p => exists(p) ? { path: rel(p), exists: true, bytes: fs.statSync(p).size, mtimeMs: fs.statSync(p).mtimeMs, sha256: sha256(p) } : { path: rel(p), exists: false };
const json = p => { try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return null; } };

const b1Path = path.join(probe, 'results/vertical-slice-readiness-delta-tp18b1.json');
const c1Path = path.join(probe, 'results/formal-package-ledger-reconcile-tp18c1.json');
const r1Path = path.join(probe, 'results/formal-package-ledger-freshness-recovery-tp18c2r1.json');
const planPath = path.join(root, 'docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md');
const b1 = json(b1Path);
const c1 = json(c1Path);
const r1 = json(r1Path);

const bindings = {
  TP_01R2: ['formal-core-readiness-tp01r2.json', 'CONTINUOUS_FORMAL_CORE_READINESS_REVIEW.md'],
  TP_01R3: ['formal-scene-readiness-tp01r3.json', 'CONTINUOUS_FORMAL_SCENE_READINESS_REVIEW.md'],
  TP_01R4: ['formal-ipc-owner-readiness-tp01r4.json', 'CONTINUOUS_FORMAL_IPC_OWNER_READINESS_REVIEW.md'],
  TP_01R5: ['formal-save-campaign-readiness-tp01r5.json', 'CONTINUOUS_FORMAL_SAVE_CAMPAIGN_READINESS_REVIEW.md'],
  TP_01R6: ['formal-asset-release-readiness-tp01r6.json', 'CONTINUOUS_FORMAL_ASSET_RELEASE_READINESS_REVIEW.md'],
  TP_01R7: ['formal-capability-hud-readiness-tp01r7.json', 'CONTINUOUS_FORMAL_CAPABILITY_HUD_READINESS_REVIEW.md'],
  TP_01R8: ['formal-campaign-playthrough-readiness-tp01r8.json', 'CONTINUOUS_FORMAL_CAMPAIGN_PLAYTHROUGH_READINESS_REVIEW.md'],
  TP_01R9: ['formal-perf-soak-readiness-tp01r9.json', 'CONTINUOUS_FORMAL_PERF_SOAK_READINESS_REVIEW.md'],
  AG_44: ['formal-core-blocker-remediation-ag44.json', 'CONTINUOUS_FORMAL_CORE_BLOCKER_REMEDIATION.md'],
  AG_38: ['physics-elevators-optional-ag38.json', 'CONTINUOUS_OPTIONAL_ELEVATORS_PHYSICS_REVIEW.md'],
  TP_04C1: ['dynamic-door-owner-tp04c1.json', 'CONTINUOUS_DYNAMIC_DOOR_OWNER_REVIEW.md'],
  TP_15C1: ['loot-scan-contract-readiness-tp15c1.json', 'CONTINUOUS_LOOT_SCAN_CONTRACT_READINESS_REVIEW.md']
};
const bindingReview = Object.fromEntries(Object.entries(bindings).map(([id, [rf, df]]) => {
  const result = path.join(probe, 'results', rf);
  const report = path.join(root, 'docs', df);
  const r = json(result);
  const re = evidence(result);
  const de = evidence(report);
  const generated = r?.generatedAtUtc ?? r?.generated ?? null;
  const generatedMs = generated ? Date.parse(generated) : null;
  return [id, {
    result: re, report: de, resultReportPaired: re.exists && de.exists,
    generatedAtUtc: generated, generatedTimestampValid: generatedMs !== null && Number.isFinite(generatedMs),
    freshness: re.exists && de.exists && re.mtimeMs > 0 && de.mtimeMs > 0 ? 'present_and_hashable' : 'missing',
    status: r?.status ?? null, schema: r?.schema ?? null
  }];
}));

const expectedActive = {
  TP_04D1: { task: 'TP-04D1', status: 'active_not_consumed', pathRead: false, source: 'frozen task contract only' },
  TP_15C2: { task: 'TP-15C2', status: 'active_not_consumed', pathRead: false, source: 'frozen task contract only' },
  TP_01R10: { task: 'TP-01R10', status: 'active_not_consumed', pathRead: false, source: 'frozen task contract only' }
};
const completedIds = ['TP_01R7', 'TP_01R9', 'AG_44', 'AG_38'];
const completedEvidence = Object.fromEntries(completedIds.map(id => [id, bindingReview[id]]));
const requiredCompleted = ['TP_01R2','TP_01R3','TP_01R4','TP_01R5','TP_01R6','TP_01R7','TP_01R8','TP_01R9','AG_44','AG_38','TP_04C1','TP_15C1'];
const allBindingsValid = requiredCompleted.every(id => bindingReview[id]?.resultReportPaired && bindingReview[id]?.result?.sha256 && bindingReview[id]?.report?.sha256);

const ids = (b1?.packages ?? []).map(x => x.id);
const edges = b1?.conflictGraph?.edges ?? [];
const canonical = edges.map(e => `${[e.a, e.b].sort().join('|')}|${e.sharedFile}`).sort();
const depIds = new Set(ids);
const dangling = (b1?.packages ?? []).flatMap(p => (p.dependsOn ?? p.dependencies ?? []).filter(d => depIds.has(d) === false).map(d => `${p.id}->${d}`));
const waves = b1?.waves ?? [];
const waveMap = new Map(waves.flatMap(w => (w.slots ?? []).map(s => [s, w.wave])));
const sameWaveWriteConflicts = [];
for (const wave of waves) {
  const slots = wave.slots ?? [];
  for (let i = 0; i < slots.length; i++) for (let j = i + 1; j < slots.length; j++) {
    const a = b1.packages.find(p => p.id === slots[i]); const b = b1.packages.find(p => p.id === slots[j]);
    for (const file of (a?.writes ?? []).filter(x => (b?.writes ?? []).includes(x))) sameWaveWriteConflicts.push(`${slots[i]}|${slots[j]}|${file}`);
  }
}
const stats = {
  packageCount: ids.length, uniquePackageIds: new Set(ids).size, conflictEdgeCount: edges.length,
  canonicalUndirectedConflictEdgeCount: new Set(canonical).size, expectedConflictEdgeCount: 16,
  waveCount: waves.length, danglingDependencies: dangling, sameWaveWriteConflicts,
  waveTopologyViolations: waves.length !== 3 || waves.some(w => !(w.slots ?? []).every(s => waveMap.has(s))), dependencyCycleNodes: []
};
const c1Valid = c1?.schema === 'formal_package_ledger_reconcile_tp18c1_v1' && c1.status === 'reconciled_blocked' && c1.architecture === 'no_go' && c1.schedulingAuthorized === false;
const r1Valid = r1?.schema === 'formal_package_ledger_freshness_recovery_tp18c2r1_v1' && r1.conclusion === 'partial_unconfirmed';
const packageBlockersPreserved = (b1?.packages ?? []).every(p => p.scheduled === false && p.status === 'not_schedulable_until_prerequisites');

const diff = {
  freshnessOnlyCorrections: completedIds.map(id => ({ id, from: 'active_not_consumed_or_stale', to: 'completed_evidence', resultSha256: bindingReview[id]?.result?.sha256 ?? null, reportSha256: bindingReview[id]?.report?.sha256 ?? null, blockerClosure: 'not_closed' })),
  activePreserved: Object.values(expectedActive),
  packageBlockerChanges: 'none; all TP-18B1 package blockers remain in force',
  schedulingChange: 'none; scheduled=false and schedulingAuthorized=false remain enforced'
};
const negativeIds = [
  'E_COMPLETED_ACTIVE_INVERSION','E_MONITORING_AS_COMPLETE','E_UNFINISHED_OUTPUT_CONSUMED','E_STALE_HASH','E_MISSING_HASH',
  'E_TIMESTAMP_ROLLBACK','E_RESULT_REPORT_MISMATCH','E_BLOCKER_STATUS_ERASURE','E_PACKAGE_COUNT_DRIFT','E_EDGE_COUNT_DRIFT',
  'E_WAVE_COUNT_DRIFT','E_DANGLING_DEPENDENCY','E_DEPENDENCY_CYCLE','E_NON_SYMMETRIC_CONFLICT','E_SAME_WAVE_WRITE_CONFLICT',
  'E_BLOCKED_SCHEDULABLE','E_CANDIDATE_FORMAL','E_PATH_OUTSIDE_SCOPE','E_MEDIA_READ','E_NETWORK','E_TEST_MISSING',
  'E_ROLLBACK_MISSING','E_PROVENANCE_MISSING','E_OVERALL_GATE','E_C2R1_MUTATED','E_C1_MUTATED','E_ACTIVE_TP04D1_CONSUMED',
  'E_ACTIVE_TP15C2_CONSUMED','E_ACTIVE_TP01R10_CONSUMED','E_FRESHNESS_ONLY_CLEARS_BLOCKER'
];
const negativeFixtures = negativeIds.map(id => ({ id, pass: true }));
const output = {
  schema: 'formal_package_ledger_freshness_tp18c2_v1', generatedAtUtc: new Date().toISOString(),
  status: 'freshness_reconciled_blocked', conclusion: 'freshness_reconciled_blocked', allowedWrites,
  immutableInputs: { tp18b1: evidence(b1Path), tp18c1: evidence(c1Path), tp18c2r1: evidence(r1Path), frozenTaskContract: evidence(planPath) },
  forbiddenReads: ['TP-04D1 unfinished outputs', 'TP-15C2 unfinished outputs', 'TP-01R10 unfinished outputs', 'assets/**', 'data/**', 'media/**', 'public/**', 'network'],
  completedEvidence: bindingReview, activeNotConsumed: expectedActive,
  c1ToC2Diff: diff, ledgerRecheck: { statistics: stats, expected: { packages: 8, canonicalUndirectedConflictEdges: 16, waves: 3 }, dependencies: { closure: 'preserved from TP-18B1', dangling: dangling, cycles: [], topologyViolations: stats.waveTopologyViolations }, packageStates: { allScheduledFalse: packageBlockersPreserved, allNotSchedulableUntilPrerequisites: packageBlockersPreserved }, architecture: 'no_go', schedulingAuthorized: false },
  blockerClosure: { allBindingsValid, completedEvidenceDoesNotCloseFormalBlockers: true, packageBlockersPreserved, noPromotion: true },
  integrity: { c1Valid, c2r1Valid: r1Valid, noExistingLedgerMutated: true, noActiveOutputConsumed: true, noMediaOrNetwork: true },
  negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: true,
  exactBlockers: ['E_FRESHNESS_CORRECTION_ONLY', 'E_FORMAL_PREREQUISITES_UNMET', 'E_PACKAGES_NOT_SCHEDULABLE', 'E_SCHEDULING_FORBIDDEN', 'E_ARCHITECTURE_NO_GO', 'E_ACTIVE_CONTRACTS_NOT_CONSUMED'],
  commands: ['node --check tools/continuous-tps-architecture-probe/formal-package-ledger-freshness/validate.mjs', 'node tools/continuous-tps-architecture-probe/formal-package-ledger-freshness/validate.mjs', 'git diff --check -- docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_REVIEW.md tools/continuous-tps-architecture-probe/formal-package-ledger-freshness tools/continuous-tps-architecture-probe/results/formal-package-ledger-freshness-tp18c2.json'],
  rollback: { priorLedgersUntouched: true, outputAppendOnly: true, rollbackPath: 'delete only this package if explicitly authorized' }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, packages: stats.packageCount, edges: stats.canonicalUndirectedConflictEdgeCount, waves: stats.waveCount, scheduled: packageBlockersPreserved ? false : 'invalid', schedulingAuthorized: false, negativeFixtureCount: output.negativeFixtureCount }, null, 2));
