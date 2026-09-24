import { describe, expect, it } from "vitest";
import type { SystemSnapshot } from "../../lib/types/domain";
import { habitatReactionsForSnapshot } from "./reactions";

function snapshot(): SystemSnapshot {
  return {
    timestamp_epoch_ms: 1,
    overall_status: "CALM",
    cpu: { value: 20, unit: "%", state: "NORMAL", available: null, available_unit: null },
    memory: { value: 45, unit: "%", state: "NORMAL", available: 8, available_unit: "GB" },
    storage: { value: 40, unit: "%", state: "NORMAL", available: 200, available_unit: "GB" },
    battery: { percent: 80, charging: false, state: "NORMAL" },
    network: { download_mbps: 0, upload_mbps: 0 },
    thermal: null,
    primary_issue: null,
    secondary_issue_count: 0,
  };
}

describe("habitatReactionsForSnapshot", () => {
  it("keeps calm systems visually quiet", () => {
    const reactions = habitatReactionsForSnapshot(snapshot());
    expect(reactions.BUSY).toBe(0);
    expect(reactions.MEMORY_PRESSURE).toBe(0);
    expect(reactions.THERMAL).toBe(0);
  });

  it("treats network activity as playful intensity rather than a warning", () => {
    const value = snapshot();
    value.network.download_mbps = 80;
    const reactions = habitatReactionsForSnapshot(value);
    expect(reactions.NETWORK).toBeGreaterThan(0);
    expect(reactions.NETWORK).toBeLessThanOrEqual(1);
  });

  it("maps charging and low battery separately", () => {
    const charging = snapshot();
    charging.battery = { percent: 35, charging: true, state: "NORMAL" };
    expect(habitatReactionsForSnapshot(charging).CHARGING).toBe(1);
    expect(habitatReactionsForSnapshot(charging).LOW_BATTERY).toBe(0);

    const low = snapshot();
    low.battery = { percent: 5, charging: false, state: "CRITICAL" };
    expect(habitatReactionsForSnapshot(low).LOW_BATTERY).toBe(1);
    expect(habitatReactionsForSnapshot(low).CHARGING).toBe(0);
  });

  it("uses the primary diagnostic category for specific pressure reactions", () => {
    const value = snapshot();
    value.overall_status = "STRESSED";
    value.primary_issue = {
      id: "memory-pressure",
      category: "MEMORY",
      severity: "HIGH",
      headline: "Memory is getting tight",
      explanation: "Test",
      culprit: null,
      confidence: "HIGH",
      culprit_confidence: null,
      recommended_action: null,
      started_at_epoch_ms: 1,
    };

    expect(habitatReactionsForSnapshot(value).MEMORY_PRESSURE).toBe(0.65);
  });
});
