import type { AppPreferences } from "../../lib/types/domain";

export function createOnboardingAppDraft(
  preferences: AppPreferences,
): AppPreferences {
  return {
    ...preferences,
    hidden_foreground_apps: [...preferences.hidden_foreground_apps],
    notifications_enabled: preferences.onboarding_completed
      ? preferences.notifications_enabled
      : false,
  };
}
