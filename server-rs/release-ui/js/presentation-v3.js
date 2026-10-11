(function (root, factory) {
  const api = factory(root);
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  if (root) root.FormalView = api.defaultView;
})(typeof window !== "undefined" ? window : globalThis, function (root) {
  "use strict";
  const SCHEMA = "freeze-v02-interfaces/1.2";
  const PROTOCOL = "continuous-ipc";
  const VERSION = 1;

  function fail(reason, expectedRevision) { return { accepted: false, reason, expectedRevision }; }
  function finiteInteger(value) { return Number.isSafeInteger(value) && value >= 0; }

  function createFormalView(dependencies = {}) {
    let renderer = null, hud = null, requestFullSnapshot = null, identity = null, disposed = false;
    let lastRevision = null, lastTick = null, lastAckSeq = null, lastServerTimeMs = null;
    const createRenderer = dependencies.createRenderer || (options => new root.FreezeV02TpsRenderer.FormalTpsRenderer(options));
    const createHud = dependencies.createHud || (options => new root.FreezeV02CapabilityHud.CapabilityHud(options));

    function init({ canvas, mapCanvas, quality, submitInput, requestFullSnapshot: request } = {}) {
      if (renderer || hud) throw new Error("E_ALREADY_INITIALIZED");
      if (typeof request !== "function") throw new Error("E_REQUEST_FULL_REQUIRED");
      requestFullSnapshot = request;
      renderer = createRenderer({ canvas, quality }); hud = createHud({ mapCanvas }); disposed = false;
      return { initialized: true, inputSubmissionOwnedByIntegrator: typeof submitInput === "function" };
    }

    function reset({ worldId, worldEpoch } = {}) {
      if (typeof worldId !== "string" || !worldId || !finiteInteger(worldEpoch)) throw new Error("E_INVALID_IDENTITY");
      identity = { worldId, worldEpoch }; lastRevision = null; lastTick = null; lastAckSeq = null; lastServerTimeMs = null; renderer?.reset(); hud?.reset();
    }

    function validVec3(value) { return value && Number.isFinite(value.xM) && Number.isFinite(value.yM) && Number.isFinite(value.zM); }
    function validTransform(value) { return value && validVec3(value.positionM) && Number.isFinite(value.yawRad); }
    function validViewNumbers(view) {
      if (!view.player || !validTransform(view.player.transform) || !validVec3(view.player.velocityMps)) return false;
      if (!(view.actors || []).every(actor => actor && validTransform(actor.transform))) return false;
      if (!(view.doors || []).every(door => door && validTransform(door.transform))) return false;
      const map = view.capabilities && view.capabilities.exploredMap;
      if (!map || !validVec3(map.playerPositionM)) return false;
      if (!(map.rooms || []).every(room => Array.isArray(room.outlineM) && room.outlineM.every(validVec3))) return false;
      if (!(map.objectives || []).every(objective => validVec3(objective.positionM))) return false;
      return true;
    }

    function validateEnvelope(envelope) {
      if (!envelope || envelope.kind !== "full") {
        if (envelope?.kind === "delta") { requestFullSnapshot?.({ reason: "E_DELTA_UNSUPPORTED", expectedRevision: lastRevision === null ? 0 : lastRevision + 1 }); return fail("E_DELTA_UNSUPPORTED", lastRevision === null ? 0 : lastRevision + 1); }
        return fail("E_KIND_UNSUPPORTED", lastRevision === null ? 0 : lastRevision + 1);
      }
      if (envelope.schemaVersion !== SCHEMA) return fail("E_SCHEMA_MISMATCH", lastRevision === null ? 0 : lastRevision + 1);
      const view = envelope.view;
      if (!view || view.protocol !== PROTOCOL || view.version !== VERSION || view.schemaVersion !== SCHEMA) return fail("E_VIEW_SCHEMA_MISMATCH", lastRevision === null ? 0 : lastRevision + 1);
      for (const key of ["worldEpoch", "serverTick", "authorityRevision", "ackSeq", "serverTimeMs"]) if (!finiteInteger(view[key])) return fail("E_NON_MONOTONIC_CLOCK", lastRevision === null ? 0 : lastRevision + 1);
      if (!validViewNumbers(view)) return fail("E_NON_FINITE", lastRevision === null ? 0 : lastRevision + 1);
      if (envelope.worldId !== view.worldId || envelope.worldEpoch !== view.worldEpoch || envelope.serverTick !== view.serverTick || envelope.authorityRevision !== view.authorityRevision || envelope.ackSeq !== view.ackSeq) return fail("E_ENVELOPE_VIEW_MISMATCH", lastRevision === null ? 0 : lastRevision + 1);
      if (identity && (view.worldId !== identity.worldId || view.worldEpoch !== identity.worldEpoch)) return fail("E_WORLD_IDENTITY_MISMATCH", lastRevision === null ? 0 : lastRevision + 1);
      if (lastRevision !== null && view.authorityRevision <= lastRevision) return fail("E_STALE_REVISION", lastRevision + 1);
      if (lastTick !== null && view.serverTick <= lastTick) return fail("E_STALE_TICK", lastRevision + 1);
      if (lastAckSeq !== null && view.ackSeq < lastAckSeq) return fail("E_STALE_ACK", lastRevision + 1);
      if (lastServerTimeMs !== null && view.serverTimeMs <= lastServerTimeMs) return fail("E_STALE_SERVER_TIME", lastRevision + 1);
      if (lastRevision !== null && view.authorityRevision !== lastRevision + 1) { requestFullSnapshot?.({ reason: "E_REVISION_GAP", expectedRevision: lastRevision + 1 }); return fail("E_REVISION_GAP", lastRevision + 1); }
      return { accepted: true, view };
    }

    function consume(envelope) {
      if (disposed || !renderer || !hud) return fail("E_NOT_INITIALIZED", lastRevision === null ? 0 : lastRevision + 1);
      const checked = validateEnvelope(envelope); if (!checked.accepted) return checked;
      const view = checked.view;
      renderer.applyView(view); hud.apply(view.capabilities);
      identity ||= { worldId: view.worldId, worldEpoch: view.worldEpoch };
      lastRevision = view.authorityRevision; lastTick = view.serverTick; lastAckSeq = view.ackSeq; lastServerTimeMs = view.serverTimeMs;
      return { accepted: true, reason: "OK", expectedRevision: lastRevision + 1 };
    }

    function setQuality(value) { if (!renderer) throw new Error("E_NOT_INITIALIZED"); renderer.setQuality(value); }
    function dispose() { if (disposed) return; disposed = true; renderer?.dispose(); hud?.dispose(); renderer = null; hud = null; identity = null; lastRevision = null; lastTick = null; lastAckSeq = null; lastServerTimeMs = null; requestFullSnapshot = null; }
    return { init, consume, setQuality, reset, dispose, constants: { SCHEMA, PROTOCOL, VERSION } };
  }

  return { createFormalView, defaultView: createFormalView(), SCHEMA, PROTOCOL, VERSION };
});
