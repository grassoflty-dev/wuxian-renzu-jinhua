import { validSentinelChargeEnvelope } from "./SentinelEncounter.js";
import type { PresentationEvent, PresentationEventV2, Vec3, ActorView, ActorMemberView, WorldSnapshotEnvelope, SignalPositionProjection } from "./types.js";

export type ClockworksEnemyCueKind = "EnemyAlert" | "EnemyAttackTelegraph" | "EnemyStagger" | "EnemyDeath" |
  "PressureShotMotion" | "FurnaceHoundLeapMotion" | "EnemyChargeMotion" | "EnemyLungeMotion" | "SignalShotMotion" | "SignalBlink" | "EnemyMemberHit" | "EnemyMemberDisperse" | "EnemyAttackImpact";
export type EnemyAttackKind = "pressure_shot" | "bite" | "leap" | "charge" | "slam" | "lunge" | "tide_swing" | "tide_charge" | "signal_shot";
export type ClockworksEnemyEvent = PresentationEventV2 & { kind: ClockworksEnemyCueKind; actorId: string; durationMs: number };
const KINDS = new Set<string>(["EnemyAlert", "EnemyAttackTelegraph", "EnemyStagger", "EnemyDeath",
  "PressureShotMotion", "FurnaceHoundLeapMotion", "EnemyChargeMotion", "EnemyLungeMotion", "SignalShotMotion", "SignalBlink", "EnemyMemberHit", "EnemyMemberDisperse", "EnemyAttackImpact"]);
const MOTION = new Set<string>(["SignalShotMotion", "PressureShotMotion", "FurnaceHoundLeapMotion", "EnemyChargeMotion", "EnemyLungeMotion"]);
const ATTACK = new Set<string>(["EnemyAttackTelegraph", "EnemyAttackImpact", ...MOTION]);

export function isClockworksEnemyCue(kind: string): boolean { return KINDS.has(kind); }
export function isClockworksEnemyMotion(kind: string): boolean { return MOTION.has(kind); }
export function presentationCounter(value: unknown): value is number {
  return Number.isSafeInteger(value) && (value as number) >= 0;
}
export function finitePresentationPosition(value: unknown): value is Vec3 {
  if (!value || typeof value !== "object") return false;
  const position = value as Partial<Vec3>;
  return [position.xM, position.yM, position.zM].every(item => typeof item === "number" && Number.isFinite(item) && Math.abs(item) <= 10_000);
}

/** Envelope validation precedes every cursor mutation; unknown generic kinds remain forward compatible. */
export function validPresentationEventEnvelope(value: unknown): value is PresentationEvent {
  if (!value || typeof value !== "object") return false;
  const event = value as PresentationEvent;
  return (event.protocolVersion === 1 || event.protocolVersion === 2) &&
    presentationCounter(event.eventId) && event.eventId > 0 && presentationCounter(event.worldEpoch) &&
    presentationCounter(event.serverTick) && typeof event.kind === "string" && event.kind.length > 0 && event.kind.length <= 128 &&
    finitePresentationPosition(event.positionM) && Number.isFinite(event.directionRad) &&
    Number.isFinite(event.radiusM) && event.radiusM >= 0 && event.radiusM <= 10_000 &&
    Number.isFinite(event.intensity) && event.intensity >= 0 && event.intensity <= 2 &&
    validSentinelChargeEnvelope(event) &&
    (!isClockworksEnemyCue(event.kind) || validClockworksEnemyEventEnvelope(event)) &&
    (event.protocolVersion !== 2 || event.combatFeedback === undefined || validCombatFeedbackEnvelope(event));
}

const COMBAT_OUTCOMES: Readonly<Record<string, string>> = {
  Hit: "enemy_hit", PulseHit: "enemy_hit", PierceHit: "enemy_hit", Damaged: "player_hurt",
  GuardImpact: "blocked", DamageAbsorbed: "absorbed", ActionRejected: "rejected",
};
const combatIdentity = (value: unknown): value is string => typeof value === "string" && /^[A-Za-z0-9_.:/-]{1,256}$/.test(value);
export type CombatFeedbackEvent = PresentationEventV2 & {
  combatFeedback: NonNullable<PresentationEventV2["combatFeedback"]>;
  durationMs: number;
};

/** Contact metadata is emitted only after the owner resolves a real combat result. */
export function validCombatFeedbackEnvelope(event: PresentationEvent): event is CombatFeedbackEvent {
  if (event.protocolVersion !== 2 || !event.combatFeedback) return false;
  const feedback = event.combatFeedback;
  if (!combatIdentity(feedback.worldId) || !combatIdentity(feedback.sceneId) ||
      COMBAT_OUTCOMES[event.kind] !== feedback.outcome ||
      !Number.isSafeInteger(event.durationMs) || event.durationMs! <= 0 || event.durationMs! > 2000 ||
      (feedback.requestId !== undefined && (!presentationCounter(feedback.requestId) || feedback.requestId <= 0)) ||
      (feedback.targetId !== undefined && !combatIdentity(feedback.targetId)) ||
      (feedback.sourceId !== undefined && !combatIdentity(feedback.sourceId)) ||
      (feedback.reason !== undefined && !combatIdentity(feedback.reason))) return false;
  if (feedback.outcome === "rejected") return feedback.sourceId === "player" && feedback.requestId !== undefined && feedback.reason !== undefined;
  if (feedback.outcome === "absorbed") return feedback.targetId === "player" && !!feedback.sourceId &&
    (feedback.reason === "invulnerable" || feedback.reason === "damage_reduced_to_zero");
  if (feedback.reason !== undefined) return false;
  if (feedback.outcome === "enemy_hit") return feedback.sourceId === "player" && !!feedback.targetId && feedback.targetId !== "player";
  return feedback.targetId === "player" && !!feedback.sourceId;
}

/** A lethal hit keeps its contact mark even after that same actor becomes inactive. */
export function combatFeedbackMatchesSnapshot(event: PresentationEvent, snapshot: WorldSnapshotEnvelope): event is CombatFeedbackEvent {
  if (!validCombatFeedbackEnvelope(event) || snapshot.protocolVersion !== 3 ||
      snapshot.player.currentHp <= 0 || event.worldEpoch !== snapshot.worldEpoch ||
      event.combatFeedback.worldId !== snapshot.worldId || event.combatFeedback.sceneId !== snapshot.sceneId ||
      event.serverTick > snapshot.serverTick || (snapshot.serverTick - event.serverTick) * 1000 / 60 >= event.durationMs) return false;
  if (event.combatFeedback.outcome !== "enemy_hit") return true;
  const targets = snapshot.actors.filter(actor => actor.entityId === event.combatFeedback.targetId && actor.entityId !== snapshot.player.entityId);
  if (targets.length !== 1) return false;
  const target = targets[0]!;
  return target.entityType !== "enemy.mist_harbor.signal_wraith" ||
    isNativeSignalWraith(target, snapshot.worldId, snapshot.sceneId) && validatedSignalPerception(target)?.precise === true;
}

/** Only wire shape is checked here. Current actor identity/liveness belongs to the presentation model. */
export function validClockworksEnemyEventEnvelope(event: PresentationEvent): event is ClockworksEnemyEvent {
  return !!event && event.protocolVersion === 2 && presentationCounter(event.eventId) && event.eventId > 0 &&
    presentationCounter(event.worldEpoch) && presentationCounter(event.serverTick) && KINDS.has(event.kind) &&
    typeof event.actorId === "string" && event.actorId.trim().length > 0 && event.actorId.length <= 256 &&
    finitePresentationPosition(event.positionM) && Number.isFinite(event.directionRad) &&
    Number.isFinite(event.radiusM) && event.radiusM > 0 && event.radiusM <= 64 &&
    (event.rangeM === undefined || (Number.isFinite(event.rangeM) && event.rangeM >= 0 && event.rangeM <= 128)) &&
    (event.kind !== "SignalBlink" || (event.attackKind === undefined && typeof event.rangeM === "number" && event.rangeM > 0 && event.rangeM <= 2)) &&
    (event.kind !== "EnemyAttackTelegraph" || (typeof event.rangeM === "number" && event.rangeM > 0)) &&
    Number.isFinite(event.intensity) && event.intensity >= 0 && event.intensity <= 2 &&
    typeof event.durationMs === "number" && Number.isFinite(event.durationMs) && event.durationMs > 0 && event.durationMs <= 60_000 &&
    (event.attackKind === undefined || ["pressure_shot", "bite", "leap", "charge", "slam", "lunge", "tide_swing", "tide_charge", "signal_shot"].includes(event.attackKind)) &&
    (!ATTACK.has(event.kind) || event.attackKind !== undefined) &&
    (event.attackKind === "tide_swing"
      ? typeof event.halfAngleRad === "number" && Number.isFinite(event.halfAngleRad) && event.halfAngleRad > 0 && event.halfAngleRad <= Math.PI
      : event.halfAngleRad === undefined) &&
    (!["tide_swing", "tide_charge"].includes(event.attackKind ?? "") || (typeof event.rangeM === "number" && event.rangeM > 0)) &&
    (["EnemyMemberHit", "EnemyMemberDisperse"].includes(event.kind)
      ? typeof event.memberId === "string" && event.memberId.length > 0 && event.memberId.length <= 300 && event.attackKind === undefined
      : event.memberId === undefined);
}

/** Actor association is deferred until the event tick is current. It uses only
 * public snapshot identity/liveness, never hidden AI or enemy health. */
export function clockworksEnemyActorForEvent(event: PresentationEvent, snapshot: WorldSnapshotEnvelope): ActorView | null {
  if (!snapshot || snapshot.protocolVersion !== 3 ||
      !((snapshot.worldId === "clockworks" && typeof snapshot.sceneId === "string" && snapshot.sceneId.startsWith("cw_")) ||
        (snapshot.worldId === "grey_hive" && ["gh_gate_b", "gh_lockdown", "gh_deep_decon"].includes(snapshot.sceneId)) ||
        (snapshot.worldId === "mist_harbor" && ["mh_drowned_quay", "mh_tidal_warehouse", "mh_signal_yard", "mh_breakwater", "mh_resonance_tower"].includes(snapshot.sceneId))) ||
      !presentationCounter(snapshot.worldEpoch) || !presentationCounter(snapshot.serverTick) || !Array.isArray(snapshot.actors) ||
      !validClockworksEnemyEventEnvelope(event) || event.worldEpoch !== snapshot.worldEpoch ||
      event.serverTick > snapshot.serverTick || event.durationMs - (snapshot.serverTick - event.serverTick) * 1000 / 60 <= 0) return null;
  const matches = snapshot.actors.filter(actor => actor?.entityId === event.actorId);
  const actor = matches[0];
  if (matches.length !== 1 || !actor || actor.actorKind !== "enemy" ||
      !["enemy.clockworks.pressure_drone", "enemy.clockworks.furnace_hound", "enemy.grey_hive.brute", "enemy.grey_hive.swarm", "enemy.mist_harbor.tidebound", "enemy.mist_harbor.signal_wraith"].includes(actor.entityType) ||
      !finitePresentationPosition(actor.transform?.positionM) ||
      (event.kind === "EnemyDeath" ? actor.active !== false : event.kind !== "EnemyMemberDisperse" && actor.active !== true)) return null;
  const wraith = actor.entityType === "enemy.mist_harbor.signal_wraith";
  if (wraith && (!isNativeSignalWraith(actor, snapshot.worldId, snapshot.sceneId) || !validatedSignalPerception(actor))) return null;
  // Ordinary attacks must have a precise, cue-bearing current body frame. No
  // stale ambiguity may conceal a cast or be accepted by the event cursor.
  if (wraith && (event.kind === "EnemyAttackTelegraph" || MOTION.has(event.kind)) && !actor.signalPerception?.precise) return null;
  const tidebound = actor.entityType === "enemy.mist_harbor.tidebound";
  if (tidebound) {
    const count = ({ mh_drowned_quay: 1, mh_breakwater: 2, mh_resonance_tower: 1 } as Record<string, number>)[snapshot.sceneId] ?? 0;
    if (snapshot.worldId !== "mist_harbor" || !Array.from({ length: count }, (_, i) => `${snapshot.sceneId}_tidebound_0${i + 1}`).includes(actor.entityId)) return null;
  }
  const swarm = actor.entityType === "enemy.grey_hive.swarm";
  if (swarm) {
    if (snapshot.worldId !== "grey_hive" || !["gh_lockdown", "gh_deep_decon"].includes(snapshot.sceneId) ||
        ![`${snapshot.sceneId}_swarm_01`, `${snapshot.sceneId}_swarm_02`].includes(actor.entityId)) return null;
    const members = validatedSwarmMembers(actor);
    if (!members) return null;
    if (event.memberId !== undefined) {
      const member = members.find(m => m.memberId === event.memberId);
      if (!member || member.active !== (event.kind === "EnemyMemberHit")) return null;
    }
  } else if (event.memberId !== undefined) return null;
  const brute = actor.entityType === "enemy.grey_hive.brute";
  if (!swarm && !tidebound && !wraith && (brute ? snapshot.worldId !== "grey_hive" || snapshot.sceneId !== "gh_gate_b" : snapshot.worldId !== "clockworks")) return null;
  const attacks = wraith ? ["signal_shot"] : tidebound ? ["tide_swing", "tide_charge"] : swarm ? ["lunge"] : brute ? ["charge", "slam"] : actor.entityType === "enemy.clockworks.pressure_drone" ? ["pressure_shot"] : ["bite", "leap"];
  if (event.attackKind !== undefined && !attacks.includes(event.attackKind)) return null;
  if (event.kind === "SignalShotMotion" && (!wraith || event.attackKind !== "signal_shot")) return null;
  if (event.kind === "SignalBlink" && !wraith) return null;
  if (event.kind === "PressureShotMotion" && (actor.entityType !== "enemy.clockworks.pressure_drone" || event.attackKind !== "pressure_shot")) return null;
  if (event.kind === "FurnaceHoundLeapMotion" && (actor.entityType !== "enemy.clockworks.furnace_hound" || event.attackKind !== "leap")) return null;
  if (event.kind === "EnemyChargeMotion" && (!(brute || tidebound) || event.attackKind !== (tidebound ? "tide_charge" : "charge"))) return null;
  if (event.kind === "EnemyLungeMotion" && (!swarm || event.attackKind !== "lunge")) return null;
  return actor;
}

/** Public member identity and liveness only; no hidden health or client simulation. */
export function validatedSwarmMembers(actor: ActorView): readonly ActorMemberView[] | null {
  if (actor.entityType !== "enemy.grey_hive.swarm" || !Array.isArray(actor.members) || actor.members.length !== 4 ||
      !finitePresentationPosition(actor.transform?.positionM)) return null;
  const center = actor.transform.positionM;
  if (!actor.members.every((m, i) => !!m && m.memberId === `${actor.entityId}/member/${i + 1}` &&
      typeof m.active === "boolean" && finitePresentationPosition(m.positionM) &&
      Number.isFinite(m.radiusM) && m.radiusM > 0 && m.radiusM <= 1 &&
      Math.abs(m.positionM.yM - center.yM) <= 0.0001 &&
      Math.hypot(m.positionM.xM - center.xM, m.positionM.zM - center.zM) + m.radiusM <= 1.0001) ||
      actor.active !== actor.members.some(m => m.active)) return null;
  return actor.members;
}

export function isNativeSignalWraith(actor: ActorView, worldId: string, sceneId: string): boolean {
  const count = ({ mh_tidal_warehouse: 1, mh_signal_yard: 3, mh_breakwater: 2, mh_resonance_tower: 4 } as Record<string, number>)[sceneId] ?? 0;
  return worldId === "mist_harbor" && actor?.entityType === "enemy.mist_harbor.signal_wraith" && actor.actorKind === "enemy" &&
    Array.from({ length: count }, (_, i) => `${sceneId}_signal_wraith_0${i + 1}`).includes(actor.entityId);
}
/** No true-position flag, AI, health or capability decision enters this projection. */
export function validatedSignalPerception(actor: ActorView): SignalPositionProjection | null {
  const p = actor.signalPerception;
  if (actor.entityType !== "enemy.mist_harbor.signal_wraith" || !p ||
      Object.keys(p).sort().join(",") !== "positionsM,precise,uncertaintyRadiusM" ||
      typeof p.precise !== "boolean" || !Number.isFinite(p.uncertaintyRadiusM) ||
      !Array.isArray(p.positionsM) || p.positionsM.length !== (p.precise ? 1 : 2) ||
      !p.positionsM.every(finitePresentationPosition) || !finitePresentationPosition(actor.transform?.positionM)) return null;
  const anchor = actor.transform.positionM;
  if (p.precise) return p.uncertaintyRadiusM === 0 && p.positionsM.every(point =>
    Math.hypot(point.xM-anchor.xM,point.yM-anchor.yM,point.zM-anchor.zM) <= 0.0001) ? p : null;
  const [a,b] = p.positionsM;
  if (!a || !b || !(p.uncertaintyRadiusM > 0 && p.uncertaintyRadiusM <= 2) ||
      Math.abs(a.yM-anchor.yM)>0.0001 || Math.abs(b.yM-anchor.yM)>0.0001 ||
      Math.abs(Math.hypot(a.xM-b.xM,a.zM-b.zM)-p.uncertaintyRadiusM)>0.0001 ||
      Math.hypot((a.xM+b.xM)/2-anchor.xM,(a.zM+b.zM)/2-anchor.zM)>0.0001) return null;
  return p;
}

/** Uses the Rust-owned projection only; never infers a source radius or grants mapping. */
export function signalWraithInterference(snapshot: WorldSnapshotEnvelope): ActorView | null {
  if (snapshot.protocolVersion !== 3 || !Array.isArray(snapshot.actors)) return null;
  const sources = snapshot.actors.filter(actor => actor.active && isNativeSignalWraith(actor,snapshot.worldId,snapshot.sceneId)
    && validatedSignalPerception(actor)?.precise === false);
  return sources.find(actor => snapshot.actors.filter(other => other.entityId === actor.entityId).length === 1) ?? null;
}

export function invalidSignalWraithProjection(snapshot: WorldSnapshotEnvelope): boolean {
  if (snapshot.protocolVersion !== 3 || !Array.isArray(snapshot.actors)) return false;
  return snapshot.actors.some(actor => actor.entityType === "enemy.mist_harbor.signal_wraith" &&
    (!isNativeSignalWraith(actor,snapshot.worldId,snapshot.sceneId) || !validatedSignalPerception(actor) ||
      snapshot.actors.filter(other => other.entityId === actor.entityId).length !== 1));
}
