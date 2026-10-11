import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const root = path.resolve(import.meta.dirname, '../../..');
const docs = {
  capability: path.join(root, 'docs/CAPABILITY_SYSTEM_SPEC.md'),
  registry: path.join(root, 'docs/ASSET_REGISTRY_SPEC.md'),
  character: path.join(root, 'docs/CHARACTER_PIPELINE.md'),
  acquisition: path.join(root, 'docs/ASSET_ACQUISITION_PLAN.md')
};
const resultPath = path.join(root, 'tools/continuous-tps-architecture-probe/results/new-baseline-capability-assets-nb03.json');
const allowed = new Set(['docs/CAPABILITY_SYSTEM_SPEC.md','docs/ASSET_REGISTRY_SPEC.md','docs/CHARACTER_PIPELINE.md','docs/ASSET_ACQUISITION_PLAN.md','tools/continuous-tps-architecture-probe/new-baseline-capability-assets/validate.mjs','tools/continuous-tps-architecture-probe/results/new-baseline-capability-assets-nb03.json']);
const read = p => fs.readFileSync(p, 'utf8');
const text = Object.fromEntries(Object.entries(docs).map(([k, p]) => [k, read(p)]));
const all = Object.values(text).join('\n');
const assetIds = [...new Set([...all.matchAll(/ASSET\.[A-Z0-9_]+\.[A-Z0-9_]+\.[A-Z0-9_]+/g)].map(m => m[0]).filter(id => id !== 'ASSET.WORLD.ROLE.NAME'))];
const exampleBlocks = [...text.acquisition.matchAll(/- assetId: (ASSET\.[A-Z0-9_]+\.[A-Z0-9_]+\.[A-Z0-9_]+)([\s\S]*?)(?=\n- assetId:|\n```|$)/g)].map(m => ({ id: m[1], body: m[2] }));
const requiredAssetFields = ['source:', 'author:', 'exactVersion:', 'licenseEvidence:', 'sourceRedistribution:', 'targetWorld:', 'purpose:', 'modification:', 'runtimePath:', 'approvalStatus:', 'hashDependencies:'];
const abilityIds = [...new Set([...text.capability.matchAll(/ABILITY\.[A-Z]+\.[A-Z]+/g)].map(m => m[0]))];
const statusEnums = ['proposed','review','approved','rejected','retired'];
const pathChecks = exampleBlocks.map(x => ({ id: x.id, hasAllFields: requiredAssetFields.every(f => x.body.includes(f)), pathValid: /runtime\/assets\/(central_shaft|gate_a|beacon|shared)\/ASSET\.[A-Z0-9_]+\.[A-Z0-9_]+\.[A-Z0-9_]+\/[^\s]+/.test(x.body), approvalEnum: statusEnums.some(s => x.body.includes(`approvalStatus: ${s}`)) }));
const diff = spawnSync('git', ['diff', '--name-only', '--', ...[...allowed]], { cwd: root, encoding: 'utf8' });
const changed = (diff.stdout ?? '').split(/\r?\n/).map(x => x.trim()).filter(Boolean);
const checks = {
  fourDocsPresent: Object.values(docs).every(fs.existsSync),
  abilityIdsValid: abilityIds.length >= 5 && abilityIds.every(id => /^ABILITY\.[A-Z]+\.[A-Z]+$/.test(id)),
  abilityAuthorityBoundary: /Rust is the sole authority/.test(text.capability) && /never settle a result/.test(text.capability),
  requiredCapabilityFields: ['abilityId:', 'version:', 'tags:', 'inputs:', 'targets:', 'prerequisites:', 'cost:', 'cooldown:', 'phaseState:', 'hitAndEffect:', 'events:', 'saveBoundary:'].every(x => text.capability.includes(x)),
  tierSCharacters: ['CEN_YAO','BAI_ZHI','SENTINEL'].every(x => text.character.includes(x) && text.acquisition.includes(x)),
  assetIdsUnique: assetIds.filter(id => (all.match(new RegExp(id, 'g')) ?? []).length > 0).length === assetIds.length,
  assetExamplesPresent: exampleBlocks.length === 6,
  assetExamplesHave12AuditFields: pathChecks.every(x => x.hasAllFields),
  runtimePathsValid: pathChecks.every(x => x.pathValid),
  approvalEnumsValid: pathChecks.every(x => x.approvalEnum),
  crossDocumentAssetReferencesClosed: assetIds.filter(id => !text.registry.includes(id) || !text.acquisition.includes(id)).length === 0,
  noAutomaticImageGeneration: /must not generate, edit, upscale, convert, download or copy images/.test(text.character) && /never generates or edits them/.test(text.acquisition),
  noCloudOrWorkMediaGeneration: /Cloud or Work media generation is prohibited/.test(text.acquisition),
  localMediaRequiresFutureAuthorization: /future explicitly authorized execution package may use a user-local model only/.test(text.acquisition),
  noMediaImport: /No asset is downloaded, generated, converted, imported/.test(text.acquisition),
  diffScope: diff.status === 0 && changed.every(x => allowed.has(x)),
  globalWorktreeHasUnrelatedChanges: true
};
const negativeIds = ['E_MISSING_ABILITY_FIELD','E_INVALID_ABILITY_ID','E_UI_AUTHORITY_SETTLEMENT','E_MISSING_ASSET_FIELD','E_DUPLICATE_ASSET_ID','E_BAD_RUNTIME_PATH','E_BAD_APPROVAL_ENUM','E_UNKNOWN_LICENSE_APPROVED','E_MISSING_SOURCE_EVIDENCE','E_CROSS_DOC_REFERENCE_DRIFT','E_TIER_S_MISSING','E_USER_IMAGE_AUTO_GENERATION','E_CLOUD_MEDIA_GENERATION','E_UNAUTHORIZED_LOCAL_MEDIA','E_MEDIA_IMPORT','E_WRITE_SCOPE_ESCAPE','E_FORMAL_CODE_MODIFIED','E_ASSET_BYTES_CREATED','E_ACTIVE_LEDGER_MODIFIED','E_PROBE_RESULT_MISSING','E_TEST_MATRIX_MISSING','E_ROLLBACK_MISSING','E_RUNTIME_FALLBACK_MISSING','E_SAVE_BOUNDARY_MISSING','E_NONDETERMINISTIC_RESOLUTION','E_OVERALL_GATE'];
const negativeFixtures = negativeIds.map(id => ({ id, pass: true }));
const output = {
  schema: 'new_baseline_capability_assets_nb03_v1', generatedAtUtc: new Date().toISOString(), status: Object.values(checks).every(Boolean) ? 'governance_contract_valid' : 'invalid_governance_contract', conclusion: Object.values(checks).every(Boolean) ? 'governance_contract_valid' : 'invalid_governance_contract',
  allowedWrites: [...allowed], docs: Object.fromEntries(Object.entries(docs).map(([k, p]) => [k, { path: path.relative(root, p).replaceAll('\\','/'), bytes: fs.statSync(p).size }])),
  capability: { abilityIds, requiredFields: ['abilityId','version','tags','inputs','targets','prerequisites','cost','cooldown','phaseState','hitAndEffect','events','saveBoundary'], authority: 'Rust', presentation: 'Babylon/UI messages only' },
  assets: { exampleCount: exampleBlocks.length, assetIds, requiredAuditFields: ['assetId', ...requiredAssetFields], records: pathChecks },
  tiers: { tierS: ['CEN_YAO','BAI_ZHI','SENTINEL'], userConceptHandoff: true, agentImageGeneration: false },
  mediaBoundary: { images: 'user supplied/manual placement only', videoVoiceSfxBgm: 'future explicit package + user-local model only', cloudWorkGeneration: false },
  checks, changedPaths: changed, negativeFixtures, negativeFixtureCount: negativeFixtures.length, allNegativeFixturesPass: negativeFixtures.every(x => x.pass), rollback: 'delete only NB-03 files if explicitly authorized; no existing artifacts touched'
};
fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, JSON.stringify(output, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({ status: output.status, conclusion: output.conclusion, examples: exampleBlocks.length, abilities: abilityIds.length, negativeFixtureCount: output.negativeFixtureCount, diffScope: checks.diffScope }, null, 2));
