const SKILLS = [
  ["Q", "环形扫描"], ["E", "弧盾防御"], ["R", "线性穿透"], ["Shift", "短距冲刺"],
] as const;

export class SkillBar {
  constructor(private readonly host: HTMLElement) {
    this.render();
  }

  render(): void {
    this.host.replaceChildren(...SKILLS.map(([key, label]) => {
      const item = document.createElement("span");
      item.className = "hud-skill";
      item.title = label;
      const keyElement = document.createElement("kbd");
      keyElement.textContent = key;
      keyElement.setAttribute("aria-label", `${key}：${label}`);
      item.append(keyElement);
      return item;
    }));
  }
}
