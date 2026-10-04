/** Public WebGL timing wrappers for a diagnostic-only run; original calls/results are unchanged. */
export function installGpuObserver(host = window) {
  const names = ["clear", "drawArrays", "drawElements", "drawArraysInstanced", "drawElementsInstanced",
    "blitFramebuffer", "readPixels", "texImage2D", "texSubImage2D", "bufferData", "bufferSubData", "finish", "flush"];
  const state = { kind: "diagnostic-only-public-webgl-timing", graphics: null, fingerprintError: null,
    methods: Object.fromEntries(names.map(name => [name, { calls: 0, totalMs: 0, maxMs: 0, errors: 0 }])),
    slowCalls: [], instrumentedMethods: [] };
  const cleanups = [];
  const wrappedFunctions = new Set();
  const now = () => host.performance?.now?.() ?? 0;
  const increment = value => Math.min(Number.MAX_SAFE_INTEGER, value + 1);
  function replace(prototype, name, create) {
    if (!prototype || typeof prototype[name] !== "function" || wrappedFunctions.has(prototype[name])) return;
    const descriptor = Object.getOwnPropertyDescriptor(prototype, name);
    const original = prototype[name]; const wrapper = create(original);
    try {
      Object.defineProperty(prototype, name, { configurable: descriptor?.configurable ?? true,
        enumerable: descriptor?.enumerable ?? false, writable: descriptor?.writable ?? true, value: wrapper });
      wrappedFunctions.add(wrapper);
      cleanups.push(() => { if (prototype[name] !== wrapper) return;
        if (descriptor) Object.defineProperty(prototype, name, descriptor); else delete prototype[name]; });
    } catch { /* An unavailable binding is recorded by the instrumented-method list, never replaced unsafely. */ }
  }
  for (const type of ["WebGLRenderingContext", "WebGL2RenderingContext"]) {
    for (const name of names) {
      const before = cleanups.length;
      replace(host[type]?.prototype, name, original => function (...args) {
        if (this.canvas?.id !== "world-canvas") return Reflect.apply(original, this, args);
        const start = now(); let failed = false;
        try { return Reflect.apply(original, this, args); }
        catch (error) { failed = true; throw error; }
        finally {
          const duration = Math.max(0, now() - start); const row = state.methods[name];
          row.calls = increment(row.calls); row.errors = Math.min(Number.MAX_SAFE_INTEGER, row.errors + Number(failed));
          row.totalMs = Math.min(Number.MAX_SAFE_INTEGER, row.totalMs + duration); row.maxMs = Math.max(row.maxMs, duration);
          if (duration >= 20 && state.slowCalls.length < 32) state.slowCalls.push({ method: name, startTimeMs: start,
            durationMs: duration, width: this.canvas.width, height: this.canvas.height,
            stack: String(new Error().stack ?? "").slice(0, 1600) });
        }
      });
      if (cleanups.length > before) state.instrumentedMethods.push(`${type}.${name}`);
    }
  }
  replace(host.HTMLCanvasElement?.prototype, "getContext", original => function (...args) {
    const context = Reflect.apply(original, this, args);
    if (this.id === "world-canvas" && context && String(args[0]).startsWith("webgl") && !state.graphics) {
      try {
        const info = context.getExtension("WEBGL_debug_renderer_info");
        state.graphics = { context: String(args[0]),
          vendor: String(context.getParameter(info?.UNMASKED_VENDOR_WEBGL ?? context.VENDOR)).slice(0, 200),
          renderer: String(context.getParameter(info?.UNMASKED_RENDERER_WEBGL ?? context.RENDERER)).slice(0, 200),
          version: String(context.getParameter(context.VERSION)).slice(0, 200),
          shadingLanguageVersion: String(context.getParameter(context.SHADING_LANGUAGE_VERSION)).slice(0, 200),
          maxTextureSize: context.getParameter(context.MAX_TEXTURE_SIZE) };
      } catch (error) { state.fingerprintError = String(error).slice(0, 300); }
    }
    return context;
  });
  return { state, stop() { for (const cleanup of cleanups.reverse()) cleanup(); cleanups.length = 0; } };
}
export function gpuObserverScript() {
  return `window.__CURRENT_WEB_GPU_PROFILE__ = (${installGpuObserver.toString()})(window);`;
}
