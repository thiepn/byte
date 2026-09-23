import { invoke } from "@tauri-apps/api/core";
import type { ByteConfig, CompanionPreferences, SystemSnapshot } from "../types/domain";

const DEFAULT_CONFIG: ByteConfig = {
  schema_version: 1,
  companion: {
    character: "BYTE",
    habitat: "MEADOW",
    display_mode: "HABITAT",
    size: "MEDIUM",
    interaction_level: "NORMAL",
  },
  app: {
    hide_in_fullscreen: true,
    sound_enabled: false,
    launch_at_startup: false,
    activity_history_enabled: true,
  },
};

const MOCK_SNAPSHOT: SystemSnapshot = {
  timestamp_epoch_ms: Date.now(),
  overall_status: "CALM",
  cpu: { value: 18, unit: "%", state: "NORMAL", available: null, available_unit: null },
  memory: { value: 52, unit: "%", state: "NORMAL", available: 7.4, available_unit: "GB" },
  storage: { value: 61, unit: "%", state: "NORMAL", available: 287, available_unit: "GB" },
  battery: { percent: 82, charging: true, state: "NORMAL" },
  network: { download_mbps: 0.4, upload_mbps: 0.1 },
  thermal: null,
  primary_issue: null,
  secondary_issue_count: 0,
};

let browserConfig = clone(DEFAULT_CONFIG);

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function getSnapshot(): Promise<SystemSnapshot> {
  if (!inTauri()) return { ...clone(MOCK_SNAPSHOT), timestamp_epoch_ms: Date.now() };
  return invoke<SystemSnapshot>("get_snapshot");
}

export async function getPreferences(): Promise<ByteConfig> {
  if (!inTauri()) return clone(browserConfig);
  return invoke<ByteConfig>("get_preferences");
}

export async function updateCompanionPreferences(
  preferences: CompanionPreferences,
): Promise<ByteConfig> {
  if (!inTauri()) {
    browserConfig = { ...browserConfig, companion: clone(preferences) };
    return clone(browserConfig);
  }
  return invoke<ByteConfig>("update_companion_preferences", { preferences });
}

export async function showMainWindow(): Promise<void> {
  if (inTauri()) await invoke("show_main_window");
}

export async function showQuickPanel(): Promise<void> {
  if (inTauri()) await invoke("show_quick_panel");
}

export async function hideQuickPanel(): Promise<void> {
  if (inTauri()) await invoke("hide_quick_panel");
}
