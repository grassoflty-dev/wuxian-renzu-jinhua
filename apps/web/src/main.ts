import { developerPresentationEnabled } from "./ui/DeveloperPresentation.js";
import { isClockworksEnemyCue } from "./renderer/ClockworksEnemyModel.js";
import { BuildCloseBarrier, isCurrentBuildReceipt } from "./ui/InventoryCommandController.js";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { RuntimeAssetLoader } from "./assets/RuntimeAssetLoader.js";
import { SceneDefinitionLoader } from "./assets/SceneDefinitionLoader.js";
import { TauriClient, type SaveSlotSummary } from "./bridge/tauri-client.js";
import { canContinueSaveSlot, canOverwriteSaveSlot } from "./bridge/save-slot-policy.js";
import { SessionLoop } from "./game/SessionLoop.js";
import { SceneDefinitionSession } from "./game/SceneDefinitionSession.js";
import { capabilityTerminalSummary } from "./game/CapabilityTerminal.js";
import { missionTerminalSummary } from "./game/MissionTerminal.js";
import { confirmedClockworksEpilogue, confirmedReturnStationAfterClockworks } from "./game/ClockworksCampaign.js";
import { confirmedGreyHiveNarrative } from "./game/GreyHiveNarrative.js";
import { confirmedMistHarborAcousticMappingLine, confirmedMistHarborBeaconSync, MistHarborSignalLineState } from "./game/MistHarborNarrative.js";
import { confirmedReturnStationAfterGreyHive, confirmedReturnStationNewJourney } from "./game/ReturnStationNarrative.js";
import { dispatchInteractable, interactionErrorText, isCurrentSceneInteractionResult, isCurrentSceneInteractionFeedback } from "./game/SceneInteraction.js";
import { WorldRenderer } from "./renderer/WorldRenderer.js";
import { combatRejectionText } from "./renderer/CombatFeedbackModel.js";
import { AudioCuePlayer } from "./audio/AudioCuePlayer.js";
import { EnhancementStatusHud } from "./ui/EnhancementStatusHud.js";
import { AccessibilityPreferenceStore } from "./ui/AccessibilityPreferences.js";
import { CoreUiPresenter } from "./ui/CoreUiPresenter.js";
import { HudPresenter, type HudElements } from "./ui/Hud.js";
import { ThemeManager } from "./ui/theme/ThemeManager.js";
import { BuildIdentityOverlay, shouldToggleBuildIdentity } from "./ui/BuildIdentityOverlay.js";
import type { WorldSnapshotV3 } from "./protocol/types.js";
import "./style.css";
import "./visual-authority.css";

const root = document.querySelector<HTMLDivElement>("#app");
if (!root) throw new Error("E_APP_ROOT_MISSING");

const accessibilityPreferences = new AccessibilityPreferenceStore(
  () => window.localStorage,
  document.documentElement,
);
accessibilityPreferences.restore();
const audioCuePlayer = new AudioCuePlayer();

root.innerHTML = `
  <main class="shell" data-view="hub" data-world="return_station">
    <div class="ambient ambient-one"></div><div class="ambient ambient-two"></div>
    <header class="topbar">
      <div class="brand"><span class="brand-mark" aria-hidden="true">◇</span><span>无限人族进化</span></div>
      <div class="topbar-status"><span class="status-light" id="connection-light"></span><span id="connection-label">正在连接归航链路</span><button class="core-open" id="core-ui-open" type="button">终端</button></div>
    </header>
    <section class="hub" id="hub" aria-labelledby="hub-title">
      <div class="hub-copy">
        <p class="eyebrow">RETURN STATION · 归航站</p>
        <h1 id="hub-title">每一次归来，<br /><em>都是新的进化。</em></h1>
        <p class="lead">从归航站出发，进入已接入的灰巢设施。其他坐标尚未开放。</p>
        <div class="actions" aria-label="主菜单">
          <button class="primary" id="new-journey" type="button" disabled>新旅程 <span aria-hidden="true">↗</span></button>
          <button class="secondary" id="continue-journey" type="button" disabled>继续</button>
          <button class="secondary" id="main-settings" type="button">设置</button>
          <button class="secondary menu-exit" id="exit-game" type="button">退出</button>
        </div>
        <section class="save-picker" id="continue-picker" hidden aria-label="选择存档继续"><label for="continue-slot">选择有效存档</label><select id="continue-slot"></select><p id="continue-slot-note" role="status"></p><button class="secondary" id="continue-selected" type="button" disabled>继续所选存档</button></section>
        <p class="feedback" id="feedback" role="status" aria-live="polite">正在读取当前状态…</p>
      </div>
      <div class="network" aria-label="三世界网络">
        <div class="network-ring ring-outer"></div><div class="network-ring ring-inner"></div>
        <div class="network-core"><span>R·S</span><small>归航站</small></div>
        <div class="network-node node-gh"><span>01</span><strong>灰巢设施</strong><small>GREY HIVE</small></div>
        <div class="network-node node-mh"><span>02</span><strong>雾港余烬</strong><small>待解析 · MIST HARBOR</small></div>
        <div class="network-node node-cw"><span>03</span><strong>钟骨工厂</strong><small>待解析 · CLOCKWORKS</small></div>
        <div class="network-line line-one"></div><div class="network-line line-two"></div><div class="network-line line-three"></div>
      </div>
    </section>
    <section class="journey" id="journey" aria-label="当前世界" hidden>
      <div class="world-frame">
        <canvas id="world-canvas" aria-label="2.5D 世界画布"></canvas>
        <section class="hud" id="game-hud" aria-label="游戏状态">
          <div class="hud-vitals">
            <div class="hud-identity"><div class="hud-portrait" id="hud-portrait" aria-label="岑遥头像"></div><div><strong>岑遥</strong><span id="hud-action-state">状态未报告</span></div></div>
            <div class="hud-meter"><span>HP</span><div class="meter-track"><i id="hp-fill" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0"></i></div><strong id="hp-value">— / —</strong></div>
            <div class="hud-meter energy"><span>ENERGY</span><div class="meter-track"><i id="energy-fill" role="meter" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0"></i></div><strong id="energy-value">— / —</strong></div>
          </div>
          <div class="hud-actions" id="hud-skills" aria-label="动作键位"></div>
          <div class="hud-world"><strong id="hud-world">—</strong><span id="hud-scene">—</span></div>
          <section class="hud-objectives" id="hud-objectives" aria-label="当前目标"><span class="hud-label">当前目标</span><ul id="hud-objective-list" aria-live="polite"></ul></section>
          <p class="hud-interaction" id="hud-interaction" role="status" aria-live="polite" hidden></p>
          <p class="hud-door" id="hud-door" aria-live="polite"></p>
          <p class="hud-pump-status" id="hud-pump-status" role="status" aria-live="polite" hidden></p>
          <div class="hud-boss"><p id="hud-boss-status" aria-label="首领状态" hidden></p><div class="meter-track" hidden><i id="boss-fill" role="meter" aria-label="首领生命" aria-valuemin="0" aria-valuemax="100" aria-valuenow="0"></i></div></div>
          <p class="hud-environment-status" id="hud-environment-status" aria-label="环境危险状态" hidden></p>
          <p class="hud-feedback" id="hud-feedback" role="status" aria-live="polite"></p>
          <button class="hud-pause" id="hud-pause" type="button" aria-pressed="false">暂停</button><button class="hud-return" id="back-to-hub" type="button">返回主界面</button>
          <div class="pause-overlay" id="pause-overlay" hidden><div><p class="eyebrow">PAUSE · 权威状态</p><strong id="pause-title">旅程已暂停</strong><p id="pause-detail"></p><section id="first-enhancement" hidden aria-label="灰巢首通强化"><strong>灰巢首通强化 · 任选一项</strong><div class="enhancement-choices"><button type="button" data-enhancement-id="information.local_map_i">局部地图</button><button type="button" data-enhancement-id="perception.rear_view_i">后方视野</button><button type="button" data-enhancement-id="body.regeneration_i">再生能力</button></div><p id="enhancement-feedback" role="status" aria-live="polite"></p></section><label for="pause-slot">保存到所选存档</label><select id="pause-slot" disabled><option value="">正在读取存档…</option></select><label for="new-slot-name">新存档名称</label><input id="new-slot-name" maxlength="48" value="现场记录" disabled /><div class="save-actions"><button id="save-new-slot" type="button" disabled>另存为新存档</button><button id="overwrite-slot" type="button" disabled>覆盖所选存档</button></div><p id="save-feedback" role="status" aria-live="polite"></p><button id="hud-resume" type="button">继续</button><section class="enhancement-status" id="enhancement-status" aria-label="首通强化状态" aria-live="polite" hidden></section></div></div>
          <div class="death-overlay" id="death-overlay" role="dialog" aria-modal="true" aria-labelledby="death-title" hidden>
            <div><p class="eyebrow">JOURNEY ENDED · 旅程终止</p><h2 id="death-title" tabindex="-1">你已倒下</h2><p>生命值已归零，本次现场已终止。死亡状态不会保存，也无法直接恢复。</p>
              <section id="death-save-picker" hidden><label for="death-slot">选择仍然存活的存档</label><select id="death-slot" disabled></select></section>
              <p id="death-feedback" role="status" aria-live="polite">正在查找可恢复的存档…</p>
              <button id="death-retry" type="button" disabled hidden>从所选存档重试</button>
              <button id="death-new-journey" type="button" hidden>开始新旅程</button>
              <button id="death-back-to-hub" type="button">返回主界面（不保存）</button>
            </div>
          </div>
        </section>
      </div>
      <div class="journey-panel" id="developer-journey-panel" hidden><p class="eyebrow">WORLD LINK ACTIVE</p><h2 id="world-title">灰巢设施</h2><p id="world-detail">正在接收世界状态…</p></div>
    </section>
    <aside id="build-identity-overlay" hidden></aside>
    <section class="core-panel" id="core-panel" role="dialog" aria-modal="true" aria-labelledby="core-panel-title" hidden>
      <div class="core-panel-frame">
        <header class="core-panel-header"><div><p class="eyebrow">RETURN STATION · CORE UI</p><h2 id="core-panel-title">角色</h2><p id="core-panel-subtitle">等待状态</p></div><button id="core-panel-close" type="button" aria-label="关闭终端">关闭 ×</button></header>
        <div class="core-panel-layout"><div class="core-panel-navigation"><nav class="core-panel-tabs core-panel-tabs-primary" role="tablist" aria-label="终端主导航"><button role="tab" type="button" data-core-panel="world_network">世界网络</button><button role="tab" type="button" data-core-panel="capability">能力</button><button role="tab" type="button" data-core-panel="mission">任务</button><button role="tab" type="button" data-core-panel="archive">档案</button><button role="tab" type="button" data-core-panel="settings">系统</button></nav><nav class="core-panel-tabs core-panel-tabs-secondary" role="tablist" aria-label="终端次级入口"><button role="tab" type="button" data-core-panel="inventory">背包</button><button role="tab" type="button" data-core-panel="character">角色</button><button role="tab" type="button" data-core-panel="save">存档</button></nav></div><div class="core-panel-content" id="core-panel-content"></div></div>
      </div>
    </section>
    <footer class="footer"><span>三世界链路</span><span>GREY HIVE · MIST HARBOR · CLOCKWORKS</span></footer>
  </main>`;

const shell = root.querySelector<HTMLElement>(".shell")!;
const themeManager = new ThemeManager();
const client = new TauriClient(invoke);
function applyWorldTheme(worldId: string): void {
  themeManager.applyWorld(worldId, shell);
}
applyWorldTheme(shell.dataset.world || "return_station");
const buildIdentityOverlay = new BuildIdentityOverlay(
  root.querySelector<HTMLElement>("#build-identity-overlay")!,
  __BUILD_IDENTITY__,
  {
    isTauri,
    getNativeIdentity: () => client.nativeBuildIdentity(),
    entryModuleUrl: import.meta.url,
    timeoutMs: 7000,
  },
);
window.addEventListener("keydown", event => {
  if (!shouldToggleBuildIdentity(event)) return;
  event.preventDefault();
  event.stopImmediatePropagation();
  buildIdentityOverlay.toggle();
});
const coreOpenButton = root.querySelector<HTMLButtonElement>("#core-ui-open")!;
const buildCloseBarrier = new BuildCloseBarrier();
let pendingHubBuild: Promise<import("./bridge/tauri-client.js").SlotCommandReceipt> | null = null;
async function waitForHubBuild(): Promise<void> { await pendingHubBuild?.catch(() => undefined); }
const coreUi = new CoreUiPresenter(root.querySelector<HTMLElement>("#core-panel")!, accessibilityPreferences, audioCuePlayer,
  async (action, source) => {
    if (busy || buildCloseBarrier.closing || nativeJourneyUncertain) throw new Error("E_BUILD_SESSION_CHANGED");
    const loop = sessionLoop;
    const generation = journeyRequestId;
    if (loop) {
      if (!loop.acceptsExternalResults || loop.snapshots.view()?.worldEpoch !== source.worldEpoch) throw new Error("E_BUILD_SESSION_CHANGED");
      const receipt = await loop.runWithSession(() => client.buildCommand(action, source));
      const current = loop.snapshots.view();
      if (!isCurrentBuildReceipt(sessionLoop === loop && loop.acceptsExternalResults,
        current?.protocolVersion === 3 ? current : null, source, receipt)) throw new Error("E_BUILD_RECEIPT_STALE_SESSION");
      await loop.acceptAuthoritativeSnapshot(receipt.snapshot);
      return receipt;
    }
    if (pendingHubBuild || !coreUi.currentSnapshot || !isTauri()) throw new Error("E_BUILD_SESSION_CHANGED");
    const operation = client.buildCommand(action, source);
    pendingHubBuild = operation;
    try {
      const receipt = await operation;
      if (!isCurrentBuildReceipt(!sessionLoop && !busy && generation === journeyRequestId,
        coreUi.currentSnapshot, source, receipt)) throw new Error("E_BUILD_RECEIPT_STALE_SESSION");
      coreUi.apply(receipt.snapshot);
      return receipt;
    } finally { if (pendingHubBuild === operation) pendingHubBuild = null; }
  });
document.addEventListener("pointerdown", event => {
  if (event.isTrusted) void audioCuePlayer.unlock();
}, { capture: true, passive: true });
document.addEventListener("keydown", event => {
  if (event.isTrusted) void audioCuePlayer.unlock();
}, true);
document.addEventListener("visibilitychange", () => {
  if (document.hidden) audioCuePlayer.suspend();
  else if (!sessionLoop || sessionLoop.pausePresentationState === "running") audioCuePlayer.resume();
});
root.addEventListener("click", event => {
  if (!event.isTrusted || !(event.target instanceof Element)) return;
  const button = event.target.closest("button");
  if (button && !button.disabled) audioCuePlayer.playUiConfirm();
}, true);
const feedback = root.querySelector<HTMLElement>("#feedback")!;
function worldDisplayName(worldId: string): string {
  switch (worldId) {
    case "return_station": return "归航站";
    case "grey_hive": return "灰巢设施";
    case "mist_harbor": return "雾港余烬";
    case "clockworks": return "钟骨工厂";
    default: return worldId || "未知区域";
  }
}
const connectionLabel = root.querySelector<HTMLElement>("#connection-label")!;
const connectionLight = root.querySelector<HTMLElement>("#connection-light")!;
const newButton = root.querySelector<HTMLButtonElement>("#new-journey")!;
const continueButton = root.querySelector<HTMLButtonElement>("#continue-journey")!;
const mainSettingsButton = root.querySelector<HTMLButtonElement>("#main-settings")!;
const exitButton = root.querySelector<HTMLButtonElement>("#exit-game")!;
const continuePicker = root.querySelector<HTMLElement>("#continue-picker")!;
const continueSlot = root.querySelector<HTMLSelectElement>("#continue-slot")!;
const continueNote = root.querySelector<HTMLElement>("#continue-slot-note")!;
const continueSelected = root.querySelector<HTMLButtonElement>("#continue-selected")!;
const pauseSlot = root.querySelector<HTMLSelectElement>("#pause-slot")!;
const newSlotName = root.querySelector<HTMLInputElement>("#new-slot-name")!;
const saveNewSlotButton = root.querySelector<HTMLButtonElement>("#save-new-slot")!;
const overwriteSlotButton = root.querySelector<HTMLButtonElement>("#overwrite-slot")!;
const saveFeedback = root.querySelector<HTMLElement>("#save-feedback")!;
const deathOverlay = root.querySelector<HTMLElement>("#death-overlay")!;
const deathTitle = root.querySelector<HTMLElement>("#death-title")!;
const deathSavePicker = root.querySelector<HTMLElement>("#death-save-picker")!;
const deathSlot = root.querySelector<HTMLSelectElement>("#death-slot")!;
const deathFeedback = root.querySelector<HTMLElement>("#death-feedback")!;
const deathRetryButton = root.querySelector<HTMLButtonElement>("#death-retry")!;
const deathNewButton = root.querySelector<HTMLButtonElement>("#death-new-journey")!;
const deathHubButton = root.querySelector<HTMLButtonElement>("#death-back-to-hub")!;
const enhancementSection = root.querySelector<HTMLElement>("#first-enhancement")!;
const enhancementFeedback = root.querySelector<HTMLElement>("#enhancement-feedback")!;
const enhancementButtons = Array.from(root.querySelectorAll<HTMLButtonElement>("[data-enhancement-id]"));
const hub = root.querySelector<HTMLElement>("#hub")!;
const journey = root.querySelector<HTMLElement>("#journey")!;
const worldTitle = root.querySelector<HTMLElement>("#world-title")!;
const worldDetail = root.querySelector<HTMLElement>("#world-detail")!;
const developerPresentation = developerPresentationEnabled(typeof location === "undefined" ? "" : location.search);
root.querySelector<HTMLElement>("#developer-journey-panel")!.hidden = !developerPresentation;
root.querySelector<HTMLElement>(".journey")!.classList.toggle("developer-presentation", developerPresentation);
let canvas = root.querySelector<HTMLCanvasElement>("#world-canvas")!;
const canvasHost = canvas.parentElement!;
const enhancementStatusHud = new EnhancementStatusHud(root.querySelector<HTMLElement>("#enhancement-status")!);
const hud = new HudPresenter({
  portrait: root.querySelector<HTMLElement>("#hud-portrait")!,
  actionState: root.querySelector<HTMLElement>("#hud-action-state")!,
  skills: root.querySelector<HTMLElement>("#hud-skills")!,
  objectivePanel: root.querySelector<HTMLElement>("#hud-objectives")!,
  interactionPrompt: root.querySelector<HTMLElement>("#hud-interaction")!,
  hpValue: root.querySelector<HTMLElement>("#hp-value")!,
  hpFill: root.querySelector<HTMLElement>("#hp-fill")!,
  energyValue: root.querySelector<HTMLElement>("#energy-value")!,
  energyFill: root.querySelector<HTMLElement>("#energy-fill")!,
  world: root.querySelector<HTMLElement>("#hud-world")!,
  scene: root.querySelector<HTMLElement>("#hud-scene")!,
  objectives: root.querySelector<HTMLElement>("#hud-objectives")!,
  interaction: root.querySelector<HTMLElement>("#hud-interaction")!,
  doorStatus: root.querySelector<HTMLElement>("#hud-door")!,
  pumpStatus: root.querySelector<HTMLElement>("#hud-pump-status")!,
  bossStatus: root.querySelector<HTMLElement>("#hud-boss-status")!,
  bossFill: root.querySelector<HTMLElement>("#boss-fill")!,
  environmentStatus: root.querySelector<HTMLElement>("#hud-environment-status")!,
  feedback: root.querySelector<HTMLElement>("#hud-feedback")!,
  pauseButton: root.querySelector<HTMLButtonElement>("#hud-pause")!,
  pauseOverlay: root.querySelector<HTMLElement>("#pause-overlay")!,
  pauseTitle: root.querySelector<HTMLElement>("#pause-title")!,
  pauseDetail: root.querySelector<HTMLElement>("#pause-detail")!,
  resumeButton: root.querySelector<HTMLButtonElement>("#hud-resume")!,
  deathOverlay,
} satisfies HudElements);

const runtimeAssetLoader = new RuntimeAssetLoader();
const sceneDefinitionLoader = new SceneDefinitionLoader({
  baseUrl: document.baseURI,
  expectedManifestSha256: __SCENE_DEFINITION_MANIFEST_SHA256__,
});
let renderer: WorldRenderer | null = null;
let sceneSession: SceneDefinitionSession | null = null;
let sessionLoop: SessionLoop | null = null;
let activeJourneyLoad: AbortController | null = null;
let journeyRequestId = 0;
let busy = false;
// A timed-out native reset may still finish later. Require a process restart before another entry.
let nativeJourneyUncertain = false;
let interactionBusy = false;
let hudIdentity: string | null = null;
let saveSlots: SaveSlotSummary[] = [];
let slotsReady = false;
// Fallback only: named slots retain their existing explicit-selection behavior.
let latestSaveAvailable = false;
let slotReadId = 0;
let deathRecoveryTarget: "slot" | "latest" | "new" | null = null;
let saveBusy = false;
let enhancementBusy = false;

function selectedSlot(select: HTMLSelectElement): SaveSlotSummary | undefined {
  return saveSlots.find(slot => slot.slotId === select.value);
}

function renderSlotOptions(select: HTMLSelectElement, selectedId?: string): void {
  select.replaceChildren();
  for (const slot of saveSlots) {
    const option = document.createElement("option");
    option.value = slot.slotId;
    const kind = !canContinueSaveSlot(slot) ? `不可继续 · ${slot.errorCode || "无有效生命值"}` : slot.readOnly ? "旧版只读档 · 续玩时安全迁移" : "可读写";
    option.textContent = `${slot.displayName} · ${slot.worldId || "未知世界"} · ${kind}`;
    option.disabled = !canContinueSaveSlot(slot);
    select.append(option);
  }
  if (selectedId && saveSlots.some(slot => slot.slotId === selectedId)) select.value = selectedId;
  else select.selectedIndex = -1;
}

function updateSaveControls(): void {
  const paused = sessionLoop?.pausePresentationState === "paused" && !sessionLoop.isDead;
  const slot = selectedSlot(pauseSlot);
  pauseSlot.disabled = !paused || saveBusy || !slotsReady;
  newSlotName.disabled = !paused || saveBusy;
  saveNewSlotButton.disabled = !paused || saveBusy || !slotsReady || !newSlotName.value.trim();
  overwriteSlotButton.disabled = !paused || saveBusy || !canOverwriteSaveSlot(slot);
}

function updateEnhancementControls(): void {
  const view = sessionLoop?.snapshots.view();
  const snapshot = view?.protocolVersion === 3 ? view : null;
  const progress = snapshot?.progression.worlds.find(item => {
    if (!item || typeof item !== "object") return false;
    return (item as Record<string, unknown>).worldId === "grey_hive" &&
      (item as Record<string, unknown>).completed === true &&
      (item as Record<string, unknown>).firstCompletion === true;
  });
  const capabilities = snapshot?.capabilities as { items?: unknown } | undefined;
  const selected = Array.isArray(capabilities?.items) && capabilities.items.some(item =>
    !!item && typeof item === "object" &&
    ["information.local_map_i", "perception.rear_view_i", "body.regeneration_i"].includes(
      String((item as Record<string, unknown>).capabilityId)) &&
    ((item as Record<string, unknown>).granted === true || (item as Record<string, unknown>).selected === true));
  const visible = sessionLoop?.pausePresentationState === "paused" && snapshot?.worldId === "grey_hive" &&
    !!progress && !selected;
  enhancementSection.hidden = !visible;
  for (const button of enhancementButtons) button.disabled = !visible || enhancementBusy;
}

const enhancementNames: Record<string, string> = {
  "information.local_map_i": "局部地图",
  "perception.rear_view_i": "后方视野",
  "body.regeneration_i": "再生能力",
};

async function chooseFirstEnhancement(button: HTMLButtonElement): Promise<void> {
  const loop = sessionLoop;
  const capabilityId = button.dataset.enhancementId;
  const source = loop?.snapshots.view();
  if (!loop || loop.isDead || !capabilityId || !source || source.protocolVersion !== 3 || enhancementBusy ||
      loop.pausePresentationState !== "paused") return;
  enhancementBusy = true;
  enhancementFeedback.textContent = "正在确认所选强化…";
  updateEnhancementControls();
  try {
    const receipt = await loop.runWhilePaused(() => client.chooseFirstEnhancement(capabilityId));
    if (sessionLoop !== loop || !loop.acceptsExternalResults || loop.pausePresentationState !== "paused") return;
    const current = loop.snapshots.view();
    if (!current || current.protocolVersion !== 3 || current.worldId !== source.worldId ||
        current.sceneId !== source.sceneId || current.worldEpoch !== source.worldEpoch ||
        receipt.snapshot.worldId !== source.worldId || receipt.snapshot.sceneId !== source.sceneId ||
        receipt.snapshot.worldEpoch !== source.worldEpoch || receipt.authorityRevision <= source.authorityRevision) {
      throw new Error("E_ENHANCEMENT_RECEIPT_STALE_SESSION");
    }
    await loop.acceptAuthoritativeSnapshot(receipt.snapshot);
    if (sessionLoop !== loop || !loop.acceptsExternalResults) return;
    hud.setFeedback(`已选择「${enhancementNames[capabilityId] || capabilityId}」，强化已生效。`);
    updateEnhancementControls();
  } catch (error) {
    if (sessionLoop === loop && loop.acceptsExternalResults) {
      const message = `选择失败：${error instanceof Error ? error.message : String(error)}`;
      enhancementFeedback.textContent = message;
      if (loop.pausePresentationState !== "paused") hud.setFeedback(message);
    }
  } finally {
    enhancementBusy = false;
    updateEnhancementControls();
  }
}

for (const button of enhancementButtons) button.addEventListener("click", () => void chooseFirstEnhancement(button));

async function refreshSaveSlots(): Promise<boolean | undefined> {
  const readId = ++slotReadId;
  const generation = journeyRequestId;
  const owner = sessionLoop;
  const isCurrent = () => readId === slotReadId && generation === journeyRequestId && sessionLoop === owner;
  slotsReady = false;
  latestSaveAvailable = false;
  continueButton.disabled = true;
  continueSelected.disabled = true;
  continueSlot.disabled = true;
  try {
    const slots = await client.listSaveSlots();
    if (!isCurrent()) return;
    // Probe the default source independently: named slots may change between reads.
    const hasLatestSave = !slots.some(canContinueSaveSlot) && await client.hasSave(true);
    if (!isCurrent()) return;
    saveSlots = slots;
    latestSaveAvailable = hasLatestSave;
    renderSlotOptions(continueSlot);
    renderSlotOptions(pauseSlot);
    if (latestSaveAvailable) {
      const option = document.createElement("option");
      option.value = "";
      option.textContent = "最近自动存档";
      continueSlot.append(option);
      continueSlot.value = "";
    }
    continueSlot.disabled = latestSaveAvailable;
    continueSelected.textContent = latestSaveAvailable ? "继续自动存档" : "继续所选存档";
    continueNote.textContent = latestSaveAvailable
      ? "找到可恢复的存活自动存档。确认后将从此存档继续。"
      : saveSlots.some(canContinueSaveSlot)
      ? "旧版只读档会在明确选择续玩时由 Rust 备份迁移。"
      : "当前没有可继续的有效存档。";
    slotsReady = true;
  } catch (error) {
    if (!isCurrent()) return;
    saveSlots = [];
    continueNote.textContent = `读取存档失败：${error instanceof Error ? error.message : String(error)}`;
  }
  continueButton.disabled = nativeJourneyUncertain || !isTauri() || !slotsReady ||
    !(latestSaveAvailable || saveSlots.some(canContinueSaveSlot));
  continueSelected.disabled = nativeJourneyUncertain || !slotsReady ||
    !(latestSaveAvailable || canContinueSaveSlot(selectedSlot(continueSlot))) || busy;
  updateSaveControls();
  return slotsReady;
}

function slotStatusText(slot: SaveSlotSummary): string {
  if (!canContinueSaveSlot(slot)) return `此存档不可继续：${slot.errorCode || "没有有效的存活状态"}`;
  if (slot.readOnly) return "这是旧版只读存档。继续时 Rust 会先备份并安全迁移；不能覆盖。";
  return `将从「${slot.displayName}」恢复 ${slot.worldId || "当前世界"}。`;
}

function resetDeathControls(): void {
  slotReadId++;
  slotsReady = false;
  latestSaveAvailable = false;
  continueButton.disabled = true;
  continueSelected.disabled = true;
  deathRecoveryTarget = null;
  deathOverlay.hidden = true;
  deathSavePicker.hidden = true;
  deathRetryButton.hidden = true;
  deathRetryButton.disabled = true;
  deathNewButton.hidden = true;
  deathSlot.disabled = true;
  deathSlot.replaceChildren();
  coreOpenButton.disabled = false;
}

function updateDeathControls(): void {
  const enabled = !!sessionLoop?.isDead && !busy && !nativeJourneyUncertain && !buildCloseBarrier.closing;
  deathSlot.disabled = !enabled || !slotsReady || deathRecoveryTarget !== "slot";
  deathRetryButton.disabled = !enabled || !slotsReady || !(deathRecoveryTarget === "latest" ||
    (deathRecoveryTarget === "slot" && canContinueSaveSlot(selectedSlot(deathSlot))));
  deathNewButton.disabled = !enabled || deathRecoveryTarget !== "new";
  deathHubButton.disabled = !enabled;
}

async function showDeathRecovery(loop: SessionLoop): Promise<void> {
  if (sessionLoop !== loop || !loop.isDead) return;
  const generation = journeyRequestId;
  deathRecoveryTarget = null;
  coreUi.close();
  coreOpenButton.disabled = true;
  hud.setPauseStatus("dead");
  enhancementSection.hidden = true;
  deathSavePicker.hidden = true;
  deathRetryButton.hidden = true;
  deathNewButton.hidden = true;
  deathFeedback.textContent = "正在查找可恢复的存活存档…";
  deathTitle.focus();
  updateSaveControls();
  updateDeathControls();
  const readId = slotReadId + 1;
  const isCurrent = () => sessionLoop === loop && loop.isDead && generation === journeyRequestId && readId === slotReadId;
  const loaded = await refreshSaveSlots();
  if (loaded === undefined || !isCurrent()) return;
  if (!loaded) {
    deathFeedback.textContent = "无法读取存档，尚不能确认恢复方式。请返回主界面重试读取。";
    updateDeathControls();
    return;
  }
  const living = saveSlots.filter(canContinueSaveSlot).sort((a, b) => b.updatedAtMs - a.updatedAtMs || a.slotId.localeCompare(b.slotId));
  renderSlotOptions(deathSlot, living[0]?.slotId);
  if (living.length > 0) {
    deathRecoveryTarget = "slot";
    deathSavePicker.hidden = false;
    deathRetryButton.hidden = false;
    deathRetryButton.textContent = "从所选存档重试";
    deathFeedback.textContent = `已选最近的存活存档。${slotStatusText(living[0]!)}`;
  } else {
    deathRecoveryTarget = latestSaveAvailable ? "latest" : "new";
    deathRetryButton.hidden = !latestSaveAvailable;
    deathRetryButton.textContent = "从最近存档重试";
    deathNewButton.hidden = latestSaveAvailable;
    deathFeedback.textContent = latestSaveAvailable ? "找到可恢复的存活自动存档，可以从最近存档重试。"
      : "没有可恢复的存活存档。可以开始新旅程，或返回主界面。";
  }
  updateDeathControls();
}

function snapshotIdentity(snapshot: WorldSnapshotV3): string {
  return `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
}

function showError(error: unknown): void {
  resetDeathControls();
  const message = error instanceof Error ? error.message : String(error);
  if (!document.hidden) audioCuePlayer.resume();
  audioCuePlayer.stop();
  audioCuePlayer.clearScene();
  audioCuePlayer.playUiError();
  journey.hidden = true;
  hub.hidden = false;
  shell.dataset.view = "hub";
  shell.dataset.world = "return_station";
  applyWorldTheme("return_station");
  feedback.textContent = nativeJourneyUncertain
    ? "世界操作状态未确认，需退出并重启此程序。"
    : `链路响应失败：${message}`;
  coreUi.close();
  coreUi.apply(null);
  hud.reset();
  enhancementStatusHud.reset();
  interactionBusy = false;
  hudIdentity = null;
  busy = false;
  newButton.disabled = nativeJourneyUncertain || !isTauri();
  continueButton.disabled = nativeJourneyUncertain || !isTauri() || !slotsReady ||
    !(latestSaveAvailable || saveSlots.some(canContinueSaveSlot));
  continueSelected.disabled = true;
  // A failed load may mean the on-disk save changed after the last lookup.
  if (!sessionLoop && !nativeJourneyUncertain && isTauri()) void refreshSaveSlots();
}

function assertJourneyRequest(signal: AbortSignal, requestId: number): void {
  if (signal.aborted || requestId !== journeyRequestId) throw new DOMException("Journey entry was cancelled", "AbortError");
}

async function enterJourney(snapshot: WorldSnapshotV3, signal: AbortSignal, requestId: number, entryNarrative: string | null = null): Promise<void> {
  resetDeathControls();
  continuePicker.hidden = true;
  audioCuePlayer.setEpoch(snapshot.worldEpoch);
  audioCuePlayer.setScene(snapshot.worldId, snapshot.sceneId);
  audioCuePlayer.resume();
  const signalLine = new MistHarborSignalLineState(snapshot);
  hub.hidden = true;
  journey.hidden = false;
  shell.dataset.view = "journey";
  shell.dataset.world = snapshot.worldId;
  applyWorldTheme(snapshot.worldId);
  worldTitle.textContent = worldDisplayName(snapshot.worldId);
  worldDetail.textContent = `世界状态已连接 · 第 ${snapshot.serverTick} 帧`;
  hudIdentity = snapshotIdentity(snapshot);
  hud.apply(snapshot);
  if (entryNarrative) hud.setFeedback(entryNarrative);
  coreUi.apply(snapshot);
  enhancementStatusHud.reset();
  enhancementStatusHud.apply(snapshot);
  hud.setPauseStatus(snapshot.entryToken ? "loading" : "running");
  if (!renderer) {
    const registry = await runtimeAssetLoader.load({ signal });
    assertJourneyRequest(signal, requestId);
    const portraitError = hud.setPortraitRegistry(registry);
    if (portraitError) hud.setFeedback(portraitError);
    const corePortraitError = coreUi.setPortraitRegistry(registry);
    if (corePortraitError) hud.setFeedback(corePortraitError);
    renderer = new WorldRenderer(registry);
    if (!canvas.isConnected) {
      const replacement = canvas.cloneNode(false) as HTMLCanvasElement;
      canvasHost.append(replacement);
      canvas = replacement;
    }
    await renderer.init(canvas);
    canvas.addEventListener("webglcontextlost", () => audioCuePlayer.stop(), { once: true });
    assertJourneyRequest(signal, requestId);
  }
  sceneSession = new SceneDefinitionSession(renderer, sceneDefinitionLoader);
  await sceneSession.prepare(snapshot, signal);
  assertJourneyRequest(signal, requestId);
  sessionLoop = new SessionLoop(client, sceneSession, {
    pointerTarget: canvas,
    onSoundCues: (events, latest) => {
      if (latest.protocolVersion === 3) renderer?.acceptSoundCues(events, latest);
    },
    onEvents: events => { audioCuePlayer.handlePresentationEvents(events); },
    onEventsWithSnapshot: (events, latest) => {
      audioCuePlayer.setListenerSnapshot(latest);
      // Held scene-entry/pause refreshes admit enemy warnings, never replay old contacts.
      const combatIds = sessionLoop?.pausePresentationState === "running"
        ? renderer?.acceptCombatPresentationEvents(events, latest) ?? [] : [];
      const rejection = combatRejectionText(events, combatIds);
      if (rejection && sessionLoop?.pausePresentationState === "running") hud.setFeedback(rejection);
      const acceptedCombatIds = new Set(combatIds);
      const acceptedWarningIds = new Set([
        ...(renderer?.acceptSentinelPresentationEvents(events, latest) ?? []),
        ...(renderer?.acceptPressureWavePresentationEvents(events, latest) ?? []),
        ...(renderer?.acceptClockworksEnemyPresentationEvents(events, latest) ?? []),
      ]);
      return events.filter(event => (event.protocolVersion !== 2 || !event.combatFeedback || acceptedCombatIds.has(event.eventId)) && (
        (event.kind !== "SentinelAttackWindup" && event.kind !== "SentinelHeavyWindup" && event.kind !== "SentinelChargeWindup" &&
          event.kind !== "ForgedGuardPressureWindup" && event.kind !== "ForgedGuardPressureImpact" &&
          event.kind !== "PrimeRegulatorPressureWindup" && event.kind !== "PrimeRegulatorPressureImpact" &&
          !isClockworksEnemyCue(event.kind)) ||
        acceptedWarningIds.has(event.eventId)));
    },
    onClearTransientPresentation: () => { renderer?.clearTransientPresentation?.(); audioCuePlayer.stop(); },
    onSnapshot: latest => {
      audioCuePlayer.setListenerSnapshot(latest);
      renderer?.reconcileSoundCue(latest);
      worldDetail.textContent = `世界状态已连接 · 第 ${latest.serverTick} 帧`;
      if (latest.protocolVersion === 3) {
        audioCuePlayer.setEpoch(latest.worldEpoch);
        audioCuePlayer.setScene(latest.worldId, latest.sceneId);
        shell.dataset.world = latest.worldId;
        worldTitle.textContent = worldDisplayName(latest.worldId);
        applyWorldTheme(latest.worldId);
        const nextHudIdentity = snapshotIdentity(latest);
        if (hudIdentity !== nextHudIdentity) {
          hud.reset();
          enhancementStatusHud.reset();
        }
        hudIdentity = nextHudIdentity;
        hud.apply(latest);
        coreUi.apply(latest);
        enhancementStatusHud.apply(latest);
        const signalNarrative = signalLine.accept(latest,
          !interactionBusy && sessionLoop?.pausePresentationState === "running");
        if (signalNarrative) hud.setFeedback(signalNarrative);
      } else {
        coreUi.apply(null);
        enhancementStatusHud.reset();
      }
      updateEnhancementControls();
      sceneSession?.acceptSnapshot(latest);
    },
    onInteract: latest => interactFromSnapshot(latest),
    onDeath: () => { void showDeathRecovery(ownedLoop); },
    onPauseState: (status, message) => {
      if (sessionLoop !== ownedLoop) return;
      hud.setPauseStatus(status, message);
      if (status === "running") audioCuePlayer.resume();
      else audioCuePlayer.suspend();
      if (status === "error" && !document.hidden) {
        audioCuePlayer.resume();
        audioCuePlayer.playUiError();
      }
      if (status !== "running") {
        renderer?.clearSoundCues();
      }
      if (status === "paused") void refreshSaveSlots();
      updateSaveControls();
      updateEnhancementControls();
      if (status === "error" && message) saveFeedback.textContent = `暂停状态未确认：${message}`;
    },
    onError: error => {
      if (sessionLoop !== ownedLoop) return;
      if (ownedLoop.hasUnsettledStopMutation) nativeJourneyUncertain = true;
      showError(error);
    },
    onCleanup: () => {
      if (sessionLoop !== ownedLoop) return;
      resetDeathControls();
      audioCuePlayer.suspend();
      renderer = null;
      sceneSession = null;
      sessionLoop = null;
      enhancementStatusHud.reset();
      // Runtime errors show the hub before asynchronous cleanup finishes.
      // Refresh only after ownership is released, and never for an obsolete exit.
      if (!busy && !nativeJourneyUncertain && !signal.aborted && requestId === journeyRequestId && shell.dataset.view === "hub") {
        void refreshSaveSlots();
      }
    },
  });
  const ownedLoop = sessionLoop;
  await sessionLoop.start(snapshot);
  assertJourneyRequest(signal, requestId);
  busy = false;
  updateDeathControls();
}

async function interactFromSnapshot(snapshot: import("./protocol/types.js").WorldSnapshotEnvelope): Promise<void> {
  if (interactionBusy || snapshot.protocolVersion !== 3 || !sessionLoop || sessionLoop.isDead) return;
  const activeLoop = sessionLoop;
  const sourceSequence = activeLoop.sceneEntrySequence;
  let feedbackSequence = sourceSequence;
  let feedbackSnapshot = snapshot;
  const state = hud.apply(snapshot);
  const interactable = state.interactionId
    ? snapshot.interactables.find(item => item.entityId === state.interactionId)
    : undefined;
  if (!interactable) {
    hud.setFeedback(state.nearbyDoorText === "门锁闭" ? "门锁闭" : state.interactionText);
    return;
  }
  interactionBusy = true;
  hud.setFeedback(interactable.kind.startsWith("scene_") ? "正在请求现场操作…" : "正在交互…");
  try {
    const result = await dispatchInteractable(client, interactable, snapshot.worldEpoch,
      { worldId: snapshot.worldId, sceneId: snapshot.sceneId });
    const current = activeLoop.snapshots.view();
    const currentV3 = current?.protocolVersion === 3 ? current : null;
    if (!isCurrentSceneInteractionResult(sessionLoop === activeLoop && activeLoop.acceptsExternalResults,
      currentV3, snapshot, result)) return;
    feedbackSnapshot = result.snapshot;
    feedbackSequence = sourceSequence + (result.snapshot.entryToken ? 1 : 0);
    await activeLoop.acceptAuthoritativeSnapshot(result.snapshot);
    if (sessionLoop !== activeLoop || !activeLoop.acceptsExternalResults) return;
    const acceptedCurrent = activeLoop.snapshots.view();
    if (activeLoop.sceneEntrySequence !== feedbackSequence || activeLoop.pausePresentationState === "loading" ||
        !isCurrentSceneInteractionFeedback(acceptedCurrent?.protocolVersion === 3 ? acceptedCurrent : null,
          feedbackSnapshot)) return;
    const applied = result.applied || ("alreadyApplied" in result && result.alreadyApplied);
    const narrative = confirmedGreyHiveNarrative(snapshot.worldId, snapshot.sceneId,
      interactable.entityId, interactable.kind, result.applied === true) ??
      confirmedMistHarborBeaconSync(snapshot, interactable.entityId, interactable.kind, result) ??
      confirmedMistHarborAcousticMappingLine(snapshot, interactable.entityId, interactable.kind, result) ??
      confirmedClockworksEpilogue(snapshot, interactable.entityId, interactable.kind, result);
    const returnNarrative = confirmedReturnStationAfterGreyHive(snapshot,
      interactable.entityId, interactable.kind, result) ??
      confirmedReturnStationAfterClockworks(snapshot, interactable.entityId, interactable.kind, result);
    if (!applied) hud.setFeedback(interactionErrorText(result.errorCode || "E_INTERACTION_NOT_APPLIED"));
    else if (narrative) hud.setFeedback(narrative);
    else if (returnNarrative) hud.setFeedback(returnNarrative);
    else if (interactable.kind === "beacon_mount") hud.setFeedback("信标已部署并挂载，可以前往撤离。");
    else if (interactable.kind === "beacon_collect") hud.setFeedback("便携信标已收取，可以前往撤离。");
    else if (interactable.kind === "scene_transition") hud.setFeedback("已进入新现场。");
    else if (interactable.kind === "scene_checkpoint") hud.setFeedback("检查点已更新。");
    else if (interactable.kind === "scene_trigger") hud.setFeedback("现场事件已触发。");
    else if (interactable.kind === "world_gate") hud.setFeedback("归航链路已切换，世界状态已更新。");
    else if (interactable.kind === "mission_terminal") hud.setFeedback(missionTerminalSummary(result.snapshot));
    else if (interactable.kind === "capability_terminal") hud.setFeedback(capabilityTerminalSummary(result.snapshot));
    else if (interactable.kind === "save_rest_terminal") hud.setFeedback("休整完成，已恢复状态并保存。");
    else if (interactable.kind === "pump_control") hud.setFeedback(result.applied === true
      ? "排水泵已启动。" : result.snapshot.mistHarborPump?.state === "drained"
        ? "排水系统已完成。" : "排水进行中。");
    else if (interactable.kind === "coolant_valve" && result.applied === true) hud.setFeedback("炉区已冷却 6 秒。");
    else hud.setFeedback("交互已完成");
  } catch (error) {
    const current = activeLoop.snapshots.view();
    if (sessionLoop === activeLoop && activeLoop.acceptsExternalResults &&
        activeLoop.sceneEntrySequence === feedbackSequence && activeLoop.pausePresentationState !== "loading" &&
        isCurrentSceneInteractionFeedback(current?.protocolVersion === 3 ? current : null, feedbackSnapshot)) {
      hud.setFeedback(interactionErrorText(error instanceof Error ? error.message : String(error)));
    }
  } finally {
    if (sessionLoop === activeLoop) interactionBusy = false;
  }
}

async function begin(kind: "new", recoveryLoop?: SessionLoop): Promise<void> {
  if (busy || buildCloseBarrier.closing || nativeJourneyUncertain || !isTauri()) return;
  if (sessionLoop && sessionLoop !== recoveryLoop) return;
  if (recoveryLoop && (sessionLoop !== recoveryLoop || !recoveryLoop.isDead)) return;
  journeyRequestId++;
  const requestId = journeyRequestId;
  const controller = new AbortController();
  activeJourneyLoad = controller;
  busy = true;
  newButton.disabled = true;
  continueButton.disabled = true;
  feedback.textContent = "正在建立世界链路…";
  updateDeathControls();
  try {
    if (recoveryLoop) await recoveryLoop.stop("replace");
    assertJourneyRequest(controller.signal, requestId);
    await waitForHubBuild();
    assertJourneyRequest(controller.signal, requestId);
    const snapshot = await client.newJourney();
    assertJourneyRequest(controller.signal, requestId);
    await enterJourney(snapshot, controller.signal, requestId, confirmedReturnStationNewJourney(snapshot));
  } catch (error) {
    if (error instanceof Error && (error.message === "E_NEW_JOURNEY_TIMEOUT" ||
        error.message === "E_CONTINUE_SLOT_TIMEOUT" || error.message === "E_SESSION_STOP_UNCERTAIN_MUTATION")) nativeJourneyUncertain = true;
    if (controller.signal.aborted || requestId !== journeyRequestId) return;
    if (sessionLoop) await sessionLoop.stop("error").catch(() => undefined);
    else if (sceneSession) {
      await sceneSession.destroy().catch(() => undefined);
      sceneSession = null;
      renderer = null;
    }
    else if (renderer) {
      await renderer.destroy().catch(() => undefined);
      renderer = null;
    }
    if (!controller.signal.aborted && requestId === journeyRequestId) showError(error);
  } finally {
    if (activeJourneyLoad === controller) activeJourneyLoad = null;
    if (requestId === journeyRequestId && controller.signal.aborted) busy = false;
  }
}

newButton.addEventListener("click", () => void begin("new"));
mainSettingsButton.addEventListener("click", () => coreUi.open(mainSettingsButton, "settings"));
exitButton.addEventListener("click", () => {
  if (buildCloseBarrier.closing || busy) return;
  if (!isTauri()) {
    feedback.textContent = "预览模式不会关闭窗口；请在桌面版中退出。";
    return;
  }
  coreUi.close();
  exitButton.disabled = true;
  feedback.textContent = "正在安全关闭窗口…";
  void buildCloseBarrier.close(async () => {
    await waitForHubBuild();
    if (sessionLoop?.acceptsExternalResults) await sessionLoop.runWithSession(async () => undefined);
  }, () => getCurrentWindow().close()).catch(error => {
    exitButton.disabled = false;
    feedback.textContent = `退出失败：${error instanceof Error ? error.message : String(error)}`;
    audioCuePlayer.playUiError();
  });
});
coreOpenButton.addEventListener("click", () => {
  if (busy || buildCloseBarrier.closing || coreUi.isOpen || sessionLoop?.isDead || coreUi.currentSnapshot?.player.currentHp === 0) return;
  const loop = sessionLoop;
  if (!loop) {
    coreUi.open(coreOpenButton);
    return;
  }
  void (async () => {
    try {
      if (loop.pausePresentationState !== "paused") await loop.pause("manual");
      if (sessionLoop !== loop || loop.isDead || loop.pausePresentationState !== "paused") {
        hud.setFeedback("终端需等待旅程暂停确认。");
        return;
      }
      coreUi.open(coreOpenButton);
    } catch (error) {
      if (sessionLoop === loop) hud.setFeedback(`终端暂不可用：${error instanceof Error ? error.message : String(error)}`);
    }
  })();
});
continueButton.addEventListener("click", () => {
  if (nativeJourneyUncertain || busy || buildCloseBarrier.closing || !slotsReady) return;
  continuePicker.hidden = !continuePicker.hidden;
  if (!continuePicker.hidden) void refreshSaveSlots();
});
continueSlot.addEventListener("change", () => {
  const slot = selectedSlot(continueSlot);
  if (!slotsReady || latestSaveAvailable) return;
  continueNote.textContent = slot ? slotStatusText(slot) : "请选择一个有效存档。";
  continueSelected.disabled = nativeJourneyUncertain || !canContinueSaveSlot(slot) || busy;
});
async function continueJourney(slot: SaveSlotSummary | undefined, recoveryLoop?: SessionLoop, latestSave = false): Promise<void> {
  if (busy || buildCloseBarrier.closing || nativeJourneyUncertain || !isTauri()) return;
  if (!slotsReady) return;
  if (latestSave ? !latestSaveAvailable || (recoveryLoop && deathRecoveryTarget !== "latest") : !canContinueSaveSlot(slot)) return;
  if (sessionLoop && sessionLoop !== recoveryLoop) return;
  if (recoveryLoop && (sessionLoop !== recoveryLoop || !recoveryLoop.isDead)) return;
  if (slot?.readOnly && !window.confirm(`「${slot.displayName}」是旧版只读存档。Rust 将保留原档并备份迁移后继续。现在继续？`)) return;
  journeyRequestId++;
  const requestId = journeyRequestId;
  const controller = new AbortController();
  activeJourneyLoad = controller;
  busy = true;
  continueSelected.disabled = true;
  newButton.disabled = true;
  continueButton.disabled = true;
  feedback.textContent = latestSave ? "正在从最近存档恢复…" : `正在从「${slot!.displayName}」恢复…`;
  deathFeedback.textContent = feedback.textContent;
  updateDeathControls();
  await (async () => {
    try {
      if (recoveryLoop) await recoveryLoop.stop("replace");
      assertJourneyRequest(controller.signal, requestId);
      await waitForHubBuild();
      assertJourneyRequest(controller.signal, requestId);
      const receipt = latestSave ? { snapshot: await client.continueJourney() }
        : await client.continueSlot(slot!.slotId);
      assertJourneyRequest(controller.signal, requestId);
      await enterJourney(receipt.snapshot, controller.signal, requestId);
    } catch (error) {
      if (error instanceof Error && (error.message === "E_CONTINUE_SLOT_TIMEOUT" ||
          error.message === "E_CONTINUE_TIMEOUT" || error.message === "E_SESSION_STOP_UNCERTAIN_MUTATION")) nativeJourneyUncertain = true;
      if (controller.signal.aborted || requestId !== journeyRequestId) return;
      if (sessionLoop) await sessionLoop.stop("error").catch(() => undefined);
      else if (sceneSession) {
        await sceneSession.destroy().catch(() => undefined);
        sceneSession = null;
        renderer = null;
      } else if (renderer) {
        await renderer.destroy().catch(() => undefined);
        renderer = null;
      }
      if (!controller.signal.aborted && requestId === journeyRequestId) showError(error);
    } finally {
      if (activeJourneyLoad === controller) activeJourneyLoad = null;
      if (requestId === journeyRequestId && controller.signal.aborted) busy = false;
    }
  })();
}
continueSelected.addEventListener("click", () => void continueJourney(selectedSlot(continueSlot), undefined, latestSaveAvailable));
deathSlot.addEventListener("change", () => {
  const slot = selectedSlot(deathSlot);
  deathFeedback.textContent = slot ? slotStatusText(slot) : "请选择仍然存活的存档。";
  updateDeathControls();
});
deathRetryButton.addEventListener("click", () => {
  const loop = sessionLoop;
  if (!loop?.isDead || !slotsReady || deathRetryButton.hidden) return;
  void continueJourney(selectedSlot(deathSlot), loop, deathRecoveryTarget === "latest");
});
deathNewButton.addEventListener("click", () => {
  const loop = sessionLoop;
  if (!loop?.isDead || deathNewButton.hidden || deathRecoveryTarget !== "new") return;
  void begin("new", loop);
});
pauseSlot.addEventListener("change", updateSaveControls);
newSlotName.addEventListener("input", updateSaveControls);

async function saveWhilePaused(create: boolean): Promise<void> {
  const loop = sessionLoop;
  if (!loop || loop.isDead || loop.pausePresentationState !== "paused" || saveBusy) return;
  const existing = selectedSlot(pauseSlot);
  if (!create && !canOverwriteSaveSlot(existing)) return;
  const displayName = newSlotName.value.trim();
  if (create && !displayName) { saveFeedback.textContent = "请先填写存档名称。"; return; }
  if (!create && existing && !window.confirm(`确定覆盖「${existing.displayName}」？此操作会替换该槽中的进度。`)) return;
  const slotId = create ? `slot-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}` : existing!.slotId;
  saveBusy = true;
  saveFeedback.textContent = create ? "正在创建新存档…" : "正在覆盖所选存档…";
  updateSaveControls();
  try {
    const receipt = await loop.runWhilePaused(async () => {
      const result = await client.saveSlot(slotId, create ? displayName : existing!.displayName, create);
      await loop.acceptAuthoritativeSnapshot(result.snapshot);
      return result;
    });
    if (sessionLoop !== loop || !loop.acceptsExternalResults) return;
    saveFeedback.textContent = `已保存「${create ? displayName : existing!.displayName}」· 第 ${receipt.serverTick} 帧。`;
    if (create) newSlotName.value = "现场记录";
    await refreshSaveSlots();
    if (sessionLoop !== loop || !loop.acceptsExternalResults) return;
    pauseSlot.value = slotId;
  } catch (error) {
    if (sessionLoop === loop && loop.acceptsExternalResults) {
      saveFeedback.textContent = `保存失败：${error instanceof Error ? error.message : String(error)}`;
      audioCuePlayer.playUiError();
    }
  } finally {
    saveBusy = false;
    updateSaveControls();
  }
}
saveNewSlotButton.addEventListener("click", () => void saveWhilePaused(true));
overwriteSlotButton.addEventListener("click", () => void saveWhilePaused(false));
function returnToMain(): void {
  if (busy || buildCloseBarrier.closing) return;
  const wasDead = sessionLoop?.isDead === true;
  journeyRequestId++;
  const requestId = journeyRequestId;
  slotReadId++;
  busy = true;
  updateDeathControls();
  newButton.disabled = true;
  continueButton.disabled = true;
  void (async () => {
    await waitForHubBuild();
    if (requestId !== journeyRequestId) return;
    enhancementStatusHud.reset();
    audioCuePlayer.suspend();
    await sessionLoop?.stop("hub");
    if (requestId !== journeyRequestId) return;
    const snapshot = await client.returnToHub();
    if (requestId !== journeyRequestId) return;
    audioCuePlayer.setEpoch(snapshot.worldEpoch);
    audioCuePlayer.setScene(snapshot.worldId, snapshot.sceneId);
    audioCuePlayer.resume();
    hud.reset();
    enhancementStatusHud.reset();
    coreUi.apply(wasDead ? null : snapshot);
    coreUi.close();
    hudIdentity = null;
    journey.hidden = true;
    hub.hidden = false;
    shell.dataset.view = "hub";
    shell.dataset.world = snapshot.worldId;
    applyWorldTheme(snapshot.worldId);
    feedback.textContent = wasDead ? "已返回主界面。死亡现场未保存；请选择存活存档或开始新旅程。" : `当前世界：${worldDisplayName(snapshot.worldId)}`;
    newButton.disabled = nativeJourneyUncertain;
    await refreshSaveSlots();
    if (requestId !== journeyRequestId) return;
    busy = false;
  })().catch(error => {
    if (error instanceof Error && error.message === "E_SESSION_STOP_TIMEOUT") nativeJourneyUncertain = true;
    if (requestId === journeyRequestId) showError(error);
  });
}
root.querySelector<HTMLButtonElement>("#back-to-hub")!.addEventListener("click", returnToMain);
deathHubButton.addEventListener("click", returnToMain);

function togglePause(): void {
  if (!sessionLoop || sessionLoop.isDead) return;
  if (sessionLoop.pausePresentationState === "error") void sessionLoop.retryPauseState();
  else if (sessionLoop.isPaused) void sessionLoop.resume();
  else void sessionLoop.pause();
}
root.querySelector<HTMLButtonElement>("#hud-pause")!.addEventListener("click", togglePause);
root.querySelector<HTMLButtonElement>("#hud-resume")!.addEventListener("click", () => {
  if (sessionLoop?.isDead) return;
  if (sessionLoop?.pausePresentationState === "error") void sessionLoop.retryPauseState();
  else if (sessionLoop) void sessionLoop.resume();
});

window.addEventListener("pagehide", () => {
  journeyRequestId++;
  activeJourneyLoad?.abort();
  resetDeathControls();
  hud.reset();
  enhancementStatusHud.reset();
  hudIdentity = null;
  void audioCuePlayer.dispose();
  if (sessionLoop) void sessionLoop.stop("unload");
  else sceneSession?.invalidate();
}, { once: true });

async function boot(): Promise<void> {
  if (!isTauri()) {
    connectionLabel.textContent = "请在桌面版连接";
    feedback.textContent = "归航链路仅在桌面版中可用。";
    return;
  }
  try {
    const snapshot = await client.snapshot();
    audioCuePlayer.setEpoch(snapshot.worldEpoch);
    audioCuePlayer.setScene(snapshot.worldId, snapshot.sceneId);
    shell.dataset.world = snapshot.worldId;
    applyWorldTheme(snapshot.worldId);
    coreUi.apply(snapshot.player.currentHp === 0 ? null : snapshot);
    connectionLight.classList.add("connected");
    connectionLabel.textContent = "归航链路已连接";
    feedback.textContent = `当前世界：${worldDisplayName(snapshot.worldId)}`;
    newButton.disabled = nativeJourneyUncertain;
    await refreshSaveSlots();
  } catch (error) {
    showError(error);
    connectionLabel.textContent = "归航链路中断";
  }
}

void boot();
