import { createHash } from "node:crypto";
import { lstat, readFile, readdir, realpath, writeFile } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";

export const BUNDLE_IDENTITY_PATH = "bundle-identity.json";
export const BUNDLE_IDENTITY_SCHEMA_ID = "native-web-bundle-identity/1";
const SHA40 = /^[0-9a-f]{40}$/;
const SHA64 = /^[0-9a-f]{64}$/;
const VERSION = /^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const CONTENT_VERSION = /^[A-Za-z0-9._+-]{1,96}$/;

function fail(code, detail = "") {
  throw new Error(detail ? `${code}:${detail}` : code);
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function isSafeBundlePath(value) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\\") || value.includes(":") ||
      value.includes("\0") || value.startsWith("/") || value !== value.normalize("NFC")) return false;
  const parts = value.split("/");
  return parts.every(part => part.length > 0 && part !== "." && part !== "..");
}

function normalizeBuildIdentity(identity) {
  if (!identity || typeof identity !== "object" || Array.isArray(identity)) fail("E_BUNDLE_IDENTITY_BUILD_IDENTITY");
  const keys = [
    "schemaVersion", "gitSha", "appVersion", "contentVersion", "saveV6SchemaVersion",
    "runtimeAssetManifestSha256", "sceneDefinitionManifestSha256", "builtAtUtc",
    "hasTrackedDiff", "hasUntrackedFiles", "sourceTreeDirty",
  ];
  if (Object.keys(identity).length !== keys.length || keys.some(key => !Object.hasOwn(identity, key)) ||
      identity.schemaVersion !== 1 || typeof identity.gitSha !== "string" || !SHA40.test(identity.gitSha) ||
      typeof identity.appVersion !== "string" || !VERSION.test(identity.appVersion) ||
      typeof identity.contentVersion !== "string" || !CONTENT_VERSION.test(identity.contentVersion) ||
      !Number.isSafeInteger(identity.saveV6SchemaVersion) || identity.saveV6SchemaVersion <= 0 ||
      typeof identity.runtimeAssetManifestSha256 !== "string" || !SHA64.test(identity.runtimeAssetManifestSha256) ||
      typeof identity.sceneDefinitionManifestSha256 !== "string" || !SHA64.test(identity.sceneDefinitionManifestSha256) ||
      typeof identity.builtAtUtc !== "string" || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/.test(identity.builtAtUtc) ||
      typeof identity.hasTrackedDiff !== "boolean" || typeof identity.hasUntrackedFiles !== "boolean" ||
      typeof identity.sourceTreeDirty !== "boolean" ||
      identity.sourceTreeDirty !== (identity.hasTrackedDiff || identity.hasUntrackedFiles)) {
    fail("E_BUNDLE_IDENTITY_BUILD_IDENTITY");
  }
  return Object.fromEntries(keys.map(key => [key, identity[key]]));
}

function resolveContained(root, relativePath) {
  if (!isSafeBundlePath(relativePath)) fail("E_BUNDLE_IDENTITY_PATH", String(relativePath));
  const outputPath = resolve(root, ...relativePath.split("/"));
  const relativeOutput = relative(root, outputPath);
  if (relativeOutput === ".." || relativeOutput.startsWith(`..${sep}`) || resolve(root, relativeOutput) !== outputPath) {
    fail("E_BUNDLE_IDENTITY_PATH_ESCAPE", relativePath);
  }
  return outputPath;
}

async function realDistRoot(distRoot) {
  const stat = await lstat(distRoot).catch(error => fail("E_BUNDLE_IDENTITY_DIST_MISSING", error?.code));
  if (!stat.isDirectory() || stat.isSymbolicLink()) fail("E_BUNDLE_IDENTITY_DIST_TYPE");
  return realpath(distRoot);
}

async function walkPayloadFiles(root, current = root) {
  const entries = await readdir(current, { withFileTypes: true });
  const files = [];
  for (const entry of entries.sort((a, b) => compare(a.name, b.name))) {
    const absolutePath = resolve(current, entry.name);
    const relativePath = relative(root, absolutePath).split(sep).join("/");
    if (!isSafeBundlePath(relativePath)) fail("E_BUNDLE_IDENTITY_PATH", relativePath);
    const stat = await lstat(absolutePath);
    if (stat.isSymbolicLink()) fail("E_BUNDLE_IDENTITY_SYMLINK", relativePath);
    if (stat.isDirectory()) {
      files.push(...await walkPayloadFiles(root, absolutePath));
      continue;
    }
    if (!stat.isFile()) fail("E_BUNDLE_IDENTITY_FILE_TYPE", relativePath);
    const resolvedPath = await realpath(absolutePath);
    const relativeResolved = relative(root, resolvedPath);
    if (relativeResolved === ".." || relativeResolved.startsWith(`..${sep}`) || resolve(root, relativeResolved) !== resolvedPath) {
      fail("E_BUNDLE_IDENTITY_SYMLINK_ESCAPE", relativePath);
    }
    if (relativePath !== BUNDLE_IDENTITY_PATH) {
      const bytes = await readFile(resolvedPath);
      files.push({ path: relativePath, sizeBytes: bytes.length, sha256: digest(bytes) });
    }
  }
  return files;
}

function normalizeFileEntries(files) {
  if (!Array.isArray(files)) fail("E_BUNDLE_IDENTITY_FILES");
  const seen = new Set();
  const normalized = files.map(file => {
    if (!file || typeof file !== "object" || Array.isArray(file) || !isSafeBundlePath(file.path) ||
        file.path === BUNDLE_IDENTITY_PATH || !Number.isSafeInteger(file.sizeBytes) || file.sizeBytes < 0 ||
        typeof file.sha256 !== "string" || !SHA64.test(file.sha256) || seen.has(file.path)) {
      fail("E_BUNDLE_IDENTITY_FILE_ENTRY", String(file?.path));
    }
    seen.add(file.path);
    return { path: file.path, sizeBytes: file.sizeBytes, sha256: file.sha256 };
  }).sort((left, right) => compare(left.path, right.path));
  return normalized;
}

export function serializeBundleIdentity({ buildIdentity, entryPath, files }) {
  const identity = normalizeBuildIdentity(buildIdentity);
  if (!isSafeBundlePath(entryPath) || !entryPath.toLowerCase().endsWith(".js")) fail("E_BUNDLE_IDENTITY_ENTRY_PATH");
  const normalizedFiles = normalizeFileEntries(files);
  const matches = normalizedFiles.filter(file => file.path === entryPath);
  if (matches.length !== 1) fail("E_BUNDLE_IDENTITY_ENTRY_MISSING", entryPath);
  const manifest = {
    schemaId: BUNDLE_IDENTITY_SCHEMA_ID,
    schemaVersion: 1,
    buildIdentity: identity,
    entry: { ...matches[0] },
    files: normalizedFiles,
  };
  return `${JSON.stringify(manifest, null, 2)}\n`;
}

export async function createBundleIdentity(distRoot, buildIdentity, entryPath) {
  const root = await realDistRoot(distRoot);
  const sidecarPath = resolveContained(root, BUNDLE_IDENTITY_PATH);
  try {
    await lstat(sidecarPath);
    fail("E_BUNDLE_IDENTITY_STALE_SIDECAR");
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
  const files = (await walkPayloadFiles(root)).sort((left, right) => compare(left.path, right.path));
  const bytes = Buffer.from(serializeBundleIdentity({ buildIdentity, entryPath, files }), "utf8");
  return { bytes, sha256: digest(bytes), fileCount: files.length, entryPath };
}

export async function writeBundleIdentity(distRoot, buildIdentity, entryPath) {
  const root = await realDistRoot(distRoot);
  const sidecar = await createBundleIdentity(root, buildIdentity, entryPath);
  const sidecarPath = resolveContained(root, BUNDLE_IDENTITY_PATH);
  await writeFile(sidecarPath, sidecar.bytes, { flag: "wx" });
  const verified = await verifyBundleIdentity(root, buildIdentity);
  return { ...sidecar, ...verified };
}

export async function verifyBundleIdentity(distRoot, expectedBuildIdentity) {
  const root = await realDistRoot(distRoot);
  const sidecarPath = resolveContained(root, BUNDLE_IDENTITY_PATH);
  const sidecarStat = await lstat(sidecarPath).catch(error => fail("E_BUNDLE_IDENTITY_SIDECAR_MISSING", error?.code));
  if (!sidecarStat.isFile() || sidecarStat.isSymbolicLink()) fail("E_BUNDLE_IDENTITY_SIDECAR_TYPE");
  const sidecarBytes = await readFile(sidecarPath);
  let manifest;
  try { manifest = JSON.parse(sidecarBytes.toString("utf8")); }
  catch { fail("E_BUNDLE_IDENTITY_SIDECAR_JSON"); }
  if (!manifest || typeof manifest !== "object" || Array.isArray(manifest) ||
      manifest.schemaId !== BUNDLE_IDENTITY_SCHEMA_ID || manifest.schemaVersion !== 1) fail("E_BUNDLE_IDENTITY_SCHEMA");
  const canonicalBytes = Buffer.from(serializeBundleIdentity({
    buildIdentity: manifest.buildIdentity,
    entryPath: manifest.entry?.path,
    files: manifest.files,
  }), "utf8");
  if (!canonicalBytes.equals(sidecarBytes)) fail("E_BUNDLE_IDENTITY_NONCANONICAL");
  if (expectedBuildIdentity &&
      !Buffer.from(JSON.stringify(normalizeBuildIdentity(manifest.buildIdentity))).equals(
        Buffer.from(JSON.stringify(normalizeBuildIdentity(expectedBuildIdentity))))) {
    fail("E_BUNDLE_IDENTITY_BUILD_MISMATCH");
  }
  if (!manifest.entry || manifest.entry.sizeBytes !== manifest.files.find(file => file.path === manifest.entry.path)?.sizeBytes ||
      manifest.entry.sha256 !== manifest.files.find(file => file.path === manifest.entry.path)?.sha256) {
    fail("E_BUNDLE_IDENTITY_ENTRY_RECORD");
  }

  const expectedFiles = normalizeFileEntries(manifest.files);
  const actualFiles = (await walkPayloadFiles(root)).sort((left, right) => compare(left.path, right.path));
  if (actualFiles.length !== expectedFiles.length) fail("E_BUNDLE_IDENTITY_FILE_SET", `expected=${expectedFiles.length}:actual=${actualFiles.length}`);
  for (let index = 0; index < expectedFiles.length; index++) {
    const expected = expectedFiles[index];
    const actual = actualFiles[index];
    if (actual.path !== expected.path) fail("E_BUNDLE_IDENTITY_FILE_SET", `expected=${expected.path}:actual=${actual.path}`);
    if (actual.sizeBytes !== expected.sizeBytes || actual.sha256 !== expected.sha256) {
      fail("E_BUNDLE_IDENTITY_FILE_SHA", expected.path);
    }
  }
  const entryPath = resolveContained(root, manifest.entry.path);
  const entryStat = await lstat(entryPath).catch(() => fail("E_BUNDLE_IDENTITY_ENTRY_MISSING", manifest.entry.path));
  if (!entryStat.isFile() || entryStat.isSymbolicLink()) fail("E_BUNDLE_IDENTITY_ENTRY_TYPE", manifest.entry.path);
  return {
    result: "pass",
    sidecarPath: BUNDLE_IDENTITY_PATH,
    sidecarSha256: digest(sidecarBytes),
    entry: { ...manifest.entry },
    fileCount: expectedFiles.length,
    filesSha256: digest(Buffer.from(expectedFiles.map(file => `${file.path}\0${file.sizeBytes}\0${file.sha256}\n`).join(""), "utf8")),
  };
}

export function bundleIdentityPlugin() {
  let resolved;
  let buildIdentity;
  let outputPaths;
  let entryPath;
  let wroteSidecar = false;
  return {
    name: "bundle-identity-sidecar-v1",
    apply: "build",
    enforce: "post",
    configResolved(config) {
      resolved = config;
      if (config.build.emptyOutDir !== true) fail("E_BUNDLE_IDENTITY_EMPTY_OUT_DIR_REQUIRED");
      const rawIdentity = config.define?.__BUILD_IDENTITY__;
      if (typeof rawIdentity !== "string") fail("E_BUNDLE_IDENTITY_BUILD_DEFINE_MISSING");
      try { buildIdentity = normalizeBuildIdentity(JSON.parse(rawIdentity)); }
      catch (error) {
        if (error instanceof SyntaxError) fail("E_BUNDLE_IDENTITY_BUILD_DEFINE_JSON");
        throw error;
      }
    },
    writeBundle(outputOptions, bundle) {
      if (!resolved || !buildIdentity) fail("E_BUNDLE_IDENTITY_NOT_CONFIGURED");
      if (outputPaths) fail("E_BUNDLE_IDENTITY_MULTIPLE_OUTPUTS_UNSUPPORTED");
      const entryChunks = Object.values(bundle).filter(output => output.type === "chunk" && output.isEntry);
      if (entryChunks.length !== 1) fail("E_BUNDLE_IDENTITY_ENTRY_COUNT", String(entryChunks.length));
      entryPath = entryChunks[0].fileName;
      if (!isSafeBundlePath(entryPath) || !entryPath.toLowerCase().endsWith(".js")) fail("E_BUNDLE_IDENTITY_ENTRY_PATH");
      const distRoot = resolve(resolved.root, outputOptions.dir || resolved.build.outDir);
      const expectedRoot = resolve(resolved.root, resolved.build.outDir);
      if (distRoot !== expectedRoot) fail("E_BUNDLE_IDENTITY_OUTPUT_DIR", distRoot);

      outputPaths = Object.keys(bundle).map(path => {
        if (!isSafeBundlePath(path)) fail("E_BUNDLE_IDENTITY_OUTPUT_PATH", path);
        return path;
      }).sort(compare);
      if (new Set(outputPaths).size !== outputPaths.length) fail("E_BUNDLE_IDENTITY_OUTPUT_DUPLICATE");
    },
    closeBundle: {
      order: "post",
      async handler() {
        if (!resolved || !buildIdentity || !outputPaths || !entryPath) fail("E_BUNDLE_IDENTITY_OUTPUT_NOT_RECORDED");
        if (wroteSidecar) fail("E_BUNDLE_IDENTITY_MULTIPLE_OUTPUTS_UNSUPPORTED");
        const distRoot = resolve(resolved.root, resolved.build.outDir);
        const generatedFiles = (await walkPayloadFiles(await realDistRoot(distRoot))).sort((left, right) => compare(left.path, right.path));
        if (generatedFiles.length !== outputPaths.length || generatedFiles.some((file, index) => file.path !== outputPaths[index])) {
          fail("E_BUNDLE_IDENTITY_OUTPUT_CLOSURE", `bundle=${outputPaths.length}:dist=${generatedFiles.length}`);
        }
        const result = await writeBundleIdentity(distRoot, buildIdentity, entryPath);
        wroteSidecar = true;
        if (result.entry.path !== entryPath) fail("E_BUNDLE_IDENTITY_ENTRY_MISMATCH");
      },
    },
  };
}
