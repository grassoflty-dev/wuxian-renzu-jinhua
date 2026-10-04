import { assertSentinelEncounter, sentinelGeometryKey, SENTINEL_ID, type SentinelEncounter } from "../protocol/SentinelEncounter.js";
import { greyHiveBeaconSpriteVisible } from "../game/GreyHiveBeacon.js";
import { isMappedProp, projectPropPresentation } from "./PropPresentationModel.js";
import { sharesWorldDepth, worldDepthRanks } from "./WorldDepthModel.js";
import { facilityFloorGeometry } from "./FacilityFloorGeometry.js";
import type { FacilityFloor } from "./FacilityMapModel.js";
import { validatedSignalPerception, isNativeSignalWraith } from "../protocol/PresentationEventValidation.js";
import { projectVerticalSupports, type VerticalSupportFrame } from "./VerticalSupportModel.js";
import { validatedSwarmMembers } from "../protocol/PresentationEventValidation.js";
import { projectMovingSurfaces, type MovingSurfaceFrame } from "./MovingSurfaceModel.js";
import { ClockworksEnemyModel, enemyPlaceholderStyle, enemyWarningGroundPoints, type ClockworksEnemyFrame } from "./ClockworksEnemyModel.js";
import { projectWardenEncounter, type WardenEncounterFrame } from "./WardenEncounterModel.js";
import { Application, Assets, Container, Graphics, Mesh, MeshGeometry, Rectangle, Sprite, Text, Texture } from "pixi.js";
import { AssetRegistry, type RuntimeAsset } from "../assets/AssetRegistry.js";
import type { ActorView, PresentationEvent, SoundCueEvent, Vec3, WorldSnapshotEnvelope, WorldSnapshotV3 } from "../protocol/types.js";
import { CameraRig } from "./CameraRig.js";
import { BASE_PIXELS_PER_METER, projectWorldPoint, resolveSpriteDisplayScale, viewportPixelsPerMeter, type CameraFrame } from "./CameraModel.js";
import { BossCameraModel } from "./BossCameraModel.js";
import { RENDER_LAYERS, SceneEpochTracker, type RenderLayerId } from "./LayerModel.js";
import { normalizeRenderSnapshot, type RenderSnapshot } from "./RenderSnapshot.js";
import { actorAssetId, buildRendererResourcePlan, doorAssetId, selectSpriteFrame, type SpriteFrameSelection } from "./RendererResourcePlan.js";
import { screenFacingDirection } from "./RenderSnapshot.js";
import { planScenePresentation, projectScenePresentation, SceneResourceTracker, type ScenePresentationPlan, type VerifiedSceneDefinition } from "./ScenePresentation.js";
import { projectSceneGlows, type SceneGlow } from "./SceneGlowModel.js";
import { OccluderFader, pointInPolygon, worldSkin } from "./WorldSkin.js";
import { projectWorldUi, type WorldUiMarker } from "./WorldUiModel.js";
import { ActionVfxModel, type ActionVfx } from "./ActionVfxModel.js";
import { CombatFeedbackModel } from "./CombatFeedbackModel.js";
import { CombatPresentationLayer, combatContactDepth, drawActionPresentation, type CombatDepthAnchor } from "./CombatPresentationLayer.js";
import { SoundCueModel, type VisibleSoundCue } from "./SoundCueModel.js";
import { FogLayerModel } from "./FogLayerModel.js";
import { WaterRippleModel, type WaterRippleFrame } from "./WaterRippleModel.js";
import { PumpSceneFeedbackModel, type PumpSceneFeedbackFrame } from "./PumpSceneFeedbackModel.js";
import { projectEnvironmentHazards, type EnvironmentHazardFrame } from "./EnvironmentHazardModel.js";
import { SentinelTelegraphModel, type SentinelTelegraphFrame } from "./SentinelTelegraphModel.js";
import {
  ClockworksPressureWaveModel,
  projectPrimeRegulatorBossMark,
  type ClockworksPressureWaveFrame,
  type PrimeRegulatorBossMarkFrame,
} from "./ClockworksPressureWaveModel.js";
import { projectMovingMachinery, type MovingMachineryFrame } from "./MovingMachineryModel.js";
import { projectClockworksHeat, type ClockworksHeatFrame } from "./ClockworksHeatModel.js";
import { PresentationSurface } from "./PresentationSurface.js";
import { pointerOffsetFromPresentedFrame, type PresentedPlayerAimFrame, type PointerAimOffset,
  type PointerSurfaceRect } from "./PlayerAimFrame.js";
import { boundActorPreviewIds } from "./ActorPresentationBindings.js";
import { PlayerLocomotionModel } from "./PlayerLocomotionModel.js";
import { PlayerLocomotionMesh } from "./PlayerLocomotionMesh.js";

// Pixi Assets is a process-wide cache, so pending and resident leases must also
// span renderer instances during replacement, not merely one scene generation.
const ATLAS_LOAD_OWNERS = new Map<string, Set<symbol>>();
const ATLAS_RESIDENT_OWNERS = new Map<string, Set<symbol>>();
const ATLAS_PENDING_UNLOADS = new Map<string, Promise<void>>();

function retireUnownedAtlas(url: string): Promise<void> | null {
  if (ATLAS_LOAD_OWNERS.get(url)?.size || ATLAS_RESIDENT_OWNERS.get(url)?.size) return null;
  const pending = ATLAS_PENDING_UNLOADS.get(url);
  if (pending) return pending;
  const unloading = Promise.resolve().then(() => Assets.unload(url));
  ATLAS_PENDING_UNLOADS.set(url, unloading);
  return unloading.finally(() => {
    if (ATLAS_PENDING_UNLOADS.get(url) === unloading) ATLAS_PENDING_UNLOADS.delete(url);
  });
}

interface SpriteRecord {
  sprite: Sprite;
  frameTexture: Texture;
  frameKey: string;
  assetId: string;
  layer: RenderLayerId;
}

interface DesiredSprite {
  key: string;
  asset: RuntimeAsset;
  position: Vec3;
  layer: RenderLayerId;
  footY: number;
  zGroup: number;
  screenX: number;
  screenY: number;
  displayScale: number;
  displayScaleY?: number;
  cutout?: readonly (readonly [number,number])[];
  frameSelection?: SpriteFrameSelection;
}

interface WorldUiRecord {
  container: Container;
  label?: Text;
}

export type RenderCommitErrorCode = "E_SUPPORT_RENDER_LAYER_MISSING" | "E_RENDERER_NOT_READY" | "E_RENDERER_CANCELLED" |
  "E_RENDERER_CONTEXT_LOST" | "E_RENDERER_STALE_FRAME" | "E_RENDERER_NO_SURFACE" |
  "E_RENDERER_PRESENT_FAILED" | "E_RENDERER_SURFACE_ERROR" | "E_RENDERER_SCENE_NOT_READY";

/** A rejected frame never supplies first-frame readiness to the authoritative entry gate. */
export class RenderCommitError extends Error {
  constructor(readonly code: RenderCommitErrorCode, options?: ErrorOptions) {
    super(code, options);
    this.name = "RenderCommitError";
  }
}

/** Pixi projects authoritative snapshots only; it never changes HP, collision, or route state. */
export class WorldRenderer {
  private readonly app = new Application();
  private readonly world = new Container();
  private readonly layerContainers = new Map<RenderLayerId, Container>();
  private readonly sprites = new Map<string, SpriteRecord>();
  private worldObjectDepths: ReadonlyMap<string,number> = new Map();
  private readonly propCutoutMasks = new Map<string, Graphics>();
  private facilityFloorGraphic: Graphics | null = null;
  private facilityWallGraphic: Graphics | null = null;
  private facilityFloorMesh: Mesh<MeshGeometry> | null = null;
  private readonly actorFeedbackTints = new Map<string, { sprite: Sprite; original: number }>();
  private signalFeedbackTargets = new Map<string, { sprite: Sprite; footY: number } | null>();
  private readonly atlasTextures = new Map<string, Texture>();
  private readonly atlasLoadOwners = ATLAS_LOAD_OWNERS;
  private readonly atlasPendingUnloads = ATLAS_PENDING_UNLOADS;
  private readonly atlasResidentOwner = Symbol("renderer-atlases");
  private readonly frameTextures = new Map<string, Texture>();
  private readonly sceneGraphics = new Map<string, Graphics>();
  private boundActorPreviews: ReadonlyMap<string, string> = new Map();
  private readonly playerLocomotionModel = new PlayerLocomotionModel();
  private playerLocomotionMesh: PlayerLocomotionMesh | null = null;
  private playerLocomotionSpriteKey: string | null = null;
  private playerContactShadow: Graphics | null = null;
  private readonly sceneGlows = new Map<string, Graphics>();
  private readonly actionVfx = new Map<string, Graphics>();
  private sentinelEncounterGraphic: Graphics | null = null;
  private sentinelEncounterGlow: Graphics | null = null;
  private sentinelEncounterLabel: Text | null = null;
  private readonly sentinelTelegraphGraphics = new Map<number, Graphics>();
  private readonly sentinelGlowGraphics = new Map<number, Graphics>();
  private readonly actionVfxModel = new ActionVfxModel();
  private readonly combatFeedbackModel = new CombatFeedbackModel();
  private readonly combatFeedbackLayer = new CombatPresentationLayer();
  private readonly sentinelTelegraphModel = new SentinelTelegraphModel();
  private readonly clockworksEnemyModel = new ClockworksEnemyModel();
  private readonly clockworksEnemyGraphics = new Map<number, Graphics>();
  private readonly clockworksEnemyGlows = new Map<number, Graphics>();
  private swarmMemberBodies = new Map<string, { sprite: Sprite; shadow: Graphics; outline: Graphics }>();
  private readonly clockworksEnemyBodies = new Map<string, { shadow: Graphics; outline: Graphics }>();
  private readonly pressureWaveModel = new ClockworksPressureWaveModel();
  private readonly pressureWaveGraphics = new Map<number, Graphics>();
  private movingSurfaceGraphic: Graphics | null = null;
  private verticalSupportGraphic: Graphics | null = null;
  private regulatorBossMark: Graphics | null = null;
  private machineryGraphics = new Map<string, Graphics>();
  private heatZoneGraphic: Graphics | null = null;
  private wardenWarning: { graphic: Graphics; label: Text } | null = null;
  private readonly environmentHazards = new Map<string, { graphic: Graphics; label: Text }>();
  private readonly fogLayerModel = new FogLayerModel();
  private readonly waterRippleModel = new WaterRippleModel();
  private waterRippleGraphic: Graphics | null = null;
  private readonly pumpSceneFeedbackModel = new PumpSceneFeedbackModel();
  private pumpMachineGraphic: Graphics | null = null;
  private pumpIndicatorGraphic: Graphics | null = null;
  private readonly soundCueModel = new SoundCueModel();
  private soundCueUi: Container | null = null;
  private soundCueUiEventId: number | null = null;
  private readonly worldUi = new Map<string, WorldUiRecord>();
  private actorPositionsForWorldUi: ReadonlyMap<string, Vec3> | undefined;
  private readonly sceneResources = new SceneResourceTracker();
  private readonly sceneEpoch = new SceneEpochTracker();
  private readonly cameraRig = new CameraRig();
  private readonly bossCameraModel = new BossCameraModel();
  private readonly occluderFader = new OccluderFader();
  private scenePresentation: ScenePresentationPlan | null = null;
  private expectedSceneKey: string | null = null;
  private sceneRequestRevision = 0;
  private sceneInvalidationCleanup: Promise<void> = Promise.resolve();
  private renderQueue: Promise<void> = Promise.resolve();
  private cameraLastTimeMs: number | null = null;
  private transientPresentationRevision = 0;
  private ready = false;
  private cancelled = false;
  private contextLost = false;
  private committedFrame: { sceneKey: string; revision: number; surface: PresentationSurface; supportGeometryKey: string | null; sentinelGeometryKey: string | null;
    playerAim: PresentedPlayerAimFrame } | null = null;
  private surface: PresentationSurface | null = null;
  private surfaceCanvas: HTMLCanvasElement | null = null;
  private surfaceObserver: ResizeObserver | null = null;
  private surfaceError: unknown = null;
  private transientsCleared = false;
  private lastFrame: { snapshot: WorldSnapshotEnvelope; playerPosition?: Vec3;
    actorPositions?: ReadonlyMap<string, Vec3>; timeMs: number } | null = null;
  private readonly resizeSurface = (): void => {
    const host = this.surfaceCanvas?.parentElement ?? this.surfaceCanvas;
    if (this.ready && host && (Math.max(1, host.clientWidth) !== this.app.screen.width ||
      Math.max(1, host.clientHeight) !== this.app.screen.height)) this.surface?.request(true);
  };
  private readonly loseContext = (event: Event): void => {
    event.preventDefault();
    this.contextLost = true;
    this.committedFrame = null;
    this.lastFrame = null;
    this.sceneRequestRevision++;
    this.surface?.lost();
    this.clearTransientPresentation();
    this.clearClockworksEnemyBodies();
  };
  // Recovery requires an explicit journey retry and replacement renderer. A restored
  // WebGL context alone cannot revive the authoritative first-frame proof.
  private readonly restoreContext = (): void => { this.surface?.invalidate(); };

  constructor(private readonly registry: AssetRegistry) {}

  /** Injects a compiler-produced SceneDefinition after the Rust scene registry has accepted it. */
  setSceneDefinition(definition: VerifiedSceneDefinition): void {
    this.clearMovingSurface();
    this.clearVerticalSupports();
    this.committedFrame = null;
    this.scenePresentation = null;
    this.fogLayerModel.reset();
    this.waterRippleModel.reset();
    this.clearWaterRipple();
    this.pumpSceneFeedbackModel.reset();
    this.clearPumpSceneFeedback();
    this.sceneRequestRevision++;
    const plan = planScenePresentation(this.registry, definition);
    const expectedIdentity = this.expectedSceneKey?.split("\0");
    if (expectedIdentity && (expectedIdentity[0] !== plan.worldId || expectedIdentity[1] !== plan.sceneId)) {
      throw new Error(`E_SCENE_PRESENTATION_IDENTITY_MISMATCH:${plan.worldId}:${plan.sceneId}`);
    }
    this.scenePresentation = plan;
    this.boundActorPreviews = boundActorPreviewIds(definition);
  }

  /** Invalidates queued/in-flight frames as soon as an authoritative snapshot changes scene or epoch. */
  expectSceneIdentity(identity: { worldId: string; sceneId: string; worldEpoch: number }): void {
    const nextKey = `${identity.worldId}\0${identity.sceneId}\0${identity.worldEpoch}`;
    if (nextKey === this.expectedSceneKey) return;
    this.expectedSceneKey = nextKey;
    this.cancelled = false;
    this.committedFrame = null;
    this.lastFrame = null;
    this.surface?.invalidate();
    this.clearTransientPresentation();
    this.soundCueModel.reset();
    this.fogLayerModel.reset();
    this.waterRippleModel.reset();
    this.clearWaterRipple();
    this.clearSoundCueUi();
    this.sceneRequestRevision++;
    this.scenePresentation = null;
    this.boundActorPreviews = new Map();
    this.sceneInvalidationCleanup = this.clearSceneResources();
    void this.sceneInvalidationCleanup.catch(() => undefined);
  }

  /** Revokes both completed proof and work already queued before cancellation. */
  invalidate(): void {
    this.clearMovingSurface();
    this.clearVerticalSupports();
    this.cancelled = true;
    this.clearTransientPresentation();
    this.clearClockworksEnemyBodies();
    this.sceneRequestRevision++;
    this.committedFrame = null;
    this.lastFrame = null;
    this.expectedSceneKey = null;
    this.scenePresentation = null;
    this.surface?.invalidate();
  }

  isReadyFor(snapshot: WorldSnapshotEnvelope): boolean {
    const committed = this.committedFrame;
    if (!this.ready || this.cancelled || this.contextLost || this.surfaceError !== null || !committed ||
      committed.revision !== this.sceneRequestRevision || committed.surface !== this.surface ||
      !committed.surface.isAvailable()) return false;
    const view = normalizeRenderSnapshot(snapshot);
    const key = `${view.worldId}\0${view.sceneId}\0${view.worldEpoch}`;
    if (committed.sceneKey !== key || this.expectedSceneKey && this.expectedSceneKey !== key) return false;
    try { return committed.supportGeometryKey === projectVerticalSupports(this.scenePresentation, snapshot).geometryKey &&
      committed.sentinelGeometryKey === sentinelGeometryKey(snapshot); }
    catch { return false; }
  }

  pointerAim(snapshot: WorldSnapshotEnvelope, clientX: number, clientY: number, rect: PointerSurfaceRect): PointerAimOffset | null {
    const frame = this.committedFrame;
    if (!this.ready || this.cancelled || this.contextLost || this.surfaceError !== null || !frame ||
        frame.revision !== this.sceneRequestRevision || frame.surface !== this.surface || !frame.surface.isAvailable() ||
        frame.playerAim.width !== this.app.screen.width || frame.playerAim.height !== this.app.screen.height) return null;
    const view = normalizeRenderSnapshot(snapshot);
    const key = `${view.worldId}\0${view.sceneId}\0${view.worldEpoch}`;
    if (frame.sceneKey !== key || this.expectedSceneKey && this.expectedSceneKey !== key) return null;
    return pointerOffsetFromPresentedFrame(frame.playerAim, rect, clientX, clientY);
  }

  private assertFrameCurrent(sceneKey: string, renderRevision: number): PresentationSurface {
    if (!this.ready) throw new RenderCommitError("E_RENDERER_NOT_READY");
    if (this.contextLost) throw new RenderCommitError("E_RENDERER_CONTEXT_LOST");
    if (this.cancelled) throw new RenderCommitError("E_RENDERER_CANCELLED");
    if (renderRevision !== this.sceneRequestRevision || (this.expectedSceneKey && this.expectedSceneKey !== sceneKey)) {
      throw new RenderCommitError("E_RENDERER_STALE_FRAME");
    }
    if (this.expectedSceneKey && !this.scenePresentation) throw new RenderCommitError("E_RENDERER_SCENE_NOT_READY");
    if (this.surfaceError !== null) throw new RenderCommitError("E_RENDERER_SURFACE_ERROR", { cause: this.surfaceError });
    if (!this.surface) throw new RenderCommitError("E_RENDERER_NO_SURFACE");
    if (!this.surface.isAvailable()) throw new RenderCommitError("E_RENDERER_PRESENT_FAILED");
    return this.surface;
  }

  acceptSoundCues(events: readonly SoundCueEvent[], snapshot: WorldSnapshotV3): void {
    this.soundCueModel.accept(events, snapshot, performance.now());
  }

  acceptSentinelPresentationEvents(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    if (snapshot.protocolVersion !== 3) {
      this.clearTransientPresentation();
      return [];
    }
    return this.sentinelTelegraphModel.accept(events, snapshot);
  }

  acceptPressureWavePresentationEvents(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    if (snapshot.protocolVersion !== 3) {
      this.pressureWaveModel.reset();
      this.destroyGraphics(this.pressureWaveGraphics);
      return [];
    }
    return this.pressureWaveModel.accept(events, snapshot);
  }

  acceptClockworksEnemyPresentationEvents(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    if (this.cancelled || this.contextLost) return [];
    return this.clockworksEnemyModel.accept(events, snapshot);
  }

  acceptCombatPresentationEvents(events: readonly PresentationEvent[], snapshot: WorldSnapshotEnvelope): number[] {
    if (this.cancelled || this.contextLost) return [];
    return this.combatFeedbackModel.accept(events, snapshot);
  }

  clearTransientPresentation(): void {
    this.clearActorFeedbackTints();
    this.playerLocomotionModel?.reset();
    this.clearPlayerLocomotionMesh();
    this.transientPresentationRevision++;
    this.transientsCleared = true;
    this.combatFeedbackModel?.reset();
    this.combatFeedbackLayer?.clear((id,color)=>this.setActorFeedbackTint(id,color));
    this.actionVfxModel?.reset();
    for (const graphic of this.actionVfx?.values() ?? []) graphic.destroy();
    this.actionVfx?.clear();
    this.sentinelTelegraphModel.reset();
    this.pressureWaveModel?.reset();
    this.clockworksEnemyModel?.reset();
    if (this.clockworksEnemyGraphics) this.destroyGraphics(this.clockworksEnemyGraphics);
    if (this.clockworksEnemyGlows) this.destroyGraphics(this.clockworksEnemyGlows);
    this.destroyGraphics(this.sentinelTelegraphGraphics);
    this.destroyGraphics(this.sentinelGlowGraphics);
    if (this.pressureWaveGraphics) this.destroyGraphics(this.pressureWaveGraphics);
    this.clearRegulatorBossMark();
    this.clearClockworksHeat();
    this.clearMovingMachinery();
    this.surface?.request();
  }

  reconcileSoundCue(snapshot: WorldSnapshotEnvelope): void {
    if (snapshot.protocolVersion !== 3 || !this.soundCueModel.view(snapshot, performance.now())) {
      this.clearSoundCueUi();
    }
  }

  clearSoundCues(): void {
    this.soundCueModel.reset();
    this.clearSoundCueUi();
    this.surface?.request();
  }

  async init(canvas: HTMLCanvasElement): Promise<void> {
    const host = canvas.parentElement ?? canvas;
    // SessionLoop owns animation. Pixi must not redraw independently while
    // presentation is paused, hidden, stopped or waiting for verified assets.
    await this.app.init({ canvas, width: Math.max(1, host.clientWidth), height: Math.max(1, host.clientHeight),
      backgroundColor: 0x091720, autoStart: false, sharedTicker: false });
    this.app.stage.addChild(this.world);
    for (const layerId of RENDER_LAYERS) {
      const layer = new Container();
      layer.label = layerId;
      layer.sortableChildren = true;
      this.layerContainers.set(layerId, layer);
      this.world.addChild(layer);
    }
    this.ready = true;
    this.surfaceCanvas = canvas;
    this.surface = new PresentationSurface(() => this.app.render(), () => this.refreshSurface(),
      error => { this.surfaceError = error; this.committedFrame = null; });
    window.addEventListener("resize", this.resizeSurface);
    canvas.addEventListener("webglcontextlost", this.loseContext);
    canvas.addEventListener("webglcontextrestored", this.restoreContext);
    if (typeof ResizeObserver !== "undefined") {
      this.surfaceObserver = new ResizeObserver(this.resizeSurface);
      this.surfaceObserver.observe(host);
    }
  }

  private refreshSurface(): Promise<void> {
    const revision = this.sceneRequestRevision;
    const operation = this.renderQueue.then(async () => {
      if (!this.ready || revision !== this.sceneRequestRevision) return;
      const host = this.surfaceCanvas?.parentElement ?? this.surfaceCanvas;
      if (!host) return;
      const width = Math.max(1, host.clientWidth), height = Math.max(1, host.clientHeight);
      if (width !== this.app.screen.width || height !== this.app.screen.height) this.app.renderer.resize(width, height);
      const frame = this.lastFrame;
      if (frame) await this.renderFrame(frame.snapshot, frame.playerPosition, frame.actorPositions, frame.timeMs, revision);
    });
    this.renderQueue = operation.catch(() => undefined);
    return operation;
  }

  /** Serializes frames so an older epoch cannot finish loading after a newer one. */
  render(snapshot: WorldSnapshotEnvelope, playerPosition?: Vec3, actorPositions?: ReadonlyMap<string, Vec3>): Promise<void> {
    const requestedTransientRevision = this.transientPresentationRevision;
    const requestedRevision = this.sceneRequestRevision;
    const scheduled = this.renderQueue.then(() => {
      if (requestedTransientRevision === this.transientPresentationRevision) this.transientsCleared = false;
      return this.renderFrame(snapshot, playerPosition, actorPositions, undefined, requestedRevision);
    }).catch(error => {
      if (requestedRevision === this.sceneRequestRevision) this.committedFrame = null;
      throw error;
    });
    this.renderQueue = scheduled.catch(() => undefined);
    return scheduled;
  }

  private async renderFrame(snapshot: WorldSnapshotEnvelope, playerPosition?: Vec3, actorPositions?: ReadonlyMap<string, Vec3>, replayTimeMs?: number, renderRevision = this.sceneRequestRevision): Promise<void> {
    const frameTimeMs = replayTimeMs ?? performance.now();
    const sentinelFrame = snapshot.protocolVersion === 3 ? assertSentinelEncounter(snapshot) : null;
    const sentinelFrameKey = sentinelGeometryKey(snapshot);
    if (sentinelFrame && actorPositions?.has(SENTINEL_ID)) {
      const authoritativePositions = new Map(actorPositions); authoritativePositions.delete(SENTINEL_ID); actorPositions = authoritativePositions;
    }
    const view = normalizeRenderSnapshot(snapshot);
    const scene = { worldId: view.worldId, sceneId: view.sceneId, worldEpoch: view.worldEpoch };
    const transientRevision = this.transientPresentationRevision;
    const sceneKey = `${scene.worldId}\0${scene.sceneId}\0${scene.worldEpoch}`;
    this.assertFrameCurrent(sceneKey, renderRevision);
    await this.sceneInvalidationCleanup;
    this.assertFrameCurrent(sceneKey, renderRevision);
    if (this.sceneEpoch.enter(scene)) await this.clearSceneResources();
    this.assertFrameCurrent(sceneKey, renderRevision);
    const reduceFogMotion = document.documentElement.dataset.motion === "reduced" ||
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;
    const pumpFeedback = this.pumpSceneFeedbackModel.project(snapshot, frameTimeMs, reduceFogMotion);
    if (!pumpFeedback) this.clearPumpSceneFeedback();
    const skin = worldSkin(view.worldId);
    this.app.renderer.background.color = skin.background;
    this.layerContainers.get("L0_BACKGROUND")!.tint = skin.backPropTint;
    this.layerContainers.get("L1_FLOOR")!.tint = skin.floorTint;
    this.layerContainers.get("L2_BACK_PROPS")!.tint = skin.backPropTint;
    this.layerContainers.get("L4_DYNAMIC_PROPS")!.tint = skin.dynamicTint;
    this.layerContainers.get("L5_FRONT_PROPS")!.tint = skin.frontPropTint;
    this.layerContainers.get("L7_VFX")!.tint = skin.vfxTint;
    const scenePresentation = this.scenePresentation;
    if (scenePresentation && (scenePresentation.worldId !== view.worldId || scenePresentation.sceneId !== view.sceneId)) {
      throw new Error(`E_SCENE_PRESENTATION_IDENTITY_MISMATCH:${view.worldId}:${view.sceneId}`);
    }

    const supportFrame = projectVerticalSupports(scenePresentation, snapshot);
    if (supportFrame.frames.length) playerPosition = view.playerPosition;

    const actors: ActorView[] = [
      { entityId: view.playerEntityId, entityType: "player.cenyao", actorKind: "player",
        transform: { positionM: playerPosition ?? view.playerPosition, yawRad: view.playerYawRad }, active: true },
      ...view.actors,
    ];
    const facingDirection = screenFacingDirection(view.playerFacingX, view.playerFacingZ);
    const resourcePlan = buildRendererResourcePlan(this.registry, view.worldId, actors, view.doors);
    await this.preloadAtlasPages([...resourcePlan.atlasPages, ...(scenePresentation?.atlasPages ?? [])]);
    this.assertFrameCurrent(sceneKey, renderRevision);

    const assetsById = new Map(resourcePlan.assets.map(asset => [asset.assetId, asset]));
    const cameraOrigin = playerPosition ?? view.playerPosition;
    const deltaSeconds = this.cameraLastTimeMs === null ? 0 : Math.min(0.1, Math.max(0, (frameTimeMs - this.cameraLastTimeMs) / 1000));
    this.cameraLastTimeMs = frameTimeMs;
    const basePixelsPerMeter = viewportPixelsPerMeter(this.app.screen.width, this.app.screen.height);
    const { camera, pixelsPerMeter } = this.updateCameraFrame(snapshot, cameraOrigin, view.playerAimX, view.playerAimZ,
      this.app.screen.width, this.app.screen.height, deltaSeconds, basePixelsPerMeter);
    const presentationScale = pixelsPerMeter / BASE_PIXELS_PER_METER;
    const sceneBounds = scenePresentation?.bounds ?? { width: 24, depth: 16 };
    const desired: DesiredSprite[] = [];
    for (const actor of actors) {
      if (!actor.active) continue;
      if (actor.entityType === "enemy.mist_harbor.signal_wraith" &&
          (!isNativeSignalWraith(actor, view.worldId, view.sceneId) || !validatedSignalPerception(actor))) {
        throw new Error(`E_SIGNAL_PRESENTATION_INVALID:${actor.entityId}`);
      }
      const asset = assetsById.get(actorAssetId(actor));
      if (!asset) throw new Error(`E_RENDERER_UNKNOWN_ASSET:${actor.entityType}`);
      const position = supportFrame.frames.length && actor.entityId === view.playerEntityId
        ? view.playerPosition : sentinelFrame && actor.entityId === SENTINEL_ID
          ? actor.transform.positionM : actorPositions?.get(actor.entityId) ?? actor.transform.positionM;
      const screen = projectWorldPoint(position, camera);
      const actorFacing = actor.entityId !== view.playerEntityId
        ? screenFacingDirection(Math.sin(actor.transform.yawRad), Math.cos(actor.transform.yawRad))
        : facingDirection;
      const frameSelection = selectSpriteFrame(asset, actorFacing,
        actor.entityId === view.playerEntityId ? view.playerActionState : "non-player-actor");
      const frame = frameSelection?.atlasFrame ?? asset.atlasFrame;
      desired.push({
        key: `actor:${actor.entityId}`,
        asset,
        position,
        layer: asset.kind === "Prop" || asset.kind === "Door" ? "L4_DYNAMIC_PROPS" : "L3_ACTORS",
        footY: screen.footY,
        zGroup: 0,
        screenX: screen.x,
        screenY: screen.y,
        displayScale: resolveSpriteDisplayScale(asset, frame[2], frame[3], pixelsPerMeter, sceneBounds),
        frameSelection,
      });
    }
    for (const door of view.doors) {
      const assetId = doorAssetId(door.doorId);
      const asset = assetsById.get(assetId);
      if (!asset) throw new Error(`E_RENDERER_UNKNOWN_ASSET:${assetId}`);
      const screen = projectWorldPoint(door.transform.positionM, camera);
      desired.push({
        key: `door:${door.doorId}`,
        asset,
        position: door.transform.positionM,
        layer: "L4_DYNAMIC_PROPS",
        footY: screen.footY,
        zGroup: 0,
        screenX: screen.x,
        screenY: screen.y,
        displayScale: resolveSpriteDisplayScale(asset, asset.atlasFrame[2], asset.atlasFrame[3],
          pixelsPerMeter, sceneBounds, "visual.props_dynamic"),
      });
    }

    const projectedScene = scenePresentation
      ? projectScenePresentation(scenePresentation, camera)
      : { sprites: [], occluders: [] };
    this.renderFacilityFloor(scenePresentation?.facilityFloor, camera);
    const fogFrame = this.fogLayerModel.project(scene, projectedScene.sprites, frameTimeMs, reduceFogMotion);
    for (const placement of projectedScene.sprites) {
      if (!greyHiveBeaconSpriteVisible(snapshot, placement.id)) continue;
      if (this.boundActorPreviews.has(placement.id)) continue;
      const fogOffset = fogFrame.offsets.get(placement.id);
      desired.push({
        key: `scene:${placement.id}`,
        asset: placement.asset,
        position: placement.position,
        layer: placement.layer,
        footY: placement.footY,
        zGroup: placement.zGroup,
        screenX: placement.screenX + (fogOffset?.x ?? 0),
        screenY: placement.screenY + (fogOffset?.y ?? 0),
        displayScale: placement.displaySizeM ? placement.displaySizeM.width * pixelsPerMeter / placement.asset.atlasFrame[2]
          : resolveSpriteDisplayScale(placement.asset, placement.asset.atlasFrame[2], placement.asset.atlasFrame[3], pixelsPerMeter, sceneBounds, placement.layerId),
        ...(placement.displaySizeM ? {displayScaleY: placement.displaySizeM.height * pixelsPerMeter / placement.asset.atlasFrame[3]} : {}),
      });
    }

    // Registered live props own their exact placement; do not retain a duplicate static preview.
    const liveProps = desired.filter(item => !item.key.startsWith("scene:") && isMappedProp(item.asset.assetId));
    for (let i=desired.length-1;i>=0;i--) {
      const item=desired[i]!;
      if(item.key.startsWith("scene:") && liveProps.some(live=>live.asset.assetId===item.asset.assetId && live.position.xM===item.position.xM && live.position.yM===item.position.yM && live.position.zM===item.position.zM)){desired.splice(i,1);continue;}
      const prop=projectPropPresentation(item.asset,snapshot,view.doors);
      if(prop===null){desired.splice(i,1);continue;}
      if(prop){item.asset=prop.asset;delete item.frameSelection;item.displayScale=prop.heightM*pixelsPerMeter/prop.asset.atlasFrame[3];if(prop.cutout)item.cutout=prop.cutout;else delete item.cutout;}
    }
    const publicBodies: {key:string;footY:number;layer:RenderLayerId}[]=[];
    for(const actor of actors){
      if(!actor.active)continue;
      const signal=validatedSignalPerception(actor);
      if(signal)signal.positionsM.forEach((p,index)=>publicBodies.push({key:`signal:${actor.entityId}:${index}`,footY:projectWorldPoint(p,camera).footY,layer:"L3_ACTORS"}));
      const members=validatedSwarmMembers(actor);
      if(members)for(const member of members)if(member.active)publicBodies.push({key:`swarm:${member.memberId}`,footY:projectWorldPoint(member.positionM,camera).footY,layer:"L3_ACTORS"});
    }
    const depthRanks = worldDepthRanks([...desired,...publicBodies]);
    this.worldObjectDepths=depthRanks;
    this.clearActorFeedbackTints();
    const visible = new Set(desired.map(item => item.key));
    for (const [key, record] of this.sprites) {
      if (!visible.has(key)) this.removeSprite(key, record);
    }
    for (const layerId of RENDER_LAYERS) {
      const sorted = desired.filter(item => item.layer === layerId).sort((left, right) =>
        (layerId === "L1_FLOOR"
          ? left.zGroup - right.zGroup || left.footY - right.footY
          : left.footY - right.footY || left.zGroup - right.zGroup) ||
        (left.key < right.key ? -1 : left.key > right.key ? 1 : 0));
      for (let index = 0; index < sorted.length; index++) {
        const item = sorted[index];
        if (!item) continue;
        const record = this.ensureSprite(item.key, item.asset, layerId, item.frameSelection, item.displayScale);
        if (item.key.startsWith("scene:")) {
          const key = item.key;
          this.sceneResources.trackIfAbsent(`sprite:${key}`, () => {
            const sceneSprite = this.sprites.get(key);
            if (sceneSprite) this.removeSprite(key, sceneSprite);
          });
        }
        if (item.displayScaleY !== undefined) record.sprite.scale.y = item.displayScaleY;
        record.sprite.position.set(item.screenX, item.screenY);
        if(item.cutout){
          let mask=this.propCutoutMasks.get(item.key);
          if(!mask){mask=new Graphics();this.layerContainers.get("L3_ACTORS")!.addChild(mask);this.propCutoutMasks.set(item.key,mask);record.sprite.mask=mask;}
          const [, , width,height]=item.asset.atlasFrame;
          mask.clear().poly(item.cutout.flatMap(([x,y])=>[item.screenX+(x-item.asset.anchorX*width)*item.displayScale,item.screenY+(y-item.asset.anchorY*height)*(item.displayScaleY??item.displayScale)])).fill(0xffffff);
        }
        record.sprite.zIndex = depthRanks.get(item.key) ?? index;
        record.sprite.tint = !sharesWorldDepth(item.layer,item.asset.assetId) ? 0xffffff : item.layer === "L2_BACK_PROPS" ? skin.backPropTint
          : item.layer === "L4_DYNAMIC_PROPS" ? skin.dynamicTint
          : item.layer === "L5_FRONT_PROPS" ? skin.frontPropTint : 0xffffff;
      }
    }
    this.renderClockworksEnemyBodies(actors, camera, actorPositions);
    this.renderPlayerLocomotion(snapshot, view, facingDirection, reduceFogMotion);
    this.renderSceneGlows(scenePresentation ? projectSceneGlows(view.worldId, view.sceneId, projectedScene.sprites) : [], skin.vfxTint, presentationScale);
    this.renderMovingSurface(projectMovingSurfaces(scenePresentation, snapshot, reduceFogMotion), camera);
    this.renderVerticalSupports(supportFrame.frames, camera);
    this.renderSentinelEncounter(sentinelFrame, camera, reduceFogMotion);
    if (transientRevision === this.transientPresentationRevision && !this.transientsCleared) {
      this.renderSentinelTelegraphs(this.sentinelTelegraphModel.project(snapshot, reduceFogMotion), camera);
      this.renderPressureWave(this.pressureWaveModel.project(snapshot, reduceFogMotion), camera);
      this.renderClockworksEnemyCues(this.clockworksEnemyModel.project(snapshot, reduceFogMotion), camera);
      this.renderRegulatorBossMark(projectPrimeRegulatorBossMark(snapshot, reduceFogMotion), camera);
      this.renderClockworksHeat(projectClockworksHeat(snapshot, reduceFogMotion), camera);
      this.renderMovingMachinery(projectMovingMachinery(snapshot, reduceFogMotion), camera);
      const combatAnchors: CombatDepthAnchor[] = [];
      for (const item of desired) {
        if (!["L2_BACK_PROPS", "L3_ACTORS", "L4_DYNAMIC_PROPS", "L5_FRONT_PROPS"].includes(item.layer)) continue;
        const targetId = item.key.startsWith("actor:") ? item.key.slice(6) : undefined;
        if (targetId && this.signalFeedbackTargets.has(targetId)) {
          // Only the one precise public body can identify a Wraith contact. The hidden
          // resource owner and ambiguous alternatives must never become target anchors.
          const target = this.signalFeedbackTargets.get(targetId);
          if (target) combatAnchors.push({ targetId, footY: target.footY, zIndex: target.sprite.zIndex });
        } else {
          combatAnchors.push({ footY: item.footY, zIndex: this.sprites.get(item.key)!.sprite.zIndex,
            ...(targetId ? { targetId } : {}) });
        }
      }
      this.combatFeedbackLayer.render(this.combatFeedbackModel.project(snapshot, reduceFogMotion, frameTimeMs),
        this.layerContainers.get("L3_ACTORS")!, camera, frame => combatContactDepth(frame, camera, combatAnchors), (id,color)=>this.setActorFeedbackTint(id,color));
    }
    // Persistent hazard telegraphs keep their authoritative frozen state while paused.
    this.renderEnvironmentHazards(projectEnvironmentHazards(snapshot, reduceFogMotion), camera);
    this.renderWardenWarning(projectWardenEncounter(snapshot, reduceFogMotion), camera);
    this.renderPumpSceneFeedback(pumpFeedback, camera);
    if (view.playerFacingSource !== "v3-authoritative") this.actionVfxModel.reset();
    this.renderActionVfx(!this.transientsCleared && view.playerFacingSource === "v3-authoritative" ? this.actionVfxModel.update({
      worldId: view.worldId, sceneId: view.sceneId, worldEpoch: view.worldEpoch,
      actionState: view.playerActionState, position: view.playerPosition,
      ...(snapshot.protocolVersion === 3 && snapshot.player.actionPresentation ? { actionPresentation: snapshot.player.actionPresentation } : {}),
      serverTick: snapshot.serverTick,
      facingX: view.playerFacingX, facingZ: view.playerFacingZ, nowMs: frameTimeMs,
    }) : null, camera, reduceFogMotion, this.sprites.get(`actor:${view.playerEntityId}`)?.sprite.zIndex ?? 0);
    this.renderWaterRipple(this.waterRippleModel.project(snapshot, frameTimeMs, reduceFogMotion), camera);
    this.renderOccluders(projectedScene.occluders, cameraOrigin, skin.occluderColor);
    this.actorPositionsForWorldUi = actorPositions;
    try {
      this.renderWorldUi(snapshot, camera);
    } finally {
      this.actorPositionsForWorldUi = undefined;
    }
    this.renderSoundCue(snapshot, frameTimeMs);
    const surface = this.assertFrameCurrent(sceneKey, renderRevision);
    // Do not record readiness (or a replayable frame) until Pixi actually drew it.
    this.committedFrame = null;
    if (!surface.present()) {
      this.assertFrameCurrent(sceneKey, renderRevision);
      throw new RenderCommitError("E_RENDERER_PRESENT_FAILED");
    }
    if (this.assertFrameCurrent(sceneKey, renderRevision) !== surface) throw new RenderCommitError("E_RENDERER_STALE_FRAME");
    this.lastFrame = { snapshot, ...(playerPosition ? { playerPosition } : {}),
      ...(actorPositions ? { actorPositions } : {}), timeMs: frameTimeMs };
    const presentedPlayer = projectWorldPoint(playerPosition ?? view.playerPosition, camera);
    this.committedFrame = { sceneKey, revision: renderRevision, surface, supportGeometryKey: supportFrame.geometryKey, sentinelGeometryKey: sentinelFrameKey,
      playerAim: { width: camera.width, height: camera.height, footX: presentedPlayer.x, footY: presentedPlayer.y } };
  }

  private renderPlayerLocomotion(snapshot: WorldSnapshotEnvelope, view: RenderSnapshot,
    direction: Parameters<PlayerLocomotionMesh["applyLocomotion"]>[1], reducedMotion: boolean): void {
    const key = `actor:${view.playerEntityId}`;
    const sprite = this.sprites.get(key)?.sprite;
    if (!sprite?.parent) { this.clearPlayerLocomotionMesh(); return; }
    const hp = snapshot.protocolVersion === 3 ? snapshot.player.currentHp : snapshot.view.player.currentHp;
    const frame = this.playerLocomotionModel.project({ worldId: view.worldId, sceneId: view.sceneId,
      worldEpoch: view.worldEpoch, playerEntityId: view.playerEntityId, serverTick: view.serverTick,
      position: view.playerPosition, velocity: view.playerVelocity, actionState: view.playerActionState,
      alive: hp > 0, paused: this.transientsCleared,
      loading: snapshot.protocolVersion === 3 && snapshot.entryToken !== undefined, reducedMotion });
    // The contact shadow shares the actual raised/lowered footpoint, not ground-plane depth.
    if (!this.playerContactShadow) {
      this.playerContactShadow = new Graphics();
      this.playerContactShadow.label = "temporary_visual=true:player-contact-shadow";
    }
    if (this.playerContactShadow.parent !== sprite.parent) sprite.parent.addChild(this.playerContactShadow);
    this.playerContactShadow.clear().ellipse(0, 0, sprite.texture.width * 0.15, sprite.texture.height * 0.018)
      .fill({ color: 0x050b10, alpha: 0.35 });
    this.playerContactShadow.position.copyFrom(sprite.position);
    this.playerContactShadow.scale.copyFrom(sprite.scale);
    this.playerContactShadow.zIndex = sprite.zIndex - 0.25;
    if (!frame.active) { this.clearPlayerLocomotionMesh(); return; }
    if (!this.playerLocomotionMesh) this.playerLocomotionMesh = new PlayerLocomotionMesh(sprite.texture);
    const mesh = this.playerLocomotionMesh;
    if (mesh.parent !== sprite.parent) sprite.parent.addChild(mesh);
    mesh.setFrame(sprite.texture, [sprite.anchor.x, sprite.anchor.y]);
    mesh.applyLocomotion(frame, direction);
    mesh.position.copyFrom(sprite.position);
    mesh.scale.copyFrom(sprite.scale);
    mesh.zIndex = sprite.zIndex;
    mesh.tint = sprite.tint;
    mesh.alpha = sprite.alpha;
    this.playerLocomotionSpriteKey = key;
    sprite.visible = false;
  }

  private clearPlayerLocomotionMesh(): void {
    if (this.playerLocomotionSpriteKey) {
      const sprite = this.sprites.get(this.playerLocomotionSpriteKey)?.sprite;
      if (sprite) sprite.visible = true;
    }
    this.playerLocomotionMesh?.parent?.removeChild(this.playerLocomotionMesh);
    this.playerLocomotionMesh?.destroy();
    this.playerLocomotionMesh = null;
    this.playerLocomotionSpriteKey = null;
  }

  /** Feedback follows the displayed public body; ambiguity never selects a Wraith alternative. */
  setActorFeedbackTint(actorId: string, color: number | null): void {
    const key = `actor:${actorId}`;
    const sprite = this.signalFeedbackTargets?.has(actorId)
      ? this.signalFeedbackTargets.get(actorId)?.sprite : this.sprites.get(key)?.sprite;
    const owned = this.actorFeedbackTints.get(key);
    if (color === null) {
      if (!owned) return;
      this.actorFeedbackTints.delete(key);
      if (sprite !== owned.sprite || sprite.destroyed) return;
      sprite.tint = owned.original;
      if (this.playerLocomotionSpriteKey === key && this.playerLocomotionMesh) this.playerLocomotionMesh.tint = owned.original;
      return;
    }
    if (!sprite || sprite.destroyed || !Number.isInteger(color) || color < 0 || color > 0xffffff) return;
    if (!owned || owned.sprite !== sprite) this.actorFeedbackTints.set(key, { sprite, original: sprite.tint });
    sprite.tint = color;
    if (this.playerLocomotionSpriteKey === key && this.playerLocomotionMesh) this.playerLocomotionMesh.tint = color;
  }

  private clearActorFeedbackTints(): void {
    // Restore before this frame's normal style is assigned. A late clear then becomes a no-op,
    // so it cannot overwrite a freshly projected placeholder color or a replacement actor.
    for (const key of this.actorFeedbackTints?.keys() ?? []) this.setActorFeedbackTint(key.slice(6), null);
  }

  private clearSignalFeedbackTargets(): void {
    // Restore through the old binding before a reused alternative gets its new style,
    // or before a scene/death cleanup destroys it. Late clears cannot affect replacements.
    for (const id of this.signalFeedbackTargets?.keys() ?? []) {
      if (this.actorFeedbackTints?.has(`actor:${id}`)) this.setActorFeedbackTint(id, null);
    }
    this.signalFeedbackTargets?.clear();
  }

  /** temporary_visual=true: admitted textures plus stable, deliberately different silhouettes. */
  private renderClockworksEnemyBodies(actors: readonly ActorView[], camera: CameraFrame,
    actorPositions?: ReadonlyMap<string, Vec3>): void {
    const visible = new Set<string>();
    const visibleMembers = new Set<string>();
    this.swarmMemberBodies ??= new Map();
    this.clearSignalFeedbackTargets();
    this.signalFeedbackTargets ??= new Map();
    const layer = this.layerContainers.get("L3_ACTORS");
    const scale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
    for (const actor of actors) {
      const style = enemyPlaceholderStyle(actor.entityType);
      const sprite = this.sprites.get(`actor:${actor.entityId}`)?.sprite;
      if (!style || actor.actorKind !== "enemy" || !actor.active || !sprite || !layer) continue;
      if (style.silhouette === "signal") {
        sprite.visible = false;
        const projection = validatedSignalPerception(actor);
        if (!projection) throw new Error(`E_SIGNAL_PRESENTATION_INVALID:${actor.entityId}`);
        this.signalFeedbackTargets.set(actor.entityId, null);
        projection.positionsM.forEach((position, index) => {
          const key = `${actor.entityId}/signal/${index}`;
          visibleMembers.add(key);
          let body = this.swarmMemberBodies.get(key);
          if (!body) {
            body = { sprite: new Sprite(sprite.texture), shadow: new Graphics(), outline: new Graphics() };
            body.sprite.label = `temporary_visual=true:signal-alternative:${key}`;
            body.shadow.label = `temporary_visual=true:signal-shadow:${key}`;
            body.outline.label = `temporary_visual=true:signal-outline:${key}`;
            layer.addChild(body.shadow, body.sprite, body.outline); this.swarmMemberBodies.set(key, body);
          }
          const point = projectWorldPoint(position, camera);
          body.sprite.texture = sprite.texture; body.sprite.anchor.copyFrom(sprite.anchor);
          body.sprite.width = style.widthPx*scale; body.sprite.height = style.heightPx*scale;
          body.sprite.tint = style.tint; body.sprite.alpha = projection.precise ? 1 : 0.74;
          body.sprite.position.set(point.x,point.y-style.liftPx*scale);
          body.sprite.zIndex = this.worldObjectDepths?.get(`signal:${actor.entityId}:${index}`) ?? sprite.zIndex+(point.footY-projectWorldPoint(actor.transform.positionM,camera).footY)*0.001;
          if (projection.precise) this.signalFeedbackTargets.set(actor.entityId, { sprite: body.sprite, footY: point.footY });
          body.shadow.clear().ellipse(0,0,15,5).fill({color:0x060a10,alpha:0.48});
          body.shadow.scale.set(scale); body.shadow.position.set(point.x,point.y); body.shadow.zIndex=body.sprite.zIndex-0.25;
          body.outline.clear().ellipse(0,-28,18,8).stroke({color:style.tint,width:1.5,alpha:0.85})
            .moveTo(-13,-18).lineTo(-17,-5).moveTo(13,-18).lineTo(17,-5).stroke({color:0xe6ddff,width:1.2});
          body.outline.scale.set(scale);body.outline.position.set(point.x,point.y-style.liftPx*scale);body.outline.zIndex=body.sprite.zIndex+0.25;
        });
        continue;
      }
      if (style.silhouette === "cluster") {
        sprite.visible = false;
        const members = validatedSwarmMembers(actor);
        if (members) for (const member of members) {
          if (!member.active) continue;
          visibleMembers.add(member.memberId);
          let body = this.swarmMemberBodies.get(member.memberId);
          if (!body) {
            body = { sprite: new Sprite(sprite.texture), shadow: new Graphics(), outline: new Graphics() };
            body.sprite.label = `temporary_visual=true:swarm-member:${member.memberId}`;
            body.shadow.label = `temporary_visual=true:swarm-member-shadow:${member.memberId}`;
            body.outline.label = `temporary_visual=true:swarm-member-outline:${member.memberId}`;
            layer.addChild(body.shadow, body.sprite, body.outline);
            this.swarmMemberBodies.set(member.memberId, body);
          }
          // Absolute public member points, with no local swarm motion or HP model.
          const point = projectWorldPoint(member.positionM, camera);
          body.sprite.anchor.copyFrom(sprite.anchor); body.sprite.width = 14 * scale; body.sprite.height = 12 * scale;
          body.sprite.texture = sprite.texture; body.sprite.tint = style.tint; body.sprite.position.set(point.x, point.y);
          body.sprite.zIndex = this.worldObjectDepths?.get(`swarm:${member.memberId}`) ?? sprite.zIndex + (point.footY - projectWorldPoint(actor.transform.positionM, camera).footY) * 0.001;
          body.shadow.clear().ellipse(0, 0, member.radiusM * BASE_PIXELS_PER_METER, 3).fill({ color: 0x060a10, alpha: 0.48 });
          body.shadow.scale.set(scale); body.shadow.position.set(point.x, point.y); body.shadow.zIndex = body.sprite.zIndex - 0.25;
          body.outline.clear().ellipse(0, -5, 5, 4).fill({ color: style.tint, alpha: 0.65 })
            .moveTo(-4, -7).lineTo(-8, -9).moveTo(4, -7).lineTo(8, -9)
            .moveTo(-5, -4).lineTo(-8, -1).moveTo(5, -4).lineTo(8, -1).stroke({ color: 0xdcebbd, width: 1.2 });
          body.outline.scale.set(scale); body.outline.position.set(point.x, point.y); body.outline.zIndex = body.sprite.zIndex + 0.25;
        }
        continue;
      }
      sprite.visible = true;
      visible.add(actor.entityId);
      const point = projectWorldPoint(actorPositions?.get(actor.entityId) ?? actor.transform.positionM, camera);
      sprite.label = `temporary_visual=true:${actor.entityType}:${actor.entityId}`;
      sprite.width = style.widthPx * scale;
      sprite.height = style.heightPx * scale;
      sprite.tint = style.tint;
      sprite.position.set(point.x, point.y - style.liftPx * scale);
      let body = this.clockworksEnemyBodies.get(actor.entityId);
      if (!body) {
        body = { shadow: new Graphics(), outline: new Graphics() };
        body.shadow.label = `temporary_visual=true:enemy-contact-shadow:${actor.entityId}`;
        body.outline.label = `temporary_visual=true:enemy-${style.silhouette}:${actor.entityId}`;
        layer.addChild(body.shadow, body.outline);
        this.clockworksEnemyBodies.set(actor.entityId, body);
      }
      body.shadow.clear().ellipse(0, 0, style.widthPx * 0.44, 5).fill({ color: 0x060a10, alpha: 0.48 });
      body.shadow.scale.set(scale);
      body.shadow.position.set(point.x, point.y);
      body.shadow.zIndex = sprite.zIndex - 0.25;
      const outline = body.outline;
      outline.clear();
      outline.scale.set(scale);
      outline.position.set(point.x, point.y - style.liftPx * scale);
      outline.zIndex = sprite.zIndex + 0.25;
      if (style.silhouette === "rotor") {
        outline.ellipse(0, -20, 24, 6).stroke({ color: style.tint, width: 2, alpha: 0.9 });
        outline.moveTo(-24, -20).lineTo(-31, -20).moveTo(24, -20).lineTo(31, -20)
          .stroke({ color: style.tint, width: 2, alpha: 0.9 });
        outline.circle(0, -16, 5).stroke({ color: 0xeaffff, width: 1.5, alpha: 0.95 });
      } else if (style.silhouette === "tidal") {
        outline.moveTo(-23, -40).lineTo(-28, -59).lineTo(-14, -66).lineTo(14, -66).lineTo(28, -59).lineTo(23, -40)
          .moveTo(-21, -40).lineTo(-25, -18).lineTo(-17, -12).moveTo(21, -40).lineTo(25, -18).lineTo(17, -12)
          .moveTo(-12, -18).lineTo(-14, 0).moveTo(12, -18).lineTo(14, 0)
          .stroke({ color: style.tint, width: 2.2, alpha: 0.95 });
        outline.ellipse(0, -36, 13, 4).stroke({ color: 0xc2f1e8, width: 1.4, alpha: 0.8 });
      } else if (style.silhouette === "heavy") {
        outline.moveTo(-31, -50).lineTo(-36, -70).lineTo(-19, -78).lineTo(19, -78).lineTo(36, -70).lineTo(31, -50)
          .moveTo(-25, -50).lineTo(-21, -18).lineTo(-24, 0).moveTo(25, -50).lineTo(21, -18).lineTo(24, 0)
          .moveTo(-19, -59).lineTo(19, -59).moveTo(-17, -49).lineTo(17, -49)
          .stroke({ color: style.tint, width: 2.5, alpha: 0.95 });
      } else {
        outline.moveTo(-30, -13).lineTo(-20, -26).lineTo(20, -26).lineTo(31, -18).lineTo(26, -10)
          .moveTo(22, -25).lineTo(25, -34).lineTo(29, -24)
          .moveTo(-24, -13).lineTo(-27, 0).moveTo(-12, -12).lineTo(-10, 0)
          .moveTo(15, -12).lineTo(12, 0).moveTo(25, -11).lineTo(28, 0)
          .stroke({ color: style.tint, width: 2.2, alpha: 0.95 });
      }
    }
    for (const [id, body] of this.swarmMemberBodies) {
      if (visibleMembers.has(id)) continue;
      body.sprite.destroy(); body.shadow.destroy(); body.outline.destroy(); this.swarmMemberBodies.delete(id);
    }
    for (const [id, body] of this.clockworksEnemyBodies) {
      if (visible.has(id)) continue;
      body.shadow.destroy(); body.outline.destroy(); this.clockworksEnemyBodies.delete(id);
    }
  }

  private clearClockworksEnemyBodies(): void {
    this.clearSignalFeedbackTargets();
    if (this.swarmMemberBodies) {
      for (const body of this.swarmMemberBodies.values()) { body.sprite.destroy(); body.shadow.destroy(); body.outline.destroy(); }
      this.swarmMemberBodies.clear();
    }
    if (!this.clockworksEnemyBodies) return;
    for (const body of this.clockworksEnemyBodies.values()) { body.shadow.destroy(); body.outline.destroy(); }
    this.clockworksEnemyBodies.clear();
  }

  /** No local trajectory: warning endpoints use committed yaw/radius; motion uses exact server points. */
  private renderClockworksEnemyCues(frames: readonly ClockworksEnemyFrame[], camera: CameraFrame): void {
    this.releaseInvisibleGraphics(this.clockworksEnemyGraphics, new Set(frames.map(frame => frame.eventId)));
    this.releaseInvisibleGraphics(this.clockworksEnemyGlows, new Set(frames.filter(frame => frame.kind === "EnemyAttackTelegraph").map(frame => frame.eventId)));
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) return;
    const scale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
    for (const frame of frames) {
      const point = projectWorldPoint(frame.positionM, camera);
      const body = projectWorldPoint(frame.actorPositionM, camera);
      const style = enemyPlaceholderStyle(frame.entityType)!;
      let cue = this.clockworksEnemyGraphics.get(frame.eventId);
      if (!cue) {
        cue = new Graphics();
        cue.label = `temporary_visual=true:${frame.kind}:${frame.actorId}:${frame.eventId}`;
        this.clockworksEnemyGraphics.set(frame.eventId, cue); layer.addChild(cue);
      }
      cue.clear(); cue.scale.set(scale); cue.position.set(point.x, point.footY); cue.zIndex = 32000 + frame.eventId;
      cue.alpha = frame.reducedMotion ? 0.9 : 0.8 + Math.sin(frame.progress * Math.PI) * 0.1;
      if (frame.kind === "EnemyAttackTelegraph") {
        const boundary = enemyWarningGroundPoints(frame).map(position => projectWorldPoint(position, camera));
        cue.poly(boundary.flatMap(corner => [(corner.x - point.x) / scale, (corner.footY - point.footY) / scale]))
          .stroke({ color: 0xff384c, width: 1.3, alpha: 0.8 });
        // Bite can hit behind or beside the Hound. Its ring must not imply a safe rear arc.
        if (frame.attackKind !== "bite" && frame.attackKind !== "slam") {
          const end = projectWorldPoint({ xM: frame.positionM.xM + Math.sin(frame.directionRad) * frame.rangeM,
            yM: frame.positionM.yM, zM: frame.positionM.zM + Math.cos(frame.directionRad) * frame.rangeM }, camera);
          const dx = (end.x - point.x) / scale, dy = (end.footY - point.footY) / scale;
          const angle = Math.atan2(dy, dx);
          cue.moveTo(0, 0).lineTo(dx, dy).stroke({ color: 0xff384c, width: 1.6, alpha: 0.96 });
          cue.moveTo(dx - Math.cos(angle - 0.5) * 8, dy - Math.sin(angle - 0.5) * 8).lineTo(dx, dy)
            .lineTo(dx - Math.cos(angle + 0.5) * 8, dy - Math.sin(angle + 0.5) * 8)
            .stroke({ color: 0xff7078, width: 1.5, alpha: 0.95 });
          cue.ellipse(0, 0, style.widthPx * 0.5, 7).stroke({ color: 0xff384c, width: 1.5, alpha: 0.9 });
          if (frame.attackKind === "leap") cue.ellipse(dx, dy, 12, 5).stroke({ color: 0xffa88b, width: 1.6 });
        }
        let glow = this.clockworksEnemyGlows.get(frame.eventId);
        if (!glow) { glow = new Graphics(); this.clockworksEnemyGlows.set(frame.eventId, glow); layer.addChild(glow); }
        glow.label = `temporary_visual=true:enemy-body-warning:${frame.actorId}`;
        glow.clear().ellipse(0, 0, style.widthPx * 0.48, style.heightPx * 0.5).fill({ color: 0xff384c, alpha: 0.22 })
          .ellipse(0, 0, style.widthPx * 0.32, style.heightPx * 0.36).stroke({ color: 0xffa19c, width: 1.6, alpha: 0.8 });
        glow.scale.set(scale); glow.position.set(body.x, body.y - (style.liftPx + style.heightPx / 2) * scale);
        glow.alpha = cue.alpha; glow.blendMode = "add"; glow.zIndex = cue.zIndex + 0.1;
      } else if (frame.kind === "SignalBlink") {
        const end = projectWorldPoint({ xM: frame.positionM.xM+Math.sin(frame.directionRad)*frame.rangeM,
          yM: frame.positionM.yM,zM:frame.positionM.zM+Math.cos(frame.directionRad)*frame.rangeM },camera);
        cue.ellipse(0,0,16,6).stroke({color:style.tint,width:1.6})
          .ellipse((end.x-point.x)/scale,(end.footY-point.footY)/scale,16,6).stroke({color:style.tint,width:1.6});
      } else if (frame.kind === "SignalShotMotion") {
        cue.position.y = point.y;
        cue.circle(0,0,6).fill({color:0xe6ddff,alpha:0.9}).circle(0,0,9).stroke({color:style.tint,width:1.5});
      } else if (frame.kind === "PressureShotMotion") {
        cue.position.y = point.y;
        cue.circle(0, 0, 5).fill({ color: 0xc6faff, alpha: 0.85 }).circle(0, 0, 8).stroke({ color: 0x7bdded, width: 1.4 });
      } else if (frame.kind === "FurnaceHoundLeapMotion") {
        cue.position.y = point.y;
        cue.moveTo(-12, 0).lineTo(0, -5).lineTo(12, 0).lineTo(0, 5).closePath().stroke({ color: 0xffb66d, width: 2 });
      } else if (frame.kind === "EnemyChargeMotion") {
        cue.position.y = point.footY;
        cue.moveTo(-17, 5).lineTo(-8, 0).moveTo(17, 5).lineTo(8, 0)
          .stroke({ color: 0xd9c79e, width: 2.2, alpha: 0.85 });
      } else if (frame.kind === "EnemyLungeMotion") {
        cue.ellipse(0, 0, 25, 9).stroke({ color: style.tint, width: 1.5, alpha: 0.65 });
      } else if (frame.kind === "EnemyMemberHit" || frame.kind === "EnemyMemberDisperse") {
        const radius = frame.kind === "EnemyMemberDisperse" ? 8 : 5;
        cue.circle(0, -4, radius).stroke({ color: style.tint, width: 1.5 });
        if (frame.kind === "EnemyMemberDisperse") cue.moveTo(-7, -10).lineTo(-11, -13).moveTo(7, -10).lineTo(11, -13)
          .moveTo(-7, 1).lineTo(-11, 4).moveTo(7, 1).lineTo(11, 4).stroke({ color: style.tint, width: 1.3 });
      } else if (frame.kind === "EnemyAttackImpact") {
        cue.ellipse(0, 0, 14, 7).stroke({ color: style.tint, width: 2 });
        cue.moveTo(-9, -7).lineTo(9, 7).moveTo(9, -7).lineTo(-9, 7).stroke({ color: 0xffedc8, width: 1.5 });
      } else if (frame.kind === "EnemyAlert") {
        cue.position.set(body.x, body.y - (style.liftPx + style.heightPx + 6) * scale);
        cue.moveTo(0, -8).lineTo(0, -2).stroke({ color: 0xffdf97, width: 2.2 }).circle(0, 2, 1.5).fill(0xffdf97);
      } else if (frame.kind === "EnemyStagger") {
        cue.position.set(body.x, body.y - (style.liftPx + style.heightPx / 2) * scale);
        cue.moveTo(-12, -7).lineTo(-4, 3).lineTo(4, -3).lineTo(12, 7).stroke({ color: 0xffedcb, width: 2 });
      } else {
        const spread = frame.reducedMotion ? 0 : frame.progress * 8;
        cue.alpha = frame.reducedMotion ? 0.65 : 0.9 * (1 - frame.progress);
        cue.moveTo(-12 - spread, -3).lineTo(-5 - spread, -6).moveTo(5 + spread, -2).lineTo(12 + spread, 2)
          .moveTo(-2, 3 + spread / 2).lineTo(5, 5 + spread / 2).stroke({ color: style.tint, width: 2.5 });
      }
    }
  }

  private clearSentinelEncounter(): void {
    for (const item of [this.sentinelEncounterGraphic, this.sentinelEncounterGlow, this.sentinelEncounterLabel]) item?.destroy();
    this.sentinelEncounterGraphic = null; this.sentinelEncounterGlow = null; this.sentinelEncounterLabel = null;
  }

  /** Snapshot-owned telegraphs remain frozen and visible during entry and pause. */
  private renderSentinelEncounter(frame: SentinelEncounter | null, camera: CameraFrame, reducedMotion: boolean): void {
    if (!frame || (!frame.warning && frame.phase !== "hit" && frame.phase !== "stagger")) {
      this.clearSentinelEncounter(); return;
    }
    const floorLayer = this.layerContainers.get("L1_FLOOR");
    const bodyLayer = this.layerContainers.get("L3_ACTORS");
    const uiLayer = this.layerContainers.get("L8_WORLD_UI");
    if (!floorLayer || !bodyLayer || !uiLayer) throw new Error("E_SENTINEL_PRESENTATION_LAYER");
    if (!this.sentinelEncounterGraphic) {
      this.sentinelEncounterGraphic = new Graphics(); this.sentinelEncounterGraphic.label = "temporary_visual:sentinel-authority-ground";
      this.sentinelEncounterGlow = new Graphics(); this.sentinelEncounterGlow.label = "temporary_visual:sentinel-authority-glow";
      this.sentinelEncounterGlow.blendMode = "add";
      this.sentinelEncounterLabel = new Text({ text: "", style: { fontFamily: "Arial, sans-serif", fontSize: 13, fontWeight: "700", fill: 0xffedcc, stroke: { color: 0x101820, width: 3 } } });
      this.sentinelEncounterLabel.label = "temporary_visual:sentinel-phase-label";
      floorLayer.addChild(this.sentinelEncounterGraphic);
      bodyLayer.addChild(this.sentinelEncounterGlow);
      uiLayer.addChild(this.sentinelEncounterLabel);
    }
    const ground = this.sentinelEncounterGraphic, glow = this.sentinelEncounterGlow!, label = this.sentinelEncounterLabel!;
    ground.clear(); glow.clear();
    const scale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
    const body = projectWorldPoint(frame.positionM, camera);
    const color = frame.warning ? 0xff384c : frame.phase === "stagger" ? 0xffd86b : 0xffeddf;
    if (frame.warning) {
      const w = frame.warning;
      const points = w.shape === "circle" ? Array.from({length:64}, (_, i) => {
        const angle = i * 2 * Math.PI / 64;
        return { xM: w.originM.xM + Math.sin(angle) * w.radiusM, yM: w.originM.yM, zM: w.originM.zM + Math.cos(angle) * w.radiusM };
      }) : enemyWarningGroundPoints({ positionM:w.originM, directionRad:w.directionRad, rangeM:w.rangeM, radiusM:w.radiusM, attackKind:"charge" });
      const projected = points.map(point => projectWorldPoint(point, camera));
      ground.poly(projected.flatMap(point => [point.x, point.y])).closePath().fill({color,alpha:0.09}).stroke({color,width:2 * scale,alpha:0.95});
      if (w.shape === "corridor") {
        const start = projectWorldPoint(w.originM,camera);
        const tip = projectWorldPoint({xM:w.originM.xM + Math.sin(w.directionRad)*w.rangeM,yM:w.originM.yM,zM:w.originM.zM + Math.cos(w.directionRad)*w.rangeM},camera);
        ground.moveTo(start.x,start.y).lineTo(tip.x,tip.y).stroke({color:0xffc0c4,width:scale,alpha:0.85});
      }
    } else {
      // Broken outline and text distinguish actual Stagger from a short ordinary Hit without color alone.
      const r = frame.phase === "stagger" ? 17 : 10;
      ground.position.set(body.x,body.y);
      ground.moveTo(-r*scale,-5*scale).lineTo(-r*scale,4*scale).lineTo(-7*scale,8*scale)
        .moveTo(r*scale,-5*scale).lineTo(r*scale,4*scale).lineTo(7*scale,8*scale).stroke({color,width:2*scale});
    }
    if (frame.warning) ground.position.set(0,0);
    ground.zIndex=30000;
    glow.position.set(body.x,body.y-24*scale);glow.scale.set(scale);
    glow.zIndex=(this.sprites.get(`actor:${frame.actorId}`)?.sprite.zIndex ?? 0)+0.15;
    glow.ellipse(0,0,16,24).fill({color,alpha:reducedMotion?0.16:0.23}).stroke({color,width:1.5,alpha:0.75});
    const name = frame.phase === "charge_windup" ? "冲压预警" : frame.phase === "charge" ? "冲压" : frame.phase === "heavy_windup" ? "重击预警" : frame.phase === "light_windup" ? "攻击预警" : frame.phase === "stagger" ? "硬直 · 可反击" : "受击";
    label.text = `${name} ${(frame.remainingMs / 1000).toFixed(2)} 秒`;
    label.position.set(body.x,body.y-58*scale);label.anchor.set(0.5,1);label.scale.set(scale);label.zIndex=30002;
  }

  private renderSentinelTelegraphs(frames: readonly SentinelTelegraphFrame[], camera: CameraFrame): void {
    const visibleWarnings = new Set(frames.map(frame => frame.eventId));
    const visibleGlows = new Set(frames.filter(frame => frame.kind !== "SentinelDeath").map(frame => frame.eventId));
    this.releaseInvisibleGraphics(this.sentinelTelegraphGraphics, visibleWarnings);
    this.releaseInvisibleGraphics(this.sentinelGlowGraphics, visibleGlows);
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) return;
    const pixelsPerMeter = camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER;
    const presentationScale = pixelsPerMeter / BASE_PIXELS_PER_METER;
    for (const frame of frames) {
      const point = projectWorldPoint(frame.positionM, camera);
      let warning = this.sentinelTelegraphGraphics.get(frame.eventId);
      if (!warning) {
        warning = new Graphics();
        warning.label = `temporary_visual:sentinel-warning:${frame.eventId}`;
        layer.addChild(warning);
        this.sentinelTelegraphGraphics.set(frame.eventId, warning);
      }
      warning.clear();
      warning.scale.set(presentationScale);
      warning.position.set(point.x, point.footY + 2 * presentationScale);
      warning.zIndex = 30000 + frame.eventId;
      if (frame.kind === "SentinelDeath") {
        warning.alpha = frame.reducedMotion ? 0.62 : 0.76 * (1 - frame.progress);
        drawDeathFragments(warning, frame.reducedMotion ? 0 : frame.progress);
        continue;
      }

      const heavy = frame.kind === "SentinelHeavyWindup";
      const sway = frame.reducedMotion ? 0 : Math.sin(frame.ageTicks * 0.22) * 0.05;
      warning.alpha = (heavy ? 0.9 : 0.67) + sway;
      drawGroundWarning(warning, heavy ? 37 : 27, heavy ? 14 : 10, heavy ? 2.3 : 1.65);

      let glow = this.sentinelGlowGraphics.get(frame.eventId);
      if (!glow) {
        glow = new Graphics();
        glow.label = `temporary_visual:sentinel-warning-glow:${frame.eventId}`;
        glow.blendMode = "add";
        layer.addChild(glow);
        this.sentinelGlowGraphics.set(frame.eventId, glow);
      }
      glow.clear();
      glow.scale.set(presentationScale);
      glow.position.set(point.x, point.footY - 20 * presentationScale);
      glow.zIndex = 30001 + frame.eventId;
      glow.alpha = frame.reducedMotion ? (heavy ? 0.82 : 0.66) : (heavy ? 0.88 : 0.7) + sway;
      glow.ellipse(0, 0, heavy ? 19 : 14, heavy ? 27 : 21).fill({ color: 0xff384c, alpha: heavy ? 0.27 : 0.2 });
      glow.ellipse(0, 0, heavy ? 11 : 8, heavy ? 18 : 14).stroke({ color: 0xff9aa3, width: heavy ? 2 : 1.5, alpha: 0.7 });
    }
  }

  // temporary_visual=true: static Boss identity mark derived from the unique live Rust actor.
  private renderRegulatorBossMark(frame: PrimeRegulatorBossMarkFrame | null, camera: CameraFrame): void {
    if (!frame) {
      this.clearRegulatorBossMark();
      return;
    }
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) {
      this.clearRegulatorBossMark();
      return;
    }
    if (!this.regulatorBossMark) {
      const mark = new Graphics();
      mark.label = "temporary_visual=true:prime-regulator-boss-mark";
      layer.addChild(mark);
      this.regulatorBossMark = mark;
    }
    const mark = this.regulatorBossMark;
    const point = projectWorldPoint(frame.positionM, camera);
    const presentationScale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
    mark.clear();
    mark.scale.set(presentationScale);
    mark.position.set(point.x, point.footY - 42 * presentationScale);
    mark.zIndex = 33000;
    // The mark is fully static; reduced motion lowers its contrast without adding animation.
    mark.alpha = frame.reducedMotion ? 0.78 : 0.9;
    mark.circle(0, 0, 14).stroke({ color: 0xd99b59, width: 2.5, alpha: 0.95 });
    mark.circle(0, 0, 9).stroke({ color: 0xf2c27f, width: 1.2, alpha: 0.82 });
    mark.moveTo(0, -19).lineTo(0, -14).moveTo(0, 14).lineTo(0, 19)
      .moveTo(-19, 0).lineTo(-14, 0).moveTo(14, 0).lineTo(19, 0)
      .stroke({ color: 0xe7ad68, width: 2.1, alpha: 0.92 });
    mark.moveTo(0, -5).lineTo(5, 0).lineTo(0, 5).lineTo(-5, 0).closePath()
      .fill({ color: 0xffd58b, alpha: 0.94 });
  }

  private clearRegulatorBossMark(): void {
    if (!this.regulatorBossMark) return;
    this.regulatorBossMark.parent?.removeChild(this.regulatorBossMark);
    this.regulatorBossMark.destroy();
    this.regulatorBossMark = null;
  }

  // temporary_visual=true: brass footprint placeholder; final machinery art is still missing.
  private renderMovingMachinery(frames: readonly MovingMachineryFrame[], camera: CameraFrame): void {
    const visible = new Set(frames.map(frame => frame.entityId));
    for (const [id, graphic] of this.machineryGraphics) {
      if (!visible.has(id)) { graphic.parent?.removeChild(graphic); graphic.destroy(); this.machineryGraphics.delete(id); }
    }
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) { this.clearMovingMachinery(); return; }
    for (const frame of frames) {
      let graphic = this.machineryGraphics.get(frame.entityId);
      if (!graphic) {
        graphic = new Graphics();
        graphic.label = "temporary_visual=true:moving-machinery";
        layer.addChild(graphic);
        this.machineryGraphics.set(frame.entityId, graphic);
      }
      graphic.clear();
      graphic.zIndex = 31600;
      const points = frame.polygonM.map(([xM, zM]) => projectWorldPoint({ xM, yM: 0, zM }, camera));
      const first = points[0];
      if (!first) continue;
      graphic.moveTo(first.x, first.footY);
      for (const point of points.slice(1)) graphic.lineTo(point.x, point.footY);
      graphic.closePath();
      graphic.fill({ color: frame.warning ? 0xeecb7b : 0xa87836, alpha: frame.reducedMotion ? 0.22 : 0.3 });
      graphic.stroke({ color: frame.warning ? 0xffe3a5 : 0xdb9c51, width: frame.warning ? 2 : 3, alpha: 0.95 });
      // Static cross distinguishes mechanical impact from the orange furnace outline.
      const center = points.reduce((a, p) => ({ x: a.x + p.x / points.length, y: a.y + p.footY / points.length }), { x: 0, y: 0 });
      graphic.moveTo(center.x - 6, center.y - 6).lineTo(center.x + 6, center.y + 6)
        .moveTo(center.x + 6, center.y - 6).lineTo(center.x - 6, center.y + 6)
        .stroke({ color: 0xffe3a5, width: 2, alpha: 0.95 });
    }
  }

  private clearMovingMachinery(): void {
    for (const graphic of this.machineryGraphics.values()) { graphic.parent?.removeChild(graphic); graphic.destroy(); }
    this.machineryGraphics.clear();
  }

  // temporary_visual=true: readable ground geometry only; no admitted final hazard art.
  private renderEnvironmentHazards(frames: readonly EnvironmentHazardFrame[], camera: CameraFrame): void {
    const visible = new Set(frames.map(frame => frame.entityId));
    for (const [id, record] of this.environmentHazards) {
      if (!visible.has(id)) {
        record.graphic.parent?.removeChild(record.graphic); record.graphic.destroy();
        record.label.parent?.removeChild(record.label); record.label.destroy();
        this.environmentHazards.delete(id);
      }
    }
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) { this.clearEnvironmentHazards(); return; }
    for (const frame of frames) {
      let record = this.environmentHazards.get(frame.entityId);
      if (!record) {
        const graphic = new Graphics();
        graphic.label = `temporary_visual=true:environment:${frame.entityId}`;
        const label = new Text({ text: "", style: { fontFamily: "Arial, sans-serif", fontSize: 12,
          fontWeight: "700", fill: 0xf5f5e8, stroke: { color: 0x101820, width: 3 } } });
        layer.addChild(graphic, label); record = { graphic, label };
        this.environmentHazards.set(frame.entityId, record);
      }
      const points = frame.polygonM.map(([xM,zM]) => projectWorldPoint({ xM,yM:0,zM },camera));
      const first = points[0]; if (!first) continue;
      record.graphic.clear(); record.graphic.zIndex = 31400;
      record.graphic.moveTo(first.x, first.footY);
      for (const point of points.slice(1)) record.graphic.lineTo(point.x, point.footY);
      record.graphic.closePath().fill({ color:frame.color,alpha:frame.alpha })
        .stroke({ color:frame.color,width:frame.phase === "warning" ? 3 : 2,alpha:.9 });
      record.label.text = frame.label; record.label.zIndex = 31401;
      record.label.position.set(first.x+4, first.footY+4);
    }
  }

  private renderWardenWarning(frame: WardenEncounterFrame | null, camera: CameraFrame): void {
      const warning = frame?.warning;
      const layer = this.layerContainers.get("L7_VFX");
      if (!warning || !layer) {
          this.clearWardenWarning();
          return;
      }
      if (!this.wardenWarning) {
          const graphic = new Graphics();
          graphic.label = "temporary_visual=true:warden-warning";
          const label = new Text({ text: "", style: { fontFamily: "Arial, sans-serif", fontSize: 13, fontWeight: "700", fill: 0xffe4be, stroke: { color: 0x101820, width: 3 } } });
          layer.addChild(graphic, label);
          this.wardenWarning = { graphic, label };
      }
      const { graphic, label } = this.wardenWarning;
      graphic.clear();
      graphic.zIndex = 31500;
      label.zIndex = 31501;
      const points = warning.polygonM.map(([xM, zM]) => projectWorldPoint({ xM, yM: 0, zM }, camera));
      const first = points[0];
      if (!first)
          return;
      graphic.moveTo(first.x, first.footY);
      for (const p of points.slice(1))
          graphic.lineTo(p.x, p.footY);
      graphic.closePath().fill({ color: warning.color, alpha: warning.alpha }).stroke({ color: warning.color, width: 3, alpha: .95 });
      if (frame.source) {
          const p = projectWorldPoint(frame.source, camera);
          graphic.ellipse(p.x, p.footY - 24, 18, 30).stroke({ color: 0x58d7f0, width: 3, alpha: .95 });
      }
      label.text = warning.label;
      label.position.set(first.x + 5, first.footY + 5);
  }
  private clearWardenWarning(): void {
      if (!this.wardenWarning)
          return;
      for (const node of [this.wardenWarning.graphic, this.wardenWarning.label]) {
          node.parent?.removeChild(node);
          node.destroy();
      }
      this.wardenWarning = null;
  }

  private clearEnvironmentHazards(): void {
    for (const { graphic, label } of this.environmentHazards.values()) {
      graphic.parent?.removeChild(graphic); graphic.destroy();
      label.parent?.removeChild(label); label.destroy();
    }
    this.environmentHazards.clear();
  }

  // temporary_visual=true: this overlay uses only the Rust projected heat phase and zone.
  private renderClockworksHeat(frame: ClockworksHeatFrame | null, camera: CameraFrame): void {
    if (!frame) {
      this.clearClockworksHeat();
      return;
    }
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) {
      this.clearClockworksHeat();
      return;
    }
    if (!this.heatZoneGraphic) {
      this.heatZoneGraphic = new Graphics();
      this.heatZoneGraphic.label = "temporary_visual=true:clockworks-furnace-heat-zone";
      layer.addChild(this.heatZoneGraphic);
    }
    const graphic = this.heatZoneGraphic;
    graphic.clear();
    graphic.zIndex = 31500;
    const points = frame.polygonM.map(([xM, zM]) => projectWorldPoint({ xM, yM: 0, zM }, camera));
    const first = points[0];
    if (!first) return;
    graphic.moveTo(first.x, first.footY);
    for (const point of points.slice(1)) graphic.lineTo(point.x, point.footY);
    graphic.closePath();
    const warming = frame.warningRemainingMs !== null && frame.warningRemainingMs > 0;
    const alpha = frame.active ? (warming ? 0.22 : 0.13) : 0.045;
    graphic.fill({ color: warming ? 0xff9a45 : 0xe65c36, alpha: frame.reducedMotion ? alpha * 0.85 : alpha });
    graphic.stroke({ color: warming ? 0xffc36b : 0xf07545, width: 2, alpha: frame.active ? 0.82 : 0.38 });
  }

  private clearClockworksHeat(): void {
    if (!this.heatZoneGraphic) return;
    this.heatZoneGraphic.parent?.removeChild(this.heatZoneGraphic);
    this.heatZoneGraphic.destroy();
    this.heatZoneGraphic = null;
  }

  private clearVerticalSupports(): void {
    this.verticalSupportGraphic?.parent?.removeChild(this.verticalSupportGraphic);
    this.verticalSupportGraphic?.destroy();
    this.verticalSupportGraphic = null;
  }

  // temporary_visual=true: source-bound deck/lift footprint and elevation posts.
  // Every corner and the rider use the same authoritative snapshot, without a
  // renderer clock or independent platform animation.
  private renderVerticalSupports(frames: readonly VerticalSupportFrame[], camera: CameraFrame): void {
    if (!frames.length) { this.clearVerticalSupports(); return; }
    const layer = this.layerContainers.get("L1_FLOOR");
    if (!layer) throw new RenderCommitError("E_SUPPORT_RENDER_LAYER_MISSING");
    if (!this.verticalSupportGraphic) {
      this.verticalSupportGraphic = new Graphics();
      this.verticalSupportGraphic.label = "temporary_visual:vertical-support";
      layer.addChild(this.verticalSupportGraphic);
    }
    const graphic = this.verticalSupportGraphic;
    graphic.clear(); graphic.zIndex = 30010;
    for (const frame of frames) {
      const color = frame.rider ? 0x86e6ed : frame.kind === "moving" ? 0xf1bc60 : 0x8ba5b7;
      for (const [xM,zM] of frame.polygon) {
        const foot = projectWorldPoint({ xM,yM:0,zM },camera), top = projectWorldPoint({ xM,yM:frame.heightM,zM },camera);
        graphic.moveTo(foot.x,foot.y).lineTo(top.x,top.y).stroke({ color,width:1.2,alpha:0.45 });
      }
      const points = frame.polygon.flatMap(([xM,zM]) => { const p=projectWorldPoint({xM,yM:frame.heightM,zM},camera); return [p.x,p.y]; });
      graphic.poly(points).fill({color,alpha:0.26}).stroke({color,width:frame.rider?2.2:1.5,alpha:0.95});
    }
  }

  private clearMovingSurface(): void {
    this.movingSurfaceGraphic?.parent?.removeChild(this.movingSurfaceGraphic);
    this.movingSurfaceGraphic?.destroy();
    this.movingSurfaceGraphic = null;
  }

  // temporary_visual=true: readable authored belt footprint/direction, no gameplay authority.
  private clearFacilityFloor(): void {
    this.facilityFloorGraphic?.destroy(); this.facilityFloorGraphic = null;
    this.facilityWallGraphic?.destroy(); this.facilityWallGraphic = null;
    if (this.facilityFloorMesh) { const geometry=this.facilityFloorMesh.geometry; this.facilityFloorMesh.destroy(); geometry.destroy(true); this.facilityFloorMesh=null; }
  }

  private renderFacilityFloor(plan: FacilityFloor | undefined, camera: CameraFrame): void {
    if (!plan) { this.clearFacilityFloor(); return; }
    const floorLayer = this.layerContainers.get("L1_FLOOR")!;
    const wallLayer = this.layerContainers.get("L2_BACK_PROPS")!;
    const atlas=this.atlasTextures.get(plan.asset.atlasUrl);
    if(!atlas) throw new Error("E_FACILITY_ATLAS_NOT_LOADED");
    const data=facilityFloorGeometry(plan,camera,atlas.source.width,atlas.source.height);
    if(!this.facilityFloorMesh){this.facilityFloorMesh=new Mesh({geometry:new MeshGeometry(data),texture:atlas});floorLayer.addChild(this.facilityFloorMesh);}
    else {this.facilityFloorMesh.geometry.uvs=data.uvs;this.facilityFloorMesh.geometry.positions=data.positions;this.facilityFloorMesh.geometry.indices=data.indices;this.facilityFloorMesh.texture=atlas;}
    this.facilityFloorMesh.zIndex=-9999;
    if (!this.facilityFloorGraphic) { this.facilityFloorGraphic = new Graphics(); floorLayer.addChild(this.facilityFloorGraphic); }
    if (!this.facilityWallGraphic) { this.facilityWallGraphic = new Graphics(); wallLayer.addChild(this.facilityWallGraphic); }
    const {x,z,width,depth} = plan.bounds;
    const polygon = [[x,z],[x+width,z],[x+width,z+depth],[x,z+depth]] as const;
    const points = (vertices: readonly (readonly [number,number])[]) => vertices.flatMap(([xM,zM]) => { const p=projectWorldPoint({xM,yM:0,zM},camera); return [p.x,p.y]; });
    const floor=this.facilityFloorGraphic; floor.clear().poly(points(polygon)).fill(0x56636a); floor.zIndex=-10000;
    // Exact authoritative solid footprints; no invented wall crosses an authored opening.
    const walls=this.facilityWallGraphic; walls.clear(); walls.zIndex=-9999;
    for(const wall of plan.walls) walls.poly(points(wall)).fill({color:0x83939a,alpha:.8}).stroke({color:0xb7c4c8,width:1});
  }

  private renderMovingSurface(frames: readonly MovingSurfaceFrame[], camera: CameraFrame): void {
    if (frames.length === 0) { this.clearMovingSurface(); return; }
    const layer = this.layerContainers.get("L1_FLOOR");
    if (!layer) return;
    if (!this.movingSurfaceGraphic) {
      this.movingSurfaceGraphic = new Graphics();
      this.movingSurfaceGraphic.label = "temporary_visual:moving-surface";
      layer.addChild(this.movingSurfaceGraphic);
    }
    const graphic = this.movingSurfaceGraphic;
    graphic.clear(); graphic.zIndex = 30000;
    const project = (xM: number,zM: number) => projectWorldPoint({xM,yM:0,zM},camera);
    for (const frame of frames) {
      const points = frame.polygon.flatMap(([x,z]) => { const p=project(x,z); return [p.x,p.y]; });
      graphic.poly(points).fill({color:0x354551,alpha:0.3}).stroke({color:0xe5ae58,width:1.4,alpha:0.85});
      for (const arrow of frame.arrows) {
        const tip=project(arrow.x+arrow.dx*0.3,arrow.z+arrow.dz*0.3);
        const left=project(arrow.x-arrow.dx*0.2-arrow.dz*0.18,arrow.z-arrow.dz*0.2+arrow.dx*0.18);
        const right=project(arrow.x-arrow.dx*0.2+arrow.dz*0.18,arrow.z-arrow.dz*0.2-arrow.dx*0.18);
        graphic.moveTo(left.x,left.y).lineTo(tip.x,tip.y).lineTo(right.x,right.y).stroke({color:0xe5ae58,width:1.5,alpha:0.7});
      }
    }
  }

  // temporary_visual=true: procedural Pixi feedback only; it carries no gameplay authority.
  private renderPressureWave(frames: readonly ClockworksPressureWaveFrame[], camera: CameraFrame): void {
    const visible = new Set(frames.map(frame => frame.eventId));
    this.releaseInvisibleGraphics(this.pressureWaveGraphics, visible);
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) return;
    const presentationScale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
    for (const frame of frames) {
      const point = projectWorldPoint(frame.originM, camera);
      let ring = this.pressureWaveGraphics.get(frame.eventId);
      if (!ring) {
        ring = new Graphics();
        ring.label = `temporary_visual:clockworks-pressure-wave:${frame.kind}:${frame.eventId}`;
        layer.addChild(ring);
        this.pressureWaveGraphics.set(frame.eventId, ring);
      }
      ring.clear();
      ring.scale.set(presentationScale);
      ring.position.set(point.x, point.footY + 2 * presentationScale);
      ring.zIndex = 32000 + frame.eventId;
      const radius = frame.radiusM * BASE_PIXELS_PER_METER;
      if (frame.kind.endsWith("Windup")) {
        const pulse = frame.reducedMotion ? 0 : Math.sin(frame.progress * Math.PI * 18) * 0.05;
        ring.alpha = 0.82 + pulse;
        ring.ellipse(0, 0, radius, radius * 0.48).stroke({ color: 0xff6247, width: 2.4, alpha: 0.88 });
        ring.ellipse(0, 0, radius * 0.96, radius * 0.46).stroke({ color: 0xffbf78, width: 1, alpha: 0.48 });
        ring.circle(0, 0, 4).fill({ color: 0xff604b, alpha: 0.82 });
      } else {
        const expansion = frame.reducedMotion ? 1 : 0.7 + frame.progress * 0.3;
        ring.alpha = frame.reducedMotion ? 0.66 : 0.8 * (1 - frame.progress);
        ring.ellipse(0, 0, radius * expansion, radius * 0.48 * expansion)
          .stroke({ color: 0xffd09a, width: frame.reducedMotion ? 2 : 3, alpha: 0.9 });
        ring.ellipse(0, 0, radius * expansion * 0.7, radius * 0.34 * expansion)
          .stroke({ color: 0xff704d, width: 1.5, alpha: 0.7 });
      }
    }
  }

  /** Shared camera-scale path for the rig and every world projection in this frame. */
  private updateCameraFrame(snapshot: WorldSnapshotEnvelope, origin: Vec3, aimX: number, aimZ: number,
    width: number, height: number, deltaSeconds: number, basePixelsPerMeter = viewportPixelsPerMeter(width, height)):
    { camera: CameraFrame; pixelsPerMeter: number } {
    const pixelsPerMeter = basePixelsPerMeter * this.bossCameraModel.update(snapshot, deltaSeconds);
    const camera = this.cameraRig.update(origin, aimX, aimZ, width, height, deltaSeconds, pixelsPerMeter);
    return { camera, pixelsPerMeter };
  }

  private renderSoundCue(snapshot: WorldSnapshotEnvelope, nowMs: number): void {
    const cue = snapshot.protocolVersion === 3 ? this.soundCueModel.view(snapshot, nowMs) : null;
    if (!cue) { this.clearSoundCueUi(); return; }
    const layer = this.layerContainers.get("L8_WORLD_UI");
    if (!layer) return;
    if (!this.soundCueUi) {
      this.soundCueUi = new Container();
      this.soundCueUi.label = "sound-cue:signal-ping";
      this.soundCueUi.zIndex = 100000;
      layer.addChild(this.soundCueUi);
    }
    if (this.soundCueUiEventId !== cue.eventId) {
      this.soundCueUi.removeChildren().forEach(child => child.destroy());
      this.drawSoundCue(this.soundCueUi, cue);
      this.soundCueUiEventId = cue.eventId;
    }
    this.soundCueUi.position.set(this.app.screen.width / 2, Math.max(56, this.app.screen.height - 82));
  }

  private drawSoundCue(container: Container, cue: VisibleSoundCue): void {
    const background = new Graphics();
    background.roundRect(-105, -23, 210, 46, 11).fill({ color: 0x0b121a, alpha: 0.88 });
    background.roundRect(-105, -23, 210, 46, 11).stroke({ color: 0x58d7f0, width: 1.5, alpha: 0.9 });
    const arrow = new Graphics();
    const x = cue.screenDirectionX;
    const y = cue.screenDirectionY;
    const tipX = -75 + x * 13;
    const tipY = y * 13;
    arrow.moveTo(-75 - x * 9, -y * 9).lineTo(tipX, tipY).stroke({ color: 0x58d7f0, width: 3 });
    arrow.moveTo(tipX, tipY).lineTo(tipX - x * 7 - y * 5, tipY - y * 7 + x * 5)
      .lineTo(tipX - x * 7 + y * 5, tipY - y * 7 - x * 5).closePath().fill(0x58d7f0);
    const label = new Text({ text: `声学映射 · ${cue.sourceLabel} ${cue.distanceLabel}`,
      style: { fontFamily: "Arial, sans-serif", fontSize: 13, fontWeight: "700", fill: 0xe7edf2 } });
    label.anchor.set(0, 0.5);
    label.position.set(-50, 0);
    container.addChild(background, arrow, label);
  }

  private clearSoundCueUi(): void {
    if (!this.soundCueUi) return;
    this.soundCueUi.parent?.removeChild(this.soundCueUi);
    this.soundCueUi.destroy({ children: true });
    this.soundCueUi = null;
    this.soundCueUiEventId = null;
  }

  private renderWorldUi(snapshot: WorldSnapshotEnvelope, camera: CameraFrame): void {
    const layer = this.layerContainers.get("L8_WORLD_UI");
    if (!layer) return;
    const markers = snapshot.protocolVersion === 3
      ? projectWorldUi(snapshot, camera, this.actorPositionsForWorldUi) : [];
    const visible = new Set(markers.map(marker => marker.id));
    for (const id of this.worldUi.keys()) {
      if (!visible.has(id)) this.sceneResources.dispose(`world-ui:${id}`);
    }
    markers.forEach((marker, index) => {
      let record = this.worldUi.get(marker.id);
      if (!record) {
        record = this.createWorldUi(marker, layer);
        this.worldUi.set(marker.id, record);
        const owned = record;
        this.sceneResources.trackIfAbsent(`world-ui:${marker.id}`, () => {
          layer.removeChild(owned.container);
          owned.container.destroy({ children: true });
          this.worldUi.delete(marker.id);
        });
      }
      if (record.label && marker.label && record.label.text !== marker.label) record.label.text = marker.label;
      const uiScale = (camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER;
      record.container.scale.set(uiScale);
      const verticalOffset = (marker.kind === "signal_region" ? 0 : marker.kind === "enemy_vital" ? 34 : 24) * uiScale;
      record.container.position.set(marker.screenX, marker.screenY - verticalOffset);
      record.container.zIndex = index;
    });
  }

  private createWorldUi(marker: WorldUiMarker, layer: Container): WorldUiRecord {
    const container = new Container();
    container.label = `world-ui:${marker.id}`;
    const graphic = new Graphics();
    let label: Text | undefined;
    if (marker.kind === "interaction") {
      graphic.circle(0, 0, 8).fill({ color: 0x0b121a, alpha: 0.88 });
      graphic.circle(0, 0, 8).stroke({ color: 0x58d7f0, width: 2, alpha: 0.95 });
      const label = new Text({
        text: "F",
        style: { fontFamily: "Arial, sans-serif", fontSize: 13, fontWeight: "700", fill: 0xe7edf2 },
      });
      label.anchor.set(0.5, 0.5);
      label.position.set(0, -1);
      container.addChild(graphic, label);
    } else if (marker.kind === "signal_region") {
      // A broad ring and question mark indicate a search area, never an exact target.
      graphic.circle(0, 0, 28).fill({ color: 0x0b121a, alpha: 0.28 });
      graphic.circle(0, 0, 28).stroke({ color: 0x58d7f0, width: 2, alpha: 0.82 });
      graphic.circle(0, 0, 20).stroke({ color: 0xe7edf2, width: 1, alpha: 0.48 });
      const label = new Text({
        text: "?",
        style: { fontFamily: "Arial, sans-serif", fontSize: 19, fontWeight: "700", fill: 0xe7edf2 },
      });
      label.anchor.set(0.5, 0.5);
      container.addChild(graphic, label);
    } else if (marker.kind === "enemy_vital") {
      graphic.roundRect(-34, -10, 68, 20, 6).fill({ color: 0x0b121a, alpha: 0.88 });
      graphic.roundRect(-34, -10, 68, 20, 6).stroke({ color: 0x58d7f0, width: 1, alpha: 0.78 });
      label = new Text({
        text: marker.label ?? "",
        style: { fontFamily: "Arial, sans-serif", fontSize: 11, fontWeight: "700", fill: 0xe7edf2 },
      });
      label.anchor.set(0.5, 0.5);
      container.addChild(graphic, label);
    } else {
      graphic.circle(0, 0, 5).fill({ color: 0x58d7f0, alpha: 0.82 });
      graphic.circle(0, 0, 8).stroke({ color: 0xe7edf2, width: 1.5, alpha: 0.9 });
      container.addChild(graphic);
    }
    layer.addChild(container);
    return label ? { container, label } : { container };
  }

  private renderOccluders(occluders: ReturnType<typeof projectScenePresentation>["occluders"], playerPosition: Vec3, color: number): void {
    const visible = new Set(occluders.map(item => item.id));
    for (const [id, graphic] of this.sceneGraphics) {
      if (visible.has(id)) continue;
      this.sceneResources.dispose(`occluder:${id}`);
      this.occluderFader.forget(id);
    }
    const layer = this.layerContainers.get("L6_OCCLUDERS");
    if (!layer) return;
    for (let index = 0; index < occluders.length; index++) {
      const occluder = occluders[index];
      if (!occluder) continue;
      let graphic = this.sceneGraphics.get(occluder.id);
      if (!graphic) {
        graphic = new Graphics();
        graphic.label = `scene-occluder:${occluder.id}`;
        this.sceneGraphics.set(occluder.id, graphic);
        layer.addChild(graphic);
        const ownedGraphic = graphic;
        this.sceneResources.trackIfAbsent(`occluder:${occluder.id}`, () => {
          layer.removeChild(ownedGraphic);
          ownedGraphic.destroy();
          this.sceneGraphics.delete(occluder.id);
          this.occluderFader.forget(occluder.id);
        });
      }
      graphic.clear();
      graphic.poly(occluder.points.flatMap(([x, y]) => [x, y]));
      const covered = pointInPolygon(playerPosition.xM, playerPosition.zM, occluder.polygon);
      graphic.fill({ color, alpha: this.occluderFader.alpha(occluder.id, occluder.fadeTo, covered, performance.now()) });
      graphic.zIndex = index;
    }
  }

  private renderSceneGlows(glows: readonly SceneGlow[], skinTint: number, presentationScale: number): void {
    const visible = new Set(glows.map(glow => glow.id));
    for (const id of this.sceneGlows.keys()) {
      if (!visible.has(id)) this.sceneResources.dispose(`scene-glow:${id}`);
    }
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) return;
    for (let index = 0; index < glows.length; index++) {
      const glow = glows[index];
      if (!glow) continue;
      let graphic = this.sceneGlows.get(glow.id);
      if (!graphic) {
        graphic = new Graphics();
        graphic.label = `scene-glow:${glow.id}`;
        graphic.blendMode = "add";
        this.sceneGlows.set(glow.id, graphic);
        layer.addChild(graphic);
        const ownedGraphic = graphic;
        this.sceneResources.trackIfAbsent(`scene-glow:${glow.id}`, () => {
          layer.removeChild(ownedGraphic);
          ownedGraphic.destroy();
          this.sceneGlows.delete(glow.id);
        });
      }
      graphic.clear();
      // Two soft flat ellipses keep the cue local and subtle without introducing a new texture.
      graphic.ellipse(0, 0, glow.radiusX, glow.radiusY).fill({ color: glow.color, alpha: glow.alpha * 0.42 });
      graphic.ellipse(0, 0, glow.radiusX * 0.56, glow.radiusY * 0.56).fill({ color: glow.color, alpha: glow.alpha });
      graphic.tint = skinTint;
      graphic.scale.set(presentationScale);
      graphic.position.set(glow.screenX, glow.screenY);
      graphic.zIndex = 10000 + index;
    }
  }

  private renderActionVfx(vfx: ActionVfx | null, camera: CameraFrame, reducedMotion: boolean, playerDepth: number): void {
    const visible = new Set<string>(vfx ? [vfx.kind] : []);
    for (const [kind, graphic] of this.actionVfx) {
      if (visible.has(kind)) continue;
      graphic.parent?.removeChild(graphic);
      graphic.destroy();
      this.actionVfx.delete(kind);
    }
    if (!vfx) return;
    const layer = this.layerContainers.get(vfx.kind === "guard" ? "L3_ACTORS" : "L1_FLOOR");
    if (!layer) return;
    let graphic = this.actionVfx.get(vfx.kind);
    if (!graphic) {
      graphic = new Graphics();
      graphic.label = `action-vfx:${vfx.kind}`;
      graphic.blendMode = "add";
      this.actionVfx.set(vfx.kind, graphic);
      layer.addChild(graphic);
    }
    drawActionPresentation(graphic, vfx, camera, reducedMotion);
    graphic.zIndex = vfx.kind === "guard" ? playerDepth + .1 : Number.MAX_SAFE_INTEGER;
  }

  private renderWaterRipple(frame: WaterRippleFrame | null, camera: CameraFrame): void {
    if (!frame) { this.clearWaterRipple(); return; }
    const layer = this.layerContainers.get("L7_VFX");
    if (!layer) return;
    const foot = projectWorldPoint(frame.position, camera);
    if (!Number.isFinite(foot.x) || !Number.isFinite(foot.footY)) { this.clearWaterRipple(); return; }
    if (!this.waterRippleGraphic) {
      this.waterRippleGraphic = new Graphics();
      this.waterRippleGraphic.label = "water-ripple:mh_drowned_quay";
      layer.addChild(this.waterRippleGraphic);
    }
    const graphic = this.waterRippleGraphic;
    graphic.scale.set((camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER);
    const radius = frame.reducedMotion ? 9 : 7 + frame.progress * 10;
    const alpha = frame.reducedMotion ? 0.24 : 0.28 * (1 - frame.progress);
    graphic.clear();
    graphic.ellipse(0, 0, radius, radius * 0.36).stroke({ color: 0xa8e7ed, width: 1.5, alpha });
    graphic.ellipse(0, 0, radius * 0.58, radius * 0.21).stroke({ color: 0xe0f8f5, width: 1, alpha: alpha * 0.65 });
    graphic.position.set(foot.x, foot.footY + 3);
    graphic.zIndex = 9000;
  }

  private clearWaterRipple(): void {
    if (!this.waterRippleGraphic) return;
    this.waterRippleGraphic.parent?.removeChild(this.waterRippleGraphic);
    this.waterRippleGraphic.destroy();
    this.waterRippleGraphic = null;
  }

  private renderPumpSceneFeedback(
    frame: PumpSceneFeedbackFrame | null,
    camera: CameraFrame,
  ): void {
    if (!frame) { this.clearPumpSceneFeedback(); return; }
    const machineLayer = this.layerContainers.get("L4_DYNAMIC_PROPS");
    const vfxLayer = this.layerContainers.get("L7_VFX");
    if (!machineLayer || !vfxLayer) return;
    if (!this.pumpMachineGraphic) {
      this.pumpMachineGraphic = new Graphics();
      this.pumpMachineGraphic.label = "temporary_visual:pump-machinery:mh_pump_station";
      machineLayer.addChild(this.pumpMachineGraphic);
    }
    if (!this.pumpIndicatorGraphic) {
      this.pumpIndicatorGraphic = new Graphics();
      this.pumpIndicatorGraphic.label = "temporary_visual:pump-indicator:mh_pump_station";
      this.pumpIndicatorGraphic.blendMode = "add";
      vfxLayer.addChild(this.pumpIndicatorGraphic);
    }

    const pump = projectWorldPoint({ xM: 17, yM: 0, zM: 7 }, camera);
    const indicator = projectWorldPoint({ xM: 17, yM: 0, zM: 8 }, camera);
    const machine = this.pumpMachineGraphic;
    machine.scale.set((camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER);
    machine.clear();
    machine.position.set(pump.x, pump.footY);
    machine.zIndex = 9000;
    machine.ellipse(0, 2, 25, 8).fill({ color: 0x07151a, alpha: 0.56 });
    machine.moveTo(-23, -2).lineTo(-16, -11).lineTo(15, -11).lineTo(23, -2).closePath()
      .fill({ color: 0x263e43, alpha: 0.96 }).stroke({ color: 0x90aeb0, width: 1.4, alpha: 0.94 });
    machine.roundRect(-13 + frame.machineOffsetX, -31, 26, 20, 4)
      .fill({ color: 0x34565a, alpha: 0.98 }).stroke({ color: 0xb3c9c7, width: 1.3, alpha: 0.95 });
    machine.roundRect(-7 + frame.machineOffsetX, -39, 14, 8, 2)
      .fill({ color: 0x1d3338, alpha: 1 }).stroke({ color: 0x92acad, width: 1, alpha: 0.9 });
    machine.moveTo(0, -11).lineTo(0, -3).lineTo(15, -3).stroke({ color: 0x9fbfc0, width: 2.5, alpha: 0.9 });
    for (const x of [-17, 17]) machine.circle(x, -1, 1.8).fill({ color: 0xd5e2de, alpha: 0.9 });

    const light = this.pumpIndicatorGraphic;
    light.scale.set((camera.pixelsPerMeter ?? BASE_PIXELS_PER_METER) / BASE_PIXELS_PER_METER);
    light.clear();
    light.position.set(indicator.x, indicator.footY - 8);
    light.zIndex = 11000;
    light.ellipse(0, 0, 13, 7).fill({ color: 0x07151a, alpha: 0.62 });
    light.ellipse(0, 0, 10, 5).fill({ color: frame.indicatorColor, alpha: frame.indicatorAlpha * 0.34 });
    light.ellipse(0, 0, 8, 3.5).stroke({ color: frame.indicatorColor, width: 1.8, alpha: frame.indicatorAlpha });
    light.circle(0, 0, 2.2).fill({ color: frame.indicatorColor, alpha: frame.indicatorAlpha });
    if (frame.state === "draining" && !frame.reducedMotion) {
      const dx = indicator.x - pump.x;
      const dy = indicator.footY - pump.footY;
      for (let index = 0; index < 3; index++) {
        const t = (frame.motionPhase + index / 3) % 1;
        const x = -dx * t;
        const y = -dy * t - 12;
        light.circle(x, y, 1.7).fill({ color: 0x76eee0, alpha: frame.flowAlpha });
      }
      light.moveTo(-dx, -dy - 12).lineTo(0, -12)
        .stroke({ color: 0x4bd3c8, width: 1.2, alpha: frame.flowAlpha * 0.55 });
    }
  }

  private clearPumpSceneFeedback(): void {
    for (const graphic of [this.pumpMachineGraphic, this.pumpIndicatorGraphic]) {
      if (!graphic) continue;
      graphic.parent?.removeChild(graphic);
      graphic.destroy();
    }
    this.pumpMachineGraphic = null;
    this.pumpIndicatorGraphic = null;
  }

  private async preloadAtlasPages(pages: readonly { atlasUrl: string }[]): Promise<void> {
    const missing = [...new Set(pages.map(page => page.atlasUrl))].filter(url => !this.atlasTextures.has(url));
    if (!missing.length) return;
    const revision = this.sceneRequestRevision;
    const owner = Symbol("atlas-load");
    for (const url of missing) {
      const owners = this.atlasLoadOwners.get(url) ?? new Set<symbol>();
      owners.add(owner); this.atlasLoadOwners.set(url, owners);
    }
    try {
      // Drain the batch before releasing leases, including partial failures.
      const loaded = await Promise.allSettled(missing.map(async url => {
        // A new epoch arriving after retirement began must wait, then acquire
        // a new cached texture instead of borrowing one being destroyed.
        await this.atlasPendingUnloads.get(url);
        return [url, await Assets.load<Texture>(url)] as const;
      }));
      const failure = loaded.find(result => result.status === "rejected");
      if (failure?.status === "rejected") throw failure.reason;
      if (!this.ready || revision !== this.sceneRequestRevision) return;
      for (const result of loaded) {
        if (result.status === "fulfilled") {
          const [url, texture] = result.value;
          this.atlasTextures.set(url, texture);
          const residents = ATLAS_RESIDENT_OWNERS.get(url) ?? new Set<symbol>();
          residents.add(this.atlasResidentOwner); ATLAS_RESIDENT_OWNERS.set(url, residents);
        }
      }
    } finally {
      const retire: Promise<void>[] = [];
      for (const url of missing) {
        const owners = this.atlasLoadOwners.get(url);
        owners?.delete(owner);
        if (owners?.size) continue;
        this.atlasLoadOwners.delete(url);
        const unloading = retireUnownedAtlas(url);
        if (unloading) retire.push(unloading);
      }
      // Cleanup failures are observable; never report retirement as successful
      // while silently retaining a failed cache resource.
      await Promise.all(retire);
    }
  }

  private ensureSprite(key: string, asset: RuntimeAsset, layerId: RenderLayerId,
    selection: SpriteFrameSelection | undefined, displayScale: number): SpriteRecord {
    const frame = selection ?? {
      atlasUrl: asset.atlasUrl, atlasSha256: asset.atlasSha256, atlasPage: asset.atlasPage,
      atlasFrame: asset.atlasFrame, anchor: [asset.anchorX, asset.anchorY] as const, mode: "static" as const,
    };
    const previous = this.sprites.get(key);
    if (previous?.assetId === asset.assetId && previous.layer === layerId) {
      const nextTexture = this.getFrameTexture(asset, frame);
      const nextFrameKey = this.frameKey(frame);
      if (previous.frameKey !== nextFrameKey) {
        previous.sprite.texture = nextTexture;
        previous.sprite.anchor.set(frame.anchor[0], frame.anchor[1]);
        previous.frameTexture = nextTexture;
        previous.frameKey = nextFrameKey;
      }
      previous.sprite.scale.set(displayScale);
      return previous;
    }
    if (previous) this.removeSprite(key, previous);
    const frameTexture = this.getFrameTexture(asset, frame);
    const sprite = new Sprite(frameTexture);
    sprite.anchor.set(frame.anchor[0], frame.anchor[1]);
    sprite.scale.set(displayScale);
    sprite.blendMode = layerId === "L7_VFX" ? "add" : "normal";
    this.layerContainers.get(sharesWorldDepth(layerId, asset.assetId) ? "L3_ACTORS" : layerId)?.addChild(sprite);
    const record = { sprite, frameTexture, frameKey: this.frameKey(frame), assetId: asset.assetId, layer: layerId };
    this.sprites.set(key, record);
    return record;
  }

  private frameKey(frame: SpriteFrameSelection): string {
    return `${frame.atlasUrl}\u0000${frame.atlasPage}\u0000${frame.atlasFrame.join(",")}`;
  }

  private getFrameTexture(asset: RuntimeAsset, frame: SpriteFrameSelection): Texture {
    const atlas = this.atlasTextures.get(frame.atlasUrl);
    if (!atlas) throw new Error(`E_RENDERER_ATLAS_NOT_PRELOADED:${frame.atlasUrl}`);
    const [x, y, width, height] = frame.atlasFrame;
    const frameKey = this.frameKey(frame);
    let frameTexture = this.frameTextures.get(frameKey);
    if (!frameTexture) {
      frameTexture = new Texture({
        source: atlas.source,
        frame: new Rectangle(x, y, width, height),
        orig: new Rectangle(0, 0, width, height),
        label: `${asset.assetId}:${frame.mode}:${frame.atlasPage}:${frame.atlasFrame.join(",")}`,
      });
      this.frameTextures.set(frameKey, frameTexture);
    }
    return frameTexture;
  }

  private removeSprite(key: string, record: SpriteRecord): void {
    const mask=this.propCutoutMasks.get(key);if(mask){record.sprite.mask=null;mask.destroy();this.propCutoutMasks.delete(key);}
    this.actorFeedbackTints?.delete(key);
    this.sceneResources.forget(`sprite:${key}`);
    record.sprite.parent?.removeChild(record.sprite);
    record.sprite.destroy({ texture: false, textureSource: false });
    this.sprites.delete(key);
  }

  private destroyGraphics(graphics: Map<number, Graphics>): void {
    for (const graphic of graphics.values()) {
      graphic.parent?.removeChild(graphic);
      graphic.destroy();
    }
    graphics.clear();
  }

  private releaseInvisibleGraphics(graphics: Map<number, Graphics>, visible: ReadonlySet<number>): void {
    for (const [eventId, graphic] of graphics) {
      if (visible.has(eventId)) continue;
      graphic.parent?.removeChild(graphic);
      graphic.destroy();
      graphics.delete(eventId);
    }
  }

  private async clearSceneResources(): Promise<void> {
    this.clearEnvironmentHazards();
    this.clearSentinelEncounter();
    this.clearFacilityFloor();
    this.worldObjectDepths=new Map();
    this.clearPlayerLocomotionMesh();
    this.playerContactShadow?.parent?.removeChild(this.playerContactShadow);
    this.playerContactShadow?.destroy();
    this.playerContactShadow = null;
    this.clearMovingSurface();
    this.clearVerticalSupports();
    this.clearClockworksEnemyBodies();
    this.clearWardenWarning();
    this.clearSoundCueUi();
    this.clearTransientPresentation();
    this.cameraRig.reset();
    this.cameraLastTimeMs = null;
    this.sceneResources.clear();
    this.actionVfxModel.reset();
    this.fogLayerModel.reset();
    this.waterRippleModel.reset();
    this.clearWaterRipple();
    this.pumpSceneFeedbackModel.reset();
    this.clearPumpSceneFeedback();
    for (const graphic of this.actionVfx.values()) graphic.destroy();
    this.actionVfx.clear();
    this.occluderFader.clear();
    for (const [key, record] of this.sprites) this.removeSprite(key, record);
    for (const texture of this.frameTextures.values()) texture.destroy(false);
    this.frameTextures.clear();
    const urls = [...this.atlasTextures.keys()];
    this.atlasTextures.clear();
    const retirement: Promise<void>[] = [];
    for (const url of urls) {
      const residents = ATLAS_RESIDENT_OWNERS.get(url);
      residents?.delete(this.atlasResidentOwner);
      if (!residents?.size) ATLAS_RESIDENT_OWNERS.delete(url);
      const unloading = retireUnownedAtlas(url);
      if (unloading) retirement.push(unloading);
    }
    await Promise.all(retirement);
    for (const layer of this.layerContainers.values()) layer.removeChildren();
  }

  async destroy(): Promise<void> {
    const wasReady = this.ready;
    this.ready = false;
    this.committedFrame = null;
    this.sceneRequestRevision++;
    this.lastFrame = null;
    this.surface?.dispose(); this.surface = null;
    if (typeof window !== "undefined") window.removeEventListener("resize", this.resizeSurface);
    this.surfaceObserver?.disconnect(); this.surfaceObserver = null;
    this.surfaceCanvas?.removeEventListener("webglcontextlost", this.loseContext);
    this.surfaceCanvas?.removeEventListener("webglcontextrestored", this.restoreContext);
    this.surfaceCanvas = null;
    this.clearSoundCues();
    await this.clearSceneResources();
    if (wasReady) this.app.destroy(true);
    this.layerContainers.clear();
    this.sceneEpoch.reset();
    this.scenePresentation = null;
    this.expectedSceneKey = null;
  }
}

function drawGroundWarning(graphic: Graphics, radiusX: number, radiusY: number, width: number): void {
  const arcs: ReadonlyArray<readonly [number, number]> = [
    [-2.84, -1.05], [-0.42, 0.42], [1.05, 2.84],
  ];
  for (const [start, end] of arcs) {
    const steps = 12;
    for (let index = 0; index <= steps; index++) {
      const angle = start + (end - start) * index / steps;
      const x = Math.cos(angle) * radiusX;
      const y = Math.sin(angle) * radiusY;
      if (index === 0) graphic.moveTo(x, y);
      else graphic.lineTo(x, y);
    }
    graphic.stroke({ color: 0xff3d50, width, alpha: 0.92, cap: "round" });
  }
  graphic.moveTo(-5, 0).lineTo(5, 0).stroke({ color: 0xff8c98, width: Math.max(1, width * 0.65), alpha: 0.8 });
}

function drawDeathFragments(graphic: Graphics, progress: number): void {
  const scatter = progress * 14;
  for (let index = 0; index < 6; index++) {
    const angle = index * Math.PI / 3;
    const inner = 7 + scatter;
    const outer = 15 + scatter;
    graphic.moveTo(Math.cos(angle) * inner, -15 + Math.sin(angle) * inner)
      .lineTo(Math.cos(angle + 0.18) * outer, -15 + Math.sin(angle + 0.18) * outer);
  }
  graphic.stroke({ color: 0xdce5e6, width: 2.1, alpha: 0.86 });
  graphic.moveTo(-11 - scatter * 0.35, -29 - scatter * 0.2).lineTo(-3, -35 - scatter * 0.3)
    .lineTo(8 + scatter * 0.25, -30 - scatter * 0.2);
  graphic.moveTo(-9 - scatter * 0.2, -21).lineTo(-2, -26 - scatter * 0.25)
    .lineTo(11 + scatter * 0.3, -20);
  graphic.stroke({ color: 0xff7b83, width: 1.5, alpha: 0.8 });
}
