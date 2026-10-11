import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { evaluateContract, runNegativeFixtures, REQUIRED_CONTROLS, PACKAGE_WRITES } from './contract-gate.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const base = 'tools/continuous-tps-architecture-probe/';
const own = `${base}loot-scan-contract-readiness/`;
const resultPath = `${base}results/loot-scan-contract-readiness-tp15c1.json`;
const reportPath = 'docs/CONTINUOUS_LOOT_SCAN_CONTRACT_READINESS_REVIEW.md';
const proposalPath = `${own}proposed-contract.json`;
const activeIds = ['AG-38', 'TP-01R9', 'TP-18C1'];
const paths = {
  ag12: `${base}results/capability-ag12.json`, ag12Source: `${base}capability/validate.mjs`, ag12Report: 'docs/CONTINUOUS_CAPABILITY_CONTRACT_REVIEW.md',
  r7: `${base}results/formal-capability-hud-readiness-tp01r7.json`, r7Report: 'docs/CONTINUOUS_FORMAL_CAPABILITY_HUD_READINESS_REVIEW.md',
  tp15a: `${base}results/hud-map-view-model-tp15a1.json`, tp15b: `${base}results/tauri-hud-map-tp15b1.json`,
  tp14a: `${base}results/campaign-progression-authority-tp14a1.json`, tp14b: `${base}results/world-runtime-routing-tp14b1.json`, tp14c: `${base}results/tauri-world-routing-tp14c1.json`,
  plan: 'docs/TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md', gateReport: 'docs/CONTINUOUS_TPS_ARCHITECTURE_GATE_REPORT.md',
  world: 'server-rs/src/world_v3.rs', campaign: 'server-rs/src/campaign.rs', main: 'server-rs/src/campaign_main.rs', store: 'server-rs/src/campaign_store.rs', input: 'server-rs/src/continuous_input.rs',
  app: 'server-rs/release-ui/js/app.js', renderer: 'server-rs/release-ui/js/tps-renderer.js', boot: 'server-rs/release-ui/js/renderer-boot.js', html: 'server-rs/release-ui/index.html', css: 'server-rs/release-ui/css/style.css',
  flowTests: 'server-rs/tests/campaign_flow.rs', storeTests: 'server-rs/tests/campaign_store_test.rs', worldTests: 'server-rs/tests/world_v3_contract_test.rs', inputTests: 'server-rs/tests/continuous_input_test.rs',
};
const exactReadonlyInputs = Object.values(paths);
const approvedRead = new Set(exactReadonlyInputs);
const sha = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const relative = (p) => path.relative(root, p).replaceAll('\\', '/');
function canonical(p) {
  if (typeof p !== 'string' || !p || p !== p.trim() || /[:\x00-\x1f]/.test(p)) throw new Error('E_PATH_INVALID');
  const result = p.replaceAll('\\', '/');
  if (result.startsWith('/') || result.split('/').some((x) => !x || x === '.' || x === '..' || /[. ]$/.test(x))) throw new Error('E_PATH_INVALID');
  return result;
}
function assertReadable(p) {
  p = canonical(p);
  if (/(^|\/)(assets|data|media|public)\//i.test(p) || /\.(png|jpe?g|gif|webp|ico|mp[34]|wav|flac|ogg|glb|gltf|blend)$/i.test(p)) throw new Error('E_MEDIA_READ_FORBIDDEN');
  if (/physics-elevators-optional|formal-perf-soak-readiness|tp01r9|tp18c1|ag38/i.test(p)) throw new Error('E_ACTIVE_OUTPUT_CONSUMPTION');
  if (!approvedRead.has(p)) throw new Error('E_READ_OUT_OF_SCOPE');
  return p;
}
const snapshots = [];
const sources = Object.fromEntries(Object.entries(paths).map(([id, p]) => {
  const safe = assertReadable(p);
  const real = relative(fs.realpathSync(path.join(root, safe)));
  if (real.toLowerCase() !== safe.toLowerCase()) throw new Error(`E_READ_REDIRECT ${p}`);
  const bytes = fs.readFileSync(path.join(root, safe));
  snapshots.push({ id, path: safe, sha256: sha(bytes), bytes: bytes.length });
  return [id, bytes.toString('utf8')];
}));
const json = (id) => JSON.parse(sources[id]);
const ag12 = json('ag12'), r7 = json('r7');
function write(p, content) {
  p = canonical(p);
  if (!(p.startsWith(own) || p === resultPath || p === reportPath)) throw new Error('E_WRITE_OUT_OF_SCOPE');
  const target = path.join(root, p);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  if (relative(fs.realpathSync(path.dirname(target))).toLowerCase() !== path.posix.dirname(p).toLowerCase()) throw new Error('E_WRITE_REDIRECT');
  fs.writeFileSync(target, content);
}
function ref(id, anchor) {
  const found = sources[id].split(/\r?\n/).flatMap((line, i) => line.includes(anchor) ? [{ line: i + 1, excerpt: line.trim().slice(0, 200) }] : []);
  return { path: paths[id], sha256: snapshots.find((s) => s.id === id).sha256, anchor, matches: found };
}
const R = {
  keys: ref('app', 'key === "q" || key === "e" || key === "r"'),
  skillRequest: ref('app', 'function requestSkill'), skillCatalog: ref('campaign', 'fn skill_catalog'), skill: ref('campaign', 'fn skill(&mut self'),
  interaction: ref('campaign', 'fn interact(&mut self'), interactRange: ref('campaign', 'distance_sq(ax, ay, bx, by) <= 2'),
  owner: ref('main', 'struct Session(Mutex<CampaignStore>)'), store: ref('store', 'pub struct CampaignStore'),
  supply: ref('campaign', 'fn collect_supply'), supplyField: ref('campaign', 'supply_claimed: BTreeSet<String>'),
  nearby: ref('campaign', 'fn nearby_value'), sort: ref('campaign', 'entries.sort_by'), nearest: ref('app', 'function nearestNearby'),
  los: ref('campaign', 'fn has_line_of_sight'), reveal: ref('campaign', 'fn reveal('),
  world: ref('world', 'pub struct WorldView'), save: ref('world', 'pub struct SaveV3'), state: ref('world', 'pub struct WorldStateV3'),
  input: ref('input', 'pub struct InputButtons'), action: ref('campaign', 'pub fn act('),
  cooldown: ref('campaign', 'if cooldown_left > 0'), saveAction: ref('store', 'fn should_save_now'),
  signalText: ref('app', '保持移动，扫描附近信号。'), aria: ref('html', 'aria-live'),
  supplyTest: ref('flowTests', 'fn supply_is_one_shot_and_revisit_has_no_meta_reward'),
  saveTest: ref('storeTests', 'fn high_frequency_actions_are_throttled_but_checkpoints_save_now'),
  worldTest: ref('worldTests', 'fn world_state_and_world_view_roundtrip_preserve_epoch_tick_ack_and_handles_none'),
  params: ref('ag12Source', 'function validateParameters'), unknown: ref('ag12Source', 'return "E_UNKNOWN_CAPABILITY"'),
  plan14: ref('plan', '| TP-14 | Campaign Migration |'), plan15: ref('plan', '| TP-15 | HUD/Map Integration |'),
};
const scanPattern = /perception\.loot_scan|\bloot_scan\b|\blootScan\b|\bLootScan\b|\bScanRequest\b|\bScanResult\b/;
const formalSourceIds = ['world', 'campaign', 'main', 'store', 'input', 'app', 'renderer', 'boot', 'html', 'css', 'flowTests', 'storeTests', 'worldTests', 'inputTests'];
const formalSymbolAudit = formalSourceIds.map((id) => ({ path: paths[id], sha256: snapshots.find((s) => s.id === id).sha256, matches: sources[id].split(/\r?\n/).flatMap((text, i) => scanPattern.test(text) ? [{ line: i + 1, text: text.trim().slice(0, 200) }] : []) }));
const noExistingScan = formalSymbolAudit.every((row) => row.matches.length === 0);
const capabilityIds = Object.keys(ag12.contract.capabilityIds);

const matrix = [];
function missing(id, classOfExistingEvidence, refs, observed, required, code) {
  matrix.push({ id, existingEvidenceClass: classOfExistingEvidence, lootScanContractStatus: 'missing', formalImplementation: false, pass: false, errorCode: code, evidence: refs, observed, missingContract: required });
}
missing('capability-id-fields', 'candidate_contract', [R.params, R.unknown, R.world], `AG-12只接受${capabilityIds.join(', ')}；正式WorldView只有通用snapshot字段。`, '已冻结的loot_scan标识、版本、参数、请求和结果类型；perception.loot_scan仅为本报告建议名称。', 'E_LOOT_SCAN_FIELDS_MISSING');
missing('single-owner', 'legacy_action', [R.owner, R.store, R.state], 'Mutex<CampaignStore>拥有旧CampaignState；通用WorldStateV3存在。', '由唯一正式runtime owner持有scan状态，复用正式grant authority，UI无命中或授权权威。', 'E_LOOT_SCAN_OWNER_MISSING');
missing('grant-revoke', 'candidate_contract', [R.params, R.skill], 'AG-12有其他能力的候选grant/revoke；旧技能检查exchange.owned。', 'scan有效grant计算、来源优先级、grant/revoke幂等、失效/重授予及cooldown tombstone。', 'E_SCAN_GRANT_REVOKE_MISSING');
missing('epoch-generation-tick', 'formal_implementation', [R.world, R.input], 'WorldView具备worldEpoch/serverTick/ackSeq；这些是通用字段，不是scan的接线证明。', 'scan请求/结果绑定worldId、epoch、ownerGeneration、grantRevision和权威sampledTick，陈旧响应不得生效。', 'E_SCAN_LIFECYCLE_BINDING_MISSING');
missing('input-qr', 'legacy_action', [R.keys, R.skillCatalog, R.skillRequest, R.input], 'Q/E/R映射skill slot0/1/2，Q=pulse、R=pierce；连续输入只有jump/dash/interact。', '独立可重映射scan intent与有界requestId/sequence；不得把Q/R战斗技能改名为scan。', 'E_SCAN_INPUT_CONTRACT_MISSING');
missing('spatial-range', 'legacy_action', [R.skill, R.interactRange, R.nearby], 'Q过滤敌人dist²≤6；R沿八方向1..3格找首敌；interact/nearby为dist²≤2。', '权威米制origin/range与物品eligibility；边界、NaN/Inf、跨世界和对象销毁处理。', 'E_SCAN_RANGE_CONTRACT_MISSING');
missing('line-of-sight', 'legacy_action', [R.los, R.skill, R.interaction, R.reveal], '旧Bresenham LOS helper用于敌人；Q/R/interact/nearby与reveal未调用它。', '来自同epoch正式几何的LOS端口；墙/动态门/缺失查询时的明确fail-closed语义。', 'E_SCAN_LOS_CONTRACT_MISSING');
missing('cooldown', 'legacy_action', [R.cooldown, R.skill, R.action], '旧技能确有拥有权、能量、剩余冷却校验；legacy tick递减并在入场重置。', 'scan的owner tick冷却、空结果消耗、重复request、revoke/regrant、重连/换世界/reload政策。', 'E_SCAN_COOLDOWN_CONTRACT_MISSING');
missing('stable-sort', 'legacy_action', [R.sort, R.nearest], 'Rust nearby已按距离平方、ID排序；UI重新按Manhattan距离选第一项。', '授权scan命中的稳定距离+ID排序、去重及确定性量化政策；不得用UI排序决定权威命中。', 'E_SCAN_SORT_CONTRACT_MISSING');
missing('bounded-results', 'absent', [R.nearby, R.sort], 'nearby按近距收集多类对象，没有显式结果上限或scan截断字段。', '最大命中数、候选查询工作量、payload bytes和truncated计数界限。', 'E_SCAN_LIMIT_CONTRACT_MISSING');
missing('result-leakage', 'legacy_action', [R.nearby, ref('campaign', 'fn map_view'), R.world], 'legacy map_view可输出完整points等数组；nearby还包括enemy/NPC/gate/portal。', '只输出经授权+range+LOS+eligibility过滤的loot结果；不输出隐藏对象/内部候选列表/完整map。', 'E_SCAN_RESULT_PRIVACY_CONTRACT_MISSING');
missing('query-not-claim', 'legacy_action', [R.supply, R.interaction, R.supplyTest], 'F→interact→collect_supply会修改done、supply_claimed与奖励，已有防重复领取测试。', 'scan仅查询、不得隐含拾取/奖励；实际领取再次独立验证对象与权限，处理扫描后对象消失。', 'E_SCAN_QUERY_CLAIM_BOUNDARY_MISSING');
missing('hud-projection', 'legacy_action', [R.signalText, R.nearest, ref('renderer', 'map.points.forEach')], '“扫描附近信号”是默认提示；现有marker/picking与普通地图渲染不是scan视图。', 'empty/unavailable/denied/cooldown/ready/truncated/expired状态与语义结果，含陈旧回调拒绝。', 'E_SCAN_HUD_CONTRACT_MISSING');
missing('accessibility', 'legacy_action', [R.aria, R.signalText, ref('app', 'aria-label')], '基础ARIA与技能键位/冷却文本存在。', 'scan结果文字列表、键盘操作、状态播报及重映射；基础字幕/字体/对比度不依赖grant。', 'E_SCAN_ACCESSIBILITY_CONTRACT_MISSING');
missing('dispose-close', 'absent', [R.world, ref('renderer', 'dispose'), R.owner], '通用renderer清理与旧session存在，未持有scan结果生命周期。', 'revoke/expiry/death/world replacement/close清空结果；取消并隔离旧generation回调。', 'E_SCAN_DISPOSE_CONTRACT_MISSING');
missing('save-boundary', 'legacy_action', [R.supplyField, R.saveAction, R.save, R.saveTest], '旧CampaignState可保存supply_claimed，SaveV3只有schema/saveId/checkpoint/transform；读取的是源码。', '仅保存获准语义grant；scan hits/request cache/runtime handles均为瞬态；reload不得绕过cooldown。', 'E_SCAN_SAVE_CONTRACT_MISSING');
missing('dedicated-tests', 'absent', [R.supplyTest, R.saveTest, R.worldTest], '现有测试覆盖领取防重、存档节流和通用epoch/tick；没有专用scan契约测试。', 'grant、spatial、cooldown、生命周期、泄漏、无障碍、存档边界与正式接线测试。', 'E_LOOT_SCAN_TESTS_MISSING');

const mappingSpecs = [
  ['TP-15A1', 'tp15a', 'HUD/map immutable ViewModel', '误绑为loot_scan', R.plan15],
  ['TP-15B1', 'tp15b', '真实WebView HUD/map文字DOM桥接', '误绑为loot_scan', R.plan15],
  ['TP-14A1', 'tp14a', 'Campaign progression authority候选', '误绑为HUD bridge或loot_scan', R.plan14],
  ['TP-14B1', 'tp14b', 'world runtime routing候选', '误绑为HUD bridge或loot_scan', R.plan14],
  ['TP-14C1', 'tp14c', '真实WebView world routing候选', '误绑为HUD bridge或loot_scan', R.plan14],
];
const mappingCorrections = mappingSpecs.map(([id, key, actualDomain, incorrectConsumerBinding, originalPlan]) => ({ id, path: paths[key], schema: json(key).schema, historicalStatus: json(key).status, originalPlan, originalPlanWasCorrect: true, incorrectConsumerBinding, actualDomain, mayProveLootScan: false, allowableUse: '证明证据领域边界；不能满足loot_scan的任何契约前置项。', errorCodeIfMisbound: 'E_EVIDENCE_DOMAIN_MISMATCH' }));
const tests = [
  ['no_grant', 'E_SCAN_NOT_GRANTED'], ['duplicate_grant_no_extra_owner', 'idempotent'], ['duplicate_revoke_exactly_once', 'idempotent'],
  ['old_epoch', 'E_SCAN_STALE_EPOCH'], ['old_generation', 'E_SCAN_STALE_GENERATION'], ['tick_regression_or_future', 'E_SCAN_TICK_INVALID'],
  ['duplicate_request_no_new_query', 'cached-identical-or-stable-duplicate'], ['cooldown_and_regrant_bypass', 'E_SCAN_COOLDOWN'],
  ['empty_result_consumes_cooldown', 'empty-with-nextAllowedTick'], ['range_boundary_nan_infinity', 'E_SCAN_RANGE_INVALID'],
  ['blocked_los_and_missing_backend', 'hidden-or-E_SCAN_QUERY_UNAVAILABLE'], ['cross_world_or_claimed_item', 'filtered'],
  ['stable_sort_distance_then_id', 'deterministic'], ['max_hits_and_payload', 'bounded-with-truncated'],
  ['hidden_or_internal_result_fields', 'E_SCAN_RESULT_FIELD_FORBIDDEN'], ['scan_does_not_claim_or_reward', 'state-unchanged'],
  ['revoke_expiry_close_invalidates_callbacks', 'no-result-after-invalidation'], ['thirty_epoch_replacements', 'one-owner-no-stale-cache'],
  ['accessibility_without_grant', 'baseline-remains-available'], ['keyboard_and_empty_cooldown_announcements', 'text-and-focus-consistent'],
  ['save_excludes_hits_and_runtime_handles', 'semantic-only'], ['reload_cooldown_conservative', 'cannot-scan-early'],
  ['preserve_q_e_r_f_and_existing_supply', 'legacy-regression'], ['real_owner_hud_integration', 'formal-only-after-upstream-gates'],
].map(([id, expected]) => ({ id, expected, implemented: false, executed: false }));
const futureFiles = [
  ['server-rs/src/loot_scan.rs', 'new', '纯Rust契约/查询组件，由既有唯一正式runtime owner持有，不建立第二线程或grant authority。'],
  ['server-rs/src/lib.rs', 'existing', '导出loot_scan模块。'],
  [paths.world, 'existing', '只读scan投影与owner接合字段；版本化serde边界。'],
  [paths.main, 'existing', '正式owner内处理scan intent，绑定epoch/generation并传输投影。'],
  ['server-rs/release-ui/js/loot-scan-view.js', 'new', '可单测的纯文本视图投影/陈旧结果拒绝，无空间或授权权威。'],
  [paths.app, 'existing', '接收正式投影、scan触发/状态；保留现役Q/E/R/F语义。'],
  [paths.html, 'existing', '加载纯投影代码与可访问文字结果列表。'],
  [paths.css, 'existing', '基础可读性、键盘焦点与状态样式。'],
  ['server-rs/tests/loot_scan_contract_test.rs', 'new', 'Rust契约/空间/幂等/cooldown/保存排除测试。'],
  ['server-rs/tests/loot_scan_integration_test.rs', 'new', '正式owner、权威查询、callback失效与只查询无奖励测试。'],
  [paths.worldTests, 'existing', '版本化WorldView序列化/无内部状态泄漏回归。'],
  ['server-rs/release-ui/tests/loot-scan-view.test.mjs', 'new', 'HUD状态与可访问投影/旧epoch-generation-tick拒绝测试。'],
];
const proposal = {
  schema: 'loot_scan_future_contract_proposal_tp15c1_v1', status: 'proposed_not_implemented', readinessEvidence: false, authorizedNow: false,
  packageId: 'PKG-FORMAL-LOOT-SCAN-MINIMAL-PROPOSAL', capabilityId: 'perception.loot_scan', capabilityIdStatus: 'suggestion_only_requires_freeze',
  owner: { role: 'one production runtime owner; a subordinate Rust LootScanState/query component', location: 'server-rs/src/loot_scan.rs', grantAuthority: 'upstream formal capability resolver only', uiAuthority: false, newOwnerThread: false },
  api: {
    request: 'request_loot_scan(intent: ScanIntent, ownerContext, readonlyQueryPort) -> Result<LootScanView, ScanError>',
    revoke: 'invalidate_scan(reason, currentEpoch, nextGeneration) -> empty/revoked LootScanView',
    queryPort: ['eligible_loot_within(worldEpoch, originM, rangeM, workLimit)', 'has_line_of_sight(worldEpoch, originM, targetM, geometryGeneration)'],
    intentFields: ['protocol', 'version', 'requestId', 'expectedWorldId', 'expectedEpoch', 'expectedGeneration', 'inputSequence'],
    authoritativeContext: ['actorId', 'worldId', 'worldEpoch', 'ownerGeneration', 'serverTick', 'actorTransformM', 'effectiveGrantId', 'grantRevision', 'geometryGeneration'],
    resultFields: ['protocol', 'version', 'worldId', 'worldEpoch', 'ownerGeneration', 'sampledTick', 'expiresAtTick', 'requestId', 'grantRevision', 'status', 'nextAllowedTick', 'truncated', 'hits'],
    hitFields: ['itemId', 'kind', 'distanceM', 'textLabel'],
    forbiddenResultFields: ['unfilteredCandidates', 'fullMap', 'hiddenItemIds', 'grantAuthority', 'filePath', 'mediaBytes', 'runtimeHandle', 'rewardMutation'],
    uiNeverProvides: ['effectiveGrant', 'originM', 'rangeM', 'cooldown', 'resultIds', 'LOS verdict'],
  },
  policy: {
    state: 'all values below are recommendations requiring a future explicit contract freeze, not current product requirements',
    suggestedDefaults: { ownerHz: 60, rangeM: 8, cooldownTicks: 180, maxHits: 16, maxQueryCandidates: 256, maxPayloadBytes: 4096, resultTtlTicks: 6 },
    acceptance: 'Validate identity/epoch/generation/sequence, effective grant and cooldown before bounded same-epoch query. Backend unavailable fails closed. Successful query including zero hits consumes cooldown exactly once.',
    rangeLos: 'Authoritative continuous-metre transform only; finite bounded range; authorized, unclaimed, same-world loot only; wall/dynamic-door LOS must use formal geometry. No fallback to unrestricted legacy map_view.',
    sorting: 'Sort authorized hits by deterministic distance key then canonical itemId; deduplicate IDs before maxHits; explicit truncated flag and payload bound.',
    grantRevoke: 'Repeated same grant/revoke is idempotent; conflicting grant payload is rejected. Revoke clears results and cancels callbacks but retains same-world cooldown tombstone; regrant cannot reset it.',
    lifecycle: 'Result expiry, revoke, death, world replacement and close invalidate cached views/callback tokens; generation increments before new work. Duplicate request must not query or spend twice.',
    cooldownRestore: 'Suggested initial/resume policy for persistent grant: full cooldown on resume/reattach/new world; do not restore transient scan hits. A fresh earned grant may be initially ready only under the frozen upstream grant policy.',
    save: 'Persist only already-authorized semantic grant data through upstream save boundary; do not serialize scan hits, callbacks, request caches, geometry/GPU handles. No new player-save format is authorized by this audit.',
    queryClaim: 'Scanning never collects, awards, reveals unexplored map globally or mutates inventory; ordinary pickup revalidates separately.',
    accessibility: 'Text-only HUD first: keyboard trigger/list, remappable scan action without repurposing Q/R, empty/unavailable/cooldown/truncated announcements. Baseline accessibility stays available without capability.',
  },
  futureFormalWhitelist: futureFiles.map(([file, kind, purpose]) => ({ path: file, kind, purpose })),
  minimalScopeReason: 'Initial package uses text HUD. 3D scan markers, renderer changes, assets and media are not required for this proposed minimal package. If later required, issue a separate exact whitelist.',
  dependencies: [
    { id: 'contract-policy-freeze', status: 'missing', requirement: 'Approve canonical ID, numeric limits, cooldown/reset/empty-result policies and input binding.' },
    { id: 'formal-capability-owner', status: 'blocked', requirement: 'Upstream single grant/resolver owner with semantic lifecycle; AG-12 is only reference.' },
    { id: 'formal-loot-query-and-los', status: 'missing', requirement: 'Public read-only same-epoch loot/claim/geometry query interface; never import legacy grid as continuous metres.' },
    { id: 'formal-ipc-owner', status: 'blocked', requirement: 'Production intent/WorldView transport, owner generation and close/replacement barrier.' },
    { id: 'formal-save-policy', status: 'blocked', requirement: 'Approved semantic-grant persistence and conservative resume policy; no scan cache in saves.' },
    { id: 'formal-hud-accessibility', status: 'blocked', requirement: 'Actual release entrypoint and manual keyboard/accessibility acceptance.' },
  ],
  dependencyBoundary: 'This whitelist assumes upstream dependencies already expose approved APIs. Missing query/resolver/save APIs require their own exact packages before this package can start; do not expand this list implicitly.',
  tests,
  rollback: 'Stop accepting scan intents; increment generation, cancel pending query callbacks and clear transient results; restore only future package files to pre-package versions, retain ordinary Q/E/R/F interaction and baseline accessibility, rerun legacy and schema regressions. Do not rewrite saves, supply claims, existing evidence or media.',
  gate: { capabilityHudTakeover: 'blocked/no_go', architecture: 'blocked/no_go', modifiesTP01R7Blockers: false },
};
write(proposalPath, `${JSON.stringify(proposal, null, 2)}\n`);

const controls = Object.fromEntries(Object.keys(REQUIRED_CONTROLS).map((key) => [key, false]));
const assessmentModel = {
  packageId: 'TP-15C1', capabilityId: 'perception.loot_scan', capabilityIdStatus: 'suggestion_only', scope: 'candidate_contract', architecture: 'blocked/no_go', formalTakeover: false,
  fields: [], owners: [], contractHash: null, provenance: [], controls, writes: [...PACKAGE_WRITES], readonlyInputs: exactReadonlyInputs, media: false, network: false, tests: [], rollback: proposal.rollback,
};
const assessment = evaluateContract(assessmentModel, { exactReadonlyInputs, trustedEvidence: {}, verifiedControls: controls });
const fixtures = runNegativeFixtures();
const pathFixtures = [
  ['relative-traversal', `${own}../formal-perf-soak-readiness/validate.mjs`, 'E_PATH_INVALID'],
  ['active-backslashes', 'tools\\continuous-tps-architecture-probe\\results\\formal-perf-soak-readiness-tp01r9.json', 'E_ACTIVE_OUTPUT_CONSUMPTION'],
  ['active-tp18c1', `${base}results/unconsumed-tp18c1.json`, 'E_ACTIVE_OUTPUT_CONSUMPTION'],
  ['active-ag38', `${base}results/physics-elevators-optional-ag38.json`, 'E_ACTIVE_OUTPUT_CONSUMPTION'],
  ['unapproved-source', 'server-rs/src/unapproved.rs', 'E_READ_OUT_OF_SCOPE'],
  ['media-uppercase', 'MEDIA/example.PNG', 'E_MEDIA_READ_FORBIDDEN'],
  ['nested-icon', 'server-rs/icons/icon.ico', 'E_MEDIA_READ_FORBIDDEN'],
  ['absolute-drive', 'C:/outside/file.rs', 'E_PATH_INVALID'],
].map(([id, input, expectedCode]) => {
  let actualCode = null;
  try { assertReadable(input); } catch (error) { actualCode = error.message; }
  return { id, input, expectedCode, actualCode, pass: expectedCode === actualCode, fileBytesRead: false };
});
const immutableInputs = snapshots.map((s) => ({ ...s, unchanged: sha(fs.readFileSync(path.join(root, s.path))) === s.sha256 }));
const checks = {
  allowedSourceSymbolsAbsent: noExistingScan,
  ag12HasNoLootScan: !capabilityIds.some((id) => /loot.*scan/i.test(id)) && sources.ag12Source.includes('return "E_UNKNOWN_CAPABILITY"'),
  qrMappingVerified: R.keys.matches.length > 0 && sources.campaign.includes('id: "pulse"') && sources.campaign.includes('id: "pierce"'),
  legacySortAndSupplyNotMisclassified: R.sort.matches.length > 0 && R.supplyTest.matches.length > 0 && matrix.every((r) => r.lootScanContractStatus === 'missing'),
  originalPlanningMappingCorrect: R.plan14.matches.length > 0 && R.plan15.matches.length > 0,
  mappingNeverProvesLootScan: mappingCorrections.every((r) => r.mayProveLootScan === false),
  evidenceAnchorsResolved: matrix.every((r) => r.evidence.every((e) => e.matches.length > 0)),
  tp01r7BlockerPreserved: r7.status === 'blocked' && r7.blockers.some((b) => b.code === 'E_LOOT_SCAN_CONTRACT_EVIDENCE_MISSING'),
  inputsUnchanged: immutableInputs.every((s) => s.unchanged),
  contractStillMissing: assessment.decision === 'blocked_missing_contract',
  negativeFixturesPass: fixtures.pass,
  forbiddenPathFixturesPass: pathFixtures.every((f) => f.pass),
};
const blockers = matrix.map((r) => ({ code: r.errorCode, area: r.id, detail: r.missingContract, evidencePaths: r.evidence.map((e) => e.path) }));
blockers.push({ code: 'E_SCAN_POLICY_NOT_FROZEN', area: 'proposal', detail: '新API、名称、默认范围/冷却/上限是未来建议，未成为已有或获准正式契约。', evidencePaths: [proposalPath] });
const output = {
  schema: 'loot_scan_contract_readiness_tp15c1_v1', status: 'blocked_missing_contract', auditPass: Object.values(checks).every(Boolean), generatedAtUtc: new Date().toISOString(),
  allowedDecisions: ['contract_ready_candidate_only', 'blocked_missing_contract'], scope: { allowedWrites: [...PACKAGE_WRITES], exactReadonlyInputs, activeIdsNotConsumed: activeIds, formalFilesModified: false, oldToolsResultsReportsModified: false, mediaBytesRead: false, mediaCopiedOrGenerated: false, networkAccess: false, gitHistoryOrRemoteAccess: false },
  classificationMeaning: { absent: '在指定范围内找不到该scan契约', legacy_action: '已有旧战役动作/投影，不能证明scan', candidate_contract: '其他能力的候选契约，不等于loot_scan', formal_implementation: '已存在的通用正式组件，仅其自身领域，不等于scan接线' },
  formalSymbolAudit, matrix, mappingCorrections,
  historicalEvidence: ['ag12', 'tp15a', 'tp15b', 'tp14a', 'tp14b', 'tp14c'].map((id) => ({ path: paths[id], schema: json(id).schema, status: json(id).status, usedAsLootScanProof: false, sourceScopeOnly: true })),
  proposal: { path: proposalPath, sha256: sha(fs.readFileSync(path.join(root, proposalPath))), acceptedExistingContract: false, implemented: false, futureFormalWhitelist: proposal.futureFormalWhitelist, dependencies: proposal.dependencies, tests: proposal.tests, rollback: proposal.rollback },
  existingTests: [R.supplyTest, R.saveTest, R.worldTest].map((e) => ({ ...e, inspected: true, rerunInThisPackage: false, provesLootScan: false })),
  validationScope: 'Pure metadata/contract gate fixtures and source evidence checks only; no cargo, legacy validator, game, WebView or media runtime was run.',
  assessment, negativeFixtures: fixtures, forbiddenPathFixtures: pathFixtures, blockers, checks, immutableInputs,
  validatorProvenance: ['validate.mjs', 'contract-gate.mjs'].map((name) => ({ path: `${own}${name}`, sha256: sha(fs.readFileSync(path.join(here, name))) })),
  gate: { contract: 'blocked_missing_contract', formalCapabilityHudTakeover: 'blocked/no_go', architecture: 'blocked/no_go', tp01r7Unchanged: true },
};
write(resultPath, `${JSON.stringify(output, null, 2)}\n`);
const fixturePassed = fixtures.passed, fixtureTotal = fixtures.total;
const matrixTable = matrix.map((r) => `| ${r.id} | ${r.existingEvidenceClass} | missing | ${r.errorCode} | ${r.observed} |`).join('\n');
const mappingTable = mappingCorrections.map((r) => `| ${r.id} | ${r.incorrectConsumerBinding} | ${r.actualDomain} | 不能 |`).join('\n');
const fileTable = proposal.futureFormalWhitelist.map((f) => `| \`${f.path}\` | ${f.kind} | ${f.purpose} |`).join('\n');
write(reportPath, `# TP-15C1 Loot Scan Capability 契约缺口复核

## 结论

**blocked_missing_contract**；auditPass=${output.auditPass}。本次只做契约readiness：在指定源码和证据中没有loot_scan候选契约或正式实现。未来建议已写入 \`${proposalPath}\`，建议不等于既有实现、测试通过或正式授权。TP-01R7原blockers不变，capability/HUD正式接管与Architecture Gate保持blocked/no_go。

唯一写入为本包工具目录、结果JSON和本报告。只读取明确列出的${exactReadonlyInputs.length}个输入文件，前后hash核验；AG-38、TP-01R9、TP-18C1活跃输出不在读取清单中。未联网、截图、读写/复制/生成媒体，未运行游戏、WebView、Cargo或既有会改写结果的validator。

## 现役语义

- Q/E/R映射skill slot0/1/2：Q=pulse（脉冲），R=pierce（穿刺）；F才调用interact(id)。Q/R不能冒充scan或被本包重绑定。
- Q对存活敌人dist²≤6造成伤害，R沿八方向1..3格找首敌；它们使用旧网格单位。旧技能确有拥有权、能量和冷却校验，legacy tick递减冷却；这不是scan权威时钟或米制查询契约。
- interact与nearby使用dist²≤2。Rust nearby已按距离平方后ID排序，UI再按Manhattan距离选第一项；nearby没有显式结果上限且包含多类对象，不是授权loot列表。
- 旧LOS helper存在，用于敌人判断；Q/R/interact/nearby及reveal未调用它。缺的是scan的LOS接线与策略，不能写成“工程完全没有LOS”。
- loot相关现役行为是collect_supply：记录supply_claimed、标记done并发放一次奖励；扫描未来必须只查询，实际领取另行验证。
- “保持移动，扫描附近信号”是提示文案。普通地图marker、picking、基础ARIA、技能冷却文字均不能证明scan HUD。
- CampaignStore持有旧CampaignState并保存整个state；读取的是保存源码，没有打开真实玩家数据。SaveV3只含schema/saveId/checkpoint/transform。没有scan hits/cache/handles的排除契约。

## 证据／缺失矩阵

下表的class描述已有证据自身所属层级；所有行的loot_scan契约状态都是missing。通用正式WorldView字段存在，不代表scan已经正式接线。机器矩阵记录每项path、hash、定位行、缺失字段与所需契约。

| 契约项 | 现有证据分类 | scan状态 | blocker | 现有事实 |
|---|---|---|---|---|
${matrixTable}

AG-12仅接受perception.rear_view、map.local、mobility.dash，其他标识返回E_UNKNOWN_CAPABILITY。perception.loot_scan是本报告的候选命名建议。

## 证据映射纠偏

原始规划工作包表（TPS_GREY_HIVE_IMPLEMENTATION_PLAN.md:149–150）已经正确写明TP-14=Campaign Migration、TP-15=HUD/Map Integration。需要纠正的是后续消费证据时的错误标签，不能写成“旧规划已经完成loot_scan”。这些输入在本包只用于证明领域边界。

| 证据 | 应拒绝的消费标签 | 实际领域 | 可作为loot_scan证明 |
|---|---|---|---|
${mappingTable}

## 未来最小正式包建议

候选API：\`request_loot_scan(intent, ownerContext, readonlyQueryPort) -> LootScanView | ScanError\`。唯一正式runtime owner持有scan组件，正式capability resolver提供有效grant；UI只发送意图，不能提供位置、范围、授权、LOS结论或命中名单。请求/结果绑定world、epoch、generation、sequence与权威tick，结果只含itemId、kind、distanceM、textLabel等已过滤语义数据。

候选数值建议为8米、60Hz owner下180 tick冷却、最多16结果、256候选工作上限、4096-byte payload、6 tick结果TTL；**均须未来冻结，不是当前已批准要求**。同epoch几何查询不可用时失败关闭；成功的空结果也消耗一次冷却。重试不重复查询/消费，revoke清空结果但保留同世界冷却记录，regrant不能绕过。建议持久grant在resume/reattach/新世界进入时恢复完整冷却，排除瞬态结果与runtime handles的保存，避免以重载绕过。

初包采用可访问文字HUD。3D扫描标记或renderer新功能需要另列后续白名单。最小正式包必须等capability owner、loot/LOS查询、IPC owner、保存政策与输入/数值政策先满足；若这些上游API缺失，先独立完成前置包，不能隐式扩展下列写集。

| 建议未来文件 | 类型 | 目的 |
|---|---|---|
${fileTable}

未来测试${tests.length}项详见proposal与机器结果，均标记implemented=false/executed=false：grant/revoke幂等、epoch/generation/tick、重试/cooldown/regrant、范围/LOS、过滤排序限量、无结果泄漏、不领取奖励、close回调、无障碍、存档与reload、现役Q/E/R/F回归和正式owner/HUD接线。

Rollback：停止scan意图，递增generation、取消查询并清空瞬态结果；仅恢复未来包写入文件，保持普通交互、基础无障碍及原有存档；复跑schema和legacy回归。当前审计撤销只涉及自身三条白名单，未执行任何Git历史操作。

## 验证与边界

Gate负向夹具${fixturePassed}/${fixtureTotal}通过；禁读路径夹具${pathFixtures.filter((f) => f.pass).length}/${pathFixtures.length}通过。夹具修改完整synthetic模型，断言具体错误码和blocked_missing_contract；synthetic正向对照最多contract_ready_candidate_only，不能作为项目ready证据。

现有campaign_flow的补给测试证明重复领取/回访不重复奖励；campaign_store测试证明保存节流/checkpoint；WorldView测试证明通用snapshot字段。它们均只读审查、未重跑，也不证明loot_scan。新validator不把自身Gate夹具当作未来scan行为测试。

验证命令：

- \`node --check ${own}validate.mjs\`
- \`node ${own}validate.mjs\`
- \`git diff --check -- ${reportPath} ${own.slice(0, -1)} ${resultPath}\`

机器结果：\`${resultPath}\`。精确blockers为矩阵中的17项missing，加E_SCAN_POLICY_NOT_FROZEN；TP-01R7的E_LOOT_SCAN_CONTRACT_EVIDENCE_MISSING保持原样。
`);
console.log(JSON.stringify({ status: output.status, auditPass: output.auditPass, matrixRows: matrix.length, negativeFixtures: { passed: fixturePassed, total: fixtureTotal }, pathFixtures: pathFixtures.length, checks, resultPath }, null, 2));
if (!output.auditPass) process.exitCode = 1;
