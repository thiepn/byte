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
export type Personality = "CHILL" | "CURIOUS" | "ENERGETIC";
export type EdgeAnchor = "LEFT" | "RIGHT";

export interface SavedPlacement {
  monitor_name: string | null;
  x: number;
  y: number;
}

export interface WindowPlacements {
  habitat: SavedPlacement | null;
  perch: SavedPlacement | null;
  mini: SavedPlacement | null;
  edge: SavedPlacement | null;
}

export interface HabitatDecorationPreferences {
  large_background: string;
  wall_or_sky: string;
  surface_left: string;
  surface_right: string;
  small_prop: string;
  ambient: string;
}

export interface CompanionCustomization {
  headwear: string;
  face_accessory: string;
  body_accessory: string;
  back_accessory: string;
  hand_prop: string;
  decorations: HabitatDecorationPreferences;
}

export interface CompanionPreferences {
  character: "BYTE" | "MOCHI" | "PIP" | "KIWI";
  palette: string;
  habitat: "MEADOW" | "DESK" | "BEDROOM" | "SPACE" | "AQUARIUM" | "ROOFTOP";
  display_mode: DisplayMode;
  size: CompanionSize;
  interaction_level: InteractionLevel;
  personality: Personality;
  edge_anchor: EdgeAnchor;
  placements: WindowPlacements;
  customization: CompanionCustomization;
}

export interface WindowShellState {
  move_mode: boolean;
  click_through: boolean;
}

export interface AppPreferences {
  hide_in_fullscreen: boolean;
  sound_enabled: boolean;
  launch_at_startup: boolean;
  activity_history_enabled: boolean;
  system_monitoring_enabled: boolean;
  notifications_enabled: boolean;
  reduce_motion: boolean;
  high_contrast: boolean;
  text_scale_percent: 100 | 110 | 125;
  onboarding_completed: boolean;
}

export interface ByteConfig {
  schema_version: number;
  companion: CompanionPreferences;
  app: AppPreferences;
}


export type ActivityEventKind = "ISSUE_OPENED" | "ISSUE_RESOLVED" | "POWER";
export type ActivityTone = "NORMAL" | "INFO" | "WARNING" | "CRITICAL";

export interface ActivityEvent {
  id: number;
  timestamp_epoch_ms: number;
  kind: ActivityEventKind;
  tone: ActivityTone;
  title: string;
  detail: string;
}

export interface TrendPoint {
  timestamp_epoch_ms: number;
  cpu_percent: number;
  memory_percent: number;
  storage_percent: number;
  battery_percent: number | null;
  network_mbps: number;
  thermal_c: number | null;
}

export interface ActivitySnapshot {
  events: ActivityEvent[];
  trends: TrendPoint[];
}


export interface AppUsageSummary {
  name: string;
  process_count: number;
  cpu_percent: number;
  memory_mb: number;
  cpu_share: number;
  memory_share: number;
  cpu_confidence: Confidence | null;
  memory_confidence: Confidence | null;
}

export interface AppAttribution {
  name: string;
  confidence: Confidence;
  share: number;
  value: number;
}

export interface AppDiagnosticsSnapshot {
  timestamp_epoch_ms: number;
  apps: AppUsageSummary[];
  cpu_leader: AppAttribution | null;
  memory_leader: AppAttribution | null;
}


export type CollectionItemKind = "COSMETIC" | "DECORATION" | "PALETTE" | "IDLE";
export type CollectionDiscoveryKind = "RARE_A" | "RARE_B";

export interface CollectionItemProgress {
  unlock_id: string;
  item_id: string;
  kind: CollectionItemKind;
  title: string;
  description: string;
  condition: string;
  unlocked: boolean;
  progress_current: number | null;
  progress_target: number | null;
}

export interface CollectionSnapshot {
  first_seen_epoch_ms: number;
  typing_events: number;
  charging_sessions: number;
  network_moments: number;
  unlocked_ids: string[];
  items: CollectionItemProgress[];
}
