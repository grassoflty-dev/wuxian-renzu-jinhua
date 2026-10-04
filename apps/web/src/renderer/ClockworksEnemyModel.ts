import type { PresentationEvent, Vec3, WorldSnapshotEnvelope, WorldSnapshotV3 } from "../protocol/types.js";

// Historical export names remain stable; the same ordinary model also serves the Gate B Brute.
export type ClockworksEnemyType = "enemy.clockworks.pressure_drone" | "enemy.clockworks.furnace_hound" | "enemy.grey_hive.brute" | "enemy.grey_hive.swarm" | "enemy.mist_harbor.tidebound" | "enemy.mist_harbor.signal_wraith";
import { clockworksEnemyActorForEvent as clockworksEnemyForEvent, validClockworksEnemyEventEnvelope as validEvent, presentationCounter as counter,
  type ClockworksEnemyEvent as EnemyEvent,
  type ClockworksEnemyCueKind, type EnemyAttackKind } from "../protocol/PresentationEventValidation.js";
export { clockworksEnemyActorForEvent as clockworksEnemyForEvent, isClockworksEnemyCue, isClockworksEnemyMotion } from "../protocol/PresentationEventValidation.js";
export type { ClockworksEnemyCueKind, EnemyAttackKind } from "../protocol/PresentationEventValidation.js";
export interface EnemyPlaceholderStyle {
  readonly temporary_visual: true;
  readonly assetId: string;
  readonly widthPx: number;
  readonly heightPx: number;
  readonly liftPx: number;
  readonly tint: number;
  readonly silhouette: "rotor" | "quadruped" | "heavy" | "cluster" | "tidal" | "signal";
}

// Reuse admitted textures only. These transformations/identity outlines are temporary,
// not authored Drone/Hound animation sets or collision/body dimensions.
export const CLOCKWORKS_ENEMY_PLACEHOLDERS: Readonly<Record<ClockworksEnemyType, EnemyPlaceholderStyle>> = Object.freeze({
  "enemy.clockworks.pressure_drone": Object.freeze({ temporary_visual: true, assetId: "runtime2d.enemy.mistharbor.signal_wraith.v2",
    widthPx: 32, heightPx: 32, liftPx: 16, tint: 0x9ddde5, silhouette: "rotor" }),
  "enemy.clockworks.furnace_hound": Object.freeze({ temporary_visual: true, assetId: "runtime2d.enemy.clockworks.forged_guard.v2",
    widthPx: 62, heightPx: 30, liftPx: 0, tint: 0xffae73, silhouette: "quadruped" }),
  "enemy.grey_hive.brute": Object.freeze({ temporary_visual: true, assetId: "runtime2d.enemy.infected_security.v1",
    widthPx: 64, heightPx: 86, liftPx: 0, tint: 0xb2ac99, silhouette: "heavy" }),
  "enemy.grey_hive.swarm": Object.freeze({ temporary_visual: true, assetId: "runtime2d.enemy.infected_maintenance_worker.v1",
    widthPx: 76, heightPx: 24, liftPx: 0, tint: 0xb9d897, silhouette: "cluster" }),
  "enemy.mist_harbor.signal_wraith": Object.freeze({ temporary_visual: true, assetId: "runtime2d.enemy.mistharbor.signal_wraith.v2",
    widthPx: 36, heightPx: 50, liftPx: 8, tint: 0xc1b6ed, silhouette: "signal" }),
  "enemy.mist_harbor.tidebound": Object.freeze({ temporary_visual: true, assetId: "runtime2d.enemy.mistharbor.drowned.v2",
    widthPx: 56, heightPx: 70, liftPx: 0, tint: 0x82c8bd, silhouette: "tidal" }),
});
const MOTION = new Set<string>(["SignalShotMotion", "PressureShotMotion", "FurnaceHoundLeapMotion", "EnemyChargeMotion", "EnemyLungeMotion"]);
const ATTACK = new Set<string>(["EnemyAttackTelegraph", "EnemyAttackImpact", ...MOTION]);
const TICK_MS = 1000 / 60; // Transport clock only; combat timing comes from durationMs.

export interface ClockworksEnemyFrame {
  temporary_visual: true;
  eventId: number;
  actorId: string;
  memberId?: string;
  halfAngleRad?: number;
  entityType: ClockworksEnemyType;
  kind: ClockworksEnemyCueKind;
  attackKind?: EnemyAttackKind;
  positionM: Vec3;
  actorPositionM: Vec3;
  directionRad: number;
  radiusM: number;
  rangeM: number;
  intensity: number;
  remainingMs: number;
  progress: number;
  reducedMotion: boolean;
}

export function enemyPlaceholderStyle(entityType: string): EnemyPlaceholderStyle | null {
  return Object.hasOwn(CLOCKWORKS_ENEMY_PLACEHOLDERS, entityType)
    ? CLOCKWORKS_ENEMY_PLACEHOLDERS[entityType as ClockworksEnemyType] : null;
}

/** One latest cue of each kind per actor; never extrapolates projectile or leap positions. */
export class ClockworksEnemyModel {
  private context: string | null = null;
  private lastEventId = 0;
  private lastTick = 0;
  private readonly active = new Map<string, EnemyEvent & { boundType: string }>();

  accept(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    if (!this.sync(snapshot) || !Array.isArray(events)) return [];
    const accepted: number[] = [];
    for (const event of events) {
      if (!event || event.protocolVersion !== 2 || !counter(event.eventId) || event.eventId === 0 ||
          event.worldEpoch !== snapshot.worldEpoch || event.eventId <= this.lastEventId) continue;
      const actor = clockworksEnemyForEvent(event, snapshot);
      if (!actor || !validEvent(event)) continue;
      // A same-tick terminal receipt wins over a stale trailing motion row even
      // when transport order gives that row a larger event ID. A fresh later
      // attack (or restored epoch) can still begin at its authoritative point.
      if (MOTION.has(event.kind) && [...this.active.values()].some(prior =>
        prior.actorId === event.actorId && ["EnemyAttackImpact", "EnemyStagger", "EnemyDeath"].includes(prior.kind) &&
        prior.serverTick >= event.serverTick)) continue;
      this.lastEventId = event.eventId;
      for (const [key, prior] of this.active) {
        if (prior.actorId !== event.actorId) continue;
        if (event.kind === "EnemyDeath" || event.kind === "EnemyStagger") {
          // One group transaction publishes distinct member feedback before its
          // aggregate terminal cue. Keep that validated same-tick feedback.
          if (!(prior.memberId !== undefined && prior.serverTick === event.serverTick)) this.active.delete(key);
        } else if (ATTACK.has(event.kind) && ATTACK.has(prior.kind)) this.active.delete(key);
      }
      // A motion row replaces a warning only after the server has committed motion.
      this.active.set(`${event.actorId}\0${event.memberId ?? ""}\0${event.kind}`, { ...event, positionM: { ...event.positionM }, boundType: actor.entityType });
      accepted.push(event.eventId);
    }
    this.prune(snapshot);
    const surviving = new Set([...this.active.values()].map(event => event.eventId));
    return accepted.filter(id => surviving.has(id));
  }

  project(snapshot: WorldSnapshotEnvelope, reducedMotion: boolean): ClockworksEnemyFrame[] {
    if (!this.sync(snapshot)) return [];
    this.prune(snapshot);
    return [...this.active.values()].flatMap(event => {
      const actor = clockworksEnemyForEvent(event, snapshot);
      if (!actor || actor.entityType !== event.boundType) return [];
      const remaining = remainingMs(event, snapshot.serverTick);
      return [{ temporary_visual: true as const, eventId: event.eventId, actorId: event.actorId,
        ...(event.memberId ? { memberId: event.memberId } : {}),
        ...(event.halfAngleRad !== undefined ? { halfAngleRad: event.halfAngleRad } : {}),
        entityType: actor.entityType as ClockworksEnemyType, kind: event.kind,
        ...(event.attackKind ? { attackKind: event.attackKind } : {}), positionM: { ...event.positionM },
        actorPositionM: { ...actor.transform.positionM }, directionRad: event.directionRad, radiusM: event.radiusM, rangeM: event.rangeM ?? 0,
        intensity: event.intensity, remainingMs: remaining, progress: 1 - remaining / event.durationMs, reducedMotion }];
    });
  }

  reset(): void { this.context = null; this.lastEventId = 0; this.lastTick = 0; this.active.clear(); }

  private sync(snapshot: WorldSnapshotEnvelope): snapshot is WorldSnapshotV3 {
    if (!validContext(snapshot)) { this.reset(); return false; }
    const key = `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
    if (key !== this.context) { this.reset(); this.context = key; }
    if (snapshot.serverTick < this.lastTick) { this.active.clear(); return false; }
    this.lastTick = snapshot.serverTick;
    return true;
  }

  private prune(snapshot: WorldSnapshotEnvelope): void {
    for (const [key, event] of this.active) {
      const actor = clockworksEnemyForEvent(event, snapshot);
      if (!actor || actor.entityType !== event.boundType) this.active.delete(key);
    }
  }
}

function validContext(snapshot: WorldSnapshotEnvelope): snapshot is WorldSnapshotV3 {
  return !!snapshot && snapshot.protocolVersion === 3 &&
    ((snapshot.worldId === "clockworks" && typeof snapshot.sceneId === "string" && snapshot.sceneId.startsWith("cw_")) ||
      (snapshot.worldId === "grey_hive" && ["gh_gate_b", "gh_lockdown", "gh_deep_decon"].includes(snapshot.sceneId)) ||
      (snapshot.worldId === "mist_harbor" && ["mh_drowned_quay", "mh_tidal_warehouse", "mh_signal_yard", "mh_breakwater", "mh_resonance_tower"].includes(snapshot.sceneId))) &&
    counter(snapshot.worldEpoch) && counter(snapshot.serverTick) && Array.isArray(snapshot.actors);
}
function remainingMs(event: EnemyEvent, tick: number): number { return event.durationMs - (tick - event.serverTick) * TICK_MS; }
/** Ground boundary follows public attack geometry: radial Bite, aimed Shot/Leap. */
export function enemyWarningGroundPoints(frame: Pick<ClockworksEnemyFrame, "positionM" | "directionRad" | "rangeM" | "radiusM" | "attackKind" | "halfAngleRad">): Vec3[] {
  const { positionM: p, directionRad: yaw, rangeM: reach, radiusM: radius } = frame;
  // Bite authority checks center distance <= attack range plus LOS, with no body-radius addition.
  // Segment count is visual tessellation only; no client combat radius is introduced.
  if (frame.attackKind === "bite" || frame.attackKind === "slam") return Array.from({ length: 64 }, (_, index) => {
    const angle = index * Math.PI * 2 / 64;
    return { xM: p.xM + Math.sin(angle) * reach, yM: p.yM, zM: p.zM + Math.cos(angle) * reach };
  });
  if (frame.attackKind === "tide_swing") {
    const halfAngle = frame.halfAngleRad;
    if (halfAngle === undefined || !Number.isFinite(halfAngle) || halfAngle <= 0 || halfAngle > Math.PI) return [];
    return [{ ...p }, ...Array.from({ length: 33 }, (_, i) => {
      const angle = yaw - halfAngle + 2 * halfAngle * i / 32;
      return { xM: p.xM + Math.sin(angle) * reach, yM: p.yM, zM: p.zM + Math.cos(angle) * reach };
    })];
  }
  const dx = Math.sin(yaw), dz = Math.cos(yaw), px = dz * radius, pz = -dx * radius;
  if (frame.attackKind === "charge" || frame.attackKind === "lunge" || frame.attackKind === "tide_charge") {
    // Grounded charge sweeps a disc, including both endpoint caps. This is
    // committed warning geometry, never local motion or hit simulation.
    const start = Array.from({ length: 33 }, (_, i) => {
      const angle = i * Math.PI / 32;
      return { xM: p.xM + px * Math.cos(angle) - dx * radius * Math.sin(angle), yM: p.yM,
        zM: p.zM + pz * Math.cos(angle) - dz * radius * Math.sin(angle) };
    });
    const end = Array.from({ length: 33 }, (_, i) => {
      const angle = i * Math.PI / 32;
      return { xM: p.xM + dx * reach - px * Math.cos(angle) + dx * radius * Math.sin(angle), yM: p.yM,
        zM: p.zM + dz * reach - pz * Math.cos(angle) + dz * radius * Math.sin(angle) };
    });
    return [...start, ...end];
  }
  return [
    { xM: p.xM + px, yM: p.yM, zM: p.zM + pz },
    { xM: p.xM + dx * reach + px, yM: p.yM, zM: p.zM + dz * reach + pz },
    { xM: p.xM + dx * reach - px, yM: p.yM, zM: p.zM + dz * reach - pz },
    { xM: p.xM - px, yM: p.yM, zM: p.zM - pz },
  ];
}
