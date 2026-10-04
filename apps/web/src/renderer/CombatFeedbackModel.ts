import type { CombatFeedback, PresentationEvent, Vec3, WorldSnapshotEnvelope } from "../protocol/types.js";
import { combatFeedbackMatchesSnapshot, validPresentationEventEnvelope, type CombatFeedbackEvent } from "../protocol/PresentationEventValidation.js";

export const MAX_COMBAT_FEEDBACK = 64;
export interface CombatFeedbackFrame {
  eventId: number;
  kind: string;
  outcome: CombatFeedback["outcome"];
  positionM: Vec3;
  targetId?: string;
  progress: number;
  remainingMs: number;
  reducedMotion: boolean;
}

/** Bounded contact presentation. No input prediction, HP deltas or target guessing. */
export class CombatFeedbackModel {
  private context = "";
  private lastTick = 0;
  private lastEventId = 0;
  private readonly active = new Map<number, CombatFeedbackEvent>();
  private readonly clocks = new Map<number, { acceptedAt: number; ageMs: number }>();

  accept(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope, nowMs = performance.now()): number[] {
    if (!Number.isFinite(nowMs) || !this.sync(snapshot)) return [];
    const accepted: number[] = [];
    for (const event of [...events].sort((a, b) => (a?.eventId ?? 0) - (b?.eventId ?? 0))) {
      if (!validPresentationEventEnvelope(event) || !combatFeedbackMatchesSnapshot(event, snapshot) || event.eventId <= this.lastEventId) continue;
      this.lastEventId = event.eventId;
      this.active.set(event.eventId, { ...event, positionM: { ...event.positionM }, combatFeedback: { ...event.combatFeedback } });
      this.clocks.set(event.eventId, { acceptedAt: nowMs, ageMs: (snapshot.serverTick - event.serverTick) * 1000 / 60 });
      accepted.push(event.eventId);
      if (this.active.size > MAX_COMBAT_FEEDBACK) this.remove(this.active.keys().next().value!);
    }
    this.prune(snapshot, nowMs);
    return accepted;
  }

  project(snapshot: WorldSnapshotEnvelope, reducedMotion: boolean, nowMs = performance.now()): CombatFeedbackFrame[] {
    if (!Number.isFinite(nowMs) || !this.sync(snapshot)) return [];
    this.prune(snapshot, nowMs);
    return [...this.active.values()].map(event => {
      const elapsed = this.elapsed(event, snapshot, nowMs);
      return { eventId: event.eventId, kind: event.kind, outcome: event.combatFeedback.outcome,
        positionM: { ...event.positionM }, ...(event.combatFeedback.targetId ? { targetId: event.combatFeedback.targetId } : {}),
        remainingMs: event.durationMs - elapsed, progress: elapsed / event.durationMs, reducedMotion };
    });
  }

  reset(): void { this.context = ""; this.lastTick = 0; this.lastEventId = 0; this.active.clear(); this.clocks.clear(); }

  private sync(snapshot: WorldSnapshotEnvelope): boolean {
    if (snapshot.protocolVersion !== 3 || snapshot.player.currentHp <= 0) { this.reset(); return false; }
    const context = `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
    if (context !== this.context) { this.reset(); this.context = context; }
    if (snapshot.serverTick < this.lastTick) { this.active.clear(); this.clocks.clear(); return false; }
    this.lastTick = snapshot.serverTick;
    return true;
  }

  private elapsed(event: CombatFeedbackEvent, snapshot: WorldSnapshotEnvelope, nowMs: number): number {
    const clock = this.clocks.get(event.eventId)!;
    return Math.max((snapshot.serverTick - event.serverTick) * 1000 / 60, clock.ageMs + Math.max(0, nowMs - clock.acceptedAt));
  }
  private remove(id: number): void { this.active.delete(id); this.clocks.delete(id); }
  private prune(snapshot: WorldSnapshotEnvelope, nowMs: number): void {
    for (const [id, event] of this.active) if (!combatFeedbackMatchesSnapshot(event, snapshot) || this.elapsed(event, snapshot, nowMs) >= event.durationMs) this.remove(id);
  }
}

export function combatRejectionText(events: readonly PresentationEvent[], admittedIds: readonly number[]): string | null {
  const ids = new Set(admittedIds);
  const event = [...events].reverse().find(event => event.protocolVersion === 2 && ids.has(event.eventId) && event.combatFeedback?.outcome === "rejected");
  if (!event || event.protocolVersion !== 2) return null;
  const reason = event.combatFeedback!.reason;
  return ({ cooldown: "技能仍在冷却", insufficient_energy: "能量不足", action_busy: "当前动作尚未结束",
    guard_not_active: "防御已结束", dash_rejected: "当前无法冲刺", no_target: "未命中目标" } as Record<string, string>)[reason ?? ""] ?? "当前无法执行该动作";
}
