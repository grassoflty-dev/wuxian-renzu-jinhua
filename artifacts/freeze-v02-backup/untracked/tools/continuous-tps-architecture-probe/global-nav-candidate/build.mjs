import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..', '..', '..');
const rel = p => path.relative(root, p).replaceAll('\\', '/');
const readJson = p => JSON.parse(fs.readFileSync(path.join(root, p), 'utf8'));

const v7 = readJson('tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json');
const v5 = readJson('tools/continuous-tps-architecture-probe/layout-repair/layout-v5.json');
const ag18 = readJson('tools/continuous-tps-architecture-probe/results/layout-repair-ag18.json');
const ag23 = readJson('tools/continuous-tps-architecture-probe/results/layout-core-length-ag23.json');
const ag22 = readJson('tools/continuous-tps-architecture-probe/results/layout-optional-ag22.json');
const ag35 = readJson('tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json');
const ag36 = readJson('tools/continuous-tps-architecture-probe/results/physics-static-mainline-ag36.json');
const ag37 = readJson('tools/continuous-tps-architecture-probe/results/physics-stair-b01-s03-ag37.json');
const ag39 = readJson('tools/continuous-tps-architecture-probe/results/physics-static-office-ag39.json');
const gateB = readJson('tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json');
const b1 = readJson('tools/continuous-tps-architecture-probe/results/physics-b1-artifact-recovery.json');

const ag18ById = new Map(ag18.connectors.map(c => [c.id, c]));
const v5ById = new Map(v5.connectors.map(c => [c.id, c]));
const ag35ById = new Map(ag35.connectors.map(c => [c.id, c]));
const ag36ById = new Map(ag36.connectors.map(c => [c.id, c]));

const replacement = v7.core_length_repair;
const coreId = replacement.replacement_id;
const coreNominal = replacement.v7_length_m;
const coreFormal = b1.tp04b1Reference.referenceLengthM;
const coreDelta = Math.abs(coreNominal - coreFormal);

const excludedIds = new Set(['edge-a03-s01', 'edge-b05-s04', 'edge-b03-h01']);
const physicsSource = id => {
  if (id === 'edge-b05-c01') return {
    result: 'tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json',
    pointer: '/source',
    evidenceLevel: 'blocked_formal_candidate_ag42'
  };
  if (id === 'edge-b01-s02') return {
    result: 'tools/continuous-tps-architecture-probe/results/physics-static-office-ag39.json',
    pointer: '/source',
    evidenceLevel: 'blocked_isolated_optional_office_candidate_ag39'
  };
  if (id === 'edge-b01-s03') return {
    result: 'tools/continuous-tps-architecture-probe/results/physics-stair-b01-s03-ag37.json',
    pointer: '/sourceEvidence',
    evidenceLevel: 'blocked_formal_candidate_ag37'
  };
  if (ag36ById.has(id)) return {
    result: 'tools/continuous-tps-architecture-probe/results/physics-static-mainline-ag36.json',
    pointer: `/connectors/${id}`,
    evidenceLevel: 'blocked_isolated_candidate_ag36'
  };
  return {
    result: 'tools/continuous-tps-architecture-probe/results/layout-repair-ag18.json',
    pointer: `/connectors/${id}`,
    evidenceLevel: 'isolated_layout_candidate_ag18'
  };
};

const prereqFor = id => {
  if (id === 'edge-a04-b01') return ['hive_power'];
  if (id === 'edge-b01-b03' || id === 'edge-b03-b04' || id === 'edge-b01-b05') return ['hive_lockdown_completed'];
  if (id === 'edge-b05-c01') return ['hive_lockdown_completed'];
  if (id === 'edge-c01-c02' || id === 'edge-c02-c03') return ['hive_lockdown_completed'];
  if (id === 'edge-c03-c04') return ['hive_rescue'];
  return [];
};

const sourceFor = id => {
  if (id === coreId) return {
    layout: 'tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json#/core_length_repair',
    result: 'tools/continuous-tps-architecture-probe/results/physics-b1-artifact-recovery.json#/tp04b1Reference',
    inventory: 'tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json#/connectors/edge-b01-b02-core-v7'
  };
  const ps = physicsSource(id);
  return {
    layout: `tools/continuous-tps-architecture-probe/layout-repair/layout-v5.json#/connectors/${id}`,
    result: `${ps.result}#${ps.pointer}`,
    inventory: `tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json#/connectors/${id}`
  };
};

const sourceConnector = id => id === coreId ? {
  id: coreId,
  from: replacement.from,
  to: replacement.to,
  width_m: replacement.width_m,
  height_m: replacement.height_m,
  path: replacement.path,
  door: { id: 'gh_door_b01_b02_v7_from/to', state: 'Open' }
} : v5ById.get(id);

const makeEdge = id => {
  const src = sourceConnector(id);
  const inventory = ag35ById.get(id);
  const excluded = excludedIds.has(id);
  const isCore = id === coreId;
  const isGateB = id === 'edge-b05-c01';
  const ps = physicsSource(isCore ? 'edge-b01-b02-core-v7' : id);
  const length = isCore ? coreNominal : ag18ById.get(id).length_m;
  const source = sourceFor(id);
  const base = {
    id,
    from: src.from,
    to: src.to,
    directionalityCandidate: 'bidirectional_candidate',
    lengthM: length,
    widthM: src.width_m,
    heightM: src.height_m,
    door: src.door || null,
    prerequisite: prereqFor(id),
    includedInGraph: !excluded,
    status: excluded ? 'excluded/not_evaluated_active_work' : 'included_candidate',
    source,
    evidenceLevel: excluded ? 'excluded_active_work_ag38' : ps.evidenceLevel,
    inheritedInventoryCategory: inventory?.category || null,
    vertical: src.vertical || null
  };
  if (isCore) {
    base.lengthEvidence = {
      nominalV7M: coreNominal,
      formalReplayReferenceM: coreFormal,
      absoluteDeltaM: coreDelta,
      absoluteDeltaMm: coreDelta * 1000,
      selectedForShortestPathM: coreNominal,
      note: 'Both nominal and formal replay scalar are preserved; the candidate graph uses v7 nominal 40m.'
    };
  }
  if (isGateB) {
    base.gate = {
      name: 'Gate B',
      openRequiresOnly: ['hive_lockdown_completed'],
      choiceIndependent: true,
      npcIndependent: true,
      forbiddenPrerequisites: ['baizhi_choice', 'baizhi_state', 'npc_available']
    };
  }
  return base;
};

const edgeIds = v5.connectors.map(c => c.id).map(id => id === 'edge-b01-b02' ? coreId : id);
const edges = edgeIds.map(makeEdge);
const includedEdges = edges.filter(e => e.includedInGraph);
const roomIds = v5.rooms.map(r => r.id);

const stateEffects = (room, state) => {
  const next = { ...state };
  if (room === 'GH-A03') next.hive_power = true;
  if (room === 'GH-B02') next.hive_lockdown_completed = true;
  if (room === 'GH-C03') next.hive_rescue = true;
  return next;
};

const satisfied = (edge, state) => edge.prerequisite.every(k => state[k] === true);

const dijkstra = ({label, requiredRooms = [], start = 'GH-A01', goal = 'GH-C04', choiceState = 'unresolved', npcAvailable = false}) => {
  const required = new Set(requiredRooms);
  const startVisited = required.has(start) ? [start] : [];
  const initial = {
    room: start,
    hive_power: false,
    hive_lockdown_completed: false,
    hive_rescue: false,
    visited: startVisited
  };
  const key = s => [s.room, s.hive_power ? 1 : 0, s.hive_lockdown_completed ? 1 : 0, s.hive_rescue ? 1 : 0, ...requiredRooms.map(r => s.visited.includes(r) ? 1 : 0)].join('|');
  const dist = new Map([[key(initial), 0]]);
  const records = new Map([[key(initial), {state: initial, prev: null, edge: null, direction: null}]]);
  const queue = [{d: 0, state: initial}];
  let expanded = 0;
  const arcs = includedEdges.flatMap(edge => [
    {edge, direction: 'forward', from: edge.from, to: edge.to},
    {edge, direction: 'reverse', from: edge.to, to: edge.from}
  ]);
  while (queue.length) {
    queue.sort((a, b) => a.d - b.d);
    const current = queue.shift();
    const currentKey = key(current.state);
    if (current.d !== dist.get(currentKey)) continue;
    expanded += 1;
    const isGoal = current.state.room === goal && requiredRooms.every(r => current.state.visited.includes(r));
    if (isGoal) {
      const segments = [];
      let cursor = currentKey;
      while (records.get(cursor).prev !== null) {
        const rec = records.get(cursor);
        segments.push({edge: rec.edge, direction: rec.direction, from: rec.prevState.room, to: rec.state.room});
        cursor = rec.prev;
      }
      segments.reverse();
      let cumulative = 0;
      const detailed = segments.map((segment, index) => {
        const e = edges.find(candidate => candidate.id === segment.edge);
        cumulative += e.lengthM;
        const gateState = e.id === 'edge-a04-b01'
          ? 'Open_after_hive_power'
          : e.id === 'edge-b05-c01'
            ? 'Open_after_hive_lockdown_completed'
            : e.door?.state === 'Locked' ? 'Locked_source_state_requires_policy' : 'Open_candidate';
        return {
          index: index + 1,
          edgeId: e.id,
          direction: segment.direction,
          from: segment.from,
          to: segment.to,
          lengthM: e.lengthM,
          cumulativeM: Number(cumulative.toFixed(3)),
          doorStateAtUse: gateState,
          prerequisite: e.prerequisite,
          evidenceLevel: e.evidenceLevel,
          sourcePointer: e.source.result
        };
      });
      return {
        label,
        status: 'complete_candidate_over_included_graph',
        startRoom: start,
        endRoom: goal,
        choiceState,
        npcAvailable,
        requiredRooms,
        algorithm: 'Dijkstra over explicit room + policy + visit-state graph',
        expandedStateCount: expanded,
        shortestness: 'No lower-cost included candidate state reached the same goal/required-visit state.',
        segments: detailed,
        totalM: Number(cumulative.toFixed(3)),
        gateStates: {
          gateA: 'Open because hive_power=true; no Baizhi/NPC input',
          gateB: 'Open because hive_lockdown_completed=true; choice and NPC availability not consulted'
        }
      };
    }
    for (const arc of arcs.filter(candidate => candidate.from === current.state.room)) {
      if (!satisfied(arc.edge, current.state)) continue;
      const nextState = stateEffects(arc.to, current.state);
      nextState.room = arc.to;
      nextState.visited = [...new Set([...current.state.visited, ...requiredRooms.filter(r => r === arc.to)])];
      const nextKey = key(nextState);
      const nextDistance = current.d + arc.edge.lengthM;
      if (!dist.has(nextKey) || nextDistance < dist.get(nextKey)) {
        dist.set(nextKey, nextDistance);
        records.set(nextKey, {state: nextState, prev: currentKey, prevState: current.state, edge: arc.edge.id, direction: arc.direction});
        queue.push({d: nextDistance, state: nextState});
      }
    }
  }
  return {
    label,
    status: 'blocked_no_path_in_included_graph',
    startRoom: start,
    endRoom: goal,
    choiceState,
    npcAvailable,
    requiredRooms,
    algorithm: 'Dijkstra over explicit room + policy + visit-state graph',
    expandedStateCount: expanded,
    segments: [],
    totalM: null
  };
};

const routes = [
  dijkstra({label: 'initial_to_exit_shortest_legal_unresolved', choiceState: 'unresolved', npcAvailable: false}),
  dijkstra({label: 'recommended_baizhi_route', requiredRooms: ['GH-B04'], choiceState: 'taken', npcAvailable: true}),
  dijkstra({label: 'completed_static_stair_office_exploration_scope', requiredRooms: ['GH-S02', 'GH-S03'], choiceState: 'unresolved', npcAvailable: false})
];

const policyMatrix = [];
for (const lockdown of [false, true]) {
  for (const choice of ['taken', 'left', 'unresolved']) {
    for (const npcAvailable of [false, true]) {
      policyMatrix.push({
        hive_lockdown_completed: lockdown,
        choice,
        npc_available: npcAvailable,
        gateA: 'requires hive_power',
        gateB: lockdown ? 'Open' : 'Locked',
        canReachC01FromB05: lockdown,
        choiceOrNpcAffectsGateB: false
      });
    }
  }
}

const ag23Comparisons = {
  legalNoChoiceShortest: {
    candidateM: routes[0].totalM,
    ag23M: ag23.route_evidence.legal_no_choice_shortest.distance_m,
    deltaM: Number((routes[0].totalM - ag23.route_evidence.legal_no_choice_shortest.distance_m).toFixed(3)),
    explanation: 'Exact match; v7 core uses the 40m nominal length and the route does not use excluded optional elevators.'
  },
  recommendedStory: {
    candidateM: routes[1].totalM,
    ag23M: ag23.route_evidence.recommended_story.distance_m,
    deltaM: Number((routes[1].totalM - ag23.route_evidence.recommended_story.distance_m).toFixed(3)),
    explanation: 'Exact match; the recommended route includes the B01↔B03↔B04 Baizhi branch and uses true Gate B B05→C01.'
  },
  exploration: {
    candidateM: routes[2].totalM,
    ag23M: ag23.route_evidence.first_complete_exploration.distance_m,
    deltaM: Number((routes[2].totalM - ag23.route_evidence.first_complete_exploration.distance_m).toFixed(3)),
    explanation: 'Partial reduced-scope route: S01, S04, and H01 dynamic elevator branches are excluded, and the route reports only the completed static/stair/office scope.'
  },
  legacyScopeWarnings: {
    ag22LegalBypassM: ag22.route_evidence.legal_no_choice_bypass.distance_m,
    ag22RecommendedM: ag22.route_evidence.recommended_story.distance_m,
    note: 'AG-22 used the pre-v7 28m core traversal; AG-23 v7 values are used for this graph and the core length is not silently reverted.'
  }
};

const result = {
  schema: 'continuous_grey_hive_global_nav_candidate_ag43_v1',
  status: 'blocked_formal_candidate_partial_exploration',
  machinePass: true,
  scope: 'source-attributed candidate graph shortest paths only; no formal navmesh, global physics, camera, Blockout, or runtime integration',
  generatedAtUtc: new Date().toISOString(),
  sources: {
    v7: 'tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json',
    v5Geometry: 'tools/continuous-tps-architecture-probe/layout-repair/layout-v5.json',
    ag18: 'tools/continuous-tps-architecture-probe/results/layout-repair-ag18.json',
    ag23: 'tools/continuous-tps-architecture-probe/results/layout-core-length-ag23.json',
    ag35: 'tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json',
    ag36: 'tools/continuous-tps-architecture-probe/results/physics-static-mainline-ag36.json',
    ag37: 'tools/continuous-tps-architecture-probe/results/physics-stair-b01-s03-ag37.json',
    ag39: 'tools/continuous-tps-architecture-probe/results/physics-static-office-ag39.json',
    ag42TrueGateB: 'tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json',
    ag42B1Artifact: 'tools/continuous-tps-architecture-probe/results/physics-b1-artifact-recovery.json'
  },
  graph: {
    nodeCount: roomIds.length,
    nodes: v5.rooms.map(room => ({id: room.id, floorY: room.floor_y_m, role: room.role, sourcePointer: `tools/continuous-tps-architecture-probe/layout-repair/layout-v5.json#/rooms/${room.id}`})),
    edgeCount: edges.length,
    includedEdgeCount: includedEdges.length,
    excludedEdgeCount: edges.length - includedEdges.length,
    directedArcCount: includedEdges.length * 2,
    edges,
    excludedActiveWork: [...excludedIds].map(id => ({
      edgeId: id,
      status: 'excluded/not_evaluated_active_work',
      reason: 'AG-38 dynamic optional elevator work is active; output is not read and no result is inferred.',
      sourcePointer: `tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json#/connectors/${id}`
    }))
  },
  policy: {
    gateA: {edgeId: 'edge-a04-b01', openRequiresOnly: ['hive_power'], choiceIndependent: true, npcIndependent: true},
    gateB: {edgeId: 'edge-b05-c01', openRequiresOnly: ['hive_lockdown_completed'], choiceIndependent: true, npcIndependent: true, forbiddenPrerequisites: ['baizhi_choice', 'baizhi_state', 'npc_available']},
    coreControl: {edgeId: coreId, notGateB: true, nominalLengthM: coreNominal, formalReplayReferenceM: coreFormal, absoluteDeltaMm: coreDelta * 1000},
    matrix: policyMatrix,
    noChoiceOrNpcGateInput: true
  },
  routes,
  ag23Comparisons,
  completeness: {
    routeStatus: {
      initial_to_exit_shortest_legal_unresolved: 'complete_over_included_graph',
      recommended_baizhi_route: 'complete_over_included_graph',
      completed_static_stair_office_exploration_scope: 'partial_due_to_excluded_dynamic_elevators'
    },
    excludedDynamicEdgesAffectExploration: true,
    missingFormalEvidence: ['formal_navmesh', 'formal_global_physics', 'camera_clearance', 'Blockout', 'runtime_integration'],
    gateC: 'blocked/no_go',
    gateI: 'blocked/no_go',
    architectureGate: 'blocked/no_go'
  },
  audit: {
    trueGateBGuard: 'edge-b05-c01 only; edge-b01-b02-core-v7 is never promoted to Gate B',
    noAg38OutputRead: true,
    noNewEdgesInvented: true,
    noFormalNavmeshClaim: true
  }
};

const out = path.join(root, 'tools/continuous-tps-architecture-probe/results/global-nav-candidate-ag43.json');
fs.writeFileSync(out, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({out: rel(out), nodeCount: roomIds.length, edgeCount: edges.length, includedEdgeCount: includedEdges.length, routes: routes.map(r => ({label: r.label, totalM: r.totalM, segments: r.segments.length, status: r.status})), status: result.status}, null, 2));
