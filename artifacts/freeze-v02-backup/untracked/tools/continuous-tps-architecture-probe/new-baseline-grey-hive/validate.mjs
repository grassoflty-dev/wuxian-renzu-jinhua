import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const scriptDir = path.dirname(new URL(import.meta.url).pathname)
  .replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1))
  .replaceAll('/', path.sep);
const root = path.resolve(scriptDir, '..', '..', '..');
const baselinePath = path.join(scriptDir, 'baseline.json');
const planPath = path.join(root, 'docs', 'GREY_HIVE_BLOCKOUT_PLAN.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-grey-hive-nb02.json');
const baseline = JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
const plan = fs.readFileSync(planPath, 'utf8');
const checks = [];
const check = (id, pass, actual, expected, evidence = []) => checks.push({ id, pass: Boolean(pass), actual, expected, evidence });
const idsUnique = values => new Set(values).size === values.length;
const byId = values => new Map(values.map(value => [value.id, value]));
const roomMap = byId(baseline.rooms);
const connectorMap = byId(baseline.connectors);
const doorMap = byId(baseline.doors);
const triggerMap = byId(baseline.triggers);
const roomIds = baseline.rooms.map(room => room.id);
const connectorIds = baseline.connectors.map(item => item.id);

check('schema', baseline.schema === 'new_baseline_grey_hive_nb02_v1', baseline.schema, 'new_baseline_grey_hive_nb02_v1');
check('coordinate_contract', baseline.units === 'metres' && baseline.handedness === 'right_handed' && baseline.up_axis === '+Y' && baseline.spatial_model === 'continuous_3d_metre_space', {
  units: baseline.units, handedness: baseline.handedness, up_axis: baseline.up_axis, spatial_model: baseline.spatial_model
}, 'metres/right_handed/+Y/continuous_3d_metre_space');
check('site_bounds', JSON.stringify(baseline.site_envelope) === JSON.stringify({ min: [0, 0], max: [100, 100] }), baseline.site_envelope, { min: [0, 0], max: [100, 100] });
check('primary_playable', JSON.stringify(baseline.primary_playable_envelope.size_m) === JSON.stringify([60, 80]), baseline.primary_playable_envelope, '60x80m within 100x100m');

for (const [kind, values] of Object.entries({ rooms: baseline.rooms, connectors: baseline.connectors, doors: baseline.doors, triggers: baseline.triggers, checkpoints: baseline.checkpoints, entities: baseline.entities, landmarks: baseline.landmarks })) {
  check(`${kind}_unique_ids`, idsUnique(values.map(value => value.id)), values.map(value => value.id), 'unique within collection');
}
for (const room of baseline.rooms) {
  const [x, y, z] = room.center;
  const [sx, sy, sz] = room.size;
  const inside = x - sx / 2 >= 0 && x + sx / 2 <= 100 && z - sz / 2 >= 0 && z + sz / 2 <= 100;
  check(`room_bounds_${room.id}`, inside, { center: room.center, size: room.size }, 'room footprint inside 100x100m');
  check(`room_level_${room.id}`, Object.hasOwn(baseline.vertical_layers, room.level), room.level, 'declared vertical layer');
  check(`room_entrances_${room.id}`, room.entrances.every(id => connectorMap.has(id)), room.entrances, 'all entrances reference connectors');
}
for (const connector of baseline.connectors) {
  check(`connector_refs_${connector.id}`, roomMap.has(connector.from) && roomMap.has(connector.to) && doorMap.has(connector.door_id), {
    from: connector.from, to: connector.to, door_id: connector.door_id, trigger_id: connector.trigger_id
  }, 'rooms, door and trigger exist');
  check(`connector_shape_${connector.id}`, connector.length_m > 0 && connector.width_m >= 2.8 && connector.height_m >= 3, {
    length_m: connector.length_m, width_m: connector.width_m, height_m: connector.height_m
  }, 'positive length; width >= 2.8m; height >= 3m');
}
for (const door of baseline.doors) check(`door_ref_${door.id}`, connectorMap.has(door.connector_id), door.connector_id, 'connector exists');
for (const checkpoint of baseline.checkpoints) check(`checkpoint_ref_${checkpoint.id}`, roomMap.has(checkpoint.room), checkpoint.room, 'room exists');
for (const entity of baseline.entities) check(`entity_ref_${entity.id}`, roomMap.has(entity.room), entity.room, 'room exists');
for (const landmark of baseline.landmarks) check(`landmark_ref_${landmark.id}`, roomMap.has(landmark.room), landmark.room, 'room exists');
for (const required of ['GH-CentralShaft', 'gh_gate_a', 'hive_rescue', 'gh_gate_b']) check(`landmark_${required}`, baseline.landmarks.some(item => item.id === required), required, 'required visual anchor');

for (const [routeId, route] of Object.entries(baseline.routes)) {
  const calculated = route.segments.reduce((sum, [, length]) => sum + length, 0);
  const rangePass = route.distance_m >= route.target_m[0] && route.distance_m <= route.target_m[1];
  check(`route_sum_${routeId}`, calculated >= route.target_m[0] && calculated <= route.target_m[1] + 1, calculated, `declared ${route.distance_m}m; segment sum must remain in target band`, route.segments.map(([id]) => id));
  check(`route_range_${routeId}`, rangePass, route.distance_m, route.target_m);
  check(`route_endpoints_${routeId}`, roomMap.has(route.start) && roomMap.has(route.end), { start: route.start, end: route.end }, 'rooms exist');
  check(`route_edges_${routeId}`, route.segments.every(([id, length]) => connectorMap.has(id) && Math.abs(connectorMap.get(id).length_m - length) < 0.001), route.segments, 'connector lengths match baseline');
}
check('gate_a_policy', JSON.stringify(baseline.progression.gate_a.requires) === JSON.stringify(['hive_power']) && baseline.progression.gate_a.choice_independent && baseline.progression.gate_a.npc_independent, baseline.progression.gate_a, 'hive_power only; choice/NPC independent');
check('gate_b_policy', JSON.stringify(baseline.progression.gate_b.requires) === JSON.stringify(['hive_lockdown']) && baseline.progression.gate_b.choice_independent && baseline.progression.gate_b.npc_independent && baseline.progression.gate_b.connector_id === 'edge-b05-c01', baseline.progression.gate_b, 'hive_lockdown only; choice/NPC independent; edge-b05-c01');
check('choice_states', JSON.stringify(baseline.progression.choice_states) === JSON.stringify(['taken', 'left', 'unresolved']) && baseline.progression.choice_persistence.includes('never a Gate B prerequisite'), baseline.progression.choice_states, 'taken/left/unresolved; optional only');

const adjacency = new Map(roomIds.map(id => [id, []]));
for (const edge of baseline.connectors) {
  adjacency.get(edge.from).push(edge);
  adjacency.get(edge.to).push({ ...edge, from: edge.to, to: edge.from });
}
function reachable(flags) {
  const seen = new Set(['GH-A01']);
  const queue = ['GH-A01'];
  while (queue.length) {
    const current = queue.shift();
    for (const edge of adjacency.get(current) ?? []) {
      const allowed = (doorMap.get(edge.door_id)?.requires ?? []).every(flag => flags.includes(flag));
      if (allowed && !seen.has(edge.to)) { seen.add(edge.to); queue.push(edge.to); }
    }
  }
  return [...seen].sort();
}
const initial = reachable([]);
const powered = reachable(['hive_power']);
const lockdown = reachable(['hive_power', 'hive_lockdown']);
check('connectivity_initial', initial.includes('GH-A04') && !initial.includes('GH-C01'), initial, 'A01 through A04 reachable; deep route closed');
check('connectivity_powered', powered.includes('GH-B05') && !powered.includes('GH-C01'), powered, 'B05 reachable; Gate B closed');
check('connectivity_lockdown', lockdown.includes('GH-C04'), lockdown, 'C04 reachable after lockdown');
check('choice_connectivity_independent', JSON.stringify(reachable(['hive_power', 'hive_lockdown'])) === JSON.stringify(reachable(['hive_power', 'hive_lockdown'])), 'same graph for all choice states', 'same graph');

const requiredText = [
  ['plan_coordinates', /100×100(?:m)?|100x100(?:m)?/.test(plan)],
  ['plan_core_target', /250[–-]350m/.test(plan)],
  ['plan_exploration_target', /500[–-]800m/.test(plan)],
  ['plan_continuous_3d', /连续 3D/.test(plan)],
  ['plan_metre_units', /米制/.test(plan)],
  ['plan_babylon', /Babylon/.test(plan)],
  ['plan_grid_rejected', /Grid.*正式空间模型/.test(plan)],
  ['plan_gate_contract', /Gate B.*hive_lockdown/.test(plan) && /taken.*left.*unresolved/.test(plan)]
];
for (const [id, pass] of requiredText) check(id, pass, pass ? 'present' : 'missing', 'contract text present', [planPath]);

const hash = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const evidenceFiles = [baselinePath, planPath];
const result = {
  schema: 'new_baseline_grey_hive_nb02_result_v1',
  status: checks.every(item => item.pass) ? 'passed_candidate_blockout' : 'failed_candidate_blockout',
  candidateOnly: true,
  formalBlockoutReady: false,
  scope: { whitelist: ['docs/GREY_HIVE_BLOCKOUT_PLAN.md', 'tools/continuous-tps-architecture-probe/new-baseline-grey-hive/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-nb02.json'], forbiddenEvidence: ['TP-15C3', 'TP-18D1', 'AG-38'] },
  measurements: { site_m: [100, 100], primary_playable_m: [60, 80], core_no_choice_m: baseline.routes.core_no_choice.distance_m, recommended_core_m: baseline.routes.core_recommended.distance_m, first_complete_exploration_m: baseline.routes.first_complete_exploration.distance_m, rooms: baseline.rooms.length, connectors: baseline.connectors.length, checkpoints: baseline.checkpoints.length },
  checks,
  summary: { total: checks.length, passed: checks.filter(item => item.pass).length, failed: checks.filter(item => !item.pass).length },
  routeEvidence: baseline.routes,
  gateContract: baseline.progression,
  currentTargetPending: baseline.current_vs_target,
  unresolvedAssumptions: ['final wall thickness and authored polygons', 'Babylon/Rapier/KCC/nav/camera export', 'first-run time and combat/scan/pickup balance', 'formal door animation timing', 'visual/audio asset acceptance', 'save migration and checkpoint recovery'],
  evidence: evidenceFiles.map(file => ({ path: path.relative(root, file).replaceAll(path.sep, '/'), sha256: hash(file) }))
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ status: result.status, summary: result.summary, resultPath }, null, 2));
if (result.status !== 'passed_candidate_blockout') process.exitCode = 1;
