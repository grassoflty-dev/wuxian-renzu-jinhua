import { resolveBaizhiPresentation } from "../renderer/BaizhiPresentation.js";
import { assertBuildAction, type BuildAction } from "../protocol/BuildProjection.js";
import type { BaizhiChoice, BaizhiDialogueTicket, ActionCommand, InputState, PresentationEvent, SoundCueEvent, SceneEntryToken, SessionContext, WorldSnapshotV3 } from "../protocol/types.js";
import { assertSessionContext, assertSceneEntryToken, assertSnapshotV3 } from "../protocol/types.js";

export type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

export interface SaveSlotSummary {
  slotId: string;
  displayName: string;
  updatedAtMs: number;
  worldId: string | null;
  checkpointId: string | null;
  playerPositionM: { xM: number; yM: number; zM: number } | null;
  currentHp: number | null;
  maxHp: number | null;
  currentEnergy: number | null;
  maxEnergy: number | null;
  gateOpen: boolean | null;
  completedEvents: string[];
  readOnly: boolean;
  recoverable?: boolean;
  valid: boolean;
  errorCode: string | null;
}

export interface SlotCommandReceipt {
  commandId: string;
  applied: boolean;
  alreadyApplied: boolean;
  errorCode: string | null;
  worldEpoch: number;
  serverTick: number;
  authorityRevision: number;
  snapshot: WorldSnapshotV3;
}

export interface BaizhiCommandReceipt { receipt: SlotCommandReceipt; ticket: BaizhiDialogueTicket | null; resumed: boolean }

function assertBaizhiTicket(value: unknown): BaizhiDialogueTicket {
  if (!value || typeof value !== "object") throw new Error("E_BAIZHI_TICKET_INVALID");
  const ticket = value as BaizhiDialogueTicket;
  assertSessionContext(ticket);
  if (typeof ticket.ownerId !== "string" || !ticket.ownerId || !Number.isSafeInteger(ticket.generation) || ticket.generation < 1 ||
      !Number.isSafeInteger(ticket.pauseCommandSequence) || ticket.pauseCommandSequence < 1 || ticket.worldEpoch < 1 ||
      ticket.worldId !== "grey_hive" || ticket.sceneId !== "gh_bio_isolation" ||
      ticket.entityId !== "gh_bz_whitezhi_v1" || ticket.interactionId !== "gh_bz_first_contact") throw new Error("E_BAIZHI_TICKET_INVALID");
  return { ownerId: ticket.ownerId, generation: ticket.generation, worldId: ticket.worldId, sceneId: ticket.sceneId,
    worldEpoch: ticket.worldEpoch, entityId: ticket.entityId, interactionId: ticket.interactionId,
    pauseCommandSequence: ticket.pauseCommandSequence };
}
function sameBaizhiTicket(a: BaizhiDialogueTicket, b: BaizhiDialogueTicket): boolean {
  return a.ownerId === b.ownerId && a.generation === b.generation && a.worldId === b.worldId && a.sceneId === b.sceneId &&
    a.worldEpoch === b.worldEpoch && a.entityId === b.entityId && a.interactionId === b.interactionId &&
    a.pauseCommandSequence === b.pauseCommandSequence;
}
function assertBaizhiReceipt(value: unknown, commandId: string, ticket: BaizhiDialogueTicket, closing = false): BaizhiCommandReceipt {
  if (!value || typeof value !== "object") throw new Error("E_BAIZHI_RECEIPT_INVALID");
  const result = value as BaizhiCommandReceipt;
  const receipt = assertSlotReceipt(result.receipt, commandId), snapshot = receipt.snapshot;
  if (receipt.commandId !== commandId || receipt.applied !== true || receipt.alreadyApplied !== false || receipt.errorCode !== null ||
      snapshot.worldId !== ticket.worldId || snapshot.sceneId !== ticket.sceneId || snapshot.worldEpoch !== ticket.worldEpoch ||
      snapshot.entryToken || snapshot.player.currentHp <= 0 || (!closing && !resolveBaizhiPresentation(snapshot)) || typeof result.resumed !== "boolean" ||
      (closing ? result.ticket !== null : result.resumed || !sameBaizhiTicket(assertBaizhiTicket(result.ticket), ticket))) {
    throw new Error("E_BAIZHI_RECEIPT_INVALID");
  }
  return { receipt, ticket: closing ? null : assertBaizhiTicket(result.ticket), resumed: result.resumed };
}

export type FirstEnhancementReceipt = SlotCommandReceipt;
export interface EnhancementTerminalContext { id: string; requestId: string; worldEpoch: number; pauseCommandSequence: number }


export interface SceneCommandReceipt {
  commandId: string;
  applied: boolean;
  alreadyApplied: boolean;
  errorCode: string | null;
  worldEpoch: number;
  serverTick: number;
  authorityRevision: number;
  snapshot: WorldSnapshotV3;
  requestId: string;
}

function assertSlotReceipt(value: unknown, command: string): SlotCommandReceipt {
  if (!value || typeof value !== "object") throw new Error("E_SLOT_RECEIPT_INVALID");
  const receipt = value as Record<string, unknown>;
  const snapshot = assertSnapshotV3(receipt.snapshot);
  if (typeof receipt.commandId !== "string" || typeof receipt.applied !== "boolean" ||
      typeof receipt.alreadyApplied !== "boolean" ||
      (receipt.errorCode !== null && typeof receipt.errorCode !== "string") ||
      receipt.worldEpoch !== snapshot.worldEpoch || receipt.serverTick !== snapshot.serverTick ||
      receipt.authorityRevision !== snapshot.authorityRevision) {
    throw new Error("E_SLOT_RECEIPT_SNAPSHOT_MISMATCH");
  }
  if (!receipt.applied && !receipt.alreadyApplied) {
    throw new Error(`E_SLOT_COMMAND_REJECTED:${String(receipt.errorCode || command)}`);
  }
  return { ...receipt, snapshot } as SlotCommandReceipt;
}

function assertSlotSummary(value: unknown): SaveSlotSummary {
  if (!value || typeof value !== "object") throw new Error("E_SAVE_SLOT_SUMMARY_INVALID");
  const slot = value as Record<string, unknown>;
  if (typeof slot.slotId !== "string" || typeof slot.displayName !== "string" ||
      typeof slot.updatedAtMs !== "number" || typeof slot.readOnly !== "boolean" ||
      (slot.recoverable !== undefined && typeof slot.recoverable !== "boolean") ||
      typeof slot.valid !== "boolean" || !Array.isArray(slot.completedEvents) ||
      (slot.errorCode !== null && typeof slot.errorCode !== "string")) {
    throw new Error("E_SAVE_SLOT_SUMMARY_INVALID");
  }
  return slot as unknown as SaveSlotSummary;
}

export const NEW_JOURNEY_TIMEOUT_MS = 15_000;
export const CONTINUE_SLOT_TIMEOUT_MS = 15_000;
export const CONTINUE_TIMEOUT_MS = 15_000;
export const SCENE_READY_TIMEOUT_MS = 15_000;
export const LIFECYCLE_TIMEOUT_MS = 15_000;

export class TauriClient {
  private interactionRequestSequence = 0;
  private sceneRequestSequence = 0;
  private buildRequestSequence = 0;
  private lifecycleCommandSequence = 0;
  private confirmedPause: { context: SessionContext; sequence: number } | null = null;
  private readonly uncertainLifecycleContexts = new Set<string>();

  constructor(private readonly invoke: Invoke) {}

  async buildCommand(action: BuildAction, source: WorldSnapshotV3): Promise<SlotCommandReceipt> {
    assertBuildAction(action);
    assertSnapshotV3(source);
    if (!source.build || source.worldEpoch === 0) throw new Error("E_BUILD_PROJECTION_UNAVAILABLE");
    const requestId = `build-${source.worldEpoch}-${Date.now().toString(36)}-${++this.buildRequestSequence}`;
    const value = await this.invoke<unknown>("formal_build_command", { command: {
      requestId, worldEpoch: source.worldEpoch, expectedBuildRevision: source.build.revision, action,
    } });
    if (!value || typeof value !== "object") throw new Error("E_BUILD_RECEIPT_INVALID");
    const receipt = value as Record<string, unknown>;
    const snapshot = assertSnapshotV3(receipt.snapshot);
    const applied = receipt.applied === true; const duplicate = receipt.alreadyApplied === true;
    if (receipt.commandId !== `build:${requestId}` || typeof receipt.applied !== "boolean" ||
        typeof receipt.alreadyApplied !== "boolean" || (applied && duplicate) ||
        receipt.worldEpoch !== snapshot.worldEpoch || receipt.serverTick !== snapshot.serverTick ||
        receipt.authorityRevision !== snapshot.authorityRevision || !snapshot.build ||
        ((applied || duplicate) ? receipt.errorCode !== null : typeof receipt.errorCode !== "string") ||
        ((applied || duplicate) && (snapshot.worldEpoch !== source.worldEpoch || snapshot.build.revision <= source.build.revision))) {
      throw new Error("E_BUILD_RECEIPT_INVALID");
    }
    if (applied) {
      const slot = action.kind === "equip"
        ? source.build.items.find(item => item.itemId === action.itemId)?.slotId : action.slotId;
      const changed = snapshot.build.equipment.find(item => item.slotId === slot);
      if (!slot || (action.kind === "equip" ? changed?.itemId !== action.itemId : changed !== undefined)) {
        throw new Error("E_BUILD_RECEIPT_POSTCONDITION");
      }
    }
    return { ...receipt, snapshot } as SlotCommandReceipt;
  }

  async beginBaizhi(context: SessionContext, ownerId: string, generation: number): Promise<BaizhiCommandReceipt> {
    const checked = assertSessionContext(context);
    // Context may be structurally supplied by a snapshot. Rust denies unknown
    // fields, so never spread that snapshot into either request context or ticket.
    const expected: SessionContext = { worldId: checked.worldId, sceneId: checked.sceneId, worldEpoch: checked.worldEpoch };
    this.assertLifecycleCertain(expected);
    const commandSequence = ++this.lifecycleCommandSequence;
    const ticket = assertBaizhiTicket({ ...expected, ownerId, generation, entityId: "gh_bz_whitezhi_v1",
      interactionId: "gh_bz_first_contact", pauseCommandSequence: commandSequence });
    let value: unknown;
    try {
      value = await this.invoke<unknown>("formal_baizhi_begin", { request: {
        context: expected, entityId: ticket.entityId, interactionId: ticket.interactionId, ownerId, generation, commandSequence,
      } });
    } catch (error) {
      const code = typeof error === "string" ? error : error instanceof Error ? error.message : "";
      // These exact native preflight failures occur before any hold or mutation.
      // Stale/dead/loading/occupied/transport failures are deliberately not classified as running.
      if (["E_BAIZHI_OUT_OF_RANGE", "E_BAIZHI_CONFIGURATION", "E_BAIZHI_REQUEST_INVALID"].includes(code)) {
        throw new Error(`E_BAIZHI_BEGIN_NOT_STARTED:${code}`);
      }
      throw error;
    }
    const result = assertBaizhiReceipt(value, `baizhi-begin:${ownerId}:${generation}`, ticket);
    if (commandSequence === this.lifecycleCommandSequence) this.confirmedPause = { context: expected, sequence: commandSequence };
    return result;
  }

  async commitBaizhi(ticket: BaizhiDialogueTicket, requestId: string, choice: Exclude<BaizhiChoice, "unresolved">): Promise<BaizhiCommandReceipt> {
    const expected = assertBaizhiTicket(ticket);
    if (!requestId || !["taken", "left"].includes(choice)) throw new Error("E_BAIZHI_COMMIT_ARGUMENT");
    const result = assertBaizhiReceipt(await this.invoke<unknown>("formal_baizhi_commit", { ticket: expected, requestId, choice }), requestId, expected);
    if (result.receipt.snapshot.baizhi?.choice !== choice) throw new Error("E_BAIZHI_COMMIT_POSTCONDITION");
    return result;
  }

  async closeBaizhi(ticket: BaizhiDialogueTicket): Promise<BaizhiCommandReceipt> {
    const expected = assertBaizhiTicket(ticket);
    let value: unknown;
    try { value = await this.invoke<unknown>("formal_baizhi_close", { ticket: expected }); }
    catch (error) {
      const code = typeof error === "string" ? error : error instanceof Error ? error.message : "";
      // Only audited native Result Err codes prove the release did not happen.
      if (["E_RUNTIME_LOCK_POISONED", "E_BAIZHI_STALE_CONTEXT", "E_SCENE_RUNTIME_LOCK_POISONED", "E_BAIZHI_TICKET_INVALID",
        "E_BAIZHI_PAUSE_UNCONFIRMED", "E_RUNTIME_DEAD", "E_SCENE_ENTRY_NOT_READY", "E_BAIZHI_OWNER_UNAVAILABLE"].includes(code) ||
          code.startsWith("E_WORLD_REVISION:")) throw new Error(`E_BAIZHI_CLOSE_REJECTED:${code}`);
      throw error;
    }
    const result = assertBaizhiReceipt(value, `baizhi-close:${expected.ownerId}:${expected.generation}`, expected, true);
    if (result.resumed && this.confirmedPause?.sequence === expected.pauseCommandSequence) this.confirmedPause = null;
    return result;
  }

  async nativeBuildIdentity(): Promise<unknown> {
    return this.invoke<unknown>("native_build_identity");
  }

  async newJourney(timeoutMs = NEW_JOURNEY_TIMEOUT_MS): Promise<WorldSnapshotV3> {
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new Error("E_NEW_JOURNEY_TIMEOUT_INVALID");
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      const invokePromise = this.invoke<{ snapshot: WorldSnapshotV3 }>("formal_new");
      const timeoutPromise = new Promise<never>((_, reject) => {
        timeout = setTimeout(() => reject(new Error("E_NEW_JOURNEY_TIMEOUT")), timeoutMs);
      });
      const receipt = await Promise.race([invokePromise, timeoutPromise]);
      return assertSnapshotV3(receipt.snapshot);
    } finally {
      if (timeout !== undefined) clearTimeout(timeout);
    }
  }

  async hasSave(defaultOnly?: boolean): Promise<boolean> {
    if (defaultOnly !== undefined && typeof defaultOnly !== "boolean") throw new Error("E_SAVE_PROBE_MODE_INVALID");
    const result = await this.invoke<unknown>("formal_has_save", defaultOnly === undefined ? undefined : { defaultOnly });
    if (typeof result !== "boolean") throw new Error("E_SAVE_PROBE_RESULT_INVALID");
    return result;
  }

  async listSaveSlots(): Promise<SaveSlotSummary[]> {
    const slots = await this.invoke<unknown>("formal_list_save_slots");
    if (!Array.isArray(slots)) throw new Error("E_SAVE_SLOT_LIST_INVALID");
    return slots.map(assertSlotSummary);
  }

  async saveSlot(slotId: string, displayName: string, create: boolean): Promise<SlotCommandReceipt> {
    return assertSlotReceipt(await this.invoke<unknown>("formal_save_slot", { slotId, displayName, create }), "formal_save_slot");
  }

  async continueSlot(slotId: string, timeoutMs = CONTINUE_SLOT_TIMEOUT_MS): Promise<SlotCommandReceipt> {
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new Error("E_CONTINUE_SLOT_TIMEOUT_INVALID");
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      const invokePromise = this.invoke<unknown>("formal_continue_slot", { slotId });
      const timeoutPromise = new Promise<never>((_, reject) => {
        timeout = setTimeout(() => reject(new Error("E_CONTINUE_SLOT_TIMEOUT")), timeoutMs);
      });
      const receipt = await Promise.race([invokePromise, timeoutPromise]);
      return assertSlotReceipt(receipt, "formal_continue_slot");
    } finally {
      if (timeout !== undefined) clearTimeout(timeout);
    }
  }

  confirmedPauseSequence(context: SessionContext): number {
    const confirmed = this.confirmedPause;
    this.assertLifecycleCertain(context);
    if (!confirmed || confirmed.sequence !== this.lifecycleCommandSequence ||
        this.lifecycleContextKey(confirmed.context) !== this.lifecycleContextKey(context)) throw new Error("E_ENHANCEMENT_PAUSE_UNCONFIRMED");
    return confirmed.sequence;
  }

  async chooseFirstEnhancement(capabilityId: string, context: EnhancementTerminalContext): Promise<FirstEnhancementReceipt> {
    if (!capabilityId || !context || context.id !== "rs_capability_terminal_marker" || !context.requestId ||
        !Number.isSafeInteger(context.worldEpoch) || !Number.isSafeInteger(context.pauseCommandSequence) ||
        context.pauseCommandSequence <= 0) throw new Error("E_ENHANCEMENT_COMMAND_ARGUMENT");
    const value = await this.invoke<unknown>("formal_choose_first_enhancement", { capabilityId, context: { ...context } });
    if (!value || typeof value !== "object") throw new Error("E_ENHANCEMENT_RECEIPT_INVALID");
    const receipt = value as Record<string, unknown>;
    const snapshot = assertSnapshotV3(receipt.snapshot);
    if (receipt.commandId !== `enhancement:${capabilityId}` || typeof receipt.applied !== "boolean" ||
        typeof receipt.alreadyApplied !== "boolean" ||
        (receipt.errorCode !== null && typeof receipt.errorCode !== "string") ||
        receipt.worldEpoch !== snapshot.worldEpoch || receipt.serverTick !== snapshot.serverTick ||
        receipt.authorityRevision !== snapshot.authorityRevision) {
      throw new Error(receipt.commandId !== `enhancement:${capabilityId}`
        ? "E_ENHANCEMENT_RECEIPT_COMMAND_MISMATCH"
        : "E_ENHANCEMENT_RECEIPT_SNAPSHOT_MISMATCH");
    }
    if (!receipt.applied && !receipt.alreadyApplied) {
      throw new Error(`E_ENHANCEMENT_COMMAND_REJECTED:${String(receipt.errorCode || capabilityId)}`);
    }
    const capabilities = snapshot.capabilities;
    const granted = Array.isArray(capabilities.items) && capabilities.items.some(item =>
      !!item && typeof item === "object" &&
      item.capabilityId === capabilityId &&
      item.granted === true);
    if (!granted || capabilities.firstEnhancementChoice !== capabilityId) throw new Error("E_ENHANCEMENT_RECEIPT_NOT_GRANTED");
    return { ...receipt, snapshot } as FirstEnhancementReceipt;
  }

  async sceneTransition(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_scene_transition", id, worldEpoch);
  }

  async sceneCheckpoint(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_scene_checkpoint", id, worldEpoch);
  }

  async environmentControl(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_environment_control", id, worldEpoch);
  }

  async sceneTrigger(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_scene_trigger", id, worldEpoch);
  }

  async worldGate(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_world_gate", id, worldEpoch);
  }

  async missionTerminalStatus(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_mission_terminal_status", id, worldEpoch);
  }

  async capabilityTerminalStatus(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_capability_terminal_status", id, worldEpoch);
  }

  async saveRestTerminal(id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    return this.sceneCommand("formal_save_rest_terminal", id, worldEpoch);
  }

  async continueJourney(timeoutMs = CONTINUE_TIMEOUT_MS): Promise<WorldSnapshotV3> {
    const receipt = await this.invokeWithTimeout<{ snapshot: WorldSnapshotV3 }>(
      "formal_continue", undefined, timeoutMs, "E_CONTINUE_TIMEOUT");
    return assertSnapshotV3(receipt.snapshot);
  }

  async returnToHub(): Promise<WorldSnapshotV3> {
    const receipt = await this.invoke<{ snapshot: WorldSnapshotV3 }>("formal_return");
    return assertSnapshotV3(receipt.snapshot);
  }

  async snapshot(): Promise<WorldSnapshotV3> {
    return assertSnapshotV3(await this.invoke<WorldSnapshotV3>("formal_snapshot"));
  }

  /** Only a renderer-committed exact entry may clear Rust's loading pause. */
  async sceneReady(token: SceneEntryToken, remainPaused: boolean, timeoutMs = SCENE_READY_TIMEOUT_MS): Promise<WorldSnapshotV3> {
    const expectedToken = { ...assertSceneEntryToken(token) };
    const context = { worldId: expectedToken.worldId, sceneId: expectedToken.sceneId, worldEpoch: expectedToken.worldEpoch };
    if (!remainPaused) this.assertLifecycleCertain(context);
    if (typeof remainPaused !== "boolean") throw new Error("E_SCENE_READY_ARGUMENT");
    const value = await this.invokeWithTimeout<unknown>("formal_scene_ready", { token: expectedToken, remainPaused }, timeoutMs, "E_SCENE_READY_TIMEOUT",
      () => this.markLifecycleUncertain(context),
      () => { if (!remainPaused) void this.pause(context, timeoutMs).catch(() => undefined); });
    if (!value || typeof value !== "object") throw new Error("E_SCENE_READY_RECEIPT_INVALID");
    const receipt = value as Record<string, unknown>;
    const snapshot = assertSnapshotV3(receipt.snapshot);
    if (receipt.commandId !== `scene-ready:${expectedToken.generation}` ||
        typeof receipt.applied !== "boolean" || typeof receipt.alreadyApplied !== "boolean" ||
        (receipt.applied && receipt.alreadyApplied) ||
        receipt.worldEpoch !== snapshot.worldEpoch || receipt.serverTick !== snapshot.serverTick ||
        receipt.authorityRevision !== snapshot.authorityRevision ||
        (receipt.errorCode !== null && typeof receipt.errorCode !== "string")) {
      throw new Error("E_SCENE_READY_RECEIPT_INVALID");
    }
    if (!receipt.applied && !receipt.alreadyApplied) {
      throw new Error(`E_SCENE_READY_REJECTED:${receipt.errorCode || "unknown"}`);
    }
    if (receipt.errorCode !== null || snapshot.entryToken !== undefined ||
        snapshot.worldId !== expectedToken.worldId || snapshot.sceneId !== expectedToken.sceneId || snapshot.worldEpoch !== expectedToken.worldEpoch) {
      throw new Error("E_SCENE_READY_RECEIPT_IDENTITY");
    }
    return snapshot;
  }

  async pause(context?: SessionContext, timeoutMs = LIFECYCLE_TIMEOUT_MS): Promise<WorldSnapshotV3> {
    return this.lifecycleCommand("formal_pause", context, timeoutMs);
  }

  async resume(context?: SessionContext, timeoutMs = LIFECYCLE_TIMEOUT_MS): Promise<WorldSnapshotV3> {
    return this.lifecycleCommand("formal_resume", context, timeoutMs);
  }

  async submitInput(sample: InputState): Promise<WorldSnapshotV3> {
    const receipt = await this.invoke<{ snapshot: WorldSnapshotV3 }>("formal_submit_input", { sample, combat: [] });
    return assertSnapshotV3(receipt.snapshot);
  }

  async events(worldEpoch: number, afterEventId: number): Promise<PresentationEvent[]> {
    return this.invoke<PresentationEvent[]>("formal_presentation_events", { worldEpoch, afterEventId });
  }

  async soundCues(worldEpoch: number, afterEventId: number): Promise<SoundCueEvent[]> {
    return this.invoke<SoundCueEvent[]>("formal_sound_cues", { worldEpoch, afterEventId });
  }

  async submitAction(action: ActionCommand): Promise<WorldSnapshotV3> {
    const receipt = await this.invoke<{ snapshot: WorldSnapshotV3 }>("formal_submit_action", { action });
    return assertSnapshotV3(receipt.snapshot);
  }

  async interact(interactionId: string, worldEpoch?: number): Promise<{ applied: boolean; alreadyApplied?: boolean; errorCode: string | null; snapshot: WorldSnapshotV3 }> {
    if (worldEpoch !== undefined && (!interactionId || !Number.isSafeInteger(worldEpoch) || worldEpoch < 1)) {
      throw new Error("E_INTERACTION_ARGUMENT");
    }
    this.interactionRequestSequence++;
    const requestId = `web-interaction-${Date.now()}-${this.interactionRequestSequence}`;
    const value = await this.invoke<unknown>("formal_interact", {
      actorId: interactionId, requestId, ...(worldEpoch === undefined ? {} : { worldEpoch }),
    });
    // The unbound legacy bridge is retained for callers outside the native scene
    // dispatcher. Strict CW refuses it in Rust. Native F always supplies epoch.
    if (worldEpoch === undefined) {
      const result = value as { applied: boolean; errorCode: string | null };
      return { ...result, snapshot: await this.snapshot() };
    }
    if (!value || typeof value !== "object" || !("receipt" in value) ||
        !value.receipt || typeof value.receipt !== "object") throw new Error("E_INTERACTION_RECEIPT_INVALID");
    const outcome = value as Record<string, unknown>;
    const receipt = value.receipt as Record<string, unknown>;
    const snapshot = assertSnapshotV3(receipt.snapshot);
    if (receipt.commandId !== requestId) throw new Error("E_INTERACTION_RECEIPT_REQUEST_MISMATCH");
    if (typeof receipt.applied !== "boolean" || typeof receipt.alreadyApplied !== "boolean" ||
        (receipt.errorCode !== null && typeof receipt.errorCode !== "string") ||
        receipt.applied !== outcome.applied || receipt.alreadyApplied !== outcome.alreadyApplied ||
        receipt.errorCode !== outcome.errorCode || receipt.worldEpoch !== snapshot.worldEpoch ||
        receipt.serverTick !== snapshot.serverTick || receipt.authorityRevision !== snapshot.authorityRevision) {
      throw new Error("E_INTERACTION_RECEIPT_SNAPSHOT_MISMATCH");
    }
    if (snapshot.worldEpoch !== worldEpoch) throw new Error("E_INTERACTION_RECEIPT_STALE_EPOCH");
    return { applied: receipt.applied, alreadyApplied: receipt.alreadyApplied,
      errorCode: receipt.errorCode as string | null, snapshot };
  }

  private async sceneCommand(command: string, id: string, worldEpoch: number): Promise<SceneCommandReceipt> {
    if (!id || !Number.isSafeInteger(worldEpoch) || worldEpoch < 0) throw new Error("E_SCENE_COMMAND_ARGUMENT");
    this.sceneRequestSequence++;
    const requestId = `web-scene-${Date.now()}-${this.sceneRequestSequence}`;
    const value = await this.invoke<unknown>(command, { id, requestId, worldEpoch });
    if (!value || typeof value !== "object") throw new Error("E_SCENE_RECEIPT_INVALID");
    const receipt = value as Record<string, unknown>;
    const snapshot = assertSnapshotV3(receipt.snapshot);
    if (receipt.commandId !== requestId || typeof receipt.applied !== "boolean" ||
        typeof receipt.alreadyApplied !== "boolean" ||
        (receipt.errorCode !== null && typeof receipt.errorCode !== "string") ||
        receipt.worldEpoch !== snapshot.worldEpoch || receipt.serverTick !== snapshot.serverTick ||
        receipt.authorityRevision !== snapshot.authorityRevision) {
      throw new Error(receipt.commandId !== requestId
        ? "E_SCENE_RECEIPT_REQUEST_MISMATCH"
        : "E_SCENE_RECEIPT_SNAPSHOT_MISMATCH");
    }
    return { ...receipt, snapshot, requestId } as SceneCommandReceipt;
  }

  private async invokeWithTimeout<T>(command: string, args: Record<string, unknown> | undefined,
    timeoutMs: number, timeoutCode: string, onTimeout?: () => void, onLate?: () => void): Promise<T> {
    if (!Number.isSafeInteger(timeoutMs) || timeoutMs <= 0) throw new Error(`${timeoutCode}_INVALID`);
    let timeout: ReturnType<typeof setTimeout> | undefined;
    let expired = false;
    try {
      const pending = this.invoke<T>(command, args);
      // Timeout cannot cancel a native dispatch. Observe late settlement without
      // accepting its receipt, and neutralize only the original context.
      void pending.then(() => { if (expired) onLate?.(); }, () => { if (expired) onLate?.(); });
      const deadline = new Promise<never>((_, reject) => {
        timeout = setTimeout(() => {
          expired = true;
          onTimeout?.();
          reject(new Error(timeoutCode));
        }, timeoutMs);
      });
      return await Promise.race([pending, deadline]);
    } finally {
      if (timeout !== undefined) clearTimeout(timeout);
    }
  }

  private lifecycleContextKey(context: SessionContext): string {
    return JSON.stringify([context.worldId, context.sceneId, context.worldEpoch]);
  }

  private markLifecycleUncertain(context: SessionContext | undefined): void {
    if (context) this.uncertainLifecycleContexts.add(this.lifecycleContextKey(context));
  }

  private assertLifecycleCertain(context: SessionContext): void {
    if (this.uncertainLifecycleContexts.has(this.lifecycleContextKey(context))) {
      throw new Error("E_LIFECYCLE_UNCERTAIN");
    }
  }

  private async lifecycleCommand(command: "formal_pause" | "formal_resume", context: SessionContext | undefined, timeoutMs: number): Promise<WorldSnapshotV3> {
    // Copy before invoking: caller mutation must not rebind an in-flight command.
    const expected = context === undefined ? undefined : { ...assertSessionContext(context) };
    if (expected && command === "formal_resume") this.assertLifecycleCertain(expected);
    const commandSequence = expected ? ++this.lifecycleCommandSequence : undefined;
    if (commandSequence !== undefined && !Number.isSafeInteger(commandSequence)) throw new Error("E_LIFECYCLE_SEQUENCE_EXHAUSTED");
    const receipt = await this.invokeWithTimeout<{
      commandId: string;
      applied: boolean;
      alreadyApplied: boolean;
      errorCode: string | null;
      worldEpoch: number;
      serverTick: number;
      authorityRevision: number;
      snapshot: unknown;
    }>(command, expected === undefined ? undefined : { context: expected, commandSequence }, timeoutMs, "E_LIFECYCLE_TIMEOUT",
      () => this.markLifecycleUncertain(expected),
      () => { if (expected && command === "formal_resume") void this.pause(expected, timeoutMs).catch(() => undefined); });
    if (expected && (receipt.commandId !== (command === "formal_pause" ? "pause" : "resume") ||
        typeof receipt.applied !== "boolean" || typeof receipt.alreadyApplied !== "boolean" ||
        (receipt.applied && receipt.alreadyApplied) ||
        ((receipt.applied || receipt.alreadyApplied) && receipt.errorCode !== null))) {
      throw new Error("E_LIFECYCLE_RECEIPT_INVALID");
    }
    const snapshot = assertSnapshotV3(receipt.snapshot);
    if (receipt.worldEpoch !== snapshot.worldEpoch || receipt.serverTick !== snapshot.serverTick ||
        receipt.authorityRevision !== snapshot.authorityRevision) {
      throw new Error("E_LIFECYCLE_RECEIPT_SNAPSHOT_MISMATCH");
    }
    if (!receipt.applied && !receipt.alreadyApplied) {
      throw new Error(`E_LIFECYCLE_COMMAND_REJECTED:${receipt.errorCode || command}`);
    }
    if (expected && (snapshot.worldId !== expected.worldId || snapshot.sceneId !== expected.sceneId ||
        snapshot.worldEpoch !== expected.worldEpoch)) throw new Error("E_LIFECYCLE_RECEIPT_CONTEXT_MISMATCH");
    if (expected && commandSequence === this.lifecycleCommandSequence) {
      this.confirmedPause = command === "formal_pause" ? { context: expected, sequence: commandSequence } : null;
    }
    return snapshot;
  }
}
