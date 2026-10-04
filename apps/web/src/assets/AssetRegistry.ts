export interface SpriteAsset {
  kind: "actor" | "prop" | "door";
  textureUrl: string;
  anchorX: number;
  anchorY: number;
  scale: number;
}

export type AssetCategory = "actor" | "prop" | "vfx" | "ui" | "world" | "boss";
export type AssetKind = "Actor" | "Prop" | "Door" | "VFX" | "UI" | "World" | "Boss";
export type AnimationDirection = "north" | "north_east" | "east" | "south_east" | "south" | "south_west" | "west" | "north_west";
export type AtlasFrame = readonly [x: number, y: number, width: number, height: number];

interface AnimationFrameRecord {
  direction: AnimationDirection;
  state: string;
  atlasPage: number;
  atlasFrame: AtlasFrame;
  extrudedFrame: AtlasFrame;
  anchor: readonly [number, number];
  sourceFrame: { png: AssetOutput; webp: AssetOutput };
}

export interface AnimationFrame extends AnimationFrameRecord {
  /** Resolved from the validated atlas page after the manifest shape is accepted. */
  atlasUrl: string;
  atlasSha256: string;
}

interface AnimationMetadataRecord {
  directions: AnimationDirection[];
  states: string[];
  sourceMetadata: AssetOutput;
  sourceMasterSha256: string;
  sourceAsset: { assetId: string; path: string; sha256: string };
  frames: AnimationFrameRecord[];
}

export interface AnimationMetadata extends Omit<AnimationMetadataRecord, "frames"> {
  frames: AnimationFrame[];
}

export type ShadowMetadata =
  | { kind: "none" }
  | { kind: "ellipse"; offset: readonly [number, number]; scale: readonly [number, number]; opacity: number };

export interface RuntimeAsset {
  assetId: string;
  kind: AssetKind;
  category: AssetCategory;
  streamingGroup: string;
  textureUrl: string;
  textureSha256: string;
  sourcePath: string;
  sourceSha256: string;
  anchorX: number;
  anchorY: number;
  scale: number;
  atlasUrl: string;
  atlasSha256: string;
  atlasPage: number;
  atlasFrame: AtlasFrame;
  animation?: AnimationMetadata;
  shadow?: ShadowMetadata;
}

interface AuthorityAsset {
  relative_path: string;
  asset_id: string;
  sha256: string;
  release_status: string;
}

interface AuthorityManifest {
  schemaId: string;
  schemaVersion: number;
  decisionSource: { path: string; sha256: string };
  sourceTree: { commit: string; prefix: string };
  counts: { rows: number; release_approved: number; concept_only: number; reject: number };
  assets: AuthorityAsset[];
}

interface AtlasRecord {
  category: AssetCategory;
  streamingGroup: string;
  pageIndex: number;
  pngPath: string;
  pngSha256: string;
  webpPath: string;
  webpSha256: string;
  size: [number, number];
  maxSide: number;
  decodedRgbaMiB: number;
}

interface AssetOutput { path: string; sha256: string }

interface RuntimeAssetRecord {
  admission: string;
  anchor: [number, number];
  assetId: string;
  atlasFrame: [number, number, number, number];
  atlasPage: number;
  category: AssetCategory;
  crop: {
    sourceSize: [number, number];
    cropRect: [number, number, number, number];
    outputSize: [number, number];
    alphaCleanup: string;
  };
  outputs: { png: AssetOutput; webp: AssetOutput };
  runtimeStatus: string;
  scale: number;
  sourcePath: string;
  sourceSha256: string;
  streamingGroup: string;
  animation?: AnimationMetadataRecord;
  shadow?: ShadowMetadata;
}

interface PendingAssetRecord {
  assetId: string;
  category: AssetCategory;
  sourcePath: string;
  sourceSha256: string;
  dimensions: [number, number];
  maxSide: number;
  status: string;
  requiredAction: string;
}

interface RuntimeManifest {
  schemaId: string;
  manifestState: string;
  authority: { path: string; sourceCommit: string };
  tool: Record<string, unknown>;
  atlases: AtlasRecord[];
  assets: RuntimeAssetRecord[];
  pendingAssets: PendingAssetRecord[];
  residency: { budgetMiB: number; p95MiB: number; profiles: unknown[]; method: string };
}

export type OutputBytesLoader = (relativePath: string) => Promise<Uint8Array>;

const AUTHORITY_MANIFEST_SHA256 = "c644b5953e9265d45796dfb8a6dad1c0fd8abfbffb3eb4ace84987d8238cee2d";
const SOURCE_COMMIT = "030eb9bf8194ad5534dc77315567ff339cd2c625";
const DECISION_SHA256 = "fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1";
const SOURCE_PREFIX = "chatgptimage/wuxian-renzu-jinhua_2d_asset_pack_10_batches_2026-09-25/";
const CENYAO_IDLE_METADATA_PATH = "assets/metadata/cenyao-runtime-master-v1/cenyao-idle-atlas-v1.json";
const CENYAO_IDLE_METADATA_SHA256 = "a1d5f40c2a7ae47f45dbf2b85432291be3be0badb855f1f386ec9905d533641c";
const CENYAO_MASTER_SHA256 = "80a810ec6fdbcda50a0daeebcf53cd2173c0a411f043861e869187061cb59f61";
const CENYAO_ATLAS_PNG_PATH = "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.png";
const CENYAO_ATLAS_WEBP_PATH = "assets/derived/cenyao-runtime-master-v1/atlas/cenyao-idle-atlas-v1.webp";
const CENYAO_ATLAS_PNG_SHA256 = "2146eeedc2945149fbee98a4f24739bff6cefeeef548e0bdace37fe8217028fe";
const CENYAO_ATLAS_WEBP_SHA256 = "b0cc05697fdb89a3f55fdbce2bd558e6800a088f1147f7f8edce8fad842c5af0";
const CENYAO_IDLE_FRAMES: Readonly<Record<AnimationDirection, { rect: AtlasFrame; pngSha256: string; webpSha256: string }>> = {
  south: { rect: [2, 2, 336, 560], pngSha256: "b6cff22dabb303f0a9c95ba24a41fb3c2f5159ff191adefacf0c6dad38ffe875", webpSha256: "627292d979d0a8a3334dec47630ad9ac531f023a87140f52edcc9ce523ffe433" },
  south_west: { rect: [342, 2, 336, 560], pngSha256: "3d3e34d0a5d388de522fd6a94fd2769efbf8137aea1ae7c0b69881c5416520be", webpSha256: "c5f2eddc15e71e0ca388ba55b17b15d53fe4df262dbc3cad7055cbf939591ea3" },
  west: { rect: [682, 2, 336, 560], pngSha256: "fc8e0b132c48f594a49a20bd39955d731f02077502e33131499608cc3bd7a70c", webpSha256: "a33a153ba557b47f8460732895a8b2d819a39f733607a9332d4eb2d4d553e1f1" },
  north_east: { rect: [1022, 2, 336, 560], pngSha256: "19b8e318bf5216d6075f64ac986ef7c3c3299241816b472843aac24bbf4a7676", webpSha256: "464b2b653711a44cabcfa400004c78ff32bf5fd39c810a14cea334e14c3c6f38" },
  north: { rect: [2, 566, 336, 560], pngSha256: "6608ed34685f28f3380849d7fe5e5319078e8feb1fbd66cbcd4cb3782a1a8221", webpSha256: "ca89a1ee487252dcf1eeb0ad9e9d25abec10ae61882374a17839ecff2b5de770" },
  north_west: { rect: [342, 566, 336, 560], pngSha256: "8dc527b975b1da5ed7b36a54cf4ad260cdcd9d7d3ccba24660308cf971c3164e", webpSha256: "fec4be62fc81d025a18a1248803355c590bb9fe31234f3b757eb2328c11a0cf5" },
  east: { rect: [682, 566, 336, 560], pngSha256: "fd38db1daec829c0a7c2adffcdfd5c605f54b3611bb515599cd8deeea1f47402", webpSha256: "daad53355ccfd7262f2c102381e665e07918c2e4352523d43a020bbed272605a" },
  south_east: { rect: [1022, 566, 336, 560], pngSha256: "bfe0107fe1efdcd73b90d57b01283a22658960b5847bb5e88b401dae9857be56", webpSha256: "7126d9cf50ef316f990e1d8b0eab4f7538425fa92cee4a3fcee03cb0c0e01448" },
};
const SHA256_RE = /^[0-9a-f]{64}$/;
const CATEGORIES = new Set<AssetCategory>(["actor", "prop", "vfx", "ui", "world", "boss"]);
const DIRECTIONS = new Set<AnimationDirection>(["north", "north_east", "east", "south_east", "south", "south_west", "west", "north_west"]);
const LIMITS: Record<AssetCategory, number> = { actor: 4096, prop: 4096, vfx: 4096, ui: 2048, world: 4096, boss: 4096 };

function fail(code: string, detail = ""): never {
  throw new Error(detail ? `${code}:${detail}` : code);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasExactKeys(value: Record<string, unknown>, required: readonly string[], optional: readonly string[] = []): boolean {
  const keys = Object.keys(value);
  return required.every(key => Object.hasOwn(value, key)) && keys.every(key => required.includes(key) || optional.includes(key));
}

function isSha256(value: unknown): value is string {
  return typeof value === "string" && SHA256_RE.test(value);
}

function isSafePath(value: unknown, prefix: string): value is string {
  return typeof value === "string" && value.startsWith(prefix) && !value.includes("\\") &&
    !value.split("/").some(part => part === "" || part === "." || part === "..") && !value.includes(":");
}

function isFinitePair(value: unknown, min: number, max: number): value is [number, number] {
  return Array.isArray(value) && value.length === 2 && value.every(item => typeof item === "number" && Number.isFinite(item) && item >= min && item <= max);
}

function isRect(value: unknown): value is [number, number, number, number] {
  return Array.isArray(value) && value.length === 4 && value.every(item => Number.isInteger(item) && item >= 0) && value[2] > 0 && value[3] > 0;
}

function validId(value: unknown): value is string {
  return typeof value === "string" && /^[a-zA-Z0-9][a-zA-Z0-9._-]{0,159}$/.test(value);
}

function decodeJson<T>(bytes: Uint8Array, code: string): T {
  try {
    return JSON.parse(new TextDecoder().decode(bytes)) as T;
  } catch {
    return fail(code);
  }
}

function deepFreeze<T>(value: T): T {
  if (isRecord(value) || Array.isArray(value)) {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}

async function sha256(bytes: Uint8Array): Promise<string> {
  if (!globalThis.crypto?.subtle) fail("E_ASSET_CRYPTO_UNAVAILABLE");
  const copy = new Uint8Array(bytes.byteLength);
  copy.set(bytes);
  const digest = await globalThis.crypto.subtle.digest("SHA-256", copy.buffer as ArrayBuffer);
  return [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("");
}

function validateAuthority(value: unknown): AuthorityManifest {
  if (!isRecord(value) || value.schemaId !== "ai-asset-release-manifest/1" || value.schemaVersion !== 1 ||
      !isRecord(value.decisionSource) || value.decisionSource.path !== "deep-research-report.md" || value.decisionSource.sha256 !== DECISION_SHA256 ||
      !isRecord(value.sourceTree) || value.sourceTree.commit !== SOURCE_COMMIT || value.sourceTree.prefix !== SOURCE_PREFIX ||
      !isRecord(value.counts) || value.counts.rows !== 94 || value.counts.release_approved !== 43 || value.counts.concept_only !== 30 || value.counts.reject !== 21 ||
      !Array.isArray(value.assets) || value.assets.length !== 94) fail("E_ASSET_AUTHORITY_SCHEMA");
  const rows = value.assets as unknown[];
  const ids = new Set<string>();
  const paths = new Set<string>();
  let approvedCount = 0;
  let conceptCount = 0;
  let rejectCount = 0;
  for (const item of rows) {
    if (!isRecord(item) || !validId(item.asset_id) || !isSha256(item.sha256) || typeof item.relative_path !== "string" ||
        !/^batch(?:0[1-9]|10)\/[^/]+$/.test(item.relative_path) || !["release_approved", "concept_only", "reject"].includes(String(item.release_status)) ||
        ids.has(item.asset_id) || paths.has(item.relative_path)) fail("E_ASSET_AUTHORITY_ROW");
    ids.add(item.asset_id);
    paths.add(item.relative_path);
    if (item.release_status === "release_approved") approvedCount++;
    else if (item.release_status === "concept_only") conceptCount++;
    else rejectCount++;
  }
  if (approvedCount !== 43 || conceptCount !== 30 || rejectCount !== 21) fail("E_ASSET_AUTHORITY_COUNTS");
  return value as unknown as AuthorityManifest;
}

function validateAnimation(value: unknown, asset: RuntimeAssetRecord): AnimationMetadata {
  if (!isRecord(value) || !hasExactKeys(value, ["directions", "states", "frames", "sourceMetadata", "sourceMasterSha256", "sourceAsset"]) ||
      !Array.isArray(value.directions) || value.directions.length === 0 || !Array.isArray(value.states) || value.states.length === 0 || !Array.isArray(value.frames)) {
    return fail("E_ASSET_ANIMATION_SCHEMA");
  }
  const directions = value.directions as unknown[];
  const states = value.states as unknown[];
  if (directions.some(item => !DIRECTIONS.has(item as AnimationDirection)) || new Set(directions).size !== directions.length ||
      states.some(item => typeof item !== "string" || !/^[a-z0-9][a-z0-9_-]{0,39}$/.test(item)) || new Set(states).size !== states.length) {
    return fail("E_ASSET_ANIMATION_ENUM");
  }
  const frames = value.frames as unknown[];
  const keys = new Set<string>();
  for (const item of frames) {
    if (!isRecord(item) || !hasExactKeys(item, ["direction", "state", "atlasPage", "atlasFrame", "extrudedFrame", "anchor", "sourceFrame"]) ||
        !DIRECTIONS.has(item.direction as AnimationDirection) || typeof item.state !== "string" ||
        !Number.isInteger(item.atlasPage) || !isRect(item.atlasFrame) || !isRect(item.extrudedFrame) ||
        !isFinitePair(item.anchor, 0, 1) || !isRecord(item.sourceFrame) || !hasExactKeys(item.sourceFrame, ["png", "webp"])) fail("E_ASSET_ANIMATION_FRAME");
    const key = `${String(item.direction)}:${item.state}`;
    if (keys.has(key)) fail("E_ASSET_ANIMATION_DUPLICATE_FRAME", key);
    keys.add(key);
  }
  if (keys.size !== directions.length * states.length || directions.some(direction => states.some(state => !keys.has(`${direction}:${state}`)))) {
    fail("E_ASSET_ANIMATION_INCOMPLETE");
  }
  const expectedDirections = ["south", "south_west", "west", "north_east", "north", "north_west", "east", "south_east"];
  if (asset.assetId !== "runtime2d.actor.cenyao.base.v1" || directions.join(",") !== expectedDirections.join(",") ||
      states.length !== 1 || states[0] !== "idle" || frames.length !== 8 ||
      !isRecord(value.sourceMetadata) || !hasExactKeys(value.sourceMetadata, ["path", "sha256"]) ||
      value.sourceMetadata.path !== CENYAO_IDLE_METADATA_PATH || value.sourceMetadata.sha256 !== CENYAO_IDLE_METADATA_SHA256 ||
      value.sourceMasterSha256 !== CENYAO_MASTER_SHA256 || !isRecord(value.sourceAsset) ||
      !hasExactKeys(value.sourceAsset, ["assetId", "path", "sha256"]) || value.sourceAsset.assetId !== asset.assetId ||
      value.sourceAsset.path !== asset.sourcePath || value.sourceAsset.sha256 !== asset.sourceSha256) fail("E_ASSET_ANIMATION_SOURCE");
  for (const item of frames) {
    if (!isRecord(item)) fail("E_ASSET_ANIMATION_FRAME");
    const direction = item.direction as AnimationDirection;
    const expected = CENYAO_IDLE_FRAMES[direction];
    if (item.state !== "idle" || item.atlasPage !== 13 || JSON.stringify(item.atlasFrame) !== JSON.stringify(expected.rect) ||
        JSON.stringify(item.extrudedFrame) !== JSON.stringify([expected.rect[0] - 2, expected.rect[1] - 2, 340, 564]) ||
        JSON.stringify(item.anchor) !== JSON.stringify([0.5, 0.98392857]) || !isRecord(item.sourceFrame)) fail("E_ASSET_ANIMATION_FRAME_SOURCE", direction);
    for (const format of ["png", "webp"] as const) {
      const output = item.sourceFrame[format];
      const expectedSha = format === "png" ? expected.pngSha256 : expected.webpSha256;
      if (!isRecord(output) || !hasExactKeys(output, ["path", "sha256"]) ||
          output.path !== `assets/derived/cenyao-runtime-master-v1/${format}/cenyao_idle_${direction}.${format}` ||
          !isSafePath(output.path, `assets/derived/cenyao-runtime-master-v1/${format}/`) ||
          !isSha256(output.sha256) || output.sha256 !== expectedSha) fail("E_ASSET_ANIMATION_FRAME_SOURCE", `${direction}:${format}`);
    }
  }
  return value as unknown as AnimationMetadata;
}

function validateShadow(value: unknown): ShadowMetadata {
  if (!isRecord(value)) return fail("E_ASSET_SHADOW_SCHEMA");
  if (value.kind === "none" && hasExactKeys(value, ["kind"])) return value as ShadowMetadata;
  if (value.kind !== "ellipse" || !hasExactKeys(value, ["kind", "offset", "scale", "opacity"]) ||
      !isFinitePair(value.offset, -10000, 10000) || !isFinitePair(value.scale, Number.MIN_VALUE, 10000) ||
      typeof value.opacity !== "number" || !Number.isFinite(value.opacity) || value.opacity < 0 || value.opacity > 1) {
    return fail("E_ASSET_SHADOW_SCHEMA");
  }
  return value as unknown as ShadowMetadata;
}

function categoryKind(record: RuntimeAssetRecord): AssetKind {
  if (record.category === "actor") return "Actor";
  if (record.category === "prop") return /\.gate_[a-z0-9]+\./.test(record.assetId) ? "Door" : "Prop";
  if (record.category === "vfx") return "VFX";
  if (record.category === "ui") return "UI";
  if (record.category === "world") return "World";
  return "Boss";
}

function validateManifestShape(value: unknown, authority: AuthorityManifest): RuntimeManifest {
  const topKeys = ["assets", "atlases", "authority", "manifestState", "pendingAssets", "residency", "schemaId", "tool"];
  if (!isRecord(value) || !hasExactKeys(value, topKeys) || value.schemaId !== "runtime-asset-manifest/1" || value.manifestState !== "ADOPT" ||
      !isRecord(value.authority) || !hasExactKeys(value.authority, ["path", "sourceCommit"]) || value.authority.path !== "governance/assets/AI_ASSET_RELEASE_MANIFEST.json" || value.authority.sourceCommit !== SOURCE_COMMIT ||
      !Array.isArray(value.assets) || !Array.isArray(value.atlases) || !Array.isArray(value.pendingAssets) || !isRecord(value.residency) ||
      value.residency.budgetMiB !== 220 || typeof value.residency.p95MiB !== "number" || !Number.isFinite(value.residency.p95MiB) || value.residency.p95MiB < 0 || value.residency.p95MiB > 220 ||
      !Array.isArray(value.residency.profiles) || value.residency.profiles.length !== 4 || typeof value.residency.method !== "string" || !isRecord(value.tool)) {
    fail("E_ASSET_MANIFEST_SCHEMA");
  }
  const manifest = value as unknown as RuntimeManifest;
  if (!hasExactKeys(manifest.tool, ["version", "python", "pillow", "webpEncoder"]) ||
      manifest.tool.version !== "asset-pipeline-core/1.0.0" || typeof manifest.tool.python !== "string" ||
      manifest.tool.pillow !== "12.3.0" || manifest.tool.webpEncoder !== "1.6.0") fail("E_ASSET_TOOLCHAIN_SCHEMA");
  if (manifest.atlases.length !== 14) fail("E_ASSET_ATLAS_MISSING");
  const approved = new Map(authority.assets.filter(row => row.release_status === "release_approved").map(row => [row.asset_id, row]));
  const pageIds = new Set<number>();
  for (let index = 0; index < manifest.atlases.length; index++) {
    const page = manifest.atlases[index];
    const allowedB6aPage = index === 13 && page?.category === "actor" && page?.streamingGroup === "shared" &&
      page?.pngPath === CENYAO_ATLAS_PNG_PATH && page?.pngSha256 === CENYAO_ATLAS_PNG_SHA256 &&
      page?.webpPath === CENYAO_ATLAS_WEBP_PATH && page?.webpSha256 === CENYAO_ATLAS_WEBP_SHA256;
    if (!isRecord(page) || !hasExactKeys(page, ["category", "streamingGroup", "pageIndex", "pngPath", "pngSha256", "webpPath", "webpSha256", "size", "maxSide", "decodedRgbaMiB"]) ||
        !CATEGORIES.has(page.category as AssetCategory) || typeof page.streamingGroup !== "string" || !page.streamingGroup || page.pageIndex !== index || pageIds.has(page.pageIndex) ||
        !(allowedB6aPage || (index !== 13 && isSafePath(page.pngPath, "assets/derived/asset-pipeline-v1/") && isSafePath(page.webpPath, "assets/derived/asset-pipeline-v1/"))) ||
        !isSha256(page.pngSha256) || !isSha256(page.webpSha256) || !isFinitePair(page.size, 1, 10000) || !Number.isInteger(page.size[0]) || !Number.isInteger(page.size[1]) ||
        page.maxSide !== LIMITS[page.category as AssetCategory] || typeof page.decodedRgbaMiB !== "number" || !Number.isFinite(page.decodedRgbaMiB) || page.decodedRgbaMiB <= 0) {
      fail("E_ASSET_ATLAS_SCHEMA", String(index));
    }
    const [width, height] = page.size;
    const max = LIMITS[page.category as AssetCategory];
    if (width > max || height > max) fail("E_ASSET_ATLAS_BUDGET", String(index));
    if (index === 13 && (page.category !== "actor" || page.streamingGroup !== "shared" || page.size[0] !== 1360 || page.size[1] !== 1128)) fail("E_ASSET_CENYAO_ATLAS");
    if (Math.abs(page.decodedRgbaMiB - width * height * 4 / (1024 * 1024)) > 0.002) fail("E_ASSET_ATLAS_SIZE_METADATA", String(index));
    pageIds.add(page.pageIndex as number);
  }

  const roundMiB = (value: number): number => Math.round(value * 1000) / 1000;
  const groupMiB = new Map<string, number>();
  for (const page of manifest.atlases) {
    const key = `${page.category}:${page.streamingGroup}`;
    groupMiB.set(key, (groupMiB.get(key) ?? 0) + page.decodedRgbaMiB);
  }
  const worlds = ["grey_hive", "mistharbor", "clockworks", "returnstation"];
  const computedProfiles = worlds.map(activeWorld => {
    const groups = new Set([["actor", "shared"], ["vfx", "shared"], ["ui", "shared"], ["world", activeWorld], ["actor", activeWorld]].map(parts => parts.join(":")));
    if (activeWorld === "grey_hive") {
      groups.add("prop:grey_hive");
      groups.add("boss:sentinel");
    }
    const activeGroups = [...groups].filter(key => groupMiB.has(key)).sort();
    return {
      activeWorld,
      residentTextureMiB: roundMiB(activeGroups.reduce((sum, key) => sum + (groupMiB.get(key) ?? 0), 0)),
      residentGroups: activeGroups.map(key => key.split(":")),
    };
  });
  const p95 = [...computedProfiles.map(profile => profile.residentTextureMiB)].sort((a, b) => a - b)[Math.ceil(0.95 * computedProfiles.length) - 1];
  if (manifest.residency.method !== "RGBA decoded texture footprint; lazy-load shared plus one active world; nearest-rank P95 over four world profiles" ||
      manifest.residency.profiles.some((profile, index) => {
        const expected = computedProfiles[index];
        return !expected || !isRecord(profile) || !hasExactKeys(profile, ["activeWorld", "residentTextureMiB", "residentGroups"]) ||
          profile.activeWorld !== expected.activeWorld || profile.residentTextureMiB !== expected.residentTextureMiB ||
          JSON.stringify(profile.residentGroups) !== JSON.stringify(expected.residentGroups);
      }) || manifest.residency.p95MiB !== p95) {
    fail("E_ASSET_RESIDENCY_SCHEMA");
  }

  const seen = new Set<string>();
  const placements = new Map<number, Array<[string, number, number, number, number]>>();
  for (const raw of manifest.assets as unknown[]) {
    if (!isRecord(raw)) fail("E_ASSET_RECORD_SCHEMA");
    const required = ["admission", "anchor", "assetId", "atlasFrame", "atlasPage", "category", "crop", "outputs", "runtimeStatus", "scale", "sourcePath", "sourceSha256", "streamingGroup"];
    if (!hasExactKeys(raw, required, ["animation", "shadow"])) fail("E_ASSET_RECORD_SCHEMA");
    const item = raw as unknown as RuntimeAssetRecord;
    const source = approved.get(item.assetId);
    if (!source || seen.has(item.assetId)) fail("E_ASSET_NOT_APPROVED", item.assetId);
    if (item.admission !== "release_approved" || item.runtimeStatus !== "ADOPT") fail("E_ASSET_NOT_ADOPTED", item.assetId);
    if (item.sourcePath !== source.relative_path || item.sourceSha256 !== source.sha256 || !isSha256(item.sourceSha256)) fail("E_ASSET_SOURCE_SHA", item.assetId);
    if (!CATEGORIES.has(item.category) || !validId(item.assetId) || typeof item.streamingGroup !== "string" || !item.streamingGroup) fail("E_ASSET_RECORD_SCHEMA", item.assetId);
    if (!isFinitePair(item.anchor, 0, 1) || typeof item.scale !== "number" || !Number.isFinite(item.scale) || item.scale <= 0) fail("E_ASSET_TRANSFORM", item.assetId);
    if (!isRect(item.atlasFrame) || !Number.isInteger(item.atlasPage)) fail("E_ASSET_FRAME_SCHEMA", item.assetId);
    const page = manifest.atlases[item.atlasPage];
    if (!page || page.category !== item.category || page.streamingGroup !== item.streamingGroup) fail("E_ASSET_ATLAS_REFERENCE", item.assetId);
    const [x, y, width, height] = item.atlasFrame;
    if (x + width > page.size[0] || y + height > page.size[1]) fail("E_ASSET_FRAME_BOUNDS", item.assetId);
    const pagePlacements = placements.get(item.atlasPage) ?? [];
    for (const [otherId, ox, oy, ow, oh] of pagePlacements) {
      if (x < ox + ow && ox < x + width && y < oy + oh && oy < y + height) fail("E_ASSET_FRAME_OVERLAP", `${item.assetId},${otherId}`);
    }
    pagePlacements.push([item.assetId, x, y, width, height]);
    placements.set(item.atlasPage, pagePlacements);
    if (!isRecord(item.crop) || !hasExactKeys(item.crop, ["sourceSize", "cropRect", "outputSize", "alphaCleanup"]) ||
        !Array.isArray(item.crop.sourceSize) || item.crop.sourceSize.length !== 2 || item.crop.sourceSize.some(n => !Number.isInteger(n) || n <= 0) ||
        !isRect(item.crop.cropRect) || !Array.isArray(item.crop.outputSize) || item.crop.outputSize.length !== 2 ||
        item.crop.outputSize[0] !== width || item.crop.outputSize[1] !== height || item.crop.alphaCleanup !== "crop-alpha-bounds; zero-rgb-where-alpha-zero") fail("E_ASSET_CROP_SCHEMA", item.assetId);
    if (!isRecord(item.outputs) || !hasExactKeys(item.outputs, ["png", "webp"])) fail("E_ASSET_OUTPUT_SCHEMA", item.assetId);
    for (const format of ["png", "webp"] as const) {
      const output = item.outputs[format];
      if (!isRecord(output) || !hasExactKeys(output, ["path", "sha256"]) || !isSafePath(output.path, "assets/derived/asset-pipeline-v1/") || !isSha256(output.sha256)) fail("E_ASSET_OUTPUT_SCHEMA", `${item.assetId}:${format}`);
    }
    if (Object.hasOwn(raw, "animation")) {
      const animation = validateAnimation(raw.animation, item);
      for (const frame of animation.frames) {
        const animationPage = manifest.atlases[frame.atlasPage];
        if (!animationPage || animationPage.category !== item.category || animationPage.streamingGroup !== item.streamingGroup || !isRect(frame.atlasFrame) ||
            frame.atlasFrame[0] + frame.atlasFrame[2] > animationPage.size[0] || frame.atlasFrame[1] + frame.atlasFrame[3] > animationPage.size[1]) {
          fail("E_ASSET_ANIMATION_FRAME_BOUNDS", item.assetId);
        }
        const [fx, fy, fw, fh] = frame.atlasFrame;
        for (const [otherId, ox, oy, ow, oh] of placements.get(frame.atlasPage) ?? []) {
          if (fx < ox + ow && ox < fx + fw && fy < oy + oh && oy < fy + fh) fail("E_ASSET_FRAME_OVERLAP", `${item.assetId}:${otherId}`);
        }
        const animPlacements = placements.get(frame.atlasPage) ?? [];
        animPlacements.push([`${item.assetId}:${frame.direction}:${frame.state}`, fx, fy, fw, fh]);
        placements.set(frame.atlasPage, animPlacements);
      }
    }
    if ((item.assetId === "runtime2d.actor.cenyao.base.v1") !== Object.hasOwn(raw, "animation")) fail("E_ASSET_CENYAO_ANIMATION_REQUIRED");
    if (Object.hasOwn(raw, "shadow")) validateShadow(raw.shadow);
    seen.add(item.assetId);
  }

  for (const raw of manifest.pendingAssets as unknown[]) {
    if (!isRecord(raw) || !hasExactKeys(raw, ["assetId", "category", "sourcePath", "sourceSha256", "dimensions", "maxSide", "status", "requiredAction"])) fail("E_ASSET_PENDING_SCHEMA");
    const source = approved.get(String(raw.assetId));
    if (!source || seen.has(String(raw.assetId)) || raw.sourcePath !== source.relative_path || raw.sourceSha256 !== source.sha256 || raw.status !== "PENDING_AUTHORED_EXTRACTION") fail("E_ASSET_PENDING_AUTHORITY");
    if (!CATEGORIES.has(raw.category as AssetCategory) || !Array.isArray(raw.dimensions) || raw.dimensions.length !== 2 || raw.dimensions.some(n => !Number.isInteger(n) || n <= 0) ||
        raw.maxSide !== LIMITS[raw.category as AssetCategory] || typeof raw.requiredAction !== "string" || raw.requiredAction.length === 0 ||
        (raw.category === "world" ? Math.max(raw.dimensions[0], raw.dimensions[1]) <= Number(raw.maxSide) : raw.dimensions.some(n => n <= Number(raw.maxSide)))) fail("E_ASSET_PENDING_BUDGET");
    seen.add(String(raw.assetId));
  }
  if (seen.size !== approved.size || [...approved.keys()].some(id => !seen.has(id))) fail("E_ASSET_APPROVED_SET_MISMATCH");
  if (manifest.atlases.some(page => !placements.has(page.pageIndex))) fail("E_ASSET_ATLAS_UNUSED_PAGE");

  const bossPages = new Set(manifest.assets.filter(item => item.category === "boss").map(item => item.atlasPage));
  if (bossPages.size !== 1 || [...bossPages].some(index => manifest.atlases[index]?.category !== "boss")) fail("E_ASSET_BOSS_PAGE");
  return manifest;
}

export class AssetRegistry {
  private readonly entries = new Map<string, SpriteAsset>();
  private readonly typedEntries = new Map<string, RuntimeAsset>();

  /** Legacy registration remains for the current WorldRenderer contract. */
  register(entityType: string, asset: SpriteAsset): void {
    if (!entityType.trim() || this.entries.has(entityType) || this.typedEntries.has(entityType) || !asset.textureUrl.trim()) {
      throw new Error(`E_ASSET_REGISTRY_INVALID:${entityType}`);
    }
    if (![asset.anchorX, asset.anchorY, asset.scale].every(Number.isFinite) || asset.anchorX < 0 || asset.anchorX > 1 || asset.anchorY < 0 || asset.anchorY > 1 || asset.scale <= 0) {
      throw new Error(`E_ASSET_REGISTRY_TRANSFORM:${entityType}`);
    }
    this.entries.set(entityType, Object.freeze({ ...asset }));
  }

  /**
   * Verifies the pinned B0 authority and every runtime output before atomically
   * registering the manifest. The caller supplies bytes so this remains usable
   * in browsers and tests without importing Node APIs into the web bundle.
   */
  async registerManifest(manifestBytes: Uint8Array, authorityBytes: Uint8Array, loadBytes: OutputBytesLoader): Promise<number> {
    if (await sha256(authorityBytes) !== AUTHORITY_MANIFEST_SHA256) fail("E_ASSET_AUTHORITY_SHA");
    const authority = validateAuthority(decodeJson<unknown>(authorityBytes, "E_ASSET_AUTHORITY_JSON"));
    const manifest = validateManifestShape(decodeJson<unknown>(manifestBytes, "E_ASSET_MANIFEST_JSON"), authority);
    const staged = new Map<string, RuntimeAsset>();
    const stagedAliases = new Map<string, SpriteAsset>();
    const verifyOutput = async (path: string, expectedSha: string): Promise<void> => {
      let bytes: Uint8Array;
      try { bytes = await loadBytes(path); } catch { return fail("E_ASSET_OUTPUT_MISSING", path); }
      if (!(bytes instanceof Uint8Array) || await sha256(bytes) !== expectedSha) fail("E_ASSET_OUTPUT_SHA", path);
    };

    // Bound retained output bytes to four in-flight files. Every manifest output
    // is still fetched and hashed before any entry is published. A failed worker
    // stops new work; drain the others before returning so retries cannot overlap
    // abandoned reads. Report the earliest manifest-order failure deterministically.
    const outputs = [
      ...manifest.atlases.flatMap(page => [
        { path: page.pngPath, sha256: page.pngSha256 },
        { path: page.webpPath, sha256: page.webpSha256 },
      ]),
      ...manifest.assets.flatMap(record => [record.outputs.png, record.outputs.webp]),
    ];
    let nextOutput = 0;
    const failures = new Map<number, unknown>();
    const verifyWorker = async (): Promise<void> => {
      while (failures.size === 0 && nextOutput < outputs.length) {
        const index = nextOutput++;
        const output = outputs[index]!;
        try { await verifyOutput(output.path, output.sha256); }
        catch (error) {
          failures.set(index, error);
        }
      }
    };
    await Promise.all(Array.from({ length: Math.min(4, outputs.length) }, verifyWorker));
    if (failures.size) throw failures.get(Math.min(...failures.keys()));

    for (const record of manifest.assets) {
      const page = manifest.atlases[record.atlasPage];
      if (!page) fail("E_ASSET_ATLAS_REFERENCE", record.assetId);
      const runtimeAnimation: AnimationMetadata | undefined = record.animation ? {
        ...structuredClone(record.animation),
        frames: record.animation.frames.map(frame => {
          const animationPage = manifest.atlases[frame.atlasPage];
          if (!animationPage) fail("E_ASSET_ANIMATION_ATLAS_REFERENCE", record.assetId);
          return { ...structuredClone(frame), atlasUrl: animationPage.webpPath, atlasSha256: animationPage.webpSha256 };
        }),
      } : undefined;
      const runtimeAsset: RuntimeAsset = {
        assetId: record.assetId,
        kind: categoryKind(record),
        category: record.category,
        streamingGroup: record.streamingGroup,
        textureUrl: record.outputs.webp.path,
        textureSha256: record.outputs.webp.sha256,
        sourcePath: record.sourcePath,
        sourceSha256: record.sourceSha256,
        anchorX: record.anchor[0],
        anchorY: record.anchor[1],
        scale: record.scale,
        atlasUrl: page.webpPath,
        atlasSha256: page.webpSha256,
        atlasPage: record.atlasPage,
        atlasFrame: Object.freeze([...record.atlasFrame]) as unknown as AtlasFrame,
        ...(runtimeAnimation ? { animation: deepFreeze(runtimeAnimation) } : {}),
        ...(record.shadow ? { shadow: deepFreeze(structuredClone(record.shadow)) } : {}),
      };
      staged.set(record.assetId, Object.freeze(runtimeAsset));
      const alias = this.legacyAlias(record.assetId);
      if (alias) {
        const sprite: SpriteAsset = Object.freeze({
          kind: runtimeAsset.kind === "Actor" ? "actor" : runtimeAsset.kind === "Door" ? "door" : "prop",
          textureUrl: runtimeAsset.textureUrl,
          anchorX: runtimeAsset.anchorX,
          anchorY: runtimeAsset.anchorY,
          scale: runtimeAsset.scale,
        });
        if (stagedAliases.has(alias) || this.entries.has(alias) || this.typedEntries.has(alias)) fail("E_ASSET_ALIAS_COLLISION", alias);
        stagedAliases.set(alias, sprite);
      }
    }
    for (const key of staged.keys()) if (this.typedEntries.has(key) || this.entries.has(key)) fail("E_ASSET_REGISTRY_DUPLICATE", key);
    for (const [key, value] of staged) this.typedEntries.set(key, value);
    for (const [key, value] of stagedAliases) this.entries.set(key, value);
    return staged.size;
  }

  resolve(entityType: string): SpriteAsset {
    const asset = this.entries.get(entityType);
    if (!asset) throw new Error(`E_ASSET_NOT_APPROVED:${entityType}`);
    return asset;
  }

  resolveAsset(assetId: string): RuntimeAsset {
    const asset = this.typedEntries.get(assetId);
    if (!asset) throw new Error(`E_ASSET_NOT_APPROVED:${assetId}`);
    return asset;
  }

  has(entityType: string): boolean { return this.entries.has(entityType) || this.typedEntries.has(entityType); }
  hasAsset(assetId: string): boolean { return this.typedEntries.has(assetId); }

  private legacyAlias(assetId: string): string | undefined {
    if (assetId === "runtime2d.actor.cenyao.base.v1") return "player.cenyao";
    if (assetId === "runtime2d.prop.power_console.off_on.v1") return "prop.grey_hive.power_console";
    if (assetId === "runtime2d.prop.gate_a.closed_open.v1") return "door.gh_gate_a";
    if (assetId === "runtime2d.prop.gate_b.v1") return "door.gh_gate_b";
    return undefined;
  }
}
