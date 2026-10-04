import { isSentinelWarning, sentinelWarningMatches } from "../protocol/SentinelEncounter.js";
import type { PresentationEvent, Vec3, WorldSnapshotEnvelope, WorldSnapshotV3 } from "../protocol/types.js";

export type SentinelTelegraphKind = "SentinelAttackWindup" | "SentinelHeavyWindup" | "SentinelChargeWindup" | "SentinelDeath";

export interface SentinelTelegraphFrame {
  eventId: number;
  kind: SentinelTelegraphKind;
  positionM: Vec3;
  ageTicks: number;
  progress: number;
  reducedMotion: boolean;
}

export const SENTINEL_LIGHT_WINDUP_TICKS = 15;
export const SENTINEL_HEAVY_WINDUP_TICKS = 24;
export const SENTINEL_DEATH_VISUAL_TICKS = 60;

const SENTINEL_ID = "gh_sentinel_arena_sentinel_01";
const SENTINEL_TYPE = "enemy.grey_hive.sentinel";
const WINDUP_KINDS = new Set<string>(["SentinelAttackWindup", "SentinelHeavyWindup", "SentinelChargeWindup"]);
const TELEGRAPH_KINDS = new Set<string>([...WINDUP_KINDS, "SentinelDeath"]);

/** Consumes only the registered Sentinel's short-lived, Rust-authored presentation events. */
export class SentinelTelegraphModel {
  private identityKey: string | null = null;
  private lastEventId = 0;
  private readonly activeEvents: PresentationEvent[] = [];

  accept(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    const current = this.syncSnapshot(snapshot);
    if (!current || !Array.isArray(events)) return [];
    const sentinel = uniqueSentinel(current);
    const acceptedWarningIds: number[] = [];
    for (const event of events) {
      if (!validEventEnvelope(event) || event.worldEpoch !== current.worldEpoch || event.eventId <= this.lastEventId) continue;
      if (isSentinelWarning(event.kind) && (current.sentinelEncounter || event.kind === "SentinelChargeWindup")) {
        if (!sentinelWarningMatches(event, current)) continue;
        this.lastEventId = event.eventId;
        acceptedWarningIds.push(event.eventId);
        continue;
      }
      if (event.kind === "SentinelDeath" && current.sentinelEncounter && current.sentinelEncounter.phase !== "dead") continue;
      this.lastEventId = event.eventId;
      if (!TELEGRAPH_KINDS.has(event.kind)) continue;
      const dying = event.kind === "SentinelDeath";
      if (!sentinel || (dying ? sentinel.active !== false : sentinel.active !== true)) continue;
      if (!isFinitePosition(sentinel.transform?.positionM) || !isFinitePosition(event.positionM)) continue;
      const ageTicks = current.serverTick - event.serverTick;
      if (ageTicks < 0 || ageTicks >= lifetimeTicks(event.kind as SentinelTelegraphKind)) continue;
      if (dying) this.activeEvents.length = 0;
      this.activeEvents.push(event);
      if (WINDUP_KINDS.has(event.kind)) acceptedWarningIds.push(event.eventId);
    }
    this.expire(current.serverTick);
    return acceptedWarningIds;
  }

  project(snapshot: WorldSnapshotEnvelope, reducedMotion: boolean): SentinelTelegraphFrame[] {
    const current = this.syncSnapshot(snapshot);
    if (!current) return [];
    this.expire(current.serverTick);
    const sentinel = uniqueSentinel(current);
    if (!sentinel) {
      this.activeEvents.length = 0;
      return [];
    }
    for (let index = this.activeEvents.length - 1; index >= 0; index--) {
      const event = this.activeEvents[index];
      if (!event || (event.kind === "SentinelDeath" ? sentinel.active !== false ||
          (current.sentinelEncounter !== undefined && current.sentinelEncounter.phase !== "dead") : sentinel.active !== true)) {
        this.activeEvents.splice(index, 1);
      }
    }
    if (current.sentinelEncounter) {
      for (let i = this.activeEvents.length - 1; i >= 0; i--) {
        if (this.activeEvents[i]?.kind !== "SentinelDeath") this.activeEvents.splice(i, 1);
      }
    }
    return this.activeEvents.filter(event => event.kind === "SentinelDeath"
      ? sentinel?.active === false : sentinel?.active === true).map(event => {
      const ageTicks = current.serverTick - event.serverTick;
      const lifetime = lifetimeTicks(event.kind as SentinelTelegraphKind);
      return { eventId: event.eventId, kind: event.kind as SentinelTelegraphKind,
        positionM: event.positionM, ageTicks, progress: Math.max(0, Math.min(1, ageTicks / lifetime)), reducedMotion };
    });
  }

  reset(): void {
    this.identityKey = null;
    this.lastEventId = 0;
    this.activeEvents.length = 0;
  }

  private syncSnapshot(snapshot: WorldSnapshotEnvelope): WorldSnapshotV3 | null {
    if (snapshot.protocolVersion !== 3 || !isFiniteCounter(snapshot.worldEpoch) || !isFiniteCounter(snapshot.serverTick)) {
      this.reset();
      return null;
    }
    const key = `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
    if (key !== this.identityKey) {
      this.identityKey = key;
      this.lastEventId = 0;
      this.activeEvents.length = 0;
    }
    if (snapshot.worldId !== "grey_hive" || snapshot.sceneId !== "gh_sentinel_arena") {
      this.activeEvents.length = 0;
      return null;
    }
    return snapshot;
  }

  private expire(serverTick: number): void {
    for (let index = this.activeEvents.length - 1; index >= 0; index--) {
      const event = this.activeEvents[index];
      if (!event || serverTick - event.serverTick >= lifetimeTicks(event.kind as SentinelTelegraphKind)) {
        this.activeEvents.splice(index, 1);
      }
    }
  }
}

function lifetimeTicks(kind: SentinelTelegraphKind): number {
  if (kind === "SentinelAttackWindup") return SENTINEL_LIGHT_WINDUP_TICKS;
  if (kind === "SentinelHeavyWindup") return SENTINEL_HEAVY_WINDUP_TICKS;
  return SENTINEL_DEATH_VISUAL_TICKS;
}

function uniqueSentinel(snapshot: WorldSnapshotV3) {
  if (!Array.isArray(snapshot.actors)) return null;
  const candidates = snapshot.actors.filter(actor => actor?.entityId === SENTINEL_ID || actor?.entityType === SENTINEL_TYPE);
  if (candidates.length !== 1) return null;
  const actor = candidates[0];
  return actor?.entityId === SENTINEL_ID && actor.entityType === SENTINEL_TYPE ? actor : null;
}

function validEventEnvelope(event: PresentationEvent): event is Extract<PresentationEvent, { protocolVersion: 2 }> {
  if (!event || event.protocolVersion !== 2 || !isFiniteCounter(event.eventId) || event.eventId === 0 ||
      !isFiniteCounter(event.worldEpoch) || !isFiniteCounter(event.serverTick) || typeof event.kind !== "string" ||
      !isFinitePosition(event.positionM) || !Number.isFinite(event.directionRad) ||
      !Number.isFinite(event.radiusM) || event.radiusM <= 0 || event.radiusM > 8 ||
      !Number.isFinite(event.intensity) || event.intensity < 0 || event.intensity > 2) return false;
  return true;
}

function isFiniteCounter(value: unknown): value is number {
  return Number.isSafeInteger(value) && (value as number) >= 0;
}

function isFinitePosition(value: unknown): value is Vec3 {
  if (!value || typeof value !== "object") return false;
  const point = value as Partial<Vec3>;
  return [point.xM, point.yM, point.zM].every(coordinate => Number.isFinite(coordinate) && Math.abs(coordinate as number) <= 10_000);
}
