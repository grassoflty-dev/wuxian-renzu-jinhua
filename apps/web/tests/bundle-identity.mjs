import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  BUNDLE_IDENTITY_PATH,
  bundleIdentityPlugin,
  createBundleIdentity,
  isSafeBundlePath,
  serializeBundleIdentity,
  verifyBundleIdentity,
  writeBundleIdentity,
} from "../scripts/bundle-identity.mjs";
import { collectRuntimeBundle, EXPECTED_ASSET_OUTPUTS } from "../scripts/runtime-asset-bundle.mjs";
import { collectSceneDefinitionBundle } from "../scripts/scene-definition-bundle.mjs";

const repoRoot = resolve(fileURLToPath(new URL("../../..", import.meta.url)));
const fixtureIdentity = {
  schemaVersion: 1,
  gitSha: "a".repeat(40),
  appVersion: "1.0.0",
  contentVersion: "campaign-content-v1",
  saveV6SchemaVersion: 6,
  runtimeAssetManifestSha256: "b".repeat(64),
  sceneDefinitionManifestSha256: "c".repeat(64),
  builtAtUtc: "2026-09-27T00:00:00.000Z",
  hasTrackedDiff: false,
  hasUntrackedFiles: false,
  sourceTreeDirty: false,
};

async function fixture(files = {
  "assets/index-a1.js": "console.log('entry');\n",
  "assets/chunk-b2.js": "export const chunk = 2;\n",
  "assets/pixel.bin": Buffer.from([0, 1, 2, 255]),
}) {
  const root = await mkdtemp(join(tmpdir(), "wuxian-bundle-identity-"));
  for (const [path, value] of Object.entries(files)) {
    const fullPath = join(root, ...path.split("/"));
    await mkdir(dirname(fullPath), { recursive: true });
    await writeFile(fullPath, value);
  }
  return root;
}

function sha(bytes) { return createHash("sha256").update(bytes).digest("hex"); }

test("manifest sorting, serialization, and self-exclusion are deterministic and independently hashable", async () => {
  const first = await fixture({ "z-last.js": "last", "a-entry.js": "entry", "m.bin": "middle" });
  const second = await fixture({ "m.bin": "middle", "a-entry.js": "entry", "z-last.js": "last" });
  const firstSidecar = await createBundleIdentity(first, fixtureIdentity, "a-entry.js");
  const secondSidecar = await createBundleIdentity(second, fixtureIdentity, "a-entry.js");
  assert.deepEqual(firstSidecar.bytes, secondSidecar.bytes);
  assert.equal(firstSidecar.sha256, sha(firstSidecar.bytes));
  const parsed = JSON.parse(firstSidecar.bytes.toString("utf8"));
  assert.deepEqual(parsed.files.map(file => file.path), ["a-entry.js", "m.bin", "z-last.js"]);
  assert.equal(parsed.files.some(file => file.path === BUNDLE_IDENTITY_PATH), false);
  const written = await writeBundleIdentity(first, fixtureIdentity, "a-entry.js");
  const verified = await verifyBundleIdentity(first, fixtureIdentity);
  assert.equal(verified.sidecarSha256, sha(await readFile(join(first, BUNDLE_IDENTITY_PATH))));
  assert.equal(verified.sidecarSha256, written.sidecarSha256);
  assert.equal(verified.fileCount, 3);
});

test("product sidecar includes source-derived scenes and all 114 approved runtime outputs", async () => {
  const runtime = await collectRuntimeBundle(repoRoot);
  const scenes = await collectSceneDefinitionBundle(repoRoot);
  const fixtureEntry = { path: "assets/index-test.js", bytes: Buffer.from("fixture entry\n", "utf8") };
  const sourcePayload = [
    ...runtime.manifests,
    ...runtime.outputs,
    scenes.manifest,
    ...scenes.files,
    fixtureEntry,
  ];
  const files = sourcePayload.map(file => ({ path: file.path, sizeBytes: file.bytes.length, sha256: sha(file.bytes) }));
  const sidecarBytes = Buffer.from(serializeBundleIdentity({
    buildIdentity: fixtureIdentity,
    entryPath: fixtureEntry.path,
    files,
  }), "utf8");
  const manifest = JSON.parse(sidecarBytes.toString("utf8"));
  const paths = new Set(manifest.files.map(file => file.path));
  assert.equal(runtime.outputs.length, EXPECTED_ASSET_OUTPUTS);
  assert.equal(scenes.files.length, 30);
  for (const file of [...runtime.manifests, ...runtime.outputs]) assert.ok(paths.has(file.path), `missing runtime output ${file.path}`);
  assert.ok(paths.has(scenes.manifest.path));
  for (const file of scenes.files) assert.ok(paths.has(file.path), `missing scene ${file.path}`);
  assert.equal(manifest.files.length, paths.size);
  assert.equal(manifest.files.some(file => file.path === BUNDLE_IDENTITY_PATH), false);
  assert.equal(manifest.files.map(file => file.path).join("\n"), [...paths].sort().join("\n"));
  assert.equal(manifest.entry.sha256, manifest.files.find(file => file.path === manifest.entry.path)?.sha256);
  assert.equal(sha(sidecarBytes), sha(Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`, "utf8")));
  const sourceBytes = new Map(sourcePayload.map(file => [file.path, file.bytes]));
  for (const file of manifest.files) {
    const bytes = sourceBytes.get(file.path);
    assert.ok(bytes, `missing fixture payload ${file.path}`);
    assert.equal(bytes.length, file.sizeBytes, `wrong byte size ${file.path}`);
    assert.equal(sha(bytes), file.sha256, `wrong SHA-256 ${file.path}`);
  }
  assert.ok([...paths].every(path => !/^(?:assets\/source\/approved|chatgptimage|release-ui)(?:\/|$)/i.test(path)));
  assert.ok([...paths].every(path => !/(?:concept_only|\/reject\/|\/concept\/)/i.test(path)));
});

test("modified, missing, and extra payload files fail closed", async () => {
  const changed = await fixture();
  await writeBundleIdentity(changed, fixtureIdentity, "assets/index-a1.js");
  await writeFile(join(changed, "assets/index-a1.js"), "console.log('changed');\n");
  await assert.rejects(verifyBundleIdentity(changed), /E_BUNDLE_IDENTITY_FILE_SHA/);

  const missing = await fixture();
  await writeBundleIdentity(missing, fixtureIdentity, "assets/index-a1.js");
  const { unlink } = await import("node:fs/promises");
  await unlink(join(missing, "assets/pixel.bin"));
  await assert.rejects(verifyBundleIdentity(missing), /E_BUNDLE_IDENTITY_FILE_SET/);

  const extra = await fixture();
  await writeBundleIdentity(extra, fixtureIdentity, "assets/index-a1.js");
  await writeFile(join(extra, "late-file.js"), "extra");
  await assert.rejects(verifyBundleIdentity(extra), /E_BUNDLE_IDENTITY_FILE_SET/);
});

test("unsafe and duplicate paths, stale sidecars, and invalid entries fail closed", async () => {
  assert.equal(isSafeBundlePath("assets/main.js"), true);
  for (const path of ["../outside.js", "assets/../outside.js", "C:/outside.js", "/root.js", "a\\b.js", "a//b.js", "a/./b.js"]) {
    assert.equal(isSafeBundlePath(path), false, path);
  }
  const file = { path: "index.js", sizeBytes: 1, sha256: "d".repeat(64) };
  assert.throws(() => serializeBundleIdentity({ buildIdentity: fixtureIdentity, entryPath: "../index.js", files: [file] }), /E_BUNDLE_IDENTITY_ENTRY_PATH/);
  assert.throws(() => serializeBundleIdentity({ buildIdentity: fixtureIdentity, entryPath: "index.js", files: [file, file] }), /E_BUNDLE_IDENTITY_FILE_ENTRY/);
  assert.throws(() => serializeBundleIdentity({ buildIdentity: fixtureIdentity, entryPath: "wrong.js", files: [file] }), /E_BUNDLE_IDENTITY_ENTRY_MISSING/);

  const stale = await fixture();
  await writeBundleIdentity(stale, fixtureIdentity, "assets/index-a1.js");
  await assert.rejects(createBundleIdentity(stale, fixtureIdentity, "assets/index-a1.js"), /E_BUNDLE_IDENTITY_STALE_SIDECAR/);

  const malformed = await fixture();
  await writeBundleIdentity(malformed, fixtureIdentity, "assets/index-a1.js");
  await writeFile(join(malformed, BUNDLE_IDENTITY_PATH), "{broken json\n");
  await assert.rejects(verifyBundleIdentity(malformed), /E_BUNDLE_IDENTITY_SIDECAR_JSON/);

  const traversing = await fixture();
  await writeBundleIdentity(traversing, fixtureIdentity, "assets/index-a1.js");
  const sidecarPath = join(traversing, BUNDLE_IDENTITY_PATH);
  const sidecar = JSON.parse(await readFile(sidecarPath, "utf8"));
  sidecar.files[0].path = "../outside.js";
  await writeFile(sidecarPath, JSON.stringify(sidecar));
  await assert.rejects(verifyBundleIdentity(traversing), /E_BUNDLE_IDENTITY_ENTRY_PATH|E_BUNDLE_IDENTITY_FILE_ENTRY/);
});

test("filesystem links cannot escape the dist closure", async () => {
  const root = await fixture();
  await writeBundleIdentity(root, fixtureIdentity, "assets/index-a1.js");
  const outside = await mkdtemp(join(tmpdir(), "wuxian-bundle-outside-"));
  await writeFile(join(outside, "outside.js"), "outside");
  await symlink(outside, join(root, "outside-link"), "junction");
  await assert.rejects(verifyBundleIdentity(root), /E_BUNDLE_IDENTITY_SYMLINK/);
});

test("Vite plugin requires a clean output directory and exactly one entry chunk", async () => {
  const plugin = bundleIdentityPlugin();
  assert.throws(() => plugin.configResolved({ build: { emptyOutDir: false } }), /E_BUNDLE_IDENTITY_EMPTY_OUT_DIR_REQUIRED/);

  const output = await fixture({ "assets/index.js": "console.log('vite');\n" });
  const resolvedIdentity = { ...fixtureIdentity, gitSha: "e".repeat(40) };
  const configured = bundleIdentityPlugin();
  configured.configResolved({
    root: output,
    build: { emptyOutDir: true, outDir: output },
    define: { __BUILD_IDENTITY__: JSON.stringify(resolvedIdentity) },
  });
  await configured.writeBundle({ dir: output }, {
    "assets/index.js": { type: "chunk", isEntry: true, fileName: "assets/index.js" },
  });
  await assert.rejects(readFile(join(output, BUNDLE_IDENTITY_PATH)), { code: "ENOENT" });
  await configured.closeBundle.handler();
  const generated = JSON.parse(await readFile(join(output, BUNDLE_IDENTITY_PATH), "utf8"));
  assert.deepEqual(generated.buildIdentity, resolvedIdentity);

  const lateOutput = await fixture({ "assets/index.js": "console.log('vite');\n" });
  const lateFile = bundleIdentityPlugin();
  lateFile.configResolved({
    root: lateOutput,
    build: { emptyOutDir: true, outDir: lateOutput },
    define: { __BUILD_IDENTITY__: JSON.stringify(fixtureIdentity) },
  });
  lateFile.writeBundle({ dir: lateOutput }, {
    "assets/index.js": { type: "chunk", isEntry: true, fileName: "assets/index.js" },
  });
  await writeFile(join(lateOutput, "late-generated.js"), "late");
  await assert.rejects(lateFile.closeBundle.handler(), /E_BUNDLE_IDENTITY_OUTPUT_CLOSURE/);
  await assert.rejects(readFile(join(lateOutput, BUNDLE_IDENTITY_PATH)), { code: "ENOENT" });

  const emptyOutput = await fixture({});
  plugin.configResolved({
    build: { emptyOutDir: true, outDir: emptyOutput },
    define: { __BUILD_IDENTITY__: JSON.stringify(fixtureIdentity) },
  });
  assert.throws(() => plugin.writeBundle({ dir: emptyOutput }, {}), /E_BUNDLE_IDENTITY_ENTRY_COUNT/);

  const multipleEntries = bundleIdentityPlugin();
  multipleEntries.configResolved({
    build: { emptyOutDir: true, outDir: emptyOutput },
    define: { __BUILD_IDENTITY__: JSON.stringify(fixtureIdentity) },
  });
  assert.throws(() => multipleEntries.writeBundle({ dir: emptyOutput }, {
    "one.js": { type: "chunk", isEntry: true, fileName: "one.js" },
    "two.js": { type: "chunk", isEntry: true, fileName: "two.js" },
  }), /E_BUNDLE_IDENTITY_ENTRY_COUNT:2/);
});
