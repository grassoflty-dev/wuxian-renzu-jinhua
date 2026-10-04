import type { SceneIdentity } from "./LayerModel.js";
import { sceneIdentityKey } from "./LayerModel.js";
import type { ProjectedSceneSprite } from "./ScenePresentation.js";

export type FogDepth = "background" | "mid" | "foreground";

export interface FogOffset { x: number; y: number; depth: FogDepth }
export interface FogCoverage { background: boolean; mid: boolean; foreground: boolean }
export interface FogFrame { offsets: ReadonlyMap<string, FogOffset>; coverage: FogCoverage }

const FOG_BANK = "runtime2d.world.mistharbor.fog_bank.v1";
const FOG_VFX = "runtime2d.world.mistharbor.fog_vfx.v1";
const MOTION: Record<FogDepth, { amplitudeX: number; amplitudeY: number; periodMs: number }> = {
  background: { amplitudeX: 3, amplitudeY: 1, periodMs: 19000 },
  mid: { amplitudeX: 5, amplitudeY: 2, periodMs: 12000 },
  foreground: { amplitudeX: 7, amplitudeY: 3, periodMs: 7500 },
};

/** Identifies only approved Mist Harbor fog placements and their authored source band. */
export function fogDepth(worldId: string, sprite: ProjectedSceneSprite): FogDepth | null {
  if (worldId !== "mist_harbor") return null;
  if (sprite.asset.assetId === FOG_BANK && sprite.layerId === "visual.props_back" && sprite.layer === "L2_BACK_PROPS") {
    return "background";
  }
  if (sprite.asset.assetId === FOG_VFX && sprite.layerId === "visual.vfx_markers" && sprite.layer === "L4_DYNAMIC_PROPS") {
    return "mid";
  }
  if (sprite.asset.assetId === FOG_BANK && sprite.layerId === "visual.foreground" && sprite.layer === "L5_FRONT_PROPS") {
    return "foreground";
  }
  return null;
}

function stablePhase(id: string): number {
  let hash = 2166136261;
  for (let index = 0; index < id.length; index++) {
    hash = Math.imul(hash ^ id.charCodeAt(index), 16777619);
  }
  return (hash >>> 0) / 0x100000000 * Math.PI * 2;
}

/** Pure screen-space motion; neither scene positions nor gameplay perception are changed. */
export class FogLayerModel {
  private identityKey: string | null = null;
  private startedAtMs = 0;

  reset(): void {
    this.identityKey = null;
    this.startedAtMs = 0;
  }

  project(identity: SceneIdentity, sprites: readonly ProjectedSceneSprite[], nowMs: number, reducedMotion: boolean): FogFrame {
    const key = sceneIdentityKey(identity);
    if (key !== this.identityKey || !Number.isFinite(this.startedAtMs)) {
      this.identityKey = key;
      this.startedAtMs = nowMs;
    }
    const elapsedRaw = nowMs - this.startedAtMs;
    const elapsedMs = Number.isFinite(elapsedRaw) ? Math.max(0, elapsedRaw) : 0;
    const offsets = new Map<string, FogOffset>();
    const coverage = { background: false, mid: false, foreground: false };
    for (const sprite of sprites) {
      const depth = fogDepth(identity.worldId, sprite);
      if (!depth) continue;
      coverage[depth] = true;
      const config = MOTION[depth];
      const phase = stablePhase(sprite.id);
      // A phase-relative oscillation starts at the authored point and stays bounded forever.
      const angle = (elapsedMs % config.periodMs) / config.periodMs * Math.PI * 2;
      offsets.set(sprite.id, {
        x: reducedMotion ? 0 : config.amplitudeX * (Math.sin(phase + angle) - Math.sin(phase)),
        y: reducedMotion ? 0 : config.amplitudeY * (Math.cos(phase + angle) - Math.cos(phase)),
        depth,
      });
    }
    return { offsets, coverage };
  }
}
