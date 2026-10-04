import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { SceneDefinitionLoader, SCENE_DEFINITION_MANIFEST_PATH } from "../dist/assets/SceneDefinitionLoader.js";
import { SceneDefinitionSession, SceneDefinitionSessionError } from "../dist/game/SceneDefinitionSession.js";

const digest = bytes => createHash("sha256").update(bytes).digest("hex");
const bytes = value => Buffer.from(typeof value === "string" ? value : `${JSON.stringify(value)}\n`, "utf8");
const scene = (worldId = "grey_hive", sceneId = "gh_entry") => ({
  schemaVersion: 1, worldId, sceneId, boundsM: { x: 0, z: 0, width: 4, depth: 4 },
  presentation: { layers: [], sprites: [] }, collision: [], navigation: { nodes: [], links: [] },
  spawns: [], interactions: [], doors: [], triggers: [], hazards: [], checkpoints: [], objectives: [],
  transitions: [], cameraZones: [], logic: { traversal: [] }, occluders: [], vfxMarkers: [],
});
const identity = (worldId = "grey_hive", sceneId = "gh_entry", worldEpoch = 1) => ({ worldId, sceneId, worldEpoch });
function response(url, payload, status = 200) {
  const body = payload instanceof Uint8Array ? payload : bytes(payload);
  return { ok: status >= 200 && status < 300, status, url, arrayBuffer: async () => body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength) };
}
function manifestWith(definition, overrides = {}) {
  const sceneBytes = bytes(definition);
  const entry = { worldId: definition.worldId, sceneId: definition.sceneId,
    path: `scene-definitions/compiled/${definition.worldId}/${definition.sceneId}.json`, sha256: digest(sceneBytes), ...overrides };
  const manifestBytes = bytes({ schemaVersion: 1, scenes: [entry] });
  return { sceneBytes, entry, manifestBytes, manifestSha: digest(manifestBytes) };
}
function loaderFor(bundle, extra = {}) {
  const root = "https://game.test/app/";
  const files = new Map([
    [new URL(SCENE_DEFINITION_MANIFEST_PATH, root).pathname, bundle.manifestBytes],
    [new URL(bundle.entry.path, root).pathname, bundle.sceneBytes],
  ]);
  const fetchImpl = extra.fetchImpl ?? (async input => {
    const url = new URL(String(input));
    const found = files.get(url.pathname);
    return found ? response(url.href, found) : response(url.href, "", 404);
  });
  return new SceneDefinitionLoader({ baseUrl: root, expectedManifestSha256: extra.expectedManifestSha256 ?? bundle.manifestSha, fetchImpl });
}

test("loader fetches only manifested production paths and verifies exact identity and SHA-256", async () => {
  const definition = scene();
  const bundle = manifestWith(definition);
  const requested = [];
  const loader = loaderFor(bundle, { fetchImpl: async input => {
    const url = new URL(String(input)); requested.push(url.pathname);
    const found = url.pathname.endsWith("SCENE_DEFINITION_MANIFEST.json") ? bundle.manifestBytes : bundle.sceneBytes;
    return response(url.href, found);
  } });
  const loaded = await loader.loadForSnapshot(identity());
  assert.deepEqual([loaded.schemaVersion, loaded.worldId, loaded.sceneId], [1, "grey_hive", "gh_entry"]);
  assert.deepEqual(requested, [
    "/app/scene-definitions/SCENE_DEFINITION_MANIFEST.json",
    "/app/scene-definitions/compiled/grey_hive/gh_entry.json",
  ]);
});

test("absent scenes, changed manifest bytes, changed scene bytes, and swapped scene identities report explicit errors", async () => {
  const definition = scene();
  const bundle = manifestWith(definition);
  const emptyBytes = bytes({ schemaVersion: 1, scenes: [] });
  const emptyLoader = new SceneDefinitionLoader({ baseUrl: "https://game.test/app/", expectedManifestSha256: digest(emptyBytes),
    fetchImpl: async input => response(String(input), emptyBytes) });
  await assert.rejects(emptyLoader.loadForSnapshot(identity()), /E_SCENE_DEFINITION_MISSING:grey_hive:gh_entry/);

  const alteredManifest = loaderFor(bundle, { expectedManifestSha256: "0".repeat(64) });
  await assert.rejects(alteredManifest.loadForSnapshot(identity()), /E_SCENE_DEFINITION_MANIFEST_SHA/);

  const changedFile = loaderFor(bundle, { fetchImpl: async input => {
    const url = new URL(String(input));
    return response(url.href, url.pathname.endsWith("SCENE_DEFINITION_MANIFEST.json") ? bundle.manifestBytes : bytes(scene("grey_hive", "other_scene")));
  } });
  await assert.rejects(changedFile.loadForSnapshot(identity()), /E_SCENE_DEFINITION_SHA/);

  const swapped = manifestWith(scene("clockworks", "cw_entry"));
  const swappedLoader = loaderFor(swapped);
  await assert.rejects(swappedLoader.loadForSnapshot(identity()), /E_SCENE_DEFINITION_MISSING:grey_hive:gh_entry/);
});

test("manifest paths cannot escape the compiler scene root, even when the manifest hash is valid", async () => {
  const definition = scene();
  const bundle = manifestWith(definition, { path: "../../fixtures/scene.json" });
  const loader = loaderFor(bundle);
  await assert.rejects(loader.loadForSnapshot(identity()), /E_SCENE_DEFINITION_MANIFEST_ENTRY/);
});

test("missing files and aborted fetches are surfaced without trying alternate scene sources", async () => {
  const bundle = manifestWith(scene());
  const missing = loaderFor(bundle, { fetchImpl: async input => {
    const url = new URL(String(input));
    if (url.pathname.endsWith("SCENE_DEFINITION_MANIFEST.json")) return response(url.href, bundle.manifestBytes);
    return response(url.href, "", 404);
  } });
  await assert.rejects(missing.loadForSnapshot(identity()), /E_SCENE_DEFINITION_HTTP:404:scene-definitions\/compiled\/grey_hive\/gh_entry\.json/);

  const controller = new AbortController();
  controller.abort();
  await assert.rejects(loaderFor(bundle).loadForSnapshot(identity(), { signal: controller.signal }), error => error.name === "AbortError");
});

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
async function flush() { for (let i = 0; i < 12; i++) await Promise.resolve(); }
function snapshot(worldId, sceneId, worldEpoch) { return { protocolVersion: 3, worldId, sceneId, worldEpoch }; }

test("scene session rejects stale loads on authority changes and cannot revive after stop", async () => {
  const pending = [];
  const installed = [];
  const rendered = [];
  const expected = [];
  let destroyed = 0;
  const baseRenderer = {
    expectSceneIdentity: value => expected.push(value),
    setSceneDefinition: value => installed.push(value),
    render: async value => { rendered.push(value); },
    isReadyFor: value => rendered.at(-1) === value,
    invalidate() {},
    destroy: async () => { destroyed++; },
  };
  const loader = { loadForSnapshot: (_identity, { signal }) => {
    const operation = deferred(); pending.push({ ...operation, signal }); return operation.promise;
  } };
  const session = new SceneDefinitionSession(baseRenderer, loader);
  const first = snapshot("grey_hive", "gh_entry", 1);
  const second = snapshot("grey_hive", "gh_power", 2);
  const firstPrepare = assert.rejects(session.prepare(first), /E_SCENE_DEFINITION_STALE_FRAME/);
  const firstRender = assert.rejects(session.render(first), /E_SCENE_DEFINITION_STALE_FRAME/);
  session.acceptSnapshot(second);
  const secondRender = session.render(second);
  assert.equal(pending[0].signal.aborted, true);
  assert.equal(expected.at(-1).sceneId, "gh_power");
  pending[0].resolve(scene("grey_hive", "gh_entry"));
  await flush();
  assert.deepEqual(installed, []);
  assert.deepEqual(rendered, []);
  pending[1].resolve(scene("grey_hive", "gh_power"));
  await Promise.all([firstPrepare, firstRender, secondRender]);
  assert.deepEqual(installed.map(item => item.sceneId), ["gh_power"]);
  assert.deepEqual(rendered.map(item => item.sceneId), ["gh_power"]);

  const third = snapshot("clockworks", "cw_entry", 3);
  const stoppedPrepare = assert.rejects(session.prepare(third), /E_SCENE_DEFINITION_CANCELLED/);
  const destroyPromise = session.destroy();
  assert.equal(pending[2].signal.aborted, true);
  pending[2].resolve(scene("clockworks", "cw_entry"));
  await Promise.all([stoppedPrepare, destroyPromise]);
  assert.deepEqual(installed.map(item => item.sceneId), ["gh_power"]);
  assert.equal(destroyed, 1);
});


function sessionFixture(renderImpl) {
  let committed = null;
  let invalidations = 0;
  const baseRenderer = {
    expectSceneIdentity() { committed = null; }, setSceneDefinition() {},
    async render(value) { if (renderImpl) await renderImpl(value); committed = value; },
    isReadyFor: value => committed !== null && value.worldId === committed.worldId &&
      value.sceneId === committed.sceneId && value.worldEpoch === committed.worldEpoch,
    invalidate() { invalidations++; committed = null; }, destroy: async () => {},
  };
  const session = new SceneDefinitionSession(baseRenderer, { loadForSnapshot: async value => scene(value.worldId, value.sceneId) });
  return { session, baseRenderer, invalidations: () => invalidations };
}

test("scene readiness requires an exact committed identity and tracks renderer loss after render resolves", async () => {
  const f = sessionFixture(); const value = snapshot("grey_hive", "gh_entry", 1);
  await f.session.prepare(value); assert.equal(f.session.isReadyFor(value), false);
  await f.session.render(value); assert.equal(f.session.isReadyFor(value), true);
  for (const other of [snapshot("mist_harbor", "gh_entry", 1), snapshot("grey_hive", "gh_power", 1), snapshot("grey_hive", "gh_entry", 2)]) {
    assert.equal(f.session.isReadyFor(other), false);
  }
  f.baseRenderer.invalidate(); assert.equal(f.session.isReadyFor(value), false);
});

test("cancel synchronously revokes renderer proof and a late renderer completion cannot report success", async () => {
  const deferredFrame = deferred(); const f = sessionFixture(() => deferredFrame.promise);
  const value = snapshot("grey_hive", "gh_entry", 1);
  const pending = assert.rejects(f.session.render(value), error =>
    error instanceof SceneDefinitionSessionError && error.code === "E_SCENE_DEFINITION_CANCELLED");
  await flush(); f.session.cancel();
  assert.equal(f.invalidations(), 1); assert.equal(f.session.isReadyFor(value), false);
  await pending; // Cancellation does not wait for an uncooperative renderer/load.
  deferredFrame.resolve(); await flush();
  assert.equal(f.session.isReadyFor(value), false);
  await assert.rejects(f.session.render(value), /E_SCENE_DEFINITION_CANCELLED/);
});

test("a scene change while the renderer is awaited rejects the old frame even if its renderer resolves", async () => {
  const deferredFrame = deferred(); const f = sessionFixture(() => deferredFrame.promise);
  const first = snapshot("grey_hive", "gh_entry", 1), second = snapshot("grey_hive", "gh_power", 2);
  const pending = assert.rejects(f.session.render(first), /E_SCENE_DEFINITION_STALE_FRAME/);
  await flush(); f.session.acceptSnapshot(second); await pending;
  deferredFrame.resolve(); await flush();
  assert.equal(f.session.isReadyFor(first), false); assert.equal(f.session.isReadyFor(second), false);
});

test("renderer completion without a committed surface fails closed", async () => {
  const f = sessionFixture(); f.baseRenderer.isReadyFor = () => false;
  const value = snapshot("grey_hive", "gh_entry", 1);
  await assert.rejects(f.session.render(value), error =>
    error instanceof SceneDefinitionSessionError && error.code === "E_SCENE_DEFINITION_NOT_PRESENTED");
  assert.equal(f.session.isReadyFor(value), false);
});

test("an old prepare abort signal cannot cancel its replacement identity", async () => {
  const f = sessionFixture(); const abortOld = new AbortController();
  const first = snapshot("grey_hive", "gh_entry", 1), second = snapshot("grey_hive", "gh_power", 2);
  const old = assert.rejects(f.session.prepare(first, abortOld.signal), /E_SCENE_DEFINITION_STALE_FRAME/);
  f.session.acceptSnapshot(second); abortOld.abort(); await old;
  await f.session.render(second); assert.equal(f.session.isReadyFor(second), true);
  assert.equal(f.invalidations(), 0);
});

test("prepare cancellation is terminal even when the scene loader ignores abort", async () => {
  const pendingLoad = deferred(); let invalidations = 0, installed = 0;
  const renderer = { expectSceneIdentity() {}, invalidate() { invalidations++; }, setSceneDefinition() { installed++; } };
  const session = new SceneDefinitionSession(renderer, { loadForSnapshot: () => pendingLoad.promise });
  const value = snapshot("grey_hive", "gh_entry", 1), controller = new AbortController();
  const pending = assert.rejects(session.prepare(value, controller.signal), /E_SCENE_DEFINITION_CANCELLED/);
  controller.abort(); assert.equal(invalidations, 1); await pending;
  pendingLoad.resolve(scene()); await flush(); assert.equal(installed, 0);
});
