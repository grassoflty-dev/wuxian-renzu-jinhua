/** World colour treatment only; scene art and admission remain owned by approved assets. */
export interface WorldSkin {
  background: number;
  floorTint: number;
  backPropTint: number;
  dynamicTint: number;
  frontPropTint: number;
  occluderColor: number;
  vfxTint: number;
}

const SKINS: Readonly<Record<string, WorldSkin>> = {
  return_station: {
    background: 0x0b121a, floorTint: 0xd9e8eb, backPropTint: 0xd5e3e7,
    dynamicTint: 0xf3ece0, frontPropTint: 0xd9e8eb, occluderColor: 0x152832, vfxTint: 0xb7eff5,
  },
  grey_hive: {
    background: 0x081922, floorTint: 0xc3dce6, backPropTint: 0xb4d2df,
    dynamicTint: 0xd5eaf0, frontPropTint: 0xc3dce6, occluderColor: 0x07131a, vfxTint: 0x9de9f5,
  },
  mist_harbor: {
    background: 0x101e29, floorTint: 0xc2d4d8, backPropTint: 0xb3cbd2,
    dynamicTint: 0xd8e2df, frontPropTint: 0xb8ced5, occluderColor: 0x10232c, vfxTint: 0xbbe5e2,
  },
  clockworks: {
    background: 0x211811, floorTint: 0xe5d2ad, backPropTint: 0xd1b991,
    dynamicTint: 0xf0d7ab, frontPropTint: 0xdcc39d, occluderColor: 0x24170f, vfxTint: 0xffdc9a,
  },
};

export function worldSkin(worldId: string): WorldSkin {
  const skin = SKINS[worldId];
  if (!skin) throw new Error(`E_WORLD_SKIN_UNKNOWN:${worldId}`);
  return skin;
}

export const OCCLUDER_FADE_MS = 200;

/** Includes the boundary so standing on a wall edge cannot flicker between states. */
export function pointInPolygon(x: number, z: number, polygon: readonly (readonly [number, number])[]): boolean {
  if (!Number.isFinite(x) || !Number.isFinite(z)) return false;
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const [xi, zi] = polygon[i]!;
    const [xj, zj] = polygon[j]!;
    const cross = (x - xi) * (zj - zi) - (z - zi) * (xj - xi);
    if (Math.abs(cross) < 1e-8 && x >= Math.min(xi, xj) && x <= Math.max(xi, xj) &&
        z >= Math.min(zi, zj) && z <= Math.max(zi, zj)) return true;
    if ((zi > z) !== (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi) inside = !inside;
  }
  return inside;
}

interface Fade { from: number; to: number; sinceMs: number }

/** Interpolates a scene occluder to its authored fadeTo over 200 ms. */
export class OccluderFader {
  private readonly fades = new Map<string, Fade>();

  alpha(id: string, fadeTo: number, covered: boolean, nowMs: number): number {
    if (!Number.isFinite(nowMs) || !Number.isFinite(fadeTo) || fadeTo < 0 || fadeTo > 1) {
      throw new Error("E_OCCLUDER_FADE_INPUT");
    }
    const target = covered ? fadeTo : 1;
    let fade = this.fades.get(id);
    if (!fade) {
      fade = { from: target, to: target, sinceMs: nowMs };
      this.fades.set(id, fade);
      return target;
    }
    const elapsed = Math.max(0, nowMs - fade.sinceMs);
    const current = elapsed >= OCCLUDER_FADE_MS
      ? fade.to : fade.from + (fade.to - fade.from) * (elapsed / OCCLUDER_FADE_MS);
    if (fade.to !== target) {
      fade = { from: current, to: target, sinceMs: nowMs };
      this.fades.set(id, fade);
      return current;
    }
    return current;
  }

  forget(id: string): void { this.fades.delete(id); }
  clear(): void { this.fades.clear(); }
}
