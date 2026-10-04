export const RENDER_LAYERS = [
  "L0_BACKGROUND",
  "L1_FLOOR",
  "L2_BACK_PROPS",
  "L3_ACTORS",
  "L4_DYNAMIC_PROPS",
  "L5_FRONT_PROPS",
  "L6_OCCLUDERS",
  "L7_VFX",
  "L8_WORLD_UI",
] as const;

export type RenderLayerId = typeof RENDER_LAYERS[number];

export interface DepthItem {
  stableKey: string;
  footY: number;
}

/** Code-unit ordering makes equal-foot-depth ordering independent of locale. */
export function sortByFootDepth<T extends DepthItem>(items: readonly T[]): T[] {
  return [...items].sort((left, right) => left.footY - right.footY ||
    (left.stableKey < right.stableKey ? -1 : left.stableKey > right.stableKey ? 1 : 0));
}

export interface SceneIdentity {
  worldId: string;
  sceneId: string;
  worldEpoch: number;
}

export function sceneIdentityKey(identity: SceneIdentity): string {
  return `${identity.worldId}\u0000${identity.sceneId}\u0000${identity.worldEpoch}`;
}

/** Returns true once for every world, scene, or authority epoch transition. */
export class SceneEpochTracker {
  private currentKey: string | null = null;

  enter(identity: SceneIdentity): boolean {
    const nextKey = sceneIdentityKey(identity);
    if (nextKey === this.currentKey) return false;
    this.currentKey = nextKey;
    return true;
  }

  reset(): void { this.currentKey = null; }
}
