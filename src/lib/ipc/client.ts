import { invoke } from "@tauri-apps/api/core";
import type {
  ByteConfig,
  CompanionPreferences,
  CompanionSize,
  DisplayMode,
  EdgeAnchor,
  SystemSnapshot,
  WindowShellState,
} from "../types/domain";

const DEFAULT_CONFIG: ByteConfig = {
  schema_version: 2,
  companion: {
    character: "BYTE",
    habitat: "MEADOW",
    display_mode: "HABITAT",
    size: "MEDIUM",
    interaction_level: "NORMAL",
    edge_anchor: "RIGHT",
    placements: {
      habitat: null,
      perch: null,
      mini: null,
      edge: null,
    },
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
let browserShellState: WindowShellState = {
  move_mode: false,
  click_through: false,
};

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

export async function getWindowShellState(): Promise<WindowShellState> {
  if (!inTauri()) return clone(browserShellState);
  return invoke<WindowShellState>("get_window_shell_state");
}

export async function setDisplayMode(mode: DisplayMode): Promise<ByteConfig> {
  if (!inTauri()) {
    browserConfig.companion.display_mode = mode;
    return clone(browserConfig);
  }
  return invoke<ByteConfig>("set_display_mode", { mode });
}

export async function setCompanionSize(size: CompanionSize): Promise<ByteConfig> {
  if (!inTauri()) {
    browserConfig.companion.size = size;
    return clone(browserConfig);
  }
  return invoke<ByteConfig>("set_companion_size", { size });
}

export async function setEdgeAnchor(anchor: EdgeAnchor): Promise<ByteConfig> {
  if (!inTauri()) {
    browserConfig.companion.edge_anchor = anchor;
    return clone(browserConfig);
  }
  return invoke<ByteConfig>("set_edge_anchor", { anchor });
}

export async function beginMoveMode(): Promise<WindowShellState> {
  if (!inTauri()) {
    browserShellState = { move_mode: true, click_through: false };
    return clone(browserShellState);
  }
  return invoke<WindowShellState>("begin_move_mode");
}

export async function dragCompanion(): Promise<WindowShellState> {
  if (!inTauri()) {
    browserShellState.move_mode = false;
    return clone(browserShellState);
  }
  return invoke<WindowShellState>("drag_companion");
}

export async function finishMoveMode(): Promise<WindowShellState> {
  if (!inTauri()) {
    browserShellState.move_mode = false;
    return clone(browserShellState);
  }
  return invoke<WindowShellState>("finish_move_mode");
}

export async function setCompanionClickThrough(enabled: boolean): Promise<WindowShellState> {
  if (!inTauri()) {
    browserShellState = { move_mode: false, click_through: enabled };
    return clone(browserShellState);
  }
  return invoke<WindowShellState>("set_companion_click_through", { enabled });
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
