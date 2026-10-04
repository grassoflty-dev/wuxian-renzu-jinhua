import type { VerifiedSceneDefinition } from "./ScenePresentation.js";

// These existing compiler-authored Grey Hive previews duplicate runtime actors.
// Match a concrete spawn and its authored position; an unrelated decorative
// sprite using the same approved source is not suppressed.
const PREVIEW_ASSETS: Readonly<Record<string, string>> = Object.freeze({
  "grey_hive.infected_maintenance_worker": "runtime2d.enemy.infected_maintenance_worker.v1",
  "grey_hive.infected_security": "runtime2d.enemy.infected_security.v1",
});
const PREVIEW_BINDINGS: Readonly<Record<string, Readonly<Record<string, string>>>> = {
  "gh_bio_isolation": {
    "bio_security_01_visual": "gh_bio_security_01",
    "bio_security_02_visual": "gh_bio_security_02"
  },
  "gh_central_shaft": {
    "shaft_worker_01_visual": "gh_shaft_worker_01",
    "shaft_worker_02_visual": "gh_shaft_worker_02",
    "shaft_worker_03_visual": "gh_shaft_worker_03",
    "shaft_security_01_visual": "gh_shaft_security_01",
    "shaft_security_02_visual": "gh_shaft_security_02"
  },
  "gh_deep_decon": {
    "deep_decon_worker_01_visual": "gh_decon_worker_01",
    "deep_decon_worker_02_visual": "gh_decon_worker_02"
  },
  "gh_entry_maintenance": {
    "entry_worker_01_visual": "gh_entry_worker_01",
    "entry_worker_02_visual": "gh_entry_worker_02"
  },
  "gh_gate_a": {
    "gate_security_01_visual": "gh_gate_a_security_01",
    "gate_security_02_visual": "gh_gate_a_security_02"
  },
  "gh_gate_b": {
    "gate_b_security_01_visual": "gh_gate_b_security_01",
    "gate_b_security_02_visual": "gh_gate_b_security_02"
  },
  "gh_lockdown": {
    "lockdown_worker_01_visual": "gh_lockdown_worker_01",
    "lockdown_worker_02_visual": "gh_lockdown_worker_02",
    "lockdown_worker_03_visual": "gh_lockdown_worker_03",
    "lockdown_security_01_visual": "gh_lockdown_security_01",
    "lockdown_security_02_visual": "gh_lockdown_security_02",
    "lockdown_security_03_visual": "gh_lockdown_security_03"
  },
  "gh_power_room": {
    "power_worker_01_visual": "gh_power_worker_01",
    "power_worker_02_visual": "gh_power_worker_02",
    "power_security_01_visual": "gh_power_security_01"
  }
};

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
function position(value: unknown): value is number[] {
  return Array.isArray(value) && value.length === 3 && value.every(Number.isFinite);
}
export function boundActorPreviewIds(scene: VerifiedSceneDefinition): ReadonlyMap<string, string> {
  const bindings = new Map<string, string>();
  if (scene.worldId !== "grey_hive" || !Array.isArray(scene.spawns) || !Array.isArray(scene.presentation.sprites)) return bindings;
  for (const visual of scene.presentation.sprites) {
    if (!record(visual) || typeof visual.id !== "string" || !visual.id.endsWith("_visual") || !position(visual.position)) continue;
    const expectedSpawnId = PREVIEW_BINDINGS[scene.sceneId]?.[visual.id];
    if (!expectedSpawnId) continue;
    const at = visual.position;
    const matches = scene.spawns.filter(spawn => record(spawn) && spawn.kind === "enemy" && spawn.id === expectedSpawnId &&
      typeof spawn.entityType === "string" && PREVIEW_ASSETS[spawn.entityType] === visual.assetId && position(spawn.position) &&
      spawn.position.every((coordinate, index) => coordinate === at[index]));
    if (matches.length === 1) bindings.set(visual.id, (matches[0] as Record<string, unknown>).id as string);
  }
  return bindings;
}
