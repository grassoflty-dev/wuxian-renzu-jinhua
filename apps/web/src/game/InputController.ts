import type { ActionCommand, ActionKind, InputState } from "../protocol/types.js";

function normal(x: number, z: number): { x: number; z: number } {
  const length = Math.hypot(x, z);
  return length > 0 ? { x: x / length, z: z / length } : { x: 0, z: 0 };
}

/** Inverse of the fixed oblique projection: W always moves up on screen. */
export function screenDirectionToWorld(screenX: number, screenY: number): { x: number; z: number } {
  const x = screenX / 96 + screenY / 48;
  const z = screenY / 48 - screenX / 96;
  return normal(x, z);
}

export class InputController {
  private readonly held = new Set<string>();
  private pointerX = 0;
  // No pointer has been observed yet: preserve the authority's existing facing.
  private pointerY = 0;
  private seq = 0;
  private requestId = 0;

  keyDown(key: string, worldEpoch = 0, clientTimeMs = Date.now()): ActionCommand | null {
    const normalized = key.toLowerCase();
    const repeated = this.held.has(normalized);
    this.held.add(normalized);
    const action: Record<string, ActionKind> = {
      j: "primaryAttack", shift: "dash", q: "pulse", e: "guardStart", r: "pierce", f: "interact", " ": "contextTraversal",
    };
    const kind = action[normalized];
    return kind && !repeated ? { protocolVersion: 2, worldEpoch, requestId: ++this.requestId, clientTimeMs, kind } : null;
  }

  keyUp(key: string, worldEpoch = 0, clientTimeMs = Date.now()): ActionCommand | null {
    const normalized = key.toLowerCase();
    const wasHeld = this.held.delete(normalized);
    return wasHeld && normalized === "e"
      ? { protocolVersion: 2, worldEpoch, requestId: ++this.requestId, clientTimeMs, kind: "guardEnd" }
      : null;
  }

  /** Pointer presses are discrete edges, independent of a held keyboard J. */
  primaryAttack(worldEpoch: number, clientTimeMs: number): ActionCommand {
    return { protocolVersion: 2, worldEpoch, requestId: ++this.requestId, clientTimeMs, kind: "primaryAttack" };
  }

  /** Clears held movement and emits the authoritative release edge for a held guard. */
  releaseAll(worldEpoch = 0, clientTimeMs = Date.now()): ActionCommand[] {
    const guardEnd = this.keyUp("e", worldEpoch, clientTimeMs);
    this.held.clear();
    return guardEnd ? [guardEnd] : [];
  }

  isHeld(key: string): boolean { return this.held.has(key.toLowerCase()); }
  pointer(screenXFromCenter: number, screenYFromCenter: number): void {
    if (Number.isFinite(screenXFromCenter) && Number.isFinite(screenYFromCenter)) {
      this.pointerX = screenXFromCenter;
      this.pointerY = screenYFromCenter;
    }
  }

  aim(): Pick<InputState, "aimX" | "aimZ"> {
    const aim = screenDirectionToWorld(this.pointerX, this.pointerY);
    return { aimX: aim.x, aimZ: aim.z };
  }

  sample(worldEpoch: number, clientTimeMs: number): InputState {
    const sx = Number(this.held.has("d")) - Number(this.held.has("a"));
    const sy = Number(this.held.has("s")) - Number(this.held.has("w"));
    const move = screenDirectionToWorld(sx, sy);
    return {
      protocol: "continuous-input", protocolVersion: 2, worldEpoch, seq: ++this.seq, clientTimeMs,
      moveX: move.x, moveZ: move.z, ...this.aim(),
    };
  }

  reset(): void { this.held.clear(); this.seq = 0; this.requestId = 0; }
}
