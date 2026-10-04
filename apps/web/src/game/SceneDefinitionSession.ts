import type { WorldSnapshotEnvelope } from "../protocol/types.js";
import type { SessionRenderer } from "./SessionLoop.js";
import type { WorldRenderer } from "../renderer/WorldRenderer.js";
import { SceneDefinitionLoader } from "../assets/SceneDefinitionLoader.js";
import type { PointerAimOffset, PointerSurfaceRect } from "../renderer/PlayerAimFrame.js";

interface SnapshotIdentity { worldId: string; sceneId: string; worldEpoch: number }
function identity(snapshot: WorldSnapshotEnvelope): SnapshotIdentity {
  if (snapshot.protocolVersion !== 3) throw new Error("E_SCENE_DEFINITION_SNAPSHOT_PROTOCOL");
  return { worldId: snapshot.worldId, sceneId: snapshot.sceneId, worldEpoch: snapshot.worldEpoch };
}
function identityKey(value: SnapshotIdentity): string {
  return `${value.worldId}\0${value.sceneId}\0${value.worldEpoch}`;
}

export type SceneDefinitionSessionErrorCode = "E_SCENE_DEFINITION_CANCELLED" |
  "E_SCENE_DEFINITION_STALE_FRAME" | "E_SCENE_DEFINITION_NOT_PRESENTED";
export class SceneDefinitionSessionError extends Error {
  constructor(readonly code: SceneDefinitionSessionErrorCode) {
    super(code);
    this.name = "SceneDefinitionSessionError";
  }
}

/** Loads one exact authoritative scene identity before forwarding its snapshot to Pixi. */
export class SceneDefinitionSession implements SessionRenderer {
  private currentKey: string | null = null;
  private committedKey: string | null = null;
  private generation = 0;
  private controller: AbortController | null = null;
  private loadPromise: Promise<void> = Promise.resolve();
  private cancelled = false;

  constructor(private readonly renderer: WorldRenderer, private readonly loader: SceneDefinitionLoader) {}

  prepare(snapshot: WorldSnapshotEnvelope, signal?: AbortSignal): Promise<void> {
    if (signal?.aborted) {
      this.invalidate();
      return Promise.reject(new SceneDefinitionSessionError("E_SCENE_DEFINITION_CANCELLED"));
    }
    this.acceptSnapshot(snapshot);
    const controller = this.controller!;
    const generation = this.generation;
    const key = this.currentKey!;
    // Capture this request's controller. Aborting an older prepare must never
    // cancel the newer identity that replaced it.
    const abort = () => { if (controller === this.controller) this.invalidate(); };
    signal?.addEventListener("abort", abort, { once: true });
    return this.waitForCurrent(this.loadPromise, key, generation, controller)
      .finally(() => signal?.removeEventListener("abort", abort));
  }

  acceptSnapshot(snapshot: WorldSnapshotEnvelope): void {
    if (this.cancelled) throw new SceneDefinitionSessionError("E_SCENE_DEFINITION_CANCELLED");
    const sceneIdentity = identity(snapshot);
    const key = identityKey(sceneIdentity);
    if (key === this.currentKey) return;
    this.currentKey = key;
    this.committedKey = null;
    this.generation++;
    const generation = this.generation;
    this.controller?.abort();
    const controller = new AbortController();
    this.controller = controller;
    this.renderer.expectSceneIdentity(sceneIdentity);
    const operation = this.loader.loadForSnapshot(sceneIdentity, { signal: controller.signal }).then(definition => {
      this.assertCurrent(key, generation, controller);
      this.renderer.setSceneDefinition(definition);
    });
    this.loadPromise = operation;
    // Snapshot delivery may start a load before render awaits it. Keep its error
    // available to render/prepare without an unhandled background rejection.
    void operation.catch(() => undefined);
  }

  private assertCurrent(key: string, generation: number, controller: AbortController): void {
    if (this.cancelled) throw new SceneDefinitionSessionError("E_SCENE_DEFINITION_CANCELLED");
    if (generation !== this.generation || key !== this.currentKey || controller !== this.controller) {
      throw new SceneDefinitionSessionError("E_SCENE_DEFINITION_STALE_FRAME");
    }
    if (controller.signal.aborted) throw new SceneDefinitionSessionError("E_SCENE_DEFINITION_CANCELLED");
  }

  private async waitForCurrent(operation: Promise<void>, key: string, generation: number, controller: AbortController): Promise<void> {
    // Observe even if invoking the operation synchronously invalidated this session.
    void operation.catch(() => undefined);
    this.assertCurrent(key, generation, controller);
    let abort!: () => void;
    const cancelled = new Promise<never>((_, reject) => {
      abort = () => {
        try { this.assertCurrent(key, generation, controller); }
        catch (error) { reject(error); }
      };
      controller.signal.addEventListener("abort", abort, { once: true });
    });
    try {
      await Promise.race([operation, cancelled]);
      this.assertCurrent(key, generation, controller);
    } catch (error) {
      this.assertCurrent(key, generation, controller);
      throw error;
    } finally {
      controller.signal.removeEventListener("abort", abort);
    }
  }

  async render(snapshot: WorldSnapshotEnvelope, playerPosition?: Parameters<SessionRenderer["render"]>[1], actorPositions?: Parameters<SessionRenderer["render"]>[2]): Promise<void> {
    this.acceptSnapshot(snapshot);
    const key = identityKey(identity(snapshot));
    const generation = this.generation;
    const controller = this.controller!;
    await this.waitForCurrent(this.loadPromise, key, generation, controller);
    this.assertCurrent(key, generation, controller);
    await this.waitForCurrent(this.renderer.render(snapshot, playerPosition, actorPositions), key, generation, controller);
    // Authority, cancellation and context state can change inside the awaited
    // renderer, or immediately after its draw. None may become readiness.
    this.assertCurrent(key, generation, controller);
    if (!this.renderer.isReadyFor(snapshot)) throw new SceneDefinitionSessionError("E_SCENE_DEFINITION_NOT_PRESENTED");
    this.committedKey = key;
  }

  isReadyFor(snapshot: WorldSnapshotEnvelope): boolean {
    if (this.cancelled || !this.controller || this.controller.signal.aborted || snapshot.protocolVersion !== 3) return false;
    const key = identityKey(identity(snapshot));
    return key === this.currentKey && key === this.committedKey && this.renderer.isReadyFor(snapshot);
  }

  pointerAim(snapshot: WorldSnapshotEnvelope, clientX: number, clientY: number, rect: PointerSurfaceRect): PointerAimOffset | null {
    if (this.cancelled || !this.controller || this.controller.signal.aborted || snapshot.protocolVersion !== 3) return null;
    const key = identityKey(identity(snapshot));
    if (key !== this.currentKey || key !== this.committedKey) return null;
    return this.renderer.pointerAim(snapshot, clientX, clientY, rect);
  }

  async destroy(): Promise<void> {
    this.invalidate();
    await this.renderer.destroy();
  }

  /** Cancellation is terminal for this session; an explicit retry creates a new one. */
  cancel(): void { this.invalidate(); }

  invalidate(): void {
    this.cancelled = true;
    this.generation++;
    this.currentKey = null;
    this.committedKey = null;
    // Revoke the renderer synchronously, before abort listeners/microtasks run.
    this.renderer.invalidate();
    this.controller?.abort();
    this.controller = null;
  }
}
