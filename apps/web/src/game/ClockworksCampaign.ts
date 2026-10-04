import type { WorldSnapshotV3 } from "../protocol/types.js";

export const CLOCKWORKS_EPILOGUE = "原来门一直不止三扇。";
export const CLOCKWORKS_RETURN_LINE = "三处坐标不是孤立事件。网络中仍存在未识别节点。";
const EVENTS = ["clockworks_valves", "clockworks_core", "clockworks_shutdown"];
const AIR_STEP = "mobility.air_step_i";

type Receipt = { applied: boolean; alreadyApplied?: boolean; errorCode: string | null; snapshot: WorldSnapshotV3 };
type Progress = Record<string, unknown>;

function progress(snapshot: WorldSnapshotV3, worldId: string): Progress | null {
  const rows = snapshot.progression?.worlds;
  if (!Array.isArray(rows)) return null;
  const matching = rows.filter(row => row && typeof row === "object" &&
    (row as Progress).worldId === worldId);
  return matching.length === 1 ? matching[0] as Progress : null;
}

function events(row: Progress | null): Set<string> | null {
  if (!Array.isArray(row?.completedEvents) ||
      !row.completedEvents.every(event => typeof event === "string" && EVENTS.includes(event))) return null;
  const unique = new Set<string>(row.completedEvents);
  return unique.size === row.completedEvents.length ? unique : null;
}

/** Completion is projected by Rust only after the shutdown transaction. */
export function clockworksCompleted(snapshot: WorldSnapshotV3): boolean {
  const row = progress(snapshot, "clockworks");
  const completedEvents = events(row);
  const airSteps = Array.isArray(snapshot.capabilities?.items)
    ? snapshot.capabilities.items.filter(item => item.capabilityId === AIR_STEP) : [];
  return snapshot.kind === "full" && snapshot.protocolVersion === 3 &&
    row?.completed === true && row.firstCompletion === true &&
    Number.isSafeInteger(row.revisitCount) && (row.revisitCount as number) >= 0 && (row.revisitCount as number) <= 4 &&
    Number.isSafeInteger(row.visitId) && row.visitId === (row.revisitCount as number) + 1 &&
    completedEvents?.size === 3 && EVENTS.every(event => completedEvents.has(event)) &&
    airSteps.length === 1 && airSteps[0]?.granted === true;
}

export function clockworksStatus(snapshot: WorldSnapshotV3): string {
  const row = progress(snapshot, "clockworks");
  if (!row || typeof row.completed !== "boolean" || typeof row.firstCompletion !== "boolean" ||
      !events(row) || !Number.isSafeInteger(row.revisitCount) ||
      (row.revisitCount as number) < 0 || (row.revisitCount as number) > 4) return "状态未报告";
  if (snapshot.worldId === "clockworks") return clockworksCompleted(snapshot)
    ? "当前世界 · 已完成结算" : "当前世界 · 尚未完成结算";
  const gates = snapshot.interactables.filter(item => item.entityId === "rs_cw_world_gate_marker" && item.kind === "world_gate");
  const gate = gates.length === 1 ? gates[0] : undefined;
  if (clockworksCompleted(snapshot)) {
    if (row.revisitCount === 4) return "已完成 · 复访次数用尽";
    return gate ? gate.active ? "已完成 · 限次复访入口可用" : "已完成 · 入口当前不可用"
      : "已完成 · 前往归航站查询复访";
  }
  if (row.completed === true) return "结算状态未完整报告";
  if (gate) return gate.active ? "归航站入口可用" : "入口当前不可用";
  const mh = progress(snapshot, "mist_harbor");
  return mh?.completed === true && mh.firstCompletion === true
    ? "前往归航站查询链路" : "雾港实际撤离后解锁";
}

function validReceipt(before: WorldSnapshotV3, receipt: Receipt): boolean {
  const after = receipt.snapshot;
  return receipt.applied === true && receipt.alreadyApplied !== true && receipt.errorCode === null &&
    before.kind === "full" && after.kind === "full" &&
    before.protocolVersion === 3 && after.protocolVersion === 3 &&
    before.player?.currentHp > 0 && after.player?.currentHp > 0 &&
    [before.worldEpoch, after.worldEpoch, before.serverTick, after.serverTick,
      before.authorityRevision, after.authorityRevision, before.progression?.eventSeq,
      after.progression?.eventSeq].every(value => Number.isSafeInteger(value) && value >= 0) &&
    after.serverTick >= before.serverTick && after.authorityRevision > before.authorityRevision;
}

/** Never inferred from load or a render: only the first accepted physical shutdown. */
export function confirmedClockworksEpilogue(
  before: WorldSnapshotV3, targetId: string, kind: string, receipt: Receipt,
): string | null {
  const after = receipt.snapshot;
  const prior = progress(before, "clockworks");
  const current = progress(after, "clockworks");
  const priorEvents = events(prior);
  if (!validReceipt(before, receipt) || targetId !== "cw_master_shutdown_staged" || kind !== "terminal" ||
      before.worldId !== "clockworks" || before.sceneId !== "cw_shutdown_exit" ||
      after.worldId !== before.worldId || after.sceneId !== before.sceneId ||
      after.worldEpoch !== before.worldEpoch || before.entryToken || after.entryToken ||
      prior?.completed !== false || prior.firstCompletion !== false ||
      !priorEvents?.has("clockworks_valves") || !priorEvents.has("clockworks_core") || !clockworksCompleted(after) ||
      before.progression.currentWorldId !== "clockworks" || after.progression.currentWorldId !== "clockworks" ||
      current?.visitId !== prior.visitId || current?.revisitCount !== prior.revisitCount ||
      after.progression.eventSeq - before.progression.eventSeq !== (priorEvents.has("clockworks_shutdown") ? 1 : 2)) return null;
  return CLOCKWORKS_EPILOGUE;
}

/** The first successful return carries the authored Return Station follow-up. */
export function confirmedReturnStationAfterClockworks(
  before: WorldSnapshotV3, targetId: string, kind: string, receipt: Receipt,
): string | null {
  const after = receipt.snapshot;
  const prior = progress(before, "clockworks");
  const current = progress(after, "clockworks");
  if (!validReceipt(before, receipt) || targetId !== "cw_shutdown_return_to_rs" || kind !== "world_gate" ||
      before.worldId !== "clockworks" || before.sceneId !== "cw_shutdown_exit" ||
      after.worldId !== "return_station" || after.sceneId !== "rs_core_room" ||
      after.worldEpoch !== before.worldEpoch + 1 || !clockworksCompleted(before) || !clockworksCompleted(after) ||
      prior?.visitId !== 1 || prior.revisitCount !== 0 || current?.visitId !== 1 || current.revisitCount !== 0 ||
      before.progression.currentWorldId !== "clockworks" || after.progression.currentWorldId !== "clockworks" ||
      after.progression.eventSeq !== before.progression.eventSeq) return null;
  return CLOCKWORKS_RETURN_LINE;
}
