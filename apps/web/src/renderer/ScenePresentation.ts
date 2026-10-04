import { planFacilityMap, type FacilityFloor } from "./FacilityMapModel.js";
import { verifiedSceneSourceSha256 } from "../assets/SceneDefinitionLoader.js";
import { parseVerticalSupports, type VerticalSupportDefinition } from "./VerticalSupportModel.js";
import { parseMovingSurfaces, type MovingSurface } from "./MovingSurfaceModel.js";
import type { RuntimeAsset } from "../assets/AssetRegistry.js";
import type { Vec3 } from "../protocol/types.js";
import { projectWorldPoint, type CameraFrame } from "./CameraModel.js";
import type { RenderLayerId } from "./LayerModel.js";
import type { AssetLookup, AtlasPageRequest } from "./RendererResourcePlan.js";

const WORLD_IDS = new Set(["grey_hive", "mist_harbor", "clockworks", "return_station"]);
// Authored Grey Hive labels currently use the same fixed oblique projection.
const CAMERA_PROFILES = new Set(["oblique_default", "facility_oblique", "facility_power_room", "facility_gate_corridor"]);
const ID_PATTERN = /^[A-Za-z0-9_.:-]{1,128}$/;
const MIST_HARBOR_FOG_VFX = "runtime2d.world.mistharbor.fog_vfx.v1";
const LAYER_TARGETS: Record<string, RenderLayerId> = {
  "visual.background": "L0_BACKGROUND",
  "visual.floor": "L1_FLOOR",
  "visual.floor_detail": "L1_FLOOR",
  "visual.props_back": "L2_BACK_PROPS",
  "visual.props_dynamic": "L4_DYNAMIC_PROPS",
  "visual.foreground": "L5_FRONT_PROPS",
  "visual.occluders": "L6_OCCLUDERS",
  "visual.vfx_markers": "L7_VFX",
};
const SPRITE_LAYERS = new Set([
  "visual.background", "visual.floor", "visual.floor_detail", "visual.props_back", "visual.props_dynamic", "visual.foreground",
]);

export interface SceneBoundsM { x: number; z: number; width: number; depth: number }
/** Render-facing type for a SceneDefinition that has already passed Rust registry validation. */
export interface VerifiedSceneDefinition {
  schemaVersion: 1;
  worldId: string;
  sceneId: string;
  boundsM: SceneBoundsM;
  presentation: Readonly<Record<string, unknown>>;
  readonly [key: string]: unknown;
}
export interface SceneLayer { id: string; zGroup: number; renderLayer: RenderLayerId }
export interface SceneSpritePlacement { id: string; layerId: string; layer: RenderLayerId; zGroup: number; asset: RuntimeAsset; position: Vec3; displaySizeM?: {width:number;height:number} }
export interface SceneOccluder { id: string; polygon: readonly (readonly [number, number])[]; fadeTo: number }
export interface ScenePresentationPlan {
  worldId: string;
  sceneId: string;
  bounds: SceneBoundsM;
  layers: readonly SceneLayer[];
  sprites: readonly SceneSpritePlacement[];
  occluders: readonly SceneOccluder[];
  vfxMarkers: readonly SceneSpritePlacement[];
  assets: readonly RuntimeAsset[];
  atlasPages: readonly AtlasPageRequest[];
  movingSurfaces: readonly MovingSurface[];
  facilityFloor?: FacilityFloor;
  verticalSupports?: readonly VerticalSupportDefinition[];
  verifiedSourceSha256?: string;
}

export interface ProjectedSceneSprite extends SceneSpritePlacement {
  screenX: number;
  screenY: number;
  footY: number;
}
export interface ProjectedSceneOccluder extends SceneOccluder {
  points: readonly (readonly [number, number])[];
  footY: number;
}
export interface ProjectedScenePresentation {
  sprites: ProjectedSceneSprite[];
  occluders: ProjectedSceneOccluder[];
}

/** Owns scene-only Pixi objects so a world/scene/epoch transition can release each exactly once. */
export class SceneResourceTracker {
  private readonly cleanup = new Map<string, () => void>();

  trackIfAbsent(key: string, release: () => void): void {
    if (!this.cleanup.has(key)) this.cleanup.set(key, release);
  }

  forget(key: string): void { this.cleanup.delete(key); }

  dispose(key: string): void {
    const release = this.cleanup.get(key);
    if (!release) return;
    this.cleanup.delete(key);
    release();
  }

  clear(): void {
    const releases = [...this.cleanup.values()];
    this.cleanup.clear();
    for (const release of releases) release();
  }

  get size(): number { return this.cleanup.size; }
}

function invalid(code: string, detail?: string): never {
  throw new Error(detail ? `${code}:${detail}` : code);
}
function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function onlyKeys(value: Record<string, unknown>, allowed: readonly string[]): boolean {
  return Object.keys(value).every(key => allowed.includes(key));
}
function validId(value: unknown): value is string {
  return typeof value === "string" && ID_PATTERN.test(value);
}
function finite(value: unknown): value is number { return typeof value === "number" && Number.isFinite(value); }
function point3(value: unknown, bounds: SceneBoundsM, label: string): Vec3 {
  if (!Array.isArray(value) || value.length !== 3 || !value.every(finite)) invalid("E_SCENE_PRESENTATION_POSITION", label);
  const [xM, yM, zM] = value as [number, number, number];
  if (xM < bounds.x || xM > bounds.x + bounds.width || zM < bounds.z || zM > bounds.z + bounds.depth || yM < 0 || yM > 3) {
    invalid("E_SCENE_PRESENTATION_OUT_OF_BOUNDS", label);
  }
  return { xM, yM, zM };
}
function parseBounds(value: unknown): SceneBoundsM {
  if (!record(value) || !onlyKeys(value, ["x", "z", "width", "depth"])) invalid("E_SCENE_PRESENTATION_BOUNDS");
  const { x, z, width, depth } = value;
  if (!finite(x) || !finite(z) || !finite(width) || !finite(depth) || x < 0 || z < 0 || width <= 0 || depth <= 0 ||
      !finite(x + width) || !finite(z + depth)) invalid("E_SCENE_PRESENTATION_BOUNDS");
  return { x, z, width, depth };
}
function idOf(value: unknown, label: string): string {
  if (!record(value) || !validId(value.id)) invalid("E_SCENE_PRESENTATION_ID", label);
  return value.id;
}
function uniqueId(value: string, ids: Set<string>): void {
  if (ids.has(value)) invalid("E_SCENE_PRESENTATION_DUPLICATE_ID", value);
  ids.add(value);
}
function collectDefinitionIds(scene: Record<string, unknown>, ids: Set<string>): void {
  const arrayFields = ["collision", "spawns", "interactions", "doors", "triggers", "hazards", "checkpoints", "objectives", "transitions", "cameraZones", "movingSupports", "standingDecks"];
  for (const field of arrayFields) {
    if (scene[field] === undefined) continue;
    if (!Array.isArray(scene[field])) invalid("E_SCENE_PRESENTATION_SCENE", field);
    for (const item of scene[field]) uniqueId(idOf(item, field), ids);
  }
  const navigation = scene.navigation;
  if (navigation !== undefined) {
    if (!record(navigation) || !Array.isArray(navigation.nodes)) invalid("E_SCENE_PRESENTATION_SCENE", "navigation");
    for (const node of navigation.nodes) uniqueId(idOf(node, "navigation.nodes"), ids);
  }
  const logic = scene.logic;
  if (logic !== undefined) {
    if (!record(logic) || (logic.traversal !== undefined && !Array.isArray(logic.traversal))) invalid("E_SCENE_PRESENTATION_SCENE", "logic");
    for (const item of (logic.traversal ?? []) as unknown[]) uniqueId(idOf(item, "logic.traversal"), ids);
  }
}

/** Strictly parses the compiler's presentation subset and resolves every placement through approved runtime assets. */
export function planScenePresentation(registry: AssetLookup, value: unknown): ScenePresentationPlan {
  if (!record(value) || value.schemaVersion !== 1 || !validId(value.worldId) || !WORLD_IDS.has(value.worldId) || !validId(value.sceneId)) {
    invalid("E_SCENE_PRESENTATION_SCENE");
  }
  const bounds = parseBounds(value.boundsM);
  const verticalSupports = parseVerticalSupports(value.movingSupports, value.standingDecks, bounds);
  const sourceSha = verifiedSceneSourceSha256(value);
  if (verticalSupports.length && !sourceSha) invalid("E_SUPPORT_SCENE_SOURCE_UNVERIFIED");
  if (!record(value.presentation)) invalid("E_SCENE_PRESENTATION_SECTION");
  const presentation = value.presentation;
  if (presentation.layers !== undefined && !Array.isArray(presentation.layers)) invalid("E_SCENE_PRESENTATION_LAYERS");
  if (presentation.sprites !== undefined && !Array.isArray(presentation.sprites)) invalid("E_SCENE_PRESENTATION_SPRITES");
  if (!onlyKeys(presentation, ["cameraProfile", "backgroundAsset", "layers", "sprites"])) invalid("E_SCENE_PRESENTATION_SECTION");
  if (presentation.cameraProfile !== undefined && !CAMERA_PROFILES.has(presentation.cameraProfile as string)) {
    invalid("E_SCENE_PRESENTATION_CAMERA_PROFILE");
  }
  if (value.occluders !== undefined && !Array.isArray(value.occluders)) invalid("E_SCENE_PRESENTATION_OCCLUDERS");
  if (value.vfxMarkers !== undefined && !Array.isArray(value.vfxMarkers)) invalid("E_SCENE_PRESENTATION_VFX");

  const ids = new Set<string>();
  collectDefinitionIds(value, ids);
  const layers: SceneLayer[] = [];
  const layerById = new Map<string, SceneLayer>();
  for (const raw of (presentation.layers ?? []) as unknown[]) {
    const id = idOf(raw, "presentation.layers");
    uniqueId(id, ids);
    if (!record(raw) || !onlyKeys(raw, ["id", "zGroup"]) || !Object.hasOwn(LAYER_TARGETS, id) ||
        typeof raw.zGroup !== "number" || !Number.isInteger(raw.zGroup) || raw.zGroup < -2147483648 || raw.zGroup > 2147483647) invalid("E_SCENE_PRESENTATION_LAYER", id);
    const layer = { id, zGroup: raw.zGroup as number, renderLayer: LAYER_TARGETS[id]! };
    layers.push(layer);
    layerById.set(id, layer);
  }

  const assetsById = new Map<string, RuntimeAsset>();
  const resolveAsset = (assetId: unknown, label: string): RuntimeAsset => {
    if (!validId(assetId)) invalid("E_SCENE_PRESENTATION_ASSET", label);
    let asset = assetsById.get(assetId);
    if (!asset) {
      try { asset = registry.resolveAsset(assetId); }
      catch { invalid("E_SCENE_PRESENTATION_ASSET_NOT_APPROVED", assetId); }
      if (asset.assetId !== assetId || !asset.atlasUrl || !asset.atlasSha256 || !Number.isSafeInteger(asset.atlasPage) || asset.atlasPage < 0) {
        invalid("E_SCENE_PRESENTATION_ASSET", assetId);
      }
      assetsById.set(assetId, asset);
    }
    return asset;
  };

  const sprites: SceneSpritePlacement[] = [];
  for (const raw of (presentation.sprites ?? []) as unknown[]) {
    const id = idOf(raw, "presentation.sprites");
    uniqueId(id, ids);
    if (!record(raw) || !onlyKeys(raw, ["id", "layer", "assetId", "position"])) invalid("E_SCENE_PRESENTATION_SPRITE", id);
    if (!validId(raw.layer) || !SPRITE_LAYERS.has(raw.layer)) invalid("E_SCENE_PRESENTATION_LAYER", `${id}:${String(raw.layer)}`);
    const layer = layerById.get(raw.layer);
    if (!layer) invalid("E_SCENE_PRESENTATION_LAYER_MISSING", `${id}:${raw.layer}`);
    sprites.push({ id, layerId: raw.layer, layer: layer.renderLayer, zGroup: layer.zGroup,
      asset: resolveAsset(raw.assetId, id), position: point3(raw.position, bounds, id) });
  }

  const occluders: SceneOccluder[] = [];
  for (const raw of (value.occluders ?? []) as unknown[]) {
    const id = idOf(raw, "occluders");
    if (!record(raw) || !onlyKeys(raw, ["id", "polygon", "fadeTo"]) || !Array.isArray(raw.polygon) || raw.polygon.length < 3) invalid("E_SCENE_PRESENTATION_POLYGON", id);
    if (!layerById.has("visual.occluders")) invalid("E_SCENE_PRESENTATION_LAYER_MISSING", "visual.occluders");
    let area2 = 0;
    const vertices = raw.polygon as unknown[];
    const polygon = vertices.map((vertex: unknown, index: number) => {
      if (!Array.isArray(vertex) || vertex.length !== 2 || !vertex.every(finite)) invalid("E_SCENE_PRESENTATION_POLYGON", `${id}:${index}`);
      const [x, z] = vertex as [number, number];
      if (x < bounds.x || x > bounds.x + bounds.width || z < bounds.z || z > bounds.z + bounds.depth) invalid("E_SCENE_PRESENTATION_OUT_OF_BOUNDS", `${id}:${index}`);
      const next = vertices[(index + 1) % vertices.length] as [number, number];
      area2 += x * next[1] - next[0] * z;
      return [x, z] as const;
    });
    if (Math.abs(area2) <= 1e-9) invalid("E_SCENE_PRESENTATION_POLYGON", id);
    const fadeTo = raw.fadeTo === undefined ? 0.25 : raw.fadeTo;
    if (!finite(fadeTo) || fadeTo < 0 || fadeTo > 1) invalid("E_SCENE_PRESENTATION_OCCLUDER", id);
    uniqueId(id, ids);
    occluders.push({ id, polygon, fadeTo });
  }

  const vfxMarkers: SceneSpritePlacement[] = [];
  for (const raw of (value.vfxMarkers ?? []) as unknown[]) {
    const id = idOf(raw, "vfxMarkers");
    if (!record(raw) || !onlyKeys(raw, ["id", "assetId", "position"])) invalid("E_SCENE_PRESENTATION_VFX", id);
    if (!layerById.has("visual.vfx_markers")) invalid("E_SCENE_PRESENTATION_LAYER_MISSING", "visual.vfx_markers");
    uniqueId(id, ids);
    const asset = resolveAsset(record(raw) ? raw.assetId : undefined, id);
    if (asset.kind !== "VFX") invalid("E_SCENE_PRESENTATION_VFX_ASSET", id);
    const layer = layerById.get("visual.vfx_markers")!;
    const renderLayer = value.worldId === "mist_harbor" && layer.id === "visual.vfx_markers" &&
      asset.assetId === MIST_HARBOR_FOG_VFX && asset.kind === "VFX" ? "L4_DYNAMIC_PROPS" : layer.renderLayer;
    vfxMarkers.push({ id, layerId: layer.id, layer: renderLayer, zGroup: layer.zGroup, asset,
      position: point3(record(raw) ? raw.position : undefined, bounds, id) });
  }

  let background: SceneSpritePlacement | undefined;
  if (presentation.backgroundAsset !== undefined) {
    const layer = layerById.get("visual.background");
    if (!layer) invalid("E_SCENE_PRESENTATION_LAYER_MISSING", "visual.background");
    const id = "__scene_background";
    uniqueId(id, ids);
    background = { id, layerId: layer.id, layer: layer.renderLayer, zGroup: layer.zGroup,
      asset: resolveAsset(presentation.backgroundAsset, id),
      position: { xM: bounds.x + bounds.width / 2, yM: 0, zM: bounds.z + bounds.depth / 2 } };
  }

  const assets = [...assetsById.values()].sort((a, b) => a.assetId < b.assetId ? -1 : a.assetId > b.assetId ? 1 : 0);
  const atlasPageMap = new Map<string, AtlasPageRequest>();
  for (const asset of assets) {
    const key = `${asset.atlasUrl}\0${asset.atlasPage}`;
    const prior = atlasPageMap.get(key);
    if (prior && (prior.atlasSha256 !== asset.atlasSha256 || prior.category !== asset.category || prior.streamingGroup !== asset.streamingGroup)) {
      invalid("E_SCENE_PRESENTATION_ATLAS_CONFLICT", asset.assetId);
    }
    if (!prior) atlasPageMap.set(key, { atlasUrl: asset.atlasUrl, atlasSha256: asset.atlasSha256,
      atlasPage: asset.atlasPage, category: asset.category, streamingGroup: asset.streamingGroup });
  }
  const atlasPages = [...atlasPageMap.values()].sort((a, b) => a.atlasPage - b.atlasPage || (a.atlasUrl < b.atlasUrl ? -1 : a.atlasUrl > b.atlasUrl ? 1 : 0));
  const facility = planFacilityMap(value.worldId, value.sceneId, bounds, sprites, value.collision);
  return { worldId: value.worldId, sceneId: value.sceneId, bounds, layers, sprites: background ? [background, ...facility.sprites] : facility.sprites,
    ...(facility.floor ? {facilityFloor: facility.floor} : {}),
    occluders, vfxMarkers, assets, atlasPages, movingSurfaces: parseMovingSurfaces(value.terrainRegions, bounds),
    ...(verticalSupports.length ? { verticalSupports, verifiedSourceSha256: sourceSha! } : {}) };
}

/** Applies the fixed oblique camera projection to all static scene geometry. */
export function projectScenePresentation(plan: ScenePresentationPlan, camera: CameraFrame): ProjectedScenePresentation {
  const sprites = [...plan.sprites, ...plan.vfxMarkers].map(item => {
    const point = projectWorldPoint(item.position, camera);
    return { ...item, screenX: point.x, screenY: point.y, footY: point.footY };
  }).sort((a, b) => (a.layer < b.layer ? -1 : a.layer > b.layer ? 1 : 0) ||
    (a.layer === "L1_FLOOR" ? a.zGroup - b.zGroup || a.footY - b.footY : a.footY - b.footY || a.zGroup - b.zGroup) ||
    (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  const occluders = plan.occluders.map(item => {
    const points = item.polygon.map(([xM, zM]) => {
      const point = projectWorldPoint({ xM, yM: camera.origin.yM, zM }, camera);
      return [point.x, point.y] as const;
    });
    return { ...item, points, footY: points.reduce((sum, point) => sum + point[1], 0) / points.length };
  }).sort((a, b) => a.footY - b.footY || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
  return { sprites, occluders };
}
