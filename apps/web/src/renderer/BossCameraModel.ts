import type { WorldSnapshotEnvelope, WorldSnapshotV3 } from "../protocol/types.js";

export const SENTINEL_BOSS_ZOOM = 0.86;
export const BOSS_CAMERA_SMOOTH_TIME_SECONDS = 0.24;

const SENTINEL_ENTITY_ID = "gh_sentinel_arena_sentinel_01";
const SENTINEL_ENTITY_TYPE = "enemy.grey_hive.sentinel";

/** Presentation-only zoom driven by the exact authoritative Sentinel actor identity. */
export class BossCameraModel {
  private sceneKey: string | null = null;
  private scale = 1;
  private suppressUntilBase = false;

  update(snapshot: WorldSnapshotEnvelope, deltaSeconds: number): number {
    if (!Number.isFinite(deltaSeconds) || deltaSeconds < 0) throw new Error("E_BOSS_CAMERA_DELTA");
    const v3 = snapshot.protocolVersion === 3 ? snapshot : null;
    const sceneKey = v3 ? `${v3.worldId}\0${v3.sceneId}\0${v3.worldEpoch}` : null;
    if (sceneKey !== this.sceneKey) {
      this.sceneKey = sceneKey;
      this.suppressUntilBase = this.scale < 1 - 1e-9;
    }

    const authorized = v3 && isSentinelEncounter(v3) && hasSingleActiveSentinel(v3);
    const target = authorized && !this.suppressUntilBase ? SENTINEL_BOSS_ZOOM : 1;
    const alpha = 1 - Math.exp(-deltaSeconds / BOSS_CAMERA_SMOOTH_TIME_SECONDS);
    this.scale += (target - this.scale) * alpha;
    if (Math.abs(target - this.scale) < 1e-9) this.scale = target;
    if (this.suppressUntilBase && this.scale === 1) this.suppressUntilBase = false;
    return Math.max(SENTINEL_BOSS_ZOOM, Math.min(1, this.scale));
  }

  reset(): void {
    this.sceneKey = null;
    this.scale = 1;
    this.suppressUntilBase = false;
  }
}

function isSentinelEncounter(snapshot: WorldSnapshotV3): boolean {
  return snapshot.worldId === "grey_hive" && snapshot.sceneId === "gh_sentinel_arena";
}

function hasSingleActiveSentinel(snapshot: WorldSnapshotV3): boolean {
  if (!Array.isArray(snapshot.actors)) return false;
  // A repeated id or type is ambiguous authority, even if only one row has the exact pair.
  const candidates = snapshot.actors.filter(actor => actor?.entityId === SENTINEL_ENTITY_ID || actor?.entityType === SENTINEL_ENTITY_TYPE);
  return candidates.length === 1 && candidates[0]?.entityId === SENTINEL_ENTITY_ID &&
    candidates[0]?.entityType === SENTINEL_ENTITY_TYPE && candidates[0]?.active === true;
}
