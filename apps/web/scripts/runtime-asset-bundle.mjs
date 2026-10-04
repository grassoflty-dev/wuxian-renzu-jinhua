import { createHash } from "node:crypto";
import { readFile, readdir, realpath } from "node:fs/promises";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { collectSceneDefinitionBundle } from "./scene-definition-bundle.mjs";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = resolve(SCRIPT_DIR, "../../..");
export const B0_AUTHORITY_PATH = "governance/assets/AI_ASSET_RELEASE_MANIFEST.json";
export const RUNTIME_MANIFEST_PATH = "governance/assets/RUNTIME_ASSET_MANIFEST.json";
export const CENYAO_IDLE_METADATA_PATH = "assets/metadata/cenyao-runtime-master-v1/cenyao-idle-atlas-v1.json";
export const CENYAO_IDLE_METADATA_SHA256 = "a1d5f40c2a7ae47f45dbf2b85432291be3be0badb855f1f386ec9905d533641c";
export const CENYAO_MASTER_SHA256 = "80a810ec6fdbcda50a0daeebcf53cd2173c0a411f043861e869187061cb59f61";
export const CENYAO_ATLAS_PNG_PATH = "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.png";
export const CENYAO_ATLAS_WEBP_PATH = "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp";
export const CENYAO_ATLAS_PNG_SHA256 = "2146eeedc2945149fbee98a4f24739bff6cefeeef548e0bdace37fe8217028fe";
export const CENYAO_ATLAS_WEBP_SHA256 = "b0cc05697fdb89a3f55fdbce2bd558e6800a088f1147f7f8edce8fad842c5af0";
export const B0_AUTHORITY_SHA256 = "c644b5953e9265d45796dfb8a6dad1c0fd8abfbffb3eb4ace84987d8238cee2d";
export const RUNTIME_MANIFEST_SHA256 = "081bebad5eede90e6ed50f2f7cacb930a67f2295961d405f58141d23874ec414";
export const SOURCE_COMMIT = "030eb9bf8194ad5534dc77315567ff339cd2c625";
export const EXPECTED_ASSET_OUTPUTS = 114;
const EXPECTED_ASSETS = 43;
const EXPECTED_PAGES = 14;
const CENYAO_DIRECTIONS = ["south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east"];
const CENYAO_FRAME_HASHES = {
  south: ["b6cff22dabb303f0a9c95ba24a41fb3c2f5159ff191adefacf0c6dad38ffe875", "627292d979d0a8a3334dec47630ad9ac531f023a87140f52edcc9ce523ffe433"],
  south_west: ["3d3e34d0a5d388de522fd6a94fd2769efbf8137aea1ae7c0b69881c5416520be", "c5f2eddc15e71e0ca388ba55b17b15d53fe4df262dbc3cad7055cbf939591ea3"],
  west: ["fc8e0b132c48f594a49a20bd39955d731f02077502e33131499608cc3bd7a70c", "a33a153ba557b47f8460732895a8b2d819a39f733607a9332d4eb2d4d553e1f1"],
  north_east: ["19b8e318bf5216d6075f64ac986ef7c3c3299241816b472843aac24bbf4a7676", "464b2b653711a44cabcfa400004c78ff32bf5fd39c810a14cea334e14c3c6f38"],
  north: ["6608ed34685f28f3380849d7fe5e5319078e8feb1fbd66cbcd4cb3782a1a8221", "ca89a1ee487252dcf1eeb0ad9e9d25abec10ae61882374a17839ecff2b5de770"],
  north_west: ["8dc527b975b1da5ed7b36a54cf4ad260cdcd9d7d3ccba24660308cf971c3164e", "fec4be62fc81d025a18a1248803355c590bb9fe31234f3b757eb2328c11a0cf5"],
  east: ["fd38db1daec829c0a7c2adffcdfd5c605f54b3611bb515599cd8deeea1f47402", "daad53355ccfd7262f2c102381e665e07918c2e4352523d43a020bbed272605a"],
  south_east: ["bfe0107fe1efdcd73b90d57b01283a22658960b5847bb5e88b401dae9857be56", "7126d9cf50ef316f990e1d8b0eab4f7538425fa92cee4a3fcee03cb0c0e01448"],
};
const SHA_RE = /^[0-9a-f]{64}$/;
const CATEGORY_LIMIT = { actor: 4096, prop: 4096, vfx: 4096, boss: 4096, ui: 2048, world: 4096 };

function fail(code, detail = "") {
  throw new Error(detail ? `${code}:${detail}` : code);
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function parseJson(bytes, code) {
  try { return JSON.parse(bytes.toString("utf8")); } catch { return fail(code); }
}

function safeRelativePath(value, prefix) {
  if (typeof value !== "string" || !value.startsWith(prefix) || value.includes("\\") || value.includes(":") || value.startsWith("/")) return false;
  const parts = value.split("/");
  return parts.every(part => part.length > 0 && part !== "." && part !== "..");
}

function isSha(value) { return typeof value === "string" && SHA_RE.test(value); }

function validateAuthority(authorityBytes) {
  if (digest(authorityBytes) !== B0_AUTHORITY_SHA256) fail("E_B0_MANIFEST_SHA");
  const value = parseJson(authorityBytes, "E_B0_MANIFEST_JSON");
  if (value.schemaId !== "ai-asset-release-manifest/1" || value.schemaVersion !== 1 ||
      value.decisionSource?.path !== "deep-research-report.md" || value.decisionSource?.sha256 !== "fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1" ||
      value.sourceTree?.commit !== SOURCE_COMMIT || value.sourceTree?.prefix !== "chatgptimage/wuxian-renzu-jinhua_2d_asset_pack_10_batches_2026-09-25/" ||
      value.counts?.rows !== 94 || value.counts?.release_approved !== 43 || value.counts?.concept_only !== 30 || value.counts?.reject !== 21 ||
      !Array.isArray(value.assets) || value.assets.length !== 94) fail("E_B0_MANIFEST_SCHEMA");
  const rows = new Map();
  const paths = new Set();
  const counts = { release_approved: 0, concept_only: 0, reject: 0 };
  for (const row of value.assets) {
    if (!row || typeof row.asset_id !== "string" || !row.asset_id || !isSha(row.sha256) ||
        !/^batch(?:0[1-9]|10)\/[^/]+$/.test(row.relative_path) || !(row.release_status in counts) || rows.has(row.asset_id) || paths.has(row.relative_path)) {
      fail("E_B0_ASSET_ROW");
    }
    rows.set(row.asset_id, row);
    paths.add(row.relative_path);
    counts[row.release_status]++;
  }
  if (counts.release_approved !== 43 || counts.concept_only !== 30 || counts.reject !== 21) fail("E_B0_STATUS_COUNTS");
  return { bytes: authorityBytes, value, rows };
}

function validateRuntimeManifest(runtimeBytes, authority, b6aMetadataBytes, verifyPinnedSha = true) {
  if (verifyPinnedSha && digest(runtimeBytes) !== RUNTIME_MANIFEST_SHA256) fail("E_RUNTIME_MANIFEST_SHA");
  const value = parseJson(runtimeBytes, "E_RUNTIME_MANIFEST_JSON");
  const b6aMetadata = parseJson(b6aMetadataBytes, "E_CENYAO_B6A_METADATA_JSON");
  if (digest(b6aMetadataBytes) !== CENYAO_IDLE_METADATA_SHA256 || b6aMetadata.schemaVersion !== "cenyao-idle-atlas-v1" ||
      b6aMetadata.sourceMaster?.sha256 !== CENYAO_MASTER_SHA256 || b6aMetadata.atlas?.png?.sha256 !== CENYAO_ATLAS_PNG_SHA256 ||
      b6aMetadata.atlas?.webp?.sha256 !== CENYAO_ATLAS_WEBP_SHA256 || b6aMetadata.frames?.map(frame => frame.direction).join(",") !== CENYAO_DIRECTIONS.join(",")) fail("E_CENYAO_B6A_METADATA_SHA");
  if (value.schemaId !== "runtime-asset-manifest/1" || value.manifestState !== "ADOPT" ||
      value.authority?.path !== B0_AUTHORITY_PATH || value.authority?.sourceCommit !== SOURCE_COMMIT ||
      !Array.isArray(value.assets) || value.assets.length !== EXPECTED_ASSETS ||
      !Array.isArray(value.pendingAssets) || value.pendingAssets.length !== 0 ||
      !Array.isArray(value.atlases) || value.atlases.length !== EXPECTED_PAGES ||
      value.residency?.budgetMiB !== 220 || typeof value.residency?.p95MiB !== "number" || value.residency.p95MiB > 220) {
    fail("E_RUNTIME_MANIFEST_SCHEMA");
  }

  const approved = new Map([...authority.rows].filter(([, row]) => row.release_status === "release_approved"));
  const seenAssets = new Set();
  const seenPages = new Set();
  const seenOutputPaths = new Set();
  const outputs = [];
  const pageByIndex = new Map();
  for (let index = 0; index < value.atlases.length; index++) {
    const page = value.atlases[index];
    if (!page || page.pageIndex !== index || seenPages.has(page.pageIndex) || !(page.category in CATEGORY_LIMIT) ||
        typeof page.streamingGroup !== "string" || !page.streamingGroup ||
        !(index === 13 ? page.pngPath === CENYAO_ATLAS_PNG_PATH && page.webpPath === CENYAO_ATLAS_WEBP_PATH && page.pngSha256 === CENYAO_ATLAS_PNG_SHA256 && page.webpSha256 === CENYAO_ATLAS_WEBP_SHA256 : safeRelativePath(page.pngPath, "assets/derived/asset-pipeline-v1/") && safeRelativePath(page.webpPath, "assets/derived/asset-pipeline-v1/")) ||
        !isSha(page.pngSha256) || !isSha(page.webpSha256) || !Array.isArray(page.size) || page.size.length !== 2 || page.size.some(n => !Number.isInteger(n) || n <= 0) ||
        page.maxSide !== CATEGORY_LIMIT[page.category]) fail("E_RUNTIME_ATLAS_SCHEMA", String(index));
    const [width, height] = page.size;
    const limit = CATEGORY_LIMIT[page.category];
    if (width > limit || height > limit) fail("E_RUNTIME_ATLAS_BUDGET", page.pngPath);
    if (index === 13 && (page.category !== "actor" || page.streamingGroup !== "shared" || width !== 1360 || height !== 1128)) fail("E_RUNTIME_CENYAO_ATLAS");
    if (page.category === "boss" && page.streamingGroup !== "sentinel") fail("E_RUNTIME_BOSS_GROUP", page.pngPath);
    const decodedMiB = Math.round(width * height * 4 / (1024 * 1024) * 1000) / 1000;
    if (typeof page.decodedRgbaMiB !== "number" || Math.abs(page.decodedRgbaMiB - decodedMiB) > 0.002) fail("E_RUNTIME_ATLAS_SIZE", page.pngPath);
    addOutput(outputs, seenOutputPaths, page.pngPath, page.pngSha256, "atlas/png");
    addOutput(outputs, seenOutputPaths, page.webpPath, page.webpSha256, "atlas/webp");
    seenPages.add(page.pageIndex);
    pageByIndex.set(page.pageIndex, page);
  }
  if (![...value.atlases].some(page => page.category === "boss")) fail("E_RUNTIME_BOSS_PAGE_MISSING");

  const placements = new Map();
  for (const asset of value.assets) {
    const approvedRow = approved.get(asset.assetId);
    if (!approvedRow || seenAssets.has(asset.assetId)) fail("E_RUNTIME_ASSET_NOT_APPROVED", asset.assetId);
    if (asset.admission !== "release_approved" || asset.runtimeStatus !== "ADOPT" ||
        asset.sourcePath !== approvedRow.relative_path || asset.sourceSha256 !== approvedRow.sha256 || !isSha(asset.sourceSha256)) {
      fail("E_RUNTIME_ASSET_SOURCE", asset.assetId);
    }
    if (!(asset.category in CATEGORY_LIMIT) || typeof asset.streamingGroup !== "string" || !Number.isInteger(asset.atlasPage) || !Array.isArray(asset.atlasFrame) ||
        asset.atlasFrame.length !== 4 || asset.atlasFrame.some(n => !Number.isInteger(n) || n < 0) || asset.atlasFrame[2] <= 0 || asset.atlasFrame[3] <= 0) {
      fail("E_RUNTIME_ASSET_SCHEMA", asset.assetId);
    }
    const page = pageByIndex.get(asset.atlasPage);
    if (!page || page.category !== asset.category || page.streamingGroup !== asset.streamingGroup) fail("E_RUNTIME_ASSET_PAGE", asset.assetId);
    const [x, y, width, height] = asset.atlasFrame;
    if (x + width > page.size[0] || y + height > page.size[1]) fail("E_RUNTIME_FRAME_BOUNDS", asset.assetId);
    const prior = placements.get(asset.atlasPage) ?? [];
    if (prior.some(([px, py, pw, ph]) => x < px + pw && px < x + width && y < py + ph && py < y + height)) fail("E_RUNTIME_FRAME_OVERLAP", asset.assetId);
    prior.push([x, y, width, height]);
    placements.set(asset.atlasPage, prior);
    if (!asset.outputs || Object.keys(asset.outputs).sort().join(",") !== "png,webp") fail("E_RUNTIME_OUTPUT_SCHEMA", asset.assetId);
    for (const format of ["png", "webp"]) {
      const output = asset.outputs[format];
      if (!output || !safeRelativePath(output.path, "assets/derived/asset-pipeline-v1/") || !isSha(output.sha256)) fail("E_RUNTIME_OUTPUT_SCHEMA", `${asset.assetId}:${format}`);
      addOutput(outputs, seenOutputPaths, output.path, output.sha256, `sprite/${format}`);
    }
    if (asset.assetId === "runtime2d.actor.cenyao.base.v1") {
      const animation = asset.animation;
      if (!animation || animation.sourceMetadata?.path !== CENYAO_IDLE_METADATA_PATH || animation.sourceMetadata?.sha256 !== CENYAO_IDLE_METADATA_SHA256 ||
          animation.sourceMasterSha256 !== CENYAO_MASTER_SHA256 || animation.sourceAsset?.assetId !== asset.assetId ||
          animation.sourceAsset?.path !== asset.sourcePath || animation.sourceAsset?.sha256 !== asset.sourceSha256 ||
          animation.directions?.join(",") !== CENYAO_DIRECTIONS.join(",") || animation.states?.join(",") !== "idle" || animation.frames?.length !== 8) fail("E_RUNTIME_CENYAO_ANIMATION");
      for (let frameIndex = 0; frameIndex < 8; frameIndex++) {
        const frame = animation.frames[frameIndex];
        const sourceFrame = b6aMetadata.frames[frameIndex];
        const expectedExtruded = [sourceFrame.rect[0] - 2, sourceFrame.rect[1] - 2, sourceFrame.rect[2] + 4, sourceFrame.rect[3] + 4];
        if (frame.direction !== sourceFrame.direction || frame.state !== "idle" || frame.atlasPage !== 13 ||
            JSON.stringify(frame.atlasFrame) !== JSON.stringify(sourceFrame.rect) || JSON.stringify(frame.extrudedFrame) !== JSON.stringify(sourceFrame.extrudedRect) ||
            JSON.stringify(frame.extrudedFrame) !== JSON.stringify(expectedExtruded) || JSON.stringify(frame.anchor) !== JSON.stringify([sourceFrame.anchor.x, sourceFrame.anchor.y]) ||
            JSON.stringify(frame.sourceFrame) !== JSON.stringify(sourceFrame.sourceFrame)) fail("E_RUNTIME_CENYAO_FRAME", frame.direction);
        for (const format of ["png", "webp"]) {
          const output = frame.sourceFrame[format];
          if (!output || output.sha256 !== CENYAO_FRAME_HASHES[frame.direction]?.[format === "png" ? 0 : 1] ||
              !safeRelativePath(output.path, `assets/derived/cenyao-runtime-master-v1/${format}/`)) fail("E_RUNTIME_CENYAO_FRAME_SOURCE", `${frame.direction}:${format}`);
        }
        const [fx, fy, fw, fh] = frame.atlasFrame;
        if (placements.get(13)?.some(([px, py, pw, ph]) => fx < px + pw && px < fx + fw && fy < py + ph && py < fy + fh)) fail("E_RUNTIME_FRAME_OVERLAP", `cenyao:${frame.direction}`);
        const pagePlacements = placements.get(13) ?? [];
        pagePlacements.push([fx, fy, fw, fh]);
        placements.set(13, pagePlacements);
      }
    } else if (asset.animation) fail("E_RUNTIME_CENYAO_ANIMATION");
    seenAssets.add(asset.assetId);
  }
  if (seenAssets.size !== EXPECTED_ASSETS || [...approved.keys()].some(id => !seenAssets.has(id))) fail("E_RUNTIME_APPROVED_SET");
  if (outputs.length !== EXPECTED_ASSET_OUTPUTS) fail("E_RUNTIME_OUTPUT_COUNT", String(outputs.length));
  if (value.atlases.some(page => !placements.has(page.pageIndex))) fail("E_RUNTIME_UNUSED_ATLAS_PAGE");
  const bossAssetPages = new Set(value.assets.filter(asset => asset.category === "boss").map(asset => asset.atlasPage));
  if (bossAssetPages.size !== 1 || [...bossAssetPages].some(pageIndex => pageByIndex.get(pageIndex)?.category !== "boss")) fail("E_RUNTIME_BOSS_PAGE");
  return { value, outputs };
}

function addOutput(outputs, seenPaths, path, sha256, role) {
  if (seenPaths.has(path)) fail("E_RUNTIME_OUTPUT_DUPLICATE", path);
  seenPaths.add(path);
  outputs.push({ path, sha256, role });
}

async function readContained(repoRoot, relativePath) {
  const candidate = resolve(repoRoot, ...relativePath.split("/"));
  const resolved = await realpath(candidate).catch(() => fail("E_RUNTIME_OUTPUT_MISSING", relativePath));
  const relativeReal = relative(repoRoot, resolved);
  if (relativeReal === ".." || relativeReal.startsWith(`..${sep}`) || resolve(repoRoot, relativeReal) !== resolved) fail("E_RUNTIME_PATH_ESCAPE", relativePath);
  return { path: resolved, bytes: await readFile(resolved) };
}

export async function collectRuntimeBundle(repoRoot = REPO_ROOT, overrides = {}) {
  const root = await realpath(repoRoot);
  const authorityFile = overrides.authorityBytes ?? (await readContained(root, B0_AUTHORITY_PATH)).bytes;
  const runtimeFile = overrides.runtimeManifestBytes ?? (await readContained(root, RUNTIME_MANIFEST_PATH)).bytes;
  const b6aMetadataFile = await readContained(root, CENYAO_IDLE_METADATA_PATH);
  const authority = validateAuthority(Buffer.from(authorityFile));
  const runtime = validateRuntimeManifest(Buffer.from(runtimeFile), authority, Buffer.from(b6aMetadataFile.bytes), !overrides.runtimeManifestBytes);
  const manifests = [
    { path: B0_AUTHORITY_PATH, bytes: Buffer.from(authorityFile), sha256: B0_AUTHORITY_SHA256, role: "authority-manifest" },
    { path: RUNTIME_MANIFEST_PATH, bytes: Buffer.from(runtimeFile), sha256: RUNTIME_MANIFEST_SHA256, role: "runtime-manifest" },
  ];
  const outputs = [];
  for (const item of runtime.outputs) {
    const source = await readContained(root, item.path);
    const actualSha = digest(source.bytes);
    if (actualSha !== item.sha256) fail("E_RUNTIME_OUTPUT_SHA", `${item.path}:expected=${item.sha256}:actual=${actualSha}`);
    outputs.push({ ...item, bytes: source.bytes, sourcePath: source.path });
  }
  return { manifests, outputs, approvedAssets: runtime.value.assets.length, atlasPages: runtime.value.atlases.length };
}

export function runtimeAssetBundlePlugin(options = {}) {
  const repoRoot = options.repoRoot ?? REPO_ROOT;
  const collect = options.collectBundle ?? collectRuntimeBundle;
  let payload;
  return {
    name: "runtime-asset-bundle-v1",
    apply: "build",
    enforce: "pre",
    async buildStart() {
      payload = await collect(repoRoot);
      for (const file of [...payload.manifests, ...payload.outputs]) this.addWatchFile(file.sourcePath ?? resolve(repoRoot, ...file.path.split("/")));
    },
    generateBundle() {
      if (!payload) fail("E_RUNTIME_BUNDLE_NOT_VALIDATED");
      for (const file of [...payload.manifests, ...payload.outputs]) this.emitFile({ type: "asset", fileName: file.path, source: file.bytes });
    },
  };
}

async function walkFiles(root, current = root) {
  const entries = await readdir(current, { withFileTypes: true });
  const result = [];
  for (const entry of entries.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
    const path = resolve(current, entry.name);
    if (entry.isDirectory()) result.push(...await walkFiles(root, path));
    else if (entry.isFile()) result.push({ path, relativePath: relative(root, path).split(sep).join("/") });
  }
  return result;
}

export async function verifyDistBundle(repoRoot = REPO_ROOT, distRoot = resolve(repoRoot, "apps/web/dist")) {
  const payload = await collectRuntimeBundle(repoRoot);
  const scenePayload = await collectSceneDefinitionBundle(repoRoot);
  const dist = await realpath(distRoot);
  const expected = new Map([
    ...payload.manifests.map(file => [file.path, file.sha256]),
    ...payload.outputs.map(file => [file.path, file.sha256]),
    [scenePayload.manifest.path, scenePayload.manifest.sha256],
    ...scenePayload.files.map(file => [file.path, file.sha256]),
  ]);
  const bundledBytes = new Map();
  for (const [path, expectedSha] of expected) {
    const { bytes } = await readContained(dist, path);
    const actualSha = digest(bytes);
    if (actualSha !== expectedSha) fail("E_DIST_RUNTIME_SHA", `${path}:expected=${expectedSha}:actual=${actualSha}`);
    bundledBytes.set(path, bytes.length);
  }
  const files = await walkFiles(dist);
  for (const file of files) {
    if ((file.relativePath.startsWith("assets/derived/") || file.relativePath.startsWith("governance/assets/")) && !expected.has(file.relativePath)) {
      fail("E_DIST_UNEXPECTED_RUNTIME_RESOURCE", file.relativePath);
    }
    if (file.relativePath.startsWith("scene-definitions/") && !expected.has(file.relativePath)) fail("E_DIST_UNEXPECTED_SCENE_DEFINITION", file.relativePath);
    if (/^(?:assets\/source\/approved|chatgptimage|release-ui)(?:\/|$)/i.test(file.relativePath)) fail("E_DIST_FORBIDDEN_SOURCE_OR_LEGACY", file.relativePath);
    if (/\.(?:png|webp|jpe?g|gif|svg|glb|gltf|babylon|tps|wav|mp3|ogg|mp4|webm)$/i.test(file.relativePath) && !expected.has(file.relativePath)) fail("E_DIST_UNEXPECTED_MEDIA", file.relativePath);
    if (/\.(?:glb|gltf|babylon|tps)$/i.test(file.relativePath) || /(^|\/)(?:release-ui|babylon|glb|tps)(\/|$)/i.test(file.relativePath)) {
      fail("E_DIST_LEGACY_RESOURCE", file.relativePath);
    }
  }
  const tree = createHash("sha256");
  let distBytes = 0;
  for (const file of files.sort((a, b) => a.relativePath < b.relativePath ? -1 : a.relativePath > b.relativePath ? 1 : 0)) {
    const bytes = await readFile(file.path);
    const sha = digest(bytes);
    tree.update(`${file.relativePath}\0${sha}\n`);
    distBytes += bytes.length;
  }
  const runtimeBytes = [...bundledBytes.values()].reduce((sum, size) => sum + size, 0);
  return {
    result: "pass",
    manifestFiles: payload.manifests.length,
    runtimeOutputFiles: payload.outputs.length,
    runtimeOutputBytes: runtimeBytes,
    sceneDefinitionFiles: scenePayload.files.length,
    sceneDefinitionManifestSha256: scenePayload.manifest.sha256,
    distFileCount: files.length,
    distBytes,
    distTreeSha256: tree.digest("hex"),
    runtimeFilesSha256: createHash("sha256").update([...expected].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).map(([path, sha]) => `${path}\0${sha}\n`).join("")).digest("hex"),
    excludedApprovedSourceCopies: true,
    excludedConceptRejectAndLegacyAssets: true,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = process.argv[2] === "verify-dist" ? await verifyDistBundle() : await collectRuntimeBundle();
    if (process.argv[2] !== "verify-dist") {
      console.log(JSON.stringify({ result: "pass", manifests: result.manifests.length, runtimeOutputFiles: result.outputs.length,
        runtimeOutputBytes: result.outputs.reduce((sum, item) => sum + item.bytes.length, 0), approvedAssets: result.approvedAssets, atlasPages: result.atlasPages }));
    } else {
      console.log(JSON.stringify(result));
    }
  } catch (error) {
    console.error(`RUNTIME ASSET BUNDLE FAIL CLOSED: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  }
}
