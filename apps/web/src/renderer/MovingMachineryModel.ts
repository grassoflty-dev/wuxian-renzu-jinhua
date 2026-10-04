import type { WorldSnapshotEnvelope } from "../protocol/types.js";
import { hasLiveRegulator } from "./ClockworksHeatModel.js";

export interface MovingMachineryFrame {
  entityId: string;
  polygonM: Array<[number, number]>;
  warning: boolean;
  reducedMotion: boolean;
}

/** Geometry is the authority's current damaging footprint, never a Web motion simulation. */
export function projectMovingMachinery(snapshot: WorldSnapshotEnvelope, reducedMotion: boolean): MovingMachineryFrame[] {
  if (!hasLiveRegulator(snapshot) || !Array.isArray(snapshot.hazards)) return [];
  const hazards = snapshot.hazards.filter(h => h?.kind === "moving_machinery");
  const ids = new Set<string>();
  for (const h of hazards) {
    if (typeof h.entityId !== "string" || h.entityId.length === 0 || ids.has(h.entityId)) return [];
    ids.add(h.entityId);
  }
  return hazards.flatMap(h => {
    if (h.active !== true || h.phaseActive !== true || !Array.isArray(h.polygonM) || h.polygonM.length < 3 || h.polygonM.length > 128 ||
      !h.polygonM.every(p => Array.isArray(p) && p.length === 2 && p.every(Number.isFinite))) return [];
    const warning = h.warningRemainingMs;
    if (warning !== null && warning !== undefined && (!Number.isSafeInteger(warning) || warning < 0 || warning > 10000)) return [];
    return [{ entityId: h.entityId, polygonM: h.polygonM, warning: (warning ?? 0) > 0, reducedMotion }];
  });
}
