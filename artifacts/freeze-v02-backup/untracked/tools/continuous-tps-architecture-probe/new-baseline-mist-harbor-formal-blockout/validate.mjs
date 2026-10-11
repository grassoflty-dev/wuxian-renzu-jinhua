import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const planPath = path.join(root, 'docs', 'MIST_HARBOR_FORMAL_BLOCKOUT_REQUIREMENTS.md');
const matrixPath = path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-mist-harbor-formal-blockout', 'matrix.json');
const plan = fs.readFileSync(planPath, 'utf8');
const matrix = JSON.parse(fs.readFileSync(matrixPath, 'utf8'));
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const requiredFields = ['id','kind','scope','priority','source','preconditions','expected','evidence','automation','status','blockingDeps'];
const expectedFields = ['topology','authority','persistence','transition','failure','retry','accessibility'];
const ids = matrix.items.map((x) => x.id);
const sourceIds = new Set(matrix.sourceCatalog.map((x) => x.id));
const itemIds = new Set(ids);

check('world_id', matrix.worldId === 'mist_harbor' && plan.includes('worldId = mist_harbor'), 'formal world ID');
check('sources', matrix.sourceCatalog.length >= 4 && matrix.sourceCatalog.every((s) => s.id && s.path && s.title && s.priority && s.confidence && s.conflictResolution) && plan.includes('S1') && plan.includes('S2') && plan.includes('S3'), 'source/confidence/conflict matrix');
check('minimum_items', matrix.items.length >= 20, 'at least 20 positive requirements');
check('minimum_negatives', matrix.negativeCases.length >= 8, 'at least 8 negative cases');
check('unique_ids', ids.length === itemIds.size && matrix.negativeCases.every((x) => x.id && !itemIds.has(x.id)), 'stable unique IDs');
check('item_fields', matrix.items.every((x) => requiredFields.every((k) => Object.hasOwn(x, k)) && expectedFields.every((k) => Object.hasOwn(x.expected, k))), 'required matrix fields');
check('source_refs', matrix.items.every((x) => x.source.every((s) => sourceIds.has(s))), 'source references close');
check('deps_close', matrix.items.every((x) => x.blockingDeps.every((d) => typeof d === 'string' && d.length > 0)), 'blocking dependencies named');
check('continuous_contract', matrix.worldContract.continuousWorld === true && matrix.worldContract.coordinateUnit === 'meter_committed' && matrix.worldContract.formalGrid === false && ['Rust','Babylon','HTML/CSS/JS','Tauri'].every((x) => plan.includes(x)), 'Continuous World authority/presentation contract');
check('no_grid', matrix.worldContract.formalGrid === false && ['正式模型禁止 Grid','禁止 Grid','no-Grid','Grid'].every((x) => plan.includes(x)), 'formal model excludes Grid');
check('status_boundary', matrix.items.every((x) => ['requirements_ready','proposal','pending_design_decision'].includes(x.status)) && !plan.includes('implemented=true') && !plan.includes('approved=true') && !plan.includes('runtimeExecuted=true'), 'requirements remain unimplemented');
check('topology_kinds', ['world_boundary','zone','area','connector','door','trigger','checkpoint','objective','mechanic','encounter','interaction','navigation','capability','transition','accessibility','release_gate'].every((k) => matrix.items.some((x) => x.kind === k)), 'world element coverage');
check('route_semantics', ['primary','optional','shortcut','failure','retry','extraction','transition'].every((x) => plan.includes(x)), 'route and transition semantics');
check('authority_persistence', ['Rust 负责','Babylon','Save v3','presentation 写 quest/door/save'].every((x) => plan.includes(x)), 'authority/persistence boundary');
check('negative_targets', matrix.negativeCases.every((x) => itemIds.has(x.target)), 'negative cases target known IDs');
check('proposal_boundary', plan.includes('proposal') && plan.includes('pending_design_decision') && plan.includes('不得伪装为 committed'), 'proposal/committed boundary');
check('grey_hive_comparison', plan.includes('Compared metrics with Grey Hive') && plan.includes('不复制 Grey Hive 数值'), 'comparable metric framework');
check('unresolved', (plan.match(/^\d+\. /gm) || []).length >= 10 && plan.includes('Unresolved design decisions'), 'unresolved decisions only');
check('media_flags', matrix.flags?.mediaBytesRead === false && matrix.flags?.mediaCreated === false && matrix.flags?.mediaModelCalled === false && matrix.flags?.network === false, 'media/model/network flags');
check('forbidden_scope', !plan.includes('TP-15C3') && !plan.includes('TP-18D1') && !plan.includes('TP-01R11') && !plan.includes('TP-18D2'), 'forbidden TP references absent');

const result = {
  schema: 'new_baseline_mist_harbor_formal_blockout_nb45_v1',
  status: 'mist_harbor_blockout_requirements_ready_not_implemented',
  conclusion: 'mist_harbor_blockout_requirements_ready_not_implemented',
  formalPlayableAccepted: false,
  runtimeExecuted: false,
  worldDataCreated: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  itemCount: matrix.items.length,
  negativeCount: matrix.negativeCases.length,
  committedCount: matrix.items.filter((x) => x.status === 'requirements_ready').length,
  proposedCount: matrix.items.filter((x) => x.status === 'proposal').length,
  pendingDesignDecisionCount: matrix.items.filter((x) => x.status === 'pending_design_decision').length,
  candidateSpaceElementCount: matrix.items.filter((x) => ['zone','area','connector','door','trigger','checkpoint'].includes(x.kind)).length,
  unresolvedDecisionCount: 10,
  checks,
};
process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
if (checks.some((x) => !x.ok)) process.exitCode = 1;
