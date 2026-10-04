export interface PresentationSurfaceScheduler {
  request(callback: () => void): number;
  cancel(id: number): void;
}

/** No independent animation loop: draw once per committed frame or explicit invalidation. */
export class PresentationSurface {
  private alive = true;
  private contextLost = false;
  private generation = 0;
  private pending: number | null = null;
  private refreshRequested = false;

  constructor(
    private readonly draw: () => void,
    private readonly refresh: () => Promise<void>,
    private readonly onError: (error: unknown) => void,
    private readonly scheduler: PresentationSurfaceScheduler = {
      request: callback => requestAnimationFrame(callback),
      cancel: id => cancelAnimationFrame(id),
    },
  ) {}

  isAvailable(): boolean { return this.alive && !this.contextLost; }

  present(): boolean {
    if (!this.isAvailable()) return false;
    // A committed frame already includes any pending overlay clear.
    if (this.pending !== null && !this.refreshRequested) this.invalidate();
    const generation = this.generation;
    this.draw();
    // A synchronous context-loss/disposal callback during drawing is not a commit.
    return this.isAvailable() && generation === this.generation;
  }

  /** Overlay clearing needs one draw; resize/context restoration also reprojects. */
  request(refresh = false): void {
    if (!this.alive || this.contextLost) return;
    this.refreshRequested ||= refresh;
    if (this.pending !== null) return;
    const generation = this.generation;
    this.pending = this.scheduler.request(() => {
      if (!this.alive || this.contextLost || generation !== this.generation) return;
      this.pending = null;
      const refresh = this.refreshRequested;
      this.refreshRequested = false;
      if (refresh) {
        try {
          void this.refresh().catch(error => {
            if (this.alive && generation === this.generation) this.onError(error);
          });
        } catch (error) { this.onError(error); }
      } else {
        try { this.present(); } catch (error) { this.onError(error); }
      }
    });
  }

  lost(): void { this.contextLost = true; this.invalidate(); }
  restored(): void {
    if (!this.alive) return;
    this.contextLost = false;
    this.request(true);
  }
  invalidate(): void {
    this.generation++;
    if (this.pending !== null) this.scheduler.cancel(this.pending);
    this.pending = null;
    this.refreshRequested = false;
  }
  dispose(): void { this.alive = false; this.invalidate(); }
}
