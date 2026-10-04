import { MeshPlane, type DestroyOptions, type Texture } from "pixi.js";
import type { AnimationDirection } from "../assets/AssetRegistry.js";
import { PLAYER_LOCOMOTION_KIND, type PlayerLocomotionFrame } from "./PlayerLocomotionModel.js";

export const PLAYER_LOCOMOTION_VERTICES_X = 21;
export const PLAYER_LOCOMOTION_VERTICES_Y = 33;
/** The top half and hip seam are exactly immobile in texture-local coordinates. */
export const PLAYER_LOCOMOTION_HIP_Y = 0.5;
export const PLAYER_LOCOMOTION_MAX_X = 0.06;
export const PLAYER_LOCOMOTION_MAX_Y = 0.036;

type Leg = readonly [hipX: number, kneeX: number, ankleX: number, ankleY: number, width: number];
/** Texture-local leg centers measured against all eight approved 336 x 560 idle frames. */
const LEGS: Record<AnimationDirection, readonly [Leg, Leg]> = {
  south: [[0.46, 0.38, 0.30, 0.965, 0.135], [0.62, 0.67, 0.70, 0.965, 0.135]],
  north: [[0.39, 0.32, 0.29, 0.97, 0.135], [0.57, 0.65, 0.70, 0.97, 0.135]],
  south_east: [[0.25, 0.18, 0.15, 0.87, 0.115], [0.37, 0.46, 0.51, 0.96, 0.14]],
  south_west: [[0.27, 0.24, 0.19, 0.87, 0.12], [0.39, 0.49, 0.50, 0.96, 0.14]],
  north_east: [[0.24, 0.14, 0.10, 0.89, 0.12], [0.35, 0.44, 0.49, 0.97, 0.14]],
  north_west: [[0.58, 0.56, 0.52, 0.97, 0.14], [0.69, 0.80, 0.84, 0.89, 0.12]],
  // Profile art overlaps the far leg: a shorter far-leg mask avoids moving the near ankle twice.
  east: [[0.45, 0.43, 0.49, 0.97, 0.105], [0.56, 0.54, 0.58, 0.82, 0.065]],
  west: [[0.51, 0.54, 0.50, 0.97, 0.105], [0.40, 0.40, 0.36, 0.86, 0.075]],
};
const clamp = (value: number, low: number, high: number): number => Math.max(low, Math.min(high, value));
const smooth = (value: number): number => { const x = clamp(value, 0, 1); return x * x * (3 - 2 * x); };

function legWeight(x: number, y: number, leg: Leg): number {
  const [hipX, kneeX, ankleX, ankleY, width] = leg;
  const progress = clamp((y - PLAYER_LOCOMOTION_HIP_Y) / (ankleY - PLAYER_LOCOMOTION_HIP_Y), 0, 1);
  const center = progress < 0.55
    ? hipX + (kneeX - hipX) * progress / 0.55
    : kneeX + (ankleX - kneeX) * (progress - 0.55) / 0.45;
  const lateral = 1 - smooth((Math.abs(x - center) - width * 0.55) / (width * 0.85));
  const end = ankleY < 0.93 ? 1 - smooth((y - ankleY) / 0.09) : 1;
  return lateral * end;
}

/** Pure vertex calculation is exported for geometric checks without a GPU or browser clock. */
export function deformPlayerLocomotionVertices(rest: Float32Array, output: Float32Array,
  width: number, height: number, frame: PlayerLocomotionFrame | null,
  direction: AnimationDirection): void {
  if (rest.length !== output.length || rest.length % 2 !== 0 ||
      !Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
    throw new Error("E_PLAYER_LOCOMOTION_GEOMETRY");
  }
  output.set(rest);
  if (!frame?.active || frame.kind !== PLAYER_LOCOMOTION_KIND || !LEGS[direction] ||
      ![frame.phaseRad, frame.amplitude, frame.travelX, frame.travelY].every(Number.isFinite)) return;
  // The narrow profile-view far leg limits stretch before triangles can fold over.
  const amplitude = clamp(frame.amplitude, 0, frame.reducedMotion ? 0.18 : 0.65);
  const phase = frame.reducedMotion ? Math.PI / 2 : frame.phaseRad;
  const travelX = frame.reducedMotion ? 1 : clamp(frame.travelX, -1, 1);
  const travelY = frame.reducedMotion ? 0 : clamp(frame.travelY, -1, 1);
  const legs = LEGS[direction];
  for (let index = 0; index < rest.length; index += 2) {
    const x = rest[index]! / width;
    const y = rest[index + 1]! / height;
    if (y <= PLAYER_LOCOMOTION_HIP_Y) continue;
    const weights = [legWeight(x, y, legs[0]), legWeight(x, y, legs[1])];
    const total = Math.max(1, weights[0]! + weights[1]!);
    let offsetX = 0;
    let offsetY = 0;
    for (let legIndex = 0; legIndex < 2; legIndex++) {
      const leg = legs[legIndex]!;
      const weight = weights[legIndex]! / total;
      const legPhase = phase + legIndex * Math.PI;
      const swing = Math.sin(legPhase);
      const lift = Math.max(0, Math.cos(legPhase));
      const progress = clamp((y - PLAYER_LOCOMOTION_HIP_Y) / (leg[3] - PLAYER_LOCOMOTION_HIP_Y), 0, 1);
      const ankle = smooth(progress);
      const knee = Math.sin(progress * Math.PI) ** 2;
      // Opposite leg phases create actual independent strides. Knee and ankle lifts bend
      // each leg locally; no body translation, mirroring, rotation, or torso bob is used.
      offsetX += weight * (swing * ankle * travelX * 0.055 +
        swing * knee * (legIndex === 0 ? -1 : 1) * 0.006);
      offsetY += weight * (swing * ankle * travelY * 0.018 -
        lift * (ankle * 0.016 + knee * 0.014));
    }
    output[index] = rest[index]! + clamp(offsetX * amplitude, -PLAYER_LOCOMOTION_MAX_X, PLAYER_LOCOMOTION_MAX_X) * width;
    output[index + 1] = rest[index + 1]! + clamp(offsetY * amplitude, -PLAYER_LOCOMOTION_MAX_Y, PLAYER_LOCOMOTION_MAX_Y) * height;
  }
}

/** Owns only its plane geometry; frame textures and atlas sources remain registry-owned. */
export class PlayerLocomotionMesh extends MeshPlane {
  private restPositions: Float32Array;

  constructor(texture: Texture, anchor: readonly [number, number] = [0.5, 0.98392857]) {
    super({ texture, verticesX: PLAYER_LOCOMOTION_VERTICES_X, verticesY: PLAYER_LOCOMOTION_VERTICES_Y });
    this.label = `${PLAYER_LOCOMOTION_KIND}.player_locomotion`;
    this.restPositions = this.geometry.positions.slice();
    this.setFrame(texture, anchor);
  }

  setFrame(texture: Texture, anchor: readonly [number, number]): void {
    if (![anchor[0], anchor[1]].every(Number.isFinite)) throw new Error("E_PLAYER_LOCOMOTION_ANCHOR");
    const resized = texture.width !== this.texture.width || texture.height !== this.texture.height;
    this.texture = texture;
    if (resized) this.restPositions = this.geometry.positions.slice();
    // Equivalent to Sprite.anchor, without changing the authoritative entity position.
    this.pivot.set(texture.width * anchor[0], texture.height * anchor[1]);
  }

  applyLocomotion(frame: PlayerLocomotionFrame | null, direction: AnimationDirection): void {
    deformPlayerLocomotionVertices(this.restPositions, this.geometry.positions,
      this.texture.width, this.texture.height, frame, direction);
    this.geometry.getBuffer("aPosition").update();
  }

  override destroy(options?: DestroyOptions): void {
    if (this.destroyed) return;
    const geometry = this.geometry;
    super.destroy({ ...(typeof options === "object" ? options : {}), texture: false, textureSource: false });
    geometry.destroy(true);
  }
}
