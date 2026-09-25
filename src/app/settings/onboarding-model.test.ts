import { describe, expect, it } from "vitest";
import type { AppPreferences } from "../../lib/types/domain";
import { createOnboardingAppDraft } from "./onboarding-model";

function preferences(
  onboardingCompleted: boolean,
  notificationsEnabled: boolean,
): AppPreferences {
  return {
    hide_in_fullscreen: true,
    hide_in_presentation: true,
    exclude_from_capture: true,
    hidden_foreground_apps: ["obs64"],
    sound_enabled: false,
    launch_at_startup: false,
    activity_history_enabled: true,
    system_monitoring_enabled: true,
    notifications_enabled: notificationsEnabled,
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
    onboarding_completed: onboardingCompleted,
  };
}

describe("onboarding settings draft", () => {
  it("defaults native notifications off only for genuine first run", () => {
    const source = preferences(false, true);
    const draft = createOnboardingAppDraft(source);

    expect(draft.notifications_enabled).toBe(false);
    expect(source.notifications_enabled).toBe(true);
  });

  it("preserves the existing notification choice when onboarding is rerun", () => {
    expect(
      createOnboardingAppDraft(preferences(true, true)).notifications_enabled,
    ).toBe(true);
    expect(
      createOnboardingAppDraft(preferences(true, false)).notifications_enabled,
    ).toBe(false);
  });

  it("does not share the mutable excluded-app list with saved preferences", () => {
    const source = preferences(true, true);
    const draft = createOnboardingAppDraft(source);

    draft.hidden_foreground_apps.push("powerpnt");
    expect(source.hidden_foreground_apps).toEqual(["obs64"]);
  });
});
