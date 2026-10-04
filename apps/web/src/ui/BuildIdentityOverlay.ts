import { verifyNativeBundleEntry, type NativeBundleVerificationDependencies } from "./NativeBundleVerification.js";

export interface BuildIdentity {
  schemaVersion: 1;
  gitSha: string;
  appVersion: string;
  contentVersion: string;
  saveV6SchemaVersion: number;
  runtimeAssetManifestSha256: string;
  sceneDefinitionManifestSha256: string;
  builtAtUtc: string;
  hasTrackedDiff: boolean;
  hasUntrackedFiles: boolean;
  sourceTreeDirty: boolean;
}

const SHA40 = /^[0-9a-f]{40}$/;
const SHA64 = /^[0-9a-f]{64}$/;
const VERSION = /^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const CONTENT_VERSION = /^[A-Za-z0-9._+-]{1,96}$/;

export function isBuildIdentity(value: unknown): value is BuildIdentity {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const identity = value as Partial<BuildIdentity>;
  if (identity.schemaVersion !== 1 ||
      typeof identity.gitSha !== "string" || !SHA40.test(identity.gitSha) ||
      typeof identity.appVersion !== "string" || !VERSION.test(identity.appVersion) ||
      typeof identity.contentVersion !== "string" || !CONTENT_VERSION.test(identity.contentVersion) ||
      !Number.isSafeInteger(identity.saveV6SchemaVersion) || (identity.saveV6SchemaVersion ?? 0) <= 0 ||
      typeof identity.runtimeAssetManifestSha256 !== "string" || !SHA64.test(identity.runtimeAssetManifestSha256) ||
      typeof identity.sceneDefinitionManifestSha256 !== "string" || !SHA64.test(identity.sceneDefinitionManifestSha256) ||
      typeof identity.builtAtUtc !== "string" || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$/.test(identity.builtAtUtc) ||
      typeof identity.hasTrackedDiff !== "boolean" ||
      typeof identity.hasUntrackedFiles !== "boolean" ||
      typeof identity.sourceTreeDirty !== "boolean") return false;
  return identity.sourceTreeDirty === (identity.hasTrackedDiff || identity.hasUntrackedFiles);
}

export function formatBuildIdentity(value: unknown): string | null {
  if (!isBuildIdentity(value)) return null;
  const dirty = value.hasTrackedDiff || value.hasUntrackedFiles;
  const sourceState = dirty
    ? "源码状态：有未提交或未跟踪内容；此 SHA 只指向 HEAD，不能代表当前全部源码。"
    : "源码状态：HEAD 工作树干净。";
  return [
    "前端编译身份",
    "Git HEAD: " + value.gitSha,
    "应用版本: " + value.appVersion,
    "内容版本: " + value.contentVersion,
    "Save V6 Schema: " + value.saveV6SchemaVersion,
    "Runtime Asset Manifest SHA-256: " + value.runtimeAssetManifestSha256,
    "Scene Definition Manifest SHA-256: " + value.sceneDefinitionManifestSha256,
    "构建时间 UTC: " + value.builtAtUtc,
    sourceState,
    "前端编译身份；尚未证明当前 EXE/安装包嵌入此 bundle。",
  ].join("\n");
}

export function shouldToggleBuildIdentity(event: Pick<KeyboardEvent, "key" | "ctrlKey" | "altKey" | "metaKey" | "shiftKey" | "repeat" | "defaultPrevented" | "target">): boolean {
  if (event.key !== "F3" || event.ctrlKey || event.altKey || event.metaKey || event.shiftKey ||
      event.repeat || event.defaultPrevented) return false;
  const target = event.target as (EventTarget & {
    tagName?: string;
    isContentEditable?: boolean;
    closest?: (selector: string) => Element | null;
  }) | null;
  if (!target) return true;
  const tagName = target.tagName?.toUpperCase();
  if (tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT" || target.isContentEditable) return false;
  if (typeof target.closest === "function" && target.closest("input, textarea, select, [contenteditable='true'], [role='textbox']")) return false;
  return true;
}

export class BuildIdentityOverlay {
  private generation = 0;
  private output: HTMLPreElement | null = null;

  constructor(
    private readonly host: HTMLElement,
    private readonly identity: unknown,
    private readonly verification?: NativeBundleVerificationDependencies,
  ) {
    host.hidden = true;
    host.setAttribute("aria-hidden", "true");
    host.setAttribute("aria-label", "前端构建身份");
    host.style.position = "absolute";
    host.style.top = "12px";
    host.style.right = "12px";
    host.style.zIndex = "20";
    host.style.width = "min(540px, calc(100% - 24px))";
    host.style.maxHeight = "min(70vh, 560px)";
    host.style.overflow = "auto";
    host.style.padding = "12px 14px";
    host.style.border = "1px solid rgba(137, 205, 212, .72)";
    host.style.background = "rgba(7, 17, 23, .96)";
    host.style.color = "#d9eaec";
    host.style.font = "12px/1.55 ui-monospace, Consolas, monospace";
    host.style.textShadow = "none";
    host.style.pointerEvents = "auto";
    host.style.userSelect = "text";
  }

  toggle(): void { this.setOpen(this.host.hidden); }

  setOpen(open: boolean): void {
    const generation = ++this.generation;
    this.host.hidden = !open;
    this.host.setAttribute("aria-hidden", String(!open));
    if (!open) return;
    const output = document.createElement("pre");
    output.style.margin = "0";
    output.style.whiteSpace = "pre-wrap";
    output.style.overflowWrap = "anywhere";
    output.setAttribute("role", "status");
    output.setAttribute("aria-live", "polite");
    output.style.color = "#58D7F0";
    output.textContent = `${formatBuildIdentity(this.identity) ?? "构建身份不可用：数据缺失或格式无效。"}\n\n入口字节核验：正在核验…`;
    this.output = output;
    this.host.replaceChildren(output);
    if (!this.verification) {
      output.textContent = `${formatBuildIdentity(this.identity) ?? "构建身份不可用：数据缺失或格式无效。"}\n\n入口字节核验：UNSATISFIED：核验器不可用。`;
      return;
    }
    void verifyNativeBundleEntry(this.identity, this.verification).then(result => {
      if (generation !== this.generation || this.host.hidden || this.output !== output) return;
      output.textContent = `${formatBuildIdentity(this.identity) ?? "构建身份不可用：数据缺失或格式无效。"}\n\n入口字节核验：${result.message}`;
    }, () => {
      if (generation !== this.generation || this.host.hidden || this.output !== output) return;
      output.textContent = `${formatBuildIdentity(this.identity) ?? "构建身份不可用：数据缺失或格式无效。"}\n\n入口字节核验：UNSATISFIED：核验失败。`;
    });
  }
}
