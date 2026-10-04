import type { SoundCueEvent, WorldSnapshotV3 } from "../protocol/types.js";

export const SOUND_CUE_DURATION_MS = 2400;
const ACOUSTIC_MAPPING_ID = "perception.acoustic_mapping_i";

export interface VisibleSoundCue {
  eventId: number;
  screenDirectionX: number;
  screenDirectionY: number;
  distanceLabel: string;
  sourceLabel: string;
}

function mappingAuthorized(snapshot: WorldSnapshotV3): boolean {
  if(snapshot.capabilities.acousticMappingAuthorized !== undefined) return snapshot.capabilities.acousticMappingAuthorized === true;
  const items = snapshot.capabilities?.items;
  if (!Array.isArray(items)) return false;
  const rows = items.filter(item => item?.capabilityId === ACOUSTIC_MAPPING_ID);
  return rows.length === 1 && rows[0]?.granted === true && rows[0]?.selected === true;
}

function matches(event: SoundCueEvent, snapshot: WorldSnapshotV3): boolean {
  return event.worldEpoch === snapshot.worldEpoch && event.worldId === snapshot.worldId &&
    event.sceneId === snapshot.sceneId;
}

function finiteProjection(event: SoundCueEvent): event is SoundCueEvent & { directionRad: number; distanceM: number } {
  return typeof event.directionRad === "number" && Number.isFinite(event.directionRad) &&
    typeof event.distanceM === "number" && Number.isFinite(event.distanceM) && event.distanceM >= 0;
}

function coarseDistance(distanceM: number): string {
  if (distanceM < 2) return "近处";
  if (distanceM < 7.5) return "约 5 米";
  return `约 ${Math.round(distanceM / 10) * 10} 米`;
}

/** Keeps only an authorized, short-lived projection. Later grants cannot reveal past cues. */
export class SoundCueModel {
  private active: { event: SoundCueEvent; expiresAtMs: number } | null = null;

  accept(events: readonly SoundCueEvent[], snapshot: WorldSnapshotV3, nowMs: number): void {
    if (!mappingAuthorized(snapshot) || !Number.isFinite(nowMs)) { this.active = null; return; }
    for (const event of events) {
      if (event.protocolVersion !== 1 || (event.kind !== "signal_ping" && event.kind !== "warden_true_call") ||
          event.kind === "warden_true_call" && (event.worldId !== "mist_harbor" || event.sceneId !== "mh_warden_arena") ||
          !matches(event, snapshot) || !finiteProjection(event)) continue;
      this.active = { event, expiresAtMs: nowMs + SOUND_CUE_DURATION_MS };
    }
  }

  view(snapshot: WorldSnapshotV3, nowMs: number): VisibleSoundCue | null {
    const active = this.active;
    if (!active || !Number.isFinite(nowMs) || nowMs >= active.expiresAtMs ||
        !mappingAuthorized(snapshot) || !matches(active.event, snapshot) || !finiteProjection(active.event)) {
      this.active = null;
      return null;
    }
    const dx = Math.sin(active.event.directionRad);
    const dz = Math.cos(active.event.directionRad);
    const screenX = dx - dz;
    const screenY = dx + dz;
    const length = Math.hypot(screenX, screenY);
    return {
      eventId: active.event.eventId,
      screenDirectionX: screenX / length,
      screenDirectionY: screenY / length,
      sourceLabel: active.event.kind === "warden_true_call" ? "真实声源" : "信号",
      distanceLabel: coarseDistance(active.event.distanceM),
    };
  }

  reset(): void { this.active = null; }
}
