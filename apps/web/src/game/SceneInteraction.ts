import type { InteractableView, WorldSnapshotV3 } from "../protocol/types.js";
import type { TauriClient } from "../bridge/tauri-client.js";

export type SceneInteractionResult = Awaited<ReturnType<TauriClient["interact"]>>;
type SceneCommandResult = Awaited<ReturnType<TauriClient["sceneTransition"]>>;
export type DispatchedInteraction = SceneInteractionResult | SceneCommandResult;

export type SceneInteractionClient = Pick<TauriClient,
  "interact" | "sceneTransition" | "sceneCheckpoint" | "sceneTrigger" | "worldGate" |
  "missionTerminalStatus" | "capabilityTerminalStatus" | "saveRestTerminal" | "environmentControl">;

type InteractionSource = Pick<WorldSnapshotV3, "worldId" | "sceneId">;

const ORDINARY_INTERACTION_KINDS = new Set([
  "power_console", "door_panel", "terminal", "lockdown_terminal",
  "facility_log", "bio_log_terminal", "gate_b_panel", "extraction_console",
  "beacon", "pump_control", "valve_control",
]);

export function interactionErrorText(code: string): string {
  if (code === "E_SCANNER_POWER_REQUIRED") return "先恢复灰巢供电，再恢复扫描模块。";
  if (code === "E_BEACON_REQUIRED") return "先收取或部署便携信标；可从西侧返回信标室。";
  if (code === "E_BEACON_NOT_CARRIED") return "请先收取便携信标，再进行挂载。";
  if (code === "E_SCENE_SENTINEL_FIRST_KILL_REQUIRED") return "先击败哨卫，再前往信标室。";
  if (code === "E_WORLD_GATE_GH_EXTRACTION_REQUIRED") return "完成灰巢撤离后才能返回归航站。";
  if (code === "E_WORLD_GATE_MH_EXTRACTION_REQUIRED") return "完成雾港实际撤离后才能进入钟骨工厂。";
  if (code === "E_CLOCKWORKS_SHUTDOWN_PREREQUISITES_MISSING") return "先完成压力阀门与调节者核心目标。";
  if (code === "E_CLOCKWORKS_REGULATOR_DEFEAT_REQUIRED") return "先击败主调节者，再确认停机结算。";
  if (code === "E_CLOCKWORKS_OBJECTIVE_SOURCE_REQUIRED") return "请确认三个压力阀门与调节者核心终端，再停机结算。";
  if (code === "E_MH_EXTRACTION_REQUIRED") return "先完成雾港西侧航标、东侧航标和信号目标，再撤离。";
  if (code === "E_WORLD_GATE_UNAVAILABLE") return "这条归航链路当前不可用。";
  if (code === "E_MISSION_TERMINAL_UNAVAILABLE") return "任务终端当前不可用。";
  if (code === "E_MISSION_TERMINAL_OUT_OF_RANGE") return "请靠近归航站任务终端。";
  if (code === "E_CAPABILITY_TERMINAL_UNAVAILABLE") return "能力终端当前不可用。";
  if (code === "E_CAPABILITY_TERMINAL_OUT_OF_RANGE") return "请靠近归航站能力终端。";
  if (code === "E_REST_TERMINAL_COMBAT_ACTIVE") return "战斗状态结束后才能休整并保存。";
  if (code === "E_REST_TERMINAL_UNAVAILABLE") return "休整终端当前不可用。";
  if (code === "E_REST_TERMINAL_OUT_OF_RANGE") return "请靠近归航站休整终端。";
  if (code === "E_PUMP_EastBeaconIncomplete") return "先激活东侧航标，再启动排水泵。";
  if (code === "E_PUMP_WrongWorld") return "排水泵只能在雾港启动。";
  if (code === "E_SCENE_TRIGGER_UNAVAILABLE") return "此现场事件的正式命令尚未接入，未发送操作。";
  if (/ProgressionLocked|LOCKED|NOT_UNLOCKED/i.test(code)) return "路线尚未解锁，现场状态未改变。";
  if (/StaleEpoch|EPOCH_STALE|STALE_EPOCH|EXPIRED/i.test(code)) return "现场状态已过期，请重新靠近目标后再试。";
  if (code.startsWith("E_INTERACTION_KIND_UNKNOWN:")) return "无法识别此目标类型，已阻止操作。";
  if (/OutOfRange/i.test(code)) return "距离目标太远，请靠近后再试。";
  return `现场操作失败：${code}`;
}

/** Routes only known authoritative kinds; unknown kinds never fall through to generic interaction. */
export function dispatchInteractable(
  client: SceneInteractionClient,
  interactable: InteractableView,
  worldEpoch: number,
  source?: InteractionSource,
): Promise<DispatchedInteraction> {
  if (!interactable.active) return Promise.reject(new Error("E_INTERACTION_INACTIVE"));
  switch (interactable.kind) {
    case "scene_transition": return client.sceneTransition(interactable.entityId, worldEpoch);
    case "scene_checkpoint": return client.sceneCheckpoint(interactable.entityId, worldEpoch);
    case "scene_trigger": return client.sceneTrigger(interactable.entityId, worldEpoch);
    case "world_gate": {
      if (interactable.entityId === "rs_world_gate_marker") {
        if (source?.worldId !== "return_station" || source.sceneId !== "rs_core_room") {
          return Promise.reject(new Error("E_WORLD_GATE_SOURCE_MISMATCH"));
        }
        return client.worldGate("rs_world_gate_to_gh", worldEpoch);
      }
      if (interactable.entityId === "rs_mh_world_gate_marker") {
        if (source?.worldId !== "return_station" || source.sceneId !== "rs_core_room") {
          return Promise.reject(new Error("E_WORLD_GATE_SOURCE_MISMATCH"));
        }
        return client.worldGate("rs_world_gate_to_mh", worldEpoch);
      }
      if (interactable.entityId === "rs_cw_world_gate_marker") {
        if (source?.worldId !== "return_station" || source.sceneId !== "rs_core_room") {
          return Promise.reject(new Error("E_WORLD_GATE_SOURCE_MISMATCH"));
        }
        return client.worldGate("rs_world_gate_to_cw", worldEpoch);
      }
      if (interactable.entityId === "cw_shutdown_return_to_rs") {
        if (source?.worldId !== "clockworks" || source.sceneId !== "cw_shutdown_exit") {
          return Promise.reject(new Error("E_WORLD_GATE_SOURCE_MISMATCH"));
        }
      }
      if (interactable.entityId === "mh_extraction_return_to_rs") {
        if (source?.worldId !== "mist_harbor" || source.sceneId !== "mh_extraction") {
          return Promise.reject(new Error("E_WORLD_GATE_SOURCE_MISMATCH"));
        }
      }
      return client.worldGate(interactable.entityId, worldEpoch);
    }
    case "beacon_collect":
    case "beacon_mount": {
      const expectedId = interactable.kind === "beacon_collect" ? "gh_beacon_deploy_marker" : "gh_beacon_storage_mount_marker";
      if (source?.worldId !== "grey_hive" || source.sceneId !== "gh_beacon" || interactable.entityId !== expectedId) {
        return Promise.reject(new Error("E_INTERACTION_SOURCE_MISMATCH"));
      }
      return client.interact(interactable.entityId, worldEpoch);
    }
    case "coolant_valve": {
      if (interactable.entityId !== "cw_regulator_valve_furnace_link_staged" ||
          source?.worldId !== "clockworks" || source.sceneId !== "cw_regulator_core") {
        return Promise.reject(new Error("E_INTERACTION_SOURCE_MISMATCH"));
      }
      return client.interact(interactable.entityId, worldEpoch);
    }
    case "environment_control": return client.environmentControl(interactable.entityId, worldEpoch);
    case "mission_terminal": return client.missionTerminalStatus(interactable.entityId, worldEpoch);
    case "capability_terminal": return client.capabilityTerminalStatus(interactable.entityId, worldEpoch);
    case "save_rest_terminal": return client.saveRestTerminal(interactable.entityId, worldEpoch);
    default:
      if (ORDINARY_INTERACTION_KINDS.has(interactable.kind)) return client.interact(interactable.entityId, worldEpoch);
      return Promise.reject(new Error(`E_INTERACTION_KIND_UNKNOWN:${interactable.kind || "empty"}`));
  }
}

/** A delayed result is usable only by the same live session and source epoch. */
export function isCurrentSceneInteractionResult(
  sameSession: boolean,
  current: WorldSnapshotV3 | null,
  sourceIdentity: Pick<WorldSnapshotV3, "worldId" | "sceneId" | "worldEpoch">,
  result: DispatchedInteraction,
): boolean {
  if (!sameSession || !current || current.worldId !== sourceIdentity.worldId ||
      current.sceneId !== sourceIdentity.sceneId || current.worldEpoch !== sourceIdentity.worldEpoch ||
      result.snapshot.worldEpoch < sourceIdentity.worldEpoch) return false;
  if (result.snapshot.worldEpoch > sourceIdentity.worldEpoch) return true;
  return result.snapshot.serverTick >= current.serverTick &&
    result.snapshot.authorityRevision >= current.authorityRevision;
}

/** A command's feedback belongs only to its accepted, ready destination. */
export function isCurrentSceneInteractionFeedback(
  current: WorldSnapshotV3 | null,
  expected: WorldSnapshotV3,
): boolean {
  return !!current && !current.entryToken && current.player.currentHp > 0 &&
    current.worldId === expected.worldId && current.sceneId === expected.sceneId &&
    current.worldEpoch === expected.worldEpoch && current.serverTick >= expected.serverTick &&
    current.authorityRevision >= expected.authorityRevision;
}
