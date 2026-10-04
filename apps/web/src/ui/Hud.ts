import { BAIZHI_INTERACTION_ID, baizhiInteractionPrompt } from "../game/BaizhiDialogue.js";
import { CLOCKWORKS_FURNACE_DIALOGUE_ID, CLOCKWORKS_FURNACE_DIALOGUE_PROMPT, isClockworksFurnaceDialogueAvailable, isClockworksFurnaceDialogueTarget } from "../game/ClockworksFurnaceDialogue.js";
import { scannerTerminalLabel } from "../game/ScannerReward.js";
import { sentinelStatus } from "../protocol/SentinelEncounter.js";
import { greyHiveBeaconStatus } from "../game/GreyHiveBeacon.js";
import { projectWardenEncounter } from "../renderer/WardenEncounterModel.js";
import { projectEnvironmentHazards } from "../renderer/EnvironmentHazardModel.js";
import type { ObjectiveView, WorldSnapshotV3 } from "../protocol/types.js";
import type { PauseStatus } from "../game/SessionLoop.js";
import { HudPortrait } from "./components/HudPortrait.js";
import { SkillBar } from "./components/SkillBar.js";
import { ObjectivePanel } from "./components/ObjectivePanel.js";
import { InteractionPrompt } from "./components/InteractionPrompt.js";
import type { AssetRegistry } from "../assets/AssetRegistry.js";
import { deriveMistHarborPumpStatus } from "./components/MistHarborPumpStatus.js";

export const INTERACTION_RANGE_M = 2.5;

const AUTHORITATIVE_INTERACTION_KINDS = new Set([
  "environment_control", "beacon_collect", "beacon_mount",
  "power_console", "door_panel", "terminal", "lockdown_terminal",
  "facility_log", "bio_log_terminal", "gate_b_panel", "extraction_console",
  "beacon", "pump_control", "valve_control",
  "scene_transition", "scene_checkpoint", "scene_trigger", "world_gate",
  "mission_terminal", "capability_terminal", "save_rest_terminal",
]);

export interface HudObjective {
  id: string;
  state: string;
}

export interface HudState {
  hpText: string;
  hpRatio: number;
  energyText: string;
  energyRatio: number;
  worldLabel: string;
  sceneId: string;
  objectives: HudObjective[];
  actionState: string;
  interactionId: string | null;
  interactionText: string;
  nearbyDoorText: string;
  environmentText: string;
  boss: ReturnType<typeof projectWardenEncounter>;
  pumpStatus: ReturnType<typeof deriveMistHarborPumpStatus>;
}

function boundedMeter(current: number, maximum: number): { text: string; ratio: number } {
  if (!Number.isFinite(current) || !Number.isFinite(maximum) || maximum <= 0) {
    return { text: "— / —", ratio: 0 };
  }
  const safeMaximum = Math.max(0, maximum);
  const safeCurrent = Math.min(safeMaximum, Math.max(0, current));
  return {
    text: `${Math.round(safeCurrent)} / ${Math.round(safeMaximum)}`,
    ratio: safeMaximum > 0 ? safeCurrent / safeMaximum : 0,
  };
}

function distance3d(a: WorldSnapshotV3["player"]["transform"]["positionM"], b: WorldSnapshotV3["player"]["transform"]["positionM"]): number {
  return Math.hypot(a.xM - b.xM, a.yM - b.yM, a.zM - b.zM);
}

function worldLabel(worldId: string): string {
  switch (worldId) {
    case "grey_hive": return "灰巢设施";
    case "mist_harbor": return "雾港余烬";
    case "clockworks": return "钟骨工厂";
    case "return_station": return "归航站";
    default: return worldId || "未知世界";
  }
}

function interactionLabel(kind: string, id: string): string {
  switch (kind) {
    case "power_console": return "电力控制台";
    case "door_panel": return "门禁面板";
    case "terminal": return "终端";
    case "extraction_console": return "撤离控制台";
    case "beacon": return "雾港航标";
    case "beacon_collect": return "收取便携信标";
    case "beacon_mount": return "部署并挂载信标";
    case "pump_control": return "启动排水泵";
    case "environment_control": return "环境调节";
    case "valve_control": return "压力阀";
    case "scene_transition": return id === "gh_exit_to_beacon" ? "返回信标室" : "前往新现场";
    case "scene_checkpoint": return "现场检查点";
    case "scene_trigger": return "现场事件";
    case "world_gate":
      if (id === "gh_extraction_return_to_rs" || id === "mh_extraction_return_to_rs") return "撤离后返回归航站";
      if (id === "rs_mh_world_gate_marker") return "前往雾港余烬";
      return "前往灰巢设施";
    case "mission_terminal": return "任务状态";
    case "capability_terminal": return "能力状态";
    case "save_rest_terminal": return "休整并保存";
    default: return kind || id;
  }
}

function objectiveViews(objectives: ObjectiveView[]): HudObjective[] {
  return objectives.slice(0, 3).map(objective => ({ id: objective.objectiveId, state: objective.state }));
}

/** Projects only authoritative v3 fields; no scene text or inferred progression is used. */
export function deriveHudState(snapshot: WorldSnapshotV3): HudState {
  const hp = boundedMeter(snapshot.player.currentHp, snapshot.player.maxHp);
  const energy = boundedMeter(snapshot.player.currentEnergy, snapshot.player.maxEnergy);
  const pumpStatus = deriveMistHarborPumpStatus(snapshot);
  const playerPosition = snapshot.player.transform.positionM;
  const nearestInteraction = snapshot.interactables
    .filter(item => snapshot.player.currentHp > 0 && item.active &&
      (item.kind === "static_dialogue_marker" || item.entityId === CLOCKWORKS_FURNACE_DIALOGUE_ID
        ? isClockworksFurnaceDialogueAvailable(snapshot, item)
        : AUTHORITATIVE_INTERACTION_KINDS.has(item.kind)) &&
      (item.kind !== "pump_control" || (snapshot.worldId === "mist_harbor" &&
        snapshot.sceneId === "mh_pump_station" && item.entityId === "mh_pump_control_primary" &&
        (snapshot.mistHarborPump === undefined || pumpStatus?.state === "ready"))) &&
      distance3d(playerPosition, item.transform.positionM) <= INTERACTION_RANGE_M)
    .map(item => ({ item, distance: distance3d(playerPosition, item.transform.positionM) }))
    .sort((left, right) => left.distance - right.distance || left.item.entityId.localeCompare(right.item.entityId))[0]?.item;
  const nearestDoor = snapshot.doors
    .map(door => ({ door, distance: distance3d(playerPosition, door.transform.positionM) }))
    .filter(entry => entry.distance <= INTERACTION_RANGE_M)
    .sort((left, right) => left.distance - right.distance || left.door.doorId.localeCompare(right.door.doorId))[0]?.door;
  const baizhiPrompt = baizhiInteractionPrompt(snapshot);
  const interactionText = baizhiPrompt ?? (nearestInteraction
    ? isClockworksFurnaceDialogueTarget(snapshot, nearestInteraction) ? CLOCKWORKS_FURNACE_DIALOGUE_PROMPT
      : nearestInteraction.kind === "pump_control" ? "[F] 启动排水泵"
      : snapshot.worldId === "grey_hive" && snapshot.sceneId === "gh_entry_maintenance" &&
        nearestInteraction.kind === "terminal" && nearestInteraction.entityId === "gh_entry_tutorial_terminal"
        ? "[F] 查看教程终端"
      : `F 交互 · ${(scannerTerminalLabel(snapshot, nearestInteraction) ?? interactionLabel(nearestInteraction.kind, nearestInteraction.entityId))}`
    : "");
  const nearbyDoorText = nearestDoor
    ? nearestDoor.locked ? "门锁闭" : nearestDoor.open ? "门已开启" : "门关闭"
    : "";

  return {
    hpText: hp.text,
    hpRatio: hp.ratio,
    energyText: energy.text,
    energyRatio: energy.ratio,
    worldLabel: worldLabel(snapshot.worldId),
    sceneId: snapshot.sceneId || "未知场景",
    objectives: objectiveViews(snapshot.objectives),
    actionState: snapshot.player.currentHp === 0 ? "已倒下" : snapshot.player.actionState || "状态未报告",
    interactionId: baizhiPrompt ? BAIZHI_INTERACTION_ID : nearestInteraction?.entityId ?? null,
    interactionText,
    nearbyDoorText,
    pumpStatus,
    boss: projectWardenEncounter(snapshot),
    environmentText: [greyHiveBeaconStatus(snapshot), sentinelStatus(snapshot), ...projectEnvironmentHazards(snapshot, true).slice(0,3).map(frame =>
      frame.label.replace(/ \d+\.\d秒/, ""))].filter(Boolean).join(" · "),
  };
}

export interface HudElements {
  portrait: HTMLElement;
  actionState: HTMLElement;
  skills: HTMLElement;
  objectivePanel: HTMLElement;
  interactionPrompt: HTMLElement;
  hpValue: HTMLElement;
  hpFill: HTMLElement;
  energyValue: HTMLElement;
  energyFill: HTMLElement;
  world: HTMLElement;
  scene: HTMLElement;
  objectives: HTMLElement;
  interaction: HTMLElement;
  doorStatus: HTMLElement;
  pumpStatus: HTMLElement;
  environmentStatus?: HTMLElement;
  bossStatus?: HTMLElement;
  bossFill?: HTMLElement;
  feedback: HTMLElement;
  pauseButton: HTMLButtonElement;
  pauseOverlay: HTMLElement;
  pauseTitle: HTMLElement;
  pauseDetail: HTMLElement;
  resumeButton: HTMLButtonElement;
  deathOverlay?: HTMLElement;
}

export class HudPresenter {
  private readonly portrait: HudPortrait;
  private readonly skillBar: SkillBar;
  private readonly objectivePanel: ObjectivePanel;
  private readonly interactionPrompt: InteractionPrompt;

  constructor(private readonly elements: HudElements) {
    this.portrait = new HudPortrait(elements.portrait);
    this.skillBar = new SkillBar(elements.skills);
    this.objectivePanel = new ObjectivePanel(elements.objectivePanel);
    this.interactionPrompt = new InteractionPrompt(elements.interactionPrompt);
  }

  setPortraitRegistry(registry: AssetRegistry): string | null {
    return this.portrait.bindRegistry(registry);
  }

  apply(snapshot: WorldSnapshotV3): HudState {
    const state = deriveHudState(snapshot);
    this.elements.hpValue.textContent = state.hpText;
    this.elements.hpFill.style.transform = `scaleX(${state.hpRatio})`;
    this.elements.hpFill.setAttribute("aria-valuenow", String(Math.round(state.hpRatio * 100)));
    this.elements.energyValue.textContent = state.energyText;
    this.elements.energyFill.style.transform = `scaleX(${state.energyRatio})`;
    this.elements.energyFill.setAttribute("aria-valuenow", String(Math.round(state.energyRatio * 100)));
    this.elements.world.textContent = state.worldLabel;
    this.elements.scene.textContent = state.sceneId;
    this.elements.actionState.textContent = state.actionState;
    this.objectivePanel.render(state.objectives);
    this.interactionPrompt.render(state.interactionText);
    this.elements.interaction.textContent = state.interactionText;
    this.elements.doorStatus.textContent = state.nearbyDoorText;
    this.elements.pumpStatus.textContent = state.pumpStatus?.text ?? "";
    this.elements.pumpStatus.hidden = state.pumpStatus === null;
    this.elements.pumpStatus.setAttribute("data-pump-state", state.pumpStatus?.state ?? "");
    if (this.elements.environmentStatus) {
      if (this.elements.environmentStatus.textContent !== state.environmentText) this.elements.environmentStatus.textContent = state.environmentText;
      this.elements.environmentStatus.hidden = state.environmentText === "";
    }
    if(this.elements.bossStatus){this.elements.bossStatus.textContent=state.boss?.text??"";this.elements.bossStatus.hidden=!state.boss;}
    if(this.elements.bossFill){this.elements.bossFill.style.transform=`scaleX(${state.boss?.hpRatio??0})`;this.elements.bossFill.parentElement!.hidden=!state.boss;
      this.elements.bossFill.setAttribute("aria-valuenow",String(Math.round((state.boss?.hpRatio??0)*100)));}
    return state;
  }

  setPauseStatus(status: PauseStatus, message = ""): void {
    const dead = status === "dead";
    const visible = status !== "running" && !dead;
    if (this.elements.deathOverlay) this.elements.deathOverlay.hidden = !dead;
    this.elements.pauseOverlay.setAttribute("data-loading", String(status === "loading"));
    this.elements.pauseOverlay.setAttribute("aria-busy", String(status === "loading"));
    this.elements.pauseOverlay.hidden = !visible;
    this.elements.pauseTitle.textContent = status === "loading" ? "正在载入现场…" : status === "paused" ? "旅程已暂停"
      : status === "pausing" ? "正在请求暂停…"
        : status === "resuming" ? "正在等待恢复确认…"
          : status === "error" ? "暂停状态未确认" : "";
    this.elements.pauseDetail.textContent = status === "loading" ? "正在准备场景与首帧画面，世界保持暂停。" : status === "error"
      ? `本地输入已锁定 · ${message || "请重试确认"}`
      : status === "pausing" || status === "resuming" ? "收到 Rust 权威回执后才会切换游戏状态。" : "";
    const buttonText = dead ? "已倒下" : status === "loading" ? "载入中" : status === "running" ? "暂停"
      : status === "pausing" ? "恢复"
        : status === "paused" ? "继续"
          : status === "resuming" ? "暂停" : "重试确认";
    this.elements.pauseButton.textContent = buttonText;
    this.elements.pauseButton.disabled = status === "loading" || dead;
    this.elements.resumeButton.disabled = status === "loading" || dead;
    this.elements.resumeButton.hidden = dead;
    this.elements.resumeButton.textContent = status === "error" ? "重试确认" : "继续";
    this.elements.pauseButton.setAttribute("aria-pressed", String(status !== "running"));
  }

  setPaused(paused: boolean): void {
    this.setPauseStatus(paused ? "paused" : "running");
  }

  setFeedback(message: string): void { this.elements.feedback.textContent = message; }

  reset(): void {
    if(this.elements.bossStatus){this.elements.bossStatus.textContent="";this.elements.bossStatus.hidden=true;}
    if(this.elements.bossFill){this.elements.bossFill.style.transform="scaleX(0)";if(this.elements.bossFill.parentElement)this.elements.bossFill.parentElement.hidden=true;}
    this.elements.hpValue.textContent = "— / —";
    this.elements.hpFill.style.transform = "scaleX(0)";
    this.elements.hpFill.setAttribute("aria-valuenow", "0");
    this.elements.energyValue.textContent = "— / —";
    this.elements.energyFill.style.transform = "scaleX(0)";
    this.elements.energyFill.setAttribute("aria-valuenow", "0");
    this.elements.world.textContent = "—";
    this.elements.scene.textContent = "—";
    this.elements.actionState.textContent = "状态未报告";
    this.objectivePanel.render([]);
    this.interactionPrompt.render("");
    this.elements.interaction.textContent = "";
    this.elements.doorStatus.textContent = "";
    this.elements.pumpStatus.textContent = "";
    this.elements.pumpStatus.hidden = true;
    this.elements.pumpStatus.setAttribute("data-pump-state", "");
    if (this.elements.environmentStatus) { this.elements.environmentStatus.textContent = ""; this.elements.environmentStatus.hidden = true; }
    this.elements.feedback.textContent = "";
    this.setPauseStatus("running");
  }
}
