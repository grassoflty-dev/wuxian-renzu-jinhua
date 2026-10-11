import fs from 'node:fs';
import path from 'node:path';

const here = path.dirname(new URL(import.meta.url).pathname.replace(/^\//, '').replace(/^([A-Za-z]):/, '$1:'));
const root = path.resolve(here, '..', '..', '..');
const readJson = (p) => JSON.parse(fs.readFileSync(path.join(root, p), 'utf8'));
const contract = readJson('tools/continuous-tps-architecture-probe/grey-hive-scene-export/scene-export-v1.json');
const positive = readJson('tools/continuous-tps-architecture-probe/grey-hive-scene-export/synthetic-positive-fixture.json');
const negative = readJson('tools/continuous-tps-architecture-probe/grey-hive-scene-export/v7-candidate-negative-fixture.json');
const v7 = readJson('tools/continuous-tps-architecture-probe/layout-core-length/layout-v7.json');
const ag31 = readJson('tools/continuous-tps-architecture-probe/results/checkpoint-resolver-ag31.json');
const ag35 = readJson('tools/continuous-tps-architecture-probe/results/connector-inventory-ag35.json');
const ag36 = readJson('tools/continuous-tps-architecture-probe/results/physics-static-mainline-ag36.json');
const ag39 = readJson('tools/continuous-tps-architecture-probe/results/physics-static-office-ag39.json');
const ag42 = readJson('tools/continuous-tps-architecture-probe/results/physics-gate-b05-c01.json');

const REQUIRED_CHECKPOINTS = ['gh_cp_airlock', 'gh_cp_power', 'gh_cp_central', 'gh_cp_post_bio', 'gh_cp_pre_sentinel', 'gh_cp_exit'];
const RESUME_CHECKPOINTS = REQUIRED_CHECKPOINTS.filter((id) => id !== 'gh_cp_exit');
const CHOICE_STATES = ['taken', 'left', 'unresolved'];
const errors = (fixture) => {
  const out = [];
  const add = (code, p, detail) => out.push({ code, path: p, detail });
  const finite = (value) => typeof value === 'number' && Number.isFinite(value);
  const vector = (value) => Array.isArray(value) && value.length === 3 && value.every(finite);
  const bounds = (value) => value && vector(value.min) && vector(value.max) && value.min.every((n, i) => n <= value.max[i]);
  const ids = new Map();
  const remember = (kind, id, p) => {
    if (!id) return;
    if (ids.has(id)) add('E_ID_DUPLICATE', p, `${kind} ${id} already declared at ${ids.get(id)}`);
    else ids.set(id, p);
  };
  const inside = (point, box) => vector(point) && bounds(box) && point.every((n, i) => n >= box.min[i] && n <= box.max[i]);
  const intersects = (a, b) => bounds(a) && bounds(b) && a.min.every((n, i) => n <= b.max[i] && a.max[i] >= b.min[i]);

  if (!fixture.metadata || typeof fixture.metadata !== 'object') add('E_SOURCE_SCENE_HASH_MISSING', '/metadata', 'metadata is absent');
  else if (!/^[0-9a-f]{64}$/.test(fixture.metadata.sourceSceneSha256 || '')) add('E_SOURCE_SCENE_HASH_MISSING', '/metadata/sourceSceneSha256', 'formal exporter must provide a 64-character scene hash');

  const rooms = Array.isArray(fixture.rooms) ? fixture.rooms : [];
  const roomIds = new Set();
  rooms.forEach((room, i) => {
    remember('room', room.id, `/rooms/${i}/id`);
    if (roomIds.has(room.id)) add('E_ID_DUPLICATE', `/rooms/${i}/id`, `room ${room.id} repeats`);
    roomIds.add(room.id);
    if (!bounds(room.bounds) || !finite(room.floorY) || !finite(room.ceilingY)) add('E_NAVMESH_INCONSISTENT', `/rooms/${i}`, 'room bounds/floor/ceiling must be finite metres');
  });

  const staticColliders = fixture.colliders?.static;
  if (!Array.isArray(staticColliders) || staticColliders.length === 0) add('E_COLLIDER_BAKE_MISSING', '/colliders/static', 'no baked static collider records');
  else staticColliders.forEach((c, i) => { remember('collider', c.id, `/colliders/static/${i}/id`); if (!bounds(c.bounds)) add('E_NAVMESH_INCONSISTENT', `/colliders/static/${i}/bounds`, 'collider bounds are not finite'); });
  (fixture.colliders?.dynamic || []).forEach((c, i) => remember('collider', c.id, `/colliders/dynamic/${i}/id`));

  const nav = fixture.navmesh;
  if (!nav || nav.baked !== true || !/^[0-9a-f]{64}$/.test(nav.hash || '') || !bounds(nav.bounds) || !Array.isArray(nav.polygons) || nav.polygons.length === 0) add('E_NAVMESH_BAKE_MISSING', '/navmesh', 'baked navmesh/hash/bounds/polygons are absent');
  const polygonIds = new Set();
  (nav?.polygons || []).forEach((poly, i) => {
    remember('nav polygon', poly.id, `/navmesh/polygons/${i}/id`);
    polygonIds.add(poly.id);
    if (!roomIds.has(poly.roomId) || !bounds(poly.bounds) || (bounds(nav?.bounds) && !inside(poly.bounds.min, nav.bounds) || bounds(nav?.bounds) && !inside(poly.bounds.max, nav.bounds))) add('E_NAVMESH_INCONSISTENT', `/navmesh/polygons/${i}`, 'polygon room or bounds is inconsistent');
  });
  (nav?.navLinks || []).forEach((link, i) => {
    remember('nav link', link.id, `/navmesh/navLinks/${i}/id`);
    if (!polygonIds.has(link.from) || !polygonIds.has(link.to)) add('E_NAVMESH_INCONSISTENT', `/navmesh/navLinks/${i}`, 'nav link endpoint polygon is unknown');
  });

  const connectors = Array.isArray(fixture.connectors) ? fixture.connectors : [];
  const connectorIds = new Set();
  connectors.forEach((connector, i) => {
    const p = `/connectors/${i}`;
    remember('connector', connector.id, `${p}/id`);
    connectorIds.add(connector.id);
    if (!connector.endpoints?.every((id) => roomIds.has(id))) add('E_CONNECTOR_ENDPOINT_UNKNOWN', `${p}/endpoints`, 'both connector endpoint room ids must exist');
    if (!Array.isArray(connector.centerline) || !connector.legalBounds || !connector.centerline.every((point) => inside(point, connector.legalBounds))) add('E_CENTERLINE_OUT_OF_BOUNDS', `${p}/centerline`, 'centerline point leaves connector legal bounds');
    remember('door', connector.door?.id, `${p}/door/id`);
    remember('collider', connector.door?.colliderId, `${p}/door/colliderId`);
    if (connector.id === 'edge-b05-c01' || ['dynamic', 'elevator'].includes(connector.type)) {
      const motion = connector.motionProfile;
      if (!motion || !['axis', 'startM', 'endM', 'speedMps', 'accelerationMps2', 'durationS', 'states'].every((key) => motion[key] !== undefined) || !Array.isArray(motion.states) || motion.states.length < 2) add('E_DYNAMIC_MOTION_MISSING', `${p}/motionProfile`, 'dynamic connector lacks a complete motion profile');
      if (motion && Array.isArray(motion.states) && motion.states.length >= 2 && (!finite(motion.durationS) || motion.durationS <= 0)) add('E_DYNAMIC_SWEEP_INCOMPLETE', `${p}/motionProfile/durationS`, 'dynamic duration must be positive');
    }
  });

  const volumes = fixture.camera?.volumes;
  if (!Array.isArray(volumes) || volumes.length === 0) add('E_CAMERA_BOUNDS_MISSING', '/camera/volumes', 'camera containment volumes are absent');
  else volumes.forEach((volume, i) => {
    if (!roomIds.has(volume.roomId) || !bounds(volume.bounds)) add('E_CAMERA_BOUNDS_MISSING', `/camera/volumes/${i}`, 'camera volume must have a room and finite bounds');
    if ((staticColliders || []).some((collider) => collider.solid && intersects(volume.bounds, collider.bounds))) add('E_CAMERA_WALL_PENETRATION', `/camera/volumes/${i}`, 'camera volume intersects a solid collider');
  });

  const checkpoints = Array.isArray(fixture.checkpoints) ? fixture.checkpoints : [];
  const checkpointById = new Map();
  checkpoints.forEach((checkpoint, i) => {
    remember('checkpoint', checkpoint.id, `/checkpoints/${i}/id`);
    checkpointById.set(checkpoint.id, checkpoint);
    if (checkpoint.id !== 'gh_cp_exit') {
      const transform = checkpoint.footTransform;
      if (!transform || !vector(transform.positionM) || !finite(transform.yawRad)) add('E_CHECKPOINT_TRANSFORM_MISSING', `/checkpoints/${i}/footTransform`, 'resume checkpoint requires a finite metre foot transform');
      if (!checkpoint.navPolygonId || !polygonIds.has(checkpoint.navPolygonId)) add('E_CHECKPOINT_NAV_REF_MISSING', `/checkpoints/${i}/navPolygonId`, 'resume checkpoint requires a baked nav polygon reference');
      if (!finite(checkpoint.capsuleClearanceM) || checkpoint.capsuleClearanceM <= 0) add('E_CHECKPOINT_CLEARANCE_MISSING', `/checkpoints/${i}/capsuleClearanceM`, 'resume checkpoint requires positive capsule clearance');
      const position = transform?.positionM;
      if ((staticColliders || []).some((collider) => collider.solid && inside(position, collider.bounds)) || (fixture.colliders?.dynamic || []).some((collider) => collider.swept && inside(position, collider.bounds))) add('E_CHECKPOINT_BLOCKED', `/checkpoints/${i}`, 'checkpoint is inside a blocker or dynamic sweep');
    } else if (checkpoint.map !== null || checkpoint.noSpawn !== true || checkpoint.footTransform !== null) add('E_CHECKPOINT_TRANSFORM_MISSING', `/checkpoints/${i}`, 'terminal exit must be isolated as map=null with no fabricated spawn');
  });
  REQUIRED_CHECKPOINTS.forEach((id) => { if (!checkpointById.has(id)) add('E_CHECKPOINT_TRANSFORM_MISSING', '/checkpoints', `canonical checkpoint ${id} is absent`); });
  const postBio = checkpointById.get('gh_cp_post_bio');
  if (postBio && postBio.roomId !== 'GH-B05') add('E_NAVMESH_INCONSISTENT', '/checkpoints/gh_cp_post_bio/roomId', 'post-bio checkpoint must remain in GH-B05');

  const policy = fixture.policy;
  if (JSON.stringify(policy?.gateA) !== JSON.stringify({ id: 'hive_power', prerequisites: ['hive_power'] })) add('E_NAVMESH_INCONSISTENT', '/policy/gateA', 'Gate A must be hive_power only');
  if (policy?.gateB?.connectorId !== 'edge-b05-c01' || JSON.stringify(policy.gateB.prerequisites) !== JSON.stringify(['hive_lockdown_completed'])) add('E_NAVMESH_INCONSISTENT', '/policy/gateB', 'Gate B must be edge-b05-c01 and require hive_lockdown_completed only');
  if (JSON.stringify(policy?.choiceStates) !== JSON.stringify(CHOICE_STATES) || policy?.gateB?.choiceIndependent !== true || policy?.gateB?.npcIndependent !== true) add('E_NAVMESH_INCONSISTENT', '/policy', 'choice states and independence policy are not explicit');
  if (!connectorIds.has('edge-b05-c01')) add('E_CONNECTOR_ENDPOINT_UNKNOWN', '/connectors', 'true Gate B connector is absent');
  return out;
};

const positiveErrors = errors(positive);
const negativeErrors = errors(negative);
const requiredNegativeCodes = ['E_SOURCE_SCENE_HASH_MISSING', 'E_COLLIDER_BAKE_MISSING', 'E_NAVMESH_BAKE_MISSING', 'E_DYNAMIC_MOTION_MISSING', 'E_CAMERA_BOUNDS_MISSING', 'E_CHECKPOINT_TRANSFORM_MISSING', 'E_CHECKPOINT_NAV_REF_MISSING', 'E_CHECKPOINT_CLEARANCE_MISSING'];
const negativeCodes = [...new Set(negativeErrors.map((e) => e.code))];
const missingCodes = requiredNegativeCodes.filter((code) => !negativeCodes.includes(code));
const positivePass = positiveErrors.length === 0;
const negativeExpectedFailure = negativeErrors.length > 0 && missingCodes.length === 0;
const result = {
  schema: 'grey_hive_scene_export_tp09a1_v1',
  status: positivePass && negativeExpectedFailure ? 'passed_contract_with_candidate_negative_readiness' : 'failed_contract_validation',
  pass: positivePass && negativeExpectedFailure,
  fixtureCount: 2,
  errorCount: positiveErrors.length + (missingCodes.length ? missingCodes.length : 0),
  positiveFixture: { id: positive.fixture, schemaOnly: true, formalEligible: false, pass: positivePass, errorCount: positiveErrors.length, errors: positiveErrors },
  negativeReadiness: { id: negative.fixture, candidateOnly: true, formalReady: false, expectedFailure: negativeExpectedFailure, errorCount: negativeErrors.length, errors: negativeErrors, requiredCodes: requiredNegativeCodes, missingCodes },
  sourceAudit: {
    v7: { path: 'layout-core-length/layout-v7.json', status: v7.status, candidateOnly: true, promotion: 'forbidden' },
    ag31: { path: 'results/checkpoint-resolver-ag31.json', status: ag31.status, formalTransform: 'missing_blocked' },
    ag35: { path: 'results/connector-inventory-ag35.json', status: ag35.status, connectorCount: ag35.connector_count },
    ag36: { path: 'results/physics-static-mainline-ag36.json', status: ag36.status, connectorCount: ag36.connector_count },
    ag39: { path: 'results/physics-static-office-ag39.json', status: ag39.status, candidateOnly: ag39.source?.candidate_only ?? true },
    ag42: { path: 'results/physics-gate-b05-c01.json', status: ag42.status, connectorId: ag42.source?.connector_id, gateBPrerequisite: ag42.source?.policy?.open_requires }
  },
  policyAudit: { gateAOnly: 'hive_power', gateBOnly: 'edge-b05-c01 -> hive_lockdown_completed', choiceStates: CHOICE_STATES, baizhiAndNpcAreNotGatePrerequisites: true },
  checkpointAudit: { canonicalIds: REQUIRED_CHECKPOINTS, finiteResumeTransforms: RESUME_CHECKPOINTS, postBioRoom: 'GH-B05', terminalExit: 'gh_cp_exit map=null noSpawn=true', choiceIndependent: true },
  formalGate: { gateC: 'blocked', gateI: 'blocked', saveResolver: 'blocked', architecture: 'no_go', formalExporterCreated: false },
  futureExporterWhitelist: [
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/blender-export-v1.json',
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/babylon-import-v1.json',
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/rapier-nav-import-v1.json',
    'tools/continuous-tps-architecture-probe/formal-scene-exporter/validator.mjs',
    'docs/CONTINUOUS_FORMAL_SCENE_EXPORT_PIPELINE_REVIEW.md'
  ],
  rollback: 'surgical removal of futureExporterWhitelist only; no existing runtime/evidence files are mutated'
};
console.log(JSON.stringify(result, null, 2));
process.exit(result.pass ? 0 : 1);
