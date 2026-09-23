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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemIssue {
    pub id: String,
    pub category: IssueCategory,
    pub severity: ResourceState,
    pub headline: String,
    pub explanation: String,
    pub culprit: Option<ProcessSummary>,
    pub confidence: Confidence,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionPreferences {
    pub character: String,
    pub habitat: String,
    pub display_mode: DisplayMode,
    pub size: CompanionSize,
    pub interaction_level: InteractionLevel,
}

impl Default for CompanionPreferences {
    fn default() -> Self {
        Self {
            character: "BYTE".into(),
            habitat: "MEADOW".into(),
            display_mode: DisplayMode::Habitat,
            size: CompanionSize::Medium,
            interaction_level: InteractionLevel::Normal,
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
            schema_version: 1,
            companion: CompanionPreferences::default(),
            app: AppPreferences::default(),
        }
    }
}
