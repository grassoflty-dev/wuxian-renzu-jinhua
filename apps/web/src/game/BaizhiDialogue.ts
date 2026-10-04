import type { BaizhiCommandReceipt, TauriClient } from "../bridge/tauri-client.js";
import type { BaizhiChoice, BaizhiDialogueTicket, WorldSnapshotV3 } from "../protocol/types.js";
import { resolveBaizhiPresentation } from "../renderer/BaizhiPresentation.js";
import type { SessionLoop } from "./SessionLoop.js";

export const BAIZHI_INTERACTION_ID = "gh_bz_first_contact";
export const BAIZHI_RESULTS: Record<BaizhiChoice, { player: string; baizhi: string }> = {
  taken: { player: "我会帮你离开这里。", baizhi: "好，我会等你找到安全的出口。" },
  left: { player: "我不能带你走。", baizhi: "我明白。" },
  unresolved: { player: "我还需要确认一些事情。", baizhi: "那就先别打开那扇门。" },
};
export function baizhiInteractionPrompt(snapshot: WorldSnapshotV3): string | null {
  const projection = resolveBaizhiPresentation(snapshot);
  const p = snapshot.player.transform.positionM;
  if (!projection?.baizhi.canInteract || snapshot.entryToken || snapshot.worldEpoch < 1 || snapshot.player.currentHp <= 0 ||
      ![p.xM, p.yM, p.zM].every(Number.isFinite) || Math.hypot(p.xM - 8, p.yM, p.zM - 10.5) > 2.5) return null;
  return projection.baizhi.choice === "unresolved" ? "[F] 与白芷交谈" : "[F] 查看白芷状态";
}
export interface BaizhiOwner {
  loop: Pick<SessionLoop, "beginDialoguePause" | "closeDialoguePause" | "ownsDialoguePause" | "runWhilePaused" |
    "acceptAuthoritativeSnapshot" | "snapshots" | "pausePresentationState" | "acceptsExternalResults" | "isDead" | "sceneEntrySequence">;
  isCurrent(): boolean;
}
export type BaizhiPage = "opening" | "dialogue" | "choices" | "confirmation" | "result";
let ownerSequence = 0;

/** Four-page presentation backed only by a native ticket and the GH read projection. */
export class BaizhiDialogue {
  private generation = 0;
  private owner: BaizhiOwner | null = null;
  private source: WorldSnapshotV3 | null = null;
  private ownerId = "";
  private ticket: BaizhiDialogueTicket | null = null;
  private entrySequence = 0;
  private requestSequence = 0;
  private requestId = "";
  page: BaizhiPage = "opening";
  choice: BaizhiChoice = "unresolved";
  busy = false;
  message = "";
  get visible(): boolean { return this.owner !== null; }
  get ready(): boolean { return this.current() && !!this.ticket && this.owner?.loop.pausePresentationState === "paused"; }
  constructor(private readonly client: Pick<TauriClient, "beginBaizhi" | "commitBaizhi" | "closeBaizhi" | "snapshot">,
    private readonly changed: () => void) {}

  private sameSession(): boolean {
    const current = this.owner?.loop.snapshots.view();
    return !!this.owner?.isCurrent() && this.owner.loop.acceptsExternalResults && !this.owner.loop.isDead &&
      this.owner.loop.sceneEntrySequence === this.entrySequence && !!current && !!this.source && current.protocolVersion === 3 &&
      !current.entryToken && current.worldId === this.source.worldId && current.sceneId === this.source.sceneId && current.worldEpoch === this.source.worldEpoch;
  }
  private current(): boolean { return this.sameSession() && this.owner!.loop.ownsDialoguePause(this.ownerId); }
  reconcile(): void { if (this.owner && (!this.sameSession() || !this.owner.loop.ownsDialoguePause(this.ownerId))) this.reset(); }
  reset(): void {
    this.generation++; this.owner = null; this.source = null; this.ticket = null; this.ownerId = "";
    this.busy = false; this.message = ""; this.page = "opening"; this.choice = "unresolved"; this.requestId = ""; this.changed();
  }
  async open(owner: BaizhiOwner, source: WorldSnapshotV3): Promise<void> {
    if (this.visible || owner.loop.pausePresentationState !== "running" || !baizhiInteractionPrompt(source)) return;
    const generation = ++this.generation;
    this.owner = owner; this.source = source; this.entrySequence = owner.loop.sceneEntrySequence;
    this.ownerId = `baizhi-${Date.now().toString(36)}-${++ownerSequence}`;
    const ownerId = this.ownerId;
    this.busy = true; this.page = "opening"; this.message = "正在连接对话…"; this.changed();
    try {
      const result = await owner.loop.beginDialoguePause(ownerId, () => this.client.beginBaizhi(source, ownerId, generation));
      if (generation !== this.generation || !this.current()) return;
      const projection = resolveBaizhiPresentation(result.receipt.snapshot);
      if (!result.ticket || !projection) throw new Error("E_BAIZHI_BEGIN_PROJECTION_INVALID");
      this.ticket = result.ticket; this.choice = projection.baizhi.choice;
      this.page = this.choice === "unresolved" ? "dialogue" : "result";
      this.busy = false; this.message = ""; this.changed();
    } catch {
      if (generation !== this.generation || this.owner !== owner) return;
      this.reset();
    }
  }
  next(): void { if (this.ready && !this.busy && this.page === "dialogue") { this.page = "choices"; this.message = ""; this.changed(); } }
  select(choice: BaizhiChoice): void {
    if (!this.ready || this.busy || this.page !== "choices" || !["taken", "left", "unresolved"].includes(choice)) return;
    this.choice = choice; this.requestId = ""; this.message = "";
    this.page = choice === "unresolved" ? "result" : "confirmation"; this.changed();
  }
  back(): void {
    if (!this.ready || this.busy) return;
    if (this.page === "confirmation") this.page = "choices";
    else if (this.page === "choices") this.page = "dialogue";
    this.message = ""; this.changed();
  }
  escape(): void {
    if (this.busy || !this.ready) return;
    if (this.page === "choices" || this.page === "confirmation") this.back();
    else void this.close();
  }
  async confirm(): Promise<void> {
    if (!this.ready || this.busy || this.page !== "confirmation" || this.choice === "unresolved" || !this.owner || !this.ticket) return;
    const owner = this.owner, generation = this.generation, ticket = { ...this.ticket }, choice = this.choice;
    this.requestId ||= `${this.ownerId}-choice-${++this.requestSequence}`;
    const requestId = this.requestId;
    this.busy = true; this.message = "正在保存选择…"; this.changed();
    try {
      const result = await owner.loop.runWhilePaused(() => this.client.commitBaizhi(ticket, requestId, choice));
      if (generation !== this.generation || !this.current()) return;
      // The native receipt is the persistence boundary. Presentation-event reads
      // must never turn an already-saved terminal choice into a retryable failure.
      this.ticket = result.ticket; this.page = "result"; this.message = "";
      try { await this.accept(result); } catch { /* Keep the confirmed result while the world remains held. */ }
      if (generation !== this.generation || !this.current()) return;
      this.busy = false; this.changed();
    } catch {
      if (generation !== this.generation || !this.current()) return;
      // A lost response may follow a successful atomic write; successful request
      // replay is intentionally rejected by Rust. Reconcile read-only authority
      // before presenting a retry, without inventing cancellation or another write.
      try {
        const snapshot = await owner.loop.runWhilePaused(() => this.client.snapshot());
        if (generation !== this.generation || !this.current()) return;
        const projection = resolveBaizhiPresentation(snapshot);
        const current = owner.loop.snapshots.view()!;
        if (projection?.baizhi.choice === choice && snapshot.worldEpoch === ticket.worldEpoch &&
            snapshot.worldId === ticket.worldId && snapshot.sceneId === ticket.sceneId && !snapshot.entryToken &&
            snapshot.player.currentHp > 0 && snapshot.authorityRevision >= current.authorityRevision && snapshot.serverTick >= current.serverTick) {
          this.page = "result"; this.message = "";
          try { await this.acceptSnapshot(snapshot); } catch { /* The confirmed terminal projection still stands. */ }
          if (generation !== this.generation || !this.current()) return;
          this.busy = false; this.changed(); return;
        }
      } catch { /* An unavailable read cannot authorize a result; retain the same retry payload. */ }
      if (generation !== this.generation || !this.current()) return;
      this.busy = false; this.message = "选择保存失败，请重试。"; this.changed();
    }
  }
  private accept(result: BaizhiCommandReceipt): Promise<void> { return this.acceptSnapshot(result.receipt.snapshot); }
  private async acceptSnapshot(snapshot: WorldSnapshotV3): Promise<void> {
    const current = this.owner!.loop.snapshots.view()!;
    if (snapshot.worldId !== this.source!.worldId || snapshot.sceneId !== this.source!.sceneId ||
        snapshot.worldEpoch !== this.source!.worldEpoch || snapshot.authorityRevision < current.authorityRevision ||
        snapshot.serverTick < current.serverTick) throw new Error("E_BAIZHI_RECEIPT_STALE");
    await this.owner!.loop.acceptAuthoritativeSnapshot(snapshot);
  }
  async close(): Promise<void> {
    if (!this.ready || this.busy || !this.owner || !this.ticket) return;
    const owner = this.owner, generation = this.generation, ticket = { ...this.ticket }, ownerId = this.ownerId;
    this.busy = true; this.message = "正在关闭对话…"; this.changed();
    try {
      await owner.loop.closeDialoguePause(ownerId, () => this.client.closeBaizhi(ticket));
      if (generation === this.generation && this.owner === owner) this.reset();
    } catch {
      if (generation !== this.generation || !this.current()) return;
      this.busy = false; this.message = "对话关闭失败，请重试。"; this.changed();
    }
  }
}
