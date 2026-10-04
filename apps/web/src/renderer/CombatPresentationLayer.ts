import { Container, Graphics } from "pixi.js";
import { BASE_PIXELS_PER_METER, projectWorldPoint, type CameraFrame } from "./CameraModel.js";
import type { ActionVfx } from "./ActionVfxModel.js";
import type { CombatFeedbackFrame } from "./CombatFeedbackModel.js";

/** One low-contrast accent on a body; callers may apply it to both sprite and mesh. */
export type CombatActorTint = (actorId: string, color: number | null) => void;
export interface CombatDepthAnchor { targetId?: string; footY: number; zIndex: number }

/** Uses the same world-XZ foot depth as physical actors/props, including a lethal contact without a live sprite. */
export function combatContactDepth(frame: CombatFeedbackFrame, camera: CameraFrame, anchors: readonly CombatDepthAnchor[]): number {
  const target = frame.targetId && anchors.find(anchor => anchor.targetId === frame.targetId);
  if (target) return target.zIndex + .12;
  const footY = projectWorldPoint(frame.positionM, camera).footY;
  const sorted = [...anchors].sort((a,b)=>a.footY-b.footY || a.zIndex-b.zIndex);
  const after = sorted.findIndex(anchor => anchor.footY > footY);
  if (after === 0) return sorted[0]!.zIndex - .2;
  if (after < 0) return (sorted.at(-1)?.zIndex ?? 0) + .2;
  return (sorted[after-1]!.zIndex + sorted[after]!.zIndex) / 2;
}

export class CombatPresentationLayer {
  private readonly marks = new Map<number, Graphics>();
  private readonly tinted = new Set<string>();

  render(frames: readonly CombatFeedbackFrame[], layer: Container, camera: CameraFrame,
    placement: (frame: CombatFeedbackFrame) => number | null, tintActor?: CombatActorTint): void {
    this.clearTints(tintActor);
    const placements = new Map(frames.map(frame => [frame.eventId, placement(frame)]));
    const visible = new Set(frames.filter(frame => frame.outcome !== "rejected" && placements.get(frame.eventId) !== null).map(frame => frame.eventId));
    for (const [id, mark] of this.marks) if (!visible.has(id)) { mark.destroy(); this.marks.delete(id); }
    const scale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
    for (const frame of frames) {
      if (!visible.has(frame.eventId)) continue;
      let mark = this.marks.get(frame.eventId);
      if (!mark) { mark = new Graphics(); mark.label = `temporary_visual:combat:${frame.outcome}:${frame.eventId}`; this.marks.set(frame.eventId, mark); layer.addChild(mark); }
      const point = projectWorldPoint(frame.positionM, camera);
      mark.clear(); mark.scale.set(scale); mark.position.set(point.x, point.y - 18 * scale); mark.zIndex = placements.get(frame.eventId)!;
      mark.alpha = frame.reducedMotion ? 1 : 1 - frame.progress * .55;
      const spread = frame.reducedMotion ? 0 : frame.progress * 5;
      const color = frame.outcome === "player_hurt" ? 0xff8c86 : frame.outcome === "blocked" ? 0x8de8ff : frame.outcome === "absorbed" ? 0xa8b2bf : 0xffe2a3;
      // Dark backing and static outlined symbols remain readable in reduced motion.
      mark.circle(0, 0, 14).fill({ color: 0x101821, alpha: .85 });
      if (frame.outcome === "blocked") {
        mark.moveTo(-10,-10).lineTo(10,-10).lineTo(8,4).lineTo(0,12).lineTo(-8,4).closePath().stroke({color,width:2.5});
        mark.moveTo(-4,0).lineTo(0,4).lineTo(6,-4).stroke({color,width:2});
      } else if (frame.outcome === "absorbed") {
        mark.circle(0,0,10).stroke({color,width:2}).moveTo(-7,7).lineTo(7,-7).stroke({color,width:2});
      } else if (frame.outcome === "player_hurt") {
        mark.moveTo(-11,-4).lineTo(-4,-10).lineTo(1,-2).lineTo(8,-9).lineTo(11,4).lineTo(3,11).closePath().stroke({color,width:2.5});
      } else {
        mark.moveTo(-10-spread,-8-spread).lineTo(-3,-2).moveTo(10+spread,8+spread).lineTo(3,2)
          .moveTo(-10-spread,8+spread).lineTo(-3,2).moveTo(10+spread,-8-spread).lineTo(3,-2).stroke({color,width:3});
        mark.circle(0,0,3).fill(color);
      }
      if (frame.targetId && tintActor && (frame.outcome === "enemy_hit" || frame.outcome === "player_hurt")) {
        tintActor(frame.targetId, frame.outcome === "player_hurt" ? 0xffbbb5 : 0xffe1ac); this.tinted.add(frame.targetId);
      }
    }
  }

  clear(tintActor?: CombatActorTint): void {
    this.clearTints(tintActor);
    for (const mark of this.marks.values()) mark.destroy();
    this.marks.clear();
  }
  private clearTints(tintActor?: CombatActorTint): void {
    for (const actorId of this.tinted) tintActor?.(actorId, null);
    this.tinted.clear();
  }
}

/** Procedural phase cue, not an authored animation or a client collision boundary. */
export function drawActionPresentation(graphic: Graphics, action: ActionVfx, camera: CameraFrame, reducedMotion: boolean): void {
  const center = projectWorldPoint(action.position, camera);
  const unit = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
  const direction = projectWorldPoint({ ...action.position, xM: action.position.xM + action.directionX, zM: action.position.zM + action.directionZ }, camera);
  const angle = Math.atan2(direction.y - center.y, direction.x - center.x);
  const active = action.phase === "active";
  const color = action.kind === "guard" ? 0x8de8ff : action.kind === "pierce" ? 0xffd48c : action.kind === "primaryAttack" ? 0xffe5b8 : 0x82e9ff;
  graphic.clear(); graphic.tint = 0xffffff; graphic.blendMode = "normal"; graphic.scale.set(1);
  graphic.position.set(center.x, center.y);
  graphic.alpha = active ? 1 : action.phase === "windup" ? .65 : .38;
  const line = (points: {x:number;y:number}[], closed = false) => {
    const first = points[0]; if (!first) return;
    graphic.moveTo(first.x-center.x, first.y-center.y);
    for (const point of points.slice(1)) graphic.lineTo(point.x-center.x, point.y-center.y);
    if (closed) graphic.closePath();
    graphic.stroke({color:0x0c1820,width:6*unit,alpha:.9});
    graphic.stroke({color,width:(active?3:1.8)*unit,alpha:1});
  };
  if (action.kind === "guard") {
    // Omnidirectional protection is shown by a shield badge, never a claimed front blocking sector.
    const x = Math.cos(angle)*20*unit, y = Math.sin(angle)*10*unit-26*unit;
    graphic.moveTo(x-11*unit,y-12*unit).lineTo(x+11*unit,y-12*unit).lineTo(x+9*unit,y+3*unit)
      .lineTo(x,y+13*unit).lineTo(x-9*unit,y+3*unit).closePath()
      .fill({color:active?0x134457:0x192b38,alpha:.92}).stroke({color,width:(active?3:1.5)*unit});
    if (active) graphic.moveTo(x-4*unit,y).lineTo(x,y+4*unit).lineTo(x+6*unit,y-5*unit).stroke({color:0xd6f8ff,width:2*unit});
  } else if (action.kind === "pulse") {
    const radius = action.rangeM * (reducedMotion || active ? 1 : action.phase === "windup" ? .78 : 1);
    line(Array.from({length:33},(_,i)=>projectWorldPoint({ ...action.position,
      xM:action.position.xM+Math.cos(i*Math.PI/16)*radius,zM:action.position.zM+Math.sin(i*Math.PI/16)*radius},camera)), true);
  } else if (action.kind === "pierce") {
    const f = {x:action.directionX,z:action.directionZ}, width = action.lineHalfWidthM;
    const p = (along:number,side:number) => projectWorldPoint({...action.position,
      xM:action.position.xM+f.x*along-f.z*side,zM:action.position.zM+f.z*along+f.x*side},camera);
    line([p(0,-width),p(action.rangeM,-width),p(action.rangeM,width),p(0,width)],true);
    if(active)line([p(0,0),p(action.rangeM,0)]);
  } else if (action.kind === "primaryAttack") {
    const reach = Math.max(.1,action.rangeM);
    line(Array.from({length:13},(_,i)=>{
      const a = Math.atan2(action.directionZ,action.directionX)-Math.PI/3+i*Math.PI/18;
      return projectWorldPoint({...action.position,xM:action.position.xM+Math.cos(a)*reach,zM:action.position.zM+Math.sin(a)*reach},camera);
    }));
  } else {
    graphic.moveTo(-Math.cos(angle)*28*unit,-Math.sin(angle)*28*unit).lineTo(-Math.cos(angle)*5*unit,-Math.sin(angle)*5*unit)
      .stroke({color:0x153440,width:10*unit}).stroke({color,width:5*unit});
  }
}
