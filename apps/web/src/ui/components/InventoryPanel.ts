import type { BuildAction, BuildProjection } from "../../protocol/BuildProjection.js";
export interface InventoryPanelState {
  build: BuildProjection | undefined;
  itemMessage: string;
  selectedItemId: string | null;
  pending: boolean;
  feedback: string;
  onSelect: (id: string) => void;
  onCommand: (action: BuildAction) => void;
}

export function renderInventoryPanel(host: HTMLElement, state: InventoryPanelState): void {
  const fragment = document.createDocumentFragment();
  const sections = ["物品", "装备", "描述"].map((title, index) => {
    const section = document.createElement("section");
    const id = ["inventory-items", "inventory-equipment", "inventory-description"][index]!;
    section.className = "structured-panel-column inventory-column";
    section.setAttribute("aria-labelledby", id);
    const heading = document.createElement("h3"); heading.id = id; heading.textContent = title;
    section.append(heading); fragment.append(section); return section;
  });
  const [items, equipment, details] = sections as [HTMLElement, HTMLElement, HTMLElement];
  const message = (host: HTMLElement, text: string) => {
    const p = document.createElement("p"); p.className = "structured-panel-empty"; p.textContent = text; host.append(p);
  };
  const build = state.build;
  if (!build) {
    message(items, state.itemMessage); message(equipment, "尚未收到权威装备投影，暂不显示装备。");
    message(details, "选择与描述数据尚未接入。");
  } else {
    const selected = build.items.find(item => item.itemId === state.selectedItemId) ?? build.items[0];
    if (!build.items.length) message(items, "背包为空。");
    for (const item of build.items) {
      const button = document.createElement("button"); button.type = "button";
      button.className = "inventory-item"; button.dataset.inventoryFocus = `item:${item.itemId}`;
      button.setAttribute("aria-pressed", String(selected?.itemId === item.itemId));
      button.textContent = `${item.label} × ${item.quantity}${item.equipped ? " · 已装备" : ""}`;
      button.addEventListener("click", () => state.onSelect(item.itemId)); items.append(button);
    }
    if (!build.equipment.length) message(equipment, "尚未装备物品。");
    for (const slot of build.equipment) {
      const row = document.createElement("div"); row.className = "inventory-slot";
      const label = document.createElement("p"); label.textContent = `${slot.label} · ${slot.slotId}`;
      const remove = document.createElement("button"); remove.type = "button"; remove.textContent = "卸下";
      remove.dataset.inventoryFocus = `remove:${slot.slotId}`; remove.disabled = state.pending;
      remove.setAttribute("aria-label", `卸下${slot.label}`);
      remove.addEventListener("click", () => state.onCommand({ kind: "unequip", slotId: slot.slotId, expectedItemId: slot.itemId }));
      row.append(label, remove); equipment.append(row);
    }
    if (selected) {
      const title = document.createElement("h4"); title.textContent = selected.label; details.append(title);
      message(details, selected.description); message(details, `数量：${selected.quantity} · 槽位：${selected.slotId}`);
      if (!selected.releaseEligible) message(details, "内部运行时物品 · 未获公开发行资格");
      const equip = document.createElement("button"); equip.type = "button";
      equip.dataset.inventoryFocus = `equip:${selected.itemId}`; equip.textContent = selected.equipped ? "已装备" : "装备";
      equip.disabled = state.pending || selected.equipped;
      equip.addEventListener("click", () => state.onCommand({ kind: "equip", itemId: selected.itemId,
        expectedItemId: build.equipment.find(slot => slot.slotId === selected.slotId)?.itemId ?? null }));
      details.append(equip);
    } else message(details, "选择物品后查看说明。");
  }
  const feedback = document.createElement("p"); feedback.className = "inventory-feedback";
  feedback.setAttribute("role", "status"); feedback.setAttribute("aria-live", "polite");
  feedback.textContent = state.feedback; details.append(feedback);
  host.setAttribute("aria-busy", String(state.pending)); host.replaceChildren(fragment);
}
