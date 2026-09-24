import type { SystemSnapshot } from "../../lib/types/domain";
import type { BehaviorId, BehaviorSource } from "./types";

export interface SystemBehavior {
  behavior: BehaviorId;
  source: BehaviorSource;
}

export function systemBehaviorForSnapshot(snapshot: SystemSnapshot): SystemBehavior {
  if (snapshot.overall_status === "NEEDS_ATTENTION") {
    return { behavior: "needs_attention", source: "critical" };
  }

  if (snapshot.overall_status === "STRESSED") {
    switch (snapshot.primary_issue?.category) {
      case "MEMORY":
        return { behavior: "memory_pressure", source: "diagnostic" };
      case "THERMAL":
        return { behavior: "hot", source: "diagnostic" };
      case "BATTERY":
        return { behavior: "low_battery", source: "diagnostic" };
      default:
        return { behavior: "stressed", source: "diagnostic" };
    }
  }

  if (snapshot.overall_status === "BUSY") {
    return { behavior: "busy", source: "system" };
  }

  return { behavior: "idle", source: "idle" };
}
