import type { WorldSnapshotEnvelope } from "../protocol/types.js";

// temporary_visual=true: local vector feedback only; this is not production art.
export const PUMP_SCENE_FEEDBACK_TEMPORARY_VISUAL = true;

export type PumpSceneState = "ready" | "draining" | "drained";

export interface PumpSceneFeedbackFrame {
  state: PumpSceneState;
  motionPhase: number;
  machineOffsetX: number;
  flowAlpha: number;
  indicatorColor: number;
  indicatorAlpha: number;
  reducedMotion: boolean;
}

const PUMP_WORLD = "mist_harbor";
const PUMP_SCENE = "mh_pump_station";
const VALID_STATES = new Set<PumpSceneState>(["ready", "draining", "drained"]);
const ANIMATION_PERIOD_MS = 1_350;

/** Pure presentation projection from the authoritative snapshot; it never advances pump state. */
export class PumpSceneFeedbackModel {
  private identity: string | null = null;
  private state: PumpSceneState | null = null;
  private stateEnteredAtMs = 0;

  project(
    snapshot: WorldSnapshotEnvelope,
    nowMs: number,
    reducedMotion: boolean,
  ): PumpSceneFeedbackFrame | null {
    if (snapshot.protocolVersion !== 3 || !Number.isSafeInteger(snapshot.worldEpoch) ||
      snapshot.worldEpoch < 0 || !Number.isFinite(nowMs) ||
      snapshot.worldId !== PUMP_WORLD || snapshot.sceneId !== PUMP_SCENE) {
      this.reset();
      return null;
    }

    const state = snapshot.mistHarborPump?.state;
    if (typeof state !== "string" || !VALID_STATES.has(state as PumpSceneState)) {
      this.reset();
      return null;
    }

    const identity = `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
    if (identity !== this.identity) {
      this.identity = identity;
      this.state = null;
      this.stateEnteredAtMs = nowMs;
    }
    if (state !== this.state) {
      this.state = state as PumpSceneState;
      this.stateEnteredAtMs = nowMs;
    }

    const moving = state === "draining" && !reducedMotion;
    const elapsedMs = Math.max(0, nowMs - this.stateEnteredAtMs);
    const motionPhase = moving ? (elapsedMs % ANIMATION_PERIOD_MS) / ANIMATION_PERIOD_MS : 0;
    const wave = moving ? (Math.sin(motionPhase * Math.PI * 2) + 1) / 2 : 0;
    const amber = state === "ready";
    return {
      state: state as PumpSceneState,
      motionPhase,
      machineOffsetX: moving ? Math.sin(motionPhase * Math.PI * 4) * 0.8 : 0,
      flowAlpha: moving ? 0.44 + wave * 0.24 : 0,
      indicatorColor: amber ? 0xffb34d : 0x61ddcf,
      indicatorAlpha: state === "draining" && !reducedMotion ? 0.66 + wave * 0.28 : 0.92,
      reducedMotion,
    };
  }

  reset(): void {
    this.identity = null;
    this.state = null;
    this.stateEnteredAtMs = 0;
  }
}
