import { Container, Graphics, Text } from "pixi.js";
import { BASE_PIXELS_PER_METER } from "./CameraModel.js";
import { BAIZHI_PLACEHOLDER_HEIGHT_M, BAIZHI_PLACEHOLDER_KIND, BAIZHI_PLACEHOLDER_WIDTH_M,
  type BaizhiPlaceholderFrame } from "./BaizhiPresentation.js";

/** Bottom-center, monochrome development silhouette. Never an attack/AI/party actor. */
export class BaizhiPlaceholder extends Container {
  readonly body = new Graphics();
  readonly nameLabel = new Text({ text: "白芷", style: {
    fontFamily: "sans-serif", fontSize: 14, fill: 0xe3e8e9, align: "center",
    stroke: { color: 0x091720, width: 3 },
  } });

  constructor() {
    super();
    this.label = BAIZHI_PLACEHOLDER_KIND;
    this.eventMode = "none";
    this.interactiveChildren = false;
    this.nameLabel.anchor.set(0.5, 1);
    this.nameLabel.position.set(0, -BAIZHI_PLACEHOLDER_HEIGHT_M * BASE_PIXELS_PER_METER - 5);
    this.addChild(this.body, this.nameLabel);
  }

  applyFrame(frame: BaizhiPlaceholderFrame, depthRank: number): void {
    const ppm = BASE_PIXELS_PER_METER, width = BAIZHI_PLACEHOLDER_WIDTH_M * ppm;
    const height = BAIZHI_PLACEHOLDER_HEIGHT_M * ppm;
    const headRadius = width * 0.4, headY = -height + headRadius;
    const neckY = headY + headRadius * 0.8;
    const markerX = frame.facing.x * headRadius;
    const markerY = headY + frame.facing.y * headRadius;
    this.body.clear().roundRect(-width / 2, neckY, width, -neckY, width / 2)
      .circle(0, headY, headRadius)
      // A small same-colour head marker preserves the frozen -Z direction.
      .poly([markerX - 2, markerY + 2, markerX + 2, markerY + 2,
        frame.facing.x * width / 2, headY + frame.facing.y * width / 2])
      .fill(0xd5dedf);
    this.position.set(frame.screenX, frame.screenY);
    this.scale.set(frame.displayScale);
    this.zIndex = depthRank;
    // No new shadow shape: only the frozen body/marker and player-facing name.
  }

  override destroy(): void {
    if (this.destroyed) return;
    super.destroy({ children: true });
  }
}
