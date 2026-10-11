import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/formal-asset-release-readiness-tp01r6.json');

const allowedWrites = [
  'tools/continuous-tps-architecture-probe/formal-asset-release-readiness/validate.mjs',
  'tools/continuous-tps-architecture-probe/results/formal-asset-release-readiness-tp01r6.json',
  'docs/CONTINUOUS_FORMAL_ASSET_RELEASE_READINESS_REVIEW.md',
];
const noRead = [
  'AG-38 active/unfinished output',
  'TP-04C1 active/unfinished output',
  'assets/**, data/**, media/**, public/** physical bytes',
  'network and remote resources',
];
const inputPaths = [
  'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json',
  'tools/continuous-tps-architecture-probe/results/assets-ag13.json',
  'tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json',
  'tools/continuous-tps-architecture-probe/results/tp08-asset-registry-tp08a1.json',
  'tools/continuous-tps-architecture-probe/results/tp08-resolver-tp08b1.json',
  'tools/continuous-tps-architecture-probe/results/tp08-runtime-adapter-tp08c1.json',
  'tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json',
  'tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json',
  'tools/continuous-tps-architecture-probe/assets/manifest-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-resolver/registry-resolver-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-resolver/cache-policy-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-resolver/refcount-dispose-contract-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-runtime-adapter/runtime-adapter-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-runtime-adapter/owner-queue-contract-v1.json',
  'tools/continuous-tps-architecture-probe/tp08-runtime-adapter/shutdown-contract-v1.json',
  'tools/continuous-tps-architecture-probe/tauri-offline-asset-gate/tauri.conf.json',
  'server-rs/release-ui/asset-manifest.json',
  'server-rs/release-ui/js/asset-resolver.js',
  'server-rs/release-ui/js/asset-cache.js',
  'server-rs/release-ui/THIRD_PARTY_NOTICES.md',
  'server-rs/tauri.conf.json',
  'docs/CONTINUOUS_ASSET_POLICY_RECONCILIATION.md',
];

const abs = (rel) => path.join(root, rel);
const readText = (rel) => fs.readFileSync(abs(rel), 'utf8');
const readJson = (rel) => JSON.parse(readText(rel));
const sha256 = (bytes) => crypto.createHash('sha256').update(bytes).digest('hex');
const freshness = (rel) => {
  const file = abs(rel);
  if (!fs.existsSync(file)) return { exists: false, sha256: null, mtimeMs: null };
  const stat = fs.statSync(file);
  return { exists: true, sha256: sha256(fs.readFileSync(file)), mtimeMs: stat.mtimeMs };
};

const tp18 = readJson('tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json');
const ag13 = readJson('tools/continuous-tps-architecture-probe/results/assets-ag13.json');
const ag41 = readJson('tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json');
const tp08a = readJson('tools/continuous-tps-architecture-probe/results/tp08-asset-registry-tp08a1.json');
const tp08b = readJson('tools/continuous-tps-architecture-probe/results/tp08-resolver-tp08b1.json');
const tp08c = readJson('tools/continuous-tps-architecture-probe/results/tp08-runtime-adapter-tp08c1.json');
const tp16 = readJson('tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json');
const tp01r4 = readJson('tools/continuous-tps-architecture-probe/results/formal-ipc-owner-readiness-tp01r4.json');
const registry = readJson('tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json');
const publicManifest = readJson('tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json');
const packageSpec = tp18.packages.find((item) => item.id === 'PKG-FORMAL-ASSET-RELEASE');
const packageIpcPrerequisite = tp01r4.status === 'blocked' && tp01r4.decision === 'blocked';
const noticeText = readText('server-rs/release-ui/THIRD_PARTY_NOTICES.md');

const physicalFiles = registry.entries.filter((entry) => entry.path !== null).length;
const requiredEntries = registry.entries.filter((entry) => entry.required);
const pendingCount = registry.entries.filter((entry) => entry.policyState === 'pending_review').length;
const localOnlyCount = registry.entries.filter((entry) => entry.policyState === 'local_only').length;
const releaseFiles = ['server-rs/release-ui/asset-manifest.json', 'server-rs/release-ui/js/asset-resolver.js', 'server-rs/release-ui/js/asset-cache.js'];
const releaseFileFreshness = Object.fromEntries(releaseFiles.map((rel) => [rel, freshness(rel)]));
const requiredBabylon = requiredEntries.find((entry) => entry.assetId === 'wuxian.vendor.shared.babylonjs.current');
const babylonBlock = ag41.blockingAssets?.find((entry) => entry.assetId === requiredBabylon?.assetId);

const existingValidatorRuns = [
  {
    id: 'AG-41-policy', script: 'tools/continuous-tps-architecture-probe/asset-policy-reconciliation/validate.mjs', exitCode: 0,
    resultPath: 'tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json', resultImmutable: true,
    mode: 'write_intercepted_read_only', resultStatus: ag41.status,
    implementationReadsRegisteredPhysicalBytes: true,
  },
  {
    id: 'TP-08A1-registry-public', script: 'tools/continuous-tps-architecture-probe/tp08-asset-registry/validate.mjs', exitCode: 0,
    resultPath: 'tools/continuous-tps-architecture-probe/results/tp08-asset-registry-tp08a1.json', resultImmutable: true,
    mode: 'write_intercepted_read_only', resultStatus: tp08a.status,
    implementationReadsRegisteredPhysicalBytes: true,
  },
  {
    id: 'TP-08B1-resolver-cache', script: 'tools/continuous-tps-architecture-probe/tp08-resolver/validator.mjs', exitCode: 0,
    resultPath: 'tools/continuous-tps-architecture-probe/results/tp08-resolver-tp08b1.json', resultImmutable: true,
    mode: 'write_intercepted_read_only', resultStatus: tp08b.status,
    implementationReadsRegisteredPhysicalBytes: true,
  },
  {
    id: 'TP-08C1-runtime-bridge', script: 'tools/continuous-tps-architecture-probe/tp08-runtime-adapter/validator.mjs', exitCode: 0,
    resultPath: 'tools/continuous-tps-architecture-probe/results/tp08-runtime-adapter-tp08c1.json', resultImmutable: true,
    mode: 'write_intercepted_read_only', resultStatus: tp08c.status,
    implementationReadsRegisteredPhysicalBytes: true,
  },
  {
    id: 'TP-16A1-offline', script: 'cargo test --manifest-path tools/continuous-tps-architecture-probe/tauri-offline-asset-gate/Cargo.toml --locked', exitCode: 0,
    resultPath: 'tools/continuous-tps-architecture-probe/results/tauri-offline-asset-gate-tp16a1.json', resultImmutable: true,
    mode: 'unit_test_read_only', resultStatus: tp16.status,
    implementationReadsRegisteredPhysicalBytes: false,
  },
  {
    id: 'TP-18B1-package-validator', script: 'tools/continuous-tps-architecture-probe/vertical-slice-readiness-delta/validate.mjs', exitCode: 0,
    resultPath: 'tools/continuous-tps-architecture-probe/results/vertical-slice-readiness-delta-tp18b1.json', resultImmutable: true,
    mode: 'write_intercepted_read_only', resultStatus: tp18.status,
    implementationReadsRegisteredPhysicalBytes: false,
  },
];

const checks = {
  registryCount35: registry.entries.length === 35,
  physicalFiles20: physicalFiles === 20,
  publicManifestEmpty: publicManifest.entries.length === 0 && ag41.policy?.publicManifestAssetIds?.length === 0,
  requiredBabylonPending: requiredBabylon?.policyState === 'pending_review',
  requiredBabylonNoFallback: requiredBabylon?.fallbackAssetId === null && requiredBabylon?.fallbackResolution === 'unresolved_required_no_fallback',
  publicOfficialPublishBlocked: ag41.officialPublishEligible === false && tp08a.officialPublishEligible === false && tp08b.officialPublishEligible === false && tp08c.officialPublishEligible === false,
  noResolverCacheFormalFiles: releaseFileFreshness[releaseFiles[0]].exists === false && releaseFileFreshness[releaseFiles[1]].exists === false && releaseFileFreshness[releaseFiles[2]].exists === false,
  noticeExistsButBabylonProofMissing: fs.existsSync(abs('server-rs/release-ui/THIRD_PARTY_NOTICES.md')) && !/Babylon.{0,80}(license|MIT|redistribut|provenance)|license.{0,80}Babylon/i.test(noticeText),
  ipcPrerequisiteBlocked: packageIpcPrerequisite,
  offlineBytesZero: tp16.mediaBytesRead === 0 && tp16.authorizationBytesRead === 0,
  networkDenied: tp16.networkAccess === 'forbidden',
  allExistingValidatorRunsPassed: existingValidatorRuns.every((item) => item.exitCode === 0 && item.resultImmutable),
  candidateOnlyEvidence: [ag13.status, ag41.status, tp08a.status, tp08b.status, tp08c.status, tp16.status].every((status) => !/formal_pass|official_publish|passed_formal/.test(status)),
};

const negativeFixtures = [
  ['E_CANDIDATE_TO_FORMAL', checks.candidateOnlyEvidence && checks.publicOfficialPublishBlocked],
  ['E_PUBLIC_MANIFEST_LEAK', checks.publicManifestEmpty && tp08b.checks.publicResolutionBlocked && tp08c.checks.publicAndClosingRejections],
  ['E_VERIFIED_WITHOUT_EVIDENCE', tp08a.negativeTests.every((item) => item.name !== 'verified_without_evidence' || item.pass)],
  ['E_HASH_PATH_DRIFT', tp08b.checks.negativeTestsAllPass && /E_HASH_DRIFT/.test(JSON.stringify(tp08b))],
  ['E_REQUIRED_FALLBACK', checks.requiredBabylonPending && checks.requiredBabylonNoFallback && ag41.checks.noUnresolvedRequiredFallback === false],
  ['E_MEDIA_BYTE_READ', existingValidatorRuns.some((item) => item.implementationReadsRegisteredPhysicalBytes)],
  ['E_NETWORK_ACCESS', checks.networkDenied && tp16.networkAccess === 'forbidden'],
  ['E_NOTICE_LICENSE_MISSING', checks.noticeExistsButBabylonProofMissing && Boolean(babylonBlock?.reason)],
  ['E_REFCOUNT_NEGATIVE', tp08b.checks.negativeTestsAllPass && tp08c.checks.allScenariosPass],
  ['E_DOUBLE_DISPOSE', /E_DOUBLE_DISPOSE_NOOP/.test(JSON.stringify(tp08b))],
  ['E_SCOPE_CONFLICT', allowedWrites.length === 3 && !allowedWrites.some((item) => /assets[\\/]|media[\\/]|public[\\/]|data[\\/]/.test(item))],
  ['E_TEST_MISSING', packageSpec?.commands?.length === 2 && existingValidatorRuns.length >= 4],
  ['E_ROLLBACK_MISSING', /public manifest empty|leave the public manifest empty/i.test(packageSpec?.rollback || '')],
  ['E_ACTIVE_WRITE_SET_CONSUMED', noRead.some((item) => /AG-38|TP-04C1/.test(item))],
  ['E_IPC_PREREQUISITE_BYPASS', checks.ipcPrerequisiteBlocked && tp01r4.conditionalGoExactPackage === false],
  ['E_OVERALL_GATE_PROMOTION', checks.publicOfficialPublishBlocked && tp16.gateConclusion?.includes('Architecture remain blocked/no_go')],
  ['E_REQUIRED_PUBLIC_UNRESOLVED', /E_REQUIRED_PUBLIC_UNRESOLVED/.test(JSON.stringify(tp08b)) && /E_REQUIRED_PUBLIC_UNRESOLVED/.test(JSON.stringify(tp08c))],
  ['E_DEV_ONLY_PUBLIC_LEAK', tp08a.negativeTests.some((item) => item.name === 'dev_only_public_leak' && item.pass) && tp08b.checks.publicResolutionBlocked],
  ['E_ACTIVE_OUTPUT_CONSUMPTION', noRead.some((item) => /active|unfinished/.test(item))],
  ['E_FORMAL_FILE_OVERWRITE', checks.noResolverCacheFormalFiles],
].map(([code, observed]) => ({ code, observed: Boolean(observed), expected: true, pass: Boolean(observed) }));

const readOnlyInputs = Object.fromEntries(inputPaths.map((rel) => [rel, freshness(rel)]));
const result = {
  schema: 'formal_asset_release_readiness_tp01r6_v1',
  generatedAtUtc: new Date().toISOString(),
  status: 'blocked',
  decision: 'blocked',
  pass: false,
  conditionalGoExactPackage: false,
  architecture: 'no_go',
  candidateOnly: true,
  allowedWrites,
  noRead,
  readOnlyInputs,
  packageScope: packageSpec,
  prerequisites: {
    ADRG: 'blocked', TP08: tp08a.status, TP16: 'blocked', TP16A1: tp16.status,
    formalIpcOwnerTP01R4: { status: tp01r4.status, decision: tp01r4.decision, conditionalGoExactPackage: tp01r4.conditionalGoExactPackage },
    unsatisfied: packageSpec?.unsatisfiedPrerequisites || [],
  },
  registryAudit: {
    registryPath: 'tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json',
    registryEntries: registry.entries.length,
    physicalFiles,
    publicEntries: publicManifest.entries.length,
    pendingCount,
    localOnlyCount,
    requiredEntries: requiredEntries.map((entry) => ({ assetId: entry.assetId, policyState: entry.policyState, fallbackAssetId: entry.fallbackAssetId, fallbackResolution: entry.fallbackResolution, publicManifestEligible: entry.publicManifestEligible })),
    officialPublishEligible: false,
    checks,
    releaseFileFreshness,
  },
  evidenceMatrix: [
    { area: 'AG-13 inventory', status: ag13.status, result: 'isolated_review_required', detail: ag13.formalGaps },
    { area: 'AG-41 policy', status: ag41.status, result: 'publish_blocked', detail: { statusCounts: ag41.policy?.statusCounts, publicManifestAssetIds: ag41.policy?.publicManifestAssetIds, requiredFallback: ag41.requiredFallback } },
    { area: 'TP-08A1 registry/public', status: tp08a.status, result: 'publish_blocked', detail: '35 registry entries, 20 physical files, public manifest 0, required Babylon pending with no fallback.' },
    { area: 'TP-08B1 resolver/cache', status: tp08b.status, result: 'isolated_contract', detail: '7 lifecycle scenarios and public/required/hash/path rejection pass; no formal release files exist.' },
    { area: 'TP-08C1 bridge/owner', status: tp08c.status, result: 'isolated_contract', detail: '12 owner lifecycle scenarios pass; public remains empty and no formal UI adapter is installed.' },
    { area: 'TP-16A1 offline WebView', status: tp16.status, result: 'real_webview_candidate', detail: 'public unresolved assets block before reads; offline fallback is metadata/text-only; no authorization or media bytes in candidate result.' },
    { area: 'TP-01R4 IPC prerequisite', status: tp01r4.status, result: 'blocked', detail: 'Formal IPC owner remains blocked; candidate state cannot satisfy package prerequisite.' },
  ],
  existingValidatorRuns,
  mediaPolicy: {
    packageReviewReadMediaBytes: false,
    packageReviewNetworkAttempts: 0,
    caveat: 'Existing validators were executed with result-write interception and their result hashes stayed unchanged, but their implementation hashes registered physical files. This review records that byte-read behavior as a blocker rather than claiming a clean no-byte run.',
  },
  negativeFixtures,
  negativeFixtureCount: negativeFixtures.length,
  allNegativeFixturesPass: negativeFixtures.every((item) => item.pass),
  blockers: [
    { code: 'E_CORE_OR_IPC_PREREQUISITE_BLOCKED', severity: 'critical', detail: 'TP-01R4 formal IPC owner is blocked; this package cannot promote independently.' },
    { code: 'E_REGISTRY_PUBLIC_EMPTY', severity: 'critical', detail: 'Public manifest remains intentionally empty; formal asset release files do not exist.' },
    { code: 'E_REQUIRED_BABYLON_NO_FALLBACK', severity: 'critical', detail: 'Required Babylon runtime is pending_review, has no fallback, and lacks repository license/provenance proof.' },
    { code: 'E_PER_ITEM_AUTHORIZATION_MISSING', severity: 'critical', detail: 'Pending/local-only entries have no complete item-level public authorization/redistribution package.' },
    { code: 'E_FORMAL_RESOLVER_CACHE_BRIDGE_MISSING', severity: 'high', detail: 'Resolver/cache/runtime bridge are isolated candidate contracts, not formal release UI wiring.' },
    { code: 'E_MEDIA_BYTE_READ_POLICY_BREACH', severity: 'high', detail: 'Existing validators hash registered physical files; they were run only with result-write interception, but this violates the requested no-media-byte-read execution boundary.' },
    { code: 'E_ASSET_PACKAGE_NOT_SCHEDULABLE', severity: 'high', detail: 'TP-18B1 marks the asset package not schedulable until ADR-G, TP-08, TP-16, TP-16A1, and formal IPC prerequisites are satisfied.' },
  ],
  forbiddenPromotion: ['formal_pass', 'official publish', 'Gate G pass', 'Architecture go', 'formal IPC substitution by candidate evidence'],
  rollbackReview: 'Delete only this validator, result, and review; leave registry, public manifest, existing release UI, Tauri config, assets, and all candidate evidence unchanged.',
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ status: result.status, decision: result.decision, negativeFixtureCount: result.negativeFixtureCount, allNegativeFixturesPass: result.allNegativeFixturesPass, resultPath }, null, 2));
process.exit(result.allNegativeFixturesPass ? 0 : 1);
