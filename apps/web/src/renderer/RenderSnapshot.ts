import { assertSnapshot, assertSnapshotV3, type ActorView, type DoorView, type Vec3, type WorldSnapshotEnvelope } from "../protocol/types.js";
import type { AnimationDirection } from "../assets/AssetRegistry.js";
import { projectWorldPoint } from "./CameraModel.js";

export interface RenderSnapshot {
  worldId: string;
  sceneId: string;
  worldEpoch: number;
  serverTick: number;
  playerEntityId: string;
  playerPosition: Vec3;
  playerVelocity: Vec3;
  playerYawRad: number;
  playerFacingX: number;
  playerFacingZ: number;
  playerAimX: number;
  playerAimZ: number;
  playerFacingSource: "v3-authoritative" | "v2-yaw-migration";
  playerActionState: string;
  actors: readonly ActorView[];
  doors: readonly DoorView[];
}

/** Accept current v3 snapshots and the v2 nested-view migration shape. */
export function normalizeRenderSnapshot(input: WorldSnapshotEnvelope): RenderSnapshot {
  if (input.protocolVersion === 2) {
    const snapshot = assertSnapshot(input);
    const view = snapshot.view;
    return {
      worldId: view.worldId,
      sceneId: "legacy-v2",
      worldEpoch: view.worldEpoch,
      serverTick: view.serverTick,
      playerEntityId: view.player.entityId,
      playerPosition: view.player.transform.positionM,
      playerVelocity: view.player.velocityMps,
      playerYawRad: view.player.transform.yawRad,
      playerFacingX: Math.sin(view.player.transform.yawRad),
      playerFacingZ: Math.cos(view.player.transform.yawRad),
      playerAimX: 0,
      playerAimZ: 0,
      playerFacingSource: "v2-yaw-migration",
      playerActionState: "idle",
      actors: view.actors,
      doors: view.doors,
    };
  }
  const snapshot = assertSnapshotV3(input);
  return {
    worldId: snapshot.worldId,
    sceneId: snapshot.sceneId,
    worldEpoch: snapshot.worldEpoch,
    serverTick: snapshot.serverTick,
    playerEntityId: snapshot.player.entityId,
    playerPosition: snapshot.player.transform.positionM,
    playerVelocity: snapshot.player.velocityMps,
    playerYawRad: snapshot.player.transform.yawRad,
    playerFacingX: snapshot.player.facingX,
    playerFacingZ: snapshot.player.facingZ,
    playerAimX: snapshot.player.aimX,
    playerAimZ: snapshot.player.aimZ,
    playerFacingSource: "v3-authoritative",
    playerActionState: snapshot.player.actionState,
    actors: snapshot.actors,
    doors: snapshot.doors,
  };
}

const SCREEN_DIRECTIONS: readonly AnimationDirection[] = [
  "east", "south_east", "south", "south_west", "west", "north_west", "north", "north_east",
];

/** Projects the world-facing vector with the same fixed isometric camera as world positions. */
export function screenFacingDirection(facingX: number, facingZ: number): AnimationDirection {
  if (!Number.isFinite(facingX) || !Number.isFinite(facingZ) || Math.hypot(facingX, facingZ) === 0) {
    throw new Error("E_RENDERER_FACING_VECTOR");
  }
  const projected = projectWorldPoint(
    { xM: facingX, yM: 0, zM: facingZ },
    { width: 0, height: 0, origin: { xM: 0, yM: 0, zM: 0 } },
  );
  if (!Number.isFinite(projected.x) || !Number.isFinite(projected.footY) || Math.hypot(projected.x, projected.footY) === 0) {
    throw new Error("E_RENDERER_FACING_VECTOR");
  }
  const sector = Math.round(Math.atan2(projected.footY, projected.x) / (Math.PI / 4));
  return SCREEN_DIRECTIONS[((sector % 8) + 8) % 8]!;
}
