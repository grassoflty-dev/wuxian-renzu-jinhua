import { isSentinelWarning, sentinelWarningMatches } from "../protocol/SentinelEncounter.js";
import { supportGeometryKey } from "../protocol/SupportProjection.js";
import { validPresentationEventEnvelope, isClockworksEnemyCue, clockworksEnemyActorForEvent, combatFeedbackMatchesSnapshot } from "../protocol/PresentationEventValidation.js";
import type { PresentationEvent, SoundCueEvent, Vec3, WorldSnapshotEnvelope } from "../protocol/types.js";
import { assertSnapshot, assertSnapshotV3 } from "../protocol/types.js";

function lerp(a: number, b: number, alpha: number): number { return a + (b - a) * alpha; }
function vecLerp(a: Vec3, b: Vec3, alpha: number): Vec3 {
  return { xM: lerp(a.xM, b.xM, alpha), yM: lerp(a.yM, b.yM, alpha), zM: lerp(a.zM, b.zM, alpha) };
}

// Bounded transport backlog; excess rows remain refetchable through the unchanged cursor.
export const MAX_DEFERRED_PRESENTATION_EVENTS = 256;

/** Holds two server states for rendering only. All gameplay values remain Rust-owned. */
export class SnapshotClient {
  private previous: WorldSnapshotEnvelope | null = null;
  private current: WorldSnapshotEnvelope | null = null;
  private lastEventId = 0;
  private lastSoundCueId = 0;
  private readonly deferredEvents = new Map<number, PresentationEvent>();
  private presentationContextRevision = 0;

  /** Returns false for ignored/duplicate receipts; callers must never present them. */
  accept(snapshot: WorldSnapshotEnvelope): boolean {
    if (snapshot.protocolVersion === 3) assertSnapshotV3(snapshot);
    else assertSnapshot(snapshot);
    if (this.current && isPlayerDead(this.current)) {
      // A dead session is terminal. Late/mixed-counter receipts are harmlessly
      // ignored; a genuinely newer revival/replacement is a protocol violation.
      if (snapshot.worldEpoch < this.current.worldEpoch) return false;
      if (snapshot.worldEpoch === this.current.worldEpoch &&
          (snapshot.serverTick < this.current.serverTick || snapshot.authorityRevision < this.current.authorityRevision ||
           snapshot.serverTick === this.current.serverTick && snapshot.authorityRevision === this.current.authorityRevision)) return false;
      if (!sameContext(this.current, snapshot)) throw new Error("E_SNAPSHOT_DEAD_SESSION_REPLACEMENT");
      if (!isPlayerDead(snapshot)) throw new Error("E_SNAPSHOT_DEAD_REVIVAL");
      // The fatal view is committed exactly once; even newer still-dead
      // receipts may not move the corpse, HUD, cursors, or saved context.
      return false;
    }
    if (this.current && snapshot.worldEpoch < this.current.worldEpoch) throw new Error("E_SNAPSHOT_STALE_EPOCH");
    if (this.current && snapshot.worldEpoch === this.current.worldEpoch) {
      if (sameContext(this.current, snapshot) && this.current.protocolVersion === 3 && snapshot.protocolVersion === 3) {
        const before = this.current.supportScene, after = snapshot.supportScene;
        if (!!before !== !!after || before && after &&
            (before.sceneSourceSha256 !== after.sceneSourceSha256 ||
             after.serverTimeMs < before.serverTimeMs || snapshot.serverTick === this.current.serverTick && supportGeometryKey(before) !== supportGeometryKey(after))) {
          throw new Error("E_SUPPORT_SNAPSHOT_CONTINUITY");
        }
      }
      if (snapshot.serverTick < this.current.serverTick || snapshot.authorityRevision < this.current.authorityRevision) {
        throw new Error("E_SNAPSHOT_REGRESSION");
      }
      if (snapshot.serverTick === this.current.serverTick && snapshot.authorityRevision === this.current.authorityRevision) return false;
      this.previous = this.current;
    } else {
      this.previous = null;
      this.lastEventId = 0;
      this.lastSoundCueId = 0;
    }
    if (!this.current || !sameContext(this.current, snapshot) || isPlayerDead(snapshot)) this.clearDeferredEvents();
    this.current = snapshot;
    if (isPlayerDead(snapshot)) this.previous = null;
    return true;
  }

  view(): WorldSnapshotEnvelope | null { return this.current; }

  interpolatedPlayer(alpha: number): Vec3 | null {
    if (!this.current) return null;
    // Support and rider poses must come from one authority sample. Interpolating
    // only the rider against current platform geometry invents a visible gap.
    if (this.current.protocolVersion === 3 && this.current.supportScene) return playerPosition(this.current);
    if (!this.previous) return playerPosition(this.current);
    const t = Math.min(1, Math.max(0, alpha));
    return vecLerp(playerPosition(this.previous), playerPosition(this.current), t);
  }

  interpolatedActor(entityId: string, alpha: number): Vec3 | null {
    const current = this.current && actors(this.current).find(actor => actor.entityId === entityId);
    if (!current) return null;
    const previous = this.previous && actors(this.previous).find(actor => actor.entityId === entityId);
    if (!previous || previous.entityType !== current.entityType) return current.transform.positionM;
    return vecLerp(previous.transform.positionM, current.transform.positionM, Math.min(1, Math.max(0, alpha)));
  }

  acceptEvents(events: PresentationEvent[]): PresentationEvent[] {
    const current = this.current;
    if (!current || !Array.isArray(events)) return [];
    const candidates = new Map(this.deferredEvents);
    for (const event of events) {
      if (!validPresentationEventEnvelope(event) || event.worldEpoch !== current.worldEpoch ||
          event.eventId <= this.lastEventId) continue;
      // Event IDs are immutable within an epoch. Keep the first valid row while deferred.
      if (!candidates.has(event.eventId)) candidates.set(event.eventId, { ...event, positionM: { ...event.positionM } });
    }
    this.deferredEvents.clear();
    const accepted: PresentationEvent[] = [];
    let blockedByFuture = false;
    for (const event of [...candidates.values()].sort((a, b) => a.eventId - b.eventId)) {
      if (event.eventId <= this.lastEventId) continue;
      if (event.serverTick <= current.serverTick && isClockworksEnemyCue(event.kind) &&
          !clockworksEnemyActorForEvent(event, current)) continue;
      if (event.serverTick <= current.serverTick && isSentinelWarning(event.kind) &&
          (event.kind === "SentinelChargeWindup" || (current.protocolVersion === 3 && current.sentinelEncounter)) &&
          !sentinelWarningMatches(event, current)) continue;
      if (event.serverTick <= current.serverTick && event.protocolVersion === 2 && event.combatFeedback &&
          !combatFeedbackMatchesSnapshot(event, current)) continue;
      if (event.serverTick > current.serverTick) blockedByFuture = true;
      if (blockedByFuture) {
        // Never consume a later ID over an earlier future row. Excess rows are
        // refetched using the unchanged cursor; the queue remains bounded.
        if (this.deferredEvents.size < MAX_DEFERRED_PRESENTATION_EVENTS) this.deferredEvents.set(event.eventId, event);
        continue;
      }
      accepted.push(event);
      this.lastEventId = event.eventId;
    }
    return accepted;
  }

  clearDeferredEvents(): void {
    this.deferredEvents.clear();
    this.presentationContextRevision++;
  }

  eventContextRevision(): number { return this.presentationContextRevision; }
  deferredEventCount(): number { return this.deferredEvents.size; }

  eventCursor(): number { return this.lastEventId; }

  acceptSoundCues(value: unknown): SoundCueEvent[] {
    const current = this.current;
    if (!current || !Array.isArray(value)) throw new Error("E_SOUND_CUE_PROTOCOL");
    // Validate the whole batch before advancing the cursor.
    for (const item of value) {
      if (!validSoundCue(item)) throw new Error("E_SOUND_CUE_PROTOCOL");
    }
    if (current.protocolVersion !== 3) return [];
    const accepted: SoundCueEvent[] = [];
    for (const event of value as SoundCueEvent[]) {
      if (event.worldEpoch !== current.worldEpoch || event.eventId <= this.lastSoundCueId) continue;
      this.lastSoundCueId = event.eventId;
      if (event.worldId === current.worldId && event.sceneId === current.sceneId) accepted.push(event);
    }
    return accepted;
  }

  soundCueCursor(): number { return this.lastSoundCueId; }
}

function validSoundCue(value: unknown): value is SoundCueEvent {
  if (!value || typeof value !== "object") return false;
  const event = value as Record<string, unknown>;
  const counter = (number: unknown) => Number.isSafeInteger(number) && (number as number) >= 0;
  const optionalFinite = (number: unknown) => number === null || number === undefined ||
    (typeof number === "number" && Number.isFinite(number));
  return event.protocolVersion === 1 && counter(event.eventId) && (event.eventId as number) > 0 &&
    counter(event.worldEpoch) && counter(event.serverTick) &&
    typeof event.worldId === "string" && event.worldId.length > 0 &&
    typeof event.sceneId === "string" && event.sceneId.length > 0 &&
    (event.kind === "signal_ping" || event.kind === "warden_true_call" && event.worldId === "mist_harbor" && event.sceneId === "mh_warden_arena") && optionalFinite(event.directionRad) && optionalFinite(event.distanceM) &&
    (event.distanceM === null || event.distanceM === undefined || (event.distanceM as number) >= 0);
}

function playerPosition(snapshot: WorldSnapshotEnvelope): Vec3 {
  return snapshot.protocolVersion === 3
    ? snapshot.player.transform.positionM
    : snapshot.view.player.transform.positionM;
}
function actors(snapshot: WorldSnapshotEnvelope) {
  return snapshot.protocolVersion === 3 ? snapshot.actors : snapshot.view.actors;
}

/** Rust health is authoritative. A fatal view cannot be revived within this client. */
export function isPlayerDead(snapshot: WorldSnapshotEnvelope): boolean {
  const hp = snapshot.protocolVersion === 3 ? snapshot.player.currentHp : snapshot.view.player.currentHp;
  return Number.isFinite(hp) && hp <= 0;
}
function sameContext(left: WorldSnapshotEnvelope, right: WorldSnapshotEnvelope): boolean {
  return left.worldEpoch === right.worldEpoch && left.worldId === right.worldId && left.protocolVersion === right.protocolVersion &&
    (left.protocolVersion !== 3 || right.protocolVersion !== 3 || left.sceneId === right.sceneId);
}
