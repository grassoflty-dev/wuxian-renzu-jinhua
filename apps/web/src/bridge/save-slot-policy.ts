import type { SaveSlotSummary } from "./tauri-client.js";

/** Legacy valid slots may be explicitly continued for Rust's backup migration. */
export function canContinueSaveSlot(slot: SaveSlotSummary | undefined): boolean {
  return !!slot?.valid && typeof slot.currentHp === "number" && Number.isFinite(slot.currentHp) && slot.currentHp > 0;
}

/** Read-only legacy and corrupt slots can never be overwrite targets. */
export function canOverwriteSaveSlot(slot: SaveSlotSummary | undefined): boolean {
  return !!slot?.valid && !slot.readOnly && !slot.recoverable;
}

/** Read-only transaction backups use the existing explicit Continue entry. */
export function saveSlotAvailabilityLabel(slot: SaveSlotSummary): string {
  if (!canContinueSaveSlot(slot)) return `不可继续 · ${slot.errorCode || "无有效生命值"}`;
  if (slot.recoverable) return "可恢复 · 继续时安全恢复";
  return slot.readOnly ? "旧版只读档 · 续玩时安全迁移" : "可读写";
}
