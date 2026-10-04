import type { HudObjective } from "../Hud.js";

export class ObjectivePanel {
  private readonly list: HTMLElement;

  constructor(private readonly host: HTMLElement) {
    this.list = host.querySelector?.("ul") ?? host;
  }

  render(objectives: HudObjective[]): void {
    this.list.replaceChildren();
    for (const objective of objectives.slice(0, 3)) {
      const item = document.createElement("li");
      item.className = `hud-objective hud-objective-${objective.state || "unknown"}`;
      item.textContent = `${objective.state === "complete" ? "✓" : "◇"} ${objective.id}`;
      item.setAttribute("data-state", objective.state);
      this.list.append(item);
    }
    this.host.hidden = objectives.length === 0;
  }
}
