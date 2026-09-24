import type {
  AppPreferences,
  NotificationPermissionState,
} from "../../lib/types/domain";

export type NotificationCategoryPreference =
  | "notification_memory_enabled"
  | "notification_thermal_enabled"
  | "notification_storage_enabled"
  | "notification_battery_enabled"
  | "notification_runaway_process_enabled";

export const NOTIFICATION_CATEGORIES: Array<{
  key: NotificationCategoryPreference;
  title: string;
  description: string;
}> = [
  {
    key: "notification_memory_enabled",
    title: "Critical memory",
    description: "Only sustained critical memory pressure.",
  },
  {
    key: "notification_thermal_enabled",
    title: "Serious thermal pressure",
    description: "Sustained high or critical temperature readings.",
  },
  {
    key: "notification_storage_enabled",
    title: "Critically low storage",
    description: "Only when monitored storage is almost full.",
  },
  {
    key: "notification_battery_enabled",
    title: "Critically low battery",
    description: "Only when the battery is almost empty and not charging.",
  },
  {
    key: "notification_runaway_process_enabled",
    title: "Prolonged runaway app",
    description: "Requires a confident culprit and about 10 minutes of critical CPU pressure.",
  },
];

export function snoozeForHours(
  preferences: AppPreferences,
  hours: number,
  nowEpochMs = Date.now(),
): AppPreferences {
  return {
    ...preferences,
    notification_snoozed_until_epoch_ms:
      nowEpochMs + Math.max(1, hours) * 60 * 60 * 1000,
  };
}

export function snoozeUntilTomorrow(
  preferences: AppPreferences,
  now = new Date(),
): AppPreferences {
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);
  tomorrow.setHours(8, 0, 0, 0);
  return {
    ...preferences,
    notification_snoozed_until_epoch_ms: tomorrow.getTime(),
  };
}

export function clearSnooze(preferences: AppPreferences): AppPreferences {
  return {
    ...preferences,
    notification_snoozed_until_epoch_ms: null,
  };
}

export function snoozeLabel(
  untilEpochMs: number | null,
  nowEpochMs = Date.now(),
): string | null {
  if (untilEpochMs == null || untilEpochMs <= nowEpochMs) return null;
  const date = new Date(untilEpochMs);
  return `Snoozed until ${date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  })}`;
}

export function permissionLabel(
  state: NotificationPermissionState | null,
): string {
  if (state === "GRANTED") return "Allowed by Windows";
  if (state === "DENIED") return "Blocked by Windows";
  if (state === "PROMPT") return "Permission not decided";
  return "Checking Windows permission…";
}
