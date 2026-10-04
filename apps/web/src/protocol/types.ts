import { assertSentinelEncounter, type SentinelEncounter } from "./SentinelEncounter.js";
import { assertGreyHiveBeacon, type GreyHiveBeaconProjection } from "../game/GreyHiveBeacon.js";
export interface SignalPositionProjection { positionsM: Vec3[]; precise: boolean; uncertaintyRadiusM: number }
import { assertSupportScene, type SupportSceneProjection } from "./SupportProjection.js";
import { assertBuildProjection, type BuildProjection } from "./BuildProjection.js";
export interface Vec3 { xM: number; yM: number; zM: number }
export interface Transform { positionM: Vec3; yawRad: number }
export interface ActorMemberView { memberId: string; positionM: Vec3; radiusM: number; active: boolean }
export interface ActorView { entityId: string; entityType: string; actorKind: string; transform: Transform; active: boolean; members?: ActorMemberView[]; signalPerception?: SignalPositionProjection }
export interface PlayerView {
  entityId: string;
  transform: Transform;
  velocityMps: Vec3;
  currentHp: number;
  maxHp: number;
  currentEnergy: number;
  maxEnergy: number;
}
export interface PlayerViewV3 extends PlayerView {
  facingX: number;
  facingZ: number;
  aimX: number;
  aimZ: number;
  actionState: string;
  /** Rust-authored whole-action counters; no frontend cooldown or phase simulation. */
  actionPresentation?: ActionPresentation;
}
export interface ActionPresentation {
  requestId: number;
  phase: "windup" | "active" | "recovery";
  elapsedMs: number;
  durationMs: number;
  rangeM: number;
  lineHalfWidthM: number;
}

export interface CombatFeedback {
  worldId: string;
  sceneId: string;
  outcome: "enemy_hit" | "player_hurt" | "blocked" | "absorbed" | "rejected";
  targetId?: string;
  sourceId?: string;
  requestId?: number;
  reason?: string;
}
export interface DoorView { doorId: string; transform: Transform; open: boolean; locked: boolean }

/** Frozen v2 read shape retained for persisted fixtures and one-way migration tests. */
export interface WorldViewV2 {
  protocol: "continuous-ipc";
  version: number;
  schemaVersion: string;
  worldId: string;
  worldEpoch: number;
  serverTick: number;
  authorityRevision: number;
  ackSeq: number;
  serverTimeMs: number;
  player: PlayerView;
  actors: ActorView[];
  doors: DoorView[];
  capabilities?: CapabilityProjection;
  progression?: RouteProjection;
}
export interface WorldSnapshotV2 {
  kind: "full";
  protocolVersion: 2;
  schemaVersion: string;
  worldId: string;
  worldEpoch: number;
  serverTick: number;
  authorityRevision: number;
  ackSeq: number;
  view: WorldViewV2;
}

export interface InteractableView { entityId: string; kind: string; transform: Transform; active: boolean }
export interface EnvironmentHazardProjection {
  phase: "warning" | "active" | "recovery" | "suppressed";
  remainingMs: number;
  exposureBps: number;
  tag: "heat" | "pressure" | "toxin";
}
export interface HazardView {
  environment?: EnvironmentHazardProjection;
  entityId: string;
  kind: string;
  transform: Transform;
  active: boolean;
  polygonM?: Array<[number, number]>;
  warningRemainingMs?: number | null;
  phaseActive?: boolean;
}
export interface ObjectiveView { objectiveId: string; state: string; positionM: Vec3 }
export interface CapabilityItemProjection {
  capabilityId: string;
  granted: boolean;
  selected: boolean;
  cooldownRemainingMs: number;
}
export interface ExploredRoomProjection { roomId: string; outlineM: Vec3[] }
export interface KnownConnectionProjection { connectionId: string; fromRoomId: string; toRoomId: string }
export interface KnownObjectiveProjection { objectiveId: string; positionM: Vec3 }
export interface ExploredMapProjection {
  worldId: string;
  playerPositionM: Vec3;
  rooms: ExploredRoomProjection[];
  connections: KnownConnectionProjection[];
  objectives: KnownObjectiveProjection[];
}
export interface RearViewAuthorizationProjection {
  granted: boolean;
  grantId: string | null;
  grantedAtRevision: number | null;
}
/** Rust-filtered current-scene knowledge, separate from actual exploration history. */
export type MapKnowledgeChannel = "topology" | "terrain" | "connections" | "objectives" |
  "enemies" | "hazards" | "loot" | "npcs" | "secrets";
export type MapKnowledgeLevel = "none" | "explored_only" | "detected_only" | "known" | "full";
export interface MapKnowledgePoint { x: number; y: number }
export interface MapKnowledgeProjection {
  regions: Array<{ id: string; channel: MapKnowledgeChannel;
    polygon: { vertices: MapKnowledgePoint[] }; terrain: string | null; boundary: string | null }>;
  features: Array<{ id: string | null; channel: MapKnowledgeChannel; position: MapKnowledgePoint;
    knowledge: MapKnowledgeLevel; terrain: string | null; hazard: string | null;
    obstacle: string | null; boundary: string | null }>;
}
export interface CapabilityProjection {
  schemaVersion: number;
  firstEnhancementChoice?: string | null;
  items?: CapabilityItemProjection[];
  exploredMap?: ExploredMapProjection;
  /** Effective channel authorization; does not imply acquiring a permanent capability. */
  mapTopologyAuthorized?: boolean;
  acousticMappingAuthorized?: boolean;
  mapKnowledge?: MapKnowledgeProjection;
  enemyVitals?: Array<{ entityId: string; tier: "healthy" | "wounded" | "severelyWounded" | "critical" }>;
  rearView?: RearViewAuthorizationProjection;
  [key: string]: unknown;
}
export interface RouteProjection { schemaVersion: number; currentWorldId: string; eventSeq: number; worlds: unknown[] }

/** Exact transient authority identity for lifecycle commands; never saved. */
export interface SessionContext {
  worldId: string;
  sceneId: string;
  worldEpoch: number;
}

export function assertSessionContext(value: unknown): SessionContext {
  if (!value || typeof value !== "object") throw new Error("E_SESSION_CONTEXT_INVALID");
  const context = value as Record<string, unknown>;
  if (!safeCounter(context.worldEpoch) || typeof context.worldId !== "string" || !context.worldId ||
      typeof context.sceneId !== "string" || !context.sceneId) throw new Error("E_SESSION_CONTEXT_INVALID");
  return value as SessionContext;
}

/** Ephemeral renderer-entry challenge. Never part of a save schema. */
export interface SceneEntryToken {
  generation: number;
  worldId: string;
  sceneId: string;
  worldEpoch: number;
}

export function assertSceneEntryToken(value: unknown): SceneEntryToken {
  if (!value || typeof value !== "object") throw new Error("E_SCENE_ENTRY_TOKEN_INVALID");
  const token = value as Record<string, unknown>;
  if (!safeCounter(token.generation) || token.generation === 0 ||
      !safeCounter(token.worldEpoch) || typeof token.worldId !== "string" || !token.worldId ||
      typeof token.sceneId !== "string" || !token.sceneId) throw new Error("E_SCENE_ENTRY_TOKEN_INVALID");
  return value as SceneEntryToken;
}

/** Read-only, optional Bio projection. Invalid optional data is isolated by BaizhiPresentation. */
export type BaizhiChoice = "unresolved" | "taken" | "left";
export interface BaizhiProjection { schemaVersion: 1; choice: BaizhiChoice; available: boolean; canInteract: boolean }
export interface BaizhiNpcProjection {
  entityId: "gh_bz_whitezhi_v1";
  entityType: "npc.baizhi";
  position: [number, number, number];
  yawRad: number;
  interactable: boolean;
}
/** Transient authority ticket, never part of Save or an inferred snapshot field. */
export interface BaizhiDialogueTicket extends SessionContext {
  ownerId: string;
  generation: number;
  entityId: "gh_bz_whitezhi_v1";
  interactionId: "gh_bz_first_contact";
  pauseCommandSequence: number;
}

/** Canonical flattened authoritative snapshot for the v3 wire contract. */
export interface WorldSnapshotV3 {
  kind: "full";
  protocolVersion: 3;
  schemaVersion: string;
  worldId: string;
  sceneId: string;
  checkpointId: string | null;
  entryToken?: SceneEntryToken;
  supportScene?: SupportSceneProjection;
  baizhi?: BaizhiProjection;
  npcs?: BaizhiNpcProjection[];
  greyHiveBeacon?: GreyHiveBeaconProjection;
  sentinelEncounter?: SentinelEncounter;
  worldEpoch: number;
  serverTick: number;
  authorityRevision: number;
  ackSeq: number;
  player: PlayerViewV3;
  actors: ActorView[];
  doors: DoorView[];
  interactables: InteractableView[];
  hazards: HazardView[];
  objectives: ObjectiveView[];
  capabilities: CapabilityProjection;
  progression: RouteProjection;
  /** Optional for older v3 senders; absence never implies an empty authorized inventory. */
  build?: BuildProjection;
  bossEncounter?: BossEncounterProjection;
  /** Present only in Mist Harbor; projected from Rust's persistent pump state. */
  mistHarborPump?: { state: "ready" | "draining" | "drained" };
}
export interface BossEncounterProjection {
  entityId: string; entityType: string; displayName: string;
  currentHp: number; maxHp: number; phase: number; state: string;
  temporaryVisual: boolean; publicReleaseEligible: boolean; mappedTrueSource: boolean;
  warning?: { attackSerial: number; kind: "strike" | "pulse" | "decoy"; originM: [number, number, number];
    directionRad: number; radiusM: number; halfAngleRad: number; remainingMs: number; ordinaryVisible: boolean };
}
/** Legacy API type kept for the renderer and v2 fixture migration path. */
export type WorldSnapshot = WorldSnapshotV2;
export type WorldSnapshotEnvelope = WorldSnapshotV2 | WorldSnapshotV3;

export interface InputStateV2 {
  protocol: "continuous-input";
  protocolVersion: 2;
  worldEpoch: number;
  seq: number;
  clientTimeMs: number;
  moveX: number;
  moveZ: number;
  aimX: number;
  aimZ: number;
}
export type InputState = InputStateV2;

export type ActionKind = "primaryAttack" | "dash" | "pulse" | "guardStart" | "guardEnd" | "pierce" | "interact" | "contextTraversal";
export interface ActionCommandV2 {
  protocolVersion: 2;
  worldEpoch: number;
  requestId: number;
  clientTimeMs: number;
  kind: ActionKind;
}
export type ActionCommand = ActionCommandV2;

/** Event IDs are monotonic only within worldEpoch; consumers deduplicate that pair. */
export interface PresentationEventV2 {
  attackId?: number;
  combatFeedback?: CombatFeedback;
  /** Optional ordinary-enemy cue identity and Rust-authored remaining lifetime at serverTick. */
  actorId?: string;
  memberId?: string;
  halfAngleRad?: number;
  durationMs?: number;
  /** Committed attack reach; radiusM remains attack/projectile width. */
  rangeM?: number;
  attackKind?: "pressure_shot" | "bite" | "leap" | "charge" | "slam" | "lunge" | "tide_swing" | "tide_charge" | "signal_shot";
  protocolVersion: 2;
  eventId: number;
  worldEpoch: number;
  serverTick: number;
  kind: string;
  positionM: Vec3;
  directionRad: number;
  radiusM: number;
  intensity: number;
}
export interface PresentationEventV1 {
  protocolVersion: 1;
  eventId: number;
  worldEpoch: number;
  serverTick: number;
  kind: string;
  positionM: Vec3;
  directionRad: number;
  radiusM: number;
  intensity: number;
}
export type PresentationEvent = PresentationEventV1 | PresentationEventV2;

/** Ephemeral Rust-authored sound projection; no source coordinates cross this wire. */
export interface SoundCueEvent {
  protocolVersion: 1;
  eventId: number;
  worldEpoch: number;
  serverTick: number;
  worldId: string;
  sceneId: string;
  kind: "signal_ping" | "warden_true_call";
  directionRad: number | null;
  distanceM: number | null;
}

function safeCounter(value: unknown): value is number {
  return Number.isSafeInteger(value) && (value as number) >= 0;
}

export function assertSnapshot(value: unknown): WorldSnapshotV2 {
  if (value === null || typeof value !== "object") throw new Error("E_SNAPSHOT_PROTOCOL");
  const snapshot = value as Record<string, unknown>;
  if (snapshot.kind !== "full" || typeof snapshot.worldId !== "string" ||
      !safeCounter(snapshot.worldEpoch) || !safeCounter(snapshot.serverTick) ||
      !safeCounter(snapshot.authorityRevision) || !safeCounter(snapshot.ackSeq)) {
    throw new Error("E_SNAPSHOT_PROTOCOL");
  }
  if (snapshot.protocolVersion === 2) {
    const view = snapshot.view as Record<string, unknown> | null;
    if (!view || view.worldEpoch !== snapshot.worldEpoch || view.serverTick !== snapshot.serverTick ||
        !view.player || !Array.isArray(view.actors) || !Array.isArray(view.doors)) {
      throw new Error("E_SNAPSHOT_PROTOCOL");
    }
    return value as WorldSnapshotV2;
  }
  throw new Error("E_SNAPSHOT_PROTOCOL");
}

export function assertSnapshotV3(value: unknown): WorldSnapshotV3 {
  if (value === null || typeof value !== "object") throw new Error("E_SNAPSHOT_PROTOCOL");
  const snapshot = value as Record<string, unknown>;
  if (snapshot.kind === "full" && snapshot.protocolVersion === 3 &&
      typeof snapshot.worldId === "string" && typeof snapshot.schemaVersion === "string" &&
      typeof snapshot.sceneId === "string" &&
      (snapshot.checkpointId === null || typeof snapshot.checkpointId === "string") &&
      safeCounter(snapshot.worldEpoch) && safeCounter(snapshot.serverTick) &&
      safeCounter(snapshot.authorityRevision) && safeCounter(snapshot.ackSeq) &&
      snapshot.player && typeof snapshot.player === "object" &&
      validPlayerV3(snapshot.player) &&
      Array.isArray(snapshot.actors) && Array.isArray(snapshot.doors) &&
      Array.isArray(snapshot.interactables) && Array.isArray(snapshot.hazards) &&
      Array.isArray(snapshot.objectives) && snapshot.capabilities && typeof snapshot.capabilities === "object" &&
      snapshot.progression && typeof snapshot.progression === "object") {
    const choice = (snapshot.capabilities as Record<string, unknown>).firstEnhancementChoice;
    if (choice !== undefined && choice !== null && (typeof choice !== "string" || !["information.local_map_i", "perception.rear_view_i", "body.regeneration_i"].includes(choice))) {
      throw new Error("E_FIRST_ENHANCEMENT_CHOICE_INVALID");
    }
    assertSentinelEncounter(snapshot as unknown as WorldSnapshotV3);
    if (Object.hasOwn(snapshot, "greyHiveBeacon")) assertGreyHiveBeacon(snapshot.greyHiveBeacon, snapshot.worldId as string);
    if (Object.hasOwn(snapshot, "build")) assertBuildProjection(snapshot.build);
    if (Object.hasOwn(snapshot, "supportScene")) assertSupportScene(snapshot.supportScene, snapshot as unknown as WorldSnapshotV3);
    if (Object.hasOwn(snapshot, "entryToken")) {
      const token = assertSceneEntryToken(snapshot.entryToken);
      if (token.worldId !== snapshot.worldId || token.sceneId !== snapshot.sceneId || token.worldEpoch !== snapshot.worldEpoch) {
        throw new Error("E_SCENE_ENTRY_TOKEN_IDENTITY");
      }
    }
    return value as WorldSnapshotV3;
  }
  throw new Error("E_SNAPSHOT_PROTOCOL");
}

function finiteAxis(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && Math.abs(value) <= 1;
}
function validPlayerV3(value: unknown): value is PlayerViewV3 {
  if (!value || typeof value !== "object") return false;
  const player = value as Record<string, unknown>;
  const transform = player.transform as Record<string, unknown> | null;
  const position = transform && transform.positionM as Record<string, unknown> | null;
  return !!position && [position.xM, position.yM, position.zM].every(value => typeof value === "number" && Number.isFinite(value)) &&
    finiteAxis(player.facingX) && finiteAxis(player.facingZ) &&
    finiteAxis(player.aimX) && finiteAxis(player.aimZ) && typeof player.actionState === "string" &&
    (player.actionPresentation === undefined || validActionPresentation(player.actionPresentation, player.actionState));
}

export function validActionPresentation(value: unknown, actionState: string): value is ActionPresentation {
  if (!value || typeof value !== "object" || !["primaryAttack", "dash", "pulse", "guard", "pierce"].includes(actionState)) return false;
  const phase = value as ActionPresentation;
  return Number.isSafeInteger(phase.requestId) && phase.requestId > 0 &&
    ["windup", "active", "recovery"].includes(phase.phase) &&
    Number.isSafeInteger(phase.elapsedMs) && phase.elapsedMs >= 0 &&
    Number.isSafeInteger(phase.durationMs) && phase.durationMs > 0 && phase.durationMs <= 60_000 &&
    phase.elapsedMs < phase.durationMs && Number.isFinite(phase.rangeM) && phase.rangeM >= 0 && phase.rangeM <= 64 &&
    Number.isFinite(phase.lineHalfWidthM) && phase.lineHalfWidthM >= 0 && phase.lineHalfWidthM <= 8;
}

export function assertInputV2(value: unknown): InputStateV2 {
  if (!value || typeof value !== "object") throw new Error("E_INPUT_PROTOCOL");
  const input = value as Record<string, unknown>;
  if (input.protocol !== "continuous-input" || input.protocolVersion !== 2 ||
      !safeCounter(input.worldEpoch) || !safeCounter(input.seq) || !safeCounter(input.clientTimeMs) ||
      !finiteAxis(input.moveX) || !finiteAxis(input.moveZ) || !finiteAxis(input.aimX) || !finiteAxis(input.aimZ)) {
    throw new Error("E_INPUT_PROTOCOL");
  }
  return value as InputStateV2;
}

export function assertActionV2(value: unknown): ActionCommandV2 {
  if (!value || typeof value !== "object") throw new Error("E_ACTION_PROTOCOL");
  const action = value as Record<string, unknown>;
  const kinds: ActionKind[] = ["primaryAttack", "dash", "pulse", "guardStart", "guardEnd", "pierce", "interact", "contextTraversal"];
  if (action.protocolVersion !== 2 || !safeCounter(action.worldEpoch) || !safeCounter(action.requestId) ||
      action.requestId === 0 || !safeCounter(action.clientTimeMs) || !kinds.includes(action.kind as ActionKind)) {
    throw new Error("E_ACTION_PROTOCOL");
  }
  return value as ActionCommandV2;
}
