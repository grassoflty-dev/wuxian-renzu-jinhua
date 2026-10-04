import { signalWraithInterference, invalidSignalWraithProjection } from "../protocol/PresentationEventValidation.js";
import type { WorldSnapshotV3 } from "../protocol/types.js";
import { projectWorldPoint, type CameraFrame } from "./CameraModel.js";

export const WORLD_UI_INTERACTION_RANGE_M = 2.5;

const INTERACTION_KINDS = new Set([
  "environment_control",
  "power_console", "door_panel", "terminal", "lockdown_terminal",
  "facility_log", "bio_log_terminal", "gate_b_panel", "extraction_console",
  "scene_transition", "scene_checkpoint", "scene_trigger", "world_gate",
  "mission_terminal", "capability_terminal", "save_rest_terminal",
]);

export interface WorldUiMarker {
  id: string;
  kind: "interaction" | "known_objective" | "signal_region" | "enemy_vital";
  screenX: number;
  screenY: number;
  footY: number;
  label?: string;
}

const ENEMY_VITAL_LABELS = {
  healthy: "Healthy",
  wounded: "Wounded",
  severelyWounded: "Severe",
  critical: "Critical",
} as const;

function distance3d(a: { xM: number; yM: number; zM: number }, b: { xM: number; yM: number; zM: number }): number {
  return Math.hypot(a.xM - b.xM, a.yM - b.yM, a.zM - b.zM);
}

function hasGrantedLocalMap(snapshot: WorldSnapshotV3): boolean {
  return Array.isArray(snapshot.capabilities?.items) && snapshot.capabilities.items.some(item =>
    item?.capabilityId === "information.local_map_i" && item.granted === true && item.selected === true);
}

function hasGrantedEnemyVitals(snapshot: WorldSnapshotV3): boolean {
  return Array.isArray(snapshot.capabilities?.items) && snapshot.capabilities.items.some(item =>
    item?.capabilityId === "information.enemy_vitals_basic" && item.granted === true);
}

function isFinitePosition(position: unknown): position is { xM: number; yM: number; zM: number } {
  if (!position || typeof position !== "object") return false;
  const value = position as { xM?: unknown; yM?: unknown; zM?: unknown };
  return Number.isFinite(value.xM) && Number.isFinite(value.yM) && Number.isFinite(value.zM);
}

function projectEnemyVitals(
  snapshot: WorldSnapshotV3,
  camera: CameraFrame,
  actorPositions?: ReadonlyMap<string, { xM: number; yM: number; zM: number }>,
): WorldUiMarker[] {
  if (!hasGrantedEnemyVitals(snapshot) || !Array.isArray(snapshot.capabilities.enemyVitals)) return [];

  const vitals = snapshot.capabilities.enemyVitals;
  const vitalIdCounts = new Map<string, number>();
  for (const vital of vitals) {
    if (typeof vital?.entityId !== "string" || vital.entityId.trim().length === 0) continue;
    vitalIdCounts.set(vital.entityId, (vitalIdCounts.get(vital.entityId) ?? 0) + 1);
  }

  const actors = Array.isArray(snapshot.actors) ? snapshot.actors : [];
  const markers: WorldUiMarker[] = [];
  for (const vital of vitals) {
    if (typeof vital?.entityId !== "string" || vital.entityId.trim().length === 0 ||
        vitalIdCounts.get(vital.entityId) !== 1) continue;
    if (typeof vital.tier !== "string" || !Object.hasOwn(ENEMY_VITAL_LABELS, vital.tier)) continue;
    const label = ENEMY_VITAL_LABELS[vital.tier as keyof typeof ENEMY_VITAL_LABELS];

    const matches = actors.filter(actor => actor?.entityId === vital.entityId);
    if (matches.length !== 1) continue;
    const actor = matches[0];
    if (!actor || actor.active !== true || actor.actorKind === "player" ||
        actor.entityId === snapshot.player.entityId) continue;

    const position = actorPositions?.has(actor.entityId)
      ? actorPositions.get(actor.entityId)
      : actor.transform?.positionM;
    if (!isFinitePosition(position)) continue;
    const point = projectWorldPoint(position, camera);
    markers.push({ id: `enemy-vital:${actor.entityId}`, kind: "enemy_vital", label,
      screenX: point.x, screenY: point.y, footY: point.footY });
  }
  return markers;
}

/** Missing or malformed authority data never permits exact navigation in this scene. */
function signalRegion(snapshot: WorldSnapshotV3): { precision: "exact" | "area" | "unknown"; positionM?: { xM: number; yM: number; zM: number }; sourceId?: string } {
  if (snapshot.worldId !== "mist_harbor") return { precision: "exact" };
  if (invalidSignalWraithProjection(snapshot)) return { precision: "unknown" };
  const source = signalWraithInterference(snapshot);
  if (source) return { precision: "area", positionM: source.transform.positionM, sourceId: source.entityId };
  if (snapshot.sceneId !== "mh_signal_yard") return { precision: "exact" };
  const matches = Array.isArray(snapshot.hazards)
    ? snapshot.hazards.filter(hazard => hazard?.entityId === "mh_signal_interference_region") : [];
  if (matches.length !== 1) return { precision: "unknown" };
  const hazard = matches[0];
  const positionM = hazard?.transform?.positionM;
  if (hazard?.kind !== "signal_interference_zone" || typeof hazard.active !== "boolean" ||
      !positionM || ![positionM.xM, positionM.yM, positionM.zM].every(Number.isFinite)) {
    return { precision: "unknown" };
  }
  return hazard.active ? { precision: "area", positionM } : { precision: "exact" };
}

/** Projects authorized interactions, map points, and coarse enemy vital tiers into screen markers. */
export function projectWorldUi(
  snapshot: WorldSnapshotV3,
  camera: CameraFrame,
  actorPositions?: ReadonlyMap<string, { xM: number; yM: number; zM: number }>,
): WorldUiMarker[] {
  const player = snapshot.player.transform.positionM;
  const markers: WorldUiMarker[] = [];

  const nearest = snapshot.interactables
    .filter(item => item.active && INTERACTION_KINDS.has(item.kind) &&
      distance3d(player, item.transform.positionM) <= WORLD_UI_INTERACTION_RANGE_M)
    .sort((a, b) => distance3d(player, a.transform.positionM) - distance3d(player, b.transform.positionM) ||
      a.entityId.localeCompare(b.entityId))[0];
  if (nearest) {
    const point = projectWorldPoint(nearest.transform.positionM, camera);
    markers.push({ id: `interaction:${nearest.entityId}`, kind: "interaction", screenX: point.x, screenY: point.y, footY: point.footY });
  }

  const exploredMap = snapshot.capabilities?.exploredMap;
  if (hasGrantedLocalMap(snapshot) && exploredMap?.worldId === snapshot.worldId && Array.isArray(exploredMap.objectives)) {
    const region = signalRegion(snapshot);
    if (region.precision === "area" && region.positionM) {
      const point = projectWorldPoint(region.positionM, camera);
      markers.push({ id: `signal-region:${region.sourceId ?? "mh_signal_interference_region"}`, kind: "signal_region",
        screenX: point.x, screenY: point.y, footY: point.footY });
    } else if (region.precision === "exact") {
      for (const objective of exploredMap.objectives) {
        if (!objective || typeof objective.objectiveId !== "string" || !objective.objectiveId ||
            !objective.positionM || ![objective.positionM.xM, objective.positionM.yM, objective.positionM.zM].every(Number.isFinite)) continue;
        const point = projectWorldPoint(objective.positionM, camera);
        markers.push({ id: `known-objective:${objective.objectiveId}`, kind: "known_objective", screenX: point.x, screenY: point.y, footY: point.footY });
      }
    }
  }

  markers.push(...projectEnemyVitals(snapshot, camera, actorPositions));

  return markers.sort((a, b) => a.footY - b.footY || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
}
