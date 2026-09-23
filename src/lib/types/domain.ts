export type SystemStatus = "CALM" | "BUSY" | "STRESSED" | "NEEDS_ATTENTION";
export type ResourceState = "NORMAL" | "ELEVATED" | "HIGH" | "CRITICAL" | "UNKNOWN";
export type IssueCategory = "CPU" | "MEMORY" | "THERMAL" | "BATTERY" | "STORAGE";

export interface ResourceSummary {
  value: number;
  unit: string;
  state: ResourceState;
  available: number | null;
  available_unit: string | null;
}

export interface BatterySummary {
  percent: number;
  charging: boolean;
  state: ResourceState;
}

export interface NetworkSummary {
  download_mbps: number;
  upload_mbps: number;
}

export interface ProcessSummary {
  name: string;
  pid: number | null;
  cpu_percent: number | null;
  memory_mb: number | null;
}

export type Confidence = "LOW" | "MEDIUM" | "HIGH";
export type RecommendedActionKind =
  | "OPEN_TASK_MANAGER"
  | "OPEN_STORAGE_SETTINGS"
  | "OPEN_BATTERY_SETTINGS"
  | "VIEW_DETAILS";

export interface RecommendedAction {
  kind: RecommendedActionKind;
  label: string;
}

export interface SystemIssue {
  id: string;
  category: IssueCategory;
  severity: ResourceState;
  headline: string;
  explanation: string;
  culprit: ProcessSummary | null;
  confidence: Confidence;
  culprit_confidence: Confidence | null;
  recommended_action: RecommendedAction | null;
  started_at_epoch_ms: number;
}

export interface SystemSnapshot {
  timestamp_epoch_ms: number;
  overall_status: SystemStatus;
  cpu: ResourceSummary;
  memory: ResourceSummary;
  storage: ResourceSummary;
  battery: BatterySummary | null;
  network: NetworkSummary;
  thermal: ResourceSummary | null;
  primary_issue: SystemIssue | null;
  secondary_issue_count: number;
}

export type DisplayMode = "HABITAT" | "PERCH" | "MINI" | "EDGE" | "TRAY";
export type CompanionSize = "SMALL" | "MEDIUM" | "LARGE";
export type InteractionLevel = "QUIET" | "NORMAL" | "PLAYFUL";

export interface CompanionPreferences {
  character: "BYTE" | "MOCHI" | "PIP" | "KIWI";
  habitat: "MEADOW" | "DESK" | "BEDROOM" | "SPACE" | "AQUARIUM" | "ROOFTOP";
  display_mode: DisplayMode;
  size: CompanionSize;
  interaction_level: InteractionLevel;
}

export interface AppPreferences {
  hide_in_fullscreen: boolean;
  sound_enabled: boolean;
  launch_at_startup: boolean;
  activity_history_enabled: boolean;
}

export interface ByteConfig {
  schema_version: number;
  companion: CompanionPreferences;
  app: AppPreferences;
}
