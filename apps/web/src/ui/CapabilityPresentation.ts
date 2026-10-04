import type { CapabilityItemProjection, WorldSnapshotV3 } from "../protocol/types.js";

/** GAMEPLAY-2026-09-28 §7: organization only, never an unlock or gameplay rule. */
export const CAPABILITY_CATEGORIES = [
  ["perception", "感知"],
  ["information", "信息"],
  ["body", "身体"],
  ["mobility", "机动"],
  ["cognition", "认知"],
  ["combat", "战斗"],
  ["space", "空间"],
  ["soul", "灵魂"],
  ["rule", "规则"],
  ["utility", "实用"],
] as const;

/** Mirrors the six currently implemented Rust capability IDs, not future content. */
export const CAPABILITY_LABELS = {
  "information.local_map_i": "局部地图",
  "information.enemy_vitals_basic": "敌人生命信息",
  "perception.rear_view_i": "后方视野",
  "perception.acoustic_mapping_i": "声学映射",
  "body.regeneration_i": "再生能力",
  "mobility.air_step_i": "空中踏步",
} as const;
export type KnownCapabilityId = keyof typeof CAPABILITY_LABELS;
export type CapabilityState =
  | { status: "missing" }
  | { status: "invalid" }
  | { status: "valid"; row: CapabilityItemProjection };

export function capabilityState(snapshot: WorldSnapshotV3, id: KnownCapabilityId): CapabilityState {
  if (!Object.hasOwn(CAPABILITY_LABELS, id)) return { status: "invalid" };
  const items: unknown = snapshot.capabilities?.items;
  if (!Array.isArray(items)) return { status: "missing" };
  // Count before validating: a malformed duplicate must not leave a valid row visible.
  const matches = items.filter(value => value && typeof value === "object" && value.capabilityId === id);
  if (matches.length === 0) return { status: "missing" };
  if (matches.length !== 1) return { status: "invalid" };
  const row = matches[0];
  if (typeof row.granted !== "boolean" || typeof row.selected !== "boolean" ||
    (!row.granted && row.selected) || (row.cooldownRemainingMs !== undefined &&
      (!Number.isSafeInteger(row.cooldownRemainingMs) || row.cooldownRemainingMs < 0))) return { status: "invalid" };
  return { status: "valid", row };
}

export function knownCapabilityRows(snapshot: WorldSnapshotV3): CapabilityItemProjection[] {
  return (Object.keys(CAPABILITY_LABELS) as KnownCapabilityId[]).flatMap(id => {
    const state = capabilityState(snapshot, id);
    return state.status === "valid" ? [state.row] : [];
  });
}

export function capabilityLabel(id: string): string | null {
  return Object.hasOwn(CAPABILITY_LABELS, id) ? CAPABILITY_LABELS[id as KnownCapabilityId] : null;
}

export function capabilityStatus(snapshot: WorldSnapshotV3, id: KnownCapabilityId): string {
  const state = capabilityState(snapshot, id);
  if (state.status === "missing") return "状态未报告·不可用";
  if (state.status === "invalid") return "投影状态不一致·不可用";
  if (!state.row.granted) return "未获得·不可用";
  return state.row.selected ? "已获得·已选中" : "已获得·未选中";
}
