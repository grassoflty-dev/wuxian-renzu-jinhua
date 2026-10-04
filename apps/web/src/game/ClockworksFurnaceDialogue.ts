import type { InteractableView, WorldSnapshotV3 } from "../protocol/types.js";

export const CLOCKWORKS_FURNACE_DIALOGUE_ID = "cw_cy_heat_01_static_dialogue_marker";
export const CLOCKWORKS_FURNACE_DIALOGUE_PROMPT = "[F] 查看炉心状态";
const LINE = "炉心不是失控——它被维持在过载边缘。";
type Source = Pick<WorldSnapshotV3, "worldId" | "sceneId">;
type Target = Pick<InteractableView, "entityId" | "kind">;
type Receipt = { applied: boolean; alreadyApplied?: boolean; errorCode: string | null; snapshot: WorldSnapshotV3 };

/** A single authored information point, not a generic static-dialogue allowlist. */
export function isClockworksFurnaceDialogueTarget(source: Source | undefined, target: Target): boolean {
  return source?.worldId === "clockworks" && source.sceneId === "cw_furnace_heart" &&
    target.entityId === CLOCKWORKS_FURNACE_DIALOGUE_ID && target.kind === "static_dialogue_marker";
}

function projectedTarget(snapshot: WorldSnapshotV3, active: boolean): InteractableView | null {
  const matches = snapshot.interactables.filter(item => item.entityId === CLOCKWORKS_FURNACE_DIALOGUE_ID);
  const target = matches[0];
  return matches.length === 1 && target && isClockworksFurnaceDialogueTarget(snapshot, target) &&
    target.active === active ? target : null;
}

/** The hint uses the live, unique projection; it never substitutes for Rust validation. */
export function isClockworksFurnaceDialogueAvailable(snapshot: WorldSnapshotV3, target: Target): boolean {
  if (!isClockworksFurnaceDialogueTarget(snapshot, target) || !(snapshot.player.currentHp > 0)) return false;
  const prior = projectedTarget(snapshot, true);
  if (!prior) return false;
  const player = snapshot.player.transform.positionM;
  const position = prior.transform.positionM;
  return [player.xM, player.yM, player.zM, position.xM, position.yM, position.zM].every(Number.isFinite) &&
    Math.hypot(player.xM - position.xM, player.yM - position.yM, player.zM - position.zM) <= 2.5;
}

/** Called only after main accepts the receipt through its session/entry/ready fences. */
export function confirmedClockworksFurnaceDialogue(
  before: WorldSnapshotV3, targetId: string, kind: string, receipt: Receipt,
): string | null {
  const after = receipt.snapshot;
  if (!isClockworksFurnaceDialogueTarget(before, { entityId: targetId, kind }) ||
      receipt.applied !== true || receipt.alreadyApplied === true || receipt.errorCode !== null ||
      before.kind !== "full" || after.kind !== "full" || before.protocolVersion !== 3 || after.protocolVersion !== 3 ||
      before.entryToken || after.entryToken || !(before.player.currentHp > 0) || !(after.player.currentHp > 0) ||
      after.worldId !== before.worldId || after.sceneId !== before.sceneId || after.worldEpoch !== before.worldEpoch ||
      ![before.worldEpoch, before.serverTick, after.serverTick, before.authorityRevision, after.authorityRevision]
        .every(value => Number.isSafeInteger(value) && value >= 0) ||
      before.worldEpoch < 1 || after.serverTick < before.serverTick || after.authorityRevision <= before.authorityRevision) return null;
  if (!isClockworksFurnaceDialogueAvailable(before, { entityId: targetId, kind }) ||
      !projectedTarget(after, false)) return null;
  return LINE;
}
