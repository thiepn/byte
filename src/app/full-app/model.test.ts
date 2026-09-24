import { describe, expect, it } from "vitest";
import type { ActivityEvent, TrendPoint } from "../../lib/types/domain";
import {
  eventKindLabel,
  groupEventsByDay,
  sparklinePath,
} from "./model";

function trend(time: number, cpu: number): TrendPoint {
  return {
    timestamp_epoch_ms: time,
    cpu_percent: cpu,
    memory_percent: 50,
    storage_percent: 60,
    battery_percent: 80,
    network_mbps: 1,
    thermal_c: null,
  };
}

describe("full application presentation model", () => {
  it("builds a bounded sparkline path from trend points", () => {
    const path = sparklinePath(
      [trend(0, 10), trend(15_000, 30), trend(30_000, 20)],
      "cpu_percent",
      100,
      30,
    );

    expect(path.startsWith("M 0.0")).toBe(true);
    expect(path).toContain("L 50.0");
    expect(path).toContain("L 100.0");
  });

  it("returns an empty sparkline when a sensor has no values", () => {
    const value = trend(0, 10);
    value.thermal_c = null;
    expect(sparklinePath([value], "thermal_c")).toBe("");
  });

  it("uses human category labels for meaningful events", () => {
    expect(eventKindLabel("ISSUE_OPENED")).toBe("Issue");
    expect(eventKindLabel("ISSUE_RESOLVED")).toBe("Resolved");
    expect(eventKindLabel("POWER")).toBe("Power");
  });

  it("groups activity events by local calendar day", () => {
    const events: ActivityEvent[] = [
      {
        id: 1,
        timestamp_epoch_ms: new Date("2026-09-24T10:00:00").getTime(),
        kind: "POWER",
        tone: "NORMAL",
        title: "Charging started",
        detail: "Battery is charging.",
      },
      {
        id: 2,
        timestamp_epoch_ms: new Date("2026-09-23T10:00:00").getTime(),
        kind: "ISSUE_RESOLVED",
        tone: "NORMAL",
        title: "Resolved",
        detail: "Issue cleared.",
      },
    ];

    expect(groupEventsByDay(events)).toHaveLength(2);
  });
});
