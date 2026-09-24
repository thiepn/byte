import { describe, expect, it } from "vitest";
import { validateCharacterManifest } from "./manifest";

function fixture(): unknown {
  const anchors = {
    head: { x: 4, y: 1 },
    face: { x: 4, y: 2 },
    body: { x: 4, y: 4 },
    back: { x: 4, y: 4 },
    left_hand: { x: 2, y: 4 },
    right_hand: { x: 6, y: 4 },
    ground: { x: 4, y: 7 },
  };

  return {
    schemaVersion: 1,
    id: "test",
    name: "Test",
    nativeSize: 8,
    animationCanvas: 8,
    atlas: {
      src: "/test.png",
      width: 8,
      height: 8,
      frameWidth: 8,
      frameHeight: 8,
      columns: 1,
    },
    frames: {
      idle: { index: 0, anchors },
    },
    clips: {
      idle: {
        frames: [{ frame: "idle", durationMs: 100 }],
        loop: true,
        reducedMotionFrame: "idle",
      },
    },
    behaviors: { idle: "idle" },
    transitions: {},
    idleProfile: {
      minDelayMs: 1000,
      maxDelayMs: 2000,
      choices: [{ behavior: "idle", weight: 1 }],
    },
    anchors: Object.keys(anchors),
    status: "placeholder",
  };
}

describe("validateCharacterManifest", () => {
  it("accepts a complete manifest", () => {
    expect(validateCharacterManifest(fixture()).id).toBe("test");
  });

  it("rejects behavior references to missing clips", () => {
    const value = fixture() as Record<string, unknown>;
    value.behaviors = { idle: "missing" };
    expect(() => validateCharacterManifest(value)).toThrow(/missing clip/);
  });

  it("rejects frames outside the atlas", () => {
    const value = fixture() as any;
    value.frames.idle.index = 2;
    expect(() => validateCharacterManifest(value)).toThrow(/outside the atlas/);
  });
});
