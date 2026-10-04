// Serialized into the existing Edge page. Readiness observes layout only: it
// never changes viewport dimensions, DOM content, styles, fonts or overflow.
export function waitForStableHubLayout(expected, budgetMs = 2000) {
  return new Promise(resolve => {
    const started = performance.now();
    let frame = null;
    let finished = false;
    let frames = 0;
    let stableFrames = 0;
    let previous = null;
    let last = null;
    const finish = (ready, reason) => {
      if (finished) return;
      finished = true;
      clearTimeout(timer);
      if (frame !== null) cancelAnimationFrame(frame);
      resolve({ ready, reason, frames, stableFrames, elapsedMs: Math.ceil(performance.now() - started), last });
    };
    const timer = setTimeout(() => finish(false, "layout-readiness-deadline"), budgetMs);
    if (!Number.isInteger(expected?.width) || !Number.isInteger(expected?.height)
      || expected.width <= 0 || expected.height <= 0 || !Number.isFinite(budgetMs) || budgetMs <= 0 || budgetMs > 2000) {
      finish(false, "invalid-layout-readiness-request"); return;
    }
    const rect = element => {
      const value = element.getBoundingClientRect();
      return { top: value.top, bottom: value.bottom, left: value.left, right: value.right };
    };
    const measure = () => {
      const hub = document.querySelector(".hub");
      const copy = document.querySelector(".hub-copy");
      const header = document.querySelector(".topbar");
      const footer = document.querySelector(".footer");
      const title = document.querySelector(".hub h1");
      if (!hub || !copy || !header || !footer || !title) throw new Error("missing-hub-layout-elements");
      const style = getComputedStyle(hub);
      const text = getComputedStyle(title);
      return {
        viewport: { width: innerWidth, height: innerHeight },
        fonts: document.fonts.status,
        hub: { width: hub.clientWidth, height: hub.clientHeight, scrollHeight: hub.scrollHeight, ...rect(hub) },
        copy: rect(copy), header: rect(header), footer: rect(footer),
        computed: { paddingTop: style.paddingTop, paddingBottom: style.paddingBottom,
          gridTemplateColumns: style.gridTemplateColumns, alignContent: style.alignContent,
          overflowY: style.overflowY, titleFontSize: text.fontSize, titleLineHeight: text.lineHeight,
          titleFontFamily: text.fontFamily.slice(0, 160) },
        media: {
          landscapeCompact: matchMedia("(min-width: 901px) and (max-height: 800px)").matches,
          shortLandscape: matchMedia("(min-width: 901px) and (min-height: 500px) and (max-height: 600px)").matches,
          narrow: matchMedia("(max-width: 900px)").matches,
        },
      };
    };
    const inspectFrame = () => {
      frame = null;
      if (finished) return;
      if (performance.now() - started >= budgetMs) { finish(false, "layout-readiness-deadline"); return; }
      frames++;
      try {
        last = measure();
        if (performance.now() - started >= budgetMs) { finish(false, "layout-readiness-deadline"); return; }
        const dimensionsMatch = last.viewport.width === expected.width && last.viewport.height === expected.height;
        const signature = JSON.stringify(last);
        if (dimensionsMatch && last.fonts === "loaded") {
          stableFrames = signature === previous ? stableFrames + 1 : 1;
          previous = signature;
        } else { stableFrames = 0; previous = null; }
        // Deliberately no fit predicate. A stable overflow is returned to the
        // original no-scroll/visibility assertions and must still fail there.
        if (stableFrames >= 2) { finish(true, "stable-layout"); return; }
        if (performance.now() - started >= budgetMs) { finish(false, "layout-readiness-deadline"); return; }
        frame = requestAnimationFrame(inspectFrame);
      } catch (error) { finish(false, String(error.message).slice(0, 160)); }
    };
    if (!document.fonts?.ready) { finish(false, "font-readiness-unavailable"); return; }
    Promise.resolve(document.fonts.ready).then(() => {
      if (!finished) frame = requestAnimationFrame(inspectFrame);
    }).catch(() => finish(false, "font-readiness-failed"));
  });
}
