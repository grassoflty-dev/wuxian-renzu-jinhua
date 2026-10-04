import type { WorldSnapshotV3 } from "../protocol/types.js";
import { CAPABILITY_LABELS, capabilityState, capabilityStatus, knownCapabilityRows, type KnownCapabilityId } from "../ui/CapabilityPresentation.js";

const FIRST_CLEAR_CHOICES: Record<string, string> = {
  "information.local_map_i": "局部地图",
  "perception.rear_view_i": "后方视野",
  "body.regeneration_i": "再生能力",
};

const ACOUSTIC_MAPPING_ID = "perception.acoustic_mapping_i";

function isGreyHiveFirstClear(snapshot: WorldSnapshotV3): boolean {
  return Array.isArray(snapshot.progression?.worlds) && snapshot.progression.worlds.some(value => {
    if (!value || typeof value !== "object") return false;
    const row = value as Record<string, unknown>;
    return row.worldId === "grey_hive" && row.completed === true && row.firstCompletion === true;
  });
}

/** Fail closed if the acoustic mapping projection is absent, duplicated, or malformed. */
export function acousticMappingStatus(snapshot: WorldSnapshotV3): string {
  return capabilityStatus(snapshot, ACOUSTIC_MAPPING_ID);
}

/** Formats only the capability and first-clear values in the Rust v3 snapshot. */
export function capabilityTerminalSummary(snapshot: WorldSnapshotV3): string {
  const rows = knownCapabilityRows(snapshot);
  const chosen = rows.filter(row => row.granted && row.selected &&
    Object.hasOwn(FIRST_CLEAR_CHOICES, row.capabilityId));
  const firstClear = isGreyHiveFirstClear(snapshot);
  const inconsistentChoice = Object.keys(FIRST_CLEAR_CHOICES).some(id =>
    capabilityState(snapshot, id as KnownCapabilityId).status === "invalid");
  const choiceText = !firstClear
    ? "首次撤离尚未完成（首通选择未开放）"
    : inconsistentChoice
      ? "首通选择状态需复核"
      : chosen.length === 1
      ? `首通选择：已获得${FIRST_CLEAR_CHOICES[chosen[0]!.capabilityId]}`
      : chosen.length > 1
        ? "首通选择状态需复核"
        : "首通选择：尚未确认";
  const capabilityRows = (Object.keys(CAPABILITY_LABELS) as KnownCapabilityId[])
    .filter(id => id !== ACOUSTIC_MAPPING_ID && capabilityState(snapshot, id).status !== "missing")
    .map(id => `${CAPABILITY_LABELS[id]}：${capabilityStatus(snapshot, id)}`);
  capabilityRows.push(`声学映射：${acousticMappingStatus(snapshot)}`);
  const capabilityText = capabilityRows.join("；");
  return `${choiceText}。能力状态：${capabilityText}。`;
}
