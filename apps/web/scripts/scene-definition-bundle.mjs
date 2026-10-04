import { createHash } from "node:crypto";
import { lstat, readFile, readdir, realpath } from "node:fs/promises";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = resolve(SCRIPT_DIR, "../../..");
export const SOURCE_RELATIVE = "content/scenes/compiled";
export const MANIFEST_PATH = "scene-definitions/SCENE_DEFINITION_MANIFEST.json";
const WORLD_IDS = new Set(["grey_hive", "mist_harbor", "clockworks", "return_station"]);
const ID_PATTERN = /^[A-Za-z0-9_.:-]{1,128}$/;

function fail(code, detail = "") {
  throw new Error(detail ? `${code}:${detail}` : code);
}
function digest(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
function compare(left, right) { return left < right ? -1 : left > right ? 1 : 0; }
async function walkCompiled(root, current = root) {
  const entries = await readdir(current, { withFileTypes: true });
  const files = [];
  for (const entry of entries.sort((a, b) => compare(a.name, b.name))) {
    if (entry.isSymbolicLink()) fail("E_SCENE_DEFINITION_SOURCE_SYMLINK", relative(root, resolve(current, entry.name)));
    const path = resolve(current, entry.name);
    if (entry.isDirectory()) fail("E_SCENE_DEFINITION_SOURCE_NESTED", relative(root, path));
    if (!entry.isFile() || !entry.name.endsWith(".json")) fail("E_SCENE_DEFINITION_SOURCE_FILE", relative(root, path));
    files.push({ path, fileName: entry.name });
  }
  return files;
}

export async function collectSceneDefinitionBundle(repoRoot = REPO_ROOT) {
  const resolvedRepoRoot = await realpath(repoRoot);
  let checkedRoot = resolvedRepoRoot;
  for (const part of SOURCE_RELATIVE.split("/")) {
    checkedRoot = resolve(checkedRoot, part);
    let sourceStat;
    try { sourceStat = await lstat(checkedRoot); }
    catch (error) {
      if (error?.code === "ENOENT") {
        const manifestBytes = Buffer.from(`${JSON.stringify({ schemaVersion: 1, scenes: [] }, null, 2)}\n`, "utf8");
        return { manifest: { path: MANIFEST_PATH, bytes: manifestBytes, sha256: digest(manifestBytes) }, files: [] };
      }
      throw error;
    }
    if (sourceStat.isSymbolicLink()) fail("E_SCENE_DEFINITION_SOURCE_SYMLINK", relative(resolvedRepoRoot, checkedRoot));
    if (!sourceStat.isDirectory()) fail("E_SCENE_DEFINITION_SOURCE_TYPE", relative(resolvedRepoRoot, checkedRoot));
  }
  const sourceCandidate = checkedRoot;
  const sourceRoot = await realpath(sourceCandidate);
  const sourceRelative = relative(resolvedRepoRoot, sourceRoot);
  if (resolve(resolvedRepoRoot, SOURCE_RELATIVE) !== sourceRoot || sourceRelative === ".." ||
      sourceRelative.startsWith(`..${sep}`) || resolve(resolvedRepoRoot, sourceRelative) !== sourceRoot) {
    fail("E_SCENE_DEFINITION_SOURCE_ESCAPE", SOURCE_RELATIVE);
  }

  const files = [];
  const identities = new Set();
  for (const file of await walkCompiled(sourceRoot)) {
    const bytes = await readFile(file.path);
    let scene;
    try { scene = JSON.parse(bytes.toString("utf8")); }
    catch { fail("E_SCENE_DEFINITION_SOURCE_JSON", file.fileName); }
    if (!scene || typeof scene !== "object" || Array.isArray(scene) || scene.schemaVersion !== 1 ||
        typeof scene.worldId !== "string" || !WORLD_IDS.has(scene.worldId) ||
        typeof scene.sceneId !== "string" || !ID_PATTERN.test(scene.sceneId) || file.fileName !== `${scene.sceneId}.json`) {
      fail("E_SCENE_DEFINITION_SOURCE_SCHEMA", file.fileName);
    }
    const identity = `${scene.worldId}\0${scene.sceneId}`;
    if (identities.has(identity)) fail("E_SCENE_DEFINITION_SOURCE_DUPLICATE", `${scene.worldId}:${scene.sceneId}`);
    identities.add(identity);
    const outputPath = `scene-definitions/compiled/${scene.worldId}/${scene.sceneId}.json`;
    files.push({ path: outputPath, bytes, sha256: digest(bytes), sourcePath: file.path, worldId: scene.worldId, sceneId: scene.sceneId });
  }
  files.sort((a, b) => compare(a.worldId, b.worldId) || compare(a.sceneId, b.sceneId));
  const manifestBytes = Buffer.from(`${JSON.stringify({
    schemaVersion: 1,
    scenes: files.map(({ worldId, sceneId, path, sha256 }) => ({ worldId, sceneId, path, sha256 })),
  }, null, 2)}\n`, "utf8");
  return { manifest: { path: MANIFEST_PATH, bytes: manifestBytes, sha256: digest(manifestBytes) }, files };
}

export function sceneDefinitionBundlePlugin(options = {}) {
  const repoRoot = options.repoRoot ?? REPO_ROOT;
  const collect = options.collectBundle ?? collectSceneDefinitionBundle;
  let payload;
  return {
    name: "scene-definition-bundle-v1",
    enforce: "pre",
    async config() {
      payload = await collect(repoRoot);
      return { define: { __SCENE_DEFINITION_MANIFEST_SHA256__: JSON.stringify(payload.manifest.sha256) } };
    },
    async buildStart() {
      const latest = await collect(repoRoot);
      if (payload && latest.manifest.sha256 !== payload.manifest.sha256) fail("E_SCENE_DEFINITION_SOURCE_CHANGED_DURING_BUILD");
      payload = latest;
      for (const file of payload.files) this.addWatchFile(file.sourcePath);
    },
    generateBundle() {
      if (!payload) fail("E_SCENE_DEFINITION_BUNDLE_NOT_VALIDATED");
      this.emitFile({ type: "asset", fileName: payload.manifest.path, source: payload.manifest.bytes });
      for (const file of payload.files) this.emitFile({ type: "asset", fileName: file.path, source: file.bytes });
    },
  };
}

export async function verifySceneDefinitionDist(repoRoot = REPO_ROOT, distRoot = resolve(repoRoot, "apps/web/dist")) {
  const payload = await collectSceneDefinitionBundle(repoRoot);
  const resolvedDist = await realpath(distRoot);
  const expected = [payload.manifest, ...payload.files];
  const tree = createHash("sha256");
  let totalBytes = 0;
  for (const file of expected) {
    const outputPath = resolve(resolvedDist, ...file.path.split("/"));
    const resolvedOutput = await realpath(outputPath).catch(() => fail("E_SCENE_DEFINITION_DIST_MISSING", file.path));
    const relativeOutput = relative(resolvedDist, resolvedOutput);
    if (relativeOutput === ".." || relativeOutput.startsWith(`..${sep}`) || resolve(resolvedDist, relativeOutput) !== resolvedOutput) {
      fail("E_SCENE_DEFINITION_DIST_ESCAPE", file.path);
    }
    const bytes = await readFile(resolvedOutput);
    const actual = digest(bytes);
    if (actual !== file.sha256) fail("E_SCENE_DEFINITION_DIST_SHA", `${file.path}:expected=${file.sha256}:actual=${actual}`);
    totalBytes += bytes.length;
    tree.update(`${file.path}\0${actual}\n`);
  }
  return { result: "pass", manifestPath: payload.manifest.path, manifestSha256: payload.manifest.sha256,
    sceneCount: payload.files.length, sceneBytes: payload.files.reduce((sum, file) => sum + file.bytes.length, 0),
    verifiedFiles: expected.length, verifiedBytes: totalBytes, sceneFilesSha256: tree.digest("hex") };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = process.argv[2] === "verify-dist"
      ? await verifySceneDefinitionDist()
      : await collectSceneDefinitionBundle().then(payload => ({
        result: "pass", sceneCount: payload.files.length, manifestSha256: payload.manifest.sha256,
        sceneBytes: payload.files.reduce((sum, file) => sum + file.bytes.length, 0),
      }));
    console.log(JSON.stringify(result));
  } catch (error) {
    console.error(`SCENE DEFINITION BUNDLE FAIL CLOSED: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  }
}
