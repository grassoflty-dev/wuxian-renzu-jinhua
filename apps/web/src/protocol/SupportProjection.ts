import type { Vec3 } from "./types.js";
export interface SupportPose {
  supportId: string;
  phase: "lower_hold" | "rising" | "upper_hold" | "falling" | "stationary";
  heightM: number; velocityMps: number; phaseElapsedMs: number;
}
export interface SupportRider {
  positionM: Vec3; radiusM: number;
  mode: "surface" | "floor" | "airborne" | "traversal";
  supportId: string | null;
}
/** One locked Rust state; neither phase nor rider position is client-owned. */
export interface SupportSceneProjection {
  schemaVersion: 1;
  worldId: string; sceneId: string; worldEpoch: number;
  serverTick: number; authorityRevision: number; serverTimeMs: number;
  sceneSourceSha256: string;
  poses: SupportPose[]; rider: SupportRider;
}
interface SnapshotIdentity {
  worldId: unknown; sceneId: unknown; worldEpoch: unknown; serverTick: unknown;
  authorityRevision: unknown; player: unknown;
}
const ID = /^[A-Za-z0-9_.:-]{1,128}$/;
const SHA = /^[0-9a-f]{64}$/;
const finite = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);
const counter = (v: unknown): v is number => typeof v === "number" && Number.isSafeInteger(v) && v >= 0;
const record = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);
const exact = (v: Record<string, unknown>, keys: string[]) => Object.keys(v).length === keys.length && keys.every(k => Object.hasOwn(v, k));
function fail(): never { throw new Error("E_SUPPORT_PROJECTION_INVALID"); }
function vec(v: unknown): v is Vec3 {
  return record(v) && exact(v, ["xM", "yM", "zM"]) && finite(v.xM) && finite(v.yM) && finite(v.zM) && v.yM >= 0 && v.yM <= 3;
}
export function assertSupportScene(value: unknown, snapshot: SnapshotIdentity): SupportSceneProjection {
  if (!record(value) || !exact(value,["schemaVersion","worldId","sceneId","worldEpoch","serverTick","authorityRevision","serverTimeMs","sceneSourceSha256","poses","rider"]) ||
      value.schemaVersion !== 1 || value.worldId !== snapshot.worldId || value.sceneId !== snapshot.sceneId ||
      value.worldEpoch !== snapshot.worldEpoch || value.serverTick !== snapshot.serverTick || value.authorityRevision !== snapshot.authorityRevision ||
      !counter(value.worldEpoch) || value.worldEpoch === 0 || !counter(value.serverTick) || !counter(value.authorityRevision) || !counter(value.serverTimeMs) ||
      typeof value.sceneSourceSha256 !== "string" || !SHA.test(value.sceneSourceSha256) ||
      !Array.isArray(value.poses) || value.poses.length === 0 || value.poses.length > 64) fail();
  let previous = "";
  for (const pose of value.poses) {
    if (!record(pose) || !exact(pose,["supportId","phase","heightM","velocityMps","phaseElapsedMs"]) ||
        typeof pose.supportId !== "string" || !ID.test(pose.supportId) || pose.supportId <= previous ||
        !finite(pose.heightM) || pose.heightM < 0 || pose.heightM > 3 || !finite(pose.velocityMps) || Math.abs(pose.velocityMps) > 12 ||
        !counter(pose.phaseElapsedMs) || pose.phaseElapsedMs > 60000 || typeof pose.phase !== "string" || !["lower_hold","rising","upper_hold","falling","stationary"].includes(pose.phase)) fail();
    if (pose.phase === "stationary" && (pose.velocityMps !== 0 || pose.phaseElapsedMs !== 0) ||
        (pose.phase === "lower_hold" || pose.phase === "upper_hold") && (pose.velocityMps !== 0 || pose.phaseElapsedMs > 30000) ||
        pose.phase === "rising" && pose.velocityMps <= 0 || pose.phase === "falling" && pose.velocityMps >= 0) fail();
    previous = pose.supportId;
  }
  const rider = value.rider;
  const player = record(snapshot.player) && record(snapshot.player.transform) ? snapshot.player.transform.positionM : undefined;
  if (!record(rider) || !exact(rider,["positionM","radiusM","mode","supportId"]) || !vec(rider.positionM) || !vec(player) ||
      rider.positionM.xM !== player.xM || rider.positionM.yM !== player.yM || rider.positionM.zM !== player.zM ||
      !finite(rider.radiusM) || rider.radiusM <= 0 || typeof rider.mode !== "string" || !["surface","floor","airborne","traversal"].includes(rider.mode)) fail();
  if (rider.mode === "surface") {
    const pose = typeof rider.supportId === "string" && value.poses.find(p => p.supportId === rider.supportId);
    if (!pose || Math.abs(pose.heightM - rider.positionM.yM) > 0.001) fail();
  } else if (rider.supportId !== null || rider.mode === "floor" && Math.abs(rider.positionM.yM) > 0.001) fail();
  return value as unknown as SupportSceneProjection;
}
/** Exact visible geometry identity. Authority-only metadata changes at Ready or
 * pause do not make an otherwise unchanged visible frame different. */
export function supportGeometryKey(value: SupportSceneProjection): string {
  return JSON.stringify([value.worldId,value.sceneId,value.worldEpoch,value.serverTick,value.serverTimeMs,
    value.sceneSourceSha256,value.poses,value.rider]);
}
