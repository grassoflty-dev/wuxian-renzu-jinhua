import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const plan = fs.readFileSync(path.join(root, 'docs', 'THREE_WORLD_ENCOUNTER_AI_AUTHORITY_ACCEPTANCE.md'), 'utf8');
const matrix = JSON.parse(fs.readFileSync(path.join(root, 'tools', 'continuous-tps-architecture-probe', 'new-baseline-three-world-encounter-ai-authority', 'matrix.json'), 'utf8'));
const checks = [];
const check = (id, ok, evidence) => checks.push({ id, ok: Boolean(ok), evidence });
const worlds = ['grey_hive','mist_harbor','clockworks'];
const requiredFields = ['id','world','actorClass','priority','source','preconditions','actionStimulus','expectedAuthority','persistence','presentation','evidence','automation','status','blockingDeps'];
const sourceIds = new Set(matrix.sourceCatalog.map((x) => x.id));
const ids = matrix.items.map((x) => x.id);
const itemIds = new Set(ids);
const byId = (id) => matrix.items.find((x) => x.id === id);

check('source_catalog', matrix.sourceCatalog.length >= 4 && matrix.sourceCatalog.every((x) => x.id && x.path && x.priority && x.confidence && x.conflictResolution), 'sources with priority/confidence/conflict resolution');
check('worlds', worlds.every((w) => matrix.items.filter((x) => x.world === w).length >= 7), 'each world has at least seven positive items');
check('minimum_counts', matrix.items.length >= 32 && matrix.negativeCases.length >= 13, 'positive and negative minimums');
check('unique_ids', ids.length === itemIds.size && matrix.negativeCases.every((x) => x.id && !itemIds.has(x.id)), 'stable unique IDs');
check('item_schema', matrix.items.every((x) => requiredFields.every((k) => Object.hasOwn(x, k))), 'machine matrix fields');
check('source_refs', matrix.items.every((x) => x.source.every((s) => sourceIds.has(s))), 'closed source references');
check('negative_refs', matrix.negativeCases.every((x) => worlds.includes(x.world) && itemIds.has(x.target) && x.category && x.failure && x.expected), 'negative cases close to items/worlds');
check('state_domains', matrix.stateDomains.length >= 13 && ['identity_archetype','perception_detection','alert_suspicion_target','combat_damage','stealth_noise_visibility','dialogue_interaction_quest','defeat_despawn_loot','checkpoint_save_class','presentation_snapshot'].every((x) => matrix.stateDomains.includes(x)), 'authority state domains');
check('ownership', matrix.stateOwnership.rust.includes('all') && matrix.stateOwnership.babylon.includes('only') && plan.includes('所有玩法状态由 Rust 写入') && plan.includes('Babylon 只接收 snapshot'), 'Rust/Babylon/UI/Tauri boundary');
check('ai_transitions', matrix.aiStateMachine.legalTransitions.length >= 12 && matrix.aiStateMachine.legalTransitions.every((x) => /^[a-z]+->[a-z]+$/.test(x)) && matrix.aiStateMachine.status === 'proposal', 'legal AI transitions and pending state names');
check('transactions', ['command/input','hit/query','damage/status','defeat/despawn/loot/world flags','stimulus','detection accumulation','threshold','target/alert propagation','decay/search/reset'].every((x) => Object.values(matrix.transactionContracts).some((v) => v.includes(x))), 'combat and stealth transaction contracts');
check('continuous_perception', matrix.transactionContracts.perception.includes('meter distance') && matrix.transactionContracts.perception.includes('LOS') && matrix.transactionContracts.perception.includes('occlusion') && matrix.transactionContracts.perception.includes('room-zone') && matrix.transactionContracts.perception.includes('Nav'), 'continuous world perception');
check('world_focus', ['Pursuer','Guard','Sentinel','白芷','浓雾','声音','信号','重型敌人','动态机械','时间窗口'].every((x) => plan.includes(x)), 'source-backed three-world focus');
check('gate_b_independence', plan.includes('hive_lockdown') && plan.includes('Gate B 的唯一主线控制条件') && plan.includes('hive_choice') && plan.includes('taken') && plan.includes('left') && plan.includes('unresolved') && plan.includes('不能代替另一项'), 'Gate B/hive_choice/NPC independence');
check('checkpoint_reload', plan.includes('checkpoint') && plan.includes('reload') && plan.includes('旧实体不重复') && plan.includes('掉落不重复'), 'checkpoint/reload minimum acceptance');
check('negative_categories', ['failure_retry','navigation','spawn','interaction','lifecycle','combat','persistence','authority','quest','npc','architecture','transition'].every((x) => matrix.negativeCases.some((n) => n.category === x)), 'negative category coverage');
check('pending_boundary', matrix.items.every((x) => ['requirements_ready','proposal','pending'].includes(x.status)) && plan.includes('未明确的 archetype') && plan.includes('pending'), 'unconfirmed content remains pending');
check('no_grid', plan.includes('Grid') && plan.includes('不能进入正式 AI') && !matrix.items.some((x) => JSON.stringify(x).includes('gridField')), 'no Grid in formal authority');
check('flags', Object.entries(matrix.flags).every(([k,v]) => v === false), 'all implementation/runtime/media/network flags false');
check('forbidden_scope', !plan.includes('TP-15C3') && !plan.includes('TP-18D1') && !plan.includes('TP-01R11') && !plan.includes('TP-18D2'), 'forbidden TP references absent');

const counts = Object.fromEntries(worlds.map((w) => [w, matrix.items.filter((x) => x.world === w).length]));
const result = {
  schema: 'three_world_encounter_ai_authority_nb50_v1',
  status: 'three_world_encounter_ai_authority_acceptance_ready_gate_blocked',
  conclusion: 'three_world_encounter_ai_authority_acceptance_ready_gate_blocked',
  formalEncounterAccepted: false,
  aiImplemented: false,
  combatImplemented: false,
  stealthImplemented: false,
  navIntegrated: false,
  saveIntegrated: false,
  runtimeExecuted: false,
  mediaBytesRead: false,
  mediaCreated: false,
  mediaModelCalled: false,
  network: false,
  itemCount: matrix.items.length,
  negativeCount: matrix.negativeCases.length,
  worldItemCounts: counts,
  requirementsReadyCount: matrix.items.filter((x) => x.status === 'requirements_ready').length,
  proposedCount: matrix.items.filter((x) => x.status === 'proposal').length,
  pendingCount: matrix.items.filter((x) => x.status === 'pending').length,
  pendingProductDecisionCount: 8,
  checks,
};
process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
if (checks.some((x) => !x.ok)) process.exitCode = 1;
