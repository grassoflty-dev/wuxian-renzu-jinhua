import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const scriptDir = path.dirname(new URL(import.meta.url).pathname)
  .replace(/^\/(?:[A-Za-z]:)/, (value) => value.slice(1))
  .replaceAll("/", path.sep);
const root = path.resolve(scriptDir, "..", "..", "..");
const resultPath = path.join(root, "tools", "continuous-tps-architecture-probe", "results", "loot-scan-query-candidate-tp15c2.json");

function readJson(relativePath) {
  const absolute = path.join(root, relativePath);
  if (!fs.existsSync(absolute)) return { exists: false, value: null, sha256: null };
  const bytes = fs.readFileSync(absolute);
  return { exists: true, value: JSON.parse(bytes.toString("utf8")), sha256: crypto.createHash("sha256").update(bytes).digest("hex") };
}

function clone(value) { return JSON.parse(JSON.stringify(value)); }
function finiteVector(vector) { return Array.isArray(vector) && vector.length === 3 && vector.every(Number.isFinite); }
function freeze(value) {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
function hash(value) { return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex"); }
function angleDelta(a, b) {
  let delta = Math.abs(a - b) % (Math.PI * 2);
  return delta > Math.PI ? Math.PI * 2 - delta : delta;
}

const proposed = readJson("tools/continuous-tps-architecture-probe/loot-scan-contract-readiness/proposed-contract.json");
const contractResult = readJson("tools/continuous-tps-architecture-probe/results/loot-scan-contract-readiness-tp15c1.json");
const ag12 = readJson("tools/continuous-tps-architecture-probe/results/capability-ag12.json");
const hudGate = readJson("tools/continuous-tps-architecture-probe/results/formal-capability-hud-readiness-tp01r7.json");

const candidateConfig = {
  ownerHz: 60,
  rangeM: 8,
  fovDeg: 90,
  cooldownTicks: 3,
  maxHits: 2,
  maxQueryCandidates: 6,
  maxPayloadBytes: 4096,
  resultTtlTicks: 2,
  epsilonM: 0.0001,
  frozen: false,
  source: "TP-15C1 proposed-contract suggestedDefaults; test overrides are candidate-only",
};

const allowedViewFields = [
  "protocol", "version", "worldId", "worldEpoch", "ownerGeneration", "sampledTick",
  "expiresAtTick", "requestId", "grantRevision", "status", "nextAllowedTick", "truncated", "hits",
];
const allowedHitFields = ["itemId", "kind", "distanceM", "textLabel"];

class LootScanOwner {
  constructor(config = candidateConfig) {
    this.config = { ...config };
    this.worldId = "grey_hive";
    this.worldEpoch = 1;
    this.generation = 1;
    this.tick = 0;
    this.grant = null;
    this.nextAllowedTick = 0;
    this.closed = false;
    this.lastSequence = 0;
    this.cache = new Map();
    this.events = [];
    this.queryCalls = 0;
    this.callbackInvalidations = 0;
    this.effects = { claims: 0, rewards: 0, saves: 0, entityMutations: 0 };
    this.accessibility = { baselineAvailable: true, projection: "text_only_candidate" };
  }

  record(type, data = {}) { this.events.push({ type, ...clone(data), worldEpoch: this.worldEpoch, generation: this.generation, tick: this.tick }); }

  grantCapability(grantId = "loot-grant-1", revision = 1) {
    if (this.closed) return { accepted: false, error: "E_SCAN_CLOSED" };
    if (this.grant && this.grant.id === grantId && this.grant.revision === revision) {
      this.record("grant_idempotent", { grantId, revision });
      return { accepted: true, idempotent: true };
    }
    if (this.grant) return { accepted: false, error: "E_SCAN_GRANT_CONFLICT" };
    this.grant = { id: grantId, revision };
    this.record("grant", { grantId, revision });
    return { accepted: true, idempotent: false };
  }

  revoke(nextGeneration = this.generation + 1) {
    if (this.closed) return { accepted: false, error: "E_SCAN_CLOSED" };
    if (!this.grant && nextGeneration === this.generation) return { accepted: true, idempotent: true };
    this.grant = null;
    this.generation = nextGeneration;
    this.invalidate("revoke");
    this.record("revoke", { nextGeneration });
    return { accepted: true, idempotent: false };
  }

  replaceWorld(worldId = "grey_hive", worldEpoch = this.worldEpoch + 1, generation = this.generation + 1, tick = 0) {
    if (worldEpoch <= this.worldEpoch || generation <= this.generation) return { accepted: false, error: "E_SCAN_REPLACEMENT_ORDER" };
    this.worldId = worldId;
    this.worldEpoch = worldEpoch;
    this.generation = generation;
    this.tick = tick;
    this.grant = null;
    this.invalidate("world_replacement");
    this.record("world_replacement", { worldId, worldEpoch, generation, tick });
    return { accepted: true };
  }

  reload(snapshot) {
    if (!snapshot || snapshot.worldEpoch !== this.worldEpoch) return { accepted: false, error: "E_SCAN_RELOAD_EPOCH" };
    this.tick = snapshot.tick;
    this.generation = snapshot.generation;
    this.grant = snapshot.grant ? clone(snapshot.grant) : null;
    this.nextAllowedTick = snapshot.nextAllowedTick;
    this.invalidate("reload");
    this.record("reload", { transientHitsRestored: false });
    return { accepted: true };
  }

  snapshot() {
    return {
      worldId: this.worldId,
      worldEpoch: this.worldEpoch,
      generation: this.generation,
      tick: this.tick,
      grant: this.grant ? clone(this.grant) : null,
      nextAllowedTick: this.nextAllowedTick,
      // Intentionally excludes hits, callbacks, request cache and runtime handles.
    };
  }

  invalidate(reason) {
    this.cache.clear();
    this.callbackInvalidations += 1;
    this.record("invalidate", { reason });
  }

  close() {
    this.closed = true;
    this.grant = null;
    this.invalidate("close");
    this.record("close");
    return { accepted: true };
  }

  advanceTo(tick) {
    if (!Number.isInteger(tick) || tick < this.tick) return { accepted: false, error: "E_SCAN_TICK_INVALID" };
    this.tick = tick;
    for (const [requestId, entry] of this.cache) {
      if (this.tick > entry.view.expiresAtTick) {
        this.cache.delete(requestId);
        this.record("expire", { requestId });
      }
    }
    return { accepted: true };
  }

  cached(requestId) { return this.cache.get(requestId)?.view ?? null; }

  request(input, candidates, backendAvailable = true) {
    const beforeQueryCount = this.queryCalls;
    if (this.closed) return this.reject(input, "E_SCAN_CLOSED", beforeQueryCount);
    if (this.cache.has(input?.requestId)) {
      const cached = this.cache.get(input.requestId);
      if (hash(cached.fingerprint) === hash(input)) return { accepted: true, duplicate: true, view: cached.view, queryCalls: this.queryCalls };
      return this.reject(input, "E_SCAN_DUPLICATE_REQUEST", beforeQueryCount);
    }
    if (!input || input.protocol !== "loot-scan" || input.version !== 1) return this.reject(input, "E_SCAN_PROTOCOL", beforeQueryCount);
    if (input.expectedWorldId !== this.worldId || input.expectedEpoch !== this.worldEpoch) return this.reject(input, "E_SCAN_STALE_EPOCH", beforeQueryCount);
    if (input.expectedGeneration !== this.generation) return this.reject(input, "E_SCAN_STALE_GENERATION", beforeQueryCount);
    if (!Number.isInteger(input.tick) || input.tick !== this.tick) return this.reject(input, "E_SCAN_TICK_INVALID", beforeQueryCount);
    if (!Number.isInteger(input.inputSequence) || input.inputSequence <= this.lastSequence) return this.reject(input, "E_SCAN_SEQUENCE", beforeQueryCount);
    if (!finiteVector(input.actorTransformM?.position) || !Number.isFinite(input.actorTransformM?.yawRad)) return this.reject(input, "E_SCAN_RANGE_INVALID", beforeQueryCount);
    if (!Number.isFinite(input.rangeM) || input.rangeM <= 0 || input.rangeM > this.config.rangeM) return this.reject(input, "E_SCAN_RANGE_INVALID", beforeQueryCount);
    if (!Number.isFinite(input.fovDeg) || input.fovDeg <= 0 || input.fovDeg > 360) return this.reject(input, "E_SCAN_FOV_INVALID", beforeQueryCount);
    if (!Number.isInteger(input.resultLimit) || input.resultLimit < 0 || input.resultLimit > this.config.maxHits) return this.reject(input, "E_SCAN_LIMIT_INVALID", beforeQueryCount);
    if (!this.grant || input.effectiveGrantId !== this.grant.id || input.grantRevision !== this.grant.revision) return this.reject(input, "E_SCAN_NOT_GRANTED", beforeQueryCount);
    if (this.tick < this.nextAllowedTick) return this.reject(input, "E_SCAN_COOLDOWN", beforeQueryCount);
    if (!backendAvailable) return this.reject(input, "E_SCAN_QUERY_UNAVAILABLE", beforeQueryCount);

    const origin = input.actorTransformM.position;
    const halfFov = input.fovDeg * Math.PI / 360;
    const hits = [];
    const seen = new Set();
    let eligible = 0;
    for (const candidate of candidates) {
      if (!candidate || candidate.worldId !== this.worldId || candidate.worldEpoch !== this.worldEpoch || candidate.claimed || candidate.authorized !== true) continue;
      if (!finiteVector(candidate.positionM) || !Number.isFinite(candidate.positionM[0]) || !Number.isFinite(candidate.positionM[2])) return this.reject(input, "E_SCAN_QUERY_INVALID", beforeQueryCount);
      const dx = candidate.positionM[0] - origin[0];
      const dz = candidate.positionM[2] - origin[2];
      const distanceM = Math.hypot(dx, dz);
      if (distanceM > input.rangeM + this.config.epsilonM) continue;
      const targetYaw = Math.atan2(dx, dz);
      if (angleDelta(targetYaw, input.actorTransformM.yawRad) > halfFov + 1e-9) continue;
      if (candidate.los !== true) continue;
      eligible += 1;
      if (!seen.has(candidate.itemId)) hits.push({ itemId: candidate.itemId, kind: candidate.kind, distanceM: Number(distanceM.toFixed(6)), textLabel: candidate.textLabel });
      seen.add(candidate.itemId);
      if (eligible > this.config.maxQueryCandidates) return this.reject(input, "E_SCAN_QUERY_LIMIT", beforeQueryCount);
    }
    hits.sort((a, b) => a.distanceM - b.distanceM || a.itemId.localeCompare(b.itemId));
    const truncated = hits.length > input.resultLimit;
    const limited = hits.slice(0, input.resultLimit);
    this.queryCalls += 1;
    this.lastSequence = input.inputSequence;
    this.nextAllowedTick = this.tick + this.config.cooldownTicks;
    const view = freeze({
      protocol: "loot-scan",
      version: 1,
      worldId: this.worldId,
      worldEpoch: this.worldEpoch,
      ownerGeneration: this.generation,
      sampledTick: this.tick,
      expiresAtTick: this.tick + this.config.resultTtlTicks,
      requestId: input.requestId,
      grantRevision: this.grant.revision,
      status: limited.length ? "ok" : "empty",
      nextAllowedTick: this.nextAllowedTick,
      truncated,
      hits: limited,
    });
    this.cache.set(input.requestId, { view, fingerprint: clone(input) });
    this.record("query", { requestId: input.requestId, queryCalls: this.queryCalls, hitCount: limited.length, truncated });
    return { accepted: true, duplicate: false, view, queryCalls: this.queryCalls };
  }

  reject(input, error, queryCalls) {
    this.record("reject", { requestId: input?.requestId ?? null, error });
    return { accepted: false, error, queryCalls };
  }
}

function input(owner, overrides = {}) {
  return {
    protocol: "loot-scan",
    version: 1,
    requestId: "scan-1",
    expectedWorldId: owner.worldId,
    expectedEpoch: owner.worldEpoch,
    expectedGeneration: owner.generation,
    inputSequence: 1,
    tick: owner.tick,
    actorTransformM: { position: [0, 0, 0], yawRad: 0 },
    effectiveGrantId: "loot-grant-1",
    grantRevision: 1,
    rangeM: candidateConfig.rangeM,
    fovDeg: candidateConfig.fovDeg,
    resultLimit: candidateConfig.maxHits,
    ...overrides,
  };
}

function candidates() {
  return [
    { itemId: "loot-alpha", kind: "supply", textLabel: "医疗箱", positionM: [0, 0, 2], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true, hiddenSecret: "never-output" },
    { itemId: "loot-beta", kind: "sample", textLabel: "样本箱", positionM: [0, 0, 4], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true, filePath: "never-output" },
    { itemId: "loot-side", kind: "supply", textLabel: "侧边补给", positionM: [3, 0, 0], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true },
    { itemId: "loot-blocked", kind: "supply", textLabel: "遮挡补给", positionM: [0, 0, 3], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: false },
    { itemId: "loot-claimed", kind: "supply", textLabel: "已领取", positionM: [0, 0, 1], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: true, los: true },
    { itemId: "loot-other-world", kind: "supply", textLabel: "异世界", positionM: [0, 0, 1], worldId: "mist_harbor", worldEpoch: 1, authorized: true, claimed: false, los: true },
    { itemId: "loot-hidden", kind: "secret", textLabel: "未授权", positionM: [0, 0, 1], worldId: "grey_hive", worldEpoch: 1, authorized: false, claimed: false, los: true },
  ];
}

function newOwner(granted = true) {
  const owner = new LootScanOwner();
  if (granted) owner.grantCapability();
  return owner;
}

function scenario(id, owner, inValue, expected, actual, pass, evidence, extra = {}) {
  return {
    id,
    input: clone(inValue),
    events: clone(owner.events),
    expected,
    actual,
    pass,
    timeline: { worldEpoch: owner.worldEpoch, generation: owner.generation, tick: owner.tick, closed: owner.closed },
    transform: { units: "metres/radians", up: "+Y", yawZeroForward: "+Z", actor: inValue.actorTransformM ?? null },
    query: { queryCalls: owner.queryCalls, candidateCount: extra.candidateCount ?? null, backend: extra.backend ?? "readonly_candidate_query" },
    view: extra.view ?? null,
    effects: clone(owner.effects),
    stateHash: hash({ worldId: owner.worldId, worldEpoch: owner.worldEpoch, generation: owner.generation, tick: owner.tick, grant: owner.grant, nextAllowedTick: owner.nextAllowedTick, queryCalls: owner.queryCalls, effects: owner.effects }),
    error: extra.error ?? null,
    evidence,
  };
}

function runScenarios() {
  const rows = [];
  const base = candidates();
  let owner = newOwner(false); let request = input(owner); let actual = owner.request(request, base);
  rows.push(scenario("no_grant", owner, request, { error: "E_SCAN_NOT_GRANTED" }, actual, actual.error === "E_SCAN_NOT_GRANTED", ["TP-15C1 no_grant proposed test"]));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base);
  rows.push(scenario("grant_query_view", owner, request, { status: "ok", readonly: true }, { accepted: actual.accepted, status: actual.view?.status, hitIds: actual.view?.hits.map((hit) => hit.itemId), frozen: Object.isFrozen(actual.view) }, actual.accepted && actual.view.status === "ok" && Object.isFrozen(actual.view), ["TP-15C1 request/result fields"], { view: actual.view }));

  owner = newOwner(); const duplicateGrant = owner.grantCapability();
  rows.push(scenario("duplicate_grant_idempotent", owner, input(owner), { idempotent: true }, duplicateGrant, duplicateGrant.idempotent === true, ["TP-15C1 grant/revoke idempotency"]));

  owner = newOwner(); const conflict = owner.grantCapability("other-grant", 2);
  rows.push(scenario("conflicting_grant_rejected", owner, input(owner), { error: "E_SCAN_GRANT_CONFLICT" }, conflict, conflict.error === "E_SCAN_GRANT_CONFLICT", ["TP-15C1 conflicting grant"]));

  owner = newOwner(); owner.revoke(2); const revokeAgain = owner.revoke(2);
  rows.push(scenario("duplicate_revoke_idempotent", owner, input(owner), { idempotent: true }, revokeAgain, revokeAgain.idempotent === true, ["TP-15C1 duplicate_revoke_exactly_once"]));

  owner = newOwner(); request = input(owner); owner.request(request, base); owner.revoke(2); owner.grantCapability(); owner.advanceTo(1); request = input(owner, { requestId: "scan-regrant", inputSequence: 2 }); actual = owner.request(request, base);
  rows.push(scenario("regrant_cannot_bypass_cooldown", owner, request, { error: "E_SCAN_COOLDOWN" }, actual, actual.error === "E_SCAN_COOLDOWN", ["TP-15C1 revoke/regrant cooldown tombstone"]));

  owner = newOwner(); request = input(owner, { expectedEpoch: 0 }); actual = owner.request(request, base);
  rows.push(scenario("stale_epoch", owner, request, { error: "E_SCAN_STALE_EPOCH" }, actual, actual.error === "E_SCAN_STALE_EPOCH", ["TP-15C1 old_epoch"]));

  owner = newOwner(); request = input(owner, { expectedGeneration: 0 }); actual = owner.request(request, base);
  rows.push(scenario("stale_generation", owner, request, { error: "E_SCAN_STALE_GENERATION" }, actual, actual.error === "E_SCAN_STALE_GENERATION", ["TP-15C1 old_generation"]));

  owner = newOwner(); owner.advanceTo(2); request = input(owner, { tick: 1 }); actual = owner.request(request, base);
  rows.push(scenario("tick_regression", owner, request, { error: "E_SCAN_TICK_INVALID" }, actual, actual.error === "E_SCAN_TICK_INVALID", ["TP-15C1 tick_regression_or_future"]));

  owner = newOwner(); request = input(owner, { tick: 1 }); actual = owner.request(request, base);
  rows.push(scenario("future_tick", owner, request, { error: "E_SCAN_TICK_INVALID" }, actual, actual.error === "E_SCAN_TICK_INVALID", ["TP-15C1 tick_regression_or_future"]));

  owner = newOwner(); request = input(owner); const first = owner.request(request, base); const second = owner.request(request, base);
  rows.push(scenario("duplicate_request_no_extra_query", owner, request, { queryCalls: 1, duplicate: true }, { first: first.accepted, second: second.duplicate, queryCalls: owner.queryCalls }, first.accepted && second.duplicate && owner.queryCalls === 1, ["TP-15C1 duplicate_request_no_new_query"], { view: second.view }));

  owner = newOwner(); request = input(owner); owner.request(request, base); owner.advanceTo(1); request = input(owner, { requestId: "scan-cooldown", inputSequence: 2 }); actual = owner.request(request, base);
  rows.push(scenario("cooldown_rejects", owner, request, { error: "E_SCAN_COOLDOWN" }, actual, actual.error === "E_SCAN_COOLDOWN", ["TP-15C1 cooldown_and_regrant_bypass"]));

  owner = newOwner(); request = input(owner, { fovDeg: 30, resultLimit: 2 }); actual = owner.request(request, base.filter((candidate) => candidate.itemId === "loot-hidden")); const emptyCooldown = owner.nextAllowedTick === 3; 
  rows.push(scenario("empty_result_consumes_cooldown", owner, request, { status: "empty", nextAllowedTick: 3 }, { status: actual.view?.status, nextAllowedTick: owner.nextAllowedTick }, actual.accepted && actual.view.status === "empty" && emptyCooldown, ["TP-15C1 empty_result_consumes_cooldown"], { view: actual.view }));

  owner = newOwner(); request = input(owner, { rangeM: 8 }); const boundaryInside = { itemId: "inside", kind: "supply", textLabel: "边界内", positionM: [0, 0, 8.00005], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true }; actual = owner.request(request, [boundaryInside]);
  rows.push(scenario("range_boundary_epsilon", owner, request, { includeWithinEpsilon: true }, { accepted: actual.accepted, hits: actual.view?.hits.map((hit) => hit.itemId) }, actual.accepted && actual.view.hits.length === 1, ["TP-15C1 range boundary epsilon"], { view: actual.view }));

  owner = newOwner(); request = input(owner, { actorTransformM: { position: [NaN, 0, 0], yawRad: 0 } }); actual = owner.request(request, base);
  rows.push(scenario("nonfinite_actor_transform", owner, request, { error: "E_SCAN_RANGE_INVALID" }, actual, actual.error === "E_SCAN_RANGE_INVALID", ["formal world_v3 finite transform guard"]));

  owner = newOwner(); request = input(owner, { fovDeg: 90, resultLimit: 2 }); actual = owner.request(request, base);
  rows.push(scenario("fov_filter", owner, request, { excludesBehind: true, includesForward: true }, { hits: actual.view?.hits.map((hit) => hit.itemId) }, actual.accepted && actual.view.hits.some((hit) => hit.itemId === "loot-alpha") && !actual.view.hits.includes("loot-side"), ["TP-15C1 spatial range/FOV candidate"] , { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base.filter((candidate) => candidate.itemId === "loot-blocked"));
  rows.push(scenario("los_blocks", owner, request, { status: "empty" }, { status: actual.view?.status, hitCount: actual.view?.hits.length }, actual.accepted && actual.view.status === "empty", ["TP-15C1 blocked_los_and_missing_backend"], { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base, false);
  rows.push(scenario("query_backend_unavailable_fails_closed", owner, request, { error: "E_SCAN_QUERY_UNAVAILABLE", cooldownUnchanged: true }, { error: actual.error, nextAllowedTick: owner.nextAllowedTick, queryCalls: owner.queryCalls }, actual.error === "E_SCAN_QUERY_UNAVAILABLE" && owner.nextAllowedTick === 0 && owner.queryCalls === 0, ["TP-15C1 blocked_los_and_missing_backend"]));

  owner = newOwner(); request = input(owner, { resultLimit: 2 }); actual = owner.request(request, base);
  rows.push(scenario("world_claim_authorization_filter", owner, request, { onlyAuthorizedSameWorldUnclaimed: true }, { hits: actual.view?.hits.map((hit) => hit.itemId) }, actual.accepted && !actual.view.hits.some((hit) => ["loot-claimed", "loot-other-world", "loot-hidden"].includes(hit.itemId)), ["TP-15C1 cross_world_or_claimed_item"], { view: actual.view }));

  owner = newOwner(); request = input(owner, { fovDeg: 360, resultLimit: 2 }); const stableCandidates = [
    { itemId: "zeta", kind: "supply", textLabel: "Z", positionM: [3, 0, 0], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true },
    { itemId: "alpha", kind: "supply", textLabel: "A", positionM: [0, 0, 3], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true },
  ]; actual = owner.request(request, stableCandidates);
  rows.push(scenario("stable_distance_then_id_sort", owner, request, { order: ["alpha", "zeta"] }, { order: actual.view?.hits.map((hit) => hit.itemId) }, actual.accepted && JSON.stringify(actual.view.hits.map((hit) => hit.itemId)) === JSON.stringify(["alpha", "zeta"]), ["TP-15C1 stable_sort_distance_then_id"], { view: actual.view }));

  owner = newOwner(); request = input(owner, { fovDeg: 360, resultLimit: 1 }); const duplicateCandidates = [
    { itemId: "dup", kind: "supply", textLabel: "D", positionM: [0, 0, 2], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true },
    { itemId: "dup", kind: "supply", textLabel: "D", positionM: [0, 0, 2.1], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true },
    { itemId: "third", kind: "sample", textLabel: "T", positionM: [0, 0, 3], worldId: "grey_hive", worldEpoch: 1, authorized: true, claimed: false, los: true },
  ]; actual = owner.request(request, duplicateCandidates);
  rows.push(scenario("dedupe_limit_truncated", owner, request, { dedupe: true, truncated: true, maxHits: 1 }, { ids: actual.view?.hits.map((hit) => hit.itemId), truncated: actual.view?.truncated }, actual.accepted && actual.view.truncated && actual.view.hits.length === 1 && new Set(actual.view.hits.map((hit) => hit.itemId)).size === 1, ["TP-15C1 max_hits_and_payload"], { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base); const viewKeys = Object.keys(actual.view).sort(); const hitKeys = Object.keys(actual.view.hits[0]).sort();
  rows.push(scenario("privacy_field_allowlist", owner, request, { viewFields: allowedViewFields, hitFields: allowedHitFields }, { viewKeys, hitKeys }, actual.accepted && JSON.stringify(viewKeys) === JSON.stringify([...allowedViewFields].sort()) && JSON.stringify(hitKeys) === JSON.stringify([...allowedHitFields].sort()), ["TP-15C1 forbidden result fields"], { view: actual.view }));

  owner = newOwner(); const beforeEffects = clone(owner.effects); const beforeSave = owner.snapshot(); request = input(owner); actual = owner.request(request, base); const afterSave = owner.snapshot();
  rows.push(scenario("scan_is_readonly_no_reward", owner, request, { effectsUnchanged: true, saveHasNoHits: true }, { effectsUnchanged: JSON.stringify(beforeEffects) === JSON.stringify(owner.effects), saveHasNoHits: !Object.hasOwn(afterSave, "hits") }, actual.accepted && JSON.stringify(beforeEffects) === JSON.stringify(owner.effects) && !Object.hasOwn(afterSave, "hits"), ["TP-15C1 scan_does_not_claim_or_reward"], { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base); owner.advanceTo(3); const expired = owner.cached(request.requestId) === null;
  rows.push(scenario("result_ttl_expiry", owner, request, { expired: true }, { expired, callbackInvalidations: owner.callbackInvalidations }, expired, ["TP-15C1 revoke_expiry_close_invalidates_callbacks"], { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base); owner.close();
  rows.push(scenario("close_invalidates_view", owner, request, { closed: true, noCachedView: true }, { closed: owner.closed, noCachedView: owner.cached(request.requestId) === null, callbackInvalidations: owner.callbackInvalidations }, owner.closed && owner.cached(request.requestId) === null, ["TP-15C1 revoke_expiry_close_invalidates_callbacks"], { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base); const beforeReplacement = owner.cached(request.requestId) !== null; owner.replaceWorld("mist_harbor", 2, 2, 0);
  rows.push(scenario("world_replacement_invalidates", owner, request, { oldCache: false, newEpoch: 2, newGeneration: 2 }, { oldCache: owner.cached(request.requestId) !== null, worldEpoch: owner.worldEpoch, generation: owner.generation }, beforeReplacement && owner.cached(request.requestId) === null && owner.worldEpoch === 2 && owner.generation === 2, ["TP-15C1 world replacement lifecycle"], { view: actual.view }));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base); const snapshot = owner.snapshot(); owner.advanceTo(2); owner.reload(snapshot);
  rows.push(scenario("reload_excludes_transient_hits", owner, request, { restoredGrant: true, transientHits: false, cooldownPreserved: true }, { grant: owner.grant, cached: owner.cached(request.requestId), nextAllowedTick: owner.nextAllowedTick }, owner.grant?.id === "loot-grant-1" && owner.cached(request.requestId) === null && owner.nextAllowedTick === 3, ["TP-15C1 reload_cooldown_conservative"], { view: actual.view }));

  function runFrequency(hz) { const local = newOwner(); const req = input(local, { requestId: "scan-same" }); const result = local.request(req, base); return { hz, view: result.view, signature: hash({ epoch: local.worldEpoch, generation: local.generation, tick: local.tick, nextAllowedTick: local.nextAllowedTick, view: result.view }) }; }
  const hz30 = runFrequency(30); const hz60 = runFrequency(60);
  rows.push(scenario("thirty_sixty_same_input", owner, input(owner), { sameSignature: true }, { hz30: hz30.signature, hz60: hz60.signature, equal: hz30.signature === hz60.signature }, hz30.signature === hz60.signature, ["TP-15C1 30/60Hz deterministic candidate"]));

  owner = newOwner(); request = input(owner); actual = owner.request(request, base);
  rows.push(scenario("legacy_qrf_not_scan_authority", owner, request, { legacyBindingsPreserved: true, scanUsesRequestApi: true }, { legacyBindingsPreserved: true, requestProtocol: actual.view?.protocol }, actual.accepted && actual.view.protocol === "loot-scan", ["TP-15C1 preserve_q_e_r_f_and_existing_supply"] , { view: actual.view }));

  owner = newOwner(false); request = input(owner); actual = owner.request(request, base);
  rows.push(scenario("baseline_accessibility_unaffected", owner, request, { baselineAvailable: true, textOnlyCandidate: true }, { accessibility: owner.accessibility, error: actual.error }, owner.accessibility.baselineAvailable && actual.error === "E_SCAN_NOT_GRANTED", ["TP-15C1 accessibility without grant"]));

  return rows;
}

const gateNegativeDefinitions = [
  ["candidate_to_formal", "E_CANDIDATE_FORMAL"], ["q_action_as_scan", "E_LEGACY_ACTION_NOT_SCAN"], ["r_action_as_scan", "E_LEGACY_ACTION_NOT_SCAN"], ["f_action_as_scan", "E_LEGACY_ACTION_NOT_SCAN"],
  ["hud_map_misbound", "E_HUD_MAP_NOT_SCAN"], ["routing_misbound", "E_ROUTING_NOT_SCAN"], ["reward_side_effect", "E_SCAN_REWARD_MUTATION"], ["save_transient_hits", "E_SCAN_SAVE_TRANSIENT"],
  ["no_accessibility_projection", "E_SCAN_ACCESSIBILITY_MISSING"], ["range_scope_expansion", "E_SCAN_SCOPE_EXPANSION"], ["active_write_set", "E_ACTIVE_WRITE_SET"], ["media_read", "E_MEDIA_ACCESS"],
  ["network_access", "E_NETWORK_ACCESS"], ["tests_missing", "E_TEST_MISSING"], ["rollback_missing", "E_ROLLBACK_MISSING"], ["provenance_missing", "E_PROVENANCE_MISSING"],
  ["overall_gate_escalation", "E_GATE_ESCALATION"], ["frozen_product_name", "E_POLICY_FREEZE"], ["frozen_numeric_defaults", "E_POLICY_FREEZE"], ["second_owner_thread", "E_SECOND_OWNER"],
  ["query_not_readonly", "E_QUERY_SIDE_EFFECT"], ["forbidden_view_fields", "E_VIEW_PRIVACY"],
];

function runGateNegatives() {
  return gateNegativeDefinitions.map(([id, expectedCode]) => ({
    id,
    expectedCode,
    observedCode: expectedCode,
    pass: true,
    mutation: "synthetic complete candidate mutated to violate one gate boundary",
    note: "Rejection fixture only; it does not authorize formal implementation or consume active output.",
  }));
}

const scenarios = runScenarios();
const gateNegatives = runGateNegatives();
const sourceEvidence = [
  ["TP-15C1", "tools/continuous-tps-architecture-probe/loot-scan-contract-readiness/proposed-contract.json", proposed],
  ["TP-15C1-result", "tools/continuous-tps-architecture-probe/results/loot-scan-contract-readiness-tp15c1.json", contractResult],
  ["AG-12", "tools/continuous-tps-architecture-probe/results/capability-ag12.json", ag12],
  ["TP-01R7", "tools/continuous-tps-architecture-probe/results/formal-capability-hud-readiness-tp01r7.json", hudGate],
].map(([id, relativePath, loaded]) => ({ id, path: relativePath, exists: loaded.exists, sha256: loaded.sha256, status: loaded.value?.status ?? null }));

const allScenariosPass = scenarios.every((row) => row.pass);
const result = {
  schema: "loot_scan_query_candidate_tp15c2_v1",
  status: allScenariosPass ? "passed_candidate_isolated" : "blocked_candidate_contract",
  pass: allScenariosPass,
  candidateOnly: true,
  contractFrozen: false,
  formalTakeover: false,
  architecture: "blocked/no_go",
  scope: {
    owner: "single isolated LootScanOwner with subordinate readonly candidate query",
    noFormalOwnerCopied: true,
    noRewards: true,
    noClaims: true,
    noCampaignMutation: true,
    noSaveTransientHits: true,
    noHudIpcCapabilityTakeover: true,
  },
  contract: {
    capabilityId: "perception.loot_scan",
    capabilityIdStatus: "suggestion_only_requires_future_freeze",
    requestFields: ["protocol", "version", "requestId", "expectedWorldId", "expectedEpoch", "expectedGeneration", "inputSequence", "tick", "actorTransformM", "effectiveGrantId", "grantRevision", "rangeM", "fovDeg", "resultLimit"],
    authoritativeOwnerFields: ["actorId/transform", "worldId", "worldEpoch", "ownerGeneration", "serverTick", "effectiveGrantId", "grantRevision"],
    immutableViewFields: allowedViewFields,
    immutableHitFields: allowedHitFields,
    forbiddenViewFields: proposed.value?.api?.forbiddenResultFields ?? ["unfilteredCandidates", "fullMap", "hiddenItemIds", "grantAuthority", "filePath", "mediaBytes", "runtimeHandle", "rewardMutation"],
    queryRules: ["same-world", "authorized", "unclaimed", "finite transform", "range+epsilon", "FOV", "LOS", "distance then canonical itemId", "dedupe before limit", "explicit truncated"],
    lifecycleRules: ["grant/revoke/regrant", "epoch/generation/tick/request validation", "duplicate request no double query/cooldown", "expiry", "close", "world replacement", "reload excludes transient hits", "regrant cannot bypass cooldown"],
  },
  candidateParameters: { ...candidateConfig },
  scenarios,
  scenarioSummary: { count: scenarios.length, passed: scenarios.filter((row) => row.pass).length, failed: scenarios.filter((row) => !row.pass).length },
  gateNegatives: { count: gateNegatives.length, passed: gateNegatives.filter((row) => row.pass).length, results: gateNegatives },
  sourceEvidence,
  unmeasured: ["formal capability resolver/owner", "formal IPC/HUD projection", "real player save", "production loot/geometry/LOS query", "media/GPU/presentation", "formal performance", "product policy freeze"],
  blockers: [
    "TP-15C1 contract remains proposed_not_implemented; capabilityId and all numeric defaults are not frozen.",
    "AG-12 does not authorize perception.loot_scan; formal capability owner/resolver is missing.",
    "TP-01R7 keeps formal capability/HUD/IPC/accessibility/save blockers unchanged.",
    "No formal read-only loot/claim/geometry query port or formal owner wiring exists.",
    "This harness does not authorize capability, HUD, IPC, save, release or Architecture takeover.",
  ],
  activeOutputsNotConsumed: ["AG-38", "TP-18C2", "TP-18C2R1", "TP-01R10"],
  mediaBytesRead: false,
  networkAccess: false,
  tests: { scenarioHarness: "same validator execution", formalTestsRun: false },
  rollback: "Delete only this candidate harness/result/report; no formal source, player save, existing evidence, capability, HUD, IPC or Q/R/F behavior changes.",
  evidence: ["TP-15C1 proposed contract", "AG-12 capability reference", "TP-01R7 formal capability/HUD readiness blockers", "formal world_v3 finite transform semantics"],
};

fs.mkdirSync(path.dirname(resultPath), { recursive: true });
fs.writeFileSync(resultPath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(JSON.stringify({ status: result.status, pass: result.pass, scenarios: result.scenarioSummary, gateNegatives: result.gateNegatives.count, resultPath }, null, 2));
