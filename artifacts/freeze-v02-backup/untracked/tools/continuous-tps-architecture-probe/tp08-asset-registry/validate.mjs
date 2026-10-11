import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const REGISTRY_PATH = path.join(HERE, "asset-registry-v1.json");
const AUTH_PATH = path.join(HERE, "item-authorization-evidence-v1.json");
const PUBLIC_PATH = path.join(HERE, "public-manifest-v1.json");
const AG13_MANIFEST_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/assets/manifest-v1.json");
const AG41_RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/tp08-asset-registry-tp08a1.json");

const STATES = new Set(["verified_original", "verified_redistributable", "pending_review", "local_only", "rejected"]);
const VERIFIED = new Set(["verified_original", "verified_redistributable"]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function absRepo(relative) {
  return path.join(ROOT, ...relative.split("/"));
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function unique(items) {
  return new Set(items).size === items.length;
}

function validateModel({ registry, authorization, publicManifest, ag13Manifest, strictRequiredFallback = false }) {
  const errors = [];
  const requiredUnresolved = [];
  const entries = Array.isArray(registry.entries) ? registry.entries : [];
  const authEntries = Array.isArray(authorization.entries) ? authorization.entries : [];
  const publicEntries = Array.isArray(publicManifest.entries) ? publicManifest.entries : [];
  const expectedById = new Map((ag13Manifest.entries ?? []).map((entry) => [entry.id, entry]));
  const registryById = new Map();
  const authById = new Map();

  if (registry.schema !== "tp08_asset_registry_v1") errors.push({ code: "E_REGISTRY_SCHEMA", detail: "unexpected registry schema" });
  if (registry.registryVersion !== 1) errors.push({ code: "E_REGISTRY_VERSION", detail: "registryVersion must be 1" });
  if (entries.length !== 35) errors.push({ code: "E_ENTRY_COUNT", detail: `expected 35 registry entries, got ${entries.length}` });
  if (!unique(entries.map((entry) => entry.assetId))) errors.push({ code: "E_DUPLICATE_ID", detail: "registry asset IDs must be unique" });
  if (!unique(entries.filter((entry) => entry.path !== null).map((entry) => entry.path))) errors.push({ code: "E_DUPLICATE_PATH", detail: "registry physical paths must be unique" });

  for (const entry of entries) {
    if (!STATES.has(entry.policyState)) errors.push({ code: "E_POLICY_STATE", id: entry.assetId, detail: entry.policyState });
    registryById.set(entry.assetId, entry);
    const expected = expectedById.get(entry.assetId);
    if (!expected) {
      errors.push({ code: "E_AG13_ID_MISMATCH", id: entry.assetId, detail: "registry ID absent from AG-13 manifest" });
      continue;
    }
    for (const field of ["path", "sha256", "sizeBytes", "required", "preload", "fallbackAssetId", "devOnly"]) {
      if (JSON.stringify(entry[field]) !== JSON.stringify(expected[field])) errors.push({ code: "E_AG13_FIELD_MISMATCH", id: entry.assetId, field, detail: `registry ${field} differs from AG-13` });
    }
    if (entry.path !== null) {
      const file = absRepo(entry.path);
      if (!fs.existsSync(file)) errors.push({ code: "E_MISSING_FILE", id: entry.assetId, path: entry.path, detail: "registry physical file is missing" });
      else {
        if (fs.statSync(file).size !== entry.sizeBytes) errors.push({ code: "E_SIZE_MISMATCH", id: entry.assetId, path: entry.path, detail: "physical size differs from registry" });
        if (sha256(file) !== entry.sha256) errors.push({ code: "E_HASH_MISMATCH", id: entry.assetId, path: entry.path, detail: "physical SHA-256 differs from registry" });
      }
    } else if (entry.sha256 !== null || entry.sizeBytes !== null) {
      errors.push({ code: "E_LOGICAL_FILE_FIELDS", id: entry.assetId, detail: "logical entry must have null hash and size" });
    }
    if (entry.required && entry.fallbackAssetId === null) {
      requiredUnresolved.push(entry);
      if (strictRequiredFallback) errors.push({ code: "E_REQUIRED_FALLBACK", id: entry.assetId, detail: "required registry entry has no fallback" });
    }
  }

  if (authEntries.length !== 35) errors.push({ code: "E_AUTH_COUNT", detail: `expected 35 authorization entries, got ${authEntries.length}` });
  if (!unique(authEntries.map((entry) => entry.assetId))) errors.push({ code: "E_AUTH_DUPLICATE_ID", detail: "authorization asset IDs must be unique" });
  for (const item of authEntries) {
    authById.set(item.assetId, item);
    if (!registryById.has(item.assetId)) errors.push({ code: "E_AUTH_ORPHAN", id: item.assetId, detail: "authorization item is absent from registry" });
    for (const field of ["authorizationEvidenceId", "evidenceIds", "evidencePaths", "reviewedAt", "authorizationStatus", "authorizationComplete"]) {
      if (!(field in item)) errors.push({ code: "E_AUTH_SCHEMA", id: item.assetId, detail: `missing authorization field ${field}` });
    }
    if (!Array.isArray(item.evidenceIds) || item.evidenceIds.length === 0) errors.push({ code: "E_AUTH_EVIDENCE_IDS", id: item.assetId, detail: "evidenceIds must be non-empty" });
    const registryEntry = registryById.get(item.assetId);
    if (registryEntry && item.authorizationEvidenceId !== registryEntry.authorizationEvidenceId) errors.push({ code: "E_AUTH_ID_MISMATCH", id: item.assetId, detail: "authorization evidence ID does not match registry" });
  }
  for (const entry of entries) if (!authById.has(entry.assetId)) errors.push({ code: "E_AUTH_MISSING", id: entry.assetId, detail: "registry item lacks authorization record" });

  for (const publicEntry of publicEntries) {
    const registryEntry = registryById.get(publicEntry.assetId);
    const authorizationEntry = authById.get(publicEntry.assetId);
    if (!registryEntry) errors.push({ code: "E_PUBLIC_UNKNOWN_ID", id: publicEntry.assetId, detail: "public manifest entry is absent from registry" });
    else {
      if (!VERIFIED.has(registryEntry.policyState)) errors.push({ code: "E_PUBLIC_POLICY_STATE", id: publicEntry.assetId, detail: "pending/local/rejected entry leaked into public manifest" });
      if (registryEntry.devOnly) errors.push({ code: "E_PUBLIC_DEV_ONLY", id: publicEntry.assetId, detail: "devOnly entry leaked into public manifest" });
      if (!authorizationEntry?.authorizationComplete) errors.push({ code: "E_PUBLIC_AUTHORIZATION", id: publicEntry.assetId, detail: "public entry lacks complete authorization" });
    }
  }
  if (publicManifest.officialPublishEligible !== false) errors.push({ code: "E_PUBLIC_ELIGIBILITY", detail: "public manifest must remain ineligible for this package" });
  return { errors, requiredUnresolved, registryById, authById };
}

function runNegative(name, mutate, expectedCode, baseline) {
  const model = {
    registry: clone(baseline.registry),
    authorization: clone(baseline.authorization),
    publicManifest: clone(baseline.publicManifest),
    ag13Manifest: baseline.ag13Manifest
  };
  mutate(model);
  const strictRequiredFallback = model.__strictRequiredFallback === true;
  delete model.__strictRequiredFallback;
  const result = validateModel({ ...model, strictRequiredFallback });
  return { name, pass: result.errors.some((error) => error.code === expectedCode), expectedCode, errorCodes: [...new Set(result.errors.map((error) => error.code))], errorCount: result.errors.length };
}

function main() {
  const registry = readJson(REGISTRY_PATH);
  const authorization = readJson(AUTH_PATH);
  const publicManifest = readJson(PUBLIC_PATH);
  const ag13Manifest = readJson(AG13_MANIFEST_PATH);
  const ag41 = readJson(AG41_RESULT_PATH);
  const baseline = { registry, authorization, publicManifest, ag13Manifest };
  const baseValidation = validateModel(baseline);
  const registryEntries = registry.entries;
  const physicalFiles = registryEntries.filter((entry) => entry.path !== null);
  const statusCounts = registryEntries.reduce((acc, entry) => { acc[entry.policyState] = (acc[entry.policyState] ?? 0) + 1; return acc; }, {});
  const negativeTests = [
    runNegative("illegal_policy_state", (model) => { model.registry.entries[0].policyState = "publish_candidate"; }, "E_POLICY_STATE", baseline),
    runNegative("duplicate_id", (model) => { model.registry.entries.push(clone(model.registry.entries[0])); }, "E_DUPLICATE_ID", baseline),
    runNegative("duplicate_path", (model) => { const copy = clone(model.registry.entries[0]); copy.assetId = "wuxian.image.shared.duplicate.path"; model.registry.entries.push(copy); }, "E_DUPLICATE_PATH", baseline),
    runNegative("hash_drift", (model) => { model.registry.entries[0].sha256 = "0".repeat(64); }, "E_HASH_MISMATCH", baseline),
    runNegative("verified_without_evidence", (model) => { model.registry.entries[0].policyState = "verified_original"; model.authorization.entries[0].evidenceIds = []; model.authorization.entries[0].authorizationComplete = true; model.publicManifest.entries = [clone(model.registry.entries[0])]; }, "E_AUTH_EVIDENCE_IDS", baseline),
    runNegative("pending_public_leak", (model) => { model.publicManifest.entries = [clone(model.registry.entries[0])]; }, "E_PUBLIC_POLICY_STATE", baseline),
    runNegative("dev_only_public_leak", (model) => { const item = model.registry.entries.find((entry) => entry.devOnly); item.policyState = "verified_redistributable"; model.authorization.entries.find((entry) => entry.assetId === item.assetId).authorizationComplete = true; model.publicManifest.entries = [clone(item)]; }, "E_PUBLIC_DEV_ONLY", baseline),
    runNegative("required_without_fallback", (model) => { model.__strictRequiredFallback = true; }, "E_REQUIRED_FALLBACK", baseline),
    runNegative("missing_physical_file", (model) => { model.registry.entries[0].path = "server-rs/release-ui/assets/ag41-missing-tp08a1.png"; }, "E_MISSING_FILE", baseline)
  ];
  const positiveChecks = {
    baselineRegistryValid: baseValidation.errors.length === 0,
    ag13Ag41Scope35Entries20Files: registryEntries.length === 35 && physicalFiles.length === 20 && ag41.coverage?.manifestEntries === 35 && ag41.coverage?.physicalFiles === 20,
    registryMatchesAg13IdentityFields: !baseValidation.errors.some((error) => ["E_AG13_ID_MISMATCH", "E_AG13_FIELD_MISMATCH", "E_HASH_MISMATCH", "E_SIZE_MISMATCH"].includes(error.code)),
    authorizationCompleteCoverage: authorization.entries.length === 35 && authorization.entries.every((entry) => entry.evidenceIds.length > 0 && entry.reviewedAt && entry.authorizationStatus),
    publicManifestEmpty: publicManifest.entries.length === 0,
    requiredFallbackBlockRecorded: baseValidation.requiredUnresolved.some((entry) => entry.assetId === "wuxian.vendor.shared.babylonjs.current"),
    negativeTestsAllRejected: negativeTests.every((test) => test.pass)
  };
  const result = {
    schema: "tp08_asset_registry_tp08a1_result_v1",
    status: Object.values(positiveChecks).every(Boolean) ? "isolated_tp08a1_contract_passed_with_publish_block" : "failed",
    pass: Object.values(positiveChecks).every(Boolean),
    machinePass: Object.values(positiveChecks).every(Boolean),
    scope: "isolated TP-08A1 registry/authorization/public-manifest contract over AG-13's exact 35 entries and 20 physical files",
    realAssetsModified: false,
    source: {
      ag13Manifest: "tools/continuous-tps-architecture-probe/assets/manifest-v1.json",
      ag13Result: "tools/continuous-tps-architecture-probe/results/assets-ag13.json",
      ag41Result: "tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json"
    },
    coverage: { registryEntries: registryEntries.length, physicalFiles: physicalFiles.length, ag13Entries: ag13Manifest.entries.length, ag41Entries: ag41.coverage?.manifestEntries, ag41PhysicalFiles: ag41.coverage?.physicalFiles },
    policy: { states: [...STATES], statusCounts, verifiedCount: registryEntries.filter((entry) => VERIFIED.has(entry.policyState)).length, publicManifestCount: publicManifest.entries.length },
    requiredFallback: { unresolved: baseValidation.requiredUnresolved.map((entry) => ({ assetId: entry.assetId, path: entry.path, fallbackAssetId: entry.fallbackAssetId, state: entry.policyState })) },
    officialPublishEligible: false,
    publicManifest: { path: "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json", entries: publicManifest.entries, officialPublishEligible: publicManifest.officialPublishEligible },
    checks: positiveChecks,
    negativeTests,
    futureFormalTp08ResolverWhitelist: [
      "tools/continuous-tps-architecture-probe/tp08-resolver/registry-resolver-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-resolver/cache-policy-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-resolver/refcount-dispose-contract-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-resolver/validator.mjs",
      "docs/CONTINUOUS_TP08_RESOLVER_CACHE_REVIEW.md"
    ],
    rollback: {
      newFilesOnly: true,
      trigger: "Any resolver/cache/refcount validation failure or unauthorized public-manifest promotion",
      action: "Remove only the future TP-08 resolver whitelist files, restore public manifest to the empty v1 contract, and leave AG-13/AG-41/current assets untouched",
      forbidden: ["modify or delete existing assets", "rewrite AG-13/AG-41 evidence", "publish pending/devOnly/rejected entries"]
    },
    formalDecision: "TP-08A1 proves only an isolated registry/policy contract. No authorization is created. Gate G and the overall Architecture Gate remain blocked/no_go."
  };
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ status: result.status, pass: result.pass, officialPublishEligible: result.officialPublishEligible, registryEntries: registryEntries.length, physicalFiles: physicalFiles.length, publicManifest: publicManifest.entries.length, statusCounts, negativeTestsPassed: negativeTests.filter((test) => test.pass).length }));
  if (!result.pass) process.exitCode = 1;
}

main();
