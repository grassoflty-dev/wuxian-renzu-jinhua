import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, '').replace(/^([A-Za-z]):/, '$1:'));
const root = path.resolve(here, '..', '..', '..');
const readJson = (p) => JSON.parse(fs.readFileSync(path.join(root, p), 'utf8'));
const positive = readJson('tools/continuous-tps-architecture-probe/grey-hive-scene-export/synthetic-positive-fixture.json');
const negative = readJson('tools/continuous-tps-architecture-probe/grey-hive-scene-export/v7-candidate-negative-fixture.json');
const trueGateB = readJson('tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json');

const STAGES = ['blender-export', 'babylon-import', 'rapier-nav-bake'];
const CHECKPOINTS = ['gh_cp_airlock', 'gh_cp_power', 'gh_cp_central', 'gh_cp_post_bio', 'gh_cp_pre_sentinel', 'gh_cp_exit'];
const RESUME_CHECKPOINTS = CHECKPOINTS.filter((id) => id !== 'gh_cp_exit');
const REQUIRED_NEGATIVE_CODES = ['E_SOURCE_SCENE_HASH_MISSING', 'E_COLLIDER_BAKE_MISSING', 'E_NAVMESH_BAKE_MISSING', 'E_DYNAMIC_MOTION_MISSING', 'E_CAMERA_BOUNDS_MISSING', 'E_CHECKPOINT_TRANSFORM_MISSING', 'E_CHECKPOINT_NAV_REF_MISSING', 'E_CHECKPOINT_CLEARANCE_MISSING'];

const canonical = (value) => {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
};
const sha256 = (value) => crypto.createHash('sha256').update(canonical(value)).digest('hex');
const stageHash = (stage) => sha256({ stage: stage.stage, inputHash: stage.inputHash, payload: stage.payload });
const finite = (n) => typeof n === 'number' && Number.isFinite(n);
const vector = (v) => Array.isArray(v) && v.length === 3 && v.every(finite);
const bounds = (b) => b && vector(b.min) && vector(b.max) && b.min.every((n, i) => n <= b.max[i]);
const inside = (point, box) => vector(point) && bounds(box) && point.every((n, i) => n >= box.min[i] && n <= box.max[i]);
const overlaps = (a, b) => bounds(a) && bounds(b) && a.min.every((n, i) => n <= b.max[i] && a.max[i] >= b.min[i]);

function coordinateLock(scene) {
  const m = scene.metadata || {};
  return { unitsMeters: m.unitsMeters, rightHanded: m.rightHanded, upAxis: m.upAxis, footAnchor: m.footAnchor, pivotPolicy: m.pivotPolicy || 'world_origin', scale: m.scale ?? 1 };
}

function idsAndCounts(scene) {
  const doors = (scene.connectors || []).map((c) => c.door?.id).filter(Boolean);
  const roomIds = (scene.rooms || []).map((x) => x.id);
  const connectorIds = (scene.connectors || []).map((x) => x.id);
  const checkpointIds = (scene.checkpoints || []).map((x) => x.id);
  const staticColliderIds = (scene.colliders?.static || []).map((x) => x.id);
  const dynamicColliderIds = (scene.colliders?.dynamic || []).map((x) => x.id);
  const navLinkIds = (scene.navmesh?.navLinks || []).map((x) => x.id);
  const cameraIds = (scene.camera?.volumes || []).map((x) => x.id);
  return {
    entityIds: { rooms: roomIds, connectors: connectorIds, doors, checkpoints: checkpointIds, staticColliders: staticColliderIds, dynamicColliders: dynamicColliderIds, navLinks: navLinkIds, cameraVolumes: cameraIds },
    counts: { rooms: roomIds.length, connectors: connectorIds.length, doors: doors.length, checkpoints: checkpointIds.length, staticColliders: staticColliderIds.length, dynamicColliders: dynamicColliderIds.length, navLinks: navLinkIds.length, cameraVolumes: cameraIds.length }
  };
}

function sceneSummary(scene) {
  const { entityIds, counts } = idsAndCounts(scene);
  return {
    coordinateLock: coordinateLock(scene),
    entityIds,
    counts,
    rooms: scene.rooms || [],
    connectors: scene.connectors || [],
    doors: (scene.connectors || []).map((c) => ({ connectorId: c.id, door: c.door || null })),
    checkpoints: scene.checkpoints || [],
    colliders: scene.colliders || { static: [], dynamic: [] },
    navmesh: scene.navmesh || null,
    cameraVolumes: scene.camera?.volumes || []
  };
}

function makeChain(scene) {
  const summary = sceneSummary(scene);
  const sourceHash = scene.metadata?.sourceSceneSha256 || null;
  const blenderPayload = { coordinateLock: summary.coordinateLock, entityIds: summary.entityIds, counts: summary.counts, rooms: summary.rooms, connectors: summary.connectors, doors: summary.doors, checkpoints: summary.checkpoints, provenance: { fixture: scene.fixture || 'unknown', candidateOnly: scene.candidateOnly === true, schemaOnly: String(scene.fixture || '').includes('synthetic') } };
  const blender = { stage: STAGES[0], inputHash: sourceHash, payload: blenderPayload };
  blender.outputHash = stageHash(blender);
  const babylonPayload = { coordinateLock: summary.coordinateLock, entityIds: summary.entityIds, counts: summary.counts, bounds: summary.rooms.map((r) => ({ id: r.id, bounds: r.bounds || null })), cameraVolumes: summary.cameraVolumes, checkpointTransforms: summary.checkpoints.map((c) => ({ id: c.id, roomId: c.roomId, footTransform: c.footTransform || null, navPolygonId: c.navPolygonId || null })) };
  const babylon = { stage: STAGES[1], inputHash: blender.outputHash, payload: babylonPayload };
  babylon.outputHash = stageHash(babylon);
  const rapierPayload = { coordinateLock: summary.coordinateLock, entityIds: summary.entityIds, counts: summary.counts, colliders: summary.colliders, navmesh: summary.navmesh, dynamicMotion: summary.connectors.filter((c) => c.id === 'edge-b05-c01').map((c) => ({ connectorId: c.id, motionProfile: c.motionProfile || null })), cameraVolumes: summary.cameraVolumes, checkpoints: summary.checkpoints };
  const rapier = { stage: STAGES[2], inputHash: babylon.outputHash, payload: rapierPayload };
  rapier.outputHash = stageHash(rapier);
  return { sourceSceneSha256: sourceHash, coordinateLock: summary.coordinateLock, stages: [blender, babylon, rapier] };
}

function validateScene(scene) {
  const out = [];
  const add = (code, stage, p, detail) => out.push({ code, stage, path: p, detail });
  const meta = scene.metadata || {};
  const schemaOnly = scene.fixture === 'synthetic-positive-schema-only' || scene.schemaOnly === true;
  if (!/^[0-9a-f]{64}$/.test(meta.sourceSceneSha256 || '')) add('E_SOURCE_SCENE_HASH_MISSING', 'blender-export', '/metadata/sourceSceneSha256', 'source scene hash is required and cannot be inferred from v7');
  if (meta.unitsMeters !== 'meters' || meta.scale !== undefined && meta.scale !== 1) add('E_UNIT_SCALE_DRIFT', 'blender-export', '/metadata/unitsMeters', 'pipeline is locked to meters and scale=1');
  if (meta.rightHanded !== true) add('E_HANDEDNESS_DRIFT', 'blender-export', '/metadata/rightHanded', 'pipeline is locked to right-handed coordinates');
  if (meta.upAxis !== '+Y') add('E_UP_AXIS_DRIFT', 'blender-export', '/metadata/upAxis', 'pipeline is locked to +Y');
  if (meta.footAnchor !== 'capsule_foot') add('E_FOOT_ANCHOR_DRIFT', 'blender-export', '/metadata/footAnchor', 'pipeline is locked to capsule_foot');
  if (meta.pivotPolicy !== undefined && meta.pivotPolicy !== 'world_origin') add('E_PIVOT_DRIFT', 'blender-export', '/metadata/pivotPolicy', 'pipeline is locked to world_origin');
  if (scene.candidateOnly === true) add('E_FORMAL_CANDIDATE_NOT_READY', 'blender-export', '/candidateOnly', 'candidate projection cannot be promoted');

  const roomIds = new Set();
  const connectorIds = new Set();
  const polygonIds = new Set();
  const allIds = new Map();
  const remember = (kind, id, p) => {
    if (!id) return;
    if (allIds.has(id)) add('E_ID_DUPLICATE', 'blender-export', p, `${kind} ${id} repeats ${allIds.get(id)}`);
    else allIds.set(id, p);
  };
  const rooms = Array.isArray(scene.rooms) ? scene.rooms : [];
  rooms.forEach((room, i) => {
    remember('room', room.id, `/rooms/${i}/id`);
    roomIds.add(room.id);
    if (!bounds(room.bounds)) add('E_NAVMESH_INCONSISTENT', 'babylon-import', `/rooms/${i}/bounds`, 'room bounds are missing or non-finite');
  });
  const connectors = Array.isArray(scene.connectors) ? scene.connectors : [];
  connectors.forEach((c, i) => {
    connectorIds.add(c.id);
    remember('connector', c.id, `/connectors/${i}/id`);
    if (!Array.isArray(c.endpoints) || c.endpoints.some((id) => !roomIds.has(id))) add('E_ID_UNKNOWN', 'blender-export', `/connectors/${i}/endpoints`, 'connector endpoint room is unknown');
    if (!Array.isArray(c.centerline) || !c.legalBounds || c.centerline.some((point) => !inside(point, c.legalBounds))) add('E_CENTERLINE_OUT_OF_BOUNDS', 'blender-export', `/connectors/${i}/centerline`, 'connector centerline leaves legal bounds');
    remember('door', c.door?.id, `/connectors/${i}/door/id`);
    if (c.id === 'edge-b05-c01' || ['dynamic', 'elevator'].includes(c.type)) {
      const motion = c.motionProfile;
      if (!motion || !['axis', 'startM', 'endM', 'speedMps', 'accelerationMps2', 'durationS', 'states'].every((k) => motion[k] !== undefined) || !Array.isArray(motion.states) || motion.states.length < 2) add('E_DYNAMIC_MOTION_MISSING', 'rapier-nav-bake', `/connectors/${i}/motionProfile`, 'dynamic motion and states are required');
    }
  });
  const staticColliders = scene.colliders?.static;
  const dynamicColliders = scene.colliders?.dynamic;
  if (!Array.isArray(staticColliders) || staticColliders.length === 0) add('E_COLLIDER_BAKE_MISSING', 'rapier-nav-bake', '/colliders/static', 'static collider bake is missing');
  (staticColliders || []).forEach((c, i) => { remember('static collider', c.id, `/colliders/static/${i}/id`); if (!bounds(c.bounds)) add('E_NONFINITE', 'rapier-nav-bake', `/colliders/static/${i}/bounds`, 'collider bounds are not finite'); });
  if (!Array.isArray(dynamicColliders) || dynamicColliders.length === 0) add('E_DYNAMIC_MOTION_MISSING', 'rapier-nav-bake', '/colliders/dynamic', 'dynamic swept collider is missing');
  (dynamicColliders || []).forEach((c, i) => { remember('dynamic collider', c.id, `/colliders/dynamic/${i}/id`); if (c.swept !== true || !bounds(c.bounds)) add('E_DYNAMIC_MOTION_MISSING', 'rapier-nav-bake', `/colliders/dynamic/${i}`, 'dynamic collider must include swept bounds'); });

  const nav = scene.navmesh;
  if (!nav || nav.baked !== true || !/^[0-9a-f]{64}$/.test(nav.hash || '') || !bounds(nav.bounds) || !Array.isArray(nav.polygons) || nav.polygons.length === 0) add('E_NAVMESH_BAKE_MISSING', 'rapier-nav-bake', '/navmesh', 'baked navmesh/hash/bounds/polygons are missing');
  (nav?.polygons || []).forEach((poly, i) => {
    polygonIds.add(poly.id);
    remember('nav polygon', poly.id, `/navmesh/polygons/${i}/id`);
    if (!roomIds.has(poly.roomId)) add('E_ID_UNKNOWN', 'rapier-nav-bake', `/navmesh/polygons/${i}/roomId`, 'nav polygon room is unknown');
    if (!bounds(poly.bounds) || (bounds(nav?.bounds) && (!inside(poly.bounds.min, nav.bounds) || !inside(poly.bounds.max, nav.bounds)))) add('E_COLLIDER_NAV_BOUNDS_MISMATCH', 'rapier-nav-bake', `/navmesh/polygons/${i}/bounds`, 'nav polygon leaves baked nav bounds');
  });
  (nav?.navLinks || []).forEach((link, i) => {
    remember('nav link', link.id, `/navmesh/navLinks/${i}/id`);
    if (!polygonIds.has(link.from) || !polygonIds.has(link.to) || !connectorIds.has(link.connectorId)) add('E_ID_UNKNOWN', 'rapier-nav-bake', `/navmesh/navLinks/${i}`, 'nav link endpoint or connector is unknown');
  });

  const volumes = scene.camera?.volumes;
  if (!Array.isArray(volumes) || volumes.length === 0) add('E_CAMERA_BOUNDS_MISSING', 'babylon-import', '/camera/volumes', 'camera containment is missing');
  (volumes || []).forEach((v, i) => {
    remember('camera volume', v.id, `/camera/volumes/${i}/id`);
    if (!roomIds.has(v.roomId) || !bounds(v.bounds)) add('E_CAMERA_BOUNDS_MISSING', 'babylon-import', `/camera/volumes/${i}`, 'camera volume requires finite bounds and a room');
    if ((staticColliders || []).some((c) => c.solid && overlaps(v.bounds, c.bounds))) add('E_CAMERA_WALL_PENETRATION', 'babylon-import', `/camera/volumes/${i}/bounds`, 'camera volume crosses a solid wall');
  });

  const checkpoints = Array.isArray(scene.checkpoints) ? scene.checkpoints : [];
  const checkpointIds = new Set(checkpoints.map((c) => c.id));
  CHECKPOINTS.forEach((id) => { if (!checkpointIds.has(id)) add('E_CHECKPOINT_TRANSFORM_MISSING', 'babylon-import', '/checkpoints', `canonical checkpoint ${id} is missing`); });
  checkpoints.forEach((c, i) => {
    remember('checkpoint', c.id, `/checkpoints/${i}/id`);
    if (c.id === 'gh_cp_exit') {
      if (c.map !== null || c.noSpawn !== true || c.footTransform !== null) add('E_CHECKPOINT_TRANSFORM_MISSING', 'babylon-import', `/checkpoints/${i}`, 'terminal exit must be map=null and noSpawn=true without a transform');
      return;
    }
    if (!c.footTransform || !vector(c.footTransform.positionM) || !finite(c.footTransform.yawRad)) add('E_CHECKPOINT_TRANSFORM_MISSING', 'babylon-import', `/checkpoints/${i}/footTransform`, 'checkpoint transform must be finite metres');
    if (!c.navPolygonId || !polygonIds.has(c.navPolygonId)) add('E_CHECKPOINT_NAV_REF_MISSING', 'rapier-nav-bake', `/checkpoints/${i}/navPolygonId`, 'checkpoint nav polygon reference is missing');
    if (!finite(c.capsuleClearanceM) || c.capsuleClearanceM <= 0) add('E_CHECKPOINT_CLEARANCE_MISSING', 'rapier-nav-bake', `/checkpoints/${i}/capsuleClearanceM`, 'checkpoint capsule clearance must be positive');
    if ((staticColliders || []).some((col) => col.solid && inside(c.footTransform?.positionM, col.bounds)) || (dynamicColliders || []).some((col) => col.swept && inside(c.footTransform?.positionM, col.bounds))) add('E_CHECKPOINT_BLOCKED', 'rapier-nav-bake', `/checkpoints/${i}`, 'checkpoint overlaps blocker or dynamic sweep');
    if (!c.provenance) add('E_PROVENANCE_MISSING', 'blender-export', `/checkpoints/${i}/provenance`, 'checkpoint provenance is required');
  });
  if (scene.checkpoints?.find((c) => c.id === 'gh_cp_post_bio')?.roomId !== 'GH-B05') add('E_CHECKPOINT_ROOM_MISMATCH', 'babylon-import', '/checkpoints/gh_cp_post_bio/roomId', 'post-bio checkpoint must remain in GH-B05');
  if (scene.policy?.gateA?.id !== 'hive_power' || JSON.stringify(scene.policy?.gateA?.prerequisites) !== JSON.stringify(['hive_power'])) add('E_GATE_POLICY_MISMATCH', 'blender-export', '/policy/gateA', 'Gate A must be hive_power only');
  if (scene.policy?.gateB?.connectorId !== 'edge-b05-c01' || JSON.stringify(scene.policy?.gateB?.prerequisites) !== JSON.stringify(['hive_lockdown_completed'])) add('E_GATE_POLICY_MISMATCH', 'blender-export', '/policy/gateB', 'true Gate B is edge-b05-c01 and requires hive_lockdown_completed only');
  if (trueGateB.source?.connector_id !== 'edge-b05-c01' || JSON.stringify(trueGateB.source?.policy?.open_requires) !== JSON.stringify(['hive_lockdown_completed'])) add('E_GATE_POLICY_SOURCE_MISMATCH', 'blender-export', '/policy/gateB', 'AG-42 true Gate B source facts disagree');
  return out;
}

function validateChain(scene, chain) {
  const out = validateScene(scene);
  const add = (code, stage, p, detail) => out.push({ code, stage, path: p, detail });
  if (!chain || !Array.isArray(chain.stages)) return [...out, { code: 'E_STAGE_MISSING', stage: 'pipeline', path: '/stages', detail: 'stage array is missing' }];
  if (chain.stages.length !== STAGES.length) add('E_STAGE_MISSING', 'pipeline', '/stages', `expected ${STAGES.length} stages`);
  const names = chain.stages.map((s) => s.stage);
  if (JSON.stringify(names) !== JSON.stringify(STAGES)) add('E_STAGE_ORDER', 'pipeline', '/stages', `expected order ${STAGES.join(' -> ')}`);
  const expectedCoordinate = { unitsMeters: 'meters', rightHanded: true, upAxis: '+Y', footAnchor: 'capsule_foot', pivotPolicy: 'world_origin', scale: 1 };
  if (JSON.stringify(chain.coordinateLock) !== JSON.stringify(expectedCoordinate)) add('E_COORDINATE_LOCK', 'pipeline', '/coordinateLock', 'coordinate lock is not meters/right-handed/+Y/world-origin/capsule-foot');
  const sourceHash = scene.metadata?.sourceSceneSha256 || null;
  if (chain.sourceSceneSha256 !== sourceHash) add('E_HASH_MISMATCH', 'blender-export', '/sourceSceneSha256', 'chain source hash differs from scene metadata');
  for (let i = 0; i < chain.stages.length; i += 1) {
    const stage = chain.stages[i];
    const expectedInput = i === 0 ? sourceHash : chain.stages[i - 1]?.outputHash;
    if (stage.inputHash !== expectedInput) add('E_HASH_CHAIN_BROKEN', stage.stage || 'pipeline', `/stages/${i}/inputHash`, 'stage input hash does not equal prior output');
    if (stage.coordinateLock && JSON.stringify(stage.coordinateLock) !== JSON.stringify(expectedCoordinate)) add('E_COORDINATE_LOCK', stage.stage || 'pipeline', `/stages/${i}/payload/coordinateLock`, 'stage coordinate lock drifted');
    if (stage.payload && stage.outputHash !== stageHash(stage)) add('E_HASH_MISMATCH', stage.stage || 'pipeline', `/stages/${i}/outputHash`, 'stage output hash does not match canonical payload');
  }
  const reference = chain.stages[0]?.payload;
  chain.stages.slice(1).forEach((stage, i) => {
    if (JSON.stringify(stage.payload?.entityIds) !== JSON.stringify(reference?.entityIds)) add('E_ID_DRIFT', stage.stage, `/stages/${i + 1}/payload/entityIds`, 'entity IDs changed between stages');
    if (JSON.stringify(stage.payload?.counts) !== JSON.stringify(reference?.counts)) add('E_COUNT_DRIFT', stage.stage, `/stages/${i + 1}/payload/counts`, 'entity counts changed between stages');
  });
  return out;
}

const clone = (value) => structuredClone(value);
const cases = [];
function tamper(id, mutate, expectedCode, targetStage) {
  const scene = clone(positive);
  let chain = makeChain(scene);
  mutate(scene, chain);
  if (id === 'hash-tamper' || id === 'stage-missing' || id === 'stage-reorder') {
    // These mutations intentionally target the chain after its valid construction.
  } else if (id !== 'hash-tamper') chain = makeChain(scene);
  const actual = validateChain(scene, chain);
  cases.push({ id, expectedCode, targetStage, pass: actual.some((e) => e.code === expectedCode && (!targetStage || e.stage === targetStage)), errorCount: actual.length, errors: actual });
}

const positiveChain = makeChain(positive);
const positiveErrors = validateChain(positive, positiveChain);
const negativeChain = makeChain(negative);
const negativeErrors = validateChain(negative, negativeChain);

tamper('hash-tamper', (_scene, chain) => { chain.stages[0].outputHash = 'f'.repeat(64); }, 'E_HASH_MISMATCH', 'blender-export');
tamper('stage-missing', (_scene, chain) => { chain.stages.splice(1, 1); }, 'E_STAGE_MISSING', 'pipeline');
tamper('stage-reorder', (_scene, chain) => { [chain.stages[0], chain.stages[1]] = [chain.stages[1], chain.stages[0]]; }, 'E_STAGE_ORDER', 'pipeline');
tamper('unit-scale', (scene) => { scene.metadata.unitsMeters = 'centimeters'; scene.metadata.scale = 100; }, 'E_UNIT_SCALE_DRIFT', 'blender-export');
tamper('handedness-up-pivot-flip', (scene) => { scene.metadata.rightHanded = false; scene.metadata.upAxis = '-Y'; scene.metadata.pivotPolicy = 'asset_center'; }, 'E_HANDEDNESS_DRIFT', 'blender-export');
tamper('duplicate-unknown-id', (scene) => { scene.rooms[1].id = scene.rooms[0].id; scene.connectors[0].endpoints[1] = 'GH-UNKNOWN'; }, 'E_ID_DUPLICATE', 'blender-export');
tamper('nonfinite', (scene) => { scene.checkpoints[0].footTransform.positionM[0] = Infinity; }, 'E_CHECKPOINT_TRANSFORM_MISSING', 'babylon-import');
tamper('collider-nav-bounds-mismatch', (scene) => { scene.navmesh.polygons[0].bounds.max[0] = 999; }, 'E_COLLIDER_NAV_BOUNDS_MISMATCH', 'rapier-nav-bake');
tamper('camera-wall-crossing', (scene) => { scene.camera.volumes[0].bounds = clone(scene.colliders.static[0].bounds); }, 'E_CAMERA_WALL_PENETRATION', 'babylon-import');
tamper('dynamic-motion-missing', (scene) => { delete scene.connectors.find((c) => c.id === 'edge-b05-c01').motionProfile; scene.colliders.dynamic[0].swept = false; }, 'E_DYNAMIC_MOTION_MISSING', 'rapier-nav-bake');
tamper('checkpoint-blocker-overlap', (scene) => { scene.checkpoints[0].footTransform.positionM = [100, 0, 100]; }, 'E_CHECKPOINT_BLOCKED', 'rapier-nav-bake');

const requiredNegativeCodes = REQUIRED_NEGATIVE_CODES.filter((code) => !new Set(negativeErrors.map((e) => e.code)).has(code));
const failedCases = cases.filter((c) => !c.pass);
const result = {
  schema: 'formal_scene_exporter_tp09b1_v1',
  status: positiveErrors.length === 0 && negativeErrors.length > 0 && requiredNegativeCodes.length === 0 && failedCases.length === 0 ? 'passed_pipeline_contract_only' : 'failed_pipeline_contract',
  pass: positiveErrors.length === 0 && negativeErrors.length > 0 && requiredNegativeCodes.length === 0 && failedCases.length === 0,
  formalSceneReady: false,
  schemaOnly: true,
  syntheticOnly: true,
  stageCount: 3,
  positive: { fixture: positive.fixture, chainPass: positiveErrors.length === 0, errorCount: positiveErrors.length, sourceHash: positive.metadata.sourceSceneSha256, stageHashes: positiveChain.stages.map((s) => ({ stage: s.stage, inputHash: s.inputHash, outputHash: s.outputHash })) },
  v7Negative: { fixture: negative.fixture, candidateOnly: true, expectedFailure: negativeErrors.length > 0, formalSceneReady: false, errorCount: negativeErrors.length, requiredCodes: REQUIRED_NEGATIVE_CODES, missingRequiredCodes: requiredNegativeCodes, errorCodes: [...new Set(negativeErrors.map((e) => e.code))] },
  tamperCases: { count: cases.length, passed: cases.filter((c) => c.pass).length, failed: failedCases.length, cases: cases.map((c) => ({ id: c.id, expectedCode: c.expectedCode, targetStage: c.targetStage, pass: c.pass, errorCount: c.errorCount, observedCodes: [...new Set(c.errors.map((e) => e.code))] })) },
  coordinateLock: { unitsMeters: 'meters', rightHanded: true, upAxis: '+Y', footAnchor: 'capsule_foot', pivotPolicy: 'world_origin', scale: 1 },
  invariantAudit: { hashChain: 'sourceSceneSha256 -> blenderExportHash -> babylonImportHash -> rapierNavBakeHash', idAndCountFrozen: true, dynamicConnector: 'edge-b05-c01 motion+swept required', postBioRoom: 'GH-B05', trueGateB: 'edge-b05-c01', exit: 'map=null noSpawn=true', baizhiChoice: ['taken', 'left', 'unresolved'] },
  futureExecutableWhitelist: [
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/blender-export-executable/**',
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/babylon-import-executable/**',
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/rapier-nav-bake-executable/**',
    'docs/CONTINUOUS_FORMAL_SCENE_EXPORT_PIPELINE_REVIEW.md'
  ],
  rollback: 'surgical removal of futureExecutableWhitelist only; leave schema contracts, candidate evidence, runtime, UI, media, and existing reports untouched',
  formalGates: { gateC: 'blocked', gateI: 'blocked', saveResolver: 'blocked', architecture: 'no_go' }
};
console.log(JSON.stringify(result, null, 2));
process.exit(result.pass ? 0 : 1);
