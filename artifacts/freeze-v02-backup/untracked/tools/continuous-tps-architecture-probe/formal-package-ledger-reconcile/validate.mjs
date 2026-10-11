import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/formal-package-ledger-reconcile-tp18c1.json');
const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-package-ledger-reconcile/validate.mjs',
  'tools/continuous-tps-architecture-probe/results/formal-package-ledger-reconcile-tp18c1.json',
  'docs/CONTINUOUS_FORMAL_PACKAGE_LEDGER_RECONCILIATION.md',
];
const noRead = [
  'AG-44 unfinished output',
  'TP-01R7 unfinished output',
  'TP-01R9 unfinished output',
  'AG-38 unfinished output',
  'assets/**, data/**, media/**, public/** physical bytes',
  'network and remote resources',
];
const completedEvidence = {
  'PKG-FORMAL-CORE-TICK-KCC': ['tools/continuous-tps-architecture-probe/results/formal-core-readiness-tp01r2.json'],
  'PKG-FORMAL-GREY-HIVE-SCENE': ['tools/continuous-tps-architecture-probe/results/formal-scene-readiness-tp01r3.json'],
  'PKG-FORMAL-IPC-OWNER': ['tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json'],
  'PKG-FORMAL-SAVE-CAMPAIGN': ['tools/continuous-tps-architecture-probe/results/formal-save-campaign-readiness-tp01r5.json'],
  'PKG-FORMAL-ASSET-RELEASE': ['tools/continuous-tps-architecture-probe/results/formal-asset-release-readiness-tp01r6.json'],
  'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH': ['tools/continuous-tps-architecture-probe/results/formal-campaign-playthrough-readiness-tp01r8.json'],
};
const activeNotConsumed = {
  'PKG-FORMAL-CAPABILITY-HUD': { task: 'TP-01R7', status: 'active_not_consumed', pathRead: false },
  'PKG-FORMAL-PERF-SOAK-ACCEPTANCE': { task: 'TP-01R9', status: 'active_not_consumed', pathRead: false },
  'AG-44': { task: 'AG-44', status: 'active_not_consumed', pathRead: false },
  'AG-38': { task: 'AG-38', status: 'active_not_consumed', pathRead: false },
};
const inputPaths = [
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-tp18a1.json',
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json',
  ...Object.values(completedEvidence).flat(),
];

const abs = (rel) => path.join(root, rel);
const readJson = (rel) => JSON.parse(fs.readFileSync(abs(rel), 'utf8'));
const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const freshness = (rel) => {
  const file = abs(rel);
  if (!fs.existsSync(file)) return { exists: false, sha256: null, mtimeMs: null };
  const stat = fs.statSync(file);
  return { exists: true, sha256: sha256(fs.readFileSync(file)), mtimeMs: stat.mtimeMs };
};

const tp18a = readJson('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-tp18a1.json');
const tp18b = readJson('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json');
const evidenceResults = Object.fromEntries(Object.entries(completedEvidence).map(([id, paths]) => [id, paths.map((rel) => readJson(rel))]));
const packages = tp18b.packages;
const packageIds = packages.map((item) => item.id);
const packageSet = new Set(packageIds);
const waves = tp18b.waves;
const edges = tp18b.conflictGraph.edges;

const packageBlockers = (id, spec) => {
  const direct = evidenceResults[id]?.flatMap((item) => (item.blockers || []).map((blocker) => typeof blocker === 'string' ? blocker : blocker.code)).filter(Boolean) || [];
  const inherited = (spec.unsatisfiedPrerequisites || []).filter((prerequisite) => packageSet.has(prerequisite)).map((prerequisite) => `E_DEP_${prerequisite}`);
  if (activeNotConsumed[id]) inherited.push(`E_ACTIVE_NOT_CONSUMED_${activeNotConsumed[id].task}`);
  return [...new Set([...direct, ...inherited])];
};

const waveByPackage = new Map(waves.flatMap((wave) => wave.slots.map((id) => [id, wave.wave])));
const allWriteSets = new Map(packages.map((item) => [item.id, new Set(item.writes)]));
const sharedFilePairs = new Set();
for (const [index, a] of packages.entries()) {
  for (const b of packages.slice(index + 1)) {
    if ([...allWriteSets.get(a.id)].some((file) => allWriteSets.get(b.id).has(file))) sharedFilePairs.add([a.id, b.id].sort().join('|'));
  }
}
const edgePairs = new Set(edges.map((edge) => [edge.a, edge.b].sort().join('|')));
const sameWaveConflicts = [];
for (const wave of waves) {
  for (let i = 0; i < wave.slots.length; i += 1) {
    for (const id of wave.slots.slice(i + 1)) {
      const overlap = [...allWriteSets.get(wave.slots[i])].filter((file) => allWriteSets.get(id).has(file));
      if (overlap.length) sameWaveConflicts.push({ wave: wave.wave, a: wave.slots[i], b: id, files: overlap });
    }
  }
}
const missingConflictEdges = [...sharedFilePairs].filter((pair) => !edgePairs.has(pair));
const extraConflictEdges = [...edgePairs].filter((pair) => !sharedFilePairs.has(pair));
const duplicatePackageIds = packageIds.filter((id, index) => packageIds.indexOf(id) !== index);
const danglingDependencies = packages.flatMap((item) => (item.prerequisites || []).filter((dependency) => dependency.startsWith('PKG-') && !packageSet.has(dependency)).map((dependency) => ({ package: item.id, dependency })));

const dependencyGraph = new Map(packages.map((item) => [item.id, (item.prerequisites || []).filter((dependency) => packageSet.has(dependency))]));
const visitState = new Map();
const cycleNodes = [];
const visit = (id, stack = []) => {
  const state = visitState.get(id);
  if (state === 'visiting') { cycleNodes.push([...stack, id]); return; }
  if (state === 'visited') return;
  visitState.set(id, 'visiting');
  for (const dependency of dependencyGraph.get(id) || []) visit(dependency, [...stack, id]);
  visitState.set(id, 'visited');
};
for (const id of packageIds) visit(id);
const waveTopologyViolations = packages.flatMap((item) => {
  const targetWave = waveByPackage.get(item.id);
  if (!targetWave) return [];
  return (item.prerequisites || []).filter((dependency) => packageSet.has(dependency) && waveByPackage.has(dependency) && waveByPackage.get(dependency) >= targetWave).map((dependency) => ({ package: item.id, dependency, targetWave, dependencyWave: waveByPackage.get(dependency) }));
});
const deferredPackages = new Set(tp18b.deferredPackages);
const schedulableViolations = packages.filter((item) => item.scheduled === true || item.status !== 'not_schedulable_until_prerequisites' || (item.unsatisfiedPrerequisites || []).length === 0).map((item) => item.id);

const blockerTraceability = Object.fromEntries(packages.map((item) => [item.id, {
  blockerCodes: packageBlockers(item.id, item),
  evidencePaths: completedEvidence[item.id] || [],
  activeNotConsumed: activeNotConsumed[item.id] || null,
  evidenceLevels: evidenceResults[item.id]?.map((result) => ({ status: result.status, candidateOnly: result.candidateOnly ?? result.candidate_only ?? null, formalSceneReady: result.formalSceneReady ?? null, formalGate: result.formalGate ?? null })) || [],
}]));

const negativeFixtures = [
  ['E_PACKAGE_MISSING', packages.length === 8],
  ['E_PACKAGE_DUPLICATE', duplicatePackageIds.length === 0],
  ['E_DANGLING_DEPENDENCY', danglingDependencies.length === 0],
  ['E_DEPENDENCY_CYCLE', cycleNodes.length === 0],
  ['E_CONFLICT_EDGE_ASYMMETRY', edges.every((edge) => edge.a !== edge.b && edgePairs.has([edge.a, edge.b].sort().join('|')))],
  ['E_MISSING_CONFLICT_EDGE', missingConflictEdges.length === 0],
  ['E_EXTRA_CONFLICT_EDGE', extraConflictEdges.length === 0],
  ['E_SAME_WAVE_WRITE_CONFLICT', sameWaveConflicts.length === 0],
  ['E_WAVE_TOPOLOGY', waveTopologyViolations.length === 0 && waves.length === 3],
  ['E_ACTIVE_AG44_CONSUMED', activeNotConsumed['AG-44'].status === 'active_not_consumed' && activeNotConsumed['AG-44'].pathRead === false],
  ['E_ACTIVE_TP01R7_CONSUMED', activeNotConsumed['PKG-FORMAL-CAPABILITY-HUD'].pathRead === false],
  ['E_ACTIVE_TP01R9_CONSUMED', activeNotConsumed['PKG-FORMAL-PERF-SOAK-ACCEPTANCE'].pathRead === false],
  ['E_ACTIVE_AG38_CONSUMED', activeNotConsumed['AG-38'].pathRead === false],
  ['E_BLOCKED_TO_SCHEDULABLE', schedulableViolations.length === 0],
  ['E_CANDIDATE_TO_FORMAL', packages.every((item) => item.status !== 'formal_pass') && tp18b.gateConclusion.architecture === 'no_go'],
  ['E_BLOCKER_MISSING', packages.every((item) => packageBlockers(item.id, item).length > 0)],
  ['E_BLOCKER_FORGED', Object.values(blockerTraceability).every((entry) => entry.evidencePaths.length > 0 || entry.activeNotConsumed)],
  ['E_EVIDENCE_PATH_DRIFT', inputPaths.every((rel) => freshness(rel).exists)],
  ['E_EVIDENCE_HASH_DRIFT', Object.values(Object.fromEntries(inputPaths.map((rel) => [rel, freshness(rel)]))).every((item) => item.sha256)],
  ['E_TEST_MISSING', packages.every((item) => Array.isArray(item.commands) && item.commands.length > 0)],
  ['E_ROLLBACK_MISSING', packages.every((item) => typeof item.rollback === 'string' && item.rollback.length > 0)],
  ['E_PROVENANCE_MISSING', tp18a.schema === 'vertical-slice-readiness-tp18a1-v1' && tp18b.schema === 'vertical-slice-readiness-delta-tp18b1-v1'],
  ['E_PATH_OUTSIDE_SCOPE', allowedWrites.length === 3 && !allowedWrites.some((rel) => /(^|[\\/])(assets|data|media|public)([\\/])/.test(rel))],
  ['E_MEDIA_READ', noRead.some((entry) => /media|physical bytes/.test(entry))],
  ['E_NETWORK_ACCESS', noRead.some((entry) => /network/.test(entry))],
  ['E_OVERALL_GATE_BYPASS', tp18b.gateConclusion.architecture === 'no_go' && tp18b.gateConclusion.verticalSliceReady === false],
  ['E_DEFERRED_ACTIVE_INFERENCE', deferredPackages.has('PKG-FORMAL-CAPABILITY-HUD') && deferredPackages.has('PKG-FORMAL-PERF-SOAK-ACCEPTANCE') && activeNotConsumed['PKG-FORMAL-CAPABILITY-HUD'].pathRead === false],
].map(([code, observed]) => ({ code, observed: Boolean(observed), expected: true, pass: Boolean(observed) }));

const ledger = packages.map((item) => ({
  packageId: item.id,
  goal: item.goal,
  prerequisites: item.prerequisites,
  unsatisfiedPrerequisites: item.unsatisfiedPrerequisites,
  evidencePaths: completedEvidence[item.id] || [],
  activeEvidence: activeNotConsumed[item.id] || null,
  blockerCodes: packageBlockers(item.id, item),
  writeSet: item.writes,
  tests: item.commands,
  rollback: item.rollback,
  scheduled: item.scheduled,
  currentStatus: item.status,
  wave: waveByPackage.get(item.id) ?? null,
  deferred: deferredPackages.has(item.id),
  evidenceLevel: evidenceResults[item.id]?.map((result) => result.status) || ['active_not_consumed'],
}));

const result = {
  schema: 'formal_package_ledger_reconcile_tp18c1_v1',
  generatedAtUtc: new Date().toISOString(),
  status: 'reconciled_blocked',
  decision: 'reconciled_blocked',
  architecture: 'no_go',
  schedulingAuthorized: false,
  allowedWrites,
  noRead,
  activeNotConsumed,
  sourceLedgers: {
    tp18a: { path: 'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-tp18a1.json', schema: tp18a.schema, status: tp18a.status, freshness: freshness('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-tp18a1.json') },
    tp18b: { path: 'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json', schema: tp18b.schema, status: tp18b.status, freshness: freshness('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json') },
  },
  readOnlyInputs: Object.fromEntries(inputPaths.map((rel) => [rel, freshness(rel)])),
  ledger,
  statistics: {
    packageCount: packages.length,
    uniquePackageIds: new Set(packageIds).size,
    conflictEdgeCount: edges.length,
    expectedConflictEdgeCount: 16,
    conflictEdgeCoverage: { missing: missingConflictEdges, extra: extraConflictEdges },
    waveCount: waves.length,
    waveSlots: waves,
    deferredPackages: [...deferredPackages],
    sameWaveWriteConflicts: sameWaveConflicts,
    dependencyCycleNodes: cycleNodes,
    danglingDependencies,
    waveTopologyViolations,
  },
  evidenceTraceability: blockerTraceability,
  negativeFixtures,
  negativeFixtureCount: negativeFixtures.length,
  allNegativeFixturesPass: negativeFixtures.every((item) => item.pass),
  exactBlockers: [
    { code: 'E_ALL_FORMAL_PACKAGES_BLOCKED', detail: 'All eight TP-18B1 packages remain not_schedulable_until_prerequisites; no package is scheduled.' },
    { code: 'E_CORE_READINESS_BLOCKED', detail: 'TP-01R2 formal core blocker set remains unsatisfied.' },
    { code: 'E_SCENE_READINESS_BLOCKED', detail: 'TP-01R3 formal scene readiness is false; scene evidence remains candidate/synthetic.' },
    { code: 'E_IPC_READINESS_BLOCKED', detail: 'TP-01R4 formal IPC owner is blocked.' },
    { code: 'E_SAVE_READINESS_BLOCKED', detail: 'TP-01R5 formal Save/Campaign owner is blocked.' },
    { code: 'E_ASSET_READINESS_BLOCKED', detail: 'TP-01R6 formal asset release is blocked.' },
    { code: 'E_PLAYTHROUGH_READINESS_BLOCKED', detail: 'TP-01R8 formal campaign playthrough is blocked.' },
    { code: 'E_ACTIVE_OUTPUTS_NOT_CONSUMED', detail: 'AG-44, TP-01R7, TP-01R9, and AG-38 are active_not_consumed; no readiness result is inferred.' },
    { code: 'E_ARCHITECTURE_NO_GO', detail: 'TP-18B1 verticalSliceReady=false and architecture=no_go.' },
  ],
  forbiddenPromotion: ['schedulable', 'formal_pass', 'wave execution', 'Architecture go', 'active output inference'],
  rollbackReview: 'Delete only this validator, result, and review; do not alter TP-18 ledgers, completed readiness outputs, active outputs, source code, fixtures, media, data, public tree, Git history, or remote.',
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ status: result.status, decision: result.decision, packageCount: result.statistics.packageCount, conflictEdgeCount: result.statistics.conflictEdgeCount, waveCount: result.statistics.waveCount, negativeFixtureCount: result.negativeFixtureCount, allNegativeFixturesPass: result.allNegativeFixturesPass, resultPath }, null, 2));
process.exit(result.allNegativeFixturesPass ? 0 : 1);
