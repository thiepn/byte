import type {
  ActivityEvent,
  ActivityEventKind,
  ActivityTone,
  AppUsageSummary,
  Confidence,
  TrendPoint,
} from "../../lib/types/domain";

export type TrendMetric =
  | "cpu_percent"
  | "memory_percent"
  | "storage_percent"
  | "battery_percent"
  | "network_mbps"
  | "thermal_c";

export function sparklinePath(
  points: TrendPoint[],
  metric: TrendMetric,
  width = 160,
  height = 42,
): string {
  const values = points
    .map((point) => ({
      timestamp: point.timestamp_epoch_ms,
      value: point[metric],
    }))
    .filter(
      (point): point is { timestamp: number; value: number } =>
        typeof point.value === "number" && Number.isFinite(point.value),
    );

  if (values.length === 0) return "";

  const minTime = values[0].timestamp;
  const maxTime = values.at(-1)?.timestamp ?? minTime;
  const rawValues = values.map((point) => point.value);
  const minValue = Math.min(...rawValues);
  const maxValue = Math.max(...rawValues);
  const valueSpan = Math.max(1, maxValue - minValue);
  const timeSpan = Math.max(1, maxTime - minTime);

  return values
    .map((point, index) => {
      const x = ((point.timestamp - minTime) / timeSpan) * width;
      const y =
        height -
        3 -
        ((point.value - minValue) / valueSpan) * Math.max(1, height - 6);
      return `${index === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`;
    })
    .join(" ");
}

export function eventKindLabel(kind: ActivityEventKind): string {
  switch (kind) {
    case "ISSUE_OPENED":
      return "Issue";
    case "ISSUE_RESOLVED":
      return "Resolved";
    case "POWER":
      return "Power";
  }
}

export function eventToneClass(tone: ActivityTone): string {
  return tone.toLowerCase();
}

export function eventTime(timestampEpochMs: number): string {
  return new Date(timestampEpochMs).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function groupEventsByDay(
  events: ActivityEvent[],
): Array<{ day: string; events: ActivityEvent[] }> {
  const groups = new Map<string, ActivityEvent[]>();

  for (const event of events) {
    const key = new Date(event.timestamp_epoch_ms).toLocaleDateString([], {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
    const bucket = groups.get(key) ?? [];
    bucket.push(event);
    groups.set(key, bucket);
  }

  return [...groups.entries()].map(([day, values]) => ({
    day,
    events: values,
  }));
}


export type AppSort = "RELEVANCE" | "CPU" | "MEMORY";

export function sortApps(
  apps: AppUsageSummary[],
  sort: AppSort,
): AppUsageSummary[] {
  return [...apps].sort((left, right) => {
    if (sort === "CPU") {
      return (
        right.cpu_percent - left.cpu_percent ||
        right.memory_mb - left.memory_mb ||
        left.name.localeCompare(right.name)
      );
    }

    if (sort === "MEMORY") {
      return (
        right.memory_mb - left.memory_mb ||
        right.cpu_percent - left.cpu_percent ||
        left.name.localeCompare(right.name)
      );
    }

    const leftScore = Math.max(left.cpu_share, left.memory_share);
    const rightScore = Math.max(right.cpu_share, right.memory_share);
    return (
      rightScore - leftScore ||
      right.memory_mb - left.memory_mb ||
      left.name.localeCompare(right.name)
    );
  });
}

export function appSignalLabel(app: AppUsageSummary): string {
  if (app.cpu_confidence && app.memory_confidence) return "CPU + memory";
  if (app.cpu_confidence) return "CPU stands out";
  if (app.memory_confidence) return "Memory stands out";
  return "Context only";
}

export function confidenceLabel(confidence: Confidence | null): string {
  if (!confidence) return "No strong signal";
  return `${confidence[0]}${confidence.slice(1).toLowerCase()} confidence`;
}

export function formatMemoryMb(memoryMb: number): string {
  if (!Number.isFinite(memoryMb) || memoryMb < 0) return "—";
  if (memoryMb >= 1024) {
    const gb = memoryMb / 1024;
    return `${gb >= 10 ? Math.round(gb) : gb.toFixed(1)} GB`;
  }
  return `${Math.round(memoryMb)} MB`;
}

export function formatShare(share: number): string {
  if (!Number.isFinite(share) || share <= 0) return "<1%";
  return `${Math.max(1, Math.round(share * 100))}%`;
}
