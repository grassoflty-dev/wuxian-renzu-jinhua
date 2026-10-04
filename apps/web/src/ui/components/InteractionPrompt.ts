export class InteractionPrompt {
  constructor(private readonly host: HTMLElement) {}

  render(text: string): void {
    this.host.textContent = text;
    this.host.hidden = text.length === 0;
  }
}
