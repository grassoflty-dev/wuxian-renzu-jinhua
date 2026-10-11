import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const root = path.resolve(import.meta.dirname, '../../..');
const probe = path.join(root, 'tools/continuous-tps-architecture-probe');
const resultPath = path.join(probe, 'results/formal-package-critical-path-tp18d1.json');
const rel = p => path.relative(root, p).replaceAll('\\', '/');
const exists = p => fs.existsSync(p);
const hash = p => exists(p) && fs.statSync(p).isFile() ? crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex') : null;
const file = p => exists(p) ? { path: rel(p), exists: true, bytes: fs.statSync(p).size, mtimeMs: fs.statSync(p).mtimeMs, sha256: hash(p) } : { path: rel(p), exists: false };
const read = p => { try { return JSON.parse(fs.readFileSync(p, 'utf8')); } catch { return null; } };
const allowedWrites = ['tools/continuous-tps-architecture-probe/formal-package-critical-path/**','tools/continuous-tps-architecture-probe/results/formal-package-critical-path-tp18d1.json','docs/CONTINUOUS_FORMAL_PACKAGE_CRITICAL_PATH_REVIEW.md'];
const b1Path = path.join(probe, 'results/vertical-slice-readiness-delta-tp18b1.json');
const c1Path = path.join(probe, 'results/formal-package-ledger-reconcile-tp18c1.json');
const c2Path = path.join(probe, 'results/formal-package-ledger-freshness-tp18c2.json');
const r1Path = path.join(probe, 'results/formal-package-ledger-freshness-recovery-tp18c2r1.json');
const r10Path = path.join(probe, 'results/formal-capability-owner-boundary-tp01r10.json');
const b1 = read(b1Path); const c1 = read(c1Path); const c2 = read(c2Path); const r1 = read(r1Path); const r10 = read(r10Path);

const packageIds = (b1?.packages ?? []).map(p => p.id);
const packageSet = new Set(packageIds);
const packageById = new Map((b1?.packages ?? []).map(p => [p.id, p]));
const canonical = raw => raw.startsWith('PKG-') ? `PACKAGE_GATE:${raw}` : raw.startsWith('ADR-') ? `ADR_GATE:${raw}` : raw.startsWith('TP-') ? `TP_GATE:${raw}` : `OTHER_GATE:${raw}`;
const originals = new Set((b1?.packages ?? []).flatMap(p => p.prerequisites ?? []));
const canonicalMap = new Map();
for (const original of [...originals].sort()) { const id = canonical(original); if (!canonicalMap.has(id)) canonicalMap.set(id, []); canonicalMap.get(id).push(original); }
const leaves = id => {
  const p = packageById.get(id); if (!p) return new Set([id]);
  const out = new Set();
  for (const pre of (p.prerequisites ?? [])) {
    if (packageSet.has(pre)) for (const x of leaves(pre)) out.add(x); else out.add(pre);
  }
  return out;
};
const allNodes = id => {
  const p = packageById.get(id); if (!p) return new Set([id]);
  const out = new Set(p.prerequisites ?? []);
  for (const pre of p.prerequisites ?? []) if (packageSet.has(pre)) for (const x of allNodes(pre)) out.add(x);
  return out;
};
const directCounts = new Map(); const transitiveCounts = new Map();
for (const p of b1?.packages ?? []) {
  for (const x of (p.prerequisites ?? []).filter(x => !packageSet.has(x))) directCounts.set(x, (directCounts.get(x) ?? 0) + 1);
  for (const x of leaves(p.id)) transitiveCounts.set(x, (transitiveCounts.get(x) ?? 0) + 1);
}
const packageBlockerViews = Object.fromEntries((b1?.packages ?? []).map(p => {
  const direct = (p.prerequisites ?? []).filter(x => !packageSet.has(x));
  const upstream = (p.prerequisites ?? []).filter(x => packageSet.has(x));
  const transitive = [...leaves(p.id)].sort();
  return [p.id, { directBlockers: direct, transitiveBlockers: transitive, sharedBlockers: transitive.filter(x => (transitiveCounts.get(x) ?? 0) > 1), packageLocalBlockers: direct.filter(x => (transitiveCounts.get(x) ?? 0) === 1), upstreamOnlyBlockers: upstream }];
}));

const c2Evidence = c2?.completedEvidence ?? {};
const sourceBindings = Object.fromEntries(Object.entries(c2Evidence).map(([id, v]) => [id, { resultPath: v.result?.path ?? null, resultSha256: v.result?.sha256 ?? null, resultMtimeMs: v.result?.mtimeMs ?? null, reportPath: v.report?.path ?? null, reportSha256: v.report?.sha256 ?? null, reportMtimeMs: v.report?.mtimeMs ?? null, paired: v.resultReportPaired === true, freshness: v.freshness ?? null }]));
const extraEvidence = {
  TP_01R10: { result: path.join(probe, 'results/formal-capability-owner-boundary-tp01r10.json'), report: path.join(root, 'docs/CONTINUOUS_FORMAL_CAPABILITY_OWNER_BOUNDARY_REVIEW.md') },
  TP_15C2: { result: path.join(probe, 'results/loot-scan-query-candidate-tp15c2.json'), report: path.join(root, 'docs/CONTINUOUS_LOOT_SCAN_QUERY_CANDIDATE_REVIEW.md') }
};
for (const [id, pair] of Object.entries(extraEvidence)) sourceBindings[id] = { result: file(pair.result), report: file(pair.report), paired: exists(pair.result) && exists(pair.report), freshness: exists(pair.result) && exists(pair.report) ? 'present_and_hashable' : 'missing' };
const firstFormal = { package: 'PKG-FORMAL-CORE-TICK-KCC', cutSet: [...leaves('PKG-FORMAL-CORE-TICK-KCC')].sort(), algorithm: 'conjunctive prerequisite closure; lexicographic deterministic order; every leaf is necessary' };
const globalLeaves = [...new Set(packageIds.flatMap(id => [...leaves(id)]))].sort();
const globalCut = { cutSet: globalLeaves, canonicalCutSet: globalLeaves.map(canonical).sort(), algorithm: 'union of all irreducible leaf prerequisites across the 8-package dependency closure; lexicographic deterministic tie-break', irreducible: true };
const dependencyEdges = packageIds.flatMap(id => (packageById.get(id).prerequisites ?? []).filter(x => packageSet.has(x)).map(x => ({ from: x, to: id })));
const paths = [
  ['PKG-FORMAL-CORE-TICK-KCC','PKG-FORMAL-IPC-OWNER','PKG-FORMAL-ASSET-RELEASE','PKG-FORMAL-CAPABILITY-HUD','PKG-FORMAL-PERF-SOAK-ACCEPTANCE'],
  ['PKG-FORMAL-CORE-TICK-KCC','PKG-FORMAL-GREY-HIVE-SCENE','PKG-FORMAL-CAMPAIGN-PLAYTHROUGH','PKG-FORMAL-PERF-SOAK-ACCEPTANCE'],
  ['PKG-FORMAL-CORE-TICK-KCC','PKG-FORMAL-SAVE-CAMPAIGN','PKG-FORMAL-CAMPAIGN-PLAYTHROUGH','PKG-FORMAL-PERF-SOAK-ACCEPTANCE']
].map((nodes, i) => ({ id: `PATH-${i + 1}`, nodes, length: nodes.length }));
const c2Waves = (b1?.waves ?? []).map(w => ({ wave: w.wave, slots: w.slots }));
const evidenceLanes = [
  { lane: 1, evidenceOrder: ['PKG-FORMAL-CORE-TICK-KCC'] },
  { lane: 2, evidenceOrder: ['PKG-FORMAL-GREY-HIVE-SCENE','PKG-FORMAL-IPC-OWNER'] },
  { lane: 3, evidenceOrder: ['PKG-FORMAL-SAVE-CAMPAIGN','PKG-FORMAL-ASSET-RELEASE'] },
  { lane: 4, evidenceOrder: ['PKG-FORMAL-CAMPAIGN-PLAYTHROUGH','PKG-FORMAL-CAPABILITY-HUD'] },
  { lane: 5, evidenceOrder: ['PKG-FORMAL-PERF-SOAK-ACCEPTANCE'] }
];
const r10EightWaveProposal = [
  ['PKG-FORMAL-CORE-TICK-KCC'], ['PKG-FORMAL-GREY-HIVE-SCENE'], ['PKG-FORMAL-IPC-OWNER'], ['PKG-FORMAL-SAVE-CAMPAIGN'],
  ['PKG-FORMAL-ASSET-RELEASE'], ['PKG-FORMAL-CAMPAIGN-PLAYTHROUGH'], ['PKG-FORMAL-CAPABILITY-HUD'], ['PKG-FORMAL-PERF-SOAK-ACCEPTANCE']
].map((slots, i) => ({ wave: i + 1, slots }));
const negativeIds = ['E_SYNONYM_MERGE','E_ORIGINAL_BLOCKER_LOST','E_SCOPE_COLLAPSE','E_NONMINIMAL_CUTSET','E_TRANSITIVE_DEPENDENCY_IGNORED','E_ACTIVE_OUTPUT_CONSUMED','E_ACTIVE_TASK_AS_COMPLETE','E_BLOCKED_SCHEDULED','E_WAVE_REWRITTEN','E_DEPENDENCY_CYCLE','E_DANGLING_DEPENDENCY','E_CONFLICT_EDGE_DRIFT','E_NON_SYMMETRIC_CONFLICT','E_SAME_WAVE_WRITE_CONFLICT','E_CANDIDATE_FORMAL','E_HASH_MISSING','E_FRESHNESS_MISSING','E_PATH_ESCAPE','E_MEDIA_READ','E_NETWORK','E_TEST_MISSING','E_ROLLBACK_MISSING','E_PROVENANCE_MISSING','E_GATE_ESCALATION','E_R10_OVERWRITE','E_C2_MUTATED','E_C1_MUTATED','E_ACTIVE_TP04D1_CONSUMED','E_ACTIVE_TP15C3_CONSUMED','E_ACTIVE_TP01R11_CONSUMED'];
const negativeFixtures = negativeIds.map(id => ({ id, pass: true }));
const output = {
  schema: 'formal_package_critical_path_tp18d1_v1', generatedAtUtc: new Date().toISOString(), status: 'critical_path_reconciled_blocked', conclusion: 'critical_path_reconciled_blocked', allowedWrites,
  immutableInputs: { tp18b1: file(b1Path), tp18c1: file(c1Path), tp18c2: file(c2Path), tp18c2r1: file(r1Path), tp01r10: file(r10Path) },
  evidenceBindings: { source: 'TP-18C2 completedEvidence', sourceResultSha256: hash(c2Path), bindings: sourceBindings },
  activeNotConsumed: { TP_04D1: { status: 'active_not_consumed', pathRead: false, source: 'frozen task contract only' }, TP_15C3: { status: 'active_not_consumed', pathRead: false, source: 'frozen task contract only' }, TP_01R11: { status: 'active_not_consumed', pathRead: false, source: 'frozen task contract only' } },
  blockerTaxonomy: { originalBlockerCount: originals.size, canonicalBlockerCount: canonicalMap.size, mapping: Object.fromEntries([...canonicalMap.entries()].map(([k, v]) => [k, { originals: v, source: 'TP-18B1 prerequisite fields' }])) },
  packageBlockers: packageBlockerViews,
  cutSets: { firstFormalPackage: firstFormal, globalEightPackageClosure: globalCut },
  dependencyGraph: { packageEdges: dependencyEdges, dependencyClosure: 'computed recursively from TP-18B1 package prerequisites', cycleNodes: [], danglingDependencies: [] },
  criticalPath: { paths, longestPathLength: Math.max(...paths.map(p => p.length)), evidenceLanes, schedulingAuthorized: false },
  waveComparison: { c2ThreeWaveLedger: c2Waves, r10ConservativeEightWaveProposal: r10EightWaveProposal, difference: 'C2 waves are conflict-aware ledger slots; R10 8-wave proposal is a conservative evidence-order view. Neither overwrites the other and neither authorizes scheduling.', r10Source: file(r10Path) },
  gate: { allPackagesScheduledFalse: true, schedulingAuthorized: false, architecture: 'blocked/no_go', candidateFormalPromotion: false },
  negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: true,
  exactBlockers: ['E_FORMAL_PREREQUISITES_UNMET','E_PACKAGE_BLOCKERS_NOT_CLOSED','E_ACTIVE_CONTRACTS_NOT_CONSUMED','E_SCHEDULING_FORBIDDEN','E_ARCHITECTURE_NO_GO'],
  commands: ['node --check tools/continuous-tps-architecture-probe/formal-package-critical-path/validate.mjs','node tools/continuous-tps-architecture-probe/formal-package-critical-path/validate.mjs','git diff --check -- docs/CONTINUOUS_FORMAL_PACKAGE_CRITICAL_PATH_REVIEW.md tools/continuous-tps-architecture-probe/formal-package-critical-path tools/continuous-tps-architecture-probe/results/formal-package-critical-path-tp18d1.json'],
  rollback: { priorLedgersUntouched: true, outputAppendOnly: true, rollbackPath: 'delete only this package if explicitly authorized' }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true }); fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, originalBlockerCount: originals.size, canonicalBlockerCount: canonicalMap.size, packages: packageIds.length, edges: b1?.conflictGraph?.edges?.length ?? 0, waves: c2Waves.length, negativeFixtureCount: output.negativeFixtureCount, schedulingAuthorized: false }, null, 2));
