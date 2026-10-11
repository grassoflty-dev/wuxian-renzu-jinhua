import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const RESULT_PATH = path.join(HERE, "..", "results", "capability-ag12.json");
const PROTOCOL = "continuous-capability";
const VERSION = 1;
const SOURCE_PRIORITY = Object.freeze({ system: 100, progression: 80, equipment: 60, bloodline: 50, world: 40, temporary: 20 });
const TIER_RANK = Object.freeze({ S: 5, A: 4, B: 3, C: 2, D: 1 });
const ACCESSIBILITY = Object.freeze({
  "ui.accessibility": { captions: true, scalableText: true, remappableControls: true, highContrast: true },
  "controls.basic": { move: true, attack: true, interact: true, pause: true },
});

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function deepFreeze(value) {
  if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
  Object.freeze(value);
  Object.values(value).forEach(deepFreeze);
  return value;
}

function stableGrantKey(grant) {
  return JSON.stringify({
    capabilityId: grant.capabilityId,
    grantId: grant.grantId,
    source: grant.source,
    tier: grant.tier,
    parameters: grant.parameters,
    scope: grant.scope,
    worldId: grant.worldId ?? null,
    expiresAtTick: grant.expiresAtTick ?? null,
  });
}

function validateParameters(capabilityId, parameters) {
  if (!parameters || typeof parameters !== "object" || Array.isArray(parameters)) return "E_PARAMETERS_OBJECT";
  const finite = (key) => Number.isFinite(Number(parameters[key]));
  if (capabilityId === "perception.rear_view") {
    if (!finite("distanceM") || Number(parameters.distanceM) < 4 || Number(parameters.distanceM) > 30) return "E_REAR_DISTANCE";
    if (!finite("yawFovDeg") || Number(parameters.yawFovDeg) < 90 || Number(parameters.yawFovDeg) > 200) return "E_REAR_FOV";
    if (!["low", "medium", "high"].includes(parameters.resolutionTier)) return "E_REAR_RESOLUTION";
    return null;
  }
  if (capabilityId === "map.local") {
    if (!finite("radiusM") || Number(parameters.radiusM) < 2 || Number(parameters.radiusM) > 50) return "E_MAP_RADIUS";
    if (parameters.mode !== "room") return "E_MAP_MODE";
    return null;
  }
  if (capabilityId === "mobility.dash") {
    if (!finite("speedMps") || Number(parameters.speedMps) <= 0 || Number(parameters.speedMps) > 20) return "E_DASH_SPEED";
    if (!finite("cooldownMs") || Number(parameters.cooldownMs) < 100 || Number(parameters.cooldownMs) > 10000) return "E_DASH_COOLDOWN";
    return null;
  }
  return "E_UNKNOWN_CAPABILITY";
}

function normalizeGrant(input, resolver) {
  const grant = clone(input);
  if (!grant || typeof grant !== "object" || Array.isArray(grant)) throw new Error("E_GRANT_OBJECT");
  if (!grant.grantId || typeof grant.grantId !== "string") throw new Error("E_GRANT_ID");
  if (!grant.capabilityId || typeof grant.capabilityId !== "string") throw new Error("E_CAPABILITY_ID");
  if (!Object.prototype.hasOwnProperty.call(SOURCE_PRIORITY, grant.source)) throw new Error("E_SOURCE");
  if (!Object.prototype.hasOwnProperty.call(TIER_RANK, grant.tier)) throw new Error("E_TIER");
  if (!["persistent", "world", "run", "session"].includes(grant.scope)) throw new Error("E_SCOPE");
  const parameterError = validateParameters(grant.capabilityId, grant.parameters);
  if (parameterError) throw new Error(parameterError);
  if (grant.scope === "world" || grant.scope === "run") {
    if (grant.worldId !== resolver.worldId) throw new Error("E_WORLD_SCOPE");
    if (grant.worldEpoch !== undefined && grant.worldEpoch !== resolver.worldEpoch) throw new Error("E_WORLD_EPOCH");
  } else if (grant.worldId !== undefined || grant.worldEpoch !== undefined) {
    throw new Error("E_GLOBAL_SCOPE_WORLD_FIELDS");
  }
  if (grant.expiresAtTick !== undefined && (!Number.isInteger(grant.expiresAtTick) || grant.expiresAtTick <= resolver.tick)) throw new Error("E_EXPIRED_GRANT");
  grant.schemaVersion = VERSION;
  return grant;
}

function compareGrant(a, b) {
  return (SOURCE_PRIORITY[b.source] - SOURCE_PRIORITY[a.source])
    || (TIER_RANK[b.tier] - TIER_RANK[a.tier])
    || String(a.grantId).localeCompare(String(b.grantId));
}

class CapabilityResolver {
  constructor(worldId = "grey_hive") {
    this.worldId = worldId;
    this.worldEpoch = 1;
    this.tick = 0;
    this.grants = new Map();
    this.activePresentations = new Map();
    this.releaseEvents = [];
  }

  grant(input) {
    const grant = normalizeGrant(input, this);
    const old = this.grants.get(grant.grantId);
    if (old && stableGrantKey(old) === stableGrantKey(grant)) return { status: "idempotent", grantId: grant.grantId };
    this.grants.set(grant.grantId, grant);
    return { status: old ? "replaced" : "granted", grantId: grant.grantId };
  }

  revoke({ grantId, capabilityId, source } = {}) {
    const removed = [];
    for (const [id, grant] of this.grants.entries()) {
      if ((grantId && id !== grantId) || (capabilityId && grant.capabilityId !== capabilityId) || (source && grant.source !== source)) continue;
      this.grants.delete(id);
      removed.push(id);
    }
    for (const id of removed) this.releasePresentation(id, "grant_revoked");
    return { removed: removed.sort() };
  }

  tickTo(nextTick) {
    if (!Number.isInteger(nextTick) || nextTick < this.tick) throw new Error("E_TICK_ORDER");
    this.tick = nextTick;
    const expired = [];
    for (const [id, grant] of this.grants.entries()) {
      if (grant.expiresAtTick !== undefined && grant.expiresAtTick <= this.tick) {
        this.grants.delete(id);
        expired.push(id);
      }
    }
    expired.forEach((id) => this.releasePresentation(id, "grant_expired"));
    return expired.sort();
  }

  deathRespawn() {
    const removed = [];
    for (const [id, grant] of this.grants.entries()) {
      if (grant.scope !== "run") continue;
      this.grants.delete(id);
      removed.push(id);
    }
    this.worldEpoch += 1;
    removed.forEach((id) => this.releasePresentation(id, "death_respawn"));
    return { worldEpoch: this.worldEpoch, removed: removed.sort() };
  }

  switchWorld(worldId) {
    if (!worldId || worldId === this.worldId) throw new Error("E_WORLD_SWITCH");
    for (const [id, grant] of this.grants.entries()) {
      if (grant.scope === "world" || grant.scope === "run") {
        this.grants.delete(id);
        this.releasePresentation(id, "world_exit");
      }
    }
    for (const [id] of this.activePresentations.entries()) this.releasePresentation(id, "world_exit");
    this.worldId = worldId;
    this.worldEpoch += 1;
    return { worldId: this.worldId, worldEpoch: this.worldEpoch };
  }

  effective(capabilityId = null) {
    const candidates = [...this.grants.values()].filter((grant) => {
      if (capabilityId && grant.capabilityId !== capabilityId) return false;
      if (grant.expiresAtTick !== undefined && grant.expiresAtTick <= this.tick) return false;
      if ((grant.scope === "world" || grant.scope === "run") && (grant.worldId !== this.worldId || grant.worldEpoch > this.worldEpoch)) return false;
      return true;
    });
    const byCapability = new Map();
    for (const grant of candidates.sort(compareGrant)) {
      if (!byCapability.has(grant.capabilityId)) byCapability.set(grant.capabilityId, grant);
    }
    return [...byCapability.values()].sort((a, b) => a.capabilityId.localeCompare(b.capabilityId));
  }

  requestPresentation(capabilityId) {
    const grant = this.effective(capabilityId)[0];
    if (!grant) return { status: "denied", errorCode: "E_CAPABILITY_NOT_EFFECTIVE" };
    if (capabilityId !== "perception.rear_view") return { status: "allowed", capabilityId, parameters: clone(grant.parameters) };
    const resourceToken = `presentation:${this.worldEpoch}:${grant.grantId}`;
    this.activePresentations.set(grant.grantId, { capabilityId, worldEpoch: this.worldEpoch, resourceToken });
    return { status: "create_requested", action: "create_second_view", capabilityId, worldEpoch: this.worldEpoch, parameters: clone(grant.parameters), resourceToken };
  }

  releasePresentation(grantId, reason) {
    if (!this.activePresentations.has(grantId)) return false;
    const presentation = this.activePresentations.get(grantId);
    this.activePresentations.delete(grantId);
    this.releaseEvents.push({ action: "release_second_view", grantId, capabilityId: presentation.capabilityId, worldEpoch: presentation.worldEpoch, reason });
    return true;
  }

  view() {
    const effective = this.effective().map((grant) => ({
      capabilityId: grant.capabilityId,
      grantId: grant.grantId,
      source: grant.source,
      tier: grant.tier,
      scope: grant.scope,
      parameters: clone(grant.parameters),
    }));
    return deepFreeze({
      protocol: PROTOCOL,
      version: VERSION,
      worldId: this.worldId,
      worldEpoch: this.worldEpoch,
      tick: this.tick,
      effective,
      baseline: clone(ACCESSIBILITY),
    });
  }

  save() {
    return {
      schemaVersion: VERSION,
      worldId: this.worldId,
      grants: [...this.grants.values()]
        .filter((grant) => grant.scope === "persistent" || (grant.scope === "world" && grant.worldId === this.worldId))
        .map((grant) => clone(grant)),
    };
  }

  static fromSave(save) {
    if (!save || save.schemaVersion !== VERSION || !Array.isArray(save.grants)) throw new Error("E_SAVE_SCHEMA");
    const resolver = new CapabilityResolver(save.worldId);
    for (const grant of save.grants) resolver.grant(grant);
    return resolver;
  }
}

function check(name, inputs, expected, actual, pass, notes = null) {
  return { name, inputs, expected, actual, pass: Boolean(pass), notes };
}

function rearViewLifecycle() {
  const resolver = new CapabilityResolver();
  const before = resolver.view();
  const grant = { grantId: "grant-rear-progression", capabilityId: "perception.rear_view", source: "progression", tier: "A", scope: "persistent", parameters: { distanceM: 12, yawFovDeg: 140, resolutionTier: "medium" } };
  const granted = resolver.grant(grant);
  const requested = resolver.requestPresentation("perception.rear_view");
  const revoked = resolver.revoke({ grantId: grant.grantId });
  const afterRevoke = resolver.view();
  const release = resolver.releaseEvents.at(-1) ?? null;
  const regranted = resolver.grant({ ...grant, parameters: { ...grant.parameters, distanceM: 10 } });
  const requestedAgain = resolver.requestPresentation("perception.rear_view");
  return check("rear_view_no_grant_grant_view_revoke_release_regrant", { steps: ["no", "grant", "presentation_request", "revoke", "release", "grant_again"] }, { before: false, firstRequest: "create_requested", afterRevoke: false, release: "release_second_view", secondRequest: "create_requested" }, { before: before.effective.some((x) => x.capabilityId === "perception.rear_view"), granted, requested: requested.status, afterRevoke: afterRevoke.effective.some((x) => x.capabilityId === "perception.rear_view"), revoked, release, regranted, requestedAgain: requestedAgain.status }, !before.effective.some((x) => x.capabilityId === "perception.rear_view") && requested.status === "create_requested" && !afterRevoke.effective.some((x) => x.capabilityId === "perception.rear_view") && release?.action === "release_second_view" && requestedAgain.status === "create_requested");
}

function idempotentGrant() {
  const resolver = new CapabilityResolver();
  const grant = { grantId: "grant-dash-equipment", capabilityId: "mobility.dash", source: "equipment", tier: "C", scope: "run", worldId: "grey_hive", worldEpoch: 1, parameters: { speedMps: 8, cooldownMs: 900 } };
  const first = resolver.grant(grant);
  const second = resolver.grant(grant);
  return check("repeated_grant_is_idempotent", { grantId: grant.grantId }, { first: "granted", second: "idempotent", effectiveCount: 1, runScopeExcludedFromSave: true }, { first, second, effectiveCount: resolver.effective("mobility.dash").length, savedCount: resolver.save().grants.length }, first.status === "granted" && second.status === "idempotent" && resolver.effective("mobility.dash").length === 1 && resolver.save().grants.length === 0);
}

function multiSourcePriority() {
  const resolver = new CapabilityResolver();
  const base = { capabilityId: "perception.rear_view", tier: "C", scope: "persistent", parameters: { distanceM: 8, yawFovDeg: 120, resolutionTier: "low" } };
  const low = resolver.grant({ ...base, grantId: "rear-equipment", source: "equipment" });
  const high = resolver.grant({ ...base, grantId: "rear-progression", source: "progression", tier: "A", parameters: { distanceM: 16, yawFovDeg: 160, resolutionTier: "high" } });
  const winner = resolver.view().effective.find((x) => x.capabilityId === "perception.rear_view");
  const afterHighRevoke = resolver.revoke({ grantId: "rear-progression" });
  const fallback = resolver.view().effective.find((x) => x.capabilityId === "perception.rear_view");
  return check("multi_source_priority_and_fallback", { sourcePriority: SOURCE_PRIORITY, grants: ["equipment", "progression"] }, { winner: "rear-progression", fallback: "rear-equipment" }, { low, high, winner, afterHighRevoke, fallback }, winner?.grantId === "rear-progression" && fallback?.grantId === "rear-equipment");
}

function scopeEpochRules() {
  const resolver = new CapabilityResolver();
  resolver.grant({ grantId: "rear-persistent", capabilityId: "perception.rear_view", source: "progression", tier: "A", scope: "persistent", parameters: { distanceM: 10, yawFovDeg: 130, resolutionTier: "medium" } });
  resolver.grant({ grantId: "map-world", capabilityId: "map.local", source: "world", tier: "C", scope: "world", worldId: "grey_hive", worldEpoch: 1, parameters: { radiusM: 12, mode: "room" } });
  resolver.grant({ grantId: "dash-run", capabilityId: "mobility.dash", source: "temporary", tier: "D", scope: "run", worldId: "grey_hive", worldEpoch: 1, parameters: { speedMps: 6, cooldownMs: 700 } });
  const before = resolver.view();
  const death = resolver.deathRespawn();
  const afterDeath = resolver.view();
  const switched = resolver.switchWorld("mist_harbor");
  const afterWorld = resolver.view();
  return check("scope_death_respawn_and_world_epoch_isolation", { scopes: ["persistent", "world", "run"], initialWorld: "grey_hive", nextWorld: "mist_harbor" }, { death: ["perception.rear_view", "map.local"], worldSwitch: ["perception.rear_view"], epochIncreases: true }, { before: before.effective.map((x) => x.capabilityId), death, afterDeath: afterDeath.effective.map((x) => x.capabilityId), switched, afterWorld: afterWorld.effective.map((x) => x.capabilityId), releaseEvents: resolver.releaseEvents }, afterDeath.effective.some((x) => x.capabilityId === "perception.rear_view") && !afterDeath.effective.some((x) => x.capabilityId === "mobility.dash") && afterDeath.effective.some((x) => x.capabilityId === "map.local") && afterWorld.effective.length === 1 && afterWorld.effective[0].capabilityId === "perception.rear_view" && switched.worldEpoch > death.worldEpoch);
}

function invalidAndExpiry() {
  const resolver = new CapabilityResolver();
  const errors = [];
  for (const grant of [
    { grantId: "bad-rear", capabilityId: "perception.rear_view", source: "progression", tier: "A", scope: "persistent", parameters: { distanceM: 2, yawFovDeg: 120, resolutionTier: "medium" } },
    { grantId: "bad-dash", capabilityId: "mobility.dash", source: "temporary", tier: "D", scope: "run", worldId: "wrong", worldEpoch: 1, parameters: { speedMps: 6, cooldownMs: 700 } },
  ]) { try { resolver.grant(grant); } catch (error) { errors.push(error.message); } }
  resolver.grant({ grantId: "expiring-map", capabilityId: "map.local", source: "temporary", tier: "D", scope: "session", expiresAtTick: 5, parameters: { radiusM: 8, mode: "room" } });
  const before = resolver.view();
  const expired = resolver.tickTo(5);
  const after = resolver.view();
  return check("invalid_scope_parameters_and_expiry", { invalid: ["rear_distance", "world_id"], expiresAtTick: 5 }, { errors: ["E_REAR_DISTANCE", "E_WORLD_SCOPE"], expired: ["expiring-map"], after: false }, { errors, before: before.effective.map((x) => x.capabilityId), expired, after: after.effective.map((x) => x.capabilityId) }, errors.join(",") === "E_REAR_DISTANCE,E_WORLD_SCOPE" && expired.includes("expiring-map") && !after.effective.some((x) => x.capabilityId === "map.local"));
}

function saveRestoreAndNoGpuHandles() {
  const resolver = new CapabilityResolver();
  resolver.grant({ grantId: "rear-persistent", capabilityId: "perception.rear_view", source: "progression", tier: "A", scope: "persistent", parameters: { distanceM: 12, yawFovDeg: 140, resolutionTier: "medium" } });
  resolver.grant({ grantId: "map-world", capabilityId: "map.local", source: "world", tier: "C", scope: "world", worldId: "grey_hive", worldEpoch: 1, parameters: { radiusM: 10, mode: "room" } });
  const saved = resolver.save();
  const serialized = JSON.stringify(saved);
  const restored = CapabilityResolver.fromSave(JSON.parse(serialized));
  const originalIds = resolver.view().effective.map((x) => x.capabilityId);
  const restoredIds = restored.view().effective.map((x) => x.capabilityId);
  const noGpuHandle = !serialized.includes("renderTexture") && !serialized.includes("mesh") && !serialized.includes("resourceToken") && !serialized.includes("WebGL");
  return check("save_restore_recompute_without_gpu_handles", { savedFields: Object.keys(saved), serializedBytes: serialized.length }, { idsEqual: true, noGpuHandle: true, worldEpoch: 1 }, { originalIds, restoredIds, saved, noGpuHandle, restoredView: restored.view() }, JSON.stringify(originalIds) === JSON.stringify(restoredIds) && noGpuHandle && restored.view().worldEpoch === 1);
}

function mapAndDashRules() {
  const resolver = new CapabilityResolver();
  const map = resolver.grant({ grantId: "map-grant", capabilityId: "map.local", source: "progression", tier: "B", scope: "persistent", parameters: { radiusM: 18, mode: "room" } });
  const dash = resolver.grant({ grantId: "dash-grant", capabilityId: "mobility.dash", source: "progression", tier: "B", scope: "persistent", parameters: { speedMps: 10, cooldownMs: 650 } });
  const view = resolver.view();
  const mapRequest = resolver.requestPresentation("map.local");
  const dashRequest = resolver.requestPresentation("mobility.dash");
  return check("map_local_and_mobility_dash_rules", { capabilities: ["map.local", "mobility.dash"] }, { map: "allowed", dash: "allowed" }, { map, dash, view, mapRequest, dashRequest }, mapRequest.status === "allowed" && dashRequest.status === "allowed" && view.effective.some((x) => x.capabilityId === "map.local") && view.effective.some((x) => x.capabilityId === "mobility.dash"));
}

function accessibilityAlwaysAvailable() {
  const resolver = new CapabilityResolver();
  const before = resolver.view();
  const attempted = resolver.revoke({ capabilityId: "ui.accessibility" });
  const after = resolver.view();
  return check("accessibility_and_basic_controls_are_not_capability_locked", { attemptedRevoke: "ui.accessibility" }, { captions: true, controls: true }, { before: before.baseline, attempted, after: after.baseline }, before.baseline["ui.accessibility"].captions === true && after.baseline["ui.accessibility"].captions === true && after.baseline["controls.basic"].move === true);
}

function sourceAudit() {
  return [
    { file: "server-rs/src/state.rs", lines: "156-320", finding: "GameState is the older progression authority: gene_lock/gene_stage, bloodline, skills, equipped_skills, equipment, tier/proficiency maps and save_version are serde-persisted; no CapabilityId/Grant/CapabilityView field exists." },
    { file: "server-rs/src/state.rs", lines: "481-518", finding: "migrate_save performs the idempotent gene_lock→gene_stage migration and keeps skill proficiency fields as serde-default; no capability grant resolver is present." },
    { file: "server-rs/src/gene.rs", lines: "70-107", finding: "gene_stage_of/set_gene_stage and power_maxed/saint_ready are authoritative helpers over GameState, but only expose progression checks and combat-oriented stages." },
    { file: "server-rs/src/bloodlines.rs", lines: "223-230", finding: "bloodline_def/bloodline_of resolve one active bloodline; passive effects are static data, not a versioned capability view." },
    { file: "server-rs/src/power.rs", lines: "30-95", finding: "power() derives combat strength from hp, weapon, gene stage, cultivation, equipment and skill count; it is not a capability grant system." },
    { file: "server-rs/src/campaign.rs", lines: "270-321", finding: "CampaignState persists exchanges/skills/objectives plus private flags and run fields; current Campaign view exposes skill readiness and exchange ownership." },
    { file: "server-rs/src/campaign.rs", lines: "1600-1710", finding: "campaign::view serializes exchange ownership, skills, objectives, map and player data for UI; no authoritative capability field is emitted." },
    { file: "server-rs/src/campaign.rs", lines: "2138-2248", finding: "skill actions are slot/ownership/energy/cooldown gated in Rust; dash is a discrete action elsewhere, with no rear_view or capability grant path." },
    { file: "server-rs/src/campaign_store.rs", lines: "90-176,350-430", finding: "campaign store writes v2 JSON through a temp file and validates a fixed schema; current store has no CapabilityView or GPU-resource boundary." },
    { file: "server-rs/release-ui/js/app.js", lines: "311-420", finding: "UI renders hub skills and three skill cards from state.skills/state.upgrades; ownership/readiness is displayed, not authority." },
    { file: "server-rs/release-ui/js/app.js", lines: "640-735,810-835", finding: "UI sends dash/skill actions and pause clears held/queued input; no capability resolver, rear_view request, or grant/revoke event exists." },
    { file: "server-rs/release-ui/js/tps-renderer.js", lines: "1185-1225", finding: "TPS renderer exposes one third-person follow camera and cameraBehind diagnostic; no second view RenderTexture/mesh resource contract exists." },
    { file: "server-rs/release-ui/js/tps-renderer.js", lines: "1238-1275", finding: "renderer.dispose tears down scene/engine/camera/materials; this is a renderer lifecycle, not an authoritative capability release protocol." },
  ];
}

function main() {
  const scenarios = [
    rearViewLifecycle(),
    idempotentGrant(),
    multiSourcePriority(),
    scopeEpochRules(),
    invalidAndExpiry(),
    saveRestoreAndNoGpuHandles(),
    mapAndDashRules(),
    accessibilityAlwaysAvailable(),
  ];
  const allPass = scenarios.every((scenario) => scenario.pass);
  const report = {
    schema: "continuous_capability_probe_ag12_v1",
    status: allPass ? "isolated_capability_probe_passed_with_formal_gaps" : "isolated_capability_probe_failed",
    pass: allPass,
    scope: "isolated versioned capability contract/state probe; no formal runtime capability system or GPU second view",
    realRuntimeModified: false,
    environment: { node: process.version, platform: process.platform, arch: process.arch, osRelease: os.release() },
    currentImplementation: { capabilityId: false, grantResolver: false, authoritativeCapabilityView: false, rearView: false, secondViewRenderTexture: false },
    contract: {
      protocol: PROTOCOL,
      version: VERSION,
      capabilityIds: {
        "perception.rear_view": { parameters: ["distanceM", "yawFovDeg", "resolutionTier"], presentation: "create_second_view request only; GPU handles never enter SaveV3" },
        "map.local": { parameters: ["radiusM", "mode"], presentation: "authoritative map reveal request" },
        "mobility.dash": { parameters: ["speedMps", "cooldownMs"], presentation: "authoritative movement rule" },
      },
      grant: { fields: ["schemaVersion", "grantId", "capabilityId", "source", "tier", "parameters", "scope", "worldId?", "worldEpoch?", "expiresAtTick?"], sourcePriority: SOURCE_PRIORITY, scopes: ["persistent", "world", "run", "session"], tieBreak: "source priority desc, tier desc, grantId lexicographic" },
      resolver: { rule: "recompute effective set from semantic grants after every grant/revoke/tick/death/world switch/load", duplicateGrant: "same grantId and payload is idempotent", invalid: "reject with deterministic error code", epoch: "world/run grants require current worldId and worldEpoch; world switch increments worldEpoch and removes world/run grants" },
      capabilityView: { fields: ["protocol", "version", "worldId", "worldEpoch", "tick", "effective", "baseline"], readonly: true, gpuHandles: false, baselineAlwaysOn: ["ui.accessibility", "controls.basic"] },
      persistence: { saveSemanticGrantsOnly: true, includedScopes: ["persistent", "current-world"], excludedRuntimeHandles: ["renderTexture", "mesh", "resourceToken", "WebGL"] },
    },
    sourceAudit: sourceAudit(),
    scenarios,
    checks: { scenarioCount: scenarios.length, passed: scenarios.filter((scenario) => scenario.pass).length, allPass, noGpuHandlesInSave: scenarios.find((scenario) => scenario.name === "save_restore_recompute_without_gpu_handles")?.pass === true, accessibilityAlwaysOn: scenarios.find((scenario) => scenario.name === "accessibility_and_basic_controls_are_not_capability_locked")?.pass === true },
    formalGaps: [
      "No formal Rust/Serde CapabilityId, Grant, Resolver, authoritative set or CapabilityView exists yet.",
      "rear_view presentation request is only an isolated intent; no Babylon RenderTexture/second camera/culling implementation was added.",
      "TP-13 must verify real grant/revoke, resource release, world/death lifecycle, 50 enable/disable cycles, occlusion/distance/camera feel and offline load.",
      "AG-11 SaveV3 currently has no capability section; TP-01/TP-03 must decide the final semantic grant persistence and migration boundary.",
      "Accessibility captions, scalable text, remappable controls and basic controls remain baseline and must not be gated by a capability."
    ],
  };
  fs.mkdirSync(path.dirname(RESULT_PATH), { recursive: true });
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(report, null, 2)}\n`, "utf8");
  console.log(JSON.stringify({ status: report.status, scenarios: scenarios.length, passed: report.checks.passed, pass: report.pass }));
  if (!allPass) process.exitCode = 1;
}

main();
