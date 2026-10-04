import { enemyPlaceholderStyle } from "./ClockworksEnemyModel.js";
import type { AnimationDirection, AtlasFrame, RuntimeAsset } from "../assets/AssetRegistry.js";
import type { ActorView, DoorView } from "../protocol/types.js";
import { isolatedActorFrame } from "./ActorFrameLayout.js";

export interface AssetLookup {
  resolveAsset(assetId: string): RuntimeAsset;
}

export interface AtlasPageRequest {
  atlasUrl: string;
  atlasSha256: string;
  atlasPage: number;
  category: RuntimeAsset["category"];
  streamingGroup: string;
}

export interface RendererResourcePlan {
  assets: RuntimeAsset[];
  atlasPages: AtlasPageRequest[];
}

export interface SpriteFrameSelection {
  atlasUrl: string;
  atlasSha256: string;
  atlasPage: number;
  atlasFrame: AtlasFrame;
  anchor: readonly [number, number];
  mode: "static" | "idle-animation" | "static-action-fallback" | "temporary-static-actor";
}

const WORLD_ASSETS: Record<string, readonly string[]> = {
  grey_hive: [
    "runtime2d.grey_hive.floor_tiles.v1",
    "runtime2d.grey_hive.wall_tiles.v1",
    "runtime2d.grey_hive.pipes_cables.v1",
    "runtime2d.grey_hive.clutter.v1",
    "runtime2d.world.grey_hive.sentinel_arena.v1.attempt",
  ],
  mist_harbor: [
    "runtime2d.world.mistharbor.fog_vfx.v1",
    "runtime2d.world.mistharbor.water_edge.v1",
    "runtime2d.world.mistharbor.fog_bank.v1",
    "runtime2d.world.mistharbor.water_edge_stone.v1",
  ],
  clockworks: [
    "runtime2d.world.clockworks.factory_tiles.v1",
    "runtime2d.world.clockworks.pressure_pipes.v1",
    "runtime2d.world.clockworks.furnace.v1",
    "runtime2d.world.clockworks.floor_steel.v1",
    "runtime2d.world.clockworks.pressure_pipe_valve.v1",
  ],
  return_station: [
    "runtime2d.world.returnstation.terminals.v1",
    "runtime2d.world.returnstation.terminal.v1",
  ],
};

const SHARED_VFX = [
  "runtime2d.vfx.dash.v1",
  "runtime2d.vfx.pulse.v1",
  "runtime2d.vfx.guard_pierce.v1",
] as const;

const ACTOR_ALIASES: Record<string, string> = {
  "player.cenyao": "runtime2d.actor.cenyao.base.v1",
  "prop.grey_hive.power_console": "runtime2d.prop.power_console.off_on.v1",
  "enemy.grey_hive.sentinel": "runtime2d.enemy.sentinel.base.v1",
  "enemy.mist_harbor.drowned": "runtime2d.enemy.mistharbor.drowned.v2",
  "enemy.mist_harbor.signal_wraith": "runtime2d.enemy.mistharbor.signal_wraith.v2",
  "enemy.clockworks.forged_guard": "runtime2d.enemy.clockworks.forged_guard.v2",
};

export function actorAssetId(actor: ActorView): string {
  return enemyPlaceholderStyle(actor.entityType)?.assetId ?? (actor.entityType.startsWith("runtime2d.") ? actor.entityType : ACTOR_ALIASES[actor.entityType] ?? actor.entityType);
}

export function doorAssetId(doorId: string): string {
  if (doorId === "gh_gate_a") return "runtime2d.prop.gate_a.closed_open.v1";
  if (doorId === "gh_gate_b") return "runtime2d.prop.gate_b.v1";
  if (doorId.startsWith("runtime2d.")) return doorId;
  throw new Error(`E_RENDERER_UNKNOWN_ASSET:door.${doorId}`);
}

/** Resolves all needed approved assets first and deduplicates atlas page loads. */
export function buildRendererResourcePlan(
  registry: AssetLookup,
  worldId: string,
  actors: readonly ActorView[],
  doors: readonly DoorView[],
): RendererResourcePlan {
  const worldAssets = WORLD_ASSETS[worldId];
  if (!worldAssets) throw new Error(`E_RENDERER_UNKNOWN_WORLD:${worldId}`);
  const assetIds = new Set<string>([...worldAssets, ...SHARED_VFX]);
  for (const actor of actors) {
    if (actor.active) assetIds.add(actorAssetId(actor));
  }
  for (const door of doors) assetIds.add(doorAssetId(door.doorId));

  const assets = [...assetIds].sort().map(assetId => registry.resolveAsset(assetId));
  const pageMap = new Map<string, AtlasPageRequest>();
  const addPage = (asset: RuntimeAsset, atlasUrl: string, atlasSha256: string, atlasPage: number): void => {
    if (!atlasUrl || !atlasSha256 || !Number.isInteger(atlasPage) || atlasPage < 0) {
      throw new Error(`E_RENDERER_ATLAS_PAGE_INVALID:${asset.assetId}`);
    }
    const pageKey = `${atlasUrl}\u0000${atlasPage}`;
    const page = pageMap.get(pageKey);
    if (page && (page.atlasSha256 !== atlasSha256 || page.category !== asset.category || page.streamingGroup !== asset.streamingGroup)) {
      throw new Error(`E_RENDERER_ATLAS_PAGE_CONFLICT:${asset.assetId}`);
    }
    if (!page) pageMap.set(pageKey, {
      atlasUrl, atlasSha256, atlasPage, category: asset.category, streamingGroup: asset.streamingGroup,
    });
  };
  for (const asset of assets) {
    addPage(asset, asset.atlasUrl, asset.atlasSha256, asset.atlasPage);
    for (const frame of asset.animation?.frames ?? []) addPage(asset, frame.atlasUrl, frame.atlasSha256, frame.atlasPage);
  }
  const atlasPages = [...pageMap.values()].sort((left, right) =>
    (left.streamingGroup < right.streamingGroup ? -1 : left.streamingGroup > right.streamingGroup ? 1 : 0) ||
    left.atlasPage - right.atlasPage ||
    (left.atlasUrl < right.atlasUrl ? -1 : left.atlasUrl > right.atlasUrl ? 1 : 0));
  return { assets, atlasPages };
}

/** Uses only the accepted Cenyao idle set; other actions retain the static approved sprite. */
export function selectSpriteFrame(
  asset: RuntimeAsset,
  direction: AnimationDirection,
  actionState: string,
): SpriteFrameSelection {
  if (asset.assetId !== "runtime2d.actor.cenyao.base.v1") {
    const crop = isolatedActorFrame(asset);
    if (crop) return { atlasUrl: asset.atlasUrl, atlasSha256: asset.atlasSha256, atlasPage: asset.atlasPage,
      ...crop, mode: "temporary-static-actor" };
    return { atlasUrl: asset.atlasUrl, atlasSha256: asset.atlasSha256, atlasPage: asset.atlasPage,
      atlasFrame: asset.atlasFrame, anchor: [asset.anchorX, asset.anchorY], mode: "static" };
  }
  const frame = asset.animation?.frames.find(item => item.direction === direction && item.state === "idle");
  if (!frame || !frame.atlasUrl || !frame.atlasSha256) throw new Error(`E_RENDERER_IDLE_FRAME_MISSING:${direction}`);
  return { atlasUrl: frame.atlasUrl, atlasSha256: frame.atlasSha256, atlasPage: frame.atlasPage,
    atlasFrame: frame.atlasFrame, anchor: frame.anchor, mode: actionState === "idle" ? "idle-animation" : "static-action-fallback" };
}
