import { describe, expect, it } from "vitest";
import type { SystemSnapshot } from "../../lib/types/domain";
import type {
  BehaviorRequest,
  CharacterManifest,
} from "../animation/types";
import {
  PersonalityDirector,
  ambientIntensityForPersonality,
  idleProfileForPersonality,
} from "./profiles";

function manifest(): CharacterManifest {
  const anchors = {
    head: { x: 5, y: 2 },
    face: { x: 5, y: 3 },
    body: { x: 5, y: 5 },
    back: { x: 5, y: 5 },
    left_hand: { x: 3, y: 5 },
    right_hand: { x: 7, y: 5 },
    ground: { x: 5, y: 9 },
  };

  return {
    schemaVersion: 1,
    id: "test",
    name: "Test",
    nativeSize: 8,
    animationCanvas: 10,
    preview: "/test.png",
    atlas: {
      src: "/test.png",
      width: 10,
      height: 10,
      frameWidth: 10,
      frameHeight: 10,
      columns: 1,
    },
    paletteSlots: { primary: "#112233" },
    defaultPalette: "default",
    palettes: [
      {
        id: "default",
        name: "Default",
        colors: { primary: "#112233" },
      },
    ],
    frames: { idle: { index: 0, anchors } },
    clips: {
      idle: { frames: [{ frame: "idle" }], loop: true },
    },
    behaviors: {
      idle: "idle",
      blink: "idle",
      curious: "idle",
      look_left: "idle",
      look_right: "idle",
      happy: "idle",
      surprised: "idle",
      annoyed: "idle",
      sleep: "idle",
      wake: "idle",
      rare_a: "idle",
    },
    transitions: {},
    idleProfile: {
      minDelayMs: 5_000,
      maxDelayMs: 10_000,
      choices: [
        { behavior: "blink", weight: 5 },
        { behavior: "curious", weight: 2 },
        { behavior: "rare_a", weight: 1 },
      ],
    },
    anchors: Object.keys(anchors),
    status: "production",
  };
}

class FakeAnimator {
  requests: BehaviorRequest[] = [];
  releases: string[] = [];

  requestBehavior(request: BehaviorRequest): boolean {
    this.requests.push(request);
    return true;
  }

  releaseSource(source: string): boolean {
    this.releases.push(source);
    return true;
  }
}

function snapshot(charging: boolean): SystemSnapshot {
  return {
    timestamp_epoch_ms: 1,
    overall_status: "CALM",
    cpu: { value: 20, unit: "%", state: "NORMAL", available: null, available_unit: null },
    memory: { value: 40, unit: "%", state: "NORMAL", available: 8, available_unit: "GB" },
    storage: { value: 40, unit: "%", state: "NORMAL", available: 200, available_unit: "GB" },
    battery: { percent: 80, charging, state: "NORMAL" },
    network: { download_mbps: 0, upload_mbps: 0 },
    thermal: null,
    primary_issue: null,
    secondary_issue_count: 0,
  };
}

describe("personality profiles", () => {
  it("orders idle cadence from chill to curious to energetic", () => {
    const source = manifest();
    const chill = idleProfileForPersonality(source, "CHILL", "NORMAL");
    const curious = idleProfileForPersonality(source, "CURIOUS", "NORMAL");
    const energetic = idleProfileForPersonality(source, "ENERGETIC", "NORMAL");

    expect(chill.minDelayMs).toBeGreaterThan(curious.minDelayMs);
    expect(curious.minDelayMs).toBeGreaterThan(energetic.minDelayMs);
    expect(chill.choices.some((choice) => choice.behavior === "rare_a")).toBe(true);
  });

  it("keeps quiet mode calmer than playful mode", () => {
    const source = manifest();
    const quiet = idleProfileForPersonality(source, "CURIOUS", "QUIET");
    const playful = idleProfileForPersonality(source, "CURIOUS", "PLAYFUL");

    expect(quiet.minDelayMs).toBeGreaterThan(playful.minDelayMs);
    expect(
      ambientIntensityForPersonality("ENERGETIC", "QUIET"),
    ).toBeLessThan(
      ambientIntensityForPersonality("ENERGETIC", "PLAYFUL"),
    );
  });

  it("gives chill immediate sleep and curious a delayed wind-down", () => {
    const chillAnimator = new FakeAnimator();
    const chill = new PersonalityDirector("CHILL", "NORMAL");
    chill.onIdleStart(chillAnimator as any);
    expect(chillAnimator.requests.at(-1)?.behavior).toBe("sleep");

    const curiousAnimator = new FakeAnimator();
    const curious = new PersonalityDirector("CURIOUS", "NORMAL");
    curious.onIdleStart(curiousAnimator as any);
    expect(curiousAnimator.requests.at(-1)?.behavior).toBe("curious");

    for (let index = 0; index < 121; index += 1) {
      curious.tick(125, curiousAnimator as any);
    }
    expect(curiousAnimator.requests.at(-1)?.behavior).toBe("sleep");
  });

  it("reacts to a rapid click burst according to personality", () => {
    const animator = new FakeAnimator();
    const director = new PersonalityDirector("CURIOUS", "NORMAL");

    for (const timestamp of [100, 250, 400, 550]) {
      director.onMouseLeft(timestamp, animator as any);
    }

    expect(animator.requests.at(-1)).toMatchObject({
      behavior: "surprised",
      source: "interaction",
    });
  });

  it("celebrates only a transition into charging", () => {
    const animator = new FakeAnimator();
    const director = new PersonalityDirector("ENERGETIC", "NORMAL");

    director.observeSnapshot(snapshot(true), animator as any);
    expect(animator.requests).toHaveLength(0);

    director.observeSnapshot(snapshot(false), animator as any);
    director.observeSnapshot(snapshot(true), animator as any);

    expect(animator.requests.at(-1)).toMatchObject({
      behavior: "happy",
      source: "personality",
    });
  });
});
