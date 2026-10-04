/** Rust-owned Build data. Release eligibility is informational, not runtime ownership. */
export interface InventoryItemProjection {
  itemId: string; quantity: number; slotId: string; label: string; description: string;
  equipped: boolean; releaseEligible: boolean;
}
export interface EquipmentProjection { slotId: string; itemId: string; label: string }
export interface BuildProjection {
  schemaVersion: 1; revision: number; items: InventoryItemProjection[]; equipment: EquipmentProjection[];
}
export type BuildAction = { kind: "equip"; itemId: string; expectedItemId: string | null }
  | { kind: "unequip"; slotId: string; expectedItemId: string };
const id = (value: unknown): value is string => typeof value === "string" && /^[a-z0-9._:-]{1,128}$/.test(value);
const text = (value: unknown, max: number): value is string => typeof value === "string" &&
  value.trim().length > 0 && new TextEncoder().encode(value).length <= max && !/[\u0000-\u001f\u007f-\u009f]/u.test(value);
function record(value: unknown): value is Record<string, unknown> { return !!value && typeof value === "object" && !Array.isArray(value); }
function keys(value: Record<string, unknown>, expected: string[]): boolean {
  return Object.keys(value).length === expected.length && expected.every(key => Object.hasOwn(value, key));
}
export function assertBuildProjection(value: unknown): BuildProjection {
  if (!record(value) || !keys(value, ["schemaVersion", "revision", "items", "equipment"]) || value.schemaVersion !== 1 ||
      !Number.isSafeInteger(value.revision) || (value.revision as number) < 0 ||
      !Array.isArray(value.items) || value.items.length > 512 || !Array.isArray(value.equipment) || value.equipment.length > 32) {
    throw new Error("E_BUILD_PROJECTION_INVALID");
  }
  const items = new Map<string, InventoryItemProjection>();
  for (const item of value.items) {
    if (!record(item) || !keys(item, ["itemId", "quantity", "slotId", "label", "description", "equipped", "releaseEligible"]) ||
        !id(item.itemId) || !id(item.slotId) || !Number.isSafeInteger(item.quantity) || (item.quantity as number) < 1 ||
        (item.quantity as number) > 4_294_967_295 || !text(item.label, 192) || !text(item.description, 1024) ||
        typeof item.equipped !== "boolean" || typeof item.releaseEligible !== "boolean" || items.has(item.itemId)) {
      throw new Error("E_BUILD_PROJECTION_INVALID");
    }
    items.set(item.itemId, item as unknown as InventoryItemProjection);
  }
  const equipped = new Set<string>(); const slots = new Set<string>();
  for (const item of value.equipment) {
    if (!record(item) || !keys(item, ["slotId", "itemId", "label"]) || !id(item.itemId) || !id(item.slotId) ||
        !text(item.label, 192) || slots.has(item.slotId) || equipped.has(item.itemId)) throw new Error("E_BUILD_PROJECTION_INVALID");
    const owned = items.get(item.itemId);
    if (!owned || !owned.equipped || owned.slotId !== item.slotId || owned.label !== item.label) throw new Error("E_BUILD_PROJECTION_INVALID");
    slots.add(item.slotId); equipped.add(item.itemId);
  }
  if ([...items.values()].some(item => item.equipped !== equipped.has(item.itemId))) throw new Error("E_BUILD_PROJECTION_INVALID");
  return value as unknown as BuildProjection;
}
export function assertBuildAction(value: unknown): BuildAction {
  if (!record(value) || (value.kind !== "equip" && value.kind !== "unequip")) throw new Error("E_BUILD_ACTION_INVALID");
  const valid = value.kind === "equip"
    ? keys(value, ["kind", "itemId", "expectedItemId"]) && id(value.itemId) && (value.expectedItemId === null || id(value.expectedItemId))
    : keys(value, ["kind", "slotId", "expectedItemId"]) && id(value.slotId) && id(value.expectedItemId);
  if (!valid) throw new Error("E_BUILD_ACTION_INVALID");
  return value as unknown as BuildAction;
}
