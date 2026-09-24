import type {
  DesktopAwarenessSnapshot,
  VisibilitySuppressionReason,
} from "../../lib/types/domain";

const REASON_LABELS: Record<VisibilitySuppressionReason, string> = {
  FULLSCREEN: "Fullscreen app",
  PRESENTATION: "Presentation mode",
  LOCKED: "Session locked",
  DISPLAY_SLEEP: "Display asleep",
  EXCLUDED_APP: "Excluded foreground app",
};

export function awarenessTitle(snapshot: DesktopAwarenessSnapshot | null): string {
  if (!snapshot) return "Checking desktop state…";
  if (!snapshot.suppressed) return "Byte is available on the desktop";
  return REASON_LABELS[snapshot.reason ?? "FULLSCREEN"];
}

export function awarenessDetail(snapshot: DesktopAwarenessSnapshot | null): string {
  if (!snapshot) return "Waiting for the native awareness service.";
  if (!snapshot.suppressed) {
    return snapshot.capture_exclusion_enabled
      ? "Screen-capture exclusion is active."
      : "Screen-capture exclusion is off.";
  }

  if (snapshot.reason === "EXCLUDED_APP" && snapshot.foreground_app) {
    return `Hidden while ${snapshot.foreground_app} is foreground.`;
  }

  switch (snapshot.reason) {
    case "FULLSCREEN":
      return "The companion and Quick Panel stay hidden until fullscreen ends.";
    case "PRESENTATION":
      return "The companion stays hidden while Windows reports presentation/busy mode.";
    case "LOCKED":
      return "Byte stays off the lock/secure desktop.";
    case "DISPLAY_SLEEP":
      return "Byte stays hidden while the console display is off.";
    default:
      return "Desktop awareness is currently suppressing the companion.";
  }
}

export function normalizeExcludedAppInput(value: string): string | null {
  const trimmed = value.trim();
  if (!trimmed || trimmed.length > 96 || /[\\/:]/.test(trimmed)) return null;
  const withoutExe = trimmed.replace(/\.exe$/i, "").trim().toLowerCase();
  return withoutExe || null;
}
