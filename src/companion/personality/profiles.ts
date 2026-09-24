import type {
  InteractionLevel,
  Personality,
  SystemSnapshot,
} from "../../lib/types/domain";
import type { CharacterAnimator } from "../animation/state-machine";
import type {
  BehaviorId,
  CharacterManifest,
  IdleProfile,
} from "../animation/types";

export interface PersonalityChoice {
  id: Personality;
  name: string;
  description: string;
  traits: string;
}

export const PERSONALITY_CHOICES: PersonalityChoice[] = [
  {
    id: "CHILL",
    name: "Chill",
    description:
      "Calmer idle pacing, fewer ambient flourishes, and the quickest transition into rest.",
    traits: "Slow · sleepy · understated",
  },
  {
    id: "CURIOUS",
    name: "Curious",
    description:
      "Balanced pacing with more looking, inspecting, and curiosity-driven reactions.",
    traits: "Observant · balanced · inquisitive",
  },
  {
    id: "ENERGETIC",
    name: "Energetic",
    description:
      "Frequent expressive idles, maximum ambient life, and a longer wind-down before sleep.",
    traits: "Lively · expressive · playful",
  },
];

interface PersonalityProfile {
  idleDelayScale: number;
  ambientIntensity: number;
  idleEntryBehavior: BehaviorId;
  sleepDelayMs: number;
  pointerBehavior: BehaviorId;
  normalDragBehavior: BehaviorId;
  rapidDragBehavior: BehaviorId;
  typingDoneBehavior: BehaviorId;
  clickBurstBehavior: BehaviorId;
  bonusChoices: Array<{ behavior: BehaviorId; weight: number }>;
}

const PROFILES: Record<Personality, PersonalityProfile> = {
  CHILL: {
    idleDelayScale: 1.35,
    ambientIntensity: 0.55,
    idleEntryBehavior: "sleep",
    sleepDelayMs: 0,
    pointerBehavior: "blink",
    normalDragBehavior: "curious",
    rapidDragBehavior: "annoyed",
    typingDoneBehavior: "blink",
    clickBurstBehavior: "annoyed",
    bonusChoices: [
      { behavior: "look_left", weight: 0.4 },
      { behavior: "look_right", weight: 0.4 },
    ],
  },
  CURIOUS: {
    idleDelayScale: 0.9,
    ambientIntensity: 0.8,
    idleEntryBehavior: "curious",
    sleepDelayMs: 15_000,
    pointerBehavior: "curious",
    normalDragBehavior: "curious",
    rapidDragBehavior: "surprised",
    typingDoneBehavior: "curious",
    clickBurstBehavior: "surprised",
    bonusChoices: [
      { behavior: "curious", weight: 3 },
      { behavior: "look_left", weight: 1.5 },
      { behavior: "look_right", weight: 1.5 },
    ],
  },
  ENERGETIC: {
    idleDelayScale: 0.6,
    ambientIntensity: 1,
    idleEntryBehavior: "happy",
    sleepDelayMs: 35_000,
    pointerBehavior: "happy",
    normalDragBehavior: "happy",
    rapidDragBehavior: "surprised",
    typingDoneBehavior: "happy",
    clickBurstBehavior: "happy",
    bonusChoices: [
      { behavior: "happy", weight: 3 },
      { behavior: "surprised", weight: 1.5 },
      { behavior: "curious", weight: 1 },
    ],
  },
};

const INTERACTION_TUNING: Record<
  InteractionLevel,
  { idleDelayScale: number; ambientScale: number; sleepDelayScale: number }
> = {
  QUIET: {
    idleDelayScale: 1.35,
    ambientScale: 0.75,
    sleepDelayScale: 0.7,
  },
  NORMAL: {
    idleDelayScale: 1,
    ambientScale: 1,
    sleepDelayScale: 1,
  },
  PLAYFUL: {
    idleDelayScale: 0.75,
    ambientScale: 1,
    sleepDelayScale: 1.3,
  },
};

export function idleProfileForPersonality(
  manifest: CharacterManifest,
  personality: Personality,
  interactionLevel: InteractionLevel,
): IdleProfile {
  const profile = PROFILES[personality];
  const tuning = INTERACTION_TUNING[interactionLevel];
  const weights = new Map<string, number>();

  const add = (behavior: BehaviorId, weight: number): void => {
    if (!manifest.behaviors[behavior] || !Number.isFinite(weight) || weight <= 0) {
      return;
    }
    weights.set(behavior, (weights.get(behavior) ?? 0) + weight);
  };

  for (const choice of manifest.idleProfile.choices) {
    add(
      choice.behavior,
      choice.weight * behaviorWeight(personality, choice.behavior),
    );
  }

  for (const bonus of profile.bonusChoices) {
    add(bonus.behavior, bonus.weight);
  }

  const delayScale = profile.idleDelayScale * tuning.idleDelayScale;

  return {
    minDelayMs: Math.max(
      1_500,
      Math.round(manifest.idleProfile.minDelayMs * delayScale),
    ),
    maxDelayMs: Math.max(
      1_500,
      Math.round(manifest.idleProfile.maxDelayMs * delayScale),
    ),
    choices: [...weights.entries()].map(([behavior, weight]) => ({
      behavior,
      weight,
    })),
  };
}

export function ambientIntensityForPersonality(
  personality: Personality,
  interactionLevel: InteractionLevel,
): number {
  return clamp01(
    PROFILES[personality].ambientIntensity *
      INTERACTION_TUNING[interactionLevel].ambientScale,
  );
}

function behaviorWeight(
  personality: Personality,
  behavior: BehaviorId,
): number {
  const rare = behavior.startsWith("rare_");

  if (personality === "CHILL") {
    if (behavior === "blink") return 1.5;
    if (behavior === "curious") return 0.65;
    if (behavior === "happy" || behavior === "surprised") return 0.5;
    if (rare) return 0.65;
    return 0.85;
  }

  if (personality === "CURIOUS") {
    if (
      behavior === "curious" ||
      behavior === "look_left" ||
      behavior === "look_right"
    ) {
      return 1.8;
    }
    if (rare) return 1.15;
    return 1;
  }

  if (behavior === "blink") return 0.6;
  if (behavior === "happy" || behavior === "surprised") return 2;
  if (behavior === "curious") return 1.3;
  if (rare) return 1.8;
  return 1.2;
}

export class PersonalityDirector {
  private idle = false;
  private sleepCountdownMs: number | null = null;
  private previousCharging: boolean | null = null;
  private recentClicks: number[] = [];
  private lastClickBurstAt = -Infinity;

  constructor(
    private readonly personality: Personality,
    private readonly interactionLevel: InteractionLevel,
  ) {}

  idleProfile(manifest: CharacterManifest): IdleProfile {
    return idleProfileForPersonality(
      manifest,
      this.personality,
      this.interactionLevel,
    );
  }

  ambientIntensity(): number {
    return ambientIntensityForPersonality(
      this.personality,
      this.interactionLevel,
    );
  }

  onIdleStart(animator: CharacterAnimator): void {
    this.idle = true;
    const profile = PROFILES[this.personality];
    const tuning = INTERACTION_TUNING[this.interactionLevel];

    if (profile.sleepDelayMs <= 0) {
      this.sleepCountdownMs = null;
      animator.requestBehavior({
        behavior: "sleep",
        source: "personality",
      });
      return;
    }

    this.sleepCountdownMs = profile.sleepDelayMs * tuning.sleepDelayScale;
    animator.requestBehavior({
      behavior: profile.idleEntryBehavior,
      source: "personality",
    });
  }

  onIdleEnd(animator: CharacterAnimator): void {
    const wasIdle = this.idle;
    this.idle = false;
    this.sleepCountdownMs = null;
    animator.releaseSource("personality");

    if (wasIdle) {
      animator.requestBehavior({
        behavior: "wake",
        source: "input",
      });
    }
  }

  tick(deltaMs: number, animator: CharacterAnimator): void {
    if (!this.idle || this.sleepCountdownMs == null) return;

    this.sleepCountdownMs -= Math.min(Math.max(deltaMs, 0), 125);
    if (this.sleepCountdownMs > 0) return;

    this.sleepCountdownMs = null;
    animator.requestBehavior({
      behavior: "sleep",
      source: "personality",
    });
  }

  observeSnapshot(
    snapshot: SystemSnapshot,
    animator: CharacterAnimator,
  ): void {
    const charging = snapshot.battery?.charging ?? false;

    if (this.previousCharging === false && charging) {
      animator.requestBehavior({
        behavior: "happy",
        source: "personality",
      });
    }

    this.previousCharging = charging;
  }

  onPointerEnter(animator: CharacterAnimator): void {
    animator.requestBehavior({
      behavior: PROFILES[this.personality].pointerBehavior,
      source: "interaction",
    });
  }

  onDragComplete(durationMs: number, animator: CharacterAnimator): void {
    const profile = PROFILES[this.personality];
    animator.requestBehavior({
      behavior:
        durationMs < 800
          ? profile.rapidDragBehavior
          : profile.normalDragBehavior,
      source: "interaction",
    });
  }

  onFastTypingStop(animator: CharacterAnimator): void {
    animator.requestBehavior({
      behavior: PROFILES[this.personality].typingDoneBehavior,
      source: "personality",
    });
  }

  onMouseLeft(timestampEpochMs: number, animator: CharacterAnimator): void {
    this.recentClicks = this.recentClicks.filter(
      (timestamp) => timestampEpochMs - timestamp <= 900,
    );
    this.recentClicks.push(timestampEpochMs);

    if (
      this.recentClicks.length < 4 ||
      timestampEpochMs - this.lastClickBurstAt < 5_000
    ) {
      return;
    }

    this.lastClickBurstAt = timestampEpochMs;
    this.recentClicks = [];
    animator.requestBehavior({
      behavior: PROFILES[this.personality].clickBurstBehavior,
      source: "interaction",
    });
  }
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}
