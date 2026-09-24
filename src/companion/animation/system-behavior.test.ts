import { describe, expect, it } from "vitest";
import type { SystemSnapshot } from "../../lib/types/domain";
import { systemBehaviorForSnapshot } from "./system-behavior";

function snapshot(
  status: SystemSnapshot["overall_status"],
  category: SystemSnapshot["primary_issue"] extends infer T
    ? T extends { category: infer C }
      ? C
      : never
    : never = "CPU",
): SystemSnapshot {
  return {
    timestamp_epoch_ms: 1,
    overall_status: status,
    cpu: { value: 10, unit: "%", state: "NORMAL", available: null, available_unit: null },
    memory: { value: 40, unit: "%", state: "NORMAL", available: 8, available_unit: "GB" },
    storage: { value: 50, unit: "%", state: "NORMAL", available: 100, available_unit: "GB" },
    battery: null,
    network: { download_mbps: 0, upload_mbps: 0 },
    thermal: null,
    primary_issue:
      status === "STRESSED" || status === "NEEDS_ATTENTION"
        ? {
            id: "test",
            category: category as any,
            severity: "HIGH",
            headline: "Test",
            explanation: "Test",
            culprit: null,
            confidence: "HIGH",
            culprit_confidence: null,
            recommended_action: null,
            started_at_epoch_ms: 1,
          }
        : null,
    secondary_issue_count: 0,
  };
}

describe("systemBehaviorForSnapshot", () => {
  it("keeps ordinary load in the non-alarming busy behavior", () => {
    expect(systemBehaviorForSnapshot(snapshot("BUSY"))).toEqual({
      behavior: "busy",
      source: "system",
    });
  });

  it("maps memory and thermal pressure to specific diagnostic behaviors", () => {
    expect(systemBehaviorForSnapshot(snapshot("STRESSED", "MEMORY")).behavior).toBe(
      "memory_pressure",
    );
    expect(systemBehaviorForSnapshot(snapshot("STRESSED", "THERMAL")).behavior).toBe("hot");
  });

  it("maps needs-attention to the highest-priority behavior", () => {
    expect(systemBehaviorForSnapshot(snapshot("NEEDS_ATTENTION"))).toEqual({
      behavior: "needs_attention",
      source: "critical",
    });
  });
});
