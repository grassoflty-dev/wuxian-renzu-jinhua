import { signalWraithInterference, invalidSignalWraithProjection } from "../../protocol/PresentationEventValidation.js";
import type { WorldSnapshotV3 } from "../../protocol/types.js";

const WORLD_ID = "mist_harbor";

export interface MistHarborStatusView {
  westBeacon: string;
  eastBeacon: string;
  signal: string;
  signalInterference: string;
  waterSlowdown: string;
}

const NOT_APPLICABLE = "不适用 · 当前不在对应现场";
const NOT_REPORTED = "未报告 · 权威状态不可用";

function hazardOccupancy(snapshot: WorldSnapshotV3, sceneId: string, entityId: string, kind: string): string {
  if (snapshot.sceneId !== sceneId) return NOT_APPLICABLE;
  if (!Array.isArray(snapshot.hazards)) return NOT_REPORTED;
  const matches = snapshot.hazards.filter(hazard => hazard?.entityId === entityId);
  if (matches.length !== 1) return NOT_REPORTED;
  const hazard = matches[0];
  const position = hazard?.transform?.positionM;
  if (hazard?.kind !== kind || typeof hazard.active !== "boolean" ||
      !position || ![position.xM, position.yM, position.zM, hazard.transform.yawRad].every(Number.isFinite)) {
    return NOT_REPORTED;
  }
  return hazard.active ? "区内 · Rust 权威占用" : "区外 · Rust 权威占用";
}

function completedEvents(snapshot: WorldSnapshotV3): ReadonlySet<string> | null {
  if (snapshot.progression?.currentWorldId !== WORLD_ID ||
      !Array.isArray(snapshot.progression.worlds)) return null;
  const worlds = snapshot.progression.worlds.filter(row =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === WORLD_ID);
  if (worlds.length !== 1) return null;
  const events = (worlds[0] as Record<string, unknown>).completedEvents;
  if (!Array.isArray(events) || !events.every(event => typeof event === "string")) return null;
  return new Set(events);
}

/** Only the current world's Rust route events can confirm these three milestones. */
export function deriveMistHarborStatus(snapshot: WorldSnapshotV3): MistHarborStatusView | null {
  if (snapshot.worldId !== WORLD_ID) return null;
  const events = completedEvents(snapshot);
  const status = (eventId: string): string => !events ? "未确认 · 权威进度未报告"
    : events.has(eventId) ? "已完成 · Rust 权威进度" : "待完成 · Rust 权威进度";
  return {
    westBeacon: status("mist_beacon_west"),
    eastBeacon: status("mist_beacon_east"),
    signal: status("mist_signal"),
    signalInterference: invalidSignalWraithProjection(snapshot) ? NOT_REPORTED : signalWraithInterference(snapshot) ? "灵影干扰 · 信号方向不精确" : hazardOccupancy(snapshot, "mh_signal_yard", "mh_signal_interference_region", "signal_interference_zone"),
    waterSlowdown: hazardOccupancy(snapshot, "mh_drowned_quay", "mh_water_depth_region", "water_depth_slowdown"),
  };
}
