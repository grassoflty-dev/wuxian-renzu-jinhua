import type { RuntimeAsset } from "../assets/AssetRegistry.js";
import type { WorldSnapshotEnvelope } from "../protocol/types.js";
import type { CameraFrame } from "./CameraModel.js";
import type { ScenePresentationPlan } from "./ScenePresentation.js";
import { projectPropPresentation } from "./PropPresentationModel.js";
import { returnStationArchitecture } from "./ReturnStationArchitecture.js";
import { PLAYER_LOCOMOTION_MAX_X, PLAYER_LOCOMOTION_MAX_Y } from "./PlayerLocomotionMesh.js";

export interface UnitVisualPoint { u: number; v: number }
const TERMINALS = new Set(["world_gate_terminal_art", "mission_terminal_art", "capability_terminal_art",
  "save_rest_terminal_art", "storage_terminal_art"]);
const unit = (x: number, y: number, z: number): UnitVisualPoint => ({ u: x-z, v: (x+z)/2-y });
const fail = (): never => { throw new Error("E_RETURN_STATION_ROOM_FIT"); };

/** Conservative visual envelope, never collision/navigation or a new world scale.
 * The public runtime idle/action fallback and existing bounded
 * temporary locomotion mesh are covered at every room corner, not just spawn.
 */
export function returnStationRoomEnvelope(scene: ScenePresentationPlan, actor: RuntimeAsset,
  snapshot: WorldSnapshotEnvelope): readonly UnitVisualPoint[] {
  if (scene.worldId !== "return_station" || scene.sceneId !== "rs_core_room" ||
      actor.assetId !== "runtime2d.actor.cenyao.base.v1") fail();
  const { x,z,width,depth } = scene.bounds;
  if (![x,z,width,depth,actor.scale].every(Number.isFinite) || width<=0 || depth<=0 || actor.scale<=0) fail();
  const corners = [[x,z],[x+width,z],[x+width,z+depth],[x,z+depth]] as const;
  const points: UnitVisualPoint[] = corners.map(([cx,cz])=>unit(cx,0,cz));
  for (const face of returnStationArchitecture(scene.worldId,scene.sceneId))
    for (const p of face.points) points.push(unit(p.xM,p.yM,p.zM));
  const rect = (base: UnitVisualPoint, left: number, top: number, right: number, bottom: number) => {
    if (![base.u,base.v,left,top,right,bottom].every(Number.isFinite) || right<=left || bottom<=top) fail();
    for (const [u,v] of [[left,top],[right,top],[right,bottom],[left,bottom]])
      points.push({u:base.u+u!,v:base.v+v!});
  };
  const idle = actor.animation?.frames.filter(f=>f.state==="idle");
  if (!idle?.length || new Set(idle.map(f=>f.direction)).size!==8) fail();
  for (const frame of idle!) {
    const w=frame.atlasFrame[2],h=frame.atlasFrame[3],s=actor.scale*1.72/h;
    if (!(w>0&&h>0) || !frame.anchor.every(v=>Number.isFinite(v)&&v>=0&&v<=1)) fail();
    // Global clamps in deformPlayerLocomotionVertices bound every phase/direction.
    const dx=w*PLAYER_LOCOMOTION_MAX_X,dy=h*PLAYER_LOCOMOTION_MAX_Y;
    for (const [cx,cz] of corners) rect(unit(cx,0,cz),(-frame.anchor[0]*w-dx)*s,
      (-frame.anchor[1]*h-dy)*s,((1-frame.anchor[0])*w+dx)*s,((1-frame.anchor[1])*h+dy)*s);
  }
  const terminals=scene.sprites.filter(p=>TERMINALS.has(p.id));
  if (terminals.length!==5 || new Set(terminals.map(p=>p.id)).size!==5) fail();
  for (const p of terminals) {
    const prop=projectPropPresentation(p.asset,snapshot,[]);
    if (!prop) fail();
    const a=prop!.asset,h=prop!.heightM,w=h*a.atlasFrame[2]/a.atlasFrame[3];
    rect(unit(p.position.xM,p.position.yM,p.position.zM),-a.anchorX*w,-a.anchorY*h,
      (1-a.anchorX)*w,(1-a.anchorY)*h);
  }
  return points;
}

/** Exact 5%-per-axis containment. No extra clamp, lead, follow, boss zoom or smoothing.
 * HUD occlusion/readability remains a separate acceptance gate.
 */
export function fitReturnStationRoom(points: readonly UnitVisualPoint[], width: number, height: number): CameraFrame {
  if (![width,height].every(Number.isFinite) || width<=0 || height<=0 || !points.length ||
      points.some(p=>!Number.isFinite(p.u)||!Number.isFinite(p.v))) fail();
  const us=points.map(p=>p.u),vs=points.map(p=>p.v);
  const umin=Math.min(...us),umax=Math.max(...us),vmin=Math.min(...vs),vmax=Math.max(...vs);
  if (!(umax>umin&&vmax>vmin)) fail();
  const L=.05*width,R=.95*width,T=.05*height,B=.95*height;
  const pixelsPerMeter=Math.min((R-L)/(umax-umin),(B-T)/(vmax-vmin));
  const tx=(L+R-pixelsPerMeter*(umin+umax))/2,ty=(T+B-pixelsPerMeter*(vmin+vmax))/2;
  const a=(width/2-tx)/pixelsPerMeter,b=(height/2-ty)/pixelsPerMeter;
  const origin={xM:(a+2*b)/2,yM:0,zM:(2*b-a)/2};
  if (![pixelsPerMeter,origin.xM,origin.zM].every(Number.isFinite) || pixelsPerMeter<=0) fail();
  return {width,height,origin,pixelsPerMeter};
}
