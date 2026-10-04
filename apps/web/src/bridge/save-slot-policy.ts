import type { SaveSlotSummary } from "./tauri-client.js";

/** Legacy valid slots may be explicitly continued for Rust's backup migration. */
export function canContinueSaveSlot(slot: SaveSlotSummary | undefined): boolean {
  return !!slot?.valid && typeof slot.currentHp === "number" && Number.isFinite(slot.currentHp) && slot.currentHp > 0;
}

/** Read-only legacy and corrupt slots can never be overwrite targets. */
export function canOverwriteSaveSlot(slot: SaveSlotSummary | undefined): boolean {
  return !!slot?.valid && !slot.readOnly;
}
