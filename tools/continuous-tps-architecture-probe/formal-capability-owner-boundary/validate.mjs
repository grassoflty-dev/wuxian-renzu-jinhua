import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { RULES, closure, intersect, evaluateBoundary, runNegativeFixtures } from './boundary-gate.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const base = 'tools/continuous-tps-architecture-probe/';
const own = `${base}formal-capability-owner-boundary/`;
const resultPath = `${base}results/formal-capability-owner-boundary-tp01r10.json`;
const reportPath = 'docs/CONTINUOUS_FORMAL_CAPABILITY_OWNER_BOUNDARY_REVIEW.md';
const proposalPath = `${own}future-boundary.json`;
const allowedWrites = [`${own}**`, resultPath, reportPath];
const activeIds = ['AG-38', 'TP-15C2', 'TP-18C2R1'];
const resultNames = {
  r7: 'formal-capability-hud-readiness-tp01r7', c1: 'loot-scan-contract-readiness-tp15c1', ag12: 'capability-ag12',
  rearA: 'capability-rear-view-tp13a1', rearB: 'tauri-rear-view-tp13b1',
  queueA: 'runtime-integration-soak-tp17a1', queueB: 'runtime-owner-observability-tp17b1', queueC: 'tauri-runtime-observability-tp17c1',
  plan: 'vertical-slice-readiness-delta-tp18b1', ipc: 'formal-ipc-owner-readiness-tp01r4', assets: 'formal-asset-release-readiness-tp01r6', perf: 'formal-perf-soak-readiness-tp01r9',
};
const sourcePaths = {
  world: 'server-rs/src/world_v3.rs', entry: 'server-rs/src/campaign_main.rs', lib: 'server-rs/src/lib.rs',
  app: 'server-rs/release-ui/js/app.js', renderer: 'server-rs/release-ui/js/tps-renderer.js', boot: 'server-rs/release-ui/js/renderer-boot.js',
  html: 'server-rs/release-ui/index.html', css: 'server-rs/release-ui/css/style.css',
  worldTests: 'server-rs/tests/world_v3_contract_test.rs', inputTests: 'server-rs/tests/continuous_input_test.rs', heartbeatTests: 'server-rs/tests/tp06_a_heartbeat_test.rs',
  config: 'server-rs/tauri.conf.json', cargo: 'server-rs/Cargo.toml', build: 'server-rs/build.rs',
  ag12Source: `${base}capability/validate.mjs`,
};
const inputs = { ...Object.fromEntries(Object.entries(resultNames).map(([id, name]) => [id, `${base}results/${name}.json`])), ...sourcePaths };
const exactReadonlyInputs = Object.values(inputs);
const permittedReads = new Set(exactReadonlyInputs);
const hash = (b) => crypto.createHash('sha256').update(b).digest('hex');
const rel = (p) => path.relative(root, p).replaceAll('\\', '/');
function canonical(p) {
  if (typeof p !== 'string' || !p || p !== p.trim() || /[:*?\x00-\x1f]/.test(p)) throw new Error('E_INVALID_PATH');
  p = p.replaceAll('\\', '/');
  if (p.startsWith('/') || p.split('/').some((x) => !x || x === '.' || x === '..' || /[. ]$/.test(x))) throw new Error('E_INVALID_PATH');
  return p;
}
function allowRead(p) {
  p = canonical(p);
  if (/ag[-_]?38|tp[-_]?15c2|tp[-_]?18c2r1|physics-elevators-optional/i.test(p)) throw new Error('E_ACTIVE_OUTPUT_FORBIDDEN');
  if (/(^|\/)(assets|data|media|public)\//i.test(p) || /\.(png|jpe?g|webp|ico|mp3|mp4|wav|glb|gltf|blend)$/i.test(p)) throw new Error('E_MEDIA_READ_FORBIDDEN');
  if (!permittedReads.has(p)) throw new Error('E_READ_SCOPE');
  return p;
}
const snapshots = [];
const text = Object.fromEntries(Object.entries(inputs).map(([id, name]) => {
  const p = allowRead(name), abs = path.join(root, p);
  if (rel(fs.realpathSync(abs)).toLowerCase() !== p.toLowerCase()) throw new Error('E_READ_REDIRECT');
  const bytes = fs.readFileSync(abs);
  snapshots.push({ id, path: p, sha256: hash(bytes), bytes: bytes.length });
  return [id, bytes.toString('utf8')];
}));
const evidence = Object.fromEntries(Object.keys(resultNames).map((id) => [id, JSON.parse(text[id])]));
function write(p, content) {
  p = canonical(p);
  if (!(p.startsWith(own) || p === resultPath || p === reportPath)) throw new Error('E_WRITE_SCOPE');
  const abs = path.join(root, p);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  if (rel(fs.realpathSync(path.dirname(abs))).toLowerCase() !== path.posix.dirname(p).toLowerCase()) throw new Error('E_WRITE_REDIRECT');
  fs.writeFileSync(abs, content);
}
const shaFor = (id) => snapshots.find((s) => s.id === id).sha256;
const citation = (id, anchor = null) => ({ path: inputs[id], sha256: shaFor(id), anchor, lines: anchor === null ? [] : text[id].split(/\r?\n/).flatMap((s, i) => s.includes(anchor) ? [i + 1] : []) });
const targetId = 'PKG-FORMAL-CAPABILITY-HUD';
const originalPackages = evidence.plan.packages;
const originalGraph = Object.fromEntries(originalPackages.map((p) => [p.id, p]));
const original = originalGraph[targetId];
if (!original || originalPackages.length !== 8) throw new Error('E_FROZEN_PLAN_SCHEMA');
const rollback = [
  { id: 'stop-intents', action: 'Stop capability grant/action ingress and detach its subscriptions without stopping unrelated gameplay owners.' },
  { id: 'invalidate-generation', action: 'Advance owner/presentation generation and revoke effective views before accepting any replacement owner.' },
  { id: 'cleanup-presentation', action: 'Publish empty/revoked view; release rear-view resources exactly once and reject late callbacks before UI/renderer disposal.' },
  { id: 'drain-close', action: 'Cancel/drain only owned pending work; detach bounded queue/Channel port and wait for in-flight callbacks; do not destroy shared IPC authority.' },
  { id: 'restore-files', action: 'Restore only the future exact package files; remove proposed modules/exports when reverting. Keep basic accessibility/text fallback and previous entrypoint intact.' },
  { id: 'verify-regression', action: 'Freshly compile reverted sources, rerun owner/lifecycle/schema/legacy tests and record baseline counts; no rewriting saves, grants, media or historical evidence.' },
];
const ownerModule = 'server-rs/src/capability_owner.rs';
const viewModule = 'server-rs/release-ui/js/capability-view.js';
const futureSpec = [
  [ownerModule, 'future_proposed', 'Single authoritative Rust grant ledger/resolver, explicit provider descriptors and lifecycle; owned by production runtime, not a second thread.'],
  [sourcePaths.lib, 'existing', 'Export proposed capability owner module after separate implementation authorization.'],
  [sourcePaths.world, 'existing', 'Versioned read-only CapabilityView embedding; runtime/GPU handles remain outside serde/save.'],
  [sourcePaths.entry, 'existing', 'Wire one owner into actual production runtime ingress/transport and shutdown barrier.'],
  [viewModule, 'future_proposed', 'Pure immutable UI projection and stale-token filtering; never grant authority.'],
  [sourcePaths.app, 'existing', 'Actual release entry: subscribe/render capability snapshots and preserve basic controls/accessibility.'],
  [sourcePaths.renderer, 'existing', 'Own rear-view GPU presentation pair, reconcile authorized deltas and dispose exactly once.'],
  [sourcePaths.boot, 'existing', 'Delegate presentation lifecycle using owner/epoch/generation binding; no parallel authority.'],
  [sourcePaths.html, 'existing', 'Load presentation-only view module and semantic HUD/accessibility nodes.'],
  [sourcePaths.css, 'existing', 'HUD readability, focus and accessibility presentation.'],
  ['server-rs/tests/capability_owner_test.rs', 'future_proposed', 'Pure owner/grant/provider/queue/cooldown/semantic serialization unit contract.'],
  ['server-rs/tests/capability_owner_lifecycle_test.rs', 'future_proposed', 'Production host replacement, callback barrier, revoke/close and no double owner.'],
  [sourcePaths.worldTests, 'existing', 'WorldView version/fields/immutability and save excludes runtime handles.'],
  ['server-rs/release-ui/tests/capability-view.test.mjs', 'future_proposed', 'Pure UI epoch/generation/stale-callback/accessibility projection regression.'],
];
const files = futureSpec.map(([p, state, purpose]) => ({ path: p, state, purpose }));
const futureWrites = files.map((f) => f.path);
// Existence checks are limited to exact future paths; no content from proposed
// files or active packages is opened.
const existence = Object.fromEntries([...futureWrites, 'server-rs/release-ui/js/capability-resolver.js', 'server-rs/release-ui/js/client.js'].map((p) => [p, fs.existsSync(path.join(root, p))]));
const entryCorrection = {
  frontendDist: JSON.parse(text.config).build.frontendDist,
  actualEntry: sourcePaths.app,
  evidence: [citation('config', 'frontendDist'), citation('html', 'app.js'), citation('html', 'renderer-boot.js'), citation('entry', 'struct Session(Mutex<CampaignStore>)')],
  nonexistentOldAssumptions: ['server-rs/release-ui/js/capability-resolver.js', 'server-rs/release-ui/js/client.js'].map((p) => ({ path: p, exists: existence[p], replacement: p.endsWith('/client.js') ? sourcePaths.app : 'Rust capability_owner.rs authority plus presentation-only capability-view.js, both future proposals' })),
  oldUiExcludedFromNewWrites: 'server-rs/ui/index.html',
};
const additionalDeps = ['PKG-FORMAL-GREY-HIVE-SCENE', 'PKG-FORMAL-SAVE-CAMPAIGN', 'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH'];
const proposedGraph = structuredClone(originalGraph);
proposedGraph[targetId] = { ...original, writes: futureWrites, prerequisites: [...new Set([...original.prerequisites, ...additionalDeps])], status: 'future_proposed_not_authorized' };
const originalClosure = closure(targetId, originalGraph), proposedClosure = closure(targetId, proposedGraph);
const packageWrites = Object.fromEntries(Object.entries(proposedGraph).map(([id, p]) => [id, p.writes]));
const packageDependencies = Object.fromEntries(Object.entries(proposedGraph).map(([id, p]) => [id, p.prerequisites]));
const conflicts = originalPackages.filter((p) => p.id !== targetId).map((p) => ({ packageId: p.id, sharedFiles: intersect(futureWrites, p.writes), hasFileConflict: intersect(futureWrites, p.writes).length > 0, dependencyOfOwner: proposedClosure.nodes.includes(p.id), downstreamAcceptance: p.id === 'PKG-FORMAL-PERF-SOAK-ACCEPTANCE', noFileConflictDoesNotAuthorizeParallelExecution: true }));
const allPairConflicts = [];
const ids = Object.keys(packageWrites);
for (let i = 0; i < ids.length; i++) for (let j = i + 1; j < ids.length; j++) {
  const shared = intersect(packageWrites[ids[i]], packageWrites[ids[j]]);
  if (shared.length) allPairConflicts.push({ a: ids[i], b: ids[j], sharedFiles: shared });
}
// Conservative singleton waves: dependency order is computed, never dispatched.
const order = [], visiting = new Set(), done = new Set();
function visit(id) {
  if (visiting.has(id)) throw new Error('E_DEPENDENCY_CYCLE');
  if (done.has(id)) return;
  visiting.add(id);
  for (const d of proposedGraph[id].prerequisites) if (proposedGraph[d]) visit(d);
  visiting.delete(id); done.add(id); order.push(id);
}
ids.forEach(visit);
const waves = order.map((id) => [id]);
const dependencyRegistry = Object.fromEntries(proposedClosure.nodes.map((id) => {
  const evidenceId = id === 'PKG-FORMAL-IPC-OWNER' ? 'ipc' : id === 'PKG-FORMAL-ASSET-RELEASE' || id === 'TP-16' ? 'assets' : id === 'ADR-H' ? 'ag12' : id === 'TP-13' ? 'rearB' : id === 'TP-15' ? 'r7' : 'plan';
  return [id, { status: 'blocked', level: 'completed_review_or_planning_not_formal', evidencePath: inputs[evidenceId], sourceHash: shaFor(evidenceId), originalStatus: evidence[evidenceId].status, packageStatus: proposedGraph[id]?.status ?? null, missingFormalPrerequisiteEvidence: true }];
}));
const testPlan = [
  ['owner-contract', 'cargo test --offline --locked --manifest-path server-rs/Cargo.toml --test capability_owner_test', 'grant/revoke/priority/expiry, one mutable owner, unregistered extension rejected, queue/cooldown ports and semantic-only serialization'],
  ['owner-lifecycle', 'cargo test --offline --locked --manifest-path server-rs/Cargo.toml --test capability_owner_lifecycle_test', 'epoch/generation fences, duplicate/reentrant callback, revoke/regrant, 30 transitions, one owner and baseline resource accounting'],
  ['world-view-schema', 'cargo test --offline --locked --manifest-path server-rs/Cargo.toml --test world_v3_contract_test', 'versioned HUD/capability fields, no runtime/GPU handles and no stale authority'],
  ['ui-projection', 'node server-rs/release-ui/tests/capability-view.test.mjs', 'independent baseline accessibility, immutable semantic projection, stale callbacks and dispose'],
  ['rust-regression', 'cargo test --offline --locked --manifest-path server-rs/Cargo.toml', 'full current-source Rust regression, including legacy behavior and new owner'],
  ['release-build', 'cargo build --offline --locked --release --manifest-path server-rs/Cargo.toml', 'exact authorized source revision and real release entrypoint; upstream asset/media permissions required in that future execution package'],
  ['rear-view-real-load', 'future separately authorized real WebView/release lifecycle acceptance', '50 enable/revoke cycles, formal world changes, real GPU cleanup/occlusion/feel and performance; candidate TP13 does not substitute'],
].map(([id, command, purpose]) => ({ id, command, purpose, status: 'future_required_not_run', sourceCompilationRequired: id !== 'ui-projection' && id !== 'rear-view-real-load' }));
const compilePolicy = { sourceRebuildRequired: true, exactSourceHashesRequired: true, prebuiltBinarySufficient: false, syntaxCheckSufficient: false, cargoLockMustStayUnchanged: true, targetDirectoryRequiresFutureAuthorization: true, currentPackageRunsCargo: false, currentPackageRunsWebView: false, provenanceFieldsRequired: ['source hashes/revision', 'lockfile hash', 'compiler/toolchain', 'features', 'profile', 'commands and exit codes', 'test names/counts', 'runtime/hardware/source binding for actual release acceptance'] };
const matrix = [];
function row(id, owner, api, paths, deps, tests, refs, missing) {
  matrix.push({ id, owner, api, files: paths.map((p) => ({ path: p, state: files.find((f) => f.path === p)?.state ?? 'dependency_owned_readonly' })), dependencies: deps, requiredTests: tests, rollbackSteps: rollback.map((r) => r.id), evidence: refs.map((id) => citation(id)), currentStatus: 'formal_boundary_not_implemented', blocker: missing });
}
row('authority', 'One Rust CapabilityOwner inside existing production runtime; UI cannot mutate grants.', ['apply_semantic_event(event, sourceIdentity, epoch, generation)', 'snapshot() -> readonly CapabilityView'], [ownerModule, sourcePaths.lib, sourcePaths.entry], ['PKG-FORMAL-IPC-OWNER', 'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH'], ['owner-contract', 'owner-lifecycle'], ['ag12', 'ipc', 'r7'], 'E_FORMAL_CAPABILITY_OWNER_MISSING');
row('rear-view', 'Rust decides effective permission; TPS renderer owns its own disposable presentation resources.', ['CapabilityPresentationDelta', 'reconcile(view, epoch, generation, capabilityRevision)', 'release(reason)'], [ownerModule, sourcePaths.renderer, sourcePaths.boot], ['TP-13', 'PKG-FORMAL-GREY-HIVE-SCENE', 'PKG-FORMAL-ASSET-RELEASE'], ['owner-lifecycle', 'rear-view-real-load'], ['rearA', 'rearB', 'assets'], 'E_FORMAL_REAR_VIEW_HANDOFF_MISSING');
row('future-loot-slot', 'Generic descriptor registry in same owner; no loot provider, product name or numbers registered.', ['register_approved_descriptor(opaqueId, version, semanticSchema, actionPolicyPort)', 'unknown_or_unregistered_id -> stable rejection'], [ownerModule], ['separate future product/contract approval, not satisfied by current audit'], ['owner-contract'], ['c1', 'ag12'], 'E_EXTENSION_PROVIDER_REQUIRES_SEPARATE_PACKAGE');
row('queue', 'Use bounded IPC-owner port; no parallel UI/shadow queue authority.', ['enqueue_bounded(intent, epoch, generation, seq)', 'drain_outside_callback_lock()'], [ownerModule, sourcePaths.entry], ['PKG-FORMAL-CORE-TICK-KCC', 'PKG-FORMAL-IPC-OWNER'], ['owner-contract', 'owner-lifecycle'], ['queueA', 'queueB', 'queueC', 'ipc'], 'E_FORMAL_QUEUE_PORT_MISSING');
row('cooldown', 'Rust owner tick plus per-approved-descriptor policy; no UI timer authority.', ['evaluate_registered_action_policy(nowTick, priorLedger)', 'preserve_semantic_cooldown_boundary_on_restore(policy)'], [ownerModule], ['PKG-FORMAL-CORE-TICK-KCC', 'PKG-FORMAL-SAVE-CAMPAIGN', 'future descriptor policy approval'], ['owner-contract'], ['ag12', 'c1'], 'E_FORMAL_COOLDOWN_POLICY_PORT_MISSING');
row('accessibility', 'UI baseline preferences independent of all grants.', ['project_baseline_preferences()', 'project_capability_status_without_hiding_baseline()'], [viewModule, sourcePaths.app, sourcePaths.html, sourcePaths.css], ['TP-15'], ['ui-projection'], ['r7'], 'E_FORMAL_ACCESSIBILITY_PROJECTION_MISSING');
row('epoch-generation', 'Host owner controls world epoch, owner generation and capability/presentation revision.', ['replace_world: stop ingress -> invalidate generation -> cleanup -> new owner', 'validate_token_before_mutation_and_before_callback_effect()'], [ownerModule, sourcePaths.entry, viewModule, sourcePaths.boot], ['PKG-FORMAL-IPC-OWNER'], ['owner-lifecycle', 'ui-projection'], ['queueC', 'ipc', 'r7'], 'E_FORMAL_STALE_CALLBACK_BARRIER_MISSING');
row('grant-revoke', 'Only authoritative semantic sources may grant; deterministic effective set.', ['grant_once(grantId, source, scope, semanticParameters)', 'revoke_once(grantId)', 'expire_at_owner_tick()', 'recompute_effective_set()'], [ownerModule, sourcePaths.world], ['ADR-H', 'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH'], ['owner-contract', 'world-view-schema'], ['ag12', 'rearA'], 'E_FORMAL_GRANT_REVOKE_MISSING');
row('cleanup', 'Owner invalidates authority; renderer acknowledges release; shared host stays independently owned.', ['close_capability_ingress()', 'invalidate_tokens()', 'release_presentation_once()', 'drain_owned_work_and_detach_ports()'], [ownerModule, sourcePaths.entry, sourcePaths.app, sourcePaths.renderer, sourcePaths.boot], ['PKG-FORMAL-IPC-OWNER', 'TP-13'], ['owner-lifecycle', 'rear-view-real-load'], ['rearB', 'queueC', 'perf'], 'E_FORMAL_CLEANUP_ACCEPTANCE_MISSING');
row('hud-projection', 'WorldView producer and pure JS view projector; no JS capability resolver authority.', ['CapabilityView{protocol,version,worldId,worldEpoch,tick,ownerGeneration,revision,effective,baseline}', 'project_hud(view) -> immutable semantic fields'], [sourcePaths.world, viewModule, sourcePaths.app, sourcePaths.html], ['TP-15', 'PKG-FORMAL-IPC-OWNER'], ['world-view-schema', 'ui-projection'], ['r7', 'ipc'], 'E_FORMAL_HUD_PROJECTION_MISSING');
row('save-port', 'Existing Save/Campaign package owns durable storage; capability owner exports semantic state only.', ['export_semantic_grants()', 'restore_approved_semantic_snapshot(snapshot) -> recompute; never restore GPU/callback/result caches'], [ownerModule, sourcePaths.world], ['PKG-FORMAL-SAVE-CAMPAIGN'], ['owner-contract', 'world-view-schema', 'rust-regression'], ['ag12', 'c1'], 'E_FORMAL_SAVE_PORT_CONTRACT_MISSING');

const rules = Object.fromEntries(Object.keys(RULES).map((k) => [k, true]));
const boundary = {
  packageId: 'TP-01R10', basedOn: targetId, proposalOnly: true, implemented: false, authorizedNow: false,
  owners: [{ id: 'future-capability-owner', path: ownerModule, authority: 'future_rust_single_owner', hosting: 'owned component of production runtime; no extra thread/global owner' }],
  entry: sourcePaths.app, files, registeredCapabilities: ['perception.rear_view'], registrationMeaning: 'future rear-view scope only; zero providers implemented or registered by this audit',
  lootSlot: { name: null, numericPolicy: null, registered: false, implemented: false },
  lootPolicyDisposition: 'All TP-15C1 suggested names and defaults remain unresolved; no values copied into the owner contract. Only opaque future descriptor extension is specified.',
  rules, rulesMeaning: 'Proposed boundary requirements, NOT observed formal implementation or passed runtime tests.',
  dependencies: proposedClosure.nodes.map((id) => ({ id, status: 'blocked', evidencePath: dependencyRegistry[id].evidencePath })),
  addedDependencyReasons: { 'PKG-FORMAL-GREY-HIVE-SCENE': 'rear-view uses formal world/scene transforms and lifecycle', 'PKG-FORMAL-SAVE-CAMPAIGN': 'semantic grant restore boundary and cooldown port cannot invent persistence', 'PKG-FORMAL-CAMPAIGN-PLAYTHROUGH': 'production grant-source integration must come from authoritative Campaign handoff, not synthetic grants' },
  perfIsDownstreamAcceptanceNotUpstream: true, waves, wavesMeaning: 'Conservative future order only; every wave remains blocked and undispatched.', dispatchNow: false,
  tests: testPlan, compilePolicy, rollback,
  splitConditions: [
    'New Rust capability_owner.rs is proposed with lib.rs export; do not scatter a second resolver into campaign_main or UI.',
    'If queue/query/save ports are missing, their owning upstream package must supply them first; no implicit Cargo/config/CampaignStore/new module writes.',
    'If rear-view presentation grows into a separate module, name that exact file and test in a new revision before implementation; current minimum keeps it within existing tps-renderer/renderer-boot.',
    'If a future capability requires spatial query or product parameters, issue its own descriptor/provider package and tests. This package reserves an empty extension boundary only.',
    'Scene/IPC/Save/Core/Campaign shared files require a fresh exact-source rebase and rescheduled write lease; historical TP18 paths do not authorize later-expanded packages.',
  ],
  actualWrites: [`${own}validate.mjs`, `${own}boundary-gate.mjs`, proposalPath, resultPath, reportPath],
  provenance: snapshots.filter((s) => Object.hasOwn(resultNames, s.id)).map((s) => ({ id: s.id, path: s.path, sha256: s.sha256, level: 'candidate_or_completed_review' })),
  architecture: 'blocked/no_go', formalTakeover: false, releaseAuthorized: false, media: false, network: false,
};
const facts = { actualEntry: entryCorrection.actualEntry, verifiedRules: rules, futureWrites, existingFiles: existence, requiredDependencies: proposedClosure.nodes, dependencyRegistry, packageWrites, packageDependencies, exactReadonlyInputs, provenance: Object.fromEntries(snapshots.map((s) => [s.path, { sha256: s.sha256, level: 'candidate_or_completed_review' }])) };
const evaluation = evaluateBoundary(boundary, facts);
const negatives = runNegativeFixtures();
const pathNegatives = [
  ['active-ag38', `${base}results/physics-elevators-optional-ag38.json`, 'E_ACTIVE_OUTPUT_FORBIDDEN'],
  ['active-tp15c2', `${base}results/loot-scan-tp15c2.json`, 'E_ACTIVE_OUTPUT_FORBIDDEN'],
  ['active-tp18c2r1', `${base}results/delta-tp18c2r1.json`, 'E_ACTIVE_OUTPUT_FORBIDDEN'],
  ['traversal', `${own}../other/file.json`, 'E_INVALID_PATH'],
  ['media', 'MEDIA/image.PNG', 'E_MEDIA_READ_FORBIDDEN'],
  ['unapproved', 'server-rs/src/unapproved.rs', 'E_READ_SCOPE'],
].map(([id, input, expectedCode]) => { let actualCode = null; try { allowRead(input); } catch (e) { actualCode = e.message; } return { id, input, expectedCode, actualCode, pass: actualCode === expectedCode, bytesRead: false }; });
const graphChecks = {
  originalCycleFree: originalClosure.cycles.length === 0 && originalClosure.unknownPackages.length === 0,
  proposedCycleFree: proposedClosure.cycles.length === 0 && proposedClosure.unknownPackages.length === 0,
  detectsCycle: closure('PKG-A', { 'PKG-A': { prerequisites: ['PKG-B'] }, 'PKG-B': { prerequisites: ['PKG-A'] } }).cycles.length > 0,
  detectsUnknown: closure('PKG-A', { 'PKG-A': { prerequisites: ['PKG-MISSING'] } }).unknownPackages.includes('PKG-MISSING'),
  preservedOriginalPrerequisites: original.prerequisites.every((d) => proposedGraph[targetId].prerequisites.includes(d)),
  perfNotAddedUpstream: !proposedClosure.nodes.includes('PKG-FORMAL-PERF-SOAK-ACCEPTANCE'),
  allSevenCompared: conflicts.length === 7,
};
const unchangedInputs = snapshots.map((s) => ({ ...s, unchanged: hash(fs.readFileSync(path.join(root, s.path))) === s.sha256 }));
const upstreamBlockers = Object.fromEntries(['r7', 'c1', 'ipc', 'assets', 'perf'].map((id) => [id, { path: inputs[id], sha256: shaFor(id), status: evidence[id].status, blockers: evidence[id].blockers, modified: false }]));
const blockers = [
  ...matrix.filter((r) => r.id !== 'future-loot-slot').map((r) => ({ code: r.blocker, area: r.id, evidencePaths: r.evidence.map((e) => e.path) })),
  { code: 'E_IPC_PREREQUISITE_BLOCKED', area: 'dependency', evidencePaths: [inputs.ipc] },
  { code: 'E_ASSET_PREREQUISITE_BLOCKED', area: 'dependency', evidencePaths: [inputs.assets] },
  { code: 'E_ASSET_EVIDENCE_MEDIA_BOUNDARY_BREACH_RECORDED', area: 'provenance', evidencePaths: [inputs.assets], detail: 'R6 itself records E_MEDIA_BYTE_READ_POLICY_BREACH; this audit reads only its JSON and does not rerun validators or read assets.' },
  { code: 'E_PREREQUISITE_CLOSURE_NOT_FORMAL', area: 'dependency', dependencyIds: proposedClosure.nodes, evidencePaths: [inputs.plan] },
  { code: 'E_FORMAL_SOURCE_BUILD_UNMEASURED', area: 'tests', detail: 'Only this metadata validator ran; future source compilation and exact build provenance required.' },
  { code: 'E_FORMAL_RELEASE_ACCEPTANCE_MISSING', area: 'release', evidencePaths: [inputs.perf] },
];
const checks = {
  graphChecksPass: Object.values(graphChecks).every(Boolean),
  entryIsActual: text.html.includes('app.js') && JSON.parse(text.config).build.frontendDist === './release-ui',
  oldAssumptionsRemainAbsent: entryCorrection.nonexistentOldAssumptions.every((r) => r.exists === false),
  proposedModulesNotImplemented: files.filter((f) => f.state === 'future_proposed').every((f) => existence[f.path] === false),
  originalBlockersPreserved: evidence.r7.status === 'blocked' && evidence.c1.status === 'blocked_missing_contract' && unchangedInputs.every((s) => s.unchanged),
  lootPolicyUnresolved: boundary.lootSlot.name === null && boundary.lootSlot.numericPolicy === null && boundary.lootSlot.registered === false,
  negativeFixturesPass: negatives.pass, forbiddenPathFixturesPass: pathNegatives.every((f) => f.pass),
  remainsBlocked: evaluation.decision === 'blocked_boundary',
  noWaveConflictOrOrderError: !evaluation.errorCodes.some((c) => ['E_SAME_WAVE_WRITE_CONFLICT', 'E_DEPENDENCY_ORDER', 'E_WAVE_MISSING', 'E_WAVE_INVALID'].includes(c)),
};
write(proposalPath, `${JSON.stringify(boundary, null, 2)}\n`);
const output = {
  schema: 'formal_capability_owner_boundary_tp01r10_v1', status: 'blocked_boundary', auditPass: Object.values(checks).every(Boolean), generatedAtUtc: new Date().toISOString(),
  scope: { allowedWrites, exactReadonlyInputs, activeIdsNotConsumed: activeIds, activeFrozenContractsRead: false, activeNote: 'No active output or contract file was opened; IDs and no-consumption rule come from this task. Scheduling calculations use only frozen completed TP18B1 formal package specs.', mediaBytesRead: false, mediaGeneratedOrCopied: false, networkAccess: false, formalFilesModified: false, existingArtifactsModified: false },
  originalPackage: original, entryCorrection, matrix,
  dependencyAnalysis: { originalClosure, proposedClosure, addedDirectDependencies: additionalDeps, dependencyRegistry, conflictsWithSevenPackages: conflicts, allPairConflicts, futureOnlyWaves: waves, allWavesBlocked: true, graphChecks, interpretation: 'Shared files computed from exact TP18B1 paths versus proposed 14-file owner set. No direct Assets/Perf conflict is not prerequisite satisfaction. Perf stays downstream to avoid an acceptance cycle.' },
  futureBoundary: { path: proposalPath, sha256: hash(fs.readFileSync(path.join(root, proposalPath))), files, splitConditions: boundary.splitConditions, lootSlot: boundary.lootSlot, tests: testPlan, compilePolicy, rollback },
  unchangedUpstreamBlockers: upstreamBlockers, blockers, evaluation, negativeFixtures: negatives, pathNegativeFixtures: pathNegatives, checks, readOnlyInputs: unchangedInputs,
  validationScope: 'Metadata/source review and pure negative fixtures only. No Cargo compilation, old validator rerun, game, WebView, screenshot or media access.',
  validatorProvenance: ['validate.mjs', 'boundary-gate.mjs'].map((name) => ({ path: `${own}${name}`, sha256: hash(fs.readFileSync(path.join(here, name))) })),
  gate: { boundary: 'blocked_boundary', maximum: 'boundary_ready_candidate_only', formalOwnerHudLootRelease: 'blocked/no_go', architecture: 'blocked/no_go' },
};
write(resultPath, `${JSON.stringify(output, null, 2)}\n`);
const matrixRows = matrix.map((r) => `| ${r.id} | ${r.owner} | ${r.api.join('; ')} | ${r.dependencies.join(', ')} | ${r.requiredTests.join(', ')} |`).join('\n');
const conflictRows = conflicts.map((r) => `| ${r.packageId} | ${r.sharedFiles.join(', ') || '无直接共享文件'} | ${r.dependencyOfOwner ? 'owner上游' : '下游验收'} |`).join('\n');
write(reportPath, `# TP-01R10 Formal Capability Owner Boundary 精确前置复核

## 结论

**blocked_boundary**；auditPass=${output.auditPass}。已完成未来包边界草案、依赖闭包和写集冲突计算；没有实现或授权正式owner/HUD/loot_scan/发布。Architecture仍blocked/no_go；TP-01R7、TP-15C1及其他上游blockers在机器结果中逐项保留，旧文件hash不变。

只写本包工具目录、结果JSON和本报告；读取${exactReadonlyInputs.length}项明确输入。未读取AG-38、TP-15C2、TP-18C2R1活跃输出；连冻结契约文件也未打开，调度比较仅使用已完成TP-18B1的正式包路径，不推定当前活跃包的最新写集。未联网、读写/复制/生成媒体、运行Cargo、WebView或旧validator。

## 当前入口与假设纠偏

tauri.conf.json的frontendDist是./release-ui，正式二进制入口为campaign_main.rs；release-ui/index.html加载tps-renderer.js、renderer-boot.js、app.js。app.js是当前业务入口。capability-resolver.js、release-ui/js/client.js均不存在，不能作为现有owner或测试入口。

新capability_owner.rs与capability-view.js均只列future_proposed：Rust为唯一授权状态owner，JS只投影。原计划旧server-rs/ui/index.html不进入本次未来最小写集。现有WorldView通用epoch/tick字段、legacy队列与基础ARIA、renderer dispose不等于正式capability接管；TP06A仍为synthetic heartbeat路径。

## owner/API/file/dependency/test/rollback矩阵

机器矩阵还含每项精确文件状态、证据path/hash与全部rollback步骤；下表为可读摘要。

| 领域 | owner | future API边界 | 依赖 | 必须测试 |
|---|---|---|---|---|
${matrixRows}

loot_scan名称和全部数值政策保持**unresolved**（机器字段name=null、numericPolicy=null、registered=false、implemented=false）。TP-15C1的建议名称/默认值不复制到owner契约。注册器只规定后续获准descriptor的扩展端口，未知/未注册能力拒绝执行；不得借队列或cooldown模块偷偷实现scan。此未实现插槽是刻意边界，不假称已消除C1缺口。

## 依赖闭包与冲突

原包直接prerequisites完整保留。拟增加Scene、Save、Campaign三个正式包前置：分别约束rear-view世界资源、语义grant恢复、生产grant来源接线。Core通过这些包及IPC传递依赖。Perf是后续真实验收，**不反向加入owner前置**，避免owner↔Perf环。

原闭包${originalClosure.nodes.length}项；拟定闭包${proposedClosure.nodes.length}项（${proposedClosure.nodes.filter((id) => id.startsWith('PKG-')).length}个正式包与${proposedClosure.nodes.filter((id) => !id.startsWith('PKG-')).length}个ADR/TP叶节点）。机器结果列所有节点、边、来源hash和blocked状态，未查证的叶节点不自动pass。

| 对比正式包 | 与未来owner共享文件 | 关系 |
|---|---|---|
${conflictRows}

Assets/Perf无直接共享文件也不能绕过前置/验收。全包两两冲突边共${allPairConflicts.length}条。保守未来单包wave顺序：${waves.map((w) => w[0]).join(' → ')}。所有wave均blocked、未调度；同wave共享写文件被夹具拒绝。真正执行前仍需重新核对准确源码和当时活跃契约。

## 最小未来正式白名单与拆分

${files.map((f) => `- \`${f.path}\` — ${f.state}；${f.purpose}`).join('\n')}

${boundary.splitConditions.map((s) => `- ${s}`).join('\n')}

不把Cargo/config修改默认为必要项；若引入新依赖或新存档字段，必须先另列其精确文件与测试并重新计算冲突。上述模块只是未来建议，未修改任何正式文件。

## 测试编译要求

${testPlan.map((t) => `- ${t.id}：\`${t.command}\`；${t.purpose}；future_required_not_run。`).join('\n')}

必须重新编译未来执行包的准确源码，记录source/lockfile hash、toolchain/features/profile、命令码与测试清单。Node语法检查、候选探针和预编译binary复跑均不能替代。当前包不运行这些正式命令；Tauri构建/真实验收涉及的输出与媒体权限须由未来执行包单独明确。

TP-01R4仍有正式IPC/production owner缺口；TP-01R6记录发布授权、manifest、正式resolver缺口，并自报E_MEDIA_BYTE_READ_POLICY_BREACH，不能忽略或重跑越界流程；TP-01R9缺正式构建/硬件/负载/性能证据。它们的完整blockers保留在机器结果。

## Rollback顺序

${rollback.map((r, i) => `${i + 1}. ${r.action}`).join('\n')}

当前审计如撤销，仅涉及自身白名单。没有执行回滚或Git历史操作。

## 本包验证

${negatives.passed}/${negatives.total}个Gate负向夹具与${pathNegatives.filter((f) => f.pass).length}/${pathNegatives.length}个禁读路径夹具通过；闭包循环/未知包识别测试通过。合成正向对照只证明校验器可返回boundary_ready_candidate_only，绝不作正式证据。

- \`node --check ${own}validate.mjs\`
- \`node ${own}validate.mjs\`
- \`git diff --check -- ${reportPath} ${own.slice(0, -1)} ${resultPath}\`

机器结果：\`${resultPath}\`；未来边界：\`${proposalPath}\`。
`);
console.log(JSON.stringify({ status: output.status, auditPass: output.auditPass, checks, matrixRows: matrix.length, closureNodes: proposedClosure.nodes.length, conflictPairs: allPairConflicts.length, negatives: { total: negatives.total, passed: negatives.passed }, pathNegatives: pathNegatives.length, resultPath }, null, 2));
if (!output.auditPass) process.exitCode = 1;
