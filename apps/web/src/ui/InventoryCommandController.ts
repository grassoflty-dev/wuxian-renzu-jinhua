import type { BuildAction } from "../protocol/BuildProjection.js";
import type { WorldSnapshotV3 } from "../protocol/types.js";
import type { SlotCommandReceipt } from "../bridge/tauri-client.js";

export type BuildCommandHandler = (action: BuildAction, source: WorldSnapshotV3) => Promise<SlotCommandReceipt>;

/** Receipt gating shared by main and tests. No response may roll back a newer session/frame. */
export function isCurrentBuildReceipt(active: boolean, current: WorldSnapshotV3 | null,
  source: WorldSnapshotV3, receipt: SlotCommandReceipt): boolean {
  return active && !!current && current.worldEpoch === source.worldEpoch &&
    current.worldId === source.worldId && current.sceneId === source.sceneId &&
    receipt.snapshot.worldEpoch === source.worldEpoch && receipt.snapshot.worldId === source.worldId &&
    receipt.snapshot.sceneId === source.sceneId && receipt.serverTick >= current.serverTick &&
    receipt.authorityRevision >= current.authorityRevision && !!receipt.snapshot.build &&
    (!current.build || receipt.snapshot.build.revision >= current.build.revision);
}

/** Single-flight UI state only. Rust owns quantities, slots, effects and all mutations. */
export class InventoryCommandController {
  private snapshot: WorldSnapshotV3 | null = null;
  private generation = 0;
  private operation: object | null = null;
  feedback = "";
  get pending(): boolean { return this.operation !== null; }
  constructor(private readonly execute: BuildCommandHandler | undefined, private readonly changed: () => void) {}

  apply(snapshot: WorldSnapshotV3 | null): void {
    if (!snapshot || !this.snapshot || snapshot.worldEpoch !== this.snapshot.worldEpoch ||
        snapshot.worldId !== this.snapshot.worldId || snapshot.sceneId !== this.snapshot.sceneId) this.cancel();
    this.snapshot = snapshot;
  }
  cancel(): void { this.generation++; this.operation = null; this.feedback = ""; }

  async submit(action: BuildAction): Promise<void> {
    const source = this.snapshot;
    if (this.pending || !this.execute || !source?.build) return;
    const generation = this.generation; const operation = {};
    this.operation = operation; this.feedback = "正在等待装备确认…"; this.changed();
    try {
      const receipt = await this.execute(action, source);
      if (generation !== this.generation || this.operation !== operation) return;
      if (receipt.snapshot.worldEpoch !== source.worldEpoch || receipt.snapshot.worldId !== source.worldId ||
          receipt.snapshot.sceneId !== source.sceneId) throw new Error("E_BUILD_SESSION_CHANGED");
      this.feedback = receipt.applied || receipt.alreadyApplied ? "装备已更新。"
        : `操作未应用：${receipt.errorCode || "E_BUILD_REJECTED"}`;
    } catch (error) {
      if (generation !== this.generation || this.operation !== operation) return;
      this.feedback = `装备操作失败：${error instanceof Error ? error.message : String(error)}`;
    } finally {
      if (generation === this.generation && this.operation === operation) {
        this.operation = null; this.changed();
      }
    }
  }
}

/** Synchronous closing barrier shared by hub equipment and New/Continue admission. */
export class BuildCloseBarrier {
  closing = false;
  async close(drain: () => Promise<void>, closeWindow: () => Promise<void>): Promise<void> {
    if (this.closing) throw new Error("E_BUILD_CLOSING");
    this.closing = true;
    try { await drain(); await closeWindow(); }
    catch (error) { this.closing = false; throw error; }
  }
}
