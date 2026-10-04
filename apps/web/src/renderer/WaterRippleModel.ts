import type { Vec3, WorldSnapshotEnvelope } from "../protocol/types.js";
import { sceneIdentityKey } from "./LayerModel.js";

const PERIOD_MS = 1800;
const WATER_REGION = "mh_water_depth_region";

export interface WaterRippleFrame {
  position: Vec3;
  progress: number;
  reducedMotion: boolean;
}

function finiteTransform(value: unknown): value is { positionM: Vec3; yawRad: number } {
  if (typeof value !== "object" || value === null) return false;
  const transform = value as { positionM?: Vec3; yawRad?: number };
  const position = transform.positionM;
  return typeof transform.yawRad === "number" && Number.isFinite(transform.yawRad) &&
    typeof position === "object" && position !== null &&
    [position.xM, position.yM, position.zM].every(Number.isFinite);
}

/** Presentation-only cue for the Rust-projected occupied Drowned Quay water region. */
export class WaterRippleModel {
  private identityKey: string | null = null;
  private startedAtMs: number | null = null;

  reset(): void {
    this.identityKey = null;
    this.startedAtMs = null;
  }

  project(snapshot: WorldSnapshotEnvelope, nowMs: number, reducedMotion: boolean): WaterRippleFrame | null {
    if (snapshot.protocolVersion !== 3 || snapshot.worldId !== "mist_harbor" ||
        snapshot.sceneId !== "mh_drowned_quay" || !Array.isArray(snapshot.hazards) ||
        !finiteTransform(snapshot.player?.transform) || !Number.isFinite(nowMs)) {
      this.reset();
      return null;
    }
    const identity = sceneIdentityKey(snapshot);
    if (identity !== this.identityKey) {
      this.identityKey = identity;
      this.startedAtMs = null;
    }
    const matches = snapshot.hazards.filter(hazard => hazard?.entityId === WATER_REGION);
    if (matches.length !== 1) { this.startedAtMs = null; return null; }
    const hazard = matches[0]!;
    if (hazard.kind !== "water_depth_slowdown" || hazard.active !== true ||
        !finiteTransform(hazard.transform)) {
      this.startedAtMs = null;
      return null;
    }
    if (this.startedAtMs === null) this.startedAtMs = nowMs;
    let elapsedMs = nowMs - this.startedAtMs;
    if (!Number.isFinite(elapsedMs) || elapsedMs < 0) {
      this.startedAtMs = nowMs;
      elapsedMs = 0;
    }
    return { position: { ...snapshot.player.transform.positionM },
      progress: reducedMotion ? 0 : (elapsedMs % PERIOD_MS) / PERIOD_MS,
      reducedMotion };
  }
}
