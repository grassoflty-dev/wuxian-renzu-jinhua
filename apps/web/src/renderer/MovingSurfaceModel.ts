import type { WorldSnapshotEnvelope } from "../protocol/types.js";

export interface MovingSurface {
  id: string;
  polygon: readonly (readonly [number, number])[];
  velocityMps: readonly [number, number];
}
export interface MovingSurfaceFrame extends MovingSurface {
  temporary_visual: true;
  arrows: readonly { x: number; z: number; dx: number; dz: number }[];
}
interface Bounds { x: number; z: number; width: number; depth: number }
const finite = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value);
const record = (value: unknown): value is Record<string, unknown> => !!value && typeof value === "object" && !Array.isArray(value);

/** Read-only presentation of an already hash-verified, Rust-admitted scene. */
export function parseMovingSurfaces(value: unknown, bounds: Bounds): MovingSurface[] {
  if (value === undefined) return [];
  if (!Array.isArray(value)) throw new Error("E_SCENE_SURFACE_REGION");
  const ids = new Set<string>();
  return value.flatMap(raw => {
    if (!record(raw)) throw new Error("E_SCENE_SURFACE_REGION");
    if (raw.tag !== "conveyor") return [];
    if (typeof raw.id !== "string" || !/^[A-Za-z0-9_.:-]{1,128}$/.test(raw.id) || ids.has(raw.id) ||
        Object.keys(raw).some(key => !["id","tag","polygon","surfaceVelocityMps"].includes(key))) {
      throw new Error("E_SCENE_SURFACE_ID");
    }
    ids.add(raw.id);
    const velocity = raw.surfaceVelocityMps;
    if (!Array.isArray(velocity) || velocity.length !== 2 || !velocity.every(finite) ||
        Math.hypot(...velocity) < 0.1 || Math.hypot(...velocity) > 4) throw new Error("E_SCENE_SURFACE_VELOCITY");
    if (!Array.isArray(raw.polygon) || raw.polygon.length < 3 || raw.polygon.length > 128) throw new Error("E_SCENE_SURFACE_POLYGON");
    const polygon = raw.polygon.map(point => {
      if (!Array.isArray(point) || point.length !== 2 || !point.every(finite)) throw new Error("E_SCENE_SURFACE_POLYGON");
      const [x,z] = point as [number,number];
      if (x < bounds.x || x > bounds.x + bounds.width || z < bounds.z || z > bounds.z + bounds.depth) throw new Error("E_SCENE_SURFACE_POLYGON");
      return [x,z] as const;
    });
    const area = polygon.reduce((sum,[x,z],index) => {
      const next = polygon[(index+1)%polygon.length]!; return sum + x*next[1] - next[0]*z;
    },0);
    if (Math.abs(area)<1e-9) throw new Error("E_SCENE_SURFACE_POLYGON");
    return [{ id: raw.id, polygon, velocityMps: velocity as [number,number] }];
  });
}

function inside(x: number,z: number,polygon: MovingSurface["polygon"]): boolean {
  let hit=false;
  for (let i=0;i<polygon.length;i++) {
    const a=polygon[i]!,b=polygon[(i+1)%polygon.length]!;
    if ((a[1]>z)!==(b[1]>z) && x<(b[0]-a[0])*(z-a[1])/(b[1]-a[1])+a[0]) hit=!hit;
  }
  return hit;
}

/** Tick-driven arrows freeze during pause; no movement, HP or collision authority. */
export function projectMovingSurfaces(
  plan: {worldId:string;sceneId:string;movingSurfaces?: readonly MovingSurface[]}|null,
  snapshot: WorldSnapshotEnvelope, reducedMotion: boolean,
): MovingSurfaceFrame[] {
  if (!plan || snapshot.protocolVersion!==3 || snapshot.worldId!==plan.worldId || snapshot.sceneId!==plan.sceneId ||
      !Number.isSafeInteger(snapshot.worldEpoch) || snapshot.worldEpoch<1 ||
      !Number.isSafeInteger(snapshot.serverTick) || snapshot.serverTick<0) return [];
  return (plan.movingSurfaces??[]).map(surface => {
    const speed=Math.hypot(...surface.velocityMps);
    const dx=surface.velocityMps[0]/speed,dz=surface.velocityMps[1]/speed;
    const offset=reducedMotion?0:(snapshot.serverTick/60*speed)%1.5;
    const xs=surface.polygon.map(p=>p[0]),zs=surface.polygon.map(p=>p[1]);
    const arrows: {x:number;z:number;dx:number;dz:number}[]=[];
    const minX=Math.min(...xs), minZ=Math.min(...zs);
    const width=Math.max(...xs)-minX, depth=Math.max(...zs)-minZ;
    const columns=Math.min(16,Math.max(1,Math.ceil(width/1.5)));
    const rows=Math.min(8,Math.max(1,Math.ceil(depth/1.5)));
    for (let column=0;column<columns;column++) {
      for (let row=0;row<rows;row++) {
        const px=minX+(column+0.5)*width/columns+dx*offset;
        const pz=minZ+(row+0.5)*depth/rows+dz*offset;
        if (inside(px,pz,surface.polygon)) arrows.push({x:px,z:pz,dx,dz});
      }
    }
    return {...surface,temporary_visual:true,arrows};
  });
}
