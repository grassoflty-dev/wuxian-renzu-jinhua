import type { EnvironmentHazardProjection, WorldSnapshotEnvelope } from "../protocol/types.js";

export interface EnvironmentHazardFrame extends EnvironmentHazardProjection {
  entityId: string;
  polygonM: Array<[number, number]>;
  label: string;
  color: number;
  alpha: number;
}
const phases = new Set(["warning", "active", "recovery", "suppressed"]);
const tagForKind: Record<string, string> = { heat_zone: "heat", steam_jet: "pressure", decon_mist: "toxin" };
const labels: Record<EnvironmentHazardProjection["tag"], string> = { heat: "热区", pressure: "蒸汽", toxin: "消杀雾" };
const phaseLabels: Record<EnvironmentHazardProjection["phase"], string> = {
  warning: "预警", active: "危险", recovery: "间歇", suppressed: "已抑制",
};
/** Temporary procedural feedback for Rust-projected regions and clocks only. */
export function projectEnvironmentHazards(snapshot: WorldSnapshotEnvelope, reducedMotion: boolean): EnvironmentHazardFrame[] {
  if (snapshot.protocolVersion !== 3 || !counter(snapshot.worldEpoch) || snapshot.worldEpoch === 0 ||
      !counter(snapshot.serverTick) || !Array.isArray(snapshot.hazards)) return [];
  const hazards = snapshot.hazards.filter(h => h?.environment !== undefined && h.environment !== null);
  if (hazards.length > 32 || new Set(hazards.map(h => h.entityId)).size !== hazards.length) return [];
  return hazards.flatMap(hazard => {
    const state = hazard.environment;
    if (!state || typeof hazard.entityId !== "string" || !/^[A-Za-z0-9_.:-]{1,128}$/.test(hazard.entityId) ||
        !phases.has(state.phase) || !Object.hasOwn(labels, state.tag) || tagForKind[hazard.kind] !== state.tag ||
        !counter(state.remainingMs) || state.remainingMs === 0 || state.remainingMs > 60_000 ||
        state.phase === "warning" && state.remainingMs > 10_000 ||
        !counter(state.exposureBps) || state.exposureBps > 10_000 ||
        state.tag !== "heat" && state.exposureBps !== 0 ||
        hazard.active !== (state.phase === "active") || hazard.phaseActive !== true ||
        !validPolygon(hazard.polygonM)) return [];
    const exposure = state.tag === "heat" ? ` · 热量 ${Math.round(state.exposureBps / 100)}%` : "";
    const warningPulse = state.phase === "warning" && !reducedMotion
      ? Math.sin(snapshot.serverTick / 11) * 0.035 : 0;
    return [{ ...state, entityId: hazard.entityId, polygonM: hazard.polygonM.map(point => [...point] as [number, number]),
      label: `${labels[state.tag]} · ${phaseLabels[state.phase]} ${(state.remainingMs / 1000).toFixed(1)}秒${exposure}`,
      color: state.phase === "warning" ? 0xffc36b : state.phase === "active" ? 0xf07545 : state.phase === "suppressed" ? 0x58d7cf : 0x95a8b0,
      alpha: state.phase === "active" ? .25 : state.phase === "warning" ? .14 + warningPulse : .045 }];
  });
}
function counter(value: unknown): value is number { return Number.isSafeInteger(value) && (value as number) >= 0; }
function validPolygon(value: unknown): value is Array<[number, number]> {
  if (!Array.isArray(value) || value.length < 3 || value.length > 128 || !value.every(point =>
      Array.isArray(point) && point.length === 2 && point.every(axis => typeof axis === "number" && Number.isFinite(axis) && axis >= 0 && axis <= 4096))) return false;
  const area = value.reduce((sum, point, index) => {
    const next = value[(index + 1) % value.length]; return sum + point[0] * next[1] - next[0] * point[1];
  }, 0);
  return Math.abs(area) > 1e-6;
}
