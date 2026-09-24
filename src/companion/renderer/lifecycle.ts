import type { LifecycleState } from "../../lib/types/domain";

export function lifecycleSuspendsVisuals(state: LifecycleState): boolean {
  return (
    state === "LOCKED" ||
    state === "DISPLAY_SLEEP" ||
    state === "SYSTEM_SLEEP" ||
    state === "SHUTTING_DOWN"
  );
}
