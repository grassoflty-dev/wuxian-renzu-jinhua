import type { BaizhiNpcProjection, BaizhiProjection, Vec3 } from "../protocol/types.js";
import { BASE_PIXELS_PER_METER, projectWorldPoint, type CameraFrame } from "./CameraModel.js";

export const BAIZHI_ENTITY_ID = "gh_bz_whitezhi_v1";
export const BAIZHI_ENTITY_TYPE = "npc.baizhi";
export const BAIZHI_PLACEHOLDER_KIND = "DEV_WHITEZHI_PLACEHOLDER";
export const BAIZHI_PLACEHOLDER_HEIGHT_M = 1.7;
export const BAIZHI_PLACEHOLDER_WIDTH_M = 0.45;
const POSITION = [8, 0, 10.5] as const;

export interface BaizhiPresentation {
  baizhi: BaizhiProjection;
  npc: BaizhiNpcProjection;
}

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

/** The optional NPC channel is isolated: bad/old wire data never rejects the main snapshot. */
export function resolveBaizhiPresentation(snapshot: unknown): BaizhiPresentation | null {
  if (!record(snapshot) || snapshot.protocolVersion !== 3 || snapshot.worldId !== "grey_hive" ||
      snapshot.sceneId !== "gh_bio_isolation" || !Number.isSafeInteger(snapshot.worldEpoch) ||
      (snapshot.worldEpoch as number) < 1 || !record(snapshot.baizhi) ||
      !Array.isArray(snapshot.npcs) || snapshot.npcs.length !== 1) return null;
  const baizhi = snapshot.baizhi, npc = snapshot.npcs[0];
  if (baizhi.schemaVersion !== 1 || !["unresolved", "taken", "left"].includes(baizhi.choice as string) ||
      typeof baizhi.available !== "boolean" || typeof baizhi.canInteract !== "boolean" ||
      !baizhi.available || !record(npc) || npc.entityId !== BAIZHI_ENTITY_ID ||
      npc.entityType !== BAIZHI_ENTITY_TYPE || !Array.isArray(npc.position) || npc.position.length !== 3 ||
      !Array.from(npc.position).every((coordinate, index) => typeof coordinate === "number" &&
        Number.isFinite(coordinate) && coordinate === POSITION[index]) ||
      typeof npc.yawRad !== "number" || !Number.isFinite(npc.yawRad) ||
      Math.abs(Math.sin(npc.yawRad)) > 1e-6 || Math.abs(Math.cos(npc.yawRad) + 1) > 1e-6 ||
      typeof npc.interactable !== "boolean" || npc.interactable !== baizhi.canInteract) return null;
  // Return detached, known fields only; no route/flags, actor, HP, AI, or asset fallback.
  return {
    baizhi: { schemaVersion: 1, choice: baizhi.choice as BaizhiProjection["choice"],
      available: true, canInteract: baizhi.canInteract },
    npc: { entityId: BAIZHI_ENTITY_ID, entityType: BAIZHI_ENTITY_TYPE,
      position: [POSITION[0], POSITION[1], POSITION[2]], yawRad: npc.yawRad, interactable: npc.interactable },
  };
}

export interface BaizhiPlaceholderFrame {
  key: string;
  layer: "L3_ACTORS";
  position: Vec3;
  screenX: number;
  screenY: number;
  footY: number;
  displayScale: number;
  yawRad: number;
  /** Unit facing under the existing fixed projection, never camera rotation. */
  facing: { x: number; y: number };
}

/** A dedicated exact-NPC code drawing; it does not register an asset or ActorView. */
export function projectBaizhiPlaceholder(snapshot: unknown, camera: CameraFrame): BaizhiPlaceholderFrame | null {
  const resolved = resolveBaizhiPresentation(snapshot);
  const pixelsPerMeter = camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER;
  if (!resolved || ![camera.width, camera.height, camera.origin.xM, camera.origin.yM,
      camera.origin.zM, pixelsPerMeter].every(Number.isFinite) || pixelsPerMeter <= 0 ||
      camera.width <= 0 || camera.height <= 0) return null;
  const { npc } = resolved;
  const position = { xM: npc.position[0], yM: npc.position[1], zM: npc.position[2] };
  const screen = projectWorldPoint(position, camera);
  const forward = projectWorldPoint({ xM: position.xM + Math.sin(npc.yawRad), yM: position.yM,
    zM: position.zM + Math.cos(npc.yawRad) }, camera);
  const dx = forward.x - screen.x, dy = forward.y - screen.y, length = Math.hypot(dx, dy);
  if (![screen.x, screen.y, screen.footY, length].every(Number.isFinite) || length <= 0) return null;
  return { key: `npc:${BAIZHI_ENTITY_ID}`, layer: "L3_ACTORS", position,
    screenX: screen.x, screenY: screen.y, footY: screen.footY,
    displayScale: pixelsPerMeter / BASE_PIXELS_PER_METER, yawRad: npc.yawRad,
    facing: { x: dx / length, y: dy / length } };
}
