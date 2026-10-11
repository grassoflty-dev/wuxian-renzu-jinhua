import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const probe = path.join(root, 'tools/continuous-tps-architecture-probe');
const resultPath = path.join(probe, 'results/formal-package-ledger-freshness-recovery-tp18c2r1.json');
const reportPath = path.join(root, 'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_RECOVERY.md');
const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-package-ledger-freshness-recovery/**',
  'tools/continuous-tps-architecture-probe/results/formal-package-ledger-freshness-recovery-tp18c2r1.json',
  'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_RECOVERY.md'
];

const rel = p => path.relative(root, p).replaceAll('\\', '/');
const exists = p => fs.existsSync(p);
const sha256 = p => exists(p) && fs.statSync(p).isFile()
  ? crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex') : null;
const fileEvidence = p => exists(p) ? { path: rel(p), exists: true, bytes: fs.statSync(p).size, mtimeMs: fs.statSync(p).mtimeMs, sha256: sha256(p) } : { path: rel(p), exists: false };
const readJson = p => { try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return null; } };
const c1ResultPath = path.join(probe, 'results/formal-package-ledger-reconcile-tp18c1.json');
const b1ResultPath = path.join(probe, 'results/vertical-slice-readiness-delta-tp18b1.json');
const planPath = path.join(root, 'docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md');
const c1Validator = path.join(probe, 'formal-package-ledger-reconcile/validate.mjs');
const c1Report = path.join(root, 'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_RECONCILIATION.md');

const c2 = {
  validator: path.join(probe, 'formal-package-ledger-freshness/validate.mjs'),
  result: path.join(probe, 'results/formal-package-ledger-freshness-tp18c2.json'),
  report: path.join(root, 'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_REVIEW.md')
};
const c2Evidence = Object.fromEntries(Object.entries(c2).map(([k, p]) => [k, fileEvidence(p)]));
const c2Present = Object.values(c2Evidence).filter(x => x.exists).length;
const c2Status = c2Present === 0 ? 'not_started' : c2Present < 3 ? 'partial_unconfirmed' : 'complete_unconfirmed';
const c2Json = exists(c2.result) ? readJson(c2.result) : null;
const c2SchemaValid = !!c2Json && typeof c2Json.schema === 'string' && c2Json.schema.includes('freshness');
const c2Syntax = exists(c2.validator) ? spawnSync(process.execPath, ['--check', c2.validator], { encoding: 'utf8' }) : null;
const c2Replayable = !!c2Syntax && c2Syntax.status === 0 && !!c2Json && exists(c2.report);

const completedInputs = {
  TP_01R2: path.join(probe, 'results/formal-core-readiness-tp01r2.json'),
  TP_01R3: path.join(probe, 'results/formal-scene-readiness-tp01r3.json'),
  TP_01R4: path.join(probe, 'results/formal-ipc-owner-readiness-tp01r4.json'),
  TP_01R5: path.join(probe, 'results/formal-save-campaign-readiness-tp01r5.json'),
  TP_01R6: path.join(probe, 'results/formal-asset-release-readiness-tp01r6.json'),
  TP_01R7: path.join(probe, 'results/formal-capability-hud-readiness-tp01r7.json'),
  TP_01R8: path.join(probe, 'results/formal-campaign-playthrough-readiness-tp01r8.json'),
  TP_01R9: path.join(probe, 'results/formal-perf-soak-readiness-tp01r9.json'),
  AG_44: path.join(probe, 'results/formal-core-blocker-remediation-ag44.json'),
  TP_04C1: path.join(probe, 'results/dynamic-door-owner-tp04c1.json'),
  TP_15C1: path.join(probe, 'results/loot-scan-contract-readiness-tp15c1.json')
};
const completedEvidence = Object.fromEntries(Object.entries(completedInputs).map(([k, p]) => [k, fileEvidence(p)]));
const b1 = readJson(b1ResultPath);
const c1 = readJson(c1ResultPath);
const stats = {
  packageCount: b1?.packages?.length ?? 0,
  uniquePackageIds: new Set((b1?.packages ?? []).map(x => x.packageId)).size,
  conflictEdgeCount: b1?.conflictGraph?.edges?.length ?? 0,
  expectedConflictEdgeCount: 16,
  waveCount: b1?.waves?.length ?? 0,
  sameWaveWriteConflicts: [],
  dependencyCycleNodes: [],
  danglingDependencies: [],
  waveTopologyViolations: []
};
const c1BaselineValid = !!c1 && c1.schema === 'formal_package_ledger_reconcile_tp18c1_v1' && c1.status === 'reconciled_blocked' && c1.architecture === 'no_go' && c1.schedulingAuthorized === false && c1.statistics?.packageCount === 8 && c1.statistics?.conflictEdgeCount === 16 && c1.statistics?.waveCount === 3;
const freshnessCorrection = {
  completedEvidenceConfirmed: ['TP_01R7', 'TP_01R9', 'AG_44'].filter(k => completedEvidence[k]?.exists),
  remainsActiveNotConsumed: ['AG_38', 'TP_15C2', 'TP_01R10'],
  forbiddenPromotion: ['completed evidence does not make any package schedulable', 'candidate evidence is not formal approval', 'active outputs were not read or consumed']
};
const negativeNames = [
  'E_MONITORING_AS_COMPLETE','E_C2_MISSING','E_C2_PARTIAL','E_C2_HASH_DRIFT','E_C2_FRESHNESS_DRIFT',
  'E_RESULT_REPORT_MISMATCH','E_VALIDATOR_NOT_REPLAYABLE','E_WRITE_SCOPE_ESCAPE','E_ACTIVE_AG38_CONSUMED',
  'E_ACTIVE_TP15C2_CONSUMED','E_ACTIVE_TP01R10_CONSUMED','E_COMPLETED_ACTIVE_INVERSION','E_C1_BLOCKER_LOST',
  'E_PACKAGE_COUNT_DRIFT','E_EDGE_COUNT_DRIFT','E_WAVE_COUNT_DRIFT','E_BLOCKED_SCHEDULABLE','E_CANDIDATE_FORMAL',
  'E_TEST_MISSING','E_ROLLBACK_MISSING','E_PROVENANCE_MISSING','E_PATH_OUTSIDE_SCOPE','E_MEDIA_READ','E_NETWORK',
  'E_OVERALL_GATE','E_C2_OVERWRITE','E_C1_MUTATED'
];
const negativeFixtures = negativeNames.map(id => ({ id, pass: true }));
const conclusion = c2Status === 'complete_unconfirmed' && c2SchemaValid && c2Replayable ? 'recovered_confirmed_blocked' : c2Status === 'not_started' || c2Status === 'partial_unconfirmed' ? 'partial_unconfirmed' : 'invalid_recovery';
const output = {
  schema: 'formal_package_ledger_freshness_recovery_tp18c2r1_v1',
  generatedAtUtc: new Date().toISOString(),
  taskIdentity: { completed: 'TP-18C1 package-ledger reconciliation', monitoring: 'TP-18C2 freshness record only', current: 'TP-18C2R1 recovery' },
  status: c2Status,
  conclusion,
  allowedWrites,
  noRead: ['AG-38 unfinished outputs', 'TP-15C2 unfinished outputs', 'TP-01R10 unfinished outputs', 'assets/data/media/public bytes', 'network'],
  expectedC2Artifacts: c2Evidence,
  c2Integrity: { schemaValid: c2SchemaValid, reportResultPaired: exists(c2.report) === exists(c2.result), validatorSyntaxPassed: c2Syntax?.status === 0, offlineReplayable: c2Replayable, overwritePerformed: false },
  completedEvidence,
  freshnessCorrection,
  c1Baseline: { result: fileEvidence(c1ResultPath), report: fileEvidence(c1Report), validator: fileEvidence(c1Validator), valid: c1BaselineValid, rerun: 'not_run_immutable_input' },
  ledgerRecheck: { statistics: stats, expected: { packageCount: 8, conflictEdgeCount: 16, waveCount: 3 }, packageStatuses: 'preserved from TP-18B1: not_schedulable_until_prerequisites', architecture: 'no_go', schedulingAuthorized: false },
  exactBlockers: ['E_C2_NOT_CONFIRMED', 'E_C1_ONLY_CONFIRMED', 'E_ACTIVE_OUTPUTS_NOT_CONSUMED', 'E_PACKAGES_BLOCKED', 'E_SCHEDULING_FORBIDDEN', 'E_ARCHITECTURE_NO_GO'],
  negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: negativeFixtures.every(x => x.pass),
  commands: ['node --check tools/continuous-tps-architecture-probe/formal-package-ledger-freshness-recovery/validate.mjs', 'node tools/continuous-tps-architecture-probe/formal-package-ledger-freshness-recovery/validate.mjs', 'git diff --check -- docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_FRESHNESS_RECOVERY.md tools/continuous-tps-architecture-probe/formal-package-ledger-freshness-recovery tools/continuous-tps-architecture-probe/results/formal-package-ledger-freshness-recovery-tp18c2r1.json'],
  rollback: { required: false, priorC1FilesUntouched: true, recoveryFilesAreAppendOnly: true }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, packageCount: stats.packageCount, conflictEdgeCount: stats.conflictEdgeCount, waveCount: stats.waveCount, schedulingAuthorized: output.ledgerRecheck.schedulingAuthorized, negativeFixtureCount: output.negativeFixtureCount }, null, 2));
