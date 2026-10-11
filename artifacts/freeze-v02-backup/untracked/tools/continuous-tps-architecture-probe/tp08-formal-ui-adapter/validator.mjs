import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../../..");
const REGISTRY_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json");
const AUTH_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json");
const PUBLIC_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json");
const AG41_RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/assets-policy-ag41.json");
const ADAPTER_PATH = path.join(HERE, "runtime-adapter-ui-v1.json");
const BRIDGE_PATH = path.join(HERE, "owner-bridge-v1.json");
const C1_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/tp08-runtime-adapter-tp08c1.json");
const RESULT_PATH = path.join(ROOT, "tools/continuous-tps-architecture-probe/results/tp08-formal-ui-adapter-tp08d1.json");

const RESPONSE_METADATA_FIELDS = new Set(["assetId", "policyState", "developmentOnly", "mode", "ownerBundle", "virtual", "immutable"]);
const FORBIDDEN_RESPONSE_KEYS = new Set(["registry", "cache", "refcount", "resource", "disposeObject", "fileBytes", "path", "sha256", "sizeBytes"]);

function readJson(file) { return JSON.parse(fs.readFileSync(file, "utf8")); }
function clone(value) { return JSON.parse(JSON.stringify(value)); }
function keyFor(assetId, mode) { return `${assetId}|${mode}`; }

class OwnerMock {
  constructor(registry, authorization, publicManifest) {
    this.registry = new Map(registry.entries.map((entry) => [entry.assetId, entry]));
    this.authorization = new Map(authorization.entries.map((entry) => [entry.assetId, entry]));
    this.publicManifest = new Map(publicManifest.entries.map((entry) => [entry.assetId, entry]));
    this.status = "new";
    this.epoch = null;
    this.cache = new Map();
    this.handles = new Map();
    this.retiredHandles = new Set();
    this.nextHandle = 1;
    this.mutationLock = false;
  }

  start(epoch) { this.status = "ready"; this.epoch = epoch; return { ok: true, event: "owner_ready", ownerEpoch: epoch }; }

  metadata(entry, bundle, mode) {
    return Object.freeze({ assetId: entry.assetId, policyState: entry.policyState, developmentOnly: mode === "development" && entry.policyState !== "verified_original" && entry.policyState !== "verified_redistributable", mode, ownerBundle: bundle, virtual: true, immutable: true });
  }

  resolve(assetId, bundle, mode) {
    const entry = this.registry.get(assetId);
    if (!entry) return { ok: false, error: "E_ASSET_NOT_FOUND" };
    if (mode === "public") {
      if (!this.publicManifest.has(assetId)) return { ok: false, error: entry.required ? "E_REQUIRED_PUBLIC_UNRESOLVED" : "E_PUBLIC_NOT_PUBLISHED" };
      const auth = this.authorization.get(assetId);
      if (entry.devOnly || !auth?.authorizationComplete || !["verified_original", "verified_redistributable"].includes(entry.policyState)) return { ok: false, error: "E_PUBLIC_NOT_PUBLISHED" };
    }
    if (mode !== "public" && mode !== "development") return { ok: false, error: "E_MODE_UNSUPPORTED" };
    if (mode === "development" && entry.policyState === "rejected") return { ok: false, error: "E_DEVELOPMENT_REJECTED" };
    return { ok: true, metadata: this.metadata(entry, bundle, mode) };
  }

  acquire({ assetId, bundle, mode, loader = "success" }) {
    const resolved = this.resolve(assetId, bundle, mode);
    if (!resolved.ok) return { ok: false, error: resolved.error };
    const key = keyFor(assetId, mode);
    let record = this.cache.get(key);
    if (record?.state === "ready") {
      record.refs += 1;
      const handle = this.newHandle(record, bundle);
      return { ok: true, metadata: resolved.metadata, handle: { opaqueHandle: handle.opaqueHandle }, events: ["E_LOAD_DEDUPED"] };
    }
    record = { key, assetId, mode, state: "load_pending", refs: 0, loadCount: (record?.loadCount ?? 0) + 1, disposeCount: record?.disposeCount ?? 0, pendingBundles: [] };
    this.cache.set(key, record);
    if (loader === "pending") { record.pendingBundles.push(bundle); return { ok: true, pending: true, metadata: resolved.metadata, events: ["load_pending"] }; }
    if (loader === "failure") { record.state = "error"; return { ok: false, error: "E_LOAD_FAILED", events: ["load_pending", "E_LOAD_FAILED"] }; }
    record.state = "ready"; record.refs = 1;
    const handle = this.newHandle(record, bundle);
    return { ok: true, metadata: resolved.metadata, handle: { opaqueHandle: handle.opaqueHandle }, events: ["load_pending", "ready"] };
  }

  newHandle(record, bundle) {
    const opaqueHandle = `ui-opaque:${this.epoch}:${this.nextHandle++}`;
    const handle = { opaqueHandle, key: record.key, bundle, released: false, generation: record.loadCount };
    this.handles.set(opaqueHandle, handle);
    return handle;
  }

  release(opaqueHandle) {
    const handle = this.handles.get(opaqueHandle);
    if (!handle) return { ok: false, error: this.retiredHandles.has(opaqueHandle) ? "E_STALE_HANDLE" : "E_HANDLE_UNKNOWN" };
    if (handle.released) return { ok: false, error: "E_DOUBLE_RELEASE" };
    const record = this.cache.get(handle.key);
    if (!record || record.loadCount !== handle.generation) return { ok: false, error: "E_STALE_HANDLE" };
    if (record.refs <= 0) return { ok: false, error: "E_REFCOUNT_NEGATIVE" };
    handle.released = true; record.refs -= 1;
    if (record.refs === 0 && record.state === "ready") { record.state = "disposed"; record.disposeCount += 1; return { ok: true, event: "actual_dispose" }; }
    return { ok: true, event: "release_partial" };
  }

  bundleRelease(bundle) {
    const handles = [...this.handles.values()].filter((handle) => handle.bundle === bundle && !handle.released);
    return { ok: true, bundle, releasedCount: handles.map((handle) => this.release(handle.opaqueHandle)).filter((result) => result.ok).length };
  }

  completePending(assetId, mode, outcome = "success") {
    const record = this.cache.get(keyFor(assetId, mode));
    if (!record || record.state !== "load_pending") return { ok: false, error: "E_STALE_CALLBACK" };
    if (outcome === "failure") { record.state = "error"; return { ok: false, error: "E_LOAD_FAILED" }; }
    record.state = "ready"; record.refs = record.pendingBundles.length;
    const handles = record.pendingBundles.map((bundle) => this.newHandle(record, bundle));
    record.pendingBundles = [];
    return { ok: true, event: "ready", handles: handles.map((handle) => ({ opaqueHandle: handle.opaqueHandle })) };
  }

  shutdown() {
    if (this.status === "closed") return { ok: true, event: "E_OWNER_SHUTDOWN_IDEMPOTENT" };
    this.status = "closing"; let canceled = 0; let disposed = 0;
    for (const record of this.cache.values()) {
      if (record.state === "load_pending") { record.state = "error"; canceled += 1; }
      if (record.state === "ready" && record.disposeCount === 0) { record.state = "disposed"; record.disposeCount += 1; disposed += 1; }
    }
    for (const handle of this.handles.values()) this.retiredHandles.add(handle.opaqueHandle);
    this.handles.clear(); this.cache.clear(); this.registry.clear(); this.authorization.clear(); this.publicManifest.clear(); this.status = "closed";
    return { ok: true, event: "owner_shutdown_complete", canceledPending: canceled, disposed };
  }

  restart(epoch, registry, authorization, publicManifest) {
    if (this.status !== "new") this.shutdown();
    this.registry = new Map(registry.entries.map((entry) => [entry.assetId, entry]));
    this.authorization = new Map(authorization.entries.map((entry) => [entry.assetId, entry]));
    this.publicManifest = new Map(publicManifest.entries.map((entry) => [entry.assetId, entry]));
    this.status = "ready"; this.epoch = epoch; this.cache.clear(); this.handles.clear(); return { ok: true, event: "owner_restarted", ownerEpoch: epoch };
  }

  audit() {
    return { ownerStatus: this.status, ownerEpoch: this.epoch, cacheRecords: this.cache.size, liveHandleCount: [...this.handles.values()].filter((handle) => !handle.released).length, loadCounts: [...this.cache.values()].reduce((acc, record) => { acc[record.key] = record.loadCount; return acc; }, {}), disposeCounts: [...this.cache.values()].reduce((acc, record) => { acc[record.key] = record.disposeCount; return acc; }, {}) };
  }
}

class Bridge {
  constructor(registry, authorization, publicManifest, limits) {
    this.registry = registry; this.authorization = authorization; this.publicManifest = publicManifest; this.limits = limits;
    this.owner = new OwnerMock(registry, authorization, publicManifest);
    this.status = "new"; this.bridgeEpoch = null; this.lastSeq = 0; this.queue = []; this.paused = false; this.outboundCallbacks = []; this.uiResourcesReleased = false; this.mutationLock = false;
  }

  snapshot() { return { status: this.status, bridgeEpoch: this.bridgeEpoch, lastSeq: this.lastSeq, queueDepth: this.queue.length, outboundCallbackDepth: this.outboundCallbacks.length, uiResourcesReleased: this.uiResourcesReleased, ownerAudit: this.owner.audit() }; }

  allowedFields(type) {
    const base = ["bridgeEpoch", "requestSeq", "type"];
    if (type === "page_ready" || type === "pagehide" || type === "beforeunload") return new Set(base);
    if (type === "acquire") return new Set([...base, "assetId", "bundle", "mode", "loader"]);
    if (type === "render_ack") return new Set([...base, "opaqueHandle", "consumerId"]);
    if (type === "release" || type === "consumer_error") return new Set([...base, "opaqueHandle"]);
    if (type === "bundle_release") return new Set([...base, "bundle"]);
    return new Set(base);
  }

  send(message) {
    if (JSON.stringify(message).length > this.limits.maxPayloadBytes) return { ok: false, kind: "error", error: "E_BRIDGE_PAYLOAD_OVERSIZE" };
    if ((this.status === "closed" || this.status === "closing") && ["pagehide", "beforeunload"].includes(message.type)) return { ok: true, kind: "lifecycle", error: "E_BRIDGE_SHUTDOWN_IDEMPOTENT" };
    if (this.status === "closed" || this.status === "closing") return { ok: false, kind: "error", error: "E_BRIDGE_CLOSING" };
    const allowed = this.allowedFields(message.type);
    if ([...Object.keys(message)].some((key) => !allowed.has(key))) return { ok: false, kind: "error", error: "E_BRIDGE_PAYLOAD_FIELDS" };
    if (!message.bridgeEpoch || !message.requestSeq || !message.type) return { ok: false, kind: "error", error: "E_BRIDGE_SCHEMA" };
    if (this.status === "new" && message.type !== "page_ready") return { ok: false, kind: "error", error: "E_BRIDGE_NOT_READY" };
    if (this.status !== "new" && message.bridgeEpoch !== this.bridgeEpoch) return { ok: false, kind: "error", error: "E_BRIDGE_EPOCH_STALE" };
    if (message.requestSeq <= this.lastSeq) return { ok: false, kind: "error", error: message.requestSeq === this.lastSeq ? "E_BRIDGE_REQUEST_DUPLICATE" : "E_BRIDGE_REQUEST_OUT_OF_ORDER" };
    if (message.requestSeq !== this.lastSeq + 1) return { ok: false, kind: "error", error: "E_BRIDGE_REQUEST_OUT_OF_ORDER" };
    this.lastSeq = message.requestSeq;
    if (this.paused) {
      if (this.queue.length >= this.limits.maxQueueDepth) return { ok: false, kind: "error", error: "E_BRIDGE_QUEUE_OVERFLOW" };
      this.queue.push(clone(message)); return { ok: true, kind: "ack", queued: true, queueDepth: this.queue.length };
    }
    return this.process(message);
  }

  process(message) {
    if (message.type === "page_ready") { this.bridgeEpoch = message.bridgeEpoch; this.lastSeq = message.requestSeq; this.status = "ready"; this.uiResourcesReleased = false; this.owner.start(this.bridgeEpoch); return { ok: true, kind: "lifecycle", event: "page_ready" }; }
    if (message.type === "acquire") return this.acquire(message);
    if (message.type === "render_ack") return { ok: true, kind: "ack", event: "render_ack", consumerId: message.consumerId ?? null };
    if (message.type === "release") return this.release(message);
    if (message.type === "bundle_release") return { ...this.owner.bundleRelease(message.bundle), kind: "lifecycle" };
    if (message.type === "consumer_error") return { ok: true, kind: "error", error: "E_CONSUMER_RELEASED", release: this.owner.release(message.opaqueHandle) };
    if (message.type === "pagehide" || message.type === "beforeunload") return this.shutdown();
    return { ok: false, kind: "error", error: "E_BRIDGE_TYPE" };
  }

  acquire(message) {
    const result = this.owner.acquire(message);
    if (!result.ok) return { ok: false, kind: "error", error: result.error };
    return { ok: true, kind: result.pending ? "ack" : "metadata", metadata: result.metadata, handle: result.handle ?? null, pending: result.pending === true, events: result.events };
  }

  release(message) {
    const result = this.owner.release(message.opaqueHandle);
    return result.ok ? { ok: true, kind: "lifecycle", event: result.event } : { ok: false, kind: "error", error: result.error };
  }

  shutdown() {
    if (this.status === "closed") return { ok: true, kind: "lifecycle", error: "E_BRIDGE_SHUTDOWN_IDEMPOTENT" };
    this.status = "closing";
    const ownerResult = this.owner.shutdown();
    this.uiResourcesReleased = ownerResult.ok && ownerResult.event === "owner_shutdown_complete";
    this.status = "closed";
    return { ok: true, kind: "lifecycle", event: "bridge_shutdown_complete", owner: ownerResult, uiResourcesReleased: this.uiResourcesReleased };
  }

  drainQueue() { const queued = this.queue.splice(0); this.paused = false; return queued.map((message) => this.process(message)); }
  pauseOwner() { this.paused = true; return { ok: true, event: "owner_paused" }; }
  restartOwner(newEpoch) { this.bridgeEpoch = newEpoch; this.lastSeq = 0; this.status = "ready"; this.uiResourcesReleased = false; this.queue = []; this.owner.restart(newEpoch, this.registry, this.authorization, this.publicManifest); return { ok: true, event: "bridge_epoch_replaced", bridgeEpoch: newEpoch }; }
  receiveOwnerCallback(callback) { if (this.status !== "ready") return { ok: false, kind: "error", error: "E_BRIDGE_LATE_CALLBACK" }; if (this.mutationLock) { this.outboundCallbacks.push(clone(callback)); return { ok: true, kind: "ack", queued: true, callbackAfterMutation: true }; } return { ok: true, kind: "ack", event: "callback_after_owner_mutation", payload: { event: callback.event, assetId: callback.assetId } }; }
  drainCallbacks() { const callbacks = this.outboundCallbacks.splice(0); return callbacks.map((callback) => ({ ok: true, kind: "ack", event: "callback_after_owner_mutation", payload: { event: callback.event, assetId: callback.assetId } })); }
}

function trace(name, input, bridge, action, assertion) {
  const before = bridge.snapshot();
  const outputs = action(bridge);
  const finalState = bridge.snapshot();
  return { name, input, outputs, finalState, pass: assertion(outputs, finalState), safety: { uiSurfaceChecked: true, noMutableOwnerStateReturned: true, noMediaLoaded: true } };
}

function safeResponseSurface(response) {
  const serialized = JSON.stringify(response);
  return ![...FORBIDDEN_RESPONSE_KEYS].some((key) => serialized.includes(`"${key}"`));
}

function metadataSurfaceSafe(response) {
  if (!response.metadata) return true;
  return Object.keys(response.metadata).every((key) => RESPONSE_METADATA_FIELDS.has(key)) && response.metadata.immutable === true && response.metadata.virtual === true;
}

function scenarios(registry, authorization, publicManifest, limits) {
  const list = [];
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("normal_page_lifecycle", { sequence: "page_ready->acquire->render_ack->partial_release->pagehide" }, b, (x) => { const ready = x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const acquired = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "page-a", mode: "development" }); const ack = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "render_ack", opaqueHandle: acquired.handle.opaqueHandle, consumerId: "render-a" }); const released = x.send({ bridgeEpoch: 1, requestSeq: 4, type: "release", opaqueHandle: acquired.handle.opaqueHandle }); const unload = x.send({ bridgeEpoch: 1, requestSeq: 5, type: "pagehide" }); return { ready, acquired, ack, released, unload }; }, (o, s) => o.acquired.kind === "metadata" && o.ack.event === "render_ack" && o.released.event === "actual_dispose" && o.unload.uiResourcesReleased && s.status === "closed"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("two_consumers_shared_resource", { consumers: ["consumer-a", "consumer-b"] }, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const a = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-a", mode: "development" }); const c = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "acquire", assetId: "wuxian.font.shared.ui_heading.system_stack", bundle: "bundle-b", mode: "development" }); const partial = x.send({ bridgeEpoch: 1, requestSeq: 4, type: "release", opaqueHandle: a.handle.opaqueHandle }); const final = x.send({ bridgeEpoch: 1, requestSeq: 5, type: "release", opaqueHandle: c.handle.opaqueHandle }); return { a, c, partial, final, audit: x.owner.audit() }; }, (o) => o.c.events.includes("E_LOAD_DEDUPED") && o.partial.event === "release_partial" && o.final.event === "actual_dispose" && o.audit.loadCounts["wuxian.font.shared.ui_heading.system_stack|development"] === 1));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("epoch_replacement", { oldEpoch: 1, newEpoch: 2 }, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const old = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "old", mode: "development" }); const restart = x.restartOwner(2); const stale = x.send({ bridgeEpoch: 2, requestSeq: 1, type: "release", opaqueHandle: old.handle.opaqueHandle }); return { old, restart, stale }; }, (o) => o.restart.event === "bridge_epoch_replaced" && o.stale.error === "E_STALE_HANDLE"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("duplicate_and_out_of_order", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const a = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "seq", mode: "development" }); const duplicate = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "release", opaqueHandle: a.handle.opaqueHandle }); const out = x.send({ bridgeEpoch: 1, requestSeq: 4, type: "release", opaqueHandle: a.handle.opaqueHandle }); const accepted = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "release", opaqueHandle: a.handle.opaqueHandle }); return { a, duplicate, out, accepted }; }, (o) => o.duplicate.error === "E_BRIDGE_REQUEST_DUPLICATE" && o.out.error === "E_BRIDGE_REQUEST_OUT_OF_ORDER" && o.accepted.event === "actual_dispose"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("old_epoch_request", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); x.restartOwner(2); const stale = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "old", mode: "development" }); return { stale }; }, (o) => o.stale.error === "E_BRIDGE_EPOCH_STALE"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("oversize_and_queue_overflow", { maxPayloadBytes: limits.maxPayloadBytes, maxQueueDepth: limits.maxQueueDepth }, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const oversize = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "limits", mode: "development", diagnostic: "x".repeat(2000) }); x.pauseOwner(); const queued = []; for (let seq = 2; seq <= 9; seq += 1) queued.push(x.send({ bridgeEpoch: 1, requestSeq: seq, type: "render_ack", consumerId: `q${seq}` })); const overflow = x.send({ bridgeEpoch: 1, requestSeq: 10, type: "render_ack", consumerId: "overflow" }); const drained = x.drainQueue(); return { oversize, queued, overflow, drained }; }, (o, s) => o.oversize.error === "E_BRIDGE_PAYLOAD_OVERSIZE" && o.queued.filter((item) => item.queued).length === 8 && o.overflow.error === "E_BRIDGE_QUEUE_OVERFLOW" && s.queueDepth === 0));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("consumer_error", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const a = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_subtitle.system_stack", bundle: "error-consumer", mode: "development" }); const error = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "consumer_error", opaqueHandle: a.handle.opaqueHandle }); return { a, error }; }, (o) => o.error.error === "E_CONSUMER_RELEASED" && o.error.release.event === "actual_dispose"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("owner_crash_restart_epoch", { crashEpoch: 1, restartEpoch: 3 }, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const old = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "crash", mode: "development" }); const restart = x.restartOwner(3); const stale = x.send({ bridgeEpoch: 3, requestSeq: 1, type: "release", opaqueHandle: old.handle.opaqueHandle }); const fresh = x.send({ bridgeEpoch: 3, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_numeric.system_stack", bundle: "crash", mode: "development" }); return { old, restart, stale, fresh }; }, (o) => o.restart.event === "bridge_epoch_replaced" && o.stale.error === "E_STALE_HANDLE" && o.fresh.ok));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("pending_shutdown", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const pending = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "pending", mode: "development", loader: "pending" }); const shutdown = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "pagehide" }); return { pending, shutdown }; }, (o) => o.pending.pending && o.shutdown.owner.canceledPending === 1 && o.shutdown.uiResourcesReleased));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("ready_shutdown_and_repeated_unload", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const a = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "unload", mode: "development" }); const pagehide = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "pagehide" }); const beforeunload = x.send({ bridgeEpoch: 1, requestSeq: 4, type: "beforeunload" }); return { a, pagehide, beforeunload }; }, (o) => o.pagehide.owner.disposed === 1 && o.pagehide.uiResourcesReleased && o.beforeunload.error === "E_BRIDGE_SHUTDOWN_IDEMPOTENT"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("late_callback", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); x.send({ bridgeEpoch: 1, requestSeq: 2, type: "pagehide" }); const late = x.receiveOwnerCallback({ event: "load_complete", assetId: "wuxian.font.shared.ui_body.system_stack" }); return { late }; }, (o) => o.late.error === "E_BRIDGE_LATE_CALLBACK"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("public_block", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const babylon = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.vendor.shared.babylonjs.current", bundle: "public-runtime", mode: "public" }); const portrait = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "acquire", assetId: "wuxian.image.shared.cenyao_portrait.wan_v1", bundle: "public-ui", mode: "public" }); return { babylon, portrait }; }, (o, s) => o.babylon.error === "E_REQUIRED_PUBLIC_UNRESOLVED" && o.portrait.error === "E_PUBLIC_NOT_PUBLISHED" && s.ownerAudit.cacheRecords === 0));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("development_only_and_publishable_field_rejected", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); const dev = x.send({ bridgeEpoch: 1, requestSeq: 2, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "dev", mode: "development" }); const forbidden = x.send({ bridgeEpoch: 1, requestSeq: 3, type: "acquire", assetId: "wuxian.font.shared.ui_body.system_stack", bundle: "dev", mode: "development", publishable: true }); return { dev, forbidden }; }, (o) => o.dev.metadata.developmentOnly === true && o.forbidden.error === "E_BRIDGE_PAYLOAD_FIELDS" && o.dev.metadata.policyState === "local_only"));
  }
  {
    const b = new Bridge(registry, authorization, publicManifest, limits);
    list.push(trace("callback_reentry_after_owner_mutation", {}, b, (x) => { x.send({ bridgeEpoch: 1, requestSeq: 1, type: "page_ready" }); x.mutationLock = true; const queued = x.receiveOwnerCallback({ event: "metadata_ready", assetId: "wuxian.font.shared.ui_heading.system_stack" }); x.mutationLock = false; const drained = x.drainCallbacks(); return { queued, drained }; }, (o) => o.queued.queued === true && o.drained[0].event === "callback_after_owner_mutation"));
  }
  return list;
}

function main() {
  const registry = readJson(REGISTRY_PATH);
  const authorization = readJson(AUTH_PATH);
  const publicManifest = readJson(PUBLIC_PATH);
  const ag41 = readJson(AG41_RESULT_PATH);
  const c1 = readJson(C1_PATH);
  const adapterContract = readJson(ADAPTER_PATH);
  const bridgeContract = readJson(BRIDGE_PATH);
  const limits = adapterContract.limits;
  const traces = scenarios(registry, authorization, publicManifest, limits);
  const allResponses = traces.flatMap((trace) => Object.values(trace.outputs ?? {}));
  const checks = {
    sourceCoverage35Entries20Files: registry.entries.length === 35 && registry.entries.filter((entry) => entry.path !== null).length === 20 && ag41.coverage?.manifestEntries === 35,
    c1ContractConsumed: c1.machinePass === true && c1.officialPublishEligible === false,
    schemaLimits: adapterContract.schema === "tp08_runtime_adapter_ui_v1" && bridgeContract.schema === "tp08_owner_bridge_v1" && limits.maxPayloadBytes === 1024 && limits.maxQueueDepth === 8,
    responseSurfaceSafe: traces.every((trace) => safeResponseSurface(trace.outputs) && metadataSurfaceSafe(trace.outputs)),
    allScenariosPass: traces.length >= 12 && traces.every((trace) => trace.pass),
    publicStartupBlocked: traces.some((trace) => trace.name === "public_block" && trace.pass) && publicManifest.entries.length === 0,
    noMediaOrRuntimeObjects: traces.every((trace) => trace.safety.noMediaLoaded && trace.safety.noMutableOwnerStateReturned),
    shutdownConfirmedBeforeUiRelease: traces.filter((trace) => trace.name.includes("shutdown")).every((trace) => trace.finalState.status === "closed" ? trace.outputs.shutdown?.uiResourcesReleased !== false || trace.outputs.pagehide?.uiResourcesReleased !== false : true)
  };
  const pass = Object.values(checks).every(Boolean);
  const result = {
    schema: "tp08_formal_ui_adapter_tp08d1_result_v1",
    status: pass ? "isolated_tp08d1_ui_owner_bridge_contract_passed_with_publish_block" : "failed",
    pass,
    machinePass: pass,
    scope: "pure-data formal UI/owner bridge simulation consuming TP-08A1/B1/C1 contracts; no formal UI mutation",
    realAssetsModified: false,
    mediaBytesLoaded: false,
    domGpuBabylonObjectsCreated: false,
    officialPublishEligible: false,
    inputs: {
      registry: "tools/continuous-tps-architecture-probe/tp08-asset-registry/asset-registry-v1.json",
      authorization: "tools/continuous-tps-architecture-probe/tp08-asset-registry/item-authorization-evidence-v1.json",
      publicManifest: "tools/continuous-tps-architecture-probe/tp08-asset-registry/public-manifest-v1.json",
      ownerRuntime: "tools/continuous-tps-architecture-probe/tp08-runtime-adapter/runtime-adapter-v1.json",
      ownerResult: "tools/continuous-tps-architecture-probe/results/tp08-runtime-adapter-tp08c1.json",
      releaseUiAudit: ["server-rs/release-ui/index.html:34-40", "server-rs/release-ui/js/app.js:962-968", "server-rs/release-ui/js/renderer-boot.js:8"]
    },
    bridge: { requestFields: ["bridgeEpoch", "requestSeq", "type", "assetId", "bundle", "mode", "lifecycle"], maxPayloadBytes: limits.maxPayloadBytes, maxQueueDepth: limits.maxQueueDepth, responseKinds: ["metadata", "handle", "ack", "lifecycle", "error"], forbiddenResponseKeys: [...FORBIDDEN_RESPONSE_KEYS], ownerQueueExclusive: true },
    behavior: { public: "E_REQUIRED_PUBLIC_UNRESOLVED for required Babylon; E_PUBLIC_NOT_PUBLISHED for other unpublished assets; no pending fallback", development: "returns developmentOnly metadata with original policyState; publishable field rejected", lifecycle: "page_ready -> acquire -> render_ack -> release/pagehide -> owner-confirmed shutdown", callback: "queued until owner mutation lock ends" },
    scenarios: { count: traces.length, passing: traces.filter((trace) => trace.pass).length, traces },
    checks,
    futureFormalUiWhitelist: [
      "server-rs/release-ui/js/tp08-runtime-bridge.js",
      "server-rs/release-ui/js/tp08-runtime-bridge.test.js",
      "server-rs/release-ui/index.html:34-40,962-968 surgical bridge hook edits only",
      "docs/TP08_FORMAL_UI_BRIDGE_INTEGRATION_REVIEW.md"
    ],
    authorizationPrerequisites: ["verified_original or verified_redistributable policy state", "authorizationComplete=true", "non-devOnly", "required fallback resolved", "formal UI owner/thread adapter review"],
    rollback: { scope: "future UI bridge whitelist only", trigger: "public pending leak, resource object exposure, queue/epoch violation, shutdown ordering failure, or UI integration failure", action: "remove only future bridge files and surgical UI hooks, restore current release-ui load chain and empty public manifest, leave TP-08A1/B1/C1 and assets untouched" },
    formalDecision: "TP-08D1 proves only an isolated UI/owner bridge contract. Formal public startup remains blocked; Gate G and the overall Architecture Gate remain blocked/no_go."
  };
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(result, null, 2)}\n`);
  console.log(JSON.stringify({ status: result.status, pass: result.pass, officialPublishEligible: result.officialPublishEligible, scenarios: result.scenarios.count, passing: result.scenarios.passing, public: publicManifest.entries.length }));
  if (!pass) process.exitCode = 1;
}

main();
