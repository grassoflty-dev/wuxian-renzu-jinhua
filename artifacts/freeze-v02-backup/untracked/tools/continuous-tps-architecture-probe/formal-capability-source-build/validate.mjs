import fs from 'node:fs';
import path from 'node:path';
import { own, root, relative, digest, inside, snapshot, upstream, activePrefixes, forbidden, write } from './build.mjs';
import { assess, fixtures } from './gate.mjs';

const read = p => fs.readFileSync(path.join(root, p), 'utf8');
const json = p => JSON.parse(read(p));
const build = JSON.parse(fs.readFileSync(path.join(own, 'build-evidence.json'), 'utf8'));
const r10Path = upstream[0], r10 = json(r10Path);
const boundaryPath = 'tools/continuous-tps-architecture-probe/formal-capability-owner-boundary/future-boundary.json';
const boundary = json(boundaryPath);
const currentHashes = snapshot();
const inputsUnchanged = JSON.stringify(currentHashes) === JSON.stringify(build.inputsBefore) && build.hashesUnchanged;
const sourceHashes = currentHashes.filter(p => p.path.startsWith('server-rs/'));
const hashFor = p => currentHashes.find(e => e.path === p)?.sha256;
const ownerPath = 'server-rs/src/capability_owner.rs';
const viewPath = 'server-rs/release-ui/js/capability-view.js';
const lib = read('server-rs/src/lib.rs');
const formalOwnerExport = /^pub\s+mod\s+capability_owner\s*;/m.test(lib);
const futureFiles = boundary.files.map(f => ({ path: f.path, exists: fs.existsSync(path.join(root, f.path)),
  action: fs.existsSync(path.join(root, f.path)) ? 'modify' : 'create', proposedState: f.state,
  sha256: hashFor(f.path) ?? null, purpose: f.purpose,
  testTarget: f.path.includes('/tests/') ? path.basename(f.path).replace(/\.(rs|mjs)$/, '') : null }));
const entryPaths = ['server-rs/release-ui/js/app.js', 'server-rs/release-ui/js/capability-resolver.js', 'server-rs/release-ui/js/client.js'];
const entry = { actual: entryPaths[0], paths: entryPaths.map(p => ({ path: p, exists: fs.existsSync(path.join(root,p)), sha256: hashFor(p) ?? null })),
  indexLoadsApp: /src=["']js\/app\.js["']/.test(read('server-rs/release-ui/index.html')),
  indexLoadsBabylonRenderer: /js\/tps-renderer\.js/.test(read('server-rs/release-ui/index.html')),
  bootBabylonOnly: read('server-rs/release-ui/js/renderer-boot.js').includes('Babylon-only'),
  productionBinaryDeclared: /path\s*=\s*"src\/campaign_main\.rs"/.test(read('server-rs/Cargo.toml')) };

const expectations = {
  authority: ['CapabilityOwner', 'apply_semantic_event'],
  'rear-view': ['CapabilityPresentationDelta', 'reconcile_capability_view'],
  'future-loot-slot': ['register_approved_descriptor'],
  queue: ['enqueue_bounded'], cooldown: ['evaluate_registered_action_policy'],
  accessibility: ['project_baseline_preferences', 'project_capability_status_without_hiding_baseline'],
  'epoch-generation': ['validate_token_before_mutation_and_before_callback_effect'],
  'grant-revoke': ['grant_once', 'revoke_once'], cleanup: ['close_capability_ingress', 'invalidate_tokens', 'release_presentation_once'],
  'hud-projection': ['CapabilityView', 'project_hud'], 'save-port': ['export_semantic_grants', 'restore_approved_semantic_snapshot'],
};
const baselineNotes = {
  authority: 'lib.rs exports current modules; campaign_main owns CampaignStore and synthetic TP06A runtime, not CapabilityOwner.',
  'rear-view': 'Babylon renderer/dispose exists; no formally authorized rear-view capability handoff.',
  'future-loot-slot': 'TP15C2 is an isolated candidate; descriptor name/numbers remain unresolved and no formal registration is inferred.',
  queue: 'InputReceiver exposes receive/consume/set_world_epoch; this is not a capability action port.',
  cooldown: 'Legacy campaign cooldown values are not a registered capability policy/restore contract.',
  accessibility: 'Existing ARIA/preferences remain basic UI behavior; no formal capability projection contract.',
  'epoch-generation': 'WorldView.world_epoch and InputReceiver.set_world_epoch exist; complete owner-generation callback barriers do not.',
  'grant-revoke': 'No formal capability ledger/module or exported grant/revoke API.',
  cleanup: 'Renderer dispose and TP06A close exist; no complete capability invalidation/resource acknowledgment barrier.',
  'hud-projection': 'WorldView has protocol/version/epoch/tick/ack/world/player/actors/doors; no CapabilityView field.',
  'save-port': 'CampaignStore and SaveV3 exist; no approved capability semantic export/restore API.',
};
const inspectPaths = sourceHashes.filter(p => /\/src\/.*\.rs$|release-ui\/js\/.*\.js$/.test(p.path)).map(p => p.path);
const sources = new Map(inspectPaths.map(p => [p, read(p)]));
const symbolMatrix = r10.matrix.map(row => {
  const expected = expectations[row.id];
  const matches = expected.flatMap(symbol => [...sources].flatMap(([p, text]) => {
    const re = new RegExp('\\b' + symbol + '\\b', 'g');
    return [...text.matchAll(re)].map(m => ({ symbol, path: p, line: text.slice(0,m.index).split('\n').length, sha256: hashFor(p) }));
  }));
  return { id: row.id, proposedApi: row.api, expectedSymbols: expected, status: !formalOwnerExport && !matches.length ? 'missing_formal_api' : 'requires_manual_review',
    matches, inspectedPaths: inspectPaths, currentBaseline: baselineNotes[row.id], evidence: row.files.map(f => typeof f === 'string' ? f : f.path).filter(Boolean).map(p => ({ path:p, sha256:hashFor(p) ?? null, exists:fs.existsSync(path.join(root,p)) })),
    limitation: 'Names are future expectation probes from R10, not a frozen API; generic/alternative legacy APIs are not treated as equivalent.',
    inheritedBlocker: row.blocker };
});

function lockPackages(text) {
  return text.split('[[package]]').slice(1).map(block => ({ name: block.match(/^name = "([^"]+)"/m)?.[1], version: block.match(/^version = "([^"]+)"/m)?.[1], checksum: block.match(/^checksum = "([^"]+)"/m)?.[1] ?? null }));
}
const formalPackages = lockPackages(read('server-rs/Cargo.lock'));
const probePackages = lockPackages(fs.readFileSync(path.join(own,'Cargo.lock'),'utf8'));
const registryPackageDrift = probePackages.filter(p => p.checksum && !formalPackages.some(f => f.name === p.name && f.version === p.version && f.checksum === p.checksum));
const logsVerified = build.commands.every(c => inside(path.join(root,c.log)) && fs.existsSync(path.join(root,c.log)) && digest(path.join(root,c.log)) === c.logSha256);
const testCommand = build.commands.find(c => c.id === 'cargo-test-locked');
const ownerModuleStillMissing = !fs.existsSync(path.join(root,ownerPath)) && !formalOwnerExport && !fs.existsSync(path.join(root,viewPath));
const originalBlockers = structuredClone(r10.blockers);
const testCount = build.testCounts.totalPassed;
const envBounded = ['CARGO_HOME','CARGO_TARGET_DIR','TEMP','TMP','TMPDIR'].every(k => inside(build.env[k]));
const model = {
  packageId:'TP-01R11', architecture:'blocked/no_go', formalReady:false, releaseAuthorized:false, network:false, offline:build.env.CARGO_NET_OFFLINE === 'true', media:false,
  prebuiltBinaryUsed:false, emptyTargetAtStart:build.emptyTargetAtStart, target:build.target, compiledFromSource:build.compiledLibrary?.fresh === false,
  testsRecompiled:build.freshTestArtifacts, testPassed:testCommand?.exitCode === 0, testCount,
  toolchain:build.toolchain.rustc, lockHash:build.formalLockSha256, probeLockHash:build.probeLockSha256, manifestHash:build.probeManifestSha256,
  features:build.resolvedFeatures, profile:build.profile, sourceHashes:build.inputsBefore.filter(p => p.path.startsWith('server-rs/')),
  inputHashesUnchanged:inputsUnchanged, symbolMatrix, missingCompileExpected:build.missingOwnerCompileFailure.confirmed,
  adapter:{ probeOnly:true, implementsOwner:false, formalModule:false, libraryPath:'server-rs/src/lib.rs', excluded:'application build.rs, campaign_main binary, frontend resource bundling', noShimImplementation:true },
  futureFiles, proofKind:'fresh_rust_library_source_only', evidenceLevel:'baseline_only', commands:build.commands,
  rollback:['stop-probe','preserve-evidence','remove-only-own-generated-artifacts','retain-formal-sources'],
  consumedPaths:currentHashes.map(p=>p.path), outputPaths:[relative(own)+'/build-evidence.json','tools/continuous-tps-architecture-probe/results/formal-capability-source-build-tp01r11.json','docs/CONTINUOUS_FORMAL_CAPABILITY_SOURCE_BUILD_REVIEW.md'],
  lootName:null, lootNumericPolicy:null, lootImplemented:false, nativeReleaseMeasured:false, futureOwnerBuildMeasured:false, originalBlockersPreserved:true,
};
const trusted = { ...structuredClone(model), toolchain:build.toolchain.rustc, sourceHashes, commandLogsVerified:logsVerified,
  lockHash:digest(path.join(root,'server-rs/Cargo.lock')), probeLockHash:digest(path.join(own,'Cargo.lock')), manifestHash:digest(path.join(own,'Cargo.toml')),
  compiledFromSource:build.buildPassed && envBounded && build.artifactPathsInside && registryPackageDrift.length === 0,
  originalBlockersPreserved:ownerModuleStillMissing && originalBlockers.length === r10.blockers.length };
const evaluation = assess(model,trusted), negatives = fixtures();
const checks = {
  baselineBuildPassed:build.buildPassed, sourceHashesUnchanged:inputsUnchanged, commandLogsVerified:logsVerified,
  exactRegistryVersions:registryPackageDrift.length === 0, pathsAndEnvBounded:envBounded && build.artifactPathsInside,
  fourteenFiles:futureFiles.length === 14, futureFiveMissing:futureFiles.filter(f=>!f.exists).length === 5,
  actualEntry:entry.indexLoadsApp && entry.productionBinaryDeclared && entry.paths[0].exists && entry.paths.slice(1).every(p=>!p.exists),
  missingOwnerRetained:ownerModuleStillMissing, missingMatrixComplete:symbolMatrix.length === 11 && symbolMatrix.every(r=>r.status==='missing_formal_api'),
  negativeFixtures:negatives.pass, originalBlockersRetained:JSON.stringify(originalBlockers) === JSON.stringify(r10.blockers),
  noActiveInputs:model.consumedPaths.every(p=>!forbidden(p)),
};
const auditPass = Object.values(checks).every(Boolean) && evaluation.pass;
const status = auditPass ? 'baseline_built_formal_owner_missing' : 'baseline_build_failed';
const output = {
  schema:'formal_capability_source_build_tp01r11_v1', status, auditPass, generatedAtUtc:new Date().toISOString(),
  scope:{ sourceOnly:true, probeOnly:true, nativeRelease:false, formalOwner:false, mediaAccess:false, network:false, activeContracts:'User supplied frozen whitelist text only; outputs excluded.', activeExcludedPrefixes:activePrefixes },
  baseline:build, checks, evaluation, futureFiles, moduleExport:{ path:'server-rs/src/lib.rs', sha256:hashFor('server-rs/src/lib.rs'), capabilityOwnerExport:formalOwnerExport },
  entryCorrection:entry, missingApiMatrix:symbolMatrix,
  sharedConflicts:{ source:r10Path, sourceSha256:hashFor(r10Path), conflicts:r10.dependencyAnalysis.conflictsWithSevenPackages, noFormalWrites:true },
  registryPackageDrift, model, negativeFixtures:negatives,
  upstreamEvidence:upstream.map(p=>{const r=json(p);return {path:p,sha256:hashFor(p),status:r.status,blockers:r.blockers ?? r.unchangedUpstreamBlockers ?? null};}),
  unchangedR10Blockers:originalBlockers,
  blockerInterpretation:'E_FORMAL_SOURCE_BUILD_UNMEASURED remains for the future owner/native application. This run only measures the unchanged current library in an isolated manifest; no R10 blocker is erased.',
  blockers:[...originalBlockers, {code:'E_FORMAL_NATIVE_APPLICATION_BUILD_UNMEASURED',detail:build.exclusionReason}, ...(!auditPass ? [{code:'E_CURRENT_SOURCE_BASELINE_FAILED',errors:evaluation.errorCodes,checks}] : [])],
  artifactPolicy:{ root:relative(own), generatedTarget:build.target, temp:build.temp, cargoHome:build.cargoHome, emptyTargetAtStart:build.emptyTargetAtStart,
    pathsChecked:build.artifacts.flatMap(a=>a.filenames), outputScopeVerified:envBounded && build.artifactPathsInside,
    limitation:'Path/env/compiler artifact provenance checked, not a system-wide filesystem audit. Shared dependency-cache junctions are input locations, not output authorization.' },
  gate:{architecture:'blocked/no_go',formalReady:false,ownerHudLootRelease:'blocked/no_go'},
};
const resultPath='tools/continuous-tps-architecture-probe/results/formal-capability-source-build-tp01r11.json';
const reportPath='docs/CONTINUOUS_FORMAL_CAPABILITY_SOURCE_BUILD_REVIEW.md';
// The only two explicitly permitted output files outside own directory.
fs.writeFileSync(path.join(root,resultPath),JSON.stringify(output,null,2)+'\n');
const report = `# TP-01R11 当前源码构建基线与 Missing API 复核

结论：**${status}**，auditPass=${auditPass}。Architecture仍blocked/no_go。只证明当前库源码及公开API基线，不证明未来capability owner或原生发布构建通过；TP-01R10全部${originalBlockers.length}项blockers原样保留。

## 构建范围与来源

隔离source-only manifest的lib.path直接引用未修改的server-rs/src/lib.rs，保留正式库的依赖声明与默认features。没有复制库源码、替换模块、实现trait shim或伪造owner；根探针仅调用公开Campaign/InputReceiver/WorldView相关API，并include现有测试原文。适配层明确probeOnly。

正式build.rs无条件调用tauri_build::build()，因此本包排除应用build.rs、campaign_main二进制及前端资源打包，遵守游戏媒体禁读边界。完整库源码包含TP06A及Tauri库依赖；这不是正式应用manifest或release profile构建。未来正式owner/原生应用的E_FORMAL_SOURCE_BUILD_UNMEASURED并未消除。

工具链：

\`\`\`text
${build.toolchain.rustc}
${build.toolchain.cargo}
${build.toolchain.active}
Node ${build.toolchain.node}
\`\`\`

- Formal Cargo.lock SHA-256：${build.formalLockSha256}
- Probe Cargo.lock SHA-256：${build.probeLockSha256}
- Probe manifest SHA-256：${build.probeManifestSha256}
- Source-only adapter manifest SHA-256：${build.baselineManifestSha256}
- Requested probe features：[]；resolved dependency features逐项见机器结果baseline.resolvedFeatures。默认库features未关闭。
- Profile：test/dev，opt-level=0、debug=0、incremental=false；未测release。registry版本/checksum相对正式lock漂移${registryPackageDrift.length}项。
- ${sourceHashes.length}项正式源码/配置/测试输入hash，全部输入${currentHashes.length}项；构建前后和报告时hash一致=${inputsUnchanged}。
- 开始${build.startedAt}；结束${build.endedAt}。

## 实际命令与测试

所有Cargo命令均设置CARGO_NET_OFFLINE=true、CARGO_TARGET_DIR/TEMP/TMP/CARGO_HOME在本包内，未使用预编译游戏binary。既有Rust dependency cache仅作为离线输入；第一次不挂接archive cache的尝试因offline缺adler2失败，原日志保留，未联网。

| 命令 | 退出码 | 时间 | 日志 |
|---|---|---|---|
${build.commands.map(c=>`| ${c.exe} ${c.args.join(' ')} | ${c.exitCode} | ${c.startedAt} → ${c.endedAt} | ${c.log} |`).join('\n')}

测试通过${testCount}项；新建空target=${build.emptyTargetAtStart}；正式库fresh=false=${build.compiledLibrary?.fresh === false}；4个测试target全部重新编译=${build.freshTestArtifacts}。既有world_v3_contract、campaign_flow、continuous_input测试源码原文include，无真实存档/媒体操作。探针新增3个公开API调用测试。

expect-formal-owner故意引用wuxian_horror_ch1::capability_owner::CapabilityOwner；预期E0432缺模块编译失败，确认=${build.missingOwnerCompileFailure.confirmed}。该非零退出是缺失符号证据，不冒称测试通过。Node语法检查不替代上述Rust编译。

## 14文件存在性与入口

| 文件 | 当前存在 | 未来动作 | 测试目标 |
|---|---|---|---|
${futureFiles.map(f=>`| ${f.path} | ${f.exists} | ${f.action} | ${f.testTarget ?? '—'} |`).join('\n')}

正式lib.rs导出capability_owner=${formalOwnerExport}。5个未来文件仍缺失。发布index加载app.js；capability-resolver.js/client.js仍不存在。renderer-boot已是Babylon-only，但渲染器切换不等于连续运行时、capability/HUD或发布验收完成。

共享文件冲突沿用R10完成结果并携带其hash；本包不写任何正式文件。Core/IPC/Save涉及lib.rs，IPC/Campaign涉及campaign_main.rs，Scene涉及world_v3.rs/world_v3_contract_test.rs。Assets/Perf无直接共享文件，仍受前置/下游验收约束。活跃TP04D1/TP15C3/TP18C2仅使用协调任务给出的冻结路径文本排除，未读取其产物。

## Missing API/symbol matrix

| 领域 | 未来期待符号（非冻结API） | 状态 | 当前基线与限制 |
|---|---|---|---|
${symbolMatrix.map(r=>`| ${r.id} | ${r.expectedSymbols.join(', ')} | ${r.status} | ${r.currentBaseline} |`).join('\n')}

静态搜索范围及每个证据path/hash、匹配行在JSON中；不存在的模块不能构成授权owner。loot名称/数值仍null，未实现或注册。C2 candidate不提升为正式能力。

## 负向夹具与产物边界

${negatives.passed}/${negatives.total}个实际变异负向夹具通过：预编译复用、target越界、lock/toolchain/source/manifest缺失及漂移、features/profile漂移、缺符号忽略、shim冒充formal、未来文件当现有、JS替代Rust、测试未重编译、3活跃包消费、candidate升级、媒体/联网/写集/rollback/provenance/Gate越权。合成正向仅检查Gate逻辑，不作构建证据。

产物位于${build.target}，临时文件位于${build.temp}，Cargo锁和global-cache位于${build.cargoHome}。检查环境路径、空target、所有compiler-artifact输出路径；未向server-rs/target下发构建。本检查不是全系统文件访问审计。

Rollback：停止本探针→保留审计证据→如需撤销仅清理本包生成产物→正式源码与原报告保持不动。没有实际删除或恢复操作。

## 阻塞项

${output.blockers.map(b=>`- ${b.code}`).join('\n')}

原R10 owner/lifecycle/HUD/release阻塞以及R7/C1/C2/R4/R6/R9证据级别均保持。R6历史E_MEDIA_BYTE_READ_POLICY_BREACH仅作为既有报告元数据读取，未重跑其验证器。

本包验证：cargo fmt --check、cargo test --locked见命令表；node --check validate.mjs、node validate.mjs与限定git diff --check由执行任务单独运行并回传。
`;
fs.writeFileSync(path.join(root,reportPath),report);
write(path.join(own,'validation-evidence.json'),JSON.stringify({status,auditPass,checks,negativeFixtures:{passed:negatives.passed,total:negatives.total},resultPath},null,2)+'\n');
console.log(JSON.stringify({status,auditPass,checks,tests:testCount,negativeFixtures:{passed:negatives.passed,total:negatives.total},resultPath},null,2));
if (!auditPass) process.exitCode=1;
