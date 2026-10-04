import type { WorldSnapshotEnvelope, WorldSnapshotV3 } from "../protocol/types.js";

export interface ClockworksHeatFrame {
  polygonM: Array<[number, number]>;
  active: boolean;
  warningRemainingMs: number | null;
  reducedMotion: boolean;
}

const ZONE_ID = "cw_regulator_furnace_heat_zone";
const BOSS_ID = "cw_prime_regulator";
const BOSS_TYPE = "runtime2d.enemy.clockworks.forged_guard.v2";

/** Shows only the Rust-authorized furnace hazard in the unique live regulator encounter. */
export function projectClockworksHeat(
  snapshot: WorldSnapshotEnvelope,
  reducedMotion: boolean,
): ClockworksHeatFrame | null {
  if (!hasLiveRegulator(snapshot)) return null;
  if (!Array.isArray(snapshot.hazards)) return null;
  const zones = snapshot.hazards.filter(zone => zone?.entityId === ZONE_ID);
  if (zones.length !== 1) return null;
  const zone = zones[0];
  if (!zone || zone.kind !== "heat_zone" || zone.phaseActive !== true || zone.active !== true && zone.active !== false ||
    !validPolygon(zone.polygonM)) return null;
  const warning = zone.warningRemainingMs;
  if (warning !== undefined && warning !== null && (!counter(warning) || warning > 600)) return null;
  return {
    polygonM: zone.polygonM,
    active: zone.active,
    warningRemainingMs: warning ?? null,
    reducedMotion,
  };
}

function validPolygon(value: unknown): value is Array<[number, number]> {
  return Array.isArray(value) && value.length >= 3 && value.length <= 128 &&
    value.every(point => Array.isArray(point) && point.length === 2 && point.every(axis => Number.isFinite(axis)));
}
function counter(value: unknown): value is number {
  return Number.isSafeInteger(value) && (value as number) >= 0;
}

export function hasLiveRegulator(snapshot: WorldSnapshotEnvelope): snapshot is WorldSnapshotV3 {
  if (snapshot.protocolVersion !== 3 || snapshot.worldId !== "clockworks" ||
    snapshot.sceneId !== "cw_regulator_core" || !counter(snapshot.worldEpoch) || !counter(snapshot.serverTick)) {
    return false;
  }
  const candidates = snapshot.actors.filter(actor => actor?.entityId === BOSS_ID || actor?.entityType === BOSS_TYPE);
  if (candidates.length !== 1 || candidates[0]?.entityId !== BOSS_ID ||
    candidates[0]?.entityType !== BOSS_TYPE || !candidates[0].active) return false;
  return true;
}
