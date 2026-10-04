import type { PresentationEvent, Vec3, WorldSnapshotEnvelope, WorldSnapshotV3 } from "./types.js";

export const SENTINEL_ID = "gh_sentinel_arena_sentinel_01";
export const SENTINEL_TYPE = "enemy.grey_hive.sentinel";
export type SentinelPhase = "chase" | "light_windup" | "heavy_windup" | "charge_windup" | "charge" | "recover" | "hit" | "stagger" | "dead" | "inactive";
export interface SentinelWarning {
  shape: "circle" | "corridor";
  originM: Vec3;
  directionRad: number;
  radiusM: number;
  rangeM: number;
}
export interface SentinelEncounter {
  actorId: string;
  phase: SentinelPhase;
  remainingMs: number;
  attackSerial: number;
  positionM: Vec3;
  warning: SentinelWarning | null;
}
const PHASES = new Set<SentinelPhase>(["chase", "light_windup", "heavy_windup", "charge_windup", "charge", "recover", "hit", "stagger", "dead", "inactive"]);
const THREATS = new Set<SentinelPhase>(["light_windup", "heavy_windup", "charge_windup", "charge"]);
const EPS = 0.001;
function object(value: unknown): value is Record<string, unknown> { return !!value && typeof value === "object" && !Array.isArray(value); }
function keys(value: object, expected: string): boolean { return Object.keys(value).sort().join(",") === expected; }
function point(value: unknown): value is Vec3 {
  return object(value) && keys(value,"xM,yM,zM") && [value.xM,value.yM,value.zM].every(n=>typeof n === "number" && Number.isFinite(n) && Math.abs(n)<=10_000);
}
function near(a: number,b: number): boolean { return Math.abs(a-b)<=EPS; }
function same(a: Vec3,b: Vec3): boolean { return near(a.xM,b.xM)&&near(a.yM,b.yM)&&near(a.zM,b.zM); }
function fail(): never { throw new Error("E_SENTINEL_ENCOUNTER_INVALID"); }

/** Native V3 arena snapshots cannot release the owner without the exact current warning. */
export function assertSentinelEncounter(snapshot: WorldSnapshotV3): SentinelEncounter | null {
  const native = snapshot.worldId === "grey_hive" && snapshot.sceneId === "gh_sentinel_arena";
  if (!native) { if (Object.hasOwn(snapshot,"sentinelEncounter")) fail(); return null; }
  if (!Array.isArray(snapshot.actors)) fail();
  const candidates = snapshot.actors.filter(a=>a?.entityId===SENTINEL_ID || a?.entityType===SENTINEL_TYPE);
  const actor = candidates[0];
  if (candidates.length!==1 || !actor || actor.entityId!==SENTINEL_ID || actor.entityType!==SENTINEL_TYPE ||
      actor.actorKind!=="sentinel" || typeof actor.active!=="boolean" || !point(actor.transform?.positionM)) fail();
  const p = snapshot.sentinelEncounter;
  if (!object(p) || !keys(p,"actorId,attackSerial,phase,positionM,remainingMs,warning") || p.actorId!==SENTINEL_ID ||
      !PHASES.has(p.phase as SentinelPhase) || !Number.isSafeInteger(p.attackSerial) || (p.attackSerial as number)<0 ||
      typeof p.remainingMs!=="number" || !Number.isSafeInteger(p.remainingMs) || p.remainingMs<0 || p.remainingMs>60_000 ||
      !point(p.positionM) || !same(p.positionM,actor.transform.positionM)) fail();
  const phase = p.phase as SentinelPhase;
  const inactive = phase === "dead" || phase === "inactive";
  if (inactive !== !actor.active || ((phase==="chase" || inactive) && p.remainingMs!==0)) fail();
  if ((phase === "hit" && p.remainingMs > 134) || (phase === "stagger" && p.remainingMs > 884) || (phase === "recover" && p.remainingMs > 500)) fail();
  if (THREATS.has(phase)) {
    const cap = phase==="light_windup" ? 250 : phase==="heavy_windup" ? 400 : phase==="charge_windup" ? 884 : 600;
    if (!(p.remainingMs>0 && p.remainingMs<=cap)) fail();
    const w = p.warning;
    if (!object(w) || !keys(w,"directionRad,originM,radiusM,rangeM,shape") || !point(w.originM) ||
        typeof w.directionRad!=="number" || !Number.isFinite(w.directionRad) || Math.abs(w.directionRad)>Math.PI+EPS ||
        typeof w.radiusM!=="number" || !Number.isFinite(w.radiusM) || typeof w.rangeM!=="number" || !Number.isFinite(w.rangeM)) fail();
    if (phase==="light_windup" || phase==="heavy_windup") {
      if (w.shape!=="circle" || !near(w.radiusM,phase==="light_windup"?1.8:2.2) || w.rangeM!==0 || w.directionRad!==0 || !same(w.originM,p.positionM)) fail();
    } else {
      if (w.shape!=="corridor" || !near(w.radiusM,0.6) || !(w.rangeM>0 && w.rangeM<=3.6+EPS) || !near(w.originM.yM,p.positionM.yM) || p.attackSerial===0) fail();
      const dx=p.positionM.xM-w.originM.xM,dz=p.positionM.zM-w.originM.zM;
      const along=dx*Math.sin(w.directionRad)+dz*Math.cos(w.directionRad),side=dx*Math.cos(w.directionRad)-dz*Math.sin(w.directionRad);
      if (Math.abs(side)>EPS || along < -EPS || along>w.rangeM+EPS || (phase==="charge_windup" && !same(w.originM,p.positionM))) fail();
    }
  } else if (p.warning!==null) fail();
  return p as unknown as SentinelEncounter;
}

/** Deliberately excludes authorityRevision: a no-motion receipt may share the drawn geometry. */
export function sentinelGeometryKey(snapshot: WorldSnapshotEnvelope): string | null {
  if (snapshot.protocolVersion!==3) return null;
  const p=assertSentinelEncounter(snapshot);
  if (!p) return null;
  const w=p.warning;
  return JSON.stringify([p.actorId,p.phase,p.remainingMs,p.attackSerial,p.positionM.xM,p.positionM.yM,p.positionM.zM,
    w && [w.shape,w.originM.xM,w.originM.yM,w.originM.zM,w.directionRad,w.radiusM,w.rangeM]]);
}

export function sentinelStatus(snapshot: WorldSnapshotV3): string {
  const p=assertSentinelEncounter(snapshot);
  if (!p) return "";
  if (p.phase==="charge_windup") return "哨卫蓄力冲压 · 移出红色走廊";
  if (p.phase==="charge") return "哨卫冲压中";
  if (p.phase==="stagger") return "哨卫硬直 · 可反击";
  if (p.phase==="hit") return "哨卫受击";
  if (p.phase==="inactive") return "哨卫已停用";
  return "";
}

export function isSentinelWarning(kind: string): boolean { return ["SentinelAttackWindup","SentinelHeavyWindup","SentinelChargeWindup"].includes(kind); }

/** New native warning audio is bound to the exact visible phase and serial. */
export function sentinelWarningMatches(event: PresentationEvent,snapshot: WorldSnapshotEnvelope): boolean {
  if (!isSentinelWarning(event.kind)) return true;
  if (snapshot.protocolVersion!==3 || snapshot.worldId!=="grey_hive" || snapshot.sceneId!=="gh_sentinel_arena") return false;
  let p: SentinelEncounter | null;
  try { p=assertSentinelEncounter(snapshot); } catch { return false; }
  const expected=event.kind==="SentinelAttackWindup"?"light_windup":event.kind==="SentinelHeavyWindup"?"heavy_windup":"charge_windup";
  return !!p?.warning && p.phase===expected && event.protocolVersion===2 && event.worldEpoch===snapshot.worldEpoch &&
    event.serverTick<=snapshot.serverTick && event.actorId===p.actorId && event.attackId===p.attackSerial &&
    point(event.positionM) && same(event.positionM,p.warning.originM) && near(event.directionRad,p.warning.directionRad) &&
    near(event.radiusM,p.warning.radiusM) && typeof event.rangeM==="number" && near(event.rangeM,p.warning.rangeM) &&
    typeof event.durationMs==="number" && Number.isSafeInteger(event.durationMs) && event.durationMs>0 &&
    event.durationMs <= (expected === "light_windup" ? 250 : expected === "heavy_windup" ? 400 : 884) &&
    (snapshot.serverTick-event.serverTick)*1000/60 < event.durationMs;
}

/** Charge is a new typed cue; malformed future rows cannot occupy the cursor queue. */
export function validSentinelChargeEnvelope(event: PresentationEvent): boolean {
  if (event.kind !== "SentinelChargeWindup") return true;
  return event.protocolVersion === 2 && event.actorId === SENTINEL_ID &&
    Number.isSafeInteger(event.attackId) && (event.attackId as number) > 0 &&
    typeof event.durationMs === "number" && Number.isSafeInteger(event.durationMs) && event.durationMs > 0 && event.durationMs <= 884 &&
    point(event.positionM) && near(event.radiusM,0.6) && Number.isFinite(event.directionRad) && Math.abs(event.directionRad)<=Math.PI+EPS &&
    typeof event.rangeM === "number" && Number.isFinite(event.rangeM) && event.rangeM>0 && event.rangeM<=3.6+EPS;
}
