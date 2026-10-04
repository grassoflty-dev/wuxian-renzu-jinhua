import { AssetRegistry, type OutputBytesLoader } from "./AssetRegistry.js";

export const RUNTIME_AUTHORITY_MANIFEST_PATH = "governance/assets/AI_ASSET_RELEASE_MANIFEST.json";
export const RUNTIME_ASSET_MANIFEST_PATH = "governance/assets/RUNTIME_ASSET_MANIFEST.json";
export const RUNTIME_ASSET_MANIFEST_SHA256 = "081bebad5eede90e6ed50f2f7cacb930a67f2295961d405f58141d23874ec414";
export const RUNTIME_ASSET_OUTPUT_COUNT = 114;

export interface RuntimeAssetLoadProgress {
  phase: "fetching-manifests" | "verifying-outputs" | "complete" | "error";
  completedFiles: number;
  totalFiles: number;
  completedOutputs: number;
  totalOutputs: number;
  bytesFetched: number;
  elapsedMs: number;
  currentPath?: string;
}

export interface RuntimeAssetLoadOptions {
  /** Base document URL (normally document.baseURI) or an explicit dist URL. */
  baseUrl?: string | URL;
  fetchImpl?: typeof fetch;
}

export interface RuntimeAssetLoadRequest {
  signal?: AbortSignal;
  onProgress?: (progress: RuntimeAssetLoadProgress) => void;
}

function abortError(): DOMException {
  return new DOMException("Runtime asset loading was cancelled", "AbortError");
}

function throwIfAborted(signal?: AbortSignal): void {
  if (signal?.aborted) throw abortError();
}

async function sha256(bytes: Uint8Array): Promise<string> {
  if (!globalThis.crypto?.subtle) throw new Error("E_RUNTIME_ASSET_CRYPTO_UNAVAILABLE");
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes.slice().buffer as ArrayBuffer);
  return [...new Uint8Array(digest)].map(value => value.toString(16).padStart(2, "0")).join("");
}

function resolveBaseUrl(value?: string | URL): URL {
  const source = value ?? (typeof document !== "undefined" ? document.baseURI : undefined);
  if (!source) throw new Error("E_RUNTIME_ASSET_BASE_URL");
  try {
    const url = source instanceof URL ? new URL(source.href) : new URL(source);
    if (url.protocol !== "http:" && url.protocol !== "https:" && url.protocol !== "tauri:") {
      throw new Error("E_RUNTIME_ASSET_BASE_URL");
    }
    return url;
  } catch {
    throw new Error("E_RUNTIME_ASSET_BASE_URL");
  }
}

/**
 * Loads and validates the release bundle once per loader instance. Only the
 * registry metadata survives a successful load; output byte arrays are handed
 * to AssetRegistry with at most four reads in flight and are never retained by this loader.
 */
export class RuntimeAssetLoader {
  private readonly fetchImpl: typeof fetch;
  private readonly baseUrl: URL;
  private inFlight: Promise<AssetRegistry> | undefined;
  private registered?: AssetRegistry;

  constructor(options: RuntimeAssetLoadOptions = {}) {
    this.baseUrl = resolveBaseUrl(options.baseUrl);
    this.fetchImpl = options.fetchImpl ?? globalThis.fetch.bind(globalThis);
  }

  load(request: RuntimeAssetLoadRequest = {}): Promise<AssetRegistry> {
    throwIfAborted(request.signal);
    if (this.registered) return Promise.resolve(this.registered);
    if (this.inFlight) return this.inFlight;
    const operation = this.loadOnce(request).then(registry => {
      this.registered = registry;
      return registry;
    });
    this.inFlight = operation.finally(() => { this.inFlight = undefined; });
    return this.inFlight;
  }

  private async loadOnce(request: RuntimeAssetLoadRequest): Promise<AssetRegistry> {
    const startedAt = performance.now();
    const progress: RuntimeAssetLoadProgress = {
      phase: "fetching-manifests", completedFiles: 0, totalFiles: RUNTIME_ASSET_OUTPUT_COUNT + 2,
      completedOutputs: 0, totalOutputs: RUNTIME_ASSET_OUTPUT_COUNT, bytesFetched: 0, elapsedMs: 0,
    };
    const notify = (phase: RuntimeAssetLoadProgress["phase"], currentPath?: string): void => {
      progress.phase = phase;
      progress.elapsedMs = performance.now() - startedAt;
      if (currentPath === undefined) delete progress.currentPath;
      else progress.currentPath = currentPath;
      try { request.onProgress?.({ ...progress }); } catch { /* Observers cannot interrupt integrity checks. */ }
    };
    const read = async (path: string): Promise<Uint8Array> => {
      throwIfAborted(request.signal);
      let response: Response;
      try {
        response = await this.fetchImpl(new URL(path, this.baseUrl), {
          method: "GET", cache: "no-store", credentials: "same-origin",
          ...(request.signal ? { signal: request.signal } : {}),
        });
      } catch (error) {
        if (request.signal?.aborted || (error instanceof DOMException && error.name === "AbortError")) throw abortError();
        throw new Error(`E_RUNTIME_ASSET_FETCH:${path}`, { cause: error });
      }
      throwIfAborted(request.signal);
      if (!response.ok) throw new Error(`E_RUNTIME_ASSET_HTTP:${response.status}:${path}`);
      let bytes: Uint8Array;
      try { bytes = new Uint8Array(await response.arrayBuffer()); }
      catch (error) {
        if (request.signal?.aborted) throw abortError();
        throw new Error(`E_RUNTIME_ASSET_BODY:${path}`, { cause: error });
      }
      throwIfAborted(request.signal);
      progress.completedFiles++;
      progress.bytesFetched += bytes.byteLength;
      if (path.startsWith("assets/derived/")) progress.completedOutputs++;
      notify(path.startsWith("assets/derived/") ? "verifying-outputs" : "fetching-manifests", path);
      return bytes;
    };

    try {
      const authorityBytes = await read(RUNTIME_AUTHORITY_MANIFEST_PATH);
      const manifestBytes = await read(RUNTIME_ASSET_MANIFEST_PATH);
      if (await sha256(manifestBytes) !== RUNTIME_ASSET_MANIFEST_SHA256) {
        throw new Error("E_RUNTIME_ASSET_MANIFEST_SHA");
      }
      throwIfAborted(request.signal);
      const loadOutput: OutputBytesLoader = path => read(path);
      const registry = new AssetRegistry();
      await registry.registerManifest(manifestBytes, authorityBytes, loadOutput);
      throwIfAborted(request.signal);
      if (progress.completedOutputs !== RUNTIME_ASSET_OUTPUT_COUNT) {
        throw new Error(`E_RUNTIME_ASSET_OUTPUT_COUNT:${progress.completedOutputs}`);
      }
      notify("complete");
      return registry;
    } catch (error) {
      if (request.signal?.aborted) throw abortError();
      notify("error");
      throw error;
    }
  }
}
