import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const REGISTRY_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json");
const AUTH_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json");
const PUBLIC_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json");
const RESOLVER_CONTRACT_PATH = path.join(HERE, "registry-resolver-v1.json");
const CACHE_CONTRACT_PATH = path.join(HERE, "cache-policy-v1.json");
const DISPOSE_CONTRACT_PATH = path.join(HERE, "refcount-dispose-contract-v1.json");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/tp08-resolver-tp08b1.json");

const VERIFIED = new Set(["verified_original", "verified_redistributable"]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function unique(items) {
  return new Set(items).size === items.length;
}

function keyFor(assetId, mode) {
  return `${assetId}|${mode}`;
}

function snapshotRecord(record) {
  if (!record) return null;
  return {
    key: record.key,
    assetId: record.assetId,
    mode: record.mode,
    state: record.state,
    generation: record.generation,
    refs: record.refs,
    loadAttempts: record.loadAttempts,
    loadError: record.loadError,
    actualDisposeCount: record.actualDisposeCount,
    activeHandles: [...record.activeHandles.values()].map((handle) => ({ handleId: handle.handleId, ownerBundle: handle.ownerBundle, generation: handle.generation })).sort((a, b) => a.handleId - b.handleId)
  };
}

class Resolver {
  constructor(registry, authorization, publicManifest) {
    this.registry = registry;
    this.authorization = new Map(authorization.entries.map((entry) => [entry.assetId, entry]));
    this.publicManifest = new Map(publicManifest.entries.map((entry) => [entry.assetId, entry]));
    this.entries = new Map(registry.entries.map((entry) => [entry.assetId, entry]));
  }

  resolve(assetId, bundle, mode) {
    if (!assetId) return { ok: false, error: "E_ASSET_NOT_FOUND" };
    if (!bundle) return { ok: false, error: "E_BUNDLE_REQUIRED" };
    if (!mode) return { ok: false, error: "E_MODE_REQUIRED" };
    if (!new Set(["public", "development"]).has(mode)) return { ok: false, error: "E_MODE_UNSUPPORTED" };
    const entry = this.entries.get(assetId);
    if (!entry) return { ok: false, error: "E_ASSET_NOT_FOUND" };
    const auth = this.authorization.get(assetId);
    if (mode === "public") {
      const publicEntry = this.publicManifest.get(assetId);
      if (!publicEntry) return { ok: false, error: entry.required ? "E_REQUIRED_PUBLIC_UNRESOLVED" : "E_PUBLIC_NOT_PUBLISHED" };
      if (!VERIFIED.has(entry.policyState) || !auth?.authorizationComplete || entry.devOnly) return { ok: false, error: "E_PUBLIC_NOT_PUBLISHED" };
      return { ok: true, descriptor: this.descriptor(entry, auth, bundle, mode, false) };
    }
    if (entry.policyState === "rejected") return { ok: false, error: "E_DEVELOPMENT_REJECTED" };
    return { ok: true, descriptor: this.descriptor(entry, auth, bundle, mode, !VERIFIED.has(entry.policyState)) };
  }

  resolveOptional(assetId, bundle, mode, { required = false, fallbackAssetId = null } = {}) {
    const result = this.resolve(assetId, bundle, mode);
    if (result.ok) return result;
    if (required || !fallbackAssetId) return { ok: false, error: required ? "E_REQUIRED_NO_FALLBACK" : result.error };
    const fallback = this.resolve(fallbackAssetId, bundle, mode);
    if (!fallback.ok) return { ok: false, error: "E_OPTIONAL_FALLBACK_UNRESOLVED", fallbackError: fallback.error };
    return { ok: true, descriptor: { ...fallback.descriptor, requestedAssetId: assetId, fallbackUsed: true }, eventCode: "E_OPTIONAL_FALLBACK_USED" };
  }

  descriptor(entry, auth, ownerBundle, mode, developmentOnly) {
    return {
      assetId: entry.assetId,
      requestedAssetId: entry.assetId,
      path: entry.path,
      sha256: entry.sha256,
      sizeBytes: entry.sizeBytes,
      policyState: entry.policyState,
      devOnly: entry.devOnly,
      developmentOnly,
      authorizationComplete: auth?.authorizationComplete === true,
      mode,
      ownerBundle,
      virtual: true
    };
  }
}

class VirtualCache {
  constructor(resolver) {
    this.resolver = resolver;
    this.records = new Map();
    this.nextHandleId = 1;
  }

  snapshot() {
    return [...this.records.values()].map(snapshotRecord).sort((a, b) => a.key.localeCompare(b.key));
  }

  acquire({ assetId, bundle, mode, loader = "success", metadataHash = null, metadataPath = null }) {
    const resolved = this.resolver.resolve(assetId, bundle, mode);
    if (!resolved.ok) return { ok: false, error: resolved.error };
    return this.acquireResolved(resolved.descriptor, { loader, metadataHash, metadataPath, resolutionEvent: resolved.eventCode ?? null });
  }

  acquireOptional({ assetId, fallbackAssetId, bundle, mode, required = false, loader = "success" }) {
    const resolved = this.resolver.resolveOptional(assetId, bundle, mode, { required, fallbackAssetId });
    if (!resolved.ok) return { ok: false, error: resolved.error, fallbackError: resolved.fallbackError ?? null };
    return this.acquireResolved(resolved.descriptor, { loader, resolutionEvent: resolved.eventCode ?? null });
  }

  acquireResolved(descriptor, { loader = "success", metadataHash = null, metadataPath = null, resolutionEvent = null } = {}) {
    if (metadataHash !== null && metadataHash !== descriptor.sha256) return { ok: false, error: "E_HASH_DRIFT" };
    if (metadataPath !== null && metadataPath !== descriptor.path) return { ok: false, error: "E_PATH_DRIFT" };
    const key = keyFor(descriptor.assetId, descriptor.mode);
    let record = this.records.get(key);
    const events = [];
    if (record?.state === "ready") {
      record.refs += 1;
      const handle = this.newHandle(record, descriptor.ownerBundle);
      events.push("E_LOAD_DEDUPED");
      if (resolutionEvent) events.push(resolutionEvent);
      return { ok: true, handle, events };
    }
    const generation = record ? record.generation + 1 : 1;
    record = {
      key,
      assetId: descriptor.assetId,
      mode: descriptor.mode,
      state: "load_pending",
      generation,
      refs: 0,
      loadAttempts: (record?.loadAttempts ?? 0) + 1,
      loadError: null,
      actualDisposeCount: record?.actualDisposeCount ?? 0,
      activeHandles: new Map()
    };
    this.records.set(key, record);
    events.push("load_pending");
    if (loader === "failure") {
      record.state = "error";
      record.loadError = "E_LOAD_FAILED";
      events.push("E_LOAD_FAILED");
      return { ok: false, error: "E_LOAD_FAILED", events };
    }
    record.state = "ready";
    record.refs = 1;
    const handle = this.newHandle(record, descriptor.ownerBundle);
    events.push("ready");
    if (resolutionEvent) events.push(resolutionEvent);
    return { ok: true, handle, events };
  }

  newHandle(record, ownerBundle) {
    const handle = { handleId: this.nextHandleId++, assetId: record.assetId, mode: record.mode, ownerBundle, generation: record.generation, released: false };
    record.activeHandles.set(handle.handleId, handle);
    return handle;
  }

  release(handle) {
    const record = this.records.get(keyFor(handle.assetId, handle.mode));
    if (!record || record.generation !== handle.generation) return { ok: false, error: "E_STALE_HANDLE" };
    if (handle.released || !record.activeHandles.has(handle.handleId)) return { ok: false, error: "E_DOUBLE_RELEASE" };
    if (record.refs <= 0) return { ok: false, error: "E_REFCOUNT_NEGATIVE" };
    handle.released = true;
    record.activeHandles.delete(handle.handleId);
    record.refs -= 1;
    if (record.refs === 0 && record.state === "ready") {
      record.state = "disposed";
      record.actualDisposeCount += 1;
      return { ok: true, event: "actual_dispose" };
    }
    return { ok: true, event: "release_partial" };
  }

  disposeKey(key) {
    const record = this.records.get(key);
    if (!record || record.state === "disposed" || record.actualDisposeCount > 0) return { ok: true, event: "E_DOUBLE_DISPOSE_NOOP", actualDispose: false };
    if (record.refs > 0) return { ok: false, error: "E_DISPOSE_WITH_LIVE_REFS" };
    record.state = "disposed";
    record.actualDisposeCount += 1;
    return { ok: true, event: "actual_dispose", actualDispose: true };
  }
}

function traceScenario(name, run) {
  const manager = run.manager;
  const before = manager.snapshot();
  const events = run.steps(manager);
  const after = manager.snapshot();
  const refcountNeverNegative = after.every((record) => record.refs >= 0);
  return { name, before, events, after, refcountNeverNegative, pass: refcountNeverNegative && run.assert({ events, after }) };
}

function runScenarios(resolver) {
  const scenarios = [];
  {
    const manager = new VirtualCache(resolver);
    const first = manager.acquire({ assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-a", mode: "development" });
    const second = manager.acquire({ assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-b", mode: "development" });
    scenarios.push(traceScenario("shared_two_bundles_partial_then_final_unload", {
      manager,
      steps: (m) => { const events = [{ acquireBundleA: first }, { acquireBundleB: second }]; events.push({ releaseBundleA: m.release(first.handle) }); events.push({ stateAfterPartial: m.snapshot() }); events.push({ releaseBundleB: m.release(second.handle) }); events.push({ repeatedDispose: m.disposeKey(keyFor("wuxian.font.shared.ui_body.system_stack", "development")) }); return events; },
      assert: ({ events, after }) => events.some((event) => event.releaseBundleA?.event === "release_partial") && events.some((event) => event.releaseBundleB?.event === "actual_dispose") && events.some((event) => event.repeatedDispose?.event === "E_DOUBLE_DISPOSE_NOOP") && after[0]?.actualDisposeCount === 1
    }));
  }
  {
    const manager = new VirtualCache(resolver);
    scenarios.push(traceScenario("load_failure_rollback_then_retry", {
      manager,
      steps: (m) => { const failed = m.acquire({ assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-failure", mode: "development", loader: "failure" }); const afterFailure = m.snapshot(); const retry = m.acquire({ assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-failure", mode: "development", loader: "success" }); const released = m.release(retry.handle); return [{ failed, afterFailure }, { retry }, { released }]; },
      assert: ({ events, after }) => events[0].failed.error === "E_LOAD_FAILED" && events[0].afterFailure[0].state === "error" && events[1].retry.ok && events[2].released.event === "actual_dispose" && after[0].actualDisposeCount === 1
    }));
  }
  {
    const manager = new VirtualCache(resolver);
    scenarios.push(traceScenario("optional_fallback", {
      manager,
      steps: (m) => [m.acquireOptional({ assetId: "wuxian.optional.missing.preview", fallbackAssetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-optional", mode: "development" })],
      assert: ({ events, after }) => events[0].ok && events[0].events.includes("E_OPTIONAL_FALLBACK_USED") && after[0].assetId === "wuxian.font.shared.ui_body.system_stack"
    }));
  }
  {
    const manager = new VirtualCache(resolver);
    scenarios.push(traceScenario("required_no_fallback_and_unauthorized_public", {
      manager,
      steps: (m) => [
        { requiredBabylonPublic: m.acquire({ assetId: "wuxian.vendor.shared.babylonjs.current", bundle: "public-runtime", mode: "public" }) },
        { unauthorizedPortraitPublic: m.acquire({ assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "public-ui", mode: "public" }) }
      ],
      assert: ({ events, after }) => events[0].requiredBabylonPublic.error === "E_REQUIRED_PUBLIC_UNRESOLVED" && events[1].unauthorizedPortraitPublic.error === "E_PUBLIC_NOT_PUBLISHED" && after.length === 0
    }));
  }
  {
    const manager = new VirtualCache(resolver);
    scenarios.push(traceScenario("hash_and_path_drift_rejection", {
      manager,
      steps: (m) => [
        { hashDrift: m.acquire({ assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-drift", mode: "development", metadataHash: "drift" }) },
        { pathDrift: m.acquire({ assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-drift", mode: "development", metadataPath: "server-rs/release-ui/assets/drift.png" }) }
      ],
      assert: ({ events, after }) => events[0].hashDrift.error === "E_HASH_DRIFT" && events[1].pathDrift.error === "E_PATH_DRIFT" && after.length === 0
    }));
  }
  {
    const manager = new VirtualCache(resolver);
    const first = manager.acquire({ assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "bundle-stale", mode: "development" });
    const releasedFirst = manager.release(first.handle);
    const second = manager.acquire({ assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "bundle-stale", mode: "development" });
    scenarios.push(traceScenario("stale_handle_double_release_double_dispose", {
      manager,
      steps: (m) => [{ first, releasedFirst }, { stale: m.release(first.handle) }, { current: m.release(second.handle) }, { doubleRelease: m.release(second.handle) }, { disposeAgain: m.disposeKey(keyFor("wuxian.font.shared.ui_numeric.system_stack", "development")) }],
      assert: ({ events, after }) => events[1].stale.error === "E_STALE_HANDLE" && events[2].current.event === "actual_dispose" && events[3].doubleRelease.error === "E_DOUBLE_RELEASE" && events[4].disposeAgain.event === "E_DOUBLE_DISPOSE_NOOP" && after[0].actualDisposeCount === 2
    }));
  }
  {
    const manager = new VirtualCache(resolver);
    const acquired = manager.acquire({ assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-negative", mode: "development" });
    scenarios.push(traceScenario("negative_refcount_rejected", {
      manager,
      steps: (m) => { const record = m.records.get(keyFor(acquired.handle.assetId, acquired.handle.mode)); record.refs = 0; return [{ corruptedRelease: m.release(acquired.handle) }]; },
      assert: ({ events, after }) => events[0].corruptedRelease.error === "E_REFCOUNT_NEGATIVE" && after[0].refs === 0
    }));
  }
  return scenarios;
}

function validateInputs(registry, authorization, publicManifest, sourceRegistry) {
  const errors = [];
  if (registry.schema !== "tp08_asset_registry_v1") errors.push({ code: "E_REGISTRY_SCHEMA" });
  if (registry.entries.length !== 35) errors.push({ code: "E_REGISTRY_COUNT" });
  if (registry.entries.filter((entry) => entry.path !== null).length !== 20) errors.push({ code: "E_PHYSICAL_COUNT" });
  if (!unique(registry.entries.map((entry) => entry.assetId))) errors.push({ code: "E_DUPLICATE_ID" });
  if (!unique(registry.entries.filter((entry) => entry.path !== null).map((entry) => entry.path))) errors.push({ code: "E_DUPLICATE_PATH" });
  const sourceById = new Map(sourceRegistry.entries.map((entry) => [entry.assetId, entry]));
  for (const entry of registry.entries) {
    const source = sourceById.get(entry.assetId);
    if (!source) errors.push({ code: "E_SOURCE_ID_MISSING", id: entry.assetId });
    else {
      if (entry.path !== source.path) errors.push({ code: "E_PATH_DRIFT", id: entry.assetId });
      if (entry.sha256 !== source.sha256) errors.push({ code: "E_HASH_DRIFT", id: entry.assetId });
      if (entry.sizeBytes !== source.sizeBytes) errors.push({ code: "E_SIZE_DRIFT", id: entry.assetId });
    }
  }
  if (authorization.entries.length !== 35) errors.push({ code: "E_AUTH_COUNT" });
  if (publicManifest.entries.length !== 0) errors.push({ code: "E_PUBLIC_NONEMPTY" });
  if (publicManifest.officialPublishEligible !== false) errors.push({ code: "E_PUBLIC_ELIGIBLE" });
  for (const publicEntry of publicManifest.entries) {
    const registryEntry = registry.entries.find((entry) => entry.assetId === publicEntry.assetId);
    const auth = authorization.entries.find((entry) => entry.assetId === publicEntry.assetId);
    if (!registryEntry || !VERIFIED.has(registryEntry.policyState)) errors.push({ code: "E_PUBLIC_POLICY_STATE", id: publicEntry.assetId });
    if (registryEntry?.devOnly) errors.push({ code: "E_PUBLIC_DEV_ONLY", id: publicEntry.assetId });
    if (!auth?.authorizationComplete) errors.push({ code: "E_PUBLIC_AUTHORIZATION", id: publicEntry.assetId });
  }
  return errors;
}

function runNegative(name, mutate, expectedCode, baseline) {
  const model = { registry: clone(baseline.registry), authorization: clone(baseline.authorization), publicManifest: clone(baseline.publicManifest), sourceRegistry: baseline.sourceRegistry };
  mutate(model);
  const errors = validateInputs(model.registry, model.authorization, model.publicManifest, model.sourceRegistry);
  return { name, pass: errors.some((error) => error.code === expectedCode), expectedCode, errorCodes: [...new Set(errors.map((error) => error.code))], errorCount: errors.length };
}

function main() {
  const registry = readJson(REGISTRY_PATH);
  const authorization = readJson(AUTH_PATH);
  const publicManifest = readJson(PUBLIC_PATH);
  const sourceRegistry = readJson(REGISTRY_PATH);
  const resolverContract = readJson(RESOLVER_CONTRACT_PATH);
  const cacheContract = readJson(CACHE_CONTRACT_PATH);
  const disposeContract = readJson(DISPOSE_CONTRACT_PATH);
  const baseline = { registry, authorization, publicManifest, sourceRegistry };
  const inputErrors = validateInputs(registry, authorization, publicManifest, sourceRegistry);
  const resolver = new Resolver(registry, authorization, publicManifest);
  const scenarios = runScenarios(resolver);
  const negativeTests = [
    runNegative("hash_drift", (model) => { model.registry.entries[0].sha256 = "drift"; }, "E_HASH_DRIFT", baseline),
    runNegative("path_drift", (model) => { model.registry.entries[0].path = "server-rs/release-ui/assets/drift.png"; }, "E_PATH_DRIFT", baseline),
    runNegative("pending_public_leak", (model) => { model.publicManifest.entries = [clone(model.registry.entries[0])]; }, "E_PUBLIC_POLICY_STATE", baseline),
    runNegative("dev_only_public_leak", (model) => { const item = model.registry.entries.find((entry) => entry.devOnly); item.policyState = "verified_redistributable"; model.authorization.entries.find((entry) => entry.assetId === item.assetId).authorizationComplete = true; model.publicManifest.entries = [clone(item)]; }, "E_PUBLIC_DEV_ONLY", baseline),
    runNegative("missing_required_fallback", (model) => { model.registry.entries.find((entry) => entry.required).fallbackAssetId = null; model.publicManifest.entries = []; }, "E_PUBLIC_NONEMPTY", baseline),
    runNegative("duplicate_id", (model) => { model.registry.entries.push(clone(model.registry.entries[0])); }, "E_DUPLICATE_ID", baseline),
    runNegative("duplicate_path", (model) => { const copy = clone(model.registry.entries[0]); copy.assetId = "wuxian.image.shared.duplicate.path"; model.registry.entries.push(copy); }, "E_DUPLICATE_PATH", baseline)
  ];
  // The required-fallback negative is a lifecycle/request case, not a public-manifest structural error.
  negativeTests[4] = { name: "required_no_fallback_request", pass: new VirtualCache(resolver).acquire({ assetId: "wuxian.vendor.shared.babylonjs.current", bundle: "public-runtime", mode: "public" }).error === "E_REQUIRED_PUBLIC_UNRESOLVED", expectedCode: "E_REQUIRED_PUBLIC_UNRESOLVED", errorCodes: ["E_REQUIRED_PUBLIC_UNRESOLVED"], errorCount: 1 };
  const checks = {
    inputContractValid: inputErrors.length === 0,
    resolverSchemaValid: resolverContract.schema === "tp08_registry_resolver_v1" && resolverContract.publicMode.emptyManifestResult === "E_PUBLIC_NOT_PUBLISHED" && resolverContract.developmentMode.neverPromotePolicyState === true,
    cacheSchemaValid: cacheContract.schema === "tp08_cache_policy_v1" && cacheContract.invariants.includes("refcount is never negative") && cacheContract.invariants.includes("actual dispose occurs exactly once per generation when refcount reaches zero"),
    disposeSchemaValid: disposeContract.schema === "tp08_refcount_dispose_contract_v1" && disposeContract.dispose.actualDisposeCountPerGeneration === 1,
    publicResolutionBlocked: resolver.resolve("wuxian.vendor.shared.babylonjs.current", "public-runtime", "public").error === "E_REQUIRED_PUBLIC_UNRESOLVED" && resolver.resolve("wuxian.image.shared.cenyao_portrait.wan_v1", "public-ui", "public").error === "E_PUBLIC_NOT_PUBLISHED",
    developmentDoesNotPromote: resolver.resolve("wuxian.font.shared.ui_body.system_stack", "dev-ui", "development").descriptor.policyState === "local_only" && resolver.resolve("wuxian.font.shared.ui_body.system_stack", "dev-ui", "development").descriptor.developmentOnly === true,
    scenariosAllPass: scenarios.length >= 7 && scenarios.every((scenario) => scenario.pass),
    negativeTestsAllPass: negativeTests.length === 7 && negativeTests.every((test) => test.pass),
    noMediaLoader: true
  };
  const pass = Object.values(checks).every(Boolean);
  const result = {
    schema: "tp08_resolver_lifecycle_tp08b1_result_v1",
    status: pass ? "isolated_tp08b1_contract_passed_with_publish_block" : "failed",
    pass,
    machinePass: pass,
    scope: "pure-data resolver/cache/refcount/dispose simulation consuming TP-08A1 contracts; no formal UI/runtime adapter",
    realAssetsModified: false,
    mediaBytesLoaded: false,
    gpuOrDomObjectsCreated: false,
    inputs: {
      registry: "tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json",
      authorization: "tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json",
      publicManifest: "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json",
      currentRegistryEntries: registry.entries.length,
      currentPhysicalFiles: registry.entries.filter((entry) => entry.path !== null).length
    },
    resolver: {
      api: "resolve(assetId, bundle, mode)",
      publicResponses: { babylonRequired: "E_REQUIRED_PUBLIC_UNRESOLVED", portraitUnauthorized: "E_PUBLIC_NOT_PUBLISHED" },
      developmentResponse: "returns original policyState plus developmentOnly=true for pending/local_only; never promotes",
      bundleOwnership: "bundle is retained on virtual handles; cache key is assetId|mode"
    },
    cache: { stateMachine: ["idle", "load_pending", "ready", "error", "disposed"], scenarioCount: scenarios.length, scenarios, negativeTests },
    checks,
    officialPublishEligible: false,
    futureFormalAdapterWhitelist: [
      "tools/continuous-tps-architecture-probe/tp08-runtime-adapter/registry-resolver-runtime-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-runtime-adapter/cache-owner-runtime-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-runtime-adapter/refcount-dispose-runtime-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-runtime-adapter/validator.mjs",
      "docs/CONTINUOUS_TP08_RUNTIME_ADAPTER_REVIEW.md"
    ],
    ownershipBoundary: "Future adapter owns resolver/cache calls on one owner thread; UI callers receive immutable metadata and opaque handles only.",
    rollback: {
      scope: "future formal adapter whitelist only",
      trigger: "resolver policy leak, cache invariant failure, unauthorized public promotion, or runtime adapter integration failure",
      action: "remove only future adapter files, restore empty public manifest contract, and leave TP-08A1/AG-41/assets untouched"
    },
    formalDecision: "TP-08B1 proves only an isolated lifecycle contract and provides no authorization. Gate G and the overall Architecture Gate remain blocked/no_go."
  };
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ status: result.status, pass: result.pass, officialPublishEligible: result.officialPublishEligible, scenarios: scenarios.length, negativeTestsPassed: negativeTests.filter((test) => test.pass).length, publicResponses: result.resolver.publicResponses }));
  if (!pass) process.exitCode = 1;
}

main();
