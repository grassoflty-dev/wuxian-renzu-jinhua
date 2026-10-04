import type { ProjectedSceneSprite } from "./ScenePresentation.js";

export interface SceneGlow {
  id: string;
  screenX: number;
  screenY: number;
  radiusX: number;
  radiusY: number;
  color: number;
  alpha: number;
}

interface ApprovedGlowPlacement {
  worldId: string;
  sceneId: string;
  placementId: string;
  assetId: string;
  layer: string;
  offsetY: number;
  radiusX: number;
  radiusY: number;
  color: number;
}

// Exact scene/placement/asset identities keep overlays limited to reviewed props.
const APPROVED_GLOW_PLACEMENTS: readonly ApprovedGlowPlacement[] = [
  { worldId: "grey_hive", sceneId: "gh_lockdown", placementId: "lockdown_terminal_visual", assetId: "runtime2d.prop.lockdown_terminal.v1", layer: "L4_DYNAMIC_PROPS", offsetY: -25, radiusX: 20, radiusY: 11, color: 0x71dbe5 },
  { worldId: "grey_hive", sceneId: "gh_exit", placementId: "exit_extraction_console_approved", assetId: "runtime2d.prop.lockdown_terminal.v1", layer: "L4_DYNAMIC_PROPS", offsetY: -25, radiusX: 20, radiusY: 11, color: 0x71dbe5 },
  { worldId: "grey_hive", sceneId: "gh_bio_isolation", placementId: "bio_log_terminal_approved", assetId: "runtime2d.prop.lockdown_terminal.v1", layer: "L4_DYNAMIC_PROPS", offsetY: -25, radiusX: 20, radiusY: 11, color: 0x71dbe5 },
  { worldId: "grey_hive", sceneId: "gh_power_room", placementId: "power_console_off_on", assetId: "runtime2d.prop.power_console.off_on.v1", layer: "L4_DYNAMIC_PROPS", offsetY: -27, radiusX: 21, radiusY: 12, color: 0x71dbe5 },
  { worldId: "return_station", sceneId: "rs_core_room", placementId: "world_gate_terminal_art", assetId: "runtime2d.world.returnstation.terminals.v1", layer: "L2_BACK_PROPS", offsetY: -27, radiusX: 19, radiusY: 11, color: 0x9bd8d1 },
  { worldId: "return_station", sceneId: "rs_core_room", placementId: "mission_terminal_art", assetId: "runtime2d.world.returnstation.terminal.v1", layer: "L2_BACK_PROPS", offsetY: -26, radiusX: 17, radiusY: 10, color: 0x9bd8d1 },
  { worldId: "return_station", sceneId: "rs_core_room", placementId: "capability_terminal_art", assetId: "runtime2d.world.returnstation.terminal.v1", layer: "L2_BACK_PROPS", offsetY: -26, radiusX: 17, radiusY: 10, color: 0x9bd8d1 },
  { worldId: "return_station", sceneId: "rs_core_room", placementId: "save_rest_terminal_art", assetId: "runtime2d.world.returnstation.terminals.v1", layer: "L2_BACK_PROPS", offsetY: -27, radiusX: 19, radiusY: 11, color: 0x9bd8d1 },
  { worldId: "return_station", sceneId: "rs_core_room", placementId: "storage_terminal_art", assetId: "runtime2d.world.returnstation.terminal.v1", layer: "L2_BACK_PROPS", offsetY: -26, radiusX: 17, radiusY: 10, color: 0x9bd8d1 },
  { worldId: "clockworks", sceneId: "cw_furnace_heart", placementId: "cw_heart_furnace_v1", assetId: "runtime2d.world.clockworks.furnace.v1", layer: "L2_BACK_PROPS", offsetY: -35, radiusX: 34, radiusY: 18, color: 0xffa94a },
  { worldId: "clockworks", sceneId: "cw_furnace_heart", placementId: "cw_heart_furnace_v2", assetId: "runtime2d.world.clockworks.furnace.v2", layer: "L4_DYNAMIC_PROPS", offsetY: -37, radiusX: 37, radiusY: 20, color: 0xffa94a },
];

/** Produces a small overlay only for an exact reviewed placement in a verified scene plan. */
export function projectSceneGlows(
  worldId: string,
  sceneId: string,
  sprites: readonly ProjectedSceneSprite[],
): SceneGlow[] {
  const approved = APPROVED_GLOW_PLACEMENTS.filter(item => item.worldId === worldId && item.sceneId === sceneId);
  const glows: SceneGlow[] = [];
  for (const sprite of sprites) {
    const item = approved.find(candidate => candidate.placementId === sprite.id &&
      candidate.assetId === sprite.asset.assetId && candidate.layer === sprite.layer);
    if (!item) continue;
    glows.push({ id: sprite.id, screenX: sprite.screenX, screenY: sprite.screenY + item.offsetY,
      radiusX: item.radiusX, radiusY: item.radiusY, color: item.color, alpha: 0.12 });
  }
  return glows;
}
