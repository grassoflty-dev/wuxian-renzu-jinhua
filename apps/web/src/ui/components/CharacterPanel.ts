import type { WorldSnapshotV3 } from "../../protocol/types.js";

const LABELS: Record<string, string> = {
  "information.local_map_i": "局部地图",
  "information.enemy_vitals_basic": "敌人生命信息",
  "perception.rear_view_i": "后方视野",
  "body.regeneration_i": "再生能力",
};

export function renderCharacterPanel(host: HTMLElement, snapshot: WorldSnapshotV3, portraitUrl?: string): void {
  host.replaceChildren();
  const identity = document.createElement("section");
  identity.className = "character-identity structured-panel-column";
  const portrait = document.createElement("div");
  portrait.className = "character-portrait";
  portrait.setAttribute("aria-label", "岑遥头像");
  if (portraitUrl) {
    const image = document.createElement("img");
    image.src = portraitUrl;
    image.alt = "岑遥头像";
    image.decoding = "async";
    portrait.append(image);
  } else {
    portrait.textContent = "岑";
    portrait.setAttribute("aria-label", "岑遥头像占位：portrait.cenyao.v1 尚未加载");
  }
  const name = document.createElement("h3");
  name.textContent = "岑遥";
  const note = document.createElement("p");
  note.className = "structured-panel-empty";
  note.textContent = portraitUrl ? "portrait.cenyao.v1 · 已通过注册表接入" : "portrait.cenyao.v1 · 资产未加载，显示占位";
  identity.append(portrait, name, note);

  const state = document.createElement("section");
  state.className = "character-state structured-panel-column";
  state.setAttribute("aria-label", "角色状态");
  const heading = document.createElement("h3");
  heading.textContent = "当前状态";
  const list = document.createElement("dl");
  const rows = [
    ["HP", `${Math.round(snapshot.player.currentHp)} / ${Math.round(snapshot.player.maxHp)}`],
    ["能量", `${Math.round(snapshot.player.currentEnergy)} / ${Math.round(snapshot.player.maxEnergy)}`],
    ["动作", snapshot.player.actionState || "状态未报告"],
    ["身体状态", "权威身体状态未提供"],
  ] as const;
  for (const [label, value] of rows) {
    const line = document.createElement("div");
    line.className = "structured-state-row";
    const term = document.createElement("dt");
    term.textContent = label;
    const detail = document.createElement("dd");
    detail.textContent = value;
    line.append(term, detail);
    list.append(line);
  }
  state.append(heading, list);

  const capabilities = document.createElement("section");
  capabilities.className = "character-capabilities structured-panel-column";
  const capabilityHeading = document.createElement("h3");
  capabilityHeading.textContent = "已授予能力";
  capabilities.append(capabilityHeading);
  const granted = Array.isArray(snapshot.capabilities?.items)
    ? snapshot.capabilities.items.filter(item => item.granted === true)
    : [];
  const capabilityText = document.createElement("p");
  capabilityText.className = "structured-panel-empty";
  capabilityText.textContent = granted.length
    ? granted.map(item => `${LABELS[item.capabilityId] ?? item.capabilityId}${item.selected ? " · 已选中" : ""}`).join("、")
    : "尚无已授予能力";
  capabilities.append(capabilityText);

  const equipment = document.createElement("section");
  equipment.className = "character-equipment structured-panel-column";
  const equipmentHeading = document.createElement("h3");
  equipmentHeading.textContent = "装备";
  const equipmentText = document.createElement("p");
  equipmentText.className = "structured-panel-empty";
  equipmentText.textContent = snapshot.build
    ? snapshot.build.equipment.length ? snapshot.build.equipment.map(item => `${item.label} · ${item.slotId}`).join("、") : "尚未装备物品。"
    : "权威装备投影尚未接入，装备不可用。";
  equipment.append(equipmentHeading, equipmentText);
  host.append(identity, state, capabilities, equipment);
}
