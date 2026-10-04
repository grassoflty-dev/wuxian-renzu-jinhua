import { InventoryCommandController, type BuildCommandHandler } from "./InventoryCommandController.js";
import type { WorldSnapshotV3 } from "../protocol/types.js";
import type { AssetRegistry } from "../assets/AssetRegistry.js";
import { CORE_PANEL_IDS, CORE_PANEL_LABELS, deriveCorePanel, type CorePanelId } from "./CorePanelModel.js";
import { renderCapabilityPanel } from "./components/CapabilityPanel.js";
import { renderCharacterPanel } from "./components/CharacterPanel.js";
import { renderInventoryPanel } from "./components/InventoryPanel.js";
import { renderGreyHiveStatusPanel } from "./components/GreyHiveStatusPanel.js";
import { renderMistHarborSonarMap } from "./components/MistHarborSonarMap.js";
import { AccessibilityPreferenceStore, bindAccessibilityPreferenceToggle, TEXT_SCALE_PERCENTAGES, UI_SCALE_PERCENTAGES } from "./AccessibilityPreferences.js";
import type { AudioCuePlayer } from "../audio/AudioCuePlayer.js";

/** HTML/CSS Core UI renders snapshots and local accessibility/audio preferences. */
export class CoreUiPresenter {
  private snapshot: WorldSnapshotV3 | null = null;
  private current: CorePanelId = "character";
  private returnFocus: HTMLElement | null = null;
  private readonly title: HTMLElement;
  private readonly subtitle: HTMLElement;
  private readonly content: HTMLElement;
  private readonly closeButton: HTMLButtonElement;
  private readonly tabs: HTMLButtonElement[];
  private portraitUrl: string | undefined;
  private audioStatus: HTMLElement | null = null;
  private audioMuteButton: HTMLButtonElement | null = null;
  private audioVolume: HTMLInputElement | null = null;
  private audioVolumeValue: HTMLOutputElement | null = null;
  private selectedInventoryItem: string | null = null;
  private readonly inventoryCommands: InventoryCommandController;

  constructor(
    private readonly root: HTMLElement,
    private readonly accessibilityPreferences: AccessibilityPreferenceStore,
    private readonly audioCuePlayer?: AudioCuePlayer,
    onBuildCommand?: BuildCommandHandler,
  ) {
    this.inventoryCommands = new InventoryCommandController(onBuildCommand, () => {
      if (this.isOpen && this.current === "inventory") this.render();
    });
    this.title = this.require<HTMLElement>("#core-panel-title");
    this.subtitle = this.require<HTMLElement>("#core-panel-subtitle");
    this.content = this.require<HTMLElement>("#core-panel-content");
    this.closeButton = this.require<HTMLButtonElement>("#core-panel-close");
    this.tabs = Array.from(root.querySelectorAll<HTMLButtonElement>("[data-core-panel]"));
    this.audioCuePlayer?.subscribe(() => this.syncAudioControls());
    for (const tab of this.tabs) {
      tab.addEventListener("click", () => {
        const candidate = tab.dataset.corePanel;
        if (candidate && CORE_PANEL_IDS.includes(candidate as CorePanelId)) this.select(candidate as CorePanelId);
      });
      tab.addEventListener("keydown", event => {
        if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
        event.preventDefault();
        const index = this.tabs.indexOf(tab);
        const next = event.key === "Home" ? 0 : event.key === "End" ? this.tabs.length - 1
          : (index + (event.key === "ArrowRight" ? 1 : -1) + this.tabs.length) % this.tabs.length;
        this.tabs[next]?.focus();
        const candidate = this.tabs[next]?.dataset.corePanel;
        if (candidate && CORE_PANEL_IDS.includes(candidate as CorePanelId)) this.select(candidate as CorePanelId);
      });
    }
    this.closeButton.addEventListener("click", () => this.close());
    root.addEventListener("keydown", event => {
      if (event.key === "Escape") { event.preventDefault(); this.close(); }
      if (event.key !== "Tab" || !this.isOpen) return;
      const focusable = Array.from(this.root.querySelectorAll<HTMLElement>("button:not([disabled]):not([tabindex=\"-1\"]), select:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex=\"-1\"])"));
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (!first || !last) return;
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    });
  }

  private require<T extends HTMLElement>(selector: string): T {
    const element = this.root.querySelector<T>(selector);
    if (!element) throw new Error(`E_CORE_UI_ELEMENT_MISSING:${selector}`);
    return element;
  }

  get isOpen(): boolean { return !this.root.hidden; }
  get currentSnapshot(): WorldSnapshotV3 | null { return this.snapshot; }

  apply(snapshot: WorldSnapshotV3 | null): void {
    const previous = this.snapshot;
    this.snapshot = snapshot;
    this.inventoryCommands.apply(snapshot);
    if (!snapshot || previous?.worldEpoch !== snapshot.worldEpoch) this.selectedInventoryItem = null;
    const sameInventory = this.current === "inventory" && previous?.worldEpoch === snapshot?.worldEpoch &&
      previous?.worldId === snapshot?.worldId && previous?.sceneId === snapshot?.sceneId &&
      previous?.build?.revision === snapshot?.build?.revision;
    // Settings are local and do not depend on snapshots; preserve keyboard focus while the world updates.
    if (this.isOpen && !sameInventory && this.current !== "settings") this.render();
  }

  setPortraitRegistry(registry: AssetRegistry): string | null {
    try {
      const asset = registry.resolveAsset("portrait.cenyao.v1");
      this.portraitUrl = new URL(asset.textureUrl, document.baseURI).href;
      if (this.isOpen && this.current === "character") this.render();
      return null;
    } catch (error) {
      this.portraitUrl = undefined;
      if (this.isOpen && this.current === "character") this.render();
      return `头像资产 portrait.cenyao.v1 不可用：${error instanceof Error ? error.message : String(error)}`;
    }
  }

  open(trigger?: HTMLElement, id: CorePanelId = "character"): void {
    this.returnFocus = trigger ?? null;
    this.root.hidden = false;
    this.select(id);
    this.closeButton.focus();
  }

  close(): void {
    if (!this.isOpen) return;
    this.inventoryCommands.cancel();
    this.root.hidden = true;
    this.returnFocus?.focus();
    this.returnFocus = null;
  }

  select(id: CorePanelId): void {
    if (id !== this.current) this.inventoryCommands.cancel();
    this.current = id;
    this.render();
  }

  private render(): void {
    const view = deriveCorePanel(this.current, this.snapshot);
    this.title.textContent = view.title;
    this.subtitle.textContent = this.current === "settings"
      ? "本机辅助偏好 · 始终可用"
      : view.subtitle;
    for (const tab of this.tabs) {
      const selected = tab.dataset.corePanel === this.current;
      tab.setAttribute("aria-selected", String(selected));
      tab.setAttribute("aria-current", selected ? "page" : "false");
      tab.tabIndex = selected ? 0 : -1;
      tab.textContent = CORE_PANEL_LABELS[tab.dataset.corePanel as CorePanelId] ?? tab.textContent;
    }
    this.content.className = `core-panel-content core-panel-content-${this.current}`;
    if (this.current === "inventory") {
      const focus = this.content.contains(document.activeElement) && document.activeElement instanceof HTMLElement
        ? document.activeElement.dataset.inventoryFocus : undefined;
      renderInventoryPanel(this.content, {
        build: this.snapshot?.build,
        itemMessage: "尚未收到权威背包投影，暂不显示物品。",
        selectedItemId: this.selectedInventoryItem,
        pending: this.inventoryCommands.pending,
        feedback: this.inventoryCommands.feedback,
        onSelect: id => { this.selectedInventoryItem = id; this.render(); },
        onCommand: action => { void this.inventoryCommands.submit(action); },
      });
      if (focus) {
        const targets = Array.from(this.content.querySelectorAll<HTMLButtonElement>("button[data-inventory-focus]"));
        const target = targets.find(button => button.dataset.inventoryFocus === focus && !button.disabled)
          ?? targets.find(button => button.getAttribute("aria-pressed") === "true");
        target?.focus();
      }
      return;
    }
    if (this.current === "character" && this.snapshot) {
      renderCharacterPanel(this.content, this.snapshot, this.portraitUrl);
      return;
    }
    if (this.current === "capability" && this.snapshot) {
      renderCapabilityPanel(this.content, this.snapshot);
      return;
    }
    const list = document.createElement("dl");
    list.className = "core-panel-list";
    for (const row of view.rows) {
      const line = document.createElement("div");
      line.className = "core-panel-row";
      const term = document.createElement("dt");
      term.textContent = row.label;
      const value = document.createElement("dd");
      value.textContent = row.value;
      line.append(term, value);
      list.append(line);
    }
    this.content.replaceChildren(list);
    if (this.current === "mission" && this.snapshot?.worldId === "grey_hive") {
      const status = document.createElement("div");
      renderGreyHiveStatusPanel(status, this.snapshot);
      this.content.prepend(status);
    }
    if (this.current === "mission" && this.snapshot?.worldId === "mist_harbor") {
      const sonar = document.createElement("section");
      renderMistHarborSonarMap(sonar, this.snapshot);
      this.content.prepend(sonar);
    }
    if (this.current === "settings") {
      this.renderAudioControls();
      this.renderAccessibilityControls();
    }
  }

  private renderAudioControls(): void {
    if (!this.audioCuePlayer) return;
    const section = document.createElement("section");
    section.className = "core-audio-settings core-panel-list";

    const title = document.createElement("h3");
    title.textContent = "音频";

    const volumeRow = document.createElement("div");
    volumeRow.className = "core-panel-row";
    const volumeLabel = document.createElement("span");
    volumeLabel.textContent = "主音量";
    const volume = document.createElement("input");
    volume.type = "range";
    volume.min = "0";
    volume.max = "1";
    volume.step = "0.05";
    volume.setAttribute("aria-label", "主音量");
    const value = document.createElement("output");
    const volumeTerm = document.createElement("dt");
    volumeTerm.append(volumeLabel);
    const volumeControls = document.createElement("dd");
    volumeControls.append(volume, value);
    volumeRow.append(volumeTerm, volumeControls);
    volume.addEventListener("input", () => this.audioCuePlayer?.setVolume(Number(volume.value)));

    const mute = document.createElement("button");
    mute.type = "button";
    mute.className = "core-toggle";
    mute.textContent = "静音";
    mute.addEventListener("click", () => {
      if (!this.audioCuePlayer) return;
      this.audioCuePlayer.setMuted(!this.audioCuePlayer.settings.muted);
      if (!this.audioCuePlayer.settings.muted) void this.audioCuePlayer.unlock();
    });

    const status = document.createElement("p");
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    const statusRow = document.createElement("div");
    statusRow.className = "core-panel-row";
    const statusTerm = document.createElement("dt");
    statusTerm.textContent = "播放状态";
    const statusValue = document.createElement("dd");
    statusValue.append(status);
    statusRow.append(statusTerm, statusValue);

    const note = document.createElement("p");
    note.textContent = "当前音效由代码即时合成，均为临时声音。";
    const noteRow = document.createElement("div");
    noteRow.className = "core-panel-row";
    const noteTerm = document.createElement("dt");
    noteTerm.textContent = "声音来源";
    const noteValue = document.createElement("dd");
    noteValue.append(note);
    noteRow.append(noteTerm, noteValue);

    section.append(title, volumeRow, mute, statusRow, noteRow);
    this.content.append(section);
    this.audioStatus = status;
    this.audioMuteButton = mute;
    this.audioVolume = volume;
    this.audioVolumeValue = value;
    this.syncAudioControls();
  }

  private syncAudioControls(): void {
    if (!this.audioCuePlayer) return;
    if (this.audioStatus?.isConnected) this.audioStatus.textContent = this.audioCuePlayer.statusText;
    if (this.audioMuteButton?.isConnected) {
      this.audioMuteButton.setAttribute("aria-pressed", String(this.audioCuePlayer.settings.muted));
    }
    if (this.audioVolume?.isConnected) {
      const percent = Math.round(this.audioCuePlayer.settings.volume * 100);
      this.audioVolume.value = String(percent / 100);
      this.audioVolume.setAttribute("aria-valuetext", `${percent}%`);
      if (this.audioVolumeValue?.isConnected) this.audioVolumeValue.value = `${percent}%`;
    }
  }

  private renderAccessibilityControls(): void {
    const controls = document.createElement("div");
    controls.className = "core-accessibility";
    for (const setting of [
      { label: "高对比度", key: "highContrast" },
      { label: "减少动态效果", key: "reducedMotion" },
    ] as const) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "core-toggle";
      button.textContent = setting.label;
      bindAccessibilityPreferenceToggle({
        setPressed: pressed => button.setAttribute("aria-pressed", String(pressed)),
        onToggle: handler => button.addEventListener("click", handler),
      }, this.accessibilityPreferences, setting.key);
      controls.append(button);
    }
    for (const setting of [
      { label: "文字大小", key: "textScalePercent", options: TEXT_SCALE_PERCENTAGES },
      { label: "界面缩放", key: "uiScalePercent", options: UI_SCALE_PERCENTAGES },
    ] as const) {
      const label = document.createElement("label"); label.className = "core-scale-control";
      const title = document.createElement("span"); title.textContent = setting.label;
      const select = document.createElement("select"); select.id = `settings-${setting.key}`;
      label.htmlFor = select.id;
      for (const value of setting.options) {
        const option = document.createElement("option"); option.value = String(value);
        option.textContent = `${value}%${value === 100 ? "（默认）" : ""}`; select.append(option);
      }
      select.value = String(this.accessibilityPreferences.getScale()[setting.key]);
      select.addEventListener("change", () => {
        const value = this.accessibilityPreferences.setScale(setting.key, Number(select.value));
        select.value = String(value[setting.key]);
      });
      label.append(title, select); controls.append(label);
    }
    this.content.append(controls);
  }
}
