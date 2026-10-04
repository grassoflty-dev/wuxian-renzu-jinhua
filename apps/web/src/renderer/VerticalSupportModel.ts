import type { WorldSnapshotEnvelope, Vec3 } from "../protocol/types.js";
import { assertSupportScene, supportGeometryKey, type SupportPose } from "../protocol/SupportProjection.js";
export interface VerticalSupportDefinition {
  id: string; kind: "moving" | "standing";
  polygon: readonly (readonly [number, number])[];
  lowerM: number; upperM: number; travelMs: number; endpointHoldMs: number;
}
export interface VerticalSupportFrame { id: string; kind: "moving" | "standing"; polygon: VerticalSupportDefinition["polygon"]; heightM: number; rider: boolean }
interface Bounds { x:number;z:number;width:number;depth:number }
interface Plan { worldId:string;sceneId:string;verticalSupports?: readonly VerticalSupportDefinition[];verifiedSourceSha256?:string }
const record=(value:unknown):value is Record<string,unknown>=>typeof value==="object"&&value!==null&&!Array.isArray(value);
const finite=(value:unknown):value is number=>typeof value==="number"&&Number.isFinite(value);
const integer=(value:unknown):value is number=>typeof value==="number"&&Number.isSafeInteger(value);
const id=(value:unknown):value is string=>typeof value==="string"&&/^[A-Za-z0-9_.:-]{1,128}$/.test(value);
function fail(code="E_SUPPORT_SCENE_GEOMETRY"):never {throw new Error(code);}
/** Geometry is used only after the source loader's existing manifest-SHA proof. */
export function parseVerticalSupports(moving:unknown,standing:unknown,bounds:Bounds):VerticalSupportDefinition[] {
  const result:VerticalSupportDefinition[]=[];const ids=new Set<string>();
  for (const [kind,value] of [["moving",moving],["standing",standing]] as const) {
    if(value===undefined)continue;if(!Array.isArray(value))fail();
    for(const raw of value) {
      if(!record(raw)||!id(raw.id)||ids.has(raw.id)||!Array.isArray(raw.polygon)||raw.polygon.length<3||raw.polygon.length>128)fail();
      ids.add(raw.id);const keys=kind==="moving"?["id","polygon","lowerM","upperM","travelMs","endpointHoldMs"]:["id","polygon","heightM"];
      if(Object.keys(raw).length!==keys.length||keys.some(key=>!Object.hasOwn(raw,key)))fail();
      const polygon=raw.polygon.map(point=>{if(!Array.isArray(point)||point.length!==2||!point.every(finite))fail();const[x,z]=point as[number,number];
        if(x<bounds.x||x>bounds.x+bounds.width||z<bounds.z||z>bounds.z+bounds.depth)fail();return[Math.fround(x),Math.fround(z)] as const;});
      let winding=0;
      for(let i=0;i<polygon.length;i++){const a=polygon[i]!,b=polygon[(i+1)%polygon.length]!,c=polygon[(i+2)%polygon.length]!;
        const cross=(b[0]-a[0])*(c[1]-b[1])-(b[1]-a[1])*(c[0]-b[0]);if(!Number.isFinite(cross)||Math.abs(cross)<1e-6||winding!==0&&Math.sign(cross)!==winding)fail();winding=Math.sign(cross);
        for(let j=0;j<polygon.length;j++){if(j===i||j===(i+1)%polygon.length)continue;const p=polygon[j]!;if(((b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0]))*winding<=1e-6)fail();}}
      const low=kind==="moving"?raw.lowerM:raw.heightM,high=kind==="moving"?raw.upperM:raw.heightM;
      if(!finite(low)||!finite(high)||low<0||high>3||low>high)fail();
      let travel=0,hold=0;
      if(kind==="moving"){if(high-low<0.1||!integer(raw.travelMs)||raw.travelMs<250||raw.travelMs>60000||!integer(raw.endpointHoldMs)||raw.endpointHoldMs<0||raw.endpointHoldMs>30000)fail();travel=raw.travelMs;hold=raw.endpointHoldMs;}
      result.push({id:raw.id,kind,polygon,lowerM:Math.fround(low),upperM:Math.fround(high),travelMs:travel,endpointHoldMs:hold});
    }
  }
  if(result.length>64)fail();return result.sort((a,b)=>a.id<b.id?-1:a.id>b.id?1:0);
}
function poseAt(definition:VerticalSupportDefinition,now:number):Omit<SupportPose,"supportId"> {
  const d=definition,f=Math.fround;if(d.kind==="standing")return{phase:"stationary",heightM:d.lowerM,velocityMps:0,phaseElapsedMs:0};
  const t=now%(2*(d.travelMs+d.endpointHoldMs)),distance=f(d.upperM-d.lowerM),speed=f(f(distance*1000)/d.travelMs);
  if(t<d.endpointHoldMs)return{phase:"lower_hold",heightM:d.lowerM,velocityMps:0,phaseElapsedMs:t};
  if(t<d.endpointHoldMs+d.travelMs){const elapsed=t-d.endpointHoldMs;return{phase:"rising",heightM:f(d.lowerM+f(distance*f(elapsed/d.travelMs))),velocityMps:speed,phaseElapsedMs:elapsed};}
  if(t<2*d.endpointHoldMs+d.travelMs)return{phase:"upper_hold",heightM:d.upperM,velocityMps:0,phaseElapsedMs:t-d.endpointHoldMs-d.travelMs};
  const elapsed=t-2*d.endpointHoldMs-d.travelMs;return{phase:"falling",heightM:f(d.upperM-f(distance*f(elapsed/d.travelMs))),velocityMps:-speed,phaseElapsedMs:elapsed};
}
function fullFoot(position:Vec3,radius:number,polygon:VerticalSupportDefinition["polygon"]):boolean {
  let winding=0;
  for(let i=0;i<polygon.length;i++){const a=polygon[i]!,b=polygon[(i+1)%polygon.length]!,dx=b[0]-a[0],dz=b[1]-a[1];
    const cross=dx*(position.zM-a[1])-dz*(position.xM-a[0]);if(winding===0)winding=Math.sign(cross);if(cross*winding<radius*Math.hypot(dx,dz)-0.0001)return false;}
  return true;
}
/** No local animation clock, authority mutation, interpolation or guessed pose. */
export function projectVerticalSupports(plan:Plan|null,snapshot:WorldSnapshotEnvelope):{frames:VerticalSupportFrame[];geometryKey:string|null} {
  const definitions=plan?.verticalSupports??[];const projection=snapshot.protocolVersion===3?snapshot.supportScene:undefined;
  if(definitions.length===0){if(projection)fail("E_SUPPORT_SCENE_UNEXPECTED");return{frames:[],geometryKey:null};}
  if(!plan||snapshot.protocolVersion!==3||!projection||snapshot.worldId!==plan.worldId||snapshot.sceneId!==plan.sceneId)fail("E_SUPPORT_SCENE_REQUIRED");
  const value=assertSupportScene(projection,snapshot);
  if(!plan.verifiedSourceSha256||plan.verifiedSourceSha256!==value.sceneSourceSha256)fail("E_SUPPORT_SCENE_SOURCE_MISMATCH");
  if(value.poses.length!==definitions.length)fail("E_SUPPORT_SCENE_CATALOG_MISMATCH");
  const frames=definitions.map((definition,index)=>{const pose=value.poses[index]!;const expected=poseAt(definition,value.serverTimeMs);
    if(pose.supportId!==definition.id||pose.phase!==expected.phase||pose.phaseElapsedMs!==expected.phaseElapsedMs||Math.abs(pose.heightM-expected.heightM)>0.0001||Math.abs(pose.velocityMps-expected.velocityMps)>0.0001)fail("E_SUPPORT_SCENE_PHASE_MISMATCH");
    if(value.rider.supportId===definition.id&&!fullFoot(value.rider.positionM,value.rider.radiusM,definition.polygon))fail("E_SUPPORT_SCENE_CONTACT_MISMATCH");
    return{id:definition.id,kind:definition.kind,polygon:definition.polygon,heightM:pose.heightM,rider:value.rider.supportId===definition.id};});
  return{frames,geometryKey:supportGeometryKey(value)};
}
