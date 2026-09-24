export interface AnimationTick {
  now: number;
  deltaMs: number;
}

export class FramePacer {
  private readonly intervalMs: number;
  private lastTickAt: number | null = null;

  constructor(
    framesPerSecond = 12,
    private readonly maxDeltaMs = 125,
  ) {
    this.intervalMs = 1000 / framesPerSecond;
  }

  step(now: number): AnimationTick | null {
    if (this.lastTickAt == null) {
      this.lastTickAt = now;
      return { now, deltaMs: 0 };
    }

    const elapsed = now - this.lastTickAt;
    if (elapsed < this.intervalMs) return null;

    this.lastTickAt = now;
    return {
      now,
      deltaMs: Math.min(Math.max(elapsed, 0), this.maxDeltaMs),
    };
  }

  reset(now: number | null = null): void {
    this.lastTickAt = now;
  }
}

export class AnimationScheduler {
  private readonly listeners = new Set<(tick: AnimationTick) => void>();
  private readonly pacer = new FramePacer(12, 125);
  private animationFrame: number | null = null;
  private visibilityBound = false;

  subscribe(listener: (tick: AnimationTick) => void): () => void {
    this.listeners.add(listener);
    this.bindVisibility();
    this.ensureRunning();

    return () => {
      this.listeners.delete(listener);
      if (this.listeners.size === 0) this.stop();
    };
  }

  private bindVisibility(): void {
    if (this.visibilityBound || typeof document === "undefined") return;
    this.visibilityBound = true;
    document.addEventListener("visibilitychange", () => {
      if (document.hidden) {
        this.stop();
      } else {
        this.pacer.reset(performance.now());
        this.ensureRunning();
      }
    });
  }

  private ensureRunning(): void {
    if (
      this.animationFrame != null ||
      this.listeners.size === 0 ||
      typeof requestAnimationFrame === "undefined" ||
      (typeof document !== "undefined" && document.hidden)
    ) {
      return;
    }

    this.animationFrame = requestAnimationFrame(this.onFrame);
  }

  private readonly onFrame = (now: number): void => {
    this.animationFrame = null;
    const tick = this.pacer.step(now);
    if (tick) {
      for (const listener of this.listeners) listener(tick);
    }
    this.ensureRunning();
  };

  private stop(): void {
    if (this.animationFrame != null && typeof cancelAnimationFrame !== "undefined") {
      cancelAnimationFrame(this.animationFrame);
    }
    this.animationFrame = null;
    this.pacer.reset();
  }
}

export const animationScheduler = new AnimationScheduler();
