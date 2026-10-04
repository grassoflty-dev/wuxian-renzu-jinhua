import type { WorldSnapshotV3 } from "../protocol/types.js";

type ReturnSnapshot = Pick<WorldSnapshotV3,
  "worldId" | "sceneId" | "worldEpoch" | "authorityRevision" | "progression">;
type GateReceipt = {
  applied: boolean;
  alreadyApplied?: boolean;
  errorCode: string | null;
  snapshot: ReturnSnapshot;
};

const AFTER_GREY_HIVE_LINE = "灰巢信标已解析。检测到第二组低可信坐标：雾港余烬。";
const NEW_JOURNEY_LINE = "归航链路稳定。请选择首次回收坐标：灰巢设施。";
const REQUIRED_EVENTS = ["hive_power", "hive_lockdown", "hive_extraction"];

/** A fresh native journey begins in Return Station before its bootstrap Grey Hive visit. */
export function confirmedReturnStationNewJourney(snapshot: WorldSnapshotV3): string | null {
  if (snapshot?.kind !== "full" || snapshot.protocolVersion !== 3 ||
      snapshot.worldId !== "return_station" || snapshot.sceneId !== "rs_core_room" ||
      !Number.isSafeInteger(snapshot.worldEpoch) ||
      !Number.isSafeInteger(snapshot.authorityRevision)) return null;
  const route = snapshot.progression;
  if (route?.currentWorldId !== "grey_hive" || route.eventSeq !== 1 ||
      !Array.isArray(route.worlds)) return null;
  const matching = route.worlds.filter(row =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === "grey_hive");
  if (matching.length !== 1) return null;
  const progress = matching[0] as Record<string, unknown>;
  if (progress.completed !== false || progress.firstCompletion !== false ||
      progress.visitId !== 1 || progress.cycleId !== 1 ||
      progress.revisitCount !== 0 || !Array.isArray(progress.completedEvents) ||
      progress.completedEvents.length !== 0) return null;
  return NEW_JOURNEY_LINE;
}

function firstGreyHiveClear(snapshot: ReturnSnapshot): ReadonlySet<string> | null {
  const route = snapshot.progression;
  if (route?.currentWorldId !== "grey_hive" || !Array.isArray(route.worlds)) return null;
  const matching = route.worlds.filter(row =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === "grey_hive");
  if (matching.length !== 1) return null;
  const progress = matching[0] as Record<string, unknown>;
  const events = progress.completedEvents;
  if (progress.completed !== true || progress.firstCompletion !== true ||
      progress.visitId !== 1 || progress.revisitCount !== 0 ||
      !Array.isArray(events) || !events.every(event => typeof event === "string")) return null;
  const completed = new Set<string>(events);
  return REQUIRED_EVENTS.every(event => completed.has(event)) ? completed : null;
}

/** The catalog line is delivered by the first applied extraction return receipt. */
export function confirmedReturnStationAfterGreyHive(
  before: ReturnSnapshot,
  targetId: string,
  kind: string,
  receipt: GateReceipt,
): string | null {
  const after = receipt.snapshot;
  if (before.worldId !== "grey_hive" || before.sceneId !== "gh_exit" ||
      targetId !== "gh_extraction_return_to_rs" || kind !== "world_gate" ||
      receipt.applied !== true || receipt.alreadyApplied === true || receipt.errorCode !== null ||
      after.worldId !== "return_station" || after.sceneId !== "rs_core_room" ||
      !Number.isSafeInteger(before.worldEpoch) || !Number.isSafeInteger(after.worldEpoch) ||
      after.worldEpoch !== before.worldEpoch + 1 ||
      !Number.isSafeInteger(before.authorityRevision) ||
      !Number.isSafeInteger(after.authorityRevision) ||
      after.authorityRevision <= before.authorityRevision ||
      !Number.isSafeInteger(before.progression?.eventSeq) ||
      after.progression?.eventSeq !== before.progression.eventSeq) return null;
  const prior = firstGreyHiveClear(before);
  const current = firstGreyHiveClear(after);
  if (!prior || !current || prior.size !== current.size ||
      [...prior].some(event => !current.has(event))) return null;
  return AFTER_GREY_HIVE_LINE;
}
