(function (root) {
  "use strict";
  let started = false;
  function start({ canvas, mapCanvas, quality, submitInput, requestFullSnapshot, worldId, worldEpoch } = {}) {
    if (started) throw new Error("E_RENDERER_BOOT_ALREADY_STARTED");
    if (!root.FormalView) throw new Error("E_FORMAL_VIEW_UNAVAILABLE");
    root.FormalView.init({ canvas, mapCanvas, quality, submitInput, requestFullSnapshot });
    if (worldId !== undefined) root.FormalView.reset({ worldId, worldEpoch });
    started = true; return root.FormalView;
  }
  function stop() { if (!started) return; root.FormalView.dispose(); started = false; }
  root.FreezeV02RendererBoot = { start, stop, isStarted: () => started };
  if (typeof module !== "undefined" && module.exports) module.exports = root.FreezeV02RendererBoot;
})(typeof window !== "undefined" ? window : globalThis);
