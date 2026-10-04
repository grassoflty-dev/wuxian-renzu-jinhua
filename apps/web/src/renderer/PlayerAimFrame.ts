/** Screen coordinates from a frame that has actually reached the canvas. */
export interface PresentedPlayerAimFrame {
  width: number;
  height: number;
  footX: number;
  footY: number;
}

export interface PointerSurfaceRect { left: number; top: number; width: number; height: number }
export interface PointerAimOffset { x: number; y: number }

/** Development tuning: retain the last authoritative facing within eight CSS pixels. */
export const POINTER_AIM_DEADZONE_CSS_PX = 8;

export function pointerOffsetFromPresentedFrame(frame: PresentedPlayerAimFrame, rect: PointerSurfaceRect,
  clientX: number, clientY: number): PointerAimOffset | null {
  if (![frame.width, frame.height, frame.footX, frame.footY, rect.left, rect.top, rect.width, rect.height,
    clientX, clientY].every(Number.isFinite) || frame.width <= 0 || frame.height <= 0 ||
      rect.width <= 0 || rect.height <= 0) return null;
  const footCssX = rect.left + frame.footX * rect.width / frame.width;
  const footCssY = rect.top + frame.footY * rect.height / frame.height;
  const dx = clientX - footCssX;
  const dy = clientY - footCssY;
  if (Math.hypot(dx, dy) <= POINTER_AIM_DEADZONE_CSS_PX) return { x: 0, y: 0 };
  // Undo CSS scaling before applying InputController's inverse oblique projection.
  return { x: dx * frame.width / rect.width, y: dy * frame.height / rect.height };
}
