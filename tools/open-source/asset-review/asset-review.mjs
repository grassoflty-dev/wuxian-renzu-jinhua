import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..', '..');
const BASE_SHA = '51533dfce9c1e88de51860c94c6b1b4ea16aee55';
const INVENTORY = path.join(ROOT, 'governance/assets/inventory.json');
const PUBLIC_MANIFEST = path.join(ROOT, 'governance/assets/public-release-manifest.json');
const STATUSES = new Set(['verified_original', 'verified_redistributable', 'verified_generated_redistributable', 'pending_review', 'local_only', 'legacy_unused', 'rejected']);
const PUBLIC_STATUSES = new Set(['verified_original', 'verified_redistributable', 'verified_generated_redistributable']);
const ROOTS = ['server-rs/ui/assets/', 'server-rs/ui_3d_archive/', 'tools/design/', 'tools/shots/'];
const EXTENSIONS = new Set(['.png','.jpg','.jpeg','.webp','.gif','.svg','.glb','.gltf','.fbx','.obj','.wav','.mp3','.ogg','.flac','.mp4','.webm','.ttf','.otf']);

function git(args) {
  return execFileSync('git', args, { cwd: ROOT, encoding: 'utf8' });
}

function baseEntries() {
  const result = new Map();
  for (const line of git(['ls-tree', '-r', '-l', BASE_SHA]).split(/\r?\n/)) {
    if (!line) continue;
    const match = line.match(/^\d+\s+blob\s+([0-9a-f]{40})\s+(\d+)\t(.+)$/);
    if (match) result.set(match[3].replaceAll('\\', '/'), { hash: match[1], bytes: Number(match[2]) });
  }
  return result;
}

function currentAssetDrift() {
  const tracked = git(['diff', '--name-only', BASE_SHA, '--', ...ROOTS]).split(/\r?\n/).filter(Boolean);
  const untracked = git(['ls-files', '--others', '--exclude-standard', '--', ...ROOTS]).split(/\r?\n/).filter(Boolean);
  return [...new Set([...tracked, ...untracked].map(file => file.replaceAll('\\', '/')).filter(isAsset))].sort();
}

function isAsset(file) {
  const normalized = file.replaceAll('\\', '/');
  return ROOTS.some(root => normalized.startsWith(root)) && EXTENSIONS.has(path.posix.extname(normalized).toLowerCase());
}

function assetType(file) {
  const ext = path.posix.extname(file).toLowerCase();
  if (['.png','.jpg','.jpeg','.webp','.gif','.svg'].includes(ext)) return 'image';
  if (['.wav','.mp3','.ogg','.flac'].includes(ext)) return 'audio';
  if (['.mp4','.webm'].includes(ext)) return 'video';
  if (['.glb','.gltf','.fbx','.obj'].includes(ext)) return 'model';
  if (['.ttf','.otf'].includes(ext)) return 'font';
  return 'other';
}

function assetId(file) {
  return `asset.${file.replace(/\.[^.]+$/, '').replace(/[^A-Za-z0-9]+/g, '.').replace(/^\.|\.$/g, '').toLowerCase()}`;
}

function makeInventory() {
  const tree = baseEntries();
  const drift = currentAssetDrift();
  const assets = [...tree.entries()].filter(([file]) => isAsset(file)).map(([file, meta]) => {
    const runtime = file.startsWith('server-rs/ui/assets/');
    return {
      asset_id: assetId(file),
      type: assetType(file),
      source_path: file,
      runtime_path: runtime ? file : null,
      source: runtime ? 'repository-baseline; provenance evidence pending' : 'repository-baseline local production/archive artifact',
      author: null,
      license: null,
      license_evidence: [],
      redistributable_source: false,
      modified: false,
      version: `base-${BASE_SHA.slice(0, 12)}`,
      hash: `git-blob-sha1:${meta.hash}`,
      world: 'unassigned',
      lod: null,
      status: runtime ? 'pending_review' : 'local_only'
    };
  }).sort((a,b) => a.source_path.localeCompare(b.source_path));
  return {
    schema_version: 'oss-asset-provenance-v1',
    base_sha: BASE_SHA,
    inventory_method: 'git ls-tree metadata only; media bytes were not opened',
    source_evidence: [
      'tools/assets_manifest.json',
      'tools/assets_manifest_cops2.json',
      'tools/assets_manifest_zy_worlds.json',
      'tools/assets_manifest_zhouyuan.json',
      'tools/design/*_assets*.md source logs',
      'tools/design/wan_prompts.md and other versioned design/source logs'
    ],
    active_asset_drift: drift,
    rescan_required: drift.length > 0,
    assets
  };
}

function safeRepoPath(value) {
  if (typeof value !== 'string' || !value || path.isAbsolute(value)) return false;
  const normalized = value.replaceAll('\\', '/');
  return !normalized.startsWith('/') && !normalized.split('/').includes('..') && !normalized.includes(':');
}

function assetSha256(sourcePath) {
  if (!safeRepoPath(sourcePath)) return null;
  try {
    const rootReal = fs.realpathSync(ROOT);
    const absolute = path.resolve(rootReal, sourcePath);
    const actualPath = fs.realpathSync(absolute);
    const relative = path.relative(rootReal, actualPath);
    if (relative.startsWith('..') || path.isAbsolute(relative) || !fs.statSync(actualPath).isFile()) return null;
    return createHash('sha256').update(fs.readFileSync(actualPath)).digest('hex');
  } catch {
    return null;
  }
}

function completeLegacyUnusedEvidence(item) {
  const e = item.non_use_evidence;
  return !!e && safeRepoPath(e.formal_entry_path) && /^[0-9a-f]{40}$/.test(e.formal_entry_commit ?? '') &&
    safeRepoPath(e.reference_check_record_path) && /^[0-9a-f]{40}$/.test(e.reference_check_commit ?? '') &&
    /^[0-9a-f]{64}$/.test(e.reference_check_record_sha256 ?? '') && !!e.checked_at && !!e.checked_by &&
    !!e.checked_scope && !!e.method && e.result === 'complete_no_runtime_reference_found';
}

function generatedEvidenceErrors(item, add, fileHashResolver) {
  const evidence = item.generation_evidence;
  const nonEmpty = value => typeof value === 'string' && value.trim().length > 0;
  const recorded = e => !!e && nonEmpty(e.reviewer) && nonEmpty(e.reviewed_at);
  if (!evidence) {
    add('GENERATED_EVIDENCE_MISSING');
    return;
  }
  const g = evidence.generation_record;
  if (!g || g.provider !== 'OpenAI' || g.product !== 'ChatGPT' || !nonEmpty(g.conversation_id) || !nonEmpty(g.record_reference) || !nonEmpty(g.generated_at)) add('GENERATION_CHAIN_INCOMPLETE');
  const input = evidence.input_rights;
  if (!input || input.rights_confirmed !== true || !Array.isArray(input.evidence) || !input.evidence.some(nonEmpty) || !recorded(input)) add('INPUT_RIGHTS_EVIDENCE_INCOMPLETE');
  const references = evidence.third_party_reference_review;
  if (!references || references.unauthorized_references_excluded !== true || !recorded(references) || !nonEmpty(references.record_reference)) add('THIRD_PARTY_REFERENCE_REVIEW_INCOMPLETE');
  const copying = evidence.copying_instruction_review;
  if (!copying || copying.unauthorized_copying_instructions_excluded !== true || !recorded(copying) || !nonEmpty(copying.record_reference)) add('COPYING_INSTRUCTION_REVIEW_INCOMPLETE');
  const terms = evidence.openai_terms;
  if (!terms || !nonEmpty(terms.reference) || !nonEmpty(terms.version) || !recorded(terms) || terms.applicable_law_considered !== true || terms.non_unique_output_acknowledged !== true || terms.copyright_not_guaranteed_acknowledged !== true) add('OPENAI_TERMS_REVIEW_INCOMPLETE');
  const review = evidence.manual_review;
  if (!review || !recorded(review) || review.logo_check !== 'passed' || review.similarity_check !== 'passed' || review.random_text_check !== 'passed' || !nonEmpty(review.record_reference)) add('MANUAL_REVIEW_INCOMPLETE');
  if (item.license !== 'CC0-1.0' || !Array.isArray(item.license_evidence) || !item.license_evidence.some(nonEmpty)) add('GENERATED_LICENSE_EVIDENCE_INCOMPLETE');
  if (item.type !== 'image') add('GENERATED_ASSET_TYPE_INVALID');
  if (item.redistributable_source !== true) add('GENERATED_REDISTRIBUTION_NOT_VERIFIED');
  if (typeof item.runtime_use !== 'boolean') add('RUNTIME_USE_UNDECLARED');
  if (!/^[0-9a-f]{64}$/.test(evidence.final_sha256 ?? '')) add('FINAL_SHA256_MISSING');
  else {
    const actual = fileHashResolver(item.source_path);
    if (!actual) add('FINAL_ASSET_SHA256_UNAVAILABLE');
    else if (actual !== evidence.final_sha256) add('FINAL_SHA256_MISMATCH');
  }
}

export function validateInventory(inventory, tree = baseEntries(), fileHashResolver = assetSha256) {
  const errors = [];
  const ids = new Set();
  for (const [index, item] of (inventory.assets ?? []).entries()) {
    const label = item.asset_id || `index:${index}`;
    if (ids.has(item.asset_id)) errors.push({ code: 'DUPLICATE_ID', asset_id: label });
    ids.add(item.asset_id);
    if (!STATUSES.has(item.status)) errors.push({ code: 'UNKNOWN_STATUS', asset_id: label });
    if (!safeRepoPath(item.source_path) || (item.runtime_path !== null && !safeRepoPath(item.runtime_path))) errors.push({ code: 'PATH_OUT_OF_SCOPE', asset_id: label });
    const base = tree.get(item.source_path);
    if (!base) errors.push({ code: 'SOURCE_NOT_IN_BASE', asset_id: label });
    else if (item.hash !== `git-blob-sha1:${base.hash}`) errors.push({ code: 'HASH_MISMATCH', asset_id: label });
    if (PUBLIC_STATUSES.has(item.status)) {
      if (!item.author || !item.license || !Array.isArray(item.license_evidence) || item.license_evidence.length === 0) errors.push({ code: 'LICENSE_EVIDENCE_MISSING', asset_id: label });
      if (!item.redistributable_source) errors.push({ code: 'REDISTRIBUTION_NOT_VERIFIED', asset_id: label });
      if (!['human_original', 'third_party', 'ai_generated'].includes(item.source_type)) errors.push({ code: 'SOURCE_TYPE_REQUIRED', asset_id: label });
    }
    if (item.status === 'legacy_unused' && !completeLegacyUnusedEvidence(item)) errors.push({ code: 'LEGACY_UNUSED_EVIDENCE_MISSING', asset_id: label });
    if (item.source_type === 'ai_generated' && item.status === 'verified_original') errors.push({ code: 'AI_GENERATED_CANNOT_BE_ORIGINAL', asset_id: label });
    if (item.status === 'verified_generated_redistributable' && item.source_type !== 'ai_generated') errors.push({ code: 'GENERATED_STATUS_REQUIRES_AI_SOURCE', asset_id: label });
    if (PUBLIC_STATUSES.has(item.status) && item.source_type === 'ai_generated') {
      generatedEvidenceErrors(item, code => errors.push({ code, asset_id: label }), fileHashResolver);
    }
  }
  return errors;
}

export function makePublicManifest(inventory, tree = baseEntries(), fileHashResolver = assetSha256) {
  const validationErrors = validateInventory(inventory, tree, fileHashResolver);
  if (validationErrors.length) throw new Error(`inventory invalid: ${JSON.stringify(validationErrors)}`);
  const assets = [];
  const exclusions = [];
  for (const item of inventory.assets) {
    if (PUBLIC_STATUSES.has(item.status)) assets.push(item);
    else exclusions.push({ asset_id: item.asset_id, source_path: item.source_path, status: item.status, reason: item.status === 'pending_review' ? 'license_and_provenance_not_verified' : item.status === 'local_only' ? 'local_or_production_artifact_not_for_release' : item.status === 'legacy_unused' ? 'not_used_by_formal_runtime' : 'rejected_by_review' });
  }
  const eligibility = ['verified_original', 'verified_redistributable'];
  if (assets.some(item => item.status === 'verified_generated_redistributable')) eligibility.push('verified_generated_redistributable');
  return {
    schema_version: 'oss-public-asset-manifest-v1',
    base_sha: inventory.base_sha,
    eligibility,
    assets,
    exclusions,
    summary: { included: assets.length, excluded: exclusions.length }
  };
}

function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

function checkFixtures() {
  const fixtureDir = path.join(HERE, 'fixtures');
  const expected = new Map([
    ['pending-review.json', 'license_and_provenance_not_verified'],
    ['local-only.json', 'local_or_production_artifact_not_for_release'],
    ['wrong-hash.json', 'HASH_MISMATCH'],
    ['path-escape.json', 'PATH_OUT_OF_SCOPE'],
    ['duplicate-id.json', 'DUPLICATE_ID'],
    ['unknown-status.json', 'UNKNOWN_STATUS']
  ]);
  const results = [];
  for (const [name, code] of expected) {
    const fixture = JSON.parse(fs.readFileSync(path.join(fixtureDir, name), 'utf8'));
    let observed = [];
    try {
      const validation = validateInventory(fixture);
      observed = validation.map(x => x.code);
      if (!validation.length) observed = makePublicManifest(fixture).exclusions.map(x => x.reason);
      else if (name === 'pending-review.json' || name === 'local-only.json') observed = makePublicManifest(fixture).exclusions.map(x => x.reason);
    } catch (error) {
      observed.push(String(error));
    }
    if (!observed.some(value => value.includes(code))) throw new Error(`${name} did not produce ${code}: ${observed.join(',')}`);
    results.push({ fixture: name, expected: code, pass: true });
  }
  const fixtureTree = new Map([
    ['fixtures/synthetic-legacy.png', { hash: '1'.repeat(40) }],
    ['fixtures/synthetic-generated.png', { hash: '2'.repeat(40) }]
  ]);
  for (const name of ['legacy-unused.json', 'legacy-unused-missing-evidence.json']) {
    const fixture = JSON.parse(fs.readFileSync(path.join(fixtureDir, name), 'utf8'));
    const validation = validateInventory(fixture, fixtureTree);
    const expected = name === 'legacy-unused.json' ? 'not_used_by_formal_runtime' : 'LEGACY_UNUSED_EVIDENCE_MISSING';
    const observed = validation.length ? validation.map(x => x.code) : makePublicManifest(fixture, fixtureTree).exclusions.map(x => x.reason);
    if (!observed.includes(expected)) throw new Error(`${name} did not produce ${expected}: ${observed.join(',')}`);
    results.push({ fixture: name, expected, pass: true });
  }
  return results;
}

function syntheticGeneratedAsset(overrides = {}) {
  const bytes = Buffer.from('OSS-05 synthetic fixture payload; not an image');
  const finalSha = createHash('sha256').update(bytes).digest('hex');
  const sourcePath = 'fixtures/synthetic-generated.png';
  const item = {
    asset_id: 'fixture.generated', type: 'image', source_path: sourcePath, runtime_path: null,
    source: 'synthetic test record', source_type: 'ai_generated', author: 'fixture reviewer',
    license: 'CC0-1.0', license_evidence: ['fixture:cc0-assignment'], redistributable_source: true,
    runtime_use: false, modified: false, version: 'fixture', hash: `git-blob-sha1:${'2'.repeat(40)}`,
    world: 'unassigned', lod: null, status: 'verified_generated_redistributable',
    generation_evidence: {
      generation_record: { provider: 'OpenAI', product: 'ChatGPT', conversation_id: 'fixture-conversation', record_reference: 'fixture:generation', generated_at: '2026-09-23T00:00:00Z' },
      input_rights: { rights_confirmed: true, evidence: ['fixture:input-rights'], reviewer: 'fixture-reviewer', reviewed_at: '2026-09-23T00:00:00Z' },
      third_party_reference_review: { unauthorized_references_excluded: true, reviewer: 'fixture-reviewer', reviewed_at: '2026-09-23T00:00:00Z', record_reference: 'fixture:references' },
      copying_instruction_review: { unauthorized_copying_instructions_excluded: true, reviewer: 'fixture-reviewer', reviewed_at: '2026-09-23T00:00:00Z', record_reference: 'fixture:copying' },
      openai_terms: { reference: 'fixture:terms', version: 'fixture-version', reviewer: 'fixture-reviewer', reviewed_at: '2026-09-23T00:00:00Z', applicable_law_considered: true, non_unique_output_acknowledged: true, copyright_not_guaranteed_acknowledged: true },
      manual_review: { reviewer: 'fixture-reviewer', reviewed_at: '2026-09-23T00:00:00Z', logo_check: 'passed', similarity_check: 'passed', random_text_check: 'passed', record_reference: 'fixture:visual-review' },
      final_sha256: finalSha
    },
    ...overrides
  };
  const inventory = { schema_version: 'oss-asset-provenance-v1', base_sha: BASE_SHA, assets: [item] };
  const tree = new Map([[sourcePath, { hash: '2'.repeat(40) }]]);
  const resolver = pathName => pathName === sourcePath ? finalSha : null;
  return { item, inventory, tree, resolver, finalSha };
}

function checkGeneratedCases() {
  const results = [];
  const assertRejected = (name, sample, expected) => {
    const errors = validateInventory(sample.inventory, sample.tree, sample.resolver);
    if (!errors.some(error => error.code === expected)) throw new Error(`${name} did not produce ${expected}: ${JSON.stringify(errors)}`);
    results.push({ case: name, expected, pass: true });
  };
  const complete = syntheticGeneratedAsset();
  const publicManifest = makePublicManifest(complete.inventory, complete.tree, complete.resolver);
  if (publicManifest.assets.length !== 1 || publicManifest.assets[0].runtime_use !== false || !publicManifest.eligibility.includes('verified_generated_redistributable')) throw new Error('complete generated fixture was not included with matching eligibility metadata and runtime_use preserved');
  results.push({ case: 'complete-generated-hash-verified', expected: 'public_eligible_runtime_use_false_preserved', pass: true });

  const missingSourceType = syntheticGeneratedAsset({ source_type: undefined });
  delete missingSourceType.item.source_type;
  assertRejected('generated-missing-source-type', missingSourceType, 'SOURCE_TYPE_REQUIRED');

  const missingChain = syntheticGeneratedAsset();
  delete missingChain.item.generation_evidence.generation_record;
  assertRejected('generated-missing-chatgpt-chain', missingChain, 'GENERATION_CHAIN_INCOMPLETE');

  const missingInputRights = syntheticGeneratedAsset();
  delete missingInputRights.item.generation_evidence.input_rights;
  assertRejected('generated-missing-input-rights', missingInputRights, 'INPUT_RIGHTS_EVIDENCE_INCOMPLETE');

  const missingReferenceReview = syntheticGeneratedAsset();
  delete missingReferenceReview.item.generation_evidence.third_party_reference_review;
  assertRejected('generated-missing-third-party-review', missingReferenceReview, 'THIRD_PARTY_REFERENCE_REVIEW_INCOMPLETE');

  const missingCopyReview = syntheticGeneratedAsset();
  delete missingCopyReview.item.generation_evidence.copying_instruction_review;
  assertRejected('generated-missing-copying-review', missingCopyReview, 'COPYING_INSTRUCTION_REVIEW_INCOMPLETE');

  const missingTerms = syntheticGeneratedAsset();
  delete missingTerms.item.generation_evidence.openai_terms;
  assertRejected('generated-missing-openai-terms-review', missingTerms, 'OPENAI_TERMS_REVIEW_INCOMPLETE');

  for (const reviewField of ['logo_check', 'similarity_check', 'random_text_check']) {
    const sample = syntheticGeneratedAsset();
    delete sample.item.generation_evidence.manual_review[reviewField];
    assertRejected(`generated-missing-${reviewField}`, sample, 'MANUAL_REVIEW_INCOMPLETE');
  }

  const wrongLicense = syntheticGeneratedAsset({ license: 'GPL-3.0-only' });
  assertRejected('generated-non-cc0-license', wrongLicense, 'GENERATED_LICENSE_EVIDENCE_INCOMPLETE');

  const noLicenseEvidence = syntheticGeneratedAsset({ license_evidence: [] });
  assertRejected('generated-missing-license-evidence', noLicenseEvidence, 'GENERATED_LICENSE_EVIDENCE_INCOMPLETE');

  const noRedistribution = syntheticGeneratedAsset({ redistributable_source: false });
  assertRejected('generated-redistribution-unverified', noRedistribution, 'GENERATED_REDISTRIBUTION_NOT_VERIFIED');

  const undeclaredRuntimeUse = syntheticGeneratedAsset({ runtime_use: undefined });
  delete undeclaredRuntimeUse.item.runtime_use;
  assertRejected('generated-runtime-use-undeclared', undeclaredRuntimeUse, 'RUNTIME_USE_UNDECLARED');

  const badHash = syntheticGeneratedAsset();
  badHash.item.generation_evidence.final_sha256 = '0'.repeat(64);
  assertRejected('generated-wrong-final-sha256', badHash, 'FINAL_SHA256_MISMATCH');

  const aiOriginal = syntheticGeneratedAsset({ status: 'verified_original' });
  assertRejected('ai-generated-original-status', aiOriginal, 'AI_GENERATED_CANNOT_BE_ORIGINAL');

  const compatibleStatus = syntheticGeneratedAsset({ status: 'verified_redistributable' });
  const compatibleManifest = makePublicManifest(compatibleStatus.inventory, compatibleStatus.tree, compatibleStatus.resolver);
  if (compatibleManifest.assets.length !== 1 || compatibleManifest.assets[0].runtime_use !== false) throw new Error('AI-generated verified_redistributable compatibility path failed or changed runtime_use');
  results.push({ case: 'ai-generated-compatible-redistributable-status', expected: 'same-evidence-gate-and-runtime_use-preserved', pass: true });

  const legacyValid = JSON.parse(fs.readFileSync(path.join(HERE, 'fixtures', 'legacy-unused.json'), 'utf8'));
  const legacyManifest = makePublicManifest(legacyValid, new Map([['fixtures/synthetic-legacy.png', { hash: '1'.repeat(40) }]]));
  if (legacyManifest.assets.length !== 0 || legacyManifest.exclusions[0]?.reason !== 'not_used_by_formal_runtime') throw new Error('legacy_unused must remain excluded with its dedicated reason');
  results.push({ case: 'legacy-unused-stays-excluded', expected: 'not_used_by_formal_runtime', pass: true });
  return results;
}

function main() {
  const command = process.argv[2] ?? 'check';
  if (command === 'generate') {
    const inventory = makeInventory();
    writeJson(INVENTORY, inventory);
    writeJson(PUBLIC_MANIFEST, makePublicManifest(inventory));
    console.log(JSON.stringify({ status: 'generated', inventory: inventory.assets.length, public: 0 }, null, 2));
    return;
  }
  if (command === 'check') {
    const inventory = JSON.parse(fs.readFileSync(INVENTORY, 'utf8'));
    const publicManifest = JSON.parse(fs.readFileSync(PUBLIC_MANIFEST, 'utf8'));
    const errors = validateInventory(inventory);
    const drift = currentAssetDrift();
    const expected = makePublicManifest(inventory);
    if (errors.length) throw new Error(JSON.stringify(errors));
    if (JSON.stringify(inventory.active_asset_drift) !== JSON.stringify(drift) || inventory.rescan_required !== (drift.length > 0)) throw new Error('asset drift metadata is stale');
    if (JSON.stringify(publicManifest) !== JSON.stringify(expected)) throw new Error('public manifest is stale or was manually broadened');
    const fixtures = checkFixtures();
    const syntheticCases = checkGeneratedCases();
    console.log(JSON.stringify({ status: 'pass', inventory: inventory.assets.length, public: publicManifest.assets.length, excluded: publicManifest.exclusions.length, fixtures, syntheticCases }, null, 2));
    return;
  }
  throw new Error(`unknown command: ${command}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
