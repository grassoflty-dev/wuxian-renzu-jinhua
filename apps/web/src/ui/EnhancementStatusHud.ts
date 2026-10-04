import type { WorldSnapshotV3 } from "../protocol/types.js";

export const FIRST_CLEAR_ENHANCEMENTS = {
  "information.local_map_i": "局部地图",
  "perception.rear_view_i": "后方视野",
  "body.regeneration_i": "再生能力",
} as const;

export type FirstClearEnhancementId = keyof typeof FIRST_CLEAR_ENHANCEMENTS;

export interface EnhancementStatusView {
  capabilityId: FirstClearEnhancementId;
  label: string;
  availability: string;
  details: string[];
}

function isFirstGreyHiveClear(snapshot: WorldSnapshotV3): boolean {
  const worlds = snapshot.progression?.worlds;
  if (!Array.isArray(worlds)) return false;
  return worlds.some(progress => {
    if (!progress || typeof progress !== "object") return false;
    const world = progress as Record<string, unknown>;
    return world.worldId === "grey_hive" && world.completed === true;
  });
}

function mapDetails(snapshot: WorldSnapshotV3): string[] {
  const map = snapshot.capabilities.exploredMap;
  if (!map || typeof map !== "object" || map.worldId !== snapshot.worldId ||
      !Array.isArray(map.rooms) || !Array.isArray(map.connections) || !Array.isArray(map.objectives)) return [];
  const details: string[] = [];
  const rooms = map.rooms.filter(room => !!room && typeof room === "object" &&
    typeof room.roomId === "string" && room.roomId.length > 0);
  const connections = map.connections.filter(connection =>
    !!connection && typeof connection === "object" &&
    typeof connection.fromRoomId === "string" && connection.fromRoomId.length > 0 &&
    typeof connection.toRoomId === "string" && connection.toRoomId.length > 0);
  const objectives = map.objectives.filter(objective =>
    !!objective && typeof objective === "object" &&
    typeof objective.objectiveId === "string" && objective.objectiveId.length > 0);
  if (rooms.length > 0) {
    const label = snapshot.capabilities.mapTopologyAuthorized === true ? "获准地图区域" : "已探索区域";
    details.push(`${label}：${rooms.map(room => room.roomId).join("、")}`);
  }
  if (connections.length > 0) {
    details.push(`已知通路：${connections.map(link => `${link.fromRoomId} → ${link.toRoomId}`).join("、")}`);
  }
  if (objectives.length > 0) details.push(`已知目标：${objectives.map(objective => objective.objectiveId).join("、")}`);
  return details;
}

export function deriveEnhancementStatus(snapshot: WorldSnapshotV3): EnhancementStatusView | null {
  if (!isFirstGreyHiveClear(snapshot) || !Array.isArray(snapshot.capabilities.items)) return null;
  const id = snapshot.capabilities.firstEnhancementChoice;
  if (typeof id !== "string" || !Object.hasOwn(FIRST_CLEAR_ENHANCEMENTS, id)) return null;
  const choices = snapshot.capabilities.items.filter(item => item?.capabilityId === id && item.granted === true);
  if (choices.length !== 1) return null;
  const capabilityId = id as FirstClearEnhancementId;
  if (capabilityId === "information.local_map_i" && !choices[0]!.selected) return { capabilityId, label: FIRST_CLEAR_ENHANCEMENTS[capabilityId],
    availability: "已获得 · 请在能力管理中配置", details: [] };
  switch (capabilityId) {
    case "information.local_map_i": {
      const details = mapDetails(snapshot);
      return {
        capabilityId,
        label: FIRST_CLEAR_ENHANCEMENTS[capabilityId],
        availability: details.length > 0 ? "已授权 · 已接收探索投影" : "已授权 · 当前无探索投影",
        details,
      };
    }
    case "perception.rear_view_i": {
      const authorization = snapshot.capabilities.rearView;
      const available = authorization?.granted === true && authorization.grantId === capabilityId;
      return {
        capabilityId,
        label: FIRST_CLEAR_ENHANCEMENTS[capabilityId],
        availability: available ? "服务端授权有效" : "尚未收到服务端授权",
        details: [],
      };
    }
    case "body.regeneration_i":
      return {
        capabilityId,
        label: FIRST_CLEAR_ENHANCEMENTS[capabilityId],
        availability: "已授权",
        details: [Number.isFinite(snapshot.player.currentHp) && Number.isFinite(snapshot.player.maxHp) && snapshot.player.maxHp > 0
          ? `HP（权威快照）：${snapshot.player.currentHp} / ${snapshot.player.maxHp}`
          : "HP（权威快照）：— / —"],
      };
  }
  return null;
}

export class EnhancementStatusHud {
  private lastSignature: string | null = null;

  constructor(private readonly root: HTMLElement) {}

  apply(snapshot: WorldSnapshotV3): void {
    const status = deriveEnhancementStatus(snapshot);
    if (!status) {
      this.reset();
      return;
    }
    const signature = JSON.stringify(status);
    if (signature === this.lastSignature) return;
    this.lastSignature = signature;
    const heading = document.createElement("strong");
    heading.textContent = `${status.label} · ${status.availability}`;
    const details = status.details.map(detail => {
      const line = document.createElement("p");
      line.textContent = detail;
      return line;
    });
    this.root.replaceChildren(heading, ...details);
    this.root.hidden = false;
  }

  reset(): void {
    if (this.lastSignature === null && this.root.hidden) return;
    this.lastSignature = null;
    this.root.replaceChildren();
    this.root.hidden = true;
  }
}
