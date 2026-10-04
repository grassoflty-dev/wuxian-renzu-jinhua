import type { VerifiedSceneDefinition } from "../renderer/ScenePresentation.js";

export const SCENE_DEFINITION_MANIFEST_PATH = "scene-definitions/SCENE_DEFINITION_MANIFEST.json";
const WORLD_IDS = new Set(["grey_hive", "mist_harbor", "clockworks", "return_station"]);
const ID_PATTERN = /^[A-Za-z0-9_.:-]{1,128}$/;
const SHA_PATTERN = /^[0-9a-f]{64}$/;
const MANIFEST_LIMIT = 1024 * 1024;
const SCENE_LIMIT = 16 * 1024 * 1024;

const verifiedSourceHashes = new WeakMap<object, string>();
/** Available only for the exact object successfully authenticated by this loader. */
export function verifiedSceneSourceSha256(scene: object): string | undefined { return verifiedSourceHashes.get(scene); }

interface SceneManifestEntry { worldId: string; sceneId: string; path: string; sha256: string }
interface SceneManifest { schemaVersion: 1; scenes: SceneManifestEntry[] }
export interface SceneDefinitionFetchOptions { signal?: AbortSignal }
export interface SceneDefinitionLoaderOptions {
  baseUrl?: string | URL;
  expectedManifestSha256: string;
  fetchImpl?: typeof fetch;
}

function fail(code: string, detail?: string): never { throw new Error(detail ? `${code}:${detail}` : code); }
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function sha256Text(bytes: Uint8Array): Promise<string> {
  if (!globalThis.crypto?.subtle) return Promise.reject(new Error("E_SCENE_DEFINITION_CRYPTO_UNAVAILABLE"));
  return globalThis.crypto.subtle.digest("SHA-256", bytes.slice().buffer as ArrayBuffer)
    .then(hash => [...new Uint8Array(hash)].map(value => value.toString(16).padStart(2, "0")).join(""));
}
function compare(left: string, right: string): number { return left < right ? -1 : left > right ? 1 : 0; }
function abortError(): DOMException { return new DOMException("Scene definition loading was cancelled", "AbortError"); }
function validateManifest(value: unknown): SceneManifest {
  if (!isRecord(value) || value.schemaVersion !== 1 || !Array.isArray(value.scenes) ||
      Object.keys(value).some(key => key !== "schemaVersion" && key !== "scenes")) fail("E_SCENE_DEFINITION_MANIFEST_SCHEMA");
  const scenes: SceneManifestEntry[] = [];
  let previousKey = "";
  for (const raw of value.scenes) {
    if (!isRecord(raw) || Object.keys(raw).length !== 4 ||
        Object.keys(raw).some(key => !["worldId", "sceneId", "path", "sha256"].includes(key)) ||
        typeof raw.worldId !== "string" || !WORLD_IDS.has(raw.worldId) ||
        typeof raw.sceneId !== "string" || !ID_PATTERN.test(raw.sceneId) ||
        typeof raw.path !== "string" || raw.path !== `scene-definitions/compiled/${raw.worldId}/${raw.sceneId}.json` ||
        typeof raw.sha256 !== "string" || !SHA_PATTERN.test(raw.sha256)) fail("E_SCENE_DEFINITION_MANIFEST_ENTRY");
    const key = `${raw.worldId}\0${raw.sceneId}`;
    if (previousKey && compare(previousKey, key) >= 0) fail("E_SCENE_DEFINITION_MANIFEST_ORDER", `${raw.worldId}:${raw.sceneId}`);
    previousKey = key;
    scenes.push({ worldId: raw.worldId, sceneId: raw.sceneId, path: raw.path, sha256: raw.sha256 });
  }
  return { schemaVersion: 1, scenes };
}

/** Loads only compiler-bundled scene files authenticated by the build-pinned manifest hash. */
export class SceneDefinitionLoader {
  private readonly baseUrl: URL;
  private readonly fetchImpl: typeof fetch;

  constructor(private readonly options: SceneDefinitionLoaderOptions) {
    const base = options.baseUrl ?? (typeof document !== "undefined" ? document.baseURI : undefined);
    if (!base || !SHA_PATTERN.test(options.expectedManifestSha256)) fail("E_SCENE_DEFINITION_LOADER_CONFIG");
    try {
      this.baseUrl = base instanceof URL ? new URL(base.href) : new URL(base);
      if (!["http:", "https:", "tauri:"].includes(this.baseUrl.protocol)) fail("E_SCENE_DEFINITION_BASE_URL");
    } catch { fail("E_SCENE_DEFINITION_BASE_URL"); }
    this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis);
  }

  async loadForSnapshot(identity: { worldId: string; sceneId: string; worldEpoch: number }, options: SceneDefinitionFetchOptions = {}): Promise<VerifiedSceneDefinition> {
    if (!WORLD_IDS.has(identity.worldId) || !ID_PATTERN.test(identity.sceneId) || !Number.isSafeInteger(identity.worldEpoch) || identity.worldEpoch < 0) {
      fail("E_SCENE_DEFINITION_SNAPSHOT_IDENTITY");
    }
    const manifest = await this.loadManifest(options.signal);
    const entry = manifest.scenes.find(item => item.worldId === identity.worldId && item.sceneId === identity.sceneId);
    if (!entry) fail("E_SCENE_DEFINITION_MISSING", `${identity.worldId}:${identity.sceneId}`);
    const bytes = await this.read(entry.path, SCENE_LIMIT, options.signal, "E_SCENE_DEFINITION");
    const actualSha = await sha256Text(bytes);
    if (actualSha !== entry.sha256) fail("E_SCENE_DEFINITION_SHA", `${identity.worldId}:${identity.sceneId}`);
    let scene: unknown;
    try { scene = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)); }
    catch { fail("E_SCENE_DEFINITION_JSON", `${identity.worldId}:${identity.sceneId}`); }
    if (!isRecord(scene) || scene.schemaVersion !== 1 || scene.worldId !== identity.worldId || scene.sceneId !== identity.sceneId) {
      fail("E_SCENE_DEFINITION_IDENTITY", `${identity.worldId}:${identity.sceneId}`);
    }
    // Keep the exact authenticated object graph immutable after proof creation.
    const pending: object[] = [scene];
    while (pending.length) {
      const item = pending.pop()!;
      for (const child of Object.values(item)) if (child && typeof child === "object") pending.push(child);
      Object.freeze(item);
    }
    verifiedSourceHashes.set(scene, actualSha);
    return scene as unknown as VerifiedSceneDefinition;
  }

  private async loadManifest(signal?: AbortSignal): Promise<SceneManifest> {
    const bytes = await this.read(SCENE_DEFINITION_MANIFEST_PATH, MANIFEST_LIMIT, signal, "E_SCENE_DEFINITION_MANIFEST");
    if (await sha256Text(bytes) !== this.options.expectedManifestSha256) fail("E_SCENE_DEFINITION_MANIFEST_SHA");
    let parsed: unknown;
    try { parsed = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)); }
    catch { fail("E_SCENE_DEFINITION_MANIFEST_JSON"); }
    return validateManifest(parsed);
  }

  private async read(path: string, maxBytes: number, signal: AbortSignal | undefined, prefix: string): Promise<Uint8Array> {
    if (signal?.aborted) throw abortError();
    let url: URL;
    try { url = new URL(path, this.baseUrl); }
    catch { fail(`${prefix}_PATH`, path); }
    let response: Response;
    try {
      response = await this.fetchImpl(url, { method: "GET", cache: "no-store", credentials: "same-origin", ...(signal ? { signal } : {}) });
    } catch (error) {
      if (signal?.aborted || (error instanceof DOMException && error.name === "AbortError")) throw abortError();
      throw new Error(`${prefix}_FETCH:${path}`, { cause: error });
    }
    if (signal?.aborted) throw abortError();
    if (!response.ok) fail(`${prefix}_HTTP`, `${response.status}:${path}`);
    const finalUrl = response.url ? new URL(response.url, url) : url;
    if (finalUrl.origin !== url.origin || finalUrl.pathname !== url.pathname) fail(`${prefix}_REDIRECT`, path);
    let bytes: Uint8Array;
    try { bytes = new Uint8Array(await response.arrayBuffer()); }
    catch (error) { throw new Error(`${prefix}_BODY:${path}`, { cause: error }); }
    if (signal?.aborted) throw abortError();
    if (bytes.byteLength > maxBytes) fail(`${prefix}_SIZE`, path);
    return bytes;
  }
}
