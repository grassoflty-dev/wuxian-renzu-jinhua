import type { PresentationEvent, Vec3, WorldSnapshotEnvelope, WorldSnapshotV3 } from "../protocol/types.js";

export type ClockworksPressureWaveKind = "ForgedGuardPressureWindup" | "ForgedGuardPressureImpact" |
  "PrimeRegulatorPressureWindup" | "PrimeRegulatorPressureImpact";
export interface ClockworksPressureWaveFrame {
  eventId: number;
  kind: ClockworksPressureWaveKind;
  originM: Vec3;
  radiusM: number;
  ageTicks: number;
  progress: number;
  reducedMotion: boolean;
}

const ELITE_ID = "cw_forged_guard_elite";
const ELITE_TYPE = "runtime2d.enemy.clockworks.forged_guard.v2";
const REGULATOR_ID = "cw_prime_regulator";
const REGULATOR_TYPE = "runtime2d.enemy.clockworks.forged_guard.v2";
const WINDUP_TICKS = 54;
const REGULATOR_WINDUP_TICKS = 84;
const IMPACT_TICKS = 12;
const ALLOWED_KINDS = new Set<string>([
  "ForgedGuardPressureWindup", "ForgedGuardPressureImpact",
  "PrimeRegulatorPressureWindup", "PrimeRegulatorPressureImpact",
]);

export interface PrimeRegulatorBossMarkFrame {
  entityId: typeof REGULATOR_ID;
  entityType: typeof REGULATOR_TYPE;
  positionM: Vec3;
  worldEpoch: number;
  reducedMotion: boolean;
}

/** A static temporary identity mark, authorized only by the current unique live core Boss snapshot. */
export function projectPrimeRegulatorBossMark(
  snapshot: WorldSnapshotEnvelope,
  reducedMotion: boolean,
): PrimeRegulatorBossMarkFrame | null {
  if (snapshot.protocolVersion !== 3 || snapshot.worldId !== "clockworks" ||
    snapshot.sceneId !== "cw_regulator_core" || !counter(snapshot.worldEpoch) || !counter(snapshot.serverTick)) {
    return null;
  }
  const actor = uniqueRegulator(snapshot);
  const positionM = actor?.transform?.positionM;
  if (!actor?.active || !finitePosition(positionM)) return null;
  return {
    entityId: REGULATOR_ID,
    entityType: REGULATOR_TYPE,
    positionM,
    worldEpoch: snapshot.worldEpoch,
    reducedMotion,
  };
}

/** Projects only epoch-scoped Rust events authorized by a unique live Clockworks boss. */
export class ClockworksPressureWaveModel {
  private identityKey: string | null = null;
  private lastEventId = 0;
  private readonly events: PresentationEvent[] = [];

  accept(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    const current = this.syncSnapshot(snapshot);
    if (!current || !Array.isArray(events)) return [];
    const actor = uniqueBoss(current);
    if (!actor?.active) return [];
    const accepted: number[] = [];
    for (const event of events) {
      if (!validEvent(event) || event.worldEpoch !== current.worldEpoch || event.eventId <= this.lastEventId) continue;
      this.lastEventId = event.eventId;
      if (!ALLOWED_KINDS.has(event.kind)) continue;
      if (!kindMatchesScene(event.kind, current.sceneId)) continue;
      const age = current.serverTick - event.serverTick;
      if (age < 0 || age >= lifetime(event.kind as ClockworksPressureWaveKind)) continue;
      this.events.push(event);
      accepted.push(event.eventId);
    }
    this.expire(current.serverTick);
    return accepted;
  }

  project(snapshot: WorldSnapshotEnvelope, reducedMotion: boolean): ClockworksPressureWaveFrame[] {
    const current = this.syncSnapshot(snapshot);
    if (!current) return [];
    this.expire(current.serverTick);
    if (!uniqueBoss(current)?.active) {
      this.events.length = 0;
      return [];
    }
    return this.events.map(event => {
      const ageTicks = current.serverTick - event.serverTick;
      const maxAge = lifetime(event.kind as ClockworksPressureWaveKind);
      return {
        eventId: event.eventId,
        kind: event.kind as ClockworksPressureWaveKind,
        originM: event.positionM,
        radiusM: event.radiusM,
        ageTicks,
        progress: Math.max(0, Math.min(1, ageTicks / maxAge)),
        reducedMotion,
      };
    });
  }

  reset(): void {
    this.identityKey = null;
    this.lastEventId = 0;
    this.events.length = 0;
  }

  private syncSnapshot(snapshot: WorldSnapshotEnvelope): WorldSnapshotV3 | null {
    if (snapshot.protocolVersion !== 3 || !counter(snapshot.worldEpoch) || !counter(snapshot.serverTick)) {
      this.reset();
      return null;
    }
    const key = `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
    if (key !== this.identityKey) {
      this.identityKey = key;
      this.lastEventId = 0;
      this.events.length = 0;
    }
    if (snapshot.worldId !== "clockworks" ||
      !["cw_forged_guard_arena", "cw_regulator_core"].includes(snapshot.sceneId)) {
      this.events.length = 0;
      return null;
    }
    return snapshot;
  }

  private expire(serverTick: number): void {
    for (let index = this.events.length - 1; index >= 0; index--) {
      const event = this.events[index];
      if (!event || serverTick - event.serverTick >= lifetime(event.kind as ClockworksPressureWaveKind)) this.events.splice(index, 1);
    }
  }
}

function lifetime(kind: ClockworksPressureWaveKind): number {
  if (kind === "PrimeRegulatorPressureWindup") return REGULATOR_WINDUP_TICKS;
  return kind.endsWith("Windup") ? WINDUP_TICKS : IMPACT_TICKS;
}
function validEvent(event: PresentationEvent): event is Extract<PresentationEvent, { protocolVersion: 2 }> {
  return !!event && event.protocolVersion === 2 && counter(event.eventId) && event.eventId > 0 &&
    counter(event.worldEpoch) && counter(event.serverTick) && ALLOWED_KINDS.has(event.kind) &&
    finitePosition(event.positionM) && Number.isFinite(event.radiusM) && event.radiusM > 0 && event.radiusM <= 8 &&
    Number.isFinite(event.intensity) && event.intensity >= 0 && event.intensity <= 2;
}
function kindMatchesScene(kind: string, sceneId: string): boolean {
  return sceneId === "cw_forged_guard_arena" ? kind.startsWith("ForgedGuard") :
    sceneId === "cw_regulator_core" && kind.startsWith("PrimeRegulator");
}
function uniqueBoss(snapshot: WorldSnapshotV3) {
  if (!Array.isArray(snapshot.actors)) return null;
  const isArena = snapshot.sceneId === "cw_forged_guard_arena";
  const id = isArena ? ELITE_ID : REGULATOR_ID;
  const type = isArena ? ELITE_TYPE : REGULATOR_TYPE;
  const candidates = snapshot.actors.filter(actor => actor?.entityId === id || actor?.entityType === type);
  if (candidates.length !== 1) return null;
  const actor = candidates[0];
  return actor?.entityId === id && actor.entityType === type ? actor : null;
}
function uniqueRegulator(snapshot: WorldSnapshotV3) {
  if (!Array.isArray(snapshot.actors)) return null;
  const candidates = snapshot.actors.filter(actor =>
    actor?.entityId === REGULATOR_ID || actor?.entityType === REGULATOR_TYPE);
  if (candidates.length !== 1) return null;
  const actor = candidates[0];
  return actor?.entityId === REGULATOR_ID && actor.entityType === REGULATOR_TYPE ? actor : null;
}
function counter(value: unknown): value is number { return Number.isSafeInteger(value) && (value as number) >= 0; }
function finitePosition(value: unknown): value is Vec3 {
  if (!value || typeof value !== "object") return false;
  const point = value as Partial<Vec3>;
  return [point.xM, point.yM, point.zM].every(axis => Number.isFinite(axis) && Math.abs(axis as number) <= 10_000);
}
