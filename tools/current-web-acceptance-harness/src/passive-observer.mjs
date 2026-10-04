/** Bounded diagnostic observation only. Never calls the IPC bridge or schedules input. */
export function installPassiveObserver(host = window) {
  const now = () => host.performance?.now?.() ?? 0;
  const increment = value => Math.min(Number.MAX_SAFE_INTEGER, value + 1);
  const state = { kind: "passive-render-timing", startedAtMs: now(),
    frames: 0, maxFrameGapMs: 0, lastFrameAtMs: null, frameSamples: [],
    longTaskCount: 0, maxLongTaskMs: 0, longTasks: [] };
  let running = true;
  let frameId;
  let observer;
  const push = (array, value) => { if (array.length === 64) array.shift(); array.push(value); };
  const frame = timestamp => {
    if (!running) return;
    state.frames = increment(state.frames);
    if (state.lastFrameAtMs !== null) state.maxFrameGapMs = Math.max(state.maxFrameGapMs, timestamp - state.lastFrameAtMs);
    state.lastFrameAtMs = timestamp;
    if (state.frames % 30 === 0) push(state.frameSamples, timestamp);
    frameId = host.requestAnimationFrame(frame);
  };
  if (typeof host.requestAnimationFrame === "function") frameId = host.requestAnimationFrame(frame);
  if (typeof host.PerformanceObserver === "function") {
    try {
      observer = new host.PerformanceObserver(list => {
        if (!running) return;
        for (const entry of list.getEntries()) {
          state.longTaskCount = increment(state.longTaskCount);
          state.maxLongTaskMs = Math.max(state.maxLongTaskMs, entry.duration);
          push(state.longTasks, { startTimeMs: entry.startTime, durationMs: entry.duration });
        }
      });
      observer.observe({ type: "longtask", buffered: true });
    } catch { observer?.disconnect(); }
  }
  return { state, stop() { running = false; if (frameId !== undefined) host.cancelAnimationFrame?.(frameId); observer?.disconnect(); } };
}

export function passiveObserverScript() {
  return `window.__CURRENT_WEB_PASSIVE__ = (${installPassiveObserver.toString()})(window);`;
}
