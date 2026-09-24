import { describe, expect, it } from "vitest";
import { CharacterAnimator } from "./state-machine";
import type { CharacterManifest } from "./types";

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
    preview: "/test-preview.png",
    atlas: {
      src: "/test.png",
      width: 40,
      height: 10,
      frameWidth: 10,
      frameHeight: 10,
      columns: 4,
    },
    paletteSlots: {
      primary: "#112233",
      accent: "#445566",
    },
    defaultPalette: "default",
    palettes: [
      {
        id: "default",
        name: "Default",
        colors: {
          primary: "#112233",
          accent: "#445566",
        },
      },
    ],
    frames: {
      idle: { index: 0, anchors },
      happy: { index: 1, anchors },
      alert: { index: 2, anchors },
      transition: { index: 3, anchors },
    },
    clips: {
      idle: {
        frames: [{ frame: "idle", durationMs: 100 }],
        loop: true,
        reducedMotionFrame: "idle",
      },
      happy: {
        frames: [{ frame: "happy", durationMs: 50 }],
        loop: false,
        reducedMotionFrame: "happy",
        reducedMotionDurationMs: 50,
      },
      alert: {
        frames: [{ frame: "alert", durationMs: 100 }],
        loop: true,
        reducedMotionFrame: "alert",
      },
      enter_alert: {
        frames: [{ frame: "transition", durationMs: 40 }],
        loop: false,
        reducedMotionFrame: "transition",
        reducedMotionDurationMs: 20,
      },
    },
    behaviors: {
      idle: "idle",
      happy: "happy",
      needs_attention: "alert",
    },
    transitions: {
      "idle>needs_attention": "enter_alert",
    },
    idleProfile: {
      minDelayMs: 10_000,
      maxDelayMs: 10_000,
      choices: [{ behavior: "happy", weight: 1 }],
    },
    anchors: Object.keys(anchors),
    status: "placeholder",
  };
}

describe("CharacterAnimator", () => {
  it("prevents lower-priority idle behavior from interrupting interaction", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    expect(animator.requestBehavior({ behavior: "happy", source: "interaction" })).toBe(true);
    expect(animator.requestBehavior({ behavior: "happy", source: "idle" })).toBe(false);
  });

  it("allows critical behavior to interrupt lower-priority behavior", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.requestBehavior({ behavior: "happy", source: "interaction" });
    expect(
      animator.requestBehavior({
        behavior: "needs_attention",
        source: "critical",
      }),
    ).toBe(true);
    expect(animator.currentBehavior()).toBe("needs_attention");
  });

  it("plays a transition clip before the requested target", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.requestBehavior({
      behavior: "needs_attention",
      source: "critical",
    });

    expect(animator.frame().clipId).toBe("enter_alert");
    animator.tick(45);
    expect(animator.frame().clipId).toBe("alert");
  });

  it("returns to the base behavior after a one-shot reaction", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.requestBehavior({ behavior: "happy", source: "interaction" });
    animator.tick(60);
    expect(animator.currentBehavior()).toBe("idle");
  });

  it("does not restart an unchanged base state", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.requestBehavior({ behavior: "happy", source: "interaction" });
    animator.setBaseBehavior("idle", "idle");
    expect(animator.currentBehavior()).toBe("happy");
  });

  it("allows a persistent critical base state to downgrade after recovery", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.setBaseBehavior("needs_attention", "critical");
    animator.tick(50);
    expect(animator.currentBehavior()).toBe("needs_attention");

    animator.setBaseBehavior("idle", "idle");
    expect(animator.currentBehavior()).toBe("idle");
  });

  it("can release a looping transient source back to the base state", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.requestBehavior({ behavior: "happy", source: "input" });
    expect(animator.releaseSource("input")).toBe(true);
    expect(animator.currentBehavior()).toBe("idle");
    expect(animator.releaseSource("input")).toBe(false);
  });

  it("uses reduced-motion frames while preserving completion semantics", () => {
    const animator = new CharacterAnimator(manifest(), 1);
    animator.setReducedMotion(true);
    animator.requestBehavior({ behavior: "happy", source: "interaction" });

    expect(animator.frame().frameId).toBe("happy");
    animator.tick(60);
    expect(animator.currentBehavior()).toBe("idle");
  });
});
