import type { CharacterAnimator } from "./state-machine";
import type { InputReactionEvent } from "../../lib/types/input";

export function applyInputReaction(
  animator: CharacterAnimator,
  event: InputReactionEvent,
): void {
  switch (event.kind) {
    case "TYPING_TAP_LEFT":
      animator.requestBehavior({ behavior: "typing_left", source: "input" });
      break;
    case "TYPING_TAP_RIGHT":
      animator.requestBehavior({ behavior: "typing_right", source: "input" });
      break;
    case "TYPING_FAST_START":
      animator.requestBehavior({ behavior: "typing_fast", source: "input" });
      break;
    case "TYPING_FAST_STOP":
      animator.releaseSource("input");
      break;
    case "MOUSE_LEFT":
      if (animator.currentBehavior() !== "typing_fast") {
        animator.requestBehavior({ behavior: "mouse_click", source: "input" });
      }
      break;
    case "MOUSE_RIGHT":
      if (animator.currentBehavior() !== "typing_fast") {
        animator.requestBehavior({ behavior: "right_click", source: "input" });
      }
      break;
    case "SCROLL":
      if (animator.currentBehavior() !== "typing_fast") {
        animator.requestBehavior({ behavior: "scroll", source: "input" });
      }
      break;
    case "IDLE_START":
      animator.requestBehavior({ behavior: "sleep", source: "personality" });
      break;
    case "IDLE_END":
      animator.releaseSource("personality");
      animator.requestBehavior({ behavior: "wake", source: "input" });
      break;
  }
}
