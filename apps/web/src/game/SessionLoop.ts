import type { TauriClient } from "../bridge/tauri-client.js";
import type { ActionCommand, InputState, PresentationEvent, SoundCueEvent, SceneEntryToken, SessionContext, Vec3, WorldSnapshotEnvelope } from "../protocol/types.js";
import { InputController } from "./InputController.js";
import { isPlayerDead, SnapshotClient } from "./SnapshotClient.js";
import type { PointerAimOffset, PointerSurfaceRect } from "../renderer/PlayerAimFrame.js";

const STEP_MS = 1000 / 60;
export const SESSION_STOP_TIMEOUT_MS = 15_000;

export interface SessionRenderer {
  render(snapshot: WorldSnapshotEnvelope, playerPosition?: Vec3, actorPositions?: ReadonlyMap<string, Vec3>): Promise<void>;
  /** Aim only from the last successfully displayed player pose, never a pending snapshot. */
  pointerAim?(snapshot: WorldSnapshotEnvelope, clientX: number, clientY: number, rect: PointerSurfaceRect): PointerAimOffset | null;
  /** True only while the exact scene has a committed, usable first frame. */
  isReadyFor?(snapshot: WorldSnapshotEnvelope): boolean;
  cancel?(): void;
  destroy?(): Promise<void>;
}

export interface SessionScheduler {
  now(): number;
  clientTimeMs(): number;
  setInterval(callback: () => void, delayMs: number): unknown;
  clearInterval(handle: unknown): void;
  requestAnimationFrame(callback: (timestamp: number) => void): number;
  cancelAnimationFrame(handle: number): void;
}

export interface SessionEventSurface {
  addEventListener(type: string, listener: EventListener): void;
  removeEventListener(type: string, listener: EventListener): void;
}

export interface SessionPointerSurface extends SessionEventSurface {
  getBoundingClientRect(): Pick<DOMRect, "left" | "top" | "width" | "height">;
}
export interface SessionVisibilitySurface extends SessionEventSurface { readonly hidden?: boolean; hasFocus?(): boolean }

export interface SessionLoopOptions {
  stopTimeoutMs?: number;
  presentationTimeoutMs?: number;
  input?: InputController;
  snapshots?: SnapshotClient;
  scheduler?: Partial<SessionScheduler>;
  windowTarget?: SessionEventSurface;
  documentTarget?: SessionVisibilitySurface;
  pointerTarget?: SessionPointerSurface;
  onEvents?: (events: PresentationEvent[]) => void;
  onEventsWithSnapshot?: (events: PresentationEvent[], snapshot: WorldSnapshotEnvelope) => PresentationEvent[] | void;
  onClearTransientPresentation?: () => void;
  onSoundCues?: (events: SoundCueEvent[], snapshot: WorldSnapshotEnvelope) => void;
  onSnapshot?: (snapshot: WorldSnapshotEnvelope) => void;
  onDeath?: (snapshot: WorldSnapshotEnvelope) => void;
  onInteract?: (snapshot: WorldSnapshotEnvelope) => void | Promise<void>;
  onPauseState?: (status: PauseStatus, message?: string) => void;
  onError?: (error: unknown) => void;
  onCleanup?: () => void;
}

type StopReason = "hub" | "error" | "unload" | "replace";
export type StopAuthorityState = "pending" | "confirmed" | "failed" | "dead";
export type PauseStatus = "dead" | "loading" | "running" | "pausing" | "paused" | "resuming" | "error";

function defaultScheduler(): SessionScheduler {
  return {
    now: () => performance.now(),
    clientTimeMs: () => Date.now(),
    setInterval: (callback, delayMs) => window.setInterval(callback, delayMs),
    clearInterval: handle => window.clearInterval(handle as number),
    requestAnimationFrame: callback => window.requestAnimationFrame(callback),
    cancelAnimationFrame: handle => window.cancelAnimationFrame(handle),
  };
}

function actorViews(snapshot: WorldSnapshotEnvelope) {
  return snapshot.protocolVersion === 3 ? snapshot.actors : snapshot.view.actors;
}

/** Serializes authoritative Tauri commands, drops overdue input samples, and renders only presentation state. */
export class SessionLoop {
  readonly input: InputController;
  readonly snapshots: SnapshotClient;
  private readonly scheduler: SessionScheduler;
  private readonly stopTimeoutMs: number;
  private readonly presentationTimeoutMs: number;
  private readonly windowTarget: SessionEventSurface | undefined;
  private readonly documentTarget: SessionVisibilitySurface | undefined;
  private readonly pointerTarget: SessionPointerSurface | undefined;
  private readonly onEvents: ((events: PresentationEvent[]) => void) | undefined;
  private readonly onEventsWithSnapshot: ((events: PresentationEvent[], snapshot: WorldSnapshotEnvelope) => void) | undefined;
  private readonly onClearTransientPresentation: (() => void) | undefined;
  private readonly onSoundCues: ((events: SoundCueEvent[], snapshot: WorldSnapshotEnvelope) => void) | undefined;
  private readonly onSnapshot: ((snapshot: WorldSnapshotEnvelope) => void) | undefined;
  private readonly onInteract: ((snapshot: WorldSnapshotEnvelope) => void | Promise<void>) | undefined;
  private readonly onPauseState: ((status: PauseStatus, message?: string) => void) | undefined;
  private readonly onError: ((error: unknown) => void) | undefined;
  private readonly onCleanup: (() => void) | undefined;
  private dead = false;
  private readonly onDeath: ((snapshot: WorldSnapshotEnvelope) => void) | undefined;
  private active = false;
  private entryLoading = false;
  private rendererEntryVerified = false;
  private entrySequence = 0;
  private entryToken: SceneEntryToken | undefined;
  private entryTask: Promise<void> | null = null;
  private paused = false;
  private desiredPaused = false;
  private authorityPauseState: "running" | "paused" | "unknown" = "running";
  private resumeReceiptRequired = false;
  private pauseStatus: PauseStatus = "running";
  private pauseMessage = "";
  private manualPauseRequested = false;
  private lifecyclePauseRequested = false;
  private lifecycleTask: Promise<void> | null = null;
  private lifecycleSequence = 0;
  private lifecycleResuming = false;
  private controlSequence = 0;
  private readonly externalCommands = new Set<Promise<unknown>>();
  private readonly commandIdleWaiters: Array<() => void> = [];
  private stopping = false;
  private commandBusy = false;
  private renderBusy = false;
  private renderInFlight: Promise<void> | null = null;
  private intervalHandle: unknown;
  private animationFrameHandle: number | null = null;
  private inFlight: Promise<void> | null = null;
  private stopPromise: Promise<void> | null = null;
  private stopAuthorityStatus: StopAuthorityState | null = null;
  private stopAuthorityFailure: unknown;
  private stopPauseSequence = 0;
  private stopMutationUncertain = false;
  private readonly actions: Array<{ action: ActionCommand; aim?: Pick<InputState, "aimX" | "aimZ"> }> = [];
  private lastSnapshotAt = 0;
  private lastClientTimeMs = 0;
  private pointerClientPosition: { x: number; y: number } | null = null;

  private readonly keyDownListener: EventListener = event => {
    const keyboard = event as KeyboardEvent;
    // Modal controls get first refusal (for example Escape closing a terminal).
    // Never consume text entry or native keyboard activation of a UI control.
    const target = keyboard.target as HTMLElement | null;
    if (keyboard.defaultPrevented || keyboard.isComposing || keyboard.ctrlKey || keyboard.altKey || keyboard.metaKey) return;
    const key = keyboard.key.toLowerCase();
    if (key === "escape") {
      if (!this.active || this.stopping) return;
      keyboard.preventDefault();
      if (!keyboard.repeat) this.keyDown(keyboard.key);
      return;
    }
    if (!this.active || this.paused || this.stopping || target?.isContentEditable ||
      target?.closest?.("input, textarea, select, button, [contenteditable='true']")) return;
    if (["w", "a", "s", "d", "j", "shift", "q", "e", "r", "f", " "].includes(key)) keyboard.preventDefault();
    // After a pause clears controls, an OS repeat is still not a new press.
    if (keyboard.repeat) return;
    this.keyDown(keyboard.key);
  };
  private readonly keyUpListener: EventListener = event => this.keyUp((event as KeyboardEvent).key);
  private readonly pointerMoveListener: EventListener = event => {
    const pointer = event as PointerEvent;
    if (!Number.isFinite(pointer.clientX) || !Number.isFinite(pointer.clientY)) return;
    this.pointerClientPosition = { x: pointer.clientX, y: pointer.clientY };
    this.refreshPointerAim();
  };
  private readonly pointerDownListener: EventListener = event => {
    const pointer = event as PointerEvent;
    const target = pointer.target as HTMLElement | null;
    if (pointer.defaultPrevented || pointer.button !== 0 || pointer.isPrimary === false ||
        pointer.ctrlKey || pointer.altKey || pointer.metaKey ||
        !Number.isFinite(pointer.clientX) || !Number.isFinite(pointer.clientY) ||
        target?.isContentEditable || target?.closest?.("input, textarea, select, button, [contenteditable='true']") ||
        !this.active || this.paused || this.entryLoading || this.stopping || this.dead) return;
    const snapshot = this.snapshots.view();
    if (!snapshot) return;
    // A click may be the first pointer event, and must keep its own aim if IPC
    // is busy. Rust receives that sample before the corresponding attack edge.
    this.pointerClientPosition = { x: pointer.clientX, y: pointer.clientY };
    if (!this.refreshPointerAim()) return;
    pointer.preventDefault();
    const time = this.clientTimeMs();
    this.enqueueAction(this.input.primaryAttack(snapshot.worldEpoch, time), this.input.aim());
  };
  private readonly blurListener: EventListener = () => { void this.pause("lifecycle"); };
  private readonly focusListener: EventListener = () => {
    if (this.lifecyclePauseRequested && !this.manualPauseRequested) void this.resume("lifecycle");
  };
  private readonly visibilityListener: EventListener = () => {
    const hidden = typeof this.documentTarget?.hidden === "boolean"
      ? this.documentTarget.hidden
      : typeof document !== "undefined" && document.hidden;
    if (hidden) void this.pause("lifecycle");
    else if (this.lifecyclePauseRequested && !this.manualPauseRequested) void this.resume("lifecycle");
  };
  private readonly pageHideListener: EventListener = () => { void this.stop("unload").catch(() => undefined); };

  constructor(
    private readonly client: Pick<TauriClient, "submitInput" | "submitAction" | "events"> & Partial<Pick<TauriClient, "pause" | "resume" | "soundCues" | "sceneReady" | "snapshot">>,
    private readonly renderer: SessionRenderer,
    options: SessionLoopOptions = {},
  ) {
    this.stopTimeoutMs = options.stopTimeoutMs ?? SESSION_STOP_TIMEOUT_MS;
    this.presentationTimeoutMs = options.presentationTimeoutMs ?? 5_000;
    if (!Number.isSafeInteger(this.presentationTimeoutMs) || this.presentationTimeoutMs <= 0) throw new Error("E_PRESENTATION_TIMEOUT_INVALID");
    if (!Number.isSafeInteger(this.stopTimeoutMs) || this.stopTimeoutMs <= 0) throw new Error("E_SESSION_STOP_TIMEOUT_INVALID");
    const defaults = defaultScheduler();
    this.scheduler = { ...defaults, ...options.scheduler };
    this.input = options.input ?? new InputController();
    this.snapshots = options.snapshots ?? new SnapshotClient();
    this.windowTarget = options.windowTarget ?? (typeof window === "undefined" ? undefined : window);
    this.documentTarget = options.documentTarget ?? (typeof document === "undefined" ? undefined : document);
    this.pointerTarget = options.pointerTarget;
    this.onEvents = options.onEvents;
    this.onEventsWithSnapshot = options.onEventsWithSnapshot;
    this.onClearTransientPresentation = options.onClearTransientPresentation;
    this.onSoundCues = options.onSoundCues;
    this.onSnapshot = options.onSnapshot;
    this.onDeath = options.onDeath;
    this.onInteract = options.onInteract;
    this.onPauseState = options.onPauseState;
    this.onError = options.onError;
    this.onCleanup = options.onCleanup;
  }

  /** Current requested target, so the HUD can reverse an in-flight transition. */
  get hasUnsettledStopMutation(): boolean { return this.stopMutationUncertain; }
  get stopAuthorityState(): StopAuthorityState | null { return this.stopAuthorityStatus; }
  get stopAuthorityError(): unknown { return this.stopAuthorityFailure; }
  get isDead(): boolean { return this.dead; }
  get isPaused(): boolean { return this.desiredPaused; }
  get pausePresentationState(): PauseStatus { return this.pauseStatus; }
  get acceptsExternalResults(): boolean { return this.active && !this.stopping && !this.dead; }
  /** Read-only fence for presentation owned by an asynchronous interaction. */
  get sceneEntrySequence(): number { return this.entrySequence; }

  async start(initialSnapshot: WorldSnapshotEnvelope): Promise<void> {
    if (this.active || this.stopping) throw new Error("E_SESSION_LOOP_ALREADY_STARTED");
    this.active = true;
    this.applySnapshot(initialSnapshot);
    this.refreshPauseIntent();
    try {
      this.bindEvents();
      if (this.dead) { await this.renderSnapshot(1); return; }
      if (initialSnapshot.protocolVersion === 3 && initialSnapshot.entryToken) {
        await this.beginEntry(initialSnapshot);
      } else {
        // Compatibility for older snapshots. Formal entries always carry a token.
        this.entryLoading = true;
        await this.renderSnapshot(1);
        await this.pollEvents(initialSnapshot);
        this.entryLoading = false;
        if (this.stopping) return;
        this.refreshPauseIntent();
        if (this.desiredPaused) await this.ensurePauseLifecycle();
        else { this.startSchedulers(); this.publishPauseState("running"); }
      }
    } catch (error) {
      await this.stop("error");
      throw error;
    }
  }

  private refreshPauseIntent(): void {
    if (this.documentTarget?.hidden === true || this.documentTarget?.hasFocus?.() === false) this.lifecyclePauseRequested = true;
    this.desiredPaused = this.dead || this.manualPauseRequested || this.lifecyclePauseRequested;
  }

  private entryIsCurrent(sequence: number): boolean {
    return this.active && !this.stopping && !this.dead && sequence === this.entrySequence;
  }

  private assertEntryFrame(snapshot: WorldSnapshotEnvelope): void {
    if (!this.renderer.isReadyFor?.(snapshot)) throw new Error("E_SCENE_ENTRY_FRAME_NOT_READY");
  }

  /** The entry transaction owns its own IPC lane. It must not wait for the
   * external interaction whose receipt introduced this very token. */
  private beginEntry(snapshot: WorldSnapshotEnvelope): Promise<void> {
    const context = this.sessionContext(snapshot);
    const token = snapshot.protocolVersion === 3 ? snapshot.entryToken : undefined;
    if (!token) throw new Error("E_SCENE_ENTRY_TOKEN_REQUIRED");
    if (this.entryLoading && this.entryToken?.generation === token.generation) return this.entryTask ?? Promise.resolve();
    const sequence = ++this.entrySequence;
    this.entryToken = token;
    // An old lifecycle IPC may never settle. The new entry owns a new lane.
    this.lifecycleTask = null;
    this.lifecycleSequence++;
    this.lifecycleResuming = false;
    this.rendererEntryVerified = true;
    this.entryLoading = true;
    this.paused = true;
    this.authorityPauseState = "paused";
    this.resumeReceiptRequired = false;
    this.stopSchedulers();
    // Rust resets controls at entry. Never transplant a queued GuardEnd into the new scene.
    this.input.releaseAll(snapshot.worldEpoch, this.clientTimeMs());
    this.actions.length = 0;
    this.controlSequence++;
    this.onClearTransientPresentation?.();
    this.publishPauseState("loading");
    let readyDispatched = false;
    let requiredDrawnSnapshot = snapshot;
    const operation = Promise.resolve().then(async () => {
      // A previous frame may be unwinding after scene invalidation.
      await this.renderInFlight?.catch(() => undefined);
      if (!this.entryIsCurrent(sequence)) return;
      await this.renderSnapshot(1);
      if (!this.entryIsCurrent(sequence)) return;
      this.assertEntryFrame(requiredDrawnSnapshot);
      // Scene resource setup may clear transients during the base draw. Admit
      // held-phase visuals afterward, before the cue-bearing readiness draw.
      await this.pollEvents(snapshot, true, () => this.entryIsCurrent(sequence));
      if (!this.entryIsCurrent(sequence)) return;
      if (this.snapshots.deferredEventCount() !== 0) throw new Error("E_HELD_PRESENTATION_EVENTS_AHEAD");
      this.assertEntryFrame(requiredDrawnSnapshot);
      this.refreshPauseIntent();
      // A frame prepared while hidden/unfocused is not the player's first
      // visible frame. Refresh immediately before an entry may release Rust.
      if (!this.desiredPaused) {
        await this.renderSnapshot(1);
        if (!this.entryIsCurrent(sequence)) return;
        this.assertEntryFrame(requiredDrawnSnapshot);
        this.refreshPauseIntent();
      }
      const remainPaused = this.desiredPaused;
      if (!this.client.sceneReady) throw new Error("E_SCENE_READY_UNAVAILABLE");
      readyDispatched = true;
      const receipt = await this.client.sceneReady(token, remainPaused);
      // A stop cannot leave an already-dispatched ready(false) running behind the Hub.
      if (!this.entryIsCurrent(sequence)) {
        if (this.stopping && !remainPaused) await this.pauseStoppedContext(context).catch(() => undefined);
        return;
      }
      this.assertEntryFrame(requiredDrawnSnapshot);
      // A Ready receipt may not replace the held support pose behind the draw
      // proof. Compensate/pause on any changed geometry before publishing input.
      this.assertEntryFrame(receipt);
      this.applySnapshot(receipt);
      if (this.dead) return;
      this.authorityPauseState = remainPaused ? "paused" : "running";
      this.resumeReceiptRequired = false;
      // Blur/manual pause may arrive while ready IPC is in flight. Keep all input
      // locked and reconcile the current intent before publishing a running state.
      while (this.entryIsCurrent(sequence)) {
        this.assertEntryFrame(requiredDrawnSnapshot);
        this.refreshPauseIntent();
        const targetPaused = this.desiredPaused;
        if (this.authorityPauseState === (targetPaused ? "paused" : "running")) break;
        const command = targetPaused ? this.client.pause : this.client.resume;
        if (!command) throw new Error(targetPaused ? "E_FORMAL_PAUSE_UNAVAILABLE" : "E_FORMAL_RESUME_UNAVAILABLE");
        if (!targetPaused) {
          const prepared = await this.preparePausedPresentation(this.snapshots.view() ?? snapshot, () => this.entryIsCurrent(sequence));
          if (!this.entryIsCurrent(sequence)) return;
          // Even a newly paused intent may have drawn a newer held support pose.
          requiredDrawnSnapshot = this.snapshots.view() ?? snapshot;
          if (!prepared) continue;
          this.assertEntryFrame(requiredDrawnSnapshot);
        }
        this.authorityPauseState = "unknown";
        const updated = await command.call(this.client, context);
        if (!this.entryIsCurrent(sequence)) {
          if (this.stopping && !targetPaused) await this.pauseStoppedContext(context).catch(() => undefined);
          return;
        }
        this.assertEntryFrame(requiredDrawnSnapshot);
        if (!targetPaused) this.assertEntryFrame(updated);
        this.applySnapshot(updated);
        if (this.dead) return;
        this.authorityPauseState = targetPaused ? "paused" : "running";
      }
      if (!this.entryIsCurrent(sequence)) return;
      this.entryLoading = false;
      this.entryToken = undefined;
      this.paused = this.desiredPaused;
      this.publishPauseState(this.paused ? "paused" : "running");
      this.startSchedulers();
    }).catch(async error => {
      if (!this.entryIsCurrent(sequence)) return;
      // Readiness lost after dispatch (or a malformed/failed receipt) is uncertain:
      // request a compensating authoritative pause and keep the entry locked.
      if (readyDispatched) {
        try {
          if (!this.client.pause) throw new Error("E_FORMAL_PAUSE_UNAVAILABLE");
          await this.client.pause(context);
          if (this.entryIsCurrent(sequence)) this.authorityPauseState = "paused";
        } catch { if (this.entryIsCurrent(sequence)) this.authorityPauseState = "unknown"; }
      }
      if (!this.entryIsCurrent(sequence)) return;
      this.paused = true;
      this.stopSchedulers();
      this.publishPauseState("error", error instanceof Error ? error.message : String(error));
      throw error;
    });
    this.entryTask = operation;
    void operation.then(() => { if (this.entryTask === operation) this.entryTask = null; },
      () => { if (this.entryTask === operation) this.entryTask = null; });
    return operation;
  }

  /** Key handlers are public so hosts and deterministic tests can drive the same path as DOM events. */
  keyDown(key: string): ActionCommand | null {
    if (!this.active || this.stopping || this.dead) return null;
    if (key.toLowerCase() === "escape") {
      if (this.pauseStatus === "error") void this.retryPauseState();
      else if (this.desiredPaused) void this.resume();
      else void this.pause();
      return null;
    }
    if (this.paused || this.entryLoading) return null;
    const snapshot = this.snapshots.view();
    if (!snapshot) return null;
    const action = this.input.keyDown(key, snapshot.worldEpoch, this.clientTimeMs());
    if (key.toLowerCase() === "f") {
      if (!action) return null;
      try {
        const result = this.onInteract?.(snapshot);
        if (result && typeof (result as Promise<void>).then === "function") {
          const operation = Promise.resolve(result);
          this.externalCommands.add(operation);
          const sequence = this.entrySequence;
          void operation.catch(error => {
            if (sequence === this.entrySequence && this.snapshotIsCurrent(snapshot)) this.fail(error);
          }).finally(() => this.externalCommands.delete(operation));
        }
      } catch (error) { this.fail(error); }
      return null;
    }
    if (action) {
      if (action.kind === "primaryAttack" || action.kind === "pierce" || action.kind === "dash") {
        // Directional keyboard edges need the same input-before-action receipt
        // as clicks. Reproject a stationary cursor against the displayed pose;
        // an unusable canvas pose must not resend a cached pointer direction.
        const pointerAimReady = this.refreshPointerAim();
        const aim = this.pointerClientPosition && this.pointerTarget && !pointerAimReady
          ? { aimX: 0, aimZ: 0 } : this.input.aim();
        this.enqueueAction(action, aim);
      } else this.enqueueAction(action);
    }
    return action;
  }

  keyUp(key: string): ActionCommand | null {
    if (!this.active || this.paused || this.entryLoading || this.stopping) return null;
    const snapshot = this.snapshots.view();
    if (!snapshot) return null;
    const action = this.input.keyUp(key, snapshot.worldEpoch, this.clientTimeMs());
    if (action) this.enqueueAction(action);
    return action;
  }

  pointer(screenXFromCenter: number, screenYFromCenter: number): void {
    if (this.active && !this.paused && !this.entryLoading && !this.stopping) this.input.pointer(screenXFromCenter, screenYFromCenter);
  }

  private refreshPointerAim(): boolean {
    if (!this.active || this.paused || this.entryLoading || this.stopping || this.dead) return false;
    const pointer = this.pointerClientPosition;
    const rect = this.pointerTarget?.getBoundingClientRect();
    const snapshot = this.snapshots.view();
    if (!pointer || !rect || !snapshot || !Number.isFinite(rect.width) || !Number.isFinite(rect.height) ||
        rect.width <= 0 || rect.height <= 0) return false;
    if (this.renderer.pointerAim) {
      const aim = this.renderer.pointerAim(snapshot, pointer.x, pointer.y, rect);
      // A cancelled/loading/resize frame cannot steer the next authority input.
      this.input.pointer(aim?.x ?? 0, aim?.y ?? 0);
      return aim !== null;
    } else {
      // Retained only for old render fixtures without a projected-pose interface.
      this.input.pointer(pointer.x - rect.left - rect.width / 2, pointer.y - rect.top - rect.height / 2);
      return true;
    }
  }

  pause(source: "manual" | "lifecycle" = "manual"): Promise<void> {
    if (!this.active || this.stopping || this.dead) return Promise.resolve();
    if (source === "manual") this.manualPauseRequested = true;
    else this.lifecyclePauseRequested = true;
    this.desiredPaused = true;
    this.resumeReceiptRequired = false;
    this.paused = true;
    this.stopSchedulers();
    this.onClearTransientPresentation?.();
    this.clearControls();
    if (this.entryLoading) return this.entryTask ?? Promise.resolve();
    this.publishPauseState(this.authorityPauseState === "paused" ? "paused" : "pausing");
    return this.ensurePauseLifecycle(true);
  }

  resume(source: "manual" | "lifecycle" = "manual"): Promise<void> {
    if (!this.active || this.stopping || this.dead) return Promise.resolve();
    if (source === "manual") {
      this.manualPauseRequested = false;
      this.lifecyclePauseRequested = false;
    } else this.lifecyclePauseRequested = false;
    this.refreshPauseIntent();
    this.resumeReceiptRequired = true;
    // Do not reopen local input or show the running view before the Rust receipt arrives.
    this.paused = true;
    this.stopSchedulers();
    // Pause owns transient clearing. Repeated resume/focus intent must not erase
    // a freshly admitted held-phase cue while its readiness draw is in flight.
    if (this.entryLoading) return this.entryTask ?? Promise.resolve();
    this.publishPauseState("resuming");
    return this.ensurePauseLifecycle();
  }

  retryPauseState(): Promise<void> {
    if (!this.active || this.stopping || this.dead) return Promise.resolve();
    if (this.entryLoading) return this.entryTask ?? Promise.resolve();
    this.publishPauseState(this.desiredPaused ? "pausing" : "resuming");
    return this.ensurePauseLifecycle();
  }

  /** Track a mode-neutral authoritative command before yielding, so Return/Save/Resume
   * cannot overtake an already-dispatched equipment mutation. No pause policy is added. */
  runWithSession<T>(operation: () => Promise<T>): Promise<T> {
    if (!this.active || this.stopping || this.dead || this.entryLoading) return Promise.reject(new Error("E_BUILD_SESSION_CHANGED"));
    const epoch = this.snapshots.view()?.worldEpoch;
    const previous = [...this.externalCommands];
    const pending = (async () => {
      await Promise.allSettled(previous);
      await this.waitForCommandIdle();
      if (!this.active || this.stopping || this.dead || this.entryLoading || this.snapshots.view()?.worldEpoch !== epoch) throw new Error("E_BUILD_SESSION_CHANGED");
      const result = await operation();
      if (!this.active || this.stopping || this.dead || this.entryLoading || this.snapshots.view()?.worldEpoch !== epoch) throw new Error("E_BUILD_SESSION_CHANGED");
      return result;
    })();
    this.externalCommands.add(pending);
    void pending.then(() => { this.externalCommands.delete(pending); this.resolveCommandIdleWaiters(); },
      () => { this.externalCommands.delete(pending); this.resolveCommandIdleWaiters(); });
    return pending;
  }

  /** Runs a save or other authoritative command only after Rust confirms pause.
   * The operation participates in the same lifecycle/stop queue as interactions,
   * so a resume or Hub transition cannot apply its receipt over a newer session.
   */
  runWhilePaused<T>(operation: () => Promise<T>): Promise<T> {
    if (!this.active || this.stopping || this.dead || !this.paused || this.pauseStatus !== "paused") {
      return Promise.reject(new Error("E_SAVE_REQUIRES_CONFIRMED_PAUSE"));
    }
    const worldEpoch = this.snapshots.view()?.worldEpoch;
    // Register synchronously before yielding. A same-turn resume or stop then
    // sees and waits for this save instead of overtaking it.
    const priorExternalCommands = [...this.externalCommands];
    const pending = (async () => {
      await Promise.allSettled(priorExternalCommands);
      await this.waitForCommandIdle();
      if (!this.active || this.stopping || this.dead || !this.paused || this.authorityPauseState !== "paused") {
        throw new Error("E_SAVE_SESSION_CHANGED");
      }
      const result = await operation();
      // A queued resume may already be presenting `resuming`; it waits for this
      // external command and will apply its own newer receipt after this one.
      if (!this.active || this.stopping || this.dead || this.snapshots.view()?.worldEpoch !== worldEpoch) {
        throw new Error("E_SAVE_SESSION_CHANGED");
      }
      return result;
    })();
    this.externalCommands.add(pending);
    void pending.then(() => {
      this.externalCommands.delete(pending);
      this.resolveCommandIdleWaiters();
    }, () => {
      this.externalCommands.delete(pending);
      this.resolveCommandIdleWaiters();
    });
    return pending;
  }

  private sessionContext(snapshot = this.snapshots.view()): SessionContext | undefined {
    // Only persisted v2 fixtures lack scene identity. Production always uses v3.
    return snapshot?.protocolVersion === 3
      ? { worldId: snapshot.worldId, sceneId: snapshot.sceneId, worldEpoch: snapshot.worldEpoch }
      : undefined;
  }

  private clearControls(): void {
    this.controlSequence++;
    this.input.releaseAll(this.snapshots.view()?.worldEpoch ?? 0, this.clientTimeMs());
    this.actions.length = 0;
    // Rust pause clears movement, pending combat and held guard atomically.
    // A separately queued GuardEnd could arrive in a replacement scene.
  }

  private lifecycleIsCurrent(source: WorldSnapshotEnvelope, sequence: number, lifecycleSequence: number): boolean {
    return lifecycleSequence === this.lifecycleSequence && this.active && !this.stopping && !this.dead && !this.entryLoading &&
      sequence === this.entrySequence && this.snapshotIsCurrent(source);
  }

  private ensurePauseLifecycle(priorityPause = false): Promise<void> {
    if (this.lifecycleTask && !(priorityPause && this.lifecycleResuming)) return this.lifecycleTask;
    // A safety pause may supersede a resume whose receipt is still outstanding.
    // The bridge's ordered command sequence also rejects delayed native resumes.
    const sequence = ++this.lifecycleSequence;
    this.lifecycleResuming = false;
    const task = this.reconcilePauseLifecycle(sequence);
    this.lifecycleTask = task;
    void task.then(() => { if (this.lifecycleTask === task) this.lifecycleTask = null; },
      () => { if (this.lifecycleTask === task) this.lifecycleTask = null; });
    return task;
  }

  private async reconcilePauseLifecycle(lifecycleSequence: number): Promise<void> {
    let resumeDispatched = false;
    let source = this.snapshots.view();
    let sequence = this.entrySequence;
    let context = this.sessionContext(source);
    try {
      while (this.active && !this.stopping) {
        if (this.entryLoading || this.dead) return;
        source = this.snapshots.view();
        if (!source) throw new Error("E_SESSION_CONTEXT_UNAVAILABLE");
        sequence = this.entrySequence;
        context = this.sessionContext(source);
        this.refreshPauseIntent();
        if (this.desiredPaused && this.authorityPauseState === "paused") {
          this.paused = true;
          this.stopSchedulers();
          this.publishPauseState("paused");
          return;
        }
        if (!this.desiredPaused && this.authorityPauseState === "running" && !this.resumeReceiptRequired) {
          this.paused = false;
          this.startSchedulers();
          this.publishPauseState("running");
          // Held refresh cues were visual-only. Fetch fresh resume-owned IDs
          // after audio is resumed; never replay a buffered sound on focus.
          if (resumeDispatched) await this.pollEvents(this.snapshots.view() ?? source, false,
            () => this.lifecycleIsCurrent(source!, sequence, lifecycleSequence));
          return;
        }

        const targetPaused = this.desiredPaused;
        this.lifecycleResuming = !targetPaused;
        this.paused = true;
        this.stopSchedulers();
        this.publishPauseState(targetPaused ? "pausing" : "resuming");
        // Safety pause owns a priority lane; do not wait for hung input, event
        // polling or an interaction. Resume retains save/build ordering.
        if (!targetPaused) {
          await this.waitForExternalCommands();
          await this.waitForCommandIdle();
        }
        if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
        if (!targetPaused && this.rendererEntryVerified) {
          if (!await this.preparePausedPresentation(source, () => this.lifecycleIsCurrent(source!, sequence, lifecycleSequence))) {
            if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
            continue;
          }
        }
        this.refreshPauseIntent();
        if (this.desiredPaused !== targetPaused) continue;
        if (!targetPaused && this.rendererEntryVerified) this.assertEntryFrame(this.snapshots.view() ?? source);
        this.authorityPauseState = "unknown";
        const command = targetPaused ? this.client.pause : this.client.resume;
        if (!command) throw new Error(targetPaused ? "E_FORMAL_PAUSE_UNAVAILABLE" : "E_FORMAL_RESUME_UNAVAILABLE");
        resumeDispatched = !targetPaused;
        this.lifecycleResuming = !targetPaused;
        let snapshot: WorldSnapshotEnvelope;
        try { snapshot = await command.call(this.client, context); }
        catch (error) {
          if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
          throw error;
        }
        if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) {
          // A resume may commit after the priority stop pause. Re-pause only
          // its original identity; a new world must never receive this command.
          if (this.stopping && !targetPaused) await this.pauseStoppedContext(context).catch(() => undefined);
          return;
        }
        this.lifecycleResuming = false;
        if (context && (snapshot.protocolVersion !== 3 || snapshot.worldId !== context.worldId ||
            snapshot.sceneId !== context.sceneId || snapshot.worldEpoch !== context.worldEpoch)) {
          throw new Error("E_LIFECYCLE_RECEIPT_CONTEXT_MISMATCH");
        }
        if (!targetPaused && this.rendererEntryVerified) this.assertEntryFrame(snapshot);
        await this.acceptSnapshot(snapshot);
        if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
        this.authorityPauseState = targetPaused ? "paused" : "running";
        if (!targetPaused) this.resumeReceiptRequired = false;
      }
    } catch (error) {
      if (!source || !this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
      this.lifecycleResuming = false;
      this.authorityPauseState = "unknown";
      this.paused = true;
      this.stopSchedulers();
      if (resumeDispatched) {
        try {
          if (!this.client.pause) throw new Error("E_FORMAL_PAUSE_UNAVAILABLE");
          await this.client.pause(context);
          if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
          this.authorityPauseState = "paused";
        } catch {
          if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
          this.authorityPauseState = "unknown";
        }
      }
      if (!this.lifecycleIsCurrent(source, sequence, lifecycleSequence)) return;
      this.publishPauseState("error", error instanceof Error ? error.message : String(error));
    }
  }

  /** Refresh the committed phase while Rust is still held, then prove its visible
   * frame before the higher-sequence resume can release the owner. No timer resets. */
  private async preparePausedPresentation(source: WorldSnapshotEnvelope, isCurrent: () => boolean): Promise<boolean> {
    if (!this.client.pause) throw new Error("E_FORMAL_PAUSE_UNAVAILABLE");
    const context = this.sessionContext(source);
    this.authorityPauseState = "unknown";
    const held = await this.client.pause(context);
    if (!isCurrent()) return false;
    if (context && (held.protocolVersion !== 3 || held.worldId !== context.worldId ||
        held.sceneId !== context.sceneId || held.worldEpoch !== context.worldEpoch)) {
      throw new Error("E_LIFECYCLE_RECEIPT_CONTEXT_MISMATCH");
    }
    this.applySnapshot(held);
    if (!isCurrent()) return false;
    this.authorityPauseState = "paused";
    const current = this.snapshots.view() ?? held;
    await this.pollEvents(current, true, isCurrent);
    if (!isCurrent()) return false;
    if (this.snapshots.deferredEventCount() !== 0) throw new Error("E_HELD_PRESENTATION_EVENTS_AHEAD");
    await this.renderInFlight?.catch(() => undefined);
    if (!isCurrent()) return false;
    await this.renderSnapshot(1);
    if (!isCurrent()) return false;
    this.assertEntryFrame(current);
    this.refreshPauseIntent();
    return !this.desiredPaused;
  }

  private publishPauseState(status: PauseStatus, message?: string): void {
    const normalizedMessage = message ?? "";
    if (this.pauseStatus === status && this.pauseMessage === normalizedMessage) return;
    this.pauseStatus = status;
    this.pauseMessage = normalizedMessage;
    this.onPauseState?.(status, message);
  }

  private async waitForExternalCommands(): Promise<void> {
    while (this.externalCommands.size > 0 && !this.stopping) {
      await Promise.allSettled([...this.externalCommands]);
    }
  }

  private async waitForCommandIdle(): Promise<void> {
    while (this.commandBusy && !this.stopping) {
      await new Promise<void>(resolve => this.commandIdleWaiters.push(resolve));
    }
  }

  private resolveCommandIdleWaiters(): void {
    while (this.commandIdleWaiters.length) this.commandIdleWaiters.shift()?.();
  }

  /** Accepts the post-interaction authoritative snapshot through the normal epoch/event path. */
  acceptAuthoritativeSnapshot(snapshot: WorldSnapshotEnvelope): Promise<void> {
    return this.acceptSnapshot(snapshot);
  }

  async stop(reason: StopReason = "hub"): Promise<void> {
    if (this.stopPromise) return this.stopPromise;
    if (!this.active && !this.stopping) return;
    // Capture authority before callbacks, cancellation or any await can replace it.
    const context = this.sessionContext();
    // Registered save/build IPC may be unbound to an epoch at the native side.
    // Detaching it cannot authorize a new journey while its outcome is unknown.
    this.stopMutationUncertain = this.externalCommands.size > 0;
    this.stopping = true;
    this.snapshots.clearDeferredEvents();
    this.paused = true;
    this.stopSchedulers();
    this.unbindEvents();
    this.clearControls();
    this.resolveCommandIdleWaiters();
    // No receipt is applied and no late completion publishes state to the host.
    // Cleanup must not depend on the bridge being responsive.
    const pause = this.dead ? Promise.resolve() : this.pauseStoppedContext(context);
    if (this.dead) this.stopAuthorityStatus = "dead";
    void pause.catch(() => undefined);
    try { this.onClearTransientPresentation?.(); } catch { /* Continue teardown. */ }
    try { this.renderer.cancel?.(); } catch { /* Continue teardown. */ }
    this.stopPromise = this.finishStop(reason, pause);
    return this.stopPromise;
  }

  private async pauseStoppedContext(context: SessionContext | undefined): Promise<void> {
    const sequence = ++this.stopPauseSequence;
    this.stopAuthorityStatus = "pending";
    this.stopAuthorityFailure = undefined;
    try {
      if (!this.client.pause) throw new Error("E_FORMAL_PAUSE_UNAVAILABLE");
      await this.client.pause(context);
      if (sequence === this.stopPauseSequence) this.stopAuthorityStatus = "confirmed";
    } catch (error) {
      if (sequence === this.stopPauseSequence) {
        this.stopAuthorityStatus = "failed";
        this.stopAuthorityFailure = error;
      }
      throw error;
    }
  }

  private clientTimeMs(): number {
    this.lastClientTimeMs = Math.max(this.lastClientTimeMs, Math.floor(this.scheduler.clientTimeMs()));
    return this.lastClientTimeMs;
  }

  private bindEvents(): void {
    this.windowTarget?.addEventListener("keydown", this.keyDownListener);
    this.windowTarget?.addEventListener("keyup", this.keyUpListener);
    this.windowTarget?.addEventListener("blur", this.blurListener);
    this.windowTarget?.addEventListener("focus", this.focusListener);
    this.windowTarget?.addEventListener("pagehide", this.pageHideListener);
    this.documentTarget?.addEventListener("visibilitychange", this.visibilityListener);
    this.pointerTarget?.addEventListener("pointermove", this.pointerMoveListener);
    this.pointerTarget?.addEventListener("pointerdown", this.pointerDownListener);
  }

  private unbindEvents(): void {
    this.windowTarget?.removeEventListener("keydown", this.keyDownListener);
    this.windowTarget?.removeEventListener("keyup", this.keyUpListener);
    this.windowTarget?.removeEventListener("blur", this.blurListener);
    this.windowTarget?.removeEventListener("focus", this.focusListener);
    this.windowTarget?.removeEventListener("pagehide", this.pageHideListener);
    this.documentTarget?.removeEventListener("visibilitychange", this.visibilityListener);
    this.pointerTarget?.removeEventListener("pointermove", this.pointerMoveListener);
    this.pointerTarget?.removeEventListener("pointerdown", this.pointerDownListener);
  }

  private startSchedulers(): void {
    if (!this.active || this.paused || this.entryLoading || this.stopping) return;
    if (this.intervalHandle === undefined) this.intervalHandle = this.scheduler.setInterval(() => this.inputTick(), STEP_MS);
    if (this.animationFrameHandle === null) this.animationFrameHandle = this.scheduler.requestAnimationFrame(this.frameCallback);
  }

  private stopSchedulers(): void {
    if (this.intervalHandle !== undefined) {
      this.scheduler.clearInterval(this.intervalHandle);
      this.intervalHandle = undefined;
    }
    if (this.animationFrameHandle !== null) {
      this.scheduler.cancelAnimationFrame(this.animationFrameHandle);
      this.animationFrameHandle = null;
    }
  }

  private readonly frameCallback = (timestamp: number): void => {
    this.animationFrameHandle = null;
    if (!this.active || this.paused || this.entryLoading || this.stopping) return;
    this.animationFrameHandle = this.scheduler.requestAnimationFrame(this.frameCallback);
    if (this.renderBusy) return;
    const elapsed = Math.max(0, timestamp - this.lastSnapshotAt);
    const alpha = Math.min(1, elapsed / STEP_MS);
    const source = this.snapshots.view();
    const sequence = this.entrySequence;
    void this.renderSnapshot(alpha).catch(error => {
      if (source && sequence === this.entrySequence && this.snapshotIsCurrent(source)) this.fail(error);
    });
  };

  private inputTick(): void {
    if (!this.active || this.paused || this.entryLoading || this.stopping || this.commandBusy || this.actions.length > 0) return;
    const snapshot = this.snapshots.view();
    if (!snapshot) return;
    this.refreshPointerAim();
    const sample: InputState = this.input.sample(snapshot.worldEpoch, this.clientTimeMs());
    this.commandBusy = true;
    const operation = this.acceptCommandResult(this.client.submitInput(sample), this.entrySequence, this.controlSequence);
    this.trackCommand(operation);
  }

  private enqueueAction(action: ActionCommand, aim?: Pick<InputState, "aimX" | "aimZ">): void {
    this.actions.push(aim ? { action, aim } : { action });
    void this.drainActions();
  }

  private drainActions(): void {
    if (!this.active || this.paused || this.stopping || this.dead || this.entryLoading || this.commandBusy || this.actions.length === 0) return;
    const queued = this.actions.shift();
    if (!queued) return;
    this.commandBusy = true;
    const entrySequence = this.entrySequence;
    const controlSequence = this.controlSequence;
    const operation = queued.aim
      ? this.submitAimedAction(queued.action, queued.aim, entrySequence, controlSequence)
      : this.acceptCommandResult(this.client.submitAction(queued.action), entrySequence, controlSequence);
    this.trackCommand(operation);
  }

  private async submitAimedAction(action: ActionCommand, aim: Pick<InputState, "aimX" | "aimZ">, entrySequence: number, controlSequence: number): Promise<void> {
    const source = this.snapshots.view();
    // Keep the directional edge's aim/time, but sample movement at dispatch so
    // queued clicks/keys cannot replay movement that the player has released.
    const input = { ...this.input.sample(action.worldEpoch, action.clientTimeMs), ...aim };
    await this.acceptCommandResult(this.client.submitInput(input), entrySequence, controlSequence);
    // The aim receipt may reveal death/entry, or pause/stop may overtake it.
    // Never revive a canceled edge after a resume or apply it to another scene.
    if (!source || !this.active || this.paused || this.entryLoading || this.stopping || this.dead ||
        entrySequence !== this.entrySequence || controlSequence !== this.controlSequence || !this.snapshotIsCurrent(source)) return;
    await this.acceptCommandResult(this.client.submitAction(action), entrySequence, controlSequence);
  }

  /** A new entry invalidates transport results/errors dispatched by the old
   * scene. Errors from accepting a still-current result remain observable. */
  private async acceptCommandResult(command: Promise<WorldSnapshotEnvelope>, entrySequence: number, controlSequence: number): Promise<void> {
    const source = this.snapshots.view();
    let snapshot: WorldSnapshotEnvelope;
    try { snapshot = await command; }
    catch (error) {
      if (this.stopping || entrySequence !== this.entrySequence) return;
      const message = error instanceof Error ? error.message : String(error);
      if (controlSequence !== this.controlSequence && /(?:^|\b)E_RUNTIME_PAUSED(?:$|\b)/.test(message)) return;
      if (!/(?:^|\b)E_RUNTIME_DEAD(?:$|\b)/.test(message)) throw error;
      // The owner may die between client submissions. Mutation IPC is correctly
      // rejected; fetch its terminal authority view instead of guessing death.
      if (!source || !this.client.snapshot) throw new Error("E_DEATH_SNAPSHOT_UNAVAILABLE");
      let fatal: WorldSnapshotEnvelope;
      try { fatal = await this.client.snapshot(); }
      catch (lookupError) {
        if (this.stopping || entrySequence !== this.entrySequence) return;
        throw lookupError;
      }
      if (this.stopping || entrySequence !== this.entrySequence) return;
      if (!this.snapshotIsCurrent(source) || fatal.protocolVersion !== source.protocolVersion ||
          fatal.worldEpoch !== source.worldEpoch || fatal.worldId !== source.worldId ||
          fatal.protocolVersion === 3 && source.protocolVersion === 3 && fatal.sceneId !== source.sceneId ||
          !isPlayerDead(fatal)) throw new Error("E_DEATH_SNAPSHOT_IDENTITY");
      await this.acceptSnapshot(fatal);
      if (!this.dead) throw new Error("E_DEATH_SNAPSHOT_NOT_ACCEPTED");
      return;
    }
    if (this.stopping || entrySequence !== this.entrySequence) return;
    // Priority pause may overtake this receipt. Preserve a current fatal result,
    // but never let canceled living input overwrite a newer pause snapshot.
    if (controlSequence !== this.controlSequence && !isPlayerDead(snapshot)) return;
    await this.acceptSnapshot(snapshot);
  }

  private trackCommand(operation: Promise<void | undefined>): void {
    this.inFlight = operation;
    const source = this.snapshots.view();
    const sequence = this.entrySequence;
    void operation.catch(error => {
      if (!this.stopping && source && sequence === this.entrySequence && this.snapshotIsCurrent(source)) this.fail(error);
    }).finally(() => {
      if (this.inFlight === operation) this.inFlight = null;
      this.commandBusy = false;
      this.resolveCommandIdleWaiters();
      if (!this.stopping) this.drainActions();
    });
  }

  private applySnapshot(snapshot: WorldSnapshotEnvelope): boolean {
    if (!this.snapshots.accept(snapshot)) return false;
    this.lastSnapshotAt = this.scheduler.now();
    const newlyDead = !this.dead && isPlayerDead(snapshot);
    if (newlyDead) {
      this.dead = true;
      this.paused = true;
      this.desiredPaused = true;
      this.authorityPauseState = "paused";
      this.resumeReceiptRequired = false;
      this.entryLoading = false;
      this.entryToken = undefined;
      this.stopSchedulers();
      this.input.releaseAll(snapshot.worldEpoch, this.clientTimeMs());
      this.actions.length = 0;
      this.controlSequence++;
      this.onClearTransientPresentation?.();
    }
    // Only accepted snapshots reach consumers. An ignored old living receipt may
    // not overwrite fatal HUD/core state even though the snapshot store rejected it.
    this.onSnapshot?.(snapshot);
    if (this.dead) this.publishPauseState("dead");
    if (newlyDead) this.onDeath?.(snapshot);
    return true;
  }

  private async acceptSnapshot(snapshot: WorldSnapshotEnvelope): Promise<void> {
    if (this.stopping) return;
    const previous = this.snapshots.view();
    if (!this.applySnapshot(snapshot)) return;
    if (this.dead) {
      await this.renderInFlight?.catch(() => undefined);
      if (!this.stopping) await this.renderSnapshot(1);
      return;
    }
    if (snapshot.protocolVersion === 3 && snapshot.entryToken) {
      await this.beginEntry(snapshot);
      return;
    }
    if (previous && previous.worldEpoch !== snapshot.worldEpoch) {
      this.clearControls();
    }
    await this.pollEvents(snapshot);
  }

  private snapshotIsCurrent(snapshot: WorldSnapshotEnvelope): boolean {
    const current = this.snapshots.view();
    return !!current && current.worldEpoch === snapshot.worldEpoch &&
      (current.protocolVersion !== 3 || snapshot.protocolVersion !== 3 ||
        current.worldId === snapshot.worldId && current.sceneId === snapshot.sceneId);
  }

  /** Read-only deadline: timing out never permits the eventual reply to mutate
   * a cursor or admit cues. The owner remains held throughout preparation. */
  private async readPresentationEvents(worldEpoch: number, cursor: number): Promise<PresentationEvent[]> {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      const read = this.client.events(worldEpoch, cursor);
      const deadline = new Promise<never>((_, reject) => {
        timeout = setTimeout(() => reject(new Error("E_PRESENTATION_EVENTS_TIMEOUT")), this.presentationTimeoutMs);
      });
      return await Promise.race([read, deadline]);
    } finally {
      if (timeout !== undefined) clearTimeout(timeout);
    }
  }

  private async pollEvents(snapshot: WorldSnapshotEnvelope, admitWhilePaused = false, stillCurrent: () => boolean = () => true): Promise<void> {
    const contextRevision = this.snapshots.eventContextRevision();
    const events = await this.readPresentationEvents(snapshot.worldEpoch, this.snapshots.eventCursor());
    if (this.stopping || this.dead || !this.snapshotIsCurrent(snapshot) || !stillCurrent() ||
        contextRevision !== this.snapshots.eventContextRevision()) return;
    // Background polling while paused must not consume cues behind the visual
    // admission barrier. A held refresh explicitly authorizes visual-only intake.
    if (this.paused && !admitWhilePaused) return;
    if (events.some(event => event && event.protocolVersion !== undefined && event.protocolVersion !== 2)) throw new Error("E_SESSION_EVENT_V2_REQUIRED");
    const accepted = this.snapshots.acceptEvents(events);
    if (accepted.length && (!this.paused || admitWhilePaused) && !this.stopping) {
      const snapshotFiltered = this.onEventsWithSnapshot?.(accepted, this.snapshots.view() ?? snapshot);
      if (!this.paused) this.onEvents?.(Array.isArray(snapshotFiltered) ? snapshotFiltered : accepted);
    }
    if (snapshot.protocolVersion !== 3 || !this.client.soundCues || this.stopping || this.paused) return;
    const cues = await this.client.soundCues(snapshot.worldEpoch, this.snapshots.soundCueCursor());
    if (this.stopping || this.dead || !this.snapshotIsCurrent(snapshot)) return;
    const acceptedCues = this.snapshots.acceptSoundCues(cues);
    const current = this.snapshots.view();
    if (acceptedCues.length && current) this.onSoundCues?.(acceptedCues, current);
  }

  private async renderSnapshot(alpha: number): Promise<void> {
    const snapshot = this.snapshots.view();
    if (!snapshot || this.renderBusy || this.stopping) return;
    this.renderBusy = true;
    const playerPosition = this.snapshots.interpolatedPlayer(alpha) ?? undefined;
    const actorPositions = new Map<string, Vec3>();
    for (const actor of actorViews(snapshot)) {
      const position = this.snapshots.interpolatedActor(actor.entityId, alpha);
      if (position) actorPositions.set(actor.entityId, position);
    }
    const operation = Promise.resolve().then(() => this.renderer.render(snapshot, playerPosition, actorPositions));
    this.renderInFlight = operation;
    try { await operation; }
    finally {
      if (this.renderInFlight === operation) this.renderInFlight = null;
      this.renderBusy = false;
    }
  }

  private fail(error: unknown): void {
    if (this.stopping || !this.active || this.dead) return;
    void this.stop("error").catch(() => undefined);
    // Report while this session still owns the failure, never after delayed
    // teardown could have allowed a newer journey to take over the host UI.
    this.onError?.(error);
  }

  private async finishStop(reason: StopReason, pause: Promise<void>): Promise<void> {
    try {
      if (reason === "hub") {
        // Return/save must follow registered equipment/save operations. Safety
        // pause was already dispatched before entering this ordering barrier.
        let timeout: ReturnType<typeof setTimeout> | undefined;
        try {
          const drain = (async () => {
            await Promise.allSettled([...this.externalCommands]);
            await this.inFlight?.catch(() => undefined);
            await pause;
          })();
          const deadline = new Promise<never>((_, reject) => {
            timeout = setTimeout(() => reject(new Error("E_SESSION_STOP_TIMEOUT")), this.stopTimeoutMs);
          });
          await Promise.race([drain, deadline]);
          this.stopMutationUncertain = false;
        } finally {
          if (timeout !== undefined) clearTimeout(timeout);
        }
      }
      if (reason === "replace" && this.stopMutationUncertain) throw new Error("E_SESSION_STOP_UNCERTAIN_MUTATION");
    } finally {
      this.active = false;
      try {
        await this.renderInFlight?.catch(() => undefined);
        await this.renderer.destroy?.();
      } finally { this.onCleanup?.(); }
    }
  }

}
