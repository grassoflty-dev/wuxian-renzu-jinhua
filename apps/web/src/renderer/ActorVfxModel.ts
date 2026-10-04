import type { AnimationDirection, AnimationFrame, RuntimeAsset } from "../assets/AssetRegistry.js";

export const ACTOR_DIRECTIONS = [
  "north", "north_east", "east", "south_east", "south", "south_west", "west", "north_west",
] as const satisfies readonly AnimationDirection[];

export const ACTOR_STATES = [
  "idle", "walk", "run", "primary_attack", "dash", "pulse", "guard", "pierce",
  "hit", "death", "interact", "context_traversal",
] as const;
export type ActorState = typeof ACTOR_STATES[number];

export interface MissingActorFrame { state: ActorState; direction: AnimationDirection }

/** Completeness is an explicit 12 × 8 matrix, never inferred from one idle sheet. */
export function missingActorFrames(asset: Pick<RuntimeAsset, "animation">): MissingActorFrame[] {
  const keys = new Set((asset.animation?.frames ?? []).map(frame => `${frame.state}\0${frame.direction}`));
  const missing: MissingActorFrame[] = [];
  for (const state of ACTOR_STATES) {
    for (const direction of ACTOR_DIRECTIONS) {
      if (!keys.has(`${state}\0${direction}`)) missing.push({ state, direction });
    }
  }
  return missing;
}

/** Maps only states actually named by the authority protocol or the production contract. */
export function actorStateFromAuthority(actionState: string): ActorState | null {
  if (actionState === "primaryAttack") return "primary_attack";
  if (actionState === "contextTraversal") return "context_traversal";
  return (ACTOR_STATES as readonly string[]).includes(actionState) ? actionState as ActorState : null;
}

export type ActorFrameResult =
  | { status: "ready"; state: ActorState; direction: AnimationDirection; frame: AnimationFrame }
  | { status: "unsupported_state"; authorityState: string }
  | { status: "missing_frame"; state: ActorState; direction: AnimationDirection };

/** Future renderer integration must not present Idle as an authored action animation. */
export function selectActorActionFrame(asset: Pick<RuntimeAsset, "animation">, direction: AnimationDirection,
  authorityState: string): ActorFrameResult {
  const state = actorStateFromAuthority(authorityState);
  if (!state) return { status: "unsupported_state", authorityState };
  const frame = asset.animation?.frames.find(item => item.state === state && item.direction === direction);
  if (!frame || !frame.atlasUrl || !/^[a-f0-9]{64}$/i.test(frame.atlasSha256)) {
    return { status: "missing_frame", state, direction };
  }
  return { status: "ready", state, direction, frame };
}

export const VFX_REQUIREMENTS = [
  { effect: "pulse", shape: "expanding_ring", sourceAssetId: "runtime2d.vfx.pulse.v1", durationMs: null },
  { effect: "guard", shape: "front_arc", arcDegrees: 140, sourceAssetId: null, durationMs: null },
  { effect: "pierce", shape: "linear_ray", sourceAssetId: null, durationMs: null },
  { effect: "dash", shape: "short_trail", sourceAssetId: "runtime2d.vfx.dash.v1", durationMs: 300 },
] as const;
export type VfxEffect = typeof VFX_REQUIREMENTS[number]["effect"];

export interface VfxCoverage { effect: VfxEffect; status: "approved_source_only" | "distinct_source_missing" }

/** Source admission is only one gate; this never asserts a reviewed runtime effect. */
export function vfxSourceCoverage(approvedAssetIds: ReadonlySet<string>): VfxCoverage[] {
  return VFX_REQUIREMENTS.map(requirement => ({
    effect: requirement.effect,
    status: requirement.sourceAssetId && approvedAssetIds.has(requirement.sourceAssetId)
      ? "approved_source_only" : "distinct_source_missing",
  }));
}
