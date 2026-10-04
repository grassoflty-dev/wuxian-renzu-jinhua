// These are automation scheduling budgets, never application IPC or FPS budgets.
export const STRICT_PROFILE = Object.freeze({ name: "strict", testTimeoutMs: 60_000,
  expectTimeoutMs: 15_000, actionTimeoutMs: 0, performanceAcceptance: false, nativeGameplayAcceptance: false });
export const SOFTWARE_CI_PROFILE = Object.freeze({ name: "windows-edge-swiftshader-functional",
  testTimeoutMs: 180_000, expectTimeoutMs: 60_000, actionTimeoutMs: 60_000,
  performanceAcceptance: false, nativeGameplayAcceptance: false });
export function executionProfile(name, mode) {
  if (name === undefined || name === "strict") return STRICT_PROFILE;
  if (name !== SOFTWARE_CI_PROFILE.name) throw new Error("E_CURRENT_WEB_UNKNOWN_EXECUTION_PROFILE");
  if (mode !== "functional") throw new Error("E_CURRENT_WEB_SOFTWARE_PROFILE_FUNCTIONAL_ONLY");
  return SOFTWARE_CI_PROFILE;
}
export function validateFingerprint(profile, fingerprint) {
  if (profile.name === STRICT_PROFILE.name) return;
  if (fingerprint?.platform !== "win32" || fingerprint?.browserChannel !== "msedge" ||
      !/SwiftShader/i.test(fingerprint?.graphics?.renderer ?? "") ||
      fingerprint?.deviceScaleFactor !== 1) throw new Error("E_CURRENT_WEB_SOFTWARE_FINGERPRINT_REQUIRED");
}
// Serialize this function for a single public API observation, without replacing
// any GL methods, installing a scheduler, changing resolution or generating input.
export function readGraphicsFingerprint(canvas) {
  if (!canvas) return null;
  const gl = canvas.getContext("webgl2") || canvas.getContext("webgl");
  if (!gl) return null;
  const debug = gl.getExtension("WEBGL_debug_renderer_info");
  const bounded = value => String(value ?? "").slice(0, 256);
  return { vendor: bounded(gl.getParameter(debug ? debug.UNMASKED_VENDOR_WEBGL : gl.VENDOR)),
    renderer: bounded(gl.getParameter(debug ? debug.UNMASKED_RENDERER_WEBGL : gl.RENDERER)),
    version: bounded(gl.getParameter(gl.VERSION)), maxTextureSize: gl.getParameter(gl.MAX_TEXTURE_SIZE) };
}
export function readActualRendererFingerprint(host, readGraphics) {
  const count = host.__CURRENT_WEB_MOCK__?.count("formal_submit_input") ?? 0;
  if (count === 0) return { status: "not_started", graphics: null };
  if (!host.document.querySelector('.shell[data-view="journey"]')) return { status: "inactive", graphics: null };
  if (!(host.__CURRENT_WEB_MOCK__?.state?.ack > 0)) return { status: "not_started", graphics: null };
  const canvas = host.document.querySelector("#world-canvas");
  if (!canvas) return { status: "missing", graphics: null };
  return { status: "started", deviceScaleFactor: host.devicePixelRatio, graphics: readGraphics(canvas) };
}
export function actualRendererProbe() {
  return `((host) => (${readActualRendererFingerprint.toString()})(host, (${readGraphicsFingerprint.toString()})))`;
}
