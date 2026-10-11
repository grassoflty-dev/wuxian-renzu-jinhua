"use strict";

const PROTOCOL = "continuous-capability";
const VERSION = 1;
const CAPABILITY_ID = "perception.rear_view";
const RESOLUTION = { low: 64, medium: 128, high: 256 };

function error(code, message) {
  const e = new Error(message || code);
  e.code = code;
  return e;
}

function finite(value, label) {
  if (typeof value !== "number" || !Number.isFinite(value)) throw error("E_NONFINITE", `${label} must be finite`);
  return value;
}

function object(value, label) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw error("E_SCHEMA", `${label} must be an object`);
  return value;
}

function playerTransform(value) {
  const transform = object(value, "playerTransform");
  const p = object(transform.positionM, "playerTransform.positionM");
  return { position: { x: finite(p.xM, "positionM.xM"), y: finite(p.yM, "positionM.yM"), z: finite(p.zM, "positionM.zM") }, yawRad: finite(transform.yawRad, "yawRad") };
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function normalizeEffective(view) {
  const entries = Array.isArray(view.effective) ? view.effective : [];
  return entries.filter((entry) => entry && entry.capabilityId === CAPABILITY_ID).map((entry) => {
    const parameters = object(entry.parameters, "rear_view.parameters");
    const distanceM = finite(parameters.distanceM, "distanceM");
    const yawFovDeg = finite(parameters.yawFovDeg, "yawFovDeg");
    if (distanceM <= 0 || yawFovDeg <= 0 || yawFovDeg >= 180) throw error("E_PARAMETERS", "rear_view distance/FOV out of range");
    if (!Object.prototype.hasOwnProperty.call(RESOLUTION, parameters.resolutionTier)) throw error("E_PARAMETERS", "unsupported resolutionTier");
    const grantId = typeof entry.grantId === "string" && entry.grantId.length > 0 ? entry.grantId : `${entry.source || "unknown"}:${CAPABILITY_ID}`;
    return { grantId, source: entry.source || "unknown", parameters: { distanceM, yawFovDeg, resolutionTier: parameters.resolutionTier } };
  });
}

function validateView(view, expectedEpoch) {
  object(view, "capabilityView");
  if (view.protocol !== PROTOCOL || view.version !== VERSION) throw error("E_SCHEMA", "CapabilityView protocol/version mismatch");
  if (!Number.isInteger(view.worldEpoch) || view.worldEpoch < 0) throw error("E_SCHEMA", "worldEpoch must be a non-negative integer");
  if (expectedEpoch !== undefined && view.worldEpoch !== expectedEpoch) throw error("E_EPOCH", "CapabilityView epoch mismatch");
  if (typeof view.worldId !== "string" || view.worldId.length === 0) throw error("E_SCHEMA", "worldId must be non-empty");
  return normalizeEffective(view);
}

function rearDirection(BABYLON, yawRad, distanceM) {
  const x = Math.sin(yawRad) * distanceM;
  const z = Math.cos(yawRad) * distanceM;
  return new BABYLON.Vector3(x, 0, z);
}

function createRearViewAdapter({ BABYLON, scene, cameraName = "cap-rear-view-camera", textureName = "cap-rear-view-rtt" } = {}) {
  if (!BABYLON || typeof BABYLON.FreeCamera !== "function" || typeof BABYLON.RenderTargetTexture !== "function") throw error("E_BABYLON_UNAVAILABLE", "Babylon FreeCamera and RenderTargetTexture are required");
  if (!scene || typeof scene.getEngine !== "function") throw error("E_SCENE_UNAVAILABLE", "a real Babylon Scene is required");
  let closed = false;
  let currentEpoch = null;
  let generation = 0;
  let resource = null;
  let disposedCount = 0;
  let renderCount = 0;
  let lastPlayer = null;
  let sources = new Map();

  function disposeResource(reason) {
    if (!resource) return false;
    const old = resource;
    resource = null;
    if (old.texture && Array.isArray(scene.customRenderTargets)) {
      const targetIndex = scene.customRenderTargets.indexOf(old.texture);
      if (targetIndex >= 0) scene.customRenderTargets.splice(targetIndex, 1);
    }
    if (old.texture && typeof old.texture.dispose === "function") old.texture.dispose();
    if (old.camera && typeof old.camera.dispose === "function") old.camera.dispose();
    disposedCount += 1;
    return reason;
  }

  function createResource(params, player) {
    generation += 1;
    const camera = new BABYLON.FreeCamera(`${cameraName}-gen-${generation}`, new BABYLON.Vector3(0, 0, 0), scene);
    camera.minZ = 0.05;
    camera.maxZ = Math.max(params.distanceM * 4, 20);
    camera.fov = params.yawFovDeg * Math.PI / 180;
    camera.metadata = { capabilityId: CAPABILITY_ID, generation, candidate: true, coordinateContract: "identity metres, right-handed, +Y, yaw0->+Z" };
    const offset = rearDirection(BABYLON, player.yawRad, params.distanceM);
    camera.position.set(player.position.x + offset.x, player.position.y, player.position.z + offset.z);
    camera.setTarget(new BABYLON.Vector3(player.position.x, player.position.y, player.position.z));
    const size = RESOLUTION[params.resolutionTier];
    const texture = new BABYLON.RenderTargetTexture(`${textureName}-gen-${generation}`, { width: size, height: size }, scene, false);
    texture.renderList = [];
    texture.metadata = { capabilityId: CAPABILITY_ID, generation, candidate: true, resolutionTier: params.resolutionTier, resolutionPx: size };
    if (Array.isArray(scene.customRenderTargets) && !scene.customRenderTargets.includes(texture)) scene.customRenderTargets.push(texture);
    resource = { camera, texture, parameters: clone(params), generation };
  }

  function applyCapabilityView(view, worldEpoch, transform) {
    if (closed) return { accepted: false, kind: "closed" };
    if (!Number.isInteger(worldEpoch) || worldEpoch < 0) return { accepted: false, kind: "E_EPOCH" };
    let player;
    let nextSources;
    try {
      player = playerTransform(transform);
      nextSources = validateView(view, worldEpoch);
    } catch (e) {
      return { accepted: false, kind: e.code || "E_SCHEMA", error: e.message };
    }
    if (currentEpoch !== null && worldEpoch < currentEpoch) return { accepted: false, kind: "stale_epoch", generation: resource?.generation || null };
    const epochChanged = currentEpoch !== null && worldEpoch > currentEpoch;
    if (epochChanged) {
      disposeResource("world_epoch_replaced");
      sources = new Map();
    }
    currentEpoch = worldEpoch;
    lastPlayer = player;
    const nextMap = new Map(nextSources.map((entry) => [`${entry.grantId}|${entry.source}`, entry]));
    const changed = JSON.stringify([...sources.values()]) !== JSON.stringify([...nextMap.values()]);
    sources = nextMap;
    if (sources.size === 0) {
      disposeResource("last_source_revoked");
      return { accepted: true, kind: "revoked", generation: null, epoch: currentEpoch, disposed: true };
    }
    if (!resource) createResource([...sources.values()][0].parameters, player);
    else if (changed) {
      const params = [...sources.values()][0].parameters;
      if (JSON.stringify(params) !== JSON.stringify(resource.parameters)) {
        disposeResource("parameters_changed");
        createResource(params, player);
      }
    }
    updateTransform(player);
    return { accepted: true, kind: epochChanged ? "epoch_replaced" : (changed ? "updated" : "idempotent"), generation: resource?.generation || null, sourceCount: sources.size, epoch: currentEpoch, disposed: false };
  }

  function updateTransform(player) {
    if (!resource) return;
    const offset = rearDirection(BABYLON, player.yawRad, resource.parameters.distanceM);
    resource.camera.position.set(player.position.x + offset.x, player.position.y, player.position.z + offset.z);
    resource.camera.setTarget(new BABYLON.Vector3(player.position.x, player.position.y, player.position.z));
    lastPlayer = player;
  }

  function update(worldEpoch, transform) {
    if (closed) return { updated: false, kind: "closed" };
    if (currentEpoch === null || worldEpoch !== currentEpoch) return { updated: false, kind: worldEpoch < currentEpoch ? "stale_epoch" : "epoch_mismatch" };
    try { updateTransform(playerTransform(transform)); } catch (e) { return { updated: false, kind: e.code || "E_SCHEMA" }; }
    return { updated: true, kind: resource ? "updated" : "baseline_only", generation: resource?.generation || null };
  }

  function render(worldEpoch) {
    if (closed) return { rendered: false, kind: "closed" };
    if (currentEpoch === null || worldEpoch !== currentEpoch) return { rendered: false, kind: worldEpoch < currentEpoch ? "stale_epoch" : "epoch_mismatch" };
    if (!resource) return { rendered: false, kind: "no_grant" };
    resource.texture.render(false);
    renderCount += 1;
    return { rendered: true, kind: "rear_view", generation: resource.generation, epoch: currentEpoch };
  }

  function dispose() {
    if (closed) return { disposed: false, kind: "closed" };
    disposeResource("adapter_dispose");
    closed = true;
    currentEpoch = null;
    sources = new Map();
    lastPlayer = null;
    return { disposed: true, kind: "disposed" };
  }

  function serializeCapabilityView(view) {
    const safe = { protocol: view.protocol, version: view.version, worldId: view.worldId, worldEpoch: view.worldEpoch, tick: view.tick, effective: clone(view.effective || []), baseline: clone(view.baseline || ["ui.accessibility", "controls.basic"]) };
    return safe;
  }

  function serializeSaveSnapshot(snapshot) {
    const safe = object(snapshot, "snapshot");
    return { schemaVersion: safe.schemaVersion, worldId: safe.worldId, worldEpoch: safe.worldEpoch, semanticGrants: clone(safe.semanticGrants || []), accessibility: clone(safe.accessibility || { captions: true, scalableText: true, remappableControls: true }) };
  }

  return {
    applyCapabilityView,
    update,
    render,
    dispose,
    serializeCapabilityView,
    serializeSaveSnapshot,
    accessibilityState: () => ({ available: true, baseline: ["ui.accessibility", "controls.basic"], subtitles: true, scalableText: true, remappableControls: true, readability: true }),
    stats: () => ({ closed, epoch: currentEpoch, generation, sourceCount: sources.size, hasCamera: Boolean(resource?.camera), hasTexture: Boolean(resource?.texture), disposedCount, renderCount, camera: resource?.camera || null, texture: resource?.texture || null, player: lastPlayer })
  };
}

module.exports = { PROTOCOL, VERSION, CAPABILITY_ID, createRearViewAdapter };
