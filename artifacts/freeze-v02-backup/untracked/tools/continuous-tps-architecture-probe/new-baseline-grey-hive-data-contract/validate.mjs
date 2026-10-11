import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const scriptDir = path.dirname(new URL(import.meta.url).pathname).replace(/^\/(?:[A-Za-z]:)/, value => value.slice(1)).replaceAll('/', path.sep);
const root = path.resolve(scriptDir, '..', '..', '..');
const fixturePath = path.join(scriptDir, 'runtime-fixture.json');
const schemaPath = path.join(scriptDir, 'runtime-data-contract.schema.json');
const baselinePath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-grey-hive', 'baseline.json');
const planPath = path.join(root, 'docs', 'GREY_HIVE_BLOCKOUT_PLAN.md');
const contractPath = path.join(root, 'docs', 'GREY_HIVE_RUNTIME_DATA_CONTRACT.md');
const resultPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'results', 'new-baseline-grey-hive-data-contract-nb05.json');
const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
const baseline = JSON.parse(fs.readFileSync(baselinePath, 'utf8'));
const contractText = fs.readFileSync(contractPath, 'utf8');
const planText = fs.readFileSync(planPath, 'utf8');
const errors = (code, detail) => ({ code, detail });
const ids = collection => collection.map(item => item.id);
const mapBy = collection => new Map(collection.map(item => [item.id, item]));
const finiteVec3 = value => Array.isArray(value) && value.length === 3 && value.every(Number.isFinite);
const allUnique = values => new Set(values).size === values.length;
const clone = value => structuredClone(value);
const baselineRooms = mapBy(baseline.rooms);
const baselineConnectors = mapBy(baseline.connectors);
const baselineDoors = mapBy(baseline.doors);
const baselineTriggers = mapBy(baseline.triggers);

function validateContract(data) {
  const out = [];
  if (data.schemaVersion !== 'grey_hive_runtime_data_contract_v1') out.push(errors('E_UNKNOWN_SCHEMA_VERSION', data.schemaVersion));
  if (data.worldId !== 'grey_hive') out.push(errors('E_WORLD_ID', data.worldId));
  if (!data.static || !data.dynamic || !data.presentation) out.push(errors('E_REQUIRED_LAYER_MISSING', 'static/dynamic/presentation'));
  if (!data.static) return out;
  const s = data.static;
  const collections = ['zones', 'rooms', 'connectors', 'doors', 'triggers', 'checkpoints', 'landmarks', 'navHints'];
  const allObjects = [];
  for (const name of collections) {
    if (!Array.isArray(s[name])) { out.push(errors('E_COLLECTION_MISSING', name)); continue; }
    allObjects.push(...s[name].map(item => `${name}:${item.id}`));
    if (!allUnique(ids(s[name]))) out.push(errors('E_DUPLICATE_ID', name));
    for (const item of s[name]) if (typeof item.id !== 'string' || !/^[a-z0-9][a-z0-9._-]{2,63}$|^[A-Z][A-Z0-9-]{2,63}$/.test(item.id)) out.push(errors('E_ID_FORMAT', `${name}:${item.id}`));
  }
  if (!allUnique(allObjects)) out.push(errors('E_GLOBAL_ID_COLLISION', 'typed object IDs collide'));
  const roomMap = mapBy(s.rooms ?? []); const connectorMap = mapBy(s.connectors ?? []); const doorMap = mapBy(s.doors ?? []); const triggerMap = mapBy(s.triggers ?? []); const checkpointMap = mapBy(s.checkpoints ?? []);
  const site = s.space?.sitePlanBounds;
  if (!site || JSON.stringify(site.min) !== JSON.stringify([0, 0]) || JSON.stringify(site.max) !== JSON.stringify([100, 100])) out.push(errors('E_SITE_BOUNDS', site));
  const playable = s.space?.primaryPlayableAabb;
  if (!playable || JSON.stringify(playable.min) !== JSON.stringify([16, -3, 8]) || JSON.stringify(playable.max) !== JSON.stringify([76, 15, 88])) out.push(errors('E_PLAYABLE_BOUNDS', playable));
  if (s.space?.formalGrid !== false || Object.keys(s.space ?? {}).some(key => /^(grid|tile|px|py)$/i.test(key))) out.push(errors('E_GRID_FORMAL_FIELD', Object.keys(s.space ?? {})));
  for (const room of s.rooms ?? []) {
    const [x, y, z] = room.center ?? []; const [hx, hy, hz] = room.halfExtents ?? [];
    // NB-02 room Y is a floor marker, while its candidate size Y is a height; only X/Z is a formal site-plane bound here.
    if (![x, y, z, hx, hy, hz].every(Number.isFinite) || x - hx < 0 || x + hx > 100 || z - hz < 0 || z + hz > 100) out.push(errors('E_OUT_OF_BOUNDS', room.id));
  }
  for (const connector of s.connectors ?? []) {
    if (!roomMap.has(connector.from) || !roomMap.has(connector.to) || !doorMap.has(connector.doorId) || (connector.triggerId !== null && !triggerMap.has(connector.triggerId))) out.push(errors('E_DANGLING_REFERENCE', connector.id));
    if (!Number.isFinite(connector.lengthM) || connector.lengthM <= 0) out.push(errors('E_CONNECTOR_GEOMETRY', connector.id));
  }
  for (const door of s.doors ?? []) if (!connectorMap.has(door.connectorId) || !Array.isArray(door.states) || !door.states.includes('Open')) out.push(errors('E_DANGLING_REFERENCE', `door:${door.id}`));
  for (const trigger of s.triggers ?? []) if (!roomMap.has(trigger.roomId)) out.push(errors('E_DANGLING_REFERENCE', `trigger:${trigger.id}`));
  for (const checkpoint of s.checkpoints ?? []) if (!roomMap.has(checkpoint.roomId)) out.push(errors('E_DANGLING_REFERENCE', `checkpoint:${checkpoint.id}`));
  for (const landmark of s.landmarks ?? []) if (!roomMap.has(landmark.roomId)) out.push(errors('E_DANGLING_REFERENCE', `landmark:${landmark.id}`));
  for (const zone of s.zones ?? []) if (!zone.roomIds?.every(id => roomMap.has(id))) out.push(errors('E_DANGLING_REFERENCE', `zone:${zone.id}`));
  for (const route of Object.values(s.routes ?? {})) {
    if (!roomMap.has(route.start) || !roomMap.has(route.end)) out.push(errors('E_DANGLING_REFERENCE', `route:${route.start}`));
    if (!Array.isArray(route.targetBandM) || route.distanceM < route.targetBandM[0] || route.distanceM > route.targetBandM[1]) out.push(errors('E_ROUTE_TARGET_BAND', route.distanceM));
    if (route.evidence !== 'candidate_target_band') out.push(errors('E_ROUTE_FORMAL_NAVMESH', route.evidence));
  }
  const gateA = doorMap.get('gh_gate_a'); const gateB = doorMap.get('gh_gate_b');
  if (!gateA || JSON.stringify(gateA.requires) !== JSON.stringify(['hive_power'])) out.push(errors('E_GATE_A_POLICY', gateA?.requires));
  if (!gateB || JSON.stringify(gateB.requires) !== JSON.stringify(['hive_lockdown'])) out.push(errors('E_GATE_B_POLICY', gateB?.requires));
  if (!['taken', 'left', 'unresolved'].includes(data.dynamic?.choice)) out.push(errors('E_INVALID_CHOICE', data.dynamic?.choice));
  if (data.dynamic?.choice !== 'unresolved' && data.dynamic?.choiceRequired === true) out.push(errors('E_CHOICE_REQUIRED', data.dynamic.choice));
  if (data.dynamic?.npc?.isGatePrerequisite !== false) out.push(errors('E_NPC_GATE_PREREQUISITE', data.dynamic?.npc));
  if (!data.dynamic || Object.keys(s).some(key => ['worldEpoch', 'flags', 'choice', 'npc'].includes(key))) out.push(errors('E_STATIC_DYNAMIC_MIX', Object.keys(s)));
  if (data.presentation?.babylon?.mayWrite?.length) out.push(errors('E_BABYLON_AUTHORITY', data.presentation.babylon.mayWrite));
  if (!data.presentation?.babylon?.readOnly || !data.presentation?.ui?.readOnly) out.push(errors('E_BABYLON_AUTHORITY', 'presentation must be read-only'));
  if (data.save?.excludedKeys?.some(key => /grid|px|py|tile/i.test(key) === false && key === 'authoritative.flags')) out.push(errors('E_SAVE_AUTHORITY', data.save.excludedKeys));
  return out;
}

const positiveErrors = validateContract(fixture);
const positiveChecks = [
  { id: 'contract_valid', pass: positiveErrors.length === 0, actual: positiveErrors.map(e => e.code), expected: 'no errors' },
  { id: 'room_projection_count', pass: fixture.static.rooms.length === 18 && baseline.rooms.length === 18, actual: [fixture.static.rooms.length, baseline.rooms.length], expected: [18, 18] },
  { id: 'connector_projection_count', pass: fixture.static.connectors.length === 17 && baseline.connectors.length === 17, actual: [fixture.static.connectors.length, baseline.connectors.length], expected: [17, 17] },
  { id: 'checkpoint_projection_count', pass: fixture.static.checkpoints.length === 6 && baseline.checkpoints.length === 6, actual: [fixture.static.checkpoints.length, baseline.checkpoints.length], expected: [6, 6] },
  { id: 'source_room_ids', pass: fixture.static.rooms.every(room => baselineRooms.has(room.id)), actual: ids(fixture.static.rooms), expected: 'all NB-02 room IDs' },
  { id: 'source_connector_ids', pass: fixture.static.connectors.every(edge => baselineConnectors.has(edge.id)), actual: ids(fixture.static.connectors), expected: 'all NB-02 connector IDs' },
  { id: 'source_coordinates', pass: fixture.static.rooms.every(room => JSON.stringify(room.center) === JSON.stringify(baselineRooms.get(room.id).center)), actual: 'all centers match baseline', expected: 'exact source centers' },
  { id: 'gate_contract', pass: JSON.stringify(fixture.static.doors.find(d => d.id === 'gh_gate_b').requires) === JSON.stringify(['hive_lockdown']) && fixture.dynamic.choice === 'unresolved', actual: { gateB: fixture.static.doors.find(d => d.id === 'gh_gate_b').requires, choice: fixture.dynamic.choice }, expected: 'lockdown only; unresolved allowed' },
  { id: 'authority_separation', pass: positiveErrors.length === 0 && fixture.dynamic.owner === 'Rust' && fixture.presentation.babylon.readOnly, actual: { owner: fixture.dynamic.owner, babylonReadOnly: fixture.presentation.babylon.readOnly }, expected: 'Rust authority; Babylon read-only' },
  { id: 'route_evidence', pass: Object.values(fixture.static.routes).every(route => route.evidence === 'candidate_target_band'), actual: Object.values(fixture.static.routes).map(route => ({ distanceM: route.distanceM, evidence: route.evidence, segmentSumM: route.segmentSumM })), expected: 'candidate target-band only' },
  { id: 'text_contract', pass: /Rust.*权威|Rust.*authoritative/i.test(contractText) && /Babylon.*只读|Babylon.*read-only/i.test(contractText) && /Grid/.test(contractText) && /candidate_target_band/.test(contractText) && /Gate B/.test(contractText), actual: 'required contract terms present', expected: 'authority, presentation, no Grid, route evidence, Gate B' },
  { id: 'nb02_plan_boundary', pass: /候选正式 blockout/.test(planText) && /不是已交付的 Babylon/.test(planText), actual: 'NB-02 boundary retained', expected: 'candidate-only boundary' }
];

const mutations = [
  ['duplicate_id', value => { value.static.rooms.push(clone(value.static.rooms[0])); return 'E_DUPLICATE_ID'; }],
  ['dangling_reference', value => { value.static.connectors[0].from = 'GH-MISSING'; return 'E_DANGLING_REFERENCE'; }],
  ['out_of_bounds', value => { value.static.rooms[0].center[0] = 101; return 'E_OUT_OF_BOUNDS'; }],
  ['wrong_gate_b_dependency', value => { value.static.doors.find(d => d.id === 'gh_gate_b').requires = ['hive_choice']; return 'E_GATE_B_POLICY'; }],
  ['choice_required', value => { value.dynamic.choiceRequired = true; value.dynamic.choice = 'taken'; return 'E_CHOICE_REQUIRED'; }],
  ['invalid_choice', value => { value.dynamic.choice = 'maybe'; return 'E_INVALID_CHOICE'; }],
  ['babylon_overwrites_authority', value => { value.presentation.babylon.mayWrite = ['dynamic.flags']; return 'E_BABYLON_AUTHORITY'; }],
  ['grid_formal_field', value => { value.static.space.grid = { x: 1, y: 1 }; return 'E_GRID_FORMAL_FIELD'; }],
  ['route_claims_navmesh', value => { value.static.routes.coreNoChoice.evidence = 'formal_walkable_polygon'; return 'E_ROUTE_FORMAL_NAVMESH'; }],
  ['unknown_schema_version', value => { value.schemaVersion = 'grey_hive_runtime_data_contract_v99'; return 'E_UNKNOWN_SCHEMA_VERSION'; }],
  ['static_dynamic_mix', value => { value.static.choice = 'unresolved'; return 'E_STATIC_DYNAMIC_MIX'; },
  ],
  ['npc_becomes_gate', value => { value.dynamic.npc.isGatePrerequisite = true; return 'E_NPC_GATE_PREREQUISITE'; }]
];
const negativeChecks = mutations.map(([id, mutate, expected]) => {
  const candidate = clone(fixture); const expectedCode = mutate(candidate); const actual = validateContract(candidate).map(error => error.code);
  return { id, pass: actual.includes(expectedCode), actual, expected: expectedCode };
});

const sha256 = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const sourceFiles = [baselinePath, planPath, contractPath, schemaPath, fixturePath];
const result = {
  schema: 'new_baseline_grey_hive_data_contract_nb05_result_v1',
  status: positiveChecks.every(item => item.pass) && negativeChecks.every(item => item.pass) ? 'passed_candidate_runtime_contract' : 'failed_candidate_runtime_contract',
  candidateOnly: true,
  formalRuntimeReady: false,
  source: sourceFiles.map(file => ({ path: path.relative(root, file).replaceAll(path.sep, '/'), bytes: fs.statSync(file).size, sha256: sha256(file) })),
  projection: { sourceSchema: baseline.schema, rooms: fixture.static.rooms.length, connectors: fixture.static.connectors.length, doors: fixture.static.doors.length, triggers: fixture.static.triggers.length, checkpoints: fixture.static.checkpoints.length, landmarks: fixture.static.landmarks.length, routeEvidence: 'candidate_target_band', routeValuesM: { coreNoChoice: 252.092, recommendedDeclared: 338.35, recommendedSegmentSum: 350.583, firstCompleteExploration: 786.783 } },
  positiveChecks,
  negativeChecks,
  errorCodes: [...new Set(negativeChecks.flatMap(item => item.actual))].sort(),
  summary: { positiveTotal: positiveChecks.length, positivePassed: positiveChecks.filter(item => item.pass).length, negativeTotal: negativeChecks.length, negativeRejected: negativeChecks.filter(item => item.pass).length },
  responsibility: { rust: ['authoritative flags', 'door/trigger/checkpoint resolution', 'Vec3/Transform/Velocity', 'save v3'], babylon: ['read-only scene presentation', 'interpolation/prediction/reconciliation'], ui: ['read-only summary', 'input intent and interaction request'] },
  unresolved: ['formal collider/navmesh/camera export', 'wall thickness and dynamic door motion', 'first-run pacing and combat balance', 'formal source scene hash/import chain', 'Save resolver integration'],
  scope: { allowed: ['docs/GREY_HIVE_RUNTIME_DATA_CONTRACT.md', 'tools/continuous-tps-architecture-probe/new-baseline-grey-hive-data-contract/**', 'tools/continuous-tps-architecture-probe/results/new-baseline-grey-hive-data-contract-nb05.json'], forbiddenConsumed: ['TP-15C3', 'TP-18D1', 'server-rs writes', 'media'] }
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`);
console.log(JSON.stringify({ status: result.status, summary: result.summary, resultPath }, null, 2));
if (result.status !== 'passed_candidate_runtime_contract') process.exitCode = 1;
