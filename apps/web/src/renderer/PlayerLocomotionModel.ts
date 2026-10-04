import type { Vec3 } from "../protocol/types.js";
import { projectWorldPoint } from "./CameraModel.js";

/** A bounded interim deformation of the approved idle art, never an authored Walk clip. */
export const PLAYER_LOCOMOTION_KIND = "temporary_visual" as const;
export const PLAYER_LOCOMOTION_TICK_HZ = 60;
const MAX_GAP_TICKS = 15;
const MIN_SPEED_MPS = 0.025;
const MAX_WALK_SPEED_MPS = 8;
const STRIDE_METERS = 2.4;
const TAU = Math.PI * 2;

export interface PlayerLocomotionInput {
  worldId: string;
  sceneId: string;
  worldEpoch: number;
  playerEntityId: string;
  serverTick: number;
  /** Both vectors must be authoritative, not interpolated positions or input commands. */
  position: Vec3;
  velocity: Vec3;
  actionState: string;
  alive: boolean;
  paused: boolean;
  loading: boolean;
  reducedMotion: boolean;
}

export interface PlayerLocomotionFrame {
  readonly kind: typeof PLAYER_LOCOMOTION_KIND;
  readonly active: boolean;
  readonly phaseRad: number;
  readonly amplitude: number;
  /** Screen-space travel direction, independent of the authoritative combat facing. */
  readonly travelX: number;
  readonly travelY: number;
  readonly reducedMotion: boolean;
}

interface Sample {
  key: string;
  tick: number;
  position: Vec3;
  speed: number;
}

function finiteVector(value: Vec3): boolean {
  return value !== null && typeof value === "object" &&
    [value.xM, value.yM, value.zM].every(Number.isFinite);
}

function neutral(reducedMotion: boolean): PlayerLocomotionFrame {
  return { kind: PLAYER_LOCOMOTION_KIND, active: false, phaseRad: 0, amplitude: 0,
    travelX: 0, travelY: 0, reducedMotion };
}

/** No wall clock/ticker, input reads, transform writes, collision changes, or combat decisions. */
export class PlayerLocomotionModel {
  private previous: Sample | null = null;
  private phaseRad = 0;
  private frame = neutral(false);

  reset(): void {
    this.previous = null;
    this.phaseRad = 0;
    this.frame = neutral(false);
  }

  project(input: PlayerLocomotionInput): PlayerLocomotionFrame {
    const reducedMotion = input.reducedMotion === true;
    if (!input.worldId || !input.sceneId || !input.playerEntityId ||
        !Number.isSafeInteger(input.worldEpoch) || input.worldEpoch < 0 ||
        !Number.isSafeInteger(input.serverTick) || input.serverTick < 0 ||
        !finiteVector(input.position) || !finiteVector(input.velocity) ||
        input.alive !== true || input.paused !== false || input.loading !== false ||
        input.actionState !== "idle") {
      this.reset();
      return this.frame = neutral(reducedMotion);
    }
    const speed = Math.hypot(input.velocity.xM, input.velocity.zM);
    const sample: Sample = {
      key: JSON.stringify([input.worldId, input.sceneId, input.worldEpoch, input.playerEntityId]),
      tick: input.serverTick, position: { ...input.position }, speed,
    };
    const previous = this.previous;
    this.previous = sample;
    if (!previous || previous.key !== sample.key || speed < MIN_SPEED_MPS || speed > MAX_WALK_SPEED_MPS) {
      this.phaseRad = 0;
      return this.frame = neutral(reducedMotion);
    }
    const ticks = sample.tick - previous.tick;
    const dx = sample.position.xM - previous.position.xM;
    const dz = sample.position.zM - previous.position.zM;
    const dy = sample.position.yM - previous.position.yM;
    const distance = Math.hypot(dx, dz);
    // Repainting a snapshot cannot advance a gait. A contradictory same-tick sample resets it.
    if (ticks === 0 && distance === 0 && dy === 0) {
      return this.frame = this.displayFrame(this.frame.active, this.frame.amplitude,
        this.frame.travelX, this.frame.travelY, reducedMotion);
    }
    // Fixed-rate time is derived explicitly from authoritative ticks, not browser elapsed time.
    const seconds = ticks / PLAYER_LOCOMOTION_TICK_HZ;
    const plausibleDistance = Math.max(previous.speed, speed) * seconds * 1.75 + 0.08;
    if (ticks <= 0 || ticks > MAX_GAP_TICKS || !Number.isFinite(distance) ||
        distance > Math.min(2, plausibleDistance) || Math.abs(dy) > 0.8 ||
        distance / seconds > MAX_WALK_SPEED_MPS || distance / seconds < MIN_SPEED_MPS) {
      this.phaseRad = 0;
      return this.frame = neutral(reducedMotion);
    }
    const screen = projectWorldPoint({ xM: dx, yM: 0, zM: dz },
      { width: 0, height: 0, origin: { xM: 0, yM: 0, zM: 0 }, pixelsPerMeter: 1 });
    const screenDistance = Math.hypot(screen.x, screen.footY);
    let travelX = screen.x / screenDistance;
    let travelY = screen.footY / screenDistance;
    if (this.frame.active && !this.frame.reducedMotion) {
      // Travel can strafe or reverse while Mouse/Rust facing stays completely unchanged.
      const blend = 1 - Math.exp(-seconds / 0.07);
      travelX = this.frame.travelX + (travelX - this.frame.travelX) * blend;
      travelY = this.frame.travelY + (travelY - this.frame.travelY) * blend;
    }
    this.phaseRad = (this.phaseRad + Math.min(distance / STRIDE_METERS, seconds * 1.8) * TAU) % TAU;
    const amplitude = Math.min(1, Math.max(0.2, Math.min(speed, distance / seconds) / 2));
    return this.frame = this.displayFrame(true, amplitude, travelX, travelY, reducedMotion);
  }

  private displayFrame(active: boolean, amplitude: number, travelX: number, travelY: number,
    reducedMotion: boolean): PlayerLocomotionFrame {
    if (!active) return neutral(reducedMotion);
    // A small, fixed split stance has no rapid gait animation, even as new ticks arrive.
    return { kind: PLAYER_LOCOMOTION_KIND, active: true,
      phaseRad: reducedMotion ? Math.PI / 2 : this.phaseRad,
      amplitude: reducedMotion ? 0.18 : amplitude,
      travelX: reducedMotion ? 1 : travelX, travelY: reducedMotion ? 0 : travelY, reducedMotion };
  }
}
