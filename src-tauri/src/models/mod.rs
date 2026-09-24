use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemStatus {
    Calm,
    Busy,
    Stressed,
    NeedsAttention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceState {
    Normal,
    Elevated,
    High,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueCategory {
    Cpu,
    Memory,
    Thermal,
    Battery,
    Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    pub value: f32,
    pub unit: String,
    pub state: ResourceState,
    pub available: Option<f32>,
    pub available_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySummary {
    pub percent: f32,
    pub charging: bool,
    pub state: ResourceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub download_mbps: f32,
    pub upload_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSummary {
    pub name: String,
    pub pid: Option<u32>,
    pub cpu_percent: Option<f32>,
    pub memory_mb: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecommendedActionKind {
    OpenTaskManager,
    OpenStorageSettings,
    OpenBatterySettings,
    ViewDetails,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub kind: RecommendedActionKind,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIssue {
    pub id: String,
    pub category: IssueCategory,
    pub severity: ResourceState,
    pub headline: String,
    pub explanation: String,
    pub culprit: Option<ProcessSummary>,
    pub confidence: Confidence,
    pub culprit_confidence: Option<Confidence>,
    pub recommended_action: Option<RecommendedAction>,
    pub started_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp_epoch_ms: u64,
    pub overall_status: SystemStatus,
    pub cpu: ResourceSummary,
    pub memory: ResourceSummary,
    pub storage: ResourceSummary,
    pub battery: Option<BatterySummary>,
    pub network: NetworkSummary,
    pub thermal: Option<ResourceSummary>,
    pub primary_issue: Option<SystemIssue>,
    pub secondary_issue_count: u8,
}

impl SystemSnapshot {
    pub fn unavailable() -> Self {
        let unknown = || ResourceSummary {
            value: 0.0,
            unit: "%".into(),
            state: ResourceState::Unknown,
            available: None,
            available_unit: None,
        };

        Self {
            timestamp_epoch_ms: now_epoch_ms(),
            overall_status: SystemStatus::Calm,
            cpu: unknown(),
            memory: unknown(),
            storage: unknown(),
            battery: None,
            network: NetworkSummary {
                download_mbps: 0.0,
                upload_mbps: 0.0,
            },
            thermal: None,
            primary_issue: None,
            secondary_issue_count: 0,
        }
    }

    pub fn development_default() -> Self {
        Self {
            timestamp_epoch_ms: now_epoch_ms(),
            overall_status: SystemStatus::Calm,
            cpu: ResourceSummary {
                value: 18.0,
                unit: "%".into(),
                state: ResourceState::Normal,
                available: None,
                available_unit: None,
            },
            memory: ResourceSummary {
                value: 52.0,
                unit: "%".into(),
                state: ResourceState::Normal,
                available: Some(7.4),
                available_unit: Some("GB".into()),
            },
            storage: ResourceSummary {
                value: 61.0,
                unit: "%".into(),
                state: ResourceState::Normal,
                available: Some(287.0),
                available_unit: Some("GB".into()),
            },
            battery: Some(BatterySummary {
                percent: 82.0,
                charging: true,
                state: ResourceState::Normal,
            }),
            network: NetworkSummary {
                download_mbps: 0.4,
                upload_mbps: 0.1,
            },
            thermal: None,
            primary_issue: None,
            secondary_issue_count: 0,
        }
    }
}

pub fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisplayMode {
    Habitat,
    Perch,
    Mini,
    Edge,
    Tray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EdgeAnchor {
    Left,
    #[default]
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPlacement {
    pub monitor_name: Option<String>,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowPlacements {
    pub habitat: Option<SavedPlacement>,
    pub perch: Option<SavedPlacement>,
    pub mini: Option<SavedPlacement>,
    pub edge: Option<SavedPlacement>,
}

impl WindowPlacements {
    pub fn get(&self, mode: DisplayMode) -> Option<&SavedPlacement> {
        match mode {
            DisplayMode::Habitat => self.habitat.as_ref(),
            DisplayMode::Perch => self.perch.as_ref(),
            DisplayMode::Mini => self.mini.as_ref(),
            DisplayMode::Edge => self.edge.as_ref(),
            DisplayMode::Tray => None,
        }
    }

    pub fn set(&mut self, mode: DisplayMode, placement: SavedPlacement) {
        match mode {
            DisplayMode::Habitat => self.habitat = Some(placement),
            DisplayMode::Perch => self.perch = Some(placement),
            DisplayMode::Mini => self.mini = Some(placement),
            DisplayMode::Edge => self.edge = Some(placement),
            DisplayMode::Tray => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompanionSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InteractionLevel {
    Quiet,
    Normal,
    Playful,
}

fn none_selection() -> String {
    "none".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitatDecorationPreferences {
    #[serde(default = "none_selection")]
    pub large_background: String,
    #[serde(default = "none_selection")]
    pub wall_or_sky: String,
    #[serde(default = "none_selection")]
    pub surface_left: String,
    #[serde(default = "none_selection")]
    pub surface_right: String,
    #[serde(default = "none_selection")]
    pub small_prop: String,
    #[serde(default = "none_selection")]
    pub ambient: String,
}

impl Default for HabitatDecorationPreferences {
    fn default() -> Self {
        Self {
            large_background: none_selection(),
            wall_or_sky: none_selection(),
            surface_left: none_selection(),
            surface_right: none_selection(),
            small_prop: none_selection(),
            ambient: none_selection(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionCustomization {
    #[serde(default = "none_selection")]
    pub headwear: String,
    #[serde(default = "none_selection")]
    pub face_accessory: String,
    #[serde(default = "none_selection")]
    pub body_accessory: String,
    #[serde(default = "none_selection")]
    pub back_accessory: String,
    #[serde(default = "none_selection")]
    pub hand_prop: String,
    #[serde(default)]
    pub decorations: HabitatDecorationPreferences,
}

impl Default for CompanionCustomization {
    fn default() -> Self {
        Self {
            headwear: none_selection(),
            face_accessory: none_selection(),
            body_accessory: none_selection(),
            back_accessory: none_selection(),
            hand_prop: none_selection(),
            decorations: HabitatDecorationPreferences::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionPreferences {
    pub character: String,
    #[serde(default = "default_character_palette")]
    pub palette: String,
    pub habitat: String,
    pub display_mode: DisplayMode,
    pub size: CompanionSize,
    pub interaction_level: InteractionLevel,
    #[serde(default)]
    pub edge_anchor: EdgeAnchor,
    #[serde(default)]
    pub placements: WindowPlacements,
    #[serde(default)]
    pub customization: CompanionCustomization,
}

fn default_character_palette() -> String {
    "default".into()
}

impl Default for CompanionPreferences {
    fn default() -> Self {
        Self {
            character: "BYTE".into(),
            palette: default_character_palette(),
            habitat: "MEADOW".into(),
            display_mode: DisplayMode::Habitat,
            size: CompanionSize::Medium,
            interaction_level: InteractionLevel::Normal,
            edge_anchor: EdgeAnchor::Right,
            placements: WindowPlacements::default(),
            customization: CompanionCustomization::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub hide_in_fullscreen: bool,
    pub sound_enabled: bool,
    pub launch_at_startup: bool,
    pub activity_history_enabled: bool,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            hide_in_fullscreen: true,
            sound_enabled: false,
            launch_at_startup: false,
            activity_history_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteConfig {
    pub schema_version: u32,
    pub companion: CompanionPreferences,
    pub app: AppPreferences,
}

impl Default for ByteConfig {
    fn default() -> Self {
        Self {
            schema_version: 4,
            companion: CompanionPreferences::default(),
            app: AppPreferences::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WindowShellState {
    pub move_mode: bool,
    pub click_through: bool,
}
