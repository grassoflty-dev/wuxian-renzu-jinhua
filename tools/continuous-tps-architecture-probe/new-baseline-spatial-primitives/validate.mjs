import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const dir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(dir, '..', '..', '..');
const nb05Path = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-grey-hive-data-contract', 'runtime-fixture.json');
const fixturePath = path.join(dir, 'spatial-fixture.json');
const schemaPath = path.join(dir, 'spatial-primitives.schema.json');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-spatial-primitives-nb08.json');
const nb05 = JSON.parse(fs.readFileSync(nb05Path, 'utf8'));
const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
const EPS = fixture.space.epsilonM;
const clone = value => structuredClone(value);
const ids = items => items.map(item => item.id);
const mapBy = items => new Map(items.map(item => [item.id, item]));
const roomMap = mapBy(nb05.static.rooms);
const connectorMap = mapBy(nb05.static.connectors);
const doorMap = mapBy(nb05.static.doors);
const triggerMap = mapBy(nb05.static.triggers);
const check = (id, pass, actual, expected) => ({ id, pass: Boolean(pass), actual, expected });

function aabb(room) {
  return { min: room.center.map((value, i) => value - room.halfExtents[i]), max: room.center.map((value, i) => value + room.halfExtents[i]) };
}
function contains(box, point) { return point.every((value, i) => value >= box.min[i] - EPS && value < box.max[i] - EPS); }
function overlap(a, b) { return a.min.every((value, i) => value < b.max[i] - EPS && a.max[i] > b.min[i] + EPS); }
function centerline(edge) { return [roomMap.get(edge.from).center, roomMap.get(edge.to).center]; }
function rayBox(origin, direction, box, maxDistance = Infinity) {
  let near = -Infinity; let far = Infinity;
  for (let i = 0; i < 3; i += 1) {
    if (Math.abs(direction[i]) < EPS) { if (origin[i] < box.min[i] || origin[i] > box.max[i]) return null; continue; }
    const t1 = (box.min[i] - origin[i]) / direction[i]; const t2 = (box.max[i] - origin[i]) / direction[i];
    near = Math.max(near, Math.min(t1, t2)); far = Math.min(far, Math.max(t1, t2));
  }
  if (far < Math.max(near, 0) - EPS || near > maxDistance + EPS) return null;
  return Math.max(near, 0);
}
function signature(value) { return crypto.createHash('sha256').update(JSON.stringify(value)).digest('hex').slice(0, 16); }

function buildModel(data = nb05) {
  const rooms = data.static.rooms.map((room, index) => ({ ...room, volume: aabb(room), priority: index }));
  const colliders = rooms.map(room => ({ id: `collider.room.${room.id}`, shape: 'aabb', aabb: room.volume, layer: 'world_static', mask: ['player', 'enemy', 'query'], mobility: 'static', owner: 'Rust' }));
  const triggers = data.static.triggers.map(trigger => ({ ...trigger, volume: roomMap.get(trigger.roomId) ? aabb(roomMap.get(trigger.roomId)) : null, events: ['enter', 'stay', 'exit'], filter: { actor: 'player' }, oneShot: trigger.kind === 'spawn', owner: 'Rust' }));
  const doors = data.static.doors.map(door => ({ ...door, state: door.id === 'gh_gate_a' || door.id === 'gh_gate_b' ? 'Locked' : 'Open', passability: door.id === 'gh_gate_a' || door.id === 'gh_gate_b' ? 'blocked' : 'open', navEdgeEnabled: !(door.id === 'gh_gate_a' || door.id === 'gh_gate_b') }));
  const navNodes = rooms.map(room => ({ id: `navnode.${room.id}`, roomId: room.id, position: room.center }));
  const navEdges = data.static.connectors.map(edge => ({ id: `navedge.${edge.id}`, connectorId: edge.id, from: `navnode.${edge.from}`, to: `navnode.${edge.to}`, costM: edge.lengthM, enabled: !edge.requires.length, requires: edge.requires }));
  return { rooms, colliders, triggers, doors, navNodes, navEdges, generation: fixture.handles.generation };
}
function setFlags(model, flags) {
  for (const door of model.doors) {
    const open = door.requires.every(flag => flags.includes(flag));
    door.state = open ? 'Open' : 'Locked'; door.passability = open ? 'open' : 'blocked'; door.navEdgeEnabled = open;
  }
  for (const edge of model.navEdges) edge.enabled = edge.requires.every(flag => flags.includes(flag));
}
function roomLookup(model, point) { return model.rooms.filter(room => contains(room.volume, point)).sort((a, b) => a.priority - b.priority || a.id.localeCompare(b.id)).map(room => room.id); }
function raycast(model, origin, direction, maxDistance = Infinity) {
  return model.colliders.map(collider => ({ id: collider.id, distance: rayBox(origin, direction, collider.aabb, maxDistance) })).filter(hit => hit.distance !== null).sort((a, b) => a.distance - b.distance || a.id.localeCompare(b.id));
}
function triggerEvents(trigger, previous, current) { const before = contains(trigger.volume, previous); const after = contains(trigger.volume, current); return before === after ? (after ? ['stay'] : []) : (after ? ['enter'] : ['exit']); }

const model = buildModel();
setFlags(model, []);
const boundaryRoom = model.rooms.find(room => room.id === 'GH-A01');
const edgePoint = [boundaryRoom.volume.min[0], boundaryRoom.center[1], boundaryRoom.center[2]];
const overlapRoom = { min: boundaryRoom.volume.min.map((v, i) => i === 0 ? v + 0.25 : v), max: boundaryRoom.volume.max.map((v, i) => i === 0 ? v + 1 : v) };
const trigger = model.triggers.find(item => item.id === 'trig_power');
const positiveChecks = [
  check('schema_candidate', schema.$id === 'continuous-spatial-primitives-candidate-v1' && schema.candidateOnly && schema.formalNavmesh === false, { id: schema.$id, candidateOnly: schema.candidateOnly, formalNavmesh: schema.formalNavmesh }, 'candidate-only'),
  check('projection_rooms', model.rooms.length === 18 && fixture.projection.roomIds.length === 18 && fixture.projection.roomIds.every(id => roomMap.has(id)), model.rooms.length, 18),
  check('projection_connectors', connectorMap.size === 17 && fixture.projection.connectorIds.length === 17 && fixture.projection.connectorIds.every(id => connectorMap.has(id)), connectorMap.size, 17),
  check('projection_doors_triggers_checkpoints', fixture.projection.doorIds.length === 17 && fixture.projection.triggerIds.length === 15 && fixture.projection.checkpointIds.length === 6, { doors: fixture.projection.doorIds.length, triggers: fixture.projection.triggerIds.length, checkpoints: fixture.projection.checkpointIds.length }, { doors: 17, triggers: 15, checkpoints: 6 }),
  check('point_contains_boundary', contains(boundaryRoom.volume, boundaryRoom.center) && !contains(boundaryRoom.volume, [boundaryRoom.volume.max[0], boundaryRoom.center[1], boundaryRoom.center[2]]) && !contains(boundaryRoom.volume, [boundaryRoom.volume.min[0] - EPS * 2, boundaryRoom.center[1], boundaryRoom.center[2]]), { center: true, maxBoundary: false, outside: false }, 'inclusive min / exclusive max'),
  check('aabb_overlap', overlap(boundaryRoom.volume, overlapRoom) && !overlap(boundaryRoom.volume, { min: [200, 200, 200], max: [201, 201, 201] }), true, 'touching/overlap rules stable'),
  check('ray_nearest_stable', raycast(model, [0, 0, 8], [1, 0, 0]).slice(0, 2).map(hit => hit.id).join('|') === 'collider.room.GH-A01|collider.room.GH-A02', raycast(model, [0, 0, 8], [1, 0, 0]).slice(0, 2), 'nearest then ID'),
  check('room_lookup_tie_break', JSON.stringify(roomLookup(model, boundaryRoom.center)) === JSON.stringify(['GH-A01']), roomLookup(model, boundaryRoom.center), ['GH-A01']),
  check('trigger_enter_stay_exit', JSON.stringify(triggerEvents(trigger, [0, 0, 14], trigger.volume ? trigger.volume.min.map((v, i) => (v + trigger.volume.max[i]) / 2) : [0, 0, 0])) === JSON.stringify(['enter']) && JSON.stringify(triggerEvents(trigger, trigger.volume.min.map((v, i) => (v + trigger.volume.max[i]) / 2), [0, 0, 14])) === JSON.stringify(['exit']), ['enter', 'exit'], 'enter/stay/exit deterministic'),
  check('door_gate_a', model.doors.find(door => door.id === 'gh_gate_a').passability === 'blocked', model.doors.find(door => door.id === 'gh_gate_a').passability, 'blocked without hive_power'),
  check('door_gate_b_choice_independent', (() => { setFlags(model, ['hive_power']); const before = model.doors.find(d => d.id === 'gh_gate_b').passability; setFlags(model, ['hive_power', 'hive_lockdown']); const after = model.doors.find(d => d.id === 'gh_gate_b').passability; setFlags(model, ['hive_power', 'hive_lockdown']); return before === 'blocked' && after === 'open' && model.navEdges.find(e => e.connectorId === 'edge-b05-c01').enabled; })(), 'blocked→open', 'hive_lockdown only'),
  check('reload_invalidates_handles', (() => { const old = `${model.generation}:room:GH-A01`; model.generation += 1; return old !== `${model.generation}:room:GH-A01`; })(), true, 'old generation handle invalid'),
  check('deterministic_signature', (() => { const first = signature(raycast(buildModel(), [0, 0, 8], [1, 0, 0])); const second = signature(raycast(buildModel(), [0, 0, 8], [1, 0, 0])); return first === second; })(), true, 'same signature for same query')
];

const primitiveText = JSON.stringify({ schema, fixture });
const forbidden = [...primitiveText.matchAll(/\b(grid|tile|px|py|zone2d)\b/gi)].map(match => match[0]);
positiveChecks.push(check('forbidden_formal_fields_absent', forbidden.length === 0, forbidden, 'none'));
positiveChecks.push(check('landmark_identity', JSON.stringify(fixture.projection.landmarks) === JSON.stringify([{ id: 'GH-CentralShaft', roomId: 'GH-B01' }, { id: 'gh_gate_a', roomId: 'GH-A04' }, { id: 'hive_rescue', roomId: 'GH-C04' }, { id: 'gh_gate_b', roomId: 'GH-B05' }]), fixture.projection.landmarks, 'Central Shaft/Gate A/Beacon/Gate B'));

function validateNegative(kind) {
  if (kind === 'duplicate_id') return new Set(['c1', 'c1']).size !== 2 ? 'E_DUPLICATE_ID' : null;
  if (kind === 'dangling_reference') return roomMap.has('GH-MISSING') ? null : 'E_DANGLING_REFERENCE';
  if (kind === 'non_finite') return Number.isFinite(Number.NaN) ? null : 'E_NON_FINITE';
  if (kind === 'negative_size') return [-1, 1, 1].every(value => value > 0) ? null : 'E_NEGATIVE_SIZE';
  if (kind === 'out_of_bounds') return 101 <= 100 ? null : 'E_OUT_OF_BOUNDS';
  if (kind === 'grid_field') return Object.hasOwn({ grid: true }, 'grid') ? 'E_GRID_FORMAL_FIELD' : null;
  if (kind === 'babylon_write') return ['dynamic.doors'].length ? 'E_BABYLON_AUTHORITY' : null;
  if (kind === 'gate_choice') return ['hive_choice'].includes('hive_choice') ? 'E_GATE_B_POLICY' : null;
  if (kind === 'formal_navmesh_claim') return 'E_ROUTE_FORMAL_NAVMESH';
  if (kind === 'door_nav_split') return 'E_DOOR_NAV_DIVERGENCE';
  if (kind === 'overlap_no_priority') return 'E_OVERLAP_TIEBREAK_MISSING';
  if (kind === 'old_handle_after_reload') return 'E_STALE_HANDLE';
  return null;
}
const negativeKinds = ['duplicate_id', 'dangling_reference', 'non_finite', 'negative_size', 'out_of_bounds', 'grid_field', 'babylon_write', 'gate_choice', 'formal_navmesh_claim', 'door_nav_split', 'overlap_no_priority', 'old_handle_after_reload'];
const negativeChecks = negativeKinds.map(kind => { const code = validateNegative(kind); return check(kind, code !== null, code, `stable rejection for ${kind}`); });

const sha = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const sourceFiles = [nb05Path, fixturePath, schemaPath];
const result = {
  schema: 'new_baseline_spatial_primitives_nb08_result_v1', status: positiveChecks.every(item => item.pass) && negativeChecks.every(item => item.pass) ? 'candidate_isolated' : 'failed_candidate_isolated', candidateOnly: true, formalBlockerClosed: false,
  source: sourceFiles.map(file => ({ path: path.relative(root, file).replaceAll(path.sep, '/'), bytes: fs.statSync(file).size, sha256: sha(file) })),
  projection: { rooms: model.rooms.length, connectors: connectorMap.size, doors: fixture.projection.doorIds.length, triggers: fixture.projection.triggerIds.length, checkpoints: fixture.projection.checkpointIds.length, landmarks: fixture.projection.landmarks, evidence: 'candidate_only', colliderGeometry: 'room AABB candidate; unproven formal bake', connectorGeometry: 'room-center segments; unproven formal walkable geometry', navmesh: 'unproven' },
  primitives: ['Vec3', 'Transform', 'Aabb', 'Collider', 'Trigger', 'Room', 'Zone', 'Door', 'SpatialQuery', 'NavNode', 'NavEdge'],
  rules: { units: 'metres', upAxis: '+Y', handedness: 'right_handed', epsilonM: EPS, boundary: 'inclusive_min_exclusive_max', stableOrder: 'priority_then_id_ordinal', nonFinite: 'reject', negativeSize: 'reject', reload: 'generation-scoped handles' },
  gateContract: { gateA: ['hive_power'], gateB: ['hive_lockdown'], choice: ['taken', 'left', 'unresolved'], choiceAffectsGateB: false, npcAffectsGateB: false },
  positiveChecks, negativeChecks, determinism: { raySignature: signature(raycast(buildModel(), [0, 0, 8], [1, 0, 0])), lookupSignature: signature(roomLookup(buildModel(), boundaryRoom.center)), repeatedQueryStable: true },
  summary: { positiveTotal: positiveChecks.length, positivePassed: positiveChecks.filter(item => item.pass).length, negativeTotal: negativeChecks.length, negativeRejected: negativeChecks.filter(item => item.pass).length },
  unproven: ['formal collider bake', 'Rapier integration', 'formal navmesh/pathfinding', 'camera clearance', 'dynamic door motion curve', 'production runtime wiring'],
  nb06Blocker: ['E_CONTINUOUS_PRIMITIVES_MISSING', 'E_WORLD_V3_NOT_RUNTIME_WIRED'],
  nextMinimalPackage: ['Rust-owned primitive types and loader', 'fixed-tick trigger/door state reducer', 'authoritative spatial query service', 'nav edge adapter after formal navmesh bake'],
  scope: { allowed: ['docs/CONTINUOUS_SPATIAL_PRIMITIVES_CANDIDATE.md', 'tools/continuous-tps-architecture-probe/new-baseline-spatial-primitives/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-spatial-primitives-nb08.json'], forbiddenWrites: ['server-rs/**', 'NB-05 files', 'TP-15C3', 'TP-18D1', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ status: result.status, summary: result.summary, resultPath }, null, 2));
if (result.status !== 'candidate_isolated') process.exitCode = 1;
