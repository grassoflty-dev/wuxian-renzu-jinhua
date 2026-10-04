import { validActionPresentation, type ActionPresentation, type Vec3 } from "../protocol/types.js";

export type ActionVfxKind = "primaryAttack" | "pulse" | "guard" | "pierce" | "dash";

export interface ActionVfxSource {
  worldId: string;
  sceneId: string;
  worldEpoch: number;
  actionState: string;
  actionPresentation?: ActionPresentation;
  serverTick?: number;
  position: Vec3;
  facingX: number;
  facingZ: number;
  nowMs: number;
}

export interface ActionVfx {
  kind: ActionVfxKind;
  position: Vec3;
  directionX: number;
  directionZ: number;
  progress: number;
  arcDegrees: number;
  phase: ActionPresentation["phase"];
  rangeM: number;
  lineHalfWidthM: number;
}

export const ACTION_VFX_DASH_DURATION_MS = 300;
export const ACTION_VFX_PULSE_DURATION_MS = 560;
export const ACTION_VFX_GUARD_ARC_DEGREES = 140;

function isActionVfxKind(value: string): value is ActionVfxKind {
  return value === "primaryAttack" || value === "pulse" || value === "guard" || value === "pierce" || value === "dash";
}

/** Produces presentation cues only from a v3-authoritative action state and transform. */
export class ActionVfxModel {
  private sceneKey: string | null = null;
  private lastReceipt = "";
  private receivedAt = 0;

  update(source: ActionVfxSource): ActionVfx | null {
    const sceneKey = `${source.worldId}\0${source.sceneId}\0${source.worldEpoch}`;
    if (sceneKey !== this.sceneKey) {
      this.sceneKey = sceneKey;
      this.lastReceipt = "";
    }

    const action = source.actionPresentation;
    if (!isActionVfxKind(source.actionState) || !validActionPresentation(action, source.actionState) || !Number.isFinite(source.nowMs)) return null;
    const receipt = `${source.serverTick ?? ""}:${action.requestId}:${action.elapsedMs}:${action.phase}`;
    if (receipt !== this.lastReceipt) { this.lastReceipt = receipt; this.receivedAt = source.nowMs; }
    // Never turn windup into active locally. An overdue authority receipt can
    // only remove an effect; fresh authority is required to show it again.
    if (source.nowMs - this.receivedAt >= Math.min(250, action.durationMs - action.elapsedMs)) return null;
    const length = Math.hypot(source.facingX, source.facingZ);
    if (!Number.isFinite(length) || length <= 1e-6) return null;
    return {
      kind: source.actionState,
      position: source.position,
      directionX: source.facingX / length,
      directionZ: source.facingZ / length,
      progress: action.elapsedMs / action.durationMs,
      arcDegrees: source.actionState === "guard" ? ACTION_VFX_GUARD_ARC_DEGREES : 0,
      phase: action.phase, rangeM: action.rangeM, lineHalfWidthM: action.lineHalfWidthM,
    };
  }

  reset(): void {
    this.sceneKey = null;
    this.lastReceipt = "";
    this.receivedAt = 0;
  }
}
