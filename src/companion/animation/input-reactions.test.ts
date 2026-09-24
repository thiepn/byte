import { describe, expect, it } from "vitest";
import { applyInputReaction } from "./input-reactions";
import type { InputReactionEvent } from "../../lib/types/input";

class FakeAnimator {
  requests: Array<{ behavior: string; source: string }> = [];
  releases: string[] = [];
  behavior = "idle";

  requestBehavior(request: { behavior: string; source: string }): boolean {
    this.requests.push(request);
    this.behavior = request.behavior;
    return true;
  }

  currentBehavior(): string {
    return this.behavior;
  }

  releaseSource(source: string): boolean {
    this.releases.push(source);
    return true;
  }
}

function event(kind: InputReactionEvent["kind"]): InputReactionEvent {
  return {
    kind,
    timestamp_epoch_ms: 1,
    keys_per_second: null,
  };
}

describe("applyInputReaction", () => {
  it("alternates semantic typing sides from native events", () => {
    const animator = new FakeAnimator();
    applyInputReaction(animator as any, event("TYPING_TAP_LEFT"));
    applyInputReaction(animator as any, event("TYPING_TAP_RIGHT"));

    expect(animator.requests.map((request) => request.behavior)).toEqual([
      "typing_left",
      "typing_right",
    ]);
  });

  it("starts and releases the fast typing loop", () => {
    const animator = new FakeAnimator();
    applyInputReaction(animator as any, event("TYPING_FAST_START"));
    applyInputReaction(animator as any, event("TYPING_FAST_STOP"));

    expect(animator.requests[0]).toEqual({
      behavior: "typing_fast",
      source: "input",
    });
    expect(animator.releases).toEqual(["input"]);
  });

  it("does not let mouse reactions interrupt fast typing", () => {
    const animator = new FakeAnimator();
    applyInputReaction(animator as any, event("TYPING_FAST_START"));
    applyInputReaction(animator as any, event("MOUSE_LEFT"));

    expect(animator.requests.map((request) => request.behavior)).toEqual([
      "typing_fast",
    ]);
  });

  it("sleeps on idle and wakes on resumed activity", () => {
    const animator = new FakeAnimator();
    applyInputReaction(animator as any, event("IDLE_START"));
    applyInputReaction(animator as any, event("IDLE_END"));

    expect(animator.requests[0]).toEqual({
      behavior: "sleep",
      source: "personality",
    });
    expect(animator.releases).toContain("personality");
    expect(animator.requests.at(-1)).toEqual({
      behavior: "wake",
      source: "input",
    });
  });
});
