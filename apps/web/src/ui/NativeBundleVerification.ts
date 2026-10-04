import { isBuildIdentity, type BuildIdentity } from "./BuildIdentityOverlay.js";

export interface NativeEntryIdentity {
  path: string;
  sizeBytes: number;
  sha256: string;
}

export interface NativeBuildIdentity {
  schemaVersion: 1;
  buildIdentity: BuildIdentity;
  sidecarSha256: string;
  entry: NativeEntryIdentity;
  fileCount: number;
}

export type NativeBundleVerificationResult =
  | { status: "PASS"; message: string; entryPath: string; sizeBytes: number; sha256: string }
  | { status: "UNSATISFIED"; message: string };

export interface NativeBundleVerificationDependencies {
  isTauri: () => boolean;
  getNativeIdentity: () => Promise<unknown>;
  entryModuleUrl: string;
  fetcher?: typeof fetch;
  subtle?: SubtleCrypto;
  timeoutMs?: number;
}

const SHA64 = /^[0-9a-f]{64}$/;
const PASS_MESSAGE = "当前 WebView 可取回入口字节与 Rust 锁定入口一致。此项仅证明入口字节一致，不代表完整性闭环，也不替代截图、EXE 或安装包原生验收。";

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

export function isNativeBuildIdentity(value: unknown): value is NativeBuildIdentity {
  if (!isRecord(value) || value.schemaVersion !== 1 || !isBuildIdentity(value.buildIdentity) ||
      typeof value.sidecarSha256 !== "string" || !SHA64.test(value.sidecarSha256) ||
      !Number.isSafeInteger(value.fileCount) || (value.fileCount as number) < 1 || !isRecord(value.entry)) return false;
  const entry = value.entry;
  return typeof entry.path === "string" && entry.path.length > 0 &&
    !entry.path.startsWith("/") && !entry.path.includes("\\") &&
    !entry.path.split("/").some(segment => !segment || segment === "." || segment === "..") &&
    Number.isSafeInteger(entry.sizeBytes) && (entry.sizeBytes as number) >= 0 &&
    typeof entry.sha256 === "string" && SHA64.test(entry.sha256);
}

function sameBuildIdentity(left: BuildIdentity, right: BuildIdentity): boolean {
  const keys: (keyof BuildIdentity)[] = [
    "schemaVersion", "gitSha", "appVersion", "contentVersion", "saveV6SchemaVersion",
    "runtimeAssetManifestSha256", "sceneDefinitionManifestSha256", "builtAtUtc",
    "hasTrackedDiff", "hasUntrackedFiles", "sourceTreeDirty",
  ];
  return keys.every(key => left[key] === right[key]);
}

export function entryPathFromModuleUrl(moduleUrl: string): string | null {
  try {
    const url = new URL(moduleUrl);
    if (!url.pathname || url.pathname.endsWith("/")) return null;
    const path = decodeURIComponent(url.pathname).replace(/^\/+/, "");
    if (!path || path.split("/").some(segment => !segment || segment === "." || segment === "..")) return null;
    return path;
  } catch {
    return null;
  }
}

function timeout<T>(promise: Promise<T>, timeoutMs: number, reason: string): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timed = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(reason)), timeoutMs);
  });
  return Promise.race([promise, timed]).finally(() => {
    if (timer !== undefined) clearTimeout(timer);
  });
}

export async function verifyNativeBundleEntry(
  expectedIdentity: unknown,
  dependencies: NativeBundleVerificationDependencies,
): Promise<NativeBundleVerificationResult> {
  const unsatisfied = (message: string): NativeBundleVerificationResult => ({ status: "UNSATISFIED", message: `UNSATISFIED：${message}` });
  if (!isBuildIdentity(expectedIdentity)) return unsatisfied("前端编译身份不可用。");
  if (!dependencies.isTauri()) return unsatisfied("当前为 Web 上下文，没有调用原生身份接口。");
  const timeoutMs = dependencies.timeoutMs ?? 7000;
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) return unsatisfied("核验超时设置无效。");
  const actualPath = entryPathFromModuleUrl(dependencies.entryModuleUrl);
  if (!actualPath) return unsatisfied("无法从当前入口模块 URL 解析入口路径。");

  try {
    const identity = await timeout(dependencies.getNativeIdentity(), timeoutMs, "E_NATIVE_IDENTITY_TIMEOUT");
    if (!isNativeBuildIdentity(identity)) return unsatisfied("Rust 返回的入口身份缺失或格式无效。");
    if (!sameBuildIdentity(identity.buildIdentity, expectedIdentity)) return unsatisfied("Rust 锁定的构建身份与当前前端身份不一致。");
    if (identity.entry.path !== actualPath) return unsatisfied("当前入口 URL 路径与 Rust 锁定入口路径不一致。");

    const response = await timeout((dependencies.fetcher ?? fetch)(dependencies.entryModuleUrl, { cache: "no-store" }), timeoutMs, "E_ENTRY_FETCH_TIMEOUT");
    if (!response.ok) return unsatisfied(`当前入口字节无法取回（HTTP ${response.status}）。`);
    const bytes = await timeout(response.arrayBuffer(), timeoutMs, "E_ENTRY_READ_TIMEOUT");
    if (bytes.byteLength !== identity.entry.sizeBytes) return unsatisfied("当前入口字节数与 Rust 锁定值不一致。");
    const subtle = dependencies.subtle ?? globalThis.crypto?.subtle;
    if (!subtle) return unsatisfied("当前环境不支持入口 SHA-256 核验。");
    const digest = await subtle.digest("SHA-256", bytes);
    const sha256 = Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("");
    if (sha256 !== identity.entry.sha256) return unsatisfied("当前入口 SHA-256 与 Rust 锁定值不一致。");
    return { status: "PASS", message: PASS_MESSAGE, entryPath: actualPath, sizeBytes: bytes.byteLength, sha256 };
  } catch (error) {
    const code = error instanceof Error ? error.message : "E_NATIVE_BUNDLE_VERIFY";
    return unsatisfied(code.includes("TIMEOUT") ? "原生身份或入口读取超时。" : "原生身份或入口读取失败。");
  }
}
