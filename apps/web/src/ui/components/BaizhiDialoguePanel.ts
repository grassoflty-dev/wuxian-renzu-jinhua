import { BAIZHI_RESULTS, type BaizhiDialogue } from "../../game/BaizhiDialogue.js";

/** Existing HTML/CSS modal; no portrait image or asset-admission fallback. */
export class BaizhiDialoguePanel {
  private focusKey = "";
  private previousFocus: HTMLElement | null = null;
  constructor(private readonly element: HTMLElement, private readonly dialogue: BaizhiDialogue) {
    element.innerHTML = `<div class="baizhi-dialogue-frame"><div class="baizhi-portrait" aria-label="白芷肖像开发占位">肖像待补·开发占位</div><div class="baizhi-dialogue-content"><h2 id="baizhi-title" tabindex="-1">白芷</h2><div id="baizhi-lines"></div><p id="baizhi-status" role="status" aria-live="polite"></p><div class="baizhi-actions"></div></div></div>`;
  }
  onKeyDown(event: KeyboardEvent): void {
    if (!this.dialogue.visible) return;
    if (event.key === "Escape" || event.key.toLowerCase() === "f") {
      event.preventDefault(); event.stopImmediatePropagation();
      if (event.key === "Escape" && !event.repeat) this.dialogue.escape();
    } else if (event.key === "Tab") {
      const buttons = Array.from(this.element.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"));
      const first = buttons[0], last = buttons.at(-1);
      if (!first) { event.preventDefault(); this.element.querySelector<HTMLElement>("h2")?.focus(); }
      else if (event.shiftKey && (document.activeElement === first || !this.element.contains(document.activeElement))) {
        event.preventDefault(); last?.focus();
      } else if (!event.shiftKey && (document.activeElement === last || !this.element.contains(document.activeElement))) {
        event.preventDefault(); first.focus();
      }
    }
  }
  render(): void {
    const d = this.dialogue, wasHidden = this.element.hidden;
    this.element.hidden = !d.visible;
    if (!d.visible) {
      this.focusKey = "";
      if (!wasHidden && this.previousFocus?.isConnected) this.previousFocus.focus();
      this.previousFocus = null; return;
    }
    if (wasHidden) this.previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    this.element.setAttribute("aria-busy", String(d.busy));
    this.element.querySelector<HTMLElement>("#baizhi-title")!.textContent = d.page === "choices" ? "你的决定" : "白芷";
    const lines = this.element.querySelector<HTMLElement>("#baizhi-lines")!;
    lines.replaceChildren();
    const addLine = (text: string) => { const line = document.createElement("p"); line.textContent = text; lines.append(line); };
    if (d.page === "dialogue") {
      addLine("别开那扇门。先听我说。");
      addLine("我不知道这里还能撑多久，但我知道里面的东西不该被随便释放。");
    } else if (d.page === "confirmation") addLine(d.choice === "taken" ? "你决定帮助白芷撤离。" : "你决定不介入白芷的处境。");
    else if (d.page === "result") { addLine(`岑遥：${BAIZHI_RESULTS[d.choice].player}`); addLine(`白芷：${BAIZHI_RESULTS[d.choice].baizhi}`); }
    this.element.querySelector<HTMLElement>("#baizhi-status")!.textContent = d.message;
    const actions = this.element.querySelector<HTMLElement>(".baizhi-actions")!;
    actions.replaceChildren();
    let defaultButton: HTMLButtonElement | undefined;
    const button = (id: string, label: string, action: () => void, preferred = false) => {
      const element = document.createElement("button"); element.type = "button"; element.dataset.baizhiAction = id;
      element.textContent = label; element.disabled = d.busy || !d.ready; element.addEventListener("click", action); actions.append(element);
      if (preferred) defaultButton = element;
    };
    if (d.page === "dialogue") { button("next", "继续", () => d.next(), true); button("close", "关闭", () => void d.close()); }
    else if (d.page === "choices") {
      button("taken", "帮助她撤离", () => d.select("taken")); button("left", "放弃帮助", () => d.select("left"));
      button("unresolved", "暂不决定", () => d.select("unresolved"), true);
    } else if (d.page === "confirmation") {
      button("confirm", "确认", () => void d.confirm()); button("back", "返回", () => d.back(), true);
    } else if (d.page === "result") button("close", "关闭", () => void d.close(), true);
    const key = `${d.page}:${d.busy}:${d.ready}`;
    if (key !== this.focusKey || !this.element.contains(document.activeElement)) {
      this.focusKey = key;
      (defaultButton && !defaultButton.disabled ? defaultButton : this.element.querySelector<HTMLElement>("h2"))?.focus();
    }
  }
}
