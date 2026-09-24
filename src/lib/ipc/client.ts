import { invoke } from "@tauri-apps/api/core";
import type {
  ActivitySnapshot,
  AppDiagnosticsSnapshot,
  ByteConfig,
  CollectionDiscoveryKind,
  CollectionSnapshot,
  CompanionPreferences,
  CompanionSize,
  DisplayMode,
  EdgeAnchor,
  RecommendedActionKind,
  SystemSnapshot,
  WindowShellState,
} from "../types/domain";

const DEFAULT_CONFIG: ByteConfig = {
  schema_version: 5,
  companion: {
    character: "BYTE",
    palette: "default",
    habitat: "MEADOW",
    display_mode: "HABITAT",
    size: "MEDIUM",
    interaction_level: "NORMAL",
    personality: "CURIOUS",
    edge_anchor: "RIGHT",
    placements: {
      habitat: null,
      perch: null,
      mini: null,
      edge: null,
    },
    customization: {
      headwear: "none",
      face_accessory: "none",
      body_accessory: "none",
      back_accessory: "none",
      hand_prop: "none",
      decorations: {
        large_background: "none",
        wall_or_sky: "none",
        surface_left: "none",
        surface_right: "none",
        small_prop: "none",
        ambient: "none",
      },
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

export async function getActivityHistory(): Promise<ActivitySnapshot> {
  if (!inTauri()) {
    return {
      events: [],
      trends: [
        {
          timestamp_epoch_ms: Date.now(),
          cpu_percent: MOCK_SNAPSHOT.cpu.value,
          memory_percent: MOCK_SNAPSHOT.memory.value,
          storage_percent: MOCK_SNAPSHOT.storage.value,
          battery_percent: MOCK_SNAPSHOT.battery?.percent ?? null,
          network_mbps:
            MOCK_SNAPSHOT.network.download_mbps + MOCK_SNAPSHOT.network.upload_mbps,
          thermal_c: MOCK_SNAPSHOT.thermal?.value ?? null,
        },
      ],
    };
  }
  return invoke<ActivitySnapshot>("get_activity_history");
}

export async function inspectApps(): Promise<AppDiagnosticsSnapshot> {
  if (!inTauri()) {
    return {
      timestamp_epoch_ms: Date.now(),
      apps: [
        {
          name: "Browser",
          process_count: 8,
          cpu_percent: 18,
          memory_mb: 1420,
          cpu_share: 0.42,
          memory_share: 0.38,
          cpu_confidence: "MEDIUM",
          memory_confidence: "HIGH",
        },
        {
          name: "Editor",
          process_count: 4,
          cpu_percent: 7,
          memory_mb: 860,
          cpu_share: 0.16,
          memory_share: 0.23,
          cpu_confidence: null,
          memory_confidence: "MEDIUM",
        },
      ],
      cpu_leader: {
        name: "Browser",
        confidence: "MEDIUM",
        share: 0.42,
        value: 18,
      },
      memory_leader: {
        name: "Browser",
        confidence: "HIGH",
        share: 0.38,
        value: 1420,
      },
    };
  }
  return invoke<AppDiagnosticsSnapshot>("inspect_apps");
}

export async function getCollection(): Promise<CollectionSnapshot> {
  if (!inTauri()) {
    return {
      first_seen_epoch_ms: Date.now(),
      typing_events: 0,
      charging_sessions: 0,
      network_moments: 0,
      unlocked_ids: [],
      items: [],
    };
  }
  return invoke<CollectionSnapshot>("get_collection");
}

export async function recordCollectionDiscovery(
  discovery: CollectionDiscoveryKind,
): Promise<CollectionSnapshot> {
  if (!inTauri()) return getCollection();
  return invoke<CollectionSnapshot>("record_collection_discovery", { discovery });
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

export async function executeRecommendedAction(
  action: RecommendedActionKind,
): Promise<void> {
  if (!inTauri()) return;
  await invoke("execute_recommended_action", { action });
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
