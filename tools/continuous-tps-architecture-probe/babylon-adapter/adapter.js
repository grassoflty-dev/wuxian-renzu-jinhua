"use strict";

const PROTOCOL = "continuous-ipc";
const VERSION = 1;

function adapterError(code, message) {
  const error = new Error(message || code);
  error.code = code;
  return error;
}

function finite(value) {
  return typeof value === "number" && Number.isFinite(value);
}

function requireFinite(value, label) {
  if (!finite(value)) throw adapterError("reject_non_finite", `${label} must be finite`);
  return value;
}

function requireObject(value, label) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw adapterError("reject_schema", `${label} must be an object`);
  }
  return value;
}

function requireInteger(value, label) {
  if (!Number.isInteger(value) || value < 0) {
    throw adapterError("reject_schema", `${label} must be a non-negative integer`);
  }
  return value;
}

function adaptWorldViewPlayer(worldView) {
  requireObject(worldView, "worldView");
  if (worldView.protocol !== PROTOCOL || worldView.version !== VERSION) {
    throw adapterError("reject_schema", "WorldView protocol/version mismatch");
  }
  requireInteger(worldView.worldEpoch, "worldEpoch");
  requireInteger(worldView.serverTick, "serverTick");
  if (typeof worldView.worldId !== "string" || worldView.worldId.length === 0) {
    throw adapterError("reject_schema", "worldId must be a non-empty string");
  }
  const player = requireObject(worldView.player, "player");
  const transform = requireObject(player.transform, "player.transform");
  const position = requireObject(transform.positionM, "player.transform.positionM");
  const velocity = requireObject(player.velocityMps, "player.velocityMps");
  return {
    epoch: worldView.worldEpoch,
    tick: worldView.serverTick,
    position: {
      x: requireFinite(position.xM, "positionM.xM"),
      y: requireFinite(position.yM, "positionM.yM"),
      z: requireFinite(position.zM, "positionM.zM"),
    },
    yaw: requireFinite(transform.yawRad, "yawRad"),
    velocity: {
      x: requireFinite(velocity.xM, "velocityMps.xM"),
      y: requireFinite(velocity.yM, "velocityMps.yM"),
      z: requireFinite(velocity.zM, "velocityMps.zM"),
    },
  };
}

function cloneSample(sample, timeMs) {
  return {
    epoch: sample.epoch,
    tick: sample.tick,
    timeMs,
    position: { ...sample.position },
    yaw: sample.yaw,
    velocity: { ...sample.velocity },
  };
}

function lerp(a, b, alpha) {
  return a + (b - a) * alpha;
}

function lerpSample(a, b, alpha) {
  return {
    position: {
      x: lerp(a.position.x, b.position.x, alpha),
      y: lerp(a.position.y, b.position.y, alpha),
      z: lerp(a.position.z, b.position.z, alpha),
    },
    yaw: lerp(a.yaw, b.yaw, alpha),
  };
}

function applyNodeTransform(node, sample) {
  if (!node || !node.position || !node.rotation) return;
  if (typeof node.position.set === "function") {
    node.position.set(sample.position.x, sample.position.y, sample.position.z);
  } else {
    node.position.x = sample.position.x;
    node.position.y = sample.position.y;
    node.position.z = sample.position.z;
  }
  node.rotation.y = sample.yaw;
}

function createTp07aPlaceholder(scene, options) {
  if (!scene || typeof scene !== "object") {
    throw adapterError("reject_schema", "scene must be an object");
  }
  const config = options && typeof options === "object" ? options : {};
  const Babylon = config.BABYLON || (typeof globalThis !== "undefined" ? globalThis.BABYLON : null);
  const TransformNode = config.TransformNode || (Babylon && Babylon.TransformNode);
  if (typeof TransformNode !== "function") {
    throw adapterError("projection_failed", "Babylon TransformNode constructor unavailable");
  }
  const node = new TransformNode(config.name || "tp07a-placeholder-player", scene);
  const tickDurationMs = finite(config.tickDurationMs) && config.tickDurationMs > 0 ? config.tickDurationMs : 1000 / 60;
  const allowExtrapolation = config.allowExtrapolation === true;
  const horizonMs = finite(config.horizonMs) && config.horizonMs >= 0 ? config.horizonMs : 50;
  const correctionMeters = finite(config.correctionMeters) && config.correctionMeters >= 0 ? config.correctionMeters : 0.25;
  let closed = false;
  let epoch = null;
  let lastTick = null;
  let previous = null;
  let latest = null;

  function rejectResult(error) {
    return { accepted: false, kind: error.code || "reject_schema", error: error.message };
  }

  function accept(worldView, receivedAtMs) {
    if (closed) return { accepted: false, kind: "closed" };
    let sample;
    try {
      sample = adaptWorldViewPlayer(worldView);
    } catch (error) {
      return rejectResult(error);
    }
    const timeMs = receivedAtMs === undefined ? sample.tick * tickDurationMs : receivedAtMs;
    if (!finite(timeMs)) return { accepted: false, kind: "reject_non_finite", error: "receivedAtMs must be finite" };
    if (epoch !== null && sample.epoch < epoch) return { accepted: false, kind: "drop_stale_epoch" };
    if (epoch !== null && sample.epoch === epoch && sample.tick <= lastTick) {
      return { accepted: false, kind: "drop_tick_regression" };
    }
    const next = cloneSample(sample, timeMs);
    if (epoch === null || sample.epoch > epoch) {
      const epochReset = epoch !== null && sample.epoch > epoch;
      epoch = sample.epoch;
      lastTick = sample.tick;
      previous = next;
      latest = next;
      applyNodeTransform(node, next);
      return { accepted: true, kind: "accepted_snap", epoch: epoch, tick: lastTick, epochReset };
    }
    previous = latest;
    latest = next;
    lastTick = sample.tick;
    return { accepted: true, kind: "accepted_interpolation", epoch: epoch, tick: lastTick };
  }

  function render(nowMs) {
    if (closed) return { rendered: false, kind: "closed" };
    if (!finite(nowMs)) return { rendered: false, kind: "reject_non_finite" };
    if (!latest) return { rendered: false, kind: "no_sample" };
    if (previous === latest || !previous) {
      applyNodeTransform(node, latest);
      return { rendered: true, kind: "accepted_snap", epoch: latest.epoch, tick: latest.tick };
    }
    const duration = latest.timeMs - previous.timeMs;
    const alpha = duration > 0 ? Math.max(0, Math.min(1, (nowMs - previous.timeMs) / duration)) : 1;
    if (nowMs > latest.timeMs && allowExtrapolation) {
      const dtMs = Math.min(nowMs - latest.timeMs, horizonMs);
      const predicted = {
        position: {
          x: latest.position.x + latest.velocity.x * dtMs / 1000,
          y: latest.position.y + latest.velocity.y * dtMs / 1000,
          z: latest.position.z + latest.velocity.z * dtMs / 1000,
        },
        yaw: latest.yaw,
      };
      const correction = Math.hypot(
        predicted.position.x - latest.position.x,
        predicted.position.y - latest.position.y,
        predicted.position.z - latest.position.z,
      );
      if (correction <= correctionMeters) {
        applyNodeTransform(node, predicted);
        return { rendered: true, kind: "accepted_extrapolation", horizonMs: dtMs, correctionMeters: correction };
      }
    }
    const interpolated = lerpSample(previous, latest, alpha);
    applyNodeTransform(node, interpolated);
    return { rendered: true, kind: "accepted_interpolation", alpha };
  }

  function dispose() {
    if (closed) return;
    closed = true;
    if (node && typeof node.dispose === "function") node.dispose();
    previous = null;
    latest = null;
  }

  return { accept, render, dispose };
}

module.exports = {
  PROTOCOL,
  VERSION,
  adaptWorldViewPlayer,
  createTp07aPlaceholder,
};
