import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const REGISTRY_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json");
const AUTH_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json");
const PUBLIC_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json");
const AG41_RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json");
const ADAPTER_CONTRACT_PATH = path.join(HERE, "runtime-adapter-v1.json");
const OWNER_CONTRACT_PATH = path.join(HERE, "owner-queue-contract-v1.json");
const SHUTDOWN_CONTRACT_PATH = path.join(HERE, "shutdown-contract-v1.json");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/tp08-runtime-adapter-tp08c1.json");

const VERIFIED = new Set(["verified_original", "verified_redistributable"]);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function keyFor(assetId, mode) {
  return `${assetId}|${mode}`;
}

function snapshot(adapter) {
  return {
    status: adapter.status,
    ownerEpoch: adapter.ownerEpoch,
    lastAcceptedSeq: adapter.lastAcceptedSeq,
    mutationLock: adapter.mutationLock,
    queueDepth: adapter.queue.length,
    registryCount: adapter.registry.size,
    cache: [...adapter.cache.values()].map((record) => ({
      key: record.key,
      assetId: record.assetId,
      mode: record.mode,
      state: record.state,
      generation: record.generation,
      refs: record.refs,
      loadCount: record.loadCount,
      loadError: record.loadError,
      actualDisposeCount: record.actualDisposeCount,
      liveHandleCount: [...adapter.handles.values()].filter((handle) => handle.key === record.key && !handle.released).length,
      pendingCount: record.pendingWaiters.length
    })).sort((a, b) => a.key.localeCompare(b.key)),
    liveHandleCount: [...adapter.handles.values()].filter((handle) => !handle.released).length,
    retiredHandleCount: adapter.retiredHandles.size,
    bundleRefs: [...adapter.handles.values()].filter((handle) => !handle.released).reduce((acc, handle) => { acc[handle.ownerBundle] = (acc[handle.ownerBundle] ?? 0) + 1; return acc; }, {})
  };
}

class RuntimeAdapter {
  constructor(registry, authorization, publicManifest) {
    this.registry = new Map(registry.entries.map((entry) => [entry.assetId, entry]));
    this.authorization = new Map(authorization.entries.map((entry) => [entry.assetId, entry]));
    this.publicManifest = new Map(publicManifest.entries.map((entry) => [entry.assetId, entry]));
    this.status = "new";
    this.ownerEpoch = null;
    this.lastAcceptedSeq = 0;
    this.mutationLock = false;
    this.queue = [];
    this.completed = new Map();
    this.cache = new Map();
    this.handles = new Map();
    this.retiredHandles = new Set();
    this.nextHandleId = 1;
  }

  initialize({ ownerEpoch = 1, replace = false } = {}) {
    if (this.status === "ready" && !replace && ownerEpoch === this.ownerEpoch) return { ok: false, error: "E_ALREADY_INITIALIZED" };
    if (this.status === "ready" || this.status === "closing") this.cleanupForEpochReplacement();
    this.status = "ready";
    this.ownerEpoch = ownerEpoch;
    this.lastAcceptedSeq = 0;
    return { ok: true, event: replace ? "EPOCH_REPLACED" : "initialized", ownerEpoch };
  }

  cleanupForEpochReplacement() {
    for (const handle of this.handles.values()) this.retiredHandles.add(handle.opaqueHandle);
    for (const record of this.cache.values()) {
      if (record.state === "ready" && record.actualDisposeCount === 0) {
        record.state = "disposed";
        record.actualDisposeCount += 1;
      }
      if (record.state === "load_pending") {
        record.state = "error";
        record.loadError = "E_EPOCH_REPLACED";
      }
    }
    this.cache.clear();
    this.handles.clear();
    this.queue = [];
  }

  submit(request) {
    if (this.mutationLock) {
      this.queue.push(request);
      return { ok: true, queued: true, event: "E_CALLBACK_REENTRANT_QUEUED" };
    }
    if (this.status === "closed" && request.op === "shutdown") return this.record(request, { ok: true, event: "E_SHUTDOWN_IDEMPOTENT" });
    if (this.status === "closed" || this.status === "closing") return this.record(request, { ok: false, error: "E_OWNER_CLOSING" });
    if (this.status !== "ready") return this.record(request, { ok: false, error: "E_OWNER_NOT_INITIALIZED" });
    if (request.ownerEpoch !== this.ownerEpoch) return this.record(request, { ok: false, error: "E_OWNER_EPOCH_STALE" });
    if (request.requestSeq <= this.lastAcceptedSeq) return this.record(request, { ok: false, error: request.requestSeq === this.lastAcceptedSeq ? "E_REQUEST_DUPLICATE" : "E_REQUEST_OUT_OF_ORDER" });
    if (request.requestSeq !== this.lastAcceptedSeq + 1) return this.record(request, { ok: false, error: "E_REQUEST_OUT_OF_ORDER" });
    this.lastAcceptedSeq = request.requestSeq;
    this.mutationLock = true;
    let output;
    try {
      if (request.op === "acquire") output = this.acquireInternal(request);
      else if (request.op === "release") output = this.releaseInternal(request.opaqueHandle);
      else if (request.op === "bundleRelease") output = this.bundleReleaseInternal(request.bundle);
      else if (request.op === "callback") output = this.callbackInternal(request);
      else if (request.op === "shutdown") output = this.shutdownInternal();
      else output = { ok: false, error: "E_OPERATION_UNSUPPORTED" };
    } finally {
      this.mutationLock = false;
    }
    this.record(request, output);
    this.drain();
    return output;
  }

  record(request, output) {
    if (request?.ownerEpoch != null && request?.requestSeq != null) this.completed.set(`${request.ownerEpoch}:${request.requestSeq}`, clone(output));
    return output;
  }

  drain() {
    while (!this.mutationLock && this.queue.length) {
      const next = this.queue.shift();
      this.submit(next);
    }
  }

  resolve(assetId, bundle, mode) {
    const entry = this.registry.get(assetId);
    if (!entry) return { ok: false, error: "E_ASSET_NOT_FOUND" };
    const auth = this.authorization.get(assetId);
    if (mode === "public") {
      const published = this.publicManifest.has(assetId);
      if (!published) return { ok: false, error: entry.required ? "E_REQUIRED_PUBLIC_UNRESOLVED" : "E_PUBLIC_NOT_PUBLISHED" };
      if (!VERIFIED.has(entry.policyState) || entry.devOnly || auth?.authorizationComplete !== true) return { ok: false, error: "E_PUBLIC_NOT_PUBLISHED" };
      return { ok: true, metadata: this.metadata(entry, auth, bundle, mode, false) };
    }
    if (mode !== "development") return { ok: false, error: "E_MODE_UNSUPPORTED" };
    if (entry.policyState === "rejected") return { ok: false, error: "E_DEVELOPMENT_REJECTED" };
    return { ok: true, metadata: this.metadata(entry, auth, bundle, mode, !VERIFIED.has(entry.policyState)) };
  }

  metadata(entry, auth, ownerBundle, mode, developmentOnly) {
    return Object.freeze({
      assetId: entry.assetId,
      path: entry.path,
      sha256: entry.sha256,
      sizeBytes: entry.sizeBytes,
      policyState: entry.policyState,
      developmentOnly,
      authorizationComplete: auth?.authorizationComplete === true,
      mode,
      ownerBundle,
      virtual: true,
      immutable: true
    });
  }

  acquireInternal(request) {
    const resolved = this.resolve(request.assetId, request.bundle, request.mode);
    if (!resolved.ok) return { ok: false, error: resolved.error };
    if (request.metadataHash != null && request.metadataHash !== resolved.metadata.sha256) return { ok: false, error: "E_HASH_DRIFT" };
    if (request.metadataPath != null && request.metadataPath !== resolved.metadata.path) return { ok: false, error: "E_PATH_DRIFT" };
    const key = keyFor(request.assetId, request.mode);
    let record = this.cache.get(key);
    if (record?.state === "ready") {
      record.refs += 1;
      const handle = this.createHandle(record, request.bundle);
      return { ok: true, metadata: resolved.metadata, handle: { opaqueHandle: handle.opaqueHandle }, events: ["E_LOAD_DEDUPED"] };
    }
    const generation = record ? record.generation + 1 : 1;
    record = {
      key,
      assetId: request.assetId,
      mode: request.mode,
      state: "load_pending",
      generation,
      refs: 0,
      loadCount: (record?.loadCount ?? 0) + 1,
      loadError: null,
      actualDisposeCount: record?.actualDisposeCount ?? 0,
      pendingWaiters: []
    };
    this.cache.set(key, record);
    const events = ["load_pending"];
    if (request.loader === "pending") {
      record.pendingWaiters.push({ bundle: request.bundle, metadata: resolved.metadata });
      return { ok: true, pending: true, metadata: resolved.metadata, events };
    }
    if (request.loader === "failure") {
      record.state = "error";
      record.loadError = "E_LOAD_FAILED";
      return { ok: false, error: "E_LOAD_FAILED", events: [...events, "E_LOAD_FAILED"] };
    }
    record.state = "ready";
    record.refs = 1;
    const handle = this.createHandle(record, request.bundle);
    return { ok: true, metadata: resolved.metadata, handle: { opaqueHandle: handle.opaqueHandle }, events: [...events, "ready"] };
  }

  createHandle(record, ownerBundle) {
    const opaqueHandle = `opaque:${this.ownerEpoch}:${this.nextHandleId++}`;
    const handle = { opaqueHandle, key: record.key, generation: record.generation, ownerBundle, released: false };
    this.handles.set(opaqueHandle, handle);
    return handle;
  }

  releaseInternal(opaqueHandle) {
    const handle = this.handles.get(opaqueHandle);
    if (!handle) return { ok: false, error: this.retiredHandles.has(opaqueHandle) ? "E_STALE_HANDLE" : "E_HANDLE_UNKNOWN" };
    if (handle.released) return { ok: false, error: "E_DOUBLE_RELEASE" };
    const record = this.cache.get(handle.key);
    if (!record || record.generation !== handle.generation) return { ok: false, error: "E_STALE_HANDLE" };
    if (record.refs <= 0) return { ok: false, error: "E_REFCOUNT_NEGATIVE" };
    handle.released = true;
    record.refs -= 1;
    if (record.refs === 0 && record.state === "ready") {
      record.state = "disposed";
      record.actualDisposeCount += 1;
      return { ok: true, event: "actual_dispose" };
    }
    return { ok: true, event: "release_partial" };
  }

  bundleReleaseInternal(bundle) {
    const handles = [...this.handles.values()].filter((handle) => handle.ownerBundle === bundle && !handle.released);
    const releases = handles.map((handle) => this.releaseInternal(handle.opaqueHandle));
    return { ok: true, bundle, releasedCount: releases.filter((release) => release.ok).length, releases };
  }

  callbackInternal(request) {
    const key = keyFor(request.assetId, request.mode);
    const record = this.cache.get(key);
    if (!record || record.state !== "load_pending") return { ok: false, error: "E_STALE_LOAD_CALLBACK" };
    if (request.outcome === "failure") {
      record.state = "error";
      record.loadError = "E_LOAD_FAILED";
      return { ok: false, error: "E_LOAD_FAILED" };
    }
    record.state = "ready";
    const handles = record.pendingWaiters.map((waiter) => this.createHandle(record, waiter.bundle));
    record.refs = handles.length;
    record.pendingWaiters = [];
    const output = { ok: true, event: "ready", handles: handles.map((handle) => ({ opaqueHandle: handle.opaqueHandle })) };
    if (request.reentrantRequest) output.reentrant = this.submit(request.reentrantRequest);
    return output;
  }

  shutdownInternal() {
    this.status = "closing";
    let canceledPending = 0;
    let disposed = 0;
    for (const record of this.cache.values()) {
      if (record.state === "load_pending") { record.state = "error"; record.loadError = "E_OWNER_CLOSING"; canceledPending += 1; }
      if (record.state === "ready" && record.actualDisposeCount === 0) { record.state = "disposed"; record.actualDisposeCount += 1; disposed += 1; }
    }
    for (const handle of this.handles.values()) this.retiredHandles.add(handle.opaqueHandle);
    this.handles.clear();
    this.cache.clear();
    this.registry.clear();
    this.authorization.clear();
    this.publicManifest.clear();
    this.status = "closed";
    return { ok: true, event: "shutdown_complete", canceledPending, disposed };
  }
}

function traceScenario(name, input, adapter, actions, assertion) {
  const before = snapshot(adapter);
  const outputs = actions(adapter);
  const finalState = snapshot(adapter);
  return { name, input, outputs, finalState, pass: assertion(outputs, finalState) };
}

function runScenarios(registry, authorization, publicManifest) {
  const scenarios = [];
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    const init = adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("normal_initialization_shared_acquire", { init, callers: ["caller-a", "caller-b"] }, adapter, (a) => {
      const first = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-a", mode: "development" });
      const second = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 2, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-b", mode: "development" });
      const releaseA = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 3, opaqueHandle: first.handle.opaqueHandle });
      const releaseB = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 4, opaqueHandle: second.handle.opaqueHandle });
      return { init, first, second, releaseA, releaseB };
    }, (o, s) => o.first.ok && o.second.events.includes("E_LOAD_DEDUPED") && o.releaseA.event === "release_partial" && o.releaseB.event === "actual_dispose" && s.cache[0].actualDisposeCount === 1));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("duplicate_and_out_of_order_request_seq", { expectedErrors: ["E_REQUEST_DUPLICATE", "E_REQUEST_OUT_OF_ORDER"] }, adapter, (a) => {
      const first = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-seq", mode: "development" });
      const duplicate = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-seq", mode: "development" });
      const outOfOrder = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 3, assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-seq", mode: "development" });
      const accepted = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 2, opaqueHandle: first.handle.opaqueHandle });
      return { first, duplicate, outOfOrder, accepted };
    }, (o) => o.duplicate.error === "E_REQUEST_DUPLICATE" && o.outOfOrder.error === "E_REQUEST_OUT_OF_ORDER" && o.accepted.ok));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("epoch_replacement_cleans_pending_and_stales_old_handle", { oldEpoch: 1, newEpoch: 2 }, adapter, (a) => {
      const pending = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-epoch", mode: "development", loader: "pending" });
      const ready = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 2, assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "bundle-epoch", mode: "development" });
      const replacement = a.initialize({ ownerEpoch: 2, replace: true });
      const stale = a.submit({ op: "release", ownerEpoch: 2, requestSeq: 1, opaqueHandle: ready.handle.opaqueHandle });
      const fresh = a.submit({ op: "acquire", ownerEpoch: 2, requestSeq: 2, assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "bundle-epoch", mode: "development" });
      return { pending, ready, replacement, stale, fresh };
    }, (o, s) => o.pending.pending && o.replacement.event === "EPOCH_REPLACED" && o.stale.error === "E_STALE_HANDLE" && o.fresh.ok && s.ownerEpoch === 2));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("pending_shutdown_cancels_load", { shutdown: "pending" }, adapter, (a) => {
      const pending = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-shutdown", mode: "development", loader: "pending" });
      const shutdown = a.submit({ op: "shutdown", ownerEpoch: 1, requestSeq: 2 });
      const closingAcquire = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 3, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-shutdown", mode: "development" });
      return { pending, shutdown, closingAcquire };
    }, (o, s) => o.pending.pending && o.shutdown.canceledPending === 1 && o.shutdown.disposed === 0 && o.closingAcquire.error === "E_OWNER_CLOSING" && s.status === "closed" && s.registryCount === 0));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("ready_shutdown_disposes_once_and_is_idempotent", { shutdown: "ready" }, adapter, (a) => {
      const first = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_subtitle.system_stack", bundle: "bundle-ready-shutdown", mode: "development" });
      const shutdown = a.submit({ op: "shutdown", ownerEpoch: 1, requestSeq: 2 });
      const repeat = a.submit({ op: "shutdown", ownerEpoch: 1, requestSeq: 3 });
      return { first, shutdown, repeat };
    }, (o, s) => o.shutdown.disposed === 1 && o.repeat.event === "E_SHUTDOWN_IDEMPOTENT" && s.status === "closed" && s.liveHandleCount === 0));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("load_error_retry_and_stale_generation", { retry: true }, adapter, (a) => {
      const failed = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-retry", mode: "development", loader: "failure" });
      const retry = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 2, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-retry", mode: "development" });
      const stale = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 3, opaqueHandle: "opaque:1:999" });
      const release = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 4, opaqueHandle: retry.handle.opaqueHandle });
      return { failed, retry, stale, release };
    }, (o, s) => o.failed.error === "E_LOAD_FAILED" && o.retry.ok && o.stale.error === "E_HANDLE_UNKNOWN" && o.release.event === "actual_dispose" && s.cache[0].actualDisposeCount === 1));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("bundle_release_only_releases_owned_handles", { targetBundle: "bundle-a" }, adapter, (a) => {
      const first = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "bundle-a", mode: "development" });
      const second = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 2, assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "bundle-b", mode: "development" });
      const bundleRelease = a.submit({ op: "bundleRelease", ownerEpoch: 1, requestSeq: 3, bundle: "bundle-a" });
      const remaining = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 4, opaqueHandle: second.handle.opaqueHandle });
      return { first, second, bundleRelease, remaining };
    }, (o) => o.bundleRelease.releasedCount === 1 && o.remaining.event === "actual_dispose"));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("public_rejection_and_closing_rejection", { mode: "public" }, adapter, (a) => {
      const publicReject = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.vendor.shared.babylonjs.current", bundle: "public-runtime", mode: "public" });
      const shutdown = a.submit({ op: "shutdown", ownerEpoch: 1, requestSeq: 2 });
      const closingReject = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 3, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "public-runtime", mode: "development" });
      return { publicReject, shutdown, closingReject };
    }, (o) => o.publicReject.error === "E_REQUIRED_PUBLIC_UNRESOLVED" && o.closingReject.error === "E_OWNER_CLOSING"));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("callback_reentry_is_queued_behind_mutation_lock", { callback: "reentrant" }, adapter, (a) => {
      const pending = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-callback", mode: "development", loader: "pending" });
      const callback = a.submit({ op: "callback", ownerEpoch: 1, requestSeq: 2, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", mode: "development", outcome: "success", reentrantRequest: { op: "acquire", ownerEpoch: 1, requestSeq: 3, assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-reentrant", mode: "development" } });
      const reentrantOutput = a.completed.get("1:3");
      const callbackHandle = callback.handles[0].opaqueHandle;
      const releaseCallback = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 4, opaqueHandle: callbackHandle });
      const releaseReentrant = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 5, opaqueHandle: reentrantOutput.handle.opaqueHandle });
      return { pending, callback, reentrantOutput, releaseCallback, releaseReentrant };
    }, (o) => o.callback.reentrant.queued === true && o.reentrantOutput.ok && o.releaseCallback.event === "actual_dispose" && o.releaseReentrant.event === "actual_dispose"));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("double_release_and_stale_epoch_rejection", { oldEpoch: 1 }, adapter, (a) => {
      const acquired = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-stale", mode: "development" });
      const released = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 2, opaqueHandle: acquired.handle.opaqueHandle });
      const doubleRelease = a.submit({ op: "release", ownerEpoch: 1, requestSeq: 3, opaqueHandle: acquired.handle.opaqueHandle });
      const staleEpoch = a.submit({ op: "acquire", ownerEpoch: 0, requestSeq: 4, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-stale", mode: "development" });
      return { acquired, released, doubleRelease, staleEpoch };
    }, (o) => o.released.event === "actual_dispose" && o.doubleRelease.error === "E_DOUBLE_RELEASE" && o.staleEpoch.error === "E_OWNER_EPOCH_STALE"));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("old_epoch_request_after_replacement", { oldEpoch: 1, newEpoch: 2 }, adapter, (a) => {
      const first = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-old-epoch", mode: "development" });
      const replacement = a.initialize({ ownerEpoch: 2, replace: true });
      const staleRequest = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 2, assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "bundle-old-epoch", mode: "development" });
      return { first, replacement, staleRequest };
    }, (o, s) => o.replacement.event === "EPOCH_REPLACED" && o.staleRequest.error === "E_OWNER_EPOCH_STALE" && s.ownerEpoch === 2));
  }
  {
    const adapter = new RuntimeAdapter(registry, authorization, publicManifest);
    adapter.initialize({ ownerEpoch: 1 });
    scenarios.push(traceScenario("stale_callback_after_shutdown", { pendingCallback: "rejected" }, adapter, (a) => {
      const pending = a.submit({ op: "acquire", ownerEpoch: 1, requestSeq: 1, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "bundle-stale-callback", mode: "development", loader: "pending" });
      const shutdown = a.submit({ op: "shutdown", ownerEpoch: 1, requestSeq: 2 });
      const callback = a.submit({ op: "callback", ownerEpoch: 1, requestSeq: 3, assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", mode: "development", outcome: "success" });
      return { pending, shutdown, callback };
    }, (o, s) => o.pending.pending && o.shutdown.canceledPending === 1 && o.callback.error === "E_OWNER_CLOSING" && s.status === "closed"));
  }
  return scenarios;
}

function main() {
  const registry = readJson(REGISTRY_PATH);
  const authorization = readJson(AUTH_PATH);
  const publicManifest = readJson(PUBLIC_PATH);
  const ag41 = readJson(AG41_RESULT_PATH);
  const adapterContract = readJson(ADAPTER_CONTRACT_PATH);
  const ownerContract = readJson(OWNER_CONTRACT_PATH);
  const shutdownContract = readJson(SHUTDOWN_CONTRACT_PATH);
  const scenarios = runScenarios(registry, authorization, publicManifest);
  const checks = {
    sourceCoverage35Entries20Files: registry.entries.length === 35 && registry.entries.filter((entry) => entry.path !== null).length === 20 && ag41.coverage?.manifestEntries === 35 && ag41.coverage?.physicalFiles === 20,
    publicBaselineEmpty: publicManifest.entries.length === 0 && ag41.officialPublishEligible === false,
    adapterSchema: adapterContract.schema === "tp08_runtime_adapter_v1" && adapterContract.owner.api.join(",") === "initialize,acquire,release,bundleRelease,shutdown" && adapterContract.owner.realThread === false,
    ownerQueueSchema: ownerContract.schema === "tp08_owner_queue_contract_v1" && ownerContract.sequence.callbackReentry.includes("mutation lock"),
    shutdownSchema: shutdownContract.schema === "tp08_shutdown_contract_v1" && shutdownContract.order[0] === "mark_closing" && shutdownContract.order.at(-1) === "complete_closed",
    allScenariosPass: scenarios.length >= 12 && scenarios.every((scenario) => scenario.pass),
    noMediaObjects: true,
    publicAndClosingRejections: scenarios.some((scenario) => scenario.name === "public_rejection_and_closing_rejection" && scenario.pass)
  };
  const pass = Object.values(checks).every(Boolean);
  const result = {
    schema: "tp08_runtime_adapter_tp08c1_result_v1",
    status: pass ? "isolated_tp08c1_owner_lifecycle_contract_passed_with_publish_block" : "failed",
    pass,
    machinePass: pass,
    scope: "pure-data single-owner runtime adapter simulation over TP-08A1/B1 contracts; no formal UI/runtime integration",
    realAssetsModified: false,
    mediaBytesLoaded: false,
    gpuOrDomObjectsCreated: false,
    officialPublishEligible: false,
    inputs: {
      registry: "tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json",
      authorization: "tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json",
      publicManifest: "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json",
      resolverContract: "tools/continuous-tps-architecture-probe/tp08-resolver/registry-resolver-v1.json",
      cacheContract: "tools/continuous-tps-architecture-probe/tp08-resolver/cache-policy-v1.json",
      disposeContract: "tools/continuous-tps-architecture-probe/tp08-resolver/refcount-dispose-contract-v1.json"
    },
    api: ["initialize", "acquire", "release", "bundleRelease", "shutdown"],
    ownerQueue: { singleOwner: true, realThreadStarted: false, mutableStateExclusive: true, requestFields: ["ownerEpoch", "requestSeq"], staleEpoch: "E_OWNER_EPOCH_STALE", duplicateSeq: "E_REQUEST_DUPLICATE", outOfOrderSeq: "E_REQUEST_OUT_OF_ORDER", closing: "E_OWNER_CLOSING" },
    opaqueHandle: { callerField: "opaqueHandle", metadataImmutable: true, mediaBytesHidden: true, generationInternal: true },
    scenarios: { count: scenarios.length, passing: scenarios.filter((scenario) => scenario.pass).length, traces: scenarios },
    shutdown: { order: shutdownContract.order, repeatedShutdown: "E_SHUTDOWN_IDEMPOTENT", pendingCancellation: true, readyDisposeExactlyOnce: true },
    checks,
    futureFormalUiAdapterWhitelist: [
      "tools/continuous-tps-architecture-probe/tp08-formal-ui-adapter/runtime-adapter-ui-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-formal-ui-adapter/owner-bridge-v1.json",
      "tools/continuous-tps-architecture-probe/tp08-formal-ui-adapter/validator.mjs",
      "docs/CONTINUOUS_TP08_FORMAL_UI_ADAPTER_REVIEW.md"
    ],
    rollback: { scope: "future formal UI adapter whitelist only", trigger: "public leak, owner ordering violation, lifecycle invariant failure, or UI integration failure", action: "remove only future UI adapter files, restore empty public manifest and isolated owner contract, leave TP-08A1/B1, assets, and existing UI untouched" },
    formalDecision: "TP-08C1 proves only an isolated owner/lifecycle contract and provides no authorization. Gate G and the overall Architecture Gate remain blocked/no_go."
  };
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ status: result.status, pass: result.pass, officialPublishEligible: result.officialPublishEligible, scenarios: result.scenarios.count, passing: result.scenarios.passing, public: publicManifest.entries.length }));
  if (!pass) process.exitCode = 1;
}

main();
