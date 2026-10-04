import type { WorldSnapshotV3, WorldSnapshotEnvelope } from "../protocol/types.js";

export interface GreyHiveBeaconProjection {
  state: "uncollected" | "carried" | "mounted";
  legacyCompletedWithoutReceipt: boolean;
}

/** Missing old wire data grants no ownership and never paints a deployed device. */
export function assertGreyHiveBeacon(value: unknown, worldId: string): GreyHiveBeaconProjection {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("E_GH_BEACON_PROJECTION");
  const item = value as Record<string, unknown>;
  if (worldId !== "grey_hive" || Object.keys(item).length !== 2 ||
      !["uncollected", "carried", "mounted"].includes(item.state as string) ||
      typeof item.legacyCompletedWithoutReceipt !== "boolean" ||
      (item.legacyCompletedWithoutReceipt && item.state !== "uncollected")) {
    throw new Error("E_GH_BEACON_PROJECTION");
  }
  return value as GreyHiveBeaconProjection;
}

export function greyHiveBeaconStatus(snapshot: WorldSnapshotV3): string {
  const beacon = snapshot.greyHiveBeacon;
  if (!beacon || snapshot.worldId !== "grey_hive") return "";
  assertGreyHiveBeacon(beacon, snapshot.worldId);
  if (beacon.legacyCompletedWithoutReceipt) return "灰巢已通关 · 信标尚未重新确认";
  if (beacon.state === "carried") return "信标已携带 · 可前往撤离";
  if (beacon.state === "mounted") return "信标已部署 · 可前往撤离";
  return snapshot.sceneId === "gh_exit" ? "尚未收取信标 · 返回信标室收取" : "便携信标尚未收取";
}

/** Authoritative state controls the two preloaded, admitted prop images. */
export function greyHiveBeaconSpriteVisible(snapshot: WorldSnapshotEnvelope, spriteId: string): boolean {
  if (spriteId === "exit_beacon_deployed_at_mount") return false;
  if (spriteId !== "beacon_folded_approved" && spriteId !== "beacon_mounted_approved") return true;
  if (snapshot.protocolVersion !== 3 || snapshot.worldId !== "grey_hive" || snapshot.sceneId !== "gh_beacon" || !snapshot.greyHiveBeacon) return false;
  const state = assertGreyHiveBeacon(snapshot.greyHiveBeacon, snapshot.worldId).state;
  return spriteId === "beacon_folded_approved" ? state === "uncollected" : state === "mounted";
}
