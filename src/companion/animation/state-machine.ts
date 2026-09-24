import { SeededRandom, hashSeed } from "./random";
import {
  BEHAVIOR_PRIORITY,
  type AnimationClipDefinition,
  type BehaviorId,
  type BehaviorRequest,
  type BehaviorSource,
  type CharacterManifest,
  type IdleProfile,
  type RenderFrame,
} from "./types";

interface Playback {
  behavior: BehaviorId;
  clipId: string;
  source: BehaviorSource;
  priority: number;
  frameCursor: number;
  frameElapsedMs: number;
  reducedElapsedMs: number;
  pending: BehaviorRequest | null;
}

export class CharacterAnimator {
  private baseBehavior: BehaviorId = "idle";
  private baseSource: BehaviorSource = "idle";
  private active: Playback;
  private reducedMotion = false;
  private readonly random: SeededRandom;
  private idleProfile: IdleProfile;
  private idleCountdownMs = 0;

  constructor(
    private readonly manifest: CharacterManifest,
    seed = hashSeed(manifest.id),
  ) {
    this.random = new SeededRandom(seed);
    this.idleProfile = this.normalizeIdleProfile(manifest.idleProfile);
    this.active = this.createPlayback("idle", "idle", null);
    this.resetIdleCountdown();
  }

  setReducedMotion(enabled: boolean): void {
    this.reducedMotion = enabled;
    this.active.frameCursor = 0;
    this.active.frameElapsedMs = 0;
    this.active.reducedElapsedMs = 0;
  }

  setIdleProfile(profile: IdleProfile): void {
    this.idleProfile = this.normalizeIdleProfile(profile);
    this.resetIdleCountdown();
  }

  setBaseBehavior(behavior: BehaviorId, source: BehaviorSource): void {
    if (behavior === this.baseBehavior && source === this.baseSource) return;

    const previousBase = this.baseBehavior;
    this.baseBehavior = behavior;
    this.baseSource = source;

    const currentTracksPreviousBase =
      this.active.behavior === previousBase &&
      (this.active.source === "idle" ||
        this.active.source === "system" ||
        this.active.source === "diagnostic" ||
        this.active.source === "critical");

    if (currentTracksPreviousBase) {
      this.requestBehavior({ behavior, source, force: true });
    } else if (BEHAVIOR_PRIORITY[source] >= this.active.priority) {
      this.requestBehavior({ behavior, source });
    }

    if (behavior === "idle") this.resetIdleCountdown();
  }

  requestBehavior(request: BehaviorRequest): boolean {
    const clipId = this.manifest.behaviors[request.behavior];
    if (!clipId || !this.manifest.clips[clipId]) return false;

    const priority = BEHAVIOR_PRIORITY[request.source];
    const currentClip = this.clip();

    if (!request.force) {
      if (
        request.behavior === this.active.behavior &&
        request.source === this.active.source &&
        clipId === this.active.clipId
      ) {
        return true;
      }

      if (currentClip.interruptible === false && priority <= this.active.priority) {
        return false;
      }

      if (priority < this.active.priority) return false;
    }

    const transitionId =
      this.active.behavior === request.behavior
        ? undefined
        : this.manifest.transitions[`${this.active.behavior}>${request.behavior}`];

    if (transitionId && this.manifest.clips[transitionId]) {
      this.active = {
        behavior: request.behavior,
        clipId: transitionId,
        source: request.source,
        priority,
        frameCursor: 0,
        frameElapsedMs: 0,
        reducedElapsedMs: 0,
        pending: request,
      };
    } else {
      this.active = this.createPlayback(request.behavior, request.source, null);
    }

    return true;
  }

  tick(deltaMs: number): RenderFrame {
    const safeDelta = Math.max(0, Math.min(deltaMs, 125));

    if (
      this.baseBehavior === "idle" &&
      this.active.behavior === "idle" &&
      this.active.clipId === this.manifest.behaviors.idle
    ) {
      this.idleCountdownMs -= safeDelta;
      if (this.idleCountdownMs <= 0) {
        const idleBehavior = this.chooseIdleBehavior();
        this.resetIdleCountdown();
        if (idleBehavior) {
          this.requestBehavior({
            behavior: idleBehavior,
            source: "idle",
          });
        }
      }
    }

    this.advance(safeDelta);
    return this.frame();
  }

  frame(): RenderFrame {
    const clip = this.clip();
    const frameId =
      this.reducedMotion && clip.reducedMotionFrame
        ? clip.reducedMotionFrame
        : clip.frames[this.active.frameCursor]?.frame ?? clip.frames[0].frame;
    const frame = this.manifest.frames[frameId];

    return {
      behavior: this.active.behavior,
      clipId: this.active.clipId,
      frameId,
      atlasIndex: frame.index,
      anchors: frame.anchors,
      source: this.active.source,
      priority: this.active.priority,
      reducedMotion: this.reducedMotion,
    };
  }

  currentBehavior(): BehaviorId {
    return this.active.behavior;
  }

  releaseSource(source: BehaviorSource): boolean {
    if (this.active.source !== source) return false;
    this.active = this.createPlayback(this.baseBehavior, this.baseSource, null);
    return true;
  }

  private advance(deltaMs: number): void {
    const clip = this.clip();

    if (this.reducedMotion) {
      if (!clip.loop) {
        this.active.reducedElapsedMs += deltaMs;
        if (
          this.active.reducedElapsedMs >=
          Math.max(1, clip.reducedMotionDurationMs ?? 180)
        ) {
          this.finishClip();
        }
      }
      return;
    }

    let remaining = deltaMs;
    while (remaining > 0) {
      const frame = clip.frames[this.active.frameCursor];
      const duration = Math.max(1, frame.durationMs ?? 100);
      const available = duration - this.active.frameElapsedMs;

      if (remaining < available) {
        this.active.frameElapsedMs += remaining;
        return;
      }

      remaining -= available;
      this.active.frameElapsedMs = 0;
      this.active.frameCursor += 1;

      if (this.active.frameCursor >= clip.frames.length) {
        if (clip.loop) {
          this.active.frameCursor = 0;
        } else {
          this.finishClip();
          return;
        }
      }
    }
  }

  private finishClip(): void {
    const pending = this.active.pending;
    if (pending) {
      this.active = this.createPlayback(pending.behavior, pending.source, null);
      return;
    }

    this.active = this.createPlayback(this.baseBehavior, this.baseSource, null);
  }

  private createPlayback(
    behavior: BehaviorId,
    source: BehaviorSource,
    pending: BehaviorRequest | null,
  ): Playback {
    const clipId =
      this.manifest.behaviors[behavior] ?? this.manifest.behaviors.idle;
    if (!clipId) {
      throw new Error(`Character "${this.manifest.id}" has no idle behavior`);
    }

    return {
      behavior,
      clipId,
      source,
      priority: BEHAVIOR_PRIORITY[source],
      frameCursor: 0,
      frameElapsedMs: 0,
      reducedElapsedMs: 0,
      pending,
    };
  }

  private clip(): AnimationClipDefinition {
    const clip = this.manifest.clips[this.active.clipId];
    if (!clip) {
      throw new Error(`Missing animation clip "${this.active.clipId}"`);
    }
    return clip;
  }

  private chooseIdleBehavior(): BehaviorId | null {
    const choices = this.idleProfile.choices;
    const totalWeight = choices.reduce((sum, choice) => sum + choice.weight, 0);
    if (totalWeight <= 0) return null;

    let value = this.random.next() * totalWeight;
    for (const choice of choices) {
      value -= choice.weight;
      if (value <= 0) return choice.behavior;
    }
    return choices.at(-1)?.behavior ?? null;
  }

  private resetIdleCountdown(): void {
    const idle = this.idleProfile;
    this.idleCountdownMs = this.random.between(idle.minDelayMs, idle.maxDelayMs);
  }

  private normalizeIdleProfile(profile: IdleProfile): IdleProfile {
    const fallback = this.manifest.idleProfile;
    const minimum = Number.isFinite(profile.minDelayMs)
      ? Math.max(500, profile.minDelayMs)
      : fallback.minDelayMs;
    const requestedMaximum = Number.isFinite(profile.maxDelayMs)
      ? Math.max(500, profile.maxDelayMs)
      : fallback.maxDelayMs;
    const choices = profile.choices.filter(
      (choice) =>
        Number.isFinite(choice.weight) &&
        choice.weight > 0 &&
        Boolean(this.manifest.behaviors[choice.behavior]),
    );

    return {
      minDelayMs: minimum,
      maxDelayMs: Math.max(minimum, requestedMaximum),
      choices: choices.length > 0 ? choices : [...fallback.choices],
    };
  }
}
