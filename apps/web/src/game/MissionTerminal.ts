import { clockworksStatus } from "./ClockworksCampaign.js";
import { mistHarborRevisitStatus } from "./MistHarborRevisit.js";
import type { WorldSnapshotV3 } from "../protocol/types.js";

interface WorldProgressRow {
  worldId?: unknown;
  completed?: unknown;
  firstCompletion?: unknown;
  completedEvents?: unknown;
  revisitCount?: unknown;
}

/** Presents mission state strictly from the authoritative Rust v3 projection. */
export function missionTerminalSummary(snapshot: WorldSnapshotV3): string {
  const rows = Array.isArray(snapshot.progression?.worlds)
    ? snapshot.progression.worlds as WorldProgressRow[]
    : [];
  const greyHive = rows.find(row => row.worldId === "grey_hive");
  const greyHiveGate = snapshot.interactables.find(item =>
    item.entityId === "rs_world_gate_marker" && item.kind === "world_gate");
  const mistHarborGate = snapshot.interactables.find(item =>
    item.entityId === "rs_mh_world_gate_marker" && item.kind === "world_gate");
  const mistHarbor = rows.find(row => row.worldId === "mist_harbor");
  const completedEvents = Array.isArray(mistHarbor?.completedEvents)
    ? mistHarbor.completedEvents.filter((event): event is string => typeof event === "string")
    : [];
  const greyHiveAccess = greyHiveGate
    ? greyHiveGate.active ? "入口可用" : "入口当前不可用"
    : "入口状态未报告";
  const greyHiveFirstClear = greyHive
    ? greyHive.completed === true && greyHive.firstCompletion === true ? "已完成" : "尚未完成"
    : "状态未报告";
  const ghCleared = greyHive?.completed === true && greyHive.firstCompletion === true;
  const mistHarborStatus = snapshot.worldId === "mist_harbor"
    ? `当前世界 · 主线目标 ${completedEvents.filter(event =>
      ["mist_beacon_west", "mist_beacon_east", "mist_signal"].includes(event)).length}/3`
    : mistHarbor?.completed === true
      ? mistHarborRevisitStatus(mistHarbor.revisitCount, mistHarborGate)
      : mistHarborGate?.active ? "归航站入口可用"
        : ghCleared ? "已解锁 · 前往归航站进入" : "灰巢首次撤离后解锁";
  return `灰巢设施：${greyHiveAccess}；首次撤离：${greyHiveFirstClear}。雾港余烬：${mistHarborStatus}。钟骨工厂：${clockworksStatus(snapshot)}。`;
}
