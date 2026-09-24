import { describe, expect, it } from "vitest";
import type { SystemSnapshot } from "../../lib/types/domain";
import {
  actionLabel,
  networkLabel,
  quickMetrics,
  relativeFreshness,
  statusTone,
  thermalLabel,
} from "./model";

function snapshot(): SystemSnapshot {
  return {
    timestamp_epoch_ms: 10_000,
    overall_status: "CALM",
    cpu: {
      value: 24.4,
      unit: "%",
      state: "NORMAL",
      available: null,
      available_unit: null,
    },
    memory: {
      value: 52,
      unit: "%",
      state: "NORMAL",
      available: 7.4,
      available_unit: "GB",
    },
    storage: {
      value: 61,
      unit: "%",
      state: "NORMAL",
      available: 287,
      available_unit: "GB",
    },
    battery: {
      percent: 82,
      charging: true,
      state: "NORMAL",
    },
    network: {
      download_mbps: 8.24,
      upload_mbps: 0.08,
    },
    thermal: {
      value: 62.5,
      unit: "°C",
      state: "NORMAL",
      available: null,
      available_unit: null,
    },
    primary_issue: null,
    secondary_issue_count: 0,
  };
}

describe("Quick Panel presentation model", () => {
  it("maps product states to calm semantic tones", () => {
    expect(statusTone("CALM")).toBe("normal");
    expect(statusTone("BUSY")).toBe("info");
    expect(statusTone("STRESSED")).toBe("warning");
    expect(statusTone("NEEDS_ATTENTION")).toBe("critical");
  });

  it("builds compact resource metrics without exposing raw sensor tables", () => {
    const metrics = quickMetrics(snapshot());
    expect(metrics.map((metric) => metric.id)).toEqual([
      "cpu",
      "memory",
      "storage",
      "battery",
    ]);
    expect(metrics[1].detail).toBe("7.4 GB free");
    expect(metrics[2].value).toBe("287 GB free");
  });

  it("formats lightweight network and thermal context", () => {
    expect(networkLabel(snapshot())).toBe("↓ 8.2 Mbps  ↑ 0.1 Mbps");
    expect(thermalLabel(snapshot())).toBe("63°C");
  });

  it("keeps recommended action labels fixed and narrow", () => {
    expect(actionLabel("OPEN_TASK_MANAGER")).toBe("Open Task Manager");
    expect(actionLabel("OPEN_STORAGE_SETTINGS")).toBe("Open Storage Settings");
    expect(actionLabel("OPEN_BATTERY_SETTINGS")).toBe("Open Battery Settings");
    expect(actionLabel("VIEW_DETAILS")).toBe("View details");
  });

  it("communicates cached snapshot age", () => {
    expect(relativeFreshness(10_000, 12_000)).toBe("Updated now");
    expect(relativeFreshness(10_000, 22_000)).toBe("Updated 12s ago");
    expect(relativeFreshness(10_000, 80_000)).toBe(
      "Snapshot is older than a minute",
    );
  });
});
