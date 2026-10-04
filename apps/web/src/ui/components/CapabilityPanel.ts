import type { CapabilityItemProjection, WorldSnapshotV3 } from "../../protocol/types.js";
import { acousticMappingStatus } from "../../game/CapabilityTerminal.js";

import { CAPABILITY_CATEGORIES, capabilityLabel, knownCapabilityRows } from "../CapabilityPresentation.js";

export function capabilityGroups(snapshot: WorldSnapshotV3): Array<{ prefix: string; label: string; items: CapabilityItemProjection[] }> {
  const granted = knownCapabilityRows(snapshot).filter(item => item.granted === true);
  return CAPABILITY_CATEGORIES.map(([prefix, label]) => ({
    prefix,
    label,
    items: granted.filter(item => item.capabilityId.startsWith(`${prefix}.`)),
  }));
}

export function renderCapabilityPanel(host: HTMLElement, snapshot: WorldSnapshotV3): void {
  const groups = capabilityGroups(snapshot);
  const fragment = document.createDocumentFragment();
  const list = document.createElement("section");
  list.className = "capability-groups structured-panel-column";
  const heading = document.createElement("h3");
  heading.textContent = "已授予能力";
  list.append(heading);
  const rows = document.createElement("div");
  rows.className = "capability-groups-list";
  for (const { prefix, label: groupName, items: groupItems } of groups) {
    const group = document.createElement("section");
    group.className = "capability-group";
    const groupHeading = document.createElement("h4");
    groupHeading.textContent = `${groupName} · ${prefix.toUpperCase()}`;
    group.append(groupHeading);
    if (groupItems.length === 0) {
        const empty = document.createElement("p");
        empty.className = "structured-panel-empty";
        empty.textContent = "暂无已授予能力";
        group.append(empty);
    } else {
      for (const item of groupItems) {
        const row = document.createElement("div");
        row.className = "capability-row";
        const label = document.createElement("strong");
        label.textContent = capabilityLabel(item.capabilityId);
        const status = document.createElement("span");
        status.textContent = item.selected ? "已获得 · 已选中" : "已获得 · 未选中";
        row.append(label, status);
        group.append(row);
      }
    }
    rows.append(group);
  }
  list.append(rows);
  const acousticStatus = acousticMappingStatus(snapshot);
  if (!acousticStatus.startsWith("已获得")) {
    const acoustic = document.createElement("p");
    acoustic.className = "structured-panel-note";
    acoustic.textContent = `声学映射：${acousticStatus}`;
    list.append(acoustic);
  }
  const unavailable = document.createElement("p");
  unavailable.className = "structured-panel-note";
  unavailable.textContent = "能力树节点关系与成本数据尚未接入；当前面板只读，不提供解锁操作。";
  list.append(unavailable);
  fragment.append(list);
  host.replaceChildren(fragment);
}
