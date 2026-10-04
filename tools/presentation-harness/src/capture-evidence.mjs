import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";

export async function saveScreenshotEvidence(page, destination, label) {
  await mkdir(path.dirname(destination), { recursive: true });
  const png = await page.screenshot({ path: destination, animations: "disabled" });
  const decoded = PNG.sync.read(png);
  const colors = new Set();
  const stride = Math.max(1, Math.floor(Math.sqrt((decoded.width * decoded.height) / 4096)));
  for (let y = 0; y < decoded.height; y += stride) {
    for (let x = 0; x < decoded.width; x += stride) {
      const offset = (y * decoded.width + x) * 4;
      colors.add(`${decoded.data[offset]},${decoded.data[offset + 1]},${decoded.data[offset + 2]},${decoded.data[offset + 3]}`);
    }
  }
  return {
    label,
    path: destination,
    sha256: createHash("sha256").update(png).digest("hex"),
    width: decoded.width,
    height: decoded.height,
    sampledDistinctRgba: colors.size,
    sampleStride: stride
  };
}

export async function saveCanvasEvidence(page, destination, label) {
  await mkdir(path.dirname(destination), { recursive: true });
  const png = await page.locator("#gameCanvas").screenshot({ path: destination, animations: "disabled" });
  const decoded = PNG.sync.read(png);
  const colors = new Set();
  const stride = Math.max(1, Math.floor(Math.sqrt((decoded.width * decoded.height) / 4096)));
  for (let y = 0; y < decoded.height; y += stride) {
    for (let x = 0; x < decoded.width; x += stride) {
      const offset = (y * decoded.width + x) * 4;
      colors.add(`${decoded.data[offset]},${decoded.data[offset + 1]},${decoded.data[offset + 2]},${decoded.data[offset + 3]}`);
    }
  }
  return {
    label,
    path: destination,
    sha256: createHash("sha256").update(png).digest("hex"),
    width: decoded.width,
    height: decoded.height,
    sampledDistinctRgba: colors.size,
    sampleStride: stride
  };
}

export async function sampleFrameTimes(page, durationMs = 1200) {
  return page.evaluate(async duration => {
    const deltas = [];
    const started = performance.now();
    let previous = started;
    await new Promise(resolve => {
      const frame = now => {
        deltas.push(now - previous);
        previous = now;
        if (now - started >= duration) resolve();
        else requestAnimationFrame(frame);
      };
      requestAnimationFrame(frame);
    });
    const sorted = deltas.slice(1).sort((a, b) => a - b);
    const at = p => sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor((sorted.length - 1) * p))] : null;
    const meanMs = deltas.length > 1 ? deltas.slice(1).reduce((sum, value) => sum + value, 0) / (deltas.length - 1) : null;
    return {
      durationMs: Math.round(deltas.length ? previous - started : 0),
      frames: Math.max(0, deltas.length - 1),
      fps: meanMs ? 1000 / meanMs : 0,
      meanFrameMs: meanMs,
      p50FrameMs: at(0.5),
      p95FrameMs: at(0.95),
      devicePixelRatio: window.devicePixelRatio
    };
  }, durationMs);
}

export async function appendRunEvidence(destination, record) {
  await mkdir(path.dirname(destination), { recursive: true });
  await writeFile(destination, `${JSON.stringify(record, null, 2)}\n`, "utf8");
}
