import { clockworksCompleted, clockworksStatus } from "../game/ClockworksCampaign.js";
import { mistHarborRevisitStatus } from "../game/MistHarborRevisit.js";
import type { WorldSnapshotV3 } from "../protocol/types.js";
import { capabilityLabel, knownCapabilityRows } from "./CapabilityPresentation.js";
import { deriveMistHarborStatus } from "./components/MistHarborStatusPanel.js";

export const CORE_PANEL_IDS = [
  "inventory", "character", "capability", "mission", "archive", "save", "settings", "world_network",
] as const;
export type CorePanelId = typeof CORE_PANEL_IDS[number];

export const CORE_PANEL_LABELS: Record<CorePanelId, string> = {
  inventory: "背包", character: "角色", capability: "能力", mission: "任务",
  archive: "档案", save: "存档", settings: "系统", world_network: "世界网络",
};

export interface CorePanelRow { label: string; value: string }
export interface CorePanelView {
  title: string;
  subtitle: string;
  rows: CorePanelRow[];
}

function meter(current: number, maximum: number): string {
  if (!Number.isFinite(current) || !Number.isFinite(maximum) || maximum <= 0) return "状态未报告";
  return `${Math.round(Math.max(0, Math.min(current, maximum)))} / ${Math.round(maximum)}`;
}

function worldName(id: string): string {
  switch (id) {
    case "return_station": return "归航站";
    case "grey_hive": return "灰巢设施";
    case "mist_harbor": return "雾港余烬";
    case "clockworks": return "钟骨工厂";
    default: return id || "状态未报告";
  }
}

function progress(snapshot: WorldSnapshotV3, worldId: string): Record<string, unknown> | undefined {
  const rows = snapshot.progression?.worlds;
  if (!Array.isArray(rows)) return undefined;
  return rows.find((row): row is Record<string, unknown> =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === worldId) as Record<string, unknown> | undefined;
}

function record(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

/** Validate only Archive's evidence; unrelated route rows cannot hide a valid world. */
function archiveEvents(worlds: unknown[], worldId: string, allowed: readonly string[]): ReadonlySet<string> | null {
  const matching = worlds.filter(row => record(row) && row.worldId === worldId);
  if (matching.length !== 1) return null;
  const events = (matching[0] as Record<string, unknown>).completedEvents;
  if (!Array.isArray(events)) return null;
  const confirmed = new Set<string>();
  for (const event of events) {
    if (typeof event !== "string" || !allowed.includes(event) || confirmed.has(event)) return null;
    confirmed.add(event);
  }
  return confirmed;
}

/** Current confirmed events, not reading history or a promise that progress was saved. */
function archivePanel(snapshot: WorldSnapshotV3 | null): CorePanelView {
  const unavailable = { title: "档案", subtitle: "档案数据尚未同步。", rows: [] };
  // Full v3 and RouteProjection v1 are the existing wire contract, including restored old saves.
  if (!record(snapshot) || snapshot.kind !== "full" || snapshot.protocolVersion !== 3 ||
      !record(snapshot.progression) || snapshot.progression.schemaVersion !== 1 ||
      !Array.isArray(snapshot.progression.worlds)) return unavailable;
  const greyHive = archiveEvents(snapshot.progression.worlds, "grey_hive",
    ["hive_power", "hive_lockdown", "hive_extraction"]);
  const mistHarbor = archiveEvents(snapshot.progression.worlds, "mist_harbor",
    ["mist_beacon_west", "mist_beacon_east", "mist_signal"]);
  if (!greyHive && !mistHarbor) return unavailable;
  const rows: CorePanelRow[] = [];
  // Frozen order and original catalog bodies: gh_cy_power_01, gh_sys_lockdown, mh_sys_beacon_sync.
  if (greyHive?.has("hive_power")) rows.push({ label: "灰巢：主电恢复记录", value: "主电还活着，只是被人为切断。" });
  if (greyHive?.has("hive_lockdown")) rows.push({ label: "灰巢：隔离协议记录", value: "隔离协议已被局部覆盖。" });
  if (mistHarbor?.has("mist_beacon_west") && mistHarbor.has("mist_beacon_east")) {
    rows.push({ label: "雾港：双基准同步记录", value: "双基准建立，中心干扰源可定位。" });
  }
  return { title: "档案", subtitle: "已确认事件资料", rows };
}

/** Read-only HTML panel content. Missing authority remains explicitly unknown. */
export function deriveCorePanel(id: CorePanelId, snapshot: WorldSnapshotV3 | null): CorePanelView {
  const title = CORE_PANEL_LABELS[id];
  if (id === "settings") return {
    title, subtitle: "辅助设置始终可用", rows: [
      { label: "显示", value: "可在此切换高对比度和减少动态效果" },
      { label: "能力", value: "辅助设置不依赖任何能力解锁" },
    ],
  };
  if (id === "inventory") return {
    title, subtitle: "储物状态", rows: [
      { label: "背包", value: snapshot?.build ? `${snapshot.build.items.length} 种物品` : "尚未收到权威背包投影，暂不显示物品" },
      { label: "装备", value: snapshot?.build ? `${snapshot.build.equipment.length} 个已装备槽位` : "尚未收到权威装备投影" },
    ],
  };
  if (id === "save") return {
    title, subtitle: "现有存档流程", rows: [
      { label: "旅程中", value: "暂停后使用保存面板创建或覆盖存档" },
      { label: "主界面", value: "从有效存档列表选择继续" },
    ],
  };
  if (id === "archive") return archivePanel(snapshot);
  if (!snapshot) return { title, subtitle: "旅程尚未连接", rows: [{ label: "状态", value: "等待 Rust 权威快照" }] };

  if (id === "character") return {
    title, subtitle: "岑遥 · 状态来自当前快照", rows: [
      { label: "生命", value: meter(snapshot.player.currentHp, snapshot.player.maxHp) },
      { label: "能量", value: meter(snapshot.player.currentEnergy, snapshot.player.maxEnergy) },
      { label: "动作", value: snapshot.player.actionState || "状态未报告" },
      { label: "身体状态", value: "权威身体状态未提供" },
    ],
  };
  if (id === "capability") {
    const granted = knownCapabilityRows(snapshot).filter(item => item.granted === true);
    return {
      title, subtitle: "只显示权威授予的能力；树关系与成本未接入", rows: granted.length
        ? granted.map(item => ({
          label: capabilityLabel(item.capabilityId)!,
          value: item.selected ? "已获得 · 已选中" : "已获得",
        }))
        : [{ label: "能力", value: "尚无已授予能力" }],
    };
  }
  if (id === "mission") {
    const objectives = Array.isArray(snapshot.objectives) ? snapshot.objectives : [];
    const greyHive = progress(snapshot, "grey_hive");
    const mistHarbor = deriveMistHarborStatus(snapshot);
    return {
      title, subtitle: mistHarbor ? "当前任务与雾港进度" : "当前任务与灰巢进度", rows: [
        { label: "当前世界", value: worldName(snapshot.worldId) },
        { label: "灰巢首撤", value: greyHive?.completed === true && greyHive.firstCompletion === true ? "已完成" : "尚未完成" },
        ...(mistHarbor ? [
          { label: "雾港西侧航标", value: mistHarbor.westBeacon },
          { label: "雾港东侧航标", value: mistHarbor.eastBeacon },
          { label: "雾港信号", value: mistHarbor.signal },
          { label: "当前信号干扰", value: mistHarbor.signalInterference },
          { label: "当前水域减速", value: mistHarbor.waterSlowdown },
        ] : []),
        ...objectives.map(item => ({ label: item.objectiveId, value: item.state || "状态未报告" })),
      ],
    };
  }
  const greyHiveGate = snapshot.interactables.find(item =>
    item.entityId === "rs_world_gate_marker" && item.kind === "world_gate");
  const mistHarborGate = snapshot.interactables.find(item =>
    item.entityId === "rs_mh_world_gate_marker" && item.kind === "world_gate");
  const greyHive = progress(snapshot, "grey_hive");
  const mistHarbor = progress(snapshot, "mist_harbor");
  const greyHiveFirstClear = greyHive?.completed === true && greyHive.firstCompletion === true;
  const mistHarborStatus = snapshot.worldId === "mist_harbor" ? "当前世界"
    : mistHarbor?.completed === true ? mistHarborRevisitStatus(mistHarbor.revisitCount, mistHarborGate)
      : mistHarborGate?.active ? "归航站入口可用"
        : greyHiveFirstClear ? "已解锁 · 前往归航站进入" : "灰巢首次撤离后解锁";
  return {
    title, subtitle: "当前原生链路状态", rows: [
      { label: "当前坐标", value: worldName(snapshot.worldId) },
      { label: "灰巢设施", value: snapshot.worldId === "grey_hive" ? "当前世界"
        : greyHiveGate ? greyHiveGate.active ? "归航站入口可用" : "归航站入口当前不可用"
          : greyHiveFirstClear ? "首撤已完成 · 复访入口未报告" : "灰巢入口状态未报告" },
      { label: "雾港余烬", value: mistHarborStatus },
      { label: "钟骨工厂", value: clockworksStatus(snapshot) },
      ...(clockworksCompleted(snapshot) ? [
        { label: "???", value: "未识别节点" },
        { label: "???", value: "未识别节点" },
        { label: "???", value: "未识别节点" },
      ] : []),
    ],
  };
}
