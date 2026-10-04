import type { AssetRegistry } from "../../assets/AssetRegistry.js";

const PORTRAIT_ASSET_ID = "portrait.cenyao.v1";

export class HudPortrait {
  constructor(private readonly host: HTMLElement) {}

  bindRegistry(registry: AssetRegistry): string | null {
    try {
      const asset = registry.resolveAsset(PORTRAIT_ASSET_ID);
      const image = document.createElement("img");
      image.className = "hud-portrait-image";
      image.alt = "岑遥头像";
      image.decoding = "async";
      image.src = new URL(asset.textureUrl, document.baseURI).href;
      this.host.replaceChildren(image);
      this.host.dataset.assetState = "resolved";
      return null;
    } catch (error) {
      this.showPlaceholder();
      return `头像资产 ${PORTRAIT_ASSET_ID} 不可用：${error instanceof Error ? error.message : String(error)}`;
    }
  }

  showPlaceholder(): void {
    const placeholder = document.createElement("span");
    placeholder.className = "hud-portrait-placeholder";
    placeholder.textContent = "岑";
    placeholder.setAttribute("aria-label", "岑遥头像占位");
    this.host.replaceChildren(placeholder);
    this.host.dataset.assetState = "placeholder";
  }
}
