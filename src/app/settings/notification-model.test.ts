import { describe, expect, it } from "vitest";
import type { AppPreferences } from "../../lib/types/domain";
import {
  clearSnooze,
  permissionLabel,
  snoozeForHours,
  snoozeLabel,
  snoozeUntilTomorrow,
} from "./notification-model";

function preferences(): AppPreferences {
  return {
    hide_in_fullscreen: true,
    hide_in_presentation: true,
    exclude_from_capture: true,
    hidden_foreground_apps: [],
    sound_enabled: false,
    launch_at_startup: false,
    activity_history_enabled: true,
    system_monitoring_enabled: true,
    notifications_enabled: true,
    notification_memory_enabled: true,
    notification_thermal_enabled: true,
    notification_storage_enabled: true,
    notification_battery_enabled: true,
    notification_runaway_process_enabled: true,
    notification_quiet_mode: false,
    notification_snoozed_until_epoch_ms: null,
    reduce_motion: false,
    high_contrast: false,
    text_scale_percent: 100,
    update_channel: "STABLE",
    onboarding_completed: true,
  };
}

describe("smart notification settings model", () => {
  it("creates finite hour snoozes", () => {
    const next = snoozeForHours(preferences(), 4, 1_000);
    expect(next.notification_snoozed_until_epoch_ms).toBe(14_401_000);
  });

  it("snoozes until 08:00 on the next local day", () => {
    const now = new Date(2026, 8, 24, 18, 0, 0);
    const next = snoozeUntilTomorrow(preferences(), now);
    const date = new Date(next.notification_snoozed_until_epoch_ms!);
    expect(date.getDate()).toBe(25);
    expect(date.getHours()).toBe(8);
    expect(date.getMinutes()).toBe(0);
  });

  it("clears snooze without changing the master switch", () => {
    const base = preferences();
    base.notification_snoozed_until_epoch_ms = 123;
    const next = clearSnooze(base);
    expect(next.notification_snoozed_until_epoch_ms).toBeNull();
    expect(next.notifications_enabled).toBe(true);
  });

  it("hides expired snooze labels", () => {
    expect(snoozeLabel(1_000, 2_000)).toBeNull();
    expect(snoozeLabel(null, 2_000)).toBeNull();
  });

  it("uses plain Windows permission labels", () => {
    expect(permissionLabel("GRANTED")).toBe("Allowed by Windows");
    expect(permissionLabel("DENIED")).toBe("Blocked by Windows");
    expect(permissionLabel("PROMPT")).toBe("Permission not decided");
  });
});
