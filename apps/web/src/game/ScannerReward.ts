import type { InteractableView, WorldSnapshotV3 } from "../protocol/types.js";
import type { DispatchedInteraction } from "./SceneInteraction.js";

const CAPABILITY_ID = "information.enemy_vitals_basic";
export function isScannerTerminal(snapshot: Pick<WorldSnapshotV3, "worldId" | "sceneId">,
  item: Pick<InteractableView, "entityId" | "kind">): boolean {
  return snapshot.worldId === "grey_hive" && snapshot.sceneId === "gh_central_shaft" &&
    item.entityId === "gh_log_shaft_01" && item.kind === "facility_log";
}
function acquired(snapshot: WorldSnapshotV3): boolean {
  return snapshot.capabilities.items?.some(item => item.capabilityId === CAPABILITY_ID && item.granted === true) === true;
}
export function scannerTerminalLabel(snapshot: WorldSnapshotV3, item: InteractableView): string | null {
  return isScannerTerminal(snapshot, item) ? acquired(snapshot) ? "扫描模块已在线" : "恢复扫描模块" : null;
}
/** Feedback consumes only the accepted formal F receipt and its durable grant projection. */
export function scannerRewardFeedback(source: WorldSnapshotV3, item: InteractableView,
  result: DispatchedInteraction): string | null {
  if (!isScannerTerminal(source, item) || !isScannerTerminal(result.snapshot, item) ||
    source.worldEpoch !== result.snapshot.worldEpoch ||
    !(result.applied || ("alreadyApplied" in result && result.alreadyApplied)) ||
    result.errorCode || !acquired(result.snapshot)) return null;
  return acquired(source) || ("alreadyApplied" in result && result.alreadyApplied)
    ? "扫描模块已在线。" : "扫描模块已恢复，敌人生命状态信息已启用。";
}
