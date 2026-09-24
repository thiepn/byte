import type {
  RecommendedActionKind,
  ResourceState,
  SystemSnapshot,
  SystemStatus,
} from "../../lib/types/domain";

export interface QuickMetric {
  id: "cpu" | "memory" | "storage" | "battery";
  label: string;
  value: string;
  detail: string;
  percent: number;
  state: ResourceState;
}

export type StatusTone = "normal" | "info" | "warning" | "critical";

export function statusTone(status: SystemStatus): StatusTone {
  if (status === "NEEDS_ATTENTION") return "critical";
  if (status === "STRESSED") return "warning";
  if (status === "BUSY") return "info";
  return "normal";
}

export function quickMetrics(snapshot: SystemSnapshot): QuickMetric[] {
  const metrics: QuickMetric[] = [
    {
      id: "cpu",
      label: "Processor",
      value: `${Math.round(snapshot.cpu.value)}${snapshot.cpu.unit}`,
      detail: resourceDetail(snapshot.cpu.state),
      percent: clampPercent(snapshot.cpu.value),
      state: snapshot.cpu.state,
    },
    {
      id: "memory",
      label: "Memory",
      value: `${Math.round(snapshot.memory.value)}${snapshot.memory.unit}`,
      detail:
        snapshot.memory.available == null
          ? resourceDetail(snapshot.memory.state)
          : `${formatAvailable(snapshot.memory.available, snapshot.memory.available_unit)} free`,
      percent: clampPercent(snapshot.memory.value),
      state: snapshot.memory.state,
    },
    {
      id: "storage",
      label: "Storage",
      value:
        snapshot.storage.available == null
          ? `${Math.round(snapshot.storage.value)}${snapshot.storage.unit}`
          : `${formatAvailable(snapshot.storage.available, snapshot.storage.available_unit)} free`,
      detail: resourceDetail(snapshot.storage.state),
      percent: clampPercent(snapshot.storage.value),
      state: snapshot.storage.state,
    },
  ];

  if (snapshot.battery) {
    metrics.push({
      id: "battery",
      label: "Battery",
      value: `${Math.round(snapshot.battery.percent)}%`,
      detail: snapshot.battery.charging
        ? "Charging"
        : resourceDetail(snapshot.battery.state),
      percent: clampPercent(snapshot.battery.percent),
      state: snapshot.battery.state,
    });
  }

  return metrics;
}

export function networkLabel(snapshot: SystemSnapshot): string {
  const down = formatRate(snapshot.network.download_mbps);
  const up = formatRate(snapshot.network.upload_mbps);
  return `↓ ${down}  ↑ ${up}`;
}

export function thermalLabel(snapshot: SystemSnapshot): string | null {
  if (!snapshot.thermal || snapshot.thermal.state === "UNKNOWN") return null;
  return `${Math.round(snapshot.thermal.value)}${snapshot.thermal.unit}`;
}

export function actionLabel(action: RecommendedActionKind): string {
  switch (action) {
    case "OPEN_TASK_MANAGER":
      return "Open Task Manager";
    case "OPEN_STORAGE_SETTINGS":
      return "Open Storage Settings";
    case "OPEN_BATTERY_SETTINGS":
      return "Open Battery Settings";
    case "VIEW_DETAILS":
      return "View details";
  }
}

export function relativeFreshness(
  timestampEpochMs: number,
  nowEpochMs = Date.now(),
): string {
  const ageSeconds = Math.max(
    0,
    Math.round((nowEpochMs - timestampEpochMs) / 1000),
  );
  if (ageSeconds < 4) return "Updated now";
  if (ageSeconds < 60) return `Updated ${ageSeconds}s ago`;
  return "Snapshot is older than a minute";
}

function resourceDetail(state: ResourceState): string {
  switch (state) {
    case "NORMAL":
      return "Normal";
    case "ELEVATED":
      return "Elevated";
    case "HIGH":
      return "High";
    case "CRITICAL":
      return "Critical";
    case "UNKNOWN":
      return "Unavailable";
  }
}

function formatAvailable(value: number, unit: string | null): string {
  const rounded = value >= 100 ? Math.round(value) : Math.round(value * 10) / 10;
  return `${rounded} ${unit ?? ""}`.trim();
}

function formatRate(value: number): string {
  if (!Number.isFinite(value) || value < 0.05) return "0 Mbps";
  if (value < 10) return `${value.toFixed(1)} Mbps`;
  return `${Math.round(value)} Mbps`;
}

function clampPercent(value: number): number {
  return Math.min(100, Math.max(0, Number.isFinite(value) ? value : 0));
}
