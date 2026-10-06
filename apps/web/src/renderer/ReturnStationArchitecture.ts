import { projectWorldPoint, type CameraFrame } from "./CameraModel.js";
import type { Vec3 } from "../protocol/types.js";

export interface ArchitectureFace { points: readonly Vec3[]; color: number }

/** Only these generated rear panels belong to the room's decorative wall body. */
export function isReturnStationRearWall(worldId: string, sceneId: string, key: string, assetId: string, layer: string): boolean {
  return worldId === "return_station" && sceneId === "rs_core_room" && layer === "L2_BACK_PROPS" &&
    assetId === "runtime2d.grey_hive.wall_tiles.v1" && key.startsWith("scene:approved_facility_walls:panel:0:");
}

/** temporary_visual=true: structural blockout, not final art or gameplay geometry.
 * Lives on the existing north bulkhead; contains no floor, door, collider or interaction.
 * The open ring is physical steel architecture, never the world-network globe UI.
 */
export function returnStationArchitecture(worldId: string, sceneId: string): readonly ArchitectureFace[] {
  if (worldId !== "return_station" || sceneId !== "rs_core_room") return [];
  const faces: ArchitectureFace[] = [];
  const point = (xM: number, yM: number, zM = 0.25): Vec3 => ({ xM, yM, zM });
  const panel = (x: number, y: number, w: number, h: number, color: number) =>
    faces.push({ color, points: [point(x,y),point(x+w,y),point(x+w,y+h),point(x,y+h)] });
  // Rear service spine and buttresses sit wholly in the authored solid wall strip.
  panel(1, 0, 12, .38, 0x263640);
  for (const x of [2.1, 10.5]) {
    panel(x, .2, .85, 4.9, 0x172630);
    panel(x+.12, .35, .15, 4.55, 0x61757e);
    panel(x+.35, .6, .32, 3.3, 0x304650);
    for (const y of [1,2.1,3.2,4.3]) panel(x-.1,y,1.05,.18,0x465963);
    panel(x+.39, 3.6, .12, .3, 0xc69b62);
  }
  // Side service pipes connect the structural supports without inventing obstacles.
  for (const y of [.65, .91]) {
    panel(.65,y,2.15,.12,0x607680);
    panel(10.9,y,3.1,.12,0x607680);
  }
  const supports = faces.splice(0);
  const cx=6.6, cy=2.85, rx=3.5, ry=2.65;
  const ringPoint = (angle: number, inset: number, z=.25) =>
    point(cx+Math.cos(angle)*(rx-inset),cy+Math.sin(angle)*(ry-inset),z);
  // Segmented annulus: the aperture is empty and existing wall/floor art stays intact.
  for (let i=0;i<40;i++) {
    const a=i*Math.PI/20+.008, b=(i+1)*Math.PI/20-.008;
    faces.push({color:i%2?0x354953:0x435a65,points:[ringPoint(a,0),ringPoint(b,0),ringPoint(b,.46),ringPoint(a,.46)]});
    faces.push({color:0x8aabb6,points:[ringPoint(a,.06),ringPoint(b,.06),ringPoint(b,.1),ringPoint(a,.1)]});
    if (i%5!==0) faces.push({color:0xaddce7,points:[ringPoint(a,.39),ringPoint(b,.39),ringPoint(b,.44),ringPoint(a,.44)]});
    if(i%5===0) faces.push({color:0x162630,points:[ringPoint(a-.022,-.09),ringPoint(a+.08,-.09),ringPoint(a+.08,.54),ringPoint(a-.022,.54)]});
  }
  // The renderer places the existing north wall first; ring then rear supports.
  return [...faces, ...supports];
}

/** Use the existing camera projection; never fit the world or change actor scale. */
export function projectArchitectureFace(face: ArchitectureFace, camera: CameraFrame): number[] {
  return face.points.flatMap(point => { const p=projectWorldPoint(point,camera); return [p.x,p.y]; });
}
