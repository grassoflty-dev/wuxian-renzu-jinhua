import type { Vec3 } from "../protocol/types.js";
import { projectWorldPoint, type CameraFrame } from "./CameraModel.js";

export interface CameraRigConfig {
  /** Screen-space dead zone around the aim-offset target, in pixels. */
  deadZoneHalfWidthPx: number;
  deadZoneHalfHeightPx: number;
  /** Exponential follow time constant. */
  smoothTimeSeconds: number;
  /** Distance that the camera target leads the authoritative aim vector. */
  aimOffsetMeters: number;
}

export const DEFAULT_CAMERA_RIG_CONFIG: Readonly<CameraRigConfig> = Object.freeze({
  deadZoneHalfWidthPx: 72,
  deadZoneHalfHeightPx: 42,
  smoothTimeSeconds: 0.18,
  aimOffsetMeters: 1.25,
});

/** Fixed-angle camera state. It changes only the projection origin, never yaw or game state. */
export class CameraRig {
  private origin: Vec3 | null = null;

  constructor(private readonly config: Readonly<CameraRigConfig> = DEFAULT_CAMERA_RIG_CONFIG) {
    if (!Number.isFinite(config.deadZoneHalfWidthPx) || config.deadZoneHalfWidthPx < 0 ||
        !Number.isFinite(config.deadZoneHalfHeightPx) || config.deadZoneHalfHeightPx < 0 ||
        !Number.isFinite(config.smoothTimeSeconds) || config.smoothTimeSeconds <= 0 ||
        !Number.isFinite(config.aimOffsetMeters) || config.aimOffsetMeters < 0) {
      throw new Error("E_CAMERA_RIG_CONFIG");
    }
  }

  reset(): void { this.origin = null; }

  update(player: Vec3, aimX: number, aimZ: number, width: number, height: number, deltaSeconds: number,
    pixelsPerMeter = 48): CameraFrame {
    if (![player.xM, player.yM, player.zM, aimX, aimZ, width, height, deltaSeconds, pixelsPerMeter].every(Number.isFinite) ||
        width < 0 || height < 0 || deltaSeconds < 0 || pixelsPerMeter <= 0) {
      throw new Error("E_CAMERA_RIG_INPUT");
    }
    const viewportScale = pixelsPerMeter / 48;
    const frame = (origin: Vec3): CameraFrame => pixelsPerMeter === 48
      ? { width, height, origin: { ...origin } }
      : { width, height, origin: { ...origin }, pixelsPerMeter };

    const aimLength = Math.hypot(aimX, aimZ);
    const target: Vec3 = aimLength > 0
      ? { xM: player.xM + aimX / aimLength * this.config.aimOffsetMeters, yM: player.yM, zM: player.zM + aimZ / aimLength * this.config.aimOffsetMeters }
      : { ...player };

    if (!this.origin) {
      this.origin = target;
      return frame(this.origin);
    }

    const currentFrame: CameraFrame = { width, height, origin: this.origin, pixelsPerMeter };
    const screen = projectWorldPoint(player, currentFrame);
    const targetX = -(target.xM - player.xM - (target.zM - player.zM)) * pixelsPerMeter;
    const targetY = -(target.xM - player.xM + (target.zM - player.zM)) * pixelsPerMeter / 2;
    const relativeX = screen.x - width / 2;
    const relativeY = screen.footY - height / 2;
    const deadZoneHalfWidthPx = this.config.deadZoneHalfWidthPx * viewportScale;
    const deadZoneHalfHeightPx = this.config.deadZoneHalfHeightPx * viewportScale;
    const clampedX = clamp(relativeX, targetX - deadZoneHalfWidthPx, targetX + deadZoneHalfWidthPx);
    const clampedY = clamp(relativeY, targetY - deadZoneHalfHeightPx, targetY + deadZoneHalfHeightPx);

    // The projection maps world x/z to screen x/footY by (x-z)*pixelsPerMeter and (x+z)*pixelsPerMeter/2.
    // Invert that mapping to move the origin just enough to return the player to the dead zone.
    const screenCorrectionX = relativeX - clampedX;
    const screenCorrectionY = relativeY - clampedY;
    const desired: Vec3 = {
      xM: this.origin.xM + (screenCorrectionX / pixelsPerMeter + screenCorrectionY / (pixelsPerMeter / 2)) / 2,
      yM: target.yM,
      zM: this.origin.zM + (screenCorrectionY / (pixelsPerMeter / 2) - screenCorrectionX / pixelsPerMeter) / 2,
    };

    const alpha = 1 - Math.exp(-deltaSeconds / this.config.smoothTimeSeconds);
    this.origin = {
      xM: lerp(this.origin.xM, desired.xM, alpha),
      yM: lerp(this.origin.yM, desired.yM, alpha),
      zM: lerp(this.origin.zM, desired.zM, alpha),
    };
    return frame(this.origin);
  }
}

function clamp(value: number, min: number, max: number): number { return Math.max(min, Math.min(max, value)); }
function lerp(from: number, to: number, amount: number): number { return from + (to - from) * amount; }
