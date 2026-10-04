import type { EnhancementTerminalContext, FirstEnhancementReceipt, SceneCommandReceipt, TauriClient } from "../bridge/tauri-client.js";
import type { WorldSnapshotV3 } from "../protocol/types.js";
import type { SessionLoop } from "./SessionLoop.js";

export function pendingFirstEvolution(snapshot: WorldSnapshotV3): boolean {
  return snapshot.capabilities.firstEnhancementChoice === null && snapshot.progression.worlds.some(row =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === "grey_hive" &&
    (row as Record<string, unknown>).completed === true);
}

export interface EvolutionOwner {
  loop: Pick<SessionLoop, "pause" | "resume" | "runWhilePaused" | "acceptAuthoritativeSnapshot" | "snapshots" | "pausePresentationState" | "acceptsExternalResults" | "isDead">;
  isCurrent(): boolean;
}

/** Owns exactly one terminal receipt and its confirmed pause, never an ordinary pause menu. */
export class FirstEvolution {
  private generation = 0;
  private owner: EvolutionOwner | null = null;
  private source: WorldSnapshotV3 | null = null;
  private context: EnhancementTerminalContext | null = null;
  busy = false;
  message = "";
  get visible(): boolean { return this.owner !== null; }
  get ready(): boolean { return this.current() && this.context !== null && this.owner?.loop.pausePresentationState === "paused"; }
  constructor(private readonly client: Pick<TauriClient, "confirmedPauseSequence" | "chooseFirstEnhancement">,
    private readonly changed: () => void) {}

  private current(): boolean {
    const current = this.owner?.loop.snapshots.view();
    return !!this.owner?.isCurrent() && this.owner.loop.acceptsExternalResults && !this.owner.loop.isDead &&
      !!current && !!this.source && current.protocolVersion === 3 &&
      current.worldId === this.source.worldId && current.sceneId === this.source.sceneId && current.worldEpoch === this.source.worldEpoch;
  }

  reconcile(): void {
    if (this.owner && (!this.current() || this.owner.loop.pausePresentationState === "running")) this.reset();
  }
  reset(): void { this.generation++; this.owner = null; this.source = null; this.context = null; this.busy = false; this.message = ""; this.changed(); }

  async open(owner: EvolutionOwner, receipt: SceneCommandReceipt): Promise<void> {
    const snapshot = receipt.snapshot;
    if (this.visible || !receipt.applied || receipt.commandId !== receipt.requestId ||
        !receipt.requestId || snapshot.worldId !== "return_station" || snapshot.sceneId !== "rs_core_room" ||
        !pendingFirstEvolution(snapshot) || owner.loop.pausePresentationState !== "running") return;
    const generation = ++this.generation;
    this.owner = owner; this.source = snapshot; this.busy = true; this.message = "正在连接进化终端…"; this.changed();
    try {
      await owner.loop.pause();
      if (generation !== this.generation || this.owner !== owner) return;
      if (!this.current()) { this.reset(); return; }
      if (String(owner.loop.pausePresentationState) !== "paused") throw new Error("E_ENHANCEMENT_PAUSE_UNCONFIRMED");
      this.context = { id: "rs_capability_terminal_marker", requestId: receipt.requestId, worldEpoch: snapshot.worldEpoch,
        pauseCommandSequence: this.client.confirmedPauseSequence(snapshot) };
      this.message = "灰巢首通强化 · 任选一项";
    } catch (error) {
      if (generation !== this.generation || this.owner !== owner) return;
      this.message = `终端未就绪：${String(error)}`;
    }
    if (generation !== this.generation || this.owner !== owner) return;
    this.busy = false; this.changed();
  }

  async close(): Promise<void> {
    if (!this.owner || this.busy) return;
    const owner = this.owner;
    const resume = this.ready;
    this.reset();
    if (resume && owner.isCurrent()) await owner.loop.resume();
  }

  async choose(capabilityId: string): Promise<FirstEnhancementReceipt | null> {
    if (!this.ready || this.busy || !this.owner || !this.context) return null;
    const owner = this.owner, source = this.source!, context = { ...this.context }, generation = this.generation;
    this.busy = true; this.message = "正在确认并保存强化…"; this.changed();
    try {
      const receipt = await owner.loop.runWhilePaused(() => this.client.chooseFirstEnhancement(capabilityId, context));
      if (generation !== this.generation || this.owner !== owner) return null;
      if (!this.current()) { this.reset(); return null; }
      if (owner.loop.pausePresentationState !== "paused" || receipt.snapshot.worldId !== source.worldId ||
          receipt.snapshot.sceneId !== source.sceneId || receipt.snapshot.worldEpoch !== source.worldEpoch ||
          receipt.authorityRevision <= source.authorityRevision) throw new Error("E_ENHANCEMENT_RECEIPT_STALE_SESSION");
      await owner.loop.acceptAuthoritativeSnapshot(receipt.snapshot);
      if (generation !== this.generation || this.owner !== owner) return null;
      if (!this.current()) { this.reset(); return null; }
      this.busy = false;
      await this.close();
      const current = owner.loop.snapshots.view();
      if (this.generation !== generation + 1 || !owner.isCurrent() || !owner.loop.acceptsExternalResults ||
          !current || current.protocolVersion !== 3 || current.worldEpoch !== source.worldEpoch ||
          current.worldId !== source.worldId || current.sceneId !== source.sceneId) return null;
      return receipt;
    } catch (error) {
      if (generation !== this.generation || this.owner !== owner) return null;
      if (this.current()) { this.message = `选择未完成，可重试：${error instanceof Error ? error.message : String(error)}`; this.busy = false; this.changed(); }
      else this.reset();
      return null;
    }
  }
}
