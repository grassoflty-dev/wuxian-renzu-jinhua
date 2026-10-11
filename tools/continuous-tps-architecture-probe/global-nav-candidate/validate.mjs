import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..', '..', '..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/global-nav-candidate-ag43.json');
const result = JSON.parse(fs.readFileSync(resultPath, 'utf8'));
const failures = [];
const check = (name, condition, detail = '') => {
  if (!condition) failures.push({name, detail});
};
const approx = (a, b, tolerance = 0.001) => Math.abs(a - b) <= tolerance;

check('schema', result.schema === 'continuous_grey_hive_global_nav_candidate_ag43_v1');
check('status', result.status === 'blocked_formal_candidate_partial_exploration');
check('machinePass', result.machinePass === true);
check('graphCounts', result.graph.nodeCount === 18 && result.graph.edgeCount === 17 && result.graph.includedEdgeCount === 14 && result.graph.excludedEdgeCount === 3);
check('directedArcCount', result.graph.directedArcCount === 28);
check('allEdgesHaveAttribution', result.graph.edges.every(e => e.source && e.source.layout && e.source.result && e.source.inventory && e.evidenceLevel && Number.isFinite(e.lengthM)));
check('allEdgesHavePolicyFields', result.graph.edges.every(e => Array.isArray(e.prerequisite) && e.directionalityCandidate === 'bidirectional_candidate' && typeof e.includedInGraph === 'boolean'));

const core = result.graph.edges.find(e => e.id === 'edge-b01-b02-core-v7');
const gateB = result.graph.edges.find(e => e.id === 'edge-b05-c01');
check('coreExistsAndIsNotGateB', Boolean(core) && core.gate == null && core.lengthEvidence?.nominalV7M === 40 && approx(core.lengthEvidence.formalReplayReferenceM, 39.998558044433594, 1e-12));
check('coreDeltaPreserved', approx(core?.lengthEvidence.absoluteDeltaMm, 1.44195556640625, 1e-12));
check('trueGateBScope', Boolean(gateB) && gateB.gate?.name === 'Gate B' && gateB.id === 'edge-b05-c01' && gateB.from === 'GH-B05' && gateB.to === 'GH-C01');
check('noWrongGateBPromotion', result.policy.gateB.edgeId === 'edge-b05-c01' && result.policy.coreControl.notGateB === true);
check('gateBSourceIsAg42', gateB?.source?.result.includes('physics-gate-b05-c01.json#/source'));

const excludedExpected = ['edge-a03-s01', 'edge-b05-s04', 'edge-b03-h01'];
check('excludedDynamicSet', JSON.stringify(result.graph.excludedActiveWork.map(e => e.edgeId).sort()) === JSON.stringify(excludedExpected.sort()));
check('excludedDynamicStatuses', result.graph.edges.filter(e => !e.includedInGraph).every(e => e.status === 'excluded/not_evaluated_active_work' && e.evidenceLevel === 'excluded_active_work_ag38'));
check('noExcludedEdgeInRoutes', result.routes.every(r => r.segments.every(s => !excludedExpected.includes(s.edgeId))));

check('gateAPolicy', JSON.stringify(result.policy.gateA.openRequiresOnly) === JSON.stringify(['hive_power']));
check('gateBPolicy', JSON.stringify(result.policy.gateB.openRequiresOnly) === JSON.stringify(['hive_lockdown_completed']) && result.policy.gateB.choiceIndependent && result.policy.gateB.npcIndependent);
check('forbiddenChoiceNpcInputs', result.policy.gateB.forbiddenPrerequisites.includes('baizhi_choice') && result.policy.gateB.forbiddenPrerequisites.includes('npc_available'));
check('policyMatrixCount', result.policy.matrix.length === 12);
check('policyMatrixTruth', result.policy.matrix.every(row => row.canReachC01FromB05 === row.hive_lockdown_completed && row.choiceOrNpcAffectsGateB === false));

const routesByLabel = new Map(result.routes.map(r => [r.label, r]));
const legal = routesByLabel.get('initial_to_exit_shortest_legal_unresolved');
const recommended = routesByLabel.get('recommended_baizhi_route');
const exploration = routesByLabel.get('completed_static_stair_office_exploration_scope');
check('routeCount', result.routes.length === 3);
for (const route of result.routes) {
  check(`routeHasDijkstra:${route.label}`, route.algorithm.includes('Dijkstra') && route.segments.length > 0 && route.status === 'complete_candidate_over_included_graph');
  let total = 0;
  for (let i = 0; i < route.segments.length; i += 1) {
    const segment = route.segments[i];
    check(`segmentFields:${route.label}:${i}`, segment.edgeId && segment.direction && segment.from && segment.to && Number.isFinite(segment.lengthM) && Number.isFinite(segment.cumulativeM) && segment.sourcePointer);
    total += segment.lengthM;
    check(`segmentCumulative:${route.label}:${i}`, approx(segment.cumulativeM, Number(total.toFixed(3)), 0.001));
  }
  check(`routeTotal:${route.label}`, approx(route.totalM, Number(total.toFixed(3)), 0.001));
}
check('legalRouteLength', approx(legal?.totalM, 252.092, 0.001));
check('recommendedRouteLength', approx(recommended?.totalM, 338.35, 0.001));
check('explorationRouteLength', approx(exploration?.totalM, 490.316, 0.001));
check('legalRouteExit', legal?.segments.at(-1)?.to === 'GH-C04' && legal.choiceState === 'unresolved');
check('recommendedIncludesBaizhi', recommended?.segments.some(s => s.to === 'GH-B04') && recommended?.segments.some(s => s.from === 'GH-B04'));
check('explorationIncludesOfficeAndStair', exploration?.segments.some(s => s.to === 'GH-S02') && exploration?.segments.some(s => s.to === 'GH-S03'));
check('explorationPartial', result.completeness.routeStatus.completed_static_stair_office_exploration_scope === 'partial_due_to_excluded_dynamic_elevators' && result.completeness.excludedDynamicEdgesAffectExploration === true);

const comparisons = result.ag23Comparisons;
check('ag23LegalMatch', approx(comparisons.legalNoChoiceShortest.candidateM, 252.092, 0.001) && approx(comparisons.legalNoChoiceShortest.ag23M, 252.092, 0.001) && comparisons.legalNoChoiceShortest.deltaM === 0);
check('ag23RecommendedMatch', approx(comparisons.recommendedStory.candidateM, 338.35, 0.001) && approx(comparisons.recommendedStory.ag23M, 338.35, 0.001) && comparisons.recommendedStory.deltaM === 0);
check('ag23ExplorationExplained', comparisons.exploration.candidateM === 490.316 && comparisons.exploration.ag23M === 786.783 && comparisons.exploration.deltaM === -296.467 && comparisons.exploration.explanation.includes('excluded'));
check('formalGatesRemainBlocked', result.completeness.gateC === 'blocked/no_go' && result.completeness.gateI === 'blocked/no_go' && result.completeness.architectureGate === 'blocked/no_go');
check('auditGuards', result.audit.trueGateBGuard.includes('edge-b05-c01 only') && result.audit.noAg38OutputRead && result.audit.noNewEdgesInvented && result.audit.noFormalNavmeshClaim);

const report = {
  schema: 'continuous_grey_hive_global_nav_candidate_validator_ag43_v1',
  allChecksPassed: failures.length === 0,
  checks: {
    schemaAndStatus: failures.every(f => !['schema', 'status', 'machinePass'].includes(f.name)),
    graphAttribution: failures.every(f => !['graphCounts', 'directedArcCount', 'allEdgesHaveAttribution', 'allEdgesHavePolicyFields'].includes(f.name)),
    gateScopeAndDualLength: failures.every(f => !f.name.includes('core') && !f.name.includes('GateB') && !f.name.includes('gateBSource') && !f.name.includes('WrongGateB')),
    excludedActiveWork: failures.every(f => !f.name.includes('excluded') && !f.name.includes('Excluded')),
    policyMatrix: failures.every(f => !f.name.includes('Policy') && !f.name.includes('policy') && !f.name.includes('forbidden') && !f.name.includes('Matrix')),
    routes: failures.every(f => !f.name.includes('route') && !f.name.includes('Route') && !f.name.includes('segment') && !f.name.includes('Segment')),
    ag23Comparison: failures.every(f => !f.name.includes('ag23') && !f.name.includes('Ag23')),
    formalBoundary: failures.every(f => !f.name.includes('formal') && !f.name.includes('audit'))
  },
  failures
};
console.log(JSON.stringify(report, null, 2));
if (failures.length) process.exitCode = 1;
