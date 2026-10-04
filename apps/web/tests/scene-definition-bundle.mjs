import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { createHash } from "node:crypto";
import { collectSceneDefinitionBundle, MANIFEST_PATH, sceneDefinitionBundlePlugin } from "../scripts/scene-definition-bundle.mjs";

const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const definition = (worldId, sceneId) => ({ schemaVersion: 1, worldId, sceneId, boundsM: { x: 0, z: 0, width: 3, depth: 3 }, presentation: { layers: [], sprites: [] } });

test("missing production compiled-scene directory yields a deterministic empty manifest", async () => {
  const root = await mkdtemp(resolve(tmpdir(), "scene-bundle-empty-"));
  try {
    const payload = await collectSceneDefinitionBundle(root);
    assert.equal(payload.files.length, 0);
    assert.equal(payload.manifest.path, MANIFEST_PATH);
    assert.deepEqual(JSON.parse(payload.manifest.bytes.toString("utf8")), { schemaVersion: 1, scenes: [] });
    assert.equal(payload.manifest.sha256, sha(payload.manifest.bytes));
  } finally { await rm(root, { recursive: true, force: true }); }
});

test("bundler sorts compiler outputs by identity, pins file hashes, and emits only canonical compiled paths", async () => {
  const root = await mkdtemp(resolve(tmpdir(), "scene-bundle-prod-"));
  try {
    const source = resolve(root, "content/scenes/compiled");
    await mkdir(source, { recursive: true });
    const bytesB = Buffer.from(`${JSON.stringify(definition("clockworks", "cw_entry"), null, 2)}\n`);
    const bytesA = Buffer.from(`${JSON.stringify(definition("grey_hive", "gh_entry"), null, 2)}\n`);
    await writeFile(resolve(source, "cw_entry.json"), bytesB);
    await writeFile(resolve(source, "gh_entry.json"), bytesA);

    const first = await collectSceneDefinitionBundle(root);
    const second = await collectSceneDefinitionBundle(root);
    assert.deepEqual(first.manifest.bytes, second.manifest.bytes);
    assert.deepEqual(first.files.map(file => file.path), [
      "scene-definitions/compiled/clockworks/cw_entry.json",
      "scene-definitions/compiled/grey_hive/gh_entry.json",
    ]);
    assert.deepEqual(first.files.map(file => file.sha256), [sha(bytesB), sha(bytesA)]);

    const plugin = sceneDefinitionBundlePlugin({ repoRoot: root });
    const config = await plugin.config();
    assert.equal(config.define.__SCENE_DEFINITION_MANIFEST_SHA256__, JSON.stringify(first.manifest.sha256));
    const watched = [];
    await plugin.buildStart.call({ addWatchFile: path => watched.push(path) });
    const emitted = [];
    plugin.generateBundle.call({ emitFile: file => emitted.push(file) });
    assert.deepEqual(emitted.map(file => file.fileName), [MANIFEST_PATH, ...first.files.map(file => file.path)]);
    assert.deepEqual(emitted.map(file => Buffer.from(file.source)), [first.manifest.bytes, ...first.files.map(file => file.bytes)]);
    assert.equal(watched.length, 2);
  } finally { await rm(root, { recursive: true, force: true }); }
});

test("compiler output identity/path mismatches, nested directories, and malformed JSON fail closed", async () => {
  const root = await mkdtemp(resolve(tmpdir(), "scene-bundle-invalid-"));
  const source = resolve(root, "content/scenes/compiled");
  try {
    await mkdir(source, { recursive: true });
    await writeFile(resolve(source, "wrong_name.json"), JSON.stringify(definition("grey_hive", "gh_real")));
    await assert.rejects(collectSceneDefinitionBundle(root), /E_SCENE_DEFINITION_SOURCE_SCHEMA/);
    await rm(resolve(source, "wrong_name.json"));

    await writeFile(resolve(source, "gh_real.json"), "{");
    await assert.rejects(collectSceneDefinitionBundle(root), /E_SCENE_DEFINITION_SOURCE_JSON/);
    await rm(resolve(source, "gh_real.json"));

    await mkdir(resolve(source, "nested"));
    await assert.rejects(collectSceneDefinitionBundle(root), /E_SCENE_DEFINITION_SOURCE_NESTED/);
  } finally { await rm(root, { recursive: true, force: true }); }
});
