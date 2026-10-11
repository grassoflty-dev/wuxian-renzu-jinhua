import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const planPath = path.join(root, 'docs', 'CHARACTER_FORMAL_IMPLEMENTATION_DAG.md');
const matrixPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-character-formal-implementation-dag', 'matrix.json');
const plan = fs.readFileSync(planPath, 'utf8');
const matrix = JSON.parse(fs.readFileSync(matrixPath, 'utf8'));
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const roles = ['CEN_YAO', 'BAI_ZHI', 'SENTINEL'];
const forbiddenStatuses = new Set(['generated', 'approved', 'implemented', 'verified']);
const allowedCurrent = new Set(['requirements_ready', 'awaiting_user_generation', 'not_started']);
const stage = (name) => matrix.nodeCatalog.find((n) => n.stage === name);
const rolePrefix = { CEN_YAO: 'CY', BAI_ZHI: 'BZ', SENTINEL: 'SE' };
const expandedNodeId = (role, templateId) => `${rolePrefix[role]}-${templateId.slice('ROLE-'.length)}`;

check('source_specs', matrix.sourceSpecifications.every((p) => plan.includes(p)), 'all declared source specifications are cited');
check('roles', roles.every((r) => matrix.roles.some((x) => x.role === r)), 'three independent role plans');
check('node_contract', matrix.nodeCatalog.every((n) => ['nodeId','owner','inputs','outputs','prerequisiteNodes','parallelGroup','approval','evidence','failureRollback','currentStatus','sourceSpecifications'].every((k) => Object.hasOwn(n, k))), 'node contract fields');
check('required_stages', ['image_requirements','user_local_generation','manual_placement','intake_metadata','human_concept_approval','modeling','topology_uv','material','rig','animation','lod_collider_sockets','production_gate','runtime_metadata','registry_review','license_evidence_review','packaging_content_gate','packaging_compliance_gate','babylon_runtime_smoke','public_release'].every((s) => matrix.nodeCatalog.some((n) => n.stage === s)), 'complete formal stage set');
check('role_expansion', roles.every((r) => { const plan = matrix.roles.find((x) => x.role === r); return plan.nodeIds.length === matrix.nodeCatalog.length && matrix.nodeCatalog.every((n) => plan.nodeIds.includes(expandedNodeId(r, n.nodeId))); }), 'every role expands the complete DAG');
check('closed_refs', matrix.nodeCatalog.every((n) => n.prerequisiteNodes.every((p) => matrix.nodeCatalog.some((x) => x.nodeId === p) || p.startsWith('SHARED-'))), 'all prerequisite references close');
check('acyclic', (() => { const graph = new Map(matrix.nodeCatalog.map((n) => [n.nodeId, n.prerequisiteNodes.filter((p) => matrix.nodeCatalog.some((x) => x.nodeId === p))])); const state = new Map(); const visit = (id) => { if (state.get(id) === 1) return false; if (state.get(id) === 2) return true; state.set(id, 1); for (const p of graph.get(id) || []) if (!visit(p)) return false; state.set(id, 2); return true; }; return [...graph.keys()].every(visit); })(), 'dependency graph has no cycle');
check('current_conservative', matrix.nodeCatalog.every((n) => allowedCurrent.has(n.currentStatus) && !forbiddenStatuses.has(n.currentStatus)), 'no generated/approved/implemented/verified current status');
check('user_boundary', ['user_local_generation','manual_placement'].every((s) => matrix.nodeCatalog.find((n) => n.stage === s).owner === 'user_local') && plan.includes('没有生成') && plan.includes('cannot substitute for these actions'), 'user-owned generation and placement boundary');
check('parallel_serial', plan.includes('CONCEPT-ALL') && plan.includes('REGISTRY-LICENSE-ROLE') && plan.includes('PACKAGING-DOUBLE-GATE') && plan.includes('Material and rig are parallel'), 'parallel groups and serial gates');
check('packaging_double_gate', stage('packaging_content_gate').prerequisiteNodes.length === 2 && stage('packaging_content_gate').prerequisiteNodes.every((p) => ['ROLE-14','ROLE-15'].includes(p)) && plan.includes('two required gates'), 'Registry/License converge before two packaging gates');
check('smoke_order', stage('babylon_runtime_smoke').prerequisiteNodes.includes('ROLE-17'), 'smoke follows metadata/registry/license/package closure');
check('release_order', stage('public_release').prerequisiteNodes.includes('ROLE-18'), 'public release follows smoke');
check('lineage', ['assetId','assetVersion','exactVersion','contentHash','parentAssetId','parentVersion','predecessor/successor','Hash or version drift'].every((x) => plan.includes(x)), 'immutable identity and lineage');
check('placeholder_no_shortcut', ['placeholder', 'cannot shorten this DAG', 'satisfy concept approval', 'pass packaging'].every((x) => plan.includes(x)), 'placeholder policy');
check('recovery', ['concept rejected', 'hash/version drift', 'model superseded', 'license rejected', 'smoke failure'].every((x) => plan.includes(x)), 'rework and rollback paths');
check('independent_approval', plan.includes('No role may satisfy another role') && plan.includes('cannot be reused to approve another role'), 'cross-character approval isolation');
check('media_flags', matrix.flags.mediaBytesRead === false && matrix.flags.mediaCreated === false && matrix.flags.mediaModelCalled === false && matrix.flags.network === false && plan.includes('没有读取图片'), 'media/network/model prohibition');
check('forbidden_scope', !plan.includes('NB-31') && !plan.includes('NB-36') && !plan.includes('TP-18D2'), 'forbidden inputs absent');

const result = {
  schema: 'new_baseline_character_formal_implementation_dag_nb39_v1',
  status: 'character_implementation_dag_ready_current_assets_not_generated',
  conclusion: 'character_implementation_dag_ready_current_assets_not_generated',
  implemented: false,
  roles,
  nodeCount: matrix.roles.length * matrix.nodeCatalog.length,
  edgeCount: matrix.roles.length * matrix.nodeCatalog.reduce((sum, n) => sum + n.prerequisiteNodes.length, 0),
  parallelGroups: matrix.parallelGroups,
  currentBlockers: ['user_local_concept_generation', 'manual_placement', 'human_concept_approval', 'all downstream formal gates'],
  flags: matrix.flags,
  checks,
};
process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
if (checks.some((x) => !x.ok)) process.exitCode = 1;
