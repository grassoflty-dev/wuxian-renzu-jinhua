import { isFacilityCollection } from "./FacilityMapModel.js";
import type { Vec3 } from "../protocol/types.js";

export interface CameraFrame {
  width: number;
  height: number;
  origin: Vec3;
  pixelsPerMeter?: number;
}

export interface ScreenPoint {
  x: number;
  y: number;
  footY: number;
}

export interface SpriteScaleProfile {
  assetId: string;
  kind: string;
  category: string;
  anchorY: number;
  scale: number;
  layerId?: string;
}

export interface SceneScaleBounds { width: number; depth: number }

export const BASE_VIEWPORT_WIDTH = 1280;
export const BASE_VIEWPORT_HEIGHT = 720;
export const BASE_PIXELS_PER_METER = 48;

/** Keeps a stable world scale as the game viewport grows beyond the 720p baseline. */
export function viewportPixelsPerMeter(width: number, height: number): number {
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) throw new Error("E_CAMERA_VIEWPORT");
  const viewportScale = Math.max(0.5, Math.min(width / BASE_VIEWPORT_WIDTH, height / BASE_VIEWPORT_HEIGHT));
  return BASE_PIXELS_PER_METER * viewportScale;
}

/** Converts authored frame pixels to a stable physical footprint without changing world coordinates. */
export function resolveSpriteDisplayScale(
  asset: SpriteScaleProfile,
  frameWidth: number,
  frameHeight: number,
  pixelsPerMeter: number,
  bounds: SceneScaleBounds,
  layerOverride?: string,
): number {
  if (![frameWidth, frameHeight, pixelsPerMeter, bounds.width, bounds.depth, asset.scale, asset.anchorY]
    .every(Number.isFinite) || frameWidth <= 0 || frameHeight <= 0 || pixelsPerMeter <= 0 ||
    bounds.width <= 0 || bounds.depth <= 0 || asset.scale < 0 || asset.anchorY < 0 || asset.anchorY > 1) {
    throw new Error("E_CAMERA_SPRITE_SCALE");
  }

  const layerId = layerOverride ?? asset.layerId;
  const fullSceneLayer = layerId === "visual.background" || layerId === "visual.floor" ||
    layerId === "visual.floor_detail" ||
    (asset.category === "world" && asset.anchorY < 0.75 && layerId === "visual.props_back");
  if (fullSceneLayer && !isFacilityCollection(asset.assetId)) {
    return asset.scale * Math.min(bounds.width * pixelsPerMeter / frameWidth, bounds.depth * pixelsPerMeter / frameHeight);
  }

  let worldHeightMeters: number | null = null;
  if (asset.kind === "Actor") worldHeightMeters = 1.72;
  else if (asset.kind === "Boss") worldHeightMeters = 2.6;
  else if (asset.kind === "Door") worldHeightMeters = 2.4;
  else if (asset.kind === "Prop" || /(?:^|[._-])terminals?(?:[._-]|$)/i.test(asset.assetId)) worldHeightMeters = 2.2;
  else if (asset.category === "world" && layerId === "visual.props_back") worldHeightMeters = Math.min(bounds.depth * 0.5, 8);

  return worldHeightMeters === null
    ? asset.scale
    : asset.scale * worldHeightMeters * pixelsPerMeter / frameHeight;
}

/** Frozen fixed-angle 2.5D projection: no camera yaw, zoom, or game-state writes. */
export function projectWorldPoint(position: Vec3, camera: CameraFrame): ScreenPoint {
  const pixelsPerMeter = camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER;
  const dx = position.xM - camera.origin.xM;
  const dz = position.zM - camera.origin.zM;
  const footY = camera.height / 2 + (dx + dz) * pixelsPerMeter / 2;
  return {
    x: camera.width / 2 + (dx - dz) * pixelsPerMeter,
    y: footY - (position.yM - camera.origin.yM) * pixelsPerMeter,
    footY,
  };
}
