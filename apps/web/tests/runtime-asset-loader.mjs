import test from "node:test";
import assert from "node:assert/strict";
import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { RuntimeAssetLoader } from "../dist/assets/RuntimeAssetLoader.js";

const testDir = resolve(fileURLToPath(new URL(".", import.meta.url)));
const distRoot = resolve(testDir, "../dist");
const outputsRoot = "assets/derived/";

async function withDistServer(run) {
  const requests = [];
  const server = createServer(async (request, response) => {
    const pathname = new URL(request.url ?? "/", "http://localhost").pathname;
    const relative = decodeURIComponent(pathname).replace(/^\//, "");
    const filePath = resolve(distRoot, relative);
    if (!filePath.startsWith(`${distRoot}${sep}`)) {
      response.writeHead(400).end();
      return;
    }
    requests.push(relative);
    try {
      await stat(filePath);
      response.writeHead(200, { "cache-control": "no-store" });
      createReadStream(filePath).pipe(response);
    } catch {
      response.writeHead(404).end();
    }
  });
  await new Promise((resolveListen, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolveListen);
  });
  const address = server.address();
  try { await run(`http://127.0.0.1:${address.port}/`, requests); }
  finally { await new Promise(resolveClose => server.close(resolveClose)); }
}

test("loads real production dist bytes, verifies all outputs, reports progress, and caches only registry metadata", async () => {
  await withDistServer(async (baseUrl, requests) => {
    const progress = [];
    const loader = new RuntimeAssetLoader({ baseUrl });
    const first = loader.load({ onProgress: item => progress.push(item) });
    const duplicate = loader.load();
    const registry = await first;
    assert.equal(await duplicate, registry);
    assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), true);
    assert.equal(registry.resolve("player.cenyao").kind, "actor");
    assert.equal(requests.length, 116);
    assert.equal(requests.filter(path => path.startsWith(outputsRoot)).length, 114);
    assert.equal(new Set(requests).size, 116);
    assert.equal(await loader.load(), registry);
    assert.equal(requests.length, 116, "subsequent loads must not redownload or retain output byte arrays");
    assert.equal(progress.at(-1).phase, "complete");
    assert.equal(progress.at(-1).completedFiles, 116);
    assert.equal(progress.at(-1).completedOutputs, 114);
    assert.ok(progress.at(-1).bytesFetched > 240_000_000);
    assert.ok(progress.at(-1).elapsedMs >= 0);
  });
});

test("missing output and changed output bytes fail closed without returning a registry", async () => {
  await withDistServer(async (baseUrl) => {
    const missing = new RuntimeAssetLoader({
      baseUrl,
      fetchImpl: async (input, init) => {
        if (new URL(input).pathname.includes("/assets/derived/")) return new Response("missing", { status: 404 });
        return fetch(input, init);
      },
    });
    await assert.rejects(missing.load(), /E_ASSET_OUTPUT_MISSING/);

    let changed = false;
    const wrongSha = new RuntimeAssetLoader({
      baseUrl,
      fetchImpl: async (input, init) => {
        const response = await fetch(input, init);
        if (!changed && new URL(input).pathname.includes("/assets/derived/")) {
          changed = true;
          const bytes = new Uint8Array(await response.arrayBuffer());
          bytes[0] ^= 1;
          return new Response(bytes, { status: 200 });
        }
        return response;
      },
    });
    await assert.rejects(wrongSha.load(), /E_ASSET_OUTPUT_SHA/);
    assert.equal(changed, true);
  });
});

test("rejects malformed base URLs and propagates cancellation before output requests", async () => {
  assert.throws(() => new RuntimeAssetLoader({ baseUrl: "not a url" }), /E_RUNTIME_ASSET_BASE_URL/);
  const protocolFailure = new RuntimeAssetLoader({
    baseUrl: "tauri://localhost/",
    fetchImpl: async () => { throw new TypeError("custom protocol unavailable"); },
  });
  await assert.rejects(protocolFailure.load(), /E_RUNTIME_ASSET_FETCH:governance\/assets\/AI_ASSET_RELEASE_MANIFEST\.json/);
  await withDistServer(async (baseUrl, requests) => {
    const controller = new AbortController();
    let outputRequested = false;
    const loader = new RuntimeAssetLoader({
      baseUrl,
      fetchImpl: async (input, init) => {
        if (new URL(input).pathname.includes("/assets/derived/")) {
          outputRequested = true;
          setTimeout(() => controller.abort(), 0);
          return new Promise((resolve, reject) => {
            init.signal.addEventListener("abort", () => reject(new DOMException("cancelled", "AbortError")), { once: true });
            if (init.signal.aborted) reject(new DOMException("cancelled", "AbortError"));
          });
        }
        return fetch(input, init);
      },
    });
    const loading = loader.load({ signal: controller.signal });
    await assert.rejects(loading, error => error?.name === "AbortError");
    assert.equal(outputRequested, true);
    assert.equal(requests.length, 2);
  });
});

test("abort drains all four active output reads before a clean retry", { timeout: 20_000 }, async () => {
  await withDistServer(async (baseUrl) => {
    const controller = new AbortController();
    let active = 0; let outputReads = 0; let firstAttempt = true;
    let firstFour;
    const ready = new Promise(resolve => { firstFour = resolve; });
    const loader = new RuntimeAssetLoader({ baseUrl, fetchImpl: async (input, init) => {
      if (!firstAttempt || !new URL(input).pathname.includes("/assets/derived/")) return fetch(input, init);
      outputReads++; active++;
      if (outputReads === 4) firstFour();
      try {
        return await new Promise((_resolve, reject) => {
          const abort = () => reject(new DOMException("cancelled", "AbortError"));
          init.signal.addEventListener("abort", abort, { once: true });
          if (init.signal.aborted) abort();
        });
      } finally { active--; }
    } });
    const loading = loader.load({ signal: controller.signal });
    const rejected = assert.rejects(loading, error => error.name === "AbortError");
    await ready; assert.equal(active, 4);
    controller.abort(); await rejected;
    assert.equal(active, 0); assert.equal(outputReads, 4);
    firstAttempt = false;
    const progress = [];
    const registry = await loader.load({ onProgress: item => progress.push(item) });
    assert.equal(registry.hasAsset("runtime2d.actor.cenyao.base.v1"), true);
    assert.equal(progress.at(-1).completedOutputs, 114);
  });
});
